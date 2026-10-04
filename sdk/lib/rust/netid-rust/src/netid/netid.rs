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
    pub szComputerName: [u16; (MAX_COMPUTERNAME_LENGTH + 1) as usize],
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

// L"SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters"
static LANMANSERVER_PARAMS_KEY_BUF: [u16; 58] = [
    b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16,
    b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16,
    b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16,
    b'c' as u16, b'e' as u16, b's' as u16, b'\\' as u16, b'L' as u16, b'a' as u16, b'n' as u16, b'm' as u16, b'a' as u16, b'n' as u16,
    b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'e' as u16, b'r' as u16, b'\\' as u16, b'P' as u16, b'a' as u16, b'r' as u16,
    b'a' as u16, b'm' as u16, b'e' as u16, b't' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0,
];
// L"srvcomment"
static SRVCOMMENT_VALUE_BUF: [u16; 11] = [
    b's' as u16, b'r' as u16, b'v' as u16, b'c' as u16, b'o' as u16, b'm' as u16, b'm' as u16, b'e' as u16, b'n' as u16, b't' as u16,
    0,
];

static mut hDllInstance: HINSTANCE = core::ptr::null_mut();
/// The body of the C's variadic `FormatMessageBox`, with the `va_list` passed IN.
///
/// DEFINING a C-variadic is the one thing stable Rust cannot do (`c_variadic`), so the
/// variadic entry point — the only place `va_start` may legally appear — lives in
/// `netid_varargs_shim.c` and immediately hands control back here. All logic stays in Rust;
/// this is a shim for a language gap, not a translation reverted to C. See
/// `sdk/lib/rust/varargs-rust` for the pattern.
///
/// `ap` is a POINTER to the C frame's `va_list`, never the `va_list` by value: `va_list` is
/// `char*` on our three Windows targets but an array-of-struct on SysV x86-64, and the
/// by-pointer form is correct on both. It must not outlive the call from the C entry point.
/// Nothing here reads an argument off it — the list is forwarded verbatim to FormatMessageW,
/// exactly as the C does.
#[no_mangle]
pub unsafe extern "C" fn netid_FormatMessageBox_body(
    hDlg: HWND,
    uType: u32,
    dwMessage: u32,
    ap: *mut c_void,
) -> i32 {
    let mut szTitle: [u16; 256] = [0; 256];
    let mut szMessage: [u16; 256] = [0; 256];
    let mut szText: [u16; 512] = [0; 512];

    LoadStringW(
        hDllInstance,
        4,
        szTitle.as_mut_ptr(),
        (ARRAYSIZE!(szTitle)) as i32
    );

    LoadStringW(
        hDllInstance,
        dwMessage,
        szMessage.as_mut_ptr(),
        (ARRAYSIZE!(szMessage)) as i32
    );

    FormatMessageW(
        FORMAT_MESSAGE_FROM_STRING,
        (szMessage.as_ptr()).cast(),
        0,
        0,
        szText.as_mut_ptr(),
        ARRAYSIZE!(szText),
        ap
    );

    MessageBoxW(hDlg, szText.as_ptr(), szTitle.as_ptr(), uType)
}

