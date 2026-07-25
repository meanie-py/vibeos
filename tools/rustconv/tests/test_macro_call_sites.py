"""A C macro is spelled `NAME(x)`; a Rust macro is spelled `NAME!(x)`.

The model reads the C correctly and emits a plain call. Which names are macros on the Rust
side is a fact about the crate's PRELUDE — invisible from the C — so it is not the model's to
know and not a thing to pay for. mbcs.c needed 65 of these across 5 names.
"""

from __future__ import annotations

from rustconv.crate_integrate import fix_macro_calls, prelude_macros

MACROS = {"MSVCRT_CHECK_PMT", "HIBYTE", "LOBYTE", "min"}


def test_a_plain_call_becomes_a_macro_invocation():
    out, fixed = fix_macro_calls("if MSVCRT_CHECK_PMT(x != 0) == 0 {}", MACROS, set())
    assert out == "if MSVCRT_CHECK_PMT!(x != 0) == 0 {}"
    assert fixed == ["MSVCRT_CHECK_PMT×1"]


def test_every_occurrence_is_fixed_and_counted():
    out, fixed = fix_macro_calls("HIBYTE(c) + HIBYTE(d) + LOBYTE(c)", MACROS, set())
    assert out == "HIBYTE!(c) + HIBYTE!(d) + LOBYTE!(c)"
    assert sorted(fixed) == ["HIBYTE×2", "LOBYTE×1"]


def test_already_correct_sites_are_untouched():
    """Idempotency: re-integrating a file must not produce `NAME!!(`."""
    src = "HIBYTE!(c)"
    out, fixed = fix_macro_calls(src, MACROS, set())
    assert out == src and not fixed


def test_running_twice_is_a_fixpoint():
    once, _ = fix_macro_calls("HIBYTE(c)", MACROS, set())
    twice, fixed = fix_macro_calls(once, MACROS, set())
    assert once == twice and not fixed


def test_whitespace_before_the_paren_is_handled():
    out, _ = fix_macro_calls("min (a, b)", MACROS, set())
    assert out == "min!(a, b)"


def test_a_negated_call_is_still_fixed():
    """`if !NAME(x)` is how C spells a boolean test — 17 of mbcs.c's 42 CHECK_PMT sites.

    An earlier lookbehind excluded a preceding `!` and silently skipped every one of them,
    which read as "the pass handled this file" while a third of it was untouched.
    """
    out, fixed = fix_macro_calls("if !MSVCRT_CHECK_PMT(x) {}", MACROS, set())
    assert out == "if !MSVCRT_CHECK_PMT!(x) {}"
    assert fixed == ["MSVCRT_CHECK_PMT×1"]


def test_a_longer_name_ending_in_the_macro_name_is_not_touched():
    src = "my_HIBYTE(c) + xHIBYTE(c)"
    out, fixed = fix_macro_calls(src, MACROS, set())
    assert out == src and not fixed


def test_a_name_this_file_defines_as_a_function_is_left_alone():
    """A local item shadows the prelude glob, so the plain call is already right."""
    src = "min(a, b)"
    out, fixed = fix_macro_calls(src, MACROS, {"min"})
    assert out == src and not fixed


def test_the_appended_original_c_block_is_never_rewritten():
    """That block is the 1:1 C reference the next translation diffs against.

    Rewriting it produced `if(!MSVCRT_CHECK_PMT!(dest != NULL))` — Rust-flavoured pseudo-C
    that is neither valid C nor the source it claims to be.
    """
    src = (
        "if MSVCRT_CHECK_PMT(x) == 0 {}\n"
        "\n// ORIGINAL C SOURCE\n"
        "//     if(!MSVCRT_CHECK_PMT(dest != NULL)) return EINVAL;\n"
        "// END ORIGINAL C SOURCE\n"
    )
    out, fixed = fix_macro_calls(src, MACROS, set())
    assert "if MSVCRT_CHECK_PMT!(x) == 0 {}" in out
    assert "//     if(!MSVCRT_CHECK_PMT(dest != NULL)) return EINVAL;" in out
    assert fixed == ["MSVCRT_CHECK_PMT×1"]  # the comment copy is not counted


def test_a_name_that_is_not_a_prelude_macro_is_left_alone():
    src = "strlen(s)"
    out, fixed = fix_macro_calls(src, MACROS, set())
    assert out == src and not fixed


def test_no_macros_is_a_no_op():
    src = "HIBYTE(c)"
    out, fixed = fix_macro_calls(src, set(), set())
    assert out == src and not fixed


def test_prelude_macros_reads_the_crate(tmp_path):
    src = tmp_path / "src"
    src.mkdir()
    (src / "prelude.rs").write_text(
        "#[macro_export]\nmacro_rules! DPRINT { ($($a:tt)*) => {{}}; }\n"
        "macro_rules! HIBYTE { ($w:expr) => { 0 }; }\n"
        "pub fn strlen() {}\n"
    )
    assert prelude_macros(tmp_path) == {"DPRINT", "HIBYTE"}


def test_prelude_macros_is_empty_without_a_prelude(tmp_path):
    assert prelude_macros(tmp_path) == set()
