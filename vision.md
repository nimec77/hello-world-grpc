# Technical Vision - gRPC Learning Project

## 1. Technologies

**Core Stack (Latest Stable Versions):**
- **Rust** (1.70+ MSRV)
- **tonic** (~0.12) - gRPC framework with async support
- **tokio** (1.47+ LTS) - async runtime with full features 
- **prost** - Protocol Buffers codegen (included with tonic)

**Essential Libraries for Ease of Use:**
- **anyhow** (1.0) - flexible error handling with context
- **tracing** (0.1) - structured logging and diagnostics  
- **tracing-subscriber** (0.3) - log formatting and output
- **serde** (1.0) - serialization for configuration
- **config** (0.14) - layered configuration management
- **tokio-stream** (0.1) - stream utilities for gRPC streaming

**Development Tools:**
- **protoc** - Protocol Buffers compiler
- Standard Rust toolchain (cargo, rustfmt, clippy)

**Cargo.toml Dependencies:**
```toml
[dependencies]
tokio = { version = "1.47", features = ["full"] }
tonic = "0.12"
prost = "0.13"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1.0", features = ["derive"] }
config = "0.14"
tokio-stream = "0.1"

[build-dependencies]
tonic-build = "0.12"
```

**What we're deliberately excluding (KISS principle):**
- Authentication/authorization systems
- Advanced metrics (Prometheus, etc.)
- Database integration
- Complex middleware stacks
- Load balancing libraries
- Service discovery mechanisms

---

## 2. Development Principles

**Core Development Approach:**

1. **KISS (Keep It Simple, Stupid)**
   - Start with the simplest solution that works
   - Add complexity only when absolutely necessary
   - Prefer readable code over clever code
   - Measure before optimizing

2. **Test-Driven Development (TDD)**
   - Write unit tests first for all public functions
   - Add integration tests as functionality grows
   - Use `#[tokio::test]` for async test functions
   - Aim for high test coverage on critical paths

3. **Documentation-First**
   - Write `///` doc comments for all public APIs
   - Include usage examples in documentation
   - Document error conditions and panics
   - Keep README and inline docs synchronized

**Rust-Specific Principles:**

4. **Embrace Ownership & Borrowing**
   - Use ownership to prevent data races naturally
   - Prefer borrowing (`&T`) over cloning when possible
   - Use `Arc<T>` for shared immutable data across threads
   - Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for shared mutable state

5. **Idiomatic Error Handling**
   - Use `Result<T, E>` for fallible operations
   - Use `anyhow::Result<T>` for application errors
   - Use `?` operator for error propagation
   - Add context with `.context()` for debugging
   - Fail fast with meaningful error messages

6. **Zero-Cost Abstractions**
   - Leverage Rust's type system for compile-time guarantees
   - Use enums for state machines and variants
   - Prefer `Option<T>` over null pointers
   - Use iterator chains over manual loops where clear

**Async Programming Principles:**

7. **Async-First Design**
   - Use `async fn` for all I/O operations
   - Spawn tasks with `tokio::spawn` for concurrency
   - Use `select!` for racing multiple async operations
   - Avoid blocking calls in async context

8. **Resource Management**
   - Use RAII pattern for cleanup (Drop trait)
   - Implement graceful shutdown with signal handling
   - Use bounded channels to prevent memory bloat
   - Handle backpressure in streaming operations

9. **Concurrent Safety**
   - Use `tokio::sync` primitives instead of `std::sync`
   - Prefer message passing over shared mutable state
   - Use `Arc` for shared ownership, `Rc` for single-threaded
   - Avoid `.unwrap()` in production code

**Code Organization:**

10. **Modular Architecture**
    - Separate concerns into focused modules
    - Keep functions small (< 20 lines when possible)
    - Use clear, descriptive names (`is_ready`, `has_data`)
    - Follow Rust naming conventions (snake_case, PascalCase)

11. **Performance Mindset**
    - Profile before optimizing
    - Use `cargo bench` for performance regression detection
    - Avoid unnecessary allocations and clones
    - Use `Cow<'_, T>` for conditional ownership

12. **Code Quality**
    - Run `cargo clippy` for linting
    - Use `cargo fmt` for consistent formatting
    - Keep dependencies minimal and justified
    - Write self-documenting code with clear variable names

