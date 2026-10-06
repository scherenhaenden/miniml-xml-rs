use miniml_xml::{
    parse, AttributeDescriptor, AttributeType, ChildDescriptor, ContentType, ErrorCode, ErrorKind,
    ParseError, ParserConfig, Schema, SchemaNode, SchemaNodeId, SinkError, TargetSink, TextValue,
    SCHEMA_VERSION,
};

#[derive(Default, Debug, PartialEq, Eq)]
struct M07Result {
    count: Option<i64>,
    label_borrowed: Option<String>,
    label_decoded: Option<String>,
}

struct M07Sink {
    result: M07Result,
}

impl M07Sink {
    fn new() -> Self {
        Self {
            result: M07Result::default(),
        }
    }
}

impl<'a> TargetSink<'a> for M07Sink {
    fn element_integer_content(&mut self, node_id: u16, content: i64) -> Result<(), SinkError> {
        if node_id == 1 {
            // count
            self.result.count = Some(content);
        }
        Ok(())
    }

    fn attribute_string(
        &mut self,
        node_id: u16,
        _name: &'a str,
        content: TextValue<'a, '_>,
    ) -> Result<(), SinkError> {
        if node_id == 0 {
            // root
            match content {
                TextValue::Borrowed(s) => self.result.label_borrowed = Some(s.to_string()),
                TextValue::Decoded(s) => self.result.label_decoded = Some(s.to_string()),
            }
        }
        Ok(())
    }
}

const SCHEMA_NODES: &[SchemaNode] = &[
    SchemaNode {
        name: "root",
        content: ContentType::Elements,
        children: &[ChildDescriptor {
            node_id: SchemaNodeId(1),
            min_occurs: 1,
            max_occurs: 1,
        }],
        attributes: &[AttributeDescriptor {
            name: "label",
            attr_type: AttributeType::String,
            required: false,
        }],
    },
    SchemaNode {
        name: "count",
        content: ContentType::Integer,
        children: &[],
        attributes: &[],
    },
];

fn parse_fixture(bytes: &[u8], config: Option<ParserConfig>) -> Result<M07Result, ParseError> {
    let schema = Schema::new(SCHEMA_NODES, SchemaNodeId(0), SCHEMA_VERSION).unwrap();
    let mut sink = M07Sink::new();
    let cfg = config.unwrap_or_default();
    parse(bytes, &schema, &mut sink, cfg)?;
    Ok(sink.result)
}

fn assert_resource_error(bytes: &[u8], config: ParserConfig) {
    let err = parse_fixture(bytes, Some(config)).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Resource);
    assert_eq!(err.code, ErrorCode::Resource);
}

#[test]
fn test_01_valid_root_without_label() {
    let input = include_bytes!("../../../tests/fixtures/m07/01_valid_root_without_label.xml");
    let res = parse_fixture(input, None).unwrap();
    assert_eq!(res.count, Some(42));
    assert_eq!(res.label_borrowed, None);
    assert_eq!(res.label_decoded, None);
}

#[test]
fn test_02_valid_plain_text_label() {
    let input = include_bytes!("../../../tests/fixtures/m07/02_valid_plain_text_label.xml");
    let res = parse_fixture(input, None).unwrap();
    assert_eq!(res.count, Some(42));
    assert_eq!(res.label_borrowed, Some("plain text".to_string()));
    assert_eq!(res.label_decoded, None);
}

#[test]
fn test_03_valid_entity_decoded_label() {
    let input = include_bytes!("../../../tests/fixtures/m07/03_valid_entity_decoded_label.xml");
    let res = parse_fixture(input, None).unwrap();
    assert_eq!(res.count, Some(42));
    assert_eq!(res.label_borrowed, None);
    assert_eq!(res.label_decoded, Some("A&B".to_string()));
}

