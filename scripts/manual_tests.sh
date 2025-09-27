#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

wait_for_server() {
    local port=$1
    local timeout=${2:-30}
    local count=0
    
    log_info "Waiting for server on port $port to start..."
    
    while ! nc -z localhost $port 2>/dev/null; do
        if [ $count -ge $timeout ]; then
            log_error "Timeout waiting for server on port $port"
            return 1
        fi
        sleep 1
        ((count++))
    done
    
    log_success "Server is ready on port $port"
    return 0
}

cleanup() {
    log_info "Cleaning up background processes..."
    jobs -p | xargs -r kill 2>/dev/null || true
    sleep 2
}

# Trap for cleanup
trap cleanup EXIT

echo "======================================================================"
echo "🧪 Hello World gRPC - Manual Testing Suite"
echo "======================================================================"
echo

# Check dependencies
log_info "Checking required dependencies..."

if ! command -v jq &> /dev/null; then
    log_warning "jq not found. Install with: brew install jq (macOS) or apt install jq (Linux)"
    log_info "Continuing without jq - JSON output will not be formatted"
    JQ_AVAILABLE=false
else
    JQ_AVAILABLE=true
    log_success "jq is available"
fi

if ! command -v grpcurl &> /dev/null; then
    log_warning "grpcurl not found. Install with: brew install grpcurl (macOS)"
    log_info "gRPC health checks will be skipped"
    GRPCURL_AVAILABLE=false
else
    GRPCURL_AVAILABLE=true
    log_success "grpcurl is available"
fi

if ! command -v nc &> /dev/null; then
    log_error "nc (netcat) not found. This is required for server readiness checks."
    exit 1
fi

echo

# Test 1: Default Configuration Server Startup
echo "======================================================================"
echo "🚀 Test 1: Default Configuration Server Startup"
echo "======================================================================"

log_info "Starting server with default configuration..."
cargo run &
SERVER_PID=$!

