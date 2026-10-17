//! FileMenu_* owner-draw PIDL menu API + Shell_MergeMenus —
//! translation of dll/win32/shell32/wine/shlmenu.c.
//!
//! All 25 exports are WINAPI → `extern "system"` → `_Name@N` on i386 (confirmed against
//! the pre-flip object's symbol table). This is the `__REACTOS__` build config
//! (`-D_WIN32_WINNT=0x600 -D_SHELL32_ -D_WINE -DUNICODE -D__REACTOS__`); the only string
//! literal that survives into the object is the wide L"(empty)" (verified via
//! `strings -el shlmenu.c.obj`), so there are no Wine-only branches to translate here.
//!
//! TRACE/WARN/ERR/FIXME and the `pdump()` pidl debug-dump are no-ops (no observable
//! behavior) and are omitted per the translation contract.
//!
//! The internal PIDL helpers from the still-C pidl.c resolve as:
//!   `_ILClone@4`, `_ILCombine@8`        → WINAPI  (`extern "system"`)
//!   `_ILGetDisplayNameExW`, `_PidlToSicIndex` → cdecl (`extern "C"`, no `@N`)
//! `_IID_IShellFolder` is a cdecl DATA export.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use crate::types::*;
use ntdll_sys::nt::SIZE_T;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILCombine;
use crate::wine::pidl::ILGetDisplayNameExW;
use crate::shell32::iconcache::PidlToSicIndex;
use crate::wine::shellole::SHAlloc;
use crate::desktop_folder_tu::SHGetDesktopFolder;
// One declaration per external symbol: island.rs.
use crate::island::CreatePopupMenu;
use crate::island::DeleteMenu;
use crate::island::DestroyMenu;
use crate::island::ExtTextOutW;
use crate::island::GetDC;
use crate::island::GetMenuInfo;
use crate::island::GetMenuItemCount;
use crate::island::GetMenuItemInfoW;
use crate::island::GetMenuItemRect;
use crate::island::GetTextExtentPoint32W;
use crate::island::HeapAlloc;
use crate::island::ImageList_Draw;
use crate::island::InsertMenuA;
use crate::island::InsertMenuItemW;
use crate::island::InsertMenuW;
use crate::island::ReleaseDC;
use crate::island::SetBkColor;
use crate::island::SetMenuInfo;
use crate::island::SetTextColor;
use crate::island::TrackPopupMenuEx;
use core::ffi::c_void;
use core::mem::{size_of, zeroed};
use core::ptr::{null, null_mut};

// ── file-local types not yet in types.rs ───────────────────────────────────────────────
pub type ULONG = u32; // winnt.h
pub type COLORREF = DWORD; // windef.h
pub use crate::island::HMENU; // windef.h
pub use crate::island::HDC; // windef.h
pub use crate::island::HBITMAP; // windef.h
pub use crate::island::HBRUSH; // windef.h
pub use crate::island::HIMAGELIST; // commctrl.h
pub type WPARAM = UINT_PTR; // minwindef.h
pub type LRESULT = LONG_PTR; // minwindef.h

/// ITEMIDLIST (shtypes.idl) — only ever used through a pointer, kept opaque.
#[repr(C)]
pub struct ITEMIDLIST {
    _private: [u8; 0],
}
pub type LPITEMIDLIST = *mut ITEMIDLIST;
pub type LPCITEMIDLIST = *const ITEMIDLIST;

/// GUID / IID (guiddef.h): { u32; u16; u16; [u8;8] }
pub use crate::island::GUID;
pub type IID = GUID;
pub type REFIID = *const IID;

/// RECT (windef.h): { LONG left, top, right, bottom; }
pub use crate::island::RECT;

/// SIZE (windef.h): { LONG cx; LONG cy; }
pub use crate::island::SIZE;

/// MENUINFO (winuser.h)
pub use crate::island::MENUINFO;

/// MENUITEMINFOW (winuser.h) — `hbmpItem` present because `_WIN32_WINNT >= 0x0500`.
pub use crate::island::MENUITEMINFOW;

/// MEASUREITEMSTRUCT (winuser.h)
pub use crate::island::MEASUREITEMSTRUCT;
pub type LPMEASUREITEMSTRUCT = *mut MEASUREITEMSTRUCT;

/// DRAWITEMSTRUCT (winuser.h)
pub use crate::island::DRAWITEMSTRUCT;
pub type LPDRAWITEMSTRUCT = *mut DRAWITEMSTRUCT;

/// TPMPARAMS (winuser.h)
pub use crate::island::TPMPARAMS;
pub type LPTPMPARAMS = *mut TPMPARAMS;

/// LPFNFMCALLBACK (undocshell.h): `void (CALLBACK *)(LPCITEMIDLIST, LPCITEMIDLIST)`.
/// CALLBACK == __stdcall → `extern "system"`. Nullable → `Option<fn>`.
pub type LPFNFMCALLBACK = Option<unsafe extern "system" fn(LPCITEMIDLIST, LPCITEMIDLIST)>;

/// IContextMenu — only ever returned as an opaque pointer here.
#[repr(C)]
pub struct IContextMenu {
    _private: [u8; 0],
}

// ── file-private bookkeeping structs (faithful #[repr(C)] of the C originals) ───────────

/// FMINFO / *LPFMINFO — the per-menu data stored in `MENUINFO.dwMenuData`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FMINFO {
    pub bInitialized: BOOL,
    pub bFixedItems: BOOL,
    // create
    pub crBorderColor: COLORREF,
    pub nBorderWidth: INT,
    pub hBorderBmp: HBITMAP,
    // insert using pidl
    pub pidl: LPITEMIDLIST,
    pub uID: UINT,
    pub uFlags: UINT,
    pub uEnumFlags: UINT,
    pub lpfnCallback: LPFNFMCALLBACK,
}
pub type LPFMINFO = *mut FMINFO;

