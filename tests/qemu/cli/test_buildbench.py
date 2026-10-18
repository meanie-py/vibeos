"""Focused unit tests for the staged buildbench module."""

import argparse
import json
from pathlib import Path
from types import SimpleNamespace

import pytest

from . import buildbench


def test_meminfo_converts_kib_to_bytes(tmp_path):
    source = tmp_path / "meminfo"
    source.write_text("MemTotal:       100 kB\nHugePages_Total: 2\n")
    assert buildbench._read_meminfo(source) == {"MemTotal": 102400, "HugePages_Total": 2}


def test_vmstat_keeps_oom_and_major_fault_counters(tmp_path):
    source = tmp_path / "vmstat"
    source.write_text("oom_kill 4\npgmajfault 9\npgfault 100\n")
    assert buildbench._read_vmstat(source) == {"oom_kill": 4, "pgmajfault": 9}


def test_missing_psi_is_explicitly_unavailable(tmp_path):
    pressure = buildbench._read_pressure(tmp_path)
    assert all(
        value == {"available": False, "reason": "kernel PSI file absent"}
        for value in pressure.values()
    )


def test_parser_uses_dispatcher_fn_and_arch_choices():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers()
    buildbench.add_parser(sub)
    args = parser.parse_args(["build-bench", "--arches", "amd64", "i386"])
    assert args.arches == ["amd64", "i386"]
    assert args.repeat == 1
    assert args.fn is buildbench.run


def test_parallel_children_record_exit_and_launch_failure(tmp_path, monkeypatch):
    def fake_run(command, **kwargs):
        if command[-1] == "spawn-error":
            raise OSError("executable unavailable")
        return SimpleNamespace(returncode=3)

    monkeypatch.setattr(buildbench.subprocess, "run", fake_run)
    rows = buildbench._run_children(
        [("child-bad-exit", ["tool", "nonzero"]), ("child-spawn", ["tool", "spawn-error"])],
        tmp_path,
        tmp_path,
    )
    by_name = {row["name"]: row for row in rows}
    assert by_name["child-bad-exit"]["exit_code"] == 3
    assert by_name["child-spawn"]["exit_code"] == 127
    assert "executable unavailable" in by_name["child-spawn"]["launch_error"]
    assert Path(by_name["child-spawn"]["log"]).is_file()


def test_arch_media_paths_include_arm64_ramdisk(tmp_path):
    assert buildbench._media_path(tmp_path, "i386") == tmp_path / "build-i386/livecd.iso"
    assert buildbench._media_path(tmp_path, "amd64") == tmp_path / "build-amd64/livecd.iso"
    assert buildbench._media_path(tmp_path, "arm64") == tmp_path / "qemu/ramdisk.img"


def test_zero_exit_without_child_verdict_fails(tmp_path, monkeypatch):
    monkeypatch.setattr(
        buildbench.subprocess, "run", lambda *a, **kw: SimpleNamespace(returncode=0)
    )
    row = buildbench._run_child("build", ["tool"], tmp_path, tmp_path / "build.log")
    assert row["exit_code"] == 1
    assert "summary" in row["artifact_error"]


@pytest.mark.parametrize("failed_build", [False, True])
def test_benchmark_defers_artifact_checks_and_preserves_failures(
    tmp_path, monkeypatch, failed_build
):
    monkeypatch.setattr(buildbench, "_git_state", lambda repo: {"head": "abc", "dirty": False})
    monkeypatch.setattr(buildbench.buildpolicy, "read", lambda path: {})
    monkeypatch.setattr(buildbench.hostjobs, "status", lambda repo: {})
    phases = []

    def children(commands, repo, log_dir):
        phases.append([name for name, _ in commands])
        if len(phases) == 1:
            assert "--no-check" in commands[0][1]
            assert "source-early" in commands[-1][1]
            artifact = tmp_path / "build-i386/livecd.iso"
            artifact.parent.mkdir()
            artifact.write_bytes(b"media")
        return [
            {"name": name, "exit_code": int(failed_build and name == "build-i386")}
            for name, _ in commands
        ]

    monkeypatch.setattr(buildbench, "_run_children", children)
    out = tmp_path / "report.json"
    args = SimpleNamespace(repeat=1, arches=["i386"], clean=False, output=out)
    assert buildbench.run(args, tmp_path) == int(failed_build)
    report = json.loads(out.read_text())
    assert report["status"] == ("FAILED" if failed_build else "OK")
    assert len(phases) == (1 if failed_build else 2)
    if not failed_build:
        assert phases[1] == ["check-source-deferred", "check-arch-i386"]
    assert Path(report["repeats"][0]["samples_jsonl"]).is_file()
    with pytest.raises(SystemExit, match="already exists"):
        buildbench.run(args, tmp_path)


def test_sampler_read_failure_is_not_reported_as_success(tmp_path, monkeypatch):
    def fail():
        raise OSError("unreadable meminfo")

    monkeypatch.setattr(buildbench, "_read_meminfo", fail)
    sampler = buildbench.Sampler(tmp_path / "samples.jsonl")
    sampler.start()
    assert sampler.thread is not None
    sampler.thread.join()
    with pytest.raises(RuntimeError, match="unreadable meminfo"):
        sampler.stop()


@pytest.mark.parametrize("changed_inputs", [False, True])
def test_matrix_restages_amd64_after_i386_and_detects_input_change(
    tmp_path, monkeypatch, changed_inputs
):
    final_state = {"head": "def" if changed_inputs else "abc", "dirty": False}
    states = iter([{"head": "abc", "dirty": False}, final_state])
    monkeypatch.setattr(buildbench, "_git_state", lambda repo: next(states, final_state))
    monkeypatch.setattr(buildbench.buildpolicy, "read", lambda path: {})
    monkeypatch.setattr(buildbench.hostjobs, "status", lambda repo: {})
    phases = []

    def children(commands, repo, log_dir):
        phases.append(commands)
        if len(phases) == 1:
            for arch in ("amd64", "i386"):
                artifact = tmp_path / f"build-{arch}/livecd.iso"
                artifact.parent.mkdir()
                artifact.write_bytes(b"media")
        elif len(phases) == 2:
            (tmp_path / "build-amd64/livecd.iso").write_bytes(b"restaged media")
        return [{"name": name, "exit_code": 0} for name, _ in commands]

    monkeypatch.setattr(buildbench, "_run_children", children)
    out = tmp_path / "report.json"
    args = SimpleNamespace(repeat=1, arches=["amd64", "i386"], clean=True, output=out)
    assert buildbench.run(args, tmp_path) == int(changed_inputs)
    assert "--no-wow64-stage" in phases[0][0][1]
    assert phases[1][0][0] == "build-amd64-restage"
    assert "--clean" not in phases[1][0][1]
    report = json.loads(out.read_text())
    if changed_inputs:
        assert report["status"] == "FAILED"
        assert "git state changed" in report["repeats"][0]["error"]
    else:
        assert report["status"] == "OK"
        assert report["repeats"][0]["artifacts"]["amd64"]["size_bytes"] == len(b"restaged media")
