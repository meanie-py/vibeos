"""pecheck reads a PE directly; these build tiny synthetic images so the checks that
gate every payload build are tested without a toolchain or build output."""

from __future__ import annotations

import importlib.util
import struct
from pathlib import Path

_spec = importlib.util.spec_from_file_location("pecheck", Path(__file__).with_name("pecheck.py"))
assert _spec and _spec.loader
pecheck = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pecheck)


def _pe(imports: dict[str, list[str]], *, subsystem: int = 2, wide: bool = False) -> bytes:
    """A minimal PE32/PE32+ image: one section at RVA 0x1000 holding the import table."""
    step, fmt = (8, "<Q") if wide else (4, "<I")
    sec = bytearray(0x800)
    nd = len(imports)
    cur = 20 * (nd + 1)  # descriptors, then thunks + names
    descs = []
    for dll, fns in imports.items():
        thunk = cur
        cur += step * (len(fns) + 1)
        name_rvas = []
        for fn in fns:
            name_rvas.append(0x1000 + cur)
            sec[cur + 2 : cur + 2 + len(fn)] = fn.encode()
            cur += 2 + len(fn) + 1
        for i, r in enumerate(name_rvas):
            struct.pack_into(fmt, sec, thunk + step * i, r)
        dll_rva = 0x1000 + cur
        sec[cur : cur + len(dll)] = dll.encode()
        cur += len(dll) + 1
        descs.append((0x1000 + thunk, dll_rva))
    for i, (thunk, name) in enumerate(descs):
        struct.pack_into("<IIIII", sec, 20 * i, thunk, 0, 0, name, thunk)
    optsize = 240 if wide else 224
    img = bytearray(0x200)
    img[:2] = b"MZ"
    struct.pack_into("<I", img, 0x3C, 0x40)
    img[0x40:0x44] = b"PE\0\0"
    struct.pack_into("<HHIIIHH", img, 0x44, 0x8664 if wide else 0x14C, 1, 0, 0, 0, optsize, 0)
    opt = 0x58
    struct.pack_into("<H", img, opt, 0x20B if wide else 0x10B)
    struct.pack_into("<H", img, opt + 68, subsystem)
    ddir = opt + (112 if wide else 96)
    struct.pack_into("<II", img, ddir + 8, 0x1000, 20 * (nd + 1))
    sh = opt + optsize
    img[sh : sh + 6] = b".idata"
    struct.pack_into("<IIII", img, sh + 8, len(sec), 0x1000, len(sec), 0x200)
    return bytes(img) + bytes(sec)


def _write(tmp_path: Path, **kw) -> Path:
    p = tmp_path / "x.exe"
    p.write_bytes(_pe(**kw))
    return p


def test_parse_reads_dlls_functions_and_subsystem_on_both_widths(tmp_path):
    for wide in (False, True):
        p = _write(tmp_path, imports={"ucrtbase.dll": ["strlen", "isalpha"]}, wide=wide)
        subsystem, imports = pecheck.parse(p)
        assert subsystem == 2
        assert imports == {"ucrtbase.dll": ["strlen", "isalpha"]}


def test_a_forbidden_import_fails_every_time(tmp_path, capsys):
    """The pipe version PASSED this case whenever grep's early exit won the race."""
    p = _write(tmp_path, imports={"api-ms-win-crt-string-l1-1-0.dll": ["strlen"]})
    for _ in range(20):
        assert pecheck.main([str(p), "--tag", "t", "--forbid-dll", "api-ms-win-crt"]) == 1
    assert "forbidden" in capsys.readouterr().out


def test_a_required_import_passes_every_time(tmp_path):
    """...and the pipe version FAILED this one at random (ucrtdiff sweep, shard 1/8)."""
    p = _write(tmp_path, imports={"UCRTBASE.dll": ["strlen"]})
    for _ in range(20):
        assert pecheck.main([str(p), "--tag", "t", "--require-dll", "ucrtbase.dll"]) == 0


def test_console_subsystem_is_refused_with_gui(tmp_path):
    p = _write(tmp_path, imports={"msvcrt.dll": ["printf"]}, subsystem=3)
    assert pecheck.main([str(p), "--tag", "t", "--gui"]) == 1


