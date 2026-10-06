use crate::config::MAX_DEPTH;
use crate::error::{ErrorCode, ErrorKind, ParseError, Position};

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct ElementStack<'a> {
    names: [&'a str; MAX_DEPTH],
    positions: [Position; MAX_DEPTH],
    pub(crate) schema_node_ids: [u16; MAX_DEPTH],
    pub(crate) child_indices: [usize; MAX_DEPTH],
    pub(crate) child_occurrences: [usize; MAX_DEPTH],
    pub(crate) child_counts: [usize; MAX_DEPTH],
    len: usize,
    max_depth: usize,
    has_root: bool,
}

#[allow(dead_code)]
impl<'a> ElementStack<'a> {
    pub(crate) fn new(max_depth: usize) -> Self {
        Self {
            names: [""; MAX_DEPTH],
            positions: [Position::default(); MAX_DEPTH],
            schema_node_ids: [0; MAX_DEPTH],
            child_indices: [0; MAX_DEPTH],
            child_occurrences: [0; MAX_DEPTH],
            child_counts: [0; MAX_DEPTH],
            len: 0,
            max_depth: max_depth.min(MAX_DEPTH),
            has_root: false,
        }
    }

    pub(crate) fn push(
        &mut self,
        name: &'a str,
        pos: Position,
        node_id: u16,
    ) -> Result<(), ParseError> {
        if self.len == 0 && self.has_root {
            return Err(ParseError::new(
                ErrorKind::Syntax,
                ErrorCode::MultipleRoot,
                pos,
            ));
        }

        if self.len >= self.max_depth {
            return Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                pos,
            ));
        }

        self.names[self.len] = name;
        self.positions[self.len] = pos;
        self.schema_node_ids[self.len] = node_id;
        self.child_indices[self.len] = 0;
        self.child_occurrences[self.len] = 0;
        self.child_counts[self.len] = 0;
        self.len += 1;
        self.has_root = true;

        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn top_name(&self) -> Option<&'a str> {
        self.len.checked_sub(1).map(|index| self.names[index])
    }

    pub(crate) fn pop(&mut self, name: &str, pos: Position) -> Result<(), ParseError> {
        if self.len == 0 {
            return Err(ParseError::new(ErrorKind::Syntax, ErrorCode::Mismatch, pos));
        }

        let top_name = self.names[self.len - 1];
        if top_name != name {
            return Err(ParseError::new(ErrorKind::Syntax, ErrorCode::Mismatch, pos));
        }

        self.len -= 1;
        Ok(())
    }

    pub(crate) fn self_close(&mut self, _name: &'a str, pos: Position) -> Result<(), ParseError> {
        if self.len == 0 && self.has_root {
            return Err(ParseError::new(
                ErrorKind::Syntax,
                ErrorCode::MultipleRoot,
                pos,
            ));
        }

        // A self-closing element still occupies one depth level, even though
        // it does not need a slot in the open-element stack.
        if self.len >= self.max_depth {
            return Err(ParseError::new(
                ErrorKind::Resource,
                ErrorCode::Resource,
                pos,
            ));
        }

        self.has_root = true;
        Ok(())
    }

    pub(crate) fn finish(&self, pos: Position) -> Result<(), ParseError> {
        if !self.has_root {
            return Err(ParseError::new(ErrorKind::Syntax, ErrorCode::Eof, pos));
        }

        if self.len > 0 {
            return Err(ParseError::new(ErrorKind::Syntax, ErrorCode::Eof, pos));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_element() {
        let mut stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);
        assert!(stack.self_close("root", pos).is_ok());
        assert!(stack.finish(pos).is_ok());
    }

    #[test]
    fn test_nesting() {
        let mut stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);
        assert!(stack.push("root", pos, 0).is_ok());
        assert!(stack.push("child", pos, 1).is_ok());
        assert!(stack.pop("child", pos).is_ok());
        assert!(stack.pop("root", pos).is_ok());
        assert!(stack.finish(pos).is_ok());
    }

    #[test]
    fn test_matching_close() {
        let mut stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);
        assert!(stack.push("root", pos, 0).is_ok());
        assert!(stack.pop("root", pos).is_ok());
        assert!(stack.finish(pos).is_ok());
    }

    #[test]
    fn test_mismatched_close() {
        let mut stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);
        assert!(stack.push("root", pos, 0).is_ok());
        assert!(stack.push("child", pos, 1).is_ok());

        let err = stack.pop("wrong", pos).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert_eq!(err.code, ErrorCode::Mismatch);

        // Stack state must be unchanged
        assert_eq!(stack.len, 2);
    }

    #[test]
    fn test_unmatched_close() {
        let mut stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);

        let err = stack.pop("unmatched", pos).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert_eq!(err.code, ErrorCode::Mismatch);
    }

    #[test]
    fn test_multiple_roots() {
        let mut stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);
        assert!(stack.push("root1", pos, 0).is_ok());
        assert!(stack.pop("root1", pos).is_ok());

        let err = stack.push("root2", pos, 1).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert_eq!(err.code, ErrorCode::MultipleRoot);

        // State unchanged
        assert_eq!(stack.len, 0);

        let mut stack2 = ElementStack::new(5);
        assert!(stack2.self_close("root1", pos).is_ok());
        let err2 = stack2.self_close("root2", pos).unwrap_err();
        assert_eq!(err2.kind, ErrorKind::Syntax);
        assert_eq!(err2.code, ErrorCode::MultipleRoot);
    }

    #[test]
    fn test_missing_root() {
        let stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);

        let err = stack.finish(pos).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert_eq!(err.code, ErrorCode::Eof);
    }

    #[test]
    fn test_incomplete_input() {
        let mut stack = ElementStack::new(5);
        let pos = Position::new(0, 1, 1);
        assert!(stack.push("root", pos, 0).is_ok());

        let err = stack.finish(pos).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Syntax);
        assert_eq!(err.code, ErrorCode::Eof);
    }

    #[test]
    fn test_exact_depth() {
        let mut stack = ElementStack::new(3);
        let pos = Position::new(0, 1, 1);
        assert!(stack.push("a", pos, 0).is_ok());
        assert!(stack.push("b", pos, 1).is_ok());
        assert!(stack.push("c", pos, 2).is_ok());
        assert!(stack.pop("c", pos).is_ok());
        assert!(stack.pop("b", pos).is_ok());
        assert!(stack.pop("a", pos).is_ok());
        assert!(stack.finish(pos).is_ok());
    }

    #[test]
    fn test_depth_overflow() {
        let mut stack = ElementStack::new(2);
        let pos = Position::new(0, 1, 1);
        assert!(stack.push("a", pos, 0).is_ok());
        assert!(stack.push("b", pos, 1).is_ok());

        let err = stack.push("c", pos, 2).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Resource);
        assert_eq!(err.code, ErrorCode::Resource);

        // Stack state unchanged
        assert_eq!(stack.len, 2);
    }

    #[test]
    fn self_closing_elements_respect_depth_limits_atomically() {
        let pos = Position::new(4, 1, 5);

        let mut no_depth = ElementStack::new(0);
        let error = no_depth.push("root", pos, 0).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Resource);
        assert_eq!(error.code, ErrorCode::Resource);
        assert_eq!(error.position, pos);
        assert_eq!(no_depth.len, 0);
        assert!(!no_depth.has_root);

        let error = no_depth.self_close("root", pos).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Resource);
        assert_eq!(error.code, ErrorCode::Resource);
        assert_eq!(error.position, pos);
        assert!(!no_depth.has_root);
        assert_eq!(no_depth.len, 0);

        let mut nested_at_limit = ElementStack::new(1);
        nested_at_limit.push("root", pos, 0).unwrap();
        let error = nested_at_limit.self_close("child", pos).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Resource);
        assert_eq!(error.code, ErrorCode::Resource);
        assert_eq!(error.position, pos);
        assert_eq!(nested_at_limit.len, 1);
        assert_eq!(nested_at_limit.names[0], "root");
        assert_eq!(nested_at_limit.positions[0], pos);
    }

    #[test]
    fn configured_depth_cannot_exceed_fixed_stack_storage() {
        let pos = Position::new(0, 1, 1);
        let mut stack = ElementStack::new(MAX_DEPTH + 1);

        for _ in 0..MAX_DEPTH {
            stack.push("item", pos, 0).unwrap();
        }
        let error = stack.push("overflow", pos, 0).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Resource);
        assert_eq!(error.code, ErrorCode::Resource);
        assert_eq!(stack.len, MAX_DEPTH);

        for _ in 0..MAX_DEPTH {
            stack.pop("item", pos).unwrap();
        }
        stack.finish(pos).unwrap();
    }
}
