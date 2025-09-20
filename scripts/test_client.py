#!/usr/bin/env python3
"""
Advanced gRPC Test Client for Hello World Service

This script provides programmatic testing capabilities beyond what
shell scripts can easily accomplish, including:
- Concurrent request testing with timing
- Error condition testing
- Connection management testing
- Load testing scenarios
"""

import asyncio
import grpc
import sys
import time
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Tuple, Optional
import json

# Generated protobuf imports would go here in a real implementation
# For this demo, we'll simulate the interface

class HelloRequest:
    """Simulated protobuf message for demo purposes"""
    def __init__(self, name: str):
        self.name = name

class HelloReply:
    """Simulated protobuf message for demo purposes"""
    def __init__(self, message: str):
        self.message = message

class MockGreeterStub:
    """Mock gRPC stub for demonstration - in real implementation would use generated code"""
    
    def __init__(self, channel):
        self.channel = channel
        self._simulate_connection = True
    
    def SayHello(self, request, timeout=None):
        """Mock implementation - in real code would call actual gRPC service"""
        if not self._simulate_connection:
            raise grpc.RpcError("Connection failed")
        
        # Simulate processing time
        time.sleep(0.001)  # 1ms
        
        if not request.name or request.name.strip() == "":
            raise grpc.RpcError("INVALID_ARGUMENT: Person name cannot be empty")
        
        if len(request.name) > 100:
            raise grpc.RpcError("INVALID_ARGUMENT: Person name cannot exceed 100 characters")
        
        return HelloReply(f"Hello, {request.name.strip()}!")

def create_channel(address: str) -> grpc.Channel:
    """Create a gRPC channel to the server"""
    return grpc.insecure_channel(address)

def single_request_test(address: str, name: str) -> Tuple[bool, str, float]:
    """
    Test a single gRPC request
    
    Returns:
        (success, response/error, duration_ms)
    """
    start_time = time.time()
    
    try:
        with create_channel(address) as channel:
            stub = MockGreeterStub(channel)
            request = HelloRequest(name)
            response = stub.SayHello(request, timeout=5.0)
            
            duration_ms = (time.time() - start_time) * 1000
            return True, response.message, duration_ms
            
    except Exception as e:
        duration_ms = (time.time() - start_time) * 1000
        return False, str(e), duration_ms

def concurrent_requests_test(address: str, num_requests: int, request_names: List[str]) -> dict:
    """
    Test concurrent gRPC requests
    
    Args:
        address: Server address
        num_requests: Number of concurrent requests
        request_names: List of names to use in requests
    
    Returns:
        Dictionary with test results
    """
    print(f"🚀 Starting {num_requests} concurrent requests...")
    
    start_time = time.time()
    results = {
        'total_requests': num_requests,
        'successful_requests': 0,
        'failed_requests': 0,
        'total_duration_ms': 0,
        'max_duration_ms': 0,
        'min_duration_ms': float('inf'),
        'errors': []
    }
    
    # Prepare requests
    requests = []
    for i in range(num_requests):
        name = request_names[i % len(request_names)] if request_names else f"User{i}"
        requests.append((address, name))
    
    # Execute concurrent requests
    with ThreadPoolExecutor(max_workers=min(num_requests, 50)) as executor:
        # Submit all requests
        future_to_request = {
            executor.submit(single_request_test, addr, name): (addr, name)
            for addr, name in requests
        }
        
        # Collect results
        for future in as_completed(future_to_request):
            try:
                success, response, duration_ms = future.result(timeout=10.0)
                
                if success:
                    results['successful_requests'] += 1
                    print(f"✅ Request successful: {response} ({duration_ms:.1f}ms)")
                else:
                    results['failed_requests'] += 1
                    results['errors'].append(response)
                    print(f"❌ Request failed: {response} ({duration_ms:.1f}ms)")
                
                # Update duration statistics
                results['max_duration_ms'] = max(results['max_duration_ms'], duration_ms)
                results['min_duration_ms'] = min(results['min_duration_ms'], duration_ms)
                
            except Exception as e:
                results['failed_requests'] += 1
                results['errors'].append(str(e))
                print(f"❌ Request exception: {e}")
    
    results['total_duration_ms'] = (time.time() - start_time) * 1000
    results['avg_duration_ms'] = results['total_duration_ms'] / num_requests if num_requests > 0 else 0
    
    # Fix min_duration if no successful requests
    if results['min_duration_ms'] == float('inf'):
        results['min_duration_ms'] = 0
    
    return results

def error_conditions_test(address: str) -> dict:
    """Test various error conditions"""
    print("🧪 Testing error conditions...")
    
    test_cases = [
        ("", "Empty name"),
        ("   ", "Whitespace-only name"),
        ("a" * 101, "Name too long (101 chars)"),
        ("Valid Name", "Valid name (should succeed)")
    ]
    
    results = {
        'test_cases': len(test_cases),
        'passed': 0,
        'failed': 0,
        'results': []
    }
    
    for test_input, description in test_cases:
        print(f"  Testing: {description}")
        success, response, duration_ms = single_request_test(address, test_input)
        
        # Determine if the result is expected
        should_succeed = test_input.strip() and len(test_input.strip()) <= 100
        test_passed = success == should_succeed
        
        result = {
            'description': description,
            'input': test_input,
            'success': success,
            'response': response,
            'duration_ms': duration_ms,
            'test_passed': test_passed
        }
        
        results['results'].append(result)
        
        if test_passed:
            results['passed'] += 1
            status = "✅ PASS"
        else:
            results['failed'] += 1
            status = "❌ FAIL"
        
        print(f"    {status}: {response} ({duration_ms:.1f}ms)")
    
    return results