13. **Domain Driven Design (DDD)**
    - Implement domain validation in business logic layer
    - Separate domain models from transport models (protobuf)
    - Use rich domain types instead of primitive strings/numbers
    - Validate input at domain boundaries, not just transport layer

**gRPC-Specific Principles:**

14. **Service Design**
    - Keep protobuf schemas simple and forward-compatible
    - Use streaming for large data or real-time updates
    - Handle network errors and timeouts gracefully
    - Implement proper health checks

15. **Production Readiness**
    - Log structured data with `tracing`
    - Handle client disconnections gracefully
    - Implement request tracing for debugging
    - Use environment variables for configuration

---

## 3. Project Structure

Following KISS principles with modern Rust 2018+ module system (no `mod.rs` files):

```
hello-world-grpc/
├── Cargo.toml              # Dependencies and project configuration
├── build.rs                # Protobuf build script (tonic-build)
├── README.md               # Project documentation
├── .env.example            # Environment variable template
├── .gitignore              # Git ignore patterns
│
├── proto/                  # Protocol Buffer definitions
│   └── hello_world.proto   # Basic greeting service schema
│
├── src/
│   ├── main.rs             # Server entry point and runtime setup
│   ├── lib.rs              # Library exports and common types
│   ├── config.rs           # Configuration management with `config` crate
│   ├── error.rs            # Custom error types with `anyhow` integration
│   ├── utils.rs            # Common utility functions
│   │
│   └── services/           # gRPC service implementations
│       └── hello_world.rs  # Greeting service implementation
│
└── tests/
    ├── common.rs           # Shared test utilities and helpers
    └── integration/        # Integration tests directory
        └── hello_world_test.rs  # Service integration tests
```

**Module Organization (Modern Rust 2018+):**

In `src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod utils;
pub mod services {
    pub mod hello_world;
}
```

In `src/main.rs`:
```rust
use hello_world_grpc::{config, services};
// Clean imports without mod.rs files
```

**Key Design Decisions:**

1. **No `mod.rs` Files**: Using Rust 2018+ module system for cleaner organization
2. **Single Binary**: Simple `main.rs` approach as requested
3. **Separated Concerns**: Each module has a clear, single responsibility
4. **Test Structure**: Integration tests separate from unit tests
5. **Minimal Start**: Only essential files, easy to extend later

**Growth Path:**
- Add new services as `src/services/calculator.rs`, `src/services/chat.rs`
- Extend `utils.rs` or split into `src/utils/` directory when needed
- Add `src/middleware/` for interceptors when complexity grows

This structure balances simplicity with clear organization and follows modern Rust practices.

---

## 4. Data Model

Following KISS principles with DDD validation, starting with the **simplest possible** schema:

**`proto/hello_world.proto`:**
```protobuf
syntax = "proto3";

package hello_world;

// The greeting service definition
service Greeter {
  // Sends a greeting (unary RPC - simplest possible)
  rpc SayHello (HelloRequest) returns (HelloReply);
}

// The request message containing the user's name
message HelloRequest {
  string name = 1;
}

// The response message containing the greetings
message HelloReply {
  string message = 1;
}
```

**Domain Model with DDD Validation** (Rust side):

```rust
// Domain types (separate from protobuf)
#[derive(Debug, Clone)]
pub struct PersonName(String);

impl PersonName {
    pub fn new(name: String) -> anyhow::Result<Self> {
        if name.trim().is_empty() {
            anyhow::bail!("Name cannot be empty");
        }
        if name.len() > 100 {
            anyhow::bail!("Name too long (max 100 characters)");
        }
        if name.chars().any(|c| c.is_control()) {
            anyhow::bail!("Name contains invalid characters");
        }
        Ok(PersonName(name.trim().to_string()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct GreetingMessage(String);

impl GreetingMessage {
    pub fn new(message: String) -> Self {
        GreetingMessage(message)
    }
    
    pub fn into_string(self) -> String {
        self.0
    }
}
```

**Key Design Decisions:**

1. **Simple Schema**: Only name → greeting message
2. **Domain Validation**: Rich types with business rules validation
3. **Separation**: Domain models separate from protobuf transport models
4. **Fail Fast**: Validation at domain boundaries using `anyhow::Result`
5. **No Complex Types**: Start with strings, no nested structures

