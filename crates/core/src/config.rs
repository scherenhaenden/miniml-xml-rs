/// Configuration for the parser.
/// Limits control resource exhaustion and are checked before expensive work.
/// Zero limits have explicit deterministic meanings (e.g., max_attributes_per_element = 0 means attributes are forbidden).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserConfig {
    pub max_document_bytes: usize,
    pub max_depth: usize,
    pub max_attributes_per_element: usize,
    pub max_children_per_element: usize,
    pub max_text_bytes: usize,
    pub max_occurrences: usize,
    pub max_tokens: usize,
    pub strict_processing_instructions: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_document_bytes: 4096,
            max_depth: 16,
            max_attributes_per_element: 8,
            max_children_per_element: 64,
            max_text_bytes: 256,
            max_occurrences: 32,
            max_tokens: 4096,
            strict_processing_instructions: true,
        }
    }
}

impl ParserConfig {
    /// Returns a new builder for configuring the parser.
    pub fn builder() -> ParserConfigBuilder {
        ParserConfigBuilder::default()
    }
}

/// Builder for `ParserConfig`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParserConfigBuilder {
    config: ParserConfig,
}

impl ParserConfigBuilder {
    pub fn max_document_bytes(mut self, limit: usize) -> Self {
        self.config.max_document_bytes = limit;
        self
    }

    pub fn max_depth(mut self, limit: usize) -> Self {
        self.config.max_depth = limit;
        self
    }

    pub fn max_attributes_per_element(mut self, limit: usize) -> Self {
        self.config.max_attributes_per_element = limit;
        self
    }

    pub fn max_children_per_element(mut self, limit: usize) -> Self {
        self.config.max_children_per_element = limit;
        self
    }

    pub fn max_text_bytes(mut self, limit: usize) -> Self {
        self.config.max_text_bytes = limit;
        self
    }

    pub fn max_occurrences(mut self, limit: usize) -> Self {
        self.config.max_occurrences = limit;
        self
    }

    pub fn max_tokens(mut self, limit: usize) -> Self {
        self.config.max_tokens = limit;
        self
    }

    pub fn strict_processing_instructions(mut self, strict: bool) -> Self {
        self.config.strict_processing_instructions = strict;
        self
    }

    pub fn build(self) -> ParserConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ParserConfig::default();
        assert_eq!(config.max_document_bytes, 4096);
        assert_eq!(config.max_depth, 16);
        assert_eq!(config.max_attributes_per_element, 8);
        assert_eq!(config.max_children_per_element, 64);
        assert_eq!(config.max_text_bytes, 256);
        assert_eq!(config.max_occurrences, 32);
        assert_eq!(config.max_tokens, 4096);
        assert!(config.strict_processing_instructions);
    }

    #[test]
    fn test_builder() {
        let config = ParserConfig::builder()
            .max_document_bytes(1024)
            .max_depth(5)
            .max_attributes_per_element(2)
            .max_children_per_element(10)
            .max_text_bytes(50)
            .max_occurrences(4)
            .max_tokens(100)
            .strict_processing_instructions(false)
            .build();

        assert_eq!(config.max_document_bytes, 1024);
        assert_eq!(config.max_depth, 5);
        assert_eq!(config.max_attributes_per_element, 2);
        assert_eq!(config.max_children_per_element, 10);
        assert_eq!(config.max_text_bytes, 50);
        assert_eq!(config.max_occurrences, 4);
        assert_eq!(config.max_tokens, 100);
        assert!(!config.strict_processing_instructions);
    }

    #[test]
    fn test_builder_zero() {
        let config = ParserConfig::builder()
            .max_attributes_per_element(0)
            .build();
        assert_eq!(config.max_attributes_per_element, 0);
    }
}
