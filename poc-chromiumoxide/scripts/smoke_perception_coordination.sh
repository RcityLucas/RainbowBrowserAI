#!/usr/bin/env bash
set -euo pipefail

# Simple end-to-end smoke test for session-bound navigation + perception.
# Usage: ./scripts/smoke_perception_coordination.sh 3002

PORT="${1:-3001}"
BASE="http://localhost:${PORT}"

echo "[1/6] Health check on ${BASE}"
curl -fsS "${BASE}/api/health" > /dev/null || {
  echo "Error: server not healthy on ${BASE}" >&2
  exit 1
}

echo "[2/6] Create session"
SESSION_JSON=$(curl -fsS -X POST "${BASE}/api/session/create")
SESSION_ID=$(echo "$SESSION_JSON" | sed -n 's/.*"session_id":"\([^"]*\)".*/\1/p')
if [[ -z "${SESSION_ID}" ]]; then
  echo "Error: could not extract session_id from: $SESSION_JSON" >&2
  exit 1
fi
echo "     session_id=${SESSION_ID}"

TARGET_URL="https://example.com"
echo "[3/6] Navigate (session-aware) => ${TARGET_URL}"
NAV_RES=$(curl -fsS -X POST "${BASE}/api/navigate" \
  -H 'Content-Type: application/json' \
  -d "{\"url\":\"${TARGET_URL}\",\"session_id\":\"${SESSION_ID}\"}")
echo "     navigate result: ${NAV_RES}"

echo "[4/6] Verify session state updated"
SESSION_INFO=$(curl -fsS "${BASE}/api/session/${SESSION_ID}")
CURR_URL=$(echo "$SESSION_INFO" | sed -n 's/.*"current_url":"\([^"]*\)".*/\1/p')
if [[ "${CURR_URL}" != "${TARGET_URL}" ]]; then
  echo "Error: session current_url mismatch. got='${CURR_URL}' expected='${TARGET_URL}'" >&2
  echo "Session info: ${SESSION_INFO}"
  exit 1
fi

echo "[5/6] Perception (lightning) bound to session"
PER_RES=$(curl -fsS -X POST "${BASE}/api/perceive-mode" \
  -H 'Content-Type: application/json' \
  -d "{\"mode\":\"lightning\",\"session_id\":\"${SESSION_ID}\"}")
echo "     perception result: ${PER_RES}"

URL_FIELD=$(echo "$PER_RES" | sed -n 's/.*"url":"\([^"]*\)".*/\1/p')
TITLE_FIELD=$(echo "$PER_RES" | sed -n 's/.*"title":"\([^"]*\)".*/\1/p')

if [[ -z "${URL_FIELD}" || "${URL_FIELD}" == "about:blank" ]]; then
  echo "Error: perception url not set or about:blank (url='${URL_FIELD}')" >&2
  exit 1
fi

echo "[6/6] Success"
echo "     url=${URL_FIELD} title=${TITLE_FIELD}"
exit 0

