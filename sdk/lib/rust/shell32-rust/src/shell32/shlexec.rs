//! `dll/win32/shell32/shlexec.cpp`, translated and FLIPPED (2026-08-01) — the C is commented
//! out of the CMake source list and these `#[no_mangle]` definitions are what shell32 links.
//! What it was hiding, and what its substrate needed, is in `shell32/mod.rs` beside the
//! `pub mod shlexec;`. This is a PAID artifact — do not delete it to tidy up.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0C71B3CB5580: [WCHAR; 8] = [0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_0D13EF6B270F: [WCHAR; 6] = [0x0050, 0x0041, 0x0054, 0x0048, 0x003D, 0x0000];
pub static CPP_COM_WSTR_16420528D4CF: [WCHAR; 4] = [0x003A, 0x0025, 0x0070, 0x0000];
pub static CPP_COM_WSTR_1B9334FEECE6: [WCHAR; 2] = [0x0022, 0x0000];
pub static CPP_COM_WSTR_1BC9EA5B0770: [WCHAR; 7] = [0x005C, 0x0074, 0x006F, 0x0070, 0x0069, 0x0063, 0x0000];
pub static CPP_COM_WSTR_1EDCE7FBFAA4: [WCHAR; 9] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_31DA8B28DF2D: [WCHAR; 8] = [0x0068, 0x0074, 0x0074, 0x0070, 0x003A, 0x002F, 0x002F, 0x0000];
pub static CPP_COM_WSTR_4FB34BD59E1D: [WCHAR; 11] = [0x0065, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000];
pub static CPP_COM_WSTR_5548E462B2EE: [WCHAR; 51] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_58EE07D1B44C: [WCHAR; 19] = [0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0045, 0x0078, 0x0065, 0x0063, 0x0075, 0x0074, 0x0065, 0x0048, 0x006F, 0x006F, 0x006B, 0x0073, 0x0000];
pub static CPP_COM_WSTR_6C1D0DF6BC60: [WCHAR; 8] = [0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x005C, 0x0000];
pub static CPP_COM_WSTR_6E8A9C6CE19B: [WCHAR; 5] = [0x0028, 0x0025, 0x0073, 0x0029, 0x0000];
pub static CPP_COM_WSTR_731D7EC48EEF: [WCHAR; 3] = [0x0020, 0x0009, 0x0000];
pub static CPP_COM_WSTR_7ABCE230ED71: [WCHAR; 5] = [0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_7B35AAC93EFF: [WCHAR; 4] = [0x002E, 0x0073, 0x006F, 0x0000];
pub static CPP_COM_WSTR_7DA531BC3C5E: [WCHAR; 20] = [0x0065, 0x0078, 0x0065, 0x0020, 0x0070, 0x0069, 0x0066, 0x0020, 0x0062, 0x0061, 0x0074, 0x0020, 0x0063, 0x006D, 0x0064, 0x0020, 0x0063, 0x006F, 0x006D, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_87660BB31B23: [WCHAR; 28] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0065, 0x0078, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0065, 0x0078, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0073, 0x0000];
pub static CPP_COM_WSTR_878FA9FF9662: [WCHAR; 6] = [0x0072, 0x0075, 0x006E, 0x0061, 0x0073, 0x0000];
pub static CPP_COM_WSTR_8C31B1E57F5A: [WCHAR; 8] = [0x005C, 0x0069, 0x0066, 0x0065, 0x0078, 0x0065, 0x0063, 0x0000];
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_8D831E206EA6: [WCHAR; 53] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0041, 0x0070, 0x0070, 0x0020, 0x0050, 0x0061, 0x0074, 0x0068, 0x0073, 0x005C, 0x0000];
pub static CPP_COM_WSTR_94D8FB2B7BE6: [WCHAR; 8] = [0x0077, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0000];
pub static CPP_COM_WSTR_94FDE40D078A: [WCHAR; 4] = [0x0077, 0x0077, 0x0077, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9896841D0A6A: [WCHAR; 5] = [0x0022, 0x0025, 0x0073, 0x0022, 0x0000];
pub static CPP_COM_WSTR_9A56E6BBE687: [WCHAR; 5] = [0x0050, 0x0061, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_A16E78DA5D50: [WCHAR; 9] = [0x005C, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_AAEB94F421D7: [WCHAR; 7] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x003A, 0x0000];
pub static CPP_COM_WSTR_AB68B18B313D: [WCHAR; 13] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_B74AC138CDFD: [WCHAR; 7] = [0x0072, 0x0075, 0x006E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_C9E3E098911A: [WCHAR; 8] = [0x0064, 0x0064, 0x0065, 0x0065, 0x0078, 0x0065, 0x0063, 0x0000];
pub static CPP_COM_WSTR_E7BEDDFB7AB6: [WCHAR; 13] = [0x004D, 0x0061, 0x0078, 0x0069, 0x006D, 0x0069, 0x007A, 0x0065, 0x0041, 0x0070, 0x0070, 0x0073, 0x0000];
pub static CPP_COM_WSTR_E9A72DDF42EB: [WCHAR; 7] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_F2DD73FBEC92: [WCHAR; 4] = [0x003A, 0x003A, 0x007B, 0x0000];
pub static CPP_COM_WSTR_F99F36C2F40B: [WCHAR; 9] = [0x0070, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0000];
pub static CPP_COM_WSTR_FC9A94A80631: [WCHAR; 13] = [0x005C, 0x0061, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::wine::classes::HCR_GetProgIdKeyOfExtension;
use crate::wine::pidl::ILGetSize;
use crate::prelude::*;
use crate::shell32::sdk_consts::*;
// Both globs define this; an explicit import beats a glob, so it cannot go ambiguous.
use crate::prelude::INVALID_FILE_ATTRIBUTES;
// The island renders these as bindgen C++ classes (`vtable_`), which have no callable
// members; the translated call sites go through the C `lpVtbl`. Explicit beats glob.
use crate::shellcom::{IContextMenu, IUnknown};

// shell32's own helpers and the shlwapi imports the island's include closure does not reach.
// Declared here rather than in the prelude for the reason `shlfileop` gives for its own block:
// the Rust modules that DEFINE these are typed against `crate::types`, so importing one drags
// its aliases into an island-first TU and splits every call site nominally.
//
// Three of these draw a `redeclared with a different signature` warning against the crate's
// OTHER declaration of the same export, and every one is the alias spelling rather than the
// ABI: `*mut *mut HKEY__` vs `*mut *mut c_void`, `extern "system"` vs `extern "stdcall"` on
// i386. That is the same nominal split prelude.rs documents, and ~80 of these already exist
// crate-wide. Arity, widths and pointer classes were checked against each counterpart.
extern "system" {
    /// shlwapi.
    fn PathFindFileNameW(path: LPCWSTR) -> LPWSTR;
    /// `COpenWithMenu.cpp:100` has C++ linkage and no header declaration, so there is no
    /// unmangled `SH32_InvokeOpenWith` to bind to — the same file already exports a WINAPI
    /// bridge for exactly this, which `open_with_menu.rs` also uses. (`win32.rs` declares the
    /// bare name in an `extern "system"` block; that one has no callers and would not link.)
    #[link_name = "Shell32RustInvokeOpenWith"]
    fn SH32_InvokeOpenWith(path: PCWSTR, info: LPCMINVOKECOMMANDINFO, process: *mut HANDLE) -> HRESULT;
}
extern "C" {
    /// `shpolicy.c` — plain __cdecl, NOT WINAPI, so `extern "C"`: `extern "system"` would look
    /// for `_SH32_InternalRestricted@4` and i386 is the only arch that catches the difference.
    fn SH32_InternalRestricted(rest: DWORD) -> DWORD;
    /// `classes.c` — both __cdecl, same reasoning.
    fn HCR_GetDefaultVerbW(hkeyClass: HKEY, szVerb: LPCWSTR, szDest: LPWSTR, len: DWORD) -> BOOL;
}
extern "C" {
    /// The interface ids. `IID_IShellExtInit` / `IID_IShellExecuteHookW` come from the island;
    /// these three do not, because nothing else in shell32's include closure names them.
    static IID_IUnknown: GUID;
    static IID_IDataObject: GUID;
    static IID_IObjectWithSite: GUID;
}

/// wtypes.idl:341. Verified against shlexec.cpp's own include closure with the cross-compiler
/// rather than transcribed — a wrong constant compiles clean and misbehaves at runtime.
const CLSCTX_INPROC_SERVER: DWORD = 1;

/// `SHELL_ErrorBox` is already bound crate-wide against its `_RustBridge` WINAPI shim; import
/// the one binding rather than declaring a second, which would be two symbols for one export.
use crate::win32::SHELL_ErrorBox;

// Deterministic from shobjidl.h. Declared file-locally like the five other TUs that use it —
// see the note in `shellcom.rs` on why it does NOT live there.
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(
        This: *mut IShellExtInit,
        pidlFolder: PCIDLIST_ABSOLUTE,
        pdtobj: *mut core::ffi::c_void,
        hkeyProgID: HKEY,
    ) -> HRESULT,
}

// shobjidl.h. The island DOES carry a name for each of these, and neither is usable here:
// `IShellExecuteHookW` arrives as a bindgen C++ class whose vtable is OPAQUE
// (`IShellExecuteHookW__bindgen_vtable(c_void)`), so `p.Execute(..)` resolves to nothing at all;
// `IObjectWithSite` is re-exported from com-sys, whose slots are `Option<fn>` — calling one is
// E0618 and `.unwrap()`ing it would put a panic on a COM call in a shipping DLL. Both are
// declared file-locally, in the `lpVtbl` shape the rest of the crate uses. An item defined in
// the module beats a glob import, so these shadow the island rather than colliding with it.
#[repr(C)]
pub struct IShellExecuteHookW {
    pub lpVtbl: *const IShellExecuteHookWVtbl,
}
#[repr(C)]
pub struct IShellExecuteHookWVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellExecuteHookW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExecuteHookW) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExecuteHookW) -> ULONG,
    pub Execute:
        unsafe extern "system" fn(This: *mut IShellExecuteHookW, pei: LPSHELLEXECUTEINFOW) -> HRESULT,
}

