"""Stripping bindgen's C++ method wrappers must not take the bitfield accessors with them."""

import importlib.util
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "strip_cpp_method_impls", Path(__file__).resolve().parents[1] / "strip_cpp_method_impls.py"
)
SCMI = importlib.util.module_from_spec(_SPEC)
assert _SPEC.loader is not None
_SPEC.loader.exec_module(SCMI)


WRAPPER = """\
impl CFSFolder {
    #[inline]
    pub unsafe fn FormatSize(size: LONGLONG, Buf: LPWSTR) -> HRESULT {
        CFSFolder_FormatSize(size, Buf)
    }
}
"""

BITFIELD = """\
impl SHELLSTATE {
    #[inline]
    pub fn fShowAllObjects(&self) -> BOOL {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(0usize, 1u8) as u32) }
    }
}
"""

GENERIC = """\
impl<T> __BindgenUnionField<T> {
    #[inline]
    pub const fn new() -> Self {
        __BindgenUnionField(::core::marker::PhantomData)
    }
}
"""

TRAIT_IMPL = """\
impl ::core::default::Default for __BindgenUnionField<u8> {
    fn default() -> Self {
        Self::new()
    }
}
"""


def test_a_cpp_method_wrapper_is_removed():
    out, removed = SCMI.strip(WRAPPER)
    assert removed == ["CFSFolder"]
    assert "CFSFolder_FormatSize" not in out


def test_bitfield_accessors_survive():
    """The only correct way to read a C `:1` field — removing these would be the bug."""
    out, removed = SCMI.strip(BITFIELD)
    assert removed == []
    assert out == BITFIELD


def test_generic_helper_impls_survive():
    out, removed = SCMI.strip(GENERIC)
    assert removed == [] and out == GENERIC


def test_trait_impls_survive():
    out, removed = SCMI.strip(TRAIT_IMPL)
    assert removed == [] and out == TRAIT_IMPL


def test_mixed_file_keeps_everything_but_the_wrapper():
    out, removed = SCMI.strip(GENERIC + BITFIELD + WRAPPER + TRAIT_IMPL)
    assert removed == ["CFSFolder"]
    for keep in ("__BindgenUnionField", "fShowAllObjects", "core::default::Default"):
        assert keep in out
    assert "CFSFolder_FormatSize" not in out


def test_several_wrappers_are_all_removed():
    second = WRAPPER.replace("CFSFolder", "CDrivesFolder")
    out, removed = SCMI.strip(WRAPPER + second)
    assert sorted(removed) == ["CDrivesFolder", "CFSFolder"]
    assert out.strip() == ""


def test_a_method_named_like_the_type_but_not_a_call_is_kept():
    """`CFoo_bar` appearing only as a TYPE name must not condemn the block."""
    src = "impl CFoo {\n    pub fn size() -> usize {\n        core::mem::size_of::<CFoo_bar>()\n    }\n}\n"
    out, removed = SCMI.strip(src)
    assert removed == [] and out == src