def load_test(address: str, duration_seconds: int, requests_per_second: int) -> dict:
    """
    Perform a simple load test
    
    Args:
        address: Server address
        duration_seconds: How long to run the test
        requests_per_second: Target requests per second
    
    Returns:
        Dictionary with load test results
    """
    print(f"🔥 Load testing for {duration_seconds}s at {requests_per_second} req/s...")
    
    start_time = time.time()
    end_time = start_time + duration_seconds
    request_interval = 1.0 / requests_per_second
    
    results = {
        'duration_seconds': duration_seconds,
        'target_rps': requests_per_second,
        'total_requests': 0,
        'successful_requests': 0,
        'failed_requests': 0,
        'actual_rps': 0,
        'avg_response_time_ms': 0,
        'errors': []
    }
    
    request_count = 0
    total_response_time = 0
    
    while time.time() < end_time:
        request_start = time.time()
        
        # Make request
        success, response, duration_ms = single_request_test(address, f"LoadTest{request_count}")
        request_count += 1
        total_response_time += duration_ms
        
        if success:
            results['successful_requests'] += 1
        else:
            results['failed_requests'] += 1
            results['errors'].append(response)
        
        # Calculate sleep time to maintain target RPS
        request_duration = time.time() - request_start
        sleep_time = max(0, request_interval - request_duration)
        
        if sleep_time > 0:
            time.sleep(sleep_time)
    
    # Calculate final statistics
    actual_duration = time.time() - start_time
    results['total_requests'] = request_count
    results['actual_rps'] = request_count / actual_duration if actual_duration > 0 else 0
    results['avg_response_time_ms'] = total_response_time / request_count if request_count > 0 else 0
    
    print(f"📊 Load test completed:")
    print(f"  Total requests: {results['total_requests']}")
    print(f"  Successful: {results['successful_requests']}")
    print(f"  Failed: {results['failed_requests']}")
    print(f"  Actual RPS: {results['actual_rps']:.1f}")
    print(f"  Avg response time: {results['avg_response_time_ms']:.1f}ms")
    
    return results

def main():
    parser = argparse.ArgumentParser(description='gRPC Test Client for Hello World Service')
    parser.add_argument('--address', default='localhost:50051', help='Server address')
    parser.add_argument('--test', default='all', choices=['single', 'concurrent', 'errors', 'load', 'all'],
                       help='Test type to run')
    parser.add_argument('--name', default='TestUser', help='Name for single request test')
    parser.add_argument('--concurrent', type=int, default=10, help='Number of concurrent requests')
    parser.add_argument('--load-duration', type=int, default=10, help='Load test duration in seconds')
    parser.add_argument('--load-rps', type=int, default=10, help='Load test requests per second')
    parser.add_argument('--output-json', help='Output results to JSON file')
    
    args = parser.parse_args()
    
    print("🤖 Hello World gRPC Test Client")
    print(f"📡 Server: {args.address}")
    print("=" * 50)
    
    all_results = {}
    
    try:
        if args.test in ['single', 'all']:
            print("\n🔍 Single Request Test")
            print("-" * 30)
            success, response, duration = single_request_test(args.address, args.name)
            if success:
                print(f"✅ Success: {response} ({duration:.1f}ms)")
            else:
                print(f"❌ Failed: {response} ({duration:.1f}ms)")
            
            all_results['single_request'] = {
                'success': success,
                'response': response,
                'duration_ms': duration
            }
        
        if args.test in ['concurrent', 'all']:
            print("\n⚡ Concurrent Requests Test")
            print("-" * 30)
            names = [f"User{i}" for i in range(args.concurrent)]
            results = concurrent_requests_test(args.address, args.concurrent, names)
            
            print(f"\n📊 Concurrent Test Results:")
            print(f"  Total requests: {results['total_requests']}")
            print(f"  Successful: {results['successful_requests']}")
            print(f"  Failed: {results['failed_requests']}")
            print(f"  Success rate: {(results['successful_requests']/results['total_requests']*100):.1f}%")
            print(f"  Total duration: {results['total_duration_ms']:.1f}ms")
            print(f"  Avg duration: {results['avg_duration_ms']:.1f}ms")
            print(f"  Min duration: {results['min_duration_ms']:.1f}ms")
            print(f"  Max duration: {results['max_duration_ms']:.1f}ms")
            
            all_results['concurrent_requests'] = results
        
        if args.test in ['errors', 'all']:
            print("\n🚨 Error Conditions Test")
            print("-" * 30)
            results = error_conditions_test(args.address)
            
            print(f"\n📊 Error Test Results:")
            print(f"  Test cases: {results['test_cases']}")
            print(f"  Passed: {results['passed']}")
            print(f"  Failed: {results['failed']}")
            print(f"  Pass rate: {(results['passed']/results['test_cases']*100):.1f}%")
            
            all_results['error_conditions'] = results
        
        if args.test in ['load', 'all']:
            print(f"\n🔥 Load Test ({args.load_duration}s @ {args.load_rps} req/s)")
            print("-" * 30)
            results = load_test(args.address, args.load_duration, args.load_rps)
            
            all_results['load_test'] = results
        
        # Output JSON results if requested
        if args.output_json:
            with open(args.output_json, 'w') as f:
                json.dump(all_results, f, indent=2)
            print(f"\n📄 Results saved to: {args.output_json}")
        
        print("\n🎉 All tests completed!")
        
    except KeyboardInterrupt:
        print("\n\n⏹️  Tests interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Unexpected error: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
