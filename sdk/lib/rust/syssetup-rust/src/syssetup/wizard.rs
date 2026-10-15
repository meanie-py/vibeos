// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _REGISTRATIONDATA {
    pub hwndDlg: HWND,
    pub DllCount: ULONG,
    pub Registered: ULONG,
    pub DefaultContext: PVOID,
    pub pNotify: PREGISTRATIONNOTIFY,
}
pub type REGISTRATIONDATA = _REGISTRATIONDATA;
pub type PREGISTRATIONDATA = *mut _REGISTRATIONDATA;

#[repr(C)]
pub struct _TIMEZONE_ENTRY {
    pub Prev: *mut _TIMEZONE_ENTRY,
    pub Next: *mut _TIMEZONE_ENTRY,
    pub Description: [WCHAR; 128],
    pub StandardName: [WCHAR; 32],
    pub DaylightName: [WCHAR; 32],
    pub TimezoneInfo: REG_TZI_FORMAT,
    pub Index: ULONG,
}
pub type TIMEZONE_ENTRY = _TIMEZONE_ENTRY;
pub type PTIMEZONE_ENTRY = *mut _TIMEZONE_ENTRY;

// wizard.c:381
pub static s_szProductOptions: [WCHAR; 48] = wide!("SYSTEM\\CurrentControlSet\\Control\\ProductOptions");

// wizard.c:382
pub static s_szRosVersion: [WCHAR; 58] = wide!("SYSTEM\\CurrentControlSet\\Control\\ReactOS\\Settings\\Version");

// wizard.c:383
pub static s_szControlWindows: [WCHAR; 41] = wide!("SYSTEM\\CurrentControlSet\\Control\\Windows");

// wizard.c:384
pub static s_szWinlogon: [WCHAR; 54] = wide!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon");

// wizard.c:385
pub static s_szDefaultSoundEvents: [WCHAR; 32] = wide!("AppEvents\\Schemes\\Apps\\.Default");

// wizard.c:386
pub static s_szExplorerSoundEvents: [WCHAR; 32] = wide!("AppEvents\\Schemes\\Apps\\Explorer");

#[repr(C)]
pub struct _PRODUCT_OPTION_DATA {
    pub ProductSuite: LPCWSTR,
    pub ProductType: LPCWSTR,
    pub ReportAsWorkstation: DWORD,
    pub CSDVersion: DWORD,
    pub LogonType: DWORD,
}
pub type PRODUCT_OPTION_DATA = _PRODUCT_OPTION_DATA;

pub static s_ProductOptionData: SyncTable<[PRODUCT_OPTION_DATA; 2]> = SyncTable([
    PRODUCT_OPTION_DATA {
        ProductSuite: w!("Terminal Server\0"),
        ProductType: w!("ServerNT"),
        ReportAsWorkstation: 0,
        CSDVersion: 0x200,
        LogonType: 0,
    },
    PRODUCT_OPTION_DATA {
        ProductSuite: w!("\0"),
        ProductType: w!("WinNT"),
        ReportAsWorkstation: 1,
        CSDVersion: 0x300,
        LogonType: 1,
    },
]);

pub static s_DefaultSoundEvents: SyncTable<[[LPCWSTR; 2]; 25]> = SyncTable([
    [
        w!(".Default"),
        w!("%SystemRoot%\\Media\\ReactOS_Default.wav"),
    ],
    [
        w!("AppGPFault"),
        w!(""),
    ],
    [
        w!("Close"),
        w!(""),
    ],
    [
        w!("CriticalBatteryAlarm"),
        w!("%SystemRoot%\\Media\\ReactOS_Battery_Critical.wav"),
    ],
    [
        w!("DeviceConnect"),
        w!("%SystemRoot%\\Media\\ReactOS_Hardware_Insert.wav"),
    ],
    [
        w!("DeviceDisconnect"),
        w!("%SystemRoot%\\Media\\ReactOS_Hardware_Remove.wav"),
    ],
    [
        w!("DeviceFail"),
        w!("%SystemRoot%\\Media\\ReactOS_Hardware_Fail.wav"),
    ],
    [
        w!("LowBatteryAlarm"),
        w!("%SystemRoot%\\Media\\ReactOS_Battery_Low.wav"),
    ],
    [
        w!("MailBeep"),
        w!("%SystemRoot%\\Media\\ReactOS_Notify.wav"),
    ],
    [
        w!("Maximize"),
        w!("%SystemRoot%\\Media\\ReactOS_Restore.wav"),
    ],
    [
        w!("MenuCommand"),
        w!("%SystemRoot%\\Media\\ReactOS_Menu_Command.wav"),
    ],
    [
        w!("MenuPopup"),
        w!(""),
    ],
    [
        w!("Minimize"),
        w!("%SystemRoot%\\Media\\ReactOS_Minimize.wav"),
    ],
    [
        w!("Open"),
        w!(""),
    ],
    [
        w!("PrintComplete"),
        w!("%SystemRoot%\\Media\\ReactOS_Print_Complete.wav"),
    ],
    [
        w!("RestoreDown"),
        w!(""),
    ],
    [
        w!("RestoreUp"),
        w!(""),
    ],
    [
        w!("SystemAsterisk"),
        w!("%SystemRoot%\\Media\\ReactOS_Ding.wav"),
    ],
    [
        w!("SystemExclamation"),
        w!("%SystemRoot%\\Media\\ReactOS_Exclamation.wav"),
    ],
    [
        w!("SystemExit"),
        w!("%SystemRoot%\\Media\\ReactOS_Shutdown.wav"),
    ],
    [
        w!("SystemHand"),
        w!("%SystemRoot%\\Media\\ReactOS_Critical_Stop.wav"),
    ],
    [
        w!("SystemNotification"),
        w!("%SystemRoot%\\Media\\ReactOS_Balloon.wav"),
    ],
    [
        w!("SystemQuestion"),
        w!("%SystemRoot%\\Media\\ReactOS_Ding.wav"),
    ],
    [
        w!("SystemStart"),
        w!("%SystemRoot%\\Media\\ReactOS_Startup.wav"),
    ],
    [
        w!("WindowsLogoff"),
        w!("%SystemRoot%\\Media\\ReactOS_LogOff.wav"),
    ],
]);

pub static s_ExplorerSoundEvents: SyncTable<[[LPCWSTR; 2]; 2]> = SyncTable([
    [
        w!("EmptyRecycleBin"),
        w!("%SystemRoot%\\Media\\ReactOS_Recycle.wav"),
    ],
    [
        w!("Navigating"),
        w!("%SystemRoot%\\Media\\ReactOS_Start.wav"),
    ],
]);

#[repr(C)]
pub struct ThemeInfo {
    pub PreviewBitmap: LPCWSTR,
    pub DisplayName: UINT,
    pub ThemeFile: LPCWSTR,
}

pub static Themes: SyncTable<[ThemeInfo; 4]> = SyncTable([
    ThemeInfo {
        PreviewBitmap: 100 as LPCWSTR,
        DisplayName: 4101,
        ThemeFile: core::ptr::null(),
    },
    ThemeInfo {
        PreviewBitmap: 101 as LPCWSTR,
        DisplayName: 4102,
        ThemeFile: w!("themes\\lautus\\lautus.msstyles"),
    },
    ThemeInfo {
        PreviewBitmap: 102 as LPCWSTR,
        DisplayName: 4103,
        ThemeFile: w!("themes\\lunar\\lunar.msstyles"),
    },
    ThemeInfo {
        PreviewBitmap: 103 as LPCWSTR,
        DisplayName: 4104,
        ThemeFile: w!("themes\\mizu\\mizu.msstyles"),
    },
]);

