# M03 Entities & Text Contracts

## Public Interfaces

### Entity Decoder (`crates/core/src/entity.rs`)
- `decode_entity(input: &[u8]) -> Result<(char, usize), EntityError>`
  - Decodes predefined XML entities (`&amp;`, `&lt;`, `&gt;`, `&apos;`, `&quot;`) and numeric character references (decimal `&#...;` and hexadecimal `&#x...;`).
  - Input should start at `&`.
  - Returns the successfully decoded character and the total bytes consumed, or an `EntityError` wrapping the detailed error offset from the start of the `input`.
- `is_xml_char(c: char) -> bool`
  - Validates whether a given scalar is a valid XML 1.0 character.
- `EntityError`
  - Encapsulates granular entity syntax errors (`MissingSemicolon`, `MissingDigits`, `InvalidDigit`, `IntegerOverflow`, `InvalidXmlCharacter`, `UnknownEntity`, `Truncated`) carrying the error byte offset.

### Text Processor (`crates/core/src/text.rs`)
- `Text<'a, const N: usize = 256>`
  - Its storage is private: unchanged text borrows the input, while decoded text uses an inline `[u8; N]` buffer. Callers cannot forge an out-of-range length or invalid owned UTF-8.
- `Text::as_str(&self) -> &str`
  - Reads the guaranteed-valid borrowed or internally built UTF-8 value.
- `Text::borrowed(&self) -> Option<&'a str>` / `Text::is_borrowed(&self) -> bool`
  - Distinguishes input-backed text from materialized decoded output, so callbacks can retain only the former.
- `NormalizeMode`
  - Distinguishes parsing rules per XML standard: `Attribute` vs `ElementContent`.
- `decode_text<'a, const N: usize>(input: &'a [u8], mode: NormalizeMode, limit: usize) -> Result<Text<'a, N>, TextError>`
  - Processes input raw bytes decoding the five predefined entities and numeric references, validates XML scalars and UTF-8, and applies line-ending and attribute-whitespace normalization. The output byte length must fit both `N` and `limit`.
- `TextError`
  - Error type detailing specific issues `CapacityOverflow`, `InvalidUtf8 { offset }`, `InvalidXmlCharacter { offset }`, wrapping underlying `EntityError` logic mapping byte offsets.

## Constraints & Memory Complexity

- **Allocations**: Zero-allocation approach strictly implemented throughout parsing and evaluation routines using direct UTF8 inline storage buffers without heap (`alloc::string::String`) or unsafe interactions.
- **Dependencies**: No extra or third-party dependencies outside of `core`.
- **Time Complexity**: Every text and entity loop linearly evaluates its input, executing efficiently in worst case $O(N)$ scanning characters one-by-one with no recursive entity expansions implemented protecting against billion-laugh attacks.
- **Lifetimes**: Sliced elements preserving lifetimes correctly using safely bounded `'a` references in the borrowed scenario.

## XML Design Decisions

- **Line Endings**: Raw `\r\n` (CRLF) and `\r` (CR) normalize to one `\n` in element content. In attributes, raw `\t`, `\n`, `\r` and `\r\n` normalize to spaces. Numeric character references preserve the referenced scalar.
- **Attribute Normalization**: Adheres exactly where specified under `NormalizeMode::Attribute` modifying parsed EOL patterns without touching decoded numeric references (which inherently retain their referenced scalar equivalents).
