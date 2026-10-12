use crate::fsutil::common::FindHandler;
use crate::fsutil::common::OpenVolume;
use crate::fsutil::common::PrintDefaultUsage;
use crate::fsutil::common::PrintErrorMessage;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_08AFF42C5A62: [u16; 6] = [0x0071u16, 0x0075, 0x0065, 0x0072, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_A5207BF78705: [u16; 18] = [
    0x0053u16, 0x0065, 0x0074, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0064, 0x0069,
    0x0072, 0x0074, 0x0079, 0x0020, 0x0062, 0x0069, 0x0074, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_E3190BF6C52F: [u16; 19] = [
    0x0053u16, 0x0068, 0x006F, 0x0077, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0064,
    0x0069, 0x0072, 0x0074, 0x0079, 0x0020, 0x0062, 0x0069, 0x0074, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_FCABAF3CA758: [u16; 4] = [0x0073u16, 0x0065, 0x0074, 0x0000];

static mut HandlersList: [HandlerItem; 2] = unsafe {
    [
        HandlerItem {
            Handler: Some(
                QueryMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_08AFF42C5A62.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_E3190BF6C52F.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                SetMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_FCABAF3CA758.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_A5207BF78705.as_ptr() as *const TCHAR,
        },
    ]
};

// `static HandlerProc QueryMain;` and `static HandlerProc SetMain;` are forward
// prototypes only; the Rust functions of those names are defined elsewhere in the crate.

// Raw pointers are not Sync, so the table must be `static mut`.

unsafe extern "C" fn QueryMain(mut argc: ::core::ffi::c_int, mut argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut Volume: HANDLE = core::ptr::null_mut();
    let mut VolumeStatus: ULONG = 0;
    let mut BytesRead: ULONG = 0;

    /* We need a volume (letter or GUID) */
    if argc < 2 {
        _ftprintf(__iob_func().add(2), (&[0x0055, 0x0073, 0x0061, 0x0067, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0064, 0x0069, 0x0072, 0x0074, 0x0079, 0x0020, 0x0071, 0x0075, 0x0065, 0x0072, 0x0079, 0x0020, 0x003c, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x003e, 0x000a, 0x0000u16]).as_ptr() /* _T "Usage: fsutil dirty query <volume>\n" */);
        _ftprintf(__iob_func().add(2), (&[0x0009, 0x0046, 0x006f, 0x0072, 0x0020, 0x0065, 0x0078, 0x0061, 0x006d, 0x0070, 0x006c, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0064, 0x0069, 0x0072, 0x0074, 0x0079, 0x0020, 0x0071, 0x0075, 0x0065, 0x0072, 0x0079, 0x0020, 0x0063, 0x003a, 0x000a, 0x0000u16]).as_ptr() /* _T "\tFor example: fsutil dirty query c:\n" */);
        return 1;
    }

    /* Get a handle for the volume */
    Volume = OpenVolume(
        *argv.offset(1 as isize),
        FALSE as BOOLEAN,
        FALSE as BOOLEAN,
    );
    if Volume == INVALID_HANDLE_VALUE {
        return 1;
    }

    /* And query the dirty status */
    if DeviceIoControl(
        Volume,
        FSCTL_IS_VOLUME_DIRTY as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(VolumeStatus) as LPVOID,
        core::mem::size_of::<ULONG>() as DWORD,
        core::ptr::addr_of_mut!(BytesRead),
        core::ptr::null_mut(),
    ) == (FALSE as i32)
    {
        PrintErrorMessage(GetLastError());
        CloseHandle(Volume);
        return 1;
    }

    CloseHandle(Volume);

    /* Print the status */
    _ftprintf(
        __iob_func().add(1),
        (&[0x0054, 0x0068, 0x0065, 0x0020, 0x0025, 0x0073, 0x0020, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0069, 0x0073, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "The %s volume is %s\n" */,
        *argv.offset(1 as isize),
        if (VolumeStatus & VOLUME_IS_DIRTY as ULONG) != 0 {
            (&[0x0064, 0x0069, 0x0072, 0x0074, 0x0079, 0x0000u16]).as_ptr() /* _T "dirty" */
        } else {
            (&[0x0063, 0x006c, 0x0065, 0x0061, 0x006e, 0x0000u16]).as_ptr() /* _T "clean" */
        },
    );

    return 0;
}

unsafe extern "C" fn SetMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut Volume: HANDLE = core::ptr::null_mut();
    let mut BytesRead: DWORD = 0;

    /* We need a volume (letter or GUID) */
    if argc < 2 {
        _ftprintf(__iob_func().add(2), (&[0x0055, 0x0073, 0x0061, 0x0067, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0064, 0x0069, 0x0072, 0x0074, 0x0079, 0x0020, 0x0073, 0x0065, 0x0074, 0x0020, 0x003c, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x003e, 0x000a, 0x0000u16]).as_ptr() /* _T "Usage: fsutil dirty set <volume>\n" */);
        _ftprintf(__iob_func().add(2), (&[0x0009, 0x0046, 0x006f, 0x0072, 0x0020, 0x0065, 0x0078, 0x0061, 0x006d, 0x0070, 0x006c, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0064, 0x0069, 0x0072, 0x0074, 0x0079, 0x0020, 0x0073, 0x0065, 0x0074, 0x0020, 0x0063, 0x003a, 0x000a, 0x0000u16]).as_ptr() /* _T "\tFor example: fsutil dirty set c:\n" */);
        return 1;
    }

    /* Get a handle for the volume */
    Volume = OpenVolume(
        *argv.offset(1 as isize),
        (FALSE) as BOOLEAN,
        (FALSE) as BOOLEAN,
    );
    if Volume == (INVALID_HANDLE_VALUE) {
        return 1;
    }

    /* And set the dirty bit */
    if DeviceIoControl(
        Volume,
        (FSCTL_MARK_VOLUME_DIRTY) as DWORD,
        core::ptr::null_mut(),
        (0) as DWORD,
        core::ptr::null_mut(),
        (0) as DWORD,
        core::ptr::addr_of_mut!(BytesRead),
        core::ptr::null_mut(),
    ) == (FALSE as i32)
    {
        PrintErrorMessage(GetLastError());
        CloseHandle(Volume);
        return 1;
    }

    CloseHandle(Volume);

    /* Print the status */
    _ftprintf(
        __iob_func().add(1),
        (&[0x0054, 0x0068, 0x0065, 0x0020, 0x0025, 0x0073, 0x0020, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0069, 0x0073, 0x0020, 0x006e, 0x006f, 0x0077, 0x0020, 0x006d, 0x0061, 0x0072, 0x006b, 0x0065, 0x0064, 0x0020, 0x0061, 0x0073, 0x0020, 0x0064, 0x0069, 0x0072, 0x0074, 0x0079, 0x000a, 0x0000u16]).as_ptr() /* _T "The %s volume is now marked as dirty\n" */,
        *argv.offset(1 as isize),
    );

    return 0;
}

unsafe extern "C" fn PrintUsage(Command: *const TCHAR) {
    let DIRTY: [u16; 8] = [' ' as u16, 'D' as u16, 'I' as u16, 'R' as u16, 'T' as u16, 'Y' as u16, ' ' as u16, 0];
    PrintDefaultUsage(
        DIRTY.as_ptr() as *const TCHAR,
        Command,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&(*core::ptr::addr_of!(HandlersList))[0]))
            as core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DirtyMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    return FindHandler(
        argc,
        argv,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&(*core::ptr::addr_of!(HandlersList))[0]))
            as ::core::ffi::c_int,
        Some(PrintUsage),
    );
}
