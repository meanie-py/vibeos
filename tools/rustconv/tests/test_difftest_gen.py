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
    INT_OUT,
    SIZE_OUT,
    UCHAR,
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
        ("unsigned char", UCHAR),
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


def test_a_locale_is_supplied_as_a_PAIR_not_shared(tmp_path):
    """A `_locale_t` carries `mbcinfo`. One pointer handed to both sides would aim them
    at the same fixture, so a function that wrongly mutates it would have the reference
    read the mutated state and agree — hiding the bug two slots exist to catch."""
    plan = _plan(tmp_path, "int f(unsigned int c, _locale_t l) { return c; }\n")
    body = emit_case(_case(plan, "f"))
    assert "LOCALES.pairs()" in body
    assert "f(c, l_s)" in body and "ref_f(c, l_r)" in body


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


def test_a_version_guarded_body_is_refused(tmp_path):
    """The reference TU compiles at _MSVCR_VER=0, so a body behind `#if _MSVCR_VER>=80`
    has no `ref_*` symbol at all and its case would fail to LINK.

    These tests scan a bare file with no compile database, which is the un-preprocessed
    mode where the directives survive and the function reaches the plan. On a real repo
    file the scanner preprocesses and drops it earlier; this is the backstop for when it
    cannot.
    """
    plan = _plan(
        tmp_path,
        "int a(unsigned int c) { return c; }\n"
        "#if _MSVCR_VER>=80\n"
        "int b(unsigned int c) { return c; }\n"
        "#endif\n"
        "int d(unsigned int c) { return c; }\n",
    )
    assert {c.name for c in plan.cases} == {"a", "d"}
    assert [r.function for r in plan.refusals] == ["b"]


def test_a_nested_if_does_not_end_the_version_region(tmp_path):
    """An ordinary `#if` inside the version block must not be mistaken for its `#endif`,
    or every function after it silently comes back as gradeable."""
    plan = _plan(
        tmp_path,
        "#if _MSVCR_VER>=80\n"
        "#if defined(SOMETHING)\n"
        "#endif\n"
        "int b(unsigned int c) { return c; }\n"
        "#endif\n"
        "int d(unsigned int c) { return c; }\n",
    )
    assert {c.name for c in plan.cases} == {"d"}
    assert [r.function for r in plan.refusals] == ["b"]


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


def test_an_in_place_mutator_gets_a_SEEDED_buffer(tmp_path):
    """`_mbslwr(s)` READS AND REWRITES its argument. Handed a blank buffer it folds an
    empty string, does nothing, returns the pointer, and both sides agree — the case
    runs, passes, and grades nothing at all."""
    plan = _plan(tmp_path, "unsigned char* f(unsigned char *s) { return s; }\n")
    body = emit_case(_case(plan, "f"))
    assert "Guarded::seeded(&seed)" in body
    assert "for seed in corpus.items()" in body
    assert "Guarded::pair()" not in body
    # The seed loop is synthesised, not derived from a parameter — so the check that
    # emits `let corpus = ...` has to look at the LOOPS, not at the parameter binds.
    assert "let corpus = Corpus::new();" in body


def test_a_destination_buffer_stays_BLANK(tmp_path):
    """The opposite case: `dst` in `f(dst, src, n)` is a sink, and seeding it would let
    leftover bytes stand in for a short write."""
    plan = _plan(
        tmp_path,
        "unsigned char* f(unsigned char *dst, const unsigned char *src, size_t n)"
        " { return dst; }\n",
    )
    body = emit_case(_case(plan, "f"))
    assert "Guarded::pair()" in body
    assert "seeded" not in body


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


def test_a_c_parameter_named_like_a_rust_keyword_is_escaped(tmp_path):
    """`_mbbtype_l(int c, int type, _locale_t)` is a legal C signature and a Rust syntax
    error. Generated code has no compiler in the loop until much later, so this must be
    handled at emission."""
    plan = _plan(tmp_path, "int f(unsigned int c, unsigned int type) { return c; }\n")
    body = emit_case(_case(plan, "f"))
    assert "type_" in body
    assert "for type in" not in body


def test_a_comparator_reaching_the_arch_forked_strcmp_is_graded_by_SIGN(tmp_path):
    """msvcrt's `strncmp` forks on `_WIN64`: the 64-bit arm clamps to -1/0/1, i386
    returns the raw byte difference. The harness's reference is built for the host
    (i386 arm) and the Rust for a 64-bit target (clamped), so magnitudes legitimately
    differ and only the sign is common ground."""
    plan = _plan(
        tmp_path,
        "int strncmp(const char*a, const char*b, size_t n) { return 0; }\n"
        "int f(const unsigned char *s, const unsigned char *t)"
        " { return strncmp((const char*)s, (const char*)t, 1); }\n",
    )
    body = emit_case(_case(plan, "f"))
    assert ".signum()" in body
    assert "if got != want" not in body