pub type PFNREQUESTWIZARDPAGES = Option<unsafe extern "system" fn(PDWORD, *mut HPROPSHEETPAGE, PSETUPDATA) -> DWORD>;
unsafe extern "C" fn CenterWindow(hWnd: HWND) {
    let mut hWndParent: HWND;
    let mut rcParent: RECT = core::mem::zeroed();
    let mut rcWindow: RECT = core::mem::zeroed();

    hWndParent = GetParent(hWnd);
    if hWndParent == core::ptr::null_mut() {
        hWndParent = GetDesktopWindow();
    }

    GetWindowRect(hWndParent, &mut rcParent);
    GetWindowRect(hWnd, &mut rcWindow);

    SetWindowPos(
        hWnd,
        HWND_TOP,
        ((rcParent.right - rcParent.left) - (rcWindow.right - rcWindow.left)) / 2,
        ((rcParent.bottom - rcParent.top) - (rcWindow.bottom - rcWindow.top)) / 2,
        0,
        0,
        (SWP_NOSIZE) as u32,
    );
}
unsafe extern "C" fn CreateTitleFont() -> HFONT {
    let mut LogFont: LOGFONTW = core::mem::zeroed();
    let hdc: HDC;
    let hFont: HFONT;

    LogFont.lfWeight = (FW_BOLD) as i32;
    wcscpy(LogFont.lfFaceName.as_mut_ptr(), w!("MS Shell Dlg") as LPCWSTR);

    hdc = GetDC(core::ptr::null_mut());
    LogFont.lfHeight = -MulDiv(12, GetDeviceCaps(hdc, (LOGPIXELSY) as i32), 72);

    hFont = CreateFontIndirectW(&mut LogFont);

    ReleaseDC(core::ptr::null_mut(), hdc);

    hFont
}
unsafe extern "C" fn CreateBoldFont() -> HFONT {
    let mut tmpFont: LOGFONTW = core::mem::zeroed();
    let hBoldFont: HFONT;
    let hDc: HDC;

    hDc = GetDC(core::ptr::null_mut());

    tmpFont.lfHeight = -MulDiv(8, GetDeviceCaps(hDc, (LOGPIXELSY) as i32), 72);
    tmpFont.lfWeight = (FW_BOLD) as i32;
    wcscpy(tmpFont.lfFaceName.as_mut_ptr(), w!("MS Shell Dlg"));

    hBoldFont = CreateFontIndirectW(&mut tmpFont);

    ReleaseDC(core::ptr::null_mut(), hDc);

    return hBoldFont;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn GplDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut GplTextResource: HRSRC;
    let mut GplTextMem: HGLOBAL;
    let mut GplTextLocked: PVOID;
    let mut GplText: PCHAR;
    let mut Size: DWORD;

    match uMsg {
        WM_INITDIALOG => {
            GplTextResource = FindResourceW(
                hDllInstance,
                MAKEINTRESOURCE!(IDR_GPL),
                w!("RT_TEXT") as LPCWSTR,
            );
            if GplTextResource == core::ptr::null_mut() {
                return FALSE;
            }
            Size = SizeofResource(hDllInstance, GplTextResource);
            if Size == 0 {
                return FALSE;
            }
            GplText = HeapAlloc(
                GetProcessHeap(),
                0,
                Size + 1,
            ) as PCHAR;
            if GplText == core::ptr::null_mut() {
                return FALSE;
            }
            GplTextMem = LoadResource(hDllInstance, GplTextResource);
            if GplTextMem == core::ptr::null_mut() {
                HeapFree(GetProcessHeap(), 0, GplText as LPVOID);
                return FALSE;
            }
            GplTextLocked = LockResource(GplTextMem);
            if GplTextLocked == core::ptr::null_mut() {
                HeapFree(GetProcessHeap(), 0, GplText as LPVOID);
                return FALSE;
            }
            memcpy(GplText as *mut core::ffi::c_void, GplTextLocked, Size as usize);
            *GplText.offset(Size as isize) = b'\0' as i8;
            SendMessageA(
                GetDlgItem(hwndDlg, (IDC_GPL_TEXT) as i32),
                WM_SETTEXT,
                0,
                GplText as LPARAM,
            );
            HeapFree(GetProcessHeap(), 0, GplText as LPVOID);
            SetFocus(GetDlgItem(hwndDlg, (IDOK) as i32));
            return FALSE;
        }

        WM_CLOSE => {
            EndDialog(hwndDlg, (IDCANCEL) as i32);
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (BN_CLICKED) as u16 && IDOK == LOWORD!(wParam) as u32 {
                EndDialog(hwndDlg, (IDOK) as i32);
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn GplDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut GplTextResource: HRSRC;
    let mut GplTextMem: HGLOBAL;
    let mut GplTextLocked: PVOID;
    let mut GplText: PCHAR;
    let mut Size: DWORD;

    match uMsg {
        WM_INITDIALOG => {
            GplTextResource = FindResourceW(
                hDllInstance,
                MAKEINTRESOURCE!(IDR_GPL),
                w!("RT_TEXT") as LPCWSTR,
            );
            if GplTextResource == core::ptr::null_mut() {
                return (FALSE) as i64;
            }
            Size = SizeofResource(hDllInstance, GplTextResource);
            if Size == 0 {
                return (FALSE) as i64;
            }
            GplText = HeapAlloc(
                GetProcessHeap(),
                0,
                (Size + 1) as u64,
            ) as PCHAR;
            if GplText == core::ptr::null_mut() {
                return (FALSE) as i64;
            }
            GplTextMem = LoadResource(hDllInstance, GplTextResource);
            if GplTextMem == core::ptr::null_mut() {
                HeapFree(GetProcessHeap(), 0, GplText as LPVOID);
                return (FALSE) as i64;
            }
            GplTextLocked = LockResource(GplTextMem);
            if GplTextLocked == core::ptr::null_mut() {
                HeapFree(GetProcessHeap(), 0, GplText as LPVOID);
                return (FALSE) as i64;
            }
            memcpy(GplText as *mut core::ffi::c_void, GplTextLocked, Size as usize);
            *GplText.offset(Size as isize) = b'\0' as i8;
            SendMessageA(
                GetDlgItem(hwndDlg, (IDC_GPL_TEXT) as i32),
                (WM_SETTEXT) as u32,
                0,
                GplText as LPARAM,
            );
            HeapFree(GetProcessHeap(), 0, GplText as LPVOID);
            SetFocus(GetDlgItem(hwndDlg, (IDOK) as i32));
            return (FALSE) as i64;
        }

        WM_CLOSE => {
            EndDialog(hwndDlg, (IDCANCEL) as i64);
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (BN_CLICKED) as u16 && IDOK == LOWORD!(wParam) as u32 {
                EndDialog(hwndDlg, (IDOK) as i64);
            }
        }

        _ => {}
    }

    (FALSE) as i64
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn WelcomeDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hwndControl: HWND;
            let mut dwStyle: DWORD;

            pSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            hwndControl = GetParent(hwndDlg);

            CenterWindow(hwndControl);

            dwStyle = GetWindowLongPtr(hwndControl, GWL_STYLE) as DWORD;
            SetWindowLongPtr(hwndControl, (GWL_STYLE) as c_int, ((dwStyle & !WS_SYSMENU)) as LONG_PTR);

            hwndControl = GetDlgItem(GetParent(hwndDlg), (IDCANCEL) as i32);
            ShowWindow(hwndControl, (SW_HIDE) as i32);
            EnableWindow(hwndControl, FALSE);

            SendDlgItemMessage(
                hwndDlg,
                (IDC_WELCOMETITLE) as i32,
                WM_SETFONT,
                (*pSetupData).hTitleFont as WPARAM,
                TRUE as LPARAM,
            );
        }

        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    LogItem!(w!("BEGIN") as LPCWSTR, w!("WelcomePage") as LPCWSTR);
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 {
                        SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, (IDD_ACKPAGE) as LONG_PTR);
                        return TRUE;
                    }
                }

                PSN_WIZNEXT => {
                    LogItem!(w!("END") as LPCWSTR, w!("WelcomePage") as LPCWSTR);
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn WelcomeDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hwndControl: HWND;
            let mut dwStyle: DWORD;

            pSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            hwndControl = GetParent(hwndDlg);

            CenterWindow(hwndControl);

            dwStyle = GetWindowLongPtr(hwndControl, GWL_STYLE) as DWORD;
            SetWindowLongPtr(hwndControl, (GWL_STYLE) as c_int, ((dwStyle & (!WS_SYSMENU) as u32)) as LONG_PTR);

            hwndControl = GetDlgItem(GetParent(hwndDlg), (IDCANCEL) as i32);
            ShowWindow(hwndControl, (SW_HIDE) as i32);
            EnableWindow(hwndControl, (FALSE) as i32);

            SendDlgItemMessage(
                hwndDlg,
                (IDC_WELCOMETITLE) as i32,
                (WM_SETFONT) as u32,
                (*pSetupData).hTitleFont as WPARAM,
                TRUE as LPARAM,
            );
        }

        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    LogItem!(w!("BEGIN") as LPCWSTR, w!("WelcomePage") as LPCWSTR);
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 {
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((IDD_ACKPAGE)) as LONG_PTR);
                        return (TRUE) as i64;
                    }
                }

                PSN_WIZNEXT => {
                    LogItem!(w!("END") as LPCWSTR, w!("WelcomePage") as LPCWSTR);
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as i64
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn AckPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let lpnm: LPNMHDR;
    let mut Projects: PWSTR;
    let mut End: PWSTR;
    let mut CurrentProject: PWSTR;
    let mut ProjectsSize: INT;
    let ProjectsCount: INT;
    let pSetupData: PSETUPDATA;

    let pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let pSetupData = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            Projects = core::ptr::null_mut();
            ProjectsSize = 256;
            loop {
                Projects = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    ((ProjectsSize as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
                ) as PWSTR;
                if Projects.is_null() {
                    return FALSE;
                }
                let ProjectsCount = LoadStringW(
                    hDllInstance,
                    IDS_ACKPROJECTS,
                    Projects,
                    ProjectsSize,
                );
                if 0 == ProjectsCount {
                    HeapFree(GetProcessHeap(), 0, Projects as LPVOID);
                    return FALSE;
                }
                if ProjectsCount < ProjectsSize - 1 {
                    break;
                }
                HeapFree(GetProcessHeap(), 0, Projects as LPVOID);
                ProjectsSize *= 2;
            }

            CurrentProject = Projects;
            while *CurrentProject != 0u16 {
                End = wcschr(CurrentProject, '\n' as u16);
                if !End.is_null() {
                    *End = 0u16;
                }
                ListBox_AddString!(
                    GetDlgItem(hwndDlg, (IDC_PROJECTS) as i32),
                    CurrentProject,
                );
                if !End.is_null() {
                    CurrentProject = End.offset(1 as isize);
                } else {
                    CurrentProject = CurrentProject.offset(wcslen(CurrentProject) as isize);
                }
            }
            HeapFree(GetProcessHeap(), 0, Projects as LPVOID);
        }

        WM_COMMAND => {
            if HIWORD!(wParam as DWORD) == (BN_CLICKED) as u16
                && IDC_VIEWGPL == LOWORD!(wParam as DWORD) as u32
            {
                DialogBox!(
                    hDllInstance,
                    MAKEINTRESOURCE!(IDD_GPL),
                    core::ptr::null_mut(),
                    Some(GplDlgProc),
                );
                SetForegroundWindow(GetParent(hwndDlg));
            }
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(
                        GetParent(hwndDlg),
                        PSWIZB_BACK | PSWIZB_NEXT,
                    );
                    if (*pSetupData).UnattendSetup != 0 {
                        SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, (IDD_PRODUCT) as LONG_PTR);
                        return TRUE;
                    }
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn AckPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let lpnm: LPNMHDR;
    let mut Projects: PWSTR;
    let mut End: PWSTR;
    let mut CurrentProject: PWSTR;
    let mut ProjectsSize: INT;
    let ProjectsCount: INT;
    let pSetupData: PSETUPDATA;

    let pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let pSetupData = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            Projects = core::ptr::null_mut();
            ProjectsSize = 256;
            loop {
                Projects = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    ((ProjectsSize as usize) * core::mem::size_of::<WCHAR>()) as u64,
                ) as PWSTR;
                if Projects.is_null() {
                    return (FALSE) as i64;
                }
                let ProjectsCount = LoadStringW(
                    hDllInstance,
                    (IDS_ACKPROJECTS) as u32,
                    Projects,
                    ProjectsSize,
                );
                if 0 == ProjectsCount {
                    HeapFree(GetProcessHeap(), 0, Projects as LPVOID);
                    return (FALSE) as i64;
                }
                if ProjectsCount < ProjectsSize - 1 {
                    break;
                }
                HeapFree(GetProcessHeap(), 0, Projects as LPVOID);
                ProjectsSize *= 2;
            }

            CurrentProject = Projects;
            while *CurrentProject != 0u16 {
                End = wcschr(CurrentProject, '\n' as u16);
                if !End.is_null() {
                    *End = 0u16;
                }
                ListBox_AddString!(
                    GetDlgItem(hwndDlg, (IDC_PROJECTS) as i32),
                    CurrentProject,
                );
                if !End.is_null() {
                    CurrentProject = End.offset(1 as isize);
                } else {
                    CurrentProject = CurrentProject.offset(wcslen(CurrentProject) as isize);
                }
            }
            HeapFree(GetProcessHeap(), 0, Projects as LPVOID);
        }

        WM_COMMAND => {
            if HIWORD!(wParam as DWORD) == (BN_CLICKED) as u16
                && IDC_VIEWGPL == LOWORD!(wParam as DWORD) as u32
            {
                DialogBox!(
                    hDllInstance,
                    MAKEINTRESOURCE!(IDD_GPL),
                    core::ptr::null_mut(),
                    Some(GplDlgProc),
                );
                SetForegroundWindow(GetParent(hwndDlg));
            }
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(
                        GetParent(hwndDlg),
                        PSWIZB_BACK | PSWIZB_NEXT,
                    );
                    if (*pSetupData).UnattendSetup != 0 {
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((IDD_PRODUCT)) as LONG_PTR);
                        return (TRUE) as i64;
                    }
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as i64;
}
unsafe extern "C" fn DoWriteSoundEvents(
    hKey: HKEY,
    lpSubkey: LPCWSTR,
    lpEventsArray: *const *const [LPCWSTR; 2],
    dwSize: DWORD,
) -> BOOL {
    let mut hRootKey: HKEY = core::ptr::null_mut();
    let mut hEventKey: HKEY = core::ptr::null_mut();
    let mut hDefaultKey: HKEY = core::ptr::null_mut();
    let mut error: LONG;
    let mut i: ULONG;
    let mut szDest: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut dwAttribs: DWORD;
    let mut cbData: DWORD;

    error = RegOpenKeyExW(hKey, lpSubkey, 0, KEY_READ, &mut hRootKey);
    if error != 0 {
        // DPRINT removed
        'Error: {
            if !hRootKey.is_null() {
                RegCloseKey(hRootKey);
            }
            return (error == (ERROR_SUCCESS) as i32) as BOOL;
        }
    }

    i = 0;
    while i < dwSize {
        if ExpandEnvironmentStringsW(
            (*(*lpEventsArray.add(i as usize)))[1],
            szDest.as_mut_ptr(),
            _countof!(szDest),
        ) == 0
        {
            i += 1;
            continue;
        }

        dwAttribs = GetFileAttributesW(szDest.as_ptr());
        if (dwAttribs == INVALID_FILE_ATTRIBUTES) || (dwAttribs & FILE_ATTRIBUTE_DIRECTORY != 0) {
            i += 1;
            continue;
        }

        error = RegOpenKeyExW(
            hRootKey,
            (*(*lpEventsArray.add(i as usize)))[0],
            0,
            KEY_READ,
            &mut hEventKey,
        );
        if error != 0 {
            i += 1;
            continue;
        }

        error = RegOpenKeyExW(
            hEventKey,
            w!(".Default") as LPCWSTR,
            0,
            KEY_WRITE,
            &mut hDefaultKey,
        );
        RegCloseKey(hEventKey);
        if error != 0 {
            i += 1;
            continue;
        }

        cbData = ((lstrlenW((*(*lpEventsArray.add(i as usize)))[1]) + 1) * (core::mem::size_of::<WCHAR>() as DWORD) as i32) as u32;
        error = RegSetValueExW(
            hDefaultKey,
            core::ptr::null(),
            0,
            (REG_EXPAND_SZ) as u32,
            (*(*lpEventsArray.add(i as usize)))[1] as *const BYTE,
            cbData,
        );
        RegCloseKey(hDefaultKey);
        if error != 0 {
            i += 1;
            continue;
        }

        i += 1;
    }

    if !hRootKey.is_null() {
        RegCloseKey(hRootKey);
    }

    (error == (ERROR_SUCCESS) as i32) as BOOL
}
unsafe extern "C" fn DoWriteProductOption(nOption: PRODUCT_OPTION) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LONG;
    let mut pszData: LPCWSTR;
    let mut dwValue: DWORD;
    let mut cbData: DWORD;
    let pData: *const PRODUCT_OPTION_DATA = &s_ProductOptionData[nOption as usize];

    'Error: {
        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szProductOptions.as_ptr(), 0, KEY_WRITE, &mut hKey);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        pszData = (*pData).ProductSuite;
        cbData = ((lstrlenW(pszData) + 2) * (core::mem::size_of::<WCHAR>() as u32) as i32) as u32;
        error = RegSetValueExW(hKey, w!("ProductSuite") as LPCWSTR, 0, (REG_MULTI_SZ) as u32, pszData as *const BYTE, cbData);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        pszData = (*pData).ProductType;
        cbData = ((lstrlenW(pszData) + 1) * (core::mem::size_of::<WCHAR>() as u32) as i32) as u32;
        error = RegSetValueExW(hKey, w!("ProductType") as LPCWSTR, 0, (REG_SZ) as u32, pszData as *const BYTE, cbData);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        RegCloseKey(hKey);

        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szRosVersion.as_ptr(), 0, KEY_WRITE, &mut hKey);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        dwValue = (*pData).ReportAsWorkstation;
        cbData = core::mem::size_of::<DWORD>() as u32;
        error = RegSetValueExW(hKey, w!("ReportAsWorkstation") as LPCWSTR, 0, (REG_DWORD) as u32, &dwValue as *const DWORD as *const BYTE, cbData);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        RegCloseKey(hKey);

        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szControlWindows.as_ptr(), 0, KEY_WRITE, &mut hKey);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        dwValue = (*pData).CSDVersion;
        cbData = core::mem::size_of::<DWORD>() as u32;
        error = RegSetValueExW(hKey, w!("CSDVersion") as LPCWSTR, 0, (REG_DWORD) as u32, &dwValue as *const DWORD as *const BYTE, cbData);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        RegCloseKey(hKey);

        error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szWinlogon.as_ptr(), 0, KEY_WRITE, &mut hKey);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        dwValue = (*pData).LogonType;
        cbData = core::mem::size_of::<DWORD>() as u32;
        error = RegSetValueExW(hKey, w!("LogonType") as LPCWSTR, 0, (REG_DWORD) as u32, &dwValue as *const DWORD as *const BYTE, cbData);
        if error != 0 {
            // DPRINT removed
            break 'Error;
        }

        if nOption == PRODUCT_OPTION_WORKSTATION {
            DoWriteSoundEvents(HKEY_CURRENT_USER, s_szDefaultSoundEvents.as_ptr(), s_DefaultSoundEvents.as_ptr() as *const *const [LPCWSTR; 2], _countof!(s_DefaultSoundEvents));
            DoWriteSoundEvents(HKEY_CURRENT_USER, s_szExplorerSoundEvents.as_ptr(), s_ExplorerSoundEvents.as_ptr() as *const *const [LPCWSTR; 2], _countof!(s_ExplorerSoundEvents));
        }
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    (error == (ERROR_SUCCESS) as i32) as BOOL
}
unsafe extern "C" fn OnChooseOption(hwndDlg: HWND, nOption: PRODUCT_OPTION) {
    let mut szText: [WCHAR; 256] = [0; 256];

    match nOption {
        PRODUCT_OPTION_SERVER => {
            LoadStringW(hDllInstance, (IDS_PRODUCTSERVERINFO) as u32, szText.as_mut_ptr(), (_countof!(szText)) as i32);
        }

        PRODUCT_OPTION_WORKSTATION => {
            LoadStringW(hDllInstance, (IDS_PRODUCTWORKSTATIONINFO) as u32, szText.as_mut_ptr(), (_countof!(szText)) as i32);
        }

        _ => {
            return;
        }
    }

    SetDlgItemTextW(hwndDlg, (IDC_PRODUCT_DESCRIPTION) as i32, szText.as_ptr());
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn ProductPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut lpnm: LPNMHDR;
    let mut pSetupData: PSETUPDATA;
    let mut iItem: INT;
    let mut szText: [WCHAR; 64] = [0; 64];
    let mut szDefault: [WCHAR; 64] = [0; 64];
    let mut hIcon: HICON;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            pSetupData = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            LoadStringW(hDllInstance, IDS_DEFAULT, szDefault.as_mut_ptr(), (_countof!(szDefault)) as i32);

            LoadStringW(hDllInstance, IDS_PRODUCTSERVERNAME, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            if PRODUCT_OPTION_DEFAULT == PRODUCT_OPTION_SERVER {
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, w!(" ") as LPCWSTR);
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, szDefault.as_ptr());
            }
            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, CB_ADDSTRING, 0, szText.as_ptr() as LPARAM);

            LoadStringW(hDllInstance, IDS_PRODUCTWORKSTATIONNAME, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            if PRODUCT_OPTION_DEFAULT == PRODUCT_OPTION_WORKSTATION {
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, w!(" ") as LPCWSTR);
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, szDefault.as_ptr());
            }
            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, CB_ADDSTRING, 0, szText.as_ptr() as LPARAM);

            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, CB_SETCURSEL, PRODUCT_OPTION_DEFAULT, 0);
            OnChooseOption(hwndDlg, PRODUCT_OPTION_DEFAULT);

            hIcon = LoadIcon(core::ptr::null_mut(), IDI_WINLOGO);
            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_ICON) as i32, STM_SETICON, hIcon as WPARAM, 0);
            return TRUE;
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 && IDC_PRODUCT_OPTIONS == LOWORD!(wParam) as u32 {
                iItem = SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, CB_GETCURSEL, 0, 0);
                OnChooseOption(hwndDlg, iItem as PRODUCT_OPTION);
            }
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 {
                        OnChooseOption(hwndDlg, (*pSetupData).ProductOption);
                        DoWriteProductOption((*pSetupData).ProductOption);
                        SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, (IDD_LOCALEPAGE) as LONG_PTR);
                        return TRUE;
                    }
                }

                PSN_WIZNEXT => {
                    iItem = SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, CB_GETCURSEL, 0, 0);
                    (*pSetupData).ProductOption = iItem as PRODUCT_OPTION;
                    DoWriteProductOption((*pSetupData).ProductOption);
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn ProductPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut lpnm: LPNMHDR;
    let mut pSetupData: PSETUPDATA;
    let mut iItem: INT;
    let mut szText: [WCHAR; 64] = [0; 64];
    let mut szDefault: [WCHAR; 64] = [0; 64];
    let mut hIcon: HICON;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            pSetupData = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            LoadStringW(hDllInstance, (IDS_DEFAULT) as u32, szDefault.as_mut_ptr(), (_countof!(szDefault)) as i32);

            LoadStringW(hDllInstance, (IDS_PRODUCTSERVERNAME) as u32, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            if PRODUCT_OPTION_DEFAULT == PRODUCT_OPTION_SERVER {
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, w!(" ") as LPCWSTR);
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, szDefault.as_ptr());
            }
            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, (CB_ADDSTRING) as u32, 0, szText.as_ptr() as LPARAM);

            LoadStringW(hDllInstance, (IDS_PRODUCTWORKSTATIONNAME) as u32, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            if PRODUCT_OPTION_DEFAULT == PRODUCT_OPTION_WORKSTATION {
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, w!(" ") as LPCWSTR);
                StringCchCatW(szText.as_mut_ptr(), (_countof!(szText)) as usize, szDefault.as_ptr());
            }
            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, (CB_ADDSTRING) as u32, 0, szText.as_ptr() as LPARAM);

            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, CB_SETCURSEL, (PRODUCT_OPTION_DEFAULT) as u64, 0);
            OnChooseOption(hwndDlg, PRODUCT_OPTION_DEFAULT);

            hIcon = LoadIcon(core::ptr::null_mut(), IDI_WINLOGO);
            SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_ICON) as i32, (STM_SETICON) as u32, hIcon as WPARAM, 0);
            return (TRUE) as i64;
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 && IDC_PRODUCT_OPTIONS == LOWORD!(wParam) as u32 {
                iItem = (SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, (CB_GETCURSEL) as u32, 0, 0)) as i32;
                OnChooseOption(hwndDlg, iItem as PRODUCT_OPTION);
            }
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 {
                        OnChooseOption(hwndDlg, (*pSetupData).ProductOption);
                        DoWriteProductOption((*pSetupData).ProductOption);
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((IDD_LOCALEPAGE)) as LONG_PTR);
                        return (TRUE) as i64;
                    }
                }

                PSN_WIZNEXT => {
                    iItem = (SendDlgItemMessageW(hwndDlg, (IDC_PRODUCT_OPTIONS) as i32, (CB_GETCURSEL) as u32, 0, 0)) as i32;
                    (*pSetupData).ProductOption = iItem as PRODUCT_OPTION;
                    DoWriteProductOption((*pSetupData).ProductOption);
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as i64;
}
unsafe extern "C" fn WriteOwnerSettings(
    OwnerName: *mut WCHAR,
    OwnerOrganization: *mut WCHAR,
) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut res: LONG;

    res = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("Software\\Microsoft\\Windows NT\\CurrentVersion") as LPCWSTR,
        0,
        KEY_ALL_ACCESS,
        &mut hKey,
    );

    if res != (ERROR_SUCCESS) as i32 {
        return (FALSE) as i32;
    }

    res = RegSetValueExW(
        hKey,
        w!("RegisteredOwner") as LPCWSTR,
        0,
        (REG_SZ) as u32,
        OwnerName as LPBYTE,
        ((wcslen(OwnerName) + 1) * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
    );

    if res != (ERROR_SUCCESS) as i32 {
        RegCloseKey(hKey);
        return (FALSE) as i32;
    }

    res = RegSetValueExW(
        hKey,
        w!("RegisteredOrganization") as LPCWSTR,
        0,
        (REG_SZ) as u32,
        OwnerOrganization as LPBYTE,
        ((wcslen(OwnerOrganization) + 1) * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
    );

    RegCloseKey(hKey);
    return (res == (ERROR_SUCCESS) as i32) as BOOL;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn OwnerPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut OwnerName: [WCHAR; 51] = [0; 51];
    let mut OwnerOrganization: [WCHAR; 51] = [0; 51];
    let mut Title: [WCHAR; 64] = [0; 64];
    let mut ErrorName: [WCHAR; 256] = [0; 256];
    let lpnm: LPNMHDR;
    let mut pSetupData: PSETUPDATA;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            pSetupData = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            if LoadStringW(hDllInstance, IDS_MACHINE_OWNER_NAME, OwnerName.as_mut_ptr(), (_countof!(OwnerName)) as i32) != 0 {
                SendDlgItemMessage(hwndDlg, (IDC_OWNERNAME) as i32, WM_SETTEXT, 0, OwnerName.as_ptr() as LPARAM);
            }

            SendDlgItemMessage(hwndDlg, (IDC_OWNERNAME) as i32, EM_LIMITTEXT, 50, 0);
            SendDlgItemMessage(hwndDlg, (IDC_OWNERORGANIZATION) as i32, EM_LIMITTEXT, 50, 0);

            SetFocus(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32));

            SendDlgItemMessage(hwndDlg, (IDC_OWNERNAME) as i32, EM_SETSEL, 0, (-1) as LPARAM);
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 {
                        SendMessage(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32), WM_SETTEXT, 0, (*pSetupData).OwnerName.as_ptr() as LPARAM);
                        SendMessage(GetDlgItem(hwndDlg, (IDC_OWNERORGANIZATION) as i32), WM_SETTEXT, 0, (*pSetupData).OwnerOrganization.as_ptr() as LPARAM);
                        if WriteOwnerSettings((*pSetupData).OwnerName.as_mut_ptr(), (*pSetupData).OwnerOrganization.as_mut_ptr()) != 0 {
                            SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, (IDD_COMPUTERPAGE) as LONG_PTR);
                            return TRUE;
                        }
                    }
                }

                PSN_WIZNEXT => {
                    OwnerName[0] = 0;
                    if GetDlgItemTextW(hwndDlg, (IDC_OWNERNAME) as i32, OwnerName.as_mut_ptr(), 50) == 0 {
                        if 0 == LoadStringW(hDllInstance, IDS_REACTOS_SETUP, Title.as_mut_ptr(), (ARRAYSIZE!(Title)) as i32) {
                            wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as PCWSTR);
                        }
                        if 0 == LoadStringW(hDllInstance, IDS_WZD_NAME, ErrorName.as_mut_ptr(), (ARRAYSIZE!(ErrorName)) as i32) {
                            wcscpy(ErrorName.as_mut_ptr(), w!("Setup cannot continue until you enter your name.") as PCWSTR);
                        }
                        MessageBoxW(hwndDlg, ErrorName.as_ptr(), Title.as_ptr(), MB_ICONERROR | MB_OK);

                        SetFocus(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((-1)) as LONG_PTR);

                        return TRUE;
                    }

                    OwnerOrganization[0] = 0;
                    GetDlgItemTextW(hwndDlg, (IDC_OWNERORGANIZATION) as i32, OwnerOrganization.as_mut_ptr(), 50);

                    if WriteOwnerSettings(OwnerName.as_mut_ptr(), OwnerOrganization.as_mut_ptr()) == 0 {
                        SetFocus(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((-1)) as LONG_PTR);
                        return TRUE;
                    }
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn OwnerPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut OwnerName: [WCHAR; 51] = [0; 51];
    let mut OwnerOrganization: [WCHAR; 51] = [0; 51];
    let mut Title: [WCHAR; 64] = [0; 64];
    let mut ErrorName: [WCHAR; 256] = [0; 256];
    let lpnm: LPNMHDR;
    let mut pSetupData: PSETUPDATA;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            pSetupData = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            if LoadStringW(hDllInstance, (IDS_MACHINE_OWNER_NAME) as u32, OwnerName.as_mut_ptr(), (_countof!(OwnerName)) as i32) != 0 {
                SendDlgItemMessage(hwndDlg, (IDC_OWNERNAME) as i32, (WM_SETTEXT) as u32, 0, OwnerName.as_ptr() as LPARAM);
            }

            SendDlgItemMessage(hwndDlg, (IDC_OWNERNAME) as i32, (EM_LIMITTEXT) as u32, 50, 0);
            SendDlgItemMessage(hwndDlg, (IDC_OWNERORGANIZATION) as i32, (EM_LIMITTEXT) as u32, 50, 0);

            SetFocus(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32));

            SendDlgItemMessage(hwndDlg, (IDC_OWNERNAME) as i32, (EM_SETSEL) as u32, 0, (-1) as LPARAM);
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 {
                        SendMessage(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32), (WM_SETTEXT) as u32, 0, (*pSetupData).OwnerName.as_ptr() as LPARAM);
                        SendMessage(GetDlgItem(hwndDlg, (IDC_OWNERORGANIZATION) as i32), (WM_SETTEXT) as u32, 0, (*pSetupData).OwnerOrganization.as_ptr() as LPARAM);
                        if WriteOwnerSettings((*pSetupData).OwnerName.as_mut_ptr(), (*pSetupData).OwnerOrganization.as_mut_ptr()) != 0 {
                            SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((IDD_COMPUTERPAGE)) as LONG_PTR);
                            return (TRUE) as i64;
                        }
                    }
                }

                PSN_WIZNEXT => {
                    OwnerName[0] = 0;
                    if GetDlgItemTextW(hwndDlg, (IDC_OWNERNAME) as i32, OwnerName.as_mut_ptr(), 50) == 0 {
                        if 0 == LoadStringW(hDllInstance, (IDS_REACTOS_SETUP) as u32, Title.as_mut_ptr(), (ARRAYSIZE!(Title)) as i32) {
                            wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as PCWSTR);
                        }
                        if 0 == LoadStringW(hDllInstance, (IDS_WZD_NAME) as u32, ErrorName.as_mut_ptr(), (ARRAYSIZE!(ErrorName)) as i32) {
                            wcscpy(ErrorName.as_mut_ptr(), w!("Setup cannot continue until you enter your name.") as PCWSTR);
                        }
                        MessageBoxW(hwndDlg, ErrorName.as_ptr(), Title.as_ptr(), (MB_ICONERROR | MB_OK) as u32);

                        SetFocus(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((-1)) as LONG_PTR);

                        return (TRUE) as i64;
                    }

                    OwnerOrganization[0] = 0;
                    GetDlgItemTextW(hwndDlg, (IDC_OWNERORGANIZATION) as i32, OwnerOrganization.as_mut_ptr(), 50);

                    if WriteOwnerSettings(OwnerName.as_mut_ptr(), OwnerOrganization.as_mut_ptr()) == 0 {
                        SetFocus(GetDlgItem(hwndDlg, (IDC_OWNERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((-1)) as LONG_PTR);
                        return (TRUE) as i64;
                    }
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as i64
}
unsafe extern "C" fn WriteComputerSettings(ComputerName: PWSTR, hwndDlg: HWND) -> BOOL {
    let mut Title: [WCHAR; 64] = [0; 64];
    let mut ErrorComputerName: [WCHAR; 256] = [0; 256];
    let mut lError: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();

    if SetComputerNameW(ComputerName) == 0 {
        if hwndDlg != core::ptr::null_mut() {
            if 0 == LoadStringW(hDllInstance, (IDS_REACTOS_SETUP) as u32, Title.as_mut_ptr(), (ARRAYSIZE!(Title)) as i32) {
                wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as PWSTR);
            }
            if 0 == LoadStringW(hDllInstance, (IDS_WZD_SETCOMPUTERNAME) as u32, ErrorComputerName.as_mut_ptr(), (ARRAYSIZE!(ErrorComputerName)) as i32) {
                wcscpy(ErrorComputerName.as_mut_ptr(), w!("Setup failed to set the computer name.") as PWSTR);
            }
            MessageBoxW(hwndDlg, ErrorComputerName.as_ptr(), Title.as_ptr(), (MB_ICONERROR | MB_OK) as u32);
        }

        return (FALSE) as i32;
    }

    SetComputerNameExW(ComputerNamePhysicalDnsDomain, w!("") as PCWSTR);

    SetComputerNameExW(ComputerNamePhysicalDnsHostname, ComputerName);

    SetAccountsDomainSid(core::ptr::null_mut(), ComputerName);

    lError = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
                             w!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters") as LPCWSTR,
                             0,
                             core::ptr::null_mut(),
                             REG_OPTION_NON_VOLATILE,
                             KEY_WRITE,
                             core::ptr::null_mut(),
                             &mut hKey,
                             core::ptr::null_mut());
    if lError == (ERROR_SUCCESS) as i32 {
        lError = RegSetValueEx(hKey,
                               w!("Hostname") as LPCWSTR,
                               0,
                               (REG_SZ) as u32,
                               ComputerName as LPBYTE,
                               ((wcslen(ComputerName) + 1) * (core::mem::size_of::<WCHAR>() as DWORD) as usize) as u32);
        if lError != (ERROR_SUCCESS) as i32 {
            // DPRINT removed
        }

        RegCloseKey(hKey);
    } else {
        // DPRINT removed
    }

    (TRUE) as i32
}
unsafe extern "C" fn WriteDefaultLogonData(Domain: LPWSTR) -> BOOL {
    let mut szAdministratorName: [u16; 256] = core::mem::zeroed();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG;

    if LoadStringW(
        hDllInstance,
        (IDS_ADMINISTRATOR_NAME) as u32,
        szAdministratorName.as_mut_ptr(),
        (ARRAYSIZE!(szAdministratorName)) as i32,
    ) == 0
    {
        wcscpy(szAdministratorName.as_mut_ptr(), w!("Administrator") as *const u16);
    }

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon") as *const u16,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    );
    if lError != (ERROR_SUCCESS) as i32 {
        return (FALSE) as i32;
    }

    lError = RegSetValueEx(
        hKey,
        w!("DefaultDomainName") as *const u16,
        0,
        (REG_SZ) as u32,
        Domain as LPBYTE,
        ((wcslen(Domain) + 1) * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
    );
    if lError != (ERROR_SUCCESS) as i32 {
        // DPRINT removed
    }

    lError = RegSetValueEx(
        hKey,
        w!("DefaultUserName") as *const u16,
        0,
        (REG_SZ) as u32,
        szAdministratorName.as_ptr() as LPBYTE,
        ((wcslen(szAdministratorName.as_ptr()) + 1) * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
    );
    if lError != (ERROR_SUCCESS) as i32 {
        // DPRINT removed
    }

    RegCloseKey(hKey);

    return (TRUE) as i32;
}
unsafe extern "C" fn GenerateComputerName(lpBuffer: LPWSTR) {
    static Chars: &[u16] = &[
        '0' as u16, '1' as u16, '2' as u16, '3' as u16, '4' as u16, '5' as u16, '6' as u16, '7' as u16, '8' as u16, '9' as u16,
        'A' as u16, 'B' as u16, 'C' as u16, 'D' as u16, 'E' as u16, 'F' as u16, 'G' as u16, 'H' as u16, 'I' as u16, 'J' as u16,
        'K' as u16, 'L' as u16, 'M' as u16, 'N' as u16, 'O' as u16, 'P' as u16, 'Q' as u16, 'R' as u16, 'S' as u16, 'T' as u16,
        'U' as u16, 'V' as u16, 'W' as u16, 'X' as u16, 'Y' as u16, 'Z' as u16, 0
    ];
    let cChars: u32 = (core::mem::size_of_val(Chars) / core::mem::size_of::<u16>() - 1) as u32;
    let mut i: u32;

    wcscpy(lpBuffer, w!("REACTOS-"));

    srand(GetTickCount());

    i = 8;
    while i < 15 {
        *lpBuffer.offset(i as isize) = Chars[(rand() % (cChars) as i32) as usize];
        i += 1;
    }

    *lpBuffer.offset(15 as isize) = UNICODE_NULL;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn ComputerPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut ComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] = [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
    let mut Password1: [WCHAR; 128] = [0; 128];
    let mut Password2: [WCHAR; 128] = [0; 128];
    let mut Password: PWCHAR;
    let mut Title: [WCHAR; 64] = [0; 64];
    let mut EmptyComputerName: [WCHAR; 256] = [0; 256];
    let mut NotMatchPassword: [WCHAR; 256] = [0; 256];
    let mut WrongPassword: [WCHAR; 256] = [0; 256];
    let lpnm: LPNMHDR;
    let mut pSetupData: PSETUPDATA;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    if 0 == LoadStringW(hDllInstance, IDS_REACTOS_SETUP, Title.as_mut_ptr(), (ARRAYSIZE!(Title)) as i32) {
        wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as PWSTR);
    }

    match uMsg {
        WM_INITDIALOG => {
            pSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            GenerateComputerName(ComputerName.as_mut_ptr());

            SetDlgItemTextW(hwndDlg, (IDC_COMPUTERNAME) as i32, ComputerName.as_ptr());

            SendDlgItemMessage(hwndDlg, (IDC_COMPUTERNAME) as i32, EM_LIMITTEXT, MAX_COMPUTERNAME_LENGTH, 0);
            SendDlgItemMessage(hwndDlg, (IDC_ADMINPASSWORD1) as i32, EM_LIMITTEXT, 127, 0);
            SendDlgItemMessage(hwndDlg, (IDC_ADMINPASSWORD2) as i32, EM_LIMITTEXT, 127, 0);

            SetFocus(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32));
            if (*pSetupData).UnattendSetup != 0 {
                SendMessage(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32), WM_SETTEXT, 0, (*pSetupData).ComputerName.as_ptr() as LPARAM);
                SendMessage(GetDlgItem(hwndDlg, (IDC_ADMINPASSWORD1) as i32), WM_SETTEXT, 0, (*pSetupData).AdminPassword.as_ptr() as LPARAM);
                SendMessage(GetDlgItem(hwndDlg, (IDC_ADMINPASSWORD2) as i32), WM_SETTEXT, 0, (*pSetupData).AdminPassword.as_ptr() as LPARAM);
                WriteComputerSettings((*pSetupData).ComputerName.as_mut_ptr(), core::ptr::null_mut());
                SetAdministratorPassword((*pSetupData).AdminPassword.as_ptr());
            }

            WriteDefaultLogonData((*pSetupData).ComputerName.as_mut_ptr());
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 && WriteComputerSettings((*pSetupData).ComputerName.as_mut_ptr(), hwndDlg) != 0 {
                        SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, (IDD_DATETIMEPAGE) as LONG_PTR);
                        return TRUE;
                    }
                }

                PSN_WIZNEXT => {
                    if 0 == GetDlgItemTextW(hwndDlg, IDC_COMPUTERNAME as i32, ComputerName.as_mut_ptr(), (MAX_COMPUTERNAME_LENGTH + 1) as i32) {
                        if 0 == LoadStringW(hDllInstance, IDS_WZD_COMPUTERNAME as u32, EmptyComputerName.as_mut_ptr(), (ARRAYSIZE!(EmptyComputerName)) as i32) {
                            wcscpy(EmptyComputerName.as_mut_ptr(), w!("Setup cannot continue until you enter the name of your computer.") as PWSTR);
                        }
                        MessageBoxW(hwndDlg, EmptyComputerName.as_ptr(), Title.as_ptr(), MB_ICONERROR | MB_OK);
                        SetFocus(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                        return TRUE;
                    }

                    if WriteComputerSettings(ComputerName.as_mut_ptr(), hwndDlg) == 0 {
                        SetFocus(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                        return TRUE;
                    }

                    GetDlgItemTextW(hwndDlg, (IDC_ADMINPASSWORD1) as i32, Password1.as_mut_ptr(), 128);
                    GetDlgItemTextW(hwndDlg, (IDC_ADMINPASSWORD2) as i32, Password2.as_mut_ptr(), 128);

                    if wcscmp(Password1.as_ptr(), Password2.as_ptr()) != 0 {
                        if 0 == LoadStringW(hDllInstance, IDS_WZD_PASSWORDMATCH, NotMatchPassword.as_mut_ptr(), (ARRAYSIZE!(NotMatchPassword)) as i32) {
                            wcscpy(NotMatchPassword.as_mut_ptr(), w!("The passwords you entered do not match. Please enter the desired password again.") as PWSTR);
                        }
                        MessageBoxW(hwndDlg, NotMatchPassword.as_ptr(), Title.as_ptr(), MB_ICONERROR | MB_OK);
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                        return TRUE;
                    }

                    Password = Password1.as_mut_ptr();
                    while *Password != 0 {
                        if isprint(*Password as i32) == 0 {
                            if 0 == LoadStringW(hDllInstance, IDS_WZD_PASSWORDCHAR, WrongPassword.as_mut_ptr(), (ARRAYSIZE!(WrongPassword)) as i32) {
                                wcscpy(WrongPassword.as_mut_ptr(), w!("The password you entered contains invalid characters. Please enter a cleaned password.") as PWSTR);
                            }
                            MessageBoxW(hwndDlg, WrongPassword.as_ptr(), Title.as_ptr(), MB_ICONERROR | MB_OK);
                            SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                            return TRUE;
                        }
                        Password = Password.offset(1);
                    }

                    SetAdministratorPassword(Password1.as_mut_ptr());
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn ComputerPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut ComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] = [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
    let mut Password1: [WCHAR; 128] = [0; 128];
    let mut Password2: [WCHAR; 128] = [0; 128];
    let mut Password: PWCHAR;
    let mut Title: [WCHAR; 64] = [0; 64];
    let mut EmptyComputerName: [WCHAR; 256] = [0; 256];
    let mut NotMatchPassword: [WCHAR; 256] = [0; 256];
    let mut WrongPassword: [WCHAR; 256] = [0; 256];
    let lpnm: LPNMHDR;
    let mut pSetupData: PSETUPDATA;

    pSetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    if 0 == LoadStringW(hDllInstance, (IDS_REACTOS_SETUP) as u32, Title.as_mut_ptr(), (ARRAYSIZE!(Title)) as i32) {
        wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as PWSTR);
    }

    match uMsg {
        WM_INITDIALOG => {
            pSetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, (pSetupData) as LONG_PTR);

            GenerateComputerName(ComputerName.as_mut_ptr());

            SetDlgItemTextW(hwndDlg, (IDC_COMPUTERNAME) as i32, ComputerName.as_ptr());

            SendDlgItemMessage(hwndDlg, (IDC_COMPUTERNAME) as i32, (EM_LIMITTEXT) as u32, (MAX_COMPUTERNAME_LENGTH) as u64, 0);
            SendDlgItemMessage(hwndDlg, (IDC_ADMINPASSWORD1) as i32, (EM_LIMITTEXT) as u32, 127, 0);
            SendDlgItemMessage(hwndDlg, (IDC_ADMINPASSWORD2) as i32, (EM_LIMITTEXT) as u32, 127, 0);

            SetFocus(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32));
            if (*pSetupData).UnattendSetup != 0 {
                SendMessage(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32), (WM_SETTEXT) as u32, 0, (*pSetupData).ComputerName.as_ptr() as LPARAM);
                SendMessage(GetDlgItem(hwndDlg, (IDC_ADMINPASSWORD1) as i32), (WM_SETTEXT) as u32, 0, (*pSetupData).AdminPassword.as_ptr() as LPARAM);
                SendMessage(GetDlgItem(hwndDlg, (IDC_ADMINPASSWORD2) as i32), (WM_SETTEXT) as u32, 0, (*pSetupData).AdminPassword.as_ptr() as LPARAM);
                WriteComputerSettings((*pSetupData).ComputerName.as_mut_ptr(), core::ptr::null_mut());
                SetAdministratorPassword((*pSetupData).AdminPassword.as_ptr());
            }

            WriteDefaultLogonData((*pSetupData).ComputerName.as_mut_ptr());
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*pSetupData).UnattendSetup != 0 && WriteComputerSettings((*pSetupData).ComputerName.as_mut_ptr(), hwndDlg) != 0 {
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((IDD_DATETIMEPAGE)) as LONG_PTR);
                        return (TRUE) as i64;
                    }
                }

                PSN_WIZNEXT => {
                    if 0 == GetDlgItemTextW(hwndDlg, (IDC_COMPUTERNAME) as i32, ComputerName.as_mut_ptr(), (MAX_COMPUTERNAME_LENGTH + 1) as i32) {
                        if 0 == LoadStringW(hDllInstance, (IDS_WZD_COMPUTERNAME) as u32, EmptyComputerName.as_mut_ptr(), (ARRAYSIZE!(EmptyComputerName)) as i32) {
                            wcscpy(EmptyComputerName.as_mut_ptr(), w!("Setup cannot continue until you enter the name of your computer.") as PWSTR);
                        }
                        MessageBoxW(hwndDlg, EmptyComputerName.as_ptr(), Title.as_ptr(), (MB_ICONERROR | MB_OK) as u32);
                        SetFocus(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                        return (TRUE) as i64;
                    }

                    if WriteComputerSettings(ComputerName.as_mut_ptr(), hwndDlg) == 0 {
                        SetFocus(GetDlgItem(hwndDlg, (IDC_COMPUTERNAME) as i32));
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                        return (TRUE) as i64;
                    }

                    GetDlgItemTextW(hwndDlg, (IDC_ADMINPASSWORD1) as i32, Password1.as_mut_ptr(), 128);
                    GetDlgItemTextW(hwndDlg, (IDC_ADMINPASSWORD2) as i32, Password2.as_mut_ptr(), 128);

                    if wcscmp(Password1.as_ptr(), Password2.as_ptr()) != 0 {
                        if 0 == LoadStringW(hDllInstance, (IDS_WZD_PASSWORDMATCH) as u32, NotMatchPassword.as_mut_ptr(), (ARRAYSIZE!(NotMatchPassword)) as i32) {
                            wcscpy(NotMatchPassword.as_mut_ptr(), w!("The passwords you entered do not match. Please enter the desired password again.") as PWSTR);
                        }
                        MessageBoxW(hwndDlg, NotMatchPassword.as_ptr(), Title.as_ptr(), (MB_ICONERROR | MB_OK) as u32);
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                        return (TRUE) as i64;
                    }

                    Password = Password1.as_mut_ptr();
                    while *Password != 0 {
                        if isprint(*Password as i32) == 0 {
                            if 0 == LoadStringW(hDllInstance, (IDS_WZD_PASSWORDCHAR) as u32, WrongPassword.as_mut_ptr(), (ARRAYSIZE!(WrongPassword)) as i32) {
                                wcscpy(WrongPassword.as_mut_ptr(), w!("The password you entered contains invalid characters. Please enter a cleaned password.") as PWSTR);
                            }
                            MessageBoxW(hwndDlg, WrongPassword.as_ptr(), Title.as_ptr(), (MB_ICONERROR | MB_OK) as u32);
                            SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (-(1 as i32)) as LONG_PTR);
                            return (TRUE) as i64;
                        }
                        Password = Password.offset(1);
                    }

                    SetAdministratorPassword(Password1.as_mut_ptr());
                }

                PSN_WIZBACK => {
                    (*pSetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as i64
}
unsafe extern "C" fn SetUserLocaleName(hwnd: HWND) {
    let mut CurLocale: [WCHAR; 256] = [0; 256];
    let mut CurGeo: [WCHAR; 256] = [0; 256];
    let mut ResText: [WCHAR; 256] = [0; 256];
    let mut LocaleText: [WCHAR; 256 * 2] = [0; 256 * 2];

    GetLocaleInfoW(GetUserDefaultLCID(), (LOCALE_SLANGUAGE) as u32, CurLocale.as_mut_ptr(), (ARRAYSIZE!(CurLocale)) as i32);
    GetGeoInfoW(GetUserGeoID(GEOCLASS_NATION), GEO_FRIENDLYNAME, CurGeo.as_mut_ptr(), (ARRAYSIZE!(CurGeo)) as i32, (GetThreadLocale()) as u16);

    LoadStringW(hDllInstance, (IDS_LOCALETEXT) as u32, ResText.as_mut_ptr(), (ARRAYSIZE!(ResText)) as i32);
    StringCchPrintfW!(LocaleText.as_mut_ptr(), (ARRAYSIZE!(LocaleText)) as usize, ResText.as_ptr(), CurLocale.as_ptr(), CurGeo.as_ptr());

    SetWindowTextW(hwnd, LocaleText.as_ptr());
}
// Ported from upstream d482579dff3 ("[SYSSETUP] Fix retrieval of the currently active keyboard
// layout ID and its name", #9420). The earlier body guessed the KLID from the default input
// HKL (LOWORD as the layout, a 0xF000 HIWORD as a "Layout Id" to search for), which cannot map
// an HKL back to its KLID: with an en-AU locale and a U.S. or Dvorak layout it named the wrong
// layout or none. The KLID is now read from HKCU\Keyboard Layout\Preload\1, put through
// HKCU\Keyboard Layout\Substitutes, and only then looked up under Keyboard Layouts; when no
// name is found the KLID itself is shown.
unsafe extern "C" fn SetKeyboardLayoutName(hwnd: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut res: LONG;
    let mut dwSize: DWORD;
    let mut dwType: DWORD = 0;
    let mut szKLID: [WCHAR; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];
    let mut szLayoutId: [WCHAR; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];
    let mut LayoutName: [WCHAR; 128] = [0; 128];
    let mut LayoutPath: [WCHAR; 256] = [0; 256];
    let mut ResText: [WCHAR; 256] = [0; 256];

    // Retrieve the current keyboard layout ID; fall back to the U.S. layout if none was found.
    szKLID[0] = UNICODE_NULL;
    if RegOpenKeyExW(HKEY_CURRENT_USER, w!("Keyboard Layout\\Preload") as PCWSTR, 0, KEY_QUERY_VALUE, &mut hKey) == (ERROR_SUCCESS) as i32 {
        dwSize = core::mem::size_of_val(&szKLID) as DWORD;
        res = RegQueryValueExW(hKey, w!("1") as PCWSTR, core::ptr::null_mut(), &mut dwType, (szKLID.as_mut_ptr() as PVOID).cast(), &mut dwSize);
        if res != (ERROR_SUCCESS) as i32 || dwType != REG_SZ || dwSize != core::mem::size_of_val(&szKLID) as DWORD {
            szKLID[0] = UNICODE_NULL;
        }
        RegCloseKey(hKey);
    }
    if szKLID[0] == UNICODE_NULL {
        StringCchCopyW(szKLID.as_mut_ptr(), (ARRAYSIZE!(szKLID)) as usize, w!("00000409") as PCWSTR);
    }

    // If this is a substituted layout ID, replace it with the target ID.
    if RegOpenKeyExW(HKEY_CURRENT_USER, w!("Keyboard Layout\\Substitutes") as PCWSTR, 0, KEY_QUERY_VALUE, &mut hKey) == (ERROR_SUCCESS) as i32 {
        dwSize = core::mem::size_of_val(&szLayoutId) as DWORD;
        res = RegQueryValueExW(hKey, szKLID.as_ptr(), core::ptr::null_mut(), &mut dwType, (szLayoutId.as_mut_ptr() as PVOID).cast(), &mut dwSize);
        if res == (ERROR_SUCCESS) as i32 && dwType == REG_SZ && dwSize == core::mem::size_of_val(&szKLID) as DWORD {
            StringCchCopyW(szKLID.as_mut_ptr(), (ARRAYSIZE!(szKLID)) as usize, szLayoutId.as_ptr());
        }
        RegCloseKey(hKey);
    }

    LayoutName[0] = UNICODE_NULL;

    // Open the layout ID registry key and retrieve its human-readable name.
    StringCchPrintfW!(LayoutPath.as_mut_ptr(), (ARRAYSIZE!(LayoutPath)) as usize, w!("SYSTEM\\CurrentControlSet\\Control\\Keyboard Layouts\\%s") as PCWSTR, szKLID.as_ptr());
    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, LayoutPath.as_ptr(), 0, KEY_QUERY_VALUE, &mut hKey) == (ERROR_SUCCESS) as i32 {
        dwSize = core::mem::size_of_val(&LayoutName) as DWORD;
        res = RegQueryValueExW(hKey, w!("Layout Text") as PCWSTR, core::ptr::null_mut(), &mut dwType, (LayoutName.as_mut_ptr() as PVOID).cast(), &mut dwSize);
        if res != (ERROR_SUCCESS) as i32 || dwType != REG_SZ {
            LayoutName[0] = UNICODE_NULL;
        }
        RegCloseKey(hKey);
    }

    // If no layout name was found, just display the current layout ID.
    if LayoutName[0] == UNICODE_NULL {
        StringCchCopyW(LayoutName.as_mut_ptr(), (ARRAYSIZE!(LayoutName)) as usize, szKLID.as_ptr());
    }

    LoadStringW(hDllInstance, (IDS_LAYOUTTEXT) as u32, ResText.as_mut_ptr(), (ARRAYSIZE!(ResText)) as i32);
    StringCchPrintfW!(LayoutPath.as_mut_ptr(), (ARRAYSIZE!(LayoutPath)) as usize, ResText.as_ptr(), LayoutName.as_ptr());

    SetWindowTextW(hwnd, LayoutPath.as_ptr());
}
unsafe extern "C" fn RunControlPanelApplet(hwnd: HWND, pwszCPLParameters: PCWSTR) -> BOOL {
    let mut msg: MSG = core::mem::zeroed();
    let MainWindow = GetParent(hwnd);
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let mut CmdLine: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    let prefix = b"r\0u\0n\0d\0l\0l\x003\x002\0.\0e\0x\0e\0 \0s\0h\0e\0l\0l\x003\x002\0.\0d\0l\0l\0,\0C\0o\0n\0t\0r\0o\0l\0_\0R\0u\0n\0D\0L\0L\0 \0\0\0";
    core::ptr::copy_nonoverlapping(prefix.as_ptr() as *const WCHAR, CmdLine.as_mut_ptr(), 41);

    if pwszCPLParameters.is_null() {
        MessageBoxW(hwnd, w!("Error: Failed to launch the Control Panel Applet.") as LPCWSTR, core::ptr::null(), (MB_ICONERROR) as u32);
        return (FALSE) as i32;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(StartupInfo) as PVOID, core::mem::size_of::<STARTUPINFOW>());
    StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
    ZeroMemory!(core::ptr::addr_of_mut!(ProcessInformation) as PVOID, core::mem::size_of::<PROCESS_INFORMATION>());

    wcscat(CmdLine.as_mut_ptr(), pwszCPLParameters);

    if CreateProcessW(
        core::ptr::null_mut(),
        CmdLine.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as i32,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(StartupInfo),
        core::ptr::addr_of_mut!(ProcessInformation)
    ) == 0 {
        MessageBoxW(hwnd, w!("Error: Failed to launch the Control Panel Applet.") as LPCWSTR, core::ptr::null(), (MB_ICONERROR) as u32);
        return (FALSE) as i32;
    }

    PropSheet_SetWizButtons!(MainWindow, 0);
    EnableWindow(MainWindow, (FALSE) as i32);

    while MsgWaitForMultipleObjects(1, core::ptr::addr_of_mut!(ProcessInformation.hProcess), (FALSE) as i32, (INFINITE) as u32, (QS_ALLINPUT | QS_ALLPOSTMESSAGE) as u32) != (WAIT_OBJECT_0) as u32 {
        while PeekMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0, (PM_REMOVE) as u32) != 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }
    CloseHandle(ProcessInformation.hThread);
    CloseHandle(ProcessInformation.hProcess);

    PropSheet_SetWizButtons!(MainWindow, PSWIZB_BACK | PSWIZB_NEXT);
    EnableWindow(MainWindow, (TRUE) as i32);

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn EnableVisualTheme(hwndParent: HWND, ThemeFile: PCWSTR) {
    // wizard.c:1375 is `enum { THEME_FILE, STYLE_FILE, UNKNOWN } fType;` — an ANONYMOUS enum
    // declaring three constants and one variable. The translation inverted it into a named enum
    // whose VARIANTS were initialised from the (undefined) constants, which is the opposite
    // shape and does not compile. Three consts, values in C declaration order.
    const THEME_FILE: i32 = 0;
    const STYLE_FILE: i32 = 1;
    const UNKNOWN: i32 = 2; // nolint: const-contradiction (wizard.c's local anonymous enum)


    let mut szPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szStyleFile: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    let mut fType = THEME_FILE as i32;
    let mut ThemeFile = ThemeFile;

    if !ThemeFile.is_null() {
        if ExpandEnvironmentStringsW(ThemeFile, szPath.as_mut_ptr(), _countof!(szPath)) != 0 {
            ThemeFile = szPath.as_ptr();
        }

        fType = UNKNOWN;
        {
            let pszExt = wcsrchr(ThemeFile, 0x002E);
            if !pszExt.is_null() {
                if _wcsicmp(pszExt, w!(".theme") as PCWSTR) == 0 {
                    fType = THEME_FILE;
                } else if _wcsicmp(pszExt, w!(".msstyles") as PCWSTR) == 0 {
                    fType = STYLE_FILE;
                }
            }
        }

        if fType == UNKNOWN {
            // DPRINT removed
            return;
        }
    }

    // DPRINT removed

    if !ThemeFile.is_null() && fType == THEME_FILE {
        if GetPrivateProfileStringW(
            w!("VisualStyles") as PCWSTR,
            w!("Path") as PCWSTR,
            core::ptr::null(),
            szStyleFile.as_mut_ptr(),
            _countof!(szStyleFile),
            ThemeFile
        ) != 0 && *szStyleFile.as_ptr() != 0
        {
            ThemeFile = szStyleFile.as_ptr();
            if ExpandEnvironmentStringsW(ThemeFile, szPath.as_mut_ptr(), _countof!(szPath)) != 0 {
                ThemeFile = szPath.as_ptr();
            }
        } else {
            ThemeFile = core::ptr::null();
        }

        // DPRINT removed
    }

    if !ThemeFile.is_null() {
        let mut wszParams: [u16; 1024] = [0; 1024];
        let format = w!("desk.cpl,,2 /Action:ActivateMSTheme /file:\"%s\"") as PCWSTR;

        StringCchPrintfW!(wszParams.as_mut_ptr(), (_countof!(wszParams)) as usize, format, ThemeFile);
        RunControlPanelApplet(hwndParent, wszParams.as_ptr());
    } else {
        RunControlPanelApplet(hwndParent, w!("desk.cpl,,2 /Action:ActivateMSTheme") as PCWSTR);
    }
}
unsafe extern "C" fn WriteUserLocale() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lcid: LCID;
    let mut Locale: [u16; 9] = [0; 9];
    Locale[0] = '0' as u16;
    Locale[1] = '0' as u16;
    Locale[2] = '0' as u16;
    Locale[3] = '0' as u16;

    lcid = GetSystemDefaultLCID();

    if GetLocaleInfoW(MAKELCID!(lcid, SORT_DEFAULT), (LOCALE_ILANGUAGE) as u32, Locale.as_mut_ptr().offset(4), (_countof!(Locale) - 4) as i32) != 0 {
        if RegCreateKeyExW(HKEY_CURRENT_USER,
                            w!("Control Panel\\International") as LPCWSTR,
                            0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE,
                            KEY_WRITE, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::null_mut()) == (ERROR_SUCCESS) as i32 {
            RegSetValueExW(hKey, w!("Locale") as LPCWSTR, 0, (REG_SZ) as u32, Locale.as_ptr() as LPBYTE, ((wcslen(Locale.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
            RegCloseKey(hKey);
        }
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn LocalePageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut SetupData: PSETUPDATA;

    SetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);
            WriteUserLocale();

            SetUserLocaleName(GetDlgItem(hwndDlg, (IDC_LOCALETEXT) as i32));
            SetKeyboardLayoutName(GetDlgItem(hwndDlg, (IDC_LAYOUTTEXT) as i32));
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                match (LOWORD!(wParam)) as u32 {
                    IDC_CUSTOMLOCALE => {
                        RunControlPanelApplet(hwndDlg, w!("intl.cpl,,5") as PCWSTR);
                        SetUserLocaleName(GetDlgItem(hwndDlg, (IDC_LOCALETEXT) as i32));
                    }

                    IDC_CUSTOMLAYOUT => {
                        RunControlPanelApplet(hwndDlg, w!("input.dll,@1") as PCWSTR);
                        SetKeyboardLayoutName(GetDlgItem(hwndDlg, (IDC_LAYOUTTEXT) as i32));
                    }

                    _ => {}
                }
            }
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*SetupData).UnattendSetup != 0 {
                        {
                            RunControlPanelApplet(hwndDlg, w!("intl.cpl,,/f:\"$winnt$.inf\"") as PCWSTR);
                        }

                        SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, (IDD_OWNERPAGE) as LONG_PTR);
                        return TRUE;
                    }
                }

                PSN_WIZNEXT => {}

                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn LocalePageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut SetupData: PSETUPDATA;

    SetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);
            WriteUserLocale();

            SetUserLocaleName(GetDlgItem(hwndDlg, (IDC_LOCALETEXT) as i32));
            SetKeyboardLayoutName(GetDlgItem(hwndDlg, (IDC_LAYOUTTEXT) as i32));
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                match LOWORD!(wParam) as u32 {
                    IDC_CUSTOMLOCALE => {
                        RunControlPanelApplet(hwndDlg, w!("intl.cpl,,5") as PCWSTR);
                        SetUserLocaleName(GetDlgItem(hwndDlg, (IDC_LOCALETEXT) as i32));
                    }

                    IDC_CUSTOMLAYOUT => {
                        RunControlPanelApplet(hwndDlg, w!("input.dll,@1") as PCWSTR);
                        SetKeyboardLayoutName(GetDlgItem(hwndDlg, (IDC_LAYOUTTEXT) as i32));
                    }

                    _ => {}
                }
            }
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*SetupData).UnattendSetup != 0 {
                        {
                            RunControlPanelApplet(hwndDlg, w!("intl.cpl,,/f:\"$winnt$.inf\"") as PCWSTR);
                        }

                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((IDD_OWNERPAGE)) as LONG_PTR);
                        return (TRUE) as i64;
                    }
                }

                PSN_WIZNEXT => {}

                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as i64;
}
unsafe extern "C" fn GetLargerTimeZoneEntry(SetupData: PSETUPDATA, Index: DWORD) -> PTIMEZONE_ENTRY {
    let mut Entry: PTIMEZONE_ENTRY = (*SetupData).TimeZoneListHead;
    while !Entry.is_null() {
        if (*Entry).Index >= Index {
            return Entry;
        }
        Entry = (*Entry).Next;
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn RetrieveTimeZone(
    hZoneKey: HKEY,
    Context: PVOID,
) -> LONG {
    let mut lError: LONG;
    let SetupData = Context as PSETUPDATA;
    let mut Entry: PTIMEZONE_ENTRY;
    let mut Current: PTIMEZONE_ENTRY;
    let mut DescriptionSize: ULONG;
    let mut StandardNameSize: ULONG;
    let mut DaylightNameSize: ULONG;

    Entry = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<TIMEZONE_ENTRY>()) as SIZE_T) as PTIMEZONE_ENTRY;
    if Entry == core::ptr::null_mut()
    {
        return (ERROR_NOT_ENOUGH_MEMORY) as i32;
    }

    DescriptionSize = core::mem::size_of_val(&(*Entry).Description) as ULONG;
    StandardNameSize = core::mem::size_of_val(&(*Entry).StandardName) as ULONG;
    DaylightNameSize = core::mem::size_of_val(&(*Entry).DaylightName) as ULONG;

    lError = QueryTimeZoneData(hZoneKey,
                               core::ptr::addr_of_mut!((*Entry).Index),
                               core::ptr::addr_of_mut!((*Entry).TimezoneInfo),
                               (*Entry).Description.as_mut_ptr(),
                               core::ptr::addr_of_mut!(DescriptionSize),
                               (*Entry).StandardName.as_mut_ptr(),
                               core::ptr::addr_of_mut!(StandardNameSize),
                               (*Entry).DaylightName.as_mut_ptr(),
                               core::ptr::addr_of_mut!(DaylightNameSize));
    if lError != (ERROR_SUCCESS) as i32
    {
        HeapFree(GetProcessHeap(), 0, Entry as LPVOID);
        return lError;
    }

    if (*SetupData).TimeZoneListHead == core::ptr::null_mut() &&
        (*SetupData).TimeZoneListTail == core::ptr::null_mut()
    {
        (*Entry).Prev = core::ptr::null_mut();
        (*Entry).Next = core::ptr::null_mut();
        (*SetupData).TimeZoneListHead = Entry;
        (*SetupData).TimeZoneListTail = Entry;
    }
    else
    {
        Current = GetLargerTimeZoneEntry(SetupData, (*Entry).Index);
        if Current != core::ptr::null_mut()
        {
            if Current == (*SetupData).TimeZoneListHead
            {
                (*Entry).Prev = core::ptr::null_mut();
                (*Entry).Next = (*SetupData).TimeZoneListHead;
                (*(*SetupData).TimeZoneListHead).Prev = Entry;
                (*SetupData).TimeZoneListHead = Entry;
            }
            else
            {
                (*Entry).Prev = (*Current).Prev;
                (*Entry).Next = Current;
                (*(*Current).Prev).Next = Entry;
                (*Current).Prev = Entry;
            }
        }
        else
        {
            (*Entry).Prev = (*SetupData).TimeZoneListTail;
            (*Entry).Next = core::ptr::null_mut();
            (*(*SetupData).TimeZoneListTail).Next = Entry;
            (*SetupData).TimeZoneListTail = Entry;
        }
    }

    return (ERROR_SUCCESS) as i32;
}
unsafe extern "C" fn CreateTimeZoneList(SetupData: PSETUPDATA) {
    EnumerateTimeZoneList(Some(RetrieveTimeZone), SetupData as PVOID);
}
unsafe extern "C" fn DestroyTimeZoneList(SetupData: PSETUPDATA) {
    let mut Entry: PTIMEZONE_ENTRY;

    while (*SetupData).TimeZoneListHead != core::ptr::null_mut() {
        Entry = (*SetupData).TimeZoneListHead;

        (*SetupData).TimeZoneListHead = (*Entry).Next;
        if (*SetupData).TimeZoneListHead != core::ptr::null_mut() {
            (*(*SetupData).TimeZoneListHead).Prev = core::ptr::null_mut();
        }

        HeapFree(GetProcessHeap(), 0, Entry as LPVOID);
    }

    (*SetupData).TimeZoneListTail = core::ptr::null_mut();
}
unsafe extern "C" fn ShowTimeZoneList(hwnd: HWND, SetupData: PSETUPDATA, mut dwEntryIndex: DWORD) {
    let mut Entry: PTIMEZONE_ENTRY;
    let mut dwIndex: DWORD = 0;
    let mut dwCount: DWORD;

    GetTimeZoneListIndex(&mut dwEntryIndex);

    Entry = (*SetupData).TimeZoneListHead;
    while !Entry.is_null() {
        dwCount = (SendMessage(
            hwnd,
            (CB_ADDSTRING) as u32,
            0,
            (*Entry).Description.as_ptr() as LPARAM,
        )) as u32;

        if dwEntryIndex != 0 && dwEntryIndex == (*Entry).Index {
            dwIndex = dwCount;
        }

        Entry = (*Entry).Next;
    }

    SendMessage(
        hwnd,
        (CB_SETCURSEL) as u32,
        dwIndex as WPARAM,
        0,
    );
}
// Upstream 3bbdfb932a3 (CORE-20406): the "automatically adjust for daylight saving" box
// must be unchecked and disabled for a zone with no DST, and follow the zone selection. The
// translation predates it and always checked the box, so picking a no-DST zone still turned
// automatic DST adjustment on.
unsafe fn HasDaylightSavingTime(Entry: PTIMEZONE_ENTRY) -> bool {
    /* If StandardDate.wMonth and DaylightDate.wMonth are zero,
     * the timezone does not observe daylight saving time */
    (*Entry).TimezoneInfo.StandardDate.wMonth != 0 && (*Entry).TimezoneInfo.DaylightDate.wMonth != 0
}

