# Contributing

The project is at the documentation baseline stage. Read the [requirements](docs/requirements.md), [architecture](docs/architecture.md) and [roadmap](docs/roadmap.md) before implementing a slice.

## Implementation rules

- Keep the parser core `no_std` and forbid unsafe code in that crate.
- Keep XSD processing and code generation on the host; embedded runtime receives static descriptors.
- Isolate raw-pointer interoperability in the optional FFI crate.
- Document time, memory, lifetimes and all resource limits in public APIs.
- Reject unsupported features explicitly and return structured errors for malformed input.
- Justify every core runtime dependency in an ADR using the [template](docs/adr/template.md).
- Record changes to baseline requirements with a superseding ADR and update affected documents.

## Verification

The [test architecture](docs/testing.md) requires unit tests alone to reach 100% production line and function coverage. Integration and N2N/E2E are separate gates. Add malformed-input, exact-limit, limit-plus-one and regression cases for the code being added. Fuzz regressions become permanent unit tests.

The [CI policy](docs/ci-quality-gates.md) defines formatting, lint, mutation, no_std, cross-target, MSRV, documentation and determinism checks. Commands in those documents are illustrative until the Cargo workspace and tooling exist.

## Changes for review

Explain the behavior, relevant requirement IDs and verification results. Keep generated output deterministic, update the supported profiles when behavior changes and distinguish completed implementation from planned work.

The original PDF is a preserved reference. Maintain evolving decisions in Markdown with explicit ADRs rather than silently changing source requirements.
