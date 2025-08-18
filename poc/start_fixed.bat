@echo off
echo Starting RainbowBrowserAI (Fixed Version)...
echo.

REM Kill old processes
taskkill /F /IM rainbow-poc.exe 2>nul
taskkill /F /IM chromedriver.exe 2>nul
taskkill /F /IM cargo.exe 2>nul

echo Waiting for processes to stop...
timeout /t 3 >nul

REM Check ChromeDriver
if not exist chromedriver.exe (
    echo ChromeDriver not found! Run: FIX_CHROMEDRIVER.bat
    pause
    exit /b 1
)

REM Start ChromeDriver
echo Starting ChromeDriver...
start /B chromedriver.exe --port=9515
timeout /t 2 >nul

REM Enable Mock Mode
set RAINBOW_MOCK_MODE=true

REM Build fresh version with fix
echo Building fixed version...
cargo build --release

REM Start server with fixed version
echo.
echo Starting server with batch testing fix...
echo Server: http://localhost:3000
echo.
start /min cmd /c "timeout /t 5 >nul && start http://localhost:3000"
cargo run --release -- serve --port 3000