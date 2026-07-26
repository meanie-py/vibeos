use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_COMPUTERDESCRIPTION_LENGTH: usize = 255;

pub const MAX_HOSTNAME_LENGTH: usize = 63;

pub const MAX_DOMAINNAME_LENGTH: usize = 255;

#[repr(C)]
pub struct _NETIDDATA {
    pub szHostName: [u16; MAX_HOSTNAME_LENGTH + 1],
    pub szOldHostName: [u16; MAX_HOSTNAME_LENGTH + 1],
    pub szDomainName: [u16; MAX_DOMAINNAME_LENGTH + 1],
    pub szOldDomainName: [u16; MAX_DOMAINNAME_LENGTH + 1],
    pub szComputerName: [u16; MAX_COMPUTERNAME_LENGTH + 1],
    pub bSyncDomainWithMembership: i32,
    pub JoinName: *mut u16,
    pub JoinStatus: u32,
    pub bHostNameChanged: i32,
    pub bDomainNameChanged: i32,
    pub bSyncDomainWithMembershipChanged: i32,
    pub bEnable: i32,
}

pub type NETIDDATA = _NETIDDATA;
pub type PNETIDDATA = *mut _NETIDDATA;

static mut hDllInstance: HINSTANCE = core::ptr::null_mut();
unsafe extern "C" fn FormatMessageBox(
    hDlg: HWND,
    uType: u32,
    dwMessage: u32,
    mut args: ...
) -> i32 {
    let mut szTitle: [u16; 256] = [0; 256];
    let mut szMessage: [u16; 256] = [0; 256];
    let mut szText: [u16; 512] = [0; 512];
    let mut args: va_list = core::ptr::null_mut();

    LoadStringW(
        hDllInstance,
        4,
        szTitle.as_mut_ptr(),
        ARRAYSIZE(szTitle.len())
    );

    LoadStringW(
        hDllInstance,
        dwMessage,
        szMessage.as_mut_ptr(),
        ARRAYSIZE(szMessage.len())
    );

    va_start(args, dwMessage);
    FormatMessageW(
        FORMAT_MESSAGE_FROM_STRING,
        szMessage.as_ptr(),
        0,
        0,
        szText.as_mut_ptr(),
        ARRAYSIZE(szText.len()),
        core::ptr::addr_of_mut!(args)
    );
    va_end(args);

    MessageBoxW(hDlg, szText.as_ptr(), szTitle.as_ptr(), uType)
}
unsafe extern "C" fn GetComputerNames(
    pNetIdData: *mut NETIDDATA,
) -> i32 {
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut dwSize: u32;
    let mut dwError: u32;

    dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0T\0c\0p\0I\0p\0\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0".as_ptr() as *const u16,
                            0,
                            KEY_QUERY_VALUE,
                            &mut KeyHandle);
    if dwError == ERROR_SUCCESS
    {
        dwSize = core::mem::size_of_val(&(*pNetIdData).szHostName) as u32;
        RegQueryValueExW(KeyHandle,
                         b"N\0V\0 \0H\0o\0s\0t\0N\0a\0m\0e\0\0\0".as_ptr() as *const u16,
                         0,
                         core::ptr::null_mut(),
                         (*pNetIdData).szHostName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).szOldHostName) as u32;
        RegQueryValueExW(KeyHandle,
                         b"H\0o\0s\0t\0N\0a\0m\0e\0\0\0".as_ptr() as *const u16,
                         0,
                         core::ptr::null_mut(),
                         (*pNetIdData).szOldHostName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).szDomainName) as u32;
        RegQueryValueExW(KeyHandle,
                         b"N\0V\0 \0D\0o\0m\0a\0i\0n\0\0\0".as_ptr() as *const u16,
                         0,
                         core::ptr::null_mut(),
                         (*pNetIdData).szDomainName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).szOldDomainName) as u32;
        RegQueryValueExW(KeyHandle,
                         b"D\0o\0m\0a\0i\0n\0\0\0".as_ptr() as *const u16,
                         0,
                         core::ptr::null_mut(),
                         (*pNetIdData).szOldDomainName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).bSyncDomainWithMembership) as u32;
        if RegQueryValueExW(KeyHandle,
                             b"S\0y\0n\0c\0D\0o\0m\0a\0i\0n\0W\0i\0t\0h\0M\0e\0m\0b\0e\0r\0s\0h\0i\0p\0\0\0".as_ptr() as *const u16,
                             0,
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!((*pNetIdData).bSyncDomainWithMembership) as LPBYTE,
                             &mut dwSize) != 0
        {
            (*pNetIdData).bSyncDomainWithMembership = TRUE;
        }

        RegCloseKey(KeyHandle);
    }

    dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0C\0o\0m\0p\0u\0t\0e\0r\0N\0a\0m\0e\0\\\0C\0o\0m\0p\0u\0t\0e\0r\0N\0a\0m\0e\0\0\0".as_ptr() as *const u16,
                            0,
                            KEY_QUERY_VALUE,
                            &mut KeyHandle);
    if dwError == ERROR_SUCCESS
    {
        dwSize = core::mem::size_of_val(&(*pNetIdData).szComputerName) as u32;
        RegQueryValueExW(KeyHandle,
                         b"C\0o\0m\0p\0u\0t\0e\0r\0N\0a\0m\0e\0\0\0".as_ptr() as *const u16,
                         0,
                         core::ptr::null_mut(),
                         (*pNetIdData).szComputerName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        RegCloseKey(KeyHandle);
    }

    if NetGetJoinInformation(core::ptr::null_mut(), &mut (*pNetIdData).JoinName, &mut (*pNetIdData).JoinStatus) != NERR_Success
    {
        (*pNetIdData).JoinName = core::ptr::null_mut();
        (*pNetIdData).JoinStatus = NetSetupUnknownStatus;
    }

    TRUE
}
unsafe extern "C" fn IsUserAdmin() -> BOOL {
    let mut bIsAdmin: BOOL = 0;
    let Authority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_NT_AUTHORITY,
    };
    let mut pAdminsSid: PSID = core::ptr::null_mut();

    if AllocateAndInitializeSid(
        &Authority as *const SID_IDENTIFIER_AUTHORITY as *mut SID_IDENTIFIER_AUTHORITY,
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut pAdminsSid,
    ) == 0
    {
        return FALSE;
    }

    if CheckTokenMembership(core::ptr::null_mut(), pAdminsSid, &mut bIsAdmin) == 0 {
        bIsAdmin = FALSE;
    }
    FreeSid(pAdminsSid);

    return bIsAdmin;
}
unsafe extern "C" fn IsValidDomainName(
    hDlg: HWND,
    uId: u32,
) -> u8 {
    let mut szDomainName: [u16; 256] = [0; 256];
    let dwError: u32;

    if GetDlgItemTextW(hDlg, uId, szDomainName.as_mut_ptr(), ARRAYSIZE(szDomainName.as_ptr())) == 0 {
        return TRUE;
    }

    dwError = DnsValidateName_W(szDomainName.as_ptr(), DnsNameDomain);
    if dwError != ERROR_SUCCESS {
        match dwError {
            DNS_ERROR_NON_RFC_NAME => {
                if FormatMessageBox(hDlg, MB_YESNO | MB_ICONWARNING, 7, szDomainName.as_ptr()) == IDYES {
                    return TRUE;
                }
            }

            ERROR_INVALID_NAME => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 8, szDomainName.as_ptr());
            }

            DNS_ERROR_NUMERIC_NAME => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1031, szDomainName.as_ptr());
            }

            DNS_ERROR_INVALID_NAME_CHAR => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1032, szDomainName.as_ptr());
            }

            _ => {}
        }

        return FALSE;
    }

    return TRUE;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn DNSSuffixPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg
    {
        WM_INITDIALOG => {
            pNetIdData = lParam as *mut NETIDDATA;
            if pNetIdData != core::ptr::null_mut()
            {
                SetWindowLongPtr(hDlg, DWLP_USER, pNetIdData as isize);

                SetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_ptr());
                SendDlgItemMessage(hDlg, 115, BM_SETCHECK, (*pNetIdData).bSyncDomainWithMembership as WPARAM, 0);
                SetDlgItemTextW(hDlg, 1013, (*pNetIdData).szComputerName.as_ptr());
                EnableWindow(GetDlgItem(hDlg, IDOK), FALSE);
            }
            return TRUE;
        }

        WM_COMMAND => {
            match LOWORD(wParam)
            {
                115 => {
                    if HIWORD(wParam) == BN_CLICKED
                    {
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                        (*pNetIdData).bSyncDomainWithMembershipChanged = TRUE;
                    }
                }

                1011 => {
                    if HIWORD(wParam) == EN_CHANGE
                    {
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                        (*pNetIdData).bDomainNameChanged = TRUE;
                    }
                }

                IDOK => {
                    if IsValidDomainName(hDlg, 1011) == 0
                    {
                        SetFocus(GetDlgItem(hDlg, 1011));
                        return FALSE;
                    }

                    if (*pNetIdData).bDomainNameChanged != 0
                    {
                        GetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_mut_ptr(), ARRAYSIZE((*pNetIdData).szDomainName.as_ptr()));
                    }

                    if (*pNetIdData).bSyncDomainWithMembershipChanged != 0
                    {
                        if SendDlgItemMessage(hDlg, 115, BM_GETCHECK, 0, 0) == BST_CHECKED
                        {
                            (*pNetIdData).bSyncDomainWithMembership = TRUE;
                        }
                        else
                        {
                            (*pNetIdData).bSyncDomainWithMembership = FALSE;
                        }
                    }

                    EndDialog(hDlg, TRUE);
                }

                IDCANCEL => {
                    (*pNetIdData).bDomainNameChanged = FALSE;
                    (*pNetIdData).bSyncDomainWithMembershipChanged = FALSE;
                    EndDialog(hDlg, FALSE);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DNSSuffixPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg
    {
        WM_INITDIALOG => {
            pNetIdData = lParam as *mut NETIDDATA;
            if pNetIdData != core::ptr::null_mut()
            {
                SetWindowLongPtr(hDlg, DWLP_USER, pNetIdData as isize);

                SetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_ptr());
                SendDlgItemMessage(hDlg, 115, BM_SETCHECK, (*pNetIdData).bSyncDomainWithMembership as WPARAM, 0);
                SetDlgItemTextW(hDlg, 1013, (*pNetIdData).szComputerName.as_ptr());
                EnableWindow(GetDlgItem(hDlg, IDOK), FALSE);
            }
            return TRUE;
        }

        WM_COMMAND => {
            match LOWORD(wParam)
            {
                115 => {
                    if HIWORD(wParam) == BN_CLICKED
                    {
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                        (*pNetIdData).bSyncDomainWithMembershipChanged = TRUE;
                    }
                }

                1011 => {
                    if HIWORD(wParam) == EN_CHANGE
                    {
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                        (*pNetIdData).bDomainNameChanged = TRUE;
                    }
                }

                IDOK => {
                    if IsValidDomainName(hDlg, 1011) == 0
                    {
                        SetFocus(GetDlgItem(hDlg, 1011));
                        return FALSE;
                    }

                    if (*pNetIdData).bDomainNameChanged != 0
                    {
                        GetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_mut_ptr(), ARRAYSIZE((*pNetIdData).szDomainName.as_ptr()));
                    }

                    if (*pNetIdData).bSyncDomainWithMembershipChanged != 0
                    {
                        if SendDlgItemMessage(hDlg, 115, BM_GETCHECK, 0, 0) == BST_CHECKED
                        {
                            (*pNetIdData).bSyncDomainWithMembership = TRUE;
                        }
                        else
                        {
                            (*pNetIdData).bSyncDomainWithMembership = FALSE;
                        }
                    }

                    EndDialog(hDlg, TRUE);
                }

                IDCANCEL => {
                    (*pNetIdData).bDomainNameChanged = FALSE;
                    (*pNetIdData).bSyncDomainWithMembershipChanged = FALSE;
                    EndDialog(hDlg, FALSE);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}
unsafe extern "C" fn SetRadioBtnState(hDlg: HWND, IsDomain: BOOL) {
    SendDlgItemMessage(hDlg, 1008, BM_SETCHECK, IsDomain as WPARAM, 0);
    SendDlgItemMessage(hDlg, 1004, BM_SETCHECK, (if IsDomain != 0 { 0 } else { 1 }) as WPARAM, 0);
    EnableWindow(GetDlgItem(hDlg, 116), IsDomain);
    EnableWindow(GetDlgItem(hDlg, 1007), if IsDomain != 0 { 0 } else { 1 });
}
unsafe extern "C" fn IsValidComputerName(
    hDlg: HWND,
    uId: u32,
) -> i32 {
    let mut szHostName: [u16; 256] = [0; 256];
    let dwError: u32;

    GetWindowText(GetDlgItem(hDlg, uId), szHostName.as_mut_ptr(), ARRAYSIZE!(szHostName));

    dwError = DnsValidateName_W(szHostName.as_ptr(), DnsNameHostnameLabel);
    if dwError != ERROR_SUCCESS {
        match dwError {
            DNS_ERROR_NON_RFC_NAME => {
                if FormatMessageBox(hDlg, MB_YESNO | MB_ICONWARNING, 10, szHostName.as_ptr()) == IDYES {
                    return TRUE;
                }
            }

            ERROR_INVALID_NAME => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 11);
                return FALSE;
            }

            DNS_ERROR_NUMERIC_NAME => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1029, szHostName.as_ptr());
            }

            DNS_ERROR_INVALID_NAME_CHAR => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1030, szHostName.as_ptr());
            }

            _ => {}
        }

        return FALSE;
    }

    return TRUE;
}
unsafe extern "C" fn SetFullComputerName(
    hDlg: HWND,
    uId: u32,
    pNetIdData: *mut _NETIDDATA,
) {
    let mut szFullComputerName: [u16; 512] = [0; 512];

    wsprintf(
        szFullComputerName.as_mut_ptr(),
        b"L\0%\0s\0.\0%\0s\0\0\0".as_ptr() as *const u16,
        (*pNetIdData).szHostName.as_ptr(),
        (*pNetIdData).szDomainName.as_ptr(),
    );
    SetDlgItemText(hDlg, uId, szFullComputerName.as_ptr());
}
unsafe extern "C" fn UpdateFullComputerName(
    hDlg: HWND,
    uId: u32,
    pNetIdData: *mut NETIDDATA,
) {
    let mut szFullComputerName: [u16; 512] = [0; 512];
    let mut szHostName: [u16; 256] = [0; 256];

    GetWindowText(GetDlgItem(hDlg, 1002), szHostName.as_mut_ptr(), ARRAYSIZE(szHostName.as_ptr()));

    wsprintf(szFullComputerName.as_mut_ptr(), b"L\"%s.%s\"\0".as_ptr(), szHostName.as_ptr(), (*pNetIdData).szDomainName.as_ptr());
    SetDlgItemText(hDlg, uId, szFullComputerName.as_ptr());
}
unsafe extern "C" fn UpdateNetbiosName(
    hDlg: HWND,
    uId: u32,
    pNetIdData: *mut NETIDDATA,
) {
    let mut szHostName: [u16; 256] = [0; 256];
    let mut dwSize: u32;

    GetWindowText(GetDlgItem(hDlg, 1002), szHostName.as_mut_ptr(), ARRAYSIZE(szHostName.len()));

    dwSize = ARRAYSIZE((*pNetIdData).szComputerName.len());
    DnsHostnameToComputerNameW(szHostName.as_ptr(),
                           (*pNetIdData).szComputerName.as_mut_ptr(),
                           &mut dwSize);
}
unsafe extern "C" fn CheckForChangedNames(
    hwndDlg: HWND,
    pNetIdData: *mut NETIDDATA,
) {
    let mut nShow: i32 = SW_HIDE;

    if (_wcsicmp((*pNetIdData).szOldHostName.as_ptr(), (*pNetIdData).szHostName.as_ptr()) != 0) ||
       (_wcsicmp((*pNetIdData).szOldDomainName.as_ptr(), (*pNetIdData).szDomainName.as_ptr()) != 0) {
        nShow = SW_SHOW;
    }

    ShowWindow(GetDlgItem(hwndDlg, IDC_RESTARTICON), nShow);
    ShowWindow(GetDlgItem(hwndDlg, IDC_RESTARTTEXT), nShow);
}
unsafe extern "C" fn NetworkDlg_OnInitDialog(
    hDlg: HWND,
    pNetIdData: *mut _NETIDDATA,
) {
    let mut MsgText: [u16; MAX_PATH * 2] = [0; MAX_PATH * 2];

    if LoadStringW(hDllInstance, 25, MsgText.as_mut_ptr(), ARRAYSIZE!(MsgText)) != 0 {
        SetDlgItemText(hDlg, 1017, MsgText.as_ptr());
    }

    SendMessage(GetDlgItem(hDlg, 1002), EM_SETLIMITTEXT, MAX_HOSTNAME_LENGTH, 0);
    SetDlgItemText(hDlg, 1002, (*pNetIdData).szHostName.as_ptr());
    SetFullComputerName(hDlg, 1001, pNetIdData);

    match (*pNetIdData).JoinStatus {
        NetSetupDomainName => {
            SetDlgItemText(hDlg, 116, (*pNetIdData).JoinName);
            SetRadioBtnState(hDlg, TRUE);
        }

        NetSetupWorkgroupName => {
            SetDlgItemText(hDlg, 1007, (*pNetIdData).JoinName);
            SetRadioBtnState(hDlg, FALSE);
        }

        _ => {}
    }
}
unsafe extern "C" fn NetworkDlg_OnOK(
    hDlg: HWND,
    pNetIdData: *mut NETIDDATA,
) -> i32 {
    let mut szMsgText: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut szMsgTitle: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut dwError: u32;

    if (*pNetIdData).bHostNameChanged != 0 {
        if IsValidComputerName(hDlg, 1002) == 0 {
            SetFocus(GetDlgItem(hDlg, 1002));
            return FALSE;
        }

        GetWindowText(GetDlgItem(hDlg, 1002), (*pNetIdData).szHostName.as_mut_ptr(), ARRAYSIZE((*pNetIdData).szHostName.as_ptr()));

        if SetComputerNameExW(ComputerNamePhysicalDnsHostname, (*pNetIdData).szHostName.as_mut_ptr()) == 0 {
            LoadStringW(hDllInstance, 4001, szMsgText.as_mut_ptr(), ARRAYSIZE(szMsgText.as_ptr()));
            MessageBoxW(hDlg, szMsgText.as_mut_ptr(), core::ptr::null_mut(), MB_OK | MB_ICONERROR);
            return FALSE;
        }

        (*pNetIdData).bHostNameChanged = FALSE;
    }

    if (*pNetIdData).bDomainNameChanged != 0 {
        if SetComputerNameExW(ComputerNamePhysicalDnsDomain, (*pNetIdData).szDomainName.as_mut_ptr()) == 0 {
            return FALSE;
        }

        (*pNetIdData).bDomainNameChanged = FALSE;
    }

    if (*pNetIdData).bSyncDomainWithMembershipChanged != 0 {
        dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                                b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0T\0c\0p\0I\0p\0\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0".as_ptr() as *const u16,
                                0,
                                KEY_SET_VALUE,
                                &mut KeyHandle);
        if dwError == ERROR_SUCCESS {
            RegSetValueExW(KeyHandle,
                           b"S\0y\0n\0c\0D\0o\0m\0a\0i\0n\0W\0i\0t\0h\0M\0e\0m\0b\0e\0r\0s\0h\0i\0p\0\0\0".as_ptr() as *const u16,
                           0,
                           REG_DWORD,
                           core::ptr::addr_of_mut!((*pNetIdData).bSyncDomainWithMembership) as LPBYTE,
                           core::mem::size_of_val(&(*pNetIdData).bSyncDomainWithMembership) as u32);
            RegCloseKey(KeyHandle);
        }

        (*pNetIdData).bSyncDomainWithMembershipChanged = FALSE;
    }

    LoadStringW(hDllInstance, 4000, szMsgTitle.as_mut_ptr(), ARRAYSIZE(szMsgTitle.as_ptr()));
    LoadStringW(hDllInstance, 24, szMsgText.as_mut_ptr(), ARRAYSIZE(szMsgText.as_ptr()));
    MessageBoxW(hDlg, szMsgText.as_mut_ptr(), szMsgTitle.as_mut_ptr(), MB_OK | MB_ICONINFORMATION);

    return TRUE;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn NetworkPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg {
        WM_INITDIALOG => {
            let pNetIdData = lParam as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hDlg, DWLP_USER, pNetIdData as isize);
                NetworkDlg_OnInitDialog(hDlg, pNetIdData);
                EnableWindow(GetDlgItem(hDlg, IDOK), FALSE);
            }
            return TRUE;
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                1002 => {
                    if HIWORD(wParam) == EN_CHANGE {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        UpdateNetbiosName(hDlg, 1001, pNetIdData);
                        (*pNetIdData).bHostNameChanged = TRUE;
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                    }
                }

                1008 | 1004 => {
                    if SendDlgItemMessage(hDlg, 1008, BM_GETCHECK, 0, 0) == BST_CHECKED {
                        SetRadioBtnState(hDlg, TRUE);
                    } else {
                        SetRadioBtnState(hDlg, FALSE);
                    }
                }

                1003 => {
                    if DialogBoxParam(hDllInstance,
                                   MAKEINTRESOURCE(IDD_PROPPAGEDNSANDNETBIOS),
                                   hDlg,
                                   DNSSuffixPropDlgProc,
                                   pNetIdData as LPARAM) != 0 {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                    }
                }

                IDOK => {
                    if NetworkDlg_OnOK(hDlg, pNetIdData) != 0 {
                        EndDialog(hDlg, TRUE);
                    }
                }

                IDCANCEL => {
                    EndDialog(hDlg, FALSE);
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn NetworkPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg {
        WM_INITDIALOG => {
            let pNetIdData = lParam as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hDlg, DWLP_USER, pNetIdData as isize);
                NetworkDlg_OnInitDialog(hDlg, pNetIdData);
                EnableWindow(GetDlgItem(hDlg, IDOK), FALSE);
            }
            return TRUE;
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                1002 => {
                    if HIWORD(wParam) == EN_CHANGE {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        UpdateNetbiosName(hDlg, 1001, pNetIdData);
                        (*pNetIdData).bHostNameChanged = TRUE;
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                    }
                }

                1008 | 1004 => {
                    if SendDlgItemMessage(hDlg, 1008, BM_GETCHECK, 0, 0) == BST_CHECKED {
                        SetRadioBtnState(hDlg, TRUE);
                    } else {
                        SetRadioBtnState(hDlg, FALSE);
                    }
                }

                1003 => {
                    if DialogBoxParam(hDllInstance,
                                   MAKEINTRESOURCE(IDD_PROPPAGEDNSANDNETBIOS),
                                   hDlg,
                                   DNSSuffixPropDlgProc,
                                   pNetIdData as LPARAM) != 0 {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
                    }
                }

                IDOK => {
                    if NetworkDlg_OnOK(hDlg, pNetIdData) != 0 {
                        EndDialog(hDlg, TRUE);
                    }
                }

                IDCANCEL => {
                    EndDialog(hDlg, FALSE);
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}
unsafe extern "C" fn NetIDPage_OnInitDialog(
    hwndDlg: HWND,
    pNetIdData: *mut NETIDDATA) {
    let mut ComputerDescription: [u16; MAX_COMPUTERDESCRIPTION_LENGTH + 1] = [0; MAX_COMPUTERDESCRIPTION_LENGTH + 1];
    let mut RegSize: u32 = (core::mem::size_of_val(&ComputerDescription)) as u32;
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut szBuffer: [u16; 512] = [0; 512];
    let mut lError: i32;

    SendDlgItemMessage(hwndDlg, IDC_COMPDESC, EM_SETLIMITTEXT, MAX_COMPUTERDESCRIPTION_LENGTH, 0);

    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                           b"SYSTEM\\CurrentControlSet\\Services\\LanmanServer\\Parameters\0".as_ptr() as *const u16,
                           0,
                           KEY_QUERY_VALUE,
                           &mut KeyHandle);
    if lError == ERROR_SUCCESS
    {
        lError = RegQueryValueExW(KeyHandle,
                                  b"srvcomment\0".as_ptr() as *const u16,
                                  0,
                                  core::ptr::null_mut(),
                                  ComputerDescription.as_mut_ptr() as LPBYTE,
                                  &mut RegSize);
        if lError == ERROR_SUCCESS
        {
            ComputerDescription[(RegSize / core::mem::size_of::<u16>() as u32) as usize] = UNICODE_NULL;
            SetDlgItemText(hwndDlg, IDC_COMPDESC, ComputerDescription.as_ptr());
        }

        RegCloseKey(KeyHandle);
    }

    LoadStringW(hDllInstance, if (*pNetIdData).JoinStatus == NetSetupDomainName { 6 } else { 5 }, szBuffer.as_mut_ptr(), ARRAYSIZE(szBuffer.as_ptr()));
    SetDlgItemText(hwndDlg, IDC_WORKGROUPDOMAIN, szBuffer.as_ptr());
    SetDlgItemText(hwndDlg, IDC_WORKGROUPDOMAIN_NAME, (*pNetIdData).JoinName);

    if IsUserAdmin() == 0
    {
        LoadStringW(hDllInstance, 1021, szBuffer.as_mut_ptr(), ARRAYSIZE(szBuffer.as_ptr()));
        SetDlgItemText(hwndDlg, IDC_MESSAGETEXT, szBuffer.as_ptr());

        EnableWindow(GetDlgItem(hwndDlg, IDC_NETWORK_ID), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_NETWORK_PROPERTY), FALSE);
    }
}
unsafe extern "C" fn NetIDPage_OnApply(
    hwndDlg: HWND) -> i32 {
    let mut ComputerDescription: [u16; MAX_COMPUTERDESCRIPTION_LENGTH + 1] = core::mem::zeroed();
    let mut NewComputerDescription: [u16; MAX_COMPUTERDESCRIPTION_LENGTH + 1] = core::mem::zeroed();
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut dwSize: u32;
    let mut lError: i32;

    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                           b"SYSTEM\\CurrentControlSet\\Services\\LanmanServer\\Parameters\0".as_ptr() as *const u16,
                           0,
                           KEY_QUERY_VALUE | KEY_SET_VALUE,
                           &mut KeyHandle);
    if lError != ERROR_SUCCESS {
        return lError;
    }

    GetDlgItemTextW(hwndDlg,
                    IDC_COMPDESC,
                    NewComputerDescription.as_mut_ptr(),
                    ARRAYSIZE(NewComputerDescription.as_ptr() as *const _));
    if GetLastError() != ERROR_SUCCESS {
        lError = GetLastError();
        'done: loop {
            if KeyHandle != core::ptr::null_mut() {
                RegCloseKey(KeyHandle);
            }
            return lError;
        }
    }

    dwSize = core::mem::size_of_val(&ComputerDescription) as u32;
    lError = RegQueryValueExW(KeyHandle,
                              b"srvcomment\0".as_ptr() as *const u16,
                              0,
                              core::ptr::null_mut(),
                              ComputerDescription.as_mut_ptr() as PBYTE,
                              &mut dwSize);
    if lError != ERROR_SUCCESS && lError != ERROR_FILE_NOT_FOUND {
        'done: loop {
            if KeyHandle != core::ptr::null_mut() {
                RegCloseKey(KeyHandle);
            }
            return lError;
        }
    }

    lError = ERROR_SUCCESS;
    if wcscmp(ComputerDescription.as_ptr(), NewComputerDescription.as_ptr()) != 0 {
        lError = RegSetValueExW(KeyHandle,
                                b"srvcomment\0".as_ptr() as *const u16,
                                0,
                                REG_SZ,
                                NewComputerDescription.as_ptr() as PBYTE,
                                (wcslen(NewComputerDescription.as_ptr()) + 1) * core::mem::size_of::<u16>() as u32);
    }

    if KeyHandle != core::ptr::null_mut() {
        RegCloseKey(KeyHandle);
    }

    lError
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn NetIDPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut NETIDDATA;

    match uMsg {
        WM_INITDIALOG => {
            pNetIdData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<NETIDDATA>(),
            ) as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hwndDlg, DWLP_USER, pNetIdData as isize);
                GetComputerNames(pNetIdData);
                SetFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
                NetIDPage_OnInitDialog(hwndDlg, pNetIdData);
                (*pNetIdData).bEnable = TRUE;
                CheckForChangedNames(hwndDlg, pNetIdData);
            }
            return TRUE;
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code {
                PSN_APPLY => {
                    NetIDPage_OnApply(hwndDlg);
                }
                _ => {}
            }
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDC_COMPDESC => {
                    if HIWORD(wParam) == EN_CHANGE && (*pNetIdData).bEnable == TRUE {
                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_NETWORK_PROPERTY => {
                    if DialogBoxParam(
                        hDllInstance,
                        MAKEINTRESOURCE(IDD_PROPPAGECOMPNAMECHENGE),
                        hwndDlg,
                        NetworkPropDlgProc,
                        pNetIdData as LPARAM,
                    ) != 0
                    {
                        UpdateFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
                        CheckForChangedNames(hwndDlg, pNetIdData);
                    }
                }
                _ => {}
            }
        }

        WM_DESTROY => {
            if !pNetIdData.is_null() {
                if !(*pNetIdData).JoinName.is_null() {
                    NetApiBufferFree((*pNetIdData).JoinName);
                }
                HeapFree(GetProcessHeap(), 0, pNetIdData as LPVOID);
                pNetIdData = core::ptr::null_mut();
                SetWindowLongPtr(hwndDlg, DWLP_USER, 0 as isize);
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn NetIDPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut NETIDDATA;

    match uMsg {
        WM_INITDIALOG => {
            pNetIdData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<NETIDDATA>(),
            ) as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hwndDlg, DWLP_USER, pNetIdData as isize);
                GetComputerNames(pNetIdData);
                SetFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
                NetIDPage_OnInitDialog(hwndDlg, pNetIdData);
                (*pNetIdData).bEnable = TRUE;
                CheckForChangedNames(hwndDlg, pNetIdData);
            }
            return TRUE;
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code {
                PSN_APPLY => {
                    NetIDPage_OnApply(hwndDlg);
                }
                _ => {}
            }
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDC_COMPDESC => {
                    if HIWORD(wParam) == EN_CHANGE && (*pNetIdData).bEnable == TRUE {
                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_NETWORK_PROPERTY => {
                    if DialogBoxParam(
                        hDllInstance,
                        MAKEINTRESOURCE(IDD_PROPPAGECOMPNAMECHENGE),
                        hwndDlg,
                        NetworkPropDlgProc,
                        pNetIdData as LPARAM,
                    ) != 0
                    {
                        UpdateFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
                        CheckForChangedNames(hwndDlg, pNetIdData);
                    }
                }
                _ => {}
            }
        }

        WM_DESTROY => {
            if !pNetIdData.is_null() {
                if !(*pNetIdData).JoinName.is_null() {
                    NetApiBufferFree((*pNetIdData).JoinName);
                }
                HeapFree(GetProcessHeap(), 0, pNetIdData as LPVOID);
                pNetIdData = core::ptr::null_mut();
                SetWindowLongPtr(hwndDlg, DWLP_USER, 0 as isize);
            }
        }

        _ => {}
    }

    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn CreateNetIDPropertyPage() -> HPROPSHEETPAGE {
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();

    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
    psp.dwFlags = PSP_DEFAULT;
    psp.hInstance = hDllInstance;
    psp.pszTemplate = MAKEINTRESOURCE(IDD_PROPPAGECOMPUTER);
    psp.pfnDlgProc = Some(NetIDPageProc);

    CreatePropertySheetPage(&mut psp)
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: u32,
    lpvReserved: LPVOID,
) -> i32 {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            hDllInstance = hinstDLL;
            DisableThreadLibraryCalls(hinstDLL);
        }
        _ => {}
    }

    TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/netid/netid.c
