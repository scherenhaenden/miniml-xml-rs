# M02 Contracts: Cursor and Name

## Scope
This document outlines the exact public interfaces, time, and lifetime contracts for the `cursor` and `name` modules implemented in Milestone 0.1.0 (Task M02).

## Constraints
- Safe Rust only (`#![no_std]`).
- No heap allocation or external dependencies.
- Tokenizer-oriented logic.

## Cursor Module

### Types
- `Position`:
  - `byte_offset: usize`: 0-based offset into the original byte slice.
  - `line: usize`: 1-based line number.
  - `column: usize`: 1-based Unicode scalar value column within the current line.
- `CursorError`:
  - `InvalidUtf8 { offset: usize }`
  - `InvalidXmlChar { offset: usize }`
  - `InvalidOffset { offset: usize }`
- `Cursor<'a>`: A lightweight read-only pointer into the source data.
  - Contains a `&'a str` slice (reference to the original buffer).

### Interfaces
- `Cursor::new(bytes: &'a [u8]) -> Result<Cursor<'a>, CursorError>`: Creates a new cursor. Time complexity is $O(N)$ due to UTF-8 validation.
- `Cursor::position(&self) -> Position`: Returns the current cursor position. $O(1)$ time.
- `Cursor::remaining(&self) -> &'a str`: Returns the remaining string slice. $O(1)$ time.
- `Cursor::is_eof(&self) -> bool`: Returns true if end-of-file is reached. $O(1)$ time.
- `Cursor::peek(&self) -> Result<Option<char>, CursorError>`: Returns the next scalar without advancing. $O(1)$ time. Validates XML characters.
- `Cursor::consume(&mut self) -> Result<Option<char>, CursorError>`: Returns the next scalar and advances the cursor, updating `Position`. $O(1)$ time. Validates XML characters and correctly handles CRLF.
- `Cursor::advance(&mut self, bytes: usize) -> Result<(), CursorError>`: Advances the cursor precisely by a byte amount, stopping at the target boundary. $O(K)$ time where $K$ is the number of characters advanced. Validates XML characters and calculates lines/columns over the advanced range.

## Name Module

### Types
- `NameError`:
  - `InvalidStartChar { offset: usize }`
  - `InvalidNameChar { offset: usize }`
  - `Eof { offset: usize }`
  - `CursorError(CursorError)`

### Interfaces
- `is_name_start_char(c: char) -> bool`: Validates XML 1.0 Name Start Characters (5th Edition). $O(1)$ time.
- `is_name_char(c: char) -> bool`: Validates XML 1.0 Name Continuation Characters (5th Edition). $O(1)$ time.
- `parse_name<'a>(cursor: &mut Cursor<'a>) -> Result<&'a str, NameError>`: Parses a valid XML name from the current cursor position, advancing the cursor past the name. The returned `&'a str` borrows from the cursor's underlying data. Time complexity is $O(N)$ where $N$ is the byte length of the name. Stops at EOF or first non-name character without throwing an error if a valid start character was found.