/// FMITEM / *LPFMITEM — the per-item data stored in `MENUITEMINFOW.dwItemData`.
/// `szItemText[1]` is a flexible trailing array; allocated as `sizeof(FMITEM)+len*WCHAR`.
#[repr(C)]
pub struct FMITEM {
    pub cchItemText: INT,
    pub iIconIndex: INT,
    pub hMenu: HMENU,
    pub szItemText: [WCHAR; 1],
}
pub type LPFMITEM = *mut FMITEM;

// ── COM vtables (STDMETHODCALLTYPE == __stdcall on i386 → extern "system") ───────────────
// Only the slots actually invoked carry real signatures; the rest are pointer-sized
// placeholders so the slot offsets stay correct (verified vs build-i386 shobjidl.h).

#[repr(C)]
pub struct IShellFolder {
    pub lpVtbl: *mut IShellFolderVtbl,
}
#[repr(C)]
pub struct IShellFolderVtbl {
    pub QueryInterface: usize,                                       // 0
    pub AddRef: usize,                                               // 1
    pub Release: unsafe extern "system" fn(*mut IShellFolder) -> ULONG, // 2
    pub ParseDisplayName: usize,                                     // 3
    pub EnumObjects: unsafe extern "system" fn(
        *mut IShellFolder,
        HWND,
        DWORD,
        *mut *mut IEnumIDList,
    ) -> HRESULT, // 4
    pub BindToObject: unsafe extern "system" fn(
        *mut IShellFolder,
        LPCITEMIDLIST,
        *mut c_void,
        REFIID,
        *mut LPVOID,
    ) -> HRESULT, // 5
    pub BindToStorage: usize,                                        // 6
    pub CompareIDs: usize,                                           // 7
    pub CreateViewObject: usize,                                     // 8
    pub GetAttributesOf: unsafe extern "system" fn(
        *mut IShellFolder,
        UINT,
        *const LPCITEMIDLIST,
        *mut ULONG,
    ) -> HRESULT, // 9
    pub GetUIObjectOf: usize,                                        // 10
    pub GetDisplayNameOf: usize,                                     // 11
    pub SetNameOf: usize,                                            // 12
}

#[repr(C)]
pub struct IEnumIDList {
    pub lpVtbl: *mut IEnumIDListVtbl,
}
#[repr(C)]
pub struct IEnumIDListVtbl {
    pub QueryInterface: usize,                                       // 0
    pub AddRef: usize,                                               // 1
    pub Release: unsafe extern "system" fn(*mut IEnumIDList) -> ULONG, // 2
    pub Next: unsafe extern "system" fn(
        *mut IEnumIDList,
        ULONG,
        *mut LPITEMIDLIST,
        *mut ULONG,
    ) -> HRESULT, // 3
    pub Skip: usize,                                                 // 4
    pub Reset: usize,                                                // 5
    pub Clone: usize,                                                // 6
}

// ── constants pinned from SDK headers ───────────────────────────────────────────────────
const MAX_PATH: usize = 260; // minwindef.h

const MIM_MENUDATA: DWORD = 8; // winuser.h
const MIIM_STATE: UINT = 1; // winuser.h
const MIIM_ID: UINT = 2; // winuser.h
const MIIM_SUBMENU: UINT = 4; // winuser.h
const MIIM_CHECKMARKS: UINT = 8; // winuser.h
const MIIM_TYPE: UINT = 16; // winuser.h
const MIIM_DATA: UINT = 32; // winuser.h

const MFT_OWNERDRAW: UINT = 256; // winuser.h
const MFT_SEPARATOR: UINT = 0x800; // winuser.h
const MFT_STRING: UINT = 0; // winuser.h
const MFS_ENABLED: UINT = 0; // winuser.h
const MFS_DEFAULT: UINT = 4096; // winuser.h

const MF_BYCOMMAND: UINT = 0; // winuser.h
const MF_BYPOSITION: UINT = 1024; // winuser.h
const MF_SEPARATOR: UINT = 0x800; // winuser.h

const ODS_SELECTED: UINT = 1; // winuser.h
const COLOR_MENU: INT = 4; // winuser.h
const COLOR_MENUTEXT: INT = 7; // winuser.h
const COLOR_HIGHLIGHT: INT = 13; // winuser.h
const COLOR_HIGHLIGHTTEXT: INT = 14; // winuser.h

const ETO_OPAQUE: UINT = 0x0002; // wingdi.h
const ILD_NORMAL: UINT = 0x0; // commctrl.h

const HEAP_ZERO_MEMORY: DWORD = 0x00000008; // winnt.h
const CP_ACP: UINT = 0; // winnls.h

const SFGAO_FOLDER: ULONG = 0x20000000; // shobjidl.h
const SHCONTF_FOLDERS: DWORD = 32; // shobjidl.h
const SHCONTF_NONFOLDERS: DWORD = 64; // shobjidl.h
const ILGDN_FORPARSING: DWORD = 0; // undocshell.h

const MM_ADDSEPARATOR: ULONG = 0x00000001; // shlobj.h
const MM_SUBMENUSHAVEIDS: ULONG = 0x00000002; // shlobj.h

// undocshell.h FileMenu sentinels
const FM_SEPARATOR: LPCWSTR = 1 as LPCWSTR; // #define FM_SEPARATOR (LPCWSTR)1 (in shlmenu.c)
const FM_BLANK_ICON: INT = -1; // undocshell.h
const FM_DEFAULT_HEIGHT: INT = 0; // undocshell.h

