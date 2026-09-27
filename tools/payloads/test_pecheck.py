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


def test_an_unreadable_image_fails_loud(tmp_path):
    p = tmp_path / "x.exe"
    p.write_bytes(b"not a pe")
    assert pecheck.main([str(p), "--tag", "t", "--gui"]) == 1
