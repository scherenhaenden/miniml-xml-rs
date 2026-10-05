//! Core XML parser structures and boundaries.
//! This crate guarantees zero I/O, no global state, and strictly bounded time/stack/lifetime
//! behavior when processing input via `#![no_std]` and fixed resource limits.

#![no_std]
#![forbid(unsafe_code)]

pub mod budget;
pub mod config;
pub mod convert;
pub mod cursor;
pub mod entity;
pub mod error;
pub mod name;
pub mod schema;
pub mod sink;
pub mod text;
pub mod tokenizer;

pub use budget::ResourceBudget;
pub use config::{ParserConfig, ParserConfigBuilder};
pub use error::{ErrorCode, ErrorKind, ParseError, Position};