def test_a_non_comparator_is_still_graded_EXACTLY(tmp_path):
    plan = _plan(tmp_path, "int f(unsigned int c) { return c; }\n")
    body = emit_case(_case(plan, "f"))
    assert ".signum()" not in body
    assert "if got != want" in body


def test_a_cursor_parameter_points_INTO_its_base_string(tmp_path):
    """`_mbsdec_l(start, cur)` does pointer arithmetic between its two arguments. Two
    independent corpus buffers make that arithmetic meaningless — it reported
    "offset 18767 vs ref 122479", which is the distance between two allocations."""
    plan = _plan(
        tmp_path,
        "unsigned char* f(const unsigned char *start, const unsigned char *cur)"
        " { return (unsigned char*)cur; }\n",
    )
    body = emit_case(_case(plan, "f"))
    assert "start_s.offsets()" in body
    assert "start_s.as_ptr().add(cur_at)" in body
    assert "corpus.pairs()" in body  # `start` itself is still a real corpus string


def test_two_independent_strings_are_left_alone(tmp_path):
    """`_mbscmp(str, cmp)` really does take two unrelated strings — the cursor rule must
    not capture the far more common case."""
    plan = _plan(
        tmp_path,
        "int f(const unsigned char *str, const unsigned char *cmp) { return 0; }\n",
    )
    body = emit_case(_case(plan, "f"))
    assert "offsets()" not in body


# --- written-through result scalars (the `_s` family) ----------------------


def test_a_written_through_scalar_is_a_supplied_kind():
    """`_mbccpy_s(dst, n, &copied, src)` returns only an errno_t; `copied` IS the answer."""
    assert classify_type("int *") == INT_OUT
    assert classify_type("size_t *") == SIZE_OUT


def test_a_CONST_scalar_pointer_stays_a_refusal():
    """`const int *` is an INPUT array, not a result cell. Guessing which one a signature
    means is how a harness supplies a 1-element array to a function that wanted 40."""
    assert classify_type("const int *").startswith("?:")


def test_an_out_scalar_is_COMPARED_not_just_supplied(tmp_path):
    """Supplying the cell and never reading it back is the vacuous pass this kind exists to
    avoid: two EINVALs compare equal while the counts written disagree."""
    plan = _plan(tmp_path, "int f(const unsigned char *s, int *copied) { *copied = 1; return 0; }\n")
    body = emit_case(_case(plan, "f"))
    assert "copied_s != copied_r" in body, "the written value must be compared"
    assert "f(s_s.as_ptr(), &mut copied_s)" in body
    assert "ref_f(s_r.as_ptr(), &mut copied_r)" in body


def test_an_out_scalar_is_seeded_with_a_SENTINEL_not_zero(tmp_path):
    """Zero-initialising makes "did not write" and "wrote 0" indistinguishable — both sides
    would agree on a value neither produced."""
    plan = _plan(tmp_path, "int f(const unsigned char *s, size_t *n) { *n = 1; return 0; }\n")
    body = emit_case(_case(plan, "f"))
    assert "0x5A5A_5A5A_5A5A_5A5Ausize" in body
    assert "= (0, 0)" not in body


def test_each_side_gets_its_OWN_out_scalar(tmp_path):
    """One shared cell would let a function that writes through it feed the reference the
    subject's answer — the two-slot discipline the locale fixture already uses."""
    plan = _plan(tmp_path, "int f(int *n) { *n = 1; return 0; }\n")
    body = emit_case(_case(plan, "f"))
    assert "(mut n_s, mut n_r)" in body
    assert "&mut n_s" in body and "&mut n_r" in body


def test_an_out_scalar_cell_is_declared_INSIDE_the_loop(tmp_path):
    """Hoisted out, a skipped write inherits the previous iteration's value — which is
    exactly the bug (a missing write) the cell exists to detect."""
    plan = _plan(
        tmp_path,
        "int f(const unsigned char *s, int *n) { *n = 1; return 0; }\n",
    )
    body = emit_case(_case(plan, "f"))
    lines = body.splitlines()
    decl = next(i for i, ln in enumerate(lines) if "mut n_s" in ln)
    loop = next(i for i, ln in enumerate(lines) if "corpus.pairs()" in ln)
    assert decl > loop, "the cell must be declared inside the corpus loop, not above it"


def test_an_out_scalar_gets_the_right_extern_type(tmp_path):
    plan = _plan(tmp_path, "int f(int *a, size_t *b) { *a = 1; *b = 2; return 0; }\n")
    src = emit(plan, "m.c", total=1)
    assert "a: *mut i32" in src and "b: *mut usize" in src
