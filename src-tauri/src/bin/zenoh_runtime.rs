use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use zenoh::config::Config;
use zenoh::internal::{plugins::PluginsManager, runtime::Runtime, runtime::RuntimeBuilder};

use zenoh_sandbox_lib::logs::LogEntry;
use zenoh_sandbox_lib::protocol::{MainToRuntime, RuntimeToMain};

// ============================================================================
// Log Capture Layer
// ============================================================================

struct RuntimeLogLayer {
    log_tx: mpsc::UnboundedSender<LogEntry>,
}

impl RuntimeLogLayer {
    fn new(log_tx: mpsc::UnboundedSender<LogEntry>) -> Self {
        Self { log_tx }
    }
}

impl<S> Layer<S> for RuntimeLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: event.metadata().level().into(),
            target: event.metadata().target().to_string(),
            message: extract_message(event),
        };

        // Send log through channel (ignore errors if receiver dropped)
        let _ = self.log_tx.send(entry);
    }
}

/// Extract message field from a tracing event
fn extract_message(event: &tracing::Event<'_>) -> String {
    let mut message = String::new();
    event.record(&mut MessageVisitor(&mut message));
    message
}

struct MessageVisitor<'a>(&'a mut String);

impl tracing::field::Visit for MessageVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.0 = format!("{:?}", value);
            // Remove surrounding quotes if present
            if self.0.starts_with('"') && self.0.ends_with('"') && self.0.len() >= 2 {
                *self.0 = self.0[1..self.0.len() - 1].to_string();
            }
        }
    }
}

// ============================================================================
// Socket Communication Helpers
// ============================================================================

/// Send a message to the main process via the socket
async fn send_message(writer: &mut OwnedWriteHalf, msg: &RuntimeToMain) -> Result<(), String> {
    let json = serde_json::to_string(msg).map_err(|e| format!("Serialization error: {e}"))?;
    writer
        .write_all(format!("{json}\n").as_bytes())
        .await
        .map_err(|e| format!("Write error: {e}"))?;
    writer.flush().await.map_err(|e| format!("Flush error: {e}"))?;
    Ok(())
}

/// Read a message from the main process via the socket
/// Returns None if socket is closed, Some(msg) on success
async fn read_message(
    reader: &mut BufReader<OwnedReadHalf>,
    line: &mut String,
) -> Result<Option<MainToRuntime>, String> {
    line.clear();
    match reader.read_line(line).await {
        Ok(0) => Ok(None), // Socket closed
        Ok(_) => {
            let msg = serde_json::from_str(line).map_err(|e| format!("Parse error: {e}"))?;
            Ok(Some(msg))
        }
        Err(e) => Err(format!("Read error: {e}")),
    }
}

// ============================================================================
// Logging Setup
// ============================================================================

/// Initialize the tracing subscriber with log capture
fn setup_logging(log_tx: mpsc::UnboundedSender<LogEntry>) {
    // Set RUST_LOG for maximum verbosity from Zenoh
    unsafe {
        std::env::set_var("RUST_LOG", "trace");
    }

    let log_layer = RuntimeLogLayer::new(log_tx)
        .with_filter(tracing_subscriber::filter::LevelFilter::TRACE);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .with(log_layer)
        .init();
}

// ============================================================================
// Runtime Management
// ============================================================================

/// Build and start a Zenoh runtime with the given configuration
async fn start_runtime(zenoh_config: Config) -> Result<(zenoh::session::ZenohId, Runtime), String> {
    eprintln!("🟦 start_runtime: Setting up plugins manager");
    let mut plugins_mgr = PluginsManager::static_plugins_only();
    plugins_mgr.declare_static_plugin::<zenoh_plugin_remote_api::RemoteApiPlugin, &str>(
        "remote_api",
        true,
    );

    eprintln!("🟦 start_runtime: Building Zenoh runtime");
    tracing::info!("Building Zenoh runtime");

    let mut runtime = RuntimeBuilder::new(zenoh_config)
        .plugins_manager(plugins_mgr)
        .build()
        .await
        .map_err(|e| format!("Failed to build runtime: {e}"))?;

    let zid = runtime.zid();
    eprintln!("🟦 start_runtime: Runtime built with ZID: {}", zid);
    tracing::info!("Runtime built with ZID: {zid}");

    eprintln!("🟦 start_runtime: Starting runtime");
    runtime
        .start()
        .await
        .map_err(|e| format!("Failed to start runtime: {e}"))?;

    eprintln!("🟦 start_runtime: Runtime started successfully");
    tracing::info!("Runtime started successfully");
    Ok((zid, runtime))
}

