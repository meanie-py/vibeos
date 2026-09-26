//! shell32's DllRegisterServer / DllUnregisterServer — Phase 3 of retiring shell32.cpp, where
//! they were ATL's CComModule over BEGIN_OBJECT_MAP plus three extra registry scripts.
//!
//! WHAT ATL DID, reproduced step for step (sdk/lib/atl/atlbase.h):
//!   register:   each object-map entry's UpdateRegistry(TRUE) in map order, stopping at the
//!               first failure; then the type library (AtlLoadTypeLib + RegisterTypeLib);
//!               then shell32.cpp's own UpdateRegistryFromResource over IDR_FOLDEROPTIONS,
//!               IDR_EXPLORER, IDR_SYSTEMFILEASSOC (stopping at the first failure); then
//!               SHELL_RegisterShellFolders.
//!   unregister: each entry's UpdateRegistry(FALSE), same order and stopping rule; then the
//!               type library via its own TLIBATTR; then the three scripts with FALSE.
//!   UpdateRegistryFromResource(id): a FRESH registrar per call, with `Module` and `Module_Raw`
//!               both replaced by GetModuleFileNameW of this DLL, then Resource(Un)Register
//!               of resource `id`, type "REGISTRY". shell32's 38 scripts use only %MODULE%
//!               and `%%`.
//!
//! The registrar is ATL's STATIC one, the parser shell32.cpp compiled in, ported to Rust in
//! statreg.rs. atl100.dll's (Wine's) registrar was tried first and cannot parse shell32's
//! scripts — it has no `e` value type — which tools/payloads/regdiff measured; that payload
//! diffs everything DllRegisterServer and DllUnregisterServer write against a baseline build.
#![allow(non_snake_case)]

use crate::island::{GetLastError, GetModuleFileNameW, DWORD, HRESULT, LPCWSTR, LPWSTR, UINT, WCHAR};
use crate::shell32::statreg::RegObject;

const S_OK: HRESULT = 0;
const E_POINTER: HRESULT = 0x8000_4003_u32 as HRESULT;
const MAX_PATH: usize = 260;
const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

#[inline]
fn failed(hr: HRESULT) -> bool {
    hr < 0
}

#[inline]
fn hresult_from_win32(err: DWORD) -> HRESULT {
    // winerror.h HRESULT_FROM_WIN32: 0 stays 0, anything else is FACILITY_WIN32 | code.
    if err as i32 <= 0 { err as HRESULT } else { ((err & 0xFFFF) | 0x8007_0000) as HRESULT }
}

const fn w<const N: usize>(s: &str) -> [WCHAR; N] {
    let b = s.as_bytes();
    let mut out = [0 as WCHAR; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as WCHAR;
        i += 1;
    }
    out
}
static W_MODULE: [WCHAR; 7] = w("Module");
static W_MODULE_RAW: [WCHAR; 11] = w("Module_Raw");
static W_REGISTRY: [WCHAR; 9] = w("REGISTRY");
static W_APPID: [WCHAR; 6] = w("APPID");
static W_EMPTY: [WCHAR; 1] = [0];
static W_DOT_TLB: [WCHAR; 5] = w(".tlb");

extern "system" {
    fn LoadTypeLib(file: LPCWSTR, tlib: *mut *mut com_sys::ITypeLib) -> HRESULT;
    fn RegisterTypeLib(tlib: *mut com_sys::ITypeLib, path: LPCWSTR, helpdir: LPCWSTR) -> HRESULT;
    fn UnRegisterTypeLib(
        libid: *const com_sys::GUID, major: u16, minor: u16, lcid: u32, syskind: com_sys::SYSKIND,
    ) -> HRESULT;
}

/// This DLL's path, as ATL's CommonInitRegistrar computes it.
unsafe fn module_path(buf: &mut [WCHAR; MAX_PATH + 5]) -> Result<usize, HRESULT> {
    let len = GetModuleFileNameW(
        crate::shell32::dllmain::shell32_hInstance, buf.as_mut_ptr(), MAX_PATH as DWORD,
    ) as usize;
    if len == MAX_PATH {
        return Err(hresult_from_win32(ERROR_INSUFFICIENT_BUFFER));
    }
    if len == 0 {
        return Err(hresult_from_win32(GetLastError()));
    }
    Ok(len)
}

