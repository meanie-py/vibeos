//! intl-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{
    c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1 — MACROS, never out-of-line fns (field manual §9.65: an
// out-of-line no-op is an optimization barrier).
//
// DPRINT1 is NOT a no-op. In debug.h it sits under `#if DBG` above the comment "These are
// always printed", and only DPRINT is further gated on `#ifndef NDEBUG`. Our builds set
// DBG=TRUE, so every DPRINT1 in the C we replaced prints — and a no-op here is a diagnostic
// silently deleted, on exactly the failure paths that need one.
extern "C" {
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export]
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        unsafe { $crate::prelude::DbgPrint(concat!($fmt, "\0").as_ptr() as *const _ $(, $arg)*) };
    }};
    // Some grind output already lowered the format to a NUL-terminated `*const c_char`
    // (dhcpcsvc's util.rs writes `"ERROR: %s\n\0".as_ptr() as *const i8`). Tried second, so a
    // plain literal still takes the arm above and gets its NUL appended exactly once.
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        unsafe { $crate::prelude::DbgPrint($fmt $(, $arg)*) };
    }};
}
pub use crate::{DPRINT, DPRINT1};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// ================= wide literals ==============================================================
// This replaces the byte-string-as-wide-pointer form, which is wrong twice over: `b"..."` has
// ALIGNMENT 1 (an odd address is an amd64-only 0x1E STATUS_DATATYPE_MISALIGNMENT), and Rust
// does not append the SECOND zero byte a UTF-16 terminator needs — `b"\\\0"` is one code unit
// with no terminator at all, and an odd byte count cannot be whole u16 units.
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
// The ARRAY sibling of `w!`. A C `static const WCHAR x[] = L"..."` is an ARRAY, and the
// difference matters twice: `[WCHAR; N]` is `Sync` where `*const u16` is not (so it can BE a
// static), and the declared `N` is checked against the literal instead of being a number
// somebody counted.
#[macro_export]
macro_rules! wide {
    ($s:expr $(,)?) => {{
        const INPUT: &[u8] = $s.as_bytes();
        const OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "wide!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out // trailing element stays 0 => NUL-terminated
        };
        OUT
    }};
}
pub use crate::wide;
// A C file-scope `static const` TABLE OF POINTERS has no direct Rust spelling: a raw pointer is
// not `Sync`, so an array of them cannot be a `static`. This newtype promises exactly what the C
// already promises — the table is read-only and every pointer in it is `'static` (each one comes
// from `w!`, i.e. another static). `Deref` keeps the call sites spelled as the C spells them.
#[repr(transparent)]
pub struct SyncTable<T: 'static>(pub T);
unsafe impl<T: 'static> Sync for SyncTable<T> {}
impl<T: 'static> core::ops::Deref for SyncTable<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ================= UNICODE A/W aliases ========================================================
// intl compiles with -DUNICODE, so `#define SendMessage SendMessageW` and its siblings are
// resolved by the PREPROCESSOR and never reach the island. DERIVED, not typed: each name below
// was produced by taking the island's `<N>W` for every unresolved `<N>` the compiler named, so
// a pair cannot be mismatched. Aliases and not re-declarations — a hand-written prototype for
// SendMessageW is the exact shape that links and then corrupts.
pub use crate::types::{
    GetLocaleInfoW as GetLocaleInfo, PropertySheetW as PropertySheet,
    RegEnumValueW as RegEnumValue, RegQueryInfoKeyW as RegQueryInfoKey,
    SendDlgItemMessageW as SendDlgItemMessage, SendMessageW as SendMessage,
    SetupFindFirstLineW as SetupFindFirstLine, SetupGetLineCountW as SetupGetLineCount,
    SetupGetStringFieldW as SetupGetStringField,
    SetupOpenAppendInfFileW as SetupOpenAppendInfFile,
};
// Same rule for the TYPES the A/W macros cover. commctrl.h:2342 `#define LV_ITEM LVITEM`, and
// LVITEM is itself LVITEMW under UNICODE — two macro hops, so neither name is in the island.
// intl builds -DUNICODE, but ntdll-sys is built WITHOUT it, so its `nt::*` glob says
// `TCHAR = c_char` / `LPTSTR = LPSTR` — the ANSI half. Left alone that makes every LPTSTR
// callback take `*mut i8` and silently fail to unify with the W enum-proc slots. An explicit
// item shadows a glob import, so these four re-point the family at the UNICODE half.
pub type TCHAR = WCHAR; // nolint: type-alias-contradiction (intl is -DUNICODE; ntdll-sys is not)
pub type LPTSTR = LPWSTR; // nolint: type-alias-contradiction (same)
pub type LPCTSTR = LPCWSTR; // nolint: type-alias-contradiction (same)
pub type PTSTR = LPWSTR; // nolint: type-alias-contradiction (same)