// L"(empty)"
static EMPTY_W: [WCHAR; 8] = [0x28, 0x65, 0x6d, 0x70, 0x74, 0x79, 0x29, 0x00];

// ── imports ─────────────────────────────────────────────────────────────────────────────
extern "system" {
    // user32
// @0
// @12
// @4
// @4
// @8
// @8
// @8
// @4
// @16
// @16
// @20
// @20
// @16
    fn GetSysColor(nIndex: INT) -> DWORD; // @4
// @24
    // gdi32
// @32
// @16
// @8
// @8
    // comctl32 (imported through the IAT thunk `__imp__ImageList_Draw@24`)
// @24
    // kernel32
    fn GetProcessHeap() -> HANDLE; // @0
    fn GetVersion() -> DWORD; // @0
// @12
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // @12
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT; // @24
    // shlwapi
    fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR; // @4
    // shell32 itself
// @4
    fn SHFree(pv: LPVOID); // @4
// @4
    fn Shell_GetImageLists(lpBigList: *mut HIMAGELIST, lpSmallList: *mut HIMAGELIST) -> BOOL; // @8
    // pidl.c (still-C) WINAPI helpers
// @4
// @8
}

// pidl.c / shell32_main.c (still-C) cdecl helpers — no `@N` decoration.
extern "C" {
}

// data import: `_IID_IShellFolder` (cdecl DATA export of shell32)
extern "C" {
    static IID_IShellFolder: IID;
}

// ── small in-file helpers ────────────────────────────────────────────────────────────────
#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}

/// strlenW (wine/unicode.h) — reimplemented in-file to avoid pulling the `__imp_` thunk.
#[inline]
unsafe fn strlenW(mut s: *const WCHAR) -> INT {
    let mut n: INT = 0;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}

/// strcpyW (wine/unicode.h) — reimplemented in-file.
#[inline]
unsafe fn strcpyW(dst: *mut WCHAR, mut src: *const WCHAR) -> *mut WCHAR {
    let mut d = dst;
    loop {
        let c = *src;
        *d = c;
        if c == 0 {
            break;
        }
        d = d.add(1);
        src = src.add(1);
    }
    dst
}

/// SHELL_OsIsUnicode() — shell32_main.h: `!(GetVersion() & 0x80000000)`.
#[inline]
unsafe fn shell_os_is_unicode() -> bool {
    (GetVersion() & 0x80000000) == 0
}

/// `bAbortInit` — file-private global (faithful to the C `static BOOL bAbortInit;`).
static mut bAbortInit: BOOL = 0;

// ════════════════════════════════════════════════════════════════════════════════════════
//  Internal helpers (file-private)
// ════════════════════════════════════════════════════════════════════════════════════════

/// FM_GetMenuInfo [internal] — fetch the LPFMINFO from a menu's dwMenuData.
unsafe fn FM_GetMenuInfo(hmenu: HMENU) -> LPFMINFO {
    let mut MenuInfo: MENUINFO = zeroed();

    MenuInfo.cbSize = size_of::<MENUINFO>() as DWORD;
    MenuInfo.fMask = MIM_MENUDATA;

    if GetMenuInfo((hmenu).cast(), &mut MenuInfo) == FALSE {
        return null_mut();
    }

    let menudata = MenuInfo.dwMenuData as LPFMINFO;

    if menudata.is_null() || (MenuInfo.cbSize != size_of::<MENUINFO>() as DWORD) {
        return null_mut();
    }

    menudata
}

/// FM_SetMenuParameter [internal]
unsafe fn FM_SetMenuParameter(
    hmenu: HMENU,
    uID: UINT,
    pidl: LPCITEMIDLIST,
    uFlags: UINT,
    uEnumFlags: UINT,
    lpfnCallback: LPFNFMCALLBACK,
) -> LPFMINFO {
    let menudata = FM_GetMenuInfo(hmenu);

    SHFree((*menudata).pidl as LPVOID);

    (*menudata).uID = uID;
    (*menudata).pidl = (ILClone((pidl).cast())).cast();
    (*menudata).uFlags = uFlags;
    (*menudata).uEnumFlags = uEnumFlags;
    (*menudata).lpfnCallback = lpfnCallback;

    menudata
}

