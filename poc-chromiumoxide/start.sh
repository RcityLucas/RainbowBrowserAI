#!/bin/bash

# Fail pipeline if any stage fails (e.g., cargo build)
set -o pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     Starting RainbowBrowserAI (Chromiumoxide Edition)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Show usage if --help is passed
if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
    echo -e "${GREEN}Usage:${NC}"
    echo -e "  ./start.sh                    # Start with browser window and open dashboard"
    echo -e "  ./start.sh --headless         # Start in headless mode (no browser window)"
    echo -e "  ./start.sh --no-browser       # Start but don't auto-open dashboard"
    echo -e "  ./start.sh --headless --no-browser  # Headless mode and don't open dashboard"
    echo ""
    echo -e "${GREEN}Features:${NC}"
    echo -e "  • Integrated visualization dashboard"
    echo -e "  • No ChromeDriver needed (uses Chrome DevTools Protocol)"
    echo -e "  • Automatic browser pool management"
    echo -e "  • Real-time operation monitoring"
    exit 0
fi

# Configuration
SERVER_PORT=3001
BUILD_MODE="release"

# Function to check if port is in use
check_port() {
    local port=$1
    # Use netstat to check if port is listening (works with any language output)
    if netstat -an 2>/dev/null | grep -E ":${port}\s" >/dev/null; then
        return 0  # Port is in use
    fi
    return 1  # Port is free
}

# Find free port for server
find_free_port() {
    local base_port=$1
    local port=$base_port
    while check_port $port; do
        echo -e "${YELLOW}⚠ Port $port is in use, trying $(($port + 1))...${NC}" >&2
        port=$(($port + 1))
        if [ $port -gt $(($base_port + 10)) ]; then
            echo -e "${RED}✗ Could not find free port after 10 attempts${NC}" >&2
            return 1
        fi
    done
    echo $port
}

# Clean up old processes
echo -e "${YELLOW}🔄 Cleaning up old processes...${NC}"
# Try different cleanup methods
pkill -f rainbow-poc-chromiumoxide 2>/dev/null && echo -e "${GREEN}  ✓ Killed old processes (pkill)${NC}"
if command -v taskkill >/dev/null 2>&1; then
    taskkill //F //IM rainbow-poc-chromiumoxide.exe 2>/dev/null && echo -e "${GREEN}  ✓ Killed old processes (taskkill)${NC}"
fi
# Wait a bit for processes to fully terminate
sleep 1

# Find free port
echo -e "\n${BLUE}Finding available port for server...${NC}"
NEW_SERVER_PORT=$(find_free_port $SERVER_PORT)
if [ $? -ne 0 ]; then
    echo -e "${RED}✗ Failed to find free port for server${NC}"
    exit 1
fi
SERVER_PORT=$NEW_SERVER_PORT
echo -e "${GREEN}  ✓ Server will use port ${SERVER_PORT}${NC}"

# Build the project
echo -e "\n${BLUE}Building the project...${NC}"
if [ "$BUILD_MODE" = "release" ]; then
    echo -e "${YELLOW}  Building in release mode (optimized)...${NC}"
    # Show only key cargo lines and real Rust errors (avoid matching #[error("...")])
    if cargo build --release --color never 2>&1 | grep -E "^[[:space:]]*(Compiling|Finished|error(\[|:))"; then
        echo -e "${GREEN}  ✓ Build completed${NC}"
    else
        echo -e "${RED}  ✗ Build failed${NC}"
        exit 1
    fi
    # Check if we're on Windows (Git Bash/MSYS) or Linux
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ -f "./target/release/rainbow-poc-chromiumoxide.exe" ]]; then
        BINARY="./target/release/rainbow-poc-chromiumoxide.exe"
    else
        BINARY="./target/release/rainbow-poc-chromiumoxide"
    fi
else
    echo -e "${YELLOW}  Building in debug mode...${NC}"
    # Show only key cargo lines and real Rust errors (avoid matching #[error("...")])
    if cargo build --color never 2>&1 | grep -E "^[[:space:]]*(Compiling|Finished|error(\[|:))"; then
        echo -e "${GREEN}  ✓ Build completed${NC}"
    else
        echo -e "${RED}  ✗ Build failed${NC}"
        exit 1
    fi
    # Check if we're on Windows (Git Bash/MSYS) or Linux
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ -f "./target/debug/rainbow-poc-chromiumoxide.exe" ]]; then
        BINARY="./target/debug/rainbow-poc-chromiumoxide.exe"
    else
        BINARY="./target/debug/rainbow-poc-chromiumoxide"
    fi
