use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use winsrv_sys::consts::*;
use winsrv_sys::winsrv::*;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// Win32 APIs are stdcall (extern "system") — declaring them extern "C"
// links on amd64/arm64 but produces the wrong `_Name` (vs `_Name@N`)
// decoration on i386, an undefined-symbol link failure. (SUCCEEDED and
// StringCbCatW come inline from winsrv_sys::consts / explorer_sys::externs;
// ZeroMemory is a macro → RtlZeroMemory.)
extern "system" {
    fn SHGetFolderPathW(hwnd: *mut core::ffi::c_void, csidl: i32, hToken: *mut core::ffi::c_void, dwFlags: u32, pszPath: LPWSTR) -> HRESULT;
    fn GetBinaryTypeW(lpApplicationName: LPCWSTR, lpBinaryType: *mut u32) -> i32;
    fn ShellExecuteExW(pExecInfo: *mut SHELLEXECUTEINFOW) -> i32;
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: u32) -> u32;
    fn IsUserAnAdmin() -> i32;
}
// msvcrt CRT helpers are genuinely cdecl.
extern "C" {
    fn wcslen(s: *const u16) -> usize;
    fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    fn debugstr_w(s: *const u16) -> *const u8;
}

// ---- file-scope preamble (translated) ----
pub static mut s_hStartupMutex: HANDLE = 0 as HANDLE;
pub const INVALID_RUNCMD_RETURN: i32 = -1;

