use crate::prelude::*;
// `DFM_CMD_PROPERTIES` and the shellutils.h inline `SHELL_ExecuteControlPanelCPL` already
// have canonical copies in `desktop_folder` (CDrivesFolder.rs imports the same constant);
// import rather than re-derive, so there is one definition of each.
use crate::desktop_folder::DFM_CMD_PROPERTIES;

extern "system" {
    fn SHRunControlPanel(cpl: LPCWSTR, hwnd: HWND) -> BOOL;
}
/// `shellutils.h:218` `SHELL_ExecuteControlPanelCPL(hwnd, cpl)` := `SHRunControlPanel(cpl, hwnd)`
/// -- a header inline with no out-of-line symbol, so each TU carries its own copy (the
/// sibling `desktop_folder.rs:2239` is the same, but private to that module). Note the
/// argument ORDER swaps.
#[inline]
unsafe fn SHELL_ExecuteControlPanelCPL(hwnd: HWND, cpl: LPCWSTR) -> BOOL {
    SHRunControlPanel(cpl, hwnd)
}
use crate::shell32::shlfolder::SHELL32_DefaultContextMenuCallBack;
use crate::dialogs_item_prop::SHELL32_ShowShellExtensionProperties;
use crate::shell32::shlfolder::SH_GetApidlFromDataObject;
use crate::wine::pidl::_ILFreeaPidl;
// CRegFolder.cpp:339 `IsRequiredItem` / :105 `IsRegItem` are real methods, defined once in the
// sibling TU that owns the class (folders/CRegFolder.rs, module `reg_folder`) as
// `CRegFolder_IsRequiredItem_body` / `CRegFolder_IsRegItem_body` free fns -- this crate's
// established stand-in for a COM class's inherent methods. `CRegFolder` itself must come from
// there too: a second nominal `struct CRegFolder` in THIS file (as the machine translation had)
// has no `impl`, so `(*pRegFolder).IsRequiredItem(...)` cannot resolve, and a duplicate layout
// is free to drift from the real one besides.
use crate::reg_folder::{CRegFolder, CRegFolder_IsRegItem_body, CRegFolder_IsRequiredItem_body};
// CRegFolder.cpp:141-224 `CGuidItemExtractIcon_CreateInstance` substrate.
use crate::shell32::extracticon_free::SHCreateDefaultExtractIcon;
use crate::shell32::CExtractIcon::{IDefaultExtractIconInit, IID_IDefaultExtractIconInit};
use crate::folders::swShell32Name;
use crate::wine::pidl::{_ILGetGUIDPointer, _ILIsBitBucket};
use crate::wine::classes::{HCR_GetIconW, HCU_GetIconW};
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_279CE0FC8724: [WCHAR; 5] = [0x0046, 0x0075, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_31CCAA880800: [WCHAR; 12] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_3D93EF175FEC: [WCHAR; 60] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_63638B1ECF1C: [WCHAR; 6] = [0x0045, 0x006D, 0x0070, 0x0074, 0x0079, 0x0000];
pub static CPP_COM_WSTR_6E342AB1A0F1: [WCHAR; 7] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];
pub static CPP_COM_WSTR_804899D06140: [WCHAR; 9] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_8D44D4323B7D: [WCHAR; 4] = [0x0025, 0x0068, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A3FDF2CDA91F: [WCHAR; 15] = [0x0053, 0x006F, 0x0072, 0x0074, 0x004F, 0x0072, 0x0064, 0x0065, 0x0072, 0x0049, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub static CPP_COM_WSTR_E110B6F7562E: [WCHAR; 16] = [0x0048, 0x0069, 0x0064, 0x0065, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0056, 0x0065, 0x0072, 0x0062, 0x0073, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The machine translation re-declared `CRegFolderInfo` / `CRegFolderEnum` / `CRegFolder` here
// as second, nominally-distinct copies of the layouts the sibling TUs (CRegFolder.rs,
// CRegFolderEnum.rs) already own and export. Nothing in this file used `CRegFolderInfo` or
// `CRegFolderEnum` at all, and the one thing that DID use `CRegFolder`
// (`RegFolderContextMenuCallback`, below) needs the real one -- imported above as
// `crate::reg_folder::CRegFolder` -- to reach `CRegFolder_IsRequiredItem_body` /
// `CRegFolder_IsRegItem_body`. Per the repair brief: an unused duplicate is deleted rather
// than kept free to drift; the used one is imported rather than redefined.

/// The `CComPtr` destructor for this file's locals: releases and NULLs the slot on every exit
/// path (including an early `return`, since Rust drops in-scope locals on any return). Each
/// free-function TU in this crate defines its own copy of this shape rather than sharing one
/// (see shell32/extracticon_free.rs, control_panel_folder.rs, desktop_folder.rs, ...).
pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

// CRegFolder.cpp:155,220 `IDI_SHELL_DESKTOP` / `IDI_SHELL_DOCUMENT` (shresdef.h resource ids).
// Declared locally, matching the established per-file pattern (CRegFolder.rs:327,329,
// CRegFolderEnum.rs:172,174, cfs_folder.rs:806, recycle_bin.rs:1337, ...) rather than importing
// a sibling module's private consts.
const IDI_SHELL_DESKTOP: UINT = 35;
const IDI_SHELL_DOCUMENT: UINT = 1;
// CRegFolder.cpp:177 `SHCONTF_FOLDERS | SHCONTF_NONFOLDERS` (shobjidl.h). Same per-file pattern
// (file_sys_enum.rs:303, drives_folder.rs:956, recycle_bin.rs:950, CDrivesFolderEnum.rs:314).
const SHCONTF_FOLDERS: DWORD = 32;
const SHCONTF_NONFOLDERS: DWORD = 64;

extern "C" {
    // CRegFolder.cpp:175 `IID_PPV_ARG(IShellFolder2, &psfRecycleBin)`. Same per-file pattern
    // (my_docs_folder.rs:30, admin_tools_folder.rs:121, recycle_bin.rs:90).
    pub static IID_IShellFolder2: IID;
}

pub const COL_NAME: u32 = SHFSF_COL_NAME;
pub const COL_TYPE: u32 = SHFSF_COL_TYPE;
pub const COL_INFOTIP: u32 = SHFSF_COL_COMMENT;
pub const REGFOLDERCOLUMNCOUNT: u32 = (if COL_INFOTIP > COL_TYPE { COL_INFOTIP } else { COL_TYPE }) + 1;
#[no_mangle]
pub unsafe extern "C" fn FormatGUIDKey(
    KeyName: LPWSTR,
    KeySize: SIZE_T,
    RegPath: LPCWSTR,
    riid: *const GUID
) -> HRESULT {
    let mut xriid = [0u16; CHARS_IN_GUID as usize];
    StringFromGUID2(riid.cast(), xriid.as_mut_ptr(), xriid.len() as INT);
    strsafe_rust::StringCchPrintfW!(KeyName, KeySize as usize, RegPath, xriid.as_ptr())
}
pub unsafe extern "system" fn RegFolderContextMenuCallback(
    psf: *mut IShellFolder,
    hwnd: HWND,
    pdtobj: *mut IDataObject,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> HRESULT {
    if uMsg != DFM_INVOKECOMMAND || wParam != DFM_CMD_PROPERTIES as WPARAM {
        return SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg);
    }

    let mut pidlFolder: PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut apidl: *mut PUITEMID_CHILD = core::ptr::null_mut();
    let mut cidl: UINT = 0;
    let mut hr: HRESULT = SH_GetApidlFromDataObject(pdtobj, &mut pidlFolder, &mut apidl, &mut cidl);
    if ((hr) < 0) {
        return hr;
    }

    let pCLSID: *const CLSID;
    let pRegFolder: *mut CRegFolder = psf as *mut CRegFolder;
    // CRegFolder.cpp:936 `pRegFolder->IsRequiredItem(apidl[0])`. `apidl[0]` dereferences the
    // child array's first element (a single pidl); the machine translation's
    // `apidl.add(0 as usize)` is pointer arithmetic on `apidl` ITSELF and stays a
    // `*mut PUITEMID_CHILD` (one level too many). And `IsRequiredItem` is a real method,
    // reached here as the sibling TU's `CRegFolder_IsRequiredItem_body` free fn (see the
    // `use` at the top of this file).
    let pRequired: *const REQUIREDREGITEM = CRegFolder_IsRequiredItem_body(pRegFolder, *apidl.add(0 as usize))
        as *const REQUIREDREGITEM;
    if !pRequired.is_null() && !(*pRequired).pszCpl.is_null() {
        let mut buf: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        wsprintfW(
            buf.as_mut_ptr(),
            CPP_COM_WSTR_8D44D4323B7D.as_ptr(),
            (*pRequired).pszCpl,
        );
        hr = if SHELL_ExecuteControlPanelCPL(hwnd, buf.as_ptr()) != 0 {
            S_OK
        } else {
            E_FAIL
        };
    } else if {
        // CRegFolder.cpp:950 `pRegFolder->IsRegItem(apidl[0])`; see the note above.
        // `CRegFolder_IsRegItem_body` returns `*mut CLSID`; `pCLSID` here is `*const CLSID`.
        pCLSID = CRegFolder_IsRegItem_body(pRegFolder, *apidl.add(0 as usize)) as *const CLSID;
        !pCLSID.is_null()
    } {
        if !IsEqualGUID(core::ptr::addr_of!(CLSID_MyDocuments), pCLSID) {
            hr = SHELL32_ShowShellExtensionProperties(pCLSID, pdtobj);
        } else {
            hr = S_FALSE;
        }
    } else {
        hr = S_FALSE;
    }

    SHFree(pidlFolder.cast());
    _ILFreeaPidl(apidl, cidl);
    hr
}
// CRegFolder.cpp:995 `CRegFolder_CreateInstance` is NOT reimplemented here. It is a one-line
// wrapper over `ShellObjectCreatorInit<CRegFolder>`, and the cpp-com scaffold already
// generates exactly that factory, under exactly this name, in the sibling module
// `crate::reg_folder`. The worker emitted a second copy calling a generic
// `ShellObjectCreatorInit::<CRegFolder>` that does not exist in Rust; rustconv's free-half
// audit flagged it as "worker emitted forbidden activation factory implementation", which is
// precisely right -- the scaffold owns activation.

// CRegFolder.cpp:141-224 `CGuidItemExtractIcon_CreateInstance`. The machine translation was
// REJECTED by rustconv's free-half audit: it renamed the `EnumIDList` local to snake_case
// (defeating the seal scaffolding's match), called COM methods with `.` syntax on raw
// pointers, and hand-rolled `Release()` beside a `CComPtr` (a double free once the guard also
// runs). Hand-written here: vtable-call form throughout, one `CppComPtrGuard` per `CComPtr`
// local and no explicit `Release()` beside any of them.
#[no_mangle]
pub unsafe extern "C" fn CGuidItemExtractIcon_CreateInstance(
    pidl: LPCITEMIDLIST,
    iid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // CRegFolder.cpp:143 `CComPtr<IDefaultExtractIconInit> initIcon;`
    let mut initIcon: *mut IDefaultExtractIconInit = core::ptr::null_mut();
    let _initIcon_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(initIcon));
    let mut hr: HRESULT;
    let riid: *const GUID;
    let mut icon_idx: i32 = 0;
    let mut wTemp: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    // CRegFolder.cpp:149
    hr = SHCreateDefaultExtractIcon(
        core::ptr::addr_of!(IID_IDefaultExtractIconInit),
        core::ptr::addr_of_mut!(initIcon) as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        return hr;
    }

    // CRegFolder.cpp:153-157
    if _ILIsDesktop(pidl) != 0 {
        ((*(*initIcon).lpVtbl).SetNormalIcon)(
            initIcon,
            core::ptr::addr_of!(swShell32Name).cast(),
            -(IDI_SHELL_DESKTOP as i32),
        );
        return ((*(*initIcon).lpVtbl).QueryInterface)(initIcon, iid, ppvOut);
    }

    // CRegFolder.cpp:159-161
    riid = _ILGetGUIDPointer(pidl) as *const GUID;
    if riid.is_null() {
        return E_FAIL;
    }

    // CRegFolder.cpp:163-187 `/* Choose a correct icon for Recycle Bin (full or empty) */`
    let mut iconname: LPCWSTR = core::ptr::null();
    if _ILIsBitBucket(pidl) != 0 {
        // CRegFolder.cpp:167 `CComPtr<IEnumIDList> EnumIDList;`
        let mut EnumIDList: *mut IEnumIDList = core::ptr::null_mut();
        let _EnumIDList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(EnumIDList));
        CoInitialize(core::ptr::null_mut());

        // CRegFolder.cpp:170-171
        let mut psfRecycleBin: *mut IShellFolder2 = core::ptr::null_mut();
        let _psfRecycleBin_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfRecycleBin));
        let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
        let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));

        hr = SHGetDesktopFolder(core::ptr::addr_of_mut!(psfDesktop));

        if hr >= 0 {
            hr = ((*(*psfDesktop).lpVtbl).BindToObject)(
                psfDesktop,
                pidl,
                core::ptr::null_mut(),
                core::ptr::addr_of!(IID_IShellFolder2).cast(),
                core::ptr::addr_of_mut!(psfRecycleBin) as *mut *mut core::ffi::c_void,
            );
        }
        if hr >= 0 {
            // CRegFolder.cpp:177 `psfRecycleBin->EnumObjects(...)`. This crate's `shellcom`
            // leaves `IShellFolder2Vtbl::EnumObjects` an unusable `usize` placeholder
            // (shellcom.rs:436) because no other landed call site goes through it -- but
            // IShellFolder2 inherits IShellFolder and does not override EnumObjects, so the
            // vtable prefix (QueryInterface/AddRef/Release/ParseDisplayName/EnumObjects) is
            // layout-identical between the two interfaces. Cast to IShellFolder and call
            // through ITS (fully-typed) EnumObjects slot instead of inventing a body for the
            // placeholder.
            let psfRecycleBinAsFolder: *mut IShellFolder = psfRecycleBin.cast();
            hr = ((*(*psfRecycleBinAsFolder).lpVtbl).EnumObjects)(
                psfRecycleBinAsFolder,
                core::ptr::null_mut(),
                SHCONTF_FOLDERS | SHCONTF_NONFOLDERS,
                core::ptr::addr_of_mut!(EnumIDList),
            );
        }

        let mut itemcount: ULONG = 0;
        let mut pidl_next: LPITEMIDLIST = core::ptr::null_mut();
        // CRegFolder.cpp:181 `SUCCEEDED(hr) && (hr = EnumIDList->Next(1, &pidl, &itemcount)) ==
        // S_OK` -- two conditions with a side effect in between, not one. `pidl` is reused here
        // as `pidl_next` to avoid shadowing this function's own `pidl` parameter across the
        // whole rest of the function; the C++ shadow is scoped to this `if`-block only.
        let mut got_item = false;
        if hr >= 0 {
            hr = ((*(*EnumIDList).lpVtbl).Next)(
                EnumIDList,
                1,
                core::ptr::addr_of_mut!(pidl_next),
                core::ptr::addr_of_mut!(itemcount),
            );
            got_item = hr == S_OK;
        }
        if got_item {
            CoTaskMemFree(pidl_next.cast());
            iconname = CPP_COM_WSTR_279CE0FC8724.as_ptr(); // L"Full"
        } else {
            iconname = CPP_COM_WSTR_63638B1ECF1C.as_ptr(); // L"Empty"
        }
    }

    // CRegFolder.cpp:190-197 `/* Prepare registry path ... */`
    let mut KeyName: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    hr = FormatGUIDKey(
        KeyName.as_mut_ptr(),
        KeyName.len() as SIZE_T,
        CPP_COM_WSTR_3D93EF175FEC.as_ptr(), // L"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\CLSID\%s"
        riid,
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    // CRegFolder.cpp:200 `/* Load icon for the current user */`
    let mut ret: BOOL = HCU_GetIconW(
        KeyName.as_ptr(),
        wTemp.as_mut_ptr(),
        iconname,
        wTemp.len() as DWORD,
        core::ptr::addr_of_mut!(icon_idx),
    );
    if ret == 0 {
        // CRegFolder.cpp:203-208 `/* Failed, load default system-wide icon */`
        hr = FormatGUIDKey(
            KeyName.as_mut_ptr(),
            KeyName.len() as SIZE_T,
            CPP_COM_WSTR_804899D06140.as_ptr(), // L"CLSID\%s"
            riid,
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
        ret = HCR_GetIconW(
            KeyName.as_ptr(),
            wTemp.as_mut_ptr(),
            iconname,
            wTemp.len() as DWORD,
            core::ptr::addr_of_mut!(icon_idx),
        );
    }

    if ret != 0 {
        // CRegFolder.cpp:211-215 `/* Success, set loaded icon */`
        ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, wTemp.as_ptr(), icon_idx);
    } else {
        // CRegFolder.cpp:216-221 `/* Everything has failed, set blank paper icon */`
        ((*(*initIcon).lpVtbl).SetNormalIcon)(
            initIcon,
            core::ptr::addr_of!(swShell32Name).cast(),
            (IDI_SHELL_DOCUMENT as i32) - 1,
        );
    }

    // CRegFolder.cpp:223
    ((*(*initIcon).lpVtbl).QueryInterface)(initIcon, iid, ppvOut)
}
