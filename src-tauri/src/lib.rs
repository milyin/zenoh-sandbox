use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    path::PathBuf,
    process::Stdio,
    str::FromStr,
};

use protocol::{MainToRuntime, RuntimeToMain};
use tauri::State;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
    process::Child,
    sync::{RwLock, mpsc, oneshot},
    task::JoinHandle,
};
use zenoh::config::Config;
use zenoh::session::ZenohId;

// ============================================================================
// Modules
// ============================================================================

pub mod logs;
pub mod protocol;
pub mod ts;

use logs::{LogEntry, LogStorage};

use crate::ts::config::{ZenohConfigEdit, ZenohConfigJson};

// ============================================================================
// State management for Zenoh runtimes
// ============================================================================

/// Request type for communication with the runtime background task
enum RuntimeRequest {
    /// Request to get the config, with a oneshot channel for the response
    GetConfig(oneshot::Sender<Config>),
    /// Request to stop the runtime
    Stop(oneshot::Sender<()>),
}

/// Information about a running runtime process
struct RuntimeProcess {
    /// The original sandbox configuration
    sandbox_config: ZenohConfigJson,
    /// The child process handle
    child: Child,
    /// Task handle for log receiving and request handling
    receiver_task: JoinHandle<()>,
    /// Channel to send requests to the receiver task
    request_tx: mpsc::Sender<RuntimeRequest>,
}

/// Holds all active Zenoh runtime processes
pub struct ZenohRuntimes {
    runtimes: RwLock<HashMap<ZenohId, RuntimeProcess>>,
    /// Port tracker for ensuring unique port assignments
    port_tracker: RwLock<HashSet<u16>>,
    /// Directory for UDS sockets
    socket_dir: PathBuf,
    /// Directory for runtime logs
    log_dir: PathBuf,
}

impl ZenohRuntimes {
    pub fn new(socket_dir: PathBuf, log_dir: PathBuf) -> Self {
        // Ensure socket directory exists
        std::fs::create_dir_all(&socket_dir).ok();

        // Ensure log directory exists
        std::fs::create_dir_all(&log_dir).ok();

        Self {
            runtimes: RwLock::new(HashMap::new()),
            port_tracker: RwLock::new(HashSet::new()),
            socket_dir,
            log_dir,
        }
    }

    /// Allocate a free port
    /// Returns the next available port starting from 10000
    pub async fn allocate_port(&self) -> u16 {
        let mut tracker = self.port_tracker.write().await;
        let mut port = 10000;
        while tracker.contains(&port) {
            port += 1;
        }
        tracker.insert(port);
        port
    }

    /// Release a port back to the pool
    pub async fn release_port(&self, port: u16) {
        let mut tracker = self.port_tracker.write().await;
        tracker.remove(&port);
    }
}

impl Default for ZenohRuntimes {
    fn default() -> Self {
        let socket_dir = std::env::temp_dir().join("zenoh_sandbox");

        // Use environment variable if set, otherwise default to temp dir
        let log_dir = std::env::var("ZENOH_SANDBOX_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("zenoh_sandbox/logs"));

        Self::new(socket_dir, log_dir)
    }
}

// ============================================================================
// Tauri commands
// ============================================================================

/// Verify raw JSON as a valid zenoh::Config and extract editable fields
/// Returns both the validated config and the editable fields
#[tauri::command]
async fn verify_zenoh_config_json(
    json: serde_json::Value,
) -> Result<(ZenohConfigEdit, ZenohConfigJson), String> {
    // Validate and create ZenohConfigJson
    let config_json = ZenohConfigJson::from_json(json)?;

    // Convert to zenoh::Config to extract fields
    let zenoh_config =
        serde_json::from_value::<zenoh::config::Config>(config_json.as_json().clone())
            .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Extract editable fields
    let edit_fields = ZenohConfigEdit::from_config(&zenoh_config);

    Ok((edit_fields, config_json))
}

/// Apply editable fields to an existing validated config
/// Returns a new validated config with the changes applied
#[tauri::command]
async fn apply_zenoh_config_edit(
    config_json: ZenohConfigJson,
    edit: ZenohConfigEdit,
) -> Result<ZenohConfigJson, String> {
    // Convert to zenoh::Config
    let mut zenoh_config = config_json.into_zenoh_config()?;

    // Apply mode change
    let what_am_i = edit.to_what_am_i();
    zenoh_config
        .set_mode(Some(what_am_i))
        .map_err(|e| format!("Failed to set mode: {e:?}"))?;

    // Convert back to JSON and wrap
    let new_json = serde_json::to_value(&zenoh_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    ZenohConfigJson::from_json(new_json)
}

/// Create a new validated Zenoh config from edit fields with auto-assigned port
/// Returns validated config and assigned port
#[tauri::command]
async fn create_zenoh_config(
    edit: ZenohConfigEdit,
    runtimes_state: State<'_, ZenohRuntimes>,
) -> Result<ZenohConfigJson, String> {
    // Allocate a free port
    let port = runtimes_state.allocate_port().await;

    // Create the validated config from edit fields
    let config_json = ZenohConfigJson::create_from_edit(&edit, port)?;

    Ok(config_json)
}

/// Create a new Zenoh runtime with the given configuration.
/// Returns the ZenohId as a string on success.
#[tauri::command]
async fn zenoh_runtime_start(
    config: ZenohConfigJson,
    runtimes_state: State<'_, ZenohRuntimes>,
    logs_state: State<'_, LogStorage>,
) -> Result<String, String> {
    eprintln!(
        "🔵 zenoh_runtime_start called with config: port={:?}",
        config.get_websocket_port()
    );

    // Store the original config for later retrieval
    let sandbox_config = config.clone();

    // Convert ZenohConfigJson to zenoh::Config
    let zenoh_config = config.into_zenoh_config()?;

    // Create a unique socket path with short name to avoid SUN_LEN limit
    // Use a short random suffix instead of full UUID
    let random_id: u32 = rand::random();
    let socket_path = runtimes_state
        .socket_dir
        .join(format!("z{:x}.sock", random_id));

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

    // Use the log_dir from state (already created in ZenohRuntimes::new)
    let log_dir = &runtimes_state.log_dir;

    // Create log files for stdout and stderr
    let log_prefix = format!("z{:x}", random_id);
    let stdout_log = log_dir.join(format!("{}-stdout.log", log_prefix));
    let stderr_log = log_dir.join(format!("{}-stderr.log", log_prefix));

    let stdout_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stdout_log)
        .map_err(|e| {
            format!(
                "Failed to create stdout log file {}: {}",
                stdout_log.display(),
                e
            )
        })?;

    let stderr_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stderr_log)
        .map_err(|e| {
            format!(
                "Failed to create stderr log file {}: {}",
                stderr_log.display(),
                e
            )
        })?;

