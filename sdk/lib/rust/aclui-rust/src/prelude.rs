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

extern "C" {
    pub fn wcslen(s: *const WCHAR) -> usize;
    pub fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscmp(a: *const WCHAR, b: *const WCHAR) -> c_int;
    pub fn _wcsnicmp(a: *const WCHAR, b: *const WCHAR, n: usize) -> c_int;
    pub fn abs(x: c_int) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ================= UNICODE A/W aliases ========================================================
// aclui compiles with -DUNICODE, so `#define SendMessage SendMessageW` and its ~90 siblings are
// resolved by the PREPROCESSOR and never reach the island. DERIVED, not typed: each name below
// was produced by taking the island's `<N>W` for every unresolved `<N>` the compiler named, so a
// pair cannot be mismatched. Aliases and not re-declarations — a hand-written prototype for
// SendMessageW is the exact shape that links and then corrupts.
pub use crate::types::{
    ConvertSidToStringSidW as ConvertSidToStringSid, CreateEventW as CreateEvent,
    CreatePropertySheetPageW as CreatePropertySheetPage, DefWindowProcW as DefWindowProc,
    GetTextExtentPoint32W as GetTextExtentPoint32, GetTextMetricsW as GetTextMetrics,
    LoadBitmapW as LoadBitmap, LoadCursorW as LoadCursor, LoadStringW as LoadString,
    MessageBoxW as MessageBox, PropertySheetW as PropertySheet, RegisterClassW as RegisterClass,
    RegisterClipboardFormatW as RegisterClipboardFormat, SendMessageW as SendMessage,
    SetDlgItemTextW as SetDlgItemText, SetWindowTextW as SetWindowText,
    SystemParametersInfoW as SystemParametersInfo, UnregisterClassW as UnregisterClass,
};
pub use crate::types::{
    LPPROPSHEETPAGEW as LPPROPSHEETPAGE, LVCOLUMNW as LVCOLUMN, LVCOLUMNW as LV_COLUMN,
    LVFINDINFOW as LVFINDINFO, LVITEMW as LVITEM, PROPSHEETHEADERW as PROPSHEETHEADER,
    PROPSHEETPAGEW as PROPSHEETPAGE, LVITEMW as _LVITEMW,
};
// DrawTextW/SetWindowLongPtrW/GetWindowLongPtrW: on i386 the Ptr forms are `#define`s onto the
// 32-bit calls, which is why they have no `W` twin of their own to alias on that arch.
pub use crate::types::DrawTextW as DrawText;
#[cfg(target_pointer_width = "32")]
pub use crate::types::{GetWindowLongW as GetWindowLongPtr, SetWindowLongW as SetWindowLongPtr};
#[cfg(target_pointer_width = "64")]
pub use crate::types::{
    GetWindowLongPtrW as GetWindowLongPtr, SetWindowLongPtrW as SetWindowLongPtr,
};

// ================= constants bindgen cannot fold ==============================================
// Cast expressions and macro expressions in their headers, so the island carries none of them.
// psdk/winerror.h:
pub const S_OK: HRESULT = 0;
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
// psdk/ntstatus.h `#define STATUS_NONE_MAPPED ((NTSTATUS)0xC0000073)`
pub const STATUS_NONE_MAPPED: NTSTATUS = 0xC0000073u32 as NTSTATUS;
// winuser.h `#define IDC_ARROW MAKEINTRESOURCE(32512)`, winuser.h `#define RT_STRING
// MAKEINTRESOURCE(6)` — MAKEINTRESOURCE is a cast, so both are macro EXPRESSIONS.
pub const IDC_ARROW: LPCWSTR = 32512 as LPCWSTR;
pub const RT_STRING: LPCWSTR = 6 as LPCWSTR;
// winuser.h `#define DWLP_USER (DWLP_MSGRESULT + sizeof(LRESULT))`, itself built on
// `DWLP_MSGRESULT 0` — computed, so bindgen emits neither.
pub const DWLP_USER: c_int = (core::mem::size_of::<LRESULT>() * 2) as c_int;
// objsel.h `#define CFSTR_DSOP_DS_SELECTION_LIST TEXT("CFSTR_DSOP_DS_SELECTION_LIST")` — a wide
// string under UNICODE, so it needs the w!() backing rather than the island's narrow byte array.
#[macro_export]
macro_rules! CFSTR_DSOP_DS_SELECTION_LIST {
    () => {
        $crate::w!("CFSTR_DSOP_DS_SELECTION_LIST")
    };
}

// ================= header macros ==============================================================
// C macros never reach the island. All stay `macro_rules!` and never become out-of-line fns: an
// out-of-line forwarder is an optimization barrier that uncloaks latent ABI mismatches (§9.65).
#[macro_export]
macro_rules! LOWORD { ($l:expr $(,)?) => { (($l as DWORD_PTR) & 0xffff) as WORD }; }
#[macro_export]
macro_rules! HIWORD { ($l:expr $(,)?) => { ((($l as DWORD_PTR) >> 16) & 0xffff) as WORD }; }
#[macro_export]
macro_rules! MAKEWPARAM {
    ($lo:expr, $hi:expr $(,)?) => { ((($lo as WORD) as DWORD) | ((($hi as WORD) as DWORD) << 16)) as WPARAM };
}
#[macro_export]
macro_rules! RGB {
    ($r:expr, $g:expr, $b:expr $(,)?) => {
        (($r as COLORREF) | (($g as COLORREF) << 8) | (($b as COLORREF) << 16)) as COLORREF
    };
}
// winuser.h `#define MAKEINTRESOURCEW(i) ((LPWSTR)((ULONG_PTR)((WORD)(i))))`
#[macro_export]
macro_rules! MAKEINTRESOURCE { ($i:expr $(,)?) => { (($i as WORD) as usize) as LPWSTR }; }
// winuser.h `#define IS_INTRESOURCE(_r) ((((ULONG_PTR)(_r)) >> 16) == 0)`
#[macro_export]
macro_rules! IS_INTRESOURCE { ($r:expr $(,)?) => { ((($r as ULONG_PTR) >> 16) == 0) }; }
// winnt.h `#define FIELD_OFFSET(t, f) ((LONG)(LONG_PTR)&(((t*)0)->f))`
#[macro_export]
macro_rules! FIELD_OFFSET {
    // `FIELD_OFFSET(T, arr[n])` — the C's "header plus n elements" allocation size, which is how
    // sidcache.c sizes SIDCACHEMGR around its trailing WCHAR SystemName[1].
    ($t:ty, $f:ident [ $i:expr ] $(,)?) => {
        (core::mem::offset_of!($t, $f) + ($i as usize) * core::mem::size_of::<WCHAR>()) as LONG
    };
    ($t:ty, $f:ident $(,)?) => { core::mem::offset_of!($t, $f) as LONG };
}
// winerror.h `#define SUCCEEDED(hr) (((HRESULT)(hr)) >= 0)` / `FAILED(hr) (... < 0)`.
// The comparison is on the SIGNED HRESULT: an intervening `as u32` makes SUCCEEDED
// unconditionally true and deletes the failure branch (field manual, "Semantic rewrites").
#[macro_export]
macro_rules! SUCCEEDED { ($hr:expr $(,)?) => { (($hr) as HRESULT) >= 0 }; }
#[macro_export]
macro_rules! FAILED { ($hr:expr $(,)?) => { (($hr) as HRESULT) < 0 }; }
// sidcache.c:36-37 — the module's own two casts between its opaque handle and the manager.
#[macro_export]
macro_rules! HandleToScm { ($h:expr $(,)?) => { ($h) as PSIDCACHEMGR }; }
#[macro_export]
macro_rules! ScmToHandle { ($s:expr $(,)?) => { ($s) as HANDLE }; }

// winnt.h macros over the CRT block ops.
#[macro_export]
macro_rules! CopyMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {
        core::ptr::copy_nonoverlapping($src as *const u8, $dst as *mut u8, $len as usize)
    };
}
#[macro_export]
macro_rules! ZeroMemory {
    ($dst:expr, $len:expr $(,)?) => {
        core::ptr::write_bytes($dst as *mut u8, 0u8, $len as usize)
    };
}
// The C's `max`/`min` (windef.h), which evaluate their arguments TWICE — kept macros so that
// stays true rather than becoming a single-evaluation fn the C never had.
#[macro_export]
macro_rules! max { ($a:expr, $b:expr $(,)?) => { if ($a) > ($b) { $a } else { $b } }; }
#[macro_export]
macro_rules! min { ($a:expr, $b:expr $(,)?) => { if ($a) < ($b) { $a } else { $b } }; }
// oleauto.h VARIANT accessors: `#define V_VT(X) ((X)->vt)`, `#define V_ARRAY(X) V_UNION(X, parray)`
#[macro_export]
macro_rules! V_VT { ($x:expr $(,)?) => { (*($x)).n1.n2.as_ref().vt }; }
#[macro_export]
macro_rules! V_ARRAY { ($x:expr $(,)?) => { *(*($x)).n1.n2.as_ref().n3.parray.as_ref() }; }
// winbase.h maps these onto the intrinsics; the C semantics are "return the NEW value".
#[macro_export]
macro_rules! InterlockedIncrement { ($p:expr $(,)?) => { $crate::prelude::_InterlockedIncrement($p) }; }
#[macro_export]
macro_rules! InterlockedDecrement { ($p:expr $(,)?) => { $crate::prelude::_InterlockedDecrement($p) }; }

