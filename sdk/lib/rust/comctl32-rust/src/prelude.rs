//! comctl32-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// wine/debug.h's TRACE/ERR/WARN/FIXME, as advapi32/kernel32's winedebug.rs spell them. ERR and
// FIXME are ON by default (libwine's default_flags), so they print; WARN and TRACE are off
// unless DEBUGCHANNEL opens the channel, so silence is the faithful default for those two.
// Always macros, never out-of-line no-op fns (field manual §9.65).
extern "C" {
    /// ntdll's `DbgPrint` -- the serial sink `wine_dbg_log` ends up in anyway.
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}
#[macro_export]
macro_rules! ERR {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(
                concat!("err:(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
    // A format the translation already made a C pointer (`b"..\0".as_ptr()`).
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(concat!("err:(", file!(), ":", line!(), ") \0").as_ptr() as *const _);
            $crate::prelude::DbgPrint($fmt as *const _ $(, $arg)*)
        }
    };
}
#[macro_export]
macro_rules! FIXME {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(
                concat!("fixme:(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
    // A format the translation already made a C pointer (`b"..\0".as_ptr()`).
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(concat!("fixme:(", file!(), ":", line!(), ") \0").as_ptr() as *const _);
            $crate::prelude::DbgPrint($fmt as *const _ $(, $arg)*)
        }
    };
}
// nolint: scan_dropped_prints (channel off by default -- see above)
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
// nolint: scan_dropped_prints (channel off by default -- see above)
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, FIXME, TRACE, WARN};

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
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAY_SIZE, ARRAYSIZE, GetAValue, RGBA, GET_WHEEL_DELTA_WPARAM, IS_INTRESOURCE, InterlockedDecrement, InterlockedIncrement, MAKEINTATOM, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= Hand-written: header names the island cannot carry =========================
pub use core::ffi::c_longlong;
pub type VOID = c_void;
// windef.h `#define TRUE 1` / `FALSE 0` are C `int`s; ntdll-sys's bindgen spells them `u32`,
// which put 1,857 `expected i32, found u32` on every `return FALSE;` from a BOOL function.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
// guiddef.h `#define REFIID const IID *` / REFCLSID (C, not C++).
pub type REFIID = *const IID;
pub type REFCLSID = *const IID;

// wine/debug.h. The translations expanded TRACE/ERR/WARN/FIXME into the header's own inline
// form (test `__dbch->flags`, then call ros_dbg_log), so they need the header's types and each
// file's channel exactly as `WINE_DEFAULT_DEBUG_CHANNEL(ch)` defines them -- see
// `wine_default_debug_channel!`, invoked once per module.
pub type __wine_debug_class = c_int;
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: c_uchar,
    pub name: [c_char; 15],
}
/// The `char name[15]` a channel is initialised with: the name, NUL-padded.
/// `name` of a channel from its static's identifier: `__wine_dbch_listview` -> "listview".
pub const fn __wine_channel_name(s: &[u8]) -> [c_char; 15] {
    const PREFIX: &[u8] = b"__wine_dbch_";
    let mut skip = 0;
    while skip < PREFIX.len() && skip < s.len() && s[skip] == PREFIX[skip] {
        skip += 1;
    }
    if skip != PREFIX.len() {
        skip = 0;
    }
    let mut out = [0 as c_char; 15];
    let mut i = 0;
    while skip + i < s.len() && i < 14 {
        out[i] = s[skip + i] as c_char;
        i += 1;
    }
    out
}
/// `WINE_DEFAULT_DEBUG_CHANNEL(ch)`: `static struct __wine_debug_channel __wine_dbch_ch =
/// { (unsigned char)~0, "ch" }` plus the file's `__wine_dbch___default` pointing at it.
#[macro_export]
macro_rules! wine_default_debug_channel {
    ($chan:ident) => {
        #[allow(non_upper_case_globals)]
        static mut $chan: $crate::prelude::__wine_debug_channel =
            $crate::prelude::__wine_debug_channel {
                flags: !0,
                name: $crate::prelude::__wine_channel_name(
                    stringify!($chan).as_bytes(),
                ),
            };
        #[allow(non_upper_case_globals)]
        static mut __wine_dbch___default: *mut $crate::prelude::__wine_debug_channel =
            unsafe { &raw mut $chan };
    };
}
extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const c_char,
        func: *const c_char,
        line: c_int,
        format: *const c_char,
        ...
    ) -> c_int;
    pub fn wine_dbg_sprintf(format: *const c_char, ...) -> *const c_char;
}
// wine/debug.h static inlines.
pub unsafe fn wine_dbgstr_point(pt: *const POINT) -> *const c_char {
    if pt.is_null() {
        return c"(null)".as_ptr();
    }
    wine_dbg_sprintf(c"(%ld,%ld)".as_ptr(), (*pt).x, (*pt).y)
}
pub unsafe fn wine_dbgstr_rect(rect: *const RECT) -> *const c_char {
    if rect.is_null() {
        return c"(null)".as_ptr();
    }
    let r = &*rect;
    wine_dbg_sprintf(c"(%ld,%ld)-(%ld,%ld)".as_ptr(), r.left, r.top, r.right, r.bottom)
}

// wine/heap.h static inlines.
pub unsafe fn heap_alloc(len: SIZE_T) -> *mut c_void {
    HeapAlloc(GetProcessHeap(), 0, len)
}
pub unsafe fn heap_alloc_zero(len: SIZE_T) -> *mut c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
pub unsafe fn heap_realloc(mem: *mut c_void, len: SIZE_T) -> *mut c_void {
    if mem.is_null() {
        return HeapAlloc(GetProcessHeap(), 0, len);
    }
    HeapReAlloc(GetProcessHeap(), 0, mem, len)
}
pub unsafe fn heap_free(mem: *mut c_void) {
    HeapFree(GetProcessHeap(), 0, mem);
}

// wine/list.h static inlines.
pub unsafe fn list_add_after(elem: *mut list, to_add: *mut list) {
    (*to_add).next = (*elem).next;
    (*to_add).prev = elem;
    (*(*elem).next).prev = to_add;
    (*elem).next = to_add;
}
pub unsafe fn list_tail(list: *const list) -> *mut list {
    // list_prev(list, list): the element before the head, or NULL for an empty list.
    let ret = (*list).prev;
    if ret as *const list == list { core::ptr::null_mut() } else { ret }
}

// commctrl.h / winuser.h / prsht.h / mmsystem.h macros.
pub const LPSTR_TEXTCALLBACKW: LPWSTR = -1isize as LPWSTR;
pub const LPSTR_TEXTCALLBACKA: LPSTR = -1isize as LPSTR;
pub const TVI_ROOT: HTREEITEM = (-0x10000isize) as HTREEITEM;
pub const TVI_FIRST: HTREEITEM = (-0xffffisize) as HTREEITEM;
pub const TVI_LAST: HTREEITEM = (-0xfffeisize) as HTREEITEM;
pub const TVI_SORT: HTREEITEM = (-0xfffdisize) as HTREEITEM;
pub const HWND_TOPMOST: HWND = -1isize as HWND;
pub const HINST_COMMCTRL: HINSTANCE = -1isize as HINSTANCE;
// winuser.h: IDI_HAND / IDI_EXCLAMATION / IDI_ASTERISK, UNICODE (MAKEINTRESOURCEW).
pub const IDI_ERROR: LPCWSTR = 32513usize as LPCWSTR;
pub const IDI_WARNING: LPCWSTR = 32515usize as LPCWSTR;
pub const IDI_INFORMATION: LPCWSTR = 32516usize as LPCWSTR;
pub type NM_UPDOWN = NMUPDOWN;
pub type LPTOOLINFOW = LPTTTOOLINFOW;
#[macro_export]
macro_rules! mmioFOURCC {
    ($c0:expr, $c1:expr, $c2:expr, $c3:expr $(,)?) => {
        (($c0 as u8 as u32) | (($c1 as u8 as u32) << 8) | (($c2 as u8 as u32) << 16)
            | (($c3 as u8 as u32) << 24))
    };
}
pub use crate::mmioFOURCC;
/// commctrl.h `CCSIZEOF_STRUCT(structname, member)`: offsetof + sizeof the member.
#[macro_export]
macro_rules! CCSIZEOF_STRUCT {
    ($ty:ty, $field:ident) => {
        core::mem::offset_of!($ty, $field) + {
            const fn __sz<T>(_: *const T) -> usize {
                core::mem::size_of::<T>()
            }
            let u = core::mem::MaybeUninit::<$ty>::uninit();
            __sz(unsafe { core::ptr::addr_of!((*u.as_ptr()).$field) })
        }
    };
}
pub use crate::CCSIZEOF_STRUCT;
pub const TTTOOLINFOA_V1_SIZE: usize = CCSIZEOF_STRUCT!(TTTOOLINFOA, lpszText);
pub const TTTOOLINFOW_V1_SIZE: usize = CCSIZEOF_STRUCT!(TTTOOLINFOW, lpszText);
pub const TTTOOLINFOA_V2_SIZE: usize = CCSIZEOF_STRUCT!(TTTOOLINFOA, lParam);
pub const TTTOOLINFOW_V2_SIZE: usize = CCSIZEOF_STRUCT!(TTTOOLINFOW, lParam);
pub const TTTOOLINFOA_V3_SIZE: usize = CCSIZEOF_STRUCT!(TTTOOLINFOA, lpReserved);
pub const TTTOOLINFOW_V3_SIZE: usize = CCSIZEOF_STRUCT!(TTTOOLINFOW, lpReserved);
pub const REBARBANDINFOA_V3_SIZE: usize = CCSIZEOF_STRUCT!(REBARBANDINFOA, wID);
pub const REBARBANDINFOW_V3_SIZE: usize = CCSIZEOF_STRUCT!(REBARBANDINFOW, wID);
pub const REBARBANDINFOA_V6_SIZE: usize = CCSIZEOF_STRUCT!(REBARBANDINFOA, cxHeader);
pub const REBARBANDINFOW_V6_SIZE: usize = CCSIZEOF_STRUCT!(REBARBANDINFOW, cxHeader);
pub const PROPSHEETPAGEA_V1_SIZE: usize = CCSIZEOF_STRUCT!(PROPSHEETPAGEA, pcRefParent);
pub const PROPSHEETPAGEW_V1_SIZE: usize = CCSIZEOF_STRUCT!(PROPSHEETPAGEW, pcRefParent);

