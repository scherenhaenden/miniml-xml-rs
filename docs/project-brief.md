# Project / User Brief

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 3-4.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

## Background

The project creates a small, validating XML parser in Rust for embedded and systems software. It is inspired by the design goals of the C project miniML-Parser: a compact parser, schema-driven validation, extraction of XML content into typed targets, and a host-side generator that converts an XML Schema definition into code usable by the parser. The Rust project is not intended to be a line-by-line translation. Its goal is to preserve the useful embedded behavior while redesigning ownership, safety, APIs and testing around Rust.

The original project exposes a single top-level parse call, validates against a schema tree, extracts typed values, supports callbacks and several target-addressing modes, and includes a code generator for schema-derived C structures. Its README reports a code footprint of about 1.8 kB for the parser in one embedded compiler configuration. Those capabilities form the behavioral baseline, while this specification deliberately adds stronger safety, deterministic resource controls, richer diagnostics and first-class Rust ergonomics.

Core product idea A tiny schema-aware XML parser that can be used safely in firmware and other constrained software without a heap, while still offering generated typed models and an optional C-compatible migration path.

## Problem statement

Embedded systems frequently need to consume small configuration, provisioning, command or telemetry documents. Full XML stacks are often too large or require allocation, while hand-written parsers and small C libraries can expose memory-safety risks and require manual schema-to-structure mapping. The project must provide a small and deterministic middle ground: validation plus typed extraction without bringing a large runtime or unsafe parser core.

## Primary users

- Firmware and embedded Rust developers who need deterministic XML parsing without std.

- C/C++ embedded teams that want a gradual migration path through an optional C ABI.

- Systems developers who need generated, typed structures from a constrained XSD subset.

- Library maintainers who require strong malformed-input resistance and reproducible resource bounds.

## Value proposition

| Value | Meaning |
| --- | --- |
| Memory safety | All parser logic is safe Rust. Unsafe code is forbidden in the core crate. |
| Embedded suitability | no_std first; heap allocation is optional rather than assumed. |
| Deterministic behavior | Explicit input, depth, token and occurrence limits prevent uncontrolled resource growth. |
| Typed extraction | Schema-driven conversion produces application values without ad-hoc string handling. |
| Migration path | Optional C ABI can replace or coexist with legacy C parsing at a narrow boundary. |
| Tooling | Host-side XSD subset compiler generates Rust and, optionally, C compatibility models. |
| Verifiability | 100% unit coverage is a hard gate before integration and N2N/E2E testing. |

## Success criteria

- A useful XML + schema subset can be parsed and validated with no heap allocation.

- The core compiles with #![no_std] and #![forbid(unsafe_code)].

- All production logic reaches 100% coverage using unit tests alone.

- Malformed XML never causes panic, undefined behavior, out-of-bounds access or unbounded recursion.

- Generated Rust types and schema descriptors compile without manual edits for supported XSD inputs.

- N2N/E2E proves XSD -> generated code -> build -> XML parse -> typed output.

- Optional C consumers can invoke the Rust implementation through a deliberately small ABI.

## Non-goals for v1

- Full W3C XML 1.0/1.1 and full XSD 1.0/1.1 implementation.

- DOM construction as the default model.

- XPath, XQuery, XSLT, DTD validation, external entity loading or network access.

- Transparent support for arbitrary encodings; UTF-8 is the required v1 encoding.

- A general-purpose desktop XML framework competing with quick-xml, roxmltree or libxml2.
