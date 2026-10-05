#!/bin/sh
set -eu

echo "Running mandatory checks..."

echo "Formatting..."
cargo fmt --check

echo "Clippy..."
cargo clippy --workspace --all-targets -- -D warnings

echo "Unit tests..."
cargo test --workspace --lib

echo "Integration and doc tests..."
cargo test --workspace --tests
cargo test --workspace --doc

echo "Docs..."
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps

echo "no_std check..."
cargo check --workspace --no-default-features

if [ "${CHECK_MSRV:-0}" = "1" ]; then
    echo "MSRV check..."
    cargo +1.85.0 check --workspace
fi

if [ "${CHECK_CROSS:-0}" = "1" ]; then
    echo "Cross compilation checks..."
    cargo check --workspace --target thumbv7em-none-eabihf
    cargo check --workspace --target riscv32imac-unknown-none-elf
fi

if [ "${CHECK_COVERAGE:-0}" = "1" ]; then
    echo "Coverage check..."
    if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
        echo "Error: cargo-llvm-cov is missing."
        echo "Run: cargo install cargo-llvm-cov && rustup component add llvm-tools-preview"
        exit 1
    fi
    cargo llvm-cov --workspace --lib --fail-under-lines 100 --fail-under-functions 100
fi

if [ "${CHECK_MUTATION:-0}" = "1" ]; then
    echo "Mutation check..."
    if ! command -v cargo-mutants >/dev/null 2>&1; then
        echo "Error: cargo-mutants is missing."
        echo "Run: cargo install cargo-mutants"
        exit 1
    fi
    # Currently checks only the budget logic. When convert/schema/state are integrated,
    # cargo-mutants will check them as well since we run it on the workspace lib targets.
    cargo mutants -vV --test-tool=cargo -- --lib
fi

echo "All checks passed successfully."
