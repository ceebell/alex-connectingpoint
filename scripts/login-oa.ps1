param([string]$SshHost='Hermes',[ValidateRange(1,65535)][int]$Port=18082)
$ErrorActionPreference='Stop'
$tunnel=$null
try {
    if ($SshHost -notmatch '^[A-Za-z0-9][A-Za-z0-9_.@-]*$') { throw 'Invalid SSH host.' }
    $listeners=[System.Net.NetworkInformation.IPGlobalProperties]::GetIPGlobalProperties().GetActiveTcpListeners()
    if ($listeners | Where-Object Port -EQ $Port) { throw "Port $Port is already in use. Close the previous login-oa window first." }
    $ssh=(Get-Command ssh.exe -CommandType Application -ErrorAction Stop).Source
    $tunnel=Start-Process -FilePath $ssh -ArgumentList @('-N','-T','-n','-o','BatchMode=yes','-o','ConnectTimeout=10','-o','ExitOnForwardFailure=yes','-o','ServerAliveInterval=15','-L',"127.0.0.1:${Port}:127.0.0.1:${Port}",$SshHost) -PassThru -WindowStyle Hidden
    $deadline=[DateTime]::UtcNow.AddSeconds(12)
    while (-not ([System.Net.NetworkInformation.IPGlobalProperties]::GetIPGlobalProperties().GetActiveTcpListeners() | Where-Object Port -EQ $Port)) {
        if ($tunnel.HasExited -or [DateTime]::UtcNow -ge $deadline) { throw 'SSH login tunnel failed.' }
        Start-Sleep -Milliseconds 150
    }
    $url="http://localhost:$Port"
    $health=Invoke-RestMethod -Uri "$url/health" -TimeoutSec 5
    if ($health.status -ne 'ok') { throw 'Connector health check failed.' }
    Write-Host "Open $url and use the server ADMIN_USERNAME/ADMIN_PASSWORD."
    Write-Host 'Then log in to LINE inside the remote browser. Ctrl+C closes only this login tunnel.'
    Start-Process $url
    while (-not $tunnel.HasExited) { Start-Sleep -Seconds 1 }
} catch { Write-Host ('[ERROR] ' + $_.Exception.Message); exit 1 }
finally { if ($null -ne $tunnel -and -not $tunnel.HasExited) { $tunnel.Kill(); $tunnel.WaitForExit() } }
