use crate::config::{ParserConfig, MAX_TEXT_BYTES};
use crate::convert::{self, ConvertError};
use crate::cursor::Cursor;
use crate::error::{ErrorCode, ErrorKind, ParseError, Position};
use crate::schema::{
    AttributeType, ContentType, Schema, SchemaNode, SchemaNodeId, MAX_SCHEMA_ATTRS,
};
use crate::sink::{SinkError, TargetSink, TextValue};
use crate::state::ElementStack;
use crate::text::Text;
use crate::tokenizer::{Attribute, Event, Tokenizer};

/// Validates an XML document against a prevalidated static schema and sends
/// typed values to the sink as the document is consumed.
///
/// A successful return means the complete document, including trailing
/// whitespace and comments, was accepted. If this function returns an error,
/// the sink may contain partial output and must be treated as incomplete.
pub fn parse<'input, 'schema>(
    input: &'input [u8],
    schema: &Schema<'schema>,
    sink: &mut dyn TargetSink<'input>,
    config: ParserConfig,
) -> Result<(), ParseError> {
    config.validate(Position::default())?;
    let mut tokenizer = Tokenizer::new(input, config.clone())?;
    let mut stack = ElementStack::new(config.max_depth);
    let mut text = TextAccumulator::new();
    let mut total_occurrences = 0usize;

    loop {
        match tokenizer.next()? {
            Event::Eof => {
                stack.finish(document_end_position(input))?;
                return Ok(());
            }
            Event::StartTag {
                name,
                attributes,
                empty,
                position,
            } => {
                let (node_id, node) = if stack.len() == 0 {
                    let root_id = schema.root_id();
                    let root = schema_node(schema, root_id, position)?;
                    if root.name != name {
                        return Err(ParseError::new(
                            ErrorKind::Schema,
                            ErrorCode::Mismatch,
                            position,
                        ));
                    }
                    (root_id, root)
                } else {
                    let parent_frame = stack.len() - 1;
                    resolve_child(
                        schema,
                        &mut stack,
                        parent_frame,
                        name,
                        &config,
                        &mut total_occurrences,
                        position,
                    )?
                };

                validate_attributes(attributes, node, &config, position)?;

                if empty {
                    validate_empty_element(node, position)?;
                    stack.self_close(name, position)?;
                    sink.begin_element(node_id.0)
                        .map_err(|error| map_sink_error(error, position))?;
                    emit_attributes(attributes, node, node_id, &config, sink)?;
                    if node.content == ContentType::String {
                        sink.element_string_content(node_id.0, TextValue::Borrowed(""))
                            .map_err(|error| map_sink_error(error, position))?;
                    }
                    sink.end_element(node_id.0)
                        .map_err(|error| map_sink_error(error, position))?;
                } else {
                    stack.push(name, position, node_id.0)?;
                    text.reset();
                    sink.begin_element(node_id.0)
                        .map_err(|error| map_sink_error(error, position))?;
                    emit_attributes(attributes, node, node_id, &config, sink)?;
                }
            }
            Event::EndTag { name, position } => {
                let Some(open_name) = stack.top_name() else {
                    return Err(ParseError::new(
                        ErrorKind::Syntax,
                        ErrorCode::Mismatch,
                        position,
                    ));
                };
                if open_name != name {
                    return Err(ParseError::new(
                        ErrorKind::Syntax,
                        ErrorCode::Mismatch,
                        position,
                    ));
                }

                let frame = stack.len() - 1;
                let node_id = SchemaNodeId(stack.schema_node_ids[frame]);
                let node = schema_node(schema, node_id, position)?;

                if node.content == ContentType::Elements {
                    validate_children_complete(node, &stack, frame, position)?;
                } else if node.content == ContentType::Integer {
                    if !text.has_text {
                        return Err(ParseError::new(
                            ErrorKind::Schema,
                            ErrorCode::Required,
                            position,
                        ));
                    }
                    let parsed = convert::parse_integer(text.as_str())
                        .map_err(|error| map_convert_error(error, text.position))?;
                    sink.element_integer_content(node_id.0, parsed)
                        .map_err(|error| map_sink_error(error, text.position))?;
                } else if node.content == ContentType::String {
                    sink.element_string_content(node_id.0, text.value())
                        .map_err(|error| map_sink_error(error, text.position))?;
                }

                stack.pop(name, position)?;
                sink.end_element(node_id.0)
                    .map_err(|error| map_sink_error(error, position))?;
                text.reset();
            }
            Event::Text { value, position } => {
                if stack.len() == 0 {
                    if !is_xml_whitespace(value.as_str()) {
                        return Err(ParseError::new(
                            ErrorKind::Syntax,
                            ErrorCode::MixedContent,
                            position,
                        ));
                    }
                    continue;
                }

                let frame = stack.len() - 1;
                let node_id = SchemaNodeId(stack.schema_node_ids[frame]);
                let node = schema_node(schema, node_id, position)?;
                if node.content == ContentType::Empty || node.content == ContentType::Elements {
                    if !is_xml_whitespace(value.as_str()) {
                        return Err(ParseError::new(
                            ErrorKind::Schema,
                            ErrorCode::MixedContent,
                            position,
                        ));
                    }
                    continue;
                }

                text.append(&value, config.max_text_bytes, position)?;
            }
        }
    }
}

fn resolve_child<'schema, 'input>(
    schema: &Schema<'schema>,
    stack: &mut ElementStack<'input>,
    frame: usize,
    name: &str,
    config: &ParserConfig,
    total_occurrences: &mut usize,
    position: Position,
) -> Result<(SchemaNodeId, &'schema SchemaNode<'schema>), ParseError> {
    let parent_id = SchemaNodeId(stack.schema_node_ids[frame]);
    let parent = schema_node(schema, parent_id, position)?;
    if parent.content != ContentType::Elements {
        return Err(ParseError::new(
            ErrorKind::Schema,
            ErrorCode::MixedContent,
            position,
        ));
    }

    let first_child_index = stack.child_indices[frame];
    for (child_index, descriptor) in parent.children.iter().enumerate().skip(first_child_index) {
        let child = schema_node(schema, descriptor.node_id, position)?;
        let occurrences = if child_index == first_child_index {
            stack.child_occurrences[frame]
        } else {
            0
        };
        if child.name == name {
            if occurrences >= descriptor.max_occurs as usize {
                return Err(ParseError::new(
                    ErrorKind::Schema,
                    ErrorCode::Occurrence,
                    position,
                ));
            }
            let next_occurrences = total_occurrences.checked_add(1).ok_or_else(|| {
                ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position)
            })?;
            if next_occurrences > config.max_occurrences {
                return Err(ParseError::new(
                    ErrorKind::Resource,
                    ErrorCode::Resource,
                    position,
                ));
            }
            let next_count = stack.child_counts[frame].checked_add(1).ok_or_else(|| {
                ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position)
            })?;
            if next_count > config.max_children_per_element {
                return Err(ParseError::new(
                    ErrorKind::Resource,
                    ErrorCode::Resource,
                    position,
                ));
            }
            stack.child_counts[frame] = next_count;
            stack.child_occurrences[frame] = occurrences + 1;
            stack.child_indices[frame] = child_index;
            *total_occurrences = next_occurrences;
            return Ok((descriptor.node_id, child));
        }

        if stack.child_occurrences[frame] < descriptor.min_occurs as usize {
            return Err(ParseError::new(
                ErrorKind::Schema,
                ErrorCode::Required,
                position,
            ));
        }
        stack.child_indices[frame] = child_index.saturating_add(1);
        stack.child_occurrences[frame] = 0;
    }

    Err(ParseError::new(
        ErrorKind::Schema,
        ErrorCode::Order,
        position,
    ))
}