unsafe fn GetSelectedTimeZoneEntry(SetupData: PSETUPDATA, dwEntryIndex: DWORD) -> PTIMEZONE_ENTRY {
    let mut Entry: PTIMEZONE_ENTRY = (*SetupData).TimeZoneListHead;
    while !Entry.is_null() {
        if (*Entry).Index == dwEntryIndex {
            return Entry;
        }
        Entry = (*Entry).Next;
    }
    core::ptr::null_mut()
}

unsafe fn GetTimeZoneEntryByIndex(SetupData: PSETUPDATA, dwComboIndex: DWORD) -> PTIMEZONE_ENTRY {
    let mut Entry: PTIMEZONE_ENTRY = (*SetupData).TimeZoneListHead;
    let mut i: DWORD = 0;
    while !Entry.is_null() && i < dwComboIndex {
        i += 1;
        Entry = (*Entry).Next;
    }
    Entry
}

/// Check and enable the auto-daylight box exactly when `Entry` observes DST.
unsafe fn SetAutoDaylightCheckbox(hwndDlg: HWND, bHasDST: bool) {
    /* Enable or disable the checkbox based on DST support */
    EnableWindow(GetDlgItem(hwndDlg, (IDC_AUTODAYLIGHT) as i32), bHasDST as BOOL);

    /* Check the checkbox only if DST is supported, otherwise uncheck it */
    SendDlgItemMessage(
        hwndDlg,
        (IDC_AUTODAYLIGHT) as i32,
        BM_SETCHECK as UINT,
        (if bHasDST { BST_CHECKED } else { BST_UNCHECKED }) as WPARAM,
        0,
    );
}

unsafe fn UpdateAutoDaylightCheckbox(hwndDlg: HWND, Entry: PTIMEZONE_ENTRY) {
    SetAutoDaylightCheckbox(hwndDlg, !Entry.is_null() && HasDaylightSavingTime(Entry));
}

