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

use crate::ts::{config::{ZenohConfigEdit, ZenohConfigJson}, log::LogEntryLevel};

// ============================================================================
// State management for Zenoh runtimes
// ============================================================================

/// Runtime ID type - used as primary identifier for runtimes
pub type RuntimeId = u32;

/// Request type for communication with the runtime background task
enum RuntimeRequest {
    /// Request to get the config, with a oneshot channel for the response
    GetConfig(oneshot::Sender<Config>),
    /// Request to stop the runtime
    Stop(oneshot::Sender<()>),
}

/// Information about a running runtime process
struct RuntimeProcess {
    /// The Zenoh ID (available after runtime starts)
    zenoh_id: Option<ZenohId>,
    /// The original sandbox configuration
    sandbox_config: ZenohConfigJson,
    /// The child process handle
    child: Option<Child>,
    /// Task handle for log receiving and request handling
    receiver_task: Option<JoinHandle<()>>,
    /// Channel to send requests to the receiver task
    request_tx: Option<mpsc::Sender<RuntimeRequest>>,
    /// The allocated port for remote_api
    allocated_port: u16,
}

/// Holds all active Zenoh runtime processes
pub struct ZenohRuntimes {
    runtimes: RwLock<HashMap<RuntimeId, RuntimeProcess>>,
    /// Next runtime ID to allocate
    next_runtime_id: RwLock<RuntimeId>,
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
            next_runtime_id: RwLock::new(0),
            port_tracker: RwLock::new(HashSet::new()),
            socket_dir,
            log_dir,
        }
    }

    /// Allocate a new runtime ID
    pub async fn allocate_runtime_id(&self) -> RuntimeId {
        let mut next_id = self.next_runtime_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
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

/// Create a new validated Zenoh config from edit content
#[tauri::command]
async fn create_zenoh_config(
    edit: ZenohConfigEdit,
) -> Result<(ZenohConfigEdit, ZenohConfigJson), String> {
    let config = edit.to_config()?;

    let config_json = serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    let validated = ZenohConfigJson::from_json(config_json)?;

    Ok((edit, validated))
}

/// Get the default configuration as JSON string
#[tauri::command]
async fn get_default_config_json() -> Result<String, String> {
    let config = zenoh::config::Config::default();

    serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize default config: {}", e))
}

/// Validate JSON string as zenoh config and return validated JSON
#[tauri::command]
async fn validate_config(content: String) -> Result<ZenohConfigJson, String> {
    let config = zenoh::Config::from_json5(&content).map_err(|e| {
        format!("Invalid JSON5 config: {}", e)
    })?;
    let config_json = serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    ZenohConfigJson::from_json(config_json)
}

/// Compute the difference between two JSON configurations.
/// Returns a JSON object containing only fields that differ from base.
/// Deleted fields are represented as null.
#[tauri::command]
async fn compute_config_diff(
    base: ZenohConfigJson,
    modified: ZenohConfigJson,
) -> Result<serde_json::Value, String> {
    let diff = ts::config::json_diff(base.as_json(), modified.as_json());
    Ok(diff)
}

/// Declare a new runtime with the given config, allocating resources but not starting it yet.
/// Returns the RuntimeId that can be used to start the runtime.
#[tauri::command]
async fn declare_runtime(
    config: ZenohConfigJson,
    runtimes_state: State<'_, ZenohRuntimes>,
) -> Result<RuntimeId, String> {
    // Allocate runtime ID
    let runtime_id = runtimes_state.allocate_runtime_id().await;

    // Allocate port
    let port = runtimes_state.allocate_port().await;

    // Create runtime entry with uninitialized fields
    let runtime_process = RuntimeProcess {
        zenoh_id: None,
        sandbox_config: config,
        child: None,
        receiver_task: None,
        request_tx: None,
        allocated_port: port,
    };

    // Store in state
    let mut runtimes = runtimes_state.runtimes.write().await;
    runtimes.insert(runtime_id, runtime_process);

    Ok(runtime_id)
}

/// Start a previously declared runtime.
/// Returns the ZenohId string.
#[tauri::command]
async fn start_runtime(
    runtime_id: RuntimeId,
    runtimes_state: State<'_, ZenohRuntimes>,
    logs_state: State<'_, LogStorage>,
) -> Result<String, String> {
    // Get the runtime process and config
    let (config, port) = {
        let runtimes = runtimes_state.runtimes.read().await;
        let runtime_process = runtimes
            .get(&runtime_id)
            .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;
        (runtime_process.sandbox_config.clone(), runtime_process.allocated_port)
    };

    eprintln!(
        "🔵 start_runtime called for runtime_id={} with config: port={:?}",
        runtime_id,
        config.get_websocket_port()
    );

    // Convert ZenohConfigJson to zenoh::Config
    let mut zenoh_config: zenoh::config::Config = config.try_into()?;

    // Apply runtime-specific config modifications (not visible to GUI)
    // Enable adminspace
    zenoh_config
        .adminspace
        .set_enabled(true)
        .map_err(|e| format!("Failed to enable adminspace: {e}"))?;

    // Enable plugins loading
    zenoh_config
        .plugins_loading
        .set_enabled(true)
        .map_err(|e| format!("Failed to enable plugins loading: {e}"))?;

    // Add remote_api plugin configuration
    zenoh_config
        .insert_json5("plugins/remote_api", "{}")
        .map_err(|e| format!("Failed to add remote_api plugin config: {e}"))?;

    // Set websocket_port for remote_api
    zenoh_config
        .insert_json5(
            "plugins/remote_api/websocket_port",
            &format!(r#""{}""#, port),
        )
        .map_err(|e| format!("Failed to set websocket_port: {e}"))?;

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
    let runtime_id_clone = runtime_id;

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
                                        logs_storage.add_log(runtime_id_clone, entry);
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

    // Update the runtime process with the started runtime details
    eprintln!("🔷 About to acquire write lock for runtime_id: {}", runtime_id);
    {
        let mut runtimes = runtimes_state.runtimes.write().await;
        eprintln!("🔷 Write lock acquired for runtime_id: {}", runtime_id);
        if let Some(runtime_process) = runtimes.get_mut(&runtime_id) {
            runtime_process.zenoh_id = Some(zid);
            runtime_process.child = Some(child);
            runtime_process.receiver_task = Some(receiver_task);
            runtime_process.request_tx = Some(request_tx);
        } else {
            return Err(format!("Runtime {} disappeared during startup", runtime_id));
        }
        eprintln!("🔷 Updated runtime for runtime_id: {}", runtime_id);
    }
    eprintln!("🔷 Write lock released for runtime_id: {}", runtime_id);

    // Clean up socket file
    let _ = tokio::fs::remove_file(&socket_path).await;

    eprintln!("🟢 start_runtime returning success: {} on port {}", zid, port);
    Ok(zid.to_string())
}

/// stop (close) a Zenoh runtime by its RuntimeId.
#[tauri::command]
async fn zenoh_runtime_stop(
    runtime_id: RuntimeId,
    runtimes_state: State<'_, ZenohRuntimes>,
    _logs_state: State<'_, LogStorage>,
) -> Result<(), String> {
    // Get and update the runtime process
    let (child_opt, receiver_task_opt, request_tx_opt, port) = {
        let mut runtimes = runtimes_state.runtimes.write().await;
        let runtime_process = runtimes
            .get_mut(&runtime_id)
            .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

        // Extract the running components and clear them
        let child = runtime_process.child.take();
        let receiver_task = runtime_process.receiver_task.take();
        let request_tx = runtime_process.request_tx.take();
        let port = runtime_process.allocated_port;

        (child, receiver_task, request_tx, port)
    };

    // Send Stop request through the channel if available
    if let Some(request_tx) = request_tx_opt {
        let (response_tx, response_rx) = oneshot::channel();
        let _ = request_tx.send(RuntimeRequest::Stop(response_tx)).await;
        // Wait for the stop to be sent (with timeout)
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), response_rx).await;
    }

    // Wait for the child process to exit
    if let Some(mut child) = child_opt {
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            child.wait(),
        )
        .await;
        // Kill the child process if it's still running
        let _ = child.kill().await;
    }

    // Abort the receiver task
    if let Some(receiver_task) = receiver_task_opt {
        receiver_task.abort();
    }

    // Release the allocated port
    runtimes_state.release_port(port).await;

    // Don't clear logs - keep them available for stopped runtime
    // Don't remove from state - keep runtime entry for UI

    Ok(())
}

