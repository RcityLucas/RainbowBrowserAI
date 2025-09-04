# RainbowBrowserAI Chromiumoxide - PowerShell Start Script
# Usage: .\Start-RainbowBrowser.ps1 [-Port 3002] [-Headless] [-Debug]

param(
    [int]$Port = 3002,
    [switch]$Headless,
    [switch]$Debug,
    [switch]$OpenBrowser,
    [switch]$Verbose
)

# Set colors for output
$Host.UI.RawUI.ForegroundColor = "White"

function Write-ColorOutput($ForegroundColor, $Message) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    Write-Output $Message
    $host.UI.RawUI.ForegroundColor = $fc
}

function Test-Port($Port) {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    try {
        $tcpClient.Connect("localhost", $Port)
        $tcpClient.Close()
        return $true
    } catch {
        return $false
    }
}

function Find-FreePort($BasePort) {
    $port = $BasePort
    $maxTries = 10
    $tries = 0
    
    while ((Test-Port $port) -and ($tries -lt $maxTries)) {
        Write-ColorOutput Yellow "Port $port is in use, trying $(($port + 1))..."
        $port++
        $tries++
    }
    
    if ($tries -eq $maxTries) {
        throw "Could not find free port after $maxTries attempts"
    }
    
    return $port
}

function Start-RainbowBrowserService {
    Clear-Host
    Write-ColorOutput Cyan @"
═══════════════════════════════════════════════════════════════
     🌈 RainbowBrowserAI - Chromiumoxide Edition 🌈
═══════════════════════════════════════════════════════════════
"@

    # Check if Rust is installed
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-ColorOutput Red "❌ Cargo not found. Please install Rust from https://rustup.rs/"
        return
    }

    # Kill existing processes
    Write-ColorOutput Yellow "🔄 Cleaning up old processes..."
    Get-Process | Where-Object { $_.ProcessName -like "*rainbow-poc-chromiumoxide*" } | Stop-Process -Force -ErrorAction SilentlyContinue
    Get-Process | Where-Object { $_.MainWindowTitle -like "*DevTools*" } | Stop-Process -Force -ErrorAction SilentlyContinue
    Write-ColorOutput Green "  ✓ Cleanup complete"

    # Find available port
    Write-ColorOutput Blue "`n🔍 Finding available port..."
    try {
        $actualPort = Find-FreePort $Port
        Write-ColorOutput Green "  ✓ Using port $actualPort"
    } catch {
        Write-ColorOutput Red "❌ $_"
        return
    }

    # Build the project
    $buildMode = if ($Debug) { "debug" } else { "release" }
    Write-ColorOutput Blue "`n🔨 Building project in $buildMode mode..."
    
    if ($buildMode -eq "release") {
        $buildResult = cargo build --release 2>&1
        $binary = ".\target\release\rainbow-poc-chromiumoxide.exe"
    } else {
        $buildResult = cargo build 2>&1
        $binary = ".\target\debug\rainbow-poc-chromiumoxide.exe"
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput Red "❌ Build failed"
        if ($Verbose) {
            Write-Output $buildResult
        }
        return
    }
    Write-ColorOutput Green "  ✓ Build completed"

    # Test browser connection
    Write-ColorOutput Blue "`n🧪 Testing browser connection..."
    $testArgs = if ($Headless) { "test", "--headless" } else { "test" }
    $testResult = & $binary $testArgs 2>&1
    
    if ($testResult -match "All tests passed") {
        Write-ColorOutput Green "  ✓ Browser test passed"
    } else {
        Write-ColorOutput Yellow "  ⚠ Browser test had issues, but continuing..."
        if ($Verbose) {
            Write-Output $testResult
        }
    }

    # Start the server
    Write-ColorOutput Cyan @"

═══════════════════════════════════════════════════════════════
           🚀 Starting RainbowBrowserAI Server 🚀
═══════════════════════════════════════════════════════════════
"@
    
    $modeText = if ($Headless) { "HEADLESS" } else { "HEADED (Browser Visible)" }
    
    Write-ColorOutput Green "  📍 Dashboard: http://localhost:$actualPort"
    Write-ColorOutput Green "  📊 API Health: http://localhost:$actualPort/api/health"
    Write-ColorOutput Green "  🔧 Tools API: http://localhost:$actualPort/api/tools"
    Write-ColorOutput Yellow "  🎯 Mode: $modeText"
    Write-ColorOutput Cyan "═══════════════════════════════════════════════════════════════"
    Write-Output ""

    # Prepare server arguments
    $serverArgs = @("serve", "--port", $actualPort)
    if ($Headless) {
        $serverArgs += "--headless"
    }

    # Start server in background job
    $job = Start-Job -ScriptBlock {
        param($binary, $args)
        & $binary $args
    } -ArgumentList $binary, $serverArgs

    # Wait for server to start
    Write-ColorOutput Blue "⏳ Waiting for server to start..."
    $maxWait = 30
    $waited = 0
    
    while ($waited -lt $maxWait) {
        Start-Sleep -Seconds 2
        $waited += 2
        
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:$actualPort/api/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
            if ($response.StatusCode -eq 200) {
                Write-ColorOutput Green "`n✅ Service started successfully!"
                
                if ($OpenBrowser) {
                    Write-ColorOutput Blue "🌐 Opening dashboard in browser..."
                    Start-Process "http://localhost:$actualPort"
                }
                
                Write-ColorOutput Yellow "`nPress Ctrl+C to stop the server"
                Write-ColorOutput Cyan "═══════════════════════════════════════════════════════════════`n"
                
                # Stream job output
                try {
                    while ($true) {
                        Receive-Job $job
                        Start-Sleep -Milliseconds 500
                        
                        # Check if job is still running
                        if ($job.State -ne 'Running') {
                            Write-ColorOutput Red "`n❌ Server stopped unexpectedly"
                            break
                        }
                    }
                } finally {
                    # Cleanup on exit
                    Write-ColorOutput Yellow "`n🛑 Shutting down..."
                    Stop-Job $job
                    Remove-Job $job
                    Get-Process | Where-Object { $_.ProcessName -like "*rainbow-poc-chromiumoxide*" } | Stop-Process -Force -ErrorAction SilentlyContinue
                    Write-ColorOutput Green "✓ Shutdown complete"
                }
                
                return
            }
        } catch {
            # Continue waiting
        }
    }
    
    Write-ColorOutput Red "❌ Service failed to start after $maxWait seconds"
    Stop-Job $job -ErrorAction SilentlyContinue
    Remove-Job $job -ErrorAction SilentlyContinue
}

# Run the service
Start-RainbowBrowserService