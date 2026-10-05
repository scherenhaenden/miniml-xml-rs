#![forbid(unsafe_code)]

/// Consistent with planned parser depth 32 / attrs 16.
pub const MAX_SCHEMA_DEPTH: usize = 32;
pub const MAX_SCHEMA_ATTRS: usize = 16;
pub const MAX_SCHEMA_CHILDREN: usize = 32;
pub const MAX_SCHEMA_NODES: usize = 256;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaNodeId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Empty,
    Integer,
    String,
    Elements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Integer,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChildDescriptor {
    pub node_id: SchemaNodeId,
    pub min_occurs: u32,
    pub max_occurs: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeDescriptor<'a> {
    pub name: &'a str,
    pub attr_type: AttributeType,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaNode<'a> {
    pub name: &'a str,
    pub content: ContentType,
    pub children: &'a [ChildDescriptor],
    pub attributes: &'a [AttributeDescriptor<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError<'a> {
    VersionMismatch,
    InvalidRootId,
    InvalidNodeId(SchemaNodeId),
    TooManyNodes,
    TooManyAttributes(SchemaNodeId),
    TooManyChildren(SchemaNodeId),
    DuplicateAttribute(SchemaNodeId, &'a str), // Storing name could be tricky with lifetimes, simplified to str slices
    DuplicateChild(SchemaNodeId, SchemaNodeId),
    MixedContent(SchemaNodeId),
    InvalidOccurrences(SchemaNodeId, SchemaNodeId), // parent, child
    CyclicReference(SchemaNodeId),
    DepthExceeded(SchemaNodeId),
}

#[derive(Debug)]
pub struct Schema<'a> {
    nodes: &'a [SchemaNode<'a>],
    root_id: SchemaNodeId,
    version: u32,
}

