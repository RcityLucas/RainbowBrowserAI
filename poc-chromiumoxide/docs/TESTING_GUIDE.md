# RainbowBrowserAI Testing Guide

This document provides comprehensive information about the testing framework implemented for the RainbowBrowserAI Chromiumoxide Edition.

## Quick Start

### Run All Tests
```bash
scripts/run_comprehensive_tests.sh
```

### Run Quick Test Suite (Recommended for Development)
```bash
scripts/run_comprehensive_tests.sh quick
```

### Run Specific Test Category
```bash
scripts/run_comprehensive_tests.sh perception
scripts/run_comprehensive_tests.sh performance
scripts/run_comprehensive_tests.sh e2e
```

## Test Structure

The testing framework is organized into the following categories:

### 📁 tests/
```
tests/
├── run_all_tests.sh              # Master test runner
├── test_config.yaml              # Test configuration
├── reports/                      # Test results and logs
├── api/
│   └── test_all_endpoints.sh     # API endpoint tests
├── ui/
│   └── test_interface.sh         # User interface tests
├── browser/
│   └── test_browser_suite.sh     # Browser automation tests
├── tools/
│   └── test_all_tools.sh         # Tool functionality tests
├── perception/
│   └── test_perception_suite.sh  # AI perception tests
├── performance/
│   └── test_performance_suite.sh # Performance & load tests
├── e2e/
│   └── test_e2e_suite.sh         # End-to-end workflow tests
└── integration/
    ├── test_api_integration.sh   # API integration tests
    └── test_browser_integration.sh # Browser integration tests
```

## Test Categories

### 1. API Tests (`api/`)
Tests all REST API endpoints including:
- Health check endpoints
- Tools API functionality
- Perception API endpoints
- Error handling and validation
- Response format consistency
- Concurrent request handling

**Example:**
```bash
scripts/run_comprehensive_tests.sh api
```

### 2. User Interface Tests (`ui/`)
Tests the web interface including:
- HTML structure and navigation
- JavaScript functionality
- CSS styling and responsiveness
- Form interactions
- Tool execution from UI
- Error display and user feedback

**Example:**
```bash
scripts/run_comprehensive_tests.sh ui
```

### 3. Browser Automation Tests (`browser/`)
Tests core browser functionality:
- Navigation between websites
- Element interaction (click, type, hover)
- Data extraction and manipulation
- Session management
- Wait conditions and timeouts
- Error recovery

**Example:**
```bash
scripts/run_comprehensive_tests.sh browser
```

### 4. Tools Tests (`tools/`)
Comprehensive testing of all available tools:
- Navigation tools (navigate, back, forward, refresh)
- Interaction tools (click, type, hover, focus, select)
- Extraction tools (text, links, data, element info)
- Utility tools (wait, session storage, cache)
- Tool chaining and workflows
- Error handling for each tool

**Example:**
```bash
scripts/run_comprehensive_tests.sh tools
```

### 5. Perception Tests (`perception/`)
Tests AI-powered perception capabilities:
- Page analysis and classification
- Natural language element finding
- Intelligent command execution
- Form analysis and auto-fill
- Context awareness
- Confidence scoring

**Example:**
```bash
scripts/run_comprehensive_tests.sh perception
```

### 6. Performance Tests (`performance/`)
Tests system performance and scalability:
- Response time measurements
- Load testing with concurrent requests
- Memory usage monitoring
- Throughput testing
- Resource cleanup verification
- Stress testing scenarios

**Example:**
```bash
scripts/run_comprehensive_tests.sh performance
```

### 7. End-to-End Tests (`e2e/`)
Tests complete user workflows:
- Form filling workflows
- Multi-site navigation scenarios
- Perception-powered interactions
- Session state management
- Error recovery scenarios
- Complex multi-step processes

**Example:**
```bash
scripts/run_comprehensive_tests.sh e2e
```

### 8. Integration Tests (`integration/`)
Tests system integration between components:
- API-to-browser integration
- Perception-to-tools integration
- Cross-component data flow
- Error propagation between layers
- State consistency across components

**Example:**
```bash
scripts/run_comprehensive_tests.sh integration
```

## Test Configuration

### Environment Variables
Set these environment variables to customize test behavior:
```bash
export RAINBOW_MOCK_MODE=true    # Use mock LLM for faster testing
export SERVER_PORT=3002          # Test server port
export HEADLESS=true            # Run browser in headless mode
```

### Test Configuration File
The `tests/test_config.yaml` file contains:
- Server configuration
- Browser settings
- Test site URLs
- Timeout values
- Tool configurations
- Report settings

## Running Tests

### Prerequisites
Ensure you have the following tools installed:
- `curl` - for HTTP requests
- `jq` - for JSON processing
- `bc` - for mathematical calculations
- Rust and Cargo - for building the project

### Starting the Test Server
The test framework automatically starts and stops the test server. The server runs on port 3002 by default.

### Manual Test Server Start
```bash
RAINBOW_MOCK_MODE=true cargo run --release -- serve --port 3002 --headless
```

