# Manual Testing Guide

This guide provides comprehensive manual testing procedures for the Hello World gRPC service to validate functionality, performance, and configuration options.

## Prerequisites

### Required Tools

- **Rust toolchain**: `cargo` for building and running the server
- **curl**: For HTTP health check testing  
- **netcat (nc)**: For port availability checking

### Optional Tools (Enhanced Testing)

- **jq**: For pretty JSON formatting (`brew install jq` or `apt install jq`)
- **grpcurl**: For gRPC testing (`brew install grpcurl`)

### Installation Commands

```bash
# macOS
brew install jq grpcurl

# Ubuntu/Debian
sudo apt install jq
# grpcurl installation: https://github.com/fullstorydev/grpcurl#installation

# Arch Linux
sudo pacman -S jq
```

## Quick Start

### Automated Testing Suite

Run the comprehensive automated test suite:

```bash
./scripts/manual_tests.sh
```

This script will:
- ✅ Test default configuration startup
- ✅ Test environment variable configuration  
- ✅ Test gRPC communication (if grpcurl available)
- ✅ Test concurrent requests
- ✅ Validate log formats
- ✅ Test health endpoints

### Individual Test Scenarios

For focused testing, run individual scenarios below.

---

## Test Scenarios

### 1. Basic Server Startup and Health Checks

**Objective**: Verify server starts correctly with default configuration

#### Steps:

1. **Start the server**:
   ```bash
   cargo run
   ```

2. **Verify server startup logs**:
   - Look for: `"Starting Hello World gRPC Server with configuration"`  
   - Should show: `grpc_address`, `health_port`, `log_level`, `version`

3. **Test HTTP health endpoint**:
   ```bash
   curl -i http://localhost:8081/health
   ```

4. **Expected HTTP response**:
   ```
   HTTP/1.1 200 OK
   content-type: application/json
   
   {
     "status": "healthy",
     "service": "hello-world-grpc", 
     "timestamp": "2025-09-20T10:30:45.123Z",
     "version": "0.1.0"
   }
   ```

5. **Test gRPC health endpoint** (requires grpcurl):
   ```bash
   grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
   ```

6. **Expected gRPC health response**:
   ```json
   {
     "status": "SERVING"
   }
   ```

#### Success Criteria:
- ✅ Server starts without errors
- ✅ HTTP health returns 200 with correct JSON
- ✅ gRPC health returns SERVING status
- ✅ Logs show structured startup information

---

### 2. Environment Variable Configuration Testing

**Objective**: Verify server respects environment variable configuration

#### Test Cases:

##### A. JSON Logging Format
```bash
export APP__LOGGING__FORMAT=json
export APP__LOGGING__LEVEL=debug
cargo run
```

**Expected**: Log entries are JSON formatted:
```json
{"timestamp":"2025-09-20T10:30:45.123Z","level":"INFO","message":"Starting Hello World gRPC Server","target":"hello_world_grpc"}
```

##### B. Custom Ports
```bash
export APP__SERVER__HEALTH_PORT=8082
cargo run
```

**Test**: `curl http://localhost:8082/health` (should work on port 8082)

##### C. Pretty Logging (Development)
```bash
export APP__LOGGING__FORMAT=pretty  
export APP__LOGGING__LEVEL=info
cargo run
```

**Expected**: Human-readable logs:
```
2025-09-20T10:30:45.123Z  INFO hello_world_grpc: Starting Hello World gRPC Server
```

#### Success Criteria:
- ✅ JSON format produces valid JSON logs
- ✅ Custom ports are respected
- ✅ Pretty format is human-readable
- ✅ Log levels filter appropriately

---

### 3. gRPC Communication Testing

**Objective**: Verify gRPC service functionality and error handling

#### Prerequisites:
```bash
# Install grpcurl if not available
brew install grpcurl  # macOS
# or follow: https://github.com/fullstorydev/grpcurl#installation
```

#### Test Cases:

##### A. Valid Requests
```bash
# Start server
cargo run &

# Test valid greeting
grpcurl -plaintext -d '{"name": "Alice"}' localhost:50051 HelloService/SayHello
```

**Expected**:
```json
{
  "message": "Hello, Alice!"
}
```

##### B. Whitespace Handling
```bash
grpcurl -plaintext -d '{"name": "  Bob  "}' localhost:50051 hello_world.Greeter/SayHello
```

**Expected**:
```json
{
  "message": "Hello, Bob!"
}
```

##### C. Error Conditions

**Empty name**:
```bash
grpcurl -plaintext -d '{"name": ""}' localhost:50051 hello_world.Greeter/SayHello
```

