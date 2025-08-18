@echo off
echo ========================================
echo    COMPARING OLD vs FIXED VERSIONS
echo ========================================
echo.

echo Testing OLD server (port 3000) - WITHOUT fix:
echo ----------------------------------------------
curl -s -X POST http://localhost:3000/command ^
  -H "Content-Type: application/json" ^
  -d "{\"command\":\"test google with screenshots\"}" 2>nul | findstr /C:"message" /C:"results"

echo.
echo.
echo Testing NEW server (port 3001) - WITH fix:
echo ------------------------------------------
curl -s -X POST http://localhost:3001/command ^
  -H "Content-Type: application/json" ^
  -d "{\"command\":\"test google with screenshots\"}" 2>nul | findstr /C:"message" /C:"results"

echo.
echo.
echo ========================================
echo EXPLANATION:
echo.
echo OLD (port 3000): Shows "message": "Test command recognized but not executed"
echo NEW (port 3001): Shows "results": with actual test data and screenshots
echo.
echo To use the FIXED version:
echo 1. Close the terminal running port 3000
echo 2. Use port 3001 for testing
echo    OR
echo 3. Run: switch_to_fixed.ps1
echo ========================================
pause