**Validation Rules (DDD):**
- **Name**: Non-empty, max 100 chars, no control characters
- **Message**: No validation (generated by our service)

**Future Growth Path** (when needed):
```protobuf
// Future service methods (NOT implementing now):
service Greeter {
  rpc SayHello (HelloRequest) returns (HelloReply);
  // rpc SayHelloStream (HelloRequest) returns (stream HelloReply);
  // rpc SayHelloBatch (HelloBatchRequest) returns (HelloBatchReply);
}

// Future message extensions:
// message HelloRequest {
//   string name = 1;
//   string language = 2;    // Optional language preference  
//   int32 count = 3;        // Number of greetings
// }
```

This provides a solid foundation with proper domain validation while keeping the transport layer minimal.

---

## 5. Working with gRPC

Our **minimal but production-ready** approach with logging and graceful shutdown:

**Basic Server Implementation Pattern:**

```rust
// src/services/hello_world.rs
use tonic::{Request, Response, Status};
use tracing::{info, warn, error};
use crate::{error::AppError, utils::extract_client_info};

pub struct GreeterService;

#[tonic::async_trait]
impl hello_world_proto::greeter_server::Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let client_info = extract_client_info(&request);
        
        // Request logging
        info!(
            method = "SayHello",
            client_addr = %client_info.addr,
            request_id = %client_info.request_id,
            "Processing greeting request"
        );
        
        // 1. Extract and validate domain data
        let req = request.into_inner();
        let person_name = PersonName::new(req.name)
            .map_err(|e| {
                warn!(
                    error = %e,
                    client_addr = %client_info.addr,
                    "Invalid name provided"
                );
                Status::invalid_argument(e.to_string())
            })?;
        
        // 2. Business logic (domain layer)
        let greeting = format!("Hello, {}!", person_name.as_str());
        let message = GreetingMessage::new(greeting);
        
        // Response logging
        info!(
            method = "SayHello",
            client_addr = %client_info.addr,
            request_id = %client_info.request_id,
            name = person_name.as_str(),
            "Successfully processed greeting"
        );
        
        // 3. Return response
        Ok(Response::new(HelloReply {
            message: message.into_string(),
        }))
    }
}
```

**Server Startup with Graceful Shutdown (main.rs):**

```rust
use tokio::signal;
use tonic::transport::Server;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize tracing early
    tracing_subscriber::fmt::init();
    info!("Starting hello-world-grpc server");
    
    // 2. Load configuration
    let config = config::load_config()?;
    let addr = config.server.address.parse()?;
    
    // 3. Create service
    let greeter = GreeterService::default();
    
    info!(address = %addr, "Configuring gRPC server");
    
    // 4. Start server with graceful shutdown
    let server = Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_shutdown(addr, shutdown_signal());
    
    info!(address = %addr, "gRPC server started successfully");
    
    // 5. Handle shutdown
    if let Err(e) = server.await {
        error!(error = %e, "Server error occurred");
        return Err(e.into());
    }
    
    info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C, shutting down gracefully");
        }
        _ = terminate => {
            warn!("Received SIGTERM, shutting down gracefully");
        }
    }
}
```

**Essential gRPC Features (Only What We Need):**

1. **Unary RPC**: Single request → single response (SayHello)
2. **Request Logging**: Structured logging with tracing
3. **Error Handling**: Domain validation → gRPC Status codes
4. **Graceful Shutdown**: Handle SIGTERM/Ctrl+C properly
5. **Basic Metadata**: Extract client info for logging

**Request Logging Pattern:**
- **Incoming**: Log method, client address, request ID
- **Validation Errors**: Log validation failures with context
- **Success**: Log successful operations with key data
- **Structured**: Use tracing fields for easy filtering/analysis

**Error Mapping:**
- Domain validation errors → `Status::invalid_argument`
- Internal errors → `Status::internal`
- Not found → `Status::not_found`
- Clear error messages for client debugging

**What We're NOT Implementing Now:**
- Streaming RPCs (server/client/bidirectional)
- Authentication/authorization
- Interceptors/middleware
- Health checks (will add in monitoring section)
- Connection pooling
- Load balancing

This approach gives us production-ready basics while maintaining simplicity and focusing only on our immediate needs.

