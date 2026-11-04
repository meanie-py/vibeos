"""Unit tests for wow64gen — the WoW64 syscall thunk generator."""

import importlib.util
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("wow64gen", os.path.join(_HERE, "wow64gen.py"))
w = importlib.util.module_from_spec(_spec)
sys.modules["wow64gen"] = w  # dataclass annotation resolution needs this
_spec.loader.exec_module(w)


def _spec_file(tmp_path, body):
    p = tmp_path / "t.spec"
    p.write_text(body)
    return str(p)


def test_parses_nt_entries_and_skips_zw(tmp_path):
    sp = _spec_file(
        tmp_path,
        (
            "@ stdcall NtAllocateVirtualMemory(long ptr ptr ptr long long)\n"
            "@ stdcall ZwAllocateVirtualMemory(long ptr ptr ptr long long)\n"
            "@ stdcall RtlUnwind(ptr ptr ptr ptr)\n"
        ),
    )
    syscalls = w.parse_spec(sp, "arm64")
    names = [s.name for s in syscalls]
    assert names == ["NtAllocateVirtualMemory"]  # Zw + Rtl skipped
    assert syscalls[0].ordinal == 0
    assert syscalls[0].slots == 6  # six single-dword args


def test_ordinals_are_sequential_over_applicable_entries(tmp_path):
    sp = _spec_file(
        tmp_path, ("@ stdcall NtA(long)\n@ stdcall NtB(ptr)\n@ stdcall NtC(long long)\n")
    )
    syscalls = w.parse_spec(sp, "arm64")
    assert [s.ordinal for s in syscalls] == [0, 1, 2]
    assert [s.name for s in syscalls] == ["NtA", "NtB", "NtC"]


def _sysfuncs_file(tmp_path, body):
    p = tmp_path / "sysfuncs.h"
    p.write_text(body)
    return str(p)


def test_sysfuncs_order_overrides_spec_order(tmp_path):
    # Spec lists A then B; sysfuncs numbers B=0, A=1 — the table must follow
    # sysfuncs (that's what guest EAX carries), not the spec.
    sp = _spec_file(tmp_path, "@ stdcall NtA(long)\n@ stdcall NtB(ptr ptr)\n")
    sf = _sysfuncs_file(tmp_path, "    SVC_(B, 2)\n    SVC_(A, 1)\n")
    ordered = w.order_by_sysfuncs(w.parse_sysfuncs(sf), w.parse_spec(sp, "arm64"))
    assert [(s.name, s.ordinal) for s in ordered] == [("NtB", 0), ("NtA", 1)]
    assert not ordered[0].needs_manual and not ordered[1].needs_manual


def test_sysfuncs_entry_missing_from_spec_is_loud_manual(tmp_path):
    sp = _spec_file(tmp_path, "@ stdcall NtA(long)\n")
    sf = _sysfuncs_file(tmp_path, "SVC_(A, 1)\nSVC_(Mystery, 4)\n")
    ordered = w.order_by_sysfuncs(w.parse_sysfuncs(sf), w.parse_spec(sp, "arm64"))
    assert ordered[1].name == "NtMystery"
    assert ordered[1].needs_manual
    assert ordered[1].slots == 4  # argcount still recorded for stack accounting


def test_sysfuncs_argcount_mismatch_is_loud_manual(tmp_path):
    sp = _spec_file(tmp_path, "@ stdcall NtA(long)\n")  # spec says 1 dword
    sf = _sysfuncs_file(tmp_path, "SVC_(A, 3)\n")  # table says 3
    ordered = w.order_by_sysfuncs(w.parse_sysfuncs(sf), w.parse_spec(sp, "arm64"))
    assert ordered[0].needs_manual
    assert "argcount" in ordered[0].manual_reason


def test_real_sysfuncs_alignment():
    """The real sysfuncs.h x real ntdll.spec: parse, align, and stay deterministic."""
    repo = os.path.abspath(os.path.join(_HERE, "..", ".."))
    spec = os.path.join(repo, "dll", "ntdll", "def", "ntdll.spec")
    sf = os.path.join(repo, "ntoskrnl", "include", "sysfuncs.h")
    if not (os.path.exists(spec) and os.path.exists(sf)):
        return  # not in a full checkout
    sysfuncs = w.parse_sysfuncs(sf)
    assert len(sysfuncs) > 250
    ordered = w.order_by_sysfuncs(sysfuncs, w.parse_spec(spec, "arm64"))
    assert len(ordered) == len(sysfuncs)
    # Known anchors: index 0 is NtAcceptConnectPort in NT 5.2 tables.
    assert ordered[0].name == "NtAcceptConnectPort"
    c1 = w.emit_c(ordered, spec, "arm64", "-")
    c2 = w.emit_c(w.order_by_sysfuncs(sysfuncs, w.parse_spec(spec, "arm64")), spec, "arm64", "-")
    assert c1 == c2


