import json
import subprocess

import pytest

from . import artifacts, gate, lints, queue
from .test_cli_units import _difftest_once_stub, _fake_smoke, _gate_args, _tooling_once_stub

TRACE = "Traceback (most recent call last):\nFileNotFoundError: vanished\n"


@pytest.mark.parametrize(
    "stdout,stderr,code",
    [
        ("", TRACE, 1),
        ("file.rs:1: finding\n", TRACE, 1),
        ("file.rs:1: finding\n", "", -9),
        ("", "permission denied", 1),
        ("", TRACE, 0),
    ],
)
def test_crash_is_not_a_finding_or_baseline_success(tmp_path, monkeypatch, stdout, stderr, code):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    run = artifacts.Run("check", "all")
    monkeypatch.setattr(
        run, "run", lambda *a, **k: subprocess.CompletedProcess([], code, stdout, stderr)
    )
    monkeypatch.setattr(lints, "_baseline", lambda *a: {"file.rs:1: finding"})
    stage = lints._lint_stage(run, tmp_path, "fixture", ["fixture"], strict=False)
    assert stage.status == "fail"  # even an advisory scan cannot claim completion
    assert run.classification == "SCANNER_ERROR"
    assert "coverage unavailable" in stage.detail
    assert "SCANNER_ERROR" in queue.INCONCLUSIVE


def test_normal_baselined_findings_remain_accepted(tmp_path, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    run = artifacts.Run("check", "all")
    monkeypatch.setattr(
        run, "run", lambda *a, **k: subprocess.CompletedProcess([], 1, "a.rs:1: bug\n", "")
    )
    monkeypatch.setattr(lints, "_baseline", lambda *a: {"a.rs:1: bug"})
    assert lints._lint_stage(run, tmp_path, "fixture", ["fixture"], strict=True).status == "ok"
    assert not run.classification


def test_lint_partition_preserves_scanner_classification(tmp_path, monkeypatch):
    monkeypatch.setattr(
        gate.subprocess, "run", lambda *a, **k: subprocess.CompletedProcess([], 1, "", TRACE)
    )
    monkeypatch.setattr(lints, "source_partition_specs", lambda *a: [("fixture", [], "stdout")])
    monkeypatch.setattr(
        gate,
        "_read_summary",
        lambda *a: {
            "classification": "SCANNER_ERROR",
            "checked_source_checks": ["fixture"],
        },
    )
    result = gate._lint_once(tmp_path, "source-early", tmp_path / "run")
    assert not result["ok"]
    assert result["classification"] == "SCANNER_ERROR"


def test_baseline_update_refuses_partial_crash_output(tmp_path, monkeypatch):
    monkeypatch.setattr(lints, "source_lint_specs", lambda *a: [("fixture", ["tool"], "stdout")])
    monkeypatch.setattr(
        lints.subprocess,
        "run",
        lambda *a, **k: subprocess.CompletedProcess([], 1, "a.rs:1: bug\n", TRACE),
    )
    with pytest.raises(RuntimeError, match="refusing baseline"):
        lints.write_baselines(tmp_path)
    assert not (tmp_path / lints.BASELINE_DIR / "fixture.txt").exists()


def test_gate_crash_blocks_stamp_without_blaming_findings(tmp_path, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    monkeypatch.setattr(
        gate, "_smoke_once", _fake_smoke({a: ["OK"] for a in ("i386", "amd64", "arm64")})
    )
    monkeypatch.setattr(gate, "_tooling_once", _tooling_once_stub(True))
    monkeypatch.setattr(gate, "_difftest_once", _difftest_once_stub(True))

    def lint(repo, arch, root):
        return {
            "arch": arch,
            "ok": False,
            "exit": 1,
            "seconds": 0,
            "salient": "scanner crash",
            "classification": "SCANNER_ERROR",
        }

    monkeypatch.setattr(gate, "_lint_once", lint)
    stamped = []
    monkeypatch.setattr(gate, "_write_gate_stamp", lambda *a, **k: stamped.append(True))
    assert gate.gate(_gate_args(no_lint=False), tmp_path) == 1
    assert not stamped
    summaries = list((tmp_path / "runs").glob("*/summary.json"))
    assert len(summaries) == 1
    summary = json.loads(summaries[0].read_text())
    assert summary["classification"] == "SCANNER_ERROR"
    stages = [stage for stage in summary["stages"] if stage["name"].startswith("lint:")]
    assert stages and all("NEW lint findings" not in stage["detail"] for stage in stages)
