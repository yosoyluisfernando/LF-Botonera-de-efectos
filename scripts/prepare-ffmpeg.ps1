# Prepara un FFmpeg local para DEV.bat. No descarga ni publica ejecutables.
param(
    [Parameter(Mandatory=$true)][string]$SourcePath,
    [Parameter(Mandatory=$true)][string]$LicensePath,
    [string]$ExpectedSha256
)
$ErrorActionPreference = 'Stop'
$source = (Resolve-Path -LiteralPath $SourcePath).Path
$license = (Resolve-Path -LiteralPath $LicensePath).Path
$actual = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
if ($ExpectedSha256 -and $ExpectedSha256 -ine $actual) { throw 'SHA-256 no coincide' }
$folder = Join-Path $PSScriptRoot '../src-tauri/binaries'
New-Item -ItemType Directory -Path $folder -Force | Out-Null
Copy-Item -LiteralPath $source -Destination (Join-Path $folder 'ffmpeg.exe')
Copy-Item -LiteralPath $license -Destination (Join-Path $folder 'FFMPEG_LICENSE.txt')
Write-Host "FFmpeg local preparado. SHA-256: $actual"