/// CAtlModule::UpdateRegistryFromResource(id, register).
unsafe fn update_registry_from_resource(id: UINT, register: bool) -> HRESULT {
    let mut path = [0 as WCHAR; MAX_PATH + 5];
    if let Err(hr) = module_path(&mut path) {
        return hr;
    }
    // CommonInitRegistrar: AddCommonRGSReplacements (APPID, which CAtlModuleT::GetAppId
    // leaves empty), then Module and Module_Raw, both the module path.
    let mut reg = RegObject::new();
    let mut hr = reg.add_replacement(W_APPID.as_ptr(), W_EMPTY.as_ptr());
    if !failed(hr) {
        hr = reg.add_replacement(W_MODULE.as_ptr(), path.as_ptr());
    }
    if !failed(hr) {
        hr = reg.add_replacement(W_MODULE_RAW.as_ptr(), path.as_ptr());
    }
    if !failed(hr) {
        hr = reg.resource_register(path.as_ptr(), id, W_REGISTRY.as_ptr(), register);
    }
    hr
}

/// AtlLoadTypeLib(inst, NULL, ...): the module itself, else `<module>.tlb`.
unsafe fn load_type_lib(path: &mut [WCHAR; MAX_PATH + 5], tlib: *mut *mut com_sys::ITypeLib) -> HRESULT {
    let len = match module_path(path) {
        Ok(n) => n,
        Err(hr) => return hr,
    };
    let mut hr = LoadTypeLib(path.as_ptr(), tlib);
    if failed(hr) {
        // Replace the extension (or append one) with ".tlb", searching back for '\\' or '.'.
        let mut p = len - 1;
        while p > 0 && path[p] != b'\\' as WCHAR && path[p] != b'.' as WCHAR {
            p -= 1;
        }
        if path[p] != b'.' as WCHAR {
            p = len;
        }
        path[p..p + W_DOT_TLB.len()].copy_from_slice(&W_DOT_TLB);
        hr = LoadTypeLib(path.as_ptr(), tlib);
    }
    hr
}

/// shell32.cpp's BEGIN_OBJECT_MAP order; each entry's DECLARE_REGISTRY_RESOURCEID (or none).
/// Generated from the class headers the CXxxRustEntry shims inherited from.
static OBJECT_MAP_REGISTRY: [Option<UINT>; 36] = [
    // CLSID_ActiveDesktop: IDR_ACTIVEDESKTOP
    Some(157),
    // CLSID_ShellFSFolder: IDR_SHELLFSFOLDER
    Some(143),
    // CLSID_MyComputer: IDR_MYCOMPUTER
    Some(136),
    // CLSID_ShellDesktop: IDR_SHELLDESKTOP
    Some(142),
    // CLSID_ShellFileDefExt: IDR_FILEDEFEXT
    Some(147),
    // CLSID_ShellDrvDefExt: IDR_DRVDEFEXT
    Some(148),
    // CLSID_ShellItem: DECLARE_NO_REGISTRY / an UpdateRegistry that returns S_OK — registers nothing
    None,
    // CLSID_ShellLink: IDR_SHELLLINK
    Some(144),
    // CLSID_Shell: IDR_SHELL
    Some(156),
    // CLSID_ShellLibrary: IDR_SHELLLIBRARY
    Some(165),
    // CLSID_DragDropHelper: IDR_DRAGDROPHELPER
    Some(131),
    // CLSID_ControlPanel: IDR_CONTROLPANEL
    Some(130),
    // CLSID_OpenControlPanel: DECLARE_NO_REGISTRY / an UpdateRegistry that returns S_OK — registers nothing
    None,
    // CLSID_MyDocuments: IDR_MYDOCUMENTS
    Some(137),
    // CLSID_NetworkPlaces: IDR_NETWORKPLACES
    Some(138),
    // CLSID_FontsFolderShortcut: IDR_FONTSFOLDERSHORTCUT
    Some(134),
    // CLSID_Printers: IDR_PRINTERS
    Some(140),
    // CLSID_AdminFolderShortcut: IDR_ADMINFOLDERSHORTCUT
    Some(128),
    // CLSID_ShellFldSetExt: IDR_FOLDEROPTIONS
    Some(132),
    // CLSID_RecycleBinCleaner: IDR_RECYCLEBINCLEANER
    Some(162),
    // CLSID_RecycleBin: IDR_RECYCLEBIN
    Some(141),
    // CLSID_OpenWithMenu: IDR_OPENWITHMENU
    Some(146),
    // CLSID_NewMenu: IDR_NEWMENU
    Some(139),
    // CLSID_SendToMenu: IDR_SENDTOMENU
    Some(158),
    // CLSID_CopyAsPathMenu: IDR_COPYASPATHMENU
    Some(161),
    // CLSID_CopyToMenu: IDR_COPYTOMENU
    Some(159),
    // CLSID_MoveToMenu: IDR_MOVETOMENU
    Some(160),
    // CLSID_StartMenu: IDR_STARTMENU
    Some(145),
    // CLSID_MenuBandSite: IDR_MENUBANDSITE
    Some(135),
    // CLSID_MenuBand: IDR_MENUBAND
    Some(149),
    // CLSID_MenuDeskBar: IDR_MENUDESKBAR
    Some(150),
    // CLSID_MergedFolder: IDR_MERGEDFOLDER
    Some(153),
    // CLSID_ExeDropHandler: IDR_EXEDROPHANDLER
    Some(151),
    // CLSID_QueryAssociations: IDR_QUERYASSOCIATIONS
    Some(152),
    // CLSID_UserNotification: IDR_USERNOTIFICATION
    Some(155),
    // CLSID_UserEventTimer: IDR_USEREVENTTIMER
    Some(166),
];

