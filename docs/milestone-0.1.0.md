# Milestone 0.1.0 execution ledger

Status: in progress. Coordinator branch: `feature/milestone-0.1.0`.

## Scope and completion

The first public milestone in `roadmap.md` is 0.1.0: tokenizer and validating parser for minimal static schema descriptors, a safe `no_std` core, and the first typed vertical slice (root, required integer child, optional string attribute). Static descriptors are authored directly until the 0.3.0 XSD generator milestone. The complete 1.0.0 acceptance checklist remains the release roadmap, not a claim about 0.1.0.

Completion requires workspace formatting, lint and documentation checks; stable/MSRV and embedded builds; independent 100% production line/function unit coverage; separate integration and consumer checks; deterministic resource rejection and malformed-input regressions; a documented supported profile; version 0.1.0 manifests, changelog and Git version reference. No completion claim precedes evidence.

## Delegation rules

- At most ten active milestone tasks; normally fewer due to dependencies.
- Search existing Jules sessions, branches and PRs before dispatch. One session per task ID; resume or retrieve that session before considering a replacement.
- Record session IDs and exact base commits here. Each task owns distinct paths. Integrate reviewed changes serially.
- Work is committed on the coordinator branch. Jules must not merge or modify the default branch.
- Do not publish packages to registries as part of this milestone.

## Tasks

| ID | Deliverable / owned paths | Dependencies | State | Jules session |
| --- | --- | --- | --- | --- |
| M01 | Workspace, core/facade skeleton, error/config/budget contracts, implementation ADR | None | Integrated and verified | `6412285179774184537` |
| M02 | UTF-8 cursor, positions and XML names | Independent module; integration after M01 | Integrated in `2a8d515`; verified with descriptor-name adapter in `e44ddcc` | `4325102533201049110` |
| M03 | Entity/reference decoding and fixed-capacity text representation | Independent module; integration after M01 | In progress; existing session remains active | `14253481875008285439` |
| M04 | Static schema descriptors, minimal integer conversion and sink contracts | Independent module; integration after M01 | Integrated in `339b9fe`; XML name validation in `e44ddcc` | `9272682563869501346` |
| M05 | Tokenizer: tags, attributes, text, comments, explicit unsupported features | M02, M03 | Planned | — |
| M06 | Bounded syntax/schema state machine and facade integration | M04, M05 | Planned | — |
| M07 | Separate integration fixtures and executable typed consumer example | M06 | Planned | — |
| M08 | CI, independent coverage, MSRV/embedded and adversarial checks | M01; final execution after M06 | CI/scripts integrated; full parser gates pending | `8372780934571335340` |
| M09 | Final supported profile, README, changelog, coverage/regression closure and version reference | M07, M08 | Planned | — |

## Open decision

The owner selected MIT. Keep manifests unpublished during implementation. M01 records MSRV 1.85 and initial API/profile details as explicit implementation decisions.

## Evidence

- Owner requires console-only Jules interaction and parallel dispatch wherever work is independent. M02–M04 can develop isolated modules using temporary rustc test harnesses while M01 supplies Cargo/common contracts; coordinator integrates adapters afterward. No bootstrap duplication or overlapping module ownership.

- Initial repository: documentation only; PR #1 already merged; no implementation issues/PRs.
- Initial coordinator base: `0a79dce66592119b83d8ea1da089b2e9a686b60a` (`origin/master`).
- Jules CLI v0.1.42 installed and authenticated; repository connected.
- Local Rust 1.97.1 available. Coverage tooling and embedded targets are not installed initially.
- M01: retrieved final patch through CLI; excluded the stale ledger copied into Jules's diff. Coordinator corrected ADR date, enforced fixed depth/attribute/text caps and added direct overflow tests. 17 unit tests pass; independent cargo-llvm-cov reports 100% lines/functions/regions. fmt, clippy and warnings-denied docs pass. Rust 1.85 and ARM/RISC-V no_std builds pass.
- M01 mutation baseline: `cargo mutants --file crates/core/src/budget.rs --file crates/core/src/config.rs --jobs 4 --timeout 30 -- --lib` tested 58 mutations: 57 caught, one equivalent survivor (`ParserConfig::builder` already returns `ParserConfigBuilder::default()`, so replacing it with `Default::default()` is identical). No unexplained survivor.
- M08: CLI result reviewed; coordinator corrected default branch to master, made builds locked, explicitly selects stable and installs LLVM coverage tooling. Coverage report stays in GitHub artifacts. Local script passes baseline format/lint/unit/docs/coverage/MSRV/embedded checks; full parser/integration/adversarial evidence remains pending M06/M07. Coverage export directory fix is committed in `69bf746`.
- M04: integrated from the reviewed CLI patch in `339b9fe`; schema graph validation is iterative and bounded, conversion handles signed i64 boundaries, and sink text is explicitly borrowed or decoded. Schema node and attribute names are validated through the shared M02 name rules. Local core tests, clippy, docs and independent unit coverage pass. Jules opened a separate PR for the same task; coordinator branch is the integration source of truth.
- M02 and current common contracts: `CHECK_MSRV=1 CHECK_CROSS=1 CHECK_COVERAGE=1 sh scripts/check.sh` passes after integration: 64 core unit tests, 100% line/function coverage across production modules, fmt, Clippy, docs, `no_std`, Rust 1.85 and ARM/RISC-V. Integration/consumer and parser-specific gates await M05–M07.
- PR #2 merged as `44cc374`, integrating M01 (`aaae8e3`), M02 (`2a8d515`), M04 (`339b9fe`) and M08 (`b544fd6`), plus coordinator fixes. PRs #3, #4 and #5 were closed; their original session output is already represented by those integration commits.
- PR #6 retains the Jules M02 commit `324b1fb` in its history and merges the integrated master fixes: shared positions, atomic cursor advancement, checked line/column arithmetic, CRLF state, and schema-compatible name validation. Its expanded API contract is corrected to match that implementation. `AGENTS.md` records the owner's requirement to avoid duplicate sessions and consume completed Jules output through reviewed integration.
