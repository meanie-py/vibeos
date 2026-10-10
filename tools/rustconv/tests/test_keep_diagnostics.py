"""Plain C keeps ERR/WARN/ASSERT in the model's view, under its own bank namespace."""

from __future__ import annotations

from types import SimpleNamespace

from rustconv.__main__ import _cache_suffix
from rustconv.cpp_com import KEPT_DIAGNOSTICS_STRIPPED, strip_debug_statements

BODY = """NTSTATUS F(ULONG x)
{
    TRACE("enter %lu\\n", x);
    ERR("bad %lu\\n", x);
    WARN("odd\\n");
    ASSERT(x != 0);
    FIXME("later\\n");
    return 0;
}"""


def test_kept_view_strips_only_what_a_release_build_compiles_out():
    kept = strip_debug_statements(BODY, KEPT_DIAGNOSTICS_STRIPPED)
    assert 'ERR("bad %lu\\n", x);' in kept
    assert 'WARN("odd\\n");' in kept
    assert "ASSERT(x != 0);" in kept
    assert "TRACE" not in kept and "FIXME" not in kept


def test_the_default_view_is_unchanged_for_the_com_path():
    stripped = strip_debug_statements(BODY)
    assert "ERR" not in stripped and "ASSERT" not in stripped


def test_kept_diagnostics_get_their_own_bank_namespace():
    args = SimpleNamespace(cache_tag=None)
    old = _cache_suffix(args, True, True, True)
    new = _cache_suffix(args, True, True, True, True)
    assert old == "__factseal_v1__callseal_v1__wideseal_v1"
    assert new == old + "__diag_v1"


def test_a_cast_alias_of_a_function_like_macro_is_function_like():
    from rustconv.macro_authority import _function_like

    names = _function_like(
        "#define RTL_VERIFY(exp) ((exp) ? 1 : 0)\n#define ASSERT (void)RTL_VERIFY\n"
        "#define NOT_A_MACRO_ALIAS (void)ordinary_fn\n"
    )
    assert "ASSERT" in names
    assert "NOT_A_MACRO_ALIAS" not in names


def test_assert_written_as_a_call_becomes_the_macro():
    from rustconv.debug_macros import preserve_debug_macros

    out = preserve_debug_macros('unsafe fn f(x: u32) { ASSERT(x != 0); ERR(c"bad"); }')
    assert "ASSERT!(x != 0)" in out and "ERR!(" in out