/// FM_InitMenuPopup [internal]
unsafe fn FM_InitMenuPopup(hmenu: HMENU, pAlternatePidl: LPCITEMIDLIST) -> INT {
    let mut lpsf: *mut IShellFolder = null_mut();
    let mut ulItemAttr: ULONG = SFGAO_FOLDER;
    let uID: UINT;
    let uEnumFlags: UINT;
    let lpfnCallback: LPFNFMCALLBACK;
    let pidl: LPCITEMIDLIST;
    let mut sTemp: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut NumberOfItems: INT = 0;
    let mut iIcon: INT = 0;
    let mut MenuInfo: MENUINFO = zeroed();

    MenuInfo.cbSize = size_of::<MENUINFO>() as DWORD;
    MenuInfo.fMask = MIM_MENUDATA;

    if GetMenuInfo((hmenu).cast(), &mut MenuInfo) == FALSE {
        return FALSE;
    }

    let menudata = MenuInfo.dwMenuData as LPFMINFO;

    if menudata.is_null() || (MenuInfo.cbSize != size_of::<MENUINFO>() as DWORD) {
        return 0;
    }

    if (*menudata).bInitialized != FALSE {
        return 0;
    }

    pidl = if !pAlternatePidl.is_null() {
        pAlternatePidl
    } else {
        (*menudata).pidl as LPCITEMIDLIST
    };
    if pidl.is_null() {
        return 0;
    }

    uID = (*menudata).uID;
    uEnumFlags = (*menudata).uEnumFlags;
    lpfnCallback = (*menudata).lpfnCallback;
    (*menudata).bInitialized = FALSE;

    SetMenuInfo((hmenu).cast(), &MenuInfo);

    if SUCCEEDED(SHGetDesktopFolder((&raw mut lpsf).cast())) {
        let mut lpsf2: *mut IShellFolder = null_mut();
        if SUCCEEDED(((*(*lpsf).lpVtbl).BindToObject)(
            lpsf,
            pidl,
            null_mut(),
            &IID_IShellFolder,
            &mut lpsf2 as *mut *mut IShellFolder as *mut LPVOID,
        )) {
            let mut lpe: *mut IEnumIDList = null_mut();

            if SUCCEEDED(((*(*lpsf2).lpVtbl).EnumObjects)(lpsf2, null_mut(), uEnumFlags, &mut lpe)) {
                let mut pidlTemp: LPITEMIDLIST = null_mut();
                let mut ulFetched: ULONG = 0;

                while (bAbortInit == FALSE)
                    && (S_OK
                        == ((*(*lpe).lpVtbl).Next)(lpe, 1, &mut pidlTemp, &mut ulFetched))
                {
                    if SUCCEEDED(((*(*lpsf).lpVtbl).GetAttributesOf)(
                        lpsf,
                        1,
                        &pidlTemp as *const LPITEMIDLIST as *const LPCITEMIDLIST,
                        &mut ulItemAttr,
                    )) {
                        ILGetDisplayNameExW(
                            null_mut(),
                            (pidlTemp).cast(),
                            sTemp.as_mut_ptr(),
                            ILGDN_FORPARSING,
                        );
                        if PidlToSicIndex((lpsf).cast(), (pidlTemp).cast(), FALSE, 0, &mut iIcon) == FALSE {
                            iIcon = FM_BLANK_ICON;
                        }
                        if (SFGAO_FOLDER & ulItemAttr) != 0 {
                            let hMenuPopup = CreatePopupMenu();

                            let lpFmMi = HeapAlloc(
                                GetProcessHeap(),
                                HEAP_ZERO_MEMORY,
                                (size_of::<FMINFO>()) as _,
                            ) as LPFMINFO;

                            (*lpFmMi).pidl = (ILCombine((pidl).cast(), (pidlTemp as LPCITEMIDLIST).cast())).cast();
                            (*lpFmMi).uEnumFlags = SHCONTF_FOLDERS | SHCONTF_NONFOLDERS;

                            let mut MenuInfo2: MENUINFO = zeroed();
                            MenuInfo2.cbSize = size_of::<MENUINFO>() as DWORD;
                            MenuInfo2.fMask = MIM_MENUDATA;
                            MenuInfo2.dwMenuData = lpFmMi as ULONG_PTR;
                            SetMenuInfo(hMenuPopup, &MenuInfo2);

                            FileMenu_AppendItemW(
                                hmenu,
                                sTemp.as_ptr(),
                                uID,
                                iIcon,
                                (hMenuPopup).cast(),
                                FM_DEFAULT_HEIGHT,
                            );
                        } else {
                            let pExt = PathFindExtensionW(sTemp.as_ptr());
                            if !pExt.is_null() {
                                *pExt = 0;
                            }
                            FileMenu_AppendItemW(
                                hmenu,
                                sTemp.as_ptr(),
                                uID,
                                iIcon,
                                null_mut(),
                                FM_DEFAULT_HEIGHT,
                            );
                        }
                    }

                    if let Some(cb) = lpfnCallback {
                        cb(pidl, pidlTemp as LPCITEMIDLIST);
                    }

                    NumberOfItems += 1;
                }
                ((*(*lpe).lpVtbl).Release)(lpe);
            }
            ((*(*lpsf2).lpVtbl).Release)(lpsf2);
        }
        ((*(*lpsf).lpVtbl).Release)(lpsf);
    }

    if GetMenuItemCount((hmenu).cast()) == 0 {
        FileMenu_AppendItemW(
            hmenu,
            EMPTY_W.as_ptr(),
            uID,
            FM_BLANK_ICON,
            null_mut(),
            FM_DEFAULT_HEIGHT,
        );
        NumberOfItems += 1;
    }

    (*menudata).bInitialized = TRUE;
    SetMenuInfo((hmenu).cast(), &MenuInfo);

    NumberOfItems
}

