"""Native tooling retries retain complete, attributable evidence for each invocation."""

import json
import subprocess
import sys
from types import SimpleNamespace

import pytest

from tests.qemu.cli import artifacts, tooling


@pytest.fixture
def tooling_run(tmp_path, monkeypatch):
    root = tmp_path / "repo"
    (root / "tools").mkdir(parents=True)
    subprocess.run(["git", "init", "-q", str(root)], check=True)
    runs = tmp_path / "runs"
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", runs)
    monkeypatch.setattr(artifacts, "ledger_append", lambda *a, **kw: None)
    monkeypatch.setattr(tooling, "lint_jobs", lambda: 2)
    monkeypatch.setenv("VIBEOS_TOOLING_CACHE", "0")
    monkeypatch.setattr(tooling, "runtime_identity", lambda _: "runtime")
    return root, runs, SimpleNamespace(no_tests=True, no_types=True, skip_generated=False)


def fixture_checks(fail=False):
    checks = []
    for label in ("ruff:tools/*.py", "fmt:tools/*.py", "generated:ddk_enum_constants"):
        code = 23 if fail and label.startswith("ruff:") else 0
        script = (
            "import sys; print('BEGIN evidence that exceeds the console tail'); "
            "[print('context',i) for i in range(60)]; "
            f"print('stderr witness',file=sys.stderr); sys.exit({code})"
        )
        checks.append({"label": label, "cmd": [sys.executable, "-c", script]})
    return {"checks": checks, "skipped": []}


def test_failure_and_green_retry_keep_distinct_full_logs_and_exact_exit_codes(
    tooling_run, monkeypatch
):
    root, runs, args = tooling_run
    monkeypatch.setattr(tooling, "_checks", lambda *_: fixture_checks(True))
    assert tooling.check_tooling(args, root) == 1
    first = next(runs.iterdir())
    red = json.loads((first / "summary.json").read_text())
    assert red["command"] == "check-tooling" and red["worktree"] == str(root)
    assert red["verdict"] == "FAIL" and red["classification"] == "TOOLING_FAIL"
    assert red["checks"]["total"] == 3 and red["checks"]["failed"] == 1
    stage = next(s for s in red["stages"] if s["name"] == "ruff:tools/*.py")
    assert stage["exit"] == 23 and stage["seconds"] > 0 and stage["detail"] == "executed"
    log = (first / "full.log").read_text()
    assert "BEGIN evidence" in log and "context 59" in log and "stderr witness" in log
    assert len(red["command_timings"]) == 3
    monkeypatch.setattr(tooling, "_checks", lambda *_: fixture_checks())
    assert tooling.check_tooling(args, root) == 0
    second = next(p for p in runs.iterdir() if p != first)
    assert json.loads((second / "summary.json").read_text())["verdict"] == "OK"
    assert json.loads((first / "summary.json").read_text()) == red
    assert (first / "full.log").read_text() == log


@pytest.mark.parametrize(
    "exception,classification",
    [(OSError, "TOOLING_ERROR"), (KeyboardInterrupt, "TOOLING_INTERRUPTED")],
)
def test_runner_exception_still_finalizes_a_failed_summary(
    tooling_run, monkeypatch, exception, classification
):
    root, runs, args = tooling_run

    def broken(*_):
        raise exception("controlled tooling runner failure")

    monkeypatch.setattr(tooling, "_checks", broken)
    with pytest.raises(exception, match="controlled tooling runner failure"):
        tooling.check_tooling(args, root)
    run = next(runs.iterdir())
    summary = json.loads((run / "summary.json").read_text())
    assert summary["verdict"] == "FAIL" and summary["classification"] == classification
    assert "controlled tooling runner failure" in (run / "full.log").read_text()


def test_receipt_hits_and_validation_failure_are_visible(tooling_run, monkeypatch):
    root, runs, args = tooling_run
    monkeypatch.setenv("VIBEOS_TOOLING_CACHE", "1")
    monkeypatch.setattr(tooling, "_checks", lambda *_: fixture_checks())
    assert tooling.check_tooling(args, root) == 0
    before = set(runs.iterdir())
    identities = iter(["runtime", "changed runtime"])
    monkeypatch.setattr(tooling, "runtime_identity", lambda _: next(identities))
    assert tooling.check_tooling(args, root) == 1
    run = next(p for p in runs.iterdir() if p not in before)
    summary = json.loads((run / "summary.json").read_text())
    assert summary["verdict"] == "FAIL" and len(summary["checks"]["reused"]) == 3
    assert all(s["detail"] == "reused" for s in summary["stages"][:3])
    assert summary["stages"][-1]["name"] == "receipt-validation"
    assert summary["command_timings"] == []
    assert "BEGIN evidence" in (run / "full.log").read_text()
    assert "Tooling inputs changed" in (run / "full.log").read_text()
