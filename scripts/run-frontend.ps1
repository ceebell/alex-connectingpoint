param([switch]$Check, [switch]$ProbeOnly)

$ErrorActionPreference = 'Stop'
$repoPath = Split-Path -Parent $PSScriptRoot
$frontendPath = Join-Path $repoPath 'frontend'
$envPath = Join-Path $repoPath '.env'
$envLines = @()
$tunnel = $null
$tunnelLog = $null
$exitCode = 0

function Get-FrontendSetting([string]$Name, [string]$Default) {
    $value = [Environment]::GetEnvironmentVariable($Name, 'Process')
    if ($null -ne $value) { return $value }
    foreach ($line in $envLines) {
        if ($line -match ('^\s*(?:export\s+)?' + [regex]::Escape($Name) + '\s*=\s*(.*)$')) {
            $value = $Matches[1].Trim()
            if ($value -match '^"([^"\r\n]*)"\s*(?:#.*)?$') { return $Matches[1] }
            if ($value -match "^'([^']*)'\s*(?:#.*)?$") { return $Matches[1] }
            return ($value -replace '\s+#.*$', '').Trim()
        }
    }
    return $Default
}

function Get-FrontendPort([string]$Name, [int]$Default) {
    $port = 0
    if (-not [int]::TryParse((Get-FrontendSetting $Name "$Default"), [ref]$port) -or $port -lt 1 -or $port -gt 65535) {
        throw "$Name must be an integer between 1 and 65535."
    }
    return $port
}

function Test-FrontendPort([int]$Port) {
    return [bool]([System.Net.NetworkInformation.IPGlobalProperties]::GetIPGlobalProperties().GetActiveTcpListeners() |
        Where-Object Port -EQ $Port | Select-Object -First 1)
}

function Get-PortProcesses([int]$Port) {
    $owners = @(Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue |
        Select-Object -ExpandProperty OwningProcess -Unique)
    if ($owners.Count -eq 0) { throw "Cannot identify the process listening on port $Port. Retry after it finishes starting." }
    foreach ($ownerId in $owners) {
        $process = Get-CimInstance Win32_Process -Filter "ProcessId = $ownerId"
        if ($null -eq $process -or [string]::IsNullOrWhiteSpace($process.CommandLine)) {
            throw "Cannot identify the process listening on port $Port. No processes were stopped."
        }
        $process
    }
}

