[CmdletBinding()]
param(
    [ValidateRange(0, 65535)]
    [int]$Revision = 0,
    [string]$MakeAppxPath
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$previousChannel = [Environment]::GetEnvironmentVariable('LF_DISTRIBUTION_CHANNEL', 'Process')

try {
    $env:LF_DISTRIBUTION_CHANNEL = 'store'
    Push-Location $projectRoot
    try {
        & npm.cmd run tauri -- build --no-bundle
        if ($LASTEXITCODE -ne 0) {
            throw "La compilacion Release para Store termino con codigo $LASTEXITCODE."
        }
    } finally {
        Pop-Location
    }
} finally {
    if ($null -eq $previousChannel) {
        Remove-Item Env:LF_DISTRIBUTION_CHANNEL -ErrorAction SilentlyContinue
    } else {
        $env:LF_DISTRIBUTION_CHANNEL = $previousChannel
    }
}

$buildParams = @{
    IdentityName = 'LuisFernandoVelasquez.LFBotoneradeEfectos'
    Publisher = 'CN=AD90DE58-447F-47AE-AC1A-3D369955282B'
    PublisherDisplayName = 'Luis Fernando Velasquez'
    Revision = $Revision
}
if ($MakeAppxPath) { $buildParams.MakeAppxPath = $MakeAppxPath }

& (Join-Path $PSScriptRoot 'build-msix.ps1') @buildParams
