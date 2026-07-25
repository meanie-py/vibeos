"""semdiff must judge argument types against the real callee prototypes.

Both false positives measured on landed, hand-audited code came from the model
applying C reasoning to the Rust side:

  1. `SHQueryValueExW(hkey, name, 0, ...)` in C vs `null_mut()` in Rust was
     reported CRITICAL "8-byte pointer where a 4-byte DWORD is expected". The real
     prototype takes `LPDWORD`; the C `0` is a null pointer constant and the Rust
     is correct. The model saw two bodies and no signature — while the signature
     sat in the very file the Rust body came from.
  2. `flags & FLAG != 0` was reported HIGH "inverted condition, evaluates
     `flags & (FLAG != 0)`". That is C's precedence. In Rust `&` binds tighter
     than `!=`, and the other parse (`u32 & bool`) would not even compile.
"""

from rustconv.semdiff import SYSTEM_PROMPT, extract_extern_prototypes


def test_extracts_declarations_from_an_extern_block():
    src = """
extern "system" {
    fn SHQueryValueExW(
        hKey: HKEY,
        pszValue: LPCWSTR,
        pdwReserved: LPDWORD,
        pcbData: LPDWORD,
    ) -> LONG;
    fn RegCloseKey(hKey: HKEY) -> LONG;
}
"""
    protos = extract_extern_prototypes(src)
    assert set(protos) == {"SHQueryValueExW", "RegCloseKey"}
    # The parameter types are the whole point — they must survive.
    assert "pdwReserved: LPDWORD" in protos["SHQueryValueExW"]
    assert protos["RegCloseKey"].endswith(";")


def test_definitions_are_not_prototypes():
    """A fn with a body is a definition; only declarations are signatures."""
    src = 'pub unsafe extern "C" fn Demo(x: u32) -> u32 { x + 1 }\n'
    assert extract_extern_prototypes(src) == {}


def test_mixed_file_yields_only_the_declared_ones():
    src = """
extern "system" {
    fn Declared(a: *mut u32) -> i32;
}
pub unsafe fn Defined(b: u32) -> u32 { b }
"""
    assert set(extract_extern_prototypes(src)) == {"Declared"}


def test_system_prompt_states_rust_precedence():
    """The precedence rule must stay in the prompt — it is the whole fix for FP #2."""
    assert "bind TIGHTER than the" in SYSTEM_PROMPT
    assert "pointer constant" in SYSTEM_PROMPT
    assert "CALLEE PROTOTYPES" in SYSTEM_PROMPT
