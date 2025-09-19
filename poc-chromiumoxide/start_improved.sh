#!/bin/bash

# Improved start script with better process management and cleanup

set -o pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     Starting RainbowBrowserAI (Improved Edition)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Configuration
SERVER_PORT=3001
BUILD_MODE="release"
BINARY=""

# Track all spawned PIDs
declare -a SPAWNED_PIDS=()

# Enhanced cleanup function
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down...${NC}"
    
    # First, try graceful shutdown with SIGTERM
    if [ -n "${SERVER_PID:-}" ]; then
        echo -e "${YELLOW}  Sending SIGTERM to main process (PID: $SERVER_PID)...${NC}"
        kill -TERM "$SERVER_PID" 2>/dev/null || true
        
        # Give it 2 seconds to shut down gracefully
        for i in {1..4}; do
            if ! kill -0 "$SERVER_PID" 2>/dev/null; then
                echo -e "${GREEN}  ✓ Process terminated gracefully${NC}"
                break
            fi
            sleep 0.5
        done
        
        # Force kill if still running
        if kill -0 "$SERVER_PID" 2>/dev/null; then
            echo -e "${YELLOW}  Forcing termination...${NC}"
            if command -v taskkill >/dev/null 2>&1; then
                taskkill //PID "$SERVER_PID" //T //F 2>/dev/null || true
            else
                kill -9 "$SERVER_PID" 2>/dev/null || true
            fi
        fi
    fi
    
    # Kill all spawned child processes
    for pid in "${SPAWNED_PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            echo -e "${YELLOW}  Killing child process $pid...${NC}"
            kill -9 "$pid" 2>/dev/null || true
        fi
    done
    
    # Kill by process name (catch any stragglers)
    echo -e "${YELLOW}  Cleaning up any remaining processes...${NC}"
    
    # Windows specific cleanup
    if command -v taskkill >/dev/null 2>&1; then
        # Get all PIDs for rainbow processes
        tasklist | grep -i "rainbow-poc-chromiumoxide" | while read -r line; do
            pid=$(echo "$line" | awk '{print $2}')
            if [ -n "$pid" ] && [ "$pid" != "PID" ]; then
                taskkill //PID "$pid" //F 2>/dev/null || true
            fi
        done
    fi
    
    # POSIX cleanup
    pkill -f rainbow-poc-chromiumoxide 2>/dev/null || true
    
    # Clean up ports explicitly
    cleanup_ports
    
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Function to clean up ports
cleanup_ports() {
    echo -e "${YELLOW}  Freeing ports...${NC}"
    
    for port in 3001 3002 3003 3004 3005 3006 3007 3008 3009 3010; do
        if command -v powershell >/dev/null 2>&1; then
            # Get PIDs using the port
            local pids=$(powershell -NoProfile -Command \
                "Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue | Select -ExpandProperty OwningProcess" 2>/dev/null | tr -d '\r')
            
            if [ -n "$pids" ]; then
                for pid in $pids; do
                    echo -e "${YELLOW}    Killing process $pid on port $port${NC}"
                    taskkill //PID "$pid" //F 2>/dev/null || true
                done
            fi
        elif command -v lsof >/dev/null 2>&1; then
            # Unix/Linux fallback
            local pids=$(lsof -t -i :$port 2>/dev/null)
            if [ -n "$pids" ]; then
                echo -e "${YELLOW}    Killing processes on port $port: $pids${NC}"
                kill -9 $pids 2>/dev/null || true
            fi
        fi
    done
}

# Check if port is in use
check_port() {
    local port=$1
    if command -v powershell >/dev/null 2>&1; then
        local used=$(powershell -NoProfile -Command \
            "if (Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue) { 'inuse' }" 2>/dev/null)
        [ "$used" == "inuse" ]
    else
        netstat -an 2>/dev/null | grep -Ei ":${port}[^0-9].*(LISTEN|LISTENING)" >/dev/null
    fi
}

