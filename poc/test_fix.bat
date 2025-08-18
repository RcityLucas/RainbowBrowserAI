@echo off
echo Testing Mock Mode Batch Fix...
echo.

echo Starting server...
start /B cargo run --bin rainbow-poc

echo Waiting for server startup...
timeout /t 10 /nobreak >nul

echo.
echo Testing batch command:
echo Command: test google,github with screenshots
echo.

curl -X POST http://localhost:3000/command ^
  -H "Content-Type: application/json" ^
  -d "{\"command\":\"test google,github with screenshots\"}" 2>nul

echo.
echo.
echo Test completed! If successful, you should see actual browser automation results instead of a mock message.

echo.
echo Cleaning up...
taskkill /F /IM cargo.exe 2>nul
taskkill /F /IM rainbow-poc.exe 2>nul

pause