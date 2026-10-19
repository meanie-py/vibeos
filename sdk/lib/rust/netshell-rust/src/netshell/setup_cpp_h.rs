use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_05A95BD1856B: [WCHAR; 73] = [0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x0020, 0x0063, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0063, 0x006F, 0x006E, 0x0074, 0x0069, 0x006E, 0x0075, 0x0065, 0x0020, 0x0075, 0x006E, 0x0074, 0x0069, 0x006C, 0x0020, 0x0079, 0x006F, 0x0075, 0x000A, 0x0065, 0x006E, 0x0074, 0x0065, 0x0072, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x006E, 0x0061, 0x006D, 0x0065, 0x0020, 0x006F, 0x0066, 0x0020, 0x0079, 0x006F, 0x0075, 0x0072, 0x0020, 0x0064, 0x006F, 0x006D, 0x0061, 0x0069, 0x006E, 0x002F, 0x0077, 0x006F, 0x0072, 0x006B, 0x0067, 0x0072, 0x006F, 0x0075, 0x0070, 0x002E, 0x0000];
pub static CPP_COM_WSTR_3E79A2679AF5: [WCHAR; 10] = [0x0057, 0x004F, 0x0052, 0x004B, 0x0047, 0x0052, 0x004F, 0x0055, 0x0050, 0x0000];
pub static CPP_COM_WSTR_A970D97AAA44: [WCHAR; 9] = [0x0053, 0x0079, 0x0073, 0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x0000];
pub static CPP_COM_WSTR_AB7B4070E95B: [WCHAR; 14] = [0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct NETWORKSETUPDATA {
    pub dwMagic: DWORD,
    pub pSetupData: PSETUPDATA,

    /* settings page data */
    pub bInitialSetupRun: BOOL,
    pub bTypicalNetworkSetup: BOOL,
    pub hwndPage: HWND,
}
pub type PNETWORKSETUPDATA = *mut NETWORKSETUPDATA;

#[repr(C)]
pub struct NETWORKCOMPONENT {
    pub pszDeviceId: LPWSTR,
    pub pClassGuid: *const GUID,
}
unsafe impl Sync for NETWORKCOMPONENT {}
pub type PNETWORKCOMPONENT = NETWORKCOMPONENT;

pub static MS_TCPIP_W: [u16; 9] = [
    b'M' as u16, b'S' as u16, b'_' as u16, b'T' as u16, b'C' as u16,
    b'P' as u16, b'I' as u16, b'P' as u16, 0,
];
pub static MS_NDISUIO_W: [u16; 11] = [
    b'M' as u16, b'S' as u16, b'_' as u16, b'N' as u16, b'D' as u16,
    b'I' as u16, b'S' as u16, b'U' as u16, b'I' as u16, b'O' as u16, 0,
];

pub static NetworkComponents: [NETWORKCOMPONENT; 2] = [
    NETWORKCOMPONENT {
        pszDeviceId: &MS_TCPIP_W as *const [u16; 9] as *mut u16,
        pClassGuid: unsafe { core::ptr::addr_of!(GUID_DEVCLASS_NETTRANS) },
    },
    NETWORKCOMPONENT {
        pszDeviceId: &MS_NDISUIO_W as *const [u16; 11] as *mut u16,
        pClassGuid: unsafe { core::ptr::addr_of!(GUID_DEVCLASS_NETTRANS) },
    },
];

static mut UNUSED_PLACEHOLDER_NEVER_USED: u8 = 0;






unsafe extern "C" fn SetBoldText(hwndDlg: HWND, control: INT, pSetupData: PSETUPDATA) {
    SendDlgItemMessageW(hwndDlg, control, WM_SETFONT, (*pSetupData).hBoldFont as WPARAM, MAKELPARAM!(TRUE, 0));
}

unsafe extern "C" fn InstallNetworkComponent(pszComponentName: LPWSTR, pClassGuid: *const GUID) -> HRESULT {
    let mut pNetCfg: *mut INetCfg = core::ptr::null_mut();
    let mut pNetCfgLock: *mut INetCfgLock = core::ptr::null_mut();
    let mut pNetCfgClassSetup: *mut INetCfgClassSetup = core::ptr::null_mut();
    let mut pNetCfgComponent: *mut INetCfgComponent = core::ptr::null_mut();
    let mut fWriteLocked: BOOL = (FALSE as i32);
    let mut fInitialized: BOOL = (FALSE as i32);
    let mut OboToken: OBO_TOKEN = core::mem::zeroed();
    let mut hr: HRESULT;

    'exit: {
        hr = CoInitialize(core::ptr::null_mut());
        if hr != S_OK {
            break 'exit;
        }

        hr = CoCreateInstance(
            &CLSID_CNetCfg,
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_INetCfg,
            core::ptr::addr_of_mut!(pNetCfg) as *mut PVOID,
        );
        if hr != S_OK {
            break 'exit;
        }

        /* Acquire the write-lock */
        hr = (*pNetCfg).QueryInterface(
            &IID_INetCfgLock,
            core::ptr::addr_of_mut!(pNetCfgLock) as *mut PVOID,
        );
        if hr != S_OK {
            break 'exit;
        }

        hr = (*pNetCfgLock).AcquireWriteLock(
            5000,
            CPP_COM_WSTR_A970D97AAA44.as_ptr(),
            core::ptr::null_mut(),
        );
        if hr != S_OK {
            break 'exit;
        }

        fWriteLocked = (TRUE as i32);

        /* Initialize the network configuration */
        hr = (*pNetCfg).Initialize(core::ptr::null_mut());
        if hr != S_OK {
            break 'exit;
        }

        fInitialized = (TRUE as i32);

        hr = (*pNetCfg).QueryNetCfgClass(
            pClassGuid,
            &IID_INetCfgClassSetup,
            core::ptr::addr_of_mut!(pNetCfgClassSetup) as *mut PVOID,
        );
        if hr != S_OK {
            break 'exit;
        }

        ZeroMemory!(
            core::ptr::addr_of_mut!(OboToken) as PVOID,
            core::mem::size_of::<OBO_TOKEN>(),
        );
        OboToken.Type = OBO_USER;

        hr = (*pNetCfgClassSetup).Install(
            pszComponentName,
            core::ptr::addr_of_mut!(OboToken),
            0, //NSF_PRIMARYINSTALL,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(pNetCfgComponent),
        );
        if hr == S_OK {
            if !pNetCfgComponent.is_null() {
                (*pNetCfgComponent).Release();
            }
        } else {
            break 'exit;
        }

        hr = (*pNetCfg).Apply();
        if hr != S_OK {
            break 'exit;
        }
    }

    // exit:
    if !pNetCfgClassSetup.is_null() {
        (*pNetCfgClassSetup).Release();
    }

    if fInitialized != 0 {
        (*pNetCfg).Uninitialize();
    }

    if fWriteLocked != 0 {
        (*pNetCfgLock).ReleaseWriteLock();
    }

    if !pNetCfgLock.is_null() {
        (*pNetCfgLock).Release();
    }

    if !pNetCfg.is_null() {
        (*pNetCfg).Release();
    }

    CoUninitialize();

    return hr;
}