/// FileMenu_AppendItem [internal, SHELL32.115] — the W worker.
unsafe fn FileMenu_AppendItemW(
    hMenu: HMENU,
    lpText: LPCWSTR,
    uID: UINT,
    icon: INT,
    hMenuPopup: HMENU,
    _nItemHeight: INT,
) -> BOOL {
    let mut mii: MENUITEMINFOW = zeroed();
    let mut MenuInfo: MENUINFO = zeroed();

    mii.cbSize = size_of::<MENUITEMINFOW>() as UINT;

    if lpText != FM_SEPARATOR {
        let len = strlenW(lpText);
        let myItem = SHAlloc((size_of::<FMITEM>() + len as usize * size_of::<WCHAR>()) as SIZE_T) as LPFMITEM;
        strcpyW((*myItem).szItemText.as_mut_ptr(), lpText);
        (*myItem).cchItemText = len;
        (*myItem).iIconIndex = icon;
        (*myItem).hMenu = hMenu;
        mii.fMask = MIIM_DATA;
        mii.dwItemData = myItem as ULONG_PTR;
    }

    if !hMenuPopup.is_null() {
        // sub menu
        mii.fMask |= MIIM_TYPE | MIIM_SUBMENU;
        mii.fType = MFT_OWNERDRAW;
        mii.hSubMenu = (hMenuPopup).cast();
    } else if lpText == FM_SEPARATOR {
        mii.fMask |= MIIM_ID | MIIM_TYPE;
        mii.fType = MFT_SEPARATOR;
    } else {
        // normal item
        mii.fMask |= MIIM_ID | MIIM_TYPE | MIIM_STATE;
        mii.fState = MFS_ENABLED | MFS_DEFAULT;
        mii.fType = MFT_OWNERDRAW;
    }
    mii.wID = uID;

    InsertMenuItemW((hMenu).cast(), !0u32, TRUE, &mii); // (UINT)-1

    // set bFixedItems to true
    MenuInfo.cbSize = size_of::<MENUINFO>() as DWORD;
    MenuInfo.fMask = MIM_MENUDATA;

    if GetMenuInfo((hMenu).cast(), &mut MenuInfo) == FALSE {
        return FALSE;
    }

    let menudata = MenuInfo.dwMenuData as LPFMINFO;
    if menudata.is_null() || (MenuInfo.cbSize != size_of::<MENUINFO>() as DWORD) {
        return FALSE;
    }

    (*menudata).bFixedItems = TRUE;
    SetMenuInfo((hMenu).cast(), &MenuInfo);

    TRUE
}

/// _SHIsMenuSeparator [internal]
unsafe fn _SHIsMenuSeparator(hm: HMENU, i: INT) -> BOOL {
    let mut mii: MENUITEMINFOW = zeroed();

    mii.cbSize = size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_TYPE;
    mii.cch = 0; // WARNING: We MUST initialize it to 0

    if GetMenuItemInfoW((hm).cast(), i as UINT, TRUE, &mut mii) == FALSE {
        return FALSE;
    }

    if (mii.fType & MFT_SEPARATOR) != 0 {
        return TRUE;
    }

    FALSE
}

// ════════════════════════════════════════════════════════════════════════════════════════
//  Exports (all WINAPI extern "system")
// ════════════════════════════════════════════════════════════════════════════════════════

/// FileMenu_Create [SHELL32.114]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_Create(
    crBorderColor: COLORREF,
    nBorderWidth: INT,
    hBorderBmp: HBITMAP,
    _nSelHeight: INT,
    _uFlags: UINT,
) -> HMENU {
    let mut MenuInfo: MENUINFO = zeroed();

    let hMenu = CreatePopupMenu();

    let menudata = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (size_of::<FMINFO>()) as _) as LPFMINFO;
    (*menudata).crBorderColor = crBorderColor;
    (*menudata).nBorderWidth = nBorderWidth;
    (*menudata).hBorderBmp = hBorderBmp;

    MenuInfo.cbSize = size_of::<MENUINFO>() as DWORD;
    MenuInfo.fMask = MIM_MENUDATA;
    MenuInfo.dwMenuData = menudata as ULONG_PTR;
    SetMenuInfo(hMenu, &MenuInfo);

    (hMenu).cast()
}

/// FileMenu_Destroy [SHELL32.118]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_Destroy(hmenu: HMENU) {
    FileMenu_DeleteAllItems(hmenu);

    let menudata = FM_GetMenuInfo(hmenu);

    SHFree((*menudata).pidl as LPVOID);
    HeapFree(GetProcessHeap(), 0, menudata as LPVOID);

    DestroyMenu((hmenu).cast());
}

/// FileMenu_AppendItemAW
#[no_mangle]
pub unsafe extern "system" fn FileMenu_AppendItemAW(
    hMenu: HMENU,
    lpText: LPCVOID,
    uID: UINT,
    icon: INT,
    hMenuPopup: HMENU,
    nItemHeight: INT,
) -> BOOL {
    let ret: BOOL;

    if lpText.is_null() {
        return FALSE;
    }

    if shell_os_is_unicode() || lpText == FM_SEPARATOR as LPCVOID {
        ret = FileMenu_AppendItemW(hMenu, lpText as LPCWSTR, uID, icon, hMenuPopup, nItemHeight);
    } else {
        let len = MultiByteToWideChar(CP_ACP, 0, lpText as LPCSTR, -1, null_mut(), 0);
        let lpszText = HeapAlloc(GetProcessHeap(), 0, (len as usize * size_of::<WCHAR>()) as _) as LPWSTR;
        if lpszText.is_null() {
            return FALSE;
        }
        MultiByteToWideChar(CP_ACP, 0, lpText as LPCSTR, -1, lpszText, len);
        ret = FileMenu_AppendItemW(hMenu, lpszText, uID, icon, hMenuPopup, nItemHeight);
        HeapFree(GetProcessHeap(), 0, lpszText as LPVOID);
    }

    ret
}

/// FileMenu_InsertUsingPidl [SHELL32.110]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_InsertUsingPidl(
    hmenu: HMENU,
    uID: UINT,
    pidl: LPCITEMIDLIST,
    uFlags: UINT,
    uEnumFlags: UINT,
    lpfnCallback: LPFNFMCALLBACK,
) -> INT {
    bAbortInit = FALSE;

    FM_SetMenuParameter(hmenu, uID, pidl, uFlags, uEnumFlags, lpfnCallback);

    FM_InitMenuPopup(hmenu, null())
}

