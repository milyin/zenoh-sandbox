use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use zenoh::internal::{plugins::PluginsManager, runtime::RuntimeBuilder};
use zenoh_config::WhatAmI;

use zenoh_sandbox_lib::logs::LogEntry;
use zenoh_sandbox_lib::protocol::{MainToRuntime, RuntimeToMain};
use zenoh_sandbox_lib::sandbox::ZenohConfig;

// ============================================================================
// Log Capture for Runtime Process
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
        let level = event.metadata().level().to_string();
        let target = event.metadata().target().to_string();

        let mut message = String::new();
        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            target,
            message,
        };

        // Send log through channel (non-blocking)
        let _ = self.log_tx.send(entry);
    }
}

struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.0 = format!("{:?}", value);
            if self.0.starts_with('"') && self.0.ends_with('"') {
                *self.0 = self.0[1..self.0.len() - 1].to_string();
            }
        }
    }
}

// ============================================================================
// Main runtime process
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <socket_path>", args[0]);
        std::process::exit(1);
    }

    let socket_path = &args[1];

    // Connect to the UDS socket
    let socket = UnixStream::connect(socket_path).await?;
    let (reader, writer) = socket.into_split();
    let mut writer = writer;

    // Create channel for log communication
    let (log_tx, mut log_rx) = mpsc::unbounded_channel::<LogEntry>();

    // Set RUST_LOG for maximum verbosity from Zenoh
    unsafe {
        std::env::set_var("RUST_LOG", "trace");
    }

    // Set up logging to capture logs and send them through the channel
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
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Wait for Start message
    match reader.read_line(&mut line).await {
        Ok(0) => {
            // Socket closed before receiving start message
            return Ok(());
        }
        Ok(_) => {
            let msg: MainToRuntime = serde_json::from_str(&line)?;
            match msg {
                MainToRuntime::Start(config) => {
                    // Start the runtime
                    match start_runtime(config).await {
                        Ok(zid) => {
                            let response = RuntimeToMain::Started(zid.to_string());
                            let json = serde_json::to_string(&response)?;
                            writer.write_all(format!("{}\n", json).as_bytes()).await?;
                            writer.flush().await?;

                            // Keep running and processing messages until socket closes
                            line.clear();
                            loop {
                                tokio::select! {
                                    // Handle incoming commands
                                    result = reader.read_line(&mut line) => {
                                        match result {
                                            Ok(0) => {
                                                // Socket closed, exit
                                                break;
                                            }
                                            Ok(_) => {
                                                if let Ok(msg) = serde_json::from_str::<MainToRuntime>(&line) {
                                                    match msg {
                                                        MainToRuntime::Stop => {
                                                            let response = RuntimeToMain::Stopped;
                                                            let json = serde_json::to_string(&response)?;
                                                            writer.write_all(format!("{}\n", json).as_bytes()).await?;
                                                            writer.flush().await?;
                                                            break;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                line.clear();
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                    // Handle log messages
                                    Some(entry) = log_rx.recv() => {
                                        let msg = RuntimeToMain::Log(entry);
                                        if let Ok(json) = serde_json::to_string(&msg) {
                                            if writer.write_all(format!("{}\n", json).as_bytes()).await.is_ok() {
                                                let _ = writer.flush().await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let response = RuntimeToMain::StartError(e);
                            let json = serde_json::to_string(&response)?;
                            writer.write_all(format!("{}\n", json).as_bytes()).await?;
                            writer.flush().await?;
                        }
                    }
                }
                MainToRuntime::Stop => {
                    // Nothing to stop yet
                }
            }
        }
        Err(_) => {
            return Ok(());
        }
    }

    Ok(())
}

async fn start_runtime(
    config: ZenohConfig,
) -> Result<zenoh::session::ZenohId, String> {
    // Convert config
    let mut zenoh_config = config
        .into_zenoh_config()
        .map_err(|e| format!("Failed to convert config: {}", e))?;

    // Enable verbose logging for zenoh
    zenoh_config
        .set_mode(Some(WhatAmI::Peer))
        .map_err(|e| format!("Failed to set mode: {:?}", e))?;

    // Create plugins manager
    let mut plugins_mgr = PluginsManager::static_plugins_only();
    plugins_mgr.declare_static_plugin::<zenoh_plugin_remote_api::RemoteApiPlugin, &str>(
        "remote_api",
        true,
    );

    tracing::info!("Building Zenoh runtime with config");

    // Build runtime
    let mut runtime = RuntimeBuilder::new(zenoh_config)
        .plugins_manager(plugins_mgr)
        .build()
        .await
        .map_err(|e| format!("Failed to build runtime: {}", e))?;

    let zid = runtime.zid();
    tracing::info!("Runtime built with ZID: {}", zid);

    // Start runtime
    runtime
        .start()
        .await
        .map_err(|e| format!("Failed to start runtime: {}", e))?;

    tracing::info!("Runtime started successfully");

    Ok(zid)
}
