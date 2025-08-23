#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}       Starting RainbowBrowserAI Service${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Configuration
CHROMEDRIVER_PORT=9515
SERVER_PORT=3001
RETRY_COUNT=3

# Function to check if port is in use
check_port() {
    local port=$1
    # Try multiple methods to check port availability
    if nc -z localhost $port 2>/dev/null; then
        return 0  # Port is in use
    elif netstat -tulpn 2>/dev/null | grep -q ":${port} "; then
        return 0  # Port is in use
    elif lsof -Pi :${port} -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Function to find free port
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
pkill -f rainbow-poc 2>/dev/null && echo -e "${GREEN}  ✓ Killed old rainbow-poc processes${NC}"
pkill -f chromedriver 2>/dev/null && echo -e "${GREEN}  ✓ Killed old chromedriver processes${NC}"
# Also kill any process on our target ports
for port in 9515 9516 3001; do
    pid=$(lsof -ti:$port 2>/dev/null)
    if [ ! -z "$pid" ]; then
        kill -9 $pid 2>/dev/null && echo -e "${GREEN}  ✓ Killed process on port $port${NC}"
    fi
done
sleep 2

# Check ChromeDriver availability
echo -e "\n${BLUE}Checking ChromeDriver...${NC}"
CHROMEDRIVER_CMD=""
if [ -f /usr/bin/chromedriver ]; then
    CHROMEDRIVER_CMD="/usr/bin/chromedriver"
    echo -e "${GREEN}  ✓ Found ChromeDriver at /usr/bin/chromedriver${NC}"
elif [ -f ./chromedriver ]; then
    CHROMEDRIVER_CMD="./chromedriver"
    echo -e "${GREEN}  ✓ Found ChromeDriver at ./chromedriver${NC}"
else
    echo -e "${RED}✗ ChromeDriver not found!${NC}"
    echo -e "${YELLOW}  Please install ChromeDriver first:${NC}"
    echo -e "${YELLOW}  Run: sudo apt-get install chromium-chromedriver${NC}"
    echo -e "${YELLOW}  Or download from: https://chromedriver.chromium.org/${NC}"
    exit 1
fi

# Find free port for ChromeDriver
echo -e "\n${BLUE}Finding available port for ChromeDriver...${NC}"
NEW_CHROMEDRIVER_PORT=$(find_free_port $CHROMEDRIVER_PORT)
if [ $? -ne 0 ]; then
    echo -e "${RED}✗ Failed to find free port for ChromeDriver${NC}"
    exit 1
fi
CHROMEDRIVER_PORT=$NEW_CHROMEDRIVER_PORT
echo -e "${GREEN}  ✓ ChromeDriver will use port ${CHROMEDRIVER_PORT}${NC}"

# Find free port for server
echo -e "\n${BLUE}Finding available port for server...${NC}"
NEW_SERVER_PORT=$(find_free_port $SERVER_PORT)
if [ $? -ne 0 ]; then
    echo -e "${RED}✗ Failed to find free port for server${NC}"
    exit 1
fi
SERVER_PORT=$NEW_SERVER_PORT
echo -e "${GREEN}  ✓ Server will use port ${SERVER_PORT}${NC}"

# Start ChromeDriver
echo -e "\n${BLUE}Starting ChromeDriver on port ${CHROMEDRIVER_PORT}...${NC}"
$CHROMEDRIVER_CMD --port=${CHROMEDRIVER_PORT} > /tmp/chromedriver.log 2>&1 &
CHROMEDRIVER_PID=$!

# Wait for ChromeDriver to start
for i in {1..10}; do
    if check_port $CHROMEDRIVER_PORT; then
        echo -e "${GREEN}  ✓ ChromeDriver started successfully (PID: $CHROMEDRIVER_PID)${NC}"
        break
    fi
    if [ $i -eq 10 ]; then
        echo -e "${RED}✗ ChromeDriver failed to start${NC}"
        echo -e "${YELLOW}  Check /tmp/chromedriver.log for details${NC}"
        exit 1
    fi
    sleep 1
done

# Build the project first (optional, speeds up startup)
echo -e "\n${BLUE}Building the project...${NC}"
echo -e "${YELLOW}  This may take a few minutes on first run...${NC}"
RAINBOW_MOCK_MODE=true cargo build --bin rainbow-poc 2>&1 | while read line; do
    if [[ $line == *"Compiling"* ]]; then
        echo -ne "\r${YELLOW}  ⚙ Compiling... ${NC}"
    elif [[ $line == *"Finished"* ]]; then
        echo -e "\r${GREEN}  ✓ Build completed successfully${NC}"
    fi
done

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down...${NC}"
    if [ ! -z "$CHROMEDRIVER_PID" ]; then
        kill $CHROMEDRIVER_PID 2>/dev/null && echo -e "${GREEN}  ✓ ChromeDriver stopped${NC}"
    fi
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null && echo -e "${GREEN}  ✓ Server stopped${NC}"
    fi
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Set up cleanup on exit
trap cleanup EXIT INT TERM

# Start the main application
echo -e "\n${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🚀 Starting RainbowBrowserAI Server${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}📍 Dashboard URL: ${BLUE}http://localhost:$SERVER_PORT${NC}"
echo -e "${GREEN}📊 Health Check: ${BLUE}http://localhost:$SERVER_PORT/health${NC}"
echo -e "${GREEN}🔧 ChromeDriver: ${BLUE}Port $CHROMEDRIVER_PORT${NC}"
echo -e "${YELLOW}⚙  Mock Mode: ${GREEN}Enabled${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Open browser after a delay
(sleep 5 && echo -e "${GREEN}🌐 Opening dashboard in browser...${NC}" && \
 xdg-open http://localhost:$SERVER_PORT 2>/dev/null || \
 open http://localhost:$SERVER_PORT 2>/dev/null || \
 echo -e "${YELLOW}  Please open http://localhost:$SERVER_PORT in your browser${NC}") &

# Run the main application
export RAINBOW_MOCK_MODE=true
export CHROMEDRIVER_PORT=$CHROMEDRIVER_PORT
exec cargo run --bin rainbow-poc -- serve --port $SERVER_PORT