fn schema_node<'schema>(
    schema: &Schema<'schema>,
    node_id: SchemaNodeId,
    position: Position,
) -> Result<&'schema SchemaNode<'schema>, ParseError> {
    schema
        .get_node(node_id)
        .ok_or_else(|| ParseError::new(ErrorKind::Schema, ErrorCode::Schema, position))
}

fn validate_attributes(
    attributes: &[Attribute<'_>],
    node: &SchemaNode<'_>,
    config: &ParserConfig,
    element_position: Position,
) -> Result<(), ParseError> {
    let mut seen = [false; MAX_SCHEMA_ATTRS];
    for attribute in attributes {
        let Some(index) = node
            .attributes
            .iter()
            .position(|descriptor| descriptor.name == attribute.name)
        else {
            return Err(ParseError::new(
                ErrorKind::Schema,
                ErrorCode::Attribute,
                attribute.position,
            ));
        };
        if seen[index] {
            return Err(ParseError::new(
                ErrorKind::Attribute,
                ErrorCode::Attribute,
                attribute.position,
            ));
        }
        seen[index] = true;

        let value = attribute.decode_value::<MAX_TEXT_BYTES>(config.max_text_bytes)?;
        if node.attributes[index].attr_type == AttributeType::Integer {
            convert::parse_integer(value.as_str())
                .map_err(|error| map_convert_error(error, attribute.position))?;
        }
    }
    for (index, descriptor) in node.attributes.iter().enumerate() {
        if descriptor.required && !seen[index] {
            return Err(ParseError::new(
                ErrorKind::Schema,
                ErrorCode::Required,
                element_position,
            ));
        }
    }
    Ok(())
}

fn emit_attributes<'input>(
    attributes: &[Attribute<'input>],
    node: &SchemaNode<'_>,
    node_id: SchemaNodeId,
    config: &ParserConfig,
    sink: &mut dyn TargetSink<'input>,
) -> Result<(), ParseError> {
    for attribute in attributes {
        let Some(descriptor) = node
            .attributes
            .iter()
            .find(|descriptor| descriptor.name == attribute.name)
        else {
            return Err(ParseError::new(
                ErrorKind::Schema,
                ErrorCode::Attribute,
                attribute.position,
            ));
        };
        let value = attribute.decode_value::<MAX_TEXT_BYTES>(config.max_text_bytes)?;
        match descriptor.attr_type {
            AttributeType::String => {
                let value = match value.borrowed() {
                    Some(value) => TextValue::Borrowed(value),
                    None => TextValue::Decoded(value.as_str()),
                };
                sink.attribute_string(node_id.0, attribute.name, value)
                    .map_err(|error| map_sink_error(error, attribute.position))?;
            }
            AttributeType::Integer => {
                let integer = convert::parse_integer(value.as_str())
                    .map_err(|error| map_convert_error(error, attribute.position))?;
                sink.attribute_integer(node_id.0, attribute.name, integer)
                    .map_err(|error| map_sink_error(error, attribute.position))?;
            }
        }
    }
    Ok(())
}

fn validate_empty_element(node: &SchemaNode<'_>, position: Position) -> Result<(), ParseError> {
    match node.content {
        ContentType::Empty | ContentType::String => Ok(()),
        ContentType::Integer => Err(ParseError::new(
            ErrorKind::Schema,
            ErrorCode::Required,
            position,
        )),
        ContentType::Elements => {
            if node.children.iter().any(|child| child.min_occurs > 0) {
                Err(ParseError::new(
                    ErrorKind::Schema,
                    ErrorCode::Required,
                    position,
                ))
            } else {
                Ok(())
            }
        }
    }
}

fn validate_children_complete(
    node: &SchemaNode<'_>,
    stack: &ElementStack<'_>,
    frame: usize,
    position: Position,
) -> Result<(), ParseError> {
    let first_missing = stack.child_indices[frame];
    for (index, child) in node.children.iter().enumerate().skip(first_missing) {
        let occurrences = if index == first_missing {
            stack.child_occurrences[frame]
        } else {
            0
        };
        if occurrences < child.min_occurs as usize {
            return Err(ParseError::new(
                ErrorKind::Schema,
                ErrorCode::Required,
                position,
            ));
        }
    }
    Ok(())
}

struct TextAccumulator<'input> {
    borrowed: Option<&'input str>,
    bytes: [u8; MAX_TEXT_BYTES],
    len: usize,
    has_text: bool,
    position: Position,
}

impl<'input> TextAccumulator<'input> {
    fn new() -> Self {
        Self {
            borrowed: None,
            bytes: [0; MAX_TEXT_BYTES],
            len: 0,
            has_text: false,
            position: Position::default(),
        }
    }

    fn reset(&mut self) {
        self.borrowed = None;
        self.len = 0;
        self.has_text = false;
        self.position = Position::default();
    }

    fn append(
        &mut self,
        value: &Text<'input, MAX_TEXT_BYTES>,
        limit: usize,
        position: Position,
    ) -> Result<(), ParseError> {
        if !self.has_text {
            self.has_text = true;
            self.position = position;
            if let Some(value) = value.borrowed() {
                self.borrowed = Some(value);
                return Ok(());
            }
            return self.append_str(value.as_str(), limit, position);
        }

        if let Some(previous) = self.borrowed.take() {
            self.append_str(previous, limit, position)?;
        }
        self.append_str(value.as_str(), limit, position)
    }