function Get-ExistingFrontend([int]$Port, [string]$BackendUrl) {
    $processes = @(Get-PortProcesses $Port)
    $serverPath = Join-Path $frontendPath 'node_modules\next\dist\server\lib\start-server.js'
    $serverPattern = '(?:^|\s)"?' + [regex]::Escape($serverPath) + '"?(?:\s|$)'
    if ($processes.Count -ne 1 -or $processes[0].Name -ine 'node.exe' -or
        $processes[0].CommandLine -notmatch $serverPattern) {
        throw "Frontend port $Port belongs to another application. Close that application or set DEV_FRONTEND_PORT. No processes were stopped."
    }
    # Check the running dev server's API destination before reusing it.
    $manifestPath = Join-Path $frontendPath '.next\dev\routes-manifest.json'
    if (-not (Test-Path -LiteralPath $manifestPath)) {
        throw "Frontend port $Port belongs to this project, but its API destination cannot be verified. Stop its original window with Ctrl+C, then rerun runall.bat."
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Encoding UTF8 -Raw | ConvertFrom-Json
    $rewrites = @($manifest.rewrites.beforeFiles) + @($manifest.rewrites.afterFiles) + @($manifest.rewrites.fallback)
    $apiRewrites = @($rewrites | Where-Object { $_.source -eq '/api/:path*' })
    if ($apiRewrites.Count -ne 1 -or $apiRewrites[0].destination -cne "$BackendUrl/api/:path*") {
        throw "Frontend port $Port is running with a different backend. Stop its original window with Ctrl+C, then rerun runall.bat to load the new configuration."
    }
    return Get-Process -Id $processes[0].ProcessId -ErrorAction Stop
}

function Test-ExistingTunnel([int]$Port, [int]$RemotePort, [string]$SshHost) {
    $forward = "127.0.0.1:${Port}:127.0.0.1:${RemotePort}"
    $forwardPattern = '(?:^|\s)-L\s+"?' + [regex]::Escape($forward) + '"?(?:\s|$)'
    $hostPattern = '(?:^|\s)"?' + [regex]::Escape($SshHost) + '"?\s*$'
    foreach ($process in @(Get-PortProcesses $Port)) {
        if ($process.Name -ine 'ssh.exe' -or $process.CommandLine -notmatch $forwardPattern -or
            $process.CommandLine -notmatch $hostPattern -or $process.CommandLine -notmatch '(?:^|\s)-N(?:\s|$)') {
            throw "Tunnel port $Port belongs to another process or SSH destination. Close its original window or set DEV_TUNNEL_PORT. No processes were stopped."
        }
    }
    return $true
}

try {
    if (Test-Path -LiteralPath $envPath) { $envLines = Get-Content -LiteralPath $envPath -Encoding UTF8 }
    $sshHost = Get-FrontendSetting 'DEV_SSH_HOST' 'Hermes'
    if ($sshHost -notmatch '^[A-Za-z0-9][A-Za-z0-9_.@-]*$') { throw 'DEV_SSH_HOST must be an SSH alias or user@hostname.' }
    $tunnelPort = Get-FrontendPort 'DEV_TUNNEL_PORT' 28080
    $remotePort = Get-FrontendPort 'DEV_REMOTE_PORT' 18080
    $frontendPort = Get-FrontendPort 'DEV_FRONTEND_PORT' 3000
    if ($frontendPort -eq $tunnelPort) { throw 'Frontend and tunnel ports must differ.' }
    $ssh = Get-Command ssh.exe -CommandType Application -ErrorAction Stop | Select-Object -First 1
    $null = Get-Command node.exe -CommandType Application -ErrorAction Stop
    $npm = Get-Command npm.cmd -CommandType Application -ErrorAction Stop | Select-Object -First 1
    if (-not (Test-Path -LiteralPath (Join-Path $frontendPath 'package.json'))) { throw 'frontend/package.json was not found.' }
    if ($Check) {
        Write-Host "[check] Frontend localhost:$frontendPort -> SSH $sshHost -> backend 127.0.0.1:$remotePort."
        Write-Host '[check] Preflight passed. No services were started.'
        exit 0
    }
    $env:BACKEND_URL = "http://127.0.0.1:$tunnelPort"
    $existingFrontend = $null
    if (-not $ProbeOnly -and (Test-FrontendPort $frontendPort)) {
        $existingFrontend = Get-ExistingFrontend $frontendPort $env:BACKEND_URL
        Write-Host "[frontend] Reusing this project's frontend on port $frontendPort (PID $($existingFrontend.Id))."
    }
    if (Test-FrontendPort $tunnelPort) {
        $null = Test-ExistingTunnel $tunnelPort $remotePort $sshHost
        Write-Host "[backend] Reusing SSH tunnel on port $tunnelPort. Keep its original window open."
    } else {
        # Read-only SSH check; never fall back to a local backend or another database.
        & $ssh.Source -n -o BatchMode=yes -o ConnectTimeout=10 $sshHost exit
        if ($LASTEXITCODE -ne 0) { throw "SSH failed. Configure key access first: ssh $sshHost" }
        $tunnelLog = [System.IO.Path]::GetTempFileName()
        $tunnel = Start-Process -FilePath $ssh.Source -ArgumentList @('-N', '-T', '-n', '-o', 'BatchMode=yes',
            '-o', 'ExitOnForwardFailure=yes', '-o', 'ServerAliveInterval=15', '-o', 'ServerAliveCountMax=3',
            '-L', "127.0.0.1:${tunnelPort}:127.0.0.1:${remotePort}", $sshHost) -PassThru -WindowStyle Hidden -RedirectStandardError $tunnelLog
        $deadline = [DateTime]::UtcNow.AddSeconds(12)
        while (-not (Test-FrontendPort $tunnelPort)) {
            if ($tunnel.HasExited) { throw 'SSH tunnel exited. Check the SSH host and local tunnel port.' }
            if ([DateTime]::UtcNow -ge $deadline) { throw 'SSH tunnel did not open within 12 seconds.' }
            Start-Sleep -Milliseconds 150
        }
    }
    $health = Invoke-RestMethod -Uri "$env:BACKEND_URL/api/health" -TimeoutSec 10
    if ($health.status -ne 'ok') { throw 'Ubuntu backend health check failed.' }
    try {
        $null = Invoke-WebRequest -UseBasicParsing -Uri "$env:BACKEND_URL/api/admin/session" -TimeoutSec 10
    } catch {
        $status = 0
        if ($_.Exception.Response) { $status = [int]$_.Exception.Response.StatusCode }
        if ($status -eq 404) {
            throw 'Ubuntu backend needs the admin release. On Ubuntu run: sudo bash ./docker-build.sh && sudo bash ./docker-deploy.sh'
        }
        if ($status -ne 401) { throw 'Ubuntu admin API is unavailable. Check the server backend logs.' }
    }
    Write-Host "[backend] Connected to Ubuntu through SSH $sshHost (remote port $remotePort)."
    if ($ProbeOnly) { Write-Host '[check] Health and admin API passed.'; exit 0 }
    if ($null -ne $existingFrontend) {
        $frontendHealth = Invoke-RestMethod -Uri "http://localhost:$frontendPort/api/health" -TimeoutSec 10
        if ($frontendHealth.status -ne 'ok') { throw 'Existing frontend could not reach the Ubuntu backend.' }
        Write-Host "[frontend] Ready at http://localhost:$frontendPort. Refresh the browser to continue."
        if ($null -ne $tunnel) {
            Write-Host '[backend] SSH connection restored. Keep this window open. Ctrl+C closes only this new tunnel.'
            while (-not $existingFrontend.HasExited) {
                if ($tunnel.WaitForExit(1000)) { throw 'SSH tunnel disconnected. Rerun runall.bat to restore the connection.' }
            }
        } else {
            Write-Host '[frontend] Already running. No additional frontend or tunnel was started.'
        }
        exit 0
    }
    Push-Location -LiteralPath $frontendPath
    try {
        if (-not (Test-Path -LiteralPath (Join-Path $frontendPath 'node_modules'))) {
            & $npm.Source ci
            if ($LASTEXITCODE -ne 0) { throw 'npm ci failed.' }
        }
        Write-Host "[frontend] Open http://localhost:$frontendPort and log in with the Ubuntu admin account."
        Write-Host '[frontend] This workspace uses LIVE server data. Ctrl+C stops the frontend and its SSH tunnel.'
        & $npm.Source run dev -- --hostname localhost --port $frontendPort
        $exitCode = $LASTEXITCODE
    } finally { Pop-Location }
} catch {
    Write-Host ('[ERROR] ' + $_.Exception.Message)
    $exitCode = 1
} finally {
    # Only terminate the SSH child created by this launcher.
    if ($null -ne $tunnel -and -not $tunnel.HasExited) { $tunnel.Kill(); $tunnel.WaitForExit() }
    if ($tunnelLog -and (Test-Path -LiteralPath $tunnelLog)) { Remove-Item -LiteralPath $tunnelLog -Force }
}
exit $exitCode
