param(
  [int[]]$Ports = @(3001,3002,3003,3004,3005)
)

Write-Host "Killing listeners on ports: $($Ports -join ', ')" -ForegroundColor Yellow

foreach ($port in $Ports) {
  try {
    $conns = Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue
    if ($conns) {
      $pids = $conns | Select-Object -ExpandProperty OwningProcess | Sort-Object -Unique
      foreach ($pid in $pids) {
        try {
          Write-Host "  Port $port -> PID $pid : stopping" -ForegroundColor Yellow
          Stop-Process -Id $pid -Force -ErrorAction Stop
        } catch {
          Write-Host "  Port $port -> PID $pid : taskkill fallback" -ForegroundColor Yellow
          taskkill /PID $pid /F | Out-Null
        }
      }
    } else {
      Write-Host "  Port $port : no listeners" -ForegroundColor DarkGray
    }
  } catch {
    Write-Host "  Port $port : error $_" -ForegroundColor Red
  }
}

Write-Host "Done." -ForegroundColor Green

