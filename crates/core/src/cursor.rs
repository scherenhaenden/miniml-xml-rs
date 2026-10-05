use core::str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize, // Unicode scalar value column
}

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

            let prev = if self.pos.byte_offset > 0 {
                self.data[..self.pos.byte_offset].chars().next_back()
            } else {
                None
            };

            self.pos.byte_offset = self.pos.byte_offset.checked_add(c.len_utf8()).unwrap();
            Self::update_pos(&mut self.pos, c, prev);

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

        let mut current_offset = self.pos.byte_offset;
        for c in self.data[self.pos.byte_offset..to_consume_end].chars() {
            if !is_xml_char(c) {
                return Err(CursorError::InvalidXmlChar {
                    offset: current_offset,
                });
            }
            let prev = if current_offset > 0 {
                self.data[..current_offset].chars().next_back()
            } else {
                None
            };

            current_offset = current_offset.checked_add(c.len_utf8()).unwrap();
            Self::update_pos(&mut self.pos, c, prev);
            self.pos.byte_offset = current_offset;
        }
        Ok(())
    }

    fn update_pos(pos: &mut Position, c: char, prev: Option<char>) {
        if c == '\r' {
            pos.line = pos.line.checked_add(1).unwrap();
            pos.column = 1;
        } else if c == '\n' {
            if prev == Some('\r') {
                pos.column = 1;
            } else {
                pos.line = pos.line.checked_add(1).unwrap();
                pos.column = 1;
            }
        } else {
            pos.column = pos.column.checked_add(1).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_eof() {
        let mut cursor = Cursor::new(b"").unwrap();
        assert_eq!(cursor.is_eof(), true);
        assert_eq!(cursor.peek(), Ok(None));
        assert_eq!(cursor.consume(), Ok(None));
        assert_eq!(cursor.remaining(), "");
    }

    #[test]
    fn test_invalid_utf8() {
        let err = Cursor::new(b"\xFF").unwrap_err();
        assert_eq!(err, CursorError::InvalidUtf8 { offset: 0 });
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
}
