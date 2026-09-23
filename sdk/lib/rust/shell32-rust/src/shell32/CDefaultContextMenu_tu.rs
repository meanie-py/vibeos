use crate::default_context_menu::CDefaultContextMenu;
use crate::prelude::*;
use crate::default_context_menu_support::*;
// The COM interfaces this half calls through a vtable, by explicit `use` so they beat the
// prelude's island glob — bindgen renders those as C++ classes with an opaque `vtable_` and
// no `lpVtbl` to call through. `IContextMenu` comes from the class half rather than from
// `shellcom` because that copy has every slot typed, and one definition across both halves
// is what keeps `DCMENTRY.pCM` and `DynamicShellEntry.pCM` the same nominal type.
use crate::default_context_menu::{IContextMenu, IContextMenuVtbl};
use crate::default_context_menu_support::{IShellExtInit, IShellExtInitVtbl, DCMENTRY};
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1338A3B61B21: [WCHAR; 5] = [0x0066, 0x0069, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_28B86EE8967F: [WCHAR; 30] = [0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0065, 0x0078, 0x005C, 0x004D, 0x0061, 0x0079, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000];
pub static CPP_COM_WSTR_6E342AB1A0F1: [WCHAR; 7] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];
pub static CPP_COM_WSTR_878FA9FF9662: [WCHAR; 6] = [0x0072, 0x0075, 0x006E, 0x0061, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A66B4CE0DBF3: [WCHAR; 7] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0061, 0x0073, 0x0000];
pub static CPP_COM_WSTR_DACCBBBCFACF: [WCHAR; 6] = [0x0070, 0x0072, 0x0069, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_FDF5BD4663B6: [WCHAR; 5] = [0x0065, 0x0064, 0x0069, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_VERB: usize = 260;

// DynamicShellEntry / StaticShellEntry are declared by the CLASS half, which is where the C++
// puts them and which lowers `pCM` as the `CComPtr<IContextMenu>` the C++ actually has. The
// duplicates this half generated were dead and typed `pCM` as a raw pointer, so they would
// have drifted from the real ones on the next re-assembly of either file.

pub const DCM_FCIDM_SHVIEW_OFFSET: u32 = 0x7000;

#[repr(C)]
pub struct _StaticInvokeCommandMap_ {
    pub szStringVerb: LPCSTR,
    pub IntVerb: WORD,
    pub DfmCmd: SHORT,
}

// Same as `VerbMap` in the class half: a `static` must be Sync and `LPCSTR` is not. The verb
// strings are this file's own byte literals, immutable for the life of the process.
unsafe impl Sync for _StaticInvokeCommandMap_ {}

// Verbs from the registry must not be listed here: "open" mapped to the never-implemented
// DoOpenOrExplore (always E_FAIL), and "runas"/"print"/"preview" to IntVerb 0, so none of
// them could reach the file type's registered verb (upstream 16bb138e325, CORE-20690).
// "copyto"/"moveto" stay: upstream dropped them in the same commit only alongside moving
// Copy To/Move To into CCopyMoveToMenu, which this tree has not taken.
pub static g_StaticInvokeCmdMap: [_StaticInvokeCommandMap_; 10] = [
    _StaticInvokeCommandMap_ { szStringVerb: CMDSTR_NEWFOLDERA.as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_NEWFOLDER as u16, DfmCmd: DFM_CMD_NEWFOLDER as SHORT },
    _StaticInvokeCommandMap_ { szStringVerb: b"cut\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_CUT as u16, DfmCmd: 0 },
    _StaticInvokeCommandMap_ { szStringVerb: b"copy\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_COPY as u16, DfmCmd: DFM_CMD_COPY as SHORT },
    _StaticInvokeCommandMap_ { szStringVerb: b"paste\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_INSERT as u16, DfmCmd: DFM_CMD_PASTE as SHORT },
    _StaticInvokeCommandMap_ { szStringVerb: b"link\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_CREATELINK as u16, DfmCmd: DFM_CMD_LINK as SHORT },
    _StaticInvokeCommandMap_ { szStringVerb: b"delete\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_DELETE as u16, DfmCmd: DFM_CMD_DELETE as SHORT },
    _StaticInvokeCommandMap_ { szStringVerb: b"properties\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_PROPERTIES as u16, DfmCmd: DFM_CMD_PROPERTIES as SHORT },
    _StaticInvokeCommandMap_ { szStringVerb: b"rename\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_RENAME as u16, DfmCmd: DFM_CMD_RENAME as SHORT },
    _StaticInvokeCommandMap_ { szStringVerb: b"copyto\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_COPYTO as u16, DfmCmd: 0 },
    _StaticInvokeCommandMap_ { szStringVerb: b"moveto\0".as_ptr() as LPCSTR, IntVerb: FCIDM_SHVIEW_MOVETO as u16, DfmCmd: 0 },
];
unsafe extern "C" fn SHELL_GetRegCLSID(hKey: HKEY, SubKey: PCWSTR, Value: PCWSTR, clsid: *mut CLSID) -> HRESULT {
    let mut buf: [u16; 42] = core::mem::zeroed();
    let mut cb: DWORD = core::mem::size_of_val(&buf) as DWORD;
    let err: DWORD = RegGetValueW(hKey, SubKey, Value, RRF_RT_REG_SZ, core::ptr::null_mut(), buf.as_mut_ptr().cast(), core::ptr::addr_of_mut!(cb)) as u32;
    if err == 0 {
        CLSIDFromString(buf.as_ptr(), clsid)
    } else {
        HRESULT_FROM_WIN32(err as i32)
    }
}
// NB the parameter is `Pos_`, not the C's `Pos`: `ntdll-sys` exports
// `pub const Pos: _BUS_DATA_TYPE` (a bindgen enum alias) and the prelude glob brings it
// into scope here, so an identifier `Pos` in a parameter list resolves as a CONSTANT
// PATTERN rather than a binding. rustc then reports the mismatch against the const's
// i32 type, on the parameter list itself, with no mention of the collision.
unsafe extern "C" fn InsertMenuItemAt(hMenu: HMENU, Pos_: UINT, Flags: UINT) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = FIELD_OFFSET!(MENUITEMINFOW, hbmpItem) as UINT;
    mii.fMask = MIIM_TYPE;
    mii.fType = Flags;
    InsertMenuItemW(hMenu, Pos_, TRUE, &mut mii)
}
unsafe extern "C" fn DCMA_DestroyEntry(dcme: *mut DCMENTRY) {
    if (*dcme).pCM.is_null() {
        return;
    }
    IUnknown_SetSite((*dcme).pCM.cast(), core::ptr::null_mut());
    ((*(*(*dcme).pCM).lpVtbl).Release)((*dcme).pCM);
    (*dcme).pCM = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn DCMA_Destroy(hDCMA: HDCMA) {
    let mut i = 0;
    loop {
        let p = DCMA_GetEntry(hDCMA, i);
        if p.is_null() {
            break;
        }
        DCMA_DestroyEntry(p);
        i += 1;
    }
    DSA_Destroy(hDCMA);
}
#[no_mangle]
pub unsafe extern "C" fn DCMA_InsertMenuItems(
    hDCMA: HDCMA,
    hDCIA: HDCIA,
    pidlFolder: LPCITEMIDLIST,
    pDO: *mut IDataObject,
    pKeys: *mut HKEY,
    nKeys: UINT,
    pQCMI: *mut QCMINFO,
    fCmf: UINT,
    pUnkSite: *mut IUnknown,
) -> UINT {
    let mut idCmdBase = (*pQCMI).idCmdFirst;
    let mut idCmdFirst = idCmdBase;
    let mut nOffset = 0;

    let mut iCls = DCIA_GetCount(hDCIA) - 1;
    while iCls >= 0 {
        let clsid = *DCIA_GetEntry(hDCIA, iCls);
        if (fCmf & CMF_DEFAULTONLY) != 0 {
            let mut szKey = [0u16; MAX_PATH as usize];
            wcscpy(szKey.as_mut_ptr(), CPP_COM_WSTR_6E342AB1A0F1.as_ptr());
            StringFromGUID2(
                core::ptr::addr_of!(clsid).cast(),
                szKey.as_mut_ptr().offset(_countof!(CPP_COM_WSTR_6E342AB1A0F1) as isize - 1),
                CHARS_IN_GUID as i32,
            );
            wcscpy(
                szKey.as_mut_ptr().offset(
                    (_countof!(CPP_COM_WSTR_6E342AB1A0F1) - 1 + CHARS_IN_GUID - 1) as isize,
                ),
                CPP_COM_WSTR_28B86EE8967F.as_ptr(),
            );
            if !RegKeyExists(HKEY_CLASSES_ROOT, szKey.as_ptr()) {
                iCls -= 1;
                continue;
            }
        }

        let mut iKey = 0;
        while iKey < nKeys {
            let mut pInit: *mut IShellExtInit = core::ptr::null_mut();
            let mut hr = SHExtCoCreateInstance(
                core::ptr::null_mut(),
                &clsid,
                core::ptr::null_mut(),
                core::ptr::addr_of!(IID_IShellExtInit).cast(), core::ptr::addr_of_mut!(pInit).cast(),
            );
            if hr < 0 {
                break;
            }
            hr = ((*(*pInit).lpVtbl).Initialize)(pInit, pidlFolder, pDO, *pKeys.offset(iKey as isize));
            if hr < 0 {
                iKey += 1;
                continue;
            }

            let mut pCM: *mut IContextMenu = core::ptr::null_mut();
            hr = ((*(*pInit).lpVtbl).QueryInterface)(pInit, core::ptr::addr_of!(IID_IContextMenu).cast(), core::ptr::addr_of_mut!(pCM).cast());
            if hr < 0 {
                break;
            }
            IUnknown_SetSite(pCM.cast(), pUnkSite.cast());

            hr = ((*(*pCM).lpVtbl).QueryContextMenu)(pCM,
                (*pQCMI).hmenu,
                (*pQCMI).indexMenu + nOffset,
                idCmdFirst,
                (*pQCMI).idCmdLast,
                fCmf,
            );
            let nCount: UINT = HRESULT_CODE(hr) as UINT;
            let idThisFirst = idCmdFirst - idCmdBase;
            let mut dcme = DCMENTRY {
                pCM,
                idCmdFirst: idThisFirst,
                idCmdLast: idThisFirst + nCount - 1,
            };
            if hr > 0 {
                idCmdFirst += nCount;
                if DSA_AppendItem(hDCMA, &dcme) >= 0 {
                    if nOffset == 0
                        && GetMenuDefaultItem((*pQCMI).hmenu, TRUE as u32, 0)
                            == 0
                    {
                        nOffset += 1;
                    }
                    break;
                }
            }
            DCMA_DestroyEntry(&mut dcme);
            iKey += 1;
        }
        iCls -= 1;
    }
    idCmdFirst
}
#[no_mangle]
pub unsafe extern "C" fn DCMA_InvokeCommand(hDCMA: HDCMA, pICI: *mut CMINVOKECOMMANDINFO) -> HRESULT {
    let mut hr: HRESULT = S_FALSE;
    let mut i: UINT = 0;
    loop {
        let p: *mut DCMENTRY = DCMA_GetEntry(hDCMA, i as i32);
        if p.is_null() {
            return hr;
        }

        let id: UINT = ((*pICI).lpVerb as usize & 0xffff) as UINT;  // C: LOWORD(pICI->lpVerb)
        if !IS_INTRESOURCE((*pICI).lpVerb) {
            hr = ((*(*(*p).pCM).lpVtbl).InvokeCommand)((*p).pCM, pICI);
            if hr >= 0 {
                return hr;
            }
        } else if id >= (*p).idCmdFirst && id <= (*p).idCmdLast {
            let mut ici: CMINVOKECOMMANDINFOEX = core::mem::zeroed();
            CopyMemory(
                &mut ici as *mut CMINVOKECOMMANDINFOEX as *mut core::ffi::c_void,
                pICI as *const core::ffi::c_void,
                min(core::mem::size_of::<CMINVOKECOMMANDINFOEX>(), (*pICI).cbSize as usize),
            );
            ici.cbSize = min(core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as UINT, (*pICI).cbSize);
            ici.lpVerb = MAKEINTRESOURCEA(id.wrapping_sub((*p).idCmdFirst));
            ici.lpVerbW = ici.lpVerb as PWSTR;
            return ((*(*(*p).pCM).lpVtbl).InvokeCommand)((*p).pCM, &mut ici as *mut CMINVOKECOMMANDINFOEX as *mut CMINVOKECOMMANDINFO);
        }

        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MapFcidmCmdToVerb(CmdId: UINT_PTR) -> PCSTR {
    let mut i: usize = 0;
    while i < _countof!(g_StaticInvokeCmdMap) as usize {
        if UINT_PTR::from(g_StaticInvokeCmdMap[i].IntVerb) == CmdId && CmdId != 0 {
            return g_StaticInvokeCmdMap[i].szStringVerb;
        }
        i += 1;
    }
    core::ptr::null()
}
#[no_mangle]
pub unsafe extern "C" fn MapVerbToDfmCmd(verba: LPCSTR) -> UINT {
    let mut i = 0;
    while i < _countof!(g_StaticInvokeCmdMap) as usize {
        if lstrcmpiA(g_StaticInvokeCmdMap[i as usize].szStringVerb, verba) == 0 {
            return g_StaticInvokeCmdMap[i as usize].DfmCmd as UINT;
        }
        i += 1;
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn MapVerbToCmdId(Verb: PVOID, IsUnicode: BOOL, pCM: *mut IContextMenu, idFirst: UINT, idLast: UINT) -> HRESULT {
    let gcs = if IsUnicode != 0 { GCS_VERBW } else { GCS_VERBA };
    let mut id = idFirst;
    while id <= idLast {
        let mut buf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        buf[0] = UNICODE_NULL;
        let hr = ((*(*pCM).lpVtbl).GetCommandString)(pCM, id as UINT_PTR, gcs, core::ptr::null_mut(), buf.as_mut_ptr() as LPSTR, _countof!(buf));
        if (hr) < 0 || buf[0] == 0 {
            id += 1;
            continue;
        } else if IsUnicode != 0 && _wcsicmp(Verb as LPWSTR, buf.as_ptr()) == 0 {
            return id as HRESULT;
        } else if IsUnicode == 0 && lstrcmpiA(Verb as LPSTR, buf.as_ptr() as LPSTR) == 0 {
            return id as HRESULT;
        }
        id += 1;
    }
    HRESULT_FROM_WIN32(ERROR_NOT_FOUND as i32)
}
unsafe extern "C" fn IsVerbListSeparator(Ch: u16) -> bool {
    Ch == 0x0020u16 || Ch == 0x002Cu16
}
unsafe extern "C" fn FindVerbInDefaultVerbList(mut List: PCWSTR, Verb: PCWSTR) -> i32 {
    let mut index: u32 = 0;
    while *List != 0 {
        while IsVerbListSeparator(*List) {
            List = List.offset(1);
        }
        let Start = List;
        while *List != 0 && !IsVerbListSeparator(*List) {
            List = List.offset(1);
        }
        if _wcsnicmp(Verb, Start, (List as usize - Start as usize) / core::mem::size_of::<u16>()) == 0 && List > Start {
            return index as i32;
        }
        index += 1;
    }
    -(1 as i32)
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_EnumDefaultVerbList(
    mut List: PCWSTR,
    Index: UINT,
    Verb: PWSTR,
    cchMax: usize,
) -> HRESULT {
    let mut i: UINT = 0;
    while *List != 0 {
        while IsVerbListSeparator(*List) {
            List = List.offset(1);
        }
        let Start: PCWSTR = List;
        while *List != 0 && !IsVerbListSeparator(*List) {
            List = List.offset(1);
        }
        if List > Start && i == Index {
            return StringCchCopyNW(Verb, cchMax, Start, List.offset_from(Start) as usize);
        }
        i += 1;
    }
    return HRESULT_FROM_WIN32(ERROR_NO_MORE_ITEMS as i32);
}
unsafe extern "C" fn GetFriendlyVerb(pszVerb: PCWSTR, pszBuf: PWSTR, cchMax: usize) -> HRESULT {
    const MAP: &[(PCWSTR, u16)] = &[
        (CPP_COM_WSTR_FDF5BD4663B6.as_ptr(), IDS_EDIT_VERB as u16),
        (CPP_COM_WSTR_DACCBBBCFACF.as_ptr(), IDS_PRINT_VERB as u16),
        (CPP_COM_WSTR_878FA9FF9662.as_ptr(), IDS_RUNAS_VERB as u16),
        (CPP_COM_WSTR_A66B4CE0DBF3.as_ptr(), IDS_OPEN_VERB as u16),
        (CPP_COM_WSTR_1338A3B61B21.as_ptr(), IDS_FIND_VERB as u16),
    ];
    let mut i: usize = 0;
    while i < _countof!(MAP) as usize {
        if _wcsicmp(pszVerb, MAP[i].0) == 0 &&
           LoadStringW(shell32_hInstance, MAP[i].1.into(), pszBuf, cchMax as i32) != 0
        {
            return S_OK;
        }
        i += 1;
    }

    let mut cchMax_mut = cchMax;
    if (*pszVerb.offset(0)) < 127 &&
       StrChrW(pszVerb, 0x0026u16) == core::ptr::null_mut() &&
       {
           cchMax_mut -= 1;
           StringCchCopyW(pszBuf.offset(1), cchMax_mut, pszVerb)
       } >= 0
    {
        *pszBuf = 0x0026u16;
        return S_OK;
    }
    E_FAIL
}
unsafe extern "C" fn HasClipboardData() -> u8 {
    let mut bRet: u8 = FALSE as u8;
    let mut pDataObj: *mut IDataObject = core::ptr::null_mut();

    if OleGetClipboard(&mut pDataObj) >= 0 {
        let mut formatetc: FORMATETC = core::mem::zeroed();

        InitFormatEtc(&mut formatetc, RegisterClipboardFormatW(CFSTR_SHELLIDLIST.as_ptr()), TYMED_HGLOBAL);
        bRet = if ((*(*pDataObj).lpVtbl).QueryGetData)(pDataObj, core::ptr::addr_of_mut!(formatetc).cast()) >= 0 { 1u8 } else { 0u8 };
    }

    if !pDataObj.is_null() {
        ((*(*pDataObj).lpVtbl).Release)(pDataObj, );
    }

    bRet
}

#[no_mangle]
pub unsafe extern "system" fn _InsertMenuItemW(
    hMenu: HMENU,
    indexMenu: UINT,
    fByPosition: BOOL,
    wID: UINT,
    fType: UINT,
    dwTypeData: LPCWSTR,
    fState: UINT,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut wszText: [WCHAR; 100] = core::mem::zeroed();

    ZeroMemory(
        &mut mii as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of_val(&mii),
    );
    mii.cbSize = core::mem::size_of_val(&mii) as UINT;
    if fType == MFT_SEPARATOR {
        mii.fMask = MIIM_ID | MIIM_TYPE;
    } else if fType == MFT_STRING {
        mii.fMask = MIIM_ID | MIIM_TYPE | MIIM_STATE;
        if IS_INTRESOURCE(dwTypeData) {
            // C: LoadStringW(shell32_hInstance, LOWORD((ULONG_PTR)dwTypeData), wszText,
            // _countof(wszText)). The generator dropped the LOWORD argument and shifted the
            // rest left; restored from the C.
            if LoadStringW(
                shell32_hInstance,
                ((dwTypeData as usize) & 0xffff) as UINT,
                wszText.as_mut_ptr(),
                _countof!(wszText) as i32,
            ) != 0
            {
                mii.dwTypeData = wszText.as_mut_ptr();
            } else {
                return FALSE;
            }
        } else {
            mii.dwTypeData = dwTypeData as LPWSTR;
        }
        mii.fState = fState;
    }

    mii.wID = wID;
    mii.fType = fType;
    InsertMenuItemW(hMenu, indexMenu, fByPosition, &mut mii)
}
#[no_mangle]
pub unsafe extern "C" fn SHGetMenuIdFromMenuMsg(uMsg: UINT, lParam: LPARAM, CmdId: *mut UINT) -> HRESULT {
    if uMsg == WM_DRAWITEM {
        let pDrawStruct: *mut DRAWITEMSTRUCT = lParam as *mut DRAWITEMSTRUCT;
        *CmdId = (*pDrawStruct).itemID;
        return S_OK;
    } else if uMsg == WM_MEASUREITEM {
        let pMeasureStruct: *mut MEASUREITEMSTRUCT = lParam as *mut MEASUREITEMSTRUCT;
        *CmdId = (*pMeasureStruct).itemID;
        return S_OK;
    }
    E_FAIL
}
// `ShellObjectCreatorInit<CDefaultContextMenu>` expands to the non-aggregating factory that
// the CLASS half already carries — that is where the vtables and the ctor live, so this half
// imports it instead of standing up a second, divergent copy.
use crate::default_context_menu::CDefaultContextMenu_CreateInstance;

#[no_mangle]
pub unsafe extern "system" fn SHCreateDefaultContextMenu(
    pdcm: *const DEFCONTEXTMENU,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut hr: HRESULT;

    if ppv.is_null() {
        return E_INVALIDARG;
    }

    hr = CDefaultContextMenu_CreateInstance(pdcm.cast(), None, riid, ppv);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn CDefFolderMenu_Create2(
    pidlFolder: PCIDLIST_ABSOLUTE,
    hwnd: HWND,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    psf: *mut IShellFolder,
    lpfn: LPFNDFMCALLBACK,
    nKeys: UINT,
    ahkeyClsKeys: *const HKEY,
    ppcm: *mut *mut IContextMenu,
) -> HRESULT {
    let mut dcm: DEFCONTEXTMENU = core::mem::zeroed();
    dcm.hwnd = hwnd;
    dcm.pcmcb = core::ptr::null_mut();
    dcm.pidlFolder = pidlFolder;
    dcm.psf = psf;
    dcm.cidl = cidl;
    dcm.apidl = apidl;
    dcm.punkAssociationInfo = core::ptr::null_mut();
    dcm.cKeys = nKeys;
    dcm.aKeys = ahkeyClsKeys;

    let hr = CDefaultContextMenu_CreateInstance(
        core::ptr::addr_of!(dcm).cast(),
        lpfn,
        core::ptr::addr_of!(IID_IContextMenu).cast(), (ppcm).cast(),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DCMA_InsertMenuItems_RustBridge(
    hDCMA: HDCMA,
    hDCIA: HDCIA,
    pidlFolder: LPCITEMIDLIST,
    pDO: *mut IDataObject,
    pKeys: *mut HKEY,
    nKeys: UINT,
    pQCMI: *mut QCMINFO,
    fCmf: UINT,
    pUnkSite: *mut IUnknown,
) -> UINT {
    DCMA_InsertMenuItems(hDCMA, hDCIA, pidlFolder, pDO, pKeys, nKeys, pQCMI, fCmf, pUnkSite)
}

#[no_mangle]
pub unsafe extern "system" fn DCMA_InvokeCommand_RustBridge(
    hDCMA: HDCMA,
    pICI: *mut CMINVOKECOMMANDINFO,
) -> HRESULT {
    DCMA_InvokeCommand(hDCMA, pICI)
}

#[no_mangle]
pub unsafe extern "system" fn DCMA_Destroy_RustBridge(hDCMA: HDCMA) {
    DCMA_Destroy(hDCMA);
}
