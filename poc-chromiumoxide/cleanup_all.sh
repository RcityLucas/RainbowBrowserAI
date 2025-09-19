#!/bin/bash

# Enhanced cleanup script to kill ALL rainbow processes and free ALL ports

echo "==================================="
echo "FULL CLEANUP - Rainbow Browser AI"
echo "==================================="
echo ""

# Kill all processes on ports 3001-3020
echo "Killing processes on ports 3001-3020..."
for port in {3001..3020}; do
    if command -v powershell >/dev/null 2>&1; then
        pids=$(powershell -NoProfile -Command "Get-NetTCPConnection -State Listen -LocalPort $port 2>nul | Select -ExpandProperty OwningProcess" 2>/dev/null | tr -d '\r')
        if [ -n "$pids" ]; then
            for pid in $pids; do
                echo "  Killing PID $pid on port $port"
                taskkill //PID "$pid" //F 2>/dev/null || true
                powershell -NoProfile -Command "Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue" 2>/dev/null || true
            done
        fi
    fi
done

# Kill all rainbow processes by name
echo ""
echo "Killing all rainbow processes by name..."
tasklist | grep -i rainbow | while read -r line; do
    pid=$(echo "$line" | awk '{print $2}')
    if [ -n "$pid" ]; then
        echo "  Killing rainbow process PID: $pid"
        taskkill //PID "$pid" //F 2>/dev/null || true
    fi
done

# Also use pattern matching
taskkill //IM rainbow-poc-chromiumoxide.exe //F 2>/dev/null || true
pkill -f rainbow 2>/dev/null || true

# Wait for processes to die
sleep 2

# Verify cleanup
echo ""
echo "Verification:"
echo "-------------"

# Check for remaining processes
remaining=$(tasklist | grep -i rainbow | wc -l)
if [ "$remaining" -eq 0 ]; then
    echo "✓ All rainbow processes killed"
else
    echo "⚠ Warning: $remaining rainbow processes still running"
    tasklist | grep -i rainbow
fi

# Check ports
echo ""
echo "Port status:"
occupied_ports=""
for port in {3001..3012}; do
    if netstat -an | grep -q ":${port}.*LISTEN"; then
        occupied_ports="$occupied_ports $port"
    fi
done

if [ -z "$occupied_ports" ]; then
    echo "✓ All ports 3001-3012 are free"
else
    echo "⚠ Still occupied ports:$occupied_ports"
fi

echo ""
echo "Cleanup complete!"