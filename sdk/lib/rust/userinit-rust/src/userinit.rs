use winsrv_sys::winsrv::*;
use crate::types::*;
use crate::preamble::*;

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


// CRT + ExpandEnvironmentStringsW live in crate::preamble (shared).
// WINAPI (stdcall on i386).
extern "system" {
    fn LookupPrivilegeValueW( lpSystemName: LPCWSTR, lpName: LPCWSTR, lpLuid: *mut LUID, ) -> BOOL;
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HINSTANCE;
    // StringCchCatW (strsafe inline) is reimplemented in crate::preamble.
    // IsLiveCD / RunLiveCD are #[no_mangle] in livecd.rs (cross-TU, as in C).
}
unsafe extern "C" {
    fn IsLiveCD() -> BOOL;
    fn RunLiveCD(State: *mut STATE);
}

// ---- file-scope preamble (translated) ----
pub const CMP_MAGIC: u32 = 0x01234567;

// hInstance now lives in crate::types (shared with livecd.rs).

pub static g_RegColorNames: [[u16; 32]; 31] = [
    [0x53,0x63,0x72,0x6f,0x6c,0x6c,0x62,0x61,0x72,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x61,0x63,0x6b,0x67,0x72,0x6f,0x75,0x6e,0x64,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x41,0x63,0x74,0x69,0x76,0x65,0x54,0x69,0x74,0x6c,0x65,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x49,0x6e,0x61,0x63,0x74,0x69,0x76,0x65,0x54,0x69,0x74,0x6c,0x65,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x4d,0x65,0x6e,0x75,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x57,0x69,0x6e,0x64,0x6f,0x77,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x57,0x69,0x6e,0x64,0x6f,0x77,0x46,0x72,0x61,0x6d,0x65,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x4d,0x65,0x6e,0x75,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x57,0x69,0x6e,0x64,0x6f,0x77,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x54,0x69,0x74,0x6c,0x65,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x41,0x63,0x74,0x69,0x76,0x65,0x42,0x6f,0x72,0x64,0x65,0x72,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x49,0x6e,0x61,0x63,0x74,0x69,0x76,0x65,0x42,0x6f,0x72,0x64,0x65,0x72,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x41,0x70,0x70,0x57,0x6f,0x72,0x6b,0x53,0x70,0x61,0x63,0x65,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x48,0x69,0x6c,0x69,0x67,0x68,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x48,0x69,0x6c,0x69,0x67,0x68,0x74,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x75,0x74,0x74,0x6f,0x6e,0x46,0x61,0x63,0x65,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x75,0x74,0x74,0x6f,0x6e,0x53,0x68,0x61,0x64,0x6f,0x77,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x47,0x72,0x61,0x79,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x75,0x74,0x74,0x6f,0x6e,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x49,0x6e,0x61,0x63,0x74,0x69,0x76,0x65,0x54,0x69,0x74,0x6c,0x65,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x75,0x74,0x74,0x6f,0x6e,0x48,0x69,0x6c,0x69,0x67,0x68,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x75,0x74,0x74,0x6f,0x6e,0x44,0x6b,0x53,0x68,0x61,0x64,0x6f,0x77,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x75,0x74,0x74,0x6f,0x6e,0x4c,0x69,0x67,0x68,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x49,0x6e,0x66,0x6f,0x54,0x65,0x78,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x49,0x6e,0x66,0x6f,0x57,0x69,0x6e,0x64,0x6f,0x77,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x42,0x75,0x74,0x74,0x6f,0x6e,0x41,0x6c,0x74,0x65,0x72,0x6e,0x61,0x74,0x65,0x46,0x61,0x63,0x65,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x48,0x6f,0x74,0x54,0x72,0x61,0x63,0x6b,0x69,0x6e,0x67,0x43,0x6f,0x6c,0x6f,0x72,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x47,0x72,0x61,0x64,0x69,0x65,0x6e,0x74,0x41,0x63,0x74,0x69,0x76,0x65,0x54,0x69,0x74,0x6c,0x65,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x47,0x72,0x61,0x64,0x69,0x65,0x6e,0x74,0x49,0x6e,0x61,0x63,0x74,0x69,0x76,0x65,0x54,0x69,0x74,0x6c,0x65,0,0,0,0,0,0,0,0,0,0,0],
    [0x4d,0x65,0x6e,0x75,0x48,0x69,0x6c,0x69,0x67,0x68,0x74,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0x4d,0x65,0x6e,0x75,0x42,0x61,0x72,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
];

pub type PCMP_REPORT_LOGON = Option<unsafe extern "system" fn(u32, u32) -> u32>;

#[no_mangle]
pub unsafe extern "C" fn ReadRegSzKey(
    hKey: HKEY,
    pszKey: LPCWSTR,
    pValue: *mut LPWSTR,
) -> LONG {
    let mut rc: LONG;
    let mut dwType: DWORD = 0;
    let mut cbData: DWORD = 0;
    let Value: LPWSTR;

    rc = RegQueryValueExW(
        hKey,
        pszKey,
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut cbData,
    );
    if rc != ERROR_SUCCESS as LONG {
        return rc;
    }
    if dwType != REG_SZ {
        return ERROR_FILE_NOT_FOUND as LONG;
    }
    Value = HeapAlloc(
        GetProcessHeap(),
        0,
        (cbData as usize + core::mem::size_of::<u16>()) as _,
    ) as LPWSTR;
    if Value.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }
    rc = RegQueryValueExW(
        hKey,
        pszKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Value as LPBYTE,
        &mut cbData,
    );
    if rc != ERROR_SUCCESS as LONG {
        HeapFree(GetProcessHeap(), 0, Value as _);
        return rc;
    }
    *Value.add(cbData as usize / core::mem::size_of::<u16>()) = 0u16;

    *pValue = Value;
    ERROR_SUCCESS as LONG
}
unsafe fn IsConsoleShell() -> BOOL {
    let mut ControlKey: HKEY = core::ptr::null_mut();
    let mut SystemStartOptions: LPWSTR = core::ptr::null_mut();
    let mut CurrentOption: LPWSTR;
    let mut NextOption: LPWSTR;
    let mut rc: LONG;
    let mut ret: BOOL = 0;

    'cleanup: {
        rc = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            REGSTR_PATH_CURRENT_CONTROL_SET(),
            0,
            KEY_QUERY_VALUE,
            &mut ControlKey,
        );
        if rc != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }

        rc = ReadRegSzKey(
            ControlKey,
            w!("SystemStartOptions").as_ptr() as LPCWSTR,
            &mut SystemStartOptions,
        );
        if rc != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }

        CurrentOption = SystemStartOptions;
        while !CurrentOption.is_null() {
            NextOption = wcschr(CurrentOption, ' ' as u16);
            if !NextOption.is_null() {
                *NextOption = 0;
            }
            if _wcsicmp(
                CurrentOption,
                w!("CONSOLE").as_ptr() as LPCWSTR,
            ) == 0
            {
                ret = 1;
                break 'cleanup;
            }
            CurrentOption = if !NextOption.is_null() {
                NextOption.add(1)
            } else {
                core::ptr::null_mut()
            };
        }
    }

    if !ControlKey.is_null() {
        RegCloseKey(ControlKey);
    }
    HeapFree(GetProcessHeap(), 0, SystemStartOptions as *mut _);

    ret
}

