"""The shared scanner must retain per-rule baseline and coverage guarantees."""

import os
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
    assert set(names) >= rustlints.PORTED
    # Candidates remain callable for parity before being routed into the gate.
    registry = {name for name, _cmd, _stream in lints.source_lint_specs(repo)}
    assert set(names) <= registry
    assert len(names) == len(set(names))


@pytest.mark.parametrize("jobs", [1, 2, 4])
def test_shared_c_index_completes_with_nested_parallel_rules(tmp_path, jobs):
    repo = Path(__file__).resolve().parents[3]
    binary = rustlints.ensure(repo)
    rust = tmp_path / "sdk/lib/rust/demo/src"
    rust.mkdir(parents=True)
    (rust / "lib.rs").write_text("pub unsafe fn Shared(out: *mut u32) -> i32 { *out = 1; 0 }\n")
    source = tmp_path / "demo"
    source.mkdir()
    for name in ("one", "two"):
        (source / f"{name}.c").write_text(
            "int Shared(unsigned *out) { if (out) *out = 1; return 0; }\n"
        )
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    subprocess.run(["git", "-C", str(tmp_path), "add", "demo"], check=True)
    names = ["scan_dropped_null_guard", "scan_store_on_failure", "scan_ptr_byte_diff"]
    cp = subprocess.run(
        [str(binary), "--root", str(tmp_path), "--rules", ",".join(names)],
        env={**os.environ, "VIBEOS_LINT_JOBS": str(jobs)},
        capture_output=True,
        text=True,
        timeout=15,
        check=True,
    )
    results = rustlints.split_output(cp, names)
    assert "dropped-null-guard Shared(out)" in results[names[0]].stdout
    assert all(not results[name].stdout for name in names[1:])


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


@pytest.mark.parametrize("changed_input", ["rust", "conventions"])
def test_bootstrap_rejects_sources_changed_during_build(tmp_path, monkeypatch, changed_input):
    source = tmp_path / "tools/vibeos-lint"
    (source / "src").mkdir(parents=True)
    (source / "Cargo.toml").write_text("[package]\nname='test'\nversion='0.1.0'\n")
    (source / "Cargo.lock").write_text("")
    (tmp_path / "rust-toolchain.toml").write_text("[toolchain]\nchannel='stable'\n")
    conventions = tmp_path / "tools/rustconv/rustconv/conventions.py"
    conventions.parent.mkdir(parents=True)
    conventions.write_text('KW_TO_CONV = {"A": "system"}\n')
    rust = source / "src/main.rs"
    rust.write_text("fn main() {}")
    builds = []

    def build(command, **kwargs):
        builds.append(command)
        artifact = tmp_path / ".cache/vibeos-lint/target/release/vibeos-lint"
        artifact.parent.mkdir(parents=True, exist_ok=True)
        artifact.write_bytes(b"compiled")
        if len(builds) == 1:
            # A new source file must invalidate the build too, not only edits.
            if changed_input == "rust":
                (source / "src/new.rs").write_text("fn newly_added() {}")
            else:
                conventions.write_text('KW_TO_CONV = {"B": "cdecl"}\n')
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr(rustlints.subprocess, "run", build)
    with pytest.raises(RuntimeError, match="sources changed during bootstrap"):
        rustlints.ensure(tmp_path)
    target = tmp_path / ".cache/vibeos-lint/vibeos-lint"
    assert not target.exists()
    assert not target.with_suffix(".sha256").exists()
    assert rustlints.ensure(tmp_path).read_bytes() == b"compiled"
    assert rustlints.ensure(tmp_path) == target
    assert len(builds) == 2
    conventions.write_text('KW_TO_CONV = {"C": "fastcall"}\n')
    assert rustlints.ensure(tmp_path) == target
    assert len(builds) == 3
