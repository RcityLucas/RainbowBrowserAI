@echo off
cls
color 0E
title Fixing ChromeDriver Version Mismatch

echo =====================================
echo    ChromeDriver Version Fix
echo =====================================
echo.
echo Your Chrome: Version 120
echo Current ChromeDriver: Version 114
echo.
echo Fixing this now...
echo.

REM Stop old ChromeDriver
taskkill /F /IM chromedriver.exe 2>nul
timeout /t 1 >nul

REM Backup old ChromeDriver
if exist chromedriver.exe (
    echo Backing up old ChromeDriver...
    move chromedriver.exe chromedriver_old.exe >nul 2>&1
)

echo.
echo Downloading ChromeDriver 120...
echo This may take a minute...
echo.

REM Download ChromeDriver 120
powershell -Command "$ProgressPreference='SilentlyContinue'; try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $url = 'https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/120.0.6099.109/win32/chromedriver-win32.zip'; Write-Host 'Downloading from: ' $url -ForegroundColor Gray; Invoke-WebRequest -Uri $url -OutFile 'chromedriver.zip'; Write-Host 'Extracting...' -ForegroundColor Yellow; Expand-Archive 'chromedriver.zip' -Force; if (Test-Path '.\chromedriver-win32\chromedriver.exe') { Move-Item '.\chromedriver-win32\chromedriver.exe' '.' -Force; Write-Host 'Success!' -ForegroundColor Green } else { Move-Item '.\chromedriver-win64\chromedriver.exe' '.' -Force -ErrorAction SilentlyContinue }; Remove-Item 'chromedriver.zip' -Force; Remove-Item 'chromedriver-win32' -Recurse -Force -ErrorAction SilentlyContinue; Remove-Item 'chromedriver-win64' -Recurse -Force -ErrorAction SilentlyContinue } catch { Write-Host 'Error: ' $_.Exception.Message -ForegroundColor Red }"

if exist chromedriver.exe (
    echo.
    echo =====================================
    echo    SUCCESS! ChromeDriver Updated
    echo =====================================
    echo.
    chromedriver --version
    echo.
    echo Now starting the application...
    echo.
    timeout /t 2 >nul
    
    REM Start ChromeDriver
    start /B chromedriver.exe --port=9515 --silent
    timeout /t 2 >nul
    
    REM Start server
    echo Starting server at http://localhost:3000
    echo Browser will open in 5 seconds...
    echo.
    start /min cmd /c "timeout /t 5 >nul && start http://localhost:3000"
    cargo run -- serve --port 3000
) else (
    echo.
    echo =====================================
    echo    MANUAL DOWNLOAD REQUIRED
    echo =====================================
    echo.
    echo Automatic download failed.
    echo.
    echo Please download manually:
    echo 1. Visit: https://googlechromelabs.github.io/chrome-for-testing/
    echo 2. Find version 120.0.6099.x
    echo 3. Download chromedriver-win32.zip
    echo 4. Extract chromedriver.exe here
    echo.
    echo Or try alternative URL:
    echo https://chromedriver.storage.googleapis.com/index.html?path=120.0.6099.109/
    echo.
    pause
)