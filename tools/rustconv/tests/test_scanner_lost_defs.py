"""scanner: a definition tree-sitter loses is recovered from the LIVE text (calc, 2026-10-10)."""

from pathlib import Path

from rustconv.scanner import scan_file


def test_a_header_split_across_a_decided_arm_is_recovered(tmp_path: Path):
    # calc's winmain.c: two headers for one body, the first under an arm that is false here.
    c = tmp_path / "winmain.c"
    c.write_text(
        "static int helper(void) { return 0; }\n"
        "#if defined(__GNUC__) && !defined(__REACTOS__)\n"
        "int WINAPI WinMain(HINSTANCE h, HINSTANCE p, LPSTR c, int n)\n"
        "#else\n"
        "int WINAPI _tWinMain(HINSTANCE h, HINSTANCE p, LPTSTR c, int n)\n"
        "#endif\n"
        "{\n    return helper();\n}\n"
    )
    names = {f.name for f in scan_file(c).functions}
    assert {"helper", "_tWinMain"} <= names and "WinMain" not in names


def test_a_statement_macro_without_its_semicolon_does_not_swallow_the_next_function(
    tmp_path: Path,
):
    # calc's theme.c: `GET_CB(x)` brings its own `;`, and Theme_Stop vanished after it.
    c = tmp_path / "theme.c"
    c.write_text(
        "#define GET_CB(n) p_##n = 0;\n"
        "void Theme_Start(void)\n{\n    GET_CB(OpenThemeData)\n    GET_CB(CloseThemeData)\n}\n\n"
        "void Theme_Stop(void)\n{\n    return;\n}\n"
    )
    names = {f.name for f in scan_file(c).functions}
    assert {"Theme_Start", "Theme_Stop"} <= names
