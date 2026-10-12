use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "system" fn StartApplication(lpDlg: LPVOID) -> DWORD {
    let hList: HWND = GetDlgItem(lpDlg as HWND, (IDC_COMP_LIST as i32));
    let mut Index: INT;
    let Count: INT = SendMessage(hList, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut szData: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let key: [u16; 56] = [
        'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
        '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
        'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16,
        'w' as u16, 's' as u16, '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16,
        'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16,
        'n' as u16, '\\' as u16, 'R' as u16, 'u' as u16, 'n' as u16, 'O' as u16, 'n' as u16, 'c' as u16,
        'e' as u16, '\\' as u16, 'S' as u16, 'e' as u16, 't' as u16, 'u' as u16, 'p' as u16, 0,
    ];

    Index = 0 as INT;
    while Index < Count {
        SendMessage(hList, LB_GETTEXT as UINT, Index as WPARAM, core::ptr::addr_of_mut!(szData) as *mut TCHAR as LPARAM);

        ZeroMemory!(core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void, core::mem::size_of::<STARTUPINFOW>());
        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
        si.dwFlags = STARTF_USESHOWWINDOW as DWORD;
        si.wShowWindow = SW_SHOW as WORD;

        if CreateProcess(core::ptr::null_mut(), core::ptr::addr_of_mut!(szData) as LPTSTR, core::ptr::null_mut(), core::ptr::null_mut(), (FALSE as i32), 0 as DWORD, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(si), core::ptr::addr_of_mut!(pi)) != 0 {
            WaitForSingleObjectEx(pi.hProcess, INFINITE as DWORD, TRUE as BOOL);

            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
        }
        Index += 1;
    }

    RegDeleteKey(HKEY_LOCAL_MACHINE, key.as_ptr() as LPCTSTR);

    PostMessage(lpDlg as HWND, WM_CLOSE as UINT, 0 as WPARAM, 0 as LPARAM);

    return 0 as DWORD;
}

unsafe extern "C" fn InitDialog(mut hDlg: HWND) {
    let mut szAppPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szData: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwIndex: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwData: DWORD = 0;
    let mut dwThreadId: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    if RegOpenKeyEx(
        HKEY_LOCAL_MACHINE as HKEY,
        (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0073, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x005c, 0x0052, 0x0075, 0x006e, 0x004f, 0x006e, 0x0063, 0x0065, 0x005c, 0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x0000u16]).as_ptr() /* _T "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce\\Setup" */,
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        &raw mut hKey,
    ) == ERROR_SUCCESS as LSTATUS
    {
        dwIndex = 0 as DWORD;
        loop {
            dwSize = ::core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() as DWORD;
            dwData = (::core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>()
                / (core::mem::size_of::<TCHAR>() as usize)) as DWORD;
            if RegEnumValue(
                hKey,
                dwIndex,
                &raw mut szAppPath as LPWSTR,
                &raw mut dwSize,
                core::ptr::null_mut(),
                &raw mut dwType,
                &raw mut szData as *mut TCHAR as LPBYTE,
                &raw mut dwData,
            ) == ERROR_SUCCESS as LSTATUS
            {
                'body: {
                    if dwType != REG_SZ as DWORD {
                        break 'body;
                    }
                    SendMessage(
                        GetDlgItem(hDlg, IDC_COMP_LIST as i32),
                        LB_ADDSTRING as UINT,
                        0 as WPARAM,
                        &raw mut szData as *mut TCHAR as LPARAM,
                    );
                }
            }
            dwIndex = dwIndex.wrapping_add(1);
        }

        RegCloseKey(hKey);
    }

    CloseHandle(CreateThread(
        core::ptr::null_mut(),
        0 as DWORD,
        Some(StartApplication),
        hDlg as LPVOID,
        0 as DWORD,
        &raw mut dwThreadId,
    ));
}

unsafe extern "system" fn RunOnceDlgProc(hDlg: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match Msg {
        WM_INITDIALOG => {
            InitDialog(hDlg);
        }
        WM_CLOSE => {
            EndDialog(hDlg, 0);
        }
        _ => {}
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInst: HINSTANCE,
    hPrevInst: HINSTANCE,
    lpCmdLine: LPTSTR,
    nCmdShow: INT,
) -> INT {
    let mut lpCmd: LPCTSTR = GetCommandLine() as LPCTSTR;
    let mut szAppPath: [TCHAR; (MAX_PATH) as _] = core::mem::zeroed();
    let mut szData: [TCHAR; (MAX_PATH) as _] = core::mem::zeroed();
    let mut dwIndex: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwData: DWORD = 0;
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut bRunApps: BOOL = (FALSE as i32);
    let mut hKey: HKEY = core::ptr::null_mut();

    while *lpCmd != 0 {
        while *lpCmd != 0 && *lpCmd != '/' as TCHAR && *lpCmd != '-' as TCHAR {
            lpCmd = lpCmd.add(1);
        }
        if *lpCmd == 0 {
            break;
        }
        lpCmd = lpCmd.add(1);
        if *lpCmd == 'r' as TCHAR {
            bRunApps = (TRUE as i32);
        }
        lpCmd = lpCmd.add(1);
    }

    if bRunApps != 0 {
        let RUNONCE_KEY: [u16; 50] = [
            'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16,
            'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16,
            's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16,
            'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16, 'C' as u16,
            'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16,
            'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16,
            'R' as u16, 'u' as u16, 'n' as u16, 'O' as u16, 'n' as u16, 'c' as u16, 'e' as u16,
            0,
        ];

        if RegOpenKeyEx(
            HKEY_LOCAL_MACHINE,
            core::ptr::addr_of!(RUNONCE_KEY) as LPCTSTR,
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hKey),
        ) == (ERROR_SUCCESS as i32)
        {
            dwIndex = 0;
            loop {
                'body: {
                    dwSize = core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() as DWORD;
                    dwData = (core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>()
                        / (core::mem::size_of::<TCHAR>() as usize)) as DWORD;

                    if RegEnumValue(
                        hKey,
                        dwIndex,
                        core::ptr::addr_of_mut!(szAppPath) as *mut TCHAR,
                        core::ptr::addr_of_mut!(dwSize),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwType),
                        core::ptr::addr_of_mut!(szData) as *mut TCHAR as LPBYTE,
                        core::ptr::addr_of_mut!(dwData),
                    ) == (ERROR_SUCCESS as i32)
                    {
                        RegDeleteValue(hKey, core::ptr::addr_of_mut!(szAppPath) as *mut TCHAR);

                        if dwType != REG_SZ {
                            break 'body;
                        }

                        ZeroMemory!(
                            core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void,
                            core::mem::size_of::<STARTUPINFOW>(),
                        );
                        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
                        si.dwFlags = STARTF_USESHOWWINDOW;
                        si.wShowWindow = (SW_SHOW as u16);

                        if CreateProcess(
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(szData) as *mut TCHAR,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            (FALSE as i32),
                            0,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(si),
                            core::ptr::addr_of_mut!(pi),
                        ) == 0
                        {
                            break 'body;
                        }

                        WaitForSingleObjectEx(pi.hProcess, INFINITE, (TRUE) as BOOL);

                        CloseHandle(pi.hProcess);
                        CloseHandle(pi.hThread);
                    }
                }
                dwIndex = dwIndex.wrapping_add(1);
            }

            RegCloseKey(hKey);
        }

        return 1;
    }

    DialogBox!(
        hInst,
        MAKEINTRESOURCE!(IDD_RUNONCE_DLG),
        core::ptr::null_mut(),
        Some(RunOnceDlgProc),
    );

    return 0;
}