// winuser.h:4110-4111 + the UNICODE `MAKEINTRESOURCE`. Module-local because LPARAM/WPARAM/LRESULT
// are pointer-width island aliases; only the MAKELONG inside them is shared.
#[macro_export]
macro_rules! MAKEINTRESOURCE { ($i:expr $(,)?) => { $crate::prelude::MAKEINTRESOURCEW!($i) }; }
pub use crate::MAKEINTRESOURCE;
#[macro_export]
macro_rules! MAKELPARAM { ($lo:expr, $hi:expr $(,)?) => { $crate::prelude::MAKELONG!($lo, $hi) as LPARAM }; }
pub use crate::MAKELPARAM;
#[macro_export]
macro_rules! MAKEWPARAM { ($lo:expr, $hi:expr $(,)?) => { $crate::prelude::MAKELONG!($lo, $hi) as WPARAM }; }
pub use crate::MAKEWPARAM;
#[macro_export]
macro_rules! MAKELRESULT { ($lo:expr, $hi:expr $(,)?) => { $crate::prelude::MAKELONG!($lo, $hi) as LRESULT }; }
pub use crate::MAKELRESULT;
// winuser.h:4469-4470 `CreateWindowA/W(a..k)` = `CreateWindowExA/W(0, a..k)`.
#[macro_export]
macro_rules! CreateWindowW {
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr, $j:expr, $k:expr $(,)?) => {
        $crate::prelude::CreateWindowExW(0, $a, $b, $c, $d, $e, $f, $g, $h, $i, $j, $k)
    };
}
pub use crate::CreateWindowW;
#[macro_export]
macro_rules! CreateWindowA {
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr, $j:expr, $k:expr $(,)?) => {
        $crate::prelude::CreateWindowExA(0, $a, $b, $c, $d, $e, $f, $g, $h, $i, $j, $k)
    };
}
pub use crate::CreateWindowA;
// treeview.c:194 `((x) >> 12) & 0x0f` and listview.c:408 `((x) & LVIS_STATEIMAGEMASK) >> 12`
// are the same value (LVIS_STATEIMAGEMASK is 0xF000), so one macro serves both TUs.
#[macro_export]
macro_rules! STATEIMAGEINDEX { ($x:expr $(,)?) => { ((($x) >> 12) & 0x0f) }; }
pub use crate::STATEIMAGEINDEX;
// commctrl.h:2333 / :425 -- untyped shifts, so the operand keeps its own type.
#[macro_export]
macro_rules! INDEXTOSTATEIMAGEMASK { ($i:expr $(,)?) => { (($i) << 12) }; }
pub use crate::INDEXTOSTATEIMAGEMASK;
#[macro_export]
macro_rules! INDEXTOOVERLAYMASK { ($i:expr $(,)?) => { (($i) << 8) }; }
pub use crate::INDEXTOOVERLAYMASK;
extern "system" {
    pub fn CreateWindowExA(dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    pub fn ChildWindowFromPoint(hWndParent: HWND, Point: POINT) -> HWND;
    pub fn DialogBoxParamW(hInstance: HINSTANCE, lpTemplateName: LPCWSTR, hWndParent: HWND, lpDialogFunc: DLGPROC, dwInitParam: LPARAM) -> INT_PTR;
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LONG;
    pub fn GetVersion() -> DWORD;
}

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ABS_DOWNDISABLED: ARROWBTNSTATES = ARROWBTNSTATES_ABS_DOWNDISABLED; // island enum
pub const ABS_DOWNHOT: ARROWBTNSTATES = ARROWBTNSTATES_ABS_DOWNHOT; // island enum
pub const ABS_DOWNNORMAL: ARROWBTNSTATES = ARROWBTNSTATES_ABS_DOWNNORMAL; // island enum
pub const ABS_LEFTDISABLED: ARROWBTNSTATES = ARROWBTNSTATES_ABS_LEFTDISABLED; // island enum
pub const ABS_LEFTHOT: ARROWBTNSTATES = ARROWBTNSTATES_ABS_LEFTHOT; // island enum
pub const ABS_LEFTNORMAL: ARROWBTNSTATES = ARROWBTNSTATES_ABS_LEFTNORMAL; // island enum
pub const ABS_LEFTPRESSED: ARROWBTNSTATES = ARROWBTNSTATES_ABS_LEFTPRESSED; // island enum
pub const ABS_RIGHTDISABLED: ARROWBTNSTATES = ARROWBTNSTATES_ABS_RIGHTDISABLED; // island enum
pub const ABS_RIGHTHOT: ARROWBTNSTATES = ARROWBTNSTATES_ABS_RIGHTHOT; // island enum
pub const ABS_RIGHTNORMAL: ARROWBTNSTATES = ARROWBTNSTATES_ABS_RIGHTNORMAL; // island enum
pub const ABS_RIGHTPRESSED: ARROWBTNSTATES = ARROWBTNSTATES_ABS_RIGHTPRESSED; // island enum
pub const ABS_UPDISABLED: ARROWBTNSTATES = ARROWBTNSTATES_ABS_UPDISABLED; // island enum
pub const ABS_UPHOT: ARROWBTNSTATES = ARROWBTNSTATES_ABS_UPHOT; // island enum
pub const ABS_UPNORMAL: ARROWBTNSTATES = ARROWBTNSTATES_ABS_UPNORMAL; // island enum
pub const BP_CHECKBOX: BUTTONPARTS = BUTTONPARTS_BP_CHECKBOX; // island enum
pub const BP_GROUPBOX: BUTTONPARTS = BUTTONPARTS_BP_GROUPBOX; // island enum
pub const BP_PUSHBUTTON: BUTTONPARTS = BUTTONPARTS_BP_PUSHBUTTON; // island enum
pub const BP_RADIOBUTTON: BUTTONPARTS = BUTTONPARTS_BP_RADIOBUTTON; // island enum
pub const CBF_BEENFOCUSED: combobox_state_flags = combobox_state_flags_CBF_BEENFOCUSED; // island enum
pub const CBF_BUTTONDOWN: combobox_state_flags = combobox_state_flags_CBF_BUTTONDOWN; // island enum
pub const CBF_CAPTURE: combobox_state_flags = combobox_state_flags_CBF_CAPTURE; // island enum
pub const CBF_DROPPED: combobox_state_flags = combobox_state_flags_CBF_DROPPED; // island enum
pub const CBF_EDIT: combobox_state_flags = combobox_state_flags_CBF_EDIT; // island enum
pub const CBF_EUI: combobox_state_flags = combobox_state_flags_CBF_EUI; // island enum
pub const CBF_FOCUSED: combobox_state_flags = combobox_state_flags_CBF_FOCUSED; // island enum
pub const CBF_HOT: combobox_state_flags = combobox_state_flags_CBF_HOT; // island enum
pub const CBF_MEASUREITEM: combobox_state_flags = combobox_state_flags_CBF_MEASUREITEM; // island enum
pub const CBF_NOEDITNOTIFY: combobox_state_flags = combobox_state_flags_CBF_NOEDITNOTIFY; // island enum
pub const CBF_NOLBSELECT: combobox_state_flags = combobox_state_flags_CBF_NOLBSELECT; // island enum
pub const CBF_NOREDRAW: combobox_state_flags = combobox_state_flags_CBF_NOREDRAW; // island enum
pub const CBF_NORESIZE: combobox_state_flags = combobox_state_flags_CBF_NORESIZE; // island enum
pub const CBF_NOROLLUP: combobox_state_flags = combobox_state_flags_CBF_NOROLLUP; // island enum
pub const CBF_NOTIFY: combobox_state_flags = combobox_state_flags_CBF_NOTIFY; // island enum
pub const CBF_SELCHANGE: combobox_state_flags = combobox_state_flags_CBF_SELCHANGE; // island enum
pub const CBS_CHECKEDDISABLED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDDISABLED; // island enum
pub const CBS_CHECKEDHOT: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDHOT; // island enum
pub const CBS_CHECKEDNORMAL: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDNORMAL; // island enum
pub const CBS_CHECKEDPRESSED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDPRESSED; // island enum
pub const CBS_MIXEDDISABLED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_MIXEDDISABLED; // island enum
pub const CBS_MIXEDHOT: CHECKBOXSTATES = CHECKBOXSTATES_CBS_MIXEDHOT; // island enum
pub const CBS_MIXEDNORMAL: CHECKBOXSTATES = CHECKBOXSTATES_CBS_MIXEDNORMAL; // island enum
pub const CBS_MIXEDPRESSED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_MIXEDPRESSED; // island enum
pub const CBS_UNCHECKEDDISABLED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDDISABLED; // island enum
pub const CBS_UNCHECKEDHOT: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDHOT; // island enum
pub const CBS_UNCHECKEDNORMAL: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDNORMAL; // island enum
pub const CBS_UNCHECKEDPRESSED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDPRESSED; // island enum
pub const CBXS_DISABLED: COMBOBOXSTYLESTATES = COMBOBOXSTYLESTATES_CBXS_DISABLED; // island enum
pub const CBXS_HOT: COMBOBOXSTYLESTATES = COMBOBOXSTYLESTATES_CBXS_HOT; // island enum
pub const CBXS_NORMAL: COMBOBOXSTYLESTATES = COMBOBOXSTYLESTATES_CBXS_NORMAL; // island enum
pub const CBXS_PRESSED: COMBOBOXSTYLESTATES = COMBOBOXSTYLESTATES_CBXS_PRESSED; // island enum
pub const CHEVS_HOT: CHEVRONSTATES = CHEVRONSTATES_CHEVS_HOT; // island enum
pub const CHEVS_NORMAL: CHEVRONSTATES = CHEVRONSTATES_CHEVS_NORMAL; // island enum
pub const CHEVS_PRESSED: CHEVRONSTATES = CHEVRONSTATES_CHEVS_PRESSED; // island enum
pub const CLASS_E_NOAGGREGATION: HRESULT = 0x80040110u32 as HRESULT; // ole32-rust/src/hresults.rs
pub const CP_DROPDOWNBUTTON: COMBOBOXPARTS = COMBOBOXPARTS_CP_DROPDOWNBUTTON; // island enum
pub const CW_USEDEFAULT: INT = 0x8000_0000u32 as INT; // explorer-sys/src/server.rs
pub const DNS_DISABLED: DOWNSTATES = DOWNSTATES_DNS_DISABLED; // island enum
pub const DNS_HOT: DOWNSTATES = DOWNSTATES_DNS_HOT; // island enum
pub const DNS_NORMAL: DOWNSTATES = DOWNSTATES_DNS_NORMAL; // island enum
pub const DNS_PRESSED: DOWNSTATES = DOWNSTATES_DNS_PRESSED; // island enum
pub const EP_EDITTEXT: EDITPARTS = EDITPARTS_EP_EDITTEXT; // island enum
pub const ETS_DISABLED: EDITTEXTSTATES = EDITTEXTSTATES_ETS_DISABLED; // island enum
pub const ETS_FOCUSED: EDITTEXTSTATES = EDITTEXTSTATES_ETS_FOCUSED; // island enum
pub const ETS_NORMAL: EDITTEXTSTATES = EDITTEXTSTATES_ETS_NORMAL; // island enum
pub const ETS_READONLY: EDITTEXTSTATES = EDITTEXTSTATES_ETS_READONLY; // island enum
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT; // desk-rust/src/prelude.rs
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // appwiz-rust/src/prelude.rs
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT; // desk-rust/src/prelude.rs
pub const GBS_DISABLED: GROUPBOXSTATES = GROUPBOXSTATES_GBS_DISABLED; // island enum
pub const GBS_NORMAL: GROUPBOXSTATES = GROUPBOXSTATES_GBS_NORMAL; // island enum
pub const GLPS_CLOSED: GLYPHSTATES = GLYPHSTATES_GLPS_CLOSED; // island enum
pub const GLPS_OPENED: GLYPHSTATES = GLYPHSTATES_GLPS_OPENED; // island enum
pub const HIS_HOT: HEADERITEMSTATES = HEADERITEMSTATES_HIS_HOT; // island enum
pub const HIS_NORMAL: HEADERITEMSTATES = HEADERITEMSTATES_HIS_NORMAL; // island enum
pub const HIS_PRESSED: HEADERITEMSTATES = HEADERITEMSTATES_HIS_PRESSED; // island enum
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // appwiz-rust/src/prelude.rs
pub const HP_HEADERITEM: HEADERPARTS = HEADERPARTS_HP_HEADERITEM; // island enum
pub const HWND_TOP: HWND = 0 as HWND; // syssetup-rust/src/prelude.rs
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR; // desk-rust/src/prelude.rs
pub const IDC_HAND: LPCWSTR = 32649 as LPCWSTR; // shell32-rust/src/dialogs/dialogs.rs
pub const ILC_SYSTEM: u32 = 256; // shell32-rust/src/island_amd64.rs
pub const INT_MAX: u32 = 2147483647; // msv1_0-rust/src/types_amd64.rs
pub const LIM_LARGE: _LI_METRIC = _LI_METRIC_LIM_LARGE; // island enum
pub const LIM_SMALL: _LI_METRIC = _LI_METRIC_LIM_SMALL; // island enum
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400; // explorer-sys/src/server.rs
pub const NM_NCHITTEST: UINT = 0xFFFF_FFF2; // explorer-sys/src/lib.rs
pub const OBJID_CLIENT: u32 = 4294967292; // desk-rust/src/types_amd64.rs
pub const OBM_COMBO: u32 = 32738; // usetup-rust/src/types_amd64.rs
pub const PBS_DEFAULTED: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_DEFAULTED; // island enum
pub const PBS_DISABLED: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_DISABLED; // island enum
pub const PBS_HOT: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_HOT; // island enum
pub const PBS_NORMAL: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_NORMAL; // island enum
pub const PBS_PRESSED: PUSHBUTTONSTATES = PUSHBUTTONSTATES_PBS_PRESSED; // island enum
pub const PP_BAR: PROGRESSPARTS = PROGRESSPARTS_PP_BAR; // island enum
pub const PP_BARVERT: PROGRESSPARTS = PROGRESSPARTS_PP_BARVERT; // island enum
pub const PP_CHUNK: PROGRESSPARTS = PROGRESSPARTS_PP_CHUNK; // island enum
pub const PP_CHUNKVERT: PROGRESSPARTS = PROGRESSPARTS_PP_CHUNKVERT; // island enum
pub const RBS_CHECKEDDISABLED: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_CHECKEDDISABLED; // island enum
pub const RBS_CHECKEDHOT: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_CHECKEDHOT; // island enum
pub const RBS_CHECKEDNORMAL: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_CHECKEDNORMAL; // island enum
pub const RBS_CHECKEDPRESSED: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_CHECKEDPRESSED; // island enum
pub const RBS_UNCHECKEDDISABLED: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_UNCHECKEDDISABLED; // island enum
pub const RBS_UNCHECKEDHOT: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_UNCHECKEDHOT; // island enum
pub const RBS_UNCHECKEDNORMAL: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_UNCHECKEDNORMAL; // island enum
pub const RBS_UNCHECKEDPRESSED: RADIOBUTTONSTATES = RADIOBUTTONSTATES_RBS_UNCHECKEDPRESSED; // island enum
pub const RP_BAND: REBARPARTS = REBARPARTS_RP_BAND; // island enum
pub const RP_CHEVRON: REBARPARTS = REBARPARTS_RP_CHEVRON; // island enum
pub const RP_GRIPPER: REBARPARTS = REBARPARTS_RP_GRIPPER; // island enum
pub const RP_GRIPPERVERT: REBARPARTS = REBARPARTS_RP_GRIPPERVERT; // island enum
pub const RT_DIALOG: usize = 5; // oleaut32-rust/src/tierd_consts.rs
pub const RT_STRING: LPCWSTR = 6usize as LPCWSTR; // desk-rust/src/prelude.rs
pub const SBP_ARROWBTN: SCROLLBARPARTS = SCROLLBARPARTS_SBP_ARROWBTN; // island enum
pub const SBP_GRIPPERHORZ: SCROLLBARPARTS = SCROLLBARPARTS_SBP_GRIPPERHORZ; // island enum
pub const SBP_GRIPPERVERT: SCROLLBARPARTS = SCROLLBARPARTS_SBP_GRIPPERVERT; // island enum
pub const SBP_LOWERTRACKHORZ: SCROLLBARPARTS = SCROLLBARPARTS_SBP_LOWERTRACKHORZ; // island enum
pub const SBP_LOWERTRACKVERT: SCROLLBARPARTS = SCROLLBARPARTS_SBP_LOWERTRACKVERT; // island enum
pub const SBP_SIZEBOX: SCROLLBARPARTS = SCROLLBARPARTS_SBP_SIZEBOX; // island enum
pub const SBP_THUMBBTNHORZ: SCROLLBARPARTS = SCROLLBARPARTS_SBP_THUMBBTNHORZ; // island enum
pub const SBP_THUMBBTNVERT: SCROLLBARPARTS = SCROLLBARPARTS_SBP_THUMBBTNVERT; // island enum
pub const SBP_UPPERTRACKHORZ: SCROLLBARPARTS = SCROLLBARPARTS_SBP_UPPERTRACKHORZ; // island enum
pub const SBP_UPPERTRACKVERT: SCROLLBARPARTS = SCROLLBARPARTS_SBP_UPPERTRACKVERT; // island enum
pub const SCRBS_DISABLED: SCROLLBARSTYLESTATES = SCROLLBARSTYLESTATES_SCRBS_DISABLED; // island enum
pub const SCRBS_HOT: SCROLLBARSTYLESTATES = SCROLLBARSTYLESTATES_SCRBS_HOT; // island enum
pub const SCRBS_NORMAL: SCROLLBARSTYLESTATES = SCROLLBARSTYLESTATES_SCRBS_NORMAL; // island enum
pub const SPNP_DOWN: SPINPARTS = SPINPARTS_SPNP_DOWN; // island enum
pub const SPNP_DOWNHORZ: SPINPARTS = SPINPARTS_SPNP_DOWNHORZ; // island enum
pub const SPNP_UP: SPINPARTS = SPINPARTS_SPNP_UP; // island enum
pub const SPNP_UPHORZ: SPINPARTS = SPINPARTS_SPNP_UPHORZ; // island enum
pub const SP_GRIPPER: STATUSPARTS = STATUSPARTS_SP_GRIPPER; // island enum
pub const SP_GRIPPERPANE: STATUSPARTS = STATUSPARTS_SP_GRIPPERPANE; // island enum
pub const SP_PANE: STATUSPARTS = STATUSPARTS_SP_PANE; // island enum
pub const STREAM_SEEK_CUR: DWORD = 1; // explorer-sys/src/lib.rs
pub const STREAM_SEEK_SET: DWORD = 0; // explorer-sys/src/lib.rs
pub const SZB_RIGHTALIGN: SIZEBOXSTATES = SIZEBOXSTATES_SZB_RIGHTALIGN; // island enum
pub const SZB_TOPLEFTALIGN: SIZEBOXSTATES = SIZEBOXSTATES_SZB_TOPLEFTALIGN; // island enum
pub const S_FALSE: HRESULT = 1; // explorer-sys/src/lib.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const TABP_PANE: TABPARTS = TABPARTS_TABP_PANE; // island enum
pub const TABP_TABITEM: TABPARTS = TABPARTS_TABP_TABITEM; // island enum
pub const TABP_TABITEMBOTHEDGE: TABPARTS = TABPARTS_TABP_TABITEMBOTHEDGE; // island enum
pub const TABP_TABITEMLEFTEDGE: TABPARTS = TABPARTS_TABP_TABITEMLEFTEDGE; // island enum
pub const TABP_TABITEMRIGHTEDGE: TABPARTS = TABPARTS_TABP_TABITEMRIGHTEDGE; // island enum
pub const TABP_TOPTABITEM: TABPARTS = TABPARTS_TABP_TOPTABITEM; // island enum
pub const TABP_TOPTABITEMBOTHEDGE: TABPARTS = TABPARTS_TABP_TOPTABITEMBOTHEDGE; // island enum
pub const TABP_TOPTABITEMLEFTEDGE: TABPARTS = TABPARTS_TABP_TOPTABITEMLEFTEDGE; // island enum
pub const TABP_TOPTABITEMRIGHTEDGE: TABPARTS = TABPARTS_TABP_TOPTABITEMRIGHTEDGE; // island enum
pub const TDCBF_CANCEL_BUTTON: _TASKDIALOG_COMMON_BUTTON_FLAGS = _TASKDIALOG_COMMON_BUTTON_FLAGS_TDCBF_CANCEL_BUTTON; // island enum
pub const TDCBF_CLOSE_BUTTON: _TASKDIALOG_COMMON_BUTTON_FLAGS = _TASKDIALOG_COMMON_BUTTON_FLAGS_TDCBF_CLOSE_BUTTON; // island enum
pub const TDCBF_NO_BUTTON: _TASKDIALOG_COMMON_BUTTON_FLAGS = _TASKDIALOG_COMMON_BUTTON_FLAGS_TDCBF_NO_BUTTON; // island enum
pub const TDCBF_OK_BUTTON: _TASKDIALOG_COMMON_BUTTON_FLAGS = _TASKDIALOG_COMMON_BUTTON_FLAGS_TDCBF_OK_BUTTON; // island enum
pub const TDCBF_RETRY_BUTTON: _TASKDIALOG_COMMON_BUTTON_FLAGS = _TASKDIALOG_COMMON_BUTTON_FLAGS_TDCBF_RETRY_BUTTON; // island enum
pub const TDCBF_YES_BUTTON: _TASKDIALOG_COMMON_BUTTON_FLAGS = _TASKDIALOG_COMMON_BUTTON_FLAGS_TDCBF_YES_BUTTON; // island enum
pub const TDE_CONTENT: _TASKDIALOG_ELEMENTS = _TASKDIALOG_ELEMENTS_TDE_CONTENT; // island enum
pub const TDE_EXPANDED_INFORMATION: _TASKDIALOG_ELEMENTS = _TASKDIALOG_ELEMENTS_TDE_EXPANDED_INFORMATION; // island enum
pub const TDE_FOOTER: _TASKDIALOG_ELEMENTS = _TASKDIALOG_ELEMENTS_TDE_FOOTER; // island enum
pub const TDE_MAIN_INSTRUCTION: _TASKDIALOG_ELEMENTS = _TASKDIALOG_ELEMENTS_TDE_MAIN_INSTRUCTION; // island enum
pub const TDF_ALLOW_DIALOG_CANCELLATION: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_ALLOW_DIALOG_CANCELLATION; // island enum
pub const TDF_CALLBACK_TIMER: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_CALLBACK_TIMER; // island enum
pub const TDF_CAN_BE_MINIMIZED: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_CAN_BE_MINIMIZED; // island enum
pub const TDF_ENABLE_HYPERLINKS: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_ENABLE_HYPERLINKS; // island enum
pub const TDF_EXPANDED_BY_DEFAULT: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_EXPANDED_BY_DEFAULT; // island enum
pub const TDF_EXPAND_FOOTER_AREA: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_EXPAND_FOOTER_AREA; // island enum
pub const TDF_NO_DEFAULT_RADIO_BUTTON: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_NO_DEFAULT_RADIO_BUTTON; // island enum
pub const TDF_NO_SET_FOREGROUND: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_NO_SET_FOREGROUND; // island enum
pub const TDF_POSITION_RELATIVE_TO_WINDOW: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_POSITION_RELATIVE_TO_WINDOW; // island enum
pub const TDF_RTL_LAYOUT: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_RTL_LAYOUT; // island enum
pub const TDF_SHOW_MARQUEE_PROGRESS_BAR: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_SHOW_MARQUEE_PROGRESS_BAR; // island enum
pub const TDF_SHOW_PROGRESS_BAR: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_SHOW_PROGRESS_BAR; // island enum
pub const TDF_SIZE_TO_CONTENT: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_SIZE_TO_CONTENT; // island enum
pub const TDF_USE_COMMAND_LINKS: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_USE_COMMAND_LINKS; // island enum
pub const TDF_USE_COMMAND_LINKS_NO_ICON: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_USE_COMMAND_LINKS_NO_ICON; // island enum
pub const TDF_USE_HICON_FOOTER: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_USE_HICON_FOOTER; // island enum
pub const TDF_USE_HICON_MAIN: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_USE_HICON_MAIN; // island enum
pub const TDF_VERIFICATION_FLAG_CHECKED: _TASKDIALOG_FLAGS = _TASKDIALOG_FLAGS_TDF_VERIFICATION_FLAG_CHECKED; // island enum
pub const TDIE_ICON_FOOTER: _TASKDIALOG_ICON_ELEMENTS = _TASKDIALOG_ICON_ELEMENTS_TDIE_ICON_FOOTER; // island enum
pub const TDIE_ICON_MAIN: _TASKDIALOG_ICON_ELEMENTS = _TASKDIALOG_ICON_ELEMENTS_TDIE_ICON_MAIN; // island enum
pub const TDM_CLICK_BUTTON: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_CLICK_BUTTON; // island enum
pub const TDM_CLICK_RADIO_BUTTON: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_CLICK_RADIO_BUTTON; // island enum
pub const TDM_CLICK_VERIFICATION: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_CLICK_VERIFICATION; // island enum
pub const TDM_ENABLE_BUTTON: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_ENABLE_BUTTON; // island enum
pub const TDM_ENABLE_RADIO_BUTTON: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_ENABLE_RADIO_BUTTON; // island enum
pub const TDM_NAVIGATE_PAGE: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_NAVIGATE_PAGE; // island enum
pub const TDM_SET_BUTTON_ELEVATION_REQUIRED_STATE: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_SET_BUTTON_ELEVATION_REQUIRED_STATE; // island enum
pub const TDM_SET_ELEMENT_TEXT: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_SET_ELEMENT_TEXT; // island enum
pub const TDM_SET_MARQUEE_PROGRESS_BAR: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_SET_MARQUEE_PROGRESS_BAR; // island enum
pub const TDM_SET_PROGRESS_BAR_MARQUEE: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_SET_PROGRESS_BAR_MARQUEE; // island enum
pub const TDM_SET_PROGRESS_BAR_POS: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_SET_PROGRESS_BAR_POS; // island enum
pub const TDM_SET_PROGRESS_BAR_RANGE: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_SET_PROGRESS_BAR_RANGE; // island enum
pub const TDM_SET_PROGRESS_BAR_STATE: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_SET_PROGRESS_BAR_STATE; // island enum
pub const TDM_UPDATE_ELEMENT_TEXT: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_UPDATE_ELEMENT_TEXT; // island enum
pub const TDM_UPDATE_ICON: _TASKDIALOG_MESSAGES = _TASKDIALOG_MESSAGES_TDM_UPDATE_ICON; // island enum
pub const TDN_BUTTON_CLICKED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_BUTTON_CLICKED; // island enum
pub const TDN_CREATED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_CREATED; // island enum
pub const TDN_DESTROYED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_DESTROYED; // island enum
pub const TDN_DIALOG_CONSTRUCTED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_DIALOG_CONSTRUCTED; // island enum
pub const TDN_EXPANDO_BUTTON_CLICKED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_EXPANDO_BUTTON_CLICKED; // island enum
pub const TDN_HELP: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_HELP; // island enum
pub const TDN_HYPERLINK_CLICKED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_HYPERLINK_CLICKED; // island enum
pub const TDN_NAVIGATED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_NAVIGATED; // island enum
pub const TDN_RADIO_BUTTON_CLICKED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_RADIO_BUTTON_CLICKED; // island enum
pub const TDN_TIMER: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_TIMER; // island enum
pub const TDN_VERIFICATION_CLICKED: _TASKDIALOG_NOTIFICATIONS = _TASKDIALOG_NOTIFICATIONS_TDN_VERIFICATION_CLICKED; // island enum
pub const TIS_FOCUSED: TABITEMSTATES = TABITEMSTATES_TIS_FOCUSED; // island enum
pub const TIS_HOT: TABITEMSTATES = TABITEMSTATES_TIS_HOT; // island enum
pub const TIS_NORMAL: TABITEMSTATES = TABITEMSTATES_TIS_NORMAL; // island enum
pub const TIS_SELECTED: TABITEMSTATES = TABITEMSTATES_TIS_SELECTED; // island enum
pub const TKP_THUMB: TRACKBARPARTS = TRACKBARPARTS_TKP_THUMB; // island enum
pub const TKP_THUMBBOTTOM: TRACKBARPARTS = TRACKBARPARTS_TKP_THUMBBOTTOM; // island enum
pub const TKP_THUMBLEFT: TRACKBARPARTS = TRACKBARPARTS_TKP_THUMBLEFT; // island enum
pub const TKP_THUMBRIGHT: TRACKBARPARTS = TRACKBARPARTS_TKP_THUMBRIGHT; // island enum
pub const TKP_THUMBTOP: TRACKBARPARTS = TRACKBARPARTS_TKP_THUMBTOP; // island enum
pub const TKP_THUMBVERT: TRACKBARPARTS = TRACKBARPARTS_TKP_THUMBVERT; // island enum
pub const TKP_TICS: TRACKBARPARTS = TRACKBARPARTS_TKP_TICS; // island enum
pub const TKP_TICSVERT: TRACKBARPARTS = TRACKBARPARTS_TKP_TICSVERT; // island enum
pub const TKP_TRACK: TRACKBARPARTS = TRACKBARPARTS_TKP_TRACK; // island enum
pub const TKP_TRACKVERT: TRACKBARPARTS = TRACKBARPARTS_TKP_TRACKVERT; // island enum
pub const TKS_NORMAL: TRACKBARSTYLESTATES = TRACKBARSTYLESTATES_TKS_NORMAL; // island enum
pub const TSS_NORMAL: TICSSTATES = TICSSTATES_TSS_NORMAL; // island enum
pub const TS_DRAW: THEMESIZE = THEMESIZE_TS_DRAW; // island enum
pub const TUS_DISABLED: THUMBSTATES = THUMBSTATES_TUS_DISABLED; // island enum
pub const TUS_HOT: THUMBSTATES = THUMBSTATES_TUS_HOT; // island enum
pub const TUS_NORMAL: THUMBSTATES = THUMBSTATES_TUS_NORMAL; // island enum
pub const TUS_PRESSED: THUMBSTATES = THUMBSTATES_TUS_PRESSED; // island enum
pub const TVP_GLYPH: TREEVIEWPARTS = TREEVIEWPARTS_TVP_GLYPH; // island enum
pub const UINT_MAX: u32 = 4294967295; // msv1_0-rust/src/types_amd64.rs
extern "C" {
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> core::ffi::c_uchar; // rtl-diag/src/shim.rs
    pub fn _assert(message: *const i8, file: *const i8, line: u32); // crt-shared-rust/src/ucrt_tls.rs
    pub fn abs(x: i32) -> i32; // cmlib-rust/src/cmcheck.rs
    pub fn atoi(s: PCSTR) -> i32; // freeldr-rust/src/custom.rs
    pub fn floor(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn isalnum(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isdigit(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>); // ucrt-rust/src/stdlib/qsort.rs
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char; // crt-shared-rust/src/prelude.rs
    pub fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8; // cabinet-rust/src/cabinet/cabinet_main.rs
    pub fn strlen(s: *const i8) -> usize; // advapi32-rust/src/prelude.rs
    pub fn towupper(c: c_int) -> c_int; // dnsapi-rust/src/prelude.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscspn(s: *const u16, set: *const u16) -> usize; // kernel32-rust/src/prelude.rs
    pub fn wcsicmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // acpi-rust/src/buspdo.rs
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // inflib-rust/src/infget.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
}
extern "system" {
    pub fn ActivateActCtx(arg1: HANDLE, arg2: *mut ULONG_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn AdjustWindowRect(arg1: LPRECT, arg2: DWORD, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn AdjustWindowRectEx(arg1: LPRECT, arg2: DWORD, arg3: BOOL, arg4: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn AppendMenuW(arg1: HMENU, arg2: UINT, arg3: UINT_PTR, arg4: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn BeginDeferWindowPos(arg1: ::core::ffi::c_int) -> HDWP; // aclui-rust/src/types_amd64.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CallWindowProcA( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CallWindowProcW( arg1: WNDPROC, arg2: HWND, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CharLowerBuffW(lpsz: LPWSTR, cchLength: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn CharLowerW(arg1: LPWSTR) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn CharNextA(arg1: LPCSTR) -> LPSTR; // aclui-rust/src/types_amd64.rs
    pub fn CharUpperBuffW(lpsz: LPWSTR, cchLength: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn CharUpperW(arg1: LPWSTR) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseClipboard() -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CloseThemeData(arg1: HTHEME) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CombineRgn( hrgnDest: HRGN, hrgnSrc1: HRGN, hrgnSrc2: HRGN, fnCombineMode: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn CompareFileTime(arg1: *const FILETIME, arg2: *const FILETIME) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn CompareStringA( Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCSTR, cchCount1: ::core::ffi::c_int, lpString2: LPCSTR, cchCount2: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn CompareStringW( Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCWSTR, cchCount1: ::core::ffi::c_int, lpString2: LPCWSTR, cchCount2: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn CopyIcon(arg1: HICON) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn CopyImage( arg1: HANDLE, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CopyRect(arg1: LPRECT, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateActCtxW(arg1: PCACTCTXW) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateBitmap( cx: INT, cy: INT, cPlanes: UINT, cBitsPerPel: UINT, pvBits: *const ::core::ffi::c_void, ) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateCaret( arg1: HWND, arg2: HBITMAP, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CreateDIBSection( hdc: HDC, pbmi: *const BITMAPINFO, usage: UINT, ppvBits: *mut *mut ::core::ffi::c_void, hSection: HANDLE, offset: DWORD, ) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateDialogIndirectParamA( arg1: HINSTANCE, arg2: LPCDLGTEMPLATE, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn CreateDialogIndirectParamW( arg1: HINSTANCE, arg2: LPCDLGTEMPLATE, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn CreateDirectoryW( lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_amd64.rs
    pub fn CreateIconIndirect(arg1: PICONINFO) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn CreatePatternBrush(arg1: HBITMAP) -> HBRUSH; // aclui-rust/src/types_amd64.rs
    pub fn CreatePen(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: COLORREF) -> HPEN; // aclui-rust/src/types_amd64.rs
    pub fn CreatePenIndirect(arg1: *const LOGPEN) -> HPEN; // aclui-rust/src/types_amd64.rs
    pub fn CreatePolygonRgn( pptl: *const POINT, cPoint: ::core::ffi::c_int, iMode: ::core::ffi::c_int, ) -> HRGN; // aclui-rust/src/types_amd64.rs
    pub fn CreatePopupMenu() -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn CreateRectRgn( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> HRGN; // aclui-rust/src/types_amd64.rs
    pub fn CreateRoundRectRgn( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, ) -> HRGN; // aclui-rust/src/types_amd64.rs
    pub fn CreateSolidBrush(arg1: COLORREF) -> HBRUSH; // aclui-rust/src/types_amd64.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn DPtoLP(hdc: HDC, lppt: LPPOINT, c: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeactivateActCtx(arg1: DWORD, arg2: ULONG_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DeferWindowPos( arg1: HDWP, arg2: HWND, arg3: HWND, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: UINT, ) -> HDWP; // aclui-rust/src/types_amd64.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyCaret() -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyCursor(arg1: HCURSOR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxIndirectParamW( arg1: HINSTANCE, arg2: LPCDLGTEMPLATE, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DragDetect(arg1: HWND, arg2: POINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawEdge(arg1: HDC, arg2: LPRECT, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawFocusRect(arg1: HDC, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawFrameControl(arg1: HDC, arg2: LPRECT, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawIcon( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: HICON, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawIconEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: HICON, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, arg8: HBRUSH, arg9: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawStateW( arg1: HDC, arg2: HBRUSH, arg3: DRAWSTATEPROC, arg4: LPARAM, arg5: WPARAM, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn DrawThemeBackground( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *const RECT, arg6: *const RECT, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DrawThemeEdge( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *const RECT, arg6: UINT, arg7: UINT, arg8: *mut RECT, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DrawThemeParentBackground(arg1: HWND, arg2: HDC, arg3: *mut RECT) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DrawThemeText( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: LPCWSTR, arg6: ::core::ffi::c_int, arg7: DWORD, arg8: DWORD, arg9: *const RECT, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn EmptyClipboard() -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnableScrollBar(arg1: HWND, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnableThemeDialogTexture(arg1: HWND, arg2: DWORD) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDeferWindowPos(arg1: HDWP) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn EnumChildWindows(arg1: HWND, arg2: WNDENUMPROC, arg3: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EqualRect(arg1: LPCRECT, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ExcludeClipRect( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ExtCreatePen( iPenStyle: DWORD, cWidth: DWORD, plbrush: *const LOGBRUSH, cStyle: DWORD, pstyle: *const DWORD, ) -> HPEN; // aclui-rust/src/types_amd64.rs
    pub fn ExtCreateRegion(lpx: *const XFORM, nCount: DWORD, lpData: *const RGNDATA) -> HRGN; // aclui-rust/src/types_amd64.rs
    pub fn ExtTextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, options: UINT, lprect: *const RECT, lpString: LPCWSTR, c: UINT, lpDx: *const INT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FillRgn(arg1: HDC, arg2: HRGN, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindResourceA(arg1: HMODULE, arg2: LPCSTR, arg3: LPCSTR) -> HRSRC; // aclui-rust/src/types_amd64.rs
    pub fn FindResourceW(arg1: HMODULE, arg2: LPCWSTR, arg3: LPCWSTR) -> HRSRC; // aclui-rust/src/types_amd64.rs
    pub fn FrameRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FrameRgn( arg1: HDC, arg2: HRGN, arg3: HBRUSH, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FreeResource(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GdiAlphaBlend( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HDC, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: ::core::ffi::c_int, arg11: BLENDFUNCTION, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GdiFlush() -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GdiGetCharDimensions(arg1: HDC, arg2: LPTEXTMETRICW, arg3: *mut LONG) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn GdiGetCodePage(arg1: HDC) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetActiveWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetAsyncKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_amd64.rs
    pub fn GetBkColor(arg1: HDC) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn GetBkMode(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetCapture() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetCharABCWidthsW(hdc: HDC, wFirst: UINT, wLast: UINT, lpABC: LPABC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCharWidthW(hdc: HDC, iFirst: UINT, iLast: UINT, lpBuffer: LPINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClassInfoExW(arg1: HINSTANCE, arg2: LPCWSTR, arg3: LPWNDCLASSEXW) -> BOOL; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetClassLongW")]
    pub fn GetClassLongPtrW(hWnd: HWND, nIndex: INT) -> ULONG_PTR; // user32-rust/src/controls/appswitch.rs
    pub fn GetClassNameW( hWnd: HWND, lpClassName: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClipBox(arg1: HDC, arg2: LPRECT) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetClipRgn(arg1: HDC, arg2: HRGN) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetClipboardData(arg1: UINT) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentObject(arg1: HDC, arg2: UINT) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentThreadId() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetDCEx(arg1: HWND, arg2: HRGN, arg3: DWORD) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetDIBColorTable(hdc: HDC, iStart: UINT, cEntries: UINT, prgbq: *mut RGBQUAD) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetDIBits( hdc: HDC, hbm: HBITMAP, start: UINT, cLines: UINT, lpvBits: LPVOID, lpbmi: LPBITMAPINFO, usage: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetDateFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetDeviceCaps(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgCtrlID(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetDoubleClickTime() -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetDriveTypeW(arg1: LPCWSTR) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetFocus() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetIconInfo(arg1: HICON, arg2: PICONINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyNameTextW( lParam: LONG, lpString: LPWSTR, cchSize: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardLayout(arg1: DWORD) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLayout(arg1: HDC) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLocalTime(arg1: LPSYSTEMTIME); // aclui-rust/src/types_amd64.rs
    pub fn GetLocaleInfoW( Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn GetMenuState(arg1: HMENU, arg2: UINT, arg3: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetMessagePos() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetMessageTime() -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetModuleHandleA(arg1: LPCSTR) -> HMODULE; // aclui-rust/src/types_amd64.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_amd64.rs
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetNearestColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn GetNextDlgGroupItem(arg1: HWND, arg2: HWND, arg3: BOOL) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetNextDlgTabItem(arg1: HWND, arg2: HWND, arg3: BOOL) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetObjectType(arg1: HGDIOBJ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetPixel(arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetPropW(arg1: HWND, arg2: LPCWSTR) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetScrollBarInfo(arg1: HWND, arg2: LONG, arg3: PSCROLLBARINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetScrollInfo(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPSCROLLINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetScrollPos(arg1: HWND, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetScrollRange(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPINT, arg4: LPINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetSysColorBrush(arg1: ::core::ffi::c_int) -> HBRUSH; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMenu(arg1: HWND, arg2: BOOL) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetTextCharsetInfo(arg1: HDC, arg2: LPFONTSIGNATURE, arg3: DWORD) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetTextColor(arg1: HDC) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn GetTextExtentExPointW( hdc: HDC, lpszString: LPCWSTR, cchString: ::core::ffi::c_int, nMaxExtent: ::core::ffi::c_int, lpnFit: LPINT, lpnDx: LPINT, lpSize: LPSIZE, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetTextExtentPoint32A( hdc: HDC, lpString: LPCSTR, c: ::core::ffi::c_int, psizl: LPSIZE, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetTextExtentPoint32W( hdc: HDC, lpString: LPCWSTR, c: ::core::ffi::c_int, psizl: LPSIZE, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetTextExtentPointA( hdc: HDC, lpString: LPCSTR, c: ::core::ffi::c_int, lpsz: LPSIZE, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetTextExtentPointW( hdc: HDC, lpString: LPCWSTR, c: ::core::ffi::c_int, lpsz: LPSIZE, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetTextMetricsW(arg1: HDC, arg2: LPTEXTMETRICW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetThemeBackgroundContentRect( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *const RECT, arg6: *mut RECT, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThemeBackgroundExtent( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *const RECT, arg6: *mut RECT, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThemeColor( arg1: HTHEME, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *mut COLORREF, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThemeFont( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: *mut LOGFONTW, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThemeInt( arg1: HTHEME, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *mut ::core::ffi::c_int, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThemeMargins( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: *mut RECT, arg7: *mut MARGINS, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThemePartSize( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: *mut RECT, arg6: THEMESIZE, arg7: *mut SIZE, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThemeTextExtent( arg1: HTHEME, arg2: HDC, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: LPCWSTR, arg6: ::core::ffi::c_int, arg7: DWORD, arg8: *const RECT, arg9: *mut RECT, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn GetThreadLocale() -> LCID; // desk-rust/src/types_amd64.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetTopWindow(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetUserDefaultLCID() -> LCID; // desk-rust/src/types_amd64.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowRgn(arg1: HWND, arg2: HRGN) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowTextA( hWnd: HWND, lpString: LPSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowTextLengthA(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowTheme(arg1: HWND) -> HTHEME; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GlobalAddAtomW(arg1: LPCWSTR) -> ATOM; // aclui-rust/src/types_amd64.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn GlobalDeleteAtom(arg1: ATOM) -> ATOM; // aclui-rust/src/types_amd64.rs
    pub fn GlobalFree(arg1: HGLOBAL) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_amd64.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn InflateRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn InternalGetWindowText( hWnd: HWND, pString: LPWSTR, cchMaxCount: ::core::ffi::c_int, ) -> INT; // aclui-rust/src/types_amd64.rs
    pub fn IntersectClipRect( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn IntersectRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRgn(arg1: HWND, arg2: HRGN, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InvertRect(arg1: HDC, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsBadCodePtr(arg1: FARPROC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsBadStringPtrA(arg1: LPCSTR, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsBadStringPtrW(arg1: LPCWSTR, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsBadWritePtr(arg1: PVOID, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsChild(arg1: HWND, arg2: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsClipboardFormatAvailable(arg1: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsDBCSLeadByte(arg1: BYTE) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn IsDialogMessageW(arg1: HWND, arg2: LPMSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsIconic(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsRectEmpty(arg1: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsThemeBackgroundPartiallyTransparent( arg1: HTHEME, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsValidLocale(arg1: LCID, arg2: DWORD) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsWindowEnabled(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsWindowUnicode(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn KillSystemTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn LineTo(arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadBitmapW(arg1: HINSTANCE, arg2: LPCWSTR) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_amd64.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadMenuA(arg1: HINSTANCE, arg2: LPCSTR) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn LoadResource(arg1: HINSTANCE, arg2: HRSRC) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalLock(arg1: HLOCAL) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn LocalReAlloc(arg1: HLOCAL, arg2: SIZE_T, arg3: UINT) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalSize(arg1: HLOCAL) -> SIZE_T; // aclui-rust/src/types_amd64.rs
    pub fn LocalUnlock(arg1: HLOCAL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LockResource(arg1: HGLOBAL) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn MapDialogRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MapVirtualKeyW(arg1: UINT, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn MapWindowPoints( hWndFrom: HWND, hWndTo: HWND, lpPoints: LPPOINT, cPoints: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MonitorFromRect(arg1: LPCRECT, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_amd64.rs
    pub fn MonitorFromWindow(arg1: HWND, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_amd64.rs
    pub fn MoveToEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPPOINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn OffsetWindowOrgEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPPOINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn OpenClipboard(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn OpenThemeData(arg1: HWND, arg2: LPCWSTR) -> HTHEME; // aclui-rust/src/types_amd64.rs
    pub fn PatBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PlayEnhMetaFile(arg1: HDC, arg2: HENHMETAFILE, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PolyPolyline(hdc: HDC, apt: *const POINT, asz: *const DWORD, csz: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn Polygon(hdc: HDC, apt: *const POINT, cpt: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_amd64.rs
    pub fn PtInRect(arg1: LPCRECT, arg2: POINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RectVisible(arg1: HDC, arg2: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn Rectangle( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RedrawWindow(arg1: HWND, arg2: LPCRECT, arg3: HRGN, arg4: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyA(arg1: HKEY, arg2: LPCSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExA( hKey: HKEY, lpValueName: LPCSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_amd64.rs
    pub fn RegisterClassW(arg1: *const WNDCLASSW) -> ATOM; // aclui-rust/src/types_amd64.rs
    pub fn RegisterWindowMessageW(arg1: LPCWSTR) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseCapture() -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn RemovePropW(arg1: HWND, arg2: LPCWSTR) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ScriptBreak(pwcChars: *const WCHAR, cChars: INT, psa: *const SCRIPT_ANALYSIS, psla: *mut SCRIPT_LOGATTR) -> HRESULT; // user32-rust/src/controls/edit.rs
    pub fn ScriptStringAnalyse(hdc: HDC, pString: *const c_void, cString: INT, cGlyphs: INT, iCharset: INT, dwFlags: DWORD, iReqWidth: INT, psControl: *mut c_void, psState: *mut c_void, piDx: *const INT, pTabdef: *mut SCRIPT_TABDEF, pbInClass: *const BYTE, pssa: *mut SCRIPT_STRING_ANALYSIS) -> HRESULT; // user32-rust/src/controls/edit.rs
    pub fn ScriptStringCPtoX(ssa: SCRIPT_STRING_ANALYSIS, icp: INT, fTrailing: BOOL, pX: *mut INT) -> HRESULT; // user32-rust/src/controls/edit.rs
    pub fn ScriptStringFree(pssa: *mut SCRIPT_STRING_ANALYSIS) -> HRESULT; // user32-rust/src/controls/edit.rs
    pub fn ScriptStringGetLogicalWidths(ssa: SCRIPT_STRING_ANALYSIS, piDx: *mut INT) -> HRESULT; // user32-rust/src/controls/edit.rs
    pub fn ScriptStringOut(ssa: SCRIPT_STRING_ANALYSIS, iX: INT, iY: INT, uOptions: UINT, prc: *const RECT, iMinSel: INT, iMaxSel: INT, fDisabled: BOOL) -> HRESULT; // user32-rust/src/controls/edit.rs
    pub fn ScriptStringXtoCP(ssa: SCRIPT_STRING_ANALYSIS, iX: INT, piCh: *mut INT, piTrailing: *mut INT) -> HRESULT; // user32-rust/src/controls/edit.rs
    pub fn ScriptString_pSize(ssa: SCRIPT_STRING_ANALYSIS) -> *const SIZE; // user32-rust/src/controls/edit.rs
    pub fn ScriptString_pcOutChars(ssa: SCRIPT_STRING_ANALYSIS) -> *const INT; // user32-rust/src/controls/edit.rs
    pub fn ScrollWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPCRECT, arg5: LPCRECT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ScrollWindowEx( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPCRECT, arg5: LPCRECT, arg6: HRGN, arg7: LPRECT, arg8: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SelectClipRgn(arg1: HDC, arg2: HRGN) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SelectPalette(arg1: HDC, arg2: HPALETTE, arg3: BOOL) -> HPALETTE; // aclui-rust/src/types_amd64.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetBkColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn SetBkMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetCapture(hWnd: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetCaretPos(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetClipboardData(arg1: UINT, arg2: HANDLE) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_amd64.rs
    pub fn SetDIBColorTable(hdc: HDC, iStart: UINT, cEntries: UINT, prgbq: *const RGBQUAD) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn SetDIBits( arg1: HDC, arg2: HBITMAP, arg3: UINT, arg4: UINT, arg5: *const ::core::ffi::c_void, arg6: *const BITMAPINFO, arg7: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn SetParent(arg1: HWND, arg2: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetPixel( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: COLORREF, ) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn SetPolyFillMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetPropW(arg1: HWND, arg2: LPCWSTR, arg3: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetRectEmpty(arg1: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetRectRgn( arg1: HRGN, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetScrollInfo( arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCSCROLLINFO, arg4: BOOL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetScrollPos( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: BOOL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetScrollRange( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetSystemTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn SetTextAlign(arg1: HDC, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn SetViewportOrgEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPPOINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongA")]
    pub fn SetWindowLongPtrA(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/wine/shellole.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LONG) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowOrgEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPPOINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowRgn(arg1: HWND, arg2: HRGN, arg3: BOOL) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowTextA(arg1: HWND, arg2: LPCSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowCaret(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowScrollBar(arg1: HWND, arg2: ::core::ffi::c_int, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SizeofResource(arg1: HINSTANCE, arg2: HRSRC) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn StretchBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HDC, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: ::core::ffi::c_int, arg11: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn StretchDIBits( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: *const ::core::ffi::c_void, arg11: *const BITMAPINFO, arg12: UINT, arg13: DWORD, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SubtractRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TabbedTextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, lpString: LPCWSTR, chCount: ::core::ffi::c_int, nTabPositions: ::core::ffi::c_int, lpnTabStopPositions: *const INT, nTabOrigin: ::core::ffi::c_int, ) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn TextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, lpString: LPCWSTR, c: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TrackMouseEvent(arg1: LPTRACKMOUSEEVENT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UnionRect(arg1: LPRECT, arg2: LPCRECT, arg3: LPCRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UnregisterClassW(arg1: LPCWSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn WindowFromPoint(arg1: POINT) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_amd64.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::comctl32::comctl32undoc::Alloc;
pub use crate::comctl32::button::BUTTON_Register;
pub use crate::comctl32::button::BUTTON_Unregister;
pub use crate::comctl32::listbox::COMBOLBOX_Register;
pub use crate::comctl32::listbox::COMBOLBOX_Unregister;
pub use crate::comctl32::combo::COMBO_Register;
pub use crate::comctl32::combo::COMBO_Unregister;
pub use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
pub use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
pub use crate::comctl32::datetime::DATETIME_Register;
pub use crate::comctl32::datetime::DATETIME_Unregister;
pub use crate::comctl32::commctrl::DefSubclassProc;
pub use crate::comctl32::edit::EDIT_Register;
pub use crate::comctl32::edit::EDIT_Unregister;
pub use crate::comctl32::flatsb::FLATSB_Register;
pub use crate::comctl32::flatsb::FLATSB_Unregister;
pub use crate::comctl32::comctl32undoc::Free;
pub use crate::comctl32::header::HEADER_Register;
pub use crate::comctl32::header::HEADER_Unregister;
pub use crate::comctl32::hotkey::HOTKEY_Register;
pub use crate::comctl32::hotkey::HOTKEY_Unregister;
pub use crate::comctl32::ipaddress::IPADDRESS_Register;
pub use crate::comctl32::ipaddress::IPADDRESS_Unregister;
pub use crate::comctl32::imagelist::ImageList_Add;
pub use crate::comctl32::imagelist::ImageList_BeginDrag;
pub use crate::comctl32::imagelist::ImageList_Create;
pub use crate::comctl32::imagelist::ImageList_Destroy;
pub use crate::comctl32::imagelist::ImageList_DragMove;
pub use crate::comctl32::imagelist::ImageList_DragShowNolock;
pub use crate::comctl32::imagelist::ImageList_Draw;
pub use crate::comctl32::imagelist::ImageList_DrawEx;
pub use crate::comctl32::imagelist::ImageList_EndDrag;
pub use crate::comctl32::imagelist::ImageList_GetIconSize;
pub use crate::comctl32::imagelist::ImageList_GetImageInfo;
pub use crate::comctl32::imagelist::ImageList_SetOverlayImage;
pub use crate::comctl32::listbox::LISTBOX_Register;
pub use crate::comctl32::listbox::LISTBOX_Unregister;
pub use crate::comctl32::listview::LISTVIEW_Register;
pub use crate::comctl32::listview::LISTVIEW_Unregister;
pub use crate::comctl32::monthcal::MONTHCAL_CalculateDayOfWeek;
pub use crate::comctl32::monthcal::MONTHCAL_CompareSystemTime;
pub use crate::comctl32::monthcal::MONTHCAL_MonthLength;
pub use crate::comctl32::monthcal::MONTHCAL_Register;
pub use crate::comctl32::monthcal::MONTHCAL_Unregister;
pub use crate::comctl32::nativefont::NATIVEFONT_Register;
pub use crate::comctl32::nativefont::NATIVEFONT_Unregister;
pub use crate::comctl32::pager::PAGER_Register;
pub use crate::comctl32::pager::PAGER_Unregister;
pub use crate::comctl32::progress::PROGRESS_Register;
pub use crate::comctl32::progress::PROGRESS_Unregister;
pub use crate::comctl32::rebar::REBAR_Register;
pub use crate::comctl32::rebar::REBAR_Unregister;
pub use crate::comctl32::commctrl::RemoveWindowSubclass;
pub use crate::comctl32::r#static::STATIC_Register;
pub use crate::comctl32::r#static::STATIC_Unregister;
pub use crate::comctl32::status::STATUS_Register;
pub use crate::comctl32::status::STATUS_Unregister;
pub use crate::comctl32::syslink::SYSLINK_Register;
pub use crate::comctl32::syslink::SYSLINK_Unregister;
pub use crate::comctl32::commctrl::SetPathWordBreakProc;
pub use crate::comctl32::commctrl::SetWindowSubclass;
pub use crate::comctl32::string::Str_GetPtrW;
pub use crate::comctl32::string::Str_SetPtrW;
pub use crate::comctl32::tab::TAB_Register;
pub use crate::comctl32::tab::TAB_Unregister;
pub use crate::comctl32::theming::THEMING_CallOriginalClass;
pub use crate::comctl32::theming::THEMING_Initialize;
pub use crate::comctl32::theming::THEMING_Uninitialize;
pub use crate::comctl32::tooltips::TOOLTIPS_Register;
pub use crate::comctl32::tooltips::TOOLTIPS_Unregister;
pub use crate::comctl32::trackbar::TRACKBAR_Register;
pub use crate::comctl32::trackbar::TRACKBAR_Unregister;
pub use crate::comctl32::treeview::TREEVIEW_Register;
pub use crate::comctl32::treeview::TREEVIEW_Unregister;
pub use crate::comctl32::updown::UPDOWN_Register;
pub use crate::comctl32::updown::UPDOWN_Unregister;
pub use crate::comctl32::button::set_control_clipping;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const FOURCC_MEM: DWORD = 0x204D454D; // clang, animate.c
pub const FOURCC_RIFF: DWORD = 0x46464952; // clang, animate.c
pub const HWND_DESKTOP: HWND = core::ptr::null_mut(); // clang, combo.c
pub const ICTYPE_VIDEO: DWORD = 0x63646976; // clang, animate.c
pub const IDC_IBEAM: ULONG_PTR = 0x7F01; // clang, edit.c
pub const IDC_NO: ULONG_PTR = 0x7F88; // clang, draglist.c
pub const IDC_SIZE: ULONG_PTR = 0x7F80; // clang, rebar.c
pub const IDC_SIZENS: ULONG_PTR = 0x7F85; // clang, rebar.c
pub const IDC_SIZEWE: ULONG_PTR = 0x7F84; // clang, rebar.c
pub const INT_MIN: i32 = 0x80000000_u32 as i32; // clang, datetime.c
pub const L_MAX_URL_LENGTH: u32 = 0x824; // clang, syslink.c
pub const MCHITTESTINFO_V1_SIZE: u32 = 0x20; // clang, monthcal.c
pub const NM_CLICK: u32 = 0xFFFFFFFE; // clang, listview.c
pub const NM_KEYDOWN: u32 = 0xFFFFFFF1; // clang, toolbar.c
pub const NM_LDOWN: u32 = 0xFFFFFFEC; // clang, toolbar.c
pub const NM_CUSTOMDRAW: u32 = 0xFFFFFFF4; // clang, button.c
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // clang, listview.c
pub const NM_HOVER: u32 = 0xFFFFFFF3; // clang, listview.c
pub const NM_KILLFOCUS: u32 = 0xFFFFFFF8; // clang, datetime.c
pub const NM_OUTOFMEMORY: u32 = 0xFFFFFFFF; // clang, trackbar.c
pub const NM_RCLICK: u32 = 0xFFFFFFFB; // clang, header.c
pub const NM_RDBLCLK: u32 = 0xFFFFFFFA; // clang, listview.c
pub const NM_RELEASEDCAPTURE: u32 = 0xFFFFFFF0; // clang, header.c
pub const NM_RETURN: u32 = 0xFFFFFFFC; // clang, listview.c
pub const NM_SETCURSOR: u32 = 0xFFFFFFEF; // clang, comboex.c
pub const NM_SETFOCUS: u32 = 0xFFFFFFF9; // clang, datetime.c
pub const NM_TOOLTIPSCREATED: u32 = 0xFFFFFFED; // clang, commctrl.c
pub const RT_BITMAP: ULONG_PTR = 2; // clang, commctrl.c
pub const TD_ERROR_ICON: LPWSTR = 0xFFFEusize as LPWSTR; // clang, taskdialog.c
pub const TD_INFORMATION_ICON: LPWSTR = 0xFFFDusize as LPWSTR; // clang, taskdialog.c
pub const TD_SHIELD_ICON: LPWSTR = 0xFFFCusize as LPWSTR; // clang, taskdialog.c
pub const TD_WARNING_ICON: LPWSTR = 0xFFFFusize as LPWSTR; // clang, taskdialog.c
pub const TVHT_ONITEM: i32 = 0x46; // clang, treeview.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CBE_ITEMDATA: c2rust, comboex.c
pub type CBE_ITEMDATA = _CBE_ITEMDATA;
// CachedBrush: c2rust, monthcal.c
pub type CachedBrush = ::core::ffi::c_uint;
// HDPA: c2rust, dpa.c
pub type HDPA = *mut _DPA;
// HDSA: c2rust, dsa.c
pub type HDSA = *mut _DSA;
// HIMAGELIST: c2rust, comboex.c
pub type HIMAGELIST = *mut _IMAGELIST;
// HTREEITEM: c2rust, treeview.c
pub type HTREEITEM = *mut _TREEITEM;
// SCROLL_HITTEST: c2rust, theme_scrollbar.c
pub type SCROLL_HITTEST = ::core::ffi::c_uint;
// _CBE_ITEMDATA: c2rust, comboex.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CBE_ITEMDATA {
    pub next: *mut _CBE_ITEMDATA,
    pub mask: UINT,
    pub pszText: LPWSTR,
    pub pszTemp: LPWSTR,
    pub cchTextMax: ::core::ffi::c_int,
    pub iImage: ::core::ffi::c_int,
    pub iSelectedImage: ::core::ffi::c_int,
    pub iOverlay: ::core::ffi::c_int,
    pub iIndent: ::core::ffi::c_int,
    pub lParam: LPARAM,
}
// _DPA: c2rust, dpa.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _DPA {
    pub nItemCount: INT,
    pub ptrs: *mut LPVOID,
    pub hHeap: HANDLE,
    pub nGrow: INT,
    pub nMaxCount: INT,
}
// _DSA: c2rust, dsa.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _DSA {
    pub nItemCount: INT,
    pub pData: LPVOID,
    pub nMaxCount: INT,
    pub nItemSize: INT,
    pub nGrow: INT,
}
// _IMAGELIST: c2rust, imagelist.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IMAGELIST {
    pub IImageList2_iface: IImageList2,
    pub cCurImage: INT,
    pub cMaxImage: INT,
    pub cGrow: INT,
    pub cx: INT,
    pub cy: INT,
    pub x4: DWORD,
    pub flags: UINT,
    pub clrFg: COLORREF,
    pub clrBk: COLORREF,
    pub hbmImage: HBITMAP,
    pub hbmMask: HBITMAP,
    pub hdcImage: HDC,
    pub hdcMask: HDC,
    pub nOvlIdx: [INT; 15],
    pub usMagic: ULONG,
    pub hbrBlend25: HBRUSH,
    pub hbrBlend50: HBRUSH,
    pub cInitial: INT,
    pub uBitsPixel: UINT,
    pub has_alpha: *mut ::core::ffi::c_char,
    pub color_table_set: BOOL,
    pub ref_0: LONG,
    pub usVersion: USHORT,
}
// _TREEITEM: c2rust, treeview.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _TREEITEM {
    pub parent: HTREEITEM,
    pub nextSibling: HTREEITEM,
    pub firstChild: HTREEITEM,
    pub callbackMask: UINT,
    pub state: UINT,
    pub stateMask: UINT,
    pub pszText: LPWSTR,
    pub cchTextMax: ::core::ffi::c_int,
    pub iImage: ::core::ffi::c_int,
    pub iSelectedImage: ::core::ffi::c_int,
    pub iExpandedImage: ::core::ffi::c_int,
    pub cChildren: ::core::ffi::c_int,
    pub lParam: LPARAM,
    pub iIntegral: ::core::ffi::c_int,
    pub iLevel: ::core::ffi::c_int,
    pub lastChild: HTREEITEM,
    pub prevSibling: HTREEITEM,
    pub rect: RECT,
    pub linesOffset: LONG,
    pub stateOffset: LONG,
    pub imageOffset: LONG,
    pub textOffset: LONG,
    pub textWidth: LONG,
    pub visibleOrder: LONG,
    pub infoPtr: *const crate::comctl32::treeview::TREEVIEW_INFO,
}
// list: c2rust, syslink.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct list {
    pub next: *mut list,
    pub prev: *mut list,
}
// nav_direction: c2rust, monthcal.c
pub type nav_direction = ::core::ffi::c_uint;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _snprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Count: size_t,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // listview.c
    pub fn _wcsnicmp(
        _Str1: *const wchar_t,
        _Str2: *const wchar_t,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // listbox.c
}
extern "system" {
    pub fn GetCharWidthInfo(_: HDC, _: *mut crate::comctl32::edit::char_width_info) -> BOOL; // edit.c
    pub fn ImmGetCompositionStringW(hIMC: HIMC, dwIndex: DWORD, lpBuf: LPVOID, dwBufLen: DWORD)
        -> LONG; // edit.c
    pub fn ImmGetContext(hWnd: HWND) -> HIMC; // edit.c
    pub fn ImmIsIME(hKL: HKL) -> BOOL; // edit.c
    pub fn ImmLockIMC(hIMC: HIMC) -> LPINPUTCONTEXT; // edit.c
    pub fn ImmNotifyIME(hIMC: HIMC, dwAction: DWORD, dwIndex: DWORD, dwValue: DWORD_PTR) -> BOOL; // edit.c
    pub fn ImmReleaseContext(hWnd: HWND, hIMC: HIMC) -> BOOL; // edit.c
    pub fn ImmSetCompositionFontW(hIMC: HIMC, lplf: LPLOGFONTW) -> BOOL; // edit.c
    pub fn ImmSetCompositionWindow(hIMC: HIMC, lpCompForm: LPCOMPOSITIONFORM) -> BOOL; // edit.c
    pub fn ImmUnlockIMC(hIMC: HIMC) -> BOOL; // edit.c
    pub fn mmioAscend(_: HMMIO, _: LPMMCKINFO, _: UINT) -> MMRESULT; // animate.c
    pub fn mmioClose(_: HMMIO, _: UINT) -> MMRESULT; // animate.c
    pub fn mmioDescend(_: HMMIO, _: LPMMCKINFO, _: *const MMCKINFO, _: UINT) -> MMRESULT; // animate.c
    pub fn mmioOpenW(pszFileName: LPWSTR, pmmioinfo: LPMMIOINFO, fdwOpen: DWORD) -> HMMIO; // animate.c
    pub fn mmioRead(hmmio: HMMIO, pch: HPSTR, cch: LONG) -> LONG; // animate.c
    pub fn mmioSeek(_: HMMIO, _: LONG, _: ::core::ffi::c_int) -> LONG; // animate.c
}
pub use crate::prelude::GetMonitorInfoW as GetMonitorInfo; // sdk alias
pub use crate::prelude::GetTextExtentPoint32W as GetTextExtentPoint32; // sdk alias
pub use crate::prelude::SystemParametersInfoW as SystemParametersInfo; // sdk alias
pub use crate::prelude::_snprintf as snprintf; // sdk alias
pub use crate::prelude::_wcsnicmp as wcsnicmp; // sdk alias
// ==== end port externs ====

// Hand ports of C macros that read the CALLER's locals. `macro_rules!` hygiene hides a call
// site's locals from the macro body, so each captured local is an explicit leading parameter.
// rebar.c:251 HIDDENBAND(a) reads `infoPtr`.
macro_rules! HIDDENBAND {
    ($infoPtr:expr, $a:expr $(,)?) => {
        ((((*$a).fStyle & RBBS_HIDDEN) != 0)
            || ((((*$infoPtr).dwStyle & CCS_VERT) != 0) && (((*$a).fStyle & RBBS_NOVERT) != 0)))
    };
}
pub(crate) use HIDDENBAND;
// hotkey.c IsOnlySet(flags) reads `infoPtr`.
macro_rules! IsOnlySet {
    ($infoPtr:expr, $flags:expr $(,)?) => {
        ((*$infoPtr).CurrMod == $flags)
    };
}
pub(crate) use IsOnlySet;
// rebar.c:255 REBAR_GetInfoPtr(wndPtr) ignores its argument and reads `hwnd`; every caller
// passes `hwnd`, so the argument IS the local it reads.
macro_rules! REBAR_GetInfoPtr {
    ($hwnd:expr $(,)?) => {
        (GetWindowLongPtrW($hwnd, 0) as *mut REBAR_INFO)
    };
}
pub(crate) use REBAR_GetInfoPtr;
// propsheet.c:196 add_flag(a) reads `dwFlags` and `string`; C's `#a` is NUL-terminated.
macro_rules! add_flag {
    ($dwFlags:expr, $string:expr, $a:ident $(,)?) => {
        if ($dwFlags & $a as DWORD) != 0 {
            strcat($string.as_mut_ptr() as _, concat!(stringify!($a), "\0").as_ptr() as _);
            strcat($string.as_mut_ptr() as _, b" \0".as_ptr() as _);
        }
    };
}
pub(crate) use add_flag;
// <assert.h> under !NDEBUG (comctl32 builds Debug): `(!!(e)) || (_assert(#e, __FILE__,
// __LINE__), 0)`. Shadows core's `assert!`, which PANICS -- unwinding out of a window procedure
// in a user DLL -- and wants a `bool` where C takes any scalar. `CTruth` is C's `!!`.
pub trait CTruth { fn c_truth(self) -> bool; }
macro_rules! c_truth_int { ($($t:ty)*) => { $(impl CTruth for $t { #[inline(always)] fn c_truth(self) -> bool { self != 0 } })* }; }
c_truth_int!(i8 u8 i16 u16 i32 u32 i64 u64 isize usize);
impl CTruth for bool { #[inline(always)] fn c_truth(self) -> bool { self } }
impl<T: ?Sized> CTruth for *const T { #[inline(always)] fn c_truth(self) -> bool { !self.is_null() } }
impl<T: ?Sized> CTruth for *mut T { #[inline(always)] fn c_truth(self) -> bool { !self.is_null() } }
macro_rules! assert {
    ($e:expr $(,)?) => {
        if !$crate::prelude::CTruth::c_truth($e) {
            $crate::prelude::_assert(
                concat!(stringify!($e), "\0").as_ptr() as _,
                concat!(file!(), "\0").as_ptr() as _,
                line!(),
            );
        }
    };
}
pub(crate) use assert;
// wine/debug.h:360 `TRACE_ON(ch)` = `__WINE_IS_DEBUG_ON(_TRACE, &__wine_dbch_ch)`: the channel's
// own TRACE bit, then `__wine_dbg_get_channel_flags`. Every comctl32 site names its file's
// DEFAULT channel, so `__wine_dbch___default` (an item, resolved at the call site) is that
// channel -- `macro_rules!` cannot paste `__wine_dbch_` onto `$ch`.
macro_rules! TRACE_ON {
    ($ch:ident) => {
        ((((*__wine_dbch___default).flags as u32) & (1u32 << $crate::prelude::__WINE_DBCL_TRACE as u32)) != 0
            && (($crate::prelude::__wine_dbg_get_channel_flags(__wine_dbch___default) as u32)
                & (1u32 << $crate::prelude::__WINE_DBCL_TRACE as u32))
                != 0)
    };
}
pub(crate) use TRACE_ON;
// rebar.c:204 `#define SEP_WIDTH ((infoPtr->dwStyle & RBS_BANDBORDERS) ? SEP_WIDTH_SIZE : 0)` --
// object-like, but it reads the caller's `infoPtr`, so it takes it. C's type is `int`.
macro_rules! SEP_WIDTH {
    ($infoPtr:expr) => {
        (if ((*$infoPtr).dwStyle & RBS_BANDBORDERS as DWORD) != 0 { SEP_WIDTH_SIZE as i32 } else { 0i32 })
    };
}
pub(crate) use SEP_WIDTH;
// basetsd.h:75-76 `LongToHandle(h) ((HANDLE)(LONG_PTR)(h))` / `ULongToHandle(h)
// ((HANDLE)(ULONG_PTR)(h))`: straight to pointer width, no narrowing first.
macro_rules! LongToHandle { ($h:expr $(,)?) => { (($h) as isize) as HANDLE }; }
pub(crate) use LongToHandle;
macro_rules! ULongToHandle { ($h:expr $(,)?) => { (($h) as usize) as HANDLE }; }
pub(crate) use ULongToHandle;
// windowsx.h:311 `GetWindowFont(hwnd)` = `(HFONT)(UINT_PTR)SendMessage(hwnd, WM_GETFONT, 0, 0)`.
macro_rules! GetWindowFont {
    ($hwnd:expr $(,)?) => { SendMessageW($hwnd, WM_GETFONT, 0, 0) as UINT_PTR as HFONT };
}
pub(crate) use GetWindowFont;
// ================= wine/debug.h, wine/list.h and commctrl.h inlines ============================
// `static __inline` in the headers, so no library exports them: the C got them for free and the
// Rust must define them (comctl32's first i386 link listed every one as undefined).
extern "C" {
    pub fn wine_dbgstr_an(s: *const c_char, n: c_int) -> *const c_char;
    pub fn wine_dbgstr_wn(s: *const WCHAR, n: c_int) -> *const c_char;
}
#[inline] pub unsafe fn debugstr_a(s: *const c_char) -> *const c_char { wine_dbgstr_an(s, -1) }
#[inline] pub unsafe fn debugstr_an(s: *const c_char, n: c_int) -> *const c_char { wine_dbgstr_an(s, n) }
#[inline] pub unsafe fn debugstr_w(s: *const WCHAR) -> *const c_char { wine_dbgstr_wn(s, -1) }
#[inline] pub unsafe fn debugstr_wn(s: *const WCHAR, n: c_int) -> *const c_char { wine_dbgstr_wn(s, n) }
/// wine/debug.h:181 `wine_dbgstr_guid`.
pub unsafe fn debugstr_guid(id: *const GUID) -> *const c_char {
    if id.is_null() {
        return b"(null)\0".as_ptr() as *const c_char;
    }
    if (id as ULONG_PTR) >> 16 == 0 {
        return wine_dbg_sprintf(b"<guid-0x%04lx>\0".as_ptr() as *const c_char, (id as ULONG_PTR) & 0xffff);
    }
    let g = &*id;
    wine_dbg_sprintf(
        b"{%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x}\0".as_ptr() as *const c_char,
        g.Data1 as c_uint, g.Data2 as c_uint, g.Data3 as c_uint,
        g.Data4[0] as c_uint, g.Data4[1] as c_uint, g.Data4[2] as c_uint, g.Data4[3] as c_uint,
        g.Data4[4] as c_uint, g.Data4[5] as c_uint, g.Data4[6] as c_uint, g.Data4[7] as c_uint,
    )
}
/// wine/list.h `list_init`: an empty list points at itself both ways.
#[inline] pub unsafe fn list_init(list: *mut list) {
    (*list).next = list;
    (*list).prev = list;
}
/// wine/list.h `list_add_tail` = `list_add_before(list, elem)`.
#[inline] pub unsafe fn list_add_tail(list: *mut list, elem: *mut list) {
    (*elem).next = list;
    (*elem).prev = (*list).prev;
    (*(*list).prev).next = elem;
    (*list).prev = elem;
}
/// wine/list.h `list_remove`.
#[inline] pub unsafe fn list_remove(elem: *mut list) {
    (*(*elem).next).prev = (*elem).prev;
    (*(*elem).prev).next = (*elem).next;
}
/// commctrl.h:4972 `#define DPA_GetPtrCount(hdpa) (*(int *)(hdpa))`.
#[inline] pub unsafe fn DPA_GetPtrCount(hdpa: HDPA) -> INT { *(hdpa as *const INT) }
/// comctl32.h:312 `#define GetDpiForWindow(PVOID) 96` under __REACTOS__: user32 here has no such
/// export below _WIN32_WINNT 0x0605.
#[inline] pub fn GetDpiForWindow(_hwnd: HWND) -> UINT { 96 }
// pseh2.h:185-186 under DUMMY_PSEH (sdk/lib/pseh/dummy.c): `__TRY { A } __EXCEPT_PAGE_FAULT { B }
// __ENDTRY` compiles to `A;` then `if (_SEH2_Volatile0 || ..) { B }` -- a runtime-false guard,
// kept so the handler still compiles and says what the C does on a fault.
extern "C" {
    pub static _SEH2_Volatile0: c_int;
    pub static _SEH2_VolatileExceptionCode: c_int;
}
// uuid.lib's IID_IUnknown (objbase.h), which the island's allowlist leaves out.
extern "C" {
    pub static IID_IUnknown: GUID;
}
/// winerror.h `HRESULT_FROM_WIN32(x)`: `(HRESULT)(x) > 0 ? ((x & 0xFFFF) | (FACILITY_WIN32 << 16)
/// | 0x80000000) : (HRESULT)(x)`. DWORD in, as every caller passes `GetLastError()`.
#[inline]
pub fn HRESULT_FROM_WIN32(x: DWORD) -> HRESULT {
    if (x as HRESULT) > 0 {
        ((x & 0x0000FFFF) | (7 << 16) | 0x80000000) as HRESULT
    } else {
        x as HRESULT
    }
}
// edit.c:59 `(((size) + (GROWLENGTH - 1)) & ~(GROWLENGTH - 1))`: the int constant converts to
// the size's type (UINT at some sites, size_t at others), which `as _` says.
macro_rules! ROUND_TO_GROW {
    ($size:expr $(,)?) => {{
        let size = $size;
        let mut mask = size; // gives `mask` the size's type; the cast below then has a target
        mask = (GROWLENGTH - 1) as _;
        (size + mask) & !mask
    }};
}
// toolbar.c:245 `(infoPtr->iVersion >= 5 ? HIWORD(i) : 0)`: the WORD arm and the int arm meet
// at `int`.
macro_rules! GETHIMLID {
    ($infoPtr:expr, $i:expr $(,)?) => {
        (if ((*$infoPtr).iVersion) >= 5 { HIWORD!($i) as i32 } else { 0 })
    };
}
pub(crate) use GETHIMLID;
// tab.c `(DEFAULT_MIN_TAB_WIDTH - (DEFAULT_PADDING_X - (infoPtr)->uHItemPadding) * 2)`: the WORD
// padding promotes to int.
macro_rules! GET_DEFAULT_MIN_TAB_WIDTH {
    ($infoPtr:expr $(,)?) => {
        (DEFAULT_MIN_TAB_WIDTH - (DEFAULT_PADDING_X - (*$infoPtr).uHItemPadding as i32) * 2)
    };
}
pub(crate) use GET_DEFAULT_MIN_TAB_WIDTH;
pub(crate) use ROUND_TO_GROW;
// ================= COBJMACROS (objidl.h / commoncontrols.h) ===================================
// `This->lpVtbl->Slot(This, ..)`. IStream's vtable is ISequentialStream's five slots, then Seek
// (objidl.h order); only the slots comctl32 calls are spelled out.
#[repr(C)]
pub struct IStreamVtblHead {
    pub seq: ISequentialStreamVtbl,
    pub Seek: Option<unsafe extern "system" fn(*mut c_void, LARGE_INTEGER, DWORD, *mut ULARGE_INTEGER) -> HRESULT>,
}
#[inline(always)]
unsafe fn stream_vtbl<T>(p: *mut T) -> *const IStreamVtblHead {
    *(p as *mut *const IStreamVtblHead)
}
#[inline(always)]
pub unsafe fn IStream_Read<T>(p: *mut T, pv: *mut c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT {
    ((*stream_vtbl(p)).seq.Read.unwrap_unchecked())(p as *mut ISequentialStream, pv, cb, pcbRead)
}
#[inline(always)]
pub unsafe fn IStream_Write<T>(p: *mut T, pv: *const c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT {
    ((*stream_vtbl(p)).seq.Write.unwrap_unchecked())(p as *mut ISequentialStream, pv, cb, pcbWritten)
}
#[inline(always)]
pub unsafe fn IStream_Seek<T>(p: *mut T, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT {
    ((*stream_vtbl(p)).Seek.unwrap_unchecked())(p as *mut c_void, dlibMove, dwOrigin, plibNewPosition)
}
#[inline(always)]
pub unsafe fn IUnknown_QueryInterface<T>(p: *mut T, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    let vtbl = *(p as *mut *const IUnknownVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut IUnknown, riid, ppv)
}
#[inline(always)]
pub unsafe fn IImageList_Release(p: *mut IImageList) -> ULONG {
    ((*(*p).lpVtbl).Release.unwrap_unchecked())(p)
}
#[inline(always)]
pub unsafe fn IImageList2_AddRef(p: *mut IImageList2) -> ULONG {
    ((*(*p).lpVtbl).AddRef.unwrap_unchecked())(p)
}
#[inline(always)]
pub unsafe fn IImageList2_Release(p: *mut IImageList2) -> ULONG {
    ((*(*p).lpVtbl).Release.unwrap_unchecked())(p)
}
#[inline(always)]
pub unsafe fn IImageList2_QueryInterface(p: *mut IImageList2, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    ((*(*p).lpVtbl).QueryInterface.unwrap_unchecked())(p, riid, ppv)
}
// guiddef.h `#define IsEqualIID(a,b) IsEqualGUID(a,b)` = `!memcmp(a, b, sizeof(GUID))`: C's `int`,
// and every comctl32 site compares it with `!= 0`.
#[inline(always)]
pub unsafe fn IsEqualIID(a: *const GUID, b: *const GUID) -> BOOL {
    IsEqualGUID!(a, b) as BOOL
}
#[inline(always)]
pub unsafe fn IsEqualCLSID(a: *const GUID, b: *const GUID) -> BOOL {
    IsEqualGUID!(a, b) as BOOL
}
// vsstyle.h TOOLBARPARTS / TOOLBARSTYLESTATES under their C names (toolbar.rs postdates resolve).
pub const TP_BUTTON: TOOLBARPARTS = TOOLBARPARTS_TP_BUTTON;
pub const TP_DROPDOWNBUTTON: TOOLBARPARTS = TOOLBARPARTS_TP_DROPDOWNBUTTON;
pub const TP_SPLITBUTTON: TOOLBARPARTS = TOOLBARPARTS_TP_SPLITBUTTON;
pub const TP_SPLITBUTTONDROPDOWN: TOOLBARPARTS = TOOLBARPARTS_TP_SPLITBUTTONDROPDOWN;
pub const TP_SEPARATOR: TOOLBARPARTS = TOOLBARPARTS_TP_SEPARATOR;
pub const TP_SEPARATORVERT: TOOLBARPARTS = TOOLBARPARTS_TP_SEPARATORVERT;
pub const TS_NORMAL: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_NORMAL;
pub const TS_HOT: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_HOT;
pub const TS_PRESSED: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_PRESSED;
pub const TS_DISABLED: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_DISABLED;
pub const TS_CHECKED: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_CHECKED;
pub const TS_HOTCHECKED: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_HOTCHECKED;
pub const TS_NEARHOT: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_NEARHOT;
pub const TS_OTHERSIDEHOT: TOOLBARSTYLESTATES = TOOLBARSTYLESTATES_TS_OTHERSIDEHOT;
// tab.c:89 `max((infoPtr)->cbInfo, sizeof(LPARAM))`: C converts the INT to size_t to compare,
// and `max` is the shared crate's generic FUNCTION, not a macro.
macro_rules! EXTRA_ITEM_SIZE {
    ($infoPtr:expr $(,)?) => {
        max((*$infoPtr).cbInfo as usize, core::mem::size_of::<LPARAM>())
    };
}
pub(crate) use EXTRA_ITEM_SIZE;

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/comctl32/button.c:85: #define BUTTON_NOTIFY_PARENT(hWnd, code) do { TRACE("notification " #code " sent to hwnd=%p\n", GetParent(hWnd)); SendMessageW(GetParent(hWnd), WM_COMMAND, MAKEWPARAM(GetWindowLongPtrW((hWnd),GWLP_ID), (code)), (LPARAM)(hWnd)); } while(0)
#[allow(unused_macros)]
macro_rules! BUTTON_NOTIFY_PARENT {
    ($hWnd:expr, $code:expr $(,)?) => {{
        TRACE!(concat!("notification ", stringify!($code), " sent to hwnd=%p\n"), GetParent($hWnd));
        SendMessageW(GetParent($hWnd), WM_COMMAND, MAKEWPARAM!(GetWindowLongPtrW($hWnd, GWLP_ID), ($code)), ($hWnd) as LPARAM);
    }};
}
#[allow(unused_imports)]
pub(crate) use BUTTON_NOTIFY_PARENT;
// dll/win32/comctl32/combo.c:54: #define CB_DISABLED(lphc) (!IsWindowEnabled((lphc)->self))
#[allow(unused_macros)]
macro_rules! CB_DISABLED {
    ($lphc:expr $(,)?) => {
        (!IsWindowEnabled((*$lphc).self_))
    };
}
#[allow(unused_imports)]
pub(crate) use CB_DISABLED;
// dll/win32/comctl32/combo.c:58: #define CB_GETTYPE(lphc) ((lphc)->dwStyle & (CBS_DROPDOWNLIST))
#[allow(unused_macros)]
macro_rules! CB_GETTYPE {
    ($lphc:expr $(,)?) => {
        ((*$lphc).dwStyle & (CBS_DROPDOWNLIST))
    };
}
#[allow(unused_imports)]
pub(crate) use CB_GETTYPE;
// dll/win32/comctl32/combo.c:56: #define CB_HASSTRINGS(lphc) ((lphc)->dwStyle & CBS_HASSTRINGS)
#[allow(unused_macros)]
macro_rules! CB_HASSTRINGS {
    ($lphc:expr $(,)?) => {
        ((*$lphc).dwStyle & CBS_HASSTRINGS)
    };
}
#[allow(unused_imports)]
pub(crate) use CB_HASSTRINGS;
// dll/win32/comctl32/combo.c:50: #define CB_NOTIFY(lphc, code) (SendMessageW((lphc)->owner, WM_COMMAND, MAKEWPARAM(GetWindowLongPtrW((lphc)->self,GWLP_ID), (code)), (LPARAM)(lphc)->self))
#[allow(unused_macros)]
macro_rules! CB_NOTIFY {
    ($lphc:expr, $code:expr $(,)?) => {
        (SendMessageW((*$lphc).owner, WM_COMMAND, MAKEWPARAM!(GetWindowLongPtrW((*$lphc).self_, GWLP_ID), ($code)), ((*$lphc).self_) as LPARAM))
    };
}
#[allow(unused_imports)]
pub(crate) use CB_NOTIFY;
// dll/win32/comctl32/combo.c:55: #define CB_OWNERDRAWN(lphc) ((lphc)->dwStyle & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE))
#[allow(unused_macros)]
macro_rules! CB_OWNERDRAWN {
    ($lphc:expr $(,)?) => {
        ((*$lphc).dwStyle & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE))
    };
}
#[allow(unused_imports)]
pub(crate) use CB_OWNERDRAWN;
// dll/win32/comctl32/comboex.c:114: #define COMBOEX_GetInfoPtr(hwnd) ((COMBOEX_INFO *)GetWindowLongPtrW (hwnd, 0))
#[allow(unused_macros)]
macro_rules! COMBOEX_GetInfoPtr {
    ($hwnd:expr $(,)?) => {
        (GetWindowLongPtrW($hwnd, 0) as *mut COMBOEX_INFO)
    };
}
#[allow(unused_imports)]
pub(crate) use COMBOEX_GetInfoPtr;
// dll/win32/comctl32/combo.c:75: #define COMBO_EDITBUTTONSPACE() 0
#[allow(unused_macros)]
macro_rules! COMBO_EDITBUTTONSPACE {
    () => {
        (0)
    };
}
#[allow(unused_imports)]
pub(crate) use COMBO_EDITBUTTONSPACE;
// dll/win32/comctl32/combo.c:73: #define COMBO_XBORDERSIZE() 2
#[allow(unused_macros)]
macro_rules! COMBO_XBORDERSIZE {
    () => {
        (2)
    };
}
#[allow(unused_imports)]
pub(crate) use COMBO_XBORDERSIZE;
// dll/win32/comctl32/combo.c:74: #define COMBO_YBORDERSIZE() 2
#[allow(unused_macros)]
macro_rules! COMBO_YBORDERSIZE {
    () => {
        (2)
    };
}
#[allow(unused_imports)]
pub(crate) use COMBO_YBORDERSIZE;
// dll/win32/comctl32/monthcal.c:71: #define DAYSTO100NSECS(days) (((ULONGLONG)(days))*24*60*60*10000000)
#[allow(unused_macros)]
macro_rules! DAYSTO100NSECS {
    ($days:expr $(,)?) => {
        ((($days) as ULONGLONG) * 24 * 60 * 60 * 10000000)
    };
}
#[allow(unused_imports)]
pub(crate) use DAYSTO100NSECS;
// dll/win32/comctl32/combo.c:76: #define EDIT_CONTROL_PADDING() 1
#[allow(unused_macros)]
macro_rules! EDIT_CONTROL_PADDING {
    () => {
        (1)
    };
}
#[allow(unused_imports)]
pub(crate) use EDIT_CONTROL_PADDING;
// dll/win32/comctl32/treeview.c:199: #define GETBKCOLOR(x) ((x) == CLR_NONE ? comctl32_color.clrWindow : (x))
#[allow(unused_macros)]
macro_rules! GETBKCOLOR {
    ($x:expr $(,)?) => {
        (if ($x) == CLR_NONE { comctl32_color.clrWindow } else { ($x) })
    };
}
#[allow(unused_imports)]
pub(crate) use GETBKCOLOR;
// dll/win32/comctl32/toolbar.c:246: #define GETDEFIMAGELIST(infoPtr, id) TOOLBAR_GetImageList(infoPtr->himlDef, infoPtr->cimlDef, id)
#[allow(unused_macros)]
macro_rules! GETDEFIMAGELIST {
    ($infoPtr:expr, $id:expr $(,)?) => {
        (TOOLBAR_GetImageList((*$infoPtr).himlDef, (*$infoPtr).cimlDef, $id))
    };
}
#[allow(unused_imports)]
pub(crate) use GETDEFIMAGELIST;
// dll/win32/comctl32/toolbar.c:248: #define GETDISIMAGELIST(infoPtr, id) TOOLBAR_GetImageList(infoPtr->himlDis, infoPtr->cimlDis, id)
#[allow(unused_macros)]
macro_rules! GETDISIMAGELIST {
    ($infoPtr:expr, $id:expr $(,)?) => {
        (TOOLBAR_GetImageList((*$infoPtr).himlDis, (*$infoPtr).cimlDis, $id))
    };
}
#[allow(unused_imports)]
pub(crate) use GETDISIMAGELIST;
// dll/win32/comctl32/toolbar.c:247: #define GETHOTIMAGELIST(infoPtr, id) TOOLBAR_GetImageList(infoPtr->himlHot, infoPtr->cimlHot, id)
#[allow(unused_macros)]
macro_rules! GETHOTIMAGELIST {
    ($infoPtr:expr, $id:expr $(,)?) => {
        (TOOLBAR_GetImageList((*$infoPtr).himlHot, (*$infoPtr).cimlHot, $id))
    };
}
#[allow(unused_imports)]
pub(crate) use GETHOTIMAGELIST;
// dll/win32/comctl32/toolbar.c:244: #define GETIBITMAP(infoPtr, i) (infoPtr->iVersion >= 5 ? LOWORD(i) : i)
#[allow(unused_macros)]
macro_rules! GETIBITMAP {
    ($infoPtr:expr, $i:expr $(,)?) => {
        (if ((*$infoPtr).iVersion) >= 5 { LOWORD!($i) } else { $i })
    };
}
#[allow(unused_imports)]
pub(crate) use GETIBITMAP;
// dll/win32/comctl32/treeview.c:198: #define GETLINECOLOR(x) ((x) == CLR_DEFAULT ? comctl32_color.clrGrayText : (x))
#[allow(unused_macros)]
macro_rules! GETLINECOLOR {
    ($x:expr $(,)?) => {
        (if ($x) == CLR_DEFAULT { comctl32_color.clrGrayText } else { ($x) })
    };
}
#[allow(unused_imports)]
pub(crate) use GETLINECOLOR;
// dll/win32/comctl32/treeview.c:200: #define GETTXTCOLOR(x) ((x) == CLR_NONE ? comctl32_color.clrWindowText : (x))
#[allow(unused_macros)]
macro_rules! GETTXTCOLOR {
    ($x:expr $(,)?) => {
        (if ($x) == CLR_NONE { comctl32_color.clrWindowText } else { ($x) })
    };
}
#[allow(unused_imports)]
pub(crate) use GETTXTCOLOR;
// dll/win32/comctl32/listbox.c:103: #define HAS_STRINGS(descr) (!IS_OWNERDRAW(descr) || ((descr)->style & LBS_HASSTRINGS))
#[allow(unused_macros)]
macro_rules! HAS_STRINGS {
    ($descr:expr $(,)?) => {
        ((IS_OWNERDRAW!($descr) == false) || (((*$descr).style & LBS_HASSTRINGS) != 0))
    };
}
#[allow(unused_imports)]
pub(crate) use HAS_STRINGS;
// dll/win32/comctl32/treeview.c:196: #define ISVISIBLE(x) ((x)->visibleOrder >= 0)
#[allow(unused_macros)]
macro_rules! ISVISIBLE {
    ($x:expr $(,)?) => {
        ((*$x).visibleOrder >= 0)
    };
}
#[allow(unused_imports)]
pub(crate) use ISVISIBLE;
// dll/win32/comctl32/listbox.c:107: #define IS_MULTISELECT(descr) ((descr)->style & (LBS_MULTIPLESEL|LBS_EXTENDEDSEL) && !((descr)->style & LBS_NOSEL))
#[allow(unused_macros)]
macro_rules! IS_MULTISELECT {
    ($descr:expr $(,)?) => {
        ((((*$descr).style & (LBS_MULTIPLESEL | LBS_EXTENDEDSEL)) != 0) && (((*$descr).style & LBS_NOSEL) == 0))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_MULTISELECT;
// dll/win32/comctl32/listbox.c:100: #define IS_OWNERDRAW(descr) ((descr)->style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE))
#[allow(unused_macros)]
macro_rules! IS_OWNERDRAW {
    ($descr:expr $(,)?) => {
        ((((*$descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE))) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_OWNERDRAW;
// dll/win32/comctl32/listview.c:414: #define LISTVIEW_DUMP(iP) do { TRACE("hwndSelf=%p, clrBk=0x%06x, clrText=0x%06x, clrTextBk=0x%06x, ItemHeight=%d, ItemWidth=%d, Style=0x%08x\n", iP->hwndSelf, iP->clrBk, iP->clrText, iP->clrTextBk, iP->nItemHeight, iP->nItemWidth, iP->dwStyle); TRACE("hwndSelf=%p, himlNor=%p, himlSml=%p, himlState=%p, Focused=%d, Hot=%d, exStyle=0x%08x, Focus=%d\n", iP->hwndSelf, iP->himlNormal, iP->himlSmall, iP->himlState, iP->nFocusedItem, iP->nHotItem, iP->dwLvExStyle, iP->bFocus ); TRACE("hwndSelf=%p, ntmH=%d, icSz.cx=%d, icSz.cy=%d, icSp.cx=%d, icSp.cy=%d, notifyFmt=%d\n", iP->hwndSelf, iP->ntmHeight, iP->iconSize.cx, iP->iconSize.cy, iP->iconSpacing.cx, iP->iconSpacing.cy, iP->notifyFormat); TRACE("hwndSelf=%p, rcList=%s\n", iP->hwndSelf, wine_dbgstr_rect(&iP->rcList)); } while(0)
#[allow(unused_macros)]
macro_rules! LISTVIEW_DUMP {
    ($iP:expr $(,)?) => {{
        TRACE!("hwndSelf=%p, clrBk=0x%06x, clrText=0x%06x, clrTextBk=0x%06x, ItemHeight=%d, ItemWidth=%d, Style=0x%08x\n", (*$iP).hwndSelf, (*$iP).clrBk, (*$iP).clrText, (*$iP).clrTextBk, (*$iP).nItemHeight, (*$iP).nItemWidth, (*$iP).dwStyle);
        TRACE!("hwndSelf=%p, himlNor=%p, himlSml=%p, himlState=%p, Focused=%d, Hot=%d, exStyle=0x%08x, Focus=%d\n", (*$iP).hwndSelf, (*$iP).himlNormal, (*$iP).himlSmall, (*$iP).himlState, (*$iP).nFocusedItem, (*$iP).nHotItem, (*$iP).dwLvExStyle, (*$iP).bFocus);
        TRACE!("hwndSelf=%p, ntmH=%d, icSz.cx=%d, icSz.cy=%d, icSp.cx=%d, icSp.cy=%d, notifyFmt=%d\n", (*$iP).hwndSelf, (*$iP).ntmHeight, (*$iP).iconSize.cx, (*$iP).iconSize.cy, (*$iP).iconSpacing.cx, (*$iP).iconSpacing.cy, (*$iP).notifyFormat);
        TRACE!("hwndSelf=%p, rcList=%s\n", (*$iP).hwndSelf, wine_dbgstr_rect(&(*$iP).rcList));
    }};
}
#[allow(unused_imports)]
pub(crate) use LISTVIEW_DUMP;
// dll/win32/comctl32/nativefont.c:47: #define NATIVEFONT_GetInfoPtr(hwnd) ((NATIVEFONT_INFO *)GetWindowLongPtrW (hwnd, 0))
#[allow(unused_macros)]
macro_rules! NATIVEFONT_GetInfoPtr {
    ($hwnd:expr $(,)?) => {
        (GetWindowLongPtrW($hwnd, 0) as *mut NATIVEFONT_INFO)
    };
}
#[allow(unused_imports)]
pub(crate) use NATIVEFONT_GetInfoPtr;
// dll/win32/comctl32/rebar.c:244: #define REBARSPACE(a) ((a->fStyle & RBBS_CHILDEDGE) ? 2*REBAR_DIVIDER : 0)
#[allow(unused_macros)]
macro_rules! REBARSPACE {
    ($a:expr $(,)?) => {
        (if ((*$a).fStyle & RBBS_CHILDEDGE) != 0 { 2 * REBAR_DIVIDER } else { 0 })
    };
}
#[allow(unused_imports)]
pub(crate) use REBARSPACE;
// dll/win32/comctl32/listbox.c:111: #define SEND_NOTIFICATION(descr, code) (SendMessageW( (descr)->owner, WM_COMMAND, MAKEWPARAM( GetWindowLongPtrW((descr->self),GWLP_ID), (code)), (LPARAM)(descr->self) ))
#[allow(unused_macros)]
macro_rules! SEND_NOTIFICATION {
    ($descr:expr, $code:expr $(,)?) => {
        (SendMessageW((*$descr).owner, WM_COMMAND, MAKEWPARAM!(GetWindowLongPtrW((*$descr).self_, GWLP_ID), ($code)), ((*$descr).self_) as LPARAM))
    };
}
#[allow(unused_imports)]
pub(crate) use SEND_NOTIFICATION;
// dll/win32/comctl32/tab.c:145: #define TAB_GetInfoPtr(hwnd) ((TAB_INFO *)GetWindowLongPtrW(hwnd,0))
#[allow(unused_macros)]
macro_rules! TAB_GetInfoPtr {
    ($hwnd:expr $(,)?) => {
        (GetWindowLongPtrW($hwnd, 0) as *mut TAB_INFO)
    };
}
#[allow(unused_imports)]
pub(crate) use TAB_GetInfoPtr;
// dll/win32/comctl32/tab.c:90: #define TAB_ITEM_SIZE(infoPtr) FIELD_OFFSET(TAB_ITEM, extra[EXTRA_ITEM_SIZE(infoPtr)])
#[allow(unused_macros)]
macro_rules! TAB_ITEM_SIZE {
    ($infoPtr:expr $(,)?) => {
        FIELD_OFFSET!(TAB_ITEM, extra[EXTRA_ITEM_SIZE!($infoPtr)])
    };
}
#[allow(unused_imports)]
pub(crate) use TAB_ITEM_SIZE;
// dll/win32/comctl32/tooltips.c:158: #define TOOLTIPS_GetInfoPtr(hWindow) ((TOOLTIPS_INFO *)GetWindowLongPtrW (hWindow, 0))
#[allow(unused_macros)]
macro_rules! TOOLTIPS_GetInfoPtr {
    ($hWindow:expr $(,)?) => {
        (GetWindowLongPtrW($hWindow, 0) as *mut TOOLTIPS_INFO)
    };
}
#[allow(unused_imports)]
pub(crate) use TOOLTIPS_GetInfoPtr;
// dll/win32/comctl32/updown.c:91: #define UPDOWN_GetInfoPtr(hwnd) ((UPDOWN_INFO *)GetWindowLongPtrW (hwnd,0))
#[allow(unused_macros)]
macro_rules! UPDOWN_GetInfoPtr {
    ($hwnd:expr $(,)?) => {
        (GetWindowLongPtrW($hwnd, 0) as *mut UPDOWN_INFO)
    };
}
#[allow(unused_imports)]
pub(crate) use UPDOWN_GetInfoPtr;
// dll/win32/comctl32/imagelist.c:125: #define WinVerMajor() LOBYTE(GetVersion())
#[allow(unused_macros)]
macro_rules! WinVerMajor {
    () => {
        (LOBYTE!(GetVersion()))
    };
}
#[allow(unused_imports)]
pub(crate) use WinVerMajor;
// dll/win32/comctl32/listview.c:3196: #define ranges_check(ranges, desc) if (TRACE_ON!(listview)) ranges_assert(ranges, desc, __FILE__, __LINE__)
#[allow(unused_macros)]
macro_rules! ranges_check {
    ($ranges:expr, $desc:expr $(,)?) => {{
        if TRACE_ON!(listview) {
            ranges_assert($ranges, $desc, concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char, line!() as core::ffi::c_int)
        }
    }};
}
#[allow(unused_imports)]
pub(crate) use ranges_check;
// ==== end port macros ====
