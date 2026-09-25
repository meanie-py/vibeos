// rustconv-assembly: free-functions
// Deterministic UTF-16 literals from the selected C++ helper closure.
use crate::prelude::*;
pub static CPP_COM_WSTR_1DFA8F4A4E17: [WCHAR; 13] = [0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x0000];
pub static CPP_COM_WSTR_7CBCF6E893AD: [WCHAR; 8] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000];
// CControlPanelFolder.cpp:31. NOT emitted by the assembler: the two g_RegFolderInfo string
// fields came back pointing at statics that were never declared -- the model INVENTED two
// plausible-looking CPP_COM_WSTR_<digits> names for them. The call/fact seal covers constants
// inside function bodies; file-scope initializer data goes through the preamble, which has no
// such protection. It failed loudly here only because an undeclared name does not compile.
// This is the real literal, transcribed from the C++; pszName's is the already-declared
// "ControlPanel" below.
pub static CPP_COM_WSTR_CPANEL_RELPATH: [WCHAR; 82] = [0x003A, 0x003A, 0x007B, 0x0032, 0x0030, 0x0044, 0x0030, 0x0034, 0x0046, 0x0045, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0038, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x005C, 0x003A, 0x003A, 0x007B, 0x0032, 0x0031, 0x0045, 0x0043, 0x0032, 0x0030, 0x0032, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0044, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x0000];
pub static CPP_COM_WSTR_804899D06140: [WCHAR; 9] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_8BCDE6289FB6: [WCHAR; 23] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x002E, 0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_8E7284B41D6C: [WCHAR; 18] = [0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x0057, 0x004F, 0x0057, 0x0036, 0x0034, 0x0000];
pub static CPP_COM_WSTR_CB760546C01C: [WCHAR; 64] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0025, 0x0073, 0x005C, 0x004E, 0x0061, 0x006D, 0x0065, 0x0053, 0x0070, 0x0061, 0x0063, 0x0065, 0x0000];

use core::ffi::c_void;
// Substrate comes from the bindgen island via `prelude`, and from nowhere else.
// Re-importing the same names through shellcom/types/win32 does not add anything the
// prelude lacks -- it supplies them by a second path, and every one becomes ambiguous
// at the use site. See prelude.rs for why this is "instead of", not "as well as".

/// shlobj.h. `sdk_consts` carries SHCOLSTATE_TYPE_STR but not SHCOLSTATE_ONBYDEFAULT, and the
/// island carries neither, so every flipped folder module in this crate keeps its own copy
/// (drives_folder.rs, cfs_folder.rs, recycle_bin.rs, printers_enum.rs, ...). Values copied from
/// those, not derived.
pub const SHCOLSTATE_TYPE_STR: DWORD = 1;
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

// ---- file-scope preamble (translated) ----
//
// REGFOLDERINFO/REGFOLDERINITDATA/CRegFolder_CreateInstance (shfldr.h) and `shvheader`/
// `ControlPanelSFHeader` (this TU's own file-scope preamble) are shared across the three
// slices this one CControlPanelFolder.cpp TU was cut into. This file (the free-functions
// slice) is the designated owner: the definitions live here, `pub`, and CControlPanelEnum.rs /
// CCPLItemMenu.rs import them by name from `crate::control_panel_folder_tu` instead of each
// carrying its own copy.
//
// REGFOLDERINFO/REGFOLDERINITDATA are declared locally rather than taken from the island (which
// has same-named/same-shaped structs) because g_RegFolderInfo below needs `unsafe impl Sync`
// (raw pointer fields aren't auto-Sync) -- implementing that trait for the island's SHARED type
// would collide with any other TU doing the same for ITS OWN g_RegFolderInfo (CDesktopFolder.rs,
// CDrivesFolder.rs both build one). A locally-declared, glob-shadowing struct keeps the impl
// scoped to just this file, at zero cross-file collision risk.
#[repr(C)]
pub struct REGFOLDERINFO {
    pub PidlType: BYTE,
    pub Count: BYTE,
    pub Items: *const c_void,
    pub clsid: *const CLSID,
    pub pszParsingPath: PCWSTR,
    pub pszEnumKeyName: PCWSTR,
}
// Read-only data written once at link time, never mutated -- matches C's `static const`
// (debughlp.rs's InterfaceDescEntry / recycle_bin.rs's columninfo do the same).
unsafe impl Sync for REGFOLDERINFO {}

