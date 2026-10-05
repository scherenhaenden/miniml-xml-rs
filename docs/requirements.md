# Requirements

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 4-7.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

The following requirements are normative for the initial implementation unless an Architecture Decision Record (ADR) explicitly supersedes them.

## Functional requirements

| ID | Requirement |
| --- | --- |
| FR-001 | Parse complete XML documents from an immutable UTF-8 byte/string input. |
| FR-002 | Validate element names, hierarchy, child order and occurrence bounds against a compiled schema model. |
| FR-003 | Validate declared attributes, required/optional status and supported attribute value types. |
| FR-004 | Extract element text and attribute values into typed Rust values. |
| FR-005 | Support at minimum string, boolean, signed integer, unsigned integer and floating-point content types in v1. |
| FR-006 | Support numeric minimum/maximum facets and string length constraints in the schema model. |
| FR-007 | Support minOccurs/maxOccurs equivalents within an explicitly bounded implementation range. |
| FR-008 | Return structured errors containing category, reason and source position where determinable. |
| FR-009 | Allow application-defined element completion hooks without requiring heap allocation. |
| FR-010 | Provide a schema code generator that accepts the supported XSD subset and emits Rust types plus static schema descriptors. |
| FR-011 | Optionally emit C-compatible structures/descriptors for legacy interoperability. |
| FR-012 | Provide an optional C ABI crate exposing a stable, minimal parsing surface. |
| FR-013 | Support caller-provided resource limits for document bytes, nesting depth, attributes, children and occurrences. |
| FR-014 | Provide deterministic rejection for XML features intentionally unsupported in v1. |
| FR-015 | Expose parser behavior through a small safe Rust facade with sensible defaults and an explicit configuration builder. |
| FR-016 | Permit no-allocation operation for schemas and target types that do not require dynamic storage. |
| FR-017 | Permit optional alloc-backed convenience targets behind a feature flag. |
| FR-018 | Preserve borrowed slices where possible rather than copying XML content. |
| FR-019 | Validate the entire document; successful return means syntax, schema and extraction all succeeded. |
| FR-020 | Provide machine-readable compatibility manifests for generated code to detect generator/runtime version mismatches. |

## Non-functional requirements

| ID | Requirement |
| --- | --- |
| NFR-001 | The parser core SHALL compile with #![no_std]. |
| NFR-002 | The parser core SHALL use #![forbid(unsafe_code)]. |
| NFR-003 | The core SHALL NOT perform I/O, filesystem access, network access or global mutable state. |
| NFR-004 | Parsing SHALL be deterministic for identical input, schema and configuration. |
| NFR-005 | The default parsing algorithm SHALL be non-recursive or shall enforce a strict configured depth bound before recursion can exhaust the stack. |
| NFR-006 | The no-alloc profile SHALL not allocate dynamically. |
| NFR-007 | Panics SHALL NOT be used for expected malformed-input control flow. |
| NFR-008 | Public APIs SHALL document time, memory and lifetime behavior. |
| NFR-009 | Unit tests alone SHALL produce 100% line and function coverage for production logic; region/branch coverage SHALL target 100% where tooling can measure it reliably. |
| NFR-010 | Integration and N2N/E2E suites SHALL be additional quality gates and SHALL NOT be required to raise unit coverage to 100%. |
| NFR-011 | Fuzzing SHALL include malformed, truncated, deeply nested and adversarial documents. |
| NFR-012 | The project SHALL support reproducible builds and a pinned minimum supported Rust version (MSRV). |
| NFR-013 | Core dependencies SHALL be minimized and every runtime dependency SHALL be justified in an ADR. |
| NFR-014 | Generated code SHALL be deterministic: the same schema and generator version produce byte-for-byte stable output except explicitly documented metadata. |
| NFR-015 | The public Rust API SHALL follow semantic versioning; C ABI compatibility policy SHALL be documented separately. |

## Security requirements

| ID | Requirement |
| --- | --- |
| SEC-001 | External entities, external DTDs and network-loaded entities are forbidden. |
| SEC-002 | Entity expansion beyond the explicitly supported predefined XML entities is forbidden in v1. |
| SEC-003 | All index arithmetic SHALL use checked/saturating logic where overflow could affect bounds. |
| SEC-004 | Every parser loop SHALL make forward progress or return an error. |
| SEC-005 | Maximum nesting depth and document size SHALL be enforced before resource exhaustion. |
| SEC-006 | Conversion overflow and underflow SHALL return typed errors rather than clamp silently. |
| SEC-007 | FFI pointers SHALL be validated at the boundary to the extent possible; raw pointers SHALL NOT enter the core API. |
| SEC-008 | The C ABI crate SHALL contain all project unsafe code unless an ADR explicitly creates another isolated unsafe boundary. |
| SEC-009 | Fuzz regressions SHALL be converted into permanent unit tests before closure. |
| SEC-010 | No parser behavior SHALL depend on locale or ambient process state. |

## Compatibility requirements

| ID | Requirement |
| --- | --- |
| COMP-001 | Behavioral compatibility with miniML-Parser concepts is desirable where it does not conflict with Rust safety or the defined v1 subset. |
| COMP-002 | The Rust-native API is primary; source-level emulation of the C API is secondary. |
| COMP-003 | The optional C ABI SHALL expose opaque handles or POD- compatible descriptors rather than Rust layout-dependent types. |
| COMP-004 | Generated C artifacts SHALL use fixed-width integer types and explicit ownership rules. |
| COMP-005 | Cross-language N2N/E2E tests SHALL compile and run at least one C consumer against the Rust static library on CI host targets. |
