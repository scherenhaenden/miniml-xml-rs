# miniml-xml-rs

A memory-safe, schema-validating XML parser for embedded Rust, inspired by miniML-Parser.

## Current status

The 0.1.0 parser scope is implemented in `miniml-xml-core` with a public facade in `miniml-xml`. The core is `#![no_std]`, forbids unsafe code, and uses fixed storage with deterministic resource limits. It validates XML against statically authored schema descriptors.

XSD generation and C ABI interoperability are not part of 0.1.0.

## Quick Start

The example below uses the `miniml-xml` facade. The crate is not published yet, so add it to a workspace with a local path dependency before compiling the example. From this checkout, run the included consumer with `cargo run -p miniml-xml --example m07_consumer`. The `String` allocation shown belongs to the application's sink; the parser core itself does not allocate.

```rust
use miniml_xml::{
    parse, AttributeDescriptor, AttributeType, ChildDescriptor, ContentType, ParserConfig, Schema,
    SchemaNode, SchemaNodeId, SinkError, TargetSink, TextValue, SCHEMA_VERSION,
};

#[derive(Default, Debug)]
struct MyResult {
    count: Option<i64>,
    label: Option<String>,
}

struct MySink {
    result: MyResult,
}

impl<'a> TargetSink<'a> for MySink {
    fn element_integer_content(&mut self, node_id: u16, content: i64) -> Result<(), SinkError> {
        if node_id == 1 {
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
            // Using `as_str().to_string()` requires `alloc` in your application
            self.result.label = Some(content.as_str().to_string());
        }
        Ok(())
    }
}

// 1. Define the schema descriptors
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

    // 2. Initialize schema and boundaries
    let schema = Schema::new(SCHEMA_NODES, SchemaNodeId(0), SCHEMA_VERSION).unwrap();
    let config = ParserConfig::default();

    // 3. Initialize sink and parse
    let mut sink = MySink { result: MyResult::default() };
    match parse(input, &schema, &mut sink, config) {
        Ok(_) => println!("Success: {:?}", sink.result),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Documentation

Start with the [documentation index](docs/README.md).

- [Supported Profile](docs/supported-profile.md) (XML, static schemas, bounds, errors, `no_std`)
- [Project brief](docs/project-brief.md)
- [Requirements: FR, NFR, SEC and COMP](docs/requirements.md)
- [XML/XSD profiles, API, errors, features and resource budgets](docs/specification.md)
- [Architecture and ownership](docs/architecture.md)
- [Testing and standalone 100% unit coverage](docs/testing.md)
- [Implementation roadmap](docs/roadmap.md)
- [Acceptance criteria](docs/acceptance.md)
- [Architecture decisions](docs/adr/baseline.md)
- [Contributing](CONTRIBUTING.md) and [security design baseline](SECURITY.md)

## Source and naming

The source is [embedded-xml-schema_project_specification.pdf](docs/reference/embedded-xml-schema_project_specification.pdf), architecture baseline v0.1, dated 5 October 2026. Markdown documentation keeps its requirements in English.

miniML-Parser is the behavioral reference described by the PDF. Its reported footprint and capabilities are historical source claims, not measurements or verified compatibility of this Rust project. See [source baseline](docs/source-baseline.md).

## License

This project is licensed under the [MIT License](LICENSE).
