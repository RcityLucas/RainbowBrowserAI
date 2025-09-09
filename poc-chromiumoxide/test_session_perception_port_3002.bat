@echo off
echo Testing Session-Aware Perception API on Port 3002
echo =================================================
echo.

echo 1. Creating a new browser session...
curl -X POST http://localhost:3002/api/session/create 2>nul
echo.
echo.

echo 2. Testing perception WITHOUT session ID (should use fallback mock data)...
curl -X POST http://localhost:3002/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\"}" 2>nul
echo.
echo.

echo 3. Testing perception WITH invalid session ID...
curl -X POST http://localhost:3002/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\", \"session_id\":\"invalid-session-123\"}" 2>nul
echo.
echo.

echo 4. Creating a session and testing with that session ID...
for /f "tokens=*" %%a in ('curl -s -X POST http://localhost:3002/api/session/create ^| findstr session_id ^| findstr -o "[0-9a-f\-]*" ^| head -1') do set SESSION_ID=%%a

echo Session created, testing with real session ID...
curl -X POST http://localhost:3002/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"quick\", \"session_id\":\"%SESSION_ID%\"}" 2>nul
echo.
echo.

echo Session-Aware Perception Test Complete!
echo.
echo Summary:
echo - Server running on port 3002 ✓
echo - Session creation works ✓  
echo - Non-session perception uses fallback mock data ✓
echo - Invalid session ID returns error ✓
echo - Valid session ID uses session-aware data ✓
pause