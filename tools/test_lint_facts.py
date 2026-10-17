"""Incremental checks must notice edits outside the file reporting a defect."""

import os
import subprocess
from pathlib import Path

import pytest

from tests.qemu.cli import rustlints
from tools import lint_facts


def test_content_cache_and_uncached_oracle(tmp_path, monkeypatch):
    monkeypatch.setattr(lint_facts, "ROOT", tmp_path)
    calls = []

    @lint_facts.source_facts()
    def scan(data):
        calls.append(data)
        return {"bad": data.count(b"bad")}

    source = tmp_path / "input.rs"
    source.write_bytes(b"bad")
    original = source.stat()
    assert scan(source.read_bytes()) == {"bad": 1}
    assert scan(source.read_bytes()) == {"bad": 1}
    assert len(calls) == 1
    source.write_bytes(b"yes")
    os.utime(source, ns=(original.st_atime_ns, original.st_mtime_ns))
    assert scan(source.read_bytes()) == {"bad": 0}
    assert len(calls) == 2
    monkeypatch.setenv("VIBEOS_LINT_CACHE", "0")
    assert scan(b"bad") == {"bad": 1}
    assert len(calls) == 3


def test_scanner_and_helper_changes_invalidate(tmp_path, monkeypatch):
    monkeypatch.setattr(lint_facts, "ROOT", tmp_path)
    helper = tmp_path / "helper.py"
    helper.write_text("version = 1")
    script = tmp_path / "scanner.py"

    def load(value):
        script.write_text(f"def scan(data):\n    return {value}\n")
        scope = {}
        exec(compile(script.read_text(), str(script), "exec"), scope)
        return lint_facts.source_facts("helper.py")(scope["scan"])

    assert load(1)(b"same input") == 1
    assert load(2)(b"same input") == 2
    # Same scanner bytes, different helper contents must miss the old entry.
    scanner = load(2)
    assert scanner(b"helper input") == 2
    helper.write_text("version = 2")
    reloaded = load(2)
    assert reloaded(b"helper input") == 2
    import sqlite3

    with sqlite3.connect(tmp_path / ".cache/vibeos-lint/scanner.sqlite3") as db:
        assert db.execute("SELECT count(*) FROM facts").fetchone()[0] == 4


def test_failed_scan_is_not_cached(tmp_path, monkeypatch):
    monkeypatch.setattr(lint_facts, "ROOT", tmp_path)
    attempts = []

    @lint_facts.source_facts()
    def broken(data):
        attempts.append(data)
        raise RuntimeError("incomplete")

    for _ in range(2):
        with pytest.raises(RuntimeError, match="incomplete"):
            broken(b"input")
    assert len(attempts) == 2


def _native(root, rule):
    repo = Path(__file__).resolve().parents[1]
    command = [str(rustlints.ensure(repo)), "--root", str(root), "--rules", rule]
    cp = subprocess.run(command, capture_output=True, text=True)
    assert cp.returncode == 0, cp.stderr
    return rustlints.split_output(cp, [rule])[rule].stdout


def test_changed_definition_rechecks_unchanged_caller(tmp_path):
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    crate = tmp_path / "sdk/lib/rust/probe-rust"
    crate.mkdir(parents=True)
    (crate / "Cargo.toml").write_text('[package]\nname = "probe-rust"\n')
    (crate / "src").mkdir()
    caller = crate / "src/caller.rs"
    caller.write_text(
        'extern "system" { fn Probe(x: *mut u32); }\nunsafe fn caller(x: *mut u32) { Probe(x); }\n'
    )
    definition = crate / "src/definition.rs"
    definition.write_text('#[no_mangle] pub extern "system" fn Probe(x: *mut u32) {}')
    assert _native(tmp_path, "scan_extern_decl_drift") == ""
    definition.write_text('#[no_mangle] pub extern "system" fn Probe(x: *mut u64) {}')
    findings = _native(tmp_path, "scan_extern_decl_drift")
    assert "caller.rs" in findings and "WIDTH" in findings
    definition.unlink()
    assert _native(tmp_path, "scan_extern_decl_drift") == ""


def test_irql_wrapper_edit_invalidates_caller(tmp_path):
    sources = tmp_path / "sdk/lib/rust/probe-rust/src"
    sources.mkdir(parents=True)
    wrapper = sources / "wrapper.rs"
    wrapper.write_text("unsafe fn wrapper() -> u8 { 0 }")
    (sources / "caller.rs").write_text(
        "unsafe fn caller(cond: bool) { let saved = wrapper(); "
        "if cond { return; } KeLowerIrql(saved); }"
    )
    assert _native(tmp_path, "scan_irql_balance") == ""
    wrapper.write_text(
        "unsafe fn wrapper() -> u8 { let saved = KeRaiseIrqlToDpcLevel(); return saved; }"
    )
    assert "IRQL_LEAK_RETURN" in _native(tmp_path, "scan_irql_balance")


def test_lint_workers_use_host_affinity(monkeypatch):
    from tests.qemu.cli.lints import lint_jobs

    monkeypatch.delenv("VIBEOS_LINT_JOBS", raising=False)
    monkeypatch.setattr(os, "sched_getaffinity", lambda pid: set(range(24)))
    assert lint_jobs() == 24
