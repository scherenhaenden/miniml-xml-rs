# Implementation Roadmap

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 21, 24.

Status: This roadmap began as an implementation proposal. The workspace, parser core, static schema runtime, typed Rust consumer, and base quality gates for 0.1.0 are integrated. Closure documentation and version evidence remain in progress in the [milestone ledger](milestone-0.1.0.md); the implemented behavior is described in the [supported profile](supported-profile.md). XSD generation, a C ABI, broader compatibility, fuzzing, and publication remain later roadmap work. Repository name: `miniml-xml-rs`; the Rust crates are `miniml-xml-core` and `miniml-xml`. ABI names in the source examples remain proposals until a C ABI is designed.

| Phase | Deliverable |
| --- | --- |
| Phase 0 - Bootstrap | Workspace, CI, lint policy, coverage gate, ADR template, no_std skeleton. |
| Phase 1 - Cursor + tokenizer | UTF-8-safe cursor, positions, XML names, text, attributes, entities, comments; 100% unit coverage before proceeding. |
| Phase 2 - Syntax state machine | Start/end tags, nesting, self-closing tags, strict progress/resource invariants. |
| Phase 3 - Schema runtime | Application-authored static SchemaNode model, order and cardinality validation. |
| Phase 4 - Typed extraction | Additional primitive conversions and facets beyond the 0.1.0 typed consumer slice. |
| Phase 5 - XSD codegen | Supported XSD frontend -> canonical IR -> deterministic Rust emitter. |
| Phase 6 - Integration + N2N | Generated Rust sample applications and full schema-to-value path. |
| Phase 7 - C compatibility | C emitter + optional FFI crate + cross-language N2N suite. |
| Phase 8 - Hardening | Broader differential testing vs miniML C, fuzzing, expanded mutation testing, benchmarks, security documentation. |
| Phase 9 - Release | Semver/API review, MSRV lock, compatibility matrix, crates.io publication and release artifacts. |

## Implemented 0.1.0 slice

The 0.1.0 consumer parses a root element with one required integer child and one optional string attribute through the core parser and facade. Applications author the static schema descriptors directly; this milestone does not include an XSD generator. See the [supported profile](supported-profile.md) and [milestone ledger](milestone-0.1.0.md) for the implemented contracts and verification evidence.

## Suggested public milestones

| Milestone | Public promise |
| --- | --- |
| 0.1.0 | Tokenizer + validating parser for minimal application-authored static schemas; no_std; safe core. |
| 0.2.0 | Additional primitive conversions and facets beyond the 0.1.0 typed slice; richer diagnostics. |
| 0.3.0 | XSD subset generator producing Rust models and schema descriptors. |
| 0.4.0 | Optional alloc conveniences and callbacks. |
| 0.5.0 | C artifact generation + C ABI preview. |
| 0.9.0 | Compatibility/fuzz/mutation hardening and API freeze candidate. |
| 1.0.0 | Stable supported XML/XSD profile, documented C ABI policy, full quality gates. |