---

## 6. gRPC Monitoring

Following KISS principles, our **essential monitoring** without overengineering:

**Basic Health Check (gRPC Standard):**

```rust
// Add to Cargo.toml:
// tonic-health = "0.12"
// hyper = { version = "1.0", features = ["full"] }

// src/services/health.rs
use tonic_health::server::{health_reporter, HealthReporter};
use tonic_health::ServingStatus;

pub async fn setup_health_check() -> HealthReporter {
    let (mut health_reporter, health_service) = health_reporter();
    
    // Mark our service as serving
    health_reporter
        .set_serving::<GreeterServer<GreeterService>>()
        .await;
    
    health_reporter
}
```

**Simple HTTP Health Endpoint:**

```rust
// src/utils.rs - Simple HTTP health check
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;

async fn health_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let health_status = serde_json::json!({
        "status": "healthy",
        "service": "hello-world-grpc",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    });
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(health_status.to_string()))
        .unwrap())
}

pub async fn start_health_server(port: u16) {
    let addr = ([0, 0, 0, 0], port).into();
    
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(health_handler))
    });
    
    let server = Server::bind(&addr).serve(make_svc);
    
    tracing::info!(port = port, "HTTP health check server started");
    
    if let Err(e) = server.await {
        tracing::error!(error = %e, "Health server error");
    }
}
```

**Request Duration Tracking:**

```rust
// src/services/hello_world.rs (updated with duration tracking)
use std::time::Instant;

#[tonic::async_trait]
impl hello_world_proto::greeter_server::Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let start_time = Instant::now();
        let client_info = extract_client_info(&request);
        
        // Request logging
        info!(
            method = "SayHello",
            client_addr = %client_info.addr,
            request_id = %client_info.request_id,
            "Processing greeting request"
        );
        
        // ... existing validation and business logic ...
        
        let duration = start_time.elapsed();
        
        // Response logging with duration
        info!(
            method = "SayHello",
            client_addr = %client_info.addr,
            request_id = %client_info.request_id,
            name = person_name.as_str(),
            duration_ms = duration.as_millis(),
            "Successfully processed greeting"
        );
        
        // Update metrics
        self.metrics.increment_request();
        self.metrics.increment_success();
        
        Ok(Response::new(HelloReply {
            message: message.into_string(),
        }))
    }
}
```

**Simple Metrics Collection:**

```rust
// src/utils.rs - Enhanced metrics with duration tracking
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct SimpleMetrics {
    pub requests_total: AtomicU64,
    pub requests_success: AtomicU64,
    pub requests_error: AtomicU64,
    pub total_duration_ms: AtomicU64,
}

impl SimpleMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_error: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
        })
    }
    
    pub fn record_request_duration(&self, duration_ms: u64) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }
    
    pub fn increment_success(&self) {
        self.requests_success.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_error(&self) {
        self.requests_error.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn log_stats(&self) {
        let total = self.requests_total.load(Ordering::Relaxed);
        let success = self.requests_success.load(Ordering::Relaxed);
        let errors = self.requests_error.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        
        let avg_duration = if total > 0 { total_duration / total } else { 0 };
        
        tracing::info!(
            requests_total = total,
            requests_success = success,
            requests_error = errors,
            success_rate = if total > 0 { (success as f64 / total as f64) * 100.0 } else { 0.0 },
            avg_duration_ms = avg_duration,
            "Server metrics summary"
        );
    }
}
```

**Server Setup with Both Endpoints:**

```rust
// src/main.rs (updated)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting hello-world-grpc server");
    
    let config = config::load_config()?;
    let grpc_addr = config.server.grpc_address.parse()?;
    let health_port = config.server.health_port;
    
    let metrics = SimpleMetrics::new();
    let greeter = GreeterService::new(metrics.clone());
    
    // Start HTTP health check server
    tokio::spawn(start_health_server(health_port));
    
    // Start periodic metrics logging
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            metrics_clone.log_stats();
        }
    });
    
    // Start gRPC server
    info!(grpc_address = %grpc_addr, health_port = health_port, "Starting servers");
    
    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_shutdown(grpc_addr, shutdown_signal())
        .await?;
    
    Ok(())
}
```

**What We Monitor:**