unsafe extern "C" fn SetLocalTimeZone(hwnd: HWND, SetupData: PSETUPDATA) {
    let mut TimeZoneInformation: TIME_ZONE_INFORMATION = core::mem::zeroed();
    let mut Entry: PTIMEZONE_ENTRY;
    let dwIndex: DWORD;
    let mut i: DWORD;

    dwIndex = (SendMessage(
        hwnd,
        (CB_GETCURSEL) as u32,
        0,
        0
    )) as u32;

    i = 0;
    Entry = (*SetupData).TimeZoneListHead;
    while i < dwIndex {
        if Entry.is_null() {
            return;
        }

        i += 1;
        Entry = (*Entry).Next;
    }

    wcscpy(
        TimeZoneInformation.StandardName.as_mut_ptr(),
        (*Entry).StandardName.as_ptr()
    );
    wcscpy(
        TimeZoneInformation.DaylightName.as_mut_ptr(),
        (*Entry).DaylightName.as_ptr()
    );

    TimeZoneInformation.Bias = (*Entry).TimezoneInfo.Bias;
    TimeZoneInformation.StandardBias = (*Entry).TimezoneInfo.StandardBias;
    TimeZoneInformation.DaylightBias = (*Entry).TimezoneInfo.DaylightBias;

    memcpy(
        core::ptr::addr_of_mut!(TimeZoneInformation.StandardDate) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*Entry).TimezoneInfo.StandardDate) as *const core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>()
    );
    memcpy(
        core::ptr::addr_of_mut!(TimeZoneInformation.DaylightDate) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*Entry).TimezoneInfo.DaylightDate) as *const core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>()
    );

    SetTimeZoneInformation(core::ptr::addr_of_mut!(TimeZoneInformation));
}
unsafe extern "C" fn GetLocalSystemTime(hwnd: HWND, SetupData: PSETUPDATA) -> BOOL {
    let mut Date: SYSTEMTIME = core::mem::zeroed();
    let mut Time: SYSTEMTIME = core::mem::zeroed();

    if DateTime_GetSystemtime!(GetDlgItem(hwnd, (IDC_DATEPICKER) as i32), &mut Date) != (GDT_VALID) as u32 {
        return (FALSE) as i32;
    }

    if DateTime_GetSystemtime!(GetDlgItem(hwnd, (IDC_TIMEPICKER) as i32), &mut Time) != (GDT_VALID) as u32 {
        return (FALSE) as i32;
    }

    (*SetupData).SystemTime.wYear = Date.wYear;
    (*SetupData).SystemTime.wMonth = Date.wMonth;
    (*SetupData).SystemTime.wDayOfWeek = Date.wDayOfWeek;
    (*SetupData).SystemTime.wDay = Date.wDay;
    (*SetupData).SystemTime.wHour = Time.wHour;
    (*SetupData).SystemTime.wMinute = Time.wMinute;
    (*SetupData).SystemTime.wSecond = Time.wSecond;
    (*SetupData).SystemTime.wMilliseconds = Time.wMilliseconds;

    return (TRUE) as i32;
}
unsafe extern "C" fn SetSystemLocalTime(hwnd: HWND, SetupData: PSETUPDATA) -> BOOL {
    let mut Ret: BOOL = (FALSE) as i32;

    Ret = (SetLocalTime(core::ptr::addr_of_mut!((*SetupData).SystemTime)) != 0 &&
           SetLocalTime(core::ptr::addr_of_mut!((*SetupData).SystemTime)) != 0) as BOOL;

    return Ret;
}
unsafe extern "C" fn UpdateLocalSystemTime(hwnd: HWND, LocalTime: SYSTEMTIME) {
    DateTime_SetSystemtime!(GetDlgItem(hwnd, (IDC_DATEPICKER) as i32), GDT_VALID, &LocalTime);
    DateTime_SetSystemtime!(GetDlgItem(hwnd, (IDC_TIMEPICKER) as i32), GDT_VALID, &LocalTime);
}
unsafe extern "C" fn WriteDateTimeSettings(hwndDlg: HWND, SetupData: PSETUPDATA) -> BOOL {
    let mut Title: [WCHAR; 64] = [0; 64];
    let mut ErrorLocalTime: [WCHAR; 256] = [0; 256];

    GetLocalSystemTime(hwndDlg, SetupData);
    SetLocalTimeZone(GetDlgItem(hwndDlg, (IDC_TIMEZONELIST) as i32),
                     SetupData);

    SetAutoDaylight(
        (SendDlgItemMessage(hwndDlg, (IDC_AUTODAYLIGHT) as i32, (BM_GETCHECK) as u32, 0, 0)
            != BST_UNCHECKED as LRESULT) as BOOL,
    );
    if SetSystemLocalTime(hwndDlg, SetupData) == 0 {
        if 0 == LoadStringW(hDllInstance, (IDS_REACTOS_SETUP) as u32, Title.as_mut_ptr(), (ARRAYSIZE!(Title)) as i32) {
            wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as LPWSTR);
        }
        if 0 == LoadStringW(hDllInstance, (IDS_WZD_LOCALTIME) as u32, ErrorLocalTime.as_mut_ptr(),
                             (ARRAYSIZE!(ErrorLocalTime)) as i32) {
            wcscpy(ErrorLocalTime.as_mut_ptr(), w!("Setup was unable to set the local time.") as LPWSTR);
        }
        MessageBoxW(hwndDlg, ErrorLocalTime.as_ptr(), Title.as_ptr(), (MB_ICONWARNING | MB_OK) as u32);
        return (FALSE) as i32;
    }

    return (TRUE) as i32;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn DateTimePageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let SetupData: PSETUPDATA = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let SetupData: PSETUPDATA = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);

            CreateTimeZoneList(SetupData);

            if (*SetupData).UnattendSetup != 0 {
                ShowTimeZoneList(
                    GetDlgItem(hwndDlg, (IDC_TIMEZONELIST) as i32),
                    SetupData,
                    (*SetupData).TimeZoneIndex,
                );

                let Entry = GetSelectedTimeZoneEntry(SetupData, (*SetupData).TimeZoneIndex);
                SetAutoDaylightCheckbox(
                    hwndDlg,
                    (*SetupData).DisableAutoDaylightTimeSet == 0
                        && !Entry.is_null()
                        && HasDaylightSavingTime(Entry),
                );
            } else {
                /* Get the default time zone index from the registry */
                let mut dwEntryIndex: DWORD = (1 as i32).wrapping_neg() as u32;
                GetTimeZoneListIndex(&mut dwEntryIndex);

                ShowTimeZoneList(
                    GetDlgItem(hwndDlg, (IDC_TIMEZONELIST) as i32),
                    SetupData,
                    (1 as i32).wrapping_neg() as u32,
                );

                /* Set the auto-daylight checkbox based on whether
                 * the selected timezone observes DST */
                UpdateAutoDaylightCheckbox(hwndDlg, GetSelectedTimeZoneEntry(SetupData, dwEntryIndex));
            }
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 && LOWORD!(wParam) as u32 == IDC_TIMEZONELIST {
                /* User changed the timezone selection */
                let dwIndex = SendDlgItemMessage(
                    hwndDlg,
                    (IDC_TIMEZONELIST) as i32,
                    CB_GETCURSEL as UINT,
                    0,
                    0,
                ) as DWORD;
                UpdateAutoDaylightCheckbox(hwndDlg, GetTimeZoneEntryByIndex(SetupData, dwIndex));
            }
        }

        WM_TIMER => {
            let mut LocalTime: SYSTEMTIME = core::mem::zeroed();

            GetLocalTime(&mut LocalTime);
            UpdateLocalSystemTime(hwndDlg, LocalTime);

            SetTimer(
                hwndDlg,
                1,
                1000 - LocalTime.wMilliseconds as u32,
                None,
            );
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_SETACTIVE => {
                    let mut LocalTime: SYSTEMTIME = core::mem::zeroed();

                    GetLocalTime(&mut LocalTime);
                    UpdateLocalSystemTime(hwndDlg, LocalTime);

                    PropSheet_SetWizButtons!(
                        GetParent(hwndDlg),
                        PSWIZB_BACK | PSWIZB_NEXT,
                    );

                    if (*SetupData).UnattendSetup != 0 && WriteDateTimeSettings(hwndDlg, SetupData) != 0 {
                        SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, (IDD_THEMEPAGE) as LONG_PTR);
                        return TRUE;
                    }

                    SetTimer(
                        hwndDlg,
                        1,
                        1000 - LocalTime.wMilliseconds as u32,
                        None,
                    );
                }

                PSN_KILLACTIVE | DTN_DATETIMECHANGE => {
                    KillTimer(hwndDlg, 1);
                }

                PSN_WIZNEXT => {
                    WriteDateTimeSettings(hwndDlg, SetupData);
                }

                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        WM_DESTROY => {
            DestroyTimeZoneList(SetupData);
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn DateTimePageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let SetupData: PSETUPDATA = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let SetupData: PSETUPDATA = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);

            CreateTimeZoneList(SetupData);

            if (*SetupData).UnattendSetup != 0 {
                ShowTimeZoneList(
                    GetDlgItem(hwndDlg, (IDC_TIMEZONELIST) as i32),
                    SetupData,
                    (*SetupData).TimeZoneIndex,
                );

                let Entry = GetSelectedTimeZoneEntry(SetupData, (*SetupData).TimeZoneIndex);
                SetAutoDaylightCheckbox(
                    hwndDlg,
                    (*SetupData).DisableAutoDaylightTimeSet == 0
                        && !Entry.is_null()
                        && HasDaylightSavingTime(Entry),
                );
            } else {
                /* Get the default time zone index from the registry */
                let mut dwEntryIndex: DWORD = (1 as i32).wrapping_neg() as u32;
                GetTimeZoneListIndex(&mut dwEntryIndex);

                ShowTimeZoneList(
                    GetDlgItem(hwndDlg, (IDC_TIMEZONELIST) as i32),
                    SetupData,
                    (1 as i32).wrapping_neg() as u32,
                );

                /* Set the auto-daylight checkbox based on whether
                 * the selected timezone observes DST */
                UpdateAutoDaylightCheckbox(hwndDlg, GetSelectedTimeZoneEntry(SetupData, dwEntryIndex));
            }
        }

        WM_COMMAND => {
            if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 && LOWORD!(wParam) as u32 == IDC_TIMEZONELIST {
                /* User changed the timezone selection */
                let dwIndex = SendDlgItemMessage(
                    hwndDlg,
                    (IDC_TIMEZONELIST) as i32,
                    CB_GETCURSEL as UINT,
                    0,
                    0,
                ) as DWORD;
                UpdateAutoDaylightCheckbox(hwndDlg, GetTimeZoneEntryByIndex(SetupData, dwIndex));
            }
        }

        WM_TIMER => {
            let mut LocalTime: SYSTEMTIME = core::mem::zeroed();

            GetLocalTime(&mut LocalTime);
            UpdateLocalSystemTime(hwndDlg, LocalTime);

            SetTimer(
                hwndDlg,
                1,
                1000 - LocalTime.wMilliseconds as u32,
                None,
            );
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_SETACTIVE => {
                    let mut LocalTime: SYSTEMTIME = core::mem::zeroed();

                    GetLocalTime(&mut LocalTime);
                    UpdateLocalSystemTime(hwndDlg, LocalTime);

                    PropSheet_SetWizButtons!(
                        GetParent(hwndDlg),
                        PSWIZB_BACK | PSWIZB_NEXT,
                    );

                    if (*SetupData).UnattendSetup != 0 && WriteDateTimeSettings(hwndDlg, SetupData) != 0 {
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, ((IDD_THEMEPAGE)) as LONG_PTR);
                        return (TRUE) as i64;
                    }

                    SetTimer(
                        hwndDlg,
                        1,
                        1000 - LocalTime.wMilliseconds as u32,
                        None,
                    );
                }

                PSN_KILLACTIVE | DTN_DATETIMECHANGE => {
                    KillTimer(hwndDlg, 1);
                }

                PSN_WIZNEXT => {
                    WriteDateTimeSettings(hwndDlg, SetupData);
                }

                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        WM_DESTROY => {
            DestroyTimeZoneList(SetupData);
        }

        _ => {}
    }

    (FALSE) as i64
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn ThemePageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut SetupData: PSETUPDATA;
    let mut pnmv: LPNMLISTVIEW;

    SetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hListView: HWND;
            let mut himl: HIMAGELIST;
            let mut n: DWORD;
            let mut lvi: LVITEM = core::mem::zeroed();

            SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);

            hListView = GetDlgItem(hwndDlg, (IDC_THEMEPICKER) as i32);

            himl = ImageList_Create(
                180,
                163,
                ILC_COLOR32 | ILC_MASK,
                (ARRAYSIZE!(Themes)) as i32,
                1,
            );
            lvi.mask = LVIF_TEXT | LVIF_IMAGE | LVIF_STATE;

            n = 0;
            while n < ARRAYSIZE!(Themes) {
                let mut DisplayName: [WCHAR; 100] = [0; 100];
                let image = LoadImageW(
                    hDllInstance,
                    Themes[n as usize].PreviewBitmap,
                    IMAGE_BITMAP,
                    0,
                    0,
                    LR_CREATEDIBSECTION,
                );
                ImageList_AddMasked(himl, (image).cast(), RGB!(255, 0, 255));

                LoadStringW(
                    hDllInstance,
                    Themes[n as usize].DisplayName,
                    DisplayName.as_mut_ptr(),
                    (ARRAYSIZE!(DisplayName)) as i32,
                );
                DisplayName[(ARRAYSIZE!(DisplayName) - 1) as usize] = UNICODE_NULL;

                lvi.iItem = n as i32;
                lvi.iImage = n as i32;
                lvi.pszText = DisplayName.as_mut_ptr();
                ListView_InsertItem!(hListView, &mut lvi);

                n += 1;
            }

            ListView_SetImageList!(hListView, himl, LVSIL_NORMAL);
            ListView_SetBkColor!(hListView, CLR_NONE);
            ListView_SetTextBkColor!(hListView, CLR_NONE);
            ListView_SetIconSpacing!(hListView, 190, 173);
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                LVN_ITEMCHANGED => {
                    pnmv = lParam as LPNMLISTVIEW;
                    if ((*pnmv).uChanged & LVIF_STATE) != 0
                        && ((*pnmv).uNewState & LVIS_SELECTED) != 0
                    {
                        let iTheme = (*pnmv).iItem;
                        // DPRINT removed

                        if !Themes[iTheme as usize].ThemeFile.is_null() {
                            let mut wszTheme: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
                            SHGetFolderPathAndSubDirW(
                                core::ptr::null_mut(),
                                CSIDL_RESOURCES as i32,
                                core::ptr::null_mut(),
                                (SHGFP_TYPE_DEFAULT) as DWORD,
                                Themes[iTheme as usize].ThemeFile,
                                wszTheme.as_mut_ptr(),
                            );
                            EnableVisualTheme(hwndDlg, wszTheme.as_ptr());
                        } else {
                            EnableVisualTheme(hwndDlg, Themes[iTheme as usize].ThemeFile);
                        }
                    }
                }
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*SetupData).UnattendSetup != 0 {
                        SetWindowLongPtr(hwndDlg, (DWLP_MSGRESULT) as c_int, ((*SetupData).uFirstNetworkWizardPage) as LONG_PTR);
                        return TRUE;
                    }
                }
                PSN_WIZNEXT => {}
                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = FALSE;
                }
                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn ThemePageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut SetupData: PSETUPDATA;
    let mut pnmv: LPNMLISTVIEW;

    SetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hListView: HWND;
            let mut himl: HIMAGELIST;
            let mut n: DWORD;
            let mut lvi: LVITEM = core::mem::zeroed();

            SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);

            hListView = GetDlgItem(hwndDlg, (IDC_THEMEPICKER) as i32);

            himl = ImageList_Create(
                180,
                163,
                (ILC_COLOR32 | ILC_MASK) as u32,
                (ARRAYSIZE!(Themes)) as i32,
                1,
            );
            lvi.mask = (LVIF_TEXT | LVIF_IMAGE | LVIF_STATE) as u32;

            n = 0;
            while n < ARRAYSIZE!(Themes) {
                let mut DisplayName: [WCHAR; 100] = [0; 100];
                let image = LoadImageW(
                    hDllInstance,
                    Themes[n as usize].PreviewBitmap,
                    (IMAGE_BITMAP) as u32,
                    0,
                    0,
                    (LR_CREATEDIBSECTION) as u32,
                );
                ImageList_AddMasked(himl, (image).cast(), RGB!(255, 0, 255));

                LoadStringW(
                    hDllInstance,
                    Themes[n as usize].DisplayName,
                    DisplayName.as_mut_ptr(),
                    (ARRAYSIZE!(DisplayName)) as i32,
                );
                DisplayName[(ARRAYSIZE!(DisplayName) - 1) as usize] = UNICODE_NULL;

                lvi.iItem = n as i32;
                lvi.iImage = n as i32;
                lvi.pszText = DisplayName.as_mut_ptr();
                ListView_InsertItem!(hListView, &mut lvi);

                n += 1;
            }

            ListView_SetImageList!(hListView, himl, LVSIL_NORMAL);
            ListView_SetBkColor!(hListView, CLR_NONE);
            ListView_SetTextBkColor!(hListView, CLR_NONE);
            ListView_SetIconSpacing!(hListView, 190, 173);
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                LVN_ITEMCHANGED => {
                    pnmv = lParam as LPNMLISTVIEW;
                    if ((*pnmv).uChanged & (LVIF_STATE) as u32) != 0
                        && ((*pnmv).uNewState & (LVIS_SELECTED) as u32) != 0
                    {
                        let iTheme = (*pnmv).iItem;
                        // DPRINT removed

                        if !Themes[iTheme as usize].ThemeFile.is_null() {
                            let mut wszTheme: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
                            SHGetFolderPathAndSubDirW(
                                core::ptr::null_mut(),
                                (CSIDL_RESOURCES) as i32,
                                core::ptr::null_mut(),
                                (SHGFP_TYPE_DEFAULT) as u32,
                                Themes[iTheme as usize].ThemeFile,
                                wszTheme.as_mut_ptr(),
                            );
                            EnableVisualTheme(hwndDlg, wszTheme.as_ptr());
                        } else {
                            EnableVisualTheme(hwndDlg, Themes[iTheme as usize].ThemeFile);
                        }
                    }
                }
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    if (*SetupData).UnattendSetup != 0 {
                        SetWindowLongPtr(hwndDlg, ((DWLP_MSGRESULT)) as c_int, (((*SetupData).uFirstNetworkWizardPage)) as LONG_PTR);
                        return (TRUE) as i64;
                    }
                }
                PSN_WIZNEXT => {}
                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = (FALSE) as i32;
                }
                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as i64
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn RegistrationNotificationProc(
    Context: PVOID,
    Notification: UINT,
    Param1: UINT_PTR,
    Param2: UINT_PTR,
) -> UINT {
    let RegistrationData: PREGISTRATIONDATA;
    let StatusInfo: PSP_REGISTER_CONTROL_STATUSW;
    let mut MessageID: UINT;

    RegistrationData = Context as PREGISTRATIONDATA;

    if Notification == SPFILENOTIFY_STARTREGISTRATION ||
       Notification == SPFILENOTIFY_ENDREGISTRATION {
        StatusInfo = Param1 as PSP_REGISTER_CONTROL_STATUSW;
        (*(*RegistrationData).pNotify).CurrentItem = wcsrchr((*StatusInfo).FileName, '\\' as u16);
        if (*(*RegistrationData).pNotify).CurrentItem == core::ptr::null_mut() {
            (*(*RegistrationData).pNotify).CurrentItem = (*StatusInfo).FileName;
        } else {
            (*(*RegistrationData).pNotify).CurrentItem = (*(*RegistrationData).pNotify).CurrentItem.offset(1);
        }

        if Notification == SPFILENOTIFY_STARTREGISTRATION {
            // DPRINT removed
            (*(*RegistrationData).pNotify).Progress = (*RegistrationData).Registered;

            // DPRINT removed
            SendMessage((*RegistrationData).hwndDlg, PM_STEP_START, 0, (*RegistrationData).pNotify as LPARAM);
        } else {
            // DPRINT removed
            // DPRINT removed
            if (*StatusInfo).FailureCode != SPREG_SUCCESS {
                MessageID = match (*StatusInfo).FailureCode {
                    SPREG_LOADLIBRARY => IDS_LOADLIBRARY_FAILED,
                    SPREG_GETPROCADDR => IDS_GETPROCADDR_FAILED,
                    SPREG_REGSVR => IDS_REGSVR_FAILED,
                    SPREG_DLLINSTALL => IDS_DLLINSTALL_FAILED,
                    SPREG_TIMEOUT => IDS_TIMEOUT,
                    _ => IDS_REASON_UNKNOWN,
                };

                (*(*RegistrationData).pNotify).MessageID = MessageID;
                (*(*RegistrationData).pNotify).LastError = (*StatusInfo).Win32Error;
            } else {
                (*(*RegistrationData).pNotify).MessageID = 0;
                (*(*RegistrationData).pNotify).LastError = ERROR_SUCCESS;
            }

            if (*RegistrationData).Registered < (*RegistrationData).DllCount {
                (*RegistrationData).Registered = (*RegistrationData).Registered + 1;
            }

            (*(*RegistrationData).pNotify).Progress = (*RegistrationData).Registered;
            // DPRINT removed
            SendMessage((*RegistrationData).hwndDlg, PM_STEP_END, 0, (*RegistrationData).pNotify as LPARAM);
        }

        return FILEOP_DOIT;
    } else {
        // DPRINT removed
        return SetupDefaultQueueCallback((*RegistrationData).DefaultContext,
                                      Notification, Param1, Param2);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn RegistrationNotificationProc(
    Context: PVOID,
    Notification: UINT,
    Param1: UINT_PTR,
    Param2: UINT_PTR,
) -> UINT {
    let RegistrationData: PREGISTRATIONDATA;
    let StatusInfo: PSP_REGISTER_CONTROL_STATUSW;
    let mut MessageID: UINT;

    RegistrationData = Context as PREGISTRATIONDATA;

    if Notification == (SPFILENOTIFY_STARTREGISTRATION) as u32 ||
       Notification == (SPFILENOTIFY_ENDREGISTRATION) as u32 {
        StatusInfo = Param1 as PSP_REGISTER_CONTROL_STATUSW;
        (*(*RegistrationData).pNotify).CurrentItem = wcsrchr((*StatusInfo).FileName, '\\' as u16);
        if (*(*RegistrationData).pNotify).CurrentItem == core::ptr::null_mut() {
            (*(*RegistrationData).pNotify).CurrentItem = (*StatusInfo).FileName;
        } else {
            (*(*RegistrationData).pNotify).CurrentItem = (*(*RegistrationData).pNotify).CurrentItem.offset(1);
        }

        if Notification == (SPFILENOTIFY_STARTREGISTRATION) as u32 {
            // DPRINT removed
            (*(*RegistrationData).pNotify).Progress = (*RegistrationData).Registered;

            // DPRINT removed
            SendMessage((*RegistrationData).hwndDlg, (PM_STEP_START) as u32, 0, (*RegistrationData).pNotify as LPARAM);
        } else {
            // DPRINT removed
            // DPRINT removed
            if (*StatusInfo).FailureCode != (SPREG_SUCCESS) as u32 {
                MessageID = match (*StatusInfo).FailureCode {
                    SPREG_LOADLIBRARY => (IDS_LOADLIBRARY_FAILED) as u32,
                    SPREG_GETPROCADDR => (IDS_GETPROCADDR_FAILED) as u32,
                    SPREG_REGSVR => (IDS_REGSVR_FAILED) as u32,
                    SPREG_DLLINSTALL => (IDS_DLLINSTALL_FAILED) as u32,
                    SPREG_TIMEOUT => (IDS_TIMEOUT) as u32,
                    _ => (IDS_REASON_UNKNOWN) as u32,
                };

                (*(*RegistrationData).pNotify).MessageID = MessageID;
                (*(*RegistrationData).pNotify).LastError = (*StatusInfo).Win32Error;
            } else {
                (*(*RegistrationData).pNotify).MessageID = 0;
                (*(*RegistrationData).pNotify).LastError = ERROR_SUCCESS;
            }

            if (*RegistrationData).Registered < (*RegistrationData).DllCount {
                (*RegistrationData).Registered = (*RegistrationData).Registered + 1;
            }

            (*(*RegistrationData).pNotify).Progress = (*RegistrationData).Registered;
            // DPRINT removed
            SendMessage((*RegistrationData).hwndDlg, (PM_STEP_END) as u32, 0, (*RegistrationData).pNotify as LPARAM);
        }

        return (FILEOP_DOIT) as u32;
    } else {
        // DPRINT removed
        return SetupDefaultQueueCallback((*RegistrationData).DefaultContext,
                                      Notification, Param1, Param2);
    }
}
unsafe extern "C" fn RegisterDlls(
    pItemsData: PITEMSDATA,
    pNotify: PREGISTRATIONNOTIFY,
) -> DWORD {
    let mut RegistrationData: REGISTRATIONDATA = core::mem::zeroed();
    let mut SectionName: [WCHAR; 512] = core::mem::zeroed();
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut DllCount: LONG = 0;
    let mut Error: DWORD = NO_ERROR;

    ZeroMemory!(
        &mut RegistrationData as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<REGISTRATIONDATA>(),
    );
    RegistrationData.hwndDlg = (*pItemsData).hwndDlg;
    RegistrationData.Registered = 0;

    if SetupFindFirstLineW(
        hSysSetupInf,
        w!("RegistrationPhase2") as PCWSTR,
        w!("RegisterDlls") as PCWSTR,
        &mut Context,
    ) == 0
    {
        // DPRINT removed
        return GetLastError();
    }

    if SetupGetStringFieldW(
        &mut Context,
        1,
        SectionName.as_mut_ptr(),
        ARRAYSIZE!(SectionName),
        core::ptr::null_mut(),
    ) == 0
    {
        // DPRINT removed
        return GetLastError();
    }

    DllCount = SetupGetLineCountW(hSysSetupInf, SectionName.as_ptr());
    // DPRINT removed
    if DllCount < 0 {
        return (STATUS_NOT_FOUND) as u32;
    }

    RegistrationData.DllCount = DllCount as ULONG;
    RegistrationData.DefaultContext = SetupInitDefaultQueueCallback(RegistrationData.hwndDlg);
    RegistrationData.pNotify = pNotify;

    // A DllRegisterServer may use COM -- quartz, qcap, devenum and qedit register through
    // IFilterMapper2 -- and this worker thread never initialised it, so each one failed
    // with CO_E_NOTINITIALIZED (upstream 588c5df465e). regsvr32 brackets the call the same way.
    let HrCoInit = CoInitialize(core::ptr::null_mut());

    if SetupInstallFromInfSectionW(
        GetParent(RegistrationData.hwndDlg),
        hSysSetupInf,
        w!("RegistrationPhase2") as PCWSTR,
        (SPINST_REGISTRY | ((SPINST_REGISTERCALLBACKAWARE) as i32) as u32 | SPINST_REGSVR) as u32,
        core::ptr::null_mut(),
        core::ptr::null(),
        0,
        Some(RegistrationNotificationProc),
        &mut RegistrationData as *mut _ as PVOID,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        Error = GetLastError();
    }

    if HrCoInit >= 0 {
        CoUninitialize();
    }

    SetupTermDefaultQueueCallback(RegistrationData.DefaultContext);

    Error
}
unsafe extern "C" fn RegisterComponents(
    pItemsData: PITEMSDATA,
) {
    let mut SectionName: [u16; 512] = [0; 512];
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut Steps: i32 = 0;
    let mut Error: u32 = NO_ERROR;
    let mut Notify: REGISTRATIONNOTIFY = core::mem::zeroed();

    ZeroMemory!(&mut Notify as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<REGISTRATIONNOTIFY>());

    if SetupFindFirstLineW(hSysSetupInf, w!("RegistrationPhase2") as PCWSTR,
                             w!("RegisterDlls") as PCWSTR, &mut Context) == 0 {
        // DPRINT removed
        return;
    }

    if SetupGetStringFieldW(&mut Context, 1, SectionName.as_mut_ptr(),
                              ARRAYSIZE!(SectionName),
                              core::ptr::null_mut()) == 0 {
        // DPRINT removed
        return;
    }

    Steps += SetupGetLineCountW(hSysSetupInf, SectionName.as_ptr());

    Steps += SetupGetLineCountW(hSysSetupInf, w!("TypeLibraries") as PCWSTR);

    // DPRINT removed
    SendMessage((*pItemsData).hwndDlg, (PM_ITEM_START) as u32, 0, Steps as LPARAM);

    Error = RegisterDlls(pItemsData, &mut Notify);
    if Error == ERROR_SUCCESS {
        RegisterTypeLibraries(pItemsData, &mut Notify, hSysSetupInf, w!("TypeLibraries") as PCWSTR);
    }

    // DPRINT removed
    SendMessage((*pItemsData).hwndDlg, (PM_ITEM_END) as u32, 0, Error as LPARAM);
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn ItemCompletionThread(
    Parameter: LPVOID,
) -> DWORD {
    let pItemsData: PITEMSDATA;
    let hwndDlg: HWND;

    pItemsData = Parameter as PITEMSDATA;
    hwndDlg = (*pItemsData).hwndDlg;

    RegisterComponents(pItemsData);

    InstallStartMenuItems(pItemsData);

    RtlCreateBootStatusDataFile();

    HeapFree(GetProcessHeap(), 0, pItemsData as LPVOID);

    PostMessage(hwndDlg, PM_ITEMS_DONE, 0, 0);

    return 0;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn ItemCompletionThread(
    Parameter: LPVOID,
) -> DWORD {
    let pItemsData: PITEMSDATA;
    let hwndDlg: HWND;

    pItemsData = Parameter as PITEMSDATA;
    hwndDlg = (*pItemsData).hwndDlg;

    RegisterComponents(pItemsData);

    InstallStartMenuItems(pItemsData);

    RtlCreateBootStatusDataFile();

    HeapFree(GetProcessHeap(), 0, pItemsData as LPVOID);

    PostMessage(hwndDlg, (PM_ITEMS_DONE) as u32, 0, 0);

    return 0;
}
unsafe extern "C" fn RunItemCompletionThread(
    hwndDlg: HWND) -> BOOL {
    let hCompletionThread: HANDLE;
    let pItemsData: PITEMSDATA;

    pItemsData = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<ITEMSDATA>()) as SIZE_T) as PITEMSDATA;
    if pItemsData == core::ptr::null_mut()
    {
        return (FALSE) as i32;
    }

    (*pItemsData).hwndDlg = hwndDlg;

    hCompletionThread = CreateThread(core::ptr::null_mut(),
                                     0,
                                     Some(ItemCompletionThread),
                                     pItemsData as LPVOID,
                                     0,
                                     core::ptr::null_mut());
    if hCompletionThread == core::ptr::null_mut()
    {
        HeapFree(GetProcessHeap(), 0, pItemsData as LPVOID);
    }
    else
    {
        CloseHandle(hCompletionThread);
        return (TRUE) as i32;
    }

    return (FALSE) as i32;
}
unsafe extern "C" fn ShowItemError(
    hwndDlg: HWND,
    LastError: DWORD) {
    let mut ErrorMessage: LPWSTR = core::ptr::null_mut();
    let mut UnknownError: [u16; 84] = [0; 84];
    let mut Title: [u16; 64] = [0; 64];

    if FormatMessageW((FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM) as u32,
                       core::ptr::null_mut(), LastError, 0,
                       core::ptr::addr_of_mut!(ErrorMessage) as LPWSTR, 0, core::ptr::null_mut()) == 0
    {
        if LoadStringW(hDllInstance, (IDS_UNKNOWN_ERROR) as u32,
                        UnknownError.as_mut_ptr(),
                        (ARRAYSIZE!(UnknownError) - 20) as i32) == 0
        {
            wcscpy(UnknownError.as_mut_ptr(), w!("Unknown error") as LPCWSTR);
        }
        wcscat(UnknownError.as_mut_ptr(), w!(" ") as LPCWSTR);
        _ultow(LastError, UnknownError.as_mut_ptr().offset(wcslen(UnknownError.as_ptr()) as isize), 10);
        ErrorMessage = UnknownError.as_mut_ptr();
    }

    if !ErrorMessage.is_null()
    {
        if LoadStringW(hDllInstance, (IDS_REACTOS_SETUP) as u32,
                        Title.as_mut_ptr(), (ARRAYSIZE!(Title)) as i32) == 0
        {
            wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as LPCWSTR);
        }

        MessageBoxW(hwndDlg, ErrorMessage, Title.as_ptr(), (MB_ICONERROR | MB_OK) as u32);
    }

    if !ErrorMessage.is_null() &&
        ErrorMessage != UnknownError.as_mut_ptr()
    {
        LocalFree(ErrorMessage as HLOCAL);
    }
}
unsafe extern "C" fn ShowStepError(
    hwndDlg: HWND,
    RegistrationNotify: PREGISTRATIONNOTIFY,
) {
    let mut ErrorMessage: [u16; 128] = [0; 128];
    let mut Title: [u16; 64] = [0; 64];

    if LoadStringW(
        hDllInstance,
        (*RegistrationNotify).MessageID,
        ErrorMessage.as_mut_ptr(),
        (ARRAYSIZE!(ErrorMessage)) as i32,
    ) == 0
    {
        ErrorMessage[0] = 0;
    }

    if (*RegistrationNotify).MessageID != (IDS_TIMEOUT) as u32 {
        FormatMessageW(
            (FORMAT_MESSAGE_FROM_SYSTEM) as u32,
            core::ptr::null_mut(),
            (*RegistrationNotify).LastError,
            0,
            ErrorMessage.as_mut_ptr().add(wcslen(ErrorMessage.as_ptr())),
            ARRAYSIZE!(ErrorMessage) - (wcslen(ErrorMessage.as_ptr())) as u32,
            core::ptr::null_mut(),
        );
    }

    if ErrorMessage[0] != 0 {
        if LoadStringW(
            hDllInstance,
            (IDS_REACTOS_SETUP) as u32,
            Title.as_mut_ptr(),
            (ARRAYSIZE!(Title)) as i32,
        ) == 0
        {
            wcscpy(Title.as_mut_ptr(), w!("ReactOS Setup") as *const u16);
        }

        MessageBoxW(
            hwndDlg,
            ErrorMessage.as_ptr(),
            Title.as_ptr(),
            (MB_ICONERROR | MB_OK) as u32,
        );
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn ProcessPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut SetupData: PSETUPDATA;
    let mut RegistrationNotify: PREGISTRATIONNOTIFY;
    static mut s_hCheckIcon: HICON = core::ptr::null_mut();
    static mut s_hArrowIcon: HICON = core::ptr::null_mut();
    static mut s_hCrossIcon: HICON = core::ptr::null_mut();
    static mut s_hNormalFont: HFONT = core::ptr::null_mut();

    SetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_TASKTEXT3) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_TASKTEXT4) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_CHECK3) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_CHECK4) as i32), (SW_HIDE) as i32);
            s_hCheckIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_CHECKICON), IMAGE_ICON, 16, 16, 0) as HICON;
            s_hArrowIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_ARROWICON), IMAGE_ICON, 16, 16, 0) as HICON;
            s_hCrossIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_CROSSICON), IMAGE_ICON, 16, 16, 0) as HICON;
            s_hNormalFont = SendDlgItemMessage(hwndDlg, (IDC_TASKTEXT1) as i32, WM_GETFONT, 0, 0) as HFONT;
        }

        WM_DESTROY => {
            DestroyIcon(s_hCheckIcon);
            DestroyIcon(s_hArrowIcon);
            DestroyIcon(s_hCrossIcon);
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_SETACTIVE => {
                    LogItem!(w!("BEGIN") as LPCWSTR, w!("ProcessPage") as LPCWSTR);
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), 0);
                    RunItemCompletionThread(hwndDlg);
                }

                PSN_WIZNEXT => {
                    LogItem!(w!("END") as LPCWSTR, w!("ProcessPage") as LPCWSTR);
                }

                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = FALSE;
                }

                _ => {}
            }
        }

        PM_ITEM_START => {
            SendDlgItemMessage(hwndDlg, (IDC_PROCESSPROGRESS) as i32, PBM_SETRANGE, 0, MAKELPARAM!(0, lParam as ULONG));
            SendDlgItemMessage(hwndDlg, (IDC_PROCESSPROGRESS) as i32, PBM_SETPOS, 0, 0);
            SendDlgItemMessage(hwndDlg, (IDC_TASKTEXT1 + wParam) as i32, WM_SETFONT, (*SetupData).hBoldFont as WPARAM, TRUE as LPARAM);
            SendDlgItemMessage(hwndDlg, (IDC_CHECK1 + wParam) as i32, STM_SETIMAGE, IMAGE_ICON, s_hArrowIcon as LPARAM);
        }

        PM_ITEM_END => {
            SendDlgItemMessage(hwndDlg, (IDC_TASKTEXT1 + wParam) as i32, WM_SETFONT, s_hNormalFont as WPARAM, TRUE as LPARAM);
            if lParam == (ERROR_SUCCESS) as i32 {
                SendDlgItemMessage(hwndDlg, (IDC_CHECK1 + wParam) as i32, STM_SETIMAGE, IMAGE_ICON, s_hCheckIcon as LPARAM);
            } else {
                SendDlgItemMessage(hwndDlg, (IDC_CHECK1 + wParam) as i32, STM_SETIMAGE, IMAGE_ICON, s_hCrossIcon as LPARAM);
                ShowItemError(hwndDlg, lParam as DWORD);
            }
        }

        PM_STEP_START => {
            RegistrationNotify = lParam as PREGISTRATIONNOTIFY;
            SendDlgItemMessage(
                hwndDlg,
                (IDC_ITEM) as i32,
                WM_SETTEXT,
                0,
                if (*RegistrationNotify).CurrentItem != core::ptr::null_mut() {
                    (*RegistrationNotify).CurrentItem
                } else {
                    w!("")
                } as LPARAM,
            );
        }

        PM_STEP_END => {
            RegistrationNotify = lParam as PREGISTRATIONNOTIFY;
            SendDlgItemMessage(hwndDlg, (IDC_PROCESSPROGRESS) as i32, PBM_SETPOS, (*RegistrationNotify).Progress, 0);
            if (*RegistrationNotify).LastError != ERROR_SUCCESS {
                ShowStepError(hwndDlg, RegistrationNotify);
            }
        }

        PM_ITEMS_DONE => {
            PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
            PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_NEXT);
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn ProcessPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut SetupData: PSETUPDATA;
    let mut RegistrationNotify: PREGISTRATIONNOTIFY;
    static mut s_hCheckIcon: HICON = core::ptr::null_mut();
    static mut s_hArrowIcon: HICON = core::ptr::null_mut();
    static mut s_hCrossIcon: HICON = core::ptr::null_mut();
    static mut s_hNormalFont: HFONT = core::ptr::null_mut();

    SetupData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;
            SetWindowLongPtr(hwndDlg, (DWLP_USER) as c_int, ((SetupData as DWORD_PTR)) as LONG_PTR);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_TASKTEXT3) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_TASKTEXT4) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_CHECK3) as i32), (SW_HIDE) as i32);
            ShowWindow(GetDlgItem(hwndDlg, (IDC_CHECK4) as i32), (SW_HIDE) as i32);
            s_hCheckIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_CHECKICON), (IMAGE_ICON) as u32, 16, 16, 0) as HICON;
            s_hArrowIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_ARROWICON), (IMAGE_ICON) as u32, 16, 16, 0) as HICON;
            s_hCrossIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_CROSSICON), (IMAGE_ICON) as u32, 16, 16, 0) as HICON;
            s_hNormalFont = SendDlgItemMessage(hwndDlg, (IDC_TASKTEXT1) as i32, (WM_GETFONT) as u32, 0, 0) as HFONT;
        }

        WM_DESTROY => {
            DestroyIcon(s_hCheckIcon);
            DestroyIcon(s_hArrowIcon);
            DestroyIcon(s_hCrossIcon);
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_SETACTIVE => {
                    LogItem!(w!("BEGIN") as LPCWSTR, w!("ProcessPage") as LPCWSTR);
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), 0);
                    RunItemCompletionThread(hwndDlg);
                }

                PSN_WIZNEXT => {
                    LogItem!(w!("END") as LPCWSTR, w!("ProcessPage") as LPCWSTR);
                }

                PSN_WIZBACK => {
                    (*SetupData).UnattendSetup = (FALSE) as i32;
                }

                _ => {}
            }
        }

        PM_ITEM_START => {
            SendDlgItemMessage(hwndDlg, (IDC_PROCESSPROGRESS) as i32, (PBM_SETRANGE) as u32, 0, MAKELPARAM!(0, lParam as ULONG));
            SendDlgItemMessage(hwndDlg, (IDC_PROCESSPROGRESS) as i32, (PBM_SETPOS) as u32, 0, 0);
            SendDlgItemMessage(hwndDlg, (IDC_TASKTEXT1 + ((wParam) as i32) as u32) as i32, (WM_SETFONT) as u32, (*SetupData).hBoldFont as WPARAM, TRUE as LPARAM);
            SendDlgItemMessage(hwndDlg, (IDC_CHECK1 + ((wParam) as i32) as u32) as i32, (STM_SETIMAGE) as u32, (IMAGE_ICON) as u64, s_hArrowIcon as LPARAM);
        }

        PM_ITEM_END => {
            SendDlgItemMessage(hwndDlg, (IDC_TASKTEXT1 + ((wParam) as i32) as u32) as i32, (WM_SETFONT) as u32, s_hNormalFont as WPARAM, TRUE as LPARAM);
            if lParam == (ERROR_SUCCESS) as i64 {
                SendDlgItemMessage(hwndDlg, (IDC_CHECK1 + ((wParam) as i32) as u32) as i32, (STM_SETIMAGE) as u32, (IMAGE_ICON) as u64, s_hCheckIcon as LPARAM);
            } else {
                SendDlgItemMessage(hwndDlg, (IDC_CHECK1 + ((wParam) as i32) as u32) as i32, (STM_SETIMAGE) as u32, (IMAGE_ICON) as u64, s_hCrossIcon as LPARAM);
                ShowItemError(hwndDlg, lParam as DWORD);
            }
        }

        PM_STEP_START => {
            RegistrationNotify = lParam as PREGISTRATIONNOTIFY;
            SendDlgItemMessage(
                hwndDlg,
                (IDC_ITEM) as i32,
                (WM_SETTEXT) as u32,
                0,
                if (*RegistrationNotify).CurrentItem != core::ptr::null_mut() {
                    (*RegistrationNotify).CurrentItem
                } else {
                    w!("")
                } as LPARAM,
            );
        }

        PM_STEP_END => {
            RegistrationNotify = lParam as PREGISTRATIONNOTIFY;
            SendDlgItemMessage(hwndDlg, (IDC_PROCESSPROGRESS) as i32, PBM_SETPOS, ((*RegistrationNotify).Progress) as u64, 0);
            if (*RegistrationNotify).LastError != ERROR_SUCCESS {
                ShowStepError(hwndDlg, RegistrationNotify);
            }
        }

        PM_ITEMS_DONE => {
            PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
            PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_NEXT);
        }

        _ => {}
    }

    (FALSE) as i64
}
unsafe extern "C" fn SetInstallationCompleted() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let InProgress: DWORD = 0;
    let mut InstallDate: DWORD;

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                       w!("SYSTEM\\Setup") as LPCWSTR,
                       0,
                       KEY_WRITE,
                       &mut hKey) == (ERROR_SUCCESS) as i32
    {
        RegSetValueExW(hKey, w!("SystemSetupInProgress") as LPCWSTR, 0, (REG_DWORD) as u32, (&InProgress as *const DWORD) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD);
        RegCloseKey(hKey);
    }

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                       w!("Software\\Microsoft\\Windows NT\\CurrentVersion") as LPCWSTR,
                       0,
                       KEY_WRITE,
                       &mut hKey) == (ERROR_SUCCESS) as i32
    {
        InstallDate = time(core::ptr::null_mut()) as DWORD;
        RegSetValueExW(hKey, w!("InstallDate") as LPCWSTR, 0, (REG_DWORD) as u32, (&InstallDate as *const DWORD) as LPBYTE, core::mem::size_of::<DWORD>() as DWORD);
        RegCloseKey(hKey);
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn FinishDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;

            if (*SetupData).UnattendSetup == 0 || (*SetupData).DisableGeckoInst == 0 {
                Control_RunDLLW(
                    hwndDlg,
                    core::ptr::null_mut(),
                    w!("appwiz.cpl,,install_gecko") as LPCWSTR,
                    SW_SHOW as i32,
                );
            }

            SendDlgItemMessage(
                hwndDlg,
                (IDC_FINISHTITLE) as i32,
                WM_SETFONT,
                (*SetupData).hTitleFont as WPARAM,
                TRUE as LPARAM,
            );
            if (*SetupData).UnattendSetup != 0 {
                KillTimer(hwndDlg, 1);
                SetInstallationCompleted();
                PostQuitMessage(0);
            }
        }

        WM_DESTROY => {
            SetInstallationCompleted();
            PostQuitMessage(0);
            return TRUE;
        }

        WM_TIMER => {
            let hWndProgress = GetDlgItem(hwndDlg, (IDC_RESTART_PROGRESS) as i32);
            let Position = SendMessage(
                hWndProgress,
                PBM_GETPOS,
                0,
                0,
            );
            if Position == 300 {
                KillTimer(hwndDlg, 1);
                PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_FINISH);
            } else {
                SendMessage(
                    hWndProgress,
                    PBM_SETPOS,
                    (Position + 1) as u32,
                    0,
                );
            }
            return TRUE;
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH);

                    SendDlgItemMessage(
                        hwndDlg,
                        (IDC_RESTART_PROGRESS) as i32,
                        PBM_SETRANGE,
                        0,
                        MAKELPARAM!(0, 300),
                    );
                    SendDlgItemMessage(
                        hwndDlg,
                        (IDC_RESTART_PROGRESS) as i32,
                        PBM_SETPOS,
                        0,
                        0,
                    );
                    SetTimer(
                        hwndDlg,
                        1,
                        50,
                        None,
                    );
                }

                PSN_WIZFINISH => {
                    DestroyWindow(GetParent(hwndDlg));
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn FinishDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let SetupData = (*(lParam as LPPROPSHEETPAGE)).lParam as PSETUPDATA;

            if (*SetupData).UnattendSetup == 0 || (*SetupData).DisableGeckoInst == 0 {
                Control_RunDLLW(
                    hwndDlg,
                    core::ptr::null_mut(),
                    w!("appwiz.cpl,,install_gecko") as LPCWSTR,
                    (SW_SHOW) as i32,
                );
            }

            SendDlgItemMessage(
                hwndDlg,
                (IDC_FINISHTITLE) as i32,
                (WM_SETFONT) as u32,
                (*SetupData).hTitleFont as WPARAM,
                TRUE as LPARAM,
            );
            if (*SetupData).UnattendSetup != 0 {
                KillTimer(hwndDlg, 1);
                SetInstallationCompleted();
                PostQuitMessage(0);
            }
        }

        WM_DESTROY => {
            SetInstallationCompleted();
            PostQuitMessage(0);
            return (TRUE) as i64;
        }

        WM_TIMER => {
            let hWndProgress = GetDlgItem(hwndDlg, (IDC_RESTART_PROGRESS) as i32);
            let Position = SendMessage(
                hWndProgress,
                (PBM_GETPOS) as u32,
                0,
                0,
            );
            if Position == 300 {
                KillTimer(hwndDlg, 1);
                PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_FINISH);
            } else {
                SendMessage(
                    hWndProgress,
                    (PBM_SETPOS) as u32,
                    (Position + 1) as u64,
                    0,
                );
            }
            return (TRUE) as i64;
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH);

                    SendDlgItemMessage(
                        hwndDlg,
                        (IDC_RESTART_PROGRESS) as i32,
                        (PBM_SETRANGE) as u32,
                        0,
                        MAKELPARAM!(0, 300),
                    );
                    SendDlgItemMessage(
                        hwndDlg,
                        (IDC_RESTART_PROGRESS) as i32,
                        (PBM_SETPOS) as u32,
                        0,
                        0,
                    );
                    SetTimer(
                        hwndDlg,
                        1,
                        50,
                        None,
                    );
                }

                PSN_WIZFINISH => {
                    DestroyWindow(GetParent(hwndDlg));
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as i64
}
unsafe extern "C" fn GetInstallSourceWin32(
    pwszPath: PWSTR,
    cchPathMax: DWORD,
    pwszNTPath: PCWSTR,
) -> BOOL {
    let mut wszDrives: [WCHAR; 512] = [0; 512];
    let mut wszNTPath: [WCHAR; 512] = [0; 512];
    let cchDrives: DWORD;
    let mut pwszDrive: PWSTR;

    *pwszPath = UNICODE_NULL;

    cchDrives = GetLogicalDriveStringsW(_countof!(wszDrives) - 1, wszDrives.as_mut_ptr());
    if cchDrives == 0 || cchDrives >= _countof!(wszDrives) {
        LogItem!(core::ptr::null_mut(), w!("GetLogicalDriveStringsW failed") as PCWSTR);
        return (FALSE) as i32;
    }

    pwszDrive = wszDrives.as_mut_ptr();
    while *pwszDrive != 0 {
        let mut wszBuf: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

        *pwszDrive.offset(2 as isize) = UNICODE_NULL;
        QueryDosDeviceW(pwszDrive, wszNTPath.as_mut_ptr(), _countof!(wszNTPath));
        *pwszDrive.offset(2 as isize) = '\\' as u16;

        wcscat(wszNTPath.as_mut_ptr(), w!("\\") as PCWSTR);

        wsprintf(
            wszBuf.as_mut_ptr(),
            w!("Testing '%s' --> '%s' %s a CD") as PCWSTR,
            pwszDrive,
            wszNTPath.as_ptr(),
            if GetDriveTypeW(pwszDrive) == (DRIVE_CDROM) as u32 {
                w!("is") as PCWSTR
            } else {
                w!("is not") as PCWSTR
            },
        );
        LogItem!(core::ptr::null_mut(), wszBuf.as_ptr());

        if _wcsicmp(wszNTPath.as_ptr(), pwszNTPath) == 0 {
            wcscpy(pwszPath, pwszDrive);

            wsprintf(wszBuf.as_mut_ptr(), w!("GetInstallSourceWin32: %s") as PCWSTR, pwszPath);
            LogItem!(core::ptr::null_mut(), wszBuf.as_ptr());
            wcscat(wszBuf.as_mut_ptr(), w!("\n") as PCWSTR);
            OutputDebugStringW(wszBuf.as_ptr());

            return (TRUE) as i32;
        }

        pwszDrive = pwszDrive.offset(wcslen(pwszDrive) as isize + 1 as isize);
    }

    return (FALSE) as i32;
}
/// Whether `HARDWARE\DEVICEMAP\VIDEO` points at a device of the service, i.e. whether its
/// driver is the one running now.
unsafe fn IsDisplayServiceActive(pszServiceName: &[u16]) -> bool {
    let mut hKey: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("HARDWARE\\DEVICEMAP\\VIDEO") as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) != ERROR_SUCCESS as LONG
    {
        return false;
    }

    // The DEVICEMAP entries point to Services\<ServiceName>\DeviceX.
    let mut pattern = [0u16; MAX_PATH as usize];
    let mut n = 0usize;
    for c in "\\SERVICES\\".encode_utf16().chain(pszServiceName.iter().copied()).chain([b'\\' as u16]) {
        if n == pattern.len() - 1 {
            break; // _snwprintf truncates the same way
        }
        pattern[n] = c;
        n += 1;
    }
    let pattern = &mut pattern[..n];
    CharUpperBuffW(pattern.as_mut_ptr(), pattern.len() as DWORD);

    let mut bActive = false;
    let mut szValueName = [0u16; 64];
    let mut szData = [0u16; MAX_PATH as usize];
    let mut dwIndex: DWORD = 0;
    while !bActive {
        let mut cchValueName = szValueName.len() as DWORD;
        let mut cbData = (core::mem::size_of_val(&szData) - core::mem::size_of::<u16>()) as DWORD;
        let mut dwType: DWORD = 0;
        if RegEnumValueW(
            hKey,
            dwIndex,
            szValueName.as_mut_ptr(),
            &mut cchValueName,
            core::ptr::null_mut(),
            &mut dwType,
            szData.as_mut_ptr() as LPBYTE,
            &mut cbData,
        ) == ERROR_NO_MORE_ITEMS as LONG
        {
            break;
        }
        dwIndex += 1;
        if dwType != REG_SZ {
            continue;
        }
        let data = &mut szData[..cbData as usize / 2];
        CharUpperBuffW(data.as_mut_ptr(), data.len() as DWORD);
        bActive = data.windows(pattern.len()).any(|w| w == &pattern[..]);
    }

    RegCloseKey(hKey);
    bActive
}

/// Store `pdm`'s unattended fields as the DefaultSettings of every DeviceN key of the service.
unsafe fn WriteDisplaySettingsToService(pszServiceName: LPCWSTR, pdm: &DEVMODEW) {
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SYSTEM\\CurrentControlSet\\Services") as LPCWSTR,
        0,
        KEY_ENUMERATE_SUB_KEYS,
        &mut hServicesKey,
    ) != ERROR_SUCCESS as LONG
    {
        return;
    }
    let opened = RegOpenKeyExW(hServicesKey, pszServiceName, 0, KEY_ENUMERATE_SUB_KEYS, &mut hServiceKey);
    RegCloseKey(hServicesKey);
    if opened != ERROR_SUCCESS as LONG {
        return;
    }

    const DEVICE: &[u8] = b"Device";
    let mut szDeviceName = [0u16; MAX_PATH as usize];
    let mut dwIndex: DWORD = 0;
    loop {
        let mut cchDeviceName = szDeviceName.len() as DWORD;
        if RegEnumKeyExW(
            hServiceKey,
            dwIndex,
            szDeviceName.as_mut_ptr(),
            &mut cchDeviceName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) != ERROR_SUCCESS as LONG
        {
            break;
        }
        dwIndex += 1;

        // _wcsnicmp(szDeviceName, L"Device", 6)
        let name = &szDeviceName[..cchDeviceName as usize];
        if name.len() < DEVICE.len()
            || !name[..DEVICE.len()]
                .iter()
                .zip(DEVICE)
                .all(|(&a, b)| a < 0x80 && (a as u8).eq_ignore_ascii_case(b))
        {
            continue;
        }

        let mut hDeviceKey: HKEY = core::ptr::null_mut();
        if RegOpenKeyExW(hServiceKey, szDeviceName.as_ptr(), 0, KEY_SET_VALUE, &mut hDeviceKey)
            != ERROR_SUCCESS as LONG
        {
            continue;
        }

        let set = |value: LPCWSTR, data: &DWORD| {
            RegSetValueExW(
                hDeviceKey,
                value,
                0,
                REG_DWORD,
                data as *const DWORD as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        };
        if pdm.dmFields & DM_BITSPERPEL as u32 != 0 {
            set(w!("DefaultSettings.BitsPerPel") as LPCWSTR, &pdm.dmBitsPerPel);
        }
        if pdm.dmFields & DM_PELSWIDTH as u32 != 0 {
            set(w!("DefaultSettings.XResolution") as LPCWSTR, &pdm.dmPelsWidth);
        }
        if pdm.dmFields & DM_PELSHEIGHT as u32 != 0 {
            set(w!("DefaultSettings.YResolution") as LPCWSTR, &pdm.dmPelsHeight);
        }
        if pdm.dmFields & DM_DISPLAYFREQUENCY as u32 != 0 {
            set(w!("DefaultSettings.VRefresh") as LPCWSTR, &pdm.dmDisplayFrequency);
        }

        RegCloseKey(hDeviceKey);
    }

    RegCloseKey(hServiceKey);
}

/// Display drivers installed during the second stage (e.g. from modules/optional) only
/// become active after a reboot, so ChangeDisplaySettings cannot reach them. Store the
/// unattended settings in their registry keys, for the next boot (upstream b2116d022fc).
unsafe fn ApplyDisplaySettingsToInactiveAdapters(pdm: &DEVMODEW) {
    if pdm.dmFields == 0 {
        return;
    }

    let mut ClassGuid: GUID = core::mem::zeroed();
    let mut dwGuids: DWORD = 0;
    if SetupDiClassGuidsFromNameW(w!("Display") as LPCWSTR, &mut ClassGuid, 1, &mut dwGuids) == 0
        || dwGuids == 0
    {
        return;
    }

    let hDevInfo = SetupDiGetClassDevsW(&ClassGuid, core::ptr::null(), core::ptr::null_mut(), DIGCF_PRESENT);
    if hDevInfo == INVALID_HANDLE_VALUE as HDEVINFO {
        return;
    }

    let mut DevInfoData: SP_DEVINFO_DATA = core::mem::zeroed();
    DevInfoData.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;
    let mut szServiceName = [0u16; MAX_PATH as usize];
    let mut dwIndex: DWORD = 0;
    while SetupDiEnumDeviceInfo(hDevInfo, dwIndex, &mut DevInfoData) != 0 {
        dwIndex += 1;
        if SetupDiGetDeviceRegistryPropertyW(
            hDevInfo,
            &mut DevInfoData,
            SPDRP_SERVICE,
            core::ptr::null_mut(),
            szServiceName.as_mut_ptr() as PBYTE,
            core::mem::size_of_val(&szServiceName) as DWORD,
            core::ptr::null_mut(),
        ) == 0
        {
            continue;
        }

        // The settings of an active adapter were already updated by ChangeDisplaySettings.
        let len = szServiceName.iter().position(|&c| c == 0).unwrap_or(szServiceName.len());
        if IsDisplayServiceActive(&szServiceName[..len]) {
            continue;
        }

        WriteDisplaySettingsToService(szServiceName.as_ptr(), pdm);
    }

    SetupDiDestroyDeviceInfoList(hDevInfo);
}

