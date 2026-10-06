# CI/CD and Quality Gates

Source: [architecture baseline v0.1, 5 October 2026](reference/embedded-xml-schema_project_specification.pdf), PDF pages 20.

Status: documentation baseline. This document records the original proposal or broader v1 plan, while the implemented 0.1.0 scope and evidence are tracked in [milestone-0.1.0.md](milestone-0.1.0.md). The known Rust crate names are `miniml-xml-core` and `miniml-xml`; only ABI names in examples remain proposals.

| Gate | Required result |
| --- | --- |
| Formatting | cargo fmt --check passes. |
| Lint | cargo clippy with warnings denied for workspace-supported targets. |
| Unit tests | All pass on stable; unit-only coverage = 100% production lines/functions. |
| Mutation | Critical modules have no unexplained surviving mutants. |
| Integration | All crate-boundary and feature-combination tests pass. |
| N2N/E2E | Rust and C consumer pipelines pass. |
| no_std build | Core/facade supported no_std configurations compile. |
| Cross compilation | At least representative embedded targets compile, e.g. thumbv7em- none-eabihf and riscv32imac-unknown-none-elf where feasible. |
| MSRV | Workspace compiles on declared MSRV. |
| Miri | Host-testable safe APIs pass Miri where supported. |
| Fuzz smoke | Short CI fuzz run has zero crashes; longer fuzzing may run scheduled. |
| Dependency audit | cargo deny/audit policy passes; runtime dependency count is reviewed. |
| Docs | cargo doc with warnings denied; examples compile. |
| Generated determinism | Regeneration produces no unexpected diff. |

Recommended pipeline

```text
pull_request:
  fmt -> clippy -> unit-100%-coverage -> mutation-smoke
      -> integration -> e2e-rust -> e2e-c
      -> no_std-cross-build -> msrv -> docs
nightly/scheduled:
  extended-fuzz -> full-mutation -> benchmark-regression
```