/// shell32.cpp's own `static const BYTE resid[]`.
static EXTRA_REGISTRY: [UINT; 3] = [132 /* IDR_FOLDEROPTIONS */, 163 /* IDR_EXPLORER */, 164 /* IDR_SYSTEMFILEASSOC */];

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut hr = S_OK;
    for id in OBJECT_MAP_REGISTRY.iter().flatten() {
        hr = update_registry_from_resource(*id, true);
        if failed(hr) {
            break;
        }
    }
    if !failed(hr) {
        let mut path = [0 as WCHAR; MAX_PATH + 5];
        let mut tlib: *mut com_sys::ITypeLib = core::ptr::null_mut();
        hr = load_type_lib(&mut path, &mut tlib);
        if !failed(hr) {
            hr = RegisterTypeLib(tlib, path.as_ptr(), core::ptr::null());
            if let Some(release) = (*(*tlib).lpVtbl).Release {
                release(tlib);
            }
        }
    }
    if failed(hr) {
        return hr;
    }
    for id in EXTRA_REGISTRY.iter() {
        hr = update_registry_from_resource(*id, true);
        if failed(hr) {
            return hr;
        }
    }
    hr = crate::wine::shellpath::SHELL_RegisterShellFolders();
    if failed(hr) {
        return hr;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let mut hr = S_OK;
    for id in OBJECT_MAP_REGISTRY.iter().flatten() {
        hr = update_registry_from_resource(*id, false);
        if failed(hr) {
            break;
        }
    }
    if !failed(hr) {
        let mut path = [0 as WCHAR; MAX_PATH + 5];
        let mut tlib: *mut com_sys::ITypeLib = core::ptr::null_mut();
        hr = load_type_lib(&mut path, &mut tlib);
        if !failed(hr) {
            let vt = &*(*tlib).lpVtbl;
            let mut attr: *mut com_sys::TLIBATTR = core::ptr::null_mut();
            // com_sys types every vtable slot as Option; a live ITypeLib fills them all, so
            // an empty one is reported as E_POINTER rather than trusted.
            hr = match vt.GetLibAttr {
                Some(get_lib_attr) => get_lib_attr(tlib, &mut attr),
                None => E_POINTER,
            };
            if !failed(hr) {
                hr = UnRegisterTypeLib(
                    &(*attr).guid, (*attr).wMajorVerNum, (*attr).wMinorVerNum, (*attr).lcid,
                    (*attr).syskind,
                );
                if let Some(release_attr) = vt.ReleaseTLibAttr {
                    release_attr(tlib, attr);
                }
            }
            if let Some(release) = vt.Release {
                release(tlib);
            }
        }
    }
    if failed(hr) {
        return hr;
    }
    for id in EXTRA_REGISTRY.iter() {
        hr = update_registry_from_resource(*id, false);
        if failed(hr) {
            return hr;
        }
    }
    S_OK
}