def test_graded_rows_must_be_imported_unless_compiled_out(tmp_path):
    p = _write(tmp_path, imports={"ucrtbase.dll": ["strlen"]})
    src = tmp_path / "h.c"
    src.write_text("int __cdecl ref_strlen(const char*);\nint __cdecl ref_isalpha(int);\n")
    refs = tmp_path / "refs.h"
    args = [str(p), "--tag", "t", "--graded-from", "ucrtbase.dll", "--graded-decls", str(src)]
    assert pecheck.main(args) == 1  # isalpha graded but not imported
    refs.write_text("#define UCRTDIFF_REF_HAS_isalpha 0\n")
    assert pecheck.main([*args, "--refs-header", str(refs)]) == 0


def test_a_compiled_out_block_exempts_its_helpers_too(tmp_path):
    # A family's runners and the non-graded ref_ helpers they use sit in one guarded
    # block; once the family's reference is gone the whole block is compiled out, and
    # the harness no longer imports the helper's DLL twin either.
    p = _write(tmp_path, imports={"ucrtbase.dll": ["strlen"]})
    src = tmp_path / "h.c"
    src.write_text(
        "int __cdecl ref_strlen(const char*);\n"
        "#if UCRTDIFF_REF_HAS_vsprintf\n"
        "int __cdecl ref_vsprintf(char*);\n"
        "FILE *__cdecl ref_fopen(const char*, const char*);\n"
        "#else\n"
        "int __cdecl ref_strlen(const char*);\n"
        "#endif\n"
    )
    refs = tmp_path / "refs.h"
    args = [str(p), "--tag", "t", "--graded-from", "ucrtbase.dll", "--graded-decls", str(src)]
    refs.write_text("#define UCRTDIFF_REF_HAS_vsprintf 1\n")
    assert pecheck.main([*args, "--refs-header", str(refs)]) == 1  # live: fopen demanded
    refs.write_text("#define UCRTDIFF_REF_HAS_vsprintf 0\n")
    assert pecheck.main([*args, "--refs-header", str(refs)]) == 0


def test_compound_and_continued_guards_are_evaluated():
    # The spawn block's guard is a four-way `||` continued over two lines; read as "some
    # other #if" it stayed live after every one of its files was taken over.
    src = (
        "#if UCRTDIFF_REF_HAS__spawnv || UCRTDIFF_REF_HAS__spawnvp || \\\n"
        "    UCRTDIFF_REF_HAS__spawnl\n"
        "int __cdecl ref__spawnv(int);\n"
        "#endif\n"
        "#if UCRTDIFF_REF_HAS__mbschr && UCRTDIFF_REF_CALLABLE__setmbcp\n"
        "int __cdecl ref__mbschr(int);\n"
        "#elif UCRTDIFF_REF_HAS__mbsrchr\n"
        "int __cdecl ref__mbsrchr(int);\n"
        "#else\n"
        "int __cdecl ref_neither(int);\n"
        "#endif\n"
        "#if defined(FOO) && UCRTDIFF_REF_HAS__spawnv\n"
        "int __cdecl ref_unreadable(int);\n"
        "#endif\n"
    )
    off = {"_spawnv", "_spawnvp", "_spawnl", "_mbschr"}
    kept = pecheck.compiled_in(src, off)
    assert "ref__spawnv" not in kept
    assert "ref__mbschr" not in kept
    assert "ref__mbsrchr" in kept and "ref_neither" not in kept
    assert "ref_unreadable" in kept  # not a pure REF guard: taken as live
    kept = pecheck.compiled_in(src, off | {"_mbsrchr"})
    assert "ref__mbsrchr" not in kept and "ref_neither" in kept
    kept = pecheck.compiled_in(src, {"_spawnvp"})
    assert "ref__spawnv" in kept and "ref__mbschr" in kept and "ref_neither" not in kept


def test_an_unreadable_image_fails_loud(tmp_path):
    p = tmp_path / "x.exe"
    p.write_bytes(b"not a pe")
    assert pecheck.main([str(p), "--tag", "t", "--gui"]) == 1