/// Get the current zenoh configuration
fn get_config(runtime: &Runtime) -> Config {
    let config = runtime.config().lock();
    config.clone()
}

// ============================================================================
// Event Loop
// ============================================================================

/// Main event loop: forwards logs and handles commands
async fn run_event_loop(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut OwnedWriteHalf,
    log_rx: &mut mpsc::UnboundedReceiver<LogEntry>,
    runtime: &Runtime,
) -> Result<(), String> {
    let mut line = String::new();

    loop {
        tokio::select! {
            // Handle incoming commands
            result = read_message(reader, &mut line) => {
                match result? {
                    None => break, // Socket closed
                    Some(MainToRuntime::Stop) => {
                        send_message(writer, &RuntimeToMain::Stopped).await?;
                        break;
                    }
                    Some(MainToRuntime::Start(_)) => {
                        // Ignore duplicate start commands
                    }
                    Some(MainToRuntime::GetConfig) => {
                        let config = get_config(runtime);
                        send_message(writer, &RuntimeToMain::Config(Box::new(config))).await?;
                    }
                }
            }
            // Forward log entries to main process
            Some(entry) = log_rx.recv() => {
                // Ignore send errors (main process may have closed)
                let _ = send_message(writer, &RuntimeToMain::Log(entry)).await;
            }
        }
    }

    Ok(())
}

// ============================================================================
// Panic Handler
// ============================================================================

/// Flush remaining logs and send final error message
async fn send_final_error(
    writer: &mut OwnedWriteHalf,
    log_rx: &mut mpsc::UnboundedReceiver<LogEntry>,
    error_msg: String,
) {
    // Drain any remaining logs in the channel
    while let Ok(entry) = log_rx.try_recv() {
        let _ = send_message(writer, &RuntimeToMain::Log(entry)).await;
    }

    // Send final error log
    let final_log = LogEntry {
        timestamp: Utc::now(),
        level: zenoh_sandbox_lib::ts::log::LogEntryLevel::ERROR,
        target: "zenoh_runtime".to_string(),
        message: error_msg,
    };
    let _ = send_message(writer, &RuntimeToMain::Log(final_log)).await;

    // Small delay to ensure message is sent
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

/// Main execution with error handling
async fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <socket_path>", args[0]);
        std::process::exit(1);
    }
    let socket_path = &args[1];

    // Connect to UDS socket and split into reader/writer
    let socket = UnixStream::connect(socket_path).await?;
    let (reader, writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    // Set up log capture channel
    let (log_tx, mut log_rx) = mpsc::unbounded_channel::<LogEntry>();
    setup_logging(log_tx);

    // Wait for Start command
    let mut line = String::new();
    let Some(MainToRuntime::Start(config)) = read_message(&mut reader, &mut line).await? else {
        return Ok(()); // Socket closed or unexpected message
    };

    // Start the runtime
    match start_runtime(*config).await {
        Ok((zid, runtime)) => {
            // Runtime started successfully
            send_message(&mut writer, &RuntimeToMain::Started(zid.to_string())).await?;

            // Run event loop
            match run_event_loop(&mut reader, &mut writer, &mut log_rx, &runtime).await {
                Ok(()) => {
                    // Clean shutdown - flush remaining logs
                    send_final_error(&mut writer, &mut log_rx, "Runtime stopped".to_string()).await;
                }
                Err(e) => {
                    // Event loop error - flush logs and send error
                    send_final_error(&mut writer, &mut log_rx, format!("Runtime error: {}", e)).await;
                }
            }
        }
        Err(e) => {
            // Runtime start error - flush logs and send error
            send_final_error(&mut writer, &mut log_rx, format!("Failed to start: {}", e)).await;
            send_message(&mut writer, &RuntimeToMain::StartError(e)).await?;
        }
    }

    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_main().await
}
