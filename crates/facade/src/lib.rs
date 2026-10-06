//! Main parser entry point facade.
//! Inherits all `miniml-xml-core` boundaries (no I/O, no global state, deterministic time and stack size, strict borrowing).
//! Exposes configuration structures for safe embedded limits.

#![no_std]
#![forbid(unsafe_code)]

pub use miniml_xml_core::schema::{
    AttributeDescriptor, AttributeType, ChildDescriptor, ContentType, Schema, SchemaError,
    SchemaNode, SchemaNodeId, SCHEMA_VERSION,
};
pub use miniml_xml_core::sink::{SinkError, TargetSink, TextValue};
pub use miniml_xml_core::validate::parse;
pub use miniml_xml_core::{
    ErrorCode, ErrorKind, ParseError, ParserConfig, ParserConfigBuilder, Position, ResourceBudget,
};
