# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete `no_std`, zero-allocation core parser with deterministic bounded execution (`miniml-xml-core`).
- User-facing facade crate (`miniml-xml`) providing the primary entry points and configuration.
- Support for static schema descriptor validation (`Schema`, `SchemaNode`) without dynamic parsing.
- Typed `TargetSink` extraction enforcing strict order, cardinality, and integer boundary checks (`i64`).
- Comprehensive error model returning explicit position and categoric codes (`ParseError`, `ErrorKind`, `ErrorCode`).
- Full UTF-8 tokenizer with entity decoding, newline normalization, and strict XML 1.0 (Fifth Edition) name validation.
- Unit tests, facade consumer tests, an adversarial input corpus, and a bounded mutation gate.