**Expected**: gRPC error with status `INVALID_ARGUMENT`

**Too long name**:
```bash
grpcurl -plaintext -d '{"name": "'$(python3 -c 'print("a" * 101)')'"}' localhost:50051 hello_world.Greeter/SayHello
```

**Expected**: gRPC error with status `INVALID_ARGUMENT`

##### D. Service Discovery
```bash
grpcurl -plaintext localhost:50051 list
```

**Expected**:
```
hello_world.Greeter
grpc.health.v1.Health
```

#### Success Criteria:
- ✅ Valid requests return correct greetings
- ✅ Whitespace is properly trimmed
- ✅ Empty/invalid names return INVALID_ARGUMENT errors
- ✅ Service listing shows expected services

---

### 4. Concurrent Request Testing

**Objective**: Verify server handles multiple simultaneous requests

#### Manual Concurrent Test:
```bash
# Start server
cargo run &

# Send 5 concurrent requests (requires bash)
for i in {1..5}; do
  grpcurl -plaintext -d "{\"name\": \"User$i\"}" localhost:50051 hello_world.Greeter/SayHello &
done
wait

# Or use the Python test client
./scripts/test_client.py --test concurrent --concurrent 20
```

#### HTTP Load Test:
```bash
# Test HTTP endpoint concurrency  
for i in {1..10}; do
  curl -s http://localhost:8081/health &
done
wait
```

#### Success Criteria:
- ✅ All concurrent requests complete successfully
- ✅ Each request gets correct personalized response
- ✅ Server remains responsive during load
- ✅ No connection errors or timeouts

---

### 5. Server Lifecycle Testing

**Objective**: Verify graceful startup and shutdown behavior

#### Startup Testing:

1. **Monitor startup sequence**:
   ```bash
   cargo run 2>&1 | grep -E "(Starting|started|listen)"
   ```

2. **Expected startup log sequence**:
   ```
   Starting Hello World gRPC Server with configuration
   gRPC server will listen on 127.0.0.1:50051
   HTTP health check server will start on port 8081
   Started periodic metrics logging (every 60 seconds)
   Started health check servers (gRPC + HTTP)
   All services configured, starting gRPC server
   ```

#### Graceful Shutdown Testing:

1. **Start server and send SIGTERM**:
   ```bash
   cargo run &
   SERVER_PID=$!
   sleep 2
   kill -TERM $SERVER_PID
   ```

2. **Monitor shutdown logs**:
   - Should see graceful shutdown messages
   - No abrupt termination errors

#### Port Binding Testing:

1. **Test port conflicts**:
   ```bash
   # Start first server
   cargo run &
   sleep 2
   
   # Try to start second server (should fail gracefully)
   cargo run
   ```

#### Success Criteria:
- ✅ Startup sequence completes in order
- ✅ All services become available after startup
- ✅ Graceful shutdown on SIGTERM
- ✅ Port conflicts are handled appropriately

---

### 6. Metrics and Observability Testing

**Objective**: Verify metrics collection and periodic reporting

#### Metrics Testing:

1. **Start server and make requests**:
   ```bash
   cargo run &
   sleep 2
   
   # Make several requests to generate metrics
   for i in {1..10}; do
     grpcurl -plaintext -d "{\"name\": \"User$i\"}" localhost:50051 hello_world.Greeter/SayHello
   done
   
   # Wait for metrics log (every 60s) or check immediately by restarting
   ```

2. **Expected metrics log** (every 60 seconds):
   ```
   INFO hello_world_grpc::utils: Server metrics summary
     requests_total=10
     requests_success=10  
     requests_error=0
     success_rate=100
     avg_duration_ms=2
   ```

#### Log Structure Validation:

1. **Check log consistency**:
   ```bash
   cargo run 2>&1 | grep -E "request_id|duration_ms|client_addr"
   ```

2. **Expected structured fields**:
   - `request_id`: UUID format
   - `duration_ms`: Numeric value  
   - `client_addr`: IP address
   - `method`: "SayHello"

#### Success Criteria:
- ✅ Metrics are collected for all requests
- ✅ Periodic metrics logging works (60s intervals)
- ✅ Success/error rates are accurate
- ✅ Request timing data is captured

---

## Advanced Testing with Python Client

### Using the Test Client

The Python test client (`scripts/test_client.py`) provides programmatic testing capabilities:

#### Basic Usage:
```bash
# Single request test
./scripts/test_client.py --test single --name "TestUser"

# Concurrent requests test
./scripts/test_client.py --test concurrent --concurrent 50

# Error conditions test
./scripts/test_client.py --test errors

# Load testing
./scripts/test_client.py --test load --load-duration 30 --load-rps 20

# All tests
./scripts/test_client.py --test all --output-json results.json
```

