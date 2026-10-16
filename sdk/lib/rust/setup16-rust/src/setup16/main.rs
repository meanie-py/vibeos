// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_025979B0E1F2: [u16; 40] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x006C, 0x006F, 0x0061, 0x0064, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0069,
    0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0065, 0x0072, 0x002E, 0x0020,
    0x0045, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_026F4F1399FF: [u16; 27] = [
    0x0043u16, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0066, 0x0069, 0x006E,
    0x0064, 0x0020, 0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x002E, 0x006C, 0x0073,
    0x0074, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_0926265E608D: [u16; 12] = [
    0x0043u16, 0x0061, 0x0062, 0x0069, 0x006E, 0x0065, 0x0074, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_24C20F2FD54B: [u16; 27] = [
    0x0043u16, 0x006F, 0x0075, 0x006C, 0x0064, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020,
    0x0065, 0x0078, 0x0074, 0x0072, 0x0061, 0x0063, 0x0074, 0x0020, 0x0063, 0x0061,
    0x0062, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_3CB0BA00DCE1: [u16; 10] = [0x0073u16, 0x0065, 0x0074, 0x0075, 0x0070, 0x002E, 0x006C, 0x0073, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_4110572CB0DF: [u16; 8] = [0x0043u16, 0x006D, 0x0064, 0x004C, 0x0069, 0x006E, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_6C49BEACD7CB: [u16; 6] = [0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_88DB3541CA09: [u16; 11] = [
    0x0054u16, 0x006D, 0x0070, 0x0044, 0x0069, 0x0072, 0x004E, 0x0061, 0x006D, 0x0065,
    0x0000,
];
pub static RUSTCONV_WSTR_97378A774D46: [u16; 33] = [
    0x0043u16, 0x006F, 0x0075, 0x006C, 0x0064, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020,
    0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x0054, 0x0065, 0x006D,
    0x0070, 0x0020, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072,
    0x0079, 0x002E, 0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static NT_PARAMS: [::core::ffi::c_ushort; 20] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_ushort; 20]>(
        *b"N\0T\x003\0.\x005\x001\0 \0I\0n\0t\0e\0l\0 \0P\0a\0r\0a\0m\0s\0\0\0",
    )
};

pub static MSSETUP_PATH: [::core::ffi::c_ushort; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_ushort; 12]>(
        *b"~\0M\0S\0S\0E\0T\0U\0P\0.\0T\0\\\0\0\0",
    )
};

// #define WIN32_NO_STATUS   (valueless flag; no Rust constant emitted)
// #define WIN32_LEAN_AND_MEAN   (valueless flag; no Rust constant emitted)
unsafe extern "system" fn ExtCabCallback(
    Context: PVOID,
    Notification: UINT,
    Param1: UINT_PTR,
    Param2: UINT_PTR,
) -> UINT {
    let mut pInfo: *mut FILE_IN_CABINET_INFO_W = core::ptr::null_mut();
    let mut pFilePaths: *mut FILEPATHS_W = core::ptr::null_mut();

    DBG_UNREFERENCED_LOCAL_VARIABLE!(pFilePaths);

    if Notification == SPFILENOTIFY_FILEINCABINET as UINT {
        pInfo = Param1 as *mut FILE_IN_CABINET_INFO_W;
        wcscpy(
            &raw mut (*pInfo).FullTargetName as *mut wchar_t,
            Context as *const wchar_t,
        );
        wcscat(
            &raw mut (*pInfo).FullTargetName as *mut wchar_t,
            (*pInfo).NameInCabinet as *const wchar_t,
        );
        return FILEOP_DOIT as UINT;
    } else if Notification == SPFILENOTIFY_FILEEXTRACTED as UINT {
        pFilePaths = Param1 as *mut FILEPATHS_W;
        return NO_ERROR as UINT;
    }
    return NO_ERROR as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteDirectory(lpszDir: LPWSTR) -> BOOL {
    let mut fileop: SHFILEOPSTRUCT = core::mem::zeroed();
    let len: DWORD = wcslen(lpszDir as *const wchar_t) as DWORD;
    let pszFrom: *mut WCHAR = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((len + 2 as DWORD) as SIZE_T)
            * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    let mut ret: i32;

    wcscpy(pszFrom as *mut wchar_t, lpszDir as *const wchar_t);
    *pszFrom.offset(len as isize) = 0 as WCHAR;
    *pszFrom.offset((len + 1 as DWORD) as isize) = 0 as WCHAR;

    fileop.hwnd = core::ptr::null_mut();
    fileop.wFunc = FO_DELETE as UINT;
    fileop.pFrom = pszFrom as LPCWSTR;
    fileop.pTo = core::ptr::null();
    fileop.fFlags = (FOF_NOCONFIRMATION | FOF_SILENT) as FILEOP_FLAGS;
    fileop.fAnyOperationsAborted = FALSE as BOOL;
    fileop.lpszProgressTitle = core::ptr::null();
    fileop.hNameMappings = core::ptr::null_mut();

    ret = SHFileOperation(&raw mut fileop);
    HeapFree(GetProcessHeap(), 0 as DWORD, pszFrom as LPVOID);
    return (ret == 0 as i32) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetSystemDrive(lpszDrive: LPWSTR) {
    let mut szWindir: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    GetWindowsDirectoryW(core::ptr::addr_of_mut!(szWindir) as *mut WCHAR, MAX_PATH as UINT);
    _wsplitpath(core::ptr::addr_of_mut!(szWindir) as *mut WCHAR, lpszDrive, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    wcscat(lpszDrive, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32 {
    let mut szSetupPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szCabFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szCabFilePath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szTempDirName: [WCHAR; 50] = [0; 50];
    let mut szCmdLine: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szTempCmdLine: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szTempPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFullTempPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szDrive: [WCHAR; 4] = [0; 4];
    let mut dwAttrib: DWORD;
    let mut processInfo: PROCESS_INFORMATION = core::mem::zeroed();
    let mut startupInfo: STARTUPINFO = core::mem::zeroed();
    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(nCmdShow);

    GetCurrentDirectory(MAX_PATH, szSetupPath.as_mut_ptr());
    wcscat(szSetupPath.as_mut_ptr(), RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    wcscpy(szFileName.as_mut_ptr(), szSetupPath.as_mut_ptr());
    wcscat(szFileName.as_mut_ptr(), RUSTCONV_WSTR_3CB0BA00DCE1.as_ptr());

    if GetFileAttributes(szFileName.as_mut_ptr()) == INVALID_FILE_ATTRIBUTES
    {
        MessageBoxW(core::ptr::null_mut(), RUSTCONV_WSTR_026F4F1399FF.as_ptr(), RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(), MB_OK | MB_ICONERROR);
        return 1;
    }

    /* read information from setup.lst */
    GetPrivateProfileStringW((NT_PARAMS).as_ptr(), RUSTCONV_WSTR_0926265E608D.as_ptr(), core::ptr::null_mut(), szCabFileName.as_mut_ptr(), MAX_PATH, szFileName.as_mut_ptr());
    GetPrivateProfileStringW((NT_PARAMS).as_ptr(), RUSTCONV_WSTR_88DB3541CA09.as_ptr(), core::ptr::null_mut(), szTempDirName.as_mut_ptr(), 50, szFileName.as_mut_ptr());
    GetPrivateProfileStringW((NT_PARAMS).as_ptr(), RUSTCONV_WSTR_4110572CB0DF.as_ptr(), core::ptr::null_mut(), szCmdLine.as_mut_ptr(), MAX_PATH, szFileName.as_mut_ptr());

    wcscpy(szCabFilePath.as_mut_ptr(), szSetupPath.as_mut_ptr());
    wcscat(szCabFilePath.as_mut_ptr(), szCabFileName.as_mut_ptr());

    /* ceate temp directory */
    GetSystemDrive(szDrive.as_mut_ptr());
    wcscpy(szTempPath.as_mut_ptr(), szDrive.as_mut_ptr());
    wcscat(szTempPath.as_mut_ptr(), (MSSETUP_PATH).as_ptr());
    wcscpy(szFullTempPath.as_mut_ptr(), szTempPath.as_mut_ptr());
    wcscat(szFullTempPath.as_mut_ptr(), szTempDirName.as_mut_ptr());
    wcscat(szFullTempPath.as_mut_ptr(), RUSTCONV_WSTR_B7FC02907B1F.as_ptr());

    if SHCreateDirectoryEx(core::ptr::null_mut(), szFullTempPath.as_mut_ptr(), core::ptr::null_mut()) != (ERROR_SUCCESS as i32)
    {
        MessageBoxW(core::ptr::null_mut(), RUSTCONV_WSTR_97378A774D46.as_ptr(), RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(), MB_OK | MB_ICONERROR);
        return 1;
    }

    dwAttrib = GetFileAttributes(szTempPath.as_mut_ptr());
    SetFileAttributes(szTempPath.as_mut_ptr(), dwAttrib | FILE_ATTRIBUTE_HIDDEN);

    /* extract files */
    if !(SetupIterateCabinetW(szCabFilePath.as_mut_ptr(), 0, Some(ExtCabCallback), szFullTempPath.as_mut_ptr() as PVOID) != 0)
    {
        MessageBoxW(core::ptr::null_mut(), RUSTCONV_WSTR_24C20F2FD54B.as_ptr(), RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(), MB_OK | MB_ICONERROR);
        DeleteDirectory(szTempPath.as_mut_ptr());
        return 1;
    }

    /* prepare command line */
    wsprintf(szTempCmdLine.as_mut_ptr(), szCmdLine.as_mut_ptr(), szFullTempPath.as_mut_ptr(), lpCmdLine);
    wcscpy(szCmdLine.as_mut_ptr(), szFullTempPath.as_mut_ptr());
    wcscat(szCmdLine.as_mut_ptr(), szTempCmdLine.as_mut_ptr());

    /* execute the 32-Bit installer */
    ZeroMemory!(core::ptr::addr_of_mut!(processInfo) as *mut core::ffi::c_void, core::mem::size_of::<PROCESS_INFORMATION>());
    ZeroMemory!(core::ptr::addr_of_mut!(startupInfo) as *mut core::ffi::c_void, core::mem::size_of::<STARTUPINFO>());
    startupInfo.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
    if CreateProcessW(core::ptr::null_mut(), szCmdLine.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as BOOL, NORMAL_PRIORITY_CLASS, core::ptr::null_mut(), szFullTempPath.as_mut_ptr(), core::ptr::addr_of_mut!(startupInfo), core::ptr::addr_of_mut!(processInfo)) != 0
    {
        WaitForSingleObject(processInfo.hProcess, INFINITE);
        CloseHandle(processInfo.hProcess);
        CloseHandle(processInfo.hThread);
    }
    else
    {
        let mut szMsg: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        wsprintf(szMsg.as_mut_ptr(), RUSTCONV_WSTR_025979B0E1F2.as_ptr(), GetLastError());
        MessageBoxW(core::ptr::null_mut(), szMsg.as_mut_ptr(), RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(), MB_OK | MB_ICONERROR);
        DeleteDirectory(szTempPath.as_mut_ptr());
        return 1;
    }

    /* cleanup */
    DeleteDirectory(szTempPath.as_mut_ptr());

    return 0;
}