## Test Reports

### Report Location
All test results are stored in `tests/reports/` with timestamps:
- `comprehensive_test_YYYYMMDD_HHMMSS.log` - Main test log
- `{category}_test.log` - Individual test suite logs
- `server.log` - Server output during testing

### Report Format
Each test report includes:
- Test execution summary
- Pass/fail statistics
- Performance metrics
- Detailed error messages
- Timestamps and duration

### Example Report Output
```
=========================================
         TEST RESULTS SUMMARY
=========================================
Total Test Suites: 8
Passed: 7
Failed: 1
Success Rate: 87.50%

Detailed Results:
  ✅ API Endpoint Tests - PASSED (45s)
  ✅ User Interface Tests - PASSED (32s)
  ❌ Performance Tests - FAILED (120s)
  ✅ End-to-End Tests - PASSED (95s)
```

## Continuous Integration

### GitHub Actions Integration
Add this workflow to `.github/workflows/test.yml`:
```yaml
name: Comprehensive Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install dependencies
        run: sudo apt-get install -y curl jq bc
      - name: Run tests
        run: scripts/run_comprehensive_tests.sh quick
```

### Local Development Workflow
1. **During development**: Use `scripts/run_comprehensive_tests.sh quick`
2. **Before commits**: Run specific test categories related to your changes
3. **Before releases**: Run full test suite with `scripts/run_comprehensive_tests.sh`

## Test Development

### Adding New Tests
1. Choose the appropriate test category directory
2. Add test cases to the existing `.sh` file
3. Follow the established pattern:
   ```bash
   test_passed() {
       echo "✅ $1"
       ((PASSED++))
   }
   
   test_failed() {
       echo "❌ $1"
       ((FAILED++))
   }
   ```

### Creating New Test Categories
1. Create a new directory under `tests/`
2. Create a test script following the naming convention
3. Make the script executable: `chmod +x script_name.sh`
4. Add the new category to `scripts/run_comprehensive_tests.sh`

### Test Best Practices
- **Use descriptive test names**
- **Include both positive and negative test cases**
- **Test error conditions and recovery**
- **Verify state changes and cleanup**
- **Include performance considerations**
- **Document complex test scenarios**

## Troubleshooting

### Common Issues

#### Server Won't Start
```bash
# Check if port is in use
netstat -tulpn | grep :3002

# Kill existing processes
pkill -f "serve.*--port.*3002"
```

#### Tests Timeout
- Increase timeout values in `test_config.yaml`
- Check system resources and network connectivity
- Use mock mode: `export RAINBOW_MOCK_MODE=true`

#### Permission Errors
```bash
# Make all test scripts executable
find tests -name "*.sh" -exec chmod +x {} \;
```

#### Missing Dependencies
```bash
# Install required tools
sudo apt-get install curl jq bc  # Ubuntu/Debian
brew install curl jq bc          # macOS
```

### Debug Mode
Enable verbose logging by setting:
```bash
export RUST_LOG=debug
export VERBOSE_TESTS=true
```

### Manual Testing
For debugging specific test cases:
```bash
# Start server manually
RAINBOW_MOCK_MODE=true cargo run --release -- serve --port 3002 --headless

# Run individual test script
./tests/api/test_all_endpoints.sh

# Check specific endpoint
curl -s http://localhost:3002/api/health | jq
```

## Performance Benchmarks

### Expected Performance Metrics
- Health endpoint response: < 100ms
- Tool execution: < 3s
- Navigation: < 8s
- Perception analysis: < 5s
- Concurrent requests (10): < 15s

### Performance Monitoring
The performance test suite automatically monitors:
- Response times for all operations
- Memory usage during test execution
- Concurrent request handling
- Resource cleanup efficiency
- System recovery after load

## Best Practices

### For Developers
- **Run quick tests frequently** during development
- **Run relevant test categories** before committing changes
- **Add tests** for new features and bug fixes
- **Monitor performance impacts** of changes

### For CI/CD
- **Use quick test suite** for rapid feedback
- **Run full test suite** for release candidates
- **Archive test reports** for historical analysis
- **Set up alerts** for test failures

### For Production Deployment
- **All tests must pass** before deployment
- **Performance benchmarks** must be met
- **Integration tests** must verify component compatibility
- **End-to-end tests** must validate user workflows

## Contributing

When contributing to the test framework:
1. **Follow existing patterns** and naming conventions
2. **Document new test categories** and their purpose
3. **Ensure tests are deterministic** and can run independently
4. **Include both success and failure scenarios**
5. **Update this documentation** for significant changes

## Support

For test framework issues:
1. Check the test logs in `tests/reports/`
2. Verify all prerequisites are installed
3. Try running tests individually to isolate issues
4. Check system resources and network connectivity
5. Create an issue with detailed error logs and system information

---

*This testing framework provides comprehensive coverage of the RainbowBrowserAI system, ensuring reliability, performance, and user experience across all components.*