pub type LV_ITEM = LVITEMW;
pub type LV_COLUMN = LVCOLUMNW;
pub type PROPSHEETPAGE = PROPSHEETPAGEW;
pub type PROPSHEETHEADER = PROPSHEETHEADERW;
pub type LPPROPSHEETPAGE = LPPROPSHEETPAGEW;

// ================= Win32 macros ===============================================================
// bindgen never emits a FUNCTION-LIKE macro, so each of these would otherwise be an unresolved
// name — and an unresolved name is not a no-op: the model invents a declaration for it that
// compiles and is silently wrong. Bodies are transcribed from the header cited on each line.
// The word/locale/resource family lives in `win32macros-rust`, ported once from the headers and
// known-answer tested there. Re-exported here under the same names, so no call site changed.
// (The header spells MAKELCID's cast `($ULONG)`, a ReactOS preprocessor token rather than a type
// Rust can copy; the shared port resolves it to the u32 every call site relies on.)
pub use win32macros_rust::{
    HIWORD, LANGIDFROMLCID, LOWORD, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG,
    PRIMARYLANGID, SORTIDFROMLCID, ZeroMemory, _countof,
};
// MAKEINTRESOURCE (no suffix) is MAKEINTRESOURCEW only because intl compiles -DUNICODE. That is
// this module's property, not the shared crate's, so the alias stays here.
#[macro_export] macro_rules! MAKEINTRESOURCE { ($i:expr $(,)?) => { $crate::prelude::MAKEINTRESOURCEW!($i) }; }
// commctrl.h:2333 `#define INDEXTOSTATEIMAGEMASK(i) ((i) << 12)`
#[macro_export] macro_rules! INDEXTOSTATEIMAGEMASK { ($i:expr $(,)?) => { (($i as UINT) << 12) as UINT }; }
#[macro_export] macro_rules! ListView_InsertColumn {
    ($hwnd:expr, $col:expr, $pcol:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_INSERTCOLUMNW, $col as c_int as WPARAM, $pcol as *const _ as *const core::ffi::c_void as LPARAM) as c_int };
}
#[macro_export] macro_rules! ListView_InsertItem {
    ($hwnd:expr, $item:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_INSERTITEMW, 0, $item as *const _ as *const core::ffi::c_void as LPARAM) as c_int };
}
#[macro_export] macro_rules! ListView_SetExtendedListViewStyle {
    ($h:expr, $dw:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, $dw as LPARAM) as DWORD };
}
#[macro_export] macro_rules! ListView_SetItemState {
    ($hwnd:expr, $i:expr, $data:expr, $mask:expr $(,)?) => {{
        let mut __ms_lvi: LVITEMW = core::mem::zeroed();
        __ms_lvi.stateMask = $mask as UINT;
        __ms_lvi.state = $data as UINT;
        $crate::prelude::SendMessageW($hwnd, LVM_SETITEMSTATE, $i as WPARAM, core::ptr::addr_of_mut!(__ms_lvi) as LPARAM);
    }};
}
// prsht.h:357-358
#[macro_export] macro_rules! PropSheet_Changed {
    ($hdlg:expr, $hpage:expr $(,)?) => { $crate::prelude::SendMessageW($hdlg, PSM_CHANGED, $hpage as WPARAM, 0) };
}
#[macro_export] macro_rules! PropSheet_UnChanged {
    ($hdlg:expr, $hpage:expr $(,)?) => { $crate::prelude::SendMessageW($hdlg, PSM_UNCHANGED, $hpage as WPARAM, 0) };
}
pub use crate::{
    INDEXTOSTATEIMAGEMASK, ListView_InsertColumn, ListView_InsertItem,
    ListView_SetExtendedListViewStyle, ListView_SetItemState, MAKEINTRESOURCE, PropSheet_Changed,
    PropSheet_UnChanged,
};

