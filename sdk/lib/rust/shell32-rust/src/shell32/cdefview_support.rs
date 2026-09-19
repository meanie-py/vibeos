//! Hand-written substrate for `CDefView.cpp`'s translation — NOT a translated TU.
//!
//! Three kinds of thing land here, none of which a paid run can produce:
//!
//! 1. **SDK constants outside the bindgen island's closure.** Every value below is copied from
//!    the header that defines it, and the ones C composes from other constants are kept as the
//!    same EXPRESSION rather than a hand-evaluated number — an arithmetic slip in a menu-id
//!    range is exactly the silent-miscompare class that a catch-all `match` arm hides.
//! 2. **`__MINGW_NAME_AW` aliases.** shell32 is a Unicode build, so each resolves to its `W`
//!    form, which the island already renders.
//! 3. **The two file-scope `struct`s CDefView.cpp declares for its persisted view state.**
//!    These are SERIALIZED to an `IStream`, so their layout is load-bearing: `#[repr(C)]`, and
//!    the C++ `static const` members become associated consts because a C++ static member
//!    occupies no storage and must not become a field.

use crate::prelude::*;
use crate::control_panel_folder::FVM_ICON;

// ------------------------------------------------------------------------------------------
// __MINGW_NAME_AW aliases (commctrl.h / winuser.h) — shell32 is a Unicode build
// ------------------------------------------------------------------------------------------

pub type HDITEM = HDITEMW;
pub type LVCOLUMN = LVCOLUMNW;
pub type LVITEM = LVITEMW;
pub type LV_HITTESTINFO = LVHITTESTINFO;

pub const LVM_EDITLABEL: UINT = LVM_EDITLABELW;
pub const SB_SETTEXT: UINT = SB_SETTEXTW;

/// `((LPWSTR)-1L)` — commctrl.h's "ask me for the text at draw time" sentinel. It is a
/// SENTINEL, not an address: never dereferenced, only compared and stored.
pub const LPSTR_TEXTCALLBACKW: LPWSTR = usize::MAX as LPWSTR;
pub const LPSTR_TEXTCALLBACK: LPWSTR = LPSTR_TEXTCALLBACKW;

/// `((HWND)0)` — winuser.h's z-order placement value for SetWindowPos.
pub const HWND_TOP: HWND = core::ptr::null_mut();

// ------------------------------------------------------------------------------------------
// HRESULTs (winerror.h / oleidl.h / docobj.h)
// ------------------------------------------------------------------------------------------

pub const NOERROR: HRESULT = 0;

pub const DRAGDROP_S_DROP: HRESULT = 0x00040100u32 as HRESULT;
pub const DRAGDROP_S_CANCEL: HRESULT = 0x00040101u32 as HRESULT;
pub const DRAGDROP_S_USEDEFAULTCURSORS: HRESULT = 0x00040102u32 as HRESULT;

/// docobj.h derives the OLECMDERR codes from `OLE_E_LAST + 1`; kept as that expression.
pub const OLE_E_LAST: HRESULT = 0x800400FFu32 as HRESULT;
pub const OLECMDERR_E_FIRST: HRESULT = OLE_E_LAST + 1;
pub const OLECMDERR_E_NOTSUPPORTED: HRESULT = OLECMDERR_E_FIRST;
pub const OLECMDERR_E_UNKNOWNGROUP: HRESULT = OLECMDERR_E_FIRST + 4;

// ------------------------------------------------------------------------------------------
// shobjidl.h / shlobj.h view constants
// ------------------------------------------------------------------------------------------

/// shobjidl.h's FOLDERVIEWMODE tail; the island renders only part of the enum.
pub const FVM_CONTENT: DWORD = 8;

/// shobjidl.h FOLDERFLAGS members CDefView's persisted VALIDFWF mask is built from.
pub const FWF_AUTOARRANGE: UINT = 0x00000001;
pub const FWF_SNAPTOGRID: UINT = 0x00000004;
pub const FWF_NOGROUPING: UINT = 0x00100000;

pub const FVM_AUTO: c_int = -1;
pub const FVM_FIRST: c_int = FVM_ICON as c_int;
pub const FVM_LAST: c_int = FVM_CONTENT as c_int;

pub const SV3CVW3_FORCEFOLDERFLAGS: UINT = 0x00000004;
pub const SHCONTF_INCLUDESUPERHIDDEN: UINT = 0x10000;
pub const STRRET_WSTR: UINT = 0;

/// `SV2GV_*` are shobjidl.h's "which view" pseudo-indices, both negative when read as signed.
pub const SV2GV_CURRENTVIEW: UINT = UINT::MAX; // (UINT)-1
pub const SV2GV_DEFAULTVIEW: UINT = UINT::MAX - 1; // (UINT)-2

/// shellapi.h: `CMIC_MASK_UNICODE` is an alias for `SEE_MASK_UNICODE`.
pub const SEE_MASK_UNICODE: DWORD = 0x00004000;
pub const CMIC_MASK_UNICODE: DWORD = SEE_MASK_UNICODE;

