// Deterministic UTF-16 literals from the selected C++ helper closure.
//
// The island/bindgen substrate (via `prelude`) is the ONLY substrate source for this TU; see
// prelude.rs for why the hand-written `types`/`win32`/`shellcom` globs this file used to also
// carry are dropped rather than joined (they made every shared name ambiguous, E0659, at 70+
// sites). A handful of names the island does not carry come from named imports instead:
//   * `SHCOLSTATE_TYPE_STR` lives in `crate::shell32::sdk_consts` (the rest of what that module
//     used to carry duplicated the island and was pruned — see its header). `SHCOLSTATE_ONBYDEFAULT`
//     is missing from that module too, so it is defined locally below, matching every other
//     flipped folder module in this crate (drives_folder.rs, cfs_folder.rs, file_sys_enum.rs,
//     printer_folder.rs, control_panel_folder.rs, printers_enum.rs, recycle_bin.rs all do the
//     same — there is nowhere else to import it from).
//   * `IDefaultExtractIconInit`/`IID_IDefaultExtractIconInit` and `QCMINFO` are COM/menu
//     substrate the sibling `--cpp-com` assembly of this same TU (CNetFolderEnum.rs,
//     `crate::net_folder_enum`) already carries in the real lpVtbl shape the island's own
//     opaque-class rendering of these C++ interfaces cannot give real method calls; imported by
//     name rather than re-derived a third time.
//   * `CRegKeyHandleArray` has exactly one definition, `crate::shfldr` (see that module's header
//     for why a second re-derivation is a latent HKEY leak — two earlier ones both dropped the
//     destructor).
use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::shellord::SHRunControlPanel;
use crate::wine::shlmenu::Shell_MergeMenus;
use crate::net_folder_enum::{
    CNetFolderEnum_CreateInstance, IDefaultExtractIconInit, IID_IDefaultExtractIconInit, QCMINFO,
};
use crate::shell32::sdk_consts::SHCOLSTATE_TYPE_STR;
use crate::shfldr::CRegKeyHandleArray;

pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_FEA2588FF02D: [WCHAR; 9] = [0x006E, 0x0063, 0x0070, 0x0061, 0x002E, 0x0063, 0x0070, 0x006C, 0x0000];

/// shlobj.h `SHCOLSTATE_ONBYDEFAULT` — not carried by `sdk_consts` (see its header); every other
/// flipped folder module in this crate defines its own copy for the same reason.
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

// ---- file-scope preamble (translated) ----
pub const COLUMN_NAME: u32 = 0;
pub const COLUMN_CATEGORY: u32 = 1;
pub const COLUMN_WORKGROUP: u32 = 2;
pub const COLUMN_NETLOCATION: u32 = 3;

pub const NETWORKPLACESSHELLVIEWCOLUMNS: u32 = 4;

pub use crate::island::shvheader;

pub static NetworkPlacesSFHeader: [shvheader; 4] = [
    shvheader {
        colnameid: (IDS_SHV_COLUMN_NAME as u16),
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as u16,
        fmt: (LVCFMT_RIGHT as i32) as u16,
        cxChar: 15,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_CATEGORY as u16),
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as u16,
        fmt: (LVCFMT_RIGHT as i32) as u16,
        cxChar: 10,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_WORKGROUP as u16),
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as u16,
        fmt: (LVCFMT_RIGHT as i32) as u16,
        cxChar: 15,
    },
    shvheader {
        colnameid: (IDS_SHV_COLUMN_NETLOCATION as u16),
        colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as u16,
        fmt: (LVCFMT_RIGHT as i32) as u16,
        cxChar: 15,
    },
];
#[no_mangle]
pub unsafe extern "C" fn ILCreateFromNetworkPlaceW(lpNetworkPlace: LPCWSTR) -> LPITEMIDLIST {
    let cbData: i32 = (core::mem::size_of::<WORD>() as i32) + (core::mem::size_of::<WCHAR>() as i32) * ((wcslen(lpNetworkPlace) + 1) as i32);
    // C: `SHAlloc(cbData + sizeof(WORD))` — the translation dropped the `sizeof`, leaving a
    // bare type name (`WORD`) where a byte count belongs; ground truth restored here. Cast at
    // the SHAlloc call site to SIZE_T, matching the established idiom for this exact
    // "int cbData; ... SHAlloc(cbData + sizeof(WORD))" shape (see shell32/brfolder.rs's
    // `ILCloneToDepth`, which SHAllocs the same `cb + sizeof(WORD)` pattern).
    let pidl: LPITEMIDLIST = SHAlloc((cbData as usize + core::mem::size_of::<WORD>()) as SIZE_T) as LPITEMIDLIST;
    if pidl.is_null() {
        return core::ptr::null_mut();
    }

    (*pidl).mkid.cb = cbData as WORD;
    wcscpy(
        (*pidl).mkid.abID.as_mut_ptr().offset(0) as *mut WCHAR,
        lpNetworkPlace
    );
    *((pidl as *mut i8).offset(cbData as isize) as *mut WORD) = 0;

    pidl
}