impl<'a> Schema<'a> {
    pub fn new(
        nodes: &'a [SchemaNode<'a>],
        root_id: SchemaNodeId,
        version: u32,
    ) -> Result<Self, SchemaError<'a>> {
        if version != SCHEMA_VERSION {
            return Err(SchemaError::VersionMismatch);
        }

        if nodes.len() > MAX_SCHEMA_NODES {
            return Err(SchemaError::TooManyNodes);
        }

        if (root_id.0 as usize) >= nodes.len() {
            return Err(SchemaError::InvalidRootId);
        }

        for (i, node) in nodes.iter().enumerate() {
            let id = SchemaNodeId(i as u16);

            if node.attributes.len() > MAX_SCHEMA_ATTRS {
                return Err(SchemaError::TooManyAttributes(id));
            }
            if node.children.len() > MAX_SCHEMA_CHILDREN {
                return Err(SchemaError::TooManyChildren(id));
            }

            // Check for mixed content
            if node.content != ContentType::Elements && !node.children.is_empty() {
                return Err(SchemaError::MixedContent(id));
            }

            // Check attributes for duplicates
            for j in 0..node.attributes.len() {
                for k in (j + 1)..node.attributes.len() {
                    if node.attributes[j].name == node.attributes[k].name {
                        return Err(SchemaError::DuplicateAttribute(id, node.attributes[j].name));
                    }
                }
            }

            // Check children for duplicates, valid IDs, and valid bounds
            for j in 0..node.children.len() {
                let child = &node.children[j];

                if (child.node_id.0 as usize) >= nodes.len() {
                    return Err(SchemaError::InvalidNodeId(child.node_id));
                }

                if child.min_occurs > child.max_occurs {
                    return Err(SchemaError::InvalidOccurrences(id, child.node_id));
                }

                for k in (j + 1)..node.children.len() {
                    if child.node_id == node.children[k].node_id
                        || nodes[child.node_id.0 as usize].name
                            == nodes[node.children[k].node_id.0 as usize].name
                    {
                        return Err(SchemaError::DuplicateChild(id, child.node_id));
                    }
                }
            }
        }

        Self::check_graph(nodes)?;

        Ok(Schema {
            nodes,
            root_id,
            version,
        })
    }

    // Three-color DFS visits every descriptor once, including disconnected nodes.
    // Heights memoize shared subtrees; work is O(nodes + edges), stack is fixed.
    fn check_graph(nodes: &'a [SchemaNode<'a>]) -> Result<(), SchemaError<'a>> {
        let mut colors = [0u8; MAX_SCHEMA_NODES];
        let mut heights = [1usize; MAX_SCHEMA_NODES];
        let mut stack = [(0usize, 0usize); MAX_SCHEMA_DEPTH];
        for root in 0..nodes.len() {
            if colors[root] != 0 {
                continue;
            }
            colors[root] = 1;
            stack[0] = (root, 0);
            let mut depth = 1;
            while depth > 0 {
                let (current, next) = stack[depth - 1];
                if next < nodes[current].children.len() {
                    stack[depth - 1].1 += 1;
                    let child = nodes[current].children[next].node_id.0 as usize;
                    if colors[child] == 1 {
                        return Err(SchemaError::CyclicReference(SchemaNodeId(child as u16)));
                    }
                    if colors[child] == 0 {
                        if depth == MAX_SCHEMA_DEPTH {
                            return Err(SchemaError::DepthExceeded(SchemaNodeId(child as u16)));
                        }
                        colors[child] = 1;
                        stack[depth] = (child, 0);
                        depth += 1;
                    } else {
                        heights[current] = heights[current].max(heights[child] + 1);
                        if heights[current] > MAX_SCHEMA_DEPTH {
                            return Err(SchemaError::DepthExceeded(SchemaNodeId(current as u16)));
                        }
                    }
                } else {
                    colors[current] = 2;
                    depth -= 1;
                    if depth > 0 {
                        let parent = stack[depth - 1].0;
                        heights[parent] = heights[parent].max(heights[current] + 1);
                        if heights[parent] > MAX_SCHEMA_DEPTH {
                            return Err(SchemaError::DepthExceeded(SchemaNodeId(parent as u16)));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_node(&self, id: SchemaNodeId) -> Option<&'a SchemaNode<'a>> {
        self.nodes.get(id.0 as usize)
    }

    pub fn root_id(&self) -> SchemaNodeId {
        self.root_id
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_attributes_are_accepted() {
        let attrs = [
            AttributeDescriptor {
                name: "a",
                attr_type: AttributeType::String,
                required: true,
            },
            AttributeDescriptor {
                name: "b",
                attr_type: AttributeType::Integer,
                required: false,
            },
        ];
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Empty,
            children: &[],
            attributes: &attrs,
        }];
        assert!(Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).is_ok());
    }

    fn chain<const N: usize>(reverse: bool, expected: Result<(), SchemaError<'static>>) {
        let mut edges = [ChildDescriptor {
            node_id: SchemaNodeId(0),
            min_occurs: 1,
            max_occurs: 1,
        }; N];
        let mut nodes = [SchemaNode {
            name: "node",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        }; N];
        for (i, edge) in edges.iter_mut().enumerate() {
            edge.node_id = SchemaNodeId(if reverse { i.saturating_sub(1) } else { i + 1 } as u16);
        }
        for (i, node) in nodes.iter_mut().enumerate() {
            if (reverse && i > 0) || (!reverse && i + 1 < N) {
                node.content = ContentType::Elements;
                node.children = &edges[i..i + 1];
            }
        }
        assert_eq!(
            Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).map(|_| ()),
            expected
        );
    }

    #[test]
    fn graph_depth_and_shared_subtree_boundaries() {
        chain::<32>(false, Ok(()));
        chain::<33>(false, Err(SchemaError::DepthExceeded(SchemaNodeId(32))));
        chain::<32>(true, Ok(()));
        chain::<33>(true, Err(SchemaError::DepthExceeded(SchemaNodeId(32))));
    }

    #[test]
    fn shared_subtree_height_propagates_to_parent() {
        let mut edges = [ChildDescriptor {
            node_id: SchemaNodeId(0),
            min_occurs: 1,
            max_occurs: 1,
        }; 32];
        for (i, edge) in edges.iter_mut().enumerate() {
            edge.node_id = SchemaNodeId((i + 2) as u16);
        }
        edges[31].node_id = SchemaNodeId(1);
        let root_edges = [
            ChildDescriptor {
                node_id: SchemaNodeId(1),
                min_occurs: 1,
                max_occurs: 1,
            },
            ChildDescriptor {
                node_id: SchemaNodeId(32),
                min_occurs: 1,
                max_occurs: 1,
            },
        ];
        let mut nodes = [SchemaNode {
            name: "node",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        }; 33];
        nodes[0].content = ContentType::Elements;
        nodes[0].children = &root_edges;
        for i in 1..31 {
            nodes[i].content = ContentType::Elements;
            nodes[i].children = &edges[i - 1..i];
        }
        nodes[32].name = "shared";
        nodes[32].content = ContentType::Elements;
        nodes[32].children = &edges[31..32];
        assert_eq!(
            Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err(),
            SchemaError::DepthExceeded(SchemaNodeId(0))
        );
    }

    #[test]
    fn unreachable_cycle_is_rejected() {
        let edges = [ChildDescriptor {
            node_id: SchemaNodeId(1),
            min_occurs: 0,
            max_occurs: 1,
        }];
        let nodes = [
            SchemaNode {
                name: "root",
                content: ContentType::Empty,
                children: &[],
                attributes: &[],
            },
            SchemaNode {
                name: "unreachable",
                content: ContentType::Elements,
                children: &edges,
                attributes: &[],
            },
        ];
        assert_eq!(
            Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err(),
            SchemaError::CyclicReference(SchemaNodeId(1))
        );
    }

    #[test]
    fn test_valid_schema() {
        let nodes = [
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
                name: "child",
                content: ContentType::String,
                children: &[],
                attributes: &[AttributeDescriptor {
                    name: "attr",
                    attr_type: AttributeType::String,
                    required: true,
                }],
            },
        ];

        let schema = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap();
        assert_eq!(schema.root_id(), SchemaNodeId(0));
        assert_eq!(schema.version(), SCHEMA_VERSION);
        assert_eq!(schema.get_node(SchemaNodeId(1)).unwrap().name, "child");
    }

    #[test]
    fn test_version_mismatch() {
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        }];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION + 1).unwrap_err();
        assert_eq!(err, SchemaError::VersionMismatch);
    }

    #[test]
    fn test_invalid_root_id() {
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        }];
        let err = Schema::new(&nodes, SchemaNodeId(1), SCHEMA_VERSION).unwrap_err();
        assert_eq!(err, SchemaError::InvalidRootId);
    }

    #[test]
    fn test_too_many_nodes() {
        let node = SchemaNode {
            name: "root",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        };
        let nodes = [node; MAX_SCHEMA_NODES + 1];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(err, SchemaError::TooManyNodes);
    }

    #[test]
    fn test_duplicate_attributes() {
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Empty,
            children: &[],
            attributes: &[
                AttributeDescriptor {
                    name: "attr",
                    attr_type: AttributeType::String,
                    required: true,
                },
                AttributeDescriptor {
                    name: "attr",
                    attr_type: AttributeType::Integer,
                    required: false,
                },
            ],
        }];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(
            err,
            SchemaError::DuplicateAttribute(SchemaNodeId(0), "attr")
        );
    }

    #[test]
    fn test_duplicate_children() {
        let nodes = [
            SchemaNode {
                name: "root",
                content: ContentType::Elements,
                children: &[
                    ChildDescriptor {
                        node_id: SchemaNodeId(1),
                        min_occurs: 1,
                        max_occurs: 1,
                    },
                    ChildDescriptor {
                        node_id: SchemaNodeId(1),
                        min_occurs: 0,
                        max_occurs: 2,
                    },
                ],
                attributes: &[],
            },
            SchemaNode {
                name: "child",
                content: ContentType::Empty,
                children: &[],
                attributes: &[],
            },
        ];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(
            err,
            SchemaError::DuplicateChild(SchemaNodeId(0), SchemaNodeId(1))
        );
    }

    #[test]
    fn same_named_children_are_rejected_even_with_distinct_ids() {
        let children = [
            ChildDescriptor {
                node_id: SchemaNodeId(1),
                min_occurs: 0,
                max_occurs: 1,
            },
            ChildDescriptor {
                node_id: SchemaNodeId(2),
                min_occurs: 0,
                max_occurs: 1,
            },
        ];
        let nodes = [
            SchemaNode {
                name: "root",
                content: ContentType::Elements,
                children: &children,
                attributes: &[],
            },
            SchemaNode {
                name: "item",
                content: ContentType::Integer,
                children: &[],
                attributes: &[],
            },
            SchemaNode {
                name: "item",
                content: ContentType::String,
                children: &[],
                attributes: &[],
            },
        ];
        assert_eq!(
            Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err(),
            SchemaError::DuplicateChild(SchemaNodeId(0), SchemaNodeId(1))
        );
    }

    #[test]
    fn test_mixed_content() {
        let nodes = [
            SchemaNode {
                name: "root",
                content: ContentType::String,
                children: &[ChildDescriptor {
                    node_id: SchemaNodeId(1),
                    min_occurs: 1,
                    max_occurs: 1,
                }],
                attributes: &[],
            },
            SchemaNode {
                name: "child",
                content: ContentType::Empty,
                children: &[],
                attributes: &[],
            },
        ];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(err, SchemaError::MixedContent(SchemaNodeId(0)));
    }

    #[test]
    fn test_invalid_occurrences() {
        let nodes = [
            SchemaNode {
                name: "root",
                content: ContentType::Elements,
                children: &[ChildDescriptor {
                    node_id: SchemaNodeId(1),
                    min_occurs: 2,
                    max_occurs: 1,
                }],
                attributes: &[],
            },
            SchemaNode {
                name: "child",
                content: ContentType::Empty,
                children: &[],
                attributes: &[],
            },
        ];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(
            err,
            SchemaError::InvalidOccurrences(SchemaNodeId(0), SchemaNodeId(1))
        );
    }

    #[test]
    fn test_invalid_child_id() {
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &[ChildDescriptor {
                node_id: SchemaNodeId(1),
                min_occurs: 1,
                max_occurs: 1,
            }],
            attributes: &[],
        }];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(err, SchemaError::InvalidNodeId(SchemaNodeId(1)));
    }

    #[test]
    fn test_cyclic_reference() {
        let nodes = [
            SchemaNode {
                name: "n0",
                content: ContentType::Elements,
                children: &[ChildDescriptor {
                    node_id: SchemaNodeId(1),
                    min_occurs: 1,
                    max_occurs: 1,
                }],
                attributes: &[],
            },
            SchemaNode {
                name: "n1",
                content: ContentType::Elements,
                children: &[ChildDescriptor {
                    node_id: SchemaNodeId(0),
                    min_occurs: 1,
                    max_occurs: 1,
                }],
                attributes: &[],
            },
        ];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(err, SchemaError::CyclicReference(SchemaNodeId(0)));
    }

    #[test]
    fn test_too_many_attributes() {
        let attrs: [AttributeDescriptor; MAX_SCHEMA_ATTRS + 1] = [AttributeDescriptor {
            name: "attr",
            attr_type: AttributeType::String,
            required: false,
        }; MAX_SCHEMA_ATTRS + 1];
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Empty,
            children: &[],
            attributes: &attrs,
        }];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(err, SchemaError::TooManyAttributes(SchemaNodeId(0)));
    }

    #[test]
    fn test_too_many_children() {
        let children: [ChildDescriptor; MAX_SCHEMA_CHILDREN + 1] = [ChildDescriptor {
            node_id: SchemaNodeId(0),
            min_occurs: 1,
            max_occurs: 1,
        };
            MAX_SCHEMA_CHILDREN + 1];
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Elements,
            children: &children,
            attributes: &[],
        }];
        let err = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap_err();
        assert_eq!(err, SchemaError::TooManyChildren(SchemaNodeId(0)));
    }

    #[test]
    fn test_get_node() {
        let nodes = [SchemaNode {
            name: "root",
            content: ContentType::Empty,
            children: &[],
            attributes: &[],
        }];
        let schema = Schema::new(&nodes, SchemaNodeId(0), SCHEMA_VERSION).unwrap();
        assert_eq!(schema.get_node(SchemaNodeId(0)), Some(&nodes[0]));
        assert_eq!(schema.get_node(SchemaNodeId(1)), None);
    }
}
