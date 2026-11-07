param([switch]$Release, [switch]$Test, [switch]$Run, [switch]$Demo)
$ErrorActionPreference = 'Stop'
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
$portableBin = Join-Path $PSScriptRoot '.tools\w64devkit\bin'
if (Test-Path -LiteralPath $portableBin) { $env:Path = "$portableBin;$env:Path" }
if (Test-Path -LiteralPath $cargoBin) { $env:Path = "$cargoBin;$env:Path" }
Push-Location $PSScriptRoot
try {
    if ($Test) { & cargo test --locked; if ($LASTEXITCODE) { exit $LASTEXITCODE } }
    $buildArgs = @('build', '--locked')
    if ($Release) { $buildArgs += '--release' }
    & cargo @buildArgs
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
    if ($Release) {
        New-Item -ItemType Directory -Path (Join-Path $PSScriptRoot 'dist') -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'target\release\smithcraft.exe') -Destination (Join-Path $PSScriptRoot 'dist\SmithCraft.exe') -Force
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'README.md'), (Join-Path $PSScriptRoot 'LICENSE') -Destination (Join-Path $PSScriptRoot 'dist') -Force
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'examples') -Destination (Join-Path $PSScriptRoot 'dist') -Recurse -Force
    }
    if ($Run) {
        $profile = if ($Release) { 'release' } else { 'debug' }
        $exe = Join-Path $PSScriptRoot "target\$profile\smithcraft.exe"
        if ($Demo) { & $exe --demo } else { & $exe }
    }
} finally { Pop-Location }