1. **gRPC Health**: Standard gRPC health check service
2. **HTTP Health**: Simple JSON endpoint (`GET /health`)
3. **Request Duration**: Track processing time for each request
4. **Request Counts**: Total, success, error counts with success rate
5. **Average Performance**: Average request duration over time
6. **Server Lifecycle**: Startup, shutdown, and periodic stats

**Monitoring Endpoints:**
- **gRPC**: Health check via `grpc_health_v1.Health/Check`
- **HTTP**: `GET http://localhost:8081/health` (JSON response)

**What We DON'T Monitor (KISS):**
- Complex metrics systems (Prometheus, Grafana)
- Request duration histograms/percentiles
- Memory/CPU system metrics
- Distributed tracing systems
- Alert/notification systems
- Request payload analysis

This provides essential observability with both gRPC-native and simple HTTP health checks, plus request duration tracking for performance insights.

---

## 7. Work Scenarios

Our **essential scenarios** that validate the core functionality:

**Scenario 1: Successful Greeting Request**
```
Given: Server is running and healthy
When: Client sends valid HelloRequest with name "Alice"
Then: Server responds with HelloReply containing "Hello, Alice!"
And: Request is logged with duration
And: Success metrics are updated
```

**Scenario 2: Invalid Input Handling**
```
Given: Server is running and healthy  
When: Client sends HelloRequest with empty name ""
Then: Server responds with Status::invalid_argument error
And: Error is logged with client context
And: Error metrics are updated
And: Client receives clear error message
```

**Scenario 3: Health Check (HTTP)**
```
Given: Server is running on port 8081
When: HTTP client sends GET /health
Then: Server responds with 200 OK
And: Response contains JSON with service status
And: Response includes timestamp and version
```

**Scenario 4: Health Check (gRPC)**
```
Given: Server is running with health service
When: gRPC client calls grpc_health_v1.Health/Check
Then: Server responds with SERVING status
And: Health check succeeds
```

**Scenario 5: Graceful Shutdown**
```
Given: Server is running and processing requests
When: SIGTERM or Ctrl+C is received
Then: Server stops accepting new connections
And: Existing requests complete processing
And: Server shuts down gracefully
And: Shutdown is logged
```

**Scenario 6: Concurrent Requests**
```
Given: Server is running and healthy
When: Multiple clients send greeting requests simultaneously
Then: All requests are processed independently
And: Each request is logged with unique identifiers
And: Request count metrics are accurate
And: Average duration is calculated correctly
```

**Scenario 7: Server Startup**
```
Given: Configuration is valid
When: Server starts up
Then: gRPC server binds to configured address
And: HTTP health server starts on configured port
And: Metrics collection is initialized  
And: Startup is logged with addresses
And: Health endpoints become available
```

**Test Categories:**

1. **Unit Tests** (TDD approach):
   - Domain validation logic (`PersonName::new()`)
   - Business logic (greeting message creation)
   - Utility functions (`extract_client_info`)

2. **Integration Tests**:
   - End-to-end gRPC communication
   - Health endpoint functionality
   - Error handling flows
   - Graceful shutdown behavior

3. **Manual Testing Scenarios**:
   - Load testing with multiple concurrent requests
   - Network interruption handling
   - Configuration validation
   - Log output verification

**Expected Outcomes:**
- All valid requests succeed with appropriate logging
- Invalid requests fail fast with clear error messages
- Health checks accurately reflect server state
- Metrics provide useful operational insights
- Server handles lifecycle events gracefully

These scenarios cover our core functionality while keeping the scope manageable for our learning project.

---

## 8. Configuration Approach

Following **12-factor app principles** with minimal complexity:

**Configuration Structure:**

```rust
// src/config.rs
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub grpc_address: String,
    pub health_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                grpc_address: "127.0.0.1:50051".to_string(),
                health_port: 8081,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
            },
        }
    }
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        // Start with defaults
        .add_source(Config::try_from(&AppConfig::default())?)
        // Add config file if it exists (optional)
        .add_source(File::with_name("config/settings").required(false))
        // Override with environment variables (APP_SERVER__GRPC_ADDRESS, etc.)
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?;

    config.try_deserialize()
}
```

**Configuration Sources (Priority: highest to lowest):**

