@echo off
echo Testing batch functionality...
echo.

echo Command: test google,github,stackoverflow with screenshots
echo.

curl -X POST http://localhost:3000/command ^
  -H "Content-Type: application/json" ^
  -d "{\"command\":\"test google,github,stackoverflow with screenshots\"}" 2>nul

echo.
echo.
echo Test completed! Check the screenshots folder for results.
pause