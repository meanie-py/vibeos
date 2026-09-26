"""Offline cohort pins, frozen pipeline fingerprints, and translation scorecards."""

from __future__ import annotations

import hashlib
import json
import re
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from decimal import Decimal, InvalidOperation
from enum import Enum
from pathlib import Path
from types import MappingProxyType

_SHA256 = re.compile(r"^[0-9a-f]{64}$")


class ScorecardError(ValueError):
    """Invalid or incomplete benchmark provenance."""


class Split(str, Enum):
    DEVELOPMENT = "development"
    HELD_OUT = "held_out"


class RepairKind(str, Enum):
    AUTOMATIC = "automatic"
    MANUAL = "manual"


class Classification(str, Enum):
    UNTOUCHED_SUCCESS = "untouched_success"
    AUTOMATIC_REPAIR = "automatic_repair"
    MANUAL_REPAIR = "manual_repair"
    REJECTED = "rejected"
    UNSUPPORTED = "unsupported"


@dataclass(frozen=True)
class ArtifactPins:
    source_sha256: str
    candidate_sha256: str
    grader_sha256: str

    def __post_init__(self) -> None:
        for name in ("source_sha256", "candidate_sha256", "grader_sha256"):
            value = getattr(self, name)
            if not isinstance(value, str) or not _SHA256.fullmatch(value):
                raise ScorecardError(f"{name} must be a lowercase SHA-256 hex digest")

    def as_dict(self) -> dict[str, str]:
        return {
            "source_sha256": self.source_sha256,
            "candidate_sha256": self.candidate_sha256,
            "grader_sha256": self.grader_sha256,
        }


@dataclass(frozen=True)
class CohortManifest:
    development: Mapping[str, ArtifactPins]
    held_out: Mapping[str, ArtifactPins]

    def __post_init__(self) -> None:
        for name, suite in (("development", self.development), ("held_out", self.held_out)):
            if not suite:
                raise ScorecardError(f"{name} suite must contain at least one case")
            for case_id, pins in suite.items():
                if not isinstance(case_id, str) or not case_id.strip():
                    raise ScorecardError(f"{name} has an empty or invalid source ID")
                if not isinstance(pins, ArtifactPins):
                    raise ScorecardError(f"{name}/{case_id} has invalid artifact pins")
        overlap = set(self.development).intersection(self.held_out)
        if overlap:
            raise ScorecardError(
                f"source IDs overlap between development and held-out: {sorted(overlap)}"
            )
        source_overlap = {p.source_sha256 for p in self.development.values()} & {
            p.source_sha256 for p in self.held_out.values()
        }
        if source_overlap:
            raise ScorecardError("source contents overlap between development and held-out")
        object.__setattr__(self, "development", MappingProxyType(dict(self.development)))
        object.__setattr__(self, "held_out", MappingProxyType(dict(self.held_out)))

    def pins_for(self, split: Split) -> Mapping[str, ArtifactPins]:
        return self.development if split is Split.DEVELOPMENT else self.held_out

    @property
    def sha256(self) -> str:
        payload = {
            "development": {
                key: self.development[key].as_dict() for key in sorted(self.development)
            },
            "held_out": {key: self.held_out[key].as_dict() for key in sorted(self.held_out)},
        }
        return hashlib.sha256(_canonical(payload)).hexdigest()


def pipeline_fingerprint(files: Mapping[str, Path]) -> str:
    """Hash explicitly named files; never expands globs or fingerprints ambient state."""
    if not files:
        raise ScorecardError("pipeline fingerprint requires explicit files")
    entries: dict[str, str] = {}
    for label, path in files.items():
        if not isinstance(label, str) or not label.strip():
            raise ScorecardError("pipeline file labels must be nonempty strings")
        try:
            entries[label] = _hash_file(Path(path))
        except OSError as exc:
            raise ScorecardError(f"cannot fingerprint pipeline file {label!r}: {exc}") from exc
    return hashlib.sha256(_canonical(entries)).hexdigest()


def artifact_pins_for_files(source: Path, candidate: Path, grader: Path) -> ArtifactPins:
    """Compute provenance pins from the exact three files the runner supplied."""
    try:
        return ArtifactPins(_hash_file(source), _hash_file(candidate), _hash_file(grader))
    except OSError as exc:
        raise ScorecardError(f"cannot hash cohort artifact: {exc}") from exc


@dataclass(frozen=True)
class Check:
    name: str
    executed: bool
    passed: bool

    def __post_init__(self) -> None:
        if not isinstance(self.name, str) or not self.name.strip():
            raise ScorecardError("check name must be nonempty")
        if type(self.executed) is not bool or type(self.passed) is not bool:
            raise ScorecardError("check executed/passed values must be booleans")
        if self.passed and not self.executed:
            raise ScorecardError(f"check {self.name!r} cannot pass without execution")


@dataclass(frozen=True)
class Repair:
    kind: RepairKind
    evidence: str

    def __post_init__(self) -> None:
        if (
            not isinstance(self.kind, RepairKind)
            or not isinstance(self.evidence, str)
            or not self.evidence.strip()
        ):
            raise ScorecardError("repair requires an automatic/manual kind and nonempty evidence")


