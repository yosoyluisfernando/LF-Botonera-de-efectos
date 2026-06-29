param(
    [Parameter(Mandatory)][string]$Version
)

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Formato invalido. Usa x.y.z  (ejemplo: 1.2.0)"
    exit 1
}

$root = $PSScriptRoot
Set-Location $root

Write-Host ""
Write-Host "Actualizando a version $Version ..." -ForegroundColor Cyan
Write-Host ""

# 1. package.json + package-lock.json
Write-Host "[1/3] package.json y package-lock.json ..."
$out = npm version $Version --no-git-tag-version --allow-same-version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "npm version fallo: $out"
    exit 1
}

# 2. Cargo.toml - solo primera ocurrencia (seccion [package])
Write-Host "[2/3] src-tauri\Cargo.toml ..."
$cargoPath = Join-Path $root "src-tauri\Cargo.toml"
$cargo = Get-Content $cargoPath -Raw
$cargo = [regex]::Replace(
    $cargo,
    '(\[package\][\s\S]*?version = )"\d+\.\d+\.\d+"',
    "`${1}`"$Version`"",
    1
)
[System.IO.File]::WriteAllText($cargoPath, $cargo, [System.Text.UTF8Encoding]::new($false))

# 3. tauri.conf.json
Write-Host "[3/3] src-tauri\tauri.conf.json ..."
$tauriPath = Join-Path $root "src-tauri\tauri.conf.json"
$tauri = Get-Content $tauriPath -Raw
$tauri = $tauri -replace '"version":\s*"\d+\.\d+\.\d+"', "`"version`": `"$Version`""
[System.IO.File]::WriteAllText($tauriPath, $tauri, [System.Text.UTF8Encoding]::new($false))

Write-Host ""
Write-Host "Listo. Version $Version aplicada en los 3 archivos." -ForegroundColor Green
Write-Host ""
Write-Host "Pasos siguientes:"
Write-Host "  1. Regenerar Cargo.lock:"
Write-Host "       cd src-tauri"
Write-Host "       cargo check"
Write-Host "  2. Actualizar CHANGELOG.md con los cambios de esta version."
Write-Host "  3. Hacer commit:  git commit -am 'Release $Version'"
Write-Host "  4. Crear tag:     git tag v$Version"
Write-Host "  5. Publicar:      git push && git push --tags"
Write-Host ""