1. **Environment Variables**: `APP_SERVER__GRPC_ADDRESS=0.0.0.0:50051`
2. **Config File**: `config/settings.toml` (optional for development)
3. **Code Defaults**: Hardcoded reasonable defaults

**Sample Configuration File** (`config/settings.toml`):
```toml
[server]
grpc_address = "127.0.0.1:50051"
health_port = 8081

[logging]
level = "debug"
format = "json"
```

**Environment Variables** (for production):
```bash
# Server configuration
export APP_SERVER__GRPC_ADDRESS="0.0.0.0:50051"
export APP_SERVER__HEALTH_PORT="8081"

# Logging configuration  
export APP_LOGGING__LEVEL="info"
export APP_LOGGING__FORMAT="json"
```

**Sample `.env.example` file:**
```bash
# gRPC Server Configuration
APP_SERVER__GRPC_ADDRESS=127.0.0.1:50051
APP_SERVER__HEALTH_PORT=8081

# Logging Configuration
APP_LOGGING__LEVEL=info
APP_LOGGING__FORMAT=pretty
```

**Configuration Validation:**

```rust
impl AppConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate gRPC address can be parsed
        self.server.grpc_address.parse::<SocketAddr>()
            .context("Invalid gRPC address format")?;
        
        // Validate health port is in valid range
        if self.server.health_port < 1024 {
            anyhow::bail!("Health port must be >= 1024");
        }
        
        // Validate log level
        match self.logging.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => anyhow::bail!("Invalid log level: {}", self.logging.level),
        }
        
        Ok(())
    }
}
```

**Usage in main.rs:**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load and validate configuration early
    let config = config::load_config()
        .context("Failed to load configuration")?;
    
    config.validate()
        .context("Configuration validation failed")?;
    
    // Initialize logging with config
    init_tracing(&config.logging)?;
    
    tracing::info!(
        grpc_address = %config.server.grpc_address,
        health_port = config.server.health_port,
        log_level = %config.logging.level,
        "Server configuration loaded"
    );
    
    // Use config throughout the application
    let grpc_addr = config.server.grpc_address.parse()?;
    // ... rest of server setup
    
    Ok(())
}
```

**Key Design Decisions:**

1. **Simple Structure**: Only essential configuration options
2. **Sensible Defaults**: Works out of the box for development  
3. **Environment Override**: Production can override via env vars
4. **Fail Fast**: Validation at startup, not runtime
5. **Clear Naming**: Hierarchical structure with double underscores

**What We Configure:**
- Server addresses and ports
- Logging level and format
- Basic operational settings

**What We DON'T Configure (KISS):**
- Database connections (no DB yet)
- Authentication settings (no auth yet)  
- Complex middleware settings
- Performance tuning parameters
- Feature flags

This approach is **production-ready** while maintaining **simplicity** and following established patterns.

---

## 9. Logging Approach

**Structured logging** with `tracing` ecosystem, optimized for both **development** and **production**:

**Logging Initialization:**

```rust
// src/utils.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use crate::config::LoggingConfig;

pub fn init_tracing(config: &LoggingConfig) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))?;

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match config.format.as_str() {
        "json" => {
            // Production: JSON format for log aggregation
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);
            
            subscriber.with(json_layer).init();
        }
        "pretty" => {
            // Development: Human-readable format
            let pretty_layer = tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(false)
                .with_thread_ids(false);
            
            subscriber.with(pretty_layer).init();
        }
        _ => anyhow::bail!("Invalid logging format: {}", config.format),
    }

    Ok(())
}
```

**Request/Response Logging Pattern:**

```rust
// Consistent structured logging fields across all services
use tracing::{info, warn, error, Instrument};
use uuid::Uuid;

// Generate request ID for tracing
let request_id = Uuid::new_v4();

// Request start
info!(
    request_id = %request_id,
    method = "SayHello",
    client_addr = %client_addr,
    "Processing gRPC request"
);

// Business logic with span
async {
    // ... actual processing
}.instrument(tracing::info_span!(
    "greeting_processing",
    request_id = %request_id,
    name = %person_name.as_str()
)).await;