fi

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down...${NC}"
    pkill -f rainbow-poc-chromiumoxide 2>/dev/null
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Set up cleanup on exit
trap cleanup EXIT INT TERM

# Start the application
echo -e "\n${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🚀 Starting RainbowBrowserAI Server (Chromiumoxide)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}📍 Dashboard: ${BLUE}http://localhost:$SERVER_PORT${NC}"
echo -e "${GREEN}🎨 Visualization: ${BLUE}http://localhost:$SERVER_PORT${NC}"
echo -e "${GREEN}📊 Health API: ${BLUE}http://localhost:$SERVER_PORT/api/health${NC}"
echo -e "${GREEN}🔧 Tools API: ${BLUE}http://localhost:$SERVER_PORT/api/tools${NC}"
echo -e "${YELLOW}🎯 No ChromeDriver needed! Using Chrome DevTools Protocol${NC}"
echo -e "${YELLOW}📈 Real-time monitoring and visualization included!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Skip browser test for now (optional: can be enabled with --test flag)
if [ "$1" == "--test" ] || [ "$2" == "--test" ] || [ "$3" == "--test" ]; then
    echo -e "${BLUE}Testing browser connection...${NC}"
    if timeout 5 "$BINARY" test --headless 2>&1 | grep -q "All tests passed"; then
        echo -e "${GREEN}  ✓ Browser test passed${NC}"
    else
        echo -e "${YELLOW}  ⚠ Browser test had issues, but continuing...${NC}"
    fi
fi

# Determine mode (headless or headed)
if [ "$1" == "--headless" ] || [ "$2" == "--headless" ] || [ "$3" == "--headless" ]; then
    HEADLESS_ARG="--headless"
    echo -e "${YELLOW}  Running in headless mode (no browser window)${NC}"
    # Start server with headless flag
    echo -e "\n${BLUE}Starting API server...${NC}"
    "$BINARY" serve --port "$SERVER_PORT" --headless &
else
    echo -e "${GREEN}  Running in headed mode (browser window visible)${NC}"
    # Start server without headless flag
    echo -e "\n${BLUE}Starting API server...${NC}"
    "$BINARY" serve --port "$SERVER_PORT" &
fi
SERVER_PID=$!

# Wait for server to start
echo -e "${BLUE}⏳ Waiting for server to start...${NC}"
for i in {1..15}; do
    if curl -s http://localhost:$SERVER_PORT/api/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Service started successfully!${NC}"
        echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}🌐 Dashboard: ${BLUE}http://localhost:$SERVER_PORT${NC}"
        echo -e "${GREEN}📊 Health API: ${BLUE}http://localhost:$SERVER_PORT/api/health${NC}"
        echo -e "${GREEN}🎨 Visualization: ${BLUE}http://localhost:$SERVER_PORT${NC}"
        echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
        
        # Open browser to dashboard if not in headless mode and if --no-browser flag is not set
        HEADLESS_MODE=false
        if [[ "$1" == "--headless" ]] || [[ "$2" == "--headless" ]] || [[ "$3" == "--headless" ]]; then
            HEADLESS_MODE=true
        fi
        
        if [[ "$HEADLESS_MODE" != true ]] && [[ "$1" != "--no-browser" ]] && [[ "$2" != "--no-browser" ]]; then
            echo -e "${YELLOW}🚀 Opening dashboard in browser...${NC}"
            # Try different methods to open browser based on OS
            if command -v xdg-open > /dev/null 2>&1; then
                xdg-open "http://localhost:$SERVER_PORT" 2>/dev/null &
            elif command -v open > /dev/null 2>&1; then
                open "http://localhost:$SERVER_PORT" 2>/dev/null &
            elif command -v start > /dev/null 2>&1; then
                start "http://localhost:$SERVER_PORT" 2>/dev/null &
            else
                echo -e "${YELLOW}  ℹ️  Could not auto-open browser. Please manually visit: ${BLUE}http://localhost:$SERVER_PORT${NC}"
            fi
        fi
        
        echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
        echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
        break
    fi
    if [ $i -eq 15 ]; then
        echo -e "${RED}❌ Service failed to start${NC}"
        exit 1
    fi
    sleep 2
done

# Wait for the main process
wait $SERVER_PID
