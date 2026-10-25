param([switch]$Check, [switch]$RemoteBackend, [switch]$LocalBackend)

$ErrorActionPreference = 'Stop'
$repoPath = Split-Path -Parent $PSScriptRoot
$backendPath = Join-Path $repoPath 'backend'
$frontendPath = Join-Path $repoPath 'frontend'
$envPath = Join-Path $repoPath '.env'
$envLines = @()

# Read only the settings needed by the launcher. Never execute or print .env values.
# The backend loads its own .env with dotenvy; inherited environment values take precedence.
function Get-DevSetting([string]$Name, [string]$Default = '') {
    $inherited = [Environment]::GetEnvironmentVariable($Name, 'Process')
    if ($null -ne $inherited) { return $inherited }
    foreach ($line in $envLines) {
        if ($line -notmatch ('^\s*(?:export\s+)?' + [regex]::Escape($Name) + '\s*=\s*(.*)$')) { continue }
        $value = $Matches[1].Trim()
        if ($value.StartsWith('#')) { return '' }
        if ($value -match '^"((?:\\.|[^"\\])*)"\s*(?:#.*)?$') { return $Matches[1] }
        if ($value -match "^'([^']*)'\s*(?:#.*)?$") { return $Matches[1] }
        return ($value -replace '\s+#.*$', '').Trim()
    }
    return $Default
}

function Get-DevCommand([string]$Name) {
    $command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $command) { throw "$Name was not found in PATH. Install it before running runall.bat." }
    return $command.Source
}

function Test-Listening([int]$Port) {
    $listeners = [System.Net.NetworkInformation.IPGlobalProperties]::GetIPGlobalProperties().GetActiveTcpListeners()
    return [bool]($listeners | Where-Object Port -EQ $Port | Select-Object -First 1)
}

function Start-DevProcess([string]$Name, [string]$File, [string[]]$Arguments, [string]$Directory) {
    $process = Start-Process -FilePath $File -ArgumentList $Arguments -WorkingDirectory $Directory -PassThru
    Write-Host "[$Name] started (PID $($process.Id))."
    return $process
}

