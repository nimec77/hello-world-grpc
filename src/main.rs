use anyhow::Result;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use hello_world_grpc::services::hello_world::{greeter_server::GreeterServer, GreeterService};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    init_logging()?;

    info!("Starting Hello World gRPC Server");

    // Create the gRPC service instance
    let greeter_service = GreeterService;
    
    // Configure the server address
    let addr = "127.0.0.1:50051".parse()?;
    info!("gRPC server listening on {}", addr);

    // Build and start the gRPC server
    Server::builder()
        .add_service(GreeterServer::new(greeter_service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Initialize structured logging with environment-based configuration
fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| anyhow::anyhow!("Failed to initialize log filter: {}", e))?;

    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(filter)
        .init();

    Ok(())
}
