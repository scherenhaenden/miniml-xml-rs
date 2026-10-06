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
    if ! command -v python3 >/dev/null 2>&1; then
        echo "Error: python3 is required to validate cargo-mutants evidence."
        exit 1
    fi
    PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_mutation_gate.py

    # Exclude only proven equivalents: ParserConfig::builder is Default::default,
    # and Schema::new fixes version to SCHEMA_VERSION (currently 1).
    mutation_dir="$(mktemp -d "${TMPDIR:-/tmp}/miniml-xml-mutants.XXXXXX")"
    echo "Raw mutation report: $mutation_dir/mutants.out"
    mutation_status=0
    if cargo mutants --workspace --jobs 4 --timeout 30 --output "$mutation_dir" \
        --file 'crates/core/src/{budget,config,convert,schema,validate,state}.rs' \
        --exclude-re 'replace ParserConfig::builder -> ParserConfigBuilder with Default::default\(\)' \
        --exclude-re 'replace Schema.*::version -> u32 with 1' \
        -- --lib; then
        mutation_status=0
    else
        mutation_status=$?
    fi
    PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_mutation_results.py \
        "$mutation_dir/mutants.out/outcomes.json" "$mutation_status"
fi

echo "All checks passed successfully."
