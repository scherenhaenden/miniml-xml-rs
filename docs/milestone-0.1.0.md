# Milestone 0.1.0 execution ledger

Status: in progress.

## Scope and completion

The first public milestone in `docs/roadmap.md` is 0.1.0: tokenizer and validating parser for minimal static schema descriptors, a safe `no_std` core, and the first typed vertical slice (root, required integer child, optional string attribute). Static descriptors are authored directly until the 0.3.0 XSD generator milestone. The complete 1.0.0 acceptance checklist remains the release roadmap, not a claim about 0.1.0.

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
| M06-B | Schema-validating parser and core/facade API | M06-A | Integrated by PR #13 at `1c80851`; full local and PR checks pass | `7783618785134669031` (original session failed; coordinator recovered its work without a duplicate) |
| M07-A | XML fixtures for the typed consumer slice | Independent of parser implementation; no runtime/test overlap | Integrated via PR #10 at `b1748ef`; 11 fixtures reviewed and syntax-validated | `17172800555643325890` |
| M07-B | Integration tests, embedded compile sample and typed consumer | M06-B, M07-A | Integrated by PR #15 at `81979781177f5334a8c6af113ea245bd16d2c277`; source session base `1fbf9e31783a123ff41a3db16905f0bcb129e526` | `4078684291427294237` |
| M08 | CI, independent coverage, MSRV/embedded and adversarial gates | M01; final execution after M06 | Integrated via PR #17 at `499ca0b1ee91d9c5dc273078f67283b4df43c9e8`; mutation gate and CI pass; final post-M09 run remains | `8372780934571335340` |
| M08-A | Correct quality-gate status docs and run bounded mutation baseline on existing modules | M08; independent of M06-B, with no `validate.rs` assumption | Integrated by PR #12 at `1fbf9e3`; current mutation evidence records three timeouts and zero missed mutants | `1752884590832168457` (base `1a10f1820e3bb494a44598d582e1ed055b8a341f`) |
| M08-B | Add bounded malformed/adversarial smoke coverage | M06-B, M08-A | Integrated by PR #16 at `b119d218e200c405a4cc50415a6f18de3e273dcd`; source session base `1fbf9e31783a123ff41a3db16905f0bcb129e526` | `1203790517305183261` |
| M08-C | Validate mutation outcomes and classify reviewed liveness timeouts | M06-B, M08-A | Integrated via PR #17, head `38ed759919446f72b37c1f3f5112effb27e7448a`, based on `b119d218e200c405a4cc50415a6f18de3e273dcd`, merge `499ca0b1ee91d9c5dc273078f67283b4df43c9e8`; reviewed findings fixed and full gate passes | — |
| M09 | Final supported profile, README, changelog, coverage/regression closure and version reference | M07, M08 | M09-C integrated; M09-A/M09-B PRs and the final full gate/tag remain | — |
| M09-A | README quick start, changelog and supported profile | M07, M08 | Jules completed from exact base `b119d218e200c405a4cc50415a6f18de3e273dcd`; coordinator corrected review findings and compiled the exact quick-start; PR #19 is open from updated master `30690b292dc47db02088df897415de6cb76a9677` | `11032068863378026006` |
| M09-B | Clarify 1.0 acceptance and requirements status against 0.1.0 scope | M07, M08 | Jules completed from exact base `b119d218e200c405a4cc50415a6f18de3e273dcd`; diff retrieved and coordinator corrections prepared; PR pending; owns only `docs/acceptance.md` and `docs/requirements.md` | `513019483045454900` |
| M09-C | Require UTF-8-compatible XML encoding declarations | M05 | Coordinator fix integrated via PR #18, head `f31c7b3dfea1e7df1ee9e4e79370047ca3312c4b`, based on `499ca0b1ee91d9c5dc273078f67283b4df43c9e8`, merge `30690b292dc47db02088df897415de6cb76a9677`; CI and full local quality gates pass | — |

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

