"""The receiver a C++ member-call site never writes down, in its two remaining disguises.

`receiver_callees` already taught the arity check that a SIBLING method's Rust lowering
carries an explicit leading `this`. Two more call shapes hide a receiver the same way, and
both rejected correct CDefView output that no retry could ever have fixed:

* A sibling method is sealed under its LOWERED name, so the C++ default-argument tolerance
  looked the declaration up as `CDefView_LoadColumn_body` -- a spelling the source never
  contains -- found nothing, and demanded the C site's exact count.
* An ATL `CWindow` method lowers to the Win32 API of the same name with the window handle
  prepended, so `DestroyWindow()` must become `DestroyWindow(this.m_hWnd)`; the check counted
  the C++ site's zero arguments and rejected the one correct answer.
"""

from __future__ import annotations

from rustconv.call_seal import seal_calls
from rustconv.cpp_core import cwindow_method_arity, derives_from_cwindow

ATLWIN = """\
class CWindow
{
public:
    HWND m_hWnd;

    BOOL DestroyWindow()
    {
        return ::DestroyWindow(m_hWnd);
    }

    BOOL ScreenToClient(LPPOINT lpPoint) const
    {
        return ::ScreenToClient(m_hWnd, lpPoint);
    }

    BOOL EnableWindow(BOOL bEnable = TRUE)
    {
        return ::EnableWindow(m_hWnd, bEnable);
    }
};
"""


def _only(sealed, callee):
    return next(call for call in sealed.calls if call.original == callee)


def test_a_lowered_sibling_name_still_finds_its_default_argument_declaration():
    tu = (
        "class CDefView : public CWindowImpl<CDefView, CWindow, CControlWinTraits>\n"
        "{\n"
        "    HRESULT LoadColumn(UINT FoldCol, UINT ListCol, BOOL Insert, UINT ForceWidth = 0);\n"
        "};\n"
    )
    body = "void f(void)\n{\n    CDefView_LoadColumn_body(a, b, TRUE);\n}\n"
    sealed = seal_calls(
        body,
        {"CDefView_LoadColumn_body"},
        tu_source=tu,
        receiver_callees=frozenset({"CDefView_LoadColumn_body"}),
        callee_cpp_names={"CDefView_LoadColumn_body": "LoadColumn"},
    )
    call = _only(sealed, "CDefView_LoadColumn_body")
    # three at the C site + the receiver, up to four declared + the receiver.
    assert (call.argc, call.argc_max) == (4, 5)


def test_without_the_cpp_name_the_declaration_is_unreachable():
    """The bug itself: the lookup by lowered name finds nothing and demands exactness."""
    tu = "class CDefView\n{\n    HRESULT FillList(BOOL IsRefreshCommand = TRUE);\n};\n"
    body = "void f(void)\n{\n    CDefView_FillList_body();\n}\n"
    sealed = seal_calls(
        body,
        {"CDefView_FillList_body"},
        tu_source=tu,
        receiver_callees=frozenset({"CDefView_FillList_body"}),
    )
    assert _only(sealed, "CDefView_FillList_body").argc_max is None


