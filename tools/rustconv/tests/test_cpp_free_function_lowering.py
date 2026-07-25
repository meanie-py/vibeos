"""The free-function path translates C++, so it needs the C++ deterministic layer.

`--free-functions` routes a shell32 TU's classless functions through the ORDINARY
per-function contract — which is written for plain C. That was right for the COM
scaffold (no vtable, no `this`, no state layout) but it also dropped the lowerings
that are about C++ THE LANGUAGE rather than about COM:

  * `L"..."` / `L'x'` have no Rust spelling at all. Asked to translate one with no
    rule and no pre-lowering, the model emitted `u"DataObjectAttributes\\0"` — not
    Rust syntax. Measured on the Wave 4 cohort: 75 wide-literal sites across 259
    bodies, every one of them an invitation to invent something.
  * `IID_PPV_ARG(T, &out)` is a header macro expanding to TWO ABI arguments; emitted
    verbatim it is a call to a symbol that does not exist.

Both were already implemented and tested for the COM path. These tests pin that the
free-function path reaches the SAME code, and that the refactor which shared it did
not change COM behaviour.
"""

import re
from dataclasses import replace

import pytest
from rustconv.callgraph import FunctionDep
from rustconv.cpp_com import (
    normalize_cpp_idioms,
    render_wide_literal_statics,
    rewrite_wide_literals,
)
from rustconv.function_prompt import load_function_system_prompt


# --- deterministic wide-literal lowering ----------------------------------


def test_wide_string_literal_is_lowered_to_a_named_static():
    out = rewrite_wide_literals('DoThing(L"DataObjectAttributes");')
    assert 'L"' not in out
    assert re.search(r"CPP_COM_WSTR_[0-9A-F]{12}\.as_ptr\(\)", out)


def test_wide_char_literal_becomes_a_u16_constant():
    assert rewrite_wide_literals("if (c == L'\\\\')") == "if (c == 0x005Cu16)"


def test_lowering_is_content_addressed_so_equal_literals_share_one_static():
    a = rewrite_wide_literals('f(L"same");')
    b = rewrite_wide_literals('g(L"same");')
    name = re.search(r"CPP_COM_WSTR_[0-9A-F]{12}", a).group(0)
    assert name in b


def test_statics_are_rendered_for_the_literals_the_bodies_reference():
    raw = FunctionDep(name="f", body_text='void f(void) { g(L"hi"); }')
    lowered = replace(raw, body_text=rewrite_wide_literals(raw.body_text))
    statics = render_wide_literal_statics([raw], [lowered])
    name = re.search(r"CPP_COM_WSTR_[0-9A-F]{12}", lowered.body_text).group(0)
    # 'hi' is two UTF-16 units plus the NUL terminator.
    assert f"pub static {name}: [WCHAR; 3] = [0x0068, 0x0069, 0x0000];" in statics


def test_a_reference_with_no_source_literal_fails_loudly():
    """render_wide_literal_statics is what stops a dangling CPP_COM_WSTR_ reference
    from reaching the module; it must raise rather than emit nothing."""
    from rustconv.cpp_com import CppComError

    orphan = FunctionDep(name="f", body_text="g(CPP_COM_WSTR_AAAAAAAAAAAA.as_ptr());")
    with pytest.raises(CppComError, match="lost deterministic wide literal"):
        render_wide_literal_statics([], [orphan])


# --- IID_PPV_ARG, shared with the COM path --------------------------------


def test_iid_ppv_arg_expands_to_two_abi_arguments():
    out = normalize_cpp_idioms(
        "SHBindToObject(NULL, pidl, IID_PPV_ARG(IShellFolder, &spFolder));",
        "SHBindToObject(null_mut(), pidl, IID_PPV_ARG(IShellFolder, &mut spFolder));",
    )
    assert "IID_PPV_ARG" not in out
    assert "core::ptr::addr_of!(IID_IShellFolder)" in out
    assert "core::ptr::addr_of_mut!(spFolder).cast()" in out


def test_iid_ppv_arg_with_an_arbitrary_cast_expression():
    """The shape-specific rules all pin `&mut ident`. This spelling was produced by
    deepseek on shldataobject.cpp and slipped through every one of them."""
    out = normalize_cpp_idioms(
        "SHBindToObject(NULL, pidl, IID_PPV_ARG(IShellFolder, &spFolder));",
        "SHBindToObject(x, pidl, IID_PPV_ARG(IShellFolder, spFolder as *mut *mut IShellFolder));",
    )
    assert "IID_PPV_ARG" not in out
    assert "core::ptr::addr_of!(IID_IShellFolder)" in out
    assert "(spFolder as *mut *mut IShellFolder).cast()" in out


