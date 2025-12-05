use std::{collections::HashMap, path::PathBuf, process::Stdio, str::FromStr};

use protocol::{MainToRuntime, RuntimeToMain};
use tauri::State;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{unix::OwnedWriteHalf, UnixListener},
    process::Child,
    sync::RwLock,
    task::JoinHandle,
};
use zenoh::session::ZenohId;

// ============================================================================
// Modules
// ============================================================================

pub mod logs;
pub mod protocol;
pub mod sandbox;

use logs::{LogEntry, LogStorage};
use sandbox::ZenohConfig;

// ============================================================================
// State management for Zenoh runtimes
// ============================================================================

/// Information about a running runtime process
struct RuntimeProcess {
    /// The configuration used to start the runtime
    config: ZenohConfig,
    /// The child process handle
    child: Child,
    /// The write half of the UDS socket for sending commands
    socket_writer: OwnedWriteHalf,
    /// Task handle for log receiving
    log_receiver: JoinHandle<()>,
}

/// Holds all active Zenoh runtime processes
pub struct ZenohRuntimes {
    runtimes: RwLock<HashMap<ZenohId, RuntimeProcess>>,
    /// Directory for UDS sockets
    socket_dir: PathBuf,
}

impl ZenohRuntimes {
    pub fn new(socket_dir: PathBuf) -> Self {
        // Ensure socket directory exists
        std::fs::create_dir_all(&socket_dir).ok();

        Self {
            runtimes: RwLock::new(HashMap::new()),
            socket_dir,
        }
    }
}

impl Default for ZenohRuntimes {
    fn default() -> Self {
        let socket_dir = std::env::temp_dir().join("zenoh_sandbox");
        Self::new(socket_dir)
    }
}

// ============================================================================
// Tauri commands
// ============================================================================

