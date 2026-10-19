use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn strcmp(a: *const core::ffi::c_char, b: *const core::ffi::c_char) -> i32;
}

// ---- file-scope preamble (translated) ----
// IDS_SHV_COLUMN_* are resource IDs from the shell resource header; their values are not in the C preamble. Verify before building.

pub const SHCOLSTATE_TYPE_STR: u32 = 0x0000_0001; // shtypes.idl
pub const SHCOLSTATE_TYPE_DATE: u32 = 0x0000_0003;
pub const SHCOLSTATE_ONBYDEFAULT: u32 = 0x0000_0010;





















































#[no_mangle]
pub unsafe extern "system" fn _InsertMenuItemW(hmenu: HMENU, indexMenu: UINT, fByPosition: BOOL, wID: UINT, fType: UINT, dwTypeData: LPCWSTR, fState: UINT) {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut szText: [WCHAR; 100] = [0; 100];

    ZeroMemory!(core::ptr::addr_of_mut!(mii) as *mut core::ffi::c_void, core::mem::size_of::<MENUITEMINFOW>());
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    if fType == MFT_SEPARATOR {
        mii.fMask = MIIM_ID | MIIM_TYPE;
    } else if fType == MFT_STRING {
        mii.fMask = MIIM_ID | MIIM_TYPE | MIIM_STATE;
        if (HIWORD!(dwTypeData as ULONG_PTR) as ULONG_PTR) == 0 {
            let cch: usize = core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>();
            if LoadStringW(netshell_hInstance, LOWORD!(dwTypeData as ULONG_PTR) as UINT, szText.as_mut_ptr(), cch as i32) != 0 {
                szText[cch - 1 as usize] = 0 as WCHAR;
                mii.dwTypeData = szText.as_mut_ptr();
            } else {
                return;
            }
        } else {
            mii.dwTypeData = dwTypeData as LPWSTR;
        }
        mii.fState = fState;
    }

    mii.wID = wID;
    mii.fType = fType;
    InsertMenuItemW(hmenu, indexMenu, fByPosition, core::ptr::addr_of!(mii));
}
#[no_mangle]
pub unsafe extern "system" fn PropSheetExCallback(hPage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let pinfo: *mut PROPSHEETHEADERW = lParam as *mut PROPSHEETHEADERW;

    if (*pinfo).nPages < MAX_PROPERTY_SHEET_PAGE as UINT {
        let idx = (*pinfo).nPages;
        (*pinfo).nPages = idx + 1;
        *(*pinfo).__bindgen_anon_3.phpage.add(idx as usize) = hPage;
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ShowNetConnectionStatus(lpOleCmd: *mut IOleCommandTarget, pidl: PCUITEMID_CHILD, hwnd: HWND) -> HRESULT {
    if lpOleCmd.is_null() {
        return E_FAIL;
    }

    let pdata: PNETCONIDSTRUCT = ILGetConnData(pidl);
    if pdata.is_null() {
        return E_FAIL;
    }

    return (((*(*lpOleCmd).lpVtbl).Exec).unwrap_unchecked())(lpOleCmd, core::ptr::addr_of_mut!((*pdata).guidId), OLECMDID_NEW, OLECMDEXECOPT_DODEFAULT, core::ptr::null_mut(), core::ptr::null_mut());
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: u32, lParam: LPARAM) -> i32 {
    // NOTE: This callback is needed to set large icon correctly.
    let hIcon: HICON;
    if uMsg == PSCB_INITIALIZED {
        hIcon = LoadIconW(netshell_hInstance, MAKEINTRESOURCEW!(IDI_SHELL_NETWORK_FOLDER));
        SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIcon as LPARAM);
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn ShowNetConnectionProperties(pNetConnect: *mut INetConnection, hwnd: HWND) -> HRESULT {
    let mut hr: HRESULT;
    let mut ClassID: CLSID = core::mem::zeroed();
    let mut pinfo: PROPSHEETHEADERW = core::mem::zeroed();
    let mut hppages: [HPROPSHEETPAGE; (MAX_PROPERTY_SHEET_PAGE as _)] = core::mem::zeroed();
    let mut pNCP: CComPtr<INetConnectionPropertyUi> = CComPtr::new();
    let mut pProperties: *mut NETCON_PROPERTIES = core::ptr::null_mut();

    if (*pNetConnect).GetProperties(core::ptr::addr_of_mut!(pProperties)) != S_OK {
        return E_FAIL;
    }

    hr = (*pNetConnect).GetUiObjectClassId(core::ptr::addr_of_mut!(ClassID));
    if hr < 0 {
        NcFreeNetconProperties(pProperties);
        return hr;
    }

    hr = CoCreateInstance(
        core::ptr::addr_of!(ClassID),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_INetConnectionPropertyUi,
        pNCP.GetAddressOf() as *mut LPVOID,
    );
    if hr < 0 {
        NcFreeNetconProperties(pProperties);
        return hr;
    }

    hr = (*pNCP.Get()).SetConnection(pNetConnect);
    if hr >= 0 {
        ZeroMemory!(core::ptr::addr_of_mut!(pinfo) as LPVOID, core::mem::size_of::<PROPSHEETHEADERW>());
        ZeroMemory!(
            hppages.as_mut_ptr() as LPVOID,
            core::mem::size_of::<[HPROPSHEETPAGE; (MAX_PROPERTY_SHEET_PAGE as _)]>(),
        );
        pinfo.dwSize = (core::mem::size_of::<PROPSHEETHEADERW>()) as _;
        pinfo.dwFlags = PSH_NOCONTEXTHELP | PSH_PROPTITLE | PSH_NOAPPLYNOW |
                        PSH_USEICONID | PSH_USECALLBACK;
        pinfo.hInstance = netshell_hInstance;
        pinfo.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_SHELL_NETWORK_FOLDER);
        pinfo.__bindgen_anon_3.phpage = hppages.as_mut_ptr();
        pinfo.hwndParent = hwnd;
        pinfo.pfnCallback = Some(PropSheetProc);

        pinfo.pszCaption = (*pProperties).pszwName;
        hr = (*pNCP.Get()).AddPages(
            hwnd,
            Some(PropSheetExCallback),
            core::ptr::addr_of_mut!(pinfo) as LPARAM,
        );
        if hr >= 0 {
            if PropertySheetW(core::ptr::addr_of_mut!(pinfo)) < 0 {
                hr = E_FAIL;
            }
        }
    }
    NcFreeNetconProperties(pProperties);
    return hr;
}

impl CNetworkConnections {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_pidlRoot = core::ptr::null_mut();
        {
            let mut hr: HRESULT;
            hr = CoCreateInstance(&CLSID_ConnectionTray, core::ptr::null_mut(), CLSCTX_INPROC_SERVER, &IID_IOleCommandTarget, (*this_).m_lpOleCmd.GetAddressOf() as *mut *mut core::ffi::c_void);
            if FAILED_UNEXPECTEDLY!(hr) {
                (*this_).m_lpOleCmd = CComPtr::null();
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pidlRoot.is_null() {
            SHFree(((*this_).m_pidlRoot).cast());
        }
    }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        if lpClassId.is_null() {
            return E_POINTER;
        }

        *lpClassId = CLSID_NetworkConnections;

        return S_OK;
    }
    pub unsafe fn Initialize__0(&mut self, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pidlRoot.is_null() {
            SHFree((*this_).m_pidlRoot as LPVOID);
        }
        (*this_).m_pidlRoot = ILClone(pidl);

        return S_OK;
    }
    pub unsafe fn GetCurFolder(&mut self, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        if pidl.is_null() {
            return E_POINTER;
        }

        *pidl = ILClone((*this_).m_pidlRoot);

        return S_OK;
    }
    pub unsafe fn ParseDisplayName(&mut self, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut DWORD, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = E_UNEXPECTED;

        *ppidl = core::ptr::null_mut();
        if !pchEaten.is_null() {
            *pchEaten = 0; /* strange but like the original */
        }

        return hr;
    }
    pub unsafe fn EnumObjects(&mut self, hwndOwner: HWND, dwFlags: DWORD, ppEnumIDList: *mut LPENUMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        static IID_IEnumIDList: IID = IID { Data1: 0x000214F2, Data2: 0x0000, Data3: 0x0000, Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
        return CEnumIDList_CreateInstance(hwndOwner, dwFlags, &IID_IEnumIDList, ppEnumIDList as *mut *mut core::ffi::c_void);
    }
    pub unsafe fn BindToObject(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn BindToStorage(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        *ppvOut = core::ptr::null_mut();
        return E_NOTIMPL;
    }
    pub unsafe fn CompareIDs(&mut self, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
        let this_: *mut Self = self;
        const colcount: UINT = (NETCONNECTSHELLVIEWCOLUMNS as u32);

        if ILGetNext(pidl1) != core::ptr::null_mut() || ILGetNext(pidl2) != core::ptr::null_mut() {
            return E_NOTIMPL; // FIXME: Can the connection folder have subfolders?
        }

        if (lParam & (SHCIDS_CANONICALONLY as LPARAM)) != 0 {
            let p1: PNETCONIDSTRUCT = ILGetConnData(pidl1);
            let p2: PNETCONIDSTRUCT = ILGetConnData(pidl2);
            if !p1.is_null() && !p2.is_null() {
                let res: i32 = memcmp(
                    core::ptr::addr_of!((*p1).guidId) as *const GUID as *const core::ffi::c_void,
                    core::ptr::addr_of!((*p2).guidId) as *const GUID as *const core::ffi::c_void,
                    core::mem::size_of::<GUID>(),
                );
                return MAKE_COMPARE_HRESULT!(res);
            }
        }
        let psf: *mut IShellFolder2 = core::ptr::addr_of_mut!((*this_).__base4);
        return ShellFolderImpl_CompareItemIDs(
            psf,
            lParam,
            pidl1 as PCUITEMID_CHILD,
            pidl2 as PCUITEMID_CHILD,
            colcount,
            -1,
            true,
        );
    }
    pub unsafe fn CreateViewObject(&mut self, hwndOwner: HWND, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = E_NOINTERFACE;

        if ppvOut.is_null() {
            return hr;
        }

        *ppvOut = core::ptr::null_mut();

        if IsEqualIID!(riid, &IID_IShellView) != false {
            let mut cvf: CSFV = core::mem::zeroed();
            cvf.cbSize = core::mem::size_of::<CSFV>() as UINT;
            cvf.pshf = core::ptr::addr_of_mut!((*this_).__base4) as _; // the IShellFolder2 subobject, as C++'s implicit `this` conversion
            let mut pShellView: CComPtr<IShellView> = CComPtr::new();
            hr = SHCreateShellFolderViewEx(core::ptr::addr_of_mut!(cvf), pShellView.GetAddressOf());
            if FAILED_UNEXPECTEDLY!(hr) != false {
                return hr;
            }

            return (*pShellView.Get()).QueryInterface(riid, ppvOut);
        }

        return hr;
    }
    pub unsafe fn GetAttributesOf(&mut self, mut cidl: UINT, mut apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        //IGenericSFImpl *This = (IGenericSFImpl *)iface;
        // FIXME: Why are these reporting SFGAO_FILESYSTEM and SFGAO_FILESYSANCESTOR?
        let hr: HRESULT = S_OK;
        const dwNetConnectAttributes: DWORD = SFGAO_STORAGE | SFGAO_HASPROPSHEET | SFGAO_STORAGEANCESTOR |
            SFGAO_FILESYSANCESTOR | SFGAO_FOLDER | SFGAO_FILESYSTEM | SFGAO_HASSUBFOLDER | SFGAO_CANRENAME | SFGAO_CANDELETE;

        const dwNetConnectItemAttributes: DWORD = SFGAO_HASPROPSHEET | SFGAO_STORAGEANCESTOR |
            SFGAO_FILESYSANCESTOR | SFGAO_CANRENAME;

        if rgfInOut.is_null() {
            return E_INVALIDARG;
        }

        if cidl != 0 && apidl.is_null() {
            return E_INVALIDARG;
        }

        if *rgfInOut == 0 {
            *rgfInOut = !0;
        }

        if cidl > 1 {
            *rgfInOut &= !SFGAO_HASPROPSHEET;
        }

        if cidl == 0 {
            *rgfInOut = dwNetConnectAttributes;
        } else {
            *rgfInOut = dwNetConnectItemAttributes;

            while cidl > 0 && !(*apidl).is_null() {
                let pdata: PNETCONIDSTRUCT = ILGetConnData(*apidl);
                if pdata.is_null() {
                    continue;
                }

                if ((*pdata).dwCharacter & NCCF_ALLOW_RENAME) == 0 {
                    *rgfInOut &= !SFGAO_CANRENAME;
                }

                apidl = apidl.add(1);
                cidl -= 1;
            }
        }

        /* make sure SFGAO_VALIDATE is cleared, some apps depend on that */
        *rgfInOut &= !SFGAO_VALIDATE;

        return hr;
    }
    pub unsafe fn GetUIObjectOf(&mut self, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: &IID, prgfInOut: *mut UINT, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let mut pObj: *mut IUnknown = core::ptr::null_mut();
        let mut hr: HRESULT = E_INVALIDARG;

        if ppvOut.is_null() {
            return hr;
        }

        *ppvOut = core::ptr::null_mut();

        if (IsEqualIID!(riid, &IID_IContextMenu) != false || IsEqualIID!(riid, &IID_IContextMenu2) != false || IsEqualIID!(riid, &IID_IContextMenu3) != false ||
             IsEqualIID!(riid, &IID_IQueryInfo) != false || IsEqualIID!(riid, &IID_IExtractIconW) != false) && cidl >= 1
        {
            return ShellObjectCreatorInit!(CNetConUiObject, [*apidl.offset(0 as isize), (*this_).m_lpOleCmd.Get()], riid, ppvOut);
        }
        else
        {
            hr = E_NOINTERFACE;
        }

        *ppvOut = pObj as LPVOID;
        return hr;
    }
    pub unsafe fn GetDisplayNameOf(&mut self, pidl: PCUITEMID_CHILD, dwFlags: DWORD, strRet: LPSTRRET) -> HRESULT {
        let this_: *mut Self = self;
        if strRet.is_null() {
            return E_INVALIDARG;
        }

        if pidl.is_null() {
            return SHSetStrRet!(strRet, netshell_hInstance, IDS_NETWORKCONNECTION);
        }

        let pwchName: PWCHAR = ILGetConnName(pidl);
        if pwchName.is_null() {
            return E_INVALIDARG;
        }

        return SHSetStrRet!(strRet, pwchName);
    }
    pub unsafe fn SetNameOf(&mut self, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: DWORD, pPidlOut: *mut PITEMID_CHILD) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;
        let mut pCon: CComPtr<INetConnection> = CComPtr::new();

        hr = ILGetConnection(pidl, pCon.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        hr = (*pCon.Get()).Rename(lpName);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        if !pPidlOut.is_null() {
            *pPidlOut = ILCreateNetConnectItem(pCon.Get());
            if (*pPidlOut).is_null() {
                return E_FAIL;
            }
        }

        return S_OK;
    }
    pub unsafe fn GetDefaultSearchGUID(&mut self, pguid: *mut GUID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn EnumSearches(&mut self, ppenum: *mut *mut IEnumExtraSearch,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultColumn(&mut self, dwRes: DWORD,
        pSort: *mut ULONG,
        pDisplay: *mut ULONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultColumnState(&mut self, iColumn: UINT, pcsFlags: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if pcsFlags.is_null() || iColumn >= (NETCONNECTSHELLVIEWCOLUMNS as UINT) {
            return E_INVALIDARG;
        }
        *pcsFlags = (NetConnectSFHeader[iColumn as usize].pcsFlags) as u32;
        return S_OK;
    }
    pub unsafe fn GetDetailsEx(&mut self, pidl: PCUITEMID_CHILD,
        pscid: *const SHCOLUMNID,
        pv: *mut VARIANT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetDetailsOf(&mut self, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
        let this_: *mut Self = self;
        if iColumn >= (NETCONNECTSHELLVIEWCOLUMNS as UINT) {
            return E_FAIL;
        }

        (*psd).fmt = NetConnectSFHeader[iColumn as usize].fmt;
        (*psd).cxChar = NetConnectSFHeader[iColumn as usize].cxChar;
        if pidl.is_null() {
            return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), netshell_hInstance, NetConnectSFHeader[iColumn as usize].colnameid);
        }

        let pdata: PNETCONIDSTRUCT = ILGetConnData(pidl);
        if pdata.is_null() {
            return E_FAIL;
        }

        if iColumn == COLUMN_NAME as UINT {
            return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), ILGetConnName(pidl));
        } else if iColumn == COLUMN_TYPE as UINT {
            if (*pdata).MediaType == NCM_LAN || (*pdata).MediaType == NCM_SHAREDACCESSHOST_RAS {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), netshell_hInstance, IDS_TYPE_ETHERNET);
            } else {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), b"\0".as_ptr() as *const i8);
            }
        } else if iColumn == COLUMN_STATUS as UINT {
            if (*pdata).Status == NCS_HARDWARE_DISABLED {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), netshell_hInstance, IDS_STATUS_NON_OPERATIONAL);
            } else if (*pdata).Status == NCS_DISCONNECTED {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), netshell_hInstance, IDS_STATUS_UNREACHABLE);
            } else if (*pdata).Status == NCS_MEDIA_DISCONNECTED {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), netshell_hInstance, IDS_STATUS_DISCONNECTED);
            } else if (*pdata).Status == NCS_CONNECTING {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), netshell_hInstance, IDS_STATUS_CONNECTING);
            } else if (*pdata).Status == NCS_CONNECTED {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), netshell_hInstance, IDS_STATUS_CONNECTED);
            } else {
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), b"\0".as_ptr() as *const i8);
            }
        } else if iColumn == (COLUMN_DEVNAME as UINT) {
            return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), ILGetDeviceName(pidl));
        } else if iColumn == (COLUMN_PHONE as UINT) || iColumn == (COLUMN_OWNER as UINT) {
            return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), b"\0".as_ptr() as *const i8);
        }

        return E_FAIL;
    }
    pub unsafe fn MapColumnToSCID(&mut self, column: UINT,
        pscid: *mut SHCOLUMNID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn Initialize__1(&mut self, pidlFolder: PCIDLIST_ABSOLUTE,
        pdtobj: *mut IDataObject,
        hkeyProgID: HKEY,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("CNetworkConnections::Initialize()\n");
        E_NOTIMPL
    }
    pub unsafe fn Exec(&mut self, pguidCmdGroup: *const GUID,
        nCmdID: DWORD,
        nCmdexecopt: DWORD,
        pvaIn: *mut VARIANT,
        pvaOut: *mut VARIANT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("CNetworkConnections::Exec()\n");
        E_NOTIMPL
    }
    pub unsafe fn QueryStatus(&mut self, pguidCmdGroup: *const GUID,
        cCmds: ULONG,
        prgCmds: *mut OLECMD,
        pCmdText: *mut OLECMDTEXT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("CNetworkConnections::QueryStatus()\n");
        E_NOTIMPL
    }
    pub unsafe fn MessageSFVCB(&mut self, uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // FIXME("CNetworkConnections::MessageSFVCB()\n");
        E_NOTIMPL
    }
}

