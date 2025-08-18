@echo off
echo Starting RainbowBrowserAI...
echo.

REM Kill old processes
taskkill /F /IM rainbow-poc.exe 2>nul
taskkill /F /IM chromedriver.exe 2>nul

REM Check ChromeDriver
if not exist chromedriver.exe (
    echo ChromeDriver not found! Run: FIX_CHROMEDRIVER.bat
    pause
    exit /b 1
)

REM Start ChromeDriver
start /B chromedriver.exe --port=9515
timeout /t 2 >nul

REM Start server
echo Server: http://localhost:3000
start /min cmd /c "timeout /t 5 >nul && start http://localhost:3000"
cargo run -- serve --port 3000