def test_a_cwindow_method_expects_the_window_handle():
    body = "void f(void)\n{\n    DestroyWindow();\n}\n"
    sealed = seal_calls(
        body,
        {"DestroyWindow"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    call = _only(sealed, "DestroyWindow")
    assert (call.argc, call.argc_max) == (1, 1)


def test_a_cwindow_method_with_an_explicit_receiver_agrees():
    body = "void f(void)\n{\n    m_ListView.ScreenToClient(&pt);\n}\n"
    sealed = seal_calls(
        body,
        {"ScreenToClient"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    assert _only(sealed, "ScreenToClient").argc == 2


def test_a_cwindow_default_argument_widens_the_range_too():
    body = "void f(void)\n{\n    EnableWindow();\n}\n"
    sealed = seal_calls(
        body,
        {"EnableWindow"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    call = _only(sealed, "EnableWindow")
    assert (call.argc, call.argc_max) == (1, 2)


def test_a_class_that_is_not_a_window_gets_no_handle():
    """The adjustment is keyed to the derivation, so a plain class keeps exact matching."""
    body = "void f(void)\n{\n    DestroyWindow(hwnd);\n}\n"
    sealed = seal_calls(body, {"DestroyWindow"})
    assert _only(sealed, "DestroyWindow").argc == 1


def test_the_arity_table_is_read_from_the_header_not_remembered():
    arity = cwindow_method_arity(ATLWIN)
    assert arity["DestroyWindow"] == 0
    assert arity["ScreenToClient"] == 1
    assert arity["EnableWindow"] == 1


def test_derivation_is_what_makes_a_bare_call_a_member():
    source = (
        "class CDefView :\n"
        "    public CWindowImpl<CDefView, CWindow, CControlWinTraits>,\n"
        "    public IShellView3\n"
        "{\n};\n"
    )
    assert derives_from_cwindow(source, "CDefView")
    assert not derives_from_cwindow("class CPlain : public IUnknown\n{\n};\n", "CPlain")


def test_method_syntax_on_a_window_handle_is_repaired_into_a_call():
    """`m_ListView.DestroyWindow()` is not a Rust method call; the handle is argument zero."""
    from rustconv.call_seal import repair_window_receivers

    body = "void f(void)\n{\n    m_ListView.DestroyWindow();\n}\n"
    sealed = seal_calls(
        body,
        {"DestroyWindow"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    token = _only(sealed, "DestroyWindow").placeholder
    rust, repaired = repair_window_receivers(f"(*(*this).m_ListView).{token}();", sealed)
    assert rust == f"{token}((*this).m_ListView);"
    assert repaired == [f"{token}=DestroyWindow"]


def test_a_correct_window_call_is_left_alone():
    from rustconv.call_seal import repair_window_receivers

    body = "void f(void)\n{\n    DestroyWindow();\n}\n"
    sealed = seal_calls(
        body,
        {"DestroyWindow"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    token = _only(sealed, "DestroyWindow").placeholder
    rust, repaired = repair_window_receivers(f"{token}((*this).m_hWnd);", sealed)
    assert rust == f"{token}((*this).m_hWnd);"
    assert repaired == []


def test_a_receiver_with_arguments_keeps_them_after_the_handle():
    from rustconv.call_seal import repair_window_receivers

    body = "void f(void)\n{\n    m_ListView.ScreenToClient(&pt);\n}\n"
    sealed = seal_calls(
        body,
        {"ScreenToClient"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    token = _only(sealed, "ScreenToClient").placeholder
    rust, _ = repair_window_receivers(f"(*(*this).m_ListView).{token}(&mut pt);", sealed)
    assert rust == f"{token}((*this).m_ListView, &mut pt);"


def test_a_qualified_call_is_the_win32_api_and_keeps_its_arity():
    """`::GetWindowRect(hWndHdr, &r)` already passes the handle; adding one inverts the fix."""
    body = "void f(void)\n{\n    ::ScreenToClient(hWndHdr, &pt);\n}\n"
    sealed = seal_calls(
        body,
        {"ScreenToClient"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    call = _only(sealed, "ScreenToClient")
    assert call.argc == 2
    assert not call.hwnd_receiver


def test_the_same_name_bare_is_still_the_member():
    """Both spellings occur in one file, so the discrimination has to be per SITE."""
    body = "void f(void)\n{\n    ScreenToClient(&pt);\n    ::ScreenToClient(hWnd, &other);\n}\n"
    sealed = seal_calls(
        body,
        {"ScreenToClient"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    member, api = sorted(sealed.calls, key=lambda call: call.placeholder)
    assert (member.argc, member.hwnd_receiver) == (2, True)
    assert (api.argc, api.hwnd_receiver) == (2, False)


def test_a_bare_member_call_gets_the_implicit_handle():
    """`ScreenToClient(&pt)` in a CWindowImpl class means this->m_hWnd's window."""
    from rustconv.call_seal import repair_window_receivers

    body = "void f(void)\n{\n    ScreenToClient(&pt);\n}\n"
    sealed = seal_calls(
        body,
        {"ScreenToClient"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    token = _only(sealed, "ScreenToClient").placeholder
    rust, repaired = repair_window_receivers(f"{token}(&mut pt);", sealed)
    assert rust == f"{token}((*this).m_hWnd, &mut pt);"
    assert repaired == [f"{token}=ScreenToClient"]


def test_a_zero_argument_member_call_gets_only_the_handle():
    from rustconv.call_seal import repair_window_receivers

    body = "void f(void)\n{\n    DestroyWindow();\n}\n"
    sealed = seal_calls(
        body,
        {"DestroyWindow"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    token = _only(sealed, "DestroyWindow").placeholder
    rust, _ = repair_window_receivers(f"{token}();", sealed)
    assert rust == f"{token}((*this).m_hWnd);"


def test_an_already_correct_handle_is_not_doubled():
    from rustconv.call_seal import repair_window_receivers

    body = "void f(void)\n{\n    ScreenToClient(&pt);\n}\n"
    sealed = seal_calls(
        body,
        {"ScreenToClient"},
        hwnd_receiver_callees=cwindow_method_arity(ATLWIN),
    )
    token = _only(sealed, "ScreenToClient").placeholder
    rust, repaired = repair_window_receivers(f"{token}((*this).m_hWnd, &mut pt);", sealed)
    assert rust == f"{token}((*this).m_hWnd, &mut pt);"
    assert repaired == []
