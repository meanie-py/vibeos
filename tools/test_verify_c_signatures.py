"""Tests for verify_c_signatures — an ORACLE, so it needs an oracle of its own.

Every bug this checker has ever had was the same shape: it resolved a type wrongly, and then
reported its own ignorance as a defect in the code (or, worse, resolved BOTH sides wrongly to
the same value and reported "verified" having checked nothing). Review does not catch that —
the code reads fine either way. What catches it is stating, by hand, what the headers actually
say, and what the checker must and must not fire on.

The two halves below are not the same test:
  * test_resolution        — is the type system modelled correctly? (catches false POSITIVES)
  * test_catches_real_bugs — does it still fire on the bugs it exists for? (catches false
                             NEGATIVES, i.e. a checker that has gone quietly blind)
A checker only needs the second half to pass to LOOK healthy, which is precisely why the
second half must be written down. `DWORD` vs `DWORD_PTR` shipped in the advertised bug list
while the tool was structurally incapable of detecting it; this suite is what noticed.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from verify_c_signatures import (
    Ambiguous,
    Unknown,
    _braced_typedefs,
    _norm_base,
    c_functions,
    is_ptr_sized,
    ptr_depth,
)

# Ground truth, read out of sdk/include by hand. If one of these ever changes, the HEADER
# changed and this table is how you find out — do not "fix" it by copying the tool's output.
RESOLUTION = [
    # (type spelling, indirection depth)
    ("STARTUPINFOW *", 1),
    ("LPSTARTUPINFOW", 1),  # processthreadsapi.h `} STARTUPINFOW,*LPSTARTUPINFOW;`
    ("PROCESS_INFORMATION *", 1),
    ("LPPROCESS_INFORMATION", 1),
    # P-initial STRUCTS. Not pointers. The naming convention does not apply to them and a
    # blind `^P[A-Z]` rule read POINT as a pointer to a type called OINT.
    ("POINT", 0),
    ("POINT *", 1),
    ("PALETTEENTRY", 0),
    ("PIXELFORMATDESCRIPTOR", 0),
    ("PERF_DATA_BLOCK", 0),
    # Real P/LP pointer typedefs
    ("HANDLE", 1),
    ("PHANDLE", 2),
    ("PVOID", 1),
    ("LPVOID", 1),
    ("*mut LPVOID", 2),
    ("LPWSTR", 1),
    ("LPCWSTR", 1),
    ("PCWSTR", 1),
    ("void *", 1),
    # DECLARE_HANDLE(n) -> `typedef struct n##__{..} *n`. All 82 of them are depth 1.
    ("HWND", 1),
    ("HMENU", 1),
    ("HDC", 1),
    ("HKEY", 1),
    ("HFONT *", 2),
    # The integer tower. `typedef _W64 unsigned long ULONG_PTR` must not harvest `unsigned`
    # as a typedef name aliased to `_W64`.
    ("INT", 0),
    ("LPINT", 1),
    ("SIZE_T", 0),
    ("PSIZE_T", 1),
    ("DWORD", 0),
    ("DWORD_PTR", 0),
    ("ULONG_PTR", 0),
    ("unsigned long", 0),
    ("NTSTATUS", 0),
    ("BOOL", 0),
    ("SOCKET", 0),
    # Rust spellings
    ("i32", 0),
    ("u32", 0),
    ("isize", 0),
    ("*mut c_void", 1),
    ("*mut *mut c_void", 2),
    ("*mut ::core::ffi::c_void", 1),
]


@pytest.mark.parametrize(("spelling", "depth"), RESOLUTION)
def test_resolution(spelling: str, depth: int) -> None:
    assert ptr_depth(spelling) == depth


# The bugs that actually shipped into the tree. Each MUST be reported as a mismatch.
REAL_BUGS = [
    ("INT", "LPINT"),  # ws2_32 WSARemoveServiceClass return
    ("SIZE_T", "PSIZE_T"),  # ws2_32 ComputeQuerySetSize return
    ("HANDLE", "LPHANDLE"),  # ws2_32 hLookup param
    ("HANDLE", "PHANDLE"),  # lsasrv-shaped
    ("LPWSTR", "LPWSTR *"),  # double indirection
    ("DWORD", "DWORD_PTR"),  # ws2_32 WPUCompleteOverlappedRequest — WIDTH, not depth
    ("INT", "INT_PTR"),
    ("LONG", "LONG_PTR"),
    ("DWORD", "SIZE_T"),
    ("ULONG", "ULONG_PTR"),
]

# Correct translations. NONE may be reported, or the tool trains people to ignore it.
CORRECT = [
    ("INT", "i32"),
    ("DWORD", "u32"),
    ("SIZE_T", "usize"),
    ("size_t", "usize"),  # inet_ntop: the lower-case C spelling is pointer-sized too
    ("ULONG_PTR", "usize"),
    ("DWORD_PTR", "usize"),
    ("HANDLE", "HANDLE"),
    ("HANDLE", "PVOID"),
    ("LPVOID", "*mut c_void"),
    ("BOOL", "i32"),
    ("NTSTATUS", "i32"),
    ("LPCWSTR", "LPCWSTR"),
    ("LPARAM", "LPARAM"),
    ("WPARAM", "usize"),
    ("LONG", "c_long"),
    ("STARTUPINFOW *", "LPSTARTUPINFOW"),
    ("SOCKET", "SOCKET"),
]


def _differs(c: str, r: str) -> bool:
    if ptr_depth(c) != ptr_depth(r):
        return True
    return is_ptr_sized(c) != is_ptr_sized(r)


@pytest.mark.parametrize(("ctype", "rtype"), REAL_BUGS)
def test_catches_real_bugs(ctype: str, rtype: str) -> None:
    assert _differs(ctype, rtype), f"went blind to C {ctype} vs Rust {rtype}"


@pytest.mark.parametrize(("ctype", "rtype"), CORRECT)
def test_no_false_positive(ctype: str, rtype: str) -> None:
    assert not _differs(ctype, rtype), f"false positive on C {ctype} vs Rust {rtype}"


def test_unknown_is_raised_not_guessed() -> None:
    """An unresolvable name must NOT silently become depth 0.

    Guessing 0 fails in the false-GREEN direction: an unknown C type and an unknown Rust type
    both land on 0, compare equal, and the pair is reported as verified having been checked.
    """
    with pytest.raises(Unknown):
        ptr_depth("ThisTypeDoesNotExistAnywhere")


def test_ambiguous_is_raised_not_guessed() -> None:
    """BSTR is `OLECHAR*` in three places and `long` in stdole2.idl. Refuse to pick."""
    try:
        ptr_depth("BSTR")
    except Ambiguous:
        return
    except Unknown:
        pytest.fail("BSTR should be ambiguous (conflicting decls), not unknown")
    # If it ever resolves cleanly the conflict is gone from the headers — fine, but notice.


def test_idl_does_not_outvote_a_real_header() -> None:
    """cordebug.idl marshals LPSTARTUPINFOW as `SIZE_T`; processthreadsapi.h defines the real
    struct pointer. The .h wins — the C compiler never saw the .idl."""
    assert ptr_depth("LPSTARTUPINFOW") == 1


def test_norm_base_strips_modifiers_not_type_names() -> None:
    assert _norm_base("_W64 unsigned long") == "long"
    assert _norm_base("unsigned __int64") == "__int64"
    assert _norm_base("void") == "void"
    assert _norm_base("unsigned") == "int"  # bare `unsigned` means `unsigned int`


def test_braced_typedef_ignores_nested_member_braces() -> None:
    """`} NAME;` is ambiguous without brace matching: rpcasync.h has a union MEMBER literally
    named HWND, and reading it as a typedef made the real HWND ambiguous."""
    text = """
