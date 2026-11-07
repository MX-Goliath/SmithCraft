# Install the release compiler into the ignored workspace tools directory.
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $repo '.tools\inno'
$downloads = Join-Path $repo '.tools\downloads'
$compiler = Join-Path $destination 'ISCC.exe'
if (Test-Path -LiteralPath $compiler) { Write-Host $compiler; return }
New-Item -ItemType Directory -Path $destination,$downloads -Force | Out-Null
$setup = Join-Path $downloads 'innosetup-6.7.4.exe'
$expected = 'a549929319f9120716677c685535ca4671e4b937b5fdc87e2585f32d8eb30e8c'
if (!(Test-Path -LiteralPath $setup) -or (Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
    Invoke-WebRequest -Uri 'https://github.com/jrsoftware/issrc/releases/download/is-6_7_4/innosetup-6.7.4.exe' -OutFile $setup
}
if ((Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
    throw 'Inno Setup download failed SHA-256 verification'
}
$arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-', '/CURRENTUSER', '/NOICONS', '/TASKS=""', ('/DIR="{0}"' -f $destination))
$process = Start-Process -FilePath $setup -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "Inno Setup installation failed: $($process.ExitCode)" }
if (!(Test-Path -LiteralPath $compiler)) { throw "Inno Setup compiler missing: $compiler" }
Write-Host $compiler
