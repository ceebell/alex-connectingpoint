param([ValidateRange(1, 32)][int]$Jobs = 2)
$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $repoRoot

function Invoke-Docker {
    param([string[]]$DockerArgs)
    & $script:dockerExe @DockerArgs
    if ($LASTEXITCODE -ne 0) { throw "Local Docker command failed (exit $LASTEXITCODE)." }
}

try {
    $dockerCommand = Get-Command docker -ErrorAction SilentlyContinue
    $dockerExe = if ($dockerCommand) { $dockerCommand.Source } else {
        Join-Path $env:ProgramFiles 'Docker\Docker\resources\bin\docker.exe'
    }
    if (-not (Test-Path -LiteralPath $dockerExe)) { throw 'Install Docker Desktop locally and enable Linux containers.' }
    # Use a local named pipe explicitly; never compile through an SSH/TCP builder.
    $localContext = 'desktop-linux'
    $endpoint = (Invoke-Docker -DockerArgs @('context', 'inspect', $localContext, '--format', '{{.Endpoints.docker.Host}}') | Out-String).Trim()
    if (-not $endpoint.StartsWith('npipe://')) { throw 'The desktop-linux context must use a LOCAL Windows named pipe.' }
    $osType = (Invoke-Docker -DockerArgs @('--context', $localContext, 'info', '--format', '{{.OSType}}') | Out-String).Trim()
    if ($osType -ne 'linux') { throw 'Start Docker Desktop locally in Linux container mode.' }
    # A dedicated builder prevents accidentally reusing a remote default builder.
    $builder = 'alex-line-local'
    $builders = @(Invoke-Docker -DockerArgs @('buildx', 'ls', '--format', '{{json .}}') | ForEach-Object { $_ | ConvertFrom-Json })
    $builderInfo = $builders | Where-Object { $_.Name -eq $builder } | Select-Object -First 1
    if (-not $builderInfo) {
        Invoke-Docker -DockerArgs @('buildx', 'create', '--name', $builder, '--driver', 'docker-container', $localContext)
    } else {
        if ($builderInfo.Driver -ne 'docker-container' -or @($builderInfo.Nodes).Count -ne 1 -or $builderInfo.Nodes[0].Endpoint -ne $localContext) {
            throw 'The alex-line-local builder is not bound exclusively to the local desktop-linux context.'
        }
    }
    $stageRoot = Join-Path $repoRoot ('data\line-release-' + [Guid]::NewGuid().ToString('N'))
    $compiledDir = Join-Path $stageRoot 'compiled'
    New-Item -ItemType Directory -Path $compiledDir -Force | Out-Null
    Write-Host '[local] Compiling Linux/amd64 release on this computer.'
    Invoke-Docker -DockerArgs @('buildx', 'build', '--builder', $builder, '--platform', 'linux/amd64', '--target', 'artifact',
        '--build-arg', "CARGO_BUILD_JOBS=$Jobs", '-f', 'docker/backend.build.Dockerfile', '--output', "type=local,dest=$compiledDir", '.')
    $binaryPath = Join-Path $compiledDir 'backend'
    $header = New-Object byte[] 20
    $stream = [IO.File]::OpenRead($binaryPath)
    try { $read = $stream.Read($header, 0, $header.Length) } finally { $stream.Dispose() }
    if ($read -ne 20 -or [BitConverter]::ToString($header[0..6]) -ne '7F-45-4C-46-02-01-01' -or $header[18] -ne 62 -or $header[19] -ne 0) {
        throw 'Compiler output is not a Linux x86_64 ELF binary.'
    }
    $binaryHash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $tag = 'rel-' + [DateTime]::UtcNow.ToString('yyyyMMddHHmmss') + '-' + $binaryHash.Substring(0, 12)
    $releaseDir = Join-Path $repoRoot 'dist\line-linux'
    New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
    Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $releaseDir 'backend') -Force
    [IO.File]::WriteAllText((Join-Path $releaseDir 'backend.sha256'), "$binaryHash  backend`n", [Text.Encoding]::ASCII)
    [IO.File]::WriteAllText((Join-Path $releaseDir 'release.tag'), "$tag`n", [Text.Encoding]::ASCII)
    $manifest = @{ tag = $tag; platform = 'linux/amd64'; sha256 = $binaryHash; built_at = [DateTime]::UtcNow.ToString('o'); build_location = 'local' } | ConvertTo-Json
    [IO.File]::WriteAllText((Join-Path $releaseDir 'release.json'), $manifest, (New-Object Text.UTF8Encoding($false)))
    # Ship only compiled objects and runtime packaging files; never source or .env.
    $packageRoot = Join-Path $stageRoot 'package'
    $files = @('dist/line-linux/backend', 'dist/line-linux/backend.sha256', 'dist/line-linux/release.tag', 'dist/line-linux/release.json',
        'docker/backend.Dockerfile', 'docker/Caddyfile.line.example', 'docker-stack.line.yml', 'deploy-line.sh',
        'docker-build.sh', 'docker-deploy.sh', 'loadgit.sh', 'diagnose-line.sh', 'scripts/diagnose-line.py',
        'scripts/verify-line-release.sh', 'scripts/verify-frontend-release.sh', '.dockerignore', '.env.example', 'docs/deploy-line-server.md',
        'docs/step-1-line-inquiry.md', 'docs/build-local.md', 'alex-line-hermes-mcp-workflow.md', 'alex-line-hermes-mcp-workflow.html')
    foreach ($file in $files) {
        $destination = Join-Path $packageRoot $file
        New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $repoRoot $file) -Destination $destination
    }
    $archive = Join-Path $repoRoot "dist\line-$tag.tar.gz"
    & tar.exe -czf $archive -C $packageRoot .
    if ($LASTEXITCODE -ne 0) { throw 'Release archive creation failed.' }
    Write-Host "[OK] Linux release: $releaseDir"
    Write-Host "[OK] Upload this archive: $archive"
    Write-Host '[local] Commit and push dist/line-linux together with the matching code and scripts using rungit.bat.'
    Write-Host '[server] 1. bash ./loadgit.sh main'
    Write-Host '[server] 2. sudo bash ./docker-build.sh'
    Write-Host '[server] 3. sudo bash ./docker-deploy.sh (first deployment: add --init-swarm)'
} catch {
    Write-Host ('[ERROR] ' + $_.Exception.Message)
    exit 1
}
