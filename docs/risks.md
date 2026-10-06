# Risks, Constraints and Security Considerations

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 21-22.

Status: documentation baseline. This document records the original proposal or broader v1 plan, while the implemented 0.1.0 scope and evidence are tracked in [milestone-0.1.0.md](milestone-0.1.0.md). The known Rust crate names are `miniml-xml-core` and `miniml-xml`; only ABI names in examples remain proposals.

| Risk | Mitigation |
| --- | --- |
| Scope explosion into full XML/XSD | Publish a precise supported-profile document and reject unsupported features explicitly. |
| Code size grows beyond embedded value | Track size benchmarks per release; keep features opt-in; avoid heavy runtime dependencies. |
| 100% coverage becomes metric gaming | Add mutation testing and property/fuzz tests; coverage is a floor, not proof of correctness. |
| Generated code becomes complex/unsafe | Generated runtime descriptors must use only safe public constructors or audited const structures. |
| FFI undermines safety claim | State clearly: core is memory-safe Rust; FFI is an isolated unsafe compatibility boundary with separate tests. |
| Recursive schema/XML exhausts stack | Use explicit bounded state or reject beyond configured depth before recursion. |
| Zero-copy lifetimes make API difficult | Offer generated borrowed model for no-alloc and optional owned model behind alloc. |
| Differential reference contains bugs | Treat C reference as compatibility oracle, not correctness oracle; standards/profile requirements win on known discrepancies. |
| XSD parser itself becomes a second large project | Host codegen supports only a narrow subset; its parser may use established host-side Rust XML libraries because target footprint is irrelevant there. |
| Performance optimization pressures unsafe | Require benchmark evidence and ADR approval; default answer remains safe code. |
