#!/usr/bin/env bash
set -euo pipefail

PORT=3001
MODE="--headless"

for arg in "$@"; do
  case "$arg" in
    --headed)
      MODE=""
      shift
      ;;
    *)
      ;;
  esac
done

echo "[force_port_3001] Ensuring port $PORT is free..."

# Try to identify current listeners
PIDS=()
if command -v lsof >/dev/null 2>&1; then
  while IFS= read -r pid; do
    [[ -n "$pid" ]] && PIDS+=("$pid")
  done < <(lsof -t -nP -iTCP:$PORT -sTCP:LISTEN || true)
fi

if [[ ${#PIDS[@]} -eq 0 ]] && command -v fuser >/dev/null 2>&1; then
  # fuser exits non-zero if no process is using the port
  set +e
  FUSER_OUT=$(fuser $PORT/tcp 2>/dev/null)
  RC=$?
  set -e
  if [[ $RC -eq 0 ]]; then
    for pid in $FUSER_OUT; do PIDS+=("$pid"); done
  fi
fi

if [[ ${#PIDS[@]} -gt 0 ]]; then
  echo "[force_port_3001] Port $PORT is in use by PID(s): ${PIDS[*]}"
  echo "[force_port_3001] Attempting graceful stop..."
  for pid in "${PIDS[@]}"; do
    kill -TERM "$pid" 2>/dev/null || true
  done
  sleep 1
  # Check again; force kill if still present
  STILL=()
  for pid in "${PIDS[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then STILL+=("$pid"); fi
  done
  if [[ ${#STILL[@]} -gt 0 ]]; then
    echo "[force_port_3001] Forcing stop for PID(s): ${STILL[*]}"
    for pid in "${STILL[@]}"; do
      kill -KILL "$pid" 2>/dev/null || true
    done
    sleep 1
  fi
else
  echo "[force_port_3001] Port $PORT appears free."
fi

echo "[force_port_3001] Starting RainbowBrowser on port $PORT..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$SCRIPT_DIR"

"$SCRIPT_DIR/start.sh" $MODE --port "$PORT" &
SERVER_PID=$!

echo "[force_port_3001] Waiting for health..."
for i in {1..20}; do
  if curl -sSf "http://127.0.0.1:$PORT/api/health" >/dev/null 2>&1; then
    echo "[force_port_3001] Server is up on http://localhost:$PORT"
    exit 0
  fi
  sleep 0.5
done

echo "[force_port_3001] Failed to confirm server health on port $PORT. Check logs."
exit 1