def test_arch_filter_includes_and_excludes(tmp_path):
    sp = _spec_file(
        tmp_path,
        (
            "@ stdcall NtNative(long)\n"
            "@ stdcall -arch=arm64 NtOnlyArm(long)\n"
            "@ stdcall -arch=i386 NtOnlyX86(long)\n"
            "@ stdcall -arch=win64 NtWin64(long)\n"
        ),
    )
    arm = {s.name for s in w.parse_spec(sp, "arm64")}
    assert "NtNative" in arm  # no -arch => all
    assert "NtOnlyArm" in arm  # explicit match
    assert "NtOnlyX86" not in arm  # other arch excluded
    x86 = {s.name for s in w.parse_spec(sp, "i386")}
    assert "NtOnlyX86" in x86 and "NtOnlyArm" not in x86


def test_int64_and_double_consume_two_slots(tmp_path):
    sp = _spec_file(tmp_path, "@ stdcall NtBig(long int64 ptr double)\n")
    s = w.parse_spec(sp, "arm64")[0]
    assert s.slots == 1 + 2 + 1 + 2  # long + int64 + ptr + double
    assert not s.needs_manual


def test_unknown_token_flags_manual(tmp_path):
    sp = _spec_file(tmp_path, "@ stdcall NtWeird(long banana)\n")
    s = w.parse_spec(sp, "arm64")[0]
    assert s.needs_manual
    assert "banana" in s.manual_reason


def test_manual_list_forces_manual(tmp_path):
    sp = _spec_file(tmp_path, "@ stdcall NtCreateFile(long ptr ptr)\n")
    manual = {"NtCreateFile"}
    s = w.parse_spec(sp, "arm64", manual)[0]
    assert s.needs_manual
    assert "layout" in s.manual_reason


def test_emitted_thunk_widens_args_correctly(tmp_path):
    sp = _spec_file(tmp_path, "@ stdcall NtFoo(long ptr int64)\n")
    syscalls = w.parse_spec(sp, "arm64")
    c = w.emit_c(syscalls, sp, "arm64", "-")
    # 1-dword args sign-extend to 64-bit (pseudo-handle safety — see ARG_KINDS)
    assert "(ULONG_PTR)(LONG_PTR)(LONG)a[0]" in c
    assert "(PVOID)(LONG_PTR)(LONG)a[1]" in c
    assert "((ULONGLONG)a[2] | ((ULONGLONG)a[3] << 32))" in c
    assert "wow64_thunk_NtFoo" in c
    assert "Wow64ThunkTable" in c


def test_manual_entry_has_null_thunk_in_table(tmp_path):
    sp = _spec_file(tmp_path, "@ stdcall NtBar(long ptr)\n")
    syscalls = w.parse_spec(sp, "arm64", {"NtBar"})
    c = w.emit_c(syscalls, sp, "arm64", "-")
    assert '{ "NtBar", 0,' in c  # null thunk pointer
    assert "wow64_thunk_NtBar" not in c  # no auto body emitted


def test_real_ntdll_spec_generates_and_is_stable():
    """Smoke: the real spec parses, and generation is deterministic."""
    repo = os.path.abspath(os.path.join(_HERE, "..", ".."))
    spec = os.path.join(repo, "dll", "ntdll", "def", "ntdll.spec")
    if not os.path.exists(spec):
        return  # not in a full checkout
    a = w.parse_spec(spec, "arm64")
    b = w.parse_spec(spec, "arm64")
    assert len(a) > 100
    assert [s.name for s in a] == [s.name for s in b]  # deterministic order
    c1 = w.emit_c(a, spec, "arm64", "-")
    c2 = w.emit_c(b, spec, "arm64", "-")
    assert c1 == c2  # re-runnable: identical output for identical input


# --- marshalgen audit (E10): the flat-thunk blind-spot gate --------------------

_mspec = importlib.util.spec_from_file_location("marshalgen", os.path.join(_HERE, "marshalgen.py"))
mg = importlib.util.module_from_spec(_mspec)
sys.modules["marshalgen"] = mg
_mspec.loader.exec_module(mg)


