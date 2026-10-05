use crate::cursor::{Cursor, CursorError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    InvalidStartChar { offset: usize },
    Eof { offset: usize },
    CursorError(CursorError),
}

impl From<CursorError> for NameError {
    fn from(err: CursorError) -> Self {
        NameError::CursorError(err)
    }
}

pub fn is_name_start_char(c: char) -> bool {
    matches!(
        c,
        ':' | 'A'..='Z' | '_' | 'a'..='z'
            | '\u{00C0}'..='\u{00D6}'
            | '\u{00D8}'..='\u{00F6}'
            | '\u{00F8}'..='\u{02FF}'
            | '\u{0370}'..='\u{037D}'
            | '\u{037F}'..='\u{1FFF}'
            | '\u{200C}'..='\u{200D}'
            | '\u{2070}'..='\u{218F}'
            | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}'
            | '\u{F900}'..='\u{FDCF}'
            | '\u{FDF0}'..='\u{FFFD}'
            | '\u{10000}'..='\u{EFFFF}'
    )
}

pub fn is_name_char(c: char) -> bool {
    is_name_start_char(c)
        || matches!(
            c,
            '-' | '.' | '0'..='9' | '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}'
        )
}

/// Returns whether `value` is a non-empty XML 1.0 `Name`.
pub fn is_valid_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if is_name_start_char(first)) && chars.all(is_name_char)
}

pub fn parse_name<'a>(cursor: &mut Cursor<'a>) -> Result<&'a str, NameError> {
    let start_pos = cursor.position().byte_offset;
    let mut current_len = 0;

    if let Some(c) = cursor.peek()? {
        if !is_name_start_char(c) {
            return Err(NameError::InvalidStartChar { offset: start_pos });
        }
        current_len += c.len_utf8();
    } else {
        return Err(NameError::Eof { offset: start_pos });
    }

    // Now advance exactly by current_len which is just the first char.
    // Instead of advance() and peek() repeatedly, we can slice remaining string and check chars,
    // then advance the cursor in one go.

    let remaining = cursor.remaining();
    for c in remaining[current_len..].chars() {
        if !is_name_char(c) {
            break;
        }
        current_len += c.len_utf8();
    }

    let result = &remaining[..current_len];
    cursor.advance(current_len)?;

    Ok(result)
}

#[cfg(test)]
mod character_tests {
    use super::*;

    #[test]
    fn xml_name_start_ranges_and_gaps() {
        for c in [
            ':',
            'A',
            '_',
            'a',
            '\u{00C0}',
            '\u{00D6}',
            '\u{00D8}',
            '\u{00F6}',
            '\u{00F8}',
            '\u{02FF}',
            '\u{0370}',
            '\u{037D}',
            '\u{037F}',
            '\u{1FFF}',
            '\u{200C}',
            '\u{200D}',
            '\u{2070}',
            '\u{218F}',
            '\u{2C00}',
            '\u{2FEF}',
            '\u{3001}',
            '\u{D7FF}',
            '\u{F900}',
            '\u{FDCF}',
            '\u{FDF0}',
            '\u{FFFD}',
            '\u{10000}',
            '\u{EFFFF}',
        ] {
            assert!(is_name_start_char(c), "{c:?}");
            assert!(is_name_char(c), "{c:?}");
        }
        for c in [
            '\u{00D7}',
            '\u{00F7}',
            '\u{037E}',
            '\u{200B}',
            '\u{2FF0}',
            '\u{FFFE}',
            '\u{F0000}',
        ] {
            assert!(!is_name_start_char(c), "{c:?}");
            assert!(!is_name_char(c), "{c:?}");
        }
    }

    #[test]
    fn xml_name_extra_character_ranges() {
        for c in [
            '-', '.', '0', '9', '\u{00B7}', '\u{0300}', '\u{036F}', '\u{203F}', '\u{2040}',
        ] {
            assert!(is_name_char(c), "{c:?}");
        }
        assert!(!is_name_char('/'));
        assert!(is_name_start_char(':'));
        assert!(is_name_char(':'));
        for c in ['-', '.', '0', '\u{00B7}', '\u{0300}', '\u{203F}'] {
            assert!(!is_name_start_char(c));
        }
    }

    #[test]
    fn complete_name_validation_requires_a_valid_nonempty_name() {
        for name in ["name", ":prefixed", "α-β", "a\u{0300}"] {
            assert!(is_valid_name(name), "{name:?}");
        }
        for name in ["", "1name", "has space", "name/part"] {
            assert!(!is_valid_name(name), "{name:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_name() {
        let mut cursor = Cursor::new(b"a_valid:name123.").unwrap();
        assert_eq!(parse_name(&mut cursor), Ok("a_valid:name123."));
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_invalid_start_char() {
        let mut cursor = Cursor::new(b"1invalid").unwrap();
        assert_eq!(
            parse_name(&mut cursor),
            Err(NameError::InvalidStartChar { offset: 0 })
        );
    }

    #[test]
    fn test_delimiter_stopping() {
        let mut cursor = Cursor::new(b"name<rest").unwrap();
        assert_eq!(parse_name(&mut cursor), Ok("name"));
        assert_eq!(cursor.remaining(), "<rest");
    }

    #[test]
    fn test_eof() {
        let mut cursor = Cursor::new(b"").unwrap();
        assert_eq!(parse_name(&mut cursor), Err(NameError::Eof { offset: 0 }));
    }

    #[test]
    fn test_valid_name_unicode() {
        // test a valid name with unicode characters (e.g., \u{03B1} is alpha, \u{03B2} is beta, etc.)
        let mut cursor = Cursor::new("α_β\u{0300}".as_bytes()).unwrap(); // α is name start, _ is name start, β is name start, \u{0300} is name char
        assert_eq!(parse_name(&mut cursor), Ok("α_β\u{0300}"));
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_invalid_name_char_unicode() {
        // using an emoji is valid in XML 1.0 5th edition.
        // Let's use a non-name character like a space or less-than sign to stop parsing.
        let mut cursor = Cursor::new("valid\u{200B}".as_bytes()).unwrap(); // Zero-width space is not a name char
        assert_eq!(parse_name(&mut cursor), Ok("valid"));
        assert_eq!(cursor.remaining(), "\u{200B}");
    }
}

#[test]
fn test_cursor_error_propagation() {
    let mut cursor = Cursor::new(b"\x00").unwrap();
    let err = parse_name(&mut cursor).unwrap_err();
    assert_eq!(
        err,
        NameError::CursorError(crate::cursor::CursorError::InvalidXmlChar { offset: 0 })
    );
}
