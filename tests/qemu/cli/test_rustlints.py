"""The shared scanner must retain per-rule baseline and coverage guarantees."""

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from . import lints, rustlints
from .artifacts import Run


def _native_fixture(tmp_path, rule, *args):
    repo = Path(__file__).resolve().parents[3]
    return subprocess.run(
        [str(rustlints.ensure(repo)), "--root", str(tmp_path), "--rules", rule, *args],
        capture_output=True,
        text=True,
        check=False,
    )


def test_native_python_syntax_reports_invalid_source_and_accepts_repair(tmp_path):
    source = tmp_path / "tools/fixture.py"
    source.parent.mkdir()
    (tmp_path / "tests/qemu").mkdir(parents=True)
    source.write_text("def broken(:\n")
    result = _native_fixture(tmp_path, "py-syntax", "--python", sys.executable)
    assert result.returncode == 0, result.stderr
    assert "SyntaxError" in result.stdout
    source.write_text("def repaired():\n    return 1\n")
    result = _native_fixture(tmp_path, "py-syntax", "--python", sys.executable)
    assert result.returncode == 0 and not result.stdout, result.stderr


def test_native_shellcheck_checks_both_script_directories(tmp_path):
    checker = lints._shellcheck_bin()
    assert checker is not None, "shellcheck is required by check-tooling"
    script = tmp_path / "qemu/lib/fixture.sh"
    script.parent.mkdir(parents=True)
    script.write_text('#!/bin/sh\nprintf "%s\\n" $unquoted\n')
    result = _native_fixture(tmp_path, "shellcheck", "--shellcheck", checker)
    assert result.returncode == 0 and "SC2154" in result.stdout, result.stderr
    script.write_text('#!/bin/sh\nquoted=hello\nprintf "%s\\n" "$quoted"\n')
    result = _native_fixture(tmp_path, "shellcheck", "--shellcheck", checker)
    assert result.returncode == 0 and not result.stdout, result.stderr


@pytest.mark.parametrize("rule", ["verify_svc_numbering", "marshalgen_audit"])
def test_native_artifact_checks_preserve_missing_build_skips_without_a_workspace(tmp_path, rule):
    result = _native_fixture(tmp_path, rule)
    assert result.returncode == 0 and not result.stdout, result.stderr


def test_native_svc_check_rejects_a_guest_syscall_routed_to_its_neighbor(tmp_path):
    def write(path, text):
        dest = tmp_path / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(text)
        return dest

    table = write(
        "build-arm64/dll/win32/wow64/wow64_win32k_thunks.gen.c",
        '{ "NtUserFixture", 0, 1 }, /* [0] MANUAL */\n',
    )
    for dll in ("win32ss/user/user32/user32.dll", "win32ss/gdi/gdi32/gdi32.dll"):
        write(f"build-i386/{dll}", "fixture PE; disassembly supplied by the test tool\n")
    objdump = write(
        ".build-tools/llvm-mingw/bin/llvm-objdump",
        "#!/bin/sh\nprintf '0000 <_ZwUserFixture@0>:\\n  movl $0x1000, %%eax\\n'\n",
    )
    objdump.chmod(0o755)
    result = _native_fixture(tmp_path, "verify_svc_numbering")
    assert result.returncode == 0 and not result.stdout, result.stderr
    table.write_text('{ "NtUserNeighbor", 0, 1 }, /* [0] MANUAL */\n')
    result = _native_fixture(tmp_path, "verify_svc_numbering")
    assert result.returncode != 0 and "misrouted" in result.stderr
    assert "guest calls NtUserFixture" in result.stderr
    assert not result.stdout


