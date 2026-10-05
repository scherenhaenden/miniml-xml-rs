# Implementation Roadmap

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 21, 24.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

| Phase | Deliverable |
| --- | --- |
| Phase 0 - Bootstrap | Workspace, CI, lint policy, coverage gate, ADR template, no_std skeleton. |
| Phase 1 - Cursor + tokenizer | UTF-8-safe cursor, positions, XML names, text, attributes, entities, comments; 100% unit coverage before proceeding. |
| Phase 2 - Syntax state machine | Start/end tags, nesting, self-closing tags, strict progress/resource invariants. |
| Phase 3 - Schema runtime | Generated/static SchemaNode model, order and cardinality validation. |
| Phase 4 - Typed extraction | Primitive converters, facets, TargetSink, fixed-capacity patterns, borrowed strings. |
| Phase 5 - XSD codegen | Supported XSD frontend -> canonical IR -> deterministic Rust emitter. |
| Phase 6 - Integration + N2N | Generated Rust sample applications and full schema-to-value path. |
| Phase 7 - C compatibility | C emitter + optional FFI crate + cross-language N2N suite. |
| Phase 8 - Hardening | Differential testing vs miniML C, fuzzing, mutation testing, benchmarks, security documentation. |
| Phase 9 - Release | Semver/API review, MSRV lock, compatibility matrix, crates.io publication and release artifacts. |

## First implementation slice

**Recommended first vertical slice**

Implement a root element with one required integer child and one optional string attribute. Drive it through Cursor -> Tokenizer -> State -> Schema validation -> typed TargetSink, with 100% unit coverage. Only after that works should the XSD generator be introduced.

## Suggested public milestones

| Milestone | Public promise |
| --- | --- |
| 0.1.0 | Tokenizer + validating parser for minimal generated static schemas; no_std; safe core. |
| 0.2.0 | Primitive typed extraction, facets, resource limits, richer diagnostics. |
| 0.3.0 | XSD subset generator producing Rust models and schema descriptors. |
| 0.4.0 | Optional alloc conveniences and callbacks. |
| 0.5.0 | C artifact generation + C ABI preview. |
| 0.9.0 | Compatibility/fuzz/mutation hardening and API freeze candidate. |
| 1.0.0 | Stable supported XML/XSD profile, documented C ABI policy, full quality gates. |
