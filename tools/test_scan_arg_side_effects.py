"""Controls for scan_arg_side_effects.

The lint's healthy output is a small number, which is indistinguishable from the output of a
matcher that has quietly stopped matching — so the positive control is the REAL msvcrt bug,
spelled exactly as undname.c spells it, across four source lines. If that stops firing the
scanner is dead.

The negative controls are the shapes that must stay silent, and each one is a shape that made
an earlier draft of this scanner useless: `for (p = s; *p; p++)` (45 findings), prose in a
comment (53 more), and a macro whose unbalanced parentheses glued the rest of the file into
one "expression" (80 more, all attributed to a single line).
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "scan_arg_side_effects", REPO / "tools" / "scan_arg_side_effects.py"
)
assert _spec and _spec.loader
mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(mod)


def scan(tmp_path: Path, src: str):
    p = tmp_path / "t.c"
    p.write_text(src)
    return mod.scan_file(p)


# ── positive: the bug that shipped ───────────────────────────────────────────────────────

THE_BUG = """\
static BOOL get_function_signature(struct parsed_symbol* sym, struct function_signature* fs)
{
    if (!get_calling_convention(*sym->current++,
                                &fs->call_conv, &fs->exported,
                                sym->flags & ~UNDNAME_NO_ALLOCATION_LANGUAGE) ||
        !demangle_datatype(sym, &fs->return_ct, 0))
        return FALSE;
    return TRUE;
}
"""


def test_the_shipped_bug_is_found(tmp_path):
    hits = scan(tmp_path, THE_BUG)
    assert len(hits) == 1, hits
    lineno, obj, callee, _ = hits[0]
    assert (lineno, obj, callee) == (3, "sym->current", "get_calling_convention")


def test_it_is_found_across_LINES_not_just_within_one(tmp_path):
    """The increment is on line 3 and the operand that observes it on line 6. A per-line
    scan finds nothing here, which is exactly the case worth finding."""
    one_line = THE_BUG.replace("\n                                ", " ").replace(
        "\n        !demangle", " !demangle"
    )
    assert len(scan(tmp_path, one_line)) == 1
    assert len(scan(tmp_path, THE_BUG)) == 1


def test_a_later_ARGUMENT_counts_as_the_observer(tmp_path):
    src = "void f(void) { g(*p++, p, 0); }\n"
    hits = scan(tmp_path, src)
    assert len(hits) == 1 and hits[0][2] == "g"


# ── negative: shapes C already sequences, or that are not code at all ─────────────────────


def test_a_for_loop_step_clause_is_not_a_finding(tmp_path):
    src = "void f(void) { for (p = inpath; *p; p++) if (*p == '.') end = p; }\n"
    assert scan(tmp_path, src) == []


def test_an_increment_directly_in_a_condition_is_not_a_finding(tmp_path):
    src = "void f(void) { if (++a->num >= a->max) a->max = a->num; }\n"
    assert scan(tmp_path, src) == []


def test_prose_in_a_comment_is_not_code(tmp_path):
    src = """\
/* Round the size++ up so the space(size) it needs is available, then reuse size. */
void f(void) { return; }
"""
    assert scan(tmp_path, src) == []


def test_a_string_literal_is_not_code(tmp_path):
    src = 'void f(void) { log("g(*p++, p)"); }\n'
    assert scan(tmp_path, src) == []


def test_an_unbalanced_macro_does_not_glue_the_whole_file(tmp_path):
    """One `#define` with unmatched parens used to make every later line part of the same
    'expression', so a single innocent site absorbed 80 findings and reported them all
    against the macro's line number."""
    src = "#define ODD(x) ((x\nvoid f(void) { g(*p++, p, 0); }\n" + "void pad(void) { }\n" * 40
    hits = scan(tmp_path, src)
    assert len(hits) == 1 and hits[0][0] == 2, hits


def test_the_increment_must_be_OBSERVED_to_be_a_finding(tmp_path):
    """`f(*p++)` alone is fine: nothing else in the expression reads `p`, so wherever the
    translation puts the increment the result is the same."""
    assert scan(tmp_path, "void f(void) { g(*p++); }\n") == []
