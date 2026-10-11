// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1F8DB2551537: [u16; 8] = [0x0043u16, 0x006F, 0x006D, 0x0053, 0x0070, 0x0065, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_F7728ED9A211: [u16; 12] = [
    0x0053u16, 0x0048, 0x0045, 0x004C, 0x004C, 0x0033, 0x0032, 0x002E, 0x0044, 0x004C,
    0x004C, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnFileNew() {
    let mut hShell32: HMODULE = core::ptr::null_mut();
    let mut RunFileDlg: RUNFILEDLG = None;
    let mut szTitle: [WCHAR; 40] = [0; 40];
    let mut szText: [WCHAR; 256] = [0; 256];

    /* If CTRL is held, start the user's command-line shell */
    if (GetAsyncKeyState(VK_CONTROL as i32) as i32 & 0x8000) != 0 {
        let mut si: STARTUPINFOW = core::mem::zeroed();
        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
        let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
        let mut szComSpec: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut fallbackCmd: [WCHAR; 8] = [0x0063u16, 0x006D, 0x0064, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
        let cchEnv: DWORD = GetEnvironmentVariableW(
            RUSTCONV_WSTR_1F8DB2551537.as_ptr(),
            core::ptr::addr_of_mut!(szComSpec) as LPWSTR,
            _countof!(szComSpec),
        );
        if cchEnv == 0 || cchEnv > _countof!(szComSpec) as DWORD {
            /* Couldn't get the environment variable, default to cmd.exe */
            wcscpy(
                core::ptr::addr_of_mut!(szComSpec) as *mut WCHAR,
                core::ptr::addr_of_mut!(fallbackCmd) as *mut WCHAR,
            );
        }
        let mut result: BOOL = CreateProcessW(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szComSpec) as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (FALSE) as BOOL,
            CREATE_NEW_CONSOLE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(si),
            core::ptr::addr_of_mut!(pi),
        );
        if result == 0 {
            /* Couldn't start the user's command-line shell, fall back to cmd.exe */
            result = CreateProcessW(
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(fallbackCmd) as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (FALSE) as BOOL,
                CREATE_NEW_CONSOLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(si),
                core::ptr::addr_of_mut!(pi),
            );
        }
        if result != 0 {
            CloseHandle(pi.hThread);
            CloseHandle(pi.hProcess);
        }
        return;
    }

    /* Load language strings from resource file */
    LoadStringW(
        hInst,
        IDS_CREATENEWTASK,
        core::ptr::addr_of_mut!(szTitle) as LPWSTR,
        (_countof!(szTitle)) as i32,
    );
    LoadStringW(
        hInst,
        IDS_CREATENEWTASK_DESC,
        core::ptr::addr_of_mut!(szText) as LPWSTR,
        (_countof!(szText)) as i32,
    );

    hShell32 = LoadLibraryW(RUSTCONV_WSTR_F7728ED9A211.as_ptr());
    RunFileDlg = core::mem::transmute::<FARPROC, RUNFILEDLG>(GetProcAddress(
        hShell32,
        61 as usize as LPCSTR,
    ));

    /* Show "Run..." dialog */
    if let Some(RunFileDlg) = RunFileDlg {
        let hIcon: HICON = LoadIconW(
            hInst,
            MAKEINTRESOURCEW!(IDI_TASKMANAGER),
        );

        /* NOTE - don't check whether running on win 9x or NT, let's just
                  assume that a unicode build only runs on NT */
        RunFileDlg(
            hMainWnd,
            hIcon,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR,
            RFF_CALCDIRECTORY,
        );

        DeleteObject((hIcon).cast());
    }

    FreeLibrary(hShell32);
}
