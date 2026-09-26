"""Measurement failures must not become candidate-quality statistics."""

import json
from types import SimpleNamespace

import pytest

from rustconv import cohort_runner as runner
from rustconv import cohort_scorecard as score


def test_rustup_proxy_keeps_argv0(tmp_path, monkeypatch):
    proxy = tmp_path / "rustup"
    proxy.write_bytes(b"proxy")
    rustc = tmp_path / "rustc"
    rustc.symlink_to(proxy)
    monkeypatch.setattr(runner.shutil, "which", lambda _: str(rustc))
    commands = []

    def run(argv, **kwargs):
        commands.append(argv)
        return SimpleNamespace(stdout="rustc 1.90.0\n")

    monkeypatch.setattr(runner.subprocess, "run", run)
    facts = runner.compiler_provenance("rustc")
    assert commands == [[str(rustc), "--version"]]
    assert facts["executable"] == str(rustc)
    assert facts["resolved_executable"] == str(proxy)


@pytest.mark.parametrize("token", ["repo:../outside", "repo:/absolute", "unknown:file"])
def test_artifact_roots_are_explicit(tmp_path, token):
    with pytest.raises(ValueError):
        runner.resolve({"repo": tmp_path}, token)


def test_toolchain_drift_refuses_frozen_manifest(monkeypatch):
    frozen = {
        "cohort_manifest_sha256": "a",
        "pipeline_sha256": "b",
        "cases": [],
        "host_compilers": {"rustc": "old"},
    }
    monkeypatch.setattr(
        runner, "materialize", lambda _: {**frozen, "host_compilers": {"rustc": "new"}}
    )
    with pytest.raises(ValueError, match="manifest drift"):
        runner.verify_manifest(None, frozen)


@pytest.mark.parametrize("failure", ["json", "mutation", "candidate"])
def test_oracle_failure_is_distinct_from_candidate_rejection(tmp_path, monkeypatch, failure):
    stdout = tmp_path / "stdout.json"
    report = {
        "grade": {
            "cases": 10,
            "mismatches": int(failure == "candidate"),
            "guard_failures": 0,
            "passed": failure != "candidate",
        },
        "mutations": [{"detected": True}, {"detected": failure != "mutation"}],
    }
    stdout.write_text("not JSON" if failure == "json" else json.dumps(report))
    monkeypatch.setattr(runner, "execute", lambda *a, **kw: {"stdout": str(stdout), "exit_code": 1})
    args = SimpleNamespace(
        repo=tmp_path, staging=tmp_path, shell32_bank=tmp_path, fixtures_root=tmp_path
    )
    row = {
        "kind": "rc4",
        "case_id": "rc4.c",
        "split": "held_out",
        "pins": {},
        "coverage_scope": "rc4_init/rc4_crypt",
    }
    if failure == "candidate":
        result, _ = runner.run_row(row, args, tmp_path, score)
        assert result.classification is score.Classification.REJECTED
        assert result.mismatches == 1
    else:
        with pytest.raises(ValueError, match=r"no JSON|mutation controls"):
            runner.run_row(row, args, tmp_path, score)


def test_cpp_full_report_supplies_verdict_and_provenance(tmp_path):
    prefix = tmp_path / "case"
    stdout = tmp_path / "stdout"
    stdout.write_text('{"cases_compared":83}')
    report = {
        "cases_compared": 83,
        "candidate_failures": [{"method": "Next"}],
        "reference_provenance": {"headers": ["actual header hash"]},
    }
    prefix.with_suffix(".grade.json").write_text(json.dumps(report))
    command = {"stdout": str(stdout)}
    assert runner.grader_report(command, "cenum", prefix) == report
    assert "grader_report_sha256" in command
    prefix.with_suffix(".grade.json").unlink()
    with pytest.raises(FileNotFoundError):
        runner.grader_report(command, "cenum", prefix)
