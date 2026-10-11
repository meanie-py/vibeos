"""rustconv.com_vtables: a C++ island's COM interfaces get their header's C layout."""

from rustconv import com_vtables

CXX_ISLAND = """\
#[repr(C)]
pub struct IUnknown__bindgen_vtable(::core::ffi::c_void);
#[repr(C)]
#[derive(Debug)]
pub struct IUnknown {
    pub vtable_: *const IUnknown__bindgen_vtable,
}
#[repr(C)]
#[derive(Debug)]
pub struct IFoo {
    pub _base: IUnknown,
}
pub type HRESULT = i32;
#[repr(C)]
pub struct NotAnInterface {
    pub x: i32,
}
"""

C_PASS = """\
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IFooVtbl {
    pub QueryInterface: ::core::option::Option<
        unsafe extern "system" fn(
            This: *mut IFoo,
            riid: *const IID,
            ppvObject: *mut *mut ::core::ffi::c_void,
        ) -> HRESULT,
    >,
    pub Release: ::core::option::Option<unsafe extern "system" fn(This: *mut IFoo) -> ULONG>,
    pub Ping: ::core::option::Option<unsafe extern "system" fn(This: *mut IFoo, n: DWORD)>,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IFoo {
    pub lpVtbl: *mut IFooVtbl,
}
pub type HRESULT = ::core::ffi::c_long;
"""


def test_interfaces_are_the_cxx_shaped_structs():
    assert com_vtables.interfaces(CXX_ISLAND) == ["IFoo", "IUnknown"]


def test_merge_swaps_the_body_and_adds_only_new_names():
    text, missing = com_vtables.merge(CXX_ISLAND, C_PASS)
    assert "pub struct IFoo {\n    pub lpVtbl: *mut IFooVtbl,\n}" in text
    assert "pub _base: IUnknown" not in text
    assert "pub struct IFooVtbl {" in text
    # The island's own HRESULT stays; the C pass's duplicate is not appended.
    assert text.count("pub type HRESULT") == 1
    assert "pub struct NotAnInterface {" in text
    # No C section was bound for IUnknown here: it is reported, not silently dropped.
    assert missing == ["IUnknown"]


def test_methods_dispatch_through_the_slot_with_this_first():
    text, _ = com_vtables.merge(CXX_ISLAND, C_PASS)
    impls = com_vtables.methods(text)
    assert "impl IFoo {" in impls
    assert (
        "pub unsafe fn QueryInterface(&mut self, riid: *const IID, "
        "ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT {"
    ) in impls
    assert "Some(f) => f(self, riid, ppvObject)," in impls
    assert "pub unsafe fn Release(&mut self) -> ULONG {" in impls
    # A `void` slot: no return type, and the `->` of a sibling never unbalances the split.
    assert "pub unsafe fn Ping(&mut self, n: DWORD) {" in impls
    assert "Some(f) => f(self, n)," in impls
