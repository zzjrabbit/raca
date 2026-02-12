cargo build -p vdso_dylib --target loongarch64-unknown-linux-musl -Zbuild-std
cargo build -p user_boot --target loongarch64-unknown-linux-musl -Zbuild-std