unsafe extern "system" fn InstallThreadProc(lpParameter: LPVOID) -> DWORD {
    let pNetworkSetupData: PNETWORKSETUPDATA = lpParameter as PNETWORKSETUPDATA;
    let mut i: UINT;
    let count: UINT;

    count = (core::mem::size_of_val(&NetworkComponents) / core::mem::size_of::<NETWORKCOMPONENT>()) as UINT;

    SendDlgItemMessage((*pNetworkSetupData).hwndPage, (IDC_INSTALL_PROGRESS as i32), PBM_SETRANGE, 0, MAKELPARAM!(0, count));
    SendDlgItemMessage((*pNetworkSetupData).hwndPage, (IDC_INSTALL_PROGRESS as i32), PBM_SETPOS, 0, 0);

    i = 0;
    while i < count {
        InstallNetworkComponent(NetworkComponents[i as usize].pszDeviceId, NetworkComponents[i as usize].pClassGuid);
        SendDlgItemMessage((*pNetworkSetupData).hwndPage, (IDC_INSTALL_PROGRESS as i32), PBM_SETPOS, (i) as _, 0);
        Sleep(500);
        i += 1;
    }

    /* Done */
    (*pNetworkSetupData).bInitialSetupRun = (TRUE as i32);
    PostMessage((*pNetworkSetupData).hwndPage, WM_USER, 0, 0);

    return 0;
}

