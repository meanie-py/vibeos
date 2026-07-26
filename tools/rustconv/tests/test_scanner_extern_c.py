"""EXTERN_C must not cost us function bodies.

`EXTERN_C` is winnt.h's `#define EXTERN_C extern "C"` (C++) / `extern` (C).
tree-sitter knows neither spelling of the macro, so before it is expanded the
function's extraction depends on tree-sitter ERROR RECOVERY — whose repair cost is
sensitive to token LENGTH. With a pointer return type the function survives while
the type name is <=9 characters and is lost at >=10, which is why shell32's
stubs.cpp reported exactly 2 of 66 bodies unextractable.
"""

from pathlib import Path

from rustconv.scanner import scan_file

STUB = "EXTERN_C {ty}*\nWINAPI\n{fn}(HWND hwnd)\n{{\n    return NULL;\n}}\n"


def _names(tmp_path: Path, src: str, suffix: str = ".cpp") -> set[str]:
    p = tmp_path / f"t{suffix}"
    p.write_text(src)
    return {f.name for f in scan_file(p).functions}


def test_extern_c_pointer_return_extracts_regardless_of_type_name_length(tmp_path):
    """The length cliff is gone: a 9- and a 20-character type behave identically."""
    for ty in ("SHORTTYPE", "AVeryLongInterfaceName" * 2):
        src = STUB.format(ty=ty, fn="Bar")
        assert "Bar" in _names(tmp_path, src), f"lost the body for return type {ty!r}"


def test_extern_c_real_shell32_stub_shapes(tmp_path):
    """The two bodies stubs.cpp actually lost, verbatim in shape."""
    src = (
        STUB.format(ty="IShellFolderViewCB", fn="SHGetShellFolderViewCB")
        + STUB.format(ty="FILEDESCRIPTOR", fn="GetFileDescriptor")
        + "EXTERN_C VOID\nWINAPI\nCheckDiskSpace(VOID)\n{\n    return;\n}\n"
    )
    assert _names(tmp_path, src) == {
        "SHGetShellFolderViewCB",
        "GetFileDescriptor",
        "CheckDiskSpace",
    }


def test_extern_c_is_expanded_not_deleted(tmp_path):
    """Deleting the macro looks equivalent and is not.

    This test used to assert the OPPOSITE of the first case below: that
    `FILEDESCRIPTOR* WINAPI Bar(...)` was a silent drop (zero
    function_definition nodes AND an empty `unextracted`), which proved the
    expansion path had to be taken. It was a deliberate canary — "if tree-sitter
    learns this shape, _expand_extern_c may be simplifiable".

    It has now fired, for a reason on our side rather than tree-sitter's:
    `_normalize_callbacks` drops a convention keyword sitting after a pointer
    return type, and that rule was extended to cover the aliases it had been
    leaving verbatim (CDECL/WINAPI/NTAPI/...). Stripping `WINAPI` is what makes
    this shape parse now — see test_scanner_cc_after_ptr.

    `_expand_extern_c` is still required, for a reason that was never about this
    shape: EXTERN_C is a LINKAGE specifier (`extern "C"` in C++), so deleting it
    changes name mangling rather than just parsing, and the macro token itself is
    still unknown to tree-sitter — which is what the length-cliff tests above
    cover.
    """
    p = tmp_path / "nomacro.cpp"
    p.write_text("FILEDESCRIPTOR*\nWINAPI\nBar(HWND hwnd)\n{\n    return NULL;\n}\n")
    result = scan_file(p)
    assert {f.name for f in result.functions} == {"Bar"}, (
        "the after-pointer convention strip should make this shape parse; if it "
        "does not, _normalize_callbacks' alternation has regressed"
    )
    # The macro path must keep working — that is the invariant this file exists for.
    assert "Bar" in _names(tmp_path, STUB.format(ty="FILEDESCRIPTOR", fn="Bar"))


def test_extern_c_line_numbers_are_preserved(tmp_path):
    """The substitution stays on one line, so reported line numbers still match
    the real source (the archive and every diagnostic depend on it)."""
    src = "// leading comment\n\n" + STUB.format(ty="FILEDESCRIPTOR", fn="Bar")
    p = tmp_path / "lines.cpp"
    p.write_text(src)
    fn = next(f for f in scan_file(p).functions if f.name == "Bar")
    assert fn.line_start == 3, f"definition starts on source line 3, got {fn.line_start}"


def test_extern_c_in_a_c_file_uses_the_c_expansion(tmp_path):
    """`extern "C"` is not valid C; a .c file must get the plain `extern`."""
    src = "EXTERN_C int\nWINAPI\nBar(int x)\n{\n    return x;\n}\n"
    assert "Bar" in _names(tmp_path, src, suffix=".c")
