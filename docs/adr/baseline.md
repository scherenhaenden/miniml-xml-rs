# Initial Architecture Decision Records

Source: [architecture baseline v0.1, 5 October 2026](../reference/embedded-xml-schema_project_specification.pdf), PDF pages 23.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

The source labels ADR-001 through ADR-010 as **Accepted**. This records accepted design intent in that baseline, not completed implementation. New or superseding decisions should use the [ADR template](template.md).

| ADR | Decision | Status | Rationale |
| --- | --- | --- | --- |
| ADR-001 | Core is no_std and forbids unsafe code | Accepted | This is the primary product safety boundary. |
| ADR-002 | XSD processing is host-side only | Accepted | Avoids embedding a large schema parser in firmware. |
| ADR-003 | Canonical Schema IR separates XSD parsing from emitters | Accepted | Prevents duplicated semantics and enables Rust/C generation. |
| ADR-004 | No DOM in v1 runtime | Accepted | Reduces allocation and memory footprint; streaming/event pipeline is enough for typed extraction. |
| ADR-005 | UTF-8 only in v1 | Accepted | Avoids encoding conversion complexity in embedded runtime. |
| ADR-006 | DTD and external entities rejected | Accepted | Security and footprint simplification. |
| ADR-007 | C ABI lives in separate crate | Accepted | Contains all raw-pointer unsafe boundary logic. |
| ADR-008 | 100% unit coverage is independent gate | Accepted | Ensures every production logic path has a direct unit-level test. |
| ADR-009 | Resource limits are explicit public configuration | Accepted | Prevents resource-exhaustion behavior from being implicit. |
| ADR-010 | Unsafe optimizations require benchmark + ADR | Accepted | Safety is default; exceptions require quantified evidence. |