@dataclass(frozen=True)
class CaseResult:
    case_id: str
    checks: tuple[Check, ...]
    mismatches: int = 0
    unsupported_reason: str | None = None
    repairs: tuple[Repair, ...] = ()
    cost_usd: Decimal | str | None = None
    effort_seconds: Decimal | str | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.case_id, str) or not self.case_id.strip():
            raise ScorecardError("case result ID must be nonempty")
        if (
            isinstance(self.mismatches, bool)
            or not isinstance(self.mismatches, int)
            or self.mismatches < 0
        ):
            raise ScorecardError("mismatches must be a nonnegative integer")
        if self.unsupported_reason is not None and not self.unsupported_reason.strip():
            raise ScorecardError("unsupported_reason must be nonempty when supplied")
        if any(not isinstance(item, Check) for item in self.checks):
            raise ScorecardError(f"{self.case_id}: checks must be Check records")
        if any(not isinstance(item, Repair) for item in self.repairs):
            raise ScorecardError(f"{self.case_id}: repairs must be Repair records")
        object.__setattr__(self, "checks", tuple(self.checks))
        object.__setattr__(self, "repairs", tuple(self.repairs))
        object.__setattr__(self, "cost_usd", _optional_decimal(self.cost_usd, "cost_usd"))
        object.__setattr__(
            self, "effort_seconds", _optional_decimal(self.effort_seconds, "effort_seconds")
        )

    @property
    def classification(self) -> Classification:
        if self.unsupported_reason is not None:
            return Classification.UNSUPPORTED
        if (
            self.mismatches
            or not self.checks
            or any(not c.executed or not c.passed for c in self.checks)
        ):
            return Classification.REJECTED
        if not self.repairs:
            return Classification.UNTOUCHED_SUCCESS
        if any(r.kind is RepairKind.MANUAL for r in self.repairs):
            return Classification.MANUAL_REPAIR
        return Classification.AUTOMATIC_REPAIR

    def as_dict(self) -> dict[str, object]:
        return {
            "case_id": self.case_id,
            "classification": self.classification.value,
            "checks": [
                {"name": c.name, "executed": c.executed, "passed": c.passed} for c in self.checks
            ],
            "mismatches": self.mismatches,
            "unsupported_reason": self.unsupported_reason,
            "repairs": [{"kind": r.kind.value, "evidence": r.evidence} for r in self.repairs],
            # Unknown historical values remain JSON null; they are never imputed as zero.
            "cost_usd": None if self.cost_usd is None else format(self.cost_usd, "f"),
            "effort_seconds": None
            if self.effort_seconds is None
            else format(self.effort_seconds, "f"),
        }


@dataclass(frozen=True)
class Scorecard:
    split: Split
    manifest_sha256: str
    pipeline_sha256: str
    cases: tuple[CaseResult, ...]
    valid: bool
    passed: bool

    def as_dict(self) -> dict[str, object]:
        return {
            "split": self.split.value,
            "manifest_sha256": self.manifest_sha256,
            "pipeline_sha256": self.pipeline_sha256,
            "valid": self.valid,
            "passed": self.passed,
            "case_count": len(self.cases),
            "cases": [case.as_dict() for case in self.cases],
        }


def score_run(
    manifest: CohortManifest,
    split: Split,
    pipeline_sha256: str,
    results: Sequence[CaseResult],
    actual_pins: Mapping[str, ArtifactPins],
    development_report: Scorecard | None = None,
) -> Scorecard:
    """Validate a run's provenance and score evidence; execution remains caller-owned."""
    if not isinstance(split, Split):
        raise ScorecardError("split must be Split.DEVELOPMENT or Split.HELD_OUT")
    if not isinstance(pipeline_sha256, str) or not _SHA256.fullmatch(pipeline_sha256):
        raise ScorecardError("pipeline_sha256 must be a lowercase SHA-256 digest")
    pins = manifest.pins_for(split)
    if not results:
        raise ScorecardError("cannot score an empty suite")
    result_ids = [item.case_id for item in results]
    if len(set(result_ids)) != len(result_ids) or set(result_ids) != set(pins):
        raise ScorecardError("run results must cover each pinned case ID exactly once")
    if set(actual_pins) != set(pins):
        raise ScorecardError("actual artifact hashes must cover each pinned case ID exactly once")
    for case_id, expected in pins.items():
        if actual_pins[case_id] != expected:
            raise ScorecardError(f"artifact SHA-256 mismatch for case {case_id!r}")
    if split is Split.HELD_OUT:
        if (
            development_report is None
            or development_report.split is not Split.DEVELOPMENT
            or not development_report.valid
        ):
            raise ScorecardError("held-out scoring requires a complete valid development report")
        if development_report.manifest_sha256 != manifest.sha256:
            raise ScorecardError("development report belongs to a different cohort manifest")
        if development_report.pipeline_sha256 != pipeline_sha256:
            raise ScorecardError(
                "held-out pipeline fingerprint differs from frozen development pipeline"
            )
    ordered = tuple(sorted(results, key=lambda item: item.case_id))
    valid = all(case.checks and all(check.executed for check in case.checks) for case in ordered)
    passed = (
        valid
        and bool(ordered)
        and all(
            case.classification
            in (
                Classification.UNTOUCHED_SUCCESS,
                Classification.AUTOMATIC_REPAIR,
                Classification.MANUAL_REPAIR,
            )
            and case.mismatches == 0
            and all(check.passed for check in case.checks)
            for case in ordered
        )
    )
    return Scorecard(split, manifest.sha256, pipeline_sha256, ordered, valid, passed)


def _hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode(
        "ascii"
    )


def _optional_decimal(value: Decimal | str | None, name: str) -> Decimal | None:
    if value is None:
        return None
    try:
        number = value if isinstance(value, Decimal) else Decimal(str(value))
    except (InvalidOperation, ValueError, TypeError) as exc:
        raise ScorecardError(f"{name} must be a finite nonnegative decimal or None") from exc
    if not number.is_finite() or number < 0:
        raise ScorecardError(f"{name} must be a finite nonnegative decimal or None")
    return number
