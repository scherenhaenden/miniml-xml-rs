use crate::budget::ResourceBudget;
use crate::config::{ParserConfig, MAX_ATTRIBUTES, MAX_TEXT_BYTES};
use crate::cursor::{Cursor, CursorError};
use crate::error::{ErrorCode, ErrorKind, ParseError, Position};
use crate::name::{parse_name, NameError};
use crate::text::{decode_text, NormalizeMode, Text, TextError};
use core::ops::Range;

/// One attribute in a start tag. The value remains a borrowed source slice;
/// `decode_value` applies XML entity decoding and attribute normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub raw_value: &'a [u8],
    pub position: Position,
    value_position: Position,
}

impl<'a> Attribute<'a> {
    const EMPTY: Self = Self {
        name: "",
        raw_value: &[],
        position: Position {
            byte_offset: 0,
            line: 1,
            column: 1,
        },
        value_position: Position {
            byte_offset: 0,
            line: 1,
            column: 1,
        },
    };

    /// Decodes and normalizes this value into caller-selected fixed storage.
    pub fn decode_value<const N: usize>(&self, limit: usize) -> Result<Text<'a, N>, ParseError> {
        decode_text::<N>(self.raw_value, NormalizeMode::Attribute, limit)
            .map_err(|error| text_error(error, self.value_position, self.raw_value))
    }
}

/// Position-bearing tokenizer events. Attribute slices are valid until the
/// next mutable call to [`Tokenizer::next`].
#[derive(Debug, PartialEq, Eq)]
// Text stays inline to preserve the crate's allocation-free `no_std` contract.
#[allow(clippy::large_enum_variant)]
pub enum Event<'s, 'a> {
    StartTag {
        name: &'a str,
        attributes: &'s [Attribute<'a>],
        empty: bool,
        position: Position,
    },
    EndTag {
        name: &'a str,
        position: Position,
    },
    Text {
        value: Text<'a, MAX_TEXT_BYTES>,
        position: Position,
    },
    Eof,
}

/// Allocation-free, forward-only tokenizer for the milestone's XML profile.
pub struct Tokenizer<'a> {
    input: &'a [u8],
    document_start_offset: usize,
    cursor: Cursor<'a>,
    budget: ResourceBudget,
    max_text_bytes: usize,
    strict_processing_instructions: bool,
    at_document_start: bool,
    attributes: [Attribute<'a>; MAX_ATTRIBUTES],
    attribute_count: usize,
}

impl<'a> Tokenizer<'a> {
    /// Creates a tokenizer after validating fixed-storage configuration and
    /// the document byte limit.
    pub fn new(input: &'a [u8], config: ParserConfig) -> Result<Self, ParseError> {
        config.validate(Position::default())?;
        let document_start_offset = if input.starts_with(b"\xEF\xBB\xBF") {
            3
        } else {
            0
        };
        // The offset is either zero or three bytes after a confirmed BOM.
        let cursor_input = &input[document_start_offset..];
        let cursor = Cursor::with_base_offset(cursor_input, document_start_offset)
            .map_err(|error| cursor_error(error, Position::default(), input))?;
        let budget = ResourceBudget::new(config.clone());
        budget.check_document_bytes(input.len(), Position::default())?;

        Ok(Self {
            input,
            document_start_offset,
            cursor,
            budget,
            max_text_bytes: config.max_text_bytes,
            strict_processing_instructions: config.strict_processing_instructions,
            at_document_start: true,
            attributes: [Attribute::EMPTY; MAX_ATTRIBUTES],
            attribute_count: 0,
        })
    }