/// Create a new Zenoh runtime with the given configuration.
/// Returns the ZenohId as a string on success.
#[tauri::command]
async fn zenoh_runtime_start(
    config: ZenohConfig,
    runtimes_state: State<'_, ZenohRuntimes>,
    logs_state: State<'_, LogStorage>,
) -> Result<String, String> {
    // Create a unique socket path
    let socket_path = runtimes_state
        .socket_dir
        .join(format!("runtime_{}.sock", uuid::Uuid::new_v4()));

    // Create UDS listener
    let listener = UnixListener::bind(&socket_path)
        .map_err(|e| format!("Failed to create UDS listener: {}", e))?;

    // Get the path to the runtime binary
    let runtime_binary = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe path: {}", e))?
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?
        .join(if cfg!(target_os = "windows") {
            "zenoh_runtime.exe"
        } else {
            "zenoh_runtime"
        });

    // Check if binary exists
    if !runtime_binary.exists() {
        return Err(format!(
            "Runtime binary not found at: {}. Did you run 'cargo build --bins'?",
            runtime_binary.display()
        ));
    }

    eprintln!("Starting runtime binary: {}", runtime_binary.display());

    // Spawn the runtime process
    let mut child = tokio::process::Command::new(&runtime_binary)
        .arg(socket_path.to_string_lossy().to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn runtime process: {} (path: {})", e, runtime_binary.display()))?;

    eprintln!("Runtime process spawned with PID: {:?}", child.id());

    // Accept connection from the runtime process
    eprintln!("Waiting for runtime to connect...");
    let (socket, _) = tokio::time::timeout(std::time::Duration::from_secs(10), listener.accept())
        .await
        .map_err(|_| {
            let _ = child.kill();
            "Timeout waiting for runtime to connect (10s). Check stderr output.".to_string()
        })?
        .map_err(|e| {
            let _ = child.kill();
            format!("Failed to accept connection: {}", e)
        })?;

    eprintln!("Runtime connected successfully");

    // Send Start message
    let start_msg = MainToRuntime::Start(config.clone());
    let msg_json = serde_json::to_string(&start_msg)
        .map_err(|e| format!("Failed to serialize start message: {}", e))?;

    let (reader, mut writer) = socket.into_split();
    writer
        .write_all(format!("{}\n", msg_json).as_bytes())
        .await
        .map_err(|e| format!("Failed to send start message: {}", e))?;
    writer
        .flush()
        .await
        .map_err(|e| format!("Failed to flush socket: {}", e))?;

    // Receive Started response
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let response: RuntimeToMain = serde_json::from_str(&line)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let zid = match response {
        RuntimeToMain::Started(zid_str) => {
            ZenohId::from_str(&zid_str).map_err(|e| format!("Invalid ZenohId: {}", e))?
        }
        RuntimeToMain::StartError(err) => {
            // Kill the child process
            let _ = child.kill().await;
            return Err(err);
        }
        _ => {
            let _ = child.kill().await;
            return Err("Unexpected response from runtime".to_string());
        }
    };

    // Spawn log receiver task (reader continues to receive logs)
    let logs_storage = logs_state.inner().clone();
    let zid_clone = zid;

    let log_receiver = tokio::spawn(async move {
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Socket closed
                Ok(_) => {
                    if let Ok(msg) = serde_json::from_str::<RuntimeToMain>(&line) {
                        if let RuntimeToMain::Log(entry) = msg {
                            logs_storage.add_log(zid_clone, entry);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Store the runtime process (keep writer for sending Stop messages)
    {
        let mut runtimes = runtimes_state.runtimes.write().await;
        runtimes.insert(
            zid,
            RuntimeProcess {
                config,
                child,
                socket_writer: writer,
                log_receiver,
            },
        );
    }

    // Clean up socket file
    let _ = tokio::fs::remove_file(&socket_path).await;

    Ok(zid.to_string())
}

/// stop (close) a Zenoh runtime by its ZenohId string.
#[tauri::command]
async fn zenoh_runtime_stop(
    zid: String,
    runtimes_state: State<'_, ZenohRuntimes>,
    logs_state: State<'_, LogStorage>,
) -> Result<(), String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Remove from state and get the runtime process
    let mut runtime_process = {
        let mut runtimes = runtimes_state.runtimes.write().await;
        runtimes
            .remove(&zenoh_id)
            .ok_or_else(|| format!("Zenoh runtime '{}' not found", zid))?
    };

    // Send Stop message
    let stop_msg = MainToRuntime::Stop;
    if let Ok(msg_json) = serde_json::to_string(&stop_msg) {
        let _ = runtime_process
            .socket_writer
            .write_all(format!("{}\n", msg_json).as_bytes())
            .await;
        let _ = runtime_process.socket_writer.flush().await;
    }

    // Close the socket (this will cause the child process to exit)
    drop(runtime_process.socket_writer);

    // Wait for the child process to exit
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        runtime_process.child.wait(),
    )
    .await;

    // Kill the child process if it's still running
    let _ = runtime_process.child.kill().await;

    // Abort the log receiver task
    runtime_process.log_receiver.abort();

    // Clear logs for this runtime
    logs_state.clear_logs(&zenoh_id);

    Ok(())
}

/// List all active Zenoh runtime ZenohIds as strings.
#[tauri::command]
async fn zenoh_runtime_list(state: State<'_, ZenohRuntimes>) -> Result<Vec<String>, String> {
    let runtimes = state.runtimes.read().await;
    let zids: Vec<String> = runtimes.keys().map(|zid| zid.to_string()).collect();
    Ok(zids)
}

/// Get the configuration of a Zenoh runtime by its ZenohId string.
#[tauri::command]
async fn zenoh_runtime_config(
    zid: String,
    state: State<'_, ZenohRuntimes>,
) -> Result<ZenohConfig, String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Get the config from state
    let runtimes = state.runtimes.read().await;
    let runtime_process = runtimes
        .get(&zenoh_id)
        .ok_or_else(|| format!("Zenoh runtime '{}' not found", zid))?;

    Ok(runtime_process.config.clone())
}

/// Get a page of logs from a specific zenoh runtime.
/// Page 0 returns the most recent logs.
#[tauri::command]
async fn zenoh_runtime_log(
    zid: String,
    page: usize,
    state: State<'_, LogStorage>,
) -> Result<Vec<LogEntry>, String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    Ok(state.get_page(&zenoh_id, page))
}

// ============================================================================
// Tauri application entry point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize log storage
    let log_storage = LogStorage::default();

    // Initialize runtime manager
    let runtimes = ZenohRuntimes::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(runtimes)
        .manage(log_storage)
        .invoke_handler(tauri::generate_handler![
            zenoh_runtime_start,
            zenoh_runtime_stop,
            zenoh_runtime_list,
            zenoh_runtime_config,
            zenoh_runtime_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