#[repr(C)]
pub struct IObjectWithSite {
    pub lpVtbl: *const IObjectWithSiteVtbl,
}
#[repr(C)]
pub struct IObjectWithSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub SetSite:
        unsafe extern "system" fn(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT,
    pub GetSite: unsafe extern "system" fn(
        This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
}

// ---- file-scope preamble (translated) ----
pub const SEE_MASK_CLASSALL: u32 = SEE_MASK_CLASSNAME | SEE_MASK_CLASSKEY;

pub type SHELL_ExecuteW32 = unsafe extern "C" fn(
    lpCmd: *const u16,
    env: *mut u16,
    shWait: i32,
    sei: *const SHELLEXECUTEINFOW,
    sei_out: *mut SHELLEXECUTEINFOW,
) -> UINT_PTR;

#[repr(C)]
pub struct _RUNASDLGDATA {
    pub Name: *mut u16,
    pub Domain: *mut u16,
    pub Safer: i32,
    pub LogonFlags: u32,
    pub NameBuffer: [u16; MAX_PATH as usize],
    pub Password: [u16; MAX_PATH as usize],
}

pub type RUNASDLGDATA = _RUNASDLGDATA;
unsafe extern "C" fn Win32ErrFromHInst(hInst: HINSTANCE) -> i32 {
    if (hInst as usize) > 32 {
        return (ERROR_SUCCESS) as i32;
    }
    match hInst as usize as u32 {
        SE_ERR_FNF
        | SE_ERR_PNF
        | SE_ERR_ACCESSDENIED
        | SE_ERR_OOM => (hInst as usize) as i32,
        SE_ERR_SHARE => (ERROR_SHARING_VIOLATION) as i32,
        SE_ERR_DDETIMEOUT
        | SE_ERR_DDEFAIL
        | SE_ERR_DDEBUSY => (ERROR_DDE_FAIL) as i32,
        SE_ERR_DLLNOTFOUND => (ERROR_DLL_NOT_FOUND) as i32,
        //case SE_ERR_ASSOCINCOMPLETE: Note: Windows treats this as a success code
        SE_ERR_NOASSOC => (ERROR_NO_ASSOCIATION) as i32,
        10 => (ERROR_OLD_WIN_VERSION) as i32,
        12 => (ERROR_APP_WRONG_OS) as i32,
        15 => (ERROR_RMODE_APP) as i32,
        16 => (ERROR_SINGLE_INSTANCE_APP) as i32,
        20 => (ERROR_INVALID_DLL) as i32,
        _ => -1,
    }
}
unsafe extern "C" fn SHELL_InRunDllProcess() -> i32 {
    let mut szModule: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    static mut s_bInDllProcess: i32 = -1;

    if s_bInDllProcess != -1 {
        return s_bInDllProcess;
    }

    s_bInDllProcess = if GetModuleFileNameW(
        core::ptr::null_mut(),
        szModule.as_mut_ptr(),
        _countof!(szModule),
    ) != 0
        && StrStrIW(PathFindFileNameW(szModule.as_ptr()), CPP_COM_WSTR_B74AC138CDFD.as_ptr())
            != core::ptr::null_mut()
    {
        1
    } else {
        0
    };
    s_bInDllProcess
}
unsafe extern "C" fn InvokeOpenWith(hWndOwner: HWND, sei: &mut SHELLEXECUTEINFOW) -> UINT_PTR {
    let phProc = if ((*sei).fMask & SEE_MASK_NOCLOSEPROCESS) != 0 {
        core::ptr::addr_of_mut!((*sei).hProcess)
    } else {
        core::ptr::null_mut()
    };
    let fCmic = ((*sei).fMask & SEE_CMIC_COMMON_BASICFLAGS) | CMIC_MASK_FLAG_NO_UI;
    let mut ici = CMINVOKECOMMANDINFO {
        cbSize: core::mem::size_of::<CMINVOKECOMMANDINFO>() as DWORD,
        fMask: fCmic,
        hwnd: hWndOwner,
        lpVerb: core::ptr::null(),
        lpParameters: core::ptr::null(),
        lpDirectory: core::ptr::null(),
        nShow: (SW_SHOW) as i32,
        dwHotKey: 0,
        hIcon: core::ptr::null_mut(),
    };
    let hr = SH32_InvokeOpenWith((*sei).lpFile, &mut ici, phProc);
    SetLastError(ERROR_NO_ASSOCIATION);
    if (hr) >= 0 {
        42
    } else {
        (SE_ERR_NOASSOC) as UINT_PTR
    }
}
unsafe extern "C" fn InvokeShellExecuteHook(pszClsid: PCWSTR, pSEI: LPSHELLEXECUTEINFOW) -> HRESULT {
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    if ((SHExtCoCreateInstance(pszClsid, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of!(IID_IUnknown), core::ptr::addr_of_mut!(pUnk).cast())) < 0) {
        let result = S_FALSE;
        if !pUnk.is_null() {
            ((*(*pUnk).lpVtbl).Release)(pUnk);
        }
        return result;
    }
    let mut pWide: *mut IShellExecuteHookW = core::ptr::null_mut();
    if ((((*(*pUnk).lpVtbl).QueryInterface)(pUnk, core::ptr::addr_of!(IID_IShellExecuteHookW).cast(), core::ptr::addr_of_mut!(pWide).cast())) >= 0) {
        let result = ((*(*pWide).lpVtbl).Execute)(pWide, pSEI);
        if !pWide.is_null() {
            ((*(*pWide).lpVtbl).Release)(pWide);
        }
        if !pUnk.is_null() {
            ((*(*pUnk).lpVtbl).Release)(pUnk);
        }
        return result;
    }
    let hr = S_FALSE;
    if !pUnk.is_null() {
        ((*(*pUnk).lpVtbl).Release)(pUnk);
    }
    hr
}
unsafe extern "C" fn TryShellExecuteHooks(pSEI: LPSHELLEXECUTEINFOW) -> HRESULT {
    if (*pSEI).fMask & SEE_MASK_NO_HOOKS != 0 {
        return S_FALSE;
    }
    if LOBYTE(GetVersion()) >= 6 &&
        SH32_InternalRestricted(REST_SH32_ENABLESHELLEXECUTEHOOKS) == 0
    {
        return S_FALSE;
    }

    let mut hr: HRESULT = S_FALSE;
    let mut hKey: HKEY = core::ptr::null_mut();

    let mut subkey: [u16; 256] = [0u16; 256];
    let mut pos = 0;
    let mut s1: PCWSTR = REGSTR_PATH_EXPLORER.as_ptr();
    while *s1 != 0 {
        subkey[pos] = *s1;
        s1 = s1.offset(1);
        pos += 1;
    }
    let mut s2: PCWSTR = CPP_COM_WSTR_58EE07D1B44C.as_ptr() as PCWSTR;
    while *s2 != 0 {
        subkey[pos] = *s2;
        s2 = s2.offset(1);
        pos += 1;
    }
    let subkey_ptr: PCWSTR = subkey.as_mut_ptr();

    let res = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        subkey_ptr,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    );
    if res != (ERROR_SUCCESS) as i32 {
        return S_FALSE;
    }

    let mut i: u32 = 0 as u32;
    loop {
        if hr != S_FALSE {
            break;
        }
        let mut szClsid: [u16; 42] = [0u16; 42];
        let mut cch: DWORD = 42 as u32;
        let enum_res = RegEnumValueW(
            hKey,
            i,
            szClsid.as_mut_ptr(),
            core::ptr::addr_of_mut!(cch),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if enum_res != (ERROR_SUCCESS) as i32 {
            break;
        }
        hr = InvokeShellExecuteHook(szClsid.as_ptr() as PCWSTR, pSEI);
        i += 1;
    }

    RegCloseKey(hKey);
    hr
}
unsafe extern "C" fn ParseNoTildeEffect(
    res: &mut PWSTR,
    args: &mut LPCWSTR,
    len: &mut u32,
    used: &mut u32,
    argNum: i32,
) {
    let mut first_char_quote: bool = false;
    let mut quotes_opened: bool = false;
    let mut backslash_encountered: bool = false;

    let cur_arg_start: i32 = 0;
    let offset: isize = 1 as isize;

    let mut cur_arg = cur_arg_start;
    while cur_arg <= argNum && **args != 0 {
        first_char_quote = false;
        if **args == 0x22u16 {
            quotes_opened = true;
            first_char_quote = true;
            *args = (*args).add(1);
        }

        while **args != 0 {
            if **args == 0x5Cu16 {
                backslash_encountered = !backslash_encountered;
            } else if **args == 0x22u16 {
                if quotes_opened {
                    if *((*args).add(offset as usize)) != 0x22u16 {
                        quotes_opened = false;
                        *args = (*args).add(1);
                        break;
                    } else {
                        *args = (*args).add(1);
                    }
                } else {
                    quotes_opened = true;
                }
                backslash_encountered = false;
            } else {
                backslash_encountered = false;
                if **args == 0x20u16 && !first_char_quote {
                    break;
                }
            }

            if cur_arg == argNum {
                *used += 1;
                if *used < *len {
                    **res = **args;
                    *res = (*res).add(1);
                }
            }

            *args = (*args).add(1);
        }

        while **args == 0x20u16 {
            *args = (*args).add(1);
        }

        cur_arg += 1;
    }
}
unsafe extern "C" fn ParseTildeEffect(
    res: &mut PWSTR,
    args: &mut LPCWSTR,
    len: &mut DWORD,
    used: &mut DWORD,
    argNum: i32,
) {
    let mut quotes_opened = false;
    let mut backslash_encountered = false;

    let mut curArg = 0;
    while curArg <= argNum && !(*args).is_null() && **args != 0 {
        while **args != 0 {
            if **args == (b'\\') as u16 {
                backslash_encountered = !backslash_encountered;
            } else if **args == (b'"') as u16 {
                if quotes_opened {
                    if *((*args).add(1 as usize)) != (b'"') as u16 {
                        quotes_opened = false;
                    } else {
                        *args = (*args).offset(1);
                    }
                } else {
                    quotes_opened = true;
                }
                backslash_encountered = false;
            } else {
                backslash_encountered = false;
                if **args == (b' ') as u16 && !quotes_opened && curArg != argNum {
                    break;
                }
            }

            if curArg == argNum {
                *used += 1;
                if *used < *len {
                    **res = **args;
                    *res = (*res).offset(1);
                }
            }

            *args = (*args).offset(1);
        }
        curArg += 1;
    }
}
unsafe extern "C" fn SHELL_ArgifyW(
    out: PWSTR,
    mut len: DWORD,
    fmt: PCWSTR,
    lpFile: PCWSTR,
    pidl: LPITEMIDLIST,
    args: LPCWSTR,
    out_len: *mut DWORD,
    lpDir: PCWSTR,
) -> u8 {
    let mut done: u8 = (FALSE) as u8;
    let mut found_p1: u8 = (FALSE) as u8;
    let mut res: PWSTR = out;
    let mut used: DWORD = 0;
    let mut tildeEffect = false;
    let mut fmt_ptr = fmt;
    let mut args_mut: LPCWSTR = args;

    while *fmt_ptr != 0 {
        if *fmt_ptr == b'%' as u16 {
            fmt_ptr = fmt_ptr.add(1);
            match *fmt_ptr {
                0x00 /* b'\0' */ | 0x25 /* b'%' */ => {
                    used += 1;
                    if used < len {
                        *res = b'%' as u16;
                        res = res.add(1);
                    }
                }
                0x2A /* b'*' */ => {
                    if !args_mut.is_null() {
                        if *fmt_ptr == b'*' as u16 {
                            used += 1;
                            while *args_mut != 0 {
                                used += 1;
                                if used < len {
                                    *res = *args_mut;
                                    res = res.add(1);
                                    args_mut = args_mut.add(1);
                                } else {
                                    args_mut = args_mut.add(1);
                                }
                            }
                            used += 1;
                        }
                    }
                }
                0x7E /* b'~' */ | 0x32 /* b'2' */..=0x39 /* b'9' */ => {
                    if *fmt_ptr == b'~' as u16 {
                        fmt_ptr = fmt_ptr.add(1);
                        tildeEffect = true;
                    }
                    if !args_mut.is_null() {
                        if tildeEffect {
                            ParseTildeEffect(
                                &mut res,
                                &mut args_mut,
                                &mut len,
                                &mut used,
                                (*fmt_ptr as i32) - (b'2' as i32),
                            );
                            tildeEffect = false;
                        } else {
                            ParseNoTildeEffect(
                                &mut res,
                                &mut args_mut,
                                &mut len,
                                &mut used,
                                (*fmt_ptr as i32) - (b'2' as i32),
                            );
                        }
                    }
                }
                0x31 /* b'1' */ => {
                    if (done == 0 || *fmt_ptr == b'1' as u16) && !lpFile.is_null() {
                        let filelen = wcslen(lpFile);
                        used += filelen as u32;
                        if used < len {
                            wcscpy(res, lpFile);
                            res = res.add(filelen as usize);
                        }
                    }
                    found_p1 = (TRUE) as u8;
                }
                0x6C /* b'l' */ | 0x4C /* b'L' */ => {
                    if !lpFile.is_null() {
                        let filelen_used = wcslen(lpFile);
                        used += filelen_used as u32;
                        if used < len {
                            wcscpy(res, lpFile);
                            res = res.add(wcslen(lpFile) as usize);
                        }
                    }
                    found_p1 = (TRUE) as u8;
                }
                0x77 /* b'w' */ | 0x57 /* b'W' */ => {
                    if !lpDir.is_null() {
                        let dirlen_used = wcslen(lpDir);
                        used += dirlen_used as u32;
                        if used < len {
                            wcscpy(res, lpDir);
                            res = res.add(wcslen(lpDir) as usize);
                        }
                    }
                }
                0x76 /* b'v' */ | 0x56 /* b'V' */ => {
                    if !lpFile.is_null() {
                        let vlen_used = wcslen(lpFile);
                        used += vlen_used as u32;
                        if used < len {
                            wcscpy(res, lpFile);
                            res = res.add(wcslen(lpFile) as usize);
                        }
                        found_p1 = (TRUE) as u8;
                    } else if !lpDir.is_null() {
                        let vlen_used = wcslen(lpDir);
                        used += vlen_used as u32;
                        if used < len {
                            wcscpy(res, lpDir);
                            res = res.add(wcslen(lpDir) as usize);
                        }
                    }
                }
                0x69 /* b'i' */ | 0x49 /* b'I' */ => {
                    if !pidl.is_null() {
                        let mut chars: DWORD = 0;
                        let mut buf: [u16; 100 as usize] =
                            [0u16; 100 as usize];
                        let hmem = SHAllocShared(
                            pidl.cast(),
                            ILGetSize(pidl),
                            0,
                        );
                        let pv = SHLockShared(hmem, 0);
                        chars = swprintf(
                            buf.as_mut_ptr(),
                            CPP_COM_WSTR_16420528D4CF.as_ptr(),
                            pv,
                        ) as DWORD;
                        if chars >= ARRAY_SIZE!(buf) {
                            ERR!("pidl format buffer too small!\n");
                        }
                        used += chars;
                        if used < len {
                            wcscpy(res, buf.as_mut_ptr());
                            res = res.add(chars as usize);
                        }
                        SHUnlockShared(pv);
                    }
                    found_p1 = (TRUE) as u8;
                }
                _ => {
                    if !strchrW(fmt_ptr, b'%' as u16).is_null() {
                        let mut tmpBuffer: [u16; 1024 as usize] =
                            [0u16; 1024 as usize];
                        let mut tmpB = tmpBuffer.as_mut_ptr();
                        let mut tmpEnvBuff: [u16; MAX_PATH as usize] =
                            [0u16; MAX_PATH as usize];

                        while *fmt_ptr != b'%' as u16 {
                            *tmpB = *fmt_ptr;
                            tmpB = tmpB.add(1);
                            fmt_ptr = fmt_ptr.add(1);
                        }
                        *tmpB = 0;

                        let envRet = GetEnvironmentVariableW(
                            tmpBuffer.as_mut_ptr(),
                            tmpEnvBuff.as_mut_ptr(),
                            MAX_PATH,
                        );
                        if envRet == 0 || envRet > MAX_PATH {
                            let buflen = wcslen(tmpBuffer.as_mut_ptr());
                            used += buflen as u32;
                            if used < len {
                                wcscpy(res, tmpBuffer.as_mut_ptr());
                                res = res.add(wcslen(tmpBuffer.as_mut_ptr()) as usize);
                            }
                        } else {
                            let buflen = wcslen(tmpEnvBuff.as_mut_ptr());
                            used += buflen as u32;
                            if used < len {
                                wcscpy(res, tmpEnvBuff.as_mut_ptr());
                                res = res.add(wcslen(tmpEnvBuff.as_mut_ptr()) as usize);
                            }
                        }
                    }
                    done = (TRUE) as u8;
                }
            }
            if *fmt_ptr != 0 {
                fmt_ptr = fmt_ptr.add(1);
            }
        } else {
            used += 1;
            if used < len {
                *res = *fmt_ptr;
                res = res.add(1);
                fmt_ptr = fmt_ptr.add(1);
            } else {
                fmt_ptr = fmt_ptr.add(1);
            }
        }
    }

    used += 1;
    if (res.offset_from(out) as isize) < (len as isize) {
        *res = 0;
    } else {
        *out.add((len - 1) as usize) = 0u16;
    }

    if !out_len.is_null() {
        *out_len = used;
    }

    found_p1
}
unsafe extern "C" fn SHELL_GetPathFromIDListForExecuteW(pidl: LPCITEMIDLIST, pszPath: LPWSTR, uOutSize: u32) -> HRESULT {
    let mut strret: STRRET = core::mem::zeroed();
    let mut desktop: *mut IShellFolder = core::ptr::null_mut();
    let mut hr = SHGetDesktopFolder(core::ptr::addr_of_mut!(desktop));
    if hr >= 0 {
        hr = ((*(*desktop).lpVtbl).GetDisplayNameOf)(desktop, pidl.cast(), SHGDN_FORPARSING, core::ptr::addr_of_mut!(strret).cast());
        if hr >= 0 {
            StrRetToStrNW(pszPath, uOutSize, (core::ptr::addr_of_mut!(strret)).cast(), (pidl).cast());
        }
    }
    if !desktop.is_null() {
        let p_unk = desktop as *mut IUnknown;
        ((*(*p_unk).lpVtbl).Release)(p_unk);
    }
    hr
}
unsafe extern "C" fn SHELL_GetUsableDialogOwner(hWnd: HWND) -> HWND {
    let hProgman = GetShellWindow();
    if !hWnd.is_null() && IsWindowVisible(hWnd) != 0 && hWnd != GetDesktopWindow() && hWnd != hProgman && IsChild(hProgman, hWnd) == 0 {
        return hWnd;
    }
    core::ptr::null_mut()
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn RunAsDlgProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pData: *mut RUNASDLGDATA = GetWindowLongPtrW(hwnd, DWLP_USER) as *mut RUNASDLGDATA;
    match uMsg {
        WM_INITDIALOG => {
            pData = lParam as *mut RUNASDLGDATA;
            SetWindowLongPtrW(hwnd, DWLP_USER, pData as LPARAM);
            SendDlgItemMessageW(
                hwnd,
                IDC_RUNAS_NAME as i32,
                EM_LIMITTEXT,
                (_countof!((*pData).NameBuffer) - 1) as _,
                0,
            );
            SendDlgItemMessageW(
                hwnd,
                IDC_RUNAS_PWD as i32,
                EM_LIMITTEXT,
                (_countof!((*pData).Password) - 1) as _,
                0,
            );
            SendDlgItemMessageW(
                hwnd,
                IDC_RUNAS_OTHER as i32,
                BM_CLICK,
                0,
                0,
            );
            let hCtl = GetDlgItem(hwnd, IDC_RUNAS_CURRENT as i32);
            let mut fmtbuf: [u16; 200 as usize] = [0u16; 200 as usize];
            const FMTBUF_CCH: usize = 200; // WCHAR fmtbuf[200]
            const BUF_CCH: usize = FMTBUF_CCH + MAX_PATH as usize; // + _countof(NameBuffer)
            let mut buf: [u16; BUF_CCH] = [0u16; BUF_CCH];
            let mut cch: DWORD = _countof!((*pData).NameBuffer);
            if GetUserNameW((*pData).NameBuffer.as_mut_ptr(), &mut cch) != 0 {
                SendMessageW(
                    hCtl,
                    WM_GETTEXT,
                    _countof!(fmtbuf) as _,
                    fmtbuf.as_mut_ptr() as LPARAM,
                );
                strsafe_rust::StringCchPrintfW!(
                    fmtbuf.as_mut_ptr(),
                    _countof!(fmtbuf) as usize,
                    buf.as_mut_ptr(),
                    CPP_COM_WSTR_6E8A9C6CE19B.as_ptr(),
                );
                strsafe_rust::StringCchPrintfW!(
                    buf.as_mut_ptr(),
                    _countof!(buf) as usize,
                    fmtbuf.as_mut_ptr(),
                    (*pData).NameBuffer.as_mut_ptr(),
                );
                SendMessageW(
                    hCtl,
                    WM_SETTEXT,
                    0,
                    buf.as_mut_ptr() as LPARAM,
                );
                SendDlgItemMessageW(
                    hwnd,
                    IDC_RUNAS_NAME as i32,
                    CB_ADDSTRING,
                    0,
                    (*pData).NameBuffer.as_mut_ptr() as LPARAM,
                );
            }
            SendDlgItemMessageW(
                hwnd,
                IDC_RUNAS_NAME as i32,
                CB_SETCURSEL,
                0,
                0,
            );
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD(wParam) as u32 {
                IDCANCEL => {
                    EndDialog(hwnd, (IDCANCEL) as INT_PTR);
                }
                IDOK => {
                    SendDlgItemMessageW(
                        hwnd,
                        IDC_RUNAS_NAME as i32,
                        WM_GETTEXT,
                        _countof!((*pData).NameBuffer) as _,
                        (*pData).NameBuffer.as_mut_ptr() as LPARAM,
                    );
                    SendDlgItemMessageW(
                        hwnd,
                        IDC_RUNAS_PWD as i32,
                        WM_GETTEXT,
                        _countof!((*pData).Password) as _,
                        (*pData).Password.as_mut_ptr() as LPARAM,
                    );
                    (*pData).Name = (*pData).NameBuffer.as_mut_ptr();
                    (*pData).Domain = wcsstr((*pData).Name, CPP_COM_WSTR_B7FC02907B1F.as_ptr());
                    if !(*pData).Domain.is_null() {
                        let tmp = (*pData).Domain.add(1 as usize);
                        *((*pData).Domain.add(0 as usize)) = UNICODE_NULL as u16;
                        (*pData).Domain = (*pData).Name;
                        (*pData).Name = tmp;
                    }
                    (*pData).LogonFlags = if GetKeyState(VK_SHIFT as i32) < 0 {
                        LOGON_NETCREDENTIALS_ONLY
                    } else {
                        LOGON_WITH_PROFILE
                    };
                    (*pData).Safer = IsDlgButtonChecked(hwnd, IDC_RUNAS_SAFER as i32) as BOOL;
                    if IsDlgButtonChecked(hwnd, IDC_RUNAS_OTHER as i32) == 0 {
                        (*pData).Name = core::ptr::null_mut();
                    }
                    EndDialog(hwnd, (IDOK) as INT_PTR);
                }
                IDC_RUNAS_BROWSE => {
                    return SHELL_ErrorBox((hwnd as *mut c_void).cast(), ERROR_NOT_SUPPORTED) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }
    FALSE as INT_PTR
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn RunAsDlgProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pData: *mut RUNASDLGDATA = GetWindowLongPtrW(hwnd, DWLP_USER) as *mut RUNASDLGDATA;
    match uMsg {
        WM_INITDIALOG => {
            pData = lParam as *mut RUNASDLGDATA;
            SetWindowLongPtrW(hwnd, DWLP_USER, pData as LPARAM);
            SendDlgItemMessageW(
                hwnd,
                (IDC_RUNAS_NAME) as i32,
                EM_LIMITTEXT,
                (_countof!((*pData).NameBuffer) - 1) as _,
                0,
            );
            SendDlgItemMessageW(
                hwnd,
                (IDC_RUNAS_PWD) as i32,
                EM_LIMITTEXT,
                (_countof!((*pData).Password) - 1) as _,
                0,
            );
            SendDlgItemMessageW(
                hwnd,
                (IDC_RUNAS_OTHER) as i32,
                BM_CLICK,
                0,
                0,
            );
            let hCtl = GetDlgItem(hwnd, (IDC_RUNAS_CURRENT) as i32);
            let mut fmtbuf: [u16; 200 as usize] = [0u16; 200 as usize];
            const FMTBUF_CCH: usize = 200; // WCHAR fmtbuf[200]
            const BUF_CCH: usize = FMTBUF_CCH + MAX_PATH as usize; // + _countof(NameBuffer)
            let mut buf: [u16; BUF_CCH] = [0u16; BUF_CCH];
            let mut cch: DWORD = _countof!((*pData).NameBuffer);
            if GetUserNameW((*pData).NameBuffer.as_mut_ptr(), &mut cch) != 0 {
                SendMessageW(
                    hCtl,
                    WM_GETTEXT,
                    _countof!(fmtbuf) as _,
                    fmtbuf.as_mut_ptr() as LPARAM,
                );
                strsafe_rust::StringCchPrintfW!(
                    fmtbuf.as_mut_ptr(),
                    _countof!(fmtbuf) as usize,
                    buf.as_mut_ptr(),
                    CPP_COM_WSTR_6E8A9C6CE19B.as_ptr(),
                );
                strsafe_rust::StringCchPrintfW!(
                    buf.as_mut_ptr(),
                    _countof!(buf) as usize,
                    fmtbuf.as_mut_ptr(),
                    (*pData).NameBuffer.as_mut_ptr(),
                );
                SendMessageW(
                    hCtl,
                    WM_SETTEXT,
                    0,
                    buf.as_mut_ptr() as LPARAM,
                );
                SendDlgItemMessageW(
                    hwnd,
                    (IDC_RUNAS_NAME) as i32,
                    CB_ADDSTRING,
                    0,
                    (*pData).NameBuffer.as_mut_ptr() as LPARAM,
                );
            }
            SendDlgItemMessageW(
                hwnd,
                (IDC_RUNAS_NAME) as i32,
                CB_SETCURSEL,
                0,
                0,
            );
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD((wParam) as u32) as u32 {
                IDCANCEL => {
                    EndDialog(hwnd, (IDCANCEL) as INT_PTR);
                }
                IDOK => {
                    SendDlgItemMessageW(
                        hwnd,
                        (IDC_RUNAS_NAME) as i32,
                        WM_GETTEXT,
                        _countof!((*pData).NameBuffer) as _,
                        (*pData).NameBuffer.as_mut_ptr() as LPARAM,
                    );
                    SendDlgItemMessageW(
                        hwnd,
                        (IDC_RUNAS_PWD) as i32,
                        WM_GETTEXT,
                        _countof!((*pData).Password) as _,
                        (*pData).Password.as_mut_ptr() as LPARAM,
                    );
                    (*pData).Name = (*pData).NameBuffer.as_mut_ptr();
                    (*pData).Domain = wcsstr((*pData).Name, CPP_COM_WSTR_B7FC02907B1F.as_ptr());
                    if !(*pData).Domain.is_null() {
                        let tmp = (*pData).Domain.add(1 as usize);
                        *((*pData).Domain.add(0 as usize)) = UNICODE_NULL as u16;
                        (*pData).Domain = (*pData).Name;
                        (*pData).Name = tmp;
                    }
                    (*pData).LogonFlags = if GetKeyState((VK_SHIFT) as i32) < 0 {
                        LOGON_NETCREDENTIALS_ONLY
                    } else {
                        LOGON_WITH_PROFILE
                    };
                    (*pData).Safer = (IsDlgButtonChecked(hwnd, (IDC_RUNAS_SAFER) as i32)) as i32;
                    if IsDlgButtonChecked(hwnd, (IDC_RUNAS_OTHER) as i32) == 0 {
                        (*pData).Name = core::ptr::null_mut();
                    }
                    EndDialog(hwnd, (IDOK) as INT_PTR);
                }
                IDC_RUNAS_BROWSE => {
                    return SHELL_ErrorBox((hwnd as *mut c_void).cast(), ERROR_NOT_SUPPORTED) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }
    FALSE as INT_PTR
}
unsafe extern "C" fn PromptAndRunProcessAs(
    hwnd: HWND,
    Cmd: PWSTR,
    CreationFlags: DWORD,
    Env: PWSTR,
    Dir: PCWSTR,
    pSI: *mut STARTUPINFOW,
    pPI: *mut PROCESS_INFORMATION,
) -> HRESULT {
    let mut data: RUNASDLGDATA = core::mem::zeroed();
    let mut error: UINT;
    let mut dlgret: INT_PTR;
    let hwnd = SHELL_GetUsableDialogOwner(hwnd);

    'again: loop {
        dlgret = DialogBoxParamW(
            shell32_hInstance,
            MAKEINTRESOURCEW(IDD_RUN_AS),
            hwnd,
            Some(RunAsDlgProc),
            core::ptr::addr_of_mut!(data) as LPARAM,
        );
        if dlgret == (IDOK) as INT_PTR && !data.Name.is_null() {
            if CreateProcessWithLogonW(
                data.Name,
                data.Domain,
                data.Password.as_mut_ptr(),
                data.LogonFlags,
                core::ptr::null_mut(),
                Cmd,
                CreationFlags,
                (Env).cast(),
                Dir,
                pSI,
                pPI,
            ) != 0
            {
                error = (ERROR_SUCCESS) as u32;
            } else {
                error = GetLastError();
            }
            match error {
                ERROR_LOGON_FAILURE
                | ERROR_NO_SUCH_USER
                | ERROR_INVALID_ACCOUNT_NAME
                | ERROR_ACCOUNT_DISABLED
                | ERROR_ACCOUNT_RESTRICTION
                | ERROR_INVALID_LOGON_HOURS
                | ERROR_PASSWORD_EXPIRED => {
                    SHELL_ErrorBox((hwnd as *mut c_void).cast(), error);
                    continue 'again;
                }
                _ => {}
            }
        } else if dlgret == (IDOK) as INT_PTR {
            if CreateProcessW(
                core::ptr::null_mut(),
                Cmd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                FALSE,
                CreationFlags,
                (Env).cast(),
                Dir,
                pSI,
                pPI,
            ) != 0
            {
                error = (ERROR_SUCCESS) as u32;
            } else {
                error = GetLastError();
            }
        } else if dlgret == (IDCANCEL) as INT_PTR {
            SetLastError(ERROR_CANCELLED);
            (*pPI).hProcess = core::ptr::null_mut();
            return S_FALSE;
        } else {
            (*pPI).hProcess = core::ptr::null_mut();
            return E_FAIL;
        }
        break;
    }
    SecureZeroMemory(
        core::ptr::addr_of_mut!(data) as *mut _,
        core::mem::size_of::<RUNASDLGDATA>(),
    );
    HRESULT_FROM_WIN32((error) as i32)
}
unsafe extern "C" fn SHELL_ExecuteW(
    lpCmd: *const u16,
    env: *mut u16,
    shWait: i32,
    psei: *const SHELLEXECUTEINFOW,
    psei_out: *mut SHELLEXECUTEINFOW,
) -> UINT_PTR {
    let mut startup: STARTUPINFOW = core::mem::zeroed();
    let mut info: PROCESS_INFORMATION = core::mem::zeroed();
    let mut retval: UINT_PTR = (SE_ERR_NOASSOC) as UINT_PTR;
    let mut gcdret: u32 = 0;
    let mut lasterror: u32 = 0;
    let mut curdir: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut dwCreationFlags: u32 = CREATE_UNICODE_ENVIRONMENT;
    let mut lpDirectory: PCWSTR = core::ptr::null();

    if !StrIsNullOrEmpty((*psei).lpDirectory) {
        let attr = GetFileAttributesW((*psei).lpDirectory);
        if attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            lpDirectory = (*psei).lpDirectory;
        }
    }

    if !lpDirectory.is_null() {
        gcdret = GetCurrentDirectoryW((MAX_PATH) as u32, curdir.as_mut_ptr());
        if gcdret != 0 {
            if SetCurrentDirectoryW(lpDirectory) == 0 {
                DbgPrint(
                    b"cannot set directory %s\n\0".as_ptr() as _,
                    (lpDirectory),
                );
            }
        }
    }

    ZeroMemory(
        &mut startup as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<STARTUPINFOW>(),
    );
    // C: `startup.cb = sizeof(STARTUPINFOW)`. `size_of_val(&STARTUPINFOW)` passed the TYPE where
    // a value belongs; `cb` is how CreateProcess knows which fields to read, so a wrong one is a
    // silently truncated startup record rather than a compile error anywhere else.
    startup.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    startup.dwFlags = STARTF_USESHOWWINDOW;
    startup.wShowWindow = ((*psei).nShow) as u16;
    if ((*psei).fMask & SEE_MASK_NO_CONSOLE) == 0 {
        dwCreationFlags |= CREATE_NEW_CONSOLE;
    }
    if ((*psei).fMask & SEE_MASK_FLAG_SEPVDM) != 0 {
        dwCreationFlags |= CREATE_SEPARATE_WOW_VDM;
    }
    startup.lpTitle = if ((*psei).fMask & (SEE_MASK_HASLINKNAME | SEE_MASK_HASTITLE)) != 0 {
        (*psei).lpClass as PWSTR
    } else {
        core::ptr::null_mut()
    };

    if ((*psei).fMask & SEE_MASK_HASLINKNAME) != 0 {
        startup.dwFlags |= STARTF_TITLEISLINKNAME;
    }

    if ((*psei).fMask & SEE_MASK_HOTKEY) != 0 {
        startup.hStdInput = UlongToHandle((*psei).dwHotKey);
        startup.dwFlags |= STARTF_USEHOTKEY;
    }

    if ((*psei).fMask & SEE_MASK_ICON) != 0 {
        startup.hStdOutput = *(*psei).__bindgen_anon_1.hIcon.as_ref();
        startup.dwFlags |= STARTF_SHELLPRIVATE;
    } else if (((*psei).fMask & SEE_MASK_HMONITOR) != 0) || !(*psei).hwnd.is_null() {
        if ((*psei).fMask & SEE_MASK_HMONITOR) != 0 {
            startup.hStdOutput = *(*psei).__bindgen_anon_1.hMonitor.as_ref();
        } else if !(*psei).hwnd.is_null() {
            startup.hStdOutput = (MonitorFromWindow((*psei).hwnd, MONITOR_DEFAULTTONEAREST)).cast();
        }
        if !startup.hStdOutput.is_null() {
            startup.dwFlags |= STARTF_SHELLPRIVATE;
        }
    }

    let createdProcess: i32;
    // The label spans exactly what C's `goto done` skips; the epilogue below is `done:`.
    'done: {
        if !(*psei).lpVerb.is_null()
            && StrCmpIW(CPP_COM_WSTR_878FA9FF9662.as_ptr(), (*psei).lpVerb) == 0
        {
            let hr = PromptAndRunProcessAs(
                (*psei).hwnd,
                lpCmd as PWSTR,
                dwCreationFlags,
                env,
                lpDirectory,
                &mut startup,
                &mut info,
            );
            createdProcess = if hr == S_OK { 1 } else { 0 };
            if hr == S_FALSE {
                retval = 33;
                break 'done;
            }
        } else {
            createdProcess = CreateProcessW(
                core::ptr::null_mut(),
                lpCmd as PWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                FALSE,
                dwCreationFlags,
                env as *mut core::ffi::c_void,
                lpDirectory,
                &mut startup,
                &mut info,
            );
        }

        if createdProcess != 0 {
            if shWait != 0 {
                if WaitForInputIdle(info.hProcess, 30000)
                    == WAIT_FAILED
                {
                    DbgPrint(
                        b"WaitForInputIdle failed: Error %d\n\0".as_ptr() as _,
                        GetLastError(),
                    );
                }
            }
            retval = 33;

            if ((*psei).fMask & SEE_MASK_NOCLOSEPROCESS) != 0 {
                (*psei_out).hProcess = info.hProcess;
            } else {
                CloseHandle(info.hProcess);
            }
            CloseHandle(info.hThread);
        } else {
            lasterror = GetLastError();
            retval = lasterror as UINT_PTR;
            if retval >= 32 {
                retval = (ERROR_BAD_FORMAT) as UINT_PTR;
            }
        }
    } // 'done

    {
        DbgPrint(b"returning %lu\n\0".as_ptr() as _, retval);
        (*psei_out).hInstApp = (retval as *mut core::ffi::c_void).cast();

        if gcdret != 0 {
            if SetCurrentDirectoryW(curdir.as_ptr()) == 0 {
                DbgPrint(
                    b"cannot return to directory %s\n\0".as_ptr() as _,
                    (curdir.as_ptr()),
                );
            }
            if lasterror != 0 {
                RestoreLastError(lasterror);
            }
        }
        retval
    }
}
unsafe extern "C" fn SHELL_BuildEnvW(path: PCWSTR) -> PWSTR {
    let mut new_env: CHeapPtr<WCHAR, CLocalAllocator> = CHeapPtr::new();
    let mut strings: PWSTR;
    let mut p: PWSTR;
    let mut p2: PWSTR;
    let mut total: i32 = (wcslen(path) + 1) as i32;
    let mut got_path: BOOL = FALSE;

    strings = GetEnvironmentStringsW();
    if strings.is_null() {
        return core::ptr::null_mut();
    }
    p = strings;
    while *p != 0 {
        let len: i32 = (wcslen(p) + 1) as i32;
        if _wcsnicmp(p, CPP_COM_WSTR_0D13EF6B270F.as_ptr(), 5) == 0 {
            got_path = TRUE;
        }
        total += len;
        p = p.offset(len as isize);
    }
    if got_path == 0 {
        total += 5;
    }
    total += 1;

    if new_env.Allocate(total as usize) == 0 {
        FreeEnvironmentStringsW(strings);
        return core::ptr::null_mut();
    }
    p = strings;
    p2 = *new_env;
    while *p != 0 {
        let len: i32 = (wcslen(p) + 1) as i32;
        memcpy(p2 as *mut core::ffi::c_void, p as *const core::ffi::c_void, (len as usize) * core::mem::size_of::<WCHAR>());
        if _wcsnicmp(p, CPP_COM_WSTR_0D13EF6B270F.as_ptr(), 5) == 0 {
            *p2.offset((len - 1) as isize) = 0x003Bu16;
            wcscpy(p2.offset(len as isize), path);
            p2 = p2.offset((wcslen(path) + 1) as isize);
        }
        p = p.offset(len as isize);
        p2 = p2.offset(len as isize);
    }
    if got_path == 0 {
        wcscpy(p2, CPP_COM_WSTR_0D13EF6B270F.as_ptr());
        wcscat(p2, path);
        p2 = p2.offset((wcslen(p2) + 1) as isize);
    }
    *p2 = 0;
    FreeEnvironmentStringsW(strings);
    return new_env.Detach();
}
unsafe extern "C" fn SHELL_TryAppPathW(szName: PCWSTR, lpResult: PWSTR, env: *mut PWSTR) -> u8 {
    let mut hkApp: HKEY = core::ptr::null_mut();
    let mut buffer: [u16; 1024] = [0; 1024];
    let mut len: u32;
    let mut dwType: u32 = 0;
    let mut res: i32;
    let mut found: u8 = (FALSE) as u8;

    if !env.is_null() {
        *env = core::ptr::null_mut();
    }
    wcscpy(buffer.as_mut_ptr(), CPP_COM_WSTR_8D831E206EA6.as_ptr());
    wcscat(buffer.as_mut_ptr(), szName);
    res = RegOpenKeyExW(HKEY_LOCAL_MACHINE, buffer.as_ptr(), 0, KEY_READ, &mut hkApp);
    if res != 0 {
        if PathAddExtensionW(buffer.as_mut_ptr(), CPP_COM_WSTR_7ABCE230ED71.as_ptr()) != 0 {
            res = RegOpenKeyExW(HKEY_LOCAL_MACHINE, buffer.as_ptr(), 0, KEY_READ, &mut hkApp);
        }
        if res != 0 {
            if !hkApp.is_null() {
                RegCloseKey(hkApp);
            }
            return found;
        }
    }

    len = MAX_PATH * core::mem::size_of::<u16>() as u32;
    res = SHRegQueryValueExW(hkApp, core::ptr::null(), core::ptr::null_mut(), &mut dwType, lpResult.cast(), &mut len);
    if res != (ERROR_SUCCESS) as i32 || dwType != REG_SZ {
        if !hkApp.is_null() {
            RegCloseKey(hkApp);
        }
        return found;
    }

    found = (TRUE) as u8;

    if !env.is_null() {
        len = core::mem::size_of_val(&buffer) as u32;
        res = SHRegQueryValueExW(hkApp, CPP_COM_WSTR_9A56E6BBE687.as_ptr(), core::ptr::null_mut(), &mut dwType, buffer.as_mut_ptr().cast(), &mut len);
        if res == (ERROR_SUCCESS) as i32 && dwType == REG_SZ && buffer[0] != 0 {
            *env = SHELL_BuildEnvW(buffer.as_ptr());
        }
    }

    if !hkApp.is_null() {
        RegCloseKey(hkApp);
    }
    found
}
unsafe extern "C" fn SHELL_FindExecutableByVerb(
    lp_verb: PCWSTR,
    key: PWSTR,
    classname: PWSTR,
    command: PWSTR,
    mut commandlen: i32,
) -> u32 {
    let mut verb: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];

    let mut hkey_class: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        classname,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hkey_class),
    ) != 0
    {
        return SE_ERR_NOASSOC;
    }

    if HCR_GetDefaultVerbW(
        hkey_class,
        lp_verb,
        verb.as_mut_ptr(),
        ARRAY_SIZE!(verb),
    ) == 0
    {
        return SE_ERR_NOASSOC;
    }

    RegCloseKey(hkey_class);

    wcscat(classname, CPP_COM_WSTR_6C1D0DF6BC60.as_ptr());
    wcscat(classname, verb.as_ptr());
    wcscat(classname, CPP_COM_WSTR_A16E78DA5D50.as_ptr());

    if RegQueryValueW(
        HKEY_CLASSES_ROOT,
        classname,
        command,
        core::ptr::addr_of_mut!(commandlen),
    ) == (ERROR_SUCCESS) as i32
    {
        commandlen /= 2;
        if !key.is_null() {
            wcscpy(key, classname);
        }
        *command.add(commandlen as usize) = 0;
        return 33;
    }

    SE_ERR_NOASSOC
}
unsafe extern "C" fn SHELL_FindExecutable(
    lpPath: PCWSTR,
    mut lpFile: PCWSTR,
    lpVerb: PCWSTR,
    lpResult: LPWSTR,
    resultLen: u32,
    key: PWSTR,
    env: *mut *mut u16,
    pidl: LPITEMIDLIST,
    args: PCWSTR,
) -> u32 {
    let mut extension: *mut u16 = core::ptr::null_mut();
    let mut classname: [u16; 256] = [0u16; 256];
    let mut classnamelen: i32 = core::mem::size_of_val(&classname) as i32;
    let mut command: [u16; 1024] = [0u16; 1024];
    let mut wBuffer: [u16; 256] = [0u16; 256];
    let mut retval: u32 = SE_ERR_NOASSOC;
    let mut tok: *mut u16;
    let mut xlpFile: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    let mut attribs: u32;
    let mut curdir: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    let mut search_paths: [PCWSTR; 3] = [core::ptr::null(); 3];
    search_paths[0] = core::ptr::null();

    if lpResult.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    core::ptr::write(xlpFile.as_mut_ptr().add(0), 0u16);
    core::ptr::write(lpResult.add(0), 0u16);
    if !key.is_null() {
        core::ptr::write(key, 0u16);
    }

    if lpFile.is_null() {
        return ERROR_FILE_NOT_FOUND;
    }

    if SHELL_TryAppPathW(lpFile, lpResult, env) != 0 {
        return 33;
    }

    GetCurrentDirectoryW((ARRAY_SIZE!(curdir)) as u32, curdir.as_mut_ptr());

    if !lpPath.is_null() && *lpPath != 0u16 {
        search_paths[0] = lpPath;
        search_paths[1] = curdir.as_ptr();
    } else {
        search_paths[0] = curdir.as_ptr();
    }

    lstrcpyW(xlpFile.as_mut_ptr(), lpFile);

    let resolve_result = PathResolveW(
        xlpFile.as_mut_ptr(),
        search_paths.as_mut_ptr(),
        PRF_TRYPROGRAMEXTENSIONS | PRF_VERIFYEXISTS,
    );
    let find_result = PathFindOnPathW(xlpFile.as_mut_ptr(), search_paths.as_mut_ptr());

    if resolve_result != 0 || find_result != 0 {
        lpFile = xlpFile.as_ptr();
        PathRemoveBlanksW(xlpFile.as_mut_ptr());

        let mut i: usize = wcslen(xlpFile.as_ptr()) as usize;
        while i > 0 && xlpFile[i - 1] == 0x2Eu16 {
            i -= 1;
            xlpFile[i] = 0u16;
        }

        lstrcpyW(lpResult, xlpFile.as_ptr());
    } else {
        core::ptr::write(xlpFile.as_mut_ptr().add(0), 0u16);
    }

    attribs = GetFileAttributesW(lpFile);
    if attribs != INVALID_FILE_ATTRIBUTES && (attribs & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        wcscpy(classname.as_mut_ptr(), CPP_COM_WSTR_EB4CE2024A43.as_ptr());
    } else {
        if xlpFile[0] == 0 {
            return SE_ERR_FNF;
        }
        let extension_ptr = wcsrchr(xlpFile.as_ptr(), 0x2Eu16);
        extension = extension_ptr;
        if extension.is_null() || *extension.add(1) == 0 {
            return SE_ERR_NOASSOC;
        }

        if GetProfileStringW(
            CPP_COM_WSTR_94D8FB2B7BE6.as_ptr(),
            CPP_COM_WSTR_F99F36C2F40B.as_ptr(),
            CPP_COM_WSTR_7DA531BC3C5E.as_ptr(),
            wBuffer.as_mut_ptr(),
            (ARRAY_SIZE!(wBuffer)) as u32,
        ) > 0
        {
            CharLowerW(wBuffer.as_mut_ptr());
            tok = wBuffer.as_mut_ptr();
            while *tok != 0u16 {
                let mut p: *mut u16 = tok;
                while *p != 0u16 && *p != 0x20u16 && *p != 0x9u16 {
                    p = p.add(1);
                }
                if *p != 0u16 {
                    *p = 0;
                    p = p.add(1);
                    while *p == 0x20u16 || *p == 0x9u16 {
                        p = p.add(1);
                    }
                }
                if _wcsicmp(tok, extension.add(1)) == 0 {
                    wcscpy(lpResult, xlpFile.as_ptr());
                    return 33;
                }
                tok = p;
            }
        }

        if RegQueryValueW(
            HKEY_CLASSES_ROOT,
            extension,
            classname.as_mut_ptr(),
            &mut classnamelen,
        ) == (ERROR_SUCCESS) as i32
        {
            classnamelen /= core::mem::size_of::<u16>() as i32;
            if classnamelen == ARRAY_SIZE!(classname) as i32 {
                classnamelen -= 1;
            }
            classname[classnamelen as usize] = 0u16;
        } else {
            core::ptr::write(classname.as_mut_ptr(), 0u16);
        }
    }

    if *classname.as_ptr() != 0u16 {
        retval = SHELL_FindExecutableByVerb(
            lpVerb,
            key,
            classname.as_mut_ptr(),
            command.as_mut_ptr(),
            core::mem::size_of_val(&command) as i32,
        );

        if retval > 32 {
            let mut finishedLen: u32 = 0;
            SHELL_ArgifyW(
                lpResult,
                resultLen,
                command.as_ptr(),
                xlpFile.as_ptr(),
                pidl,
                args,
                &mut finishedLen,
                lpPath,
            );
            if finishedLen > resultLen {
                ERR!("Argify buffer not large enough.. truncated\n");
            }
            if *lpResult == 0x22u16 {
                let mut p = lpResult;
                while *p.add(1) != 0x22u16 {
                    *p = *p.add(1);
                    p = p.add(1);
                }
                *p = 0u16;
            } else {
                let mut p = lpResult;
                while *p != 0x20u16 && *p != 0u16 {
                    p = p.add(1);
                }
                *p = 0u16;
            }
        }
    } else {
        extension = extension.add(1);
        if GetProfileStringW(
            CPP_COM_WSTR_4FB34BD59E1D.as_ptr(),
            extension,
            CPP_COM_WSTR_96A296D224F2.as_ptr(),
            command.as_mut_ptr(),
            (ARRAY_SIZE!(command)) as u32,
        ) > 0
        {
            if wcslen(command.as_ptr()) != 0 {
                wcscpy(lpResult, command.as_ptr());
                tok = wcschr(lpResult, 0x5Eu16);
                if !tok.is_null() {
                    *tok.add(0) = 0u16;
                    wcscat(lpResult, xlpFile.as_ptr());
                    tok = wcschr(command.as_ptr(), 0x5Eu16);
                    if !tok.is_null() && wcslen(tok) > 5 {
                        wcscat(lpResult, tok.add(5));
                    }
                }
                retval = 33;
            }
        }
    }

    retval
}

// One definition, spelled `extern "system"`: that IS stdcall on i386 and the C
// convention on 64-bit, and the island now spells the slot it goes into the same way
// (normalize_island_abi.py). The `#[cfg]`-duplicated stdcall/C twins this replaces were
// two bodies that had to be edited in lockstep forever.
unsafe extern "system" fn dde_cb(
    uType: UINT,
    uFmt: UINT,
    hConv: HCONV,
    hsz1: HSZ,
    hsz2: HSZ,
    hData: HDDEDATA,
    dwData1: ULONG_PTR,
    dwData2: ULONG_PTR,
) -> HDDEDATA {
    // TRACE("dde_cb: %04x, %04x, %p, %p, %p, %p, %08lx, %08lx\n", uType, uFmt, hConv, hsz1, hsz2, hData, dwData1, dwData2);
    (NULL).cast()
}

unsafe extern "C" fn dde_connect(
    key: PCWSTR,
    start: PCWSTR,
    ddeexec: PWSTR,
    lpFile: PCWSTR,
    env: PWSTR,
    szCommandline: PCWSTR,
    pidl: LPITEMIDLIST,
    execfunc: SHELL_ExecuteW32,
    psei: *const SHELLEXECUTEINFOW,
    psei_out: LPSHELLEXECUTEINFOW,
) -> u32 {
    let mut regkey: [u16; 256] = [0; 256];
    let endkey_offset = wcslen(key);
    let mut app: [u16; 256] = [0; 256];
    let mut topic: [u16; 256] = [0; 256];
    let mut ifexec: [u16; 256] = [0; 256];
    let mut static_res: [u16; 256] = [0; 256];
    let mut dynamic_res: CHeapPtr<u16> = CHeapPtr::new();
    let res: PWSTR;
    let mut applen: i32;
    let mut topiclen: i32;
    let mut ifexeclen: i32;
    let mut exec: PWSTR;
    let mut ddeInst: u32 = 0;
    let mut tid: u32 = 0;
    let mut resultLen: u32 = 0;
    let endkeyLen: usize;
    let hszApp: HSZ;
    let hszTopic: HSZ;
    let mut hConv: HCONV;
    let hDdeData: HDDEDATA;
    let mut ret: u32 = SE_ERR_NOASSOC;
    let unicode: u8 = if (GetVersion() & 0x80000000) == 0 { 1 } else { 0 };

    if strlenW(key) + 1 > ARRAY_SIZE!(regkey) as usize {
        return 2;
    }
    wcscpy(regkey.as_mut_ptr(), key);
    endkeyLen = (ARRAY_SIZE!(regkey) - (((((endkey_offset) as u32) as usize) as u32) as usize) as u32) as usize;
    if strlenW(CPP_COM_WSTR_FC9A94A80631.as_ptr()) + 1 > endkeyLen {
        return 2;
    }
    wcscpy(regkey.as_mut_ptr().add(endkey_offset), CPP_COM_WSTR_FC9A94A80631.as_ptr());
    applen = core::mem::size_of_val(&app) as i32;
    if RegQueryValueW(HKEY_CLASSES_ROOT, regkey.as_ptr(), app.as_mut_ptr() as *mut _, &mut applen) != (ERROR_SUCCESS) as i32 {
        let mut command: [u16; 1024] = [0; 1024];
        let mut fullpath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
        let mut ptr: PWSTR = core::ptr::null_mut();
        let mut ret_search: u32 = 0;

        if *start == '"' as u16 {
            if strlenW(start.add(1)) + 1 > ARRAY_SIZE!(command) as usize {
                return 2;
            }
            wcscpy(command.as_mut_ptr(), start.add(1));
            ptr = wcschr(command.as_mut_ptr(), '"' as u16);
            if !ptr.is_null() {
                *ptr = 0;
            }
            ret_search = SearchPathW(core::ptr::null_mut(), command.as_ptr(), CPP_COM_WSTR_7ABCE230ED71.as_ptr(), ARRAY_SIZE!(fullpath) as u32, fullpath.as_mut_ptr(), &mut ptr);
        } else {
            let mut p: PCWSTR = start;
            loop {
                let space: PWSTR = strchrW(p, ' ' as u16) as PWSTR;
                if space.is_null() {
                    break;
                }
                let idx = space.offset_from(start) as usize;
                memcpy(command.as_mut_ptr() as *mut _, start as *const _, idx * core::mem::size_of::<u16>());
                command[idx] = 0;
                ret_search = SearchPathW(core::ptr::null_mut(), command.as_ptr(), CPP_COM_WSTR_7ABCE230ED71.as_ptr(), ARRAY_SIZE!(fullpath) as u32, fullpath.as_mut_ptr(), &mut ptr);
                if ret_search != 0 {
                    break;
                }
                p = space.add(1);
            }
            if ret_search == 0 {
                ret_search = SearchPathW(core::ptr::null_mut(), start, CPP_COM_WSTR_7ABCE230ED71.as_ptr(), ARRAY_SIZE!(fullpath) as u32, fullpath.as_mut_ptr(), &mut ptr);
            }
        }

        if ret_search == 0 {
            return ERROR_ACCESS_DENIED;
        }
        if strlenW(ptr) + 1 > ARRAY_SIZE!(app) as usize {
            return 2;
        }
        wcscpy(app.as_mut_ptr(), ptr);

        ptr = app.as_mut_ptr().add(wcslen(app.as_ptr())).sub(3);
        if ptr > app.as_mut_ptr() && wcscmp(ptr, CPP_COM_WSTR_7B35AAC93EFF.as_ptr()) == 0 {
            *ptr = 0;
        }

        ptr = strrchrW(app.as_ptr(), '.' as u16) as PWSTR;
        assert!(!ptr.is_null());
        *ptr = 0;
    }

    if strlenW(CPP_COM_WSTR_1BC9EA5B0770.as_ptr()) + 1 > endkeyLen {
        return 2;
    }
    wcscpy(regkey.as_mut_ptr().add(endkey_offset), CPP_COM_WSTR_1BC9EA5B0770.as_ptr());
    topiclen = core::mem::size_of_val(&topic) as i32;
    if RegQueryValueW(HKEY_CLASSES_ROOT, regkey.as_ptr(), topic.as_mut_ptr() as *mut _, &mut topiclen) != (ERROR_SUCCESS) as i32 {
        wcscpy(topic.as_mut_ptr(), CPP_COM_WSTR_E9A72DDF42EB.as_ptr());
    }

    if unicode != 0 {
        if DdeInitializeW(&mut ddeInst, Some(dde_cb), APPCMD_CLIENTONLY, 0) != DMLERR_NO_ERROR {
            return 2;
        }
    } else {
        if DdeInitializeA(&mut ddeInst, Some(dde_cb), APPCMD_CLIENTONLY, 0) != DMLERR_NO_ERROR {
            return 2;
        }
    }

    hszApp = DdeCreateStringHandleW(ddeInst, app.as_ptr(), (CP_WINUNICODE) as i32);
    hszTopic = DdeCreateStringHandleW(ddeInst, topic.as_ptr(), (CP_WINUNICODE) as i32);

    hConv = DdeConnect(ddeInst, hszApp, hszTopic, core::ptr::null_mut());
    exec = ddeexec;
    if hConv.is_null() {
        ret = (execfunc(start, env, TRUE, psei, psei_out)) as u32;
        if ret <= 32 {
            DdeUninitialize(ddeInst);
            return ret;
        }
        if ddeexec.is_null() {
            return 42;
        }
        if wcscmp(ddeexec, CPP_COM_WSTR_96A296D224F2.as_ptr()) == 0 {
            return 42;
        }
        hConv = DdeConnect(ddeInst, hszApp, hszTopic, core::ptr::null_mut());
        if hConv.is_null() {
            DdeUninitialize(ddeInst);
            SetLastError(ERROR_DDE_FAIL);
            return 30;
        }
        if strlenW(CPP_COM_WSTR_8C31B1E57F5A.as_ptr()) + 1 > endkeyLen {
            return 2;
        }
        strcpyW(regkey.as_mut_ptr().add(endkey_offset), CPP_COM_WSTR_8C31B1E57F5A.as_ptr());
        ifexeclen = core::mem::size_of_val(&ifexec) as i32;
        if RegQueryValueW(HKEY_CLASSES_ROOT, regkey.as_ptr(), ifexec.as_mut_ptr() as *mut _, &mut ifexeclen) == (ERROR_SUCCESS) as i32 {
            exec = ifexec.as_mut_ptr();
        }
    }

    SHELL_ArgifyW(static_res.as_mut_ptr(), ARRAY_SIZE!(static_res) as u32, exec, lpFile, pidl, szCommandline, &mut resultLen, core::ptr::null_mut());
    if resultLen > ARRAY_SIZE!(static_res) as u32 {
        dynamic_res.Allocate((resultLen) as usize);
        res = dynamic_res.m_pData;
        SHELL_ArgifyW(dynamic_res.m_pData, resultLen, exec, lpFile, pidl, szCommandline, core::ptr::null_mut(), core::ptr::null_mut());
    } else {
        res = static_res.as_mut_ptr();
    }

    if unicode != 0 {
        hDdeData = DdeClientTransaction(res as *mut u8, ((strlenW(res) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32, hConv, core::ptr::null_mut(), 0, XTYP_EXECUTE, 30000, &mut tid);
    } else {
        let lenA: u32 = (WideCharToMultiByte(CP_ACP, 0, res, -(1 as i32), core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut())) as u32;
        let mut resA: CHeapPtr<i8> = CHeapPtr::new();
        resA.Allocate((lenA) as usize);
        WideCharToMultiByte(CP_ACP, 0, res, -(1 as i32), resA.m_pData, (lenA) as i32, core::ptr::null_mut(), core::ptr::null_mut());
        hDdeData = DdeClientTransaction(resA.m_pData as *mut u8, lenA, hConv, core::ptr::null_mut(), 0, XTYP_EXECUTE, 10000, &mut tid);
    }
    if !hDdeData.is_null() {
        DdeFreeDataHandle(hDdeData);
    } else {
        WARN!(CPP_COM_WSTR_WARN_DdeClientTransaction.as_ptr(), DdeGetLastError(ddeInst));
    }
    ret = 33;

    DdeDisconnect(hConv);

    DdeUninitialize(ddeInst);

    ret
}
unsafe extern "C" fn execute_from_key(
    key: PCWSTR,
    lpFile: PCWSTR,
    env: PWSTR,
    szCommandline: PCWSTR,
    executable_name: PCWSTR,
    execfunc: SHELL_ExecuteW32,
    psei: *mut _SHELLEXECUTEINFOW,
    psei_out: *mut _SHELLEXECUTEINFOW,
) -> UINT_PTR {
    let mut cmd = [0u16; 256];
    let mut param = [0u16; 1024];
    let mut ddeexec = [0u16; 256];
    let mut cmdlen: DWORD = core::mem::size_of_val(&cmd) as u32;
    let mut ddeexeclen: DWORD = core::mem::size_of_val(&ddeexec) as u32;
    let mut retval: UINT_PTR = (SE_ERR_NOASSOC) as UINT_PTR;
    let mut resultLen: DWORD = 0;

    cmd[0] = 0;
    param[0] = 0;

    if RegQueryValueW(HKEY_CLASSES_ROOT, key, cmd.as_mut_ptr(), &mut cmdlen as *mut DWORD as *mut i32) == (ERROR_SUCCESS) as i32 {
        cmdlen /= 2;
        if cmdlen >= ARRAY_SIZE!(cmd) {
            cmdlen = (ARRAY_SIZE!(cmd) - 1) as u32;
        }
        cmd[cmdlen as usize] = 0;
        SHELL_ArgifyW(
            param.as_mut_ptr(),
            (ARRAY_SIZE!(param)) as u32,
            cmd.as_ptr(),
            lpFile,
            (*psei).lpIDList as LPITEMIDLIST,
            szCommandline,
            &mut resultLen,
            if (*psei).lpDirectory != core::ptr::null() && *(*psei).lpDirectory != 0 {
                (*psei).lpDirectory
            } else {
                core::ptr::null()
            },
        );
        if resultLen > ARRAY_SIZE!(param) {
            DbgPrint(b"Argify buffer not large enough, truncating\n\0".as_ptr() as PCSTR);
        }
    }

    let tmp: LPWSTR =
        strstrW(key, CPP_COM_WSTR_0C71B3CB5580.as_ptr()) as LPWSTR;
    assert!(!tmp.is_null());
    wcscpy(tmp, CPP_COM_WSTR_C9E3E098911A.as_ptr());

    if RegQueryValueW(
        HKEY_CLASSES_ROOT,
        key,
        ddeexec.as_mut_ptr(),
        &mut ddeexeclen as *mut DWORD as *mut i32,
    ) == (ERROR_SUCCESS) as i32
    {
        if param[0] == 0 {
            strcpyW(param.as_mut_ptr(), executable_name);
        }
        retval = (dde_connect(
            key,
            param.as_mut_ptr(),
            ddeexec.as_mut_ptr(),
            lpFile,
            env,
            szCommandline,
            (*psei).lpIDList as LPITEMIDLIST,
            execfunc,
            psei,
            psei_out,
        )) as UINT_PTR;
    } else if param[0] != 0 {
        retval = execfunc(param.as_mut_ptr(), env, FALSE, psei, psei_out);
    } else {
        DbgPrint(
            b"Nothing appropriate found for %s\n\0".as_ptr() as *const i8,
            (key),
        );
    }

    retval
}

#[no_mangle]
pub unsafe extern "system" fn FindExecutableA(
    lpFile: *const u8,
    lpDirectory: *const u8,
    lpResult: *mut u8,
) -> isize {
    let mut retval: isize;
    let mut wFile: *mut u16 = core::ptr::null_mut();
    let mut wDirectory: *mut u16 = core::ptr::null_mut();
    let mut wResult: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if !lpFile.is_null() {
        __SHCloneStrAtoW(&mut wFile, (lpFile).cast());
    }
    if !lpDirectory.is_null() {
        __SHCloneStrAtoW(&mut wDirectory, (lpDirectory).cast());
    }

    retval = FindExecutableW(wFile, wDirectory, wResult.as_mut_ptr());

    WideCharToMultiByte(
        CP_ACP,
        0,
        wResult.as_mut_ptr(),
        -(1 as i32),
        (lpResult).cast(),
        (MAX_PATH) as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    SHFree((wFile).cast());
    SHFree((wDirectory).cast());

    retval
}

#[no_mangle]
pub unsafe extern "system" fn FindExecutableW(
    lpFile: *const u16,
    lpDirectory: *const u16,
    lpResult: *mut u16,
) -> isize {
    let mut retval: usize;
    let mut old_dir: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    let mut res: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    let mut cch: u32 = (MAX_PATH) as u32;
    let mut dirs: [*const u16; 2] = [core::ptr::null(); 2];

    *lpResult = UNICODE_NULL as u16;

    GetCurrentDirectoryW(
        MAX_PATH as u32,
        old_dir.as_mut_ptr(),
    );

    if !lpDirectory.is_null() && *lpDirectory != 0u16 {
        SetCurrentDirectoryW(lpDirectory);
        dirs[0 as usize] = lpDirectory;
    } else {
        dirs[0 as usize] = old_dir.as_ptr();
    }
    dirs[1 as usize] = core::ptr::null();

    if GetShortPathNameW(lpFile, res.as_mut_ptr(), MAX_PATH as u32) == 0 {
        StringCchCopyW(
            res.as_mut_ptr(),
            (MAX_PATH as u32) as usize,
            lpFile,
        );
    }

    if PathResolveW(
        res.as_mut_ptr(),
        dirs.as_mut_ptr(),
        (PRF_TRYPROGRAMEXTENSIONS | PRF_FIRSTDIRDEF) as u32,
    ) != 0 {
        if PathIsExeW(res.as_ptr()) != 0
            || AssocQueryStringW(
                ASSOCF_NONE as u32,
                ASSOCSTR_EXECUTABLE as u32,
                res.as_mut_ptr(),
                core::ptr::null_mut(),
                res.as_mut_ptr(),
                core::ptr::addr_of_mut!(cch),
            ) >= 0
        {
            StringCchCopyW(
                lpResult,
                (MAX_PATH as u32) as usize,
                res.as_ptr(),
            );
            retval = 42;
        } else {
            retval = (SE_ERR_NOASSOC) as usize;
        }
    } else {
        retval = (SE_ERR_FNF) as usize;
    }

    SetCurrentDirectoryW(old_dir.as_mut_ptr());
    retval as isize
}
unsafe extern "C" fn ShellExecute_GetClassKey(sei: *const SHELLEXECUTEINFOW) -> HKEY {
    if ((*sei).fMask & SEE_MASK_CLASSALL) == SEE_MASK_CLASSKEY {
        return (*sei).hkeyClass;
    }

    let mut hKey: HKEY = core::ptr::null_mut();
    if (*sei).fMask & SEE_MASK_CLASSNAME != 0 {
        RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            (*sei).lpClass,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hKey),
        );
        return hKey;
    }

    let ext: PCWSTR = PathFindExtensionW((*sei).lpFile);
    if !StrIsNullOrEmpty(ext) {
        if (HCR_GetProgIdKeyOfExtension(ext, core::ptr::addr_of_mut!(hKey), FALSE))
            >= 0
        {
            return hKey;
        }
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn shellex_get_dataobj(
    sei: *mut SHELLEXECUTEINFOW,
    dataObj: *mut *mut IDataObject,
) -> HRESULT {
    let mut allocatedPidl: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();

    if ((*sei).fMask & SEE_MASK_CLASSALL) != 0 {
        pidl = (*sei).lpIDList as LPITEMIDLIST;
    } else {
        let mut fullpath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
        let ret: i32;

        fullpath[0] = 0;
        ret = (GetFullPathNameW((*sei).lpFile, (MAX_PATH) as u32, fullpath.as_mut_ptr(), core::ptr::null_mut())) as i32;
        if ret == 0 {
            return HRESULT_FROM_WIN32((GetLastError()) as i32);
        }

        pidl = ILCreateFromPathW(fullpath.as_ptr());
        allocatedPidl = pidl as *mut ITEMIDLIST;
    }

    let result = SHELL_GetUIObjectOfAbsoluteItem(core::ptr::null_mut(), pidl, core::ptr::addr_of!(IID_IDataObject).cast(), (dataObj).cast());

    if !allocatedPidl.is_null() {
        CoTaskMemFree(allocatedPidl as *mut _);
    }

    result
}
unsafe extern "C" fn shellex_run_context_menu_default(
    obj: *mut IShellExtInit,
    sei: LPSHELLEXECUTEINFOW,
) -> HRESULT {
    let mut cm: *mut IContextMenu = core::ptr::null_mut();
    let mut ici: CMINVOKECOMMANDINFOEX = core::mem::zeroed();
    let mut info: MENUITEMINFOW = core::mem::zeroed();
    let mut string: [u16; 0x80] = [0; 0x80];
    let mut def: i32 = -1;
    let mut hmenu: HMENU = core::ptr::null_mut();
    let mut r: HRESULT;

    r = ((*(*obj).lpVtbl).QueryInterface)(
        obj,
        core::ptr::addr_of!(IID_IContextMenu).cast(),
        core::ptr::addr_of_mut!(cm).cast(),
    );
    if r < 0 {
        return r;
    }

    hmenu = CreateMenu();
    if hmenu.is_null() {
        return 'end: {
            if !hmenu.is_null() {
                DestroyMenu(hmenu);
            }
            break 'end r;
        };
    }

    r = ((*(*cm).lpVtbl).QueryContextMenu)(
        cm,
        hmenu.cast(),
        0,
        0x20,
        0x7fff,
        CMF_DEFAULTONLY,
    );
    if r < 0 {
        return 'end: {
            if !hmenu.is_null() {
                DestroyMenu(hmenu);
            }
            break 'end r;
        };
    }

    let n = GetMenuItemCount(hmenu);
    for i in 0..n {
        memset(&mut info as *mut _ as *mut _, 0, core::mem::size_of_val(&info));
        info.cbSize = core::mem::size_of_val(&info) as u32;
        info.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_STATE | MIIM_DATA | MIIM_ID;
        info.dwTypeData = string.as_mut_ptr();
        info.cch = string.len() as u32;
        string[0] = 0;
        GetMenuItemInfoW(hmenu, (i) as u32, TRUE, &mut info);

        if ((*sei).lpVerb.is_null() && (info.fState & MFS_DEFAULT) != 0)
            || (!(*sei).lpVerb.is_null() && lstrcmpiW((*sei).lpVerb, string.as_ptr()) == 0)
        {
            def = i;
            break;
        }
    }

    r = E_FAIL;
    if def == -1 {
        return 'end: {
            if !hmenu.is_null() {
                DestroyMenu(hmenu);
            }
            break 'end r;
        };
    }

    memset(&mut ici as *mut _ as *mut _, 0, core::mem::size_of_val(&ici));
    ici.cbSize = core::mem::size_of_val(&ici) as u32;
    ici.fMask = ((*sei).fMask & SEE_CMIC_COMMON_BASICFLAGS) | CMIC_MASK_UNICODE;
    ici.nShow = (*sei).nShow;
    ici.lpVerb = MAKEINTRESOURCEA((def) as u32);
    ici.hwnd = (*sei).hwnd;
    ici.lpParametersW = (*sei).lpParameters;

    r = ((*(*cm).lpVtbl).InvokeCommand)(cm, core::ptr::addr_of!(ici) as *mut c_void);

    if !hmenu.is_null() {
        DestroyMenu(hmenu);
    }
    r
}
unsafe extern "C" fn shellex_load_object_and_run(
    hkey: HKEY,
    guid: *const GUID,
    sei: LPSHELLEXECUTEINFOW,
) -> HRESULT {
    let mut obj: *mut IShellExtInit = core::ptr::null_mut();
    let mut dataobj: *mut IDataObject = core::ptr::null_mut();
    let mut ows: *mut IObjectWithSite = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = CoInitialize(core::ptr::null_mut());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = CoCreateInstance(
        (guid as *const GUID).cast(),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_IShellExtInit).cast(),
        &mut obj as *mut *mut IShellExtInit as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        CoUninitialize();
        return hr;
    }

    hr = shellex_get_dataobj(sei, &mut dataobj);
    if FAILED_UNEXPECTEDLY!(hr) {
        ((*(*obj).lpVtbl).Release)(obj);
        CoUninitialize();
        return hr;
    }

    hr = ((*(*obj).lpVtbl).Initialize)(obj, core::ptr::null_mut(), dataobj.cast(), hkey);
    if FAILED_UNEXPECTEDLY!(hr) {
        ((*(*dataobj).lpVtbl).Release)(dataobj);
        ((*(*obj).lpVtbl).Release)(obj);
        CoUninitialize();
        return hr;
    }

    hr = ((*(*obj).lpVtbl).QueryInterface)(
        obj,
        &IID_IObjectWithSite as *const GUID,
        &mut ows as *mut *mut IObjectWithSite as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        ((*(*dataobj).lpVtbl).Release)(dataobj);
        ((*(*obj).lpVtbl).Release)(obj);
        CoUninitialize();
        return hr;
    }

    ((*(*ows).lpVtbl).SetSite)(ows, core::ptr::null_mut());

    hr = shellex_run_context_menu_default(obj, sei);

    ((*(*ows).lpVtbl).Release)(ows);
    ((*(*dataobj).lpVtbl).Release)(dataobj);
    ((*(*obj).lpVtbl).Release)(obj);
    CoUninitialize();

    hr
}
unsafe extern "C" fn shellex_get_contextmenu(sei: *mut SHELLEXECUTEINFOW, cm: *mut *mut IContextMenu) -> HRESULT {
    let mut allocatedPidl: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();

    if !(*sei).lpIDList.is_null() {
        pidl = (*sei).lpIDList as LPITEMIDLIST;
    } else {
        let mut sfga: SFGAOF = 0;
        let mut hr: HRESULT = SHParseDisplayName((*sei).lpFile, core::ptr::null_mut(), &mut allocatedPidl, SFGAO_STORAGECAPMASK, &mut sfga);
        if hr < 0 {
            let mut Buffer: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
            let retval: u32 = SHELL_FindExecutable((*sei).lpDirectory, (*sei).lpFile, (*sei).lpVerb, Buffer.as_mut_ptr(), _countof!(Buffer), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), (*sei).lpParameters);
            if retval <= 32 {
                return HRESULT_FROM_WIN32((retval) as i32);
            }

            hr = SHParseDisplayName(Buffer.as_ptr(), core::ptr::null_mut(), &mut allocatedPidl, SFGAO_STORAGECAPMASK, &mut sfga);
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }
        }

        pidl = allocatedPidl;
    }

    let mut shf: *mut IShellFolder = core::ptr::null_mut();
    let mut pidllast: LPCITEMIDLIST = core::ptr::null();
    let hr: HRESULT = SHBindToParent(pidl, core::ptr::addr_of!(IID_IShellFolder).cast(), core::ptr::addr_of_mut!(shf).cast(), &mut pidllast);
    if hr < 0 {
        return hr;
    }

    let result = ((*(*shf).lpVtbl).GetUIObjectOf)(shf, core::ptr::null_mut(), 1, (&pidllast as *const LPCITEMIDLIST).cast(), core::ptr::addr_of!(IID_IContextMenu).cast(), core::ptr::null_mut(), cm.cast());

    if !shf.is_null() {
        ((*(*shf).lpVtbl).Release)(shf);
    }

    if !allocatedPidl.is_null() {
        CoTaskMemFree(allocatedPidl as *mut c_void);
    }

    result
}
unsafe extern "C" fn ShellExecute_ContextMenuVerb(sei: *mut SHELLEXECUTEINFOW) -> HRESULT {
    let coInit = CCoInit::new();

    if FAILED_UNEXPECTEDLY!(coInit.hr) {
        return coInit.hr;
    }

    let mut cm: *mut IContextMenu = core::ptr::null_mut();
    let mut hr = shellex_get_contextmenu(sei, &mut cm);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut verb: *mut i8 = core::ptr::null_mut();
    let mut parameters: *mut i8 = core::ptr::null_mut();
    let mut dir: *mut i8 = core::ptr::null_mut();
    __SHCloneStrWtoA(&mut verb, (*sei).lpVerb);
    __SHCloneStrWtoA(&mut parameters, (*sei).lpParameters);
    __SHCloneStrWtoA(&mut dir, (*sei).lpDirectory);

    let fDefault = StrIsNullOrEmpty((*sei).lpVerb);
    let mut ici: CMINVOKECOMMANDINFOEX = core::mem::zeroed();
    ici.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32;
    ici.fMask = SeeFlagsToCmicFlags((*sei).fMask) | CMIC_MASK_UNICODE;
    ici.nShow = (*sei).nShow;
    if !fDefault {
        ici.lpVerb = verb;
        ici.lpVerbW = (*sei).lpVerb;
    }
    ici.hwnd = (*sei).hwnd;
    ici.lpParameters = parameters;
    ici.lpParametersW = (*sei).lpParameters;
    ici.lpDirectory = dir;
    ici.lpDirectoryW = (*sei).lpDirectory;
    ici.dwHotKey = (*sei).dwHotKey;
    ici.hIcon = *(*sei).__bindgen_anon_1.hIcon.as_ref();
    if (ici.fMask & (CMIC_MASK_HASLINKNAME | CMIC_MASK_HASTITLE)) != 0 {
        ici.lpTitleW = (*sei).lpClass;
    }

    let idFirst = 1;
    let idLast = 0x7fff;
    let hMenu = CreatePopupMenu();
    hr = ((*(*cm).lpVtbl).QueryContextMenu)(cm, hMenu.cast(), 0, idFirst, idLast, if fDefault { CMF_DEFAULTONLY } else { 0 });
    if !FAILED_UNEXPECTEDLY!(hr) {
        if fDefault {
            let mut uDefault = GetMenuDefaultItem(hMenu, (FALSE) as u32, 0);
            uDefault = if uDefault != u32::MAX { uDefault - (idFirst as i32) as u32 } else { 0 };
            ici.lpVerb = MAKEINTRESOURCEA(uDefault as u32);
            ici.lpVerbW = MAKEINTRESOURCEW(uDefault as u32);
        }

        hr = ((*(*cm).lpVtbl).InvokeCommand)(cm, core::ptr::addr_of!(ici) as *mut c_void);
        if !FAILED_UNEXPECTEDLY!(hr) {
            hr = S_OK;
        }
    }

    DestroyMenu(hMenu);

    CoTaskMemFree(verb as *mut c_void);
    CoTaskMemFree(parameters as *mut c_void);
    CoTaskMemFree(dir as *mut c_void);
    ((*(*cm).lpVtbl).Release)(cm);

    hr
}
unsafe extern "C" fn ShellExecute_FromContextMenuHandlers(sei: *mut SHELLEXECUTEINFOW) -> i32 {
    let mut hkeycm: HKEY = core::ptr::null_mut();
    let mut szguid: [u16; 39] = [0u16; 39];
    let mut hr: i32;
    let mut guid: GUID = core::mem::zeroed();
    let mut i: u32;
    let mut r: i32;

    let hkey = ShellExecute_GetClassKey(sei);
    if hkey.is_null() {
        return (ERROR_FUNCTION_FAILED) as i32;
    }

    r = RegOpenKeyW(hkey, CPP_COM_WSTR_87660BB31B23.as_ptr(), core::ptr::addr_of_mut!(hkeycm));
    if r == (ERROR_SUCCESS) as i32 {
        i = 0;
        while (1 as i32) != 0 {
            r = RegEnumKeyW(hkeycm, i, szguid.as_mut_ptr(), (ARRAY_SIZE!(szguid)) as u32);
            i += 1;
            if r != (ERROR_SUCCESS) as i32 {
                break;
            }
            hr = CLSIDFromString(szguid.as_ptr(), (&mut guid as *mut GUID).cast());
            if hr >= 0 {
                hr = shellex_load_object_and_run(hkey, &guid, sei);
                if hr >= 0 {
                    break;
                }
            }
        }
        RegCloseKey(hkeycm);
    }

    if hkey != (*sei).hkeyClass {
        RegCloseKey(hkey);
    }
    r
}
unsafe extern "C" fn SHELL_execute_class(
    wszApplicationName: PCWSTR,
    psei: LPSHELLEXECUTEINFOW,
    psei_out: LPSHELLEXECUTEINFOW,
    execfunc: SHELL_ExecuteW32,
) -> UINT_PTR {
    let mut execCmd: [u16; 1024] = [0u16; 1024];
    let mut classname: [u16; 1024] = [0u16; 1024];
    let cmask: u32 = (*psei).fMask & SEE_MASK_CLASSALL;
    let mut resultLen: u32 = 0;
    let done: u8;
    let mut rslt: UINT_PTR;

    if cmask != SEE_MASK_CLASSNAME {
        let mut wcmd: [u16; 1024] = [0u16; 1024];
        HCR_GetExecuteCommandW(
            if cmask == SEE_MASK_CLASSKEY { (*psei).hkeyClass } else { core::ptr::null_mut() },
            if cmask == SEE_MASK_CLASSNAME { (*psei).lpClass } else { core::ptr::null() },
            (*psei).lpVerb,
            execCmd.as_mut_ptr(),
            core::mem::size_of_val(&execCmd) as u32,
        );
        // FIXME: get the extension of lpFile, check if it fits to the lpClass

        wcmd[0 as usize] = 0u16;
        done = SHELL_ArgifyW(
            wcmd.as_mut_ptr(),
            (ARRAY_SIZE!(wcmd)) as u32,
            execCmd.as_mut_ptr(),
            wszApplicationName,
            (*psei).lpIDList as LPITEMIDLIST,
            (*psei).lpParameters,
            &mut resultLen,
            if !(*psei).lpDirectory.is_null() && *(*psei).lpDirectory != 0 { (*psei).lpDirectory } else { core::ptr::null() },
        );
        if done == 0 && *wszApplicationName.add(0 as usize) != 0 {
        }
        if resultLen > ARRAY_SIZE!(wcmd) {
            DbgPrint((b"Argify buffer not large enough... truncating\n\0".as_ptr()).cast());
        }
        return execfunc(
            wcmd.as_mut_ptr(),
            core::ptr::null_mut(),
            FALSE,
            psei,
            psei_out,
        );
    }

    strcpyW(classname.as_mut_ptr(), (*psei).lpClass);
    rslt = (SHELL_FindExecutableByVerb(
        (*psei).lpVerb,
        core::ptr::null_mut(),
        classname.as_mut_ptr(),
        execCmd.as_mut_ptr(),
        (core::mem::size_of_val(&execCmd) as u32) as i32,
    )) as UINT_PTR;

    if 33 > rslt {
        return rslt;
    }
    rslt = SHELL_quote_and_execute(
        execCmd.as_mut_ptr(),
        CPP_COM_WSTR_96A296D224F2.as_ptr(),
        classname.as_mut_ptr(),
        wszApplicationName,
        core::ptr::null_mut(),
        psei,
        psei_out,
        execfunc,
    );
    rslt
}
unsafe extern "C" fn SHELL_translate_idlist(
    sei: *mut SHELLEXECUTEINFOW,
    wszParameters: PWSTR,
    parametersLen: u32,
    wszApplicationName: PWSTR,
    dwApplicationNameLen: u32,
) -> u8 {
    let mut buffer: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut appKnownSingular: u8 = (FALSE) as u8;

    if SHELL_GetPathFromIDListForExecuteW(
        (*sei).lpIDList as *const ITEMIDLIST,
        buffer.as_mut_ptr(),
        (ARRAY_SIZE!(buffer)) as u32,
    ) >= 0
    {
        if buffer[0] == ':' as u16 && buffer[1] == ':' as u16 {
            if strlenW(buffer.as_ptr()) + 1 > (parametersLen) as usize {
                ERR!(
                    "parameters len exceeds buffer size (%i > %i), truncating\n",
                    lstrlenW(buffer.as_ptr()) + 1,
                    parametersLen,
                );
            }
            lstrcpynW(wszParameters, buffer.as_ptr(), (parametersLen) as i32);
            if strlenW(CPP_COM_WSTR_AB68B18B313D.as_ptr()) > (dwApplicationNameLen) as usize {
                ERR!(
                    "application len exceeds buffer size (%i), truncating\n",
                    dwApplicationNameLen,
                );
            }
            lstrcpynW(
                wszApplicationName,
                CPP_COM_WSTR_AB68B18B313D.as_ptr(),
                (dwApplicationNameLen) as i32,
            );
            appKnownSingular = (TRUE) as u8;
            (*sei).fMask &= !SEE_MASK_INVOKEIDLIST;
        } else {
            // C: `WCHAR target[max(MAX_PATH, _countof(buffer))]`. `buffer` is itself
            // `[WCHAR; MAX_PATH]` (declared at the top of this function), so both arms of the
            // `max` are MAX_PATH and the size below is the C's size. The translator emitted the
            // `max` as a dead local; dropped rather than left, since it computed nothing.
            let mut target: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
            let attribs: u32;
            let mut resultLen: u32 = 0;

            strcpyW(target.as_mut_ptr(), buffer.as_mut_ptr());
            attribs = GetFileAttributesW(buffer.as_ptr());
            if attribs != INVALID_FILE_ATTRIBUTES
                && (attribs & FILE_ATTRIBUTE_DIRECTORY) != 0
                && HCR_GetExecuteCommandW(
                    core::ptr::null_mut(),
                    CPP_COM_WSTR_EB4CE2024A43.as_ptr(),
                    (*sei).lpVerb,
                    buffer.as_mut_ptr(),
                    core::mem::size_of_val(&buffer) as u32,
                ) != 0
            {
                SHELL_ArgifyW(
                    wszApplicationName,
                    dwApplicationNameLen,
                    buffer.as_ptr(),
                    target.as_ptr(),
                    (*sei).lpIDList as LPITEMIDLIST,
                    core::ptr::null_mut::<u16>(),
                    &mut resultLen,
                    if !StrIsNullOrEmpty((*sei).lpDirectory) {
                        (*sei).lpDirectory
                    } else {
                        core::ptr::null_mut()
                    },
                );
                if resultLen > dwApplicationNameLen {
                    ERR!(
                        "Argify buffer not large enough... truncating\n",
                    );
                }
                appKnownSingular = (FALSE) as u8;

                let params: *const u16 = PathGetArgsW(wszApplicationName);
                lstrcpynW(wszParameters, params, (parametersLen) as i32);
                PathRemoveArgsW(wszApplicationName);
                PathUnquoteSpacesW(wszApplicationName);
                appKnownSingular = (TRUE) as u8;
            }
            (*sei).fMask &= !SEE_MASK_INVOKEIDLIST;
        }
    }

    appKnownSingular
}
unsafe extern "C" fn SHELL_InvokePidl(
    sei: LPSHELLEXECUTEINFOW,
    pidl: LPCITEMIDLIST,
) -> BOOL {
    let mut psfFolder: *mut IShellFolder = core::ptr::null_mut();
    let mut pidlLast: LPCITEMIDLIST = core::ptr::null_mut();
    let hr = SHBindToParent(pidl, core::ptr::addr_of!(IID_IShellFolder).cast(), core::ptr::addr_of_mut!(psfFolder).cast(), &mut pidlLast);
    if FAILED_UNEXPECTEDLY!(hr) {
        return FALSE;
    }

    let mut pCM: *mut IContextMenu = core::ptr::null_mut();
    let hr = ((*(*psfFolder).lpVtbl).GetUIObjectOf)(psfFolder, core::ptr::null_mut(), 1, (&pidlLast as *const LPCITEMIDLIST).cast(), core::ptr::addr_of!(IID_IContextMenu).cast(), core::ptr::null_mut(), core::ptr::addr_of_mut!(pCM).cast());
    if FAILED_UNEXPECTEDLY!(hr) {
        ((*(*psfFolder).lpVtbl).Release)(psfFolder);
        return FALSE;
    }

    let mut ici: CMINVOKECOMMANDINFO = core::mem::zeroed();
    ici.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFO>() as u32;
    ici.fMask = ((*sei).fMask & SEE_CMIC_COMMON_BASICFLAGS) & !CMIC_MASK_UNICODE;
    ici.nShow = (*sei).nShow;
    ici.hwnd = (*sei).hwnd;
    let mut szVerb: [i8; (VERBKEY_CCHMAX) as usize] = [0; (VERBKEY_CCHMAX) as usize];
    if !(*sei).lpVerb.is_null() && *(*sei).lpVerb.offset(0 as isize) != 0 {
        WideCharToMultiByte(CP_ACP, 0, (*sei).lpVerb, -1, szVerb.as_mut_ptr() as *mut i8, (_countof!(szVerb)) as i32, core::ptr::null_mut(), core::ptr::null_mut());
        szVerb[_countof!(szVerb) as usize - 1] = ANSI_NULL as i8;
        ici.lpVerb = szVerb.as_ptr();
    } else {
        let hMenu = CreatePopupMenu();
        let idCmdFirst: i32 = 1;
        let idCmdLast: i32 = 0x7FFF;
        let hr = ((*(*pCM).lpVtbl).QueryContextMenu)(pCM, hMenu.cast(), 0, idCmdFirst as u32, idCmdLast as u32, CMF_DEFAULTONLY);
        if FAILED_UNEXPECTEDLY!(hr) {
            DestroyMenu(hMenu);
            ((*(*pCM).lpVtbl).Release)(pCM);
            ((*(*psfFolder).lpVtbl).Release)(psfFolder);
            return FALSE;
        }

        let mut nDefaultID = GetMenuDefaultItem(hMenu, (FALSE) as u32, 0);
        DestroyMenu(hMenu);
        if nDefaultID == u32::MAX {
            nDefaultID = (idCmdFirst) as u32;
        }

        ici.lpVerb = MAKEINTRESOURCEA(((nDefaultID - (idCmdFirst) as u32) as u16) as u32);
    }
    let hr = ((*(*pCM).lpVtbl).InvokeCommand)(pCM, core::ptr::addr_of!(ici) as *mut c_void);

    let result = if FAILED_UNEXPECTEDLY!(hr) { 0u8 } else { 1u8 };
    ((*(*pCM).lpVtbl).Release)(pCM);
    ((*(*psfFolder).lpVtbl).Release)(psfFolder);
    (result) as i32
}
unsafe extern "C" fn SHELL_quote_and_execute(
    wcmd: PCWSTR,
    wszParameters: PCWSTR,
    wszKeyname: PCWSTR,
    wszApplicationName: PCWSTR,
    env: PWSTR,
    psei: *mut _SHELLEXECUTEINFOW,
    psei_out: *mut _SHELLEXECUTEINFOW,
    execfunc: SHELL_ExecuteW32,
) -> UINT_PTR {
    let retval: UINT_PTR;
    let mut len: u32;
    let mut ququoted_cmd: *mut u16;

    len = (2 + lstrlenW(wcmd) + 1) as u32;
    if *wszParameters.offset(0 as isize) != 0 {
        len += (1 + lstrlenW(wszParameters)) as u32;
    }
    ququoted_cmd = LocalAlloc(LMEM_FIXED, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
    strcpyW(ququoted_cmd, CPP_COM_WSTR_1B9334FEECE6.as_ptr());
    strcatW(ququoted_cmd, wcmd);
    strcatW(ququoted_cmd, CPP_COM_WSTR_1B9334FEECE6.as_ptr());
    if *wszParameters.offset(0 as isize) != 0 {
        strcatW(ququoted_cmd, CPP_COM_WSTR_8D71B3FAAB82.as_ptr());
        strcatW(ququoted_cmd, wszParameters);
    }

    if *wszKeyname != 0 {
        retval = execute_from_key(
            wszKeyname,
            wszApplicationName,
            env,
            (*psei).lpParameters,
            wcmd,
            execfunc,
            psei,
            psei_out,
        );
    } else {
        retval = execfunc(ququoted_cmd, env, FALSE, psei, psei_out);
    }

    LocalFree(ququoted_cmd as *mut core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn SHELL_execute_url(
    lpFile: PCWSTR,
    wcmd: PCWSTR,
    psei: *mut _SHELLEXECUTEINFOW,
    psei_out: *mut _SHELLEXECUTEINFOW,
    execfunc: SHELL_ExecuteW32,
) -> UINT_PTR {
    let retval: UINT_PTR;
    let lpstrRes: PCWSTR;
    let iSize: i32;
    let mut len: DWORD;
    let mut lpstrProtocol: *mut u16 = core::ptr::null_mut();

    lpstrRes = strchrW(lpFile, ':' as u16);
    if !lpstrRes.is_null() {
        iSize = lpstrRes.offset_from(lpFile) as i32;
    } else {
        iSize = (strlenW(lpFile)) as i32;
    }

    len = iSize as DWORD
        + (lstrlenW(CPP_COM_WSTR_6C1D0DF6BC60.as_ptr())) as u32
        + (lstrlenW(CPP_COM_WSTR_A16E78DA5D50.as_ptr())) as u32
        + 1;
    if !(*psei).lpVerb.is_null() && *((*psei).lpVerb) != 0u16 {
        len += (lstrlenW((*psei).lpVerb)) as u32;
    } else {
        len += (lstrlenW(CPP_COM_WSTR_82DB442E7A60.as_ptr())) as u32;
    }

    lpstrProtocol = LocalAlloc(0x40, ((len as usize) * 2) as SIZE_T) as *mut u16;
    memcpy(
        lpstrProtocol as *mut core::ffi::c_void,
        lpFile as *const core::ffi::c_void,
        (iSize * 2) as usize,
    );
    *lpstrProtocol.offset(iSize as isize) = 0u16;
    strcatW(lpstrProtocol, CPP_COM_WSTR_6C1D0DF6BC60.as_ptr());
    strcatW(
        lpstrProtocol,
        if !(*psei).lpVerb.is_null() && *((*psei).lpVerb) != 0u16 {
            (*psei).lpVerb
        } else {
            CPP_COM_WSTR_82DB442E7A60.as_ptr()
        },
    );
    strcatW(lpstrProtocol, CPP_COM_WSTR_A16E78DA5D50.as_ptr());

    retval = execute_from_key(
        lpstrProtocol,
        lpFile,
        core::ptr::null_mut(),
        (*psei).lpParameters,
        wcmd,
        execfunc,
        psei,
        psei_out,
    );

    LocalFree(lpstrProtocol as *mut _);
    retval
}
unsafe extern "C" fn do_error_dialog(retval: UINT_PTR, hwnd: HWND, filename: *const u16) {
    let mut msg: [u16; 2048] = [0u16; 2048];
    let mut msgArguments: [usize; 3] = [0usize; 3];
    msgArguments[0] = filename as usize;
    msgArguments[1] = 0;
    msgArguments[2] = 0;
    let error_code: u32 = GetLastError();
    if retval == (SE_ERR_NOASSOC) as UINT_PTR {
        LoadStringW(
            shell32_hInstance,
            IDS_SHLEXEC_NOASSOC,
            msg.as_mut_ptr(),
            (ARRAY_SIZE!(msg)) as i32,
        );
    } else {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY,
            core::ptr::null_mut(),
            error_code,
            LANG_USER_DEFAULT,
            msg.as_mut_ptr(),
            (ARRAY_SIZE!(msg)) as u32,
            (msgArguments.as_mut_ptr().cast::<core::ffi::c_void>()).cast(),
        );
    }
    MessageBoxW(
        hwnd,
        msg.as_ptr(),
        core::ptr::null_mut(),
        MB_ICONERROR,
    );
    SetLastError(error_code);
}
unsafe extern "C" fn expand_environment(str: PCWSTR) -> PWSTR {
    let mut buf: PWSTR = core::ptr::null_mut();
    let mut len: u32 = ExpandEnvironmentStringsW(str, core::ptr::null_mut(), 0);
    if len == 0 {
        return core::ptr::null_mut();
    }
    buf = LocalAlloc(0, ((len as usize).wrapping_mul(core::mem::size_of::<u16>())) as SIZE_T) as PWSTR;
    if buf.is_null() {
        return core::ptr::null_mut();
    }
    len = ExpandEnvironmentStringsW(str, buf, len);
    if len == 0 {
        LocalFree(buf as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    buf
}
unsafe extern "C" fn SHELL_execute(sei: LPSHELLEXECUTEINFOW, execfunc: SHELL_ExecuteW32) -> BOOL {
    let unsupportedFlags: u32 = SEE_MASK_CONNECTNETDRV | SEE_MASK_FLAG_DDEWAIT | SEE_MASK_ASYNCOK;

    let mut len: u32;
    let mut retval: UINT_PTR = (SE_ERR_NOASSOC) as UINT_PTR;
    let mut appKnownSingular: u8 = (FALSE) as u8;

    (*sei).hProcess = core::ptr::null_mut();
    let mut sei_tmp: SHELLEXECUTEINFOW = core::ptr::read(sei);

    let hr: i32 = TryShellExecuteHooks(sei);
    if hr != S_FALSE {
        let err: i32 = Win32ErrFromHInst((*sei).hInstApp);
        if err <= 0 {
            (*sei).hInstApp = (UlongToHandle(42)).cast();
            return TRUE;
        }
        SetLastError((err) as u32);
        return FALSE;
    }

    let mut wszApplicationName: CHeapPtr<u16, CLocalAllocator> = CHeapPtr::new();
    let mut dwApplicationNameLen: u32 = (MAX_PATH + 2) as u32;
    if sei_tmp.lpFile.is_null() {
        wszApplicationName.Allocate((dwApplicationNameLen) as usize);
        *wszApplicationName.m_pData = 0x0000u16;
    } else if *sei_tmp.lpFile == 0x0022u16 && {
        len = (strlenW(sei_tmp.lpFile)) as u32;
        *sei_tmp.lpFile.offset((len - 1) as isize) == 0x0022u16
    } {
        if len - 1 >= dwApplicationNameLen {
            dwApplicationNameLen = len;
        }

        wszApplicationName.Allocate((dwApplicationNameLen) as usize);
        memcpy(
            wszApplicationName.m_pData as *mut _,
            sei_tmp.lpFile.offset(1 as isize) as *const _,
            (len * core::mem::size_of::<u16>() as u32) as usize,
        );

        if len > 2 {
            *wszApplicationName.m_pData.offset((len - 2) as isize) = 0x0000u16;
        }
        appKnownSingular = (TRUE) as u8;
    } else {
        let l: u32 = (strlenW(sei_tmp.lpFile) + 1) as u32;
        if l > dwApplicationNameLen {
            dwApplicationNameLen = l + 1;
        }
        wszApplicationName.Allocate((dwApplicationNameLen) as usize);
        memcpy(
            wszApplicationName.m_pData as *mut _,
            sei_tmp.lpFile as *const _,
            (l * core::mem::size_of::<u16>() as u32) as usize,
        );

        if *wszApplicationName.m_pData.offset(2 as isize) == 0 &&
           *wszApplicationName.m_pData.offset(1 as isize) == 0x003Au16 &&
           ((0x0041u16 <= *wszApplicationName.m_pData.offset(0 as isize) &&
             *wszApplicationName.m_pData.offset(0 as isize) <= 0x005Au16) ||
            (0x0061u16 <= *wszApplicationName.m_pData.offset(0 as isize) &&
             *wszApplicationName.m_pData.offset(0 as isize) <= 0x007Au16))
        {
            PathAddBackslashW(wszApplicationName.m_pData);
        }
    }

    let mut parametersBuffer: [u16; 1024 as usize] = [0; 1024 as usize];
    let mut wszParameters: LPWSTR = parametersBuffer.as_mut_ptr();
    let mut wszParamAlloc: CHeapPtr<u16, CLocalAllocator> = CHeapPtr::new();
    let mut parametersLen: u32 = _countof!(parametersBuffer);

    if !sei_tmp.lpParameters.is_null() {
        len = (lstrlenW(sei_tmp.lpParameters) + 1) as u32;
        if len > parametersLen {
            wszParamAlloc.Allocate((len) as usize);
            wszParameters = wszParamAlloc.m_pData;
            parametersLen = len;
        }
        strcpyW(wszParameters, sei_tmp.lpParameters);
    } else {
        *wszParameters = 0x0000u16;
    }

    let mut dirBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wszDir: LPWSTR = dirBuffer.as_mut_ptr();
    *wszDir.offset(0 as isize) = UNICODE_NULL;
    let mut wszDirAlloc: CHeapPtr<u16, CLocalAllocator> = CHeapPtr::new();
    if !sei_tmp.lpDirectory.is_null() && *sei_tmp.lpDirectory != 0 {
        if (sei_tmp.fMask & SEE_MASK_DOENVSUBST) != 0 {
            let tmp: LPWSTR = expand_environment(sei_tmp.lpDirectory);
            if !tmp.is_null() {
                wszDirAlloc.Attach(tmp);
                wszDir = wszDirAlloc.m_pData;
            }
        } else {
            __SHCloneStrW(
                core::ptr::addr_of_mut!(wszDirAlloc) as *mut *mut u16,
                sei_tmp.lpDirectory,
            );
            if !wszDirAlloc.m_pData.is_null() {
                wszDir = wszDirAlloc.m_pData;
            }
        }
    }
    if *wszDir.offset(0 as isize) == 0 {
        GetCurrentDirectoryW((_countof!(dirBuffer)) as u32, dirBuffer.as_mut_ptr());
        wszDir = dirBuffer.as_mut_ptr();
    }
    if PathIsDirectoryW(wszDir) == 0 {
        let iDrive: i32 = PathGetDriveNumberW(wszDir);
        if iDrive >= 0 {
            PathStripToRootW(wszDir);
            if PathIsDirectoryW(wszDir) == 0 {
                GetWindowsDirectoryW(dirBuffer.as_mut_ptr(), (_countof!(dirBuffer)) as u32);
                wszDir = dirBuffer.as_mut_ptr();
            }
        }
    }

    sei_tmp.lpFile = wszApplicationName.m_pData;
    sei_tmp.lpParameters = wszParameters;
    sei_tmp.lpDirectory = wszDir;

    if (sei_tmp.fMask & unsupportedFlags) != 0 {
    }

    if (sei_tmp.fMask & SEE_MASK_IDLIST) != 0 &&
       (sei_tmp.fMask & SEE_MASK_INVOKEIDLIST) != SEE_MASK_INVOKEIDLIST
    {
        let pidl: LPCITEMIDLIST = sei_tmp.lpIDList as LPCITEMIDLIST;
        let hr: i32 = SHGetNameAndFlagsW(
            pidl,
            SHGDN_FORPARSING,
            wszApplicationName.m_pData,
            dwApplicationNameLen,
            core::ptr::null_mut(),
        );
        if hr < 0 {
            if dwApplicationNameLen != 0 {
                *wszApplicationName.m_pData = UNICODE_NULL;
            }
            if _ILIsDesktop(pidl) == 0 {
            }
        }
        appKnownSingular = (TRUE) as u8;
    }

    if (sei_tmp.fMask & SEE_MASK_DOENVSUBST) != 0 && !StrIsNullOrEmpty(sei_tmp.lpFile) {
        let tmp: PWSTR = expand_environment(sei_tmp.lpFile);
        if !tmp.is_null() {
            wszApplicationName.Attach(tmp);
            sei_tmp.lpFile = wszApplicationName.m_pData;
        }
    }

    if (sei_tmp.fMask & SEE_MASK_INVOKEIDLIST) == SEE_MASK_INVOKEIDLIST {
        let hr: i32 = ShellExecute_ContextMenuVerb(core::ptr::addr_of_mut!(sei_tmp));
        if hr >= 0 {
            (*sei).hInstApp = 42 as HINSTANCE;
            return TRUE;
        }
    }

    if ERROR_SUCCESS == (ShellExecute_FromContextMenuHandlers(core::ptr::addr_of_mut!(sei_tmp))) as u32 {
        (*sei).hInstApp = 33 as HINSTANCE;
        return TRUE;
    }

    if (sei_tmp.fMask & SEE_MASK_CLASSALL) != 0 {
        retval = (SHELL_execute_class(
            wszApplicationName.m_pData,
            core::ptr::addr_of_mut!(sei_tmp),
            sei,
            execfunc,
        )) as UINT_PTR;
        if retval <= 32 && (sei_tmp.fMask & SEE_MASK_FLAG_NO_UI) == 0 {
            let Info: OPENASINFO = OPENASINFO {
                pcszFile: wszApplicationName.m_pData,
                pcszClass: core::ptr::null(),
                oaifInFlags: (OAIF_ALLOW_REGISTRATION | OAIF_EXEC) as i32,
            };

            // ntdef.h `DBG_UNREFERENCED_LOCAL_VARIABLE(L)` is `(void)L` — it only silences the
            // warning. NOT lowered to a no-op `fn`: an out-of-line empty body is an optimization
            // barrier that can uncloak a latent ABI mismatch as a `ud2`.
            let _ = &Info;
            do_error_dialog(retval, sei_tmp.hwnd, wszApplicationName.m_pData);
        }
        return if retval > 32 { 1 } else { 0 };
    }

    if (sei_tmp.fMask & SEE_MASK_IDLIST) == 0 &&
       (StrCmpNIW(
           sei_tmp.lpFile,
           CPP_COM_WSTR_AAEB94F421D7.as_ptr(),
           6,
       ) == 0 ||
        StrCmpNW(
            sei_tmp.lpFile,
            CPP_COM_WSTR_F2DD73FBEC92.as_ptr(),
            3,
        ) == 0)
    {
        let mut pidlParsed: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::new();
        let hr: i32 = SHParseDisplayName(
            sei_tmp.lpFile,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(pidlParsed.m_pData) as *mut LPITEMIDLIST,
            0,
            core::ptr::null_mut(),
        );
        if hr >= 0 &&
           SHELL_InvokePidl(core::ptr::addr_of_mut!(sei_tmp), pidlParsed.m_pData) != 0
        {
            sei_tmp.hInstApp = (UlongToHandle(42)).cast();
            return TRUE;
        }
    }

    if (sei_tmp.fMask & SEE_MASK_IDLIST) != 0 {
        appKnownSingular = SHELL_translate_idlist(
            core::ptr::addr_of_mut!(sei_tmp),
            wszParameters,
            parametersLen,
            wszApplicationName.m_pData,
            dwApplicationNameLen,
        );
    }

    // PHANTOM SYMBOL: shlwapi.h:643 is `#define UrlIsFileUrlW(x) UrlIsW(x, URLIS_FILEURL)`, so
    // there is no export of that name to bind — an `extern` decl for it compiles and then fails
    // to link. Call what the macro expands to; the island spells the enumerator with bindgen's
    // doubled prefix.
    if UrlIsW(sei_tmp.lpFile, URLIS_URLIS_FILEURL) != 0 {
        let mut buf: CHeapPtr<u16, CLocalAllocator> = CHeapPtr::new();
        let mut size: u32 = (MAX_PATH) as u32;
        if buf.Allocate((size) as usize) == 0 ||
           PathCreateFromUrlW(
               sei_tmp.lpFile,
               buf.m_pData,
               core::ptr::addr_of_mut!(size),
               0,
           ) < 0
        {
            SetLastError(ERROR_OUTOFMEMORY);
            return FALSE;
        }

        wszApplicationName.Attach(buf.Detach());
        sei_tmp.lpFile = wszApplicationName.m_pData;
    }

    let lpFile: PCWSTR = sei_tmp.lpFile;
    if *sei_tmp.lpParameters == 0 && appKnownSingular == 0 {
        if *sei_tmp.lpFile.offset(0 as isize) == 0x0022u16 {
            let pszArgs: LPWSTR = PathGetArgsW(wszApplicationName.m_pData);
            PathRemoveArgsW(wszApplicationName.m_pData);
            PathUnquoteSpacesW(wszApplicationName.m_pData);
            parametersLen = (lstrlenW(pszArgs)) as u32;
            if parametersLen < _countof!(parametersBuffer) {
                StringCchCopyW(
                    parametersBuffer.as_mut_ptr(),
                    (_countof!(parametersBuffer)) as usize,
                    pszArgs,
                );
                wszParameters = parametersBuffer.as_mut_ptr();
            } else {
                wszParamAlloc.Attach(StrDupW(pszArgs));
                wszParameters = wszParamAlloc.m_pData;
            }
        } else if ((*sei).fMask & SEE_MASK_IDLIST) == SEE_MASK_IDLIST {
            let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
            let mut xlpFile: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
            let mut space: LPWSTR;
            let mut s: LPWSTR;

            let beg: LPWSTR = wszApplicationName.m_pData;
            s = beg;
            loop {
                space = strchrW(s, 0x0020u16) as LPWSTR;
                if space.is_null() {
                    break;
                }

                let idx: i32 = space.offset_from(sei_tmp.lpFile) as i32;
                memcpy(
                    buffer.as_mut_ptr() as *mut _,
                    sei_tmp.lpFile as *const _,
                    (idx as u32 * core::mem::size_of::<u16>() as u32) as usize,
                );
                *buffer.as_mut_ptr().offset(idx as isize) = 0x0000u16;

                if SearchPathW(
                    if *sei_tmp.lpDirectory != 0 {
                        sei_tmp.lpDirectory
                    } else {
                        core::ptr::null()
                    },
                    buffer.as_ptr(),
                    CPP_COM_WSTR_7ABCE230ED71.as_ptr(),
                    (_countof!(xlpFile)) as u32,
                    xlpFile.as_mut_ptr(),
                    core::ptr::null_mut(),
                ) != 0
                {
                    let p: PCWSTR = space.offset(1 as isize);
                    let mut p_mut = p as LPWSTR;

                    while isspaceW((*p_mut as u32) as u16) != 0 {
                        p_mut = p_mut.offset(1);
                    }

                    strcpyW(wszParameters, p_mut);
                    *space = 0x0000u16;

                    break;
                }

                s = space.offset(1 as isize);
            }
        }
    }

    let mut wcmdBuffer: [u16; 1024 as usize] = [0; 1024 as usize];
    let mut wcmd: LPWSTR = wcmdBuffer.as_mut_ptr();
    let mut wcmdLen: u32 = _countof!(wcmdBuffer);
    let mut wcmdAlloc: CHeapPtr<u16, CLocalAllocator> = CHeapPtr::new();

    if PathIsExeW(lpFile) != 0 {
        len = (lstrlenW(wszApplicationName.m_pData) + 3) as u32;
        if *sei_tmp.lpParameters.offset(0 as isize) != 0 {
            len += (1 + lstrlenW(wszParameters)) as u32;
        }
        if len > wcmdLen {
            wcmdAlloc.Allocate((len) as usize);
            wcmd = wcmdAlloc.m_pData;
            wcmdLen = len;
        }
        swprintf(
            wcmd,
            CPP_COM_WSTR_9896841D0A6A.as_ptr(),
            wszApplicationName.m_pData as LPWSTR,
        );
        if *sei_tmp.lpParameters.offset(0 as isize) != 0 {
            strcatW(wcmd, CPP_COM_WSTR_8D71B3FAAB82.as_ptr());
            strcatW(wcmd, wszParameters);
        }

        retval = execfunc(
            wcmd,
            core::ptr::null_mut(),
            FALSE,
            core::ptr::addr_of_mut!(sei_tmp),
            sei,
        );
        if retval > 32 {
            return TRUE;
        }
    }

    let mut wszKeyname: [u16; 256 as usize] = [0; 256 as usize];
    let mut env: CHeapPtr<u16, CLocalAllocator> = CHeapPtr::new();
    *wcmd.offset(0 as isize) = UNICODE_NULL;
    retval = SHELL_FindExecutable(
        sei_tmp.lpDirectory,
        lpFile,
        sei_tmp.lpVerb,
        wcmd,
        wcmdLen,
        wszKeyname.as_mut_ptr(),
        core::ptr::addr_of_mut!(env) as *mut *mut u16,
        sei_tmp.lpIDList as LPITEMIDLIST,
        sei_tmp.lpParameters,
    ) as UINT_PTR;
    if retval > 32 {
        retval = (SHELL_quote_and_execute(
            wcmd,
            wszParameters,
            wszKeyname.as_ptr(),
            wszApplicationName.m_pData,
            env.m_pData,
            core::ptr::addr_of_mut!(sei_tmp),
            sei,
            execfunc,
        )) as UINT_PTR;
    } else if PathIsDirectoryW(lpFile) != 0 {
        let mut wExec: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut lpQuotedFile: CHeapPtr<u16, CLocalAllocator> = CHeapPtr::new();
        if lpQuotedFile.Allocate(strlenW(lpFile) + 3) != 0 {
            retval = SHELL_FindExecutable(
                sei_tmp.lpDirectory,
                CPP_COM_WSTR_1EDCE7FBFAA4.as_ptr(),
                CPP_COM_WSTR_82DB442E7A60.as_ptr(),
                wExec.as_mut_ptr(),
                (MAX_PATH) as u32,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(env) as *mut *mut u16,
                core::ptr::null_mut(),
                core::ptr::null(),
            ) as UINT_PTR;
            if retval > 32 {
                swprintf(
                    lpQuotedFile.m_pData,
                    CPP_COM_WSTR_9896841D0A6A.as_ptr(),
                    lpFile,
                );
                retval = (SHELL_quote_and_execute(
                    wExec.as_ptr(),
                    lpQuotedFile.m_pData,
                    wszKeyname.as_ptr(),
                    wszApplicationName.m_pData,
                    env.m_pData,
                    core::ptr::addr_of_mut!(sei_tmp),
                    sei,
                    execfunc,
                )) as UINT_PTR;
            }
        } else {
            retval = 0;
        }
    } else if PathIsURLW(lpFile) != 0 {
        retval = (SHELL_execute_url(
            lpFile,
            wcmd,
            core::ptr::addr_of_mut!(sei_tmp),
            sei,
            execfunc,
        )) as UINT_PTR;
    } else if strncmpiW(lpFile, CPP_COM_WSTR_94FDE40D078A.as_ptr(), 3) == 0 {
        let mut lpstrTmpFile: [u16; 256 as usize] = [0; 256 as usize];
        strcpyW(lpstrTmpFile.as_mut_ptr(), CPP_COM_WSTR_31DA8B28DF2D.as_ptr());
        strcatW(lpstrTmpFile.as_mut_ptr(), lpFile);
        retval = ShellExecuteW(
            sei_tmp.hwnd,
            sei_tmp.lpVerb,
            lpstrTmpFile.as_ptr(),
            core::ptr::null(),
            core::ptr::null(),
            0,
        ) as UINT_PTR;
    }

    if retval <= 32 && (sei_tmp.fMask & SEE_MASK_FLAG_NO_UI) == 0 {
        if retval == (SE_ERR_NOASSOC) as UINT_PTR && ((*sei).fMask & SEE_MASK_CLASSALL) == 0 {
            retval = InvokeOpenWith(sei_tmp.hwnd, &mut *sei);
        }
        if retval <= 32 {
            do_error_dialog(retval, sei_tmp.hwnd, lpFile);
        }
    }

    (*sei).hInstApp = if retval > 32 {
        33 as HINSTANCE
    } else {
        retval as HINSTANCE
    };

    return if retval > 32 { 1 } else { 0 };
}

#[no_mangle]
pub unsafe extern "system" fn ShellExecuteA(
    hWnd: HWND,
    lpVerb: PCSTR,
    lpFile: PCSTR,
    lpParameters: PCSTR,
    lpDirectory: PCSTR,
    iShowCmd: c_int,
) -> HINSTANCE {
    let mut sei: SHELLEXECUTEINFOA = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOA>() as u32;
    sei.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
    sei.hwnd = hWnd;
    sei.lpVerb = lpVerb;
    sei.lpFile = lpFile;
    sei.lpParameters = lpParameters;
    sei.lpDirectory = lpDirectory;
    sei.nShow = iShowCmd;
    sei.lpIDList = core::ptr::null_mut();
    sei.lpClass = core::ptr::null();
    sei.hkeyClass = core::ptr::null_mut();
    sei.dwHotKey = 0;
    sei.hProcess = core::ptr::null_mut();

    let compat = SHGetAppCompatFlags(SHACF_WIN95SHLEXEC);
    if (compat & SHACF_WIN95SHLEXEC) == 0 {
        sei.fMask |= SEE_MASK_NOASYNC;
    }
    ShellExecuteExA(core::ptr::addr_of_mut!(sei));
    sei.hInstApp
}
unsafe extern "C" fn ShellExecute_Normal(sei: *mut SHELLEXECUTEINFOW) -> u32 {
    if SHELL_execute(sei, SHELL_ExecuteW) != 0 {
        return (ERROR_SUCCESS) as u32;
    }
    let err = GetLastError();
    if err == 0 {
        DbgPrint(
            b"FIXME: Failed with error 0 on '%ls'\n\0".as_ptr() as *const i8,
            (*sei).lpFile,
        );
    }
    if err != 0 { err } else { ERROR_FILE_NOT_FOUND }
}

unsafe extern "C" fn ShellExecute_ShowError(
    ExecInfo: *const SHELLEXECUTEINFOW,
    pszCaption: LPCWSTR,
    dwError: DWORD,
) {}
#[no_mangle]
pub unsafe extern "system" fn ShellExecuteExA(sei: *mut SHELLEXECUTEINFOA) -> BOOL {
    let mut wVerb: *mut u16 = core::ptr::null_mut();
    let mut wFile: *mut u16 = core::ptr::null_mut();
    let mut wParameters: *mut u16 = core::ptr::null_mut();
    let mut wDirectory: *mut u16 = core::ptr::null_mut();
    let mut wClass: *mut u16 = core::ptr::null_mut();

    // C: `if (sei->cbSize != sizeof(SHELLEXECUTEINFOA))`. The translation dropped the `sizeof`
    // and left the bare type name. This is the ENTRY GATE of a public export — it is what
    // rejects a caller's malformed or wrong-version struct before any field is read.
    if (*sei).cbSize != core::mem::size_of::<SHELLEXECUTEINFOA>() as DWORD {
        (*sei).hInstApp = UlongToHandle(ERROR_ACCESS_DENIED).cast();
        SetLastError(ERROR_ACCESS_DENIED);
        return FALSE;
    }

    let mut seiW: SHELLEXECUTEINFOW = core::mem::zeroed();
    memcpy(
        &raw mut seiW as *mut _ as *mut core::ffi::c_void,
        sei as *const _ as *const core::ffi::c_void,
        core::mem::size_of::<SHELLEXECUTEINFOW>(),
    );
    seiW.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;

    if !(*sei).lpVerb.is_null() {
        seiW.lpVerb = __SHCloneStrAtoW(&raw mut wVerb, (*sei).lpVerb);
    }

    if !(*sei).lpFile.is_null() {
        seiW.lpFile = __SHCloneStrAtoW(&raw mut wFile, (*sei).lpFile);
    }

    if !(*sei).lpParameters.is_null() {
        seiW.lpParameters = __SHCloneStrAtoW(&raw mut wParameters, (*sei).lpParameters);
    }

    if !(*sei).lpDirectory.is_null() {
        seiW.lpDirectory = __SHCloneStrAtoW(&raw mut wDirectory, (*sei).lpDirectory);
    }

    if ((*sei).fMask & SEE_MASK_CLASSALL) == SEE_MASK_CLASSNAME && !(*sei).lpClass.is_null() {
        seiW.lpClass = __SHCloneStrAtoW(&raw mut wClass, (*sei).lpClass);
    } else {
        seiW.lpClass = core::ptr::null_mut();
    }

    let ret: BOOL = ShellExecuteExW(&raw mut seiW);

    (*sei).hInstApp = seiW.hInstApp;

    if ((*sei).fMask & SEE_MASK_NOCLOSEPROCESS) != 0 {
        (*sei).hProcess = seiW.hProcess;
    }

    SHFree((wVerb).cast());
    SHFree((wFile).cast());
    SHFree((wParameters).cast());
    SHFree((wDirectory).cast());
    SHFree((wClass).cast());

    ret
}

#[no_mangle]
pub unsafe extern "system" fn ShellExecuteExW(sei: *mut SHELLEXECUTEINFOW) -> BOOL {
    // Same dropped `sizeof` as ShellExecuteExA above.
    if (*sei).cbSize != core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD {
        (*sei).hInstApp = UlongToHandle(SE_ERR_ACCESSDENIED) as HINSTANCE;
        SetLastError(ERROR_ACCESS_DENIED);
        return FALSE;
    }
    let hrCoInit: HRESULT = SHCoInitializeAnyApartment();
    if SHRegGetBoolUSValueW(
        CPP_COM_WSTR_5548E462B2EE.as_ptr(),
        CPP_COM_WSTR_E7BEDDFB7AB6.as_ptr(),
        FALSE,
        FALSE,
    ) != 0
    {
        match (*sei).nShow as u32 {
            SW_SHOW | SW_SHOWDEFAULT | SW_SHOWNORMAL | SW_RESTORE => {
                (*sei).nShow = (SW_SHOWMAXIMIZED) as i32;
            }
            _ => {}
        }
    }
    let fOldMask: ULONG = (*sei).fMask;
    if (fOldMask & SEE_MASK_NOASYNC) == 0 && SHELL_InRunDllProcess() != 0 {
        (*sei).fMask |= SEE_MASK_WAITFORINPUTIDLE | SEE_MASK_NOASYNC;
    }
    let dwError: DWORD = ShellExecute_Normal(sei);
    if dwError != 0 && dwError != ERROR_DLL_NOT_FOUND && dwError != ERROR_CANCELLED {
        ShellExecute_ShowError(sei, core::ptr::null() as LPCWSTR, dwError);
    }
    (*sei).fMask = fOldMask;
    if (hrCoInit as i32) >= 0 {
        CoUninitialize();
    }
    if dwError != 0 {
        SetLastError(dwError);
    }
    if dwError == (ERROR_SUCCESS) as u32 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn ShellExecuteW(
    hwnd: HWND,
    lpVerb: PCWSTR,
    lpFile: PCWSTR,
    lpParameters: PCWSTR,
    lpDirectory: PCWSTR,
    nShowCmd: i32,
) -> HINSTANCE {
    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
    sei.hwnd = hwnd;
    sei.lpVerb = lpVerb;
    sei.lpFile = lpFile;
    sei.lpParameters = lpParameters;
    sei.lpDirectory = lpDirectory;
    sei.nShow = (nShowCmd as u32) as i32;
    sei.lpIDList = core::ptr::null_mut();
    sei.lpClass = core::ptr::null();
    sei.hkeyClass = core::ptr::null_mut();
    sei.dwHotKey = 0;
    sei.hProcess = core::ptr::null_mut();

    if (SHGetAppCompatFlags(SHACF_WIN95SHLEXEC) & SHACF_WIN95SHLEXEC) == 0 {
        sei.fMask |= SEE_MASK_NOASYNC;
    }
    ShellExecuteExW(&mut sei);
    sei.hInstApp
}

#[no_mangle]
pub unsafe extern "system" fn WOWShellExecute(
    hWnd: HWND,
    lpVerb: *const core::ffi::c_char,
    lpFile: *const core::ffi::c_char,
    lpParameters: *const core::ffi::c_char,
    lpDirectory: *const core::ffi::c_char,
    iShowCmd: i32,
    callback: *mut core::ffi::c_void,
) -> HINSTANCE {
    let mut wVerb: PWSTR = core::ptr::null_mut();
    let mut wFile: PWSTR = core::ptr::null_mut();
    let mut wParameters: PWSTR = core::ptr::null_mut();
    let mut wDirectory: PWSTR = core::ptr::null_mut();
    let hProcess: HANDLE = core::ptr::null_mut();

    let mut seiW: SHELLEXECUTEINFOW = core::mem::zeroed();

    seiW.lpVerb = if !lpVerb.is_null() {
        __SHCloneStrAtoW(&mut wVerb, lpVerb)
    } else {
        core::ptr::null_mut()
    };
    seiW.lpFile = if !lpFile.is_null() {
        __SHCloneStrAtoW(&mut wFile, lpFile)
    } else {
        core::ptr::null_mut()
    };
    seiW.lpParameters = if !lpParameters.is_null() {
        __SHCloneStrAtoW(&mut wParameters, lpParameters)
    } else {
        core::ptr::null_mut()
    };
    seiW.lpDirectory = if !lpDirectory.is_null() {
        __SHCloneStrAtoW(&mut wDirectory, lpDirectory)
    } else {
        core::ptr::null_mut()
    };

    seiW.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    seiW.fMask = 0;
    seiW.hwnd = hWnd;
    seiW.nShow = iShowCmd;
    seiW.lpIDList = core::ptr::null_mut();
    seiW.lpClass = core::ptr::null();
    seiW.hkeyClass = core::ptr::null_mut();
    seiW.dwHotKey = 0;
    seiW.hProcess = hProcess;

    let callback_fn: SHELL_ExecuteW32 = core::mem::transmute(callback);
    SHELL_execute(core::ptr::addr_of_mut!(seiW), callback_fn);

    SHFree((wVerb).cast());
    SHFree((wFile).cast());
    SHFree((wParameters).cast());
    SHFree((wDirectory).cast());

    seiW.hInstApp
}

#[no_mangle]
pub unsafe extern "system" fn OpenAs_RunDLLW(
    hwnd: HWND,
    hinst: HINSTANCE,
    cmdline: PCWSTR,
    cmdshow: i32,
) {
    let mut info: OPENASINFO = OPENASINFO {
        pcszFile: cmdline,
        pcszClass: core::ptr::null_mut(),
        oaifInFlags: (OAIF_ALLOW_REGISTRATION | OAIF_REGISTER_EXT | OAIF_EXEC) as i32,
    };
    SHOpenWithDialog(hwnd, &mut info);
}

#[no_mangle]
pub unsafe extern "system" fn OpenAs_RunDLLA(
    hwnd: HWND,
    hinst: HINSTANCE,
    cmdline: *const u8,
    cmdshow: i32,
) {
    let mut pszCmdLineW: *mut u16 = core::ptr::null_mut();
    if !cmdline.is_null() {
        __SHCloneStrAtoW(core::ptr::addr_of_mut!(pszCmdLineW), (cmdline).cast());
    }
    OpenAs_RunDLLW(hwnd, hinst, pszCmdLineW, cmdshow);
    SHFree((pszCmdLineW).cast());
}
unsafe extern "C" fn SplitParams(psz: *const u16, pszArg0: *mut u16, cchArg0: usize) -> *const u16 {
    let mut ich: usize = 0;
    let mut pch: *const u16;
    if *psz == 0x0022u16 {
        pch = psz.add(1);
        while *pch != 0 && ich + 1 < cchArg0 {
            if *pch == 0x0022u16 && *pch.offset(1) == 0x0022u16 {
                *pszArg0.add(ich) = 0x0022u16;
            } else if *pch == 0x0022u16 {
                pch = pch.add(1);
                break;
            } else {
                *pszArg0.add(ich) = *pch;
            }
            ich += 1;
            pch = pch.add(1);
        }
    } else {
        pch = psz;
        while *pch != 0 && iswspace((*pch) as i32) == 0 && ich + 1 < cchArg0 {
            *pszArg0.add(ich) = *pch;
            ich += 1;
            pch = pch.add(1);
        }
    }
    *pszArg0.add(ich) = 0;
    while iswspace((*pch) as i32) != 0 {
        pch = pch.add(1);
    }
    pch
}
#[no_mangle]
pub unsafe extern "system" fn ShellExecCmdLine(
    hwnd: HWND,
    pwszCommand: PCWSTR,
    pwszStartDir: PCWSTR,
    nShow: i32,
    pUnused: LPVOID,
    dwSeclFlags: u32,
) -> HRESULT {
    let mut info: SHELLEXECUTEINFOW = core::mem::zeroed();
    let mut dwSize: u32;
    let mut dwType: u32 = 0;
    let mut dwFlags: u32 = SEE_MASK_DOENVSUBST | SEE_MASK_NOASYNC;
    let mut pszVerb: PCWSTR = core::ptr::null();
    let mut szFile = [0u16; (MAX_PATH) as usize];
    let mut szFile2 = [0u16; (MAX_PATH) as usize];
    let mut hr: HRESULT;
    let mut pchParams: PCWSTR;
    let mut lpCommand: PWSTR = core::ptr::null_mut();

    if pwszCommand.is_null() {
        RaiseException(
            (EXCEPTION_ACCESS_VIOLATION) as u32,
            EXCEPTION_NONCONTINUABLE,
            1,
            pwszCommand as *const ULONG_PTR,
        );
    }

    __SHCloneStrW(core::ptr::addr_of_mut!(lpCommand), pwszCommand);
    StrTrimW(lpCommand, CPP_COM_WSTR_731D7EC48EEF.as_ptr());

    if (dwSeclFlags & SECL_NO_UI) != 0 {
        dwFlags |= SEE_MASK_FLAG_NO_UI;
    }
    if (dwSeclFlags & SECL_LOG_USAGE) != 0 {
        dwFlags |= SEE_MASK_FLAG_LOG_USAGE;
    }
    if (dwSeclFlags & SECL_USE_IDLIST) != 0 {
        dwFlags |= SEE_MASK_INVOKEIDLIST;
    }

    if (dwSeclFlags & SECL_RUNAS) != 0 {
        dwSize = 0;
        hr = AssocQueryStringW(
            ASSOCF_NONE,
            ASSOCSTR_COMMAND,
            lpCommand,
            CPP_COM_WSTR_878FA9FF9662.as_ptr(),
            core::ptr::null_mut(),
            &raw mut dwSize,
        );
        if hr >= 0 && dwSize != 0 {
            pszVerb = CPP_COM_WSTR_878FA9FF9662.as_ptr();
        }
    }

    if PathIsURLW(lpCommand) != 0
        || UrlIsW(lpCommand, URLIS_APPLIABLE) != 0
    {
        StringCchCopyW(szFile.as_mut_ptr(), (_countof!(&szFile)) as usize, lpCommand);
        pchParams = core::ptr::null();
    } else {
        let mut apPathList = [core::ptr::null::<u16>(); 2];

        pchParams = SplitParams(lpCommand, szFile.as_mut_ptr(), (_countof!(&szFile)) as usize);
        if szFile[0] != UNICODE_NULL
            && szFile[1] == 0x003Au16
            && szFile[2] == UNICODE_NULL
        {
            PathAddBackslashW(szFile.as_mut_ptr());
        }

        let mut szCurDir = [0u16; (MAX_PATH) as usize];
        GetCurrentDirectoryW(_countof!(&szCurDir), szCurDir.as_mut_ptr());
        if !pwszStartDir.is_null() {
            SetCurrentDirectoryW(pwszStartDir);
        }

        if (PathIsRelativeW(szFile.as_ptr()) != 0
            && GetFullPathNameW(szFile.as_ptr(), _countof!(&szFile2), szFile2.as_mut_ptr(), core::ptr::null_mut()) != 0
            && PathFileExistsW(szFile2.as_ptr()) != 0)
            || SearchPathW(core::ptr::null(), szFile.as_ptr(), core::ptr::null(), _countof!(&szFile2), szFile2.as_mut_ptr(), core::ptr::null_mut()) != 0
        {
            StringCchCopyW(szFile.as_mut_ptr(), (_countof!(&szFile)) as usize, szFile2.as_ptr());
        }

        apPathList[0] = pwszStartDir;
        apPathList[1] = core::ptr::null();
        PathFindOnPathExW(szFile.as_mut_ptr(), apPathList.as_mut_ptr(), WHICH_DEFAULT);

        if (dwSeclFlags & SECL_ALLOW_NONEXE) == 0 {
            if GetBinaryTypeW(szFile.as_ptr(), &raw mut dwType) == 0 {
                SHFree((lpCommand).cast());

                if (dwSeclFlags & SECL_NO_UI) == 0 {
                    let mut szText = [0u16; (128 + MAX_PATH) as usize];
                    let mut szFormat = [0u16; 128];
                    LoadStringW(shell32_hInstance, IDS_FILE_NOT_FOUND, szFormat.as_mut_ptr(), (_countof!(&szFormat)) as i32);
                    strsafe_rust::StringCchPrintfW!(szText.as_mut_ptr(), (_countof!(&szText)) as usize, szFormat.as_ptr(), szFile.as_ptr());
                    MessageBoxW(hwnd, szText.as_ptr(), core::ptr::null(), MB_ICONERROR);
                }
                return CO_E_APPNOTFOUND;
            }
        } else {
            if GetFileAttributesW(szFile.as_ptr()) == INVALID_FILE_ATTRIBUTES {
                SHFree((lpCommand).cast());

                if (dwSeclFlags & SECL_NO_UI) == 0 {
                    let mut szText = [0u16; (128 + MAX_PATH) as usize];
                    let mut szFormat = [0u16; 128];
                    LoadStringW(shell32_hInstance, IDS_FILE_NOT_FOUND, szFormat.as_mut_ptr(), (_countof!(&szFormat)) as i32);
                    strsafe_rust::StringCchPrintfW!(szText.as_mut_ptr(), (_countof!(&szText)) as usize, szFormat.as_ptr(), szFile.as_ptr());
                    MessageBoxW(hwnd, szText.as_ptr(), core::ptr::null(), MB_ICONERROR);
                }
                return HRESULT_FROM_WIN32((ERROR_FILE_NOT_FOUND) as i32);
            }
        }
    }

    ZeroMemory(
        &raw mut info as *mut _,
        core::mem::size_of::<SHELLEXECUTEINFOW>(),
    );
    info.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = dwFlags;
    info.hwnd = hwnd;
    info.lpVerb = pszVerb;
    info.lpFile = szFile.as_ptr();
    info.lpParameters = if !pchParams.is_null() && *pchParams != 0 { pchParams } else { core::ptr::null() };
    info.lpDirectory = pwszStartDir;
    info.nShow = nShow as c_int;
    let error: u32 = if ShellExecuteExW(&raw mut info) != 0 {
        (ERROR_SUCCESS) as u32
    } else {
        GetLastError()
    };
    SHFree((lpCommand).cast());
    HRESULT_FROM_WIN32((error) as i32)
}

#[no_mangle]
pub unsafe extern "system" fn RealShellExecuteExA(
    hwnd: HWND,
    lpOperation: *const core::ffi::c_char,
    lpFile: *const core::ffi::c_char,
    lpParameters: *const core::ffi::c_char,
    lpDirectory: *const core::ffi::c_char,
    lpReturn: *mut core::ffi::c_char,
    lpTitle: *const core::ffi::c_char,
    lpReserved: *mut core::ffi::c_void,
    nCmdShow: core::ffi::c_int,
    lphProcess: PHANDLE,
    dwFlags: DWORD,
) -> HINSTANCE {
    TRACE!("(%p, %s, %s, %s, %s, %p, %s, %p, %u, %p, %lu)\n",
        hwnd.cast::<core::ffi::c_void>(),
        debugstr_a(lpOperation),
        debugstr_a(lpFile),
        debugstr_a(lpParameters),
        debugstr_a(lpDirectory),
        lpReserved,
        debugstr_a(lpTitle),
        lpReserved,
        nCmdShow as u32,
        lphProcess as *const core::ffi::c_void,
        dwFlags,
    );

    let mut exec_info: SHELLEXECUTEINFOA = core::mem::zeroed();
    ZeroMemory(core::ptr::addr_of_mut!(exec_info) as *mut core::ffi::c_void, core::mem::size_of::<SHELLEXECUTEINFOA>());
    exec_info.cbSize = core::mem::size_of::<SHELLEXECUTEINFOA>() as u32;
    exec_info.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
    exec_info.hwnd = hwnd;
    exec_info.lpVerb = lpOperation;
    exec_info.lpFile = lpFile;
    exec_info.lpParameters = lpParameters;
    exec_info.lpDirectory = lpDirectory;
    exec_info.nShow = (nCmdShow as u16) as i32;

    if !lpReserved.is_null() {
        exec_info.fMask |= SEE_MASK_USE_RESERVED;
        exec_info.hInstApp = lpReserved.cast();
    }

    if !lpTitle.is_null() {
        exec_info.fMask |= SEE_MASK_HASTITLE;
        exec_info.lpClass = lpTitle;
    }

    if (dwFlags & 1) != 0 {
        exec_info.fMask |= SEE_MASK_FLAG_SEPVDM;
    }

    if (dwFlags & 2) != 0 {
        exec_info.fMask |= SEE_MASK_NO_CONSOLE;
    }

    if lphProcess.is_null() {
        ShellExecuteExA(core::ptr::addr_of_mut!(exec_info));
    } else {
        exec_info.fMask |= SEE_MASK_NOCLOSEPROCESS;
        ShellExecuteExA(core::ptr::addr_of_mut!(exec_info));
        *lphProcess = exec_info.hProcess;
    }

    exec_info.hInstApp
}

#[no_mangle]
pub unsafe extern "system" fn RealShellExecuteExW(
    hwnd: HWND,
    lpOperation: LPCWSTR,
    lpFile: LPCWSTR,
    lpParameters: LPCWSTR,
    lpDirectory: LPCWSTR,
    lpReturn: LPWSTR,
    lpTitle: LPCWSTR,
    lpReserved: LPVOID,
    nCmdShow: core::ffi::c_int,
    lphProcess: PHANDLE,
    dwFlags: DWORD,
) -> HINSTANCE {
    let mut ExecInfo: SHELLEXECUTEINFOW = core::mem::zeroed();

    ZeroMemory(
        core::ptr::from_mut(&mut ExecInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<SHELLEXECUTEINFOW>(),
    );

    ExecInfo.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD;
    ExecInfo.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
    ExecInfo.hwnd = hwnd;
    ExecInfo.lpVerb = lpOperation;
    ExecInfo.lpFile = lpFile;
    ExecInfo.lpParameters = lpParameters;
    ExecInfo.lpDirectory = lpDirectory;
    ExecInfo.nShow = (nCmdShow as WORD) as i32;

    if !lpReserved.is_null() {
        ExecInfo.fMask |= SEE_MASK_USE_RESERVED;
        ExecInfo.hInstApp = lpReserved as HINSTANCE;
    }

    if !lpTitle.is_null() {
        ExecInfo.fMask |= SEE_MASK_HASTITLE;
        ExecInfo.lpClass = lpTitle;
    }

    if (dwFlags & 1) != 0 {
        ExecInfo.fMask |= SEE_MASK_FLAG_SEPVDM;
    }

    if (dwFlags & 2) != 0 {
        ExecInfo.fMask |= SEE_MASK_NO_CONSOLE;
    }

    if lphProcess.is_null() {
        ShellExecuteExW(core::ptr::addr_of_mut!(ExecInfo));
    } else {
        ExecInfo.fMask |= SEE_MASK_NOCLOSEPROCESS;
        ShellExecuteExW(core::ptr::addr_of_mut!(ExecInfo));
        *lphProcess = ExecInfo.hProcess;
    }

    ExecInfo.hInstApp
}

#[no_mangle]
pub unsafe extern "system" fn RealShellExecuteA(
    hwnd: HWND,
    lpOperation: *const i8,
    lpFile: *const i8,
    lpParameters: *const i8,
    lpDirectory: *const i8,
    lpReturn: *mut i8,
    lpTitle: *const i8,
    lpReserved: *mut core::ffi::c_void,
    nCmdShow: core::ffi::c_int,
    lphProcess: PHANDLE,
) -> HINSTANCE {
    RealShellExecuteExA(
        hwnd,
        lpOperation,
        lpFile,
        lpParameters,
        lpDirectory,
        lpReturn,
        lpTitle,
        lpReserved,
        nCmdShow,
        lphProcess,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RealShellExecuteW(
    hwnd: HWND,
    lpOperation: LPCWSTR,
    lpFile: LPCWSTR,
    lpParameters: LPCWSTR,
    lpDirectory: LPCWSTR,
    lpReturn: LPWSTR,
    lpTitle: LPCWSTR,
    lpReserved: LPVOID,
    nCmdShow: core::ffi::c_int,
    lphProcess: PHANDLE,
) -> HINSTANCE {
    RealShellExecuteExW(
        hwnd,
        lpOperation,
        lpFile,
        lpParameters,
        lpDirectory,
        lpReturn,
        lpTitle,
        lpReserved,
        nCmdShow,
        lphProcess,
        0,
    )
}

#[no_mangle]
pub unsafe extern "C" fn PathProcessCommandW(
    pszSrc: PCWSTR,
    pszDest: PWSTR,
    cchDest: i32,
    dwFlags: u32,
) -> i32 {
    if pszSrc.is_null() {
        return -1;
    }

    let mut szPath = CStringW::new();
    let mut pchArg: PCWSTR = core::ptr::null();

    if *pszSrc == 0x0022u16 {
        let pszSrc = pszSrc.add(1);
        let pch = wcschr(pszSrc, 0x0022u16);
        if !pch.is_null() {
            let len = pch.offset_from(pszSrc) as i32;
            szPath.SetString(pszSrc, len);
            pchArg = pch.add(1);
        } else {
            szPath = CStringW::from_ptr(pszSrc);
        }

        if (dwFlags & PPCF_FORCEQUALIFY) != 0 || PathIsRelativeW(szPath.as_ptr()) != 0 {
            let ret = PathResolveW(
                szPath.GetBuffer((MAX_PATH) as i32),
                core::ptr::null_mut(),
                PRF_TRYPROGRAMEXTENSIONS,
            );
            szPath.ReleaseBuffer();
            if ret == 0 {
                return -1;
            }
        }
    } else {
        let mut resolved = FALSE;
        let resolveRelative =
            PathIsRelativeW(pszSrc) != 0 || (dwFlags & PPCF_FORCEQUALIFY) != 0;
        let mut cchPath = 0;
        let mut ich = 0;

        loop {
            szPath.AppendChar(*pszSrc.add(ich as usize));

            if *pszSrc.add(ich as usize) != 0 && *pszSrc.add(ich as usize) != 0x0020u16 {
                ich += 1;
                continue;
            }

            szPath = szPath.Left(ich);

            if resolveRelative
                && PathResolveW(
                    szPath.GetBuffer((MAX_PATH) as i32),
                    core::ptr::null_mut(),
                    PRF_TRYPROGRAMEXTENSIONS,
                ) == 0
            {
                szPath.ReleaseBuffer();
                szPath.AppendChar(*pszSrc.add(ich as usize));
            } else {
                szPath.ReleaseBuffer();
                let attrs = GetFileAttributesW(szPath.as_ptr());
                if attrs != INVALID_FILE_ATTRIBUTES
                    && (attrs & FILE_ATTRIBUTE_DIRECTORY == 0
                        || (dwFlags & PPCF_NODIRECTORIES) == 0)
                {
                    resolved = TRUE;
                    pchArg = pszSrc.add(ich as usize);

                    if (dwFlags & PPCF_LONGESTPOSSIBLE) == 0 {
                        break;
                    }

                    cchPath = ich;
                    break;
                } else if !resolveRelative {
                    szPath.AppendChar(*pszSrc.add(ich as usize));
                }
            }

            if szPath.GetAt(ich) == 0 {
                szPath.ReleaseBuffer();
                break;
            }

            ich += 1;
        }

        if resolved == 0 {
            return -1;
        }

        if cchPath != 0 && (dwFlags & PPCF_LONGESTPOSSIBLE) != 0 {
            szPath = szPath.Left(cchPath);
            pchArg = pszSrc.add(cchPath as usize);
        }
    }

    let needsQuoting = (dwFlags & PPCF_ADDQUOTES) != 0
        && !wcschr(szPath.as_ptr(), 0x0020u16).is_null();

    let mut result: CStringW = if needsQuoting {
        let mut q = CStringW::from_ptr(CPP_COM_WSTR_1B9334FEECE6.as_ptr());
        q.Append(szPath.as_ptr());
        q.Append(CPP_COM_WSTR_1B9334FEECE6.as_ptr());
        q
    } else {
        szPath
    };

    if !pchArg.is_null() && (dwFlags & PPCF_ADDARGUMENTS) != 0 {
        result.Append(pchArg);
    }

    let requiredSize = result.GetLength() + 1;

    if pszDest.is_null() {
        return requiredSize;
    }

    if requiredSize > cchDest
        || StringCchCopyW(pszDest, (cchDest) as usize, result.as_ptr()) != S_OK
    {
        return -1;
    }

    requiredSize
}
unsafe extern "C" fn ShellExec_RunDLL_Helper(hwnd: HWND, hInstance: HINSTANCE, mut pszCmdLine: PCWSTR, nCmdShow: i32) {
    if pszCmdLine.is_null() || *pszCmdLine == 0 {
        return;
    }

    let mut fNewMask: u32 = SEE_MASK_NOASYNC;
    if *pszCmdLine == 0x003F_u16 {
        let mut MaskValue: i32 = 0;
        if StrToIntExW(pszCmdLine.offset(1 as isize), STIF_SUPPORT_HEX, &mut MaskValue) != 0 {
            fNewMask |= MaskValue as u32;
        }

        let pch2ndQuestion = StrChrW(pszCmdLine.offset(1 as isize), 0x003F_u16);
        if !pch2ndQuestion.is_null() {
            pszCmdLine = pch2ndQuestion.offset(1 as isize);
        }
    }

    let mut szPath: [u16; (2 * MAX_PATH) as usize] = [0u16; (2 * MAX_PATH) as usize];
    let dwFlags: u32 = PPCF_FORCEQUALIFY | PPCF_INCLUDEARGS | PPCF_ADDQUOTES;
    if PathProcessCommandW(pszCmdLine, szPath.as_mut_ptr(), szPath.len() as i32, dwFlags) == -1 {
        StrCpyNW(szPath.as_mut_ptr(), pszCmdLine, szPath.len() as i32);
    }

    let Args = PathGetArgsW(szPath.as_mut_ptr());
    if *Args != 0 {
        *Args.offset(-(1 as isize)) = UNICODE_NULL;
    }

    PathUnquoteSpacesW(szPath.as_mut_ptr());

    let mut execInfo: SHELLEXECUTEINFOW = core::mem::zeroed();
    execInfo.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    execInfo.fMask = fNewMask;
    execInfo.hwnd = hwnd;
    execInfo.lpFile = szPath.as_mut_ptr() as PCWSTR;
    execInfo.lpParameters = Args as PCWSTR;
    execInfo.nShow = (nCmdShow as u32) as i32;

    if ShellExecuteExW(&mut execInfo) == 0 {
        let dwError = GetLastError();
        if SHELL_InRunDllProcess() != 0 {
            ExitProcess(dwError);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ShellExec_RunDLLA(
    hwnd: HWND,
    hInstance: HINSTANCE,
    pszCmdLine: PCSTR,
    nCmdShow: i32,
) {
    let strCmdLine = pszCmdLine; // Keep
    ShellExec_RunDLL_Helper(hwnd, hInstance, (strCmdLine).cast(), nCmdShow);
}

#[no_mangle]
pub unsafe extern "system" fn ShellExec_RunDLLW(
    hwnd: HWND,
    hInstance: HINSTANCE,
    pszCmdLine: PCWSTR,
    nCmdShow: INT,
) {
    ShellExec_RunDLL_Helper(hwnd, hInstance, pszCmdLine, nCmdShow);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *          Shell Library Functions
//  *
//  * Copyright 1998 Marcus Meissner
//  * Copyright 2002 Eric Pouech
//  * Copyright 2018-2024 Katayama Hirofumi MZ
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
// #include "precomp.h"
// #include <winbase_undoc.h>
// #include <undocshell.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(exec);
//
// extern "C" BOOL PathIsExeW(LPCWSTR lpszPath);
//
// #define SEE_MASK_CLASSALL (SEE_MASK_CLASSNAME | SEE_MASK_CLASSKEY)
//
// typedef UINT_PTR (*SHELL_ExecuteW32)(const WCHAR *lpCmd, WCHAR *env, BOOL shWait,
//                 const SHELLEXECUTEINFOW *sei, LPSHELLEXECUTEINFOW sei_out);
//
// static int Win32ErrFromHInst(HINSTANCE hInst)
// {
//     if ((SIZE_T)hInst > 32)
//         return ERROR_SUCCESS;
//     switch ((SIZE_T)hInst)
//     {
//     case SE_ERR_FNF:
//     case SE_ERR_PNF:
//     case SE_ERR_ACCESSDENIED:
//     case SE_ERR_OOM:
//         return (UINT)(SIZE_T)hInst;
//     case SE_ERR_SHARE:
//         return ERROR_SHARING_VIOLATION;
//     case SE_ERR_DDETIMEOUT:
//     case SE_ERR_DDEFAIL:
//     case SE_ERR_DDEBUSY:
//         return ERROR_DDE_FAIL;
//     case SE_ERR_DLLNOTFOUND:
//         return ERROR_DLL_NOT_FOUND;
//     //case SE_ERR_ASSOCINCOMPLETE: Note: Windows treats this as a success code
//     case SE_ERR_NOASSOC:
//         return ERROR_NO_ASSOCIATION;
//     case 10:
//         return ERROR_OLD_WIN_VERSION;
//     case 12:
//         return ERROR_APP_WRONG_OS;
//     case 15:
//         return ERROR_RMODE_APP;
//     case 16:
//         return ERROR_SINGLE_INSTANCE_APP;
//     case 20:
//         return ERROR_INVALID_DLL;
//     }
//     return -1;
// }
//
// // Is the current process a rundll32.exe?
// static BOOL SHELL_InRunDllProcess(VOID)
// {
//     WCHAR szModule[MAX_PATH];
//     static INT s_bInDllProcess = -1;
//
//     if (s_bInDllProcess != -1)
//         return s_bInDllProcess;
//
//     s_bInDllProcess = GetModuleFileNameW(NULL, szModule, _countof(szModule)) &&
//                       (StrStrIW(PathFindFileNameW(szModule), L"rundll") != NULL);
//     return s_bInDllProcess;
// }
//
// static UINT_PTR InvokeOpenWith(HWND hWndOwner, SHELLEXECUTEINFOW &sei)
// {
//     extern HRESULT SH32_InvokeOpenWith(PCWSTR, LPCMINVOKECOMMANDINFO, HANDLE *);
//
//     HANDLE *phProc = (sei.fMask & SEE_MASK_NOCLOSEPROCESS) ? &sei.hProcess : NULL;
//     UINT fCmic = (sei.fMask & SEE_CMIC_COMMON_BASICFLAGS) | CMIC_MASK_FLAG_NO_UI;
//     CMINVOKECOMMANDINFO ici = { sizeof(ici), fCmic, hWndOwner };
//     ici.nShow = SW_SHOW;
//     HRESULT hr = SH32_InvokeOpenWith(sei.lpFile, &ici, phProc);
//     SetLastError(ERROR_NO_ASSOCIATION);
//     return SUCCEEDED(hr) ? 42 : SE_ERR_NOASSOC;
// }
//
// static HRESULT InvokeShellExecuteHook(PCWSTR pszClsid, LPSHELLEXECUTEINFOW pSEI)
// {
//     CComPtr<IUnknown> pUnk;
//     if (FAILED(SHExtCoCreateInstance(pszClsid, NULL, NULL, IID_PPV_ARG(IUnknown, &pUnk))))
//         return S_FALSE;
//     CComPtr<IShellExecuteHookW> pWide;
//     if (SUCCEEDED(pUnk->QueryInterface(IID_PPV_ARG(IShellExecuteHookW, &pWide))))
//         return pWide->Execute(pSEI);
//     HRESULT hr = S_FALSE;
//     return hr;
// }
//
// static HRESULT TryShellExecuteHooks(LPSHELLEXECUTEINFOW pSEI)
// {
//     // https://devblogs.microsoft.com/oldnewthing/20080910-00/?p=20933 claims hooks
//     // were removed in Vista but this is incorrect, they are disabled by default.
//     // https://groups.google.com/g/microsoft.public.platformsdk.shell/c/ixdOX1--IKk
//     // says they are now controlled by the EnableShellExecuteHooks policy.
//     if (pSEI->fMask & SEE_MASK_NO_HOOKS)
//         return S_FALSE;
//     if (LOBYTE(GetVersion()) >= 6 && !SH32_InternalRestricted(REST_SH32_ENABLESHELLEXECUTEHOOKS))
//         return S_FALSE;
//
//     HRESULT hr = S_FALSE;
//     HKEY hKey;
//     LRESULT res = RegOpenKeyExW(HKEY_LOCAL_MACHINE, REGSTR_PATH_EXPLORER L"\\ShellExecuteHooks", 0, KEY_READ, &hKey);
//     if (res != ERROR_SUCCESS)
//         return S_FALSE;
//     for (UINT i = 0; hr == S_FALSE; ++i)
//     {
//         WCHAR szClsid[42];
//         DWORD cch = _countof(szClsid);
//         if (RegEnumValueW(hKey, i, szClsid, &cch, NULL, NULL, NULL, NULL) != ERROR_SUCCESS)
//             break;
//         hr = InvokeShellExecuteHook(szClsid, pSEI);
//     }
//     RegCloseKey(hKey);
//     return hr;
// }
//
// static void ParseNoTildeEffect(PWSTR &res, LPCWSTR &args, DWORD &len, DWORD &used, int argNum)
// {
//     bool firstCharQuote = false;
//     bool quotes_opened = false;
//     bool backslash_encountered = false;
//
//     for (int curArg = 0; curArg <= argNum && *args; ++curArg)
//     {
//         firstCharQuote = false;
//         if (*args == '"')
//         {
//             quotes_opened = true;
//             firstCharQuote = true;
//             args++;
//         }
//
//         while(*args)
//         {
//             if (*args == '\\')
//             {
//                 // if we found a backslash then flip the variable
//                 backslash_encountered = !backslash_encountered;
//             }
//             else if (*args == '"')
//             {
//                 if (quotes_opened)
//                 {
//                     if (*(args + 1) != '"')
//                     {
//                         quotes_opened = false;
//                         args++;
//                         break;
//                     }
//                     else
//                     {
//                         args++;
//                     }
//                 }
//                 else
//                 {
//                     quotes_opened = true;
//                 }
//
//                 backslash_encountered = false;
//             }
//             else
//             {
//                 backslash_encountered = false;
//                 if (*args == ' ' && !firstCharQuote)
//                     break;
//             }
//
//             if (curArg == argNum)
//             {
//                 used++;
//                 if (used < len)
//                     *res++ = *args;
//             }
//
//             args++;
//         }
//
//         while(*args == ' ')
//             ++args;
//     }
// }
//
// static void ParseTildeEffect(PWSTR &res, LPCWSTR &args, DWORD &len, DWORD &used, int argNum)
// {
//     bool quotes_opened = false;
//     bool backslash_encountered = false;
//
//     for (int curArg = 0; curArg <= argNum && *args; ++curArg)
//     {
//         while(*args)
//         {
//             if (*args == '\\')
//             {
//                 // if we found a backslash then flip the variable
//                 backslash_encountered = !backslash_encountered;
//             }
//             else if (*args == '"')
//             {
//                 if (quotes_opened)
//                 {
//                     if (*(args + 1) != '"')
//                     {
//                         quotes_opened = false;
//                     }
//                     else
//                     {
//                         args++;
//                     }
//                 }
//                 else
//                 {
//                     quotes_opened = true;
//                 }
//
//                 backslash_encountered = false;
//             }
//             else
//             {
//                 backslash_encountered = false;
//                 if (*args == ' ' && !quotes_opened && curArg != argNum)
//                     break;
//             }
//
//             if (curArg == argNum)
//             {
//                 used++;
//                 if (used < len)
//                     *res++ = *args;
//             }
//
//             args++;
//         }
//     }
// }
//
// /***********************************************************************
//  * SHELL_ArgifyW [Internal]
//  *
//  * this function is supposed to expand the escape sequences found in the registry
//  * some diving reported that the following were used:
//  * + %1, %2...  seem to report to parameter of index N in ShellExecute pmts
//  * %1 file
//  * %2 printer
//  * %3 driver
//  * %4 port
//  * %I address of a global item ID (explorer switch /idlist)
//  * %L seems to be %1 as long filename followed by the 8+3 variation
//  * %S ???
//  * %W Working directory
//  * %V Use either %L or %W
//  * %* all following parameters (see batfile)
//  *
//  * The way we parse the command line arguments was determined through extensive
//  * testing and can be summed up by the following rules"
//  *
//  * - %2
//  *     - if first letter is " break on first non literal " and include any white spaces
//  *     - if first letter is NOT " break on first " or white space
//  *     - if " is opened any pair of consecutive " results in ONE literal "
//  *
//  * - %~2
//  *     - use rules from here http://www.autohotkey.net/~deleyd/parameters/parameters.htm
//  */
//
// static BOOL SHELL_ArgifyW(WCHAR* out, DWORD len, const WCHAR* fmt, const WCHAR* lpFile, LPITEMIDLIST pidl, LPCWSTR args, DWORD* out_len, const WCHAR* lpDir)
// {
//     BOOL    done = FALSE;
//     BOOL    found_p1 = FALSE;
//     PWSTR   res = out;
//     DWORD   used = 0;
//     bool    tildeEffect = false;
//
//     TRACE("Before parsing: %p, %d, %s, %s, %p, %p\n", out, len, debugstr_w(fmt),
//           debugstr_w(lpFile), pidl, args);
//
//     while (*fmt)
//     {
//         if (*fmt == '%')
//         {
//             switch (*++fmt)
//             {
//                 case '\0':
//                 case '%':
//                 {
//                     used++;
//                     if (used < len)
//                         *res++ = '%';
//                 };
//                 break;
//
//                 case '*':
//                 {
//                     if (args)
//                     {
//                         if (*fmt == '*')
//                         {
//                             used++;
//                             while(*args)
//                             {
//                                 used++;
//                                 if (used < len)
//                                     *res++ = *args++;
//                                 else
//                                     args++;
//                             }
//                             used++;
//                             break;
//                         }
//                     }
//                 };
//                 break;
//
//                 case '~':
//
//                 case '2':
//                 case '3':
//                 case '4':
//                 case '5':
//                 case '6':
//                 case '7':
//                 case '8':
//                 case '9':
//                     //case '0':
//                 {
//                     if (*fmt == '~')
//                     {
//                         fmt++;
//                         tildeEffect = true;
//                     }
//
//                     if (args)
//                     {
//                         if (tildeEffect)
//                         {
//                             ParseTildeEffect(res, args, len, used, *fmt - '2');
//                             tildeEffect = false;
//                         }
//                         else
//                         {
//                             ParseNoTildeEffect(res, args, len, used, *fmt - '2');
//                         }
//                     }
//                 };
//                 break;
//
//                 case '1':
//                     if ((!done || (*fmt == '1')) && lpFile)
//                     {
//                         SIZE_T filelen = wcslen(lpFile);
//                         used += filelen;
//                         if (used < len)
//                         {
//                             wcscpy(res, lpFile);
//                             res += filelen;
//                         }
//                     }
//                     found_p1 = TRUE;
//                     break;
//
//                     /*
//                      * IE uses this a lot for activating things such as windows media
//                      * player. This is not verified to be fully correct but it appears
//                      * to work just fine.
//                      */
//                 case 'l':
//                 case 'L':
//                     if (lpFile)
//                     {
//                         used += wcslen(lpFile);
//                         if (used < len)
//                         {
//                             wcscpy(res, lpFile);
//                             res += wcslen(lpFile);
//                         }
//                     }
//                     found_p1 = TRUE;
//                     break;
//
//                 case 'w':
//                 case 'W':
//                     if (lpDir)
//                     {
//                         used += wcslen(lpDir);
//                         if (used < len)
//                         {
//                             wcscpy(res, lpDir);
//                             res += wcslen(lpDir);
//                         }
//                     }
//                     break;
//
//                 case 'v':
//                 case 'V':
//                     if (lpFile)
//                     {
//                         used += wcslen(lpFile);
//                         if (used < len)
//                         {
//                             wcscpy(res, lpFile);
//                             res += wcslen(lpFile);
//                         }
//                         found_p1 = TRUE;
//                     }
//                     else if (lpDir)
//                     {
//                         used += wcslen(lpDir);
//                         if (used < len)
//                         {
//                             wcscpy(res, lpDir);
//                             res += wcslen(lpDir);
//                         }
//                     }
//                     break;
//
//                 case 'i':
//                 case 'I':
//                     if (pidl)
//                     {
//                         DWORD chars = 0;
//                         /* %p should not exceed 8, maybe 16 when looking forward to 64bit.
//                             * allowing a buffer of 100 should more than exceed all needs */
//                         WCHAR buf[100];
//                         LPVOID  pv;
//                         HGLOBAL hmem = SHAllocShared(pidl, ILGetSize(pidl), 0);
//                         pv = SHLockShared(hmem, 0);
//                         chars = swprintf(buf, L":%p", pv);
//
//                         if (chars >= ARRAY_SIZE(buf))
//                             ERR("pidl format buffer too small!\n");
//
//                         used += chars;
//
//                         if (used < len)
//                         {
//                             wcscpy(res, buf);
//                             res += chars;
//                         }
//                         SHUnlockShared(pv);
//                     }
//                     found_p1 = TRUE;
//                     break;
//
//                 default:
//                     /*
//                      * Check if this is an env-variable here...
//                      */
//
//                     /* Make sure that we have at least one more %.*/
//                     if (strchrW(fmt, '%'))
//                     {
//                         WCHAR   tmpBuffer[1024];
//                         PWSTR   tmpB = tmpBuffer;
//                         WCHAR   tmpEnvBuff[MAX_PATH];
//                         DWORD   envRet;
//
//                         while (*fmt != '%')
//                             *tmpB++ = *fmt++;
//                         *tmpB++ = 0;
//
//                         TRACE("Checking %s to be an env-var\n", debugstr_w(tmpBuffer));
//
//                         envRet = GetEnvironmentVariableW(tmpBuffer, tmpEnvBuff, MAX_PATH);
//                         if (envRet == 0 || envRet > MAX_PATH)
//                         {
//                             used += wcslen(tmpBuffer);
//                             if (used < len)
//                             {
//                                 wcscpy( res, tmpBuffer );
//                                 res += wcslen(tmpBuffer);
//                             }
//                         }
//                         else
//                         {
//                             used += wcslen(tmpEnvBuff);
//                             if (used < len)
//                             {
//                                 wcscpy( res, tmpEnvBuff );
//                                 res += wcslen(tmpEnvBuff);
//                             }
//                         }
//                     }
//                     done = TRUE;
//                     break;
//             }
//             /* Don't skip past terminator (catch a single '%' at the end) */
//             if (*fmt != '\0')
//             {
//                 fmt++;
//             }
//         }
//         else
//         {
//             used ++;
//             if (used < len)
//                 *res++ = *fmt++;
//             else
//                 fmt++;
//         }
//     }
//
//     used++;
//     if (res - out < static_cast<int>(len))
//         *res = '\0';
//     else
//         out[len-1] = '\0';
//
//     TRACE("used %i of %i space\n", used, len);
//     if (out_len)
//         *out_len = used;
//
//     TRACE("After parsing: %p, %d, %s, %s, %p, %p\n", out, len, debugstr_w(fmt),
//           debugstr_w(lpFile), pidl, args);
//
//     return found_p1;
// }
//
// static HRESULT SHELL_GetPathFromIDListForExecuteW(LPCITEMIDLIST pidl, LPWSTR pszPath, UINT uOutSize)
// {
//     STRRET strret;
//     CComPtr<IShellFolder> desktop;
//
//     HRESULT hr = SHGetDesktopFolder(&desktop);
//
//     if (SUCCEEDED(hr))
//     {
//         hr = desktop->GetDisplayNameOf(pidl, SHGDN_FORPARSING, &strret);
//
//         if (SUCCEEDED(hr))
//             StrRetToStrNW(pszPath, uOutSize, &strret, pidl);
//     }
//
//     return hr;
// }
//
// static HWND SHELL_GetUsableDialogOwner(HWND hWnd)
// {
//     // Explicitly block the shell desktop listview from becoming the owner (IContextMenu calling ShellExecute)
//     HWND hProgman = GetShellWindow();
//     if (hWnd && IsWindowVisible(hWnd) && hWnd != GetDesktopWindow() &&
//         hWnd != hProgman && !IsChild(hProgman, hWnd))
//     {
//         return hWnd;
//     }
//     return NULL;
// }
//
// /*************************************************************************
//  *    PromptAndRunProcessAs [Internal]
//  */
// typedef struct _RUNASDLGDATA
// {
//     LPWSTR Name, Domain;
//     BOOL Safer;
//     UINT LogonFlags;
//     WCHAR NameBuffer[MAX_PATH];
//     WCHAR Password[MAX_PATH];
// } RUNASDLGDATA;
//
// static
// INT_PTR
// __stdcall
// RunAsDlgProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     RUNASDLGDATA *pData = (RUNASDLGDATA *)GetWindowLongPtrW(hwnd, DWLP_USER);
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             SetWindowLongPtrW(hwnd, DWLP_USER, (LPARAM)(pData = (RUNASDLGDATA *)lParam));
//             SendDlgItemMessageW(hwnd, IDC_RUNAS_NAME, EM_LIMITTEXT, _countof(pData->NameBuffer)-1, 0);
//             SendDlgItemMessageW(hwnd, IDC_RUNAS_PWD, EM_LIMITTEXT, _countof(pData->Password)-1, 0);
//             SendDlgItemMessageW(hwnd, IDC_RUNAS_OTHER, BM_CLICK, 0, 0);
//
//             HWND hCtl = GetDlgItem(hwnd, IDC_RUNAS_CURRENT);
//             WCHAR fmtbuf[200], buf[_countof(fmtbuf) + _countof(pData->NameBuffer)];
//             DWORD cch = _countof(pData->NameBuffer);
//             if (GetUserNameW(pData->NameBuffer, &cch))
//             {
//                 SendMessageW(hCtl, WM_GETTEXT, _countof(fmtbuf), (LPARAM)buf);
//                 StringCchPrintfW(fmtbuf, _countof(fmtbuf), buf, L"(%s)"); // Change "Blah blah %s" to "Blah blah (%s)"
//                 StringCchPrintfW(buf, _countof(buf), fmtbuf, pData->NameBuffer);
//                 SendMessageW(hCtl, WM_SETTEXT, 0, (LPARAM)buf);
//                 SendDlgItemMessageW(hwnd, IDC_RUNAS_NAME, CB_ADDSTRING, 0, (LPARAM)pData->NameBuffer);
//             }
//             SendDlgItemMessageW(hwnd, IDC_RUNAS_NAME, CB_SETCURSEL, 0, 0);
//             return TRUE;
//         }
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDCANCEL:
//                     EndDialog(hwnd, IDCANCEL);
//                     break;
//
//                 case IDOK:
//                 {
//                     SendDlgItemMessageW(hwnd, IDC_RUNAS_NAME, WM_GETTEXT, _countof(pData->NameBuffer), (LPARAM)pData->NameBuffer);
//                     SendDlgItemMessageW(hwnd, IDC_RUNAS_PWD, WM_GETTEXT, _countof(pData->Password), (LPARAM)pData->Password);
//                     pData->Name = pData->NameBuffer;
//                     pData->Domain = wcsstr(pData->Name, L"\\");
//                     if (pData->Domain)
//                     {
//                         LPWSTR tmp = pData->Domain + 1;
//                         pData->Domain[0] = UNICODE_NULL;
//                         pData->Domain = pData->Name;
//                         pData->Name = tmp;
//                     }
//                     pData->LogonFlags = GetKeyState(VK_SHIFT) < 0 ? LOGON_NETCREDENTIALS_ONLY : LOGON_WITH_PROFILE;
//                     pData->Safer = IsDlgButtonChecked(hwnd, IDC_RUNAS_SAFER);
//                     if (!IsDlgButtonChecked(hwnd, IDC_RUNAS_OTHER))
//                         pData->Name = NULL;
//                     EndDialog(hwnd, IDOK);
//                     break;
//                 }
//
//                 case IDC_RUNAS_BROWSE:
//                     return SHELL_ErrorBox(hwnd, ERROR_NOT_SUPPORTED); // TODO
//             }
//             break;
//     }
//     return FALSE;
// }
//
// static
// HRESULT
// PromptAndRunProcessAs(
//      HWND hwnd,
//      LPWSTR Cmd,
//      DWORD CreationFlags,
//      LPWSTR Env,
//      LPCWSTR Dir,
//      STARTUPINFOW *pSI,
//      PROCESS_INFORMATION *pPI)
// {
//     RUNASDLGDATA data;
//     UINT error;
//     INT_PTR dlgret;
//     hwnd = SHELL_GetUsableDialogOwner(hwnd);
//
// again:
//     dlgret = DialogBoxParamW(shell32_hInstance, MAKEINTRESOURCEW(IDD_RUN_AS),
//                              hwnd, RunAsDlgProc, (LPARAM)&data);
//     if (dlgret == IDOK && data.Name)
//     {
//         if (CreateProcessWithLogonW(data.Name, data.Domain, data.Password, data.LogonFlags,
//                                     NULL, Cmd, CreationFlags, Env, Dir, pSI, pPI))
//             error = ERROR_SUCCESS;
//         else
//             error = GetLastError();
//         switch (error)
//         {
//             case ERROR_LOGON_FAILURE:
//             case ERROR_NO_SUCH_USER:
//             case ERROR_INVALID_ACCOUNT_NAME:
//             case ERROR_ACCOUNT_DISABLED:
//             case ERROR_ACCOUNT_RESTRICTION:
//             case ERROR_INVALID_LOGON_HOURS:
//             case ERROR_PASSWORD_EXPIRED:
//                 SHELL_ErrorBox(hwnd, error);
//                 goto again;
//         }
//     }
//     else if (dlgret == IDOK)
//     {
//         // TODO: Use the Safer API if requested to
//         if (CreateProcessW(NULL, Cmd, NULL, NULL, FALSE, CreationFlags, Env, Dir, pSI, pPI))
//             error = ERROR_SUCCESS;
//         else
//             error = GetLastError();
//     }
//     else if (dlgret == IDCANCEL)
//     {
//         SetLastError(ERROR_CANCELLED);
//         pPI->hProcess = NULL;
//         return S_FALSE;
//     }
//     else
//     {
//         pPI->hProcess = NULL;
//         return E_FAIL;
//     }
//     SecureZeroMemory(&data, sizeof(data));
//     return HRESULT_FROM_WIN32(error);
// }
//
// /*************************************************************************
//  *    SHELL_ExecuteW [Internal]
//  *
//  */
// static UINT_PTR SHELL_ExecuteW(const WCHAR *lpCmd, WCHAR *env, BOOL shWait,
//                                const SHELLEXECUTEINFOW *psei, LPSHELLEXECUTEINFOW psei_out)
// {
//     STARTUPINFOW  startup;
//     PROCESS_INFORMATION info;
//     UINT_PTR retval = SE_ERR_NOASSOC;
//     UINT gcdret = 0, lasterror = 0;
//     WCHAR curdir[MAX_PATH];
//     DWORD dwCreationFlags = CREATE_UNICODE_ENVIRONMENT;
//     const WCHAR *lpDirectory = NULL;
//
//     TRACE("Execute %s from directory %s\n", debugstr_w(lpCmd), debugstr_w(psei->lpDirectory));
//
//     /* make sure we don't fail the CreateProcess if the calling app passes in
//      * a bad working directory */
//     if (!StrIsNullOrEmpty(psei->lpDirectory))
//     {
//         DWORD attr = GetFileAttributesW(psei->lpDirectory);
//         if (attr != INVALID_FILE_ATTRIBUTES && attr & FILE_ATTRIBUTE_DIRECTORY)
//             lpDirectory = psei->lpDirectory;
//     }
//
//     /* ShellExecute specifies the command from psei->lpDirectory
//      * if present. Not from the current dir as CreateProcess does */
//     if (lpDirectory)
//         if ((gcdret = GetCurrentDirectoryW( MAX_PATH, curdir)))
//             if (!SetCurrentDirectoryW( lpDirectory))
//                 ERR("cannot set directory %s\n", debugstr_w(lpDirectory));
//
//     ZeroMemory(&startup, sizeof(STARTUPINFOW));
//     startup.cb = sizeof(STARTUPINFOW);
//     startup.dwFlags = STARTF_USESHOWWINDOW;
//     startup.wShowWindow = psei->nShow;
//     if (!(psei->fMask & SEE_MASK_NO_CONSOLE))
//         dwCreationFlags |= CREATE_NEW_CONSOLE;
//     if (psei->fMask & SEE_MASK_FLAG_SEPVDM)
//         dwCreationFlags |= CREATE_SEPARATE_WOW_VDM;
//     startup.lpTitle = (LPWSTR)(psei->fMask & (SEE_MASK_HASLINKNAME | SEE_MASK_HASTITLE) ? psei->lpClass : NULL);
//
//     if (psei->fMask & SEE_MASK_HASLINKNAME)
//         startup.dwFlags |= STARTF_TITLEISLINKNAME;
//
//     if (psei->fMask & SEE_MASK_HOTKEY)
//     {
//         startup.hStdInput = UlongToHandle(psei->dwHotKey);
//         startup.dwFlags |= STARTF_USEHOTKEY;
//     }
//
//     if (psei->fMask & SEE_MASK_ICON) // hIcon has higher precedence than hMonitor
//     {
//         startup.hStdOutput = psei->hIcon;
//         startup.dwFlags |= STARTF_SHELLPRIVATE;
//     }
//     else if ((psei->fMask & SEE_MASK_HMONITOR) || psei->hwnd)
//     {
//         if (psei->fMask & SEE_MASK_HMONITOR)
//             startup.hStdOutput = psei->hMonitor;
//         else if (psei->hwnd)
//             startup.hStdOutput = MonitorFromWindow(psei->hwnd, MONITOR_DEFAULTTONEAREST);
//         if (startup.hStdOutput)
//             startup.dwFlags |= STARTF_SHELLPRIVATE;
//     }
//
//     BOOL createdProcess;
//     if (psei->lpVerb && !StrCmpIW(L"runas", psei->lpVerb))
//     {
//         HRESULT hr = PromptAndRunProcessAs(psei->hwnd, (LPWSTR)lpCmd, dwCreationFlags,
//                                                  env, lpDirectory, &startup, &info);
//         createdProcess = hr == S_OK;
//         if (hr == S_FALSE)
//         {
//             retval = 33; // Pretend cancel is success.
//             goto done;
//         }
//     }
//     else
//     {
//         createdProcess = CreateProcessW(NULL, (LPWSTR)lpCmd, NULL, NULL, FALSE,
//                                         dwCreationFlags, env, lpDirectory, &startup, &info);
//     }
//
//     if (createdProcess)
//     {
//         /* Give 30 seconds to the app to come up, if desired. Probably only needed
//            when starting app immediately before making a DDE connection. */
//         if (shWait)
//             if (WaitForInputIdle(info.hProcess, 30000) == WAIT_FAILED)
//                 WARN("WaitForInputIdle failed: Error %d\n", GetLastError() );
//         retval = 33;
//
//         if (psei->fMask & SEE_MASK_NOCLOSEPROCESS)
//             psei_out->hProcess = info.hProcess;
//         else
//             CloseHandle( info.hProcess );
//         CloseHandle( info.hThread );
//     }
//     else if ((retval = lasterror = GetLastError()) >= 32)
//     {
//         WARN("CreateProcess returned error %ld\n", retval);
//         retval = ERROR_BAD_FORMAT;
//     }
//
// done:
//     TRACE("returning %lu\n", retval);
//     psei_out->hInstApp = (HINSTANCE)retval;
//
//     if (gcdret)
//     {
//         if (!SetCurrentDirectoryW(curdir))
//             ERR("cannot return to directory %s\n", debugstr_w(curdir));
//         if (lasterror)
//             RestoreLastError(lasterror);
//     }
//     return retval;
// }
//
//
// /***********************************************************************
//  *           SHELL_BuildEnvW    [Internal]
//  *
//  * Build the environment for the new process, adding the specified
//  * path to the PATH variable. Returned pointer must be freed by caller.
//  */
// static LPWSTR SHELL_BuildEnvW( const WCHAR *path )
// {
//     CHeapPtr<WCHAR, CLocalAllocator> new_env;
//     WCHAR *strings, *p, *p2;
//     int total = wcslen(path) + 1;
//     BOOL got_path = FALSE;
//
//     if (!(strings = GetEnvironmentStringsW())) return NULL;
//     p = strings;
//     while (*p)
//     {
//         int len = wcslen(p) + 1;
//         if (!_wcsnicmp( p, L"PATH=", 5 )) got_path = TRUE;
//         total += len;
//         p += len;
//     }
//     if (!got_path) total += 5;  /* we need to create PATH */
//     total++;  /* terminating null */
//
//     if (!new_env.Allocate(total))
//     {
//         FreeEnvironmentStringsW(strings);
//         return NULL;
//     }
//     p = strings;
//     p2 = new_env;
//     while (*p)
//     {
//         int len = wcslen(p) + 1;
//         memcpy(p2, p, len * sizeof(WCHAR));
//         if (!_wcsnicmp( p, L"PATH=", 5 ))
//         {
//             p2[len - 1] = ';';
//             wcscpy( p2 + len, path );
//             p2 += wcslen(path) + 1;
//         }
//         p += len;
//         p2 += len;
//     }
//     if (!got_path)
//     {
//         wcscpy(p2, L"PATH=");
//         wcscat(p2, path);
//         p2 += wcslen(p2) + 1;
//     }
//     *p2 = 0;
//     FreeEnvironmentStringsW(strings);
//     return new_env.Detach();
// }
//
// /***********************************************************************
//  *           SHELL_TryAppPathW    [Internal]
//  *
//  * Helper function for SHELL_FindExecutable
//  * @param lpResult - pointer to a buffer of size MAX_PATH
//  * On entry: szName is a filename (probably without path separators).
//  * On exit: if szName found in "App Path", place full path in lpResult, and return true
//  */
// static BOOL SHELL_TryAppPathW( LPCWSTR szName, LPWSTR lpResult, WCHAR **env)
// {
//     HKEY hkApp = NULL;
//     WCHAR buffer[1024];
//     DWORD len, dwType;
//     LONG res;
//     BOOL found = FALSE;
//
//     if (env) *env = NULL;
//     wcscpy(buffer, L"Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\");
//     wcscat(buffer, szName);
//     res = RegOpenKeyExW(HKEY_LOCAL_MACHINE, buffer, 0, KEY_READ, &hkApp);
//     if (res)
//     {
//         // Add ".exe" extension, if extension does not exists
//         if (PathAddExtensionW(buffer, L".exe"))
//         {
//             res = RegOpenKeyExW(HKEY_LOCAL_MACHINE, buffer, 0, KEY_READ, &hkApp);
//         }
//         if (res) goto end;
//     }
//
//     len = MAX_PATH * sizeof(WCHAR);
//     res = SHRegQueryValueExW(hkApp, NULL, NULL, &dwType, (LPBYTE)lpResult, &len);
//     if (res != ERROR_SUCCESS || dwType != REG_SZ)
//         goto end;
//
//     found = TRUE;
//
//     if (env)
//     {
//         len = sizeof(buffer);
//         res = SHRegQueryValueExW(hkApp, L"Path", NULL, &dwType, (LPBYTE)buffer, &len);
//         if (res == ERROR_SUCCESS && dwType == REG_SZ && buffer[0])
//             *env = SHELL_BuildEnvW(buffer);
//     }
//
// end:
//     if (hkApp) RegCloseKey(hkApp);
//     return found;
// }
//
// /*************************************************************************
//  *     SHELL_FindExecutableByVerb [Internal]
//  *
//  * called from SHELL_FindExecutable or SHELL_execute_class
//  * in/out:
//  *      classname a buffer, big enough, to get the key name to do actually the
//  *              command   "WordPad.Document.1\\shell\\open\\command"
//  *              passed as "WordPad.Document.1"
//  * in:
//  *      lpVerb the operation on it (open)
//  *      commandlen the size of command buffer (in bytes)
//  * out:
//  *      command a buffer, to store the command to do the
//  *              operation on the file
//  *      key a buffer, big enough, to get the key name to do actually the
//  *              command "WordPad.Document.1\\shell\\open\\command"
//  *              Can be NULL
//  */
// static UINT SHELL_FindExecutableByVerb(LPCWSTR lpVerb, LPWSTR key, LPWSTR classname, LPWSTR command, LONG commandlen)
// {
//     HKEY hkeyClass;
//     WCHAR verb[MAX_PATH];
//
//     if (RegOpenKeyExW(HKEY_CLASSES_ROOT, classname, 0, KEY_READ, &hkeyClass))
//         return SE_ERR_NOASSOC;
//     if (!HCR_GetDefaultVerbW(hkeyClass, lpVerb, verb, ARRAY_SIZE(verb)))
//         return SE_ERR_NOASSOC;
//     RegCloseKey(hkeyClass);
//
//     /* Looking for ...buffer\shell\<verb>\command */
//     wcscat(classname, L"\\shell\\"); // FIXME: Use HCR_GetExecuteCommandW or AssocAPI
//     wcscat(classname, verb);
//     wcscat(classname, L"\\command");
//
//     if (RegQueryValueW(HKEY_CLASSES_ROOT, classname, command,
//                        &commandlen) == ERROR_SUCCESS)
//     {
//         commandlen /= sizeof(WCHAR);
//         if (key) wcscpy(key, classname);
//
//         command[commandlen] = '\0';
//
//         return 33; /* FIXME see SHELL_FindExecutable() */
//     }
//
//     return SE_ERR_NOASSOC;
// }
//
// /*************************************************************************
//  *    SHELL_FindExecutable [Internal]
//  *
//  * Utility for code sharing between FindExecutable and ShellExecute
//  * in:
//  *      lpFile the name of a file
//  *      lpVerb the operation on it (open)
//  * out:
//  *      lpResult a buffer, big enough :-(, to store the command to do the
//  *              operation on the file
//  *      key a buffer, big enough, to get the key name to do actually the
//  *              command (it'll be used afterwards for more information
//  *              on the operation)
//  */
// static UINT SHELL_FindExecutable(LPCWSTR lpPath, LPCWSTR lpFile, LPCWSTR lpVerb,
//                                  LPWSTR lpResult, DWORD resultLen, LPWSTR key, WCHAR **env, LPITEMIDLIST pidl, LPCWSTR args)
// {
//     WCHAR *extension = NULL; /* pointer to file extension */
//     WCHAR classname[256];     /* registry name for this file type */
//     LONG  classnamelen = sizeof(classname); /* length of above */
//     WCHAR command[1024];     /* command from registry */
//     WCHAR wBuffer[256];      /* Used to GetProfileString */
//     UINT  retval = SE_ERR_NOASSOC;
//     WCHAR *tok;              /* token pointer */
//     WCHAR xlpFile[MAX_PATH]; /* result of PathResolve */
//     DWORD attribs;           /* file attributes */
//     WCHAR curdir[MAX_PATH];
//     const WCHAR *search_paths[3] = {0};
//
//     TRACE("%s\n", debugstr_w(lpFile));
//
//     if (!lpResult)
//         return ERROR_INVALID_PARAMETER;
//
//     xlpFile[0] = '\0';
//     lpResult[0] = '\0'; /* Start off with an empty return string */
//     if (key) *key = '\0';
//
//     /* trap NULL parameters on entry */
//     if (!lpFile)
//     {
//         WARN("(lpFile=%s,lpResult=%s): NULL parameter\n",
//              debugstr_w(lpFile), debugstr_w(lpResult));
//         return ERROR_FILE_NOT_FOUND; /* File not found. Close enough, I guess. */
//     }
//
//     if (SHELL_TryAppPathW( lpFile, lpResult, env ))
//     {
//         TRACE("found %s via App Paths\n", debugstr_w(lpResult));
//         return 33;
//     }
//
//     GetCurrentDirectoryW(ARRAY_SIZE(curdir), curdir);
//     if (lpPath && *lpPath)
//     {
//         search_paths[0] = lpPath;
//         search_paths[1] = curdir;
//     }
//     else
//     {
//         search_paths[0] = curdir;
//     }
//
//     lstrcpyW(xlpFile, lpFile);
//     if (PathResolveW(xlpFile, search_paths, PRF_TRYPROGRAMEXTENSIONS | PRF_VERIFYEXISTS) ||
//         PathFindOnPathW(xlpFile, search_paths))
//     {
//         TRACE("PathResolveW returned non-zero\n");
//         lpFile = xlpFile;
//         PathRemoveBlanksW(xlpFile);
//
//         /* Clear any trailing periods */
//         SIZE_T i = wcslen(xlpFile);
//         while (i > 0 && xlpFile[i - 1] == '.')
//         {
//             xlpFile[--i] = '\0';
//         }
//
//         lstrcpyW(lpResult, xlpFile);
//         /* The file was found in lpPath or one of the directories in the system-wide search path */
//     }
//     else
//     {
//         xlpFile[0] = '\0';
//     }
//
//     attribs = GetFileAttributesW(lpFile);
//     if (attribs != INVALID_FILE_ATTRIBUTES && (attribs & FILE_ATTRIBUTE_DIRECTORY))
//     {
//         wcscpy(classname, L"Folder");
//     }
//     else
//     {
//         /* Did we get something? Anything? */
//         if (xlpFile[0] == 0)
//         {
//             TRACE("Returning SE_ERR_FNF\n");
//             return SE_ERR_FNF;
//         }
//         /* First thing we need is the file's extension */
//         extension = wcsrchr(xlpFile, '.'); /* Assume last "." is the one; */
//         /* File->Run in progman uses */
//         /* .\FILE.EXE :( */
//         TRACE("xlpFile=%s,extension=%s\n", debugstr_w(xlpFile), debugstr_w(extension));
//
//         if (extension == NULL || extension[1] == 0)
//         {
//             WARN("Returning SE_ERR_NOASSOC\n");
//             return SE_ERR_NOASSOC;
//         }
//
//         /* Three places to check: */
//         /* 1. win.ini, [windows], programs (NB no leading '.') */
//         /* 2. Registry, HKEY_CLASS_ROOT\<classname>\shell\open\command */
//         /* 3. win.ini, [extensions], extension (NB no leading '.' */
//         /* All I know of the order is that registry is checked before */
//         /* extensions; however, it'd make sense to check the programs */
//         /* section first, so that's what happens here. */
//
//         /* See if it's a program - if GetProfileString fails, we skip this
//          * section. Actually, if GetProfileString fails, we've probably
//          * got a lot more to worry about than running a program... */
//         if (GetProfileStringW(L"windows", L"programs", L"exe pif bat cmd com", wBuffer, ARRAY_SIZE(wBuffer)) > 0)
//         {
//             CharLowerW(wBuffer);
//             tok = wBuffer;
//             while (*tok)
//             {
//                 WCHAR *p = tok;
//                 while (*p && *p != ' ' && *p != '\t') p++;
//                 if (*p)
//                 {
//                     *p++ = 0;
//                     while (*p == ' ' || *p == '\t') p++;
//                 }
//
//                 if (_wcsicmp(tok, &extension[1]) == 0) /* have to skip the leading "." */
//                 {
//                     wcscpy(lpResult, xlpFile);
//                     /* Need to perhaps check that the file has a path
//                      * attached */
//                     TRACE("found %s\n", debugstr_w(lpResult));
//                     return 33;
//                     /* Greater than 32 to indicate success */
//                 }
//                 tok = p;
//             }
//         }
//
//         /* Check registry */
//         if (RegQueryValueW(HKEY_CLASSES_ROOT, extension, classname,
//                            &classnamelen) == ERROR_SUCCESS)
//         {
//             classnamelen /= sizeof(WCHAR);
//             if (classnamelen == ARRAY_SIZE(classname))
//                 classnamelen--;
//
//             classname[classnamelen] = '\0';
//             TRACE("File type: %s\n", debugstr_w(classname));
//         }
//         else
//         {
//             *classname = '\0';
//         }
//     }
//
//     if (*classname)
//     {
//         /* pass the verb string to SHELL_FindExecutableByVerb() */
//         retval = SHELL_FindExecutableByVerb(lpVerb, key, classname, command, sizeof(command));
//
//         if (retval > 32)
//         {
//             DWORD finishedLen;
//             SHELL_ArgifyW(lpResult, resultLen, command, xlpFile, pidl, args, &finishedLen, lpPath);
//             if (finishedLen > resultLen)
//                 ERR("Argify buffer not large enough.. truncated\n");
//             /* Remove double quotation marks and command line arguments */
//             if (*lpResult == '"')
//             {
//                 WCHAR *p = lpResult;
//                 while (*(p + 1) != '"')
//                 {
//                     *p = *(p + 1);
//                     p++;
//                 }
//                 *p = '\0';
//             }
//             else
//             {
//                 /* Truncate on first space */
//                 WCHAR *p = lpResult;
//                 while (*p != ' ' && *p != '\0')
//                     p++;
//                 *p = '\0';
//             }
//         }
//     }
//     else /* Check win.ini */
//     {
//         /* Toss the leading dot */
//         extension++;
//         if (GetProfileStringW(L"extensions", extension, L"", command, ARRAY_SIZE(command)) > 0)
//         {
//             if (wcslen(command) != 0)
//             {
//                 wcscpy(lpResult, command);
//                 tok = wcschr(lpResult, '^'); /* should be ^.extension? */
//                 if (tok != NULL)
//                 {
//                     tok[0] = '\0';
//                     wcscat(lpResult, xlpFile); /* what if no dir in xlpFile? */
//                     tok = wcschr(command, '^'); /* see above */
//                     if ((tok != NULL) && (wcslen(tok) > 5))
//                     {
//                         wcscat(lpResult, &tok[5]);
//                     }
//                 }
//                 retval = 33; /* FIXME - see above */
//             }
//         }
//     }
//
//     TRACE("returning path %s, retval %d\n", debugstr_w(lpResult), retval);
//     return retval;
// }
//
// /******************************************************************
//  *        dde_cb
//  *
//  * callback for the DDE connection. not really useful
//  */
// static HDDEDATA __stdcall dde_cb(UINT uType, UINT uFmt, HCONV hConv,
//                                 HSZ hsz1, HSZ hsz2, HDDEDATA hData,
//                                 ULONG_PTR dwData1, ULONG_PTR dwData2)
// {
//     TRACE("dde_cb: %04x, %04x, %p, %p, %p, %p, %08lx, %08lx\n",
//           uType, uFmt, hConv, hsz1, hsz2, hData, dwData1, dwData2);
//     return NULL;
// }
//
// /******************************************************************
//  *        dde_connect
//  *
//  * ShellExecute helper. Used to do an operation with a DDE connection
//  *
//  * Handles both the direct connection (try #1), and if it fails,
//  * launching an application and trying (#2) to connect to it
//  *
//  */
// static unsigned dde_connect(const WCHAR* key, const WCHAR* start, WCHAR* ddeexec,
//                             const WCHAR* lpFile, WCHAR *env,
//                             LPCWSTR szCommandline, LPITEMIDLIST pidl, SHELL_ExecuteW32 execfunc,
//                             const SHELLEXECUTEINFOW *psei, LPSHELLEXECUTEINFOW psei_out)
// {
//     WCHAR       regkey[256];
//     WCHAR *     endkey = regkey + wcslen(key);
//     WCHAR       app[256], topic[256], ifexec[256], static_res[256];
//     CHeapPtr<WCHAR, CLocalAllocator> dynamic_res;
//     WCHAR *     res;
//     LONG        applen, topiclen, ifexeclen;
//     WCHAR *     exec;
//     DWORD       ddeInst = 0;
//     DWORD       tid;
//     DWORD       resultLen, endkeyLen;
//     HSZ         hszApp, hszTopic;
//     HCONV       hConv;
//     HDDEDATA    hDdeData;
//     unsigned    ret = SE_ERR_NOASSOC;
//     BOOL unicode = !(GetVersion() & 0x80000000);
//
//     if (strlenW(key) + 1 > ARRAY_SIZE(regkey))
//     {
//         FIXME("input parameter %s larger than buffer\n", debugstr_w(key));
//         return 2;
//     }
//     wcscpy(regkey, key);
//     endkeyLen = ARRAY_SIZE(regkey) - (endkey - regkey);
//     if (strlenW(L"\\application") + 1 > endkeyLen)
//     {
//         FIXME("endkey %s overruns buffer\n", debugstr_w(L"\\application"));
//         return 2;
//     }
//     wcscpy(endkey, L"\\application");
//     applen = sizeof(app);
//     if (RegQueryValueW(HKEY_CLASSES_ROOT, regkey, app, &applen) != ERROR_SUCCESS)
//     {
//         WCHAR command[1024], fullpath[MAX_PATH];
//         LPWSTR ptr = NULL;
//         DWORD ret = 0;
//
//         /* Get application command from start string and find filename of application */
//         if (*start == '"')
//         {
//             if (strlenW(start + 1) + 1 > ARRAY_SIZE(command))
//             {
//                 FIXME("size of input parameter %s larger than buffer\n",
//                       debugstr_w(start + 1));
//                 return 2;
//             }
//             wcscpy(command, start + 1);
//             if ((ptr = wcschr(command, '"')))
//                 * ptr = 0;
//             ret = SearchPathW(NULL, command, L".exe", ARRAY_SIZE(fullpath), fullpath, &ptr);
//         }
//         else
//         {
//             LPCWSTR p;
//             LPWSTR space;
//             for (p = start; (space = const_cast<LPWSTR>(strchrW(p, ' '))); p = space + 1)
//             {
//                 int idx = space - start;
//                 memcpy(command, start, idx * sizeof(WCHAR));
//                 command[idx] = '\0';
//                 if ((ret = SearchPathW(NULL, command, L".exe", ARRAY_SIZE(fullpath), fullpath, &ptr)))
//                     break;
//             }
//             if (!ret)
//                 ret = SearchPathW(NULL, start, L".exe", ARRAY_SIZE(fullpath), fullpath, &ptr);
//         }
//
//         if (!ret)
//         {
//             ERR("Unable to find application path for command %s\n", debugstr_w(start));
//             return ERROR_ACCESS_DENIED;
//         }
//         if (strlenW(ptr) + 1 > ARRAY_SIZE(app))
//         {
//             FIXME("size of found path %s larger than buffer\n", debugstr_w(ptr));
//             return 2;
//         }
//         wcscpy(app, ptr);
//
//         /* Remove extensions (including .so) */
//         ptr = app + wcslen(app) - 3;
//         if (ptr > app && !wcscmp(ptr, L".so"))
//             *ptr = 0;
//
//         ptr = const_cast<LPWSTR>(strrchrW(app, '.'));
//         assert(ptr);
//         *ptr = 0;
//     }
//
//     if (strlenW(L"\\topic") + 1 > endkeyLen)
//     {
//         FIXME("endkey %s overruns buffer\n", debugstr_w(L"\\topic"));
//         return 2;
//     }
//     wcscpy(endkey, L"\\topic");
//     topiclen = sizeof(topic);
//     if (RegQueryValueW(HKEY_CLASSES_ROOT, regkey, topic, &topiclen) != ERROR_SUCCESS)
//     {
//         wcscpy(topic, L"System");
//     }
//
//     if (unicode)
//     {
//         if (DdeInitializeW(&ddeInst, dde_cb, APPCMD_CLIENTONLY, 0L) != DMLERR_NO_ERROR)
//             return 2;
//     }
//     else
//     {
//         if (DdeInitializeA(&ddeInst, dde_cb, APPCMD_CLIENTONLY, 0L) != DMLERR_NO_ERROR)
//             return 2;
//     }
//
//     hszApp = DdeCreateStringHandleW(ddeInst, app, CP_WINUNICODE);
//     hszTopic = DdeCreateStringHandleW(ddeInst, topic, CP_WINUNICODE);
//
//     hConv = DdeConnect(ddeInst, hszApp, hszTopic, NULL);
//     exec = ddeexec;
//     if (!hConv)
//     {
//         TRACE("Launching %s\n", debugstr_w(start));
//         ret = execfunc(start, env, TRUE, psei, psei_out);
//         if (ret <= 32)
//         {
//             TRACE("Couldn't launch\n");
//             goto error;
//         }
//         /* if ddeexec is NULL, then we just need to exit here */
//         if (ddeexec == NULL)
//         {
//             TRACE("Exiting because ddeexec is NULL. ret=42.\n");
//             /* See https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew */
//             /* for reason why we use 42 here and also "Shell32_apitest ShellExecuteW" regression test */
//             return 42;
//         }
//         /* if ddeexec is 'empty string', then we just need to exit here */
//         if (wcscmp(ddeexec, L"") == 0)
//         {
//             TRACE("Exiting because ddeexec is 'empty string'. ret=42.\n");
//             /* See https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew */
//             /* for reason why we use 42 here and also "Shell32_apitest ShellExecuteW" regression test */
//             return 42;
//         }
//         hConv = DdeConnect(ddeInst, hszApp, hszTopic, NULL);
//         if (!hConv)
//         {
//             TRACE("Couldn't connect. ret=%d\n", ret);
//             DdeUninitialize(ddeInst);
//             SetLastError(ERROR_DDE_FAIL);
//             return 30; /* whatever */
//         }
//         if (strlenW(L"\\ifexec") + 1 > endkeyLen)
//         {
//             FIXME("endkey %s overruns buffer\n", debugstr_w(L"\\ifexec"));
//             return 2;
//         }
//         strcpyW(endkey, L"\\ifexec");
//         ifexeclen = sizeof(ifexec);
//         if (RegQueryValueW(HKEY_CLASSES_ROOT, regkey, ifexec, &ifexeclen) == ERROR_SUCCESS)
//         {
//             exec = ifexec;
//         }
//     }
//
//     SHELL_ArgifyW(static_res, ARRAY_SIZE(static_res), exec, lpFile, pidl, szCommandline, &resultLen, NULL);
//     if (resultLen > ARRAY_SIZE(static_res))
//     {
//         dynamic_res.Allocate(resultLen);
//         res = dynamic_res;
//         SHELL_ArgifyW(dynamic_res, resultLen, exec, lpFile, pidl, szCommandline, NULL, NULL);
//     }
//     else
//         res = static_res;
//     TRACE("%s %s => %s\n", debugstr_w(exec), debugstr_w(lpFile), debugstr_w(res));
//
//     /* It's documented in the KB 330337 that IE has a bug and returns
//      * error DMLERR_NOTPROCESSED on XTYP_EXECUTE request.
//      */
//     if (unicode)
//         hDdeData = DdeClientTransaction((LPBYTE)res, (strlenW(res) + 1) * sizeof(WCHAR), hConv, 0L, 0, XTYP_EXECUTE, 30000, &tid);
//     else
//     {
//         DWORD lenA = WideCharToMultiByte(CP_ACP, 0, res, -1, NULL, 0, NULL, NULL);
//         CHeapPtr<char, CLocalAllocator> resA;
//         resA.Allocate(lenA);
//         WideCharToMultiByte(CP_ACP, 0, res, -1, resA, lenA, NULL, NULL);
//         hDdeData = DdeClientTransaction( (LPBYTE)(LPSTR)resA, lenA, hConv, 0L, 0,
//                                          XTYP_EXECUTE, 10000, &tid );
//     }
//     if (hDdeData)
//         DdeFreeDataHandle(hDdeData);
//     else
//         WARN("DdeClientTransaction failed with error %04x\n", DdeGetLastError(ddeInst));
//     ret = 33;
//
//     DdeDisconnect(hConv);
//
// error:
//     DdeUninitialize(ddeInst);
//
//     return ret;
// }
//
// /*************************************************************************
//  *    execute_from_key [Internal]
//  */
// static UINT_PTR execute_from_key(LPCWSTR key, LPCWSTR lpFile, WCHAR *env,
//                                  LPCWSTR szCommandline, LPCWSTR executable_name,
//                                  SHELL_ExecuteW32 execfunc,
//                                  LPSHELLEXECUTEINFOW psei, LPSHELLEXECUTEINFOW psei_out)
// {
//     WCHAR cmd[256], param[1024], ddeexec[256];
//     DWORD cmdlen = sizeof(cmd), ddeexeclen = sizeof(ddeexec);
//     UINT_PTR retval = SE_ERR_NOASSOC;
//     DWORD resultLen;
//     LPWSTR tmp;
//
//     TRACE("%s %s %s %s %s\n", debugstr_w(key), debugstr_w(lpFile), debugstr_w(env),
//           debugstr_w(szCommandline), debugstr_w(executable_name));
//
//     cmd[0] = '\0';
//     param[0] = '\0';
//
//     /* Get the application from the registry */
//     if (RegQueryValueW(HKEY_CLASSES_ROOT, key, cmd, (LONG *)&cmdlen) == ERROR_SUCCESS)
//     {
//         TRACE("got cmd: %s\n", debugstr_w(cmd));
//
//         /* Is there a replace() function anywhere? */
//         cmdlen /= sizeof(WCHAR);
//         if (cmdlen >= ARRAY_SIZE(cmd))
//             cmdlen = ARRAY_SIZE(cmd) - 1;
//         cmd[cmdlen] = '\0';
//         SHELL_ArgifyW(param, ARRAY_SIZE(param), cmd, lpFile, (LPITEMIDLIST)psei->lpIDList, szCommandline, &resultLen,
//                       (psei->lpDirectory && *psei->lpDirectory) ? psei->lpDirectory : NULL);
//         if (resultLen > ARRAY_SIZE(param))
//             ERR("Argify buffer not large enough, truncating\n");
//     }
//
//     /* Get the parameters needed by the application
//        from the associated ddeexec key */
//     tmp = const_cast<LPWSTR>(strstrW(key, L"command"));
//     assert(tmp);
//     wcscpy(tmp, L"ddeexec");
//
//     if (RegQueryValueW(HKEY_CLASSES_ROOT, key, ddeexec, (LONG *)&ddeexeclen) == ERROR_SUCCESS)
//     {
//         TRACE("Got ddeexec %s => %s\n", debugstr_w(key), debugstr_w(ddeexec));
//         if (!param[0]) strcpyW(param, executable_name);
//         retval = dde_connect(key, param, ddeexec, lpFile, env, szCommandline, (LPITEMIDLIST)psei->lpIDList, execfunc, psei, psei_out);
//     }
//     else if (param[0])
//     {
//         TRACE("executing: %s\n", debugstr_w(param));
//         retval = execfunc(param, env, FALSE, psei, psei_out);
//     }
//     else
//         WARN("Nothing appropriate found for %s\n", debugstr_w(key));
//
//     return retval;
// }
//
// /*************************************************************************
//  * FindExecutableA            [SHELL32.@]
//  */
// HINSTANCE WINAPI FindExecutableA(LPCSTR lpFile, LPCSTR lpDirectory, LPSTR lpResult)
// {
//     HINSTANCE retval;
//     WCHAR *wFile = NULL, *wDirectory = NULL;
//     WCHAR wResult[MAX_PATH];
//
//     if (lpFile) __SHCloneStrAtoW(&wFile, lpFile);
//     if (lpDirectory) __SHCloneStrAtoW(&wDirectory, lpDirectory);
//
//     retval = FindExecutableW(wFile, wDirectory, wResult);
//     WideCharToMultiByte(CP_ACP, 0, wResult, -1, lpResult, MAX_PATH, NULL, NULL);
//     SHFree(wFile);
//     SHFree(wDirectory);
//
//     TRACE("returning %s\n", lpResult);
//     return retval;
// }
//
// /*************************************************************************
//  * FindExecutableW            [SHELL32.@]
//  *
//  * This function returns the executable associated with the specified file
//  * for the default verb.
//  *
//  * PARAMS
//  *  lpFile   [I] The file to find the association for. This must refer to
//  *               an existing file otherwise FindExecutable fails and returns
//  *               SE_ERR_FNF.
//  *  lpResult [O] Points to a buffer into which the executable path is
//  *               copied. This parameter must not be NULL otherwise
//  *               FindExecutable() segfaults. The buffer must be of size at
//  *               least MAX_PATH characters.
//  *
//  * RETURNS
//  *  A value greater than 32 on success, less than or equal to 32 otherwise.
//  *  See the SE_ERR_* constants.
//  *
//  * NOTES
//  *  On Windows XP and 2003, FindExecutable() seems to first convert the
//  *  filename into 8.3 format, thus taking into account only the first three
//  *  characters of the extension, and expects to find an association for those.
//  *  However other Windows versions behave sanely.
//  */
// HINSTANCE WINAPI FindExecutableW(LPCWSTR lpFile, LPCWSTR lpDirectory, LPWSTR lpResult)
// {
//     UINT_PTR retval;
//     WCHAR old_dir[MAX_PATH], res[MAX_PATH];
//     DWORD cch = _countof(res);
//     LPCWSTR dirs[2];
//
//     TRACE("File %s, Dir %s\n", debugstr_w(lpFile), debugstr_w(lpDirectory));
//
//     *lpResult = UNICODE_NULL;
//
//     GetCurrentDirectoryW(_countof(old_dir), old_dir);
//
//     if (lpDirectory && *lpDirectory)
//     {
//         SetCurrentDirectoryW(lpDirectory);
//         dirs[0] = lpDirectory;
//     }
//     else
//     {
//         dirs[0] = old_dir;
//     }
//     dirs[1] = NULL;
//
//     if (!GetShortPathNameW(lpFile, res, _countof(res)))
//         StringCchCopyW(res, _countof(res), lpFile);
//
//     if (PathResolveW(res, dirs, PRF_TRYPROGRAMEXTENSIONS | PRF_FIRSTDIRDEF))
//     {
//         // NOTE: The last parameter of this AssocQueryStringW call is "strange" in Windows.
//         if (PathIsExeW(res) ||
//             SUCCEEDED(AssocQueryStringW(ASSOCF_NONE, ASSOCSTR_EXECUTABLE, res, NULL, res, &cch)))
//         {
//             StringCchCopyW(lpResult, MAX_PATH, res);
//             retval = 42;
//         }
//         else
//         {
//             retval = SE_ERR_NOASSOC;
//         }
//     }
//     else
//     {
//         retval = SE_ERR_FNF;
//     }
//
//     TRACE("returning %s\n", debugstr_w(lpResult));
//     SetCurrentDirectoryW(old_dir);
//     return (HINSTANCE)retval;
// }
//
// /* FIXME: is this already implemented somewhere else? */
// static HKEY ShellExecute_GetClassKey(const SHELLEXECUTEINFOW *sei)
// {
//     if ((sei->fMask & SEE_MASK_CLASSALL) == SEE_MASK_CLASSKEY)
//         return sei->hkeyClass;
//
//     HKEY hKey = NULL;
//     if (sei->fMask & SEE_MASK_CLASSNAME)
//     {
//         TRACE("class = %s\n", debugstr_w(sei->lpClass));
//         RegOpenKeyExW(HKEY_CLASSES_ROOT, sei->lpClass, 0, KEY_READ, &hKey);
//         return hKey;
//     }
//     PCWSTR ext = PathFindExtensionW(sei->lpFile);
//     TRACE("ext = %s\n", debugstr_w(ext));
//     if (!StrIsNullOrEmpty(ext) && SUCCEEDED(HCR_GetProgIdKeyOfExtension(ext, &hKey, FALSE)))
//         return hKey;
//     return NULL;
// }
//
// static HRESULT shellex_get_dataobj( LPSHELLEXECUTEINFOW sei, CComPtr<IDataObject>& dataObj)
// {
//     CComHeapPtr<ITEMIDLIST> allocatedPidl;
//     LPITEMIDLIST pidl = NULL;
//
//     if (sei->fMask & SEE_MASK_CLASSALL) // FIXME: This makes no sense? SEE_MASK_IDLIST?
//     {
//         pidl = (LPITEMIDLIST)sei->lpIDList;
//     }
//     else
//     {
//         WCHAR fullpath[MAX_PATH];
//         BOOL ret;
//
//         fullpath[0] = 0;
//         ret = GetFullPathNameW(sei->lpFile, MAX_PATH, fullpath, NULL);
//         if (!ret)
//             return HRESULT_FROM_WIN32(GetLastError());
//
//         pidl = ILCreateFromPathW(fullpath);
//         allocatedPidl.Attach(pidl);
//     }
//     return SHELL_GetUIObjectOfAbsoluteItem(NULL, pidl, IID_PPV_ARG(IDataObject, &dataObj));
// }
//
// static HRESULT shellex_run_context_menu_default(IShellExtInit *obj,
//         LPSHELLEXECUTEINFOW sei)
// {
//     CComPtr<IContextMenu> cm = NULL;
//     CMINVOKECOMMANDINFOEX ici;
//     MENUITEMINFOW info;
//     WCHAR string[0x80];
//     INT i, n, def = -1;
//     HMENU hmenu = 0;
//     HRESULT r;
//
//     TRACE("%p %p\n", obj, sei);
//
//     r = obj->QueryInterface(IID_PPV_ARG(IContextMenu, &cm));
//     if (FAILED(r))
//         return r;
//
//     hmenu = CreateMenu();
//     if (!hmenu)
//         goto end;
//
//     /* the number of the last menu added is returned in r */
//     r = cm->QueryContextMenu(hmenu, 0, 0x20, 0x7fff, CMF_DEFAULTONLY);
//     if (FAILED(r))
//         goto end;
//
//     n = GetMenuItemCount(hmenu);
//     for (i = 0; i < n; i++)
//     {
//         memset(&info, 0, sizeof(info));
//         info.cbSize = sizeof info;
//         info.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_STATE | MIIM_DATA | MIIM_ID;
//         info.dwTypeData = string;
//         info.cch = sizeof string;
//         string[0] = 0;
//         GetMenuItemInfoW(hmenu, i, TRUE, &info);
//
//         TRACE("menu %d %s %08x %08lx %08x %08x\n", i, debugstr_w(string),
//               info.fState, info.dwItemData, info.fType, info.wID);
//         if ((!sei->lpVerb && (info.fState & MFS_DEFAULT)) ||
//             (sei->lpVerb && !lstrcmpiW(sei->lpVerb, string)))
//         {
//             def = i;
//             break;
//         }
//     }
//
//     r = E_FAIL;
//     if (def == -1)
//         goto end;
//
//     memset(&ici, 0, sizeof ici);
//     ici.cbSize = sizeof ici;
//     ici.fMask = (sei->fMask & SEE_CMIC_COMMON_BASICFLAGS) | CMIC_MASK_UNICODE;
//     ici.nShow = sei->nShow;
//     ici.lpVerb = MAKEINTRESOURCEA(def);
//     ici.hwnd = sei->hwnd;
//     ici.lpParametersW = sei->lpParameters;
//
//     r = cm->InvokeCommand((LPCMINVOKECOMMANDINFO)&ici);
//
//     TRACE("invoke command returned %08x\n", r);
//
// end:
//     if (hmenu)
//         DestroyMenu( hmenu );
//     return r;
// }
//
// static HRESULT shellex_load_object_and_run(HKEY hkey, LPCGUID guid, LPSHELLEXECUTEINFOW sei)
// {
//     TRACE("%p %s %p\n", hkey, debugstr_guid(guid), sei);
//
//     CCoInit coInit;
//
//     if (FAILED_UNEXPECTEDLY(coInit.hr))
//         return coInit.hr;
//
//     CComPtr<IShellExtInit> obj;
//     HRESULT hr = CoCreateInstance(*guid, NULL, CLSCTX_INPROC_SERVER,
//                          IID_PPV_ARG(IShellExtInit, &obj));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IDataObject> dataobj;
//     hr = shellex_get_dataobj(sei, dataobj);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = obj->Initialize(NULL, dataobj, hkey);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IObjectWithSite> ows;
//     hr = obj->QueryInterface(IID_PPV_ARG(IObjectWithSite, &ows));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     ows->SetSite(NULL);
//
//     return shellex_run_context_menu_default(obj, sei);
// }
//
// static HRESULT shellex_get_contextmenu(LPSHELLEXECUTEINFOW sei, CComPtr<IContextMenu>& cm)
// {
//     CComHeapPtr<ITEMIDLIST> allocatedPidl;
//     LPITEMIDLIST pidl = NULL;
//
//     if (sei->lpIDList)
//     {
//         pidl = (LPITEMIDLIST)sei->lpIDList;
//     }
//     else
//     {
//         SFGAOF sfga = 0;
//         HRESULT hr = SHParseDisplayName(sei->lpFile, NULL, &allocatedPidl, SFGAO_STORAGECAPMASK, &sfga);
//         if (FAILED(hr))
//         {
//             WCHAR Buffer[MAX_PATH] = {};
//             // FIXME: MAX_PATH.....
//             UINT retval = SHELL_FindExecutable(sei->lpDirectory, sei->lpFile, sei->lpVerb, Buffer, _countof(Buffer), NULL, NULL, NULL, sei->lpParameters);
//             if (retval <= 32)
//                 return HRESULT_FROM_WIN32(retval);
//
//             hr = SHParseDisplayName(Buffer, NULL, &allocatedPidl, SFGAO_STORAGECAPMASK, &sfga);
//             // This should not happen, we found it...
//             if (FAILED_UNEXPECTEDLY(hr))
//                 return hr;
//         }
//
//         pidl = allocatedPidl;
//     }
//
//     CComPtr<IShellFolder> shf;
//     LPCITEMIDLIST pidllast = NULL;
//     HRESULT hr = SHBindToParent(pidl, IID_PPV_ARG(IShellFolder, &shf), &pidllast);
//     if (FAILED(hr))
//         return hr;
//
//     return shf->GetUIObjectOf(NULL, 1, &pidllast, IID_NULL_PPV_ARG(IContextMenu, &cm));
// }
//
// static HRESULT ShellExecute_ContextMenuVerb(LPSHELLEXECUTEINFOW sei)
// {
//     TRACE("%p\n", sei);
//
//     CCoInit coInit;
//
//     if (FAILED_UNEXPECTEDLY(coInit.hr))
//         return coInit.hr;
//
//     CComPtr<IContextMenu> cm;
//     HRESULT hr = shellex_get_contextmenu(sei, cm);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComHeapPtr<char> verb, parameters, dir;
//     __SHCloneStrWtoA(&verb, sei->lpVerb);
//     __SHCloneStrWtoA(&parameters, sei->lpParameters);
//     __SHCloneStrWtoA(&dir, sei->lpDirectory);
//
//     BOOL fDefault = StrIsNullOrEmpty(sei->lpVerb);
//     CMINVOKECOMMANDINFOEX ici = { sizeof(ici) };
//     ici.fMask = SeeFlagsToCmicFlags(sei->fMask) | CMIC_MASK_UNICODE;
//     ici.nShow = sei->nShow;
//     if (!fDefault)
//     {
//         ici.lpVerb = verb;
//         ici.lpVerbW = sei->lpVerb;
//     }
//     ici.hwnd = sei->hwnd;
//     ici.lpParameters = parameters;
//     ici.lpParametersW = sei->lpParameters;
//     ici.lpDirectory = dir;
//     ici.lpDirectoryW = sei->lpDirectory;
//     ici.dwHotKey = sei->dwHotKey;
//     ici.hIcon = sei->hIcon;
//     if (ici.fMask & (CMIC_MASK_HASLINKNAME | CMIC_MASK_HASTITLE))
//         ici.lpTitleW = sei->lpClass;
//
//     enum { idFirst = 1, idLast = 0x7fff };
//     HMENU hMenu = CreatePopupMenu();
//     // Note: Windows does not pass CMF_EXTENDEDVERBS so "hidden" verbs cannot be executed
//     hr = cm->QueryContextMenu(hMenu, 0, idFirst, idLast, fDefault ? CMF_DEFAULTONLY : 0);
//     if (!FAILED_UNEXPECTEDLY(hr))
//     {
//         if (fDefault)
//         {
//             INT uDefault = GetMenuDefaultItem(hMenu, FALSE, 0);
//             uDefault = (uDefault != -1) ? uDefault - idFirst : 0;
//             ici.lpVerb = MAKEINTRESOURCEA(uDefault);
//             ici.lpVerbW = MAKEINTRESOURCEW(uDefault);
//         }
//
//         hr = cm->InvokeCommand((LPCMINVOKECOMMANDINFO)&ici);
//         if (!FAILED_UNEXPECTEDLY(hr))
//             hr = S_OK;
//     }
//
//     DestroyMenu(hMenu);
//
//     return hr;
// }
//
//
// /*************************************************************************
//  *    ShellExecute_FromContextMenu [Internal]
//  */
// static LONG ShellExecute_FromContextMenuHandlers( LPSHELLEXECUTEINFOW sei )
// {
//     HKEY hkey, hkeycm = 0;
//     WCHAR szguid[39];
//     HRESULT hr;
//     GUID guid;
//     DWORD i;
//     LONG r;
//
//     TRACE("%s\n", debugstr_w(sei->lpFile));
//
//     hkey = ShellExecute_GetClassKey(sei);
//     if (!hkey)
//         return ERROR_FUNCTION_FAILED;
//
//     // FIXME: Words cannot describe how broken this is, all of it needs to die
//     r = RegOpenKeyW(hkey, L"shellex\\ContextMenuHandlers", &hkeycm);
//     if (r == ERROR_SUCCESS)
//     {
//         i = 0;
//         while (1)
//         {
//             r = RegEnumKeyW(hkeycm, i++, szguid, ARRAY_SIZE(szguid));
//             if (r != ERROR_SUCCESS)
//                 break;
//
//             hr = CLSIDFromString(szguid, &guid);
//             if (SUCCEEDED(hr))
//             {
//                 /* stop at the first one that succeeds in running */
//                 hr = shellex_load_object_and_run(hkey, &guid, sei);
//                 if (SUCCEEDED(hr))
//                     break;
//             }
//         }
//         RegCloseKey(hkeycm);
//     }
//
//     if (hkey != sei->hkeyClass)
//         RegCloseKey(hkey);
//     return r;
// }
//
// static UINT_PTR SHELL_quote_and_execute(LPCWSTR wcmd, LPCWSTR wszParameters, LPCWSTR wszKeyname, LPCWSTR wszApplicationName, LPWSTR env, LPSHELLEXECUTEINFOW psei, LPSHELLEXECUTEINFOW psei_out, SHELL_ExecuteW32 execfunc);
//
// static UINT_PTR SHELL_execute_class(LPCWSTR wszApplicationName, LPSHELLEXECUTEINFOW psei, LPSHELLEXECUTEINFOW psei_out, SHELL_ExecuteW32 execfunc)
// {
//     WCHAR execCmd[1024], classname[1024];
//     /* launch a document by fileclass like 'WordPad.Document.1' */
//     /* the Commandline contains 'c:\Path\wordpad.exe "%1"' */
//     /* FIXME: wcmd should not be of a fixed size. Fixed to 1024, MAX_PATH is way too short! */
//     ULONG cmask = (psei->fMask & SEE_MASK_CLASSALL);
//     DWORD resultLen;
//     BOOL done;
//     UINT_PTR rslt;
//
//     /* FIXME: remove following block when SHELL_quote_and_execute supports hkeyClass parameter */
//     if (cmask != SEE_MASK_CLASSNAME)
//     {
//         WCHAR wcmd[1024];
//         HCR_GetExecuteCommandW((cmask == SEE_MASK_CLASSKEY) ? psei->hkeyClass : NULL,
//                                (cmask == SEE_MASK_CLASSNAME) ? psei->lpClass : NULL,
//                                psei->lpVerb,
//                                execCmd, sizeof(execCmd));
//
//         /* FIXME: get the extension of lpFile, check if it fits to the lpClass */
//         TRACE("SEE_MASK_CLASSNAME->%s, doc->%s\n", debugstr_w(execCmd), debugstr_w(wszApplicationName));
//
//         wcmd[0] = '\0';
//         done = SHELL_ArgifyW(wcmd, ARRAY_SIZE(wcmd), execCmd, wszApplicationName, (LPITEMIDLIST)psei->lpIDList, psei->lpParameters,
//                              &resultLen, (psei->lpDirectory && *psei->lpDirectory) ? psei->lpDirectory : NULL);
//         if (!done && wszApplicationName[0])
//         {
//         }
//         if (resultLen > ARRAY_SIZE(wcmd))
//             ERR("Argify buffer not large enough... truncating\n");
//         return execfunc(wcmd, NULL, FALSE, psei, psei_out);
//     }
//
//     strcpyW(classname, psei->lpClass);
//     rslt = SHELL_FindExecutableByVerb(psei->lpVerb, NULL, classname, execCmd, sizeof(execCmd));
//
//     TRACE("SHELL_FindExecutableByVerb returned %u (%s, %s)\n", (unsigned int)rslt, debugstr_w(classname), debugstr_w(execCmd));
//     if (33 > rslt)
//         return rslt;
//     rslt = SHELL_quote_and_execute( execCmd, L"", classname,
//                                       wszApplicationName, NULL, psei,
//                                       psei_out, execfunc );
//     return rslt;
//
// }
//
// static BOOL SHELL_translate_idlist(LPSHELLEXECUTEINFOW sei, LPWSTR wszParameters, DWORD parametersLen, LPWSTR wszApplicationName, DWORD dwApplicationNameLen)
// {
//     WCHAR buffer[MAX_PATH];
//     BOOL appKnownSingular = FALSE;
//
//     /* last chance to translate IDList: now also allow CLSID paths */
//     if (SUCCEEDED(SHELL_GetPathFromIDListForExecuteW((LPCITEMIDLIST)sei->lpIDList, buffer, ARRAY_SIZE(buffer)))) {
//         if (buffer[0] == ':' && buffer[1] == ':') {
//             /* open shell folder for the specified class GUID */
//             if (strlenW(buffer) + 1 > parametersLen)
//                 ERR("parameters len exceeds buffer size (%i > %i), truncating\n",
//                     lstrlenW(buffer) + 1, parametersLen);
//             lstrcpynW(wszParameters, buffer, parametersLen);
//             if (strlenW(L"explorer.exe") > dwApplicationNameLen)
//                 ERR("application len exceeds buffer size (%i), truncating\n",
//                     dwApplicationNameLen);
//             lstrcpynW(wszApplicationName, L"explorer.exe", dwApplicationNameLen);
//             appKnownSingular = TRUE;
//
//             sei->fMask &= ~SEE_MASK_INVOKEIDLIST;
//         } else {
//             WCHAR target[max(MAX_PATH, _countof(buffer))];
//             DWORD attribs;
//             DWORD resultLen;
//             /* Check if we're executing a directory and if so use the
//                handler for the Folder class */
//             strcpyW(target, buffer);
//             attribs = GetFileAttributesW(buffer);
//             if (attribs != INVALID_FILE_ATTRIBUTES &&
//                     (attribs & FILE_ATTRIBUTE_DIRECTORY) &&
//                     HCR_GetExecuteCommandW(0, L"Folder",
//                                            sei->lpVerb,
//                                            buffer, sizeof(buffer))) {
//                 SHELL_ArgifyW(wszApplicationName, dwApplicationNameLen,
//                               buffer, target, (LPITEMIDLIST)sei->lpIDList, NULL, &resultLen,
//                               !StrIsNullOrEmpty(sei->lpDirectory) ? sei->lpDirectory : NULL);
//                 if (resultLen > dwApplicationNameLen)
//                     ERR("Argify buffer not large enough... truncating\n"); // FIXME: Report this to the caller?
//                 appKnownSingular = FALSE;
//                 // HACKFIX: We really want the !appKnownSingular code in SHELL_execute to split the
//                 // parameters for us but we cannot guarantee that the exe in the registry is quoted.
//                 // We have now turned 'explorer.exe "%1" into 'explorer.exe "c:\path\from\pidl"' and
//                 // need to split to application and parameters.
//                 LPCWSTR params = PathGetArgsW(wszApplicationName);
//                 lstrcpynW(wszParameters, params, parametersLen);
//                 PathRemoveArgsW(wszApplicationName);
//                 PathUnquoteSpacesW(wszApplicationName);
//                 appKnownSingular = TRUE;
//             }
//             sei->fMask &= ~SEE_MASK_INVOKEIDLIST;
//         }
//     }
//     return appKnownSingular;
// }
//
// static BOOL
// SHELL_InvokePidl(
//      LPSHELLEXECUTEINFOW sei,
//      LPCITEMIDLIST pidl)
// {
//     // Bind pidl
//     CComPtr<IShellFolder> psfFolder;
//     LPCITEMIDLIST pidlLast;
//     HRESULT hr = SHBindToParent(pidl, IID_PPV_ARG(IShellFolder, &psfFolder), &pidlLast);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return FALSE;
//
//     // Get the context menu to invoke a command
//     CComPtr<IContextMenu> pCM;
//     hr = psfFolder->GetUIObjectOf(NULL, 1, &pidlLast, IID_NULL_PPV_ARG(IContextMenu, &pCM));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return FALSE;
//
//     // Invoke a command
//     CMINVOKECOMMANDINFO ici = { sizeof(ici) };
//     ici.fMask = (sei->fMask & SEE_CMIC_COMMON_BASICFLAGS) & ~CMIC_MASK_UNICODE; // FIXME: Unicode?
//     ici.nShow = sei->nShow;
//     ici.hwnd = sei->hwnd;
//     char szVerb[VERBKEY_CCHMAX];
//     if (sei->lpVerb && sei->lpVerb[0])
//     {
//         WideCharToMultiByte(CP_ACP, 0, sei->lpVerb, -1, szVerb, _countof(szVerb), NULL, NULL);
//         szVerb[_countof(szVerb) - 1] = ANSI_NULL; // Avoid buffer overrun
//         ici.lpVerb = szVerb;
//     }
//     else // The default verb?
//     {
//         HMENU hMenu = CreatePopupMenu();
//         const INT idCmdFirst = 1, idCmdLast = 0x7FFF;
//         hr = pCM->QueryContextMenu(hMenu, 0, idCmdFirst, idCmdLast, CMF_DEFAULTONLY);
//         if (FAILED_UNEXPECTEDLY(hr))
//         {
//             DestroyMenu(hMenu);
//             return FALSE;
//         }
//
//         INT nDefaultID = GetMenuDefaultItem(hMenu, FALSE, 0);
//         DestroyMenu(hMenu);
//         if (nDefaultID == -1)
//             nDefaultID = idCmdFirst;
//
//         ici.lpVerb = MAKEINTRESOURCEA(nDefaultID - idCmdFirst);
//     }
//     hr = pCM->InvokeCommand(&ici);
//
//     return !FAILED_UNEXPECTEDLY(hr);
// }
//
// static UINT_PTR SHELL_quote_and_execute(LPCWSTR wcmd, LPCWSTR wszParameters, LPCWSTR wszKeyname, LPCWSTR wszApplicationName, LPWSTR env, LPSHELLEXECUTEINFOW psei, LPSHELLEXECUTEINFOW psei_out, SHELL_ExecuteW32 execfunc)
// {
//     UINT_PTR retval;
//     DWORD len;
//     CHeapPtr<WCHAR, CLocalAllocator> wszQuotedCmd;
//
//     /* Length of quotes plus length of command plus NULL terminator */
//     len = 2 + lstrlenW(wcmd) + 1;
//     if (wszParameters[0])
//     {
//         /* Length of space plus length of parameters */
//         len += 1 + lstrlenW(wszParameters);
//     }
//     wszQuotedCmd.Allocate(len);
//     /* Must quote to handle case where cmd contains spaces,
//      * else security hole if malicious user creates executable file "C:\\Program"
//      */
//     strcpyW(wszQuotedCmd, L"\"");
//     strcatW(wszQuotedCmd, wcmd);
//     strcatW(wszQuotedCmd, L"\"");
//     if (wszParameters[0])
//     {
//         strcatW(wszQuotedCmd, L" ");
//         strcatW(wszQuotedCmd, wszParameters);
//     }
//
//     TRACE("%s/%s => %s/%s\n", debugstr_w(wszApplicationName), debugstr_w(psei->lpVerb), debugstr_w(wszQuotedCmd), debugstr_w(wszKeyname));
//
//     if (*wszKeyname)
//         retval = execute_from_key(wszKeyname, wszApplicationName, env, psei->lpParameters, wcmd, execfunc, psei, psei_out);
//     else
//         retval = execfunc(wszQuotedCmd, env, FALSE, psei, psei_out);
//
//     return retval;
// }
//
// static UINT_PTR SHELL_execute_url(LPCWSTR lpFile, LPCWSTR wcmd, LPSHELLEXECUTEINFOW psei, LPSHELLEXECUTEINFOW psei_out, SHELL_ExecuteW32 execfunc)
// {
//     UINT_PTR retval;
//     CHeapPtr<WCHAR, CLocalAllocator> lpstrProtocol;
//     LPCWSTR lpstrRes;
//     INT iSize;
//     DWORD len;
//
//     lpstrRes = strchrW(lpFile, ':');
//     if (lpstrRes)
//         iSize = lpstrRes - lpFile;
//     else
//         iSize = strlenW(lpFile);
//
//     TRACE("Got URL: %s\n", debugstr_w(lpFile));
//     /* Looking for ...<protocol>\shell\<lpVerb>\command */
//     len = iSize + lstrlenW(L"\\shell\\") + lstrlenW(L"\\command") + 1;
//     if (psei->lpVerb && *psei->lpVerb)
//         len += lstrlenW(psei->lpVerb);
//     else
//         len += lstrlenW(L"open"); // FIXME: Use HCR_GetExecuteCommandW or AssocAPI
//     lpstrProtocol.Allocate(len);
//     memcpy(lpstrProtocol, lpFile, iSize * sizeof(WCHAR));
//     lpstrProtocol[iSize] = '\0';
//     strcatW(lpstrProtocol, L"\\shell\\");
//     strcatW(lpstrProtocol, psei->lpVerb && *psei->lpVerb ? psei->lpVerb : L"open");
//     strcatW(lpstrProtocol, L"\\command");
//
//     retval = execute_from_key(lpstrProtocol, lpFile, NULL, psei->lpParameters,
//                               wcmd, execfunc, psei, psei_out);
//
//     return retval;
// }
//
// static void do_error_dialog(UINT_PTR retval, HWND hwnd, PCWSTR filename)
// {
//     WCHAR msg[2048];
//     DWORD_PTR msgArguments[3]  = { (DWORD_PTR)filename, 0, 0 };
//     const DWORD error_code = GetLastError();
//
//     if (retval == SE_ERR_NOASSOC)
//         LoadStringW(shell32_hInstance, IDS_SHLEXEC_NOASSOC, msg, ARRAY_SIZE(msg));
//     else
//         FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY,
//                        NULL,
//                        error_code,
//                        LANG_USER_DEFAULT,
//                        msg,
//                        ARRAY_SIZE(msg),
//                        (va_list*)msgArguments);
//
//     MessageBoxW(hwnd, msg, NULL, MB_ICONERROR);
//     SetLastError(error_code); // Restore
// }
//
// static WCHAR *expand_environment( const WCHAR *str )
// {
//     CHeapPtr<WCHAR, CLocalAllocator> buf;
//     DWORD len;
//
//     len = ExpandEnvironmentStringsW(str, NULL, 0);
//     if (!len) return NULL;
//
//     if (!buf.Allocate(len))
//         return NULL;
//
//     len = ExpandEnvironmentStringsW(str, buf, len);
//     if (!len)
//         return NULL;
//
//     return buf.Detach();
// }
//
// /*************************************************************************
//  *    SHELL_execute [Internal]
//  */
// static BOOL SHELL_execute(LPSHELLEXECUTEINFOW sei, SHELL_ExecuteW32 execfunc)
// {
//     static const DWORD unsupportedFlags =
//         SEE_MASK_CONNECTNETDRV | SEE_MASK_FLAG_DDEWAIT | SEE_MASK_ASYNCOK;
//
//     DWORD len;
//     UINT_PTR retval = SE_ERR_NOASSOC;
//     BOOL appKnownSingular = FALSE;
//
//     /* make a local copy of the LPSHELLEXECUTEINFO structure and work with this from now on */
//     sei->hProcess = NULL;
//     SHELLEXECUTEINFOW sei_tmp = *sei;
//
//     TRACE("mask=0x%08x hwnd=%p verb=%s file=%s parm=%s dir=%s show=0x%08x class=%s\n",
//           sei_tmp.fMask, sei_tmp.hwnd, debugstr_w(sei_tmp.lpVerb),
//           debugstr_w(sei_tmp.lpFile), debugstr_w(sei_tmp.lpParameters),
//           debugstr_w(sei_tmp.lpDirectory), sei_tmp.nShow,
//           ((sei_tmp.fMask & SEE_MASK_CLASSALL) == SEE_MASK_CLASSNAME) ?
//           debugstr_w(sei_tmp.lpClass) : "not used");
//
//     // Call hooks before expanding and resolving strings
//     HRESULT hr = TryShellExecuteHooks(sei);
//     if (hr != S_FALSE)
//     {
//         int err = Win32ErrFromHInst(sei->hInstApp);
//         if (err <= 0)
//         {
//             sei->hInstApp = (HINSTANCE)UlongToHandle(42);
//             return TRUE;
//         }
//         SetLastError(err);
//         return FALSE;
//     }
//
//     /* make copies of all path/command strings */
//     CHeapPtr<WCHAR, CLocalAllocator> wszApplicationName;
//     DWORD dwApplicationNameLen = MAX_PATH + 2;
//     if (!sei_tmp.lpFile)
//     {
//         wszApplicationName.Allocate(dwApplicationNameLen);
//         *wszApplicationName = '\0';
//     }
//     else if (*sei_tmp.lpFile == '\"' && sei_tmp.lpFile[(len = strlenW(sei_tmp.lpFile))-1] == '\"')
//     {
//         if(len-1 >= dwApplicationNameLen)
//             dwApplicationNameLen = len;
//
//         wszApplicationName.Allocate(dwApplicationNameLen);
//         memcpy(wszApplicationName, sei_tmp.lpFile + 1, len * sizeof(WCHAR));
//
//         if(len > 2)
//             wszApplicationName[len-2] = '\0';
//         appKnownSingular = TRUE;
//
//         TRACE("wszApplicationName=%s\n", debugstr_w(wszApplicationName));
//     }
//     else
//     {
//         DWORD l = strlenW(sei_tmp.lpFile) + 1;
//         if(l > dwApplicationNameLen) dwApplicationNameLen = l + 1;
//         wszApplicationName.Allocate(dwApplicationNameLen);
//         memcpy(wszApplicationName, sei_tmp.lpFile, l * sizeof(WCHAR));
//
//         if (wszApplicationName[2] == 0 && wszApplicationName[1] == L':' &&
//             ((L'A' <= wszApplicationName[0] && wszApplicationName[0] <= L'Z') ||
//              (L'a' <= wszApplicationName[0] && wszApplicationName[0] <= L'z')))
//         {
//             // 'C:' --> 'C:\'
//             PathAddBackslashW(wszApplicationName);
//         }
//     }
//
//     WCHAR parametersBuffer[1024];
//     LPWSTR wszParameters = parametersBuffer;
//     CHeapPtr<WCHAR, CLocalAllocator> wszParamAlloc;
//     DWORD parametersLen = _countof(parametersBuffer);
//
//     if (sei_tmp.lpParameters)
//     {
//         len = lstrlenW(sei_tmp.lpParameters) + 1;
//         if (len > parametersLen)
//         {
//             wszParamAlloc.Allocate(len);
//             wszParameters = wszParamAlloc;
//             parametersLen = len;
//         }
//         strcpyW(wszParameters, sei_tmp.lpParameters);
//     }
//     else
//         *wszParameters = L'\0';
//
//     // Get the working directory
//     WCHAR dirBuffer[MAX_PATH];
//     LPWSTR wszDir = dirBuffer;
//     wszDir[0] = UNICODE_NULL;
//     CHeapPtr<WCHAR, CLocalAllocator> wszDirAlloc;
//     if (sei_tmp.lpDirectory && *sei_tmp.lpDirectory)
//     {
//         if (sei_tmp.fMask & SEE_MASK_DOENVSUBST)
//         {
//             LPWSTR tmp = expand_environment(sei_tmp.lpDirectory);
//             if (tmp)
//             {
//                 wszDirAlloc.Attach(tmp);
//                 wszDir = wszDirAlloc;
//             }
//         }
//         else
//         {
//             __SHCloneStrW(&wszDirAlloc, sei_tmp.lpDirectory);
//             if (wszDirAlloc)
//                 wszDir = wszDirAlloc;
//         }
//     }
//     if (!wszDir[0])
//     {
//         ::GetCurrentDirectoryW(_countof(dirBuffer), dirBuffer);
//         wszDir = dirBuffer;
//     }
//     // NOTE: ShellExecute should accept the invalid working directory for historical reason.
//     if (!PathIsDirectoryW(wszDir))
//     {
//         INT iDrive = PathGetDriveNumberW(wszDir);
//         if (iDrive >= 0)
//         {
//             PathStripToRootW(wszDir);
//             if (!PathIsDirectoryW(wszDir))
//             {
//                 ::GetWindowsDirectoryW(dirBuffer, _countof(dirBuffer));
//                 wszDir = dirBuffer;
//             }
//         }
//     }
//
//     /* adjust string pointers to point to the new buffers */
//     sei_tmp.lpFile = wszApplicationName;
//     sei_tmp.lpParameters = wszParameters;
//     sei_tmp.lpDirectory = wszDir;
//
//     if (sei_tmp.fMask & unsupportedFlags)
//     {
//         FIXME("flags ignored: 0x%08x\n", sei_tmp.fMask & unsupportedFlags);
//     }
//
//     /* process the IDList */
//     if (sei_tmp.fMask & SEE_MASK_IDLIST &&
//         (sei_tmp.fMask & SEE_MASK_INVOKEIDLIST) != SEE_MASK_INVOKEIDLIST)
//     {
//         LPCITEMIDLIST pidl = (LPCITEMIDLIST)sei_tmp.lpIDList;
//         hr = SHGetNameAndFlagsW(pidl, SHGDN_FORPARSING, wszApplicationName, dwApplicationNameLen, NULL);
//         if (FAILED(hr))
//         {
//             if (dwApplicationNameLen)
//                 *wszApplicationName = UNICODE_NULL;
//             if (!_ILIsDesktop(pidl))
//                 TRACE("Unable to get PIDL parsing path\n");
//         }
//         appKnownSingular = TRUE;
//         TRACE("-- idlist=%p (%s)\n", sei_tmp.lpIDList, debugstr_w(wszApplicationName));
//     }
//
//     if ((sei_tmp.fMask & SEE_MASK_DOENVSUBST) && !StrIsNullOrEmpty(sei_tmp.lpFile))
//     {
//         WCHAR *tmp = expand_environment(sei_tmp.lpFile);
//         if (tmp)
//         {
//             wszApplicationName.Attach(tmp);
//             sei_tmp.lpFile = wszApplicationName;
//         }
//     }
//
//     if ((sei_tmp.fMask & SEE_MASK_INVOKEIDLIST) == SEE_MASK_INVOKEIDLIST)
//     {
//         hr = ShellExecute_ContextMenuVerb(&sei_tmp);
//         if (SUCCEEDED(hr))
//         {
//             sei->hInstApp = (HINSTANCE)42;
//             return TRUE;
//         }
//     }
//
//     if (ERROR_SUCCESS == ShellExecute_FromContextMenuHandlers(&sei_tmp))
//     {
//         sei->hInstApp = (HINSTANCE) 33;
//         return TRUE;
//     }
//
//     if (sei_tmp.fMask & SEE_MASK_CLASSALL)
//     {
//         retval = SHELL_execute_class(wszApplicationName, &sei_tmp, sei, execfunc);
//         if (retval <= 32 && !(sei_tmp.fMask & SEE_MASK_FLAG_NO_UI))
//         {
//             OPENASINFO Info;
//
//             //FIXME
//             // need full path
//
//             Info.pcszFile = wszApplicationName;
//             Info.pcszClass = NULL;
//             Info.oaifInFlags = OAIF_ALLOW_REGISTRATION | OAIF_EXEC;
//
//             //if (SHOpenWithDialog(sei_tmp.hwnd, &Info) != S_OK)
//             DBG_UNREFERENCED_LOCAL_VARIABLE(Info);
//             do_error_dialog(retval, sei_tmp.hwnd, wszApplicationName);
//         }
//         return retval > 32;
//     }
//
//     if (!(sei_tmp.fMask & SEE_MASK_IDLIST) && // Not an ID List
//         (StrCmpNIW(sei_tmp.lpFile, L"shell:", 6) == 0 ||
//          StrCmpNW(sei_tmp.lpFile, L"::{", 3) == 0))
//     {
//         CComHeapPtr<ITEMIDLIST> pidlParsed;
//         hr = SHParseDisplayName(sei_tmp.lpFile, NULL, &pidlParsed, 0, NULL);
//         if (SUCCEEDED(hr) && SHELL_InvokePidl(&sei_tmp, pidlParsed))
//         {
//             sei_tmp.hInstApp = (HINSTANCE)UlongToHandle(42);
//             return TRUE;
//         }
//     }
//
//     /* Has the IDList not yet been translated? */
//     if (sei_tmp.fMask & SEE_MASK_IDLIST)
//     {
//         appKnownSingular = SHELL_translate_idlist( &sei_tmp, wszParameters,
//                            parametersLen,
//                            wszApplicationName,
//                            dwApplicationNameLen );
//     }
//
//     /* convert file URLs */
//     if (UrlIsFileUrlW(sei_tmp.lpFile))
//     {
//         CHeapPtr<WCHAR, CLocalAllocator> buf;
//         DWORD size = MAX_PATH;
//         if (!buf.Allocate(size) || FAILED(PathCreateFromUrlW(sei_tmp.lpFile, buf, &size, 0)))
//         {
//             SetLastError(ERROR_OUTOFMEMORY);
//             return FALSE;
//         }
//
//         wszApplicationName.Attach(buf.Detach());
//         sei_tmp.lpFile = wszApplicationName;
//     }
//
//     /* Else, try to execute the filename */
//     TRACE("execute: %s,%s,%s\n", debugstr_w(wszApplicationName), debugstr_w(wszParameters), debugstr_w(wszDir));
//
//     /* separate out command line arguments from executable file name */
//     LPCWSTR lpFile = sei_tmp.lpFile;
//     if (!*sei_tmp.lpParameters && !appKnownSingular)
//     {
//         /* If the executable path is quoted, handle the rest of the command line as parameters. */
//         if (sei_tmp.lpFile[0] == L'"')
//         {
//             LPWSTR pszArgs = PathGetArgsW(wszApplicationName);
//             PathRemoveArgsW(wszApplicationName);
//             PathUnquoteSpacesW(wszApplicationName);
//             parametersLen = lstrlenW(pszArgs);
//             if (parametersLen < _countof(parametersBuffer))
//             {
//                 StringCchCopyW(parametersBuffer, _countof(parametersBuffer), pszArgs);
//                 wszParameters = parametersBuffer;
//             }
//             else
//             {
//                 wszParamAlloc.Attach(StrDupW(pszArgs));
//                 wszParameters = wszParamAlloc;
//             }
//         }
//         /* We have to test sei instead of sei_tmp because sei_tmp had its
//          * input fMask modified above in SHELL_translate_idlist.
//          * This code is needed to handle the case where we only have an
//          * lpIDList with multiple CLSID/PIDL's (not 'My Computer' only) */
//         else if ((sei->fMask & SEE_MASK_IDLIST) == SEE_MASK_IDLIST)
//         {
//             WCHAR buffer[MAX_PATH], xlpFile[MAX_PATH];
//             LPWSTR space, s;
//
//             LPWSTR beg = wszApplicationName;
//             for(s = beg; (space = const_cast<LPWSTR>(strchrW(s, L' '))); s = space + 1)
//             {
//                 int idx = space - sei_tmp.lpFile;
//                 memcpy(buffer, sei_tmp.lpFile, idx * sizeof(WCHAR));
//                 buffer[idx] = '\0';
//
//                 if (SearchPathW(*sei_tmp.lpDirectory ? sei_tmp.lpDirectory : NULL,
//                     buffer, L".exe", _countof(xlpFile), xlpFile, NULL))
//                 {
//                     /* separate out command from parameter string */
//                     LPCWSTR p = space + 1;
//
//                     while(isspaceW(*p))
//                         ++p;
//
//                     strcpyW(wszParameters, p);
//                     *space = L'\0';
//
//                     break;
//                 }
//             }
//         }
//     }
//
//     WCHAR wcmdBuffer[1024];
//     LPWSTR wcmd = wcmdBuffer;
//     DWORD wcmdLen = _countof(wcmdBuffer);
//     CHeapPtr<WCHAR, CLocalAllocator> wcmdAlloc;
//
//     /* Only execute if it has an executable extension */
//     if (PathIsExeW(lpFile))
//     {
//         len = lstrlenW(wszApplicationName) + 3;
//         if (sei_tmp.lpParameters[0])
//             len += 1 + lstrlenW(wszParameters);
//         if (len > wcmdLen)
//         {
//             wcmdAlloc.Allocate(len);
//             wcmd = wcmdAlloc;
//             wcmdLen = len;
//         }
//         swprintf(wcmd, L"\"%s\"", (LPWSTR)wszApplicationName);
//         if (sei_tmp.lpParameters[0])
//         {
//             strcatW(wcmd, L" ");
//             strcatW(wcmd, wszParameters);
//         }
//
//         retval = execfunc(wcmd, NULL, FALSE, &sei_tmp, sei);
//         if (retval > 32)
//             return TRUE;
//     }
//
//     /* Else, try to find the executable */
//     WCHAR wszKeyname[256];
//     CHeapPtr<WCHAR, CLocalAllocator> env;
//     wcmd[0] = UNICODE_NULL;
//     retval = SHELL_FindExecutable(sei_tmp.lpDirectory, lpFile, sei_tmp.lpVerb, wcmd, wcmdLen, wszKeyname, &env, (LPITEMIDLIST)sei_tmp.lpIDList, sei_tmp.lpParameters);
//     if (retval > 32)  /* Found */
//     {
//         retval = SHELL_quote_and_execute(wcmd, wszParameters, wszKeyname,
//                                          wszApplicationName, env, &sei_tmp,
//                                          sei, execfunc);
//     }
//     else if (PathIsDirectoryW(lpFile))
//     {
//         WCHAR wExec[MAX_PATH];
//         CHeapPtr<WCHAR, CLocalAllocator> lpQuotedFile;
//         if (lpQuotedFile.Allocate(strlenW(lpFile) + 3))
//         {
//             retval = SHELL_FindExecutable(sei_tmp.lpDirectory, L"explorer",
//                                           L"open", wExec, MAX_PATH,
//                                           NULL, &env, NULL, NULL);
//             if (retval > 32)
//             {
//                 swprintf(lpQuotedFile, L"\"%s\"", lpFile);
//                 retval = SHELL_quote_and_execute(wExec, lpQuotedFile,
//                                                  wszKeyname,
//                                                  wszApplicationName, env,
//                                                  &sei_tmp, sei, execfunc);
//             }
//         }
//         else
//             retval = 0; /* Out of memory */
//     }
//     else if (PathIsURLW(lpFile))    /* File not found, check for URL */
//     {
//         retval = SHELL_execute_url(lpFile, wcmd, &sei_tmp, sei, execfunc );
//     }
//     /* Check if file specified is in the form www.??????.*** */
//     else if (!strncmpiW(lpFile, L"www", 3))
//     {
//         /* if so, prefix lpFile with http:// and call ShellExecute */
//         WCHAR lpstrTmpFile[256];
//         strcpyW(lpstrTmpFile, L"http://");
//         strcatW(lpstrTmpFile, lpFile); // FIXME: Possible buffer overflow
//         // FIXME: This will not correctly return the hProcess to the caller
//         retval = (UINT_PTR)ShellExecuteW(sei_tmp.hwnd, sei_tmp.lpVerb, lpstrTmpFile, NULL, NULL, 0);
//     }
//
//     TRACE("retval %lu\n", retval);
//
//     if (retval <= 32 && !(sei_tmp.fMask & SEE_MASK_FLAG_NO_UI))
//     {
//         if (retval == SE_ERR_NOASSOC && !(sei->fMask & SEE_MASK_CLASSALL))
//             retval = InvokeOpenWith(sei_tmp.hwnd, *sei);
//         if (retval <= 32)
//             do_error_dialog(retval, sei_tmp.hwnd, lpFile);
//     }
//
//     sei->hInstApp = (HINSTANCE)(retval > 32 ? 33 : retval);
//
//     return retval > 32;
// }
//
// /*************************************************************************
//  * ShellExecuteA            [SHELL32.290]
//  */
// HINSTANCE WINAPI ShellExecuteA(HWND hWnd, LPCSTR lpVerb, LPCSTR lpFile,
//                                LPCSTR lpParameters, LPCSTR lpDirectory, INT iShowCmd)
// {
//     SHELLEXECUTEINFOA sei;
//
//     TRACE("%p,%s,%s,%s,%s,%d\n",
//           hWnd, debugstr_a(lpVerb), debugstr_a(lpFile),
//           debugstr_a(lpParameters), debugstr_a(lpDirectory), iShowCmd);
//
//     sei.cbSize = sizeof(sei);
//     sei.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
//     sei.hwnd = hWnd;
//     sei.lpVerb = lpVerb;
//     sei.lpFile = lpFile;
//     sei.lpParameters = lpParameters;
//     sei.lpDirectory = lpDirectory;
//     sei.nShow = iShowCmd;
//     sei.lpIDList = 0;
//     sei.lpClass = 0;
//     sei.hkeyClass = 0;
//     sei.dwHotKey = 0;
//     sei.hProcess = 0;
//
//     if (!(SHGetAppCompatFlags(SHACF_WIN95SHLEXEC) & SHACF_WIN95SHLEXEC))
//         sei.fMask |= SEE_MASK_NOASYNC;
//     ShellExecuteExA(&sei);
//     return sei.hInstApp;
// }
//
// static DWORD
// ShellExecute_Normal( LPSHELLEXECUTEINFOW sei)
// {
//     // FIXME
//     if (SHELL_execute(sei, SHELL_ExecuteW))
//         return ERROR_SUCCESS;
//     DWORD err = GetLastError();
//     if (!err)
//         DbgPrint("FIXME: Failed with error 0 on '%ls'\n", sei->lpFile);
//     return err ? err : ERROR_FILE_NOT_FOUND;
// }
//
// static VOID
// ShellExecute_ShowError(
//      const SHELLEXECUTEINFOW *ExecInfo,
//      LPCWSTR pszCaption,
//      DWORD dwError)
// {
//     // FIXME: Show error message
// }
//
// /*************************************************************************
//  * ShellExecuteExA                [SHELL32.292]
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// ShellExecuteExA(LPSHELLEXECUTEINFOA sei)
// {
//     SHELLEXECUTEINFOW seiW;
//     BOOL ret;
//     WCHAR *wVerb = NULL, *wFile = NULL, *wParameters = NULL, *wDirectory = NULL, *wClass = NULL;
//
//     TRACE("%p\n", sei);
//
//     if (sei->cbSize != sizeof(SHELLEXECUTEINFOA))
//     {
//         sei->hInstApp = (HINSTANCE)ERROR_ACCESS_DENIED;
//         SetLastError(ERROR_ACCESS_DENIED);
//         return FALSE;
//     }
//
//     memcpy(&seiW, sei, sizeof(SHELLEXECUTEINFOW));
//
//     seiW.cbSize = sizeof(SHELLEXECUTEINFOW);
//
//     if (sei->lpVerb)
//         seiW.lpVerb = __SHCloneStrAtoW(&wVerb, sei->lpVerb);
//
//     if (sei->lpFile)
//         seiW.lpFile = __SHCloneStrAtoW(&wFile, sei->lpFile);
//
//     if (sei->lpParameters)
//         seiW.lpParameters = __SHCloneStrAtoW(&wParameters, sei->lpParameters);
//
//     if (sei->lpDirectory)
//         seiW.lpDirectory = __SHCloneStrAtoW(&wDirectory, sei->lpDirectory);
//
//     if ((sei->fMask & SEE_MASK_CLASSALL) == SEE_MASK_CLASSNAME && sei->lpClass)
//         seiW.lpClass = __SHCloneStrAtoW(&wClass, sei->lpClass);
//     else
//         seiW.lpClass = NULL;
//
//     ret = ShellExecuteExW(&seiW);
//
//     sei->hInstApp = seiW.hInstApp;
//
//     if (sei->fMask & SEE_MASK_NOCLOSEPROCESS)
//         sei->hProcess = seiW.hProcess;
//
//     SHFree(wVerb);
//     SHFree(wFile);
//     SHFree(wParameters);
//     SHFree(wDirectory);
//     SHFree(wClass);
//
//     return ret;
// }
//
// /*************************************************************************
//  * ShellExecuteExW                [SHELL32.293]
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// ShellExecuteExW(LPSHELLEXECUTEINFOW sei)
// {
//     HRESULT hrCoInit;
//     DWORD dwError;
//     ULONG fOldMask;
//
//     if (sei->cbSize != sizeof(SHELLEXECUTEINFOW))
//     {
//         sei->hInstApp = (HINSTANCE)UlongToHandle(SE_ERR_ACCESSDENIED);
//         SetLastError(ERROR_ACCESS_DENIED);
//         return FALSE;
//     }
//
//     hrCoInit = SHCoInitializeAnyApartment();
//
//     if (SHRegGetBoolUSValueW(L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer",
//                              L"MaximizeApps", FALSE, FALSE))
//     {
//         switch (sei->nShow)
//         {
//             case SW_SHOW:
//             case SW_SHOWDEFAULT:
//             case SW_SHOWNORMAL:
//             case SW_RESTORE:
//                 sei->nShow = SW_SHOWMAXIMIZED;
//                 break;
//             default:
//                 break;
//         }
//     }
//
//     fOldMask = sei->fMask;
//
//     if (!(fOldMask & SEE_MASK_NOASYNC) && SHELL_InRunDllProcess())
//         sei->fMask |= SEE_MASK_WAITFORINPUTIDLE | SEE_MASK_NOASYNC;
//
//     dwError = ShellExecute_Normal(sei);
//
//     if (dwError && dwError != ERROR_DLL_NOT_FOUND && dwError != ERROR_CANCELLED)
//         ShellExecute_ShowError(sei, NULL, dwError);
//
//     sei->fMask = fOldMask;
//
//     if (SUCCEEDED(hrCoInit))
//         CoUninitialize();
//
//     if (dwError)
//         SetLastError(dwError);
//
//     return dwError == ERROR_SUCCESS;
// }
//
// /*************************************************************************
//  * ShellExecuteW            [SHELL32.294]
//  */
// HINSTANCE WINAPI ShellExecuteW(HWND hwnd, LPCWSTR lpVerb, LPCWSTR lpFile,
//                                LPCWSTR lpParameters, LPCWSTR lpDirectory, INT nShowCmd)
// {
//     SHELLEXECUTEINFOW sei;
//
//     TRACE("\n");
//     sei.cbSize = sizeof(sei);
//     sei.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
//     sei.hwnd = hwnd;
//     sei.lpVerb = lpVerb;
//     sei.lpFile = lpFile;
//     sei.lpParameters = lpParameters;
//     sei.lpDirectory = lpDirectory;
//     sei.nShow = nShowCmd;
//     sei.lpIDList = 0;
//     sei.lpClass = 0;
//     sei.hkeyClass = 0;
//     sei.dwHotKey = 0;
//     sei.hProcess = 0;
//
//     if (!(SHGetAppCompatFlags(SHACF_WIN95SHLEXEC) & SHACF_WIN95SHLEXEC))
//         sei.fMask |= SEE_MASK_NOASYNC;
//     ShellExecuteExW(&sei);
//     return sei.hInstApp;
// }
//
// /*************************************************************************
//  * WOWShellExecute            [SHELL32.@]
//  *
//  * FIXME: the callback function most likely doesn't work the same way on Windows.
//  */
// extern "C" HINSTANCE WINAPI WOWShellExecute(HWND hWnd, LPCSTR lpVerb, LPCSTR lpFile,
//         LPCSTR lpParameters, LPCSTR lpDirectory, INT iShowCmd, void *callback)
// {
//     SHELLEXECUTEINFOW seiW;
//     WCHAR *wVerb = NULL, *wFile = NULL, *wParameters = NULL, *wDirectory = NULL;
//     HANDLE hProcess = 0;
//
//     seiW.lpVerb = lpVerb ? __SHCloneStrAtoW(&wVerb, lpVerb) : NULL;
//     seiW.lpFile = lpFile ? __SHCloneStrAtoW(&wFile, lpFile) : NULL;
//     seiW.lpParameters = lpParameters ? __SHCloneStrAtoW(&wParameters, lpParameters) : NULL;
//     seiW.lpDirectory = lpDirectory ? __SHCloneStrAtoW(&wDirectory, lpDirectory) : NULL;
//
//     seiW.cbSize = sizeof(seiW);
//     seiW.fMask = 0;
//     seiW.hwnd = hWnd;
//     seiW.nShow = iShowCmd;
//     seiW.lpIDList = 0;
//     seiW.lpClass = 0;
//     seiW.hkeyClass = 0;
//     seiW.dwHotKey = 0;
//     seiW.hProcess = hProcess;
//
//     SHELL_execute(&seiW, (SHELL_ExecuteW32)callback);
//
//     SHFree(wVerb);
//     SHFree(wFile);
//     SHFree(wParameters);
//     SHFree(wDirectory);
//     return seiW.hInstApp;
// }
//
// /*************************************************************************
//  * OpenAs_RunDLLW          [SHELL32.@]
//  */
// extern "C" void WINAPI
// OpenAs_RunDLLW(HWND hwnd, HINSTANCE hinst, LPCWSTR cmdline, int cmdshow)
// {
//     OPENASINFO info = { cmdline, NULL, OAIF_ALLOW_REGISTRATION | OAIF_REGISTER_EXT | OAIF_EXEC };
//     TRACE("%p, %p, %s, %d\n", hwnd, hinst, debugstr_w(cmdline), cmdshow);
//     SHOpenWithDialog(hwnd, &info);
// }
//
// /*************************************************************************
//  * OpenAs_RunDLLA          [SHELL32.@]
//  */
// extern "C" void WINAPI
// OpenAs_RunDLLA(HWND hwnd, HINSTANCE hinst, LPCSTR cmdline, int cmdshow)
// {
//     LPWSTR pszCmdLineW = NULL;
//     TRACE("%p, %p, %s, %d\n", hwnd, hinst, debugstr_a(cmdline), cmdshow);
//
//     if (cmdline)
//         __SHCloneStrAtoW(&pszCmdLineW, cmdline);
//     OpenAs_RunDLLW(hwnd, hinst, pszCmdLineW, cmdshow);
//     SHFree(pszCmdLineW);
// }
//
// /*************************************************************************/
//
// static LPCWSTR
// SplitParams(LPCWSTR psz, LPWSTR pszArg0, size_t cchArg0)
// {
//     LPCWSTR pch;
//     size_t ich = 0;
//     if (*psz == L'"')
//     {
//         // 1st argument is quoted. the string in quotes is quoted 1st argument.
//         // [pch] --> [pszArg0+ich]
//         for (pch = psz + 1; *pch && ich + 1 < cchArg0; ++ich, ++pch)
//         {
//             if (*pch == L'"' && pch[1] == L'"')
//             {
//                 // doubled double quotations found!
//                 pszArg0[ich] = L'"';
//             }
//             else if (*pch == L'"')
//             {
//                 // single double quotation found!
//                 ++pch;
//                 break;
//             }
//             else
//             {
//                 // otherwise
//                 pszArg0[ich] = *pch;
//             }
//         }
//     }
//     else
//     {
//         // 1st argument is unquoted. non-space sequence is 1st argument.
//         // [pch] --> [pszArg0+ich]
//         for (pch = psz; *pch && !iswspace(*pch) && ich + 1 < cchArg0; ++ich, ++pch)
//         {
//             pszArg0[ich] = *pch;
//         }
//     }
//     pszArg0[ich] = 0;
//
//     // skip space
//     while (iswspace(*pch))
//         ++pch;
//
//     return pch;
// }
//
// HRESULT WINAPI ShellExecCmdLine(
//     HWND hwnd,
//     LPCWSTR pwszCommand,
//     LPCWSTR pwszStartDir,
//     int nShow,
//     LPVOID pUnused,
//     DWORD dwSeclFlags)
// {
//     SHELLEXECUTEINFOW info;
//     DWORD dwSize, dwType, dwFlags = SEE_MASK_DOENVSUBST | SEE_MASK_NOASYNC;
//     LPCWSTR pszVerb = NULL;
//     WCHAR szFile[MAX_PATH], szFile2[MAX_PATH];
//     HRESULT hr;
//     LPCWSTR pchParams;
//     LPWSTR lpCommand = NULL;
//
//     if (pwszCommand == NULL)
//         RaiseException(EXCEPTION_ACCESS_VIOLATION, EXCEPTION_NONCONTINUABLE,
//                        1, (ULONG_PTR*)pwszCommand);
//
//     __SHCloneStrW(&lpCommand, pwszCommand);
//     StrTrimW(lpCommand, L" \t");
//
//     if (dwSeclFlags & SECL_NO_UI)
//         dwFlags |= SEE_MASK_FLAG_NO_UI;
//     if (dwSeclFlags & SECL_LOG_USAGE)
//         dwFlags |= SEE_MASK_FLAG_LOG_USAGE;
//     if (dwSeclFlags & SECL_USE_IDLIST)
//         dwFlags |= SEE_MASK_INVOKEIDLIST;
//
//     if (dwSeclFlags & SECL_RUNAS)
//     {
//         dwSize = 0;
//         hr = AssocQueryStringW(ASSOCF_NONE, ASSOCSTR_COMMAND, lpCommand, L"runas", NULL, &dwSize);
//         if (SUCCEEDED(hr) && dwSize != 0)
//         {
//             pszVerb = L"runas";
//         }
//     }
//
//     if (PathIsURLW(lpCommand) || UrlIsW(lpCommand, URLIS_APPLIABLE))
//     {
//         StringCchCopyW(szFile, _countof(szFile), lpCommand);
//         pchParams = NULL;
//     }
//     else
//     {
//         PCWSTR apPathList[2];
//
//         pchParams = SplitParams(lpCommand, szFile, _countof(szFile));
//         if (szFile[0] != UNICODE_NULL && szFile[1] == L':' &&
//             szFile[2] == UNICODE_NULL)
//         {
//             PathAddBackslashW(szFile);
//         }
//
//         WCHAR szCurDir[MAX_PATH];
//         GetCurrentDirectoryW(_countof(szCurDir), szCurDir);
//         if (pwszStartDir)
//         {
//             SetCurrentDirectoryW(pwszStartDir);
//         }
//
//         if ((PathIsRelativeW(szFile) &&
//              GetFullPathNameW(szFile, _countof(szFile2), szFile2, NULL) &&
//              PathFileExistsW(szFile2)) ||
//             SearchPathW(NULL, szFile, NULL, _countof(szFile2), szFile2, NULL))
//         {
//             StringCchCopyW(szFile, _countof(szFile), szFile2);
//         }
//
//         apPathList[0] = pwszStartDir;
//         apPathList[1] = NULL;
//         PathFindOnPathExW(szFile, apPathList, WHICH_DEFAULT);
//
//         if (!(dwSeclFlags & SECL_ALLOW_NONEXE))
//         {
//             if (!GetBinaryTypeW(szFile, &dwType))
//             {
//                 SHFree(lpCommand);
//
//                 if (!(dwSeclFlags & SECL_NO_UI))
//                 {
//                     WCHAR szText[128 + MAX_PATH], szFormat[128];
//                     LoadStringW(shell32_hInstance, IDS_FILE_NOT_FOUND, szFormat, _countof(szFormat));
//                     StringCchPrintfW(szText, _countof(szText), szFormat, szFile);
//                     MessageBoxW(hwnd, szText, NULL, MB_ICONERROR);
//                 }
//                 return CO_E_APPNOTFOUND;
//             }
//         }
//         else
//         {
//             if (GetFileAttributesW(szFile) == INVALID_FILE_ATTRIBUTES)
//             {
//                 SHFree(lpCommand);
//
//                 if (!(dwSeclFlags & SECL_NO_UI))
//                 {
//                     WCHAR szText[128 + MAX_PATH], szFormat[128];
//                     LoadStringW(shell32_hInstance, IDS_FILE_NOT_FOUND, szFormat, _countof(szFormat));
//                     StringCchPrintfW(szText, _countof(szText), szFormat, szFile);
//                     MessageBoxW(hwnd, szText, NULL, MB_ICONERROR);
//                 }
//                 return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
//             }
//         }
//     }
//
//     ZeroMemory(&info, sizeof(info));
//     info.cbSize = sizeof(info);
//     info.fMask = dwFlags;
//     info.hwnd = hwnd;
//     info.lpVerb = pszVerb;
//     info.lpFile = szFile;
//     info.lpParameters = (pchParams && *pchParams) ? pchParams : NULL;
//     info.lpDirectory = pwszStartDir;
//     info.nShow = nShow;
//     UINT error = ShellExecuteExW(&info) ? ERROR_SUCCESS : GetLastError();
//     SHFree(lpCommand);
//     return HRESULT_FROM_WIN32(error);
// }
//
// /*************************************************************************
//  *                RealShellExecuteExA (SHELL32.266)
//  */
// extern "C"
// HINSTANCE WINAPI
// RealShellExecuteExA(
//      HWND hwnd,
//      LPCSTR lpOperation,
//      LPCSTR lpFile,
//      LPCSTR lpParameters,
//      LPCSTR lpDirectory,
//      LPSTR lpReturn,
//      LPCSTR lpTitle,
//      LPVOID lpReserved,
//      INT nCmdShow,
//      PHANDLE lphProcess,
//      DWORD dwFlags)
// {
//     SHELLEXECUTEINFOA ExecInfo;
//
//     TRACE("(%p, %s, %s, %s, %s, %p, %s, %p, %u, %p, %lu)\n",
//           hwnd, debugstr_a(lpOperation), debugstr_a(lpFile), debugstr_a(lpParameters),
//           debugstr_a(lpDirectory), lpReserved, debugstr_a(lpTitle),
//           lpReserved, nCmdShow, lphProcess, dwFlags);
//
//     ZeroMemory(&ExecInfo, sizeof(ExecInfo));
//     ExecInfo.cbSize = sizeof(ExecInfo);
//     ExecInfo.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
//     ExecInfo.hwnd = hwnd;
//     ExecInfo.lpVerb = lpOperation;
//     ExecInfo.lpFile = lpFile;
//     ExecInfo.lpParameters = lpParameters;
//     ExecInfo.lpDirectory = lpDirectory;
//     ExecInfo.nShow = (WORD)nCmdShow;
//
//     if (lpReserved)
//     {
//         ExecInfo.fMask |= SEE_MASK_USE_RESERVED;
//         ExecInfo.hInstApp = (HINSTANCE)lpReserved;
//     }
//
//     if (lpTitle)
//     {
//         ExecInfo.fMask |= SEE_MASK_HASTITLE;
//         ExecInfo.lpClass = lpTitle;
//     }
//
//     if (dwFlags & 1)
//         ExecInfo.fMask |= SEE_MASK_FLAG_SEPVDM;
//
//     if (dwFlags & 2)
//         ExecInfo.fMask |= SEE_MASK_NO_CONSOLE;
//
//     if (lphProcess == NULL)
//     {
//         ShellExecuteExA(&ExecInfo);
//     }
//     else
//     {
//         ExecInfo.fMask |= SEE_MASK_NOCLOSEPROCESS;
//         ShellExecuteExA(&ExecInfo);
//         *lphProcess = ExecInfo.hProcess;
//     }
//
//     return ExecInfo.hInstApp;
// }
//
// /*************************************************************************
//  *                RealShellExecuteExW (SHELL32.267)
//  */
// extern "C"
// HINSTANCE WINAPI
// RealShellExecuteExW(
//      HWND hwnd,
//      LPCWSTR lpOperation,
//      LPCWSTR lpFile,
//      LPCWSTR lpParameters,
//      LPCWSTR lpDirectory,
//      LPWSTR lpReturn,
//      LPCWSTR lpTitle,
//      LPVOID lpReserved,
//      INT nCmdShow,
//      PHANDLE lphProcess,
//      DWORD dwFlags)
// {
//     SHELLEXECUTEINFOW ExecInfo;
//
//     TRACE("(%p, %s, %s, %s, %s, %p, %s, %p, %u, %p, %lu)\n",
//           hwnd, debugstr_w(lpOperation), debugstr_w(lpFile), debugstr_w(lpParameters),
//           debugstr_w(lpDirectory), lpReserved, debugstr_w(lpTitle),
//           lpReserved, nCmdShow, lphProcess, dwFlags);
//
//     ZeroMemory(&ExecInfo, sizeof(ExecInfo));
//     ExecInfo.cbSize = sizeof(ExecInfo);
//     ExecInfo.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_UNKNOWN_0x1000;
//     ExecInfo.hwnd = hwnd;
//     ExecInfo.lpVerb = lpOperation;
//     ExecInfo.lpFile = lpFile;
//     ExecInfo.lpParameters = lpParameters;
//     ExecInfo.lpDirectory = lpDirectory;
//     ExecInfo.nShow = (WORD)nCmdShow;
//
//     if (lpReserved)
//     {
//         ExecInfo.fMask |= SEE_MASK_USE_RESERVED;
//         ExecInfo.hInstApp = (HINSTANCE)lpReserved;
//     }
//
//     if (lpTitle)
//     {
//         ExecInfo.fMask |= SEE_MASK_HASTITLE;
//         ExecInfo.lpClass = lpTitle;
//     }
//
//     if (dwFlags & 1)
//         ExecInfo.fMask |= SEE_MASK_FLAG_SEPVDM;
//
//     if (dwFlags & 2)
//         ExecInfo.fMask |= SEE_MASK_NO_CONSOLE;
//
//     if (lphProcess == NULL)
//     {
//         ShellExecuteExW(&ExecInfo);
//     }
//     else
//     {
//         ExecInfo.fMask |= SEE_MASK_NOCLOSEPROCESS;
//         ShellExecuteExW(&ExecInfo);
//         *lphProcess = ExecInfo.hProcess;
//     }
//
//     return ExecInfo.hInstApp;
// }
//
// /*************************************************************************
//  *                RealShellExecuteA (SHELL32.265)
//  */
// extern "C"
// HINSTANCE WINAPI
// RealShellExecuteA(
//      HWND hwnd,
//      LPCSTR lpOperation,
//      LPCSTR lpFile,
//      LPCSTR lpParameters,
//      LPCSTR lpDirectory,
//      LPSTR lpReturn,
//      LPCSTR lpTitle,
//      LPVOID lpReserved,
//      INT nCmdShow,
//      PHANDLE lphProcess)
// {
//     return RealShellExecuteExA(hwnd,
//                                lpOperation,
//                                lpFile,
//                                lpParameters,
//                                lpDirectory,
//                                lpReturn,
//                                lpTitle,
//                                lpReserved,
//                                nCmdShow,
//                                lphProcess,
//                                0);
// }
//
// /*************************************************************************
//  *                RealShellExecuteW (SHELL32.268)
//  */
// extern "C"
// HINSTANCE WINAPI
// RealShellExecuteW(
//      HWND hwnd,
//      LPCWSTR lpOperation,
//      LPCWSTR lpFile,
//      LPCWSTR lpParameters,
//      LPCWSTR lpDirectory,
//      LPWSTR lpReturn,
//      LPCWSTR lpTitle,
//      LPVOID lpReserved,
//      INT nCmdShow,
//      PHANDLE lphProcess)
// {
//     return RealShellExecuteExW(hwnd,
//                                lpOperation,
//                                lpFile,
//                                lpParameters,
//                                lpDirectory,
//                                lpReturn,
//                                lpTitle,
//                                lpReserved,
//                                nCmdShow,
//                                lphProcess,
//                                0);
// }
//
// /*************************************************************************
//  *  PathProcessCommandW [Internal]
//  *
//  * @see https://learn.microsoft.com/en-us/windows/win32/api/shlobj/nf-shlobj-pathprocesscommand
//  * @see ./wine/shellpath.c
//  */
// extern "C" LONG
// PathProcessCommandW(
//      PCWSTR pszSrc,
//      PWSTR pszDest,
//      INT cchDest,
//      DWORD dwFlags)
// {
//     TRACE("%s, %p, %d, 0x%X\n", wine_dbgstr_w(pszSrc), pszDest, cchDest, dwFlags);
//
//     if (!pszSrc)
//         return -1;
//
//     CStringW szPath;
//     PCWSTR pchArg = NULL;
//
//     if (*pszSrc == L'"') // Quoted?
//     {
//         ++pszSrc;
//
//         PCWSTR pch = wcschr(pszSrc, L'"');
//         if (pch)
//         {
//             szPath.SetString(pszSrc, pch - pszSrc);
//             pchArg = pch + 1;
//         }
//         else
//         {
//             szPath = pszSrc;
//         }
//
//         if ((dwFlags & PPCF_FORCEQUALIFY) || PathIsRelativeW(szPath))
//         {
//             BOOL ret = PathResolveW(szPath.GetBuffer(MAX_PATH), NULL, PRF_TRYPROGRAMEXTENSIONS);
//             szPath.ReleaseBuffer();
//             if (!ret)
//                 return -1;
//         }
//     }
//     else // Not quoted?
//     {
//         BOOL resolved = FALSE;
//         BOOL resolveRelative = PathIsRelativeW(pszSrc) || (dwFlags & PPCF_FORCEQUALIFY);
//         INT cchPath = 0;
//
//         for (INT ich = 0; ; ++ich)
//         {
//             szPath += pszSrc[ich];
//
//             if (pszSrc[ich] && pszSrc[ich] != L' ')
//                 continue;
//
//             szPath = szPath.Left(ich);
//
//             if (resolveRelative &&
//                 !PathResolveW(szPath.GetBuffer(MAX_PATH), NULL, PRF_TRYPROGRAMEXTENSIONS))
//             {
//                 szPath.ReleaseBuffer();
//                 szPath += pszSrc[ich];
//             }
//             else
//             {
//                 szPath.ReleaseBuffer();
//
//                 DWORD attrs = GetFileAttributesW(szPath);
//                 if (attrs != INVALID_FILE_ATTRIBUTES &&
//                     (!(attrs & FILE_ATTRIBUTE_DIRECTORY) || !(dwFlags & PPCF_NODIRECTORIES)))
//                 {
//                     resolved = TRUE;
//                     pchArg = pszSrc + ich;
//
//                     if (!(dwFlags & PPCF_LONGESTPOSSIBLE))
//                         break;
//
//                     cchPath = ich;
//                     break;
//                 }
//                 else if (!resolveRelative)
//                 {
//                     szPath += pszSrc[ich];
//                 }
//             }
//
//             if (!szPath[ich])
//             {
//                 szPath.ReleaseBuffer(); // Remove excessive '\0'
//                 break;
//             }
//         }
//
//         if (!resolved)
//             return -1;
//
//         if (cchPath && (dwFlags & PPCF_LONGESTPOSSIBLE))
//         {
//             szPath = szPath.Left(cchPath);
//             pchArg = pszSrc + cchPath;
//         }
//     }
//
//     BOOL needsQuoting = (dwFlags & PPCF_ADDQUOTES) && wcschr(szPath, L' ');
//     CStringW result = needsQuoting ? (L"\"" + szPath + L"\"") : szPath;
//
//     if (pchArg && (dwFlags & PPCF_ADDARGUMENTS))
//         result += pchArg;
//
//     LONG requiredSize = result.GetLength() + 1;
//     if (!pszDest)
//         return requiredSize;
//
//     if (requiredSize > cchDest || StringCchCopyW(pszDest, cchDest, result) != S_OK)
//         return -1;
//
//     return requiredSize;
// }
//
// // The common helper of ShellExec_RunDLLA and ShellExec_RunDLLW
// static VOID
// ShellExec_RunDLL_Helper(
//      HWND hwnd,
//      HINSTANCE hInstance,
//      PCWSTR pszCmdLine,
//      INT nCmdShow)
// {
//     TRACE("(%p, %p, %s, 0x%X)\n", hwnd, hInstance, wine_dbgstr_w(pszCmdLine), nCmdShow);
//
//     if (!pszCmdLine || !*pszCmdLine)
//         return;
//
//     // '?' enables us to specify the additional mask value
//     ULONG fNewMask = SEE_MASK_NOASYNC;
//     if (*pszCmdLine == L'?') // 1st question
//     {
//         INT MaskValue;
//         if (StrToIntExW(pszCmdLine + 1, STIF_SUPPORT_HEX, &MaskValue))
//             fNewMask |= MaskValue;
//
//         PCWSTR pch2ndQuestion = StrChrW(pszCmdLine + 1, L'?'); // 2nd question
//         if (pch2ndQuestion)
//             pszCmdLine = pch2ndQuestion + 1;
//     }
//
//     WCHAR szPath[2 * MAX_PATH];
//     DWORD dwFlags = PPCF_FORCEQUALIFY | PPCF_INCLUDEARGS | PPCF_ADDQUOTES;
//     if (PathProcessCommandW(pszCmdLine, szPath, _countof(szPath), dwFlags) == -1)
//         StrCpyNW(szPath, pszCmdLine, _countof(szPath));
//
//     // Split arguments from the path
//     LPWSTR Args = PathGetArgsW(szPath);
//     if (*Args)
//         *(Args - 1) = UNICODE_NULL;
//
//     PathUnquoteSpacesW(szPath);
//
//     // Execute
//     SHELLEXECUTEINFOW execInfo = { sizeof(execInfo) };
//     execInfo.fMask = fNewMask;
//     execInfo.hwnd = hwnd;
//     execInfo.lpFile = szPath;
//     execInfo.lpParameters = Args;
//     execInfo.nShow = nCmdShow;
//     if (!ShellExecuteExW(&execInfo))
//     {
//         DWORD dwError = GetLastError();
//         if (SHELL_InRunDllProcess()) // Is it a RUNDLL process?
//             ExitProcess(dwError); // Terminate it now
//     }
// }
//
// /*************************************************************************
//  *  ShellExec_RunDLLA [SHELL32.358]
//  *
//  * @see https://www.hexacorn.com/blog/2024/11/30/1-little-known-secret-of-shellexec_rundll/
//  */
// extern "C"
// VOID WINAPI
// ShellExec_RunDLLA(
//      HWND hwnd,
//      HINSTANCE hInstance,
//      PCSTR pszCmdLine,
//      INT nCmdShow)
// {
//     CStringW strCmdLine = pszCmdLine; // Keep
//     ShellExec_RunDLL_Helper(hwnd, hInstance, strCmdLine, nCmdShow);
// }
//
// /*************************************************************************
//  *  ShellExec_RunDLLW [SHELL32.359]
//  *
//  * @see https://www.hexacorn.com/blog/2024/11/30/1-little-known-secret-of-shellexec_rundll/
//  */
// extern "C"
// VOID WINAPI
// ShellExec_RunDLLW(
//      HWND hwnd,
//      HINSTANCE hInstance,
//      PCWSTR pszCmdLine,
//      INT nCmdShow)
// {
//     ShellExec_RunDLL_Helper(hwnd, hInstance, pszCmdLine, nCmdShow);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
