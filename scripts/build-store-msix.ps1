[CmdletBinding()]
param(
    [ValidateRange(0, 65535)]
    [int]$Revision = 0,
    [string]$MakeAppxPath
)

$ErrorActionPreference = 'Stop'
$buildParams = @{
    IdentityName = 'LuisFernandoVelasquez.LFBotoneradeEfectos'
    Publisher = 'CN=AD90DE58-447F-47AE-AC1A-3D369955282B'
    PublisherDisplayName = 'Luis Fernando Velasquez'
    Revision = $Revision
}
if ($MakeAppxPath) { $buildParams.MakeAppxPath = $MakeAppxPath }

& (Join-Path $PSScriptRoot 'build-msix.ps1') @buildParams
