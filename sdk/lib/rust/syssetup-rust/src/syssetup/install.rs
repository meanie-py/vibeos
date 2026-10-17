// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut hSysSetupInf: HINF = INVALID_HANDLE_VALUE;
#[no_mangle]
pub static mut AdminInfo: ADMIN_INFO = ADMIN_INFO {
    Name: core::ptr::null_mut(),
    Domain: core::ptr::null_mut(),
    Password: core::ptr::null_mut(),
};

#[repr(C)]
pub struct DLG_DATA {
    pub hLogoBitmap: HBITMAP,
    pub hBarBitmap: HBITMAP,
    pub hWndBarCtrl: HWND,
    pub BarCounter: DWORD,
    pub BarWidth: DWORD,
    pub BarHeight: DWORD,
}

pub type PDLG_DATA = *mut DLG_DATA;
// install.c:53 `FatalError(char *pszFmt, ...)`. The callers below reach it through this variadic
// declaration; the definition after it receives the arguments.
extern "C" {
    fn FatalError(pszFmt: *mut i8, ...);
}

// Stable Rust cannot DEFINE a C-variadic (`c_variadic`, rust-lang#44930), so this RECEIVES
// the varargs the va-slots way (`va_slots_rust`, as shell32's ShellMessageBoxA and the kernel's
// printf family do): on all three targets a variadic call puts its arguments in the same slots
// as a fixed-arity call with that many parameters (i386 cdecl stack; win64 RCX-R9 then stack;
// arm64 x0-x7 then stack, where Windows passes varargs in integer registers), and a `va_list` is
// a plain pointer. The trailing slots are ordinary parameters, staged into an array a `va_list`
// points at. This replaced the `va_start` trampoline in rust_varargs_shim.c (2026-09-25).
#[export_name = "FatalError"]
unsafe extern "C" fn FatalError_slots(
    pszFmt: *mut i8,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) {
    let mut stage = va_slots_rust::stage(&[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
    FatalError_v(pszFmt, stage.as_mut_ptr() as *mut core::ffi::c_void)
}

// The body; `ap` is the va_list itself (a single `char*` on all three targets).

#[no_mangle]
pub unsafe extern "C" fn FatalError_v(pszFmt: *mut i8, ap: *mut core::ffi::c_void) {
    let mut szBuffer: [i8; 512] = [0; 512];
    let ap = ap as *mut i8;

    vsprintf(szBuffer.as_mut_ptr(), pszFmt, ap);

    LogItem!(core::ptr::null_mut(), w!("Failed") as PCWSTR);

    strcat(szBuffer.as_mut_ptr(), b"\nRebooting now!\0".as_ptr() as *const i8);
    MessageBoxA(
        core::ptr::null_mut(),
        szBuffer.as_ptr(),
        b"ReactOS Setup\0".as_ptr() as *const i8,
        (MB_OK) as u32,
    );
}
unsafe extern "C" fn CreateShellLink(
    pszLinkPath: LPCWSTR,
    pszCmd: LPCWSTR,
    pszArg: LPCWSTR,
    pszDir: LPCWSTR,
    pszIconPath: LPCWSTR,
    iIconNr: i32,
    pszComment: LPCWSTR,
) -> HRESULT {
    let mut psl: *mut IShellLinkW = core::ptr::null_mut();
    let mut ppf: *mut IPersistFile = core::ptr::null_mut();

    let mut hr = CoCreateInstance(
        &CLSID_ShellLink,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IShellLink,
        &mut psl as *mut *mut IShellLinkW as *mut LPVOID,
    );

    if hr >= 0 {
        hr = IShellLinkW_SetPath!(psl, pszCmd);

        if !pszArg.is_null() {
            hr = IShellLinkW_SetArguments!(psl, pszArg);
        }

        if !pszDir.is_null() {
            hr = IShellLinkW_SetWorkingDirectory!(psl, pszDir);
        }

        if !pszIconPath.is_null() {
            hr = IShellLinkW_SetIconLocation!(psl, pszIconPath, iIconNr);
        }

        if !pszComment.is_null() {
            hr = IShellLinkW_SetDescription!(psl, pszComment);
        }

        hr = IShellLinkW_QueryInterface!(
            psl,
            &IID_IPersistFile,
            &mut ppf as *mut *mut IPersistFile as *mut LPVOID,
        );

        if hr >= 0 {
            hr = IPersistFile_Save!(ppf, pszLinkPath, (TRUE) as i32);
            IPersistFile_Release!(ppf);
        }

        IShellLinkW_Release!(psl);
    }

    hr
}
unsafe extern "C" fn CreateShortcut(
    pszFolder: LPCWSTR,
    pszName: LPCWSTR,
    pszCommand: LPCWSTR,
    pszDescription: LPCWSTR,
    iIconNr: i32,
    pszWorkingDir: LPCWSTR,
    pszArgs: LPCWSTR,
) -> i32 {
    let mut dwLen: u32;
    let mut Ptr: LPWSTR;
    let mut lpFilePart: LPWSTR = core::ptr::null_mut();
    let mut szPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szWorkingDirBuf: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut pszWorkingDir = pszWorkingDir;

    if pszWorkingDir.is_null() || *pszWorkingDir.offset(0) == 0 {
        if ExpandEnvironmentStringsW(pszCommand, szPath.as_mut_ptr(), ARRAYSIZE!(szPath)) == 0 {
            wcscpy(szPath.as_mut_ptr(), pszCommand);
        }

        dwLen = GetFullPathNameW(
            szPath.as_ptr(),
            ARRAYSIZE!(szWorkingDirBuf),
            szWorkingDirBuf.as_mut_ptr(),
            &mut lpFilePart,
        );
        if dwLen != 0 && dwLen <= ARRAYSIZE!(szWorkingDirBuf) {
            *lpFilePart = 0;
            lpFilePart = lpFilePart.offset(-1);
            if !(lpFilePart.offset_from(szWorkingDirBuf.as_mut_ptr()) == 2
                && *szWorkingDirBuf.as_ptr().offset(1) == ':' as u16
                && *szWorkingDirBuf.as_ptr().offset(2) == '\\' as u16)
            {
                *lpFilePart = 0;
            }
            pszWorkingDir = szWorkingDirBuf.as_ptr();
        }
    }

    if !pszWorkingDir.is_null() && *pszWorkingDir.offset(0) == 0 {
        pszWorkingDir = core::ptr::null();
    }

    wcscpy(szPath.as_mut_ptr(), pszFolder);
    Ptr = PathAddBackslash(szPath.as_mut_ptr());
    wcscpy(Ptr, pszName);

    ((CreateShellLink(
        szPath.as_ptr(),
        pszCommand,
        pszArgs,
        pszWorkingDir,
        if iIconNr != -1 {
            pszCommand
        } else {
            core::ptr::null()
        },
        iIconNr,
        pszDescription,
    )) >= 0) as i32
}
// HAND-WRITTEN (install.c:184, :243, :287). All three were `--skip-fn`'d: the model could not
// get them past the deterministic call-envelope audit — it kept emitting a sealed call marker
// zero or twice — and a retry did not help, so paying for more samples would have been waste.
// Each C `continue` inside a `do { .. } while (SetupFindNextLine(..))` has to advance the
// iterator FIRST, which is what a naive `continue` in a Rust `loop` gets wrong; expressed here
// as a single `loop` whose tail always calls SetupFindNextLine.
unsafe extern "C" fn CreateShortcutsFromSection(
    pItemsData: PITEMSDATA,
    pNotify: PREGISTRATIONNOTIFY,
    hinf: HINF,
    pszSection: LPWSTR,
    pszFolder: LPCWSTR,
) -> BOOL {
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut dwFieldCount: DWORD;
    let mut iIconNr: INT = 0;
    let mut szCommand: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szName: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szDescription: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szDirectory: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szArgs: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    if SetupFindFirstLineW(hinf, pszSection, core::ptr::null(), &mut Context) == 0 {
        return (FALSE) as i32;
    }

    loop {
        // The `continue`s below skip the body, never the iteration step, so everything from
        // here to the tail lives in a block the skips break out of.
        'line: {
            (*pNotify).Progress += 1;

            SendMessageW((*pItemsData).hwndDlg, PM_STEP_START as UINT, 0, pNotify as LPARAM);

            dwFieldCount = SetupGetFieldCount(&mut Context);
            if dwFieldCount < 3 {
                break 'line;
            }

            if SetupGetStringFieldW(&mut Context, 1, szCommand.as_mut_ptr(), ARRAYSIZE!(&szCommand), core::ptr::null_mut()) == 0 {
                break 'line;
            }

            if SetupGetStringFieldW(&mut Context, 2, szName.as_mut_ptr(), ARRAYSIZE!(&szName), core::ptr::null_mut()) == 0 {
                break 'line;
            }

            if SetupGetStringFieldW(&mut Context, 3, szDescription.as_mut_ptr(), ARRAYSIZE!(&szDescription), core::ptr::null_mut()) == 0 {
                break 'line;
            }

            if dwFieldCount < 4 || SetupGetIntField(&mut Context, 4, &mut iIconNr) == 0 {
                iIconNr = -1; /* Special value to indicate no icon */
            }

            if dwFieldCount < 5
                || SetupGetStringFieldW(&mut Context, 5, szDirectory.as_mut_ptr(), ARRAYSIZE!(&szDirectory), core::ptr::null_mut()) == 0
            {
                szDirectory[0] = 0;
            }

            if dwFieldCount < 6
                || SetupGetStringFieldW(&mut Context, 6, szArgs.as_mut_ptr(), ARRAYSIZE!(&szArgs), core::ptr::null_mut()) == 0
            {
                szArgs[0] = 0;
            }

            wcscat(szName.as_mut_ptr(), w!(".lnk"));

            CreateShortcut(
                pszFolder,
                szName.as_ptr(),
                szCommand.as_ptr(),
                szDescription.as_ptr(),
                iIconNr,
                szDirectory.as_ptr(),
                szArgs.as_ptr(),
            );

            SendMessageW((*pItemsData).hwndDlg, PM_STEP_END as UINT, 0, pNotify as LPARAM);
        }

        if SetupFindNextLine(&mut Context, &mut Context) == 0 {
            break;
        }
    }

    (TRUE) as i32
}

unsafe extern "C" fn CreateShortcuts(
    pItemsData: PITEMSDATA,
    pNotify: PREGISTRATIONNOTIFY,
    hinf: HINF,
    szSection: LPCWSTR,
) -> BOOL {
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut szPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szFolder: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szFolderSection: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut csidl: INT = 0;

    CoInitialize(core::ptr::null_mut());

    // Faithful to the C, including the leak: this early return does NOT CoUninitialize.
    if SetupFindFirstLineW(hinf, szSection, core::ptr::null(), &mut Context) == 0 {
        return (FALSE) as i32;
    }

    loop {
        'line: {
            if SetupGetFieldCount(&mut Context) < 2 {
                break 'line;
            }

            if SetupGetStringFieldW(&mut Context, 0, szFolderSection.as_mut_ptr(), ARRAYSIZE!(&szFolderSection), core::ptr::null_mut()) == 0 {
                break 'line;
            }

            if SetupGetIntField(&mut Context, 1, &mut csidl) == 0 {
                break 'line;
            }

            if SetupGetStringFieldW(&mut Context, 2, szFolder.as_mut_ptr(), ARRAYSIZE!(&szFolder), core::ptr::null_mut()) == 0 {
                break 'line;
            }

            if FAILED!(SHGetFolderPathAndSubDirW(
                core::ptr::null_mut(),
                csidl | (CSIDL_FLAG_CREATE) as i32,
                -1isize as HANDLE,
                SHGFP_TYPE_DEFAULT as DWORD,
                szFolder.as_ptr(),
                szPath.as_mut_ptr(),
            )) {
                break 'line;
            }

            CreateShortcutsFromSection(pItemsData, pNotify, hinf, szFolderSection.as_mut_ptr(), szPath.as_ptr());
        }

        if SetupFindNextLine(&mut Context, &mut Context) == 0 {
            break;
        }
    }

    CoUninitialize();

    (TRUE) as i32
}

unsafe extern "C" fn CountShortcuts(hinf: HINF, szSection: LPCWSTR) -> LONG {
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut szFolderSection: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut Steps: LONG = 0;

    // `return FALSE` from a LONG-returning function: the C's own shape, and FALSE is 0, so the
    // value is the same "no steps" this function means. Kept rather than tidied.
    if SetupFindFirstLineW(hinf, szSection, core::ptr::null(), &mut Context) == 0 {
        return FALSE as LONG;
    }

    loop {
        'line: {
            if SetupGetFieldCount(&mut Context) < 2 {
                break 'line;
            }

            if SetupGetStringFieldW(&mut Context, 0, szFolderSection.as_mut_ptr(), ARRAYSIZE!(&szFolderSection), core::ptr::null_mut()) == 0 {
                break 'line;
            }

            Steps += SetupGetLineCountW(hinf, szFolderSection.as_ptr());
        }

        if SetupFindNextLine(&mut Context, &mut Context) == 0 {
            break;
        }
    }

    Steps
}

