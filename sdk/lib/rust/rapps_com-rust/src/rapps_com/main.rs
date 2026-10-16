// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7ABCE230ED71: [u16; 5] = [0x002Eu16, 0x0065, 0x0078, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_A27DB0BA3B27: [u16; 6] = [0x0020u16, 0x0022, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_E21FC3733631: [u16; 5] = [0x002Eu16, 0x0063, 0x006F, 0x006D, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn run_rapps(mut cmdline: LPWSTR) -> ::core::ffi::c_int {
    let mut dwExit: DWORD = 0;
    let mut si: STARTUPINFOW = core::mem::zeroed();
    si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

    SetLastError(0 as DWORD);
    if CreateProcessW(
        core::ptr::null::<wchar_t>(),
        cmdline,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        0 as DWORD,
        core::ptr::null_mut(),
        core::ptr::null::<wchar_t>(),
        core::ptr::addr_of_mut!(si),
        core::ptr::addr_of_mut!(pi),
    ) == 0
    {
        fprintf(
            (__iob_func() as *mut FILE).offset(2) as *mut FILE,
            b"Unable to create rapps.exe process...\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return -(1 as ::core::ffi::c_int);
    }
    CloseHandle(pi.hThread);
    WaitForSingleObject(pi.hProcess, INFINITE as DWORD);
    GetExitCodeProcess(pi.hProcess, core::ptr::addr_of_mut!(dwExit));
    CloseHandle(pi.hProcess);
    return dwExit as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: ::core::ffi::c_int, mut argv: *mut *mut wchar_t) -> ::core::ffi::c_int {
    let mut iRet: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut arglen: size_t = 0;
    let mut RappsExe: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut cmdline: *mut wchar_t = ::core::ptr::null_mut::<wchar_t>();

    GetModuleFileNameW(
        ::core::ptr::null_mut::<HINSTANCE__>(),
        core::ptr::addr_of_mut!(RappsExe) as LPWSTR,
        ARRAYSIZE!(RappsExe) as DWORD,
    );
    arglen = wcslen(core::ptr::addr_of_mut!(RappsExe) as *mut WCHAR) as size_t;
    if arglen > 4 as size_t
        && _wcsicmp(
            (core::ptr::addr_of_mut!(RappsExe) as *mut WCHAR)
                .offset(arglen as isize)
                .offset(-(4 as isize)),
            RUSTCONV_WSTR_E21FC3733631.as_ptr(),
        ) == 0
    {
        wcscpy(
            (core::ptr::addr_of_mut!(RappsExe) as *mut WCHAR)
                .offset(arglen as isize)
                .offset(-(4 as isize)),
            RUSTCONV_WSTR_7ABCE230ED71.as_ptr(),
        );
    } else {
        fprintf(
            __iob_func().add(2),
            b"Unable to build rapps.exe path...\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return -(1 as ::core::ffi::c_int);
    }

    arglen = arglen.wrapping_add((1 as size_t) + (2 as size_t));

    n = 1 as ::core::ffi::c_int;
    while n < argc {
        arglen = arglen.wrapping_add(wcslen(*argv.offset(n as isize)) as size_t);
        arglen = arglen.wrapping_add(3 as size_t);
        n += 1;
    }

    cmdline = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (arglen.wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t)) as SIZE_T,
    ) as *mut wchar_t;
    if !cmdline.is_null() {
        let mut ptr: *mut wchar_t = cmdline;
        let mut cchRemaining: size_t = arglen;

        StringCchPrintfExW!(
            ptr as STRSAFE_LPWSTR,
            cchRemaining,
            core::ptr::addr_of_mut!(ptr),
            core::ptr::addr_of_mut!(cchRemaining),
            0 as STRSAFE_DWORD,
            RUSTCONV_WSTR_9896841D0A6A.as_ptr() as STRSAFE_LPCWSTR,
            core::ptr::addr_of_mut!(RappsExe) as *mut WCHAR,
        );

        n = 1 as ::core::ffi::c_int;
        while n < argc {
            StringCchPrintfExW!(
                ptr as STRSAFE_LPWSTR,
                cchRemaining,
                core::ptr::addr_of_mut!(ptr),
                core::ptr::addr_of_mut!(cchRemaining),
                0 as STRSAFE_DWORD,
                RUSTCONV_WSTR_A27DB0BA3B27.as_ptr() as STRSAFE_LPCWSTR,
                *argv.offset(n as isize),
            );
            n += 1;
        }
    }

    iRet = run_rapps(cmdline as LPWSTR);
    if !cmdline.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            cmdline as PVOID,
        );
    }
    return iRet;
}
