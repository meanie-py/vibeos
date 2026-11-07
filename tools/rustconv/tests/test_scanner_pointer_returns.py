"""A function whose RETURN TYPE is a pointer must still be extracted.

Both bugs here were found translating wldap32.h, a header of 62 `static inline`
marshalling helpers. Neither is loud: the scanner reports a dropped body as an
unextractable "C++ method" and the run continues, and the .spec cross-check that
would normally hard-fail cannot see a `static inline` helper because it is not an
export. The first pass translated 52 of 62 and said so in a warning.
"""

from pathlib import Path

from rustconv.scanner import scan_file


def _scan(tmp_path: Path, text: str):
    src = tmp_path / "widget.c"
    src.write_text(text)
    return scan_file(src)


def test_a_double_pointer_return_is_extracted(tmp_path: Path):
    """tree-sitter NESTS one pointer_declarator per star, so `char **f()` puts the
    function_declarator a GRANDchild deep. Scanning only direct children dropped
    every T** function — 10 of wldap32.h's 62."""
    scan = _scan(tmp_path, "char **dup2(char **in)\n{\n    return in;\n}\n")
    assert [f.name for f in scan.functions] == ["dup2"]
    assert scan.unextracted == []
    assert scan.functions[0].return_type == "char **"


def test_a_triple_pointer_return_is_extracted(tmp_path: Path):
    scan = _scan(tmp_path, "char ***deep(void)\n{\n    return 0;\n}\n")
    assert [f.name for f in scan.functions] == ["deep"]
    assert scan.functions[0].return_type == "char ***"


def test_a_single_pointer_return_still_works(tmp_path: Path):
    scan = _scan(tmp_path, "char *dup1(char *in)\n{\n    return in;\n}\n")
    assert scan.functions[0].return_type == "char *"


def test_an_elaborated_struct_return_keeps_its_TAG(tmp_path: Path):
    """`struct berval *bvdup(...)` came out with return type `*` — a pointer to
    nothing. A struct_specifier is neither a type_identifier nor an identifier, so
    the base type was silently dropped while the function still 'extracted'."""
    scan = _scan(tmp_path, "struct berval *bvdup(struct berval *bv)\n{\n    return bv;\n}\n")
    assert scan.functions[0].return_type == "struct berval *"


def test_an_elaborated_struct_double_pointer_return(tmp_path: Path):
    scan = _scan(tmp_path, "struct berval **bvarraydup(struct berval **bv)\n{\n    return bv;\n}\n")
    assert scan.functions[0].return_type == "struct berval **"


def test_an_enum_return_keeps_its_tag(tmp_path: Path):
    scan = _scan(tmp_path, "enum mode *pick(void)\n{\n    return 0;\n}\n")
    assert scan.functions[0].return_type == "enum mode *"


def test_declspec_hidden_does_not_break_the_declaration_after_it(tmp_path: Path):
    """`#define DECLSPEC_HIDDEN` expands to nothing, but tree-sitter reads it as a
    second type-identifier and fails the declaration it trails — which then makes it
    hoist the NEXT function's body to the root, where preamble_audit reports the
    locals as dropped file-scope state. wldap32.h:28 is the live case."""
    scan = _scan(
        tmp_path,
        "unsigned long map_error( int ) DECLSPEC_HIDDEN;\n"
        "int go(void)\n{\n    int local = 1;\n    return local;\n}\n",
    )
    assert [f.name for f in scan.functions] == ["go"]
    assert scan.unextracted == []


def test_declspec_hotpatch_does_not_hide_the_function(tmp_path: Path):
    """d3d8/d3d9: `IDirect3D8 * WINAPI DECLSPEC_HOTPATCH Direct3DCreate8(UINT)` parsed as
    no function, and translate refused the TU before payment."""
    scan = _scan(
        tmp_path,
        "IDirect3D8 * WINAPI DECLSPEC_HOTPATCH Direct3DCreate8(UINT sdk_version)\n"
        "{\n    return 0;\n}\n",
    )
    assert [f.name for f in scan.functions] == ["Direct3DCreate8"]
    assert "DECLSPEC_HOTPATCH" not in scan.functions[0].return_type


def test_a_printf_attribute_does_not_hide_the_rest_of_the_file(tmp_path: Path):
    """wined3d glsl_shader.c: `static void PRINTF_ATTR(4, 5) declare_in_varying(...)` lost every
    function from that line on (52 of 318 scanned)."""
    scan = _scan(
        tmp_path,
        "static void PRINTF_ATTR(4, 5) declare_in_varying(int a, const char *format, ...)\n"
        "{\n    return;\n}\n\n"
        "static void WINAPIV __WINE_PRINTF_ATTR(2,3) child(int c, const char *msg, ...)\n"
        "{\n    return;\n}\n\n"
        "static int after(void)\n{\n    return 1;\n}\n",
    )
    assert [f.name for f in scan.functions] == ["declare_in_varying", "child", "after"]
