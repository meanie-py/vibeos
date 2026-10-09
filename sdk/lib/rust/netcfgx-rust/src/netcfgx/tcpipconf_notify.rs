use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct TcpFilterSettings {
    pub EnableSecurityFilters: DWORD,
    pub szTCPAllowedPorts: LPWSTR,
    pub szUDPAllowedPorts: LPWSTR,
    pub szRawIPAllowedProtocols: LPWSTR,
    pub IPSize: DWORD,
    pub TCPSize: DWORD,
    pub UDPSize: DWORD,
}

pub type PTcpFilterSettings = *mut TcpFilterSettings;

#[repr(C)]
pub struct TcpipAdvancedDNSDlgSettings {
    pub RegisterAdapterName: DWORD,
    pub RegistrationEnabled: DWORD,
    pub UseDomainNameDevolution: DWORD,
    pub szDomain: [WCHAR; 100],
    pub szSearchList: LPWSTR,
}

pub type PTcpipAdvancedDNSDlgSettings = *mut TcpipAdvancedDNSDlgSettings;

#[repr(C)]
pub struct tagIP_ADDR {
    pub IpAddress: DWORD,
    pub u: tagIP_ADDR_u,
    pub NTEContext: ULONG,
    pub Next: *mut tagIP_ADDR,
}

#[repr(C)]
pub union tagIP_ADDR_u {
    pub Subnetmask: DWORD,
    pub Metric: USHORT,
}

pub type IP_ADDR = tagIP_ADDR;
pub type PIP_ADDR = *mut IP_ADDR;

pub const METRIC: u32 = 1;
pub const SUBMASK: u32 = 2;
pub const IPADDR: u32 = 3;

pub type COPY_TYPE = u32;

#[repr(C)]
pub struct TcpipSettings {
    pub Ip: *mut IP_ADDR,
    pub Ns: *mut IP_ADDR,
    pub Gw: *mut IP_ADDR,
    pub DhcpEnabled: UINT,
    pub AutoconfigActive: UINT,
    pub Metric: DWORD,
    pub Index: DWORD,
    pub pFilter: *mut TcpFilterSettings,
    pub pDNS: *mut TcpipAdvancedDNSDlgSettings,
}

pub type PTcpipSettings = *mut TcpipSettings;

#[repr(C)]
pub struct TcpipConfNotifyImpl {
    pub lpVtbl: *const INetCfgComponentPropertyUi,
    pub lpVtblCompControl: *const INetCfgComponentControl,
    pub ref_: LONG,
    pub pUnknown: *mut IUnknown,
    pub pNCfg: *mut INetCfg,
    pub pNComp: *mut INetCfgComponent,
    pub pCurrentConfig: *mut TcpipSettings,
    pub NetCfgInstanceId: CLSID,
}

pub type LPTcpipConfNotifyImpl = *mut TcpipConfNotifyImpl;

#[repr(C)]
pub struct TcpipGwSettings {
    pub bAdd: BOOL,
    pub hDlgCtrl: HWND,
    pub szIP: [WCHAR; 16],
    pub Metric: UINT,
}

pub type PTcpipGwSettings = *mut TcpipGwSettings;

#[repr(C)]
pub struct TcpipIpSettings {
    pub bAdd: BOOL,
    pub hDlgCtrl: HWND,
    pub szIP: [WCHAR; 16],
    pub szMask: [WCHAR; 16],
}

pub type PTcpipIpSettings = *mut TcpipIpSettings;

#[repr(C)]
pub struct TcpipDnsSettings {
    pub bAdd: BOOL,
    pub hDlgCtrl: HWND,
    pub szIP: [WCHAR; 16],
}

pub type PTcpipDnsSettings = *mut TcpipDnsSettings;

#[repr(C)]
pub struct TcpipSuffixSettings {
    pub bAdd: BOOL,
    pub hDlgCtrl: HWND,
    pub Suffix: LPWSTR,
}

pub type PTcpipSuffixSettings = *mut TcpipSuffixSettings;

#[repr(C)]
pub struct TcpipPortSettings {
    pub hDlgCtrl: HWND,
    pub ResId: UINT,
    pub MaxNum: UINT,
}

