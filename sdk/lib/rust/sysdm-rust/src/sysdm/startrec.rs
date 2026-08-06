// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _BOOTRECORD {
    pub BootType: DWORD,
    pub szSectionName: [WCHAR; 128],
    pub szBootPath: [WCHAR; MAX_PATH as usize],
    pub szOptions: [WCHAR; 512],
}

pub type BOOTRECORD = _BOOTRECORD;
pub type PBOOTRECORD = *mut _BOOTRECORD;

#[repr(C)]
pub struct _STARTINFO {
    pub szFreeldrIni: [WCHAR; (MAX_PATH + 15) as usize],
    pub szDumpFile: [WCHAR; MAX_PATH as usize],
    pub szMinidumpDir: [WCHAR; MAX_PATH as usize],
    pub dwCrashDumpEnabled: DWORD,
    pub iFreeLdrIni: INT,
}

pub type STARTINFO = _STARTINFO;
pub type PSTARTINFO = *mut _STARTINFO;

pub static mut SaveRecoveryOptions: BOOL = 0;
unsafe extern "C" fn SetTimeout(hwndDlg: HWND, Timeout: INT) {
    if Timeout == 0 {
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTUPDWN), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTEDIT), FALSE);
    } else {
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTUPDWN), TRUE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTEDIT), TRUE);
    }
    SendDlgItemMessageW(hwndDlg, IDC_STRRECLISTUPDWN, UDM_SETRANGE, 0 as WPARAM, MAKELONG(999 as i16, 0) as LPARAM);
    SendDlgItemMessageW(hwndDlg, IDC_STRRECLISTUPDWN, UDM_SETPOS, 0 as WPARAM, MAKELONG(Timeout as i16, 0) as LPARAM);
}
unsafe extern "C" fn SetRecoveryTimeout(hwndDlg: HWND, Timeout: INT) {
    if Timeout == 0 {
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECUPDWN), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECEDIT), FALSE);
    } else {
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECUPDWN), TRUE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECEDIT), TRUE);
    }
    SendDlgItemMessageW(hwndDlg, IDC_STRRECRECUPDWN, UDM_SETRANGE, 0 as WPARAM, MAKELONG(999 as i16, 0) as LPARAM);
    SendDlgItemMessageW(hwndDlg, IDC_STRRECRECUPDWN, UDM_SETPOS, 0 as WPARAM, MAKELONG(Timeout as i16, 0) as LPARAM);
}
unsafe extern "C" fn GetSystemDrive(szSystemDrive: *mut *mut u16) -> u32 {
    let mut dwBufSize: u32;

    *szSystemDrive = HeapAlloc(GetProcessHeap(), 0, MAX_PATH * core::mem::size_of::<u16>() as u32);
    if !(*szSystemDrive).is_null() {
        dwBufSize = GetEnvironmentVariableW(b"S\0y\0s\0t\0e\0m\0D\0r\0i\0v\0e\0\0\0".as_ptr() as *const u16, *szSystemDrive, MAX_PATH);
        if dwBufSize > MAX_PATH {
            let mut szTmp: *mut u16;
            let dwBufSize2: u32;

            szTmp = HeapReAlloc(GetProcessHeap(), 0, *szSystemDrive as *mut core::ffi::c_void, dwBufSize * core::mem::size_of::<u16>() as u32) as *mut u16;
            if szTmp.is_null() {
                HeapFree(GetProcessHeap(), 0, *szSystemDrive as *mut core::ffi::c_void);
                *szSystemDrive = core::ptr::null_mut();
                return 0;
            }

            *szSystemDrive = szTmp;

            dwBufSize2 = GetEnvironmentVariableW(b"S\0y\0s\0t\0e\0m\0D\0r\0i\0v\0e\0\0\0".as_ptr() as *const u16, *szSystemDrive, dwBufSize);
            if dwBufSize2 > dwBufSize || dwBufSize2 == 0 {
                HeapFree(GetProcessHeap(), 0, *szSystemDrive as *mut core::ffi::c_void);
                *szSystemDrive = core::ptr::null_mut();
                return 0;
            }
        } else if dwBufSize == 0 {
            HeapFree(GetProcessHeap(), 0, *szSystemDrive as *mut core::ffi::c_void);
            *szSystemDrive = core::ptr::null_mut();
            return 0;
        }

        return dwBufSize;
    }

    return 0;
}
unsafe extern "C" fn ReadFreeldrSection(hInf: HINF, szSectionName: *mut WCHAR) -> PBOOTRECORD {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut szName: [WCHAR; MAX_PATH] = core::mem::zeroed();
    let mut szValue: [WCHAR; MAX_PATH] = core::mem::zeroed();
    let mut LineLength: DWORD = 0;

    if SetupFindFirstLineW(
        hInf,
        szSectionName,
        core::ptr::null(),
        &mut InfContext,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    let pRecord = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<BOOTRECORD>(),
    ) as PBOOTRECORD;
    if pRecord.is_null() {
        return core::ptr::null_mut();
    }

    wcscpy(
        (*pRecord).szSectionName.as_mut_ptr(),
        szSectionName,
    );

    loop {
        if SetupGetStringFieldW(
            &mut InfContext,
            0,
            szName.as_mut_ptr(),
            szName.len() as DWORD,
            &mut LineLength,
        ) == 0
        {
            break;
        }

        if SetupGetStringFieldW(
            &mut InfContext,
            1,
            szValue.as_mut_ptr(),
            szValue.len() as DWORD,
            &mut LineLength,
        ) == 0
        {
            break;
        }

        if _wcsnicmp(
            szName.as_ptr(),
            b"B\0o\0o\0t\0T\0y\0p\0e\0\0\0".as_ptr() as *const WCHAR,
            8,
        ) == 0
        {
            if _wcsnicmp(
                szValue.as_ptr(),
                b"R\0e\0a\0c\0t\0O\0S\0\0\0".as_ptr() as *const WCHAR,
                7,
            ) == 0
            {
                (*pRecord).BootType = 1;
            } else {
                (*pRecord).BootType = 0;
            }
        } else if _wcsnicmp(
            szName.as_ptr(),
            b"S\0y\0s\0t\0e\0m\0P\0a\0t\0h\0\0\0".as_ptr() as *const WCHAR,
            10,
        ) == 0
        {
            wcscpy((*pRecord).szBootPath.as_mut_ptr(), szValue.as_ptr());
        } else if _wcsnicmp(
            szName.as_ptr(),
            b"O\0p\0t\0i\0o\0n\0s\0\0\0".as_ptr() as *const WCHAR,
            7,
        ) == 0
        {
            wcscpy((*pRecord).szOptions.as_mut_ptr(), szValue.as_ptr());
        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }

    pRecord
}
unsafe extern "C" fn LoadFreeldrSettings(hInf: HINF, hwndDlg: HWND) -> INT {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut pRecord: PBOOTRECORD;
    let mut szDefaultOs: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szName: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szValue: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut LineLength: DWORD = 0;
    let mut TimeOut: DWORD = 0;
    let mut lResult: LRESULT;

    if SetupFindFirstLineW(
        hInf,
        b"F\0R\0E\0E\0L\0O\0A\0D\0E\0R\0\0\0".as_ptr() as *const WCHAR,
        b"D\0e\0f\0a\0u\0l\0t\0O\0S\0\0\0".as_ptr() as *const WCHAR,
        &mut InfContext,
    ) == 0
    {
        return FALSE;
    }

    if SetupGetStringFieldW(
        &mut InfContext,
        1,
        szDefaultOs.as_mut_ptr(),
        (core::mem::size_of::<[WCHAR; MAX_PATH]>() / core::mem::size_of::<WCHAR>()) as DWORD,
        &mut LineLength,
    ) == 0
    {
        return FALSE;
    }

    if SetupFindFirstLineW(
        hInf,
        b"F\0R\0E\0E\0L\0O\0A\0D\0E\0R\0\0\0".as_ptr() as *const WCHAR,
        b"T\0i\0m\0e\0O\0u\0t\0\0\0".as_ptr() as *const WCHAR,
        &mut InfContext,
    ) == 0
    {
        return FALSE;
    }

    if SetupGetIntField(
        &mut InfContext,
        1,
        &mut TimeOut as *mut DWORD as PINT,
    ) == 0
    {
        return FALSE;
    }

    if SetupFindFirstLineW(
        hInf,
        b"O\0p\0e\0r\0a\0t\0i\0n\0g\0 \0S\0y\0s\0t\0e\0m\0s\0\0\0".as_ptr() as *const WCHAR,
        core::ptr::null(),
        &mut InfContext,
    ) == 0
    {
        return FALSE;
    }

    loop {
        if SetupGetStringFieldW(
            &mut InfContext,
            0,
            szName.as_mut_ptr(),
            (core::mem::size_of::<[WCHAR; MAX_PATH]>() / core::mem::size_of::<WCHAR>()) as DWORD,
            &mut LineLength,
        ) == 0
        {
            return FALSE;
        }

        if SetupGetStringFieldW(
            &mut InfContext,
            1,
            szValue.as_mut_ptr(),
            (core::mem::size_of::<[WCHAR; MAX_PATH]>() / core::mem::size_of::<WCHAR>()) as DWORD,
            &mut LineLength,
        ) == 0
        {
            return FALSE;
        }

        pRecord = ReadFreeldrSection(hInf, szName.as_mut_ptr());
        if !pRecord.is_null() {
            lResult = SendDlgItemMessageW(
                hwndDlg,
                IDC_STRECOSCOMBO,
                CB_ADDSTRING,
                0 as WPARAM,
                szValue.as_ptr() as LPARAM,
            );
            if lResult != CB_ERR {
                SendDlgItemMessageW(
                    hwndDlg,
                    IDC_STRECOSCOMBO,
                    CB_SETITEMDATA,
                    lResult as WPARAM,
                    pRecord as LPARAM,
                );
                if wcscmp(szDefaultOs.as_ptr(), szName.as_ptr()) == 0 {
                    wcscpy(szDefaultOs.as_mut_ptr(), szValue.as_ptr());
                }
            } else {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    pRecord as *mut core::ffi::c_void,
                );
            }
        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }

    lResult = SendDlgItemMessageW(
        hwndDlg,
        IDC_STRECOSCOMBO,
        CB_FINDSTRING,
        (-(1 as isize)) as WPARAM,
        szDefaultOs.as_ptr() as LPARAM,
    );
    if lResult != CB_ERR {
        SendDlgItemMessageW(
            hwndDlg,
            IDC_STRECOSCOMBO,
            CB_SETCURSEL,
            lResult as WPARAM,
            0 as LPARAM,
        );
    }

    if TimeOut != 0 {
        SendDlgItemMessageW(
            hwndDlg,
            IDC_STRECLIST,
            BM_SETCHECK,
            BST_CHECKED as WPARAM,
            0 as LPARAM,
        );
    }

    SetTimeout(hwndDlg, TimeOut as INT);

    return TRUE;
}
unsafe extern "C" fn LoadBootSettings(hInf: HINF, hwndDlg: HWND) -> INT {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut szName: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut szValue: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut LineLength: DWORD = 0;
    let mut TimeOut: DWORD = 0;
    let mut szDefaultOS: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut szOptions: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut pRecord: PBOOTRECORD;
    let mut lResult: LRESULT;

    if SetupFindFirstLineW(hInf,
                               b"b\0o\0o\0t\0 \0l\0o\0a\0d\0e\0r\0\0\0".as_ptr() as *const u16,
                               core::ptr::null(),
                               &mut InfContext) == 0 {
        return FALSE;
    }

    loop {
        if SetupGetStringFieldW(&mut InfContext,
                                   0,
                                   szName.as_mut_ptr(),
                                   (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as DWORD,
                                   &mut LineLength) == 0 {
            return FALSE;
        }

        if SetupGetStringFieldW(&mut InfContext,
                                   1,
                                   szValue.as_mut_ptr(),
                                   (core::mem::size_of_val(&szValue) / core::mem::size_of::<u16>()) as DWORD,
                                   &mut LineLength) == 0 {
            return FALSE;
        }

        if _wcsnicmp(szName.as_ptr(), b"t\0i\0m\0e\0o\0u\0t\0\0\0".as_ptr() as *const u16, 7) == 0 {
            TimeOut = _wtoi(szValue.as_ptr()) as DWORD;
        }

        if _wcsnicmp(szName.as_ptr(), b"d\0e\0f\0a\0u\0l\0t\0\0\0".as_ptr() as *const u16, 7) == 0 {
            wcscpy(szDefaultOS.as_mut_ptr(), szValue.as_ptr());
        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }

    if SetupFindFirstLineW(hInf,
                              b"o\0p\0e\0r\0a\0t\0i\0n\0g\0 \0s\0y\0s\0t\0e\0m\0s\0\0\0".as_ptr() as *const u16,
                              core::ptr::null(),
                              &mut InfContext) == 0 {
        return FALSE;
    }

    loop {
        if SetupGetStringFieldW(&mut InfContext,
                                   0,
                                   szName.as_mut_ptr(),
                                   (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as DWORD,
                                   &mut LineLength) == 0 {
            return FALSE;
        }

        if SetupGetStringFieldW(&mut InfContext,
                                   1,
                                   szValue.as_mut_ptr(),
                                   (core::mem::size_of_val(&szValue) / core::mem::size_of::<u16>()) as DWORD,
                                   &mut LineLength) == 0 {
            return FALSE;
        }

        SetupGetStringFieldW(&mut InfContext,
                              2,
                              szOptions.as_mut_ptr(),
                              (core::mem::size_of_val(&szOptions) / core::mem::size_of::<u16>()) as DWORD,
                              &mut LineLength);

        pRecord = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<BOOTRECORD>()) as PBOOTRECORD;
        if !pRecord.is_null() {
            (*pRecord).BootType = 0;
            wcscpy((*pRecord).szBootPath.as_mut_ptr(), szName.as_ptr());
            wcscpy((*pRecord).szSectionName.as_mut_ptr(), szValue.as_ptr());
            wcscpy((*pRecord).szOptions.as_mut_ptr(), szOptions.as_ptr());

            if wcscmp(szName.as_ptr(), szDefaultOS.as_ptr()) == 0 {
                wcscpy(szDefaultOS.as_mut_ptr(), szValue.as_ptr());
            }

            lResult = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_ADDSTRING, 0 as WPARAM, szValue.as_ptr() as LPARAM);
            if lResult != CB_ERR {
                SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_SETITEMDATA, lResult as WPARAM, pRecord as LPARAM);
            } else {
                HeapFree(GetProcessHeap(), 0, pRecord as *mut core::ffi::c_void);
            }
        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }

    lResult = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_FINDSTRING, 0 as WPARAM, szDefaultOS.as_ptr() as LPARAM);
    if lResult != CB_ERR {
        SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_SETCURSEL, lResult as WPARAM, 0 as LPARAM);
    }

    if TimeOut != 0 {
        SendDlgItemMessageW(hwndDlg, IDC_STRECLIST, BM_SETCHECK, BST_CHECKED as WPARAM, 0 as LPARAM);
    }

    SetTimeout(hwndDlg, TimeOut as INT);

    return TRUE;
}
unsafe extern "C" fn DeleteBootRecords(hwndDlg: HWND) {
    let mut lIndex: LRESULT;
    let mut index: i32;
    let mut pRecord: PBOOTRECORD;

    lIndex = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_GETCOUNT, 0 as WPARAM, 0 as LPARAM);
    if lIndex == CB_ERR {
        return;
    }

    index = 0;
    while index < lIndex as i32 {
        pRecord = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_GETITEMDATA, index as WPARAM, 0 as LPARAM) as PBOOTRECORD;
        if pRecord as INT_PTR != CB_ERR {
            HeapFree(GetProcessHeap(), 0, pRecord as *mut core::ffi::c_void);
        }
        index += 1;
    }

    SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_RESETCONTENT, 0 as WPARAM, 0 as LPARAM);
}
unsafe extern "C" fn LoadOSList(hwndDlg: HWND, pStartInfo: PSTARTINFO) -> LRESULT {
    let mut dwBufSize: u32;
    let mut szSystemDrive: *mut u16 = core::ptr::null_mut();
    let mut hInf: HINF;

    dwBufSize = GetSystemDrive(&mut szSystemDrive);
    if dwBufSize == 0 {
        return FALSE;
    }

    wcscpy((*pStartInfo).szFreeldrIni.as_mut_ptr(), szSystemDrive);
    wcscat((*pStartInfo).szFreeldrIni.as_mut_ptr(), w!("\\freeldr.ini").as_ptr());

    if PathFileExistsW((*pStartInfo).szFreeldrIni.as_ptr()) != 0 {
        HeapFree(GetProcessHeap(), 0, szSystemDrive as *mut core::ffi::c_void);
        hInf = SetupOpenInfFileW(
            (*pStartInfo).szFreeldrIni.as_ptr(),
            core::ptr::null(),
            INF_STYLE_OLDNT,
            core::ptr::null_mut()
        );

        if hInf != INVALID_HANDLE_VALUE {
            LoadFreeldrSettings(hInf, hwndDlg);
            SetupCloseInfFile(hInf);
            (*pStartInfo).iFreeLdrIni = 1;
            return TRUE;
        }
        return FALSE;
    }

    wcscpy((*pStartInfo).szFreeldrIni.as_mut_ptr(), szSystemDrive);
    wcscat((*pStartInfo).szFreeldrIni.as_mut_ptr(), w!("\\boot.ini").as_ptr());

    HeapFree(GetProcessHeap(), 0, szSystemDrive as *mut core::ffi::c_void);

    if PathFileExistsW((*pStartInfo).szFreeldrIni.as_ptr()) != 0 {
        hInf = SetupOpenInfFileW(
            (*pStartInfo).szFreeldrIni.as_ptr(),
            core::ptr::null(),
            INF_STYLE_OLDNT,
            core::ptr::null_mut()
        );

        if hInf != INVALID_HANDLE_VALUE {
            LoadBootSettings(hInf, hwndDlg);
            SetupCloseInfFile(hInf);
            (*pStartInfo).iFreeLdrIni = 2;
            return TRUE;
        }

        return FALSE;
    }

    return FALSE;
}
unsafe extern "C" fn SetCrashDlgItems(hwnd: HWND, pStartInfo: *mut STARTINFO) {
    if (*pStartInfo).dwCrashDumpEnabled == 0 {
        EnableWindow(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), FALSE);
        EnableWindow(GetDlgItem(hwnd, IDC_STRRECOVERWRITE), FALSE);
    } else if (*pStartInfo).dwCrashDumpEnabled == 3 {
        EnableWindow(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), TRUE);
        EnableWindow(GetDlgItem(hwnd, IDC_STRRECOVERWRITE), FALSE);
        SendMessageW(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), WM_SETTEXT, 0 as WPARAM, (*pStartInfo).szMinidumpDir.as_ptr() as LPARAM);
    } else if (*pStartInfo).dwCrashDumpEnabled == 1 || (*pStartInfo).dwCrashDumpEnabled == 2 {
        EnableWindow(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), TRUE);
        EnableWindow(GetDlgItem(hwnd, IDC_STRRECOVERWRITE), TRUE);
        SendMessageW(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), WM_SETTEXT, 0 as WPARAM, (*pStartInfo).szDumpFile.as_ptr() as LPARAM);
    }
    SendDlgItemMessageW(hwnd, IDC_STRRECDEBUGCOMBO, CB_SETCURSEL, (*pStartInfo).dwCrashDumpEnabled as WPARAM, 0 as LPARAM);
}
unsafe extern "C" fn WriteStartupRecoveryOptions(hwndDlg: HWND, pStartInfo: *mut STARTINFO) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: u32;

    lResult = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        b"System\\CurrentControlSet\\Control\\CrashControl\0".as_ptr() as *const u16,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    ) as u32;

    if lResult != ERROR_SUCCESS {
        SetLastError(lResult);
        ShowLastWin32Error(hwndDlg);
        return;
    }

    lResult = SendDlgItemMessage(
        hwndDlg,
        IDC_STRRECWRITEEVENT,
        BM_GETCHECK,
        0 as WPARAM,
        0 as LPARAM,
    ) as u32;
    RegSetValueExW(
        hKey,
        b"LogEvent\0".as_ptr() as *const u16,
        0,
        REG_DWORD,
        &lResult as *const u32 as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    );

    lResult = SendDlgItemMessage(
        hwndDlg,
        IDC_STRRECSENDALERT,
        BM_GETCHECK,
        0 as WPARAM,
        0 as LPARAM,
    ) as u32;
    RegSetValueExW(
        hKey,
        b"SendAlert\0".as_ptr() as *const u16,
        0,
        REG_DWORD,
        &lResult as *const u32 as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    );

    lResult = SendDlgItemMessage(
        hwndDlg,
        IDC_STRRECRESTART,
        BM_GETCHECK,
        0 as WPARAM,
        0 as LPARAM,
    ) as u32;
    RegSetValueExW(
        hKey,
        b"AutoReboot\0".as_ptr() as *const u16,
        0,
        REG_DWORD,
        &lResult as *const u32 as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    );

    lResult = SendDlgItemMessage(
        hwndDlg,
        IDC_STRRECOVERWRITE,
        BM_GETCHECK,
        0 as WPARAM,
        0 as LPARAM,
    ) as u32;
    RegSetValueExW(
        hKey,
        b"Overwrite\0".as_ptr() as *const u16,
        0,
        REG_DWORD,
        &lResult as *const u32 as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    );

    if (*pStartInfo).dwCrashDumpEnabled == 1
        || (*pStartInfo).dwCrashDumpEnabled == 2
    {
        SendDlgItemMessage(
            hwndDlg,
            IDC_STRRECDUMPFILE,
            WM_GETTEXT,
            (core::mem::size_of_val(&(*pStartInfo).szDumpFile) / core::mem::size_of::<u16>()) as WPARAM,
            (*pStartInfo).szDumpFile.as_ptr() as LPARAM,
        );
        RegSetValueExW(
            hKey,
            b"DumpFile\0".as_ptr() as *const u16,
            0,
            REG_EXPAND_SZ,
            (*pStartInfo).szDumpFile.as_ptr() as LPBYTE,
            (wcslen((*pStartInfo).szDumpFile.as_ptr()) + 1) * core::mem::size_of::<u16>() as u32,
        );
    } else if (*pStartInfo).dwCrashDumpEnabled == 3 {
        SendDlgItemMessage(
            hwndDlg,
            IDC_STRRECDUMPFILE,
            WM_GETTEXT,
            (core::mem::size_of_val(&(*pStartInfo).szDumpFile) / core::mem::size_of::<u16>()) as WPARAM,
            (*pStartInfo).szDumpFile.as_ptr() as LPARAM,
        );
        RegSetValueExW(
            hKey,
            b"MinidumpDir\0".as_ptr() as *const u16,
            0,
            REG_EXPAND_SZ,
            (*pStartInfo).szDumpFile.as_ptr() as LPBYTE,
            (wcslen((*pStartInfo).szDumpFile.as_ptr()) + 1) * core::mem::size_of::<u16>() as u32,
        );
    }

    RegSetValueExW(
        hKey,
        b"CrashDumpEnabled\0".as_ptr() as *const u16,
        0,
        REG_DWORD,
        &(*pStartInfo).dwCrashDumpEnabled as *const u32 as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    );
    RegCloseKey(hKey);
}
unsafe extern "C" fn LoadRecoveryOptions(hwndDlg: HWND, pStartInfo: *mut STARTINFO) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szName: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut dwValue: DWORD = 0;
    let mut dwValueLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwResult: DWORD = 0;

    dwResult = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        b"System\\CurrentControlSet\\Control\\CrashControl\0".as_ptr() as *const u16,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    ) as DWORD;

    if dwResult != ERROR_SUCCESS {
        SetLastError(dwResult);
        ShowLastWin32Error(hwndDlg);

        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECWRITEEVENT), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECSENDALERT), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRESTART), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECDEBUGCOMBO), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECDUMPFILE), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECOVERWRITE), FALSE);

        SaveRecoveryOptions = FALSE;
        return;
    }

    dwValueLength = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(hKey, b"LogEvent\0".as_ptr() as *const u16, core::ptr::null_mut(), &mut dwType, &mut dwValue as *mut DWORD as LPBYTE, &mut dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue != 0 {
        SendDlgItemMessageW(hwndDlg, IDC_STRRECWRITEEVENT, BM_SETCHECK, BST_CHECKED as WPARAM, 0 as LPARAM);
    }

    dwValueLength = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(hKey, b"SendAlert\0".as_ptr() as *const u16, core::ptr::null_mut(), &mut dwType, &mut dwValue as *mut DWORD as LPBYTE, &mut dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue != 0 {
        SendDlgItemMessageW(hwndDlg, IDC_STRRECSENDALERT, BM_SETCHECK, BST_CHECKED as WPARAM, 0 as LPARAM);
    }

    dwValueLength = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(hKey, b"AutoReboot\0".as_ptr() as *const u16, core::ptr::null_mut(), &mut dwType, &mut dwValue as *mut DWORD as LPBYTE, &mut dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue != 0 {
        SendDlgItemMessageW(hwndDlg, IDC_STRRECRESTART, BM_SETCHECK, BST_CHECKED as WPARAM, 0 as LPARAM);
    }

    dwValueLength = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(hKey, b"Overwrite\0".as_ptr() as *const u16, core::ptr::null_mut(), &mut dwType, &mut dwValue as *mut DWORD as LPBYTE, &mut dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue != 0 {
        SendDlgItemMessageW(hwndDlg, IDC_STRRECOVERWRITE, BM_SETCHECK, BST_CHECKED as WPARAM, 0 as LPARAM);
    }

    dwValueLength = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(hKey, b"CrashDumpEnabled\0".as_ptr() as *const u16, core::ptr::null_mut(), &mut dwType, &mut dwValue as *mut DWORD as LPBYTE, &mut dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue != 0 {
        (*pStartInfo).dwCrashDumpEnabled = dwValue;
    }

    dwValueLength = core::mem::size_of_val(&(*pStartInfo).szDumpFile) as DWORD;
    if RegQueryValueExW(hKey, b"DumpFile\0".as_ptr() as *const u16, core::ptr::null_mut(), &mut dwType, (*pStartInfo).szDumpFile.as_mut_ptr() as LPBYTE, &mut dwValueLength) != ERROR_SUCCESS {
        (*pStartInfo).szDumpFile[0] = 0;
    }

    dwValueLength = core::mem::size_of_val(&(*pStartInfo).szMinidumpDir) as DWORD;
    if RegQueryValueExW(hKey, b"MinidumpDir\0".as_ptr() as *const u16, core::ptr::null_mut(), &mut dwType, (*pStartInfo).szMinidumpDir.as_mut_ptr() as LPBYTE, &mut dwValueLength) != ERROR_SUCCESS {
        (*pStartInfo).szMinidumpDir[0] = 0;
    }

    if LoadStringW(hApplet, IDS_NO_DUMP, szName.as_mut_ptr(), (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as i32) != 0 {
        szName[(core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) - 1] = 0;
        SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, 0 as WPARAM, szName.as_ptr() as LPARAM);
    }

    if LoadStringW(hApplet, IDS_FULL_DUMP, szName.as_mut_ptr(), (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as i32) != 0 {
        szName[(core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) - 1] = 0;
        SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, 0 as WPARAM, szName.as_ptr() as LPARAM);
    }

    if LoadStringW(hApplet, IDS_KERNEL_DUMP, szName.as_mut_ptr(), (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as i32) != 0 {
        szName[(core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) - 1] = 0;
        SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, 0 as WPARAM, szName.as_ptr() as LPARAM);
    }

    if LoadStringW(hApplet, IDS_MINI_DUMP, szName.as_mut_ptr(), (core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) as i32) != 0 {
        szName[(core::mem::size_of_val(&szName) / core::mem::size_of::<u16>()) - 1] = 0;
        SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, 0 as WPARAM, szName.as_ptr() as LPARAM);
    }

    SetCrashDlgItems(hwndDlg, pStartInfo);
    RegCloseKey(hKey);

    SaveRecoveryOptions = TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn StartRecDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pStartInfo: PSTARTINFO;
    let mut pRecord: PBOOTRECORD;
    let mut iTimeout: i32;
    let mut lResult: LRESULT;
    let mut szTimeout: [u16; 10] = [0; 10];

    UNREFERENCED_PARAMETER(lParam);

    pStartInfo = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSTARTINFO;

    match uMsg {
        WM_INITDIALOG => {
            pStartInfo = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<STARTINFO>(),
            ) as PSTARTINFO;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pStartInfo as isize);

            LoadRecoveryOptions(hwndDlg, pStartInfo);
            SetRecoveryTimeout(hwndDlg, 0);
            return LoadOSList(hwndDlg, pStartInfo);
        }

        WM_DESTROY => {
            DeleteBootRecords(hwndDlg);
            HeapFree(GetProcessHeap(), 0, pStartInfo as *mut _);
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDC_STRRECEDIT => {
                    ShellExecuteW(
                        0,
                        b"o\0p\0e\0n\0\0\0".as_ptr() as *const u16,
                        b"n\0o\0t\0e\0p\0a\0d\0\0\0".as_ptr() as *const u16,
                        (*pStartInfo).szFreeldrIni.as_ptr(),
                        core::ptr::null_mut(),
                        SW_SHOWNORMAL,
                    );
                }

                IDOK => {
                    if SendDlgItemMessage(
                        hwndDlg,
                        IDC_STRECLIST,
                        BM_GETCHECK,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) == BST_CHECKED
                    {
                        iTimeout = SendDlgItemMessage(
                            hwndDlg,
                            IDC_STRRECLISTUPDWN,
                            UDM_GETPOS,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) as i32;
                    } else {
                        iTimeout = 0;
                    }
                    swprintf(szTimeout.as_mut_ptr(), b"%\0i\0\0\0".as_ptr() as *const u16, iTimeout);

                    lResult = SendDlgItemMessageW(
                        hwndDlg,
                        IDC_STRECOSCOMBO,
                        CB_GETCURSEL,
                        0 as WPARAM,
                        0 as LPARAM,
                    );
                    if lResult == CB_ERR {
                        DeleteBootRecords(hwndDlg);
                        return TRUE;
                    }

                    pRecord = SendDlgItemMessage(
                        hwndDlg,
                        IDC_STRECOSCOMBO,
                        CB_GETITEMDATA,
                        lResult as WPARAM,
                        0 as LPARAM,
                    ) as PBOOTRECORD;

                    if pRecord as INT_PTR != CB_ERR {
                        if (*pStartInfo).iFreeLdrIni == 1 {
                            WritePrivateProfileStringW(
                                b"F\0R\0E\0E\0L\0O\0A\0D\0E\0R\0\0\0".as_ptr() as *const u16,
                                b"T\0i\0m\0e\0O\0u\0t\0\0\0".as_ptr() as *const u16,
                                szTimeout.as_ptr(),
                                (*pStartInfo).szFreeldrIni.as_ptr(),
                            );
                            WritePrivateProfileStringW(
                                b"F\0R\0E\0E\0L\0O\0A\0D\0E\0R\0\0\0".as_ptr() as *const u16,
                                b"D\0e\0f\0a\0u\0l\0t\0O\0S\0\0\0".as_ptr() as *const u16,
                                (*pRecord).szSectionName.as_ptr(),
                                (*pStartInfo).szFreeldrIni.as_ptr(),
                            );
                        } else if (*pStartInfo).iFreeLdrIni == 2 {
                            WritePrivateProfileStringW(
                                b"b\0o\0o\0t\0 \0l\0o\0a\0d\0e\0r\0\0\0".as_ptr() as *const u16,
                                b"t\0i\0m\0e\0o\0u\0t\0\0\0".as_ptr() as *const u16,
                                szTimeout.as_ptr(),
                                (*pStartInfo).szFreeldrIni.as_ptr(),
                            );
                            WritePrivateProfileStringW(
                                b"b\0o\0o\0t\0 \0l\0o\0a\0d\0e\0r\0\0\0".as_ptr() as *const u16,
                                b"d\0e\0f\0a\0u\0l\0t\0\0\0".as_ptr() as *const u16,
                                (*pRecord).szBootPath.as_ptr(),
                                (*pStartInfo).szFreeldrIni.as_ptr(),
                            );
                        }
                    }

                    if SaveRecoveryOptions != 0 {
                        WriteStartupRecoveryOptions(hwndDlg, pStartInfo);
                    }

                    EndDialog(hwndDlg, LOWORD(wParam) as i32);
                    return TRUE;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD(wParam) as i32);
                    return TRUE;
                }

                IDC_STRECLIST => {
                    if SendDlgItemMessage(
                        hwndDlg,
                        IDC_STRECLIST,
                        BM_GETCHECK,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) == BST_CHECKED
                    {
                        SetTimeout(hwndDlg, 30);
                    } else {
                        SetTimeout(hwndDlg, 0);
                    }
                }

                IDC_STRRECREC => {
                    if SendDlgItemMessage(
                        hwndDlg,
                        IDC_STRRECREC,
                        BM_GETCHECK,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) == BST_CHECKED
                    {
                        SetRecoveryTimeout(hwndDlg, 30);
                    } else {
                        SetRecoveryTimeout(hwndDlg, 0);
                    }
                }

                IDC_STRRECDEBUGCOMBO => {
                    if HIWORD(wParam) == CBN_SELCHANGE {
                        let lResult: LRESULT;

                        lResult = SendDlgItemMessage(
                            hwndDlg,
                            IDC_STRRECDEBUGCOMBO,
                            CB_GETCURSEL,
                            0 as WPARAM,
                            0 as LPARAM,
                        );
                        if lResult != CB_ERR && lResult != (*pStartInfo).dwCrashDumpEnabled as LRESULT {
                            if (*pStartInfo).dwCrashDumpEnabled == 1
                                || (*pStartInfo).dwCrashDumpEnabled == 2
                            {
                                SendDlgItemMessageW(
                                    hwndDlg,
                                    IDC_STRRECDUMPFILE,
                                    WM_GETTEXT,
                                    (core::mem::size_of_val(&(*pStartInfo).szDumpFile) / core::mem::size_of::<u16>()) as WPARAM,
                                    (*pStartInfo).szDumpFile.as_ptr() as LPARAM,
                                );
                            } else if (*pStartInfo).dwCrashDumpEnabled == 3 {
                                SendDlgItemMessageW(
                                    hwndDlg,
                                    IDC_STRRECDUMPFILE,
                                    WM_GETTEXT,
                                    (core::mem::size_of_val(&(*pStartInfo).szMinidumpDir) / core::mem::size_of::<u16>()) as WPARAM,
                                    (*pStartInfo).szMinidumpDir.as_ptr() as LPARAM,
                                );
                            }

                            (*pStartInfo).dwCrashDumpEnabled = lResult as u32;
                            SetCrashDlgItems(hwndDlg, pStartInfo);
                        }
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/startrec.c
//  * PURPOSE:     Computer settings for startup and recovery
//  * COPYRIGHT:   Copyright 2006 Ged Murphy <gedmurphy@gmail.com>
//  *              Copyright 2006 Christoph von Wittich <Christoph@ApiViewer.de>
//  *              Copyright 2007 Johannes Anderwald <johannes.anderwald@reactos.org>
//  */
//
// #include "precomp.h"
//
// #include <shlwapi.h>
//
// typedef struct _BOOTRECORD
// {
//     DWORD BootType;
//     WCHAR szSectionName[128];
//     WCHAR szBootPath[MAX_PATH];
//     WCHAR szOptions[512];
// } BOOTRECORD, *PBOOTRECORD;
//
// typedef struct _STARTINFO
// {
//     WCHAR szFreeldrIni[MAX_PATH + 15];
//     WCHAR szDumpFile[MAX_PATH];
//     WCHAR szMinidumpDir[MAX_PATH];
//     DWORD dwCrashDumpEnabled;
//     INT iFreeLdrIni;
// } STARTINFO, *PSTARTINFO;
//
// BOOL SaveRecoveryOptions;
//
// static VOID
// SetTimeout(HWND hwndDlg, INT Timeout)
// {
//     if (Timeout == 0)
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTUPDWN), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTEDIT), FALSE);
//     }
//     else
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTUPDWN), TRUE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECLISTEDIT), TRUE);
//     }
//     SendDlgItemMessageW(hwndDlg, IDC_STRRECLISTUPDWN, UDM_SETRANGE, (WPARAM) 0, (LPARAM) MAKELONG((short) 999, 0));
//     SendDlgItemMessageW(hwndDlg, IDC_STRRECLISTUPDWN, UDM_SETPOS, (WPARAM) 0, (LPARAM) MAKELONG((short) Timeout, 0));
// }
//
// static VOID
// SetRecoveryTimeout(HWND hwndDlg, INT Timeout)
// {
//     if (Timeout == 0)
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECUPDWN), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECEDIT), FALSE);
//     }
//     else
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECUPDWN), TRUE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRECEDIT), TRUE);
//     }
//     SendDlgItemMessageW(hwndDlg, IDC_STRRECRECUPDWN, UDM_SETRANGE, (WPARAM) 0, (LPARAM) MAKELONG((short) 999, 0));
//     SendDlgItemMessageW(hwndDlg, IDC_STRRECRECUPDWN, UDM_SETPOS, (WPARAM) 0, (LPARAM) MAKELONG((short) Timeout, 0));
// }
//
//
// static DWORD
// GetSystemDrive(WCHAR **szSystemDrive)
// {
//     DWORD dwBufSize;
//
//     /* Get Path to freeldr.ini or boot.ini */
//     *szSystemDrive = HeapAlloc(GetProcessHeap(), 0, MAX_PATH * sizeof(WCHAR));
//     if (*szSystemDrive != NULL)
//     {
//         dwBufSize = GetEnvironmentVariableW(L"SystemDrive", *szSystemDrive, MAX_PATH);
//         if (dwBufSize > MAX_PATH)
//         {
//             WCHAR *szTmp;
//             DWORD dwBufSize2;
//
//             szTmp = HeapReAlloc(GetProcessHeap(), 0, *szSystemDrive, dwBufSize * sizeof(WCHAR));
//             if (szTmp == NULL)
//                 goto FailGetSysDrive;
//
//             *szSystemDrive = szTmp;
//
//             dwBufSize2 = GetEnvironmentVariableW(L"SystemDrive", *szSystemDrive, dwBufSize);
//             if (dwBufSize2 > dwBufSize || dwBufSize2 == 0)
//                 goto FailGetSysDrive;
//         }
//         else if (dwBufSize == 0)
//         {
// FailGetSysDrive:
//             HeapFree(GetProcessHeap(), 0, *szSystemDrive);
//             *szSystemDrive = NULL;
//             return 0;
//         }
//
//         return dwBufSize;
//     }
//
//     return 0;
// }
//
// static PBOOTRECORD
// ReadFreeldrSection(HINF hInf, WCHAR *szSectionName)
// {
//     PBOOTRECORD pRecord;
//     INFCONTEXT InfContext;
//     WCHAR szName[MAX_PATH];
//     WCHAR szValue[MAX_PATH];
//     DWORD LineLength;
//
//     if (!SetupFindFirstLineW(hInf,
//                             szSectionName,
//                             NULL,
//                             &InfContext))
//     {
//         /* Failed to find section */
//         return NULL;
//     }
//
//     pRecord = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(BOOTRECORD));
//     if (pRecord == NULL)
//     {
//         return NULL;
//     }
//
//     wcscpy(pRecord->szSectionName, szSectionName);
//
//     do
//     {
//         if (!SetupGetStringFieldW(&InfContext,
//                                   0,
//                                   szName,
//                                   sizeof(szName) / sizeof(WCHAR),
//                                   &LineLength))
//         {
//             break;
//         }
//
//         if (!SetupGetStringFieldW(&InfContext,
//                                   1,
//                                   szValue,
//                                   sizeof(szValue) / sizeof(WCHAR),
//                                   &LineLength))
//         {
//             break;
//         }
//
//         if (!_wcsnicmp(szName, L"BootType", 8))
//         {
//             if (!_wcsnicmp(szValue, L"ReactOS", 7))
//             {
//                 // FIXME: Store as enum
//                 pRecord->BootType = 1;
//             }
//             else
//             {
//                 pRecord->BootType = 0;
//             }
//         }
//         else if (!_wcsnicmp(szName, L"SystemPath", 10))
//         {
//             wcscpy(pRecord->szBootPath, szValue);
//         }
//         else if (!_wcsnicmp(szName, L"Options", 7))
//         {
//             // FIXME: Store flags as values
//             wcscpy(pRecord->szOptions, szValue);
//         }
//
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
//
//     return pRecord;
// }
//
//
// static INT
// LoadFreeldrSettings(HINF hInf, HWND hwndDlg)
// {
//     INFCONTEXT InfContext;
//     PBOOTRECORD pRecord;
//     WCHAR szDefaultOs[MAX_PATH];
//     WCHAR szName[MAX_PATH];
//     WCHAR szValue[MAX_PATH];
//     DWORD LineLength;
//     DWORD TimeOut;
//     LRESULT lResult;
//
//     if (!SetupFindFirstLineW(hInf,
//                            L"FREELOADER",
//                            L"DefaultOS",
//                            &InfContext))
//     {
//         /* Failed to find default os */
//         return FALSE;
//     }
//
//     if (!SetupGetStringFieldW(&InfContext,
//                              1,
//                              szDefaultOs,
//                              sizeof(szDefaultOs) / sizeof(WCHAR),
//                              &LineLength))
//     {
//         /* No key */
//         return FALSE;
//     }
//
//     if (!SetupFindFirstLineW(hInf,
//                            L"FREELOADER",
//                            L"TimeOut",
//                            &InfContext))
//     {
//         /* Expected to find timeout value */
//         return FALSE;
//     }
//
//
//     if (!SetupGetIntField(&InfContext,
//                           1,
//                           (PINT)&TimeOut))
//     {
//         /* Failed to retrieve timeout */
//         return FALSE;
//     }
//
//     if (!SetupFindFirstLineW(hInf,
//                            L"Operating Systems",
//                            NULL,
//                            &InfContext))
//     {
//        /* Expected list of operating systems */
//        return FALSE;
//     }
//
//     do
//     {
//         if (!SetupGetStringFieldW(&InfContext,
//                                  0,
//                                  szName,
//                                  sizeof(szName) / sizeof(WCHAR),
//                                  &LineLength))
//         {
//             /* The ini file is messed up */
//             return FALSE;
//         }
//
//         if (!SetupGetStringFieldW(&InfContext,
//                                  1,
//                                  szValue,
//                                  sizeof(szValue) / sizeof(WCHAR),
//                                  &LineLength))
//         {
//             /* The ini file is messed up */
//             return FALSE;
//         }
//
//         pRecord = ReadFreeldrSection(hInf, szName);
//         if (pRecord)
//         {
//             lResult = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_ADDSTRING, (WPARAM)0, (LPARAM)szValue);
//             if (lResult != CB_ERR)
//             {
//                 SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_SETITEMDATA, (WPARAM)lResult, (LPARAM)pRecord);
//                 if (!wcscmp(szDefaultOs, szName))
//                 {
//                     /* We store the friendly name as key */
//                     wcscpy(szDefaultOs, szValue);
//                 }
//             }
//             else
//             {
//                HeapFree(GetProcessHeap(), 0, pRecord);
//             }
//         }
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
//
//     /* Find default os in list */
//     lResult = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_FINDSTRING, (WPARAM)-1, (LPARAM)szDefaultOs);
//     if (lResult != CB_ERR)
//     {
//        /* Set cur sel */
//        SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_SETCURSEL, (WPARAM)lResult, (LPARAM)0);
//     }
//
//     if(TimeOut)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_STRECLIST, BM_SETCHECK, (WPARAM)BST_CHECKED, (LPARAM)0);
//     }
//
//     SetTimeout(hwndDlg, TimeOut);
//
//     return TRUE;
// }
//
// static INT
// LoadBootSettings(HINF hInf, HWND hwndDlg)
// {
//     INFCONTEXT InfContext;
//     WCHAR szName[MAX_PATH];
//     WCHAR szValue[MAX_PATH];
//     DWORD LineLength;
//     DWORD TimeOut = 0;
//     WCHAR szDefaultOS[MAX_PATH];
//     WCHAR szOptions[MAX_PATH];
//     PBOOTRECORD pRecord;
//     LRESULT lResult;
//
//     if(!SetupFindFirstLineW(hInf,
//                            L"boot loader",
//                            NULL,
//                            &InfContext))
//     {
//         return FALSE;
//     }
//
//     do
//     {
//         if (!SetupGetStringFieldW(&InfContext,
//                                  0,
//                                  szName,
//                                  sizeof(szName) / sizeof(WCHAR),
//                                  &LineLength))
//         {
//             return FALSE;
//         }
//
//         if (!SetupGetStringFieldW(&InfContext,
//                                  1,
//                                  szValue,
//                                  sizeof(szValue) / sizeof(WCHAR),
//                                  &LineLength))
//         {
//             return FALSE;
//         }
//
//         if (!_wcsnicmp(szName, L"timeout", 7))
//         {
//             TimeOut = _wtoi(szValue);
//         }
//
//         if (!_wcsnicmp(szName, L"default", 7))
//         {
//             wcscpy(szDefaultOS, szValue);
//         }
//
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
//
//     if (!SetupFindFirstLineW(hInf,
//                             L"operating systems",
//                             NULL,
//                             &InfContext))
//     {
//         /* Failed to find operating systems section */
//         return FALSE;
//     }
//
//     do
//     {
//         if (!SetupGetStringFieldW(&InfContext,
//                                  0,
//                                  szName,
//                                  sizeof(szName) / sizeof(WCHAR),
//                                  &LineLength))
//         {
//             return FALSE;
//         }
//
//         if (!SetupGetStringFieldW(&InfContext,
//                                  1,
//                                  szValue,
//                                  sizeof(szValue) / sizeof(WCHAR),
//                                  &LineLength))
//         {
//             return FALSE;
//         }
//
//         SetupGetStringFieldW(&InfContext,
//                             2,
//                             szOptions,
//                             sizeof(szOptions) / sizeof(WCHAR),
//                             &LineLength);
//
//         pRecord = (PBOOTRECORD) HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(BOOTRECORD));
//         if (pRecord)
//         {
//             pRecord->BootType = 0;
//             wcscpy(pRecord->szBootPath, szName);
//             wcscpy(pRecord->szSectionName, szValue);
//             wcscpy(pRecord->szOptions, szOptions);
//
//             if (!wcscmp(szName, szDefaultOS))
//             {
//                 /* ms boot ini stores the path not the friendly name */
//                 wcscpy(szDefaultOS, szValue);
//             }
//
//             lResult = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_ADDSTRING, (WPARAM)0, (LPARAM)szValue);
//             if (lResult != CB_ERR)
//             {
//                 SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_SETITEMDATA, (WPARAM)lResult, (LPARAM)pRecord);
//             }
//             else
//             {
//                HeapFree(GetProcessHeap(), 0, pRecord);
//             }
//         }
//
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
//
//     /* Find default os in list */
//     lResult = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_FINDSTRING, (WPARAM)0, (LPARAM)szDefaultOS);
//     if (lResult != CB_ERR)
//     {
//        /* Set cur sel */
//        SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_SETCURSEL, (WPARAM)lResult, (LPARAM)0);
//     }
//
//     if(TimeOut)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_STRECLIST, BM_SETCHECK, (WPARAM)BST_CHECKED, (LPARAM)0);
//     }
//
//     SetTimeout(hwndDlg, TimeOut);
//
//     return TRUE;
// }
//
// static VOID
// DeleteBootRecords(HWND hwndDlg)
// {
//     LRESULT lIndex;
//     LONG index;
//     PBOOTRECORD pRecord;
//
//     lIndex = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_GETCOUNT, (WPARAM)0, (LPARAM)0);
//     if (lIndex == CB_ERR)
//         return;
//
//     for (index = 0; index <lIndex; index++)
//     {
//         pRecord = (PBOOTRECORD) SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_GETITEMDATA, (WPARAM)index, (LPARAM)0);
//         if ((INT_PTR)pRecord != CB_ERR)
//         {
//             HeapFree(GetProcessHeap(), 0, pRecord);
//         }
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_RESETCONTENT, (WPARAM)0, (LPARAM)0);
// }
//
// static LRESULT
// LoadOSList(HWND hwndDlg, PSTARTINFO pStartInfo)
// {
//     DWORD dwBufSize;
//     WCHAR *szSystemDrive;
//     HINF hInf;
//
//     dwBufSize = GetSystemDrive(&szSystemDrive);
//     if (dwBufSize == 0)
//         return FALSE;
//
//     wcscpy(pStartInfo->szFreeldrIni, szSystemDrive);
//     wcscat(pStartInfo->szFreeldrIni, L"\\freeldr.ini");
//
//     if (PathFileExistsW(pStartInfo->szFreeldrIni))
//     {
//         /* Free resource previously allocated by GetSystemDrive() */
//         HeapFree(GetProcessHeap(), 0, szSystemDrive);
//         /* freeldr.ini exists */
//         hInf = SetupOpenInfFileW(pStartInfo->szFreeldrIni,
//                                 NULL,
//                                 INF_STYLE_OLDNT,
//                                 NULL);
//
//         if (hInf != INVALID_HANDLE_VALUE)
//         {
//             LoadFreeldrSettings(hInf, hwndDlg);
//             SetupCloseInfFile(hInf);
//             pStartInfo->iFreeLdrIni = 1;
//             return TRUE;
//         }
//         return FALSE;
//     }
//
//     /* Try loading boot.ini settings */
//     wcscpy(pStartInfo->szFreeldrIni, szSystemDrive);
//     wcscat(pStartInfo->szFreeldrIni, L"\\boot.ini");
//
//     /* Free resource previously allocated by GetSystemDrive() */
//     HeapFree(GetProcessHeap(), 0, szSystemDrive);
//
//     if (PathFileExistsW(pStartInfo->szFreeldrIni))
//     {
//         /* Load boot.ini settings */
//         hInf = SetupOpenInfFileW(pStartInfo->szFreeldrIni,
//                                 NULL,
//                                 INF_STYLE_OLDNT,
//                                 NULL);
//
//         if (hInf != INVALID_HANDLE_VALUE)
//         {
//             LoadBootSettings(hInf, hwndDlg);
//             SetupCloseInfFile(hInf);
//             pStartInfo->iFreeLdrIni = 2;
//             return TRUE;
//         }
//
//         return FALSE;
//     }
//
//     return FALSE;
// }
//
// static VOID
// SetCrashDlgItems(HWND hwnd, PSTARTINFO pStartInfo)
// {
//     if (pStartInfo->dwCrashDumpEnabled == 0)
//     {
//         /* No crash information required */
//         EnableWindow(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), FALSE);
//         EnableWindow(GetDlgItem(hwnd, IDC_STRRECOVERWRITE), FALSE);
//     }
//     else if (pStartInfo->dwCrashDumpEnabled == 3)
//     {
//         /* Minidump type */
//         EnableWindow(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), TRUE);
//         EnableWindow(GetDlgItem(hwnd, IDC_STRRECOVERWRITE), FALSE);
//         SendMessageW(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), WM_SETTEXT, (WPARAM)0, (LPARAM)pStartInfo->szMinidumpDir);
//     }
//     else if (pStartInfo->dwCrashDumpEnabled == 1 || pStartInfo->dwCrashDumpEnabled == 2)
//     {
//         /* Kernel or complete dump */
//         EnableWindow(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), TRUE);
//         EnableWindow(GetDlgItem(hwnd, IDC_STRRECOVERWRITE), TRUE);
//         SendMessageW(GetDlgItem(hwnd, IDC_STRRECDUMPFILE), WM_SETTEXT, (WPARAM)0, (LPARAM)pStartInfo->szDumpFile);
//     }
//     SendDlgItemMessageW(hwnd, IDC_STRRECDEBUGCOMBO, CB_SETCURSEL, (WPARAM)pStartInfo->dwCrashDumpEnabled, (LPARAM)0);
// }
//
// static VOID
// WriteStartupRecoveryOptions(HWND hwndDlg, PSTARTINFO pStartInfo)
// {
//     HKEY hKey;
//     DWORD lResult;
//
//     lResult = (DWORD)RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                                     L"System\\CurrentControlSet\\Control\\CrashControl",
//                                     0,
//                                     NULL,
//                                     REG_OPTION_NON_VOLATILE,
//                                     KEY_WRITE,
//                                     NULL,
//                                     &hKey,
//                                     NULL);
//     if (lResult != ERROR_SUCCESS)
//     {
//          /* Failed to open key */
//         SetLastError(lResult);
//         ShowLastWin32Error(hwndDlg);
//
//         return;
//     }
//
//     lResult = (DWORD) SendDlgItemMessage(hwndDlg, IDC_STRRECWRITEEVENT, BM_GETCHECK, (WPARAM)0, (LPARAM)0);
//     RegSetValueExW(hKey, L"LogEvent", 0, REG_DWORD, (LPBYTE)&lResult, sizeof(lResult));
//
//     lResult = (DWORD) SendDlgItemMessage(hwndDlg, IDC_STRRECSENDALERT, BM_GETCHECK, (WPARAM)0, (LPARAM)0);
//     RegSetValueExW(hKey, L"SendAlert", 0, REG_DWORD, (LPBYTE)&lResult, sizeof(lResult));
//
//     lResult = (DWORD) SendDlgItemMessage(hwndDlg, IDC_STRRECRESTART, BM_GETCHECK, (WPARAM)0, (LPARAM)0);
//     RegSetValueExW(hKey, L"AutoReboot", 0, REG_DWORD, (LPBYTE)&lResult, sizeof(lResult));
//
//     lResult = (DWORD) SendDlgItemMessage(hwndDlg, IDC_STRRECOVERWRITE, BM_GETCHECK, (WPARAM)0, (LPARAM)0);
//     RegSetValueExW(hKey, L"Overwrite", 0, REG_DWORD, (LPBYTE)&lResult, sizeof(lResult));
//
//
//     if (pStartInfo->dwCrashDumpEnabled == 1 || pStartInfo->dwCrashDumpEnabled == 2)
//     {
//         SendDlgItemMessage(hwndDlg, IDC_STRRECDUMPFILE, WM_GETTEXT, (WPARAM)sizeof(pStartInfo->szDumpFile) / sizeof(WCHAR), (LPARAM)pStartInfo->szDumpFile);
//         RegSetValueExW(hKey, L"DumpFile", 0, REG_EXPAND_SZ, (LPBYTE)pStartInfo->szDumpFile, (wcslen(pStartInfo->szDumpFile) + 1) * sizeof(WCHAR));
//     }
//     else if (pStartInfo->dwCrashDumpEnabled == 3)
//     {
//         SendDlgItemMessage(hwndDlg, IDC_STRRECDUMPFILE, WM_GETTEXT, (WPARAM)sizeof(pStartInfo->szDumpFile) / sizeof(WCHAR), (LPARAM)pStartInfo->szDumpFile);
//         RegSetValueExW(hKey, L"MinidumpDir", 0, REG_EXPAND_SZ, (LPBYTE)pStartInfo->szDumpFile, (wcslen(pStartInfo->szDumpFile) + 1) * sizeof(WCHAR));
//     }
//
//     RegSetValueExW(hKey, L"CrashDumpEnabled", 0, REG_DWORD, (LPBYTE)&pStartInfo->dwCrashDumpEnabled, sizeof(pStartInfo->dwCrashDumpEnabled));
//     RegCloseKey(hKey);
// }
//
// static VOID
// LoadRecoveryOptions(HWND hwndDlg, PSTARTINFO pStartInfo)
// {
//     HKEY hKey;
//     WCHAR szName[MAX_PATH];
//     DWORD dwValue, dwValueLength, dwType, dwResult;
//
//     dwResult = (DWORD)RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                                       L"System\\CurrentControlSet\\Control\\CrashControl",
//                                       0,
//                                       NULL,
//                                       REG_OPTION_NON_VOLATILE,
//                                       KEY_READ,
//                                       NULL,
//                                       &hKey,
//                                       NULL);
//     if (dwResult != ERROR_SUCCESS)
//     {
//         /* Failed to open key */
//         SetLastError(dwResult);
//         ShowLastWin32Error(hwndDlg);
//
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECWRITEEVENT), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECSENDALERT), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECRESTART), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECDEBUGCOMBO), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECDUMPFILE), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_STRRECOVERWRITE), FALSE);
//
//         SaveRecoveryOptions = FALSE;
//         return;
//     }
//
//     dwValueLength = sizeof(DWORD);
//     if (RegQueryValueExW(hKey, L"LogEvent", NULL, &dwType, (LPBYTE)&dwValue, &dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue)
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECWRITEEVENT, BM_SETCHECK, (WPARAM)BST_CHECKED, (LPARAM)0);
//
//     dwValueLength = sizeof(DWORD);
//     if (RegQueryValueExW(hKey, L"SendAlert", NULL, &dwType, (LPBYTE)&dwValue, &dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue)
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECSENDALERT, BM_SETCHECK, (WPARAM)BST_CHECKED, (LPARAM)0);
//
//     dwValueLength = sizeof(DWORD);
//     if (RegQueryValueExW(hKey, L"AutoReboot", NULL, &dwType, (LPBYTE)&dwValue, &dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue)
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECRESTART, BM_SETCHECK, (WPARAM)BST_CHECKED, (LPARAM)0);
//
//     dwValueLength = sizeof(DWORD);
//     if (RegQueryValueExW(hKey, L"Overwrite", NULL, &dwType, (LPBYTE)&dwValue, &dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue)
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECOVERWRITE, BM_SETCHECK, (WPARAM)BST_CHECKED, (LPARAM)0);
//
//     dwValueLength = sizeof(DWORD);
//     if (RegQueryValueExW(hKey, L"CrashDumpEnabled", NULL, &dwType, (LPBYTE)&dwValue, &dwValueLength) == ERROR_SUCCESS && dwType == REG_DWORD && dwValue)
//         pStartInfo->dwCrashDumpEnabled = dwValue;
//
//    dwValueLength = sizeof(pStartInfo->szDumpFile);
//    if (RegQueryValueExW(hKey, L"DumpFile", NULL, &dwType, (LPBYTE)pStartInfo->szDumpFile, &dwValueLength) != ERROR_SUCCESS)
//        pStartInfo->szDumpFile[0] = L'\0';
//
//     dwValueLength = sizeof(pStartInfo->szMinidumpDir);
//     if (RegQueryValueExW(hKey, L"MinidumpDir", NULL, &dwType, (LPBYTE)pStartInfo->szMinidumpDir, &dwValueLength) != ERROR_SUCCESS)
//         pStartInfo->szMinidumpDir[0] = L'\0';
//
//     if (LoadStringW(hApplet, IDS_NO_DUMP, szName, sizeof(szName) / sizeof(WCHAR)))
//     {
//         szName[(sizeof(szName)/sizeof(WCHAR))-1] = L'\0';
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, (WPARAM)0, (LPARAM) szName);
//     }
//
//     if (LoadStringW(hApplet, IDS_FULL_DUMP, szName, sizeof(szName) / sizeof(WCHAR)))
//     {
//         szName[(sizeof(szName)/sizeof(WCHAR))-1] = L'\0';
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, (WPARAM)0, (LPARAM) szName);
//     }
//
//     if (LoadStringW(hApplet, IDS_KERNEL_DUMP, szName, sizeof(szName) / sizeof(WCHAR)))
//     {
//         szName[(sizeof(szName)/sizeof(WCHAR))-1] = L'\0';
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, (WPARAM)0, (LPARAM) szName);
//     }
//
//     if (LoadStringW(hApplet, IDS_MINI_DUMP, szName, sizeof(szName) / sizeof(WCHAR)))
//     {
//         szName[(sizeof(szName)/sizeof(WCHAR))-1] = L'\0';
//         SendDlgItemMessageW(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_ADDSTRING, (WPARAM)0, (LPARAM) szName);
//     }
//
//     SetCrashDlgItems(hwndDlg, pStartInfo);
//     RegCloseKey(hKey);
//
//     SaveRecoveryOptions = TRUE;
// }
//
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// StartRecDlgProc(HWND hwndDlg,
//                 UINT uMsg,
//                 WPARAM wParam,
//                 LPARAM lParam)
// {
//     PSTARTINFO pStartInfo;
//     PBOOTRECORD pRecord;
//     int iTimeout;
//     LRESULT lResult;
//     WCHAR szTimeout[10];
//
//     UNREFERENCED_PARAMETER(lParam);
//
//     pStartInfo = (PSTARTINFO)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             pStartInfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(STARTINFO));
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pStartInfo);
//
//             LoadRecoveryOptions(hwndDlg, pStartInfo);
//             SetRecoveryTimeout(hwndDlg, 0);
//             return LoadOSList(hwndDlg, pStartInfo);
//
//         case WM_DESTROY:
//             DeleteBootRecords(hwndDlg);
//             HeapFree(GetProcessHeap(), 0, pStartInfo);
//             break;
//
//         case WM_COMMAND:
//             switch(LOWORD(wParam))
//             {
//                 case IDC_STRRECEDIT:
//                     ShellExecuteW(0, L"open", L"notepad", pStartInfo->szFreeldrIni, NULL, SW_SHOWNORMAL);
//                     // FIXME: Use CreateProcess and wait untill finished
//                     //  DeleteBootRecords(hwndDlg);
//                     //  LoadOSList(hwndDlg);
//                     break;
//
//                 case IDOK:
//                     /* Save timeout */
//                     if (SendDlgItemMessage(hwndDlg, IDC_STRECLIST, BM_GETCHECK, (WPARAM)0, (LPARAM)0) == BST_CHECKED)
//                         iTimeout = SendDlgItemMessage(hwndDlg, IDC_STRRECLISTUPDWN, UDM_GETPOS, (WPARAM)0, (LPARAM)0);
//                     else
//                         iTimeout = 0;
//                     swprintf(szTimeout, L"%i", iTimeout);
//
//                     lResult = SendDlgItemMessageW(hwndDlg, IDC_STRECOSCOMBO, CB_GETCURSEL, (WPARAM)0, (LPARAM)0);
//                     if (lResult == CB_ERR)
//                     {
//                         /* ? */
//                         DeleteBootRecords(hwndDlg);
//                         return TRUE;
//                     }
//
//                     pRecord = (PBOOTRECORD) SendDlgItemMessage(hwndDlg, IDC_STRECOSCOMBO, CB_GETITEMDATA, (WPARAM)lResult, (LPARAM)0);
//
//                     if ((INT_PTR)pRecord != CB_ERR)
//                     {
//                         if (pStartInfo->iFreeLdrIni == 1) // FreeLdrIni style
//                         {
//                             /* Set default timeout */
//                             WritePrivateProfileStringW(L"FREELOADER",
//                                                       L"TimeOut",
//                                                       szTimeout,
//                                                       pStartInfo->szFreeldrIni);
//                             /* Set default OS */
//                             WritePrivateProfileStringW(L"FREELOADER",
//                                                       L"DefaultOS",
//                                                       pRecord->szSectionName,
//                                                       pStartInfo->szFreeldrIni);
//
//                         }
//                         else if (pStartInfo->iFreeLdrIni == 2) // BootIni style
//                         {
//                             /* Set default timeout */
//                             WritePrivateProfileStringW(L"boot loader",
//                                                       L"timeout",
//                                                       szTimeout,
//                                                       pStartInfo->szFreeldrIni);
//                             /* Set default OS */
//                             WritePrivateProfileStringW(L"boot loader",
//                                                       L"default",
//                                                       pRecord->szBootPath,
//                                                       pStartInfo->szFreeldrIni);
//
//                         }
//                     }
//
//                     if (SaveRecoveryOptions)
//                     {
//                         WriteStartupRecoveryOptions(hwndDlg, pStartInfo);
//                     }
//
//                     EndDialog(hwndDlg,
//                               LOWORD(wParam));
//                     return TRUE;
//
//                 case IDCANCEL:
//                     EndDialog(hwndDlg,
//                               LOWORD(wParam));
//                     return TRUE;
//
//                 case IDC_STRECLIST:
//                     if (SendDlgItemMessage(hwndDlg, IDC_STRECLIST, BM_GETCHECK, (WPARAM)0, (LPARAM)0) == BST_CHECKED)
//                         SetTimeout(hwndDlg, 30);
//                     else
//                         SetTimeout(hwndDlg, 0);
//                     break;
//
//                 case IDC_STRRECREC:
//                     if (SendDlgItemMessage(hwndDlg, IDC_STRRECREC, BM_GETCHECK, (WPARAM)0, (LPARAM)0) == BST_CHECKED)
//                         SetRecoveryTimeout(hwndDlg, 30);
//                     else
//                         SetRecoveryTimeout(hwndDlg, 0);
//                     break;
//
//                 case IDC_STRRECDEBUGCOMBO:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         LRESULT lResult;
//
//                         lResult = SendDlgItemMessage(hwndDlg, IDC_STRRECDEBUGCOMBO, CB_GETCURSEL, (WPARAM)0, (LPARAM)0);
//                         if (lResult != CB_ERR && lResult != (LRESULT)pStartInfo->dwCrashDumpEnabled)
//                         {
//                             if (pStartInfo->dwCrashDumpEnabled == 1 || pStartInfo->dwCrashDumpEnabled == 2)
//                             {
//                                 SendDlgItemMessageW(hwndDlg, IDC_STRRECDUMPFILE, WM_GETTEXT, (WPARAM)sizeof(pStartInfo->szDumpFile) / sizeof(WCHAR), (LPARAM)pStartInfo->szDumpFile);
//                             }
//                             else if (pStartInfo->dwCrashDumpEnabled == 3)
//                             {
//                                 SendDlgItemMessageW(hwndDlg, IDC_STRRECDUMPFILE, WM_GETTEXT, (WPARAM)sizeof(pStartInfo->szMinidumpDir) / sizeof(WCHAR), (LPARAM)pStartInfo->szMinidumpDir);
//                             }
//
//                             pStartInfo->dwCrashDumpEnabled = (DWORD)lResult;
//                             SetCrashDlgItems(hwndDlg, pStartInfo);
//                         }
//                     }
//                     break;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