extern "C" {
    /// The variadic entry point in netid_varargs_shim.c. Callers use this exactly as the C
    /// called the original `FormatMessageBox`.
    pub fn FormatMessageBox(hDlg: HWND, uType: u32, dwMessage: u32, ...) -> i32;
}
unsafe extern "C" fn GetComputerNames(
    pNetIdData: *mut NETIDDATA,
) -> i32 {
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut dwSize: u32;
    let mut dwError: u32;

    dwError = (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            win32macros_rust::wide_bytes!(b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0T\0c\0p\0I\0p\0\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0") as *const u16,
                            0,
                            KEY_QUERY_VALUE,
                            &mut KeyHandle)) as u32;
    if dwError == ERROR_SUCCESS
    {
        dwSize = core::mem::size_of_val(&(*pNetIdData).szHostName) as u32;
        RegQueryValueExW(KeyHandle,
                         win32macros_rust::wide_bytes!(b"N\0V\0 \0H\0o\0s\0t\0N\0a\0m\0e\0\0\0") as *const u16,
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         (*pNetIdData).szHostName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).szOldHostName) as u32;
        RegQueryValueExW(KeyHandle,
                         win32macros_rust::wide_bytes!(b"H\0o\0s\0t\0N\0a\0m\0e\0\0\0") as *const u16,
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         (*pNetIdData).szOldHostName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).szDomainName) as u32;
        RegQueryValueExW(KeyHandle,
                         win32macros_rust::wide_bytes!(b"N\0V\0 \0D\0o\0m\0a\0i\0n\0\0\0") as *const u16,
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         (*pNetIdData).szDomainName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).szOldDomainName) as u32;
        RegQueryValueExW(KeyHandle,
                         win32macros_rust::wide_bytes!(b"D\0o\0m\0a\0i\0n\0\0\0") as *const u16,
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         (*pNetIdData).szOldDomainName.as_mut_ptr() as LPBYTE,
                         &mut dwSize);

        dwSize = core::mem::size_of_val(&(*pNetIdData).bSyncDomainWithMembership) as u32;
        if RegQueryValueExW(KeyHandle,
                             win32macros_rust::wide_bytes!(b"S\0y\0n\0c\0D\0o\0m\0a\0i\0n\0W\0i\0t\0h\0M\0e\0m\0b\0e\0r\0s\0h\0i\0p\0\0\0") as *const u16,
                             core::ptr::null_mut(),
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!((*pNetIdData).bSyncDomainWithMembership) as LPBYTE,
                             &mut dwSize) != 0
        {
            (*pNetIdData).bSyncDomainWithMembership = (TRUE) as i32;
        }

        RegCloseKey(KeyHandle);
    }

    dwError = (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            win32macros_rust::wide_bytes!(b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0C\0o\0m\0p\0u\0t\0e\0r\0N\0a\0m\0e\0\\\0C\0o\0m\0p\0u\0t\0e\0r\0N\0a\0m\0e\0\0\0") as *const u16,
                            0,
                            KEY_QUERY_VALUE,
                            &mut KeyHandle)) as u32;
    if dwError == ERROR_SUCCESS
    {
        dwSize = core::mem::size_of_val(&(*pNetIdData).szComputerName) as u32;
        RegQueryValueExW(KeyHandle,
                         win32macros_rust::wide_bytes!(b"C\0o\0m\0p\0u\0t\0e\0r\0N\0a\0m\0e\0\0\0") as *const u16,
                         core::ptr::null_mut(),
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

    (TRUE) as i32
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
        return (FALSE) as i32;
    }

    if CheckTokenMembership(core::ptr::null_mut(), pAdminsSid, &mut bIsAdmin) == 0 {
        bIsAdmin = (FALSE) as i32;
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

    if GetDlgItemTextW(hDlg, (uId) as i32, szDomainName.as_mut_ptr(), (ARRAYSIZE!(szDomainName)) as i32) == 0 {
        return (TRUE) as u8;
    }

    dwError = DnsValidateName_W(szDomainName.as_ptr(), DnsNameDomain);
    if dwError != ERROR_SUCCESS {
        match dwError {
            DNS_ERROR_NON_RFC_NAME => {
                if FormatMessageBox(hDlg, MB_YESNO | MB_ICONWARNING, 7, szDomainName.as_ptr()) == (IDYES) as i32 {
                    return (TRUE) as u8;
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

        return (FALSE) as u8;
    }

    return (TRUE) as u8;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn DNSSuffixPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg
    {
        WM_INITDIALOG => {
            pNetIdData = lParam as *mut NETIDDATA;
            if pNetIdData != core::ptr::null_mut()
            {
                SetWindowLongPtr(hDlg, DWLP_USER, (pNetIdData as isize) as i32);

                SetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_ptr());
                SendDlgItemMessage(hDlg, 115, BM_SETCHECK, (*pNetIdData).bSyncDomainWithMembership as WPARAM, 0);
                SetDlgItemTextW(hDlg, 1013, (*pNetIdData).szComputerName.as_ptr());
                EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (FALSE) as i32);
            }
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as u32
            {
                115 => {
                    if HIWORD!(wParam) == (BN_CLICKED) as u16
                    {
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                        (*pNetIdData).bSyncDomainWithMembershipChanged = (TRUE) as i32;
                    }
                }

                1011 => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16
                    {
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                        (*pNetIdData).bDomainNameChanged = (TRUE) as i32;
                    }
                }

                IDOK => {
                    if IsValidDomainName(hDlg, 1011) == 0
                    {
                        SetFocus(GetDlgItem(hDlg, 1011));
                        return FALSE as INT_PTR;
                    }

                    if (*pNetIdData).bDomainNameChanged != 0
                    {
                        GetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_mut_ptr(), (ARRAYSIZE!((*pNetIdData).szDomainName)) as i32);
                    }

                    if (*pNetIdData).bSyncDomainWithMembershipChanged != 0
                    {
                        if SendDlgItemMessage(hDlg, 115, BM_GETCHECK, 0, 0) == (BST_CHECKED) as i32
                        {
                            (*pNetIdData).bSyncDomainWithMembership = (TRUE) as i32;
                        }
                        else
                        {
                            (*pNetIdData).bSyncDomainWithMembership = (FALSE) as i32;
                        }
                    }

                    EndDialog(hDlg, (TRUE) as i32);
                }

                IDCANCEL => {
                    (*pNetIdData).bDomainNameChanged = (FALSE) as i32;
                    (*pNetIdData).bSyncDomainWithMembershipChanged = (FALSE) as i32;
                    EndDialog(hDlg, (FALSE) as i32);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn DNSSuffixPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg
    {
        WM_INITDIALOG => {
            pNetIdData = lParam as *mut NETIDDATA;
            if pNetIdData != core::ptr::null_mut()
            {
                SetWindowLongPtr(hDlg, DWLP_USER, (pNetIdData as isize) as i64);

                SetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_ptr());
                SendDlgItemMessage(hDlg, 115, BM_SETCHECK, (*pNetIdData).bSyncDomainWithMembership as WPARAM, 0);
                SetDlgItemTextW(hDlg, 1013, (*pNetIdData).szComputerName.as_ptr());
                EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (FALSE) as i32);
            }
            return (TRUE) as i64;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as u32
            {
                115 => {
                    if HIWORD!(wParam) == (BN_CLICKED) as u16
                    {
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                        (*pNetIdData).bSyncDomainWithMembershipChanged = (TRUE) as i32;
                    }
                }

                1011 => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16
                    {
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                        (*pNetIdData).bDomainNameChanged = (TRUE) as i32;
                    }
                }

                IDOK => {
                    if IsValidDomainName(hDlg, 1011) == 0
                    {
                        SetFocus(GetDlgItem(hDlg, 1011));
                        return (FALSE) as i64;
                    }

                    if (*pNetIdData).bDomainNameChanged != 0
                    {
                        GetDlgItemTextW(hDlg, 1011, (*pNetIdData).szDomainName.as_mut_ptr(), (ARRAYSIZE!((*pNetIdData).szDomainName)) as i32);
                    }

                    if (*pNetIdData).bSyncDomainWithMembershipChanged != 0
                    {
                        if SendDlgItemMessage(hDlg, 115, BM_GETCHECK, 0, 0) == (BST_CHECKED) as i64
                        {
                            (*pNetIdData).bSyncDomainWithMembership = (TRUE) as i32;
                        }
                        else
                        {
                            (*pNetIdData).bSyncDomainWithMembership = (FALSE) as i32;
                        }
                    }

                    EndDialog(hDlg, (TRUE) as i64);
                }

                IDCANCEL => {
                    (*pNetIdData).bDomainNameChanged = (FALSE) as i32;
                    (*pNetIdData).bSyncDomainWithMembershipChanged = (FALSE) as i32;
                    EndDialog(hDlg, (FALSE) as i64);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as i64;
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

    GetWindowText(GetDlgItem(hDlg, (uId) as i32), szHostName.as_mut_ptr(), (ARRAYSIZE!(szHostName)) as i32);

    dwError = DnsValidateName_W(szHostName.as_ptr(), DnsNameHostnameLabel);
    if dwError != ERROR_SUCCESS {
        match dwError {
            DNS_ERROR_NON_RFC_NAME => {
                if FormatMessageBox(hDlg, MB_YESNO | MB_ICONWARNING, 10, szHostName.as_ptr()) == (IDYES) as i32 {
                    return (TRUE) as i32;
                }
            }

            ERROR_INVALID_NAME => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 11);
                return (FALSE) as i32;
            }

            DNS_ERROR_NUMERIC_NAME => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1029, szHostName.as_ptr());
            }

            DNS_ERROR_INVALID_NAME_CHAR => {
                FormatMessageBox(hDlg, MB_OK | MB_ICONERROR, 1030, szHostName.as_ptr());
            }

            _ => {}
        }

        return (FALSE) as i32;
    }

    return (TRUE) as i32;
}
unsafe extern "C" fn SetFullComputerName(
    hDlg: HWND,
    uId: u32,
    pNetIdData: *mut _NETIDDATA,
) {
    let mut szFullComputerName: [u16; 512] = [0; 512];

    // C: wsprintf(szFullComputerName, L"%s.%s", ...) — the L is the wide-literal MARKER, not a
    // character. The translation encoded it as content ("L%s.%s"), which would have prefixed
    // every full computer name with a stray "L".
    wsprintf(
        szFullComputerName.as_mut_ptr(),
        w!("%s.%s"),
        (*pNetIdData).szHostName.as_ptr(),
        (*pNetIdData).szDomainName.as_ptr(),
    );
    SetDlgItemText(hDlg, (uId) as i32, szFullComputerName.as_ptr());
}
unsafe extern "C" fn UpdateFullComputerName(
    hDlg: HWND,
    uId: u32,
    pNetIdData: *mut NETIDDATA,
) {
    let mut szFullComputerName: [u16; 512] = [0; 512];
    let mut szHostName: [u16; 256] = [0; 256];

    GetWindowText(GetDlgItem(hDlg, 1002), szHostName.as_mut_ptr(), (ARRAYSIZE!(szHostName)) as i32);

    // Same C literal as above, mistranslated the other way: raw ASCII bytes of `L"%s.%s"`
    // handed to a wide-string callee.
    wsprintf(
        szFullComputerName.as_mut_ptr(),
        w!("%s.%s"),
        szHostName.as_ptr(),
        (*pNetIdData).szDomainName.as_ptr(),
    );
    SetDlgItemText(hDlg, (uId) as i32, szFullComputerName.as_ptr());
}
unsafe extern "C" fn UpdateNetbiosName(
    hDlg: HWND,
    uId: u32,
    pNetIdData: *mut NETIDDATA,
) {
    let mut szHostName: [u16; 256] = [0; 256];
    let mut dwSize: u32;

    GetWindowText(GetDlgItem(hDlg, 1002), szHostName.as_mut_ptr(), (ARRAYSIZE!(szHostName)) as i32);

    dwSize = ARRAYSIZE!((*pNetIdData).szComputerName);
    DnsHostnameToComputerNameW(szHostName.as_ptr(),
                           (*pNetIdData).szComputerName.as_mut_ptr(),
                           &mut dwSize);
}
unsafe extern "C" fn CheckForChangedNames(
    hwndDlg: HWND,
    pNetIdData: *mut NETIDDATA,
) {
    let mut nShow: i32 = (SW_HIDE) as i32;

    if (_wcsicmp((*pNetIdData).szOldHostName.as_ptr(), (*pNetIdData).szHostName.as_ptr()) != 0) ||
       (_wcsicmp((*pNetIdData).szOldDomainName.as_ptr(), (*pNetIdData).szDomainName.as_ptr()) != 0) {
        nShow = (SW_SHOW) as i32;
    }

    ShowWindow(GetDlgItem(hwndDlg, (IDC_RESTARTICON) as i32), nShow);
    ShowWindow(GetDlgItem(hwndDlg, (IDC_RESTARTTEXT) as i32), nShow);
}
unsafe extern "C" fn NetworkDlg_OnInitDialog(
    hDlg: HWND,
    pNetIdData: *mut _NETIDDATA,
) {
    let mut MsgText: [u16; (MAX_PATH * 2) as usize] = [0; (MAX_PATH * 2) as usize];

    if LoadStringW(hDllInstance, 25, MsgText.as_mut_ptr(), (ARRAYSIZE!(MsgText)) as i32) != 0 {
        SetDlgItemText(hDlg, 1017, MsgText.as_ptr());
    }

    SendMessage(GetDlgItem(hDlg, 1002), EM_SETLIMITTEXT, MAX_HOSTNAME_LENGTH as WPARAM, 0);
    SetDlgItemText(hDlg, 1002, (*pNetIdData).szHostName.as_ptr());
    SetFullComputerName(hDlg, 1001, pNetIdData);

    match (*pNetIdData).JoinStatus {
        NetSetupDomainName => {
            SetDlgItemText(hDlg, 116, (*pNetIdData).JoinName);
            SetRadioBtnState(hDlg, (TRUE) as i32);
        }

        NetSetupWorkgroupName => {
            SetDlgItemText(hDlg, 1007, (*pNetIdData).JoinName);
            SetRadioBtnState(hDlg, (FALSE) as i32);
        }

        _ => {}
    }
}
unsafe extern "C" fn NetworkDlg_OnOK(
    hDlg: HWND,
    pNetIdData: *mut NETIDDATA,
) -> i32 {
    let mut szMsgText: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szMsgTitle: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut dwError: u32;

    if (*pNetIdData).bHostNameChanged != 0 {
        if IsValidComputerName(hDlg, 1002) == 0 {
            SetFocus(GetDlgItem(hDlg, 1002));
            return (FALSE) as i32;
        }

        GetWindowText(GetDlgItem(hDlg, 1002), (*pNetIdData).szHostName.as_mut_ptr(), (ARRAYSIZE!((*pNetIdData).szHostName)) as i32);

        if SetComputerNameExW(ComputerNamePhysicalDnsHostname, (*pNetIdData).szHostName.as_mut_ptr()) == 0 {
            LoadStringW(hDllInstance, 4001, szMsgText.as_mut_ptr(), (ARRAYSIZE!(szMsgText)) as i32);
            MessageBoxW(hDlg, szMsgText.as_mut_ptr(), core::ptr::null_mut(), MB_OK | MB_ICONERROR);
            return (FALSE) as i32;
        }

        (*pNetIdData).bHostNameChanged = (FALSE) as i32;
    }

    if (*pNetIdData).bDomainNameChanged != 0 {
        if SetComputerNameExW(ComputerNamePhysicalDnsDomain, (*pNetIdData).szDomainName.as_mut_ptr()) == 0 {
            return (FALSE) as i32;
        }

        (*pNetIdData).bDomainNameChanged = (FALSE) as i32;
    }

    if (*pNetIdData).bSyncDomainWithMembershipChanged != 0 {
        dwError = (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                                win32macros_rust::wide_bytes!(b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0T\0c\0p\0I\0p\0\\\0P\0a\0r\0a\0m\0e\0t\0e\0r\0s\0\0\0") as *const u16,
                                0,
                                KEY_SET_VALUE,
                                &mut KeyHandle)) as u32;
        if dwError == ERROR_SUCCESS {
            RegSetValueExW(KeyHandle,
                           win32macros_rust::wide_bytes!(b"S\0y\0n\0c\0D\0o\0m\0a\0i\0n\0W\0i\0t\0h\0M\0e\0m\0b\0e\0r\0s\0h\0i\0p\0\0\0") as *const u16,
                           0,
                           REG_DWORD,
                           core::ptr::addr_of_mut!((*pNetIdData).bSyncDomainWithMembership) as LPBYTE,
                           core::mem::size_of_val(&(*pNetIdData).bSyncDomainWithMembership) as u32);
            RegCloseKey(KeyHandle);
        }

        (*pNetIdData).bSyncDomainWithMembershipChanged = (FALSE) as i32;
    }

    LoadStringW(hDllInstance, 4000, szMsgTitle.as_mut_ptr(), (ARRAYSIZE!(szMsgTitle)) as i32);
    LoadStringW(hDllInstance, 24, szMsgText.as_mut_ptr(), (ARRAYSIZE!(szMsgText)) as i32);
    MessageBoxW(hDlg, szMsgText.as_mut_ptr(), szMsgTitle.as_mut_ptr(), MB_OK | MB_ICONINFORMATION);

    return (TRUE) as i32;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn NetworkPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg {
        WM_INITDIALOG => {
            let pNetIdData = lParam as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hDlg, DWLP_USER, (pNetIdData as isize) as i32);
                NetworkDlg_OnInitDialog(hDlg, pNetIdData);
                EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (FALSE) as i32);
            }
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as u32 {
                1002 => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16 {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        UpdateNetbiosName(hDlg, 1001, pNetIdData);
                        (*pNetIdData).bHostNameChanged = (TRUE) as i32;
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                    }
                }

                1008 | 1004 => {
                    if SendDlgItemMessage(hDlg, 1008, BM_GETCHECK, 0, 0) == (BST_CHECKED) as i32 {
                        SetRadioBtnState(hDlg, (TRUE) as i32);
                    } else {
                        SetRadioBtnState(hDlg, (FALSE) as i32);
                    }
                }

                1003 => {
                    if DialogBoxParam(hDllInstance,
                                   MAKEINTRESOURCE!(IDD_PROPPAGEDNSANDNETBIOS),
                                   hDlg,
                                   Some(DNSSuffixPropDlgProc),
                                   pNetIdData as LPARAM) != 0 {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                    }
                }

                IDOK => {
                    if NetworkDlg_OnOK(hDlg, pNetIdData) != 0 {
                        EndDialog(hDlg, (TRUE) as i32);
                    }
                }

                IDCANCEL => {
                    EndDialog(hDlg, (FALSE) as i32);
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as i32
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn NetworkPropDlgProc(hDlg: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let pNetIdData: *mut NETIDDATA;

    pNetIdData = GetWindowLongPtr(hDlg, DWLP_USER) as *mut NETIDDATA;

    match Msg {
        WM_INITDIALOG => {
            let pNetIdData = lParam as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hDlg, DWLP_USER, (pNetIdData as isize) as i64);
                NetworkDlg_OnInitDialog(hDlg, pNetIdData);
                EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (FALSE) as i32);
            }
            return (TRUE) as i64;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as u32 {
                1002 => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16 {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        UpdateNetbiosName(hDlg, 1001, pNetIdData);
                        (*pNetIdData).bHostNameChanged = (TRUE) as i32;
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                    }
                }

                1008 | 1004 => {
                    if SendDlgItemMessage(hDlg, 1008, BM_GETCHECK, 0, 0) == (BST_CHECKED) as i64 {
                        SetRadioBtnState(hDlg, (TRUE) as i32);
                    } else {
                        SetRadioBtnState(hDlg, (FALSE) as i32);
                    }
                }

                1003 => {
                    if DialogBoxParam(hDllInstance,
                                   MAKEINTRESOURCE!(IDD_PROPPAGEDNSANDNETBIOS),
                                   hDlg,
                                   Some(DNSSuffixPropDlgProc),
                                   pNetIdData as LPARAM) != 0 {
                        UpdateFullComputerName(hDlg, 1001, pNetIdData);
                        EnableWindow(GetDlgItem(hDlg, (IDOK) as i32), (TRUE) as i32);
                    }
                }

                IDOK => {
                    if NetworkDlg_OnOK(hDlg, pNetIdData) != 0 {
                        EndDialog(hDlg, (TRUE) as i64);
                    }
                }

                IDCANCEL => {
                    EndDialog(hDlg, (FALSE) as i64);
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as i64
}
unsafe extern "C" fn NetIDPage_OnInitDialog(
    hwndDlg: HWND,
    pNetIdData: *mut NETIDDATA) {
    let mut ComputerDescription: [u16; MAX_COMPUTERDESCRIPTION_LENGTH + 1] = [0; MAX_COMPUTERDESCRIPTION_LENGTH + 1];
    let mut RegSize: u32 = (core::mem::size_of_val(&ComputerDescription)) as u32;
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut szBuffer: [u16; 512] = [0; 512];
    let mut lError: i32;

    // WPARAM, not a baked u64: this call sits in SHARED code (no cfg gate), so a 64-bit width
    // here is simply wrong on i386. Name the alias and both widths follow.
    SendDlgItemMessage(hwndDlg, (IDC_COMPDESC) as i32, EM_SETLIMITTEXT, MAX_COMPUTERDESCRIPTION_LENGTH as WPARAM, 0);

    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                           LANMANSERVER_PARAMS_KEY_BUF.as_ptr() as *const u16,
                           0,
                           KEY_QUERY_VALUE,
                           &mut KeyHandle);
    if lError == (ERROR_SUCCESS) as i32
    {
        lError = RegQueryValueExW(KeyHandle,
                                  SRVCOMMENT_VALUE_BUF.as_ptr() as *const u16,
                                  core::ptr::null_mut(),
                                  core::ptr::null_mut(),
                                  ComputerDescription.as_mut_ptr() as LPBYTE,
                                  &mut RegSize);
        if lError == (ERROR_SUCCESS) as i32
        {
            ComputerDescription[(RegSize / core::mem::size_of::<u16>() as u32) as usize] = UNICODE_NULL;
            SetDlgItemText(hwndDlg, (IDC_COMPDESC) as i32, ComputerDescription.as_ptr());
        }

        RegCloseKey(KeyHandle);
    }

    LoadStringW(hDllInstance, if (*pNetIdData).JoinStatus == NetSetupDomainName { 6 } else { 5 }, szBuffer.as_mut_ptr(), (ARRAYSIZE!(szBuffer)) as i32);
    SetDlgItemText(hwndDlg, (IDC_WORKGROUPDOMAIN) as i32, szBuffer.as_ptr());
    SetDlgItemText(hwndDlg, (IDC_WORKGROUPDOMAIN_NAME) as i32, (*pNetIdData).JoinName);

    if IsUserAdmin() == 0
    {
        LoadStringW(hDllInstance, 1021, szBuffer.as_mut_ptr(), (ARRAYSIZE!(szBuffer)) as i32);
        SetDlgItemText(hwndDlg, (IDC_MESSAGETEXT) as i32, szBuffer.as_ptr());

        EnableWindow(GetDlgItem(hwndDlg, (IDC_NETWORK_ID) as i32), (FALSE) as i32);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_NETWORK_PROPERTY) as i32), (FALSE) as i32);
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
                           LANMANSERVER_PARAMS_KEY_BUF.as_ptr() as *const u16,
                           0,
                           KEY_QUERY_VALUE | KEY_SET_VALUE,
                           &mut KeyHandle);
    if lError != (ERROR_SUCCESS) as i32 {
        return lError;
    }

    GetDlgItemTextW(hwndDlg,
                    (IDC_COMPDESC) as i32,
                    NewComputerDescription.as_mut_ptr(),
                    (ARRAYSIZE!(NewComputerDescription)) as i32);
    if GetLastError() != ERROR_SUCCESS {
        lError = (GetLastError()) as i32;
        'done: loop {
            if KeyHandle != core::ptr::null_mut() {
                RegCloseKey(KeyHandle);
            }
            return lError;
        }
    }

    dwSize = core::mem::size_of_val(&ComputerDescription) as u32;
    lError = RegQueryValueExW(KeyHandle,
                              SRVCOMMENT_VALUE_BUF.as_ptr() as *const u16,
                              core::ptr::null_mut(),
                              core::ptr::null_mut(),
                              ComputerDescription.as_mut_ptr() as PBYTE,
                              &mut dwSize);
    if lError != (ERROR_SUCCESS) as i32 && lError != (ERROR_FILE_NOT_FOUND) as i32 {
        'done: loop {
            if KeyHandle != core::ptr::null_mut() {
                RegCloseKey(KeyHandle);
            }
            return lError;
        }
    }

    lError = (ERROR_SUCCESS) as i32;
    if wcscmp(ComputerDescription.as_ptr(), NewComputerDescription.as_ptr()) != 0 {
        lError = RegSetValueExW(KeyHandle,
                                SRVCOMMENT_VALUE_BUF.as_ptr() as *const u16,
                                0,
                                REG_SZ,
                                NewComputerDescription.as_ptr() as PBYTE,
                                ((wcslen(NewComputerDescription.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
    }

    if KeyHandle != core::ptr::null_mut() {
        RegCloseKey(KeyHandle);
    }

    lError
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn NetIDPageProc(
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
                (core::mem::size_of::<NETIDDATA>()) as u32,
            ) as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hwndDlg, DWLP_USER, (pNetIdData as isize) as i32);
                GetComputerNames(pNetIdData);
                SetFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
                NetIDPage_OnInitDialog(hwndDlg, pNetIdData);
                (*pNetIdData).bEnable = (TRUE) as i32;
                CheckForChangedNames(hwndDlg, pNetIdData);
            }
            return TRUE as INT_PTR;
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code as i32 {
                PSN_APPLY => {
                    NetIDPage_OnApply(hwndDlg);
                }
                _ => {}
            }
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as u32 {
                IDC_COMPDESC => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16 && (*pNetIdData).bEnable == (TRUE) as i32 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_NETWORK_PROPERTY => {
                    if DialogBoxParam(
                        hDllInstance,
                        MAKEINTRESOURCE!(IDD_PROPPAGECOMPNAMECHENGE),
                        hwndDlg,
                        Some(NetworkPropDlgProc),
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
                    NetApiBufferFree(((*pNetIdData).JoinName).cast());
                }
                HeapFree(GetProcessHeap(), 0, pNetIdData as LPVOID);
                pNetIdData = core::ptr::null_mut();
                SetWindowLongPtr(hwndDlg, DWLP_USER, (0 as isize) as i32);
            }
        }

        _ => {}
    }

    (FALSE) as i32
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn NetIDPageProc(
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
                (core::mem::size_of::<NETIDDATA>()) as u64,
            ) as *mut NETIDDATA;
            if !pNetIdData.is_null() {
                SetWindowLongPtr(hwndDlg, DWLP_USER, (pNetIdData as isize) as i64);
                GetComputerNames(pNetIdData);
                SetFullComputerName(hwndDlg, IDC_COMPUTERNAME, pNetIdData);
                NetIDPage_OnInitDialog(hwndDlg, pNetIdData);
                (*pNetIdData).bEnable = (TRUE) as i32;
                CheckForChangedNames(hwndDlg, pNetIdData);
            }
            return (TRUE) as i64;
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code as i32 {
                PSN_APPLY => {
                    NetIDPage_OnApply(hwndDlg);
                }
                _ => {}
            }
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as u32 {
                IDC_COMPDESC => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16 && (*pNetIdData).bEnable == (TRUE) as i32 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_NETWORK_PROPERTY => {
                    if DialogBoxParam(
                        hDllInstance,
                        MAKEINTRESOURCE!(IDD_PROPPAGECOMPNAMECHENGE),
                        hwndDlg,
                        Some(NetworkPropDlgProc),
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
                    NetApiBufferFree(((*pNetIdData).JoinName).cast());
                }
                HeapFree(GetProcessHeap(), 0, pNetIdData as LPVOID);
                pNetIdData = core::ptr::null_mut();
                SetWindowLongPtr(hwndDlg, DWLP_USER, (0 as isize) as i64);
            }
        }

        _ => {}
    }

    (FALSE) as i64
}