typedef union _RPC_ASYNC_NOTIFICATION_INFO {
    struct {
        HWND hWnd;
        UINT Msg;
    } HWND;
    HANDLE hEvent;
} RPC_ASYNC_NOTIFICATION_INFO, *PRPC_ASYNC_NOTIFICATION_INFO;
"""
    decls = _braced_typedefs(text)
    assert len(decls) == 1
    assert "RPC_ASYNC_NOTIFICATION_INFO" in decls[0]
    assert "PRPC_ASYNC_NOTIFICATION_INFO" in decls[0]
    # The member must NOT have been harvested as a type.
    assert "hWnd" not in decls[0]


def test_function_pointer_typedef_is_skipped() -> None:
    assert _braced_typedefs("typedef BOOL (CALLBACK *LPFN)(HWND, LPARAM);") == []


def test_a_header_alias_for_winapi_is_not_the_return_type(tmp_path: Path) -> None:
    """vfw.h's `#define VFWAPI WINAPI`: avicap32's return type read as `VFWAPI`, not HWND."""
    src = tmp_path / "avicap32.c"
    src.write_text("HWND VFWAPI capCreateCaptureWindowW(LPCWSTR lpszWindowName)\n{\n}\n")
    assert c_functions(src)["capCreateCaptureWindowW"][0] == "HWND"