#[repr(C)]
pub struct REGFOLDERINITDATA {
    pub psfOuter: *mut IShellFolder,
    pub pInfo: *const REGFOLDERINFO,
}
pub type PREGFOLDERINITDATA = *mut REGFOLDERINITDATA;

// shfldr.h: `HRESULT CRegFolder_CreateInstance(PREGFOLDERINITDATA pInit, LPCITEMIDLIST
// pidlRoot, REFIID riid, void **ppv);` -- not WINAPI-decorated, so cdecl (`extern "C"`).
// CRegFolder.cpp stays C++; this is the only bridge these three files need into it.
extern "C" {
    fn CRegFolder_CreateInstance(pInit: PREGFOLDERINITDATA, pidlRoot: LPCITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
}

pub static g_RegFolderInfo: REGFOLDERINFO = REGFOLDERINFO {
    PidlType: PT_CONTROLS_NEWREGITEM as BYTE,
    Count: 0,
    Items: core::ptr::null(),
    clsid: core::ptr::addr_of!(CLSID_ControlPanel),
    pszParsingPath: CPP_COM_WSTR_CPANEL_RELPATH.as_ptr(),
    pszEnumKeyName: CPP_COM_WSTR_1DFA8F4A4E17.as_ptr(),
};

#[repr(C)]
pub struct shvheader {
    pub colnameid: WORD,
    pub colstate: WORD,
    pub fmt: WORD,
    pub cxChar: WORD,
}

pub static ControlPanelSFHeader: [shvheader; 2] = [
    shvheader {
        colnameid: IDS_SHV_COLUMN_NAME as WORD,
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: LVCFMT_LEFT as WORD,
        cxChar: 20,
    },
    shvheader {
        colnameid: IDS_SHV_COLUMN_COMMENTS as WORD,
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD,
        fmt: LVCFMT_LEFT as WORD,
        cxChar: 80,
    },
];

pub const CONTROLPANEL_COL_NAME: usize = 0;
pub const CONTROLPANEL_COL_COMMENT: usize = 1;
pub const CONTROLPANEL_COL_COUNT: usize = 2;

use crate::debughlp::pcheck;
use crate::control_panel_enum::{IDefaultExtractIconInit, IID_IDefaultExtractIconInit};

// _ILGetType is `static inline` in wine/pidl.h (no exported symbol) -- every TU that needs it
// carries its own copy; matches control_panel_folder.rs / desktop_folder.rs / cfs_folder.rs /
// recycle_bin.rs.
#[inline]
unsafe fn _ILGetType(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 {
        *(pidl as *const u8).add(2)
    } else {
        0
    }
}

// `_ILGetDataPointer` is a real exported symbol (wine/pidl.rs), but that translation is typed
// against `crate::types`, a different nominal type than this island-first TU's LPCITEMIDLIST/
// LPPIDLDATA of the same layout. Re-declare it here against the island types instead of casting
// at every call site -- CControlPanelEnum.rs / CCPLItemMenu.rs already carry the same local
// redeclaration.
extern "C" {
    fn _ILGetDataPointer(pidl: LPCITEMIDLIST) -> LPPIDLDATA;
}

// shobjidl.h SIGDN_DESKTOPABSOLUTEPARSING -- the island only carries it under its mangled WIDL
// enum constant name; every TU that needs it declares its own copy (open_control_panel.rs,
// wine/pidl.rs, shellcom.rs all do the same). `SIGDN` itself IS a clean island alias.
const SIGDN_DESKTOPABSOLUTEPARSING: SIGDN = 0x8002_8000;

extern "system" {
    fn SHGetSpecialFolderLocation(owner: HWND, csidl: INT, pidl: *mut LPITEMIDLIST) -> HRESULT;
    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, name: *mut PWSTR) -> HRESULT;
}

/// minwindef.h `IS_INTRESOURCE` -- true when the whole value fits in the low 16 bits (a CSIDL
/// smuggled through a PCIDLIST_ABSOLUTE, per CreateCplAbsoluteParsingPath's caller below).
#[inline]
unsafe fn IS_INTRESOURCE(value: PCIDLIST_ABSOLUTE) -> bool {
    (value as usize) >> 16 == 0
}