#[no_mangle]
pub unsafe extern "C" fn InstallStartMenuItems(pItemsData: PITEMSDATA) {
    let mut hShortcutsInf1: HINF = INVALID_HANDLE_VALUE;
    let mut hShortcutsInf2: HINF = INVALID_HANDLE_VALUE;
    let mut Steps: LONG = 0;
    let LastError: DWORD = 0;
    let mut Notify: REGISTRATIONNOTIFY = core::mem::zeroed();

    ZeroMemory!(&mut Notify as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<REGISTRATIONNOTIFY>());

    hShortcutsInf1 = SetupOpenInfFileW(
        w!("shortcuts.inf") as PCWSTR,
        core::ptr::null(),
        (INF_STYLE_WIN4) as u32,
        core::ptr::null_mut()
    );
    if hShortcutsInf1 == INVALID_HANDLE_VALUE {
        // DPRINT removed
        return;
    }

    hShortcutsInf2 = SetupOpenInfFileW(
        w!("rosapps_shortcuts.inf") as PCWSTR,
        core::ptr::null(),
        (INF_STYLE_WIN4) as u32,
        core::ptr::null_mut()
    );

    Steps = CountShortcuts(hShortcutsInf1, w!("ShortcutFolders") as PCWSTR);
    if hShortcutsInf2 != INVALID_HANDLE_VALUE {
        Steps += CountShortcuts(hShortcutsInf2, w!("ShortcutFolders") as PCWSTR);
    }

    SendMessage((*pItemsData).hwndDlg, (PM_ITEM_START) as u32, 1, Steps as LPARAM);

    'done: loop {
        if CreateShortcuts(pItemsData, &mut Notify, hShortcutsInf1, w!("ShortcutFolders") as PCWSTR) == 0 {
            // DPRINT removed
            break 'done;
        }

        if hShortcutsInf2 != INVALID_HANDLE_VALUE {
            if CreateShortcuts(pItemsData, &mut Notify, hShortcutsInf2, w!("ShortcutFolders") as PCWSTR) == 0 {
                // DPRINT removed
                break 'done;
            }
        }
        break 'done;
    }

    if hShortcutsInf2 != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hShortcutsInf2);
    }

    if hShortcutsInf1 != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hShortcutsInf1);
    }

    SendMessage((*pItemsData).hwndDlg, (PM_ITEM_END) as u32, 1, LastError as LPARAM);
}
unsafe extern "C" fn CreateTempDir(VarName: LPCWSTR) {
    let mut szTempDir: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szBuffer: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut dwLength: u32;
    let mut hKey: HKEY = core::ptr::null_mut();

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment") as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) != (ERROR_SUCCESS) as i32
    {
        FatalError(b"Error: %lu\n\0".as_ptr() as *mut i8, GetLastError());
        return;
    }

    dwLength = core::mem::size_of_val(&szBuffer) as u32;
    if RegQueryValueExW(
        hKey,
        VarName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        szBuffer.as_mut_ptr() as LPBYTE,
        &mut dwLength,
    ) != (ERROR_SUCCESS) as i32
    {
        FatalError(b"Error: %lu\n\0".as_ptr() as *mut i8, GetLastError());
        RegCloseKey(hKey);
        return;
    }

    if ExpandEnvironmentStringsW(
        szBuffer.as_ptr(),
        szTempDir.as_mut_ptr(),
        ARRAYSIZE!(szTempDir),
    ) == 0
    {
        FatalError(b"Error: %lu\n\0".as_ptr() as *mut i8, GetLastError());
        RegCloseKey(hKey);
        return;
    }

    if CreateDirectoryW(szTempDir.as_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            FatalError(b"Error: %lu\n\0".as_ptr() as *mut i8, GetLastError());
            RegCloseKey(hKey);
            return;
        }
    }

    RegCloseKey(hKey);
}
unsafe extern "C" fn InstallSysSetupInfDevices() -> BOOL {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut szLineBuffer: [u16; 256] = [0; 256];
    let mut dwLineLength: u32 = 0;

    if SetupFindFirstLineW(
        hSysSetupInf,
        w!("DeviceInfsToInstall") as PCWSTR,
        core::ptr::null_mut(),
        &mut InfContext,
    ) == (FALSE) as i32
    {
        return (FALSE) as i32;
    }

    loop {
        if SetupGetStringFieldW(
            &mut InfContext,
            0,
            szLineBuffer.as_mut_ptr(),
            ARRAYSIZE!(szLineBuffer),
            &mut dwLineLength,
        ) == (FALSE) as i32
        {
            return (FALSE) as i32;
        }

        if SetupDiInstallClassW(
            core::ptr::null_mut(),
            szLineBuffer.as_ptr(),
            (DI_QUIETINSTALL) as u32,
            core::ptr::null_mut(),
        ) == (FALSE) as i32
        {
            return (FALSE) as i32;
        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }

    (TRUE) as i32
}
unsafe extern "C" fn InstallSysSetupInfComponents() -> u32 {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut szNameBuffer: [u16; 256] = [0; 256];
    let mut szSectionBuffer: [u16; 256] = [0; 256];
    let mut hComponentInf: HINF = INVALID_HANDLE_VALUE;

    if SetupFindFirstLineW(
        hSysSetupInf,
        w!("Infs.Always") as PCWSTR,
        core::ptr::null_mut(),
        &mut InfContext
    ) == 0
    {
        // DPRINT removed
    }
    else
    {
        loop {
            if SetupGetStringFieldW(
                &mut InfContext,
                1,
                szNameBuffer.as_mut_ptr(),
                ARRAYSIZE!(szNameBuffer),
                core::ptr::null_mut()
            ) == 0
            {
                FatalError(b"Error while trying to get component name\n\0".as_ptr() as *mut i8);
                return (FALSE) as u32;
            }

            if SetupGetStringFieldW(
                &mut InfContext,
                2,
                szSectionBuffer.as_mut_ptr(),
                ARRAYSIZE!(szSectionBuffer),
                core::ptr::null_mut()
            ) == 0
            {
                FatalError(b"Error while trying to get component install section\n\0".as_ptr() as *mut i8);
                return (FALSE) as u32;
            }

            // DPRINT removed

            hComponentInf = SetupOpenInfFileW(
                szNameBuffer.as_ptr(),
                core::ptr::null_mut(),
                (INF_STYLE_WIN4) as u32,
                core::ptr::null_mut()
            );

            if hComponentInf == INVALID_HANDLE_VALUE {
                FatalError(
                    b"SetupOpenInfFileW() failed to open '%S' (Error: %lu)\n\0".as_ptr() as *mut i8,
                    szNameBuffer.as_ptr(),
                    GetLastError()
                );
                return (FALSE) as u32;
            }

            if SetupInstallFromInfSectionW(
                core::ptr::null_mut(),
                hComponentInf,
                szSectionBuffer.as_ptr(),
                (SPINST_ALL) as u32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (SP_COPY_NEWER) as u32,
                Some(SetupDefaultQueueCallbackW),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut()
            ) == 0
            {
                FatalError(
                    b"Error while trying to install : %S (Error: %lu)\n\0".as_ptr() as *mut i8,
                    szNameBuffer.as_ptr(),
                    GetLastError()
                );
                SetupCloseInfFile(hComponentInf);
                return (FALSE) as u32;
            }

            SetupCloseInfFile(hComponentInf);

            if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
                break;
            }
        }
    }

    return (TRUE) as u32;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterTypeLibraries(
    pItemsData: PITEMSDATA,
    pNotify: PREGISTRATIONNOTIFY,
    hinf: HINF,
    szSection: PCWSTR,
) -> BOOL {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut res: BOOL;
    let mut szName: [u16; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut szPath: [u16; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut csidl: INT = 0;
    let mut p: LPWSTR;
    let mut hmod: HMODULE;
    let mut hret: HRESULT;

    res = SetupFindFirstLine(hinf, szSection, core::ptr::null_mut(), &mut InfContext);
    if res == 0 {
        return (FALSE) as i32;
    }

    loop {
        // C: `do { ... continue; ... } while (SetupFindNextLine(...))` -- a `continue`
        // still advances to the next INF line. A bare `continue` here re-processed the
        // SAME line forever after any failure (FatalError only shows a message box), so
        // one bad TypeLibraries entry hung setup in an endless dialog loop.
        'next: {
            if SetupGetStringFieldW(
                &mut InfContext,
                1,
                szName.as_mut_ptr(),
                ARRAYSIZE!(szName),
                core::ptr::null_mut(),
            ) == 0
            {
                FatalError(b"SetupGetStringFieldW failed\n\0".as_ptr() as *mut i8);
                break 'next;
            }

            if SetupGetIntField(&mut InfContext, 2, &mut csidl) == 0 {
                csidl = (CSIDL_SYSTEM) as i32;
            }

            hret = SHGetFolderPathW(
                core::ptr::null_mut(),
                csidl,
                core::ptr::null_mut(),
                0,
                szPath.as_mut_ptr(),
            );
            if hret < 0 {
                FatalError(
                    b"SHGetFolderPathW failed hret=0x%lx\n\0".as_ptr() as *mut i8,
                    hret,
                );
                break 'next;
            }

            p = PathAddBackslash(szPath.as_mut_ptr());
            wcscpy(p, szName.as_mut_ptr());

            if !pItemsData.is_null() && !pNotify.is_null() {
                (*pNotify).Progress += 1;
                (*pNotify).CurrentItem = szName.as_mut_ptr();

                // DPRINT removed
                SendMessage(
                    (*pItemsData).hwndDlg,
                    (PM_STEP_START) as u32,
                    0,
                    pNotify as LPARAM,
                );
            }

            hmod = LoadLibraryW(szPath.as_mut_ptr());
            if hmod.is_null() {
                FatalError(b"LoadLibraryW failed\n\0".as_ptr() as *mut i8);
                break 'next;
            }

            __wine_register_resources(hmod);

            if !pItemsData.is_null() && !pNotify.is_null() {
                // DPRINT removed
                SendMessage(
                    (*pItemsData).hwndDlg,
                    (PM_STEP_END) as u32,
                    0,
                    pNotify as LPARAM,
                );
            }

        }

        if SetupFindNextLine(&mut InfContext, &mut InfContext) == 0 {
            break;
        }
    }

    return (TRUE) as i32;
}
unsafe extern "C" fn EnableUserModePnpManager() -> BOOL {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut bRet: BOOL = (FALSE) as i32;
    let mut BytesNeeded: DWORD = 0;
    let mut WaitTime: DWORD = 0;

    'cleanup: loop {
        hSCManager = OpenSCManagerW(core::ptr::null_mut(), core::ptr::null_mut(), (SC_MANAGER_ENUMERATE_SERVICE) as u32);
        if hSCManager.is_null() {
            // DPRINT removed
            // DPRINT removed
            break 'cleanup;
        }

        hService = OpenServiceW(hSCManager,
                                w!("PlugPlay") as LPCWSTR,
                                (SERVICE_CHANGE_CONFIG | SERVICE_START | SERVICE_QUERY_STATUS) as u32);
        if hService.is_null() {
            // DPRINT removed
            break 'cleanup;
        }

        bRet = ChangeServiceConfigW(hService,
                                    (SERVICE_NO_CHANGE) as u32,
                                    SERVICE_AUTO_START,
                                    (SERVICE_NO_CHANGE) as u32,
                                    core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                                    core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        if bRet == 0 {
            // DPRINT removed
            break 'cleanup;
        }

        bRet = StartServiceW(hService, 0, core::ptr::null_mut());
        if bRet == 0 && (GetLastError() != ERROR_SERVICE_ALREADY_RUNNING) {
            // DPRINT removed
            break 'cleanup;
        }

        loop {
            bRet = QueryServiceStatusEx(hService,
                                        SC_STATUS_PROCESS_INFO,
                                        core::ptr::addr_of_mut!(ServiceStatus) as LPBYTE,
                                        core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
                                        core::ptr::addr_of_mut!(BytesNeeded));
            if bRet == 0 {
                // DPRINT removed
                break 'cleanup;
            }

            if ServiceStatus.dwCurrentState != (SERVICE_START_PENDING) as u32 {
                break;
            }

            WaitTime = ServiceStatus.dwWaitHint / 10;
            if WaitTime < 1000 {
                WaitTime = 1000;
            } else if WaitTime > 10000 {
                WaitTime = 10000;
            }
            Sleep(WaitTime);
        }

        if ServiceStatus.dwCurrentState != (SERVICE_RUNNING) as u32 {
            bRet = (FALSE) as i32;
            // DPRINT removed
            break 'cleanup;
        }

        bRet = (TRUE) as i32;
        break 'cleanup;
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }
    if !hSCManager.is_null() {
        CloseServiceHandle(hSCManager);
    }
    bRet
}
unsafe extern "C" fn AdjustStatusMessageWindow(hwndDlg: HWND, pDlgData: *mut DLG_DATA) {
    let mut xOld: i32;
    let mut yOld: i32;
    let mut cxOld: i32;
    let mut cyOld: i32;
    let mut xNew: i32;
    let mut yNew: i32;
    let mut cxNew: i32;
    let mut cyNew: i32;
    let mut cxLabel: i32;
    let mut cyLabel: i32;
    let mut dyLabel: i32;
    let mut rc: RECT = core::mem::zeroed();
    let mut rcBar: RECT = core::mem::zeroed();
    let mut rcLabel: RECT = core::mem::zeroed();
    let mut rcWnd: RECT = core::mem::zeroed();
    let mut bmLogo: BITMAP = core::mem::zeroed();
    let mut bmBar: BITMAP = core::mem::zeroed();
    let mut style: u32;
    let mut exstyle: u32;
    let hwndLogo: HWND = GetDlgItem(hwndDlg, (IDC_ROSLOGO) as i32);
    let hwndBar: HWND = GetDlgItem(hwndDlg, (IDC_BAR) as i32);
    let hwndLabel: HWND = GetDlgItem(hwndDlg, (IDC_STATUSLABEL) as i32);

    match (PRIMARYLANGID!(GetUserDefaultLangID())) as u32 {
        LANG_CHINESE | LANG_JAPANESE | LANG_KOREAN => {}
        _ => return,
    }

    if GetObjectW(
        ((*pDlgData).hLogoBitmap).cast(),
        core::mem::size_of::<BITMAP>() as i32,
        &mut bmLogo as *mut BITMAP as LPVOID,
    ) == 0
        || GetObjectW(
            ((*pDlgData).hBarBitmap).cast(),
            core::mem::size_of::<BITMAP>() as i32,
            &mut bmBar as *mut BITMAP as LPVOID,
        ) == 0
    {
        return;
    }

    GetWindowRect(hwndBar, &mut rcBar);
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDlg,
        &mut rcBar as *mut RECT as LPPOINT,
        2,
    );
    dyLabel = bmLogo.bmHeight - rcBar.top;

    GetWindowRect(hwndLabel, &mut rcLabel);
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDlg,
        &mut rcLabel as *mut RECT as LPPOINT,
        2,
    );
    cxLabel = rcLabel.right - rcLabel.left;
    cyLabel = rcLabel.bottom - rcLabel.top;

    MoveWindow(
        hwndLogo,
        0,
        0,
        bmLogo.bmWidth,
        bmLogo.bmHeight,
        (TRUE) as i32,
    );
    MoveWindow(
        hwndBar,
        0,
        bmLogo.bmHeight,
        bmLogo.bmWidth,
        bmBar.bmHeight,
        (TRUE) as i32,
    );
    MoveWindow(
        hwndLabel,
        rcLabel.left,
        rcLabel.top + dyLabel,
        cxLabel,
        cyLabel,
        (TRUE) as i32,
    );

    GetWindowRect(hwndDlg, &mut rcWnd);
    xOld = rcWnd.left;
    yOld = rcWnd.top;
    cxOld = rcWnd.right - rcWnd.left;
    cyOld = rcWnd.bottom - rcWnd.top;

    GetClientRect(hwndDlg, &mut rc);
    SetRect(
        &mut rc,
        0,
        0,
        bmLogo.bmWidth,
        rc.bottom - rc.top,
    );

    style = GetWindowLongPtrW(hwndDlg, GWL_STYLE) as u32;
    exstyle = GetWindowLongPtrW(hwndDlg, GWL_EXSTYLE) as u32;
    AdjustWindowRectEx(&mut rc, style, (FALSE) as i32, exstyle);

    cxNew = rc.right - rc.left;
    cyNew = (rc.bottom - rc.top) + dyLabel;
    xNew = xOld - (cxNew - cxOld) / 2;
    yNew = yOld - (cyNew - cyOld) / 2;
    MoveWindow(hwndDlg, xNew, yNew, cxNew, cyNew, (TRUE) as i32);
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn StatusMessageWindowProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pDlgData: *mut DLG_DATA;
    UNREFERENCED_PARAMETER!(wParam);

    pDlgData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as *mut DLG_DATA;

    if uMsg != WM_INITDIALOG && pDlgData.is_null() {
        return FALSE;
    }

    match uMsg {
        WM_INITDIALOG => {
            let mut bm: BITMAP = core::mem::zeroed();
            let mut szMsg: [u16; 256] = [0; 256];

            pDlgData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<DLG_DATA>()) as SIZE_T,
            ) as *mut DLG_DATA;
            if !pDlgData.is_null() {
                SetWindowLongPtrW(hwndDlg, (DWLP_USER) as c_int, ((pDlgData)) as LONG_PTR);

                (*pDlgData).hLogoBitmap = LoadImageW(
                    hDllInstance,
                    MAKEINTRESOURCEW!(IDB_REACTOS),
                    IMAGE_BITMAP,
                    0,
                    0,
                    LR_DEFAULTCOLOR,
                ) as HBITMAP;

                (*pDlgData).hBarBitmap = LoadImageW(
                    hDllInstance,
                    MAKEINTRESOURCEW!(IDB_LINE),
                    IMAGE_BITMAP,
                    0,
                    0,
                    LR_DEFAULTCOLOR,
                ) as HBITMAP;
                GetObject(
                    (*pDlgData).hBarBitmap as HGDIOBJ,
                    core::mem::size_of::<BITMAP>() as i32,
                    &mut bm as *mut _ as LPVOID,
                );
                (*pDlgData).BarWidth = bm.bmWidth as u32;
                (*pDlgData).BarHeight = bm.bmHeight as u32;

                if !(*pDlgData).hLogoBitmap.is_null() && !(*pDlgData).hBarBitmap.is_null() {
                    if SetTimer(
                        hwndDlg,
                        IDT_BAR,
                        20,
                        None,
                    ) == 0
                    {
                        // DPRINT removed
                    }

                    (*pDlgData).hWndBarCtrl = GetDlgItem(hwndDlg, (IDC_BAR) as i32);
                }
            }

            if LoadStringW(
                hDllInstance,
                IDS_STATUS_INSTALL_DEV,
                szMsg.as_mut_ptr(),
                (ARRAYSIZE!(szMsg.as_ref())) as i32,
            ) == 0
            {
                return FALSE;
            }
            SetDlgItemTextW(hwndDlg, (IDC_STATUSLABEL) as i32, szMsg.as_ptr());

            AdjustStatusMessageWindow(hwndDlg, pDlgData);
            return TRUE;
        }

        WM_TIMER => {
            if !(*pDlgData).hBarBitmap.is_null() {
                (*pDlgData).BarCounter =
                    ((*pDlgData).BarCounter + 7) % (*pDlgData).BarWidth;
                InvalidateRect(
                    (*pDlgData).hWndBarCtrl,
                    core::ptr::null_mut(),
                    FALSE,
                );
                UpdateWindow((*pDlgData).hWndBarCtrl);
            }
            return TRUE;
        }

        WM_DRAWITEM => {
            let lpDis = lParam as LPDRAWITEMSTRUCT;

            if (*lpDis).CtlID != IDC_BAR {
                return FALSE;
            }

            if !(*pDlgData).hBarBitmap.is_null() {
                let hdcMem: HDC;
                let hOld: HGDIOBJ;
                let off = (*pDlgData).BarCounter;
                let iw = (*pDlgData).BarWidth;
                let ih = (*pDlgData).BarHeight;

                hdcMem = CreateCompatibleDC((*lpDis).hDC);
                hOld = SelectObject(hdcMem, (*pDlgData).hBarBitmap as HGDIOBJ);
                BitBlt(
                    (*lpDis).hDC,
                    off as i32,
                    0,
                    (iw - off) as i32,
                    ih as i32,
                    hdcMem,
                    0,
                    0,
                    SRCCOPY,
                );
                BitBlt(
                    (*lpDis).hDC,
                    0,
                    0,
                    off as i32,
                    ih as i32,
                    hdcMem,
                    (iw - off) as i32,
                    0,
                    SRCCOPY,
                );
                SelectObject(hdcMem, hOld);
                DeleteDC(hdcMem);
                return TRUE;
            }
            return FALSE;
        }

        WM_DESTROY => {
            if !(*pDlgData).hBarBitmap.is_null() {
                KillTimer(hwndDlg, IDT_BAR);
            }

            DeleteObject((*pDlgData).hLogoBitmap as HGDIOBJ);
            DeleteObject((*pDlgData).hBarBitmap as HGDIOBJ);
            HeapFree(
                GetProcessHeap(),
                0,
                pDlgData as LPVOID,
            );
            return TRUE;
        }

        _ => {}
    }
    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn StatusMessageWindowProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pDlgData: *mut DLG_DATA;
    UNREFERENCED_PARAMETER!(wParam);

    pDlgData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as *mut DLG_DATA;

    if uMsg != (WM_INITDIALOG) as u32 && pDlgData.is_null() {
        return (FALSE) as i64;
    }

    match uMsg {
        WM_INITDIALOG => {
            let mut bm: BITMAP = core::mem::zeroed();
            let mut szMsg: [u16; 256] = [0; 256];

            pDlgData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<DLG_DATA>()) as u64,
            ) as *mut DLG_DATA;
            if !pDlgData.is_null() {
                SetWindowLongPtrW(hwndDlg, (DWLP_USER) as c_int, ((pDlgData as isize)) as LONG_PTR);

                (*pDlgData).hLogoBitmap = LoadImageW(
                    hDllInstance,
                    MAKEINTRESOURCEW!(IDB_REACTOS),
                    (IMAGE_BITMAP) as u32,
                    0,
                    0,
                    (LR_DEFAULTCOLOR) as u32,
                ) as HBITMAP;

                (*pDlgData).hBarBitmap = LoadImageW(
                    hDllInstance,
                    MAKEINTRESOURCEW!(IDB_LINE),
                    (IMAGE_BITMAP) as u32,
                    0,
                    0,
                    (LR_DEFAULTCOLOR) as u32,
                ) as HBITMAP;
                GetObject(
                    (*pDlgData).hBarBitmap as HGDIOBJ,
                    core::mem::size_of::<BITMAP>() as i32,
                    &mut bm as *mut _ as LPVOID,
                );
                (*pDlgData).BarWidth = bm.bmWidth as u32;
                (*pDlgData).BarHeight = bm.bmHeight as u32;

                if !(*pDlgData).hLogoBitmap.is_null() && !(*pDlgData).hBarBitmap.is_null() {
                    if SetTimer(
                        hwndDlg,
                        (IDT_BAR) as u64,
                        20,
                        None,
                    ) == 0
                    {
                        // DPRINT removed
                    }

                    (*pDlgData).hWndBarCtrl = GetDlgItem(hwndDlg, (IDC_BAR) as i32);
                }
            }

            if LoadStringW(
                hDllInstance,
                (IDS_STATUS_INSTALL_DEV) as u32,
                szMsg.as_mut_ptr(),
                (ARRAYSIZE!(szMsg.as_ref())) as i32,
            ) == 0
            {
                return (FALSE) as i64;
            }
            SetDlgItemTextW(hwndDlg, (IDC_STATUSLABEL) as i32, szMsg.as_ptr());

            AdjustStatusMessageWindow(hwndDlg, pDlgData);
            return (TRUE) as i64;
        }

        WM_TIMER => {
            if !(*pDlgData).hBarBitmap.is_null() {
                (*pDlgData).BarCounter =
                    ((*pDlgData).BarCounter + 7) % (*pDlgData).BarWidth;
                InvalidateRect(
                    (*pDlgData).hWndBarCtrl,
                    core::ptr::null_mut(),
                    (FALSE) as i32,
                );
                UpdateWindow((*pDlgData).hWndBarCtrl);
            }
            return (TRUE) as i64;
        }

        WM_DRAWITEM => {
            let lpDis = lParam as LPDRAWITEMSTRUCT;

            if (*lpDis).CtlID != (IDC_BAR) as u32 {
                return (FALSE) as i64;
            }

            if !(*pDlgData).hBarBitmap.is_null() {
                let hdcMem: HDC;
                let hOld: HGDIOBJ;
                let off = (*pDlgData).BarCounter;
                let iw = (*pDlgData).BarWidth;
                let ih = (*pDlgData).BarHeight;

                hdcMem = CreateCompatibleDC((*lpDis).hDC);
                hOld = SelectObject(hdcMem, (*pDlgData).hBarBitmap as HGDIOBJ);
                BitBlt(
                    (*lpDis).hDC,
                    off as i32,
                    0,
                    (iw - off) as i32,
                    ih as i32,
                    hdcMem,
                    0,
                    0,
                    (SRCCOPY) as u32,
                );
                BitBlt(
                    (*lpDis).hDC,
                    0,
                    0,
                    off as i32,
                    ih as i32,
                    hdcMem,
                    (iw - off) as i32,
                    0,
                    (SRCCOPY) as u32,
                );
                SelectObject(hdcMem, hOld);
                DeleteDC(hdcMem);
                return (TRUE) as i64;
            }
            return (FALSE) as i64;
        }

        WM_DESTROY => {
            if !(*pDlgData).hBarBitmap.is_null() {
                KillTimer(hwndDlg, (IDT_BAR) as u64);
            }

            DeleteObject((*pDlgData).hLogoBitmap as HGDIOBJ);
            DeleteObject((*pDlgData).hBarBitmap as HGDIOBJ);
            HeapFree(
                GetProcessHeap(),
                0,
                pDlgData as LPVOID,
            );
            return (TRUE) as i64;
        }

        _ => {}
    }
    return (FALSE) as i64;
}
unsafe extern "system" fn ShowStatusMessageThread(
    lpParameter: LPVOID
) -> u32 {
    let mut hWnd: HWND;
    let mut Msg: MSG = core::mem::zeroed();
    UNREFERENCED_PARAMETER!(lpParameter);

    hWnd = CreateDialogParam(hDllInstance,
                             MAKEINTRESOURCE!(IDD_STATUSWINDOW_DLG),
                             GetDesktopWindow(),
                             Some(StatusMessageWindowProc),
                             0 as LPARAM);
    if hWnd.is_null() {
        return 0;
    }

    ShowWindow(hWnd, (SW_SHOW) as i32);

    while GetMessage(&mut Msg, core::ptr::null_mut(), 0, 0) != 0 {
        TranslateMessage(&Msg);
        DispatchMessage(&Msg);
    }

    EndDialog(hWnd, 0);

    return 0;
}
unsafe extern "C" fn ReadRegSzKey(
    hKey: HKEY,
    pszKey: LPCWSTR,
    pValue: *mut LPWSTR,
) -> i32 {
    let mut rc: i32;
    let mut dwType: u32 = 0;
    let mut cbData: u32 = 0;
    let mut pwszValue: LPWSTR;

    if pValue.is_null() {
        return (ERROR_INVALID_PARAMETER) as i32;
    }

    *pValue = core::ptr::null_mut();
    rc = RegQueryValueExW(hKey, pszKey, core::ptr::null_mut(), &mut dwType, core::ptr::null_mut(), &mut cbData);
    if rc != (ERROR_SUCCESS) as i32 {
        return rc;
    }
    if dwType != (REG_SZ) as u32 {
        return (ERROR_FILE_NOT_FOUND) as i32;
    }
    pwszValue = HeapAlloc(GetProcessHeap(), 0, (cbData + core::mem::size_of::<u16>() as u32) as SIZE_T) as LPWSTR;
    if pwszValue.is_null() {
        return (ERROR_NOT_ENOUGH_MEMORY) as i32;
    }
    rc = RegQueryValueExW(hKey, pszKey, core::ptr::null_mut(), core::ptr::null_mut(), pwszValue as LPBYTE, &mut cbData);
    if rc != (ERROR_SUCCESS) as i32 {
        HeapFree(GetProcessHeap(), 0, pwszValue as LPVOID);
        return rc;
    }
    *pwszValue.offset((cbData / core::mem::size_of::<u16>() as u32) as isize) = 0;

    *pValue = pwszValue;
    return (ERROR_SUCCESS) as i32;
}
unsafe extern "C" fn IsConsoleBoot() -> i32 {
    let mut hControlKey: HKEY = core::ptr::null_mut();
    let mut pwszSystemStartOptions: LPWSTR = core::ptr::null_mut();
    let mut pwszCurrentOption: LPWSTR;
    let mut pwszNextOption: LPWSTR;
    let mut bConsoleBoot: i32 = (FALSE) as i32;
    let mut rc: i32;

    rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                       w!("SYSTEM\\CurrentControlSet\\Control") as LPCWSTR,
                       0,
                       KEY_QUERY_VALUE,
                       &mut hControlKey);
    if rc != (ERROR_SUCCESS) as i32 {
        'cleanup: loop {
            if !hControlKey.is_null() {
                RegCloseKey(hControlKey);
            }
            if !pwszSystemStartOptions.is_null() {
                HeapFree(GetProcessHeap(), 0, pwszSystemStartOptions as LPVOID);
            }
            return bConsoleBoot;
        }
    }

    rc = ReadRegSzKey(hControlKey, w!("SystemStartOptions") as LPCWSTR, &mut pwszSystemStartOptions);
    if rc != (ERROR_SUCCESS) as i32 {
        'cleanup: loop {
            if !hControlKey.is_null() {
                RegCloseKey(hControlKey);
            }
            if !pwszSystemStartOptions.is_null() {
                HeapFree(GetProcessHeap(), 0, pwszSystemStartOptions as LPVOID);
            }
            return bConsoleBoot;
        }
    }

    pwszCurrentOption = pwszSystemStartOptions;
    while !pwszCurrentOption.is_null() {
        pwszNextOption = wcschr(pwszCurrentOption, ' ' as u16);
        if !pwszNextOption.is_null() {
            *pwszNextOption = 0u16 as u16;
        }
        if _wcsicmp(pwszCurrentOption, w!("CONSOLE") as LPCWSTR) == 0 {
            // DPRINT removed
            bConsoleBoot = (TRUE) as i32;
            'cleanup: loop {
                if !hControlKey.is_null() {
                    RegCloseKey(hControlKey);
                }
                if !pwszSystemStartOptions.is_null() {
                    HeapFree(GetProcessHeap(), 0, pwszSystemStartOptions as LPVOID);
                }
                return bConsoleBoot;
            }
        }
        pwszCurrentOption = if !pwszNextOption.is_null() { pwszNextOption.offset(1) } else { core::ptr::null_mut() };
    }

    'cleanup: loop {
        if !hControlKey.is_null() {
            RegCloseKey(hControlKey);
        }
        if !pwszSystemStartOptions.is_null() {
            HeapFree(GetProcessHeap(), 0, pwszSystemStartOptions as LPVOID);
        }
        return bConsoleBoot;
    }
}
unsafe extern "C" fn PreprocessUnattend(IsInstall: BOOL) {
    let mut szPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szValue: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut bDefaultThemesOff: BOOL;

    if IsInstall != 0 {
        GetSystemDirectoryW(szPath.as_mut_ptr(), _countof!(szPath));
        wcscat(szPath.as_mut_ptr(), w!("\\$winnt$.inf") as PCWSTR);
    } else {
        GetWindowsDirectoryW(szPath.as_mut_ptr(), _countof!(szPath));
        wcscat(szPath.as_mut_ptr(), w!("\\unattend.inf") as PCWSTR);
    }

    bDefaultThemesOff = (FALSE) as i32;
    if GetPrivateProfileStringW(
        w!("Shell") as PCWSTR,
        w!("DefaultThemesOff") as PCWSTR,
        w!("no") as PCWSTR,
        szValue.as_mut_ptr(),
        _countof!(szValue),
        szPath.as_ptr()
    ) != 0 && *szValue.as_ptr() != 0
    {
        bDefaultThemesOff = if _wcsicmp(
            szValue.as_ptr(),
            w!("yes") as PCWSTR
        ) == 0 { 1 } else { 0 };
    }

    if bDefaultThemesOff == 0 {
        if GetPrivateProfileStringW(
            w!("Shell") as PCWSTR,
            w!("CustomDefaultThemeFile") as PCWSTR,
            core::ptr::null(),
            szValue.as_mut_ptr(),
            _countof!(szValue),
            szPath.as_ptr()
        ) == 0 || *szValue.as_ptr() == 0
        {
            bDefaultThemesOff = (TRUE) as i32;
        }
    }

    EnableVisualTheme(
        core::ptr::null_mut(),
        if bDefaultThemesOff != 0 { core::ptr::null() } else { szValue.as_ptr() }
    );
}
unsafe extern "C" fn CommonInstall() -> BOOL {
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (FALSE) as i32;

    hSysSetupInf = SetupOpenInfFileW(
        w!("syssetup.inf") as PCWSTR,
        core::ptr::null_mut(),
        (INF_STYLE_WIN4) as u32,
        core::ptr::null_mut(),
    );
    if hSysSetupInf == INVALID_HANDLE_VALUE {
        FatalError(
            b"SetupOpenInfFileW() failed to open 'syssetup.inf' (Error: %lu)\n\0".as_ptr() as *mut i8,
            GetLastError(),
        );
        return (FALSE) as i32;
    }

    'Exit: {
        if InstallSysSetupInfDevices() == 0 {
            FatalError(b"InstallSysSetupInfDevices() failed!\n\0".as_ptr() as *mut i8);
            break 'Exit;
        }

        if InstallSysSetupInfComponents() == 0 {
            FatalError(b"InstallSysSetupInfComponents() failed!\n\0".as_ptr() as *mut i8);
            break 'Exit;
        }

        if IsConsoleBoot() == 0 {
            hThread = CreateThread(
                core::ptr::null_mut(),
                0,
                Some(ShowStatusMessageThread),
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
        }

        if EnableUserModePnpManager() == 0 {
            FatalError(b"EnableUserModePnpManager() failed!\n\0".as_ptr() as *mut i8);
            break 'Exit;
        }
        DPRINT1!("[SYSSETUP] CommonInstall: PlugPlay running, waiting for pending installs\n");

        if CMP_WaitNoPendingInstallEvents((INFINITE) as u32) != (WAIT_OBJECT_0) as u32 {
            FatalError(b"CMP_WaitNoPendingInstallEvents() failed!\n\0".as_ptr() as *mut i8);
            break 'Exit;
        }

        bResult = (TRUE) as i32;
    }

    if bResult == (FALSE) as i32 {
        SetupCloseInfFile(hSysSetupInf);
    }

    if hThread != core::ptr::null_mut() {
        PostThreadMessage(
            GetThreadId(hThread),
            (WM_QUIT) as u32,
            0,
            0,
        );
        WaitForSingleObject(hThread, (INFINITE) as u32);
        CloseHandle(hThread);
    }

    bResult
}
/// Logs each DLL `RegistrationPhase2` registers, and its result. The LiveCD path passed no
/// callback (upstream shows no progress there), which left a registration that crashes setup
/// with nothing to say which DLL did it.
unsafe extern "system" fn LiveCdRegistrationProgress(
    _Context: PVOID,
    Notification: UINT,
    Param1: UINT_PTR,
    _Param2: UINT_PTR,
) -> UINT {
    let st = Param1 as *const SP_REGISTER_CONTROL_STATUSW;
    if Notification == SPFILENOTIFY_STARTREGISTRATION && !st.is_null() {
        DPRINT1!("[SYSSETUP] registering %ls\n", (*st).FileName);
    } else if Notification == SPFILENOTIFY_ENDREGISTRATION && !st.is_null() && (*st).FailureCode != 0 {
        DPRINT1!("[SYSSETUP]   %ls failed: code=%lu err=0x%lx\n", (*st).FileName, (*st).FailureCode,
                 (*st).Win32Error);
    }
    FILEOP_DOIT as UINT
}