/// List all runtime IDs.
#[tauri::command]
async fn zenoh_runtime_list(state: State<'_, ZenohRuntimes>) -> Result<Vec<RuntimeId>, String> {
    let runtimes = state.runtimes.read().await;
    let runtime_ids: Vec<RuntimeId> = runtimes.keys().copied().collect();
    Ok(runtime_ids)
}

/// Get the initial configuration used to start a runtime by its RuntimeId.
/// Returns the sandbox::ZenohConfigJson.
#[tauri::command]
async fn zenoh_runtime_config(
    runtime_id: RuntimeId,
    state: State<'_, ZenohRuntimes>,
) -> Result<ZenohConfigJson, String> {
    // Get the config from state
    let runtimes = state.runtimes.read().await;
    let runtime_process = runtimes
        .get(&runtime_id)
        .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;

    Ok(runtime_process.sandbox_config.clone())
}

/// Get the current Zenoh configuration from a running runtime.
/// This returns the actual zenoh::Config.
#[tauri::command]
async fn zenoh_runtime_config_json(
    runtime_id: RuntimeId,
    state: State<'_, ZenohRuntimes>,
) -> Result<Config, String> {
    // Get the request channel
    let request_tx = {
        let runtimes = state.runtimes.read().await;
        let runtime_process = runtimes
            .get(&runtime_id)
            .ok_or_else(|| format!("Runtime {} not found", runtime_id))?;
        runtime_process.request_tx.clone()
            .ok_or_else(|| "Runtime not started yet".to_string())?
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

/// Get a page of logs from a specific runtime.
/// Page 0 returns the most recent logs.
#[tauri::command]
async fn zenoh_runtime_log(
    runtime_id: RuntimeId,
    level: Option<LogEntryLevel>,
    page: usize,
    state: State<'_, LogStorage>,
) -> Result<Vec<LogEntry>, String> {
    Ok(state.get_page(runtime_id, level, page))
}

/// Cleanup logs and remove a stopped runtime.
/// This should be called when removing a stopped runtime from the UI.
#[tauri::command]
async fn zenoh_runtime_cleanup(
    runtime_id: RuntimeId,
    runtimes_state: State<'_, ZenohRuntimes>,
    logs_state: State<'_, LogStorage>,
) -> Result<(), String> {
    // Remove from runtime state
    {
        let mut runtimes = runtimes_state.runtimes.write().await;
        runtimes.remove(&runtime_id);
    }

    // Clear logs for this runtime
    logs_state.clear_logs(runtime_id);

    Ok(())
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
            validate_config,
            get_default_config_json,
            compute_config_diff,
            create_zenoh_config,
            declare_runtime,
            start_runtime,
            zenoh_runtime_stop,
            zenoh_runtime_list,
            zenoh_runtime_config,
            zenoh_runtime_config_json,
            zenoh_runtime_log,
            zenoh_runtime_cleanup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
