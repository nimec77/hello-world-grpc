use anyhow::{Context, Result};
use std::time::Duration;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use hello_world_grpc::config::{load_config, LoggingConfig};
use hello_world_grpc::services::hello_world::{greeter_server::GreeterServer, GreeterService};
use hello_world_grpc::utils::{start_health_server, SimpleMetrics};

#[tokio::main]
async fn main() -> Result<()> {
    // Load and validate configuration early
    let config = load_config().context("Failed to load configuration")?;

    config
        .validate()
        .context("Configuration validation failed")?;

    // Initialize logging with config
    init_logging(&config.logging)?;

    info!(
        grpc_address = %config.server.grpc_address,
        health_port = config.server.health_port,
        log_level = %config.logging.level,
        log_format = %config.logging.format,
        version = env!("CARGO_PKG_VERSION"),
        "Starting Hello World gRPC Server with configuration"
    );

    // Create metrics collection instance
    let metrics = SimpleMetrics::new();

    // Create the gRPC service instance with metrics
    let greeter_service = GreeterService::new(metrics.clone());

    // Setup gRPC health check service
    let (health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<GreeterServer<GreeterService>>()
        .await;

    // Parse server address from configuration
    let addr = config
        .server
        .grpc_address
        .parse()
        .context("Failed to parse gRPC address")?;
    let health_port = config.server.health_port;

    info!(address = %addr, "gRPC server will listen on");
    info!(
        port = health_port,
        "HTTP health check server will start on port"
    );

    // Start periodic metrics logging task
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            metrics_clone.log_summary();
        }
    });

    info!("Started periodic metrics logging (every 60 seconds)");

    // Start HTTP health check server
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_port).await {
            tracing::error!(error = %e, "Failed to start health server");
        }
    });

    info!("Started health check servers (gRPC + HTTP)");

    info!("All services configured, starting gRPC server");

    // Build and start the gRPC server with health service
    Server::builder()
        .add_service(health_service)
        .add_service(GreeterServer::new(greeter_service))
        .serve(addr)
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

/// Initialize structured logging with configuration-based setup
fn init_logging(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .context("Failed to initialize log filter")?;

    match config.format.as_str() {
        "json" => {
            // Production: JSON format for log aggregation
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_span_events(fmt::format::FmtSpan::CLOSE)
                        .with_ansi(false), // Disable colors for JSON output
                )
                .init();
        }
        "pretty" => {
            // Development: Human-readable format
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(false)
                        .with_thread_ids(false),
                )
                .init();
        }
        _ => {
            anyhow::bail!("Invalid logging format: {}", config.format);
        }
    }

    Ok(())
}
