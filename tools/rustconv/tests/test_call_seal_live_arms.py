"""A `#if` arm the TU's own defines decide LIVE has its calls required by the call seal."""

from rustconv.call_seal import seal_calls
from rustconv.dead_regions import decided_live_lines

RUNDLL32 = """static void f(LPWSTR w, LPSTR a, int n)
{
#ifdef UNICODE
    WideCharToMultiByte(CP_ACP, 0, w, -1, a, n, NULL, NULL);
#else
    strncpy(a, w, n);
#endif
#if FOO
    maybe(a);
#endif
}
"""


def _dead(sealed) -> dict[str, bool]:
    return {c.original: c.dead for c in sealed.calls}


def test_without_defines_every_arm_stays_optional():
    sealed = seal_calls(RUNDLL32, {"WideCharToMultiByte", "strncpy", "maybe"})
    assert _dead(sealed) == {"WideCharToMultiByte": True, "strncpy": True, "maybe": True}


def test_the_tu_defines_make_the_live_arm_required():
    sealed = seal_calls(
        RUNDLL32, {"WideCharToMultiByte", "strncpy", "maybe"}, pinned={"UNICODE": 1}
    )
    # Live arm required; its #else dead, so optional; an undecidable `#if FOO` optional.
    assert _dead(sealed) == {"WideCharToMultiByte": False, "strncpy": True, "maybe": True}


def test_decided_live_lines_follow_c_rules():
    text = (
        "#define LOCAL\n#ifdef LOCAL\na();\n#endif\n"
        "#if DBG\nb();\n#elif X\nc();\n#else\nd();\n#endif\n"
        "#ifndef UNICODE\ne();\n#else\nf();\n#endif\n"
    )
    live = decided_live_lines(text, {"DBG": 1, "UNICODE": 1})
    lines = text.splitlines()
    got = sorted(lines[n - 1] for n in live)
    # `#elif` arms and what follows on that chain stay optional.
    assert got == ["a();", "b();", "f();"]


def test_audit_bounds_survive_an_ifdef_inside_the_body(tmp_path):
    """A brace count ran edit.c's EDIT_EM_ReplaceSel over 1,600 lines of its neighbours."""
    from rustconv.live_arm_audit import c_functions

    c = tmp_path / "x.c"
    c.write_text(
        "void a(int x)\n{\n    if (x) {\n#ifdef __REACTOS__\n        g(x);\n    } else {\n"
        "#endif\n        h(x);\n    }\n}\n"
        "static int b(void)\n{\n#ifdef __REACTOS__\n    return h(1);\n#endif\n    return 0;\n}\n"
    )
    got = c_functions(c)
    assert got["a"] == (1, 10) and got["b"] == (11, 17)
