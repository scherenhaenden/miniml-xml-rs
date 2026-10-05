# Software Specification

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 7-10.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

## Supported XML profile (v1)

| Feature | v1 policy |
| --- | --- |
| Encoding | UTF-8 only. |
| Elements | Supported; properly nested, case-sensitive names. |
| Attributes | Supported; quoted values; duplicate attributes rejected. |
| Text content | Supported; borrowed when possible. |
| CDATA | Optional feature; if enabled, exposed as text content. |
| Comments | Recognized and skipped; not materialized. |
| Processing instructions | Recognized and skipped, or rejected under strict mode. |
| Namespaces | Phase 2. v1 may treat colon as a legal name character without namespace resolution. |
| DTD / external entities | Rejected. |
| General entity declarations | Rejected. |
| Predefined entities | amp, lt, gt, apos, quot supported. |
| Character references | Decimal and hexadecimal numeric references supported with Unicode validity checks. |
| Mixed content | Not part of the initial typed-schema profile; reject unless schema explicitly opts in later. |

## Supported schema profile

The generator accepts a deliberately constrained XSD subset and lowers it to a canonical Schema IR. The runtime parser never interprets XSD text; it receives generated static descriptors. This keeps host-side complexity out of embedded targets.

| Schema capability | v1 |
| --- | --- |
| Named elements | Yes |
| Nested complex structures | Yes, deterministic child ordering |
| minOccurs / maxOccurs | Yes, bounded |
| Required / optional attributes | Yes |
| Primitive simple types | string, bool, signed/unsigned integers, float |
| Numeric min/max facets | Yes |
| String length facets | Yes |
| Enumerations | Recommended for v1 |
| Choice | Phase 2 unless simple deterministic alternatives can be generated |
| Namespaces/import/include | Phase 2 / host-side only |
| Pattern facets / regex | Phase 2 |
| Mixed content | No |
| any / anyAttribute | No |
| Substitution groups | No |
| Identity constraints | No |

## Public Rust API

Illustrative API - final names may change

```rust
#![no_std]
use embedded_xml_schema::{Parser, ParserConfig, Schema, ParseError};
let config = ParserConfig::builder()
    .max_depth(16)
    .max_attributes_per_element(8)
    .max_document_bytes(4096)
    .build();
let parser = Parser::new(&MY_SCHEMA, config);
let value: DeviceConfig<'_> = parser.parse(xml_bytes)?;
```

The preferred native API owns no global state. Parser is a lightweight value containing references to immutable schema data and configuration. The returned target may borrow from the input, allowing zero-copy string fields where the generated model permits it.

## Core abstractions

| Abstraction | Responsibility |
| --- | --- |
| Parser | Facade coordinating tokenization, state transitions, validation and extraction. |
| ParserConfig | Immutable resource and strictness configuration created via Builder. |
| Schema / SchemaNode | Static, generated description of allowed elements, attributes, types, cardinality and facets. |
| Tokenizer / Cursor | Bounds-safe, forward-only view over input with position tracking. |
| ParserState | Explicit deterministic state machine for XML syntax. |
| Validator | Applies structural and value Specifications to parser events. |
| TargetSink | Strategy trait receiving validated values and building typed output. |
| ParseError | Structured error with category, code and position. |
| ResourceBudget | Tracks configured hard limits. |
| Hook | Optional event/callback interface invoked at safe lifecycle points. |

## Error model

| Category | Examples |
| --- | --- |
| Syntax | Unexpected EOF, malformed tag, unclosed quote, invalid character reference. |
| Schema | Unexpected element, wrong order, missing required child, occurrence limit exceeded. |
| Attribute | Unknown attribute, duplicate attribute, missing required attribute. |
| Conversion | Invalid boolean, integer overflow, float format error. |
| Facet | Below minimum, above maximum, invalid length, invalid enumeration value. |
| Resource | Document too large, depth exceeded, token budget exceeded. |
| Unsupported | DTD, external entity, unsupported XSD-generated feature. |
| Internal contract | Generated/runtime version mismatch. Must not represent malformed-input panic paths. |

Suggested diagnostic shape

```rust
pub struct ParseError {
    pub kind: ErrorKind,
    pub position: Position,
    pub schema_node: Option<SchemaNodeId>,
}
pub struct Position {
    pub byte_offset: u32,
    pub line: u32,
    pub column: u32,
}
```

## Feature flags

| Feature | Default | Purpose |
| --- | --- | --- |
| alloc | off | Owned strings/vectors and dynamic convenience targets. |
| std | off | std::error::Error, filesystem helpers in facade only; implies alloc. |
| cdata | off or on by decision | CDATA token support. |
| callbacks | off | Application lifecycle hooks. |
| ffi | separate crate | C ABI; never a core feature. |
| serde | off | Optional host-side convenience only, not required by parser core. |

## Resource model

The parser must make memory and CPU behavior explicit. Configuration limits are checked before expensive work. The no-alloc profile stores parser state on the stack and references static schema data. Repetition targets must use fixed- capacity arrays, caller-owned buffers, callbacks/sinks or an alloc-enabled profile.

| Budget | Purpose |
| --- | --- |
| max_document_bytes | Reject oversized documents immediately. |
| max_depth | Prevent stack/state exhaustion through deep nesting. |
| max_attributes_per_element | Bound attribute scanning work. |
| max_children_per_element | Bound schema/event bookkeeping. |
| max_text_bytes | Bound conversions and target buffers. |
| max_occurrences | Bound repeated schema elements even if XSD says unbounded. |
| max_tokens | Optional global work budget for adversarial input. |