- GitHub CLI confirms PRs #1–#11 are merged into `master`. PR #7 integrated M03/M05 at `fbfd0cd`; PR #8 integrated the reviewed M06-A bounded stack at `d5c197f`; PR #9 refreshed this ledger at `a4fb75c`; PR #10 integrated the M07-A fixtures at `b1748ef`; PR #11 records their integration at `1a10f18`. The no-version-bump choice was recorded for the unpublished intermediate milestone; the external bot check did not acknowledge the selection before merge.
- The 0.1.0 milestone remains in progress; do not create its version tag until M07–M09 acceptance and release evidence are complete.
- `CHECK_MSRV=1 CHECK_CROSS=1 CHECK_COVERAGE=1 sh scripts/check.sh` passes after M06-A: 118 core unit tests, formatting, warnings-denied Clippy/docs, workspace tests, `no_std`, Rust 1.85, ARM/RISC-V, and strict LLVM coverage at 100% production lines and functions. Region coverage is 98.95%; it is not an M08 gate.
- Jules M06-A session `6709002947009389318` completed from exact base `a3b52eddac5ce35b48b1dd263df9891149022520`; its reviewed result is merged in PR #8. The coordinator fixed the zero-depth self-closing boundary and added direct regression tests before opening that PR.
- M07's acceptance review confirms that parser signature, facade re-exports, and sink completion semantics must be fixed by M06 before integration tests are implemented. M07-A fixture files are independent and can proceed in parallel; dispatch M07-B only after M06-B establishes those contracts.
- Jules M06-B session `7783618785134669031` is assigned the validating parser from exact base `d5c197f74ca2cef00a94a19abce31b313125b57f`. It owns the element-frame extension, core parser, core/facade re-exports, direct tests, and M06 contract update. It must open one reviewed PR and must not merge it.
- Jules M07-A session `17172800555643325890` completed the fixture-only task from base `fbfd0cd042f3df313b4deaab46a95a3c2f242658`. Commit `52469cb` added 11 fixtures, merged in PR #10; 8 parse as well-formed XML and 3 are intentionally malformed. M07-B integration tests and the typed consumer wait for M06-B's actual facade and callback contracts.
- Jules M08-A session `1752884590832168457` is in progress from exact base `1a10f1820e3bb494a44598d582e1ed055b8a341f`. It owns only `scripts/check.sh` and `docs/m08-quality-gates.md`; it must report equivalent mutation survivors and defer parser validation mutations until M06-B exists. This is one follow-up to the already integrated M08 base, not a duplicate M08 session.
- The M08-A audit confirms the base CI currently covers formatting, warnings-denied Clippy/docs, unit and doctest runs, no_std, MSRV 1.85, ARM/RISC-V and separate 100% production line/function coverage. Parser integration/consumer tests, malformed/adversarial smoke and mutation evidence for validation remain unproven until their dependent work is integrated.
- PR #2 merged as `44cc374`, integrating M01 (`aaae8e3`), M02 (`2a8d515`), M04 (`339b9fe`) and M08 (`b544fd6`), plus coordinator fixes. PR #6 retains Jules M02 commit `324b1fb` and includes the integrated cursor/name fixes and corrected API contract. `AGENTS.md` records the owner's console-only GitHub/Jules requirement, duplicate-session checks, and obligation to consume reviewed Jules work.

## Coordinator status before the M06-B and M08-A merges (2026-10-06)