#[no_mangle]
pub unsafe extern "system" fn CNetFolder_ILCreateFromNetworkPlaceW_RustBridge(
    lpNetworkPlace: LPCWSTR,
) -> LPITEMIDLIST {
    ILCreateFromNetworkPlaceW(lpNetworkPlace)
}
#[no_mangle]
pub unsafe extern "C" fn CNetFolderExtractIcon_CreateInstance(
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    let mut initIcon: *mut IDefaultExtractIconInit = core::ptr::null_mut();
    let hr = SHCreateDefaultExtractIcon(core::ptr::addr_of!(IID_IDefaultExtractIconInit).cast(), core::ptr::addr_of_mut!(initIcon).cast());
    // `FAILED_UNEXPECTEDLY!` expands to a real `bool` (prelude.rs), not a C-style int — compare
    // directly rather than against 0.
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    // A vtable slot is a field holding a function pointer, not a method: call through
    // `lpVtbl` (see cfs_folder_tu.rs's `ILCloneToDepth`-adjacent `SetNormalIcon` call sites for
    // the same shape). `swShell32Name` is the island's own `WCHAR[MAX_PATH]` export.
    ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, core::ptr::addr_of!(swShell32Name).cast(), -(IDI_SHELL_NETWORK_FOLDER as i32));

    ((*(*initIcon).lpVtbl).QueryInterface)(initIcon, riid, ppvOut)
}

