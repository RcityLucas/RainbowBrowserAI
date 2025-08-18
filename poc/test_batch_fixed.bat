@echo off
echo Testing batch functionality (Fixed Version)...
echo ==========================================
echo.

echo Test 1: Testing 3 websites with screenshots
echo Command: test google,github,stackoverflow with screenshots
echo.

curl -X POST http://localhost:3000/command ^
  -H "Content-Type: application/json" ^
  -d "{\"command\":\"test google,github,stackoverflow with screenshots\"}" 2>nul

echo.
echo.
echo Test completed! 
echo.
echo If you see:
echo   - "message": "Test command recognized but not executed" = OLD VERSION (not fixed)
echo   - "results": [...] with actual test data = NEW VERSION (fixed!)
echo.
echo Check the screenshots folder for captured images.
echo.
pause