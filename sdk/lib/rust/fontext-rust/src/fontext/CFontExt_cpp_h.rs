use crate::prelude::*;
use crate::fontext::CFontCache_cpp_h::g_FontCache;
use crate::fontext::fontext_cpp_h::g_ModuleRefCnt;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_49342E15DCFE: [WCHAR; 6] = [0x0066, 0x006F, 0x006E, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_B0EE5A657FD2: [WCHAR; 13] = [0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_D4A8C29EA519: [WCHAR; 6] = [0x0052, 0x0048, 0x0053, 0x0041, 0x0043, 0x0000];

use core::ffi::c_void;







































#[no_mangle]
pub unsafe extern "C" fn GetInterfaceName(InterfaceString: *const u16, buf: *mut u16, size: usize) -> bool {
    let mut LocalBuf: [u16; 100] = [0; 100];
    let mut dwType: u32 = 0;
    let mut dwDataSize: u32 = (size * core::mem::size_of::<u16>()) as u32;

    if !((StringCchPrintfW!(LocalBuf.as_mut_ptr(), _countof!(LocalBuf), CPP_COM_WSTR_B0EE5A657FD2.as_ptr(), InterfaceString)) >= 0) {
        return false;
    }

    return RegGetValueW(HKEY_CLASSES_ROOT, LocalBuf.as_ptr(), core::ptr::null_mut(), RRF_RT_REG_SZ, core::ptr::addr_of_mut!(dwType), (buf).cast(), core::ptr::addr_of_mut!(dwDataSize)) == (ERROR_SUCCESS as i32);
}
#[no_mangle]
pub unsafe extern "C" fn g2s(iid: *const CLSID) -> *mut u16 {
    static mut buf: [[u16; 300]; 2] = [[0; 300]; 2];
    static mut idx: i32 = 0;

    idx ^= 1;

    let mut tmp: LPWSTR = core::ptr::null_mut();
    let hr: HRESULT = ProgIDFromCLSID(iid, core::ptr::addr_of_mut!(tmp));
    if hr >= 0 {
        wcscpy(core::ptr::addr_of_mut!(buf[idx as usize]) as *mut u16, tmp);
        CoTaskMemFree(tmp as *mut core::ffi::c_void);
        return core::ptr::addr_of_mut!(buf[idx as usize]) as *mut u16;
    }
    StringFromGUID2(iid, core::ptr::addr_of_mut!(buf[idx as usize]) as *mut u16, (_countof!(buf[idx as usize])) as INT);
    if GetInterfaceName(core::ptr::addr_of_mut!(buf[idx as usize]) as *const u16, core::ptr::addr_of_mut!(buf[idx as usize]) as *mut u16, (_countof!(buf[idx as usize])) as _) {
        return core::ptr::addr_of_mut!(buf[idx as usize]) as *mut u16;
    }
    StringFromGUID2(iid, core::ptr::addr_of_mut!(buf[idx as usize]) as *mut u16, (_countof!(buf[idx as usize])) as INT);

    core::ptr::addr_of_mut!(buf[idx as usize]) as *mut u16
}

unsafe extern "C" fn FONTEXT_GetAttributeString(dwAttributes: DWORD, pszOut: LPWSTR, cchMax: UINT) -> HRESULT {
    let AttrLetters: PCWSTR = CPP_COM_WSTR_D4A8C29EA519.as_ptr(); // Read-only, Hidden, System, Archive, Compressed

    let mut ich: UINT = 0;
    if (dwAttributes & FILE_ATTRIBUTE_READONLY) != 0 && ich < cchMax {
        *pszOut.add(ich as usize) = *AttrLetters.add(0 as usize);
        ich += 1;
    }
    if (dwAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 && ich < cchMax {
        *pszOut.add(ich as usize) = *AttrLetters.add(1 as usize);
        ich += 1;
    }
    if (dwAttributes & FILE_ATTRIBUTE_SYSTEM) != 0 && ich < cchMax {
        *pszOut.add(ich as usize) = *AttrLetters.add(2 as usize);
        ich += 1;
    }
    if (dwAttributes & FILE_ATTRIBUTE_ARCHIVE) != 0 && ich < cchMax {
        *pszOut.add(ich as usize) = *AttrLetters.add(3 as usize);
        ich += 1;
    }
    if (dwAttributes & FILE_ATTRIBUTE_COMPRESSED) != 0 && ich < cchMax {
        *pszOut.add(ich as usize) = *AttrLetters.add(4 as usize);
        ich += 1;
    }
    if ich < cchMax {
        *pszOut.add(ich as usize) = UNICODE_NULL;
        return S_OK;
    }

    return E_FAIL;
}

impl CFontExt {
    pub unsafe extern "system" fn MenuCallback(psf: *mut IShellFolder, hwnd: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let pThis: *mut CFontExt = psf as *mut CFontExt;
        if pThis.is_null() {
            return E_FAIL;
        }

        'sw: {
            match uMsg {
                DFM_MERGECONTEXTMENU => {
                    let pqcminfo: *mut QCMINFO = lParam as *mut QCMINFO;
                    // Insert [Preview] menu item
                    let mut strPreview: CStringW = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_FONT_PREVIEW));
                    InsertMenuW((*pqcminfo).hmenu, 0, MF_BYPOSITION | MF_STRING,
                        { let v = (*pqcminfo).idCmdFirst; (*pqcminfo).idCmdFirst += 1; (v) as _ }, strPreview.GetString()); // Command 0
                    // Make it default
                    SetMenuDefaultItem((*pqcminfo).hmenu, (*pqcminfo).idCmdFirst - 1, FALSE);
                    return S_OK;
                }
                DFM_GETVERBA | DFM_GETVERBW => {
                    // Replace default "open" command action
                    let idCmd: UINT = LOWORD!(wParam) as UINT;
                    let cchMax: UINT = HIWORD!(wParam) as UINT;
                    if idCmd == 0 {
                        if uMsg == DFM_GETVERBA {
                            lstrcpynA(lParam as PSTR, b"open\0".as_ptr() as *const i8, cchMax as i32);
                        } else {
                            lstrcpynW(lParam as PWSTR, CPP_COM_WSTR_82DB442E7A60.as_ptr(), cchMax as i32);
                        }
                        return S_OK;
                    }
                    break 'sw;
                }
                DFM_INVOKECOMMANDEX => {
                    return E_NOTIMPL;
                }
                DFM_INVOKECOMMAND => {
                    if wParam == 0 {
                        return (*pThis).PreviewItems(pdtobj);
                    }
                    if wParam == (DFM_CMD_COPY) as _ {
                        return S_FALSE;
                    }
                    if wParam == (DFM_CMD_DELETE) as _ {
                        return (*pThis).DeleteItems(pdtobj);
                    }
                    if wParam == (DFM_CMD_PASTE) as _ {
                        return S_FALSE;
                    }
                    if wParam == (DFM_CMD_PROPERTIES) as _ {
                        return S_FALSE;
                    }
                    if wParam == (DFM_CMD_MOVE) as _ {
                        return E_NOTIMPL;
                    }
                    return E_FAIL;
                }
                DFM_GETDEFSTATICID => { // Required for Windows 7 to pick a default
                    return S_FALSE;
                }
                DFM_WM_INITMENUPOPUP => {
                    let hMenu: HMENU = wParam as HMENU;
                    // Delete default [Open] menu item
                    DeleteMenu(hMenu, FCIDM_SHVIEW_OPEN, MF_BYCOMMAND);
                    // Disable [Paste link] menu item
                    EnableMenuItem(hMenu, FCIDM_SHVIEW_INSERTLINK, MF_BYCOMMAND | MF_GRAYED);
                    break 'sw;
                }
                _ => {}
            }
        }
        return E_NOTIMPL;
    }
    pub unsafe fn PreviewItems(&mut self, pDataObj: *mut IDataObject) -> HRESULT {
        let this_: *mut Self = self;
        let hr: HRESULT = DoPreviewFontFiles((*this_).m_hwndView, pDataObj);
        FAILED_UNEXPECTEDLY!(hr);
        return S_OK; // Override default action
    }
    pub unsafe fn DeleteItems(&mut self, pDataObj: *mut IDataObject) -> HRESULT {
        let this_: *mut Self = self;
        let hr: HRESULT = DoDeleteFontFiles((*this_).m_hwndView, pDataObj);
        FAILED_UNEXPECTEDLY!(hr);
        return S_OK; // Override default action
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_bDragAccepted = (FALSE as i32);
        (*this_).m_hwndView = core::ptr::null_mut();
        {
            InterlockedIncrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let _ = this_;
        InterlockedDecrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
    }
    pub unsafe fn SetViewWindow(&mut self, hwndView: HWND) {
        let this_: *mut Self = self;
        (*this_).m_hwndView = hwndView;
    }
    pub unsafe fn GetDefaultSearchGUID(&mut self, lpguid: *mut GUID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn EnumSearches(&mut self, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
        let this_: *mut Self = self;
        *ppenum = core::ptr::null_mut();
        return E_NOTIMPL;
    }
    pub unsafe fn GetDefaultColumn(&mut self, dwReserved: DWORD,
        pSort: *mut ULONG,
        pDisplay: *mut ULONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultColumnState(&mut self, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
        let this_: *mut Self = self;
        if pcsFlags.is_null() || iColumn >= _countof!(g_ColumnDefs) as UINT {
            return E_INVALIDARG;
        }

        *pcsFlags = g_ColumnDefs[iColumn as usize].dwDefaultState;
        return S_OK;
    }
    pub unsafe fn GetDetailsEx(&mut self, pidl: PCUITEMID_CHILD,
        pscid: *const SHCOLUMNID,
        pv: *mut VARIANT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // ERR("%s() UNIMPLEMENTED\n", __FUNCTION__);
        E_NOTIMPL
    }
    pub unsafe fn GetDetailsOf(&mut self, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
        let this_: *mut Self = self;
        if iColumn >= _countof!(g_ColumnDefs) {
            return E_FAIL;
        }

        (*psd).cxChar = g_ColumnDefs[iColumn as usize].cxChar;
        (*psd).fmt = g_ColumnDefs[iColumn as usize].fmt;

        // No item requested, so return the column name
        if pidl.is_null() {
            return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), _AtlBaseModule.GetResourceInstance(), g_ColumnDefs[iColumn as usize].iResource);
        }

        // Validate that this pidl is the last one
        let curpidl: PCUIDLIST_RELATIVE = ILGetNext(pidl);
        if (*curpidl).mkid.cb != 0 {
            return E_FAIL;
        }

        // Name, ReactOS specific?
        if iColumn == (FONTEXT_COL_NAME as UINT) {
            return (*this_).GetDisplayNameOf(pidl, 0, core::ptr::addr_of_mut!((*psd).str_));
        }

        let fontEntry: *const FontPidlEntry = _FontFromIL(pidl);
        if fontEntry.is_null() {
            return E_FAIL;
        }

        // If we got here, we are in details view!
        let info = (*g_FontCache).Find(fontEntry);
        if info.is_null() {
            return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
        }

        let mut ret: i32;
        let mut time: SYSTEMTIME = core::mem::zeroed();
        match (iColumn) as i32 {
            FONTEXT_COL_FILENAME => {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), PathFindFileNameW(((*info).File()).GetString()));
            }
            FONTEXT_COL_SIZE => {
                (*psd).str_.uType = STRRET_CSTR;
                StrFormatKBSizeA((*(*info).FileSize()).QuadPart, (*psd).str_.__bindgen_anon_1.cStr.as_mut_ptr(), MAX_PATH);
                return S_OK;
            }
            FONTEXT_COL_MODIFIED => {
                (*psd).str_.uType = STRRET_CSTR;
                FileTimeToSystemTime((*info).FileWriteTime(), core::ptr::addr_of_mut!(time));
                ret = GetDateFormatA(LOCALE_USER_DEFAULT, DATE_SHORTDATE, core::ptr::addr_of_mut!(time), core::ptr::null_mut(), (*psd).str_.__bindgen_anon_1.cStr.as_mut_ptr(), (MAX_PATH as i32));
                if ret < 1 {
                    return E_FAIL;
                }
                (*psd).str_.__bindgen_anon_1.cStr[(ret - 1) as usize] = ' ' as _;
                GetTimeFormatA(LOCALE_USER_DEFAULT, TIME_NOSECONDS, core::ptr::addr_of_mut!(time), core::ptr::null_mut(), (*psd).str_.__bindgen_anon_1.cStr.as_mut_ptr().add(ret as usize), (MAX_PATH - (ret) as u32) as i32);
                return S_OK;
            }
            FONTEXT_COL_ATTR => {
                let mut szAttr: [WCHAR; 8 as usize] = [0; 8 as usize];
                let hr: HRESULT = FONTEXT_GetAttributeString((*info).FileAttributes(), szAttr.as_mut_ptr(), _countof!(szAttr));
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), szAttr.as_mut_ptr());
            }
            _ => {}
        }

        UNIMPLEMENTED!();
        return E_NOTIMPL;
    }
    pub unsafe fn MapColumnToSCID(&mut self, iColumn: UINT,
        pscid: *mut SHCOLUMNID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn ParseDisplayName(&mut self, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut DWORD, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if lpszDisplayName.is_null() || *lpszDisplayName.add(0 as usize) == 0 || ppidl.is_null() {
            return E_INVALIDARG;
        }

        *ppidl = core::ptr::null_mut();
        if !pchEaten.is_null() {
            *pchEaten = 0;
        }

        if *lpszDisplayName.add(0 as usize) == 0x003Au16 && *lpszDisplayName.add(1 as usize) == 0x003Au16 {
            return E_INVALIDARG;
        }

        if PathIsRelativeW(lpszDisplayName) != 0 { // Not full path?
            return E_INVALIDARG;
        }

        // Load font cache
        if (*g_FontCache).Size() == 0 {
            (*g_FontCache).Read();
        }

        let mut iFont: usize = 0 as usize;
        while iFont < (*g_FontCache).Size() {
            'body: {
                let mut fileName: CStringW = (*g_FontCache).File(iFont);
                if fileName.IsEmpty() {
                    break 'body;
                }

                let mut filePath: CStringW = (*g_FontCache).GetFontFilePath(fileName.GetString());
                if filePath.CompareNoCase(lpszDisplayName) != 0 {
                    break 'body;
                }

                let mut fontName: CStringW = (*g_FontCache).Name(iFont);
                if fontName.IsEmpty() {
                    break 'body;
                }

                // Create a PIDL
                *ppidl = _ILCreate(fontName.GetString(), fileName.GetString());
                if (*ppidl).is_null() {
                    return E_OUTOFMEMORY;
                }

                if !pchEaten.is_null() {
                    *pchEaten = wcslen(lpszDisplayName) as DWORD;
                }

                if !pdwAttributes.is_null() {
                    *pdwAttributes &= (SFGAO_CANDELETE | SFGAO_HASPROPSHEET | SFGAO_CANCOPY |
                                       SFGAO_FILESYSTEM);
                }

                return S_OK;
            }
            iFont += 1;
        }

        return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND); // Not found
    }
    pub unsafe fn EnumObjects(&mut self, hwndOwner: HWND, dwFlags: DWORD, ppEnumIDList: *mut LPENUMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        return _CEnumFonts_CreateInstance(this_, dwFlags, &IID_IEnumIDList, ppEnumIDList as *mut *mut core::ffi::c_void);
    }
    pub unsafe fn BindToObject(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn BindToStorage(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn CompareIDs(&mut self, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
        let this_: *mut Self = self;
        let fontEntry1: *const FontPidlEntry = _FontFromIL(pidl1);
        let fontEntry2: *const FontPidlEntry = _FontFromIL(pidl2);
        if fontEntry1.is_null() || fontEntry2.is_null() {
            return E_INVALIDARG;
        }

        let result: i32;
        let sortMode: DWORD = (lParam as DWORD) & 0xFFFF0000;
        let column: DWORD = (lParam as DWORD) & 0x0000FFFF;
        if sortMode == SHCIDS_ALLFIELDS {
            result = StrCmpIW((*fontEntry1.cast_mut()).Name__1(), (*fontEntry2.cast_mut()).Name__1());
        } else {
            let info1 = (*g_FontCache).Find(fontEntry1);
            let info2 = (*g_FontCache).Find(fontEntry2);

            if info1.is_null() || info2.is_null() {
                return E_INVALIDARG;
            }

            match (column) as i32 {
                0xffff | FONTEXT_COL_NAME => {
                    /* ROS bug? */
                    result = StrCmpIW((*fontEntry1.cast_mut()).Name__1(), (*fontEntry2.cast_mut()).Name__1());
                }
                FONTEXT_COL_FILENAME => {
                    result = StrCmpIW(
                        PathFindFileNameW(((*info1).File()).GetString()),
                        PathFindFileNameW(((*info2).File()).GetString()),
                    );
                }
                FONTEXT_COL_SIZE => {
                    let size1: ULONGLONG = ((*(*info1).FileSize()).QuadPart) as u64;
                    let size2: ULONGLONG = ((*(*info2).FileSize()).QuadPart) as u64;
                    result = if size1 < size2 {
                        -1
                    } else if size1 > size2 {
                        1
                    } else {
                        0
                    };
                }
                FONTEXT_COL_MODIFIED => {
                    result = CompareFileTime((*info1).FileWriteTime(), (*info2).FileWriteTime());
                }
                FONTEXT_COL_ATTR => {
                    let mut hr: HRESULT;
                    let mut szAttr1: [u16; 8] = [0; 8];
                    let mut szAttr2: [u16; 8] = [0; 8];
                    hr = FONTEXT_GetAttributeString((*info1).FileAttributes(), szAttr1.as_mut_ptr(), _countof!(szAttr1));
                    if FAILED_UNEXPECTEDLY!(hr) != false {
                        return hr;
                    }
                    hr = FONTEXT_GetAttributeString((*info2).FileAttributes(), szAttr2.as_mut_ptr(), _countof!(szAttr2));
                    if FAILED_UNEXPECTEDLY!(hr) != false {
                        return hr;
                    }
                    result = _wcsicmp(szAttr1.as_ptr(), szAttr2.as_ptr());
                }
                _ => {
                    ERR!("Unimplemented column %u\n", column);
                    return E_INVALIDARG;
                }
            }
        }

        MAKE_COMPARE_HRESULT!(result)
    }
    pub unsafe fn CreateViewObject(&mut self, hwndOwner: HWND, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = E_NOINTERFACE;

        *ppvOut = core::ptr::null_mut();

        if IsEqualIID!(riid, &IID_IDropTarget) {
            *ppvOut = (core::ptr::addr_of_mut!((*this_).__base4) as *mut IDropTarget).cast();
            (*this_).AddRef();
            hr = S_OK;
        } else if IsEqualIID!(riid, &IID_IContextMenu) {
            return CFontBackgroundMenu_Create(this_, hwndOwner, core::ptr::addr_of_mut!((*this_).__base2) as *mut IShellFolder, ppvOut as *mut *mut IContextMenu);
        } else if IsEqualIID!(riid, &IID_IShellView) {
            let mut sfviewcb: CComPtr<CFontFolderViewCB> = CComPtr::new();
            hr = ShellObjectCreator(&mut sfviewcb);
            if hr >= 0 {
                let mut create: SFV_CREATE = SFV_CREATE {
                    cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
                    pshf: core::ptr::addr_of_mut!((*this_).__base2) as *mut IShellFolder,
                    psvOuter: core::ptr::null_mut(),
                    psfvcb: core::ptr::addr_of_mut!((*sfviewcb.Get()).__base1) as *mut IShellFolderViewCB,
                };
                hr = SHCreateShellFolderView(core::ptr::addr_of_mut!(create), ppvOut as *mut *mut IShellView);
                if hr >= 0 {
                    (*sfviewcb.Get()).Initialize(this_, *ppvOut as *mut IShellView, (*this_).m_Folder.Get());
                }
            }
        }

        return hr;
    }
    pub unsafe fn GetAttributesOf(&mut self, mut cidl: UINT, mut apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if rgfInOut.is_null() || cidl == 0 || apidl.is_null() {
            return E_INVALIDARG;
        }

        if cidl <= 0 {
            *rgfInOut = 0;
            return S_OK;
        }

        let mut rgf: DWORD = (SFGAO_CANDELETE | SFGAO_HASPROPSHEET | SFGAO_CANCOPY | SFGAO_FILESYSTEM);
        while cidl > 0 && !(*apidl).is_null() {
            let fontEntry: *const FontPidlEntry = _FontFromIL(*apidl);
            if fontEntry.is_null() {
                rgf = 0;
                break;
            }

            apidl = apidl.add(1);
            cidl -= 1;
        }

        *rgfInOut &= rgf;
        *rgfInOut &= !SFGAO_VALIDATE; // Make sure SFGAO_VALIDATE is cleared, some apps depend on that

        return S_OK;
    }
    pub unsafe fn GetUIObjectOf(&mut self, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: &IID, prgfInOut: *mut UINT, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualGUID!(&(*riid), &(IID_IContextMenu)) ||
            IsEqualGUID!(&(*riid), &(IID_IContextMenu2)) ||
            IsEqualGUID!(&(*riid), &(IID_IContextMenu3))
        {
            if cidl <= 0 {
                return E_FAIL;
            }

            return CDefFolderMenu_Create2(core::ptr::null_mut(), hwndOwner, cidl, apidl, core::ptr::addr_of_mut!((*this_).__base2) as *mut IShellFolder, Some(CFontExt::MenuCallback),
                                          0, core::ptr::null_mut(), ppvOut as *mut *mut IContextMenu);
        }
        else if IsEqualGUID!(&(*riid), &(IID_IExtractIconA)) || IsEqualGUID!(&(*riid), &(IID_IExtractIconW))
        {
            if cidl == 1
            {
                let fontEntry: *const FontPidlEntry = _FontFromIL(*apidl);
                if !fontEntry.is_null()
                {
                    let dwAttributes: DWORD = FILE_ATTRIBUTE_NORMAL;
                    let mut strFileName: CStringW = (*g_FontCache).GetFontFilePath((*fontEntry.cast_mut()).FileName__1());
                    return SHCreateFileExtractIconW(strFileName.GetString(), dwAttributes, riid, ppvOut);
                }
            }
            else
            {
            }
        }
        else if IsEqualGUID!(&(*riid), &(IID_IDataObject))
        {
            if cidl >= 1
            {
                return _CDataObject_CreateInstance((*this_).m_Folder.Get(), cidl, apidl, riid, (ppvOut).cast());
            }
            else
            {
                let mut pDataObj: CComPtr<IDataObject> = CComPtr::new();
                let hr: HRESULT = OleGetClipboard(pDataObj.GetAddressOf());
                if FAILED_UNEXPECTEDLY!(hr)
                {
                    return E_FAIL;
                }
                *ppvOut = pDataObj.Detach() as LPVOID;
                return S_OK;
            }
        }
        else if IsEqualGUID!(&(*riid), &(IID_IDropTarget))
        {
            *ppvOut = core::ptr::addr_of_mut!((*this_).__base4) as LPVOID;
            (*this_).AddRef();
            return S_OK;
        }

        //ERR("%s(riid=%S) UNIMPLEMENTED\n", __FUNCTION__, g2s(riid));
        return E_NOTIMPL;
    }
    pub unsafe fn GetDisplayNameOf(&mut self, pidl: PCUITEMID_CHILD, dwFlags: DWORD, strRet: LPSTRRET) -> HRESULT {
        let this_: *mut Self = self;
        let _ = this_;
        if pidl.is_null() || strRet.is_null() {
            return E_INVALIDARG;
        }

        // Validate that this pidl is the last one
        let nextPidl = ILGetNext(pidl);
        if !nextPidl.is_null() && (*nextPidl).mkid.cb != 0 {
            return E_FAIL;
        }

        let fontEntry = _FontFromIL(pidl);
        if fontEntry.is_null() {
            return E_FAIL;
        }

        if (dwFlags & SHGDN_FORPARSING) != 0 {
            let mut fileName = CStringW::from_ptr((*fontEntry.cast_mut()).FileName__1());
            if (dwFlags & SHGDN_INFOLDER) == 0 {
                let mut fullPath = (*g_FontCache).GetFontFilePath((&fileName).GetString());
                if !fullPath.IsEmpty() {
                    return SHSetStrRet!(strRet, fullPath.GetString());
                }
            }
            return SHSetStrRet!(strRet, fileName.GetString());
        }

        return SHSetStrRet!(strRet, (*fontEntry.cast_mut()).Name__1());
    }
    pub unsafe fn SetNameOf(&mut self, hwndOwner: HWND,
        pidl: PCUITEMID_CHILD,
        lpName: LPCOLESTR,
        dwFlags: DWORD,
        pPidlOut: *mut PITEMID_CHILD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // ERR("%s() UNIMPLEMENTED\n", __FUNCTION__);
        E_NOTIMPL
    }
    pub unsafe fn GetCurFolder(&mut self, ppidl: *mut LPITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        if !ppidl.is_null() && !(*this_).m_Folder.is_null() {
            *ppidl = ILClone((*this_).m_Folder.as_ptr());
            return S_OK;
        }

        return E_POINTER;
    }
    pub unsafe fn Initialize(&mut self, pidl: LPCITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let mut PidlPath = [0u16; (MAX_PATH + 1) as _];
        let mut FontsDir = [0u16; (MAX_PATH + 1) as _];
        if SHGetPathFromIDListW(pidl, PidlPath.as_mut_ptr()) == 0 {
            return E_FAIL;
        }

        let hr: HRESULT = SHGetFolderPathW(core::ptr::null_mut(), (CSIDL_FONTS as i32), core::ptr::null_mut(), 0, FontsDir.as_mut_ptr());
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        if StrCmpIW(PidlPath.as_mut_ptr(), FontsDir.as_mut_ptr()) != 0 {
            return E_FAIL;
        }

        (*this_).m_Folder.Attach(ILCreateFromPathW(FontsDir.as_mut_ptr()));
        StringCchCatW(FontsDir.as_mut_ptr(), (_countof!(FontsDir)) as _, CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        (*g_FontCache).SetFontDir(FontsDir.as_mut_ptr());

        return S_OK;
    }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        let _ = this_;
        *lpClassId = CLSID_CFontExt;
        return S_OK;
    }
    pub unsafe fn DragEnter(&mut self, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_bDragAccepted = CheckDataObject(pDataObj);
        if (*this_).m_bDragAccepted == 0 {
            *pdwEffect = DROPEFFECT_NONE;
            return E_FAIL;
        }

        *pdwEffect &= DROPEFFECT_COPY;
        return S_OK;
    }
    pub unsafe fn DragOver(&mut self, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_bDragAccepted == 0 {
            *pdwEffect = DROPEFFECT_NONE;
            return E_FAIL;
        }
        *pdwEffect &= DROPEFFECT_COPY;
        return S_OK;
    }
    pub unsafe fn DragLeave(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn Drop(&mut self, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hwndView);
        let mut hr: HRESULT = InstallFontsFromDataObject((*this_).m_hwndView, pDataObj);

        let mut text = CStringW::new();
        let mut title = CStringW::new();
        title.LoadStringW(IDS_REACTOS_FONTS_FOLDER);
        if FAILED_UNEXPECTEDLY!(hr) {
            // Show error message
            text.LoadStringW(IDS_INSTALL_FAILED);
            MessageBoxW((*this_).m_hwndView, text.GetString(), title.GetString(), MB_ICONERROR);
        } else if hr == S_OK {
            // Refresh font cache and notify the system about the font change
            if !g_FontCache.is_null() {
                (*g_FontCache).Read();
            }

            PostMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, CPP_COM_WSTR_49342E15DCFE.as_ptr() as LPARAM);
            PostMessageW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0);

            // Show successful message
            text.LoadStringW(IDS_INSTALL_OK);
            MessageBoxW((*this_).m_hwndView, text.GetString(), title.GetString(), MB_ICONINFORMATION);
        }

        return hr;
    }
}

impl FolderViewColumns {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