unsafe extern "system" fn NetworkInitPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetworkSetupData: PNETWORKSETUPDATA = core::ptr::null_mut();
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();
    let mut lpnm: LPNMHDR = core::ptr::null_mut();

    /* Retrieve pointer to the global setup data */
    pNetworkSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PNETWORKSETUPDATA;
    if (pNetworkSetupData != core::ptr::null_mut()) &&
        ((*pNetworkSetupData).dwMagic == (NETWORK_SETUP_MAGIC as DWORD))
    {
        pSetupData = (*pNetworkSetupData).pSetupData;
    }

    match uMsg {
        WM_INITDIALOG => {
            /* Save pointer to the global setup data */
            pNetworkSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PNETWORKSETUPDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, (pNetworkSetupData as DWORD_PTR) as LONG_PTR);
            pSetupData = (*pNetworkSetupData).pSetupData;

            if (*pSetupData).UnattendSetup != 0 {
                //...
            }
        }

        WM_DESTROY => {
            /* ATTENTION: Free pNetworkSetupData only in one of the page functions!!! */
            //...
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Disable the Back and Next buttons */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), 0);

                    (*pNetworkSetupData).hwndPage = hwndDlg;
                    CreateThread(core::ptr::null_mut(), 0, Some(InstallThreadProc), pNetworkSetupData as LPVOID, 0, core::ptr::null_mut());
                }

                PSN_WIZNEXT => {}

                PSN_WIZBACK => {}

                _ => {}
            }
        }

        WM_USER => {
            TRACE!("WM_USER!\n");
            /* Enable the Next button and press it */
            PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
            PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_NEXT);
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn NetworkSettingsPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetworkSetupData: PNETWORKSETUPDATA = core::ptr::null_mut();
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();
    let mut lpnm: LPNMHDR = core::ptr::null_mut();

    /* Retrieve pointer to the global setup data */
    pNetworkSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PNETWORKSETUPDATA;
    if (pNetworkSetupData != core::ptr::null_mut()) &&
        ((*pNetworkSetupData).dwMagic == (NETWORK_SETUP_MAGIC as DWORD))
    {
        pSetupData = (*pNetworkSetupData).pSetupData;
    }

    match uMsg {
        WM_INITDIALOG => {
            /* Save pointer to the global setup data */
            pNetworkSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PNETWORKSETUPDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, (pNetworkSetupData as DWORD_PTR) as LONG_PTR);
            pSetupData = (*pNetworkSetupData).pSetupData;

            /* Set the fonts of both the options to bold */
            SetBoldText(hwndDlg, (IDC_NETWORK_TYPICAL) as INT, pSetupData);
            SetBoldText(hwndDlg, (IDC_NETWORK_CUSTOM) as INT, pSetupData);

            /* Set the typical settings option as the default */
            SendDlgItemMessage(hwndDlg, (IDC_NETWORK_TYPICAL as i32), BM_SETCHECK, BST_CHECKED as WPARAM, 0);

            if (*pSetupData).UnattendSetup != 0
            {
                //...
            }
            // break
        }

        WM_DESTROY => {
            /* ATTENTION: Free pNetworkSetupData only in one of the page functions!!! */
            //...
            // break
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Enable the Next button only */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0
                    {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (IDD_NETWORKCOMPONENTPAGE) as LONG_PTR);
                        return (TRUE) as INT_PTR;
                    }
                    // break
                }

                PSN_WIZNEXT => {
                    (*pNetworkSetupData).bTypicalNetworkSetup = (FALSE as i32);

                    /* If the Typical Settings button is chosen, then skip to the Domain Page */
                    if IsDlgButtonChecked(hwndDlg, (IDC_NETWORK_TYPICAL as i32)) == BST_CHECKED
                    {
                        (*pNetworkSetupData).bTypicalNetworkSetup = (TRUE as i32);

                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (IDD_NETWORKDOMAINPAGE) as LONG_PTR);
                        return (TRUE) as INT_PTR;
                    }
                    // break
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE as i32);
                    // break
                }

                _ => {}
            }
            // break
        }

        _ => {
            // break
        }
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn NetworkComponentPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetworkSetupData: PNETWORKSETUPDATA;
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();
    let mut lpnm: LPNMHDR;

    /* Retrieve pointer to the global setup data */
    pNetworkSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PNETWORKSETUPDATA;
    if !pNetworkSetupData.is_null() && (*pNetworkSetupData).dwMagic == (NETWORK_SETUP_MAGIC as DWORD) {
        pSetupData = (*pNetworkSetupData).pSetupData;
    }

    match uMsg {
        WM_INITDIALOG => {
            /* Save pointer to the global setup data */
            pNetworkSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PNETWORKSETUPDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, (pNetworkSetupData as DWORD_PTR) as LONG_PTR);
            pSetupData = (*pNetworkSetupData).pSetupData;

            SetBoldText(hwndDlg, (IDC_NETWORK_DEVICE) as INT, pSetupData);

            if (*pSetupData).UnattendSetup != 0 {
                //...
            }
        }

        WM_DESTROY => {
            /* ATTENTION: Free pNetworkSetupData only in one of the page functions!!! */
            //...
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Enable the Back and Next buttons */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (IDD_NETWORKDOMAINPAGE) as LONG_PTR);
                        return (TRUE) as INT_PTR;
                    }
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE as i32);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn NetworkDomainPageDlgProc(hwndDlg: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetworkSetupData: PNETWORKSETUPDATA = core::ptr::null_mut();
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();
    let mut DomainName: [u16; 51] = [0; 51];
    let mut Title: [u16; 64] = [0; 64];
    let mut ErrorName: [u16; 256] = [0; 256];
    let mut lpnm: LPNMHDR = core::ptr::null_mut();

    /* Retrieve pointer to the global setup data */
    pNetworkSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PNETWORKSETUPDATA;
    if (!pNetworkSetupData.is_null()) &&
        ((*pNetworkSetupData).dwMagic == (NETWORK_SETUP_MAGIC as DWORD)) {
        pSetupData = (*pNetworkSetupData).pSetupData;
    }

    match uMsg {
        WM_INITDIALOG => {
            /* Save pointer to the global setup data */
            pNetworkSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PNETWORKSETUPDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, (pNetworkSetupData as DWORD_PTR) as LONG_PTR);
            pSetupData = (*pNetworkSetupData).pSetupData;

            /* Set the workgroup option as the default */
            SendDlgItemMessage(hwndDlg, (IDC_SELECT_WORKGROUP as i32), BM_SETCHECK, BST_CHECKED as WPARAM, 0);

            wcscpy(DomainName.as_mut_ptr(), CPP_COM_WSTR_3E79A2679AF5.as_ptr());

            /* Display current computer name */
            SetDlgItemTextW(hwndDlg, (IDC_DOMAIN_NAME as i32), DomainName.as_ptr());

            /* Set focus to owner name */
            SetFocus(GetDlgItem(hwndDlg, (IDC_DOMAIN_NAME as i32)));

            if (*pSetupData).UnattendSetup != 0
            {
                //...
            }
        }

        WM_DESTROY => {
            //...
            /* ATTENTION: Free pNetworkSetupData only in one of the page functions!!! */
            if !pNetworkSetupData.is_null()
            {
                if (IsBadReadPtr(pNetworkSetupData as *const core::ffi::c_void, (core::mem::size_of::<NETWORKSETUPDATA>()) as UINT_PTR) == 0) &&
                    ((*pNetworkSetupData).dwMagic == (NETWORK_SETUP_MAGIC as DWORD))
                {
                    HeapFree(GetProcessHeap(), 0, pNetworkSetupData as LPVOID);
                }

                SetWindowLongPtr(hwndDlg, DWLP_USER, (0 as DWORD_PTR) as LONG_PTR);
            }
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Enable the Back and Next buttons */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0
                    {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, ((*pSetupData).uPostNetworkWizardPage as DWORD_PTR) as LONG_PTR);
                        return (TRUE) as INT_PTR;
                    }
                }

                PSN_WIZNEXT => {
                    DomainName[0] = 0;
                    if GetDlgItemTextW(hwndDlg, (IDC_DOMAIN_NAME as i32), DomainName.as_mut_ptr(), 50) == 0
                    {
                        if 0 == LoadStringW(netshell_hInstance, IDS_REACTOS_SETUP, Title.as_mut_ptr(), (core::mem::size_of::<[u16; 64]>() / core::mem::size_of_val(&Title[0])) as i32)
                        {
                            wcscpy(Title.as_mut_ptr(), CPP_COM_WSTR_AB7B4070E95B.as_ptr());
                        }
                        if 0 == LoadStringW(netshell_hInstance, IDS_WZD_DOMAIN_NAME, ErrorName.as_mut_ptr(), (core::mem::size_of::<[u16; 256]>() / core::mem::size_of_val(&ErrorName[0])) as i32)
                        {
                            wcscpy(ErrorName.as_mut_ptr(), CPP_COM_WSTR_05A95BD1856B.as_ptr());
                        }
                        MessageBoxW(hwndDlg, ErrorName.as_ptr(), Title.as_ptr(), MB_ICONERROR | MB_OK);

                        SetFocus(GetDlgItem(hwndDlg, (IDC_DOMAIN_NAME as i32)));
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (-1) as LONG_PTR);

                        //TODO: Implement setting the Domain/Workgroup

                        return (TRUE) as INT_PTR;
                    }
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE as i32);

                    /* If the Typical setup chosen, then skip back to the Settings Page */
                    if (*pNetworkSetupData).bTypicalNetworkSetup == (TRUE as BOOL)
                    {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (IDD_NETWORKSETTINGSPAGE) as LONG_PTR);
                        return (TRUE) as INT_PTR;
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn NetSetupRequestWizardPages(pPageCount: PDWORD, pPages: *mut HPROPSHEETPAGE, pSetupData: PSETUPDATA) -> DWORD {
    let mut pNetworkSetupData: PNETWORKSETUPDATA;
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();
    let mut dwPageCount: DWORD = 4;
    let mut nPage: INT = 0;

    if pPageCount.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if pPages.is_null() {
        *pPageCount = dwPageCount;
        return ERROR_SUCCESS;
    }

    if *pPageCount < dwPageCount {
        return ERROR_BUFFER_OVERFLOW;
    }

    (*pSetupData).uFirstNetworkWizardPage = IDD_NETWORKINITPAGE;

    pNetworkSetupData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<NETWORKSETUPDATA>()) as SIZE_T) as PNETWORKSETUPDATA;
    if pNetworkSetupData.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    (*pNetworkSetupData).dwMagic = (NETWORK_SETUP_MAGIC as u32);
    (*pNetworkSetupData).pSetupData = pSetupData;

    /* Create the Network Settings page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.hInstance = netshell_hInstance;
    psp.lParam = pNetworkSetupData as LPARAM;

    psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_NETWORKINITTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_NETWORKINITSUBTITLE);
    psp.pfnDlgProc = Some(NetworkInitPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_NETWORKINITPAGE);
    let idx = nPage;
    nPage += 1;
    *pPages.offset(idx as isize) = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_NETWORKSETTINGSTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_NETWORKSETTINGSSUBTITLE);
    psp.pfnDlgProc = Some(NetworkSettingsPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_NETWORKSETTINGSPAGE);
    let idx = nPage;
    nPage += 1;
    *pPages.offset(idx as isize) = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Network Components page */
    psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_NETWORKCOMPONENTTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_NETWORKCOMPONENTSUBTITLE);
    psp.pfnDlgProc = Some(NetworkComponentPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_NETWORKCOMPONENTPAGE);
    let idx = nPage;
    nPage += 1;
    *pPages.offset(idx as isize) = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Domain/Workgroup page */
    psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_NETWORKDOMAINTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_NETWORKDOMAINSUBTITLE);
    psp.pfnDlgProc = Some(NetworkDomainPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_NETWORKDOMAINPAGE);
    let idx = nPage;
    nPage += 1;
    *pPages.offset(idx as isize) = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    *pPageCount = dwPageCount;

    return ERROR_SUCCESS;
}

impl _NETWORKSETUPDATA {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl _NETWORKCOMPONENT {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
