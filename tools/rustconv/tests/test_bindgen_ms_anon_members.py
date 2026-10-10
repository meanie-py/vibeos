"""bindgen_ms_anon_members: an MS embedded base struct comes back as a named field."""

import bindgen_ms_anon_members as anon

HEADER = """
typedef struct _HEAD { HANDLE h; DWORD cLockObj; } HEAD;
typedef struct _THROBJHEAD
{
    HEAD;
    PTHREADINFO pti;
} THROBJHEAD;
"""

ISLAND = """#[derive(Copy, Clone)]
#[repr(C)]
pub struct _HEAD {
    pub h: HANDLE,
    pub cLockObj: DWORD,
}
pub type HEAD = _HEAD;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _THROBJHEAD {
    pub __bindgen_padding_0: [u8; 8usize],
    pub pti: *mut _THREADINFO,
}
"""


def test_the_base_is_restored_and_size_asserted():
    out, done, asserts = anon.transform(ISLAND, anon.bare_first_members(HEADER))
    assert done == ["_THROBJHEAD._HEAD"]
    assert "    pub __bindgen_anon_1: _HEAD,\n    pub pti" in out
    assert "size_of::<_HEAD>() == 8" in asserts[0]


def test_a_derive_the_base_lacks_is_dropped():
    # _HEAD derives no Debug (ntuser.h), so `#[derive(Debug)]` on _THROBJHEAD stops compiling.
    out, _, _ = anon.transform(ISLAND, anon.bare_first_members(HEADER))
    assert "#[derive(Clone, Copy)]\npub struct _THROBJHEAD" in out
    assert "#[derive(Copy, Clone)]\n#[repr(C)]\npub struct _HEAD" in out
