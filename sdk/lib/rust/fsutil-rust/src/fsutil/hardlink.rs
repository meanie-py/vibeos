use crate::fsutil::common::FindHandler;
use crate::fsutil::common::PrintDefaultUsage;
use crate::fsutil::common::PrintErrorMessage;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_675522AA76ED: [u16; 7] = [0x0063u16, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_87CCDCA96087: [u16; 23] = [
    0x0043u16, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x0061, 0x0020, 0x006E,
    0x0065, 0x0077, 0x0020, 0x0068, 0x0061, 0x0072, 0x0064, 0x0020, 0x006C, 0x0069,
    0x006E, 0x006B, 0x0000,
];

static mut HandlersList: [HandlerItem; 1] = unsafe {
    [HandlerItem {
        Handler: Some(
            CreateMain
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *const TCHAR,
                ) -> ::core::ffi::c_int,
        ),
        Command: RUSTCONV_C2R_WSTR_675522AA76ED.as_ptr() as *const TCHAR,
        Desc: RUSTCONV_C2R_WSTR_87CCDCA96087.as_ptr() as *const TCHAR,
    }]
};

static CREATE_NAME: [TCHAR; 7] = [
    b'c' as TCHAR, b'r' as TCHAR, b'e' as TCHAR, b'a' as TCHAR, b't' as TCHAR, b'e' as TCHAR, 0 as TCHAR,
];

static CREATE_HELP: [TCHAR; 23] = [
    b'C' as TCHAR, b'r' as TCHAR, b'e' as TCHAR, b'a' as TCHAR, b't' as TCHAR, b'e' as TCHAR, b' ' as TCHAR,
    b'a' as TCHAR, b' ' as TCHAR, b'n' as TCHAR, b'e' as TCHAR, b'w' as TCHAR, b' ' as TCHAR,
    b'h' as TCHAR, b'a' as TCHAR, b'r' as TCHAR, b'd' as TCHAR, b' ' as TCHAR,
    b'l' as TCHAR, b'i' as TCHAR, b'n' as TCHAR, b'k' as TCHAR, 0 as TCHAR,
];

unsafe extern "C" fn CreateMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut Source: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Target: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* We need a source and a destination */
    if argc < 3 {
        _ftprintf(__iob_func().add(2), (&[0x0055, 0x0073, 0x0061, 0x0067, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0068, 0x0061, 0x0072, 0x0064, 0x006c, 0x0069, 0x006e, 0x006b, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x003c, 0x006e, 0x0065, 0x0077, 0x003e, 0x0020, 0x003c, 0x0065, 0x0078, 0x0069, 0x0073, 0x0074, 0x0069, 0x006e, 0x0067, 0x003e, 0x000a, 0x0000u16]).as_ptr() /* _T "Usage: fsutil hardlink create <new> <existing>\n" */);
        _ftprintf(__iob_func().add(2), (&[0x0009, 0x0046, 0x006f, 0x0072, 0x0020, 0x0065, 0x0078, 0x0061, 0x006d, 0x0070, 0x006c, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0068, 0x0061, 0x0072, 0x0064, 0x006c, 0x0069, 0x006e, 0x006b, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x0063, 0x003a, 0x005c, 0x0074, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x002e, 0x0074, 0x0078, 0x0074, 0x0020, 0x0063, 0x003a, 0x005c, 0x0073, 0x006f, 0x0075, 0x0072, 0x0063, 0x0065, 0x002e, 0x0074, 0x0078, 0x0074, 0x000a, 0x0000u16]).as_ptr() /* _T "\tFor example: fsutil hardlink create c:\\target.txt c:\\sou" */);
        return 1;
    }

    /* Get full names */
    if GetFullPathName(*argv.offset(1 as isize), MAX_PATH, core::ptr::addr_of_mut!(Target) as *mut TCHAR, core::ptr::null_mut()) == 0 {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    if GetFullPathName(*argv.offset(2 as isize), MAX_PATH, core::ptr::addr_of_mut!(Source) as *mut TCHAR, core::ptr::null_mut()) == 0 {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    /* Simply delegate to kernel32 */
    if !CreateHardLink(core::ptr::addr_of_mut!(Target) as *mut TCHAR, core::ptr::addr_of_mut!(Source) as *mut TCHAR, core::ptr::null_mut()) != 0 {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    /* Print the status */
    _ftprintf(__iob_func().add(1), (&[0x0048, 0x0061, 0x0072, 0x0064, 0x0020, 0x006c, 0x0069, 0x006e, 0x006b, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0064, 0x0020, 0x0066, 0x006f, 0x0072, 0x0020, 0x0025, 0x0073, 0x0020, 0x003c, 0x003d, 0x003e, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "Hard link created for %s <=> %s\n" */, core::ptr::addr_of_mut!(Target) as *mut TCHAR, core::ptr::addr_of_mut!(Source) as *mut TCHAR);

    return 0;
}

unsafe extern "C" fn PrintUsage(Command: *const TCHAR) {
    let HlWide: [u16; 11] = [
        ' ' as u16, 'H' as u16, 'A' as u16, 'R' as u16, 'D' as u16, 'L' as u16,
        'I' as u16, 'N' as u16, 'K' as u16, ' ' as u16, 0,
    ];
    PrintDefaultUsage(
        HlWide.as_ptr() as *const TCHAR,
        Command,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&(*core::ptr::addr_of!(HandlersList))[0]))
            as core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HardLinkMain(mut argc: ::core::ffi::c_int, mut argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    return FindHandler(
        argc,
        argv,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            .wrapping_div(core::mem::size_of_val(
                &(*core::ptr::addr_of!(HandlersList))[0],
            ))) as ::core::ffi::c_int,
        Some(PrintUsage as unsafe extern "C" fn(*const TCHAR) -> ()),
    );
}
