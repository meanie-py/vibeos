// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_SCREENSAVERS: usize = 100;

// 13, not 12 -- same dropped NUL as monslctl's szMonitorSelWndClass. The C sizes
// `TEXT("SSDemoParent")` at 13 (screensaver.c:15); this array held only the 12 characters,
// leaving RegisterClass, CreateWindowEx and UnregisterClass to run off the end.
pub static szPreviewWndClass: [TCHAR; 13] = [
    b'S' as TCHAR, b'S' as TCHAR, b'D' as TCHAR, b'e' as TCHAR,
    b'm' as TCHAR, b'o' as TCHAR, b'P' as TCHAR, b'a' as TCHAR,
    b'r' as TCHAR, b'e' as TCHAR, b'n' as TCHAR, b't' as TCHAR, 0,
];

#[repr(C)]
pub struct SCREEN_SAVER_ITEM {
    pub bIsScreenSaver: BOOL,
    pub szFilename: [TCHAR; (MAX_PATH) as usize],
    pub szDisplayName: [TCHAR; 256],
}

pub type PSCREEN_SAVER_ITEM = *mut SCREEN_SAVER_ITEM;

#[repr(C)]
pub struct _DATA {
    pub ScreenSaverItems: [SCREEN_SAVER_ITEM; MAX_SCREENSAVERS],
    pub PrevWindowPi: PROCESS_INFORMATION,
    pub Selection: i32,
    pub OldPreviewProc: WNDPROC,
    pub ScreenSaverCount: UINT,
    pub ScreenSaverPreviewParent: HWND,
}