def test_guard_macros_and_flag_defines_are_expanded():
    # `#define LW_ANY (REF_HAS_a || ...)` and the `#if X / #define F 1 / #else /
    # #define F 0` flag idiom: both read as "some other #if" and kept compiled-out
    # blocks live, demanding imports for what they declared.
    src = (
        "#define LW_ANY (UCRTDIFF_REF_HAS__write || UCRTDIFF_REF_HAS__read)\n"
        "#if LW_ANY\n"
        "int __cdecl ref__write(int);\n"
        "#endif\n"
        "#if UCRTDIFF_REF_HAS__lseek\n"
        "#define LS_ON 1\n"
        "#else\n"
        "#define LS_ON 0\n"
        "#endif\n"
        "#if LS_ON\n"
        "int __cdecl ref__lseek(int);\n"
        "#endif\n"
    )
    kept = pecheck.compiled_in(src, {"_write", "_read", "_lseek"})
    assert "ref__write" not in kept and "ref__lseek" not in kept
    kept = pecheck.compiled_in(src, {"_write"})
    assert "ref__write" in kept and "ref__lseek" in kept


def test_a_harness_fallback_define_never_overrides_the_generated_zero():
    # The harness's `#ifndef X / #define X 1` fallbacks sit in a branch this pass
    # keeps; recording them as flags turned the generated header's 0 back into 1.
    src = (
        "#ifndef UCRTDIFF_REF_HAS__waccess\n"
        "#define UCRTDIFF_REF_HAS__waccess 1\n"
        "#endif\n"
        "#if UCRTDIFF_REF_HAS__waccess\n"
        "int __cdecl ref__waccess(int);\n"
        "#endif\n"
    )
    assert "ref__waccess" not in pecheck.compiled_in(src, {"_waccess"})


_SPEC = """\
@ stdcall RegOpenKeyExA(long str long long ptr)
@ stdcall -version=0x600+ RegDeleteTreeA(long str)
@ stdcall -version=0x502-0x600 OldOnly(long)
@ stub CancelOverlappedAccess
@ stdcall -stub HalfDone(long)
@ stdcall -arch=i386 X86Only(long)
@ stdcall -arch=!i386 NotX86(long)
@ stdcall -arch=win64 Wide(long)
@ cdecl -arch=i386 Twin(long) Twin32
@ cdecl -arch=win64 Twin(long) Twin64
123 stdcall -noname Ordinal(long)
"""


def test_spec_exports_follow_spec2def_version_arch_and_stub_rules(tmp_path):
    sp = tmp_path / "advapi32.spec"
    sp.write_text(_SPEC)
    e = pecheck.spec_exports(sp, "arm64", 0x502)
    assert e["RegOpenKeyExA"] == "ok"
    assert "0x600+" in e["RegDeleteTreeA"]
    assert e["OldOnly"] == "ok"
    assert e["CancelOverlappedAccess"] == "stub" and e["HalfDone"] == "stub"
    assert e["X86Only"].startswith("not exported") and e["NotX86"] == "ok"
    assert e["Wide"] == "ok" and e["Twin"] == "ok" and e["Ordinal"] == "ok"
    x86 = pecheck.spec_exports(sp, "i386", 0x502)
    assert x86["X86Only"] == "ok" and x86["NotX86"].startswith("not exported")
    assert x86["Wide"].startswith("not exported") and x86["Twin"] == "ok"
    assert pecheck.spec_exports(sp, "x86_64", 0x600)["RegDeleteTreeA"] == "ok"


def test_an_import_our_dll_does_not_export_fails_the_build(tmp_path, capsys, monkeypatch):
    repo = tmp_path / "repo"
    (repo / "sdk/cmake").mkdir(parents=True)
    (repo / "sdk/cmake/config.cmake").write_text('set(DLL_EXPORT_VERSION "0x502" CACHE STRING\n')
    (repo / "dll/win32/advapi32").mkdir(parents=True)
    (repo / "dll/win32/advapi32/advapi32.spec").write_text(_SPEC)
    monkeypatch.setattr(pecheck, "REPO", repo)
    good = _write(tmp_path, imports={"ADVAPI32.dll": ["RegOpenKeyExA"], "foreign.dll": ["Any"]})
    assert pecheck.main([str(good), "--tag", "t"]) == 0
    bad = tmp_path / "bad.exe"
    bad.write_bytes(_pe({"ADVAPI32.dll": ["RegOpenKeyExA", "RegDeleteTreeA"]}))
    assert pecheck.main([str(bad), "--tag", "t"]) == 1
    assert "ADVAPI32.dll!RegDeleteTreeA" in capsys.readouterr().out
    assert pecheck.main([str(bad), "--tag", "t", "--no-spec-check"]) == 0
