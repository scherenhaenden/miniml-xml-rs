# Security design baseline

Status: planned design derived from source sections 2.3, 3, 4, 10 and 11. There is no released parser or established vulnerability-reporting channel yet. This document makes no claim of a completed security audit.

## Scope and boundaries

Untrusted XML enters a safe, bounded parser core. Host-side XSD tooling accepts schema input separately and emits verified immutable descriptors. Generated/runtime version mismatches must return a structured error. The optional C ABI is an isolated unsafe boundary; raw pointers do not enter the safe core API.

## Required protections

- Core uses `#![no_std]` and `#![forbid(unsafe_code)]`; core performs no I/O, network access or global mutable state.
- Reject DTDs, external entities and general entity declarations. v1 entity handling is restricted to the predefined XML entities and valid numeric character references.
- Validate input bounds, Unicode references and conversion overflow/underflow.
- Every parser loop advances or returns an error; expected malformed input never uses panic control flow.
- Enforce document, depth, attribute, child, text and occurrence limits before exhaustion; a token budget may additionally bound work.
- Keep caller and Rust ownership explicit at FFI. Validate boundary pointers to the extent possible and provide matching destruction operations for Rust-owned opaque objects.
- Parser behavior is deterministic and independent of locale or ambient process state.

## Verification and release evidence

See [SEC-001 through SEC-010](docs/requirements.md), [test architecture](docs/testing.md), [risks](docs/risks.md) and [acceptance criteria](docs/acceptance.md).

Fuzz malformed, truncated, deep and adversarial input. Preserve every regression as a permanent unit test. Unit-only 100% line/function coverage is a separate gate; property tests and mutation tests strengthen assertions. The C reference is a compatibility oracle, not a correctness oracle.

Before release, define the vulnerability-reporting route, supported-version policy, final resource defaults and audited FFI ownership contract. No contact address or response SLA has been established in the source.
