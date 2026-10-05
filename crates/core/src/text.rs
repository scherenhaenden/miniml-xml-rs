use crate::entity::{decode_entity, is_xml_char, EntityError};
use core::str;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NormalizeMode {
    Attribute,
    ElementContent,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextError {
    Entity(EntityError),
    CapacityOverflow,
    InvalidUtf8 { offset: usize },
    InvalidXmlCharacter { offset: usize },
}

#[derive(Debug, PartialEq, Eq)]
enum TextStorage<'a, const N: usize> {
    Borrowed(&'a str),
    Owned { buf: [u8; N], len: usize },
}

/// XML text that borrows unchanged input or owns decoded bytes in fixed storage.
/// Its private representation keeps the owned length and UTF-8 invariant valid.
#[derive(Debug, PartialEq, Eq)]
pub struct Text<'a, const N: usize = 256> {
    storage: TextStorage<'a, N>,
}

impl<'a, const N: usize> Text<'a, N> {
    const fn new_owned() -> Self {
        Self {
            storage: TextStorage::Owned {
                buf: [0; N],
                len: 0,
            },
        }
    }

    const fn new_borrowed(s: &'a str) -> Self {
        Self {
            storage: TextStorage::Borrowed(s),
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.storage {
            TextStorage::Borrowed(s) => s,
            TextStorage::Owned { buf, len } => str::from_utf8(&buf[..*len]).unwrap_or(""),
        }
    }

    /// Returns the original input slice when this value did not require decoding.
    pub fn borrowed(&self) -> Option<&'a str> {
        match self.storage {
            TextStorage::Borrowed(s) => Some(s),
            TextStorage::Owned { .. } => None,
        }
    }

    pub fn is_borrowed(&self) -> bool {
        self.borrowed().is_some()
    }

    /// Appends UTF-8 text into the fixed buffer, materializing borrowed text if needed.
    /// `limit` is clamped to `N`; overflow leaves the current value unchanged.
    pub fn try_append_str(&mut self, s: &str, limit: usize) -> Result<(), TextError> {
        let max_len = if limit > N { N } else { limit };
        match &mut self.storage {
            TextStorage::Borrowed(b) => {
                let b_len = b.len();
                let s_len = s.len();
                if b_len > max_len || s_len > max_len - b_len {
                    return Err(TextError::CapacityOverflow);
                }
                let len = b_len + s_len;
                let mut buf = [0; N];
                buf[..b_len].copy_from_slice(b.as_bytes());
                buf[b_len..len].copy_from_slice(s.as_bytes());
                self.storage = TextStorage::Owned { buf, len };
                Ok(())
            }
            TextStorage::Owned { buf, len } => {
                let s_len = s.len();
                if *len > max_len || s_len > max_len - *len {
                    return Err(TextError::CapacityOverflow);
                }
                let next_len = *len + s_len;
                buf[*len..next_len].copy_from_slice(s.as_bytes());
                *len = next_len;
                Ok(())
            }
        }
    }

    pub fn try_append_char(&mut self, c: char, limit: usize) -> Result<(), TextError> {
        let mut encode_buf = [0; 4];
        let encoded = c.encode_utf8(&mut encode_buf);
        self.try_append_str(encoded, limit)
    }
}

fn decode_utf8_char(input: &[u8]) -> Option<(char, usize)> {
    let len = match input.first()? {
        0..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => return None,
    };
    if input.len() < len {
        return None;
    }
    let s = core::str::from_utf8(&input[..len]).ok()?;
    s.chars().next().map(|c| (c, len))
}

fn needs_owned(input: &[u8], mode: NormalizeMode) -> bool {
    input.iter().any(|b| {
        *b == b'&'
            || *b == b'\r'
            || (mode == NormalizeMode::Attribute && (*b == b'\t' || *b == b'\n'))
    })
}

fn adjust_entity_error(error: EntityError, base: usize) -> EntityError {
    match error {
        EntityError::MissingSemicolon { offset } => EntityError::MissingSemicolon {
            offset: offset + base,
        },
        EntityError::MissingDigits { offset } => EntityError::MissingDigits {
            offset: offset + base,
        },
        EntityError::InvalidDigit { offset } => EntityError::InvalidDigit {
            offset: offset + base,
        },
        EntityError::IntegerOverflow { offset } => EntityError::IntegerOverflow {
            offset: offset + base,
        },
        EntityError::InvalidXmlCharacter { offset } => EntityError::InvalidXmlCharacter {
            offset: offset + base,
        },
        EntityError::UnknownEntity { offset } => EntityError::UnknownEntity {
            offset: offset + base,
        },
        EntityError::Truncated { offset } => EntityError::Truncated {
            offset: offset + base,
        },
    }
}