def test_nested_parens_do_not_split_the_arguments():
    out = normalize_cpp_idioms("", "f(IID_PPV_ARG(IBar, foo(a, b).cast()));")
    assert "IID_PPV_ARG" not in out
    assert "(foo(a, b).cast()).cast()" in out


def test_a_non_identifier_interface_is_left_alone():
    """Better to leave something untouched (and fail at compile) than to rewrite it
    into confidently wrong code."""
    src = "f(IID_PPV_ARG(1 + 2, &out));"
    assert normalize_cpp_idioms("", src) == src


def test_normalize_is_idempotent():
    """It runs on both the fresh-response and cache-hit paths; a second pass must
    not corrupt an already-normalized body."""
    src = "f(IID_PPV_ARG(IFoo, &out));"
    once = normalize_cpp_idioms(src, "f(IID_PPV_ARG(IFoo, &mut out));")
    assert normalize_cpp_idioms(src, once) == once


def test_com_worker_normalization_still_routes_through_the_shared_helper():
    """The extraction must not have changed COM behaviour — same input, same fix."""
    from rustconv.cpp_com import normalize_worker_semantics

    out = normalize_worker_semantics(
        "Bind(IID_PPV_ARG(IShellFolder, &m_Folder));",
        "Bind(IID_PPV_ARG(IShellFolder, &mut (*this).m_Folder));",
    )
    assert "IID_PPV_ARG" not in out
    assert "core::ptr::addr_of!(IID_IShellFolder)" in out


# --- fail loud on un-lowered body-local RAII ------------------------------


def test_unlowered_local_raii_refuses_before_payment():
    """Silence here is not neutral. With CDataObjectHIDA absent from the gate, a paid
    run reported success and emitted `CDataObjectHIDA::new(pDO)` — a Rust type that
    exists nowhere in the tree."""
    from rustconv.cpp_com import CppComError, require_supported_worker_raii

    dep = FunctionDep(
        name="SHELL_CloneDataObject",
        body_text="HRESULT f(IDataObject *pDO) { CDataObjectHIDA hida(pDO); return S_OK; }",
    )
    with pytest.raises(CppComError) as excinfo:
        require_supported_worker_raii([dep])
    message = str(excinfo.value)
    assert "before payment" in message
    assert "SHELL_CloneDataObject: CDataObjectHIDA local" in message
    assert "--local-raii" in message, "a refusal must name its own remedy"


def test_a_provided_lowering_lifts_the_refusal():
    from rustconv.cpp_com import require_supported_worker_raii

    dep = FunctionDep(name="f", body_text="{ CDataObjectHIDA hida(pDO); }")
    require_supported_worker_raii([dep], provided={"CDataObjectHIDA"})  # must not raise


# --- the prompt coda ------------------------------------------------------


def test_cpp_coda_is_absent_for_plain_c():
    plain = load_function_system_prompt(cpp_source=False)
    assert "The source is C++, not C" not in plain


def test_cpp_coda_is_appended_for_a_cpp_source():
    cpp = load_function_system_prompt(cpp_source=True)
    assert "The source is C++, not C" in cpp
    # The two constructs that actually broke, named explicitly.
    assert 'u"..."' in cpp
    assert "IID_PPV_ARG" in cpp
    assert "CComPtr" in cpp


def test_residual_iid_ppv_arg_trailing_comma_is_not_a_tuple():
    """A trailing comma inside the macro must not become a one-tuple.

    `(expr,)` is a tuple, not a parenthesised expression. It survives the rewrite and
    only fails later at `.cast()` with "no method named `cast` found for tuple" — seen
    for real in shell32 folders.cpp.
    """
    out = normalize_cpp_idioms(
        "", "CoCreateInstance(IID_PPV_ARG(IShellIconOverlayIdentifier, pOverlay,));"
    )
    assert ",)" not in out.replace("IID_IShellIconOverlayIdentifier), ", "")
    assert "core::ptr::addr_of!(IID_IShellIconOverlayIdentifier), (pOverlay).cast()" in out
