## Instructions


Instalar QEMU
```
sudo apt update -y
sudo apt install qemu-system-x86 qemu-utils
```

Cargo config
```
rustup override set nightly
rustup component add rust-src llvm-tools-preview


find ~/.cargo/registry/src -type f -name x86_64-bootloader.json


cargo bootimage --target x86_64-hft.json

```




- Compile: `Ctrl Shift B`
- Run: `F5`