    fn append_str(
        &mut self,
        value: &str,
        limit: usize,
        position: Position,
    ) -> Result<(), ParseError> {
        let limit = limit.min(MAX_TEXT_BYTES);
        if self.len > limit || value.len() > limit - self.len {
            return Err(resource_error(position));
        }
        let end = self.len + value.len();
        self.bytes[self.len..end].copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }

    fn as_str(&self) -> &str {
        match self.borrowed {
            Some(value) => value,
            None => core::str::from_utf8(&self.bytes[..self.len]).unwrap_or(""),
        }
    }

    fn value(&self) -> TextValue<'input, '_> {
        if !self.has_text {
            return TextValue::Borrowed("");
        }
        match self.borrowed {
            Some(value) => TextValue::Borrowed(value),
            None => TextValue::Decoded(self.as_str()),
        }
    }
}

fn is_xml_whitespace(value: &str) -> bool {
    value
        .chars()
        .all(|character| matches!(character, ' ' | '\t' | '\n' | '\r'))
}

fn document_end_position(input: &[u8]) -> Position {
    let bom_len = if input.starts_with(b"\xEF\xBB\xBF") {
        3
    } else {
        0
    };
    let Ok(mut cursor) = Cursor::with_base_offset(&input[bom_len..], bom_len) else {
        return Position::new(input.len(), 1, 1);
    };
    while matches!(cursor.consume(), Ok(Some(_))) {}
    cursor.position()
}

fn map_convert_error(error: ConvertError, position: Position) -> ParseError {
    let code = match error {
        ConvertError::Overflow | ConvertError::Underflow => ErrorCode::IntegerOverflow,
        ConvertError::Empty | ConvertError::InvalidDigit | ConvertError::SignOnly => {
            ErrorCode::Syntax
        }
    };
    ParseError::new(ErrorKind::Conversion, code, position)
}

fn map_sink_error(_error: SinkError, position: Position) -> ParseError {
    ParseError::new(ErrorKind::Internal, ErrorCode::Sink, position)
}

