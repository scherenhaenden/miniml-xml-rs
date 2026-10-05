# Design Patterns and Architectural Patterns

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 13-15.

Status: documentation baseline; the proposed implementation, APIs, builds and quality gates are not yet implemented. Repository name: `miniml-xml-rs`. Crate and ABI names in examples remain proposals from the source.

The project should use patterns because they simplify safety, testability or compatibility, not because a catalog exists. The table below deliberately considers a broad set of patterns and marks whether each belongs in this project.

| Pattern | Decision | Application / rationale |
| --- | --- | --- |
| Hexagonal / Ports & Adapters | USE | Core parser depends on ports such as TargetSink and Hook; FFI/codegen/application- specific adapters stay outside. |
| Clean Architecture dependency rule | USE | Dependencies point inward toward the safe core; host tooling and C ABI cannot leak into parser logic. |
| Facade | USE | Parser offers one small entry point hiding tokenizer/state/validator/extraction internals. |
| Builder | USE | ParserConfig::builder() constructs explicit resource limits and strictness flags. |
| Strategy | USE | TargetSink, text conversion policy and optional hook behavior can vary without changing the state machine. |
| State | USE | XML syntax is implemented as an explicit state machine rather than ad-hoc nested conditionals. |
| Composite | USE | Generated SchemaNode hierarchy represents nested XML structures. |
| Specification | USE | Facets, occurrence constraints and allowed child/attribute rules are composable validation specifications. |
| Adapter | USE | C ABI adapter, generated model adapter and optional std helpers translate external interfaces into safe core contracts. |
| Visitor | USE/CONDITIONAL | Useful for host-side Schema IR traversal and emitters; avoid mandatory runtime visitor allocation. |
| Observer / Callback | CONDITIONAL | Element completion hooks support streaming consumption. Feature-gated to preserve minimal footprint. |
| Factory | USE | Codegen maps Schema IR nodes to emitted Rust/C type models through explicit factories/functions. |
| Command | USE IN TOOLING | Codegen passes can be represented as deterministic commands; runtime does not need it. |
| Chain of Responsibility | CONDITIONAL | Validation stages may be composed as a static chain; avoid dynamic dispatch in core. |
| Interpreter | CONDITIONAL | Schema rules act like a tiny declarative validation language, but runtime should execute generated descriptors rather than a generic interpreter. |
| Flyweight | USE | Schema names/metadata are static and reused; parser events borrow input slices instead of duplicating text. |
| Iterator | USE | Tokenizer can expose a pull/event iterator internally, enabling isolated tests and simple composition. |
| Newtype | USE | ByteOffset, Depth, SchemaNodeId and bounded counts prevent unit confusion and illegal cross-use. |
| Typestate | USE SELECTIVELY | Generator/config builders can make invalid construction states unrepresentable; avoid overcomplicating parser call sites. |
| RAII | USE | Any temporary resource or alloc-backed target is released automatically; especially relevant in std/alloc and test harnesses. |
| Dependency Injection | USE STATIC | Inject schema, config, sinks and hooks through generics/references; no service locator. |
| Null Object | OPTIONAL | A no-op Hook can remove Option branching where it improves generated code without size cost. |
| Bridge | NOT NEEDED | No independent class hierarchies justify a Bridge; traits/adapters already solve the variation. |
| Decorator | NOT CORE | Could decorate a TargetSink for tracing, but avoid runtime wrappers in the minimal embedded profile. |
| Proxy | NOT NEEDED | No remote/lazy object access requirement. |
| Mediator | NOT NEEDED | Components interact through a deterministic pipeline; a mediator would add indirection. |
| Memento | NOT NEEDED V1 | The parser is one-shot; resumable/rollback parsing is explicitly out of v1 scope. |
| Prototype | NOT NEEDED | Generated immutable schema descriptors do not require cloning prototypes. |
| Singleton | REJECT | Global mutable parser/schema state would harm reentrancy and testing. |
| Service Locator | REJECT | Hidden dependencies conflict with deterministic embedded design. |
| Active Object / Actor | REJECT CORE | Concurrency is application-level. Core parsing is synchronous and deterministic. |
| Repository | NOT APPLICABLE | No persistence abstraction belongs in an XML parser. |
| CQRS / Event Sourcing | NOT APPLICABLE | No domain command/read model or event persistence problem exists. |

## Rust-specific design rules

- Make invalid states unrepresentable where it reduces runtime checks without creating unusable APIs.

- Prefer enums over integer status codes inside Rust; translate to integers only at FFI.

- Prefer borrowed slices and lifetimes to ownership/copying when the source buffer outlives the result.

- Use exhaustive matches to make state transitions auditable.

- Keep generic abstractions static where code size remains acceptable; use trait objects only when feature-driven flexibility justifies them.

- Do not use unsafe as a performance shortcut. Performance claims require benchmarks before an unsafe exception is even considered.
