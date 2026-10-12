use crate::newdev::newdev::CheckBestDriver;
use crate::newdev::newdev::InstallCurrentDriver;
use crate::newdev::newdev::PrepareFoldersToScan;
use crate::newdev::newdev::ScanFoldersForDriver;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_491507152B61: [u16; 13] = [
    0x004Du16, 0x0053, 0x0020, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0044,
    0x006C, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_6693190CF469: [u16; 21] = [
    0x0049u16, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0020, 0x0053, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_D3C8E20D82DB: [u16; 39] = [
    0x007Bu16, 0x0034, 0x0044, 0x0033, 0x0036, 0x0045, 0x0039, 0x0037, 0x0045, 0x002D,
    0x0045, 0x0033, 0x0032, 0x0035, 0x002D, 0x0031, 0x0031, 0x0043, 0x0045, 0x002D,
    0x0042, 0x0046, 0x0043, 0x0031, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032,
    0x0042, 0x0045, 0x0031, 0x0030, 0x0033, 0x0031, 0x0038, 0x007D, 0x0000,
];
pub static RUSTCONV_WSTR_F3CDEC9800CB: [u16; 8] = [0x0055u16, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hThread: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut CreateTitleFont_unused_guard: () = ();

unsafe extern "C" fn CenterWindow(hWnd: HWND) {
    let mut hWndParent: HWND = core::ptr::null_mut();
    let mut rcParent: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcWindow: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    hWndParent = GetParent(hWnd);
    if hWndParent.is_null() {
        hWndParent = GetDesktopWindow();
    }

    GetWindowRect(hWndParent, core::ptr::addr_of_mut!(rcParent));
    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rcWindow));

    /* Check if the child window fits inside the parent window */
    if rcWindow.left < rcParent.left || rcWindow.top < rcParent.top ||
        rcWindow.right > rcParent.right || rcWindow.bottom > rcParent.bottom
    {
        return;
    }

    SetWindowPos(
        hWnd,
        HWND_TOP,
        ((rcParent.right - rcParent.left) - (rcWindow.right - rcWindow.left)) / 2,
        ((rcParent.bottom - rcParent.top) - (rcWindow.bottom - rcWindow.top)) / 2,
        0,
        0,
        SWP_NOSIZE,
    );
}

unsafe extern "C" fn SetFailedInstall(
    DeviceInfoSet: HDEVINFO,
    DevInfoData: PSP_DEVINFO_DATA,
    Set: BOOLEAN,
) -> BOOL {
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwFlags: DWORD = 0;

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    if SetupDiGetDeviceRegistryProperty(
        DeviceInfoSet,
        DevInfoData,
        SPDRP_CONFIGFLAGS,
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(dwFlags) as PBYTE,
        dwSize,
        core::ptr::addr_of_mut!(dwSize),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if Set != 0 {
        dwFlags |= CONFIGFLAG_FAILEDINSTALL;
    } else {
        dwFlags &= !CONFIGFLAG_FAILEDINSTALL;
    }

    if SetupDiSetDeviceRegistryProperty(
        DeviceInfoSet,
        DevInfoData,
        SPDRP_CONFIGFLAGS,
        core::ptr::addr_of_mut!(dwFlags) as PBYTE,
        dwSize,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if Set != 0 {
        /* Set the 'Unknown' device class */
        let mut pszUnknown: PWSTR = RUSTCONV_WSTR_F3CDEC9800CB.as_ptr() as PWSTR;
        SetupDiSetDeviceRegistryPropertyW(
            DeviceInfoSet,
            DevInfoData,
            SPDRP_CLASS,
            pszUnknown as PBYTE,
            ((wcslen(pszUnknown as *const u16) + 1) as usize
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );

        let mut pszUnknownGuid: PWSTR = RUSTCONV_WSTR_D3C8E20D82DB.as_ptr() as PWSTR;
        SetupDiSetDeviceRegistryPropertyW(
            DeviceInfoSet,
            DevInfoData,
            SPDRP_CLASSGUID,
            pszUnknownGuid as PBYTE,
            ((wcslen(pszUnknownGuid as *const u16) + 1) as usize
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );

        /* Set device problem code CM_PROB_FAILED_INSTALL */
        CM_Set_DevNode_Problem(
            (*DevInfoData).DevInst,
            CM_PROB_FAILED_INSTALL,
            CM_SET_DEVNODE_PROBLEM_OVERRIDE,
        );
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn CanDisableDevice(
    DevInst: DEVINST,
    hMachine: HMACHINE,
    CanDisable: *mut BOOL,
) -> BOOL {
    let mut cr: CONFIGRET;
    let mut Status: ULONG = 0;
    let mut ProblemNumber: ULONG = 0;
    let mut Ret: BOOL = (FALSE as i32);

    cr = CM_Get_DevNode_Status_Ex(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!(ProblemNumber),
        DevInst,
        0,
        hMachine,
    );
    if cr == CR_SUCCESS {
        *CanDisable = ((Status & DN_DISABLEABLE) != 0) as BOOL;
        Ret = (TRUE as i32);
    }

    return Ret;
}

unsafe extern "C" fn IsDeviceStarted(
    DevInst: DEVINST,
    hMachine: HMACHINE,
    IsEnabled: *mut BOOL,
) -> BOOL {
    let mut cr: CONFIGRET = 0;
    let mut Status: ULONG = 0;
    let mut ProblemNumber: ULONG = 0;
    let mut Ret: BOOL = (FALSE as i32);

    cr = CM_Get_DevNode_Status_Ex(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!(ProblemNumber),
        DevInst,
        0,
        hMachine,
    );
    if cr == CR_SUCCESS {
        *IsEnabled = ((Status & DN_STARTED) != 0) as BOOL;
        Ret = (TRUE as i32);
    }

    return Ret;
}

unsafe extern "C" fn StartDevice(
    mut DeviceInfoSet: HDEVINFO,
    mut DevInfoData: PSP_DEVINFO_DATA,
    mut bEnable: BOOL,
    mut HardwareProfile: DWORD,
    mut bNeedReboot: *mut BOOL,
) -> BOOL {
    let mut pcp: SP_PROPCHANGE_PARAMS = core::mem::zeroed();
    let mut dp: SP_DEVINSTALL_PARAMS = core::mem::zeroed();
    let mut LastErr: DWORD = 0;
    let mut Ret: BOOL = (FALSE as i32);

    pcp.ClassInstallHeader.cbSize = core::mem::size_of::<SP_CLASSINSTALL_HEADER>() as DWORD;
    pcp.ClassInstallHeader.InstallFunction = DIF_PROPERTYCHANGE as DI_FUNCTION;
    pcp.HwProfile = HardwareProfile;

    if bEnable != 0 {
        /* try to enable the device on the global profile */
        pcp.StateChange = DICS_ENABLE as DWORD;
        pcp.Scope = DICS_FLAG_GLOBAL as DWORD;

        /* ignore errors */
        LastErr = GetLastError();
        if SetupDiSetClassInstallParams(
            DeviceInfoSet,
            DevInfoData,
            core::ptr::addr_of_mut!(pcp.ClassInstallHeader),
            core::mem::size_of::<SP_PROPCHANGE_PARAMS>() as DWORD,
        ) != 0
        {
            SetupDiCallClassInstaller(
                DIF_PROPERTYCHANGE as DI_FUNCTION,
                DeviceInfoSet,
                DevInfoData,
            );
        }
        SetLastError(LastErr);
    }

    /* try config-specific */
    pcp.StateChange = (if bEnable != 0 {
        DICS_ENABLE
    } else {
        DICS_DISABLE
    }) as DWORD;
    pcp.Scope = DICS_FLAG_CONFIGSPECIFIC as DWORD;

    if SetupDiSetClassInstallParams(
        DeviceInfoSet,
        DevInfoData,
        core::ptr::addr_of_mut!(pcp.ClassInstallHeader),
        core::mem::size_of::<SP_PROPCHANGE_PARAMS>() as DWORD,
    ) != 0
        && SetupDiCallClassInstaller(
            DIF_PROPERTYCHANGE as DI_FUNCTION,
            DeviceInfoSet,
            DevInfoData,
        ) != 0
    {
        dp.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as DWORD;
        if SetupDiGetDeviceInstallParams(
            DeviceInfoSet,
            DevInfoData,
            core::ptr::addr_of_mut!(dp),
        ) != 0
        {
            if !bNeedReboot.is_null() {
                *bNeedReboot = ((dp.Flags & (DI_NEEDRESTART | DI_NEEDREBOOT))
                    != 0) as ::core::ffi::c_int as BOOL;
            }

            Ret = (TRUE as i32);
        }
    }
    return Ret;
}

unsafe extern "system" fn FindDriverProc(lpParam: LPVOID) -> DWORD {
    let mut DevInstData: PDEVINSTDATA = core::ptr::null_mut::<_DEVINSTDATA>();
    let mut result: BOOL = (FALSE as i32);

    DevInstData = lpParam as PDEVINSTDATA;

    result = ScanFoldersForDriver(DevInstData);

    if result != 0 {
        PostMessage(
            (*DevInstData).hDialog,
            WM_SEARCH_FINISHED as UINT,
            1 as WPARAM,
            0 as LPARAM,
        );
    } else {
        if (*DevInstData).bUpdate == 0 {
            /* Update device configuration */
            SetFailedInstall(
                (*DevInstData).hDevInfo,
                core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                TRUE as BOOLEAN,
            );
        }
        PostMessage(
            (*DevInstData).hDialog,
            WM_SEARCH_FINISHED as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }
    return 0;
}

unsafe extern "system" fn InstallDriverProc(lpParam: LPVOID) -> DWORD {
    let mut DevInstData: PDEVINSTDATA;
    let mut res: BOOL;

    DevInstData = lpParam as PDEVINSTDATA;
    res = InstallCurrentDriver(DevInstData);
    PostMessage((*DevInstData).hDialog, WM_INSTALL_FINISHED, (if res != 0 { 0 } else { 1 }) as WPARAM, 0 as LPARAM);
    return 0;
}

unsafe extern "C" fn PopulateCustomPathCombo(hwndCombo: HWND) {
    let _ = REGSTR_KEY_SETUP;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwRegType: DWORD = 0;
    let mut dwPathLength: DWORD = 0;
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut Path: LPCWSTR = core::ptr::null();
    let mut rc: LONG;

    ComboBox_ResetContent!(hwndCombo);

    'cleanup: {
        rc = RegOpenKeyEx(
            HKEY_LOCAL_MACHINE,
            REGSTR_PATH_SETUP,
            0,
            KEY_QUERY_VALUE,
            &mut hKey,
        );
        if rc != (ERROR_SUCCESS as i32) {
            break 'cleanup;
        }
        rc = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_6693190CF469.as_ptr(),
            core::ptr::null_mut(),
            &mut dwRegType,
            core::ptr::null_mut(),
            &mut dwPathLength,
        );
        if rc != (ERROR_SUCCESS as i32) || dwRegType != REG_MULTI_SZ {
            break 'cleanup;
        }

        /* Allocate enough space to add 2 NULL chars at the end of the string */
        Buffer = HeapAlloc(
            GetProcessHeap(),
            0,
            ((dwPathLength as usize)
                + (2 as usize) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPWSTR;
        if Buffer.is_null() {
            break 'cleanup;
        }
        rc = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_6693190CF469.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Buffer as LPBYTE,
            &mut dwPathLength,
        );
        if rc != (ERROR_SUCCESS as i32) {
            break 'cleanup;
        }

        *Buffer.add((dwPathLength as usize) / core::mem::size_of::<u16>()) =
            UNICODE_NULL;
        *Buffer.add(
            (dwPathLength as usize) / core::mem::size_of::<u16>()
                + (1 as usize),
        ) = UNICODE_NULL;

        /* Populate combo box */
        Path = Buffer as LPCWSTR;
        while *Path != 0 {
            ComboBox_AddString!(hwndCombo, Path);
            if Path == Buffer as LPCWSTR {
                ComboBox_SetCurSel!(hwndCombo, 0);
            }
            Path = Path.add(
                wcslen(Path) + (1 as usize),
            );
        }
    }

    // cleanup:
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        Buffer as LPVOID,
    );
}

unsafe extern "C" fn SaveCustomPath(hwndCombo: HWND) {
    let mut CustomPath: LPWSTR = core::ptr::null_mut();
    let mut CustomPathLength: DWORD = 0;
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut pBuffer: LPWSTR = core::ptr::null_mut();
    let mut ItemsCount: i32 = 0;
    let mut Length: i32 = 0;
    let mut i: i32 = 0;
    let mut TotalLength: DWORD = 0;
    let mut UseCustomPath: BOOL = (TRUE as i32);
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut rc: LONG = 0;

    'cleanup: {
        /* Get custom path */
        Length = ComboBox_GetTextLength!(hwndCombo) + 1;
        CustomPath = HeapAlloc(
            GetProcessHeap(),
            0,
            (Length as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if CustomPath.is_null() {
            break 'cleanup;
        }
        CustomPathLength = (GetWindowTextW(hwndCombo, CustomPath, Length)
            + 1) as DWORD;

        /* Calculate length of the buffer */
        ItemsCount = ComboBox_GetCount!(hwndCombo);
        if ItemsCount == CB_ERR {
            break 'cleanup;
        }
        i = 0;
        while i < ItemsCount {
            Length = ComboBox_GetLBTextLen!(hwndCombo, i);
            if Length == CB_ERR {
                break 'cleanup;
            }
            TotalLength = TotalLength.wrapping_add((Length + 1) as DWORD);
            i += 1;
        }
        TotalLength = TotalLength.wrapping_add(1); /* Final NULL char */

        /* Allocate buffer */
        Buffer = HeapAlloc(
            GetProcessHeap(),
            0,
            (CustomPathLength
                .wrapping_add(TotalLength)
                .wrapping_add(1 as DWORD) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if Buffer.is_null() {
            break 'cleanup;
        }

        /* Fill the buffer */
        pBuffer = Buffer.offset(CustomPathLength as isize);
        i = 0;
        while i < ItemsCount {
            Length = ComboBox_GetLBText!(hwndCombo, i, pBuffer);
            if Length == CB_ERR {
                break 'cleanup;
            } else if UseCustomPath != 0
                && _wcsicmp(CustomPath, pBuffer) == 0
            {
                UseCustomPath = (FALSE as i32);
            }
            pBuffer = pBuffer.offset((1 + Length) as isize);
            i += 1;
        }
        *pBuffer = 0 as WCHAR; /* Add final NULL char */

        if UseCustomPath == 0 {
            /* Nothing to save to registry */
            break 'cleanup;
        }

        TotalLength = TotalLength.wrapping_add(CustomPathLength);
        wcscpy(Buffer, CustomPath);

        /* Save the buffer */
        /* RegSetKeyValue would have been better... */
        let _ = REGSTR_KEY_SETUP; // setup subkey suffix (REGSTR_PATH_SETUP part)
        rc = RegOpenKeyEx(
            HKEY_LOCAL_MACHINE,
            REGSTR_PATH_SETUP,
            0,
            KEY_SET_VALUE,
            core::ptr::addr_of_mut!(hKey),
        );
        if rc != (ERROR_SUCCESS as i32) {
            break 'cleanup;
        }
        rc = RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_6693190CF469.as_ptr(),
            0,
            REG_MULTI_SZ,
            Buffer as *const BYTE,
            (TotalLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        );
        if rc != (ERROR_SUCCESS as i32) {
            break 'cleanup;
        }
    }

    // cleanup:
    if hKey != core::ptr::null_mut() {
        RegCloseKey(hKey);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        CustomPath as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        Buffer as PVOID,
    );
}

unsafe extern "system" fn WelcomeDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA = core::ptr::null_mut();
    UNREFERENCED_PARAMETER!(wParam);

    /* Retrieve pointer to the global setup data */
    DevInstData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    'block: {
        match uMsg {
            WM_INITDIALOG => {
                let mut hwndControl: HWND = core::ptr::null_mut();
                let mut dwStyle: DWORD = 0;

                /* Get pointer to the global setup data */
                DevInstData = (*(lParam as LPPROPSHEETPAGE)).lParam as PDEVINSTDATA;
                SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as LONG_PTR);

                hwndControl = GetParent(hwndDlg);

                /* Center the wizard window */
                CenterWindow(hwndControl);

                /* Hide the system menu */
                dwStyle = GetWindowLongPtr(hwndControl, GWL_STYLE) as DWORD;
                SetWindowLongPtr(
                    hwndControl,
                    GWL_STYLE,
                    (dwStyle & !WS_SYSMENU) as LONG_PTR,
                );

                /* Set title font */
                SendDlgItemMessage(
                    hwndDlg,
                    (IDC_WELCOMETITLE as i32),
                    WM_SETFONT,
                    (*DevInstData).hTitleFont as WPARAM,
                    TRUE as LPARAM,
                );

                SendDlgItemMessage(
                    hwndDlg,
                    (IDC_DEVICE as i32),
                    WM_SETTEXT,
                    0 as WPARAM,
                    (*DevInstData).buffer as LPARAM,
                );

                SendDlgItemMessage(
                    hwndDlg,
                    (IDC_RADIO_AUTO as i32),
                    BM_SETCHECK,
                    TRUE as WPARAM,
                    0 as LPARAM,
                );

                if (*DevInstData).bUpdate == 0 {
                    SetFailedInstall(
                        (*DevInstData).hDevInfo,
                        &raw mut (*DevInstData).devInfoData,
                        TRUE as BOOLEAN,
                    );
                }
                break 'block;
            }

            WM_NOTIFY => {
                let mut lpnm: LPNMHDR = lParam as LPNMHDR;

                match ((*lpnm).code) as i32 {
                    PSN_SETACTIVE => {
                        /* Enable the Next button */
                        PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
                        break 'block;
                    }

                    PSN_WIZNEXT => {
                        /* Handle a Next button click, if necessary */
                        if SendDlgItemMessage(
                            hwndDlg,
                            (IDC_RADIO_AUTO as i32),
                            BM_GETCHECK,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) == BST_CHECKED as LRESULT
                        {
                            if PrepareFoldersToScan(
                                DevInstData,
                                (TRUE) as BOOL,
                                (FALSE) as BOOL,
                                core::ptr::null_mut(),
                            ) != 0
                            {
                                SetWindowLongPtr(
                                    hwndDlg,
                                    DWLP_MSGRESULT,
                                    IDD_SEARCHDRV as LONG_PTR,
                                );
                            } else {
                                SetWindowLongPtr(
                                    hwndDlg,
                                    DWLP_MSGRESULT,
                                    IDD_INSTALLFAILED as LONG_PTR,
                                );
                            }
                        }
                        return TRUE as INT_PTR;
                    }

                    _ => {
                        break 'block;
                    }
                }
                break 'block;
            }

            _ => {
                break 'block;
            }
        }
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn IncludePath(Dlg: HWND, Enabled: BOOL) {
    EnableWindow(GetDlgItem(Dlg, (IDC_COMBO_PATH as i32)), Enabled);
    EnableWindow(GetDlgItem(Dlg, (IDC_BROWSE as i32)), Enabled);
}

unsafe extern "C" fn AutoDriver(Dlg: HWND, Enabled: BOOL) {
    EnableWindow(GetDlgItem(Dlg, (IDC_CHECK_MEDIA as i32)), Enabled);
    EnableWindow(GetDlgItem(Dlg, (IDC_CHECK_PATH as i32)), Enabled);
    IncludePath(Dlg, (Enabled as UINT & IsDlgButtonChecked(Dlg, (IDC_CHECK_PATH as i32))) as BOOL);
}

unsafe extern "system" fn BrowseCallbackProc(
    hwnd: HWND,
    uMsg: UINT,
    lParam: LPARAM,
    lpData: LPARAM,
) -> INT {
    let mut bValid: BOOL = (FALSE as i32);

    match uMsg {
        BFFM_INITIALIZED => 'block: {
            let pszPath: PCWSTR = (*(lpData as PDEVINSTDATA)).CustomSearchPath as PCWSTR;

            bValid = CheckBestDriver(lpData as PDEVINSTDATA, pszPath);
            SendMessageW(
                hwnd,
                BFFM_SETSELECTION as UINT,
                TRUE as WPARAM,
                pszPath as LPARAM,
            );
            SendMessageW(
                hwnd,
                BFFM_ENABLEOK as UINT,
                0 as WPARAM,
                bValid as LPARAM,
            );
            break 'block;
        }

        BFFM_SELCHANGED => 'block: {
            let mut szDir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

            if SHGetPathFromIDListW(
                lParam as LPITEMIDLIST as PCIDLIST_ABSOLUTE,
                core::ptr::addr_of_mut!(szDir) as LPWSTR,
            ) != 0
            {
                bValid = CheckBestDriver(
                    lpData as PDEVINSTDATA,
                    core::ptr::addr_of_mut!(szDir) as *mut WCHAR as PCWSTR,
                );
            }
            PostMessageW(
                hwnd,
                BFFM_ENABLEOK as UINT,
                0 as WPARAM,
                bValid as LPARAM,
            );
            break 'block;
        }

        _ => {}
    }

    return 0;
}

unsafe extern "system" fn CHSourceDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hwndControl: HWND = core::ptr::null_mut();
            let mut hwndCombo: HWND = core::ptr::null_mut();
            let mut dwStyle: DWORD = 0;
            let mut info: COMBOBOXINFO = core::mem::zeroed();
            info.cbSize = core::mem::size_of::<COMBOBOXINFO>() as DWORD;

            DevInstData = (*(lParam as LPPROPSHEETPAGE)).lParam as PDEVINSTDATA;
            SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as DWORD_PTR as LONG_PTR);

            hwndControl = GetParent(hwndDlg);

            CenterWindow(hwndControl);

            dwStyle = GetWindowLongPtr(hwndControl, GWL_STYLE) as DWORD;
            SetWindowLongPtr(hwndControl, GWL_STYLE, (dwStyle & !WS_SYSMENU) as LONG_PTR);

            hwndCombo = GetDlgItem(hwndDlg, (IDC_COMBO_PATH as i32));
            PopulateCustomPathCombo(hwndCombo);

            GetComboBoxInfo(hwndCombo, &raw mut info);
            SHAutoComplete(info.hwndItem, SHACF_FILESYS_DIRS);

            SendDlgItemMessage(
                hwndDlg,
                (IDC_RADIO_SEARCHHERE as i32),
                BM_SETCHECK,
                TRUE as WPARAM,
                0 as LPARAM);
            AutoDriver(hwndDlg, (TRUE) as BOOL);
            IncludePath(hwndDlg, (FALSE) as BOOL);

            EnableWindow(GetDlgItem(hwndDlg, (IDC_RADIO_CHOOSE as i32)), (FALSE) as BOOL);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_RADIO_SEARCHHERE => {
                    AutoDriver(hwndDlg, (TRUE) as BOOL);
                    return TRUE as INT_PTR;
                }

                IDC_RADIO_CHOOSE => {
                    AutoDriver(hwndDlg, (FALSE) as BOOL);
                    return TRUE as INT_PTR;
                }

                IDC_CHECK_PATH => {
                    IncludePath(hwndDlg, (IsDlgButtonChecked(hwndDlg, (IDC_CHECK_PATH as i32))) as BOOL);
                    return TRUE as INT_PTR;
                }

                IDC_BROWSE => {
                    let mut bi: BROWSEINFOW = core::mem::zeroed();
                    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
                    let mut Title: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
                    LoadStringW(
                        hDllInstance,
                        IDS_BROWSE_FOR_FOLDER_TITLE,
                        &raw mut Title as LPWSTR,
                        _countof!(Title) as ::core::ffi::c_int);

                    if PrepareFoldersToScan(DevInstData,
                        (FALSE) as BOOL,
                        (TRUE) as BOOL,
                        GetDlgItem(hwndDlg, (IDC_COMBO_PATH as i32))) != 0
                    {
                        bi.hwndOwner = hwndDlg;
                        bi.ulFlags = (BIF_USENEWUI | BIF_RETURNONLYFSDIRS | BIF_STATUSTEXT | BIF_NONEWFOLDERBUTTON) as UINT;
                        bi.lpszTitle = &raw mut Title as *mut WCHAR as LPCWSTR;
                        bi.lpfn = Some(BrowseCallbackProc);
                        bi.lParam = DevInstData as LPARAM;
                        pidl = SHBrowseForFolderW(&raw mut bi) as LPITEMIDLIST;
                    }

                    if !pidl.is_null() {
                        let mut Directory: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
                        let mut malloc: *mut IMalloc = core::ptr::null_mut();

                        if SHGetPathFromIDListW(pidl, &raw mut Directory as LPWSTR) != 0 {
                            SendDlgItemMessageW(
                                hwndDlg,
                                (IDC_COMBO_PATH as i32),
                                CB_SETCURSEL,
                                (-(1 as ::core::ffi::c_int)) as WPARAM,
                                0 as LPARAM);

                            SetWindowTextW(
                                GetDlgItem(hwndDlg, (IDC_COMBO_PATH as i32)),
                                &raw mut Directory as *mut WCHAR as LPCWSTR);
                        }

                        if SHGetMalloc(&raw mut malloc) >= 0 {
                            IMalloc_Free(malloc, pidl as LPVOID);
                            IMalloc_Release(malloc);
                        }
                        return TRUE as INT_PTR;
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT | PSWIZB_BACK);
                }

                PSN_WIZNEXT => {
                    if IsDlgButtonChecked(hwndDlg, (IDC_RADIO_SEARCHHERE as i32)) != 0 {
                        SaveCustomPath(GetDlgItem(hwndDlg, (IDC_COMBO_PATH as i32)));
                        HeapFree(GetProcessHeap(), 0, (*DevInstData).CustomSearchPath as PVOID);
                        (*DevInstData).CustomSearchPath = core::ptr::null_mut();
                        if PrepareFoldersToScan(
                            DevInstData,
                            IsDlgButtonChecked(hwndDlg, (IDC_CHECK_MEDIA as i32)) as BOOL,
                            IsDlgButtonChecked(hwndDlg, (IDC_CHECK_PATH as i32)) as BOOL,
                            GetDlgItem(hwndDlg, (IDC_COMBO_PATH as i32))) != 0
                        {
                            SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_SEARCHDRV as LONG_PTR);
                        }
                        else
                        {
                            SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_INSTALLFAILED as LONG_PTR);
                        }
                    }
                    else
                    {
                        /* FIXME */
                    }
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn SearchDrvDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA = ::core::ptr::null_mut::<_DEVINSTDATA>();
    let mut dwThreadId: DWORD = 0;

    /* Retrieve pointer to the global setup data */
    DevInstData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    'switch: {
        match uMsg {
            WM_INITDIALOG => {
                let mut hwndControl: HWND = ::core::ptr::null_mut::<HWND__>();
                let mut dwStyle: DWORD = 0;

                /* Get pointer to the global setup data */
                DevInstData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PDEVINSTDATA;
                SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as DWORD_PTR as LONG_PTR);

                (*DevInstData).hDialog = hwndDlg;
                hwndControl = GetParent(hwndDlg);

                /* Center the wizard window */
                CenterWindow(hwndControl);

                SendDlgItemMessage(
                    hwndDlg,
                    (IDC_DEVICE as i32),
                    WM_SETTEXT as UINT,
                    0 as WPARAM,
                    (*DevInstData).buffer as LPARAM,
                );

                /* Hide the system menu */
                dwStyle = GetWindowLongPtr(hwndControl, GWL_STYLE) as DWORD;
                SetWindowLongPtr(
                    hwndControl,
                    GWL_STYLE,
                    (dwStyle & !(WS_SYSMENU as DWORD)) as LONG_PTR,
                );
                break 'switch;
            }

            WM_SEARCH_FINISHED => {
                CloseHandle(hThread);
                hThread = core::ptr::null_mut();
                if wParam == 0 as WPARAM {
                    PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_NODRIVER as LPARAM);
                } else {
                    PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_INSTALLDRV as LPARAM);
                }
                break 'switch;
            }

            WM_NOTIFY => {
                let mut lpnm: LPNMHDR = lParam as LPNMHDR;

                'nswitch: {
                    match ((*lpnm).code) as i32 {
                        PSN_SETACTIVE => {
                            PropSheet_SetWizButtons!(
                                GetParent(hwndDlg),
                                ((PSWIZB_NEXT == 0) as ::core::ffi::c_int as LPARAM)
                                    | ((PSWIZB_BACK == 0) as ::core::ffi::c_int as LPARAM),
                            );
                            /* Yes, we can safely ignore the problem (if any) */
                            SetupDiDestroyDriverInfoList(
                                (*DevInstData).hDevInfo,
                                &raw mut (*DevInstData).devInfoData,
                                SPDIT_COMPATDRIVER as DWORD,
                            );
                            hThread = CreateThread(
                                ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
                                0 as DWORD,
                                Some(FindDriverProc),
                                DevInstData as PVOID,
                                0 as DWORD,
                                &raw mut dwThreadId,
                            );
                            break 'nswitch;
                        }

                        PSN_KILLACTIVE => {
                            if hThread != core::ptr::null_mut() {
                                SetWindowLongPtr(
                                    hwndDlg,
                                    DWLP_MSGRESULT,
                                    TRUE as LONG_PTR,
                                );
                                return TRUE as INT_PTR;
                            }
                            break 'nswitch;
                        }

                        PSN_WIZNEXT => {
                            /* Handle a Next button click, if necessary */
                            break 'nswitch;
                        }

                        _ => {
                            break 'nswitch;
                        }
                    }
                }
                break 'switch;
            }

            _ => {
                break 'switch;
            }
        }
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn InstallDrvDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA;
    let mut dwThreadId: DWORD = 0;

    /* Retrieve pointer to the global setup data */
    DevInstData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hwndControl: HWND = core::ptr::null_mut();
            let mut dwStyle: DWORD = 0;

            /* Get pointer to the global setup data */
            DevInstData = (*(lParam as LPPROPSHEETPAGE)).lParam as PDEVINSTDATA;
            SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as LONG_PTR);

            (*DevInstData).hDialog = hwndDlg;
            hwndControl = GetParent(hwndDlg);

            /* Center the wizard window */
            CenterWindow(hwndControl);

            SendDlgItemMessage(
                hwndDlg,
                (IDC_DEVICE as i32),
                WM_SETTEXT,
                0,
                core::ptr::addr_of_mut!((*DevInstData).drvInfoData.Description) as *mut WCHAR as LPARAM,
            );

            /* Hide the system menu */
            dwStyle = GetWindowLongPtr(hwndControl, GWL_STYLE) as DWORD;
            SetWindowLongPtr(
                hwndControl,
                GWL_STYLE,
                (dwStyle & !(WS_SYSMENU as DWORD)) as LONG_PTR,
            );
        }

        WM_INSTALL_FINISHED => {
            'c: {
                CloseHandle(hThread);
                hThread = 0 as HANDLE;
                if wParam == 0 as WPARAM {
                    let mut installParams: SP_DEVINSTALL_PARAMS = core::mem::zeroed();

                    SetFailedInstall(
                        (*DevInstData).hDevInfo,
                        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                        FALSE as BOOLEAN,
                    );

                    /* Should we reboot? */
                    installParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as DWORD;
                    if SetupDiGetDeviceInstallParams(
                        (*DevInstData).hDevInfo,
                        core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                        core::ptr::addr_of_mut!(installParams),
                    ) != 0
                    {
                        if installParams.Flags & ((DI_NEEDRESTART | DI_NEEDREBOOT) as DWORD) != 0 {
                            PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_NEEDREBOOT);
                        } else {
                            PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_FINISHPAGE);
                        }
                        break 'c;
                    }
                }
                PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_INSTALLFAILED);
            }
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(
                        GetParent(hwndDlg),
                        ((PSWIZB_NEXT as i32 == 0) as DWORD)
                            | ((PSWIZB_BACK as i32 == 0) as DWORD),
                    );
                    hThread = CreateThread(
                        core::ptr::null_mut(),
                        0 as DWORD,
                        Some(InstallDriverProc),
                        DevInstData as LPVOID,
                        0 as DWORD,
                        core::ptr::addr_of_mut!(dwThreadId),
                    );
                }

                PSN_KILLACTIVE => {
                    if hThread != 0 as HANDLE {
                        SetWindowLongPtr(
                            hwndDlg,
                            DWLP_MSGRESULT,
                            TRUE as LONG_PTR,
                        );
                        return TRUE as INT_PTR;
                    }
                }

                PSN_WIZNEXT => {
                    /* Handle a Next button click, if necessary */
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn NoDriverDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA = core::ptr::null_mut();
    let mut hwndControl: HWND = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(wParam);

    /* Get pointer to the global setup data */
    DevInstData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    'outer: {
        match uMsg {
            WM_INITDIALOG => {
                let mut DisableableDevice: BOOL = (FALSE as i32);

                /* Get pointer to the global setup data */
                DevInstData = (*(lParam as LPPROPSHEETPAGE)).lParam as PDEVINSTDATA;
                SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as LONG_PTR);

                /* Center the wizard window */
                CenterWindow(GetParent(hwndDlg));

                hwndControl = GetDlgItem(GetParent(hwndDlg), (IDCANCEL as i32));
                ShowWindow(hwndControl, (SW_HIDE as i32));
                EnableWindow(hwndControl, (FALSE) as BOOL);

                /* Set title font */
                SendDlgItemMessage(
                    hwndDlg,
                    (IDC_FINISHTITLE as i32),
                    WM_SETFONT,
                    (*DevInstData).hTitleFont as WPARAM,
                    TRUE as LPARAM,
                );

                /* disable the "do not show this dialog anymore" checkbox
                 if the device cannot be disabled */
                CanDisableDevice(
                    (*DevInstData).devInfoData.DevInst,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(DisableableDevice),
                );
                EnableWindow(
                    GetDlgItem(hwndDlg, (IDC_DONOTSHOWDLG as i32)),
                    DisableableDevice,
                );
                break 'outer;
            }

            WM_NOTIFY => {
                let mut lpnm: LPNMHDR = lParam as LPNMHDR;

                'inner: {
                    match ((*lpnm).code) as i32 {
                        PSN_SETACTIVE => {
                            /* Enable the correct buttons on for the active page */
                            PropSheet_SetWizButtons!(
                                GetParent(hwndDlg),
                                PSWIZB_BACK | PSWIZB_FINISH,
                            );
                            break 'inner;
                        }

                        PSN_WIZBACK => {
                            /* Handle a Back button click, if necessary */
                            hwndControl = GetDlgItem(GetParent(hwndDlg), (IDCANCEL as i32));
                            ShowWindow(hwndControl, (SW_SHOW as i32));
                            EnableWindow(hwndControl, (TRUE) as BOOL);
                            SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_CHSOURCE as LONG_PTR);
                            return TRUE as INT_PTR;
                        }

                        PSN_WIZFINISH => {
                            let mut DisableableDevice: BOOL = (FALSE as i32);
                            let mut IsStarted: BOOL = (FALSE as i32);

                            if CanDisableDevice(
                                (*DevInstData).devInfoData.DevInst,
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(DisableableDevice),
                            ) != 0
                                && DisableableDevice != 0
                                && IsDeviceStarted(
                                    (*DevInstData).devInfoData.DevInst,
                                    core::ptr::null_mut(),
                                    core::ptr::addr_of_mut!(IsStarted),
                                ) != 0
                                && IsStarted == 0
                                && SendDlgItemMessage(
                                    hwndDlg,
                                    (IDC_DONOTSHOWDLG as i32),
                                    BM_GETCHECK,
                                    0 as WPARAM,
                                    0 as LPARAM,
                                ) == BST_CHECKED as LRESULT
                            {
                                /* disable the device */
                                StartDevice(
                                    (*DevInstData).hDevInfo,
                                    core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                                    (FALSE) as BOOL,
                                    0 as DWORD,
                                    core::ptr::null_mut::<BOOL>(),
                                );
                            } else {
                                SetFailedInstall(
                                    (*DevInstData).hDevInfo,
                                    core::ptr::addr_of_mut!((*DevInstData).devInfoData),
                                    FALSE as BOOLEAN,
                                );
                            }
                            break 'inner;
                        }

                        _ => {
                            break 'inner;
                        }
                    }
                }
                break 'outer;
            }

            _ => {
                break 'outer;
            }
        }
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn InstallFailedDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA = core::ptr::null_mut();
    UNREFERENCED_PARAMETER!(wParam);

    /* Retrieve pointer to the global setup data */
    DevInstData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hwndControl: HWND = core::ptr::null_mut();

            /* Get pointer to the global setup data */
            DevInstData = (*(lParam as LPPROPSHEETPAGE)).lParam as PDEVINSTDATA;
            SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as LONG_PTR);

            /* Center the wizard window */
            CenterWindow(GetParent(hwndDlg));

            hwndControl = GetDlgItem(GetParent(hwndDlg), (IDCANCEL as i32));
            ShowWindow(hwndControl, (SW_HIDE as i32));
            EnableWindow(hwndControl, (FALSE) as BOOL);

            SendDlgItemMessage(
                hwndDlg,
                (IDC_DEVICE as i32),
                WM_SETTEXT,
                0,
                &raw mut (*DevInstData).drvInfoData.Description as *mut WCHAR as LPARAM,
            );

            /* Set title font */
            SendDlgItemMessage(
                hwndDlg,
                (IDC_FINISHTITLE as i32),
                WM_SETFONT,
                (*DevInstData).hTitleFont as WPARAM,
                TRUE as LPARAM,
            );
            // break;
        }

        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Enable the correct buttons on for the active page */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH);
                    // break;
                }

                PSN_WIZBACK => {
                    /* Handle a Back button click, if necessary */
                    // break;
                }

                PSN_WIZFINISH => {
                    /* Handle a Finish button click, if necessary */
                    // break;
                }

                _ => {
                    // break;
                }
            }
            // break;
        }

        _ => {
            // break;
        }
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn NeedRebootDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA = core::ptr::null_mut::<_DEVINSTDATA>();
    UNREFERENCED_PARAMETER!(wParam);

    /* Retrieve pointer to the global setup data */
    DevInstData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hwndControl: HWND = core::ptr::null_mut();

            /* Get pointer to the global setup data */
            DevInstData = (*(lParam as LPPROPSHEETPAGE)).lParam as PDEVINSTDATA;
            SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as DWORD_PTR as LONG_PTR);

            /* Center the wizard window */
            CenterWindow(GetParent(hwndDlg));

            hwndControl = GetDlgItem(GetParent(hwndDlg), (IDCANCEL as i32));
            ShowWindow(hwndControl, (SW_HIDE as i32));
            EnableWindow(hwndControl, (FALSE) as BOOL);

            SendDlgItemMessage(
                hwndDlg,
                (IDC_DEVICE as i32),
                WM_SETTEXT,
                0,
                core::ptr::addr_of_mut!((*DevInstData).drvInfoData.Description) as LPARAM,
            );

            /* Set title font */
            SendDlgItemMessage(
                hwndDlg,
                (IDC_FINISHTITLE as i32),
                WM_SETFONT,
                (*DevInstData).hTitleFont as WPARAM,
                TRUE as LPARAM,
            );
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Enable the correct buttons on for the active page */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH);
                }

                PSN_WIZBACK => {
                    /* Handle a Back button click, if necessary */
                }

                PSN_WIZFINISH => {
                    /* Handle a Finish button click, if necessary */
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn FinishDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut DevInstData: PDEVINSTDATA = ::core::ptr::null_mut::<_DEVINSTDATA>();
    UNREFERENCED_PARAMETER!(wParam);

    /* Retrieve pointer to the global setup data */
    DevInstData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEVINSTDATA;

    match uMsg {
        WM_INITDIALOG => 'block: {
            let mut hwndControl: HWND = ::core::ptr::null_mut::<HWND__>();

            /* Get pointer to the global setup data */
            DevInstData = (*(lParam as LPPROPSHEETPAGE)).lParam as PDEVINSTDATA;
            SetWindowLongPtr(hwndDlg, GWLP_USERDATA, DevInstData as LONG_PTR);

            /* Center the wizard window */
            CenterWindow(GetParent(hwndDlg));

            hwndControl = GetDlgItem(GetParent(hwndDlg), (IDCANCEL as i32));
            ShowWindow(hwndControl, (SW_HIDE as i32));
            EnableWindow(hwndControl, (FALSE) as BOOL);

            SendDlgItemMessage(
                hwndDlg,
                (IDC_DEVICE as i32),
                WM_SETTEXT,
                0,
                core::ptr::addr_of_mut!((*DevInstData).drvInfoData.Description) as LPARAM,
            );

            /* Set title font */
            SendDlgItemMessage(
                hwndDlg,
                (IDC_FINISHTITLE as i32),
                WM_SETFONT,
                (*DevInstData).hTitleFont as WPARAM,
                TRUE as LPARAM,
            );
            break 'block;
        }

        WM_NOTIFY => 'block: {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Enable the correct buttons on for the active page */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH);
                }

                PSN_WIZBACK => {
                    /* Handle a Back button click, if necessary */
                }

                PSN_WIZFINISH => {
                    /* Handle a Finish button click, if necessary */
                }

                _ => {}
            }
            break 'block;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn CreateTitleFont() -> HFONT {
    let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed();
    let mut LogFont: LOGFONTW = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut FontSize: INT = 0;
    let mut hFont: HFONT = core::ptr::null_mut();

    ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
    SystemParametersInfo(SPI_GETNONCLIENTMETRICS, 0, core::ptr::addr_of_mut!(ncm) as LPVOID, 0);

    LogFont = ncm.lfMessageFont;
    LogFont.lfWeight = (FW_BOLD as i32);
    wcscpy(core::ptr::addr_of_mut!(LogFont.lfFaceName) as *mut u16, RUSTCONV_WSTR_491507152B61.as_ptr());

    hdc = GetDC(core::ptr::null_mut());
    FontSize = 12;
    LogFont.lfHeight = (0 - GetDeviceCaps(hdc, (LOGPIXELSY as i32)) * FontSize / 72) as LONG;
    hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(LogFont));
    ReleaseDC(core::ptr::null_mut(), hdc);

    return hFont;
}
#[no_mangle]
pub unsafe extern "C" fn DisplayWizard(
    DevInstData: PDEVINSTDATA,
    hwndParent: HWND,
    mut startPage: UINT,
) -> BOOL {
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut ahpsp: [HPROPSHEETPAGE; (IDD_MAXIMUMPAGE + 1) as _] =
        [core::ptr::null_mut(); (IDD_MAXIMUMPAGE + 1) as _];
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();
    let mut hr: HRESULT = CoInitialize(core::ptr::null_mut()); /* for SHAutoComplete */

    /* zero based index */
    startPage = startPage.wrapping_sub(IDD_FIRSTPAGE as UINT);

    /* Create the Welcome page */
    ZeroMemory!(
        core::ptr::addr_of_mut!(psp) as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETPAGE>() as usize,
    );
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER | PSP_USETITLE) as DWORD;
    psp.hInstance = hDllInstance;
    psp.lParam = DevInstData as LPARAM;
    psp.pszTitle = MAKEINTRESOURCE!(if (*DevInstData).bUpdate != 0 {
        IDS_UPDATEWIZARDTITLE
    } else {
        IDS_INSTALLWIZARDTITLE
    }) as LPCWSTR;
    psp.pfnDlgProc = Some(WelcomeDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_WELCOMEPAGE) as LPCWSTR;
    ahpsp[(IDD_WELCOMEPAGE - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Select Source page */
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USETITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_CHSOURCE_TITLE) as LPCWSTR;
    psp.pfnDlgProc = Some(CHSourceDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_CHSOURCE) as LPCWSTR;
    ahpsp[(IDD_CHSOURCE - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Search driver page */
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USETITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_SEARCHDRV_TITLE) as LPCWSTR;
    psp.pfnDlgProc = Some(SearchDrvDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_SEARCHDRV) as LPCWSTR;
    ahpsp[(IDD_SEARCHDRV - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Install driver page */
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USETITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_INSTALLDRV_TITLE) as LPCWSTR;
    psp.pfnDlgProc = Some(InstallDrvDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_INSTALLDRV) as LPCWSTR;
    ahpsp[(IDD_INSTALLDRV - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the No driver page */
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER | PSP_USETITLE) as DWORD;
    psp.pfnDlgProc = Some(NoDriverDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_NODRIVER) as LPCWSTR;
    ahpsp[(IDD_NODRIVER - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Install failed page */
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER | PSP_USETITLE) as DWORD;
    psp.pfnDlgProc = Some(InstallFailedDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_INSTALLFAILED) as LPCWSTR;
    ahpsp[(IDD_INSTALLFAILED - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Need reboot page */
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER | PSP_USETITLE) as DWORD;
    psp.pfnDlgProc = Some(NeedRebootDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_NEEDREBOOT) as LPCWSTR;
    ahpsp[(IDD_NEEDREBOOT - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the Finish page */
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER | PSP_USETITLE) as DWORD;
    psp.pfnDlgProc = Some(FinishDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_FINISHPAGE) as LPCWSTR;
    ahpsp[(IDD_FINISHPAGE - IDD_FIRSTPAGE) as usize] =
        CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the property sheet */
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_WIZARD97 | PSH_WATERMARK | PSH_HEADER) as DWORD;
    psh.hInstance = hDllInstance;
    psh.hwndParent = hwndParent;
    psh.nPages = (IDD_MAXIMUMPAGE + 1) as UINT;
    psh.__bindgen_anon_2.nStartPage = startPage;
    psh.__bindgen_anon_3.phpage = core::ptr::addr_of_mut!(ahpsp) as *mut HPROPSHEETPAGE;
    psh.__bindgen_anon_4.pszbmWatermark = MAKEINTRESOURCE!(IDB_WATERMARK) as LPCWSTR;
    psh.__bindgen_anon_5.pszbmHeader = MAKEINTRESOURCE!(IDB_HEADER) as LPCWSTR;

    /* Create title font */
    (*DevInstData).hTitleFont = CreateTitleFont();

    /* Display the wizard */
    PropertySheet(core::ptr::addr_of_mut!(psh));

    DeleteObject((*DevInstData).hTitleFont as HGDIOBJ);

    if hr >= 0 {
        CoUninitialize();
    }
    return (TRUE) as BOOL;
}