unsafe fn GetShell(
    CommandLine: *mut WCHAR,
    hRootKey: HKEY,
) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Type: DWORD = 0;
    let mut Size: DWORD;
    let mut Shell: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let ConsoleShell: BOOL = IsConsoleShell();
    let mut rc: LONG;

    let key_path: [WCHAR; 56] = [
        0x53, 0x6f, 0x66, 0x74, 0x77, 0x61, 0x72, 0x65, 0x5c, 0x4d, 0x69, 0x63, 0x72, 0x6f,
        0x73, 0x6f, 0x66, 0x74, 0x5c, 0x57, 0x69, 0x6e, 0x64, 0x6f, 0x77, 0x73, 0x20, 0x4e,
        0x54, 0x5c, 0x43, 0x75, 0x72, 0x72, 0x65, 0x6e, 0x74, 0x56, 0x65, 0x72, 0x73, 0x69,
        0x6f, 0x6e, 0x5c, 0x57, 0x69, 0x6e, 0x6c, 0x6f, 0x67, 0x6f, 0x6e, 0x00, 0x00, 0x00,
    ];

    rc = RegOpenKeyExW(
        hRootKey,
        key_path.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    );
    if rc != ERROR_SUCCESS as LONG {
        return 0;
    }

    let console_shell_name: [WCHAR; 13] = [
        0x43, 0x6f, 0x6e, 0x73, 0x6f, 0x6c, 0x65, 0x53, 0x68, 0x65, 0x6c, 0x6c, 0x00,
    ];
    let shell_name: [WCHAR; 6] = [0x53, 0x68, 0x65, 0x6c, 0x6c, 0x00];

    Size = core::mem::size_of_val(&Shell) as DWORD;
    rc = RegQueryValueExW(
        hKey,
        if ConsoleShell != 0 {
            console_shell_name.as_ptr()
        } else {
            shell_name.as_ptr()
        },
        core::ptr::null_mut(),
        &mut Type,
        Shell.as_mut_ptr() as LPBYTE,
        &mut Size,
    );
    RegCloseKey(hKey);

    if rc != ERROR_SUCCESS as LONG {
        return 0;
    }

    if Type == REG_SZ || Type == REG_EXPAND_SZ {
        wcscpy(CommandLine, Shell.as_ptr());
        1
    } else {
        0
    }
}

unsafe fn StartProcess(CommandLine: LPCWSTR, pEnvironment: LPVOID) -> BOOL {

    let mut si: STARTUPINFO = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut ExpandedCmdLine: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut ProfilePath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut WorkingDir: LPWSTR = core::ptr::null_mut();

    ExpandEnvironmentStringsW(
        CommandLine,
        ExpandedCmdLine.as_mut_ptr(),
        ExpandedCmdLine.len() as u32,
    );

    core::ptr::write_bytes(&mut si as *mut STARTUPINFO, 0, 1);
    si.cb = core::mem::size_of::<STARTUPINFO>() as u32;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_SHOWNORMAL as u16;
    core::ptr::write_bytes(&mut pi as *mut PROCESS_INFORMATION, 0, 1);

    // Start the program in the user's profile directory, as Server 2003's userinit does
    // (a cmd.exe shell opens there). Upstream 2263c9ca0ff "Start shell from profile path".
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut hToken) != 0 {
        let mut PathSize: DWORD = ProfilePath.len() as DWORD;
        if GetUserProfileDirectoryW(hToken, ProfilePath.as_mut_ptr(), &mut PathSize) != 0 {
            WorkingDir = ProfilePath.as_mut_ptr();
        }
        CloseHandle(hToken);
    }

    if CreateProcessW(
        core::ptr::null_mut(),
        ExpandedCmdLine.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        NORMAL_PRIORITY_CLASS | CREATE_UNICODE_ENVIRONMENT,
        pEnvironment,
        WorkingDir,
        &mut si,
        &mut pi,
    ) == 0
    {
        return 0;
    }

    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    1
}

