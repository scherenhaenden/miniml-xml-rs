# Milestone 0.1.0 execution ledger

Status: in progress. Coordinator branch: `feature/m06-progress-ledger`.

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
| M03 | Entity/reference decoding and fixed-capacity text representation | Independent module; integration after M01 | Integrated via PR #7 at `fbfd0cd`; full quality gates pass | `14253481875008285439` |
| M04 | Static schema descriptors, minimal integer conversion and sink contracts | Independent module; integration after M01 | Integrated in `339b9fe`; XML name validation in `e44ddcc` | `9272682563869501346` |
| M05 | Tokenizer: tags, attributes, text, comments, explicit unsupported features | M02, M03 | Integrated via PR #7 at `fbfd0cd`; full quality gates pass | `13446562686455561403` (retrieved; do not duplicate) |
| M06-A | Bounded structural element stack | M04, M05 | Integrated via PR #8 at `d5c197f`; 118 core tests and full gates pass | `6709002947009389318` |
| M06-B | Schema-validating parser and core/facade API | M06-A | In progress on exact base `d5c197f74ca2cef00a94a19abce31b313125b57f`; PR expected after checks | `7783618785134669031` |
| M07-A | XML fixtures for the typed consumer slice | Independent of parser implementation; no runtime/test overlap | In progress; fixtures only, exact base `fbfd0cd042f3df313b4deaab46a95a3c2f242658` | `17172800555643325890` |
| M07-B | Integration tests, embedded compile sample and typed consumer | M06-B, M07-A | Waiting for the parser facade contract and callback semantics | Not dispatched |
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
- PR #2 merged as `44cc374`, integrating M01 (`aaae8e3`), M02 (`2a8d515`), M04 (`339b9fe`) and M08 (`b544fd6`), plus coordinator fixes. PRs #3, #4 and #5 were closed; their original session output is already represented by those integration commits.
- PR #6 retains the Jules M02 commit `324b1fb` in its history and merges the integrated master fixes: shared positions, atomic cursor advancement, checked line/column arithmetic, CRLF state, and schema-compatible name validation. Its expanded API contract is corrected to match that implementation. `AGENTS.md` records the owner's requirement to avoid duplicate sessions and consume completed Jules output through reviewed integration.

## Current coordinator status (2026-10-05)

- GitHub CLI confirms PRs #1–#8 are merged into `master`. PR #7 integrated M03/M05 at `fbfd0cd`; PR #8 integrated the reviewed M06-A bounded stack at `d5c197f`. The no-version-bump choice was recorded for the unpublished intermediate milestone; the external bot check did not acknowledge the selection before merge.
- The current coordinator ledger branch is `feature/m06-progress-ledger`, created from `origin/master` at `d5c197f`. The 0.1.0 milestone remains in progress; do not create its version tag until M07–M09 acceptance and release evidence are complete.
- `CHECK_MSRV=1 CHECK_CROSS=1 CHECK_COVERAGE=1 sh scripts/check.sh` passes after M06-A: 118 core unit tests, formatting, warnings-denied Clippy/docs, workspace tests, `no_std`, Rust 1.85, ARM/RISC-V, and strict LLVM coverage at 100% production lines and functions. Region coverage is 98.95%; it is not an M08 gate.
- Jules M06-A session `6709002947009389318` completed from exact base `a3b52eddac5ce35b48b1dd263df9891149022520`; its reviewed result is merged in PR #8. The coordinator fixed the zero-depth self-closing boundary and added direct regression tests before opening that PR.
- M07's acceptance review confirms that parser signature, facade re-exports, and sink completion semantics must be fixed by M06 before integration tests are implemented. M07-A fixture files are independent and can proceed in parallel; dispatch M07-B only after M06-B establishes those contracts.
- Jules M06-B session `7783618785134669031` is implementing the validating parser from exact base `d5c197f74ca2cef00a94a19abce31b313125b57f`. It owns the element-frame extension, core parser, core/facade re-exports, direct tests, and M06 contract update. It must open one reviewed PR and must not merge it.
- Jules M07-A session `17172800555643325890` is independently adding fixture XML files only under `tests/fixtures/m07/**`, from base `fbfd0cd042f3df313b4deaab46a95a3c2f242658`. Its changes do not overlap M06-B. M07-B integration tests and the typed consumer wait for M06-B's actual facade and callback contracts.
- PR #2 merged as `44cc374`, integrating M01 (`aaae8e3`), M02 (`2a8d515`), M04 (`339b9fe`) and M08 (`b544fd6`), plus coordinator fixes. PR #6 retains Jules M02 commit `324b1fb` and includes the integrated cursor/name fixes and corrected API contract. `AGENTS.md` records the owner's console-only GitHub/Jules requirement, duplicate-session checks, and obligation to consume reviewed Jules work.
