echo "Execute in this order"

sudo apt update
sudo apt install qemu-system-x86 qemu-utils
rustup component add rust-src llvm-tools-preview
cargo install bootimage --version 0.10.3 --force
