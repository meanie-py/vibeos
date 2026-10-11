"""bindgen_com_cinterface: a C++-mode island's opaque interfaces take WIDL's C shape."""

import pytest

import bindgen_com_cinterface as conv

ISLAND = """#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IUnknown {
    pub vtable_: *const IUnknown__bindgen_vtable,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ITfSource {
    pub _base: IUnknown,
}
#[repr(C)]
pub struct CESMAP {
    pub m_pComp: *mut ITfSource,
}
"""

COMRAW = """#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ITfSourceVtbl {
    pub Release: ::core::option::Option<unsafe extern "stdcall" fn(This: *mut ITfSource) -> ULONG>,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ITfSource {
    pub lpVtbl: *mut ITfSourceVtbl,
}
"""


def test_an_opaque_interface_takes_its_c_vtable_shape():
    out, done = conv.merge(ISLAND, COMRAW)
    assert done == ["ITfSource"]
    assert "pub struct ITfSource {\n    pub lpVtbl: *mut ITfSourceVtbl,\n}" in out
    # one spelling for all three islands: stdcall on i386, C elsewhere, system is both
    assert 'unsafe extern "system" fn(This: *mut ITfSource) -> ULONG' in out
    # an interface the C view has no Vtbl for, and every non-interface struct, are untouched
    assert "pub vtable_: *const IUnknown__bindgen_vtable," in out
    assert "pub m_pComp: *mut ITfSource," in out


def test_a_vtbl_the_island_already_has_is_refused():
    with pytest.raises(SystemExit):
        conv.merge(ISLAND + "pub struct ITfSourceVtbl {\n    pub x: u8,\n}\n", COMRAW)
