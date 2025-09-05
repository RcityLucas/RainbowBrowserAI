# RainbowBrowserAI Testing Suite

[![CI Status](https://github.com/RainbowBrowserAI/testing-suite/workflows/Comprehensive%20Testing%20Suite/badge.svg)](https://github.com/RainbowBrowserAI/testing-suite/actions)
[![Coverage Status](https://coveralls.io/repos/github/RainbowBrowserAI/testing-suite/badge.svg?branch=main)](https://coveralls.io/github/RainbowBrowserAI/testing-suite?branch=main)
[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=RainbowBrowserAI_testing-suite&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=RainbowBrowserAI_testing-suite)

A comprehensive, industry-standard testing suite for the RainbowBrowserAI project, following modern testing best practices and frameworks.

## 🎯 Overview

This testing suite provides complete coverage of the RainbowBrowserAI system using popular industry frameworks:

- **Jest** for unit and integration testing
- **Playwright** for end-to-end browser testing  
- **Artillery** for performance and load testing
- **Docker** for containerized testing environments
- **GitHub Actions** for CI/CD automation

## 📁 Project Structure

```
testing-suite/
├── src/
│   ├── __tests__/
│   │   ├── api/                    # API unit tests
│   │   ├── e2e/                    # End-to-end tests
│   │   ├── integration/            # Integration tests
│   │   └── performance/            # Performance tests
│   ├── setup/                      # Test configuration
│   └── utils/                      # Test utilities
├── .github/workflows/              # CI/CD pipelines
├── reports/                        # Test reports and coverage
├── docker-compose.yml              # Docker environment
├── Dockerfile                      # Multi-stage container
└── README.md                       # This file
```

## 🚀 Quick Start

### Prerequisites

- Node.js 18+ 
- Docker (optional, for containerized testing)
- Rust (for building the main application)

### Installation

```bash
# Clone the testing suite
git clone <repository-url>
cd testing-suite

# Install dependencies
npm install

# Install Playwright browsers
npx playwright install

# Build the main application (required for tests)
cd ../poc-chromiumoxide
cargo build --release
cd ../testing-suite
```

### Running Tests

```bash
# Run all tests
npm test

# Run specific test types
npm run test:api           # API tests only
npm run test:e2e           # End-to-end tests
npm run test:performance   # Performance tests
npm run test:coverage      # With coverage report

# Watch mode for development
npm run test:watch
```

### Docker Usage

```bash
# Run tests in Docker
docker-compose up rainbowbrowserai-tests

# Run performance tests
docker-compose up performance-tests

# Debug environment
docker-compose up debug
```

## 🧪 Testing Framework Features

### ✅ **Modern Jest Setup**
- TypeScript support with ts-jest
- Custom matchers for API responses  
- Code coverage with multiple reporters
- Parallel test execution
- Snapshot testing support

### 🎭 **Playwright E2E Testing**
- Cross-browser testing (Chrome, Firefox, Safari)
- Mobile device simulation
- Visual regression testing
- Automatic screenshot/video capture
- Trace collection for debugging

### ⚡ **Performance Testing**
- Load testing with Artillery
- Response time monitoring
- Concurrent request handling
- Memory usage tracking  
- Throughput benchmarking

### 🐳 **Docker Integration**
- Multi-stage Dockerfiles
- Development and production environments
- Selenium Grid for cross-browser testing
- Database and Redis for integration tests
- Monitoring with Prometheus

### 🔄 **CI/CD Pipeline**
- GitHub Actions workflows
- Security scanning with Trivy
- Code quality checks
- Parallel test execution
- Deployment readiness verification

## 📊 Test Categories

### API Tests (`/api/`)
Tests the REST API endpoints:

```bash
npm run test:api
```

**Coverage:**
- Health check endpoints
- Tool execution APIs  
- Perception engine APIs
- Error handling and validation
- Response format consistency

### End-to-End Tests (`/e2e/`)
Tests complete user workflows:

```bash
npm run test:e2e
```

**Coverage:**
- User interface interactions
- Form filling workflows
- Multi-site navigation
- Perception-powered actions
- Error recovery scenarios

### Integration Tests (`/integration/`)
Tests component integration:

```bash
npm run test:integration
```

**Coverage:**
- API-to-browser integration
- Perception-to-tools integration  
- Cross-component data flow
- State consistency verification

### Performance Tests (`/performance/`)
Tests system performance:

```bash
npm run test:performance
```

**Coverage:**
- Response time benchmarks
- Load testing scenarios
- Concurrent request handling
- Memory usage monitoring
- Throughput measurement

## 🔧 Configuration

### Environment Variables

```bash
# Test configuration
TEST_SERVER_URL=http://localhost:3002
TEST_TIMEOUT=30000
HEADLESS=true
RAINBOW_MOCK_MODE=true

# CI configuration  
CI=true
COVERAGE_THRESHOLD=80
```

### Jest Configuration
Located in `jest.config.js`:

- Coverage thresholds: 80% across all metrics
- Custom matchers for API testing
- HTML and XML reporting
- TypeScript support

### Playwright Configuration
Located in `playwright.config.ts`:

- Multi-browser support
- Mobile device testing
- Screenshot/video capture
- Trace collection
- HTML reporting

## 📈 Coverage Reporting

The testing suite includes comprehensive coverage reporting:

```bash
# Generate coverage report
npm run test:coverage

# View HTML report
open coverage/lcov-report/index.html
```

**Coverage Targets:**
- **Lines:** 80%+
- **Functions:** 80%+  
- **Branches:** 80%+
- **Statements:** 80%+

## 🔍 Quality Assurance

### Code Quality
- **ESLint** for code linting
- **Prettier** for code formatting
- **TypeScript** for type safety
- **Husky** for pre-commit hooks

### Security
- **Trivy** for vulnerability scanning
- **Dependency auditing** 
- **Secret detection**
- **Container security**

### Performance
- Response time monitoring
- Memory usage tracking
- Load testing automation
- Performance regression detection

## 🐛 Debugging

### Debug Mode
```bash
# Run tests with debug output
DEBUG=* npm test

# Run specific test with debugging
npm test -- --testNamePattern="API Tests" --verbose

# Docker debug environment
docker-compose up debug
```

### Troubleshooting

**Common Issues:**

1. **Server won't start**
   ```bash
   # Check port usage
   lsof -i :3002
   
   # Kill existing processes
   pkill -f "serve.*--port.*3002"
   ```

2. **Tests timeout**
   ```bash
   # Increase timeout
   export TEST_TIMEOUT=60000
   
   # Use mock mode
   export RAINBOW_MOCK_MODE=true
   ```

3. **Browser issues**
   ```bash
   # Reinstall browsers
   npx playwright install
   
   # Use headless mode
   export HEADLESS=true
   ```

## 📊 Monitoring and Metrics

### Test Metrics Dashboard
Access test metrics at: `http://localhost:8080` (when using Docker)

**Available Reports:**
- Test execution trends
- Coverage evolution  
- Performance benchmarks
- Error rate monitoring
- Browser compatibility matrix

### CI/CD Integration
- Automated testing on PR/push
- Coverage reports in PRs
- Performance regression detection
- Security vulnerability alerts
- Deployment readiness checks

## 🤝 Contributing

### Adding New Tests

1. **API Tests:** Add to `src/__tests__/api/`
2. **E2E Tests:** Add to `src/__tests__/e2e/`
3. **Performance Tests:** Add to `src/__tests__/performance/`

### Test Conventions

```typescript
// Test structure
describe('Feature Name', () => {
  beforeEach(async () => {
    // Setup
  });

  it('should do something specific', async () => {
    // Test implementation
    expect(result).toBeValidApiResponse();
  });

  afterEach(async () => {
    // Cleanup
  });
});
```

### Custom Matchers

```typescript
// API response validation
expect(response).toBeValidApiResponse();

// CSS selector validation  
expect(selector).toHaveValidSelector();
```

## 📋 Scripts Reference

```bash
# Testing
npm test                    # Run all tests
npm run test:watch         # Watch mode
npm run test:coverage      # With coverage
npm run test:api           # API tests only
npm run test:e2e           # E2E tests only
npm run test:performance   # Performance tests

# Quality
npm run lint               # Lint code
npm run format             # Format code  
npm run typecheck         # TypeScript check

# Docker
npm run docker:build       # Build images
npm run docker:test        # Run in container
npm run docker:debug      # Debug container

# Reports
npm run reports:generate   # Generate reports
npm run reports:serve     # Serve reports
```

## 🔗 Links

- [GitHub Repository](https://github.com/RainbowBrowserAI/testing-suite)
- [CI/CD Dashboard](https://github.com/RainbowBrowserAI/testing-suite/actions)
- [Coverage Reports](https://coveralls.io/github/RainbowBrowserAI/testing-suite)
- [Test Results](http://localhost:8080) (Docker)

## 📄 License

This testing suite is part of the RainbowBrowserAI project and follows the same licensing terms.

---

## 🎖️ Quality Standards

This testing suite follows industry best practices:

- ✅ **Comprehensive Coverage** (80%+ code coverage)
- ✅ **Modern Frameworks** (Jest, Playwright, Artillery)  
- ✅ **CI/CD Integration** (GitHub Actions)
- ✅ **Containerization** (Docker, Docker Compose)
- ✅ **Security Scanning** (Trivy, vulnerability checks)
- ✅ **Performance Testing** (Load tests, benchmarks)
- ✅ **Cross-browser Testing** (Chrome, Firefox, Safari)
- ✅ **Mobile Testing** (Device simulation)
- ✅ **Documentation** (Comprehensive guides)

**Built with ❤️ for robust, reliable testing.**