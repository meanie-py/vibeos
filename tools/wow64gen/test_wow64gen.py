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
    # long -> slot 0 as ULONG; ptr -> slot 1 zero-extended; int64 -> slots 2,3
    assert "(ULONG)a[0]" in c
    assert "(PVOID)(ULONG_PTR)a[1]" in c
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
