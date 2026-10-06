use miniml_xml::{
    parse, AttributeDescriptor, AttributeType, ChildDescriptor, ContentType, ErrorCode, ErrorKind,
    ParseError, ParserConfig, Schema, SchemaNode, SchemaNodeId, TargetSink, SCHEMA_VERSION,
};

const MAX_CORPUS_BYTES: usize = 256;

struct TestSink;

impl<'a> TargetSink<'a> for TestSink {}

fn get_schema() -> Schema<'static> {
    const NODES: &[SchemaNode] = &[
        SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &[ChildDescriptor {
                node_id: SchemaNodeId(1),
                min_occurs: 0,
                max_occurs: 1,
            }],
            attributes: &[],
        },
        SchemaNode {
            name: "child",
            content: ContentType::Elements,
            children: &[ChildDescriptor {
                node_id: SchemaNodeId(2),
                min_occurs: 0,
                max_occurs: 1,
            }],
            attributes: &[AttributeDescriptor {
                name: "id",
                attr_type: AttributeType::Integer,
                required: true,
            }],
        },
        SchemaNode {
            name: "text",
            content: ContentType::String,
            children: &[],
            attributes: &[],
        },
    ];

    Schema::new(NODES, SchemaNodeId(0), SCHEMA_VERSION).unwrap()
}

fn get_deep_schema() -> Schema<'static> {
    const NODES: &[SchemaNode] = &[
        SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &[ChildDescriptor {
                node_id: SchemaNodeId(1),
                min_occurs: 1,
                max_occurs: 1,
            }],
            attributes: &[],
        },
        SchemaNode {
            name: "first",
            content: ContentType::Elements,
            children: &[ChildDescriptor {
                node_id: SchemaNodeId(2),
                min_occurs: 1,
                max_occurs: 1,
            }],
            attributes: &[],
        },
        SchemaNode {
            name: "second",
            content: ContentType::Elements,
            children: &[ChildDescriptor {
                node_id: SchemaNodeId(3),
                min_occurs: 1,
                max_occurs: 1,
            }],
            attributes: &[],
        },
        SchemaNode {
            name: "third",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        },
    ];

    Schema::new(NODES, SchemaNodeId(0), SCHEMA_VERSION).unwrap()
}

fn bounded_config(max_depth: usize, max_text_bytes: usize, max_tokens: usize) -> ParserConfig {
    ParserConfig::builder()
        .max_document_bytes(MAX_CORPUS_BYTES)
        .max_depth(max_depth)
        .max_attributes_per_element(4)
        .max_children_per_element(8)
        .max_text_bytes(max_text_bytes)
        .max_occurrences(16)
        .max_tokens(max_tokens)
        .build()
}

macro_rules! test_fixture {
    ($name:ident, $file:expr, $config:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let input = include_bytes!(concat!("../../../tests/fixtures/m08/", $file));
            assert!(input.len() <= MAX_CORPUS_BYTES, "fixture exceeds byte cap");

            let schema = get_schema();
            let mut sink = TestSink;
            let result = parse(input, &schema, &mut sink, $config);
            assert_eq!(result, $expected);
        }
    };
}

test_fixture!(
    valid_control,
    "valid.xml",
    bounded_config(8, 64, 64),
    Ok(())
);

test_fixture!(
    truncated_constructs,
    "truncated.xml",
    bounded_config(8, 64, 64),
    Err(ParseError::new(
        ErrorKind::Syntax,
        ErrorCode::Eof,
        miniml_xml::Position::new(26, 3, 1)
    ))
);

test_fixture!(
    malformed_comment,
    "malformed_comment.xml",
    bounded_config(8, 64, 64),
    Err(ParseError::new(
        ErrorKind::Syntax,
        ErrorCode::Comment,
        miniml_xml::Position::new(26, 2, 20)
    ))
);

test_fixture!(
    malformed_entity,
    "malformed_entity.xml",
    bounded_config(8, 64, 64),
    Err(ParseError::new(
        ErrorKind::Unsupported,
        ErrorCode::Reference,
        miniml_xml::Position::new(47, 3, 22)
    ))
);

test_fixture!(
    duplicate_field,
    "duplicate_field.xml",
    bounded_config(8, 64, 64),
    Err(ParseError::new(
        ErrorKind::Schema,
        ErrorCode::Occurrence,
        miniml_xml::Position::new(70, 5, 5)
    ))
);

test_fixture!(
    unknown_field,
    "unknown_field.xml",
    bounded_config(8, 64, 64),
    Err(ParseError::new(
        ErrorKind::Schema,
        ErrorCode::Order,
        miniml_xml::Position::new(11, 2, 5)
    ))
);

#[test]
fn excessive_nesting_hits_the_depth_limit() {
    let input = include_bytes!("../../../tests/fixtures/m08/excessive_nesting.xml");
    assert!(input.len() <= MAX_CORPUS_BYTES, "fixture exceeds byte cap");

    let schema = get_deep_schema();
    let mut sink = TestSink;
    let config = bounded_config(3, 64, 64);

    assert_eq!(
        parse(input, &schema, &mut sink, config),
        Err(ParseError::new(
            ErrorKind::Resource,
            ErrorCode::Resource,
            miniml_xml::Position::new(21, 1, 22)
        ))
    );
}

test_fixture!(
    extra_roots,
    "extra_roots.xml",
    bounded_config(8, 64, 64),
    Err(ParseError::new(
        ErrorKind::Syntax,
        ErrorCode::MultipleRoot,
        miniml_xml::Position::new(74, 6, 1)
    ))
);

test_fixture!(
    trailing_content,
    "trailing_content.xml",
    bounded_config(8, 64, 64),
    Err(ParseError::new(
        ErrorKind::Syntax,
        ErrorCode::MixedContent,
        miniml_xml::Position::new(53, 1, 54)
    ))
);

test_fixture!(
    text_limit,
    "text_limit.xml",
    bounded_config(8, 10, 64),
    Err(ParseError::new(
        ErrorKind::Resource,
        ErrorCode::Resource,
        miniml_xml::Position::new(40, 3, 15)
    ))
);

test_fixture!(
    token_limit,
    "token_limit.xml",
    bounded_config(8, 64, 10),
    Err(ParseError::new(
        ErrorKind::Resource,
        ErrorCode::Resource,
        miniml_xml::Position::new(66, 5, 1)
    ))
);
