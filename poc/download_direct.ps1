# Direct download with multiple fallbacks for Chrome 120

Write-Host "Downloading ChromeDriver 120 with multiple fallbacks..." -ForegroundColor Yellow

# URLs to try
$urls = @(
    "https://storage.googleapis.com/chrome-for-testing-public/120.0.6099.109/win32/chromedriver-win32.zip",
    "https://storage.googleapis.com/chrome-for-testing-public/120.0.6099.109/win64/chromedriver-win64.zip",
    "https://edgedl.me.gvt1.com/edgedl/chrome/chrome-for-testing/120.0.6099.109/win32/chromedriver-win32.zip",
    "https://chromedriver.storage.googleapis.com/120.0.6099.109/chromedriver_win32.zip"
)

foreach ($url in $urls) {
    Write-Host "Trying: $url" -ForegroundColor Cyan
    
    try {
        # Download with Invoke-RestMethod for better error handling
        $response = Invoke-WebRequest -Uri $url -OutFile "chromedriver.zip" -PassThru
        
        if ($response.StatusCode -eq 200) {
            Write-Host "Download successful!" -ForegroundColor Green
            
            # Extract
            if (Get-Command Expand-Archive -ErrorAction SilentlyContinue) {
                Expand-Archive -Path "chromedriver.zip" -DestinationPath "temp" -Force
            } else {
                # Fallback for older PowerShell
                Add-Type -AssemblyName System.IO.Compression.FileSystem
                [System.IO.Compression.ZipFile]::ExtractToDirectory("chromedriver.zip", "temp")
            }
            
            # Find chromedriver.exe
            $exe = Get-ChildItem -Path "temp" -Recurse -Filter "chromedriver.exe" | Select-Object -First 1
            
            if ($exe) {
                Move-Item $exe.FullName "chromedriver.exe" -Force
                Remove-Item "chromedriver.zip" -Force
                Remove-Item "temp" -Recurse -Force
                
                Write-Host "✅ ChromeDriver 120 installed successfully!" -ForegroundColor Green
                & .\chromedriver.exe --version
                exit 0
            }
        }
    } catch {
        Write-Host "Failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host "All downloads failed. Opening browser for manual download..." -ForegroundColor Yellow
Start-Process "https://googlechromelabs.github.io/chrome-for-testing/#stable"