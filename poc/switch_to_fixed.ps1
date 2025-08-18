# Switch to Fixed Version Script
Write-Host "Switching to Fixed RainbowBrowserAI Version..." -ForegroundColor Green
Write-Host ""

# Kill all existing processes
Write-Host "Stopping old processes..." -ForegroundColor Yellow
Get-Process rainbow-poc -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process cargo -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process chromedriver -ErrorAction SilentlyContinue | Stop-Process -Force

Start-Sleep -Seconds 2

# Start ChromeDriver
Write-Host "Starting ChromeDriver..." -ForegroundColor Cyan
Start-Process chromedriver.exe -ArgumentList "--port=9515" -WindowStyle Hidden

Start-Sleep -Seconds 2

# Set environment variable
$env:RAINBOW_MOCK_MODE = "true"

# Start the fixed server
Write-Host "Starting fixed server on port 3000..." -ForegroundColor Green
Write-Host ""
Write-Host "Server will be available at: http://localhost:3000" -ForegroundColor Cyan
Write-Host ""

# Run the server
cargo run --release -- serve --port 3000