unsafe fn runCmd(cmdline: LPWSTR, dir: LPCWSTR, wait: i32, minimized: i32) -> i32 {
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut info: PROCESS_INFORMATION = core::mem::zeroed();
    let mut exit_code: u32 = 0;

    core::ptr::write_bytes(&mut si as *mut STARTUPINFOW, 0, 1);
    si.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    if minimized != 0 {
        si.dwFlags = STARTF_USESHOWWINDOW;
        si.wShowWindow = SW_MINIMIZE as u16;
    }
    core::ptr::write_bytes(&mut info as *mut PROCESS_INFORMATION, 0, 1);

    if CreateProcessW(
        core::ptr::null_mut(),
        cmdline,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        0,
        core::ptr::null_mut(),
        dir,
        &mut si,
        &mut info,
    ) == 0
    {
        TRACE!("Failed to run command (%lu)\n", GetLastError());
        /* Boot/test marker — startup-item visibility (task: ProcessStartupItems
         * reliability; docs/engineering/payload-harness-design.md Workstream A).
         * TRACE! is DCE'd in release builds, so these are real DbgPrints. */
        DbgPrint(
            b"[EXPLORER] startup: runCmd FAILED (%lu): %ws\n\0".as_ptr() as *const i8,
            GetLastError(),
            cmdline,
        );
        return INVALID_RUNCMD_RETURN;
    }

    TRACE!("Successfully ran command\n");
    DbgPrint(
        b"[EXPLORER] startup: runCmd OK (PID %lu): %ws\n\0".as_ptr() as *const i8,
        info.dwProcessId,
        cmdline,
    );

    if wait != 0 {
        let Handles: [HANDLE; 1] = [info.hProcess];
        let nCount: u32 = 1;
        let mut dwWait: u32;
        let mut msg: MSG = core::mem::zeroed();

        loop {
            dwWait = MsgWaitForMultipleObjects(
                nCount,
                Handles.as_ptr(),
                0,
                INFINITE,
                QS_ALLINPUT,
            );

            if dwWait != WAIT_OBJECT_0 + nCount {
                if dwWait >= WAIT_OBJECT_0 && dwWait < WAIT_OBJECT_0 + nCount {
                    TRACE!("Event %u signaled\n", dwWait - WAIT_OBJECT_0);
                } else {
                    WARN!("Return code: %u\n", dwWait);
                }
                break;
            }

            while PeekMessageW(&mut msg, core::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        GetExitCodeProcess(info.hProcess, &mut exit_code);
    }

    CloseHandle(info.hThread);
    CloseHandle(info.hProcess);

    return exit_code as i32;
}

unsafe fn ProcessRunKeys(hkRoot: HKEY, szKeyName: LPCWSTR, bDelete: i32, bSynchronous: i32) -> i32 {
    let mut hkWin: HKEY = core::ptr::null_mut();
    let mut hkRun: HKEY = core::ptr::null_mut();
    let mut res: i32;
    let mut i: u32 = 0;
    let mut cbMaxCmdLine: u32 = 0;
    let mut cchMaxValue: u32 = 0;
    let mut szCmdLine: *mut u16 = core::ptr::null_mut();
    let mut szValue: *mut u16 = core::ptr::null_mut();

    if hkRoot as usize == 0x80000002 {
        TRACE!("processing %ls entries under HKLM\n", szKeyName);
    } else {
        TRACE!("processing %ls entries under HKCU\n", szKeyName);
    }

    res = RegOpenKeyExW(
        hkRoot,
        w!("Software\\Microsoft\\Windows\\CurrentVersion").as_ptr() as LPCWSTR,
        0,
        KEY_READ,
        &mut hkWin,
    ) as i32;
    if res != ERROR_SUCCESS as i32 {
        TRACE!("RegOpenKeyW failed on Software\\Microsoft\\Windows\\CurrentVersion (%ld)\n", res);
        res = 'end: {
            break 'end res;
        };
    } else {
        res = RegOpenKeyExW(
            hkWin,
            szKeyName,
            0,
            if bDelete != 0 { KEY_ALL_ACCESS } else { KEY_READ },
            &mut hkRun,
        ) as i32;
        if res != ERROR_SUCCESS as i32 {
            if res == ERROR_FILE_NOT_FOUND as i32 {
                TRACE!("Key doesn't exist - nothing to be done\n");
                res = ERROR_SUCCESS as i32;
            } else {
                TRACE!("RegOpenKeyExW failed on run key (%ld)\n", res);
            }
            res = 'end: {
                break 'end res;
            };
        } else {
            res = RegQueryInfoKeyW(
                hkRun,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut i,
                &mut cchMaxValue,
                &mut cbMaxCmdLine,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as i32;
            if res != ERROR_SUCCESS as i32 {
                TRACE!("Couldn't query key info (%ld)\n", res);
                res = 'end: {
                    break 'end res;
                };
            } else {
                if i == 0 {
                    TRACE!("No commands to execute.\n");
                    res = ERROR_SUCCESS as i32;
                    res = 'end: {
                        break 'end res;
                    };
                } else {
                    szCmdLine = HeapAlloc(hProcessHeap, 0, cbMaxCmdLine as SIZE_T) as *mut u16;
                    if szCmdLine.is_null() {
                        TRACE!("Couldn't allocate memory for the commands to be executed\n");
                        res = ERROR_NOT_ENOUGH_MEMORY as i32;
                        res = 'end: {
                            break 'end res;
                        };
                    } else {
                        cchMaxValue += 1;
                        szValue = HeapAlloc(hProcessHeap, 0, (cchMaxValue as SIZE_T) * core::mem::size_of::<u16>() as SIZE_T) as *mut u16;
                        if szValue.is_null() {
                            TRACE!("Couldn't allocate memory for the value names\n");
                            res = ERROR_NOT_ENOUGH_MEMORY as i32;
                            res = 'end: {
                                break 'end res;
                            };
                        } else {
                            let mut out_of_memory = false;
                            while i > 0 {
                                let mut szCmdLineExp: *mut u16 = core::ptr::null_mut();
                                let mut cchValLength: u32 = cchMaxValue;
                                let mut cbDataLength: u32 = cbMaxCmdLine;
                                let mut type_: u32 = 0;

                                i -= 1;

                                res = RegEnumValueW(
                                    hkRun,
                                    i,
                                    szValue,
                                    &mut cchValLength,
                                    core::ptr::null_mut(),
                                    &mut type_,
                                    szCmdLine as PBYTE,
                                    &mut cbDataLength,
                                ) as i32;
                                if res != ERROR_SUCCESS as i32 {
                                    TRACE!("Couldn't read in value %lu - %ld\n", i, res);
                                    continue;
                                }

                                if GetSystemMetrics(SM_CLEANBOOT as i32) != 0 && *szValue != b'*' as u16 {
                                    continue;
                                }

                                if bDelete != 0 {
                                    res = RegDeleteValueW(hkRun, szValue) as i32;
                                    if res != ERROR_SUCCESS as i32 {
                                        TRACE!("Couldn't delete value - %lu, %ld. Running command anyways.\n", i, res);
                                    }
                                }

                                if type_ != REG_SZ && type_ != REG_EXPAND_SZ {
                                    TRACE!("Incorrect type of value #%lu (%lu)\n", i, type_);
                                    continue;
                                }

                                if type_ == REG_EXPAND_SZ {
                                    let dwNumOfChars: u32 = ExpandEnvironmentStringsW(szCmdLine, core::ptr::null_mut(), 0);
                                    if dwNumOfChars != 0 {
                                        szCmdLineExp = HeapAlloc(hProcessHeap, 0, (dwNumOfChars as SIZE_T) * core::mem::size_of::<u16>() as SIZE_T) as *mut u16;

                                        if szCmdLineExp.is_null() {
                                            TRACE!("Couldn't allocate memory for the commands to be executed\n");
                                            res = ERROR_NOT_ENOUGH_MEMORY as i32;
                                            // the C's `goto end`: skip the ERROR_SUCCESS below
                                            out_of_memory = true;
                                            break;
                                        }

                                        ExpandEnvironmentStringsW(szCmdLine, szCmdLineExp, dwNumOfChars);
                                    }
                                }

                                res = runCmd(
                                    if !szCmdLineExp.is_null() { szCmdLineExp } else { szCmdLine },
                                    core::ptr::null(),
                                    bSynchronous,
                                    0,
                                );
                                if res == INVALID_RUNCMD_RETURN {
                                    TRACE!("Error running cmd #%lu (%lu)\n", i, GetLastError());
                                }

                                if !szCmdLineExp.is_null() {
                                    HeapFree(hProcessHeap, 0, szCmdLineExp as *mut core::ffi::c_void);
                                    szCmdLineExp = core::ptr::null_mut();
                                }

                                TRACE!("Done processing cmd #%lu\n", i);
                            }

                            if !out_of_memory {
                                res = ERROR_SUCCESS as i32;
                            }
                        }
                    }
                }
            }
        }
    }

    if !szValue.is_null() {
        HeapFree(hProcessHeap, 0, szValue as *mut core::ffi::c_void);
    }
    if !szCmdLine.is_null() {
        HeapFree(hProcessHeap, 0, szCmdLine as *mut core::ffi::c_void);
    }
    if !hkRun.is_null() {
        RegCloseKey(hkRun);
    }
    if !hkWin.is_null() {
        RegCloseKey(hkWin);
    }

    TRACE!("done\n");

    if res == ERROR_SUCCESS as i32 { 1 } else { 0 }
}

