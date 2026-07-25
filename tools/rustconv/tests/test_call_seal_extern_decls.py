"""A forward `extern` declaration is not a call.

The mechanical prompt ORDERS one ("emit a forward `extern "system" { fn Name(args...)
-> Ret; }` block immediately above the function"), so a correct mechanical translation
names every sealed callee twice. Before this was handled, every sealed plain-C
translation failed the call envelope with "not exactly once" — the shell32 COM path
never saw it because normalize_worker_body deletes those blocks before unsealing.
"""

import pytest
from rustconv.call_seal import (
    CallSealError,
    extern_declaration_spans,
    seal_calls,
    unseal_calls,
)

C_BODY = """
int Demo(HKEY hKey)
{
    if (RegOpenKeyExW(hKey, NULL) != 0)
        return 0;
    RegCloseKey(hKey);
    return 1;
}
"""
CALLEES = {"RegOpenKeyExW", "RegCloseKey"}


def _sealed():
    return seal_calls(C_BODY, CALLEES)


def _tok(sealed, original):
    return next(c.placeholder for c in sealed.calls if c.original == original)


def test_forward_declaration_plus_call_round_trips():
    """The shape the mechanical prompt asks for must survive unsealing."""
    sealed = _sealed()
    open_t, close_t = _tok(sealed, "RegOpenKeyExW"), _tok(sealed, "RegCloseKey")
    rust = f"""
extern "system" {{
    fn {open_t}(hKey: HKEY, p: *const u16) -> i32;
    fn {close_t}(hKey: HKEY) -> i32;
}}
pub unsafe fn Demo(hKey: HKEY) -> i32 {{
    if {open_t}(hKey, core::ptr::null()) != 0 {{ return 0; }}
    {close_t}(hKey);
    1
}}
"""
    out = unseal_calls(rust, sealed)
    assert "__RUSTCONV_CALL_" not in out, "every token must be restored"
    # BOTH the declaration and the call site get the real name.
    assert out.count("RegOpenKeyExW") == 2
    assert out.count("RegCloseKey") == 2


def test_declaration_alone_does_not_satisfy_the_envelope():
    """Declaring a callee without calling it is still a hard failure."""
    sealed = _sealed()
    open_t, close_t = _tok(sealed, "RegOpenKeyExW"), _tok(sealed, "RegCloseKey")
    rust = f"""
extern "system" {{
    fn {open_t}(hKey: HKEY, p: *const u16) -> i32;
    fn {close_t}(hKey: HKEY) -> i32;
}}
pub unsafe fn Demo(hKey: HKEY) -> i32 {{
    {close_t}(hKey);
    1
}}
"""
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(rust, sealed)


def test_two_real_calls_are_still_a_duplicate():
    """The forgiveness is only for declarations — a genuinely repeated CALL fails."""
    sealed = _sealed()
    open_t, close_t = _tok(sealed, "RegOpenKeyExW"), _tok(sealed, "RegCloseKey")
    rust = f"""
extern "system" {{ fn {open_t}(hKey: HKEY, p: *const u16) -> i32; }}
pub unsafe fn Demo(hKey: HKEY) -> i32 {{
    if {open_t}(hKey, core::ptr::null()) != 0 {{ return 0; }}
    {open_t}(hKey, core::ptr::null());
    {close_t}(hKey);
    1
}}
"""
    with pytest.raises(CallSealError, match="not exactly once"):
        unseal_calls(rust, sealed)


def test_extern_block_with_a_body_is_not_a_declaration():
    """`extern "C" { fn f() { … } }` defines; it must not be span-forgiven."""
    text = 'extern "C" { fn f() { let x = 1; } }'
    assert extern_declaration_spans(text) == []
    text_decl = 'extern "C" { fn f() -> i32; }'
    assert len(extern_declaration_spans(text_decl)) == 1


def test_extern_definition_item_is_untouched():
    """A plain `extern "C" fn name(...) { … }` item has no block to forgive."""
    assert extern_declaration_spans('pub unsafe extern "C" fn Demo() -> i32 { 1 }') == []