def test_thunk_table_separates_live_flat_thunks_from_stubs(tmp_path):
    """The audit's ground truth is the GENERATED table, not the .spec: only an
    entry that still names a wow64_thunk_* body is dispatched flat."""
    gen = tmp_path / "t.gen.c"
    gen.write_text(
        "static const X Wow64ThunkTable[] = {\n"
        '    { "NtGdiAbortDoc", wow64_thunk_NtGdiAbortDoc, 1 }, /* [0] */\n'
        '    { "NtUserBeginPaint", 0, 2 }, /* [307] MANUAL: listed in --manual-list */\n'
        "};\n"
    )
    table = mg.load_thunk_table(gen)
    assert table == {"NtGdiAbortDoc": True, "NtUserBeginPaint": False}


def test_audit_verdict_flags_divergence_but_not_tool_blindness():
    # Proven layout divergence under a flat thunk -> the E6/E8/E9 bugcheck shape.
    assert mg.audit_verdict("WIDENING", "-> UNICODE_STRING") == "DIVERGENT"
    # Pointer to a 4->8 scalar (HWND*/PVOID*) -> the E7 guest-heap overrun.
    assert mg.audit_verdict("SCALAR_WIDEN", "-> HWND: 4->8 scalar pointee") == "DIVERGENT"
    # A struct marshalgen genuinely can't reason about is still a finding.
    assert mg.audit_verdict("SEMANTIC", "union/bitfield with divergent layout") == "DIVERGENT"
    # ...but "I couldn't SEE the layout" is a hole in the TOOL, not a claim
    # about the syscall. Gating on it would bury the real findings in noise.
    assert (
        mg.audit_verdict("SEMANTIC", "-> FOO: layout not in dump (add header to TU?)") == "UNKNOWN"
    )
    assert mg.audit_verdict("SEMANTIC", "unknown type 'BAR'") == "UNKNOWN"
    assert mg.audit_verdict("FLAT", "scalar") == "OK"
    assert mg.audit_verdict("IDENTICAL", "") == "OK"


def test_audit_skips_cleanly_without_a_build(tmp_path, monkeypatch):
    """No build dirs must be a silent SKIP (rc 0), never a gate failure."""
    monkeypatch.setattr(mg, "REPO", tmp_path)

    class A:
        build_dir = None
        build_dir32 = None
        table = "both"
        show_unknown = False

    assert mg.cmd_audit(A()) == 0


def test_audit_table_comes_from_sources_not_the_build(tmp_path):
    """The table must be derivable from CHECKED-IN inputs alone. If this ever
    regresses to reading build-*/...gen.c, the lint's baseline stops being
    portable and stale build dirs re-report already-fixed syscalls."""
    repo = os.path.abspath(os.path.join(_HERE, "..", ".."))
    cfg = mg.AUDIT_TABLES["win32k"]
    for key in ("spec", "sysfuncs", "manual_lists"):
        assert key in cfg, f"win32k audit config lost its {key} source"
    if not os.path.exists(os.path.join(repo, cfg["spec"])):
        return  # not a full checkout
    table = mg.compute_thunk_table(cfg, "amd64")
    assert len(table) > 500
    # Names on the manual list must NOT be reported as live flat thunks.
    assert table["NtGdiCreateDIBSection"] is False
    assert table["NtUserGetAtomName"] is False
    # ...and a plainly-scalar syscall still is one.
    assert table["NtGdiAbortDoc"] is True


def test_crosscheck_hard_fails_on_a_stale_built_table(tmp_path):
    """A build dir predating a manual-list change must be a LOUD failure, not
    stale findings. Reported by the smp-bringup lane 2026-07-24: a stale
    artifact made the audit re-report 33 already-fixed sites, and they were
    re-blessed into the baseline as accepted debt."""
    gen = tmp_path / "t.gen.c"
    gen.write_text(
        '    { "NtFoo", wow64_thunk_NtFoo, 1 }, /* [0] */\n'
        '    { "NtBar", 0, 2 }, /* [1] MANUAL: listed in --manual-list */\n'
    )
    # agreeing table -> silent
    mg.crosscheck_artifact({"NtFoo": True, "NtBar": False}, gen, "win32k")
    # sources say NtFoo is marshalled, the built table still flat-thunks it
    try:
        mg.crosscheck_artifact({"NtFoo": False, "NtBar": False}, gen, "win32k")
    except RuntimeError as e:
        assert "STALE" in str(e) and "NtFoo" in str(e)
    else:
        raise AssertionError("stale built table did not raise")


