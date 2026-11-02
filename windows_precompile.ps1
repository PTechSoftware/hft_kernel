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

# 3. Sincronizar toolchain y componentes definidos en rust-toolchain.toml
Write-Host "[3/4] Verificando toolchain y componentes de Rust (rust-src, llvm-tools-preview)..." -ForegroundColor Yellow
rustup component add rust-src llvm-tools-preview

# 4. Instalar bootimage v0.10.3 (compatible con nightly-2026-01-14 y bootloader 0.9.33)
$bootimageVer = if (Get-Command cargo-bootimage -ErrorAction SilentlyContinue) { cargo bootimage --version 2>$null } else { "" }
if ($bootimageVer -notlike "*0.10.3*") {
    Write-Host "[4/4] Instalando cargo bootimage v0.10.3..." -ForegroundColor Yellow
    cargo install bootimage --version 0.10.3 --force
} else {
    Write-Host "[4/4] cargo bootimage v0.10.3 ya se encuentra instalado." -ForegroundColor Green
}

Write-Host "`nListo! Ya podes compilar y correr el kernel con: .\run.ps1 (o cargo run)" -ForegroundColor Cyan