try {
    if (Test-Path -LiteralPath $envPath) { $envLines = Get-Content -LiteralPath $envPath -Encoding UTF8 }
    if ($RemoteBackend -and $LocalBackend) { throw 'Choose either -RemoteBackend or -LocalBackend.' }
    $backendMode = Get-DevSetting 'DEV_BACKEND_MODE' 'local'
    if ($RemoteBackend) { $backendMode = 'remote' }
    if ($LocalBackend) { $backendMode = 'local' }
    if ($backendMode -notin @('local', 'remote')) { throw 'DEV_BACKEND_MODE must be local or remote.' }
    if ($backendMode -eq 'remote') {
        & (Join-Path $PSScriptRoot 'run-frontend.ps1') -Check:$Check
        exit $LASTEXITCODE
    }
    if (-not (Test-Path -LiteralPath (Join-Path $backendPath 'Cargo.toml'))) {
        throw 'backend\Cargo.toml was not found.'
    }
    $null = Get-DevCommand 'cargo.exe'
    $hasFrontend = Test-Path -LiteralPath (Join-Path $frontendPath 'package.json')
    if ($hasFrontend) {
        $null = Get-DevCommand 'node.exe'
        $npmPath = Get-DevCommand 'npm.cmd'
    }
    $required = @('LINE_SHOP_ID', 'LINE_CHANNEL_ID', 'LINE_DESTINATION_ID',
        'LINE_CHANNEL_SECRET', 'LINE_CHANNEL_ACCESS_TOKEN', 'INQUIRY_ADMIN_TOKEN')
    $missing = @($required | Where-Object { [string]::IsNullOrWhiteSpace((Get-DevSetting $_)) })
    if ($missing.Count -gt 0) {
        Write-Host '[config] Copy .env.example to .env in the repository root and fill in the required values.'
        throw ('Missing configuration: ' + ($missing -join ', ') + '. Values may also be supplied as environment variables.')
    }
    $backendPort = 0
    if (-not [int]::TryParse((Get-DevSetting 'PORT' '8080'), [ref]$backendPort) -or $backendPort -lt 1 -or $backendPort -gt 65535) {
        throw 'PORT must be an integer between 1 and 65535.'
    }
    $mongoUri = Get-DevSetting 'MONGODB_URI' 'mongodb://localhost:27017'
    if ($mongoUri -notmatch '^mongodb(?:\+srv)?://\S+$') {
        throw 'MONGODB_URI must be a MongoDB connection URI.'
    }
    $parsedMongo = $null
    $localMongo = [Uri]::TryCreate($mongoUri, [UriKind]::Absolute, [ref]$parsedMongo) -and
        $parsedMongo.Scheme -eq 'mongodb' -and $parsedMongo.IsLoopback -and [string]::IsNullOrEmpty($parsedMongo.UserInfo)
    $mongoPort = 27017
    if ($localMongo -and $parsedMongo.Port -gt 0) { $mongoPort = $parsedMongo.Port }
    if ($localMongo -and -not (Test-Listening $mongoPort)) { $mongodPath = Get-DevCommand 'mongod.exe' }
    $backendRunning = Test-Listening $backendPort
    if ($backendRunning) {
        try {
            $health = Invoke-RestMethod -Uri "http://127.0.0.1:$backendPort/api/health" -TimeoutSec 2
            if ($health.status -ne 'ok') { throw 'Unexpected health response' }
        } catch { throw "PORT $backendPort is in use by a service without a healthy /api/health endpoint." }
    }
    Write-Host '[config] Required configuration is present.'
    if (-not $hasFrontend) { Write-Host '[frontend] No frontend/package.json yet - skipping frontend.' }
    if ($Check) {
        Write-Host '[check] Preflight passed. No services were started.'
        exit 0
    }

    # Finish package installation before opening any service windows.
    if ($hasFrontend -and -not (Test-Path -LiteralPath (Join-Path $frontendPath 'node_modules'))) {
        Push-Location -LiteralPath $frontendPath
        try {
            & $npmPath install
            if ($LASTEXITCODE -ne 0) { throw "npm install failed (exit code $LASTEXITCODE)." }
        } finally { Pop-Location }
    }

    if (-not $localMongo) {
        Write-Host '[mongo] Using the configured external or authenticated connection; no local mongod will be started.'
    } elseif (Test-Listening $mongoPort) {
        Write-Host "[mongo] Already listening on port $mongoPort - reusing it."
    } else {
        $databasePath = Join-Path $repoPath 'data\db'
        $null = New-Item -ItemType Directory -Path $databasePath -Force
        $mongoProcess = Start-DevProcess 'mongo' $mongodPath @('--dbpath', ('"{0}"' -f $databasePath),
            '--port', "$mongoPort", '--bind_ip', '127.0.0.1') $repoPath
        $deadline = [DateTime]::UtcNow.AddSeconds(20)
        while (-not (Test-Listening $mongoPort)) {
            if ($mongoProcess.HasExited) { throw "mongod exited (code $($mongoProcess.ExitCode)). Check its window for details." }
            if ([DateTime]::UtcNow -ge $deadline) { throw "MongoDB did not start on port $mongoPort within 20 seconds." }
            Start-Sleep -Milliseconds 200
        }
    }

    if ($backendRunning) {
        Write-Host "[backend] Healthy backend already listening on port $backendPort - reusing it."
    } else {
        $watch = Get-Command cargo-watch.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($watch) { $backendCommand = 'cargo watch -q -c -x run' } else { $backendCommand = 'cargo run' }
        $null = Start-DevProcess 'backend' $env:ComSpec @('/k', $backendCommand) $backendPath
    }
    if ($hasFrontend) {
        # Next.js does not read the repository root .env. Forward the actual backend port.
        $env:BACKEND_URL = "http://127.0.0.1:$backendPort"
        if (Test-Listening 3000) {
            Write-Host '[frontend] Port 3000 is already in use - skipping another frontend process.'
        } else {
            $null = Start-DevProcess 'frontend' $env:ComSpec @('/k', 'npm.cmd run dev') $frontendPath
        }
    }

    Write-Host ''
    Write-Host "Backend health : http://127.0.0.1:$backendPort/api/health"
    Write-Host "LINE webhook   : http://127.0.0.1:$backendPort/api/line/webhook"
    if ($hasFrontend) { Write-Host 'Frontend       : http://127.0.0.1:3000' }
    Write-Host 'Backend compilation may take a while on the first run. Watch its window for startup errors.'
    Write-Host 'To stop services started here, press Ctrl+C in their windows or close those windows.'
    exit 0
} catch {
    Write-Host ('[ERROR] ' + $_.Exception.Message)
    exit 1
}