    /// Returns the next start tag, end tag, text segment, or end-of-input.
    /// Valid comments, XML declarations, and permitted processing instructions
    /// are consumed internally.
    pub fn next<'s>(&'s mut self) -> Result<Event<'s, 'a>, ParseError> {
        loop {
            if self.cursor.is_eof() {
                return Ok(Event::Eof);
            }

            let position = self.cursor.position();
            let remaining = self.cursor.remaining();

            if remaining.starts_with("<!--") {
                self.budget.consume_token(position)?;
                self.at_document_start = false;
                self.skip_comment(position)?;
                continue;
            }

            if remaining.starts_with("<!") {
                self.budget.consume_token(position)?;
                return Err(ParseError::new(
                    ErrorKind::Unsupported,
                    ErrorCode::Syntax,
                    position,
                ));
            }

            if remaining.starts_with("<?") {
                self.budget.consume_token(position)?;
                if self.is_xml_declaration_start() {
                    if !self.at_document_start || position.byte_offset != self.document_start_offset
                    {
                        return Err(ParseError::new(
                            ErrorKind::Syntax,
                            ErrorCode::XmlDeclaration,
                            position,
                        ));
                    }
                    self.skip_xml_declaration(position)?;
                    self.at_document_start = false;
                    continue;
                }
                self.at_document_start = false;
                self.skip_processing_instruction(position)?;
                continue;
            }

            self.at_document_start = false;
            self.budget.consume_token(position)?;
            if remaining.starts_with("</") {
                return self.parse_end_tag(position);
            }
            if remaining.starts_with('<') {
                return self.parse_start_tag(position);
            }
            return self.parse_text(position);
        }
    }

    fn parse_start_tag<'s>(&'s mut self, position: Position) -> Result<Event<'s, 'a>, ParseError> {
        advance_cursor(&mut self.cursor, 1, self.input)?;
        let name = self.read_name(ErrorCode::Name)?;
        self.attribute_count = 0;
        self.budget.reset_element_counters();

        loop {
            let separated = self.skip_whitespace()?;
            if self.cursor.remaining().starts_with('>') {
                advance_cursor(&mut self.cursor, 1, self.input)?;
                return Ok(self.start_tag_event(name, false, position));
            }
            if self.cursor.remaining().starts_with("/>") {
                advance_cursor(&mut self.cursor, 2, self.input)?;
                return Ok(self.start_tag_event(name, true, position));
            }
            if !separated {
                return Err(self.syntax_error(ErrorCode::Attribute, self.cursor.position()));
            }
            self.parse_attribute()?;
        }
    }

    fn start_tag_event<'s>(
        &'s self,
        name: &'a str,
        empty: bool,
        position: Position,
    ) -> Event<'s, 'a> {
        Event::StartTag {
            name,
            attributes: &self.attributes[..self.attribute_count],
            empty,
            position,
        }
    }

    fn parse_attribute(&mut self) -> Result<(), ParseError> {
        let position = self.cursor.position();
        let name = self.read_name(ErrorCode::Name)?;
        for previous in &self.attributes[..self.attribute_count] {
            if previous.name == name {
                return Err(self.syntax_error(ErrorCode::Attribute, position));
            }
        }
        self.budget.consume_attribute(position)?;

        self.skip_whitespace()?;
        if !self.cursor.remaining().starts_with('=') {
            return Err(self.syntax_error(ErrorCode::Attribute, self.cursor.position()));
        }
        advance_cursor(&mut self.cursor, 1, self.input)?;
        self.skip_whitespace()?;

        let quote = read_quote(
            self.cursor.peek(),
            self.cursor.position(),
            self.input,
            ErrorCode::Attribute,
        )?;
        advance_cursor(&mut self.cursor, 1, self.input)?;
        let value_position = self.cursor.position();
        let start = value_position.byte_offset;

        loop {
            match self.cursor.peek() {
                Ok(Some(current)) if current == quote => break,
                Ok(Some('<')) => {
                    return Err(self.syntax_error(ErrorCode::Attribute, self.cursor.position()));
                }
                Ok(Some(_)) => {
                    consume_cursor(&mut self.cursor, self.input)?;
                }
                Ok(None) => {
                    return Err(self.syntax_error(ErrorCode::Eof, self.cursor.position()));
                }
                Err(error) => {
                    return Err(cursor_error(error, self.cursor.position(), self.input));
                }
            }
        }

        let end = self.cursor.position().byte_offset;
        advance_cursor(&mut self.cursor, 1, self.input)?;
        let raw_value = input_slice(self.input, start..end, value_position, ErrorCode::Attribute)?;
        let decoded =
            decode_text::<MAX_TEXT_BYTES>(raw_value, NormalizeMode::Attribute, self.max_text_bytes)
                .map_err(|error| text_error(error, value_position, raw_value))?;
        self.budget
            .check_text_bytes(decoded.as_str().len(), value_position)?;

        let slot = attribute_slot(&mut self.attributes, self.attribute_count, position)?;
        *slot = Attribute {
            name,
            raw_value,
            position,
            value_position,
        };
        self.attribute_count += 1;
        Ok(())
    }

    fn parse_end_tag(&mut self, position: Position) -> Result<Event<'_, 'a>, ParseError> {
        advance_cursor(&mut self.cursor, 2, self.input)?;
        let name = self.read_name(ErrorCode::Name)?;
        self.skip_whitespace()?;
        if !self.cursor.remaining().starts_with('>') {
            return Err(self.syntax_error(ErrorCode::Tag, self.cursor.position()));
        }
        advance_cursor(&mut self.cursor, 1, self.input)?;
        Ok(Event::EndTag { name, position })
    }

    fn parse_text(&mut self, position: Position) -> Result<Event<'_, 'a>, ParseError> {
        let start = position.byte_offset;
        loop {
            if self.cursor.is_eof() {
                break;
            }
            if self.cursor.remaining().starts_with('<') {
                break;
            }
            if self.cursor.remaining().starts_with("]]>") {
                return Err(self.syntax_error(ErrorCode::Syntax, self.cursor.position()));
            }
            consume_cursor(&mut self.cursor, self.input)?;
        }
        let end = self.cursor.position().byte_offset;
        let raw = input_slice(self.input, start..end, position, ErrorCode::Syntax)?;
        let value =
            decode_text::<MAX_TEXT_BYTES>(raw, NormalizeMode::ElementContent, self.max_text_bytes)
                .map_err(|error| text_error(error, position, raw))?;
        self.budget
            .check_text_bytes(value.as_str().len(), position)?;
        Ok(Event::Text { value, position })
    }

    fn skip_comment(&mut self, position: Position) -> Result<(), ParseError> {
        advance_cursor(&mut self.cursor, 4, self.input)?;
        loop {
            if self.cursor.is_eof() {
                return Err(self.syntax_error(ErrorCode::Comment, position));
            }
            if self.cursor.remaining().starts_with("-->") {
                advance_cursor(&mut self.cursor, 3, self.input)?;
                return Ok(());
            }
            if self.cursor.remaining().starts_with("--") {
                return Err(self.syntax_error(ErrorCode::Comment, self.cursor.position()));
            }
            consume_cursor(&mut self.cursor, self.input)?;
        }
    }

    fn skip_processing_instruction(&mut self, position: Position) -> Result<(), ParseError> {
        if self.strict_processing_instructions {
            return Err(self.syntax_error(ErrorCode::ProcessingInstruction, position));
        }
        advance_cursor(&mut self.cursor, 2, self.input)?;
        let target = self.read_name(ErrorCode::ProcessingInstruction)?;
        if target.eq_ignore_ascii_case("xml") {
            return Err(self.syntax_error(ErrorCode::ProcessingInstruction, position));
        }
        let separated = self.skip_whitespace()?;
        loop {
            if self.cursor.remaining().starts_with("?>") {
                advance_cursor(&mut self.cursor, 2, self.input)?;
                return Ok(());
            }
            if !separated {
                return Err(
                    self.syntax_error(ErrorCode::ProcessingInstruction, self.cursor.position())
                );
            }
            if self.cursor.is_eof() {
                return Err(self.syntax_error(ErrorCode::ProcessingInstruction, position));
            }
            consume_cursor(&mut self.cursor, self.input)?;
        }
    }

    fn is_xml_declaration_start(&self) -> bool {
        let Some(rest) = self.cursor.remaining().strip_prefix("<?xml") else {
            return false;
        };
        rest.is_empty()
            || rest.starts_with("?>")
            || rest.chars().next().is_some_and(is_xml_whitespace)
    }

    fn skip_xml_declaration(&mut self, position: Position) -> Result<(), ParseError> {
        advance_cursor(&mut self.cursor, 5, self.input)?;
        if !self.skip_whitespace()? {
            return Err(self.syntax_error(ErrorCode::XmlDeclaration, position));
        }

        let (name, value) = self.parse_declaration_attribute(position)?;
        if name != "version" || value != b"1.0" {
            return Err(self.syntax_error(ErrorCode::XmlDeclaration, position));
        }

        let mut seen_encoding = false;
        let mut seen_standalone = false;
        loop {
            let separated = self.skip_whitespace()?;
            if self.cursor.remaining().starts_with("?>") {
                advance_cursor(&mut self.cursor, 2, self.input)?;
                return Ok(());
            }
            if !separated {
                return Err(self.syntax_error(ErrorCode::XmlDeclaration, position));
            }

            let (name, value) = self.parse_declaration_attribute(position)?;
            match name {
                "encoding"
                    if !seen_encoding
                        && !seen_standalone
                        && valid_encoding_name(value)
                        && value.eq_ignore_ascii_case(b"UTF-8") =>
                {
                    seen_encoding = true;
                }
                "standalone" if !seen_standalone && matches!(value, b"yes" | b"no") => {
                    seen_standalone = true;
                }
                _ => return Err(self.syntax_error(ErrorCode::XmlDeclaration, position)),
            }
        }
    }

    fn parse_declaration_attribute(
        &mut self,
        declaration_position: Position,
    ) -> Result<(&'a str, &'a [u8]), ParseError> {
        let name = self.read_name(ErrorCode::XmlDeclaration)?;
        self.skip_whitespace()?;
        if !self.cursor.remaining().starts_with('=') {
            return Err(self.syntax_error(ErrorCode::XmlDeclaration, declaration_position));
        }
        advance_cursor(&mut self.cursor, 1, self.input)?;
        self.skip_whitespace()?;
        let quote = read_quote(
            self.cursor.peek(),
            self.cursor.position(),
            self.input,
            ErrorCode::XmlDeclaration,
        )?;
        advance_cursor(&mut self.cursor, 1, self.input)?;
        let start = self.cursor.position().byte_offset;
        loop {
            match self.cursor.peek() {
                Ok(Some(current)) if current == quote => break,
                Ok(Some('<' | '&')) => {
                    return Err(
                        self.syntax_error(ErrorCode::XmlDeclaration, self.cursor.position())
                    );
                }
                Ok(Some(_)) => {
                    consume_cursor(&mut self.cursor, self.input)?;
                }
                Ok(None) => {
                    return Err(self.syntax_error(ErrorCode::XmlDeclaration, declaration_position));
                }
                Err(error) => {
                    return Err(cursor_error(error, self.cursor.position(), self.input));
                }
            }
        }
        let end = self.cursor.position().byte_offset;
        advance_cursor(&mut self.cursor, 1, self.input)?;
        let raw = input_slice(
            self.input,
            start..end,
            declaration_position,
            ErrorCode::XmlDeclaration,
        )?;
        Ok((name, raw))
    }

    fn read_name(&mut self, code: ErrorCode) -> Result<&'a str, ParseError> {
        parse_name(&mut self.cursor).map_err(|error| name_error(error, code, self.input))
    }

    fn skip_whitespace(&mut self) -> Result<bool, ParseError> {
        let mut skipped = false;
        loop {
            match self.cursor.peek() {
                Ok(Some(character)) if is_xml_whitespace(character) => {
                    consume_cursor(&mut self.cursor, self.input)?;
                    skipped = true;
                }
                Ok(_) => return Ok(skipped),
                Err(error) => {
                    return Err(cursor_error(error, self.cursor.position(), self.input));
                }
            }
        }
    }

    fn syntax_error(&self, code: ErrorCode, position: Position) -> ParseError {
        ParseError::new(ErrorKind::Syntax, code, position)
    }
}

