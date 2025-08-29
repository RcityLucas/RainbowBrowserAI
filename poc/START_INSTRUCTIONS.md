# RainbowBrowserAI V8.0 Startup Instructions

## Quick Start

### Windows Users
1. Double-click `start.bat` or run it from Command Prompt:
   ```cmd
   start.bat
   ```

### Linux/macOS Users
1. Make the script executable (if not already):
   ```bash
   chmod +x start.sh
   ```

2. Run the startup script:
   ```bash
   ./start.sh
   ```

## What the Startup Script Does

### 🔧 Automatic Setup
- **Port Management**: Automatically finds available ports for ChromeDriver (default: 9515) and server (default: 3001)
- **ChromeDriver Detection**: Searches common locations and provides installation guidance if needed
- **Project Building**: Attempts release build, falls back to debug build, then cargo run if needed
- **Service Health Checks**: Waits for services to be fully ready before proceeding

### 🌐 Service Launch
- **ChromeDriver**: Starts browser automation engine
- **V8.0 Server**: Launches the main RainbowBrowserAI service with:
  - Mock LLM mode (no API keys required)
  - V8.0 perception system enabled
  - Local intent recognition enabled
  - Error recovery system enabled
  - Production monitoring enabled

### 📊 Web Dashboard
- **Auto-opens**: Browser automatically opens to the dashboard
- **URL**: `http://localhost:[PORT]` (default: http://localhost:3001)
- **Features**:
  - Real-time system monitoring
  - Task execution interface
  - Workflow templates (Shopping, Travel, Research, Custom)
  - Performance metrics
  - Health indicators

## Available Workflows

### 🛒 Shopping Workflow
- Amazon product search
- Price comparison
- Automated navigation

### ✈️ Travel Workflow
- Flight search and booking
- Hotel comparison
- Travel planning assistance

### 📚 Research Workflow
- Academic paper search
- Citation management
- Knowledge extraction

### ⚡ Custom Commands
- Natural language instructions
- Flexible task execution
- AI-powered interpretation

## Prerequisites

### Required
- **Rust**: Latest stable version
- **Chrome/Chromium**: For browser automation
- **ChromeDriver**: Matching your Chrome version

### ChromeDriver Installation

#### Windows
- Download from: https://chromedriver.chromium.org/
- Or use Chocolatey: `choco install chromedriver`
- Or place `chromedriver.exe` in the project directory

#### macOS
```bash
brew install chromedriver
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get install chromium-driver
```

## Troubleshooting

### Port Already in Use
The script automatically handles port conflicts by:
1. Detecting occupied ports
2. Killing existing processes
3. Finding alternative ports if needed

### ChromeDriver Not Found
If ChromeDriver isn't detected:
1. Install using the methods above
2. Ensure it's in your system PATH
3. Or place the executable in the project directory

### Build Failures
The script handles build issues by:
1. Trying release build first (optimized)
2. Falling back to debug build
3. Using `cargo run` as last resort

### Service Won't Start
- Check that ports 9515 and 3001 aren't blocked by firewall
- Ensure Chrome/Chromium is installed
- Try running with administrator/sudo privileges

## Configuration

### Environment Variables
The script sets these automatically:
- `RUST_LOG=info,rainbow_poc=debug`
- `RAINBOW_MOCK_MODE=true`
- `RAINBOW_V8_MODE=production`
- `RAINBOW_INTEGRATION_MODE=V8WithFallback`
- `RAINBOW_PERCEPTION_ENABLED=true`
- `RAINBOW_LOCAL_LLM_ENABLED=true`
- `RAINBOW_ERROR_RECOVERY_ENABLED=true`

### Custom Ports
To use different ports, edit the script variables:
- `CHROMEDRIVER_PORT=9515`
- `SERVER_PORT=3001`

## API Endpoints

Once running, these endpoints are available:

- **Dashboard**: `http://localhost:3001`
- **Health Check**: `http://localhost:3001/health`
- **System Status**: `http://localhost:3001/api/system/status`
- **Metrics**: `http://localhost:3001/api/metrics`
- **Workflows**: `http://localhost:3001/api/workflows`

## Stopping the Service

### Windows
- Press any key in the console window
- Or close the console window
- Services are automatically cleaned up

### Linux/macOS
- Press `Ctrl+C` in the terminal
- Services are automatically cleaned up

## Production Deployment

For production deployment with full monitoring stack:
```bash
cd production-config
docker-compose up -d
```

This launches:
- RainbowBrowserAI V8.0 service
- Chrome browser cluster
- PostgreSQL database
- Redis cache
- Prometheus monitoring
- Grafana dashboards
- Jaeger tracing
- Nginx load balancer

## Support

If you encounter issues:
1. Check the console output for error messages
2. Verify all prerequisites are installed
3. Try running with elevated privileges
4. Check firewall settings for port access