#[test]
fn test_04_missing_count() {
    let input = include_bytes!("../../../tests/fixtures/m07/04_missing_count.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Schema);
    assert_eq!(err.code, ErrorCode::Required);
}

#[test]
fn test_05_repeated_count() {
    let input = include_bytes!("../../../tests/fixtures/m07/05_repeated_count.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Schema);
    assert_eq!(err.code, ErrorCode::Occurrence);
}

#[test]
fn test_06_duplicate_label_attribute() {
    let input = include_bytes!("../../../tests/fixtures/m07/06_duplicate_label_attribute.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Syntax);
    assert_eq!(err.code, ErrorCode::Attribute);
}

#[test]
fn test_07_unknown_attribute() {
    let input = include_bytes!("../../../tests/fixtures/m07/07_unknown_attribute.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Schema);
    assert_eq!(err.code, ErrorCode::Attribute);
}

#[test]
fn test_08_unknown_child() {
    let input = include_bytes!("../../../tests/fixtures/m07/08_unknown_child.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Schema);
    assert_eq!(err.code, ErrorCode::Order);
}

#[test]
fn test_09_integer_overflow() {
    let input = include_bytes!("../../../tests/fixtures/m07/09_integer_overflow.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Conversion);
    assert_eq!(err.code, ErrorCode::IntegerOverflow);
}

#[test]
fn test_10_mismatched_tag() {
    let input = include_bytes!("../../../tests/fixtures/m07/10_mismatched_tag.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Syntax);
    assert_eq!(err.code, ErrorCode::Mismatch);
}

#[test]
fn test_11_truncated_document() {
    let input = include_bytes!("../../../tests/fixtures/m07/11_truncated_document.xml");
    let err = parse_fixture(input, None).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Syntax);
    assert_eq!(err.code, ErrorCode::Eof);
}

#[test]
fn test_budgets_exact_and_plus_one() {
    let input = include_bytes!("../../../tests/fixtures/m07/01_valid_root_without_label.xml");

    // Exact max_document_bytes
    let exact_bytes_config = ParserConfig::builder()
        .max_document_bytes(input.len())
        .build();
    assert!(parse_fixture(input, Some(exact_bytes_config)).is_ok());

    // One under max_document_bytes
    let under_bytes_config = ParserConfig::builder()
        .max_document_bytes(input.len() - 1)
        .build();
    assert_resource_error(input, under_bytes_config);

    // The fixture produces eight tokens, including element and trailing whitespace events.
    let exact_token_config = ParserConfig::builder().max_tokens(8).build();
    let exact_tokens = parse_fixture(input, Some(exact_token_config));
    assert!(exact_tokens.is_ok(), "{exact_tokens:?}");

    let under_token_config = ParserConfig::builder().max_tokens(7).build();
    assert_resource_error(input, under_token_config);
}

#[test]
fn test_each_resource_limit_accepts_exact_and_rejects_one_under() {
    let document = include_bytes!("../../../tests/fixtures/m07/01_valid_root_without_label.xml");
    let labeled_document =
        include_bytes!("../../../tests/fixtures/m07/02_valid_plain_text_label.xml");

    assert!(parse_fixture(document, Some(ParserConfig::builder().max_depth(2).build())).is_ok());
    assert_resource_error(document, ParserConfig::builder().max_depth(1).build());

    assert!(parse_fixture(
        labeled_document,
        Some(
            ParserConfig::builder()
                .max_attributes_per_element(1)
                .build()
        )
    )
    .is_ok());
    assert_resource_error(
        labeled_document,
        ParserConfig::builder()
            .max_attributes_per_element(0)
            .build(),
    );

    assert!(parse_fixture(
        document,
        Some(ParserConfig::builder().max_children_per_element(1).build())
    )
    .is_ok());
    assert_resource_error(
        document,
        ParserConfig::builder().max_children_per_element(0).build(),
    );

    assert!(parse_fixture(
        document,
        Some(ParserConfig::builder().max_text_bytes(5).build())
    )
    .is_ok());
    assert_resource_error(document, ParserConfig::builder().max_text_bytes(4).build());

    assert!(parse_fixture(
        document,
        Some(ParserConfig::builder().max_occurrences(1).build())
    )
    .is_ok());
    assert_resource_error(document, ParserConfig::builder().max_occurrences(0).build());
}
