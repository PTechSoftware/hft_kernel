param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

# Asegurar que QEMU sea visible en la sesion actual si esta en la ruta por defecto de Windows
$qemuDefaultPath = "C:\Program Files\qemu"
if ((Test-Path "$qemuDefaultPath\qemu-system-x86_64.exe") -and ($env:Path -notlike "*$qemuDefaultPath*")) {
    $env:Path += ";$qemuDefaultPath"
}

if (-not (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue)) {
    Write-Error "No se encontro 'qemu-system-x86_64'. Ejecuta primero .\windows_precompile.ps1"
    exit 1
}

if ($Release) {
    Write-Host "Compilando y ejecutando kernel en modo RELEASE..." -ForegroundColor Cyan
    cargo run --release
} else {
    Write-Host "Compilando y ejecutando kernel en modo DEBUG..." -ForegroundColor Cyan
    cargo run
}
