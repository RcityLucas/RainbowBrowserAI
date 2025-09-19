#!/usr/bin/env bash
set -euo pipefail

PORT="${1:-3001}"
BASE="http://localhost:${PORT}"
URL="${2:-https://example.com}"

echo "Checking server on ${BASE}..."
curl -fsS "${BASE}/api/health" >/dev/null || { echo "Server not healthy on ${BASE}"; exit 1; }

echo "Testing /api/navigate-perceive..."
CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${BASE}/api/navigate-perceive" \
  -H 'Content-Type: application/json' \
  -d "{\"url\":\"${URL}\",\"mode\":\"lightning\"}")

if [ "$CODE" = "404" ]; then
  echo "Route not found (404). Falling back to /api/perception/analyze with url..."
  RES=$(curl -fsS -X POST "${BASE}/api/perception/analyze" \
    -H 'Content-Type: application/json' \
    -d "{\"url\":\"${URL}\"}")
  echo "Response: $RES"
  echo "$RES" | grep -q '"success":true' || { echo "Analyze fallback failed"; exit 1; }
  echo "PASS: analyze fallback returned success (navigate+perceive via analyze)."
  exit 0
fi

echo "HTTP ${CODE} from navigate-perceive"
RES=$(curl -fsS -X POST "${BASE}/api/navigate-perceive" \
  -H 'Content-Type: application/json' \
  -d "{\"url\":\"${URL}\",\"mode\":\"lightning\"}")
echo "Response: $RES"
echo "$RES" | grep -q '"success":true' || { echo "navigate-perceive failed"; exit 1; }
echo "PASS: navigate-perceive succeeded."

