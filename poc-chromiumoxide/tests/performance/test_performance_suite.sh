#!/bin/bash

# Performance Test Suite
# Tests system performance, load handling, and resource efficiency

set -e

# Configuration
SERVER_URL="http://localhost:3002"
PASSED=0
FAILED=0

# Test utilities
test_passed() {
    echo "✅ $1"
    ((PASSED++))
}

test_failed() {
    echo "❌ $1"
    ((FAILED++))
}

measure_time() {
    local start_time=$(date +%s.%N)
    "$@"
    local end_time=$(date +%s.%N)
    echo $(echo "$end_time - $start_time" | bc -l)
}

echo "========================================="
echo "       Performance Test Suite"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Server Response Time Tests
echo "--- Server Response Time Tests ---"

# Health endpoint performance
total_time=0
iterations=10
for i in $(seq 1 $iterations); do
    start_time=$(date +%s.%N)
    curl -s "$SERVER_URL/api/health" > /dev/null
    end_time=$(date +%s.%N)
    iteration_time=$(echo "$end_time - $start_time" | bc -l)
    total_time=$(echo "$total_time + $iteration_time" | bc -l)
done

avg_health_time=$(echo "scale=3; $total_time / $iterations" | bc -l)
if (( $(echo "$avg_health_time < 0.1" | bc -l) )); then
    test_passed "Health endpoint response time (${avg_health_time}s avg)"
else
    test_failed "Health endpoint too slow (${avg_health_time}s avg)"
fi

# Tools API response time
start_time=$(date +%s.%N)
curl -s "$SERVER_URL/api/tools" > /dev/null
end_time=$(date +%s.%N)
tools_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$tools_time < 0.5" | bc -l) )); then
    test_passed "Tools API response time (${tools_time}s)"
else
    test_failed "Tools API too slow (${tools_time}s)"
fi

# Navigation Performance Tests
echo ""
echo "--- Navigation Performance Tests ---"

# Measure navigation to different sites
sites=("https://example.com" "https://httpbin.org" "https://github.com")
total_nav_time=0

for site in "${sites[@]}"; do
    start_time=$(date +%s.%N)
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"navigate_to_url\",\"parameters\":{\"url\":\"$site\"}}" \
        "$SERVER_URL/api/tools/execute" > /dev/null
    end_time=$(date +%s.%N)
    
    site_time=$(echo "$end_time - $start_time" | bc -l)
    total_nav_time=$(echo "$total_nav_time + $site_time" | bc -l)
    
    if (( $(echo "$site_time < 8.0" | bc -l) )); then
        test_passed "Navigation to $site (${site_time}s)"
    else
        test_failed "Navigation to $site too slow (${site_time}s)"
    fi
done

avg_nav_time=$(echo "scale=2; $total_nav_time / ${#sites[@]}" | bc -l)
if (( $(echo "$avg_nav_time < 6.0" | bc -l) )); then
    test_passed "Average navigation performance (${avg_nav_time}s)"
else
    test_failed "Average navigation performance poor (${avg_nav_time}s)"
fi

# Tool Execution Performance Tests
echo ""
echo "--- Tool Execution Performance Tests ---"

# Setup test page
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Measure various tool execution times
tools_performance=(
    "extract_text:{\"selector\":\"h1\"}"
    "extract_links:{\"selector\":\"a\"}"
    "get_element_info:{\"selector\":\"body\"}"
    "click:{\"selector\":\"a\"}"
    "hover:{\"selector\":\"h1\"}"
)

for tool_test in "${tools_performance[@]}"; do
    IFS=':' read -r tool_name params <<< "$tool_test"
    
    start_time=$(date +%s.%N)
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"$tool_name\",\"parameters\":$params}" \
        "$SERVER_URL/api/tools/execute" > /dev/null
    end_time=$(date +%s.%N)
    
    tool_time=$(echo "$end_time - $start_time" | bc -l)
    if (( $(echo "$tool_time < 3.0" | bc -l) )); then
        test_passed "$tool_name execution time (${tool_time}s)"
    else
        test_failed "$tool_name execution too slow (${tool_time}s)"
    fi
done

# Load Testing
echo ""
echo "--- Load Testing ---"

# Concurrent requests test
echo "Testing with 10 concurrent requests..."
start_time=$(date +%s.%N)

for i in {1..10}; do
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"extract_text","parameters":{"selector":"body"}}' \
        "$SERVER_URL/api/tools/execute" > /dev/null &
done

wait
end_time=$(date +%s.%N)
concurrent_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$concurrent_time < 15.0" | bc -l) )); then
    test_passed "10 concurrent requests handled (${concurrent_time}s)"
else
    test_failed "Concurrent request handling too slow (${concurrent_time}s)"
fi

# Rapid sequential requests
echo ""
echo "--- Rapid Sequential Requests ---"
start_time=$(date +%s.%N)

