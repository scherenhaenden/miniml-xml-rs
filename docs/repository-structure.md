# Repository and Module Structure

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 19-20.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

```text
miniml-xml-rs/
├── Cargo.toml                  # proposed workspace
├── README.md
├── LICENSE                     # license decision pending
├── SECURITY.md
├── CONTRIBUTING.md
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── compatibility/
│   └── supported-xsd.md
├── crates/
│   ├── core/                   # no_std, forbid(unsafe_code)
│   │   └── src/
│   │       ├── cursor.rs
│   │       ├── tokenizer.rs
│   │       ├── state.rs
│   │       ├── schema.rs
│   │       ├── validate.rs
│   │       ├── convert.rs
│   │       ├── sink.rs
│   │       ├── budget.rs
│   │       └── error.rs
│   ├── facade/                 # safe Rust API
│   ├── schema-ir/              # host-side model
│   ├── codegen/                # supported XSD -> IR -> emitters
│   ├── cli/
│   └── ffi/                    # isolated unsafe C ABI
├── fuzz/
│   ├── fuzz_targets/
│   └── corpus/
├── tests/
│   ├── integration/
│   ├── e2e/
│   ├── c-consumer/
│   └── fixtures/
├── examples/
│   ├── bare-metal-like/
│   ├── rust-generated/
│   └── c-compat/
└── .github/workflows/
    ├── ci.yml
    ├── coverage.yml
    ├── fuzz-smoke.yml
    └── release.yml
```

A smaller initial workspace is acceptable, but the architectural boundaries should remain. In particular, FFI must never be placed inside the safe parser core merely for convenience.
