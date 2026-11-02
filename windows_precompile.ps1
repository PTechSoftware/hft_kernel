Write-Host "=== Configurando entorno para kernel_ptechos en Windows ===" -ForegroundColor Cyan

# 1. Verificar / Instalar QEMU
$qemuDefaultPath = "C:\Program Files\qemu"
$qemuCmd = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue

if (-not $qemuCmd -and -not (Test-Path "$qemuDefaultPath\qemu-system-x86_64.exe")) {
    Write-Host "[1/4] Instalando QEMU mediante winget..." -ForegroundColor Yellow
    winget install --id SoftwareFreedomConservancy.QEMU -e --accept-source-agreements --accept-package-agreements
} else {
    Write-Host "[1/4] QEMU ya se encuentra instalado." -ForegroundColor Green
}

# 2. Asegurar que QEMU este en el PATH del usuario y de la sesion actual
if (Test-Path $qemuDefaultPath) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$qemuDefaultPath*") {
        Write-Host "[2/4] Agregando '$qemuDefaultPath' al PATH del usuario..." -ForegroundColor Yellow
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$qemuDefaultPath", "User")
    } else {
        Write-Host "[2/4] '$qemuDefaultPath' ya esta en el PATH del usuario." -ForegroundColor Green
    }

    if ($env:Path -notlike "*$qemuDefaultPath*") {
        $env:Path += ";$qemuDefaultPath"
    }
}

# 3. Agregar componentes de Rust requeridos para compilar bare-metal
Write-Host "[3/4] Instalando componentes de rustup (rust-src, llvm-tools-preview)..." -ForegroundColor Yellow
rustup component add rust-src llvm-tools-preview

# 4. Instalar bootimage
if (-not (Get-Command cargo-bootimage -ErrorAction SilentlyContinue)) {
    Write-Host "[4/4] Instalando cargo bootimage..." -ForegroundColor Yellow
    cargo install bootimage
} else {
    Write-Host "[4/4] cargo bootimage ya se encuentra instalado." -ForegroundColor Green
}

Write-Host "`nListo! Ya podes compilar y correr el kernel con: cargo run (o .\run.ps1)" -ForegroundColor Cyan
