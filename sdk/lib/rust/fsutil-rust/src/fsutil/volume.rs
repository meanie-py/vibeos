use crate::fsutil::common::FindHandler;
use crate::fsutil::common::OpenVolume;
use crate::fsutil::common::PrintDefaultUsage;
use crate::fsutil::common::PrintErrorMessage;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_066948C425FB: [u16; 19] = [
    0x0044u16, 0x0069, 0x0073, 0x006D, 0x006F, 0x0075, 0x006E, 0x0074, 0x0073, 0x0020,
    0x0061, 0x0020, 0x0076, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_0E9A807B7FD0: [u16; 30] = [
    0x0050u16, 0x0072, 0x0069, 0x006E, 0x0074, 0x0073, 0x0020, 0x0066, 0x0072, 0x0065,
    0x0065, 0x0020, 0x0073, 0x0070, 0x0061, 0x0063, 0x0065, 0x0020, 0x006F, 0x006E,
    0x0020, 0x0061, 0x0020, 0x0076, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_15B4329BA100: [u16; 9] = [0x0064u16, 0x0069, 0x0073, 0x006B, 0x0066, 0x0072, 0x0065, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_E8F3862CD9F2: [u16; 9] = [0x0064u16, 0x0069, 0x0073, 0x006D, 0x006F, 0x0075, 0x006E, 0x0074, 0x0000];

static mut HandlersList: [HandlerItem; 2] = unsafe {
    [
        HandlerItem {
            Handler: Some(
                DismountMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_E8F3862CD9F2.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_066948C425FB.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                DiskFreeMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_15B4329BA100.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_0E9A807B7FD0.as_ptr() as *const TCHAR,
        },
    ]
};

#[allow(non_snake_case)]
#[repr(C)]
pub struct HandlerItem {
    pub Proc: HandlerProc,
    pub name: *const TCHAR,
    pub help: *const TCHAR,
}

unsafe impl Sync for HandlerItem {}

pub static DISMOUNT_NAME_STR: [TCHAR; 9] = [
    'd' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'm' as TCHAR,
    'o' as TCHAR, 'u' as TCHAR, 'n' as TCHAR, 't' as TCHAR,
    0 as TCHAR,
];

pub static DISMOUNT_HELP_STR: [TCHAR; 19] = [
    'D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'm' as TCHAR,
    'o' as TCHAR, 'u' as TCHAR, 'n' as TCHAR, 't' as TCHAR,
    's' as TCHAR, ' ' as TCHAR, 'a' as TCHAR, ' ' as TCHAR,
    'v' as TCHAR, 'o' as TCHAR, 'l' as TCHAR, 'u' as TCHAR,
    'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR,
];

pub static DISKFREE_NAME_STR: [TCHAR; 9] = [
    'd' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'k' as TCHAR,
    'f' as TCHAR, 'r' as TCHAR, 'e' as TCHAR, 'e' as TCHAR,
    0 as TCHAR,
];

pub static DISKFREE_HELP_STR: [TCHAR; 30] = [
    'P' as TCHAR, 'r' as TCHAR, 'i' as TCHAR, 'n' as TCHAR,
    't' as TCHAR, 's' as TCHAR, ' ' as TCHAR, 'f' as TCHAR,
    'r' as TCHAR, 'e' as TCHAR, 'e' as TCHAR, ' ' as TCHAR,
    's' as TCHAR, 'p' as TCHAR, 'a' as TCHAR, 'c' as TCHAR,
    'e' as TCHAR, ' ' as TCHAR, 'o' as TCHAR, 'n' as TCHAR,
    ' ' as TCHAR, 'a' as TCHAR, ' ' as TCHAR, 'v' as TCHAR,
    'o' as TCHAR, 'l' as TCHAR, 'u' as TCHAR, 'm' as TCHAR,
    'e' as TCHAR, 0 as TCHAR,
];

unsafe extern "C" fn LockOrUnlockVolume(mut Volume: HANDLE, mut Lock: BOOLEAN) -> ::core::ffi::c_int {
    let mut Operation: DWORD = 0;
    let mut BytesRead: ULONG = 0;
    Operation = (if Lock as ::core::ffi::c_int != 0 {
        FSCTL_LOCK_VOLUME
    } else {
        FSCTL_UNLOCK_VOLUME
    }) as DWORD;
    if DeviceIoControl(
        Volume,
        Operation,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(BytesRead),
        core::ptr::null_mut(),
    ) == (FALSE as i32)
    {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    return 0;
}

unsafe extern "C" fn DismountMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut Volume: HANDLE = core::ptr::null_mut();
    let mut BytesRead: ULONG = 0;

    /* We need a volume (letter or GUID) */
    if argc < 2 {
        _ftprintf(__iob_func().add(2), (&[0x0055, 0x0073, 0x0061, 0x0067, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0064, 0x0069, 0x0073, 0x006d, 0x006f, 0x0075, 0x006e, 0x0074, 0x0020, 0x003c, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x003e, 0x000a, 0x0000u16]).as_ptr() /* _T "Usage: fsutil volume dismount <volume>\n" */);
        _ftprintf(__iob_func().add(2), (&[0x0009, 0x0046, 0x006f, 0x0072, 0x0020, 0x0065, 0x0078, 0x0061, 0x006d, 0x0070, 0x006c, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0064, 0x0069, 0x0073, 0x006d, 0x006f, 0x0075, 0x006e, 0x0074, 0x0020, 0x0064, 0x003a, 0x000a, 0x0000u16]).as_ptr() /* _T "\tFor example: fsutil volume dismount d:\n" */);
        return 1;
    }

    /* Get a handle for the volume */
    Volume = OpenVolume(*argv.offset(1 as isize), (FALSE) as BOOLEAN, (FALSE) as BOOLEAN);
    if Volume == INVALID_HANDLE_VALUE {
        return 1;
    }

    /* Attempt to lock the volume */
    if LockOrUnlockVolume(Volume, (TRUE) as BOOLEAN) != 0 {
        CloseHandle(Volume);
        return 1;
    }

    /* Issue the dismount command */
    if DeviceIoControl(Volume, FSCTL_DISMOUNT_VOLUME as DWORD, core::ptr::null_mut(), 0 as DWORD, core::ptr::null_mut(),
                        0 as DWORD, core::ptr::addr_of_mut!(BytesRead), core::ptr::null_mut()) == (FALSE as i32) {
        PrintErrorMessage(GetLastError());
        LockOrUnlockVolume(Volume, (FALSE) as BOOLEAN);
        CloseHandle(Volume);
        return 1;
    }

    /* Unlock and quit */
    LockOrUnlockVolume(Volume, (FALSE) as BOOLEAN);
    CloseHandle(Volume);

    _ftprintf(__iob_func().add(1), (&[0x0054, 0x0068, 0x0065, 0x0020, 0x0025, 0x0073, 0x0020, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0068, 0x0061, 0x0073, 0x0020, 0x0062, 0x0065, 0x0065, 0x006e, 0x0020, 0x0064, 0x0069, 0x0073, 0x006d, 0x006f, 0x0075, 0x006e, 0x0074, 0x0065, 0x0064, 0x000a, 0x0000u16]).as_ptr() /* _T "The %s volume has been dismounted\n" */, *argv.offset(1 as isize));

    return 0;
}

unsafe extern "C" fn DiskFreeMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    let mut FreeAvail: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalNumber: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalNumberFree: ULARGE_INTEGER = core::mem::zeroed();

    /* We need a path */
    if argc < 2 {
        _ftprintf(__iob_func().add(2), (&[0x0055, 0x0073, 0x0061, 0x0067, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0064, 0x0069, 0x0073, 0x006b, 0x0066, 0x0072, 0x0065, 0x0065, 0x0020, 0x003c, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0070, 0x0061, 0x0074, 0x0068, 0x003e, 0x000a, 0x0000u16]).as_ptr() /* _T "Usage: fsutil volume diskfree <volume path>\n" */);
        _ftprintf(__iob_func().add(2), (&[0x0009, 0x0046, 0x006f, 0x0072, 0x0020, 0x0065, 0x0078, 0x0061, 0x006d, 0x0070, 0x006c, 0x0065, 0x003a, 0x0020, 0x0066, 0x0073, 0x0075, 0x0074, 0x0069, 0x006c, 0x0020, 0x0076, 0x006f, 0x006c, 0x0075, 0x006d, 0x0065, 0x0020, 0x0064, 0x0069, 0x0073, 0x006b, 0x0066, 0x0072, 0x0065, 0x0065, 0x0020, 0x0063, 0x003a, 0x000a, 0x0000u16]).as_ptr() /* _T "\tFor example: fsutil volume diskfree c:\n" */);
        return 1;
    }

    if !(GetDiskFreeSpaceEx(
        *argv.offset(1 as isize),
        core::ptr::addr_of_mut!(FreeAvail),
        core::ptr::addr_of_mut!(TotalNumber),
        core::ptr::addr_of_mut!(TotalNumberFree),
    ) != 0)
    {
        PrintErrorMessage(GetLastError());
        return 1;
    }

    _ftprintf(__iob_func().add(1), (&[0x0054, 0x006f, 0x0074, 0x0061, 0x006c, 0x0020, 0x0066, 0x0072, 0x0065, 0x0065, 0x0020, 0x0062, 0x0079, 0x0074, 0x0065, 0x0073, 0x0009, 0x0009, 0x003a, 0x0020, 0x0025, 0x0049, 0x0036, 0x0034, 0x0069, 0x000a, 0x0000u16]).as_ptr() /* _T "Total free bytes\t\t: %I64i\n" */, TotalNumberFree.QuadPart);
    _ftprintf(__iob_func().add(1), (&[0x0054, 0x006f, 0x0074, 0x0061, 0x006c, 0x0020, 0x0062, 0x0079, 0x0074, 0x0065, 0x0073, 0x0009, 0x0009, 0x0009, 0x003a, 0x0020, 0x0025, 0x0049, 0x0036, 0x0034, 0x0069, 0x000a, 0x0000u16]).as_ptr() /* _T "Total bytes\t\t\t: %I64i\n" */, TotalNumber.QuadPart);
    _ftprintf(__iob_func().add(1), (&[0x0054, 0x006f, 0x0074, 0x0061, 0x006c, 0x0020, 0x0066, 0x0072, 0x0065, 0x0065, 0x0020, 0x0061, 0x0076, 0x0061, 0x0069, 0x006c, 0x0061, 0x0062, 0x006c, 0x0065, 0x0020, 0x0062, 0x0079, 0x0074, 0x0065, 0x0073, 0x0009, 0x003a, 0x0020, 0x0025, 0x0049, 0x0036, 0x0034, 0x0069, 0x000a, 0x0000u16]).as_ptr() /* _T "Total free available bytes\t: %I64i\n" */, FreeAvail.QuadPart);

    return 0;
}

unsafe extern "C" fn PrintUsage(Command: *const TCHAR) {
    PrintDefaultUsage(
        [' ' as u16, 'V' as u16, 'O' as u16, 'L' as u16, 'U' as u16, 'M' as u16, 'E' as u16, ' ' as u16, 0u16].as_ptr() as *const TCHAR,
        Command,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&(*core::ptr::addr_of!(HandlersList))[0])) as i32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn VolumeMain(argc: ::core::ffi::c_int, argv: *mut *const TCHAR) -> ::core::ffi::c_int {
    return FindHandler(
        argc,
        argv,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList[0])))
            as i32,
        Some(PrintUsage as unsafe extern "C" fn(*const TCHAR) -> ()),
    );
}