//  * PURPOSE:     Network ID Page
//  * COPYRIGHT:   Thomas Weidenmueller <w3seek@reactos.org>
//  *              Dmitry Chapyshev <dmitry@reactos.org>
//  *
//  */
//
// #define WIN32_NO_STATUS
// #include <stdarg.h>
// #include <windef.h>
// #include <winbase.h>
// #include <winuser.h>
// #include <winreg.h>
// #include <windns.h>
// #include <lm.h>
// #include <prsht.h>
//
// #include "resource.h"
//
//
// #define MAX_COMPUTERDESCRIPTION_LENGTH 255
// #define MAX_HOSTNAME_LENGTH             63
// #define MAX_DOMAINNAME_LENGTH          255
//
// typedef struct _NETIDDATA
// {
//     WCHAR szHostName[MAX_HOSTNAME_LENGTH + 1];
//     WCHAR szOldHostName[MAX_HOSTNAME_LENGTH + 1];
//     WCHAR szDomainName[MAX_DOMAINNAME_LENGTH + 1];
//     WCHAR szOldDomainName[MAX_DOMAINNAME_LENGTH + 1];
//     WCHAR szComputerName[MAX_COMPUTERNAME_LENGTH + 1];
//     BOOL bSyncDomainWithMembership;
//     LPWSTR JoinName;
//     NETSETUP_JOIN_STATUS JoinStatus;
//     BOOL bHostNameChanged;
//     BOOL bDomainNameChanged;
//     BOOL bSyncDomainWithMembershipChanged;
//     BOOL bEnable;
// } NETIDDATA, *PNETIDDATA;
//
//
// static HINSTANCE hDllInstance;
//
// static
// INT
// FormatMessageBox(
//     HWND hDlg,
//     UINT uType,
//     DWORD dwMessage,
//     ...)
// {
//     WCHAR szTitle[256], szMessage[256], szText[512];
//     va_list args = NULL;
//
//     LoadStringW(hDllInstance, 4, szTitle, ARRAYSIZE(szTitle));
//
//     LoadStringW(hDllInstance, dwMessage, szMessage, ARRAYSIZE(szMessage));
//
//     va_start(args, dwMessage);
//     FormatMessageW(FORMAT_MESSAGE_FROM_STRING,
//                    szMessage,
//                    0,
//                    0,
//                    szText,
//                    ARRAYSIZE(szText),
//                    &args);
//     va_end(args);
//
//     return MessageBoxW(hDlg, szText, szTitle, uType);
// }
//
// static
// BOOL
// GetComputerNames(
//     PNETIDDATA pNetIdData)
// {
//     HKEY KeyHandle;
//     DWORD dwSize;
//     DWORD dwError;
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\CurrentControlSet\\Services\\TcpIp\\Parameters",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &KeyHandle);
//     if (dwError == ERROR_SUCCESS)
//     {
//         dwSize = sizeof(pNetIdData->szHostName);
//         RegQueryValueExW(KeyHandle,
//                          L"NV HostName",
//                          0,
//                          NULL,
//                          (LPBYTE)&pNetIdData->szHostName,
//                          &dwSize);
//
//         dwSize = sizeof(pNetIdData->szOldHostName);
//         RegQueryValueExW(KeyHandle,
//                          L"HostName",
//                          0,
//                          NULL,
//                          (LPBYTE)&pNetIdData->szOldHostName,
//                          &dwSize);
//
//         dwSize = sizeof(pNetIdData->szDomainName);
//         RegQueryValueExW(KeyHandle,
//                          L"NV Domain",
//                          0,
//                          NULL,
//                          (LPBYTE)&pNetIdData->szDomainName,
//                          &dwSize);
//
//         dwSize = sizeof(pNetIdData->szOldDomainName);
//         RegQueryValueExW(KeyHandle,
//                          L"Domain",
//                          0,
//                          NULL,
//                          (LPBYTE)&pNetIdData->szOldDomainName,
//                          &dwSize);
//
//         dwSize = sizeof(pNetIdData->bSyncDomainWithMembership);
//         if (RegQueryValueExW(KeyHandle,
//                              L"SyncDomainWithMembership",
//                              0,
//                              NULL,
//                              (LPBYTE)&pNetIdData->bSyncDomainWithMembership,
//                              &dwSize))
//             pNetIdData->bSyncDomainWithMembership = TRUE;
//
//         RegCloseKey(KeyHandle);
//     }
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\CurrentControlSet\\Control\\ComputerName\\ComputerName",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &KeyHandle);
//     if (dwError == ERROR_SUCCESS)
//     {
//         dwSize = sizeof(pNetIdData->szComputerName);
//         RegQueryValueExW(KeyHandle,
//                          L"ComputerName",
//                          0,
//                          NULL,
//                          (LPBYTE)&pNetIdData->szComputerName,
//                          &dwSize);
//
//         RegCloseKey(KeyHandle);
//     }
//
//     if (NetGetJoinInformation(NULL, &pNetIdData->JoinName, &pNetIdData->JoinStatus) != NERR_Success)
//     {
//         pNetIdData->JoinName = NULL;
//         pNetIdData->JoinStatus = NetSetupUnknownStatus;
//     }
//
//     return TRUE;
// }
//
// static BOOL
// IsUserAdmin(VOID)
// {
//     BOOL bIsAdmin;
//     SID_IDENTIFIER_AUTHORITY Authority = {SECURITY_NT_AUTHORITY};
//     PSID pAdminsSid;
//
//     if (!AllocateAndInitializeSid(&Authority, 2,
//                                   SECURITY_BUILTIN_DOMAIN_RID,
//                                   DOMAIN_ALIAS_RID_ADMINS,
//                                   0, 0, 0, 0, 0, 0,
//                                   &pAdminsSid))
//     {
//         return FALSE;
//     }
//
//     if (!CheckTokenMembership(NULL, pAdminsSid, &bIsAdmin))
//         bIsAdmin = FALSE;
//     FreeSid(pAdminsSid);
//
//     return bIsAdmin;
// }
//
// static
// BOOL
// IsValidDomainName(
//     HWND hDlg,
//     UINT uId)
// {
//     WCHAR szDomainName[256];
//     DWORD dwError;
//
//     if (GetDlgItemTextW(hDlg, uId, szDomainName, ARRAYSIZE(szDomainName)) == 0)
//         return TRUE;
//
//     dwError = DnsValidateName_W(szDomainName, DnsNameDomain);
//     if (dwError != ERROR_SUCCESS)
//     {
//         switch (dwError)
//         {
//             case DNS_ERROR_NON_RFC_NAME:
//                 if (FormatMessageBox(hDlg, MB_YESNO | MB_ICONWARNING, 7, szDomainName) == IDYES)
//                     return TRUE;
//                 break;
//
//             case ERROR_INVALID_NAME:
//                 FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 8, szDomainName);
//                 break;
//
//             case DNS_ERROR_NUMERIC_NAME:
//                 FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1031, szDomainName);
//                 break;
//
//             case DNS_ERROR_INVALID_NAME_CHAR:
//                 FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1032, szDomainName);
//                 break;
//         }
//
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static
// INT_PTR __stdcall
// DNSSuffixPropDlgProc(HWND hDlg, UINT Msg, WPARAM wParam, LPARAM lParam)
// {
//     PNETIDDATA pNetIdData;
//
//     pNetIdData = (PNETIDDATA)GetWindowLongPtr(hDlg, DWLP_USER);
//
//     switch (Msg)
//     {
//         case WM_INITDIALOG:
//             pNetIdData = (PNETIDDATA)lParam;
//             if (pNetIdData != NULL)
//             {
//                 SetWindowLongPtr(hDlg, DWLP_USER, (LONG_PTR)pNetIdData);
//
//                 SetDlgItemTextW(hDlg, 1011, pNetIdData->szDomainName);
//                 SendDlgItemMessage(hDlg, 115, BM_SETCHECK, (WPARAM)pNetIdData->bSyncDomainWithMembership, 0);
//                 SetDlgItemTextW(hDlg, 1013, pNetIdData->szComputerName);
//                 EnableWindow(GetDlgItem(hDlg, IDOK), FALSE);
//             }
//             return TRUE;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case 115:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
//                         pNetIdData->bSyncDomainWithMembershipChanged = TRUE;
//                     }
//                     break;
//
//                 case 1011:
//                     if (HIWORD(wParam) == EN_CHANGE)
//                     {
//                         EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
//                         pNetIdData->bDomainNameChanged = TRUE;
//                     }
//                     break;
//
//                 case IDOK:
//                     if (!IsValidDomainName(hDlg, 1011))
//                     {
//                         SetFocus(GetDlgItem(hDlg, 1011));
//                         break;
//                     }
//
//                     if (pNetIdData->bDomainNameChanged)
//                         GetDlgItemTextW(hDlg, 1011, pNetIdData->szDomainName, ARRAYSIZE(pNetIdData->szDomainName));
//
//                     if (pNetIdData->bSyncDomainWithMembershipChanged)
//                     {
//                         if (SendDlgItemMessage(hDlg, 115, BM_GETCHECK, 0, 0) == BST_CHECKED)
//                             pNetIdData->bSyncDomainWithMembership = TRUE;
//                         else
//                             pNetIdData->bSyncDomainWithMembership = FALSE;
//                     }
//
//                     EndDialog(hDlg, TRUE);
//                     break;
//
//                 case IDCANCEL:
//                     pNetIdData->bDomainNameChanged = FALSE;
//                     pNetIdData->bSyncDomainWithMembershipChanged = FALSE;
//                     EndDialog(hDlg, FALSE);
//                     break;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// static VOID
// SetRadioBtnState(HWND hDlg, BOOL IsDomain)
// {
//     SendDlgItemMessage(hDlg, 1008, BM_SETCHECK, (WPARAM)IsDomain, 0);
//     SendDlgItemMessage(hDlg, 1004, BM_SETCHECK, (WPARAM)!IsDomain, 0);
//     EnableWindow(GetDlgItem(hDlg, 116), IsDomain);
//     EnableWindow(GetDlgItem(hDlg, 1007), !IsDomain);
// }
//
//
// static
// BOOL
// IsValidComputerName(
//     HWND hDlg,
//     UINT uId)
// {
//     WCHAR szHostName[256];
//     DWORD dwError;
//
//     GetWindowText(GetDlgItem(hDlg, uId), szHostName, ARRAYSIZE(szHostName));
//
//     dwError = DnsValidateName_W(szHostName, DnsNameHostnameLabel);
//     if (dwError != ERROR_SUCCESS)
//     {
//         switch (dwError)
//         {
//             case DNS_ERROR_NON_RFC_NAME:
//                 if (FormatMessageBox(hDlg, MB_YESNO | MB_ICONWARNING, 10, szHostName) == IDYES)
//                     return TRUE;
//                 break;
//
//             case ERROR_INVALID_NAME:
//                 FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 11);
//                 return FALSE;
//
//             case DNS_ERROR_NUMERIC_NAME:
//                 FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1029, szHostName);
//                 break;
//
//             case DNS_ERROR_INVALID_NAME_CHAR:
//                 FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1030, szHostName);
//                 break;
//         }
//
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static
// VOID
// SetFullComputerName(
//     HWND hDlg,
//     UINT uId,
//     PNETIDDATA pNetIdData)
// {
//     WCHAR szFullComputerName[512];
//
//     wsprintf(szFullComputerName, L"%s.%s", pNetIdData->szHostName, pNetIdData->szDomainName);
//     SetDlgItemText(hDlg, uId, szFullComputerName);
// }
//
// static
// VOID
// UpdateFullComputerName(
//     HWND hDlg,
//     UINT uId,
//     PNETIDDATA pNetIdData)
// {
//     WCHAR szFullComputerName[512];
//     WCHAR szHostName[256];
//
//     GetWindowText(GetDlgItem(hDlg, 1002), szHostName, ARRAYSIZE(szHostName));
//
//     wsprintf(szFullComputerName, L"%s.%s", szHostName, pNetIdData->szDomainName);
//     SetDlgItemText(hDlg, uId, szFullComputerName);
// }
//
// static
// VOID
// UpdateNetbiosName(
//     HWND hDlg,
//     UINT uId,
//     PNETIDDATA pNetIdData)
// {
//     WCHAR szHostName[256];
//     DWORD dwSize;
//
//     GetWindowText(GetDlgItem(hDlg, 1002), szHostName, ARRAYSIZE(szHostName));
//
//     dwSize = ARRAYSIZE(pNetIdData->szComputerName);
//     DnsHostnameToComputerNameW(szHostName,
//                                pNetIdData->szComputerName,
//                                &dwSize);
// }
//
// static
// VOID
// CheckForChangedNames(
//      HWND hwndDlg,
//      PNETIDDATA pNetIdData)
// {
//     INT nShow = SW_HIDE;
//
//     if ((_wcsicmp(pNetIdData->szOldHostName, pNetIdData->szHostName) != 0) ||
//         (_wcsicmp(pNetIdData->szOldDomainName, pNetIdData->szDomainName) != 0))
//     {
//         nShow = SW_SHOW;
//     }
//
//     ShowWindow(GetDlgItem(hwndDlg, IDC_RESTARTICON), nShow);
//     ShowWindow(GetDlgItem(hwndDlg, IDC_RESTARTTEXT), nShow);
// }
//
// static
// VOID
// NetworkDlg_OnInitDialog(
//     HWND hDlg,
//     PNETIDDATA pNetIdData)
// {
//     WCHAR MsgText[MAX_PATH * 2];
//
//     if (LoadStringW(hDllInstance, 25, MsgText, ARRAYSIZE(MsgText)))
//         SetDlgItemText(hDlg, 1017, MsgText);
//
//     SendMessage(GetDlgItem(hDlg, 1002), EM_SETLIMITTEXT, MAX_HOSTNAME_LENGTH, 0);
//     SetDlgItemText(hDlg, 1002, pNetIdData->szHostName);
//     SetFullComputerName(hDlg, 1001, pNetIdData);
//
//     /* Display the workgroup or domain name */
//     switch (pNetIdData->JoinStatus)
//     {
//         case NetSetupDomainName:
//             SetDlgItemText(hDlg, 116, pNetIdData->JoinName);
//             SetRadioBtnState(hDlg, TRUE);
//             break;
//
//         case NetSetupWorkgroupName:
//             SetDlgItemText(hDlg, 1007, pNetIdData->JoinName);
//             SetRadioBtnState(hDlg, FALSE);
//             break;
//
//         default:
//             break;
//     }
// }
//
// static
// BOOL
// NetworkDlg_OnOK(
//     HWND hDlg,
//     PNETIDDATA pNetIdData)
// {
//     WCHAR szMsgText[MAX_PATH], szMsgTitle[MAX_PATH];
//     HKEY KeyHandle;
//     DWORD dwError;
//
//     if (pNetIdData->bHostNameChanged)
//     {
//         if (!IsValidComputerName(hDlg, 1002))
//         {
//             SetFocus(GetDlgItem(hDlg, 1002));
//             return FALSE;
//         }
//
//         GetWindowText(GetDlgItem(hDlg, 1002), pNetIdData->szHostName, ARRAYSIZE(pNetIdData->szHostName));
//
//         if (!SetComputerNameExW(ComputerNamePhysicalDnsHostname, pNetIdData->szHostName))
//         {
//             LoadStringW(hDllInstance, 4001, szMsgText, ARRAYSIZE(szMsgText));
//             MessageBoxW(hDlg, szMsgText, NULL, MB_OK | MB_ICONERROR);
//             return FALSE;
//         }
//
//         pNetIdData->bHostNameChanged = FALSE;
//     }
//
//     if (pNetIdData->bDomainNameChanged)
//     {
//         if (!SetComputerNameExW(ComputerNamePhysicalDnsDomain, pNetIdData->szDomainName))
//         {
//             /* FIXME: Show error message */
//             return FALSE;
//         }
//
//         pNetIdData->bDomainNameChanged = FALSE;
//     }
//
//     if (pNetIdData->bSyncDomainWithMembershipChanged)
//     {
//         dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                 L"SYSTEM\\CurrentControlSet\\Services\\TcpIp\\Parameters",
//                                 0,
//                                 KEY_SET_VALUE,
//                                 &KeyHandle);
//         if (dwError == ERROR_SUCCESS)
//         {
//             RegSetValueExW(KeyHandle,
//                            L"SyncDomainWithMembership",
//                            0,
//                            REG_DWORD,
//                            (LPBYTE)&pNetIdData->bSyncDomainWithMembership,
//                            sizeof(pNetIdData->bSyncDomainWithMembership));
//             RegCloseKey(KeyHandle);
//         }
//
//         pNetIdData->bSyncDomainWithMembershipChanged = FALSE;
//     }
//
//     LoadStringW(hDllInstance, 4000, szMsgTitle, ARRAYSIZE(szMsgTitle));
//     LoadStringW(hDllInstance, 24, szMsgText, ARRAYSIZE(szMsgText));
//     MessageBoxW(hDlg, szMsgText, szMsgTitle, MB_OK | MB_ICONINFORMATION);
//
//     return TRUE;
// }
//
// static
// INT_PTR __stdcall
// NetworkPropDlgProc(HWND hDlg, UINT Msg, WPARAM wParam, LPARAM lParam)
// {
//     PNETIDDATA pNetIdData;
//
//     pNetIdData = (PNETIDDATA)GetWindowLongPtr(hDlg, DWLP_USER);
//
//     switch (Msg)
//     {
//         case WM_INITDIALOG:
//             pNetIdData = (PNETIDDATA)lParam;
//             if (pNetIdData != NULL)
//             {
//                 SetWindowLongPtr(hDlg, DWLP_USER, (LONG_PTR)pNetIdData);
//                 NetworkDlg_OnInitDialog(hDlg, pNetIdData);
//                 EnableWindow(GetDlgItem(hDlg, IDOK), FALSE);
//             }
//             return TRUE;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case 1002:
//                     if (HIWORD(wParam) == EN_CHANGE)
//                     {
//                         UpdateFullComputerName(hDlg, 1001, pNetIdData);
//                         UpdateNetbiosName(hDlg, 1001, pNetIdData);
//                         pNetIdData->bHostNameChanged = TRUE;
//                         EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
//                     }
//                     break;
//
//                 case 1008: /* Domain radio button */
//                 case 1004: /* Workgroup radio button */
//                     if (SendDlgItemMessage(hDlg, 1008, BM_GETCHECK, 0, 0) == BST_CHECKED)
//                         SetRadioBtnState(hDlg, TRUE);
//                     else
//                         SetRadioBtnState(hDlg, FALSE);
//                     break;
//
//                 case 1003:
//                     if (DialogBoxParam(hDllInstance,
//                                        MAKEINTRESOURCE(IDD_PROPPAGEDNSANDNETBIOS),
//                                        hDlg,
//                                        DNSSuffixPropDlgProc,
//                                        (LPARAM)pNetIdData))
//                     {
//                         UpdateFullComputerName(hDlg, 1001, pNetIdData);
//                         EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
//                     }
//                     break;
//
//                 case IDOK:
//                     if (NetworkDlg_OnOK(hDlg, pNetIdData))
//                         EndDialog(hDlg, TRUE);
//                     break;
//
//                 case IDCANCEL:
//                     EndDialog(hDlg, FALSE);
//                     break;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// static
// VOID
// NetIDPage_OnInitDialog(
//     HWND hwndDlg,
//     PNETIDDATA pNetIdData)
// {
//     WCHAR ComputerDescription[MAX_COMPUTERDESCRIPTION_LENGTH + 1];
//     DWORD RegSize = sizeof(ComputerDescription);
//     HKEY KeyHandle;
//     WCHAR szBuffer[512];
//     LONG lError;
//
//     /* Display computer name and description */
//     SendDlgItemMessage(hwndDlg, IDC_COMPDESC, EM_SETLIMITTEXT, MAX_COMPUTERDESCRIPTION_LENGTH, 0);
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"SYSTEM\\CurrentControlSet\\Services\\LanmanServer\\Parameters",
//                            0,
//                            KEY_QUERY_VALUE,
//                            &KeyHandle);
//     if (lError == ERROR_SUCCESS)
//     {
//         lError = RegQueryValueExW(KeyHandle,
//                                   L"srvcomment",
//                                   0,
//                                   NULL,
//                                   (LPBYTE)ComputerDescription,
//                                   &RegSize);
//         if (lError == ERROR_SUCCESS)
//         {
//             ComputerDescription[RegSize / sizeof(WCHAR)] = UNICODE_NULL;
//             SetDlgItemText(hwndDlg, IDC_COMPDESC, ComputerDescription);
//         }
//
//         RegCloseKey(KeyHandle);
//     }
//
//     /* Set the workgroup or domain name */
//     LoadStringW(hDllInstance, (pNetIdData->JoinStatus == NetSetupDomainName)? 6 : 5, szBuffer, ARRAYSIZE(szBuffer));
//     SetDlgItemText(hwndDlg, IDC_WORKGROUPDOMAIN, szBuffer);
//     SetDlgItemText(hwndDlg, IDC_WORKGROUPDOMAIN_NAME, pNetIdData->JoinName);
//
//     /* Show the administrator note and disable controls when the user is not an administator */
//     if (!IsUserAdmin())
//     {
//         LoadStringW(hDllInstance, 1021, szBuffer, ARRAYSIZE(szBuffer));
//         SetDlgItemText(hwndDlg, IDC_MESSAGETEXT, szBuffer);
//
//         EnableWindow(GetDlgItem(hwndDlg, IDC_NETWORK_ID), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_NETWORK_PROPERTY), FALSE);
//     }
// }
//
// static
// LONG
// NetIDPage_OnApply(
//     HWND hwndDlg)
// {
//     WCHAR ComputerDescription[MAX_COMPUTERDESCRIPTION_LENGTH + 1];
//     WCHAR NewComputerDescription[MAX_COMPUTERDESCRIPTION_LENGTH + 1];
//     HKEY KeyHandle = NULL;
//     DWORD dwSize;
//     LONG lError;
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"SYSTEM\\CurrentControlSet\\Services\\LanmanServer\\Parameters",
//                            0,
//                            KEY_QUERY_VALUE | KEY_SET_VALUE,
//                            &KeyHandle);
//     if (lError != ERROR_SUCCESS)
//         return lError;
//
//     GetDlgItemTextW(hwndDlg,
//                     IDC_COMPDESC,
//                     NewComputerDescription,
//                     ARRAYSIZE(NewComputerDescription));
//     if (GetLastError() != ERROR_SUCCESS)
//     {
//         lError = GetLastError();
//         goto done;
//     }
//
//     dwSize = sizeof(ComputerDescription);
//     lError = RegQueryValueExW(KeyHandle,
//                               L"srvcomment",
//                               0,
//                               NULL,
//                               (PBYTE)ComputerDescription,
//                               &dwSize);
//     if (lError != ERROR_SUCCESS && lError != ERROR_FILE_NOT_FOUND)
//         goto done;
//
//     lError = ERROR_SUCCESS;
//     if (wcscmp(ComputerDescription, NewComputerDescription) != 0)
//     {
//         lError = RegSetValueExW(KeyHandle,
//                                 L"srvcomment",
//                                 0,
//                                 REG_SZ,
//                                 (PBYTE)NewComputerDescription,
//                                 (wcslen(NewComputerDescription) + 1) * sizeof(WCHAR));
//     }
//
// done:
//     if (KeyHandle != NULL)
//         RegCloseKey(KeyHandle);
//
//     return lError;
// }
//
// static INT_PTR __stdcall
// NetIDPageProc( HWND hwndDlg,
//                UINT uMsg,
//                WPARAM wParam,
//                LPARAM lParam)
// {
//     PNETIDDATA pNetIdData;
//
//     pNetIdData = (PNETIDDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             pNetIdData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(NETIDDATA));
//             if (pNetIdData != NULL)
//             {
//                 SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pNetIdData);
//                 GetComputerNames(pNetIdData);
//                 SetFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
//                 NetIDPage_OnInitDialog(hwndDlg, pNetIdData);
//                 pNetIdData->bEnable = TRUE;
//                 CheckForChangedNames(hwndDlg, pNetIdData);
//             }
//             return TRUE;
//
//         case WM_NOTIFY:
//             switch (((LPNMHDR)lParam)->code)
//             {
//                 case PSN_APPLY:
//                     NetIDPage_OnApply(hwndDlg);
//                     break;
//             }
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_COMPDESC:
//                     if (HIWORD(wParam) == EN_CHANGE && pNetIdData->bEnable == TRUE)
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//
//                 case IDC_NETWORK_PROPERTY:
//                     if (DialogBoxParam(hDllInstance,
//                                        MAKEINTRESOURCE(IDD_PROPPAGECOMPNAMECHENGE),
//                                        hwndDlg,
//                                        NetworkPropDlgProc,
//                                        (LPARAM)pNetIdData))
//                     {
//                         UpdateFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
//                         CheckForChangedNames(hwndDlg, pNetIdData);
//                     }
//                     break;
//             }
//             break;
//
//         case WM_DESTROY:
//             if (pNetIdData != NULL)
//             {
//                 if (pNetIdData->JoinName != NULL)
//                     NetApiBufferFree(pNetIdData->JoinName);
//                 HeapFree(GetProcessHeap(), 0, pNetIdData);
//                 pNetIdData = NULL;
//                 SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)NULL);
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// HPROPSHEETPAGE WINAPI
// CreateNetIDPropertyPage(VOID)
// {
//     PROPSHEETPAGE psp = {0};
//
//     psp.dwSize = sizeof(psp);
//     psp.dwFlags = PSP_DEFAULT;
//     psp.hInstance = hDllInstance;
//     psp.pszTemplate = MAKEINTRESOURCE(IDD_PROPPAGECOMPUTER);
//     psp.pfnDlgProc = NetIDPageProc;
//
//     return CreatePropertySheetPage(&psp);
// }
//
// BOOL WINAPI
// DllMain( HINSTANCE hinstDLL,
//          DWORD dwReason,
//          LPVOID lpvReserved)
// {
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             hDllInstance = hinstDLL;
//             DisableThreadLibraryCalls(hinstDLL);
//             break;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