pub fn decode_text<'a, const N: usize>(
    input: &'a [u8],
    mode: NormalizeMode,
    limit: usize,
) -> Result<Text<'a, N>, TextError> {
    let max_len = if limit > N { N } else { limit };

    if !needs_owned(input, mode) {
        let s = core::str::from_utf8(input).map_err(|error| TextError::InvalidUtf8 {
            offset: error.valid_up_to(),
        })?;
        for (offset, c) in s.char_indices() {
            if !is_xml_char(c) {
                return Err(TextError::InvalidXmlCharacter { offset });
            }
        }
        if s.len() > max_len {
            return Err(TextError::CapacityOverflow);
        }
        return Ok(Text::new_borrowed(s));
    }

    let mut text = Text::new_owned();
    let mut i = 0;

    while i < input.len() {
        let b = input[i];

        if b == b'&' {
            let (c, consumed) = decode_entity(&input[i..])
                .map_err(|error| TextError::Entity(adjust_entity_error(error, i)))?;
            text.try_append_char(c, max_len)?;
            i += consumed;
        } else if b == b'\r' {
            let mut is_crlf = false;
            if i + 1 < input.len() && input[i + 1] == b'\n' {
                is_crlf = true;
            }

            let normalized_char = if mode == NormalizeMode::Attribute {
                ' '
            } else {
                '\n'
            };
            text.try_append_char(normalized_char, max_len)?;

            i += if is_crlf { 2 } else { 1 };
        } else if mode == NormalizeMode::Attribute && (b == b'\t' || b == b'\n') {
            text.try_append_char(' ', max_len)?;
            i += 1;
        } else if b >= 0x80 {
            let (c, len) =
                decode_utf8_char(&input[i..]).ok_or(TextError::InvalidUtf8 { offset: i })?;
            if !is_xml_char(c) {
                return Err(TextError::InvalidXmlCharacter { offset: i });
            }
            text.try_append_char(c, max_len)?;
            i += len;
        } else {
            let c = b as char;
            if !is_xml_char(c) {
                return Err(TextError::InvalidXmlCharacter { offset: i });
            }
            text.try_append_char(c, max_len)?;
            i += 1;
        }
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_borrowed() {
        let t: Text<10> = decode_text(b"hello", NormalizeMode::ElementContent, 10).unwrap();
        assert!(t.is_borrowed());
        assert_eq!(t.borrowed(), Some("hello"));
        assert_eq!(t.as_str(), "hello");
    }

    #[test]
    fn test_text_owned() {
        let mut t: Text<10> = Text::new_owned();
        t.try_append_str("hi", 10).unwrap();
        assert_eq!(t.as_str(), "hi");
    }

    #[test]
    fn zero_capacity_storage_accepts_empty_and_rejects_growth() {
        let mut text = Text::<0>::new_owned();
        assert_eq!(text.try_append_str("", 0), Ok(()));
        assert_eq!(
            text.try_append_char('x', 0),
            Err(TextError::CapacityOverflow)
        );
        assert_eq!(text.as_str(), "");
    }

    #[test]
    fn test_try_append_overflow() {
        let mut t: Text<4> = Text::new_borrowed("hi");
        let res = t.try_append_str(" world", 4);
        assert_eq!(res, Err(TextError::CapacityOverflow));

        let mut t2: Text<10> = Text::new_owned();
        t2.try_append_str("hi", 10).unwrap();
        let res2 = t2.try_append_str(" world!!!!", 10);
        assert_eq!(res2, Err(TextError::CapacityOverflow));
    }

    #[test]
    fn append_rejects_a_limit_below_existing_borrowed_or_owned_text() {
        let mut borrowed: Text<8> = Text::new_borrowed("hello");
        assert_eq!(
            borrowed.try_append_str("", 4),
            Err(TextError::CapacityOverflow)
        );
        assert_eq!(borrowed.as_str(), "hello");
        assert!(borrowed.is_borrowed());

        let mut owned: Text<8> = Text::new_owned();
        owned.try_append_str("hello", 8).unwrap();
        assert_eq!(
            owned.try_append_str("", 4),
            Err(TextError::CapacityOverflow)
        );
        assert_eq!(owned.as_str(), "hello");
    }

    #[test]
    fn append_materializes_borrowed_text_and_clamps_the_limit() {
        let mut text: Text<8> = decode_text(b"hi", NormalizeMode::ElementContent, 8).unwrap();
        text.try_append_char('!', 99).unwrap();
        assert_eq!(text.as_str(), "hi!");
        assert!(!text.is_borrowed());

        assert_eq!(
            text.try_append_str("123456", 99),
            Err(TextError::CapacityOverflow)
        );
        assert_eq!(text.as_str(), "hi!");
    }

    #[test]
    fn test_decode_text_borrowed() {
        let res: Text<20> = decode_text(b"hello world", NormalizeMode::ElementContent, 20).unwrap();
        assert!(res.is_borrowed());
        assert_eq!(res.as_str(), "hello world");
    }

    #[test]
    fn unescaped_unicode_remains_borrowed_and_respects_capacity() {
        let text: Text<9> =
            decode_text("é中🚀".as_bytes(), NormalizeMode::ElementContent, 9).unwrap();
        assert_eq!(text.borrowed(), Some("é中🚀"));
        assert_eq!(
            decode_text::<9>("é中🚀".as_bytes(), NormalizeMode::ElementContent, 8),
            Err(TextError::CapacityOverflow)
        );
        let empty = Text::<0>::new_owned();
        assert_eq!(empty.as_str(), "");
        assert_eq!(
            decode_text::<4>(b"hello", NormalizeMode::ElementContent, 99),
            Err(TextError::CapacityOverflow)
        );
        assert_eq!(
            decode_text::<4>("é".as_bytes(), NormalizeMode::ElementContent, 99)
                .unwrap()
                .as_str(),
            "é"
        );
    }

    #[test]
    fn test_decode_text_owned_entity() {
        let res: Text<20> =
            decode_text(b"hello &amp; world", NormalizeMode::ElementContent, 20).unwrap();
        assert!(!res.is_borrowed());
        assert_eq!(res.as_str(), "hello & world");
    }

    #[test]
    fn decoded_text_preserves_raw_multibyte_utf8() {
        let text: Text<16> =
            decode_text("&amp;é中🚀".as_bytes(), NormalizeMode::ElementContent, 16).unwrap();
        assert_eq!(text.as_str(), "&é中🚀");
        assert!(!text.is_borrowed());
    }

    #[test]
    fn test_decode_text_owned_crlf() {
        let res: Text<20> =
            decode_text(b"line1\r\nline2", NormalizeMode::ElementContent, 20).unwrap();
        assert_eq!(res.as_str(), "line1\nline2");

        let res2: Text<20> =
            decode_text(b"line1\rline2", NormalizeMode::ElementContent, 20).unwrap();
        assert_eq!(res2.as_str(), "line1\nline2");
    }

    #[test]
    fn test_decode_text_attribute_normalize() {
        let res: Text<20> = decode_text(b"a\tb\nc\rd\r\ne", NormalizeMode::Attribute, 20).unwrap();
        assert_eq!(res.as_str(), "a b c d e");
    }

    #[test]
    fn test_decode_text_invalid_xml_char() {
        let res: Result<Text<20>, TextError> =
            decode_text(b"hello\x00world", NormalizeMode::ElementContent, 20);
        assert_eq!(res, Err(TextError::InvalidXmlCharacter { offset: 5 }));

        let res2: Result<Text<20>, TextError> =
            decode_text(b"&amp;\x00", NormalizeMode::ElementContent, 20);
        assert_eq!(res2, Err(TextError::InvalidXmlCharacter { offset: 5 }));

        let res3: Result<Text<20>, TextError> =
            decode_text(b"\xEF\xBF\xBE", NormalizeMode::ElementContent, 20);
        assert_eq!(res3, Err(TextError::InvalidXmlCharacter { offset: 0 }));
        let res4: Result<Text<20>, TextError> =
            decode_text(b"&amp;\xEF\xBF\xBE", NormalizeMode::ElementContent, 20);
        assert_eq!(res4, Err(TextError::InvalidXmlCharacter { offset: 5 }));
    }

    #[test]
    fn test_decode_text_invalid_utf8() {
        let res: Result<Text<20>, TextError> =
            decode_text(b"hello\xFFworld", NormalizeMode::ElementContent, 20);
        assert_eq!(res, Err(TextError::InvalidUtf8 { offset: 5 }));

        let res2: Result<Text<20>, TextError> =
            decode_text(b"&amp;\xFF", NormalizeMode::ElementContent, 20);
        assert_eq!(res2, Err(TextError::InvalidUtf8 { offset: 5 }));

        for malformed in [
            &b"\xC3"[..],
            &b"\xC3("[..],
            &b"\xE0\x80\x80"[..],
            &b"\xF5\x80\x80\x80"[..],
        ] {
            assert_eq!(
                decode_text::<20>(malformed, NormalizeMode::ElementContent, 20),
                Err(TextError::InvalidUtf8 { offset: 0 })
            );
        }
    }

    #[test]
    fn test_decode_text_entity_error_offset() {
        let res: Result<Text<20>, TextError> =
            decode_text(b"hello &unknown;", NormalizeMode::ElementContent, 20);
        assert_eq!(
            res,
            Err(TextError::Entity(EntityError::UnknownEntity { offset: 7 }))
        );

        let cases: &[(&[u8], EntityError)] = &[
            (b"x&", EntityError::Truncated { offset: 2 }),
            (b"x&amp", EntityError::MissingSemicolon { offset: 5 }),
            (b"x&#;", EntityError::MissingDigits { offset: 3 }),
            (b"x&#xg;", EntityError::InvalidDigit { offset: 4 }),
            (
                b"x&#4294967296;",
                EntityError::IntegerOverflow { offset: 12 },
            ),
            (b"x&#0;", EntityError::InvalidXmlCharacter { offset: 3 }),
        ];
        for (input, expected) in cases {
            assert_eq!(
                decode_text::<32>(input, NormalizeMode::ElementContent, 32),
                Err(TextError::Entity(*expected))
            );
        }
    }

    #[test]
    fn decode_text_enforces_exact_capacity_and_plus_one() {
        let exact: Text<4> = decode_text(b"abc&amp;", NormalizeMode::ElementContent, 4).unwrap();
        assert_eq!(exact.as_str(), "abc&");
        assert_eq!(
            decode_text::<3>(b"abc&amp;", NormalizeMode::ElementContent, 3),
            Err(TextError::CapacityOverflow)
        );
    }

    #[test]
    fn decoded_unicode_capacity_counts_output_utf8_bytes() {
        let exact: Text<4> = decode_text(b"&#x1F680;", NormalizeMode::ElementContent, 4).unwrap();
        assert_eq!(exact.as_str(), "🚀");
        assert_eq!(
            decode_text::<4>(b"&#x1F680;", NormalizeMode::ElementContent, 3),
            Err(TextError::CapacityOverflow)
        );
    }

    #[test]
    fn normalization_and_utf8_paths_propagate_capacity_errors() {
        assert_eq!(
            decode_text::<4>(b"\r", NormalizeMode::ElementContent, 0),
            Err(TextError::CapacityOverflow)
        );
        assert_eq!(
            decode_text::<4>(b"\t", NormalizeMode::Attribute, 0),
            Err(TextError::CapacityOverflow)
        );
        assert_eq!(
            decode_text::<4>("&amp;é".as_bytes(), NormalizeMode::ElementContent, 1),
            Err(TextError::CapacityOverflow)
        );
    }

    #[test]
    fn utf8_char_decoder_handles_boundaries_and_truncation() {
        assert_eq!(decode_utf8_char(b""), None);
        assert_eq!(decode_utf8_char(b"A"), Some(('A', 1)));
        assert_eq!(decode_utf8_char("é".as_bytes()), Some(('é', 2)));
        assert_eq!(decode_utf8_char("中".as_bytes()), Some(('中', 3)));
        assert_eq!(decode_utf8_char("🚀".as_bytes()), Some(('🚀', 4)));
        assert_eq!(decode_utf8_char(b"\xC3"), None);
        assert_eq!(decode_utf8_char(b"\xC3("), None);
        assert_eq!(decode_utf8_char(b"\xFF"), None);
    }

    #[test]
    fn entity_error_offsets_are_adjusted_for_every_error() {
        let errors = [
            EntityError::MissingSemicolon { offset: 1 },
            EntityError::MissingDigits { offset: 1 },
            EntityError::InvalidDigit { offset: 1 },
            EntityError::IntegerOverflow { offset: 1 },
            EntityError::InvalidXmlCharacter { offset: 1 },
            EntityError::UnknownEntity { offset: 1 },
            EntityError::Truncated { offset: 1 },
        ];
        for error in errors {
            let adjusted = adjust_entity_error(error, 4);
            let offset = match adjusted {
                EntityError::MissingSemicolon { offset }
                | EntityError::MissingDigits { offset }
                | EntityError::InvalidDigit { offset }
                | EntityError::IntegerOverflow { offset }
                | EntityError::InvalidXmlCharacter { offset }
                | EntityError::UnknownEntity { offset }
                | EntityError::Truncated { offset } => offset,
            };
            assert_eq!(offset, 5);
        }
    }

    fn expected_after_one_append(capacity: usize) -> &'static str {
        if capacity > 0 {
            "x"
        } else {
            ""
        }
    }

    fn expected_after_two_appends(capacity: usize) -> &'static str {
        if capacity > 1 {
            "xy"
        } else if capacity > 0 {
            "x"
        } else {
            ""
        }
    }

    fn exercise_capacity<const N: usize>() {
        assert!(!needs_owned(b"plain", NormalizeMode::ElementContent));
        assert!(needs_owned(b"&amp;", NormalizeMode::ElementContent));
        assert!(needs_owned(b"\r", NormalizeMode::ElementContent));
        assert!(!needs_owned(b"\t", NormalizeMode::ElementContent));
        assert!(needs_owned(b"\t", NormalizeMode::Attribute));
        assert!(needs_owned(b"\n", NormalizeMode::Attribute));

        let mut borrowed = Text::<N>::new_borrowed("");
        assert_eq!(borrowed.as_str(), "");
        assert_eq!(borrowed.borrowed(), Some(""));
        assert_eq!(borrowed.try_append_str("", N), Ok(()));
        assert_eq!(borrowed.borrowed(), None);
        assert_eq!(borrowed.as_str(), "");

        let mut owned = Text::<N>::new_owned();
        assert_eq!(owned.as_str(), "");
        let append_char = owned.try_append_char('x', N);
        assert_eq!(append_char.is_ok(), N > 0);
        assert_eq!(owned.as_str(), expected_after_one_append(N));
        let append_string = owned.try_append_str("y", N);
        assert_eq!(append_string.is_ok(), N > 1);
        assert_eq!(owned.as_str(), expected_after_two_appends(N));
    }

    fn exercise_decode_paths<const N: usize>() {
        let clamped: Text<N> =
            decode_text(b"", NormalizeMode::ElementContent, N.saturating_add(1)).unwrap();
        assert_eq!(clamped.as_str(), "");
        let empty: Text<N> = decode_text(b"", NormalizeMode::ElementContent, N).unwrap();
        assert_eq!(empty.borrowed(), Some(""));
        let _ = decode_text::<N>(b"&amp;", NormalizeMode::ElementContent, N);
        let _ = decode_text::<N>(b"\r", NormalizeMode::ElementContent, N);
        let _ = decode_text::<N>(b"\r\n", NormalizeMode::ElementContent, N);
        let _ = decode_text::<N>(b"\r", NormalizeMode::Attribute, N);
        let _ = decode_text::<N>(b"x\rY", NormalizeMode::ElementContent, N);
        let _ = decode_text::<N>(b"\t", NormalizeMode::Attribute, N);
        let _ = decode_text::<N>(b"&unknown;", NormalizeMode::ElementContent, N);
        let _ = decode_text::<N>("&amp;é".as_bytes(), NormalizeMode::ElementContent, N);
        let _ = decode_text::<N>(b"&amp;\xEF\xBF\xBE", NormalizeMode::ElementContent, N);
        assert_eq!(
            decode_text::<N>(b"\xC3", NormalizeMode::ElementContent, N),
            Err(TextError::InvalidUtf8 { offset: 0 })
        );
        assert_eq!(
            decode_text::<N>(b"\0", NormalizeMode::ElementContent, N),
            Err(TextError::InvalidXmlCharacter { offset: 0 })
        );
        assert_eq!(
            decode_text::<N>(b"&amp;\0", NormalizeMode::ElementContent, N),
            Err(TextError::InvalidXmlCharacter { offset: 5 })
        );
        assert_eq!(
            decode_text::<N>(b"&amp;\xC3", NormalizeMode::ElementContent, N),
            Err(TextError::InvalidUtf8 { offset: 5 })
        );
    }

    #[test]
    fn minimum_and_configured_capacities_exercise_borrowed_and_owned_paths() {
        exercise_capacity::<0>();
        exercise_capacity::<1>();
        exercise_capacity::<{ crate::config::MAX_TEXT_BYTES }>();
        exercise_decode_paths::<{ crate::config::MAX_TEXT_BYTES }>();
    }
}
