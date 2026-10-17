//! Hand-written substrate for the two halves of `dll/win32/shell32/CDefaultContextMenu.cpp`.
//!
//! That one TU needs two rustconv runs — `--cpp-com --class` for the COM class
//! (`CDefaultContextMenu.rs`) and `--free-functions` for the rest
//! (`CDefaultContextMenu_tu.rs`) — and each run OVERWRITES its own output file. So anything
//! hand-written that lives in either of them is lost the next time that half is re-assembled.
//! It lives here instead, where re-assembly cannot reach it, and both halves `use` it.
//!
//! Everything here is either a header inline the TU calls (`utils.h`, `shellutils.h`,
//! `winerror.h`) or a constant/extern outside the bindgen island's include closure. A header
//! inline reaches no translation unit, so rustconv never sees a body for one and the name
//! simply does not resolve — these are ports, not translations, and each records its source.

use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
pub use crate::vista::RegLoadMUIStringW;
pub use crate::wine::shlmenu::Shell_MergeMenus;

// ── winerror.h / shellutils.h ────────────────────────────────────────────────────────────

/// psdk/winerror.h:188 `#define HRESULT_CODE(hr) ((hr) & 0xFFFF)`
#[inline]
pub fn HRESULT_CODE(hr: HRESULT) -> HRESULT {
    hr & 0xFFFF
}

/// reactos/shellutils.h:70
#[inline]
pub fn HResultFromWin32(err: DWORD) -> HRESULT {
    HRESULT_FROM_WIN32(err as LONG)
}

/// reactos/shellutils.h:688 `CmicFlagsToSeeFlags(flags) = (flags) & SEE_CMIC_COMMON_FLAGS`
#[inline]
pub const fn CmicFlagsToSeeFlags(flags: DWORD) -> DWORD {
    flags & SEE_CMIC_COMMON_FLAGS
}

/// winuser.h `IS_INTRESOURCE` — a verb pointer whose high bits are clear is really an
/// integer command id, not a string. Spelling copied from move_to_menu.rs:1244.
#[inline]
pub fn IS_INTRESOURCE<T>(value: *const T) -> bool {
    (value as usize) >> 16 == 0
}

// ── dll/win32/shell32/utils.h registry inlines ───────────────────────────────────────────

