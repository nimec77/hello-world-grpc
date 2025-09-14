# hello-world-grpc

A simple gRPC Hello World service implementation in Rust, built for learning gRPC fundamentals and modern Rust async patterns.

## Features

- **Simple gRPC Service**: Basic greeting service with unary RPC
- **Production Ready**: Structured logging, health checks, and graceful shutdown
- **Modern Rust**: Built with async/await, strong typing, and error handling
- **Environment Configuration**: Configurable via environment variables
- **Observability**: Built-in metrics and structured logging

## Quick Start

```bash
# Clone and build
git clone <repository-url>
cd hello-world-grpc
cargo build

# Run the server
cargo run

# Check health
curl http://localhost:8081/health
```

## Project Structure

```
hello-world-grpc/
├── Cargo.toml              # Dependencies and project configuration
├── build.rs                # Protobuf build script
├── proto/                  # Protocol Buffer definitions
├── src/
│   ├── main.rs             # Server entry point
│   ├── lib.rs              # Library exports and modules
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types and handling
│   ├── utils.rs            # Common utilities
│   └── services/           # gRPC service implementations
└── tests/                  # Integration tests
```

## Development

This project follows the technical vision outlined in [`vision.md`](vision.md) and development plan in [`doc/tasklist.md`](doc/tasklist.md).

### Key Principles

- **KISS (Keep It Simple, Stupid)**: Start simple, add complexity only when needed
- **Production Ready**: Proper logging, error handling, and monitoring from the start
- **Test-Driven**: Write tests first, ensure reliability

## Configuration

The service can be configured via environment variables:

```bash
# Server configuration
APP_SERVER__GRPC_ADDRESS=127.0.0.1:50051
APP_SERVER__HEALTH_PORT=8081

# Logging configuration
APP_LOGGING__LEVEL=info
APP_LOGGING__FORMAT=pretty
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