def test_native_marshal_audit_checks_layouts_and_rejects_stale_table(tmp_path):
    def write(path, content):
        dest = tmp_path / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(content)
        return dest

    headers = [
        "win32ss/include/ntuser.h",
        "sdk/include/psdk/ntgdi.h",
        "win32ss/include/ntgdibad.h",
        *[
            f"sdk/include/ndk/{n}funcs.h"
            for n in ("cm", "dbgk", "ex", "io", "kd", "ke", "lpc", "mm", "ob", "po", "ps", "se")
        ],
    ]
    for header in headers:
        write(header, "")
    write(headers[0], "NTSTATUS NTAPI NtUserFixture(SIZE_T *count);\n")
    write("win32ss/win32u/win32u.spec", "@ stdcall NtUserFixture(ptr)\n")
    write("win32ss/w32ksvc32.h", "SVC_(UserFixture, 1)\n")
    write("dll/ntdll/def/ntdll.spec", "@ stdcall NtFixture(long)\n")
    write("ntoskrnl/include/sysfuncs.h", "SVC_(Fixture, 1)\n")
    for name in ("manual.txt", "manual-win32k.txt", "automarshal-win32k.txt"):
        write(f"tools/wow64gen/{name}", "")
    tus = ["user32_layout_probe.c", "gdi32/wine/rust_cohab_stub.c", "ntdll/rtl/rtl_um_rust_stub.c"]
    for arch in ("i386", "amd64"):
        write(
            f"build-{arch}/compile_commands.json",
            json.dumps(
                [
                    {"file": suffix, "command": "clang -I /tmp/vibeos/sdk/include -c " + suffix}
                    for suffix in tus
                ]
            ),
        )
    clang = write(
        ".build-tools/llvm-mingw/bin/clang",
        "#!/bin/sh\ncat <<'LAYOUT'\n"
        "*** Dumping AST Record Layout\n"
        "         0 | struct _FIXTURE\n"
        "         0 |   int value\n"
        "           | [sizeof=4, align=4]\nLAYOUT\n",
    )
    clang.chmod(0o755)
    result = _native_fixture(tmp_path, "marshalgen_audit")
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == (
        "marshalgen_audit\tFLAT-DIVERGENT: win32k NtUserFixture(count) "
        "SCALAR_WIDEN -> SIZE_T: 4->8 scalar pointee"
    )
    generated = write(
        "build-amd64/dll/win32/wow64/wow64_win32k_thunks.gen.c",
        '{ "NtUserFixture", 0, 1 }, /* [0] MANUAL */\n',
    )
    result = _native_fixture(tmp_path, "marshalgen_audit")
    assert result.returncode != 0 and "stale" in result.stderr.lower()
    assert not result.stdout
    generated.unlink()
    write(headers[0], "NTSTATUS NTAPI NtUserFixture(PULONG count);\n")
    result = _native_fixture(tmp_path, "marshalgen_audit")
    assert result.returncode == 0 and not result.stdout, result.stderr


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
    assert registry == rustlints.PORTED, "add every source check as a native Rust plugin"
    assert set(names) <= registry
    assert len(names) == len(set(names))


def test_native_shellcheck_routing_retains_missing_tool_coverage(monkeypatch, tmp_path):
    monkeypatch.setattr(lints, "_shellcheck_bin", lambda: None)
    specs = {name: cmd for name, cmd, _ in lints.source_lint_specs(tmp_path)}
    assert specs["shellcheck"] == ["MISSING"]
    monkeypatch.setattr(lints, "_shellcheck_bin", lambda: "/fixture/shellcheck")
    assert dict((n, c) for n, c, _ in lints.source_lint_specs(tmp_path))["shellcheck"] == [
        "NOCHECK"
    ]
    (tmp_path / "qemu").mkdir()
    (tmp_path / "qemu/example.sh").write_text("#!/bin/sh\n")
    assert dict((n, c) for n, c, _ in lints.source_lint_specs(tmp_path))["shellcheck"] == [
        rustlints.SENTINEL,
        "shellcheck",
    ]


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


@pytest.mark.parametrize("changed_input", ["rust", "conventions", "grants", "engines"])
def test_bootstrap_rejects_sources_changed_during_build(tmp_path, monkeypatch, changed_input):
    source = tmp_path / "tools/vibeos-lint"
    (source / "src").mkdir(parents=True)
    (source / "Cargo.toml").write_text("[package]\nname='test'\nversion='0.1.0'\n")
    (source / "Cargo.lock").write_text("")
    (tmp_path / "rust-toolchain.toml").write_text("[toolchain]\nchannel='stable'\n")
    conventions = tmp_path / "tools/rustconv/rustconv/conventions.py"
    conventions.parent.mkdir(parents=True)
    conventions.write_text('KW_TO_CONV = {"A": "system"}\n')
    grants = tmp_path / "tools/vendor-grants.json"
    grants.write_text("{}\n")
    engines = tmp_path / "tools/cargo_cell_gen.py"
    engines.write_text("ENGINE_LIBS = {}\n")
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
            elif changed_input == "conventions":
                conventions.write_text('KW_TO_CONV = {"B": "cdecl"}\n')
            elif changed_input == "grants":
                grants.write_text('{"ptr:fixture": "reviewed"}\n')
            else:
                engines.write_text('ENGINE_LIBS = {"libfixture.a": "engine:fixture"}\n')
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