#### Expected Outputs:

**Single Request**:
```
✅ Success: Hello, TestUser! (1.2ms)
```

**Concurrent Requests** (50 requests):
```
📊 Concurrent Test Results:
  Total requests: 50
  Successful: 50
  Failed: 0
  Success rate: 100.0%
  Total duration: 145.3ms
  Avg duration: 2.9ms
  Min duration: 1.1ms  
  Max duration: 8.7ms
```

**Error Conditions**:
```
✅ PASS: INVALID_ARGUMENT: Person name cannot be empty (0.8ms)
✅ PASS: INVALID_ARGUMENT: Person name cannot be empty (0.7ms)
✅ PASS: INVALID_ARGUMENT: Person name cannot exceed 100 characters (0.9ms)
✅ PASS: Hello, Valid Name! (1.3ms)
```

---

## Troubleshooting Common Issues

### Server Won't Start

**Issue**: `Address already in use`
```bash
# Find process using port
lsof -i :50051
# or
netstat -tulpn | grep 50051

# Kill existing process
kill <PID>
```

**Issue**: `Permission denied` for health port
```bash
# Use port > 1024 or run with sudo
export APP__SERVER__HEALTH_PORT=8081
```

### gRPC Connection Issues

**Issue**: `Connection refused`
- Verify server is running: `ps aux | grep hello-world-grpc`  
- Check port binding: `netstat -tulpn | grep 50051`
- Check firewall settings

**Issue**: `grpcurl` not found
```bash
# Install grpcurl
brew install grpcurl  # macOS
# or follow: https://github.com/fullstorydev/grpcurl#installation
```

### Health Check Issues

**Issue**: HTTP health check returns 404
- Verify server started successfully
- Check health port configuration: `APP__SERVER__HEALTH_PORT`
- Confirm server logs show: `"HTTP health check server started"`

### Configuration Issues

**Issue**: Environment variables not working
```bash
# Verify environment variables are set
env | grep APP__

# Use correct format: APP__SECTION__FIELD
export APP__LOGGING__FORMAT=json  # ✅ Correct
export APP_LOGGING_FORMAT=json    # ❌ Wrong
```

---

## Performance Benchmarks

### Expected Performance

Based on integration tests and manual testing:

- **Single Request Latency**: 1-5ms (localhost)
- **Concurrent Throughput**: >1000 requests/second (localhost)
- **Memory Usage**: <10MB base + ~1KB per concurrent connection
- **Startup Time**: <1 second
- **Health Check Response**: <1ms

### Load Testing Guidelines

**Light Load**: 10 req/s for 30s
```bash
./scripts/test_client.py --test load --load-rps 10 --load-duration 30
```

**Medium Load**: 100 req/s for 60s  
```bash
./scripts/test_client.py --test load --load-rps 100 --load-duration 60
```

**Heavy Load**: 500 req/s for 10s
```bash
./scripts/test_client.py --test load --load-rps 500 --load-duration 10
```

### Success Criteria for Load Tests:
- ✅ Success rate > 99%
- ✅ Average response time < 10ms
- ✅ No connection errors
- ✅ Server remains responsive to health checks

---

## Continuous Integration Testing

For CI/CD pipelines, use the automated test suite:

```bash
# Run all manual tests in CI
./scripts/manual_tests.sh

# Exit code 0 = all tests passed
echo $?
```

### CI Test Matrix

Recommended environment combinations for CI:

1. **Default Configuration**
   ```bash
   ./scripts/manual_tests.sh
   ```

2. **JSON Logging + Debug Level**
   ```bash
   APP__LOGGING__FORMAT=json APP__LOGGING__LEVEL=debug ./scripts/manual_tests.sh
   ```

3. **Custom Ports**
   ```bash
   APP__SERVER__HEALTH_PORT=9090 ./scripts/manual_tests.sh
   ```

4. **Load Testing**
   ```bash
   ./scripts/test_client.py --test load --load-duration 10 --load-rps 50
   ```

---

## Conclusion

This manual testing guide provides comprehensive validation of:

- ✅ Basic functionality and gRPC communication
- ✅ Configuration management and environment variables
- ✅ Error handling and validation
- ✅ Concurrent request processing
- ✅ Health monitoring and observability
- ✅ Server lifecycle management
- ✅ Performance characteristics

For automated testing, prefer running `./scripts/manual_tests.sh` which covers all scenarios. For specific testing needs, use the individual test procedures or the Python test client for programmatic testing.

All tests should pass consistently before considering the service production-ready.
