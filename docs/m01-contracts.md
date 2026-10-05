# Milestone 0.1.0 Shared Contracts

These contracts form the baseline abstractions to be utilized in upcoming milestones. They support an extensible and safe `#![no_std]` core structure by ensuring components can be loosely coupled and independently verified.

## Exported Interfaces

### Position
Represents where in the UTF-8 XML document a token or error was encountered. Tracks byte offset (0-based) along with line and column identifiers (1-based). A single `\r\n` sequence increments the line count by 1.

### Error Model (`ParseError`, `ErrorKind`, `ErrorCode`)
A `ParseError` is the single unifying structure returned for parsing failure. It carries the high-level `ErrorKind` (e.g., `Syntax`, `Schema`), specific `ErrorCode` detailing exact violations (e.g., `IntegerOverflow`, `Required`), and the contextual `Position`. It replaces dynamic string errors to ensure deterministic `#![no_std]` behavior.

### Configuration (`ParserConfig`, `ResourceBudget`)
`ParserConfig` is an immutable specification dictating strict runtime parsing limits. `ResourceBudget` is the active, mutable bookkeeping object instantiated with this configuration. During a parse session, various tokens and occurrences are explicitly verified against limits (e.g., `budget.consume_attribute()`), which return explicit `Resource` errors if breached.

The parser must call `ParserConfig::validate(position) -> Result<(), ParseError>` before parsing. Fixed storage caps exported by `config` are `MAX_DEPTH = 32`, `MAX_ATTRIBUTES = 16`, `MAX_TEXT_BYTES = 256`; configurations above these fail deterministically, while zero forbids use. `ResourceBudget::new(config)` owns its configuration. Its `check_document_bytes(bytes, position)`, `check_text_bytes(bytes, position)`, `consume_token(position)`, `enter_depth(position)`, `consume_attribute(position)`, `consume_child(position)` and `consume_occurrence(position)` return `Result<(), ParseError>`. `exit_depth()`, `reset_element_counters()` and `reset_occurrences()` update counters without allocating. Per-element counters are helpers; the runtime must preserve independent parent counters in its bounded stack.

## Extension Points

Future milestones will build upon these abstractions with specifically purposed modules. We explicitly intend to delegate logic to these independent modules:

1. **Cursor:** Responsible for byte-level progression over an immutable UTF-8 slice. Needs to maintain our `Position` interface and guarantee safe bounds verification.
2. **Tokenizer:** Built on the Cursor. Identifies individual XML tokens without managing deep syntactic tree state.
3. **Names:** Handles exact definitions and validation constraints of XML identifiers.
4. **Decoder:** Responsible for transforming XML entities and resolving textual elements into valid Unicode.
5. **Schema:** Exposes statically generated XSD constraints through a defined `SchemaNode` trait or type for matching operations.
6. **State:** Coordinates the tokenizer against bounded transitions to assert syntax and well-formed XML tree structures.
7. **Sink:** Receives fully decoded and schema-validated element values and assigns them to the appropriate typed targets.

The error taxonomy generated in this milestone guarantees these future delegates have a rich suite of categorical return values ready for application logic.