unsafe fn ProcessRunOnceEx(hkRoot: HKEY) -> u8 {
    let mut hkRunOnceEx: HKEY = core::ptr::null_mut();
    let mut res: i32;
    let mut cmdLine: [u16; 39] = [
        'r' as u16, 'u' as u16, 'n' as u16, 'd' as u16, 'l' as u16, 'l' as u16, '3' as u16, '2' as u16, ' ' as u16,
        'i' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'o' as u16, 'n' as u16, 'c' as u16, 'e' as u16, '.' as u16,
        'd' as u16, 'l' as u16, 'l' as u16, ' ' as u16, 'R' as u16, 'u' as u16, 'n' as u16, 'O' as u16, 'n' as u16,
        'c' as u16, 'e' as u16, 'E' as u16, 'x' as u16, 'P' as u16, 'r' as u16, 'o' as u16, 'c' as u16, 'e' as u16,
        's' as u16, 's' as u16, 0
    ];
    let mut dwSubKeyCnt: u32 = 0;

    res = RegOpenKeyExW(
        hkRoot,
        ['S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16, '\\' as u16,
         'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16,
         '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16,
         'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16, 'e' as u16,
         'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16, 'R' as u16, 'u' as u16, 'n' as u16,
         'O' as u16, 'n' as u16, 'c' as u16, 'e' as u16, 'E' as u16, 'x' as u16, 0].as_ptr(),
        0,
        KEY_READ,
        &mut hkRunOnceEx
    );
    
    if res != ERROR_SUCCESS as i32 {
        goto_end(hkRunOnceEx, res)
    } else {
        res = RegQueryInfoKeyW(
            hkRunOnceEx,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwSubKeyCnt,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut()
        );

        if res != ERROR_SUCCESS as i32 {
            goto_end(hkRunOnceEx, res)
        } else {
            if dwSubKeyCnt != 0 {
                if runCmd(cmdLine.as_mut_ptr(), core::ptr::null(), 1, 1) == INVALID_RUNCMD_RETURN {
                    res = GetLastError() as i32;
                    goto_end(hkRunOnceEx, res)
                } else {
                    goto_end(hkRunOnceEx, res)
                }
            } else {
                goto_end(hkRunOnceEx, res)
            }
        }
    }
}

unsafe fn goto_end(hkRunOnceEx: HKEY, res: i32) -> u8 {
    if !hkRunOnceEx.is_null() {
        RegCloseKey(hkRunOnceEx);
    }
    if res == ERROR_SUCCESS as i32 { 1u8 } else { 0u8 }
}

