@echo off
echo Testing Session-Aware Perception API
echo =====================================
echo.

echo 1. Creating a new browser session...
for /f "delims=" %%i in ('curl -s -X POST http://localhost:3001/api/session/create ^| jq -r .data.session_id') do set SESSION_ID=%%i
echo Created session: %SESSION_ID%
echo.

echo 2. Testing perception WITHOUT session ID (uses browser pool)...
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\"}" 2>nul
echo.
echo.

echo 3. Testing perception WITH session ID (uses session browser)...
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\", \"session_id\":\"%SESSION_ID%\"}" 2>nul
echo.
echo.

echo 4. Navigate session to a specific URL...
curl -X POST http://localhost:3001/api/navigate -H "Content-Type: application/json" -d "{\"url\":\"https://example.com\"}" 2>nul
echo.
echo Navigated to example.com
echo.

echo 5. Test perception with session after navigation...
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"quick\", \"session_id\":\"%SESSION_ID%\"}" 2>nul
echo.
echo.

echo 6. Check session info...
curl -s -X GET http://localhost:3001/api/session/%SESSION_ID% | jq .
echo.

echo Session-Aware Perception Test Complete!
pause