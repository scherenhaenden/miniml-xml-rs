# M06 bounded parser state contract

M06 consumes M05 tokenizer events against the immutable M04 schema. Its runtime
must stay allocation-free, `no_std`, iterative, and bounded by the configured
limits plus fixed storage caps.

## M06-A element stack

`crates/core/src/state.rs` supplies a private `ElementStack` for structural
state. Each frame stores a borrowed element name, its opening position, schema
node ID, current ordered child index, per-descriptor occurrence count, and
per-parent child count. Fixed arrays are sized to MAX_DEPTH;
new(configured_depth) clamps the usable depth to that hard storage bound.

- `push` opens an element with its schema node ID, rejects a second document
  root, initializes that frame's child counters, and fails before mutating the
  stack when the depth limit is reached.
- `pop` accepts only the matching open name. Empty or mismatched closes return a
  syntax error and leave all stack state unchanged.
- `self_close` closes the current event without consuming a persistent stack
  slot, but still counts as one document depth level. It enforces the depth
  limit before recording a root.
- `finish` succeeds only after exactly one root has been seen and the stack is
  empty. Empty and truncated documents return structured EOF errors.

All errors carry the position of the event that could not be accepted. A parse
error means the consumer must treat its target as incomplete.

## Full runtime acceptance

The stack supports the public validating parser in
crates/core/src/validate.rs. The core exposes validate::parse, and the facade
crate re-exports that function together with Schema, schema descriptors,
TargetSink, TextValue, and the parser error/configuration types.

## Parse contract

parse takes the complete input byte slice, a Schema accepted by Schema::new,
a TargetSink tied to the input lifetime, and ParserConfig. Schema::new rejects
any schema version other than SCHEMA_VERSION before parsing starts.

The parser requires exactly one root whose name matches the schema root. It
checks every closing name, the ordered child descriptor sequence, min_occurs
and max_occurs, required and unknown attributes, and integer conversion. It
rejects a second root, unknown or out-of-order children, missing required
children, excess schema occurrences, mismatched closes, mixed content, and
non-whitespace after the root. Element-only and empty nodes accept only XML
whitespace (space, tab, carriage return, and line feed) between elements.

Integer values use the M04 i64 lexical contract: optional leading sign,
decimal ASCII digits, and surrounding XML whitespace are accepted; empty,
sign-only, internal whitespace, invalid digits, and signed overflow or
underflow are errors. String content remains unmodified except for the M03 XML
entity decoding and newline normalization rules.

Attribute count, document bytes, token count, and individual token text are
bounded by the tokenizer. max_depth is bounded by the fixed element stack.
max_children_per_element counts direct child elements independently for each
open parent. max_occurrences is a document-wide count of child-element
occurrences, in addition to schema max_occurs. Aggregate character data for a
single string or integer element is also bounded by max_text_bytes; text split
by comments cannot bypass that limit. No value is silently truncated.

## Allocation and sink lifecycle

The parser uses no heap allocation. Element state is stored in fixed arrays
bounded by MAX_DEPTH; attributes and token text use the tokenizer's fixed
storage; accumulated scalar text uses one MAX_TEXT_BYTES buffer. Unchanged
string values are passed as TextValue::Borrowed. Decoded or joined string
values are passed as TextValue::Decoded and are valid only during the callback;
the sink must copy them before returning if it needs to retain them.

For a non-empty element, callbacks occur in this order: begin_element,
attributes in source order, child callbacks or the scalar content callback,
then end_element. A self-closing string element also receives an empty string
content callback; an integer element with no text is rejected. Sink errors
become positioned parse errors and stop further callbacks.

The sink is not transactional. A parse error, including an error found after
earlier elements were emitted, can leave partial callbacks in the target. The
caller must treat the target as incomplete after any Err. Ok is returned only
after the tokenizer reaches EOF and the stack confirms one complete root and
no open elements. Trailing comments and XML whitespace are consumed before
success is reported.

## Error positions and supported profile

Positions use the M05 convention: zero-based byte offsets and one-based lines
and Unicode-scalar columns, with CRLF counted as one newline. Start-tag,
attribute, text, and closing-tag failures report the source event that could
not be accepted. Missing document structure is reported at EOF. Required
attribute failures point to the containing start tag.

The accepted lexical profile is the M05 tokenizer profile. DTDs, CDATA,
external or general entities, namespace QName semantics, schema facets,
mixed content, and processing instructions under the default strict setting
are unsupported. Relaxing the processing-instruction setting does not add
namespace or DTD support.