fn is_xml_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r')
}

fn valid_encoding_name(value: &[u8]) -> bool {
    let Some((&first, rest)) = value.split_first() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    rest.iter()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn read_quote(
    next: Result<Option<char>, CursorError>,
    position: Position,
    input: &[u8],
    code: ErrorCode,
) -> Result<char, ParseError> {
    match next {
        Ok(Some('\'')) => Ok('\''),
        Ok(Some('"')) => Ok('"'),
        Ok(_) => Err(ParseError::new(ErrorKind::Syntax, code, position)),
        Err(error) => Err(cursor_error(error, position, input)),
    }
}

fn cursor_error(error: CursorError, fallback: Position, input: &[u8]) -> ParseError {
    let (offset, code) = match error {
        CursorError::InvalidUtf8 { offset } => (offset, ErrorCode::Utf8),
        CursorError::InvalidXmlChar { offset } => (offset, ErrorCode::Syntax),
        CursorError::InvalidOffset { offset } => (offset, ErrorCode::Syntax),
    };
    let mut position = position_at(input, offset);
    if matches!(error, CursorError::InvalidOffset { .. }) && offset > input.len() {
        position = fallback;
    }
    ParseError::new(ErrorKind::Syntax, code, position)
}

fn advance_cursor(cursor: &mut Cursor<'_>, bytes: usize, input: &[u8]) -> Result<(), ParseError> {
    let fallback = cursor.position();
    match cursor.advance(bytes) {
        Ok(()) => Ok(()),
        Err(error) => Err(cursor_error(error, fallback, input)),
    }
}

fn consume_cursor(cursor: &mut Cursor<'_>, input: &[u8]) -> Result<(), ParseError> {
    let fallback = cursor.position();
    match cursor.consume() {
        Ok(Some(_)) | Ok(None) => Ok(()),
        Err(error) => Err(cursor_error(error, fallback, input)),
    }
}

fn input_slice(
    input: &[u8],
    range: Range<usize>,
    position: Position,
    code: ErrorCode,
) -> Result<&[u8], ParseError> {
    match input.get(range) {
        Some(slice) => Ok(slice),
        None => Err(ParseError::new(ErrorKind::Syntax, code, position)),
    }
}

fn attribute_slot<'s, 'a>(
    attributes: &'s mut [Attribute<'a>],
    index: usize,
    position: Position,
) -> Result<&'s mut Attribute<'a>, ParseError> {
    match attributes.get_mut(index) {
        Some(attribute) => Ok(attribute),
        None => Err(ParseError::new(
            ErrorKind::Resource,
            ErrorCode::Resource,
            position,
        )),
    }
}

