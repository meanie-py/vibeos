"""C++ default arguments make the exact-arity check reject a CORRECT Rust lowering.

`SHELL_GetAutoRunInfPath(PCWSTR DrvPath, PWSTR AriPath, BOOL ForInvoke = FALSE)` is called
with two arguments throughout CDrivesFolder.cpp. Rust has no default arguments, so a faithful
lowering must pass all three -- and the audit, which recorded the C CALL SITE's count, rejected
it as "wants 2, got 3". It rejected two different functions the same way in one run, which is
the documented tell that the fault is in the checker rather than the model.

The expected arity is therefore a RANGE for a callee with defaults: from the C site's count up
to the declaration's full parameter count. Callees without defaults keep exact matching.
"""

from __future__ import annotations

from rustconv.call_seal import _defaulted_param_count


def test_a_trailing_default_reports_the_full_parameter_count():
    src = "static UINT SHELL_GetAutoRunInfPath(PCWSTR DrvPath, PWSTR AriPath, BOOL ForInvoke = FALSE)\n{\n}\n"
    assert _defaulted_param_count(src, "SHELL_GetAutoRunInfPath") == 3


def test_no_default_means_no_range():
    src = "static UINT f(PCWSTR a, PWSTR b)\n{\n}\n"
    assert _defaulted_param_count(src, "f") is None


def test_equality_in_a_parameter_is_not_a_default():
    """`==` inside a default expression must not be read as another default marker."""
    src = "void g(int a, int b = (1 == 1))\n{\n}\n"
    assert _defaulted_param_count(src, "g") == 2


def test_a_nested_paren_does_not_split_parameters():
    src = "void h(int a, void (*cb)(int, int), int c = 0)\n{\n}\n"
    assert _defaulted_param_count(src, "h") == 3


def test_a_call_site_alone_is_not_a_declaration():
    """Only a parameter list can carry a default; `f(x, y)` at a call site has none."""
    src = "void caller(void) { f(x, y); }\n"
    assert _defaulted_param_count(src, "f") is None


def test_an_empty_parameter_list_is_not_a_default():
    src = "void k()\n{\n}\n"
    assert _defaulted_param_count(src, "k") is None
