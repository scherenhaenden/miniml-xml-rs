#![no_std]
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkError {
    RejectedByApplication,
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
    fn element_string_content(&mut self, node_id: u16, content: &'a str) -> Result<(), SinkError> {
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
        content: &'a str,
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

    struct DummySink;

    impl<'a> TargetSink<'a> for DummySink {}

    #[test]
    fn test_default_methods() {
        let mut sink = DummySink;

        assert_eq!(sink.begin_element(0), Ok(()));
        assert_eq!(sink.end_element(0), Ok(()));
        assert_eq!(sink.element_string_content(0, "content"), Ok(()));
        assert_eq!(sink.element_integer_content(0, 123), Ok(()));
        assert_eq!(sink.attribute_string(0, "attr", "val"), Ok(()));
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