- PRs #12 and #13 are open. PR #13 contains M06-B commit `293b7468aa05e5ec591c9a6d82d56c5f153cb43f` on exact base `d5c197f74ca2cef00a94a19abce31b313125b57f`; GitHub checks are still running. Its local full gate passed with 135 core tests, 100% line/function coverage, MSRV 1.85, `no_std`, and both embedded targets.
- The original Jules M06-B session `7783618785134669031` failed. The coordinator recovered and corrected the work on that session's exact base; no replacement M06-B session was created. A read-only independent review found no blocking issue.
- Mutation run on the committed M06-B source: 223 mutants tested, 212 caught, 8 unviable, 3 timed out in `Schema::check_graph`, zero missed. Two documented equivalent replacements are excluded. The three timeouts are reported as such; this command result is not described as a clean mutation pass. New parser tests catch stale text across sibling elements and preserve the exact decoded-text boundary.
- Jules M08-A session `1752884590832168457` is completed. Its retrieved diff contained a stale mutation description and a `scripts/check.sh` regression that omitted `validate.rs`; the coordinator retained parser validation in the mutation scope and updated its documented evidence. PR #12 records the reconciled ledger, script and quality-gate document.
- PR #10/#11 already integrate the M07-A fixtures. After M06 PR #13 merges, M07-B consumer tests and M08-B adversarial smoke can run concurrently on disjoint paths. They are not dispatched yet, avoiding a stale API base.
- M09 and the `v0.1.0` tag remain pending until M07/M08 checks and release documentation are complete.

## Current coordinator status after PR #12 and PR #13 (2026-10-06)

- `master` is at `1fbf9e31783a123ff41a3db16905f0bcb129e526`. PR #13 integrated M06-B as merge commit `1c80851ee92920c1752d9a846f7672d88b1a9753`; PR #12 integrated the reconciled M08-A ledger, mutation evidence and scripts as merge commit `1fbf9e31783a123ff41a3db16905f0bcb129e526`. Both PRs were approved and all reported checks passed before merge.
- M06-B is implemented and integrated. Its 135 core tests pass; CI verifies formatting, lint, unit and integration/doc tests, docs, `no_std`, MSRV 1.85, coverage and ARM/RISC-V checks. The mutation run is recorded separately under M08: 223 tested, 212 caught, 8 unviable, 3 timed out in `Schema::check_graph`, zero missed; the command did not finish with a clean mutation status.
- M07-A fixtures are integrated by PR #10 and recorded by PR #11. M07-B session `4078684291427294237` was created against this exact `master` commit. It owns `crates/facade/tests/m07_consumer.rs` and two M07 facade examples, reads `tests/fixtures/m07`, and must produce a focused PR without touching runtime, manifests, CI/scripts or this ledger.
- M08-A session `1752884590832168457` is integrated by PR #12. M08-B session `1203790517305183261` was created against the same exact `master` commit. It owns `crates/facade/tests/m08_adversarial.rs` and `tests/fixtures/m08/`; it must produce a focused PR without touching M07 files, runtime, manifests, CI/scripts, quality-gate docs or this ledger.
- The M07-B and M08-B sessions were checked against the Jules registry, existing branches and PRs before dispatch. They have distinct task IDs and file ownership and run in parallel; there are two newly active milestone tasks, below the ten-task cap. No open PRs remained when they were created.
- The coordinator is tracking their results through Jules and will review, correct, test and integrate each PR before updating this ledger with its integration commit. M09 and the `v0.1.0` tag remain pending until M07/M08 acceptance and release evidence are complete.

## Current coordinator status after PR #15 and PR #16 (2026-10-06)

