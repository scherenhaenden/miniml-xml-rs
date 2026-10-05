//! Core XML parser structures and boundaries.
//! This crate guarantees zero I/O, no global state, and strictly bounded time/stack/lifetime
//! behavior when processing input via `#![no_std]` and fixed resource limits.

#![no_std]
#![forbid(unsafe_code)]

pub mod budget;
pub mod config;
pub mod error;

pub use budget::ResourceBudget;
pub use config::{ParserConfig, ParserConfigBuilder};
pub use error::{ErrorCode, ErrorKind, ParseError, Position};
