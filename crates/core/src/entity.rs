/// Errors that can occur during entity decoding.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntityError {
    /// Entity was not closed with a semicolon.
    MissingSemicolon { offset: usize },
    /// Numeric entity had no digits.
    MissingDigits { offset: usize },
    /// Invalid digit in a numeric entity.
    InvalidDigit { offset: usize },
    /// Numeric entity value exceeds maximum character range.
    IntegerOverflow { offset: usize },
    /// Resolved character is not a valid XML 1.0 character.
    InvalidXmlCharacter { offset: usize },
    /// General entity is unknown or not one of the predefined entities.
    UnknownEntity { offset: usize },
    /// Input is truncated before the entity could be fully read.
    Truncated { offset: usize },
}

/// Checks if a given character is a valid XML 1.0 scalar value.
/// XML allowed scalars: #x9/#xA/#xD/#x20-#xD7FF/#xE000-#xFFFD/#x10000-#x10FFFF
pub fn is_xml_char(c: char) -> bool {
    crate::cursor::is_xml_char(c)
}

/// Decodes an XML entity from the given input starting at `&`.
/// Returns the decoded character and the total number of bytes consumed,
/// or an error with the offset relative to the start of the `input`.
pub fn decode_entity(input: &[u8]) -> Result<(char, usize), EntityError> {
    if input.is_empty() || input[0] != b'&' {
        return Err(EntityError::UnknownEntity { offset: 0 });
    }

    if input.len() == 1 {
        return Err(EntityError::Truncated { offset: 1 });
    }

    if input[1] == b'#' {
        if input.len() == 2 {
            return Err(EntityError::Truncated { offset: 2 });
        }

        let (is_hex, start_idx) = if input[2] == b'x' {
            (true, 3)
        } else {
            (false, 2)
        };

        if input.len() <= start_idx {
            return Err(EntityError::Truncated { offset: start_idx });
        }

        let mut value: u32 = 0;
        let mut i = start_idx;
        let mut has_digits = false;

        while i < input.len() {
            let b = input[i];
            if b == b';' {
                break;
            }

            let digit_val = if is_hex {
                match b {
                    b'0'..=b'9' => (b - b'0') as u32,
                    b'a'..=b'f' => (b - b'a' + 10) as u32,
                    b'A'..=b'F' => (b - b'A' + 10) as u32,
                    _ => return Err(EntityError::InvalidDigit { offset: i }),
                }
            } else {
                match b {
                    b'0'..=b'9' => (b - b'0') as u32,
                    _ => return Err(EntityError::InvalidDigit { offset: i }),
                }
            };

            has_digits = true;

            // Check overflow before applying
            if is_hex {
                value = value
                    .checked_mul(16)
                    .and_then(|v| v.checked_add(digit_val))
                    .ok_or(EntityError::IntegerOverflow { offset: i })?;
            } else {
                value = value
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit_val))
                    .ok_or(EntityError::IntegerOverflow { offset: i })?;
            }

            i += 1;
        }

        if i == input.len() {
            return Err(EntityError::MissingSemicolon { offset: i });
        }

        if !has_digits {
            return Err(EntityError::MissingDigits { offset: start_idx });
        }

        let c =
            char::from_u32(value).ok_or(EntityError::InvalidXmlCharacter { offset: start_idx })?;

        if !is_xml_char(c) {
            return Err(EntityError::InvalidXmlCharacter { offset: start_idx });
        }

        return Ok((c, i + 1));
    }

    let mut i = 1;
    while i < input.len() && input[i] != b';' {
        i += 1;
    }

    if i == input.len() {
        return Err(EntityError::MissingSemicolon { offset: i });
    }

    let name = &input[1..i];
    let c = match name {
        b"lt" => '<',
        b"gt" => '>',
        b"amp" => '&',
        b"apos" => '\'',
        b"quot" => '"',
        _ => return Err(EntityError::UnknownEntity { offset: 1 }),
    };

    Ok((c, i + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xml_char() {
        assert!(is_xml_char('\u{0009}'));
        assert!(is_xml_char('\u{000A}'));
        assert!(is_xml_char('\u{000D}'));
        assert!(is_xml_char('\u{0020}'));
        assert!(is_xml_char('\u{D7FF}'));
        assert!(is_xml_char('\u{E000}'));
        assert!(is_xml_char('\u{FFFD}'));
        assert!(is_xml_char('\u{10000}'));
        assert!(is_xml_char('\u{10FFFF}'));

        assert!(!is_xml_char('\u{0000}'));
        assert!(!is_xml_char('\u{0008}'));
        assert!(!is_xml_char('\u{000B}'));
        assert!(!is_xml_char('\u{000C}'));
        assert!(!is_xml_char('\u{001F}'));
        assert!(!is_xml_char('\u{FFFE}'));
        assert!(!is_xml_char('\u{FFFF}'));
    }

    #[test]
    fn test_decode_predefined() {
        assert_eq!(decode_entity(b"&amp;"), Ok(('&', 5)));
        assert_eq!(decode_entity(b"&lt;"), Ok(('<', 4)));
        assert_eq!(decode_entity(b"&gt;"), Ok(('>', 4)));
        assert_eq!(decode_entity(b"&apos;"), Ok(('\'', 6)));
        assert_eq!(decode_entity(b"&quot;"), Ok(('"', 6)));
    }

    #[test]
    fn test_decode_numeric_decimal() {
        assert_eq!(decode_entity(b"&#65;"), Ok(('A', 5)));
        assert_eq!(decode_entity(b"&#9;"), Ok(('\t', 4)));
    }

    #[test]
    fn test_decode_numeric_hex() {
        assert_eq!(decode_entity(b"&#x41;"), Ok(('A', 6)));
        assert_eq!(decode_entity(b"&#x9;"), Ok(('\t', 5)));
        assert_eq!(decode_entity(b"&#x10FFFF;"), Ok(('\u{10FFFF}', 10)));
        assert_eq!(decode_entity(b"&#xa;"), Ok(('\n', 5)));
        assert_eq!(decode_entity(b"&#xA;"), Ok(('\n', 5)));
    }

    #[test]
    fn test_errors() {
        assert_eq!(
            decode_entity(b""),
            Err(EntityError::UnknownEntity { offset: 0 })
        );
        assert_eq!(
            decode_entity(b"x"),
            Err(EntityError::UnknownEntity { offset: 0 })
        );

        assert_eq!(
            decode_entity(b"&"),
            Err(EntityError::Truncated { offset: 1 })
        );
        assert_eq!(
            decode_entity(b"&#"),
            Err(EntityError::Truncated { offset: 2 })
        );
        assert_eq!(
            decode_entity(b"&#x"),
            Err(EntityError::Truncated { offset: 3 })
        );

        assert_eq!(
            decode_entity(b"&amp"),
            Err(EntityError::MissingSemicolon { offset: 4 })
        );
        assert_eq!(
            decode_entity(b"&#65"),
            Err(EntityError::MissingSemicolon { offset: 4 })
        );
        assert_eq!(
            decode_entity(b"&#x41"),
            Err(EntityError::MissingSemicolon { offset: 5 })
        );

        assert_eq!(
            decode_entity(b"&#;"),
            Err(EntityError::MissingDigits { offset: 2 })
        );
        assert_eq!(
            decode_entity(b"&#x;"),
            Err(EntityError::MissingDigits { offset: 3 })
        );

        assert_eq!(
            decode_entity(b"&#a;"),
            Err(EntityError::InvalidDigit { offset: 2 })
        );
        assert_eq!(
            decode_entity(b"&#X41;"),
            Err(EntityError::InvalidDigit { offset: 2 })
        ); // 'X' is invalid for decimal
        assert_eq!(
            decode_entity(b"&#x4g;"),
            Err(EntityError::InvalidDigit { offset: 4 })
        );

        assert_eq!(
            decode_entity(b"&unknown;"),
            Err(EntityError::UnknownEntity { offset: 1 })
        );

        assert_eq!(
            decode_entity(b"&#0;"),
            Err(EntityError::InvalidXmlCharacter { offset: 2 })
        );
        assert_eq!(
            decode_entity(b"&#x0;"),
            Err(EntityError::InvalidXmlCharacter { offset: 3 })
        );
        assert_eq!(
            decode_entity(b"&#xD800;"),
            Err(EntityError::InvalidXmlCharacter { offset: 3 })
        ); // Surrogate

        // Overflow
        assert_eq!(
            decode_entity(b"&#4294967296;"),
            Err(EntityError::IntegerOverflow { offset: 11 })
        );
        assert_eq!(
            decode_entity(b"&#x100000000;"),
            Err(EntityError::IntegerOverflow { offset: 11 })
        );
    }
}
