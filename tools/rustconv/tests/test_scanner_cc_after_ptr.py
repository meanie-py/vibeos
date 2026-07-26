"""A calling-convention keyword between a POINTER return type and the name.

tree-sitter-c cannot parse that position in ANY spelling — it produces no
`function_definition` node at all — so `_normalize_callbacks` drops the keyword
before parsing.  The alternation it uses must cover EVERY convention alias,
including the ones rule 2 deliberately leaves verbatim.

That exemption list once held `CDECL`, which is Wine's dominant spelling:
`FILE * CDECL fopen(...)`.  The result was that `dll/win32/msvcrt/file.c`
scanned as 219 functions instead of 234 — `fopen`, `freopen`, `tmpfile`,
`_fdopen`, `__iob_func` and ten more were simply absent from the translation
plan.  A dropped function is indistinguishable from one that was never there,
and for file.c it was worse than a partial port: every entry point shares one
fd table, so a C `fopen` beside a Rust `fclose` is split state.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustconv.scanner import scan_file

REPO = Path(__file__).resolve().parents[3]
MSVCRT = REPO / "dll/win32/msvcrt"


@pytest.mark.parametrize("conv", ["CDECL", "WINAPI", "NTAPI", "FASTCALL", "__cdecl", "__stdcall"])
def test_pointer_return_with_convention_is_scanned(tmp_path: Path, conv: str) -> None:
    """The middle function must survive, whichever alias sits after the `*`."""
    src = tmp_path / "cc.c"
    src.write_text(
        f"int {conv} alpha(int x) {{ return x; }}\n"
        f"char * {conv} beta(void) {{ return 0; }}\n"
        f"int {conv} gamma(int y) {{ return y; }}\n"
    )
    names = {f.name for f in scan_file(src).functions}
    assert names == {"alpha", "beta", "gamma"}, (
        f"`char * {conv} beta` was dropped — tree-sitter cannot parse a convention "
        f"keyword after a pointer return type, so _normalize_callbacks must strip it"
    )


@pytest.mark.skipif(not (MSVCRT / "file.c").is_file(), reason="msvcrt source not present")
def test_msvcrt_file_c_exports_are_all_scanned() -> None:
    """The regression that motivated this: file.c's `FILE * CDECL` entry points.

    Asserted by NAME rather than by count, so the test keeps its meaning when
    someone edits file.c.
    """
    names = {f.name for f in scan_file(MSVCRT / "file.c").functions}
    dropped = {
        "fopen",
        "freopen",
        "tmpfile",
        "_fdopen",
        "_wfdopen",
        "__iob_func",
        "_wfopen",
        "_fsopen",
        "_wfsopen",
        "_wfreopen",
        "_getws",
        "fgetws",
        "_wmktemp",
        "_wtempnam",
        "_wtmpnam",
    } - names
    assert not dropped, f"file.c functions missing from the scan: {sorted(dropped)}"
