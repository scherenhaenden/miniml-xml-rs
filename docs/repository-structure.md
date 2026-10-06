# Repository and Module Structure

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 19-20.

Status: documentation baseline. This document records the original proposal or broader v1 plan, while the implemented 0.1.0 scope and evidence are tracked in [milestone-0.1.0.md](milestone-0.1.0.md). The known Rust crate names are `miniml-xml-core` and `miniml-xml`; only ABI names in examples remain proposals.

```text
miniml-xml-rs/
├── Cargo.toml                  # proposed workspace
├── README.md
├── LICENSE                     # MIT license
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
