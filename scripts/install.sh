#!/usr/bin/env sh
cargo install cargo-binutils
rustup default nightly
rustup target add riscv64gc-unknown-none-elf
rustup component add llvm-tools-preview
