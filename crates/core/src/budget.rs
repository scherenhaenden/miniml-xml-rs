use crate::config::{ParserConfig, MAX_ATTRIBUTES, MAX_DEPTH};
use crate::error::{ErrorCode, ErrorKind, ParseError, Position};

/// Tracks resource consumption against configured limits.
/// Limits are enforced via checked arithmetic; panics are never used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBudget {
    config: ParserConfig,
    tokens: usize,
    depth: usize,
    attributes: usize,
    children: usize,
    occurrences: usize,
}

impl ResourceBudget {
    pub fn new(config: ParserConfig) -> Self {
        Self {
            config,
            tokens: 0,
            depth: 0,
            attributes: 0,
            children: 0,
            occurrences: 0,
        }
    }

    /// Verifies the document byte count does not exceed limits.
    pub fn check_document_bytes(&self, bytes: usize, position: Position) -> Result<(), ParseError> {
        if bytes > self.config.max_document_bytes {
            Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                position,
            ))
        } else {
            Ok(())
        }
    }

    /// Verifies the text byte count does not exceed limits.
    pub fn check_text_bytes(&self, bytes: usize, position: Position) -> Result<(), ParseError> {
        if bytes > self.config.max_text_bytes {
            Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                position,
            ))
        } else {
            Ok(())
        }
    }

    /// Consumes one token.
    pub fn consume_token(&mut self, position: Position) -> Result<(), ParseError> {
        let next = self
            .tokens
            .checked_add(1)
            .ok_or_else(|| ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position))?;
        if next > self.config.max_tokens {
            return Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                position,
            ));
        }
        self.tokens = next;
        Ok(())
    }

    /// Enters a new depth level (e.g., element nesting).
    pub fn enter_depth(&mut self, position: Position) -> Result<(), ParseError> {
        let next = self
            .depth
            .checked_add(1)
            .ok_or_else(|| ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position))?;
        if next > self.config.max_depth.min(MAX_DEPTH) {
            return Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                position,
            ));
        }
        self.depth = next;
        Ok(())
    }

    /// Exits a depth level.
    pub fn exit_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// Clears per-element counts.
    pub fn reset_element_counters(&mut self) {
        self.attributes = 0;
        self.children = 0;
    }

    /// Consumes one attribute.
    pub fn consume_attribute(&mut self, position: Position) -> Result<(), ParseError> {
        let next = self
            .attributes
            .checked_add(1)
            .ok_or_else(|| ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position))?;
        if next > self.config.max_attributes_per_element.min(MAX_ATTRIBUTES) {
            return Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                position,
            ));
        }
        self.attributes = next;
        Ok(())
    }

    /// Consumes one child element.
    pub fn consume_child(&mut self, position: Position) -> Result<(), ParseError> {
        let next = self
            .children
            .checked_add(1)
            .ok_or_else(|| ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position))?;
        if next > self.config.max_children_per_element {
            return Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                position,
            ));
        }
        self.children = next;
        Ok(())
    }

    /// Consumes one occurrence of an element for schema maxOccurs bounding.
    pub fn consume_occurrence(&mut self, position: Position) -> Result<(), ParseError> {
        let next = self
            .occurrences
            .checked_add(1)
            .ok_or_else(|| ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position))?;
        if next > self.config.max_occurrences {
            return Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                position,
            ));
        }
        self.occurrences = next;
        Ok(())
    }

    pub fn reset_occurrences(&mut self) {
        self.occurrences = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_overflows_preserve_counts_and_positions() {
        let pos = Position::new(19, 3, 8);
        let mut budget = ResourceBudget::new(ParserConfig::default());
        budget.depth = usize::MAX;
        budget.attributes = usize::MAX;
        budget.children = usize::MAX;
        budget.occurrences = usize::MAX;
        let expected = Err(ParseError::new(
            ErrorKind::Resource,
            ErrorCode::Resource,
            pos,
        ));
        assert_eq!(budget.enter_depth(pos), expected);
        assert_eq!(budget.consume_attribute(pos), expected);
        assert_eq!(budget.consume_child(pos), expected);
        assert_eq!(budget.consume_occurrence(pos), expected);
        assert_eq!(budget.depth, usize::MAX);
        assert_eq!(budget.attributes, usize::MAX);
        assert_eq!(budget.children, usize::MAX);
        assert_eq!(budget.occurrences, usize::MAX);
    }

    #[test]
    fn storage_caps_apply_even_to_unvalidated_configuration() {
        let pos = Position::default();
        let mut budget = ResourceBudget::new(
            ParserConfig::builder()
                .max_depth(usize::MAX)
                .max_attributes_per_element(usize::MAX)
                .build(),
        );
        for _ in 0..MAX_DEPTH {
            assert_eq!(budget.enter_depth(pos), Ok(()));
        }
        assert!(budget.enter_depth(pos).is_err());
        for _ in 0..MAX_ATTRIBUTES {
            assert_eq!(budget.consume_attribute(pos), Ok(()));
        }
        assert!(budget.consume_attribute(pos).is_err());
    }

    #[test]
    fn test_consume_token_exact_and_overflow() {
        let config = ParserConfig::builder().max_tokens(1).build();
        let mut budget = ResourceBudget::new(config);
        let pos = Position::default();

        assert!(budget.consume_token(pos).is_ok());
        assert!(budget.consume_token(pos).is_err());
    }

    #[test]
    fn test_enter_exit_depth() {
        let config = ParserConfig::builder().max_depth(1).build();
        let mut budget = ResourceBudget::new(config);
        let pos = Position::default();

        assert!(budget.enter_depth(pos).is_ok());
        assert!(budget.enter_depth(pos).is_err());
        budget.exit_depth();
        assert!(budget.enter_depth(pos).is_ok());
    }

    #[test]
    fn test_zero_limits() {
        let config = ParserConfig::builder()
            .max_attributes_per_element(0)
            .max_children_per_element(0)
            .max_occurrences(0)
            .build();
        let mut budget = ResourceBudget::new(config);
        let pos = Position::default();

        assert!(budget.consume_attribute(pos).is_err());
        assert!(budget.consume_child(pos).is_err());
        assert!(budget.consume_occurrence(pos).is_err());
    }

    #[test]
    fn test_checked_add_overflow() {
        let config = ParserConfig::builder().max_tokens(usize::MAX).build();
        let mut budget = ResourceBudget::new(config);
        budget.tokens = usize::MAX;

        let pos = Position::default();
        assert!(budget.consume_token(pos).is_err());
    }

    #[test]
    fn test_check_bytes() {
        let config = ParserConfig::builder()
            .max_document_bytes(10)
            .max_text_bytes(5)
            .build();
        let budget = ResourceBudget::new(config);
        let pos = Position::default();

        assert!(budget.check_document_bytes(10, pos).is_ok());
        assert!(budget.check_document_bytes(11, pos).is_err());

        assert!(budget.check_text_bytes(5, pos).is_ok());
        assert!(budget.check_text_bytes(6, pos).is_err());
    }

    #[test]
    fn test_reset_counters() {
        let config = ParserConfig::builder()
            .max_attributes_per_element(1)
            .max_children_per_element(1)
            .max_occurrences(1)
            .build();
        let mut budget = ResourceBudget::new(config);
        let pos = Position::default();

        assert!(budget.consume_attribute(pos).is_ok());
        assert!(budget.consume_child(pos).is_ok());
        assert!(budget.consume_occurrence(pos).is_ok());

        budget.reset_element_counters();
        budget.reset_occurrences();

        assert!(budget.consume_attribute(pos).is_ok());
        assert!(budget.consume_child(pos).is_ok());
        assert!(budget.consume_occurrence(pos).is_ok());
    }

    #[test]
    fn test_exit_depth_at_zero() {
        let config = ParserConfig::default();
        let mut budget = ResourceBudget::new(config);
        budget.exit_depth(); // Should not underflow/panic
        assert_eq!(budget.depth, 0);
    }
}
