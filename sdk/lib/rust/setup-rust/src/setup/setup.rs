// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1C2EE8D80EB5: [u16; 13] = [
    0x0073u16, 0x0079, 0x0073, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_9C37FC2BBBBE: [u16; 9] = [0x006Eu16, 0x0065, 0x0077, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_FC4FB5C38D8E: [u16; 5] = [0x006Du16, 0x0069, 0x006E, 0x0069, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PINSTALL_REACTOS = Option<unsafe extern "system" fn(argc: i32, argv: *mut *mut u16) -> i32>;

unsafe extern "C" fn RunInstallReactOS(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut RetVal: INT;
    let mut hDll: HMODULE;
    let mut InstallReactOS: PINSTALL_REACTOS;

    hDll = LoadLibraryW(RUSTCONV_WSTR_1C2EE8D80EB5.as_ptr()) as HMODULE;
    if hDll.is_null() {
        DPRINT!("Failed to load 'syssetup.dll'!\n");
        return GetLastError() as INT;
    }
    DPRINT!("Loaded 'syssetup.dll'!\n");

    /* Call the standard Windows-compatible export */
    InstallReactOS = core::mem::transmute::<FARPROC, PINSTALL_REACTOS>(
        GetProcAddress(hDll, b"InstallWindowsNt\0".as_ptr() as *const i8),
    );
    if let Some(install) = InstallReactOS {
        RetVal = install(argc, argv);
    } else {
        RetVal = GetLastError() as INT;
        DPRINT!("Failed to get address for 'InstallWindowsNt()'!\n");
    }

    return RetVal;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut CmdLine: LPWSTR;
    let mut p: LPWSTR;

    // NOTE: Temporary, until we correctly use argc/argv.
    CmdLine = GetCommandLineW();
    DPRINT!("CmdLine: <%S>\n", CmdLine);

    p = wcschr(CmdLine as *const u16, 0x002Du16) as LPWSTR;
    if p.is_null() {
        return ERROR_INVALID_PARAMETER as INT;
    }
    p = p.add(1);

    // NOTE: On Windows, "mini" means "minimal UI", and can be used
    // in addition to "newsetup"; these options are not exclusive.
    if _wcsicmp(p as *const u16, RUSTCONV_WSTR_9C37FC2BBBBE.as_ptr()) == 0
        || _wcsicmp(p as *const u16, RUSTCONV_WSTR_FC4FB5C38D8E.as_ptr()) == 0
    {
        RunInstallReactOS(argc, argv);
    }

    return 0;
}
