param(
    [ValidateSet('x86_64-pc-windows-gnu', 'x86_64-pc-windows-msvc')]
    [string]$Target = '',
    [switch]$Installer,
    [string]$Iscc = '',
    [switch]$SkipBuild,
    [switch]$Test
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path -Parent $PSScriptRoot
$env:Path = "$(Join-Path $env:USERPROFILE '.cargo\bin');$env:Path"
$gnuTools = Join-Path $repo '.tools\w64devkit\bin'
if (Test-Path -LiteralPath $gnuTools) { $env:Path = "$gnuTools;$env:Path" }
Push-Location $repo
try {
    $hostLine = (& rustc -vV | Where-Object { $_ -like 'host: *' })
    if ($LASTEXITCODE) { throw 'rustc -vV failed' }
    $hostTarget = $hostLine.Substring(6).Trim()
    if (!$Target) {
        $Target = $hostTarget
        if ($Target -notin @('x86_64-pc-windows-gnu', 'x86_64-pc-windows-msvc')) {
            throw "Unsupported Windows target: $Target"
        }
    }
    # Keep native host build scripts under the same rustflags as the executable.
    $targetArgs = @()
    $binaryRelative = 'release\smithcraft.exe'
    if ($Target -ne $hostTarget) {
        $targetArgs = @('--target', $Target)
        $binaryRelative = "$Target\release\smithcraft.exe"
    }
    if ($Test) {
        & cargo test --locked @targetArgs
        if ($LASTEXITCODE) { throw 'Tests failed' }
    }
    if (!$SkipBuild) {
        & cargo build --release --locked @targetArgs
        if ($LASTEXITCODE) { throw 'Release build failed' }
    }
    $metadataText = & cargo metadata --no-deps --format-version 1 --locked
    if ($LASTEXITCODE) { throw 'cargo metadata failed' }
    $metadata = $metadataText | ConvertFrom-Json
    $package = $metadata.packages | Where-Object name -eq 'smithcraft'
    $version = $package.version
    $binary = Join-Path $metadata.target_directory $binaryRelative
    if (!(Test-Path -LiteralPath $binary)) { throw "Missing binary: $binary" }
    $artifacts = Join-Path $repo 'dist\artifacts'
    $stage = Join-Path $repo ('.tools\packages\windows-' + [guid]::NewGuid().ToString('N'))
    $installed = Join-Path $stage 'installed'
    $portableName = "SmithCraft-$version-windows-x86_64-portable"
    $portable = Join-Path $stage $portableName
    New-Item -ItemType Directory -Path $artifacts,$installed,$portable -Force | Out-Null
    Copy-Item -LiteralPath $binary -Destination (Join-Path $installed 'SmithCraft.exe')
    Copy-Item -LiteralPath README.md,BUILDING.md,LICENSE -Destination $installed
    Copy-Item -LiteralPath examples,help -Destination $installed -Recurse
    # MSVC release builds carry their CRT; GNU builds use the bundled linker.
    # Audit imported DLLs when changing toolchains or adding native dependencies.
    Copy-Item -Path (Join-Path $installed '*') -Destination $portable -Recurse
    Set-Content -LiteralPath (Join-Path $portable 'portable.txt') -Encoding utf8 -Value 'Settings are stored in data/state.ron next to this executable.'
    $archive = Join-Path $artifacts "$portableName.zip"
    Compress-Archive -LiteralPath $portable -DestinationPath $archive -Force
    $outputs = @($archive)
    if ($Installer) {
        $localCompiler = Join-Path $repo '.tools\inno\ISCC.exe'
        if (!$Iscc -and (Test-Path -LiteralPath $localCompiler)) { $Iscc = $localCompiler }
        if (!$Iscc) {
            $compiler = Get-Command ISCC.exe -ErrorAction SilentlyContinue
            if ($compiler) { $Iscc = $compiler.Source }
        }
        if (!$Iscc) {
            foreach ($root in @(${env:ProgramFiles(x86)}, $env:ProgramFiles, $env:LOCALAPPDATA)) {
                if (!$root) { continue }
                $candidate = Join-Path $root 'Inno Setup 6\ISCC.exe'
                if (Test-Path -LiteralPath $candidate) { $Iscc = $candidate; break }
            }
        }
        if (!$Iscc) { throw 'Run tools/install-inno.ps1 or pass -Iscc <path-to-ISCC.exe>. The portable ZIP was built successfully.' }
        & $Iscc '/Qp' "/DAppVersion=$version" "/DSourceDir=$installed" "/DArtifactDir=$artifacts" (Join-Path $repo 'packaging\windows\SmithCraft.iss')
        if ($LASTEXITCODE) { throw 'Installer compilation failed' }
        $outputs += Join-Path $artifacts "SmithCraft-$version-windows-x86_64-setup.exe"
    }
    foreach ($file in $outputs) {
        $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
        Set-Content -LiteralPath "$file.sha256" -Encoding ascii -Value "$hash  $(Split-Path -Leaf $file)"
        Write-Host $file
    }
} finally { Pop-Location }
