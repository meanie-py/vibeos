//! mswsock-rust prelude — the single import surface the translated modules use.
//!
//! Same role as ws2_32-rust/wshtcpip-rust's prelude: rustconv emits per-file `extern` blocks
//! and bare macro/enum names, so this bridges the gaps between the bindgen `types` island and
//! the translated C:
//!   1. bindgen RUSTIFIED the C enums — alias the bare names the C source actually uses.
//!   2. bindgen SKIPS cast-#defines (SOCKET_ERROR, INVALID_SOCKET, SIO_*) — transcribe them.
//!   3. C MACROS/inlines (NT_SUCCESS, RtlZeroMemory) have NO linkable symbol — implement them
//!      here (never as an out-of-line no-op `extern fn`, the field-manual §9.65 trap).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
use core::ffi::{c_int, c_void};

// ---- 0. Wide literals ----
// Replaces the byte-string-as-wide-pointer form, which is wrong twice over: `b"..."` has
// ALIGNMENT 1 (an odd address is an amd64-only 0x1E STATUS_DATATYPE_MISALIGNMENT), and Rust does
// not append the SECOND zero byte a UTF-16 terminator needs.
#[macro_export]
macro_rules! w {
    ($s:expr $(,)?) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "w!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out // trailing element stays 0 => NUL-terminated
        };
        OUT.as_ptr()
    }};
}
pub use crate::w;

// ---- 1. Types bindgen didn't cover (mswsock-private headers) ----

// MSW_BUFFER / PMSW_BUFFER — the DLL's private scratch buffer for building WSAQUERYSET blobs
// (dll/win32/mswsock/mswhelper.h). Producer (mswhelper) and consumer (nsplookup) are BOTH in
// this crate, so it never crosses a wire/ABI boundary — a faithful repr(C) transcription of the
// header struct is layout-safe. Fields verbatim from mswhelper.h:6-12.
#[repr(C)]
pub struct MSW_BUFFER {
    pub bytesUsed: DWORD,
    pub bytesMax: DWORD,
    pub buffer: *mut BYTE,
    pub bufendptr: *mut BYTE, // first "unused" byte
    pub bufok: BOOL,          // FALSE once an mswBuffer* op has failed
}
pub type PMSW_BUFFER = *mut MSW_BUFFER;

// _BLOB — wtypes BLOB (ULONG cbSize; BYTE* pBlobData). bindgen emitted the struct under the
// `tagBLOB` tag; the C source refers to it by the `_BLOB` tag. Identical layout — alias.
pub use crate::types::tagBLOB as _BLOB;

// PSVCHOST_GLOBAL_DATA — svc.h opaque svchost handle. The SvchostPushServiceGlobals stub only
// logs and NEVER dereferences it, so an opaque pointer preserves the ABI (one pointer-width
// param) exactly without hand-transcribing a struct we don't touch.
#[repr(C)]
pub struct SVCHOST_GLOBAL_DATA {
    _opaque: [u8; 0],
}
pub type PSVCHOST_GLOBAL_DATA = *mut SVCHOST_GLOBAL_DATA;

// ---- 2. Enum members bindgen prefixed with the enum type name; the C uses the bare name ----
// windns.h DNS_FREE_TYPE::DnsFreeRecordList (=1) — passed to DnsRecordListFree(dp, DnsFreeRecordList).
pub use crate::types::DNS_FREE_TYPE_DnsFreeRecordList as DnsFreeRecordList;

// ---- 3. Cast-#define consts bindgen skips — values verified against the cited header ----
// ws2def.h:167 SIO_GET_EXTENSION_FUNCTION_POINTER = _WSAIORW(IOC_WS2,6)
//   = IOC_INOUT | IOC_WS2 | 6 = 0xC0000000 | 0x08000000 | 6   (ws2def.h:149,160,280-282)
pub const SIO_GET_EXTENSION_FUNCTION_POINTER: DWORD = 0xC800_0006;
// winsock2.h:236 INVALID_SOCKET = (SOCKET)(~0) — all-ones of SOCKET (=UINT_PTR) width.
pub const INVALID_SOCKET: SOCKET = !0;
// winbase.h INVALID_HANDLE_VALUE = ((HANDLE)(LONG_PTR)-1) — all-ones pointer.
pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE;
// winreg.h HKEY_LOCAL_MACHINE = ((HKEY)(ULONG_PTR)0x80000002).
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
// winnls.h:245 CP_ACP = 0 (the ANSI code page); MultiByteToWideChar/WideCharToMultiByte take UINT.
pub const CP_ACP: UINT = 0;
// winsock.h — IP protocol number; the one use casts `as INT`. u32 to match the bindgen IPPROTO
// family, mirroring wshtcpip-rust's prelude.
pub const IPPROTO_UDP: u32 = 17;

