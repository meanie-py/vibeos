"""Controls for export_census — every case here is a wrong number this tool once printed.

The census reads a `.spec` and a build tree and says who backs each export. Nothing about it
crashes when it is wrong; it just reports a different total, confidently. So the tests are not
"does the parser work" but "does each specific miscount stay dead":

  * 22 real Rust exports filed as unresolved, because the backing symbol comes AFTER the
    argument list and a first-word regex stopped at the export name.
  * 27 i386-only exports counted against amd64, because `-arch=` was ignored.
  * one export whose backing symbol came out as the literal `()`, because the spec spells it
    `@ cdecl _endthread ()` with a space and the regex let the parenthesis fall into the
    trailing-symbol slot.
  * `extern`, the one directive whose export name and backing symbol genuinely differ — the
    blind spot that let `_mbctype` ship as two arrays.

`link_inputs` gets its own case for the same reason: msvcrt's `memmove` is defined in a
compiler-rt archive that appears in LINK_FLAGS and nowhere else, so a reader that only knows
LINK_LIBRARIES reports a live export as backed by nothing.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location("export_census", REPO / "tools" / "export_census.py")
assert _spec and _spec.loader
mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(mod)


def _parse(tmp_path: Path, text: str, arch: str = "amd64") -> dict:
    p = tmp_path / "x.spec"
    p.write_text(text)
    return mod.parse_spec(p, arch)


def test_backing_symbol_after_the_argument_list(tmp_path):
    got = _parse(tmp_path, "@ cdecl time(ptr) MSVCRT_time\n")
    assert got == {"time": ("cdecl", "MSVCRT_time")}


def test_extern_is_the_one_directive_whose_two_names_differ(tmp_path):
    got = _parse(tmp_path, "@ extern _mbctype MSVCRT_mbctype\n")
    assert got == {"_mbctype": ("extern", "MSVCRT_mbctype")}


def test_extern_without_a_second_name_backs_itself(tmp_path):
    """`@ extern _mbcasemap` — the export and the symbol are the same word, which is why the
    Rust twin needs `#[no_mangle]` under that exact name."""
    assert _parse(tmp_path, "@ extern _mbcasemap\n") == {"_mbcasemap": ("extern", "_mbcasemap")}


def test_a_space_before_the_arg_list_does_not_become_the_backing_symbol(tmp_path):
    got = _parse(tmp_path, "@ cdecl _endthread ()\n")
    assert got == {"_endthread": ("cdecl", "_endthread")}


def test_arch_filter_selects(tmp_path):
    text = "@ cdecl -arch=i386 _inp(long)\n@ cdecl -arch=x86_64 __C_specific_handler(ptr)\n"
    assert set(_parse(tmp_path, text, "i386")) == {"_inp"}
    assert set(_parse(tmp_path, text, "amd64")) == {"__C_specific_handler"}
    assert set(_parse(tmp_path, text, "arm64")) == set()


def test_win64_covers_amd64_and_arm64_but_not_i386(tmp_path):
    text = "@ cdecl -arch=win64 ??0bad_cast@@QEAA@AEBV0@@Z(ptr ptr) bad_cast_copy_ctor\n"
    assert len(_parse(tmp_path, text, "amd64")) == 1
    assert len(_parse(tmp_path, text, "arm64")) == 1
    assert len(_parse(tmp_path, text, "i386")) == 0


def test_an_unknown_arch_tag_is_refused_not_skipped(tmp_path):
    """Passing an unrecognised tag through would silently drop the line from every arch —
    an export that exists would vanish from the census with no way to notice."""
    with pytest.raises(SystemExit) as e:
        _parse(tmp_path, "@ cdecl -arch=riscv64 something()\n")
    assert "unknown -arch=" in str(e.value)


def test_forwarders_and_stubs_are_their_own_kinds(tmp_path):
    text = "@ cdecl __threadid() kernel32.GetCurrentThreadId\n@ stub _CxxThrow\n"
    got = _parse(tmp_path, text)
    assert got["__threadid"][0] == "forward"
    assert got["_CxxThrow"][0] == "stub"


def test_comments_and_blank_lines_are_not_exports(tmp_path):
    text = "# a comment\n\n@ cdecl real()\n@ extern _onexit # trailing note\n"
    assert set(_parse(tmp_path, text)) == {"real", "_onexit"}
    assert _parse(tmp_path, text)["_onexit"] == ("extern", "_onexit")


def test_link_inputs_reads_whole_archive_and_link_flags(tmp_path):
    """Order and grouping are the whole answer: `--whole-archive` is what makes the Rust
    cohab win, and an archive named only in LINK_FLAGS still defines real exports."""
    build = tmp_path
    for rel in ("rust.a", "own.a", "later.a", "builtins.a"):
        (build / rel).write_text("")
    (build / "build.ninja").write_text(
        "build dll/x.dll: LINKER file.obj\n"
        "  LINK_FLAGS = -nostdlib builtins.a -Wl,-entry,DllMain\n"
        "  LINK_LIBRARIES = -Wl,--whole-archive rust.a -Wl,--no-whole-archive own.a later.a\n"
        "build something/else: phony\n"
    )
    whole, lazy = mod.link_inputs(build, {"dll": "dll/x.dll"})
    assert [p.name for p in whole] == ["rust.a"]
    assert [p.name for p in lazy] == ["own.a", "later.a", "builtins.a"]


def test_module_objects_are_the_build_statements_inputs_not_a_glob(tmp_path):
    """A stale object left in the target's directory by a deleted source is not linked."""
    obj_dir = tmp_path / "dll/CMakeFiles/x.dir"
    obj_dir.mkdir(parents=True)
    (obj_dir / "live.c.obj").write_text("")
    (obj_dir / "deleted_shim.c.obj").write_text("")
    (tmp_path / "build.ninja").write_text(
        "build dll/x.dll: LINKER dll/CMakeFiles/x.dir/live.c.obj dll/x.res"
        " | lib/dep.a dll/CMakeFiles/x.dir/implicit.obj || order_only\n"
        "  LINK_LIBRARIES = lib/dep.a\n"
        "build something/else: phony\n"
    )
    objs = mod.module_objects(tmp_path, {"dll": "dll/x.dll"})
    assert [p.name for p in objs] == ["live.c.obj"]
