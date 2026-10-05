#[cfg(test)]
extern crate alloc;

use core::fmt;

/// Represents a source position within an XML document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// 0-based byte offset from the start of the input.
    pub byte_offset: usize,
    /// 1-based line number (CRLF treated as one newline).
    pub line: usize,
    /// 1-based Unicode-scalar column.
    pub column: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            byte_offset: 0,
            line: 1,
            column: 1,
        }
    }
}

impl Position {
    pub fn new(byte_offset: usize, line: usize, column: usize) -> Self {
        Self {
            byte_offset,
            line,
            column,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// High-level categorization of parse errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Syntax,
    Schema,
    Attribute,
    Conversion,
    Facet,
    Resource,
    Unsupported,
    Internal,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Syntax => "Syntax",
            Self::Schema => "Schema",
            Self::Attribute => "Attribute",
            Self::Conversion => "Conversion",
            Self::Facet => "Facet",
            Self::Resource => "Resource",
            Self::Unsupported => "Unsupported",
            Self::Internal => "Internal",
        };
        f.write_str(s)
    }
}

/// Detailed identification of the parsing failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    Utf8,
    Eof,
    Name,
    Tag,
    Attribute,
    Entity,
    Reference,
    Comment,
    ProcessingInstruction,
    XmlDeclaration,
    Syntax,
    Mismatch,
    MultipleRoot,
    MixedContent,
    Schema,
    Required,
    Order,
    Occurrence,
    IntegerOverflow,
    Resource,
    Sink,
    Version,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Utf8 => "Utf8",
            Self::Eof => "Eof",
            Self::Name => "Name",
            Self::Tag => "Tag",
            Self::Attribute => "Attribute",
            Self::Entity => "Entity",
            Self::Reference => "Reference",
            Self::Comment => "Comment",
            Self::ProcessingInstruction => "ProcessingInstruction",
            Self::XmlDeclaration => "XmlDeclaration",
            Self::Syntax => "Syntax",
            Self::Mismatch => "Mismatch",
            Self::MultipleRoot => "MultipleRoot",
            Self::MixedContent => "MixedContent",
            Self::Schema => "Schema",
            Self::Required => "Required",
            Self::Order => "Order",
            Self::Occurrence => "Occurrence",
            Self::IntegerOverflow => "IntegerOverflow",
            Self::Resource => "Resource",
            Self::Sink => "Sink",
            Self::Version => "Version",
        };
        f.write_str(s)
    }
}

/// A structured error returned when parsing fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub code: ErrorCode,
    pub position: Position,
}

impl ParseError {
    pub fn new(kind: ErrorKind, code: ErrorCode, position: Position) -> Self {
        Self {
            kind,
            code,
            position,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: parse error at {}",
            self.kind, self.code, self.position
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use alloc::string::ToString;

    #[test]
    fn test_position_display() {
        let pos = Position::new(42, 3, 15);
        assert_eq!(pos.to_string(), "3:15");
        let def = Position::default();
        assert_eq!(def.to_string(), "1:1");
    }

    #[test]
    fn test_errorkind_display() {
        assert_eq!(ErrorKind::Syntax.to_string(), "Syntax");
        assert_eq!(ErrorKind::Schema.to_string(), "Schema");
        assert_eq!(ErrorKind::Attribute.to_string(), "Attribute");
        assert_eq!(ErrorKind::Conversion.to_string(), "Conversion");
        assert_eq!(ErrorKind::Facet.to_string(), "Facet");
        assert_eq!(ErrorKind::Resource.to_string(), "Resource");
        assert_eq!(ErrorKind::Unsupported.to_string(), "Unsupported");
        assert_eq!(ErrorKind::Internal.to_string(), "Internal");
    }

    #[test]
    fn test_errorcode_display() {
        assert_eq!(ErrorCode::Utf8.to_string(), "Utf8");
        assert_eq!(ErrorCode::Eof.to_string(), "Eof");
        assert_eq!(ErrorCode::Name.to_string(), "Name");
        assert_eq!(ErrorCode::Tag.to_string(), "Tag");
        assert_eq!(ErrorCode::Attribute.to_string(), "Attribute");
        assert_eq!(ErrorCode::Entity.to_string(), "Entity");
        assert_eq!(ErrorCode::Reference.to_string(), "Reference");
        assert_eq!(ErrorCode::Comment.to_string(), "Comment");
        assert_eq!(
            ErrorCode::ProcessingInstruction.to_string(),
            "ProcessingInstruction"
        );
        assert_eq!(ErrorCode::XmlDeclaration.to_string(), "XmlDeclaration");
        assert_eq!(ErrorCode::Syntax.to_string(), "Syntax");
        assert_eq!(ErrorCode::Mismatch.to_string(), "Mismatch");
        assert_eq!(ErrorCode::MultipleRoot.to_string(), "MultipleRoot");
        assert_eq!(ErrorCode::MixedContent.to_string(), "MixedContent");
        assert_eq!(ErrorCode::Schema.to_string(), "Schema");
        assert_eq!(ErrorCode::Required.to_string(), "Required");
        assert_eq!(ErrorCode::Order.to_string(), "Order");
        assert_eq!(ErrorCode::Occurrence.to_string(), "Occurrence");
        assert_eq!(ErrorCode::IntegerOverflow.to_string(), "IntegerOverflow");
        assert_eq!(ErrorCode::Resource.to_string(), "Resource");
        assert_eq!(ErrorCode::Sink.to_string(), "Sink");
        assert_eq!(ErrorCode::Version.to_string(), "Version");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::new(ErrorKind::Syntax, ErrorCode::Eof, Position::new(100, 5, 20));
        assert_eq!(err.to_string(), "[Syntax] Eof: parse error at 5:20");
    }
}
