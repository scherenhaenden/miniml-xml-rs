import unittest

from check_mutation_results import EXPECTED_LIVENESS_MUTANTS, audit


def mutant_outcome(signature, summary):
    phases = []
    if summary == "Timeout":
        phases = [
            {"phase": "Build", "process_status": "Success"},
            {"phase": "Test", "process_status": "Timeout"},
        ]
    return {
        "scenario": {"Mutant": {"name": signature}},
        "summary": summary,
        "phase_results": phases,
    }


def passing_report(timeout_signatures=EXPECTED_LIVENESS_MUTANTS):
    outcomes = [{"scenario": "Baseline", "summary": "Success", "phase_results": []}]
    outcomes.extend(
        mutant_outcome(
            signature,
            "Timeout" if signature in timeout_signatures else "CaughtMutant",
        )
        for signature in EXPECTED_LIVENESS_MUTANTS
    )
    outcomes.extend(
        mutant_outcome(
            f"crates/core/src/check_{index}.rs:1:1: replace == with != in check_{index}",
            "CaughtMutant",
        )
        for index in range(212)
    )
    outcomes.extend(
        mutant_outcome(f"replace value_{index} with other_{index}", "Unviable")
        for index in range(8)
    )
    return {
        "outcomes": outcomes,
        "total_mutants": 223,
        "caught": 215 - len(timeout_signatures),
        "missed": 0,
        "timeout": len(timeout_signatures),
        "unviable": 8,
        "success": 0,
    }


class MutationGateTests(unittest.TestCase):
    def test_accepts_only_reviewed_test_phase_liveness_timeouts(self):
        self.assertEqual(audit(passing_report(), 3), [])

    def test_accepts_reviewed_liveness_mutants_caught_without_timeout(self):
        self.assertEqual(audit(passing_report(timeout_signatures=set()), 0), [])

    def test_rejects_an_unreviewed_timeout(self):
        report = passing_report()
        report["outcomes"][1] = mutant_outcome(
            "crates/core/src/unrelated.rs:1:1: replace safe with unsafe in unrelated_function",
            "Timeout",
        )
        errors = audit(report, 3)
        self.assertTrue(any("unexpected mutant timeout" in error for error in errors))

    def test_rejects_missed_or_surviving_mutants(self):
        report = passing_report()
        report["outcomes"][1] = mutant_outcome(
            next(iter(EXPECTED_LIVENESS_MUTANTS)), "MissedMutant"
        )
        report["missed"] = 1
        report["timeout"] = 2
        errors = audit(report, 3)
        self.assertTrue(any("missed mutants" in error for error in errors))

    def test_rejects_a_successful_surviving_mutant(self):
        report = passing_report()
        report["outcomes"][1] = mutant_outcome(
            next(iter(EXPECTED_LIVENESS_MUTANTS)), "Success"
        )
        report["success"] = 1
        report["timeout"] = 2
        errors = audit(report, 3)
        self.assertTrue(any("successful (surviving) mutants" in error for error in errors))

    def test_rejects_inconsistent_cargo_mutants_summary(self):
        report = passing_report()
        report["caught"] -= 1
        self.assertTrue(any("disagrees" in error for error in audit(report, 3)))

    def test_rejects_timeout_outside_test_phase(self):
        report = passing_report()
        report["outcomes"][1]["phase_results"][1]["process_status"] = "Failure"
        self.assertTrue(any("not in the test phase" in error for error in audit(report, 3)))

    def test_rejects_a_timeout_phase_hidden_by_a_non_timeout_summary(self):
        report = passing_report()
        report["outcomes"][4]["phase_results"] = [
            {"phase": "Build", "process_status": "Success"},
            {"phase": "Test", "process_status": "Timeout"},
        ]
        errors = audit(report, 3)
        self.assertTrue(any("phase timed out but mutant outcome" in error for error in errors))

    def test_rejects_duplicate_names_for_any_mutant(self):
        report = passing_report()
        first_ordinary_mutant = report["outcomes"][4]
        second_ordinary_mutant = report["outcomes"][5]
        second_ordinary_mutant["scenario"]["Mutant"]["name"] = (
            first_ordinary_mutant["scenario"]["Mutant"]["name"]
        )
        errors = audit(report, 3)
        self.assertTrue(any("duplicate mutant outcome name" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
