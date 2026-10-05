# M04: Static Schema Contracts

## Overview
This document specifies the exact interfaces, memory/time complexity, lifetimes, and constraints for the Static Schema module (M04). The goal of this module is to provide an immutable flat representation of an XML schema against which to evaluate decoded documents.

## Module Scope
The module provides zero-allocation `#![no_std]` interfaces. All data structures employ flat slices array bindings instead of heap-allocated structures or dynamic typing. The interfaces limit the depth, occurrences, branches, and cyclical references of tree processing by establishing deterministic hard caps on tree dimensions.

## Components

### Schema Definition
The core components reside in `crates/core/src/schema.rs`.

- **`Schema`**: Top-level immutable structure defined around a static set of slices and configuration arrays.
- **`SchemaNode`**: Descriptor of an XML node structure. Contains element-only children references or scalar content requirements, and a list of structural attribute descriptors.
- **`ChildDescriptor`**: Enforces strict `min_occurs`/`max_occurs` bounds on immediate element children.
- **`AttributeDescriptor`**: Specifies name, type and optionality of properties bounding an XML element.
- **`SchemaError`**: Enumeration of static construction-time failures (e.g. invalid lengths, mixed content blocks, structural cyclic bounds violations).
- Construction validates each node and attribute name as a non-empty XML 1.0 `Name` using the shared M02 rules; namespace QName semantics remain outside this profile.

### Conversion Logic
Conversion functions reside in `crates/core/src/convert.rs`.

- **`parse_integer(input: &str) -> Result<i64, ConvertError>`**: Consumes XML text slices stripped of whitespace. Interprets and verifies sign definitions against rigorous boundaries producing an `i64`. Errors are structured into `ConvertError`.
- **`parse_string(input: &str) -> Result<&str, ConvertError>`**: Provides an identity pass-through wrapper for strings.

### Sink Abstraction
The traversal logic resides in `crates/core/src/sink.rs`.

- **`TargetSink<'input>`**: Receives node IDs and integer values or `TextValue<'input, 'call>`. `TextValue::Borrowed` can be retained for the input lifetime; `TextValue::Decoded` references a temporary fixed buffer and must be copied during the callback. `as_str()` borrows either form, while `borrowed()` returns `Some` only for input-backed text.
- **`SinkError`**: Mechanism to gracefully halt traversal with application-defined rejections (`RejectedByApplication`). Note: In this architecture, parse errors mean target output is incomplete, therefore applications assume no success guarantees until an entire document is processed.

## Memory and Time Characteristics

- **Memory**: O(1) dynamic memory. Total runtime structures strictly depend on `.rodata` and explicit compile-time dimension constants (`MAX_SCHEMA_NODES`, `MAX_SCHEMA_DEPTH`, `MAX_SCHEMA_ATTRS`).
- **Time**: Structural duplicate checks are bounded by 16 attributes and 32 child descriptors per node. Graph validation uses iterative three-color DFS and memoized heights: O(nodes + edges), including shared subtrees and disconnected components. Depth above 32 and cycles are separate errors; the verifier does not recurse.
- **Lifetimes**: Schema descriptors borrow their backing slices. Input strings and temporary decoded strings have distinct callback lifetimes enforced by `TextValue`; consumers may retain only input-backed strings or copy temporary text into their own bounded storage.

## Sample Schema Definition

```rust
use crate::schema::{Schema, SchemaNode, SchemaNodeId, ChildDescriptor, AttributeDescriptor, ContentType, AttributeType, SCHEMA_VERSION};

const SCHEMA_NODES: &[SchemaNode] = &[
    SchemaNode {
        name: "root",
        content: ContentType::Elements,
        children: &[ChildDescriptor { node_id: SchemaNodeId(1), min_occurs: 1, max_occurs: 1 }],
        attributes: &[],
    },
    SchemaNode {
        name: "child",
        content: ContentType::Integer,
        children: &[],
        attributes: &[AttributeDescriptor { name: "attr", attr_type: AttributeType::String, required: false }],
    },
];

let schema = Schema::new(SCHEMA_NODES, SchemaNodeId(0), SCHEMA_VERSION).unwrap();
```
