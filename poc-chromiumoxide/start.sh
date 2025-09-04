#!/bin/bash

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

# Configuration
SERVER_PORT=3001
BUILD_MODE="release"

# Function to check if port is in use
check_port() {
    local port=$1
    if nc -z localhost $port 2>/dev/null; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Find free port for server
find_free_port() {
    local base_port=$1
    local port=$base_port
    while check_port $port; do
        echo -e "${YELLOW}⚠ Port $port is in use, trying $(($port + 1))...${NC}"
        port=$(($port + 1))
        if [ $port -gt $(($base_port + 10)) ]; then
            echo -e "${RED}✗ Could not find free port after 10 attempts${NC}"
            return 1
        fi
    done
    echo $port
}

# Clean up old processes
echo -e "${YELLOW}🔄 Cleaning up old processes...${NC}"
pkill -f rainbow-poc-chromiumoxide 2>/dev/null && echo -e "${GREEN}  ✓ Killed old processes${NC}"

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
    if cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)"; then
        echo -e "${GREEN}  ✓ Build completed${NC}"
    else
        echo -e "${RED}  ✗ Build failed${NC}"
        exit 1
    fi
    BINARY="./target/release/rainbow-poc-chromiumoxide"
else
    echo -e "${YELLOW}  Building in debug mode...${NC}"
    if cargo build 2>&1 | grep -E "(Compiling|Finished|error)"; then
        echo -e "${GREEN}  ✓ Build completed${NC}"
    else
        echo -e "${RED}  ✗ Build failed${NC}"
        exit 1
    fi
    BINARY="./target/debug/rainbow-poc-chromiumoxide"
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
echo -e "${GREEN}📊 Health: ${BLUE}http://localhost:$SERVER_PORT/api/health${NC}"
echo -e "${YELLOW}🎯 No ChromeDriver needed! Using Chrome DevTools Protocol${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Test browser first
echo -e "${BLUE}Testing browser connection...${NC}"
if $BINARY test --headless 2>&1 | grep -q "All tests passed"; then
    echo -e "${GREEN}  ✓ Browser test passed${NC}"
else
    echo -e "${YELLOW}  ⚠ Browser test had issues, but continuing...${NC}"
fi

# Determine mode (headless or headed)
HEADLESS_ARG=""
if [ "$1" == "--headless" ] || [ "$2" == "--headless" ] || [ "$3" == "--headless" ]; then
    HEADLESS_ARG="--headless"
    echo -e "${YELLOW}  Running in headless mode (no browser window)${NC}"
else
    echo -e "${GREEN}  Running in headed mode (browser window visible)${NC}"
fi

# Start server
echo -e "\n${BLUE}Starting API server...${NC}"
$BINARY serve --port $SERVER_PORT $HEADLESS_ARG &
SERVER_PID=$!

# Wait for server to start
echo -e "${BLUE}⏳ Waiting for server to start...${NC}"
for i in {1..15}; do
    if curl -s http://localhost:$SERVER_PORT/api/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Service started successfully!${NC}"
        echo -e "${GREEN}🌐 Dashboard: ${BLUE}http://localhost:$SERVER_PORT${NC}"
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