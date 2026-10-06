#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(target_os = "none")]
use core::panic::PanicInfo;
use miniml_xml::{
    parse, AttributeDescriptor, AttributeType, ChildDescriptor, ContentType, ParserConfig, Schema,
    SchemaNode, SchemaNodeId, SinkError, TargetSink, TextValue, SCHEMA_VERSION,
};

#[derive(Default)]
struct M07Result {
    count: Option<i64>,
    has_label: bool,
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
        _content: TextValue<'a, '_>,
    ) -> Result<(), SinkError> {
        if node_id == 0 {
            // root
            self.result.has_label = true;
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

fn run() {
    let input = b"<root label=\"example\">\n    <count>42</count>\n</root>";

    if let Ok(schema) = Schema::new(SCHEMA_NODES, SchemaNodeId(0), SCHEMA_VERSION) {
        let mut sink = M07Sink::new();
        let config = ParserConfig::default();

        let _ = parse(input, &schema, &mut sink, config);
    }
}

#[cfg(target_os = "none")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    run();
    loop {}
}

#[cfg(not(target_os = "none"))]
fn main() {
    run();
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
