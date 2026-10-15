use crate::prelude::*;
use crate::rapps::geninst_cpp_h::IsZipFile;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0D896F4ECFCA: [WCHAR; 53] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0055, 0x006E, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x005C, 0x0000];
pub static CPP_COM_WSTR_1A217ADA0F04: [WCHAR; 21] = [0x0025, 0x0073, 0x0020, 0x002F, 0x0053, 0x0055, 0x0050, 0x0050, 0x0052, 0x0045, 0x0053, 0x0053, 0x004D, 0x0053, 0x0047, 0x0042, 0x004F, 0x0058, 0x0045, 0x0053, 0x0000];
pub static CPP_COM_WSTR_1AA03B0B214A: [WCHAR; 5] = [0x002E, 0x006D, 0x0073, 0x0069, 0x0000];
pub static CPP_COM_WSTR_28C29EE251FE: [WCHAR; 14] = [0x0043, 0x0061, 0x0074, 0x0065, 0x0067, 0x006F, 0x0072, 0x0079, 0x0043, 0x006F, 0x0075, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_97829A80D968: [WCHAR; 8] = [0x0053, 0x0048, 0x0045, 0x004C, 0x004C, 0x0033, 0x0032, 0x0000];
pub static CPP_COM_WSTR_A50DE1AFC4FA: [WCHAR; 28] = [0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_A53534ACF67B: [WCHAR; 15] = [0x0044, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_AB68B18B313D: [WCHAR; 13] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_B422CF60B57F: [WCHAR; 3] = [0x0030, 0x0030, 0x0000];
pub static CPP_COM_WSTR_B42E8543313E: [WCHAR; 3] = [0x002F, 0x0053, 0x0000];
pub static CPP_COM_WSTR_B951C1B4FB5A: [WCHAR; 12] = [0x002F, 0x0069, 0x0020, 0x0022, 0x0025, 0x0073, 0x0022, 0x0020, 0x002F, 0x0071, 0x006E, 0x0000];
pub static CPP_COM_WSTR_BB2F7BE27363: [WCHAR; 17] = [0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_CEA6748FA84E: [WCHAR; 25] = [0x002F, 0x0053, 0x0049, 0x004C, 0x0045, 0x004E, 0x0054, 0x0020, 0x002F, 0x0056, 0x0045, 0x0052, 0x0059, 0x0053, 0x0049, 0x004C, 0x0045, 0x004E, 0x0054, 0x0020, 0x002F, 0x0053, 0x0050, 0x002D, 0x0000];
pub static CPP_COM_WSTR_D2BA60B1608F: [WCHAR; 83] = [0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x005C, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x004C, 0x006F, 0x0067, 0x005C, 0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x005C, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_D587A5BCAA56: [WCHAR; 15] = [0x0054, 0x0079, 0x0070, 0x0065, 0x0073, 0x0053, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072, 0x0074, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_E2128F4CC323: [WCHAR; 20] = [0x0043, 0x0061, 0x0074, 0x0065, 0x0067, 0x006F, 0x0072, 0x0079, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_EC88463500FB: [WCHAR; 14] = [0x0020, 0x007A, 0x0069, 0x0070, 0x0066, 0x006C, 0x0064, 0x0072, 0x002E, 0x0064, 0x006C, 0x006C, 0x002C, 0x0000];
pub static CPP_COM_WSTR_ECAE79F7CE7F: [WCHAR; 9] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0000];
pub static CPP_COM_WSTR_FB47CCC40FE6: [WCHAR; 3] = [0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hLog: HANDLE = core::ptr::null_mut();

#[allow(non_snake_case)]

const FOLDERID_UserProgramFiles: GUID = GUID { Data1: 0x5CD7AEE2, Data2: 0x2219, Data3: 0x4A67, Data4: [0xB8, 0x5D, 0x6C, 0x9C, 0xE1, 0x56, 0x60, 0xCB] };
#[no_mangle]
pub unsafe extern "C" fn ErrorBox__0(hOwner: HWND, mut Error: UINT) -> UINT {
    if Error == 0 {
        Error = ERROR_INTERNAL_ERROR; // Note: geninst.cpp depends on this
    }
    let mut buf = [0u16; 400];
    let fmf: UINT = FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM;
    FormatMessageW(fmf, core::ptr::null(), Error, 0, buf.as_mut_ptr(), _countof!(buf), core::ptr::null_mut());
    MessageBoxW(hOwner, buf.as_ptr(), core::ptr::null(), MB_OK | MB_ICONSTOP);
    return Error;
}
#[no_mangle]
pub unsafe extern "C" fn CopyTextToClipboard(lpszText: LPCWSTR) {
    if OpenClipboard(core::ptr::null_mut()) == 0 {
        return;
    }

    let mut hr: HRESULT;
    let mut ClipBuffer: HGLOBAL;
    let mut Buffer: LPWSTR;
    let mut cchBuffer: DWORD;

    EmptyClipboard();
    cchBuffer = wcslen(lpszText) as DWORD + 1;
    ClipBuffer = GlobalAlloc(GMEM_DDESHARE, (cchBuffer as usize * core::mem::size_of::<u16>()) as SIZE_T);

    Buffer = GlobalLock(ClipBuffer) as LPWSTR;
    hr = StringCchCopyW(Buffer, cchBuffer as usize, lpszText);
    GlobalUnlock(ClipBuffer);

    if hr >= 0 {
        SetClipboardData(CF_UNICODETEXT, ClipBuffer);
    }

    CloseClipboard();
}

unsafe extern "system" fn NothingDlgProc(hDlg: HWND, uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM) -> INT_PTR {
    if uMsg == WM_CLOSE {
        (DestroyWindow(hDlg)) as _
    } else {
        FALSE as INT_PTR
    }
}
#[no_mangle]
pub unsafe extern "C" fn EmulateDialogReposition(hwnd: HWND) {
    #[allow(non_upper_case_globals)]
    static DlgTmpl: [DWORD; 6] = [
        WS_POPUP | WS_CAPTION | WS_SYSMENU,
        0,
        0,
        0,
        0,
        0,
    ];
    let hDlg: HWND = CreateDialogIndirectW!(
        core::ptr::null_mut(),
        core::ptr::addr_of!(DlgTmpl) as LPDLGTEMPLATE,
        core::ptr::null_mut(),
        core::mem::transmute(NothingDlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
    );
    if !hDlg.is_null() {
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect(hwnd, core::ptr::addr_of_mut!(r));
        if SetWindowPos(
            hDlg,
            hDlg,
            r.left,
            r.top,
            r.right - r.left,
            r.bottom - r.top,
            SWP_NOZORDER | SWP_NOACTIVATE,
        ) != 0
        {
            SendMessage(
                hDlg,
                DM_REPOSITION,
                0,
                0,
            );
            if GetWindowRect(hDlg, core::ptr::addr_of_mut!(r)) != 0 {
                SetWindowPos(
                    hwnd,
                    hwnd,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        SendMessage(
            hDlg,
            WM_CLOSE,
            0,
            0,
        );
    }
}

unsafe fn MAKEINTRESOURCEW(i: UINT) -> PCWSTR {
    i as usize as PCWSTR
}
#[no_mangle]
pub unsafe extern "C" fn ShowPopupMenuEx(hwnd: HWND, hwndOwner: HWND, MenuID: UINT, DefaultItem: UINT, Point: *mut POINT) {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hPopupMenu: HMENU;
    let mut ItemInfo: MENUITEMINFO = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();

    if MenuID != 0 {
        hMenu = LoadMenuW(hInst, MAKEINTRESOURCEW(MenuID));
        hPopupMenu = GetSubMenu(hMenu, 0);
    } else {
        hPopupMenu = GetMenu(hwnd);
    }

    ZeroMemory!(core::ptr::addr_of_mut!(ItemInfo) as *mut core::ffi::c_void, core::mem::size_of::<MENUITEMINFO>());
    ItemInfo.cbSize = core::mem::size_of::<MENUITEMINFO>() as UINT;
    ItemInfo.fMask = MIIM_STATE;

    GetMenuItemInfoW(hPopupMenu, DefaultItem, (FALSE) as BOOL, core::ptr::addr_of_mut!(ItemInfo));

    if (ItemInfo.fState & MFS_GRAYED) == 0 {
        SetMenuDefaultItem(hPopupMenu, DefaultItem, FALSE);
    }

    let mut Point = Point;
    if Point.is_null() {
        Point = core::ptr::addr_of_mut!(pt);
        GetCursorPos((Point).cast());
    }

    SetForegroundWindow(hwnd);
    TrackPopupMenu(hPopupMenu, 0, (*Point).x, (*Point).y, 0, hwndOwner, core::ptr::null_mut());

    if !hMenu.is_null() {
        DestroyMenu(hMenu);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassifyFile(Path: PCWSTR) -> UINT {
    let share: UINT = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    let hFile: HANDLE = CreateFileW(Path, GENERIC_READ, share, core::ptr::null_mut(), OPEN_EXISTING, 0, core::ptr::null_mut());
    if hFile != INVALID_HANDLE_VALUE {
        let mut buf: [u8; 8] = [0; 8];
        let mut io: DWORD = 0;
        if ReadFile(hFile, buf.as_mut_ptr() as LPVOID, core::mem::size_of::<[u8; 8]>() as DWORD, core::ptr::addr_of_mut!(io), core::ptr::null_mut()) == 0 || io != core::mem::size_of::<[u8; 8]>() as DWORD {
            buf[0] = 0;
        }
        CloseHandle(hFile);

        if buf[0] == 0xD0 && buf[1] == 0xCF && buf[2] == 0x11 && buf[3] == 0xE0 &&
            buf[4] == 0xA1 && buf[5] == 0xB1 && buf[6] == 0x1A && buf[7] == 0xE1
        {
            return MAKEWORD!(b'M', PERCEIVED_TYPE_APPLICATION) as UINT; // MSI
        }
        if buf[0] == b'M' || buf[0] == b'Z' {
            let mut shfi: SHFILEINFO = core::mem::zeroed();
            if SHGetFileInfoW(Path, 0, core::ptr::addr_of_mut!(shfi), core::mem::size_of::<SHFILEINFO>() as UINT, SHGFI_EXETYPE) != 0 {
                return MAKEWORD!(b'E', PERCEIVED_TYPE_APPLICATION) as UINT;
            }
        }
        if buf[0] == b'M' && buf[1] == b'S' && buf[2] == b'C' && buf[3] == b'F' {
            return MAKEWORD!(b'C', PERCEIVED_TYPE_COMPRESSED) as UINT; // CAB
        }
    }

    if IsZipFile(Path) != 0 { // .zip last because we want to return SFX.exe with higher priority
        return MAKEWORD!(b'Z', PERCEIVED_TYPE_COMPRESSED) as UINT;
    }
    return (PERCEIVED_TYPE_UNKNOWN) as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn OpensWithExplorer(Path: PCWSTR) -> BOOL {
    let mut szCmd = [0u16; (MAX_PATH * 2) as _];
    let mut cch: DWORD = _countof!(szCmd);
    let pszExt: PCWSTR = PathFindExtensionW(Path);
    let mut hr: HRESULT = AssocQueryStringW(ASSOCF_INIT_IGNOREUNKNOWN | ASSOCF_NOTRUNCATE,
                                   ASSOCSTR_COMMAND, pszExt, core::ptr::null_mut(), szCmd.as_mut_ptr(), core::ptr::addr_of_mut!(cch));
    if ((hr) >= 0) && !StrStrIW(szCmd.as_ptr(), CPP_COM_WSTR_EC88463500FB.as_ptr()).is_null() // .zip
    {
        return (TRUE) as BOOL;
    }
    PathRemoveArgsW(szCmd.as_mut_ptr());
    return (((hr) >= 0) && (StrCmpIW(PathFindFileNameW(szCmd.as_mut_ptr()), CPP_COM_WSTR_AB68B18B313D.as_ptr()) == 0)) as BOOL; // .cab
}
#[no_mangle]
pub unsafe extern "C" fn WaitForProcess(hProcess: HANDLE) -> UINT {
    let mut code: DWORD = (STILL_ACTIVE) as u32;
    loop {
        let wait: DWORD = MsgWaitForMultipleObjects(1, core::ptr::addr_of!(hProcess), (FALSE) as BOOL, INFINITE, QS_ALLEVENTS);
        if wait == WAIT_OBJECT_0 + 1 {
            let mut msg: MSG = core::mem::zeroed();
            while PeekMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(core::ptr::addr_of_mut!(msg));
                DispatchMessageW(core::ptr::addr_of_mut!(msg));
            }
        } else {
            // TODO: if (wait == WAIT_OBJECT_0) GetExitCodeProcess for MSI reboot codes
            break;
        }
    }
    return code;
}
#[no_mangle]
pub unsafe extern "C" fn StartProcess(Path: &CStringW, Wait: BOOL) -> BOOL {
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut si: STARTUPINFOW = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(si) as LPVOID, core::mem::size_of::<STARTUPINFOW>());
    si.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = (SW_SHOW as u16);

    // The Unicode version of CreateProcess can modify the contents of this string.
    let mut Tmp = CStringW::from_ptr(Path.GetString());
    let fSuccess: BOOL = CreateProcessW(core::ptr::null_mut(), Tmp.GetBuffer(0), core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as BOOL, 0, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(si), core::ptr::addr_of_mut!(pi));
    Tmp.ReleaseBuffer();
    if fSuccess == 0 {
        return (FALSE) as BOOL;
    }

    CloseHandle(pi.hThread);

    if Wait != 0 {
        EnableWindow(hMainWnd, (FALSE) as BOOL);

        WaitForProcess(pi.hProcess);

        EnableWindow(hMainWnd, (TRUE) as BOOL);
        SetForegroundWindow(hMainWnd);
        // We got the real activation message during MsgWaitForMultipleObjects while
        // we were disabled, we need to set the focus again now.
        SetFocus(hMainWnd);
    }
    CloseHandle(pi.hProcess);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetStorageDirectory(Directory: *mut CStringW) -> BOOL {
    static mut CachedDirectory: CStringW = CStringW::new();
    static mut CachedDirectoryInitialized: BOOL = (FALSE as i32);
    let cd: *mut CStringW = core::ptr::addr_of_mut!(CachedDirectory);

    if CachedDirectoryInitialized == 0 {
        let DirectoryStr: LPWSTR = (*cd).GetBuffer(MAX_PATH as i32);
        let bHasPath: BOOL = SHGetSpecialFolderPathW(core::ptr::null_mut(), DirectoryStr, (CSIDL_LOCAL_APPDATA as i32), (TRUE) as BOOL);
        if bHasPath != 0 {
            PathAppendW(DirectoryStr, RAPPS_NAME.as_ptr());
        }
        (*cd).ReleaseBuffer();

        if bHasPath != 0 {
            if CreateDirectoryW((*cd).GetString(), core::ptr::null_mut()) == 0 && GetLastError() != ERROR_ALREADY_EXISTS {
                (*cd).Empty();
            }
        } else {
            (*cd).Empty();
        }

        CachedDirectoryInitialized = (TRUE as i32);
    }

    (*Directory).SetString((*cd).GetString(), (*cd).GetLength());
    (!(*Directory).IsEmpty()) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn InitLogs() {
    if ((SettingsInfo.bLogEnabled) == 0) {
        return;
    }

    let mut szPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwCategoryNum: DWORD = 1;
    let mut dwDisp: DWORD = 0;
    let mut dwData: DWORD = 0;
    let mut key: CRegKey = CRegKey__make_0();

    if key.Create(
            HKEY_LOCAL_MACHINE,
            CPP_COM_WSTR_D2BA60B1608F.as_ptr(), core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE, KEY_WRITE, core::ptr::null_mut(), &mut dwDisp) != (ERROR_SUCCESS as i32)
    {
        return;
    }

    if GetModuleFileNameW(core::ptr::null_mut(), szPath.as_mut_ptr(), _countof!(szPath)) == 0
    {
        return;
    }

    dwData = EVENTLOG_ERROR_TYPE | EVENTLOG_WARNING_TYPE | EVENTLOG_INFORMATION_TYPE;

    if (key.SetStringValue(CPP_COM_WSTR_BB2F7BE27363.as_ptr(), szPath.as_ptr(), REG_EXPAND_SZ) == (ERROR_SUCCESS as i32)) &&
        (key.SetStringValue(CPP_COM_WSTR_E2128F4CC323.as_ptr(), szPath.as_ptr(), REG_EXPAND_SZ) == (ERROR_SUCCESS as i32)) &&
        (key.SetDWORDValue(CPP_COM_WSTR_D587A5BCAA56.as_ptr(), dwData) == (ERROR_SUCCESS as i32)) &&
        (key.SetDWORDValue(CPP_COM_WSTR_28C29EE251FE.as_ptr(), dwCategoryNum) == (ERROR_SUCCESS as i32))

    {
        hLog = RegisterEventSourceW(core::ptr::null_mut(), CPP_COM_WSTR_A50DE1AFC4FA.as_ptr());
    }
}
#[no_mangle]
pub unsafe extern "C" fn FreeLogs() {
    if !hLog.is_null() {
        DeregisterEventSource(hLog);
    }
}
#[no_mangle]
pub unsafe extern "C" fn WriteLogMessage(wType: WORD, dwEventID: DWORD, mut lpMsg: LPCWSTR) -> BOOL {
    if !(SettingsInfo.bLogEnabled != 0) {
        return (TRUE) as BOOL;
    }

    if !(ReportEventW(hLog, wType, 0, dwEventID, core::ptr::null_mut(), 1, 0, core::ptr::addr_of_mut!(lpMsg), core::ptr::null_mut()) != 0) {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetInstalledVersion_WowUser(szVersionResult: *mut CStringW, szRegName: &CStringW, IsUserKey: BOOL, keyWow: REGSAM) -> BOOL {
    let mut bHasSucceded: BOOL = (FALSE as i32);
    let mut key = CRegKey__make_0();
    let mut szVersion: CStringW = CStringW::new();
    let mut szPath: CStringW = CStringW::from_ptr(CPP_COM_WSTR_0D896F4ECFCA.as_ptr());
    szPath += szRegName.GetString();

    if key.Open(if IsUserKey != 0 { HKEY_CURRENT_USER } else { HKEY_LOCAL_MACHINE }, szPath.GetString(), keyWow | KEY_READ) !=
        (ERROR_SUCCESS as i32)
    {
        return (FALSE) as BOOL;
    }

    if !szVersionResult.is_null() {
        let mut dwSize: ULONG = (MAX_PATH as ULONG) * (core::mem::size_of::<WCHAR>() as ULONG);

        if key.QueryStringValue(CPP_COM_WSTR_A53534ACF67B.as_ptr(), szVersion.GetBuffer((MAX_PATH as i32)), &mut dwSize as *mut ULONG) == (ERROR_SUCCESS as i32)
        {
            szVersion.ReleaseBuffer();
            *szVersionResult = szVersion;
            bHasSucceded = (TRUE as i32);
        }
        else
        {
            szVersion.ReleaseBuffer();
        }
    }
    else
    {
        bHasSucceded = (TRUE as i32);
    }

    return bHasSucceded;
}
#[no_mangle]
pub unsafe extern "C" fn GetInstalledVersion(pszVersion: *mut CStringW, szRegName: &CStringW) -> BOOL {
    return (
        (!szRegName.IsEmpty() && (GetInstalledVersion_WowUser(pszVersion, szRegName, (TRUE) as BOOL, KEY_WOW64_32KEY) != 0
            || GetInstalledVersion_WowUser(pszVersion, szRegName, (FALSE) as BOOL, KEY_WOW64_32KEY) != 0
            || GetInstalledVersion_WowUser(pszVersion, szRegName, (TRUE) as BOOL, KEY_WOW64_64KEY) != 0
            || GetInstalledVersion_WowUser(pszVersion, szRegName, (FALSE) as BOOL, KEY_WOW64_64KEY) != 0)) as BOOL
    );
}
#[no_mangle]
pub unsafe extern "C" fn IsSystem64Bit() -> BOOL {
    static mut cache: UINT = 0;
    if cache == 0 {
        cache = (1 as UINT).wrapping_add(
            (IsOS(OS_WOW6432) != (FALSE as i32)) as UINT,
        );
    }
    return cache.wrapping_sub(1 as UINT) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetNTVersion() -> ULONG {
    let mut osvi: RTL_OSVERSIONINFOW = core::mem::zeroed();
    osvi.dwOSVersionInfoSize = core::mem::size_of::<RTL_OSVERSIONINFOW>() as ULONG;
    RtlGetVersion(core::ptr::addr_of_mut!(osvi));

    return (osvi.dwMajorVersion << 8) | (osvi.dwMinorVersion);
}
#[no_mangle]
pub unsafe extern "C" fn GetSystemColorDepth() -> i32 {
    let mut pDevMode: DEVMODEW = core::mem::zeroed();
    let ColorDepth: i32;

    pDevMode.dmSize = core::mem::size_of::<DEVMODEW>() as u16;
    pDevMode.dmDriverExtra = 0;

    if EnumDisplaySettingsW(core::ptr::null_mut(), ENUM_CURRENT_SETTINGS, core::ptr::addr_of_mut!(pDevMode)) == 0 {
        /* TODO: Error message */
        return (ILC_COLOR as i32);
    }

    match pDevMode.dmBitsPerPel {
        32 => {
            ColorDepth = (ILC_COLOR32 as i32);
        }
        24 => {
            ColorDepth = (ILC_COLOR24 as i32);
        }
        16 => {
            ColorDepth = (ILC_COLOR16 as i32);
        }
        8 => {
            ColorDepth = (ILC_COLOR8 as i32);
        }
        4 => {
            ColorDepth = (ILC_COLOR4 as i32);
        }
        _ => {
            ColorDepth = (ILC_COLOR as i32);
        }
    }

    return ColorDepth;
}
#[no_mangle]
pub unsafe extern "C" fn UnixTimeToFileTime(dwUnixTime: DWORD, pFileTime: LPFILETIME) {
    // Note that LONGLONG is a 64-bit value
    let mut ll: i64;

    ll = (dwUnixTime as i32 as i64) * (10000000i32 as i64) + 116444736000000000;
    (*pFileTime).dwLowDateTime = (ll as DWORD);
    (*pFileTime).dwHighDateTime = (ll >> 32) as DWORD;
}

#[no_mangle]
pub unsafe extern "C" fn IsSameRegKey__0(hRoot: HKEY, Path1: LPCWSTR, Sam1: REGSAM, Path2: LPCWSTR, Sam2: REGSAM) -> BOOL {
    let WowMask: REGSAM = KEY_WOW64_32KEY | KEY_WOW64_64KEY;
    let mut key1 = CRegKey__make_0();
    let mut key2 = CRegKey__make_0();
    return (key1.Open(hRoot, Path1, MAXIMUM_ALLOWED | (Sam1 & WowMask)) == (ERROR_SUCCESS as i32)
        && key2.Open(hRoot, Path2, MAXIMUM_ALLOWED | (Sam2 & WowMask)) == (ERROR_SUCCESS as i32)
        && ((IsSameRegKey__1(key1.op_to_HKEY(), key2.op_to_HKEY())) != 0)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetRegString(Key: &mut CRegKey, Name: LPCWSTR, Value: &mut CStringW) -> LPCWSTR {
    loop {
        let mut cb: ULONG = 0;
        let cch: ULONG;
        let mut err: ULONG = (Key.QueryValue(Name, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(cb))) as u32;
        if err != 0 {
            break;
        }
        cch = cb / core::mem::size_of::<WCHAR>() as ULONG;
        let p: LPWSTR = Value.GetBuffer(cch as i32 + 1);
        *p.add(cch as usize) = UNICODE_NULL;
        err = (Key.QueryValue(Name, core::ptr::null_mut(), p as *mut core::ffi::c_void, core::ptr::addr_of_mut!(cb))) as u32;
        if err == ERROR_MORE_DATA {
            continue;
        }
        if err != 0 {
            break;
        }
        Value.ReleaseBuffer();
        return Value.GetString();
    }
    core::ptr::null()
}
#[no_mangle]
pub unsafe extern "C" fn ExpandEnvStrings(Str: *mut CStringW) -> bool {
    let mut buf = CStringW::new();
    let cch: DWORD = ExpandEnvironmentStringsW((*Str).GetString(), core::ptr::null_mut(), 0);
    if cch != 0 {
        if ExpandEnvironmentStringsW((*Str).GetString(), buf.GetBuffer(cch as i32), cch) == cch {
            buf.ReleaseBufferSetLength((cch - 1 as u32) as i32);
            *Str = buf;
            return true;
        }
    }
    return false;
}
#[no_mangle]
pub unsafe extern "C" fn SearchPatternMatch(szHaystack: LPCWSTR, szNeedle: LPCWSTR) -> BOOL {
    if *szNeedle == 0 {
        return (TRUE) as BOOL;
    }
    /* TODO: Improve pattern search beyond a simple case-insensitive substring search. */
    return (StrStrIW(szHaystack, szNeedle) != core::ptr::null_mut()) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteDirectoryTree(Dir: LPCWSTR, hwnd: HWND) -> BOOL {
    let mut from = CStringW::from_ptr(Dir);
    let cch: UINT = from.GetLength() as UINT;
    from.Append(CPP_COM_WSTR_B422CF60B57F.as_ptr());
    let p: LPWSTR = from.GetBuffer(0);
    *p.add(cch as usize) = 0x0000u16; // Double null-terminate
    *p.add((cch + 1) as usize) = 0x0000u16;
    let fof: UINT = FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT;
    let mut shfos: SHFILEOPSTRUCTW = SHFILEOPSTRUCTW {
        hwnd: hwnd,
        wFunc: FO_DELETE as UINT,
        pFrom: p,
        pTo: core::ptr::null_mut(),
        fFlags: fof as FILEOP_FLAGS,
        fAnyOperationsAborted: 0,
        hNameMappings: core::ptr::null_mut(),
        lpszProgressTitle: core::ptr::null(),
    };
    return SHFileOperationW(core::ptr::addr_of_mut!(shfos));
}
#[no_mangle]
pub unsafe extern "C" fn CreateDirectoryTree(Dir: LPCWSTR) -> UINT {
    let err: UINT = SHCreateDirectory(core::ptr::null_mut(), Dir) as UINT;
    return if err == ERROR_ALREADY_EXISTS { 0 } else { err };
}
#[no_mangle]
pub unsafe extern "C" fn SplitFileAndDirectory(FullPath: LPCWSTR, pDir: *mut CStringW) -> CStringW {
    let mut dir = CPathW::from_ptr(FullPath);
    //int win = dir.ReverseFind(L'\\'), nix = dir.ReverseFind(L'/'), sep = max(win, nix);
    let sep = dir.FindFileName();
    let mut file = dir.m_strPath.Mid(sep);
    if !pDir.is_null() {
        *pDir = if sep == -1 {
            CStringW::from_ptr(CPP_COM_WSTR_96A296D224F2.as_ptr())
        } else {
            dir.m_strPath.Left(sep - 1)
        };
    }
    return file;
}
#[no_mangle]
pub unsafe extern "C" fn GetSpecialPath(csidl: UINT, Path: &mut CStringW, hwnd: HWND) -> HRESULT {
    if SHGetSpecialFolderPathW(hwnd, Path.GetBuffer((MAX_PATH as i32)), csidl as i32, (TRUE) as BOOL) == 0 {
        return E_FAIL;
    }
    Path.ReleaseBuffer();
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn GetKnownPath(kfid: *const KNOWNFOLDERID, Path: &mut CStringW, Flags: DWORD) -> HRESULT {
    let mut p: PWSTR = core::ptr::null_mut();
    let f: FARPROC = GetProcAddress(LoadLibraryW(CPP_COM_WSTR_97829A80D968.as_ptr()), b"SHGetKnownFolderPath\0".as_ptr() as *const i8);
    if f.is_none() {
        return HRESULT_FROM_WIN32!(ERROR_OLD_WIN_VERSION);
    }
    let hr: HRESULT = (core::mem::transmute::<FARPROC, unsafe extern "system" fn(*const KNOWNFOLDERID, UINT, HANDLE, *mut PWSTR) -> HRESULT>(f))(kfid, Flags, core::ptr::null_mut(), &mut p as *mut PWSTR as *mut PWSTR);
    if hr < 0 {
        return hr;
    }
    *Path = CStringW::from_ptr(p);
    CoTaskMemFree(p as _);
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn GetProgramFilesPath(Path: &mut CStringW, PerUser: BOOL, hwnd: HWND) -> HRESULT {
    if PerUser == 0 {
        return GetSpecialPath(CSIDL_PROGRAM_FILES, Path, hwnd);
    }

    let mut hr: HRESULT = GetKnownPath(&FOLDERID_UserProgramFiles, Path, KF_FLAG_CREATE);
    if hr < 0 {
        hr = GetSpecialPath(CSIDL_LOCAL_APPDATA, Path, hwnd);
        // Use the correct path on NT6 (on NT5 the path becomes a bit long)
        if hr >= 0 && LOBYTE!(GetVersion()) >= 6 {
            *Path = BuildPath__T0(Path, CPP_COM_WSTR_ECAE79F7CE7F.as_ptr()); // Should not be localized
        }
    }
    return hr;
}

unsafe extern "C" fn ReadAt(Handle: HANDLE, Offset: UINT, Buffer: LPVOID, Size: UINT) -> BOOL {
    let mut ol: OVERLAPPED = core::mem::zeroed();
    ol.__bindgen_anon_1.__bindgen_anon_1.Offset = Offset;
    let mut cb: DWORD = 0;
    return (ReadFile(Handle, Buffer, Size, core::ptr::addr_of_mut!(cb), core::ptr::addr_of_mut!(ol)) != 0 && cb == Size) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetSilentInstallParameters(InstallerType: InstallerType, ExtraInfo: UINT, Installer: LPCWSTR, Parameters: &mut CStringW) -> BOOL {
    match InstallerType {
        INSTALLER_MSI => {
            atl_format!(*Parameters, CPP_COM_WSTR_B951C1B4FB5A.as_ptr(), Installer);
            return (TRUE) as BOOL;
        }

        INSTALLER_INNO => {
            let pszInnoParams: LPCWSTR = CPP_COM_WSTR_CEA6748FA84E.as_ptr();
            if ExtraInfo != 0 {
                atl_format!(*Parameters, CPP_COM_WSTR_FB47CCC40FE6.as_ptr(), pszInnoParams);
            } else {
                atl_format!(*Parameters, CPP_COM_WSTR_1A217ADA0F04.as_ptr(), pszInnoParams); // https://jrsoftware.org/files/is5.5-whatsnew.htm
            }
            return (TRUE) as BOOL;
        }

        INSTALLER_NSIS => {
            *Parameters = CStringW::from_ptr(CPP_COM_WSTR_B42E8543313E.as_ptr());
            return (TRUE) as BOOL;
        }

        _ => {
            return (FALSE) as BOOL;
        }
    }
}
/// misc.cpp `IsSameRegKey(HKEY, HKEY)`, hand-written: its function-local `struct NameInfo :
/// UNICODE_STRING` (a buffer the string points into, set up by its constructor) has no model
/// layout. Built in place and never moved, as the C++ local is.
unsafe extern "C" fn IsSameRegKey__1(hKey1: HKEY, hKey2: HKEY) -> BOOL {
    // CompareObjectHandles is Win10+ so we check the path instead.
    #[repr(C)]
    struct NameInfo {
        base: UNICODE_STRING,
        Allocation: [WCHAR; MAX_PATH as usize],
    }
    let mut Name1: NameInfo = core::mem::zeroed();
    let mut Name2: NameInfo = core::mem::zeroed();
    for n in [&mut Name1, &mut Name2] {
        n.base.MaximumLength = core::mem::size_of_val(&n.Allocation) as USHORT;
        n.base.Buffer = n.Allocation.as_mut_ptr();
    }
    let mut Length: ULONG = 0;
    (NtQueryObject(
        hKey1 as HANDLE,
        ObjectNameInformation,
        core::ptr::addr_of_mut!(Name1) as PVOID,
        core::mem::size_of::<NameInfo>() as ULONG,
        &mut Length,
    ) >= 0
        && NtQueryObject(
            hKey2 as HANDLE,
            ObjectNameInformation,
            core::ptr::addr_of_mut!(Name2) as PVOID,
            core::mem::size_of::<NameInfo>() as ULONG,
            &mut Length,
        ) >= 0
        && RtlCompareUnicodeString(&Name1.base, &Name2.base, TRUE as BOOLEAN) == 0) as BOOL
}

/// misc.cpp `RegKeyHasValues`, hand-written (the model's body did not parse).
pub unsafe extern "C" fn RegKeyHasValues(hKey: HKEY, Path: LPCWSTR, wowsam: REGSAM) -> HRESULT {
    let mut key = CRegKey__make_0();
    let mut err: LONG = key.Open(hKey, Path, KEY_QUERY_VALUE | wowsam);
    if err == ERROR_SUCCESS as LONG {
        let mut name: [WCHAR; 1] = [0; 1];
        let mut cchname: DWORD = name.len() as DWORD;
        let mut cbsize: DWORD = 0;
        err = RegEnumValueW(
            key.op_to_HKEY(),
            0,
            name.as_mut_ptr(),
            &mut cchname,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut cbsize,
        ) as LONG;
        if err == ERROR_NO_MORE_ITEMS as LONG {
            return S_FALSE as HRESULT;
        }
        if err == ERROR_MORE_DATA as LONG {
            err = ERROR_SUCCESS as LONG;
        }
    }
    HRESULT_FROM_WIN32!(err)
}

/// misc.cpp `GuessInstallerType`, hand-written: the model's body did not parse twice. Its
/// `goto done` exits are `break 'done`, and `done:`'s CloseHandle runs on every path.
pub unsafe extern "C" fn GuessInstallerType(Installer: LPCWSTR, ExtraInfo: &mut UINT) -> InstallerType {
    // kernel32 resource/library entry points the crate prelude does not declare.
    extern "system" {
        fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
        fn FindResourceW(hModule: HMODULE, lpName: LPCWSTR, lpType: LPCWSTR) -> *mut c_void;
        fn LoadResource(hModule: HMODULE, hResInfo: *mut c_void) -> HGLOBAL;
        fn LockResource(hResData: HGLOBAL) -> LPVOID;
    }
    *ExtraInfo = 0;
    let mut it: InstallerType = INSTALLER_UNKNOWN;

    let hFile = CreateFileW(
        Installer,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_DELETE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return it;
    }

    'done: {
        let mut buf: [BYTE; 0x30 + 12] = [0; 0x30 + 12];
        if ReadAt(hFile, 0, buf.as_mut_ptr() as LPVOID, buf.len() as UINT) == 0 {
            break 'done;
        }

        if buf[0] == 0xD0 && buf[1] == 0xCF && buf[2] == 0x11 && buf[3] == 0xE0 {
            if StrCmpIW(CPP_COM_WSTR_1AA03B0B214A.as_ptr(), PathFindExtensionW(Installer)) == 0 {
                it = INSTALLER_MSI;
                break 'done;
            }
        }

        if buf[0] != b'M' || buf[1] != b'Z' {
            break 'done;
        }

        // <= Inno 5.1.4
        {
            let mut sig: [UINT; 3] = [0; 3];
            const _: () = assert!(0x30 + 12 >= 0x30 + core::mem::size_of::<[UINT; 3]>());
            core::ptr::copy_nonoverlapping(
                buf.as_ptr().add(0x30),
                sig.as_mut_ptr() as *mut BYTE,
                core::mem::size_of_val(&sig),
            );
            if sig[0] == 0x6F6E6E49 && sig[1] == !sig[2] {
                it = INSTALLER_INNO;
                *ExtraInfo = 1;
                break 'done;
            }
        }

        // NSIS 1.6+
        {
            let mut nsis: [UINT; 1 + 1 + 3 + 1 + 1] = [0; 1 + 1 + 3 + 1 + 1];
            let nsisskip: UINT = 512;
            let mut nsisoffset: UINT = nsisskip * 20;
            while nsisoffset < 1024 * 1024 * 50 {
                if ReadAt(hFile, nsisoffset, nsis.as_mut_ptr() as LPVOID, core::mem::size_of_val(&nsis) as UINT) != 0
                    && nsis[1] == 0xDEADBEEF
                {
                    if nsis[2] == 0x6C6C754E && nsis[3] == 0x74666F73 && nsis[4] == 0x74736E49 {
                        it = INSTALLER_NSIS;
                        break 'done;
                    }
                }
                nsisoffset += nsisskip;
            }
        }

        let hMod = LoadLibraryExW(Installer, core::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE);
        if !hMod.is_null() {
            // Inno
            const INNORCSETUPLDR: usize = 11111;
            const RT_RCDATA: usize = 10; // winuser.h MAKEINTRESOURCE(10)
            let hRsrc = FindResourceW(hMod, INNORCSETUPLDR as LPCWSTR, RT_RCDATA as LPCWSTR);
            if !hRsrc.is_null() {
                let hGlob = LoadResource(hMod, hRsrc);
                if !hGlob.is_null() {
                    let p = LockResource(hGlob) as *const BYTE;
                    if !p.is_null() {
                        if *p == b'r' && *p.add(1) == b'D' && *p.add(2) == b'l' && *p.add(3) == b'P'
                            && *p.add(4) == b't' && *p.add(5) == b'S'
                        {
                            it = INSTALLER_INNO;
                        }
                        // UnlockResource(p): winbase.h's macro, ((void)(p), 0) -- nothing to undo.
                    }
                }
            }
            FreeLibrary(hMod);
        }
    }
    CloseHandle(hFile);
    it
}
