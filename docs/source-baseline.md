# Source Baseline and References

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 23.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

The architecture in this document is a proposed Rust design. The following source was used only to establish the behavioral baseline and original project capabilities:

miniML-Parser by kiishor - https://github.com/kiishor/miniML-Parser

- Original project description: simple/tiny validating XML parser in C for embedded applications.

- Original README states a 1.8 kB parser code footprint in one IAR ARM release build.

- Original parser validates against an xs_element_t schema tree, extracts typed content, supports callbacks and static/dynamic/relative target addressing.

- Original repository includes an XML schema code generator that emits C source/header structures. Design note: exact compatibility with every miniML-Parser behavior is not assumed. The Rust implementation should document its compatibility surface and intentionally prefer memory safety, deterministic limits and a clear supported XML/XSD profile when behaviors conflict.