// Request completion
info!(
    request_id = %request_id,
    method = "SayHello", 
    client_addr = %client_addr,
    duration_ms = duration.as_millis(),
    name = %person_name.as_str(),
    "Successfully processed gRPC request"
);
```

**Error Logging Pattern:**

```rust
// Domain validation errors
warn!(
    request_id = %request_id,
    client_addr = %client_addr,
    method = "SayHello",
    error = %validation_error,
    input = %raw_input,
    "Invalid request data"
);

// Internal server errors  
error!(
    request_id = %request_id,
    method = "SayHello",
    error = %internal_error,
    error_chain = ?internal_error.chain().collect::<Vec<_>>(),
    "Internal server error occurred"
);

// Network/infrastructure errors
error!(
    error = %network_error,
    "gRPC server binding failed"
);
```

**Application Lifecycle Logging:**

```rust
// Server startup
info!(
    grpc_address = %grpc_addr,
    health_port = health_port,
    log_level = %log_level,
    version = env!("CARGO_PKG_VERSION"),
    "hello-world-grpc server starting"
);

// Graceful shutdown
warn!(
    signal = "SIGTERM",
    "Received shutdown signal, gracefully stopping server"
);

info!("Server shut down complete");

// Periodic metrics (every 60s)
info!(
    requests_total = total_requests,
    requests_success = successful_requests,
    requests_error = error_requests,
    success_rate = success_percentage,
    avg_duration_ms = average_duration,
    "Server metrics summary"
);
```

**Log Levels Usage:**

- **ERROR**: Server startup failures, internal errors, panic conditions
- **WARN**: Invalid client input, shutdown signals, configuration warnings  
- **INFO**: Request/response logging, server lifecycle, periodic metrics
- **DEBUG**: Domain validation details, configuration loading, detailed flow
- **TRACE**: Low-level protocol details, performance profiling (rarely used)

**Logging Standards:**

1. **Consistent Fields**:
   - `request_id`: UUID for request tracing
   - `method`: gRPC method name 
   - `client_addr`: Client IP address
   - `duration_ms`: Request processing time
   - `error`: Error messages and context

2. **Structured Data**: 
   - Use field-value pairs instead of formatted strings
   - Enable easy filtering and analysis in production
   - Support JSON format for log aggregation tools

3. **Context Preservation**:
   - Preserve error chains with `error_chain`
   - Include relevant business context (user names, IDs)
   - Use spans for operation grouping

**Development vs Production:**

**Development** (`APP_LOGGING__FORMAT=pretty`):
```
2024-01-15T10:30:45.123Z INFO hello_world_grpc: Processing gRPC request
    at src/services/hello_world.rs:45
    with request_id: 550e8400-e29b-41d4-a716-446655440000, method: "SayHello", client_addr: 127.0.0.1:54321
```

**Production** (`APP_LOGGING__FORMAT=json`):
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO", 
  "fields": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "method": "SayHello",
    "client_addr": "127.0.0.1:54321",
    "duration_ms": 15
  },
  "target": "hello_world_grpc::services::hello_world",
  "message": "Successfully processed gRPC request"
}
```

**What We Log:**
- All gRPC requests/responses with timing
- Server lifecycle events (start/stop/signals)
- Domain validation errors with context
- Periodic performance metrics
- Configuration loading results
- Health check status changes

**What We DON'T Log (KISS + Security):**
- Sensitive user data (beyond names for greetings)
- Full request/response payloads
- Internal system details (memory usage, etc.)
- Debug information in production
- Third-party service details (none yet)

**Environment-Based Configuration:**

```bash
# Development
export APP_LOGGING__LEVEL=debug
export APP_LOGGING__FORMAT=pretty

# Production  
export APP_LOGGING__LEVEL=info
export APP_LOGGING__FORMAT=json

# Override with RUST_LOG for granular control
export RUST_LOG=hello_world_grpc=debug,tonic=info
```

This logging approach provides **excellent observability** while remaining **simple and focused**, supporting both local development and production operations effectively.

---

## Summary

This technical vision establishes a **simple yet production-ready** foundation for our gRPC learning project in Rust. By following KISS principles while incorporating industry best practices, we create an ideal environment for:

- **Learning gRPC fundamentals** with hands-on implementation
- **Practicing modern Rust patterns** with async/await and strong typing  
- **Building production-grade habits** from the beginning
- **Iterative development** with clear extension points

The foundation supports **immediate productivity** while providing **room to grow** as we add complexity and explore advanced gRPC features.
