#!/bin/sh
set -eu

echo "Running mandatory checks..."

echo "Formatting..."
cargo fmt --check

echo "Clippy..."
cargo clippy --workspace --locked --all-targets -- -D warnings

echo "Unit tests..."
cargo test --workspace --locked --lib

echo "Integration and doc tests..."
cargo test --workspace --locked --tests
cargo test --workspace --locked --doc

echo "Docs..."
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --locked --no-deps

echo "no_std check..."
cargo check --workspace --locked --no-default-features

if [ "${CHECK_MSRV:-0}" = "1" ]; then
    echo "MSRV check..."
    cargo +1.85.0 check --workspace --locked
fi

if [ "${CHECK_CROSS:-0}" = "1" ]; then
    echo "Cross compilation checks..."
    cargo check --workspace --locked --target thumbv7em-none-eabihf
    cargo check --workspace --locked --target riscv32imac-unknown-none-elf
fi

if [ "${CHECK_COVERAGE:-0}" = "1" ]; then
    echo "Coverage check..."
    if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
        echo "Error: cargo-llvm-cov is missing."
        echo "Run: cargo install cargo-llvm-cov && rustup component add llvm-tools-preview"
        exit 1
    fi
    cargo llvm-cov --workspace --locked --lib --fail-under-lines 100 --fail-under-functions 100
fi

if [ "${CHECK_MUTATION:-0}" = "1" ]; then
    echo "Mutation check..."
    if ! command -v cargo-mutants >/dev/null 2>&1; then
        echo "Error: cargo-mutants is missing."
        echo "Run: cargo install cargo-mutants"
        exit 1
    fi
    # Mutate the critical implemented modules. The builder replacement is exactly
    # equivalent; see the documented M01 mutation baseline in the execution ledger.
    cargo mutants --workspace --jobs 4 --timeout 30 \
        --file 'crates/core/src/{budget,config,convert,schema,validate,state}.rs' \
        --exclude-re 'replace ParserConfig::builder -> ParserConfigBuilder with Default::default\(\)' \
        -- --lib
fi

echo "All checks passed successfully."