def test_native_bridge_preserves_in_place_ordering_and_duplicate_identity(tmp_path):
    sc = w.Syscall(name="NtA", args=["long"], slots=1)
    ordered = w.order_by_sysfuncs([("NtA", 1), ("NtA", 2)], [sc])
    assert ordered[0] is sc and ordered[1] is sc
    assert sc.ordinal == 1 and sc.slots == 2 and sc.needs_manual
    assert "argcount" in sc.manual_reason
    unknown = w.Syscall(name="NtBad", args=["long", "banana"], slots=7)
    w._classify(unknown)
    assert unknown.slots == 7 and "banana" in unknown.manual_reason


def test_native_cli_failure_preserves_output_and_check_is_read_only(tmp_path):
    import subprocess
    from pathlib import Path

    tool = Path(__file__).resolve().parents[1] / "codegen"
    spec = tmp_path / "empty.spec"
    spec.write_text("@ stdcall RtlNotASyscall(long)\n")
    services = tmp_path / "sysfuncs.h"
    services.write_text("SVC_(A, 1)\n")
    output = tmp_path / "keep.c"
    output.write_text("keep")
    args = [
        str(tool),
        "wow64-thunks",
        "--arch",
        "amd64",
        "--spec",
        str(spec),
        "--sysfuncs",
        str(services),
        "--out-c",
        str(output),
    ]
    result = subprocess.run(args, capture_output=True, text=True, check=False, timeout=180)
    assert result.returncode != 0 and "no Nt* syscalls" in result.stderr
    assert output.read_text() == "keep"
    spec.write_text("@ stdcall NtA(long)\n")
    result = subprocess.run(
        [*args, "--check"], capture_output=True, text=True, check=False, timeout=180
    )
    assert result.returncode == 1 and "DRIFT" in result.stderr
    assert output.read_text() == "keep"


def test_shipping_cmake_tracks_native_generator_and_template_inputs(tmp_path):
    import json
    import shutil
    import subprocess
    import time
    from pathlib import Path

    repo = Path(__file__).resolve().parents[2]
    source = tmp_path / "source tree"
    source.mkdir()
    inputs = json.loads((repo / "tools/vibeos-codegen/tests/wow64/real-inputs.json").read_text())
    for name, text in inputs.items():
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    for name in [
        "sdk/cmake/codegen.cmake",
        "dll/win32/wow64/CMakeLists.txt",
        "rust-toolchain.toml",
        "tools/vibeos-codegen/Cargo.toml",
        "tools/vibeos-codegen/Cargo.lock",
    ]:
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repo / name, path)
    shutil.copytree(repo / "tools/vibeos-codegen/src", source / "tools/vibeos-codegen/src")
    # A host compiler tool may be shared; all generation inputs/outputs are isolated.
    binary = repo / ".cache/vibeos-codegen/target/debug/vibeos-codegen"
    assert binary.is_file()
    (source / "tools/codegen").symlink_to(binary)
    (source / "CMakeLists.txt").write_text(
        "cmake_minimum_required(VERSION 3.20)\nproject(ThunkFixture NONE)\n"
        'set(ARCH amd64)\nset(REACTOS_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}")\n'
        "foreach(cmd spec2def add_library target_include_directories set_module_type "
        "add_importlibs add_dependencies add_cd_file rust_staticlib_variant target_link_libraries)\n"
        "function(${cmd})\nendfunction()\nendforeach()\n"
        'include("${REACTOS_SOURCE_DIR}/dll/win32/wow64/CMakeLists.txt")\n'
        "add_custom_target(thunks ALL DEPENDS ${_wow64_thunks} ${_wow64_w32k_thunks})\n"
    )
    build = tmp_path / "build"

    def command(*args):
        result = subprocess.run(args, capture_output=True, text=True, check=False, timeout=180)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    command("cmake", "-S", str(source), "-B", str(build), "-G", "Ninja")
    first = command("ninja", "-C", str(build), "-v")
    assert first.count(" --spec ") == 2
    assert "no work to do" in command("ninja", "-C", str(build), "-v")
    for name in ["wow64_thunks.gen.c", "wow64_win32k_thunks.gen.c"]:
        assert (build / name).is_file() and "WOW64_THUNK_ENTRY" in (build / name).read_text()
    template = source / "tools/vibeos-codegen/src/wow64/header.txt"
    time.sleep(0.02)
    template.write_text(template.read_text() + "\n")
    rerun = command("ninja", "-C", str(build), "-v")
    assert rerun.count(" --spec ") == 2
    assert "no work to do" in command("ninja", "-C", str(build), "-v")