/// FileMenu_ReplaceUsingPidl [SHELL32.113]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_ReplaceUsingPidl(
    hmenu: HMENU,
    uID: UINT,
    pidl: LPCITEMIDLIST,
    uEnumFlags: UINT,
    lpfnCallback: LPFNFMCALLBACK,
) -> INT {
    FileMenu_DeleteAllItems(hmenu);

    FM_SetMenuParameter(hmenu, uID, pidl, 0, uEnumFlags, lpfnCallback);

    FM_InitMenuPopup(hmenu, null())
}

/// FileMenu_Invalidate [SHELL32.111]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_Invalidate(_hMenu: HMENU) {}

/// FileMenu_FindSubMenuByPidl [SHELL32.106]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_FindSubMenuByPidl(
    _hMenu: HMENU,
    _pidl: LPCITEMIDLIST,
) -> HMENU {
    null_mut()
}

/// FileMenu_AppendFilesForPidl [SHELL32.124]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_AppendFilesForPidl(
    hmenu: HMENU,
    pidl: LPCITEMIDLIST,
    bAddSeparator: BOOL,
) -> INT {
    let menudata = FM_GetMenuInfo(hmenu);

    (*menudata).bInitialized = FALSE;

    FM_InitMenuPopup(hmenu, pidl);

    if bAddSeparator != FALSE {
        FileMenu_AppendItemW(hmenu, FM_SEPARATOR, 0, 0, null_mut(), FM_DEFAULT_HEIGHT);
    }

    0
}

/// FileMenu_AddFilesForPidl [SHELL32.125]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_AddFilesForPidl(
    hmenu: HMENU,
    _uReserved: UINT,
    uID: UINT,
    pidl: LPCITEMIDLIST,
    uFlags: UINT,
    uEnumFlags: UINT,
    lpfnCallback: LPFNFMCALLBACK,
) -> INT {
    FileMenu_InsertUsingPidl(hmenu, uID, pidl, uFlags, uEnumFlags, lpfnCallback)
}

/// FileMenu_TrackPopupMenuEx [SHELL32.116]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_TrackPopupMenuEx(
    hMenu: HMENU,
    uFlags: UINT,
    x: INT,
    y: INT,
    hWnd: HWND,
    lptpm: LPTPMPARAMS,
) -> BOOL {
    TrackPopupMenuEx((hMenu).cast(), uFlags, x, y, hWnd, lptpm)
}

/// FileMenu_GetLastSelectedItemPidls [SHELL32.107]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_GetLastSelectedItemPidls(
    _uReserved: UINT,
    _ppidlFolder: *mut LPCITEMIDLIST,
    _ppidlItem: *mut LPCITEMIDLIST,
) -> BOOL {
    FALSE
}

// owner-draw geometry (file-private #defines)
const FM_ICON_SIZE: INT = 16;
const FM_Y_SPACE: INT = 4;
const FM_SPACE1: INT = 4;
const FM_SPACE2: INT = 2;
const FM_LEFTBORDER: INT = 2;
const FM_RIGHTBORDER: INT = 8;

/// FileMenu_MeasureItem [SHELL32.112]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_MeasureItem(
    hWnd: HWND,
    lpmis: LPMEASUREITEMSTRUCT,
) -> LRESULT {
    let pMyItem = (*lpmis).itemData as LPFMITEM;
    let hdc = GetDC(hWnd);
    let mut size: SIZE = zeroed();

    GetTextExtentPoint32W(
        hdc,
        (*pMyItem).szItemText.as_ptr(),
        (*pMyItem).cchItemText,
        &mut size,
    );

    (*lpmis).itemWidth =
        (size.cx + FM_LEFTBORDER + FM_ICON_SIZE + FM_SPACE1 + FM_SPACE2 + FM_RIGHTBORDER) as UINT;
    (*lpmis).itemHeight = if size.cy > (FM_ICON_SIZE + FM_Y_SPACE) {
        size.cy as UINT
    } else {
        (FM_ICON_SIZE + FM_Y_SPACE) as UINT
    };

    // add the menubitmap
    let menuinfo = FM_GetMenuInfo((*pMyItem).hMenu);
    if (*menuinfo).nBorderWidth != 0 {
        (*lpmis).itemWidth += (*menuinfo).nBorderWidth as UINT;
    }

    ReleaseDC(hWnd, hdc);
    0
}

/// FileMenu_DrawItem [SHELL32.105]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_DrawItem(hWnd: HWND, lpdis: LPDRAWITEMSTRUCT) -> LRESULT {
    let pMyItem = (*lpdis).itemData as LPFMITEM;
    let clrPrevText: COLORREF;
    let clrPrevBkgnd: COLORREF;
    let xi: INT;
    let yi: INT;
    let xt: INT;
    let yt: INT;
    let mut hImageList: HIMAGELIST = null_mut();

    if ((*lpdis).itemState & ODS_SELECTED) != 0 {
        clrPrevText = SetTextColor((*lpdis).hDC, GetSysColor(COLOR_HIGHLIGHTTEXT));
        clrPrevBkgnd = SetBkColor((*lpdis).hDC, GetSysColor(COLOR_HIGHLIGHT));
    } else {
        clrPrevText = SetTextColor((*lpdis).hDC, GetSysColor(COLOR_MENUTEXT));
        clrPrevBkgnd = SetBkColor((*lpdis).hDC, GetSysColor(COLOR_MENU));
    }

    let mut TextRect: RECT = (*lpdis).rcItem;

    // add the menubitmap
    let menuinfo = FM_GetMenuInfo((*pMyItem).hMenu);
    if (*menuinfo).nBorderWidth != 0 {
        TextRect.left += (*menuinfo).nBorderWidth;
    }

    TextRect.left += FM_LEFTBORDER;
    xi = TextRect.left + FM_SPACE1;
    yi = TextRect.top + FM_Y_SPACE / 2;
    TextRect.bottom -= FM_Y_SPACE / 2;

    xt = xi + FM_ICON_SIZE + FM_SPACE2;
    yt = yi;

    ExtTextOutW(
        (*lpdis).hDC,
        xt,
        yt,
        ETO_OPAQUE,
        &TextRect,
        (*pMyItem).szItemText.as_ptr(),
        (*pMyItem).cchItemText as UINT,
        null(),
    );

    Shell_GetImageLists(null_mut(), &mut hImageList);
    ImageList_Draw(
        (hImageList).cast(),
        (*pMyItem).iIconIndex,
        (*lpdis).hDC,
        xi,
        yi,
        ILD_NORMAL,
    );

    SetTextColor((*lpdis).hDC, clrPrevText);
    SetBkColor((*lpdis).hDC, clrPrevBkgnd);

    TRUE as LRESULT
}

