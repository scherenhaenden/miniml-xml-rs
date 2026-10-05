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
| M03 | Entity/reference decoding and fixed-capacity text representation | Independent module; integration after M01 | Integrated in `54444c0`; verified by the current 100% line/function core gate | `14253481875008285439` |
| M04 | Static schema descriptors, minimal integer conversion and sink contracts | Independent module; integration after M01 | Integrated in `339b9fe`; XML name validation in `e44ddcc` | `9272682563869501346` |
| M05 | Tokenizer: tags, attributes, text, comments, explicit unsupported features | M02, M03 | Coordinator candidate passes 100% line/function coverage and full quality gates; syncing to current master before its PR | `13446562686455561403` (retrieved; do not duplicate) |
| M06 | Bounded syntax/schema state machine and facade integration | M04, M05 | M06-A bounded structural stack dispatched from exact master base; remaining runtime work follows M05 integration | `6709002947009389318` |
| M07 | Separate integration fixtures and executable typed consumer example | M06 | Acceptance review prepared; dispatch waits for M06's actual parser/sink contract | Not dispatched |
| M08 | CI, independent coverage, MSRV/embedded and adversarial checks | M01; final execution after M06 | CI/scripts integrated; format/lint/tests/docs/no_std/MSRV/cross and current 100% line/function coverage gates pass; final parser gates pending | `8372780934571335340` |
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
- PR review: #2 is the coordinator PR. PRs #3–#5 duplicate M01/M04/M08 already integrated here and should not be merged as separate implementations. CI passes on #2 after the M08 coverage directory fix; the only remaining failing check is the external `GStraccini Checks: Pull Request`, whose reason is not exposed by GitHub CLI. PR #5's standalone workflow also targets `push: main` while the repository base is `master`.

## Current coordinator status (2026-10-05)

- GitHub CLI confirms PRs #1–#6 are merged into `master` and there are no open PRs. The current `origin/master` base is `a3b52eddac5ce35b48b1dd263df9891149022520`.
- The coordinator branch is `feature/milestone-0.1.0`, currently at `82a0095`. Its uncommitted M05 candidate owns `cursor.rs`, `lib.rs`, `text.rs`, `tokenizer.rs`, and `docs/m05-contracts.md`; no M05 PR has been opened.
- M05 has 106 passing core unit tests. `CHECK_MSRV=1 CHECK_CROSS=1 CHECK_COVERAGE=1 sh scripts/check.sh` passes formatting, warnings-denied Clippy/docs, workspace tests, `no_std`, Rust 1.85, ARM/RISC-V, and strict LLVM coverage at 100% production lines and functions. The current report has 100% line/function coverage; region coverage is 98.88% and is not an M08 gate.
- Jules M06-A session `6709002947009389318` starts from the exact `origin/master` commit above and owns only `crates/core/src/state.rs` plus a private module declaration in `crates/core/src/lib.rs`. It must return as a PR and must not merge.
- M07's acceptance review confirms that public parser signature, facade re-exports, schema-version errors, and sink completion semantics must be fixed by M06 before integration tests are implemented. Do not dispatch duplicate M07 work until those contracts are reviewable.
