//! aclui-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1 — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier).
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
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

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= enum member aliases ========================================================
// bindgen prefixes a C enum's members with the enum's tag, so the bare spelling the C uses is
// absent from the island. Alias, never re-declare: a literal here could drift from the island's
// value silently. These are the uxtheme part/state ids checklist.c draws with (SUPPORT_UXTHEME).
// None is a `case` label here, so this is convenience rather than the §9.112 correctness trap —
// but it is the same missing-bare-name cause, so it is checked the same way.
pub const BP_CHECKBOX: BUTTONPARTS = BUTTONPARTS_BP_CHECKBOX;
pub const CBS_UNCHECKEDNORMAL: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDNORMAL;
pub const CBS_UNCHECKEDHOT: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDHOT;
pub const CBS_UNCHECKEDPRESSED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDPRESSED;
pub const CBS_UNCHECKEDDISABLED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDDISABLED;
pub const CBS_CHECKEDNORMAL: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDNORMAL;
pub const CBS_CHECKEDHOT: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDHOT;
pub const CBS_CHECKEDPRESSED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDPRESSED;
pub const CBS_CHECKEDDISABLED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDDISABLED;

// ================= enum member aliases ========================================================
// bindgen prefixes a C enum's members with the enum's tag, so the bare spelling the C uses is
// absent from the island. Alias, never re-declare: a literal here could drift silently.
//
// 0 of these 1 are `case` labels, which makes island completeness a CORRECTNESS
// property (§9.112): an unresolved name in a `match` arm is not an error, it is a fresh binding
// that matches EVERYTHING and leaves the following arms dead. `verify_match_consts` names them.
//
// NB the scan for this MUST be multi-line — bindgen wraps the declaration when the name is long,
// and a single-line regex reports a confident zero (that is how authz's seven got through).
pub const PolicyAccountDomainInformation: _POLICY_INFORMATION_CLASS =
    _POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation;

