"""A macro that generates a `case` label must not cost us function bodies — or buy us junk.

`ERR_CASE(ERROR_ACCESS_DENIED)` sitting where a case label belongs is not a statement:
a bare `IDENT(ARG)` with no semicolon parses as nothing tree-sitter can place. The damage
is not local. On msvcrt's errno.c it produced ONE `ERROR` node spanning lines 132-451,
which swallowed `msvcrt_set_errno` and `_errno` whole while the walk recovered the
survivors from the wreckage. Nothing landed in `unextracted`, so the catalog came back
looking healthy — and `translate` planned to bill for 16 functions, missing the two that
matter most and including `ERR_CASE`, which is a MACRO.

That is money and a silent hole at once, so the expansion runs before the parse.
"""

from pathlib import Path

from rustconv.scanner import _expand_case_macros, scan_file


def _tu(arms: str) -> str:
    return (
        "void before(int x) { (void)x; }\n\n"
        "void set_it(int err)\n{\n  int *p = get_it();\n\n  switch(err)\n  {\n"
        "#define ERR_CASE(oserr) case oserr:\n"
        "#define ERR_MAPS(oserr, crterr) case oserr: *p = crterr; break\n"
        f"{arms}\n  default:\n    *p = EINVAL;\n  }}\n}}\n\n"
        "int after(void) { return 1; }\n"
    )


# EXACTLY two consecutive bare `ERR_CASE` arms, and no `ERR_MAPS` among them. That is not
# arbitrary — tree-sitter's error recovery here is erratic rather than monotonic, and this
# is the shape that actually loses the function: one arm recovers, four arms recover, and
# adding a single semicolon-terminated `ERR_MAPS` recovers. errno.c trips it because it
# opens with a run of eleven consecutive `ERR_CASE`. Change these two lines and the test
# can silently stop testing anything, which is why the ghost-function assertion below
# pins the symptom rather than the shape.
BREAKS_THE_PARSE = _tu("    ERR_CASE(ERROR_ACCESS_DENIED)\n    ERR_CASE(ERROR_LOCK_FAILED)")

ERRNO_SHAPED = _tu(
    "    ERR_CASE(ERROR_ACCESS_DENIED)\n"
    "    ERR_CASE(ERROR_LOCK_FAILED)\n"
    "    ERR_MAPS(ERROR_LOCK_VIOLATION,       EACCES);\n"
    "    ERR_MAPS(ERROR_PATH_NOT_FOUND,       ENOENT);"
)


def _names(tmp_path: Path, src: str) -> set[str]:
    p = tmp_path / "t.c"
    p.write_text(src)
    return {f.name for f in scan_file(p).functions}


def test_function_whose_body_hides_case_labels_in_macros_is_extracted(tmp_path):
    names = _names(tmp_path, BREAKS_THE_PARSE)
    assert names == {"before", "set_it", "after"}


def test_the_macro_itself_is_never_reported_as_a_function(tmp_path):
    """`ERR_CASE` in the catalog is a PAID call that buys a preprocessor directive."""
    assert not {"ERR_CASE", "ERR_MAPS"} & _names(tmp_path, BREAKS_THE_PARSE)


def test_expansion_produces_real_case_labels(tmp_path):
    out = _expand_case_macros(ERRNO_SHAPED)
    assert "case ERROR_ACCESS_DENIED:" in out
    assert "case ERROR_LOCK_VIOLATION: *p = EACCES; break;" in out
    # the directives are meaningless once expanded, and would re-break the parse
    assert "#define ERR_CASE" not in out
    assert "#define ERR_MAPS" not in out


def test_a_redefined_macro_expands_with_the_definition_in_force(tmp_path):
    """dinput's effect_linuxinput.c defines `FE(x)` twice, with DIFFERENT bodies.

    A name->body map silently expands the first group with the second body; only position
    disambiguates, which is why the expansion walks the text instead of substituting.
    """
    src = (
        "void f(int t, int w)\n{\n  switch (t) {\n"
        "#define FE(x) case x: type = #x; break\n"
        "    FE(FF_CONSTANT)\n    FE(FF_PERIODIC)\n  }\n"
        "  switch (w) {\n"
        "#define FE(x) case x: wave = #x; break\n"
        "    FE(FF_SQUARE)\n  }\n}\n"
    )
    out = _expand_case_macros(src)
    assert "case FF_CONSTANT: type = " in out
    assert "case FF_PERIODIC: type = " in out
    assert "case FF_SQUARE: wave = " in out
    assert "case FF_SQUARE: type = " not in out


def test_a_file_with_no_case_macro_is_byte_identical(tmp_path):
    """The pass must be inert for the overwhelming majority of TUs."""
    src = "#define MAX(a, b) ((a) > (b) ? (a) : (b))\nint f(int r){ return MAX(LIMIT, r); }\n"
    assert _expand_case_macros(src) == src


def test_expansion_never_moves_a_line():
    """Every normalizer in this pipeline is line-count-preserving, and it is load-bearing.

    `preamble_audit` reports the LINE of an unparsed region, so a pass that shifts lines
    makes that number point at innocent code. Multi-line (backslash-continued) bodies are
    the case that has to be handled deliberately: the body collapses to one logical line,
    and the directive is replaced by its own newlines rather than deleted.
    """
    multiline = (
        "void p(int r){ switch (r) {\n"
        "#define META_UNIMP(x) case x: \\\n"
        'FIXME("type "#x" not implemented.\\n"); \\\n'
        "break;\n"
        "    META_UNIMP(META_DRAWTEXT)\n"
        "    META_UNIMP(META_RESETDC)\n"
        "} }\n"
    )
    for src in (BREAKS_THE_PARSE, ERRNO_SHAPED, multiline):
        assert _expand_case_macros(src).count("\n") == src.count("\n")


def test_the_preamble_audit_and_the_catalog_agree(tmp_path):
    """They normalize independently; when they drift, the audit silently skips its checks."""
    from rustconv.preamble_audit import _scan

    p = tmp_path / "t.c"
    p.write_text(BREAKS_THE_PARSE)
    _, _, unparsed = _scan(p)
    assert unparsed == [], f"top-level ERROR at line(s) {unparsed} — audit would report INCOMPLETE"


def test_a_case_macro_in_a_dead_preproc_arm_is_never_expanded(tmp_path):
    """The pass runs AFTER dead-arm resolution, so `_MSVCR_VER >= 80` code is already gone."""
    src = (
        "void g(int e)\n{\n  switch (e) {\n"
        "#define G_CASE(x) case x:\n"
        "    G_CASE(REAL_ARM)\n"
        "  default: break;\n  }\n}\n"
        "#if 0\n#define DEAD_CASE(x) case x:\n"
        "void dead(int e){ switch(e){ DEAD_CASE(NEVER) default: break; } }\n#endif\n"
    )
    names = _names(tmp_path, src)
    assert "g" in names
    assert "dead" not in names
    assert "DEAD_CASE" not in names
