# Documentation index

These documents adapt all twelve sections and both appendices of the [source PDF](reference/embedded-xml-schema_project_specification.pdf), version 0.1, 5 October 2026. Source page ranges appear in each document. Tables retain requirement IDs and design decisions; the five diagrams are represented in Mermaid.

The baseline is proposed architecture. Nothing in these documents establishes that the implementation, coverage or release gates have already passed. User-selected repository name: `miniml-xml-rs`; source crate/ABI identifiers remain illustrative.

| Document | Source content |
| --- | --- |
| [Project brief](project-brief.md) | Section 1: problem, users, success criteria and non-goals |
| [Requirements](requirements.md) | Section 2: 20 functional, 15 non-functional, 10 security and 5 compatibility requirements |
| [Software specification](specification.md) | Section 3: XML/XSD profiles, API, errors, features and budgets |
| [Architecture](architecture.md) | Section 4: runtime, codegen, ownership and C boundary |
| [Design patterns](design-patterns.md) | Section 5: selected, conditional and rejected patterns |
| [Test architecture](testing.md) | Section 6: unit, property, mutation, fuzz, integration and N2N/E2E layers |
| [Repository structure](repository-structure.md) | Section 7: proposed workspace boundaries |
| [CI and quality gates](ci-quality-gates.md) | Section 8: required results and proposed pipeline |
| [Roadmap](roadmap.md) | Section 9 and Appendix B: phases, first slice and suggested milestones |
| [Risks](risks.md) | Section 10: constraints and mitigations |
| [Acceptance criteria](acceptance.md) | Section 11: definition of done |
| [Source baseline](source-baseline.md) | Section 12: miniML-Parser reference and compatibility caveats |
| [Initial ADRs](adr/baseline.md) | Appendix A: ten decisions accepted in the source baseline |
| [ADR template](adr/template.md) | Supporting template for subsequent decisions |

## Decisions still open

The following items need explicit decisions during implementation; this list does not amend the source requirements.

- License and declared minimum supported Rust version (MSRV).
- Final crate names, exported API identifiers and C ABI naming/versioning policy.
- CDATA default, processing-instruction strictness and optional-feature combinations.
- Exact primitive lexical rules, float handling, whitespace policy and position behavior across CRLF.
- v1 enumeration support; whether any simple deterministic `choice` is included; namespace-name policy before namespace resolution.
- Concrete resource defaults, bounded occurrence range and fixed-capacity target representation.
- Generator/runtime manifest format and version compatibility rules.
- Coverage inclusion rules and reliable branch/region tooling. Source requires 100% production line/function coverage from unit tests alone.

“Phase 2” in the feature-profile tables means deferred capability; the numbered implementation roadmap has its own phases. No feature is marked implemented.