/// FileMenu_InitMenuPopup [SHELL32.109]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_InitMenuPopup(hmenu: HMENU) -> BOOL {
    FM_InitMenuPopup(hmenu, null());
    TRUE
}

/// FileMenu_HandleMenuChar [SHELL32.108]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_HandleMenuChar(_hMenu: HMENU, _wParam: WPARAM) -> LRESULT {
    0
}

/// FileMenu_DeleteAllItems [SHELL32.104]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_DeleteAllItems(hmenu: HMENU) -> BOOL {
    let mut mii: MENUITEMINFOW = zeroed();

    mii.cbSize = size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_SUBMENU | MIIM_DATA;

    let mut i: INT = 0;
    while i < GetMenuItemCount((hmenu).cast()) {
        GetMenuItemInfoW((hmenu).cast(), i as UINT, TRUE, &mut mii);

        SHFree(mii.dwItemData as LPVOID);

        if !mii.hSubMenu.is_null() {
            FileMenu_Destroy((mii.hSubMenu).cast());
        }
        i += 1;
    }

    while DeleteMenu((hmenu).cast(), 0, MF_BYPOSITION) != FALSE {}

    let menudata = FM_GetMenuInfo(hmenu);

    (*menudata).bInitialized = FALSE;

    TRUE
}

/// FileMenu_DeleteItemByCmd [SHELL32.117]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_DeleteItemByCmd(hMenu: HMENU, uID: UINT) -> BOOL {
    let mut mii: MENUITEMINFOW = zeroed();

    mii.cbSize = size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_SUBMENU;

    GetMenuItemInfoW((hMenu).cast(), uID, FALSE, &mut mii);
    if !mii.hSubMenu.is_null() {
        // FIXME: Do what?
    }

    DeleteMenu((hMenu).cast(), MF_BYCOMMAND, uID);
    TRUE
}

/// FileMenu_DeleteItemByIndex [SHELL32.140]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_DeleteItemByIndex(hMenu: HMENU, uPos: UINT) -> BOOL {
    let mut mii: MENUITEMINFOW = zeroed();

    mii.cbSize = size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_SUBMENU;

    GetMenuItemInfoW((hMenu).cast(), uPos, TRUE, &mut mii);
    if !mii.hSubMenu.is_null() {
        // FIXME: Do what?
    }

    DeleteMenu((hMenu).cast(), MF_BYPOSITION, uPos);
    TRUE
}

/// FileMenu_DeleteItemByFirstID [SHELL32.141]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_DeleteItemByFirstID(_hMenu: HMENU, _uID: UINT) -> BOOL {
    FALSE
}

/// FileMenu_DeleteSeparator [SHELL32.142]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_DeleteSeparator(_hMenu: HMENU) -> BOOL {
    FALSE
}

/// FileMenu_EnableItemByCmd [SHELL32.143]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_EnableItemByCmd(
    _hMenu: HMENU,
    _uID: UINT,
    _bEnable: BOOL,
) -> BOOL {
    FALSE
}

/// FileMenu_GetItemExtent [SHELL32.144]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_GetItemExtent(hMenu: HMENU, uPos: UINT) -> DWORD {
    let mut rect: RECT = zeroed();

    if GetMenuItemRect(null_mut(), (hMenu).cast(), uPos, &mut rect) != FALSE {
        // ((rect.right-rect.left)<<16) + (rect.top-rect.bottom) — wrapping to mirror C.
        return (rect.right.wrapping_sub(rect.left).wrapping_shl(16))
            .wrapping_add(rect.top.wrapping_sub(rect.bottom)) as DWORD;
    }
    0x00100010 // FIXME
}

/// FileMenu_AbortInitMenu [SHELL32.120]
#[no_mangle]
pub unsafe extern "system" fn FileMenu_AbortInitMenu() {
    bAbortInit = TRUE;
}

/// SHFind_InitMenuPopup [SHELL32.149] — stub
#[no_mangle]
pub unsafe extern "system" fn SHFind_InitMenuPopup(
    _hMenu: HMENU,
    _hWndParent: HWND,
    _w: UINT,
    _x: UINT,
) -> *mut IContextMenu {
    null_mut()
}

