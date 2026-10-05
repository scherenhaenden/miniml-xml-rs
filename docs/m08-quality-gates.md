# Milestone 0.1.0 Quality Gates

This document records the quality gates required for Milestone 0.1.0.

## Current Baseline (M01)
* **Format:** Passes `cargo fmt --check`
* **Lint:** Passes `cargo clippy --workspace --all-targets -- -D warnings`
* **Unit Tests:** All pass locally via `cargo test --workspace --lib`
* **Docs:** Passes `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`
* **no_std:** Passes `cargo check --workspace --no-default-features`
* **MSRV:** Locked to 1.85, stable Rust.
* **Cross Compilation:** Locked to `thumbv7em-none-eabihf` and `riscv32imac-unknown-none-elf` checks.

## Future Full Milestone Gates (M02-M04 and beyond)
* Full code coverage enforcing 100% lines/functions for unit tests.
* Full workspace integration and doctests.
* Mutation check for all logic, including convert/schema/state once integrated.
* Bounded fuzz/adversarial smoke checks to be supplied through parser/integration tasks.
