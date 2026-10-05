# Milestone 0.1.0 task acceptance

The session registry and dispatch state live in [the ledger](milestone-0.1.0.md). A planned task is not an active Jules session. Dispatch uses an exact integrated base commit and the actual preceding interfaces, never an assumed stale API. Each session receives its own task ID, owned paths and tests; the coordinator owns this document and the ledger.

## M01 — foundations

Core and facade workspace, MIT license, version 0.1.0, MSRV 1.85, safe no_std skeleton, structured errors, immutable limit builder and checked resource budget. Record initial implementation decisions and concrete contracts. No tokenizer/schema/parser implementations.

## M02 — cursor and names

Own `crates/core/src/cursor.rs` and `name.rs`. Accept immutable UTF-8 input with a deterministic UTF-8 error. Advance only at scalar boundaries, preserve byte offsets, count scalar columns, treat CRLF as one newline. Parse the documented XML name subset with explicit colon policy; reject invalid initial characters and disallowed XML characters. Unit-test empty/truncated input, Unicode, line endings and every advancement/name rejection path. No tokenizer.

## M03 — references and bounded text

Own `crates/core/src/entity.rs` and `text.rs`. Validate and decode all five predefined entities and decimal/hex numeric references. Reject invalid XML scalars, surrogates, overflow, missing delimiters and unknown/general entities. Borrow unescaped text; materialize decoded text into explicit fixed-capacity/caller-owned storage without allocating. Unit-test exact capacity and plus-one, Unicode expansion and borrowed behavior. Never silently truncate or treat an encoded integer as a different lexical value.

## M04 — static schema and extraction contracts

Own `schema.rs`, `convert.rs`, `sink.rs` and isolated validation helpers. Define immutable static node/attribute descriptors, deterministic ordered children with bounded min/max occurrences, integer/string content and runtime compatibility version. Verify malformed descriptors with errors before parsing. Integer conversion must reject lexical errors and overflow; string extraction must preserve borrowed input or explicit bounded materialization. Sink failures propagate with positions. Unit tests cover descriptor errors, required/optional/order/occurrence limits and signed integer edges. No syntax parser.

## M05 — tokenizer

Own `tokenizer.rs`. Compose integrated cursor/name/text modules into position-bearing events for start/end/self-closing tags, attributes and text. Recognize and skip valid comments; enforce comment grammar. Explicitly handle XML declaration and strict processing-instruction policy. Reject DTDs, external/general entities and unsupported CDATA. Reject malformed quotes, duplicate attributes, missing separators, raw invalid characters and truncated forms. Every loop advances or returns; document/token/attribute/text budgets are enforced. Exhaustive direct unit tests and independent 100% coverage precede integration.

## M06 — complete validating runtime

Own `state.rs`, `validate.rs` runtime integration and facade parser. Use an explicit bounded stack; enforce one complete root, nesting, matching closing names, self-closing behavior and no trailing content. Apply schema hierarchy, child sequence/occurrences, required/unknown/duplicate attributes and integer conversion. Reject mixed content explicitly. Sink receives validated values; successful return validates the entire document, and errors never imply a completed target. Exercise all budgets and schema-version mismatch. Direct unit tests cover the full state transition and error surface, including truncation regressions.

## M07 — integration and consumer

Own separate integration tests, fixtures and a Rust consumer example. Static schema: root with required integer child and optional string attribute. Demonstrate successful typed output with/without attribute, borrowed text, encoded strings, malformed rejection, missing child, duplicate child/attribute, unknown names, integer overflow and exact/plus-one resource bounds. Consumer executes through the facade from clean checkout. Include an embedded no_std compile sample. These tests cannot raise the independent unit coverage number.

## M08 — executable quality gates

Own CI workflows and verification scripts. Run fmt, warnings-denied clippy/docs, workspace unit and separate integration/consumer checks, default no_std builds, MSRV 1.85 and representative ARM/RISC-V builds. Unit-only cargo-llvm-cov must enforce 100% production lines/functions without excluding uncovered production logic. Keep coverage runs separate and document exclusions only for non-production fixtures/examples. Provide bounded malformed/adversarial smoke checks and a mutation baseline for implemented critical conversion/validation logic; explain equivalent survivors rather than masking them. Defer absent generator/C gates honestly to their roadmap milestones.

## M09 — closure and version evidence

Review the integrated parser against this scope; fix confirmed regressions and coverage gaps with direct assertions. Publish exact 0.1.0 supported XML/static-schema profile, public lifetime/resource contracts, quick start, limits, version compatibility and future milestone boundaries. Update README/status and changelog without claiming unsupported v1 features. Record actual gate results, review final diff, and produce the 0.1.0 Git version reference only after applicable checks pass. Keep registry publication disabled. The coordinator updates PR description and reports remaining limitations explicitly.
