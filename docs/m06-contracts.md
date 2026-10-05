# M06 bounded parser state contract

M06 consumes M05 tokenizer events against the immutable M04 schema. Its runtime
must stay allocation-free, `no_std`, iterative, and bounded by the configured
limits plus fixed storage caps.

## M06-A element stack

`crates/core/src/state.rs` supplies a private `ElementStack` for structural
state. It stores borrowed element names and their opening positions in fixed
arrays sized to `MAX_DEPTH`; `new(configured_depth)` clamps the usable depth to
that hard storage bound.

- `push` opens an element, rejects a second document root, and fails before
  mutating the stack when the depth limit is reached.
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

This module is only the structural stack. The validating parser must also map
ordered child descriptors and occurrence bounds, required and unknown
attributes, scalar conversion, schema version mismatch, and all parser budgets.
It must consume the complete document before reporting success and send values
to `TargetSink` only according to the integration's documented completion
contract. The first typed vertical slice is a root with one required integer
child and an optional string attribute.
