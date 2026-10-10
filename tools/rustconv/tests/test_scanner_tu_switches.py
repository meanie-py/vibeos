"""A TU's own feature switches decide its `#ifdef` arms (cmd.exe parser.c / for.c).

Two shapes broke the planner: a statement whose braces are split across `#ifdef` arms
(`if (..) {` opens in one, `}` closes in another), and `#ifdef/#else` arms declaring
different locals. Keeping both arms yields unparseable C and the function vanishes from
the plan. The file settles its own switches; anything it does not settle stays verbatim.
"""

from __future__ import annotations

from pathlib import Path

from rustconv.scanner import scan_file

SRC = """\
#define ON_SWITCH
// #define OFF_SWITCH
#define KEEP(x) \\
    do { (x); } while (0)

int pre(void) { return 0; }

int split(int x)
{
    if (x == 0)
    {
#ifdef OFF_SWITCH
        if (x > 1)
        {
#endif
            return 0;
#ifdef OFF_SWITCH
        }
        x++;
#endif
    }
    return x;
}

int arms(int x)
{
#ifdef ON_SWITCH
    int a = x; // declares different locals per arm
#else
    long a = x, b = 0;
    for (;;) {
#endif
#ifdef ON_SWITCH
    if (a < 1)
#else
    if ((a < 1) || (a > 31))
#endif
        return -1;
#ifndef ON_SWITCH
    }
#endif
    return a;
}

int undecided(int x)
{
#ifdef SOMEWHERE_ELSE
    if (x) {
#endif
    x++;
#ifdef SOMEWHERE_ELSE
    }
#endif
    return x;
}
"""


def _names(tmp_path: Path, extra: dict[str, str] | None = None) -> set[str]:
    c = tmp_path / "t.c"
    c.write_text(SRC)
    for name, text in (extra or {}).items():
        (tmp_path / name).write_text(text)
    return {f.name for f in scan_file(c).functions}


def test_split_braces_and_else_locals_are_planned(tmp_path):
    names = _names(tmp_path)
    assert {"pre", "split", "arms", "undecided"} <= names


def test_off_switch_defined_in_a_sibling_header_stays_undecided(tmp_path):
    # The header may turn OFF_SWITCH on, so the TU cannot call it off; the split `if {`
    # keeps both arms again (the pre-fix behaviour), but a decided switch must not move.
    names = _names(tmp_path, {"cfg.h": "#define OFF_SWITCH 1\n"})
    assert "arms" in names


def test_real_cmd_tus_plan_every_function():
    # for.c needs its file-scope `#define MSCMD_FOR_QUIRKS`; parser.c needs the commented-out
    # MSCMD_PARENS_PARSE_BUGS toggle read as OFF (it splits an `if {` across two arms).
    from rustconv.c_defs import c_functions

    root = Path(__file__).resolve().parents[3] / "base/shell/cmd"
    for name in ("for.c", "parser.c"):
        c = root / name
        if not c.is_file():
            continue
        assert c_functions(c.read_text(errors="replace")) <= {
            f.name for f in scan_file(c).functions
        }
