# kernel_ptechos

## Setup

### Windows (PowerShell)
```powershell
.\windows_precompile.ps1
```

### Linux (Ubuntu / Debian)
```bash
sh ubuntu_preecompile.sh
```

## Build & Run

Compilar y ejecutar en QEMU (Windows y Linux):
```bash
cargo run
```

O en Windows mediante el script auxiliar:
```powershell
.\run.ps1
```

Solo generar la imagen booteable:
```bash
cargo bootimage --target x86_64-hft.json
```
