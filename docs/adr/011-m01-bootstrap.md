# ADR 011: Milestone 0.1.0 Bootstrap and Profile Constraints

**Date**: 2026-10-05

## Context

We are implementing the first foundational piece (Milestone 0.1.0) of the `miniml-xml-rs` repository. Before releasing a working parser, we need a standard Cargo workspace layout, core definitions for errors and configuration, and resource budgets that ensure robust embedded execution without relying on heap allocation or standard libraries. We must narrow the scope to an initial version 0.1.0 profile constraints that allow building a vertical slice (root, required integer child, optional string attribute).

## Decisions

1. **Workspace and Crates:**
   - Establish a standard Cargo workspace with a `crates/core` containing the safe logic without external dependencies and `crates/facade` representing the user-facing API surface. Both libraries are strictly `#![no_std]` and `#![forbid(unsafe_code)]`.
   - Packages are currently named `miniml-xml-core` and `miniml-xml` inside the workspace (published set to `false`).

2. **MSRV and Versioning:**
   - The Minimum Supported Rust Version (MSRV) is set to `1.85`.
   - The initial edition is `2021`.
   - Work is released under the `MIT` license, with explicit ownership (Edward Flores).

3. **Error Handling & Position:**
   - Instead of heap-allocated strings, errors are structured into a 3-part tuple: `ErrorKind` (high-level category), `ErrorCode` (specific failure reason), and `Position` (byte offset, 1-based line, 1-based scalar column).
   - Display traits are implemented manually using `core::fmt`.

4. **Resource Bounds & Configuration:**
   - `ParserConfig` introduces strict resource limitations mapped directly to embedded needs.
   - **Defaults**: document bytes = 4096, max nesting depth = 16 (hard bound 32), attributes per element = 8 (hard bound 16), children per element = 64, text bytes = 256, occurrences = 32, max tokens = 4096.
   - Values of zero imply that a particular feature is structurally forbidden (e.g., zero attributes mean no attributes). Limits are enforced actively with `ResourceBudget`, returning structured errors gracefully instead of panicking.

5. **Profile Constraints for 0.1.0:**
   - Only UTF-8 valid XML is parsed.
   - The vertical slice for this version targets schemas that expect only static integer properties or string elements.
   - A fuller complement of primitive facets will arrive in milestone 0.2; external generated configurations and a C FFI layer will be explored down the road.

## Consequences

- We can implement bounded parsing that works reliably on embedded devices without dynamically allocating memory for error tracking or resource bookkeeping.
- The 0.1.0 focus isolates our work cleanly from future complex tasks (like XSD generation or expansive XML features).
- The use of typed `ErrorCode` allows client applications robust programmatic control flow on encountering errors.
