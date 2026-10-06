# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-06

### Added
- Bounded, allocation-free `no_std` core parser for the documented UTF-8 XML profile (`miniml-xml-core`).
- User-facing facade crate (`miniml-xml`) providing the primary entry points and configuration.
- Application-authored static schema descriptors (`Schema`, `SchemaNode`); no runtime XSD parsing.
- Typed `TargetSink` extraction enforcing strict order, cardinality, and integer boundary checks (`i64`).
- Structured parse errors with positions and exact violation codes, plus schema-construction errors.
- UTF-8 tokenizer for the supported XML subset, including entity decoding, newline normalization, and XML 1.0 (Fifth Edition) name validation.
- Unit and facade consumer tests, a deterministic adversarial corpus, and a bounded mutation gate.
