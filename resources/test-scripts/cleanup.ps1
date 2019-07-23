param([Int32]$port)

$proc = Get-Process -Id (Get-NetTCPConnection -LocalPort $port).OwningProcess
Write-Host "Found process $proc.Id running on $port"
Write-Host "Killing process $proc.Id"
Stop-Process -Id $proc.Id