// commctrl.h models the whole ListView API as macros over SendMessage. Kept macros for the same
// reason, and spelled against LVM_* from the island so a message number cannot drift.
#[macro_export]
macro_rules! ListView_GetItem {
    ($hwnd:expr, $pitem:expr $(,)?) => { SendMessage($hwnd, LVM_GETITEMW as UINT, 0, $pitem as *mut c_void as LPARAM) != 0 };
}
#[macro_export]
macro_rules! ListView_SetItem {
    ($hwnd:expr, $pitem:expr $(,)?) => { SendMessage($hwnd, LVM_SETITEMW as UINT, 0, $pitem as *mut c_void as LPARAM) != 0 };
}
#[macro_export]
macro_rules! ListView_InsertItem {
    ($hwnd:expr, $pitem:expr $(,)?) => { SendMessage($hwnd, LVM_INSERTITEMW as UINT, 0, $pitem as *mut c_void as LPARAM) as c_int };
}
#[macro_export]
macro_rules! ListView_FindItem {
    ($hwnd:expr, $start:expr, $plvfi:expr $(,)?) => {
        SendMessage($hwnd, LVM_FINDITEMW as UINT, $start as WPARAM, $plvfi as *mut c_void as LPARAM) as c_int
    };
}
#[macro_export]
macro_rules! ListView_GetNextItem {
    ($hwnd:expr, $i:expr, $flags:expr $(,)?) => {
        SendMessage($hwnd, LVM_GETNEXTITEM as UINT, $i as WPARAM, ($flags as DWORD) as LPARAM) as c_int
    };
}
#[macro_export]
macro_rules! ListView_InsertColumn {
    ($hwnd:expr, $icol:expr, $pcol:expr $(,)?) => {
        SendMessage($hwnd, LVM_INSERTCOLUMNW as UINT, $icol as WPARAM, $pcol as *mut c_void as LPARAM) as c_int
    };
}
#[macro_export]
macro_rules! ListView_SetColumn {
    ($hwnd:expr, $icol:expr, $pcol:expr $(,)?) => {
        SendMessage($hwnd, LVM_SETCOLUMNW as UINT, $icol as WPARAM, $pcol as *mut c_void as LPARAM) != 0
    };
}
#[macro_export]
macro_rules! ListView_SetImageList {
    ($hwnd:expr, $himl:expr, $iImageList:expr $(,)?) => {
        SendMessage($hwnd, LVM_SETIMAGELIST as UINT, $iImageList as WPARAM, $himl as *mut c_void as LPARAM) as HIMAGELIST
    };
}
#[macro_export]
macro_rules! ListView_SortItems {
    ($hwnd:expr, $pfnCompare:expr, $lPrm:expr $(,)?) => {
        SendMessage($hwnd, LVM_SORTITEMS as UINT, $lPrm as WPARAM, $pfnCompare as *mut c_void as LPARAM) != 0
    };
}
#[macro_export]
macro_rules! ListView_SetExtendedListViewStyleEx {
    ($hwnd:expr, $mask:expr, $ex:expr $(,)?) => {
        SendMessage($hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE as UINT, $mask as WPARAM, $ex as *mut c_void as LPARAM) as DWORD
    };
}