/// InstallLiveCD's `error:` label. The box promises "press ENTER to reboot", and since
/// upstream d55a77c350e the C keeps that promise; the translation predates it and just
/// returned, leaving a LiveCD that failed to come up with nothing but a dead box.
unsafe fn LiveCdFailed() {
    MessageBoxW(
        core::ptr::null_mut(),
        w!("Failed to load LiveCD! You can shutdown your computer, or press ENTER to reboot.") as LPCWSTR,
        w!("ReactOS LiveCD") as LPCWSTR,
        (MB_OK) as u32,
    );
    // HACK (upstream too): this should be done by the caller of InstallWindowsNt().
    if pSetupEnablePrivilege(SE_SHUTDOWN_NAME!() as PCWSTR, (TRUE) as i32) == 0 {
        // DPRINT1 removed
    }
    ExitWindowsEx((EWX_REBOOT) as u32, 0);
}

unsafe extern "C" fn InstallLiveCD() -> u32 {
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let bRes: i32;

    DPRINT1!("[SYSSETUP] InstallLiveCD: start\n");
    PreprocessUnattend((FALSE) as i32);
    if CommonInstall() == 0 {
        LiveCdFailed();
        return 0;
    }

    DPRINT1!("[SYSSETUP] InstallLiveCD: CommonInstall done\n");
    let mut bRes = InstallNetworkComponent(w!("MS_TCPIP") as PWSTR);
    if bRes == 0 && GetLastError() != ERROR_FILE_NOT_FOUND {
        // DPRINT removed
    } else {
        SetupStartService(w!("Tcpip") as LPCWSTR, (FALSE) as i32);
        SetupStartService(w!("Dhcp") as LPCWSTR, (FALSE) as i32);
        SetupStartService(w!("Dnscache") as LPCWSTR, (FALSE) as i32);
    }
    DPRINT1!("[SYSSETUP] InstallLiveCD: network started, registering components\n");

    {
        if SetupInstallFromInfSectionW(
            core::ptr::null_mut(),
            hSysSetupInf,
            w!("RegistrationPhase2") as LPCWSTR,
            (SPINST_ALL) as u32,
            core::ptr::null_mut(),
            core::ptr::null(),
            0,
            Some(LiveCdRegistrationProgress),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            // DPRINT removed
        }

        RegisterTypeLibraries(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            hSysSetupInf,
            w!("TypeLibraries") as LPCWSTR,
        );
    }

    DPRINT1!("[SYSSETUP] InstallLiveCD: registration done\n");
    super::install_vibeos::mark_registration_done();
    SetupCloseInfFile(hSysSetupInf);

    // userinit is how the LiveCD reaches its shell upstream (the language page, then
    // explorer). vibeos's winlogon starts the shell itself and runs this setup alongside it,
    // so by now a shell is usually up: launching userinit then would start a second one.
    if !GetShellWindow().is_null() {
        DPRINT1!("[SYSSETUP] InstallLiveCD: shell already running, not starting userinit\n");
        return 0;
    }

    ZeroMemory!(
        &mut StartupInfo as *mut STARTUPINFOW as LPVOID,
        core::mem::size_of::<STARTUPINFOW>(),
    );
    StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    bRes = (CreateProcessW(
        w!("userinit.exe") as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as i32,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut StartupInfo,
        &mut ProcessInformation,
    )) as BOOL;
    if bRes == 0 {
        LiveCdFailed();
        return 0;
    }

    CloseHandle(ProcessInformation.hThread);
    CloseHandle(ProcessInformation.hProcess);

    return 0;
}
unsafe extern "C" fn SetSetupType(dwSetupType: u32) -> i32 {
    let mut dwError: u32;
    let mut hKey: HKEY = core::ptr::null_mut();

    dwError = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SYSTEM\\Setup") as PCWSTR,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        return (FALSE) as i32;
    }

    dwError = (RegSetValueExW(
        hKey,
        w!("SetupType") as PCWSTR,
        0,
        (REG_DWORD) as u32,
        &dwSetupType as *const u32 as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    )) as u32;
    RegCloseKey(hKey);
    if dwError != ERROR_SUCCESS {
        return (FALSE) as i32;
    }

    (TRUE) as i32
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn HotkeyThread(Parameter: LPVOID) -> u32 {
    let mut hotkey: ATOM;
    let mut msg: MSG = core::mem::zeroed();

    // DPRINT removed

    hotkey = GlobalAddAtomW(w!("Setup Shift+F10 Hotkey") as PCWSTR);
    if RegisterHotKey(core::ptr::null_mut(), hotkey as i32, MOD_SHIFT, VK_F10) == 0 {
        // DPRINT removed
    }

    while GetMessageW(&mut msg, core::ptr::null_mut(), 0, 0) != 0 {
        if msg.hwnd.is_null() && msg.message == WM_HOTKEY && msg.wParam == hotkey as WPARAM {
            let mut CmdLine: [u16; 8] = [b'c' as u16, b'm' as u16, b'd' as u16, b'.' as u16, b'e' as u16, b'x' as u16, b'e' as u16, 0];
            let mut si: STARTUPINFOW = core::mem::zeroed();
            si.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
            let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

            if CreateProcessW(
                core::ptr::null_mut(),
                CmdLine.as_mut_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                FALSE,
                CREATE_NEW_CONSOLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut si,
                &mut pi,
            ) != 0
            {
                CloseHandle(pi.hProcess);
                CloseHandle(pi.hThread);
            } else {
                // DPRINT removed
            }
        }
    }

    UnregisterHotKey(core::ptr::null_mut(), hotkey as i32);
    GlobalDeleteAtom(hotkey);

    // DPRINT removed
    return 0;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn HotkeyThread(Parameter: LPVOID) -> u32 {
    let mut hotkey: ATOM;
    let mut msg: MSG = core::mem::zeroed();

    // DPRINT removed

    hotkey = GlobalAddAtomW(w!("Setup Shift+F10 Hotkey") as PCWSTR);
    if RegisterHotKey(core::ptr::null_mut(), (hotkey) as i32, MOD_SHIFT, VK_F10) == 0 {
        // DPRINT removed
    }

    while GetMessageW(&mut msg, core::ptr::null_mut(), 0, 0) != 0 {
        if msg.hwnd.is_null() && msg.message == (WM_HOTKEY) as u32 && msg.wParam == (hotkey as usize) as u64 {
            let mut CmdLine: [u16; 8] = [b'c' as u16, b'm' as u16, b'd' as u16, b'.' as u16, b'e' as u16, b'x' as u16, b'e' as u16, 0];
            let mut si: STARTUPINFOW = core::mem::zeroed();
            si.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
            let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

            if CreateProcessW(
                core::ptr::null_mut(),
                CmdLine.as_mut_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (FALSE) as i32,
                (CREATE_NEW_CONSOLE) as u32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut si,
                &mut pi,
            ) != 0
            {
                CloseHandle(pi.hProcess);
                CloseHandle(pi.hThread);
            } else {
                // DPRINT removed
            }
        }
    }

    UnregisterHotKey(core::ptr::null_mut(), (hotkey) as i32);
    GlobalDeleteAtom(hotkey);

    // DPRINT removed
    return 0;
}
unsafe extern "C" fn InitializeProgramFilesDir() -> i32 {
    let mut Error: i32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwLength: u32;
    let mut szProgramFilesDirPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szCommonFilesDirPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szBuffer: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    if LoadStringW(
        hDllInstance,
        (IDS_PROGRAMFILES) as u32,
        szBuffer.as_mut_ptr(),
        (ARRAYSIZE!(szBuffer)) as i32,
    ) == 0
    {
        // DPRINT removed
        return (FALSE) as i32;
    }

    if LoadStringW(
        hDllInstance,
        (IDS_COMMONFILES) as u32,
        szCommonFilesDirPath.as_mut_ptr(),
        (ARRAYSIZE!(szCommonFilesDirPath)) as i32,
    ) == 0
    {
        // DPRINT removed
    }

    if ExpandEnvironmentStringsW(
        szBuffer.as_ptr(),
        szProgramFilesDirPath.as_mut_ptr(),
        ARRAYSIZE!(szProgramFilesDirPath),
    ) == 0
    {
        // DPRINT removed
        return (FALSE) as i32;
    }

    wcscpy(szBuffer.as_mut_ptr(), szProgramFilesDirPath.as_ptr());
    wcscat(szBuffer.as_mut_ptr(), w!("\\") as PCWSTR);
    wcscat(szBuffer.as_mut_ptr(), szCommonFilesDirPath.as_ptr());

    if ExpandEnvironmentStringsW(
        szBuffer.as_ptr(),
        szCommonFilesDirPath.as_mut_ptr(),
        ARRAYSIZE!(szCommonFilesDirPath),
    ) == 0
    {
        // DPRINT removed
    }

    Error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion") as PCWSTR,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    );
    if Error != (ERROR_SUCCESS) as i32 {
        // DPRINT removed
        return (FALSE) as i32;
    }

    dwLength = ((wcslen(szProgramFilesDirPath.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32;
    Error = RegSetValueExW(
        hKey,
        w!("ProgramFilesDir") as PCWSTR,
        0,
        (REG_SZ) as u32,
        szProgramFilesDirPath.as_ptr() as LPBYTE,
        dwLength,
    );
    if Error != (ERROR_SUCCESS) as i32 {
        // DPRINT removed
        RegCloseKey(hKey);
        return (FALSE) as i32;
    }

    dwLength = ((wcslen(szCommonFilesDirPath.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32;
    Error = RegSetValueExW(
        hKey,
        w!("CommonFilesDir") as PCWSTR,
        0,
        (REG_SZ) as u32,
        szCommonFilesDirPath.as_ptr() as LPBYTE,
        dwLength,
    );
    if Error != (ERROR_SUCCESS) as i32 {
        // DPRINT removed
    }

    RegCloseKey(hKey);

    if CreateDirectoryW(szProgramFilesDirPath.as_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            // DPRINT removed
            return (FALSE) as i32;
        }
    }

    if CreateDirectoryW(szCommonFilesDirPath.as_ptr(), core::ptr::null_mut()) == 0 {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            // DPRINT removed
        }
    }

    return (TRUE) as i32;
}
unsafe extern "C" fn InitializeDefaultUserLocale() {
    let mut szBuffer: [u16; 80] = [0; 80];
    let mut ptr: PWSTR = core::ptr::null_mut();
    let mut hLocaleKey: HKEY = core::ptr::null_mut();
    let mut ret: u32;
    let mut dwSize: u32;
    let mut lcid: LCID;
    let mut i: i32;

    struct LocaleDataEntry {
        LCType: u32,
        pValue: PCWSTR,
    }

    let LocaleData: [LocaleDataEntry; 38] = [
        LocaleDataEntry { LCType: (LOCALE_SDECIMAL) as u32, pValue: w!("sDecimal") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_STHOUSAND) as u32, pValue: w!("sThousand") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SNEGATIVESIGN) as u32, pValue: w!("sNegativeSign") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SPOSITIVESIGN) as u32, pValue: w!("sPositiveSign") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SGROUPING) as u32, pValue: w!("sGrouping") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SLIST) as u32, pValue: w!("sList") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SNATIVEDIGITS) as u32, pValue: w!("sNativeDigits") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_INEGNUMBER) as u32, pValue: w!("iNegNumber") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_IDIGITS) as u32, pValue: w!("iDigits") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ILZERO) as u32, pValue: w!("iLZero") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_IMEASURE) as u32, pValue: w!("iMeasure") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_IDIGITSUBSTITUTION) as u32, pValue: w!("NumShape") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SCURRENCY) as u32, pValue: w!("sCurrency") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SMONDECIMALSEP) as u32, pValue: w!("sMonDecimalSep") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SMONTHOUSANDSEP) as u32, pValue: w!("sMonThousandSep") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SMONGROUPING) as u32, pValue: w!("sMonGrouping") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ICURRENCY) as u32, pValue: w!("iCurrency") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_INEGCURR) as u32, pValue: w!("iNegCurr") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ICURRDIGITS) as u32, pValue: w!("iCurrDigits") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_STIMEFORMAT) as u32, pValue: w!("sTimeFormat") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_STIME) as u32, pValue: w!("sTime") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_S1159) as u32, pValue: w!("s1159") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_S2359) as u32, pValue: w!("s2359") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ITIME) as u32, pValue: w!("iTime") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ITIMEMARKPOSN) as u32, pValue: w!("iTimePrefix") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ITLZERO) as u32, pValue: w!("iTLZero") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SLONGDATE) as u32, pValue: w!("sLongDate") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SSHORTDATE) as u32, pValue: w!("sShortDate") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SDATE) as u32, pValue: w!("sDate") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_IFIRSTDAYOFWEEK) as u32, pValue: w!("iFirstDayOfWeek") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_IFIRSTWEEKOFYEAR) as u32, pValue: w!("iFirstWeekOfYear") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_IDATE) as u32, pValue: w!("iDate") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ICALENDARTYPE) as u32, pValue: w!("iCalendarType") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SCOUNTRY) as u32, pValue: w!("sCountry") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_SABBREVLANGNAME) as u32, pValue: w!("sLanguage") as PCWSTR },
        LocaleDataEntry { LCType: (LOCALE_ICOUNTRY) as u32, pValue: w!("iCountry") as PCWSTR },
        LocaleDataEntry { LCType: 0, pValue: core::ptr::null() },
        LocaleDataEntry { LCType: 0, pValue: core::ptr::null() },
    ];

    ret = (RegOpenKeyExW(HKEY_USERS,
                        w!(".DEFAULT\\Control Panel\\International") as PCWSTR,
                        0,
                        KEY_READ | KEY_WRITE,
                        &mut hLocaleKey)) as u32;
    if ret != ERROR_SUCCESS {
        return;
    }

    'done: loop {
        dwSize = 9 * core::mem::size_of::<u16>() as u32;
        ret = (RegQueryValueExW(hLocaleKey,
                               w!("Locale") as PCWSTR,
                               core::ptr::null_mut(),
                               core::ptr::null_mut(),
                               szBuffer.as_mut_ptr() as PBYTE,
                               &mut dwSize)) as u32;
        if ret != ERROR_SUCCESS {
            break 'done;
        }

        lcid = wcstoul(szBuffer.as_ptr(), &mut ptr, 16);
        if lcid == 0 {
            break 'done;
        }

        i = 0;
        while !LocaleData[i as usize].pValue.is_null() {
            if GetLocaleInfoW(lcid,
                               LocaleData[i as usize].LCType | (LOCALE_NOUSEROVERRIDE) as u32,
                               szBuffer.as_mut_ptr(),
                               (ARRAYSIZE!(szBuffer)) as i32) != 0 {
                RegSetValueExW(hLocaleKey,
                               LocaleData[i as usize].pValue,
                               0,
                               (REG_SZ) as u32,
                               szBuffer.as_ptr() as PBYTE,
                               ((wcslen(szBuffer.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32);
            }

            i += 1;
        }

        break 'done;
    }

    RegCloseKey(hLocaleKey);
}
unsafe extern "C" fn SaveDefaultUserHive() -> u32 {
    let mut szDefaultUserHive: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut hUserKey: HKEY = core::ptr::null_mut();
    let mut cchSize: u32;
    let mut dwError: u32;

    // DPRINT removed

    cchSize = ARRAYSIZE!(szDefaultUserHive);
    GetDefaultUserProfileDirectoryW(szDefaultUserHive.as_mut_ptr(), core::ptr::addr_of_mut!(cchSize));

    wcscat(szDefaultUserHive.as_mut_ptr(), w!("\\ntuser.dat") as LPCWSTR);

    dwError = (RegOpenKeyExW(HKEY_USERS,
                            w!(".DEFAULT") as LPCWSTR,
                            0,
                            KEY_READ,
                            core::ptr::addr_of_mut!(hUserKey))) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return dwError;
    }

    pSetupEnablePrivilege(w!("SeBackupPrivilege") as LPCWSTR, (TRUE) as i32);

    dwError = (RegSaveKeyExW(hUserKey,
                            szDefaultUserHive.as_ptr(),
                            core::ptr::null_mut(),
                            REG_STANDARD_FORMAT)) as u32;
    if dwError == ERROR_ALREADY_EXISTS {
        let mut szBackupHive: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

        wcscpy(szBackupHive.as_mut_ptr(), szDefaultUserHive.as_ptr());
        wcscpy(
            szBackupHive.as_mut_ptr().add(wcslen(szBackupHive.as_ptr()) - 4),
            w!(".bak") as LPCWSTR
        );

        if MoveFileExW(szDefaultUserHive.as_ptr(),
                         szBackupHive.as_ptr(),
                         (MOVEFILE_REPLACE_EXISTING) as u32) == 0 {
            dwError = GetLastError();
            // DPRINT removed
        } else {
            dwError = (RegSaveKeyExW(hUserKey,
                                    szDefaultUserHive.as_ptr(),
                                    core::ptr::null_mut(),
                                    REG_STANDARD_FORMAT)) as u32;
        }
    }
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
    }

    pSetupEnablePrivilege(w!("SeBackupPrivilege") as LPCWSTR, (FALSE) as i32);

    RegCloseKey(hUserKey);

    dwError
}
unsafe extern "C" fn InstallReactOS() -> u32 {
    let mut szBuffer: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut hKey: HKEY = core::ptr::null_mut();
    let hHotkeyThread: *mut core::ffi::c_void;

    // Upstream d55a77c350e: every failure below still reaches the reboot after the block
    // (it used to `return 0` and leave setup sitting in a dead process), and failing to get
    // the shutdown privilege is now logged, no longer fatal.
    'quit: {
        InitializeSetupActionLog((FALSE) as i32);
        LogItem!(core::ptr::null_mut(), w!("Installing ReactOS") as PCWSTR);

        CreateTempDir(w!("TEMP") as PCWSTR);
        CreateTempDir(w!("TMP") as PCWSTR);

        if InitializeProgramFilesDir() == 0 {
            FatalError(b"InitializeProgramFilesDir() failed\0".as_ptr() as *mut i8);
            break 'quit;
        }

        if InitializeProfiles() == 0 {
            FatalError(b"InitializeProfiles() failed\0".as_ptr() as *mut i8);
            break 'quit;
        }

        InitializeDefaultUserLocale();

        if GetWindowsDirectoryW(szBuffer.as_mut_ptr(), ARRAYSIZE!(szBuffer)) != 0 {
            if RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") as PCWSTR,
                0,
                KEY_WRITE,
                &mut hKey
            ) == (ERROR_SUCCESS) as i32 {
                RegSetValueExW(
                    hKey,
                    w!("PathName") as PCWSTR,
                    0,
                    (REG_SZ) as u32,
                    szBuffer.as_mut_ptr() as LPBYTE,
                    ((wcslen(szBuffer.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32
                );

                RegSetValueExW(
                    hKey,
                    w!("SystemRoot") as PCWSTR,
                    0,
                    (REG_SZ) as u32,
                    szBuffer.as_mut_ptr() as LPBYTE,
                    ((wcslen(szBuffer.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32
                );

                RegCloseKey(hKey);
            }

            PathAddBackslash(szBuffer.as_mut_ptr());
            wcscat(szBuffer.as_mut_ptr(), w!("system") as PCWSTR);
            CreateDirectory(szBuffer.as_mut_ptr(), core::ptr::null_mut());
        }

        if SaveDefaultUserHive() != ERROR_SUCCESS {
            FatalError(b"SaveDefaultUserHive() failed\0".as_ptr() as *mut i8);
            break 'quit;
        }

        if CopySystemProfile(0) == 0 {
            FatalError(b"CopySystemProfile() failed\0".as_ptr() as *mut i8);
            break 'quit;
        }

        hHotkeyThread = CreateThread(core::ptr::null_mut(), 0, Some(HotkeyThread), core::ptr::null_mut(), 0, core::ptr::null_mut());

        PreprocessUnattend((TRUE) as i32);
        if CommonInstall() == 0 {
            break 'quit;
        }

        // Upstream 17846e97d08 (CORE-20422): bring up TCP/IP before the wizard, so the
        // second-stage installer can download Wine Gecko again.
        let ret = InstallNetworkComponent(w!("MS_TCPIP") as PWSTR);
        if ret == 0 && GetLastError() != ERROR_FILE_NOT_FOUND {
            // DPRINT removed
        } else {
            SetupStartService(w!("Tcpip") as LPCWSTR, (FALSE) as i32);
            SetupStartService(w!("Dhcp") as LPCWSTR, (FALSE) as i32);
            SetupStartService(w!("Dnscache") as LPCWSTR, (FALSE) as i32);
        }

        InstallWizard();

        // Upstream 55f9484b958 moved this into the wizard's ItemCompletionThread
        // (SaveSettings, with progress steps). That rewrite is not ported, so it must stay
        // here: dropping it alone would leave the security settings never installed.
        InstallSecurity();

        SetAutoAdminLogon();

        SetupCloseInfFile(hSysSetupInf);
        SetSetupType(0);

        if !hHotkeyThread.is_null() {
            PostThreadMessage(GetThreadId(hHotkeyThread), (WM_QUIT) as u32, 0, 0);
            CloseHandle(hHotkeyThread);
        }

        LogItem!(core::ptr::null_mut(), w!("Installing ReactOS done") as PCWSTR);
        TerminateSetupActionLog();

        if !AdminInfo.Name.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, AdminInfo.Name as LPVOID);
        }

        if !AdminInfo.Domain.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, AdminInfo.Domain as LPVOID);
        }

        if !AdminInfo.Password.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, AdminInfo.Password as LPVOID);
        }

    } // 'quit

    // HACK (upstream too): this should be done by the caller of InstallWindowsNt().
    // Enable the shutdown privilege and reboot the machine.
    if pSetupEnablePrivilege(SE_SHUTDOWN_NAME!() as PCWSTR, (TRUE) as i32) == 0 {
        DbgPrint(
            c"[SYSSETUP] pSetupEnablePrivilege(SeShutdownPrivilege) failed (error %lu)\n".as_ptr(),
            GetLastError(),
        );
    }
    // The install ends here: GUI setup's reboot into the second installed boot. Say whether
    // it was accepted -- a refused ExitWindowsEx leaves the machine on its first-boot desktop.
    let ok = ExitWindowsEx((EWX_REBOOT) as u32, 0);
    DbgPrint(
        c"[SYSSETUP] ExitWindowsEx(EWX_REBOOT) = %d (error %lu)\n".as_ptr(),
        ok as i32,
        GetLastError(),
    );
    return 0;
}