unsafe extern "C" fn IsRegItem(pidl: LPCITEMIDLIST) -> *const CLSID {
    let type_ = _ILGetType(pidl);
    if type_ == PT_CONTROLS_OLDREGITEM as BYTE || type_ == PT_CONTROLS_NEWREGITEM as BYTE {
        return ((pidl as *const u8).add((*pidl).mkid.cb as usize - core::mem::size_of::<CLSID>())) as *const CLSID;
    }
    core::ptr::null()
}
pub unsafe extern "C" fn _ILCreateCPanelApplet(
    pszName: PCWSTR,
    pszDisplayName: PCWSTR,
    pszComment: PCWSTR,
    iIconIdx: i32,
) -> LPITEMIDLIST {
    let mut pCP: *mut PIDLCPanelStruct;
    let mut pidl: LPITEMIDLIST;
    let mut pData: LPPIDLDATA;
    let mut cchName: i32;
    let mut cchDisplayName: i32;
    let mut cchComment: i32;
    let mut cbData: i32;

    cchName = wcslen(pszName) as i32;
    cchDisplayName = wcslen(pszDisplayName) as i32;
    cchComment = wcslen(pszComment) as i32;

    cbData = (core::mem::size_of::<u16>()
        + core::mem::size_of::<u8>()
        + core::mem::size_of::<PIDLCPanelStruct>()
        - core::mem::size_of::<[u16; 1]>()
        + (cchName + cchDisplayName + cchComment + 3) as usize
            * core::mem::size_of::<u16>()) as i32;
    pidl = SHAlloc((cbData as usize + core::mem::size_of::<WORD>()) as SIZE_T)
        as LPITEMIDLIST;
    if pidl.is_null() {
        return core::ptr::null_mut();
    }

    (*pidl).mkid.cb = cbData as u16;
    pData = (*pidl).mkid.abID.as_mut_ptr() as LPPIDLDATA;
    (*pData).type_ = PT_CPLAPPLET as u8;

    pCP = core::ptr::addr_of_mut!((*pData).u.cpanel).cast::<PIDLCPanelStruct>();
    (*pCP).dummy = 0;
    (*pCP).iconIdx = iIconIdx as DWORD;
    // PIDLCPanelStruct is `#[repr(C, packed)]` (island): `.as_mut_ptr()` needs `&mut` of the
    // field, and a reference to a packed field is E0793 (may be unaligned). `addr_of_mut!` +
    // `.cast()` gets the same pointer without ever forming that reference.
    wcscpy(core::ptr::addr_of_mut!((*pCP).szName).cast::<WCHAR>(), pszName);
    (*pCP).offsDispName = (cchName + 1) as WORD;
    wcscpy(
        core::ptr::addr_of_mut!((*pCP).szName).cast::<WCHAR>().offset((*pCP).offsDispName as isize),
        pszDisplayName,
    );
    (*pCP).offsComment = ((*pCP).offsDispName as i32 + cchDisplayName + 1) as WORD;
    wcscpy(
        core::ptr::addr_of_mut!((*pCP).szName).cast::<WCHAR>().offset((*pCP).offsComment as isize),
        pszComment,
    );

    *(core::ptr::addr_of_mut!((*pCP).szName)
        .cast::<WCHAR>()
        .offset(((*pCP).offsComment as i32 + cchComment + 1) as isize)
        as *mut u16) = 0;

    pcheck(pidl.cast());

    return pidl;
}
unsafe extern "C" fn _ILGetCPanelPointer(pidl: LPCITEMIDLIST) -> *mut PIDLCPanelStruct {
    let pdata: LPPIDLDATA = _ILGetDataPointer(pidl);

    if !pdata.is_null() && (*pdata).type_ == PT_CPLAPPLET as u8 {
        return core::ptr::addr_of_mut!((*pdata).u.cpanel).cast::<PIDLCPanelStruct>();
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn CCPLExtractIcon_CreateInstance(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    let pData = _ILGetCPanelPointer(pidl);
    if pData.is_null() {
        return E_FAIL;
    }

    let mut initIcon: *mut IDefaultExtractIconInit = core::ptr::null_mut();
    let hr = SHCreateDefaultExtractIcon(
        core::ptr::addr_of!(IID_IDefaultExtractIconInit).cast(), core::ptr::addr_of_mut!(initIcon).cast()
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    // A vtable slot is a field holding a function pointer, not a method: call through
    // `lpVtbl` (see CNetFolder.rs's identical SetNormalIcon/QueryInterface call shape).
    let icon_idx: i32 = if (*pData).iconIdx as i32 != -1 {
        (*pData).iconIdx as i32
    } else {
        0
    };
    ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, core::ptr::addr_of!((*pData).szName).cast::<WCHAR>(), icon_idx);

    ((*(*initIcon).lpVtbl).QueryInterface)(initIcon, riid, ppvOut)
}
unsafe extern "C" fn GetParsingName(mut pidl: PCIDLIST_ABSOLUTE, Name: *mut PWSTR) -> HRESULT {
    let mut pidlFree: PIDLIST_ABSOLUTE = core::ptr::null_mut();
    if IS_INTRESOURCE(pidl) {
        let hr = SHGetSpecialFolderLocation(core::ptr::null_mut(), pidl as usize as INT, &mut pidlFree);
        if hr < 0 {
            return hr;
        }
        pidl = pidlFree;
    }
    let hr = SHGetNameFromIDList(pidl, SIGDN_DESKTOPABSOLUTEPARSING, Name);
    ILFree(pidlFree);
    hr
}
// CControlPanelFolder.cpp:833. Hand-written: the paid translation was refused by the call-seal
// audit, and the forensic copy under .responses/.../__rejected__/ shows why it deserved to be
// -- it had permuted three different calls, testing `StringCchPrintfW(hr, 0)` as the SUCCEEDED
// predicate and then calling `SHFree` with StringCchPrintfW's six arguments. Nothing about that
// is a near miss; the seal is the only thing that would have caught it.
unsafe extern "C" fn CreateCplAbsoluteParsingPath(
    Prefix: PCWSTR,
    InFolderParse: PCWSTR,
    Buf: PWSTR,
    cchBuf: UINT,
) -> HRESULT {
    let mut cpfolder: PWSTR = core::ptr::null_mut();
    let mut hr: HRESULT = GetParsingName(
        CSIDL_CONTROLS as PCIDLIST_ABSOLUTE,
        core::ptr::addr_of_mut!(cpfolder),
    );
    if hr >= 0 {
        // L"%s\\%s%s"
        hr = strsafe_rust::StringCchPrintfW!(
            Buf,
            cchBuf as usize,
            CPP_COM_WSTR_7CBCF6E893AD.as_ptr(),
            cpfolder,
            Prefix,
            InFolderParse,
        );
        SHFree(cpfolder as LPVOID);
    }
    hr
}

unsafe extern "C" fn FindExeCplClass(Canonical: PCWSTR, hKey: HKEY, Wow64: BOOL, clsid: PWSTR) -> HRESULT {
    let mut hr = HRESULT_FROM_WIN32(ERROR_NOT_FOUND as LONG);
    let mut hNSKey: HKEY = core::ptr::null_mut();
    let mut key: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    wsprintfW(key.as_mut_ptr(), CPP_COM_WSTR_CB760546C01C.as_ptr(),
              if Wow64 != 0 { CPP_COM_WSTR_8E7284B41D6C.as_ptr() } else { CPP_COM_WSTR_1DFA8F4A4E17.as_ptr() });
    let error = RegOpenKeyExW(hKey, key.as_ptr(), 0, KEY_READ, &mut hNSKey);
    if error != 0 {
        return HRESULT_FROM_WIN32(error);
    }
    let mut i: u32 = 0;
    loop {
        if RegEnumKeyW(hNSKey, i, key.as_mut_ptr(), _countof!(key)) != ERROR_SUCCESS as LSTATUS {
            break;
        }
        let mut validate: IID = core::mem::zeroed();
        if IIDFromString(key.as_ptr(), core::ptr::addr_of_mut!(validate).cast()) >= 0 {
            wsprintfW(buf.as_mut_ptr(), CPP_COM_WSTR_804899D06140.as_ptr(), key.as_ptr());
            let mut cb: u32 = core::mem::size_of_val(&buf) as u32;
            if RegGetValueW(HKEY_CLASSES_ROOT, buf.as_ptr(), CPP_COM_WSTR_8BCDE6289FB6.as_ptr(),
                             RRF_RT_REG_SZ, core::ptr::null_mut(), buf.as_mut_ptr() as *mut core::ffi::c_void, &mut cb) == ERROR_SUCCESS as LSTATUS {
                if lstrcmpiW(buf.as_ptr(), Canonical) == 0 {
                    lstrcpyW(clsid, key.as_ptr());
                    hr = S_OK;
                }
            }
        }
        i += 1;
    }
    RegCloseKey(hNSKey);
    hr
}

unsafe fn FindExeCplClass_2(Canonical: PCWSTR, clsid: PWSTR) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    if hr < 0 {
        hr = FindExeCplClass(Canonical, HKEY_CURRENT_USER, FALSE, clsid);
    }
    if hr < 0 {
        hr = FindExeCplClass(Canonical, HKEY_CURRENT_USER, TRUE, clsid);
    }
    if hr < 0 {
        hr = FindExeCplClass(Canonical, HKEY_LOCAL_MACHINE, FALSE, clsid);
    }
    if hr < 0 {
        hr = FindExeCplClass(Canonical, HKEY_LOCAL_MACHINE, TRUE, clsid);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn CControlPanelFolder_CreateRegFolder_RustBridge(
    psfOuter: *mut IShellFolder,
    pidlRoot: LPCITEMIDLIST,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut RegInit = REGFOLDERINITDATA {
        psfOuter: psfOuter,
        pInfo: core::ptr::addr_of!(g_RegFolderInfo),
    };
    CRegFolder_CreateInstance(core::ptr::addr_of_mut!(RegInit), pidlRoot, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CControlPanelEnum_CreateInstance_RustBridge(
    dwFlags: u32,
    pRegEnumerator: *mut IEnumIDList,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // ShellObjectCreatorInit<T> (shellutils.h) is a C++ template with no translated body --
    // call the concrete factory it would have instantiated directly (CNetFolder.rs precedent).
    // `control_panel_enum::IEnumIDList` is that module's own lpVtbl-shaped COM scaffold, a
    // same-layout but nominally distinct type from this file's island/shellcom IEnumIDList.
    crate::control_panel_enum::CControlPanelEnum_CreateInstance(dwFlags, pRegEnumerator.cast(), riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CCPLItemMenu_CreateInstance_RustBridge(
    cidl: u32,
    apidl: PCUITEMID_CHILD_ARRAY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    crate::cpl_item_menu::CCPLItemMenu_CreateInstance(cidl, apidl, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CCPLExtractIcon_CreateInstance_RustBridge(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CCPLExtractIcon_CreateInstance(psf, pidl, riid, ppvOut)
}


// ============================================================================
// ORIGINAL C SOURCE (this slice only)
// ============================================================================
//
// /*
//  * Control panel folder
//  *
//  * Copyright 2003 Martin Fuchs
//  * Copyright 2009 Andrew Hill
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <precomp.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// static const REGFOLDERINFO g_RegFolderInfo =
// {
//     PT_CONTROLS_NEWREGITEM,
//     0, NULL,
//     CLSID_ControlPanel,
//     L"::{20D04FE0-3AEA-1069-A2D8-08002B30309D}\\::{21EC2020-3AEA-1069-A2DD-08002B30309D}",
//     L"ControlPanel",
// };
//
// /***********************************************************************
// *   control panel implementation in shell namespace
// */
//
// class CControlPanelEnum :
//     public CEnumIDListBase
// {
//     public:
//         CControlPanelEnum();
//         ~CControlPanelEnum();
//         HRESULT WINAPI Initialize(DWORD dwFlags, IEnumIDList* pRegEnumerator);
//         BOOL RegisterCPanelApp(LPCWSTR path);
//         int RegisterRegistryCPanelApps(HKEY hkey_root, LPCWSTR szRepPath);
//         BOOL CreateCPanelEnumList(DWORD dwFlags);
//
//         BEGIN_COM_MAP(CControlPanelEnum)
//         COM_INTERFACE_ENTRY_IID(IID_IEnumIDList, IEnumIDList)
//         END_COM_MAP()
// };
//
// /***********************************************************************
// *   IShellFolder [ControlPanel] implementation
// */
//
// static const shvheader ControlPanelSFHeader[] = {
//     {IDS_SHV_COLUMN_NAME, SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT, LVCFMT_LEFT, 20},/*FIXME*/
//     {IDS_SHV_COLUMN_COMMENTS, SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT, LVCFMT_LEFT, 80},/*FIXME*/
// };
//
// enum controlpanel_columns
// {
//     CONTROLPANEL_COL_NAME,
//     CONTROLPANEL_COL_COMMENT,
//     CONTROLPANEL_COL_COUNT,
// };
//
// static const CLSID* IsRegItem(LPCITEMIDLIST pidl)
// {
//     BYTE type = _ILGetType(pidl);
//     if (type == PT_CONTROLS_OLDREGITEM || type == PT_CONTROLS_NEWREGITEM)
//         return (CLSID*)((BYTE*)pidl + (pidl->mkid.cb - sizeof(CLSID)));
//     return NULL;
// }
// static LPITEMIDLIST _ILCreateCPanelApplet(LPCWSTR pszName, LPCWSTR pszDisplayName, LPCWSTR pszComment, int iIconIdx)
// {
//     PIDLCPanelStruct *pCP;
//     LPITEMIDLIST pidl;
//     LPPIDLDATA pData;
//     int cchName, cchDisplayName, cchComment, cbData;
//
//     /* Calculate lengths of given strings */
//     cchName = wcslen(pszName);
//     cchDisplayName = wcslen(pszDisplayName);
//     cchComment = wcslen(pszComment);
//
//     /* Allocate PIDL */
//     cbData = sizeof(pidl->mkid.cb) + sizeof(pData->type) + sizeof(pData->u.cpanel) - sizeof(pData->u.cpanel.szName)
//              + (cchName + cchDisplayName + cchComment + 3) * sizeof(WCHAR);
//     pidl = (LPITEMIDLIST)SHAlloc(cbData + sizeof(WORD));
//     if (!pidl)
//         return NULL;
//
//     /* Copy data to allocated memory */
//     pidl->mkid.cb = cbData;
//     pData = (PIDLDATA *)pidl->mkid.abID;
//     pData->type = PT_CPLAPPLET;
//
//     pCP = &pData->u.cpanel;
//     pCP->dummy = 0;
//     pCP->iconIdx = iIconIdx;
//     wcscpy(pCP->szName, pszName);
//     pCP->offsDispName = cchName + 1;
//     wcscpy(pCP->szName + pCP->offsDispName, pszDisplayName);
//     pCP->offsComment = pCP->offsDispName + cchDisplayName + 1;
//     wcscpy(pCP->szName + pCP->offsComment, pszComment);
//
//     /* Add PIDL NULL terminator */
//     *(WORD*)(pCP->szName + pCP->offsComment + cchComment + 1) = 0;
//
//     pcheck(pidl);
//
//     return pidl;
// }
// static PIDLCPanelStruct *_ILGetCPanelPointer(LPCITEMIDLIST pidl)
// {
//     LPPIDLDATA pdata = _ILGetDataPointer(pidl);
//
//     if (pdata && pdata->type == PT_CPLAPPLET)
//         return (PIDLCPanelStruct *) & (pdata->u.cpanel);
//
//     return NULL;
// }
// HRESULT CCPLExtractIcon_CreateInstance(IShellFolder * psf, LPCITEMIDLIST pidl, REFIID riid, LPVOID * ppvOut)
// {
//     PIDLCPanelStruct *pData = _ILGetCPanelPointer(pidl);
//     if (!pData)
//         return E_FAIL;
//
//     CComPtr<IDefaultExtractIconInit> initIcon;
//     HRESULT hr = SHCreateDefaultExtractIcon(IID_PPV_ARG(IDefaultExtractIconInit, &initIcon));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     initIcon->SetNormalIcon(pData->szName, (int)pData->iconIdx != -1 ? pData->iconIdx : 0);
//
//     return initIcon->QueryInterface(riid, ppvOut);
// }
// static HRESULT GetParsingName(PCIDLIST_ABSOLUTE pidl, PWSTR*Name)
// {
//     PIDLIST_ABSOLUTE pidlFree = NULL;
//     if (IS_INTRESOURCE(pidl))
//     {
//         HRESULT hr = SHGetSpecialFolderLocation(NULL, (UINT)(SIZE_T)pidl, &pidlFree);
//         if (FAILED(hr))
//             return hr;
//         pidl = pidlFree;
//     }
//     HRESULT hr = SHGetNameFromIDList(pidl, SIGDN_DESKTOPABSOLUTEPARSING, Name);
//     ILFree(pidlFree);
//     return hr;
// }
// static HRESULT CreateCplAbsoluteParsingPath(LPCWSTR Prefix, LPCWSTR InFolderParse, PWSTR Buf, UINT cchBuf)
// {
//     PWSTR cpfolder;
//     HRESULT hr = GetParsingName((PCIDLIST_ABSOLUTE)CSIDL_CONTROLS, &cpfolder);
//     if (SUCCEEDED(hr))
//     {
//         hr = StringCchPrintfW(Buf, cchBuf, L"%s\\%s%s", cpfolder, Prefix, InFolderParse);
//         SHFree(cpfolder);
//     }
//     return hr;
// }
// static HRESULT FindExeCplClass(LPCWSTR Canonical, HKEY hKey, BOOL Wow64, LPWSTR clsid)
// {
//     HRESULT hr = HRESULT_FROM_WIN32(ERROR_NOT_FOUND);
//     HKEY hNSKey;
//     WCHAR key[MAX_PATH], buf[MAX_PATH];
//     wsprintfW(key, L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\%s\\NameSpace",
//               Wow64 ? L"ControlPanelWOW64" : L"ControlPanel");
//     LSTATUS error = RegOpenKeyExW(hKey, key, 0, KEY_READ, &hNSKey);
//     if (error)
//         return HRESULT_FROM_WIN32(error);
//     for (DWORD i = 0; RegEnumKeyW(hNSKey, i, key, _countof(key)) == ERROR_SUCCESS; ++i)
//     {
//         IID validate;
//         if (SUCCEEDED(IIDFromString(key, &validate)))
//         {
//             wsprintfW(buf, L"CLSID\\%s", key);
//             DWORD cb = sizeof(buf);
//             if (RegGetValueW(HKEY_CLASSES_ROOT, buf, L"System.ApplicationName",
//                              RRF_RT_REG_SZ, NULL, buf, &cb) == ERROR_SUCCESS)
//             {
//                 if (!lstrcmpiW(buf, Canonical))
//                 {
//                     lstrcpyW(clsid, key);
//                     hr = S_OK;
//                 }
//             }
//         }
//     }
//     RegCloseKey(hNSKey);
//     return hr;
// }
// static HRESULT FindExeCplClass(LPCWSTR Canonical, LPWSTR clsid)
// {
//     HRESULT hr = E_FAIL;
//     if (FAILED(hr))
//         hr = FindExeCplClass(Canonical, HKEY_CURRENT_USER, FALSE, clsid);
//     if (FAILED(hr))
//         hr = FindExeCplClass(Canonical, HKEY_CURRENT_USER, TRUE, clsid);
//     if (FAILED(hr))
//         hr = FindExeCplClass(Canonical, HKEY_LOCAL_MACHINE, FALSE, clsid);
//     if (FAILED(hr))
//         hr = FindExeCplClass(Canonical, HKEY_LOCAL_MACHINE, TRUE, clsid);
//     return hr;
// }
// extern "C" HRESULT WINAPI
// CControlPanelFolder_CreateRegFolder_RustBridge(
//     IShellFolder *psfOuter, LPCITEMIDLIST pidlRoot, REFIID riid, void **ppv)
// {
//     REGFOLDERINITDATA RegInit = { psfOuter, &g_RegFolderInfo };
//     return CRegFolder_CreateInstance(&RegInit, pidlRoot, riid, ppv);
// }
// extern "C" HRESULT WINAPI
// CControlPanelEnum_CreateInstance_RustBridge(
//     DWORD dwFlags, IEnumIDList *pRegEnumerator, REFIID riid, void **ppv)
// {
//     return ShellObjectCreatorInit<CControlPanelEnum>(dwFlags, pRegEnumerator, riid, ppv);
// }
// extern "C" HRESULT WINAPI
// CCPLItemMenu_CreateInstance_RustBridge(
//     UINT cidl, PCUITEMID_CHILD_ARRAY apidl, REFIID riid, void **ppv)
// {
//     return ShellObjectCreatorInit<CCPLItemMenu>(cidl, apidl, riid, ppv);
// }
// extern "C" HRESULT WINAPI
// CCPLExtractIcon_CreateInstance_RustBridge(
//     IShellFolder *psf, LPCITEMIDLIST pidl, REFIID riid, LPVOID *ppvOut)
// {
//     return CCPLExtractIcon_CreateInstance(psf, pidl, riid, ppvOut);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
