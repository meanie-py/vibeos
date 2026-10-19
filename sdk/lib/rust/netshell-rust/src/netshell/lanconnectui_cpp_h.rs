use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_285242FC4F38: [WCHAR; 94] = [0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x005C, 0x004E, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B, 0x005C, 0x007B, 0x0034, 0x0044, 0x0033, 0x0036, 0x0045, 0x0039, 0x0037, 0x0032, 0x002D, 0x0045, 0x0033, 0x0032, 0x0035, 0x002D, 0x0031, 0x0031, 0x0043, 0x0045, 0x002D, 0x0042, 0x0046, 0x0043, 0x0031, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0045, 0x0031, 0x0030, 0x0033, 0x0031, 0x0038, 0x007D, 0x005C, 0x0025, 0x0073, 0x005C, 0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_66C2A8D7C926: [WCHAR; 14] = [0x0050, 0x006E, 0x0070, 0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006E, 0x0063, 0x0065, 0x0049, 0x0064, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A402704ADB8A: [WCHAR; 47] = [0x0049, 0x004E, 0x0065, 0x0074, 0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0055, 0x0069, 0x005F, 0x0066, 0x006E, 0x0044, 0x0069, 0x0073, 0x0063, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0020, 0x0066, 0x006C, 0x0061, 0x0067, 0x0073, 0x0020, 0x0025, 0x0078, 0x000A, 0x0000];
pub static CPP_COM_WSTR_ADC29AD8CD96: [WCHAR; 61] = [0x0072, 0x0075, 0x006E, 0x0064, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065, 0x0078, 0x0065, 0x0020, 0x0064, 0x0065, 0x0076, 0x006D, 0x0067, 0x0072, 0x002E, 0x0064, 0x006C, 0x006C, 0x002C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0050, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073, 0x005F, 0x0052, 0x0075, 0x006E, 0x0044, 0x004C, 0x004C, 0x0020, 0x002F, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0049, 0x0044, 0x0020, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_B25B3C7B5934: [WCHAR; 9] = [0x0053, 0x0068, 0x006F, 0x0077, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_B6B73CAE481D: [WCHAR; 18] = [0x0049, 0x0070, 0x0043, 0x0068, 0x0065, 0x0063, 0x006B, 0x0069, 0x006E, 0x0067, 0x0045, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0064, 0x0000];

use core::ffi::c_void;















#[no_mangle]
pub unsafe extern "C" fn AddItemToListView(hDlgCtrl: HWND, pItem: PNET_ITEM, szName: LPWSTR, bChecked: BOOL, Image: UINT) {
    let mut lvItem: LVITEMW = core::mem::zeroed();

    lvItem.mask = LVIF_TEXT | LVIF_PARAM | LVIF_IMAGE;
    lvItem.pszText = szName;
    lvItem.lParam = pItem as LPARAM;
    lvItem.iImage = Image as i32;
    lvItem.iItem = 0x7fffffff; // Append at the end of the list.
    lvItem.iSubItem = 0;
    lvItem.iItem = SendMessageW(hDlgCtrl, LVM_INSERTITEMW, 0, core::ptr::addr_of_mut!(lvItem) as LPARAM) as i32;
    ListView_SetCheckState!(hDlgCtrl, lvItem.iItem, bChecked);
}

impl CNetConnectionPropertyUi {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_pProperties = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pNCfg.is_null() {
            let _ = (*(*this_).m_pNCfg.Get()).Uninitialize();
        }

        // Note: MSDN says we can only unlock after INetCfg::Uninitialize
        if !(*this_).m_NCfgLock.is_null() {
            let _ = (*(*this_).m_NCfgLock.Get()).ReleaseWriteLock();
        }

        if !(*this_).m_pProperties.is_null() {
            NcFreeNetconProperties((*this_).m_pProperties);
        }
    }
    pub unsafe fn AddPages(&mut self, hwndParent: HWND, pfnAddPage: LPFNADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        let mut hProp: HPROPSHEETPAGE;
        let mut ret: BOOL;
        let mut hr: HRESULT = E_FAIL;
        let mut initEx: INITCOMMONCONTROLSEX = core::mem::zeroed();

        initEx.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
        initEx.dwICC = ICC_LISTVIEW_CLASSES;
        if InitCommonControlsEx(core::ptr::addr_of_mut!(initEx)) == 0 {
            return E_FAIL;
        }

        hr = (*(*this_).m_pCon.Get()).GetProperties(core::ptr::addr_of_mut!((*this_).m_pProperties));
        if FAILED_UNEXPECTEDLY!(hr) != false {
            return hr;
        }

        hProp = InitializePropertySheetPage(
            MAKEINTRESOURCEW!(IDD_NETPROPERTIES),
            Some(CNetConnectionPropertyUi::LANPropertiesUIDlg),
            this_ as LPARAM,
            core::ptr::null_mut(),
        );
        if !hProp.is_null() {
            ret = (pfnAddPage.unwrap_unchecked())(hProp, lParam);
            if ret != 0 {
                hr = NOERROR;
            } else {
                DestroyPropertySheetPage(hProp);
            }
        }
        return hr;
    }
    pub unsafe fn GetIcon(&mut self, dwSize: DWORD,
        phIcon: *mut HICON,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetDeviceGuid(&mut self, pGuid: *mut GUID) -> HRESULT {
        let this_: *mut Self = self;
        CopyMemory!(pGuid as *mut core::ffi::c_void, core::ptr::addr_of!((*(*this_).m_pProperties).guidId) as *const core::ffi::c_void, core::mem::size_of::<GUID>());
        return S_OK;
    }
    pub unsafe fn SetConnection(&mut self, pCon: *mut INetConnection) -> HRESULT {
        let this_: *mut Self = self;
        if pCon.is_null() {
            return E_POINTER;
        }

        (*this_).m_pCon.assign(pCon);
        return S_OK;
    }
    pub unsafe fn Connect(&mut self, hwndParent: HWND, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pCon.is_null() {
            return E_POINTER; //FIXME
        }

        if dwFlags & NCUC_NO_UI != 0 {
            return (*(*this_).m_pCon.Get()).Connect();
        }

        return E_FAIL;
    }
    pub unsafe fn Disconnect(&mut self, hwndParent: HWND, dwFlags: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut szBuffer = [0u16; 100];
        _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_A402704ADB8A.as_ptr(), dwFlags);
        MessageBoxW(core::ptr::null_mut(), szBuffer.as_ptr(), core::ptr::null_mut(), MB_OK);

        return S_OK;
    }
    pub unsafe fn GetINetCfgComponent(&mut self, pNCfg: *mut INetCfg, pOut: *mut *mut INetCfgComponent) -> BOOL {
        let this_: *mut Self = self;
        let mut pName: LPWSTR = core::ptr::null_mut();
        let mut hr: HRESULT;
        let mut Fetched: ULONG = 0;
        let mut pEnumCfg: CComPtr<IEnumNetCfgComponent> = CComPtr::new();

        hr = (*pNCfg).EnumComponents(&GUID_DEVCLASS_NET, pEnumCfg.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) != false {
            return (FALSE) as BOOL;
        }

        loop { // while (TRUE)
            let mut pNCg: CComPtr<INetCfgComponent> = CComPtr::new();
            hr = (*pEnumCfg.Get()).Next(1, pNCg.GetAddressOf(), core::ptr::addr_of_mut!(Fetched));
            if hr != S_OK {
                break;
            }

            hr = (*pNCg.Get()).GetDisplayName(core::ptr::addr_of_mut!(pName));
            if hr >= 0 {
                if _wcsicmp(pName, (*(*this_).m_pProperties).pszwDeviceName) == 0 {
                    *pOut = pNCg.Detach();
                    return (TRUE) as BOOL;
                }
                CoTaskMemFree(pName as *mut core::ffi::c_void);
            }
        }
        return (FALSE) as BOOL;
    }
    pub unsafe fn EnumComponents(&mut self, hDlgCtrl: HWND, pNCfg: *mut INetCfg, CompGuid: *const GUID, Type: UINT, pCILD: PSP_CLASSIMAGELIST_DATA) {
        let this_: *mut Self = self;
        let mut hr: HRESULT;
        let mut pENetCfg: CComPtr<IEnumNetCfgComponent> = CComPtr::new();
        let mut Num: ULONG = 0;
        let mut dwCharacteristics: DWORD = 0;
        let mut pDisplayName: LPOLESTR = core::ptr::null_mut();
        let mut pHelpText: LPOLESTR = core::ptr::null_mut();
        let mut pItem: *mut NET_ITEM;
        let mut bChecked: BOOL;

        hr = (*pNCfg).EnumComponents(CompGuid, pENetCfg.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) != false {
            return;
        }

        while true {
            let mut pNCfgComp: CComPtr<INetCfgComponent> = CComPtr::new();
            let mut pCompBind: CComPtr<INetCfgComponentBindings> = CComPtr::new();
            let mut pAdapterCfgComp: CComPtr<INetCfgComponent> = CComPtr::new();

            hr = (*pENetCfg.Get()).Next(1, pNCfgComp.GetAddressOf(), core::ptr::addr_of_mut!(Num));
            if hr != S_OK {
                break;
            }

            hr = (*pNCfgComp.Get()).GetCharacteristics(core::ptr::addr_of_mut!(dwCharacteristics));
            if (hr >= 0) && (dwCharacteristics & NCF_HIDDEN) != 0 {
                continue;
            }

            pDisplayName = core::ptr::null_mut();
            pHelpText = core::ptr::null_mut();
            hr = (*pNCfgComp.Get()).GetDisplayName(core::ptr::addr_of_mut!(pDisplayName));
            hr = (*pNCfgComp.Get()).GetHelpText(core::ptr::addr_of_mut!(pHelpText));
            bChecked = (FALSE as i32);
            hr = (*pNCfgComp.Get()).QueryInterface(&IID_INetCfgComponentBindings, pCompBind.GetAddressOf() as *mut *mut core::ffi::c_void);
            if hr >= 0 {
                if (*this_).GetINetCfgComponent(pNCfg, pAdapterCfgComp.GetAddressOf()) != 0 {
                    hr = (*pCompBind.Get()).IsBoundTo(pAdapterCfgComp.Get());
                    if hr == S_OK {
                        bChecked = (TRUE as i32);
                    }
                }
            }

            pItem = CoTaskMemAlloc((core::mem::size_of::<NET_ITEM>()) as SIZE_T) as *mut NET_ITEM;
            if pItem.is_null() {
                continue;
            }

            (*pItem).dwCharacteristics = dwCharacteristics;
            (*pItem).szHelp = pHelpText;
            (*pItem).Type = Type as NET_TYPE;
            (*pItem).pNCfgComp = pNCfgComp.Detach();
            (*pItem).NumPropDialogOpen = 0;

            let mut image: INT = core::mem::zeroed();
            if (*pCILD).ImageList.is_null() || SetupDiGetClassImageIndex(pCILD, CompGuid, core::ptr::addr_of_mut!(image)) == 0 {
                image = I_IMAGENONE;
            }
            AddItemToListView(hDlgCtrl, pItem, pDisplayName, bChecked, image as UINT);
            CoTaskMemFree(pDisplayName as *mut core::ffi::c_void);
        }
    }
    pub unsafe fn InitializeLANPropertiesUIDlg(&mut self, hwndDlg: HWND) {
        let this_: *mut Self = self;
        let mut hr: HRESULT = 0;
        let mut pNCfg: CComPtr<INetCfg> = CComPtr::new();
        let mut pNCfgLock: CComPtr<INetCfgLock> = CComPtr::new();
        let hDlgCtrl: HWND = GetDlgItem(hwndDlg, (IDC_COMPONENTSLIST as i32));
        let mut lc: LVCOLUMNW = core::mem::zeroed();
        let mut rc: RECT = core::mem::zeroed();
        let mut dwStyle: DWORD;
        let mut pDisplayName: LPWSTR = core::ptr::null_mut();

        SendDlgItemMessageW(hwndDlg, (IDC_NETCARDNAME as i32), WM_SETTEXT, 0, (*(*this_).m_pProperties).pszwDeviceName as LPARAM);
        if ((*(*this_).m_pProperties).dwCharacter & NCCF_SHOW_ICON) != 0
        {
            /* check show item on taskbar*/
            SendDlgItemMessageW(hwndDlg, (IDC_SHOWTASKBAR as i32), BM_SETCHECK, (BST_CHECKED) as WPARAM, 0);
        }
        if ((*(*this_).m_pProperties).dwCharacter & NCCF_NOTIFY_DISCONNECTED) != 0
        {
            /* check notify item */
            SendDlgItemMessageW(hwndDlg, (IDC_NOTIFYNOCONNECTION as i32), BM_SETCHECK, (BST_CHECKED) as WPARAM, 0);
        }

        memset(core::ptr::addr_of_mut!(lc) as *mut core::ffi::c_void, 0, core::mem::size_of::<LV_COLUMN>());
        lc.mask = LVCF_TEXT | LVCF_SUBITEM | LVCF_FMT;
        lc.fmt = (LVCFMT_FIXED_WIDTH as i32);
        if GetClientRect(hDlgCtrl, core::ptr::addr_of_mut!(rc)) != 0
        {
            lc.mask |= LVCF_WIDTH;
            lc.cx = rc.right - rc.left;
        }
        lc.pszText = CPP_COM_WSTR_96A296D224F2.as_ptr() as LPWSTR;
        let _ = SendMessageW(hDlgCtrl, LVM_INSERTCOLUMNW, 0, core::ptr::addr_of_mut!(lc) as LPARAM);
        dwStyle = SendMessage(hDlgCtrl, LVM_GETEXTENDEDLISTVIEWSTYLE, 0, 0) as DWORD;
        dwStyle = dwStyle | LVS_EX_FULLROWSELECT | LVS_EX_CHECKBOXES;
        SendMessage(hDlgCtrl, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, dwStyle as LPARAM);

        hr = CoCreateInstance(core::ptr::addr_of!(CLSID_CNetCfg), core::ptr::null_mut(), CLSCTX_INPROC_SERVER, core::ptr::addr_of!(IID_INetCfg), pNCfg.GetAddressOf() as *mut *mut core::ffi::c_void);
        if hr < 0
        {
            return;
        }

        hr = (*pNCfg.Get()).QueryInterface(&IID_INetCfgLock, pNCfgLock.GetAddressOf() as *mut *mut core::ffi::c_void);
        hr = (*pNCfgLock.Get()).AcquireWriteLock(100, CPP_COM_WSTR_96A296D224F2.as_ptr() as LPCWSTR, core::ptr::addr_of_mut!(pDisplayName));
        if hr == S_FALSE
        {
            CoTaskMemFree(pDisplayName as *mut core::ffi::c_void);
            return;
        }

        hr = (*pNCfg.Get()).Initialize(core::ptr::null_mut());
        if FAILED_UNEXPECTEDLY!(hr)
        {
            (*pNCfgLock.Get()).ReleaseWriteLock();
            return;
        }

        (*this_).m_pNCfg.assign(pNCfg.Get());
        (*this_).m_NCfgLock.assign(pNCfgLock.Get());

        let mut spcid: SP_CLASSIMAGELIST_DATA = core::mem::zeroed();
        spcid.cbSize = core::mem::size_of::<SP_CLASSIMAGELIST_DATA>() as DWORD;
        if SetupDiGetClassImageList(core::ptr::addr_of_mut!(spcid)) != 0
        {
            let hIL: HIMAGELIST = ImageList_Duplicate(spcid.ImageList);
            ListView_SetImageList!(hDlgCtrl, hIL, LVSIL_SMALL);
        }
        else
        {
            spcid.ImageList = core::ptr::null_mut();
        }

        (*this_).EnumComponents(hDlgCtrl, pNCfg.Get(), core::ptr::addr_of!(GUID_DEVCLASS_NETCLIENT), NET_TYPE_CLIENT, core::ptr::addr_of_mut!(spcid));
        (*this_).EnumComponents(hDlgCtrl, pNCfg.Get(), core::ptr::addr_of!(GUID_DEVCLASS_NETSERVICE), NET_TYPE_SERVICE, core::ptr::addr_of_mut!(spcid));
        (*this_).EnumComponents(hDlgCtrl, pNCfg.Get(), core::ptr::addr_of!(GUID_DEVCLASS_NETTRANS), NET_TYPE_PROTOCOL, core::ptr::addr_of_mut!(spcid));

        if !spcid.ImageList.is_null()
        {
            SetupDiDestroyClassImageList(core::ptr::addr_of_mut!(spcid));
        }

        ListView_SetItemState!(hDlgCtrl, 0, -1, LVIS_FOCUSED | LVIS_SELECTED);
    }
    pub unsafe fn ShowNetworkComponentProperties(&mut self, hwndDlg: HWND) {
        let this_: *mut Self = self;
        let mut lvItem: LVITEMW = core::mem::zeroed();
        let mut hDlgCtrl: HWND;
        let mut Index: UINT;
        let mut Count: UINT;
        let mut hr: HRESULT;
        let mut pNCfgComp: *mut INetCfgComponent;
        let mut pItem: PNET_ITEM;

        hDlgCtrl = GetDlgItem(hwndDlg, IDC_COMPONENTSLIST as i32);
        Count = ListView_GetItemCount!(hDlgCtrl) as UINT;
        if Count == 0 {
            return;
        }

        ZeroMemory!(core::ptr::addr_of_mut!(lvItem) as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>() as usize);
        lvItem.mask = (LVIF_PARAM as UINT) | (LVIF_STATE as UINT);
        lvItem.stateMask = (1 as UINT).wrapping_neg();
        Index = 0 as UINT;
        while Index < Count {
            lvItem.iItem = Index as i32;
            if SendMessageW(hDlgCtrl, LVM_GETITEMW as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(lvItem) as LPARAM) != 0 {
                if (lvItem.state & (LVIS_SELECTED as UINT)) != 0 {
                    break;
                }
            }
            Index = Index.wrapping_add(1);
        }

        if (lvItem.state & (LVIS_SELECTED as UINT)) == 0 {
            return;
        }

        pItem = lvItem.lParam as PNET_ITEM;
        pNCfgComp = (*pItem).pNCfgComp as *mut INetCfgComponent;
        hr = (*pNCfgComp).RaisePropertyUi(GetParent(hwndDlg), NCRP_QUERY_PROPERTY_UI as DWORD, (core::ptr::addr_of_mut!((*this_).__base2) as *mut INetConnectionConnectUi).cast());
        if hr >= (0 as HRESULT) {
            hr = (*pNCfgComp).RaisePropertyUi(GetParent(hwndDlg), NCRP_SHOW_PROPERTY_UI as DWORD, (core::ptr::addr_of_mut!((*this_).__base2) as *mut INetConnectionConnectUi).cast());
        }
    }
    pub unsafe fn GetDeviceInstanceID(&mut self, DeviceInstanceID: *mut LPOLESTR) -> BOOL {
        let this_: *mut Self = self;
        let mut pStr: LPOLESTR = core::ptr::null_mut();
        let mut pResult: LPOLESTR;
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut dwInstanceID: DWORD;
        let mut szKeyName = [0u16; (2 * MAX_PATH) as _];
        let mut szInstanceID = [0u16; (2 * MAX_PATH) as _];

        if StringFromCLSID(core::ptr::addr_of!((*(*this_).m_pProperties).guidId), core::ptr::addr_of_mut!(pStr)) != (ERROR_SUCCESS as i32)
        {
            // failed to convert guid to string
            return (FALSE) as BOOL;
        }

        StringCbPrintfW!(szKeyName.as_mut_ptr(), core::mem::size_of::<[u16; (2 * MAX_PATH) as _]>(), CPP_COM_WSTR_285242FC4F38.as_ptr(), pStr);
        CoTaskMemFree(pStr as *mut core::ffi::c_void);

        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szKeyName.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey)) != (ERROR_SUCCESS as i32)
        {
            // failed to open key
            return (FALSE) as BOOL;
        }

        dwInstanceID = core::mem::size_of::<[u16; (2 * MAX_PATH) as _]>() as DWORD;
        if RegGetValueW(hKey, core::ptr::null(), CPP_COM_WSTR_66C2A8D7C926.as_ptr(), RRF_RT_REG_SZ, core::ptr::null_mut(), szInstanceID.as_mut_ptr() as PVOID, core::ptr::addr_of_mut!(dwInstanceID)) == (ERROR_SUCCESS as i32)
        {
            szInstanceID[(MAX_PATH - 1) as usize] = 0x0000u16;
            pResult = CoTaskMemAlloc(((wcslen(szInstanceID.as_ptr()) + 1 as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPOLESTR;
            if pResult != core::ptr::null_mut()
            {
                wcscpy(pResult, szInstanceID.as_ptr());
                *DeviceInstanceID = pResult;
                RegCloseKey(hKey);
                return (TRUE) as BOOL;
            }
        }
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }
    pub unsafe extern "system" fn LANPropertiesUIDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
        let mut page: *mut PROPSHEETPAGE = core::mem::zeroed();
        let mut lppl: LPNMLISTVIEW = core::mem::zeroed();
        let mut li: LVITEMW = core::mem::zeroed();
        let mut pItem: PNET_ITEM = core::mem::zeroed();
        let mut This: *mut CNetConnectionPropertyUi = core::mem::zeroed();
        let mut lppsn: LPPSHNOTIFY = core::mem::zeroed();
        let mut dwShowIcon: DWORD = core::mem::zeroed();
        let mut dwNotifyDisconnect: DWORD = core::mem::zeroed();
        let mut hr: HRESULT = core::mem::zeroed();
        let mut szKey: [WCHAR; 200] = core::mem::zeroed();
        let mut pStr: LPOLESTR = core::mem::zeroed();
        let mut hKey: HKEY = core::mem::zeroed();

        'sw: {
            match uMsg {
                WM_INITDIALOG => {
                    page = lParam as *mut PROPSHEETPAGE;
                    This = (*page).lParam as *mut CNetConnectionPropertyUi;
                    (*This).InitializeLANPropertiesUIDlg(hwndDlg);
                    SetWindowLongPtr(hwndDlg, DWLP_USER, (This as isize) as _);
                    return (TRUE) as INT_PTR;
                }
                WM_NOTIFY => {
                    lppl = lParam as LPNMLISTVIEW;
                    lppsn = lParam as LPPSHNOTIFY;
                    if (*lppsn).hdr.code == (PSN_APPLY) as u32 {
                        This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CNetConnectionPropertyUi;
                        if !(*This).m_pNCfg.is_null() {
                            hr = (*(*This).m_pNCfg.Get()).Apply();
                            if hr < 0 {
                                return (PSNRET_INVALID) as INT_PTR;
                            }
                        }

                        if SendDlgItemMessageW(hwndDlg, (IDC_SHOWTASKBAR as i32), BM_GETCHECK, 0, 0) == (BST_CHECKED as LRESULT) {
                            dwShowIcon = 1;
                        } else {
                            dwShowIcon = 0;
                        }

                        if SendDlgItemMessageW(hwndDlg, (IDC_NOTIFYNOCONNECTION as i32), BM_GETCHECK, 0, 0) == (BST_CHECKED as LRESULT) {
                            dwNotifyDisconnect = 1;
                        } else {
                            dwNotifyDisconnect = 0;
                        }

                        //NOTE: Windows write these setting with the undocumented INetLanConnection::SetInfo
                        if StringFromCLSID(&((*(*This).m_pProperties).guidId), core::ptr::addr_of_mut!(pStr)) == (ERROR_SUCCESS as i32) {
                            _swprintf(szKey.as_mut_ptr(), CPP_COM_WSTR_285242FC4F38.as_ptr(), pStr);
                            CoTaskMemFree((pStr).cast());
                            if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szKey.as_mut_ptr(), 0, KEY_WRITE, core::ptr::addr_of_mut!(hKey)) == (ERROR_SUCCESS as i32) {
                                RegSetValueExW(hKey, CPP_COM_WSTR_B25B3C7B5934.as_ptr(), 0, REG_DWORD, core::ptr::addr_of_mut!(dwShowIcon) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD);
                                RegSetValueExW(hKey, CPP_COM_WSTR_B6B73CAE481D.as_ptr(), 0, REG_DWORD, core::ptr::addr_of_mut!(dwNotifyDisconnect) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD);
                                RegCloseKey(hKey);
                            }
                        }
                        RefreshNetTaskbarNotificationIcon();

                        return (PSNRET_NOERROR) as INT_PTR;
                    }
                    if (*lppl).hdr.code == NM_DBLCLK {
                        This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CNetConnectionPropertyUi;
                        (*This).ShowNetworkComponentProperties(hwndDlg);
                        return (FALSE) as INT_PTR;
                    }

                    if (*lppl).hdr.code == (LVN_ITEMCHANGING) as u32 {
                        ZeroMemory!(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());
                        li.mask = LVIF_PARAM;
                        li.iItem = (*lppl).iItem;
                        if SendMessageW((*lppl).hdr.hwndFrom, LVM_GETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM) == 0 {
                            return (TRUE) as INT_PTR;
                        }

                        pItem = li.lParam as PNET_ITEM;
                        if pItem.is_null() {
                            return (TRUE) as INT_PTR;
                        }

                        if !(((*lppl).uOldState & LVIS_FOCUSED) != 0) && (((*lppl).uNewState & LVIS_FOCUSED) != 0) {
                            /* new focused item */
                            if ((*pItem).dwCharacteristics & NCF_NOT_USER_REMOVABLE) != 0 {
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_UNINSTALL as i32)), (FALSE) as BOOL);
                            } else {
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_UNINSTALL as i32)), (TRUE) as BOOL);
                            }

                            if ((*pItem).dwCharacteristics & NCF_HAS_UI) != 0 {
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_PROPERTIES as i32)), (TRUE) as BOOL);
                            } else {
                                EnableWindow(GetDlgItem(hwndDlg, (IDC_PROPERTIES as i32)), (FALSE) as BOOL);
                            }

                            SendDlgItemMessageW(hwndDlg, (IDC_DESCRIPTION as i32), WM_SETTEXT, 0, (*pItem).szHelp as LPARAM);
                        }
                    }
                    break 'sw;
                }
                WM_COMMAND => {
                    if LOWORD!(wParam) == (IDC_PROPERTIES as u16) {
                        This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CNetConnectionPropertyUi;
                        (*This).ShowNetworkComponentProperties(hwndDlg);
                        return (FALSE) as INT_PTR;
                    } else if LOWORD!(wParam) == (IDC_CONFIGURE as u16) {
                        let mut DeviceInstanceID: LPOLESTR = core::mem::zeroed();
                        This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CNetConnectionPropertyUi;

                        if (*This).GetDeviceInstanceID(core::ptr::addr_of_mut!(DeviceInstanceID)) != 0 {
                            let mut wszCmd: [WCHAR; (2 * MAX_PATH) as _] = core::mem::zeroed();
                            StringCbPrintfW!(wszCmd.as_mut_ptr(), core::mem::size_of::<[WCHAR; (2 * MAX_PATH) as _]>(), CPP_COM_WSTR_ADC29AD8CD96.as_ptr(), DeviceInstanceID);
                            CoTaskMemFree((DeviceInstanceID).cast());

                            let mut si: STARTUPINFOW = core::mem::zeroed();
                            let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
                            ZeroMemory!(core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void, core::mem::size_of::<STARTUPINFOW>());
                            si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
                            if CreateProcessW(core::ptr::null_mut(), wszCmd.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as BOOL, 0, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(si), core::ptr::addr_of_mut!(pi)) == 0 {
                                break 'sw;
                            }

                            CloseHandle(pi.hProcess);
                            CloseHandle(pi.hThread);
                        }
                    }
                    break 'sw;
                }
                _ => {}
            }
        }
        return (FALSE) as INT_PTR;
    }
}
