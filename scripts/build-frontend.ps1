param([string]$BackendUrl = 'http://backend:8080')
$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $repoRoot

function Invoke-Docker {
    param([string[]]$DockerArgs)
    & $script:dockerExe @DockerArgs
    if ($LASTEXITCODE -ne 0) { throw "Local Docker command failed (exit $LASTEXITCODE)." }
}

function Assert-WorkspacePath {
    param([string]$Path, [string]$Parent)
    $full = [IO.Path]::GetFullPath($Path)
    $prefix = [IO.Path]::GetFullPath($Parent).TrimEnd('\') + '\'
    if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Release path must stay inside the designated workspace directory.'
    }
    if ((Test-Path -LiteralPath $full) -and ((Get-Item -LiteralPath $full -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'Release paths must not be symbolic links or junctions.'
    }
}

try {
    if (-not (Test-Path -LiteralPath 'frontend/package.json')) {
        throw 'frontend/package.json is missing. Add the frontend application before building its release.'
    }
    if (-not (Test-Path -LiteralPath 'frontend/package-lock.json')) {
        throw 'frontend/package-lock.json is missing. Run npm install in frontend on the LOCAL computer first.'
    }
    $package = Get-Content -LiteralPath 'frontend/package.json' -Raw | ConvertFrom-Json
    if (-not $package.scripts.build) { throw 'frontend/package.json must define a build script.' }
    $backendUri = $null
    if (-not [Uri]::TryCreate($BackendUrl, [UriKind]::Absolute, [ref]$backendUri) -or
        $backendUri.Scheme -notin @('http', 'https') -or $backendUri.UserInfo -or $backendUri.Query -or $backendUri.Fragment) {
        throw 'BackendUrl must be an absolute HTTP(S) base URL without credentials, query parameters or fragments.'
    }
    $dockerCommand = Get-Command docker -ErrorAction SilentlyContinue
    $dockerExe = if ($dockerCommand) { $dockerCommand.Source } else {
        Join-Path $env:ProgramFiles 'Docker\Docker\resources\bin\docker.exe'
    }
    if (-not (Test-Path -LiteralPath $dockerExe)) { throw 'Install Docker Desktop locally and enable Linux containers.' }
    $localContext = 'desktop-linux'
    $endpoint = (Invoke-Docker -DockerArgs @('context', 'inspect', $localContext, '--format', '{{.Endpoints.docker.Host}}') | Out-String).Trim()
    if (-not $endpoint.StartsWith('npipe://')) { throw 'The desktop-linux context must use a LOCAL Windows named pipe.' }
    $osType = (Invoke-Docker -DockerArgs @('--context', $localContext, 'info', '--format', '{{.OSType}}') | Out-String).Trim()
    if ($osType -ne 'linux') { throw 'Start Docker Desktop locally in Linux container mode.' }
    $builder = 'alex-line-local'
    $builders = @(Invoke-Docker -DockerArgs @('buildx', 'ls', '--format', '{{json .}}') | ForEach-Object { $_ | ConvertFrom-Json })
    $builderInfo = $builders | Where-Object { $_.Name -eq $builder } | Select-Object -First 1
    if (-not $builderInfo) {
        Invoke-Docker -DockerArgs @('buildx', 'create', '--name', $builder, '--driver', 'docker-container', $localContext)
    } elseif ($builderInfo.Driver -ne 'docker-container' -or @($builderInfo.Nodes).Count -ne 1 -or $builderInfo.Nodes[0].Endpoint -ne $localContext) {
        throw 'The release builder must be bound exclusively to the local desktop-linux context.'
    }
    $stageRoot = Join-Path $repoRoot ('data\front-' + [Guid]::NewGuid().ToString('N').Substring(0, 12))
    $compiledDir = Join-Path $stageRoot 'compiled'
    New-Item -ItemType Directory -Path $compiledDir -Force | Out-Null
    Write-Host '[local] Building Linux/amd64 Next.js standalone release on this computer.'
    Invoke-Docker -DockerArgs @('buildx', 'build', '--builder', $builder, '--platform', 'linux/amd64', '--target', 'artifact',
        '-f', 'docker/frontend.build.Dockerfile', '--build-arg', "BACKEND_URL=$BackendUrl", '--output', "type=local,dest=$compiledDir", '.')
    if (-not (Test-Path -LiteralPath (Join-Path $compiledDir 'server.js'))) {
        throw 'Standalone server.js was not produced. Set output: standalone in next.config.ts.'
    }
    $files = @(Get-ChildItem -LiteralPath $compiledDir -Recurse -Force -File | Sort-Object FullName)
    if ($files | Where-Object { $_.Name -eq '.env' -or $_.Name.StartsWith('.env.') }) {
        throw 'Unexpected .env file found in compiled output. Release was not published.'
    }
    $hashLines = @($files | ForEach-Object {
        $relative = $_.FullName.Substring($compiledDir.Length + 1).Replace('\', '/')
        (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() + '  ' + $relative
    })
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $contentHash = -join ($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes(($hashLines -join "`n"))) | ForEach-Object { $_.ToString('x2') }) }
    finally { $sha.Dispose() }
    $tag = 'front-' + [DateTime]::UtcNow.ToString('yyyyMMddHHmmss') + '-' + $contentHash.Substring(0, 12)
    $utf8 = New-Object Text.UTF8Encoding($false)
    $manifest = @{ tag = $tag; platform = 'linux/amd64'; built_at = [DateTime]::UtcNow.ToString('o'); build_location = 'local'; backend_url = $BackendUrl; content_sha256 = $contentHash } | ConvertTo-Json
    [IO.File]::WriteAllText((Join-Path $compiledDir 'frontend-release.json'), $manifest.Replace("`r`n", "`n") + "`n", $utf8)
    [IO.File]::WriteAllText((Join-Path $compiledDir 'frontend.platform'), "linux/amd64`n", [Text.Encoding]::ASCII)
    [IO.File]::WriteAllText((Join-Path $compiledDir 'frontend.tag'), $tag + "`n", [Text.Encoding]::ASCII)
    $checksums = @(Get-ChildItem -LiteralPath $compiledDir -Recurse -Force -File | Sort-Object FullName | ForEach-Object {
        (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() + '  ' + $_.FullName.Substring($compiledDir.Length + 1).Replace('\', '/')
    })
    [IO.File]::WriteAllText((Join-Path $compiledDir 'frontend.sha256'), ($checksums -join "`n") + "`n", $utf8)
    $distDir = Join-Path $repoRoot 'dist'
    $releaseDir = Join-Path $distDir 'frontend-linux'
    $backupDir = Join-Path $repoRoot ('data\front-backup-' + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
    # Check every absolute source/destination before moving directory trees.
    Assert-WorkspacePath $compiledDir (Join-Path $repoRoot 'data')
    Assert-WorkspacePath $releaseDir $distDir
    Assert-WorkspacePath $backupDir (Join-Path $repoRoot 'data')
    foreach ($parentDir in @($distDir, (Join-Path $repoRoot 'data'))) {
        if ((Get-Item -LiteralPath $parentDir -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw 'Release parent directories must not be symbolic links or junctions.'
        }
    }
    $hadRelease = Test-Path -LiteralPath $releaseDir
    if ($hadRelease) { Move-Item -LiteralPath $releaseDir -Destination $backupDir }
    try { Move-Item -LiteralPath $compiledDir -Destination $releaseDir }
    catch {
        if ($hadRelease -and -not (Test-Path -LiteralPath $releaseDir)) {
            Move-Item -LiteralPath $backupDir -Destination $releaseDir
        }
        throw
    }
    Write-Host "[OK] Frontend Linux release: $releaseDir"
    Write-Host "[OK] Release tag: $tag"
    Write-Host '[local] Commit and push dist/frontend-linux with the matching backend release.'
    Write-Host '[server] Pull Git, then run: sudo bash ./docker-build.sh --full-stack'
} catch {
    Write-Host ('[ERROR] ' + $_.Exception.Message)
    exit 1
}