pub type DATA = _DATA;
pub type PDATA = *mut _DATA;
unsafe extern "C" fn GetCurrentScreenSaverValue(lpValue: LPTSTR) -> LPTSTR {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lpBuf: LPTSTR = core::ptr::null_mut();
    let mut BufSize: DWORD = 0;
    let mut Type: DWORD = REG_SZ;
    let mut Ret: LONG;

    Ret = RegOpenKeyEx(HKEY_CURRENT_USER,
                       w!("Control Panel\\Desktop"),
                       0,
                       KEY_READ,
                       &mut hKey);
    if Ret != (ERROR_SUCCESS) as i32 {
        return core::ptr::null_mut();
    }

    Ret = RegQueryValueEx(hKey,
                          lpValue,
                          (0) as *mut u32,
                          &mut Type,
                          core::ptr::null_mut(),
                          &mut BufSize);
    if Ret == (ERROR_SUCCESS) as i32 {
        lpBuf = HeapAlloc(GetProcessHeap(), 0, (BufSize) as SIZE_T) as LPTSTR;
        if !lpBuf.is_null() {
            Ret = RegQueryValueEx(hKey,
                                  lpValue,
                                  (0) as *mut u32,
                                  &mut Type,
                                  lpBuf as LPBYTE,
                                  &mut BufSize);
            if Ret != (ERROR_SUCCESS) as i32 {
                HeapFree(GetProcessHeap(), 0, lpBuf as *mut core::ffi::c_void);
                lpBuf = core::ptr::null_mut();
            }
        }
    }

    RegCloseKey(hKey);

    return lpBuf;
}
unsafe extern "C" fn SelectionChanged(hwndDlg: HWND, pData: PDATA) {
    let hwndCombo: HWND;
    let bEnable: BOOL;
    let mut i: INT;

    hwndCombo = GetDlgItem(hwndDlg, (IDC_SCREENS_LIST) as i32);

    i = SendMessage(hwndCombo, CB_GETCURSEL, 0, 0) as INT;
    i = SendMessage(hwndCombo, CB_GETITEMDATA, ((i) as u32) as WPARAM, 0) as INT;

    (*pData).Selection = i;

    bEnable = (i != 0) as BOOL;

    EnableWindow(GetDlgItem(hwndDlg, (IDC_SCREENS_SETTINGS) as i32), bEnable);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_SCREENS_TESTSC) as i32), bEnable);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_SCREENS_USEPASSCHK) as i32), bEnable);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_SCREENS_TIMEDELAY) as i32), bEnable);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_SCREENS_TIME) as i32), bEnable);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_WAITTEXT) as i32), bEnable);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_MINTEXT) as i32), bEnable);
}
#[no_mangle]
pub unsafe extern "system" fn RedrawSubclassProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut hwnd: HWND;
    let mut pData: PDATA;
    let mut Ret: LRESULT = (FALSE) as LPARAM;

    pData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDATA;
    if pData.is_null() {
        return Ret;
    }

    Ret = CallWindowProc((*pData).OldPreviewProc, hwndDlg, uMsg, wParam, lParam);

    if uMsg == WM_PAINT {
        hwnd = (*pData).ScreenSaverPreviewParent;
        if !hwnd.is_null() {
            RedrawWindow(
                hwnd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    return Ret;
}
unsafe extern "C" fn ShowScreenSaverPreview(draw: LPDRAWITEMSTRUCT, pData: PDATA) {
    let mut hBrush: HBRUSH;
    let hDC: HDC;
    let hOldObj: HGDIOBJ;
    let mut rcItem: RECT = RECT {
        left: (MONITOR_LEFT) as i32,
        top: (MONITOR_TOP) as i32,
        right: (MONITOR_RIGHT) as i32,
        bottom: (MONITOR_BOTTOM) as i32,
    };

    hDC = CreateCompatibleDC((*draw).hDC);
    hOldObj = SelectObject(hDC, (g_GlobalData.hMonitorBitmap).cast());

    if IsWindowVisible((*pData).ScreenSaverPreviewParent) == 0 {
        hBrush = CreateSolidBrush(g_GlobalData.desktop_color);
        FillRect(hDC, &mut rcItem, hBrush);
        DeleteObject((hBrush).cast());
    }

    GdiTransparentBlt(
        (*draw).hDC,
        (*draw).rcItem.left,
        (*draw).rcItem.top,
        (*draw).rcItem.right - (*draw).rcItem.left + 1,
        (*draw).rcItem.bottom - (*draw).rcItem.top + 1,
        hDC,
        0,
        0,
        g_GlobalData.bmMonWidth,
        g_GlobalData.bmMonHeight,
        MONITOR_ALPHA,
    );

    SelectObject(hDC, hOldObj);
    DeleteDC(hDC);
}
unsafe extern "C" fn SetScreenSaverPreviewBox(hwndDlg: HWND, pData: PDATA) {
    let hPreview = (*pData).ScreenSaverPreviewParent;
    let hr: HRESULT;
    let mut si: STARTUPINFO = core::mem::zeroed();
    let mut szCmdline: [TCHAR; 2048] = [0; 2048];

    if !(*pData).PrevWindowPi.hProcess.is_null() {
        TerminateProcess((*pData).PrevWindowPi.hProcess, 0);
        CloseHandle((*pData).PrevWindowPi.hProcess);
        CloseHandle((*pData).PrevWindowPi.hThread);
        (*pData).PrevWindowPi.hThread = core::ptr::null_mut();
        (*pData).PrevWindowPi.hProcess = core::ptr::null_mut();
    }
    ShowWindow((*pData).ScreenSaverPreviewParent, (SW_HIDE) as i32);

    if (*pData).Selection < 1 {
        return;
    }

    hr = StringCbPrintf!(
        szCmdline.as_mut_ptr(),
        core::mem::size_of_val(&szCmdline),
        w!("%s /p %Iu"),
        (*pData).ScreenSaverItems[(*pData).Selection as usize].szFilename.as_ptr(),
        hPreview as ULONG_PTR,
    );
    if hr < 0 {
        return;
    }

    ZeroMemory!(
        &mut si as *mut _ as *mut _,
        core::mem::size_of::<STARTUPINFO>(),
    );
    si.cb = core::mem::size_of::<STARTUPINFO>() as u32;
    ZeroMemory!(
        &mut (*pData).PrevWindowPi as *mut _ as *mut _,
        core::mem::size_of::<PROCESS_INFORMATION>(),
    );

    ShowWindow((*pData).ScreenSaverPreviewParent, (SW_SHOW) as i32);

    if CreateProcess(
        core::ptr::null_mut(),
        szCmdline.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as i32,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut si,
        &mut (*pData).PrevWindowPi,
    ) == 0
    {
        (*pData).PrevWindowPi.hThread = core::ptr::null_mut();
        (*pData).PrevWindowPi.hProcess = core::ptr::null_mut();
    }
}
unsafe extern "C" fn WaitForSettingsDialog(hwndDlg: HWND, hProcess: HANDLE) {
    let mut dwResult: DWORD;
    let mut msg: MSG = core::mem::zeroed();

    loop {
        dwResult = MsgWaitForMultipleObjects(
            1,
            &hProcess,
            (FALSE) as BOOL,
            INFINITE,
            QS_ALLINPUT,
        );
        if dwResult == WAIT_OBJECT_0 + 1 {
            if PeekMessage(
                &mut msg,
                core::ptr::null_mut(),
                0,
                0,
                PM_REMOVE,
            ) != 0
            {
                if msg.message == WM_QUIT {
                    return;
                }
                if IsDialogMessage(hwndDlg, &mut msg) == 0 {
                    TranslateMessage(&mut msg);
                    DispatchMessage(&mut msg);
                }
            }
        } else {
            return;
        }
    }
}
unsafe extern "C" fn ScreenSaverConfig(hwndDlg: HWND, pData: PDATA) {
    let mut hr: HRESULT;
    let mut si: STARTUPINFO = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut szCmdline: [TCHAR; 2048] = core::mem::zeroed();

    if (*pData).Selection < 1 {
        return;
    }

    hr = StringCbPrintf!(
        szCmdline.as_mut_ptr(),
        core::mem::size_of_val(&szCmdline),
        TEXT(w!("%s /c:%Iu") as LPCTSTR),
        (*pData).ScreenSaverItems[(*pData).Selection as usize].szFilename.as_ptr(),
        hwndDlg as usize,
    );
    if hr < 0 {
        return;
    }

    ZeroMemory!(
        &mut si as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<STARTUPINFO>(),
    );
    si.cb = core::mem::size_of::<STARTUPINFO>() as u32;
    ZeroMemory!(
        &mut pi as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<PROCESS_INFORMATION>(),
    );
    if CreateProcess(
        core::ptr::null_mut(),
        szCmdline.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as i32,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut si,
        &mut pi,
    ) != 0
    {
        if (*pData).PrevWindowPi.hProcess != core::ptr::null_mut() {
            TerminateProcess((*pData).PrevWindowPi.hProcess, 0);
            CloseHandle((*pData).PrevWindowPi.hProcess);
            CloseHandle((*pData).PrevWindowPi.hThread);
            (*pData).PrevWindowPi.hThread = core::ptr::null_mut();
            (*pData).PrevWindowPi.hProcess = core::ptr::null_mut();
        }

        WaitForSettingsDialog(hwndDlg, pi.hProcess);
        SetScreenSaverPreviewBox(hwndDlg, pData);

        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}
unsafe extern "C" fn ScreenSaverPreview(hwndDlg: HWND, pData: *mut DATA) {
    let mut hr: HRESULT;
    let mut si: STARTUPINFO = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut szCmdline: [TCHAR; 2048] = core::mem::zeroed();

    if (*pData).Selection < 1 {
        return;
    }

    if !(*pData).PrevWindowPi.hProcess.is_null() {
        TerminateProcess((*pData).PrevWindowPi.hProcess, 0);
        CloseHandle((*pData).PrevWindowPi.hProcess);
        CloseHandle((*pData).PrevWindowPi.hThread);
        (*pData).PrevWindowPi.hThread = core::ptr::null_mut();
        (*pData).PrevWindowPi.hProcess = core::ptr::null_mut();
    }

    hr = StringCbPrintf!(
        szCmdline.as_mut_ptr(),
        core::mem::size_of_val(&szCmdline),
        TEXT(w!("%s /s") as LPCTSTR),
        (*pData).ScreenSaverItems[(*pData).Selection as usize].szFilename.as_ptr()
    );
    if hr < 0 {
        return;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void,
        core::mem::size_of::<STARTUPINFO>()
    );
    si.cb = core::mem::size_of::<STARTUPINFO>() as u32;
    ZeroMemory!(
        core::ptr::addr_of_mut!(pi) as *mut core::ffi::c_void,
        core::mem::size_of::<PROCESS_INFORMATION>()
    );
    if CreateProcess(
        core::ptr::null_mut(),
        szCmdline.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as i32,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (core::ptr::addr_of_mut!(si)).cast(),
        core::ptr::addr_of_mut!(pi)
    ) != 0 {
        WaitForSingleObject(pi.hProcess, INFINITE);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}
unsafe extern "C" fn CheckRegScreenSaverIsSecure(hwndDlg: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szBuffer: [TCHAR; 2] = [0; 2];
    let mut bufferSize: u32 = core::mem::size_of_val(&szBuffer) as u32;
    let mut varType: u32 = REG_SZ;
    let result: i32;

    if RegOpenKeyEx(HKEY_CURRENT_USER,
                     w!("Control Panel\\Desktop"),
                     0,
                     KEY_ALL_ACCESS,
                     &mut hKey) == (ERROR_SUCCESS) as i32 {
        result = RegQueryValueEx(hKey,
                                w!("ScreenSaverIsSecure"),
                                (0) as *mut u32,
                                &mut varType,
                                szBuffer.as_mut_ptr() as LPBYTE,
                                &mut bufferSize);
        RegCloseKey(hKey);

        if result == (ERROR_SUCCESS) as i32 {
            if _ttoi(szBuffer.as_ptr()) == 1 {
                SendDlgItemMessage(hwndDlg,
                                   (IDC_SCREENS_USEPASSCHK) as i32,
                                   BM_SETCHECK,
                                   BST_CHECKED as WPARAM,
                                   0);
                return;
            }
        }

        SendDlgItemMessage(hwndDlg,
                           (IDC_SCREENS_USEPASSCHK) as i32,
                           BM_SETCHECK,
                           BST_UNCHECKED as WPARAM,
                           0);
    }
}
unsafe extern "C" fn AddScreenSaverItem(
    hwndScreenSavers: HWND,
    pData: *mut DATA,
    ScreenSaverItem: *mut SCREEN_SAVER_ITEM,
) -> BOOL {
    let i: UINT;

    if (*pData).ScreenSaverCount >= (MAX_SCREENSAVERS) as u32 {
        return (FALSE) as i32;
    }

    i = SendMessage(
        hwndScreenSavers,
        CB_ADDSTRING,
        0,
        (*ScreenSaverItem).szDisplayName.as_ptr() as LPARAM,
    ) as UINT;
    if i == (CB_ERR) as u32 || i == (CB_ERRSPACE) as u32 {
        return (FALSE) as i32;
    }

    SendMessage(
        hwndScreenSavers,
        CB_SETITEMDATA,
        i as WPARAM,
        (*pData).ScreenSaverCount as LPARAM,
    );

    (*pData).ScreenSaverCount += 1;
    (TRUE) as i32
}
unsafe extern "C" fn AddScreenSaver(
    hwndScreenSavers: HWND,
    pData: *mut DATA,
    pszFilePath: LPCTSTR,
    pszFileName: LPCTSTR,
) -> BOOL {
    let mut ScreenSaverItem: *mut SCREEN_SAVER_ITEM;
    let mut hModule: HANDLE;
    let mut hr: HRESULT;

    if (*pData).ScreenSaverCount >= (MAX_SCREENSAVERS) as u32 {
        return (FALSE) as i32;
    }

    ScreenSaverItem = (*pData).ScreenSaverItems.as_mut_ptr().add((*pData).ScreenSaverCount as usize);

    (*ScreenSaverItem).bIsScreenSaver = (TRUE) as i32;

    hModule = (LoadLibraryEx(
        pszFilePath,
        core::ptr::null_mut(),
        DONT_RESOLVE_DLL_REFERENCES | LOAD_LIBRARY_AS_DATAFILE,
    )).cast();
    if !hModule.is_null() {
        if LoadString(
            (hModule).cast(),
            1,
            (*ScreenSaverItem).szDisplayName.as_mut_ptr(),
            (_countof!((*ScreenSaverItem).szDisplayName)) as i32,
        ) == 0
        {
            hr = StringCbCopy(
                ((*ScreenSaverItem).szDisplayName.as_mut_ptr() as *mut core::ffi::c_void).cast(),
                core::mem::size_of_val(&(*ScreenSaverItem).szDisplayName),
                pszFileName,
            );
            if hr < 0 {
                FreeLibrary((hModule).cast());
                return (FALSE) as i32;
            }
            (*ScreenSaverItem).szDisplayName[_tcslen(pszFileName) - 4] = ('\0' as WCHAR);
        }
        FreeLibrary((hModule).cast());
    } else {
        hr = StringCbCopy(
            ((*ScreenSaverItem).szDisplayName.as_mut_ptr() as *mut core::ffi::c_void).cast(),
            core::mem::size_of_val(&(*ScreenSaverItem).szDisplayName),
            w!("Unknown"),
        );
        if hr < 0 {
            return (FALSE) as i32;
        }
    }

    hr = StringCbCopy(
        ((*ScreenSaverItem).szFilename.as_mut_ptr() as *mut core::ffi::c_void).cast(),
        core::mem::size_of_val(&(*ScreenSaverItem).szFilename),
        pszFilePath,
    );
    if hr < 0 {
        return (FALSE) as i32;
    }

    return AddScreenSaverItem(hwndScreenSavers, pData, ScreenSaverItem);
}
// screensaver.c:1363 — HAND-WRITTEN, not bought. Three samplings each damaged the call
// envelope a different way, the loudest being `AddScreenSaver wants 4, got 2`: the model kept
// dropping the `pszFileName` argument. `--skip-fn` took it off the worklist so the other 15
// bodies could assemble; this is the C transcribed by hand.
//
// The `do { .. continue; .. } while (FindNextFile(..))` is the shape to get right: C's
// `continue` inside a do/while jumps to the CONDITION, so the hidden-file skip must still run
// FindNextFile. A `loop` whose body is a labelled block does exactly that — `break 'next` is
// the C's `continue`, `break 'scan` is its `break`.
unsafe extern "C" fn SearchScreenSavers(
    hwndScreenSavers: HWND,
    pData: PDATA,
    pszSearchPath: LPCTSTR,
) {
    let mut hr: HRESULT;
    let mut fd: WIN32_FIND_DATA = core::mem::zeroed();
    let hFind: HANDLE;
    let mut szFilePath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    hr = StringCbPrintf!(
        szFilePath.as_mut_ptr(),
        core::mem::size_of_val(&szFilePath),
        TEXT(w!("%s\\*.scr") as LPCTSTR),
        pszSearchPath
    );
    if hr < 0 {
        return;
    }

    hFind = FindFirstFile(szFilePath.as_ptr(), &mut fd);
    if hFind == INVALID_HANDLE_VALUE {
        return;
    }

    'scan: loop {
        'next: {
            /* Don't add any hidden screensavers */
            if (fd.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
                break 'next;
            }

            if (*pData).ScreenSaverCount >= (MAX_SCREENSAVERS) as u32 {
                break 'scan;
            }

            hr = StringCbPrintf!(
                szFilePath.as_mut_ptr(),
                core::mem::size_of_val(&szFilePath),
                TEXT(w!("%s\\%s") as LPCTSTR),
                pszSearchPath,
                fd.cFileName.as_ptr()
            );
            if hr < 0 {
                break 'scan;
            }

            if AddScreenSaver(
                hwndScreenSavers,
                pData,
                szFilePath.as_ptr(),
                (fd.cFileName.as_ptr()).cast(),
            ) == 0
            {
                break 'scan;
            }
        }
        if FindNextFile(hFind, &mut fd) == 0 {
            break 'scan;
        }
    }

    FindClose(hFind);
}
unsafe extern "C" fn EnumScreenSavers(
    hwndScreenSavers: HWND,
    pData: *mut DATA,
) {
    let mut ScreenSaverItem: *mut SCREEN_SAVER_ITEM;
    let mut pBackSlash: PTCHAR;
    let mut szSearchPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szLocalPath: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    (*pData).ScreenSaverCount = 0;

    ScreenSaverItem = (*pData).ScreenSaverItems.as_mut_ptr();

    (*ScreenSaverItem).bIsScreenSaver = (FALSE) as i32;

    LoadString(
        hApplet,
        IDS_NONE,
        (*ScreenSaverItem).szDisplayName.as_mut_ptr(),
        (_countof!((*ScreenSaverItem).szDisplayName)) as i32,
    );

    AddScreenSaverItem(hwndScreenSavers, pData, ScreenSaverItem);

    GetModuleFileName(hApplet, szLocalPath.as_mut_ptr(), _countof!(szLocalPath));
    pBackSlash = (_tcsrchr(szLocalPath.as_mut_ptr(), ('\\' as WCHAR))).cast();
    if pBackSlash != core::ptr::null_mut() {
        *pBackSlash = (('\0' as WCHAR)) as i8;
        SearchScreenSavers(hwndScreenSavers, pData, szLocalPath.as_mut_ptr());
    }

    GetSystemDirectory(szSearchPath.as_mut_ptr(), _countof!(szSearchPath));
    if pBackSlash != core::ptr::null_mut() && _tcsicmp(szSearchPath.as_mut_ptr(), szLocalPath.as_mut_ptr()) != 0 {
        SearchScreenSavers(hwndScreenSavers, pData, szSearchPath.as_mut_ptr());
    }

    GetWindowsDirectory(szSearchPath.as_mut_ptr(), _countof!(szSearchPath));
    if pBackSlash != core::ptr::null_mut() && _tcsicmp(szSearchPath.as_mut_ptr(), szLocalPath.as_mut_ptr()) != 0 {
        SearchScreenSavers(hwndScreenSavers, pData, szSearchPath.as_mut_ptr());
    }
}
unsafe extern "C" fn SetScreenSaver(hwndDlg: HWND, pData: PDATA) {
    let mut regKey: HKEY = core::ptr::null_mut();

    if RegOpenKeyEx(HKEY_CURRENT_USER,
                     w!("Control Panel\\Desktop"),
                     0,
                     KEY_ALL_ACCESS,
                     &mut regKey) == (ERROR_SUCCESS) as i32
    {
        let mut Time: i32;
        let mut bRet: i32 = 0;
        let mut Sec: TCHAR;
        let Ret: u32;

        if (*pData).ScreenSaverItems[(*pData).Selection as usize].bIsScreenSaver != 0 {
            let Length: usize = (_tcslen((*pData).ScreenSaverItems[(*pData).Selection as usize].szFilename.as_ptr()) + 1) * core::mem::size_of::<TCHAR>();
            RegSetValueEx(regKey,
                          w!("SCRNSAVE.EXE"),
                          0,
                          REG_SZ,
                          (*pData).ScreenSaverItems[(*pData).Selection as usize].szFilename.as_ptr() as PBYTE,
                          Length as u32);

            SystemParametersInfo(SPI_SETSCREENSAVEACTIVE, (TRUE) as u32, (0) as *mut c_void, SPIF_UPDATEINIFILE);
        } else {
            RegDeleteValue(regKey, w!("SCRNSAVE.EXE"));

            SystemParametersInfo(SPI_SETSCREENSAVEACTIVE, (FALSE) as u32, (0) as *mut c_void, SPIF_UPDATEINIFILE);
        }

        Ret = (SendDlgItemMessage(hwndDlg,
                                 (IDC_SCREENS_USEPASSCHK) as i32,
                                 BM_GETCHECK,
                                 0,
                                 0)) as u32;
        Sec = if Ret == BST_CHECKED { '1' as WCHAR } else { '0' as WCHAR };
        RegSetValueEx(regKey,
                      w!("ScreenSaverIsSecure"),
                      0,
                      REG_SZ,
                      &Sec as *const TCHAR as PBYTE,
                      core::mem::size_of::<TCHAR>() as u32);

        Time = (GetDlgItemInt(hwndDlg,
                             (IDC_SCREENS_TIMEDELAY) as i32,
                             &mut bRet,
                             (FALSE) as BOOL)) as i32;
        if Time == 0 {
            Time = 1;
        }
        Time *= 60;

        SystemParametersInfoW(SPI_SETSCREENSAVETIMEOUT, (Time) as UINT, (0) as *mut c_void, SPIF_SENDCHANGE | SPIF_UPDATEINIFILE);

        RegCloseKey(regKey);
    }
}
unsafe extern "C" fn OnInitDialog(hwndDlg: HWND, pData: PDATA) -> BOOL {
    let hwndSSCombo = GetDlgItem(hwndDlg, (IDC_SCREENS_LIST) as i32);
    let mut pSsValue: LPTSTR;
    let mut iCurSs: INT;
    let mut wc: WNDCLASS = core::mem::zeroed();

    let pData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<DATA>()) as SIZE_T) as *mut DATA;
    if pData.is_null() {
        EndDialog(hwndDlg, -1);
        return (FALSE) as i32;
    }

    wc.lpfnWndProc = Some(DefWindowProc);
    wc.hInstance = hApplet;
    wc.hCursor = core::ptr::null_mut();
    wc.hbrBackground = GetStockObject((BLACK_BRUSH) as i32) as HBRUSH;
    wc.lpszClassName = (szPreviewWndClass.as_ptr()).cast();

    if RegisterClass(&wc) != 0 {
        let hParent = GetDlgItem(hwndDlg, (IDC_SCREENS_PREVIEW) as i32);
        let hChild: HWND;

        if !hParent.is_null() {
            (*pData).OldPreviewProc = core::mem::transmute(GetWindowLongPtr(hParent, GWLP_WNDPROC) as *const ());
            SetWindowLongPtr(hParent, GWLP_WNDPROC, RedrawSubclassProc as LONG_PTR);
            SetWindowLongPtr(hParent, GWLP_USERDATA, pData as LONG_PTR);
        }

        let hChild = CreateWindowEx(
            0,
            szPreviewWndClass.as_ptr(),
            core::ptr::null_mut(),
            WS_CHILD | WS_CLIPCHILDREN,
            0,
            0,
            0,
            0,
            hParent,
            core::ptr::null_mut(),
            hApplet,
            core::ptr::null_mut(),
        );
        if !hChild.is_null() {
            let mut rc: RECT = core::mem::zeroed();
            GetClientRect(hParent, &mut rc);
            rc.left += (MONITOR_LEFT) as i32;
            rc.top += (MONITOR_TOP) as i32;
            MoveWindow(hChild, rc.left, rc.top, (MONITOR_WIDTH) as i32, (MONITOR_HEIGHT) as i32, (FALSE) as BOOL);
        }

        (*pData).ScreenSaverPreviewParent = hChild;
    }

    SetWindowLongPtr(hwndDlg, DWLP_USER, pData as LONG_PTR);

    (*pData).Selection = -1;

    SendDlgItemMessage(
        hwndDlg,
        (IDC_SCREENS_TIME) as i32,
        UDM_SETRANGE,
        0,
        (MAKELONG!(240, 1)) as LPARAM,
    );

    EnumScreenSavers(hwndSSCombo, pData);

    CheckRegScreenSaverIsSecure(hwndDlg);

    iCurSs = 0;
    pSsValue = GetCurrentScreenSaverValue(w!("SCRNSAVE.EXE") as LPTSTR);
    if !pSsValue.is_null() {
        let mut bFound = FALSE;
        let mut i: INT;

        i = 0;
        while i < (*pData).ScreenSaverCount as INT {
            if _tcsicmp(pSsValue, (*pData).ScreenSaverItems[i as usize].szFilename.as_ptr()) == 0 {
                bFound = TRUE;
                break;
            }
            i += 1;
        }

        if bFound == 0 {
            bFound = ((AddScreenSaver(hwndSSCombo, pData, pSsValue, w!("SCRNSAVE.EXE") as LPTSTR)) as u32) as i32;
            if bFound != 0 {
                i = (*pData).ScreenSaverCount as INT - 1;
            }
        }

        HeapFree(GetProcessHeap(), 0, pSsValue as *mut core::ffi::c_void);

        if bFound != 0 {
            iCurSs = SendMessage(
                hwndSSCombo,
                CB_FINDSTRINGEXACT,
                -1i32 as WPARAM,
                (*pData).ScreenSaverItems[i as usize].szDisplayName.as_ptr() as LPARAM,
            ) as INT;
            if iCurSs == CB_ERR {
                iCurSs = 0;
            }
        }
    }
    SendMessage(hwndSSCombo, CB_SETCURSEL, iCurSs as WPARAM, 0);

    pSsValue = GetCurrentScreenSaverValue(w!("ScreenSaveTimeOut") as LPTSTR);
    if !pSsValue.is_null() {
        let Time = _ttoi(pSsValue) as UINT / 60;

        HeapFree(GetProcessHeap(), 0, pSsValue as *mut core::ffi::c_void);

        SendDlgItemMessage(
            hwndDlg,
            (IDC_SCREENS_TIME) as i32,
            UDM_SETPOS32,
            0,
            ((Time) as i32) as LPARAM,
        );
    }

    SelectionChanged(hwndDlg, pData);

    (TRUE) as i32
}
#[no_mangle]
pub unsafe extern "system" fn ScreenSaverPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pData: PDATA;

    pData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PDATA;

    match uMsg {
        WM_INITDIALOG => {
            OnInitDialog(hwndDlg, pData);
        }

        WM_DESTROY => {
            if !(*pData).ScreenSaverPreviewParent.is_null() {
                SetWindowLongPtr(
                    GetDlgItem(hwndDlg, (IDC_SCREENS_PREVIEW) as i32),
                    GWLP_WNDPROC,
                    core::mem::transmute::<_, *const ()>((*pData).OldPreviewProc) as LONG_PTR,
                );
                DestroyWindow((*pData).ScreenSaverPreviewParent);
                (*pData).ScreenSaverPreviewParent = core::ptr::null_mut();
            }
            UnregisterClass(szPreviewWndClass.as_ptr(), hApplet);
            if !(*pData).PrevWindowPi.hProcess.is_null() {
                TerminateProcess((*pData).PrevWindowPi.hProcess, 0);
                CloseHandle((*pData).PrevWindowPi.hProcess);
                CloseHandle((*pData).PrevWindowPi.hThread);
            }
            HeapFree(GetProcessHeap(), 0, pData as LPVOID);
        }

        WM_ENDSESSION => {
            SetScreenSaverPreviewBox(hwndDlg, pData);
        }

        WM_DRAWITEM => {
            let lpDrawItem: LPDRAWITEMSTRUCT;
            lpDrawItem = lParam as LPDRAWITEMSTRUCT;

            if (*lpDrawItem).CtlID == IDC_SCREENS_PREVIEW {
                ShowScreenSaverPreview(lpDrawItem, pData);
            }
        }

        WM_COMMAND => {
            let controlId = LOWORD!(wParam);
            let command = HIWORD!(wParam);

            match (controlId) as u32 {
                IDC_SCREENS_LIST => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 {
                        SelectionChanged(hwndDlg, pData);
                        SetScreenSaverPreviewBox(hwndDlg, pData);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_SCREENS_TIMEDELAY => {
                    if command == (EN_CHANGE) as u16 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_SCREENS_POWER_BUTTON => {
                    if command == (BN_CLICKED) as u16 {
                        WinExec(
                            b"rundll32 shell32.dll,Control_RunDLL powercfg.cpl\0".as_ptr() as *const i8,
                            SW_SHOWNORMAL,
                        );
                    }
                }

                IDC_SCREENS_TESTSC => {
                    if command == (BN_CLICKED) as u16 {
                        ScreenSaverPreview(hwndDlg, pData);
                        SetScreenSaverPreviewBox(hwndDlg, pData);
                    }
                }

                IDC_SCREENS_SETTINGS => {
                    if command == (BN_CLICKED) as u16 {
                        ScreenSaverConfig(hwndDlg, pData);
                    }
                }

                IDC_SCREENS_USEPASSCHK => {
                    if command == (BN_CLICKED) as u16 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match (*lpnm).code {
                PSN_APPLY => {
                    SetScreenSaver(hwndDlg, pData);
                    return (TRUE) as LPARAM;
                }

                PSN_SETACTIVE => {
                    SetScreenSaverPreviewBox(hwndDlg, pData);
                }

                PSN_KILLACTIVE => {
                    if !(*pData).PrevWindowPi.hProcess.is_null() {
                        TerminateProcess((*pData).PrevWindowPi.hProcess, 0);
                        CloseHandle((*pData).PrevWindowPi.hProcess);
                        CloseHandle((*pData).PrevWindowPi.hThread);
                        (*pData).PrevWindowPi.hThread = core::ptr::null_mut();
                        (*pData).PrevWindowPi.hProcess = core::ptr::null_mut();
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    (FALSE) as LPARAM
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/screensaver.c
//  * PURPOSE:         Screen saver property page
//  *
//  * PROGRAMMERS:     Trevor McCort (lycan359@gmail.com)
//  *                  Ged Murphy (gedmurphy@reactos.org)
//  */
//
// #include "desk.h"
//
// #define MAX_SCREENSAVERS 100
//
// static const TCHAR szPreviewWndClass[] = TEXT("SSDemoParent");
//
// typedef struct
// {
//     BOOL  bIsScreenSaver; /* Is this a valid screensaver */
//     TCHAR szFilename[MAX_PATH];
//     TCHAR szDisplayName[256];
// } SCREEN_SAVER_ITEM;
//
//
// typedef struct _DATA
// {
//     SCREEN_SAVER_ITEM   ScreenSaverItems[MAX_SCREENSAVERS];
//     PROCESS_INFORMATION PrevWindowPi;
//     int                 Selection;
//     WNDPROC             OldPreviewProc;
//     UINT                ScreenSaverCount;
//     HWND                ScreenSaverPreviewParent;
// } DATA, *PDATA;
//
//
// static LPTSTR
// GetCurrentScreenSaverValue(LPTSTR lpValue)
// {
//     HKEY hKey;
//     LPTSTR lpBuf = NULL;
//     DWORD BufSize, Type = REG_SZ;
//     LONG Ret;
//
//     Ret = RegOpenKeyEx(HKEY_CURRENT_USER,
//                        _T("Control Panel\\Desktop"),
//                        0,
//                        KEY_READ,
//                        &hKey);
//     if (Ret != ERROR_SUCCESS)
//         return NULL;
//
//     Ret = RegQueryValueEx(hKey,
//                           lpValue,
//                           0,
//                           &Type,
//                           NULL,
//                           &BufSize);
//     if (Ret == ERROR_SUCCESS)
//     {
//         lpBuf = HeapAlloc(GetProcessHeap(), 0, BufSize);
//         if (lpBuf)
//         {
//             Ret = RegQueryValueEx(hKey,
//                                   lpValue,
//                                   0,
//                                   &Type,
//                                   (LPBYTE)lpBuf,
//                                   &BufSize);
//             if (Ret != ERROR_SUCCESS)
//             {
//                 HeapFree(GetProcessHeap(), 0, lpBuf);
//                 lpBuf = NULL;
//             }
//         }
//     }
//
//     RegCloseKey(hKey);
//
//     return lpBuf;
// }
//
//
// static VOID
// SelectionChanged(HWND hwndDlg, PDATA pData)
// {
//     HWND hwndCombo;
//     BOOL bEnable;
//     INT i;
//
//     hwndCombo = GetDlgItem(hwndDlg, IDC_SCREENS_LIST);
//
//     i = (INT)SendMessage(hwndCombo, CB_GETCURSEL, 0, 0);
//     i = (INT)SendMessage(hwndCombo, CB_GETITEMDATA, i, 0);
//
//     pData->Selection = i;
//
//     bEnable = (i != 0);
//
//     EnableWindow(GetDlgItem(hwndDlg, IDC_SCREENS_SETTINGS), bEnable);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_SCREENS_TESTSC), bEnable);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_SCREENS_USEPASSCHK), bEnable);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_SCREENS_TIMEDELAY), bEnable);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_SCREENS_TIME), bEnable);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_WAITTEXT), bEnable);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_MINTEXT), bEnable);
// }
//
//
// LRESULT __stdcall
// RedrawSubclassProc(HWND hwndDlg,
//                    UINT uMsg,
//                    WPARAM wParam,
//                    LPARAM lParam)
// {
//     HWND hwnd;
//     PDATA pData;
//     LRESULT Ret = FALSE;
//
//     pData = (PDATA)GetWindowLongPtr(hwndDlg, GWLP_USERDATA);
//     if (!pData)
//         return Ret;
//
//     Ret = CallWindowProc(pData->OldPreviewProc, hwndDlg, uMsg, wParam, lParam);
//
//     if (uMsg == WM_PAINT)
//     {
//         hwnd = pData->ScreenSaverPreviewParent;
//         if (hwnd)
//             RedrawWindow(hwnd, NULL, NULL, RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN);
//     }
//
//     return Ret;
// }
//
//
// static VOID
// ShowScreenSaverPreview( LPDRAWITEMSTRUCT draw,  PDATA pData)
// {
//     HBRUSH hBrush;
//     HDC hDC;
//     HGDIOBJ hOldObj;
//     RECT rcItem = {
//         MONITOR_LEFT,
//         MONITOR_TOP,
//         MONITOR_RIGHT,
//         MONITOR_BOTTOM
//     };
//
//     hDC = CreateCompatibleDC(draw->hDC);
//     hOldObj = SelectObject(hDC, g_GlobalData.hMonitorBitmap);
//
//     if (!IsWindowVisible(pData->ScreenSaverPreviewParent))
//     {
//         /* FIXME: Draw static bitmap inside monitor. */
//         hBrush = CreateSolidBrush(g_GlobalData.desktop_color);
//         FillRect(hDC, &rcItem, hBrush);
//         DeleteObject(hBrush);
//     }
//
//     GdiTransparentBlt(draw->hDC,
//                       draw->rcItem.left, draw->rcItem.top,
//                       draw->rcItem.right - draw->rcItem.left + 1,
//                       draw->rcItem.bottom - draw->rcItem.top + 1,
//                       hDC,
//                       0, 0,
//                       g_GlobalData.bmMonWidth, g_GlobalData.bmMonHeight,
//                       MONITOR_ALPHA);
//
//     SelectObject(hDC, hOldObj);
//     DeleteDC(hDC);
// }
//
//
// /*
//  * /p:<hwnd>    Run preview, hwnd is handle of calling window
//  */
// static VOID
// SetScreenSaverPreviewBox(HWND hwndDlg, PDATA pData)
// {
//     HWND hPreview = pData->ScreenSaverPreviewParent;
//     HRESULT hr;
//     STARTUPINFO si;
//     TCHAR szCmdline[2048];
//
//     /* Kill off the previous preview process */
//     if (pData->PrevWindowPi.hProcess)
//     {
//         TerminateProcess(pData->PrevWindowPi.hProcess, 0);
//         CloseHandle(pData->PrevWindowPi.hProcess);
//         CloseHandle(pData->PrevWindowPi.hThread);
//         pData->PrevWindowPi.hThread = pData->PrevWindowPi.hProcess = NULL;
//     }
//     ShowWindow(pData->ScreenSaverPreviewParent, SW_HIDE);
//
//     if (pData->Selection < 1)
//         return;
//
//     hr = StringCbPrintf(szCmdline, sizeof(szCmdline),
//                         TEXT("%s /p %Iu"),
//                         pData->ScreenSaverItems[pData->Selection].szFilename,
//                         (ULONG_PTR)hPreview);
//     if (FAILED(hr))
//         return;
//
//     ZeroMemory(&si, sizeof(si));
//     si.cb = sizeof(si);
//     ZeroMemory(&pData->PrevWindowPi, sizeof(pData->PrevWindowPi));
//
//     ShowWindow(pData->ScreenSaverPreviewParent, SW_SHOW);
//
//     if (!CreateProcess(NULL,
//                        szCmdline,
//                        NULL,
//                        NULL,
//                        FALSE,
//                        0,
//                        NULL,
//                        NULL,
//                        &si,
//                        &pData->PrevWindowPi))
//     {
//         pData->PrevWindowPi.hThread = pData->PrevWindowPi.hProcess = NULL;
//     }
// }
//
// static VOID
// WaitForSettingsDialog(HWND hwndDlg,
//                       HANDLE hProcess)
// {
//     DWORD dwResult;
//     MSG msg;
//
//     while (TRUE)
//     {
//         dwResult = MsgWaitForMultipleObjects(1,
//                                              &hProcess,
//                                              FALSE,
//                                              INFINITE,
//                                              QS_ALLINPUT);
//         if (dwResult == WAIT_OBJECT_0 + 1)
//         {
//             if (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE))
//             {
//                 if (msg.message == WM_QUIT)
//                 {
//                     return;
//                 }
//                 if (!IsDialogMessage(hwndDlg, &msg))
//                 {
//                     TranslateMessage(&msg);
//                     DispatchMessage(&msg);
//                 }
//             }
//         }
//         else
//         {
//             return;
//         }
//     }
// }
//
//
// /*
//  * /c:<hwnd>    Run configuration, hwnd is handle of calling window
//  */
// static VOID
// ScreenSaverConfig(HWND hwndDlg, PDATA pData)
// {
//     HRESULT hr;
//     STARTUPINFO si;
//     PROCESS_INFORMATION pi;
//     TCHAR szCmdline[2048];
//
//     if (pData->Selection < 1)
//         return;
//
//     hr = StringCbPrintf(szCmdline, sizeof(szCmdline),
//                         TEXT("%s /c:%Iu"),
//                         pData->ScreenSaverItems[pData->Selection].szFilename,
//                         (ULONG_PTR)hwndDlg);
//     if (FAILED(hr))
//         return;
//
//     ZeroMemory(&si, sizeof(si));
//     si.cb = sizeof(si);
//     ZeroMemory(&pi, sizeof(pi));
//     if (CreateProcess(NULL,
//                       szCmdline,
//                       NULL,
//                       NULL,
//                       FALSE,
//                       0,
//                       NULL,
//                       NULL,
//                       &si,
//                       &pi))
//     {
//         /* Kill off the previous preview process */
//         if (pData->PrevWindowPi.hProcess)
//         {
//             TerminateProcess(pData->PrevWindowPi.hProcess, 0);
//             CloseHandle(pData->PrevWindowPi.hProcess);
//             CloseHandle(pData->PrevWindowPi.hThread);
//             pData->PrevWindowPi.hThread = pData->PrevWindowPi.hProcess = NULL;
//         }
//
//         WaitForSettingsDialog(hwndDlg, pi.hProcess);
//         SetScreenSaverPreviewBox(hwndDlg, pData);
//
//         CloseHandle(pi.hProcess);
//         CloseHandle(pi.hThread);
//     }
// }
//
// /*
//  * /s   Run normal
//  */
// static VOID
// ScreenSaverPreview(HWND hwndDlg, PDATA pData)
// {
//     HRESULT hr;
//     STARTUPINFO si;
//     PROCESS_INFORMATION pi;
//     TCHAR szCmdline[2048];
//
//     if (pData->Selection < 1)
//         return;
//
//     /* Kill off the previous preview process */
//     if (pData->PrevWindowPi.hProcess)
//     {
//         TerminateProcess(pData->PrevWindowPi.hProcess, 0);
//         CloseHandle(pData->PrevWindowPi.hProcess);
//         CloseHandle(pData->PrevWindowPi.hThread);
//         pData->PrevWindowPi.hThread = pData->PrevWindowPi.hProcess = NULL;
//     }
//
//     hr = StringCbPrintf(szCmdline, sizeof(szCmdline),
//                         TEXT("%s /s"),
//                         pData->ScreenSaverItems[pData->Selection].szFilename);
//     if (FAILED(hr))
//         return;
//
//     ZeroMemory(&si, sizeof(si));
//     si.cb = sizeof(si);
//     ZeroMemory(&pi, sizeof(pi));
//     if (CreateProcess(NULL,
//                       szCmdline,
//                       NULL,
//                       NULL,
//                       FALSE,
//                       0,
//                       NULL,
//                       NULL,
//                       &si,
//                       &pi))
//     {
//         WaitForSingleObject(pi.hProcess, INFINITE);
//         CloseHandle(pi.hProcess);
//         CloseHandle(pi.hThread);
//     }
// }
//
//
// static VOID
// CheckRegScreenSaverIsSecure(HWND hwndDlg)
// {
//     HKEY hKey;
//     TCHAR szBuffer[2];
//     DWORD bufferSize = sizeof(szBuffer);
//     DWORD varType = REG_SZ;
//     LONG result;
//
//     if (RegOpenKeyEx(HKEY_CURRENT_USER,
//                      _T("Control Panel\\Desktop"),
//                      0,
//                      KEY_ALL_ACCESS,
//                      &hKey) == ERROR_SUCCESS)
//     {
//         result = RegQueryValueEx(hKey,
//                                 _T("ScreenSaverIsSecure"),
//                                 0,
//                                 &varType,
//                                 (LPBYTE)szBuffer,
//                                 &bufferSize);
//         RegCloseKey(hKey);
//
//         if (result == ERROR_SUCCESS)
//         {
//             if (_ttoi(szBuffer) == 1)
//             {
//                 SendDlgItemMessage(hwndDlg,
//                                    IDC_SCREENS_USEPASSCHK,
//                                    BM_SETCHECK,
//                                    (WPARAM)BST_CHECKED,
//                                    0);
//                 return;
//             }
//         }
//
//         SendDlgItemMessage(hwndDlg,
//                            IDC_SCREENS_USEPASSCHK,
//                            BM_SETCHECK,
//                            (WPARAM)BST_UNCHECKED,
//                            0);
//     }
// }
//
//
// static BOOL
// AddScreenSaverItem(
//      HWND hwndScreenSavers,
//      PDATA pData,
//      SCREEN_SAVER_ITEM* ScreenSaverItem)
// {
//     UINT i;
//
//     if (pData->ScreenSaverCount >= MAX_SCREENSAVERS)
//         return FALSE;
//
//     i = SendMessage(hwndScreenSavers,
//                     CB_ADDSTRING,
//                     0,
//                     (LPARAM)ScreenSaverItem->szDisplayName);
//     if ((i == CB_ERR) || (i == CB_ERRSPACE))
//         return FALSE;
//
//     SendMessage(hwndScreenSavers,
//                 CB_SETITEMDATA,
//                 i,
//                 (LPARAM)pData->ScreenSaverCount);
//
//     pData->ScreenSaverCount++;
//     return TRUE;
// }
//
// static BOOL
// AddScreenSaver(
//      HWND hwndScreenSavers,
//      PDATA pData,
//      LPCTSTR pszFilePath,
//      LPCTSTR pszFileName)
// {
//     SCREEN_SAVER_ITEM* ScreenSaverItem;
//     HANDLE hModule;
//     HRESULT hr;
//
//     if (pData->ScreenSaverCount >= MAX_SCREENSAVERS)
//         return FALSE;
//
//     ScreenSaverItem = pData->ScreenSaverItems + pData->ScreenSaverCount;
//
//     ScreenSaverItem->bIsScreenSaver = TRUE;
//
//     hModule = LoadLibraryEx(pszFilePath,
//                             NULL,
//                             DONT_RESOLVE_DLL_REFERENCES | LOAD_LIBRARY_AS_DATAFILE);
//     if (hModule)
//     {
//         if (LoadString(hModule,
//                        1,
//                        ScreenSaverItem->szDisplayName,
//                        _countof(ScreenSaverItem->szDisplayName)) == 0)
//         {
//             /* If the string does not exist, copy the file name */
//             hr = StringCbCopy(ScreenSaverItem->szDisplayName,
//                               sizeof(ScreenSaverItem->szDisplayName),
//                               pszFileName);
//             if (FAILED(hr))
//             {
//                 FreeLibrary(hModule);
//                 return FALSE;
//             }
//             /* Remove the .scr extension */
//             ScreenSaverItem->szDisplayName[_tcslen(pszFileName)-4] = _T('\0');
//         }
//         FreeLibrary(hModule);
//     }
//     else
//     {
//         hr = StringCbCopy(ScreenSaverItem->szDisplayName,
//                           sizeof(ScreenSaverItem->szDisplayName),
//                           _T("Unknown"));
//         if (FAILED(hr))
//             return FALSE;
//     }
//
//     hr = StringCbCopy(ScreenSaverItem->szFilename,
//                       sizeof(ScreenSaverItem->szFilename),
//                       pszFilePath);
//     if (FAILED(hr))
//         return FALSE;
//
//     return AddScreenSaverItem(hwndScreenSavers, pData, ScreenSaverItem);
// }
//
// static VOID
// SearchScreenSavers(
//      HWND hwndScreenSavers,
//      PDATA pData,
//      LPCTSTR pszSearchPath)
// {
//     HRESULT hr;
//     WIN32_FIND_DATA fd;
//     HANDLE hFind;
//     TCHAR szFilePath[MAX_PATH];
//
//     hr = StringCbPrintf(szFilePath, sizeof(szFilePath),
//                         TEXT("%s\\*.scr"), pszSearchPath);
//     if (FAILED(hr))
//         return;
//
//     hFind = FindFirstFile(szFilePath, &fd);
//     if (hFind == INVALID_HANDLE_VALUE)
//         return;
//
//     do
//     {
//         /* Don't add any hidden screensavers */
//         if (fd.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN)
//             continue;
//
//         if (pData->ScreenSaverCount >= MAX_SCREENSAVERS)
//             break;
//
//         hr = StringCbPrintf(szFilePath, sizeof(szFilePath),
//                             TEXT("%s\\%s"), pszSearchPath, fd.cFileName);
//         if (FAILED(hr))
//             break;
//
//         if (!AddScreenSaver(hwndScreenSavers, pData, szFilePath, fd.cFileName))
//             break;
//
//     } while (FindNextFile(hFind, &fd));
//
//     FindClose(hFind);
// }
//
// static VOID
// EnumScreenSavers(
//      HWND hwndScreenSavers,
//      PDATA pData)
// {
//     SCREEN_SAVER_ITEM* ScreenSaverItem;
//     PTCHAR pBackSlash;
//     TCHAR szSearchPath[MAX_PATH];
//     TCHAR szLocalPath[MAX_PATH];
//
//     /* Initialize the number of list items */
//     pData->ScreenSaverCount = 0;
//
//     /* Add the "(None)" item */
//     ScreenSaverItem = pData->ScreenSaverItems;
//
//     ScreenSaverItem->bIsScreenSaver = FALSE;
//
//     LoadString(hApplet,
//                IDS_NONE,
//                ScreenSaverItem->szDisplayName,
//                _countof(ScreenSaverItem->szDisplayName));
//
//     AddScreenSaverItem(hwndScreenSavers, pData, ScreenSaverItem);
//
//     /* Add all the screensavers where the applet is stored */
//     GetModuleFileName(hApplet, szLocalPath, _countof(szLocalPath));
//     pBackSlash = _tcsrchr(szLocalPath, _T('\\'));
//     if (pBackSlash != NULL)
//     {
//         *pBackSlash = _T('\0');
//         SearchScreenSavers(hwndScreenSavers, pData, szLocalPath);
//     }
//
//     /* Add all the screensavers in the C:\ReactOS\System32 directory */
//     GetSystemDirectory(szSearchPath, _countof(szSearchPath));
//     if (pBackSlash != NULL && _tcsicmp(szSearchPath, szLocalPath) != 0)
//         SearchScreenSavers(hwndScreenSavers, pData, szSearchPath);
//
//     /* Add all the screensavers in the C:\ReactOS directory */
//     GetWindowsDirectory(szSearchPath, _countof(szSearchPath));
//     if (pBackSlash != NULL && _tcsicmp(szSearchPath, szLocalPath) != 0)
//         SearchScreenSavers(hwndScreenSavers, pData, szSearchPath);
// }
//
//
// static VOID
// SetScreenSaver(HWND hwndDlg, PDATA pData)
// {
//     HKEY regKey;
//
//     if (RegOpenKeyEx(HKEY_CURRENT_USER,
//                      _T("Control Panel\\Desktop"),
//                      0,
//                      KEY_ALL_ACCESS,
//                      &regKey) == ERROR_SUCCESS)
//     {
//         INT Time;
//         BOOL bRet;
//         TCHAR Sec;
//         UINT Ret;
//
//         /* Set the screensaver */
//         if (pData->ScreenSaverItems[pData->Selection].bIsScreenSaver)
//         {
//             SIZE_T Length = (_tcslen(pData->ScreenSaverItems[pData->Selection].szFilename) + 1) * sizeof(TCHAR);
//             RegSetValueEx(regKey,
//                           _T("SCRNSAVE.EXE"),
//                           0,
//                           REG_SZ,
//                           (PBYTE)pData->ScreenSaverItems[pData->Selection].szFilename,
//                           (DWORD)Length);
//
//             SystemParametersInfo(SPI_SETSCREENSAVEACTIVE, TRUE, 0, SPIF_UPDATEINIFILE);
//         }
//         else
//         {
//             /* Windows deletes the value if no screensaver is set */
//             RegDeleteValue(regKey, _T("SCRNSAVE.EXE"));
//
//             SystemParametersInfo(SPI_SETSCREENSAVEACTIVE, FALSE, 0, SPIF_UPDATEINIFILE);
//         }
//
//         /* Set the secure value */
//         Ret = SendDlgItemMessage(hwndDlg,
//                                  IDC_SCREENS_USEPASSCHK,
//                                  BM_GETCHECK,
//                                  0,
//                                  0);
//         Sec = (Ret == BST_CHECKED) ? _T('1') : _T('0');
//         RegSetValueEx(regKey,
//                       _T("ScreenSaverIsSecure"),
//                       0,
//                       REG_SZ,
//                       (PBYTE)&Sec,
//                       sizeof(TCHAR));
//
//         /* Set the screensaver time delay */
//         Time = GetDlgItemInt(hwndDlg,
//                              IDC_SCREENS_TIMEDELAY,
//                              &bRet,
//                              FALSE);
//         if (Time == 0)
//             Time = 1;
//         Time *= 60; // Convert to seconds
//
//         SystemParametersInfoW(SPI_SETSCREENSAVETIMEOUT, Time, 0, SPIF_SENDCHANGE | SPIF_UPDATEINIFILE);
//
//         RegCloseKey(regKey);
//     }
// }
//
//
// static BOOL
// OnInitDialog(HWND hwndDlg, PDATA pData)
// {
//     HWND hwndSSCombo = GetDlgItem(hwndDlg, IDC_SCREENS_LIST);
//     LPTSTR pSsValue;
//     INT iCurSs;
//     WNDCLASS wc = {0};
//
//     pData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(DATA));
//     if (!pData)
//     {
//         EndDialog(hwndDlg, -1);
//         return FALSE;
//     }
//
//     wc.lpfnWndProc = DefWindowProc;
//     wc.hInstance = hApplet;
//     wc.hCursor = NULL;
//     wc.hbrBackground = (HBRUSH)GetStockObject(BLACK_BRUSH);
//     wc.lpszClassName = szPreviewWndClass;
//
//     if (RegisterClass(&wc))
//     {
//         HWND hParent = GetDlgItem(hwndDlg, IDC_SCREENS_PREVIEW);
//         HWND hChild;
//
//         if (hParent != NULL)
//         {
//             pData->OldPreviewProc = (WNDPROC)GetWindowLongPtr(hParent, GWLP_WNDPROC);
//             SetWindowLongPtr(hParent, GWLP_WNDPROC, (LONG_PTR)RedrawSubclassProc);
//             SetWindowLongPtr(hParent, GWLP_USERDATA, (LONG_PTR)pData);
//         }
//
//         hChild = CreateWindowEx(0, szPreviewWndClass, NULL,
//                                 WS_CHILD | WS_CLIPCHILDREN,
//                                 0, 0, 0, 0, hParent,
//                                 NULL, hApplet, NULL);
//         if (hChild != NULL)
//         {
//             RECT rc;
//             GetClientRect(hParent, &rc);
//             rc.left += MONITOR_LEFT;
//             rc.top += MONITOR_TOP;
//             MoveWindow(hChild, rc.left, rc.top, MONITOR_WIDTH, MONITOR_HEIGHT, FALSE);
//         }
//
//         pData->ScreenSaverPreviewParent = hChild;
//     }
//
//     SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pData);
//
//     pData->Selection = -1;
//
//     SendDlgItemMessage(hwndDlg,
//                        IDC_SCREENS_TIME,
//                        UDM_SETRANGE,
//                        0,
//                        MAKELONG(240, 1));
//
//     EnumScreenSavers(hwndSSCombo, pData);
//
//     CheckRegScreenSaverIsSecure(hwndDlg);
//
//     /* Set the current screensaver in the combo box */
//     iCurSs = 0; // Default to "(None)"
//     pSsValue = GetCurrentScreenSaverValue(_T("SCRNSAVE.EXE"));
//     if (pSsValue)
//     {
//         BOOL bFound = FALSE;
//         INT i;
//
//         /* Find whether the current screensaver is in the list */
//         for (i = 0; i < pData->ScreenSaverCount; i++)
//         {
//             if (!_tcsicmp(pSsValue, pData->ScreenSaverItems[i].szFilename))
//             {
//                 bFound = TRUE;
//                 break;
//             }
//         }
//
//         if (!bFound)
//         {
//             /* The current screensaver is not in the list: add it */
//             // i = pData->ScreenSaverCount;
//             bFound = AddScreenSaver(hwndSSCombo, pData, pSsValue, _T("SCRNSAVE.EXE"));
//             if (bFound)
//                 i = pData->ScreenSaverCount - 1;
//         }
//
//         HeapFree(GetProcessHeap(), 0, pSsValue);
//
//         if (bFound)
//         {
//             /* The current screensaver should be in the list: select it */
//             iCurSs = SendMessage(hwndSSCombo,
//                                  CB_FINDSTRINGEXACT,
//                                  -1,
//                                  (LPARAM)pData->ScreenSaverItems[i].szDisplayName);
//             if (iCurSs == CB_ERR)
//                 iCurSs = 0; // Default to "(None)"
//         }
//     }
//     SendMessage(hwndSSCombo, CB_SETCURSEL, iCurSs, 0);
//
//     /* Set the current timeout */
//     pSsValue = GetCurrentScreenSaverValue(_T("ScreenSaveTimeOut"));
//     if (pSsValue)
//     {
//         UINT Time = _ttoi(pSsValue) / 60;
//
//         HeapFree(GetProcessHeap(), 0, pSsValue);
//
//         SendDlgItemMessage(hwndDlg,
//                            IDC_SCREENS_TIME,
//                            UDM_SETPOS32,
//                            0,
//                            Time);
//     }
//
//     SelectionChanged(hwndDlg, pData);
//
//     return TRUE;
// }
//
//
// INT_PTR __stdcall
// ScreenSaverPageProc(HWND hwndDlg,
//                     UINT uMsg,
//                     WPARAM wParam,
//                     LPARAM lParam)
// {
//     PDATA pData;
//
//     pData = (PDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             OnInitDialog(hwndDlg, pData);
//             break;
//         }
//
//         case WM_DESTROY:
//         {
//             if (pData->ScreenSaverPreviewParent)
//             {
//                 SetWindowLongPtr(GetDlgItem(hwndDlg, IDC_SCREENS_PREVIEW),
//                                  GWLP_WNDPROC,
//                                  (LONG_PTR)pData->OldPreviewProc);
//                 DestroyWindow(pData->ScreenSaverPreviewParent);
//                 pData->ScreenSaverPreviewParent = NULL;
//             }
//             UnregisterClass(szPreviewWndClass, hApplet);
//             if (pData->PrevWindowPi.hProcess)
//             {
//                 TerminateProcess(pData->PrevWindowPi.hProcess, 0);
//                 CloseHandle(pData->PrevWindowPi.hProcess);
//                 CloseHandle(pData->PrevWindowPi.hThread);
//             }
//             HeapFree(GetProcessHeap(), 0, pData);
//             break;
//         }
//
//         case WM_ENDSESSION:
//         {
//             SetScreenSaverPreviewBox(hwndDlg, pData);
//             break;
//         }
//
//         case WM_DRAWITEM:
//         {
//             LPDRAWITEMSTRUCT lpDrawItem;
//             lpDrawItem = (LPDRAWITEMSTRUCT)lParam;
//
//             if (lpDrawItem->CtlID == IDC_SCREENS_PREVIEW)
//                 ShowScreenSaverPreview(lpDrawItem, pData);
//             break;
//         }
//
//         case WM_COMMAND:
//         {
//             DWORD controlId = LOWORD(wParam);
//             DWORD command   = HIWORD(wParam);
//
//             switch (controlId)
//             {
//                 case IDC_SCREENS_LIST:
//                 {
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         SelectionChanged(hwndDlg, pData);
//                         SetScreenSaverPreviewBox(hwndDlg, pData);
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//                 }
//
//                 case IDC_SCREENS_TIMEDELAY:
//                 {
//                     if (command == EN_CHANGE)
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//                 }
//
//                 case IDC_SCREENS_POWER_BUTTON: // Start Powercfg.Cpl
//                 {
//                     if (command == BN_CLICKED)
//                         WinExec("rundll32 shell32.dll,Control_RunDLL powercfg.cpl",SW_SHOWNORMAL);
//                     break;
//                 }
//
//                 case IDC_SCREENS_TESTSC: // Screensaver Preview
//                 {
//                     if (command == BN_CLICKED)
//                     {
//                         ScreenSaverPreview(hwndDlg, pData);
//                         SetScreenSaverPreviewBox(hwndDlg, pData);
//                     }
//                     break;
//                 }
//
//                 case IDC_SCREENS_SETTINGS: // Screensaver Settings
//                 {
//                     if (command == BN_CLICKED)
//                         ScreenSaverConfig(hwndDlg, pData);
//                     break;
//                 }
//
//                 case IDC_SCREENS_USEPASSCHK: // Screensaver Is Secure
//                 {
//                     if (command == BN_CLICKED)
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//                 }
//             }
//             break;
//         }
//
//         case WM_NOTIFY:
//         {
//             LPNMHDR lpnm = (LPNMHDR)lParam;
//
//             switch(lpnm->code)
//             {
//                 case PSN_APPLY:
//                 {
//                     SetScreenSaver(hwndDlg, pData);
//                     return TRUE;
//                 }
//
//                 case PSN_SETACTIVE:
//                 {
//                     /* Enable screensaver preview support */
//                     SetScreenSaverPreviewBox(hwndDlg, pData);
//                     break;
//                 }
//
//                 case PSN_KILLACTIVE:
//                 {
//                     /* Kill running preview screensaver */
//                     if (pData->PrevWindowPi.hProcess)
//                     {
//                         TerminateProcess(pData->PrevWindowPi.hProcess, 0);
//                         CloseHandle(pData->PrevWindowPi.hProcess);
//                         CloseHandle(pData->PrevWindowPi.hThread);
//                         pData->PrevWindowPi.hThread = pData->PrevWindowPi.hProcess = NULL;
//                     }
//                     break;
//                 }
//             }
//         }
//         break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
