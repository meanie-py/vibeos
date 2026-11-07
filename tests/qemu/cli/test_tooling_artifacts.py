"""Tooling retries retain complete, attributable evidence for each invocation."""

import json
import sys
from types import SimpleNamespace

import pytest

from tests.qemu.cli import artifacts, tooling


@pytest.fixture
def tooling_run(tmp_path, monkeypatch):
    root = tmp_path / "repo"
    (root / "tools").mkdir(parents=True)
    runs = tmp_path / "runs"
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", runs)
    monkeypatch.setattr(artifacts, "ledger_append", lambda *a, **kw: None)
    monkeypatch.setattr(tooling, "_RUFF_ROOTS", [])
    monkeypatch.setattr(tooling, "_PYRIGHT_ROOTS", [])
    monkeypatch.setattr(tooling, "lint_jobs", lambda: 2)
    monkeypatch.setenv("VIBEOS_TOOLING_CACHE", "0")
    return root, runs, SimpleNamespace(no_tests=True, no_types=True, skip_generated=False)


def test_failure_and_green_retry_keep_distinct_full_logs_and_exact_exit_codes(
    tooling_run, monkeypatch
):
    root, runs, args = tooling_run
    real_run = tooling._run
    fail = True

    def controlled(label, _cmd, repo, **kwargs):
        code = 23 if fail and label == "ruff:tools/*.py" else 0
        script = (
            "import sys; print('BEGIN evidence that exceeds the console tail'); "
            "[print('context', i) for i in range(60)]; "
            "print('stderr witness',file=sys.stderr); "
            f"sys.exit({code})"
        )
        return real_run(label, [sys.executable, "-c", script], repo, **kwargs)

    monkeypatch.setattr(tooling, "_run", controlled)
    assert tooling.check_tooling(args, root) == 1
    first = next(runs.iterdir())
    red = json.loads((first / "summary.json").read_text())
    assert red["command"] == "check-tooling" and red["worktree"] == str(root)
    assert red["verdict"] == "FAIL" and red["classification"] == "TOOLING_FAIL"
    assert red["checks"]["total"] == 3 and red["checks"]["failed"] == 1
    stage = next(s for s in red["stages"] if s["name"] == "ruff:tools/*.py")
    assert stage["exit"] == 23 and stage["seconds"] > 0
    assert stage["detail"] == "executed"
    log = (first / "full.log").read_text()
    assert "BEGIN evidence" in log and "context 59" in log and "stderr witness" in log
    assert len(red["command_timings"]) == 3

    fail = False
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

    def broken(*_args, **_kwargs):
        raise exception("controlled tooling runner failure")

    monkeypatch.setattr(tooling, "_run", broken)
    with pytest.raises(exception, match="controlled tooling runner failure"):
        tooling.check_tooling(args, root)
    run = next(runs.iterdir())
    summary = json.loads((run / "summary.json").read_text())
    assert summary["verdict"] == "FAIL" and summary["classification"] == classification
    assert "controlled tooling runner failure" in (run / "full.log").read_text()


def test_receipt_hits_and_validation_failure_are_visible(tooling_run, monkeypatch):
    root, runs, args = tooling_run

    class Receipts:
        def __init__(self, _repo):
            self.hits = set()

        def __enter__(self):
            return self

        def __exit__(self, *_args):
            pass

        def run(self, label, cmd, compute, **kwargs):
            self.hits.add(label)
            return label, True, f"retained receipt output: {label}"

        def stale_generated_hits(self):
            return []

        def finish(self):
            raise RuntimeError("controlled receipt inputs changed")

    monkeypatch.setenv("VIBEOS_TOOLING_CACHE", "1")
    monkeypatch.setattr(tooling, "ReceiptSession", Receipts)
    assert tooling.check_tooling(args, root) == 1
    run = next(runs.iterdir())
    summary = json.loads((run / "summary.json").read_text())
    assert summary["verdict"] == "FAIL"
    assert len(summary["checks"]["reused"]) == 3
    assert "ruff:tools/*.py" in summary["checks"]["reused"]
    assert all(s["detail"] == "reused" for s in summary["stages"][:3])
    assert summary["stages"][-1]["name"] == "receipt-validation"
    assert summary["command_timings"] == []
    assert "retained receipt output" in (run / "full.log").read_text()
    assert "controlled receipt inputs changed" in (run / "full.log").read_text()
