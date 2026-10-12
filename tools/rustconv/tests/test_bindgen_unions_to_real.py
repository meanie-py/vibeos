"""bindgen_unions_to_real: a converted union's bitfield accessors lose the UnionField wrapper."""

import bindgen_unions_to_real as conv

ISLAND = """#[repr(C)]
pub struct _NODE__bindgen_ty_2 {
    pub _bitfield_1: __BindgenUnionField<__BindgenBitfieldUnit<[u8; 1usize]>>,
    pub ParentValue: __BindgenUnionField<ULONG_PTR>,
    pub bindgen_union_field: u32,
}
impl _NODE__bindgen_ty_2 {
    pub fn Red(&self) -> UCHAR {
        unsafe { ::core::mem::transmute(self._bitfield_1.as_ref().get(0usize, 1u8) as u8) }
    }
    pub unsafe fn Red_raw(this: *const Self) -> UCHAR {
        raw_get(
            (*::core::ptr::addr_of!((*this)._bitfield_1)).as_ref() as *const _,
            0usize,
        )
    }
}
"""


def test_union_bitfield_accessors_address_the_bare_unit():
    # winegdi's _RTL_BALANCED_NODE__bindgen_ty_2: 8 errors per arch.
    out = conv.transform(ISLAND)
    assert "pub union _NODE__bindgen_ty_2 {" in out
    assert "self._bitfield_1.get(0usize, 1u8)" in out
    assert "::core::ptr::addr_of!((*this)._bitfield_1)," in out
    assert ".as_ref()" not in out


def test_a_struct_whose_brace_wraps_still_gets_copy():
    """A name past rustfmt's width is emitted as `pub struct NAME` then `{`."""
    island = (
        "#[repr(C)]\n"
        "pub struct _DISPATCHER_HEADER__bindgen_ty_1__bindgen_ty_1__bindgen_ty_3__bindgen_ty_2__bindgen_ty_1\n"
        "{\n"
        "    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 1usize]>,\n"
        "}\n"
        "pub struct Unit;\n"
    )
    out = conv.transform(island)
    assert "#[derive(Copy, Clone)]\n#[repr(C)]\npub struct _DISPATCHER_HEADER" in out
    assert out.count("derive") == 1  # the unit struct is left alone
