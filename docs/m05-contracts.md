# M05 tokenizer contract

`miniml-xml-core::tokenizer::Tokenizer` reads UTF-8 XML from a borrowed byte
slice. It allocates no memory and emits borrowed element and attribute names.
Text values use the bounded `Text` type from M03: unchanged values borrow the
input, while entity decoding or line-ending normalization uses its fixed-size
inline buffer.

## Events and positions

`Tokenizer::next` returns `StartTag`, `EndTag`, `Text`, or `Eof`. A start-tag
event contains its borrowed attribute slice and indicates whether the tag was
self-closing. Attribute names, raw values, and positions refer to the original
input; `Attribute::decode_value` applies entity decoding and XML attribute
normalization. Event and attribute positions use zero-based byte offsets and
one-based line and Unicode-scalar columns. CRLF advances one line.

The attribute slice is stored in the tokenizer's fixed-capacity buffer and is
valid until its next mutable `next` call. Callers should consume or copy any
attribute data they need before asking for another event.

## Supported input profile

The tokenizer accepts XML 1.0 names, quoted attributes, text, comments, an
optional initial UTF-8 BOM, an optional XML 1.0 declaration immediately after
that BOM (or at byte zero when there is no BOM), and processing instructions
only when `strict_processing_instructions` is disabled. Byte offsets continue
to refer to the original input; the BOM does not advance document line or
column positions. It enforces comment grammar, rejects duplicate attributes
and missing separators, decodes predefined and numeric references through M03,
and rejects unknown general entities.

DTD and other `<!...>` declarations are unsupported, including CDATA sections.
The tokenizer does not validate document-level root count, tag nesting, or
matching start and end names; those checks belong to M06's validating runtime.

## Resource limits

The constructor validates fixed-storage limits and the document byte limit.
Each emitted token and skipped comment, declaration, or permitted processing
instruction consumes one token. Attribute count is bounded by both the
configured limit and `MAX_ATTRIBUTES`. Decoded text and attribute values are
bounded by both the configured text limit and `MAX_TEXT_BYTES`; overflow is a
resource error and values are never truncated.
