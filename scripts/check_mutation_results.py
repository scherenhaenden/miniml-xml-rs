#!/usr/bin/env python3
"""Validate the cargo-mutants outcome report for the 0.1.0 mutation gate."""

from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any


EXPECTED_LIVENESS_MUTANTS = {
    "crates/core/src/schema.rs:174:40: replace += with *= in Schema<'a>::check_graph",
    "crates/core/src/schema.rs:174:33: replace - with + in Schema<'a>::check_graph",
    "crates/core/src/schema.rs:174:33: replace - with / in Schema<'a>::check_graph",
}
SUMMARY_TO_FIELD = {
    "CaughtMutant": "caught",
    "MissedMutant": "missed",
    "Timeout": "timeout",
    "Unviable": "unviable",
    "Success": "success",
}
MINIMUM_MUTANTS = 223
MAXIMUM_UNVIABLE = 8


def audit(payload: dict[str, Any], process_status: int) -> list[str]:
    errors: list[str] = []
    outcomes = payload.get("outcomes")
    if not isinstance(outcomes, list):
        return ["outcomes.json has no outcomes list"]

    counts: Counter[str] = Counter()
    baseline_count = 0
    mutant_names: dict[str, list[str]] = {}
    timeout_signatures: list[str] = []
    for outcome in outcomes:
        scenario = outcome.get("scenario")
        summary = outcome.get("summary")
        if scenario == "Baseline":
            baseline_count += 1
            if summary != "Success":
                errors.append("cargo-mutants baseline did not pass")
            continue
        if not isinstance(scenario, dict) or not isinstance(scenario.get("Mutant"), dict):
            errors.append(f"unrecognized cargo-mutants scenario: {scenario!r}")
            continue

        field = SUMMARY_TO_FIELD.get(summary)
        if field is None:
            errors.append(f"unrecognized mutant outcome: {summary!r}")
            continue
        counts[field] += 1
        name = scenario["Mutant"].get("name")
        if not isinstance(name, str):
            errors.append("mutant outcome has no name")
            continue
        mutant_names.setdefault(name, []).append(summary)
        if summary == "Timeout":
            timeout_signatures.append(name)
            phase_statuses = {
                phase.get("phase"): phase.get("process_status")
                for phase in outcome.get("phase_results", [])
            }
            if phase_statuses.get("Build") != "Success":
                errors.append(f"timed-out mutant did not build successfully: {name}")
            if phase_statuses.get("Test") != "Timeout":
                errors.append(f"timeout was not in the test phase: {name}")

    if baseline_count != 1:
        errors.append(f"expected one successful baseline, found {baseline_count}")

    for field in ("caught", "missed", "timeout", "unviable", "success"):
        reported = payload.get(field)
        if reported != counts[field]:
            errors.append(
                f"outcomes.json {field} count {reported!r} disagrees with rows {counts[field]}"
            )

    total = payload.get("total_mutants")
    counted_total = sum(counts.values())
    if total != counted_total:
        errors.append(f"total_mutants {total!r} disagrees with rows {counted_total}")
    if not isinstance(total, int) or total < MINIMUM_MUTANTS:
        errors.append(f"expected at least {MINIMUM_MUTANTS} generated mutants, found {total!r}")
    if counts["unviable"] > MAXIMUM_UNVIABLE:
        errors.append(
            f"unviable mutant count grew above the reviewed baseline of {MAXIMUM_UNVIABLE}"
        )
    if counts["missed"] != 0:
        errors.append(f"found {counts['missed']} missed mutants")
    if counts["success"] != 0:
        errors.append(f"found {counts['success']} successful (surviving) mutants")
    if counts["caught"] < MINIMUM_MUTANTS - MAXIMUM_UNVIABLE - len(
        EXPECTED_LIVENESS_MUTANTS
    ):
        errors.append("caught mutant count fell below the reviewed baseline floor")

    for signature in sorted(EXPECTED_LIVENESS_MUTANTS):
        summaries = mutant_names.get(signature)
        if not summaries:
            errors.append(f"reviewed liveness mutant is missing: {signature}")
        elif any(summary not in ("CaughtMutant", "Timeout") for summary in summaries):
            errors.append(f"reviewed liveness mutant was not caught or timed out: {signature}")
        elif len(summaries) != 1:
            errors.append(f"reviewed liveness mutant appeared {len(summaries)} times: {signature}")

    unexpected_timeouts = set(timeout_signatures) - EXPECTED_LIVENESS_MUTANTS
    if unexpected_timeouts:
        errors.append("unexpected mutant timeout(s): " + "; ".join(sorted(unexpected_timeouts)))

    expected_status = 3 if counts["timeout"] else 0
    if process_status != expected_status:
        errors.append(
            f"cargo-mutants exited {process_status}; expected {expected_status} for these outcomes"
        )
    return errors


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(f"usage: {Path(argv[0]).name} OUTCOMES_JSON CARGO_MUTANTS_EXIT", file=sys.stderr)
        return 2
    outcomes_path = Path(argv[1])
    try:
        payload = json.loads(outcomes_path.read_text(encoding="utf-8"))
        process_status = int(argv[2])
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"mutation evidence is unreadable: {error}", file=sys.stderr)
        return 2

    errors = audit(payload, process_status)
    if errors:
        for error in errors:
            print(f"mutation gate failed: {error}", file=sys.stderr)
        return 1

    print(
        "Mutation gate passed: "
        f"{payload['total_mutants']} mutants; {payload['caught']} caught; "
        f"{payload['unviable']} unviable; {payload['timeout']} reviewed liveness timeout(s); "
        f"{payload['missed']} missed; {payload['success']} surviving."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
