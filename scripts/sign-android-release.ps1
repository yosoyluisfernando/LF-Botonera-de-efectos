param(
    [string]$Source,
    [string]$Destination,
    [string]$KeyDirectory = 'D:/BOTONERA/Claves/Android/produccion',
    [string]$BackupDirectory = 'D:/BOTONERA/Respaldos/Firma-Android-Estable',
    [string]$RuntimeDirectory = 'D:/BOTONERA/app/.tools/android-runtime',
    [switch]$Initialize
)
$ErrorActionPreference = 'Stop'
$java = (Get-ChildItem "$RuntimeDirectory/jdk/*/bin/java.exe" | Select-Object -First 1).FullName
if (!$java) { throw 'No se encontró el JDK del entorno Android.' }
$keytool = Join-Path (Split-Path -Parent $java) 'keytool.exe'
$signer = Join-Path $RuntimeDirectory 'sdk/build-tools/36.0.0/lib/apksigner.jar'
$key = Join-Path $KeyDirectory 'lf-botonera-android-produccion.p12'
$protected = Join-Path $KeyDirectory 'password.dpapi'
$alias = 'lf-botonera-android'

function Protect-KeyFolder([string]$Path) {
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
    $acl = [Security.AccessControl.DirectorySecurity]::new()
    $acl.SetAccessRuleProtection($true, $false)
    $owner = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $acl.SetOwner($owner)
    foreach ($sid in @($owner, [Security.Principal.SecurityIdentifier]::new('S-1-5-18'))) {
        $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new(
            $sid, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow'))
    }
    Set-Acl -LiteralPath $Path -AclObject $acl
}

try {
    if ((Test-Path -LiteralPath $key) -ne (Test-Path -LiteralPath $protected)) {
        throw 'Falta una parte de la firma. No se generará una clave diferente.'
    }
    if (!(Test-Path -LiteralPath $key)) {
        if (!$Initialize) { throw 'Inicializa la firma con -Initialize antes de firmar.' }
        if (Test-Path -LiteralPath (Join-Path $BackupDirectory 'lf-botonera-android-produccion.p12')) {
            throw 'Ya existe un respaldo de firma. Recupéralo antes de crear otra clave.'
        }
        Protect-KeyFolder $KeyDirectory
        Protect-KeyFolder $BackupDirectory
        $env:LF_ANDROID_RELEASE_PASSWORD = [Convert]::ToHexString(
            [Security.Cryptography.RandomNumberGenerator]::GetBytes(32))
        & $keytool -genkeypair -storetype PKCS12 -keystore $key `
            -storepass:env LF_ANDROID_RELEASE_PASSWORD -alias $alias `
            -keyalg RSA -keysize 4096 -sigalg SHA256withRSA -validity 10950 `
            -dname 'CN=Luis Fernando Velasquez, O=LF Botonera de Efectos'
        if ($LASTEXITCODE) { throw 'No se pudo crear la firma de producción.' }
        ConvertTo-SecureString $env:LF_ANDROID_RELEASE_PASSWORD -AsPlainText -Force |
            ConvertFrom-SecureString | Set-Content -LiteralPath $protected -Encoding ascii
        # La recuperación portátil requiere la contraseña del PKCS12, no solo DPAPI.
        Set-Content -LiteralPath (Join-Path $KeyDirectory 'RECUPERACION-PRIVADA.txt') `
            -Value $env:LF_ANDROID_RELEASE_PASSWORD -Encoding ascii
        foreach ($name in @('lf-botonera-android-produccion.p12', 'password.dpapi', 'RECUPERACION-PRIVADA.txt')) {
            Copy-Item -LiteralPath (Join-Path $KeyDirectory $name) -Destination $BackupDirectory
        }
        $instructions = @'
Firma privada de producción Android de LF Botonera de Efectos.
El PKCS12 está cifrado. RECUPERACION-PRIVADA.txt contiene su contraseña.
Ambas carpetas tienen permisos restringidos al usuario actual y SYSTEM.
DPAPI permite firmar en esta cuenta de Windows sin mostrar la contraseña.
Para recuperar en otro equipo necesitas el PKCS12 y la contraseña de recuperación.
Guarda la contraseña en tu gestor de contraseñas y una copia del PKCS12 fuera de este equipo,
en ubicaciones separadas. No publiques estos archivos ni los subas a GitHub.
No regeneres la clave para las siguientes versiones de este mismo paquete Android.
El archivo APK y el certificado público sí se pueden distribuir.
'@
        Set-Content -LiteralPath (Join-Path $KeyDirectory 'LEEME.txt') -Value $instructions -Encoding utf8
        Set-Content -LiteralPath (Join-Path $BackupDirectory 'LEEME.txt') -Value $instructions -Encoding utf8
    } else {
        $secure = (Get-Content -LiteralPath $protected -Raw).Trim() | ConvertTo-SecureString
        $env:LF_ANDROID_RELEASE_PASSWORD = [Net.NetworkCredential]::new('', $secure).Password
    }
    if ($Source) {
        if (!$Destination) { throw 'Indica Destination para el APK firmado.' }
        if (Test-Path -LiteralPath $Destination) { throw 'El APK de destino ya existe.' }
        & $java -jar $signer sign --v4-signing-enabled false --ks $key `
            --ks-key-alias $alias --ks-pass env:LF_ANDROID_RELEASE_PASSWORD --out $Destination $Source
        if ($LASTEXITCODE) { throw 'Falló la firma del APK.' }
        & $java -jar $signer verify --verbose --print-certs $Destination
        if ($LASTEXITCODE) { throw 'Falló la comprobación del APK firmado.' }
    }
    & $keytool -exportcert -rfc -keystore $key -storepass:env LF_ANDROID_RELEASE_PASSWORD `
        -alias $alias -file (Join-Path $KeyDirectory 'certificado-publico.pem')
    if ($LASTEXITCODE) { throw 'No se pudo exportar el certificado público.' }
    Write-Output "Firma disponible en $KeyDirectory"
} finally {
    Remove-Item Env:LF_ANDROID_RELEASE_PASSWORD -ErrorAction SilentlyContinue
}
