use crate::prelude::*;
use crate::fontext::precomp_h_h::IsFontDotExt;
use crate::fontext::CFontCache_cpp_h::g_FontCache;
macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_13C2F0CCF85C: [WCHAR; 5] = [0x002E, 0x006F, 0x0074, 0x0066, 0x0000];
pub static CPP_COM_WSTR_47F50243F401: [WCHAR; 5] = [0x002E, 0x0074, 0x0074, 0x0066, 0x0000];
pub static CPP_COM_WSTR_49342E15DCFE: [WCHAR; 6] = [0x0066, 0x006F, 0x006E, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_5A737710FEF3: [WCHAR; 5] = [0x002E, 0x006F, 0x0074, 0x0063, 0x0000];
pub static CPP_COM_WSTR_5B20018FE43E: [WCHAR; 5] = [0x002E, 0x0074, 0x0074, 0x0063, 0x0000];
pub static CPP_COM_WSTR_5E94DAE7ECC3: [WCHAR; 35] = [0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074, 0x0025, 0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0066, 0x006F, 0x006E, 0x0074, 0x0076, 0x0069, 0x0065, 0x0077, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_84C5060F40E8: [WCHAR; 6] = [0x002F, 0x0064, 0x0020, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_ED33FC35514E: [WCHAR; 12] = [0x0020, 0x0028, 0x0054, 0x0072, 0x0075, 0x0065, 0x0054, 0x0079, 0x0070, 0x0065, 0x0029, 0x0000];

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
        let hr = (((*(*pDataObject).lpVtbl).GetData).unwrap_unchecked())(
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

    /// Mirrors the C++ public static. It is not only the destructor's body:
    /// dll/shellext/cabview/folder.cpp calls CreateCIDA/DestroyCIDA directly to
    /// hold a CIDA across a scope the guard cannot span, so it must stay callable.
    /// shell32-rust's prelude re-exports it and move_to_menu.rs reaches it through
    /// CDataObjectHIDA_DestroyCIDA_RustBridge — a lowering without it links short.
    unsafe fn DestroyCIDA(pcida: *const CIDA, medium: *mut STGMEDIUM) {
        if !pcida.is_null() {
            GlobalUnlock((*medium).__bindgen_anon_1.hGlobal);
        }
        ReleaseStgMedium(medium);
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
        unsafe { CDataObjectHIDA::DestroyCIDA(self.m_cida, &mut self.m_medium) }
    }
}

use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static CLSID_CFontExt: GUID = GUID {
    Data1: 0xbd84b380,
    Data2: 0x8ca2,
    Data3: 0x1069,
    Data4: [0xab, 0x1d, 0x08, 0x00, 0x09, 0x48, 0xf5, 0x34],
};

pub static mut g_ModuleRefCnt: LONG = 0;

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if g_ModuleRefCnt != 0 {
        return S_FALSE;
    }
    return gModule.DllCanUnloadNow();
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: *const GUID, ppv: *mut LPVOID) -> HRESULT {
    return (*core::ptr::addr_of_mut!(gModule)).DllGetClassObject(rclsid, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let mut Path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let DesktopIniContents: &[u8] = b"[.ShellClassInfo]\r\nCLSID={BD84B380-8CA2-1069-AB1D-08000948F534}\r\nIconResource=%SystemRoot%\\system32\\shell32.dll,-39\r\n\0"; // IDI_SHELL_FONTS_FOLDER
    let mut hFile: HANDLE;
    let mut hr: HRESULT;

    hr = gModule.DllRegisterServer((FALSE as i32));
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = SHGetFolderPathW(core::ptr::null_mut(), (CSIDL_FONTS as i32), core::ptr::null_mut(), 0, Path.as_mut_ptr());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    // Make this a system folder:
    // Ideally this should not be done here, but when installing
    // Otherwise, livecd won't have this set properly
    let mut dwAttributes: DWORD = GetFileAttributesW(Path.as_ptr());
    if dwAttributes != INVALID_FILE_ATTRIBUTES {
        dwAttributes |= FILE_ATTRIBUTE_SYSTEM;
        SetFileAttributesW(Path.as_ptr(), dwAttributes);
    } else {
    }

    if ((PathAppendW(Path.as_mut_ptr(), CPP_COM_WSTR_A5DBFDEBD401.as_ptr())) == 0) {
        return E_FAIL;
    }

    hFile = CreateFileW(Path.as_ptr(), GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN, core::ptr::null_mut());
    if hFile == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    let mut BytesWritten: DWORD = 0;
    let mut BytesToWrite: DWORD = core::ffi::CStr::from_ptr(DesktopIniContents.as_ptr() as *const core::ffi::c_char).to_bytes().len() as DWORD;
    if WriteFile(hFile, DesktopIniContents.as_ptr() as *const core::ffi::c_void, BytesToWrite, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut()) != 0 {
        hr = if BytesToWrite == BytesWritten { S_OK } else { E_FAIL };
    } else {
        hr = HRESULT_FROM_WIN32!(GetLastError());
    }
    CloseHandle(hFile);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return gModule.DllUnregisterServer(FALSE as i32);
}
#[no_mangle]
pub unsafe extern "C" fn CloseRegKeyArray(array: *mut HKEY, cKeys: UINT) {
    let mut i: UINT = 0;
    while i < cKeys {
        RegCloseKey(*array.add(i as usize));
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn AddClassKeyToArray(szClass: *const WCHAR, array: *mut HKEY, cKeys: *mut UINT) -> LSTATUS {
    if *cKeys >= 16 {
        return (ERROR_MORE_DATA) as LSTATUS;
    }

    let mut key: CRegKey = CRegKey__make_0();
    let mut error: LSTATUS = key.Open(HKEY_CLASSES_ROOT, szClass, KEY_READ | KEY_QUERY_VALUE);
    if error == (ERROR_SUCCESS as LSTATUS) {
        *array.add(*cKeys as usize) = key.Detach();
        *cKeys += 1;
    }
    return error;
}
#[no_mangle]
pub unsafe extern "C" fn InstallFontFiles(pData: PINSTALL_FONT_DATA) -> HRESULT {
    let pidlParent: PCUIDLIST_ABSOLUTE = (*pData).pidlParent;
    let cidl: UINT = (*pData).cSteps;
    let apidl: PCUITEMID_CHILD_ARRAY = (*pData).apidl;

    let mut FontPaths = CAtlArray::<CStringW>::new();
    let mut n: UINT = 0;
    while n < cidl {
        let mut pidl = CComHeapPtr::<ITEMIDLIST_ABSOLUTE>::new(); // ILCombine is CoTaskMem
        pidl.Attach(ILCombine(pidlParent, (*apidl.add(n as usize))));
        if pidl.is_null() {
            return E_OUTOFMEMORY;
        }

        let mut szPath = [0u16; (MAX_PATH) as _];
        if SHGetPathFromIDListW(pidl.Get(), szPath.as_mut_ptr()) == 0
            || PathIsDirectoryW(szPath.as_ptr()) != 0
            || IsFontDotExt(PathFindExtensionW(szPath.as_ptr())) == 0
        {
            return E_FAIL;
        }

        FontPaths.Add(CStringW::from_ptr(szPath.as_ptr()));

        n += 1;
    }

    let mut keyFonts = CRegKey__make_0();
    if keyFonts.Open(FONT_HIVE, FONT_KEY.as_ptr(), KEY_WRITE) != (ERROR_SUCCESS as i32) {
        return E_FAIL;
    }

    let mut iItem: SIZE_T = 0;
    while iItem < (FontPaths.GetCount()) as _ {
        if (*pData).bCanceled != 0 {
            return E_ABORT;
        }

        let hr: HRESULT = DoInstallFontFile((*FontPaths.GetAt(iItem as usize)).GetString(), ((*g_FontCache).FontPath()).GetString(), (keyFonts).op_to_HKEY());
        if FAILED_UNEXPECTEDLY!(hr) != false {
            return hr;
        }

        if !(*pData).hwnd.is_null() {
            PostMessageW((*pData).hwnd, WM_COMMAND, (IDCONTINUE) as WPARAM, 0);
        }

        iItem += 1;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn DoInstallFontFile(pszFontPath: PCWSTR, pszFontsDir: PCWSTR, hkeyFonts: HKEY) -> HRESULT {
    ATLASSERT!(!pszFontPath.is_null());
    ATLASSERT!(!pszFontsDir.is_null());
    ATLASSERT!(!hkeyFonts.is_null());

    // Add this font to the font list, so we can query the name
    if AddFontResourceW(pszFontPath) == 0
    {
        return E_FAIL;
    }

    // Get the font name
    let mut strFontName = CStringW::new();
    let mut hr: HRESULT = DoGetFontTitle(pszFontPath, &mut strFontName);
    if FAILED_UNEXPECTEDLY!(hr)
    {
        return hr;
    }

    // Remove it now
    // WINDOWS BUG: Removing once is not enough
    let mut iTry: INT = 0;
    while iTry < 3
    {
        if RemoveFontResourceW(pszFontPath) == 0
            && RemoveFontResourceExW(pszFontPath, FR_PRIVATE, core::ptr::null_mut()) == 0
        {
            break;
        }
        iTry += 1;
    }

    // Delete font entry in registry
    RegDeleteValueW(hkeyFonts, strFontName.GetString());

    let pszFileTitle: LPCWSTR = PathFindFileName(pszFontPath);
    ATLASSERT!(!pszFileTitle.is_null());

    // Build destination path
    let mut szDestFile = CStringW::from_ptr(pszFontsDir); // pszFontsDir has backslash at back
    szDestFile += pszFileTitle;

    if StrCmpIW(szDestFile.GetString(), pszFontPath) == 0 // Same file?
    {
        return E_FAIL;
    }

    // Copy file
    if CopyFileW(pszFontPath, szDestFile.GetString(), (FALSE) as BOOL) == 0
    {
        return E_FAIL;
    }

    // Write registry for font entry
    let cbData: DWORD = ((wcslen(pszFileTitle) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
    let nError: LONG = RegSetValueExW(hkeyFonts, strFontName.GetString(), 0, REG_SZ,
                                 pszFileTitle as *const BYTE, cbData);
    if nError != 0
    {
        DeleteFileW(szDestFile.GetString());
        return E_FAIL;
    }

    // Notify file creation
    SHChangeNotify((SHCNE_CREATE) as LONG, SHCNF_PATHW, (szDestFile.GetString()).cast(), core::ptr::null_mut());

    return if AddFontResourceW(szDestFile.GetString()) != 0 { S_OK } else { E_FAIL };
}
#[no_mangle]
pub unsafe extern "C" fn DoGetFontTitle(pszFontPath: LPCWSTR, strFontName: *mut CStringW) -> HRESULT {
    let mut cbInfo: DWORD = 0;
    let mut ret: BOOL = GetFontResourceInfoW(pszFontPath, core::ptr::addr_of_mut!(cbInfo), core::ptr::null_mut(), 1);
    if ret == 0 || cbInfo == 0 {
        return E_FAIL;
    }

    let mut pszBuffer: LPWSTR = (*strFontName).GetBuffer((cbInfo / core::mem::size_of::<WCHAR>() as DWORD) as i32);
    ret = GetFontResourceInfoW(pszFontPath, core::ptr::addr_of_mut!(cbInfo), (pszBuffer).cast(), 1);
    let dwErr: DWORD = GetLastError();
    (*strFontName).ReleaseBuffer();
    if ret == 0 {
        return E_FAIL;
    }

    let pchDotExt: LPCWSTR = PathFindExtensionW(pszFontPath);
    if StrCmpIW(pchDotExt, CPP_COM_WSTR_47F50243F401.as_ptr()) == 0 || StrCmpIW(pchDotExt, CPP_COM_WSTR_5B20018FE43E.as_ptr()) == 0 ||
        StrCmpIW(pchDotExt, CPP_COM_WSTR_13C2F0CCF85C.as_ptr()) == 0 || StrCmpIW(pchDotExt, CPP_COM_WSTR_5A737710FEF3.as_ptr()) == 0
    {
        *strFontName += CPP_COM_WSTR_ED33FC35514E.as_ptr();
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn CheckDropFontFiles(hDrop: HDROP) -> BOOL {
    let cFiles: UINT = DragQueryFileW(hDrop, 0xFFFFFFFF, core::ptr::null_mut(), 0);
    if cFiles == 0 {
        return (FALSE) as BOOL;
    }

    let mut iFile: UINT = 0;
    while iFile < cFiles {
        let mut szFile: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
        if DragQueryFileW(hDrop, iFile, szFile.as_mut_ptr(), _countof!(szFile)) == 0 {
            return (FALSE) as BOOL;
        }
        let pchDotExt: LPCWSTR = PathFindExtensionW(szFile.as_mut_ptr());
        if IsFontDotExt(pchDotExt) == 0 {
            return (FALSE) as BOOL;
        }
        iFile += 1;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CheckDataObject(pDataObj: *mut IDataObject) -> BOOL {
    let mut stg: STGMEDIUM = core::mem::zeroed();
    let mut etc = FORMATETC {
        cfFormat: (CF_HDROP as u16),
        ptd: core::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT,
        lindex: -1,
        tymed: TYMED_HGLOBAL,
    };
    let hr: HRESULT = (((*(*pDataObj).lpVtbl).GetData).unwrap_unchecked())(
        pDataObj,
        core::ptr::addr_of_mut!(etc),
        core::ptr::addr_of_mut!(stg),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return (FALSE) as BOOL;
    }
    let hDrop: HDROP = stg.__bindgen_anon_1.hGlobal as HDROP;
    let bOK: BOOL = CheckDropFontFiles(hDrop);
    ReleaseStgMedium(core::ptr::addr_of_mut!(stg));
    return bOK;
}

unsafe extern "system" fn InstallThreadProc(lpParameter: LPVOID) -> DWORD {
    let mut pData: PINSTALL_FONT_DATA = lpParameter as PINSTALL_FONT_DATA;
    ATLASSERT!(!pData.is_null());
    (*pData).hrResult = InstallFontFiles(pData);
    if (*pData).bCanceled != 0 {
        (*pData).hrResult = S_FALSE;
    }
    PostMessageW((*pData).hwnd, WM_COMMAND, (IDOK) as WPARAM, 0);
    (*(*pData).pDataObj).Release();
    return 0;
}

unsafe extern "system" fn InstallDlgProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pData: PINSTALL_FONT_DATA) -> INT_PTR {
    if uMsg == WM_INITDIALOG {
        (*pData).hwnd = hwnd;
        ATLASSERT!((*pData).cSteps >= 0);
        SendDlgItemMessageW(hwnd, (IDC_INSTALL_PROGRESS as i32), PBM_SETRANGE, 0, MAKELPARAM!(0, (*pData).cSteps));
        if !(SHCreateThread(Some(InstallThreadProc), pData as *mut c_void, CTF_COINIT, None) != 0) {
            (*(*pData).pDataObj).Release();
            (*pData).hrResult = E_ABORT;
            EndDialog(hwnd, (IDABORT) as INT_PTR);
        }
        return (TRUE) as INT_PTR;
    } else if uMsg == WM_COMMAND {
        if LOWORD!(wParam) == IDOK {
            EndDialog(hwnd, (IDOK) as INT_PTR);
        } else if LOWORD!(wParam) == IDCANCEL {
            (*pData).bCanceled = (TRUE as i32);
            EndDialog(hwnd, (IDCANCEL) as INT_PTR);
        } else if LOWORD!(wParam) == IDCONTINUE {
            (*pData).iStep += 1;
            ATLASSERT!((*pData).iStep <= (*pData).cSteps);
            SendDlgItemMessageW(hwnd, (IDC_INSTALL_PROGRESS as i32), PBM_SETPOS, ((*pData).iStep) as WPARAM, 0);
        }
    }
    return 0;
}

unsafe extern "system" fn InstallDialogProc(hwnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pData: PINSTALL_FONT_DATA = GetWindowLongPtrW(hwnd, DWLP_USER) as PINSTALL_FONT_DATA;
    if uMsg == WM_INITDIALOG {
        pData = lParam as PINSTALL_FONT_DATA;
        SetWindowLongPtrW(hwnd, DWLP_USER, lParam);
    }

    ATLASSERT!(!pData.is_null());
    return InstallDlgProc(hwnd, uMsg, wParam, lParam, pData);
}
#[no_mangle]
pub unsafe extern "C" fn InstallFontsFromDataObject(hwndView: HWND, pDataObj: *mut IDataObject) -> HRESULT {
    if !(CheckDataObject(pDataObj) != 0) {
        return E_FAIL;
    }

    let cida = CDataObjectHIDA::new(pDataObj);
    if !cida.is_valid() || (*cida.as_ptr()).cidl <= 0 {
        return E_FAIL;
    }

    let pidlParent: PCUIDLIST_ABSOLUTE = HIDA_GetPIDLFolder(cida.as_ptr());
    if pidlParent.is_null() {
        return E_FAIL;
    }

    let mut apidl: CAtlArray<PCUIDLIST_RELATIVE> = CAtlArray::new();
    let mut n: UINT = 0;
    while n < (*cida.as_ptr()).cidl {
        let pidlRelative: PCUIDLIST_RELATIVE = HIDA_GetPIDLItem(cida.as_ptr(), n);
        if pidlRelative.is_null() {
            return E_FAIL;
        }
        apidl.Add(pidlRelative);
        n += 1;
    }

    // Show progress dialog
    let mut data: INSTALL_FONT_DATA = core::mem::zeroed();
    data.pDataObj = pDataObj;
    data.pidlParent = pidlParent;
    data.apidl = apidl.GetAt(0) as _;
    data.cSteps = (*cida.as_ptr()).cidl;
    (((*(*pDataObj).lpVtbl).AddRef).unwrap_unchecked())(pDataObj);
    DialogBoxParamW(
        _AtlBaseModule.GetResourceInstance(),
        MAKEINTRESOURCEW!(IDD_INSTALL),
        hwndView,
        Some(InstallDialogProc),
        core::ptr::addr_of_mut!(data) as LPARAM,
    );
    if data.bCanceled != 0 {
        return S_FALSE;
    }

    if FAILED_UNEXPECTEDLY!(data.hrResult) != false {
        E_FAIL
    } else {
        S_OK
    }
}
#[no_mangle]
pub unsafe extern "C" fn DoPreviewFontFiles(hwnd: HWND, pDataObj: *mut IDataObject) -> HRESULT {
    let mut cida = CDataObjectHIDA::new(pDataObj);
    if !cida.is_valid() || (*cida.as_ptr()).cidl <= 0 {
        return E_FAIL;
    }

    let mut iItem: UINT = 0;
    while iItem < (*cida.as_ptr()).cidl {
        let pidlChild: PCUIDLIST_RELATIVE = HIDA_GetPIDLItem(cida.as_ptr(), iItem);
        let fontEntry: *const FontPidlEntry = _FontFromIL(pidlChild);
        if !fontEntry.is_null() {
            RunFontViewer(hwnd, fontEntry);
        }
        iItem += 1;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteFontFiles(hwnd: HWND, pDataObj: *mut IDataObject) -> HRESULT {
    let mut cida = CDataObjectHIDA::new(pDataObj);
    if !cida.is_valid() || (*cida.as_ptr()).cidl <= 0 {
        return E_FAIL;
    }

    let pidlParent: PCUIDLIST_ABSOLUTE = HIDA_GetPIDLFolder(cida.as_ptr());
    if pidlParent.is_null() {
        return E_FAIL;
    }

    // Delete files
    let mut iItem: UINT = 0;
    while iItem < (*cida.as_ptr()).cidl {
        let pidlChild = HIDA_GetPIDLItem(cida.as_ptr(), iItem);
        let pEntry = _FontFromIL(pidlChild);
        if pEntry.is_null() {
            return E_FAIL;
        }

        let szPath = (*g_FontCache).GetFontFilePath((*pEntry.cast_mut()).FileName__1());

        // WINDOWS BUG: Removing once is not enough
        let mut iTry: INT = 0;
        while iTry < 3 {
            if ((RemoveFontResourceW(szPath.GetString())) == 0) && ((RemoveFontResourceExW(szPath.GetString(), FR_PRIVATE, core::ptr::null_mut())) == 0) {
                break;
            }
            iTry += 1;
        }

        if ((DeleteFileW(szPath.GetString())) == 0) {
            return E_FAIL;
        }

        let pidl = CComHeapPtr::<ITEMIDLIST_ABSOLUTE>::from_raw(ILCombine(pidlParent, pidlChild));
        if !pidl.is_null() {
            SHChangeNotify((SHCNE_DELETE) as LONG, SHCNF_IDLIST, (pidl.as_ptr() as LPCITEMIDLIST).cast(), core::ptr::null_mut());
        } else {
            ERR!("Out of memory\n");
        }
        iItem += 1;
    }

    // Delete registry values and mark the entry as deleted
    let mut key = CRegKey__make_0();
    if key.Open(FONT_HIVE, FONT_KEY.as_ptr(), KEY_WRITE) == (ERROR_SUCCESS as i32) {
        let mut iItem: UINT = 0;
        while iItem < (*cida.as_ptr()).cidl {
            let pidlChild = HIDA_GetPIDLItem(cida.as_ptr(), iItem);
            let pEntry = _FontFromIL(pidlChild);
            if !pEntry.is_null() {
                let strFontName = (*pEntry.cast_mut()).Name__1();
                key.DeleteValue(strFontName);
                (*g_FontCache).MarkDeleted(pEntry);
            }
            iItem += 1;
        }
        key.Close();
    }

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn DoDeleteFontFiles(hwnd: HWND, pDataObj: *mut IDataObject) -> HRESULT {
    let mut title = CStringW::from_resource(IDS_REACTOS_FONTS_FOLDER);
    let mut msg = CStringW::from_resource(IDS_CONFIRM_DELETE_FONT);
    if MessageBoxW(hwnd, msg.GetString(), title.GetString(), MB_YESNOCANCEL | MB_ICONWARNING) != (IDYES as i32) {
        return S_FALSE;
    }

    let mut hr: HRESULT = DeleteFontFiles(hwnd, pDataObj);
    if FAILED_UNEXPECTEDLY!(hr) != false {
        msg.LoadStringW(IDS_CANTDELETEFONT);
        MessageBoxW(hwnd, msg.GetString(), title.GetString(), MB_ICONERROR);
        return hr;
    }

    PostMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, CPP_COM_WSTR_49342E15DCFE.as_ptr() as LPARAM);
    PostMessageW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn RunFontViewer(hwnd: HWND, fontEntry: *const FontPidlEntry) {
    let Path = (*g_FontCache).GetFontFilePath((*fontEntry.cast_mut()).FileName__1());
    if Path.IsEmpty() {
        return;
    }

    // '/d' disables the install button
    let mut FontPathArg: [WCHAR; (MAX_PATH + 3) as _] = [0; (MAX_PATH + 3) as _];
    StringCchPrintfW!(FontPathArg.as_mut_ptr(), _countof!(FontPathArg), CPP_COM_WSTR_84C5060F40E8.as_ptr(), Path.GetString());
    PathQuoteSpacesW(FontPathArg.as_mut_ptr().add(3));

    let mut si: SHELLEXECUTEINFOW = core::mem::zeroed();
    si.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD;
    si.fMask = SEE_MASK_DOENVSUBST;
    si.hwnd = hwnd;
    si.lpFile = CPP_COM_WSTR_5E94DAE7ECC3.as_ptr();
    si.lpParameters = FontPathArg.as_ptr();
    si.nShow = (SW_SHOWNORMAL as i32);
    ShellExecuteExW(core::ptr::addr_of_mut!(si));
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstance);
            CFontExtModule_Init(core::ptr::addr_of_mut!(gModule), ObjectMap.as_ptr(), hInstance, core::ptr::null::<GUID>());
        }
        DLL_PROCESS_DETACH => {
            CFontExtModule_Term(core::ptr::addr_of_mut!(gModule));
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CFontExtModule_Init(this_: *mut atl_rust::dll::CComModule, p: *const atl_rust::dll::ObjMapEntry, h: HINSTANCE, plibid: *const GUID) {
    g_FontCache = CFontCache__new_0();
    (*this_).Init(p, h, plibid);
}
#[no_mangle]
pub unsafe extern "C" fn CFontExtModule_Term(this_: *mut atl_rust::dll::CComModule) {
    CFontCache__delete(g_FontCache);
    g_FontCache = core::ptr::null_mut();
    (*this_).Term();
}