# Find free port
find_free_port() {
    local base_port=$1
    local port=$base_port
    while check_port $port; do
        echo -e "${YELLOW}⚠ Port $port is in use, cleaning up...${NC}"
        # Try to clean the port
        cleanup_specific_port $port
        sleep 1
        # Check again
        if ! check_port $port; then
            echo -e "${GREEN}  ✓ Port $port freed${NC}"
            break
        fi
        # If still in use, try next port
        port=$(($port + 1))
        if [ $port -gt $(($base_port + 10)) ]; then
            echo -e "${RED}✗ Could not find free port after 10 attempts${NC}"
            return 1
        fi
    done
    echo $port
}

# Clean specific port
cleanup_specific_port() {
    local port=$1
    if command -v powershell >/dev/null 2>&1; then
        local pids=$(powershell -NoProfile -Command \
            "Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue | Select -ExpandProperty OwningProcess" 2>/dev/null | tr -d '\r')
        if [ -n "$pids" ]; then
            for pid in $pids; do
                echo -e "${YELLOW}    Force killing PID $pid on port $port${NC}"
                taskkill //PID "$pid" //F 2>/dev/null || true
            done
        fi
    fi
}

# Enhanced trap handling
trap_handler() {
    echo -e "\n${RED}Interrupt received!${NC}"
    cleanup
    exit 130
}

# Set up signal traps
trap trap_handler INT TERM
trap cleanup EXIT

# Clean up before starting
echo -e "${YELLOW}🔄 Initial cleanup...${NC}"
cleanup_ports

# Find free port
echo -e "\n${BLUE}Finding available port...${NC}"
SERVER_PORT=$(find_free_port $SERVER_PORT)
if [ $? -ne 0 ]; then
    echo -e "${RED}✗ Failed to find free port${NC}"
    exit 1
fi
echo -e "${GREEN}  ✓ Server will use port ${SERVER_PORT}${NC}"

# Build the project
echo -e "\n${BLUE}Building the project...${NC}"
if [ "$BUILD_MODE" = "release" ]; then
    echo -e "${YELLOW}  Building in release mode...${NC}"
    if cargo build --release 2>&1 | grep -E "^[[:space:]]*(Compiling|Finished|error(\[|:))"; then
        echo -e "${GREEN}  ✓ Build completed${NC}"
    else
        echo -e "${RED}  ✗ Build failed${NC}"
        exit 1
    fi
    
    if [[ -f "./target/release/rainbow-poc-chromiumoxide.exe" ]]; then
        BINARY="./target/release/rainbow-poc-chromiumoxide.exe"
    else
        BINARY="./target/release/rainbow-poc-chromiumoxide"
    fi
else
    echo -e "${YELLOW}  Building in debug mode...${NC}"
    if cargo build 2>&1 | grep -E "^[[:space:]]*(Compiling|Finished|error(\[|:))"; then
        echo -e "${GREEN}  ✓ Build completed${NC}"
    else
        echo -e "${RED}  ✗ Build failed${NC}"
        exit 1
    fi
    
    if [[ -f "./target/debug/rainbow-poc-chromiumoxide.exe" ]]; then
        BINARY="./target/debug/rainbow-poc-chromiumoxide.exe"
    else
        BINARY="./target/debug/rainbow-poc-chromiumoxide"
    fi
fi

# Start the application
echo -e "\n${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🚀 Starting RainbowBrowserAI Server${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}📍 Dashboard: ${BLUE}http://localhost:$SERVER_PORT${NC}"
echo -e "${GREEN}📊 Health API: ${BLUE}http://localhost:$SERVER_PORT/api/health${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop gracefully${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Start server in foreground (not background) for better signal handling
if [ "$1" == "--headless" ]; then
    echo -e "${YELLOW}Running in headless mode${NC}"
    exec "$BINARY" serve --port "$SERVER_PORT" --headless
else
    echo -e "${GREEN}Running in headed mode${NC}"
    exec "$BINARY" serve --port "$SERVER_PORT"
fi