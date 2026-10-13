//! Rust translation of `sdk/lib/apisets/apisets.c` — `ApiSetResolveToHost`,
//! the api-set → host-DLL resolver. Replaces reuse of the cmake `libapisets.a`
//! (whose resolver pulled deep NDK headers). The ~624-entry data table is
//! mechanically generated from the checked-in C table at BUILD TIME by this
//! crate's `build.rs` (into `$OUT_DIR/apisets_table.rs`, `include!`d below — no
//! committed table); this is the hand-translated logic.
//!
//! ntdll.dll-proper only (vista doesn't link apisets). `g_Apisets` /
//! `g_ApisetsCount` stay PRIVATE Rust (the only consumer is this resolver), so
//! there is no cross-arch data-symbol decoration to match — only the public
//! `ApiSetResolveToHost` is `#[no_mangle]` (cdecl; `_ApiSetResolveToHost` on i386).

use ntdll_sys::nt::{NTSTATUS, STATUS_SUCCESS};
// G_APISETS lives in the crate-level `apisets_table` module, which build.rs
// generates into $OUT_DIR and lib.rs include!s (no committed table).
use crate::apisets_table::G_APISETS;

/// `UNICODE_STRING` (identical C layout). Buffer is `*const u16` — the table's
/// strings are `'static` read-only.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnicodeString {
    pub length: u16,
    pub maximum_length: u16,
    pub buffer: *const u16,
}
// The buffers all point at `'static` read-only `[u16]` data.
unsafe impl Sync for UnicodeString {}

/// `ROS_APISET` (apisetsp.h): `{ UNICODE_STRING Name; UNICODE_STRING Target;
/// DWORD dwOsVersions; }`.
#[repr(C)]
pub struct RosApiset {
    pub name: UnicodeString,
    pub target: UnicodeString,
    pub dw_os_versions: u32,
}

/// Build a `UNICODE_STRING` over a NUL-terminated `'static [u16]` literal —
/// `RTL_CONSTANT_STRING` (Length excludes the NUL; MaximumLength includes it).
pub const fn us(buf: &'static [u16]) -> UnicodeString {
    let n = buf.len(); // includes the NUL terminator
    UnicodeString {
        length: ((n - 1) * 2) as u16,
        maximum_length: (n * 2) as u16,
        buffer: buf.as_ptr(),
    }
}

// "API-" / "EXT-" as the first 4 WCHARs packed LE, with the 0x20 (lowercase)
// bit cleared on the first three (the mask below upper-cases them).
const API_: u64 = 0x002D_0049_0050_0041; // 'A','P','I','-'
const EXT_: u64 = 0x002D_0054_0058_0045; // 'E','X','T','-'
const PREFIX_BYTES: u16 = 8; // sizeof(L"api-") - sizeof(WCHAR) = 4 WCHARs
const EXT_BYTES: u16 = 8; // sizeof(L".dll") - sizeof(WCHAR) = 4 WCHARs (".dll")

extern "system" {
    /// rtl: case-insensitive UNICODE_STRING compare. NTAPI = stdcall on i386
    /// (`_RtlCompareUnicodeString@12`), cdecl on amd64/arm64 — `extern "system"`
    /// picks the right decoration per arch (a bare `extern "C"` would reference
    /// the undecorated name and fail to link on i386). `UnicodeString` matches
    /// the bindgen `UNICODE_STRING` layout.
    fn RtlCompareUnicodeString(
        s1: *const UnicodeString,
        s2: *const UnicodeString,
        case_insensitive: u8,
    ) -> i32;
}