// ------------------------------------------------------------------------------------------
// CDefView's own menu-id partition of the FCIDM_SHVIEW range
// ------------------------------------------------------------------------------------------
//
// The arrange/sort commands are carved off the TOP of the range and the context-menu ids get
// what is left below them, so every bound here depends on DEFVIEW_ARRANGESORT_MAXENUM. Kept as
// the C's expressions: evaluating them by hand is how two id ranges silently start overlapping.

pub const DEFVIEW_ARRANGESORT_MAXENUM: UINT = 9;
pub const DEFVIEW_ARRANGESORT_MAX: UINT = DEFVIEW_ARRANGESORT_MAXENUM + 1;

pub const DVIDM_ARRANGESORT_LAST: UINT = FCIDM_SHVIEWLAST as UINT;
pub const DVIDM_ARRANGESORT_FIRST: UINT = DVIDM_ARRANGESORT_LAST - (DEFVIEW_ARRANGESORT_MAX - 1);
pub const DVIDM_COMMDLG_SELECT: UINT = DVIDM_ARRANGESORT_FIRST - 1;
pub const DVIDM_CONTEXTMENU_FIRST: UINT = FCIDM_SHVIEWFIRST as UINT;
pub const DVIDM_CONTEXTMENU_LAST: UINT = DVIDM_COMMDLG_SELECT - 1;

// ------------------------------------------------------------------------------------------
// Persisted view state (file-scope structs in CDefView.cpp)
// ------------------------------------------------------------------------------------------

/// `('R' << 0) | ('O' << 8) | ('S' << 16) | (<tag> << 24)` — the shared shape of both
/// signatures, written out so neither magic number is typed by hand.
const fn ros_signature(tag: u8) -> UINT {
    (b'R' as UINT) | ((b'O' as UINT) << 8) | ((b'S' as UINT) << 16) | ((tag as UINT) << 24)
}

/// Written to and read from an `IStream`, so the layout is part of the on-disk format.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PERSISTCOLUMNS {
    pub Signature: UINT,
    pub Count: UINT,
    pub Columns: [UINT; PERSISTCOLUMNS::MAXCOUNT as usize],
}

impl PERSISTCOLUMNS {
    pub const MAXCOUNT: UINT = 100;
    pub const SIG: UINT = ros_signature(b'c');
}

/// Same stream, different record. `SortColId`/`SortDir` sit between two 4-byte-aligned members,
/// so the tail padding C++ inserts before `FolderSettings` is part of the format too — which is
/// exactly what `#[repr(C)]` reproduces and what a packed or reordered struct would break.
#[repr(C)]
pub struct PERSISTCLASSICVIEWSTATE {
    pub Signature: UINT,
    pub SortColId: WORD,
    pub SortDir: INT8,
    pub FolderSettings: FOLDERSETTINGS,
}

impl PERSISTCLASSICVIEWSTATE {
    pub const SIG: UINT = ros_signature(b'c' ^ b'v' ^ b's');
    /// The folder flags this record is allowed to restore. The desktop applies FWF_NOICONS
    /// itself when appropriate, so it is deliberately NOT in the set.
    pub const VALIDFWF: UINT = FWF_AUTOARRANGE | FWF_SNAPTOGRID | FWF_NOGROUPING;
}

extern "C" {
    /// ntoskrnl/CRT-provided. Never hand-roll a byte comparison in Rust: LLVM folds the loop
    /// back into a call to this symbol and the function recurses into itself.
    pub fn memcmp(lhs: *const core::ffi::c_void, rhs: *const core::ffi::c_void, count: usize) -> c_int;
}

/// `#define ListView_GetColumn(w, i, p) (BOOL)SNDMSG((w), LVM_GETCOLUMN, (WPARAM)(int)(i),
/// (LPARAM)(LV_COLUMN *)(p))` (commctrl.h). The Unicode build sends `LVM_GETCOLUMNW`.
#[inline]
pub unsafe fn ListView_GetColumn(hwnd: HWND, iCol: c_int, pcol: *mut LVCOLUMNW) -> BOOL {
    SendMessageW(hwnd, LVM_GETCOLUMNW, iCol as WPARAM, pcol as LPARAM) as BOOL
}

// ------------------------------------------------------------------------------------------
// wingdi.h COLORREF accessors
// ------------------------------------------------------------------------------------------
//
// A COLORREF is 0x00BBGGRR, so the byte order here is the reverse of how the colour is usually
// written. Each cast is a TRUNCATION in C and must stay one in Rust.

#[inline]
pub fn GetRValue(rgb: COLORREF) -> BYTE {
    rgb as BYTE
}

#[inline]
pub fn GetGValue(rgb: COLORREF) -> BYTE {
    ((rgb as WORD) >> 8) as BYTE
}

#[inline]
pub fn GetBValue(rgb: COLORREF) -> BYTE {
    (rgb >> 16) as BYTE
}

// ------------------------------------------------------------------------------------------
// windowsx.h WM_COMMAND unpackers
// ------------------------------------------------------------------------------------------

#[inline]
pub fn GET_WM_COMMAND_ID(wp: WPARAM, _lp: LPARAM) -> WORD {
    (wp & 0xFFFF) as WORD
}

