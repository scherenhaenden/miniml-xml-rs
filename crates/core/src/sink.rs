#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkError {
    RejectedByApplication,
}

/// A string borrowed from input or decoded into a temporary fixed buffer.
/// Retain `Borrowed` for the input lifetime; copy `Decoded` during the call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextValue<'input, 'call> {
    Borrowed(&'input str),
    Decoded(&'call str),
}

impl<'input> TextValue<'input, '_> {
    /// Returns text for the duration of this value's borrow, without allocation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Decoded(s) => s,
        }
    }
    /// Returns only a string whose lifetime is tied to the original input.
    pub fn borrowed(&self) -> Option<&'input str> {
        match self {
            Self::Borrowed(s) => Some(s),
            Self::Decoded(_) => None,
        }
    }
}

/// TargetSink lifetime-safe strategy receiving validated element/attribute content and node IDs.
pub trait TargetSink<'a> {
    /// Called when an element node begins.
    fn begin_element(&mut self, node_id: u16) -> Result<(), SinkError> {
        let _ = node_id;
        Ok(())
    }

    /// Called when an element node ends.
    fn end_element(&mut self, node_id: u16) -> Result<(), SinkError> {
        let _ = node_id;
        Ok(())
    }

    /// Called to provide string content for an element.
    fn element_string_content(
        &mut self,
        node_id: u16,
        content: TextValue<'a, '_>,
    ) -> Result<(), SinkError> {
        let _ = (node_id, content);
        Ok(())
    }

    /// Called to provide integer content for an element.
    fn element_integer_content(&mut self, node_id: u16, content: i64) -> Result<(), SinkError> {
        let _ = (node_id, content);
        Ok(())
    }

    /// Called to provide a string attribute for the current element.
    fn attribute_string(
        &mut self,
        node_id: u16,
        name: &'a str,
        content: TextValue<'a, '_>,
    ) -> Result<(), SinkError> {
        let _ = (node_id, name, content);
        Ok(())
    }

    /// Called to provide an integer attribute for the current element.
    fn attribute_integer(
        &mut self,
        node_id: u16,
        name: &'a str,
        content: i64,
    ) -> Result<(), SinkError> {
        let _ = (node_id, name, content);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_and_decoded_strings_have_distinct_retention_contracts() {
        let input = "source";
        let borrowed = TextValue::Borrowed(input);
        assert_eq!(borrowed.as_str(), input);
        assert_eq!(borrowed.borrowed(), Some(input));
        let buffer = *b"dec";
        let decoded = TextValue::Decoded(core::str::from_utf8(&buffer).unwrap());
        assert_eq!(decoded.as_str(), "dec");
        assert_eq!(decoded.borrowed(), None);
    }

    struct DummySink;

    impl<'a> TargetSink<'a> for DummySink {}

    #[test]
    fn test_default_methods() {
        let mut sink = DummySink;

        assert_eq!(sink.begin_element(0), Ok(()));
        assert_eq!(sink.end_element(0), Ok(()));
        assert_eq!(
            sink.element_string_content(0, TextValue::Borrowed("content")),
            Ok(())
        );
        assert_eq!(sink.element_integer_content(0, 123), Ok(()));
        assert_eq!(
            sink.attribute_string(0, "attr", TextValue::Borrowed("val")),
            Ok(())
        );
        assert_eq!(sink.attribute_integer(0, "attr", 123), Ok(()));
    }

    struct RejectingSink;

    impl<'a> TargetSink<'a> for RejectingSink {
        fn begin_element(&mut self, _node_id: u16) -> Result<(), SinkError> {
            Err(SinkError::RejectedByApplication)
        }
    }

    #[test]
    fn test_rejecting_sink() {
        let mut sink = RejectingSink;
        assert_eq!(sink.begin_element(0), Err(SinkError::RejectedByApplication));
    }
}
