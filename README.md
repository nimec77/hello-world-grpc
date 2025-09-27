# hello-world-grpc

A production-ready gRPC Hello World service implementation in Rust, demonstrating modern async patterns, comprehensive error handling, and operational best practices.

## 🎉 Project Status: COMPLETED

**All phases successfully implemented with full production readiness**
- ✅ **79 comprehensive tests passing** (unit + integration + streaming + error handling)
- ✅ **Production-ready features**: Graceful shutdown, structured error handling, health checks, metrics, streaming
- ✅ **Server-side streaming**: Real-time time updates with configurable intervals and monitoring
- ✅ **Comprehensive manual testing suite** with automation scripts

## Features

- **🚀 Production gRPC Service**: Greeting service with domain validation and error handling
- **🕐 Server-Side Streaming**: Real-time time streaming with configurable intervals and metrics
- **⚡ Modern Rust**: Built with async/await, strong typing, and comprehensive error handling  
- **📊 Observability**: Structured logging (JSON/Pretty), metrics collection, dual health checks
- **⚙️ Configuration**: Layered configuration (defaults → file → environment variables)
- **🛡️ Robust Error Handling**: Structured error types with proper gRPC status mapping
- **🔄 Graceful Operations**: Signal handling, graceful shutdown with timeouts
- **🧪 Comprehensive Testing**: 79 tests covering domain validation, integration, streaming, and error scenarios

## Quick Start

```bash
# Build and run the server
cargo build
cargo run

# Test the service
# 1. Check HTTP health endpoint
curl http://localhost:8081/health

# 2. Test gRPC unary service with grpcurl (if installed)
grpcurl -plaintext -d '{"name": "World"}' localhost:50051 hello_world.Greeter/SayHello

# 3. Test streaming service (real-time time updates)
grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime

# 4. Run comprehensive test suite
cargo test

# 5. Run manual testing scenarios (including streaming tests)
./scripts/manual_tests.sh

# 6. Run with custom configuration
APP_LOGGING__LEVEL=debug APP_LOGGING__FORMAT=json cargo run
```

## Project Structure

```
hello-world-grpc/
├── Cargo.toml              # Dependencies and project configuration  
├── build.rs                # Protobuf build script (tonic-prost-build)
├── proto/
│   └── hello_world.proto   # gRPC service definition
├── config/
│   ├── settings.toml       # Configuration file
│   └── settings.toml.example # Configuration template
├── src/
│   ├── main.rs             # Server entry point with graceful shutdown
│   ├── lib.rs              # Library exports and modules
│   ├── config.rs           # Layered configuration management
│   ├── error.rs            # Structured error types with gRPC mapping
│   ├── utils.rs            # Utilities (metrics, health, logging)
│   └── services/
│       └── hello_world.rs  # Greeter service implementation
├── tests/
│   ├── common.rs           # Shared test utilities
│   ├── integration/
│   │   └── hello_world_test.rs # Integration tests
│   └── integration_hello_world_test.rs # Main integration tests
├── scripts/
│   ├── manual_tests.sh     # Comprehensive testing script (418 lines)
│   └── test_client.py      # Python test client (359 lines)
├── doc/
│   ├── tasklist.md         # Development progress (completed)
│   ├── manual_testing.md   # Manual testing guide (573 lines)
│   └── workflow.md         # Development workflow
├── conventions.md          # Development conventions
└── vision.md               # Technical vision and architecture
```

## Development

This project demonstrates a **completed production-ready implementation** following the technical vision in [`vision.md`](vision.md). The full development journey is documented in [`doc/tasklist.md`](doc/tasklist.md).

### Architecture Highlights

- **🎯 KISS Principle**: Simple, focused implementation with clear separation of concerns
- **🏗️ Domain-Driven Design**: Rich domain types with validation at boundaries
- **⚡ Modern Async Rust**: Built with tokio, tonic 0.14.2, and latest ecosystem
- **🛡️ Production Patterns**: Structured errors, graceful shutdown, comprehensive logging
- **🧪 Test-Driven**: 46 comprehensive tests covering all critical paths

## Configuration

The service uses layered configuration (defaults → config file → environment variables):

```bash
# Server configuration
APP__SERVER__GRPC_ADDRESS=127.0.0.1:50051    # gRPC server bind address
APP__SERVER__HEALTH_PORT=8081                # HTTP health check port

# Logging configuration  
APP__LOGGING__LEVEL=info                      # Log level: trace|debug|info|warn|error
APP__LOGGING__FORMAT=pretty                   # Format: pretty|json

# Streaming configuration
APP__STREAMING__INTERVAL_SECONDS=1           # Time between stream updates (1-3600s)
APP__STREAMING__MAX_CONNECTIONS=100          # Max concurrent streaming connections (1-10000)
APP__STREAMING__TIMEOUT_SECONDS=300          # Stream timeout duration (1-86400s)

# Example production setup
export APP__LOGGING__FORMAT=json
export APP__LOGGING__LEVEL=info
export APP__SERVER__GRPC_ADDRESS=0.0.0.0:50051
export APP__STREAMING__MAX_CONNECTIONS=500
```

### Configuration Files

- `config/settings.toml` - Local development overrides
- `config/settings.toml.example` - Configuration template

## Testing

### Automated Test Suite (79 Tests)

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test categories
cargo test unit_tests        # Domain validation tests
cargo test integration       # End-to-end gRPC tests  
cargo test streaming         # Streaming functionality tests
cargo test error_handling    # Error scenario tests
```

### Manual Testing Suite

Comprehensive manual testing infrastructure for operational validation:

```bash
# Run full manual test suite (418 lines of test scenarios)
./scripts/manual_tests.sh

# Individual test categories:
./scripts/manual_tests.sh --basic          # Basic functionality
./scripts/manual_tests.sh --streaming      # Streaming functionality tests
./scripts/manual_tests.sh --health         # Health check endpoints
./scripts/manual_tests.sh --concurrency    # Concurrent request handling
./scripts/manual_tests.sh --errors         # Error handling scenarios
./scripts/manual_tests.sh --config         # Configuration validation
./scripts/manual_tests.sh --shutdown       # Graceful shutdown testing

# Python test client for programmatic testing
python scripts/test_client.py              # Interactive gRPC client
```

### Test Coverage

- **Unit Tests (52)**: Domain validation, configuration, utilities, streaming service
- **Integration Tests (13)**: End-to-end gRPC communication, health checks
- **Streaming Tests (11)**: End-to-end streaming, concurrent clients, performance, error recovery
- **Common Tests (3)**: Shared test infrastructure utilities
- **Manual Testing**: Load testing, network interruption, configuration edge cases, streaming scenarios
- **Documentation**: Complete testing guide in [`doc/manual_testing.md`](doc/manual_testing.md)

## Production Readiness

### Operational Features

- ✅ **Graceful Shutdown**: SIGTERM/Ctrl+C handling with 30s timeout
- ✅ **Health Checks**: HTTP (`/health`) and gRPC health service
- ✅ **Structured Logging**: JSON and pretty formats with request tracing
- ✅ **Metrics Collection**: Request counts, success rates, duration tracking
- ✅ **Error Handling**: Comprehensive error types with gRPC status mapping
- ✅ **Configuration**: Environment-based with validation at startup

### Monitoring Endpoints

```bash
# HTTP health check
curl http://localhost:8081/health

# gRPC health check (with grpcurl)
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check

# gRPC service introspection
grpcurl -plaintext localhost:50051 list
grpcurl -plaintext localhost:50051 describe hello_world.Greeter

# Test streaming functionality
grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
