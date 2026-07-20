[CmdletBinding()]
param(
    [string]$IdentityName = 'LF.Botonera.Efectos.Local',
    [string]$Publisher = 'CN=Luis Fernando Velasquez',
    [string]$PublisherDisplayName = 'Luis Fernando Velásquez',
    [ValidateRange(0, 65535)]
    [int]$Revision = 0,
    [string]$MakeAppxPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$projectRoot = Split-Path -Parent $PSScriptRoot
$targetRoot = Join-Path $projectRoot 'src-tauri\target\msix'
$stageRoot = Join-Path $targetRoot 'staging'
$configPath = Join-Path $projectRoot 'src-tauri\tauri.conf.json'
$templatePath = Join-Path $projectRoot 'packaging\windows\msix\AppxManifest.template.xml'
$executablePath = Join-Path $projectRoot 'src-tauri\target\release\tauri-app.exe'

if (-not (Test-Path -LiteralPath $executablePath -PathType Leaf)) {
    throw 'Falta el ejecutable Release. Ejecute: npm run tauri -- build --no-bundle'
}

$config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
$versionParts = @([string]$config.version -split '\.')
if ($versionParts.Count -eq 3) { $versionParts += $Revision.ToString() }
elseif ($Revision -ne 0) { throw 'Revision solo se admite si la versión base tiene tres partes.' }
if ($versionParts.Count -ne 4 -or ($versionParts | Where-Object { $_ -notmatch '^\d+$' })) {
    throw "La versión '$($config.version)' no se puede convertir al formato MSIX A.B.C.D."
}
$msixVersion = $versionParts -join '.'

if (-not $MakeAppxPath) {
    $sdkBin = 'C:\Program Files (x86)\Windows Kits\10\bin'
    $MakeAppxPath = Get-ChildItem -LiteralPath $sdkBin -Recurse -Filter makeappx.exe -File |
        Where-Object { $_.DirectoryName -like '*\x64' } |
        Sort-Object FullName -Descending |
        Select-Object -First 1 -ExpandProperty FullName
}
if (-not $MakeAppxPath -or -not (Test-Path -LiteralPath $MakeAppxPath -PathType Leaf)) {
    throw 'No se encontró MakeAppx.exe en el Windows SDK.'
}

$resolvedTarget = [IO.Path]::GetFullPath($targetRoot)
$resolvedStage = [IO.Path]::GetFullPath($stageRoot)
if (-not $resolvedStage.StartsWith($resolvedTarget + [IO.Path]::DirectorySeparatorChar)) {
    throw 'La carpeta temporal quedó fuera de src-tauri\target\msix.'
}
if (Test-Path -LiteralPath $resolvedStage) {
    Remove-Item -LiteralPath $resolvedStage -Recurse -Force
}

$null = New-Item -ItemType Directory -Path (Join-Path $resolvedStage 'app') -Force
$null = New-Item -ItemType Directory -Path (Join-Path $resolvedStage 'Assets') -Force
$null = New-Item -ItemType Directory -Path (Join-Path $resolvedStage 'legal') -Force
Copy-Item -LiteralPath $executablePath -Destination (Join-Path $resolvedStage 'app\tauri-app.exe')

$assets = 'StoreLogo.png', 'Square150x150Logo.png', 'Square44x44Logo.png'
foreach ($asset in $assets) {
    Copy-Item -LiteralPath (Join-Path $projectRoot "src-tauri\icons\$asset") `
        -Destination (Join-Path $resolvedStage "Assets\$asset")
}

$legalFiles = @(
    'LICENSE', 'PRIVACY.md', 'SUPPORT.md', 'THIRD_PARTY_NOTICES.md',
    'THIRD_PARTY_LICENSES_RUST.html', 'THIRD_PARTY_LICENSES_NODE.html'
)
foreach ($legalFile in $legalFiles) {
    Copy-Item -LiteralPath (Join-Path $projectRoot $legalFile) `
        -Destination (Join-Path $resolvedStage "legal\$legalFile")
}

function ConvertTo-XmlText([string]$Value) {
    return [Security.SecurityElement]::Escape($Value)
}

$manifest = Get-Content -LiteralPath $templatePath -Raw
$manifest = $manifest.Replace('{{IDENTITY_NAME}}', (ConvertTo-XmlText $IdentityName))
$manifest = $manifest.Replace('{{PUBLISHER}}', (ConvertTo-XmlText $Publisher))
$manifest = $manifest.Replace('{{PUBLISHER_DISPLAY_NAME}}', (ConvertTo-XmlText $PublisherDisplayName))
$manifest = $manifest.Replace('{{VERSION}}', $msixVersion)
$manifestPath = Join-Path $resolvedStage 'AppxManifest.xml'
[IO.File]::WriteAllText($manifestPath, $manifest, [Text.UTF8Encoding]::new($false))

$outputName = "LF-Botonera-$msixVersion-x64-unsigned.msix"
$outputPath = Join-Path $resolvedTarget $outputName
if (Test-Path -LiteralPath $outputPath) {
    Remove-Item -LiteralPath $outputPath -Force
}
& $MakeAppxPath pack /o /d $resolvedStage /p $outputPath
if ($LASTEXITCODE -ne 0) { throw "MakeAppx terminó con código $LASTEXITCODE." }

$hash = (Get-FileHash -LiteralPath $outputPath -Algorithm SHA256).Hash
Write-Host "MSIX sin firma: $outputPath"
Write-Host "Identidad provisional: $IdentityName"
Write-Host "Versión MSIX: $msixVersion"
Write-Host "SHA-256: $hash"
