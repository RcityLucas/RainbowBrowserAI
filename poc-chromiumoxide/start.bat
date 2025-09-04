@echo off
REM RainbowBrowserAI Chromiumoxide - Windows Start Script

echo ===============================================================
echo      Starting RainbowBrowserAI (Chromiumoxide Edition)
echo ===============================================================
echo.

REM Configuration
set SERVER_PORT=3002
set BUILD_MODE=release
set HEADLESS_MODE=false

REM Parse command line arguments
:parse_args
if "%1"=="" goto end_parse
if /i "%1"=="--headless" (
    set HEADLESS_MODE=true
    echo [INFO] Running in headless mode
)
if /i "%1"=="--debug" (
    set BUILD_MODE=debug
    echo [INFO] Building in debug mode
)
if /i "%1"=="--port" (
    if not "%2"=="" (
        set SERVER_PORT=%2
        shift
    )
)
shift
goto parse_args
:end_parse

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Cargo not found. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Kill any existing processes
echo [INFO] Cleaning up old processes...
taskkill /F /IM rainbow-poc-chromiumoxide.exe >nul 2>nul
taskkill /F /IM chrome.exe /FI "WINDOWTITLE eq DevTools*" >nul 2>nul

REM Build the project
echo.
echo [INFO] Building the project in %BUILD_MODE% mode...
if "%BUILD_MODE%"=="release" (
    cargo build --release
    if %errorlevel% neq 0 (
        echo [ERROR] Build failed
        pause
        exit /b 1
    )
    set BINARY=target\release\rainbow-poc-chromiumoxide.exe
) else (
    cargo build
    if %errorlevel% neq 0 (
        echo [ERROR] Build failed
        pause
        exit /b 1
    )
    set BINARY=target\debug\rainbow-poc-chromiumoxide.exe
)

echo [SUCCESS] Build completed
echo.

REM Test browser connection
echo [INFO] Testing browser connection...
if "%HEADLESS_MODE%"=="true" (
    %BINARY% test --headless
) else (
    %BINARY% test
)
if %errorlevel% equ 0 (
    echo [SUCCESS] Browser test passed
) else (
    echo [WARNING] Browser test had issues, but continuing...
)

REM Start the server
echo.
echo ===============================================================
echo    Starting RainbowBrowserAI Server (Chromiumoxide)
echo ===============================================================
echo    Dashboard: http://localhost:%SERVER_PORT%
echo    API Health: http://localhost:%SERVER_PORT%/api/health
echo    Mode: %HEADLESS_MODE%
echo ===============================================================
echo.

if "%HEADLESS_MODE%"=="true" (
    echo [INFO] Starting server in headless mode on port %SERVER_PORT%...
    %BINARY% serve --port %SERVER_PORT% --headless
) else (
    echo [INFO] Starting server in headed mode on port %SERVER_PORT%...
    echo [INFO] Chrome browser window will be visible
    %BINARY% serve --port %SERVER_PORT%
)

pause