for i in {1..20}; do
    curl -s "$SERVER_URL/api/health" > /dev/null
done

end_time=$(date +%s.%N)
sequential_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$sequential_time < 5.0" | bc -l) )); then
    test_passed "20 sequential health checks (${sequential_time}s)"
else
    test_failed "Sequential requests too slow (${sequential_time}s)"
fi

# Perception Performance Tests
echo ""
echo "--- Perception Performance Tests ---"

# Page analysis performance
start_time=$(date +%s.%N)
curl -s -X POST -H "Content-Type: application/json" \
    -d '{}' \
    "$SERVER_URL/api/perception/analyze" > /dev/null
end_time=$(date +%s.%N)
analysis_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$analysis_time < 5.0" | bc -l) )); then
    test_passed "Page analysis performance (${analysis_time}s)"
else
    test_failed "Page analysis too slow (${analysis_time}s)"
fi

# Element finding performance
start_time=$(date +%s.%N)
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"description":"link"}' \
    "$SERVER_URL/api/perception/find" > /dev/null
end_time=$(date +%s.%N)
find_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$find_time < 3.0" | bc -l) )); then
    test_passed "Element finding performance (${find_time}s)"
else
    test_failed "Element finding too slow (${find_time}s)"
fi

# Memory Usage Tests
echo ""
echo "--- Memory Usage Tests ---"

# Get initial memory usage if available
if command -v ps &> /dev/null; then
    # Look for the server process
    server_pid=$(pgrep -f "serve.*--port.*3002" | head -1)
    if [ -n "$server_pid" ]; then
        memory_kb=$(ps -o rss= -p "$server_pid" | tr -d ' ')
        memory_mb=$(echo "scale=2; $memory_kb / 1024" | bc -l)
        
        if (( $(echo "$memory_mb < 500" | bc -l) )); then
            test_passed "Memory usage reasonable (${memory_mb}MB)"
        else
            test_failed "Memory usage high (${memory_mb}MB)"
        fi
    else
        test_failed "Cannot find server process for memory check"
    fi
else
    echo "⚠️ Memory usage test skipped (ps command not available)"
fi

# Stress Test
echo ""
echo "--- Stress Test ---"

# Mixed load test
echo "Running mixed load test (navigation + extraction + perception)..."
start_time=$(date +%s.%N)

# Launch background processes
for i in {1..3}; do
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
        "$SERVER_URL/api/tools/execute" > /dev/null &
    
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"extract_text","parameters":{"selector":"body"}}' \
        "$SERVER_URL/api/tools/execute" > /dev/null &
    
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"description":"heading"}' \
        "$SERVER_URL/api/perception/find" > /dev/null &
done

wait
end_time=$(date +%s.%N)
stress_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$stress_time < 20.0" | bc -l) )); then
    test_passed "Mixed load stress test (${stress_time}s)"
else
    test_failed "Stress test performance poor (${stress_time}s)"
fi

# Resource Cleanup Test
echo ""
echo "--- Resource Cleanup Test ---"

# Check if server is still responsive after stress test
start_time=$(date +%s.%N)
response=$(curl -s "$SERVER_URL/api/health")
end_time=$(date +%s.%N)
cleanup_time=$(echo "$end_time - $start_time" | bc -l)

if echo "$response" | jq -e '.status' > /dev/null 2>&1; then
    if (( $(echo "$cleanup_time < 1.0" | bc -l) )); then
        test_passed "Post-stress server responsiveness (${cleanup_time}s)"
    else
        test_failed "Server slow after stress test (${cleanup_time}s)"
    fi
else
    test_failed "Server unresponsive after stress test"
fi

# Throughput Test
echo ""
echo "--- Throughput Test ---"

requests_per_batch=5
batches=4
total_requests=$((requests_per_batch * batches))

start_time=$(date +%s.%N)
for batch in $(seq 1 $batches); do
    for i in $(seq 1 $requests_per_batch); do
        curl -s "$SERVER_URL/api/health" > /dev/null &
    done
    wait
done
end_time=$(date +%s.%N)

total_throughput_time=$(echo "$end_time - $start_time" | bc -l)
requests_per_second=$(echo "scale=2; $total_requests / $total_throughput_time" | bc -l)

if (( $(echo "$requests_per_second > 5.0" | bc -l) )); then
    test_passed "Throughput performance (${requests_per_second} req/s)"
else
    test_failed "Throughput too low (${requests_per_second} req/s)"
fi

# Summary
echo ""
echo "========================================="
echo "      PERFORMANCE TEST SUMMARY"
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"
echo ""
echo "Performance Metrics:"
echo "- Average Health Response: ${avg_health_time}s"
echo "- Average Navigation Time: ${avg_nav_time}s"
echo "- Concurrent Load Time: ${concurrent_time}s"
echo "- Throughput: ${requests_per_second} req/s"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All performance tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED performance test(s) failed"
    exit 1
fi