fn name_error(error: NameError, expected: ErrorCode, input: &[u8]) -> ParseError {
    let (offset, code) = match error {
        NameError::InvalidStartChar { offset } => (offset, expected),
        NameError::Eof { offset } => (offset, ErrorCode::Eof),
        NameError::CursorError(CursorError::InvalidUtf8 { offset }) => (offset, ErrorCode::Utf8),
        NameError::CursorError(CursorError::InvalidXmlChar { offset })
        | NameError::CursorError(CursorError::InvalidOffset { offset }) => {
            (offset, ErrorCode::Syntax)
        }
    };
    ParseError::new(ErrorKind::Syntax, code, position_at(input, offset))
}

fn position_at(input: &[u8], offset: usize) -> Position {
    let Some(prefix) = input.get(..offset) else {
        return Position::new(offset, 1, 1);
    };
    let prefix = if input.starts_with(b"\xEF\xBB\xBF") && offset >= 3 {
        &prefix[3..]
    } else {
        prefix
    };
    let Ok(prefix) = core::str::from_utf8(prefix) else {
        return Position::new(offset, 1, 1);
    };
    let mut line = 1usize;
    let mut column = 1usize;
    let mut previous_was_cr = false;
    for character in prefix.chars() {
        if character == '\r' {
            line = line.saturating_add(1);
            column = 1;
        } else if character == '\n' {
            if !previous_was_cr {
                line = line.saturating_add(1);
            }
            column = 1;
        } else {
            column = column.saturating_add(1);
        }
        previous_was_cr = character == '\r';
    }
    Position::new(offset, line, column)
}

fn text_error(error: TextError, base: Position, input: &[u8]) -> ParseError {
    let (relative_offset, kind, code) = match error {
        TextError::Entity(crate::entity::EntityError::UnknownEntity { offset }) => {
            (offset, ErrorKind::Unsupported, ErrorCode::Reference)
        }
        TextError::Entity(entity_error) => (
            entity_error_offset(entity_error),
            ErrorKind::Syntax,
            ErrorCode::Reference,
        ),
        TextError::CapacityOverflow => (0, ErrorKind::Resource, ErrorCode::Resource),
        TextError::InvalidUtf8 { offset } => (offset, ErrorKind::Syntax, ErrorCode::Utf8),
        TextError::InvalidXmlCharacter { offset } => (offset, ErrorKind::Syntax, ErrorCode::Syntax),
    };
    ParseError::new(kind, code, position_after(base, input, relative_offset))
}

