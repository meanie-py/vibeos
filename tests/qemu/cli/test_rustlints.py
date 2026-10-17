"""The shared scanner must retain per-rule baseline and coverage guarantees."""

import subprocess
from pathlib import Path

import pytest

from . import lints, rustlints
from .artifacts import Run


@pytest.mark.parametrize(
    "output,code",
    [
        ("a\tpartial\n", 2),
        ("other\tunknown\n", 0),
        ("untagged\n", 0),
        ("a\t\n", 0),
    ],
)
def test_bad_shared_output_fails_every_rule(output, code):
    result = rustlints.split_output(subprocess.CompletedProcess([], code, output, ""), ["a", "b"])
    assert all(lints._scanner_error(cp, "stdout") for cp in result.values())
    assert all(not cp.stdout for cp in result.values())


def test_empty_success_means_both_rules_completed():
    result = rustlints.split_output(
        subprocess.CompletedProcess([], 0, "a\tfinding\n", ""), ["a", "b"]
    )
    assert result["a"].stdout == "finding"
    assert result["b"].stdout == ""
    assert all(not lints._scanner_error(cp, "stdout") for cp in result.values())


def test_plugin_inventory_matches_gate():
    repo = Path(__file__).resolve().parents[3]
    binary = rustlints.ensure(repo)
    result = subprocess.run(
        [str(binary), "--list-rules"], capture_output=True, text=True, check=True
    )
    names = result.stdout.splitlines()
    assert set(names) == rustlints.PORTED
    assert len(names) == len(set(names))


def test_selected_plugins_run_once_and_keep_baselines(tmp_path, monkeypatch):
    monkeypatch.setattr(rustlints, "ensure", lambda repo: Path("/fixture/vibeos-lint"))
    monkeypatch.setattr(lints, "lint_jobs", lambda: 2)
    monkeypatch.setattr(
        lints,
        "source_lint_specs",
        lambda repo: [(name, [rustlints.SENTINEL, name], "stdout") for name in ("a", "b", "c")],
    )
    baseline = tmp_path / lints.BASELINE_DIR
    baseline.mkdir(parents=True)
    (baseline / "a.txt").write_text("source.rs:10: accepted\n")
    calls = []

    def invoke(self, cmd, **kwargs):
        calls.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, "a\tsource.rs:20: accepted\n", "")

    monkeypatch.setattr(Run, "run", invoke)
    run = Run("check", "amd64")
    lints.run_source_lints(run, tmp_path, strict=True, only={"a", "b"})
    assert len(calls) == 1
    assert calls[0][-1] == "a,b"
    assert [(s.name, s.status) for s in run.stages] == [("a", "ok"), ("b", "ok")]


def test_failed_shared_baseline_update_writes_nothing(tmp_path, monkeypatch):
    monkeypatch.setattr(rustlints, "ensure", lambda repo: Path("/fixture/vibeos-lint"))
    monkeypatch.setattr(
        lints, "source_lint_specs", lambda repo: [("a", [rustlints.SENTINEL, "a"], "stdout")]
    )
    monkeypatch.setattr(
        subprocess,
        "run",
        lambda *args, **kwargs: subprocess.CompletedProcess([], 2, "a\tpartial\n", "FATAL: broken"),
    )
    baseline = tmp_path / lints.BASELINE_DIR
    baseline.mkdir(parents=True)
    path = baseline / "a.txt"
    path.write_text("keep me\n")
    with pytest.raises(RuntimeError, match="refusing baseline update"):
        lints.write_baselines(tmp_path)
    assert path.read_text() == "keep me\n"


def test_bootstrap_failure_is_scanner_error(tmp_path, monkeypatch):
    def broken(repo):
        raise RuntimeError("compiler unavailable")

    monkeypatch.setattr(rustlints, "ensure", broken)
    result = rustlints.run_many(tmp_path, ["a"], lambda cmd: pytest.fail("must not invoke"))
    assert lints._scanner_error(result["a"], "stdout")
    assert "compiler unavailable" in result["a"].stderr