unsafe fn StartShell(pEnvironment: LPVOID) -> BOOL {

    let mut Shell: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szMsg: [WCHAR; RC_STRING_MAX_SIZE as usize] = [0; RC_STRING_MAX_SIZE as usize];
    let mut Type: DWORD = 0;
    let mut Size: DWORD;
    let mut Value: DWORD = 0;
    let mut rc: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();

    let safeboot_option: [WCHAR; 60] = [
        0x53, 0x59, 0x53, 0x54, 0x45, 0x4d, 0x5c, 0x43, 0x75, 0x72, 0x72, 0x65, 0x6e, 0x74,
        0x43, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c, 0x53, 0x65, 0x74, 0x5c, 0x43, 0x6f, 0x6e,
        0x74, 0x72, 0x6f, 0x6c, 0x5c, 0x53, 0x61, 0x66, 0x65, 0x42, 0x6f, 0x6f, 0x74, 0x5c,
        0x4f, 0x70, 0x74, 0x69, 0x6f, 0x6e, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let use_alt_shell: [WCHAR; 18] = [
        0x55, 0x73, 0x65, 0x41, 0x6c, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x74, 0x65, 0x53, 0x68,
        0x65, 0x6c, 0x6c, 0x00,
    ];
    let safeboot: [WCHAR; 53] = [
        0x53, 0x59, 0x53, 0x54, 0x45, 0x4d, 0x5c, 0x43, 0x75, 0x72, 0x72, 0x65, 0x6e, 0x74,
        0x43, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c, 0x53, 0x65, 0x74, 0x5c, 0x43, 0x6f, 0x6e,
        0x74, 0x72, 0x6f, 0x6c, 0x5c, 0x53, 0x61, 0x66, 0x65, 0x42, 0x6f, 0x6f, 0x74, 0x00,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let alt_shell: [WCHAR; 15] = [
        0x41, 0x6c, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x74, 0x65, 0x53, 0x68, 0x65, 0x6c, 0x6c,
        0x00,
    ];
    let backslash: [WCHAR; 2] = [0x5c, 0x00];
    let cmd_exe: [WCHAR; 8] = [0x63, 0x6d, 0x64, 0x2e, 0x65, 0x78, 0x65, 0x00];
    let explorer_exe: [WCHAR; 13] = [
        0x65, 0x78, 0x70, 0x6c, 0x6f, 0x72, 0x65, 0x72, 0x2e, 0x65, 0x78, 0x65, 0x00,
    ];

    rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        safeboot_option.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    );
    if rc == ERROR_SUCCESS as LONG {
        Size = core::mem::size_of::<DWORD>() as DWORD;
        rc = RegQueryValueExW(
            hKey,
            use_alt_shell.as_ptr(),
            core::ptr::null_mut(),
            &mut Type,
            &mut Value as *mut DWORD as LPBYTE,
            &mut Size,
        );
        RegCloseKey(hKey);

        if rc == ERROR_SUCCESS as LONG {
            if Type == REG_DWORD {
                if Value != 0 {
                    rc = RegOpenKeyExW(
                        HKEY_LOCAL_MACHINE,
                        safeboot.as_ptr(),
                        0,
                        KEY_READ,
                        &mut hKey,
                    );
                    if rc == ERROR_SUCCESS as LONG {
                        Size = core::mem::size_of_val(&Shell) as DWORD;
                        rc = RegQueryValueExW(
                            hKey,
                            alt_shell.as_ptr(),
                            core::ptr::null_mut(),
                            &mut Type,
                            Shell.as_mut_ptr() as LPBYTE,
                            &mut Size,
                        );
                        RegCloseKey(hKey);

                        if rc == ERROR_SUCCESS as LONG {
                            if Type == REG_SZ || Type == REG_EXPAND_SZ {
                                if StartProcess(Shell.as_ptr(), pEnvironment) != 0 {
                                    return 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if GetShell(Shell.as_mut_ptr(), HKEY_CURRENT_USER) != 0
        && StartProcess(Shell.as_ptr(), pEnvironment) != 0
    {
        return 1;
    }

    if GetShell(Shell.as_mut_ptr(), HKEY_LOCAL_MACHINE) != 0
        && StartProcess(Shell.as_ptr(), pEnvironment) != 0
    {
        return 1;
    }

    if IsConsoleShell() != 0 {
        Shell[0] = 0;
        if GetSystemDirectoryW(Shell.as_mut_ptr(), Shell.len() as u32 - 8) != 0 {
            StringCchCatW(Shell.as_mut_ptr(), Shell.len(), backslash.as_ptr());
        }
        StringCchCatW(Shell.as_mut_ptr(), Shell.len(), cmd_exe.as_ptr());
    } else {
        Shell[0] = 0;
        if GetSystemWindowsDirectoryW(Shell.as_mut_ptr(), Shell.len() as u32 - 13) != 0 {
            StringCchCatW(Shell.as_mut_ptr(), Shell.len(), backslash.as_ptr());
        }
        StringCchCatW(Shell.as_mut_ptr(), Shell.len(), explorer_exe.as_ptr());
    }

    if StartProcess(Shell.as_ptr(), pEnvironment) == 0 {
        LoadStringW(
            GetModuleHandleW(core::ptr::null_mut()),
            IDS_SHELL_FAIL as u32,
            szMsg.as_mut_ptr(),
            szMsg.len() as i32,
        );
        MessageBoxW(
            core::ptr::null_mut(),
            szMsg.as_ptr(),
            core::ptr::null_mut(),
            MB_OK,
        );
        return 0;
    }
    1
}

unsafe fn StrToColorref(mut lpszCol: LPWSTR) -> COLORREF {

    let mut rgb: [u8; 3] = [0; 3];

    rgb[0] = wcstoul(lpszCol, core::ptr::addr_of_mut!(lpszCol), 10) as u8;
    rgb[1] = wcstoul(lpszCol, core::ptr::addr_of_mut!(lpszCol), 10) as u8;
    rgb[2] = wcstoul(lpszCol, core::ptr::addr_of_mut!(lpszCol), 10) as u8;
    RGB(rgb[0], rgb[1], rgb[2])
}

unsafe fn SetUserSysColors() {

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut i: core::ffi::c_int;
    let mut szColor: [u16; 25] = [0; 25];
    let mut Type: u32 = 0;
    let mut Size: u32;
    let mut crColor: COLORREF;
    let mut rc: core::ffi::c_long;

    rc = RegOpenKeyExW(HKEY_CURRENT_USER, REGSTR_PATH_COLORS(),
                       0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hKey));
    if rc != ERROR_SUCCESS as core::ffi::c_long {
        return;
    }

    i = 0;
    while (i as usize) < (core::mem::size_of_val(&g_RegColorNames) / core::mem::size_of_val(&g_RegColorNames[0])) {
        Size = core::mem::size_of_val(&szColor) as u32;
        rc = RegQueryValueExW(hKey, g_RegColorNames[i as usize].as_ptr(), core::ptr::null_mut(),
                              core::ptr::addr_of_mut!(Type),
                              szColor.as_mut_ptr() as LPBYTE, core::ptr::addr_of_mut!(Size));
        if rc == ERROR_SUCCESS as core::ffi::c_long && Type == REG_SZ {
            crColor = StrToColorref(szColor.as_mut_ptr());
            SetSysColors(1, core::ptr::addr_of_mut!(i), core::ptr::addr_of_mut!(crColor));
        }

        i += 1;
    }

    RegCloseKey(hKey);
}

unsafe fn SetUserWallpaper() {

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Type: DWORD = 0;
    let mut Size: DWORD;
    let mut szWallpaper: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut rc: LONG;

    let mut RegStrPathDesktop: [u16; 17] = [
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16,
        b'o' as u16, b'l' as u16, b' ' as u16, b'P' as u16, b'a' as u16,
        b'n' as u16, b'e' as u16, b'l' as u16, b'\\' as u16, b'D' as u16,
        b'e' as u16, 0,
    ];
    let _ = &mut RegStrPathDesktop;

    rc = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        REGSTR_PATH_DESKTOP(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    );
    if rc != ERROR_SUCCESS as LONG {
        return;
    }

    Size = core::mem::size_of_val(&szWallpaper) as DWORD;

    let WallpaperName: [u16; 10] = [
        b'W' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'p' as u16,
        b'a' as u16, b'p' as u16, b'e' as u16, b'r' as u16, 0,
    ];

    rc = RegQueryValueExW(
        hKey,
        WallpaperName.as_ptr(),
        core::ptr::null_mut(),
        &mut Type,
        szWallpaper.as_mut_ptr() as LPBYTE,
        &mut Size,
    );
    RegCloseKey(hKey);

    if rc == ERROR_SUCCESS as LONG && Type == REG_SZ {
        ExpandEnvironmentStringsW(
            szWallpaper.as_ptr(),
            szWallpaper.as_mut_ptr(),
            szWallpaper.len() as DWORD,
        );

        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            szWallpaper.as_mut_ptr() as *mut core::ffi::c_void,
            SPIF_SENDCHANGE,
        );
    } else {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            core::ptr::null_mut(),
            SPIF_SENDCHANGE,
        );
    }
}

unsafe fn SetUserSettings() {
    UpdatePerUserSystemParameters(1, 1);
    SetUserSysColors();
    SetUserWallpaper();
}

unsafe fn NotifyLogon() {
    let hModule: HINSTANCE = LoadLibraryW(
        [0x0073u16, 0x0065, 0x0074, 0x0075, 0x0070, 0x0061, 0x0070, 0x0069, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000].as_ptr(),
    );
    if hModule.is_null() {
        return;
    }

    let CMP_Report_LogOn = GetProcAddress(hModule, b"CMP_Report_LogOn\0".as_ptr() as *const i8);
    if let Some(func) = CMP_Report_LogOn {
        let CMP_Report_LogOn: PCMP_REPORT_LOGON =
            Some(core::mem::transmute(func));
        if let Some(f) = CMP_Report_LogOn {
            f(0x01234567u32, GetCurrentProcessId());
        }
    }

    FreeLibrary(hModule);
}
#[no_mangle]
pub unsafe extern "C" fn ExpandInstallerPath(
    lpInstallerName: LPCWSTR,
    lpInstallerPath: LPWSTR,
    PathSize: usize,
) -> i32 {
    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();

    let cchInstallerNameLen: usize = wcslen(lpInstallerName);
    if PathSize < cchInstallerNameLen {
        *lpInstallerPath = 0;
        return 0;
    }

    GetSystemInfo(&mut SystemInfo);

    *lpInstallerPath = 0;
    GetSystemWindowsDirectoryW(lpInstallerPath, (PathSize - cchInstallerNameLen - 1) as u32);
    let ptr = wcschr(lpInstallerPath, b'\\' as u16);
    if !ptr.is_null() {
        let p = ptr.add(1);
        *p = 0;
    } else {
        *lpInstallerPath = 0;
    }

    match SystemInfo.__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture as u32 {
        PROCESSOR_ARCHITECTURE_INTEL => {
            StringCchCatW(lpInstallerPath, PathSize, w!("I386").as_ptr() as LPCWSTR);
        }
        PROCESSOR_ARCHITECTURE_MIPS => {
            let s: [u16; 5] = [b'M' as u16, b'I' as u16, b'P' as u16, b'S' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        PROCESSOR_ARCHITECTURE_ALPHA => {
            let s: [u16; 6] = [b'A' as u16, b'L' as u16, b'P' as u16, b'H' as u16, b'A' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        PROCESSOR_ARCHITECTURE_PPC => {
            let s: [u16; 4] = [b'P' as u16, b'P' as u16, b'C' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        PROCESSOR_ARCHITECTURE_SHX => {
            let s: [u16; 4] = [b'S' as u16, b'H' as u16, b'X' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        PROCESSOR_ARCHITECTURE_ARM => {
            let s: [u16; 4] = [b'A' as u16, b'R' as u16, b'M' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        PROCESSOR_ARCHITECTURE_IA64 => {
            let s: [u16; 5] = [b'I' as u16, b'A' as u16, b'6' as u16, b'4' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        PROCESSOR_ARCHITECTURE_ALPHA64 => {
            let s: [u16; 8] = [b'A' as u16, b'L' as u16, b'P' as u16, b'H' as u16, b'A' as u16, b'6' as u16, b'4' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        PROCESSOR_ARCHITECTURE_AMD64 => {
            let s: [u16; 6] = [b'A' as u16, b'M' as u16, b'D' as u16, b'6' as u16, b'4' as u16, 0];
            StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
        }
        _ => {
            SystemInfo.__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture =
                PROCESSOR_ARCHITECTURE_UNKNOWN as u16;
        }
    }

    if SystemInfo.__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture as u32
        != PROCESSOR_ARCHITECTURE_UNKNOWN
    {
        let s: [u16; 2] = [b'\\' as u16, 0];
        StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
    }
    StringCchCatW(lpInstallerPath, PathSize, lpInstallerName);

    let mut dwAttribs = GetFileAttributesW(lpInstallerPath);
    if dwAttribs != 0xFFFFFFFF && (dwAttribs & FILE_ATTRIBUTE_DIRECTORY) == 0 {
        return 1;
    }

    *lpInstallerPath = 0;
    if GetSystemWindowsDirectoryW(lpInstallerPath, (PathSize - cchInstallerNameLen - 1) as u32) != 0 {
        let s: [u16; 2] = [b'\\' as u16, 0];
        StringCchCatW(lpInstallerPath, PathSize, s.as_ptr());
    }
    StringCchCatW(lpInstallerPath, PathSize, lpInstallerName);

    dwAttribs = GetFileAttributesW(lpInstallerPath);
    if dwAttribs != 0xFFFFFFFF && (dwAttribs & FILE_ATTRIBUTE_DIRECTORY) == 0 {
        return 1;
    }

    *lpInstallerPath = 0;
    0
}

unsafe fn StartInstaller(lpInstallerName: LPCWSTR) -> BOOL {

    let mut Installer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szMsg: [u16; RC_STRING_MAX_SIZE as usize] = [0; RC_STRING_MAX_SIZE as usize];

    if ExpandInstallerPath(lpInstallerName, Installer.as_mut_ptr(), Installer.len()) != 0 {
        if StartProcess(Installer.as_ptr(), core::ptr::null_mut()) != 0 {
            return 1;
        }
    }

    LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()),
        IDS_INSTALLER_FAIL as u32,
        szMsg.as_mut_ptr(),
        szMsg.len() as i32,
    );
    MessageBoxW(
        core::ptr::null_mut(),
        szMsg.as_ptr(),
        core::ptr::null_mut(),
        MB_OK,
    );
    0
}

unsafe fn EnablePrivilege(lpszPrivilegeName: LPCWSTR, bEnablePrivilege: BOOL) -> BOOL {

    let mut Success: BOOL;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut tp: TOKEN_PRIVILEGES = core::mem::zeroed();

    Success = OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES,
        &mut hToken,
    );
    if Success == 0 {
        return Success;
    }

    'Quit: {
        Success = LookupPrivilegeValueW(
            core::ptr::null_mut(),
            lpszPrivilegeName,
            core::ptr::addr_of_mut!(tp.Privileges[0].Luid),
        );
        if Success == 0 {
            break 'Quit;
        }

        tp.PrivilegeCount = 1;
        tp.Privileges[0].Attributes = if bEnablePrivilege != 0 {
            SE_PRIVILEGE_ENABLED
        } else {
            0
        };

        Success = AdjustTokenPrivileges(
            hToken,
            0,
            &mut tp,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    CloseHandle(hToken);
    Success
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInst: HINSTANCE,
    _hPrevInstance: HINSTANCE,
    _lpszCmdLine: LPWSTR,
    _nCmdShow: i32,
) -> i32 {

    let bIsLiveCD: BOOL;
    let mut Success: BOOL = 1;
    let mut State: STATE = core::mem::zeroed();

    hInstance = hInst;

    bIsLiveCD = IsLiveCD();

    loop {
        SetUserSettings();

        if bIsLiveCD != 0 {
            State.NextPage = LOCALEPAGE;
            State.Run = SHELL;
        } else {
            State.NextPage = DONE;
            State.Run = SHELL;
        }

        if State.NextPage != DONE {
            RunLiveCD(&mut State);
        }

        match State.Run {
            SHELL => {
                // LiveCD runs us as LocalSystem with no WinLogon-built environment, so the shell
                // got no COMPUTERNAME / NUMBER_OF_PROCESSORS / PROCESSOR_* (upstream eec8c977083).
                // Build one; on failure fall back to the inherited block, as upstream does.
                let mut pEnvironment: LPVOID = core::ptr::null_mut();
                if bIsLiveCD != 0 && CreateEnvironmentBlock(&mut pEnvironment, core::ptr::null_mut(), 1) == 0 {
                    pEnvironment = core::ptr::null_mut();
                }
                Success = StartShell(pEnvironment);
                if !pEnvironment.is_null() {
                    DestroyEnvironmentBlock(pEnvironment);
                }
                if Success != 0 {
                    NotifyLogon();
                }
            }
            INSTALLER => {
                Success = StartInstaller(
                    [0x0072u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x006f, 0x0073, 0x002e, 0x0065, 0x0078, 0x0065, 0x0000].as_ptr(),
                );
            }
            REBOOT => {
                EnablePrivilege(SE_SHUTDOWN_NAME(), 1);
                ExitWindowsEx(EWX_REBOOT, 0);
                EnablePrivilege(SE_SHUTDOWN_NAME(), 0);
                Success = 1;
            }
            _ => {
                Success = 0;
            }
        }

        if bIsLiveCD != 0 && Success == 0 {
            continue;
        }

        break;
    }

    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS applications
//  *  Copyright (C) 2001, 2002 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Userinit Logon Application
//  * FILE:        base/system/userinit/userinit.c
//  * PROGRAMMERS: Thomas Weidenmueller (w3seek@users.sourceforge.net)
//  *              Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "userinit.h"
//
// #define CMP_MAGIC  0x01234567
//
// /* GLOBALS ******************************************************************/
//
// HINSTANCE hInstance;
//
// /* FUNCTIONS ****************************************************************/
//
// LONG
// ReadRegSzKey(
//     IN HKEY hKey,
//     IN LPCWSTR pszKey,
//     OUT LPWSTR *pValue)
// {
//     LONG rc;
//     DWORD dwType;
//     DWORD cbData = 0;
//     LPWSTR Value;
//
//     rc = RegQueryValueExW(hKey, pszKey, NULL, &dwType, NULL, &cbData);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegQueryValueEx(%s) failed with error %lu\n", debugstr_w(pszKey), rc);
//         return rc;
//     }
//     if (dwType != REG_SZ)
//     {
//         WARN("Wrong registry data type (%u vs %u)\n", dwType, REG_SZ);
//         return ERROR_FILE_NOT_FOUND;
//     }
//     Value = (WCHAR*) HeapAlloc(GetProcessHeap(), 0, cbData + sizeof(WCHAR));
//     if (!Value)
//     {
//         WARN("No memory\n");
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     rc = RegQueryValueExW(hKey, pszKey, NULL, NULL, (LPBYTE)Value, &cbData);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegQueryValueEx(%s) failed with error %lu\n", debugstr_w(pszKey), rc);
//         HeapFree(GetProcessHeap(), 0, Value);
//         return rc;
//     }
//     /* NULL-terminate the string */
//     Value[cbData / sizeof(WCHAR)] = L'\0';
//
//     *pValue = Value;
//     return ERROR_SUCCESS;
// }
//
// static BOOL
// IsConsoleShell(VOID)
// {
//     HKEY ControlKey = NULL;
//     LPWSTR SystemStartOptions = NULL;
//     LPWSTR CurrentOption, NextOption; /* Pointers into SystemStartOptions */
//     LONG rc;
//     BOOL ret = FALSE;
//
//     rc = RegOpenKeyExW(
//         HKEY_LOCAL_MACHINE,
//         REGSTR_PATH_CURRENT_CONTROL_SET(),
//         0,
//         KEY_QUERY_VALUE,
//         &ControlKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegOpenKeyExW() failed with error %lu\n", rc);
//         goto cleanup;
//     }
//
//     rc = ReadRegSzKey(ControlKey, L"SystemStartOptions", &SystemStartOptions);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("ReadRegSzKey() failed with error %lu\n", rc);
//         goto cleanup;
//     }
//
//     /* Check for CONSOLE switch in SystemStartOptions */
//     CurrentOption = SystemStartOptions;
//     while (CurrentOption)
//     {
//         NextOption = wcschr(CurrentOption, L' ');
//         if (NextOption)
//             *NextOption = L'\0';
//         if (_wcsicmp(CurrentOption, L"CONSOLE") == 0)
//         {
//             TRACE("Found 'CONSOLE' boot option\n");
//             ret = TRUE;
//             goto cleanup;
//         }
//         CurrentOption = NextOption ? NextOption + 1 : NULL;
//     }
//
// cleanup:
//     if (ControlKey != NULL)
//         RegCloseKey(ControlKey);
//     HeapFree(GetProcessHeap(), 0, SystemStartOptions);
//
//     TRACE("IsConsoleShell() returning %u\n", ret);
//     return ret;
// }
//
// static BOOL
// GetShell(
//     OUT WCHAR *CommandLine, /* must be at least MAX_PATH long */
//     IN HKEY hRootKey)
// {
//     HKEY hKey;
//     DWORD Type, Size;
//     WCHAR Shell[MAX_PATH];
//     BOOL ConsoleShell = IsConsoleShell();
//     LONG rc;
//
//     rc = RegOpenKeyExW(hRootKey, L"Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
//                        0, KEY_QUERY_VALUE, &hKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegOpenKeyExW() failed with error %lu\n", rc);
//         return FALSE;
//     }
//
//     Size = sizeof(Shell);
//     rc = RegQueryValueExW(hKey,
//                           ConsoleShell ? L"ConsoleShell" : L"Shell",
//                           NULL,
//                           &Type,
//                           (LPBYTE)Shell,
//                           &Size);
//     RegCloseKey(hKey);
//
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegQueryValueEx() failed with error %lu\n", rc);
//         return FALSE;
//     }
//
//     if ((Type == REG_SZ) || (Type == REG_EXPAND_SZ))
//     {
//         TRACE("Found command line %s\n", debugstr_w(Shell));
//         wcscpy(CommandLine, Shell);
//         return TRUE;
//     }
//     else
//     {
//         WARN("Wrong type %lu (expected %u or %u)\n", Type, REG_SZ, REG_EXPAND_SZ);
//         return FALSE;
//     }
// }
//
// static BOOL
// StartProcess(
//     IN LPCWSTR CommandLine)
// {
//     STARTUPINFO si;
//     PROCESS_INFORMATION pi;
//     WCHAR ExpandedCmdLine[MAX_PATH];
//
//     ExpandEnvironmentStringsW(CommandLine, ExpandedCmdLine, ARRAYSIZE(ExpandedCmdLine));
//
//     ZeroMemory(&si, sizeof(si));
//     si.cb = sizeof(si);
//     si.dwFlags = STARTF_USESHOWWINDOW;
//     si.wShowWindow = SW_SHOWNORMAL;
//     ZeroMemory(&pi, sizeof(pi));
//
//     if (!CreateProcessW(NULL,
//                         ExpandedCmdLine,
//                         NULL,
//                         NULL,
//                         FALSE,
//                         NORMAL_PRIORITY_CLASS,
//                         NULL,
//                         NULL,
//                         &si,
//                         &pi))
//     {
//         WARN("CreateProcessW() failed with error %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     CloseHandle(pi.hProcess);
//     CloseHandle(pi.hThread);
//     return TRUE;
// }
//
// static BOOL
// StartShell(VOID)
// {
//     WCHAR Shell[MAX_PATH];
//     WCHAR szMsg[RC_STRING_MAX_SIZE];
//     DWORD Type, Size;
//     DWORD Value = 0;
//     LONG rc;
//     HKEY hKey;
//
//     /* Safe Mode shell run */
//     rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                        L"SYSTEM\\CurrentControlSet\\Control\\SafeBoot\\Option",
//                        0, KEY_QUERY_VALUE, &hKey);
//     if (rc == ERROR_SUCCESS)
//     {
//         Size = sizeof(Value);
//         rc = RegQueryValueExW(hKey, L"UseAlternateShell", NULL,
//                               &Type, (LPBYTE)&Value, &Size);
//         RegCloseKey(hKey);
//
//         if (rc == ERROR_SUCCESS)
//         {
//             if (Type == REG_DWORD)
//             {
//                 if (Value)
//                 {
//                     /* Safe Mode Alternate Shell required */
//                     rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                        L"SYSTEM\\CurrentControlSet\\Control\\SafeBoot",
//                                        0, KEY_READ, &hKey);
//                     if (rc == ERROR_SUCCESS)
//                     {
//                         Size = sizeof(Shell);
//                         rc = RegQueryValueExW(hKey, L"AlternateShell", NULL,
//                                               &Type, (LPBYTE)Shell, &Size);
//                         RegCloseKey(hKey);
//
//                         if (rc == ERROR_SUCCESS)
//                         {
//                             if ((Type == REG_SZ) || (Type == REG_EXPAND_SZ))
//                             {
//                                 TRACE("Key located - %s\n", debugstr_w(Shell));
//
//                                 /* Try to run alternate shell */
//                                 if (StartProcess(Shell))
//                                 {
//                                     TRACE("Alternate shell started (Safe Mode)\n");
//                                     return TRUE;
//                                 }
//                             }
//                             else
//                             {
//                                 WARN("Wrong type %lu (expected %u or %u)\n",
//                                      Type, REG_SZ, REG_EXPAND_SZ);
//                             }
//                         }
//                         else
//                         {
//                             WARN("Alternate shell in Safe Mode required but not specified.\n");
//                         }
//                     }
//                 }
//             }
//             else
//             {
//                 WARN("Wrong type %lu (expected %u)\n", Type, REG_DWORD);
//             }
//         }
//     }
//
//     /* Try to run shell in user key */
//     if (GetShell(Shell, HKEY_CURRENT_USER) && StartProcess(Shell))
//     {
//         TRACE("Started shell from HKEY_CURRENT_USER\n");
//         return TRUE;
//     }
//
//     /* Try to run shell in local machine key */
//     if (GetShell(Shell, HKEY_LOCAL_MACHINE) && StartProcess(Shell))
//     {
//         TRACE("Started shell from HKEY_LOCAL_MACHINE\n");
//         return TRUE;
//     }
//
//     /* Try default shell */
//     if (IsConsoleShell())
//     {
//         *Shell = UNICODE_NULL;
//         if (GetSystemDirectoryW(Shell, ARRAYSIZE(Shell) - 8))
//             StringCchCatW(Shell, ARRAYSIZE(Shell), L"\\");
//         StringCchCatW(Shell, ARRAYSIZE(Shell), L"cmd.exe");
//     }
//     else
//     {
//         *Shell = UNICODE_NULL;
//         if (GetSystemWindowsDirectoryW(Shell, ARRAYSIZE(Shell) - 13))
//             StringCchCatW(Shell, ARRAYSIZE(Shell), L"\\");
//         StringCchCatW(Shell, ARRAYSIZE(Shell), L"explorer.exe");
//     }
//
//     if (!StartProcess(Shell))
//     {
//         WARN("Failed to start default shell '%s'\n", debugstr_w(Shell));
//         LoadStringW(GetModuleHandle(NULL), IDS_SHELL_FAIL, szMsg, ARRAYSIZE(szMsg));
//         MessageBoxW(NULL, szMsg, NULL, MB_OK);
//         return FALSE;
//     }
//     return TRUE;
// }
//
// const WCHAR g_RegColorNames[][32] = {
//     L"Scrollbar",             /* 00 = COLOR_SCROLLBAR */
//     L"Background",            /* 01 = COLOR_DESKTOP */
//     L"ActiveTitle",           /* 02 = COLOR_ACTIVECAPTION  */
//     L"InactiveTitle",         /* 03 = COLOR_INACTIVECAPTION */
//     L"Menu",                  /* 04 = COLOR_MENU */
//     L"Window",                /* 05 = COLOR_WINDOW */
//     L"WindowFrame",           /* 06 = COLOR_WINDOWFRAME */
//     L"MenuText",              /* 07 = COLOR_MENUTEXT */
//     L"WindowText",            /* 08 = COLOR_WINDOWTEXT */
//     L"TitleText",             /* 09 = COLOR_CAPTIONTEXT */
//     L"ActiveBorder",          /* 10 = COLOR_ACTIVEBORDER */
//     L"InactiveBorder",        /* 11 = COLOR_INACTIVEBORDER */
//     L"AppWorkSpace",          /* 12 = COLOR_APPWORKSPACE */
//     L"Hilight",               /* 13 = COLOR_HIGHLIGHT */
//     L"HilightText",           /* 14 = COLOR_HIGHLIGHTTEXT */
//     L"ButtonFace",            /* 15 = COLOR_BTNFACE */
//     L"ButtonShadow",          /* 16 = COLOR_BTNSHADOW */
//     L"GrayText",              /* 17 = COLOR_GRAYTEXT */
//     L"ButtonText",            /* 18 = COLOR_BTNTEXT */
//     L"InactiveTitleText",     /* 19 = COLOR_INACTIVECAPTIONTEXT */
//     L"ButtonHilight",         /* 20 = COLOR_BTNHIGHLIGHT */
//     L"ButtonDkShadow",        /* 21 = COLOR_3DDKSHADOW */
//     L"ButtonLight",           /* 22 = COLOR_3DLIGHT */
//     L"InfoText",              /* 23 = COLOR_INFOTEXT */
//     L"InfoWindow",            /* 24 = COLOR_INFOBK */
//     L"ButtonAlternateFace",   /* 25 = COLOR_ALTERNATEBTNFACE */
//     L"HotTrackingColor",      /* 26 = COLOR_HOTLIGHT */
//     L"GradientActiveTitle",   /* 27 = COLOR_GRADIENTACTIVECAPTION */
//     L"GradientInactiveTitle", /* 28 = COLOR_GRADIENTINACTIVECAPTION */
//     L"MenuHilight",           /* 29 = COLOR_MENUHILIGHT */
//     L"MenuBar"                /* 30 = COLOR_MENUBAR */
// };
//
// static COLORREF
// StrToColorref(
//     IN LPWSTR lpszCol)
// {
//     BYTE rgb[3];
//
//     rgb[0] = (BYTE)wcstoul(lpszCol, &lpszCol, 10);
//     rgb[1] = (BYTE)wcstoul(lpszCol, &lpszCol, 10);
//     rgb[2] = (BYTE)wcstoul(lpszCol, &lpszCol, 10);
//     return RGB(rgb[0], rgb[1], rgb[2]);
// }
//
// static VOID
// SetUserSysColors(VOID)
// {
//     HKEY hKey;
//     INT i;
//     WCHAR szColor[25];
//     DWORD Type, Size;
//     COLORREF crColor;
//     LONG rc;
//
//     rc = RegOpenKeyExW(HKEY_CURRENT_USER, REGSTR_PATH_COLORS(),
//                        0, KEY_QUERY_VALUE, &hKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegOpenKeyExW() failed with error %lu\n", rc);
//         return;
//     }
//
//     for (i = 0; i < ARRAYSIZE(g_RegColorNames); i++)
//     {
//         Size = sizeof(szColor);
//         rc = RegQueryValueExW(hKey, g_RegColorNames[i], NULL, &Type,
//                               (LPBYTE)szColor, &Size);
//         if (rc == ERROR_SUCCESS && Type == REG_SZ)
//         {
//             crColor = StrToColorref(szColor);
//             SetSysColors(1, &i, &crColor);
//         }
//         else
//         {
//             WARN("RegQueryValueEx(%s) failed with error %lu\n",
//                 debugstr_w(g_RegColorNames[i]), rc);
//         }
//     }
//
//     RegCloseKey(hKey);
// }
//
// static VOID
// SetUserWallpaper(VOID)
// {
//     HKEY hKey;
//     DWORD Type, Size;
//     WCHAR szWallpaper[MAX_PATH + 1];
//     LONG rc;
//
//     rc = RegOpenKeyExW(HKEY_CURRENT_USER, REGSTR_PATH_DESKTOP(),
//                        0, KEY_QUERY_VALUE, &hKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegOpenKeyExW() failed with error %lu\n", rc);
//         return;
//     }
//
//     Size = sizeof(szWallpaper);
//     rc = RegQueryValueExW(hKey,
//                           L"Wallpaper",
//                           NULL,
//                           &Type,
//                           (LPBYTE)szWallpaper,
//                           &Size);
//     RegCloseKey(hKey);
//
//     if (rc == ERROR_SUCCESS && Type == REG_SZ)
//     {
//         ExpandEnvironmentStringsW(szWallpaper, szWallpaper, ARRAYSIZE(szWallpaper));
//         TRACE("Using wallpaper %s\n", debugstr_w(szWallpaper));
//
//         /* Load and change the wallpaper */
//         SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, szWallpaper, SPIF_SENDCHANGE);
//     }
//     else
//     {
//         /* Remove the wallpaper */
//         TRACE("No wallpaper set in registry (error %lu)\n", rc);
//         SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, NULL, SPIF_SENDCHANGE);
//     }
// }
//
// static VOID
// SetUserSettings(VOID)
// {
//     UpdatePerUserSystemParameters(1, TRUE);
//     SetUserSysColors();
//     SetUserWallpaper();
// }
//
// typedef DWORD (WINAPI *PCMP_REPORT_LOGON)(DWORD, DWORD);
//
// static VOID
// NotifyLogon(VOID)
// {
//     HINSTANCE hModule;
//     PCMP_REPORT_LOGON CMP_Report_LogOn;
//
//     hModule = LoadLibraryW(L"setupapi.dll");
//     if (!hModule)
//     {
//         WARN("LoadLibrary() failed with error %lu\n", GetLastError());
//         return;
//     }
//
//     CMP_Report_LogOn = (PCMP_REPORT_LOGON)GetProcAddress(hModule, "CMP_Report_LogOn");
//     if (CMP_Report_LogOn)
//         CMP_Report_LogOn(CMP_MAGIC, GetCurrentProcessId());
//     else
//         WARN("GetProcAddress() failed\n");
//
//     FreeLibrary(hModule);
// }
//
// /*
//  * Expands the path for the ReactOS Installer "reactos.exe".
//  * See also base/setup/welcome/welcome.c!ExpandInstallerPath()
//  */
// BOOL
// ExpandInstallerPath(
//     IN LPCWSTR lpInstallerName,
//     OUT LPWSTR lpInstallerPath,
//     IN SIZE_T PathSize)
// {
//     SYSTEM_INFO SystemInfo;
//     SIZE_T cchInstallerNameLen;
//     PWSTR ptr;
//     DWORD dwAttribs;
//
//     cchInstallerNameLen = wcslen(lpInstallerName);
//     if (PathSize < cchInstallerNameLen)
//     {
//         /* The buffer is not large enough to contain the installer file name */
//         *lpInstallerPath = UNICODE_NULL;
//         return FALSE;
//     }
//
//     /*
//      * First, try to find the installer using the default drive, under
//      * the directory whose name corresponds to the currently-running
//      * CPU architecture.
//      */
//     GetSystemInfo(&SystemInfo);
//
//     *lpInstallerPath = UNICODE_NULL;
//     /* Alternatively one can use SharedUserData->NtSystemRoot */
//     GetSystemWindowsDirectoryW(lpInstallerPath, PathSize - cchInstallerNameLen - 1);
//     ptr = wcschr(lpInstallerPath, L'\\');
//     if (ptr)
//         *++ptr = UNICODE_NULL;
//     else
//         *lpInstallerPath = UNICODE_NULL;
//
//     /* Append the corresponding CPU architecture */
//     switch (SystemInfo.wProcessorArchitecture)
//     {
//         case PROCESSOR_ARCHITECTURE_INTEL:
//             StringCchCatW(lpInstallerPath, PathSize, L"I386");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_MIPS:
//             StringCchCatW(lpInstallerPath, PathSize, L"MIPS");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_ALPHA:
//             StringCchCatW(lpInstallerPath, PathSize, L"ALPHA");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_PPC:
//             StringCchCatW(lpInstallerPath, PathSize, L"PPC");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_SHX:
//             StringCchCatW(lpInstallerPath, PathSize, L"SHX");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_ARM:
//             StringCchCatW(lpInstallerPath, PathSize, L"ARM");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_IA64:
//             StringCchCatW(lpInstallerPath, PathSize, L"IA64");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_ALPHA64:
//             StringCchCatW(lpInstallerPath, PathSize, L"ALPHA64");
//             break;
//
//         case PROCESSOR_ARCHITECTURE_AMD64:
//             StringCchCatW(lpInstallerPath, PathSize, L"AMD64");
//             break;
//
//         // case PROCESSOR_ARCHITECTURE_MSIL: /* .NET CPU-independent code */
//         case PROCESSOR_ARCHITECTURE_UNKNOWN:
//         default:
//             WARN("Unknown processor architecture %lu\n", SystemInfo.wProcessorArchitecture);
//             SystemInfo.wProcessorArchitecture = PROCESSOR_ARCHITECTURE_UNKNOWN;
//             break;
//     }
//
//     if (SystemInfo.wProcessorArchitecture != PROCESSOR_ARCHITECTURE_UNKNOWN)
//         StringCchCatW(lpInstallerPath, PathSize, L"\\");
//     StringCchCatW(lpInstallerPath, PathSize, lpInstallerName);
//
//     dwAttribs = GetFileAttributesW(lpInstallerPath);
//     if ((dwAttribs != INVALID_FILE_ATTRIBUTES) &&
//         !(dwAttribs & FILE_ATTRIBUTE_DIRECTORY))
//     {
//         /* We have found the installer */
//         return TRUE;
//     }
//
//     WARN("Couldn't find the installer '%s', trying alternative.\n", debugstr_w(lpInstallerPath));
//
//     /*
//      * We failed. Try to find the installer from either the current
//      * ReactOS installation directory, or from our current directory.
//      */
//     *lpInstallerPath = UNICODE_NULL;
//     /* Alternatively one can use SharedUserData->NtSystemRoot */
//     if (GetSystemWindowsDirectoryW(lpInstallerPath, PathSize - cchInstallerNameLen - 1))
//         StringCchCatW(lpInstallerPath, PathSize, L"\\");
//     StringCchCatW(lpInstallerPath, PathSize, lpInstallerName);
//
//     dwAttribs = GetFileAttributesW(lpInstallerPath);
//     if ((dwAttribs != INVALID_FILE_ATTRIBUTES) &&
//         !(dwAttribs & FILE_ATTRIBUTE_DIRECTORY))
//     {
//         /* We have found the installer */
//         return TRUE;
//     }
//
//     /* Installer not found */
//     ERR("Couldn't find the installer '%s'.\n", debugstr_w(lpInstallerPath));
//     *lpInstallerPath = UNICODE_NULL;
//     return FALSE;
// }
//
// static BOOL
// StartInstaller(IN LPCWSTR lpInstallerName)
// {
//     WCHAR Installer[MAX_PATH];
//     WCHAR szMsg[RC_STRING_MAX_SIZE];
//
//     if (ExpandInstallerPath(lpInstallerName, Installer, ARRAYSIZE(Installer)))
//     {
//         /* We have found the installer */
//         if (StartProcess(Installer))
//             return TRUE;
//     }
//
//     /* We failed. Display an error message and quit. */
//     ERR("Failed to start the installer '%s'.\n", debugstr_w(Installer));
//     LoadStringW(GetModuleHandle(NULL), IDS_INSTALLER_FAIL, szMsg, ARRAYSIZE(szMsg));
//     MessageBoxW(NULL, szMsg, NULL, MB_OK);
//     return FALSE;
// }
//
// /* Used to get the shutdown privilege */
// static BOOL
// EnablePrivilege(LPCWSTR lpszPrivilegeName, BOOL bEnablePrivilege)
// {
//     BOOL   Success;
//     HANDLE hToken;
//     TOKEN_PRIVILEGES tp;
//
//     Success = OpenProcessToken(GetCurrentProcess(),
//                                TOKEN_ADJUST_PRIVILEGES,
//                                &hToken);
//     if (!Success) return Success;
//
//     Success = LookupPrivilegeValueW(NULL,
//                                     lpszPrivilegeName,
//                                     &tp.Privileges[0].Luid);
//     if (!Success) goto Quit;
//
//     tp.PrivilegeCount = 1;
//     tp.Privileges[0].Attributes = (bEnablePrivilege ? SE_PRIVILEGE_ENABLED : 0);
//
//     Success = AdjustTokenPrivileges(hToken, FALSE, &tp, 0, NULL, NULL);
//
// Quit:
//     CloseHandle(hToken);
//     return Success;
// }
//
//
// int WINAPI
// wWinMain(IN HINSTANCE hInst,
//          IN HINSTANCE hPrevInstance,
//          IN LPWSTR lpszCmdLine,
//          IN int nCmdShow)
// {
//     BOOL bIsLiveCD, Success = TRUE;
//     STATE State;
//
//     hInstance = hInst;
//
//     bIsLiveCD = IsLiveCD();
//
// Restart:
//     SetUserSettings();
//
//     if (bIsLiveCD)
//     {
//         State.NextPage = LOCALEPAGE;
//         State.Run = SHELL;
//     }
//     else
//     {
//         State.NextPage = DONE;
//         State.Run = SHELL;
//     }
//
//     if (State.NextPage != DONE) // && bIsLiveCD
//     {
//         RunLiveCD(&State);
//     }
//
//     switch (State.Run)
//     {
//         case SHELL:
//             Success = StartShell();
//             if (Success)
//                 NotifyLogon();
//             break;
//
//         case INSTALLER:
//             Success = StartInstaller(L"reactos.exe");
//             break;
//
//         case REBOOT:
//         {
//             EnablePrivilege(SE_SHUTDOWN_NAME(), TRUE);
//             ExitWindowsEx(EWX_REBOOT, 0);
//             EnablePrivilege(SE_SHUTDOWN_NAME(), FALSE);
//             Success = TRUE;
//             break;
//         }
//
//         default:
//             Success = FALSE;
//             break;
//     }
//
//     /*
//      * In LiveCD mode, go back to the main menu if we failed
//      * to either start the shell or the installer.
//      */
//     if (bIsLiveCD && !Success)
//         goto Restart;
//
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
