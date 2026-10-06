# Milestone 0.1.0 quality gates

This file records the executable gates and the evidence actually collected for
the 0.1.0 workspace. M07 consumer and M08 adversarial tests are integrated;
the final combined gate run and M09 release evidence remain outstanding.

## Executable checks

`scripts/check.sh` always runs formatting, warnings-denied Clippy, library unit
tests, separate integration and doctest commands, warnings-denied documentation,
and a workspace `no_std` build. Optional environment flags enable coverage,
MSRV, representative embedded targets, and mutation analysis.

The full local command is:

```sh
CHECK_MSRV=1 CHECK_CROSS=1 CHECK_COVERAGE=1 sh scripts/check.sh
```

On coordinator commit `293b7468aa05e5ec591c9a6d82d56c5f153cb43f` (M06-B, PR
#13), this passed with 135 library tests, Rust 1.85, `thumbv7em-none-eabihf`,
`riscv32imac-unknown-none-elf`, and 100% production line and function coverage.
Region coverage was 99.08%; it is not a release gate. M07-B and M08-B were
subsequently integrated in PRs #15 and #16, and each PR's coverage, formatting,
Clippy, workspace tests, docs, `no_std`, MSRV, and cross-target checks passed.
Run the combined command again on the final tree after M09 changes.

## Mutation evidence

Run the bounded mutation pass with:

```sh
CHECK_MUTATION=1 sh scripts/check.sh
```

On a local checkout based on master commit `65da9341c55507a318fcba2f5a679407ec847dbd`,
the full `CHECK_MUTATION=1 sh scripts/check.sh` run passed with cargo-mutants
27.1.0. It tested 223 generated mutants: 212 were caught, 8 were unviable, 3
timed out, and none were missed or survived. The production code in this run
includes M06-B (`293b7468aa05e5ec591c9a6d82d56c5f153cb43f`); PRs #15 and #16
add facade tests and fixtures without changing core mutation targets. The
three timeouts are mutations to the progress counters in the bounded
`Schema::check_graph` walk:

* `crates/core/src/schema.rs:174:40: replace += with *= in Schema::check_graph`
* `crates/core/src/schema.rs:174:33: replace - with + in Schema::check_graph`
* `crates/core/src/schema.rs:174:33: replace - with / in Schema::check_graph`

Each mutant built successfully, then hit the configured 30-second test timeout
while walking the schema graph. They are retained as liveness failures caught
by the test watchdog; they are not excluded or treated as successful mutants.
`scripts/check.sh` validates the full `cargo-mutants` JSON report and accepts
only these reviewed test-phase timeouts, a successful baseline, zero missed or
surviving mutants, and consistent aggregate counts. Any new timeout, failed
baseline, missing expected mutation, or changed summary fails the gate. The raw
report is retained under the printed temporary directory for inspection.

Run the mutation gate with:

```sh
CHECK_MUTATION=1 sh scripts/check.sh
```

The validator's seven acceptance/rejection tests run before every mutation
pass using the Python 3 standard library; it adds no project runtime
dependency.

After PRs #15 and #16 merged, the full combined command also passed on master
`b119d218e200c405a4cc50415a6f18de3e273dcd` plus the M08-C changes:

```sh
CHECK_MUTATION=1 CHECK_MSRV=1 CHECK_CROSS=1 CHECK_COVERAGE=1 sh scripts/check.sh
```

That run included the M07 consumer and M08 adversarial suites, 135 core unit
tests, 100% production line/function coverage, Rust 1.85, and both embedded
targets. Region coverage was 99.08%; it is not a release gate. Run the same
command again after the M09 documentation PR is merged to record final-tree
evidence.

The script excludes two behaviorally equivalent replacements, each with a
specific reason:

* `ParserConfig::builder` returns `ParserConfigBuilder::default()` directly.
* `Schema::new` accepts only `SCHEMA_VERSION`, which is 1 in this release, so
  replacing `Schema::version()` with `1` returns the same value for every valid
  schema in the current contract. The existing unit test compares the accessor
  with `SCHEMA_VERSION`, so changing that constant will expose a stale getter.

The mutation run added two useful regressions before this baseline was
recorded: text state resets between sibling string elements, and an empty
append after exactly reaching a text limit remains accepted. These cases cover
the parser's fixed-buffer accumulation behavior without allocating.

## Remaining milestone evidence

* PR #15 merged M07-B at `81979781177f5334a8c6af113ea245bd16d2c277`.
  Its 13 public-facade tests cover typed output, malformed and schema failures,
  and exact/one-under boundaries for document bytes, tokens, depth, attributes,
  children, text and occurrences. Both embedded targets pass.
* PR #16 merged M08-B at `b119d218e200c405a4cc50415a6f18de3e273dcd` with 11
  deterministic adversarial integration tests. Its complete GitHub check set,
  including 100% production line/function coverage, passed.
* Complete M09's supported-profile documentation, README and changelog, then
  re-run the full combined command on the final merged tree before recording
  release readiness. Create the `v0.1.0` Git tag only after that run. Do not
  publish a registry package as part of 0.1.0.

The future XSD generator and C ABI are outside this milestone, as recorded in
the roadmap; neither is silently treated as a 0.1.0 gate.
