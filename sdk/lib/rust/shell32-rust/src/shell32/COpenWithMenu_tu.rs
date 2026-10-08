use crate::prelude::*;
use crate::shell32::sdk_consts::*;
use crate::open_with_list::{
    COpenWithList, COpenWithList_delete, COpenWithList_Execute_body,
    COpenWithList_GetCount_body, COpenWithList_GetIcon_body, COpenWithList_GetList_body,
    COpenWithList_GetName_body, COpenWithList_GetRecommendedCount_body,
    COpenWithList_Load_body, COpenWithList_LoadRecommended_body, COpenWithList_new,
};
use crate::open_with_menu_support::{
    CmicFlagsToSeeFlags, HResultFromWin32, PathFindFileNameW, SApp, SHELL_ErrorBox,
    StringCbCopyW, HRESULT_FROM_WIN32, DialogBoxParamW,
};
use crate::open_with_dialog::{
    COpenWithDialog, COpenWithDialog_ctor_body, COpenWithDialog_DialogProc_body,
    COpenWithDialog_dtor_body, COpenWithDialog_IsNoOpen_body,
};
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_51AA45493C0B: [WCHAR; 13] = [0x0072, 0x0075, 0x006E, 0x0064, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_D767B87F69DC: [WCHAR; 9] = [0x0072, 0x0075, 0x006E, 0x0064, 0x006C, 0x006C, 0x0033, 0x0032, 0x0000];
pub static CPP_COM_WSTR_F3CDEC9800CB: [WCHAR; 8] = [0x0055, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0000];

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
//
// THE canonical copy. Pass it as
//   rustconv translate <tu> --free-functions --local-raii CDataObjectHIDA=tools/rustconv/data/local_raii/CDataObjectHIDA.rs
// Extracted from shell32/shldataobject.rs, which is one of the three already-flipped TUs that
// each had to carry an identical copy (the others: shell32/shlfolder.rs, dialogs/item_prop.rs).
// A fourth, COpenWithMenu.cpp, stalled unpaid for want of it — the refusal is `translate`
// declining to guess drop semantics, and the fix was always a file, never a retry.
// `hida_clipformat()` comes from the crate prelude (shellutils.rs:68); this file assumes it.
// Hand-verified Rust lowering of the C++ RAII struct `CDataObjectHIDA`
// (sdk/include/reactos/shellutils.h:738). Consumed by
// `rustconv translate --free-functions --local-raii CDataObjectHIDA=<this file>`:
// spliced verbatim into the assembled module, and its API sealed into each
// affected worker prompt so the model maps C++ usage onto this fixed surface
// instead of inventing the drop semantics.
//
// It borrows the HIDA out of an IDataObject WITHOUT copying it, so the medium
// must stay locked for exactly the lifetime of the guard — which is the whole
// reason it is RAII and the whole reason a hand lowering is required.
//
// C++ semantics preserved exactly:
//   ctor        — CreateCIDA(pDataObject, &m_cida, m_medium); m_hr = result
//   dtor        — DestroyCIDA(m_cida, m_medium)                → impl Drop
//   hr()        — the stored ctor HRESULT
//   operator bool()        → is_valid()      (m_cida != nullptr)
//   operator const CIDA*() → as_ptr()        (the implicit conversion at a call
//                                             site such as `f(x, cida)`)
//   operator->()           → (*cida.as_ptr()).field
//
// Two details that look like bugs and are not, so do not "fix" them:
//   * DestroyCIDA calls ReleaseStgMedium unconditionally. On the failure paths
//     CreateCIDA has already set tymed = TYMED_NULL, which makes that call a
//     documented no-op — including after CreateCIDA itself released the medium.
//   * C++ leaves m_medium uninitialized and only assigns pUnkForRelease. Every
//     path assigns tymed before anything reads it (GetData on success,
//     TYMED_NULL otherwise). Zeroing here is strictly safer and changes no path.
struct CDataObjectHIDA {
    m_medium: STGMEDIUM,
    m_cida: *const CIDA,
    m_hr: HRESULT,
}

impl CDataObjectHIDA {
    unsafe fn new(pDataObject: *mut IDataObject) -> CDataObjectHIDA {
        let mut medium: STGMEDIUM = core::mem::zeroed();
        let mut cida: *const CIDA = core::ptr::null();
        let hr = CDataObjectHIDA::CreateCIDA(pDataObject, &mut cida, &mut medium);
        CDataObjectHIDA { m_medium: medium, m_cida: cida, m_hr: hr }
    }

    unsafe fn CreateCIDA(
        pDataObject: *mut IDataObject,
        ppcida: *mut *const CIDA,
        medium: *mut STGMEDIUM,
    ) -> HRESULT {
        *ppcida = core::ptr::null();
        (*medium).pUnkForRelease = core::ptr::null_mut();
        // g_cfHIDA in C++ is a DECLSPEC_SELECTANY cache of the registered format.
        let mut format = FORMATETC {
            cfFormat: hida_clipformat(),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let hr = ((*(*pDataObject).lpVtbl).GetData)(
            pDataObject,
            (&mut format as *mut FORMATETC).cast(),
            medium.cast(),
        );
        if hr >= 0 {
            *ppcida = GlobalLock((*medium).__bindgen_anon_1.hGlobal) as *const CIDA;
            if !(*ppcida).is_null() {
                return S_OK;
            }
            ReleaseStgMedium(medium);
            (*medium).tymed = TYMED_NULL;
            return E_UNEXPECTED;
        }
        (*medium).tymed = TYMED_NULL;
        hr
    }

    unsafe fn hr(&self) -> HRESULT {
        self.m_hr
    }

    unsafe fn is_valid(&self) -> bool {
        !self.m_cida.is_null()
    }

    unsafe fn as_ptr(&self) -> *const CIDA {
        self.m_cida
    }
}

impl Drop for CDataObjectHIDA {
    fn drop(&mut self) {
        unsafe {
            if !self.m_cida.is_null() {
                GlobalUnlock(self.m_medium.__bindgen_anon_1.hGlobal);
            }
            ReleaseStgMedium(&mut self.m_medium);
        }
    }
}

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PSApp = *mut SApp;

#[repr(C)]
pub struct LANGANDCODEPAGE {
    pub lang: u16,
    pub code: u16,
}

pub type LPLANGANDCODEPAGE = *mut LANGANDCODEPAGE;

#[repr(C)]
pub struct CMVERBMAP {
    pub pszVerb: PCWSTR,
    pub idCmd: u32,
}

pub static mut g_VerbMap: [CMVERBMAP; 2] = [
    CMVERBMAP {
        pszVerb: win32macros_rust::wide_bytes!(b"o\0p\0e\0n\0a\0s\0\0\0") as PCWSTR,
        idCmd: 0,
    },
    CMVERBMAP {
        pszVerb: core::ptr::null(),
        idCmd: 0,
    },
];
unsafe extern "C" fn PathGetAppFromCommandLine(pszIn: PCWSTR, pszOut: PWSTR, cchMax: usize) -> usize {
    let mut count = 0;
    let mut stop = ' ' as u16;
    let mut pszIn = pszIn;
    if *pszIn.offset(0) == '"' as u16 {
        stop = *pszIn;
        pszIn = pszIn.offset(1);
    }

    let mut pwszSrc = pszIn;
    let mut pszOut = pszOut;
    while *pwszSrc != 0 && *pwszSrc != stop {
        count += 1;
        if count >= cchMax {
            return 0;
        }
        *pszOut = *pwszSrc;
        pszOut = pszOut.offset(1);
        pwszSrc = pwszSrc.offset(1);
    }
    *pszOut = UNICODE_NULL;
    count
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_GetDllFromRundll32CommandLine(pszCmd: PCWSTR, pszOut: PWSTR, cchMax: usize) -> i32 {
    let mut szDll: [u16; MAX_PATH as usize + 100] = [0; MAX_PATH as usize + 100];
    if PathGetAppFromCommandLine(pszCmd, szDll.as_mut_ptr(), _countof!(szDll) as usize) == 0 {
        return HRESULT_FROM_WIN32(ERROR_BUFFER_OVERFLOW as LONG);
    }

    let pszName: PWSTR = PathFindFileNameW(szDll.as_mut_ptr());
    if _wcsicmp(pszName, CPP_COM_WSTR_D767B87F69DC.as_ptr()) != 0 && _wcsicmp(pszName, CPP_COM_WSTR_51AA45493C0B.as_ptr()) != 0 {
        return E_UNEXPECTED;
    }

    let mut pszDllStart: PCWSTR = pszCmd.offset((pszName.offset_from(szDll.as_ptr()) + lstrlenW(pszName) as isize) as isize);

    if *pszDllStart == '\"' as u16 {
        pszDllStart = pszDllStart.offset(1);
    }
    while *pszDllStart <= ' ' as u16 && *pszDllStart != 0 {
        pszDllStart = pszDllStart.offset(1);
    }
    if PathGetAppFromCommandLine(pszDllStart, szDll.as_mut_ptr(), _countof!(szDll) as usize) != 0 {
        let quoted: u8 = if *pszDllStart == '\"' as u16 { 1 } else { 0 };
        let mut pszComma: PWSTR = szDll.as_mut_ptr().offset(lstrlenW(szDll.as_mut_ptr()) as isize);
        while quoted == 0 && pszComma > szDll.as_mut_ptr() && *pszComma != ',' as u16 && *pszComma != '\\' as u16 && *pszComma != '/' as u16 {
            pszComma = pszComma.offset(-1);
        }
        let cch: usize = pszComma.offset_from(szDll.as_ptr()) as usize;
        if cch <= cchMax && (quoted != 0 || *pszComma == ',' as u16) {
            *pszComma = UNICODE_NULL;
            lstrcpynW(pszOut, szDll.as_ptr(), cchMax as INT);
            return S_OK;
        }
    }
    HRESULT_FROM_WIN32(ERROR_BUFFER_OVERFLOW as LONG)
}
unsafe extern "C" fn SH32_EvaluateValidExecApp(pszCmd: PWSTR, cchMax: usize) -> HRESULT {
    if PathGetAppFromCommandLine(pszCmd, pszCmd, cchMax) == 0 {
        return E_FAIL;
    }

    let fPRF: u32 = PRF_VERIFYEXISTS | PRF_TRYPROGRAMEXTENSIONS | PRF_DONTFINDLNK;
    let mut szCurrDir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let pszDirsArr: [PCWSTR; 2] = [szCurrDir.as_ptr(), core::ptr::null()];
    let ppszDirs: *const PCWSTR = if GetCurrentDirectoryW(_countof!(szCurrDir), szCurrDir.as_mut_ptr()) != 0 {
        pszDirsArr.as_ptr()
    } else {
        core::ptr::null()
    };

    if PathResolveW(pszCmd, ppszDirs as *mut PCWSTR, fPRF | (if !ppszDirs.is_null() { PRF_FIRSTDIRDEF } else { 0 })) != 0 {
        return S_OK;
    }
    E_FAIL
}
#[no_mangle]
pub unsafe extern "C" fn SH32_InvokeOpenWith(
    pszPath: PCWSTR,
    pici: LPCMINVOKECOMMANDINFO,
    phProcess: *mut HANDLE,
) -> HRESULT {
    if pszPath.is_null() || pici.is_null() {
        return HResultFromWin32(ERROR_INVALID_PARAMETER);
    }

    let mut hr: HRESULT = HResultFromWin32(ERROR_NO_ASSOCIATION);
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD,
        fMask: CmicFlagsToSeeFlags((*pici).fMask),
        hwnd: (*pici).hwnd,
        lpVerb: core::ptr::null(),
        lpFile: core::ptr::null(),
        lpParameters: core::ptr::null(),
        lpDirectory: core::ptr::null(),
        nShow: 0,
        hInstApp: core::ptr::null_mut(),
        lpIDList: core::ptr::null_mut(),
        lpClass: core::ptr::null(),
        hkeyClass: core::ptr::null_mut(),
        dwHotKey: 0,
        __bindgen_anon_1: core::mem::zeroed(),
        hProcess: core::ptr::null_mut(),
    };
    sei.fMask |= SEE_MASK_CLASSKEY | SEE_MASK_NOZONECHECKS;
    sei.lpFile = pszPath;
    sei.nShow = (*pici).nShow;
    if !phProcess.is_null() {
        sei.fMask |= SEE_MASK_NOCLOSEPROCESS;
        sei.hProcess = core::ptr::null_mut();
    }

    if RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        CPP_COM_WSTR_F3CDEC9800CB.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(sei.hkeyClass),
    ) == 0
    {
        let mut szCmd: [WCHAR; MAX_PATH as usize * 2] =
            [0; MAX_PATH as usize * 2];
        let mut cch: DWORD = _countof!(szCmd);
        hr = AssocQueryStringByKeyW(
            ASSOCF_NOTRUNCATE
                | ASSOCF_NOFIXUPS
                | ASSOCF_IGNOREBASECLASS
                | ASSOCF_INIT_IGNOREUNKNOWN,
            ASSOCSTR_COMMAND,
            sei.hkeyClass,
            core::ptr::null_mut(),
            szCmd.as_mut_ptr(),
            core::ptr::addr_of_mut!(cch),
        );
        if hr >= 0
            && SH32_EvaluateValidExecApp(szCmd.as_mut_ptr(), _countof!(szCmd) as usize) < 0
        {
            let mut info = OPENASINFO {
                pcszFile: pszPath,
                pcszClass: core::ptr::null(),
                oaifInFlags: (OAIF_EXEC
                    | OAIF_REGISTER_EXT
                    | OAIF_ALLOW_REGISTRATION) as INT,
            };
            hr = SHOpenWithDialog(sei.hwnd, core::ptr::addr_of_mut!(info));
        } else {
            hr = if ShellExecuteExW(core::ptr::addr_of_mut!(sei)) != 0 {
                S_OK
            } else {
                HResultFromWin32(GetLastError())
            };
        }
        RegCloseKey(sei.hkeyClass);
    } else if ((*pici).fMask & CMIC_MASK_FLAG_NO_UI) == 0 {
        SHELL_ErrorBox(sei.hwnd.cast(), hr as UINT);
    }

    if !phProcess.is_null() {
        *phProcess = sei.hProcess;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn Shell32RustInvokeOpenWith(
    pszPath: PCWSTR,
    pici: LPCMINVOKECOMMANDINFO,
    phProcess: *mut HANDLE,
) -> HRESULT {
    SH32_InvokeOpenWith(pszPath, pici, phProcess)
}
#[no_mangle]
pub unsafe extern "system" fn COpenWithList_Create_RustBridge() -> *mut COpenWithList {
    COpenWithList_new()
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_Destroy_RustBridge(pList: *mut COpenWithList) {
    if !pList.is_null() {
        COpenWithList_delete(pList);
    }
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_Load_RustBridge(pList: *mut COpenWithList) -> i32 {
    COpenWithList_Load_body(pList)
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_LoadRecommended_RustBridge(
    pList: *mut COpenWithList,
    pszPath: PCWSTR,
) -> BOOL {
    COpenWithList_LoadRecommended_body(pList, pszPath)
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_GetRecommendedCount_RustBridge(
    pList: *mut COpenWithList,
) -> u32 {
    COpenWithList_GetRecommendedCount_body(pList)
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_GetCount_RustBridge(pList: *mut COpenWithList) -> u32 {
    COpenWithList_GetCount_body(pList)
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_GetAt_RustBridge(
    pList: *mut COpenWithList,
    Index: u32,
) -> *mut core::ffi::c_void {
    if Index < COpenWithList_GetCount_body(pList) {
        (COpenWithList_GetList_body(pList) as *mut u8).add((Index as usize) * core::mem::size_of::<SApp>()) as *mut core::ffi::c_void
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_IsRecommended_RustBridge(pApp: PVOID) -> BOOL {
    (!pApp.is_null() && (*(pApp as *mut SApp)).bRecommended != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_GetName_RustBridge(pApp: PVOID) -> PCWSTR {
    return COpenWithList_GetName_body(core::ptr::null_mut(), pApp as *mut SApp);
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_GetIcon_RustBridge(pApp: PVOID) -> HICON {
    return COpenWithList_GetIcon_body(core::ptr::null_mut(), pApp as *mut SApp);
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithList_Execute_RustBridge(
    pApp: PVOID,
    pszPath: PCWSTR,
) -> BOOL {
    COpenWithList_Execute_body(core::ptr::null_mut(), pApp as *mut SApp, pszPath)
}

#[no_mangle]
pub unsafe extern "system" fn COpenWithMenu_ExtractPath_RustBridge(
    pDataObject: *mut IDataObject,
    pszPath: PWSTR,
    cchPath: u32,
) -> i32 {
    if pDataObject.is_null() || pszPath.is_null() || cchPath < MAX_PATH {
        return E_INVALIDARG;
    }

    let mut pida = CDataObjectHIDA::new(pDataObject);
    if 0 > pida.hr() {
        return pida.hr();
    }
    if (*pida.as_ptr()).cidl < 1 {
        return E_FAIL;
    }

    let pidlFolder: LPCITEMIDLIST = HIDA_GetPIDLFolder(pida.as_ptr());
    let pidlChild: LPCITEMIDLIST = HIDA_GetPIDLItem(pida.as_ptr(), 0);
    if _ILIsValue(pidlChild) == 0 {
        return E_FAIL;
    }

    let pidl: *mut ITEMIDLIST = ILCombine(pidlFolder, pidlChild);
    if pidl.is_null() {
        return E_OUTOFMEMORY;
    }

    let result = if SHGetPathFromIDListW(pidl, pszPath) != 0 {
        S_OK
    } else {
        E_FAIL
    };

    CoTaskMemFree(pidl as *mut _);

    result
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenWithDialog(
    hwndParent: HWND,
    poainfo: *const OPENASINFO,
) -> HRESULT {
    let ret: INT_PTR;

    InitCommonControls();

    if (*poainfo).pcszClass.is_null() && (*poainfo).pcszFile.is_null() {
        return E_FAIL;
    }

    // `COpenWithDialog pDialog(poainfo);` — a STACK object whose destructor runs at every
    // scope exit, so the dtor is called on both return paths below. The translation had
    // `COpenWithDialog::new(...)` / `pDialog.IsNoOpen(...)`, which is C++ syntax, not Rust.
    let mut storage = core::mem::MaybeUninit::<COpenWithDialog>::zeroed();
    let pDialog = storage.as_mut_ptr();
    COpenWithDialog_ctor_body(pDialog, poainfo, core::ptr::null_mut());

    if COpenWithDialog_IsNoOpen_body(pDialog, hwndParent) != 0 {
        COpenWithDialog_dtor_body(pDialog);
        return S_OK;
    }

    // C++ passes `(LPARAM)&pDialog`, the address of the object. `pDialog` is already that
    // address; the translation's `&pDialog as *const _` would have passed a pointer TO the
    // pointer, and WM_INITDIALOG would have stored garbage in DWLP_USER.
    ret = DialogBoxParamW(
        shell32_hInstance,
        MAKEINTRESOURCE(IDD_OPEN_WITH),
        hwndParent,
        Some(COpenWithDialog_DialogProc_thunk),
        pDialog as LPARAM,
    );
    COpenWithDialog_dtor_body(pDialog);

    if ret == -1 as INT_PTR {
        return E_FAIL;
    }

    S_OK
}

/// DLGPROC entry for `COpenWithDialog::DialogProc`, a C++ STATIC member. The body recovers
/// its own receiver from `DWLP_USER` (and from `lParam` on WM_INITDIALOG) exactly as the C++
/// static does, so the `this` the scaffold insists on is vestigial — see the standing
/// "static members get a bogus `this`" item.
unsafe extern "system" fn COpenWithDialog_DialogProc_thunk(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    COpenWithDialog_DialogProc_body(core::ptr::null_mut(), hwndDlg, uMsg, wParam, lParam)
}