unsafe fn AutoStartupApplications(nCSIDL_Folder: i32) -> u8 {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let hResult: HRESULT;
    let hFind: HANDLE;
    let mut FoundData: WIN32_FIND_DATAW = core::mem::zeroed();
    let cchPathLen: usize;

    const SHGFP_TYPE_CURRENT: u32 = 0;
    const UNICODE_NULL: u16 = 0;

    TRACE!("(%d)\n", nCSIDL_Folder);

    hResult = SHGetFolderPathW(core::ptr::null_mut(), nCSIDL_Folder, core::ptr::null_mut(), SHGFP_TYPE_CURRENT, szPath.as_mut_ptr());
    cchPathLen = wcslen(szPath.as_ptr());
    if !SUCCEEDED(hResult) || cchPathLen == 0 {
        WARN!("SHGetFolderPath() failed with error %lu\n", GetLastError());
        return 0u8;
    }

    StringCbCatW(szPath.as_mut_ptr(), core::mem::size_of_val(&szPath), w!("\\*").as_ptr() as *const u16);

    hFind = FindFirstFileW(szPath.as_ptr(), core::ptr::addr_of_mut!(FoundData));
    if hFind == INVALID_HANDLE_VALUE {
        WARN!("FindFirstFile(%s) failed with error %lu\n", debugstr_w(szPath.as_ptr()), GetLastError());
        return 0u8;
    }

    loop {
        if wcscmp(FoundData.cFileName.as_ptr(), w!(".").as_ptr() as *const u16) == 0 ||
           wcscmp(FoundData.cFileName.as_ptr(), w!("..").as_ptr() as *const u16) == 0 {
            if FindNextFileW(hFind, core::ptr::addr_of_mut!(FoundData)) == 0 {
                break;
            }
            continue;
        }

        if (FoundData.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
            if FindNextFileW(hFind, core::ptr::addr_of_mut!(FoundData)) == 0 {
                break;
            }
            continue;
        }

        szPath[cchPathLen + 1] = UNICODE_NULL;
        StringCbCatW(szPath.as_mut_ptr(), core::mem::size_of_val(&szPath), FoundData.cFileName.as_ptr());

        TRACE!("Executing %s in directory %s\n", debugstr_w(FoundData.cFileName.as_ptr()), debugstr_w(szPath.as_ptr()));

        let mut dwType: u32 = 0;
        if GetBinaryTypeW(szPath.as_ptr(), core::ptr::addr_of_mut!(dwType)) != 0 {
            runCmd(szPath.as_mut_ptr(), core::ptr::null(), 1, 0);
        } else {
            let mut ExecInfo: SHELLEXECUTEINFOW = core::mem::zeroed();
            RtlZeroMemory(core::ptr::addr_of_mut!(ExecInfo) as *mut core::ffi::c_void, core::mem::size_of::<SHELLEXECUTEINFOW>());
            ExecInfo.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
            ExecInfo.lpFile = szPath.as_ptr();
            ShellExecuteExW(core::ptr::addr_of_mut!(ExecInfo));
        }

        if FindNextFileW(hFind, core::ptr::addr_of_mut!(FoundData)) == 0 {
            break;
        }
    }

    FindClose(hFind);
    1u8
}
#[no_mangle]
pub unsafe extern "C" fn ProcessStartupItems(bRunOnce: i32) -> i32 {
    let bNormalBoot = (GetSystemMetrics(SM_CLEANBOOT as i32) == 0) as i32;
    DbgPrint(
        b"[EXPLORER] startup: ProcessStartupItems(runonce=%d) normalboot=%d\n\0".as_ptr()
            as *const i8,
        bRunOnce,
        bNormalBoot,
    );
    let mut gen_path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut res: u32;

    res = GetWindowsDirectoryW(gen_path.as_mut_ptr(), gen_path.len() as u32);
    if res == 0 {
        TRACE!("Couldn't get the windows directory - error %lu\n", GetLastError());
        return 100;
    }

    if SetCurrentDirectoryW(gen_path.as_ptr()) == 0 {
        TRACE!("Cannot set the dir to %ls (%lu)\n", gen_path.as_ptr(), GetLastError());
        return 100;
    }

    res = 1;

    if bRunOnce != 0 {
        if res != 0 && bNormalBoot != 0 {
            ProcessRunOnceEx(HKEY_LOCAL_MACHINE);
        }

        if res != 0 && SHRestricted(REST_NOLOCALMACHINERUNONCE) == 0 {
            res = ProcessRunKeys(HKEY_LOCAL_MACHINE, w!("RunOnce").as_ptr() as LPCWSTR, 1, 1) as u32;
        }

        return (res == 0) as i32;
    }

    if res != 0 && bNormalBoot != 0 && SHRestricted(REST_NOLOCALMACHINERUN) == 0 {
        res = ProcessRunKeys(HKEY_LOCAL_MACHINE, w!("Run").as_ptr() as LPCWSTR, 0, 0) as u32;
    }

    if res != 0 && bNormalBoot != 0 && SHRestricted(REST_NOCURRENTUSERRUNONCE) == 0 {
        res = ProcessRunKeys(HKEY_CURRENT_USER, w!("Run").as_ptr() as LPCWSTR, 0, 0) as u32;
    }

    AutoStartupApplications(CSIDL_COMMON_STARTUP);
    AutoStartupApplications(CSIDL_STARTUP);

    if res != 0 && bNormalBoot != 0 && SHRestricted(REST_NOCURRENTUSERRUNONCE) == 0 {
        res = ProcessRunKeys(HKEY_CURRENT_USER, w!("RunOnce").as_ptr() as LPCWSTR, 1, 0) as u32;
    }

    TRACE!("Operation done\n");

    if res != 0 { 0 } else { 101 }
}
#[no_mangle]
pub unsafe extern "C" fn ReleaseStartupMutex() {
    if !s_hStartupMutex.is_null() {
        ReleaseMutex(s_hStartupMutex);
        CloseHandle(s_hStartupMutex);
        s_hStartupMutex = core::ptr::null_mut();
    }
}