pub type PTcpipPortSettings = *mut TcpipPortSettings;
unsafe extern "C" fn impl_from_INetCfgComponentControl(iface: *mut INetCfgComponentControl) -> LPTcpipConfNotifyImpl {
    (iface as *mut u8).offset(-(FIELD_OFFSET!(TcpipConfNotifyImpl, lpVtblCompControl) as isize)) as LPTcpipConfNotifyImpl
}
#[no_mangle]
pub unsafe extern "C" fn DisplayError(ResTxt: UINT, ResTitle: UINT, Type: UINT) {
    let mut szBuffer: [WCHAR; 300] = [0; 300];
    let mut szTitle: [WCHAR; 100] = [0; 100];

    if LoadStringW(
        netcfgx_hInstance,
        ResTxt,
        szBuffer.as_mut_ptr(),
        (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32,
    ) != 0
    {
        szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = '\0' as u16;
    } else {
        szBuffer[0] = '\0' as u16;
    }

    if LoadStringW(
        netcfgx_hInstance,
        ResTitle,
        szTitle.as_mut_ptr(),
        (core::mem::size_of_val(&szTitle) / core::mem::size_of::<WCHAR>()) as i32,
    ) != 0
    {
        szTitle[(core::mem::size_of_val(&szTitle) / core::mem::size_of::<WCHAR>()) - 1] = '\0' as u16;
    } else {
        szTitle[0] = '\0' as u16;
    }

    MessageBoxW(core::ptr::null_mut(), szBuffer.as_ptr(), szTitle.as_ptr(), Type);
}
#[no_mangle]
pub unsafe extern "system" fn TcpipFilterPortDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM
) -> INT_PTR {
    let mut pPort: *mut TcpipPortSettings;
    let mut Num: UINT;
    // Written by ZeroMemory below, exactly as the C does — but Rust wants them
    // definitely-initialised at the point the address is taken.
    let mut find: LVFINDINFOW = core::mem::zeroed();
    let mut li: LVITEMW = core::mem::zeroed();
    let mut szBuffer: [WCHAR; 100] = [0; 100];

    match uMsg {
        WM_INITDIALOG => {
            pPort = lParam as *mut TcpipPortSettings;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pPort as LONG_PTR);
            if LoadStringW(netcfgx_hInstance, (*pPort).ResId, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0
            {
                szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
                SendDlgItemMessageW(hwndDlg, IDC_PORT_DESC, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
            }

            if (*pPort).MaxNum == 65536 {
                SendDlgItemMessageW(hwndDlg, IDC_PORT_VAL, EM_LIMITTEXT, 5, 0);
            } else {
                SendDlgItemMessageW(hwndDlg, IDC_PORT_VAL, EM_LIMITTEXT, 3, 0);
            }

            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDCANCEL) as u16 {
                EndDialog(hwndDlg, FALSE as INT_PTR);
            } else if LOWORD!(wParam) == (IDC_OK) as u16 {
                pPort = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipPortSettings;
                Num = GetDlgItemInt(hwndDlg, IDC_PORT_VAL, core::ptr::null_mut(), (TRUE) as i32);
                if Num > (*pPort).MaxNum || Num == 0 {
                    if (*pPort).MaxNum == 65536 {
                        DisplayError((IDS_PORT_RANGE) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                    } else {
                        DisplayError((IDS_PROT_RANGE) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                    }

                    SetFocus(GetDlgItem(hwndDlg, IDC_PORT_VAL));
                } else {
                    if GetWindowTextW(GetDlgItem(hwndDlg, IDC_PORT_VAL), szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
                        szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
                        ZeroMemory!(core::ptr::addr_of_mut!(find) as LPVOID, core::mem::size_of::<LVFINDINFOW>() as u32);
                        find.flags = (LVFI_STRING) as u32;
                        find.psz = szBuffer.as_ptr();
                        if SendMessageW((*pPort).hDlgCtrl, (LVM_FINDITEMW) as u32, (1 as i32).wrapping_neg() as WPARAM, core::ptr::addr_of_mut!(find) as LPARAM) == ((1 as i32).wrapping_neg()) as LRESULT {
                            ZeroMemory!(core::ptr::addr_of_mut!(li) as LPVOID, core::mem::size_of::<LVITEMW>() as u32);
                            li.mask = (LVIF_TEXT) as u32;
                            li.iItem = ListView_GetItemCount!((*pPort).hDlgCtrl);
                            li.pszText = szBuffer.as_mut_ptr();
                            SendMessageW((*pPort).hDlgCtrl, (LVM_INSERTITEMW) as u32, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                            EndDialog(hwndDlg, TRUE as INT_PTR);
                        } else {
                            DisplayError((IDS_DUP_NUMBER) as u32, (IDS_PROT_RANGE) as u32, MB_ICONWARNING);
                            SetFocus(GetDlgItem(hwndDlg, IDC_PORT_VAL));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn InitFilterListBox(
    pData: LPWSTR,
    hwndDlg: HWND,
    hDlgCtrl: HWND,
    AllowButton: UINT,
    RestrictButton: UINT,
    AddButton: UINT,
    DelButton: UINT,
) {
    let mut li: LVITEMW = core::mem::zeroed();
    let mut pCur: LPWSTR;
    let mut iItem: INT;

    if pData.is_null() || _wtoi(pData) == 0 {
        CheckDlgButton(hwndDlg, (AllowButton) as i32, BST_CHECKED);
        EnableWindow(GetDlgItem(hwndDlg, (AddButton) as i32), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, (DelButton) as i32), (FALSE) as i32);
        return;
    }

    pCur = pData;
    memset(
        &mut li as *mut _ as *mut core::ffi::c_void,
        0x0,
        core::mem::size_of::<LVITEMW>(),
    );
    li.mask = (LVIF_TEXT) as u32;
    iItem = 0;

    while *pCur.offset(0) != 0 {
        li.pszText = pCur;
        li.iItem = iItem;
        SendMessageW(
            hDlgCtrl,
            (LVM_INSERTITEMW) as u32,
            0,
            &li as *const _ as LPARAM,
        );
        iItem += 1;
        pCur = pCur.offset(wcslen(pCur) as isize + 1);
    }

    if iItem == 0 {
        CheckDlgButton(hwndDlg, (AllowButton) as i32, BST_CHECKED);
    } else {
        CheckDlgButton(hwndDlg, (RestrictButton) as i32, BST_CHECKED);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreateFilterList(
    hDlgCtrl: HWND,
    Size: LPDWORD,
) -> LPWSTR {
    let mut iCount: i32;
    let mut iIndex: i32;
    let mut li: LVITEMW = core::mem::zeroed();
    let mut pData: LPWSTR;
    let mut pCur: LPWSTR;
    let mut dwSize: u32;
    let mut szBuffer: [u16; 10] = [0; 10];

    iCount = ListView_GetItemCount!(hDlgCtrl);
    if iCount == 0 {
        pData = CoTaskMemAlloc((3 * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if pData.is_null() {
            return core::ptr::null_mut();
        }
        *pData.offset(0) = '0' as u16;
        *pData.offset(1) = 0;
        *pData.offset(2) = 0;
        *Size = 3 * core::mem::size_of::<u16>() as u32;
        return pData;
    }

    pData = CoTaskMemAlloc(((6 * iCount + 1) as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
    if pData.is_null() {
        return core::ptr::null_mut();
    }

    pCur = pData;
    dwSize = 0;
    iIndex = 0;
    while iIndex < iCount {
        ZeroMemory!(&mut li as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());
        li.mask = (LVIF_TEXT) as u32;
        li.iItem = iIndex;
        li.pszText = szBuffer.as_mut_ptr();
        li.cchTextMax = (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32;
        if SendMessageW(hDlgCtrl, (LVM_GETITEMW) as u32, 0, &li as *const _ as LPARAM) != 0 {
            wcscpy(pCur, szBuffer.as_ptr());
            dwSize += wcslen(szBuffer.as_ptr()) as u32 + 1;
            pCur = pCur.offset(wcslen(szBuffer.as_ptr()) as isize + 1);
        }
        iIndex += 1;
    }
    *pCur.offset(0) = 0;
    *Size = (dwSize + 1) * core::mem::size_of::<u16>() as u32;
    pData
}
#[no_mangle]
pub unsafe extern "C" fn StoreTcpipFilterSettings(
    hwndDlg: HWND,
) -> *mut TcpFilterSettings {
    let pFilter: *mut TcpFilterSettings;

    pFilter = CoTaskMemAlloc((core::mem::size_of::<TcpFilterSettings>()) as SIZE_T) as *mut TcpFilterSettings;
    if pFilter.is_null() {
        return core::ptr::null_mut();
    }

    if IsDlgButtonChecked(hwndDlg, IDC_USE_FILTER) == BST_CHECKED as UINT {
        (*pFilter).EnableSecurityFilters = TRUE as DWORD;
    } else {
        (*pFilter).EnableSecurityFilters = FALSE as DWORD;
    }

    (*pFilter).szTCPAllowedPorts = CreateFilterList(GetDlgItem(hwndDlg, IDC_TCP_LIST), core::ptr::addr_of_mut!((*pFilter).TCPSize));
    (*pFilter).szUDPAllowedPorts = CreateFilterList(GetDlgItem(hwndDlg, IDC_UDP_LIST), core::ptr::addr_of_mut!((*pFilter).UDPSize));
    (*pFilter).szRawIPAllowedProtocols = CreateFilterList(GetDlgItem(hwndDlg, IDC_IP_LIST), core::ptr::addr_of_mut!((*pFilter).IPSize));

    pFilter
}
unsafe extern "C" fn AddItem(
    hwndDlg: HWND,
    hDlgCtrl: HWND,
    MaxItem: UINT,
    ResId: UINT,
) {
    let mut Port = TcpipPortSettings {
        MaxNum: MaxItem,
        hDlgCtrl: hDlgCtrl,
        ResId: ResId,
    };

    DialogBoxParamW(
        netcfgx_hInstance,
        MAKEINTRESOURCEW!(IDD_TCPIP_PORT_DLG),
        hwndDlg,
        Some(TcpipFilterPortDlg),
        core::ptr::addr_of_mut!(Port) as LPARAM,
    );
}
unsafe extern "C" fn DelItem(hDlgCtrl: HWND) {
    let iIndex = GetSelectedItem(hDlgCtrl);

    if iIndex != -1 {
        ListView_DeleteItem!(hDlgCtrl, iIndex);
        return;
    }
    DisplayError((IDS_NOITEMSEL) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
}
#[no_mangle]
pub unsafe extern "system" fn TcpipFilterSettingsDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pContext: *mut TcpipConfNotifyImpl;
    let mut pFilter: *mut TcpFilterSettings;

    match uMsg {
        WM_INITDIALOG => {
            pContext = lParam as *mut TcpipConfNotifyImpl;
            InsertColumnToListView(GetDlgItem(hwndDlg, IDC_TCP_LIST), (IDS_TCP_PORTS) as u32, 0, 100);
            InsertColumnToListView(GetDlgItem(hwndDlg, IDC_UDP_LIST), (IDS_UDP_PORTS) as u32, 0, 100);
            InsertColumnToListView(GetDlgItem(hwndDlg, IDC_IP_LIST), (IDS_IP_PROTO) as u32, 0, 100);
            if !(*pContext).pCurrentConfig.is_null() && !(*(*pContext).pCurrentConfig).pFilter.is_null() {
                InitFilterListBox((*(*(*pContext).pCurrentConfig).pFilter).szTCPAllowedPorts, hwndDlg, GetDlgItem(hwndDlg, IDC_TCP_LIST), (IDC_TCP_ALLOW_ALL) as u32, (IDC_TCP_RESTRICT) as u32, (IDC_TCP_ADD) as u32, (IDC_TCP_DEL) as u32);
                InitFilterListBox((*(*(*pContext).pCurrentConfig).pFilter).szUDPAllowedPorts, hwndDlg, GetDlgItem(hwndDlg, IDC_UDP_LIST), (IDC_UDP_ALLOW_ALL) as u32, (IDC_UDP_RESTRICT) as u32, (IDC_UDP_ADD) as u32, (IDC_UDP_DEL) as u32);
                InitFilterListBox((*(*(*pContext).pCurrentConfig).pFilter).szRawIPAllowedProtocols, hwndDlg, GetDlgItem(hwndDlg, IDC_IP_LIST), (IDC_IP_ALLOW_ALL) as u32, (IDC_IP_RESTRICT) as u32, (IDC_IP_ADD) as u32, (IDC_IP_DEL) as u32);
                if (*(*(*pContext).pCurrentConfig).pFilter).EnableSecurityFilters != 0 {
                    CheckDlgButton(hwndDlg, IDC_USE_FILTER, BST_CHECKED);
                }
            }
            SetWindowLongPtr(hwndDlg, DWLP_USER, pContext as LONG_PTR);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                match LOWORD!(wParam) as c_int {
                    IDC_TCP_ALLOW_ALL => {
                        if IsDlgButtonChecked(hwndDlg, IDC_TCP_ALLOW_ALL) == BST_CHECKED as UINT {
                            CheckDlgButton(hwndDlg, IDC_TCP_RESTRICT, BST_UNCHECKED);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_LIST), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_ADD), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_DEL), (FALSE) as i32);
                        }
                    }
                    IDC_TCP_RESTRICT => {
                        if IsDlgButtonChecked(hwndDlg, IDC_TCP_RESTRICT) == BST_CHECKED as UINT {
                            CheckDlgButton(hwndDlg, IDC_TCP_ALLOW_ALL, BST_UNCHECKED);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_LIST), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_ADD), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_DEL), (TRUE) as i32);
                        }
                    }
                    IDC_UDP_ALLOW_ALL => {
                        if IsDlgButtonChecked(hwndDlg, IDC_UDP_ALLOW_ALL) == BST_CHECKED as UINT {
                            CheckDlgButton(hwndDlg, IDC_UDP_RESTRICT, BST_UNCHECKED);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_LIST), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_ADD), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_DEL), (FALSE) as i32);
                        }
                    }
                    IDC_UDP_RESTRICT => {
                        if IsDlgButtonChecked(hwndDlg, IDC_UDP_RESTRICT) == BST_CHECKED as UINT {
                            CheckDlgButton(hwndDlg, IDC_UDP_ALLOW_ALL, BST_UNCHECKED);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_LIST), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_ADD), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_DEL), (TRUE) as i32);
                        }
                    }
                    IDC_IP_ALLOW_ALL => {
                        if IsDlgButtonChecked(hwndDlg, IDC_IP_ALLOW_ALL) == BST_CHECKED as UINT {
                            CheckDlgButton(hwndDlg, IDC_IP_RESTRICT, BST_UNCHECKED);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IP_LIST), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IP_ADD), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IP_DEL), (FALSE) as i32);
                        }
                    }
                    IDC_IP_RESTRICT => {
                        if IsDlgButtonChecked(hwndDlg, IDC_IP_RESTRICT) == BST_CHECKED as UINT {
                            CheckDlgButton(hwndDlg, IDC_IP_ALLOW_ALL, BST_UNCHECKED);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IP_LIST), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IP_ADD), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IP_DEL), (TRUE) as i32);
                        }
                    }
                    IDC_USE_FILTER => {
                        if IsDlgButtonChecked(hwndDlg, IDC_USE_FILTER) == BST_UNCHECKED as UINT {
                            DisplayError((IDS_DISABLE_FILTER) as u32, (IDS_TCPIP) as u32, MB_OK);
                        }
                    }
                    _ => {}
                }
            }
            match LOWORD!(wParam) as c_int {
                IDC_OK => {
                    pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl;
                    pFilter = StoreTcpipFilterSettings(hwndDlg);
                    if !pFilter.is_null() {
                        if !(*pContext).pCurrentConfig.is_null() && !(*(*pContext).pCurrentConfig).pFilter.is_null() {
                            CoTaskMemFree(((*(*(*pContext).pCurrentConfig).pFilter).szTCPAllowedPorts).cast());
                            CoTaskMemFree(((*(*(*pContext).pCurrentConfig).pFilter).szUDPAllowedPorts).cast());
                            CoTaskMemFree(((*(*(*pContext).pCurrentConfig).pFilter).szRawIPAllowedProtocols).cast());
                            CoTaskMemFree((*(*pContext).pCurrentConfig).pFilter as LPVOID);
                        }
                        (*(*pContext).pCurrentConfig).pFilter = pFilter;
                    }
                    EndDialog(hwndDlg, TRUE as INT_PTR);
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, FALSE as INT_PTR);
                }
                IDC_TCP_ADD => {
                    AddItem(hwndDlg, GetDlgItem(hwndDlg, IDC_TCP_LIST), 65536, (IDS_TCP_PORTS) as u32);
                }
                IDC_TCP_DEL => {
                    DelItem(GetDlgItem(hwndDlg, IDC_TCP_LIST));
                }
                IDC_UDP_ADD => {
                    AddItem(hwndDlg, GetDlgItem(hwndDlg, IDC_UDP_LIST), 65536, (IDS_UDP_PORTS) as u32);
                }
                IDC_UDP_DEL => {
                    DelItem(GetDlgItem(hwndDlg, IDC_UDP_LIST));
                }
                IDC_IP_ADD => {
                    AddItem(hwndDlg, GetDlgItem(hwndDlg, IDC_IP_LIST), 256, (IDS_IP_PROTO) as u32);
                }
                IDC_IP_DEL => {
                    DelItem(GetDlgItem(hwndDlg, IDC_IP_LIST));
                }
                _ => {}
            }
        }
        _ => {}
    }

    FALSE as INT_PTR
}
#[no_mangle]
pub unsafe extern "C" fn InitializePropertySheetPage(
    resname: LPWSTR,
    dlgproc: DLGPROC,
    lParam: LPARAM,
    szTitle: LPWSTR,
) -> HPROPSHEETPAGE {
    let mut ppage: PROPSHEETPAGEW = core::mem::zeroed();

    memset(
        core::ptr::addr_of_mut!(ppage) as *mut core::ffi::c_void,
        0x0,
        core::mem::size_of::<PROPSHEETPAGEW>(),
    );
    ppage.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
    ppage.dwFlags = (PSP_DEFAULT) as u32;
    *ppage.u.pszTemplate.as_mut() = resname;
    ppage.pfnDlgProc = dlgproc;
    ppage.lParam = lParam;
    ppage.hInstance = netcfgx_hInstance;
    if !szTitle.is_null() {
        ppage.dwFlags |= (PSP_USETITLE) as u32;
        ppage.pszTitle = szTitle;
    }
    CreatePropertySheetPageW(core::ptr::addr_of_mut!(ppage))
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTcpipAdvancedOptDlg(
    hwndDlg: HWND,
    This: *mut TcpipConfNotifyImpl) {
    let mut szText: [u16; 500] = [0; 500];

    SetWindowLongPtr(hwndDlg, DWLP_USER, This as INT_PTR);

    if LoadStringW(netcfgx_hInstance, (IDS_TCPFILTER) as u32, szText.as_mut_ptr(), (core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) as i32) != 0 {
        szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) - 1] = 0;
        if SendDlgItemMessageW(hwndDlg, IDC_OPTLIST, LB_ADDSTRING, 0, szText.as_ptr() as LPARAM) != (LB_ERR) as LRESULT {
            SendDlgItemMessageW(hwndDlg, IDC_OPTLIST, LB_SETCURSEL, 0, 0);
        }
    }

    if LoadStringW(netcfgx_hInstance, (IDS_TCPFILTERDESC) as u32, szText.as_mut_ptr(), (core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) as i32) != 0 {
        szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) - 1] = 0;
        SendDlgItemMessageW(hwndDlg, IDC_OPTDESC, WM_SETTEXT, 0, szText.as_ptr() as LPARAM);
    }
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAdvancedOptDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM
) -> INT_PTR {
    let This: *mut TcpipConfNotifyImpl;
    let page: LPPROPSHEETPAGE;

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as LPPROPSHEETPAGE;
            This = (*page).lParam as *mut TcpipConfNotifyImpl;
            InitializeTcpipAdvancedOptDlg(hwndDlg, This);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_OPTPROP) as u16 {
                DialogBoxParamW(netcfgx_hInstance,
                                MAKEINTRESOURCEW!(IDD_TCPIP_FILTER_DLG),
                                GetParent(hwndDlg),
                                Some(TcpipFilterSettingsDlg),
                                GetWindowLongPtr(hwndDlg, DWLP_USER) as LPARAM);
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn InsertColumnToListView(
    hDlgCtrl: HWND,
    ResId: UINT,
    SubItem: UINT,
    Size: UINT,
) {
    let mut szBuffer: [WCHAR; 200] = [0; 200];
    let mut lc: LVCOLUMNW = core::mem::zeroed();

    if LoadStringW(
        netcfgx_hInstance,
        ResId,
        szBuffer.as_mut_ptr(),
        (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32,
    ) == 0
    {
        return;
    }

    memset(
        &mut lc as *mut _ as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<LV_COLUMN>(),
    );
    lc.mask = (LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_FMT) as u32;
    lc.iSubItem = SubItem as i32;
    lc.fmt = LVCFMT_FIXED_WIDTH;
    lc.cx = Size as i32;
    lc.cchTextMax = (wcslen(szBuffer.as_ptr())) as i32;
    lc.pszText = szBuffer.as_mut_ptr();

    SendMessageW(hDlgCtrl, (LVM_INSERTCOLUMNW) as u32, SubItem as WPARAM, &lc as *const _ as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn InsertIpAddressToListView(
    hDlgCtrl: HWND,
    mut pAddr: *mut IP_ADDR,
    bSubMask: BOOL,
) {
    let mut szBuffer: [u16; 70] = [0; 70];
    let mut dwIpAddr: u32 = 0;
    let mut itemCount: u32 = 0;
    let mut li: LVITEMW = core::mem::zeroed();

    while !pAddr.is_null() {
        // C: ZeroMemory(&li, sizeof(li)). The translation gave it memset's THREE arguments
        // (dest, 0, len); ZeroMemory takes two and supplies the zero itself.
        ZeroMemory!(
            &mut li as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<LVITEMW>(),
        );
        li.mask = (LVIF_TEXT) as u32;
        li.iItem = itemCount as i32;
        li.iSubItem = 0;
        dwIpAddr = (*pAddr).IpAddress;
        swprintf(
            szBuffer.as_mut_ptr(),
            w!("%lu.%lu.%lu.%lu") as *const u16,
            FIRST_IPADDRESS!(dwIpAddr),
            SECOND_IPADDRESS!(dwIpAddr),
            THIRD_IPADDRESS!(dwIpAddr),
            FOURTH_IPADDRESS!(dwIpAddr),
        );

        li.pszText = szBuffer.as_mut_ptr();
        li.iItem = (SendMessageW(
            hDlgCtrl,
            (LVM_INSERTITEMW) as u32,
            0,
            &li as *const _ as LPARAM,
        )) as i32;
        if li.iItem != -1 {
            if bSubMask != 0 {
                dwIpAddr = (*pAddr).u.Subnetmask;
                swprintf(
                    szBuffer.as_mut_ptr(),
                    w!("%lu.%lu.%lu.%lu") as *const u16,
                    FIRST_IPADDRESS!(dwIpAddr),
                    SECOND_IPADDRESS!(dwIpAddr),
                    THIRD_IPADDRESS!(dwIpAddr),
                    FOURTH_IPADDRESS!(dwIpAddr),
                );
            } else {
                if (*pAddr).u.Metric != 0 {
                    swprintf(
                        szBuffer.as_mut_ptr(),
                        w!("%u") as *const u16,
                        (*pAddr).u.Metric as c_uint,
                    );
                } else {
                    LoadStringW(
                        netcfgx_hInstance,
                        (IDS_AUTOMATIC) as u32,
                        szBuffer.as_mut_ptr(),
                        (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32,
                    );
                }
            }

            li.mask = (LVIF_TEXT) as u32;
            li.iSubItem = 1;
            li.pszText = szBuffer.as_mut_ptr();
            SendMessageW(
                hDlgCtrl,
                (LVM_SETITEMW) as u32,
                0,
                &li as *const _ as LPARAM,
            );
        }
        itemCount += 1;
        pAddr = (*pAddr).Next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTcpipAdvancedIpDlg(
    hwndDlg: HWND,
    This: *mut TcpipConfNotifyImpl,
) {
    let mut rect: RECT = core::mem::zeroed();
    let mut li: LVITEMW = core::mem::zeroed();
    let mut szBuffer: [WCHAR; 100] = [0; 100];

    InsertColumnToListView(GetDlgItem(hwndDlg, IDC_IPLIST), (IDS_IPADDR) as u32, 0, 100);
    GetClientRect(GetDlgItem(hwndDlg, IDC_IPLIST), &mut rect);
    InsertColumnToListView(GetDlgItem(hwndDlg, IDC_IPLIST), (IDS_SUBMASK) as u32, 1, ((rect.right - rect.left - 100)) as u32);

    if (*This).pCurrentConfig.is_null() == false && (*(*This).pCurrentConfig).DhcpEnabled != 0 {
        if LoadStringW(netcfgx_hInstance, (IDS_DHCPACTIVE) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
            ZeroMemory!(&mut li as *mut _ as LPVOID, core::mem::size_of::<LVITEMW>() as u32);
            li.mask = (LVIF_TEXT) as u32;
            li.pszText = szBuffer.as_mut_ptr();
            SendDlgItemMessageW(hwndDlg, IDC_IPLIST, (LVM_INSERTITEMW) as u32, 0, &li as *const _ as LPARAM);
        }
        EnableWindow(GetDlgItem(hwndDlg, IDC_IPADD), (FALSE) as i32);
    } else {
        InsertIpAddressToListView(GetDlgItem(hwndDlg, IDC_IPLIST), (*(*This).pCurrentConfig).Ip, (TRUE) as i32);
    }

    EnableWindow(GetDlgItem(hwndDlg, IDC_IPMOD), (FALSE) as i32);
    EnableWindow(GetDlgItem(hwndDlg, IDC_IPDEL), (FALSE) as i32);

    InsertColumnToListView(GetDlgItem(hwndDlg, IDC_GWLIST), (IDS_GATEWAY) as u32, 0, 100);
    GetClientRect(GetDlgItem(hwndDlg, IDC_IPLIST), &mut rect);
    InsertColumnToListView(GetDlgItem(hwndDlg, IDC_GWLIST), (IDS_METRIC) as u32, 1, ((rect.right - rect.left - 100)) as u32);

    InsertIpAddressToListView(GetDlgItem(hwndDlg, IDC_GWLIST), (*(*This).pCurrentConfig).Gw, (FALSE) as i32);

    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMOD), (FALSE) as i32);
    EnableWindow(GetDlgItem(hwndDlg, IDC_GWDEL), (FALSE) as i32);

    SendDlgItemMessageW(hwndDlg, IDC_IFMETRIC, EM_LIMITTEXT, 4, 0);
    if (*(*This).pCurrentConfig).Metric != 0 {
        EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), (TRUE) as i32);
        SetDlgItemInt(hwndDlg, IDC_IFMETRIC, (*(*This).pCurrentConfig).Metric, (FALSE) as i32);
    } else {
        CheckDlgButton(hwndDlg, IDC_IFAUTOMETRIC, BST_CHECKED);
        EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), (FALSE) as i32);
    }
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAdvGwDlg(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut szBuffer: [u16; 70] = [0; 70];
    let mut pGwSettings: *mut TcpipGwSettings;
    let mut dwIpAddr: u32 = 0;
    let mut lpnmipa: *mut core::ffi::c_void;
    let mut find: core::mem::MaybeUninit<[u8; core::mem::size_of::<usize>() * 4]> = core::mem::MaybeUninit::uninit();

    match uMsg {
        WM_INITDIALOG => {
            pGwSettings = lParam as *mut TcpipGwSettings;
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam as LONG_PTR);

            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

            if (*pGwSettings).bAdd != 0 {
                if LoadStringW(netcfgx_hInstance, (IDS_ADD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }
                EnableWindow(GetDlgItem(hwndDlg, IDC_OK), (FALSE) as i32);
                CheckDlgButton(hwndDlg, IDC_GWAUTOMETRIC, BST_CHECKED);
            } else {
                if LoadStringW(netcfgx_hInstance, (IDS_MOD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }

                SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETADDRESS) as u32, 0, GetIpAddressFromStringW((*pGwSettings).szIP.as_mut_ptr()) as LPARAM);

                if (*pGwSettings).Metric != 0 {
                    SetDlgItemInt(hwndDlg, IDC_GWMETRIC, (*pGwSettings).Metric, (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), (TRUE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), (TRUE) as i32);
                } else {
                    CheckDlgButton(hwndDlg, IDC_GWAUTOMETRIC, BST_CHECKED);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), (FALSE) as i32);
                }
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_GWAUTOMETRIC) as u16 {
                if IsDlgButtonChecked(hwndDlg, IDC_GWAUTOMETRIC) == BST_CHECKED as UINT {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), (FALSE) as i32);
                    SendDlgItemMessageW(hwndDlg, IDC_GWMETRIC, WM_SETTEXT, 0, w!("") as LPARAM);
                } else {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), (TRUE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), (TRUE) as i32);
                }
            } else if LOWORD!(wParam) == (IDCANCEL) as u16 {
                EndDialog(hwndDlg, FALSE as INT_PTR);
            } else if LOWORD!(wParam) == (IDC_OK) as u16 {
                if SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_GETADDRESS) as u32, 0, core::ptr::addr_of_mut!(dwIpAddr) as LPARAM) == 4 {
                    pGwSettings = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipGwSettings;
                    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_GETTEXT, 16, (*pGwSettings).szIP.as_mut_ptr() as LPARAM);

                    ZeroMemory!(find.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<LVFINDINFOW>());
                    let find_ptr = find.as_mut_ptr() as *mut usize;
                    *find_ptr.offset(0) = (LVFI_STRING) as usize;
                    *find_ptr.offset(1) = (*pGwSettings).szIP.as_ptr() as usize;

                    if IsDlgButtonChecked(hwndDlg, IDC_GWAUTOMETRIC) == BST_UNCHECKED as UINT {
                        (*pGwSettings).Metric = GetDlgItemInt(hwndDlg, IDC_GWMETRIC, core::ptr::null_mut(), (FALSE) as i32);
                    } else {
                        (*pGwSettings).Metric = 0;
                    }

                    if SendMessageW((*pGwSettings).hDlgCtrl, (LVM_FINDITEMW) as u32, (1 as isize).wrapping_neg() as WPARAM, find.as_ptr() as LPARAM) == (1 as LRESULT).wrapping_neg() {
                        EndDialog(hwndDlg, TRUE as INT_PTR);
                    } else if (*pGwSettings).bAdd == 0 { // C `!pGwSettings->bAdd`; a bitwise not was always true
                        EndDialog(hwndDlg, FALSE as INT_PTR);
                    } else {
                        DisplayError((IDS_DUP_GW) as u32, (IDS_TCPIP) as u32, MB_ICONINFORMATION);
                    }
                }
            }
        }
        WM_NOTIFY => {
            lpnmipa = lParam as *mut core::ffi::c_void;
            let code = *(lpnmipa as *const u32).offset(2);
            let idFrom = *(lpnmipa as *const usize).offset(1);
            if code == (IPN_FIELDCHANGED) as u32 {
                if idFrom == (IDC_IPADDR) as usize {
                    if SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_GETADDRESS) as u32, 0, core::ptr::addr_of_mut!(dwIpAddr) as LPARAM) == 4 {
                        EnableWindow(GetDlgItem(hwndDlg, IDC_OK), (TRUE) as i32);
                    }
                }
            }
        }
        _ => {}
    }
    FALSE as INT_PTR
}
#[no_mangle]
pub unsafe extern "C" fn GetGWListEntry(hDlgCtrl: HWND, Index: i32, pGwSettings: *mut TcpipGwSettings) -> i32 {
    let mut li: LVITEMW = core::mem::zeroed();
    let mut szBuffer: [u16; 30] = [0; 30];
    let bRet: i32;

    ZeroMemory!(&mut li as *mut _ as LPVOID, core::mem::size_of::<LVITEMW>());
    li.mask = (LVIF_TEXT) as u32;
    li.cchTextMax = 16;
    li.pszText = (*pGwSettings).szIP.as_mut_ptr();
    li.iItem = Index;

    if SendMessageW(hDlgCtrl, (LVM_GETITEMW) as u32, 0, &li as *const _ as LPARAM) == 0 {
        return (FALSE) as i32;
    }
    li.pszText = szBuffer.as_mut_ptr();
    li.cchTextMax = 30;
    li.iSubItem = 1;

    bRet = (SendMessageW(hDlgCtrl, (LVM_GETITEMW) as u32, 0, &li as *const _ as LPARAM)) as i32;
    if bRet != 0 {
        (*pGwSettings).Metric = (_wtoi(szBuffer.as_ptr())) as u32;
    }

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAddIpDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut lpnmipa: LPNMIPADDRESS;
    let mut dwIpAddr: DWORD = 0;
    let mut pIpSettings: *mut TcpipIpSettings;
    let mut szBuffer: [u16; 50] = [0; 50];
    let mut find: LVFINDINFOW = core::mem::zeroed();
    let mut lResult: LRESULT;

    match uMsg {
        WM_INITDIALOG => {
            pIpSettings = lParam as *mut TcpipIpSettings;
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam as LONG_PTR);

            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

            if (*pIpSettings).bAdd != 0 {
                if LoadStringW(netcfgx_hInstance, (IDS_ADD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }
                EnableWindow(GetDlgItem(hwndDlg, IDC_OK), (FALSE) as i32);
            } else {
                if LoadStringW(netcfgx_hInstance, (IDS_MOD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }

                SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETADDRESS) as u32, 0, GetIpAddressFromStringW((*pIpSettings).szIP.as_mut_ptr()) as LPARAM);
                SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, GetIpAddressFromStringW((*pIpSettings).szMask.as_mut_ptr()) as LPARAM);
            }
            return TRUE as INT_PTR;
        }
        WM_NOTIFY => {
            lpnmipa = lParam as LPNMIPADDRESS;
            if (*lpnmipa).hdr.code == (IPN_FIELDCHANGED) as u32 {
                if (*lpnmipa).hdr.idFrom == IDC_IPADDR as UINT_PTR {
                    if SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_GETADDRESS) as u32, 0, core::ptr::addr_of_mut!(dwIpAddr) as LPARAM) == 4 {
                        if dwIpAddr <= MAKEIPADDRESS!(127, 255, 255, 255) {
                            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, MAKEIPADDRESS!(255, 0, 0, 0) as LPARAM);
                        } else if dwIpAddr <= MAKEIPADDRESS!(191, 255, 255, 255) {
                            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, MAKEIPADDRESS!(255, 255, 0, 0) as LPARAM);
                        } else if dwIpAddr <= MAKEIPADDRESS!(223, 255, 255, 255) {
                            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, MAKEIPADDRESS!(255, 255, 255, 0) as LPARAM);
                        }
                        EnableWindow(GetDlgItem(hwndDlg, IDC_OK), (TRUE) as i32);
                    }
                }
            }
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_OK) as u16 {
                pIpSettings = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipIpSettings;
                SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_GETTEXT, 16, (*pIpSettings).szIP.as_mut_ptr() as LPARAM);
                SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, WM_GETTEXT, 16, (*pIpSettings).szMask.as_mut_ptr() as LPARAM);

                ZeroMemory!(core::ptr::addr_of_mut!(find) as LPVOID, core::mem::size_of::<LVFINDINFOW>());
                find.flags = (LVFI_STRING) as u32;
                find.psz = (*pIpSettings).szIP.as_mut_ptr();
                lResult = SendMessageW((*pIpSettings).hDlgCtrl, (LVM_FINDITEMW) as u32, (-1i32) as isize as WPARAM, core::ptr::addr_of_mut!(find) as LPARAM);

                if lResult == -1 {
                    EndDialog(hwndDlg, TRUE as INT_PTR);
                    return 0;
                } else if (*pIpSettings).bAdd == 0 {
                    EndDialog(hwndDlg, FALSE as INT_PTR);
                    return 0;
                }
                DisplayError((IDS_DUP_IPADDR) as u32, (IDS_TCPIP) as u32, MB_ICONINFORMATION);
                return 0;
            } else if LOWORD!(wParam) == (IDCANCEL) as u16 {
                EndDialog(hwndDlg, FALSE as INT_PTR);
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn VerifyDNSSuffix(
    szName: LPWSTR
) -> i32 {
    let mut Index: u32;
    let Length: u32 = (wcslen(szName)) as u32;

    Index = 0;
    while Index < Length {
        if iswalnum((*szName.offset(Index as isize) as u16 as i32) as u16) == 0 && *szName.offset(Index as isize) != '.' as u16 && *szName.offset(Index as isize) != '-' as u16 {
            return (FALSE) as i32;
        }
        Index = Index + 1;
    }

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAddSuffixDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM
) -> INT_PTR {
    let mut szBuffer: [WCHAR; 100] = [0; 100];
    let mut pSettings: *mut TcpipSuffixSettings;
    let mut lLength: LRESULT;

    match uMsg {
        WM_INITDIALOG => {
            pSettings = lParam as *mut TcpipSuffixSettings;
            if (*pSettings).bAdd == 0 {
                SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_SETTEXT, 0, (*pSettings).Suffix as LPARAM);
                if LoadStringW(netcfgx_hInstance, (IDS_MOD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }
                CoTaskMemFree((*pSettings).Suffix as LPVOID);
                (*pSettings).Suffix = core::ptr::null_mut();
            } else {
                if LoadStringW(netcfgx_hInstance, (IDS_ADD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }
            }
            SetWindowLongPtr(hwndDlg, DWLP_USER, pSettings as LONG_PTR);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDCANCEL) as u16 {
                EndDialog(hwndDlg, FALSE as INT_PTR);
            } else if LOWORD!(wParam) == (IDC_OK) as u16 {
                lLength = SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_GETTEXTLENGTH, 0, 0);
                if lLength != 0 {
                    pSettings = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipSuffixSettings;
                    (*pSettings).Suffix = CoTaskMemAlloc(((lLength + 1) as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
                    if !(*pSettings).Suffix.is_null() {
                        SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_GETTEXT, (lLength + 1) as WPARAM, (*pSettings).Suffix as LPARAM);
                        if SendMessageW((*pSettings).hDlgCtrl, LB_FINDSTRING, 0, (*pSettings).Suffix as LPARAM) != (LB_ERR) as LRESULT {
                            DisplayError((IDS_DUP_SUFFIX) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                            CoTaskMemFree((*pSettings).Suffix as LPVOID);
                            return FALSE as INT_PTR;
                        }

                        if VerifyDNSSuffix((*pSettings).Suffix) == 0 {
                            DisplayError((IDS_DOMAIN_SUFFIX) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                            CoTaskMemFree((*pSettings).Suffix as LPVOID);
                            return FALSE as INT_PTR;
                        }
                        EndDialog(hwndDlg, TRUE as INT_PTR);
                    }
                }
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn GetSelectedItem(hDlgCtrl: HWND) -> i32 {
    let mut li: LVITEMW = core::mem::zeroed();
    let iItemCount: c_int;
    let mut iIndex: c_int;

    iItemCount = ListView_GetItemCount!(hDlgCtrl);
    if iItemCount == 0 {
        return -1;
    }

    iIndex = 0;
    while iIndex < iItemCount {
        ZeroMemory!(&mut li as *mut _ as LPVOID, core::mem::size_of::<LVITEMW>());
        li.mask = (LVIF_STATE) as u32;
        li.stateMask = (-1 as i32) as u32;
        li.iItem = iIndex as i32;
        if SendMessageW(hDlgCtrl, (LVM_GETITEMW) as u32, 0, &li as *const _ as LPARAM) != 0 {
            if (li.state & (LVIS_SELECTED) as u32) != 0 {
                return iIndex as i32;
            }
        }
        iIndex += 1;
    }
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn GetIPListEntry(hDlgCtrl: HWND, Index: i32, pIpSettings: *mut TcpipIpSettings) -> i32 {
    let mut li: LVITEMW = core::mem::zeroed();

    ZeroMemory!(&mut li as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());
    li.mask = (LVIF_TEXT) as u32;
    li.cchTextMax = 16;
    li.pszText = (*pIpSettings).szIP.as_mut_ptr();
    li.iItem = Index;

    if SendMessageW(hDlgCtrl, (LVM_GETITEMW) as u32, 0, &li as *const _ as LPARAM) == 0 {
        return (FALSE) as i32;
    }

    ZeroMemory!(&mut li as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());
    li.mask = (LVIF_TEXT) as u32;
    li.cchTextMax = 16;
    li.pszText = (*pIpSettings).szMask.as_mut_ptr();
    li.iSubItem = 1;
    li.iItem = Index;

    (SendMessageW(hDlgCtrl, (LVM_GETITEMW) as u32, 0, &li as *const _ as LPARAM)) as i32
}
#[no_mangle]
pub unsafe extern "C" fn DeleteItemFromList(hDlgCtrl: HWND) {
    let mut li: LVITEMW = core::mem::zeroed();

    memset(
        &mut li as *mut _ as *mut core::ffi::c_void,
        0x0,
        core::mem::size_of::<LVITEMW>(),
    );
    li.iItem = GetSelectedItem(hDlgCtrl);
    if li.iItem < 0 {
        DisplayError((IDS_NOITEMSEL) as u32, (IDS_TCPIP) as u32, MB_ICONINFORMATION);
        SetFocus(hDlgCtrl);
    } else {
        ListView_DeleteItem!(hDlgCtrl, li.iItem);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetIpAddressFromStringW(
    szBuffer: *mut u16,
) -> u32 {
    let mut dwIpAddr: u32 = 0;
    let mut Val: i32;
    let mut Index: u32 = 3;
    let mut pLast: *mut u16 = szBuffer;
    let mut pNext: *mut u16 = szBuffer;

    loop {
        pNext = wcschr(pNext, '.' as u16);
        if pNext.is_null() {
            break;
        }
        *pNext.offset(0) = 0u16;
        Val = _wtoi(pLast);
        dwIpAddr |= (Val as u32) << (Index * 8);
        Index = Index.wrapping_sub(1);
        pNext = pNext.offset(1);
        pLast = pNext;
    }
    dwIpAddr |= _wtoi(pLast) as u32;

    dwIpAddr
}
#[no_mangle]
pub unsafe extern "C" fn GetIpAddressFromStringA(
    sBuffer: *mut i8,
) -> u32 {
    let mut szIp: [u16; 16] = core::mem::zeroed();

    if MultiByteToWideChar((CP_ACP) as u32, 0, sBuffer, -1, szIp.as_mut_ptr(), 16) != 0 {
        szIp[15] = 0u16;
        return GetIpAddressFromStringW(szIp.as_mut_ptr());
    }
    return (-1i32) as u32;
}
#[no_mangle]
pub unsafe extern "C" fn FreeIPAddr(mut pAddr: *mut IP_ADDR) {
    let mut pNext: *mut IP_ADDR;

    if pAddr.is_null() {
        return;
    }

    while !pAddr.is_null() {
        pNext = (*pAddr).Next;
        CoTaskMemFree(pAddr as LPVOID);
        pAddr = pNext;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetListViewItem(
    hDlgCtrl: HWND,
    Index: UINT,
    SubIndex: UINT,
    szBuffer: *mut WCHAR,
    BufferSize: UINT,
) -> BOOL {
    let mut li: LVITEMW = core::mem::zeroed();

    ZeroMemory!(&mut li as *mut _ as PVOID, core::mem::size_of::<LVITEMW>());
    li.mask = (LVIF_TEXT) as u32;
    li.pszText = szBuffer;
    li.iItem = Index as i32;
    li.iSubItem = SubIndex as i32;
    li.cchTextMax = BufferSize as i32;
    SendMessageW(hDlgCtrl, (LVM_GETITEMW) as u32, 0, &li as *const _ as LPARAM) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn StoreIPSettings(
    hDlgCtrl: HWND,
    This: *mut TcpipConfNotifyImpl,
    bSubmask: BOOL,
) {
    let mut szBuffer: [WCHAR; 30] = [0; 30];

    let mut iIndex: INT;
    let iCount: INT;
    let mut pCur: *mut IP_ADDR;
    let mut pLast: *mut IP_ADDR;

    iCount = ListView_GetItemCount!(hDlgCtrl);
    if iCount == 0 {
        return;
    }

    pLast = core::ptr::null_mut();
    iIndex = 0;
    while iIndex < iCount {
        if GetListViewItem(
            hDlgCtrl,
            iIndex as UINT,
            0,
            szBuffer.as_mut_ptr(),
            (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as UINT,
        ) != 0
        {
            pCur = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
            if pCur.is_null() {
                break;
            }
            ZeroMemory!(pCur as LPVOID, core::mem::size_of::<IP_ADDR>());

            szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
            (*pCur).IpAddress = GetIpAddressFromStringW(szBuffer.as_mut_ptr());

            if GetListViewItem(
                hDlgCtrl,
                iIndex as UINT,
                1,
                szBuffer.as_mut_ptr(),
                (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as UINT,
            ) != 0
            {
                szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
                if bSubmask != 0 {
                    (*pCur).u.Subnetmask = GetIpAddressFromStringW(szBuffer.as_mut_ptr());
                } else {
                    (*pCur).u.Metric = _wtoi(szBuffer.as_mut_ptr()) as USHORT;
                }
            }

            if pLast.is_null() {
                if bSubmask != 0 {
                    (*(*This).pCurrentConfig).Ip = pCur;
                } else {
                    (*(*This).pCurrentConfig).Gw = pCur;
                }
            } else {
                (*pLast).Next = pCur;
            }

            pLast = pCur;
        }
        iIndex += 1;
    }
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAdvancedIpDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut This: *mut TcpipConfNotifyImpl;
    let mut page: LPPROPSHEETPAGE;
    let mut res: INT_PTR;
    let mut szBuffer: [WCHAR; 200] = [0; 200];
    let mut lppsn: LPPSHNOTIFY;
    let mut Gw: TcpipGwSettings = core::mem::zeroed();
    let mut Ip: TcpipIpSettings = core::mem::zeroed();

    let mut li: LVITEMW = core::mem::zeroed();

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as LPPROPSHEETPAGE;
            This = (*page).lParam as *mut TcpipConfNotifyImpl;
            InitializeTcpipAdvancedIpDlg(hwndDlg, This);
            SetWindowLongPtr(hwndDlg, DWLP_USER, This as INT_PTR);
            return TRUE as INT_PTR;
        }
        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == (LVN_ITEMCHANGED) as u32 {
                let lplv = lParam as LPNMLISTVIEW;
                let mut bEnable: BOOL;

                if (*lplv).hdr.idFrom == IDC_IPLIST as UINT_PTR {
                    This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl;

                    bEnable = ((((*lplv).uNewState & (LVIS_SELECTED) as u32) != 0)
                        && ((*(*This).pCurrentConfig).DhcpEnabled == 0)) as BOOL;

                    EnableWindow(GetDlgItem(hwndDlg, IDC_IPMOD), bEnable);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_IPDEL), bEnable);
                } else if (*lplv).hdr.idFrom == IDC_GWLIST as UINT_PTR {
                    bEnable = ((((*lplv).uNewState & (LVIS_SELECTED) as u32) != 0) as BOOL);

                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWMOD), bEnable);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_GWDEL), bEnable);
                }
            } else if (*lppsn).hdr.code == (PSN_KILLACTIVE) as u32 {
                This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl;
                if (*(*This).pCurrentConfig).DhcpEnabled == 0 && ListView_GetItemCount!(GetDlgItem(hwndDlg, IDC_IPLIST)) == 0i32 {
                    DisplayError((IDS_NO_IPADDR_SET) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
                    return TRUE as INT_PTR;
                }

                if IsDlgButtonChecked(hwndDlg, IDC_IFAUTOMETRIC) != BST_CHECKED {
                    let val = GetDlgItemInt(hwndDlg, IDC_IFMETRIC, core::ptr::null_mut(), (FALSE) as i32);
                    if (val < 1) || (val > 9999) {
                        DisplayError((IDS_METRIC_RANGE) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
                        return TRUE as INT_PTR;
                    }
                }
            } else if (*lppsn).hdr.code == (PSN_APPLY) as u32 {
                This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl;
                FreeIPAddr((*(*This).pCurrentConfig).Gw);
                (*(*This).pCurrentConfig).Gw = core::ptr::null_mut();
                FreeIPAddr((*(*This).pCurrentConfig).Ip);
                (*(*This).pCurrentConfig).Ip = core::ptr::null_mut();
                StoreIPSettings(GetDlgItem(hwndDlg, IDC_IPLIST), This, (TRUE) as i32);
                StoreIPSettings(GetDlgItem(hwndDlg, IDC_GWLIST), This, (FALSE) as i32);

                if IsDlgButtonChecked(hwndDlg, IDC_IFAUTOMETRIC) == BST_CHECKED as UINT {
                    (*(*This).pCurrentConfig).Metric = 0;
                } else {
                    (*(*This).pCurrentConfig).Metric = GetDlgItemInt(hwndDlg, IDC_IFMETRIC, core::ptr::null_mut(), (FALSE) as i32);
                }

                SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (PSNRET_NOERROR) as LRESULT);
                return TRUE as INT_PTR;
            }
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_IFAUTOMETRIC) as u16 {
                if IsDlgButtonChecked(hwndDlg, IDC_IFAUTOMETRIC) == BST_CHECKED as UINT {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), (FALSE) as i32);
                } else {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), (TRUE) as i32);
                }
            } else if LOWORD!(wParam) == (IDC_IPADD) as u16 {
                Ip.bAdd = (TRUE) as i32;
                Ip.hDlgCtrl = GetDlgItem(hwndDlg, IDC_IPLIST);
                res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPADDIP_DLG), hwndDlg, Some(TcpipAddIpDlg), &mut Ip as *mut _ as LPARAM);
                if res != 0 {
                    memset(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, 0x0, core::mem::size_of::<LVITEMW>());
                    li.mask = (LVIF_TEXT | LVIF_PARAM) as u32;
                    li.iItem = ListView_GetItemCount!(GetDlgItem(hwndDlg, IDC_IPLIST));
                    li.pszText = Ip.szIP.as_mut_ptr();
                    li.iItem = (SendDlgItemMessageW(hwndDlg, IDC_IPLIST, (LVM_INSERTITEMW) as u32, 0, &li as *const _ as LPARAM)) as i32;
                    if li.iItem != -1 {
                        li.mask = (LVIF_TEXT) as u32;
                        li.iSubItem = 1;
                        li.pszText = Ip.szMask.as_mut_ptr();
                        SendDlgItemMessageW(hwndDlg, IDC_IPLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                    }
                }
            } else if LOWORD!(wParam) == (IDC_IPMOD) as u16 {
                memset(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, 0x0, core::mem::size_of::<LVITEMW>());
                li.iItem = GetSelectedItem(GetDlgItem(hwndDlg, IDC_IPLIST));
                if li.iItem < 0 {
                    DisplayError((IDS_NOITEMSEL) as u32, (IDS_TCPIP) as u32, MB_ICONINFORMATION);
                    SetFocus(GetDlgItem(hwndDlg, IDC_IPLIST));
                } else {
                    Ip.bAdd = (FALSE) as i32;
                    Ip.hDlgCtrl = GetDlgItem(hwndDlg, IDC_IPLIST);
                    if GetIPListEntry(GetDlgItem(hwndDlg, IDC_IPLIST), li.iItem, &mut Ip) != 0 {
                        res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPADDIP_DLG), hwndDlg, Some(TcpipAddIpDlg), &mut Ip as *mut _ as LPARAM);
                        if res != 0 {
                            li.mask = (LVIF_TEXT) as u32;
                            li.pszText = Ip.szIP.as_mut_ptr();
                            SendDlgItemMessageW(hwndDlg, IDC_IPLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                            li.pszText = Ip.szMask.as_mut_ptr();
                            li.iSubItem = 1;
                            SendDlgItemMessageW(hwndDlg, IDC_IPLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                        }
                    }
                }
            } else if LOWORD!(wParam) == (IDC_IPDEL) as u16 {
                DeleteItemFromList(GetDlgItem(hwndDlg, IDC_IPLIST));
            } else if LOWORD!(wParam) == (IDC_GWADD) as u16 {
                Gw.bAdd = (TRUE) as i32;
                Gw.hDlgCtrl = GetDlgItem(hwndDlg, IDC_GWLIST);
                res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPGW_DLG), hwndDlg, Some(TcpipAdvGwDlg), &mut Gw as *mut _ as LPARAM);
                if res != 0 {
                    memset(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, 0x0, core::mem::size_of::<LVITEMW>());
                    li.mask = (LVIF_TEXT) as u32;
                    li.iItem = ListView_GetItemCount!(GetDlgItem(hwndDlg, IDC_GWLIST));
                    li.pszText = Gw.szIP.as_mut_ptr();
                    li.iItem = (SendDlgItemMessageW(hwndDlg, IDC_GWLIST, (LVM_INSERTITEMW) as u32, 0, &li as *const _ as LPARAM)) as i32;
                    if li.iItem >= 0 {
                        if Gw.Metric != 0 {
                            swprintf(szBuffer.as_mut_ptr(), w!("%u") as *const _, Gw.Metric);
                            li.iSubItem = 1;
                            li.pszText = szBuffer.as_mut_ptr();
                            SendDlgItemMessageW(hwndDlg, IDC_GWLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                        } else {
                            if LoadStringW(netcfgx_hInstance, (IDS_AUTOMATIC) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
                                szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = '\0' as u16;
                                li.iSubItem = 1;
                                li.pszText = szBuffer.as_mut_ptr();
                                SendDlgItemMessageW(hwndDlg, IDC_GWLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                            }
                        }
                    }
                }
            } else if LOWORD!(wParam) == (IDC_GWMOD) as u16 {
                memset(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, 0x0, core::mem::size_of::<LVITEMW>());
                li.iItem = GetSelectedItem(GetDlgItem(hwndDlg, IDC_GWLIST));
                if li.iItem < 0 {
                    DisplayError((IDS_NOITEMSEL) as u32, (IDS_TCPIP) as u32, MB_ICONINFORMATION);
                    SetFocus(GetDlgItem(hwndDlg, IDC_GWLIST));
                } else {
                    if GetGWListEntry(GetDlgItem(hwndDlg, IDC_GWLIST), li.iItem, &mut Gw) != 0 {
                        Gw.bAdd = (FALSE) as i32;
                        Gw.hDlgCtrl = GetDlgItem(hwndDlg, IDC_GWLIST);
                        res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPGW_DLG), hwndDlg, Some(TcpipAdvGwDlg), &mut Gw as *mut _ as LPARAM);
                        if res != 0 {
                            li.mask = (LVIF_TEXT) as u32;
                            li.pszText = Gw.szIP.as_mut_ptr();
                            SendDlgItemMessageW(hwndDlg, IDC_GWLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                            if Gw.Metric != 0 {
                                swprintf(szBuffer.as_mut_ptr(), w!("%u") as *const _, Gw.Metric);
                                li.iSubItem = 1;
                                li.pszText = szBuffer.as_mut_ptr();
                                SendDlgItemMessageW(hwndDlg, IDC_GWLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                            } else {
                                if LoadStringW(netcfgx_hInstance, (IDS_AUTOMATIC) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
                                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = '\0' as u16;
                                    li.iSubItem = 1;
                                    li.pszText = szBuffer.as_mut_ptr();
                                    SendDlgItemMessageW(hwndDlg, IDC_GWLIST, (LVM_SETITEMW) as u32, 0, &li as *const _ as LPARAM);
                                }
                            }
                        }
                    }
                }
            } else if LOWORD!(wParam) == (IDC_GWDEL) as u16 {
                DeleteItemFromList(GetDlgItem(hwndDlg, IDC_GWLIST));
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAddDNSDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM
) -> INT_PTR {
    let mut pSettings: *mut TcpipDnsSettings;
    let mut szBuffer: [WCHAR; 100] = [0; 100];
    let mut dwIpAddr: DWORD = 0;
    let mut lpnmipa: LPNMIPADDRESS;

    match uMsg {
        WM_INITDIALOG => {
            pSettings = lParam as *mut TcpipDnsSettings;
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam as LONG_PTR);
            if (*pSettings).bAdd == 0 {
                if LoadStringW(netcfgx_hInstance, (IDS_MOD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }
                SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_SETTEXT, 0, (*pSettings).szIP.as_ptr() as LPARAM);
                EnableWindow(GetDlgItem(hwndDlg, IDC_OK), (TRUE) as i32);
            } else {
                if LoadStringW(netcfgx_hInstance, (IDS_ADD) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
                    szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0;
                    SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
                }
                EnableWindow(GetDlgItem(hwndDlg, IDC_OK), (FALSE) as i32);
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam as u32) == (IDCANCEL) as u16 {
                EndDialog(hwndDlg, FALSE as INT_PTR);
            } else if LOWORD!(wParam as u32) == (IDC_OK) as u16 {
                pSettings = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipDnsSettings;
                SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_GETTEXT, 16, (*pSettings).szIP.as_ptr() as LPARAM);
                if SendMessageW((*pSettings).hDlgCtrl, LB_FINDSTRING, 0, (*pSettings).szIP.as_ptr() as LPARAM) == (LB_ERR) as LRESULT {
                    if (*pSettings).bAdd != 0 {
                        SendMessageW((*pSettings).hDlgCtrl, LB_ADDSTRING, 0, (*pSettings).szIP.as_ptr() as LPARAM);
                    }
                    EndDialog(hwndDlg, TRUE as INT_PTR);
                    return FALSE as INT_PTR;
                }
                if (*pSettings).bAdd == 0 {
                    EndDialog(hwndDlg, FALSE as INT_PTR);
                    return FALSE as INT_PTR;
                }
                DisplayError((IDS_DUP_SUFFIX) as u32, (IDS_TCPIP) as u32, MB_ICONERROR);
            }
        }
        WM_NOTIFY => {
            lpnmipa = lParam as LPNMIPADDRESS;
            if (*lpnmipa).hdr.code == (IPN_FIELDCHANGED) as u32 {
                if (*lpnmipa).hdr.idFrom == IDC_IPADDR as UINT_PTR {
                    if SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_GETADDRESS) as u32, 0, core::ptr::addr_of_mut!(dwIpAddr) as LPARAM) == 4 {
                        EnableWindow(GetDlgItem(hwndDlg, IDC_OK), (TRUE) as i32);
                    }
                }
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTcpipAdvancedDNSDlg(
    hwndDlg: HWND,
    This: *mut TcpipConfNotifyImpl) {
    let mut szBuffer: [u16; 200] = [0; 200];
    let mut pFirst: LPWSTR;
    let mut pSep: LPWSTR;
    let mut pList: LPWSTR;
    let mut pAddr: *mut IP_ADDR;
    let mut dwIpAddr: u32 = 0;

    pAddr = (*(*This).pCurrentConfig).Ns;
    while !pAddr.is_null() {
        dwIpAddr = (*pAddr).IpAddress;
        swprintf(szBuffer.as_mut_ptr(),
                 w!("%lu.%lu.%lu.%lu") as *const u16,
                 FIRST_IPADDRESS!(dwIpAddr), SECOND_IPADDRESS!(dwIpAddr), THIRD_IPADDRESS!(dwIpAddr), FOURTH_IPADDRESS!(dwIpAddr));

        SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_ADDSTRING, 0, szBuffer.as_ptr() as LPARAM);
        pAddr = (*pAddr).Next;
    }
    SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_SETCURSEL, 0, 0);

    if (*(*This).pCurrentConfig).pDNS.is_null() {
        return;
    }

    if (*(*(*This).pCurrentConfig).pDNS).RegisterAdapterName != 0 {
        CheckDlgButton(hwndDlg, IDC_REGSUFFIX, BST_CHECKED);
    } else {
        EnableWindow(GetDlgItem(hwndDlg, IDC_USESUFFIX), (FALSE) as i32);
    }

    if (*(*(*This).pCurrentConfig).pDNS).RegistrationEnabled != 0 {
        CheckDlgButton(hwndDlg, IDC_USESUFFIX, BST_CHECKED);
    }

    if (*(*(*This).pCurrentConfig).pDNS).szDomain[0] != 0 {
        SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
    }

    if (*(*(*This).pCurrentConfig).pDNS).UseDomainNameDevolution != 0 {
        CheckDlgButton(hwndDlg, IDC_TOPPRIMSUFFIX, BST_CHECKED);
    }

    if (*(*(*This).pCurrentConfig).pDNS).szSearchList.is_null() || wcslen((*(*(*This).pCurrentConfig).pDNS).szSearchList) == 0 {
        CheckDlgButton(hwndDlg, IDC_PRIMSUFFIX, BST_CHECKED);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXADD), (FALSE) as i32);
        return;
    }

    pList = (*(*(*This).pCurrentConfig).pDNS).szSearchList;
    if wcslen(pList) != 0 {
        pFirst = pList;
        loop {
            pSep = wcschr(pFirst, ',' as u16);
            if !pSep.is_null() {
                *pSep.offset(0 as isize) = 0;
                SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_ADDSTRING, 0, pFirst as LPARAM);
                pFirst = pSep.offset(1 as isize);
                *pSep.offset(0 as isize) = ',' as u16;
            } else {
                SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_ADDSTRING, 0, pFirst as LPARAM);
                break;
            }
            if TRUE == 0 {
                break;
            }
        }

        EnableWindow(GetDlgItem(hwndDlg, IDC_TOPPRIMSUFFIX), (FALSE) as i32);
        CheckDlgButton(hwndDlg, IDC_SELSUFFIX, BST_CHECKED);
        SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_SETCURSEL, 0, 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ToggleUpDown(hwndDlg: HWND, hDlgCtrl: HWND, UpButton: u32, DownButton: u32, ModButton: u32, DelButton: u32) {
    let lResult: isize;
    let lCount: isize;

    lResult = (SendMessageW(hDlgCtrl, LB_GETCURSEL, 0, 0)) as isize;
    lCount = (SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0)) as isize;
    if lResult != (LB_ERR) as isize {
        if lResult == 0 {
            EnableWindow(GetDlgItem(hwndDlg, UpButton as i32), (FALSE) as i32);
        } else {
            EnableWindow(GetDlgItem(hwndDlg, UpButton as i32), (TRUE) as i32);
        }

        if lResult < lCount - 1 {
            EnableWindow(GetDlgItem(hwndDlg, DownButton as i32), (TRUE) as i32);
        } else {
            EnableWindow(GetDlgItem(hwndDlg, DownButton as i32), (FALSE) as i32);
        }
    }

    if lCount != 0 {
        EnableWindow(GetDlgItem(hwndDlg, ModButton as i32), (TRUE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, DelButton as i32), (TRUE) as i32);
    } else {
        EnableWindow(GetDlgItem(hwndDlg, ModButton as i32), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, DelButton as i32), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, UpButton as i32), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, DownButton as i32), (FALSE) as i32);
    }
}
#[no_mangle]
// C: `VOID MoveItem(HWND hDlgCtrl, INT pos)` — INT, not INT_PTR. Identical on i386,
// 32 vs 64 bits on amd64/arm64, so this is the width class that compiles everywhere
// and passes garbage in the high half on exactly two of three arches.
pub unsafe extern "C" fn MoveItem(
    hDlgCtrl: HWND,
    pos: INT) {
    let mut szBuffer: [u16; 100] = [0; 100];
    let lResult: LRESULT;

    lResult = SendMessageW(hDlgCtrl, LB_GETCURSEL, 0, 0);
    if lResult != (LB_ERR) as LRESULT
    {
        if SendMessageW(hDlgCtrl, LB_GETTEXTLEN, lResult as WPARAM, 0) < (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>() - 1) as LRESULT
        {
            if SendMessageW(hDlgCtrl, LB_GETTEXT, lResult as WPARAM, szBuffer.as_mut_ptr() as LPARAM) != (LB_ERR) as LRESULT
            {
                SendMessageW(hDlgCtrl, LB_DELETESTRING, lResult as WPARAM, 0);
                SendMessageW(hDlgCtrl, LB_INSERTSTRING, (lResult + pos as LRESULT) as WPARAM, szBuffer.as_ptr() as LPARAM);
                SendMessageW(hDlgCtrl, LB_SETCURSEL, (lResult + pos as LRESULT) as WPARAM, 0);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RemoveItem(
    hDlgCtrl: HWND) {
    let mut lResult: LRESULT;
    let mut lCount: LRESULT;

    lResult = SendMessageW(hDlgCtrl, LB_GETCURSEL, 0, 0);
    if lResult != (LB_ERR) as LRESULT
    {
        SendMessageW(hDlgCtrl, LB_DELETESTRING, lResult as WPARAM, 0);
        lCount = SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0);
        if lResult + 1 < lCount {
            SendMessageW(hDlgCtrl, LB_SETCURSEL, lResult as WPARAM, 0);
        } else {
            SendMessageW(hDlgCtrl, LB_SETCURSEL, (lCount - 1) as WPARAM, 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetListViewEntries(
    hDlgCtrl: HWND,
) -> LPWSTR {
    let mut dwSize: DWORD;
    let iCount: INT;
    let mut iIndex: INT;
    let mut lResult: INT_PTR;
    let mut pszSearchList: LPWSTR;
    let mut pItem: LPWSTR;

    iCount = (SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0)) as i32;
    if iCount == 0 || iCount == LB_ERR {
        return core::ptr::null_mut();
    }

    dwSize = 0;

    iIndex = 0;
    while iIndex < iCount {
        lResult = SendMessageW(hDlgCtrl, LB_GETTEXTLEN, iIndex as WPARAM, 0);
        if lResult == (LB_ERR) as LRESULT {
            return core::ptr::null_mut();
        }

        dwSize += lResult as DWORD + 1;
        iIndex += 1;
    }

    pszSearchList = CoTaskMemAlloc(((dwSize + 1) * core::mem::size_of::<u16>() as DWORD) as SIZE_T) as LPWSTR;
    if pszSearchList.is_null() {
        return core::ptr::null_mut();
    }

    pItem = pszSearchList;
    iIndex = 0;
    while iIndex < iCount {
        lResult = SendMessageW(hDlgCtrl, LB_GETTEXT, iIndex as WPARAM, pItem as LPARAM);
        if lResult == (LB_ERR) as LRESULT {
            CoTaskMemFree(pszSearchList as LPVOID);
            return core::ptr::null_mut();
        }
        dwSize -= lResult as DWORD + 1;
        pItem = pItem.offset(wcslen(pItem) as isize);
        if iIndex != iCount - 1 {
            *pItem.offset(0 as isize) = ',' as u16;
            pItem = pItem.offset(1);
        }
        iIndex += 1;
    }
    *pItem.offset(0 as isize) = '\0' as u16;
    pszSearchList
}
#[no_mangle]
pub unsafe extern "C" fn StoreDNSSettings(
    hDlgCtrl: HWND,
    This: *mut TcpipConfNotifyImpl,
) {
    let mut iCount: i32;
    let mut iIndex: i32;
    let mut Ip: [u16; 16] = core::mem::zeroed();
    let mut pCur: *mut IP_ADDR;
    let mut pLast: *mut IP_ADDR;

    FreeIPAddr((*(*This).pCurrentConfig).Ns);
    (*(*This).pCurrentConfig).Ns = core::ptr::null_mut();

    iCount = (SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0)) as i32;
    if iCount == 0 || iCount == LB_ERR {
        return;
    }

    pLast = core::ptr::null_mut();
    iIndex = 0;
    while iIndex < iCount {
        if SendMessageW(hDlgCtrl, LB_GETTEXT, iIndex as WPARAM, Ip.as_mut_ptr() as LPARAM) == (LB_ERR) as LRESULT {
            break;
        }

        pCur = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
        if pCur.is_null() {
            break;
        }
        ZeroMemory!(pCur as *mut core::ffi::c_void, core::mem::size_of::<IP_ADDR>());
        (*pCur).IpAddress = GetIpAddressFromStringW(Ip.as_mut_ptr());

        if pLast.is_null() {
            (*(*This).pCurrentConfig).Ns = pCur;
        } else {
            (*pLast).Next = pCur;
        }

        pLast = pCur;
        pCur = (*pCur).Next;
        iIndex += 1;
    }
    (*(*This).pCurrentConfig).AutoconfigActive = FALSE as DWORD;
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAdvancedDnsDlg(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut This: *mut TcpipConfNotifyImpl;
    let mut page: LPPROPSHEETPAGE;
    let mut Dns: TcpipDnsSettings = core::mem::zeroed();
    let mut lIndex: LRESULT;
    let mut lLength: LRESULT;
    let mut Suffix: TcpipSuffixSettings = core::mem::zeroed();
    let mut lppsn: LPPSHNOTIFY;
    let mut szSuffix: [u16; 100] = [0; 100];
    let mut szFormat: [u16; 200] = [0; 200];
    let mut szBuffer: [u16; 300] = [0; 300];

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as LPPROPSHEETPAGE;
            This = (*page).lParam as *mut TcpipConfNotifyImpl;
            SetWindowLongPtr(hwndDlg, DWLP_USER, This as INT_PTR);
            InitializeTcpipAdvancedDNSDlg(hwndDlg, This);
            ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), (IDC_DNSADDRUP) as u32, (IDC_DNSADDRDOWN) as u32, (IDC_DNSADDRMOD) as u32, (IDC_DNSADDRDEL) as u32);
            ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
            return TRUE as INT_PTR;
        }
        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == (PSN_KILLACTIVE) as u32 {
                if IsDlgButtonChecked(hwndDlg, IDC_SELSUFFIX) == BST_CHECKED as UINT &&
                   SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_GETCOUNT, 0, 0) == 0 {
                    DisplayError((IDS_NO_SUFFIX) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
                    return TRUE as INT_PTR;
                }
                if SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_GETTEXT, (core::mem::size_of_val(&szSuffix) / core::mem::size_of::<u16>()) as WPARAM, szSuffix.as_mut_ptr() as LPARAM) != 0 {
                    szSuffix[(core::mem::size_of_val(&szSuffix) / core::mem::size_of::<u16>()) - 1] = 0;
                    if VerifyDNSSuffix(szSuffix.as_mut_ptr()) == (FALSE) as i32 {
                        if LoadStringW(netcfgx_hInstance, (IDS_DNS_SUFFIX) as u32, szFormat.as_mut_ptr(), (core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) as i32) != 0 {
                            szFormat[(core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) - 1] = 0;
                            swprintf(szBuffer.as_mut_ptr(), szFormat.as_ptr(), szSuffix.as_ptr());
                            if LoadStringW(netcfgx_hInstance, (IDS_TCPIP) as u32, szFormat.as_mut_ptr(), (core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) as i32) != 0 {
                                szFormat[(core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) - 1] = 0;
                            } else {
                                szFormat[0] = 0;
                            }
                            MessageBoxW(hwndDlg, szBuffer.as_ptr(), szFormat.as_ptr(), MB_ICONWARNING);
                            SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
                            SetFocus(GetDlgItem(hwndDlg, IDC_SUFFIX));
                            return TRUE as INT_PTR;
                        }
                    }
                }
            } else if (*lppsn).hdr.code == (PSN_APPLY) as u32 {
                This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl;
                if (*This).pCurrentConfig.is_null() || (*(*This).pCurrentConfig).pDNS.is_null() {
                    // break equivalent
                } else {
                    StoreDNSSettings(GetDlgItem(hwndDlg, IDC_DNSADDRLIST), This);
                    if IsDlgButtonChecked(hwndDlg, IDC_PRIMSUFFIX) == BST_CHECKED as UINT {
                        CoTaskMemFree(((*(*(*This).pCurrentConfig).pDNS).szSearchList).cast());
                        (*(*(*This).pCurrentConfig).pDNS).szSearchList = core::ptr::null_mut();
                        if IsDlgButtonChecked(hwndDlg, IDC_TOPPRIMSUFFIX) == BST_CHECKED as UINT {
                            (*(*(*This).pCurrentConfig).pDNS).UseDomainNameDevolution = TRUE as DWORD;
                        } else {
                            (*(*(*This).pCurrentConfig).pDNS).UseDomainNameDevolution = FALSE as DWORD;
                        }
                    } else {
                        CoTaskMemFree(((*(*(*This).pCurrentConfig).pDNS).szSearchList).cast());
                        (*(*(*This).pCurrentConfig).pDNS).szSearchList = core::ptr::null_mut();
                        (*(*(*This).pCurrentConfig).pDNS).UseDomainNameDevolution = FALSE as DWORD;
                        (*(*(*This).pCurrentConfig).pDNS).szSearchList = GetListViewEntries(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
                    }

                    if IsDlgButtonChecked(hwndDlg, IDC_REGSUFFIX) == BST_CHECKED as UINT {
                        (*(*(*This).pCurrentConfig).pDNS).RegisterAdapterName = TRUE as DWORD;
                        if IsDlgButtonChecked(hwndDlg, IDC_USESUFFIX) == BST_CHECKED as UINT {
                            (*(*(*This).pCurrentConfig).pDNS).RegistrationEnabled = TRUE as DWORD;
                        } else {
                            (*(*(*This).pCurrentConfig).pDNS).RegistrationEnabled = FALSE as DWORD;
                        }
                    } else {
                        (*(*(*This).pCurrentConfig).pDNS).RegisterAdapterName = FALSE as DWORD;
                        (*(*(*This).pCurrentConfig).pDNS).RegistrationEnabled = FALSE as DWORD;
                    }
                }
            }
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_DNSADDRLIST) as u16 && HIWORD!(wParam) == (LBN_SELCHANGE) as u16 {
                ToggleUpDown(hwndDlg, lParam as HWND, (IDC_DNSADDRUP) as u32, (IDC_DNSADDRDOWN) as u32, (IDC_DNSADDRMOD) as u32, (IDC_DNSADDRDEL) as u32);
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSSUFFIXLIST) as u16 && HIWORD!(wParam) == (LBN_SELCHANGE) as u16 {
                ToggleUpDown(hwndDlg, lParam as HWND, (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_PRIMSUFFIX) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                if SendMessageW(lParam as HWND, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXUP), (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXDOWN), (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXADD), (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXMOD), (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXDEL), (FALSE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_TOPPRIMSUFFIX), (TRUE) as i32);
                    SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_RESETCONTENT, 0, 0);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
            } else if LOWORD!(wParam) == (IDC_SELSUFFIX) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                if SendMessageW(lParam as HWND, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXADD), (TRUE) as i32);
                    EnableWindow(GetDlgItem(hwndDlg, IDC_TOPPRIMSUFFIX), (FALSE) as i32);
                    ToggleUpDown(hwndDlg, lParam as HWND, (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
            } else if LOWORD!(wParam) == (IDC_REGSUFFIX) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                if SendMessageW(lParam as HWND, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_USESUFFIX), (TRUE) as i32);
                } else {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_USESUFFIX), (FALSE) as i32);
                }
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSADDRUP) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                MoveItem(GetDlgItem(hwndDlg, IDC_DNSADDRLIST), (-(1 as isize)) as i32);
                ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), (IDC_DNSADDRUP) as u32, (IDC_DNSADDRDOWN) as u32, (IDC_DNSADDRMOD) as u32, (IDC_DNSADDRDEL) as u32);
                SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSADDRDOWN) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                MoveItem(GetDlgItem(hwndDlg, IDC_DNSADDRLIST), 1);
                ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), (IDC_DNSADDRUP) as u32, (IDC_DNSADDRDOWN) as u32, (IDC_DNSADDRMOD) as u32, (IDC_DNSADDRDEL) as u32);
                SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSSUFFIXUP) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                MoveItem(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), (-(1 as isize)) as i32);
                ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
                SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSSUFFIXDOWN) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                MoveItem(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), 1);
                ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
                SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSADDRDEL) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                RemoveItem(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
                ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), (IDC_DNSADDRUP) as u32, (IDC_DNSADDRDOWN) as u32, (IDC_DNSADDRMOD) as u32, (IDC_DNSADDRDEL) as u32);
                SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSSUFFIXDEL) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                RemoveItem(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
                ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
                SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            } else if LOWORD!(wParam) == (IDC_DNSADDRADD) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                Dns.bAdd = (TRUE) as i32;
                Dns.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSADDRLIST);
                if DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPDNS_DLG), core::ptr::null_mut(), Some(TcpipAddDNSDlg), core::ptr::addr_of_mut!(Dns) as LPARAM) != 0 {
                    ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), (IDC_DNSADDRUP) as u32, (IDC_DNSADDRDOWN) as u32, (IDC_DNSADDRMOD) as u32, (IDC_DNSADDRDEL) as u32);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
            } else if LOWORD!(wParam) == (IDC_DNSADDRMOD) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                lIndex = SendDlgItemMessage(hwndDlg, IDC_DNSADDRLIST, LB_GETCURSEL, 0, 0);
                if lIndex != (LB_ERR) as LRESULT {
                    Dns.bAdd = (FALSE) as i32;
                    Dns.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSADDRLIST);
                    SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_GETTEXT, lIndex as WPARAM, Dns.szIP.as_mut_ptr() as LPARAM);
                    if DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPDNS_DLG), core::ptr::null_mut(), Some(TcpipAddDNSDlg), core::ptr::addr_of_mut!(Dns) as LPARAM) != 0 {
                        SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_DELETESTRING, lIndex as WPARAM, 0);
                        SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_INSERTSTRING, lIndex as WPARAM, Dns.szIP.as_ptr() as LPARAM);
                        SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_SETCURSEL, lIndex as WPARAM, 0);
                        SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
            } else if LOWORD!(wParam) == (IDC_DNSSUFFIXADD) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                Suffix.bAdd = (TRUE) as i32;
                Suffix.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST);
                Suffix.Suffix = core::ptr::null_mut();
                if DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPSUFFIX_DLG), core::ptr::null_mut(), Some(TcpipAddSuffixDlg), core::ptr::addr_of_mut!(Suffix) as LPARAM) != 0 {
                    ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
                    lIndex = SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_ADDSTRING, 0, Suffix.Suffix as LPARAM);
                    if lIndex != (LB_ERR) as LRESULT {
                        SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_SETCURSEL, lIndex as WPARAM, 0);
                    }
                    SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    CoTaskMemFree((Suffix.Suffix).cast());
                }
            } else if LOWORD!(wParam) == (IDC_DNSSUFFIXMOD) as u16 && HIWORD!(wParam) == (BN_CLICKED) as u16 {
                lIndex = SendDlgItemMessage(hwndDlg, IDC_DNSSUFFIXLIST, LB_GETCURSEL, 0, 0);
                if lIndex != (LB_ERR) as LRESULT {
                    Suffix.bAdd = (FALSE) as i32;
                    Suffix.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST);
                    lLength = SendMessageW(Suffix.hDlgCtrl, LB_GETTEXTLEN, lIndex as WPARAM, 0);
                    if lLength != (LB_ERR) as LRESULT {
                        Suffix.Suffix = CoTaskMemAlloc(((lLength + 1) as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
                        if !Suffix.Suffix.is_null() {
                            SendMessageW(Suffix.hDlgCtrl, LB_GETTEXT, lIndex as WPARAM, Suffix.Suffix as LPARAM);
                            *Suffix.Suffix.offset((lLength) as isize) = 0;
                            if DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDD_TCPIPSUFFIX_DLG), core::ptr::null_mut(), Some(TcpipAddSuffixDlg), core::ptr::addr_of_mut!(Suffix) as LPARAM) != 0 {
                                if !Suffix.Suffix.is_null() {
                                    SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_DELETESTRING, lIndex as WPARAM, 0);
                                    SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_INSERTSTRING, lIndex as WPARAM, Suffix.Suffix as LPARAM);
                                    SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_SETCURSEL, lIndex as WPARAM, 0);
                                    SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
                                    CoTaskMemFree((Suffix.Suffix).cast());
                                }
                                ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), (IDC_DNSSUFFIXUP) as u32, (IDC_DNSSUFFIXDOWN) as u32, (IDC_DNSSUFFIXMOD) as u32, (IDC_DNSSUFFIXDEL) as u32);
                                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    FALSE as INT_PTR
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn PropSheetProc(
    hwndDlg: HWND,
    uMsg: UINT,
    lParam: LPARAM,
) -> core::ffi::c_int {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(
                netcfgx_hInstance,
                MAKEINTRESOURCEW!(IDI_NETWORK),
            );
            SendMessageW(
                hwndDlg,
                WM_SETICON,
                ICON_BIG,
                hIcon as LPARAM,
            );
        }
        _ => {}
    }
    return 0;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn PropSheetProc(
    hwndDlg: HWND,
    uMsg: UINT,
    lParam: LPARAM,
) -> core::ffi::c_int {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(
                netcfgx_hInstance,
                MAKEINTRESOURCEW!(IDI_NETWORK),
            );
            SendMessageW(
                hwndDlg,
                WM_SETICON,
                (ICON_BIG) as SIZE_T,
                hIcon as LPARAM,
            );
        }
        _ => {}
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn LaunchAdvancedTcpipSettings(
    hwndDlg: HWND,
    This: *mut TcpipConfNotifyImpl,
) {
    let mut pinfo: PROPSHEETHEADERW = core::mem::zeroed();
    let mut hppages: [HPROPSHEETPAGE; 3] = core::mem::zeroed();
    let mut szBuffer: [WCHAR; 100] = core::mem::zeroed();

    hppages[0] = InitializePropertySheetPage(MAKEINTRESOURCEW!(IDD_TCPIP_ADVIP_DLG), Some(TcpipAdvancedIpDlg), This as LPARAM, core::ptr::null_mut());
    hppages[1] = InitializePropertySheetPage(MAKEINTRESOURCEW!(IDD_TCPIP_ADVDNS_DLG), Some(TcpipAdvancedDnsDlg), This as LPARAM, core::ptr::null_mut());
    hppages[2] = InitializePropertySheetPage(MAKEINTRESOURCEW!(IDD_TCPIP_ADVOPT_DLG), Some(TcpipAdvancedOptDlg), This as LPARAM, core::ptr::null_mut());

    if LoadStringW(netcfgx_hInstance, (IDS_TCPIP) as u32, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0 {
        szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0u16;
    } else {
        szBuffer[0] = 0u16;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(pinfo) as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETHEADERW>());
    pinfo.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as u32;
    pinfo.dwFlags = (PSH_NOCONTEXTHELP | PSH_PROPTITLE | PSH_NOAPPLYNOW |
                    PSH_USEICONID | PSH_USECALLBACK) as u32;
    pinfo.u3.phpage = hppages.as_mut_ptr();
    pinfo.nPages = 3;
    pinfo.hwndParent = hwndDlg;
    pinfo.hInstance = netcfgx_hInstance;
    pinfo.pszCaption = szBuffer.as_ptr();
    *pinfo.u.pszIcon.as_mut() = MAKEINTRESOURCEW!(IDI_NETWORK) as *const u16;
    pinfo.pfnCallback = Some(PropSheetProc);

    StoreTcpipBasicSettings(hwndDlg, This, (FALSE) as i32);
    if PropertySheetW(core::ptr::addr_of_mut!(pinfo)) > 0 {
        InitializeTcpipBasicDlgCtrls(hwndDlg, (*This).pCurrentConfig);
        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
    }
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTcpipAltDlgCtrls(
    hwndDlg: HWND,
    pCurSettings: *mut TcpipSettings
) -> HRESULT {
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    if (*pCurSettings).DhcpEnabled != 0 {
        CheckRadioButton(hwndDlg, IDC_USEDHCP, IDC_NODHCP, IDC_USEDHCP);
        EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (FALSE) as i32);
    } else {
        CheckRadioButton(hwndDlg, IDC_USEDHCP, IDC_NODHCP, IDC_NODHCP);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (TRUE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (TRUE) as i32);

        if !(*pCurSettings).Ip.is_null() {
            SendDlgItemMessageA(hwndDlg, IDC_IPADDR, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).IpAddress as LPARAM);
            SendDlgItemMessageA(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).u.Subnetmask as LPARAM);
        }
    }

    if !(*pCurSettings).Gw.is_null() && (*(*pCurSettings).Gw).IpAddress != 0 {
        SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Gw).IpAddress as LPARAM);
    }

    if !(*pCurSettings).Ns.is_null() {
        SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ns).IpAddress as LPARAM);
        if !(*(*pCurSettings).Ns).Next.is_null() {
            SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETADDRESS) as u32, 0, (*(*(*pCurSettings).Ns).Next).IpAddress as LPARAM);
        } else {
            SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_CLEARADDRESS) as u32, 0, 0);
        }
    } else {
        SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_CLEARADDRESS) as u32, 0, 0);
        SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_CLEARADDRESS) as u32, 0, 0);
    }

    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn TcpipAltConfDlg(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let This: *mut TcpipConfNotifyImpl;
    let page: *mut core::ffi::c_void;
    let bNoDHCP: u8;

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as *mut core::ffi::c_void;
            This = page as *mut TcpipConfNotifyImpl;
            SetWindowLongPtr(hwndDlg, DWLP_USER, This as LONG_PTR);
            if !(*This).pCurrentConfig.is_null() {
                InitializeTcpipAltDlgCtrls(hwndDlg, (*This).pCurrentConfig);
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD!(wParam) as c_int {
                IDC_USEDHCP | IDC_NODHCP => {
                    if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                        bNoDHCP = if IsDlgButtonChecked(hwndDlg, IDC_NODHCP) == BST_CHECKED as UINT {
                            1u8
                        } else {
                            0u8
                        };
                        if bNoDHCP != 0 {
                            SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_CLEARADDRESS) as u32, 0, 0);
                            SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_CLEARADDRESS) as u32, 0, 0);
                            SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_CLEARADDRESS) as u32, 0, 0);
                        }

                        EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), bNoDHCP as i32);
                        EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), bNoDHCP as i32);
                        EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), bNoDHCP as i32);
                        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), bNoDHCP as i32);
                        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), bNoDHCP as i32);

                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn AddAlternativeDialog(
    hDlg: HWND,
    This: *mut TcpipConfNotifyImpl,
) {
    let hpage: HPROPSHEETPAGE;

    hpage = InitializePropertySheetPage(
        MAKEINTRESOURCEW!(IDD_TCPIP_ALTCF_DLG),
        Some(TcpipAltConfDlg),
        This as LPARAM,
        core::ptr::null_mut(),
    );
    if hpage.is_null() {
        return;
    }

    SendMessageW(
        hDlg,
        (PSM_INSERTPAGE) as u32,
        1,
        hpage as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn StoreTcpipBasicSettings(
    hwndDlg: HWND,
    This: *mut TcpipConfNotifyImpl,
    bApply: BOOL,
) -> INT_PTR {
    let mut dwIpAddr: DWORD = 0;

    if IsDlgButtonChecked(hwndDlg, IDC_NODHCP) == BST_CHECKED as UINT {
        (*(*This).pCurrentConfig).DhcpEnabled = FALSE as DWORD;
        if SendDlgItemMessageW(
            hwndDlg,
            IDC_IPADDR,
            (IPM_GETADDRESS) as u32,
            0,
            &mut dwIpAddr as *mut _ as LPARAM,
        ) != 4
        {
            if bApply != 0 {
                DisplayError((IDS_NO_IPADDR_SET) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
                SetFocus(GetDlgItem(hwndDlg, IDC_IPADDR));
                return (E_FAIL) as LRESULT;
            }
        }
        if (*(*This).pCurrentConfig).Ip.is_null() {
            (*(*This).pCurrentConfig).Ip = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
            if (*(*This).pCurrentConfig).Ip.is_null() {
                return (E_OUTOFMEMORY) as LRESULT;
            }
            ZeroMemory!((*(*This).pCurrentConfig).Ip as *mut _, core::mem::size_of::<IP_ADDR>());
        }
        (*(*(*This).pCurrentConfig).Ip).IpAddress = dwIpAddr;

        if SendDlgItemMessageW(
            hwndDlg,
            IDC_SUBNETMASK,
            (IPM_GETADDRESS) as u32,
            0,
            &mut dwIpAddr as *mut _ as LPARAM,
        ) != 4
        {
            if bApply != 0 {
                DisplayError((IDS_NO_SUBMASK_SET) as u32, (IDS_TCPIP) as u32, MB_ICONWARNING);
            }
            if SendDlgItemMessageW(
                hwndDlg,
                IDC_IPADDR,
                (IPM_GETADDRESS) as u32,
                0,
                &mut dwIpAddr as *mut _ as LPARAM,
            ) == 4
            {
                if dwIpAddr <= MAKEIPADDRESS!(127, 255, 255, 255) {
                    dwIpAddr = MAKEIPADDRESS!(255, 0, 0, 0);
                } else if dwIpAddr <= MAKEIPADDRESS!(191, 255, 255, 255) {
                    dwIpAddr = MAKEIPADDRESS!(255, 255, 0, 0);
                } else if dwIpAddr <= MAKEIPADDRESS!(223, 255, 255, 255) {
                    dwIpAddr = MAKEIPADDRESS!(255, 255, 255, 0);
                }

                SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, dwIpAddr as LPARAM);
            }
            if bApply != 0 {
                SetFocus(GetDlgItem(hwndDlg, IDC_SUBNETMASK));
                return (E_FAIL) as LRESULT;
            }
        }
        (*(*(*This).pCurrentConfig).Ip).u.Subnetmask = dwIpAddr;
    } else {
        (*(*This).pCurrentConfig).DhcpEnabled = TRUE as DWORD;
    }

    if SendDlgItemMessageW(
        hwndDlg,
        IDC_DEFGATEWAY,
        (IPM_GETADDRESS) as u32,
        0,
        &mut dwIpAddr as *mut _ as LPARAM,
    ) == 4
    {
        if (*(*This).pCurrentConfig).Gw.is_null() {
            (*(*This).pCurrentConfig).Gw = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
            if (*(*This).pCurrentConfig).Gw.is_null() {
                return (E_OUTOFMEMORY) as LRESULT;
            }
            ZeroMemory!((*(*This).pCurrentConfig).Gw as *mut _, core::mem::size_of::<IP_ADDR>());
        }
        (*(*(*This).pCurrentConfig).Gw).IpAddress = dwIpAddr;
    } else {
        if !(*(*This).pCurrentConfig).Gw.is_null() {
            let pNextGw = (*(*(*This).pCurrentConfig).Gw).Next;
            CoTaskMemFree((*(*This).pCurrentConfig).Gw as *mut _);
            (*(*This).pCurrentConfig).Gw = pNextGw;
        }
    }

    if IsDlgButtonChecked(hwndDlg, IDC_FIXEDDNS) == BST_CHECKED as UINT {
        let mut bSkip = FALSE;
        (*(*This).pCurrentConfig).AutoconfigActive = FALSE as DWORD;
        if SendDlgItemMessageW(
            hwndDlg,
            IDC_DNS1,
            (IPM_GETADDRESS) as u32,
            0,
            &mut dwIpAddr as *mut _ as LPARAM,
        ) == 4
        {
            if (*(*This).pCurrentConfig).Ns.is_null() {
                (*(*This).pCurrentConfig).Ns = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
                if (*(*This).pCurrentConfig).Ns.is_null() {
                    return (E_OUTOFMEMORY) as LRESULT;
                }
                ZeroMemory!((*(*This).pCurrentConfig).Ns as *mut _, core::mem::size_of::<IP_ADDR>());
            }
            (*(*(*This).pCurrentConfig).Ns).IpAddress = dwIpAddr;
        } else if !(*(*This).pCurrentConfig).Ns.is_null() {
            let pTemp = (*(*(*This).pCurrentConfig).Ns).Next;

            CoTaskMemFree((*(*This).pCurrentConfig).Ns as *mut _);
            (*(*This).pCurrentConfig).Ns = pTemp;
            bSkip = TRUE;
        }

        if SendDlgItemMessageW(
            hwndDlg,
            IDC_DNS2,
            (IPM_GETADDRESS) as u32,
            0,
            &mut dwIpAddr as *mut _ as LPARAM,
        ) == 4
        {
            if (*(*This).pCurrentConfig).Ns.is_null() || bSkip != 0 {
                if (*(*This).pCurrentConfig).Ns.is_null() {
                    (*(*This).pCurrentConfig).Ns = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
                    if (*(*This).pCurrentConfig).Ns.is_null() {
                        return (E_OUTOFMEMORY) as LRESULT;
                    }
                    ZeroMemory!((*(*This).pCurrentConfig).Ns as *mut _, core::mem::size_of::<IP_ADDR>());
                }
                (*(*(*This).pCurrentConfig).Ns).IpAddress = dwIpAddr;
            } else if (*(*(*This).pCurrentConfig).Ns).Next.is_null() {
                (*(*(*This).pCurrentConfig).Ns).Next = CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T) as *mut IP_ADDR;
                if (*(*(*This).pCurrentConfig).Ns).Next.is_null() {
                    return (E_OUTOFMEMORY) as LRESULT;
                }
                ZeroMemory!((*(*(*This).pCurrentConfig).Ns).Next as *mut _, core::mem::size_of::<IP_ADDR>());
                (*(*(*(*This).pCurrentConfig).Ns).Next).IpAddress = dwIpAddr;
            } else {
                (*(*(*(*This).pCurrentConfig).Ns).Next).IpAddress = dwIpAddr;
            }
        } else {
            if !(*(*This).pCurrentConfig).Ns.is_null() && !(*(*(*This).pCurrentConfig).Ns).Next.is_null() {
                if !(*(*(*(*This).pCurrentConfig).Ns).Next).Next.is_null() {
                    let pTemp = (*(*(*(*This).pCurrentConfig).Ns).Next).Next;
                    CoTaskMemFree((*(*(*This).pCurrentConfig).Ns).Next as *mut _);
                    (*(*(*This).pCurrentConfig).Ns).Next = pTemp;
                } else {
                    CoTaskMemFree((*(*(*This).pCurrentConfig).Ns).Next as *mut _);
                    (*(*(*This).pCurrentConfig).Ns).Next = core::ptr::null_mut();
                }
            }
        }
    } else {
        (*(*This).pCurrentConfig).AutoconfigActive = TRUE as DWORD;
    }
    (S_OK) as LRESULT
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTcpipBasicDlgCtrls(
    hwndDlg: HWND,
    pCurSettings: *mut TcpipSettings
) -> HRESULT {
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_IPADDR, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 0, MAKEIPRANGE!(1, 223));
    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 1, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 2, MAKEIPRANGE!(0, 255));
    SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETRANGE) as u32, 3, MAKEIPRANGE!(0, 255));

    if (*pCurSettings).DhcpEnabled != 0 {
        CheckDlgButton(hwndDlg, IDC_USEDHCP, BST_CHECKED);
        EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_AUTODNS), (TRUE) as i32);
    } else {
        CheckDlgButton(hwndDlg, IDC_NODHCP, BST_CHECKED);

        if !(*pCurSettings).Ip.is_null() {
            SendDlgItemMessageA(hwndDlg, IDC_IPADDR, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).IpAddress as LPARAM);
            SendDlgItemMessageA(hwndDlg, IDC_SUBNETMASK, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ip).u.Subnetmask as LPARAM);
        }
    }

    if !(*pCurSettings).Gw.is_null() && (*(*pCurSettings).Gw).IpAddress != 0 {
        SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Gw).IpAddress as LPARAM);
    }

    if (*pCurSettings).AutoconfigActive != 0 {
        CheckDlgButton(hwndDlg, IDC_AUTODNS, BST_CHECKED);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (FALSE) as i32);
    } else {
        CheckDlgButton(hwndDlg, IDC_FIXEDDNS, BST_CHECKED);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (TRUE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (TRUE) as i32);
        if !(*pCurSettings).Ns.is_null() {
            SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_SETADDRESS) as u32, 0, (*(*pCurSettings).Ns).IpAddress as LPARAM);
            if !(*(*pCurSettings).Ns).Next.is_null() {
                SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_SETADDRESS) as u32, 0, (*(*(*pCurSettings).Ns).Next).IpAddress as LPARAM);
            } else {
                SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_CLEARADDRESS) as u32, 0, 0);
            }
        } else {
            SendDlgItemMessageW(hwndDlg, IDC_DNS1, (IPM_CLEARADDRESS) as u32, 0, 0);
            SendDlgItemMessageW(hwndDlg, IDC_DNS2, (IPM_CLEARADDRESS) as u32, 0, 0);
        }
    }

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn CopyIpAddrString(
    pSrc: *mut IP_ADDR_STRING,
    pTarget: *mut *mut IP_ADDR,
    Type: COPY_TYPE,
    mut szMetric: LPWSTR,
) -> HRESULT {
    let mut pCurrent: *mut IP_ADDR_STRING;
    let mut pNew: *mut IP_ADDR;
    let mut pLast: *mut IP_ADDR;

    pCurrent = pSrc;
    pLast = core::ptr::null_mut();

    while !pCurrent.is_null() {
        pNew = (CoTaskMemAlloc((core::mem::size_of::<IP_ADDR>()) as SIZE_T)).cast();
        if pNew.is_null() {
            break;
        }
        ZeroMemory!(pNew as *mut core::ffi::c_void, core::mem::size_of::<IP_ADDR>());
        (*pNew).IpAddress = GetIpAddressFromStringA((*pCurrent).IpAddress.String.as_mut_ptr());
        if (*pNew).IpAddress == 0 {
            CoTaskMemFree(pNew as *mut core::ffi::c_void);
            return E_FAIL;
        }

        if Type == SUBMASK {
            (*pNew).u.Subnetmask = GetIpAddressFromStringA((*pCurrent).IpMask.String.as_mut_ptr());
            (*pNew).NTEContext = (*pCurrent).Context;
        } else if Type == METRIC {
            if !szMetric.is_null() && *szMetric.offset(0 as isize) != 0u16 {
                (*pNew).u.Metric = _wtoi(szMetric) as u16;
                szMetric = szMetric.offset((wcslen(szMetric) + 1) as isize);
            }
        }

        if pLast.is_null() {
            *pTarget = pNew;
        } else {
            (*pLast).Next = pNew;
        }

        pLast = pNew;
        pCurrent = (*pCurrent).Next;
    }
    (*pLast).Next = core::ptr::null_mut();
    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn TcpipBasicDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let This: *mut TcpipConfNotifyImpl;
    let page: LPPROPSHEETPAGE;
    let lpnmipa: LPNMIPADDRESS;
    let lppsn: LPPSHNOTIFY;
    let mut dwIpAddr: DWORD = 0;

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as LPPROPSHEETPAGE;
            This = (*page).lParam as *mut TcpipConfNotifyImpl;
            if !(*This).pCurrentConfig.is_null() {
                InitializeTcpipBasicDlgCtrls(hwndDlg, (*This).pCurrentConfig);
            }
            SetWindowLongPtr(hwndDlg, DWLP_USER, This as LONG_PTR);
            return TRUE as INT_PTR;
        }
        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            lpnmipa = lParam as LPNMIPADDRESS;
            if (*lpnmipa).hdr.code == (IPN_FIELDCHANGED) as u32 {
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                if (*lpnmipa).hdr.idFrom == IDC_IPADDR as UINT_PTR {
                    if SendDlgItemMessageW(
                        hwndDlg,
                        IDC_IPADDR,
                        (IPM_GETADDRESS) as u32,
                        0,
                        core::ptr::addr_of_mut!(dwIpAddr) as LPARAM,
                    ) == 4
                    {
                        if dwIpAddr <= MAKEIPADDRESS!(127, 255, 255, 255) {
                            SendDlgItemMessageW(
                                hwndDlg,
                                IDC_SUBNETMASK,
                                (IPM_SETADDRESS) as u32,
                                0,
                                MAKEIPADDRESS!(255, 0, 0, 0) as LPARAM,
                            );
                        } else if dwIpAddr <= MAKEIPADDRESS!(191, 255, 255, 255) {
                            SendDlgItemMessageW(
                                hwndDlg,
                                IDC_SUBNETMASK,
                                (IPM_SETADDRESS) as u32,
                                0,
                                MAKEIPADDRESS!(255, 255, 0, 0) as LPARAM,
                            );
                        } else if dwIpAddr <= MAKEIPADDRESS!(223, 255, 255, 255) {
                            SendDlgItemMessageW(
                                hwndDlg,
                                IDC_SUBNETMASK,
                                (IPM_SETADDRESS) as u32,
                                0,
                                MAKEIPADDRESS!(255, 255, 255, 0) as LPARAM,
                            );
                        }
                    }
                }
            } else if (*lppsn).hdr.code == (PSN_APPLY) as u32 {
                This = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl;
                if StoreTcpipBasicSettings(hwndDlg, This, (TRUE) as i32) >= 0 {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (PSNRET_NOERROR) as LRESULT);
                } else {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (PSNRET_INVALID) as LRESULT);
                }
                return TRUE as INT_PTR;
            }
        }
        WM_COMMAND => {
            if HIWORD!(wParam as DWORD) == (BN_CLICKED) as u16 {
                match LOWORD!(wParam as DWORD) as c_int {
                    IDC_USEDHCP => {
                        if SendMessageW(
                            GetParent(hwndDlg),
                            (PSM_INDEXTOID) as u32,
                            1,
                            0,
                        ) == 0
                        {
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                            SendDlgItemMessageW(
                                hwndDlg,
                                IDC_IPADDR,
                                (IPM_CLEARADDRESS) as u32,
                                0,
                                0,
                            );
                            SendDlgItemMessageW(
                                hwndDlg,
                                IDC_SUBNETMASK,
                                (IPM_CLEARADDRESS) as u32,
                                0,
                                0,
                            );
                            SendDlgItemMessageW(
                                hwndDlg,
                                IDC_DEFGATEWAY,
                                (IPM_CLEARADDRESS) as u32,
                                0,
                                0,
                            );
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), (FALSE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_AUTODNS), (TRUE) as i32);
                            AddAlternativeDialog(
                                GetParent(hwndDlg),
                                GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl,
                            );
                        }
                    }
                    IDC_NODHCP => {
                        if SendMessageW(
                            GetParent(hwndDlg),
                            (PSM_INDEXTOID) as u32,
                            1,
                            0,
                        ) != 0
                        {
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), (TRUE) as i32);
                            if IsDlgButtonChecked(hwndDlg, IDC_AUTODNS) == BST_CHECKED as UINT {
                                CheckDlgButton(hwndDlg, IDC_AUTODNS, BST_UNCHECKED);
                            }
                            EnableWindow(GetDlgItem(hwndDlg, IDC_AUTODNS), (FALSE) as i32);
                            CheckDlgButton(hwndDlg, IDC_FIXEDDNS, BST_CHECKED);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (TRUE) as i32);
                            EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (TRUE) as i32);
                            SendMessageW(
                                GetParent(hwndDlg),
                                (PSM_REMOVEPAGE) as u32,
                                1,
                                0,
                            );
                        }
                    }
                    IDC_AUTODNS => {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        SendDlgItemMessageW(
                            hwndDlg,
                            IDC_DNS1,
                            (IPM_CLEARADDRESS) as u32,
                            0,
                            0,
                        );
                        SendDlgItemMessageW(
                            hwndDlg,
                            IDC_DNS2,
                            (IPM_CLEARADDRESS) as u32,
                            0,
                            0,
                        );
                        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (FALSE) as i32);
                        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (FALSE) as i32);
                    }
                    IDC_FIXEDDNS => {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), (TRUE) as i32);
                        EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), (TRUE) as i32);
                    }
                    IDC_ADVANCED => {
                        LaunchAdvancedTcpipSettings(
                            hwndDlg,
                            GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut TcpipConfNotifyImpl,
                        );
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnQueryInterface(
    iface: *mut INetCfgComponentPropertyUi,
    iid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut TcpipConfNotifyImpl;

    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(iid, &IID_IUnknown) ||
       IsEqualIID!(iid, &IID_INetCfgComponentPropertyUi)
    {
        *ppvObj = This as LPVOID;
        INetCfgComponentPropertyUi_AddRef(iface);
        return S_OK;
    }
    else if IsEqualIID!(iid, &IID_INetCfgComponentControl)
    {
        *ppvObj = core::ptr::addr_of_mut!((*This).lpVtblCompControl) as LPVOID;
        INetCfgComponentPropertyUi_AddRef(iface);
        return S_OK;
    }

    E_NOINTERFACE
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnAddRef(
    iface: *mut INetCfgComponentPropertyUi,
) -> ULONG {
    let This = iface as *mut TcpipConfNotifyImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    return (refCount) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnRelease(
    iface: *mut INetCfgComponentPropertyUi
) -> ULONG {
    let This = iface as *mut TcpipConfNotifyImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
    }
    (refCount) as u32
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnQueryPropertyUi(
    iface: *mut INetCfgComponentPropertyUi,
    pUnkReserved: *mut IUnknown,
) -> HRESULT {
    let mut pLanInfo: *mut INetLanConnectionUiInfo = core::ptr::null_mut();
    let This = iface as *mut TcpipConfNotifyImpl;

    let hr = IUnknown_QueryInterface(
        pUnkReserved,
        &IID_INetLanConnectionUiInfo,
        &mut pLanInfo as *mut *mut INetLanConnectionUiInfo as *mut LPVOID,
    );
    if (hr) < 0 {
        return hr;
    }

    INetLanConnectionUiInfo_GetDeviceGuid(pLanInfo, &mut (*This).NetCfgInstanceId);

    IUnknown_Release(pUnkReserved);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnSetContext(
    iface: *mut INetCfgComponentPropertyUi,
    pUnkReserved: *mut IUnknown,
) -> HRESULT {
    let This = iface as *mut TcpipConfNotifyImpl;

    if iface.is_null() || pUnkReserved.is_null() {
        return E_POINTER;
    }

    (*This).pUnknown = pUnkReserved;
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn LoadDNSSettings(
    This: *mut TcpipConfNotifyImpl
) -> HRESULT {
    let mut pStr: LPOLESTR = core::ptr::null_mut();
    let mut szBuffer: [u16; 200] = [0; 200];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;

    (*(*This).pCurrentConfig).pDNS = CoTaskMemAlloc((core::mem::size_of::<TcpipAdvancedDNSDlgSettings>()) as SIZE_T) as *mut TcpipAdvancedDNSDlgSettings;
    if (*(*This).pCurrentConfig).pDNS.is_null() {
        return E_FAIL;
    }

    ZeroMemory!((*(*This).pCurrentConfig).pDNS as LPVOID, core::mem::size_of::<TcpipAdvancedDNSDlgSettings>());

    if StringFromCLSID(core::ptr::addr_of!((*This).NetCfgInstanceId), core::ptr::addr_of_mut!(pStr)) < 0 {
        return E_FAIL;
    }

    swprintf(szBuffer.as_mut_ptr(), w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%s") as LPCWSTR, pStr);
    CoTaskMemFree(pStr as LPVOID);
    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuffer.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey)) == (ERROR_SUCCESS) as i32 {
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        RegQueryValueExW(hKey, w!("RegisterAdapterName") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!((*(*(*This).pCurrentConfig).pDNS).RegisterAdapterName) as LPBYTE, core::ptr::addr_of_mut!(dwSize));

        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        RegQueryValueExW(hKey, w!("RegistrationEnabled") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!((*(*(*This).pCurrentConfig).pDNS).RegistrationEnabled) as LPBYTE, core::ptr::addr_of_mut!(dwSize));

        dwSize = core::mem::size_of_val(&(*(*(*This).pCurrentConfig).pDNS).szDomain) as DWORD;
        RegQueryValueExW(hKey, w!("Domain") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), (*(*(*This).pCurrentConfig).pDNS).szDomain.as_mut_ptr() as LPBYTE, core::ptr::addr_of_mut!(dwSize));

        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(hKey, w!("InterfaceMetric") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!((*(*This).pCurrentConfig).Metric) as LPBYTE, core::ptr::addr_of_mut!(dwSize)) != (ERROR_SUCCESS) as i32 {
            (*(*This).pCurrentConfig).Metric = 0;
        }

        RegCloseKey(hKey);
    }

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters") as LPCWSTR, 0, KEY_READ, core::ptr::addr_of_mut!(hKey)) == (ERROR_SUCCESS) as i32 {
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        RegQueryValueExW(hKey, w!("UseDomainNameDevolution") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!((*(*(*This).pCurrentConfig).pDNS).UseDomainNameDevolution) as LPBYTE, core::ptr::addr_of_mut!(dwSize));

        dwSize = 0;
        if RegQueryValueExW(hKey, w!("SearchList") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSize)) == (ERROR_SUCCESS) as i32 {
            (*(*(*This).pCurrentConfig).pDNS).szSearchList = CoTaskMemAlloc((dwSize as usize) as SIZE_T) as LPWSTR;
            if !(*(*(*This).pCurrentConfig).pDNS).szSearchList.is_null() {
                if RegQueryValueExW(hKey, w!("SearchList") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), (*(*(*This).pCurrentConfig).pDNS).szSearchList as LPBYTE, core::ptr::addr_of_mut!(dwSize)) != (ERROR_SUCCESS) as i32 {
                    CoTaskMemFree((*(*(*This).pCurrentConfig).pDNS).szSearchList as LPVOID);
                    (*(*(*This).pCurrentConfig).pDNS).szSearchList = core::ptr::null_mut();
                }
            }
        }
        RegCloseKey(hKey);
    }
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn LoadTcpFilterSettingsFromRegistry(hKey: HKEY, szName: LPCWSTR, Size: LPDWORD) -> LPWSTR {
    let mut dwSize: DWORD = 0;
    let pData: LPWSTR;

    if RegQueryValueExW(hKey, szName, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), &mut dwSize) != (ERROR_SUCCESS) as i32 {
        return core::ptr::null_mut();
    }

    pData = CoTaskMemAlloc((dwSize) as SIZE_T) as LPWSTR;
    if pData.is_null() {
        return core::ptr::null_mut();
    }

    if RegQueryValueExW(hKey, szName, core::ptr::null_mut(), core::ptr::null_mut(), pData as LPBYTE, &mut dwSize) != (ERROR_SUCCESS) as i32 {
        CoTaskMemFree(pData as LPVOID);
        return core::ptr::null_mut();
    }
    *Size = dwSize;
    return pData;
}
#[no_mangle]
pub unsafe extern "C" fn LoadFilterSettings(
    This: *mut TcpipConfNotifyImpl
) -> HRESULT {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut pFilter: *mut TcpFilterSettings;
    let mut szBuffer: [u16; 200] = [0; 200];
    let mut pStr: LPOLESTR = core::ptr::null_mut();
    let mut dwVal: u32 = 0;
    let mut dwSize: u32;

    pFilter = CoTaskMemAlloc((core::mem::size_of::<TcpFilterSettings>()) as SIZE_T) as *mut TcpFilterSettings;
    if pFilter.is_null() {
        return E_FAIL;
    }

    ZeroMemory!(pFilter as LPVOID, core::mem::size_of::<TcpFilterSettings>());
    (*(*This).pCurrentConfig).pFilter = pFilter;

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters") as LPCWSTR, 0, KEY_READ, &mut hKey) == (ERROR_SUCCESS) as i32 {
        dwSize = core::mem::size_of::<u32>() as u32;
        if RegQueryValueExW(hKey, w!("EnableSecurityFilters") as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), &mut dwVal as *mut u32 as LPBYTE, &mut dwSize) == (ERROR_SUCCESS) as i32 {
            (*pFilter).EnableSecurityFilters = dwVal;
        }
        RegCloseKey(hKey);
    } else {
        (*pFilter).EnableSecurityFilters = FALSE as DWORD;
    }

    if StringFromCLSID(core::ptr::addr_of_mut!((*This).NetCfgInstanceId), &mut pStr) < 0 {
        return E_FAIL;
    }

    swprintf(szBuffer.as_mut_ptr(), w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%s") as LPCWSTR, pStr);
    CoTaskMemFree((pStr).cast());
    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuffer.as_ptr(), 0, KEY_READ, &mut hKey) != (ERROR_SUCCESS) as i32 {
        return S_OK;
    }
    (*pFilter).szTCPAllowedPorts = LoadTcpFilterSettingsFromRegistry(hKey, w!("TCPAllowedPorts") as LPCWSTR, core::ptr::addr_of_mut!((*pFilter).TCPSize));
    (*pFilter).szUDPAllowedPorts = LoadTcpFilterSettingsFromRegistry(hKey, w!("UDPAllowedPorts") as LPCWSTR, core::ptr::addr_of_mut!((*pFilter).UDPSize));
    (*pFilter).szRawIPAllowedProtocols = LoadTcpFilterSettingsFromRegistry(hKey, w!("RawIPAllowedProtocols") as LPCWSTR, core::ptr::addr_of_mut!((*pFilter).IPSize));
    RegCloseKey(hKey);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn Initialize(This: *mut TcpipConfNotifyImpl) -> HRESULT {
    let mut dwSize: DWORD;
    let Status: DWORD;
    let mut szBuffer: [u16; 50] = [0; 50];
    let mut pCurrentAdapter: *mut IP_ADAPTER_INFO;
    let mut pInfo: *mut IP_ADAPTER_INFO;
    let mut pPerInfo: *mut IP_PER_ADAPTER_INFO;
    let mut Info: IP_PER_ADAPTER_INFO = core::mem::zeroed();
    let mut pStr: LPOLESTR = core::ptr::null_mut();
    let hr: HRESULT;
    let mut bFound: BOOL;
    let pCurSettings: *mut TcpipSettings;
    let mut uLength: ULONG;

    if !(*This).pCurrentConfig.is_null() {
        return S_OK;
    }

    hr = StringFromCLSID(
        core::ptr::addr_of_mut!((*This).NetCfgInstanceId),
        core::ptr::addr_of_mut!(pStr),
    );
    if 0 > hr {
        return hr;
    }

    dwSize = 0;
    if GetAdaptersInfo(core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSize))
        != ERROR_BUFFER_OVERFLOW
    {
        CoTaskMemFree((pStr).cast());
        return E_FAIL;
    }

    pInfo = CoTaskMemAlloc((dwSize) as SIZE_T) as *mut IP_ADAPTER_INFO;
    if pInfo.is_null() {
        CoTaskMemFree((pStr).cast());
        return E_FAIL;
    }

    if GetAdaptersInfo(pInfo, core::ptr::addr_of_mut!(dwSize)) != ERROR_SUCCESS {
        CoTaskMemFree((pStr).cast());
        CoTaskMemFree(pInfo as LPVOID);
        return E_FAIL;
    }

    pCurrentAdapter = pInfo;
    bFound = (FALSE) as i32;
    while !pCurrentAdapter.is_null() {
        szBuffer[0] = 0;
        if MultiByteToWideChar(
            (CP_ACP) as u32,
            0,
            (*pCurrentAdapter).AdapterName.as_ptr(),
            -1,
            szBuffer.as_mut_ptr(),
            (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32,
        ) != 0
        {
            szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>())
                - 1] = 0;
        }
        if _wcsicmp(szBuffer.as_ptr(), pStr) == 0 {
            bFound = (TRUE) as i32;
            break;
        }
        pCurrentAdapter = (*pCurrentAdapter).Next;
    }
    CoTaskMemFree((pStr).cast());

    if bFound == 0 {
        CoTaskMemFree(pInfo as LPVOID);
        return E_FAIL;
    }

    pCurSettings = CoTaskMemAlloc(core::mem::size_of::<TcpipSettings>() as SIZE_T)
        as *mut TcpipSettings;
    if pCurSettings.is_null() {
        CoTaskMemFree(pInfo as LPVOID);
        return E_FAIL;
    }

    ZeroMemory!(
        pCurSettings as LPVOID,
        core::mem::size_of::<TcpipSettings>(),
    );
    (*This).pCurrentConfig = pCurSettings;
    (*pCurSettings).DhcpEnabled = (*pCurrentAdapter).DhcpEnabled;
    (*pCurSettings).Index = (*pCurrentAdapter).Index;

    if (*pCurrentAdapter).DhcpEnabled == 0 {
        CopyIpAddrString(
            core::ptr::addr_of_mut!((*pCurrentAdapter).IpAddressList),
            core::ptr::addr_of_mut!((*pCurSettings).Ip),
            SUBMASK,
            core::ptr::null_mut(),
        );
    }

    CopyIpAddrString(
        core::ptr::addr_of_mut!((*pCurrentAdapter).GatewayList),
        core::ptr::addr_of_mut!((*pCurSettings).Gw),
        METRIC,
        core::ptr::null_mut(),
    );

    uLength = core::mem::size_of::<IP_PER_ADAPTER_INFO>() as u32;
    ZeroMemory!(
        core::ptr::addr_of_mut!(Info) as LPVOID,
        core::mem::size_of::<IP_PER_ADAPTER_INFO>(),
    );

    if GetPerAdapterInfo(
        (*pCurrentAdapter).Index,
        core::ptr::addr_of_mut!(Info),
        core::ptr::addr_of_mut!(uLength),
    ) == ERROR_BUFFER_OVERFLOW
    {
        pPerInfo = CoTaskMemAlloc((uLength as u64) as SIZE_T) as *mut IP_PER_ADAPTER_INFO;
        if !pPerInfo.is_null() {
            let Status = GetPerAdapterInfo(
                (*pCurrentAdapter).Index,
                pPerInfo,
                core::ptr::addr_of_mut!(uLength),
            );
            if Status == (NOERROR) as u32 {
                if (*pPerInfo).AutoconfigActive == 0 {
                    CopyIpAddrString(
                        core::ptr::addr_of_mut!((*pPerInfo).DnsServerList),
                        core::ptr::addr_of_mut!((*pCurSettings).Ns),
                        IPADDR,
                        core::ptr::null_mut(),
                    );
                }
                (*pCurSettings).AutoconfigActive = (*pPerInfo).AutoconfigActive;
            }
            CoTaskMemFree(pPerInfo as LPVOID);
        }
    } else {
        if Info.AutoconfigActive == 0 {
            CopyIpAddrString(
                core::ptr::addr_of_mut!(Info.DnsServerList),
                core::ptr::addr_of_mut!((*pCurSettings).Ns),
                IPADDR,
                core::ptr::null_mut(),
            );
        }
        (*pCurSettings).AutoconfigActive = Info.AutoconfigActive;
    }

    if 0 > LoadFilterSettings(This) {
        return E_FAIL;
    }

    if 0 > LoadDNSSettings(This) {
        return E_FAIL;
    }

    CoTaskMemFree(pInfo as LPVOID);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnMergePropPages(
    iface: *mut INetCfgComponentPropertyUi,
    pdwDefPages: *mut DWORD,
    pahpspPrivate: *mut *mut BYTE,
    pcPages: *mut UINT,
    hwndParent: HWND,
    pszStartPage: *mut LPCWSTR,
) -> HRESULT {
    let mut hppages: *mut HPROPSHEETPAGE;
    let NumPages: UINT;
    let hr: HRESULT;
    let This = iface as *mut TcpipConfNotifyImpl;

    hr = Initialize(This);
    if (hr) < 0 {
        return hr;
    }

    if (*This).pCurrentConfig.is_null() == false && (*(*This).pCurrentConfig).DhcpEnabled != 0 {
        NumPages = 2;
    } else {
        NumPages = 1;
    }

    hppages = CoTaskMemAlloc((core::mem::size_of::<HPROPSHEETPAGE>() * NumPages as usize) as SIZE_T) as *mut HPROPSHEETPAGE;
    if hppages.is_null() {
        return E_FAIL;
    }

    *hppages.offset(0 as isize) = InitializePropertySheetPage(
        MAKEINTRESOURCEW!(IDD_TCPIP_BASIC_DLG),
        Some(TcpipBasicDlg),
        This as LPARAM,
        core::ptr::null_mut(),
    );
    if (*hppages.offset(0 as isize)).is_null() {
        CoTaskMemFree(hppages as LPVOID);
        return E_FAIL;
    }
    if NumPages == 2 {
        *hppages.offset(1 as isize) = InitializePropertySheetPage(
            MAKEINTRESOURCEW!(IDD_TCPIP_ALTCF_DLG),
            Some(TcpipAltConfDlg),
            This as LPARAM,
            core::ptr::null_mut(),
        );
        if (*hppages.offset(1 as isize)).is_null() {
            DestroyPropertySheetPage(*hppages.offset(0 as isize));
            CoTaskMemFree(hppages as LPVOID);
            return E_FAIL;
        }
    }

    *pahpspPrivate = hppages as *mut BYTE;
    *pcPages = NumPages;

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnValidateProperties(
    iface: *mut INetCfgComponentPropertyUi,
    hwndDlg: HWND,
) -> HRESULT {
    MessageBoxW(
        core::ptr::null_mut(),
        w!("INetCfgComponentPropertyUi_fnValidateProperties") as LPCWSTR,
        core::ptr::null_mut(),
        MB_OK,
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnApplyProperties(
    iface: *mut INetCfgComponentPropertyUi
) -> HRESULT {
    MessageBoxW(
        core::ptr::null_mut(),
        w!("INetCfgComponentPropertyUi_fnApplyProperties"),
        core::ptr::null_mut(),
        MB_OK
    );
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPropertyUi_fnCancelProperties(
    iface: *mut INetCfgComponentPropertyUi,
) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentControl_fnQueryInterface(
    iface: *mut INetCfgComponentControl,
    iid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_INetCfgComponentControl(iface);
    INetCfgComponentPropertyUi_QueryInterface(This as *mut INetCfgComponentPropertyUi, iid, ppvObj)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentControl_fnAddRef(
    iface: *mut INetCfgComponentControl
) -> ULONG {
    let This = impl_from_INetCfgComponentControl(iface);
    INetCfgComponentPropertyUi_AddRef(This as *mut INetCfgComponentPropertyUi)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentControl_fnRelease(
    iface: *mut INetCfgComponentControl
) -> ULONG {
    let This = impl_from_INetCfgComponentControl(iface);
    INetCfgComponentPropertyUi_Release(This as *mut INetCfgComponentPropertyUi)
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentControl_fnInitialize(
    iface: *mut INetCfgComponentControl,
    pIComp: *mut INetCfgComponent,
    pINetCfg: *mut INetCfg,
    fInstalling: BOOL,
) -> HRESULT {
    let This: LPTcpipConfNotifyImpl = impl_from_INetCfgComponentControl(iface);

    (*This).pNCfg = pINetCfg;
    (*This).pNComp = pIComp;

    S_OK
}
unsafe extern "C" fn CreateMultiSzString(
    pAddr: *mut IP_ADDR,
    Type: COPY_TYPE,
    Size: LPDWORD,
    bComma: BOOL,
) -> LPWSTR {
    let mut pStr: LPWSTR;
    let pRet: LPWSTR;
    let mut dwSize: DWORD;
    let mut dwIpAddr: DWORD = 0;
    let mut szBuffer: [u16; 30] = [0; 30];
    let mut pTemp: *mut IP_ADDR = pAddr;

    dwSize = 0;
    while !pTemp.is_null() {
        if Type == IPADDR {
            dwIpAddr = (*pTemp).IpAddress;
            swprintf(
                szBuffer.as_mut_ptr(),
                w!("%lu.%lu.%lu.%lu") as *const u16,
                FIRST_IPADDRESS!(dwIpAddr),
                SECOND_IPADDRESS!(dwIpAddr),
                THIRD_IPADDRESS!(dwIpAddr),
                FOURTH_IPADDRESS!(dwIpAddr),
            );
        } else if Type == SUBMASK {
            dwIpAddr = (*pTemp).u.Subnetmask;
            swprintf(
                szBuffer.as_mut_ptr(),
                w!("%lu.%lu.%lu.%lu") as *const u16,
                FIRST_IPADDRESS!(dwIpAddr),
                SECOND_IPADDRESS!(dwIpAddr),
                THIRD_IPADDRESS!(dwIpAddr),
                FOURTH_IPADDRESS!(dwIpAddr),
            );
        } else if Type == METRIC {
            swprintf(
                szBuffer.as_mut_ptr(),
                w!("%u") as *const u16,
                (*pTemp).u.Metric as c_uint,
            );
        }

        dwSize += (wcslen(szBuffer.as_ptr()) + 1) as u32;
        pTemp = (*pTemp).Next;
    }

    if dwSize == 0 {
        return core::ptr::null_mut();
    }

    pRet = CoTaskMemAlloc(((dwSize + 1) * core::mem::size_of::<u16>() as DWORD) as SIZE_T) as LPWSTR;
    pStr = pRet;
    if pStr.is_null() {
        return core::ptr::null_mut();
    }

    pTemp = pAddr;
    while !pTemp.is_null() {
        if Type == IPADDR {
            dwIpAddr = (*pTemp).IpAddress;
            swprintf(
                pStr,
                w!("%lu.%lu.%lu.%lu") as *const u16,
                FIRST_IPADDRESS!(dwIpAddr),
                SECOND_IPADDRESS!(dwIpAddr),
                THIRD_IPADDRESS!(dwIpAddr),
                FOURTH_IPADDRESS!(dwIpAddr),
            );
        } else if Type == SUBMASK {
            dwIpAddr = (*pTemp).u.Subnetmask;
            swprintf(
                pStr,
                w!("%lu.%lu.%lu.%lu") as *const u16,
                FIRST_IPADDRESS!(dwIpAddr),
                SECOND_IPADDRESS!(dwIpAddr),
                THIRD_IPADDRESS!(dwIpAddr),
                FOURTH_IPADDRESS!(dwIpAddr),
            );
        } else if Type == METRIC {
            swprintf(
                pStr,
                w!("%u") as *const u16,
                (*pTemp).u.Metric as c_uint,
            );
        }

        if bComma != 0 {
            pStr = pStr.add(wcslen(pStr) as usize);
            if !(*pTemp).Next.is_null() {
                *pStr.add(0) = ',' as u16;
                pStr = pStr.add(1);
            }
        } else {
            pStr = pStr.add(wcslen(pStr) as usize + 1);
        }
        pTemp = (*pTemp).Next;
    }
    *pStr.add(0) = '\0' as u16;
    *Size = (dwSize + 1) * core::mem::size_of::<u16>() as DWORD;
    pRet
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentControl_fnApplyRegistryChanges(
    iface: *mut INetCfgComponentControl
) -> HRESULT {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut pStr: LPOLESTR = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;
    let mut szBuffer: [u16; 200] = [0; 200];
    let mut pCurrentConfig: *mut TcpipSettings;
    let mut pOldConfig: *mut TcpipSettings;
    let mut NTEInstance: ULONG = 0;
    let mut DhcpApiVersion: DWORD = 0;

    let This: LPTcpipConfNotifyImpl = impl_from_INetCfgComponentControl(iface);

    pCurrentConfig = (*This).pCurrentConfig;
    (*This).pCurrentConfig = core::ptr::null_mut();

    if Initialize(This) < 0 {
        (*This).pCurrentConfig = pCurrentConfig;
        return E_FAIL;
    }
    pOldConfig = (*This).pCurrentConfig;
    (*This).pCurrentConfig = pCurrentConfig;

    if RegCreateKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters") as LPCWSTR, 0, core::ptr::null_mut(), 0, KEY_WRITE, core::ptr::null_mut(), &mut hKey, core::ptr::null_mut()) == (ERROR_SUCCESS) as i32 {
        if !(*pCurrentConfig).pDNS.is_null() {
            RegSetValueExW(hKey, w!("UseDomainNameDevolution") as LPCWSTR, 0, (REG_DWORD) as u32, core::ptr::addr_of_mut!((*(*pCurrentConfig).pDNS).UseDomainNameDevolution) as LPBYTE, core::mem::size_of::<DWORD>() as u32);
            RegSetValueExW(hKey, w!("SearchList") as LPCWSTR, 0, (REG_SZ) as u32, (*(*pCurrentConfig).pDNS).szSearchList as LPBYTE,
                       ((wcslen((*(*pCurrentConfig).pDNS).szSearchList)+1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        }
        if !(*pCurrentConfig).pFilter.is_null() {
            RegSetValueExW(hKey, w!("EnableSecurityFilters") as LPCWSTR, 0, (REG_DWORD) as u32,
                      core::ptr::addr_of_mut!((*(*pCurrentConfig).pFilter).EnableSecurityFilters) as LPBYTE, core::mem::size_of::<DWORD>() as u32);
        }
        RegCloseKey(hKey);
    }

    if StringFromCLSID(core::ptr::addr_of_mut!((*This).NetCfgInstanceId), &mut pStr) < 0 {
        return E_FAIL;
    }

    swprintf(szBuffer.as_mut_ptr(), w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%s") as LPCWSTR, pStr);
    CoTaskMemFree((pStr).cast());

    if RegCreateKeyExW(HKEY_LOCAL_MACHINE, szBuffer.as_ptr(), 0, core::ptr::null_mut(), 0, KEY_WRITE, core::ptr::null_mut(), &mut hKey, core::ptr::null_mut()) == (ERROR_SUCCESS) as i32 {
        if (*pCurrentConfig).Metric != 0 {
            RegSetValueExW(hKey, w!("InterfaceMetric") as LPCWSTR, 0, (REG_DWORD) as u32, core::ptr::addr_of_mut!((*(*This).pCurrentConfig).Metric) as LPBYTE, core::mem::size_of::<DWORD>() as u32);
        } else {
            RegDeleteValueW(hKey, w!("InterfaceMetric") as LPCWSTR);
        }

        if !(*pCurrentConfig).pDNS.is_null() {
            RegSetValueExW(hKey, w!("RegisterAdapterName") as LPCWSTR, 0, (REG_DWORD) as u32, core::ptr::addr_of_mut!((*(*(*This).pCurrentConfig).pDNS).RegisterAdapterName) as LPBYTE, core::mem::size_of::<DWORD>() as u32);
            RegSetValueExW(hKey, w!("RegistrationEnabled") as LPCWSTR, 0, (REG_DWORD) as u32, core::ptr::addr_of_mut!((*(*(*This).pCurrentConfig).pDNS).RegistrationEnabled) as LPBYTE, core::mem::size_of::<DWORD>() as u32);
            RegSetValueExW(hKey, w!("Domain") as LPCWSTR, 0, (REG_SZ) as u32, (*(*(*This).pCurrentConfig).pDNS).szDomain.as_mut_ptr() as LPBYTE,
                       ((wcslen((*(*(*This).pCurrentConfig).pDNS).szDomain.as_ptr())+1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
        }
        RegSetValueExW(hKey, w!("EnableDHCP") as LPCWSTR, 0, (REG_DWORD) as u32, core::ptr::addr_of_mut!((*pCurrentConfig).DhcpEnabled) as LPBYTE, core::mem::size_of::<DWORD>() as u32);
        if (*pCurrentConfig).DhcpEnabled != 0 {
            RegSetValueExW(hKey, w!("IPAddress") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, w!("0.0.0.0\0") as LPBYTE, 9 * core::mem::size_of::<u16>() as u32);
            RegSetValueExW(hKey, w!("SubnetMask") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, w!("0.0.0.0\0") as LPBYTE, 9 * core::mem::size_of::<u16>() as u32);
            if (*pOldConfig).DhcpEnabled == 0 {
                DeleteIPAddress((*(*pOldConfig).Ip).NTEContext);

                dwSize = 0;
                if GetIpForwardTable(core::ptr::null_mut(), &mut dwSize, (FALSE) as i32) == ERROR_INSUFFICIENT_BUFFER {
                    let mut Index: DWORD;
                    let pIpForwardTable = CoTaskMemAlloc((dwSize as usize) as SIZE_T) as PMIB_IPFORWARDTABLE;
                    if !pIpForwardTable.is_null() {
                        if GetIpForwardTable(pIpForwardTable, &mut dwSize, (FALSE) as i32) == NO_ERROR {
                            Index = 0;
                            while Index < (*pIpForwardTable).dwNumEntries {
                                if (*(*pIpForwardTable).table.as_ptr().add(Index as usize)).dwForwardIfIndex == (*pOldConfig).Index &&
                                    (*(*pIpForwardTable).table.as_ptr().add(Index as usize)).dwForwardDest == 0 {
                                    DeleteIpForwardEntry((*pIpForwardTable).table.as_mut_ptr().add(Index as usize));
                                }
                                Index += 1;
                            }
                        }
                        CoTaskMemFree(pIpForwardTable as LPVOID);
                    }
                }
            }
        } else {
            if (*pOldConfig).DhcpEnabled != 0 && DhcpCApiInitialize(&mut DhcpApiVersion) == NO_ERROR {
                DhcpStaticRefreshParams((*pCurrentConfig).Index,
                                        htonl((*(*pCurrentConfig).Ip).IpAddress),
                                        htonl((*(*pCurrentConfig).Ip).u.Subnetmask));

                DhcpCApiCleanup();
            } else {
                DeleteIPAddress((*(*pOldConfig).Ip).NTEContext);

                AddIPAddress(htonl((*(*pCurrentConfig).Ip).IpAddress),
                             htonl((*(*pCurrentConfig).Ip).u.Subnetmask),
                             (*pCurrentConfig).Index,
                             core::ptr::addr_of_mut!((*(*pCurrentConfig).Ip).NTEContext),
                             &mut NTEInstance);
            }

            pStr = CreateMultiSzString((*pCurrentConfig).Ip, IPADDR, &mut dwSize, (FALSE) as i32);
            if !pStr.is_null() {
                RegSetValueExW(hKey, w!("IPAddress") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, pStr as LPBYTE, dwSize);
                CoTaskMemFree(pStr as LPVOID);
            }

            pStr = CreateMultiSzString((*pCurrentConfig).Ip, SUBMASK, &mut dwSize, (FALSE) as i32);
            if !pStr.is_null() {
                RegSetValueExW(hKey, w!("SubnetMask") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, pStr as LPBYTE, dwSize);
                CoTaskMemFree(pStr as LPVOID);
            }

            dwSize = 0;
            if GetIpForwardTable(core::ptr::null_mut(), &mut dwSize, (FALSE) as i32) == ERROR_INSUFFICIENT_BUFFER {
                let mut Index: DWORD;
                let pIpForwardTable = CoTaskMemAlloc((dwSize as usize) as SIZE_T) as PMIB_IPFORWARDTABLE;
                if !pIpForwardTable.is_null() {
                    if GetIpForwardTable(pIpForwardTable, &mut dwSize, (FALSE) as i32) == NO_ERROR {
                        Index = 0;
                        while Index < (*pIpForwardTable).dwNumEntries {
                            if (*(*pIpForwardTable).table.as_ptr().add(Index as usize)).dwForwardIfIndex == (*pOldConfig).Index &&
                                (*(*pIpForwardTable).table.as_ptr().add(Index as usize)).dwForwardDest == 0 {
                                DeleteIpForwardEntry((*pIpForwardTable).table.as_mut_ptr().add(Index as usize));
                            }
                            Index += 1;
                        }
                    }
                    CoTaskMemFree(pIpForwardTable as LPVOID);
                }
            }
        }

        if !(*pCurrentConfig).Gw.is_null() {
            let mut RouterMib: MIB_IPFORWARDROW = core::mem::zeroed();
            ZeroMemory!(core::ptr::addr_of_mut!(RouterMib) as *mut core::ffi::c_void, core::mem::size_of::<MIB_IPFORWARDROW>() as usize);

            RouterMib.dwForwardMetric1 = 1;
            RouterMib.dwForwardIfIndex = (*pOldConfig).Index;
            RouterMib.dwForwardNextHop = htonl((*(*pCurrentConfig).Gw).IpAddress);

            if CreateIpForwardEntry(core::ptr::addr_of_mut!(RouterMib)) == NO_ERROR {
                pStr = CreateMultiSzString((*pCurrentConfig).Gw, IPADDR, &mut dwSize, (FALSE) as i32);
                if !pStr.is_null() {
                    RegSetValueExW(hKey, w!("DefaultGateway") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, pStr as LPBYTE, dwSize);
                    CoTaskMemFree(pStr as LPVOID);
                }

                pStr = CreateMultiSzString((*pCurrentConfig).Gw, METRIC, &mut dwSize, (FALSE) as i32);
                if !pStr.is_null() {
                    RegSetValueExW(hKey, w!("DefaultGatewayMetric") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, pStr as LPBYTE, dwSize);
                    CoTaskMemFree(pStr as LPVOID);
                }
            }
        } else {
            RegSetValueExW(hKey, w!("DefaultGateway") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, w!("") as LPBYTE, 1 * core::mem::size_of::<u16>() as u32);
            RegSetValueExW(hKey, w!("DefaultGatewayMetric") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, w!("\0") as LPBYTE, core::mem::size_of::<u16>() as u32 * 2);
        }

        if (*pCurrentConfig).Ns.is_null() || (*pCurrentConfig).AutoconfigActive != 0 {
            RegDeleteValueW(hKey, w!("NameServer") as LPCWSTR);
        } else {
            pStr = CreateMultiSzString((*pCurrentConfig).Ns, IPADDR, &mut dwSize, (TRUE) as i32);
            if !pStr.is_null() {
                RegSetValueExW(hKey, w!("NameServer") as LPCWSTR, 0, (REG_SZ) as u32, pStr as LPBYTE, dwSize);
                CoTaskMemFree(pStr as LPVOID);
            }
        }

        RegCloseKey(hKey);
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentControl_fnApplyPnpChanges(
    iface: *mut INetCfgComponentControl,
    pICallback: *mut INetCfgPnpReconfigCallback,
) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentControl_fnCancelChanges(
    iface: *mut INetCfgComponentControl,
) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn TcpipConfigNotify_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: *const IID,
    ppv: *mut LPVOID,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    let This = CoTaskMemAlloc((core::mem::size_of::<TcpipConfNotifyImpl>()) as SIZE_T) as *mut TcpipConfNotifyImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = &vt_NetCfgComponentPropertyUi as *const INetCfgComponentPropertyUiVtbl as *const INetCfgComponentPropertyUi;
    (*This).lpVtblCompControl = &vt_NetCfgComponentControl as *const INetCfgComponentControlVtbl as *const INetCfgComponentControl;
    (*This).pNCfg = core::ptr::null_mut();
    (*This).pUnknown = core::ptr::null_mut();
    (*This).pNComp = core::ptr::null_mut();
    (*This).pCurrentConfig = core::ptr::null_mut();

    if !(INetCfgComponentPropertyUi_QueryInterface(This as *mut INetCfgComponentPropertyUi, riid, ppv) >= 0) {
        INetCfgComponentPropertyUi_Release(This as *mut INetCfgComponentPropertyUi);
        return E_NOINTERFACE;
    }

    INetCfgComponentPropertyUi_Release(This as *mut INetCfgComponentPropertyUi);
    S_OK
}

// COM vtable tables — generated from tcpipconf_notify.c by vtbl_gen.py
/// tcpipconf_notify.c:3176 `static const INetCfgComponentPropertyUiVtbl vt_NetCfgComponentPropertyUi`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgComponentPropertyUi: INetCfgComponentPropertyUiVtbl = INetCfgComponentPropertyUiVtbl {
    QueryInterface: Some(INetCfgComponentPropertyUi_fnQueryInterface),
    AddRef: Some(INetCfgComponentPropertyUi_fnAddRef),
    Release: Some(INetCfgComponentPropertyUi_fnRelease),
    QueryPropertyUi: Some(INetCfgComponentPropertyUi_fnQueryPropertyUi),
    SetContext: Some(INetCfgComponentPropertyUi_fnSetContext),
    MergePropPages: Some(INetCfgComponentPropertyUi_fnMergePropPages),
    ValidateProperties: Some(INetCfgComponentPropertyUi_fnValidateProperties),
    ApplyProperties: Some(INetCfgComponentPropertyUi_fnApplyProperties),
    CancelProperties: Some(INetCfgComponentPropertyUi_fnCancelProperties),
};

/// tcpipconf_notify.c:3578 `static const INetCfgComponentControlVtbl vt_NetCfgComponentControl`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgComponentControl: INetCfgComponentControlVtbl = INetCfgComponentControlVtbl {
    QueryInterface: Some(INetCfgComponentControl_fnQueryInterface),
    AddRef: Some(INetCfgComponentControl_fnAddRef),
    Release: Some(INetCfgComponentControl_fnRelease),
    Initialize: Some(INetCfgComponentControl_fnInitialize),
    ApplyRegistryChanges: Some(INetCfgComponentControl_fnApplyRegistryChanges),
    ApplyPnpChanges: Some(INetCfgComponentControl_fnApplyPnpChanges),
    CancelChanges: Some(INetCfgComponentControl_fnCancelChanges),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// #include <winnls.h>
// #include <winsock.h>
// #include <iphlpapi.h>
// #include <dhcpcsdk.h>
// #include <dhcpcapi.h>
//
// typedef struct
// {
//     DWORD EnableSecurityFilters;
//     LPWSTR szTCPAllowedPorts;       // KEY: Tcpip\Parameter\{InstanceGuid}\TCPAllowedPorts
//     LPWSTR szUDPAllowedPorts;       // KEY: Tcpip\Parameter\{InstanceGuid}\UDPAllowedPorts
//     LPWSTR szRawIPAllowedProtocols; // KEY: Tcpip\Parameter\{InstanceGuid}\RawIPAllowedProtocols
//     DWORD IPSize;
//     DWORD TCPSize;
//     DWORD UDPSize;
// } TcpFilterSettings;
//
// // KEY: Tcpip\Parameter\{InstanceGuid}\IpAddress | DhcpIpAddress
// // KEY: Tcpip\Parameter\{InstanceGuid}\SubnetMask | DhcpSubnetMask
// // KEY: Tcpip\Parameter\{InstanceGuid}\DefaultGateway | DhcpDefaultGateway
// // KEY: Tcpip\Parameter\{InstanceGuid}\NameServer | DhcpNameServer
// // KEY: Services\NetBT\Parameters\Interfaces\Tcpip_{INSTANCE_GUID}
//
// typedef struct
// {
//     DWORD RegisterAdapterName;
//     DWORD RegistrationEnabled;
//     DWORD UseDomainNameDevolution;
//     WCHAR szDomain[100];
//     LPWSTR szSearchList;
// } TcpipAdvancedDNSDlgSettings;
//
// typedef struct tagIP_ADDR
// {
//     DWORD IpAddress;
//     union
//     {
//         DWORD Subnetmask;
//         USHORT Metric;
//     } u;
//     ULONG NTEContext;
//     struct tagIP_ADDR *Next;
// } IP_ADDR;
//
// typedef enum
// {
//     METRIC = 1,
//     SUBMASK = 2,
//     IPADDR = 3
// } COPY_TYPE;
//
// typedef struct
// {
//     IP_ADDR *Ip;
//     IP_ADDR *Ns;
//     IP_ADDR *Gw;
//
//     UINT DhcpEnabled;
//     UINT AutoconfigActive;
//     DWORD Metric;
//     DWORD Index;
//     TcpFilterSettings *pFilter;
//     TcpipAdvancedDNSDlgSettings *pDNS;
// } TcpipSettings;
//
// typedef struct
// {
//     const INetCfgComponentPropertyUi *lpVtbl;
//     const INetCfgComponentControl *lpVtblCompControl;
//     LONG ref;
//     IUnknown *pUnknown;
//     INetCfg *pNCfg;
//     INetCfgComponent *pNComp;
//     TcpipSettings *pCurrentConfig;
//     CLSID NetCfgInstanceId;
// } TcpipConfNotifyImpl, *LPTcpipConfNotifyImpl;
//
// typedef struct
// {
//     BOOL bAdd;
//     HWND hDlgCtrl;
//     WCHAR szIP[16];
//     UINT Metric;
// } TcpipGwSettings;
//
// typedef struct
// {
//     BOOL bAdd;
//     HWND hDlgCtrl;
//     WCHAR szIP[16];
//     WCHAR szMask[16];
// } TcpipIpSettings;
//
// typedef struct
// {
//     BOOL bAdd;
//     HWND hDlgCtrl;
//     WCHAR szIP[16];
// } TcpipDnsSettings;
//
// typedef struct
// {
//     BOOL bAdd;
//     HWND hDlgCtrl;
//     LPWSTR Suffix;
// } TcpipSuffixSettings;
//
// typedef struct
// {
//     HWND hDlgCtrl;
//     UINT ResId;
//     UINT MaxNum;
// } TcpipPortSettings;
//
// static __inline LPTcpipConfNotifyImpl impl_from_INetCfgComponentControl(INetCfgComponentControl *iface)
// {
//     return (TcpipConfNotifyImpl*)((char *)iface - FIELD_OFFSET(TcpipConfNotifyImpl, lpVtblCompControl));
// }
//
// INT GetSelectedItem(HWND hDlgCtrl);
// HRESULT InitializeTcpipBasicDlgCtrls(HWND hwndDlg, TcpipSettings * pCurSettings);
// VOID InsertColumnToListView(HWND hDlgCtrl, UINT ResId, UINT SubItem, UINT Size);
// INT_PTR StoreTcpipBasicSettings(HWND hwndDlg, TcpipConfNotifyImpl * This, BOOL bApply);
// HRESULT Initialize(TcpipConfNotifyImpl * This);
// UINT GetIpAddressFromStringW(WCHAR *szBuffer);
//
// VOID
// DisplayError(UINT ResTxt, UINT ResTitle, UINT Type)
// {
//     WCHAR szBuffer[300];
//     WCHAR szTitle[100];
//
//     if (LoadStringW(netcfgx_hInstance, ResTxt, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//         szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//     else
//         szBuffer[0] = L'\0';
//
//     if (LoadStringW(netcfgx_hInstance, ResTitle, szTitle, sizeof(szTitle)/sizeof(WCHAR)))
//         szTitle[(sizeof(szTitle)/sizeof(WCHAR))-1] = L'\0';
//     else
//         szTitle[0] = L'\0';
//
//     MessageBoxW(NULL, szBuffer, szTitle, Type);
// }
//
//
// /***************************************************************
//  * TCP/IP Filter Dialog
//  *
//  */
//
// INT_PTR
// __stdcall
// TcpipFilterPortDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     TcpipPortSettings * pPort;
//     UINT Num;
//     LVFINDINFOW find;
//     LVITEMW li;
//     WCHAR szBuffer[100];
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             pPort = (TcpipPortSettings*)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pPort);
//             if (LoadStringW(netcfgx_hInstance, pPort->ResId, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//             {
//                 szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                 SendDlgItemMessageW(hwndDlg, IDC_PORT_DESC, WM_SETTEXT, 0, (LPARAM)szBuffer);
//             }
//
//             if (pPort->MaxNum == 65536)
//                 SendDlgItemMessageW(hwndDlg, IDC_PORT_VAL, EM_LIMITTEXT, 5, 0);
//             else
//                 SendDlgItemMessageW(hwndDlg, IDC_PORT_VAL, EM_LIMITTEXT, 3, 0);
//
//             return TRUE;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == IDCANCEL)
//             {
//                 EndDialog(hwndDlg, FALSE);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_OK)
//             {
//                 pPort = (TcpipPortSettings*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                 Num = GetDlgItemInt(hwndDlg, IDC_PORT_VAL, NULL, TRUE);
//                 if (Num > pPort->MaxNum || Num == 0)
//                 {
//                     if (pPort->MaxNum == 65536)
//                         DisplayError(IDS_PORT_RANGE, IDS_TCPIP, MB_ICONWARNING);
//                     else
//                         DisplayError(IDS_PROT_RANGE, IDS_TCPIP, MB_ICONWARNING);
//
//                     SetFocus(GetDlgItem(hwndDlg, IDC_PORT_VAL));
//                     break;
//                 }
//                 if (GetWindowTextW(GetDlgItem(hwndDlg, IDC_PORT_VAL), szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     ZeroMemory(&find, sizeof(LVFINDINFOW));
//                     find.flags = LVFI_STRING;
//                     find.psz = szBuffer;
//                     if (SendMessageW(pPort->hDlgCtrl, LVM_FINDITEMW, (WPARAM)-1, (LPARAM)&find) == -1)
//                     {
//                         ZeroMemory(&li, sizeof(LVITEMW));
//                         li.mask = LVIF_TEXT;
//                         li.iItem = ListView_GetItemCount(pPort->hDlgCtrl);
//                         li.pszText = szBuffer;
//                         SendMessageW(pPort->hDlgCtrl, LVM_INSERTITEMW, 0, (LPARAM)&li);
//                         EndDialog(hwndDlg, TRUE);
//                         break;
//                     }
//                     DisplayError(IDS_DUP_NUMBER, IDS_PROT_RANGE, MB_ICONWARNING);
//                     SetFocus(GetDlgItem(hwndDlg, IDC_PORT_VAL));
//                     break;
//                 }
//            }
//     }
//     return FALSE;
// }
//
// VOID
// InitFilterListBox(LPWSTR pData, HWND hwndDlg, HWND hDlgCtrl, UINT AllowButton, UINT RestrictButton, UINT AddButton, UINT DelButton)
// {
//     LVITEMW li;
//     LPWSTR pCur;
//     INT iItem;
//
//     if (!pData || !_wtoi(pData))
//     {
//         CheckDlgButton(hwndDlg, AllowButton, BST_CHECKED);
//         EnableWindow(GetDlgItem(hwndDlg, AddButton), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, DelButton), FALSE);
//         return;
//     }
//
//     pCur = pData;
//     memset(&li, 0x0, sizeof(LVITEMW));
//     li.mask = LVIF_TEXT;
//     iItem = 0;
//
//     while(pCur[0])
//     {
//         li.pszText = pCur;
//         li.iItem = iItem;
//         SendMessageW(hDlgCtrl, LVM_INSERTITEMW, 0, (LPARAM)&li);
//         iItem++;
//         pCur += wcslen(pCur) + 1;
//     }
//
//     if (!iItem)
//         CheckDlgButton(hwndDlg, AllowButton, BST_CHECKED);
//     else
//         CheckDlgButton(hwndDlg, RestrictButton, BST_CHECKED);
// }
//
// LPWSTR
// CreateFilterList(
//     HWND hDlgCtrl,
//     LPDWORD Size)
// {
//     INT iCount, iIndex;
//     LVITEMW li;
//     LPWSTR pData, pCur;
//     DWORD dwSize;
//     WCHAR szBuffer[10];
//
//     iCount = ListView_GetItemCount(hDlgCtrl);
//     if (!iCount)
//     {
//         pData = (LPWSTR)CoTaskMemAlloc(3 * sizeof(WCHAR));
//         if (!pData)
//             return NULL;
//         pData[0] = L'0';
//         pData[1] = L'\0';
//         pData[2] = L'\0';
//         *Size = 3 * sizeof(WCHAR);
//         return pData;
//     }
//
//     pData = CoTaskMemAlloc((6 * iCount + 1) * sizeof(WCHAR));
//     if (!pData)
//         return NULL;
//
//     pCur = pData;
//     dwSize = 0;
//     for(iIndex = 0; iIndex < iCount; iIndex++)
//     {
//         ZeroMemory(&li, sizeof(LVITEMW));
//         li.mask = LVIF_TEXT;
//         li.iItem = iIndex;
//         li.pszText = szBuffer;
//         li.cchTextMax = sizeof(szBuffer) /sizeof(WCHAR);
//         if (SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li))
//         {
//             wcscpy(pCur, szBuffer);
//             dwSize += wcslen(szBuffer) + 1;
//             pCur += wcslen(szBuffer) + 1;
//         }
//     }
//     pCur[0] = L'\0';
//     *Size = (dwSize+1) * sizeof(WCHAR);
//     return pData;
// }
//
// TcpFilterSettings *
// StoreTcpipFilterSettings(
//     HWND hwndDlg)
// {
//     TcpFilterSettings * pFilter;
//
//     pFilter = CoTaskMemAlloc(sizeof(TcpFilterSettings));
//     if (!pFilter)
//         return NULL;
//
//     if (IsDlgButtonChecked(hwndDlg, IDC_USE_FILTER) == BST_CHECKED)
//         pFilter->EnableSecurityFilters = TRUE;
//     else
//         pFilter->EnableSecurityFilters = FALSE;
//
//     pFilter->szTCPAllowedPorts = CreateFilterList(GetDlgItem(hwndDlg, IDC_TCP_LIST), &pFilter->TCPSize);
//     pFilter->szUDPAllowedPorts = CreateFilterList(GetDlgItem(hwndDlg, IDC_UDP_LIST), &pFilter->UDPSize);
//     pFilter->szRawIPAllowedProtocols = CreateFilterList(GetDlgItem(hwndDlg, IDC_IP_LIST), &pFilter->IPSize);
//
//     return pFilter;
// }
//
// static
// VOID
// AddItem(
//     HWND hwndDlg,
//     HWND hDlgCtrl,
//     UINT MaxItem,
//     UINT ResId)
// {
//     TcpipPortSettings Port;
//
//     Port.MaxNum = MaxItem;
//     Port.hDlgCtrl = hDlgCtrl;
//     Port.ResId = ResId;
//
//     DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIP_PORT_DLG), hwndDlg, TcpipFilterPortDlg, (LPARAM)&Port);
// }
//
// static
// VOID
// DelItem(
//     HWND hDlgCtrl)
// {
//     INT iIndex = GetSelectedItem(hDlgCtrl);
//
//     if (iIndex != -1)
//     {
//         (void)ListView_DeleteItem(hDlgCtrl, iIndex);
//         return;
//     }
//     DisplayError(IDS_NOITEMSEL, IDS_TCPIP, MB_ICONWARNING);
// }
//
// INT_PTR
// __stdcall
// TcpipFilterSettingsDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     TcpipConfNotifyImpl *pContext;
//     TcpFilterSettings *pFilter;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             pContext = (TcpipConfNotifyImpl*)lParam;
//             InsertColumnToListView(GetDlgItem(hwndDlg, IDC_TCP_LIST), IDS_TCP_PORTS, 0, 100);
//             InsertColumnToListView(GetDlgItem(hwndDlg, IDC_UDP_LIST), IDS_UDP_PORTS, 0, 100);
//             InsertColumnToListView(GetDlgItem(hwndDlg, IDC_IP_LIST), IDS_IP_PROTO, 0, 100);
//             if (pContext->pCurrentConfig->pFilter)
//             {
//                 InitFilterListBox(pContext->pCurrentConfig->pFilter->szTCPAllowedPorts, hwndDlg, GetDlgItem(hwndDlg, IDC_TCP_LIST), IDC_TCP_ALLOW_ALL, IDC_TCP_RESTRICT, IDC_TCP_ADD, IDC_TCP_DEL);
//                 InitFilterListBox(pContext->pCurrentConfig->pFilter->szUDPAllowedPorts, hwndDlg, GetDlgItem(hwndDlg, IDC_UDP_LIST), IDC_UDP_ALLOW_ALL, IDC_UDP_RESTRICT, IDC_UDP_ADD, IDC_UDP_DEL);
//                 InitFilterListBox(pContext->pCurrentConfig->pFilter->szRawIPAllowedProtocols, hwndDlg, GetDlgItem(hwndDlg, IDC_IP_LIST), IDC_IP_ALLOW_ALL, IDC_IP_RESTRICT, IDC_IP_ADD, IDC_IP_DEL);
//                 if (pContext->pCurrentConfig->pFilter->EnableSecurityFilters)
//                     CheckDlgButton(hwndDlg, IDC_USE_FILTER, BST_CHECKED);
//              }
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pContext);
//             return TRUE;
//         case WM_COMMAND:
//             if (HIWORD(wParam) == BN_CLICKED)
//             {
//                 switch (LOWORD(wParam))
//                 {
//                     case IDC_TCP_ALLOW_ALL:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_TCP_ALLOW_ALL) == BST_CHECKED)
//                         {
//                             CheckDlgButton(hwndDlg, IDC_TCP_RESTRICT, BST_UNCHECKED);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_LIST), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_ADD), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_DEL), FALSE);
//                         }
//                         break;
//                     case IDC_TCP_RESTRICT:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_TCP_RESTRICT) == BST_CHECKED)
//                         {
//                             CheckDlgButton(hwndDlg, IDC_TCP_ALLOW_ALL, BST_UNCHECKED);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_LIST), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_ADD), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_TCP_DEL), TRUE);
//                         }
//                         break;
//                     case IDC_UDP_ALLOW_ALL:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_UDP_ALLOW_ALL) == BST_CHECKED)
//                         {
//                             CheckDlgButton(hwndDlg, IDC_UDP_RESTRICT, BST_UNCHECKED);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_LIST), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_ADD), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_DEL), FALSE);
//                         }
//                         break;
//                     case IDC_UDP_RESTRICT:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_UDP_RESTRICT) == BST_CHECKED)
//                         {
//                             CheckDlgButton(hwndDlg, IDC_UDP_ALLOW_ALL, BST_UNCHECKED);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_LIST), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_ADD), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_UDP_DEL), TRUE);
//                         }
//                         break;
//                     case IDC_IP_ALLOW_ALL:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_IP_ALLOW_ALL) == BST_CHECKED)
//                         {
//                             CheckDlgButton(hwndDlg, IDC_IP_RESTRICT, BST_UNCHECKED);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IP_LIST), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IP_ADD), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IP_DEL), FALSE);
//                         }
//                         break;
//                     case IDC_IP_RESTRICT:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_IP_RESTRICT) == BST_CHECKED)
//                         {
//                             CheckDlgButton(hwndDlg, IDC_IP_ALLOW_ALL, BST_UNCHECKED);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IP_LIST), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IP_ADD), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IP_DEL), TRUE);
//                         }
//                         break;
//                     case IDC_USE_FILTER:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_USE_FILTER) == BST_UNCHECKED)
//                             DisplayError(IDS_DISABLE_FILTER, IDS_TCPIP, MB_OK);
//
//                         break;
//                 }
//             }
//             switch(LOWORD(wParam))
//             {
//                 case IDC_OK:
//                     pContext = (TcpipConfNotifyImpl*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                     pFilter = StoreTcpipFilterSettings(hwndDlg);
//                     if (pFilter)
//                     {
//                         if (pContext->pCurrentConfig->pFilter)
//                         {
//                             CoTaskMemFree(pContext->pCurrentConfig->pFilter->szTCPAllowedPorts);
//                             CoTaskMemFree(pContext->pCurrentConfig->pFilter->szUDPAllowedPorts);
//                             CoTaskMemFree(pContext->pCurrentConfig->pFilter->szRawIPAllowedProtocols);
//                             CoTaskMemFree(pContext->pCurrentConfig->pFilter);
//                         }
//                         pContext->pCurrentConfig->pFilter = pFilter;
//                     }
//                     EndDialog(hwndDlg, (INT_PTR)TRUE);
//                     break;
//                 case IDCANCEL:
//                     EndDialog(hwndDlg, FALSE);
//                     break;
//                 case IDC_TCP_ADD:
//                     AddItem(hwndDlg, GetDlgItem(hwndDlg, IDC_TCP_LIST), 65536, IDS_TCP_PORTS);
//                     break;
//                 case IDC_TCP_DEL:
//                     DelItem(GetDlgItem(hwndDlg, IDC_TCP_LIST));
//                     break;
//                 case IDC_UDP_ADD:
//                     AddItem(hwndDlg, GetDlgItem(hwndDlg, IDC_UDP_LIST), 65536, IDS_UDP_PORTS);
//                     break;
//                 case IDC_UDP_DEL:
//                     DelItem(GetDlgItem(hwndDlg, IDC_UDP_LIST));
//                     break;
//                 case IDC_IP_ADD:
//                     AddItem(hwndDlg, GetDlgItem(hwndDlg, IDC_IP_LIST), 256, IDS_IP_PROTO);
//                     break;
//                 case IDC_IP_DEL:
//                     DelItem(GetDlgItem(hwndDlg, IDC_IP_LIST));
//                     break;
//                 default:
//                     break;
//             }
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// HPROPSHEETPAGE
// InitializePropertySheetPage(LPWSTR resname, DLGPROC dlgproc, LPARAM lParam, LPWSTR szTitle)
// {
//     PROPSHEETPAGEW ppage;
//
//     memset(&ppage, 0x0, sizeof(PROPSHEETPAGEW));
//     ppage.dwSize = sizeof(PROPSHEETPAGEW);
//     ppage.dwFlags = PSP_DEFAULT;
//     *ppage.u.pszTemplate.as_mut() = resname;
//     ppage.pfnDlgProc = dlgproc;
//     ppage.lParam = lParam;
//     ppage.hInstance = netcfgx_hInstance;
//     if (szTitle)
//     {
//         ppage.dwFlags |= PSP_USETITLE;
//         ppage.pszTitle = szTitle;
//     }
//     return CreatePropertySheetPageW(&ppage);
// }
//
// /***************************************************************
//  * TCP/IP Advanced Option Dialog
//  *
//  */
//
// VOID
// InitializeTcpipAdvancedOptDlg(
//     HWND hwndDlg,
//     TcpipConfNotifyImpl * This)
// {
//     WCHAR szText[500];
//     /* store context */
//     SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)This);
//
//     if (LoadStringW(netcfgx_hInstance, IDS_TCPFILTER, szText, sizeof(szText)/sizeof(WCHAR)))
//     {
//         szText[(sizeof(szText)/sizeof(WCHAR))-1] = L'\0';
//         if (SendDlgItemMessageW(hwndDlg, IDC_OPTLIST, LB_ADDSTRING, 0, (LPARAM)szText) != LB_ERR)
//             SendDlgItemMessageW(hwndDlg, IDC_OPTLIST, LB_SETCURSEL, 0, 0);
//     }
//
//     if (LoadStringW(netcfgx_hInstance, IDS_TCPFILTERDESC, szText, sizeof(szText)/sizeof(WCHAR)))
//     {
//         szText[(sizeof(szText)/sizeof(WCHAR))-1] = L'\0';
//         SendDlgItemMessageW(hwndDlg, IDC_OPTDESC, WM_SETTEXT, 0, (LPARAM)szText);
//     }
// }
//
//
//
// INT_PTR
// __stdcall
// TcpipAdvancedOptDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     TcpipConfNotifyImpl * This;
//     LPPROPSHEETPAGE page;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             page = (LPPROPSHEETPAGE)lParam;
//             This = (TcpipConfNotifyImpl*)page->lParam;
//             InitializeTcpipAdvancedOptDlg(hwndDlg, This);
//             return TRUE;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == IDC_OPTPROP)
//             {
//                 DialogBoxParamW(netcfgx_hInstance,
//                                 MAKEINTRESOURCEW(IDD_TCPIP_FILTER_DLG),
//                                 GetParent(hwndDlg),
//                                 Some(TcpipFilterSettingsDlg),
//                                 (LPARAM)GetWindowLongPtr(hwndDlg, DWLP_USER));
//                 break;
//             }
//     }
//     return FALSE;
// }
//
// VOID
// InsertColumnToListView(
//     HWND hDlgCtrl,
//     UINT ResId,
//     UINT SubItem,
//     UINT Size)
// {
//     WCHAR szBuffer[200];
//     LVCOLUMNW lc;
//
//     if (!LoadStringW(netcfgx_hInstance, ResId, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//         return;
//
//     memset(&lc, 0, sizeof(LV_COLUMN) );
//     lc.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_FMT;
//     lc.iSubItem   = SubItem;
//     lc.fmt = LVCFMT_FIXED_WIDTH;
//     lc.cx         = Size;
//     lc.cchTextMax = wcslen(szBuffer);
//     lc.pszText    = szBuffer;
//
//     (void)SendMessageW(hDlgCtrl, LVM_INSERTCOLUMNW, SubItem, (LPARAM)&lc);
// }
//
// VOID
// InsertIpAddressToListView(
//     HWND hDlgCtrl,
//     IP_ADDR * pAddr,
//     BOOL bSubMask)
// {
//     WCHAR szBuffer[70];
//     DWORD dwIpAddr;
//     UINT itemCount = 0;
//     LVITEMW li;
//
//     while(pAddr)
//     {
//         ZeroMemory(&li, sizeof(li));
//         li.mask = LVIF_TEXT;
//         li.iItem = itemCount;
//         li.iSubItem = 0;
//         dwIpAddr = pAddr->IpAddress;
//         swprintf(szBuffer, L"%lu.%lu.%lu.%lu",
//                  FIRST_IPADDRESS(dwIpAddr), SECOND_IPADDRESS(dwIpAddr), THIRD_IPADDRESS(dwIpAddr), FOURTH_IPADDRESS(dwIpAddr));
//
//         li.pszText = szBuffer;
//         li.iItem = SendMessageW(hDlgCtrl, LVM_INSERTITEMW, 0, (LPARAM)&li);
//         if (li.iItem  != -1)
//         {
//             if (bSubMask)
//             {
//                 dwIpAddr = pAddr->u.Subnetmask;
//                 swprintf(szBuffer, L"%lu.%lu.%lu.%lu",
//                          FIRST_IPADDRESS(dwIpAddr), SECOND_IPADDRESS(dwIpAddr), THIRD_IPADDRESS(dwIpAddr), FOURTH_IPADDRESS(dwIpAddr));
//             }
//             else
//             {
//                 if (pAddr->u.Metric)
//                     swprintf(szBuffer, L"%u", pAddr->u.Metric);
//                 else
//                     LoadStringW(netcfgx_hInstance, IDS_AUTOMATIC, szBuffer, sizeof(szBuffer)/sizeof(WCHAR));
//             }
//
//             li.mask = LVIF_TEXT;
//             li.iSubItem = 1;
//             li.pszText = szBuffer;
//             SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, (LPARAM)&li);
//         }
//         itemCount++;
//         pAddr = pAddr->Next;
//     }
// }
//
// VOID
// InitializeTcpipAdvancedIpDlg(
//     HWND hwndDlg,
//     TcpipConfNotifyImpl * This)
// {
//     RECT rect;
//     LVITEMW li;
//     WCHAR szBuffer[100];
//
//     InsertColumnToListView(GetDlgItem(hwndDlg, IDC_IPLIST), IDS_IPADDR, 0, 100);
//     GetClientRect(GetDlgItem(hwndDlg, IDC_IPLIST), &rect);
//     InsertColumnToListView(GetDlgItem(hwndDlg, IDC_IPLIST), IDS_SUBMASK, 1, (rect.right - rect.left - 100));
//
//     if (This->pCurrentConfig->DhcpEnabled)
//     {
//         if (LoadStringW(netcfgx_hInstance, IDS_DHCPACTIVE, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//         {
//             ZeroMemory(&li, sizeof(LVITEMW));
//             li.mask = LVIF_TEXT;
//             li.pszText = szBuffer;
//             SendDlgItemMessageW(hwndDlg, IDC_IPLIST, LVM_INSERTITEMW, 0, (LPARAM)&li);
//         }
//         EnableWindow(GetDlgItem(hwndDlg, IDC_IPADD), FALSE);
//     }
//     else
//     {
//         InsertIpAddressToListView(GetDlgItem(hwndDlg, IDC_IPLIST), This->pCurrentConfig->Ip, TRUE);
//     }
//
//     EnableWindow(GetDlgItem(hwndDlg, IDC_IPMOD), FALSE);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_IPDEL), FALSE);
//
//     InsertColumnToListView(GetDlgItem(hwndDlg, IDC_GWLIST), IDS_GATEWAY, 0, 100);
//     GetClientRect(GetDlgItem(hwndDlg, IDC_IPLIST), &rect);
//     InsertColumnToListView(GetDlgItem(hwndDlg, IDC_GWLIST), IDS_METRIC, 1, (rect.right - rect.left - 100));
//
//     InsertIpAddressToListView(GetDlgItem(hwndDlg, IDC_GWLIST), This->pCurrentConfig->Gw, FALSE);
//
//     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMOD), FALSE);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_GWDEL), FALSE);
//
//     SendDlgItemMessageW(hwndDlg, IDC_IFMETRIC, EM_LIMITTEXT, 4, 0);
//     if (This->pCurrentConfig->Metric)
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), TRUE);
//         SetDlgItemInt(hwndDlg, IDC_IFMETRIC, This->pCurrentConfig->Metric, FALSE);
//     }
//     else
//     {
//         CheckDlgButton(hwndDlg, IDC_IFAUTOMETRIC, BST_CHECKED);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), FALSE);
//     }
// }
//
// INT_PTR
// __stdcall
// TcpipAdvGwDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     WCHAR szBuffer[70];
//     TcpipGwSettings *pGwSettings;
//     DWORD dwIpAddr;
//     LPNMIPADDRESS lpnmipa;
//     LVFINDINFOW find;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             pGwSettings = (TcpipGwSettings *)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)lParam);
//
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//             if (pGwSettings->bAdd)
//             {
//                 if (LoadStringW(netcfgx_hInstance, IDS_ADD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//                 EnableWindow(GetDlgItem(hwndDlg, IDC_OK), FALSE);
//                 CheckDlgButton(hwndDlg, IDC_GWAUTOMETRIC, BST_CHECKED);
//             }
//             else
//             {
//                 if (LoadStringW(netcfgx_hInstance, IDS_MOD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//
//                 SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETADDRESS, 0, (LPARAM)GetIpAddressFromStringW(pGwSettings->szIP));
//
//                 if (pGwSettings->Metric)
//                 {
//                     SetDlgItemInt(hwndDlg, IDC_GWMETRIC, pGwSettings->Metric, FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), TRUE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), TRUE);
//                 }
//                 else
//                 {
//                     CheckDlgButton(hwndDlg, IDC_GWAUTOMETRIC, BST_CHECKED);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), FALSE);
//                 }
//             }
//             return TRUE;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == IDC_GWAUTOMETRIC)
//             {
//                 if (IsDlgButtonChecked(hwndDlg, IDC_GWAUTOMETRIC) == BST_CHECKED)
//                 {
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), FALSE);
//                     SendDlgItemMessageW(hwndDlg, IDC_GWMETRIC, WM_SETTEXT, 0, (LPARAM)L"");
//                 }
//                 else
//                 {
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRIC), TRUE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMETRICTXT), TRUE);
//                 }
//                 break;
//             }
//             else if (LOWORD(wParam) == IDCANCEL)
//             {
//                 EndDialog(hwndDlg, FALSE);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_OK)
//             {
//                 if (SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//                 {
//                     pGwSettings = (TcpipGwSettings*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_GETTEXT, 16, (LPARAM)pGwSettings->szIP);
//
//                     ZeroMemory(&find, sizeof(LVFINDINFOW));
//                     find.flags = LVFI_STRING;
//                     find.psz = pGwSettings->szIP;
//
//                     if (IsDlgButtonChecked(hwndDlg, IDC_GWAUTOMETRIC) == BST_UNCHECKED)
//                         pGwSettings->Metric = GetDlgItemInt(hwndDlg, IDC_GWMETRIC, NULL, FALSE);
//                     else
//                         pGwSettings->Metric = 0;
//
//
//                     if (SendMessageW(pGwSettings->hDlgCtrl, LVM_FINDITEMW, (WPARAM)-1, (LPARAM)&find) == -1)
//                     {
//                         EndDialog(hwndDlg, TRUE);
//                         break;
//                     }
//                     if (!pGwSettings->bAdd)
//                     {
//                         EndDialog(hwndDlg, FALSE);
//                         break;
//                     }
//                     DisplayError(IDS_DUP_GW, IDS_TCPIP, MB_ICONINFORMATION);
//                 }
//                 break;
//             }
//             break;
//         case WM_NOTIFY:
//             lpnmipa = (LPNMIPADDRESS) lParam;
//             if (lpnmipa->hdr.code == IPN_FIELDCHANGED)
//             {
//                 if (lpnmipa->hdr.idFrom == IDC_IPADDR)
//                 {
//                     if (SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_OK), TRUE);
//                 }
//             }
//             break;
//     }
//     return FALSE;
// }
//
// BOOL
// GetGWListEntry(HWND hDlgCtrl, INT Index, TcpipGwSettings * pGwSettings)
// {
//     LVITEMW li;
//     WCHAR szBuffer[30];
//     BOOL bRet;
//
//     ZeroMemory(&li, sizeof(LVITEMW));
//     li.mask = LVIF_TEXT;
//     li.cchTextMax = 16;
//     li.pszText = pGwSettings->szIP;
//     li.iItem = Index;
//
//     if (!SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li))
//         return FALSE;
//     li.pszText = szBuffer;
//     li.cchTextMax = 30;
//     li.iSubItem = 1;
//
//     bRet = SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li);
//     if (bRet)
//     {
//         pGwSettings->Metric = _wtoi(szBuffer);
//     }
//
//     return bRet;
// }
//
// INT_PTR
// __stdcall
// TcpipAddIpDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     LPNMIPADDRESS lpnmipa;
//     DWORD dwIpAddr;
//     TcpipIpSettings *pIpSettings;
//     WCHAR szBuffer[50];
//     LVFINDINFOW find;
//     LRESULT lResult;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             pIpSettings = (TcpipIpSettings*)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)lParam);
//
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 0, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//             if (pIpSettings->bAdd)
//             {
//                 if (LoadStringW(netcfgx_hInstance, IDS_ADD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//                 EnableWindow(GetDlgItem(hwndDlg, IDC_OK), FALSE);
//             }
//             else
//             {
//                 if (LoadStringW(netcfgx_hInstance, IDS_MOD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//
//                 SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETADDRESS, 0, (LPARAM)GetIpAddressFromStringW(pIpSettings->szIP));
//                 SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)GetIpAddressFromStringW(pIpSettings->szMask));
//             }
//             return TRUE;
//         case WM_NOTIFY:
//             lpnmipa = (LPNMIPADDRESS) lParam;
//             if (lpnmipa->hdr.code == IPN_FIELDCHANGED)
//             {
//                 if (lpnmipa->hdr.idFrom == IDC_IPADDR)
//                 {
//                     if (SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//                     {
//                         if (dwIpAddr <= MAKEIPADDRESS(127, 255, 255, 255))
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)MAKEIPADDRESS(255, 0, 0, 0));
//                         else if (dwIpAddr <= MAKEIPADDRESS(191, 255, 255, 255))
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)MAKEIPADDRESS(255, 255, 0, 0));
//                         else if (dwIpAddr <= MAKEIPADDRESS(223, 255, 255, 255))
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)MAKEIPADDRESS(255, 255, 255, 0));
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_OK), TRUE);
//                      }
//                 }
//             }
//             break;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == IDC_OK)
//             {
//                 pIpSettings = (TcpipIpSettings*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                 SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_GETTEXT, 16, (LPARAM)pIpSettings->szIP);
//                 SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, WM_GETTEXT, 16, (LPARAM)pIpSettings->szMask);
//
//                 ZeroMemory(&find, sizeof(LVFINDINFOW));
//                 find.flags = LVFI_STRING;
//                 find.psz = pIpSettings->szIP;
//                 lResult = SendMessageW(pIpSettings->hDlgCtrl, LVM_FINDITEMW, (WPARAM)-1, (LPARAM)&find);
//
//                 if (lResult == -1)
//                 {
//                     EndDialog(hwndDlg, TRUE);
//                     break;
//                 }
//                 else if (!pIpSettings->bAdd)
//                 {
//                     EndDialog(hwndDlg, FALSE);
//                     break;
//                 }
//                 DisplayError(IDS_DUP_IPADDR, IDS_TCPIP, MB_ICONINFORMATION);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDCANCEL)
//             {
//                 EndDialog(hwndDlg, FALSE);
//             }
//             break;
//     }
//     return FALSE;
// }
//
// BOOL
// VerifyDNSSuffix(
//     LPWSTR szName)
// {
//     UINT Index;
//     UINT Length = wcslen(szName);
//
//     for(Index = 0; Index < Length; Index++)
//         if (iswalnum(szName[Index]) == 0 && szName[Index] != '.' && szName[Index] != '-')
//             return FALSE;
//
//     return TRUE;
// }
//
// INT_PTR
// __stdcall
// TcpipAddSuffixDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     WCHAR szBuffer[100];
//     TcpipSuffixSettings * pSettings;
//     LRESULT lLength;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             pSettings = (TcpipSuffixSettings*)lParam;
//             if (!pSettings->bAdd)
//             {
//                 SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_SETTEXT, 0, (LPARAM)pSettings->Suffix);
//                 if (LoadStringW(netcfgx_hInstance, IDS_MOD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//                 CoTaskMemFree(pSettings->Suffix);
//                 pSettings->Suffix = NULL;
//             }
//             else
//             {
//                 if (LoadStringW(netcfgx_hInstance, IDS_ADD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//             }
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pSettings);
//             return TRUE;
//         case WM_COMMAND:
//            if (LOWORD(wParam) == IDCANCEL)
//            {
//                EndDialog(hwndDlg, FALSE);
//                break;
//            }
//            else if (LOWORD(wParam) == IDC_OK)
//            {
//                lLength = SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_GETTEXTLENGTH, 0, 0);
//                if (lLength)
//                {
//                    pSettings = (TcpipSuffixSettings*) GetWindowLongPtr(hwndDlg, DWLP_USER);
//                    pSettings->Suffix = (LPWSTR)CoTaskMemAlloc((lLength + 1)* sizeof(WCHAR));
//                    if (pSettings->Suffix)
//                    {
//                        SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_GETTEXT, lLength + 1, (LPARAM)pSettings->Suffix);
//                        if (SendMessageW(pSettings->hDlgCtrl, LB_FINDSTRING, 0, (LPARAM)pSettings->Suffix) != LB_ERR)
//                        {
//                            DisplayError(IDS_DUP_SUFFIX, IDS_TCPIP, MB_ICONWARNING);
//                            CoTaskMemFree(pSettings->Suffix);
//                            break;
//                        }
//
//                        if (!VerifyDNSSuffix(pSettings->Suffix))
//                        {
//                            DisplayError(IDS_DOMAIN_SUFFIX, IDS_TCPIP, MB_ICONWARNING);
//                            CoTaskMemFree(pSettings->Suffix);
//                            break;
//                        }
//                        EndDialog(hwndDlg, TRUE);
//                    }
//                }
//                break;
//            }
//     }
//     return FALSE;
// }
//
//
// INT
// GetSelectedItem(HWND hDlgCtrl)
// {
//     LVITEMW li;
//     UINT iItemCount, iIndex;
//
//     iItemCount = ListView_GetItemCount(hDlgCtrl);
//     if (!iItemCount)
//         return -1;
//
//     for (iIndex = 0; iIndex < iItemCount; iIndex++)
//     {
//         ZeroMemory(&li, sizeof(LVITEMW));
//         li.mask = LVIF_STATE;
//         li.stateMask = (UINT)-1;
//         li.iItem = iIndex;
//         if (SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li))
//         {
//             if (li.state & LVIS_SELECTED)
//                 return iIndex;
//         }
//     }
//     return -1;
// }
//
//
// BOOL
// GetIPListEntry(HWND hDlgCtrl, INT Index, TcpipIpSettings * pIpSettings)
// {
//     LVITEMW li;
//
//     ZeroMemory(&li, sizeof(LVITEMW));
//     li.mask = LVIF_TEXT;
//     li.cchTextMax = 16;
//     li.pszText = pIpSettings->szIP;
//     li.iItem = Index;
//
//     if (!SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li))
//         return FALSE;
//
//     ZeroMemory(&li, sizeof(LVITEMW));
//     li.mask = LVIF_TEXT;
//     li.cchTextMax = 16;
//     li.pszText = pIpSettings->szMask;
//     li.iSubItem = 1;
//     li.iItem = Index;
//
//     return SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li);
// }
//
// VOID
// DeleteItemFromList(HWND hDlgCtrl)
// {
//     LVITEMW li;
//
//     memset(&li, 0x0, sizeof(LVITEMW));
//     li.iItem = GetSelectedItem(hDlgCtrl);
//     if (li.iItem < 0)
//     {
//         DisplayError(IDS_NOITEMSEL, IDS_TCPIP, MB_ICONINFORMATION);
//         SetFocus(hDlgCtrl);
//     }
//     else
//     {
//         (void)ListView_DeleteItem(hDlgCtrl, li.iItem);
//     }
// }
//
// UINT
// GetIpAddressFromStringW(
//     WCHAR * szBuffer)
// {
//     DWORD dwIpAddr = 0;
//     INT Val;
//     UINT Index = 3;
//     LPWSTR pLast = szBuffer;
//     LPWSTR pNext = szBuffer;
//
//
//     while((pNext = wcschr(pNext, L'.')))
//     {
//         pNext[0] = L'\0';
//         Val = _wtoi(pLast);
//         dwIpAddr |= (Val << Index * 8);
//         Index--;
//         pNext++;
//         pLast = pNext;
//     }
//     dwIpAddr |= _wtoi(pLast);
//
//     return dwIpAddr;
// }
//
// UINT
// GetIpAddressFromStringA(
//     char * sBuffer)
// {
//     WCHAR szIp[16];
//
//     if (MultiByteToWideChar(CP_ACP, 0, sBuffer, -1, szIp, 16))
//     {
//         szIp[15] = L'\0';
//        return GetIpAddressFromStringW(szIp);
//     }
//     return (UINT)-1;
// }
//
//
// VOID
// FreeIPAddr(IP_ADDR *pAddr)
// {
//     IP_ADDR *pNext;
//
//     if (!pAddr)
//         return;
//
//     while(pAddr)
//     {
//         pNext = pAddr->Next;
//         CoTaskMemFree(pAddr);
//         pAddr = pNext;
//     }
// }
//
// BOOL
// GetListViewItem(HWND hDlgCtrl, UINT Index, UINT SubIndex, WCHAR * szBuffer, UINT BufferSize)
// {
//     LVITEMW li;
//
//     ZeroMemory(&li, sizeof(LVITEMW));
//     li.mask = LVIF_TEXT;
//     li.pszText = szBuffer;
//     li.iItem = Index;
//     li.iSubItem = SubIndex;
//     li.cchTextMax = BufferSize;
//     return SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li);
// }
//
// VOID
// StoreIPSettings(
//     HWND hDlgCtrl,
//     TcpipConfNotifyImpl * This,
//     BOOL bSubmask)
// {
//     WCHAR szBuffer[30];
//
//     INT iIndex, iCount;
//     IP_ADDR *pCur, *pLast;
//
//     iCount = ListView_GetItemCount(hDlgCtrl);
//     if (!iCount)
//     {
//         return;
//     }
//
//     pLast = NULL;
//     for(iIndex = 0; iIndex < iCount; iIndex++)
//     {
//         if (GetListViewItem(hDlgCtrl, iIndex, 0, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//         {
//             pCur = (IP_ADDR*)CoTaskMemAlloc(sizeof(IP_ADDR));
//             if (!pCur)
//                 break;
//             ZeroMemory(pCur, sizeof(IP_ADDR));
//
//             szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//             pCur->IpAddress = GetIpAddressFromStringW(szBuffer);
//
//             if (GetListViewItem(hDlgCtrl, iIndex, 1, szBuffer, sizeof(szBuffer)/sizeof(WCHAR) ))
//             {
//                 szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                 if (bSubmask)
//                     pCur->u.Subnetmask = GetIpAddressFromStringW(szBuffer);
//                 else
//                     pCur->u.Metric  = _wtoi(szBuffer);
//             }
//
//             if (!pLast)
//             {
//                 if (bSubmask)
//                     This->pCurrentConfig->Ip = pCur;
//                 else
//                     This->pCurrentConfig->Gw = pCur;
//             }
//             else
//             {
//                 pLast->Next = pCur;
//             }
//
//             pLast = pCur;
//         }
//     }
// }
//
//
// INT_PTR
// __stdcall
// TcpipAdvancedIpDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     TcpipConfNotifyImpl * This;
//     LPPROPSHEETPAGE page;
//     INT_PTR res;
//     WCHAR szBuffer[200];
//     LPPSHNOTIFY lppsn;
//     TcpipGwSettings Gw;
//     TcpipIpSettings Ip;
//
//     LVITEMW li;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             page = (LPPROPSHEETPAGE)lParam;
//             This = (TcpipConfNotifyImpl*)page->lParam;
//             InitializeTcpipAdvancedIpDlg(hwndDlg, This);
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (INT_PTR)This);
//             return TRUE;
//         case WM_NOTIFY:
//             lppsn = (LPPSHNOTIFY) lParam;
//             if (lppsn->hdr.code == LVN_ITEMCHANGED)
//             {
//                 LPNMLISTVIEW lplv = (LPNMLISTVIEW)lParam;
//                 BOOL bEnable;
//
//                 if (lplv->hdr.idFrom == IDC_IPLIST)
//                 {
//                     This = (TcpipConfNotifyImpl*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//                     bEnable = ((lplv->uNewState & LVIS_SELECTED) != 0) &&
//                               (!This->pCurrentConfig->DhcpEnabled);
//
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_IPMOD), bEnable);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_IPDEL), bEnable);
//                 }
//                 else if (lplv->hdr.idFrom == IDC_GWLIST)
//                 {
//                     bEnable = ((lplv->uNewState & LVIS_SELECTED) != 0);
//
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWMOD), bEnable);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_GWDEL), bEnable);
//                 }
//             }
//             else if (lppsn->hdr.code == PSN_KILLACTIVE)
//             {
//                 This = (TcpipConfNotifyImpl*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                 if (!This->pCurrentConfig->DhcpEnabled && ListView_GetItemCount(GetDlgItem(hwndDlg, IDC_IPLIST)) == 0)
//                 {
//                     DisplayError(IDS_NO_IPADDR_SET, IDS_TCPIP, MB_ICONWARNING);
//                     SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
//                     return TRUE;
//                 }
//
//                 if (IsDlgButtonChecked(hwndDlg, IDC_IFAUTOMETRIC) != BST_CHECKED)
//                 {
//                     UINT val = GetDlgItemInt(hwndDlg, IDC_IFMETRIC, NULL, FALSE);
//                     if ((val < 1) || (val > 9999))
//                     {
//                         DisplayError(IDS_METRIC_RANGE, IDS_TCPIP, MB_ICONWARNING);
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
//                         return TRUE;
//                     }
//                 }
//             }
//             else if (lppsn->hdr.code == PSN_APPLY)
//             {
//                 This = (TcpipConfNotifyImpl*) GetWindowLongPtr(hwndDlg, DWLP_USER);
//                 FreeIPAddr(This->pCurrentConfig->Gw);
//                 This->pCurrentConfig->Gw = NULL;
//                 FreeIPAddr(This->pCurrentConfig->Ip);
//                 This->pCurrentConfig->Ip = NULL;
//                 StoreIPSettings(GetDlgItem(hwndDlg, IDC_IPLIST), This, TRUE);
//                 StoreIPSettings(GetDlgItem(hwndDlg, IDC_GWLIST), This, FALSE);
//
//                 if (IsDlgButtonChecked(hwndDlg, IDC_IFAUTOMETRIC) == BST_CHECKED)
//                     This->pCurrentConfig->Metric = 0;
//                 else
//                     This->pCurrentConfig->Metric = GetDlgItemInt(hwndDlg, IDC_IFMETRIC, NULL, FALSE);
//
//                 SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR);
//                 return TRUE;
//             }
//             break;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == IDC_IFAUTOMETRIC)
//             {
//                 if (IsDlgButtonChecked(hwndDlg, IDC_IFAUTOMETRIC) == BST_CHECKED)
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), FALSE);
//                 else
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_IFMETRIC), TRUE);
//             }
//             else if (LOWORD(wParam) == IDC_IPADD)
//             {
//                 Ip.bAdd = TRUE;
//                 Ip.hDlgCtrl = GetDlgItem(hwndDlg, IDC_IPLIST);
//                 res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPADDIP_DLG), hwndDlg, TcpipAddIpDlg, (LPARAM)&Ip);
//                 if (res)
//                 {
//                     memset(&li, 0x0, sizeof(LVITEMW));
//                     li.mask = LVIF_TEXT | LVIF_PARAM;
//                     li.iItem = ListView_GetItemCount(GetDlgItem(hwndDlg, IDC_IPLIST));
//                     li.pszText = Ip.szIP;
//                     li.iItem = SendDlgItemMessageW(hwndDlg, IDC_IPLIST, LVM_INSERTITEMW, 0, (LPARAM)&li);
//                     if (li.iItem  != -1)
//                     {
//                         li.mask = LVIF_TEXT;
//                         li.iSubItem = 1;
//                         li.pszText = Ip.szMask;
//                         SendDlgItemMessageW(hwndDlg, IDC_IPLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                     }
//                 }
//             }
//             else if (LOWORD(wParam) == IDC_IPMOD)
//             {
//                 memset(&li, 0x0, sizeof(LVITEMW));
//                 li.iItem = GetSelectedItem(GetDlgItem(hwndDlg, IDC_IPLIST));
//                 if (li.iItem < 0)
//                 {
//                     /* no item selected */
//                     DisplayError(IDS_NOITEMSEL, IDS_TCPIP, MB_ICONINFORMATION);
//                     SetFocus(GetDlgItem(hwndDlg, IDC_IPLIST));
//                     break;
//                 }
//                 Ip.bAdd = FALSE;
//                 Ip.hDlgCtrl = GetDlgItem(hwndDlg, IDC_IPLIST);
//                 if (GetIPListEntry(GetDlgItem(hwndDlg, IDC_IPLIST), li.iItem, &Ip))
//                 {
//                     res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPADDIP_DLG), hwndDlg, TcpipAddIpDlg, (LPARAM)&Ip);
//                     if (res)
//                     {
//                             li.mask = LVIF_TEXT;
//                             li.pszText = Ip.szIP;
//                             SendDlgItemMessageW(hwndDlg, IDC_IPLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                             li.pszText = Ip.szMask;
//                             li.iSubItem = 1;
//                             SendDlgItemMessageW(hwndDlg, IDC_IPLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                     }
//                 }
//             }
//             else if (LOWORD(wParam) == IDC_IPDEL)
//             {
//                 DeleteItemFromList(GetDlgItem(hwndDlg, IDC_IPLIST));
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_GWADD)
//             {
//                 Gw.bAdd = TRUE;
//                 Gw.hDlgCtrl = GetDlgItem(hwndDlg, IDC_GWLIST);
//                 res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPGW_DLG), hwndDlg, TcpipAdvGwDlg, (LPARAM)&Gw);
//                 if (res)
//                 {
//                     memset(&li, 0x0, sizeof(LVITEMW));
//                     li.mask = LVIF_TEXT;
//                     li.iItem = ListView_GetItemCount(GetDlgItem(hwndDlg, IDC_GWLIST));
//                     li.pszText = Gw.szIP;
//                     li.iItem = SendDlgItemMessageW(hwndDlg, IDC_GWLIST, LVM_INSERTITEMW, 0, (LPARAM)&li);
//                     if (li.iItem >= 0)
//                     {
//                         if (Gw.Metric)
//                         {
//                             swprintf(szBuffer, L"%u", Gw.Metric);
//                             li.iSubItem = 1;
//                             li.pszText = szBuffer;
//                             SendDlgItemMessageW(hwndDlg, IDC_GWLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                         }
//                         else
//                         {
//                             if (LoadStringW(netcfgx_hInstance, IDS_AUTOMATIC, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                             {
//                                 szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                                 li.iSubItem = 1;
//                                 li.pszText = szBuffer;
//                                 SendDlgItemMessageW(hwndDlg, IDC_GWLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                             }
//                         }
//                     }
//                 }
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_GWMOD)
//             {
//                 memset(&li, 0x0, sizeof(LVITEMW));
//                 li.iItem = GetSelectedItem(GetDlgItem(hwndDlg, IDC_GWLIST));
//                 if (li.iItem < 0)
//                 {
//                     /* no item selected */
//                     DisplayError(IDS_NOITEMSEL, IDS_TCPIP, MB_ICONINFORMATION);
//                     SetFocus(GetDlgItem(hwndDlg, IDC_GWLIST));
//                     break;
//                 }
//                 if (GetGWListEntry(GetDlgItem(hwndDlg, IDC_GWLIST), li.iItem, &Gw))
//                 {
//                     Gw.bAdd = FALSE;
//                     Gw.hDlgCtrl = GetDlgItem(hwndDlg, IDC_GWLIST);
//                     res = DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPGW_DLG), hwndDlg, TcpipAdvGwDlg, (LPARAM)&Gw);
//                     if (res)
//                     {
//                         li.mask = LVIF_TEXT;
//                         li.pszText = Gw.szIP;
//                         (void)SendDlgItemMessageW(hwndDlg, IDC_GWLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                         if (Gw.Metric)
//                         {
//                             swprintf(szBuffer, L"%u", Gw.Metric);
//                             li.iSubItem = 1;
//                             li.pszText = szBuffer;
//                             SendDlgItemMessageW(hwndDlg, IDC_GWLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                         }
//                         else
//                         {
//                             if (LoadStringW(netcfgx_hInstance, IDS_AUTOMATIC, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                             {
//                                 szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                                 li.iSubItem = 1;
//                                 li.pszText = szBuffer;
//                                 SendDlgItemMessageW(hwndDlg, IDC_GWLIST, LVM_SETITEMW, 0, (LPARAM)&li);
//                             }
//                         }
//                     }
//                 }
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_GWDEL)
//             {
//                 DeleteItemFromList(GetDlgItem(hwndDlg, IDC_GWLIST));
//                 break;
//             }
//             break;
//     }
//     return FALSE;
// }
//
// INT_PTR
// __stdcall
// TcpipAddDNSDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     TcpipDnsSettings * pSettings;
//     WCHAR szBuffer[100];
//     DWORD dwIpAddr;
//     LPNMIPADDRESS lpnmipa;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             pSettings = (TcpipDnsSettings*)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)lParam);
//             if (!pSettings->bAdd)
//             {
//                 if (LoadStringW(netcfgx_hInstance, IDS_MOD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//                 SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_SETTEXT, 0, (LPARAM)pSettings->szIP);
//                 EnableWindow(GetDlgItem(hwndDlg, IDC_OK), TRUE);
//             }
//             else
//             {
//                 if (LoadStringW(netcfgx_hInstance, IDS_ADD, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//                 {
//                     szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//                     SendDlgItemMessageW(hwndDlg, IDC_OK, WM_SETTEXT, 0, (LPARAM)szBuffer);
//                 }
//                 EnableWindow(GetDlgItem(hwndDlg, IDC_OK), FALSE);
//             }
//             return TRUE;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == IDCANCEL)
//             {
//                 EndDialog(hwndDlg, FALSE);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_OK)
//             {
//                 pSettings = (TcpipDnsSettings*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                 SendDlgItemMessageW(hwndDlg, IDC_IPADDR, WM_GETTEXT, 16, (LPARAM)pSettings->szIP);
//                 if (SendMessageW(pSettings->hDlgCtrl, LB_FINDSTRING, 0, (LPARAM)pSettings->szIP) == LB_ERR)
//                 {
//                     if (pSettings->bAdd)
//                         SendMessageW(pSettings->hDlgCtrl, LB_ADDSTRING, 0, (LPARAM)pSettings->szIP);
//                     EndDialog(hwndDlg, TRUE);
//                     break;
//                 }
//                 if (!pSettings->bAdd)
//                 {
//                     EndDialog(hwndDlg, FALSE);
//                     break;
//                 }
//                 DisplayError(IDS_DUP_SUFFIX, IDS_TCPIP, MB_ICONERROR);
//                 break;
//             }
//             break;
//         case WM_NOTIFY:
//             lpnmipa = (LPNMIPADDRESS) lParam;
//             if (lpnmipa->hdr.code == IPN_FIELDCHANGED)
//             {
//                 if (lpnmipa->hdr.idFrom == IDC_IPADDR)
//                 {
//                     if (SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_OK), TRUE);
//                 }
//             }
//             break;
//     }
//     return FALSE;
// }
//
//
//
// VOID
// InitializeTcpipAdvancedDNSDlg(
//     HWND hwndDlg,
//     TcpipConfNotifyImpl * This)
// {
//     WCHAR szBuffer[200];
//     LPWSTR pFirst, pSep, pList;
//     IP_ADDR * pAddr;
//     DWORD dwIpAddr;
//
//     /* insert DNS addresses */
//     pAddr = This->pCurrentConfig->Ns;
//     while(pAddr)
//     {
//         dwIpAddr = pAddr->IpAddress;
//         swprintf(szBuffer, L"%lu.%lu.%lu.%lu",
//                  FIRST_IPADDRESS(dwIpAddr), SECOND_IPADDRESS(dwIpAddr), THIRD_IPADDRESS(dwIpAddr), FOURTH_IPADDRESS(dwIpAddr));
//
//         SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_ADDSTRING, 0, (LPARAM)szBuffer);
//         pAddr = pAddr->Next;
//     }
//     SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_SETCURSEL, 0, 0);
//
//     if (!This->pCurrentConfig->pDNS)
//         return;
//
//     if (This->pCurrentConfig->pDNS->RegisterAdapterName)
//         CheckDlgButton(hwndDlg, IDC_REGSUFFIX, BST_CHECKED);
//     else
//         EnableWindow(GetDlgItem(hwndDlg, IDC_USESUFFIX), FALSE);
//
//     if (This->pCurrentConfig->pDNS->RegistrationEnabled)
//         CheckDlgButton(hwndDlg, IDC_USESUFFIX, BST_CHECKED);
//
//     if (This->pCurrentConfig->pDNS->szDomain[0])
//         SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_SETTEXT, 0, (LPARAM)szBuffer);
//
//     if (This->pCurrentConfig->pDNS->UseDomainNameDevolution)
//         CheckDlgButton(hwndDlg, IDC_TOPPRIMSUFFIX, BST_CHECKED);
//
//     if (!This->pCurrentConfig->pDNS->szSearchList || (wcslen(This->pCurrentConfig->pDNS->szSearchList) == 0))
//     {
//         CheckDlgButton(hwndDlg, IDC_PRIMSUFFIX, BST_CHECKED);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXADD), FALSE);
//
//         return;
//     }
//
//     pList = This->pCurrentConfig->pDNS->szSearchList;
//     if (wcslen(pList))
//     {
//         pFirst = pList;
//         do
//         {
//             pSep = wcschr(pFirst, L',');
//             if (pSep)
//             {
//                 pSep[0] = L'\0';
//                 SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_ADDSTRING, 0, (LPARAM)pFirst);
//                 pFirst = pSep + 1;
//                 pSep[0] = L',';
//             }
//             else
//             {
//                 SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_ADDSTRING, 0, (LPARAM)pFirst);
//                 break;
//             }
//         }while(TRUE);
//
//         EnableWindow(GetDlgItem(hwndDlg, IDC_TOPPRIMSUFFIX), FALSE);
//         CheckDlgButton(hwndDlg, IDC_SELSUFFIX, BST_CHECKED);
//         SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_SETCURSEL, 0, 0);
//     }
// }
//
// VOID
// ToggleUpDown(HWND hwndDlg, HWND hDlgCtrl, UINT UpButton, UINT DownButton, UINT ModButton, UINT DelButton)
// {
//     LRESULT lResult, lCount;
//
//     lResult = SendMessageW(hDlgCtrl, LB_GETCURSEL, 0, 0);
//     lCount = SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0);
//     if (lResult != LB_ERR)
//     {
//         if (lResult == 0)
//             EnableWindow(GetDlgItem(hwndDlg, UpButton), FALSE);
//         else
//              EnableWindow(GetDlgItem(hwndDlg, UpButton), TRUE);
//
//         if (lResult < lCount -1)
//              EnableWindow(GetDlgItem(hwndDlg, DownButton), TRUE);
//         else
//              EnableWindow(GetDlgItem(hwndDlg, DownButton), FALSE);
//     }
//
//     if (lCount)
//     {
//         EnableWindow(GetDlgItem(hwndDlg, ModButton), TRUE);
//         EnableWindow(GetDlgItem(hwndDlg, DelButton), TRUE);
//     }
//     else
//     {
//         EnableWindow(GetDlgItem(hwndDlg, ModButton), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, DelButton), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, UpButton), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, DownButton), FALSE);
//     }
// }
//
// VOID
// MoveItem(
//     HWND hDlgCtrl,
//     INT pos)
// {
//     WCHAR szBuffer[100];
//     LRESULT lResult;
//
//     lResult = SendMessageW(hDlgCtrl, LB_GETCURSEL, 0, 0);
//     if (lResult != LB_ERR)
//     {
//         if (SendMessageW(hDlgCtrl, LB_GETTEXTLEN, (WPARAM)lResult, 0) < sizeof(szBuffer)/sizeof(WCHAR) - 1)
//         {
//             if (SendMessageW(hDlgCtrl, LB_GETTEXT, (WPARAM)lResult, (LPARAM)szBuffer) != LB_ERR)
//             {
//                 SendMessageW(hDlgCtrl, LB_DELETESTRING, (WPARAM)lResult, 0);
//                 SendMessageW(hDlgCtrl, LB_INSERTSTRING, (WPARAM)lResult + pos, (LPARAM)szBuffer);
//                 SendMessageW(hDlgCtrl, LB_SETCURSEL, (WPARAM)lResult + pos, 0);
//             }
//         }
//     }
// }
// VOID
// RemoveItem(
//     HWND hDlgCtrl)
// {
//     LRESULT lResult, lCount;
//
//     lResult = SendMessageW(hDlgCtrl, LB_GETCURSEL, 0, 0);
//     if (lResult != LB_ERR)
//     {
//         SendMessageW(hDlgCtrl, LB_DELETESTRING, (WPARAM)lResult, 0);
//         lCount = SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0);
//         if (lResult + 1 < lCount)
//             SendMessageW(hDlgCtrl, LB_SETCURSEL, (WPARAM)lResult, 0);
//         else
//             SendMessageW(hDlgCtrl, LB_SETCURSEL, (WPARAM)lCount-1, 0);
//     }
// }
//
// LPWSTR
// GetListViewEntries(
//     HWND hDlgCtrl)
// {
//     DWORD dwSize;
//     INT iCount, iIndex;
//     INT_PTR lResult;
//     LPWSTR pszSearchList, pItem;
//
//     iCount = SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0);
//     if (!iCount || iCount == LB_ERR)
//         return NULL; //BUGBUG
//
//     dwSize = 0;
//
//     for (iIndex = 0; iIndex < iCount; iIndex++)
//     {
//         lResult = SendMessageW(hDlgCtrl, LB_GETTEXTLEN, iIndex, 0);
//         if (lResult == LB_ERR)
//             return NULL;
//
//         dwSize += lResult + 1;
//     }
//
//     pszSearchList = (LPWSTR)CoTaskMemAlloc((dwSize + 1) * sizeof(WCHAR));
//     if (!pszSearchList)
//         return NULL;
//
//     pItem = pszSearchList;
//     for (iIndex = 0; iIndex < iCount; iIndex++)
//     {
//         lResult = SendMessageW(hDlgCtrl, LB_GETTEXT, iIndex, (LPARAM)pItem);
//         if (lResult == LB_ERR)
//         {
//             CoTaskMemFree(pszSearchList);
//             return NULL;
//         }
//         dwSize -= lResult + 1;
//         pItem += wcslen(pItem);
//         if (iIndex != iCount -1)
//         {
//             pItem[0] = L',';
//             pItem++;
//         }
//     }
//     pItem[0] = L'\0';
//     return pszSearchList;
// }
//
// VOID
// StoreDNSSettings(
//     HWND hDlgCtrl,
//     TcpipConfNotifyImpl *This)
// {
//     INT iCount, iIndex;
//     WCHAR Ip[16];
//     IP_ADDR *pCur, *pLast;
//
//     FreeIPAddr(This->pCurrentConfig->Ns);
//     This->pCurrentConfig->Ns = NULL;
//
//     iCount = SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0);
//     if (!iCount || iCount == LB_ERR)
//     {
//         return;
//     }
//
//     pLast = NULL;
//     for(iIndex = 0; iIndex < iCount; iIndex++)
//     {
//         if (SendMessageW(hDlgCtrl, LB_GETTEXT, iIndex, (LPARAM)Ip) == LB_ERR)
//             break;
//
//         pCur = CoTaskMemAlloc(sizeof(IP_ADDR));
//         if (!pCur)
//             break;
//         ZeroMemory(pCur, sizeof(IP_ADDR));
//         pCur->IpAddress = GetIpAddressFromStringW(Ip);
//
//         if (!pLast)
//             This->pCurrentConfig->Ns = pCur;
//         else
//             pLast->Next = pCur;
//
//         pLast = pCur;
//         pCur = pCur->Next;
//     }
//     This->pCurrentConfig->AutoconfigActive = FALSE;
// }
//
// INT_PTR
// __stdcall
// TcpipAdvancedDnsDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     TcpipConfNotifyImpl * This;
//     LPPROPSHEETPAGE page;
//     TcpipDnsSettings Dns;
//     LRESULT lIndex, lLength;
//     TcpipSuffixSettings Suffix;
//     LPPSHNOTIFY lppsn;
//     WCHAR szSuffix[100];
//     WCHAR szFormat[200];
//     WCHAR szBuffer[300];
//
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             page = (LPPROPSHEETPAGE)lParam;
//             This = (TcpipConfNotifyImpl*)page->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (INT_PTR)This);
//             InitializeTcpipAdvancedDNSDlg(hwndDlg, This);
//             ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), IDC_DNSADDRUP, IDC_DNSADDRDOWN, IDC_DNSADDRMOD, IDC_DNSADDRDEL);
//             ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//             return TRUE;
//         case WM_NOTIFY:
//             lppsn = (LPPSHNOTIFY) lParam;
//             if (lppsn->hdr.code == PSN_KILLACTIVE)
//             {
//                 if (IsDlgButtonChecked(hwndDlg, IDC_SELSUFFIX) == BST_CHECKED &&
//                     SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_GETCOUNT, 0, 0) == 0)
//                 {
//                     DisplayError(IDS_NO_SUFFIX, IDS_TCPIP, MB_ICONWARNING);
//                     SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
//                     return TRUE;
//                 }
//                 if (SendDlgItemMessageW(hwndDlg, IDC_SUFFIX, WM_GETTEXT, sizeof(szSuffix)/sizeof(WCHAR), (LPARAM)szSuffix))
//                 {
//                     szSuffix[(sizeof(szSuffix)/sizeof(WCHAR))-1] = L'\0';
//                     if (VerifyDNSSuffix(szSuffix) == FALSE)
//                     {
//                         if (LoadStringW(netcfgx_hInstance, IDS_DNS_SUFFIX, szFormat, sizeof(szFormat)/sizeof(WCHAR)))
//                         {
//                             szFormat[(sizeof(szFormat)/sizeof(WCHAR))-1] = L'\0';
//                             swprintf(szBuffer, szFormat, szSuffix);
//                             if (LoadStringW(netcfgx_hInstance, IDS_TCPIP, szFormat, sizeof(szFormat)/sizeof(WCHAR)))
//                                 szFormat[(sizeof(szFormat)/sizeof(WCHAR))-1] = L'\0';
//                             else
//                                 szFormat[0] = L'\0';
//
//                             MessageBoxW(hwndDlg, szBuffer, szFormat, MB_ICONWARNING);
//                             SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
//                             SetFocus(GetDlgItem(hwndDlg, IDC_SUFFIX));
//                             return TRUE;
//                         }
//                     }
//                 }
//             }
//             else if (lppsn->hdr.code == PSN_APPLY)
//             {
//                  This = (TcpipConfNotifyImpl*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                  if (!This->pCurrentConfig->pDNS)
//                    break;
//
//                  StoreDNSSettings(GetDlgItem(hwndDlg, IDC_DNSADDRLIST), This);
//                  if (IsDlgButtonChecked(hwndDlg, IDC_PRIMSUFFIX) == BST_CHECKED)
//                  {
//                      CoTaskMemFree(This->pCurrentConfig->pDNS->szSearchList);
//                      This->pCurrentConfig->pDNS->szSearchList = NULL;
//                      if (IsDlgButtonChecked(hwndDlg, IDC_TOPPRIMSUFFIX) == BST_CHECKED)
//                          This->pCurrentConfig->pDNS->UseDomainNameDevolution = TRUE;
//                      else
//                          This->pCurrentConfig->pDNS->UseDomainNameDevolution = FALSE;
//                  }
//                  else
//                  {
//                      CoTaskMemFree(This->pCurrentConfig->pDNS->szSearchList);
//                      This->pCurrentConfig->pDNS->szSearchList = NULL;
//                      This->pCurrentConfig->pDNS->UseDomainNameDevolution = FALSE;
//                      This->pCurrentConfig->pDNS->szSearchList = GetListViewEntries(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
//                  }
//
//                  if (IsDlgButtonChecked(hwndDlg, IDC_REGSUFFIX) == BST_CHECKED)
//                  {
//                      This->pCurrentConfig->pDNS->RegisterAdapterName = TRUE;
//                      if (IsDlgButtonChecked(hwndDlg, IDC_USESUFFIX) == BST_CHECKED)
//                          This->pCurrentConfig->pDNS->RegistrationEnabled = TRUE;
//                      else
//                          This->pCurrentConfig->pDNS->RegistrationEnabled = FALSE;
//                  }
//                  else
//                  {
//                      This->pCurrentConfig->pDNS->RegisterAdapterName = FALSE;
//                      This->pCurrentConfig->pDNS->RegistrationEnabled = FALSE;
//                  }
//             }
//             break;
//         case WM_COMMAND:
//             if (LOWORD(wParam) == IDC_DNSADDRLIST && HIWORD(wParam) == LBN_SELCHANGE)
//             {
//                 ToggleUpDown(hwndDlg, (HWND)lParam, IDC_DNSADDRUP, IDC_DNSADDRDOWN, IDC_DNSADDRMOD, IDC_DNSADDRDEL);
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSSUFFIXLIST && HIWORD(wParam) == LBN_SELCHANGE)
//             {
//                 ToggleUpDown(hwndDlg, (HWND)lParam, IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_PRIMSUFFIX && HIWORD(wParam) == BN_CLICKED)
//             {
//                 if (SendMessageW((HWND)lParam, BM_GETCHECK, 0, 0) == BST_CHECKED)
//                 {
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXUP), FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXDOWN), FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXADD), FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXMOD), FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXDEL), FALSE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_TOPPRIMSUFFIX), TRUE);
//                     SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_RESETCONTENT, 0, 0);
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 }
//             }
//             else if (LOWORD(wParam) == IDC_SELSUFFIX && HIWORD(wParam) == BN_CLICKED)
//             {
//                 if (SendMessageW((HWND)lParam, BM_GETCHECK, 0, 0) == BST_CHECKED)
//                 {
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_DNSSUFFIXADD), TRUE);
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_TOPPRIMSUFFIX), FALSE);
//                     ToggleUpDown(hwndDlg, (HWND)lParam, IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 }
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_REGSUFFIX && HIWORD(wParam) == BN_CLICKED)
//             {
//                 if (SendMessageW((HWND)lParam, BM_GETCHECK, 0, 0) == BST_CHECKED)
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_USESUFFIX), TRUE);
//                 else
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_USESUFFIX), FALSE);
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//             }
//             else if (LOWORD(wParam) == IDC_DNSADDRUP && HIWORD(wParam) == BN_CLICKED)
//             {
//                 MoveItem(GetDlgItem(hwndDlg, IDC_DNSADDRLIST), -1);
//                 ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), IDC_DNSADDRUP, IDC_DNSADDRDOWN, IDC_DNSADDRMOD, IDC_DNSADDRDEL);
//                 SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSADDRDOWN && HIWORD(wParam) == BN_CLICKED)
//             {
//                 MoveItem(GetDlgItem(hwndDlg, IDC_DNSADDRLIST), 1);
//                 ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), IDC_DNSADDRUP, IDC_DNSADDRDOWN, IDC_DNSADDRMOD, IDC_DNSADDRDEL);
//                 SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSSUFFIXUP && HIWORD(wParam) == BN_CLICKED)
//             {
//                 MoveItem(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), -1);
//                 ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//                 SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSSUFFIXDOWN && HIWORD(wParam) == BN_CLICKED)
//             {
//                 MoveItem(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), 1);
//                 ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//                 SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSADDRDEL && HIWORD(wParam) == BN_CLICKED)
//             {
//                 RemoveItem(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
//                 ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), IDC_DNSADDRUP, IDC_DNSADDRDOWN, IDC_DNSADDRMOD, IDC_DNSADDRDEL);
//                 SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSSUFFIXDEL && HIWORD(wParam) == BN_CLICKED)
//             {
//                 RemoveItem(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
//                 ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//                 SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSADDRADD && HIWORD(wParam) == BN_CLICKED)
//             {
//                  Dns.bAdd = TRUE;
//                  Dns.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSADDRLIST);
//                  if (DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPDNS_DLG), NULL, TcpipAddDNSDlg, (LPARAM)&Dns))
//                  {
//                      ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSADDRLIST), IDC_DNSADDRUP, IDC_DNSADDRDOWN, IDC_DNSADDRMOD, IDC_DNSADDRDEL);
//                      PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                  }
//                  break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSADDRMOD && HIWORD(wParam) == BN_CLICKED)
//             {
//                  lIndex = SendDlgItemMessage(hwndDlg, IDC_DNSADDRLIST, LB_GETCURSEL, 0, 0);
//                  if (lIndex != LB_ERR)
//                  {
//                      Dns.bAdd = FALSE;
//                      Dns.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSADDRLIST);
//                      SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_GETTEXT, (WPARAM)lIndex, (LPARAM)Dns.szIP);
//                      if (DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPDNS_DLG), NULL, TcpipAddDNSDlg, (LPARAM)&Dns))
//                      {
//                          SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_DELETESTRING, lIndex, 0);
//                          SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_INSERTSTRING, lIndex, (LPARAM)Dns.szIP);
//                          SendDlgItemMessageW(hwndDlg, IDC_DNSADDRLIST, LB_SETCURSEL, lIndex, 0);
//                          SetFocus(GetDlgItem(hwndDlg, IDC_DNSADDRLIST));
//                          PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                      }
//                  }
//                  break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSSUFFIXADD && HIWORD(wParam) == BN_CLICKED)
//             {
//                 Suffix.bAdd = TRUE;
//                 Suffix.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST);
//                 Suffix.Suffix = NULL;
//                 if (DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPSUFFIX_DLG), NULL, TcpipAddSuffixDlg, (LPARAM)&Suffix))
//                 {
//                     ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//                     lIndex = SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_ADDSTRING, 0, (LPARAM)Suffix.Suffix);
//                     if (lIndex != LB_ERR)
//                         SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_SETCURSEL, lIndex, 0);
//                     SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     CoTaskMemFree(Suffix.Suffix);
//                 }
//                 break;
//             }
//             else if (LOWORD(wParam) == IDC_DNSSUFFIXMOD && HIWORD(wParam) == BN_CLICKED)
//             {
//                 lIndex = SendDlgItemMessage(hwndDlg, IDC_DNSSUFFIXLIST, LB_GETCURSEL, 0, 0);
//                 if (lIndex != LB_ERR)
//                 {
//                     Suffix.bAdd = FALSE;
//                     Suffix.hDlgCtrl = GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST);
//                     lLength = SendMessageW(Suffix.hDlgCtrl, LB_GETTEXTLEN, lIndex, 0);
//                     if (lLength != LB_ERR)
//                     {
//                         Suffix.Suffix = (LPWSTR)CoTaskMemAlloc((lLength + 1) * sizeof(WCHAR));
//                         if (Suffix.Suffix)
//                         {
//                             SendMessageW(Suffix.hDlgCtrl, LB_GETTEXT, lIndex, (LPARAM)Suffix.Suffix);
//                             Suffix.Suffix[lLength] = L'\0';
//                             if (DialogBoxParamW(netcfgx_hInstance, MAKEINTRESOURCEW(IDD_TCPIPSUFFIX_DLG), NULL, TcpipAddSuffixDlg, (LPARAM)&Suffix))
//                             {
//                                 if (Suffix.Suffix)
//                                 {
//                                     SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_DELETESTRING, lIndex, 0);
//                                     SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_INSERTSTRING, lIndex, (LPARAM)Suffix.Suffix);
//                                     SendDlgItemMessageW(hwndDlg, IDC_DNSSUFFIXLIST, LB_SETCURSEL, lIndex, 0);
//                                     SetFocus(GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST));
//                                     CoTaskMemFree(Suffix.Suffix);
//                                 }
//                                 ToggleUpDown(hwndDlg, GetDlgItem(hwndDlg, IDC_DNSSUFFIXLIST), IDC_DNSSUFFIXUP, IDC_DNSSUFFIXDOWN, IDC_DNSSUFFIXMOD, IDC_DNSSUFFIXDEL);
//                                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                             }
//                         }
//                     }
//                  }
//                  break;
//             }
//     }
//     return FALSE;
// }
//
// static int __stdcall
// PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam)
// {
//     // NOTE: This callback is needed to set large icon correctly.
//     HICON hIcon;
//     switch (uMsg)
//     {
//         case PSCB_INITIALIZED:
//         {
//             hIcon = LoadIconW(netcfgx_hInstance, MAKEINTRESOURCEW(IDI_NETWORK));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// VOID
// LaunchAdvancedTcpipSettings(
//     HWND hwndDlg,
//     TcpipConfNotifyImpl * This)
// {
//     PROPSHEETHEADERW pinfo;
//     HPROPSHEETPAGE hppages[3];
//     WCHAR szBuffer[100];
//
//     hppages[0] = InitializePropertySheetPage(MAKEINTRESOURCEW(IDD_TCPIP_ADVIP_DLG), TcpipAdvancedIpDlg, (LPARAM)This, NULL);
//     hppages[1] = InitializePropertySheetPage(MAKEINTRESOURCEW(IDD_TCPIP_ADVDNS_DLG), TcpipAdvancedDnsDlg, (LPARAM)This, NULL);
//     hppages[2] = InitializePropertySheetPage(MAKEINTRESOURCEW(IDD_TCPIP_ADVOPT_DLG), TcpipAdvancedOptDlg, (LPARAM)This, NULL);
//
//
//     if (LoadStringW(netcfgx_hInstance, IDS_TCPIP, szBuffer, sizeof(szBuffer)/sizeof(WCHAR)))
//         szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//     else
//         szBuffer[0] = L'\0';
//
//     ZeroMemory(&pinfo, sizeof(PROPSHEETHEADERW));
//     pinfo.dwSize = sizeof(PROPSHEETHEADERW);
//     pinfo.dwFlags = PSH_NOCONTEXTHELP | PSH_PROPTITLE | PSH_NOAPPLYNOW |
//                     PSH_USEICONID | PSH_USECALLBACK;
//     pinfo.u3.phpage = hppages;
//     pinfo.nPages = 3;
//     pinfo.hwndParent = hwndDlg;
//     pinfo.hInstance = netcfgx_hInstance;
//     pinfo.pszCaption = szBuffer;
//     pinfo.u.pszIcon = MAKEINTRESOURCEW(IDI_NETWORK);
//     pinfo.pfnCallback = PropSheetProc;
//
//     StoreTcpipBasicSettings(hwndDlg, This, FALSE);
//     if (PropertySheetW(&pinfo) > 0)
//     {
//         InitializeTcpipBasicDlgCtrls(hwndDlg, This->pCurrentConfig);
//         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//     }
// }
//
// HRESULT
// InitializeTcpipAltDlgCtrls(
//     HWND hwndDlg,
//     TcpipSettings *pCurSettings)
// {
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 0, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     if (pCurSettings->DhcpEnabled)
//     {
//         CheckRadioButton(hwndDlg, IDC_USEDHCP, IDC_NODHCP, IDC_USEDHCP);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), FALSE);
//     }
//     else
//     {
//         CheckRadioButton(hwndDlg, IDC_USEDHCP, IDC_NODHCP, IDC_NODHCP);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), TRUE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), TRUE);
//
//         if (pCurSettings->Ip)
//         {
//             /* Set current ip address */
//             SendDlgItemMessageA(hwndDlg, IDC_IPADDR, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ip->IpAddress);
//             /* Set current hostmask */
//             SendDlgItemMessageA(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ip->u.Subnetmask);
//         }
//     }
//
//     if (pCurSettings->Gw && pCurSettings->Gw->IpAddress)
//     {
//         /* Set current gateway */
//         SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Gw->IpAddress);
//     }
//
//     if (pCurSettings->Ns)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ns->IpAddress);
//         if (pCurSettings->Ns->Next)
//             SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ns->Next->IpAddress);
//         else
//             SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_CLEARADDRESS, 0, 0);
//     }
//     else
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_CLEARADDRESS, 0, 0);
//         SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_CLEARADDRESS, 0, 0);
//     }
//
//     return S_OK;
// }
//
// INT_PTR
// __stdcall
// TcpipAltConfDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     TcpipConfNotifyImpl *This;
//     LPPROPSHEETPAGE page;
//     BOOL bNoDHCP;
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             page = (LPPROPSHEETPAGE)lParam;
//             This = (TcpipConfNotifyImpl*)page->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)This);
//             if (This->pCurrentConfig)
//                 InitializeTcpipAltDlgCtrls(hwndDlg, This->pCurrentConfig);
//             return TRUE;
//         }
//         case WM_COMMAND:
//         {
//             switch (LOWORD(wParam))
//             {
//                 case IDC_USEDHCP:
//                 case IDC_NODHCP:
//                 {
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         bNoDHCP = (IsDlgButtonChecked(hwndDlg, IDC_NODHCP) == BST_CHECKED);
//                         if (bNoDHCP)
//                         {
//                             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_CLEARADDRESS, 0, 0);
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_CLEARADDRESS, 0, 0);
//                             SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_CLEARADDRESS, 0, 0);
//                         }
//
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), bNoDHCP);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), bNoDHCP);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), bNoDHCP);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), bNoDHCP);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), bNoDHCP);
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//                 }
//             }
//             break;
//         }
//     }
//     return FALSE;
// }
//
// VOID
// AddAlternativeDialog(
//     HWND hDlg,
//     TcpipConfNotifyImpl * This)
// {
//     HPROPSHEETPAGE hpage;
//
//     hpage = InitializePropertySheetPage(MAKEINTRESOURCEW(IDD_TCPIP_ALTCF_DLG), TcpipAltConfDlg, (LPARAM)This, NULL);
//     if (!hpage)
//         return;
//
//     SendMessageW(hDlg, PSM_INSERTPAGE, 1, (LPARAM)hpage);
// }
//
// INT_PTR
// StoreTcpipBasicSettings(
//     HWND hwndDlg,
//     TcpipConfNotifyImpl * This,
//     BOOL bApply)
// {
//     DWORD dwIpAddr;
//
//     if (IsDlgButtonChecked(hwndDlg, IDC_NODHCP) == BST_CHECKED)
//     {
//         This->pCurrentConfig->DhcpEnabled = FALSE;
//         if (SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) != 4)
//         {
//             if (bApply)
//             {
//                 DisplayError(IDS_NO_IPADDR_SET, IDS_TCPIP, MB_ICONWARNING);
//                 SetFocus(GetDlgItem(hwndDlg, IDC_IPADDR));
//                 return E_FAIL;
//             }
//         }
//         if (!This->pCurrentConfig->Ip)
//         {
//             This->pCurrentConfig->Ip = (IP_ADDR*)CoTaskMemAlloc(sizeof(IP_ADDR));
//             if (!This->pCurrentConfig->Ip)
//                 return E_OUTOFMEMORY;
//             ZeroMemory(This->pCurrentConfig->Ip, sizeof(IP_ADDR));
//         }
//         This->pCurrentConfig->Ip->IpAddress = dwIpAddr;
//
//         if (SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) != 4)
//         {
//             if (bApply)
//                 DisplayError(IDS_NO_SUBMASK_SET, IDS_TCPIP, MB_ICONWARNING);
//             if (SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//             {
//                 if (dwIpAddr <= MAKEIPADDRESS(127, 255, 255, 255))
//                     dwIpAddr = MAKEIPADDRESS(255, 0, 0, 0);
//                 else if (dwIpAddr <= MAKEIPADDRESS(191, 255, 255, 255))
//                     dwIpAddr = MAKEIPADDRESS(255, 255, 0, 0);
//                 else if (dwIpAddr <= MAKEIPADDRESS(223, 255, 255, 255))
//                     dwIpAddr = MAKEIPADDRESS(255, 255, 255, 0);
//
//                 SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)dwIpAddr);
//             }
//             if (bApply)
//             {
//                 SetFocus(GetDlgItem(hwndDlg, IDC_SUBNETMASK));
//                 return E_FAIL;
//             }
//         }
//         /* store subnetmask */
//         This->pCurrentConfig->Ip->u.Subnetmask = dwIpAddr;
//     }
//     else
//     {
//         This->pCurrentConfig->DhcpEnabled = TRUE;
//     }
//
//     if (SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//     {
//         if (!This->pCurrentConfig->Gw)
//         {
//             This->pCurrentConfig->Gw = (IP_ADDR*)CoTaskMemAlloc(sizeof(IP_ADDR));
//             if (!This->pCurrentConfig->Gw)
//                 return E_OUTOFMEMORY;
//             ZeroMemory(This->pCurrentConfig->Gw, sizeof(IP_ADDR));
//         }
//         /* store default gateway */
//         This->pCurrentConfig->Gw->IpAddress = dwIpAddr;
//     }
//     else
//     {
//         if (This->pCurrentConfig->Gw)
//         {
//             IP_ADDR * pNextGw = This->pCurrentConfig->Gw->Next;
//             CoTaskMemFree(This->pCurrentConfig->Gw);
//             This->pCurrentConfig->Gw = pNextGw;
//         }
//     }
//
//     if (IsDlgButtonChecked(hwndDlg, IDC_FIXEDDNS) == BST_CHECKED)
//     {
//         BOOL bSkip = FALSE;
//         This->pCurrentConfig->AutoconfigActive = FALSE;
//         if (SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//         {
//             if (!This->pCurrentConfig->Ns)
//             {
//                 This->pCurrentConfig->Ns = (IP_ADDR*)CoTaskMemAlloc(sizeof(IP_ADDR));
//                 if (!This->pCurrentConfig->Ns)
//                     return E_OUTOFMEMORY;
//                 ZeroMemory(This->pCurrentConfig->Ns, sizeof(IP_ADDR));
//             }
//             This->pCurrentConfig->Ns->IpAddress = dwIpAddr;
//         }
//         else if (This->pCurrentConfig->Ns)
//         {
//             IP_ADDR *pTemp = This->pCurrentConfig->Ns->Next;
//
//             CoTaskMemFree(This->pCurrentConfig->Ns);
//             This->pCurrentConfig->Ns = pTemp;
//             bSkip = TRUE;
//         }
//
//
//         if (SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//         {
//             if (!This->pCurrentConfig->Ns || bSkip)
//             {
//                 if (!This->pCurrentConfig->Ns)
//                 {
//                     This->pCurrentConfig->Ns = (IP_ADDR*)CoTaskMemAlloc(sizeof(IP_ADDR));
//                     if (!This->pCurrentConfig->Ns)
//                         return E_OUTOFMEMORY;
//                     ZeroMemory(This->pCurrentConfig->Ns, sizeof(IP_ADDR));
//                 }
//                 This->pCurrentConfig->Ns->IpAddress = dwIpAddr;
//             }
//             else if (!This->pCurrentConfig->Ns->Next)
//             {
//                 This->pCurrentConfig->Ns->Next = (IP_ADDR*)CoTaskMemAlloc(sizeof(IP_ADDR));
//                 if (!This->pCurrentConfig->Ns->Next)
//                    return E_OUTOFMEMORY;
//                 ZeroMemory(This->pCurrentConfig->Ns->Next, sizeof(IP_ADDR));
//                 This->pCurrentConfig->Ns->Next->IpAddress = dwIpAddr;
//             }
//             else
//             {
//                 This->pCurrentConfig->Ns->Next->IpAddress = dwIpAddr;
//             }
//         }
//         else
//         {
//             if (This->pCurrentConfig->Ns && This->pCurrentConfig->Ns->Next)
//             {
//                 if (This->pCurrentConfig->Ns->Next->Next)
//                 {
//                     IP_ADDR *pTemp = This->pCurrentConfig->Ns->Next->Next;
//                     CoTaskMemFree(This->pCurrentConfig->Ns->Next);
//                     This->pCurrentConfig->Ns->Next = pTemp;
//                 }
//                 else
//                 {
//                     CoTaskMemFree(This->pCurrentConfig->Ns->Next);
//                     This->pCurrentConfig->Ns->Next = NULL;
//                 }
//             }
//         }
//     }
//     else
//     {
//         This->pCurrentConfig->AutoconfigActive = TRUE;
//     }
//     return S_OK;
// }
//
// HRESULT
// InitializeTcpipBasicDlgCtrls(
//     HWND hwndDlg,
//     TcpipSettings * pCurSettings)
// {
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 0, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 0, MAKEIPRANGE(1, 223));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 1, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 2, MAKEIPRANGE(0, 255));
//     SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETRANGE, 3, MAKEIPRANGE(0, 255));
//
//     if (pCurSettings->DhcpEnabled)
//     {
//         CheckDlgButton(hwndDlg, IDC_USEDHCP, BST_CHECKED);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_AUTODNS), TRUE);
//     }
//     else
//     {
//         CheckDlgButton(hwndDlg, IDC_NODHCP, BST_CHECKED);
//
//         if (pCurSettings->Ip)
//         {
//             /* set current ip address */
//             SendDlgItemMessageA(hwndDlg, IDC_IPADDR, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ip->IpAddress);
//             /* set current hostmask */
//             SendDlgItemMessageA(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ip->u.Subnetmask);
//         }
//     }
//
//     if (pCurSettings->Gw && pCurSettings->Gw->IpAddress)
//     {
//         /* set current gateway */
//         SendDlgItemMessageA(hwndDlg, IDC_DEFGATEWAY, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Gw->IpAddress);
//     }
//
//     if (pCurSettings->AutoconfigActive)
//     {
//         CheckDlgButton(hwndDlg, IDC_AUTODNS, BST_CHECKED);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), FALSE);
//     }
//     else
//     {
//         CheckDlgButton(hwndDlg, IDC_FIXEDDNS, BST_CHECKED);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), TRUE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), TRUE);
//         if (pCurSettings->Ns)
//         {
//             SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ns->IpAddress);
//             if (pCurSettings->Ns->Next)
//             {
//                 SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_SETADDRESS, 0, (LPARAM)pCurSettings->Ns->Next->IpAddress);
//             }
//             else
//             {
//                 SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_CLEARADDRESS, 0, 0);
//             }
//         }
//         else
//         {
//             SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_CLEARADDRESS, 0, 0);
//             SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_CLEARADDRESS, 0, 0);
//         }
//     }
//
//     return S_OK;
// }
//
// HRESULT
// CopyIpAddrString(
//     IP_ADDR_STRING * pSrc,
//     IP_ADDR ** pTarget,
//     COPY_TYPE Type,
//     LPWSTR szMetric)
// {
//     IP_ADDR_STRING * pCurrent;
//     IP_ADDR *pNew, *pLast;
//
//     pCurrent = pSrc;
//     pLast = NULL;
//
//     while(pCurrent)
//     {
//         pNew = CoTaskMemAlloc(sizeof(IP_ADDR));
//         if (!pNew)
//         {
//            break;
//         }
//         ZeroMemory(pNew, sizeof(IP_ADDR));
//         pNew->IpAddress = GetIpAddressFromStringA(pCurrent->IpAddress.String);
//         if (!pNew->IpAddress)
//         {
//             CoTaskMemFree(pNew);
//             return E_FAIL;
//         }
//
//        if (Type == SUBMASK)
//        {
//            pNew->u.Subnetmask = GetIpAddressFromStringA(pCurrent->IpMask.String);
//            pNew->NTEContext = pCurrent->Context;
//        }
//        else if (Type == METRIC)
//        {
//            if (szMetric && szMetric[0] != L'\0')
//            {
//                pNew->u.Metric = _wtoi(szMetric);
//                szMetric += wcslen(szMetric) + 1;
//            }
//        }
//
//         if (!pLast)
//             *pTarget = pNew;
//         else
//             pLast->Next = pNew;
//
//         pLast = pNew;
//         pCurrent = pCurrent->Next;
//
//     }
//     pLast->Next = NULL;
//     return S_OK;
// }
//
//
//
// INT_PTR
// __stdcall
// TcpipBasicDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam
// )
// {
//     TcpipConfNotifyImpl * This;
//     LPPROPSHEETPAGE page;
//     LPNMIPADDRESS lpnmipa;
//     LPPSHNOTIFY lppsn;
//     DWORD dwIpAddr;
//
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             page = (LPPROPSHEETPAGE)lParam;
//             This = (TcpipConfNotifyImpl*)page->lParam;
//             if (This->pCurrentConfig)
//                 InitializeTcpipBasicDlgCtrls(hwndDlg, This->pCurrentConfig);
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)This);
//             return TRUE;
//         case WM_NOTIFY:
//             lppsn = (LPPSHNOTIFY) lParam;
//             lpnmipa = (LPNMIPADDRESS) lParam;
//             if (lpnmipa->hdr.code == IPN_FIELDCHANGED)
//             {
//                 PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 if (lpnmipa->hdr.idFrom == IDC_IPADDR)
//                 {
//                     if (SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_GETADDRESS, 0, (LPARAM)&dwIpAddr) == 4)
//                     {
//                         if (dwIpAddr <= MAKEIPADDRESS(127, 255, 255, 255))
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)MAKEIPADDRESS(255, 0, 0, 0));
//                         else if (dwIpAddr <= MAKEIPADDRESS(191, 255, 255, 255))
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)MAKEIPADDRESS(255, 255, 0, 0));
//                         else if (dwIpAddr <= MAKEIPADDRESS(223, 255, 255, 255))
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_SETADDRESS, 0, (LPARAM)MAKEIPADDRESS(255, 255, 255, 0));
//                      }
//                 }
//             }
//             else if (lppsn->hdr.code == PSN_APPLY)
//             {
//                 This = (TcpipConfNotifyImpl*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//                 if (SUCCEEDED(StoreTcpipBasicSettings(hwndDlg, This, TRUE)))
//                     SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR);
//                 else
//                     SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_INVALID);
//
//                 return TRUE;
//             }
//             break;
//         case WM_COMMAND:
//             if (HIWORD(wParam) == BN_CLICKED)
//             {
//                 switch (LOWORD(wParam))
//                 {
//                     case IDC_USEDHCP:
//                         if (SendMessageW(GetParent(hwndDlg), PSM_INDEXTOID, 1, 0) == 0)
//                         {
//                             PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                             SendDlgItemMessageW(hwndDlg, IDC_IPADDR, IPM_CLEARADDRESS, 0, 0);
//                             SendDlgItemMessageW(hwndDlg, IDC_SUBNETMASK, IPM_CLEARADDRESS, 0, 0);
//                             SendDlgItemMessageW(hwndDlg, IDC_DEFGATEWAY, IPM_CLEARADDRESS, 0, 0);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), FALSE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_AUTODNS), TRUE);
//                             AddAlternativeDialog(GetParent(hwndDlg), (TcpipConfNotifyImpl*)GetWindowLongPtr(hwndDlg, DWLP_USER));
//                         }
//                         break;
//                     case IDC_NODHCP:
//                         if (SendMessageW(GetParent(hwndDlg), PSM_INDEXTOID, 1, 0) != 0)
//                         {
//                             PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_IPADDR), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_SUBNETMASK), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_DEFGATEWAY), TRUE);
//                             if (IsDlgButtonChecked(hwndDlg, IDC_AUTODNS) == BST_CHECKED)
//                             {
//                                 CheckDlgButton(hwndDlg, IDC_AUTODNS, BST_UNCHECKED);
//                             }
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_AUTODNS), FALSE);
//                             CheckDlgButton(hwndDlg, IDC_FIXEDDNS, BST_CHECKED);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), TRUE);
//                             EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), TRUE);
//                             SendMessageW(GetParent(hwndDlg), PSM_REMOVEPAGE, 1, 0);
//                         }
//                         break;
//                     case IDC_AUTODNS:
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                         SendDlgItemMessageW(hwndDlg, IDC_DNS1, IPM_CLEARADDRESS, 0, 0);
//                         SendDlgItemMessageW(hwndDlg, IDC_DNS2, IPM_CLEARADDRESS, 0, 0);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), FALSE);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), FALSE);
//                         break;
//                     case IDC_FIXEDDNS:
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS1), TRUE);
//                         EnableWindow(GetDlgItem(hwndDlg, IDC_DNS2), TRUE);
//                         break;
//                     case IDC_ADVANCED:
//                         LaunchAdvancedTcpipSettings(hwndDlg, (TcpipConfNotifyImpl*)GetWindowLongPtr(hwndDlg, DWLP_USER));
//                         break;
//                 }
//                 break;
//             }
//         default:
//             break;
//
//     }
//     return FALSE;
// }
//
// /***************************************************************
//  * INetCfgComponentPropertyUi interface
//  */
//
// HRESULT
// WINAPI
// INetCfgComponentPropertyUi_fnQueryInterface(
//     INetCfgComponentPropertyUi * iface,
//     REFIID iid,
//     LPVOID * ppvObj)
// {
//     //LPOLESTR pStr;
//     TcpipConfNotifyImpl * This = (TcpipConfNotifyImpl*)iface;
//
//     *ppvObj = NULL;
//
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_INetCfgComponentPropertyUi))
//     {
//         *ppvObj = This;
//         INetCfgComponentPropertyUi_AddRef(iface);
//         return S_OK;
//     }
//     else if (IsEqualIID(iid, &IID_INetCfgComponentControl))
//     {
//         *ppvObj = (LPVOID*)&This->lpVtblCompControl;
//         INetCfgComponentPropertyUi_AddRef(iface);
//         return S_OK;
//     }
//
//     //StringFromCLSID(iid, &pStr);
//     //MessageBoxW(NULL, pStr, L"INetConnectionPropertyUi_fnQueryInterface", MB_OK);
//     //CoTaskMemFree(pStr);
//
//     return E_NOINTERFACE;
// }
//
//
// ULONG
// WINAPI
// INetCfgComponentPropertyUi_fnAddRef(
//     INetCfgComponentPropertyUi * iface)
// {
//     TcpipConfNotifyImpl * This = (TcpipConfNotifyImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// INetCfgComponentPropertyUi_fnRelease(
//     INetCfgComponentPropertyUi * iface)
// {
//     TcpipConfNotifyImpl * This = (TcpipConfNotifyImpl*)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     if (!refCount)
//     {
//        CoTaskMemFree(This);
//     }
//     return refCount;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentPropertyUi_fnQueryPropertyUi(
//     INetCfgComponentPropertyUi * iface,
//     IUnknown *pUnkReserved)
// {
//     INetLanConnectionUiInfo * pLanInfo;
//     HRESULT hr;
//     TcpipConfNotifyImpl * This = (TcpipConfNotifyImpl*)iface;
//
//     hr = IUnknown_QueryInterface(pUnkReserved, &IID_INetLanConnectionUiInfo, (LPVOID*)&pLanInfo);
//     if (FAILED(hr))
//         return hr;
//
//     INetLanConnectionUiInfo_GetDeviceGuid(pLanInfo, &This->NetCfgInstanceId);
//
//     //FIXME
//     // check if tcpip is enabled on that binding */
//     IUnknown_Release(pUnkReserved);
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentPropertyUi_fnSetContext(
//     INetCfgComponentPropertyUi * iface,
//     IUnknown *pUnkReserved)
// {
//     TcpipConfNotifyImpl * This = (TcpipConfNotifyImpl*)iface;
//
//     if (!iface || !pUnkReserved)
//         return E_POINTER;
//
//     This->pUnknown = pUnkReserved;
//     return S_OK;
// }
//
// HRESULT
// LoadDNSSettings(
//     TcpipConfNotifyImpl * This)
// {
//     LPOLESTR pStr;
//     WCHAR szBuffer[200];
//     HKEY hKey;
//     DWORD dwSize;
//
//
//     This->pCurrentConfig->pDNS = (TcpipAdvancedDNSDlgSettings*) CoTaskMemAlloc(sizeof(TcpipAdvancedDNSDlgSettings));
//     if (!This->pCurrentConfig->pDNS)
//         return E_FAIL;
//
//     ZeroMemory(This->pCurrentConfig->pDNS, sizeof(TcpipAdvancedDNSDlgSettings));
//
//     if (FAILED(StringFromCLSID(&This->NetCfgInstanceId, &pStr)))
//         return E_FAIL;
//
//     swprintf(szBuffer, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%s", pStr);
//     CoTaskMemFree(pStr);
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuffer, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//     {
//         dwSize = sizeof(DWORD);
//         RegQueryValueExW(hKey, L"RegisterAdapterName", NULL, NULL, (LPBYTE)&This->pCurrentConfig->pDNS->RegisterAdapterName, &dwSize);
//
//         dwSize = sizeof(DWORD);
//         RegQueryValueExW(hKey, L"RegistrationEnabled", NULL, NULL, (LPBYTE)&This->pCurrentConfig->pDNS->RegistrationEnabled, &dwSize);
//
//         dwSize = sizeof(This->pCurrentConfig->pDNS->szDomain);
//         RegQueryValueExW(hKey, L"Domain", NULL, NULL, (LPBYTE)This->pCurrentConfig->pDNS->szDomain, &dwSize);
//
//         dwSize = sizeof(DWORD);
//         if (RegQueryValueExW(hKey, L"InterfaceMetric", NULL, NULL, (LPBYTE)&This->pCurrentConfig->Metric, &dwSize) != ERROR_SUCCESS)
//             This->pCurrentConfig->Metric = 0;
//
//         RegCloseKey(hKey);
//     }
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters", 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//     {
//         dwSize = sizeof(DWORD);
//         RegQueryValueExW(hKey, L"UseDomainNameDevolution", NULL, NULL, (LPBYTE)&This->pCurrentConfig->pDNS->UseDomainNameDevolution, &dwSize);
//
//         dwSize = 0;
//         if (RegQueryValueExW(hKey, L"SearchList", NULL, NULL, NULL, &dwSize) == ERROR_SUCCESS)
//         {
//             This->pCurrentConfig->pDNS->szSearchList = (LPWSTR)CoTaskMemAlloc(dwSize);
//             if (This->pCurrentConfig->pDNS->szSearchList)
//             {
//                 if (RegQueryValueExW(hKey, L"SearchList", NULL, NULL, (LPBYTE)This->pCurrentConfig->pDNS->szSearchList, &dwSize) != ERROR_SUCCESS)
//                 {
//                     CoTaskMemFree(This->pCurrentConfig->pDNS->szSearchList);
//                     This->pCurrentConfig->pDNS->szSearchList = NULL;
//                 }
//             }
//         }
//         RegCloseKey(hKey);
//     }
//     return S_OK;
// }
//
// LPWSTR
// LoadTcpFilterSettingsFromRegistry(HKEY hKey, LPCWSTR szName, LPDWORD Size)
// {
//     DWORD dwSize;
//     LPWSTR pData;
//
//     if (RegQueryValueExW(hKey, szName, NULL, NULL, NULL, &dwSize) != ERROR_SUCCESS)
//         return NULL;
//
//     pData = (LPWSTR)CoTaskMemAlloc(dwSize);
//     if (!pData)
//         return NULL;
//
//     if (RegQueryValueExW(hKey, szName, NULL, NULL, (LPBYTE)pData, &dwSize) != ERROR_SUCCESS)
//     {
//         CoTaskMemFree(pData);
//         return NULL;
//     }
//     *Size = dwSize;
//     return pData;
// }
//
// HRESULT
// LoadFilterSettings(
//     TcpipConfNotifyImpl * This)
// {
//     HKEY hKey;
//     TcpFilterSettings *pFilter;
//     WCHAR szBuffer[200];
//     LPOLESTR pStr;
//     DWORD dwVal, dwSize;
//
//     pFilter = (TcpFilterSettings*)CoTaskMemAlloc(sizeof(TcpFilterSettings));
//     if (!pFilter)
//         return E_FAIL;
//
//     ZeroMemory(pFilter, sizeof(TcpFilterSettings));
//     This->pCurrentConfig->pFilter = pFilter;
//
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters", 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//     {
//         dwSize = sizeof(DWORD);
//         if (RegQueryValueExW(hKey, L"EnableSecurityFilters", NULL, NULL, (LPBYTE)&dwVal, &dwSize) == ERROR_SUCCESS)
//             pFilter->EnableSecurityFilters = dwVal;
//         RegCloseKey(hKey);
//     }
//     else
//     {
//         pFilter->EnableSecurityFilters = FALSE;
//     }
//
//     if (FAILED(StringFromCLSID(&This->NetCfgInstanceId, &pStr)))
//         return E_FAIL;
//
//     swprintf(szBuffer, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%s", pStr);
//     CoTaskMemFree(pStr);
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuffer, 0, KEY_READ, &hKey) != ERROR_SUCCESS)
//     {
//         return S_OK;
//     }
//     pFilter->szTCPAllowedPorts = LoadTcpFilterSettingsFromRegistry(hKey, L"TCPAllowedPorts", &pFilter->TCPSize);
//     pFilter->szUDPAllowedPorts = LoadTcpFilterSettingsFromRegistry(hKey, L"UDPAllowedPorts", &pFilter->UDPSize);
//     pFilter->szRawIPAllowedProtocols = LoadTcpFilterSettingsFromRegistry(hKey, L"RawIPAllowedProtocols", &pFilter->IPSize);
//     RegCloseKey(hKey);
//     return S_OK;
// }
//
//
// HRESULT
// Initialize(TcpipConfNotifyImpl * This)
// {
//     DWORD dwSize, Status;
//     WCHAR szBuffer[50];
//     IP_ADAPTER_INFO * pCurrentAdapter;
//     IP_ADAPTER_INFO * pInfo;
//     PIP_PER_ADAPTER_INFO pPerInfo;
//     IP_PER_ADAPTER_INFO Info;
//     LPOLESTR pStr;
//     HRESULT hr;
//     BOOL bFound;
//     TcpipSettings * pCurSettings;
//     ULONG uLength;
//
//     if (This->pCurrentConfig)
//         return S_OK;
//
//     hr = StringFromCLSID(&This->NetCfgInstanceId, &pStr);
//     if (FAILED(hr))
//         return hr;
//
//
//     dwSize = 0;
//     if (GetAdaptersInfo(NULL, &dwSize) != ERROR_BUFFER_OVERFLOW)
//     {
//         CoTaskMemFree(pStr);
//         return E_FAIL;
//     }
//
//     pInfo = CoTaskMemAlloc(dwSize);
//     if (!pInfo)
//     {
//         CoTaskMemFree(pStr);
//         return E_FAIL;
//     }
//
//     if (GetAdaptersInfo(pInfo, &dwSize) != ERROR_SUCCESS)
//     {
//         CoTaskMemFree(pStr);
//         CoTaskMemFree(pInfo);
//         return E_FAIL;
//     }
//
//     pCurrentAdapter = pInfo;
//     bFound = FALSE;
//     while(pCurrentAdapter)
//     {
//         szBuffer[0] = L'\0';
//         if (MultiByteToWideChar(CP_ACP, 0, pCurrentAdapter->AdapterName, -1, szBuffer, sizeof(szBuffer)/sizeof(szBuffer[0])))
//         {
//             szBuffer[(sizeof(szBuffer)/sizeof(WCHAR))-1] = L'\0';
//         }
//         if (!_wcsicmp(szBuffer, pStr))
//         {
//             bFound = TRUE;
//             break;
//         }
//         pCurrentAdapter = pCurrentAdapter->Next;
//     }
//     CoTaskMemFree(pStr);
//
//     if (!bFound)
//     {
//         CoTaskMemFree(pInfo);
//         return E_FAIL;
//     }
//
//     pCurSettings = CoTaskMemAlloc(sizeof(TcpipSettings));
//     if (!pCurSettings)
//     {
//         CoTaskMemFree(pInfo);
//         return E_FAIL;
//     }
//
//     ZeroMemory(pCurSettings, sizeof(TcpipSettings));
//     This->pCurrentConfig = pCurSettings;
//     pCurSettings->DhcpEnabled = pCurrentAdapter->DhcpEnabled;
//     pCurSettings->Index = pCurrentAdapter->Index;
//
//     if (!pCurrentAdapter->DhcpEnabled)
//     {
//         CopyIpAddrString(&pCurrentAdapter->IpAddressList, &pCurSettings->Ip, SUBMASK, NULL);
//     }
//
//     CopyIpAddrString(&pCurrentAdapter->GatewayList, &pCurSettings->Gw, METRIC, NULL);
//
//     uLength = sizeof(IP_PER_ADAPTER_INFO);
//     ZeroMemory(&Info, sizeof(IP_PER_ADAPTER_INFO));
//
//     if (GetPerAdapterInfo(pCurrentAdapter->Index, &Info, &uLength) == ERROR_BUFFER_OVERFLOW)
//     {
//         pPerInfo = (PIP_PER_ADAPTER_INFO)CoTaskMemAlloc(uLength);
//         if (pPerInfo)
//         {
//             Status = GetPerAdapterInfo(pCurrentAdapter->Index, pPerInfo, &uLength);
//             if (Status == NOERROR)
//             {
//                 if (!pPerInfo->AutoconfigActive)
//                 {
//                     CopyIpAddrString(&pPerInfo->DnsServerList, &pCurSettings->Ns, IPADDR, NULL);
//                 }
//                 pCurSettings->AutoconfigActive = pPerInfo->AutoconfigActive;
//             }
//             CoTaskMemFree(pPerInfo);
//         }
//     }
//     else
//     {
//         if (!Info.AutoconfigActive)
//         {
//             CopyIpAddrString(&Info.DnsServerList, &pCurSettings->Ns, IPADDR, NULL);
//         }
//         pCurSettings->AutoconfigActive = Info.AutoconfigActive;
//     }
//
//     if (FAILED(LoadFilterSettings(This)))
//         return E_FAIL;
//
//     if (FAILED(LoadDNSSettings(This)))
//         return E_FAIL;
//
//     CoTaskMemFree(pInfo);
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentPropertyUi_fnMergePropPages(
//     INetCfgComponentPropertyUi * iface,
//     DWORD *pdwDefPages,
//     BYTE **pahpspPrivate,
//     UINT *pcPages,
//     HWND hwndParent,
//     LPCWSTR *pszStartPage)
// {
//     HPROPSHEETPAGE * hppages;
//     UINT NumPages;
//     HRESULT hr;
//     TcpipConfNotifyImpl * This = (TcpipConfNotifyImpl*)iface;
//
//     hr = Initialize(This);
//     if (FAILED(hr))
//         return hr;
//
//     if (This->pCurrentConfig->DhcpEnabled)
//         NumPages = 2;
//     else
//         NumPages = 1;
//
//     hppages = (HPROPSHEETPAGE*) CoTaskMemAlloc(sizeof(HPROPSHEETPAGE) * NumPages);
//     if (!hppages)
//         return E_FAIL;
//
//     hppages[0] = InitializePropertySheetPage(MAKEINTRESOURCEW(IDD_TCPIP_BASIC_DLG), TcpipBasicDlg, (LPARAM)This, NULL);
//     if (!hppages[0])
//     {
//         CoTaskMemFree(hppages);
//         return E_FAIL;
//     }
//     if (NumPages == 2)
//     {
//         hppages[1] = InitializePropertySheetPage(MAKEINTRESOURCEW(IDD_TCPIP_ALTCF_DLG), TcpipAltConfDlg, (LPARAM)This, NULL);
//         if (!hppages[1])
//         {
//             DestroyPropertySheetPage(hppages[0]);
//             CoTaskMemFree(hppages);
//             return E_FAIL;
//         }
//     }
//
//     *pahpspPrivate = (BYTE*)hppages;
//     *pcPages = NumPages;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentPropertyUi_fnValidateProperties(
//     INetCfgComponentPropertyUi * iface,
//     HWND hwndDlg)
// {
// MessageBoxW(NULL, L"INetCfgComponentPropertyUi_fnValidateProperties", NULL, MB_OK);
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentPropertyUi_fnApplyProperties(
//     INetCfgComponentPropertyUi * iface)
// {
// MessageBoxW(NULL, L"INetCfgComponentPropertyUi_fnApplyProperties", NULL, MB_OK);
//     return S_OK;
// }
//
//
// HRESULT
// WINAPI
// INetCfgComponentPropertyUi_fnCancelProperties(
//     INetCfgComponentPropertyUi * iface)
// {
// //MessageBoxW(NULL, L"INetCfgComponentPropertyUi_fnCancelProperties", NULL, MB_OK);
//     return S_OK;
// }
//
// static const INetCfgComponentPropertyUiVtbl vt_NetCfgComponentPropertyUi =
// {
//     INetCfgComponentPropertyUi_fnQueryInterface,
//     INetCfgComponentPropertyUi_fnAddRef,
//     INetCfgComponentPropertyUi_fnRelease,
//     INetCfgComponentPropertyUi_fnQueryPropertyUi,
//     INetCfgComponentPropertyUi_fnSetContext,
//     INetCfgComponentPropertyUi_fnMergePropPages,
//     INetCfgComponentPropertyUi_fnValidateProperties,
//     INetCfgComponentPropertyUi_fnApplyProperties,
//     INetCfgComponentPropertyUi_fnCancelProperties
// };
//
// /***************************************************************
//  * INetCfgComponentControl interface
//  */
//
// HRESULT
// WINAPI
// INetCfgComponentControl_fnQueryInterface(
//     INetCfgComponentControl * iface,
//     REFIID iid,
//     LPVOID * ppvObj)
// {
//     TcpipConfNotifyImpl * This = impl_from_INetCfgComponentControl(iface);
//     return INetCfgComponentPropertyUi_QueryInterface((INetCfgComponentPropertyUi*)This, iid, ppvObj);
// }
//
// ULONG
// WINAPI
// INetCfgComponentControl_fnAddRef(
//     INetCfgComponentControl * iface)
// {
//     TcpipConfNotifyImpl * This = impl_from_INetCfgComponentControl(iface);
//     return INetCfgComponentPropertyUi_AddRef((INetCfgComponentPropertyUi*)This);
// }
//
// ULONG
// WINAPI
// INetCfgComponentControl_fnRelease(
//     INetCfgComponentControl * iface)
// {
//     TcpipConfNotifyImpl * This = impl_from_INetCfgComponentControl(iface);
//     return INetCfgComponentPropertyUi_Release((INetCfgComponentPropertyUi*)This);
// }
//
// HRESULT
// WINAPI
// INetCfgComponentControl_fnInitialize(
//     INetCfgComponentControl * iface,
//     INetCfgComponent *pIComp,
//     INetCfg *pINetCfg,
//     BOOL fInstalling)
// {
//     TcpipConfNotifyImpl * This = impl_from_INetCfgComponentControl(iface);
//
//     This->pNCfg = pINetCfg;
//     This->pNComp = pIComp;
//
//     return S_OK;
// }
//
// static
// LPWSTR
// CreateMultiSzString(IP_ADDR * pAddr, COPY_TYPE Type, LPDWORD Size, BOOL bComma)
// {
//     LPWSTR pStr, pRet;
//     DWORD dwSize, dwIpAddr;
//     WCHAR szBuffer[30];
//     IP_ADDR *pTemp = pAddr;
//
//
//     dwSize = 0;
//     while(pTemp)
//     {
//         if (Type == IPADDR)
//         {
//             dwIpAddr = pTemp->IpAddress;
//             swprintf(szBuffer, L"%lu.%lu.%lu.%lu",
//                     FIRST_IPADDRESS(dwIpAddr), SECOND_IPADDRESS(dwIpAddr), THIRD_IPADDRESS(dwIpAddr), FOURTH_IPADDRESS(dwIpAddr));
//         }else if (Type == SUBMASK)
//         {
//             dwIpAddr = pTemp->u.Subnetmask;
//             swprintf(szBuffer, L"%lu.%lu.%lu.%lu",
//                     FIRST_IPADDRESS(dwIpAddr), SECOND_IPADDRESS(dwIpAddr), THIRD_IPADDRESS(dwIpAddr), FOURTH_IPADDRESS(dwIpAddr));
//         }
//         else if (Type == METRIC)
//         {
//             swprintf(szBuffer, L"%u", pTemp->u.Metric);
//         }
//
//         dwSize += wcslen(szBuffer) + 1;
//         pTemp = pTemp->Next;
//     }
//
//     if (!dwSize)
//         return NULL;
//
//     pStr = pRet = CoTaskMemAlloc((dwSize+1) * sizeof(WCHAR));
//     if(!pStr)
//        return NULL;
//
//     pTemp = pAddr;
//     while(pTemp)
//     {
//         if (Type == IPADDR)
//         {
//             dwIpAddr = pTemp->IpAddress;
//             swprintf(pStr, L"%lu.%lu.%lu.%lu",
//                     FIRST_IPADDRESS(dwIpAddr), SECOND_IPADDRESS(dwIpAddr), THIRD_IPADDRESS(dwIpAddr), FOURTH_IPADDRESS(dwIpAddr));
//         }else if (Type == SUBMASK)
//         {
//             dwIpAddr = pTemp->u.Subnetmask;
//             swprintf(pStr, L"%lu.%lu.%lu.%lu",
//                     FIRST_IPADDRESS(dwIpAddr), SECOND_IPADDRESS(dwIpAddr), THIRD_IPADDRESS(dwIpAddr), FOURTH_IPADDRESS(dwIpAddr));
//         }
//         else if (Type == METRIC)
//         {
//             swprintf(pStr, L"%u", pTemp->u.Metric);
//         }
//
//         if (bComma)
//         {
//             pStr += wcslen(pStr);
//             if (pTemp->Next)
//             {
//                 pStr[0] = L',';
//                 pStr++;
//             }
//         }
//         else
//         {
//             pStr += wcslen(pStr) + 1;
//         }
//         pTemp = pTemp->Next;
//     }
//     pStr[0] = L'\0';
//     *Size = (dwSize+1) * sizeof(WCHAR);
//     return pRet;
// }
//
//
// HRESULT
// WINAPI
// INetCfgComponentControl_fnApplyRegistryChanges(
//     INetCfgComponentControl * iface)
// {
//     HKEY hKey;
//     LPOLESTR pStr;
//     DWORD dwSize;
//     WCHAR szBuffer[200];
//     TcpipSettings * pCurrentConfig, *pOldConfig;
//     ULONG NTEInstance;
//     DWORD DhcpApiVersion;
//
//     TcpipConfNotifyImpl * This = impl_from_INetCfgComponentControl(iface);
//
//     pCurrentConfig = This->pCurrentConfig;
//     This->pCurrentConfig = NULL;
//
//     if (FAILED(Initialize(This)))
//     {
//         This->pCurrentConfig = pCurrentConfig;
//         return E_FAIL;
//     }
//     pOldConfig = This->pCurrentConfig;
//     This->pCurrentConfig = pCurrentConfig;
//
//     //MessageBoxW(NULL, L"INetCfgComponentControl_fnApplyRegistryChanges", NULL, MB_OK);
//
//
//     if (RegCreateKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters", 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS)
//     {
//         if (pCurrentConfig->pDNS)
//         {
//             RegSetValueExW(hKey, L"UseDomainNameDevolution", 0, REG_DWORD, (LPBYTE)&pCurrentConfig->pDNS->UseDomainNameDevolution, sizeof(DWORD));
//             RegSetValueExW(hKey, L"SearchList", 0, REG_SZ, (LPBYTE)pCurrentConfig->pDNS->szSearchList,
//                        (wcslen(pCurrentConfig->pDNS->szSearchList)+1) * sizeof(WCHAR));
//         }
//         if (pCurrentConfig->pFilter)
//         {
//             RegSetValueExW(hKey, L"EnableSecurityFilters", 0, REG_DWORD,
//                       (LPBYTE)&pCurrentConfig->pFilter->EnableSecurityFilters, sizeof(DWORD));
//         }
//         RegCloseKey(hKey);
//     }
//
//     if (FAILED(StringFromCLSID(&This->NetCfgInstanceId, &pStr)))
//         return E_FAIL;
//
//     swprintf(szBuffer, L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\%s", pStr);
//     CoTaskMemFree(pStr);
//
//     if (RegCreateKeyExW(HKEY_LOCAL_MACHINE, szBuffer, 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS)
//     {
//         if (pCurrentConfig->Metric)
//             RegSetValueExW(hKey, L"InterfaceMetric", 0, REG_DWORD, (LPBYTE)&This->pCurrentConfig->Metric, sizeof(DWORD));
//         else
//             RegDeleteValueW(hKey, L"InterfaceMetric");
//
//         if (pCurrentConfig->pDNS)
//         {
//             RegSetValueExW(hKey, L"RegisterAdapterName", 0, REG_DWORD, (LPBYTE)&This->pCurrentConfig->pDNS->RegisterAdapterName, sizeof(DWORD));
//             RegSetValueExW(hKey, L"RegistrationEnabled", 0, REG_DWORD, (LPBYTE)&This->pCurrentConfig->pDNS->RegistrationEnabled, sizeof(DWORD));
//             RegSetValueExW(hKey, L"Domain", 0, REG_SZ, (LPBYTE)This->pCurrentConfig->pDNS->szDomain,
//                        (wcslen(This->pCurrentConfig->pDNS->szDomain)+1) * sizeof(WCHAR));
//         }
//         RegSetValueExW(hKey, L"EnableDHCP", 0, REG_DWORD, (LPBYTE)&pCurrentConfig->DhcpEnabled, sizeof(DWORD));
//         if (pCurrentConfig->DhcpEnabled)
//         {
//             RegSetValueExW(hKey, L"IPAddress", 0, REG_MULTI_SZ, (LPBYTE)L"0.0.0.0\0", 9 * sizeof(WCHAR));
//             RegSetValueExW(hKey, L"SubnetMask", 0, REG_MULTI_SZ, (LPBYTE)L"0.0.0.0\0", 9 * sizeof(WCHAR));
//             if (!pOldConfig->DhcpEnabled)
//             {
//                 /* Delete this adapter's current IP address */
//                 DeleteIPAddress(pOldConfig->Ip->NTEContext);
//
//                 /* Delete all default routes for this adapter */
//                 dwSize = 0;
//                 if (GetIpForwardTable(NULL, &dwSize, FALSE) == ERROR_INSUFFICIENT_BUFFER)
//                 {
//                     DWORD Index;
//                     PMIB_IPFORWARDTABLE pIpForwardTable = (PMIB_IPFORWARDTABLE)CoTaskMemAlloc(dwSize);
//                     if (pIpForwardTable)
//                     {
//                         if (GetIpForwardTable(pIpForwardTable, &dwSize, FALSE) == NO_ERROR)
//                         {
//                             for (Index = 0; Index < pIpForwardTable->dwNumEntries; Index++)
//                             {
//                                 if (pIpForwardTable->table[Index].dwForwardIfIndex == pOldConfig->Index &&
//                                     pIpForwardTable->table[Index].dwForwardDest == 0)
//                                 {
//                                     DeleteIpForwardEntry(&pIpForwardTable->table[Index]);
//                                 }
//                             }
//                         }
//                         CoTaskMemFree(pIpForwardTable);
//                     }
//                 }
//             }
//         }
//         else
//         {
//             /* Open the DHCP API if DHCP is enabled */
//             if (pOldConfig->DhcpEnabled && DhcpCApiInitialize(&DhcpApiVersion) == NO_ERROR)
//             {
//                 /* We have to tell DHCP about this */
//                 DhcpStaticRefreshParams(pCurrentConfig->Index,
//                                         htonl(pCurrentConfig->Ip->IpAddress),
//                                         htonl(pCurrentConfig->Ip->u.Subnetmask));
//
//                 /* Close the API */
//                 DhcpCApiCleanup();
//             }
//             else
//             {
//                 /* Delete this adapter's current static IP address */
//                 DeleteIPAddress(pOldConfig->Ip->NTEContext);
//
//                 /* Add the static IP address via the standard IPHLPAPI function */
//                 AddIPAddress(htonl(pCurrentConfig->Ip->IpAddress),
//                              htonl(pCurrentConfig->Ip->u.Subnetmask),
//                              pCurrentConfig->Index,
//                              &pCurrentConfig->Ip->NTEContext,
//                              &NTEInstance);
//             }
//
//             pStr = CreateMultiSzString(pCurrentConfig->Ip, IPADDR, &dwSize, FALSE);
//             if(pStr)
//             {
//                 RegSetValueExW(hKey, L"IPAddress", 0, REG_MULTI_SZ, (LPBYTE)pStr, dwSize);
//                 CoTaskMemFree(pStr);
//             }
//
//             pStr = CreateMultiSzString(pCurrentConfig->Ip, SUBMASK, &dwSize, FALSE);
//             if(pStr)
//             {
//                 RegSetValueExW(hKey, L"SubnetMask", 0, REG_MULTI_SZ, (LPBYTE)pStr, dwSize);
//                 CoTaskMemFree(pStr);
//             }
//
//             /* Delete all default routes for this adapter */
//             dwSize = 0;
//             if (GetIpForwardTable(NULL, &dwSize, FALSE) == ERROR_INSUFFICIENT_BUFFER)
//             {
//                 DWORD Index;
//                 PMIB_IPFORWARDTABLE pIpForwardTable = (PMIB_IPFORWARDTABLE)CoTaskMemAlloc(dwSize);
//                 if (pIpForwardTable)
//                 {
//                     if (GetIpForwardTable(pIpForwardTable, &dwSize, FALSE) == NO_ERROR)
//                     {
//                         for (Index = 0; Index < pIpForwardTable->dwNumEntries; Index++)
//                         {
//                             if (pIpForwardTable->table[Index].dwForwardIfIndex == pOldConfig->Index &&
//                                 pIpForwardTable->table[Index].dwForwardDest == 0)
//                             {
//                                 DeleteIpForwardEntry(&pIpForwardTable->table[Index]);
//                             }
//                         }
//                     }
//                     CoTaskMemFree(pIpForwardTable);
//                 }
//             }
//         }
//
//         if (pCurrentConfig->Gw)
//         {
//             MIB_IPFORWARDROW RouterMib;
//             ZeroMemory(&RouterMib, sizeof(MIB_IPFORWARDROW));
//
//             RouterMib.dwForwardMetric1 = 1;
//             RouterMib.dwForwardIfIndex = pOldConfig->Index;
//             RouterMib.dwForwardNextHop = htonl(pCurrentConfig->Gw->IpAddress);
//
//             //TODO
//             // add multiple gw addresses when required
//
//             if (CreateIpForwardEntry(&RouterMib) == NO_ERROR)
//             {
//                 pStr = CreateMultiSzString(pCurrentConfig->Gw, IPADDR, &dwSize, FALSE);
//                 if(pStr)
//                 {
//                     RegSetValueExW(hKey, L"DefaultGateway", 0, REG_MULTI_SZ, (LPBYTE)pStr, dwSize);
//                     CoTaskMemFree(pStr);
//                 }
//
//                 pStr = CreateMultiSzString(pCurrentConfig->Gw, METRIC, &dwSize, FALSE);
//                 if(pStr)
//                 {
//                     RegSetValueExW(hKey, L"DefaultGatewayMetric", 0, REG_MULTI_SZ, (LPBYTE)pStr, dwSize);
//                     CoTaskMemFree(pStr);
//                 }
//             }
//         }
//         else
//         {
//             RegSetValueExW(hKey, L"DefaultGateway", 0, REG_MULTI_SZ, (LPBYTE)L"", 1 * sizeof(WCHAR));
//             RegSetValueExW(hKey, L"DefaultGatewayMetric", 0, REG_MULTI_SZ, (LPBYTE)L"\0", sizeof(WCHAR) * 2);
//         }
//
//         if (!pCurrentConfig->Ns || pCurrentConfig->AutoconfigActive)
//         {
//             RegDeleteValueW(hKey, L"NameServer");
//         }
//         else
//         {
//             pStr = CreateMultiSzString(pCurrentConfig->Ns, IPADDR, &dwSize, TRUE);
//             if(pStr)
//             {
//                 RegSetValueExW(hKey, L"NameServer", 0, REG_SZ, (LPBYTE)pStr, dwSize);
//                 //RegDeleteValueW(hKey, L"DhcpNameServer");
//                 CoTaskMemFree(pStr);
//             }
//         }
//
//         RegCloseKey(hKey);
//     }
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentControl_fnApplyPnpChanges(
//     INetCfgComponentControl * iface,
//     INetCfgPnpReconfigCallback *pICallback)
// {
//     //MessageBoxW(NULL, L"INetCfgComponentControl_fnApplyPnpChanges", NULL, MB_OK);
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentControl_fnCancelChanges(
//     INetCfgComponentControl * iface)
// {
//     //MessageBoxW(NULL, L"INetCfgComponentControl_fnCancelChanges", NULL, MB_OK);
//     return S_OK;
// }
//
// static const INetCfgComponentControlVtbl vt_NetCfgComponentControl =
// {
//     INetCfgComponentControl_fnQueryInterface,
//     INetCfgComponentControl_fnAddRef,
//     INetCfgComponentControl_fnRelease,
//     INetCfgComponentControl_fnInitialize,
//     INetCfgComponentControl_fnApplyRegistryChanges,
//     INetCfgComponentControl_fnApplyPnpChanges,
//     INetCfgComponentControl_fnCancelChanges
// };
//
// HRESULT
// WINAPI
// TcpipConfigNotify_Constructor (IUnknown * pUnkOuter, REFIID riid, LPVOID * ppv)
// {
//     TcpipConfNotifyImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (TcpipConfNotifyImpl *) CoTaskMemAlloc(sizeof (TcpipConfNotifyImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const INetCfgComponentPropertyUi*)&vt_NetCfgComponentPropertyUi;
//     This->lpVtblCompControl = (const INetCfgComponentControl*)&vt_NetCfgComponentControl;
//     This->pNCfg = NULL;
//     This->pUnknown = NULL;
//     This->pNComp = NULL;
//     This->pCurrentConfig = NULL;
//
//     if (!SUCCEEDED (INetCfgComponentPropertyUi_QueryInterface ((INetCfgComponentPropertyUi*)This, riid, ppv)))
//     {
//         INetCfgComponentPropertyUi_Release((INetCfgComponentPropertyUi*)This);
//         return E_NOINTERFACE;
//     }
//
//     INetCfgComponentPropertyUi_Release((INetCfgComponentPropertyUi*)This);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