// A wide string literal, NUL-terminated, with one stable address for the program's life.
// `static` and not `const`: a const is inlined at each use site, so `.as_ptr()` would hand the
// callee the address of a temporary. ASCII-only by construction, so a non-ASCII literal fails
// at COMPILE time instead of silently truncating.
//
// This replaces the byte-string-as-wide-pointer form, which is wrong twice over: `b"..."` has
// ALIGNMENT 1 (an odd address is an amd64-only 0x1E STATUS_DATATYPE_MISALIGNMENT), and Rust
// does not append the SECOND zero byte a UTF-16 terminator needs — `b"\\\0"` is one code unit
// with no terminator at all, and an odd byte count cannot be whole u16 units.
#[macro_export]
macro_rules! w {
    ($s:expr) => {{
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

// ================= UNICODE A/W mapping =======================================================
// aclui builds with -DUNICODE, so winuser.h/commctrl.h/prsht.h #define each bare name onto its
// W form. bindgen emits declarations, never macros, so the bare spelling the translation kept is
// absent from the island. ALIAS the island's W declaration — re-declaring it here would be a
// second signature for the same import, and an arity or width slip there is a silent ABI bug
// that no C build can show.
pub use crate::types::{
    ConvertSidToStringSidW as ConvertSidToStringSid, CreateEventW as CreateEvent,
    CreatePropertySheetPageW as CreatePropertySheetPage, DefWindowProcW as DefWindowProc,
    GetTextExtentPoint32W as GetTextExtentPoint32, GetTextMetricsW as GetTextMetrics,
    LoadBitmapW as LoadBitmap, LoadCursorW as LoadCursor, LoadStringW as LoadString,
    MessageBoxW as MessageBox, PropertySheetW as PropertySheet,
    RegisterClassW as RegisterClass, RegisterClipboardFormatW as RegisterClipboardFormat,
    SendMessageW as SendMessage, SetDlgItemTextW as SetDlgItemText,
    SetWindowTextW as SetWindowText, SystemParametersInfoW as SystemParametersInfo,
    UnregisterClassW as UnregisterClass, DrawTextW as DrawText,
};
pub use crate::types::{
    LPPROPSHEETPAGEW as LPPROPSHEETPAGE, LVCOLUMNW as LVCOLUMN, LVCOLUMNW as LV_COLUMN,
    LVFINDINFOW as LVFINDINFO, LVITEMW as LVITEM, PROPSHEETHEADERW as PROPSHEETHEADER,
    PROPSHEETPAGEW as PROPSHEETPAGE,
};
// `_LVITEMW` is a translation artifact — commctrl.h spells the tag `tagLVITEMW`, which is the
// name bindgen kept. Same type, so alias rather than touch every use site.
pub use crate::types::tagLVITEMW as _LVITEMW;

// ================= enum member aliases (OLE) ==================================================
// Same rule as the uxtheme block above: bindgen prefixes an enum's members with its tag, so
// alias the island's value. A literal here could drift from the header silently.
pub const CLSCTX_INPROC_SERVER: tagCLSCTX = tagCLSCTX_CLSCTX_INPROC_SERVER;
pub const DVASPECT_CONTENT: tagDVASPECT = tagDVASPECT_DVASPECT_CONTENT;
pub const TYMED_HGLOBAL: tagTYMED = tagTYMED_TYMED_HGLOBAL;
pub const VT_ARRAY: VARENUM = VARENUM_VT_ARRAY;
pub const VT_UI1: VARENUM = VARENUM_VT_UI1;

// ================= HRESULT / NTSTATUS =========================================================
// winerror.h spells these `_HRESULT_TYPEDEF_(0x…L)`, a function-like macro bindgen cannot fold.
pub const S_OK: HRESULT = 0;
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
// ntstatus.h:445 — ((NTSTATUS)0xC0000073)
pub const STATUS_NONE_MAPPED: NTSTATUS = 0xC0000073u32 as NTSTATUS;

// ================= CRT ========================================================================
extern "C" {
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int;
}

// ================= function-like MACROS =======================================================
// windef.h `#define TRUE 1` / `FALSE 0` are UNTYPED in C and land wherever a BOOL is wanted.
// bindgen models a #define as `u32`, so every BOOL parameter rejects the -sys glob's version.
// Re-TYPED here, never re-valued: 0 and 1 are the header's values, and a local item shadows
// the glob so the whole crate sees the BOOL-typed pair.
pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;

// minwindef.h:189/190 — `((WORD)((DWORD_PTR)(l) & 0xffff))`. The C cast to DWORD_PTR is what
// makes this work for a signed LPARAM and an unsigned WPARAM alike, and DWORD_PTR is 32-bit on
// i386 and 64-bit elsewhere. A `T: Into<u64>` bound cannot express that (i32 has no
// `Into<u64>`, and WPARAM is u64 on amd64), so the widening is a trait with one impl per
// integer the call sites actually pass. Each keeps the low 32 bits exactly as the C cast does.
pub trait AsDwordPtr { fn as_dword_ptr(self) -> u64; }
impl AsDwordPtr for i16 { #[inline(always)] fn as_dword_ptr(self) -> u64 { self as u16 as u64 } }
impl AsDwordPtr for u16 { #[inline(always)] fn as_dword_ptr(self) -> u64 { self as u64 } }
impl AsDwordPtr for i32 { #[inline(always)] fn as_dword_ptr(self) -> u64 { self as u32 as u64 } }
impl AsDwordPtr for u32 { #[inline(always)] fn as_dword_ptr(self) -> u64 { self as u64 } }
impl AsDwordPtr for i64 { #[inline(always)] fn as_dword_ptr(self) -> u64 { self as u64 } }
impl AsDwordPtr for u64 { #[inline(always)] fn as_dword_ptr(self) -> u64 { self } }
impl AsDwordPtr for isize { #[inline(always)] fn as_dword_ptr(self) -> u64 { self as usize as u64 } }
impl AsDwordPtr for usize { #[inline(always)] fn as_dword_ptr(self) -> u64 { self as u64 } }
#[inline(always)]
pub fn LOWORD<T: AsDwordPtr>(l: T) -> WORD { (l.as_dword_ptr() & 0xffff) as WORD }
#[inline(always)]
pub fn HIWORD<T: AsDwordPtr>(l: T) -> WORD { ((l.as_dword_ptr() >> 16) & 0xffff) as WORD }

// checklist.c:1390-1391 `case '\r': case '\n':` — Rust's `|` in a PATTERN is alternation, so
// the translation's shape was right; `b'\r' as u16` simply is not pattern syntax. Named consts
// are, and they say which characters these are.
pub const WCHAR_CR: WCHAR = 13;
pub const WCHAR_LF: WCHAR = 10;
// `case VK_BACK:` on a WCHAR switch — C converts the int constant to WCHAR (0x08).
pub const VK_BACK_WCHAR: WCHAR = VK_BACK as WCHAR;
// commctrl.h:202 `#define LVN_FIRST (0U-100U)` — UNSIGNED, so LVN_ITEMCHANGED is 0xFFFFFF9B.
// NMHDR.code is UINT and the other arms of that switch are TU-local UINTs, so this one is
// re-typed to match them. Same 32 bits, and `as` keeps them: only the spelling changes.
pub const LVN_ITEMCHANGED_U: UINT = LVN_ITEMCHANGED as UINT;
// winuser.h:4110/4111 — MAKELONG packs low/high into a DWORD, then the parameter cast widens.
#[inline(always)]
pub fn MAKELONG(l: WORD, h: WORD) -> DWORD { ((l as DWORD) & 0xffff) | (((h as DWORD) & 0xffff) << 16) }
#[inline(always)]
pub fn MAKELPARAM(l: WORD, h: WORD) -> LPARAM { MAKELONG(l, h) as LPARAM }
#[inline(always)]
pub fn MAKEWPARAM(l: WORD, h: WORD) -> WPARAM { MAKELONG(l, h) as WPARAM }
// wingdi.h:3386
#[inline(always)]
pub fn RGB(r: BYTE, g: BYTE, b: BYTE) -> COLORREF {
    (r as COLORREF) | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}
// winuser.h:580/582
#[inline(always)]
pub fn IS_INTRESOURCE<T>(i: *const T) -> bool { (i as ULONG_PTR) >> 16 == 0 }
#[inline(always)]
pub fn MAKEINTRESOURCE(i: UINT) -> LPWSTR { LOWORD(i) as ULONG_PTR as LPWSTR }
// winuser.h:605/695 — resource ids that are MAKEINTRESOURCE'd pseudo-pointers.
pub const RT_STRING: UINT = 6;
pub const IDC_ARROW: UINT = 32512;
// winuser.h:881-883 — DWLP_USER = 0 + sizeof(LRESULT) + sizeof(DLGPROC). Computed, never a
// literal: both terms are pointer-width, so a constant here would be an i386-only value.
pub const DWLP_MSGRESULT: c_int = 0;
pub const DWLP_DLGPROC: c_int = DWLP_MSGRESULT + core::mem::size_of::<LRESULT>() as c_int;
pub const DWLP_USER: c_int = DWLP_DLGPROC + core::mem::size_of::<usize>() as c_int;
// objsel.h:27 — TEXT("…") under UNICODE. A `static`, so every use shares one address.
pub static CFSTR_DSOP_DS_SELECTION_LIST_BUF: [u16; 29] = [
    0x43, 0x46, 0x53, 0x54, 0x52, 0x5f, 0x44, 0x53, 0x4f, 0x50, 0x5f, 0x44, 0x53, 0x5f,
    0x53, 0x45, 0x4c, 0x45, 0x43, 0x54, 0x49, 0x4f, 0x4e, 0x5f, 0x4c, 0x49, 0x53, 0x54, 0,
];

#[macro_export]
macro_rules! FIELD_OFFSET { ($t:ty, $f:ident $(,)?) => { (core::mem::offset_of!($t, $f) as LONG) }; }
#[inline(always)]
pub unsafe fn ZeroMemory<T>(d: *mut T, len: usize) -> *mut c_void {
    memset(d as *mut c_void, 0, len)
}
#[inline(always)]
pub unsafe fn CopyMemory<D, S>(d: *mut D, s: *const S, len: usize) -> *mut c_void {
    memcpy(d as *mut c_void, s as *const c_void, len)
}
pub use crate::FIELD_OFFSET;
#[inline(always)]
pub fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }
#[inline(always)]
pub fn abs(v: c_int) -> c_int { v.wrapping_abs() }

// ================= Interlocked ================================================================
// The C calls the MSVC intrinsics; both return the value AFTER the operation.
#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *mut core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *mut core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}

// ================= Get/SetWindowLongPtr =======================================================
// On i386 winuser.h #defines these onto the non-`Ptr` entry points; only 64-bit exports the
// `Ptr` symbols. Declaring the `Ptr` form unconditionally is an i386-only LINK_FAIL.
#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub unsafe fn GetWindowLongPtr(wnd: HWND, index: c_int) -> LONG_PTR {
    GetWindowLongW(wnd, index) as LONG_PTR
}
#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub unsafe fn SetWindowLongPtr(wnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(wnd, index, value as LONG) as LONG_PTR
}
#[cfg(target_pointer_width = "64")]
extern "system" {
    #[link_name = "GetWindowLongPtrW"]
    pub fn GetWindowLongPtr(wnd: HWND, index: c_int) -> LONG_PTR;
    #[link_name = "SetWindowLongPtrW"]
    pub fn SetWindowLongPtr(wnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR;
}

// ================= commctrl.h ListView_* ======================================================
// Each is `SNDMSG(hwnd, LVM_*, wparam, lparam)` verbatim from commctrl.h; under UNICODE the
// bare LVM_* resolve to their W forms. WPARAM/LPARAM order is transcribed from the header, not
// recalled — a swap here is silent at compile time and wrong at run time.
#[inline(always)]
pub unsafe fn ListView_GetItem(hwnd: HWND, pitem: *mut LVITEMW) -> BOOL {
    SendMessageW(hwnd, LVM_GETITEMW, 0, pitem as LPARAM) as BOOL
}
#[inline(always)]
pub unsafe fn ListView_SetItem(hwnd: HWND, pitem: *const LVITEMW) -> BOOL {
    SendMessageW(hwnd, LVM_SETITEMW, 0, pitem as LPARAM) as BOOL
}
#[inline(always)]
pub unsafe fn ListView_InsertItem(hwnd: HWND, pitem: *const LVITEMW) -> c_int {
    SendMessageW(hwnd, LVM_INSERTITEMW, 0, pitem as LPARAM) as c_int
}
#[inline(always)]
pub unsafe fn ListView_InsertColumn(hwnd: HWND, col: c_int, pcol: *const LVCOLUMNW) -> c_int {
    SendMessageW(hwnd, LVM_INSERTCOLUMNW, col as WPARAM, pcol as LPARAM) as c_int
}
#[inline(always)]
pub unsafe fn ListView_SetColumn(hwnd: HWND, col: c_int, pcol: *const LVCOLUMNW) -> BOOL {
    SendMessageW(hwnd, LVM_SETCOLUMNW, col as WPARAM, pcol as LPARAM) as BOOL
}
#[inline(always)]
pub unsafe fn ListView_FindItem(hwnd: HWND, start: c_int, plvfi: *const LVFINDINFOW) -> c_int {
    SendMessageW(hwnd, LVM_FINDITEMW, start as WPARAM, plvfi as LPARAM) as c_int
}
#[inline(always)]
pub unsafe fn ListView_GetNextItem(hwnd: HWND, i: c_int, flags: UINT) -> c_int {
    SendMessageW(hwnd, LVM_GETNEXTITEM, i as WPARAM, MAKELPARAM(flags as WORD, 0)) as c_int
}
// NB the operands are CROSSED relative to the call: lPrm is the WPARAM, pfnCompare the LPARAM.
#[inline(always)]
pub unsafe fn ListView_SortItems(hwnd: HWND, pfn_compare: PFNLVCOMPARE, l_prm: LPARAM) -> BOOL {
    SendMessageW(hwnd, LVM_SORTITEMS, l_prm as WPARAM,
                 core::mem::transmute::<PFNLVCOMPARE, LPARAM>(pfn_compare)) as BOOL
}
#[inline(always)]
pub unsafe fn ListView_SetImageList(hwnd: HWND, himl: HIMAGELIST, kind: c_int) -> HIMAGELIST {
    SendMessageW(hwnd, LVM_SETIMAGELIST, kind as WPARAM, himl as LPARAM) as HIMAGELIST
}
#[inline(always)]
pub unsafe fn ListView_SetExtendedListViewStyleEx(hwnd: HWND, mask: DWORD, style: DWORD) -> DWORD {
    SendMessageW(hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, mask as WPARAM, style as LPARAM) as DWORD
}

// ================= oleauto.h VARIANT accessors ================================================
// This SDK builds the NONAMELESSUNION form, so V_VT(a) is `(a)->n1.n2.vt` (oleauto.h:206) and
// V_ARRAY(a) is V_UNION(a,parray) = `(a)->n1.n2.n3.parray` (oleauto.h:222). The union arm is
// selected ONCE here rather than at each use: `__BindgenUnionField<T>` is zero-sized, so
// picking the arm per field is how a wrong arm goes unnoticed.
#[inline(always)]
pub unsafe fn V_VT(a: *const VARIANT) -> VARTYPE { (*a).n1.n2.as_ref().vt }
#[inline(always)]
pub unsafe fn V_ARRAY(a: *const VARIANT) -> *mut SAFEARRAY {
    *(*a).n1.n2.as_ref().n3.parray.as_ref()
}

// ================= varargs ====================================================================
// Defined in dll/win32/aclui/rust_varargs_shim.c, which exists only because `va_start` cannot
// appear in stable Rust. Declaring and calling a C-variadic IS stable, so call sites read
// exactly as the C did.
extern "C" {
    pub fn LoadAndFormatString(
        hInstance: HINSTANCE,
        uID: UINT,
        lpTarget: *mut LPWSTR,
        ...
    ) -> DWORD;
}