fn resource_error(position: Position) -> ParseError {
    ParseError::new(ErrorKind::Resource, ErrorCode::Resource, position)
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use crate::schema::{AttributeDescriptor, ChildDescriptor, SchemaNode, SCHEMA_VERSION};
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    static BASIC_CHILDREN: [ChildDescriptor; 1] = [ChildDescriptor {
        node_id: SchemaNodeId(1),
        min_occurs: 1,
        max_occurs: 1,
    }];
    static OPTIONAL_LABEL: [AttributeDescriptor<'static>; 1] = [AttributeDescriptor {
        name: "label",
        attr_type: AttributeType::String,
        required: false,
    }];
    static BASIC_NODES: [SchemaNode<'static>; 2] = [
        SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &BASIC_CHILDREN,
            attributes: &OPTIONAL_LABEL,
        },
        SchemaNode {
            name: "count",
            content: ContentType::Integer,
            children: &[],
            attributes: &[],
        },
    ];
    static EMPTY_NODES: [SchemaNode<'static>; 1] = [SchemaNode {
        name: "empty",
        content: ContentType::Empty,
        children: &[],
        attributes: &[],
    }];
    static STRING_NODES: [SchemaNode<'static>; 1] = [SchemaNode {
        name: "text",
        content: ContentType::String,
        children: &[],
        attributes: &[],
    }];
    static REQUIRED_LABEL: [AttributeDescriptor<'static>; 1] = [AttributeDescriptor {
        name: "label",
        attr_type: AttributeType::String,
        required: true,
    }];
    static REQUIRED_LABEL_NODES: [SchemaNode<'static>; 1] = [SchemaNode {
        name: "root",
        content: ContentType::Empty,
        children: &[],
        attributes: &REQUIRED_LABEL,
    }];
    static INTEGER_ATTRIBUTE: [AttributeDescriptor<'static>; 1] = [AttributeDescriptor {
        name: "amount",
        attr_type: AttributeType::Integer,
        required: true,
    }];
    static INTEGER_ATTRIBUTE_NODES: [SchemaNode<'static>; 1] = [SchemaNode {
        name: "root",
        content: ContentType::Empty,
        children: &[],
        attributes: &INTEGER_ATTRIBUTE,
    }];
    static SEQUENCE_CHILDREN: [ChildDescriptor; 2] = [
        ChildDescriptor {
            node_id: SchemaNodeId(1),
            min_occurs: 0,
            max_occurs: 1,
        },
        ChildDescriptor {
            node_id: SchemaNodeId(2),
            min_occurs: 1,
            max_occurs: 2,
        },
    ];
    static REQUIRED_SEQUENCE_CHILDREN: [ChildDescriptor; 2] = [
        ChildDescriptor {
            node_id: SchemaNodeId(1),
            min_occurs: 1,
            max_occurs: 1,
        },
        ChildDescriptor {
            node_id: SchemaNodeId(2),
            min_occurs: 0,
            max_occurs: 1,
        },
    ];
    static REQUIRED_SEQUENCE_NODES: [SchemaNode<'static>; 3] = [
        SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &REQUIRED_SEQUENCE_CHILDREN,
            attributes: &[],
        },
        SchemaNode {
            name: "a",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
        SchemaNode {
            name: "b",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
    ];
    static OPTIONAL_CHILDREN: [ChildDescriptor; 1] = [ChildDescriptor {
        node_id: SchemaNodeId(1),
        min_occurs: 0,
        max_occurs: 1,
    }];
    static OPTIONAL_ELEMENT_NODES: [SchemaNode<'static>; 2] = [
        SchemaNode {
            name: "optional",
            content: ContentType::Elements,
            children: &OPTIONAL_CHILDREN,
            attributes: &[],
        },
        SchemaNode {
            name: "child",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
    ];
    static SIBLING_STRING_CHILDREN: [ChildDescriptor; 2] = [
        ChildDescriptor {
            node_id: SchemaNodeId(1),
            min_occurs: 1,
            max_occurs: 1,
        },
        ChildDescriptor {
            node_id: SchemaNodeId(2),
            min_occurs: 1,
            max_occurs: 1,
        },
    ];
    static SIBLING_STRING_NODES: [SchemaNode<'static>; 3] = [
        SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &SIBLING_STRING_CHILDREN,
            attributes: &[],
        },
        SchemaNode {
            name: "first",
            content: ContentType::String,
            children: &[],
            attributes: &[],
        },
        SchemaNode {
            name: "second",
            content: ContentType::String,
            children: &[],
            attributes: &[],
        },
    ];
    static SEQUENCE_NODES: [SchemaNode<'static>; 3] = [
        SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &SEQUENCE_CHILDREN,
            attributes: &[],
        },
        SchemaNode {
            name: "a",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
        SchemaNode {
            name: "b",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
    ];
    static NESTED_ROOT_CHILDREN: [ChildDescriptor; 2] = [
        ChildDescriptor {
            node_id: SchemaNodeId(1),
            min_occurs: 1,
            max_occurs: 1,
        },
        ChildDescriptor {
            node_id: SchemaNodeId(2),
            min_occurs: 1,
            max_occurs: 1,
        },
    ];
    static NESTED_A_CHILDREN: [ChildDescriptor; 1] = [ChildDescriptor {
        node_id: SchemaNodeId(3),
        min_occurs: 2,
        max_occurs: 2,
    }];
    static NESTED_B_CHILDREN: [ChildDescriptor; 1] = [ChildDescriptor {
        node_id: SchemaNodeId(4),
        min_occurs: 1,
        max_occurs: 3,
    }];
    static NESTED_NODES: [SchemaNode<'static>; 5] = [
        SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &NESTED_ROOT_CHILDREN,
            attributes: &[],
        },
        SchemaNode {
            name: "group_a",
            content: ContentType::Elements,
            children: &NESTED_A_CHILDREN,
            attributes: &[],
        },
        SchemaNode {
            name: "group_b",
            content: ContentType::Elements,
            children: &NESTED_B_CHILDREN,
            attributes: &[],
        },
        SchemaNode {
            name: "leaf_a",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
        SchemaNode {
            name: "leaf_b",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
    ];

    #[derive(Debug, PartialEq, Eq)]
    enum Call {
        Begin(u16),
        End(u16),
        AttributeString(u16, String, String, bool),
        AttributeInteger(u16, String, i64),
        ElementString(u16, String, bool),
        ElementInteger(u16, i64),
    }

    #[derive(Default)]
    struct RecordingSink {
        calls: Vec<Call>,
        reject_at: Option<usize>,
    }

    impl RecordingSink {
        fn record(&mut self, call: Call) -> Result<(), SinkError> {
            if self.reject_at == Some(self.calls.len()) {
                return Err(SinkError::RejectedByApplication);
            }
            self.calls.push(call);
            Ok(())
        }
    }

    impl<'input> TargetSink<'input> for RecordingSink {
        fn begin_element(&mut self, node_id: u16) -> Result<(), SinkError> {
            self.record(Call::Begin(node_id))
        }

        fn end_element(&mut self, node_id: u16) -> Result<(), SinkError> {
            self.record(Call::End(node_id))
        }

        fn attribute_string(
            &mut self,
            node_id: u16,
            name: &'input str,
            value: TextValue<'input, '_>,
        ) -> Result<(), SinkError> {
            self.record(Call::AttributeString(
                node_id,
                String::from(name),
                String::from(value.as_str()),
                value.borrowed().is_some(),
            ))
        }

        fn attribute_integer(
            &mut self,
            node_id: u16,
            name: &'input str,
            value: i64,
        ) -> Result<(), SinkError> {
            self.record(Call::AttributeInteger(node_id, String::from(name), value))
        }

        fn element_string_content(
            &mut self,
            node_id: u16,
            value: TextValue<'input, '_>,
        ) -> Result<(), SinkError> {
            self.record(Call::ElementString(
                node_id,
                String::from(value.as_str()),
                value.borrowed().is_some(),
            ))
        }

        fn element_integer_content(&mut self, node_id: u16, value: i64) -> Result<(), SinkError> {
            self.record(Call::ElementInteger(node_id, value))
        }
    }

    fn schema(nodes: &'static [SchemaNode<'static>]) -> Schema<'static> {
        Schema::new(nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap()
    }

    fn parse_with(
        input: &[u8],
        nodes: &'static [SchemaNode<'static>],
        config: ParserConfig,
        sink: &mut RecordingSink,
    ) -> Result<(), ParseError> {
        parse(input, &schema(nodes), sink, config)
    }

    fn expect_error(
        input: &[u8],
        nodes: &'static [SchemaNode<'static>],
        config: ParserConfig,
    ) -> ParseError {
        parse_with(input, nodes, config, &mut RecordingSink::default()).unwrap_err()
    }

    fn with_start_tag<'input>(
        input: &'input [u8],
        config: ParserConfig,
        action: impl FnOnce(&[Attribute<'input>], Position),
    ) {
        let mut tokenizer = Tokenizer::new(input, config).unwrap();
        let event = tokenizer.next().unwrap();
        assert!(matches!(event, Event::StartTag { .. }));
        if let Event::StartTag {
            attributes,
            position,
            ..
        } = event
        {
            action(attributes, position);
        }
    }

    #[test]
    fn covers_schema_lookup_and_child_state_error_paths() {
        let position = Position::new(11, 2, 4);
        let basic_schema = schema(&BASIC_NODES);
        let invalid_node =
            schema_node(&basic_schema, SchemaNodeId(u16::MAX), position).unwrap_err();
        assert_eq!(
            (invalid_node.kind, invalid_node.code, invalid_node.position),
            (ErrorKind::Schema, ErrorCode::Schema, position)
        );

        let mut invalid_parent = ElementStack::new(2);
        invalid_parent.push("missing", position, u16::MAX).unwrap();
        let mut total = 0;
        let invalid_parent_error = resolve_child(
            &basic_schema,
            &mut invalid_parent,
            0,
            "count",
            &ParserConfig::default(),
            &mut total,
            position,
        )
        .unwrap_err();
        assert_eq!(
            (invalid_parent_error.kind, invalid_parent_error.code),
            (ErrorKind::Schema, ErrorCode::Schema)
        );

        let string_schema = schema(&STRING_NODES);
        let mut scalar_parent = ElementStack::new(2);
        scalar_parent.push("text", position, 0).unwrap();
        let scalar_child_error = resolve_child(
            &string_schema,
            &mut scalar_parent,
            0,
            "child",
            &ParserConfig::default(),
            &mut total,
            position,
        )
        .unwrap_err();
        assert_eq!(
            (scalar_child_error.kind, scalar_child_error.code),
            (ErrorKind::Schema, ErrorCode::MixedContent)
        );

        let mut root = ElementStack::new(2);
        root.push("root", position, 0).unwrap();
        let unbounded = ParserConfig::builder()
            .max_children_per_element(usize::MAX)
            .max_occurrences(usize::MAX)
            .build();
        let mut max_total = usize::MAX;
        let occurrence_overflow = resolve_child(
            &basic_schema,
            &mut root,
            0,
            "count",
            &unbounded,
            &mut max_total,
            position,
        )
        .unwrap_err();
        assert_eq!(
            (occurrence_overflow.kind, occurrence_overflow.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );
        assert_eq!(max_total, usize::MAX);

        root.child_counts[0] = usize::MAX;
        let mut total = 0;
        let child_overflow = resolve_child(
            &basic_schema,
            &mut root,
            0,
            "count",
            &unbounded,
            &mut total,
            position,
        )
        .unwrap_err();
        assert_eq!(
            (child_overflow.kind, child_overflow.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );
        assert_eq!(root.child_counts[0], usize::MAX);
        assert_eq!(total, 0);
    }

    #[test]
    fn directly_checks_attribute_validation_and_emission_failures() {
        let config = ParserConfig::default();
        with_start_tag(
            b"<root label='x'/>",
            config.clone(),
            |attributes, position| {
                let duplicate = [attributes[0], attributes[0]];
                let duplicate_error =
                    validate_attributes(&duplicate, &REQUIRED_LABEL_NODES[0], &config, position)
                        .unwrap_err();
                assert_eq!(
                    (duplicate_error.kind, duplicate_error.code),
                    (ErrorKind::Attribute, ErrorCode::Attribute)
                );
            },
        );

        with_start_tag(
            b"<root other='x'/>",
            config.clone(),
            |attributes, position| {
                let unknown_error =
                    validate_attributes(attributes, &BASIC_NODES[0], &config, position)
                        .unwrap_err();
                assert_eq!(
                    (unknown_error.kind, unknown_error.code),
                    (ErrorKind::Schema, ErrorCode::Attribute)
                );
                let emit_unknown_error = emit_attributes(
                    attributes,
                    &BASIC_NODES[0],
                    SchemaNodeId(0),
                    &config,
                    &mut RecordingSink::default(),
                )
                .unwrap_err();
                assert_eq!(
                    (emit_unknown_error.kind, emit_unknown_error.code),
                    (ErrorKind::Schema, ErrorCode::Attribute)
                );
            },
        );

        with_start_tag(
            b"<root amount='x'/>",
            config.clone(),
            |attributes, position| {
                let invalid_integer =
                    validate_attributes(attributes, &INTEGER_ATTRIBUTE_NODES[0], &config, position)
                        .unwrap_err();
                assert_eq!(
                    (invalid_integer.kind, invalid_integer.code),
                    (ErrorKind::Conversion, ErrorCode::Syntax)
                );
                let emitted_invalid_integer = emit_attributes(
                    attributes,
                    &INTEGER_ATTRIBUTE_NODES[0],
                    SchemaNodeId(0),
                    &config,
                    &mut RecordingSink::default(),
                )
                .unwrap_err();
                assert_eq!(
                    (emitted_invalid_integer.kind, emitted_invalid_integer.code),
                    (ErrorKind::Conversion, ErrorCode::Syntax)
                );
            },
        );

        with_start_tag(
            b"<root amount='' />",
            config.clone(),
            |attributes, position| {
                let empty_integer =
                    validate_attributes(attributes, &INTEGER_ATTRIBUTE_NODES[0], &config, position)
                        .unwrap_err();
                assert_eq!(
                    (empty_integer.kind, empty_integer.code),
                    (ErrorKind::Conversion, ErrorCode::Syntax)
                );
            },
        );

        with_start_tag(
            b"<root amount='+'/>",
            config.clone(),
            |attributes, position| {
                let sign_only =
                    validate_attributes(attributes, &INTEGER_ATTRIBUTE_NODES[0], &config, position)
                        .unwrap_err();
                assert_eq!(
                    (sign_only.kind, sign_only.code),
                    (ErrorKind::Conversion, ErrorCode::Syntax)
                );
            },
        );

        with_start_tag(
            b"<root label='x'/>",
            config.clone(),
            |attributes, position| {
                let too_long = ParserConfig::builder().max_text_bytes(0).build();
                let decode_error =
                    validate_attributes(attributes, &REQUIRED_LABEL_NODES[0], &too_long, position)
                        .unwrap_err();
                assert_eq!(
                    (decode_error.kind, decode_error.code),
                    (ErrorKind::Resource, ErrorCode::Resource)
                );
            },
        );
    }

    #[test]
    fn covers_text_accumulator_and_document_position_fallbacks() {
        let position = Position::new(4, 1, 5);
        let borrowed = crate::text::decode_text::<MAX_TEXT_BYTES>(
            b"ab",
            crate::text::NormalizeMode::ElementContent,
            2,
        )
        .unwrap();
        let next = crate::text::decode_text::<MAX_TEXT_BYTES>(
            b"c",
            crate::text::NormalizeMode::ElementContent,
            1,
        )
        .unwrap();
        let mut joined = TextAccumulator::new();
        joined.append(&borrowed, 3, position).unwrap();
        joined.append(&next, 3, Position::new(8, 1, 9)).unwrap();
        assert_eq!(joined.as_str(), "abc");
        assert_eq!(joined.value().as_str(), "abc");
        assert_eq!(joined.position, position);

        let decoded = crate::text::decode_text::<MAX_TEXT_BYTES>(
            b"A&amp;B",
            crate::text::NormalizeMode::ElementContent,
            MAX_TEXT_BYTES,
        )
        .unwrap();
        let mut accumulated_decoded = TextAccumulator::new();
        accumulated_decoded
            .append(&decoded, MAX_TEXT_BYTES, position)
            .unwrap();
        assert_eq!(accumulated_decoded.as_str(), "A&B");

        let mut exact_limit = TextAccumulator::new();
        exact_limit.append_str("xy", 2, position).unwrap();
        exact_limit.append_str("", 2, position).unwrap();
        assert_eq!(exact_limit.as_str(), "xy");

        let mut inconsistent_limit = TextAccumulator::new();
        inconsistent_limit
            .append(&decoded, MAX_TEXT_BYTES, position)
            .unwrap();
        let empty = crate::text::decode_text::<MAX_TEXT_BYTES>(
            b"",
            crate::text::NormalizeMode::ElementContent,
            MAX_TEXT_BYTES,
        )
        .unwrap();
        let lowered_limit = inconsistent_limit.append(&empty, 2, position);
        assert_eq!(lowered_limit.unwrap_err().kind, ErrorKind::Resource);

        let mut invalid_utf8 = TextAccumulator::new();
        invalid_utf8.bytes[0] = 0xff;
        invalid_utf8.len = 1;
        invalid_utf8.has_text = true;
        assert_eq!(invalid_utf8.as_str(), "");
        assert_eq!(invalid_utf8.value().as_str(), "");

        assert!(is_xml_whitespace(""));
        assert_eq!(document_end_position(&[0xff]), Position::new(1, 1, 1));
    }

    #[test]
    fn typed_vertical_slice_accepts_borrowed_and_decoded_attributes() {
        let mut sink = RecordingSink::default();
        assert_eq!(
            parse_with(
                b"<root><count>42</count></root>",
                &BASIC_NODES,
                ParserConfig::default(),
                &mut sink,
            ),
            Ok(())
        );
        assert!(sink.calls.contains(&Call::ElementInteger(1, 42)));

        let mut plain = RecordingSink::default();
        parse_with(
            b"<root label=\"plain\"><count>42</count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
            &mut plain,
        )
        .unwrap();
        assert!(plain.calls.contains(&Call::AttributeString(
            0,
            String::from("label"),
            String::from("plain"),
            true,
        )));

        let mut decoded = RecordingSink::default();
        parse_with(
            b"<root label=\"A&amp;B\"><count>42</count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
            &mut decoded,
        )
        .unwrap();
        assert!(decoded.calls.contains(&Call::AttributeString(
            0,
            String::from("label"),
            String::from("A&B"),
            false,
        )));
    }

    #[test]
    fn rejects_wrong_root_unknown_child_and_multiple_roots() {
        let wrong_root = expect_error(
            b"<other><count>1</count></other>",
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (wrong_root.kind, wrong_root.code),
            (ErrorKind::Schema, ErrorCode::Mismatch)
        );

        let unknown_child = expect_error(
            b"<root><b/><unknown/></root>",
            &SEQUENCE_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (unknown_child.kind, unknown_child.code),
            (ErrorKind::Schema, ErrorCode::Order)
        );

        let multiple = expect_error(
            b"<root><count>1</count></root><root><count>2</count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (multiple.kind, multiple.code),
            (ErrorKind::Syntax, ErrorCode::MultipleRoot)
        );
    }

    #[test]
    fn rejects_unmatched_mismatched_and_truncated_tags_with_positions() {
        let unmatched = expect_error(b"</empty>", &EMPTY_NODES, ParserConfig::default());
        assert_eq!(
            (unmatched.kind, unmatched.code),
            (ErrorKind::Syntax, ErrorCode::Mismatch)
        );

        let mismatch = expect_error(b"<empty></other>", &EMPTY_NODES, ParserConfig::default());
        assert_eq!(
            (mismatch.kind, mismatch.code),
            (ErrorKind::Syntax, ErrorCode::Mismatch)
        );
        assert_eq!(mismatch.position.byte_offset, 7);

        let input = b"<root>\n<count>1</count>";
        let truncated = expect_error(input, &BASIC_NODES, ParserConfig::default());
        assert_eq!(
            (truncated.kind, truncated.code),
            (ErrorKind::Syntax, ErrorCode::Eof)
        );
        assert_eq!(truncated.position.byte_offset, input.len());
        assert_eq!(truncated.position.line, 2);
        assert_eq!(truncated.position.column, input.len() - 6);

        let with_bom = b"\xEF\xBB\xBF<root><count>1</count>";
        let truncated_bom = expect_error(with_bom, &BASIC_NODES, ParserConfig::default());
        assert_eq!(truncated_bom.position.byte_offset, with_bom.len());
        assert_eq!(truncated_bom.position.line, 1);
        assert_eq!(truncated_bom.position.column, with_bom.len() - 2);

        let empty = expect_error(b"", &EMPTY_NODES, ParserConfig::default());
        assert_eq!(
            (empty.kind, empty.code),
            (ErrorKind::Syntax, ErrorCode::Eof)
        );
        assert_eq!(empty.position, Position::default());
    }

    #[test]
    fn validates_order_required_counts_and_occurrence_limits() {
        let out_of_order = expect_error(
            b"<root><b/><a/></root>",
            &SEQUENCE_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (out_of_order.kind, out_of_order.code),
            (ErrorKind::Schema, ErrorCode::Order)
        );

        let missing = expect_error(
            b"<root><a/></root>",
            &SEQUENCE_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (missing.kind, missing.code),
            (ErrorKind::Schema, ErrorCode::Required)
        );

        let missing_before_later_child = expect_error(
            b"<root><b/></root>",
            &REQUIRED_SEQUENCE_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (
                missing_before_later_child.kind,
                missing_before_later_child.code
            ),
            (ErrorKind::Schema, ErrorCode::Required)
        );

        let max_occurs = expect_error(
            b"<root><b/><b/><b/></root>",
            &SEQUENCE_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (max_occurs.kind, max_occurs.code),
            (ErrorKind::Schema, ErrorCode::Occurrence)
        );

        let mut sink = RecordingSink::default();
        parse_with(
            b"<root><a/><b/><b/></root>",
            &SEQUENCE_NODES,
            ParserConfig::default(),
            &mut sink,
        )
        .unwrap();
    }

    #[test]
    fn enforces_per_parent_child_and_occurrence_resource_limits() {
        let child_limit = expect_error(
            b"<root><a/><b/></root>",
            &SEQUENCE_NODES,
            ParserConfig::builder().max_children_per_element(1).build(),
        );
        assert_eq!(
            (child_limit.kind, child_limit.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );

        let mut exact_parent_limit = RecordingSink::default();
        parse_with(
            b"<root><a/><b/><b/></root>",
            &SEQUENCE_NODES,
            ParserConfig::builder().max_children_per_element(3).build(),
            &mut exact_parent_limit,
        )
        .unwrap();

        let input = b"<root><group_a><leaf_a/><leaf_a/></group_a><group_b><leaf_b/><leaf_b/><leaf_b/></group_b></root>";
        let mut exact_global_limits = RecordingSink::default();
        parse_with(
            input,
            &NESTED_NODES,
            ParserConfig::builder()
                .max_children_per_element(3)
                .max_occurrences(7)
                .build(),
            &mut exact_global_limits,
        )
        .unwrap();

        let child_limit = expect_error(
            input,
            &NESTED_NODES,
            ParserConfig::builder()
                .max_children_per_element(2)
                .max_occurrences(7)
                .build(),
        );
        assert_eq!(
            (child_limit.kind, child_limit.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );

        let occurrence_limit = expect_error(
            input,
            &NESTED_NODES,
            ParserConfig::builder()
                .max_children_per_element(3)
                .max_occurrences(6)
                .build(),
        );
        assert_eq!(
            (occurrence_limit.kind, occurrence_limit.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );
    }

    #[test]
    fn validates_required_unknown_duplicate_and_integer_attributes() {
        let required = expect_error(b"<root/>", &REQUIRED_LABEL_NODES, ParserConfig::default());
        assert_eq!(
            (required.kind, required.code),
            (ErrorKind::Schema, ErrorCode::Required)
        );

        let unknown = expect_error(
            b"<root other=\"x\"/>",
            &REQUIRED_LABEL_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (unknown.kind, unknown.code),
            (ErrorKind::Schema, ErrorCode::Attribute)
        );

        let duplicate = expect_error(
            b"<root label=\"a\" label=\"b\"/>",
            &REQUIRED_LABEL_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (duplicate.kind, duplicate.code),
            (ErrorKind::Syntax, ErrorCode::Attribute)
        );

        let valid_int = b"<root amount=\"-42\"/>";
        let mut sink = RecordingSink::default();
        parse_with(
            valid_int,
            &INTEGER_ATTRIBUTE_NODES,
            ParserConfig::default(),
            &mut sink,
        )
        .unwrap();
        assert!(sink
            .calls
            .contains(&Call::AttributeInteger(0, String::from("amount"), -42,)));

        let overflow = expect_error(
            b"<root amount=\"9223372036854775808\"/>",
            &INTEGER_ATTRIBUTE_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (overflow.kind, overflow.code),
            (ErrorKind::Conversion, ErrorCode::IntegerOverflow)
        );
    }

    #[test]
    fn parses_string_text_and_preserves_borrowing_contract() {
        let mut borrowed = RecordingSink::default();
        parse_with(
            b"<text>plain</text>",
            &STRING_NODES,
            ParserConfig::default(),
            &mut borrowed,
        )
        .unwrap();
        assert!(borrowed
            .calls
            .contains(&Call::ElementString(0, String::from("plain"), true,)));

        let mut decoded = RecordingSink::default();
        parse_with(
            b"<text>A&amp;B</text>",
            &STRING_NODES,
            ParserConfig::default(),
            &mut decoded,
        )
        .unwrap();
        assert!(decoded
            .calls
            .contains(&Call::ElementString(0, String::from("A&B"), false,)));

        let mut empty_self_closing = RecordingSink::default();
        parse_with(
            b"<text/>",
            &STRING_NODES,
            ParserConfig::default(),
            &mut empty_self_closing,
        )
        .unwrap();
        assert!(empty_self_closing
            .calls
            .contains(&Call::ElementString(0, String::new(), true,)));

        let mut empty_pair = RecordingSink::default();
        parse_with(
            b"<text></text>",
            &STRING_NODES,
            ParserConfig::default(),
            &mut empty_pair,
        )
        .unwrap();
        assert!(empty_pair
            .calls
            .contains(&Call::ElementString(0, String::new(), true,)));

        let mut joined = RecordingSink::default();
        parse_with(
            b"<text>A<!-- split -->B</text>",
            &STRING_NODES,
            ParserConfig::default(),
            &mut joined,
        )
        .unwrap();
        assert!(joined
            .calls
            .contains(&Call::ElementString(0, String::from("AB"), false,)));
    }

    #[test]
    fn rejects_invalid_integer_content_and_integer_self_closing_element() {
        let lexical = expect_error(
            b"<root><count>12x</count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (lexical.kind, lexical.code),
            (ErrorKind::Conversion, ErrorCode::Syntax)
        );

        let overflow = expect_error(
            b"<root><count>9223372036854775808</count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (overflow.kind, overflow.code),
            (ErrorKind::Conversion, ErrorCode::IntegerOverflow)
        );

        let underflow = expect_error(
            b"<root><count>-9223372036854775809</count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (underflow.kind, underflow.code),
            (ErrorKind::Conversion, ErrorCode::IntegerOverflow)
        );

        let empty = expect_error(
            b"<root><count/></root>",
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (empty.kind, empty.code),
            (ErrorKind::Schema, ErrorCode::Required)
        );

        let paired_empty = expect_error(
            b"<root><count></count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (paired_empty.kind, paired_empty.code),
            (ErrorKind::Schema, ErrorCode::Required)
        );
    }

    #[test]
    fn accepts_only_xml_whitespace_between_element_content() {
        let mut valid = RecordingSink::default();
        parse_with(
            b" \t\r\n<root>\n <count>1</count>\r\n</root> ",
            &BASIC_NODES,
            ParserConfig::default(),
            &mut valid,
        )
        .unwrap();

        let outside = expect_error(
            "\u{00a0}<root><count>1</count></root>".as_bytes(),
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (outside.kind, outside.code),
            (ErrorKind::Syntax, ErrorCode::MixedContent)
        );

        let mixed = expect_error(
            "<root>\u{00a0}<count>1</count></root>".as_bytes(),
            &BASIC_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (mixed.kind, mixed.code),
            (ErrorKind::Schema, ErrorCode::MixedContent)
        );
    }

    #[test]
    fn enforces_document_depth_attribute_text_and_token_limits() {
        let input = b"<root><count>1</count></root>";
        let mut sink = RecordingSink::default();
        parse_with(
            input,
            &BASIC_NODES,
            ParserConfig::builder()
                .max_document_bytes(input.len())
                .build(),
            &mut sink,
        )
        .unwrap();

        let mut exact_depth = RecordingSink::default();
        parse_with(
            input,
            &BASIC_NODES,
            ParserConfig::builder().max_depth(2).build(),
            &mut exact_depth,
        )
        .unwrap();

        for config in [
            ParserConfig::builder()
                .max_document_bytes(input.len() - 1)
                .build(),
            ParserConfig::builder().max_depth(1).build(),
            ParserConfig::builder().max_tokens(0).build(),
        ] {
            let error = expect_error(input, &BASIC_NODES, config);
            assert_eq!(
                (error.kind, error.code),
                (ErrorKind::Resource, ErrorCode::Resource)
            );
        }

        let invalid_fixed_storage = expect_error(
            input,
            &BASIC_NODES,
            ParserConfig::builder()
                .max_depth(crate::config::MAX_DEPTH + 1)
                .build(),
        );
        assert_eq!(
            (invalid_fixed_storage.kind, invalid_fixed_storage.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );

        let mut exact_attribute_limit = RecordingSink::default();
        parse_with(
            b"<root label=\"x\"><count>1</count></root>",
            &BASIC_NODES,
            ParserConfig::builder()
                .max_attributes_per_element(1)
                .build(),
            &mut exact_attribute_limit,
        )
        .unwrap();
        let attr_limit = expect_error(
            b"<root label=\"x\"><count>1</count></root>",
            &BASIC_NODES,
            ParserConfig::builder()
                .max_attributes_per_element(0)
                .build(),
        );
        assert_eq!(
            (attr_limit.kind, attr_limit.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );

        let mut exact_text = RecordingSink::default();
        parse_with(
            b"<text>abc</text>",
            &STRING_NODES,
            ParserConfig::builder().max_text_bytes(3).build(),
            &mut exact_text,
        )
        .unwrap();
        let mut exact_decoded_text = RecordingSink::default();
        parse_with(
            b"<text>a&amp;</text>",
            &STRING_NODES,
            ParserConfig::builder().max_text_bytes(2).build(),
            &mut exact_decoded_text,
        )
        .unwrap();
        assert!(exact_decoded_text.calls.contains(&Call::ElementString(
            0,
            String::from("a&"),
            false
        )));

        let over_text = expect_error(
            b"<text>abcd</text>",
            &STRING_NODES,
            ParserConfig::builder().max_text_bytes(3).build(),
        );
        assert_eq!(
            (over_text.kind, over_text.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );

        let mut exact_aggregate_text = RecordingSink::default();
        parse_with(
            b"<text>ab<!-- split -->c</text>",
            &STRING_NODES,
            ParserConfig::builder().max_text_bytes(3).build(),
            &mut exact_aggregate_text,
        )
        .unwrap();
        let aggregate_overflow = expect_error(
            b"<text>ab<!-- split -->cd</text>",
            &STRING_NODES,
            ParserConfig::builder().max_text_bytes(3).build(),
        );
        assert_eq!(
            (aggregate_overflow.kind, aggregate_overflow.code),
            (ErrorKind::Resource, ErrorCode::Resource)
        );

        let mut exact_token_limit = RecordingSink::default();
        parse_with(
            b"<empty/>",
            &EMPTY_NODES,
            ParserConfig::builder().max_tokens(1).build(),
            &mut exact_token_limit,
        )
        .unwrap();
    }

    #[test]
    fn text_accumulator_resets_between_sibling_string_nodes() {
        let mut sink = RecordingSink::default();
        parse_with(
            b"<root><first>one</first><second>two</second></root>",
            &SIBLING_STRING_NODES,
            ParserConfig::default(),
            &mut sink,
        )
        .unwrap();

        assert!(sink
            .calls
            .contains(&Call::ElementString(1, String::from("one"), true)));
        assert!(sink
            .calls
            .contains(&Call::ElementString(2, String::from("two"), true)));
    }

    #[test]
    fn empty_schema_nodes_and_empty_documents_are_handled() {
        let mut sink = RecordingSink::default();
        parse_with(
            b"<empty/>",
            &EMPTY_NODES,
            ParserConfig::default(),
            &mut sink,
        )
        .unwrap();
        assert_eq!(sink.calls, vec![Call::Begin(0), Call::End(0)]);

        let mut optional_empty = RecordingSink::default();
        parse_with(
            b"<optional/>",
            &OPTIONAL_ELEMENT_NODES,
            ParserConfig::default(),
            &mut optional_empty,
        )
        .unwrap();

        let required_empty = expect_error(b"<root/>", &BASIC_NODES, ParserConfig::default());
        assert_eq!(
            (required_empty.kind, required_empty.code),
            (ErrorKind::Schema, ErrorCode::Required)
        );

        let mut whitespace_empty = RecordingSink::default();
        parse_with(
            b"<empty> \r\n </empty>",
            &EMPTY_NODES,
            ParserConfig::default(),
            &mut whitespace_empty,
        )
        .unwrap();

        let invalid = expect_error(
            b"<empty>text</empty>",
            &EMPTY_NODES,
            ParserConfig::default(),
        );
        assert_eq!(
            (invalid.kind, invalid.code),
            (ErrorKind::Schema, ErrorCode::MixedContent)
        );

        let trailing = expect_error(b"<empty/>text", &EMPTY_NODES, ParserConfig::default());
        assert_eq!(
            (trailing.kind, trailing.code),
            (ErrorKind::Syntax, ErrorCode::MixedContent)
        );
    }

    #[test]
    fn sink_rejection_is_returned_as_a_parse_error() {
        let mut sink = RecordingSink {
            calls: Vec::new(),
            reject_at: Some(1),
        };
        let error = parse_with(
            b"<root label=\"x\"><count>1</count></root>",
            &BASIC_NODES,
            ParserConfig::default(),
            &mut sink,
        )
        .unwrap_err();
        assert_eq!(
            (error.kind, error.code),
            (ErrorKind::Internal, ErrorCode::Sink)
        );
        assert_eq!(sink.calls, vec![Call::Begin(0)]);

        for rejected_callback in 0..6 {
            let mut sink = RecordingSink {
                calls: Vec::new(),
                reject_at: Some(rejected_callback),
            };
            let error = parse_with(
                b"<root label=\"x\"><count>1</count></root>",
                &BASIC_NODES,
                ParserConfig::default(),
                &mut sink,
            )
            .unwrap_err();
            assert_eq!(
                (error.kind, error.code),
                (ErrorKind::Internal, ErrorCode::Sink)
            );
            assert_eq!(sink.calls.len(), rejected_callback);
        }

        for rejected_callback in 0..3 {
            let mut sink = RecordingSink {
                calls: Vec::new(),
                reject_at: Some(rejected_callback),
            };
            let error = parse_with(
                b"<text/>",
                &STRING_NODES,
                ParserConfig::default(),
                &mut sink,
            )
            .unwrap_err();
            assert_eq!(
                (error.kind, error.code),
                (ErrorKind::Internal, ErrorCode::Sink)
            );
            assert_eq!(sink.calls.len(), rejected_callback);
        }

        let mut string_content_sink = RecordingSink {
            calls: Vec::new(),
            reject_at: Some(1),
        };
        let string_content_error = parse_with(
            b"<text>value</text>",
            &STRING_NODES,
            ParserConfig::default(),
            &mut string_content_sink,
        )
        .unwrap_err();
        assert_eq!(
            (string_content_error.kind, string_content_error.code),
            (ErrorKind::Internal, ErrorCode::Sink)
        );
        assert_eq!(string_content_sink.calls, vec![Call::Begin(0)]);

        let mut integer_attribute_sink = RecordingSink {
            calls: Vec::new(),
            reject_at: Some(1),
        };
        let error = parse_with(
            b"<root amount=\"7\"/>",
            &INTEGER_ATTRIBUTE_NODES,
            ParserConfig::default(),
            &mut integer_attribute_sink,
        )
        .unwrap_err();
        assert_eq!(
            (error.kind, error.code),
            (ErrorKind::Internal, ErrorCode::Sink)
        );
    }

    #[test]
    fn callbacks_are_partial_when_later_document_validation_fails() {
        let mut sink = RecordingSink::default();
        let error = parse_with(
            b"<root><count>1</count></root>x",
            &BASIC_NODES,
            ParserConfig::default(),
            &mut sink,
        )
        .unwrap_err();
        assert_eq!(
            (error.kind, error.code),
            (ErrorKind::Syntax, ErrorCode::MixedContent)
        );
        assert!(sink.calls.contains(&Call::End(0)));
    }
}
