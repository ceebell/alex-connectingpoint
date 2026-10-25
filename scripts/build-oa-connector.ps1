$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $repoRoot
function Invoke-LocalDocker([string[]]$DockerArgs) {
    & docker.exe @DockerArgs
    if ($LASTEXITCODE -ne 0) { throw "Local Docker command failed ($LASTEXITCODE)." }
}
function Assert-InRepo([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    if (-not $full.StartsWith($repoRoot.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Path outside workspace.' }
    if ((Test-Path -LiteralPath $full) -and ((Get-Item -LiteralPath $full -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Release path must not be a link.' }
}
try {
    $context = 'desktop-linux'
    $endpoint = (Invoke-LocalDocker -DockerArgs @('context','inspect',$context,'--format','{{.Endpoints.docker.Host}}') | Out-String).Trim()
    if (-not $endpoint.StartsWith('npipe://')) { throw 'Build requires local Docker Desktop through a named pipe.' }
    if ((Invoke-LocalDocker -DockerArgs @('--context',$context,'info','--format','{{.OSType}}') | Out-String).Trim() -ne 'linux') { throw 'Switch Docker Desktop to Linux containers.' }
    $builder = 'alex-line-local'
    $builders = @(Invoke-LocalDocker -DockerArgs @('buildx','ls','--format','{{json .}}') | ForEach-Object { $_ | ConvertFrom-Json })
    $info = $builders | Where-Object Name -EQ $builder | Select-Object -First 1
    if (-not $info) { Invoke-LocalDocker -DockerArgs @('buildx','create','--name',$builder,'--driver','docker-container',$context) }
    elseif ($info.Driver -ne 'docker-container' -or @($info.Nodes).Count -ne 1 -or $info.Nodes[0].Endpoint -ne $context) { throw 'Builder must be bound exclusively to local desktop-linux.' }
    $stage = Join-Path $repoRoot ('data\oa-release-' + [Guid]::NewGuid().ToString('N'))
    $compiled = Join-Path $stage 'compiled'
    $null = New-Item -ItemType Directory -Path $compiled -Force
    Invoke-LocalDocker -DockerArgs @('buildx','build','--builder',$builder,'--platform','linux/amd64','--target','artifact','-f','docker/oa-connector.build.Dockerfile','--output',("type=local,dest=$compiled"),'.')
    $lines = @(Get-ChildItem -LiteralPath $compiled -Recurse -File | Sort-Object FullName | ForEach-Object {
        $relative = $_.FullName.Substring($compiled.Length + 1).Replace('\','/')
        (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() + '  ' + $relative
    })
    $utf8 = New-Object Text.UTF8Encoding($false)
    $manifestPath = Join-Path $compiled 'connector.sha256'
    [IO.File]::WriteAllText($manifestPath,($lines -join "`n") + "`n",$utf8)
    $hash = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $tag = 'oa-' + [DateTime]::UtcNow.ToString('yyyyMMddHHmmss') + '-' + $hash.Substring(0,12)
    [IO.File]::WriteAllText((Join-Path $compiled 'connector.tag'),$tag + "`n",$utf8)
    $release = Join-Path $repoRoot 'dist\oa-connector-linux'
    Assert-InRepo $compiled; Assert-InRepo $release
    if (Test-Path -LiteralPath $release) { $backup = Join-Path $stage 'previous'; Assert-InRepo $backup; Move-Item -LiteralPath $release -Destination $backup }
    Move-Item -LiteralPath $compiled -Destination $release
    Write-Host "[OK] Local Linux connector release: $tag"
    Write-Host 'Commit with rungit.bat when ready. Ubuntu only packages this compiled release.'
} catch { Write-Host ('[ERROR] ' + $_.Exception.Message); exit 1 }