pub use crate::{
    CFSTR_DSOP_DS_SELECTION_LIST, CopyMemory, FAILED, HandleToScm, ScmToHandle, SUCCEEDED, FIELD_OFFSET, HIWORD, IS_INTRESOURCE,
    InterlockedDecrement, InterlockedIncrement, LOWORD, ListView_FindItem, ListView_GetItem,
    ListView_GetNextItem, ListView_InsertColumn, ListView_InsertItem,
    ListView_SetColumn, ListView_SetExtendedListViewStyleEx, ListView_SetImageList,
    ListView_SetItem, ListView_SortItems, MAKEINTRESOURCE, MAKEWPARAM, RGB, V_ARRAY, V_VT,
    ZeroMemory, max, min,
};

// msvcrt's intrinsic spellings — the C reaches these through the winbase.h macros above.
extern "system" {
    pub fn _InterlockedIncrement(addend: *mut LONG) -> LONG;
    pub fn _InterlockedDecrement(addend: *mut LONG) -> LONG;
}

// ================= the crate's own cross-TU surface ===========================================
// In C these resolve through checklist.h/misc.h/sidcache.h; in Rust each TU is a module, so the
// prelude re-exports them the way it re-exports the island. Named explicitly, not globbed: a
// glob would also pull each module's file-scope statics into every other module.
pub use crate::aclui::checklist::{RegisterCheckListControl, UnregisterCheckListControl};
pub use crate::aclui::misc::{
    FreeObjectPicker, InitializeObjectPicker, InvokeObjectPickerDialog, ListViewGetSelectedItemData,
    ListViewSelectItem,
};
// Defined in dll/win32/aclui/rust_varargs_shim.c — the variadic entry stable Rust cannot
// DEFINE. It can DECLARE one, though, so the call sites keep their real arity and rustc still
// checks the fixed prefix.
extern "C" {
    pub fn LoadAndFormatString(
        hInstance: HINSTANCE,
        uID: UINT,
        lpTarget: *mut LPWSTR,
        ...
    ) -> DWORD;
}
pub use crate::aclui::sidcache::{
    CreateSidCacheMgr, DereferenceSidReqResult, DestroySidCacheMgr, LookupSidCache,
    ReferenceSidReqResult, PSIDCACHECALLBACKINFO, PSIDCACHEENTRY, PSIDCACHEMGR, PSIDQUEUEENTRY,
    SIDCACHECALLBACKINFO, SIDCACHEENTRY, SIDCACHEMGR, SIDQUEUEENTRY,
};

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

// oleaut32 — the SAFEARRAY accessors objsel's data object hands back. Declared here because
// oleauto.h is outside the island's allowlist and pulling it in would re-emit a second VARIANT.
extern "system" {
    pub fn SafeArrayAccessData(psa: *mut SAFEARRAY, ppvData: *mut *mut c_void) -> HRESULT;
    pub fn SafeArrayUnaccessData(psa: *mut SAFEARRAY) -> HRESULT;
}

// bindgen prefixes an enumerator with its enum tag, so the bare spelling the C uses is absent.
// Alias, never re-declare — a literal here could drift from the island's value silently.
pub const CLSCTX_INPROC_SERVER: tagCLSCTX = tagCLSCTX_CLSCTX_INPROC_SERVER;
pub const DVASPECT_CONTENT: tagDVASPECT = tagDVASPECT_DVASPECT_CONTENT;
pub const TYMED_HGLOBAL: tagTYMED = tagTYMED_TYMED_HGLOBAL;
pub const VT_ARRAY: VARENUM = VARENUM_VT_ARRAY;
pub const VT_UI1: VARENUM = VARENUM_VT_UI1;