/// utils.h:43. Note it tests only for EXISTENCE — the value's type and contents are
/// irrelevant, which is why every out-parameter is NULL.
#[inline]
/// Returns `bool`, not C's `BOOL`: every caller in this TU uses it as a predicate
/// (`if RegValueExists(..)`, `hide = RegValueExists(..)`), and the two files that call it are
/// the only ones in the crate.
pub unsafe fn RegValueExists(hKey: HKEY, Name: LPCWSTR) -> bool {
    RegQueryValueExW(
        hKey,
        Name,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == ERROR_SUCCESS as LONG
}

/// utils.h:57
#[inline]
pub unsafe fn RegQueryDword(hKey: HKEY, pszPath: PCWSTR, pszName: PCWSTR, pnVal: *mut DWORD) -> UINT {
    let mut cb: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    RegGetValueW(
        hKey,
        pszPath,
        pszName,
        RRF_RT_REG_DWORD,
        core::ptr::null_mut(),
        pnVal.cast(),
        &mut cb,
    ) as UINT
}

/// utils.h:65. `nVal` is deliberately left uninitialised in the C and only read when the
/// query succeeded; the Rust zero-initialises it, which cannot change the result.
#[inline]
pub unsafe fn RegGetDword(hKey: HKEY, pszPath: PCWSTR, pszName: PCWSTR, nDefVal: DWORD) -> DWORD {
    let mut nVal: DWORD = 0;
    if RegQueryDword(hKey, pszPath, pszName, &mut nVal) == ERROR_SUCCESS as UINT {
        nVal
    } else {
        nDefVal
    }
}

/// `RRF_RT_REG_DWORD` — winreg.h:61. Not in the island's closure.
pub const RRF_RT_REG_DWORD: DWORD = 1 << 4;

// ── ole2.h / shlobj.h ────────────────────────────────────────────────────────────────────

/// shlobj.h `InitFormatEtc` — spelling copied from droptargets/CFSDropTarget.rs:109.
#[inline]
pub unsafe fn InitFormatEtc(fe: *mut FORMATETC, cf: UINT, med: DWORD) {
    (*fe).cfFormat = cf as CLIPFORMAT;
    (*fe).dwAspect = DVASPECT_CONTENT;
    (*fe).ptd = core::ptr::null_mut();
    (*fe).tymed = med;
    (*fe).lindex = -1;
}

/// shlobj.h:503 `CFSTR_PREFERREDDROPEFFECTW` = L"Preferred DropEffect".
pub static CFSTR_PREFERREDDROPEFFECT: [WCHAR; 21] = [
    b'P' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'f' as WCHAR, b'e' as WCHAR, b'r' as WCHAR,
    b'r' as WCHAR, b'e' as WCHAR, b'd' as WCHAR, b' ' as WCHAR, b'D' as WCHAR, b'r' as WCHAR,
    b'o' as WCHAR, b'p' as WCHAR, b'E' as WCHAR, b'f' as WCHAR, b'f' as WCHAR, b'e' as WCHAR,
    b'c' as WCHAR, b't' as WCHAR, 0,
];

/// shlobj.h `CFSTR_SHELLIDLISTW` = L"Shell IDList Array". `shellutils.rs` already owns the
/// array; re-exported under the TU's spelling so both halves resolve it.
pub use crate::shellutils::CFSTR_SHELLIDLISTW as CFSTR_SHELLIDLIST;

// ── shobjidl.idl:1704-1705 — aliases of the SEE_MASK_* the island does provide ───────────

pub const CMIC_MASK_HASLINKNAME: DWORD = SEE_MASK_HASLINKNAME;
pub const CMIC_MASK_HASTITLE: DWORD = SEE_MASK_HASTITLE;

// ── wtypesbase.h ─────────────────────────────────────────────────────────────────────────

/// `CLSCTX_ALL` = INPROC_SERVER|INPROC_HANDLER|LOCAL_SERVER|REMOTE_SERVER. Value copied from
/// drop_target_helper.rs:95, which spells the same constant for the same reason.
pub const CLSCTX_ALL: DWORD = 0x17;

// ── shlobj.h:2614-2626 — the DFM_CMD_* callback commands ─────────────────────────────────
//
// All are `((UINT)-N)`, and the C++ stores them in a `SHORT` field via `(SHORT)DFM_CMD_X`.
// Spelling them as UINT here keeps the comparisons in the translated bodies unsigned, which
// is what the C does.

pub const DFM_CMD_DELETE: UINT = -1i32 as UINT;
pub const DFM_CMD_COPY: UINT = -3i32 as UINT;
pub const DFM_CMD_LINK: UINT = -4i32 as UINT;
pub const DFM_CMD_PROPERTIES: UINT = -5i32 as UINT;
pub const DFM_CMD_NEWFOLDER: UINT = -6i32 as UINT;
pub const DFM_CMD_PASTE: UINT = -7i32 as UINT;
pub const DFM_CMD_RENAME: UINT = -13i32 as UINT;

// ── imports the TU makes that the island does not carry ──────────────────────────────────

extern "C" {
    /// Rust `#[no_mangle]` def: shell32/shlfolder.rs:842. CDECL, not stdcall: precomp.h:175
    /// declares it `EXTERN_C HRESULT` with no `WINAPI`, and only i386 has a convention
    /// difference to notice.
    pub fn SHELL32_ShowPropertiesDialog(pdtobj: *mut IDataObject) -> HRESULT;
}

extern "system" {
    /// ole32.
    pub fn OleGetClipboard(ppDataObj: *mut *mut IDataObject) -> HRESULT;
    pub fn OleSetClipboard(pDataObj: *mut IDataObject) -> HRESULT;
    /// shlwapi — the W form; the C++ spells it unsuffixed under UNICODE.
    #[link_name = "PathRemoveFileSpecW"]
    pub fn PathRemoveFileSpec(path: LPWSTR) -> BOOL;
    /// Rust `#[no_mangle]` def: droptargets/CRecyclerDropTarget.rs:658.
    pub fn CRecyclerDropTarget_CreateInstance_RustBridge(riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
}

extern "C" {
    /// msvcrt.
    pub fn _stricmp(left: LPCSTR, right: LPCSTR) -> INT;
    /// Rust `#[no_mangle] extern "C"` defs: wine/pidl.rs:2278 / 2291.
    pub fn _ILFreeaPidl(apidl: *mut LPITEMIDLIST, cidl: UINT);
    pub fn _ILCopyaPidl(apidlsrc: *const LPCITEMIDLIST, cidl: UINT) -> *mut LPITEMIDLIST;
}

/// strsafe.h `StringCchCopyA` — the narrow twin of the `StringCchCopyW` the cpp-com support
/// block emits. Byte-for-byte the same shape, on `c_char` instead of `WCHAR`.
#[inline]
pub unsafe fn StringCchCopyA(dst: LPSTR, capacity: usize, src: LPCSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 {
        return E_INVALIDARG;
    }
    let mut i = 0usize;
    while *src.add(i) != 0 {
        if i + 1 >= capacity {
            *dst = 0;
            // STRSAFE_E_INSUFFICIENT_BUFFER
            return 0x8007007Au32 as HRESULT;
        }
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
    S_OK
}

// ── reactos/shellutils.h:722 ─────────────────────────────────────────────────────────────

/// `CMIC_MASK_UNICODE` — shobjidl.idl. The island does not carry it.
pub const CMIC_MASK_UNICODE: DWORD = SEE_MASK_UNICODE;

/// shellutils.h:722 `IsUnicode(const CMINVOKECOMMANDINFOEX &ici)`.
///
/// BOTH halves of the test matter: the caller must have set the UNICODE mask AND passed a
/// struct big enough to actually contain the wide members. Dropping the size check reads
/// `lpVerbW` off the end of a short CMINVOKECOMMANDINFO.
#[inline]
pub unsafe fn IsUnicode(ici: *const CMINVOKECOMMANDINFOEX) -> bool {
    let minsize = FIELD_OFFSET!(CMINVOKECOMMANDINFOEX, ptInvoke) as UINT;
    ((*ici).fMask & CMIC_MASK_UNICODE) != 0 && (*ici).cbSize >= minsize
}

/// utils.h:50. Note it opens with MAXIMUM_ALLOWED and closes again — existence only.
#[inline]
/// `bool` rather than C's `BOOL`, for the same reason as `RegValueExists` above.
pub unsafe fn RegKeyExists(hKey: HKEY, Path: LPCWSTR) -> bool {
    let mut out: HKEY = core::ptr::null_mut();
    let ret = RegOpenKeyExW(hKey, Path, 0, MAXIMUM_ALLOWED, &mut out) == 0;
    if ret {
        RegCloseKey(out);
    }
    ret
}

/// `MAXIMUM_ALLOWED` — winnt.h. Outside the island's closure.
pub const MAXIMUM_ALLOWED: REGSAM = 0x0200_0000;

// ── precomp.h:198-200 / utils.h:126-127 — the DSA-backed handle arrays ───────────────────

/// precomp.h:200 `DCMA_GetEntry(hDCMA, iItem)`
#[inline]
pub unsafe fn DCMA_GetEntry(hDCMA: HDCMA, iItem: INT) -> *mut DCMENTRY {
    DSA_GetItemPtr(hDCMA as HDSA, iItem).cast()
}

/// utils.h:126 `DCIA_GetCount(hDCIA)`
#[inline]
pub unsafe fn DCIA_GetCount(hDCIA: HDCIA) -> INT {
    DSA_GetItemCount(hDCIA as HDSA)
}

/// utils.h:127 `DCIA_GetEntry(hDCIA, iItem)`
#[inline]
pub unsafe fn DCIA_GetEntry(hDCIA: HDCIA, iItem: INT) -> *const CLSID {
    DSA_GetItemPtr(hDCIA as HDSA, iItem).cast()
}

/// comctl32 `DSA_AppendItem(hdsa, p)` is `DSA_InsertItem(hdsa, DA_LAST, p)`.
#[inline]
pub unsafe fn DSA_AppendItem<T>(hdsa: HDSA, item: *const T) -> INT {
    DSA_InsertItem(hdsa, DA_LAST, item as *mut core::ffi::c_void)
}

/// commctrl.h `DA_LAST` = 0x7fffffff.
pub const DA_LAST: INT = 0x7fff_ffff;

/// commctrl.h:4831 `#define DSA_GetItemCount(hdsa) (*(int *) (hdsa))` — a MACRO reading the
/// count straight out of the handle, not an exported function. Declaring it `extern` links
/// against a symbol comctl32 does not have.
#[inline]
pub unsafe fn DSA_GetItemCount(hdsa: HDSA) -> INT {
    *(hdsa as *const INT)
}

// NOT here: `SID_SFolderView`. shobjidl.idl:904 is
// `#define SID_SFolderView IID_IFolderView` — an ALIAS for an interface id, not a symbol of
// its own, and the same for `SID_IShellBrowser` (shlguid_undoc.h:215 = `IID_IShellBrowser`).
// This module declared the first as an `extern static` and the cpp-com generator invented the
// second, because both names are GUID-shaped. Neither is exported by any library: the call
// sites name the IID directly.

// ── COM interfaces in the `lpVtbl` shape ─────────────────────────────────────────────────
//
// The island renders these two as bindgen C++ classes with an opaque `vtable_`, which no
// translated call site can go through. `shellcom` carries the rest of this TU's interfaces
// (`IContextMenu`, `IDropTarget`, `IUnknown`, …) but deliberately not these — its header
// records why `IShellExtInit` in particular stays out: five already-landed files carry their
// own file-local copy, so adding a sixth to that glob-imported module would make the name
// E0659-ambiguous in every one of them. This TU declares its own copy, as that note directs.
//
// Only the slots with callers here are typed; the rest are `usize` placeholders holding their
// correct offsets, so the ones that matter land on the right vtable index.

/// psdk/shobjidl.h `IShellExtInit`.
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *mut IShellExtInitVtbl,
}
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IShellExtInit, REFIID, *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(
        *mut IShellExtInit,
        PCIDLIST_ABSOLUTE,
        *mut IDataObject,
        HKEY,
    ) -> HRESULT,
}

/// psdk/shlobj.h:892 `IShellFolderView`. Slot order is that DECLARE_INTERFACE_ block
/// verbatim — `SetClipboard` is index 24 and `SetPoints` index 25, counting the three
/// IUnknown slots, and getting either wrong calls a neighbouring method with its arguments.
#[repr(C)]
pub struct IShellFolderView {
    pub lpVtbl: *mut IShellFolderViewVtbl,
}
#[repr(C)]
pub struct IShellFolderViewVtbl {
    pub QueryInterface: usize,
    pub AddRef: usize,
    pub Release: unsafe extern "system" fn(*mut IShellFolderView) -> ULONG,
    pub Rearrange: usize,
    pub GetArrangeParam: usize,
    pub ArrangeGrid: usize,
    pub AutoArrange: usize,
    pub GetAutoArrange: usize,
    pub AddObject: usize,
    pub GetObject: usize,
    pub RemoveObject: usize,
    pub GetObjectCount: usize,
    pub SetObjectCount: usize,
    pub UpdateObject: usize,
    pub RefreshObject: usize,
    pub SetRedraw: usize,
    pub GetSelectedCount: usize,
    pub GetSelectedObjects: usize,
    pub IsDropOnSource: usize,
    pub GetDragPoint: usize,
    pub GetDropPoint: usize,
    pub MoveIcons: usize,
    pub SetItemPos: usize,
    pub IsBkDropTarget: usize,
    pub SetClipboard: unsafe extern "system" fn(*mut IShellFolderView, BOOL) -> HRESULT,
    pub SetPoints: unsafe extern "system" fn(*mut IShellFolderView, *mut IDataObject) -> HRESULT,
    pub GetItemSpacing: usize,
    pub SetCallback: usize,
    pub Select: usize,
    pub QuerySupport: usize,
    pub SetAutomationObject: usize,
}

/// shell32/precomp.h:192 `_DCMENTRY`. Re-declared here rather than taken from the island for
/// one reason: the island types `pCM` as ITS `IContextMenu`, the bindgen C++ class with an
/// opaque `vtable_`, so `dcme.pCM->Release()` has nothing to call through. Same layout —
/// pointer, `UINT`, `UINT` — with `pCM` bound to the fully-typed interface both halves use.
#[repr(C)]
pub struct DCMENTRY {
    pub pCM: *mut crate::default_context_menu::IContextMenu,
    pub idCmdFirst: UINT,
    pub idCmdLast: UINT,
}
