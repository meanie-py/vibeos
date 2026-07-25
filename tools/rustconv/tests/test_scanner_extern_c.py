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

STUB = 'EXTERN_C {ty}*\nWINAPI\n{fn}(HWND hwnd)\n{{\n    return NULL;\n}}\n'


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

    Without a linkage specifier `FILEDESCRIPTOR* WINAPI Bar(...)` parses as an
    ambiguous declaration: ZERO function_definition nodes AND an EMPTY
    `unextracted` list — a silent drop. Pin that the expansion path is taken, by
    proving the no-macro form is genuinely the broken one.
    """
    p = tmp_path / "nomacro.cpp"
    p.write_text("FILEDESCRIPTOR*\nWINAPI\nBar(HWND hwnd)\n{\n    return NULL;\n}\n")
    result = scan_file(p)
    assert not result.functions and not result.unextracted, (
        "if tree-sitter learns this shape, _expand_extern_c may be simplifiable — "
        "until then the macro must be EXPANDED, never blanked"
    )
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
