"""bindgen_ms_anon_members: restore an MS anonymous base-struct member bindgen padded over."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import bindgen_ms_anon_members as tool  # noqa: E402


def test_plain_first_member():
    h = "typedef struct _CHANNEL_DATA_PATA {\n  CHANNEL_DATA_COMMON;\n  PVOID x;\n} X;\n"
    assert tool.bare_first_members(h) == {"_CHANNEL_DATA_PATA": "CHANNEL_DATA_COMMON"}


def test_cplusplus_arms_take_the_c_spelling():
    # ndis.h: every NDIS 5 protocol field (Name, ReceiveHandler, BindAdapterHandler, ...)
    # sat in the padding until the #else arm was read.
    h = (
        "typedef struct _NDIS50_PROTOCOL_CHARACTERISTICS {\n"
        "#ifdef __cplusplus\n"
        "  NDIS40_PROTOCOL_CHARACTERISTICS Ndis40Chars;\n"
        "#else\n"
        "  NDIS40_PROTOCOL_CHARACTERISTICS;\n"
        "#endif\n"
        "  PVOID ReservedHandlers[4];\n"
        "} NDIS50_PROTOCOL_CHARACTERISTICS;\n"
    )
    assert tool.bare_first_members(h) == {
        "_NDIS50_PROTOCOL_CHARACTERISTICS": "NDIS40_PROTOCOL_CHARACTERISTICS"
    }


def test_padding_becomes_the_named_member_with_a_size_assert():
    rust = (
        "pub struct _NDIS50_PROTOCOL_CHARACTERISTICS {\n"
        "    pub __bindgen_padding_0: __BindgenOpaqueArray<[u8; 144usize]>,\n"
        "    pub ReservedHandlers: [PVOID; 4usize],\n"
        "}\n"
        "pub struct _NDIS40_PROTOCOL_CHARACTERISTICS {\n}\n"
        "pub type NDIS40_PROTOCOL_CHARACTERISTICS = _NDIS40_PROTOCOL_CHARACTERISTICS;\n"
    )
    new, done, asserts = tool.transform(
        rust, {"_NDIS50_PROTOCOL_CHARACTERISTICS": "NDIS40_PROTOCOL_CHARACTERISTICS"}
    )
    assert "pub __bindgen_anon_1: _NDIS40_PROTOCOL_CHARACTERISTICS," in new
    assert "size_of::<_NDIS40_PROTOCOL_CHARACTERISTICS>() == 144" in asserts[0]


_WIN32K_HEADER = """
typedef struct _HEAD { HANDLE h; DWORD cLockObj; } HEAD;
typedef struct _THROBJHEAD
{
    HEAD;
    PTHREADINFO pti;
} THROBJHEAD;
"""

_WIN32K_ISLAND = """#[derive(Copy, Clone)]
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


def test_the_win32k_base_is_restored_and_size_asserted():
    out, done, asserts = tool.transform(_WIN32K_ISLAND, tool.bare_first_members(_WIN32K_HEADER))
    assert done == ["_THROBJHEAD._HEAD"]
    assert "    pub __bindgen_anon_1: _HEAD,\n    pub pti" in out
    assert "size_of::<_HEAD>() == 8" in asserts[0]


def test_a_derive_the_base_lacks_is_dropped():
    # _HEAD derives no Debug (ntuser.h), so `#[derive(Debug)]` on _THROBJHEAD stops compiling.
    out, _, _ = tool.transform(_WIN32K_ISLAND, tool.bare_first_members(_WIN32K_HEADER))
    assert "#[derive(Clone, Copy)]\npub struct _THROBJHEAD" in out
    assert "#[derive(Copy, Clone)]\n#[repr(C)]\npub struct _HEAD" in out