#[inline]
pub fn GET_WM_COMMAND_CMD(wp: WPARAM, _lp: LPARAM) -> WORD {
    ((wp >> 16) & 0xFFFF) as WORD
}

#[inline]
pub fn GET_WM_COMMAND_HWND(_wp: WPARAM, lp: LPARAM) -> HWND {
    lp as HWND
}

// ------------------------------------------------------------------------------------------
// commctrl.h DPA_* — these read the HDPA's LAYOUT directly, not through an API
// ------------------------------------------------------------------------------------------
//
// `DPA_GetPtrCount`/`DPA_GetPtrPtr` are macros that dereference the opaque handle: the count is
// the first `int` and the item array is the pointer one machine word in. That is a documented
// part of the ABI, which is why it is reproduced rather than replaced with a call.

#[inline]
pub unsafe fn DPA_GetPtrCount(hdpa: HDPA) -> c_int {
    *(hdpa as *const c_int)
}

#[inline]
pub unsafe fn DPA_GetPtrPtr(hdpa: HDPA) -> *mut *mut core::ffi::c_void {
    *((hdpa as *mut u8).add(core::mem::size_of::<*mut core::ffi::c_void>())
        as *mut *mut *mut core::ffi::c_void)
}

#[inline]
pub unsafe fn DPA_FastGetPtr(hdpa: HDPA, i: c_int) -> *mut core::ffi::c_void {
    *DPA_GetPtrPtr(hdpa).add(i as usize)
}

/// `DPA_InsertPtr(hdpa, DA_LAST, pitem)`; DA_LAST is commctrl's 0x7FFFFFFF "append" index.
#[inline]
pub unsafe fn DPA_AppendPtr(hdpa: HDPA, pitem: *mut core::ffi::c_void) -> c_int {
    DPA_InsertPtr(hdpa, DA_LAST, pitem)
}

pub const DA_LAST: c_int = 0x7FFF_FFFF;

// ------------------------------------------------------------------------------------------
// Misc header inlines
// ------------------------------------------------------------------------------------------

/// `#define IsEqualCLSID(a, b) IsEqualGUID(a, b)` — a CLSID is a GUID.
#[inline]
pub unsafe fn IsEqualCLSID(rclsid1: *const GUID, rclsid2: *const GUID) -> BOOL {
    IsEqualGUID(rclsid1, rclsid2) as BOOL
}

/// `utils.h`: does this object report the given persist class? A failure to answer is NOT a
/// match — the C returns FALSE rather than leaving `clsid` at whatever GetClassID wrote.
pub unsafe fn IsEqualPersistClassID(pPersist: *mut IPersist, clsid: *const GUID) -> BOOL {
    if pPersist.is_null() {
        return FALSE;
    }
    let mut actual: GUID = core::mem::zeroed();
    let get_class_id = match (*(*pPersist).lpVtbl).GetClassID {
        Some(f) => f,
        None => return FALSE,
    };
    if get_class_id(pPersist, &mut actual) < 0 {
        return FALSE;
    }
    IsEqualCLSID(&actual, clsid)
}

/// `#define UPDATEFOLDERVIEWFLAGS(bits, bit, set)` — clear the bit, then set it if `set`.
/// Written as a macro because the C one ASSIGNS to its first argument.
macro_rules! UPDATEFOLDERVIEWFLAGS {
    ($bits:expr, $bit:expr, $set:expr) => {
        $bits = ($bits & !$bit) | (if $set { $bit } else { 0 })
    };
}
pub(crate) use UPDATEFOLDERVIEWFLAGS;

/// CDefView.cpp:3062 `REGSTR_PATH_EXPLORER L"\\IconUnderline"` — C adjacent-string-literal
/// CONCATENATION, which is ONE string, not two expressions. The translation emitted the two
/// operands juxtaposed with no operator, which Rust parsed as a PATH (`REGSTR_PATH_EXPLORER::…`).
/// The array is the concatenated result stated in full, so it has a complete specification to
/// be checked against: L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\IconUnderline".
/// `dialogs/general.rs` carries the identical constant for general.cpp:104.
pub static REGSTR_PATH_EXPLORER_ICONUNDERLINE: [WCHAR; 65] = [
    0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063,
    0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
    0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065,
    0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072,
    0x0065, 0x0072, 0x005C, 0x0049, 0x0063, 0x006F, 0x006E, 0x0055, 0x006E, 0x0064, 0x0065, 0x0072,
    0x006C, 0x0069, 0x006E, 0x0065, 0x0000,
];

// ------------------------------------------------------------------------------------------
// user32 exports outside the island's closure
// ------------------------------------------------------------------------------------------

extern "system" {
    /// `SetShellWindowEx` (winuser.h). Not in shell32's bindgen island — the header guards it
    /// behind `#if (_WIN32_WINNT >= 0x0500)` plus `NTDDI`, and shell32's TUs never pull it in
    /// (CDefView.cpp declares it itself). shell32.dll already imports user32.
    pub fn SetShellWindowEx(hwndShell: HWND, hwndShellListView: HWND) -> BOOL;
}