impl CNetConUiObject {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_pidl = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn Initialize(&mut self, pidl: PCUITEMID_CHILD, lpOleCmd: *mut IOleCommandTarget) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pidl = pidl;
        (*this_).m_lpOleCmd.assign(lpOleCmd);
        return S_OK;
    }
    pub unsafe fn QueryContextMenu(&mut self, hMenu: HMENU, mut indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
        let this_: *mut Self = self;
        let pdata: PNETCONIDSTRUCT = ILGetConnData((*this_).m_pidl);
        if pdata.is_null() {
            return E_FAIL;
        }

        if (*pdata).Status == NCS_HARDWARE_DISABLED || (*pdata).Status == NCS_MEDIA_DISCONNECTED || (*pdata).Status == NCS_DISCONNECTED {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_ACTIVATE), MFS_DEFAULT);
        } else {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 1, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_DEACTIVATE), MFS_ENABLED);
        }

        if (*pdata).Status == NCS_HARDWARE_DISABLED || (*pdata).Status == NCS_MEDIA_DISCONNECTED || (*pdata).Status == NCS_DISCONNECTED {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 2, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_STATUS), MFS_GRAYED);
        } else if (*pdata).Status == NCS_CONNECTED {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 2, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_STATUS), MFS_DEFAULT);
        } else {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 2, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_STATUS), MFS_ENABLED);
        }

        if (*pdata).Status == NCS_HARDWARE_DISABLED || (*pdata).Status == NCS_MEDIA_DISCONNECTED {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 3, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_REPAIR), MFS_GRAYED);
        } else {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 3, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_REPAIR), MFS_ENABLED);
        }

        _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, (1 as u32).wrapping_neg(), MFT_SEPARATOR, core::ptr::null(), MFS_ENABLED);
        _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 4, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_CREATELINK), MFS_ENABLED);

        if (*pdata).dwCharacter & NCCF_ALLOW_REMOVAL != 0 {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 5, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_DELETE), MFS_ENABLED);
        } else {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 5, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_DELETE), MFS_GRAYED);
        }

        if (*pdata).dwCharacter & NCCF_ALLOW_RENAME != 0 {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 6, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_RENAME), MFS_ENABLED);
        } else {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 6, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_RENAME), MFS_GRAYED);
        }

        _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, (1 as u32).wrapping_neg(), MFT_SEPARATOR, core::ptr::null(), MFS_ENABLED);
        if (*pdata).Status == NCS_CONNECTED {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 7, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_PROPERTIES), MFS_ENABLED);
        } else {
            _InsertMenuItemW(hMenu, { let _i = indexMenu; indexMenu += 1; _i }, (TRUE) as BOOL, idCmdFirst + 7, MFT_STRING, MAKEINTRESOURCEW!(IDS_NET_PROPERTIES), MFS_DEFAULT);
        }

        return MAKE_HRESULT!(SEVERITY_SUCCESS, 0, 9);
    }
    pub unsafe fn InvokeCommand(&mut self, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
        let this_: *mut Self = self;
        let mut CmdId: UINT = (LOWORD!(((*lpcmi).lpVerb as usize) as u32) as UINT) + IDS_NET_ACTIVATE;

        /* We should get this when F2 is pressed in explorer */
        if IS_INTRESOURCE!((*lpcmi).lpVerb) == false && strcmp((*lpcmi).lpVerb, b"rename\0".as_ptr() as *const i8) == 0
        {
            CmdId = IDS_NET_RENAME;
        }
        else if IS_INTRESOURCE!((*lpcmi).lpVerb) == false && strcmp((*lpcmi).lpVerb, b"properties\0".as_ptr() as *const i8) == 0
        {
            CmdId = IDS_NET_PROPERTIES;
        }
        else if IS_INTRESOURCE!((*lpcmi).lpVerb) == false || (LOWORD!(((*lpcmi).lpVerb as usize) as u32) as UINT) > 7
        {
            return E_NOTIMPL;
        }

        match CmdId {
            IDS_NET_RENAME => {
                let mut psv: CComPtr<IShellView> = CComPtr::new();
                let hr: HRESULT = IUnknown_QueryService(
                    (*this_).m_pUnknown.Get(),
                    &IID_IFolderView,
                    &IID_IShellView,
                    psv.GetAddressOf() as *mut *mut core::ffi::c_void,
                );
                if hr >= 0
                {
                    let selFlags: SVSIF = SVSI_DESELECTOTHERS | SVSI_EDIT | SVSI_ENSUREVISIBLE | SVSI_FOCUSED | SVSI_SELECT;
                    (*psv.Get()).SelectItem((*this_).m_pidl, selFlags);
                }

                return S_OK;
            }
            IDS_NET_STATUS => {
                // TODO: Windows does not display this with an owner window
                return ShowNetConnectionStatus((*this_).m_lpOleCmd.Get(), (*this_).m_pidl, (*lpcmi).hwnd);
            }
            IDS_NET_REPAIR | IDS_NET_CREATELINK | IDS_NET_DELETE => {
                FIXME!("Command %u is not implemented\n", CmdId);
                return E_NOTIMPL;
            }
            _ => {}
        }

        let hr: HRESULT;
        let mut pCon: CComPtr<INetConnection> = CComPtr::new();

        hr = ILGetConnection((*this_).m_pidl, pCon.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) != false {
            return hr;
        }

        match CmdId {
            IDS_NET_ACTIVATE => {
                return (*pCon.Get()).Connect();
            }
            IDS_NET_DEACTIVATE => {
                return (*pCon.Get()).Disconnect();
            }
            IDS_NET_PROPERTIES => {
                return ShowNetConnectionProperties(pCon.Get(), (*lpcmi).hwnd);
            }
            _ => {}
        }

        return E_NOTIMPL;
    }
    pub unsafe fn GetCommandString(&mut self, idCommand: UINT_PTR,
        uFlags: UINT,
        lpReserved: *mut UINT,
        lpszName: LPSTR,
        uMaxNameLen: UINT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_FAIL
    }
    pub unsafe fn HandleMenuMsg(&mut self, uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn HandleMenuMsg2(&mut self, uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        plResult: *mut LRESULT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn SetSite(&mut self, pUnkSite: *mut IUnknown) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pUnknown.assign(pUnkSite);
        return S_OK;
    }
    pub unsafe fn GetSite(&mut self, riid: &IID, ppvSite: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pUnknown.is_null() {
            *ppvSite = core::ptr::null_mut();
            return E_FAIL;
        }

        return (*(*this_).m_pUnknown.Get()).QueryInterface(riid, ppvSite);
    }
    pub unsafe fn GetInfoFlags(&mut self, pdwFlags: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        *pdwFlags = 0;

        return S_OK;
    }
    pub unsafe fn GetInfoTip(&mut self, dwFlags: DWORD, ppwszTip: *mut *mut WCHAR) -> HRESULT {
        let this_: *mut Self = self;
        *ppwszTip = core::ptr::null_mut();
        return S_OK;
    }
    pub unsafe fn GetIconLocation(&mut self, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut i32, pwFlags: *mut UINT) -> HRESULT {
        let this_: *mut Self = self;
        *pwFlags = 0;
        if GetModuleFileNameW(netshell_hInstance, szIconFile, cchMax) == 0 {
            return E_FAIL;
        }

        let pdata: PNETCONIDSTRUCT = ILGetConnData((*this_).m_pidl);
        if pdata.is_null() {
            return E_FAIL;
        }

        if (*pdata).Status == NCS_CONNECTED || (*pdata).Status == NCS_CONNECTING {
            *piIndex = (-(IDI_NET_IDLE as i32)) as i32;
        } else {
            *piIndex = (-(IDI_NET_OFF as i32)) as i32;
        }

        return S_OK;
    }
    pub unsafe fn Extract(&mut self, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
        let this_: *mut Self = self;
        return SHDefExtractIconW(pszFile, (nIconIndex) as i32, 0, phiconLarge, phiconSmall, nIconSize);
    }
}