#[no_mangle]
pub unsafe extern "C" fn ProcessUnattendSection(pSetupData: PSETUPDATA) {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut szName: [WCHAR; 256] = [0; 256];
    let mut szValue: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut LineLength: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    if SetupFindFirstLineW(
        (*pSetupData).hSetupInf,
        w!("Unattend") as LPCWSTR,
        w!("UnattendSetupEnabled") as LPCWSTR,
        &mut InfContext,
    ) == 0
    {
        // DPRINT removed
        return;
    }

    if SetupGetStringFieldW(
        &mut InfContext,
        1,
        szValue.as_mut_ptr(),
        ARRAYSIZE!(szValue),
        &mut LineLength,
    ) == 0
    {
        // DPRINT removed
        return;
    }

    if _wcsicmp(szValue.as_ptr(), w!("yes") as LPCWSTR)
        != 0
    {
        // DPRINT removed
        return;
    }

    (*pSetupData).UnattendSetup = (TRUE) as i32;

    if SetupFindFirstLineW(
        (*pSetupData).hSetupInf,
        w!("Unattend") as LPCWSTR,
        core::ptr::null_mut(),
        &mut InfContext,
    ) == 0
    {
        // DPRINT removed
        return;
    }

    loop {
        if SetupGetStringFieldW(
            &mut InfContext,
            0,
            szName.as_mut_ptr(),
            ARRAYSIZE!(szName),
            &mut LineLength,
        ) == 0
        {
            // DPRINT removed
            return;
        }

        if SetupGetStringFieldW(
            &mut InfContext,
            1,
            szValue.as_mut_ptr(),
            ARRAYSIZE!(szValue),
            &mut LineLength,
        ) == 0
        {
            // DPRINT removed
            return;
        }
        // DPRINT removed
        if _wcsicmp(szName.as_ptr(), w!("FullName") as LPCWSTR) == 0 {
            if ARRAYSIZE!((*pSetupData).OwnerName) > LineLength {
                wcscpy((*pSetupData).OwnerName.as_mut_ptr(), szValue.as_ptr());
            }
        } else if _wcsicmp(szName.as_ptr(), w!("OrgName") as LPCWSTR) == 0 {
            if ARRAYSIZE!((*pSetupData).OwnerOrganization) > LineLength {
                wcscpy((*pSetupData).OwnerOrganization.as_mut_ptr(), szValue.as_ptr());
            }
        } else if _wcsicmp(szName.as_ptr(), w!("ComputerName") as LPCWSTR)
            == 0
        {
            if ARRAYSIZE!((*pSetupData).ComputerName) > LineLength {
                wcscpy((*pSetupData).ComputerName.as_mut_ptr(), szValue.as_ptr());
            }
        } else if _wcsicmp(szName.as_ptr(), w!("AdminPassword") as LPCWSTR)
            == 0
        {
            if ARRAYSIZE!((*pSetupData).AdminPassword) > LineLength {
                wcscpy((*pSetupData).AdminPassword.as_mut_ptr(), szValue.as_ptr());
            }
        } else if _wcsicmp(szName.as_ptr(), w!("TimeZoneIndex") as LPCWSTR)
            == 0
        {
            (*pSetupData).TimeZoneIndex = (_wtoi(szValue.as_ptr())) as u32;
        } else if _wcsicmp(
            szName.as_ptr(),
            w!("DisableAutoDaylightTimeSet") as LPCWSTR,
        ) == 0
        {
            (*pSetupData).DisableAutoDaylightTimeSet = (_wtoi(szValue.as_ptr())) as u32;
        } else if _wcsicmp(szName.as_ptr(), w!("DisableGeckoInst") as LPCWSTR)
            == 0
        {
            if _wcsicmp(szValue.as_ptr(), w!("yes") as LPCWSTR) == 0 {
                (*pSetupData).DisableGeckoInst = (TRUE) as i32;
            } else {
                (*pSetupData).DisableGeckoInst = (FALSE) as i32;
            }
        } else if _wcsicmp(szName.as_ptr(), w!("ProductOption") as LPCWSTR)
            == 0
        {
            (*pSetupData).ProductOption = _wtoi(szValue.as_ptr()) as PRODUCT_OPTION;
        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }

    if SetupFindFirstLineW(
        (*pSetupData).hSetupInf,
        w!("Display") as LPCWSTR,
        core::ptr::null_mut(),
        &mut InfContext,
    ) != 0
    {
        let mut dm: DEVMODEW = core::mem::zeroed();
        let mut dmUnattend: DEVMODEW = core::mem::zeroed();
        dm.dmSize = core::mem::size_of::<DEVMODEW>() as WORD;
        dmUnattend.dmSize = core::mem::size_of::<DEVMODEW>() as WORD;
        if EnumDisplaySettingsW(
            core::ptr::null_mut(),
            ENUM_CURRENT_SETTINGS,
            &mut dm,
        ) != 0
        {
            loop {
                let mut iValue: i32;
                if SetupGetStringFieldW(
                    &mut InfContext,
                    0,
                    szName.as_mut_ptr(),
                    ARRAYSIZE!(szName),
                    &mut LineLength,
                ) == 0
                {
                    // DPRINT removed
                    return;
                }

                if SetupGetStringFieldW(
                    &mut InfContext,
                    1,
                    szValue.as_mut_ptr(),
                    ARRAYSIZE!(szValue),
                    &mut LineLength,
                ) == 0
                {
                    // DPRINT removed
                    return;
                }
                iValue = _wtoi(szValue.as_ptr());
                // DPRINT removed

                if iValue != 0 {
                    if _wcsicmp(szName.as_ptr(), w!("BitsPerPel") as LPCWSTR)
                        == 0
                    {
                        dmUnattend.dmFields |= (DM_BITSPERPEL) as u32;
                        dmUnattend.dmBitsPerPel = iValue as DWORD;
                    } else if _wcsicmp(
                        szName.as_ptr(),
                        w!("XResolution") as LPCWSTR,
                    ) == 0
                    {
                        dmUnattend.dmFields |= (DM_PELSWIDTH) as u32;
                        dmUnattend.dmPelsWidth = iValue as DWORD;
                    } else if _wcsicmp(
                        szName.as_ptr(),
                        w!("YResolution") as LPCWSTR,
                    ) == 0
                    {
                        dmUnattend.dmFields |= (DM_PELSHEIGHT) as u32;
                        dmUnattend.dmPelsHeight = iValue as DWORD;
                    } else if _wcsicmp(
                        szName.as_ptr(),
                        w!("VRefresh") as LPCWSTR,
                    ) == 0
                    {
                        dmUnattend.dmFields |= (DM_DISPLAYFREQUENCY) as u32;
                        dmUnattend.dmDisplayFrequency = iValue as DWORD;
                    }
                }

                if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
                    break;
                }
            }

            dm.dmFields |= dmUnattend.dmFields;
            if dmUnattend.dmFields & DM_BITSPERPEL as u32 != 0 {
                dm.dmBitsPerPel = dmUnattend.dmBitsPerPel;
            }
            if dmUnattend.dmFields & DM_PELSWIDTH as u32 != 0 {
                dm.dmPelsWidth = dmUnattend.dmPelsWidth;
            }
            if dmUnattend.dmFields & DM_PELSHEIGHT as u32 != 0 {
                dm.dmPelsHeight = dmUnattend.dmPelsHeight;
            }
            if dmUnattend.dmFields & DM_DISPLAYFREQUENCY as u32 != 0 {
                dm.dmDisplayFrequency = dmUnattend.dmDisplayFrequency;
            }

            ChangeDisplaySettingsW(&mut dm, (CDS_UPDATEREGISTRY) as u32);
            ApplyDisplaySettingsToInactiveAdapters(&dmUnattend);
        }
    }

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce") as LPCWSTR,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    ) != (ERROR_SUCCESS) as i32
    {
        // DPRINT removed
        return;
    }

    if SetupFindFirstLineW(
        (*pSetupData).hSetupInf,
        w!("GuiRunOnce") as LPCWSTR,
        core::ptr::null_mut(),
        &mut InfContext,
    ) != 0
    {
        let mut i: i32 = 0;
        loop {
            if SetupGetStringFieldW(
                &mut InfContext,
                0,
                szValue.as_mut_ptr(),
                ARRAYSIZE!(szValue),
                core::ptr::null_mut(),
            ) != 0
            {
                let mut szPath: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
                // A WIDE format: this used to pass the bytes b"%d\0" reinterpreted as
                // UTF-16 (U+6425 then whatever byte followed in .rdata), so every GuiRunOnce
                // entry was written under the same junk value name and only the last survived.
                swprintf(szName.as_mut_ptr(), w!("%d") as LPCWSTR, i);
                // DPRINT removed

                if ExpandEnvironmentStringsW(
                    szValue.as_ptr(),
                    szPath.as_mut_ptr(),
                    MAX_PATH,
                ) != 0
                {
                    // DPRINT removed
                    if RegSetValueExW(
                        hKey,
                        szName.as_ptr(),
                        0,
                        (REG_SZ) as u32,
                        szPath.as_ptr() as *const BYTE,
                        ((wcslen(szPath.as_ptr()) + 1)
                            * (core::mem::size_of::<WCHAR>() as DWORD) as usize) as u32,
                    ) == (ERROR_SUCCESS) as i32
                    {
                        i += 1;
                    }
                }
            }

            if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
                break;
            }
        }
    }

    RegCloseKey(hKey);

    if SetupFindFirstLineW(
        (*pSetupData).hSetupInf,
        w!("Env") as LPCWSTR,
        core::ptr::null_mut(),
        &mut InfContext,
    ) != 0
    {
        if RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment") as LPCWSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE | KEY_READ,
            core::ptr::null_mut(),
            &mut hKey,
            core::ptr::null_mut(),
        ) != (ERROR_SUCCESS) as i32
        {
            // DPRINT removed
            return;
        }
        loop {
            if SetupGetStringFieldW(
                &mut InfContext,
                0,
                szName.as_mut_ptr(),
                ARRAYSIZE!(szName),
                &mut LineLength,
            ) == 0
            {
                // DPRINT removed
                return;
            }

            if SetupGetStringFieldW(
                &mut InfContext,
                1,
                szValue.as_mut_ptr(),
                ARRAYSIZE!(szValue),
                &mut LineLength,
            ) == 0
            {
                // DPRINT removed
                return;
            }
            // DPRINT removed

            let dwType: DWORD = if wcschr(szValue.as_ptr(), '%' as u16)
                != core::ptr::null_mut()
            {
                (REG_EXPAND_SZ) as u32
            } else {
                (REG_SZ) as u32
            };

            if RegSetValueExW(
                hKey,
                szName.as_ptr(),
                0,
                dwType,
                szValue.as_ptr() as *const BYTE,
                ((wcslen(szValue.as_ptr()) + 1)
                    * (core::mem::size_of::<TCHAR>() as DWORD) as usize) as u32,
            ) != (ERROR_SUCCESS) as i32
            {
                // DPRINT removed
            }

            if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
                break;
            }
        }

        RegCloseKey(hKey);
    }
}
unsafe extern "C" fn PathIsEqual(
    lpPath1: LPCWSTR,
    lpPath2: LPCWSTR,
) -> BOOL {
    let mut szPath1: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szPath2: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    if PathCanonicalizeW(szPath1.as_mut_ptr(), lpPath1) == 0 {
        return (TRUE) as i32;
    }

    if PathAddBackslashW(szPath1.as_mut_ptr()).is_null() {
        return (TRUE) as i32;
    }

    if PathCanonicalizeW(szPath2.as_mut_ptr(), lpPath2) == 0 {
        return (TRUE) as i32;
    }

    if PathAddBackslashW(szPath2.as_mut_ptr()).is_null() {
        return (TRUE) as i32;
    }

    if _wcsicmp(szPath1.as_ptr(), szPath2.as_ptr()) == 0 {
        1
    } else {
        0
    }
}
unsafe extern "C" fn AddInstallationSource(
    hKey: HKEY,
    lpPath: LPWSTR,
) {
    let mut res: LONG;
    let mut dwRegType: DWORD = 0;
    let mut dwPathLength: DWORD = 0;
    let mut dwNewLength: DWORD = 0;
    let mut Buffer: LPWSTR = core::ptr::null_mut();
    let mut Path: LPWSTR = core::ptr::null_mut();

    res = RegQueryValueExW(
        hKey,
        w!("Installation Sources") as *const u16,
        core::ptr::null_mut(),
        &mut dwRegType,
        core::ptr::null_mut(),
        &mut dwPathLength,
    );

    if res != (ERROR_SUCCESS) as i32
        || dwRegType != (REG_MULTI_SZ) as u32
        || dwPathLength == 0
        || dwPathLength % core::mem::size_of::<WCHAR>() as u32 != 0
    {
        dwPathLength = 0;
        'set: loop {
            if dwPathLength == 0 {
                dwNewLength = ((wcslen(lpPath) + 1 + 1)
                    * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32;
                Buffer = (HeapAlloc(GetProcessHeap(), 0, (dwNewLength) as SIZE_T)).cast();
                if Buffer.is_null() {
                    return;
                }

                Path = Buffer;
            }

            StringCbCopyW(
                Path,
                (dwNewLength.wrapping_sub((Path.offset_from(Buffer) as u32) * core::mem::size_of::<WCHAR>() as u32)) as usize,
                lpPath,
            );
            *Buffer.offset((dwNewLength / core::mem::size_of::<WCHAR>() as u32 - 1) as isize) =
                UNICODE_NULL;

            RegSetValueExW(
                hKey,
                w!("Installation Sources") as *const u16,
                0,
                (REG_MULTI_SZ) as u32,
                Buffer as LPBYTE,
                dwNewLength,
            );

            break 'set;
        }
        HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
        return;
    }

    dwNewLength = dwPathLength + ((wcslen(lpPath) + 1) * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32;
    Buffer = (HeapAlloc(GetProcessHeap(), 0, (dwNewLength) as SIZE_T)).cast();
    if Buffer.is_null() {
        return;
    }

    ZeroMemory!(Buffer, dwNewLength);

    res = RegQueryValueExW(
        hKey,
        w!("Installation Sources") as *const u16,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Buffer as LPBYTE,
        &mut dwPathLength,
    );

    if res != (ERROR_SUCCESS) as i32 {
        HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
        dwPathLength = 0;
        'set2: loop {
            if dwPathLength == 0 {
                dwNewLength = ((wcslen(lpPath) + 1 + 1)
                    * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32;
                Buffer = (HeapAlloc(GetProcessHeap(), 0, (dwNewLength) as SIZE_T)).cast();
                if Buffer.is_null() {
                    return;
                }

                Path = Buffer;
            }

            StringCbCopyW(
                Path,
                (dwNewLength.wrapping_sub((Path.offset_from(Buffer) as u32) * core::mem::size_of::<WCHAR>() as u32)) as usize,
                lpPath,
            );
            *Buffer.offset((dwNewLength / core::mem::size_of::<WCHAR>() as u32 - 1) as isize) =
                UNICODE_NULL;

            RegSetValueExW(
                hKey,
                w!("Installation Sources") as *const u16,
                0,
                (REG_MULTI_SZ) as u32,
                Buffer as LPBYTE,
                dwNewLength,
            );

            break 'set2;
        }
        HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
        return;
    }

    *Buffer.offset((dwPathLength / core::mem::size_of::<WCHAR>() as u32 - 2) as isize) =
        UNICODE_NULL;
    *Buffer.offset((dwPathLength / core::mem::size_of::<WCHAR>() as u32 - 1) as isize) =
        UNICODE_NULL;

    Path = Buffer;
    while *Path != 0 {
        if PathIsEqual(Path, lpPath) != 0 {
            HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
            return;
        }
        Path = Path.offset((wcslen(Path) + 1) as isize);
    }

    Path = Buffer.offset((dwPathLength / core::mem::size_of::<WCHAR>() as u32 - 1) as isize);

    'set3: loop {
        if dwPathLength == 0 {
            dwNewLength = ((wcslen(lpPath) + 1 + 1)
                * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32;
            Buffer = (HeapAlloc(GetProcessHeap(), 0, (dwNewLength) as SIZE_T)).cast();
            if Buffer.is_null() {
                return;
            }

            Path = Buffer;
        }

        StringCbCopyW(
            Path,
            (dwNewLength.wrapping_sub((Path.offset_from(Buffer) as u32) * core::mem::size_of::<WCHAR>() as u32)) as usize,
            lpPath,
        );
        *Buffer.offset((dwNewLength / core::mem::size_of::<WCHAR>() as u32 - 1) as isize) =
            UNICODE_NULL;

        RegSetValueExW(
            hKey,
            w!("Installation Sources") as *const u16,
            0,
            (REG_MULTI_SZ) as u32,
            Buffer as LPBYTE,
            dwNewLength,
        );

        break 'set3;
    }

    HeapFree(GetProcessHeap(), 0, Buffer as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ProcessSetupInf(
    pSetupData: PSETUPDATA,
) {
    let mut szPath: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szValue: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut LineLength: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut res: LONG = 0;

    (*pSetupData).hSetupInf = INVALID_HANDLE_VALUE;

    GetSystemDirectoryW(szPath.as_mut_ptr(), _countof!(szPath));
    wcscat(szPath.as_mut_ptr(), w!("\\$winnt$.inf") as LPCWSTR);

    (*pSetupData).hSetupInf = SetupOpenInfFileW(
        szPath.as_ptr(),
        core::ptr::null(),
        (INF_STYLE_OLDNT) as u32,
        core::ptr::null_mut(),
    );
    if (*pSetupData).hSetupInf == INVALID_HANDLE_VALUE {
        // DPRINT removed
        return;
    }

    if SetupFindFirstLineW(
        (*pSetupData).hSetupInf,
        w!("data") as LPCWSTR,
        w!("sourcepath") as LPCWSTR,
        &mut InfContext,
    ) == 0
    {
        // DPRINT removed
        return;
    }

    if SetupGetStringFieldW(
        &mut InfContext,
        1,
        szValue.as_mut_ptr(),
        ARRAYSIZE!(szValue),
        &mut LineLength,
    ) == 0
    {
        // DPRINT removed
        return;
    }

    *(*pSetupData).SourcePath.as_mut_ptr() = UNICODE_NULL;

    if (*pSetupData).hSetupInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile((*pSetupData).hSetupInf);
    }

    if GetInstallSourceWin32(
        (*pSetupData).SourcePath.as_mut_ptr(),
        _countof!((*pSetupData).SourcePath),
        szValue.as_ptr(),
    ) == 0
    {
        *(*pSetupData).SourcePath.as_mut_ptr() = UNICODE_NULL;
    }

    swprintf(
        szValue.as_mut_ptr(),
        w!("\"%s\"") as LPCWSTR,
        (*pSetupData).SourcePath.as_ptr(),
    );
    WritePrivateProfileStringW(
        w!("data") as LPCWSTR,
        w!("dospath") as LPCWSTR,
        szValue.as_ptr(),
        szPath.as_ptr(),
    );

    res = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Setup") as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    );
    if res == (ERROR_SUCCESS) as i32 {
        AddInstallationSource(hKey, (*pSetupData).SourcePath.as_mut_ptr());

        res = RegSetValueExW(
            hKey,
            w!("SourcePath") as LPCWSTR,
            0,
            (REG_SZ) as u32,
            (*pSetupData).SourcePath.as_ptr() as LPBYTE,
            ((wcslen((*pSetupData).SourcePath.as_ptr()) + 1)
                * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
        );

        res = RegSetValueExW(
            hKey,
            w!("ServicePackSourcePath") as LPCWSTR,
            0,
            (REG_SZ) as u32,
            (*pSetupData).SourcePath.as_ptr() as LPBYTE,
            ((wcslen((*pSetupData).SourcePath.as_ptr()) + 1)
                * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
        );

        RegCloseKey(hKey);
    }

    (*pSetupData).hSetupInf = SetupOpenInfFileW(
        szPath.as_ptr(),
        core::ptr::null(),
        (INF_STYLE_OLDNT) as u32,
        core::ptr::null_mut(),
    );
    if (*pSetupData).hSetupInf == INVALID_HANDLE_VALUE {
        // DPRINT removed
        return;
    }

    ProcessUnattendSection(pSetupData);
}
#[no_mangle]
pub unsafe extern "C" fn InstallWizard() {
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut phpage: *mut HPROPSHEETPAGE = core::ptr::null_mut();
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();
    let mut nPages: UINT = 0;
    let hWnd: HWND;
    let mut msg: MSG = core::mem::zeroed();
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();
    let mut hNetShell: HMODULE = core::ptr::null_mut();
    let mut pfn: PFNREQUESTWIZARDPAGES = None;
    let mut dwPageCount: DWORD = 10;
    let mut dwNetworkPageCount: DWORD = 0;

    LogItem!(
        w!("BEGIN_SECTION") as LPWSTR,
        w!("InstallWizard") as LPWSTR,
    );

    pSetupData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<SETUPDATA>()) as SIZE_T,
    ) as PSETUPDATA;
    if pSetupData.is_null() {
        LogItem!(
            core::ptr::null_mut(),
            w!("SetupData allocation failed!") as LPWSTR,
        );
        MessageBoxW(
            core::ptr::null_mut(),
            w!("Setup failed to allocate global data!") as LPWSTR,
            w!("ReactOS Setup") as LPWSTR,
            (MB_ICONERROR | MB_OK) as u32,
        );
        'done: loop {
            if !phpage.is_null() {
                HeapFree(GetProcessHeap(), 0, phpage as LPVOID);
            }
            if !hNetShell.is_null() {
                FreeLibrary(hNetShell);
            }
            if !pSetupData.is_null() {
                HeapFree(GetProcessHeap(), 0, pSetupData as LPVOID);
            }
            LogItem!(
                w!("END_SECTION") as LPWSTR,
                w!("InstallWizard") as LPWSTR,
            );
            break;
        }
        return;
    }
    (*pSetupData).ProductOption = PRODUCT_OPTION_DEFAULT;

    hNetShell = LoadLibraryW(w!("netshell.dll") as LPCWSTR);
    if !hNetShell.is_null() {
        // DPRINT removed

        pfn = core::mem::transmute(GetProcAddress(
            hNetShell,
            b"NetSetupRequestWizardPages\0".as_ptr() as PCHAR,
        ));
        if let Some(pfn) = pfn {
            pfn(&mut dwNetworkPageCount, core::ptr::null_mut(), core::ptr::null_mut());
            dwPageCount += dwNetworkPageCount;
        }
    }

    // DPRINT removed

    phpage = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (dwPageCount * core::mem::size_of::<HPROPSHEETPAGE>() as DWORD) as SIZE_T,
    ) as *mut HPROPSHEETPAGE;
    if phpage.is_null() {
        LogItem!(
            core::ptr::null_mut(),
            w!("Page array allocation failed!") as LPWSTR,
        );
        MessageBoxW(
            core::ptr::null_mut(),
            w!("Setup failed to allocate page array!") as LPWSTR,
            w!("ReactOS Setup") as LPWSTR,
            (MB_ICONERROR | MB_OK) as u32,
        );
        'done2: loop {
            if !phpage.is_null() {
                HeapFree(GetProcessHeap(), 0, phpage as LPVOID);
            }
            if !hNetShell.is_null() {
                FreeLibrary(hNetShell);
            }
            if !pSetupData.is_null() {
                HeapFree(GetProcessHeap(), 0, pSetupData as LPVOID);
            }
            LogItem!(
                w!("END_SECTION") as LPWSTR,
                w!("InstallWizard") as LPWSTR,
            );
            break;
        }
        return;
    }

    ProcessSetupInf(pSetupData);

    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER) as u32;
    psp.hInstance = hDllInstance;
    psp.lParam = pSetupData as LPARAM;
    psp.pfnDlgProc = Some(WelcomeDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_WELCOMEPAGE);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_ACKTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_ACKSUBTITLE);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_ACKPAGE);
    psp.pfnDlgProc = Some(AckPageDlgProc);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_PRODUCTTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_PRODUCTSUBTITLE);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PRODUCT);
    psp.pfnDlgProc = Some(ProductPageDlgProc);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_LOCALETITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_LOCALESUBTITLE);
    psp.pfnDlgProc = Some(LocalePageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_LOCALEPAGE);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_OWNERTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_OWNERSUBTITLE);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_OWNERPAGE);
    psp.pfnDlgProc = Some(OwnerPageDlgProc);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_COMPUTERTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_COMPUTERSUBTITLE);
    psp.pfnDlgProc = Some(ComputerPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_COMPUTERPAGE);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_DATETIMETITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_DATETIMESUBTITLE);
    psp.pfnDlgProc = Some(DateTimePageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_DATETIMEPAGE);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_THEMESELECTIONTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_THEMESELECTIONSUBTITLE);
    psp.pfnDlgProc = Some(ThemePageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_THEMEPAGE);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    (*pSetupData).uFirstNetworkWizardPage = (IDD_PROCESSPAGE) as u32;
    (*pSetupData).uPostNetworkWizardPage = (IDD_PROCESSPAGE) as u32;

    if let Some(pfn) = pfn {
        pfn(&mut dwNetworkPageCount, phpage.offset(nPages as isize), pSetupData);
        nPages += dwNetworkPageCount;
    }

    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as u32;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_PROCESSTITLE);
    psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_PROCESSSUBTITLE);
    psp.pfnDlgProc = Some(ProcessPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PROCESSPAGE);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER) as u32;
    psp.pfnDlgProc = Some(FinishDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_FINISHPAGE);
    *phpage.offset(nPages as isize) = CreatePropertySheetPage(&psp);
    nPages += 1;

    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_WIZARD97 | PSH_WATERMARK | PSH_HEADER | PSH_MODELESS) as u32;
    psh.hInstance = hDllInstance;
    psh.hwndParent = core::ptr::null_mut();
    psh.nPages = nPages;
    psh.__bindgen_anon_2.nStartPage = 0;
    psh.__bindgen_anon_3.phpage = phpage;
    psh.__bindgen_anon_4.pszbmWatermark = MAKEINTRESOURCE!(IDB_WATERMARK);
    psh.__bindgen_anon_5.pszbmHeader = MAKEINTRESOURCE!(IDB_HEADER);

    (*pSetupData).hTitleFont = CreateTitleFont();
    (*pSetupData).hBoldFont = CreateBoldFont();

    hWnd = PropertySheet(&psh) as HWND;
    ShowWindow(hWnd, (SW_SHOW) as i32);

    while GetMessage(&mut msg, core::ptr::null_mut(), 0, 0) != 0 {
        if IsDialogMessage(hWnd, &mut msg) == 0 {
            TranslateMessage(&mut msg);
            DispatchMessage(&mut msg);
        }
    }

    DeleteObject(((*pSetupData).hBoldFont).cast());
    DeleteObject(((*pSetupData).hTitleFont).cast());

    if (*pSetupData).hSetupInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile((*pSetupData).hSetupInf);
    }

    if !phpage.is_null() {
        HeapFree(GetProcessHeap(), 0, phpage as LPVOID);
    }

    if !hNetShell.is_null() {
        FreeLibrary(hNetShell);
    }

    if !pSetupData.is_null() {
        HeapFree(GetProcessHeap(), 0, pSetupData as LPVOID);
    }

    LogItem!(
        w!("END_SECTION") as LPWSTR,
        w!("InstallWizard") as LPWSTR,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         System setup
//  * FILE:            dll/win32/syssetup/wizard.c
//  * PURPOSE:         GUI controls
//  * PROGRAMMERS:     Eric Kohl
//  *                  Pierre Schweitzer <heis_spiter@hotmail.com>
//  *                  Ismael Ferreras Morezuelas <swyterzone+ros@gmail.com>
//  *                  Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  *                  Oleg Dubinskiy <oleg.dubinskij30@gmail.com>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #include <stdlib.h>
// #include <time.h>
// #include <winnls.h>
// #include <windowsx.h>
// #include <wincon.h>
// #include <shlobj.h>
// #include <shlwapi.h>
// #include <tzlib.h>
// #include <strsafe.h>
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _REGISTRATIONDATA
// {
//     HWND hwndDlg;
//     ULONG DllCount;
//     ULONG Registered;
//     PVOID DefaultContext;
//     PREGISTRATIONNOTIFY pNotify;
// } REGISTRATIONDATA, *PREGISTRATIONDATA;
//
// typedef struct _TIMEZONE_ENTRY
// {
//     struct _TIMEZONE_ENTRY *Prev;
//     struct _TIMEZONE_ENTRY *Next;
//     WCHAR Description[128]; /* 'Display' */
//     WCHAR StandardName[32]; /* 'Std' */
//     WCHAR DaylightName[32]; /* 'Dlt' */
//     REG_TZI_FORMAT TimezoneInfo; /* 'TZI' */
//     ULONG Index;
// } TIMEZONE_ENTRY, *PTIMEZONE_ENTRY;
//
//
// /* FUNCTIONS ****************************************************************/
//
// extern void WINAPI Control_RunDLLW(HWND hWnd, HINSTANCE hInst, LPCWSTR cmd, DWORD nCmdShow);
//
// static VOID
// CenterWindow(HWND hWnd)
// {
//     HWND hWndParent;
//     RECT rcParent;
//     RECT rcWindow;
//
//     hWndParent = GetParent(hWnd);
//     if (hWndParent == NULL)
//         hWndParent = GetDesktopWindow();
//
//     GetWindowRect(hWndParent, &rcParent);
//     GetWindowRect(hWnd, &rcWindow);
//
//     SetWindowPos(hWnd,
//                  HWND_TOP,
//                  ((rcParent.right - rcParent.left) - (rcWindow.right - rcWindow.left)) / 2,
//                  ((rcParent.bottom - rcParent.top) - (rcWindow.bottom - rcWindow.top)) / 2,
//                  0,
//                  0,
//                  SWP_NOSIZE);
// }
//
//
// static HFONT
// CreateTitleFont(VOID)
// {
//     LOGFONTW LogFont = {0};
//     HDC hdc;
//     HFONT hFont;
//
//     LogFont.lfWeight = FW_BOLD;
//     wcscpy(LogFont.lfFaceName, L"MS Shell Dlg");
//
//     hdc = GetDC(NULL);
//     LogFont.lfHeight = -MulDiv(12, GetDeviceCaps(hdc, LOGPIXELSY), 72);
//
//     hFont = CreateFontIndirectW(&LogFont);
//
//     ReleaseDC(NULL, hdc);
//
//     return hFont;
// }
//
//
// static HFONT
// CreateBoldFont(VOID)
// {
//     LOGFONTW tmpFont = {0};
//     HFONT hBoldFont;
//     HDC hDc;
//
//     /* Grabs the Drawing Context */
//     hDc = GetDC(NULL);
//
//     tmpFont.lfHeight = -MulDiv(8, GetDeviceCaps(hDc, LOGPIXELSY), 72);
//     tmpFont.lfWeight = FW_BOLD;
//     wcscpy(tmpFont.lfFaceName, L"MS Shell Dlg");
//
//     hBoldFont = CreateFontIndirectW(&tmpFont);
//
//     ReleaseDC(NULL, hDc);
//
//     return hBoldFont;
// }
//
// static INT_PTR __stdcall
// GplDlgProc(HWND hwndDlg,
//            UINT uMsg,
//            WPARAM wParam,
//            LPARAM lParam)
// {
//     HRSRC GplTextResource;
//     HGLOBAL GplTextMem;
//     PVOID GplTextLocked;
//     PCHAR GplText;
//     DWORD Size;
//
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             GplTextResource = FindResourceW(hDllInstance, MAKEINTRESOURCE!(IDR_GPL), L"RT_TEXT");
//             if (NULL == GplTextResource)
//             {
//                 break;
//             }
//             Size = SizeofResource(hDllInstance, GplTextResource);
//             if (0 == Size)
//             {
//                 break;
//             }
//             GplText = HeapAlloc(GetProcessHeap(), 0, Size + 1);
//             if (NULL == GplText)
//             {
//                 break;
//             }
//             GplTextMem = LoadResource(hDllInstance, GplTextResource);
//             if (NULL == GplTextMem)
//             {
//                 HeapFree(GetProcessHeap(), 0, GplText);
//                 break;
//             }
//             GplTextLocked = LockResource(GplTextMem);
//             if (NULL == GplTextLocked)
//             {
//                 HeapFree(GetProcessHeap(), 0, GplText);
//                 break;
//             }
//             memcpy(GplText, GplTextLocked, Size);
//             GplText[Size] = '\0';
//             SendMessageA(GetDlgItem(hwndDlg, IDC_GPL_TEXT), WM_SETTEXT, 0, (LPARAM) GplText);
//             HeapFree(GetProcessHeap(), 0, GplText);
//             SetFocus(GetDlgItem(hwndDlg, IDOK));
//             return FALSE;
//
//         case WM_CLOSE:
//             EndDialog(hwndDlg, IDCANCEL);
//             break;
//
//         case WM_COMMAND:
//             if (HIWORD!(wParam) == BN_CLICKED && IDOK == LOWORD!(wParam) as u32)
//             {
//                 EndDialog(hwndDlg, IDOK);
//             }
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static INT_PTR __stdcall
// WelcomeDlgProc(HWND hwndDlg,
//                UINT uMsg,
//                WPARAM wParam,
//                LPARAM lParam)
// {
//     PSETUPDATA pSetupData;
//
//     pSetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             HWND hwndControl;
//             DWORD dwStyle;
//
//             /* Get pointer to the global setup data */
//             pSetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pSetupData);
//
//             hwndControl = GetParent(hwndDlg);
//
//             /* Center the wizard window */
//             CenterWindow (hwndControl);
//
//             /* Hide the system menu */
//             dwStyle = GetWindowLongPtr(hwndControl, GWL_STYLE);
//             SetWindowLongPtr(hwndControl, GWL_STYLE, dwStyle & ~WS_SYSMENU);
//
//             /* Hide and disable the 'Cancel' button */
//             hwndControl = GetDlgItem(GetParent(hwndDlg), IDCANCEL);
//             ShowWindow (hwndControl, SW_HIDE);
//             EnableWindow (hwndControl, FALSE);
//
//             /* Set title font */
//             SendDlgItemMessage(hwndDlg,
//                                IDC_WELCOMETITLE,
//                                WM_SETFONT,
//                                (WPARAM)pSetupData->hTitleFont,
//                                (LPARAM)TRUE);
//         }
//         break;
//
//
//         case WM_NOTIFY:
//         {
//             LPNMHDR lpnm = (LPNMHDR)lParam;
//
//             switch (lpnm->code)
//             {
//                 case PSN_SETACTIVE:
//                     LogItem!(L"BEGIN", L"WelcomePage");
//                     /* Enable the Next button */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
//                     if (pSetupData->UnattendSetup)
//                     {
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_ACKPAGE);
//                         return TRUE;
//                     }
//                     break;
//
//                 case PSN_WIZNEXT:
//                     LogItem!(L"END", L"WelcomePage");
//                     break;
//
//                 case PSN_WIZBACK:
//                     pSetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static INT_PTR __stdcall
// AckPageDlgProc(HWND hwndDlg,
//                UINT uMsg,
//                WPARAM wParam,
//                LPARAM lParam)
// {
//     LPNMHDR lpnm;
//     PWCHAR Projects;
//     PWCHAR End, CurrentProject;
//     INT ProjectsSize, ProjectsCount;
//     PSETUPDATA pSetupData;
//
//     pSetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             pSetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pSetupData);
//
//             Projects = NULL;
//             ProjectsSize = 256;
//             while (TRUE)
//             {
//                 Projects = HeapAlloc(GetProcessHeap(), 0, ProjectsSize * sizeof(WCHAR));
//                 if (NULL == Projects)
//                 {
//                     return FALSE;
//                 }
//                 ProjectsCount =  LoadStringW(hDllInstance, IDS_ACKPROJECTS, Projects, ProjectsSize);
//                 if (0 == ProjectsCount)
//                 {
//                     HeapFree(GetProcessHeap(), 0, Projects);
//                     return FALSE;
//                 }
//                 if (ProjectsCount < ProjectsSize - 1)
//                 {
//                     break;
//                 }
//                 HeapFree(GetProcessHeap(), 0, Projects);
//                 ProjectsSize *= 2;
//             }
//
//             CurrentProject = Projects;
//             while (*CurrentProject != 0u16)
//             {
//                 End = wcschr(CurrentProject, L'\n');
//                 if (NULL != End)
//                 {
//                     *End = 0u16;
//                 }
//                 (void)ListBox_AddString!(GetDlgItem(hwndDlg, IDC_PROJECTS), CurrentProject);
//                 if (NULL != End)
//                 {
//                     CurrentProject = End + 1;
//                 }
//                 else
//                 {
//                     CurrentProject += wcslen(CurrentProject);
//                 }
//             }
//             HeapFree(GetProcessHeap(), 0, Projects);
//         }
//         break;
//
//         case WM_COMMAND:
//             if (HIWORD!(wParam) == BN_CLICKED && IDC_VIEWGPL == LOWORD!(wParam) as u32)
//             {
//                 DialogBox!(hDllInstance, MAKEINTRESOURCE!(IDD_GPL), NULL, GplDlgProc);
//                 SetForegroundWindow(GetParent(hwndDlg));
//             }
//             break;
//
//         case WM_NOTIFY:
//         {
//             lpnm = (LPNMHDR)lParam;
//
//             switch (lpnm->code)
//             {
//                 case PSN_SETACTIVE:
//                     /* Enable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
//                     if (pSetupData->UnattendSetup)
//                     {
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_PRODUCT);
//                         return TRUE;
//                     }
//                     break;
//
//                 case PSN_WIZBACK:
//                     pSetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// static const WCHAR s_szProductOptions[] = L"SYSTEM\\CurrentControlSet\\Control\\ProductOptions";
// static const WCHAR s_szRosVersion[] = L"SYSTEM\\CurrentControlSet\\Control\\ReactOS\\Settings\\Version";
// static const WCHAR s_szControlWindows[] = L"SYSTEM\\CurrentControlSet\\Control\\Windows";
// static const WCHAR s_szWinlogon[] = L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon";
// static const WCHAR s_szDefaultSoundEvents[] = L"AppEvents\\Schemes\\Apps\\.Default";
// static const WCHAR s_szExplorerSoundEvents[] = L"AppEvents\\Schemes\\Apps\\Explorer";
//
// typedef struct _PRODUCT_OPTION_DATA
// {
//     LPCWSTR ProductSuite;
//     LPCWSTR ProductType;
//     DWORD ReportAsWorkstation;
//     DWORD CSDVersion;
//     DWORD LogonType;
// } PRODUCT_OPTION_DATA;
//
// static const PRODUCT_OPTION_DATA s_ProductOptionData[] =
// {
//     { L"Terminal Server\0", L"ServerNT", 0, 0x200, 0 },
//     { L"\0", L"WinNT", 1, 0x300, 1 }
// };
//
// static const WCHAR* s_DefaultSoundEvents[][2] =
// {
//     { L".Default", L"%SystemRoot%\\Media\\ReactOS_Default.wav" },
//     { L"AppGPFault", L"" },
//     { L"Close", L"" },
//     { L"CriticalBatteryAlarm", L"%SystemRoot%\\Media\\ReactOS_Battery_Critical.wav" },
//     { L"DeviceConnect",  L"%SystemRoot%\\Media\\ReactOS_Hardware_Insert.wav" },
//     { L"DeviceDisconnect", L"%SystemRoot%\\Media\\ReactOS_Hardware_Remove.wav" },
//     { L"DeviceFail", L"%SystemRoot%\\Media\\ReactOS_Hardware_Fail.wav" },
//     { L"LowBatteryAlarm", L"%SystemRoot%\\Media\\ReactOS_Battery_Low.wav" },
//     { L"MailBeep", L"%SystemRoot%\\Media\\ReactOS_Notify.wav" },
//     { L"Maximize", L"%SystemRoot%\\Media\\ReactOS_Restore.wav" },
//     { L"MenuCommand", L"%SystemRoot%\\Media\\ReactOS_Menu_Command.wav" },
//     { L"MenuPopup", L"" },
//     { L"Minimize", L"%SystemRoot%\\Media\\ReactOS_Minimize.wav" },
//     { L"Open", L"" },
//     { L"PrintComplete", L"%SystemRoot%\\Media\\ReactOS_Print_Complete.wav" },
//     { L"RestoreDown", L"" },
//     { L"RestoreUp", L"" },
//     { L"SystemAsterisk", L"%SystemRoot%\\Media\\ReactOS_Ding.wav" },
//     { L"SystemExclamation", L"%SystemRoot%\\Media\\ReactOS_Exclamation.wav" },
//     { L"SystemExit", L"%SystemRoot%\\Media\\ReactOS_Shutdown.wav" },
//     { L"SystemHand", L"%SystemRoot%\\Media\\ReactOS_Critical_Stop.wav" },
//     { L"SystemNotification", L"%SystemRoot%\\Media\\ReactOS_Balloon.wav" },
//     { L"SystemQuestion", L"%SystemRoot%\\Media\\ReactOS_Ding.wav" },
//     { L"SystemStart", L"%SystemRoot%\\Media\\ReactOS_Startup.wav" },
//     { L"WindowsLogoff", L"%SystemRoot%\\Media\\ReactOS_LogOff.wav" }
// /* Logon sound is already set by default for both Server and Workstation */
// };
//
// static const WCHAR* s_ExplorerSoundEvents[][2] =
// {
//     { L"EmptyRecycleBin", L"%SystemRoot%\\Media\\ReactOS_Recycle.wav" },
//     { L"Navigating", L"%SystemRoot%\\Media\\ReactOS_Start.wav" }
// };
//
// static BOOL
// DoWriteSoundEvents(HKEY hKey,
//                    LPCWSTR lpSubkey,
//                    LPCWSTR lpEventsArray[][2],
//                    DWORD dwSize)
// {
//     HKEY hRootKey, hEventKey, hDefaultKey;
//     LONG error;
//     ULONG i;
//     WCHAR szDest[MAX_PATH];
//     DWORD dwAttribs;
//     DWORD cbData;
//
//     /* Open the sound events key */
//     error = RegOpenKeyExW(hKey, lpSubkey, 0, KEY_READ, &hRootKey);
//     if (error)
//     {
//         DPRINT1("RegOpenKeyExW failed\n");
//         goto Error;
//     }
//
//     /* Set each sound event */
//     for (i = 0; i < dwSize; i++)
//     {
//         /*
//          * Verify that the sound file exists and is an actual file.
//          */
//
//         /* Expand the sound file path */
//         if (!ExpandEnvironmentStringsW(lpEventsArray[i][1], szDest, _countof!(szDest)))
//         {
//             /* Failed to expand, continue with the next sound event */
//             continue;
//         }
//
//         /* Check if the sound file exists and isn't a directory */
//         dwAttribs = GetFileAttributesW(szDest);
//         if ((dwAttribs == INVALID_FILE_ATTRIBUTES) ||
//             (dwAttribs & FILE_ATTRIBUTE_DIRECTORY))
//         {
//             /* It does not, just continue with the next sound event */
//             continue;
//         }
//
//         /*
//          * Create the sound event entry.
//          */
//
//         /* Open the sound event subkey */
//         error = RegOpenKeyExW(hRootKey, lpEventsArray[i][0], 0, KEY_READ, &hEventKey);
//         if (error)
//         {
//             /* Failed to open, continue with next sound event */
//             continue;
//         }
//
//         /* Open .Default subkey */
//         error = RegOpenKeyExW(hEventKey, L".Default", 0, KEY_WRITE, &hDefaultKey);
//         RegCloseKey(hEventKey);
//         if (error)
//         {
//             /* Failed to open, continue with next sound event */
//             continue;
//         }
//
//         /* Associate the sound file to this sound event */
//         cbData = (lstrlenW(lpEventsArray[i][1]) + 1) * sizeof(WCHAR);
//         error = RegSetValueExW(hDefaultKey, NULL, 0, REG_EXPAND_SZ, (const BYTE *)lpEventsArray[i][1], cbData);
//         RegCloseKey(hDefaultKey);
//         if (error)
//         {
//             /* Failed to set the value, continue with next sound event */
//             continue;
//         }
//     }
//
// Error:
//     if (hRootKey)
//         RegCloseKey(hRootKey);
//
//     return error == ERROR_SUCCESS;
// }
//
// static BOOL
// DoWriteProductOption(PRODUCT_OPTION nOption)
// {
//     HKEY hKey;
//     LONG error;
//     LPCWSTR pszData;
//     DWORD dwValue, cbData;
//     const PRODUCT_OPTION_DATA *pData = &s_ProductOptionData[nOption];
//     ASSERT(0 <= nOption && nOption < _countof!(s_ProductOptionData));
//
//     /* open ProductOptions key */
//     error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szProductOptions, 0, KEY_WRITE, &hKey);
//     if (error)
//     {
//         DPRINT1("RegOpenKeyExW failed\n");
//         goto Error;
//     }
//
//     /* write ProductSuite */
//     pszData = pData->ProductSuite;
//     cbData = (lstrlenW(pszData) + 2) * sizeof(WCHAR);
//     error = RegSetValueExW(hKey, L"ProductSuite", 0, REG_MULTI_SZ, (const BYTE *)pszData, cbData);
//     if (error)
//     {
//         DPRINT1("RegSetValueExW failed\n");
//         goto Error;
//     }
//
//     /* write ProductType */
//     pszData = pData->ProductType;
//     cbData = (lstrlenW(pszData) + 1) * sizeof(WCHAR);
//     error = RegSetValueExW(hKey, L"ProductType", 0, REG_SZ, (const BYTE *)pszData, cbData);
//     if (error)
//     {
//         DPRINT1("RegSetValueExW failed\n");
//         goto Error;
//     }
//
//     RegCloseKey(hKey);
//
//     /* open ReactOS version key */
//     error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szRosVersion, 0, KEY_WRITE, &hKey);
//     if (error)
//     {
//         DPRINT1("RegOpenKeyExW failed\n");
//         goto Error;
//     }
//
//     /* write ReportAsWorkstation */
//     dwValue = pData->ReportAsWorkstation;
//     cbData = sizeof(dwValue);
//     error = RegSetValueExW(hKey, L"ReportAsWorkstation", 0, REG_DWORD, (const BYTE *)&dwValue, cbData);
//     if (error)
//     {
//         DPRINT1("RegSetValueExW failed\n");
//         goto Error;
//     }
//
//     RegCloseKey(hKey);
//
//     /* open Control Windows key */
//     error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szControlWindows, 0, KEY_WRITE, &hKey);
//     if (error)
//     {
//         DPRINT1("RegOpenKeyExW failed\n");
//         goto Error;
//     }
//
//     /* write Control Windows CSDVersion */
//     dwValue = pData->CSDVersion;
//     cbData = sizeof(dwValue);
//     error = RegSetValueExW(hKey, L"CSDVersion", 0, REG_DWORD, (const BYTE *)&dwValue, cbData);
//     if (error)
//     {
//         DPRINT1("RegSetValueExW failed\n");
//         goto Error;
//     }
//
//     RegCloseKey(hKey);
//
//     /* open Winlogon key */
//     error = RegOpenKeyExW(HKEY_LOCAL_MACHINE, s_szWinlogon, 0, KEY_WRITE, &hKey);
//     if (error)
//     {
//         DPRINT1("RegOpenKeyExW failed\n");
//         goto Error;
//     }
//
//     /* write LogonType */
//     dwValue = pData->LogonType;
//     cbData = sizeof(dwValue);
//     error = RegSetValueExW(hKey, L"LogonType", 0, REG_DWORD, (const BYTE *)&dwValue, cbData);
//     if (error)
//     {
//         DPRINT1("RegSetValueExW failed\n");
//         goto Error;
//     }
//
//     if (nOption == PRODUCT_OPTION_WORKSTATION)
//     {
//         /* Write system sound events values for Workstation */
//         DoWriteSoundEvents(HKEY_CURRENT_USER, s_szDefaultSoundEvents, s_DefaultSoundEvents, _countof!(s_DefaultSoundEvents));
//         DoWriteSoundEvents(HKEY_CURRENT_USER, s_szExplorerSoundEvents, s_ExplorerSoundEvents, _countof!(s_ExplorerSoundEvents));
//     }
//
// Error:
//     if (hKey)
//         RegCloseKey(hKey);
//
//     return error == ERROR_SUCCESS;
// }
//
// static void
// OnChooseOption(HWND hwndDlg, PRODUCT_OPTION nOption)
// {
//     WCHAR szText[256];
//     ASSERT(0 <= nOption && nOption < _countof!(s_ProductOptionData));
//
//     switch (nOption)
//     {
//         case PRODUCT_OPTION_SERVER:
//             LoadStringW(hDllInstance, IDS_PRODUCTSERVERINFO, szText, _countof!(szText));
//             break;
//
//         case PRODUCT_OPTION_WORKSTATION:
//             LoadStringW(hDllInstance, IDS_PRODUCTWORKSTATIONINFO, szText, _countof!(szText));
//             break;
//
//         default:
//             return;
//     }
//
//     SetDlgItemTextW(hwndDlg, IDC_PRODUCT_DESCRIPTION, szText);
// }
//
// static INT_PTR __stdcall
// ProductPageDlgProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     LPNMHDR lpnm;
//     PSETUPDATA pSetupData;
//     INT iItem;
//     WCHAR szText[64], szDefault[64];
//     HICON hIcon;
//
//     pSetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             pSetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pSetupData);
//
//             LoadStringW(hDllInstance, IDS_DEFAULT, szDefault, _countof!(szDefault));
//
//             LoadStringW(hDllInstance, IDS_PRODUCTSERVERNAME, szText, _countof!(szText));
//             if (PRODUCT_OPTION_DEFAULT == PRODUCT_OPTION_SERVER)
//             {
//                 StringCchCatW(szText, _countof!(szText), L" ");
//                 StringCchCatW(szText, _countof!(szText), szDefault);
//             }
//             SendDlgItemMessageW(hwndDlg, IDC_PRODUCT_OPTIONS, CB_ADDSTRING, 0, (LPARAM)szText);
//
//             LoadStringW(hDllInstance, IDS_PRODUCTWORKSTATIONNAME, szText, _countof!(szText));
//             if (PRODUCT_OPTION_DEFAULT == PRODUCT_OPTION_WORKSTATION)
//             {
//                 StringCchCatW(szText, _countof!(szText), L" ");
//                 StringCchCatW(szText, _countof!(szText), szDefault);
//             }
//             SendDlgItemMessageW(hwndDlg, IDC_PRODUCT_OPTIONS, CB_ADDSTRING, 0, (LPARAM)szText);
//
//             SendDlgItemMessageW(hwndDlg, IDC_PRODUCT_OPTIONS, CB_SETCURSEL, PRODUCT_OPTION_DEFAULT, 0);
//             OnChooseOption(hwndDlg, PRODUCT_OPTION_DEFAULT);
//
//             hIcon = LoadIcon(NULL, IDI_WINLOGO);
//             SendDlgItemMessageW(hwndDlg, IDC_PRODUCT_ICON, STM_SETICON, (WPARAM)hIcon, 0);
//             return TRUE;
//         }
//
//         case WM_COMMAND:
//             if (HIWORD!(wParam) == CBN_SELCHANGE && IDC_PRODUCT_OPTIONS == LOWORD!(wParam) as u32)
//             {
//                 iItem = SendDlgItemMessageW(hwndDlg, IDC_PRODUCT_OPTIONS, CB_GETCURSEL, 0, 0);
//                 OnChooseOption(hwndDlg, (PRODUCT_OPTION)iItem);
//             }
//             break;
//
//         case WM_NOTIFY:
//         {
//             lpnm = (LPNMHDR)lParam;
//
//             switch (lpnm->code)
//             {
//                 case PSN_SETACTIVE:
//                     /* Enable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
//                     if (pSetupData->UnattendSetup)
//                     {
//                         OnChooseOption(hwndDlg, pSetupData->ProductOption);
//                         DoWriteProductOption(pSetupData->ProductOption);
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_LOCALEPAGE);
//                         return TRUE;
//                     }
//                     break;
//
//                 case PSN_WIZNEXT:
//                     iItem = SendDlgItemMessageW(hwndDlg, IDC_PRODUCT_OPTIONS, CB_GETCURSEL, 0, 0);
//                     pSetupData->ProductOption = (PRODUCT_OPTION)iItem;
//                     DoWriteProductOption(pSetupData->ProductOption);
//                     break;
//
//                 case PSN_WIZBACK:
//                     pSetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// static
// BOOL
// WriteOwnerSettings(WCHAR * OwnerName,
//                    WCHAR * OwnerOrganization)
// {
//     HKEY hKey;
//     LONG res;
//
//     res = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                         L"Software\\Microsoft\\Windows NT\\CurrentVersion",
//                         0,
//                         KEY_ALL_ACCESS,
//                         &hKey);
//
//     if (res != ERROR_SUCCESS)
//     {
//         return FALSE;
//     }
//
//     res = RegSetValueExW(hKey,
//                          L"RegisteredOwner",
//                          0,
//                          REG_SZ,
//                          (LPBYTE)OwnerName,
//                          (wcslen(OwnerName) + 1) * sizeof(WCHAR));
//
//     if (res != ERROR_SUCCESS)
//     {
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     res = RegSetValueExW(hKey,
//                          L"RegisteredOrganization",
//                          0,
//                          REG_SZ,
//                          (LPBYTE)OwnerOrganization,
//                          (wcslen(OwnerOrganization) + 1) * sizeof(WCHAR));
//
//     RegCloseKey(hKey);
//     return (res == ERROR_SUCCESS);
// }
//
// static INT_PTR __stdcall
// OwnerPageDlgProc(HWND hwndDlg,
//                  UINT uMsg,
//                  WPARAM wParam,
//                  LPARAM lParam)
// {
//     WCHAR OwnerName[51];
//     WCHAR OwnerOrganization[51];
//     WCHAR Title[64];
//     WCHAR ErrorName[256];
//     LPNMHDR lpnm;
//     PSETUPDATA pSetupData;
//
//     pSetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             pSetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pSetupData);
//
//             /* set a localized ('Owner') placeholder string as default */
//             if (LoadStringW(hDllInstance, IDS_MACHINE_OWNER_NAME, OwnerName, _countof!(OwnerName)))
//             {
//                 SendDlgItemMessage(hwndDlg, IDC_OWNERNAME, WM_SETTEXT, 0, (LPARAM)OwnerName);
//             }
//
//             SendDlgItemMessage(hwndDlg, IDC_OWNERNAME, EM_LIMITTEXT, 50, 0);
//             SendDlgItemMessage(hwndDlg, IDC_OWNERORGANIZATION, EM_LIMITTEXT, 50, 0);
//
//             /* Set focus to owner name */
//             SetFocus(GetDlgItem(hwndDlg, IDC_OWNERNAME));
//
//             /* Select the default text to quickly overwrite it by typing */
//             SendDlgItemMessage(hwndDlg, IDC_OWNERNAME, EM_SETSEL, 0, -1);
//         }
//         break;
//
//
//         case WM_NOTIFY:
//         {
//             lpnm = (LPNMHDR)lParam;
//
//             switch (lpnm->code)
//             {
//                 case PSN_SETACTIVE:
//                     /* Enable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
//                     if (pSetupData->UnattendSetup)
//                     {
//                         SendMessage(GetDlgItem(hwndDlg, IDC_OWNERNAME), WM_SETTEXT, 0, (LPARAM)pSetupData->OwnerName);
//                         SendMessage(GetDlgItem(hwndDlg, IDC_OWNERORGANIZATION), WM_SETTEXT, 0, (LPARAM)pSetupData->OwnerOrganization);
//                         if (WriteOwnerSettings(pSetupData->OwnerName, pSetupData->OwnerOrganization))
//                         {
//                             SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_COMPUTERPAGE);
//                             return TRUE;
//                         }
//                     }
//                     break;
//
//                 case PSN_WIZNEXT:
//                     OwnerName[0] = 0;
//                     if (GetDlgItemTextW(hwndDlg, IDC_OWNERNAME, OwnerName, 50) == 0)
//                     {
//                         if (0 == LoadStringW(hDllInstance, IDS_REACTOS_SETUP, Title, ARRAYSIZE!(Title)))
//                         {
//                             wcscpy(Title, L"ReactOS Setup");
//                         }
//                         if (0 == LoadStringW(hDllInstance, IDS_WZD_NAME, ErrorName, ARRAYSIZE!(ErrorName)))
//                         {
//                             wcscpy(ErrorName, L"Setup cannot continue until you enter your name.");
//                         }
//                         MessageBoxW(hwndDlg, ErrorName, Title, MB_ICONERROR | MB_OK);
//
//                         SetFocus(GetDlgItem(hwndDlg, IDC_OWNERNAME));
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
//
//                         return TRUE;
//                     }
//
//                     OwnerOrganization[0] = 0;
//                     GetDlgItemTextW(hwndDlg, IDC_OWNERORGANIZATION, OwnerOrganization, 50);
//
//                     if (!WriteOwnerSettings(OwnerName, OwnerOrganization))
//                     {
//                         SetFocus(GetDlgItem(hwndDlg, IDC_OWNERNAME));
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
//                         return TRUE;
//                     }
//
//                 case PSN_WIZBACK:
//                     pSetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// static
// BOOL
// WriteComputerSettings(WCHAR * ComputerName, HWND hwndDlg)
// {
//     WCHAR Title[64];
//     WCHAR ErrorComputerName[256];
//     LONG lError;
//     HKEY hKey = NULL;
//
//     if (!SetComputerNameW(ComputerName))
//     {
//         if (hwndDlg != NULL)
//         {
//             if (0 == LoadStringW(hDllInstance, IDS_REACTOS_SETUP, Title, ARRAYSIZE!(Title)))
//             {
//                 wcscpy(Title, L"ReactOS Setup");
//             }
//             if (0 == LoadStringW(hDllInstance, IDS_WZD_SETCOMPUTERNAME, ErrorComputerName,
//                                  ARRAYSIZE!(ErrorComputerName)))
//             {
//                 wcscpy(ErrorComputerName, L"Setup failed to set the computer name.");
//             }
//             MessageBoxW(hwndDlg, ErrorComputerName, Title, MB_ICONERROR | MB_OK);
//         }
//
//         return FALSE;
//     }
//
//     /* Set the physical DNS domain */
//     SetComputerNameExW(ComputerNamePhysicalDnsDomain, L"");
//
//     /* Set the physical DNS hostname */
//     SetComputerNameExW(ComputerNamePhysicalDnsHostname, ComputerName);
//
//     /* Set the accounts domain name */
//     SetAccountsDomainSid(NULL, ComputerName);
//
//     /* Now we need to set the Hostname */
//     lError = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                              L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters",
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_WRITE,
//                              NULL,
//                              &hKey,
//                              NULL);
//     if (lError == ERROR_SUCCESS)
//     {
//         lError = RegSetValueEx(hKey,
//                                L"Hostname",
//                                0,
//                                REG_SZ,
//                                (LPBYTE)ComputerName,
//                                (wcslen(ComputerName) + 1) * sizeof(WCHAR));
//         if (lError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegSetValueEx(\"Hostname\") failed (%08lX)\n", lError);
//         }
//
//         RegCloseKey(hKey);
//     }
//     else
//     {
//         DPRINT1("RegCreateKeyExW for Tcpip\\Parameters failed (%08lX)\n", lError);
//     }
//
//     return TRUE;
// }
//
//
// static
// BOOL
// WriteDefaultLogonData(LPWSTR Domain)
// {
//     WCHAR szAdministratorName[256];
//     HKEY hKey = NULL;
//     LONG lError;
//
//     if (LoadStringW(hDllInstance,
//                     IDS_ADMINISTRATOR_NAME,
//                     szAdministratorName,
//                     ARRAYSIZE!(szAdministratorName)) == 0)
//     {
//         wcscpy(szAdministratorName, L"Administrator");
//     }
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
//                            0,
//                            KEY_SET_VALUE,
//                            &hKey);
//     if (lError != ERROR_SUCCESS)
//         return FALSE;
//
//     lError = RegSetValueEx(hKey,
//                            L"DefaultDomainName",
//                            0,
//                            REG_SZ,
//                            (LPBYTE)Domain,
//                            (wcslen(Domain)+ 1) * sizeof(WCHAR));
//     if (lError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegSetValueEx(\"DefaultDomainName\") failed!\n");
//     }
//
//     lError = RegSetValueEx(hKey,
//                            L"DefaultUserName",
//                            0,
//                            REG_SZ,
//                            (LPBYTE)szAdministratorName,
//                            (wcslen(szAdministratorName)+ 1) * sizeof(WCHAR));
//     if (lError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegSetValueEx(\"DefaultUserName\") failed!\n");
//     }
//
//     RegCloseKey(hKey);
//
//     return TRUE;
// }
//
//
// /* lpBuffer will be filled with a 15-char string (plus the null terminator) */
// static void
// GenerateComputerName(LPWSTR lpBuffer)
// {
//     static const WCHAR Chars[] = L"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
//     static const unsigned cChars = sizeof(Chars) / sizeof(WCHAR) - 1;
//     unsigned i;
//
//     wcscpy(lpBuffer, L"REACTOS-");
//
//     srand(GetTickCount());
//
//     /* fill in 7 characters */
//     for (i = 8; i < 15; i++)
//         lpBuffer[i] = Chars[rand() % cChars];
//
//     lpBuffer[15] = UNICODE_NULL; /* NULL-terminate */
// }
//
// static INT_PTR __stdcall
// ComputerPageDlgProc(HWND hwndDlg,
//                     UINT uMsg,
//                     WPARAM wParam,
//                     LPARAM lParam)
// {
//     WCHAR ComputerName[MAX_COMPUTERNAME_LENGTH + 1];
//     WCHAR Password1[128];
//     WCHAR Password2[128];
//     PWCHAR Password;
//     WCHAR Title[64];
//     WCHAR EmptyComputerName[256], NotMatchPassword[256], WrongPassword[256];
//     LPNMHDR lpnm;
//     PSETUPDATA pSetupData;
//
//     pSetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     if (0 == LoadStringW(hDllInstance, IDS_REACTOS_SETUP, Title, ARRAYSIZE!(Title)))
//     {
//         wcscpy(Title, L"ReactOS Setup");
//     }
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             pSetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pSetupData);
//
//             /* Generate a new pseudo-random computer name */
//             GenerateComputerName(ComputerName);
//
//             /* Display current computer name */
//             SetDlgItemTextW(hwndDlg, IDC_COMPUTERNAME, ComputerName);
//
//             /* Set text limits */
//             SendDlgItemMessage(hwndDlg, IDC_COMPUTERNAME, EM_LIMITTEXT, MAX_COMPUTERNAME_LENGTH, 0);
//             SendDlgItemMessage(hwndDlg, IDC_ADMINPASSWORD1, EM_LIMITTEXT, 127, 0);
//             SendDlgItemMessage(hwndDlg, IDC_ADMINPASSWORD2, EM_LIMITTEXT, 127, 0);
//
//             /* Set focus to computer name */
//             SetFocus(GetDlgItem(hwndDlg, IDC_COMPUTERNAME));
//             if (pSetupData->UnattendSetup)
//             {
//                 SendMessage(GetDlgItem(hwndDlg, IDC_COMPUTERNAME), WM_SETTEXT, 0, (LPARAM)pSetupData->ComputerName);
//                 SendMessage(GetDlgItem(hwndDlg, IDC_ADMINPASSWORD1), WM_SETTEXT, 0, (LPARAM)pSetupData->AdminPassword);
//                 SendMessage(GetDlgItem(hwndDlg, IDC_ADMINPASSWORD2), WM_SETTEXT, 0, (LPARAM)pSetupData->AdminPassword);
//                 WriteComputerSettings(pSetupData->ComputerName, NULL);
//                 SetAdministratorPassword(pSetupData->AdminPassword);
//             }
//
//             /* Store the administrator account name as the default user name */
//             WriteDefaultLogonData(pSetupData->ComputerName);
//             break;
//
//
//         case WM_NOTIFY:
//         {
//             lpnm = (LPNMHDR)lParam;
//
//             switch (lpnm->code)
//             {
//                 case PSN_SETACTIVE:
//                     /* Enable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
//                     if (pSetupData->UnattendSetup && WriteComputerSettings(pSetupData->ComputerName, hwndDlg))
//                     {
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_DATETIMEPAGE);
//                         return TRUE;
//                     }
//                     break;
//
//                 case PSN_WIZNEXT:
//                     if (0 == GetDlgItemTextW(hwndDlg, IDC_COMPUTERNAME, ComputerName, MAX_COMPUTERNAME_LENGTH + 1))
//                     {
//                         if (0 == LoadStringW(hDllInstance, IDS_WZD_COMPUTERNAME, EmptyComputerName,
//                                              ARRAYSIZE!(EmptyComputerName)))
//                         {
//                             wcscpy(EmptyComputerName, L"Setup cannot continue until you enter the name of your computer.");
//                         }
//                         MessageBoxW(hwndDlg, EmptyComputerName, Title, MB_ICONERROR | MB_OK);
//                         SetFocus(GetDlgItem(hwndDlg, IDC_COMPUTERNAME));
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
//                         return TRUE;
//                     }
//
//                     /* No need to check computer name for invalid characters,
//                      * SetComputerName() will do it for us */
//
//                     if (!WriteComputerSettings(ComputerName, hwndDlg))
//                     {
//                         SetFocus(GetDlgItem(hwndDlg, IDC_COMPUTERNAME));
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
//                         return TRUE;
//                     }
//
// #ifdef PASSWORDS_MANDATORY
//                     /* Check if admin passwords have been entered */
//                     if ((GetDlgItemText(hwndDlg, IDC_ADMINPASSWORD1, Password1, 128) == 0) ||
//                         (GetDlgItemText(hwndDlg, IDC_ADMINPASSWORD2, Password2, 128) == 0))
//                     {
//                         if (0 == LoadStringW(hDllInstance, IDS_WZD_PASSWORDEMPTY, EmptyPassword,
//                                              ARRAYSIZE!(EmptyPassword)))
//                         {
//                             wcscpy(EmptyPassword, L"You must enter a password !");
//                         }
//                         MessageBoxW(hwndDlg, EmptyPassword, Title, MB_ICONERROR | MB_OK);
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
//                         return TRUE;
//                     }
// #else
//                     GetDlgItemTextW(hwndDlg, IDC_ADMINPASSWORD1, Password1, 128);
//                     GetDlgItemTextW(hwndDlg, IDC_ADMINPASSWORD2, Password2, 128);
// #endif
//                     /* Check if passwords match */
//                     if (wcscmp(Password1, Password2))
//                     {
//                         if (0 == LoadStringW(hDllInstance, IDS_WZD_PASSWORDMATCH, NotMatchPassword,
//                                              ARRAYSIZE!(NotMatchPassword)))
//                         {
//                             wcscpy(NotMatchPassword, L"The passwords you entered do not match. Please enter the desired password again.");
//                         }
//                         MessageBoxW(hwndDlg, NotMatchPassword, Title, MB_ICONERROR | MB_OK);
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
//                         return TRUE;
//                     }
//
//                     /* Check password for invalid characters */
//                     Password = (PWCHAR)Password1;
//                     while (*Password)
//                     {
//                         if (!isprint(*Password))
//                         {
//                             if (0 == LoadStringW(hDllInstance, IDS_WZD_PASSWORDCHAR, WrongPassword,
//                                                  ARRAYSIZE!(WrongPassword)))
//                             {
//                                 wcscpy(WrongPassword, L"The password you entered contains invalid characters. Please enter a cleaned password.");
//                             }
//                             MessageBoxW(hwndDlg, WrongPassword, Title, MB_ICONERROR | MB_OK);
//                             SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, -1);
//                             return TRUE;
//                         }
//                         Password++;
//                     }
//
//                     /* Set admin password */
//                     SetAdministratorPassword(Password1);
//                     break;
//
//                 case PSN_WIZBACK:
//                     pSetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static VOID
// SetUserLocaleName(HWND hwnd)
// {
//     WCHAR CurLocale[256] = L"";
//     WCHAR CurGeo[256] = L"";
//     WCHAR ResText[256] = L"";
//     WCHAR LocaleText[256 * 2];
//
//     GetLocaleInfoW(GetUserDefaultLCID(), LOCALE_SLANGUAGE, CurLocale, ARRAYSIZE!(CurLocale));
//     GetGeoInfoW(GetUserGeoID(GEOCLASS_NATION), GEO_FRIENDLYNAME, CurGeo, ARRAYSIZE!(CurGeo), GetThreadLocale());
//
//     LoadStringW(hDllInstance, IDS_LOCALETEXT, ResText, ARRAYSIZE!(ResText));
//     StringCchPrintfW(LocaleText, ARRAYSIZE!(LocaleText), ResText, CurLocale, CurGeo);
//
//     SetWindowTextW(hwnd, LocaleText);
// }
//
// static VOID
// SetKeyboardLayoutName(HWND hwnd)
// {
//     HKL hkl;
//     BOOL LayoutSpecial = FALSE;
//     WCHAR LayoutPath[256];
//     WCHAR LocaleName[32];
//     WCHAR SpecialId[5] = L"";
//     WCHAR ResText[256] = L"";
//     DWORD dwValueSize;
//     HKEY hKey;
//     UINT i;
//
//     /* Get the default input language and method */
//     if (!SystemParametersInfoW(SPI_GETDEFAULTINPUTLANG, 0, (LPDWORD)&hkl, 0))
//     {
//         hkl = GetKeyboardLayout(0);
//     }
//
//     if ((HIWORD!(hkl) & 0xF000) == 0xF000)
//     {
//         /* Process keyboard layout with special id */
//         StringCchPrintfW(SpecialId, ARRAYSIZE!(SpecialId), L"%04x", (HIWORD!(hkl) & 0x0FFF));
//         LayoutSpecial = TRUE;
//     }
//
// #define MAX_LAYOUTS_PER_LANGID 0x10000
//     for (i = 0; i < (LayoutSpecial ? MAX_LAYOUTS_PER_LANGID : 1); i++)
//     {
//         /* Generate a hexadecimal identifier for keyboard layout registry key */
//         StringCchPrintfW(LocaleName, ARRAYSIZE!(LocaleName), L"%08lx", (i << 16) | LOWORD!(hkl));
//
//         StringCchCopyW(LayoutPath, ARRAYSIZE!(LayoutPath), L"SYSTEM\\CurrentControlSet\\Control\\Keyboard Layouts\\");
//         StringCchCatW(LayoutPath, ARRAYSIZE!(LayoutPath), LocaleName);
//         *LocaleName = UNICODE_NULL;
//
//         if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           LayoutPath,
//                           0,
//                           KEY_ALL_ACCESS,
//                           &hKey) == ERROR_SUCCESS)
//         {
//             /* Make sure the keyboard layout key we opened is the one we need.
//              * If the layout has no special id, just pass this check. */
//             dwValueSize = sizeof(LocaleName);
//             if (!LayoutSpecial ||
//                 ((RegQueryValueExW(hKey,
//                                    L"Layout Id",
//                                    NULL,
//                                    NULL,
//                                    (PVOID)&LocaleName,
//                                    &dwValueSize) == ERROR_SUCCESS) &&
//                 (wcscmp(LocaleName, SpecialId) == 0)))
//             {
//                 *LocaleName = UNICODE_NULL;
//                 dwValueSize = sizeof(LocaleName);
//                 RegQueryValueExW(hKey,
//                                  L"Layout Text",
//                                  NULL,
//                                  NULL,
//                                  (PVOID)&LocaleName,
//                                  &dwValueSize);
//                 /* Let the loop know where to stop */
//                 i = MAX_LAYOUTS_PER_LANGID;
//             }
//             RegCloseKey(hKey);
//         }
//         else
//         {
//             /* Keyboard layout registry keys are expected to go in order without gaps */
//             break;
//         }
//     }
// #undef MAX_LAYOUTS_PER_LANGID
//
//     LoadStringW(hDllInstance, IDS_LAYOUTTEXT, ResText, ARRAYSIZE!(ResText));
//     StringCchPrintfW(LayoutPath, ARRAYSIZE!(LayoutPath), ResText, LocaleName);
//
//     SetWindowTextW(hwnd, LayoutPath);
// }
//
//
// static BOOL
// RunControlPanelApplet(HWND hwnd, PCWSTR pwszCPLParameters)
// {
//     MSG msg;
//     HWND MainWindow = GetParent(hwnd);
//     STARTUPINFOW StartupInfo;
//     PROCESS_INFORMATION ProcessInformation;
//     WCHAR CmdLine[MAX_PATH] = L"rundll32.exe shell32.dll,Control_RunDLL ";
//
//     if (!pwszCPLParameters)
//     {
//         MessageBoxW(hwnd, L"Error: Failed to launch the Control Panel Applet.", NULL, MB_ICONERROR);
//         return FALSE;
//     }
//
//     ZeroMemory!(&StartupInfo, sizeof(StartupInfo));
//     StartupInfo.cb = sizeof(StartupInfo);
//     ZeroMemory!(&ProcessInformation, sizeof(ProcessInformation));
//
//     ASSERT(_countof!(CmdLine) > wcslen(CmdLine) + wcslen(pwszCPLParameters));
//     wcscat(CmdLine, pwszCPLParameters);
//
//     if (!CreateProcessW(NULL,
//                         CmdLine,
//                         NULL,
//                         NULL,
//                         FALSE,
//                         0,
//                         NULL,
//                         NULL,
//                         &StartupInfo,
//                         &ProcessInformation))
//     {
//         MessageBoxW(hwnd, L"Error: Failed to launch the Control Panel Applet.", NULL, MB_ICONERROR);
//         return FALSE;
//     }
//
//     /* Disable the Back and Next buttons and the main window
//      * while we're interacting with the control panel applet */
//     PropSheet_SetWizButtons!(MainWindow, 0);
//     EnableWindow(MainWindow, FALSE);
//
//     while ((MsgWaitForMultipleObjects(1, &ProcessInformation.hProcess, FALSE, INFINITE, QS_ALLINPUT|QS_ALLPOSTMESSAGE )) != WAIT_OBJECT_0)
//     {
//        /* We still need to process main window messages to avoid freeze */
//        while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE))
//        {
//            TranslateMessage(&msg);
//            DispatchMessageW(&msg);
//        }
//     }
//     CloseHandle(ProcessInformation.hThread);
//     CloseHandle(ProcessInformation.hProcess);
//
//     /* Enable the Back and Next buttons and the main window again */
//     PropSheet_SetWizButtons!(MainWindow, PSWIZB_BACK | PSWIZB_NEXT);
//     EnableWindow(MainWindow, TRUE);
//
//     return TRUE;
// }
//
//
// VOID
// EnableVisualTheme(
//      HWND hwndParent,
//      PCWSTR ThemeFile)
// {
//     enum { THEME_FILE, STYLE_FILE, UNKNOWN } fType;
//     WCHAR szPath[MAX_PATH]; // Expanded path of the file to use.
//     WCHAR szStyleFile[MAX_PATH];
//
//     fType = THEME_FILE; // Default to Classic theme.
//     if (ThemeFile)
//     {
//         /* Expand the path if possible */
//         if (ExpandEnvironmentStringsW(ThemeFile, szPath, _countof!(szPath)) != 0)
//             ThemeFile = szPath;
//
//         /* Determine the file type from its extension */
//         fType = UNKNOWN; {
//         PCWSTR pszExt = wcsrchr(ThemeFile, '.'); // PathFindExtensionW(ThemeFile);
//         if (pszExt)
//         {
//             if (_wcsicmp(pszExt, L".theme") == 0)
//                 fType = THEME_FILE;
//             else if (_wcsicmp(pszExt, L".msstyles") == 0)
//                 fType = STYLE_FILE;
//         } }
//         if (fType == UNKNOWN)
//         {
//             DPRINT1("EnableVisualTheme(): Unknown file '%S'\n", ThemeFile);
//             return;
//         }
//     }
//
//     DPRINT1("Applying visual %s '%S'\n",
//             (fType == THEME_FILE) ? "theme" : "style",
//             ThemeFile ? ThemeFile : L"(Classic)");
//
// //
// // TODO: Use instead uxtheme!SetSystemVisualStyle() once it is implemented,
// // https://stackoverflow.com/a/1036903
// // https://pinvoke.net/default.aspx/uxtheme.SetSystemVisualStyle
// // or ApplyTheme(NULL, 0, NULL) for restoring the classic theme.
// //
// // NOTE: The '/Action:ActivateMSTheme' is ReactOS-specific.
// //
//
//     if (ThemeFile && (fType == THEME_FILE))
//     {
//         /* Retrieve the visual style specified in the theme file.
//          * If none, fall back to the classic theme. */
//         if (GetPrivateProfileStringW(L"VisualStyles", L"Path", NULL,
//                                      szStyleFile, _countof!(szStyleFile), ThemeFile) && *szStyleFile)
//         {
//             /* Expand the path if possible */
//             ThemeFile = szStyleFile;
//             if (ExpandEnvironmentStringsW(ThemeFile, szPath, _countof!(szPath)) != 0)
//                 ThemeFile = szPath;
//         }
//         else
//         {
//             ThemeFile = NULL;
//         }
//
//         DPRINT1("--> Applying visual style '%S'\n",
//                 ThemeFile ? ThemeFile : L"(Classic)");
//     }
//
//     if (ThemeFile)
//     {
//         WCHAR wszParams[1024];
//         // FIXME: L"desk.cpl desk,@Appearance" regression, see commit 50d260a7f0
//         PCWSTR format = L"desk.cpl,,2 /Action:ActivateMSTheme /file:\"%s\"";
//
//         StringCchPrintfW(wszParams, _countof!(wszParams), format, ThemeFile);
//         RunControlPanelApplet(hwndParent, wszParams);
//     }
//     else
//     {
//         RunControlPanelApplet(hwndParent, L"desk.cpl,,2 /Action:ActivateMSTheme");
//     }
// }
//
//
// static VOID
// WriteUserLocale(VOID)
// {
//     HKEY hKey;
//     LCID lcid;
//     WCHAR Locale[9] = L"0000";
//
//     lcid = GetSystemDefaultLCID();
//
//     if (GetLocaleInfoW(MAKELCID!(lcid, SORT_DEFAULT), LOCALE_ILANGUAGE, &Locale[4], _countof!(Locale) - 4) != 0)
//     {
//         if (RegCreateKeyExW(HKEY_CURRENT_USER, L"Control Panel\\International",
//                             0, NULL, REG_OPTION_NON_VOLATILE,
//                             KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS)
//         {
//             RegSetValueExW(hKey, L"Locale", 0, REG_SZ, (LPBYTE)Locale, (wcslen(Locale) + 1) * sizeof(WCHAR));
//             RegCloseKey(hKey);
//         }
//     }
// }
//
// static INT_PTR __stdcall
// LocalePageDlgProc(HWND hwndDlg,
//                   UINT uMsg,
//                   WPARAM wParam,
//                   LPARAM lParam)
// {
//     PSETUPDATA SetupData;
//
//     /* Retrieve pointer to the global setup data */
//     SetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             /* Save pointer to the global setup data */
//             SetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, GWLP_USERDATA, (DWORD_PTR)SetupData);
//             WriteUserLocale();
//
//             SetUserLocaleName(GetDlgItem(hwndDlg, IDC_LOCALETEXT));
//             SetKeyboardLayoutName(GetDlgItem(hwndDlg, IDC_LAYOUTTEXT));
//         }
//         break;
//
//         case WM_COMMAND:
//             if (HIWORD!(wParam) == BN_CLICKED)
//             {
//                 switch (LOWORD!(wParam))
//                 {
//                     case IDC_CUSTOMLOCALE:
//                         RunControlPanelApplet(hwndDlg, L"intl.cpl,,5");
//                         SetUserLocaleName(GetDlgItem(hwndDlg, IDC_LOCALETEXT));
//                         break;
//
//                     case IDC_CUSTOMLAYOUT:
//                         RunControlPanelApplet(hwndDlg, L"input.dll,@1");
//                         SetKeyboardLayoutName(GetDlgItem(hwndDlg, IDC_LAYOUTTEXT));
//                         break;
//                 }
//             }
//             break;
//
//         case WM_NOTIFY:
//         {
//             LPNMHDR lpnm = (LPNMHDR)lParam;
//
//             switch (lpnm->code)
//             {
//                 case PSN_SETACTIVE:
//                     /* Enable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
//                     if (SetupData->UnattendSetup)
//                     {
//                         // if (!*SetupData->SourcePath)
//                         {
//                             RunControlPanelApplet(hwndDlg, L"intl.cpl,,/f:\"$winnt$.inf\""); // Should be in System32
//                         }
//
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_OWNERPAGE);
//                         return TRUE;
//                     }
//                     break;
//
//                 case PSN_WIZNEXT:
//                     break;
//
//                 case PSN_WIZBACK:
//                     SetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static PTIMEZONE_ENTRY
// GetLargerTimeZoneEntry(PSETUPDATA SetupData, DWORD Index)
// {
//     PTIMEZONE_ENTRY Entry;
//
//     Entry = SetupData->TimeZoneListHead;
//     while (Entry != NULL)
//     {
//         if (Entry->Index >= Index)
//             return Entry;
//
//         Entry = Entry->Next;
//     }
//
//     return NULL;
// }
//
// static LONG
// RetrieveTimeZone(
//      HKEY hZoneKey,
//      PVOID Context)
// {
//     LONG lError;
//     PSETUPDATA SetupData = (PSETUPDATA)Context;
//     PTIMEZONE_ENTRY Entry;
//     PTIMEZONE_ENTRY Current;
//     ULONG DescriptionSize;
//     ULONG StandardNameSize;
//     ULONG DaylightNameSize;
//
//     Entry = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(TIMEZONE_ENTRY));
//     if (Entry == NULL)
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     DescriptionSize  = sizeof(Entry->Description);
//     StandardNameSize = sizeof(Entry->StandardName);
//     DaylightNameSize = sizeof(Entry->DaylightName);
//
//     lError = QueryTimeZoneData(hZoneKey,
//                                &Entry->Index,
//                                &Entry->TimezoneInfo,
//                                Entry->Description,
//                                &DescriptionSize,
//                                Entry->StandardName,
//                                &StandardNameSize,
//                                Entry->DaylightName,
//                                &DaylightNameSize);
//     if (lError != ERROR_SUCCESS)
//     {
//         HeapFree(GetProcessHeap(), 0, Entry);
//         return lError;
//     }
//
//     if (SetupData->TimeZoneListHead == NULL &&
//         SetupData->TimeZoneListTail == NULL)
//     {
//         Entry->Prev = NULL;
//         Entry->Next = NULL;
//         SetupData->TimeZoneListHead = Entry;
//         SetupData->TimeZoneListTail = Entry;
//     }
//     else
//     {
//         Current = GetLargerTimeZoneEntry(SetupData, Entry->Index);
//         if (Current != NULL)
//         {
//             if (Current == SetupData->TimeZoneListHead)
//             {
//                 /* Prepend to head */
//                 Entry->Prev = NULL;
//                 Entry->Next = SetupData->TimeZoneListHead;
//                 SetupData->TimeZoneListHead->Prev = Entry;
//                 SetupData->TimeZoneListHead = Entry;
//             }
//             else
//             {
//                 /* Insert before current */
//                 Entry->Prev = Current->Prev;
//                 Entry->Next = Current;
//                 Current->Prev->Next = Entry;
//                 Current->Prev = Entry;
//             }
//         }
//         else
//         {
//             /* Append to tail */
//             Entry->Prev = SetupData->TimeZoneListTail;
//             Entry->Next = NULL;
//             SetupData->TimeZoneListTail->Next = Entry;
//             SetupData->TimeZoneListTail = Entry;
//         }
//     }
//
//     return ERROR_SUCCESS;
// }
//
// static VOID
// CreateTimeZoneList(PSETUPDATA SetupData)
// {
//     EnumerateTimeZoneList(RetrieveTimeZone, SetupData);
// }
//
// static VOID
// DestroyTimeZoneList(PSETUPDATA SetupData)
// {
//     PTIMEZONE_ENTRY Entry;
//
//     while (SetupData->TimeZoneListHead != NULL)
//     {
//         Entry = SetupData->TimeZoneListHead;
//
//         SetupData->TimeZoneListHead = Entry->Next;
//         if (SetupData->TimeZoneListHead != NULL)
//         {
//             SetupData->TimeZoneListHead->Prev = NULL;
//         }
//
//         HeapFree(GetProcessHeap(), 0, Entry);
//     }
//
//     SetupData->TimeZoneListTail = NULL;
// }
//
//
// static VOID
// ShowTimeZoneList(HWND hwnd, PSETUPDATA SetupData, DWORD dwEntryIndex)
// {
//     PTIMEZONE_ENTRY Entry;
//     DWORD dwIndex = 0;
//     DWORD dwCount;
//
//     GetTimeZoneListIndex(&mut dwEntryIndex);
//
//     Entry = SetupData->TimeZoneListHead;
//     while (Entry != NULL)
//     {
//         dwCount = SendMessage(hwnd,
//                               CB_ADDSTRING,
//                               0,
//                               (LPARAM)Entry->Description);
//
//         if (dwEntryIndex != 0 && dwEntryIndex == Entry->Index)
//             dwIndex = dwCount;
//
//         Entry = Entry->Next;
//     }
//
//     SendMessage(hwnd,
//                 CB_SETCURSEL,
//                 (WPARAM)dwIndex,
//                 0);
// }
//
//
// static VOID
// SetLocalTimeZone(HWND hwnd, PSETUPDATA SetupData)
// {
//     TIME_ZONE_INFORMATION TimeZoneInformation;
//     PTIMEZONE_ENTRY Entry;
//     DWORD dwIndex;
//     DWORD i;
//
//     dwIndex = SendMessage(hwnd,
//                           CB_GETCURSEL,
//                           0,
//                           0);
//
//     i = 0;
//     Entry = SetupData->TimeZoneListHead;
//     while (i < dwIndex)
//     {
//         if (Entry == NULL)
//             return;
//
//         i++;
//         Entry = Entry->Next;
//     }
//
//     wcscpy(TimeZoneInformation.StandardName,
//            Entry->StandardName);
//     wcscpy(TimeZoneInformation.DaylightName,
//            Entry->DaylightName);
//
//     TimeZoneInformation.Bias = Entry->TimezoneInfo.Bias;
//     TimeZoneInformation.StandardBias = Entry->TimezoneInfo.StandardBias;
//     TimeZoneInformation.DaylightBias = Entry->TimezoneInfo.DaylightBias;
//
//     memcpy(&TimeZoneInformation.StandardDate,
//            &Entry->TimezoneInfo.StandardDate,
//            sizeof(SYSTEMTIME));
//     memcpy(&TimeZoneInformation.DaylightDate,
//            &Entry->TimezoneInfo.DaylightDate,
//            sizeof(SYSTEMTIME));
//
//     /* Set time zone information */
//     SetTimeZoneInformation(&TimeZoneInformation);
// }
//
//
// static BOOL
// GetLocalSystemTime(HWND hwnd, PSETUPDATA SetupData)
// {
//     SYSTEMTIME Date;
//     SYSTEMTIME Time;
//
//     if (DateTime_GetSystemtime!(GetDlgItem(hwnd, IDC_DATEPICKER), &Date) != GDT_VALID)
//     {
//         return FALSE;
//     }
//
//     if (DateTime_GetSystemtime!(GetDlgItem(hwnd, IDC_TIMEPICKER), &Time) != GDT_VALID)
//     {
//         return FALSE;
//     }
//
//     SetupData->SystemTime.wYear = Date.wYear;
//     SetupData->SystemTime.wMonth = Date.wMonth;
//     SetupData->SystemTime.wDayOfWeek = Date.wDayOfWeek;
//     SetupData->SystemTime.wDay = Date.wDay;
//     SetupData->SystemTime.wHour = Time.wHour;
//     SetupData->SystemTime.wMinute = Time.wMinute;
//     SetupData->SystemTime.wSecond = Time.wSecond;
//     SetupData->SystemTime.wMilliseconds = Time.wMilliseconds;
//
//     return TRUE;
// }
//
//
// static BOOL
// SetSystemLocalTime(HWND hwnd, PSETUPDATA SetupData)
// {
//     BOOL Ret = FALSE;
//
//     /*
//      * Call SetLocalTime twice to ensure correct results
//      */
//     Ret = SetLocalTime(&SetupData->SystemTime) &&
//           SetLocalTime(&SetupData->SystemTime);
//
//     return Ret;
// }
//
//
// static VOID
// UpdateLocalSystemTime(HWND hwnd, SYSTEMTIME LocalTime)
// {
//     DateTime_SetSystemtime!(GetDlgItem(hwnd, IDC_DATEPICKER), GDT_VALID, &LocalTime);
//     DateTime_SetSystemtime!(GetDlgItem(hwnd, IDC_TIMEPICKER), GDT_VALID, &LocalTime);
// }
//
//
// static BOOL
// WriteDateTimeSettings(HWND hwndDlg, PSETUPDATA SetupData)
// {
//     WCHAR Title[64];
//     WCHAR ErrorLocalTime[256];
//
//     GetLocalSystemTime(hwndDlg, SetupData);
//     SetLocalTimeZone(GetDlgItem(hwndDlg, IDC_TIMEZONELIST),
//                      SetupData);
//
//     SetAutoDaylight(SendDlgItemMessage(hwndDlg, IDC_AUTODAYLIGHT,
//                                        BM_GETCHECK, 0, 0) != BST_UNCHECKED);
//     if (!SetSystemLocalTime(hwndDlg, SetupData))
//     {
//         if (0 == LoadStringW(hDllInstance, IDS_REACTOS_SETUP, Title, ARRAYSIZE!(Title)))
//         {
//             wcscpy(Title, L"ReactOS Setup");
//         }
//         if (0 == LoadStringW(hDllInstance, IDS_WZD_LOCALTIME, ErrorLocalTime,
//                              ARRAYSIZE!(ErrorLocalTime)))
//         {
//             wcscpy(ErrorLocalTime, L"Setup was unable to set the local time.");
//         }
//         MessageBoxW(hwndDlg, ErrorLocalTime, Title, MB_ICONWARNING | MB_OK);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static INT_PTR __stdcall
// DateTimePageDlgProc(HWND hwndDlg,
//                     UINT uMsg,
//                     WPARAM wParam,
//                     LPARAM lParam)
// {
//     PSETUPDATA SetupData;
//
//     /* Retrieve pointer to the global setup data */
//     SetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             /* Save pointer to the global setup data */
//             SetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, GWLP_USERDATA, (DWORD_PTR)SetupData);
//
//             CreateTimeZoneList(SetupData);
//
//             if (SetupData->UnattendSetup)
//             {
//                 ShowTimeZoneList(GetDlgItem(hwndDlg, IDC_TIMEZONELIST),
//                                  SetupData, SetupData->TimeZoneIndex);
//
//                 if (!SetupData->DisableAutoDaylightTimeSet)
//                 {
//                     SendDlgItemMessage(hwndDlg, IDC_AUTODAYLIGHT, BM_SETCHECK, (WPARAM)BST_CHECKED, 0);
//                 }
//             }
//             else
//             {
//                 ShowTimeZoneList(GetDlgItem(hwndDlg, IDC_TIMEZONELIST),
//                                  SetupData, -1);
//
//                 SendDlgItemMessage(hwndDlg, IDC_AUTODAYLIGHT, BM_SETCHECK, (WPARAM)BST_CHECKED, 0);
//             }
//             break;
//         }
//
//         case WM_TIMER:
//         {
//             SYSTEMTIME LocalTime;
//
//             GetLocalTime(&LocalTime);
//             UpdateLocalSystemTime(hwndDlg, LocalTime);
//
//             // Reset timeout.
//             SetTimer(hwndDlg, 1, 1000 - LocalTime.wMilliseconds, NULL);
//             break;
//         }
//
//         case WM_NOTIFY:
//             switch (((LPNMHDR)lParam)->code)
//             {
//                 case PSN_SETACTIVE:
//                 {
//                     SYSTEMTIME LocalTime;
//
//                     GetLocalTime(&LocalTime);
//                     UpdateLocalSystemTime(hwndDlg, LocalTime);
//
//                     /* Enable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
//
//                     if (SetupData->UnattendSetup && WriteDateTimeSettings(hwndDlg, SetupData))
//                     {
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_THEMEPAGE);
//                         return TRUE;
//                     }
//
//                     SetTimer(hwndDlg, 1, 1000 - LocalTime.wMilliseconds, NULL);
//                     break;
//                 }
//
//                 case PSN_KILLACTIVE:
//                 case DTN_DATETIMECHANGE:
//                     // NB: Not re-set until changing page (PSN_SETACTIVE).
//                     KillTimer(hwndDlg, 1);
//                     break;
//
//                 case PSN_WIZNEXT:
//                     WriteDateTimeSettings(hwndDlg, SetupData);
//                     break;
//
//                 case PSN_WIZBACK:
//                     SetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//             break;
//
//         case WM_DESTROY:
//             DestroyTimeZoneList(SetupData);
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// static struct ThemeInfo
// {
//     LPCWSTR PreviewBitmap;
//     UINT DisplayName;
//     LPCWSTR ThemeFile;
//
// } Themes[] = {
//     { MAKEINTRESOURCE!(IDB_CLASSIC), IDS_CLASSIC, NULL },
//     { MAKEINTRESOURCE!(IDB_LAUTUS), IDS_LAUTUS, L"themes\\lautus\\lautus.msstyles" },
//     { MAKEINTRESOURCE!(IDB_LUNAR), IDS_LUNAR, L"themes\\lunar\\lunar.msstyles" },
//     { MAKEINTRESOURCE!(IDB_MIZU), IDS_MIZU, L"themes\\mizu\\mizu.msstyles"},
// };
//
// static INT_PTR __stdcall
// ThemePageDlgProc(HWND hwndDlg,
//                     UINT uMsg,
//                     WPARAM wParam,
//                     LPARAM lParam)
// {
//     PSETUPDATA SetupData;
//     LPNMLISTVIEW pnmv;
//
//     /* Retrieve pointer to the global setup data */
//     SetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             HWND hListView;
//             HIMAGELIST himl;
//             DWORD n;
//             LVITEM lvi = {0};
//
//             /* Save pointer to the global setup data */
//             SetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, GWLP_USERDATA, (DWORD_PTR)SetupData);
//
//             hListView = GetDlgItem(hwndDlg, IDC_THEMEPICKER);
//
//             /* Common */
//             himl = ImageList_Create(180, 163, ILC_COLOR32 | ILC_MASK, ARRAYSIZE!(Themes), 1);
//             lvi.mask = LVIF_TEXT | LVIF_IMAGE |LVIF_STATE;
//
//             for (n = 0; n < ARRAYSIZE!(Themes); ++n)
//             {
//                 WCHAR DisplayName[100] = {0};
//                 /* Load the bitmap */
//                 HANDLE image = LoadImageW(hDllInstance, Themes[n].PreviewBitmap, IMAGE_BITMAP, 0, 0, LR_CREATEDIBSECTION);
//                 ImageList_AddMasked(himl, image, RGB!(255,0,255));
//
//                 /* Load the string */
//                 LoadStringW(hDllInstance, Themes[n].DisplayName, DisplayName, ARRAYSIZE!(DisplayName));
//                 DisplayName[ARRAYSIZE!(DisplayName)-1] = UNICODE_NULL;
//
//                 /* Add the listview item */
//                 lvi.iItem  = n;
//                 lvi.iImage = n;
//                 lvi.pszText = DisplayName;
//                 ListView_InsertItem!(hListView, &lvi);
//             }
//
//             /* Register the imagelist */
//             ListView_SetImageList!(hListView, himl, LVSIL_NORMAL);
//             /* Transparent background */
//             ListView_SetBkColor!(hListView, CLR_NONE);
//             ListView_SetTextBkColor!(hListView, CLR_NONE);
//             /* Reduce the size between the items */
//             ListView_SetIconSpacing!(hListView, 190, 173);
//             break;
//         }
//
//         case WM_NOTIFY:
//             switch (((LPNMHDR)lParam)->code)
//             {
//                 //case LVN_ITEMCHANGING:
//                 case LVN_ITEMCHANGED:
//                     pnmv = (LPNMLISTVIEW)lParam;
//                     if ((pnmv->uChanged & LVIF_STATE) && (pnmv->uNewState & LVIS_SELECTED))
//                     {
//                         int iTheme = pnmv->iItem;
//                         DPRINT1("Selected theme: %u\n", Themes[iTheme].DisplayName);
//
//                         if (Themes[iTheme].ThemeFile)
//                         {
//                             WCHAR wszTheme[MAX_PATH];
//                             SHGetFolderPathAndSubDirW(0, CSIDL_RESOURCES, NULL, SHGFP_TYPE_DEFAULT, Themes[iTheme].ThemeFile, wszTheme);
//                             EnableVisualTheme(hwndDlg, wszTheme);
//                         }
//                         else
//                         {
//                             EnableVisualTheme(hwndDlg, Themes[iTheme].ThemeFile);
//                         }
//                     }
//                     break;
//                 case PSN_SETACTIVE:
//                     /* Enable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
//                     if (SetupData->UnattendSetup)
//                     {
//                         SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, SetupData->uFirstNetworkWizardPage);
//                         return TRUE;
//                     }
//                     break;
//
//                 case PSN_WIZNEXT:
//                     break;
//
//                 case PSN_WIZBACK:
//                     SetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// static UINT __stdcall
// RegistrationNotificationProc(PVOID Context,
//                              UINT Notification,
//                              UINT_PTR Param1,
//                              UINT_PTR Param2)
// {
//     PREGISTRATIONDATA RegistrationData;
//     PSP_REGISTER_CONTROL_STATUSW StatusInfo;
//     UINT MessageID;
//
//     RegistrationData = (PREGISTRATIONDATA)Context;
//
//     if (Notification == SPFILENOTIFY_STARTREGISTRATION ||
//         Notification == SPFILENOTIFY_ENDREGISTRATION)
//     {
//         StatusInfo = (PSP_REGISTER_CONTROL_STATUSW) Param1;
//         RegistrationData->pNotify->CurrentItem = wcsrchr(StatusInfo->FileName, '\\');
//         if (RegistrationData->pNotify->CurrentItem == NULL)
//         {
//             RegistrationData->pNotify->CurrentItem = StatusInfo->FileName;
//         }
//         else
//         {
//             RegistrationData->pNotify->CurrentItem++;
//         }
//
//         if (Notification == SPFILENOTIFY_STARTREGISTRATION)
//         {
//             DPRINT("Received SPFILENOTIFY_STARTREGISTRATION notification for %S\n",
//                    StatusInfo->FileName);
//             RegistrationData->pNotify->Progress = RegistrationData->Registered;
//
//             DPRINT("RegisterDll: Start step %ld\n", RegistrationData->pNotify->Progress);
//             SendMessage(RegistrationData->hwndDlg, PM_STEP_START, 0, (LPARAM)RegistrationData->pNotify);
//         }
//         else
//         {
//             DPRINT("Received SPFILENOTIFY_ENDREGISTRATION notification for %S\n",
//                    StatusInfo->FileName);
//             DPRINT("Win32Error %u FailureCode %u\n", StatusInfo->Win32Error,
//                    StatusInfo->FailureCode);
//             if (StatusInfo->FailureCode != SPREG_SUCCESS)
//             {
//                 switch (StatusInfo->FailureCode)
//                 {
//                     case SPREG_LOADLIBRARY:
//                         MessageID = IDS_LOADLIBRARY_FAILED;
//                         break;
//                     case SPREG_GETPROCADDR:
//                         MessageID = IDS_GETPROCADDR_FAILED;
//                         break;
//                     case SPREG_REGSVR:
//                         MessageID = IDS_REGSVR_FAILED;
//                         break;
//                     case SPREG_DLLINSTALL:
//                         MessageID = IDS_DLLINSTALL_FAILED;
//                         break;
//                     case SPREG_TIMEOUT:
//                         MessageID = IDS_TIMEOUT;
//                         break;
//                     default:
//                         MessageID = IDS_REASON_UNKNOWN;
//                         break;
//                 }
//
//                 RegistrationData->pNotify->MessageID = MessageID;
//                 RegistrationData->pNotify->LastError = StatusInfo->Win32Error;
//             }
//             else
//             {
//                 RegistrationData->pNotify->MessageID = 0;
//                 RegistrationData->pNotify->LastError = ERROR_SUCCESS;
//             }
//
//             if (RegistrationData->Registered < RegistrationData->DllCount)
//             {
//                 RegistrationData->Registered++;
//             }
//
//             RegistrationData->pNotify->Progress = RegistrationData->Registered;
//             DPRINT("RegisterDll: End step %ld\n", RegistrationData->pNotify->Progress);
//             SendMessage(RegistrationData->hwndDlg, PM_STEP_END, 0, (LPARAM)RegistrationData->pNotify);
//         }
//
//         return FILEOP_DOIT;
//     }
//     else
//     {
//         DPRINT1("Received unexpected notification %u\n", Notification);
//         return SetupDefaultQueueCallback(RegistrationData->DefaultContext,
//                                          Notification, Param1, Param2);
//     }
// }
//
//
// static
// DWORD
// RegisterDlls(
//      PITEMSDATA pItemsData,
//      PREGISTRATIONNOTIFY pNotify)
// {
//     REGISTRATIONDATA RegistrationData;
//     WCHAR SectionName[512];
//     INFCONTEXT Context;
//     LONG DllCount = 0;
//     DWORD Error = NO_ERROR;
//
//     ZeroMemory!(&RegistrationData, sizeof(REGISTRATIONDATA));
//     RegistrationData.hwndDlg = pItemsData->hwndDlg;
//     RegistrationData.Registered = 0;
//
//     if (!SetupFindFirstLineW(hSysSetupInf, L"RegistrationPhase2",
//                              L"RegisterDlls", &Context))
//     {
//         DPRINT1("No RegistrationPhase2 section found\n");
//         return GetLastError();
//     }
//
//     if (!SetupGetStringFieldW(&Context, 1, SectionName,
//                               ARRAYSIZE!(SectionName),
//                               NULL))
//     {
//         DPRINT1("Unable to retrieve section name\n");
//         return GetLastError();
//     }
//
//     DllCount = SetupGetLineCountW(hSysSetupInf, SectionName);
//     DPRINT("SectionName %S DllCount %ld\n", SectionName, DllCount);
//     if (DllCount < 0)
//     {
//         return STATUS_NOT_FOUND;
//     }
//
//     RegistrationData.DllCount = (ULONG)DllCount;
//     RegistrationData.DefaultContext = SetupInitDefaultQueueCallback(RegistrationData.hwndDlg);
//     RegistrationData.pNotify = pNotify;
//
//     _SEH2_TRY
//     {
//         if (!SetupInstallFromInfSectionW(GetParent(RegistrationData.hwndDlg),
//                                          hSysSetupInf,
//                                          L"RegistrationPhase2",
//                                          SPINST_REGISTRY | SPINST_REGISTERCALLBACKAWARE | SPINST_REGSVR,
//                                          0,
//                                          NULL,
//                                          0,
//                                          RegistrationNotificationProc,
//                                          &RegistrationData,
//                                          NULL,
//                                          NULL))
//         {
//             Error = GetLastError();
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DPRINT("Catching exception\n");
//         Error = RtlNtStatusToDosError(_SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     SetupTermDefaultQueueCallback(RegistrationData.DefaultContext);
//
//     return Error;
// }
//
// static
// VOID
// RegisterComponents(
//     PITEMSDATA pItemsData)
// {
//     WCHAR SectionName[512];
//     INFCONTEXT Context;
//     LONG Steps = 0;
//     DWORD Error = NO_ERROR;
//     REGISTRATIONNOTIFY Notify;
//
//     ZeroMemory!(&Notify, sizeof(Notify));
//
//     /* Count the 'RegisterDlls' steps */
//     if (!SetupFindFirstLineW(hSysSetupInf, L"RegistrationPhase2",
//                              L"RegisterDlls", &Context))
//     {
//         DPRINT1("No RegistrationPhase2 section found\n");
//         return;
//     }
//
//     if (!SetupGetStringFieldW(&Context, 1, SectionName,
//                               ARRAYSIZE!(SectionName),
//                               NULL))
//     {
//         DPRINT1("Unable to retrieve section name\n");
//         return;
//     }
//
//     Steps += SetupGetLineCountW(hSysSetupInf, SectionName);
//
//     /* Count the 'TypeLibratries' steps */
//     Steps += SetupGetLineCountW(hSysSetupInf, L"TypeLibraries");
//
//     /* Start the item */
//     DPRINT("Register Components: %ld Steps\n", Steps);
//     SendMessage(pItemsData->hwndDlg, PM_ITEM_START, 0, (LPARAM)Steps);
//
//     Error = RegisterDlls(pItemsData, &Notify);
//     if (Error == ERROR_SUCCESS)
//         RegisterTypeLibraries(pItemsData, &Notify, hSysSetupInf, L"TypeLibraries");
//
//     /* End the item */
//     DPRINT("Register Components: done\n");
//     SendMessage(pItemsData->hwndDlg, PM_ITEM_END, 0, Error);
// }
//
//
// static
// DWORD
// __stdcall
// ItemCompletionThread(
//     LPVOID Parameter)
// {
//     PITEMSDATA pItemsData;
//     HWND hwndDlg;
//
//     pItemsData = (PITEMSDATA)Parameter;
//     hwndDlg = pItemsData->hwndDlg;
//
//     /* Step 0 - Registering components */
//     RegisterComponents(pItemsData);
//
//     /* Step 1 - Installing start menu items */
//     InstallStartMenuItems(pItemsData);
//
//     /* FIXME: Add completion steps here! */
//
//     // FIXME: Move this call to a separate cleanup page!
//     RtlCreateBootStatusDataFile();
//
//     /* Free the items data */
//     HeapFree(GetProcessHeap(), 0, pItemsData);
//
//     /* Tell the wizard page that we are done */
//     PostMessage(hwndDlg, PM_ITEMS_DONE, 0, 0);
//
//     return 0;
// }
//
//
// static
// BOOL
// RunItemCompletionThread(
//      HWND hwndDlg)
// {
//     HANDLE hCompletionThread;
//     PITEMSDATA pItemsData;
//
//     pItemsData = HeapAlloc(GetProcessHeap(), 0, sizeof(ITEMSDATA));
//     if (pItemsData == NULL)
//         return FALSE;
//
//     pItemsData->hwndDlg = hwndDlg;
//
//     hCompletionThread = CreateThread(NULL,
//                                      0,
//                                      ItemCompletionThread,
//                                      pItemsData,
//                                      0,
//                                      NULL);
//     if (hCompletionThread == NULL)
//     {
//         HeapFree(GetProcessHeap(), 0, pItemsData);
//     }
//     else
//     {
//         CloseHandle(hCompletionThread);
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static
// VOID
// ShowItemError(
//     HWND hwndDlg,
//     DWORD LastError)
// {
//     LPWSTR ErrorMessage = NULL;
//     WCHAR UnknownError[84];
//     WCHAR Title[64];
//
//     if (FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
//                        NULL, LastError, 0, ErrorMessage, 0, NULL) == 0)
//     {
//         if (LoadStringW(hDllInstance, IDS_UNKNOWN_ERROR,
//                         UnknownError,
//                         ARRAYSIZE!(UnknownError) - 20) == 0)
//         {
//             wcscpy(UnknownError, L"Unknown error");
//         }
//         wcscat(UnknownError, L" ");
//         _ultow(LastError, UnknownError + wcslen(UnknownError), 10);
//         ErrorMessage = UnknownError;
//     }
//
//     if (ErrorMessage != NULL)
//     {
//         if (LoadStringW(hDllInstance, IDS_REACTOS_SETUP,
//                         Title, ARRAYSIZE!(Title)) == 0)
//         {
//             wcscpy(Title, L"ReactOS Setup");
//         }
//
//         MessageBoxW(hwndDlg, ErrorMessage, Title, MB_ICONERROR | MB_OK);
//     }
//
//     if (ErrorMessage != NULL &&
//         ErrorMessage != UnknownError)
//     {
//         LocalFree(ErrorMessage);
//     }
// }
//
//
// static
// VOID
// ShowStepError(
//     HWND hwndDlg,
//     PREGISTRATIONNOTIFY RegistrationNotify)
// {
//     WCHAR ErrorMessage[128];
//     WCHAR Title[64];
//
//     if (LoadStringW(hDllInstance, RegistrationNotify->MessageID,
//                     ErrorMessage,
//                     ARRAYSIZE!(ErrorMessage)) == 0)
//     {
//         ErrorMessage[0] = 0u16;
//     }
//
//     if (RegistrationNotify->MessageID != IDS_TIMEOUT)
//     {
//         FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM, NULL,
//                        RegistrationNotify->LastError, 0,
//                        ErrorMessage + wcslen(ErrorMessage),
//                        ARRAYSIZE!(ErrorMessage) - wcslen(ErrorMessage),
//                        NULL);
//     }
//
//     if (ErrorMessage[0] != 0u16)
//     {
//         if (LoadStringW(hDllInstance, IDS_REACTOS_SETUP,
//                         Title, ARRAYSIZE!(Title)) == 0)
//         {
//             wcscpy(Title, L"ReactOS Setup");
//         }
//
//         MessageBoxW(hwndDlg, ErrorMessage,
//                     Title, MB_ICONERROR | MB_OK);
//     }
// }
//
//
// static INT_PTR __stdcall
// ProcessPageDlgProc(HWND hwndDlg,
//                    UINT uMsg,
//                    WPARAM wParam,
//                    LPARAM lParam)
// {
//     PSETUPDATA SetupData;
//     PREGISTRATIONNOTIFY RegistrationNotify;
//     static HICON s_hCheckIcon, s_hArrowIcon, s_hCrossIcon;
//     static HFONT s_hNormalFont;
//
//     /* Retrieve pointer to the global setup data */
//     SetupData = (PSETUPDATA)GetWindowLongPtr(hwndDlg, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             /* Save pointer to the global setup data */
//             SetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, GWLP_USERDATA, (DWORD_PTR)SetupData);
//             ShowWindow(GetDlgItem(hwndDlg, IDC_TASKTEXT3), SW_HIDE);
//             ShowWindow(GetDlgItem(hwndDlg, IDC_TASKTEXT4), SW_HIDE);
//             ShowWindow(GetDlgItem(hwndDlg, IDC_CHECK3), SW_HIDE);
//             ShowWindow(GetDlgItem(hwndDlg, IDC_CHECK4), SW_HIDE);
//             s_hCheckIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_CHECKICON), IMAGE_ICON, 16, 16, 0);
//             s_hArrowIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_ARROWICON), IMAGE_ICON, 16, 16, 0);
//             s_hCrossIcon = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDI_CROSSICON), IMAGE_ICON, 16, 16, 0);
//             s_hNormalFont = (HFONT)SendDlgItemMessage(hwndDlg, IDC_TASKTEXT1, WM_GETFONT, 0, 0);
//             break;
//
//         case WM_DESTROY:
//             DestroyIcon(s_hCheckIcon);
//             DestroyIcon(s_hArrowIcon);
//             DestroyIcon(s_hCrossIcon);
//             break;
//
//         case WM_NOTIFY:
//             switch (((LPNMHDR)lParam)->code)
//             {
//                 case PSN_SETACTIVE:
//                     LogItem!(L"BEGIN", L"ProcessPage");
//
//                     /* Disable the Back and Next buttons */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), 0);
//                     RunItemCompletionThread(hwndDlg);
//                     break;
//
//                 case PSN_WIZNEXT:
//                     LogItem!(L"END", L"ProcessPage");
//                     break;
//
//                 case PSN_WIZBACK:
//                     SetupData->UnattendSetup = FALSE;
//                     break;
//
//                 default:
//                     break;
//             }
//             break;
//
//         case PM_ITEM_START:
//             DPRINT("PM_ITEM_START %lu\n", (ULONG)lParam);
//             SendDlgItemMessage(hwndDlg, IDC_PROCESSPROGRESS, PBM_SETRANGE, 0, MAKELPARAM!(0, (ULONG)lParam));
//             SendDlgItemMessage(hwndDlg, IDC_PROCESSPROGRESS, PBM_SETPOS, 0, 0);
//             SendDlgItemMessage(hwndDlg, IDC_TASKTEXT1 + wParam, WM_SETFONT, (WPARAM)SetupData->hBoldFont, (LPARAM)TRUE);
//             SendDlgItemMessage(hwndDlg, IDC_CHECK1 + wParam, STM_SETIMAGE, IMAGE_ICON, (LPARAM)s_hArrowIcon);
//             break;
//
//         case PM_ITEM_END:
//             DPRINT("PM_ITEM_END\n");
//             SendDlgItemMessage(hwndDlg, IDC_TASKTEXT1 + wParam, WM_SETFONT, (WPARAM)s_hNormalFont, (LPARAM)TRUE);
//             if (lParam == ERROR_SUCCESS)
//             {
//                 SendDlgItemMessage(hwndDlg, IDC_CHECK1 + wParam, STM_SETIMAGE, IMAGE_ICON, (LPARAM)s_hCheckIcon);
//             }
//             else
//             {
//                 SendDlgItemMessage(hwndDlg, IDC_CHECK1 + wParam, STM_SETIMAGE, IMAGE_ICON, (LPARAM)s_hCrossIcon);
//                 ShowItemError(hwndDlg, (DWORD)lParam);
//             }
//             break;
//
//         case PM_STEP_START:
//             DPRINT("PM_STEP_START\n");
//             RegistrationNotify = (PREGISTRATIONNOTIFY)lParam;
//             SendDlgItemMessage(hwndDlg, IDC_ITEM, WM_SETTEXT, 0,
//                                (LPARAM)((RegistrationNotify->CurrentItem != NULL)? RegistrationNotify->CurrentItem : L""));
//             break;
//
//         case PM_STEP_END:
//             DPRINT("PM_STEP_END\n");
//             RegistrationNotify = (PREGISTRATIONNOTIFY)lParam;
//             SendDlgItemMessage(hwndDlg, IDC_PROCESSPROGRESS, PBM_SETPOS, RegistrationNotify->Progress, 0);
//             if (RegistrationNotify->LastError != ERROR_SUCCESS)
//             {
//                 ShowStepError(hwndDlg, RegistrationNotify);
//             }
//             break;
//
//         case PM_ITEMS_DONE:
//             DPRINT("PM_ITEMS_DONE\n");
//             /* Enable the Back and Next buttons */
//             PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
//             PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_NEXT);
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static VOID
// SetInstallationCompleted(VOID)
// {
//     HKEY hKey = 0;
//     DWORD InProgress = 0;
//     DWORD InstallDate;
//
//     if (RegOpenKeyExW( HKEY_LOCAL_MACHINE,
//                        L"SYSTEM\\Setup",
//                        0,
//                        KEY_WRITE,
//                        &hKey ) == ERROR_SUCCESS)
//     {
//         RegSetValueExW( hKey, L"SystemSetupInProgress", 0, REG_DWORD, (LPBYTE)&InProgress, sizeof(InProgress) );
//         RegCloseKey( hKey );
//     }
//
//     if (RegOpenKeyExW( HKEY_LOCAL_MACHINE,
//                        L"Software\\Microsoft\\Windows NT\\CurrentVersion",
//                        0,
//                        KEY_WRITE,
//                        &hKey ) == ERROR_SUCCESS)
//     {
//         InstallDate = (DWORD)time(NULL);
//         RegSetValueExW( hKey, L"InstallDate", 0, REG_DWORD, (LPBYTE)&InstallDate, sizeof(InstallDate) );
//         RegCloseKey( hKey );
//     }
// }
//
// static INT_PTR __stdcall
// FinishDlgProc(HWND hwndDlg,
//               UINT uMsg,
//               WPARAM wParam,
//               LPARAM lParam)
// {
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             /* Get pointer to the global setup data */
//             PSETUPDATA SetupData = (PSETUPDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//
//             if (!SetupData->UnattendSetup || !SetupData->DisableGeckoInst)
//             {
//                 /* Run the Wine Gecko prompt */
//                 Control_RunDLLW(hwndDlg, 0, L"appwiz.cpl,,install_gecko", SW_SHOW);
//             }
//
//             /* Set title font */
//             SendDlgItemMessage(hwndDlg,
//                                IDC_FINISHTITLE,
//                                WM_SETFONT,
//                                (WPARAM)SetupData->hTitleFont,
//                                (LPARAM)TRUE);
//             if (SetupData->UnattendSetup)
//             {
//                 KillTimer(hwndDlg, 1);
//                 SetInstallationCompleted();
//                 PostQuitMessage(0);
//             }
//         }
//         break;
//
//         case WM_DESTROY:
//         {
//             SetInstallationCompleted();
//             PostQuitMessage(0);
//             return TRUE;
//         }
//
//         case WM_TIMER:
//         {
//             INT Position;
//             HWND hWndProgress;
//
//             hWndProgress = GetDlgItem(hwndDlg, IDC_RESTART_PROGRESS);
//             Position = SendMessage(hWndProgress, PBM_GETPOS, 0, 0);
//             if (Position == 300)
//             {
//                 KillTimer(hwndDlg, 1);
//                 PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_FINISH);
//             }
//             else
//             {
//                 SendMessage(hWndProgress, PBM_SETPOS, Position + 1, 0);
//             }
//         }
//         return TRUE;
//
//         case WM_NOTIFY:
//         {
//             LPNMHDR lpnm = (LPNMHDR)lParam;
//
//             switch (lpnm->code)
//             {
//                 case PSN_SETACTIVE:
//                     /* Enable the correct buttons on for the active page */
//                     PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH);
//
//                     SendDlgItemMessage(hwndDlg, IDC_RESTART_PROGRESS, PBM_SETRANGE, 0,
//                                        MAKELPARAM!(0, 300));
//                     SendDlgItemMessage(hwndDlg, IDC_RESTART_PROGRESS, PBM_SETPOS, 0, 0);
//                     SetTimer(hwndDlg, 1, 50, NULL);
//                     break;
//
//                 case PSN_WIZFINISH:
//                     DestroyWindow(GetParent(hwndDlg));
//                     break;
//
//                 default:
//                     break;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// /*
//  * GetInstallSourceWin32 retrieves the path to the ReactOS installation medium
//  * in Win32 format, for later use by syssetup and storage in the registry.
//  */
// static BOOL
// GetInstallSourceWin32(
//      PWSTR pwszPath,
//      DWORD cchPathMax,
//      PCWSTR pwszNTPath)
// {
//     WCHAR wszDrives[512];
//     WCHAR wszNTPath[512]; // MAX_PATH ?
//     DWORD cchDrives;
//     PWCHAR pwszDrive;
//
//     *pwszPath = UNICODE_NULL;
//
//     cchDrives = GetLogicalDriveStringsW(_countof!(wszDrives) - 1, wszDrives);
//     if (cchDrives == 0 || cchDrives >= _countof!(wszDrives))
//     {
//         /* Buffer too small or failure */
//         LogItem!(NULL, L"GetLogicalDriveStringsW failed");
//         return FALSE;
//     }
//
//     for (pwszDrive = wszDrives; *pwszDrive; pwszDrive += wcslen(pwszDrive) + 1)
//     {
//         WCHAR wszBuf[MAX_PATH];
//
//         /* Retrieve the NT path corresponding to the current Win32 DOS path */
//         pwszDrive[2] = UNICODE_NULL; // Temporarily remove the backslash
//         QueryDosDeviceW(pwszDrive, wszNTPath, _countof!(wszNTPath));
//         pwszDrive[2] = '\\';        // Restore the backslash
//
//         wcscat(wszNTPath, L"\\");    // Concat a backslash
//
//         /* Logging */
//         wsprintf(wszBuf, L"Testing '%s' --> '%s' %s a CD",
//                  pwszDrive, wszNTPath,
//                  (GetDriveTypeW(pwszDrive) == DRIVE_CDROM) ? L"is" : L"is not");
//         LogItem!(NULL, wszBuf);
//
//         /* Check whether the NT path corresponds to the NT installation source path */
//         if (!_wcsicmp(wszNTPath, pwszNTPath))
//         {
//             /* Found it! */
//             wcscpy(pwszPath, pwszDrive); // cchPathMax
//
//             /* Logging */
//             wsprintf(wszBuf, L"GetInstallSourceWin32: %s", pwszPath);
//             LogItem!(NULL, wszBuf);
//             wcscat(wszBuf, L"\n");
//             OutputDebugStringW(wszBuf);
//
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// VOID
// ProcessUnattendSection(
//       PSETUPDATA pSetupData)
// {
//     INFCONTEXT InfContext;
//     WCHAR szName[256];
//     WCHAR szValue[MAX_PATH];
//     DWORD LineLength;
//     HKEY hKey;
//
//     if (!SetupFindFirstLineW(pSetupData->hSetupInf,
//                              L"Unattend",
//                              L"UnattendSetupEnabled",
//                              &InfContext))
//     {
//         DPRINT1("Error: Cannot find UnattendSetupEnabled Key! %d\n", GetLastError());
//         return;
//     }
//
//     if (!SetupGetStringFieldW(&InfContext,
//                               1,
//                               szValue,
//                               ARRAYSIZE!(szValue),
//                               &LineLength))
//     {
//         DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//         return;
//     }
//
//     if (_wcsicmp(szValue, L"yes") != 0)
//     {
//         DPRINT("Unattend setup was disabled by UnattendSetupEnabled key.\n");
//         return;
//     }
//
//     pSetupData->UnattendSetup = TRUE;
//
//     if (!SetupFindFirstLineW(pSetupData->hSetupInf,
//                              L"Unattend",
//                              NULL,
//                              &InfContext))
//     {
//         DPRINT1("Error: SetupFindFirstLine failed %d\n", GetLastError());
//         return;
//     }
//
//     do
//     {
//         if (!SetupGetStringFieldW(&InfContext,
//                                   0,
//                                   szName,
//                                   ARRAYSIZE!(szName),
//                                   &LineLength))
//         {
//             DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//             return;
//         }
//
//         if (!SetupGetStringFieldW(&InfContext,
//                                   1,
//                                   szValue,
//                                   ARRAYSIZE!(szValue),
//                                   &LineLength))
//         {
//             DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//             return;
//         }
//         DPRINT1("Name %S Value %S\n", szName, szValue);
//         if (!_wcsicmp(szName, L"FullName"))
//         {
//             if (ARRAYSIZE!(pSetupData->OwnerName) > LineLength)
//             {
//                 wcscpy(pSetupData->OwnerName, szValue);
//             }
//         }
//         else if (!_wcsicmp(szName, L"OrgName"))
//         {
//             if (ARRAYSIZE!(pSetupData->OwnerOrganization) > LineLength)
//             {
//                 wcscpy(pSetupData->OwnerOrganization, szValue);
//             }
//         }
//         else if (!_wcsicmp(szName, L"ComputerName"))
//         {
//             if (ARRAYSIZE!(pSetupData->ComputerName) > LineLength)
//             {
//                 wcscpy(pSetupData->ComputerName, szValue);
//             }
//         }
//         else if (!_wcsicmp(szName, L"AdminPassword"))
//         {
//             if (ARRAYSIZE!(pSetupData->AdminPassword) > LineLength)
//             {
//                 wcscpy(pSetupData->AdminPassword, szValue);
//             }
//         }
//         else if (!_wcsicmp(szName, L"TimeZoneIndex"))
//         {
//             pSetupData->TimeZoneIndex = _wtoi(szValue);
//         }
//         else if (!_wcsicmp(szName, L"DisableAutoDaylightTimeSet"))
//         {
//             pSetupData->DisableAutoDaylightTimeSet = _wtoi(szValue);
//         }
//         else if (!_wcsicmp(szName, L"DisableGeckoInst"))
//         {
//             if (!_wcsicmp(szValue, L"yes"))
//                 pSetupData->DisableGeckoInst = TRUE;
//             else
//                 pSetupData->DisableGeckoInst = FALSE;
//         }
//         else if (!_wcsicmp(szName, L"ProductOption"))
//         {
//             pSetupData->ProductOption = (PRODUCT_OPTION)_wtoi(szValue);
//         }
//     } while (SetupFindNextLine(&InfContext, &InfContext));
//
//     if (SetupFindFirstLineW(pSetupData->hSetupInf,
//                             L"Display",
//                             NULL,
//                             &InfContext))
//     {
//         DEVMODEW dm = { { 0 } };
//         dm.dmSize = sizeof(dm);
//         if (EnumDisplaySettingsW(NULL, ENUM_CURRENT_SETTINGS, &dm))
//         {
//             do
//             {
//                 int iValue;
//                 if (!SetupGetStringFieldW(&InfContext,
//                                           0,
//                                           szName,
//                                           ARRAYSIZE!(szName),
//                                           &LineLength))
//                 {
//                     DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//                     return;
//                 }
//
//                 if (!SetupGetStringFieldW(&InfContext,
//                                           1,
//                                           szValue,
//                                           ARRAYSIZE!(szValue),
//                                           &LineLength))
//                 {
//                     DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//                     return;
//                 }
//                 iValue = _wtoi(szValue);
//                 DPRINT1("Name %S Value %i\n", szName, iValue);
//
//                 if (!iValue)
//                     continue;
//
//                 if (!_wcsicmp(szName, L"BitsPerPel"))
//                 {
//                     dm.dmFields |= DM_BITSPERPEL;
//                     dm.dmBitsPerPel = iValue;
//                 }
//                 else if (!_wcsicmp(szName, L"XResolution"))
//                 {
//                     dm.dmFields |= DM_PELSWIDTH;
//                     dm.dmPelsWidth = iValue;
//                 }
//                 else if (!_wcsicmp(szName, L"YResolution"))
//                 {
//                     dm.dmFields |= DM_PELSHEIGHT;
//                     dm.dmPelsHeight = iValue;
//                 }
//                 else if (!_wcsicmp(szName, L"VRefresh"))
//                 {
//                     dm.dmFields |= DM_DISPLAYFREQUENCY;
//                     dm.dmDisplayFrequency = iValue;
//                 }
//             } while (SetupFindNextLine(&InfContext, &InfContext));
//
//             ChangeDisplaySettingsW(&dm, CDS_UPDATEREGISTRY);
//         }
//     }
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
//                       0,
//                       KEY_SET_VALUE,
//                       &hKey) != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: failed to open HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce\n");
//         return;
//     }
//
//     if (SetupFindFirstLineW(pSetupData->hSetupInf,
//                             L"GuiRunOnce",
//                             NULL,
//                             &InfContext))
//     {
//         int i = 0;
//         do
//         {
//             if (SetupGetStringFieldW(&InfContext,
//                                      0,
//                                      szValue,
//                                      ARRAYSIZE!(szValue),
//                                      NULL))
//             {
//                 WCHAR szPath[MAX_PATH];
//                 swprintf(szName, L"%d", i);
//                 DPRINT("szName %S szValue %S\n", szName, szValue);
//
//                 if (ExpandEnvironmentStringsW(szValue, szPath, MAX_PATH))
//                 {
//                     DPRINT("value %S\n", szPath);
//                     if (RegSetValueExW(hKey,
//                                        szName,
//                                        0,
//                                        REG_SZ,
//                                        (const BYTE*)szPath,
//                                        (wcslen(szPath) + 1) * sizeof(WCHAR)) == ERROR_SUCCESS)
//                     {
//                         i++;
//                     }
//                 }
//             }
//         } while (SetupFindNextLine(&InfContext, &InfContext));
//     }
//
//     RegCloseKey(hKey);
//
//     if (SetupFindFirstLineW(pSetupData->hSetupInf,
//         L"Env",
//         NULL,
//         &InfContext))
//     {
//         if (RegCreateKeyExW(
//                 HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment", 0, NULL,
//                 REG_OPTION_NON_VOLATILE, KEY_WRITE | KEY_READ, NULL, &hKey, NULL) != ERROR_SUCCESS)
//         {
//             DPRINT1("Error: failed to open HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment\n");
//             return;
//         }
//         do
//         {
//             if (!SetupGetStringFieldW(&InfContext,
//                 0,
//                 szName,
//                 ARRAYSIZE!(szName),
//                 &LineLength))
//             {
//                 DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//                 return;
//             }
//
//             if (!SetupGetStringFieldW(&InfContext,
//                 1,
//                 szValue,
//                 ARRAYSIZE!(szValue),
//                 &LineLength))
//             {
//                 DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//                 return;
//             }
//             DPRINT1("[ENV] %S=%S\n", szName, szValue);
//
//             DWORD dwType = wcschr(szValue, '%') != NULL ? REG_EXPAND_SZ : REG_SZ;
//
//             if (RegSetValueExW(hKey, szName, 0, dwType, (const BYTE*)szValue, (DWORD)(wcslen(szValue) + 1) * sizeof(TCHAR)) != ERROR_SUCCESS)
//             {
//                 DPRINT1(" - Error %d\n", GetLastError());
//             }
//
//         } while (SetupFindNextLine(&InfContext, &InfContext));
//
//         RegCloseKey(hKey);
//     }
// }
//
// static BOOL
// PathIsEqual(
//      LPCWSTR lpPath1,
//      LPCWSTR lpPath2)
// {
//     WCHAR szPath1[MAX_PATH];
//     WCHAR szPath2[MAX_PATH];
//
//     /* If something goes wrong, better return TRUE,
//      * so the calling function returns early.
//      */
//     if (!PathCanonicalizeW(szPath1, lpPath1))
//         return TRUE;
//
//     if (!PathAddBackslashW(szPath1))
//         return TRUE;
//
//     if (!PathCanonicalizeW(szPath2, lpPath2))
//         return TRUE;
//
//     if (!PathAddBackslashW(szPath2))
//         return TRUE;
//
//     return (_wcsicmp(szPath1, szPath2) == 0);
// }
//
// static VOID
// AddInstallationSource(
//      HKEY hKey,
//      LPWSTR lpPath)
// {
//     LONG res;
//     DWORD dwRegType;
//     DWORD dwPathLength = 0;
//     DWORD dwNewLength = 0;
//     LPWSTR Buffer = NULL;
//     LPWSTR Path;
//
//     res = RegQueryValueExW(
//         hKey,
//         L"Installation Sources",
//         NULL,
//         &dwRegType,
//         NULL,
//         &dwPathLength);
//
//     if (res != ERROR_SUCCESS ||
//         dwRegType != REG_MULTI_SZ ||
//         dwPathLength == 0 ||
//         dwPathLength % sizeof(WCHAR) != 0)
//     {
//         dwPathLength = 0;
//         goto set;
//     }
//
//     /* Reserve space for existing data + new string */
//     dwNewLength = dwPathLength + (wcslen(lpPath) + 1) * sizeof(WCHAR);
//     Buffer = HeapAlloc(GetProcessHeap(), 0, dwNewLength);
//     if (!Buffer)
//         return;
//
//     ZeroMemory!(Buffer, dwNewLength);
//
//     res = RegQueryValueExW(
//         hKey,
//         L"Installation Sources",
//         NULL,
//         NULL,
//         (LPBYTE)Buffer,
//         &dwPathLength);
//
//     if (res != ERROR_SUCCESS)
//     {
//         HeapFree(GetProcessHeap(), 0, Buffer);
//         dwPathLength = 0;
//         goto set;
//     }
//
//     /* Sanity check, these should already be zeros */
//     Buffer[dwPathLength / sizeof(WCHAR) - 2] = UNICODE_NULL;
//     Buffer[dwPathLength / sizeof(WCHAR) - 1] = UNICODE_NULL;
//
//     for (Path = Buffer; *Path; Path += wcslen(Path) + 1)
//     {
//         /* Check if path is already added */
//         if (PathIsEqual(Path, lpPath))
//             goto cleanup;
//     }
//
//     Path = Buffer + dwPathLength / sizeof(WCHAR) - 1;
//
// set:
//     if (dwPathLength == 0)
//     {
//         dwNewLength = (wcslen(lpPath) + 1 + 1) * sizeof(WCHAR);
//         Buffer = HeapAlloc(GetProcessHeap(), 0, dwNewLength);
//         if (!Buffer)
//             return;
//
//         Path = Buffer;
//     }
//
//     StringCbCopyW(Path, dwNewLength - (Path - Buffer) * sizeof(WCHAR), lpPath);
//     Buffer[dwNewLength / sizeof(WCHAR) - 1] = UNICODE_NULL;
//
//     RegSetValueExW(
//         hKey,
//         L"Installation Sources",
//         0,
//         REG_MULTI_SZ,
//         (LPBYTE)Buffer,
//         dwNewLength);
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, Buffer);
// }
//
// VOID
// ProcessSetupInf(
//       PSETUPDATA pSetupData)
// {
//     WCHAR szPath[MAX_PATH];
//     WCHAR szValue[MAX_PATH];
//     INFCONTEXT InfContext;
//     DWORD LineLength;
//     HKEY hKey;
//     LONG res;
//
//     pSetupData->hSetupInf = INVALID_HANDLE_VALUE;
//
//     /* Retrieve the path of the setup INF */
//     GetSystemDirectoryW(szPath, _countof!(szPath));
//     wcscat(szPath, L"\\$winnt$.inf");
//
//     /* Open the setup INF */
//     pSetupData->hSetupInf = SetupOpenInfFileW(szPath,
//                                               NULL,
//                                               INF_STYLE_OLDNT,
//                                               NULL);
//     if (pSetupData->hSetupInf == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("Error: Cannot open the setup information file %S with error %d\n", szPath, GetLastError());
//         return;
//     }
//
//
//     /* Retrieve the NT source path from which the 1st-stage installer was run */
//     if (!SetupFindFirstLineW(pSetupData->hSetupInf,
//                              L"data",
//                              L"sourcepath",
//                              &InfContext))
//     {
//         DPRINT1("Error: Cannot find sourcepath Key! %d\n", GetLastError());
//         return;
//     }
//
//     if (!SetupGetStringFieldW(&InfContext,
//                               1,
//                               szValue,
//                               ARRAYSIZE!(szValue),
//                               &LineLength))
//     {
//         DPRINT1("Error: SetupGetStringField failed with %d\n", GetLastError());
//         return;
//     }
//
//     *pSetupData->SourcePath = UNICODE_NULL;
//
//     /* Close the setup INF as we are going to modify it manually */
//     if (pSetupData->hSetupInf != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(pSetupData->hSetupInf);
//
//
//     /* Find the installation source path in Win32 format */
//     if (!GetInstallSourceWin32(pSetupData->SourcePath,
//                                _countof!(pSetupData->SourcePath),
//                                szValue))
//     {
//         *pSetupData->SourcePath = UNICODE_NULL;
//     }
//
//     /* Save the path in Win32 format in the setup INF */
//     swprintf(szValue, L"\"%s\"", pSetupData->SourcePath);
//     WritePrivateProfileStringW(L"data", L"dospath", szValue, szPath);
//
//     /*
//      * Save it also in the registry, in the following keys:
//      * - HKLM\Software\Microsoft\Windows\CurrentVersion\Setup ,
//      *   values "SourcePath" and "ServicePackSourcePath" (REG_SZ);
//      * - HKLM\Software\Microsoft\Windows NT\CurrentVersion ,
//      *   value "SourcePath" (REG_SZ); set to the full path (e.g. D:\I386).
//      */
//
//     res = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                           L"Software\\Microsoft\\Windows\\CurrentVersion\\Setup",
//                           0, NULL,
//                           REG_OPTION_NON_VOLATILE,
//                           KEY_ALL_ACCESS, // KEY_WRITE
//                           NULL,
//                           &hKey,
//                           NULL);
//     if (res == ERROR_SUCCESS)
//     {
//         AddInstallationSource(hKey, pSetupData->SourcePath);
//
//         res = RegSetValueExW(hKey,
//                              L"SourcePath",
//                              0,
//                              REG_SZ,
//                              (LPBYTE)pSetupData->SourcePath,
//                              (wcslen(pSetupData->SourcePath) + 1) * sizeof(WCHAR));
//
//         res = RegSetValueExW(hKey,
//                              L"ServicePackSourcePath",
//                              0,
//                              REG_SZ,
//                              (LPBYTE)pSetupData->SourcePath,
//                              (wcslen(pSetupData->SourcePath) + 1) * sizeof(WCHAR));
//
//         RegCloseKey(hKey);
//     }
//
//
//     /* Now, re-open the setup INF (this must succeed) */
//     pSetupData->hSetupInf = SetupOpenInfFileW(szPath,
//                                               NULL,
//                                               INF_STYLE_OLDNT,
//                                               NULL);
//     if (pSetupData->hSetupInf == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("Error: Cannot open the setup information file %S with error %d\n", szPath, GetLastError());
//         return;
//     }
//
//     /* Process the unattended section of the setup file */
//     ProcessUnattendSection(pSetupData);
// }
//
// typedef DWORD(WINAPI *PFNREQUESTWIZARDPAGES)(PDWORD, HPROPSHEETPAGE *, PSETUPDATA);
//
// VOID
// InstallWizard(VOID)
// {
//     PROPSHEETHEADER psh = {0};
//     HPROPSHEETPAGE *phpage = NULL;
//     PROPSHEETPAGE psp = {0};
//     UINT nPages = 0;
//     HWND hWnd;
//     MSG msg;
//     PSETUPDATA pSetupData = NULL;
//     HMODULE hNetShell = NULL;
//     PFNREQUESTWIZARDPAGES pfn = NULL;
//     DWORD dwPageCount = 10, dwNetworkPageCount = 0;
//
//     LogItem!(L"BEGIN_SECTION", L"InstallWizard");
//
//     /* Allocate setup data */
//     pSetupData = HeapAlloc(GetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            sizeof(SETUPDATA));
//     if (pSetupData == NULL)
//     {
//         LogItem!(NULL, L"SetupData allocation failed!");
//         MessageBoxW(NULL,
//                     L"Setup failed to allocate global data!",
//                     L"ReactOS Setup",
//                     MB_ICONERROR | MB_OK);
//         goto done;
//     }
//     pSetupData->ProductOption = PRODUCT_OPTION_DEFAULT;
//
//     hNetShell = LoadLibraryW(L"netshell.dll");
//     if (hNetShell != NULL)
//     {
//         DPRINT("Netshell.dll loaded!\n");
//
//         pfn = (PFNREQUESTWIZARDPAGES)GetProcAddress(hNetShell,
//                                                     "NetSetupRequestWizardPages");
//         if (pfn != NULL)
//         {
//             pfn(&dwNetworkPageCount, NULL, NULL);
//             dwPageCount += dwNetworkPageCount;
//         }
//     }
//
//     DPRINT("PageCount: %lu\n", dwPageCount);
//
//     phpage = HeapAlloc(GetProcessHeap(),
//                        HEAP_ZERO_MEMORY,
//                        dwPageCount * sizeof(HPROPSHEETPAGE));
//     if (phpage == NULL)
//     {
//         LogItem!(NULL, L"Page array allocation failed!");
//         MessageBoxW(NULL,
//                     L"Setup failed to allocate page array!",
//                     L"ReactOS Setup",
//                     MB_ICONERROR | MB_OK);
//         goto done;
//     }
//
//     /* Process the $winnt$.inf setup file */
//     ProcessSetupInf(pSetupData);
//
//     /* Create the Welcome page */
//     psp.dwSize = sizeof(PROPSHEETPAGE);
//     psp.dwFlags = PSP_DEFAULT | PSP_HIDEHEADER;
//     psp.hInstance = hDllInstance;
//     psp.lParam = (LPARAM)pSetupData;
//     psp.pfnDlgProc = WelcomeDlgProc;
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_WELCOMEPAGE);
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the Acknowledgements page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_ACKTITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_ACKSUBTITLE);
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_ACKPAGE);
//     psp.pfnDlgProc = AckPageDlgProc;
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the Product page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_PRODUCTTITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_PRODUCTSUBTITLE);
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_PRODUCT);
//     psp.pfnDlgProc = ProductPageDlgProc;
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the Locale page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_LOCALETITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_LOCALESUBTITLE);
//     psp.pfnDlgProc = LocalePageDlgProc;
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_LOCALEPAGE);
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the Owner page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_OWNERTITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_OWNERSUBTITLE);
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_OWNERPAGE);
//     psp.pfnDlgProc = OwnerPageDlgProc;
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the Computer page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_COMPUTERTITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_COMPUTERSUBTITLE);
//     psp.pfnDlgProc = ComputerPageDlgProc;
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_COMPUTERPAGE);
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the DateTime page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_DATETIMETITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_DATETIMESUBTITLE);
//     psp.pfnDlgProc = DateTimePageDlgProc;
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_DATETIMEPAGE);
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the theme selection page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_THEMESELECTIONTITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_THEMESELECTIONSUBTITLE);
//     psp.pfnDlgProc = ThemePageDlgProc;
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_THEMEPAGE);
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     pSetupData->uFirstNetworkWizardPage = IDD_PROCESSPAGE;
//     pSetupData->uPostNetworkWizardPage = IDD_PROCESSPAGE;
//
//     if (pfn)
//     {
//         pfn(&dwNetworkPageCount, &phpage[nPages], pSetupData);
//         nPages += dwNetworkPageCount;
//     }
//
//     /* Create the Process page */
//     psp.dwFlags = PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
//     psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_PROCESSTITLE);
//     psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_PROCESSSUBTITLE);
//     psp.pfnDlgProc = ProcessPageDlgProc;
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_PROCESSPAGE);
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     /* Create the Finish page */
//     psp.dwFlags = PSP_DEFAULT | PSP_HIDEHEADER;
//     psp.pfnDlgProc = FinishDlgProc;
//     psp.pszTemplate = MAKEINTRESOURCE!(IDD_FINISHPAGE);
//     phpage[nPages++] = CreatePropertySheetPage(&psp);
//
//     ASSERT(nPages == dwPageCount);
//
//     /* Create the property sheet */
//     psh.dwSize = sizeof(PROPSHEETHEADER);
//     psh.dwFlags = PSH_WIZARD97 | PSH_WATERMARK | PSH_HEADER | PSH_MODELESS;
//     psh.hInstance = hDllInstance;
//     psh.hwndParent = NULL;
//     psh.nPages = nPages;
//     psh.nStartPage = 0;
//     psh.phpage = phpage;
//     psh.pszbmWatermark = MAKEINTRESOURCE!(IDB_WATERMARK);
//     psh.pszbmHeader = MAKEINTRESOURCE!(IDB_HEADER);
//
//     /* Create title font */
//     pSetupData->hTitleFont = CreateTitleFont();
//     pSetupData->hBoldFont  = CreateBoldFont();
//
//     /* Display the wizard */
//     hWnd = (HWND)PropertySheet(&psh);
//     ShowWindow(hWnd, SW_SHOW);
//
//     while (GetMessage(&msg, NULL, 0, 0))
//     {
//         if (!IsDialogMessage(hWnd, &msg))
//         {
//             TranslateMessage(&msg);
//             DispatchMessage(&msg);
//         }
//     }
//
//     DeleteObject(pSetupData->hBoldFont);
//     DeleteObject(pSetupData->hTitleFont);
//
//     if (pSetupData->hSetupInf != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(pSetupData->hSetupInf);
//
// done:
//     if (phpage != NULL)
//         HeapFree(GetProcessHeap(), 0, phpage);
//
//     if (hNetShell != NULL)
//         FreeLibrary(hNetShell);
//
//     if (pSetupData != NULL)
//         HeapFree(GetProcessHeap(), 0, pSetupData);
//
//     LogItem!(L"END_SECTION", L"InstallWizard");
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
