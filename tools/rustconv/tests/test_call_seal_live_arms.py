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


BEEPMIDI = """#define CONTINUOUS_NOTES
#if FOO
#define MAYBE
#endif
#define GONE
#undef GONE

static void g(int n);

static void play(int n)
{
#ifdef CONTINUOUS_NOTES
    keep_going(n);
#endif
#ifdef MAYBE
    maybe(n);
#endif
#ifdef GONE
    gone(n);
#endif
#ifdef LATER
    later(n);
#endif
}
#define LATER
"""


def test_a_file_scope_define_above_the_function_makes_its_arm_required():
    """beepmidi: `#define CONTINUOUS_NOTES` is at file scope, and the seal saw only the body."""
    body = BEEPMIDI[BEEPMIDI.index("static void play") : BEEPMIDI.index("#define LATER")]
    names = {"keep_going", "maybe", "gone", "later"}
    sealed = seal_calls(body, names, tu_source=BEEPMIDI, pinned={})
    # Unconditional define above: required. Defined under an undecidable #if, #undef'd
    # again, or only defined after the function: optional, as C would have it or could.
    assert _dead(sealed) == {"keep_going": False, "maybe": True, "gone": True, "later": True}
    # Without pinned defines the seal stays in its old mode: every arm optional.
    assert all(_dead(seal_calls(body, names, tu_source=BEEPMIDI)).values())


def test_an_unlocatable_body_sees_no_file_scope_defines():
    """Two definitions of the name (or none) cannot place the body; nothing is assumed."""
    body = BEEPMIDI[BEEPMIDI.index("static void play") : BEEPMIDI.index("#define LATER")]
    tu = BEEPMIDI + "\nstatic void play(int n)\n{\n}\n"
    sealed = seal_calls(body, {"keep_going"}, tu_source=tu, pinned={})
    assert _dead(sealed) == {"keep_going": True}