#[no_mangle]
pub unsafe extern "system" fn CreateNetIDPropertyPage() -> HPROPSHEETPAGE {
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();

    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
    psp.dwFlags = PSP_DEFAULT;
    psp.hInstance = hDllInstance;
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PROPPAGECOMPUTER);
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

    (TRUE) as i32
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
//     LoadStringW(hDllInstance, 4, szTitle, ARRAYSIZE!(szTitle));
//
//     LoadStringW(hDllInstance, dwMessage, szMessage, ARRAYSIZE!(szMessage));
//
//     va_start(args, dwMessage);
//     FormatMessageW(FORMAT_MESSAGE_FROM_STRING,
//                    szMessage,
//                    0,
//                    0,
//                    szText,
//                    ARRAYSIZE!(szText),
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
//     if (GetDlgItemTextW(hDlg, uId, szDomainName, ARRAYSIZE!(szDomainName)) == 0)
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
//             switch (LOWORD!(wParam))
//             {
//                 case 115:
//                     if (HIWORD!(wParam) == BN_CLICKED)
//                     {
//                         EnableWindow(GetDlgItem(hDlg, IDOK), TRUE);
//                         pNetIdData->bSyncDomainWithMembershipChanged = TRUE;
//                     }
//                     break;
//
//                 case 1011:
//                     if (HIWORD!(wParam) == EN_CHANGE)
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
//                         GetDlgItemTextW(hDlg, 1011, pNetIdData->szDomainName, ARRAYSIZE!(pNetIdData->szDomainName));
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
//     GetWindowText(GetDlgItem(hDlg, uId), szHostName, ARRAYSIZE!(szHostName));
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
//     GetWindowText(GetDlgItem(hDlg, 1002), szHostName, ARRAYSIZE!(szHostName));
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
//     GetWindowText(GetDlgItem(hDlg, 1002), szHostName, ARRAYSIZE!(szHostName));
//
//     dwSize = ARRAYSIZE!(pNetIdData->szComputerName);
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
//     if (LoadStringW(hDllInstance, 25, MsgText, ARRAYSIZE!(MsgText)))
//         SetDlgItemText(hDlg, 1017, MsgText);
//
//     SendMessage(GetDlgItem(hDlg, 1002), EM_SETLIMITTEXT, MAX_HOSTNAME_LENGTH as WPARAM, 0);
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
//         GetWindowText(GetDlgItem(hDlg, 1002), pNetIdData->szHostName, ARRAYSIZE!(pNetIdData->szHostName));
//
//         if (!SetComputerNameExW(ComputerNamePhysicalDnsHostname, pNetIdData->szHostName))
//         {
//             LoadStringW(hDllInstance, 4001, szMsgText, ARRAYSIZE!(szMsgText));
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
//     LoadStringW(hDllInstance, 4000, szMsgTitle, ARRAYSIZE!(szMsgTitle));
//     LoadStringW(hDllInstance, 24, szMsgText, ARRAYSIZE!(szMsgText));
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
//             switch (LOWORD!(wParam))
//             {
//                 case 1002:
//                     if (HIWORD!(wParam) == EN_CHANGE)
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
//                                        MAKEINTRESOURCE!(IDD_PROPPAGEDNSANDNETBIOS),
//                                        hDlg,
//                                        Some(DNSSuffixPropDlgProc),
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
//     LoadStringW(hDllInstance, (pNetIdData->JoinStatus == NetSetupDomainName)? 6 : 5, szBuffer, ARRAYSIZE!(szBuffer));
//     SetDlgItemText(hwndDlg, IDC_WORKGROUPDOMAIN, szBuffer);
//     SetDlgItemText(hwndDlg, IDC_WORKGROUPDOMAIN_NAME, pNetIdData->JoinName);
//
//     /* Show the administrator note and disable controls when the user is not an administator */
//     if (!IsUserAdmin())
//     {
//         LoadStringW(hDllInstance, 1021, szBuffer, ARRAYSIZE!(szBuffer));
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
//                     ARRAYSIZE!(NewComputerDescription));
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
//             switch (LOWORD!(wParam))
//             {
//                 case IDC_COMPDESC:
//                     if (HIWORD!(wParam) == EN_CHANGE && pNetIdData->bEnable == TRUE)
//                         PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
//                     break;
//
//                 case IDC_NETWORK_PROPERTY:
//                     if (DialogBoxParam(hDllInstance,
//                                        MAKEINTRESOURCE!(IDD_PROPPAGECOMPNAMECHENGE),
//                                        hwndDlg,
//                                        Some(NetworkPropDlgProc),
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
//     psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PROPPAGECOMPUTER);
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
