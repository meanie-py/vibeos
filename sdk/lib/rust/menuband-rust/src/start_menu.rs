use crate::prelude::*;
use crate::shell_menu_callback::CShellMenuCallback;
use crate::shell_menu_callback::CShellMenuCallback_CreateInstance;
use crate::shell_menu_callback::CShellMenuCallback_Initialize_body;
use crate::shell_menu_callback::CShellMenuCallback__SetProgramsFolder_body;
use crate::menu_desk_bar::RSHELL_CMenuDeskBar_CreateInstance;
use crate::menu_site::RSHELL_CMenuSite_CreateInstance;
use core::ffi::c_void;
use crate::merged_folder::IAugmentedShellFolder;
use crate::shell_menu_callback::CppComPtrGuard;
use crate::shell_menu_callback::{IBandSite, IDeskBar, IShellMenu};

// ---- file-scope preamble (translated) ----
pub const IDM_RUN: u32 = 401;
pub const IDM_LOGOFF: u32 = 402;
pub const IDM_UNDOCKCOMPUTER: u32 = 410;
pub const IDM_TASKBARANDSTARTMENU: u32 = 413;
pub const IDM_LASTSTARTMENU_SEPARATOR: u32 = 450;
pub const IDM_DOCUMENTS: u32 = 501;
pub const IDM_HELPANDSUPPORT: u32 = 503;
pub const IDM_PROGRAMS: u32 = 504;
pub const IDM_CONTROLPANEL: u32 = 505;
pub const IDM_SHUTDOWN: u32 = 506;
pub const IDM_FAVORITES: u32 = 507;
pub const IDM_SETTINGS: u32 = 508;
pub const IDM_PRINTERSANDFAXES: u32 = 510;
pub const IDM_SEARCH: u32 = 520;
pub const IDM_SYNCHRONIZE: u32 = 553;
pub const IDM_NETWORKCONNECTIONS: u32 = 557;
pub const IDM_DISCONNECT: u32 = 5000;
pub const IDM_SECURITY: u32 = 5001;
#[no_mangle]
pub unsafe extern "C" fn BindToDesktop(pidl: LPCITEMIDLIST, ppsfResult: *mut *mut IShellFolder) -> HRESULT {
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
    let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
    let mut hr: HRESULT;

    *ppsfResult = core::ptr::null_mut();

    hr = SHGetDesktopFolder(core::ptr::addr_of_mut!(psfDesktop));
    if hr < 0 {
        if !psfDesktop.is_null() {
            ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
            psfDesktop = core::ptr::null_mut();
        }
        return hr;
    }

    hr = ((*(*psfDesktop).lpVtbl).BindToObject)(
        psfDesktop,
        pidl,
        core::ptr::null_mut(),
        &IID_IShellFolder,
        ppsfResult as *mut *mut core::ffi::c_void,
    );

    if !psfDesktop.is_null() {
        ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
        psfDesktop = core::ptr::null_mut();
    }

    hr
}
unsafe extern "C" fn GetMergedFolder(
    folder1: i32,
    folder2: i32,
    ppsfStartMenu: *mut *mut IShellFolder,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pidlUserStartMenu: LPITEMIDLIST = core::ptr::null_mut();
    let mut pidlCommonStartMenu: LPITEMIDLIST = core::ptr::null_mut();
    let mut psfUserStartMenu: *mut IShellFolder = core::ptr::null_mut();
    let _psfUserStartMenu_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfUserStartMenu));
    let mut psfCommonStartMenu: *mut IShellFolder = core::ptr::null_mut();
    let _psfCommonStartMenu_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfCommonStartMenu));
    // IAugmentedShellFolder: same island-opaque-vs-hand-written nominal split as
    // IShellMenu/IBandSite/IDeskBar in RSHELL_CStartMenu_CreateInstance -- reusing
    // merged_folder.rs's already-`pub` hand-written definition (the one CMergedFolder
    // itself implements) instead of inventing a second copy here.
    // A FUNCTION-SCOPED `use`, not a qualified path, for two reasons: it shadows the prelude
    // glob's island-opaque IAugmentedShellFolder, AND it keeps the declaration spelled with the
    // bare interface name. The scaffold's CComPtr audit matches
    // `let mut <name>: *mut <Interface> = ...` using the interface name from the C++
    // `CComPtr<IAugmentedShellFolder> pasf;`, so a qualified path fails it with
    // "expected one null raw-pointer declaration in worker, found 0".
    // The --cpp-com slice of this TU emits the CppComPtrGuard struct; the
    // --free-functions slice gets the scaffold's guard USES and no definition.
    // Same .cpp, so borrow the class slice's.
    let mut pasf: *mut IAugmentedShellFolder = core::ptr::null_mut();
    let _pasf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pasf));

    *ppsfStartMenu = core::ptr::null_mut();

    hr = SHGetSpecialFolderLocation(core::ptr::null_mut(), folder1, &mut pidlUserStartMenu);
    if hr < 0 {
        hr = SHGetSpecialFolderLocation(core::ptr::null_mut(), folder2, &mut pidlCommonStartMenu);
        if ((hr) < 0) {
            return hr;
        }

        hr = BindToDesktop(pidlCommonStartMenu, ppsfStartMenu);
        ILFree(pidlCommonStartMenu);
        return hr;
    }

    // shellmenu.h:69 `#define MERGE_FOLDERS 1`, so CStartMenu.cpp compiles the `#if`
    // arm here: a SECOND SHGetSpecialFolderLocation whose failure falls back to the
    // USER folder alone. The translation took the `#else` arm instead, which is a bare
    // `else` on the FIRST call -- and that is not merely a different spelling of the
    // same thing: it made every success path return the USER start menu unmerged and
    // left the whole merge tail of the function dead. rustc's "unreachable statement"
    // on the line below this block was the tell.
    hr = SHGetSpecialFolderLocation(core::ptr::null_mut(), folder2, &mut pidlCommonStartMenu);
    if ((hr) < 0) {
        hr = BindToDesktop(pidlUserStartMenu, ppsfStartMenu);
        ILFree(pidlUserStartMenu);
        return hr;
    }

    hr = BindToDesktop(pidlUserStartMenu, &mut psfUserStartMenu);
    if ((hr) < 0) {
        return hr;
    }

    hr = BindToDesktop(pidlCommonStartMenu, &mut psfCommonStartMenu);
    if ((hr) < 0) {
        if !psfUserStartMenu.is_null() {
            ((*(*psfUserStartMenu).lpVtbl).Release)(psfUserStartMenu);
            psfUserStartMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = RSHELL_CMergedFolder_CreateInstance(
        &IID_IAugmentedShellFolder,
        &mut pasf as *mut *mut IAugmentedShellFolder as *mut *mut core::ffi::c_void,
    );
    if ((hr) < 0) {
        *ppsfStartMenu = psfUserStartMenu;
        psfUserStartMenu = core::ptr::null_mut();

        if !psfCommonStartMenu.is_null() {
            ((*(*psfCommonStartMenu).lpVtbl).Release)(psfCommonStartMenu);
            psfCommonStartMenu = core::ptr::null_mut();
        }
        ILFree(pidlCommonStartMenu);
        ILFree(pidlUserStartMenu);
        return hr;
    }

    hr = ((*(*pasf).lpVtbl).AddNameSpace)(
        pasf,
        core::ptr::null_mut(),
        psfUserStartMenu.cast(),
        pidlUserStartMenu,
        0xFF00,
    );
    if ((hr) < 0) {
        if !pasf.is_null() {
            ((*(*pasf).lpVtbl).Release)(pasf);
            pasf = core::ptr::null_mut();
        }
        if !psfUserStartMenu.is_null() {
            ((*(*psfUserStartMenu).lpVtbl).Release)(psfUserStartMenu);
            psfUserStartMenu = core::ptr::null_mut();
        }
        if !psfCommonStartMenu.is_null() {
            ((*(*psfCommonStartMenu).lpVtbl).Release)(psfCommonStartMenu);
            psfCommonStartMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = ((*(*pasf).lpVtbl).AddNameSpace)(
        pasf,
        core::ptr::null_mut(),
        psfCommonStartMenu.cast(),
        pidlCommonStartMenu,
        0,
    );
    if ((hr) < 0) {
        if !pasf.is_null() {
            ((*(*pasf).lpVtbl).Release)(pasf);
            pasf = core::ptr::null_mut();
        }
        if !psfUserStartMenu.is_null() {
            ((*(*psfUserStartMenu).lpVtbl).Release)(psfUserStartMenu);
            psfUserStartMenu = core::ptr::null_mut();
        }
        if !psfCommonStartMenu.is_null() {
            ((*(*psfCommonStartMenu).lpVtbl).Release)(psfCommonStartMenu);
            psfCommonStartMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = ((*(*pasf).lpVtbl).QueryInterface)(
        pasf,
        &IID_IShellFolder,
        ppsfStartMenu as *mut *mut core::ffi::c_void,
    );

    ((*(*pasf).lpVtbl).Release)(pasf);
    pasf = core::ptr::null_mut();

    if !psfUserStartMenu.is_null() {
        ((*(*psfUserStartMenu).lpVtbl).Release)(psfUserStartMenu);
        psfUserStartMenu = core::ptr::null_mut();
    }
    if !psfCommonStartMenu.is_null() {
        ((*(*psfCommonStartMenu).lpVtbl).Release)(psfCommonStartMenu);
        psfCommonStartMenu = core::ptr::null_mut();
    }

    ILFree(pidlCommonStartMenu);
    ILFree(pidlUserStartMenu);

    hr
}
unsafe extern "C" fn GetStartMenuFolder(ppsfStartMenu: *mut *mut IShellFolder) -> HRESULT {
    GetMergedFolder(CSIDL_STARTMENU as i32, CSIDL_COMMON_STARTMENU as i32, ppsfStartMenu)
}
unsafe extern "C" fn GetProgramsFolder(ppsfStartMenu: *mut *mut IShellFolder) -> HRESULT {
    unsafe {
        GetMergedFolder(
            CSIDL_PROGRAMS as i32,
            CSIDL_COMMON_PROGRAMS as i32,
            ppsfStartMenu,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn RSHELL_CStartMenu_CreateInstance(
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // IShellMenu/IBandSite/IDeskBar: this file (start_menu.rs, the free-functions half of
    // the TU) has no local "Deterministic from shobjidl.h" scaffold the way the class half
    // (shell_menu_callback.rs) does, so the bare names would otherwise resolve through
    // `crate::prelude::*` to the island's OPAQUE bindgen structs (`vtable_` field, no
    // `lpVtbl`) -- a nominal split, AGENTS.md rule 1. Reusing shell_menu_callback.rs's
    // already-`pub` hand-written definitions (same layout: one `lpVtbl` pointer) instead of
    // inventing a second copy here.
    // Function-scoped `use`, not qualified paths: it shadows the prelude glob's
    // island-OPAQUE versions of these interfaces (bindgen renders a C++ class with no
    // callable slots) while keeping the declarations spelled with the BARE interface
    // name, which is what the scaffold's CComPtr audit matches against the C++
    // `CComPtr<IShellMenu> pShellMenu;`. A qualified path fails it with
    // "expected one null raw-pointer declaration in worker, found 0".
    let mut pShellMenu: *mut IShellMenu = core::ptr::null_mut();
    let _pShellMenu_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pShellMenu));
    let mut pBandSite: *mut IBandSite = core::ptr::null_mut();
    let _pBandSite_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pBandSite));
    let mut pDeskBar: *mut IDeskBar = core::ptr::null_mut();
    let _pDeskBar_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDeskBar));
    let mut hr: HRESULT;
    let mut psf: *mut IShellFolder = core::ptr::null_mut();
    let mut pidlProgramsAbsolute: LPITEMIDLIST = core::ptr::null_mut();
    let mut pidlPrograms: LPITEMIDLIST = core::ptr::null_mut();
    let mut psfPrograms: *mut IShellFolder = core::ptr::null_mut();
    let _psfPrograms_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfPrograms));
    // C++: `CComObject<CShellMenuCallback> *pCallback;` -- this crate has no generic
    // `CComObject<T>` ATL wrapper (see enum_merged_folder.rs's CEnumMergedFolder, which hit
    // the same gap and used a per-class `_CreateInstance` free function instead of one).
    // CShellMenuCallback's own deterministic scaffold provides exactly that factory
    // (`CShellMenuCallback_CreateInstance`, shell_menu_callback.rs), so it is used here.
    let mut pCallback: *mut CShellMenuCallback = core::ptr::null_mut();

    hr = RSHELL_CMenuBand_CreateInstance(core::ptr::addr_of!(IID_IShellMenu), core::ptr::addr_of_mut!(pShellMenu).cast());
    if ((hr) < 0) {
        return hr;
    }

    hr = RSHELL_CMenuSite_CreateInstance(core::ptr::addr_of!(IID_IBandSite), core::ptr::addr_of_mut!(pBandSite).cast());
    if ((hr) < 0) {
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = RSHELL_CMenuDeskBar_CreateInstance(core::ptr::addr_of!(IID_IDeskBar), core::ptr::addr_of_mut!(pDeskBar).cast());
    if ((hr) < 0) {
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    // C++: `hr = CComObject<CShellMenuCallback>::CreateInstance(&pCallback);` -- ATL's
    // CreateInstance hands back the object at REFCOUNT 0. This crate's factory instead
    // allocates+constructs and then does one QueryInterface(IID_IUnknown)+Release internally
    // (see CShellMenuCallback_CreateInstance), netting REFCOUNT 1 in a single call. FLAGGED
    // per the task's hard rule 6: the C++'s separate `CreateInstance` (0) + explicit
    // `pCallback->AddRef()` (1) are collapsed into this one call, which lands at the SAME
    // final refcount (1) but without a second, textually-visible AddRef -- please double
    // check this against the compiler/runtime rather than trusting the comment alone.
    let mut pCallbackUnk: *mut core::ffi::c_void = core::ptr::null_mut();
    hr = CShellMenuCallback_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(crate::shell_menu_callback::IID_IUnknown),
        core::ptr::addr_of_mut!(pCallbackUnk),
    );
    if ((hr) < 0) {
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }
    // IShellMenuCallback_iface is CShellMenuCallback's first field (#[repr(C)]), so the
    // QueryInterface'd pointer IS the struct's address -- same pattern as the
    // `(This as *mut u8).sub(core::mem::offset_of!(...))` bridges elsewhere in shell_menu_callback.rs,
    // specialized to the offset-0 case.
    pCallback = pCallbackUnk as *mut CShellMenuCallback;

    CShellMenuCallback_Initialize_body(pCallback, pShellMenu, pBandSite, pDeskBar);

    hr = ((*(*pShellMenu).lpVtbl).Initialize)(
        pShellMenu,
        core::ptr::addr_of_mut!((*pCallback).IShellMenuCallback_iface),
        (-1i32) as UINT,
        0,
        SMINIT_TOPLEVEL | SMINIT_VERTICAL,
    );
    if ((hr) < 0) {
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = GetStartMenuFolder(&mut psf);
    if ((hr) < 0) {
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    {
        hr = SHGetSpecialFolderLocation(core::ptr::null_mut(), CSIDL_PROGRAMS as INT, &mut pidlProgramsAbsolute);
        if ((hr) < 0) {
            hr = SHGetSpecialFolderLocation(core::ptr::null_mut(), CSIDL_COMMON_PROGRAMS as INT, &mut pidlProgramsAbsolute);
            if ((hr) < 0) {
                if !pDeskBar.is_null() {
                    ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
                    pDeskBar = core::ptr::null_mut();
                }
                if !pBandSite.is_null() {
                    ((*(*pBandSite).lpVtbl).Release)(pBandSite);
                    pBandSite = core::ptr::null_mut();
                }
                if !pShellMenu.is_null() {
                    ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
                    pShellMenu = core::ptr::null_mut();
                }
                return hr;
            }
        }

        let mut pcidlPrograms: LPCITEMIDLIST = core::ptr::null();
        let mut psfParent: *mut IShellFolder = core::ptr::null_mut();
        let _psfParent_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfParent));
        let mut str: STRRET = core::mem::zeroed();
        let mut szDisplayName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

        hr = SHBindToParent(pidlProgramsAbsolute.cast(), core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(psfParent).cast(), core::ptr::addr_of_mut!(pcidlPrograms).cast());
        if ((hr) < 0) {
            if !psfParent.is_null() {
                ((*(*psfParent).lpVtbl).Release)(psfParent);
                psfParent = core::ptr::null_mut();
            }
            if !pDeskBar.is_null() {
                ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
                pDeskBar = core::ptr::null_mut();
            }
            if !pBandSite.is_null() {
                ((*(*pBandSite).lpVtbl).Release)(pBandSite);
                pBandSite = core::ptr::null_mut();
            }
            if !pShellMenu.is_null() {
                ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
                pShellMenu = core::ptr::null_mut();
            }
            return hr;
        }

        hr = ((*(*psfParent).lpVtbl).GetDisplayNameOf)(
            psfParent,
            pcidlPrograms,
            SHGDN_FORPARSING | SHGDN_INFOLDER,
            core::ptr::addr_of_mut!(str).cast(),
        );
        if ((hr) < 0) {
            if !psfParent.is_null() {
                ((*(*psfParent).lpVtbl).Release)(psfParent);
                psfParent = core::ptr::null_mut();
            }
            if !pDeskBar.is_null() {
                ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
                pDeskBar = core::ptr::null_mut();
            }
            if !pBandSite.is_null() {
                ((*(*pBandSite).lpVtbl).Release)(pBandSite);
                pBandSite = core::ptr::null_mut();
            }
            if !pShellMenu.is_null() {
                ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
                pShellMenu = core::ptr::null_mut();
            }
            return hr;
        }

        StrRetToBufW(core::ptr::addr_of_mut!(str).cast(), pcidlPrograms, szDisplayName.as_mut_ptr(), szDisplayName.len() as UINT);
        ILFree(pidlProgramsAbsolute);

        hr = ((*(*psf).lpVtbl).ParseDisplayName)(
            psf,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            szDisplayName.as_mut_ptr(),
            core::ptr::null_mut(),
            &mut pidlPrograms,
            core::ptr::null_mut(),
        );
        if ((hr) < 0) {
            if !psfParent.is_null() {
                ((*(*psfParent).lpVtbl).Release)(psfParent);
                psfParent = core::ptr::null_mut();
            }
            if !pDeskBar.is_null() {
                ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
                pDeskBar = core::ptr::null_mut();
            }
            if !pBandSite.is_null() {
                ((*(*pBandSite).lpVtbl).Release)(pBandSite);
                pBandSite = core::ptr::null_mut();
            }
            if !pShellMenu.is_null() {
                ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
                pShellMenu = core::ptr::null_mut();
            }
            return hr;
        }

        if !psfParent.is_null() {
            ((*(*psfParent).lpVtbl).Release)(psfParent);
            psfParent = core::ptr::null_mut();
        }
    }

    hr = GetProgramsFolder(&mut psfPrograms);
    if ((hr) < 0) {
        if !psfPrograms.is_null() {
            ((*(*psfPrograms).lpVtbl).Release)(psfPrograms);
            psfPrograms = core::ptr::null_mut();
        }
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = CShellMenuCallback__SetProgramsFolder_body(pCallback, psfPrograms, pidlPrograms);
    if ((hr) < 0) {
        if !psfPrograms.is_null() {
            ((*(*psfPrograms).lpVtbl).Release)(psfPrograms);
            psfPrograms = core::ptr::null_mut();
        }
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = ((*(*pShellMenu).lpVtbl).SetShellFolder)(
        pShellMenu,
        psf,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SMSET_TOP,
    );
    if ((hr) < 0) {
        if !psfPrograms.is_null() {
            ((*(*psfPrograms).lpVtbl).Release)(psfPrograms);
            psfPrograms = core::ptr::null_mut();
        }
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = ((*(*pDeskBar).lpVtbl).SetClient)(pDeskBar, pBandSite.cast());
    if ((hr) < 0) {
        if !psfPrograms.is_null() {
            ((*(*psfPrograms).lpVtbl).Release)(psfPrograms);
            psfPrograms = core::ptr::null_mut();
        }
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    hr = ((*(*pBandSite).lpVtbl).AddBand)(pBandSite, pShellMenu.cast());
    if ((hr) < 0) {
        if !psfPrograms.is_null() {
            ((*(*psfPrograms).lpVtbl).Release)(psfPrograms);
            psfPrograms = core::ptr::null_mut();
        }
        if !pDeskBar.is_null() {
            ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
            pDeskBar = core::ptr::null_mut();
        }
        if !pBandSite.is_null() {
            ((*(*pBandSite).lpVtbl).Release)(pBandSite);
            pBandSite = core::ptr::null_mut();
        }
        if !pShellMenu.is_null() {
            ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
            pShellMenu = core::ptr::null_mut();
        }
        return hr;
    }

    let result: HRESULT = ((*(*pDeskBar).lpVtbl).QueryInterface)(pDeskBar, riid, ppv);

    if !psfPrograms.is_null() {
        ((*(*psfPrograms).lpVtbl).Release)(psfPrograms);
        psfPrograms = core::ptr::null_mut();
    }
    if !pDeskBar.is_null() {
        ((*(*pDeskBar).lpVtbl).Release)(pDeskBar);
        pDeskBar = core::ptr::null_mut();
    }
    if !pBandSite.is_null() {
        ((*(*pBandSite).lpVtbl).Release)(pBandSite);
        pBandSite = core::ptr::null_mut();
    }
    if !pShellMenu.is_null() {
        ((*(*pShellMenu).lpVtbl).Release)(pShellMenu);
        pShellMenu = core::ptr::null_mut();
    }

    result
}