/// `ApiSetResolveToHost` — see apisets.c. Resolves an `api-*`/`ext-*` name to
/// its host DLL for the given apiset version via binary search over `g_Apisets`.
#[no_mangle]
pub unsafe extern "C" fn ApiSetResolveToHost(
    apiset_version: u32,
    api_to_resolve: *const UnicodeString,
    resolved: *mut u8,
    output: *mut UnicodeString,
) -> NTSTATUS {
    let api = &*api_to_resolve;

    if api.length < PREFIX_BYTES {
        core::ptr::write_volatile(resolved, 0);
        return STATUS_SUCCESS;
    }

    // First 4 WCHARs, upper-cased on the first three (mask leaves WCHAR[3]).
    let prefix = (api.buffer as *const u64).read_unaligned() & 0xFFFF_FFDF_FFDF_FFDF;
    if prefix != API_ && prefix != EXT_ {
        core::ptr::write_volatile(resolved, 0);
        return STATUS_SUCCESS;
    }

    // If there's a ".dll" extension, drop it (the table stores names without).
    let mut tmp = *api;
    let ext = tmp.buffer.add(((tmp.length - EXT_BYTES) / 2) as usize);
    let dotdll = [b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16];
    let is_dll = (0..4).all(|i| (*ext.add(i) | 0x20) == dotdll[i]);
    if is_dll {
        tmp.length -= EXT_BYTES;
    }

    // Binary search (g_Apisets is sorted by Name, case-insensitive).
    let mut lbnd: i32 = 0;
    let mut ubnd: i32 = G_APISETS.len() as i32 - 1;
    while lbnd <= ubnd {
        let index = (ubnd - lbnd) / 2 + lbnd;
        let entry = &G_APISETS[index as usize];
        let result = RtlCompareUnicodeString(&tmp, &entry.name, 1);
        if result == 0 {
            if entry.dw_os_versions & apiset_version != 0 {
                core::ptr::write_volatile(resolved, 1);
                core::ptr::write_volatile(output, entry.target);
            }
            return STATUS_SUCCESS;
        } else if result < 0 {
            ubnd = index - 1;
        } else {
            lbnd = index + 1;
        }
    }

    // No exact row: match as Windows does, ignoring the trailing minor version. Its
    // schema hashes a name only up to the last '-', so `api-ms-win-core-realtime-l1-1-1`
    // resolves through the schema's `-l1-1-2` row; a program built against a minor
    // version our table never listed still loads. Rows sharing the stem are adjacent
    // (the table is sorted), and the last one this version enables wins.
    if let Some(entry) = resolve_any_minor(apiset_version, &tmp) {
        core::ptr::write_volatile(resolved, 1);
        core::ptr::write_volatile(output, entry.target);
        return STATUS_SUCCESS;
    }

    core::ptr::write_volatile(resolved, 0);
    STATUS_SUCCESS
}

/// The table row `name` resolves to with its final `-<digits>` ignored, if any.
unsafe fn resolve_any_minor(apiset_version: u32, name: &UnicodeString) -> Option<&'static RosApiset> {
    let chars = core::slice::from_raw_parts(name.buffer, (name.length / 2) as usize);
    let hyphen = chars.iter().rposition(|&c| c == b'-' as u16)?;
    if hyphen + 1 == chars.len() || !chars[hyphen + 1..].iter().all(|&c| (b'0' as u16..=b'9' as u16).contains(&c)) {
        return None;
    }
    let stem = UnicodeString {
        length: ((hyphen + 1) * 2) as u16,
        maximum_length: ((hyphen + 1) * 2) as u16,
        buffer: name.buffer,
    };
    let mut found = None;
    for entry in G_APISETS.iter() {
        let e = core::slice::from_raw_parts(entry.name.buffer, (entry.name.length / 2) as usize);
        if e.len() <= hyphen + 1 || !e[hyphen + 1..].iter().all(|&c| (b'0' as u16..=b'9' as u16).contains(&c)) {
            continue;
        }
        let head = UnicodeString { length: stem.length, maximum_length: stem.length, buffer: entry.name.buffer };
        if RtlCompareUnicodeString(&stem, &head, 1) == 0 && entry.dw_os_versions & apiset_version != 0 {
            found = Some(entry);
        }
    }
    found
}
