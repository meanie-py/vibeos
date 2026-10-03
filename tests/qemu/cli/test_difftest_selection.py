"""Exact cohort selection must preserve oracle guards and reject missing cases."""

from types import SimpleNamespace

import pytest

from . import artifacts, difftest


@pytest.mark.parametrize("count,expected", [(0, False), (1, False), (2, True), (3, False)])
def test_exact_selection_requires_requested_cardinality(tmp_path, monkeypatch, count, expected):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path)
    run = artifacts.Run("difftest", "host")
    commands = []

    def invoke(argv, **kwargs):
        commands.append(argv)
        return SimpleNamespace(returncode=0, stdout=f"test result: ok. {count} passed", stderr="")

    monkeypatch.setattr(run, "run", invoke)
    assert difftest.run_difftests(tmp_path, run, ["rtl-difftest"], cases=["one", "two"]) is expected
    assert commands[0][-6:] == ["--test", "diff", "--", "--exact", "one", "two"]


def test_selected_run_keeps_stale_oracle_failure(tmp_path, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(difftest, "discover", lambda repo: ["cnt-difftest"])
    seen = []

    def guard(repo, run):
        seen.append("guard")
        run.add(artifacts.Stage("drift", status="fail", detail="stale"))
        return False

    def grade(repo, run, only, **kwargs):
        seen.append("grade")
        assert kwargs["cases"] == ["diff_memmove"]
        run.add(artifacts.Stage("cnt-difftest", status="ok"))
        return True

    monkeypatch.setattr(difftest, "check_generated", guard)
    monkeypatch.setattr(difftest, "run_difftests", grade)
    args = SimpleNamespace(only="cnt-difftest", case=["diff_memmove"], test_target="diff")
    assert difftest.difftest(args, tmp_path) != 0
    assert seen == ["guard", "grade"]


@pytest.mark.parametrize("only,cases", [(None, ["x"]), ("a,b", ["x"]), ("a", ["x", "x"])])
def test_invalid_selection_refused_before_execution(tmp_path, monkeypatch, only, cases):
    monkeypatch.setattr(difftest, "discover", lambda repo: ["a", "b"])
    args = SimpleNamespace(only=only, case=cases)
    assert difftest.difftest(args, tmp_path) == 2
