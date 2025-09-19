#!/usr/bin/env bash
# Cross-platform port killer for RainbowBrowserAI (bash).
# Usage: ./scripts/kill_ports.sh 3001 3002 3003
set -euo pipefail

PORTS=("$@")
if [ ${#PORTS[@]} -eq 0 ]; then
  PORTS=(3001 3002 3003 3004 3005)
fi

kill_port_posix() {
  local port=$1
  if command -v lsof >/dev/null 2>&1; then
    local pids
    pids=$(lsof -t -i :"$port" 2>/dev/null || true)
    if [ -n "$pids" ]; then
      echo "Killing PIDs on port $port: $pids"
      kill -9 $pids 2>/dev/null || true
    fi
  elif command -v fuser >/dev/null 2>&1; then
    echo "Killing processes on port $port via fuser"
    fuser -k -n tcp "$port" 2>/dev/null || true
  else
    echo "No lsof/fuser; skipping POSIX kill for port $port"
  fi
}

kill_port_windows() {
  local port=$1
  if command -v powershell >/dev/null 2>&1; then
    local pids
    pids=$(powershell -NoProfile -Command "Get-NetTCPConnection -State Listen -LocalPort $port | Select -ExpandProperty OwningProcess" 2>/dev/null | tr -d '\r' || true)
    if [ -n "$pids" ]; then
      echo "Killing PIDs on port $port: $pids"
      for pid in $pids; do
        powershell -NoProfile -Command "Try { Stop-Process -Id $pid -Force -ErrorAction Stop } Catch {}" 2>/dev/null || true
        taskkill /PID "$pid" /F >/dev/null 2>&1 || true
      done
    fi
  else
    echo "No PowerShell detected; skipping Windows kill for port $port"
  fi
}

for p in "${PORTS[@]}"; do
  echo "Scanning port $p..."
  if command -v powershell >/dev/null 2>&1; then
    kill_port_windows "$p"
  else
    kill_port_posix "$p"
  fi
done

echo "Done."

