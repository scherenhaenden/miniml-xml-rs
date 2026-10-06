# Definition of Done / Acceptance Checklist

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 22-23.

Status: This is the acceptance checklist for the eventual 1.0 release. See the [milestone-0.1.0 ledger](milestone-0.1.0.md) and [roadmap](roadmap.md) for the active supported profile and implemented 0.1.0 scope. Repository name: `miniml-xml-rs`; the Rust facade crate is `miniml-xml`. ABI names in source examples remain proposals until an optional C ABI is designed.

| Area | Done when... |
| --- | --- |
| Core safety | core builds with no_std and forbid(unsafe_code); unsafe exists only in explicitly approved boundary crates. |
| API | Public Parser/Config/Error/Schema contracts are documented and semver-reviewed. |
| XML profile | All supported and unsupported XML features are documented with tests. |
| Schema profile | Supported XSD subset is documented with generator diagnostics for unsupported features. |
| Unit quality | Unit tests alone report 100% production line/function coverage. |
| Assertion quality | Mutation test baseline established; critical parser/validation modules have no unexplained survivors. |
| Integration | Feature combinations, generator/runtime boundary and FFI integration pass. |
| N2N/E2E | At least Rust and C schema-to-typed-output journeys pass from clean checkout. |
| Fuzzing | Core fuzz targets exist; crash corpus empty at release; regressions permanently captured. |
| Determinism | Generator output and parser behavior are deterministic. |
| Embedded | Representative no_std cross-target builds pass. |
| Security | SECURITY.md documents threat model, unsupported entity behavior and resource limits. |
| Docs | README has quick start, scope, comparison, generated example and migration guidance. |
| Release | MSRV, versioning, license and compatibility policy are fixed for the release. |