- `master` is at `b119d218e200c405a4cc50415a6f18de3e273dcd`. PR #15 integrated M07-B as merge commit `81979781177f5334a8c6af113ea245bd16d2c277`; PR #16 integrated M08-B as merge commit `b119d218e200c405a4cc50415a6f18de3e273dcd`.
- Jules M07-B session `4078684291427294237` completed from exact base `1fbf9e31783a123ff41a3db16905f0bcb129e526`. Its reviewed output is integrated in PR #15. Coordinator review added public-facade boundary cases for depth, attributes, children, text and occurrences, replaced an unbounded token-search loop with an exact assertion, and made the no_std example compile cleanly under host all-target Clippy as well as both embedded targets.
- M07-B has 13 passing facade tests. The examples compile for the host, `thumbv7em-none-eabihf`, and `riscv32imac-unknown-none-elf`. PR #15's formatting, Clippy, unit, integration/doc, docs, no_std, MSRV, cross-target and coverage checks passed.
- Jules M08-B session `1203790517305183261` completed from exact base `1fbf9e31783a123ff41a3db16905f0bcb129e526`. Its reviewed output is integrated in PR #16. Coordinator review corrected the schema-shaped nesting input so the depth test reaches the actual depth limit and added a separate trailing-content case. The resulting adversarial suite has 11 passing tests.
- PR #16's formatting, Clippy, unit, integration/doc, docs, no_std, MSRV, cross-target and coverage checks passed. Both Jules results were retrieved and consumed; no duplicate or replacement Jules session was created.
- M08-C is coordinator-owned to preserve Jules capacity. On a branch based on `65da9341c55507a318fcba2f5a679407ec847dbd`, `CHECK_MUTATION=1 sh scripts/check.sh` passed with cargo-mutants 27.1.0: 223 mutants tested, 212 caught, 8 unviable, 3 reviewed test-phase timeouts caused by non-progressing `Schema::check_graph` mutations, zero missed and zero surviving. The result validator rejects new timeouts, survivors, missing expected mutations, failed baselines, and inconsistent outcome counts. The final combined gate still must run on the complete merged tree after M09 edits.
- Independent review of PR #17 found that duplicate ordinary mutant rows could conceal an omitted survivor, and a test-phase timeout could be hidden by a non-`Timeout` summary. The coordinator added checks for duplicate mutant names and timeout-phase/summary consistency, plus regression tests for both findings. The updated combined local gate passes; refreshed PR CI and the final post-M09 gate remain pending.
- After PRs #15 and #16 merged, `CHECK_MUTATION=1 CHECK_MSRV=1 CHECK_CROSS=1 CHECK_COVERAGE=1 sh scripts/check.sh` passed on the current `master` base `b119d218e200c405a4cc50415a6f18de3e273dcd` plus M08-C. It covered the integrated consumer/adversarial suites, 135 core unit tests, 100% production line/function coverage, Rust 1.85, both embedded targets, and the 223-mutant gate above.
- Jules M09-A session `11032068863378026006` was checked against existing sessions, branches and PRs before dispatch; none duplicated it. It has exact base `b119d218e200c405a4cc50415a6f18de3e273dcd` and owns only README, changelog and supported-profile docs. Its PR must be reviewed and merged before the final all-gates run and tag.
- Jules M09-B session `513019483045454900` was dispatched only after checking the Jules registry, branches and PRs for duplicate work. It uses exact base `b119d218e200c405a4cc50415a6f18de3e273dcd` and owns only `docs/acceptance.md` and `docs/requirements.md`, separate from M09-A. Jules completed; the retrieved diff keeps the full 1.0 requirements and frames them as future scope.
- M09-B's reviewed diff left the requirement tables intact. The coordinator corrected stale crate-name wording and clarified that the v1 requirements describe the intended 1.0 behavior while the roadmap phases delivery.
- Independent review of M09-A found three accuracy issues: arbitrary encoding declarations were accepted despite UTF-8-only input, `MaxDepthExceeded` was not a public error code, and fixed-storage caps were mislabeled as hardware limits. PR #18 corrected parser behavior; the supported profile now documents UTF-8 declaration enforcement, `SchemaError`, actual `ErrorCode` names, and fixed-storage caps.
- PR #17 merged as `499ca0b1ee91d9c5dc273078f67283b4df43c9e8`. After fixing the duplicate-name and phase-summary audit holes, the full local gate passed: formatting, Clippy, 135 core tests, integration/docs, coverage, MSRV 1.85, ARM/RISC-V, and 223 mutants (212 caught, 8 unviable, 3 reviewed non-progressing test timeouts, 0 missed, 0 survivors). PR CI is green.
- PR #18 merged as `30690b292dc47db02088df897415de6cb76a9677`. The tokenizer now accepts only UTF-8 encoding declarations (case-insensitively); `UTF-16` and `US-ASCII` declarations are rejected. Formatting, Clippy, 135 core tests, integration/docs, coverage, MSRV 1.85, ARM/RISC-V, and PR CI pass.
- Do not create `v0.1.0` until final-tree verification passes; package publication remains disabled.