// winuser.h:4928-4932 / 5454-5458 — GetWindowLongPtrW is a real EXPORT only under _WIN64; on
// i386 the header #defines it to GetWindowLongW, whose return type is LONG and not LONG_PTR.
// Aliasing the wrong one links against a symbol i386's user32 does not export.
#[cfg(target_pointer_width = "64")]
pub use crate::types::{GetWindowLongPtrW as GetWindowLongPtr, SetWindowLongPtrW as SetWindowLongPtr};
#[cfg(target_pointer_width = "32")]
pub use crate::types::{GetWindowLongW as GetWindowLongPtr, SetWindowLongW as SetWindowLongPtr};

// ================= constants bindgen could not fold ===========================================
// Each is a #define whose body is a CAST or an expression, which bindgen does not always emit.
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as i32 as isize as HKEY;
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE; // winbase.h `(HANDLE)(-1)`
pub const HWND_BROADCAST: HWND = 0xffffusize as HWND; // winuser.h `((HWND)0xffff)`
pub const UNICODE_NULL: WCHAR = 0; // ntdef.h `((WCHAR)0)`
// winuser.h, a THREE-macro chain: `DWLP_MSGRESULT 0`, `DWLP_DLGPROC (DWLP_MSGRESULT +
// sizeof(LRESULT))`, `DWLP_USER (DWLP_DLGPROC + sizeof(DLGPROC))`. Computed rather than pasted
// so it cannot be right on i386 and wrong on the 64-bit pair.
pub const DWLP_MSGRESULT: c_int = 0;
pub const DWLP_USER: c_int =
    (core::mem::size_of::<LRESULT>() + core::mem::size_of::<DLGPROC>()) as c_int;
// commctrl.h `#define NM_CUSTOMDRAW (NM_FIRST-12)` where NM_FIRST is `(0U-0U)`. Built from the
// header's own macro rather than a pasted 0xFFFFFFF4 so it stays checkable; the C arithmetic is
// UNSIGNED and wraps.
pub const NM_CUSTOMDRAW: UINT = NM_FIRST.wrapping_sub(12);
pub const GEOCLASS_NATION: SYSGEOCLASS = 16; // winnls.h SYSGEOCLASS enum
pub const GEO_FRIENDLYNAME: SYSGEOTYPE = 8; // winnls.h SYSGEOTYPE enum

// ================= CRT ========================================================================
// msvcrt's wide string/convert family. Declared, never reimplemented — see the memcpy note above
// for why a hand-rolled one is a hang rather than a bug.
extern "C" {
    pub fn wcslen(s: *const WCHAR) -> usize;
    pub fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcsncpy(dst: *mut WCHAR, src: *const WCHAR, n: usize) -> *mut WCHAR;
    pub fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscmp(a: *const WCHAR, b: *const WCHAR) -> c_int;
    pub fn wcsstr(h: *const WCHAR, n: *const WCHAR) -> *mut WCHAR;
    pub fn wcstoul(s: *const WCHAR, end: *mut *mut WCHAR, base: c_int) -> c_ulong;
    pub fn _ultow(value: c_ulong, buf: *mut WCHAR, radix: c_int) -> *mut WCHAR;
    pub fn _itow(value: c_int, buf: *mut WCHAR, radix: c_int) -> *mut WCHAR;
    pub fn _wtoi(s: *const WCHAR) -> c_int;
    pub fn _wtol(s: *const WCHAR) -> c_long;
    pub fn iswalnum(c: c_int) -> c_int;
}
// -DUNICODE picks the W half of the lstr*/wsprintf family, same aliasing rule as above.
pub use crate::types::{lstrcpyW as lstrcpy, lstrlenW as lstrlen, wsprintfW as wsprintf};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= cross-TU (intl's own) ======================================================
// The C reaches these through intl.h; in Rust they are siblings in other modules, so the prelude
// re-exports them and every module keeps spelling the call exactly as the C does.
pub use crate::intl::currency::CurrencyPageProc;
pub use crate::intl::date::DatePageProc;
pub use crate::intl::intl::{PrintErrorMsgBox, ResourceMessageBox};
pub use crate::intl::kblayouts::AddNewKbLayoutsByLcid;
pub use crate::intl::misc::{
    GetSelectedComboBoxIndex, GetSelectedComboBoxText, InsSpacesFmt, ReplaceSubStr,
};
pub use crate::intl::numbers::NumbersPageProc;
pub use crate::intl::sort::{IsSortPageNeeded, SortPageProc};
pub use crate::intl::time::TimePageProc;
pub use crate::intl::advanced::AdvancedPageProc;
pub use crate::intl::generalp::GeneralPageProc;
pub use crate::intl::languages::LanguagesPageProc;