// ---- 4. C macros/inlines with no linkable symbol — reimplemented inline (field-manual §9.65:
// never as an out-of-line no-op extern fn). Both are DDK write/copy macros. `len` is a byte
// count (usize); the two RtlZeroMemory sites that wrote `as ULONG` are cast `as usize` at the site. ----
#[inline(always)]
pub unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}
#[inline(always)]
pub unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// ---- 5. CRT + ws2_32 symbols the translation calls but never declared. Convention per the
// exporting module (wshtcpip prelude's "one declaration, one truth"): the CRT is cdecl
// (extern "C"), ws2_32's Win32 API is stdcall (extern "system" → `_htons@4` on i386). ----
extern "C" {
    // msvcrt <ctype.h> — used only on ASCII bytes of the services DB, where it is locale-invariant.
    pub fn isspace(c: c_int) -> c_int;
}
extern "system" {
    // ws2_32.spec — host->network u16 (stdcall), the counterpart to the ntohs/ntohl family.
    pub fn htons(hostshort: u16) -> u16;
}

// strsafe.h StringCbCopyA is a `static inline` (STRSAFEAPI) — NO linkable symbol, so a
// declared extern is undefined at link. Reimplement it inline (the §9.65-sanctioned shim for a
// header inline): bounded copy, `cbDest` is the DEST buffer size in BYTES, always NUL-terminates,
// returns S_OK (0) or STRSAFE_E_INVALID_PARAMETER (0x80070057) for a zero-length dest. The 3
// mswsock call sites ignore the return and size the dest exactly.
#[inline]
pub unsafe fn StringCbCopyA(pszDest: PCHAR, cbDest: usize, pszSrc: PCHAR) -> i32 {
    if cbDest == 0 {
        return 0x8007_0057u32 as i32;
    }
    let dst = pszDest as *mut u8;
    let src = pszSrc as *const u8;
    let mut i: usize = 0;
    while i + 1 < cbDest && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
    0 // S_OK
}

// strsafe.h StringCchCatW / StringCchLengthW — `static inline`s, no linkable symbol → shims.
// (The C source calls the UNICODE macro names StringCchCat/StringCchLength.) cch* are CHAR counts.
#[inline]
pub unsafe fn StringCchCat(pszDest: LPWSTR, cchDest: usize, pszSrc: LPCWSTR) -> i32 {
    if cchDest == 0 {
        return 0x8007_0057u32 as i32; // STRSAFE_E_INVALID_PARAMETER
    }
    let mut d: usize = 0;
    while d < cchDest && *pszDest.add(d) != 0 {
        d += 1;
    }
    let mut s: usize = 0;
    while d + 1 < cchDest && *pszSrc.add(s) != 0 {
        *pszDest.add(d) = *pszSrc.add(s);
        d += 1;
        s += 1;
    }
    if d < cchDest {
        *pszDest.add(d) = 0;
    }
    0 // S_OK
}
#[inline]
pub unsafe fn StringCchLength(psz: LPCWSTR, cchMax: usize, pcchLength: *mut usize) -> i32 {
    let mut n: usize = 0;
    while n < cchMax && *psz.add(n) != 0 {
        n += 1;
    }
    if n >= cchMax {
        return 0x8007_0057u32 as i32; // no NUL within cchMax
    }
    if !pcchLength.is_null() {
        *pcchLength = n;
    }
    0 // S_OK
}
// guiddef.h InlineIsEqualGUID — byte-equal compare of the 16-byte GUID (nonzero if equal).
#[inline]
pub unsafe fn IsEqualGUID(rguid1: *const GUID, rguid2: *const GUID) -> i32 {
    let n = core::mem::size_of::<GUID>();
    (core::slice::from_raw_parts(rguid1 as *const u8, n)
        == core::slice::from_raw_parts(rguid2 as *const u8, n)) as i32
}
