# Development Conventions

> **Reference**: This document distills essential development rules from [@vision.md](./vision.md). Refer to the vision document for complete context, rationale, and detailed examples.

## Core Principles

**KISS First**: Start with the simplest solution that works. Add complexity only when absolutely necessary.

**Fail Fast**: Validate at domain boundaries. Use `anyhow::Result<T>` and `?` operator for error propagation.

**Production Ready**: Write code that can run in production immediately with proper logging, graceful shutdown, and health checks.

## Code Quality Rules

### Error Handling
- Use `Result<T, E>` for all fallible operations
- Use `anyhow::Result<T>` for application-level errors
- Add context with `.context()` for debugging
- Never use `.unwrap()` in production code
- Map domain errors to appropriate gRPC Status codes

### Async Programming
- Use `async fn` for all I/O operations
- Use `tokio::spawn` for task concurrency
- Use `tokio::sync` primitives instead of `std::sync`
- Handle backpressure in streaming operations
- Avoid blocking calls in async context

### Domain Design
- Separate domain models from protobuf transport models
- Implement domain validation in business logic layer
- Use rich domain types instead of primitive strings/numbers
- Validate input at domain boundaries

### Code Organization
- Follow Rust naming conventions (snake_case, PascalCase)
- Keep functions small (< 20 lines when possible)
- Use clear, descriptive names (`is_ready`, `has_data`)
- Separate concerns into focused modules
- No `mod.rs` files - use Rust 2018+ module system

### Resource Management
- Use RAII pattern for cleanup (Drop trait)
- Implement graceful shutdown with signal handling
- Use bounded channels to prevent memory bloat
- Prefer borrowing (`&T`) over cloning when possible

## Testing Requirements

### Test-Driven Development
- Write unit tests first for all public functions
- Use `#[tokio::test]` for async test functions
- Add integration tests as functionality grows
- Aim for high test coverage on critical paths

### Test Structure
- Unit tests for domain validation logic
- Integration tests for end-to-end gRPC communication
- Manual testing scenarios for load and network interruption

## Documentation Standards

### API Documentation
- Write `///` doc comments for all public APIs
- Include usage examples in documentation
- Document error conditions and panics
- Keep README and inline docs synchronized

### Logging Standards
- Use structured logging with `tracing`
- Include consistent fields: `request_id`, `method`, `client_addr`, `duration_ms`
- Log request/response with timing information
- Use appropriate log levels: ERROR for failures, WARN for invalid input, INFO for operations

## gRPC Specific

### Service Implementation
- Keep protobuf schemas simple and forward-compatible
- Handle network errors and timeouts gracefully
- Implement proper health checks (both gRPC and HTTP)
- Log all requests with structured data and duration

### Production Readiness
- Use environment variables for configuration
- Implement graceful shutdown handling
- Add request tracing for debugging
- Handle client disconnections gracefully

## Performance Guidelines

### Optimization Approach
- Profile before optimizing
- Avoid unnecessary allocations and clones
- Use iterator chains over manual loops where clear
- Leverage Rust's type system for compile-time guarantees

### Async Performance  
- Use `select!` for racing multiple async operations
- Minimize async overhead where sync suffices
- Use `Arc<T>` for shared immutable data across threads
- Use `Arc<Mutex<T>>` for shared mutable state

## Configuration Management

- Use layered configuration: defaults → config file → environment variables
- Validate configuration at startup, fail fast on errors
- Support both development (pretty logs) and production (JSON logs) formats
- Keep configuration minimal and focused on operational needs

## What is FORBIDDEN

### Architecture & External Dependencies
- ❌ External databases (PostgreSQL, Redis, etc.)
- ❌ Microservices or complex distributed architecture
- ❌ Complex monitoring systems (Prometheus, Grafana, etc.)
- ❌ Authentication/authorization systems
- ❌ Service discovery mechanisms
- ❌ Load balancing libraries

### Code Complexity
- ❌ Redundant abstractions and wrapper classes
- ❌ Async/await for pure business logic (use only for I/O operations)
- ❌ Complex configuration files (YAML with many options, nested JSON)
- ❌ Over-engineering simple functionality
- ❌ Clever code over readable code

### Testing & Documentation
- ❌ Full test coverage requirements (focus on critical paths only)
- ❌ Complex testing frameworks and mocking systems
- ❌ Over-documentation of internal implementation details
- ❌ Auto-generated documentation without human review

## What is REQUIRED

### Code Structure
- ✅ Simple functions with single responsibility
- ✅ One file - one clear responsibility
- ✅ Minimal classes/structs focused on domain needs
- ✅ Clear, descriptive names for variables and functions
- ✅ Functions under 20 lines when possible

### Essential Operations
- ✅ Basic logging of all operations with structured data
- ✅ Proper error handling for all fallible operations
- ✅ Domain validation at boundaries
- ✅ Graceful shutdown handling
- ✅ Basic health checks (HTTP + gRPC)

### Documentation Requirements
- ✅ Document only public API functions with `///` comments
- ✅ Include usage examples for public APIs
- ✅ Document error conditions and expected behavior
- ✅ Keep documentation minimal but accurate

### Configuration & Environment
- ✅ Environment variable configuration support
- ✅ Sensible defaults for development
- ✅ Configuration validation at startup
- ✅ Simple TOML config files when needed

---

*For complete technical specifications, examples, and rationale, see [@vision.md](./vision.md)*
