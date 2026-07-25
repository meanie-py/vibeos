"""A generated difftest that reports MATCH for the wrong reason is worse than no difftest.

Every test here guards a way the generator could produce a case that passes vacuously — the
failure mode that matters, because a green harness is what stops anyone looking again.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustconv.difftest_gen import (
    BUF_OUT,
    CHAR,
    LOCALE,
    SIZE,
    STR_IN,
    TIER_NLS,
    TIER_PURE,
    VOID,
    WSTR_IN,
    base_kinds,
    case_fn,
    classify_type,
    emit,
    emit_case,
    plan_file,
)
from rustconv.scanner import scan_file

REPO_ROOT = Path(__file__).resolve().parents[3]
MBCS = REPO_ROOT / "dll/win32/msvcrt/mbcs.c"


def _plan(tmp_path: Path, c_text: str):
    src = tmp_path / "m.c"
    src.write_text(c_text)
    return plan_file(scan_file(src), "m")


def _case(plan, name):
    return next(c for c in plan.cases if c.name == name)


# --------------------------------------------------------------------------- #
# kinds
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize(
    ("c_type", "kind"),
    [
        ("unsigned int", CHAR),
        ("int", CHAR),
        ("unsigned char", CHAR),
        ("size_t", SIZE),
        ("_locale_t", LOCALE),
        ("const unsigned char *", STR_IN),
        ("unsigned char *", BUF_OUT),
        ("const char *", STR_IN),
        ("const wchar_t *", WSTR_IN),
        ("void", VOID),
    ],
)
def test_kinds(c_type, kind):
    assert classify_type(c_type) == kind


def test_a_pointer_to_the_locale_typedef_is_not_a_locale():
    """`_locale_t` is already a pointer, so `_locale_t *` is a DOUBLE pointer — not ours."""
    assert classify_type("_locale_t *").startswith("?:")


def test_an_unknown_type_is_refused_not_guessed():
    assert classify_type("struct threadmbcinfo *").startswith("?:")


# --------------------------------------------------------------------------- #
# which argument a returned pointer is measured against
# --------------------------------------------------------------------------- #


def test_a_copy_function_measures_against_its_DESTINATION(tmp_path):
    """`_mbsnbcpy_l(dst, src, n, loc)` returns `dst`. Measuring from `src` compares offsets
    into two unrelated allocations and fails on correct code."""
    plan = _plan(
        tmp_path,
        "unsigned char* f(unsigned char *dst, const unsigned char *src, size_t n){ return dst; }\n",
    )
    assert "(dst_s.ptr(), dst_r.ptr())" in emit_case(_case(plan, "f"))


def test_a_search_function_measures_against_its_INPUT(tmp_path):
    """C's search family launders const: `strchr(const char*)` returns `char*` into it.

    A rule that matched constness strictly would refuse chr/rchr/str/pbrk — most of a
    string API.
    """
    plan = _plan(
        tmp_path,
        "unsigned char* f(const unsigned char *s, unsigned int c){ return (unsigned char*)s; }\n",
    )
    assert "(s_s.as_ptr(), s_r.as_ptr())" in emit_case(_case(plan, "f"))


def test_base_preference_puts_an_output_buffer_first():
    assert base_kinds(BUF_OUT).index(BUF_OUT) < base_kinds(BUF_OUT).index(STR_IN)


def test_a_scalar_return_has_no_base():
    assert base_kinds(CHAR) == ()


def test_a_returned_pointer_is_compared_by_OFFSET_never_by_address(tmp_path):
    """Subject and reference hold separate buffers, so raw pointers can never be equal —
    comparing them would 'pass' only when both sides return NULL."""
    plan = _plan(
        tmp_path, "unsigned char* f(const unsigned char *s) { return (unsigned char*)s; }\n"
    )
    body = emit_case(_case(plan, "f"))
    assert "as isize - sub_base as isize" in body
    assert "assert_eq!(got, want" not in body


# --------------------------------------------------------------------------- #
# refusals — the whole point is that these are LOUD, not silently covered
# --------------------------------------------------------------------------- #


def test_a_state_mutator_is_refused(tmp_path):
    """Running one inside a case replaces the pinned fixture, so every LATER case grades
    against different state — and reports the corruption as a divergence in an innocent
    function."""
    plan = _plan(
        tmp_path,
        "int create_mbcinfo(int cp) { return cp; }\n"
        "int f(unsigned int c) { return create_mbcinfo(c); }\n",
    )
    assert {r.function for r in plan.refusals} == {"create_mbcinfo", "f"}
    assert not plan.cases


def test_an_arch_conditional_body_is_refused(tmp_path):
    """The reference TU is compiled for the HOST, so a `_WIN64` arm the target takes is not
    the arm being graded."""
    plan = _plan(
        tmp_path,
        "int f(unsigned int c) {\n#if defined(_WIN64)\n  return 1;\n#else\n  return 2;\n#endif\n}\n",
    )
    assert [r.reason for r in plan.refusals] == ["arch-conditional body"]


def test_an_unsupported_parameter_is_refused_with_its_spelling(tmp_path):
    plan = _plan(tmp_path, "int f(mbstate_t *s) { return 0; }\n")
    assert plan.refusals and "mbstate_t" in plan.refusals[0].detail


def test_a_static_function_is_neither_covered_nor_refused(tmp_path):
    """Internal linkage: no symbol to rename, nothing a caller could grade."""
    plan = _plan(tmp_path, "static int helper(unsigned int c) { return c; }\n")
    assert not plan.cases and not plan.refusals and not plan.symbols


# --------------------------------------------------------------------------- #
# emitted shape
# --------------------------------------------------------------------------- #


def test_an_output_buffer_is_allocated_INSIDE_the_loops(tmp_path):
    """Hoisted out, it keeps the previous iteration's bytes, and a short write leaves stale
    content that masks the difference the comparison exists to find."""
    plan = _plan(tmp_path, "void f(unsigned char *dst, const unsigned char *src) { }\n")
    lines = emit_case(_case(plan, "f")).splitlines()
    alloc = next(i for i, ln in enumerate(lines) if "Guarded::pair()" in ln)
    loop = next(i for i, ln in enumerate(lines) if "corpus.pairs()" in ln)
    assert alloc > loop, "buffer must be allocated inside the corpus loop, not before it"


def test_the_context_string_is_built_only_on_FAILURE(tmp_path):
    """An exhaustive case runs 65,536 x len(CODEPAGES) iterations; a `format!` per iteration
    costs more than the calls being graded."""
    plan = _plan(tmp_path, "int f(unsigned int c) { return c; }\n")
    body = emit_case(_case(plan, "f"))
    assert "panic!(" in body
    assert "format!" not in body


def test_no_format_is_built_per_iteration_even_for_an_out_buffer(tmp_path):
    """The buffer comparison took an eagerly-formatted context string, which is the same
    per-iteration cost spelled differently — and slipped past the check above."""
    plan = _plan(tmp_path, "void f(unsigned char *dst, const unsigned char *src) { }\n")
    body = emit_case(_case(plan, "f"))
    assert "Guarded::same(" in body
    assert "format!" not in body


# --------------------------------------------------------------------------- #
# the emitted file must actually compile
# --------------------------------------------------------------------------- #


def test_a_case_is_not_named_after_the_symbol_it_calls(tmp_path):
    """The extern block declares `fn f` at module scope. A case also called `f` is a
    duplicate definition — and were it to compile, it would call ITSELF, grading nothing."""
    plan = _plan(tmp_path, "int f(unsigned int c) { return c; }\n")
    body = emit_case(_case(plan, "f"))
    assert "fn case_f()" in body
    assert "\nfn f()" not in body and "pub fn f()" not in body


def test_case_names_stay_distinct_when_the_c_names_differ_only_by_underscore():
    """`_mbslen` and `mbslen` must not both become `case_mbslen`."""
    assert case_fn("_mbslen") != case_fn("mbslen")


def test_every_case_appears_in_the_runner(tmp_path):
    """A case the runner does not call is dead code that reports nothing."""
    plan = _plan(
        tmp_path,
        "int f(unsigned int c) { return c; }\nint g(unsigned int c) { return c; }\n",
    )
    text = emit(plan, "m.c", 2)
    assert "pub fn run_all()" in text
    for c in plan.cases:
        assert f'("{c.name}", {case_fn(c.name)})' in text


def test_the_extern_block_is_closed_on_its_own_line(tmp_path):
    plan = _plan(tmp_path, "int f(unsigned int c) { return c; }\n")
    assert ";}" not in emit(plan, "m.c", 1)


def test_an_exhaustive_case_enumerates_the_whole_domain(tmp_path):
    plan = _plan(tmp_path, "int f(unsigned int c, _locale_t l) { return c; }\n")
    case = _case(plan, "f")
    assert case.exhaustive
    assert "for c in 0..=0xFFFFu32" in emit_case(case)


def test_a_case_taking_a_string_is_NOT_exhaustive(tmp_path):
    """You cannot enumerate a string domain; saying so keeps the summary honest."""
    plan = _plan(tmp_path, "int f(const unsigned char *s, unsigned int c) { return c; }\n")
    assert not _case(plan, "f").exhaustive


def test_no_corpus_is_constructed_when_nothing_needs_one(tmp_path):
    plan = _plan(tmp_path, "int f(unsigned int c) { return c; }\n")
    assert "Corpus::new()" not in emit_case(_case(plan, "f"))


# --------------------------------------------------------------------------- #
# against the real file
# --------------------------------------------------------------------------- #


@pytest.mark.skipif(not MBCS.is_file(), reason="msvcrt sources not present")
def test_mbcs_plan_is_stable():
    plan = plan_file(scan_file(MBCS), "mbcs")
    assert len(plan.symbols) == 175
    assert len(plan.cases) >= 150
    assert any(c.exhaustive and c.tier == TIER_PURE for c in plan.cases)
    assert any(c.tier == TIER_NLS for c in plan.cases)
    # `_setmbcp` rebuilds the very state every other case is pinned against.
    assert "_setmbcp" in {r.function for r in plan.refusals}
    assert "_setmbcp" not in {c.name for c in plan.cases}


@pytest.mark.skipif(not MBCS.is_file(), reason="msvcrt sources not present")
def test_every_emitted_case_has_balanced_braces():
    plan = plan_file(scan_file(MBCS), "mbcs")
    for case in plan.cases:
        body = emit_case(case)
        assert body.count("{") == body.count("}"), f"{case.name}: unbalanced"