extern "C" {
    // ntdll, cdecl.
    fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
}

#[no_mangle]
pub unsafe extern "system" fn InstallWindowsNt(argc: i32, argv: *mut PWSTR) -> i32 {
    let mut i: i32;
    let mut p: PWSTR;

    i = 0;
    while i < argc {
        p = *argv.offset(i as isize);
        if *p == 0x002D {
            p = p.offset(1);

            if _wcsicmp(p, w!("newsetup") as PCWSTR) == 0 {
                return InstallReactOS() as i32;
            } else if _wcsicmp(p, w!("mini") as PCWSTR) == 0 {
                return InstallLiveCD() as i32;
            }
        }
        i += 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn SetupChangeFontSize(
    hWnd: HANDLE,
    lpszFontSize: LPCWSTR,
) -> DWORD {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupChangeLocaleEx(
    hWnd: HWND,
    Lcid: LCID,
    lpSrcRootPath: LPCWSTR,
    // install.c:1751 names this `char Unknown`, but `Unknown` is ALSO a glob-imported constant
    // (ntdll-sys re-exports `pub const Unknown: _MEDIA_TYPE`), and an identifier pattern that
    // resolves to a constant is a CONSTANT PATTERN, not a binding — the same shape as the
    // §9.112 catch-all match arm, in parameter position. Renamed so it binds.
    _Unknown: c_char,
    dwUnused1: u32,
    dwUnused2: u32,
) -> u32 {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupChangeLocale(hWnd: HWND, Lcid: LCID) -> u32 {
    SetupChangeLocaleEx(hWnd, Lcid, core::ptr::null(), 0 as c_char, 0, 0)
}

#[no_mangle]
pub unsafe extern "system" fn SetupStartService(
    lpServiceName: LPCWSTR,
    bWait: BOOL,
) -> DWORD {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;

    'done: {
        hManager = OpenSCManagerW(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (SC_MANAGER_ALL_ACCESS) as u32,
        );
        if hManager.is_null() {
            dwError = GetLastError();
            break 'done;
        }

        hService = OpenServiceW(hManager, lpServiceName, (SERVICE_START) as u32);
        if hService.is_null() {
            dwError = GetLastError();
            break 'done;
        }

        if StartService(hService, 0, core::ptr::null_mut()) == 0 {
            dwError = GetLastError();
            break 'done;
        }
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    dwError
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS system libraries
//  * PURPOSE:           System setup
//  * FILE:              dll/win32/syssetup/install.c
//  * PROGRAMER:         Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #define COBJMACROS
//
// #include <io.h>
// #include <wincon.h>
// #include <winnls.h>
// #include <winsvc.h>
// #include <userenv.h>
// #include <shlobj.h>
// #include <shlwapi.h>
// #include <shobjidl.h>
// #include <rpcproxy.h>
// #include <ndk/cmfuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// //DWORD WINAPI
// //CMP_WaitNoPendingInstallEvents(DWORD dwTimeout);
//
// DWORD WINAPI
// SetupStartService(LPCWSTR lpServiceName, BOOL bWait);
//
// /* GLOBALS ******************************************************************/
//
// HINF hSysSetupInf = INVALID_HANDLE_VALUE;
// ADMIN_INFO AdminInfo;
//
// typedef struct _DLG_DATA
// {
//     HBITMAP hLogoBitmap;
//     HBITMAP hBarBitmap;
//     HWND hWndBarCtrl;
//     DWORD BarCounter;
//     DWORD BarWidth;
//     DWORD BarHeight;
// } DLG_DATA, *PDLG_DATA;
//
// /* FUNCTIONS ****************************************************************/
//
// static VOID
// FatalError(char *pszFmt,...)
// {
//     char szBuffer[512];
//     va_list ap;
//
//     va_start(ap, pszFmt);
//     vsprintf(szBuffer, pszFmt, ap);
//     va_end(ap);
//
//     LogItem!(NULL, L"Failed");
//
//     strcat(szBuffer, "\nRebooting now!");
//     MessageBoxA(NULL,
//                 szBuffer,
//                 "ReactOS Setup",
//                 MB_OK);
// }
//
// static HRESULT
// CreateShellLink(
//     LPCWSTR pszLinkPath,
//     LPCWSTR pszCmd,
//     LPCWSTR pszArg,
//     LPCWSTR pszDir,
//     LPCWSTR pszIconPath,
//     INT iIconNr,
//     LPCWSTR pszComment)
// {
//     IShellLinkW *psl;
//     IPersistFile *ppf;
//
//     HRESULT hr = CoCreateInstance(&CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, &IID_IShellLink, (LPVOID*)&psl);
//
//     if (SUCCEEDED!(hr))
//     {
//         hr = IShellLinkW_SetPath!(psl, pszCmd);
//
//         if (pszArg)
//             hr = IShellLinkW_SetArguments!(psl, pszArg);
//
//         if (pszDir)
//             hr = IShellLinkW_SetWorkingDirectory!(psl, pszDir);
//
//         if (pszIconPath)
//             hr = IShellLinkW_SetIconLocation!(psl, pszIconPath, iIconNr);
//
//         if (pszComment)
//             hr = IShellLinkW_SetDescription!(psl, pszComment);
//
//         hr = IShellLinkW_QueryInterface!(psl, &IID_IPersistFile, (LPVOID*)&ppf);
//
//         if (SUCCEEDED!(hr))
//         {
//             hr = IPersistFile_Save!(ppf, pszLinkPath, TRUE);
//             IPersistFile_Release!(ppf);
//         }
//
//         IShellLinkW_Release!(psl);
//     }
//
//     return hr;
// }
//
//
// static BOOL
// CreateShortcut(
//     LPCWSTR pszFolder,
//     LPCWSTR pszName,
//     LPCWSTR pszCommand,
//     LPCWSTR pszDescription,
//     INT iIconNr,
//     LPCWSTR pszWorkingDir,
//     LPCWSTR pszArgs)
// {
//     DWORD dwLen;
//     LPWSTR Ptr;
//     LPWSTR lpFilePart;
//     WCHAR szPath[MAX_PATH];
//     WCHAR szWorkingDirBuf[MAX_PATH];
//
//     /* If no working directory is provided, try to compute a default one */
//     if (pszWorkingDir == NULL || pszWorkingDir[0] == 0u16)
//     {
//         if (ExpandEnvironmentStringsW(pszCommand, szPath, ARRAYSIZE!(szPath)) == 0)
//             wcscpy(szPath, pszCommand);
//
//         dwLen = GetFullPathNameW(szPath,
//                                  ARRAYSIZE!(szWorkingDirBuf),
//                                  szWorkingDirBuf,
//                                  &lpFilePart);
//         if (dwLen != 0 && dwLen <= ARRAYSIZE!(szWorkingDirBuf))
//         {
//             /* Since those should only be called with (.exe) files,
//                lpFilePart has not to be NULL */
//             ASSERT(lpFilePart != NULL);
//
//             /* We're only interested in the path. Cut the file name off.
//                Also remove the trailing backslash unless the working directory
//                is only going to be a drive, i.e. C:\ */
//             *(lpFilePart--) = 0u16;
//             if (!(lpFilePart - szWorkingDirBuf == 2 &&
//                   szWorkingDirBuf[1] == ':' && szWorkingDirBuf[2] == '\\'))
//             {
//                 *lpFilePart = 0u16;
//             }
//             pszWorkingDir = szWorkingDirBuf;
//         }
//     }
//
//     /* If we failed to compute a working directory, just do not use one */
//     if (pszWorkingDir && pszWorkingDir[0] == 0u16)
//         pszWorkingDir = NULL;
//
//     /* Build the shortcut file name */
//     wcscpy(szPath, pszFolder);
//     Ptr = PathAddBackslash(szPath);
//     wcscpy(Ptr, pszName);
//
//     /* Create the shortcut */
//     return SUCCEEDED!(CreateShellLink(szPath,
//                                      pszCommand,
//                                      pszArgs,
//                                      pszWorkingDir,
//                                      /* Special value to indicate no icon */
//                                      (iIconNr != -1 ? pszCommand : NULL),
//                                      iIconNr,
//                                      pszDescription));
// }
//
//
// static BOOL
// CreateShortcutsFromSection(
//      PITEMSDATA pItemsData,
//      PREGISTRATIONNOTIFY pNotify,
//      HINF hinf,
//      LPWSTR pszSection,
//      LPCWSTR pszFolder)
// {
//     INFCONTEXT Context;
//     DWORD dwFieldCount;
//     INT iIconNr;
//     WCHAR szCommand[MAX_PATH];
//     WCHAR szName[MAX_PATH];
//     WCHAR szDescription[MAX_PATH];
//     WCHAR szDirectory[MAX_PATH];
//     WCHAR szArgs[MAX_PATH];
//
//     if (!SetupFindFirstLine(hinf, pszSection, NULL, &Context))
//         return FALSE;
//
//     do
//     {
//         pNotify->Progress++;
//
//         SendMessage(pItemsData->hwndDlg, PM_STEP_START, 0, (LPARAM)pNotify);
//
//         dwFieldCount = SetupGetFieldCount(&Context);
//         if (dwFieldCount < 3)
//             continue;
//
//         if (!SetupGetStringFieldW(&Context, 1, szCommand, ARRAYSIZE!(szCommand), NULL))
//             continue;
//
//         if (!SetupGetStringFieldW(&Context, 2, szName, ARRAYSIZE!(szName), NULL))
//             continue;
//
//         if (!SetupGetStringFieldW(&Context, 3, szDescription, ARRAYSIZE!(szDescription), NULL))
//             continue;
//
//         if (dwFieldCount < 4 || !SetupGetIntField(&Context, 4, &iIconNr))
//             iIconNr = -1; /* Special value to indicate no icon */
//
//         if (dwFieldCount < 5 || !SetupGetStringFieldW(&Context, 5, szDirectory, ARRAYSIZE!(szDirectory), NULL))
//             szDirectory[0] = 0u16;
//
//         if (dwFieldCount < 6 || !SetupGetStringFieldW(&Context, 6, szArgs, ARRAYSIZE!(szArgs), NULL))
//             szArgs[0] = 0u16;
//
//         wcscat(szName, L".lnk");
//
//         CreateShortcut(pszFolder, szName, szCommand, szDescription, iIconNr, szDirectory, szArgs);
//
//         SendMessage(pItemsData->hwndDlg, PM_STEP_END, 0, (LPARAM)pNotify);
//
//     } while (SetupFindNextLine(&Context, &Context));
//
//     return TRUE;
// }
//
// static BOOL
// CreateShortcuts(
//      PITEMSDATA pItemsData,
//      PREGISTRATIONNOTIFY pNotify,
//      HINF hinf,
//      LPCWSTR szSection)
// {
//     INFCONTEXT Context;
//     WCHAR szPath[MAX_PATH];
//     WCHAR szFolder[MAX_PATH];
//     WCHAR szFolderSection[MAX_PATH];
//     INT csidl;
//
//     CoInitialize(NULL);
//
//     if (!SetupFindFirstLine(hinf, szSection, NULL, &Context))
//         return FALSE;
//
//     do
//     {
//         if (SetupGetFieldCount(&Context) < 2)
//             continue;
//
//         if (!SetupGetStringFieldW(&Context, 0, szFolderSection, ARRAYSIZE!(szFolderSection), NULL))
//             continue;
//
//         if (!SetupGetIntField(&Context, 1, &csidl))
//             continue;
//
//         if (!SetupGetStringFieldW(&Context, 2, szFolder, ARRAYSIZE!(szFolder), NULL))
//             continue;
//
//         if (FAILED!(SHGetFolderPathAndSubDirW(NULL, csidl|CSIDL_FLAG_CREATE, (HANDLE)-1, SHGFP_TYPE_DEFAULT, szFolder, szPath)))
//             continue;
//
//         CreateShortcutsFromSection(pItemsData, pNotify, hinf, szFolderSection, szPath);
//
//     } while (SetupFindNextLine(&Context, &Context));
//
//     CoUninitialize();
//
//     return TRUE;
// }
//
// static LONG
// CountShortcuts(
//      HINF hinf,
//      LPCWSTR szSection)
// {
//     INFCONTEXT Context;
//     WCHAR szFolderSection[MAX_PATH];
//     LONG Steps = 0;
//
//     if (!SetupFindFirstLine(hinf, szSection, NULL, &Context))
//         return FALSE;
//
//     do
//     {
//         if (SetupGetFieldCount(&Context) < 2)
//             continue;
//
//         if (!SetupGetStringFieldW(&Context, 0, szFolderSection, ARRAYSIZE!(szFolderSection), NULL))
//             continue;
//
//         Steps += SetupGetLineCountW(hinf, szFolderSection);
//     } while (SetupFindNextLine(&Context, &Context));
//
//     return Steps;
// }
//
// VOID
// InstallStartMenuItems(
//      PITEMSDATA pItemsData)
// {
//     HINF hShortcutsInf1 = INVALID_HANDLE_VALUE;
//     HINF hShortcutsInf2 = INVALID_HANDLE_VALUE;
//     LONG Steps = 0;
//     DWORD LastError = 0;
//     REGISTRATIONNOTIFY Notify;
//
//     ZeroMemory!(&Notify, sizeof(Notify));
//
//     hShortcutsInf1 = SetupOpenInfFileW(L"shortcuts.inf",
//                                       NULL,
//                                       INF_STYLE_WIN4,
//                                       NULL);
//     if (hShortcutsInf1 == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("Failed to open shortcuts.inf");
//         return;
//     }
//
//     hShortcutsInf2 = SetupOpenInfFileW(L"rosapps_shortcuts.inf",
//                                        NULL,
//                                        INF_STYLE_WIN4,
//                                        NULL);
//
//     Steps = CountShortcuts(hShortcutsInf1, L"ShortcutFolders");
//     if (hShortcutsInf2 != INVALID_HANDLE_VALUE)
//         Steps += CountShortcuts(hShortcutsInf2, L"ShortcutFolders");
//
//     SendMessage(pItemsData->hwndDlg, PM_ITEM_START, 1, (LPARAM)Steps);
//
//     if (!CreateShortcuts(pItemsData, &Notify, hShortcutsInf1, L"ShortcutFolders"))
//     {
//         DPRINT1("CreateShortcuts() failed");
//         goto done;
//     }
//
//     if (hShortcutsInf2 != INVALID_HANDLE_VALUE)
//     {
//         if (!CreateShortcuts(pItemsData, &Notify, hShortcutsInf2, L"ShortcutFolders"))
//         {
//             DPRINT1("CreateShortcuts(rosapps) failed");
//             goto done;
//         }
//     }
//
// done:
//     if (hShortcutsInf2 != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(hShortcutsInf2);
//
//     if (hShortcutsInf1 != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(hShortcutsInf1);
//
//     SendMessage(pItemsData->hwndDlg, PM_ITEM_END, 1, LastError);
// }
//
// static VOID
// CreateTempDir(
//      LPCWSTR VarName)
// {
//     WCHAR szTempDir[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     DWORD dwLength;
//     HKEY hKey;
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
//                       0,
//                       KEY_QUERY_VALUE,
//                       &hKey) != ERROR_SUCCESS)
//     {
//         FatalError("Error: %lu\n", GetLastError());
//         return;
//     }
//
//     /* Get temp dir */
//     dwLength = sizeof(szBuffer);
//     if (RegQueryValueExW(hKey,
//                          VarName,
//                          NULL,
//                          NULL,
//                          (LPBYTE)szBuffer,
//                          &dwLength) != ERROR_SUCCESS)
//     {
//         FatalError("Error: %lu\n", GetLastError());
//         goto cleanup;
//     }
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szBuffer, szTempDir, ARRAYSIZE!(szTempDir)))
//     {
//         FatalError("Error: %lu\n", GetLastError());
//         goto cleanup;
//     }
//
//     /* Create profiles directory */
//     if (!CreateDirectoryW(szTempDir, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             FatalError("Error: %lu\n", GetLastError());
//             goto cleanup;
//         }
//     }
//
// cleanup:
//     RegCloseKey(hKey);
// }
//
// static BOOL
// InstallSysSetupInfDevices(VOID)
// {
//     INFCONTEXT InfContext;
//     WCHAR szLineBuffer[256];
//     DWORD dwLineLength;
//
//     if (!SetupFindFirstLineW(hSysSetupInf,
//                             L"DeviceInfsToInstall",
//                             NULL,
//                             &InfContext))
//     {
//         return FALSE;
//     }
//
//     do
//     {
//         if (!SetupGetStringFieldW(&InfContext,
//                                   0,
//                                   szLineBuffer,
//                                   ARRAYSIZE!(szLineBuffer),
//                                   &dwLineLength))
//         {
//             return FALSE;
//         }
//
//         if (!SetupDiInstallClassW(NULL, szLineBuffer, DI_QUIETINSTALL, NULL))
//         {
//             return FALSE;
//         }
//     }
//     while (SetupFindNextLine(&InfContext, &InfContext));
//
//     return TRUE;
// }
//
// static BOOL
// InstallSysSetupInfComponents(VOID)
// {
//     INFCONTEXT InfContext;
//     WCHAR szNameBuffer[256];
//     WCHAR szSectionBuffer[256];
//     HINF hComponentInf = INVALID_HANDLE_VALUE;
//
//     if (!SetupFindFirstLineW(hSysSetupInf,
//                              L"Infs.Always",
//                              NULL,
//                              &InfContext))
//     {
//         DPRINT("No Inf.Always section found\n");
//     }
//     else
//     {
//         do
//         {
//             if (!SetupGetStringFieldW(&InfContext,
//                                       1, // Get the component name
//                                       szNameBuffer,
//                                       ARRAYSIZE!(szNameBuffer),
//                                       NULL))
//             {
//                 FatalError("Error while trying to get component name\n");
//                 return FALSE;
//             }
//
//             if (!SetupGetStringFieldW(&InfContext,
//                                       2, // Get the component install section
//                                       szSectionBuffer,
//                                       ARRAYSIZE!(szSectionBuffer),
//                                       NULL))
//             {
//                 FatalError("Error while trying to get component install section\n");
//                 return FALSE;
//             }
//
//             DPRINT("Trying to execute install section '%S' from '%S'\n", szSectionBuffer, szNameBuffer);
//
//             hComponentInf = SetupOpenInfFileW(szNameBuffer,
//                                               NULL,
//                                               INF_STYLE_WIN4,
//                                               NULL);
//
//             if (hComponentInf == INVALID_HANDLE_VALUE)
//             {
//                 FatalError("SetupOpenInfFileW() failed to open '%S' (Error: %lu)\n", szNameBuffer, GetLastError());
//                 return FALSE;
//             }
//
//             if (!SetupInstallFromInfSectionW(NULL,
//                                              hComponentInf,
//                                              szSectionBuffer,
//                                              SPINST_ALL,
//                                              NULL,
//                                              NULL,
//                                              SP_COPY_NEWER,
//                                              SetupDefaultQueueCallbackW,
//                                              NULL,
//                                              NULL,
//                                              NULL))
//            {
//                 FatalError("Error while trying to install : %S (Error: %lu)\n", szNameBuffer, GetLastError());
//                 SetupCloseInfFile(hComponentInf);
//                 return FALSE;
//            }
//
//            SetupCloseInfFile(hComponentInf);
//         }
//         while (SetupFindNextLine(&InfContext, &InfContext));
//     }
//
//     return TRUE;
// }
//
//
//
// BOOL
// RegisterTypeLibraries(
//      PITEMSDATA pItemsData,
//      PREGISTRATIONNOTIFY pNotify,
//      HINF hinf,
//      LPCWSTR szSection)
// {
//     INFCONTEXT InfContext;
//     BOOL res;
//     WCHAR szName[MAX_PATH];
//     WCHAR szPath[MAX_PATH];
//     INT csidl;
//     LPWSTR p;
//     HMODULE hmod;
//     HRESULT hret;
//
//     /* Begin iterating the entries in the inf section */
//     res = SetupFindFirstLine(hinf, szSection, NULL, &InfContext);
//     if (!res) return FALSE;
//
//     do
//     {
//         /* Get the name of the current type library */
//         if (!SetupGetStringFieldW(&InfContext, 1, szName, ARRAYSIZE!(szName), NULL))
//         {
//             FatalError("SetupGetStringFieldW failed\n");
//             continue;
//         }
//
//         if (!SetupGetIntField(&InfContext, 2, &csidl))
//             csidl = CSIDL_SYSTEM;
//
//         hret = SHGetFolderPathW(NULL, csidl, NULL, 0, szPath);
//         if (FAILED!(hret))
//         {
//             FatalError("SHGetFolderPathW failed hret=0x%lx\n", hret);
//             continue;
//         }
//
//         p = PathAddBackslash(szPath);
//         wcscpy(p, szName);
//
//         if (pItemsData && pNotify)
//         {
//             pNotify->Progress++;
//             pNotify->CurrentItem = szName;
//
//             DPRINT("RegisterTypeLibraries: Start step %ld\n", pNotify->Progress);
//             SendMessage(pItemsData->hwndDlg, PM_STEP_START, 0, (LPARAM)pNotify);
//         }
//
//         hmod = LoadLibraryW(szPath);
//         if (hmod == NULL)
//         {
//             FatalError("LoadLibraryW failed\n");
//             continue;
//         }
//
//         __wine_register_resources(hmod);
//
//         if (pItemsData && pNotify)
//         {
//             DPRINT("RegisterTypeLibraries: End step %ld\n", pNotify->Progress);
//             SendMessage(pItemsData->hwndDlg, PM_STEP_END, 0, (LPARAM)pNotify);
//         }
//
//     } while (SetupFindNextLine(&InfContext, &InfContext));
//
//     return TRUE;
// }
//
// static BOOL
// EnableUserModePnpManager(VOID)
// {
//     SC_HANDLE hSCManager = NULL;
//     SC_HANDLE hService = NULL;
//     SERVICE_STATUS_PROCESS ServiceStatus;
//     BOOL bRet = FALSE;
//     DWORD BytesNeeded, WaitTime;
//
//     hSCManager = OpenSCManagerW(NULL, NULL, SC_MANAGER_ENUMERATE_SERVICE);
//     if (hSCManager == NULL)
//     {
//         DPRINT1("Unable to open the service control manager.\n");
//         DPRINT1("Last Error %d\n", GetLastError());
//         goto cleanup;
//     }
//
//     hService = OpenServiceW(hSCManager,
//                             L"PlugPlay",
//                             SERVICE_CHANGE_CONFIG | SERVICE_START | SERVICE_QUERY_STATUS);
//     if (hService == NULL)
//     {
//         DPRINT1("Unable to open PlugPlay service\n");
//         goto cleanup;
//     }
//
//     bRet = ChangeServiceConfigW(hService,
//                                 SERVICE_NO_CHANGE,
//                                 SERVICE_AUTO_START,
//                                 SERVICE_NO_CHANGE,
//                                 NULL, NULL, NULL,
//                                 NULL, NULL, NULL, NULL);
//     if (!bRet)
//     {
//         DPRINT1("Unable to change the service configuration\n");
//         goto cleanup;
//     }
//
//     bRet = StartServiceW(hService, 0, NULL);
//     if (!bRet && (GetLastError() != ERROR_SERVICE_ALREADY_RUNNING))
//     {
//         DPRINT1("Unable to start service\n");
//         goto cleanup;
//     }
//
//     while (TRUE)
//     {
//         bRet = QueryServiceStatusEx(hService,
//                                     SC_STATUS_PROCESS_INFO,
//                                     (LPBYTE)&ServiceStatus,
//                                     sizeof(ServiceStatus),
//                                     &BytesNeeded);
//         if (!bRet)
//         {
//             DPRINT1("QueryServiceStatusEx() failed for PlugPlay service (error 0x%x)\n", GetLastError());
//             goto cleanup;
//         }
//
//         if (ServiceStatus.dwCurrentState != SERVICE_START_PENDING)
//             break;
//
//         WaitTime = ServiceStatus.dwWaitHint / 10;
//         if (WaitTime < 1000) WaitTime = 1000;
//         else if (WaitTime > 10000) WaitTime = 10000;
//         Sleep(WaitTime);
//     };
//
//     if (ServiceStatus.dwCurrentState != SERVICE_RUNNING)
//     {
//         bRet = FALSE;
//         DPRINT1("Failed to start PlugPlay service\n");
//         goto cleanup;
//     }
//
//     bRet = TRUE;
//
// cleanup:
//     if (hService != NULL)
//         CloseServiceHandle(hService);
//     if (hSCManager != NULL)
//         CloseServiceHandle(hSCManager);
//     return bRet;
// }
//
// static VOID
// AdjustStatusMessageWindow(HWND hwndDlg, PDLG_DATA pDlgData)
// {
//     INT xOld, yOld, cxOld, cyOld;
//     INT xNew, yNew, cxNew, cyNew;
//     INT cxLabel, cyLabel, dyLabel;
//     RECT rc, rcBar, rcLabel, rcWnd;
//     BITMAP bmLogo, bmBar;
//     DWORD style, exstyle;
//     HWND hwndLogo = GetDlgItem(hwndDlg, IDC_ROSLOGO);
//     HWND hwndBar = GetDlgItem(hwndDlg, IDC_BAR);
//     HWND hwndLabel = GetDlgItem(hwndDlg, IDC_STATUSLABEL);
//
//     /* This adjustment is for CJK only */
//     switch (PRIMARYLANGID!(GetUserDefaultLangID()))
//     {
//         case LANG_CHINESE:
//         case LANG_JAPANESE:
//         case LANG_KOREAN:
//             break;
//
//         default:
//             return;
//     }
//
//     if (!GetObjectW(pDlgData->hLogoBitmap, sizeof(BITMAP), &bmLogo) ||
//         !GetObjectW(pDlgData->hBarBitmap, sizeof(BITMAP), &bmBar))
//     {
//         return;
//     }
//
//     GetWindowRect(hwndBar, &rcBar);
//     MapWindowPoints(NULL, hwndDlg, (LPPOINT)&rcBar, 2);
//     dyLabel = bmLogo.bmHeight - rcBar.top;
//
//     GetWindowRect(hwndLabel, &rcLabel);
//     MapWindowPoints(NULL, hwndDlg, (LPPOINT)&rcLabel, 2);
//     cxLabel = rcLabel.right - rcLabel.left;
//     cyLabel = rcLabel.bottom - rcLabel.top;
//
//     MoveWindow(hwndLogo, 0, 0, bmLogo.bmWidth, bmLogo.bmHeight, TRUE);
//     MoveWindow(hwndBar, 0, bmLogo.bmHeight, bmLogo.bmWidth, bmBar.bmHeight, TRUE);
//     MoveWindow(hwndLabel, rcLabel.left, rcLabel.top + dyLabel, cxLabel, cyLabel, TRUE);
//
//     GetWindowRect(hwndDlg, &rcWnd);
//     xOld = rcWnd.left;
//     yOld = rcWnd.top;
//     cxOld = rcWnd.right - rcWnd.left;
//     cyOld = rcWnd.bottom - rcWnd.top;
//
//     GetClientRect(hwndDlg, &rc);
//     SetRect(&rc, 0, 0, bmLogo.bmWidth, rc.bottom - rc.top); /* new client size */
//
//     style = (DWORD)GetWindowLongPtrW(hwndDlg, GWL_STYLE);
//     exstyle = (DWORD)GetWindowLongPtrW(hwndDlg, GWL_EXSTYLE);
//     AdjustWindowRectEx(&rc, style, FALSE, exstyle);
//
//     cxNew = rc.right - rc.left;
//     cyNew = (rc.bottom - rc.top) + dyLabel;
//     xNew = xOld - (cxNew - cxOld) / 2;
//     yNew = yOld - (cyNew - cyOld) / 2;
//     MoveWindow(hwndDlg, xNew, yNew, cxNew, cyNew, TRUE);
// }
//
// static INT_PTR __stdcall
// StatusMessageWindowProc(
//      HWND hwndDlg,
//      UINT uMsg,
//      WPARAM wParam,
//      LPARAM lParam)
// {
//     PDLG_DATA pDlgData;
//     UNREFERENCED_PARAMETER!(wParam);
//
//     pDlgData = (PDLG_DATA)GetWindowLongPtrW(hwndDlg, GWLP_USERDATA);
//
//     /* pDlgData is required for each case except WM_INITDIALOG */
//     if (uMsg != WM_INITDIALOG && pDlgData == NULL) return FALSE;
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             BITMAP bm;
//             WCHAR szMsg[256];
//
//             /* Allocate pDlgData */
//             pDlgData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(*pDlgData));
//             if (pDlgData)
//             {
//                 /* Set pDlgData to GWLP_USERDATA, so we can get it for new messages */
//                 SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, (LONG_PTR)pDlgData);
//
//                 /* Load bitmaps */
//                 pDlgData->hLogoBitmap = LoadImageW(hDllInstance,
//                                                     MAKEINTRESOURCEW!(IDB_REACTOS), IMAGE_BITMAP,
//                                                     0, 0, LR_DEFAULTCOLOR);
//
//                 pDlgData->hBarBitmap = LoadImageW(hDllInstance, MAKEINTRESOURCEW!(IDB_LINE),
//                                                 IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR);
//                 GetObject(pDlgData->hBarBitmap, sizeof(bm), &bm);
//                 pDlgData->BarWidth = bm.bmWidth;
//                 pDlgData->BarHeight = bm.bmHeight;
//
//                 if (pDlgData->hLogoBitmap && pDlgData->hBarBitmap)
//                 {
//                     if (SetTimer(hwndDlg, IDT_BAR, 20, NULL) == 0)
//                     {
//                         DPRINT1("SetTimer(IDT_BAR) failed: %lu\n", GetLastError());
//                     }
//
//                     /* Get the animation bar control */
//                     pDlgData->hWndBarCtrl = GetDlgItem(hwndDlg, IDC_BAR);
//                 }
//             }
//
//             /* Get and set status text */
//             if (!LoadStringW(hDllInstance, IDS_STATUS_INSTALL_DEV, szMsg, ARRAYSIZE!(szMsg)))
//                 return FALSE;
//             SetDlgItemTextW(hwndDlg, IDC_STATUSLABEL, szMsg);
//
//             AdjustStatusMessageWindow(hwndDlg, pDlgData);
//             return TRUE;
//         }
//
//         case WM_TIMER:
//         {
//             if (pDlgData->hBarBitmap)
//             {
//                 /*
//                  * Default rotation bar image width is 413 (same as logo)
//                  * We can divide 413 by 7 without remainder
//                  */
//                 pDlgData->BarCounter = (pDlgData->BarCounter + 7) % pDlgData->BarWidth;
//                 InvalidateRect(pDlgData->hWndBarCtrl, NULL, FALSE);
//                 UpdateWindow(pDlgData->hWndBarCtrl);
//             }
//             return TRUE;
//         }
//
//         case WM_DRAWITEM:
//         {
//             LPDRAWITEMSTRUCT lpDis = (LPDRAWITEMSTRUCT)lParam;
//
//             if (lpDis->CtlID != IDC_BAR)
//             {
//                 return FALSE;
//             }
//
//             if (pDlgData->hBarBitmap)
//             {
//                 HDC hdcMem;
//                 HGDIOBJ hOld;
//                 DWORD off = pDlgData->BarCounter;
//                 DWORD iw = pDlgData->BarWidth;
//                 DWORD ih = pDlgData->BarHeight;
//
//                 hdcMem = CreateCompatibleDC(lpDis->hDC);
//                 hOld = SelectObject(hdcMem, pDlgData->hBarBitmap);
//                 BitBlt(lpDis->hDC, off, 0, iw - off, ih, hdcMem, 0, 0, SRCCOPY);
//                 BitBlt(lpDis->hDC, 0, 0, off, ih, hdcMem, iw - off, 0, SRCCOPY);
//                 SelectObject(hdcMem, hOld);
//                 DeleteDC(hdcMem);
//                 return TRUE;
//             }
//             return FALSE;
//         }
//
//         case WM_DESTROY:
//         {
//             if (pDlgData->hBarBitmap)
//             {
//                 KillTimer(hwndDlg, IDT_BAR);
//             }
//
//             DeleteObject(pDlgData->hLogoBitmap);
//             DeleteObject(pDlgData->hBarBitmap);
//             HeapFree(GetProcessHeap(), 0, pDlgData);
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// static DWORD WINAPI
// ShowStatusMessageThread(
//      LPVOID lpParameter)
// {
//     HWND hWnd;
//     MSG Msg;
//     UNREFERENCED_PARAMETER!(lpParameter);
//
//     hWnd = CreateDialogParam(hDllInstance,
//                              MAKEINTRESOURCE!(IDD_STATUSWINDOW_DLG),
//                              GetDesktopWindow(),
//                              StatusMessageWindowProc,
//                              (LPARAM)NULL);
//     if (!hWnd)
//         return 0;
//
//     ShowWindow(hWnd, SW_SHOW);
//
//     /* Message loop for the Status window */
//     while (GetMessage(&Msg, NULL, 0, 0))
//     {
//         TranslateMessage(&Msg);
//         DispatchMessage(&Msg);
//     }
//
//     EndDialog(hWnd, 0);
//
//     return 0;
// }
//
// static LONG
// ReadRegSzKey(
//      HKEY hKey,
//      LPCWSTR pszKey,
//      LPWSTR* pValue)
// {
//     LONG rc;
//     DWORD dwType;
//     DWORD cbData = 0;
//     LPWSTR pwszValue;
//
//     if (!pValue)
//         return ERROR_INVALID_PARAMETER;
//
//     *pValue = NULL;
//     rc = RegQueryValueExW(hKey, pszKey, NULL, &dwType, NULL, &cbData);
//     if (rc != ERROR_SUCCESS)
//         return rc;
//     if (dwType != REG_SZ)
//         return ERROR_FILE_NOT_FOUND;
//     pwszValue = HeapAlloc(GetProcessHeap(), 0, cbData + sizeof(WCHAR));
//     if (!pwszValue)
//         return ERROR_NOT_ENOUGH_MEMORY;
//     rc = RegQueryValueExW(hKey, pszKey, NULL, NULL, (LPBYTE)pwszValue, &cbData);
//     if (rc != ERROR_SUCCESS)
//     {
//         HeapFree(GetProcessHeap(), 0, pwszValue);
//         return rc;
//     }
//     /* NULL-terminate the string */
//     pwszValue[cbData / sizeof(WCHAR)] = '\0';
//
//     *pValue = pwszValue;
//     return ERROR_SUCCESS;
// }
//
// static BOOL
// IsConsoleBoot(VOID)
// {
//     HKEY hControlKey = NULL;
//     LPWSTR pwszSystemStartOptions = NULL;
//     LPWSTR pwszCurrentOption, pwszNextOption; /* Pointers into SystemStartOptions */
//     BOOL bConsoleBoot = FALSE;
//     LONG rc;
//
//     rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                        L"SYSTEM\\CurrentControlSet\\Control",
//                        0,
//                        KEY_QUERY_VALUE,
//                        &hControlKey);
//     if (rc != ERROR_SUCCESS)
//         goto cleanup;
//
//     rc = ReadRegSzKey(hControlKey, L"SystemStartOptions", &pwszSystemStartOptions);
//     if (rc != ERROR_SUCCESS)
//         goto cleanup;
//
//     /* Check for CONSOLE switch in SystemStartOptions */
//     pwszCurrentOption = pwszSystemStartOptions;
//     while (pwszCurrentOption)
//     {
//         pwszNextOption = wcschr(pwszCurrentOption, ' ');
//         if (pwszNextOption)
//             *pwszNextOption = 0u16;
//         if (_wcsicmp(pwszCurrentOption, L"CONSOLE") == 0)
//         {
//             DPRINT("Found %S. Switching to console boot\n", pwszCurrentOption);
//             bConsoleBoot = TRUE;
//             goto cleanup;
//         }
//         pwszCurrentOption = pwszNextOption ? pwszNextOption + 1 : NULL;
//     }
//
// cleanup:
//     if (hControlKey != NULL)
//         RegCloseKey(hControlKey);
//     if (pwszSystemStartOptions)
//         HeapFree(GetProcessHeap(), 0, pwszSystemStartOptions);
//     return bConsoleBoot;
// }
//
// extern VOID
// EnableVisualTheme(
//      HWND hwndParent,
//      PCWSTR ThemeFile);
//
// /**
//  * @brief
//  * Pre-process unattended file to apply early settings.
//  *
//  * @param[in]   IsInstall
//  * TRUE if this is ReactOS installation, invoked from InstallReactOS(),
//  * FALSE if this is run as part of LiveCD, invoked form InstallLiveCD().
//  **/
// static VOID
// PreprocessUnattend(
//      BOOL IsInstall)
// {
//     WCHAR szPath[MAX_PATH];
//     WCHAR szValue[MAX_PATH];
//     BOOL bDefaultThemesOff;
//
//     if (IsInstall)
//     {
//         /* See also wizard.c!ProcessSetupInf()
//          * Retrieve the path of the setup INF */
//         GetSystemDirectoryW(szPath, _countof!(szPath));
//         wcscat(szPath, L"\\$winnt$.inf");
//     }
//     else
//     {
//         /* See also userinit/livecd.c!RunLiveCD() */
//         GetWindowsDirectoryW(szPath, _countof!(szPath));
//         wcscat(szPath, L"\\unattend.inf");
//     }
//
//     /*
//      * Apply initial default theming
//      */
//
//     /* Check whether to use the classic theme (TRUE) instead of the default theme */
//     bDefaultThemesOff = FALSE;
//     if (GetPrivateProfileStringW(L"Shell", L"DefaultThemesOff", L"no", szValue, _countof!(szValue), szPath) && *szValue)
//         bDefaultThemesOff = (_wcsicmp(szValue, L"yes") == 0);
//
//     if (!bDefaultThemesOff)
//     {
//         /* Retrieve the complete path to a .theme (or for ReactOS, a .msstyles) file */
//         if (!GetPrivateProfileStringW(L"Shell", L"CustomDefaultThemeFile", NULL, szValue, _countof!(szValue), szPath) || !*szValue)
//             bDefaultThemesOff = TRUE; // None specified, fall back to the classic theme.
//     }
//
//     /* Enable the chosen theme, or use the classic theme */
//     EnableVisualTheme(NULL, bDefaultThemesOff ? NULL : szValue);
// }
//
// static BOOL
// CommonInstall(VOID)
// {
//     HANDLE hThread = NULL;
//     BOOL bResult = FALSE;
//
//     hSysSetupInf = SetupOpenInfFileW(L"syssetup.inf",
//                                      NULL,
//                                      INF_STYLE_WIN4,
//                                      NULL);
//     if (hSysSetupInf == INVALID_HANDLE_VALUE)
//     {
//         FatalError("SetupOpenInfFileW() failed to open 'syssetup.inf' (Error: %lu)\n", GetLastError());
//         return FALSE;
//     }
//
//     if (!InstallSysSetupInfDevices())
//     {
//         FatalError("InstallSysSetupInfDevices() failed!\n");
//         goto Exit;
//     }
//
//     if (!InstallSysSetupInfComponents())
//     {
//         FatalError("InstallSysSetupInfComponents() failed!\n");
//         goto Exit;
//     }
//
//     if (!IsConsoleBoot())
//     {
//         hThread = CreateThread(NULL,
//                                0,
//                                ShowStatusMessageThread,
//                                NULL,
//                                0,
//                                NULL);
//     }
//
//     if (!EnableUserModePnpManager())
//     {
//         FatalError("EnableUserModePnpManager() failed!\n");
//         goto Exit;
//     }
//
//     if (CMP_WaitNoPendingInstallEvents(INFINITE) != WAIT_OBJECT_0)
//     {
//         FatalError("CMP_WaitNoPendingInstallEvents() failed!\n");
//         goto Exit;
//     }
//
//     bResult = TRUE;
//
// Exit:
//     if (bResult == FALSE)
//     {
//         SetupCloseInfFile(hSysSetupInf);
//     }
//
//     if (hThread != NULL)
//     {
//         PostThreadMessage(GetThreadId(hThread), WM_QUIT, 0, 0);
//         WaitForSingleObject(hThread, INFINITE);
//         CloseHandle(hThread);
//     }
//
//     return bResult;
// }
//
// static
// DWORD
// InstallLiveCD(VOID)
// {
//     STARTUPINFOW StartupInfo;
//     PROCESS_INFORMATION ProcessInformation;
//     BOOL bRes;
//
//     PreprocessUnattend(FALSE);
//     if (!CommonInstall())
//         goto error;
//
//     /* Install the TCP/IP protocol driver */
//     bRes = InstallNetworkComponent(L"MS_TCPIP");
//     if (!bRes && GetLastError() != ERROR_FILE_NOT_FOUND)
//     {
//         DPRINT("InstallNetworkComponent() failed with error 0x%lx\n", GetLastError());
//     }
//     else
//     {
//         /* Start the TCP/IP protocol driver */
//         SetupStartService(L"Tcpip", FALSE);
//         SetupStartService(L"Dhcp", FALSE);
//         SetupStartService(L"Dnscache", FALSE);
//     }
//
//     /* Register components */
//     _SEH2_TRY
//     {
//         if (!SetupInstallFromInfSectionW(NULL,
//                                          hSysSetupInf, L"RegistrationPhase2",
//                                          SPINST_ALL,
//                                          0, NULL, 0, NULL, NULL, NULL, NULL))
//         {
//             DPRINT1("SetupInstallFromInfSectionW failed!\n");
//         }
//
//         RegisterTypeLibraries(NULL, NULL, hSysSetupInf, L"TypeLibraries");
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DPRINT1("Catching exception\n");
//     }
//     _SEH2_END;
//
//     SetupCloseInfFile(hSysSetupInf);
//
//     /* Run the shell */
//     ZeroMemory!(&StartupInfo, sizeof(StartupInfo));
//     StartupInfo.cb = sizeof(StartupInfo);
//     bRes = CreateProcessW(L"userinit.exe",
//                           NULL,
//                           NULL,
//                           NULL,
//                           FALSE,
//                           0,
//                           NULL,
//                           NULL,
//                           &StartupInfo,
//                           &ProcessInformation);
//     if (!bRes)
//         goto error;
//
//     CloseHandle(ProcessInformation.hThread);
//     CloseHandle(ProcessInformation.hProcess);
//
//     return 0;
//
// error:
//     MessageBoxW(
//         NULL,
//         L"Failed to load LiveCD! You can shutdown your computer, or press ENTER to reboot.",
//         L"ReactOS LiveCD",
//         MB_OK);
//     return 0;
// }
//
//
// static BOOL
// SetSetupType(DWORD dwSetupType)
// {
//     DWORD dwError;
//     HKEY hKey;
//
//     dwError = RegOpenKeyExW(
//         HKEY_LOCAL_MACHINE,
//         L"SYSTEM\\Setup",
//         0,
//         KEY_SET_VALUE,
//         &hKey);
//     if (dwError != ERROR_SUCCESS)
//         return FALSE;
//
//     dwError = RegSetValueExW(
//         hKey,
//         L"SetupType",
//         0,
//         REG_DWORD,
//         (LPBYTE)&dwSetupType,
//         sizeof(DWORD));
//     RegCloseKey(hKey);
//     if (dwError != ERROR_SUCCESS)
//         return FALSE;
//
//     return TRUE;
// }
//
// static DWORD __stdcall
// HotkeyThread(LPVOID Parameter)
// {
//     ATOM hotkey;
//     MSG msg;
//
//     DPRINT("HotkeyThread start\n");
//
//     hotkey = GlobalAddAtomW(L"Setup Shift+F10 Hotkey");
//     if (!RegisterHotKey(NULL, hotkey, MOD_SHIFT, VK_F10))
//         DPRINT1("RegisterHotKey failed with %lu\n", GetLastError());
//
//     while (GetMessageW(&msg, NULL, 0, 0))
//     {
//         if (msg.hwnd == NULL && msg.message == WM_HOTKEY && msg.wParam == hotkey)
//         {
//             WCHAR CmdLine[] = L"cmd.exe"; // CreateProcess can modify this buffer.
//             STARTUPINFOW si = { sizeof(si) };
//             PROCESS_INFORMATION pi;
//
//             if (CreateProcessW(NULL,
//                                CmdLine,
//                                NULL,
//                                NULL,
//                                FALSE,
//                                CREATE_NEW_CONSOLE,
//                                NULL,
//                                NULL,
//                                &si,
//                                &pi))
//             {
//                 CloseHandle(pi.hProcess);
//                 CloseHandle(pi.hThread);
//             }
//             else
//             {
//                 DPRINT1("Failed to launch command prompt: %lu\n", GetLastError());
//             }
//         }
//     }
//
//     UnregisterHotKey(NULL, hotkey);
//     GlobalDeleteAtom(hotkey);
//
//     DPRINT("HotkeyThread terminate\n");
//     return 0;
// }
//
//
// static
// BOOL
// InitializeProgramFilesDir(VOID)
// {
//     LONG Error;
//     HKEY hKey;
//     DWORD dwLength;
//     WCHAR szProgramFilesDirPath[MAX_PATH];
//     WCHAR szCommonFilesDirPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//
//     /* Load 'Program Files' location */
//     if (!LoadStringW(hDllInstance,
//                      IDS_PROGRAMFILES,
//                      szBuffer,
//                      ARRAYSIZE!(szBuffer)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     if (!LoadStringW(hDllInstance,
//                      IDS_COMMONFILES,
//                      szCommonFilesDirPath,
//                      ARRAYSIZE!(szCommonFilesDirPath)))
//     {
//         DPRINT1("Warning: %lu\n", GetLastError());
//     }
//
//     /* Expand it */
//     if (!ExpandEnvironmentStringsW(szBuffer,
//                                    szProgramFilesDirPath,
//                                    ARRAYSIZE!(szProgramFilesDirPath)))
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     wcscpy(szBuffer, szProgramFilesDirPath);
//     wcscat(szBuffer, L"\\");
//     wcscat(szBuffer, szCommonFilesDirPath);
//
//     if (!ExpandEnvironmentStringsW(szBuffer,
//                                    szCommonFilesDirPath,
//                                    ARRAYSIZE!(szCommonFilesDirPath)))
//     {
//         DPRINT1("Warning: %lu\n", GetLastError());
//     }
//
//     /* Store it */
//     Error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
//                           0,
//                           KEY_SET_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         return FALSE;
//     }
//
//     dwLength = (wcslen(szProgramFilesDirPath) + 1) * sizeof(WCHAR);
//     Error = RegSetValueExW(hKey,
//                            L"ProgramFilesDir",
//                            0,
//                            REG_SZ,
//                            (LPBYTE)szProgramFilesDirPath,
//                            dwLength);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Error: %lu\n", Error);
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     dwLength = (wcslen(szCommonFilesDirPath) + 1) * sizeof(WCHAR);
//     Error = RegSetValueExW(hKey,
//                            L"CommonFilesDir",
//                            0,
//                            REG_SZ,
//                            (LPBYTE)szCommonFilesDirPath,
//                            dwLength);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("Warning: %lu\n", Error);
//     }
//
//     RegCloseKey(hKey);
//
//     /* Create directory */
//     // FIXME: Security!
//     if (!CreateDirectoryW(szProgramFilesDirPath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Error: %lu\n", GetLastError());
//             return FALSE;
//         }
//     }
//
//     /* Create directory */
//     // FIXME: Security!
//     if (!CreateDirectoryW(szCommonFilesDirPath, NULL))
//     {
//         if (GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             DPRINT1("Warning: %lu\n", GetLastError());
//             // return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// static
// VOID
// InitializeDefaultUserLocale(VOID)
// {
//     WCHAR szBuffer[80];
//     PWSTR ptr;
//     HKEY hLocaleKey;
//     DWORD ret;
//     DWORD dwSize;
//     LCID lcid;
//     INT i;
//
//     struct {LCTYPE LCType; PWSTR pValue;} LocaleData[] = {
//         /* Number */
//         {LOCALE_SDECIMAL, L"sDecimal"},
//         {LOCALE_STHOUSAND, L"sThousand"},
//         {LOCALE_SNEGATIVESIGN, L"sNegativeSign"},
//         {LOCALE_SPOSITIVESIGN, L"sPositiveSign"},
//         {LOCALE_SGROUPING, L"sGrouping"},
//         {LOCALE_SLIST, L"sList"},
//         {LOCALE_SNATIVEDIGITS, L"sNativeDigits"},
//         {LOCALE_INEGNUMBER, L"iNegNumber"},
//         {LOCALE_IDIGITS, L"iDigits"},
//         {LOCALE_ILZERO, L"iLZero"},
//         {LOCALE_IMEASURE, L"iMeasure"},
//         {LOCALE_IDIGITSUBSTITUTION, L"NumShape"},
//
//         /* Currency */
//         {LOCALE_SCURRENCY, L"sCurrency"},
//         {LOCALE_SMONDECIMALSEP, L"sMonDecimalSep"},
//         {LOCALE_SMONTHOUSANDSEP, L"sMonThousandSep"},
//         {LOCALE_SMONGROUPING, L"sMonGrouping"},
//         {LOCALE_ICURRENCY, L"iCurrency"},
//         {LOCALE_INEGCURR, L"iNegCurr"},
//         {LOCALE_ICURRDIGITS, L"iCurrDigits"},
//
//         /* Time */
//         {LOCALE_STIMEFORMAT, L"sTimeFormat"},
//         {LOCALE_STIME, L"sTime"},
//         {LOCALE_S1159, L"s1159"},
//         {LOCALE_S2359, L"s2359"},
//         {LOCALE_ITIME, L"iTime"},
//         {LOCALE_ITIMEMARKPOSN, L"iTimePrefix"},
//         {LOCALE_ITLZERO, L"iTLZero"},
//
//         /* Date */
//         {LOCALE_SLONGDATE, L"sLongDate"},
//         {LOCALE_SSHORTDATE, L"sShortDate"},
//         {LOCALE_SDATE, L"sDate"},
//         {LOCALE_IFIRSTDAYOFWEEK, L"iFirstDayOfWeek"},
//         {LOCALE_IFIRSTWEEKOFYEAR, L"iFirstWeekOfYear"},
//         {LOCALE_IDATE, L"iDate"},
//         {LOCALE_ICALENDARTYPE, L"iCalendarType"},
//
//         /* Misc */
//         {LOCALE_SCOUNTRY, L"sCountry"},
//         {LOCALE_SABBREVLANGNAME, L"sLanguage"},
//         {LOCALE_ICOUNTRY, L"iCountry"},
//         {0, NULL}};
//
//     ret = RegOpenKeyExW(HKEY_USERS,
//                         L".DEFAULT\\Control Panel\\International",
//                         0,
//                         KEY_READ | KEY_WRITE,
//                         &hLocaleKey);
//     if (ret != ERROR_SUCCESS)
//     {
//         return;
//     }
//
//     dwSize = 9 * sizeof(WCHAR);
//     ret = RegQueryValueExW(hLocaleKey,
//                            L"Locale",
//                            NULL,
//                            NULL,
//                            (PBYTE)szBuffer,
//                            &dwSize);
//     if (ret != ERROR_SUCCESS)
//         goto done;
//
//     lcid = (LCID)wcstoul(szBuffer, &ptr, 16);
//     if (lcid == 0)
//         goto done;
//
//     i = 0;
//     while (LocaleData[i].pValue != NULL)
//     {
//         if (GetLocaleInfoW(lcid,
//                            LocaleData[i].LCType | LOCALE_NOUSEROVERRIDE,
//                            szBuffer,
//                            ARRAYSIZE!(szBuffer)))
//         {
//             RegSetValueExW(hLocaleKey,
//                            LocaleData[i].pValue,
//                            0,
//                            REG_SZ,
//                            (PBYTE)szBuffer,
//                            (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//         }
//
//         i++;
//     }
//
// done:
//     RegCloseKey(hLocaleKey);
// }
//
//
// static
// DWORD
// SaveDefaultUserHive(VOID)
// {
//     WCHAR szDefaultUserHive[MAX_PATH];
//     HKEY hUserKey = NULL;
//     DWORD cchSize;
//     DWORD dwError;
//
//     DPRINT("SaveDefaultUserHive()\n");
//
//     cchSize = ARRAYSIZE!(szDefaultUserHive);
//     GetDefaultUserProfileDirectoryW(szDefaultUserHive, &cchSize);
//
//     wcscat(szDefaultUserHive, L"\\ntuser.dat");
//
//     dwError = RegOpenKeyExW(HKEY_USERS,
//                             L".DEFAULT",
//                             0,
//                             KEY_READ,
//                             &hUserKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed (Error %lu)\n", dwError);
//         return dwError;
//     }
//
//     pSetupEnablePrivilege(L"SeBackupPrivilege", TRUE);
//
//     /* Save the Default hive */
//     dwError = RegSaveKeyExW(hUserKey,
//                             szDefaultUserHive,
//                             NULL,
//                             REG_STANDARD_FORMAT);
//     if (dwError == ERROR_ALREADY_EXISTS)
//     {
//         WCHAR szBackupHive[MAX_PATH];
//
//         /* Build the backup hive file name by replacing the extension */
//         wcscpy(szBackupHive, szDefaultUserHive);
//         wcscpy(&szBackupHive[wcslen(szBackupHive) - 4], L".bak");
//
//         /* Back up the existing default user hive by renaming it, replacing any possible existing old backup */
//         if (!MoveFileExW(szDefaultUserHive,
//                          szBackupHive,
//                          MOVEFILE_REPLACE_EXISTING))
//         {
//             dwError = GetLastError();
//             DPRINT1("Failed to create a default-user hive backup '%S', MoveFileExW failed (Error %lu)\n",
//                     szBackupHive, dwError);
//         }
//         else
//         {
//             /* The backup has been done, retry saving the Default hive */
//             dwError = RegSaveKeyExW(hUserKey,
//                                     szDefaultUserHive,
//                                     NULL,
//                                     REG_STANDARD_FORMAT);
//         }
//     }
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegSaveKeyExW() failed (Error %lu)\n", dwError);
//     }
//
//     pSetupEnablePrivilege(L"SeBackupPrivilege", FALSE);
//
//     RegCloseKey(hUserKey);
//
//     return dwError;
// }
//
//
// static
// DWORD
// InstallReactOS(VOID)
// {
//     WCHAR szBuffer[MAX_PATH];
//     HANDLE token;
//     TOKEN_PRIVILEGES privs;
//     HKEY hKey;
//     HANDLE hHotkeyThread;
//
//     InitializeSetupActionLog(FALSE);
//     LogItem!(NULL, L"Installing ReactOS");
//
//     CreateTempDir(L"TEMP");
//     CreateTempDir(L"TMP");
//
//     if (!InitializeProgramFilesDir())
//     {
//         FatalError("InitializeProgramFilesDir() failed");
//         return 0;
//     }
//
//     if (!InitializeProfiles())
//     {
//         FatalError("InitializeProfiles() failed");
//         return 0;
//     }
//
//     InitializeDefaultUserLocale();
//
//     if (GetWindowsDirectoryW(szBuffer, ARRAYSIZE!(szBuffer)))
//     {
//         if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
//                           0,
//                           KEY_WRITE,
//                           &hKey) == ERROR_SUCCESS)
//         {
//             RegSetValueExW(hKey,
//                            L"PathName",
//                            0,
//                            REG_SZ,
//                            (LPBYTE)szBuffer,
//                            (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//             RegSetValueExW(hKey,
//                            L"SystemRoot",
//                            0,
//                            REG_SZ,
//                            (LPBYTE)szBuffer,
//                            (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//             RegCloseKey(hKey);
//         }
//
//         PathAddBackslash(szBuffer);
//         wcscat(szBuffer, L"system");
//         CreateDirectory(szBuffer, NULL);
//     }
//
//     if (SaveDefaultUserHive() != ERROR_SUCCESS)
//     {
//         FatalError("SaveDefaultUserHive() failed");
//         return 0;
//     }
//
//     if (!CopySystemProfile(0))
//     {
//         FatalError("CopySystemProfile() failed");
//         return 0;
//     }
//
//     hHotkeyThread = CreateThread(NULL, 0, HotkeyThread, NULL, 0, NULL);
//
//     PreprocessUnattend(TRUE);
//     if (!CommonInstall())
//         return 0;
//
//     InstallWizard();
//
//     InstallSecurity();
//
//     SetAutoAdminLogon();
//
//     SetupCloseInfFile(hSysSetupInf);
//     SetSetupType(0);
//
//     if (hHotkeyThread)
//     {
//         PostThreadMessage(GetThreadId(hHotkeyThread), WM_QUIT, 0, 0);
//         CloseHandle(hHotkeyThread);
//     }
//
//     LogItem!(NULL, L"Installing ReactOS done");
//     TerminateSetupActionLog();
//
//     if (AdminInfo.Name != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AdminInfo.Name);
//
//     if (AdminInfo.Domain != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AdminInfo.Domain);
//
//     if (AdminInfo.Password != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AdminInfo.Password);
//
//     /* Get shutdown privilege */
//     if (!OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &token))
//     {
//         FatalError("OpenProcessToken() failed!");
//         return 0;
//     }
//     if (!LookupPrivilegeValue(NULL,
//                               SE_SHUTDOWN_NAME,
//                               &privs.Privileges[0].Luid))
//     {
//         FatalError("LookupPrivilegeValue() failed!");
//         return 0;
//     }
//     privs.PrivilegeCount = 1;
//     privs.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
//     if (AdjustTokenPrivileges(token,
//                               FALSE,
//                               &privs,
//                               0,
//                               (PTOKEN_PRIVILEGES)NULL,
//                               NULL) == 0)
//     {
//         FatalError("AdjustTokenPrivileges() failed!");
//         return 0;
//     }
//
//     ExitWindowsEx(EWX_REBOOT, 0);
//     return 0;
// }
//
//
// /*
//  * Standard Windows-compatible export, which dispatches
//  * to either 'InstallReactOS' or 'InstallLiveCD'.
//  */
// INT
// WINAPI
// InstallWindowsNt(INT argc, WCHAR** argv)
// {
//     INT i;
//     PWSTR p;
//
//     for (i = 0; i < argc; ++i)
//     {
//         p = argv[i];
//         if (*p == '-')
//         {
//             p++;
//
//             // NOTE: On Windows, "mini" means "minimal UI", and can be used
//             // in addition to "newsetup"; these options are not exclusive.
//             if (_wcsicmp(p, L"newsetup") == 0)
//                 return (INT)InstallReactOS();
//             else if (_wcsicmp(p, L"mini") == 0)
//                 return (INT)InstallLiveCD();
//
//             /* Add support for other switches */
//         }
//     }
//
//     return 0;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD WINAPI
// SetupChangeFontSize(
//      HANDLE hWnd,
//      LPCWSTR lpszFontSize)
// {
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// DWORD WINAPI
// SetupChangeLocaleEx(HWND hWnd,
//                     LCID Lcid,
//                     LPCWSTR lpSrcRootPath,
//                     char Unknown,
//                     DWORD dwUnused1,
//                     DWORD dwUnused2)
// {
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// DWORD WINAPI
// SetupChangeLocale(HWND hWnd, LCID Lcid)
// {
//     return SetupChangeLocaleEx(hWnd, Lcid, NULL, 0, 0, 0);
// }
//
//
// DWORD
// WINAPI
// SetupStartService(
//     LPCWSTR lpServiceName,
//     BOOL bWait)
// {
//     SC_HANDLE hManager = NULL;
//     SC_HANDLE hService = NULL;
//     DWORD dwError = ERROR_SUCCESS;
//
//     hManager = OpenSCManagerW(NULL,
//                               NULL,
//                               SC_MANAGER_ALL_ACCESS);
//     if (hManager == NULL)
//     {
//         dwError = GetLastError();
//         goto done;
//     }
//
//     hService = OpenServiceW(hManager,
//                             lpServiceName,
//                             SERVICE_START);
//     if (hService == NULL)
//     {
//         dwError = GetLastError();
//         goto done;
//     }
//
//     if (!StartService(hService, 0, NULL))
//     {
//         dwError = GetLastError();
//         goto done;
//     }
//
// done:
//     if (hService != NULL)
//         CloseServiceHandle(hService);
//
//     if (hManager != NULL)
//         CloseServiceHandle(hManager);
//
//     return dwError;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
