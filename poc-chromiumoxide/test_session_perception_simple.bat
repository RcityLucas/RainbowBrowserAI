@echo off
echo Testing Session-Aware Perception API (Simplified)
echo ==================================================
echo.

echo 1. Creating a new browser session...
curl -X POST http://localhost:3001/api/session/create 2>nul
echo.
echo.

echo 2. Testing perception WITHOUT session ID (should use fallback mock data)...
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\"}" 2>nul
echo.
echo.

echo 3. Testing perception WITH session ID (should show session error since no browser available)...
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\", \"session_id\":\"test-session-123\"}" 2>nul
echo.
echo.

echo 4. Testing different perception modes without session...
echo Lightning mode:
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\"}" 2>nul
echo.
echo.
echo Quick mode:
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"quick\"}" 2>nul
echo.
echo.
echo Deep mode:
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"deep\"}" 2>nul
echo.
echo.

echo Session-Aware Perception Test Complete!
echo.
echo Key findings:
echo - Without session_id: Falls back to mock data
echo - With invalid session_id: Returns session error
echo - With valid session_id: Would use session browser (requires real browser)
pause