/// Shell_MergeMenus [SHELL32.67]
#[no_mangle]
pub unsafe extern "system" fn Shell_MergeMenus(
    hmDst: HMENU,
    hmSrc: HMENU,
    mut uInsert: UINT,
    uIDAdjust: UINT,
    uIDAdjustMax: UINT,
    uFlags: ULONG,
) -> UINT {
    let mut nItem: INT;
    let mut hmSubMenu: HMENU;
    let mut bAlreadySeparated: BOOL;
    let mut miiSrc: MENUITEMINFOW = zeroed();
    let mut szName: [WCHAR; 256] = [0; 256];
    let mut uTemp: UINT;
    let mut uIDMax: UINT = uIDAdjust;

    if hmDst.is_null() || hmSrc.is_null() {
        return uIDMax;
    }

    nItem = GetMenuItemCount((hmDst).cast());
    if nItem == -1 {
        return uIDMax;
    }

    if uInsert >= nItem as UINT {
        // insert position inside menu?
        uInsert = nItem as UINT; // append on the end
        bAlreadySeparated = TRUE;
    } else {
        bAlreadySeparated = _SHIsMenuSeparator(hmDst, uInsert as INT);
    }

    if (uFlags & MM_ADDSEPARATOR) != 0 && bAlreadySeparated == FALSE {
        // Add a separator between the menus
        InsertMenuA((hmDst).cast(), uInsert, MF_BYPOSITION | MF_SEPARATOR, 0, null());
        bAlreadySeparated = TRUE;
    }

    // Go through the menu items and clone them
    nItem = GetMenuItemCount((hmSrc).cast()) - 1;
    while nItem >= 0 {
        miiSrc.cbSize = size_of::<MENUITEMINFOW>() as UINT;
        miiSrc.fMask =
            MIIM_STATE | MIIM_ID | MIIM_SUBMENU | MIIM_CHECKMARKS | MIIM_TYPE | MIIM_DATA;

        // We need to reset this every time through the loop in case menus DON'T have IDs
        miiSrc.fType = MFT_STRING;
        miiSrc.dwTypeData = szName.as_mut_ptr();
        miiSrc.dwItemData = 0;
        miiSrc.cch = (szName.len()) as UINT; // sizeof(szName)/sizeof(WCHAR)

        if GetMenuItemInfoW((hmSrc).cast(), nItem as UINT, TRUE, &mut miiSrc) == FALSE {
            nItem -= 1;
            continue;
        }

        if (miiSrc.fType & MFT_SEPARATOR) != 0 {
            // This is a separator; don't put two of them in a row
            if bAlreadySeparated != FALSE {
                nItem -= 1;
                continue;
            }

            bAlreadySeparated = TRUE;
        } else if !miiSrc.hSubMenu.is_null() {
            if (uFlags & MM_SUBMENUSHAVEIDS) != 0 {
                miiSrc.wID += uIDAdjust; // add uIDAdjust to the ID

                if miiSrc.wID > uIDAdjustMax {
                    // skip IDs higher than uIDAdjustMax
                    nItem -= 1;
                    continue;
                }

                if uIDMax <= miiSrc.wID {
                    // remember the highest ID
                    uIDMax = miiSrc.wID + 1;
                }
            } else {
                miiSrc.fMask &= !MIIM_ID; // Don't set IDs for submenus that didn't have them already
            }
            hmSubMenu = (miiSrc.hSubMenu).cast();

            miiSrc.hSubMenu = CreatePopupMenu();

            if miiSrc.hSubMenu.is_null() {
                return uIDMax;
            }

            uTemp = Shell_MergeMenus(
                (miiSrc.hSubMenu).cast(),
                hmSubMenu,
                0,
                uIDAdjust,
                uIDAdjustMax,
                uFlags & MM_SUBMENUSHAVEIDS,
            );

            if uIDMax <= uTemp {
                uIDMax = uTemp;
            }

            bAlreadySeparated = FALSE;
        } else {
            // normal menu item
            miiSrc.wID += uIDAdjust; // add uIDAdjust to the ID

            if miiSrc.wID > uIDAdjustMax {
                // skip IDs higher than uIDAdjustMax
                nItem -= 1;
                continue;
            }

            if uIDMax <= miiSrc.wID {
                // remember the highest ID
                uIDMax = miiSrc.wID + 1;
            }

            bAlreadySeparated = FALSE;
        }

        if InsertMenuItemW((hmDst).cast(), uInsert, TRUE, &miiSrc) == FALSE {
            return uIDMax;
        }

        nItem -= 1;
    }

    // Ensure the correct number of separators at the beginning of the inserted menu items
    if uInsert == 0 {
        if bAlreadySeparated != FALSE {
            DeleteMenu((hmDst).cast(), uInsert, MF_BYPOSITION);
        }
    } else if _SHIsMenuSeparator(hmDst, (uInsert - 1) as INT) != FALSE {
        if bAlreadySeparated != FALSE {
            DeleteMenu((hmDst).cast(), uInsert, MF_BYPOSITION);
        }
    } else if (uFlags & MM_ADDSEPARATOR) != 0 && bAlreadySeparated == FALSE {
        // Add a separator between the menus
        InsertMenuW((hmDst).cast(), uInsert, MF_BYPOSITION | MF_SEPARATOR, 0, null());
    }

    uIDMax
}

// ADDED-TO-TYPES?: ULONG, COLORREF, HMENU/HDC/HBITMAP/HBRUSH/HIMAGELIST, WPARAM/LRESULT,
// ITEMIDLIST/LPITEMIDLIST/LPCITEMIDLIST, GUID/IID/REFIID, RECT, SIZE, MENUINFO,
// MENUITEMINFOW, MEASUREITEMSTRUCT, DRAWITEMSTRUCT, TPMPARAMS — common win32/shell layouts
// worth hoisting to types.rs (ITEMIDLIST/GUID/RECT/POINT already duplicated across
// clipboard.rs/shellole.rs; hoist once). FMINFO/FMITEM/LPFNFMCALLBACK and the
// IShellFolder/IEnumIDList vtables are shlmenu/shell-specific (keep file-local or in a
// shared shell-com module).
