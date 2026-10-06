# Milestone 0.1.0 quality gates

This file records the executable gates and the evidence actually collected for
the 0.1.0 workspace. It distinguishes completed local runs from checks still
waiting for the M07 consumer and M08 adversarial work.

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
#13), it passed with 135 library tests, Rust 1.85, `thumbv7em-none-eabihf`,
`riscv32imac-unknown-none-elf`, and 100% production line and function coverage.
Region coverage was 99.08%; it is not a release gate. The command also ran the
separate integration and doctest targets, but the M07 consumer integration
tests had not yet landed, so this run does not prove the consumer contract.

## Mutation evidence

Run the bounded mutation pass with:

```sh
CHECK_MUTATION=1 sh scripts/check.sh
```

Against the M06-B source represented by commit `293b7468aa05e5ec591c9a6d82d56c5f153cb43f`,
the library mutation run tested 223 generated mutants: 212 were caught, 8 were
unviable, 3 timed out, and none were missed. The three timeouts came from
mutations to the progress counters in the bounded `Schema::check_graph` walk.
They did not survive as passing mutants; each hit the configured 30-second
limit. The `cargo mutants` command therefore returned a nonzero result for
those timeouts, and the run is recorded as a baseline rather than claimed as a
fully passing mutation gate. Re-run it after M08-B adversarial coverage lands
and investigate any changed outcome.

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

* The local format, lint, unit, documentation, `no_std`, MSRV, cross-target and
  independent coverage checks pass on M06-B.
* M07-B still needs a public-facade consumer test, a typed example and an
  embedded compile sample.
* M08-B still needs a bounded malformed/adversarial parser smoke corpus. Keep
  it separate from the M07 consumer tests so its evidence remains independent.
* Re-run the complete command set on the final merged tree before recording
  release readiness. Do not publish a registry package as part of 0.1.0.

The future XSD generator and C ABI are outside this milestone, as recorded in
the roadmap; neither is silently treated as a 0.1.0 gate.
