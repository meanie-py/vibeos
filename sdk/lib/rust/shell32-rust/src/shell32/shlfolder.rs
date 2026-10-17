//! `shlfolder.cpp`, translated and LIVE — `shlfolder.cpp` is commented out of the shell32
//! CMakeLists. See `shell32/mod.rs` for the EXTERN_C surface this flip needed and for why the
//! `SHELL32_CoCreateInitSF` overload pair had to become two distinct names.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_32491EF24F87: [WCHAR; 6] = [0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0000];
pub static CPP_COM_WSTR_3A7BCE312332: [WCHAR; 10] = [0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079, 0x0000];
pub static CPP_COM_WSTR_6E342AB1A0F1: [WCHAR; 7] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];
pub static CPP_COM_WSTR_714C3CA85096: [WCHAR; 14] = [0x0050, 0x0065, 0x0072, 0x0063, 0x0065, 0x0069, 0x0076, 0x0065, 0x0064, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static CPP_COM_WSTR_84B1405B4E34: [WCHAR; 9] = [0x002F, 0x0073, 0x0065, 0x006C, 0x0065, 0x0063, 0x0074, 0x002C, 0x0000];
pub static CPP_COM_WSTR_A8DA479F8D2D: [WCHAR; 21] = [0x0041, 0x006C, 0x006C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x004F, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_AB68B18B313D: [WCHAR; 13] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_E8A4B2EE7EDE: [WCHAR; 2] = [0x002A, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_F38FD69158A5: [WCHAR; 26] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0041, 0x0073, 0x0073, 0x006F, 0x0063, 0x0069, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_F3CDEC9800CB: [WCHAR; 8] = [0x0055, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0000];

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
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
        let hr = ((*(*pDataObject).lpVtbl).GetData)(pDataObject, core::ptr::addr_of_mut!(format).cast(), medium.cast());
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
use crate::prelude::*;
use crate::shell32::sdk_consts::*;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to emit from this preamble.
// All visible code is function definitions (SHELL32_CompareDetails,
// SHOpenFolderAndSelectItems, and various _RustBridge wrappers), which
// will be translated separately. There are no struct definitions,
// constants, enums, or static tables to extract.
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetDefaultFolderEnumSHCONTF() -> SHCONTF {
    let mut flags: SHCONTF = SHCONTF_FOLDERS | SHCONTF_NONFOLDERS;
    let mut ss: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings((core::ptr::addr_of_mut!(ss)).cast(), SSF_SHOWALLOBJECTS | SSF_SHOWSUPERHIDDEN, FALSE);
    if ss.fShowAllObjects() != 0 {
        flags |= SHCONTF_INCLUDEHIDDEN;
    }
    if ss.fShowSuperHidden() != 0 {
        flags |= SHCONTF_INCLUDESUPERHIDDEN;
    }
    flags
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_IncludeItemInFolderEnum(
    pSF: *mut IShellFolder,
    pidl: PCUITEMID_CHILD,
    Query: SFGAOF,
    Flags: SHCONTF,
) -> BOOL {
    let mut query = Query;
    let hr = ((*(*pSF).lpVtbl).GetAttributesOf)(
        pSF,
        1,
        core::ptr::addr_of!(pidl).cast(),
        core::ptr::addr_of_mut!(query),
    );
    if hr >= 0 {
        if query & SFGAO_NONENUMERATED != 0 {
            return FALSE;
        }
        if (query & SFGAO_HIDDEN) != 0 && (Flags & SHCONTF_INCLUDEHIDDEN) == 0 {
            return FALSE;
        }
        if (query & (SFGAO_HIDDEN | SFGAO_SYSTEM))
            == (SFGAO_HIDDEN | SFGAO_SYSTEM)
            && (Flags & SHCONTF_INCLUDESUPERHIDDEN) == 0
        {
            return FALSE;
        }
        if (Flags & (SHCONTF_FOLDERS | SHCONTF_NONFOLDERS))
            != (SHCONTF_FOLDERS | SHCONTF_NONFOLDERS)
        {
            return if (Flags & SHCONTF_FOLDERS) != 0 {
                (((query & SFGAO_FOLDER) != 0) as u8) as i32
            } else {
                (((query & SFGAO_FOLDER) == 0) as u8) as i32
            };
        }
    }
    TRUE
}
unsafe extern "C" fn MapSCIDToShell32FsColumn(pscid: *const SHCOLUMNID, vt: &mut VARTYPE) -> HRESULT {
    if IsEqualGUID(core::ptr::addr_of!((*pscid).fmtid), core::ptr::addr_of!(FMTID_Storage)) {
        match (*pscid).pid {
            PID_STG_NAME => { *vt = (VT_BSTR) as u16; return (SHFSF_COL_NAME) as i32; }
            PID_STG_SIZE => { *vt = (VT_UI8) as u16; return (SHFSF_COL_SIZE) as i32; }
            PID_STG_STORAGETYPE => { *vt = (VT_BSTR) as u16; return (SHFSF_COL_TYPE) as i32; }
            PID_STG_ATTRIBUTES => { *vt = (VT_BSTR) as u16; return (SHFSF_COL_FATTS) as i32; }
            PID_STG_WRITETIME => { *vt = (VT_DATE) as u16; return (SHFSF_COL_MDATE) as i32; }
            _ => {}
        }
    }
    if IsEqualGUID(core::ptr::addr_of!((*pscid).fmtid), core::ptr::addr_of!(FMTID_SummaryInformation)) && (*pscid).pid == PIDSI_COMMENTS {
        *vt = (VT_BSTR) as u16;
        return (SHFSF_COL_COMMENT) as i32;
    }
    return E_INVALIDARG;
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_MapSCIDToColumn(pSF: *mut IShellFolder2, pscid: *const SHCOLUMNID) -> HRESULT {
    let mut i = 0;
    while i <= SHCIDS_COLUMNMASK {
        let mut scid: SHCOLUMNID = core::mem::zeroed();
        let hr = ((*(*pSF).lpVtbl).MapColumnToSCID)(pSF, i, core::ptr::addr_of_mut!(scid).cast());
        if hr < 0 {
            break;
        }
        if IsEqualPropertyKey(core::ptr::addr_of!(scid), pscid) {
            return i as HRESULT;
        }
        i = i.wrapping_add(1);
    }
    E_FAIL
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetDetailsOfAsStringVariant(
    pSF: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    Column: u32,
    pVar: *mut VARIANT,
) -> i32 {
    V_VT!(pVar) = (VT_EMPTY) as VARTYPE;
    let mut sd: SHELLDETAILS = core::mem::zeroed();
    sd.str_.uType = STRRET_CSTR;
    let mut hr: i32 = ((*(*pSF).lpVtbl).GetDetailsOf)(pSF, pidl.cast(), Column, core::ptr::addr_of_mut!(sd).cast());
    if hr < 0 {
        return hr;
    }
    hr = StrRetToBSTR(
        core::ptr::addr_of_mut!(sd.str_),
        pidl,
        core::ptr::addr_of_mut!(V_BSTR!(pVar)),
    );
    if hr < 0 {
        return hr;
    }
    V_VT!(pVar) = (VT_BSTR) as VARTYPE;
    hr
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetDetailsOfColumnAsVariant(
    pSF: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    Column: u32,
    vt: VARTYPE,
    pVar: *mut VARIANT,
) -> i32 {
    let hr = SHELL_GetDetailsOfAsStringVariant(pSF, pidl, Column, pVar);
    if hr < 0 {
        return hr;
    }
    let mut vt = vt;
    if vt == (VT_EMPTY) as u16 {
        let mut state: SHCOLSTATEF = 0;
        if (((*(*pSF).lpVtbl).GetDefaultColumnState)(pSF, Column, &mut state)) < 0 {
            state = SHCOLSTATE_TYPE_STR;
        }
        if (state & SHCOLSTATE_TYPEMASK) == SHCOLSTATE_TYPE_INT {
            vt = (VT_I8) as u16;
        } else if (state & SHCOLSTATE_TYPEMASK) == SHCOLSTATE_TYPE_DATE {
            vt = (VT_DATE) as u16;
        } else {
            vt = (VT_BSTR) as u16;
        }
    }
    if vt != (VT_BSTR) as u16 {
        VariantChangeType(pVar, pVar, 0, vt);
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn SH32_GetDetailsOfPKeyAsVariant(
    pSF: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    pscid: *const SHCOLUMNID,
    pVar: *mut VARIANT,
    UseFsColMap: BOOL,
) -> HRESULT {
    let mut vt: VARTYPE = (VT_EMPTY) as u16;
    let hr: HRESULT = if UseFsColMap != 0 {
        MapSCIDToShell32FsColumn(pscid, &mut vt)
    } else {
        SHELL_MapSCIDToColumn(pSF, pscid)
    };
    if hr >= 0 {
        SHELL_GetDetailsOfColumnAsVariant(pSF, pidl, hr as u32, vt, pVar)
    } else {
        hr
    }
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_CreateAbsolutePidl(
    pSF: *mut IShellFolder,
    pidlChild: PCUIDLIST_RELATIVE,
    ppPidl: *mut PIDLIST_ABSOLUTE,
) -> HRESULT {
    let mut pidlFolder: PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut hr: HRESULT = SHGetIDListFromObject((pSF).cast(), &mut pidlFolder);
    if (hr as i32) >= 0 {
        hr = SHILCombine(pidlFolder, pidlChild, ppPidl);
        ILFree(pidlFolder);
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn Shell_NextElement(
    ppch: *mut PWSTR,
    pszOut: PWSTR,
    cchOut: i32,
    bValidate: BOOLEAN,
) -> HRESULT {
    *pszOut = UNICODE_NULL;

    if (*ppch).is_null() {
        return S_FALSE;
    }

    let mut hr: HRESULT;
    let mut pchNext = wcschr(*ppch, 0x005Cu16);

    if !pchNext.is_null() {
        if (*ppch as usize) < (pchNext as usize) {
            let count = pchNext.offset_from(*ppch) as i32;
            StringCchCopyNW(pszOut, (cchOut) as usize, *ppch, (count) as usize);
            pchNext = pchNext.add(1);

            if *pchNext == 0 {
                pchNext = core::ptr::null_mut();
            }
            hr = S_OK;
        } else {
            pchNext = core::ptr::null_mut();
            hr = E_INVALIDARG;
        }
    } else {
        StringCchCopyW(pszOut, (cchOut) as usize, *ppch);
        hr = S_OK;
    }

    *ppch = pchNext;

    if hr == S_OK && bValidate != 0 && PathIsValidElement(pszOut) == 0 {
        *pszOut = UNICODE_NULL;
        hr = E_INVALIDARG;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_ParseNextElement(
    psf: *mut IShellFolder2,
    hwndOwner: HWND,
    pbc: *mut IBindCtx,
    pidlInOut: *mut LPITEMIDLIST,
    szNext: LPOLESTR,
    pEaten: *mut DWORD,
    pdwAttributes: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;
    let pidlIn: LPITEMIDLIST = if !pidlInOut.is_null() {
        *pidlInOut
    } else {
        core::ptr::null_mut()
    };
    let mut pidlOut: LPITEMIDLIST = core::ptr::null_mut();
    let mut pidlTemp: LPITEMIDLIST = core::ptr::null_mut();
    let mut psfChild: *mut IShellFolder = core::ptr::null_mut();

    hr = ((*(*(psf as *mut IShellFolder2)).lpVtbl).BindToObject)((psf as *mut IShellFolder2),
        pidlIn.cast(),
        pbc.cast(),
        core::ptr::addr_of!(IID_IShellFolder).cast(),
        &mut psfChild as *mut *mut IShellFolder as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        return hr;
    }

    hr = ((*(*psfChild).lpVtbl).ParseDisplayName)(psfChild, hwndOwner.cast(), pbc.cast(), szNext, pEaten, core::ptr::addr_of_mut!(pidlOut).cast(), pdwAttributes);
    if hr < 0 {
        if !psfChild.is_null() {
            ((*(*psfChild).lpVtbl).Release)(psfChild);
        }
        return hr;
    }

    pidlTemp = ILCombine(pidlIn, pidlOut);
    if pidlTemp.is_null() {
        hr = E_OUTOFMEMORY;
        if !pidlOut.is_null() {
            ILFree(pidlOut);
        }
        if !psfChild.is_null() {
            ((*(*psfChild).lpVtbl).Release)(psfChild);
        }
        return hr;
    }

    if !pidlOut.is_null() {
        ILFree(pidlOut);
    }

    if !pidlIn.is_null() {
        ILFree(pidlIn);
    }

    *pidlInOut = pidlTemp;

    if !psfChild.is_null() {
        ((*(*psfChild).lpVtbl).Release)(psfChild);
    }

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_CoCreateInitSF(
    pidlRoot: LPCITEMIDLIST,
    ppfti: *mut PERSIST_FOLDER_TARGET_INFO,
    pidlChild: LPCITEMIDLIST,
    clsid: *const GUID,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pShellFolder: *mut IShellFolder = core::ptr::null_mut();

    hr = SHCoCreateInstance(
        core::ptr::null_mut(),
        (clsid).cast(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IShellFolder).cast(),
        &mut pShellFolder as *mut *mut IShellFolder as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        return hr;
    }

    let pidlAbsolute = ILCombine(pidlRoot, pidlChild);
    let mut ppf: *mut IPersistFolder = core::ptr::null_mut();
    let mut ppf3: *mut IPersistFolder3 = core::ptr::null_mut();

    if !ppfti.is_null()
        && ((*(*pShellFolder).lpVtbl).QueryInterface)(pShellFolder,
            core::ptr::addr_of!(IID_IPersistFolder3).cast(),
            &mut ppf3 as *mut *mut IPersistFolder3 as *mut *mut core::ffi::c_void,
        ) >= 0
    {
        ((*(*ppf3).lpVtbl).InitializeEx)(ppf3, core::ptr::null_mut(), pidlAbsolute.cast(), ppfti.cast());
        ((*(*ppf3).lpVtbl).Release)(ppf3);
    } else if ((*(*pShellFolder).lpVtbl).QueryInterface)(pShellFolder,
        core::ptr::addr_of!(IID_IPersistFolder).cast(),
        &mut ppf as *mut *mut IPersistFolder as *mut *mut core::ffi::c_void,
    ) >= 0
    {
        ((*(*ppf).lpVtbl).Initialize)(ppf, pidlAbsolute as *const _);
        ((*(*ppf).lpVtbl).Release)(ppf);
    }
    ILFree(pidlAbsolute);

    let result = ((*(*pShellFolder).lpVtbl).QueryInterface)(pShellFolder, (riid).cast(), ppvOut);
    ((*(*pShellFolder).lpVtbl).Release)(pShellFolder);
    result
}

/// C++ declared this as a second OVERLOAD of `SHELL32_CoCreateInitSF`. `EXTERN_C` maps one name
/// to one symbol and so cannot carry an overload set, so shfldr.h renames this form to
/// `SHELL32_CoCreateInitSFWithCsidl` and its six C++ call sites follow.
#[no_mangle]
pub unsafe extern "C" fn SHELL32_CoCreateInitSFWithCsidl(
    pidlRoot: LPCITEMIDLIST,
    clsid: *const GUID,
    csidl: i32,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    let mut pfti: PERSIST_FOLDER_TARGET_INFO = core::mem::zeroed();
    pfti.pidlTargetFolder = core::ptr::null_mut();
    pfti.dwAttributes = !0; // C: -1 on a DWORD
    pfti.csidl = csidl;
    SHELL32_CoCreateInitSF(pidlRoot, &mut pfti, core::ptr::null_mut(), clsid, riid, ppvOut)
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_BindToSF(
    pidlRoot: LPCITEMIDLIST,
    ppfti: *mut PERSIST_FOLDER_TARGET_INFO,
    pidl: LPCITEMIDLIST,
    clsid: *const GUID,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let pidlChild: PITEMID_CHILD = ILCloneFirst(pidl);
    if pidlChild.is_null() {
        return E_FAIL;
    }

    let mut psf: *mut IShellFolder = core::ptr::null_mut();
    let hr: HRESULT = SHELL32_CoCreateInitSF(
        pidlRoot,
        ppfti,
        pidlChild,
        clsid.cast(),
        core::ptr::addr_of!(IID_IShellFolder).cast(),
        &mut psf as *mut *mut IShellFolder as *mut *mut core::ffi::c_void,
    );
    ILFree(pidlChild);

    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    if _ILIsPidlSimple(pidl) != 0 {
        let result = ((*(*psf).lpVtbl).QueryInterface)(psf, (riid).cast(), ppvOut);
        ((*(*psf).lpVtbl).Release)(psf);
        result
    } else {
        let result = ((*(*psf).lpVtbl).BindToObject)(psf, ILGetNext(pidl).cast(), core::ptr::null_mut(), riid.cast(), ppvOut);
        ((*(*psf).lpVtbl).Release)(psf);
        result
    }
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_GetDisplayNameOfChild(
    psf: *mut IShellFolder2,
    pidl: LPCITEMIDLIST,
    dwFlags: DWORD,
    strRet: *mut STRRET,
) -> HRESULT {
    let pidlFirst: LPITEMIDLIST = ILCloneFirst(pidl);
    if pidlFirst.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut psfChild: *mut IShellFolder = core::ptr::null_mut();
    let mut hr: HRESULT = ((*(*psf).lpVtbl).BindToObject)(psf,
        pidlFirst.cast(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IShellFolder).cast(),
        &mut psfChild as *mut *mut IShellFolder as *mut *mut c_void,
    );
    if hr >= 0 {
        hr = ((*(*psfChild).lpVtbl).GetDisplayNameOf)(psfChild, ILGetNext(pidl).cast(), dwFlags, strRet.cast());
    }
    if !psfChild.is_null() {
        ((*(*psfChild).lpVtbl).Release)(psfChild);
    }
    ILFree(pidlFirst);

    hr
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_CompareChildren(psf: *mut IShellFolder2, lParam: LPARAM, pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT {
    let nextpidl1 = ILGetNext(pidl1);
    let nextpidl2 = ILGetNext(pidl2);

    let isEmpty1 = _ILIsDesktop(nextpidl1);
    let isEmpty2 = _ILIsDesktop(nextpidl2);
    if isEmpty1 != 0 || isEmpty2 != 0 {
        return MAKE_COMPARE_HRESULT!((isEmpty2 as i32) - (isEmpty1 as i32));
    }

    let firstpidl = ILCloneFirst(pidl1);
    if firstpidl.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut psf2: *mut IShellFolder = core::ptr::null_mut();
    let hr = ((*(*psf).lpVtbl).BindToObject)(psf, firstpidl.cast(), core::ptr::null_mut(), core::ptr::addr_of!(IID_IShellFolder).cast(), core::ptr::addr_of_mut!(psf2).cast());
    ILFree(firstpidl);
    if hr < 0 {
        return MAKE_COMPARE_HRESULT!(0);
    }

    let result = ((*(*psf2).lpVtbl).CompareIDs)(psf2, ((lParam) as isize) as _, nextpidl1.cast(), nextpidl2.cast());
    ((*(*psf2).lpVtbl).Release)(psf2);
    result
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_CompareDetails(
    isf: *mut IShellFolder2,
    lParam: LPARAM,
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
) -> HRESULT {
    let mut sd: SHELLDETAILS = core::mem::zeroed();
    let mut wszItem1: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    let mut wszItem2: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    let col: UINT = (LOWORD((lParam) as u32)) as u32;

    let hres = ((*(*isf).lpVtbl).GetDetailsOf)(isf, pidl1 as _, col, core::ptr::addr_of_mut!(sd).cast());
    if hres < 0 {
        return MAKE_COMPARE_HRESULT!(1);
    }

    let hres = StrRetToBufW(core::ptr::addr_of_mut!(sd.str_), pidl1, wszItem1.as_mut_ptr(), (MAX_PATH) as u32);
    if hres < 0 {
        return MAKE_COMPARE_HRESULT!(1);
    }

    let hres = ((*(*isf).lpVtbl).GetDetailsOf)(isf, pidl2 as _, col, core::ptr::addr_of_mut!(sd).cast());
    if hres < 0 {
        return MAKE_COMPARE_HRESULT!(1);
    }

    let hres = StrRetToBufW(core::ptr::addr_of_mut!(sd.str_), pidl2, wszItem2.as_mut_ptr(), (MAX_PATH) as u32);
    if hres < 0 {
        return MAKE_COMPARE_HRESULT!(1);
    }

    let ret = CFSFolder_CompareUiStrings_RustBridge(wszItem1.as_ptr(), wszItem2.as_ptr(), lParam);
    if ret == 0 {
        return SHELL32_CompareChildren(isf, lParam, pidl1, pidl2);
    }

    MAKE_COMPARE_HRESULT!(ret)
}
#[no_mangle]
pub unsafe extern "C" fn CloseRegKeyArray(array: *mut HKEY, cKeys: u32) {
    let mut i: u32 = 0;
    while i < cKeys {
        RegCloseKey((*array.add(i as usize)));
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn AddClassKeyToArray(szClass: *const u16, array: *mut HKEY, cKeys: *mut UINT) -> LSTATUS {
    if *cKeys >= 16 {
        return (ERROR_MORE_DATA) as i32;
    }

    let mut hkey: HKEY = core::mem::zeroed();
    let result = RegOpenKeyExW(HKEY_CLASSES_ROOT, szClass, 0, KEY_READ | KEY_QUERY_VALUE, &mut hkey);
    if result == (ERROR_SUCCESS) as i32 {
        *(array.add(*cKeys as usize)) = hkey;
        *cKeys += 1;
    }
    result
}
#[no_mangle]
pub unsafe extern "C" fn AddClsidKeyToArray(clsid: *const CLSID, array: *mut HKEY, cKeys: *mut UINT) -> LSTATUS {
    let path_len: usize = 6 + 38 + 1;
    let mut path: [u16; 6 + 38 + 1] = [0u16; 6 + 38 + 1];
    let src = CPP_COM_WSTR_6E342AB1A0F1.as_ptr() as *const u16;
    let mut i = 0;
    while i < path_len && *src.offset(i as isize) != 0u16 {
        path[i] = *src.offset(i as isize);
        i += 1;
    }
    if i < path_len {
        path[i] = 0;
    } else {
        path[path_len - 1] = 0;
    }
    StringFromGUID2(clsid, path.as_mut_ptr().offset(6 as isize), (38 + 1) as i32);
    AddClassKeyToArray(path.as_mut_ptr(), array, cKeys)
}
#[no_mangle]
pub unsafe extern "C" fn AddFSClassKeysToArray(
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    array: *mut HKEY,
    cKeys: *mut UINT,
) {
    let pidl = *apidl.offset(0 as isize);

    if _ILIsValue(pidl) != 0 {
        let mut buf = [0u16; (MAX_PATH) as usize];
        let name: *mut u16;
        let pFileData = _ILGetFileStructW(pidl);
        if !pFileData.is_null() {
            name = core::ptr::addr_of_mut!((*pFileData).wszName) as *mut u16;
        } else {
            _ILSimpleGetTextW(pidl, buf.as_mut_ptr(), _countof!(&buf));
            name = buf.as_mut_ptr();
        }
        let extension = PathFindExtensionW(name);
        if !extension.is_null() {
            let mut wszClass = [0u16; (MAX_PATH) as usize];
            let mut dwSize = core::mem::size_of_val(&wszClass) as u32;
            if RegGetValueW(
                HKEY_CLASSES_ROOT,
                extension,
                core::ptr::null_mut(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                (wszClass.as_mut_ptr()).cast(),
                &mut dwSize,
            ) != (ERROR_SUCCESS) as i32
                || *wszClass.as_ptr() == 0u16
                || AddClassKeyToArray(wszClass.as_mut_ptr(), array, cKeys) != (ERROR_SUCCESS) as i32
            {
                AddClassKeyToArray(extension, array, cKeys);
                if cidl == 1 {
                    AddClassKeyToArray(CPP_COM_WSTR_F3CDEC9800CB.as_ptr(), array, cKeys);
                }
            }

            // C++: WCHAR wszSFA[23 + _countof(wszClass)] — a fixed STACK array, not a heap vector.
            let mut wszSFA_buf: [u16; 23 + MAX_PATH as usize] = [0u16; 23 + MAX_PATH as usize];
            let wszSFA = wszSFA_buf.as_mut_ptr();
            swprintf(wszSFA, CPP_COM_WSTR_F38FD69158A5.as_ptr(), extension);
            AddClassKeyToArray(wszSFA, array, cKeys);

            dwSize = core::mem::size_of_val(&wszClass) as u32;
            if RegGetValueW(
                HKEY_CLASSES_ROOT,
                extension,
                CPP_COM_WSTR_714C3CA85096.as_ptr(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                (wszClass.as_mut_ptr()).cast(),
                &mut dwSize,
            ) == (ERROR_SUCCESS) as i32
            {
                swprintf(wszSFA, CPP_COM_WSTR_F38FD69158A5.as_ptr(), wszClass.as_mut_ptr());
                AddClassKeyToArray(wszSFA, array, cKeys);
            }
        }

        AddClassKeyToArray(CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr(), array, cKeys);
        AddClassKeyToArray(CPP_COM_WSTR_A8DA479F8D2D.as_ptr(), array, cKeys);
    } else if _ILIsFolder(pidl) != 0 {
        AddClassKeyToArray(CPP_COM_WSTR_EB4CE2024A43.as_ptr(), array, cKeys);
        AddClassKeyToArray(CPP_COM_WSTR_A8DA479F8D2D.as_ptr(), array, cKeys);
        AddClassKeyToArray(CPP_COM_WSTR_3A7BCE312332.as_ptr(), array, cKeys);
    } else if _ILIsDrive(pidl) != 0 {
        AddClassKeyToArray(CPP_COM_WSTR_32491EF24F87.as_ptr(), array, cKeys);
        AddClassKeyToArray(CPP_COM_WSTR_EB4CE2024A43.as_ptr(), array, cKeys);
    } else {
        // whitespace
    }
}
#[no_mangle]
pub unsafe extern "C" fn AddPidlClassKeysToArray(mut pidl: LPCITEMIDLIST, array: *mut HKEY, cKeys: *mut UINT) {
    pidl = ILFindLastID(pidl);
    if !pidl.is_null() {
        AddFSClassKeysToArray(1, core::ptr::addr_of_mut!(pidl), array, cKeys);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SH_GetApidlFromDataObject(
    pDataObject: *mut IDataObject,
    ppidlfolder: *mut PIDLIST_ABSOLUTE,
    apidlItems: *mut *mut PUITEMID_CHILD,
    pcidl: *mut UINT,
) -> HRESULT {
    let mut cida = CDataObjectHIDA::new(pDataObject);

    if FAILED_UNEXPECTEDLY!(cida.hr()) {
        return cida.hr();
    }

    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let apidl = _ILCopyCidaToaPidl(
        &mut pidl,
        cida.as_ptr(),
    );
    if apidl.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppidlfolder = pidl;
    *apidlItems = apidl;
    *pcidl = (*cida.as_ptr()).cidl;

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateLinks(
    hWnd: HWND,
    lpszDir: LPCSTR,
    lpDataObject: *mut IDataObject,
    uFlags: UINT,
    lppidlLinks: *mut LPITEMIDLIST,
) -> HRESULT {
    // FIXME("%p %s %p %08x %p\n", hWnd, lpszDir, lpDataObject, uFlags, lppidlLinks);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenFolderAndSelectItems(
    pidlFolder: PCIDLIST_ABSOLUTE,
    cidl: u32,
    apidl: PCUITEMID_CHILD_ARRAY,
    dwFlags: DWORD,
) -> HRESULT {
    // Lower CComHeapPtr to a raw pointer with manual cleanup; the deterministic layer provides the type and its Drop.
    let mut freeItem: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::<ITEMIDLIST>::new();
    let pidlItem: PCIDLIST_ABSOLUTE;
    if cidl != 0 {
        if ILIsSingle((apidl.add(0)).cast()) == 0 {
            pidlItem = (apidl.add(0)).cast();
        } else {
            let mut hr: HRESULT;
            let mut tmp: PCIDLIST_ABSOLUTE = core::ptr::null_mut();
            hr = SHILCombine(pidlFolder, *apidl.add(0), core::ptr::addr_of_mut!(tmp).cast());
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }
            freeItem.Attach((tmp as *mut PIDLIST_ABSOLUTE).cast());
            pidlItem = tmp;
        }
    } else {
        pidlItem = pidlFolder;
    }

    let mut psfDesktop: CComPtr<IShellFolder> = CComPtr::<IShellFolder>::new();
    let hr: HRESULT = SHGetDesktopFolder((core::ptr::addr_of_mut!(psfDesktop)).cast());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut strret: STRRET = core::mem::zeroed();
    // IShellFolder::GetDisplayNameOf takes a REFIID and a pointer to STRRET.
    let hr = ((*(*psfDesktop.as_ptr()).lpVtbl).GetDisplayNameOf)(psfDesktop.as_ptr(), pidlItem.cast(), SHGDN_FORPARSING, core::ptr::addr_of_mut!(strret).cast());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut wszBuf: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    let hr = StrRetToBufW(core::ptr::addr_of_mut!(strret), pidlItem, wszBuf.as_mut_ptr(), wszBuf.len() as u32);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut wszParams: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    wcscpy(wszParams.as_mut_ptr(), CPP_COM_WSTR_84B1405B4E34.as_ptr());
    wcscat(wszParams.as_mut_ptr(), wszBuf.as_ptr());

    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
    memset((core::ptr::addr_of_mut!(sei)).cast(), 0, (core::mem::size_of::<SHELLEXECUTEINFOW>() as u32) as usize);
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_WAITFORINPUTIDLE;
    sei.lpFile = CPP_COM_WSTR_AB68B18B313D.as_ptr() as *mut u16;
    sei.lpParameters = wszParams.as_mut_ptr();

    if ShellExecuteExW(core::ptr::addr_of_mut!(sei)) != 0 {
        return S_OK;
    } else {
        return E_FAIL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_ShowPropertiesDialog(pdtobj: *mut IDataObject) -> HRESULT {
    if pdtobj.is_null() {
        return E_INVALIDARG;
    }
    SHELL32_ShowFilesystemItemsPropertiesDialogAsync(core::ptr::null_mut(), pdtobj)
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_DefaultContextMenuCallBack(
    psf: *mut IShellFolder,
    pdo: *mut IDataObject,
    msg: u32,
) -> i32 {
    match msg {
        DFM_MERGECONTEXTMENU => S_OK,
        DFM_INVOKECOMMAND => S_FALSE,
        DFM_GETDEFSTATICID => S_FALSE,
        _ => E_NOTIMPL,
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_BindToSF_RustBridge(
    pidlRoot: LPCITEMIDLIST,
    ppfti: *mut PERSIST_FOLDER_TARGET_INFO,
    pidl: LPCITEMIDLIST,
    clsid: *const GUID,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    SHELL32_BindToSF(pidlRoot, ppfti, pidl, clsid, riid, ppvOut)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_CompareChildren_RustBridge(
    psf: *mut IShellFolder2,
    lParam: LPARAM,
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
) -> HRESULT {
    SHELL32_CompareChildren(psf, lParam, pidl1, pidl2)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_GetDisplayNameOfChild_RustBridge(
    psf: *mut IShellFolder2,
    pidl: LPCITEMIDLIST,
    dwFlags: DWORD,
    strRet: LPSTRRET,
) -> HRESULT {
    SHELL32_GetDisplayNameOfChild(psf, pidl, dwFlags, strRet)
}

#[no_mangle]
pub unsafe extern "system" fn SH32_GetDetailsOfPKeyAsVariant_RustBridge(
    pSF: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    pscid: *const SHCOLUMNID,
    pVar: *mut VARIANT,
    UseFsColMap: BOOL,
) -> HRESULT {
    SH32_GetDetailsOfPKeyAsVariant(pSF, pidl, pscid, pVar, UseFsColMap)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_DefaultContextMenuCallBack_RustBridge(
    psf: *mut IShellFolder,
    pdo: *mut IDataObject,
    uMsg: UINT,
) -> HRESULT {
    SHELL32_DefaultContextMenuCallBack(psf, pdo, uMsg)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_CreateSHDESCRIPTIONID_RustBridge(
    pVar: *mut VARIANT,
    Id: DWORD,
    pCLSID: *const CLSID,
) -> HRESULT {
    SHELL_CreateSHDESCRIPTIONID(pVar, Id, pCLSID)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_GetDetailsOfAsStringVariant_RustBridge(
    pSF: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    Column: UINT,
    pVar: *mut VARIANT,
) -> HRESULT {
    SHELL_GetDetailsOfAsStringVariant(pSF, pidl, Column, pVar)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_CreateVariantBuffer_RustBridge(
    pVar: *mut VARIANT,
    cb: UINT,
) -> HRESULT {
    SHELL_CreateVariantBuffer(pVar, cb)
}

#[no_mangle]
pub unsafe extern "system" fn Shell_NextElement_RustBridge(
    ppch: *mut LPWSTR,
    pszOut: LPWSTR,
    cchOut: INT,
    bValidate: BOOL,
) -> HRESULT {
    Shell_NextElement(ppch, pszOut, cchOut, (bValidate) as u8)
}

#[no_mangle]
pub unsafe extern "system" fn AddFSClassKeysToArray_RustBridge(
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    array: *mut HKEY,
    cKeys: *mut UINT,
) {
    AddFSClassKeysToArray(cidl, apidl, array, cKeys);
}

#[no_mangle]
pub unsafe extern "system" fn AddClassKeyToArray_RustBridge(
    szClass: *const WCHAR,
    array: *mut HKEY,
    cKeys: *mut UINT,
) -> LSTATUS {
    AddClassKeyToArray(szClass, array, cKeys)
}

#[no_mangle]
pub unsafe extern "system" fn CRecycleBin_CoCreateInitSF_RustBridge(
    pidlRoot: LPCITEMIDLIST,
    ppfti: *mut PERSIST_FOLDER_TARGET_INFO,
    pidlChild: LPCITEMIDLIST,
    clsid: *const GUID,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    SHELL32_CoCreateInitSF(pidlRoot, ppfti, pidlChild, clsid, riid, ppvOut)
}

#[no_mangle]
pub unsafe extern "system" fn AddClsidKeyToArray_RustBridge(
    clsid: REFCLSID,
    array: *mut HKEY,
    cKeys: *mut UINT,
) -> LSTATUS {
    AddClsidKeyToArray(clsid.cast(), array, cKeys)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_ShowPropertiesDialog_RustBridge(
    pdo: *mut IDataObject,
) -> HRESULT {
    SHELL32_ShowPropertiesDialog(pdo)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *    Shell Folder stuff
//  *
//  *    Copyright 1997            Marcus Meissner
//  *    Copyright 1998, 1999, 2002    Juergen Schmied
//  *    Copyright 2018 Katayama Hirofumi MZ
//  *
//  *    IShellFolder2 and related interfaces
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
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// SHCONTF SHELL_GetDefaultFolderEnumSHCONTF()
// {
//     SHCONTF Flags = SHCONTF_FOLDERS | SHCONTF_NONFOLDERS;
//     SHELLSTATE ss;
//     SHGetSetSettings(&ss, SSF_SHOWALLOBJECTS | SSF_SHOWSUPERHIDDEN, FALSE);
//     if (ss.fShowAllObjects)
//         Flags |= SHCONTF_INCLUDEHIDDEN;
//     if (ss.fShowSuperHidden)
//         Flags |= SHCONTF_INCLUDESUPERHIDDEN;
//      return Flags;
// }
//
// BOOL SHELL_IncludeItemInFolderEnum(IShellFolder *pSF, PCUITEMID_CHILD pidl, SFGAOF Query, SHCONTF Flags)
// {
//     if (SUCCEEDED(pSF->GetAttributesOf(1, &pidl, &Query)))
//     {
//         if (Query & SFGAO_NONENUMERATED)
//             return FALSE;
//         if ((Query & SFGAO_HIDDEN) && !(Flags & SHCONTF_INCLUDEHIDDEN))
//             return FALSE;
//         if ((Query & (SFGAO_HIDDEN | SFGAO_SYSTEM)) == (SFGAO_HIDDEN | SFGAO_SYSTEM) && !(Flags & SHCONTF_INCLUDESUPERHIDDEN))
//             return FALSE;
//         if ((Flags & (SHCONTF_FOLDERS | SHCONTF_NONFOLDERS)) != (SHCONTF_FOLDERS | SHCONTF_NONFOLDERS))
//             return (Flags & SHCONTF_FOLDERS) ? (Query & SFGAO_FOLDER) : !(Query & SFGAO_FOLDER);
//     }
//     return TRUE;
// }
//
// static HRESULT
// MapSCIDToShell32FsColumn(const SHCOLUMNID *pscid, VARTYPE &vt)
// {
//     if (pscid->fmtid == FMTID_Storage)
//     {
//         switch (pscid->pid)
//         {
//             case PID_STG_NAME:        vt = VT_BSTR; return SHFSF_COL_NAME;
//             case PID_STG_SIZE:        vt = VT_UI8; return SHFSF_COL_SIZE;
//             case PID_STG_STORAGETYPE: vt = VT_BSTR; return SHFSF_COL_TYPE;
//             case PID_STG_ATTRIBUTES:  vt = VT_BSTR; return SHFSF_COL_FATTS;
//             case PID_STG_WRITETIME:   vt = VT_DATE; return SHFSF_COL_MDATE;
//         }
//     }
//     if (pscid->fmtid == FMTID_SummaryInformation && pscid->pid == PIDSI_COMMENTS)
//     {
//         vt = VT_BSTR;
//         return SHFSF_COL_COMMENT;
//     }
//     return E_INVALIDARG;
// }
//
// HRESULT
// SHELL_MapSCIDToColumn(IShellFolder2 *pSF, const SHCOLUMNID *pscid)
// {
//     for (UINT i = 0; i <= SHCIDS_COLUMNMASK; ++i)
//     {
//         SHCOLUMNID scid;
//         HRESULT hr = pSF->MapColumnToSCID(i, &scid);
//         if (FAILED(hr))
//             break;
//         if (IsEqualPropertyKey(scid, *pscid))
//             return i;
//     }
//     return E_FAIL;
// }
//
// HRESULT
// SHELL_GetDetailsOfAsStringVariant(IShellFolder2 *pSF, PCUITEMID_CHILD pidl, UINT Column, VARIANT *pVar)
// {
//     V_VT!(pVar) = (VT_EMPTY) as VARTYPE;
//     SHELLDETAILS sd;
//     sd.str_.uType = STRRET_CSTR;
//     HRESULT hr = pSF->GetDetailsOf(pidl, Column, &sd);
//     if (FAILED(hr))
//         return hr;
//     if (FAILED(hr = StrRetToBSTR(&sd.str_, pidl, &V_BSTR!(pVar))))
//         return hr;
//     V_VT!(pVar) = (VT_BSTR) as VARTYPE;
//     return hr;
// }
//
// HRESULT
// SHELL_GetDetailsOfColumnAsVariant(IShellFolder2 *pSF, PCUITEMID_CHILD pidl, UINT Column, VARTYPE vt, VARIANT *pVar)
// {
//     HRESULT hr = SHELL_GetDetailsOfAsStringVariant(pSF, pidl, Column, pVar);
//     if (FAILED(hr))
//         return hr;
//     if (vt == VT_EMPTY)
//     {
//         SHCOLSTATEF state;
//         if (FAILED(pSF->GetDefaultColumnState(Column, &state)))
//             state = SHCOLSTATE_TYPE_STR;
//         if ((state & SHCOLSTATE_TYPEMASK) == SHCOLSTATE_TYPE_INT)
//             vt = VT_I8;
//         else if ((state & SHCOLSTATE_TYPEMASK) == SHCOLSTATE_TYPE_DATE)
//             vt = VT_DATE;
//         else
//             vt = VT_BSTR;
//     }
//     if (vt != VT_BSTR)
//         VariantChangeType(pVar, pVar, 0, vt);
//     return hr;
// }
//
// HRESULT
// SH32_GetDetailsOfPKeyAsVariant(IShellFolder2 *pSF, PCUITEMID_CHILD pidl, const SHCOLUMNID *pscid, VARIANT *pVar, BOOL UseFsColMap)
// {
//     // CFSFolder and CRegFolder uses the SHFSF_COL columns and can use the faster and better version,
//     // everyone else must ask the folder to map the SCID to a column.
//     VARTYPE vt = VT_EMPTY;
//     HRESULT hr = UseFsColMap ? MapSCIDToShell32FsColumn(pscid, vt) : SHELL_MapSCIDToColumn(pSF, pscid);
//     return SUCCEEDED(hr) ? SHELL_GetDetailsOfColumnAsVariant(pSF, pidl, hr, vt, pVar) : hr;
// }
//
// HRESULT SHELL_CreateAbsolutePidl(IShellFolder *pSF, PCUIDLIST_RELATIVE pidlChild, PIDLIST_ABSOLUTE *ppPidl)
// {
//     PIDLIST_ABSOLUTE pidlFolder;
//     HRESULT hr = SHGetIDListFromObject(pSF, &pidlFolder);
//     if (SUCCEEDED(hr))
//     {
//         hr = SHILCombine(pidlFolder, pidlChild, ppPidl);
//         ILFree(pidlFolder);
//     }
//     return hr;
// }
//
// HRESULT
// Shell_NextElement(
//      LPWSTR *ppch,
//      LPWSTR pszOut,
//      INT cchOut,
//      BOOL bValidate)
// {
//     *pszOut = UNICODE_NULL;
//
//     if (!*ppch)
//         return S_FALSE;
//
//     HRESULT hr;
//     LPWSTR pchNext = wcschr(*ppch, L'\\');
//     if (pchNext)
//     {
//         if (*ppch < pchNext)
//         {
//             /* Get an element */
//             StringCchCopyNW(pszOut, cchOut, *ppch, pchNext - *ppch);
//             ++pchNext;
//
//             if (!*pchNext)
//                 pchNext = NULL; /* No next */
//
//             hr = S_OK;
//         }
//         else /* Double backslashes found? */
//         {
//             pchNext = NULL;
//             hr = E_INVALIDARG;
//         }
//     }
//     else /* No more next */
//     {
//         StringCchCopyW(pszOut, cchOut, *ppch);
//         hr = S_OK;
//     }
//
//     *ppch = pchNext; /* Go next */
//
//     if (hr == S_OK && bValidate && !PathIsValidElement(pszOut))
//     {
//         *pszOut = UNICODE_NULL;
//         hr = E_INVALIDARG;
//     }
//
//     return hr;
// }
//
// HRESULT SHELL32_ParseNextElement (IShellFolder2 * psf, HWND hwndOwner, LPBC pbc,
//                   LPITEMIDLIST * pidlInOut, LPOLESTR szNext, DWORD * pEaten, DWORD * pdwAttributes)
// {
//     HRESULT hr = E_INVALIDARG;
//     LPITEMIDLIST pidlIn = pidlInOut ? *pidlInOut : NULL;
//     LPITEMIDLIST pidlOut = NULL;
//     LPITEMIDLIST pidlTemp = NULL;
//     CComPtr<IShellFolder> psfChild;
//
//     TRACE ("(%p, %p, %p, %s)\n", psf, pbc, pidlIn, debugstr_w (szNext));
//
//     /* get the shellfolder for the child pidl and let it analyse further */
//     hr = psf->BindToObject(pidlIn, pbc, IID_PPV_ARG(IShellFolder, &psfChild));
//     if (FAILED(hr))
//         return hr;
//
//     hr = psfChild->ParseDisplayName(hwndOwner, pbc, szNext, pEaten, &pidlOut, pdwAttributes);
//     if (FAILED(hr))
//         return hr;
//
//     pidlTemp = ILCombine (pidlIn, pidlOut);
//     if (!pidlTemp)
//     {
//         hr = E_OUTOFMEMORY;
//         if (pidlOut)
//             ILFree(pidlOut);
//         return hr;
//     }
//
//     if (pidlOut)
//         ILFree (pidlOut);
//
//     if (pidlIn)
//         ILFree (pidlIn);
//
//     *pidlInOut = pidlTemp;
//
//     TRACE ("-- pidl=%p ret=0x%08x\n", pidlInOut ? *pidlInOut : NULL, hr);
//     return S_OK;
// }
//
// /***********************************************************************
//  *    SHELL32_CoCreateInitSF
//  *
//  * Creates a shell folder and initializes it with a pidl and a root folder
//  * via IPersistFolder3 or IPersistFolder.
//  *
//  * NOTES
//  *   pathRoot can be NULL for Folders being a drive.
//  *   In this case the absolute path is built from pidlChild (eg. C:)
//  */
// HRESULT SHELL32_CoCreateInitSF (LPCITEMIDLIST pidlRoot, PERSIST_FOLDER_TARGET_INFO* ppfti,
//                 LPCITEMIDLIST pidlChild, const GUID* clsid, REFIID riid, LPVOID *ppvOut)
// {
//     HRESULT hr;
//     CComPtr<IShellFolder> pShellFolder;
//
//     hr = SHCoCreateInstance(NULL, clsid, NULL, IID_PPV_ARG(IShellFolder, &pShellFolder));
//     if (FAILED(hr))
//         return hr;
//
//     LPITEMIDLIST pidlAbsolute = ILCombine (pidlRoot, pidlChild);
//     CComPtr<IPersistFolder> ppf;
//     CComPtr<IPersistFolder3> ppf3;
//
//     if (ppfti && SUCCEEDED(pShellFolder->QueryInterface(IID_PPV_ARG(IPersistFolder3, &ppf3))))
//     {
//         ppf3->InitializeEx(NULL, pidlAbsolute, ppfti);
//     }
//     else if (SUCCEEDED(pShellFolder->QueryInterface(IID_PPV_ARG(IPersistFolder, &ppf))))
//     {
//         ppf->Initialize(pidlAbsolute);
//     }
//     ILFree (pidlAbsolute);
//
//     return pShellFolder->QueryInterface(riid, ppvOut);
// }
//
// HRESULT SHELL32_CoCreateInitSF (LPCITEMIDLIST pidlRoot, const GUID* clsid,
//                                 int csidl, REFIID riid, LPVOID *ppvOut)
// {
//     /* fill the PERSIST_FOLDER_TARGET_INFO */
//     PERSIST_FOLDER_TARGET_INFO pfti = {0};
//     pfti.dwAttributes = -1;
//     pfti.csidl = csidl;
//
//     return SHELL32_CoCreateInitSF(pidlRoot, &pfti, NULL, clsid, riid, ppvOut);
// }
//
// HRESULT SHELL32_BindToSF (LPCITEMIDLIST pidlRoot, PERSIST_FOLDER_TARGET_INFO* ppfti,
//                 LPCITEMIDLIST pidl, const GUID* clsid, REFIID riid, LPVOID *ppvOut)
// {
//     PITEMID_CHILD pidlChild = ILCloneFirst (pidl);
//     if (!pidlChild)
//         return E_FAIL;
//
//     CComPtr<IShellFolder> psf;
//     HRESULT hr = SHELL32_CoCreateInitSF(pidlRoot,
//                                         ppfti,
//                                         pidlChild,
//                                         clsid,
//                                         IID_PPV_ARG(IShellFolder, &psf));
//     ILFree(pidlChild);
//
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     if (_ILIsPidlSimple (pidl))
//         return psf->QueryInterface(riid, ppvOut);
//     else
//         return psf->BindToObject(ILGetNext (pidl), NULL, riid, ppvOut);
// }
//
// /***********************************************************************
//  *    SHELL32_GetDisplayNameOfChild
//  *
//  * Retrieves the display name of a child object of a shellfolder.
//  *
//  * For a pidl eg. [subpidl1][subpidl2][subpidl3]:
//  * - it binds to the child shellfolder [subpidl1]
//  * - asks it for the displayname of [subpidl2][subpidl3]
//  *
//  * Is possible the pidl is a simple pidl. In this case it asks the
//  * subfolder for the displayname of an empty pidl. The subfolder
//  * returns the own displayname eg. "::{guid}". This is used for
//  * virtual folders with the registry key WantsFORPARSING set.
//  */
// HRESULT SHELL32_GetDisplayNameOfChild (IShellFolder2 * psf,
//                        LPCITEMIDLIST pidl, DWORD dwFlags, LPSTRRET strRet)
// {
//     LPITEMIDLIST pidlFirst = ILCloneFirst(pidl);
//     if (!pidlFirst)
//         return E_OUTOFMEMORY;
//
//     CComPtr<IShellFolder> psfChild;
//     HRESULT hr = psf->BindToObject(pidlFirst, NULL, IID_PPV_ARG(IShellFolder, &psfChild));
//     if (SUCCEEDED (hr))
//     {
//         hr = psfChild->GetDisplayNameOf(ILGetNext (pidl), dwFlags, strRet);
//     }
//     ILFree (pidlFirst);
//
//     return hr;
// }
//
// HRESULT SHELL32_CompareChildren(IShellFolder2* psf, LPARAM lParam, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2)
// {
//     PUIDLIST_RELATIVE nextpidl1 = ILGetNext (pidl1);
//     PUIDLIST_RELATIVE nextpidl2 = ILGetNext (pidl2);
//
//     bool isEmpty1 = _ILIsDesktop(nextpidl1);
//     bool isEmpty2 = _ILIsDesktop(nextpidl2);
//     if (isEmpty1 || isEmpty2)
//         return MAKE_COMPARE_HRESULT(isEmpty2 - isEmpty1);
//
//     PITEMID_CHILD firstpidl = ILCloneFirst (pidl1);
//     if (!firstpidl)
//         return E_OUTOFMEMORY;
//
//     CComPtr<IShellFolder> psf2;
//     HRESULT hr = psf->BindToObject(firstpidl, 0, IID_PPV_ARG(IShellFolder, &psf2));
//     ILFree(firstpidl);
//     if (FAILED(hr))
//         return MAKE_COMPARE_HRESULT(0);
//
//     return psf2->CompareIDs(lParam, nextpidl1, nextpidl2);
// }
//
// extern "C" HRESULT WINAPI SHELL32_CompareDetails(IShellFolder2* isf, LPARAM lParam, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2)
// {
//     SHELLDETAILS sd;
//     WCHAR wszItem1[MAX_PATH], wszItem2[MAX_PATH];
//     HRESULT hres;
//     UINT col = LOWORD(lParam); // Column index without SHCIDS_* flags
//
//     hres = isf->GetDetailsOf(pidl1, col, &sd);
//     if (FAILED(hres))
//         return MAKE_COMPARE_HRESULT(1);
//
//     hres = StrRetToBufW(&sd.str_, pidl1, wszItem1, MAX_PATH);
//     if (FAILED(hres))
//         return MAKE_COMPARE_HRESULT(1);
//
//     hres = isf->GetDetailsOf(pidl2, col, &sd);
//     if (FAILED(hres))
//         return MAKE_COMPARE_HRESULT(1);
//
//     hres = StrRetToBufW(&sd.str_, pidl2, wszItem2, MAX_PATH);
//     if (FAILED(hres))
//         return MAKE_COMPARE_HRESULT(1);
//
//     int ret = CFSFolder::CFSFolder_CompareUiStrings_RustBridge(wszItem1, wszItem2, lParam);
//     if (ret == 0)
//         return SHELL32_CompareChildren(isf, lParam, pidl1, pidl2);
//
//     return MAKE_COMPARE_HRESULT(ret);
// }
//
// void CloseRegKeyArray(HKEY* array, UINT cKeys)
// {
//     for (UINT i = 0; i < cKeys; ++i)
//         RegCloseKey(array[i]);
// }
//
// LSTATUS AddClassKeyToArray(const WCHAR* szClass, HKEY* array, UINT* cKeys)
// {
//     if (*cKeys >= 16)
//         return ERROR_MORE_DATA;
//
//     HKEY hkey;
//     LSTATUS result = RegOpenKeyExW(HKEY_CLASSES_ROOT, szClass, 0, KEY_READ | KEY_QUERY_VALUE, &hkey);
//     if (result == ERROR_SUCCESS)
//     {
//         array[*cKeys] = hkey;
//         *cKeys += 1;
//     }
//     return result;
// }
//
// LSTATUS AddClsidKeyToArray(REFCLSID clsid, HKEY* array, UINT* cKeys)
// {
//     WCHAR path[6 + 38 + 1] = L"CLSID\\";
//     StringFromGUID2(clsid, path + 6, 38 + 1);
//     return AddClassKeyToArray(path, array, cKeys);
// }
//
// void AddFSClassKeysToArray(UINT cidl, PCUITEMID_CHILD_ARRAY apidl, HKEY* array, UINT* cKeys)
// {
//     // This function opens the association array keys in canonical order for filesystem items.
//     // The order is documented: learn.microsoft.com/en-us/windows/win32/shell/fa-associationarray
//
//     ASSERT(cidl >= 1 && apidl);
//     PCUITEMID_CHILD pidl = apidl[0];
//     if (_ILIsValue(pidl))
//     {
//         WCHAR buf[MAX_PATH];
//         PWSTR name;
//         FileStructW* pFileData = _ILGetFileStructW(pidl);
//         if (pFileData)
//         {
//             name = pFileData->wszName;
//         }
//         else
//         {
//             _ILSimpleGetTextW(pidl, buf, _countof(buf));
//             name = buf;
//         }
//         LPCWSTR extension = PathFindExtensionW(name);
//
//         if (extension)
//         {
//             WCHAR wszClass[MAX_PATH], wszSFA[23 + _countof(wszClass)];
//             DWORD dwSize = sizeof(wszClass);
//             if (RegGetValueW(HKEY_CLASSES_ROOT, extension, NULL, RRF_RT_REG_SZ, NULL, wszClass, &dwSize) != ERROR_SUCCESS ||
//                 !*wszClass || AddClassKeyToArray(wszClass, array, cKeys) != ERROR_SUCCESS)
//             {
//                 // Only add the extension key if the ProgId is not valid
//                 AddClassKeyToArray(extension, array, cKeys);
//
//                 // "Open With" becomes the default when there are no verbs in the above keys
//                 if (cidl == 1)
//                     AddClassKeyToArray(L"Unknown", array, cKeys);
//             }
//
//             swprintf(wszSFA, L"SystemFileAssociations\\%s", extension);
//             AddClassKeyToArray(wszSFA, array, cKeys);
//
//             dwSize = sizeof(wszClass);
//             if (RegGetValueW(HKEY_CLASSES_ROOT, extension, L"PerceivedType", RRF_RT_REG_SZ, NULL, wszClass, &dwSize) == ERROR_SUCCESS)
//             {
//                 swprintf(wszSFA, L"SystemFileAssociations\\%s", wszClass);
//                 AddClassKeyToArray(wszSFA, array, cKeys);
//             }
//         }
//
//         AddClassKeyToArray(L"*", array, cKeys);
//         AddClassKeyToArray(L"AllFilesystemObjects", array, cKeys);
//     }
//     else if (_ILIsFolder(pidl))
//     {
//         // FIXME: Directory > Folder > AFO is the correct order and it's
//         // the order Windows reports in its undocumented association array
//         // but it is somehow not the order Windows adds the items to its menu!
//         // Until the correct algorithm in CDefaultContextMenu can be determined,
//         // we add the folder keys in "menu order".
//         AddClassKeyToArray(L"Folder", array, cKeys);
//         AddClassKeyToArray(L"AllFilesystemObjects", array, cKeys);
//         AddClassKeyToArray(L"Directory", array, cKeys);
//     }
//     else if (_ILIsDrive(pidl))
//     {
//         AddClassKeyToArray(L"Drive", array, cKeys);
//         AddClassKeyToArray(L"Folder", array, cKeys);
//     }
//     else
//     {
//         ERR("Got non FS pidl\n");
//     }
// }
//
// void AddPidlClassKeysToArray(LPCITEMIDLIST pidl, HKEY* array, UINT* cKeys)
// {
//     if ((pidl = ILFindLastID(pidl)) != NULL)
//         AddFSClassKeysToArray(1, &pidl, array, cKeys);
// }
//
// HRESULT SH_GetApidlFromDataObject(IDataObject *pDataObject, PIDLIST_ABSOLUTE* ppidlfolder, PUITEMID_CHILD **apidlItems, UINT *pcidl)
// {
//     CDataObjectHIDA cida(pDataObject);
//
//     if (FAILED_UNEXPECTEDLY(cida.hr()))
//         return cida.hr();
//
//     /* convert the data into pidl */
//     LPITEMIDLIST pidl;
//     LPITEMIDLIST *apidl = _ILCopyCidaToaPidl(&pidl, cida);
//     if (!apidl)
//     {
//         return E_OUTOFMEMORY;
//     }
//
//     *ppidlfolder = pidl;
//     *apidlItems = apidl;
//     *pcidl = cida->cidl;
//
//     return S_OK;
// }
//
// /***********************************************************************
//  *  SHCreateLinks
//  *
//  *   Undocumented.
//  */
// HRESULT WINAPI SHCreateLinks( HWND hWnd, LPCSTR lpszDir, IDataObject * lpDataObject,
//                               UINT uFlags, LPITEMIDLIST *lppidlLinks)
// {
//     FIXME("%p %s %p %08x %p\n", hWnd, lpszDir, lpDataObject, uFlags, lppidlLinks);
//     return E_NOTIMPL;
// }
//
// /***********************************************************************
//  *  SHOpenFolderAndSelectItems
//  *
//  *   Unimplemented.
//  */
// extern "C" HRESULT
// WINAPI
// SHOpenFolderAndSelectItems(PCIDLIST_ABSOLUTE pidlFolder,
//                            UINT cidl,
//                            PCUITEMID_CHILD_ARRAY apidl,
//                            DWORD dwFlags)
// {
//     ERR("SHOpenFolderAndSelectItems() is hackplemented\n");
//     CComHeapPtr<ITEMIDLIST> freeItem;
//     PCIDLIST_ABSOLUTE pidlItem;
//     if (cidl)
//     {
//         /* Firefox sends a full pidl here dispite the fact it is a PCUITEMID_CHILD_ARRAY -_- */
//         if (!ILIsSingle(apidl[0]))
//         {
//             pidlItem = apidl[0];
//         }
//         else
//         {
//             HRESULT hr = SHILCombine(pidlFolder, apidl[0], &pidlItem);
//             if (FAILED_UNEXPECTEDLY(hr))
//                 return hr;
//             freeItem.Attach(const_cast<PIDLIST_ABSOLUTE>(pidlItem));
//         }
//     }
//     else
//     {
//         pidlItem = pidlFolder;
//     }
//
//     CComPtr<IShellFolder> psfDesktop;
//
//     HRESULT hr = SHGetDesktopFolder(&psfDesktop);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     STRRET strret;
//     hr = psfDesktop->GetDisplayNameOf(pidlItem, SHGDN_FORPARSING, &strret);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     WCHAR wszBuf[MAX_PATH];
//     hr = StrRetToBufW(&strret, pidlItem, wszBuf, _countof(wszBuf));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     WCHAR wszParams[MAX_PATH];
//     wcscpy(wszParams, L"/select,");
//     wcscat(wszParams, wszBuf);
//
//     SHELLEXECUTEINFOW sei;
//     memset(&sei, 0, sizeof sei);
//     sei.cbSize = sizeof sei;
//     sei.fMask = SEE_MASK_WAITFORINPUTIDLE;
//     sei.lpFile = L"explorer.exe";
//     sei.lpParameters = wszParams;
//
//     if (ShellExecuteExW(&sei))
//         return S_OK;
//     else
//         return E_FAIL;
// }
//
// /*
//  * for internal use
//  */
// HRESULT
// SHELL32_ShowPropertiesDialog(IDataObject *pdtobj)
// {
//     if (!pdtobj)
//         return E_INVALIDARG;
//     return SHELL32_ShowFilesystemItemsPropertiesDialogAsync(NULL, pdtobj);
// }
//
// HRESULT
// SHELL32_DefaultContextMenuCallBack(IShellFolder *psf, IDataObject *pdo, UINT msg)
// {
//     switch (msg)
//     {
//         case DFM_MERGECONTEXTMENU:
//             return S_OK; // Yes, I want verbs
//         case DFM_INVOKECOMMAND:
//             return S_FALSE; // Do it for me please
//         case DFM_GETDEFSTATICID:
//             return S_FALSE; // Supposedly "required for Windows 7 to pick a default"
//     }
//     return E_NOTIMPL;
// }
//
// /**************************************************************************
//  * Rust bridges — C ABI shims over the C++-linkage folder helpers for the
//  * Rust-owned shell folder modules (shell32-rust).
//  */
//
// extern "C" HRESULT WINAPI
// SHELL32_BindToSF_RustBridge(
//     LPCITEMIDLIST pidlRoot, PERSIST_FOLDER_TARGET_INFO *ppfti,
//     LPCITEMIDLIST pidl, const GUID *clsid, REFIID riid, LPVOID *ppvOut)
// {
//     return SHELL32_BindToSF(pidlRoot, ppfti, pidl, clsid, riid, ppvOut);
// }
//
// extern "C" HRESULT WINAPI
// SHELL32_CompareChildren_RustBridge(
//     IShellFolder2 *psf, LPARAM lParam, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2)
// {
//     return SHELL32_CompareChildren(psf, lParam, pidl1, pidl2);
// }
//
// extern "C" HRESULT WINAPI
// SHELL32_GetDisplayNameOfChild_RustBridge(
//     IShellFolder2 *psf, LPCITEMIDLIST pidl, DWORD dwFlags, LPSTRRET strRet)
// {
//     return SHELL32_GetDisplayNameOfChild(psf, pidl, dwFlags, strRet);
// }
//
// extern "C" HRESULT WINAPI
// SH32_GetDetailsOfPKeyAsVariant_RustBridge(
//     IShellFolder2 *pSF, PCUITEMID_CHILD pidl, const SHCOLUMNID *pscid,
//     VARIANT *pVar, BOOL UseFsColMap)
// {
//     return SH32_GetDetailsOfPKeyAsVariant(pSF, pidl, pscid, pVar, UseFsColMap);
// }
//
// extern "C" HRESULT WINAPI
// SHELL32_DefaultContextMenuCallBack_RustBridge(
//     IShellFolder *psf, IDataObject *pdo, UINT uMsg)
// {
//     return SHELL32_DefaultContextMenuCallBack(psf, pdo, uMsg);
// }
//
// extern "C" HRESULT WINAPI
// SHELL_CreateSHDESCRIPTIONID_RustBridge(VARIANT *pVar, DWORD Id, const CLSID *pCLSID)
// {
//     return SHELL_CreateSHDESCRIPTIONID(pVar, Id, pCLSID);
// }
//
// extern "C" HRESULT WINAPI
// SHELL_GetDetailsOfAsStringVariant_RustBridge(
//     IShellFolder2 *pSF, PCUITEMID_CHILD pidl, UINT Column, VARIANT *pVar)
// {
//     return SHELL_GetDetailsOfAsStringVariant(pSF, pidl, Column, pVar);
// }
//
// extern "C" HRESULT WINAPI
// SHELL_CreateVariantBuffer_RustBridge(VARIANT *pVar, UINT cb)
// {
//     return SHELL_CreateVariantBuffer(pVar, cb);
// }
//
// extern "C" HRESULT WINAPI
// Shell_NextElement_RustBridge(LPWSTR *ppch, LPWSTR pszOut, INT cchOut, BOOL bValidate)
// {
//     return Shell_NextElement(ppch, pszOut, cchOut, bValidate);
// }
//
// extern "C" void WINAPI
// AddFSClassKeysToArray_RustBridge(UINT cidl, PCUITEMID_CHILD_ARRAY apidl, HKEY *array, UINT *cKeys)
// {
//     AddFSClassKeysToArray(cidl, apidl, array, cKeys);
// }
//
// extern "C" LSTATUS WINAPI
// AddClassKeyToArray_RustBridge(const WCHAR *szClass, HKEY *array, UINT *cKeys)
// {
//     return AddClassKeyToArray(szClass, array, cKeys);
// }
//
// extern "C" HRESULT WINAPI
// CRecycleBin_CoCreateInitSF_RustBridge(
//     LPCITEMIDLIST pidlRoot, PERSIST_FOLDER_TARGET_INFO *ppfti, LPCITEMIDLIST pidlChild,
//     const GUID *clsid, REFIID riid, LPVOID *ppvOut)
// {
//     return SHELL32_CoCreateInitSF(pidlRoot, ppfti, pidlChild, clsid, riid, ppvOut);
// }
//
// /* Rust bridges for the Rust-owned CDesktopFolder (shell32-rust desktop_folder). */
// extern "C" LSTATUS WINAPI
// AddClsidKeyToArray_RustBridge(REFCLSID clsid, HKEY *array, UINT *cKeys)
// {
//     return AddClsidKeyToArray(clsid, array, cKeys);
// }
//
// extern "C" HRESULT WINAPI
// SHELL32_ShowPropertiesDialog_RustBridge(IDataObject *pdo)
// {
//     return SHELL32_ShowPropertiesDialog(pdo);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