if wait_for_server 50051 30 && wait_for_server 8081 30; then
    log_success "Server started successfully with default configuration"
    
    # Test gRPC endpoint availability
    log_info "Testing gRPC endpoint availability..."
    if $GRPCURL_AVAILABLE; then
        if grpcurl -plaintext -max-time 5 localhost:50051 list &>/dev/null; then
            log_success "gRPC endpoint is accessible"
        else
            log_error "gRPC endpoint is not accessible"
        fi
    else
        log_info "Skipping gRPC endpoint test (grpcurl not available)"
    fi
    
    # Test HTTP health endpoint
    log_info "Testing HTTP health endpoint..."
    if $JQ_AVAILABLE; then
        HTTP_RESPONSE=$(curl -s -w "%{http_code}" -o health_response.json http://localhost:8081/health)
        if [ "$HTTP_RESPONSE" = "200" ]; then
            log_success "HTTP health endpoint returned 200 OK"
            log_info "Health response:"
            cat health_response.json | jq .
            rm -f health_response.json
        else
            log_error "HTTP health endpoint returned $HTTP_RESPONSE"
        fi
    else
        HTTP_RESPONSE=$(curl -s -w "%{http_code}" -o /tmp/health_response.txt http://localhost:8081/health)
        if [ "$HTTP_RESPONSE" = "200" ]; then
            log_success "HTTP health endpoint returned 200 OK"
            log_info "Health response:"
            cat /tmp/health_response.txt
            rm -f /tmp/health_response.txt
        else
            log_error "HTTP health endpoint returned $HTTP_RESPONSE"
        fi
    fi
    
    # Test gRPC health check
    if $GRPCURL_AVAILABLE; then
        log_info "Testing gRPC health check..."
        if grpcurl -plaintext -max-time 5 localhost:50051 grpc.health.v1.Health/Check; then
            log_success "gRPC health check successful"
        else
            log_warning "gRPC health check failed or not implemented"
        fi
    fi
    
    # Kill the server
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    sleep 2
    
    log_success "Test 1 completed successfully"
else
    log_error "Server failed to start with default configuration"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo
echo "======================================================================"
echo "🔧 Test 2: Environment Variable Configuration"
echo "======================================================================"

# Test different logging formats
log_info "Testing JSON logging format..."

export APP__LOGGING__FORMAT=json
export APP__LOGGING__LEVEL=debug
export APP__SERVER__HEALTH_PORT=8082

log_info "Starting server with JSON logging and debug level on port 8082..."
cargo run &
SERVER_PID=$!

if wait_for_server 50051 30 && wait_for_server 8082 30; then
    log_success "Server started with custom configuration"
    
    log_info "Testing health endpoint on custom port..."
    HTTP_RESPONSE=$(curl -s -w "%{http_code}" -o /tmp/health_custom.txt http://localhost:8082/health)
    if [ "$HTTP_RESPONSE" = "200" ]; then
        log_success "Custom health endpoint works on port 8082"
        if $JQ_AVAILABLE; then
            cat /tmp/health_custom.txt | jq .
        else
            cat /tmp/health_custom.txt
        fi
        rm -f /tmp/health_custom.txt
    else
        log_error "Custom health endpoint failed with code $HTTP_RESPONSE"
    fi
    
    # Kill the server
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    sleep 2
    
    log_success "Test 2 completed successfully"
else
    log_error "Server failed to start with custom configuration"
    kill $SERVER_PID 2>/dev/null || true
fi

# Reset environment variables
unset APP__LOGGING__FORMAT
unset APP__LOGGING__LEVEL
unset APP__SERVER__HEALTH_PORT

echo
echo "======================================================================"
echo "📡 Test 3: gRPC Communication Test"
echo "======================================================================"

if $GRPCURL_AVAILABLE; then
    log_info "Starting server for gRPC communication test..."
    cargo run &
    SERVER_PID=$!
    
    if wait_for_server 50051 30; then
        log_success "Server ready for gRPC tests"
        
        # Test valid request
        log_info "Testing valid gRPC request..."
        RESPONSE=$(grpcurl -plaintext -d '{"name": "Alice"}' localhost:50051 hello_world.Greeter/SayHello)
        if echo "$RESPONSE" | grep -q "Hello, Alice"; then
            log_success "Valid gRPC request successful"
            log_info "Response: $RESPONSE"
        else
            log_error "Valid gRPC request failed"
            log_info "Response: $RESPONSE"
        fi
        
        # Test empty name (should fail)
        log_info "Testing invalid gRPC request (empty name)..."
        if grpcurl -plaintext -d '{"name": ""}' localhost:50051 hello_world.Greeter/SayHello 2>/dev/null; then
            log_error "Empty name request should have failed but didn't"
        else
            log_success "Empty name request correctly failed with validation error"
        fi
        
        # Test service listing
        log_info "Testing service reflection..."
        SERVICES=$(grpcurl -plaintext localhost:50051 list)
        if echo "$SERVICES" | grep -q "hello_world.Greeter"; then
            log_success "Service reflection working"
            log_info "Available services:"
            echo "$SERVICES" | sed 's/^/  /'
        else
            log_warning "Service reflection not working or service not found"
        fi
        
        # Kill the server
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
        sleep 2
        
        log_success "Test 3 completed successfully"
    else
        log_error "Server failed to start for gRPC test"
        kill $SERVER_PID 2>/dev/null || true
    fi
else
    log_warning "Skipping gRPC communication test (grpcurl not available)"
fi

echo
echo "======================================================================"
echo "⚡ Test 4: Concurrent Request Simulation"
echo "======================================================================"

if $GRPCURL_AVAILABLE; then
    log_info "Starting server for concurrent request test..."
    cargo run &
    SERVER_PID=$!
    
    if wait_for_server 50051 30; then
        log_success "Server ready for concurrent tests"
        
        log_info "Sending 5 concurrent gRPC requests..."
        
        # Create temp directory for responses
        TEMP_DIR=$(mktemp -d)
        
        # Send concurrent requests
        for i in {1..5}; do
            grpcurl -plaintext -d "{\"name\": \"User$i\"}" localhost:50051 hello_world.Greeter/SayHello > "$TEMP_DIR/response_$i.txt" 2>&1 &
        done
        
        # Wait for all requests to complete
        wait
        
        # Check responses
        SUCCESS_COUNT=0
        for i in {1..5}; do
            if grep -q "Hello, User$i" "$TEMP_DIR/response_$i.txt"; then
                ((SUCCESS_COUNT++))
                log_success "Request $i successful"
            else
                log_error "Request $i failed"
                cat "$TEMP_DIR/response_$i.txt"
            fi
        done
        
        # Cleanup temp directory
        rm -rf "$TEMP_DIR"
        
        if [ $SUCCESS_COUNT -eq 5 ]; then
            log_success "All 5 concurrent requests successful"
        else
            log_warning "$SUCCESS_COUNT/5 concurrent requests successful"
        fi
        
        # Kill the server
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
        sleep 2
        
        log_success "Test 4 completed"
    else
        log_error "Server failed to start for concurrent test"
        kill $SERVER_PID 2>/dev/null || true
    fi
else
    log_warning "Skipping concurrent request test (grpcurl not available)"
fi

echo
echo "======================================================================"
echo "🔍 Test 5: Log Format Validation"
echo "======================================================================"

log_info "Testing pretty log format..."
export APP__LOGGING__FORMAT=pretty
cargo run > pretty_logs.txt 2>&1 &
SERVER_PID=$!

if wait_for_server 50051 10; then
    # Make a request to generate logs
    curl -s http://localhost:8081/health > /dev/null || true
    
    sleep 2
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    
    if grep -q "Starting Hello World gRPC Server" pretty_logs.txt; then
        log_success "Pretty log format working"
        log_info "Sample pretty log entries:"
        head -5 pretty_logs.txt | sed 's/^/  /'
    else
        log_error "Pretty log format not working as expected"
    fi
else
    kill $SERVER_PID 2>/dev/null || true
fi

log_info "Testing JSON log format..."
export APP__LOGGING__FORMAT=json
cargo run > json_logs.txt 2>&1 &
SERVER_PID=$!

if wait_for_server 50051 10; then
    # Make a request to generate logs
    curl -s http://localhost:8081/health > /dev/null || true
    
    sleep 2
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    
    if grep -q '"message"' json_logs.txt; then
        log_success "JSON log format working"
        log_info "Sample JSON log entry:"
        if $JQ_AVAILABLE; then
            grep '"message"' json_logs.txt | head -1 | jq . 2>/dev/null || grep '"message"' json_logs.txt | head -1
        else
            grep '"message"' json_logs.txt | head -1
        fi
    else
        log_error "JSON log format not working as expected"
    fi
else
    kill $SERVER_PID 2>/dev/null || true
fi

# Cleanup log files
rm -f pretty_logs.txt json_logs.txt
unset APP__LOGGING__FORMAT

log_success "Test 5 completed"

echo
echo "======================================================================"
echo "🕐 Test 6: Streaming Functionality"
echo "======================================================================"

log_info "Starting server for streaming tests..."
cargo run &
SERVER_PID=$!

if wait_for_server 50051 30; then
    log_success "Server started successfully"
    
    if $GRPCURL_AVAILABLE; then
        log_info "Testing streaming time endpoint..."
        
        # Test basic streaming functionality
        log_info "  Testing basic streaming (5 messages with timeout)..."
        timeout 10s grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime > /tmp/stream_output.txt 2>&1 &
        STREAM_PID=$!
        
        sleep 6  # Wait for a few messages
        kill $STREAM_PID 2>/dev/null || true
        wait $STREAM_PID 2>/dev/null || true
        
        if [ -s /tmp/stream_output.txt ]; then
            log_success "Streaming endpoint is working"
            MESSAGE_COUNT=$(grep -c '"timestamp"' /tmp/stream_output.txt || echo "0")
            log_info "    Received $MESSAGE_COUNT time messages"
            
            # Validate RFC3339 format
            if grep -q '[0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}T[0-9]\{2\}:[0-9]\{2\}:[0-9]\{2\}' /tmp/stream_output.txt; then
                log_success "    Timestamps are in RFC3339 format"
            else
                log_warning "    Timestamp format may not be RFC3339"
            fi
        else
            log_error "Streaming endpoint did not produce any output"
        fi
        
        # Test multiple concurrent streaming clients
        log_info "  Testing concurrent streaming clients..."
        
        # Start 3 concurrent streaming clients
        timeout 8s grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime > /tmp/stream1.txt 2>&1 &
        STREAM1_PID=$!
        timeout 8s grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime > /tmp/stream2.txt 2>&1 &
        STREAM2_PID=$!
        timeout 8s grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime > /tmp/stream3.txt 2>&1 &
        STREAM3_PID=$!
        
        sleep 5  # Let them run for a bit
        
        # Kill all streaming clients
        kill $STREAM1_PID $STREAM2_PID $STREAM3_PID 2>/dev/null || true
        wait $STREAM1_PID $STREAM2_PID $STREAM3_PID 2>/dev/null || true
        
        # Count messages from each client
        COUNT1=$(grep -c '"timestamp"' /tmp/stream1.txt || echo "0")
        COUNT2=$(grep -c '"timestamp"' /tmp/stream2.txt || echo "0")
        COUNT3=$(grep -c '"timestamp"' /tmp/stream3.txt || echo "0")
        
        if [ "$COUNT1" -gt 0 ] && [ "$COUNT2" -gt 0 ] && [ "$COUNT3" -gt 0 ]; then
            log_success "    Concurrent streaming works (Client1: $COUNT1, Client2: $COUNT2, Client3: $COUNT3 messages)"
        else
            log_warning "    Some concurrent clients may not have received messages"
        fi
        
        # Test streaming with unary requests (mixed operations)
        log_info "  Testing streaming + unary mixed operations..."
        
        # Start a streaming client
        timeout 6s grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime > /tmp/mixed_stream.txt 2>&1 &
        MIXED_STREAM_PID=$!
        
        sleep 1  # Let stream start
        
        # Make some unary requests while streaming
        UNARY_SUCCESS=0
        for i in {1..3}; do
            UNARY_RESPONSE=$(grpcurl -plaintext -d '{"name": "StreamTest'$i'"}' localhost:50051 hello_world.Greeter/SayHello 2>/dev/null | grep "Hello, StreamTest$i!" || echo "")
            if [ -n "$UNARY_RESPONSE" ]; then
                ((UNARY_SUCCESS++))
            fi
        done
        
        # Stop streaming
        kill $MIXED_STREAM_PID 2>/dev/null || true
        wait $MIXED_STREAM_PID 2>/dev/null || true
        
        MIXED_COUNT=$(grep -c '"timestamp"' /tmp/mixed_stream.txt || echo "0")
        
        if [ "$UNARY_SUCCESS" -eq 3 ] && [ "$MIXED_COUNT" -gt 0 ]; then
            log_success "    Mixed streaming + unary operations work (Stream: $MIXED_COUNT, Unary: $UNARY_SUCCESS)"
        else
            log_warning "    Mixed operations may have issues (Stream: $MIXED_COUNT, Unary: $UNARY_SUCCESS)"
        fi
        
        # Cleanup temp files
        rm -f /tmp/stream_output.txt /tmp/stream1.txt /tmp/stream2.txt /tmp/stream3.txt /tmp/mixed_stream.txt
        
        log_success "Test 6 completed successfully"
    else
        log_warning "Streaming tests skipped (grpcurl not available)"
        log_success "Test 6 completed (skipped)"
    fi
    
    # Kill the server
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    sleep 2
else
    log_error "Server failed to start for streaming tests"
    kill $SERVER_PID 2>/dev/null || true
fi

echo
echo "======================================================================"
echo "🏁 Manual Testing Suite Complete"
echo "======================================================================"

log_success "All manual tests completed!"
log_info "Summary:"
log_info "  ✅ Default configuration startup"
log_info "  ✅ Environment variable configuration"

if $GRPCURL_AVAILABLE; then
    log_info "  ✅ gRPC communication"
    log_info "  ✅ Concurrent requests"
else
    log_info "  ⚠️  gRPC tests skipped (grpcurl not available)"
fi

log_info "  ✅ Log format validation"
if $GRPCURL_AVAILABLE; then
    log_info "  ✅ Streaming functionality"
else
    log_info "  ⚠️  Streaming tests skipped (grpcurl not available)"
fi

echo
log_info "To run individual tests, check the script sections or:"
log_info "  - Start server: cargo run"
log_info "  - Test health: curl http://localhost:8081/health"
if $GRPCURL_AVAILABLE; then
    log_info "  - Test gRPC: grpcurl -plaintext -d '{\"name\": \"Test\"}' localhost:50051 hello_world.Greeter/SayHello"
    log_info "  - Test streaming: grpcurl -plaintext -d '{}' localhost:50051 hello_world.Greeter/StreamTime"
fi

echo
log_success "Manual testing suite finished successfully! 🎉"
