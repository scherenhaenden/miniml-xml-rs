# miniml-xml-rs

A planned memory-safe, schema-validating XML parser for embedded Rust, inspired by miniML-Parser.

## Current status

This repository contains the original project specification and its Markdown documentation. The parser, Cargo workspace, generator, tests and CI have not been implemented yet. Version 0.1 in the source denotes an architecture baseline, not a published crate release.

The intended core is `#![no_std]` and `#![forbid(unsafe_code)]`, with deterministic resource limits, a no-allocation profile, typed extraction and host-side compilation of a constrained XSD subset. Optional C interoperability belongs in a separate FFI crate.

## Documentation

Start with the [documentation index](docs/README.md).

- [Project brief](docs/project-brief.md)
- [Requirements: FR, NFR, SEC and COMP](docs/requirements.md)
- [XML/XSD profiles, API, errors, features and resource budgets](docs/specification.md)
- [Architecture and ownership](docs/architecture.md)
- [Testing and standalone 100% unit coverage](docs/testing.md)
- [Implementation roadmap](docs/roadmap.md)
- [Acceptance criteria](docs/acceptance.md)
- [Architecture decisions](docs/adr/baseline.md)
- [Contributing](CONTRIBUTING.md) and [security design baseline](SECURITY.md)

## Source and naming

The source is [embedded-xml-schema_project_specification.pdf](docs/reference/embedded-xml-schema_project_specification.pdf), architecture baseline v0.1, dated 5 October 2026. Markdown documentation keeps its requirements in English and records proposed design rather than claiming implemented behavior.

The repository name is **miniml-xml-rs**. The PDF recommends `embedded-xml-schema` and uses that name in illustrative crate/API examples. Those examples are preserved as proposals; final crate and ABI names remain to be decided.

miniML-Parser is the behavioral reference described by the PDF. Its reported footprint and capabilities are historical source claims, not measurements or verified compatibility of this Rust project. See [source baseline](docs/source-baseline.md).

## Next implementation step

Follow [Phase 0 and the first vertical slice](docs/roadmap.md): bootstrap the workspace, then support one root with a required integer child and an optional string attribute through the full parser pipeline. The XSD generator follows after that slice works.

License, MSRV, final public names and remaining profile decisions must be fixed before release. There is no build or quick-start command yet.
