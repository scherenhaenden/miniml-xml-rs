use core::str;

pub use crate::error::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorError {
    InvalidUtf8 { offset: usize },
    InvalidXmlChar { offset: usize },
    InvalidOffset { offset: usize },
}

pub fn is_xml_char(c: char) -> bool {
    matches!(c,
        '\u{0009}' | '\u{000A}' | '\u{000D}' |
        '\u{0020}'..='\u{D7FF}' |
        '\u{E000}'..='\u{FFFD}' |
        '\u{10000}'..='\u{10FFFF}'
    )
}

#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    data: &'a str,
    pos: Position,
    previous_was_cr: bool,
}

impl<'a> Cursor<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, CursorError> {
        match str::from_utf8(bytes) {
            Ok(s) => Ok(Self {
                data: s,
                pos: Position {
                    byte_offset: 0,
                    line: 1,
                    column: 1,
                },
                previous_was_cr: false,
            }),
            Err(e) => Err(CursorError::InvalidUtf8 {
                offset: e.valid_up_to(),
            }),
        }
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn remaining(&self) -> &'a str {
        &self.data[self.pos.byte_offset..]
    }

    pub fn is_eof(&self) -> bool {
        self.pos.byte_offset >= self.data.len()
    }

    pub fn peek(&self) -> Result<Option<char>, CursorError> {
        let remaining = self.remaining();
        if let Some(c) = remaining.chars().next() {
            if !is_xml_char(c) {
                return Err(CursorError::InvalidXmlChar {
                    offset: self.pos.byte_offset,
                });
            }
            Ok(Some(c))
        } else {
            Ok(None)
        }
    }

    pub fn consume(&mut self) -> Result<Option<char>, CursorError> {
        let remaining = self.remaining();
        if let Some(c) = remaining.chars().next() {
            if !is_xml_char(c) {
                return Err(CursorError::InvalidXmlChar {
                    offset: self.pos.byte_offset,
                });
            }

            let mut next_pos = self.pos;
            next_pos.byte_offset += c.len_utf8();
            Self::update_pos(&mut next_pos, c, self.previous_was_cr)?;
            self.pos = next_pos;
            self.previous_was_cr = c == '\r';

            Ok(Some(c))
        } else {
            Ok(None)
        }
    }

    pub fn advance(&mut self, bytes: usize) -> Result<(), CursorError> {
        let to_consume_end =
            self.pos
                .byte_offset
                .checked_add(bytes)
                .ok_or(CursorError::InvalidOffset {
                    offset: self.pos.byte_offset,
                })?;
        if to_consume_end > self.data.len() {
            return Err(CursorError::InvalidOffset {
                offset: to_consume_end,
            });
        }
        if !self.data.is_char_boundary(to_consume_end) {
            return Err(CursorError::InvalidOffset {
                offset: to_consume_end,
            });
        }

        let mut next_pos = self.pos;
        let mut current_offset = self.pos.byte_offset;
        let mut previous_was_cr = self.previous_was_cr;
        for c in self.data[self.pos.byte_offset..to_consume_end].chars() {
            if !is_xml_char(c) {
                return Err(CursorError::InvalidXmlChar {
                    offset: current_offset,
                });
            }
            current_offset += c.len_utf8();
            Self::update_pos(&mut next_pos, c, previous_was_cr)?;
            next_pos.byte_offset = current_offset;
            previous_was_cr = c == '\r';
        }
        self.pos = next_pos;
        self.previous_was_cr = previous_was_cr;
        Ok(())
    }

    fn update_pos(pos: &mut Position, c: char, previous_was_cr: bool) -> Result<(), CursorError> {
        if c == '\r' {
            pos.line = pos.line.checked_add(1).ok_or(CursorError::InvalidOffset {
                offset: pos.byte_offset,
            })?;
            pos.column = 1;
        } else if c == '\n' {
            if previous_was_cr {
                pos.column = 1;
            } else {
                pos.line = pos.line.checked_add(1).ok_or(CursorError::InvalidOffset {
                    offset: pos.byte_offset,
                })?;
                pos.column = 1;
            }
        } else {
            pos.column = pos
                .column
                .checked_add(1)
                .ok_or(CursorError::InvalidOffset {
                    offset: pos.byte_offset,
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_eof() {
        let mut cursor = Cursor::new(b"").unwrap();
        assert!(cursor.is_eof());
        assert_eq!(cursor.peek(), Ok(None));
        assert_eq!(cursor.consume(), Ok(None));
        assert_eq!(cursor.remaining(), "");
    }

    #[test]
    fn test_invalid_utf8() {
        let err = Cursor::new(b"\xFF").unwrap_err();
        assert_eq!(err, CursorError::InvalidUtf8 { offset: 0 });
        let err = Cursor::new(b"ok\xFF").unwrap_err();
        assert_eq!(err, CursorError::InvalidUtf8 { offset: 2 });
    }

    #[test]
    fn test_valid_cursor_position() {
        let mut cursor = Cursor::new(b"a\nb\r\nc\rd").unwrap();

        assert_eq!(cursor.consume(), Ok(Some('a')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 1,
                line: 1,
                column: 2
            }
        );

        assert_eq!(cursor.consume(), Ok(Some('\n')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 2,
                line: 2,
                column: 1
            }
        );

        assert_eq!(cursor.consume(), Ok(Some('b')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 3,
                line: 2,
                column: 2
            }
        );

        assert_eq!(cursor.consume(), Ok(Some('\r')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 4,
                line: 3,
                column: 1
            }
        );

        assert_eq!(cursor.consume(), Ok(Some('\n')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 5,
                line: 3,
                column: 1
            }
        );

        assert_eq!(cursor.consume(), Ok(Some('c')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 6,
                line: 3,
                column: 2
            }
        );

        assert_eq!(cursor.consume(), Ok(Some('\r')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 7,
                line: 4,
                column: 1
            }
        );

        assert_eq!(cursor.consume(), Ok(Some('d')));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 8,
                line: 4,
                column: 2
            }
        );
    }

    #[test]
    fn test_invalid_xml_char() {
        // \u{0000} is not a valid XML char
        let mut cursor = Cursor::new(b"\x00").unwrap();
        assert_eq!(
            cursor.peek(),
            Err(CursorError::InvalidXmlChar { offset: 0 })
        );
        assert_eq!(
            cursor.consume(),
            Err(CursorError::InvalidXmlChar { offset: 0 })
        );
    }

    #[test]
    fn test_advance() {
        let mut cursor = Cursor::new(b"test").unwrap();
        assert_eq!(cursor.advance(2), Ok(()));
        assert_eq!(
            cursor.position(),
            Position {
                byte_offset: 2,
                line: 1,
                column: 3
            }
        );
        assert_eq!(cursor.remaining(), "st");

        // Out of bounds
        assert_eq!(
            cursor.advance(3),
            Err(CursorError::InvalidOffset { offset: 5 })
        );

        // Invalid boundary
        let mut cursor2 = Cursor::new("🚀".as_bytes()).unwrap();
        assert_eq!(
            cursor2.advance(1),
            Err(CursorError::InvalidOffset { offset: 1 })
        );
    }

    #[test]
    fn advance_counts_unicode_scalars_and_crlf_across_calls() {
        let mut cursor = Cursor::new("é\r\nx".as_bytes()).unwrap();
        cursor.advance("é\r".len()).unwrap();
        assert_eq!(cursor.position(), Position::new(3, 2, 1));
        cursor.advance(1).unwrap();
        assert_eq!(cursor.position(), Position::new(4, 2, 1));
        cursor.consume().unwrap();
        assert_eq!(cursor.position(), Position::new(5, 2, 2));
    }

    #[test]
    fn invalid_xml_character_does_not_partially_advance() {
        let mut cursor = Cursor::new(b"a\x00b").unwrap();
        assert_eq!(
            cursor.advance(3),
            Err(CursorError::InvalidXmlChar { offset: 1 })
        );
        assert_eq!(cursor.position(), Position::default());
    }

    #[test]
    fn xml_character_ranges_and_invalid_boundaries() {
        for c in [
            '\u{9}',
            '\u{A}',
            '\u{D}',
            '\u{20}',
            '\u{D7FF}',
            '\u{E000}',
            '\u{FFFD}',
            '\u{10000}',
            '\u{10FFFF}',
        ] {
            assert!(is_xml_char(c), "{c:?}");
        }
        for c in ['\u{0}', '\u{8}', '\u{B}', '\u{1F}', '\u{FFFE}', '\u{FFFF}'] {
            assert!(!is_xml_char(c), "{c:?}");
        }
        let mut cursor = Cursor::new("\u{E000}".as_bytes()).unwrap();
        assert_eq!(cursor.consume(), Ok(Some('\u{E000}')));
        let cursor = Cursor::new("\u{FFFE}".as_bytes()).unwrap();
        assert_eq!(
            cursor.peek(),
            Err(CursorError::InvalidXmlChar { offset: 0 })
        );
    }

    #[test]
    fn offset_overflow_is_reported_without_mutation() {
        let mut cursor = Cursor::new(b"x").unwrap();
        cursor.consume().unwrap();
        assert_eq!(
            cursor.advance(usize::MAX),
            Err(CursorError::InvalidOffset { offset: 1 })
        );
        assert_eq!(cursor.position(), Position::new(1, 1, 2));
    }

    #[test]
    fn line_and_column_overflow_are_reported() {
        let mut pos = Position::new(7, usize::MAX, 1);
        assert_eq!(
            Cursor::update_pos(&mut pos, '\r', false),
            Err(CursorError::InvalidOffset { offset: 7 })
        );
        assert_eq!(pos, Position::new(7, usize::MAX, 1));

        let mut pos = Position::new(8, usize::MAX, 1);
        assert_eq!(
            Cursor::update_pos(&mut pos, '\n', false),
            Err(CursorError::InvalidOffset { offset: 8 })
        );

        let mut pos = Position::new(9, 1, usize::MAX);
        assert_eq!(
            Cursor::update_pos(&mut pos, 'x', false),
            Err(CursorError::InvalidOffset { offset: 9 })
        );
    }
}