fn position_after(base: Position, input: &[u8], offset: usize) -> Position {
    let Some(prefix) = input.get(..offset) else {
        return Position::new(
            base.byte_offset.saturating_add(offset),
            base.line,
            base.column,
        );
    };
    let Ok(prefix) = core::str::from_utf8(prefix) else {
        return Position::new(
            base.byte_offset.saturating_add(offset),
            base.line,
            base.column,
        );
    };
    let mut line = base.line;
    let mut column = base.column;
    let mut previous_was_cr = false;
    for character in prefix.chars() {
        if character == '\r' {
            line = line.saturating_add(1);
            column = 1;
        } else if character == '\n' {
            if !previous_was_cr {
                line = line.saturating_add(1);
            }
            column = 1;
        } else {
            column = column.saturating_add(1);
        }
        previous_was_cr = character == '\r';
    }
    Position::new(base.byte_offset.saturating_add(offset), line, column)
}

fn entity_error_offset(error: crate::entity::EntityError) -> usize {
    use crate::entity::EntityError;
    match error {
        EntityError::MissingSemicolon { offset }
        | EntityError::MissingDigits { offset }
        | EntityError::InvalidDigit { offset }
        | EntityError::IntegerOverflow { offset }
        | EntityError::InvalidXmlCharacter { offset }
        | EntityError::UnknownEntity { offset }
        | EntityError::Truncated { offset } => offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ParserConfigBuilder;

    fn make(input: &[u8]) -> Tokenizer<'_> {
        Tokenizer::new(input, ParserConfig::default()).unwrap()
    }

    fn error(input: &[u8], config: ParserConfig) -> Option<ParseError> {
        let mut tokenizer = Tokenizer::new(input, config).unwrap();
        loop {
            match tokenizer.next() {
                Err(error) => return Some(error),
                Ok(Event::Eof) => return None,
                Ok(_) => {}
            }
        }
    }

    fn assert_error(input: &[u8], config: ParserConfig, code: ErrorCode) {
        assert_eq!(
            error(input, config).map(|error| error.code),
            Some(code),
            "input: {input:?}"
        );
    }

    #[test]
    fn tokenizes_start_attributes_text_end_and_self_closing_tags() {
        let mut tokenizer = make(b"<root a='one' b=\"two\">hello</root><empty/>");
        let event = tokenizer.next().unwrap();
        assert!(matches!(
            &event,
            Event::StartTag { name, attributes, empty, position }
                if *name == "root"
                    && !empty
                    && *position == Position::default()
                    && attributes.len() == 2
                    && attributes[0].name == "a"
                    && attributes[0].raw_value == b"one"
                    && attributes[0].position.byte_offset == 6
                    && attributes[1].name == "b"
                    && attributes[1].decode_value::<16>(16).is_ok_and(|value| value.as_str() == "two")
        ));
        let event = tokenizer.next().unwrap();
        assert!(matches!(
            &event,
            Event::Text { value, position }
                if value.as_str() == "hello" && value.is_borrowed() && position.byte_offset == 22
        ));
        assert!(matches!(
            tokenizer.next().unwrap(),
            Event::EndTag { name: "root", .. }
        ));
        let event = tokenizer.next().unwrap();
        assert!(matches!(
            &event,
            Event::StartTag { name, attributes, empty, .. }
                if *name == "empty" && attributes.is_empty() && *empty
        ));
        assert_eq!(tokenizer.next().unwrap(), Event::Eof);
        assert_eq!(tokenizer.next().unwrap(), Event::Eof);
    }

    #[test]
    fn decodes_and_normalizes_text_and_attributes_without_allocating() {
        let mut tokenizer = make(b"<x a='one&amp;two&#x21;' b='a\tb\r\nc'>A&amp;B\r\nC</x>");
        let event = tokenizer.next().unwrap();
        assert!(matches!(
            &event,
            Event::StartTag { attributes, .. }
                if attributes.len() == 2
                    && attributes[0].decode_value::<32>(32).is_ok_and(|value| value.as_str() == "one&two!")
                    && attributes[1].decode_value::<32>(32).is_ok_and(|value| value.as_str() == "a b c")
        ));
        let event = tokenizer.next().unwrap();
        assert!(matches!(
            &event,
            Event::Text { value, .. }
                if value.as_str() == "A&B\nC" && !value.is_borrowed()
        ));
        assert!(matches!(tokenizer.next().unwrap(), Event::EndTag { .. }));
    }

    #[test]
    fn skips_valid_comments_and_xml_declarations() {
        let mut tokenizer =
            make(b"<?xml version='1.0' encoding=\"UTF-8\" standalone='yes'?><!--ok--><x/>");
        assert!(matches!(
            tokenizer.next().unwrap(),
            Event::StartTag { name: "x", .. }
        ));

        let mut tokenizer = make(b"<!--a-b--><!--c--><x/>");
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag { .. }));
    }

    #[test]
    fn handles_an_initial_utf8_bom_without_shifting_document_columns() {
        let declaration = b"<?xml version='1.0'?>";
        let input = b"\xEF\xBB\xBF<?xml version='1.0'?><root/>";
        let mut tokenizer = make(input);
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag {
            name: "root",
            position,
            ..
        } if position == Position::new(3 + declaration.len(), 1, declaration.len() + 1)));

        let mut tokenizer = make(b"\xEF\xBB\xBF<root/>");
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag {
            name: "root",
            position,
            ..
        } if position == Position::new(3, 1, 1)));

        let mut tokenizer = make("<root>\u{FEFF}</root>".as_bytes());
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag { .. }));
        assert!(
            matches!(tokenizer.next().unwrap(), Event::Text { value, .. }
            if value.as_str() == "\u{FEFF}")
        );

        let error = Tokenizer::new(b"\xEF\xBB\xBF\xFF", ParserConfig::default())
            .err()
            .unwrap();
        assert_eq!(error.code, ErrorCode::Utf8);
        assert_eq!(error.position, Position::new(3, 1, 1));
    }

    #[test]
    fn permits_well_formed_processing_instructions_only_when_configured() {
        let config = ParserConfigBuilder::default()
            .strict_processing_instructions(false)
            .build();
        let mut tokenizer = Tokenizer::new(b"<?target data ?><x/>", config.clone()).unwrap();
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag { .. }));

        let mut tokenizer = Tokenizer::new(b"<?target?><x/>", config).unwrap();
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag { .. }));
        assert_error(
            b"<?target data?>",
            ParserConfig::default(),
            ErrorCode::ProcessingInstruction,
        );
    }

    #[test]
    fn reports_utf8_xml_character_and_constructor_limit_errors() {
        let invalid_utf8 = Tokenizer::new(b"ok\xFF", ParserConfig::default())
            .err()
            .unwrap();
        assert_eq!(invalid_utf8.code, ErrorCode::Utf8);
        assert_eq!(invalid_utf8.position.byte_offset, 2);
        assert_eq!(invalid_utf8.position.column, 3);

        let mut tokenizer = make(b"<x>\0</x>");
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag { .. }));
        assert_eq!(tokenizer.next().unwrap_err().code, ErrorCode::Syntax);

        let invalid_config = ParserConfigBuilder::default()
            .max_text_bytes(MAX_TEXT_BYTES + 1)
            .build();
        assert_eq!(
            Tokenizer::new(b"", invalid_config).err().unwrap().code,
            ErrorCode::Resource
        );
        let too_large = ParserConfigBuilder::default().max_document_bytes(1).build();
        assert_eq!(
            Tokenizer::new(b"<x/>", too_large).err().unwrap().kind,
            ErrorKind::Resource
        );
    }

    #[test]
    fn enforces_document_token_attribute_and_text_budgets() {
        let token_limited = ParserConfigBuilder::default().max_tokens(0).build();
        assert_eq!(
            error(b"<x/>", token_limited).map(|error| error.kind),
            Some(ErrorKind::Resource)
        );

        let attr_limited = ParserConfigBuilder::default()
            .max_attributes_per_element(1)
            .build();
        assert_eq!(
            error(b"<x a='1' b='2'/>", attr_limited).map(|error| error.kind),
            Some(ErrorKind::Resource)
        );

        let text_limited = ParserConfigBuilder::default().max_text_bytes(2).build();
        assert_eq!(
            error(b"abc", text_limited.clone()).map(|error| error.kind),
            Some(ErrorKind::Resource)
        );
        assert_eq!(
            error(b"<x a='abc'/>", text_limited).map(|error| error.kind),
            Some(ErrorKind::Resource)
        );
    }

    #[test]
    fn rejects_unsupported_markup_and_unknown_general_entities() {
        for input in [
            &b"<!DOCTYPE x>"[..],
            &b"<![CDATA[x]]>"[..],
            &b"<!ENTITY x 'y'>"[..],
        ] {
            assert_error(input, ParserConfig::default(), ErrorCode::Syntax);
        }
        assert_eq!(
            error(b"<x>&general;</x>", ParserConfig::default()).map(|error| error.kind),
            Some(ErrorKind::Unsupported)
        );
        assert_error(
            b"<x>&#xZZ;</x>",
            ParserConfig::default(),
            ErrorCode::Reference,
        );
    }

    #[test]
    fn rejects_invalid_tags_attributes_comments_and_truncation() {
        let inputs: &[(&[u8], ErrorCode)] = &[
            (b"<1x/>", ErrorCode::Name),
            (b"<x a='1'b='2'/>", ErrorCode::Attribute),
            (b"<x a='1' a='2'/>", ErrorCode::Attribute),
            (b"<x a>", ErrorCode::Attribute),
            (b"<x a=unquoted/>", ErrorCode::Attribute),
            (b"<x a='<y'/>", ErrorCode::Attribute),
            (b"<x a='unterminated", ErrorCode::Eof),
            (b"</>", ErrorCode::Name),
            (b"</x", ErrorCode::Tag),
            (b"<!--unterminated", ErrorCode::Comment),
            (b"<!--bad--comment-->", ErrorCode::Comment),
            (b"text]]>tail", ErrorCode::Syntax),
        ];
        for (input, code) in inputs {
            assert_error(input, ParserConfig::default(), *code);
        }
    }

    #[test]
    fn validates_xml_declaration_position_order_values_and_truncation() {
        let inputs: &[&[u8]] = &[
            b"<?xml?>",
            b"<?xml ",
            b"<?xml version='1.1'?>",
            b"<?xml version=1.0?>",
            b"<?xml version '1.0'?>",
            b"<?xml version='1.0' encoding=''?>",
            b"<?xml version='1.0' encoding='UTF-16'?>",
            b"<?xml version='1.0' encoding='US-ASCII'?>",
            b"<?xml version='1.0' encoding='1UTF-8'?>",
            b"<?xml version='1.0' encoding='UTF+8'?>",
            b"<?xml encoding='UTF-8' version='1.0'?>",
            b"<?xml version='1.0' version='1.0'?>",
            b"<?xml version='1.0' encoding='-bad'?>",
            b"<?xml version='1.0' standalone='maybe'?>",
            b"<?xml version='1.0' standalone='yes' encoding='UTF-8'?>",
            b"<?xml version='1.0' extra='x'?>",
            b"<?xml version='1.0' encoding='UTF-8' standalone='yes' standalone='no'?>",
            b"<!--first--><?xml version='1.0'?>",
            b" <?xml version='1.0'?>",
            b"<?xml version='1.0'",
            b"<?xml version='1.0",
            b"<?xml version='1<0'?>",
            b"<?xml version='1&0'?>",
        ];
        for input in inputs {
            if *input == b"<?xml " {
                assert_eq!(
                    error(input, ParserConfig::default()).map(|error| error.code),
                    Some(ErrorCode::Eof)
                );
            } else {
                assert_error(input, ParserConfig::default(), ErrorCode::XmlDeclaration);
            }
        }
        let valid = b"<?xml version='1.0' encoding='UTF-8' standalone='no'?>";
        let mut tokenizer = make(valid);
        assert_eq!(tokenizer.next().unwrap(), Event::Eof);
        assert!(error(valid, ParserConfig::default()).is_none());

        let valid_lowercase = b"<?xml version='1.0' encoding='utf-8'?>";
        assert!(error(valid_lowercase, ParserConfig::default()).is_none());
    }

    #[test]
    fn rejects_malformed_processing_instructions_and_reserved_xml_targets() {
        let config = ParserConfigBuilder::default()
            .strict_processing_instructions(false)
            .build();
        for input in [
            &b"<??>"[..],
            &b"<?target?x?>"[..],
            &b"<?XML data?>"[..],
            &b"<?target data"[..],
        ] {
            assert_error(input, config.clone(), ErrorCode::ProcessingInstruction);
        }
        let mut tokenizer = Tokenizer::new(b"<?xml-stylesheet href='x'?>", config).unwrap();
        assert_eq!(tokenizer.next().unwrap(), Event::Eof);
    }

    #[test]
    fn maps_attribute_decode_errors_and_text_positions() {
        let mut tokenizer = make(b"<x a='&unknown;'/> ");
        assert_eq!(tokenizer.next().unwrap_err().kind, ErrorKind::Unsupported);

        let mut tokenizer = make(b"<x>\n&bad;</x>");
        assert!(matches!(tokenizer.next().unwrap(), Event::StartTag { .. }));
        let error = tokenizer.next().unwrap_err();
        assert_eq!(error.code, ErrorCode::Reference);
        assert_eq!(error.position.line, 2);
        assert_eq!(error.position.column, 2);

        let attribute = Attribute {
            name: "a",
            raw_value: b"&bad;",
            position: Position::new(5, 2, 3),
            value_position: Position::new(8, 2, 6),
        };
        let error = attribute.decode_value::<16>(16).unwrap_err();
        assert_eq!(error.position.byte_offset, 9);
        assert_eq!(error.position.line, 2);
        assert_eq!(error.position.column, 7);
        assert_eq!(
            attribute.decode_value::<32>(32).unwrap_err().code,
            ErrorCode::Reference
        );
    }

    #[test]
    fn covers_position_and_error_mapping_helpers() {
        assert_eq!(position_at("a\r\nbα".as_bytes(), 6), Position::new(6, 2, 3));
        assert_eq!(position_at(b"a\nb", 2), Position::new(2, 2, 1));
        assert_eq!(position_at(b"abc", 5), Position::new(5, 1, 1));
        assert_eq!(position_at(b"a\xFF", 2), Position::new(2, 1, 1));
        assert_eq!(position_at(b"\xEF\xBB\xBF<x", 4), Position::new(4, 1, 2));
        assert_eq!(
            position_after(Position::new(7, 3, 4), b"a\r\nb", 4),
            Position::new(11, 4, 2)
        );
        assert_eq!(
            position_after(Position::new(20, 5, 2), b"a\rb", 3),
            Position::new(23, 6, 2)
        );
        assert_eq!(
            position_after(Position::new(7, 3, 4), b"abc", 8),
            Position::new(15, 3, 4)
        );
        assert_eq!(
            position_after(Position::new(7, 3, 4), b"a\xFF", 2),
            Position::new(9, 3, 4)
        );

        let fallback = Position::new(4, 2, 2);
        assert_eq!(
            cursor_error(CursorError::InvalidOffset { offset: 99 }, fallback, b"a").position,
            fallback
        );
        assert_eq!(
            cursor_error(CursorError::InvalidOffset { offset: 1 }, fallback, b"ab").position,
            Position::new(1, 1, 2)
        );

        let errors = [
            NameError::InvalidStartChar { offset: 1 },
            NameError::Eof { offset: 1 },
            NameError::CursorError(CursorError::InvalidUtf8 { offset: 1 }),
            NameError::CursorError(CursorError::InvalidXmlChar { offset: 1 }),
            NameError::CursorError(CursorError::InvalidOffset { offset: 1 }),
        ];
        let codes = [
            ErrorCode::Name,
            ErrorCode::Eof,
            ErrorCode::Utf8,
            ErrorCode::Syntax,
            ErrorCode::Syntax,
        ];
        for (error, expected) in errors.into_iter().zip(codes) {
            assert_eq!(name_error(error, ErrorCode::Name, b"ab").code, expected);
        }

        let position = Position::new(10, 4, 3);
        assert_eq!(
            cursor_error(CursorError::InvalidUtf8 { offset: 1 }, position, b"ab").code,
            ErrorCode::Utf8
        );
        assert_eq!(
            cursor_error(CursorError::InvalidXmlChar { offset: 1 }, position, b"ab").code,
            ErrorCode::Syntax
        );
        let mut cursor = Cursor::new(b"abc").unwrap();
        assert_eq!(advance_cursor(&mut cursor, 1, b"abc"), Ok(()));
        assert_eq!(cursor.position().byte_offset, 1);
        assert_eq!(
            advance_cursor(&mut cursor, 3, b"abc").unwrap_err().code,
            ErrorCode::Syntax
        );
        assert_eq!(
            consume_cursor(&mut Cursor::new(b"\0").unwrap(), b"\0")
                .unwrap_err()
                .code,
            ErrorCode::Syntax
        );
        assert_eq!(
            input_slice(b"abcd", 1..3, position, ErrorCode::Syntax),
            Ok(&b"bc"[..])
        );
        assert_eq!(
            input_slice(b"abcd", 3..5, position, ErrorCode::Syntax)
                .unwrap_err()
                .code,
            ErrorCode::Syntax
        );
        let mut attributes = [Attribute::EMPTY; 1];
        assert!(attribute_slot(&mut attributes, 0, position).is_ok());
        assert_eq!(
            attribute_slot(&mut attributes, 1, position)
                .unwrap_err()
                .kind,
            ErrorKind::Resource
        );
        let config = ParserConfig::default();
        let mut inconsistent_source = Tokenizer {
            input: b"",
            document_start_offset: 0,
            cursor: Cursor::new(b"version='1.0'").unwrap(),
            budget: ResourceBudget::new(config.clone()),
            max_text_bytes: config.max_text_bytes,
            strict_processing_instructions: config.strict_processing_instructions,
            at_document_start: true,
            attributes: [Attribute::EMPTY; MAX_ATTRIBUTES],
            attribute_count: 0,
        };
        assert_eq!(
            inconsistent_source
                .parse_declaration_attribute(Position::default())
                .unwrap_err()
                .code,
            ErrorCode::XmlDeclaration
        );
        assert_eq!(
            read_quote(
                Err(CursorError::InvalidXmlChar { offset: 0 }),
                position,
                b"a",
                ErrorCode::Attribute
            )
            .unwrap_err()
            .position,
            Position::new(0, 1, 1)
        );
        assert_eq!(
            read_quote(Ok(None), position, b"", ErrorCode::XmlDeclaration)
                .unwrap_err()
                .code,
            ErrorCode::XmlDeclaration
        );
        assert_eq!(
            read_quote(Ok(Some('x')), position, b"", ErrorCode::Attribute)
                .unwrap_err()
                .code,
            ErrorCode::Attribute
        );
    }

    #[test]
    fn maps_each_text_error_to_a_source_position_and_category() {
        let base = Position::new(20, 5, 2);
        let raw = b"a\rb";
        let cases = [
            (
                TextError::Entity(crate::entity::EntityError::MissingSemicolon { offset: 1 }),
                ErrorCode::Reference,
            ),
            (
                TextError::Entity(crate::entity::EntityError::UnknownEntity { offset: 1 }),
                ErrorCode::Reference,
            ),
            (TextError::InvalidUtf8 { offset: 1 }, ErrorCode::Utf8),
            (
                TextError::InvalidXmlCharacter { offset: 1 },
                ErrorCode::Syntax,
            ),
        ];
        for (error, code) in cases {
            let mapped = text_error(error, base, raw);
            assert_eq!(mapped.code, code);
            assert_eq!(mapped.position.byte_offset, 21);
            assert_eq!(mapped.position.line, 5);
            assert_eq!(mapped.position.column, 3);
        }
        assert_eq!(
            text_error(
                TextError::Entity(crate::entity::EntityError::UnknownEntity { offset: 1 }),
                base,
                raw
            )
            .kind,
            ErrorKind::Unsupported
        );
        let capacity = text_error(TextError::CapacityOverflow, base, raw);
        assert_eq!(capacity.code, ErrorCode::Resource);
        assert_eq!(capacity.position, base);

        let entities = [
            crate::entity::EntityError::MissingSemicolon { offset: 0 },
            crate::entity::EntityError::MissingDigits { offset: 0 },
            crate::entity::EntityError::InvalidDigit { offset: 0 },
            crate::entity::EntityError::IntegerOverflow { offset: 0 },
            crate::entity::EntityError::InvalidXmlCharacter { offset: 0 },
            crate::entity::EntityError::UnknownEntity { offset: 0 },
            crate::entity::EntityError::Truncated { offset: 0 },
        ];
        for entity in entities {
            assert_eq!(entity_error_offset(entity), 0);
        }
    }

    #[test]
    fn covers_invalid_character_paths_in_markup_and_declarations() {
        for input in [
            &b"<x\0/>"[..],
            &b"<x a=\0/>"[..],
            &b"<x a='\0'/>"[..],
            &b"<?xml version=\0?>"[..],
            &b"<?xml version='1\0'?>"[..],
        ] {
            let error = error(input, ParserConfig::default()).unwrap();
            assert_eq!(error.code, ErrorCode::Syntax, "input: {input:?}");
        }
        let config = ParserConfigBuilder::default()
            .strict_processing_instructions(false)
            .build();
        assert_eq!(
            error(b"<?target\0?>", config).unwrap().code,
            ErrorCode::Syntax
        );
    }
}