#[no_mangle]
pub unsafe extern "system" fn CNetFolderExtractIcon_RustBridge(
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CNetFolderExtractIcon_CreateInstance(pidl, riid, ppvOut)
}
// Real symbols this TU calls but does not itself define. `SHELL32_DefaultContextMenuCallBack`
// and `AddClassKeyToArray` are shell32-internal helpers that only exist as their `_RustBridge`
// wrapper under this calling convention — same pattern cfs_folder.rs uses for both (its own
// `extern "system"` block, `#[link_name = "..._RustBridge"]`). `Shell_MergeMenus` and
// `SHRunControlPanel` are `#[no_mangle]` exports (wine/shlmenu.rs, wine/shellord.rs) reached the
// same way drives_folder.rs/desktop_folder.rs already reach them: a bare local declaration, no
// link_name needed. `AddClassKeyToArray`'s `array` parameter is deliberately
// `crate::types::HKEY`, NOT this file's island-sourced `HKEY` — it has to match
// `CRegKeyHandleArray.hKeys`'s element type (`crate::shfldr::CRegKeyHandleArray`, which is
// itself fixed to `crate::types::HKEY`; see that module's header), and the two are different
// Rust types of the same width/ABI.
extern "system" {
    #[link_name = "SHELL32_DefaultContextMenuCallBack_RustBridge"]
    fn SHELL32_DefaultContextMenuCallBack(psf: *mut IShellFolder, pdo: *mut IDataObject, msg: UINT) -> HRESULT;
    #[link_name = "AddClassKeyToArray_RustBridge"]
    fn AddClassKeyToArray(szClass: LPCWSTR, array: *mut crate::types::HKEY, cKeys: *mut UINT) -> LONG;
    #[link_name = "SHELL32_BindToSF_RustBridge"]
    fn SHELL32_BindToSF(pidlRoot: LPCITEMIDLIST, ppfti: *mut PERSIST_FOLDER_TARGET_INFO, pidl: LPCITEMIDLIST, clsid: *const CLSID, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
}

/// shellutils.h:218 `#define SHELL_ExecuteControlPanelCPL(hwnd, cpl) SHRunControlPanel(cpl, hwnd)`
/// — a header inline, so every TU that calls it carries its own copy (see desktop_folder.rs,
/// which has this exact one-liner for the same reason).
#[inline]
unsafe fn SHELL_ExecuteControlPanelCPL(hwnd: HWND, cpl: LPCWSTR) -> BOOL {
    SHRunControlPanel(cpl, hwnd)
}

// `CDefFolderMenu_Create2`'s callback parameter (`LPFNDFMCALLBACK`, shobjidl.h via the island)
// types its `IDataObject*` argument against `com_sys::IDataObject` — the island's own C++-class
// rendering of that header, not the `lpVtbl`-shaped `shellcom::IDataObject` this file's
// `NetFolderMenuCallback`/`CNetFolderBackgroundMenuCB` use like every other call site here
// (`SHELL32_DefaultContextMenuCallBack`, `IDataObject_Constructor`). Same struct at the ABI
// level, two different nominal Rust types — bridge with a thin same-ABI adapter at the
// `CDefFolderMenu_Create2` boundary rather than repoint the real callbacks at the wrong
// substrate.
unsafe extern "system" fn CNetFolderBackgroundMenuCB_DFMCallback(
    psf: *mut IShellFolder,
    hwnd: HWND,
    pdtobj: *mut com_sys::IDataObject,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> HRESULT {
    CNetFolderBackgroundMenuCB(psf, hwnd, pdtobj.cast(), uMsg, wParam, lParam)
}
unsafe extern "system" fn NetFolderMenuCallback_DFMCallback(
    psf: *mut IShellFolder,
    hwnd: HWND,
    pdtobj: *mut com_sys::IDataObject,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> HRESULT {
    NetFolderMenuCallback(psf, hwnd, pdtobj.cast(), uMsg, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn NetFolderMenuCallback(
    psf: *mut IShellFolder,
    hwnd: HWND,
    pdtobj: *mut IDataObject,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> HRESULT {
    return SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg);
}

// CNetFolder.cpp:232. Hand-written: the paid translation was refused by the call-seal audit
// for reusing one token twice, and it was RIGHT to refuse -- the model had emitted DestroyMenu
// where the tail call to SHELL32_DefaultContextMenuCallBack belongs, which would have dropped
// the default handling of every message this callback does not handle itself and passed a
// shell-folder pointer to a menu-destroying API. The forensic copy is kept under
// .responses/.../__rejected__/.
unsafe extern "system" fn CNetFolderBackgroundMenuCB(
    psf: *mut IShellFolder,
    hwnd: HWND,
    pdtobj: *mut IDataObject,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> HRESULT {
    // C++ `enum { IDC_PROPERTIES };` -- an unnamed enum whose single member is 0. Prefixed
    // because the NAME is not crate-unique: CRecycleBin.cpp's `enum { IDC_EMPTYRECYCLEBIN = 1,
    // IDC_PROPERTIES }` makes it 2 in recycle_bin.rs. Both are faithful to their own TU, and
    // reconciling the values would corrupt one of the two menus.
    const NETFOLDER_IDC_PROPERTIES: UINT = 0;

    if uMsg == DFM_INVOKECOMMAND && wParam == NETFOLDER_IDC_PROPERTIES as WPARAM {
        return if SHELL_ExecuteControlPanelCPL(hwnd, CPP_COM_WSTR_FEA2588FF02D.as_ptr()) != 0 {
            S_OK
        } else {
            E_FAIL
        };
    } else if uMsg == DFM_MERGECONTEXTMENU {
        let pqcminfo = lParam as *mut QCMINFO;
        let hpopup = CreatePopupMenu();
        _InsertMenuItemW(
            hpopup,
            0,
            TRUE,
            NETFOLDER_IDC_PROPERTIES,
            MFT_STRING,
            MAKEINTRESOURCEW(IDS_PROPERTIES),
            MFS_ENABLED,
        );
        (*pqcminfo).idCmdFirst = Shell_MergeMenus(
            (*pqcminfo).hmenu,
            hpopup,
            (*pqcminfo).indexMenu,
            (*pqcminfo).idCmdFirst,
            (*pqcminfo).idCmdLast,
            MM_ADDSEPARATOR,
        );
        DestroyMenu(hpopup);
        return S_OK;
    }
    SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg)
}

#[no_mangle]
pub unsafe extern "system" fn CNetFolderEnum_CreateInstance_RustBridge(
    hwndOwner: HWND,
    dwFlags: DWORD,
    ppEnumIDList: *mut *mut IEnumIDList,
) -> HRESULT {
    // `ShellObjectCreatorInit<T>` is a C++ template (shellutils.h) — a header inline that no
    // translated TU ever carries a body for (see shell32/mod.rs's note on this same class of
    // name). Every other enumerator in this crate resolves the equivalent call to its own
    // concrete non-aggregating factory instead; `CNetFolderEnum`'s is
    // `crate::net_folder_enum::CNetFolderEnum_CreateInstance`, which is exactly that function.
    CNetFolderEnum_CreateInstance(
        hwndOwner,
        dwFlags,
        core::ptr::addr_of!(IID_IEnumIDList).cast(),
        (ppEnumIDList).cast(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn CNetFolder_CreateBackgroundMenu_RustBridge(
    pidlRoot: LPCITEMIDLIST,
    hwndOwner: HWND,
    psf: *mut IShellFolder,
    ppContextMenu: *mut *mut IContextMenu,
) -> HRESULT {
    // `CDefFolderMenu_Create2`'s first parameter is `PCIDLIST_ABSOLUTE` in the real header
    // (shlobj.h) even though this bridge's own signature — matching the C++ ground truth
    // exactly — takes `LPCITEMIDLIST`; both are `*const` PIDL aliases of the same ABI shape, so
    // cast at the call site rather than change the signature.
    return CDefFolderMenu_Create2(
        pidlRoot.cast(),
        hwndOwner,
        0,
        core::ptr::null_mut(),
        psf,
        Some(CNetFolderBackgroundMenuCB_DFMCallback),
        0,
        core::ptr::null_mut(),
        ppContextMenu,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CNetFolder_CreateItemMenu_RustBridge(
    pidlRoot: LPCITEMIDLIST,
    hwndOwner: HWND,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    psf: *mut IShellFolder,
    ppContextMenu: *mut *mut IContextMenu,
) -> HRESULT {
    // `AddClassKeyToArray(L"Folder", keys, keys)` in C++ passes `keys` through two DIFFERENT
    // operator-overload conversions (`operator HKEY*()`, `operator UINT*()`); Rust has to spell
    // the two conversions out (see cfs_folder.rs / desktop_folder.rs, which hit this exact
    // shape for the exact same call).
    let mut keys = CRegKeyHandleArray::default();
    AddClassKeyToArray(
        CPP_COM_WSTR_EB4CE2024A43.as_ptr(),
        keys.hKeys.as_mut_ptr(),
        &mut keys.cKeys,
    );
    // Same PIDL-alias cast as CreateBackgroundMenu above, plus the `keys`/`keys` pair's other
    // two operator conversions this call needs (`operator UINT()`, `operator HKEY*() const`),
    // and a cast of the resulting `*const types::HKEY` onto the island's `*const HKEY` that
    // `CDefFolderMenu_Create2` (bindgen) declares — same ABI, different nominal pointee type.
    CDefFolderMenu_Create2(
        pidlRoot.cast(),
        hwndOwner,
        cidl,
        apidl,
        psf,
        Some(NetFolderMenuCallback_DFMCallback),
        keys.len() as UINT,
        keys.as_ptr() as *const HKEY,
        ppContextMenu,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CNetFolder_BindToSF_RustBridge(
    pidlRoot: LPCITEMIDLIST,
    ppfti: *mut PERSIST_FOLDER_TARGET_INFO,
    pidl: LPCITEMIDLIST,
    pclsid: *const CLSID,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    SHELL32_BindToSF(pidlRoot, ppfti, pidl, pclsid, riid, ppvOut)
}

#[no_mangle]
pub unsafe extern "system" fn CNetFolder_IDataObjectConstructor_RustBridge(
    hwndOwner: HWND,
    pidlRoot: PCIDLIST_ABSOLUTE,
    apidl: PCUIDLIST_RELATIVE_ARRAY,
    cidl: UINT,
    bExtended: BOOL,
    ppDataObject: *mut *mut IDataObject,
) -> HRESULT {
    // `IDataObject_Constructor` is the island's own (bindgen) declaration: its second parameter
    // is `LPCITEMIDLIST` (this fn's is `PCIDLIST_ABSOLUTE`, matching the C++ ground truth) and
    // its third is `PCUITEMID_CHILD_ARRAY` (this fn's is `PCUIDLIST_RELATIVE_ARRAY`, likewise
    // ground-truth) — two more PIDL-alias splits, same ABI shape, cast at the call site rather
    // than change the signature. Its `IDataObject` is `com_sys::IDataObject` (see the
    // `NetFolderMenuCallback_DFMCallback` note above for why that split exists at all).
    IDataObject_Constructor(hwndOwner, pidlRoot.cast(), apidl.cast(), cidl, bExtended, ppDataObject.cast())
}


// ============================================================================
// ORIGINAL C SOURCE (this slice only)
// ============================================================================
//
// /*
//  *    Network Places (Neighbourhood) folder
//  *
//  *    Copyright 1997                Marcus Meissner
//  *    Copyright 1998, 1999, 2002    Juergen Schmied
//  *    Copyright 2003              Mike McCormack for Codeweavers
//  *    Copyright 2009              Andrew Hill
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
// WINE_DEFAULT_DEBUG_CHANNEL (shell);
//
// #define HACKY_UNC_PATHS
//
// LPITEMIDLIST ILCreateFromNetworkPlaceW(LPCWSTR lpNetworkPlace)
// {
//     int cbData = sizeof(WORD) + sizeof(WCHAR) * (wcslen(lpNetworkPlace)+1);
//     LPITEMIDLIST pidl = (LPITEMIDLIST)SHAlloc(cbData + sizeof(WORD));
//     if (!pidl)
//         return NULL;
//
//     pidl->mkid.cb = cbData;
//     wcscpy((WCHAR*)&pidl->mkid.abID[0], lpNetworkPlace);
//     *(WORD*)((char*)pidl + cbData) = 0;
//
//     return pidl;
// }
// extern "C" LPITEMIDLIST WINAPI CNetFolder_ILCreateFromNetworkPlaceW_RustBridge(
//     LPCWSTR lpNetworkPlace)
// {
//     return ILCreateFromNetworkPlaceW(lpNetworkPlace);
// }
// HRESULT CNetFolderExtractIcon_CreateInstance(LPCITEMIDLIST pidl, REFIID riid, LPVOID * ppvOut)
// {
//     CComPtr<IDefaultExtractIconInit> initIcon;
//     HRESULT hr = SHCreateDefaultExtractIcon(IID_PPV_ARG(IDefaultExtractIconInit, &initIcon));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     initIcon->SetNormalIcon(swShell32Name, -IDI_SHELL_NETWORK_FOLDER);
//
//     return initIcon->QueryInterface(riid, ppvOut);
// }
// extern "C" HRESULT WINAPI CNetFolderExtractIcon_RustBridge(
//     LPCITEMIDLIST pidl, REFIID riid, LPVOID *ppvOut)
// {
//     return CNetFolderExtractIcon_CreateInstance(pidl, riid, ppvOut);
// }
// HRESULT __stdcall NetFolderMenuCallback(IShellFolder *psf,
//                                        HWND         hwnd,
//                                        IDataObject  *pdtobj,
//                                        UINT         uMsg,
//                                        WPARAM       wParam,
//                                        LPARAM       lParam)
// {
//     return SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg);
// }
// extern "C" HRESULT WINAPI CNetFolderEnum_CreateInstance_RustBridge(
//     HWND hwndOwner, DWORD dwFlags, IEnumIDList **ppEnumIDList)
// {
//     return ShellObjectCreatorInit<CNetFolderEnum>(
//         hwndOwner, dwFlags, IID_PPV_ARG(IEnumIDList, ppEnumIDList));
// }
// static HRESULT __stdcall CNetFolderBackgroundMenuCB(IShellFolder *psf, HWND hwnd, IDataObject *pdtobj,
//                                                    UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     enum { IDC_PROPERTIES };
//     if (uMsg == DFM_INVOKECOMMAND && wParam == IDC_PROPERTIES)
//     {
//         return SHELL_ExecuteControlPanelCPL(hwnd, L"ncpa.cpl") ? S_OK : E_FAIL;
//     }
//     else if (uMsg == DFM_MERGECONTEXTMENU) // TODO: DFM_MERGECONTEXTMENU_BOTTOM
//     {
//         QCMINFO *pqcminfo = (QCMINFO*)lParam;
//         HMENU hpopup = CreatePopupMenu();
//         _InsertMenuItemW(hpopup, 0, TRUE, IDC_PROPERTIES, MFT_STRING, MAKEINTRESOURCEW(IDS_PROPERTIES), MFS_ENABLED);
//         pqcminfo->idCmdFirst = Shell_MergeMenus(pqcminfo->hmenu, hpopup, pqcminfo->indexMenu, pqcminfo->idCmdFirst, pqcminfo->idCmdLast, MM_ADDSEPARATOR);
//         DestroyMenu(hpopup);
//         return S_OK;
//     }
//     return SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg);
// }
// extern "C" HRESULT WINAPI CNetFolder_CreateBackgroundMenu_RustBridge(
//     LPCITEMIDLIST pidlRoot, HWND hwndOwner, IShellFolder *psf,
//     IContextMenu **ppContextMenu)
// {
//     return CDefFolderMenu_Create2(
//         pidlRoot, hwndOwner, 0, NULL, psf, CNetFolderBackgroundMenuCB,
//         0, NULL, ppContextMenu);
// }
// extern "C" HRESULT WINAPI CNetFolder_CreateItemMenu_RustBridge(
//     LPCITEMIDLIST pidlRoot, HWND hwndOwner, UINT cidl,
//     PCUITEMID_CHILD_ARRAY apidl, IShellFolder *psf,
//     IContextMenu **ppContextMenu)
// {
//     CRegKeyHandleArray keys;
//     AddClassKeyToArray(L"Folder", keys, keys);
//     return CDefFolderMenu_Create2(
//         pidlRoot, hwndOwner, cidl, apidl, psf, NetFolderMenuCallback,
//         keys, keys, ppContextMenu);
// }
// extern "C" HRESULT WINAPI CNetFolder_BindToSF_RustBridge(
//     LPCITEMIDLIST pidlRoot, PERSIST_FOLDER_TARGET_INFO *ppfti,
//     LPCITEMIDLIST pidl, const CLSID *pclsid, REFIID riid, LPVOID *ppvOut)
// {
//     return SHELL32_BindToSF(pidlRoot, ppfti, pidl, pclsid, riid, ppvOut);
// }
// extern "C" HRESULT WINAPI CNetFolder_IDataObjectConstructor_RustBridge(
//     HWND hwndOwner, PCIDLIST_ABSOLUTE pidlRoot,
//     PCUIDLIST_RELATIVE_ARRAY apidl, UINT cidl, BOOL bExtended,
//     IDataObject **ppDataObject)
// {
//     return IDataObject_Constructor(
//         hwndOwner, pidlRoot, apidl, cidl, bExtended, ppDataObject);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