    // Spawn the runtime process
    let mut child = tokio::process::Command::new(&runtime_binary)
        .arg(socket_path.to_string_lossy().to_string())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to spawn runtime process: {} (path: {})",
                e,
                runtime_binary.display()
            )
        })?;

    eprintln!("Runtime process spawned with PID: {:?}", child.id());
    eprintln!(
        "Logs:\n{}\n{}\n",
        stdout_log.display(),
        stderr_log.display()
    );

    // Accept connection from the runtime process
    eprintln!("Waiting for runtime to connect...");
    let (socket, _) = tokio::time::timeout(std::time::Duration::from_secs(10), listener.accept())
        .await
        .map_err(|_| {
            drop(child.kill());
            "Timeout waiting for runtime to connect (10s). Check stderr output.".to_string()
        })?
        .map_err(|e| {
            drop(child.kill());
            format!("Failed to accept connection: {}", e)
        })?;

    eprintln!("Runtime connected successfully");

    // Send Start message with zenoh::Config
    eprintln!("📤 Sending start message to runtime...");
    let start_msg = MainToRuntime::Start(Box::new(zenoh_config.clone()));
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
    eprintln!("📤 Start message sent");

    // Receive Started response
    eprintln!("📥 Waiting for runtime response...");
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    eprintln!("📥 Got response: {}", line.trim());

    let response: RuntimeToMain =
        serde_json::from_str(&line).map_err(|e| format!("Failed to parse response: {}", e))?;

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
    eprintln!("✅ Parsed ZenohId: {}", zid);

    // Spawn log receiver task (reader continues to receive logs)
    // This task also handles config requests
    eprintln!("🔧 Setting up receiver task...");
    let logs_storage = logs_state.inner().clone();
    let zid_clone = zid;

    // Create channel for sending requests to the receiver task
    let (request_tx, mut request_rx) = mpsc::channel::<RuntimeRequest>(16);

    eprintln!("🚀 Spawning receiver task...");
    let receiver_task = tokio::spawn(async move {
        let mut line = String::new();
        // Track pending config request
        let mut pending_config_request: Option<oneshot::Sender<Config>> = None;

        loop {
            tokio::select! {
                // Handle incoming messages from runtime
                read_result = reader.read_line(&mut line) => {
                    match read_result {
                        Ok(0) => break, // Socket closed
                        Ok(_) => {
                            if let Ok(msg) = serde_json::from_str::<RuntimeToMain>(&line) {
                                match msg {
                                    RuntimeToMain::Log(entry) => {
                                        logs_storage.add_log(zid_clone, entry);
                                    }
                                    RuntimeToMain::Config(config) => {
                                        // Send response to pending request
                                        if let Some(tx) = pending_config_request.take() {
                                            let _ = tx.send(*config);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            line.clear();
                        }
                        Err(_) => break,
                    }
                }
                // Handle requests from main thread
                Some(request) = request_rx.recv() => {
                    match request {
                        RuntimeRequest::GetConfig(response_tx) => {
                            // Send GetConfig request to runtime
                            let msg = MainToRuntime::GetConfig;
                            if let Ok(json) = serde_json::to_string(&msg)
                                && writer.write_all(format!("{json}\n").as_bytes()).await.is_ok()
                            {
                                let _ = writer.flush().await;
                                pending_config_request = Some(response_tx);
                            }
                        }
                        RuntimeRequest::Stop(response_tx) => {
                            // Send Stop request to runtime
                            let msg = MainToRuntime::Stop;
                            if let Ok(json) = serde_json::to_string(&msg) {
                                let _ = writer.write_all(format!("{json}\n").as_bytes()).await;
                                let _ = writer.flush().await;
                            }
                            let _ = response_tx.send(());
                            break;
                        }
                    }
                }
            }
        }
    });

    // Store the runtime process
    eprintln!("🔷 About to acquire write lock for zid: {}", zid);
    {
        let mut runtimes = runtimes_state.runtimes.write().await;
        eprintln!("🔷 Write lock acquired for zid: {}", zid);
        runtimes.insert(
            zid,
            RuntimeProcess {
                sandbox_config,
                child,
                receiver_task,
                request_tx,
            },
        );
        eprintln!("🔷 Inserted runtime for zid: {}", zid);
    }
    eprintln!("🔷 Write lock released for zid: {}", zid);

    // Clean up socket file
    let _ = tokio::fs::remove_file(&socket_path).await;

    eprintln!("🟢 zenoh_runtime_start returning success: {}", zid);
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
    let zenoh_id =
        ZenohId::from_str(&zid).map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Remove from state and get the runtime process
    let mut runtime_process = {
        let mut runtimes = runtimes_state.runtimes.write().await;
        runtimes
            .remove(&zenoh_id)
            .ok_or_else(|| format!("Zenoh runtime '{}' not found", zid))?
    };

    // Release the port
    if let Some(port) = runtime_process.sandbox_config.get_websocket_port() {
        runtimes_state.release_port(port).await;
    }

    // Send Stop request through the channel
    let (response_tx, response_rx) = oneshot::channel();
    let _ = runtime_process
        .request_tx
        .send(RuntimeRequest::Stop(response_tx))
        .await;

    // Wait for the stop to be sent (with timeout)
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), response_rx).await;

    // Wait for the child process to exit
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        runtime_process.child.wait(),
    )
    .await;

    // Kill the child process if it's still running
    let _ = runtime_process.child.kill().await;

    // Abort the receiver task
    runtime_process.receiver_task.abort();

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

/// Get the initial configuration used to start a Zenoh runtime by its ZenohId string.
/// Returns the sandbox::ZenohConfigJson.
#[tauri::command]
async fn zenoh_runtime_config(
    zid: String,
    state: State<'_, ZenohRuntimes>,
) -> Result<ZenohConfigJson, String> {
    // Parse the ZenohId from string
    let zenoh_id =
        ZenohId::from_str(&zid).map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Get the config from state
    let runtimes = state.runtimes.read().await;
    let runtime_process = runtimes
        .get(&zenoh_id)
        .ok_or_else(|| format!("Zenoh runtime '{}' not found", zid))?;

    Ok(runtime_process.sandbox_config.clone())
}

/// Get the current Zenoh configuration from a running runtime.
/// This returns the actual zenoh::Config.
#[tauri::command]
async fn zenoh_runtime_config_json(
    zid: String,
    state: State<'_, ZenohRuntimes>,
) -> Result<Config, String> {
    // Parse the ZenohId from string
    let zenoh_id =
        ZenohId::from_str(&zid).map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Get the request channel
    let request_tx = {
        let runtimes = state.runtimes.read().await;
        let runtime_process = runtimes
            .get(&zenoh_id)
            .ok_or_else(|| format!("Zenoh runtime '{}' not found", zid))?;
        runtime_process.request_tx.clone()
    };

    // Send request and wait for response
    let (response_tx, response_rx) = oneshot::channel();
    request_tx
        .send(RuntimeRequest::GetConfig(response_tx))
        .await
        .map_err(|_| "Failed to send config request".to_string())?;

    // Wait for response with timeout
    let config = tokio::time::timeout(std::time::Duration::from_secs(5), response_rx)
        .await
        .map_err(|_| "Timeout waiting for config response".to_string())?
        .map_err(|_| "Config request was cancelled".to_string())?;

    Ok(config)
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
    let zenoh_id =
        ZenohId::from_str(&zid).map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

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
            verify_zenoh_config_json,
            apply_zenoh_config_edit,
            create_zenoh_config,
            zenoh_runtime_start,
            zenoh_runtime_stop,
            zenoh_runtime_list,
            zenoh_runtime_config,
            zenoh_runtime_config_json,
            zenoh_runtime_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