unsafe fn InitializeStartupMutex() -> u8 {
    if bExplorerIsShell == 0 {
        return 0u8;
    }

    if s_hStartupMutex.is_null() {
        s_hStartupMutex = CreateMutexW(core::ptr::null_mut(), 0, w!("ExplorerIsShellMutex").as_ptr() as *const u16);
        if s_hStartupMutex.is_null() {
            return 0u8;
        }
    }

    let dwWait = WaitForSingleObject(s_hStartupMutex, INFINITE);
    if dwWait != WAIT_OBJECT_0 {
        ReleaseStartupMutex();
        return 0u8;
    }
    1u8
}
#[no_mangle]
pub unsafe extern "C" fn DoStartStartupItems(Tray: *mut core::ffi::c_void) -> BOOL {
    if bExplorerIsShell == 0 || InitializeStartupMutex() == 0 {
        DbgPrint(
            b"[EXPLORER] startup: DoStartStartupItems SKIP (is_shell=%d) -- startup items will NOT run\n\0"
                .as_ptr() as *const i8,
            bExplorerIsShell,
        );
        return 0;
    }

    let dwWaitTotal: u32 = 3000;
    let dwTick: u32 = GetTickCount();
    while GetShellWindow().is_null() && GetTickCount().wrapping_sub(dwTick) < dwWaitTotal {
        TrayProcessMessages(Tray as *mut ITrayWindow);
    }

    if GetShellWindow().is_null() {
        DbgPrint(
            b"[EXPLORER] startup: shell window NEVER appeared in %lums -- startup items SKIPPED\n\0"
                .as_ptr() as *const i8,
            dwWaitTotal,
        );
        ReleaseStartupMutex(); // DoFinishStartupItems() is a C++ inline over this
        return 0;
    }
    DbgPrint(
        b"[EXPLORER] startup: shell window ready after %lums\n\0".as_ptr() as *const i8,
        GetTickCount().wrapping_sub(dwTick),
    );

    let mut hSessionKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let hr: HRESULT = SHCreateSessionKey(KEY_WRITE, &mut hSessionKey);
    if SUCCEEDED(hr) {
        let mut dwDisp: u32 = 0;
        let Error: i32 = RegCreateKeyExW(
            hSessionKey,
            w!("StartupHasBeenRun").as_ptr() as LPCWSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE,
            KEY_WRITE,
            core::ptr::null_mut(),
            &mut hKey,
            &mut dwDisp,
        );
        RegCloseKey(hSessionKey);
        RegCloseKey(hKey);
        if Error == ERROR_SUCCESS as i32 && dwDisp == REG_OPENED_EXISTING_KEY {
            DbgPrint(
                b"[EXPLORER] startup: StartupHasBeenRun already set -- SKIP (another explorer ran them)\n\0"
                    .as_ptr() as *const i8,
            );
            return 0;
        }
    }

    1
}
#[no_mangle]
pub unsafe extern "C" fn ProcessRunOnceItems() {
    if bExplorerIsShell != 0 && IsUserAnAdmin() != 0 && InitializeStartupMutex() != 0 {
        ProcessStartupItems(1);
    } else {
        DbgPrint(
            b"[EXPLORER] startup: ProcessRunOnceItems SKIP (is_shell=%d admin=%d)\n\0".as_ptr()
                as *const i8,
            bExplorerIsShell,
            IsUserAnAdmin(),
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright (C) 2002 Andreas Mohr
//  * Copyright (C) 2002 Shachar Shemesh
//  * Copyright (C) 2013 Edijs Kolesnikovics
//  * Copyright (C) 2018 Katayama Hirofumi MZ
//  * Copyright (C) 2021 He Yang
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// /* Based on the Wine "bootup" handler application
//  *
//  * This app handles the various "hooks" windows allows for applications to perform
//  * as part of the bootstrap process. Theses are roughly devided into three types.
//  * Knowledge base articles that explain this are 137367, 179365, 232487 and 232509.
//  * Also, 119941 has some info on grpconv.exe
//  * The operations performed are (by order of execution):
//  *
//  * After log in
//  * - HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\RunOnceEx (synch)
//  * - HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\RunOnce (synch)
//  * - HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Run (asynch)
//  * - HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run (asynch)
//  * - All users Startup folder "%ALLUSERSPROFILE%\Start Menu\Programs\Startup" (asynch, no imp)
//  * - Current user Startup folder "%USERPROFILE%\Start Menu\Programs\Startup" (asynch, no imp)
//  * - HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\RunOnce (asynch)
//  *
//  * None is processed in Safe Mode
//  */
//
// #include "precomp.h"
//
// // For the auto startup process
// static HANDLE s_hStartupMutex = NULL;
//
// #define INVALID_RUNCMD_RETURN -1
// /**
//  * This function runs the specified command in the specified dir.
//  * [in,out] cmdline - the command line to run. The function may change the passed buffer.
//  * [in] dir - the dir to run the command in. If it is NULL, then the current dir is used.
//  * [in] wait - whether to wait for the run program to finish before returning.
//  * [in] minimized - Whether to ask the program to run minimized.
//  *
//  * Returns:
//  * If running the process failed, returns INVALID_RUNCMD_RETURN. Use GetLastError to get the error code.
//  * If wait is FALSE - returns 0 if successful.
//  * If wait is TRUE - returns the program's return value.
//  */
// static int runCmd(LPWSTR cmdline, LPCWSTR dir, BOOL wait, BOOL minimized)
// {
//     STARTUPINFOW si;
//     PROCESS_INFORMATION info;
//     DWORD exit_code = 0;
//
//     memset(&si, 0, sizeof(si));
//     si.cb = sizeof(si);
//     if (minimized)
//     {
//         si.dwFlags = STARTF_USESHOWWINDOW;
//         si.wShowWindow = SW_MINIMIZE;
//     }
//     memset(&info, 0, sizeof(info));
//
//     if (!CreateProcessW(NULL, cmdline, NULL, NULL, FALSE, 0, NULL, dir, &si, &info))
//     {
//         TRACE("Failed to run command (%lu)\n", GetLastError());
//
//         return INVALID_RUNCMD_RETURN;
//     }
//
//     TRACE("Successfully ran command\n");
//
//     if (wait)
//     {
//         HANDLE Handles[] = { info.hProcess };
//         DWORD nCount = _countof(Handles);
//         DWORD dwWait;
//         MSG msg;
//
//         /* wait for the process to exit */
//         for (;;)
//         {
//             /* We need to keep processing messages,
//                otherwise we will hang anything that is trying to send a message to us */
//             dwWait = MsgWaitForMultipleObjects(nCount, Handles, FALSE, INFINITE, QS_ALLINPUT);
//
//             /* WAIT_OBJECT_0 + nCount signals an event in the message queue,
//                so anything other than that means we are done. */
//             if (dwWait != WAIT_OBJECT_0 + nCount)
//             {
//                 if (dwWait >= WAIT_OBJECT_0 && dwWait < WAIT_OBJECT_0 + nCount)
//                     TRACE("Event %u signaled\n", dwWait - WAIT_OBJECT_0);
//                 else
//                     WARN("Return code: %u\n", dwWait);
//                 break;
//             }
//
//             while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE))
//             {
//                 TranslateMessage(&msg);
//                 DispatchMessageW(&msg);
//             }
//         }
//
//         GetExitCodeProcess(info.hProcess, &exit_code);
//     }
//
//     CloseHandle(info.hThread);
//     CloseHandle(info.hProcess);
//
//     return exit_code;
// }
//
//
// /**
//  * Process a "Run" type registry key.
//  * hkRoot is the HKEY from which "Software\Microsoft\Windows\CurrentVersion" is
//  *      opened.
//  * szKeyName is the key holding the actual entries.
//  * bDelete tells whether we should delete each value right before executing it.
//  * bSynchronous tells whether we should wait for the prog to complete before
//  *      going on to the next prog.
//  */
// static BOOL ProcessRunKeys(HKEY hkRoot, LPCWSTR szKeyName, BOOL bDelete,
//         BOOL bSynchronous)
// {
//     HKEY hkWin = NULL, hkRun = NULL;
//     LONG res = ERROR_SUCCESS;
//     DWORD i, cbMaxCmdLine = 0, cchMaxValue = 0;
//     WCHAR *szCmdLine = NULL;
//     WCHAR *szValue = NULL;
//
//     if (hkRoot == HKEY_LOCAL_MACHINE)
//         TRACE("processing %ls entries under HKLM\n", szKeyName);
//     else
//         TRACE("processing %ls entries under HKCU\n", szKeyName);
//
//     res = RegOpenKeyExW(hkRoot,
//                         L"Software\\Microsoft\\Windows\\CurrentVersion",
//                         0,
//                         KEY_READ,
//                         &hkWin);
//     if (res != ERROR_SUCCESS)
//     {
//         TRACE("RegOpenKeyW failed on Software\\Microsoft\\Windows\\CurrentVersion (%ld)\n", res);
//
//         goto end;
//     }
//
//     res = RegOpenKeyExW(hkWin,
//                         szKeyName,
//                         0,
//                         bDelete ? KEY_ALL_ACCESS : KEY_READ,
//                         &hkRun);
//     if (res != ERROR_SUCCESS)
//     {
//         if (res == ERROR_FILE_NOT_FOUND)
//         {
//             TRACE("Key doesn't exist - nothing to be done\n");
//
//             res = ERROR_SUCCESS;
//         }
//         else
//             TRACE("RegOpenKeyExW failed on run key (%ld)\n", res);
//
//         goto end;
//     }
//
//     res = RegQueryInfoKeyW(hkRun,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            &i,
//                            &cchMaxValue,
//                            &cbMaxCmdLine,
//                            NULL,
//                            NULL);
//     if (res != ERROR_SUCCESS)
//     {
//         TRACE("Couldn't query key info (%ld)\n", res);
//
//         goto end;
//     }
//
//     if (i == 0)
//     {
//         TRACE("No commands to execute.\n");
//
//         res = ERROR_SUCCESS;
//         goto end;
//     }
//
//     szCmdLine = (WCHAR*)HeapAlloc(hProcessHeap,
//                           0,
//                           cbMaxCmdLine);
//     if (szCmdLine == NULL)
//     {
//         TRACE("Couldn't allocate memory for the commands to be executed\n");
//
//         res = ERROR_NOT_ENOUGH_MEMORY;
//         goto end;
//     }
//
//     ++cchMaxValue;
//     szValue = (WCHAR*)HeapAlloc(hProcessHeap,
//                         0,
//                         cchMaxValue * sizeof(*szValue));
//     if (szValue == NULL)
//     {
//         TRACE("Couldn't allocate memory for the value names\n");
//
//         res = ERROR_NOT_ENOUGH_MEMORY;
//         goto end;
//     }
//
//     while (i > 0)
//     {
//         WCHAR *szCmdLineExp = NULL;
//         DWORD cchValLength = cchMaxValue, cbDataLength = cbMaxCmdLine;
//         DWORD type;
//
//         --i;
//
//         res = RegEnumValueW(hkRun,
//                             i,
//                             szValue,
//                             &cchValLength,
//                             0,
//                             &type,
//                             (PBYTE)szCmdLine,
//                             &cbDataLength);
//         if (res != ERROR_SUCCESS)
//         {
//             TRACE("Couldn't read in value %lu - %ld\n", i, res);
//
//             continue;
//         }
//
//         /* safe mode - force to run if prefixed with asterisk */
//         if (GetSystemMetrics(SM_CLEANBOOT as i32) && (szValue[0] != L'*')) continue;
//
//         if (bDelete && (res = RegDeleteValueW(hkRun, szValue)) != ERROR_SUCCESS)
//         {
//             TRACE("Couldn't delete value - %lu, %ld. Running command anyways.\n", i, res);
//         }
//
//         if (type != REG_SZ && type != REG_EXPAND_SZ)
//         {
//             TRACE("Incorrect type of value #%lu (%lu)\n", i, type);
//
//             continue;
//         }
//
//         if (type == REG_EXPAND_SZ)
//         {
//             DWORD dwNumOfChars;
//
//             dwNumOfChars = ExpandEnvironmentStringsW(szCmdLine, NULL, 0);
//             if (dwNumOfChars)
//             {
//                 szCmdLineExp = (WCHAR *)HeapAlloc(hProcessHeap, 0, dwNumOfChars * sizeof(*szCmdLineExp));
//
//                 if (szCmdLineExp == NULL)
//                 {
//                     TRACE("Couldn't allocate memory for the commands to be executed\n");
//
//                     res = ERROR_NOT_ENOUGH_MEMORY;
//                     goto end;
//                 }
//
//                 ExpandEnvironmentStringsW(szCmdLine, szCmdLineExp, dwNumOfChars);
//             }
//         }
//
//         res = runCmd(szCmdLineExp ? szCmdLineExp : szCmdLine, NULL, bSynchronous, FALSE);
//         if (res == INVALID_RUNCMD_RETURN)
//         {
//             TRACE("Error running cmd #%lu (%lu)\n", i, GetLastError());
//         }
//
//         if (szCmdLineExp != NULL)
//         {
//             HeapFree(hProcessHeap, 0, szCmdLineExp);
//             szCmdLineExp = NULL;
//         }
//
//         TRACE("Done processing cmd #%lu\n", i);
//     }
//
//     res = ERROR_SUCCESS;
// end:
//     if (szValue != NULL)
//         HeapFree(hProcessHeap, 0, szValue);
//     if (szCmdLine != NULL)
//         HeapFree(hProcessHeap, 0, szCmdLine);
//     if (hkRun != NULL)
//         RegCloseKey(hkRun);
//     if (hkWin != NULL)
//         RegCloseKey(hkWin);
//
//     TRACE("done\n");
//
//     return res == ERROR_SUCCESS ? TRUE : FALSE;
// }
//
// /**
//  * Process "RunOnceEx" type registry key.
//  * rundll32.exe will be invoked if the corresponding key has items inside, and wait for it.
//  * hkRoot is the HKEY from which
//  *      "Software\Microsoft\Windows\CurrentVersion\RunOnceEx"
//  *      is opened.
//  */
// static BOOL ProcessRunOnceEx(HKEY hkRoot)
// {
//     HKEY hkRunOnceEx = NULL;
//     LONG res = ERROR_SUCCESS;
//     WCHAR cmdLine[] = L"rundll32 iernonce.dll RunOnceExProcess";
//     DWORD dwSubKeyCnt;
//
//     res = RegOpenKeyExW(hkRoot,
//                         L"Software\\Microsoft\\Windows\\CurrentVersion\\RunOnceEx",
//                         0,
//                         KEY_READ,
//                         &hkRunOnceEx);
//     if (res != ERROR_SUCCESS)
//     {
//         TRACE("RegOpenKeyW failed on Software\\Microsoft\\Windows\\CurrentVersion\\RunOnceEx (%ld)\n", res);
//         goto end;
//     }
//
//     res = RegQueryInfoKeyW(hkRunOnceEx,
//                            NULL,
//                            NULL,
//                            NULL,
//                            &dwSubKeyCnt,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL,
//                            NULL);
//
//     if (res != ERROR_SUCCESS)
//     {
//         TRACE("RegQueryInfoKeyW failed on Software\\Microsoft\\Windows\\CurrentVersion\\RunOnceEx (%ld)\n", res);
//         goto end;
//     }
//
//     if (dwSubKeyCnt != 0)
//     {
//         if (runCmd(cmdLine, NULL, TRUE, TRUE) == INVALID_RUNCMD_RETURN)
//         {
//             TRACE("runCmd failed (%ld)\n", res = GetLastError());
//             goto end;
//         }
//     }
//
// end:
//     if (hkRunOnceEx != NULL)
//         RegCloseKey(hkRunOnceEx);
//
//     TRACE("done\n");
//
//     return res == ERROR_SUCCESS ? TRUE : FALSE;
// }
//
// static BOOL
// AutoStartupApplications(INT nCSIDL_Folder)
// {
//     WCHAR szPath[MAX_PATH] = { 0 };
//     HRESULT hResult;
//     HANDLE hFind;
//     WIN32_FIND_DATAW FoundData;
//     size_t cchPathLen;
//
//     TRACE("(%d)\n", nCSIDL_Folder);
//
//     // Get the special folder path
//     hResult = SHGetFolderPathW(NULL, nCSIDL_Folder, NULL, SHGFP_TYPE_CURRENT, szPath);
//     cchPathLen = wcslen(szPath);
//     if (!SUCCEEDED(hResult) || cchPathLen == 0)
//     {
//         WARN("SHGetFolderPath() failed with error %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     // Build a path with wildcard
//     StringCbCatW(szPath, sizeof(szPath), L"\\*");
//
//     // Start enumeration of files
//     hFind = FindFirstFileW(szPath, &FoundData);
//     if (hFind == INVALID_HANDLE_VALUE)
//     {
//         WARN("FindFirstFile(%s) failed with error %lu\n", debugstr_w(szPath), GetLastError());
//         return FALSE;
//     }
//
//     // Enumerate the files
//     do
//     {
//         // Ignore "." and ".."
//         if (wcscmp(FoundData.cFileName, L".") == 0 ||
//             wcscmp(FoundData.cFileName, L"..") == 0)
//         {
//             continue;
//         }
//
//         // Don't run hidden files
//         if (FoundData.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN)
//             continue;
//
//         // Build the path
//         szPath[cchPathLen + 1] = UNICODE_NULL;
//         StringCbCatW(szPath, sizeof(szPath), FoundData.cFileName);
//
//         TRACE("Executing %s in directory %s\n", debugstr_w(FoundData.cFileName), debugstr_w(szPath));
//
//         DWORD dwType;
//         if (GetBinaryTypeW(szPath, &dwType))
//         {
//             runCmd(szPath, NULL, TRUE, FALSE);
//         }
//         else
//         {
//             SHELLEXECUTEINFOW ExecInfo;
//             ZeroMemory(&ExecInfo, sizeof(ExecInfo));
//             ExecInfo.cbSize = sizeof(ExecInfo);
//             ExecInfo.lpFile = szPath;
//             ShellExecuteExW(&ExecInfo);
//         }
//     } while (FindNextFileW(hFind, &FoundData));
//
//     FindClose(hFind);
//     return TRUE;
// }
//
// INT ProcessStartupItems(BOOL bRunOnce)
// {
//     /* TODO: ProcessRunKeys already checks SM_CLEANBOOT -- items prefixed with * should probably run even in safe mode */
//     BOOL bNormalBoot = GetSystemMetrics(SM_CLEANBOOT as i32) == 0; /* Perform the operations that are performed every boot */
//     /* First, set the current directory to SystemRoot */
//     WCHAR gen_path[MAX_PATH];
//     DWORD res;
//
//     res = GetWindowsDirectoryW(gen_path, _countof(gen_path));
//     if (res == 0)
//     {
//         TRACE("Couldn't get the windows directory - error %lu\n", GetLastError());
//
//         return 100;
//     }
//
//     if (!SetCurrentDirectoryW(gen_path))
//     {
//         TRACE("Cannot set the dir to %ls (%lu)\n", gen_path, GetLastError());
//
//         return 100;
//     }
//
//     /* Perform the operations by order checking if policy allows it, checking if this is not Safe Mode,
//      * stopping if one fails, skipping if necessary.
//      */
//     res = TRUE;
//
//     if (bRunOnce)
//     {
//         if (res && bNormalBoot)
//             ProcessRunOnceEx(HKEY_LOCAL_MACHINE);
//
//         if (res && (SHRestricted(REST_NOLOCALMACHINERUNONCE) == 0))
//             res = ProcessRunKeys(HKEY_LOCAL_MACHINE, L"RunOnce", TRUE, TRUE);
//
//         return !res;
//     }
//
//     if (res && bNormalBoot && (SHRestricted(REST_NOLOCALMACHINERUN) == 0))
//         res = ProcessRunKeys(HKEY_LOCAL_MACHINE, L"Run", FALSE, FALSE);
//
//     if (res && bNormalBoot && (SHRestricted(REST_NOCURRENTUSERRUNONCE) == 0))
//         res = ProcessRunKeys(HKEY_CURRENT_USER, L"Run", FALSE, FALSE);
//
//     /* All users Startup folder */
//     AutoStartupApplications(CSIDL_COMMON_STARTUP);
//
//     /* Current user Startup folder */
//     AutoStartupApplications(CSIDL_STARTUP);
//
//     /* TODO: HKCU\RunOnce runs even if StartupHasBeenRun exists */
//     if (res && bNormalBoot && (SHRestricted(REST_NOCURRENTUSERRUNONCE) == 0))
//         res = ProcessRunKeys(HKEY_CURRENT_USER, L"RunOnce", TRUE, FALSE);
//
//     TRACE("Operation done\n");
//
//     return res ? 0 : 101;
// }
//
// VOID ReleaseStartupMutex()
// {
//     if (s_hStartupMutex)
//     {
//         ReleaseMutex(s_hStartupMutex);
//         CloseHandle(s_hStartupMutex);
//         s_hStartupMutex = NULL;
//     }
// }
//
// static BOOL InitializeStartupMutex()
// {
//     if (!bExplorerIsShell)
//         return FALSE;
//
//     if (!s_hStartupMutex)
//     {
//         // Accidentally, there is possibility that the system starts multiple Explorers
//         // before startup of shell. We use a mutex to match the timing of shell initialization.
//         s_hStartupMutex = CreateMutexW(NULL, FALSE, L"ExplorerIsShellMutex");
//         if (s_hStartupMutex == NULL)
//             return FALSE;
//     }
//
//     DWORD dwWait = WaitForSingleObject(s_hStartupMutex, INFINITE);
//     TRACE("dwWait: 0x%08lX\n", dwWait);
//     if (dwWait != WAIT_OBJECT_0)
//     {
//         TRACE("LastError: %ld\n", GetLastError());
//
//         ReleaseStartupMutex();
//         return FALSE;
//     }
//     return TRUE;
// }
//
// BOOL DoStartStartupItems(ITrayWindow *Tray)
// {
//     if (!bExplorerIsShell || !InitializeStartupMutex())
//         return FALSE;
//
//     const DWORD dwWaitTotal = 3000;     // in milliseconds
//     DWORD dwTick = GetTickCount();
//     while (GetShellWindow() == NULL && GetTickCount() - dwTick < dwWaitTotal)
//     {
//         TrayProcessMessages(Tray as *mut ITrayWindow);
//     }
//
//     if (GetShellWindow() == NULL)
//     {
//         ReleaseStartupMutex(); // DoFinishStartupItems() is a C++ inline over this
//         return FALSE;
//     }
//
//     // Check the volatile "StartupHasBeenRun" key
//     HKEY hSessionKey, hKey;
//     HRESULT hr = SHCreateSessionKey(KEY_WRITE, &hSessionKey);
//     if (SUCCEEDED(hr))
//     {
//         ASSERT(hSessionKey);
//
//         DWORD dwDisp;
//         LONG Error = RegCreateKeyExW(hSessionKey, L"StartupHasBeenRun", 0, NULL,
//                                      REG_OPTION_VOLATILE, KEY_WRITE, NULL, &hKey, &dwDisp);
//         RegCloseKey(hSessionKey);
//         RegCloseKey(hKey);
//         if (Error == ERROR_SUCCESS && dwDisp == REG_OPENED_EXISTING_KEY)
//         {
//             return FALSE;   // Startup programs has already been run
//         }
//     }
//
//     return TRUE;
// }
//
// VOID ProcessRunOnceItems()
// {
//     if (bExplorerIsShell && IsUserAnAdmin() && InitializeStartupMutex())
//         ProcessStartupItems(TRUE);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
