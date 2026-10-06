# Supported Profile

This document describes the XML subset, static schema behavior, and resource bounds implemented for the `0.1.0` milestone. Rust package publication is disabled; the annotated Git tag `v0.1.0` identifies the release commit.

## XML Profile

The tokenizer implements a restricted XML 1.0 (Fifth Edition) lexical profile. Features outside this profile are rejected or left without namespace semantics:

- **Encoding:** Input bytes must be UTF-8. If the XML declaration includes an `encoding` value, it must name UTF-8 (case-insensitively); other encodings are rejected.
- **Names:** Case-sensitive tag and attribute names are validated strictly according to XML 1.0 NameStartChar and NameChar Unicode scalar ranges.
- **Attributes:** Values must be quoted. Duplicate attributes are rejected while parsing.
- **Text content:** Standard character data is supported. Strings are borrowed when unchanged, or passed through a fixed-capacity buffer when entity decoding, newline normalization, or joining text requires it.
- **Entities:** Only the 5 standard predefined XML entities (`&amp;`, `&lt;`, `&gt;`, `&apos;`, `&quot;`) and numeric character references (decimal and hex) are resolved. General entity declarations and DTDs are rejected.
- **Comments and Processing Instructions:** Comments and an XML declaration are recognized and skipped. Other processing instructions are rejected by default; they can be skipped by disabling `strict_processing_instructions`.
- **Unsupported:** DTDs, external entities, CDATA sections, namespace resolution (a colon remains an ordinary name character), and mixed content.

## Static Schema Profile

`miniml-xml-rs` uses a pre-compiled, static schema representation (`Schema` and `SchemaNode`). No runtime XSD parsing occurs. The schema enforces:

- Structural hierarchy (deterministic child ordering and bounds).
- Occurrence constraints (`min_occurs`, `max_occurs`).
- Required and optional attributes.
- Content types: Elements, Empty, String, and Integer (`i64` with overflow checks and the documented decimal lexical rules).
- Mixed content and `any`/`choice` constraints are rejected.

`SCHEMA_VERSION` is currently `1`. `Schema::new` rejects a descriptor version that differs from the runtime constant. There is no XSD generator in this milestone; applications author static descriptors directly.

## `no_std` and Fixed-Storage Behavior

The core parser uses `#![no_std]`, forbids `unsafe` code, and allocates no heap memory. Processing uses explicit fixed-capacity storage.

- **Borrowed vs Decoded Text:** Text passed to `TargetSink` uses `TextValue<'a, 'b>`. Unchanged text can borrow the input. Entity decoding, newline normalization, or joining text split by comments uses a fixed buffer capped by `max_text_bytes`. Decoded values are valid only during the callback.

## Resource Budget and Limits

To prevent resource exhaustion attacks (e.g., deeply nested trees or massive payloads), parsing is strictly bounded by a configurable `ParserConfig`. The default limits are:

- `max_document_bytes`: 4096 bytes
- `max_depth`: 16 (fixed-storage cap: 32)
- `max_attributes_per_element`: 8 (fixed-storage cap: 16)
- `max_children_per_element`: 64
- `max_occurrences`: 32
- `max_text_bytes`: 256 bytes (fixed-storage cap: 256)
- `max_tokens`: 4096
- `strict_processing_instructions`: true

`parse` rejects configurations that exceed a fixed-storage hard limit. Setting a limit to `0` deterministically forbids the corresponding resource; for example, `max_tokens = 0` rejects the root token.

## Error and Completion Behavior

Document parsing errors are returned as `ParseError` values containing:
- `ErrorKind`: Categorical type (for example, syntax, schema, or resource errors).
- `ErrorCode`: Exact violation (for example, `IntegerOverflow` or `Resource`).
- `Position`: The 0-based byte offset, 1-based line and 1-based column where the error was triggered.

`Schema::new` validates static descriptors before parsing and returns `SchemaError` if one is invalid. These construction errors do not contain a document position.

The sink is not transactional: callbacks for earlier elements can run before a later error. Treat the target as incomplete after any `Err`. Success (`Ok`) is returned only after the entire document has been validated, the internal stack has closed cleanly, and exactly one root element was observed.
