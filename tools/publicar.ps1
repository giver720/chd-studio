<#
    Prepara una version completa de CHD Studio:

      1. Descarga las herramientas nativas que van dentro del instalador
      2. Compila el instalador (setup.exe) firmado para el actualizador
      3. Arma la version portable en un .zip
      4. Genera latest.json, que es lo que lee el actualizador

    Requiere la clave privada de firma en %USERPROFILE%\.tauri\chd-studio.key

    Uso:  npm run release
#>

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$key = "$env:USERPROFILE\.tauri\chd-studio.key"
if (-not (Test-Path $key)) {
    throw "Falta la clave de firma en $key. Generala con:`n  npx tauri signer generate -w `"$key`" --password `"`""
}

$conf = Get-Content "src-tauri\tauri.conf.json" -Raw | ConvertFrom-Json
$version = $conf.version
Write-Host "  Preparando CHD Studio $version" -ForegroundColor Cyan

# --- 1. Herramientas -------------------------------------------------------
if (-not (Test-Path "src-tauri\binaries\chdman.exe")) {
    Write-Host "  Falta chdman, obteniendolo..." -ForegroundColor Yellow
    & "$PSScriptRoot\obtener-chdman.ps1"
}
Write-Host "  Actualizando herramientas nativas..." -ForegroundColor Cyan
& "$PSScriptRoot\obtener-herramientas.ps1" | Out-Null

# --- 2. Instalador ---------------------------------------------------------
Write-Host "  Compilando (esto tarda)..." -ForegroundColor Cyan
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = $key
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run dist
if ($LASTEXITCODE -ne 0) { throw "Fallo la compilacion" }

$bundle = "src-tauri\target\release\bundle"
$setup = Get-ChildItem "$bundle\nsis" -Filter "*-setup.exe" | Select-Object -First 1
if (-not $setup) { throw "No se genero el instalador" }

# --- 3. Portable -----------------------------------------------------------
Write-Host "  Armando la version portable..." -ForegroundColor Cyan
$out = Join-Path $root "release"
$port = Join-Path $out "CHD-Studio-$version-portable"
if (Test-Path $port) { Remove-Item $port -Recurse -Force }
New-Item -ItemType Directory -Force $port | Out-Null

Copy-Item "src-tauri\target\release\chd-studio.exe" (Join-Path $port "CHD Studio.exe")
Copy-Item "src-tauri\binaries" (Join-Path $port "binaries") -Recurse

# Este archivo es lo que activa el modo portable: los datos se quedan al lado
@"
La presencia de este archivo hace que CHD Studio guarde sus ajustes, las
herramientas que descargue y el entorno de Python en la carpeta 'datos', junto
al ejecutable, en vez de en %APPDATA%.

Borralo si prefieres que se comporte como la version instalada.
"@ | Out-File (Join-Path $port "portable.txt") -Encoding utf8

Copy-Item "README.md" $port -ErrorAction SilentlyContinue

$zip = Join-Path $out "CHD-Studio-$version-portable.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path "$port\*" -DestinationPath $zip
Remove-Item $port -Recurse -Force

# --- 4. latest.json --------------------------------------------------------
Write-Host "  Generando latest.json..." -ForegroundColor Cyan
$sigFile = "$($setup.FullName).sig"
if (-not (Test-Path $sigFile)) { throw "No aparecio la firma $sigFile. Revisa que createUpdaterArtifacts este activo." }

$manifest = [ordered]@{
    version   = $version
    notes     = "Consulta las notas de la release en GitHub."
    pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = [ordered]@{
        "windows-x86_64" = [ordered]@{
            signature = (Get-Content $sigFile -Raw).Trim()
            url       = "https://github.com/giver720/chd-studio/releases/download/v$version/$($setup.Name)"
        }
    }
}
$manifest | ConvertTo-Json -Depth 6 | Out-File (Join-Path $out "latest.json") -Encoding utf8

Copy-Item $setup.FullName $out -Force

Write-Host ""
Write-Host "  Listo. En la carpeta 'release':" -ForegroundColor Green
Get-ChildItem $out | Select-Object Name, @{N = "MB"; E = { [math]::Round($_.Length / 1MB, 2) } } | Format-Table -AutoSize
Write-Host "  Para publicarla:" -ForegroundColor Cyan
Write-Host "    gh release create v$version release\* --repo giver720/chd-studio --title `"CHD Studio $version`""
