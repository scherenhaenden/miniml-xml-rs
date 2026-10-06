use miniml_xml::{
    parse, AttributeDescriptor, AttributeType, ChildDescriptor, ContentType, ParserConfig, Schema,
    SchemaNode, SchemaNodeId, SinkError, TargetSink, TextValue, SCHEMA_VERSION,
};

#[derive(Default, Debug)]
struct M07Result {
    count: Option<i64>,
    label: Option<String>,
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
            self.result.label = Some(content.as_str().to_string());
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

fn main() {
    let input = b"<root label=\"example\">\n    <count>42</count>\n</root>";

    let schema = Schema::new(SCHEMA_NODES, SchemaNodeId(0), SCHEMA_VERSION).unwrap();
    let mut sink = M07Sink::new();
    let config = ParserConfig::default();

    match parse(input, &schema, &mut sink, config) {
        Ok(_) => println!("Success: {:?}", sink.result),
        Err(e) => println!("Error: {:?}", e),
    }
}
