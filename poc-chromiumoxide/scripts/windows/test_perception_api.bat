@echo off
echo Testing RainbowBrowserAI Perception API Endpoints
echo ==================================================
echo.

echo Testing Lightning Perception...
curl -X POST http://localhost:3001/api/perceive-mode -H "Content-Type: application/json" -d "{\"mode\":\"lightning\"}" 2>nul
echo.
echo.

echo Testing Quick Scan...
curl -X POST http://localhost:3001/api/quick-scan -H "Content-Type: application/json" -d "{}" 2>nul
echo.
echo.

echo Testing Smart Element Search...
curl -X POST http://localhost:3001/api/smart-element-search -H "Content-Type: application/json" -d "{\"query\":\"submit button\", \"max_results\":5}" 2>nul
echo.
echo.

echo Testing Complete!
echo.
echo If you see 404 errors, please restart the server with:
echo   taskkill /F /IM rainbow-poc-chromiumoxide.exe
echo   cargo run --bin rainbow-poc-chromiumoxide -- serve --port 3001
pause