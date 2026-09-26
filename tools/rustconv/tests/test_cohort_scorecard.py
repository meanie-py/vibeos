from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from rustconv.cohort_scorecard import (
    ArtifactPins,
    CaseResult,
    Check,
    Classification,
    CohortManifest,
    Repair,
    RepairKind,
    ScorecardError,
    Split,
    artifact_pins_for_files,
    pipeline_fingerprint,
    score_run,
)


def pins(byte: bytes) -> ArtifactPins:
    import hashlib

    digest = hashlib.sha256(byte).hexdigest()
    return ArtifactPins(digest, digest, digest)


class ScorecardTests(unittest.TestCase):
    def setUp(self):
        self.manifest = CohortManifest(
            development={"dev-a": pins(b"a"), "dev-b": pins(b"b")},
            held_out={"held-c": pins(b"c")},
        )
        self.pipeline = "f" * 64

    def result(self, case_id: str, **kwargs) -> CaseResult:
        return CaseResult(case_id, checks=(Check("differential", True, True),), **kwargs)

    def grade_dev(self, cases=None):
        return score_run(
            self.manifest,
            Split.DEVELOPMENT,
            self.pipeline,
            cases or [self.result("dev-a"), self.result("dev-b", mismatches=1)],
            {key: value for key, value in self.manifest.development.items()},
        )

    def test_manifest_requires_nonempty_disjoint_suites_and_pins(self):
        with self.assertRaises(ScorecardError):
            CohortManifest({}, {"held": pins(b"h")})
        with self.assertRaises(ScorecardError):
            CohortManifest({"same": pins(b"a")}, {"same": pins(b"b")})
        with self.assertRaises(ScorecardError):
            CohortManifest({"renamed-a": pins(b"a")}, {"renamed-b": pins(b"a")})
        with self.assertRaises(ScorecardError):
            ArtifactPins("bad", "0" * 64, "1" * 64)

    def test_pipeline_fingerprint_uses_only_explicit_file_contents(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            a, b = root / "prompt", root / "grader"
            a.write_text("prompt-v1")
            b.write_text("grader-v1")
            first = pipeline_fingerprint({"prompt": a, "grader": b})
            self.assertEqual(first, pipeline_fingerprint({"grader": b, "prompt": a}))
            a.write_text("prompt-v2")
            self.assertNotEqual(first, pipeline_fingerprint({"prompt": a, "grader": b}))
            with self.assertRaises(ScorecardError):
                pipeline_fingerprint({})

    def test_file_pins_cover_source_candidate_and_grader(self):
        with tempfile.TemporaryDirectory() as temp:
            paths = [Path(temp) / name for name in ("source", "candidate", "grader")]
            for path in paths:
                path.write_text(path.name)
            import hashlib

            actual = artifact_pins_for_files(*paths)
            self.assertEqual(actual.source_sha256, hashlib.sha256(b"source").hexdigest())
            self.assertEqual(actual.candidate_sha256, hashlib.sha256(b"candidate").hexdigest())
            self.assertEqual(actual.grader_sha256, hashlib.sha256(b"grader").hexdigest())

    def test_actual_pins_and_exact_coverage_are_mandatory(self):
        with self.assertRaises(ScorecardError):
            score_run(self.manifest, Split.DEVELOPMENT, self.pipeline, [], {}, None)
        with self.assertRaises(ScorecardError):
            score_run(
                self.manifest,
                Split.DEVELOPMENT,
                self.pipeline,
                [self.result("dev-a")],
                {"dev-a": pins(b"a")},
            )
        wrong = {"dev-a": pins(b"wrong"), "dev-b": pins(b"b")}
        with self.assertRaises(ScorecardError):
            score_run(
                self.manifest,
                Split.DEVELOPMENT,
                self.pipeline,
                [self.result("dev-a"), self.result("dev-b")],
                wrong,
            )

    def test_development_report_validity_is_separate_from_translation_pass(self):
        report = self.grade_dev()
        self.assertTrue(report.valid)
        self.assertFalse(report.passed)
        self.assertEqual(report.cases[1].classification, Classification.REJECTED)

    def test_held_out_requires_valid_development_and_same_fingerprint(self):
        development = self.grade_dev()
        actual = {"held-c": pins(b"c")}
        outcome = [self.result("held-c")]
        held = score_run(self.manifest, Split.HELD_OUT, self.pipeline, outcome, actual, development)
        self.assertTrue(held.passed)
        with self.assertRaises(ScorecardError):
            score_run(self.manifest, Split.HELD_OUT, "e" * 64, outcome, actual, development)
        incomplete = score_run(
            self.manifest,
            Split.DEVELOPMENT,
            self.pipeline,
            [CaseResult("dev-a", checks=()), self.result("dev-b")],
            self.manifest.development,
        )
        self.assertFalse(incomplete.valid)
        with self.assertRaises(ScorecardError):
            score_run(self.manifest, Split.HELD_OUT, self.pipeline, outcome, actual, incomplete)

    def test_classification_comes_from_checks_mismatches_and_repair_history(self):
        self.assertEqual(self.result("x").classification, Classification.UNTOUCHED_SUCCESS)
        self.assertEqual(
            self.result(
                "x", repairs=(Repair(RepairKind.AUTOMATIC, "formatter repaired signature"),)
            ).classification,
            Classification.AUTOMATIC_REPAIR,
        )
        self.assertEqual(
            self.result(
                "x", repairs=(Repair(RepairKind.MANUAL, "human corrected ownership"),)
            ).classification,
            Classification.MANUAL_REPAIR,
        )
        self.assertEqual(
            self.result(
                "x", mismatches=1, repairs=(Repair(RepairKind.AUTOMATIC, "retry"),)
            ).classification,
            Classification.REJECTED,
        )
        self.assertEqual(
            self.result("x", unsupported_reason="variadic ABI unsupported").classification,
            Classification.UNSUPPORTED,
        )
        with self.assertRaises(ScorecardError):
            Repair(RepairKind.MANUAL, " ")

    def test_passing_requires_executed_successful_checks_and_null_unknown_cost(self):
        good = self.grade_dev([self.result("dev-a"), self.result("dev-b")])
        self.assertTrue(good.passed)
        case = good.as_dict()["cases"][0]
        self.assertIsNone(case["cost_usd"])
        self.assertIsNone(case["effort_seconds"])
        unchecked = score_run(
            self.manifest,
            Split.DEVELOPMENT,
            self.pipeline,
            [CaseResult("dev-a", checks=(Check("compile", False, False),)), self.result("dev-b")],
            self.manifest.development,
        )
        self.assertFalse(unchecked.valid)
        self.assertFalse(unchecked.passed)


if __name__ == "__main__":
    unittest.main()
