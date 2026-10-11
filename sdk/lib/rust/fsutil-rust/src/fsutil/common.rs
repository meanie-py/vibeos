use ntdll_sys::nt::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn FindHandler(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *const TCHAR,
    mut HandlersList: *mut HandlerItem,
    mut HandlerListCount: ::core::ffi::c_int,
    mut UsageHelper: Option<unsafe extern "C" fn(*const TCHAR) -> ()>,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = 0;
    let mut Command: *const TCHAR = ::core::ptr::null::<TCHAR>();
    ret = 1;
    Command = ::core::ptr::null::<TCHAR>();
    i = HandlerListCount;

    /* If we have a command, does it match a known one? */
    if argc > 1 {
        /* Browse all the known commands finding the right one */
        Command = *argv.offset(1 as isize);
        i = 0 as ::core::ffi::c_int;
        while i < HandlerListCount {
            if _tcsicmp(
                Command,
                (*HandlersList.offset(i as isize)).Command,
            ) == 0
            {
                ret = (*HandlersList.offset(i as isize))
                    .Handler
                    .expect("non-null function pointer")(
                    argc - 1,
                    argv.offset(1 as isize) as *mut *const TCHAR,
                );
                break;
            } else {
                i += 1;
            }
        }
    }

    /* We failed finding someone to handle the caller's needs, print out */
    if i == HandlerListCount {
        UsageHelper(Command);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn OpenVolume(
    Volume: *const TCHAR,
    AllowRemote: BOOLEAN,
    NtfsOnly: BOOLEAN,
) -> HANDLE {
    let mut Type: UINT = 0;
    let mut hVolume: HANDLE = core::ptr::null_mut();
    let mut VolumeID: [TCHAR; PATH_MAX] = [0; PATH_MAX];

    /* Get volume type */
    if AllowRemote == 0 && *Volume.add(1 as usize) == 0x003Au16 {
        Type = GetDriveType(Volume);
        if Type == DRIVE_REMOTE {
            _ftprintf(stderr, _T("FSUTIL needs a local device\n"));
            return INVALID_HANDLE_VALUE;
        }
    }

    /* Get filesystem type */
    if NtfsOnly != 0 {
        let mut FileSystem: [TCHAR; MAX_PATH + 1] =
            [0; MAX_PATH + 1];

        _stprintf(
            core::ptr::addr_of_mut!(VolumeID) as *mut TCHAR,
            _T("\\\\.\\%s\\"),
            Volume,
        );
        if GetVolumeInformation(
            core::ptr::addr_of_mut!(VolumeID) as *mut TCHAR,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FileSystem) as *mut TCHAR,
            MAX_PATH + 1,
        ) == 0
        {
            PrintErrorMessage(GetLastError());
            return INVALID_HANDLE_VALUE;
        }

        if _tcscmp(
            core::ptr::addr_of_mut!(FileSystem) as *mut TCHAR,
            _T("NTFS"),
        ) != 0
        {
            _ftprintf(stderr, _T("FSUTIL needs a NTFS device\n"));
            return INVALID_HANDLE_VALUE;
        }
    }

    /* Create full name */
    _stprintf(
        core::ptr::addr_of_mut!(VolumeID) as *mut TCHAR,
        _T("\\\\.\\%s"),
        Volume,
    );

    /* Open the volume */
    hVolume = CreateFile(
        core::ptr::addr_of_mut!(VolumeID) as *mut TCHAR,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hVolume == INVALID_HANDLE_VALUE {
        PrintErrorMessage(GetLastError());
        return INVALID_HANDLE_VALUE;
    }

    return hVolume;
}
#[no_mangle]
pub unsafe extern "C" fn PrintDefaultUsage(
    mut Command: *const TCHAR,
    mut SubCommand: *const TCHAR,
    mut HandlersList: *mut HandlerItem,
    mut HandlerListCount: ::core::ffi::c_int,
) {
    static UNHANDLED_FMT: [u16; 24] = [
        'U' as u16, 'n' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'd' as u16,
        'l' as u16, 'e' as u16, 'd' as u16, '%' as u16, 's' as u16, 'c' as u16,
        'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16,
        ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0,
    ];
    static HANDLED_FMT: [u16; 30] = [
        '-' as u16, '-' as u16, '-' as u16, '-' as u16, ' ' as u16, 'H' as u16,
        'a' as u16, 'n' as u16, 'd' as u16, 'l' as u16, 'e' as u16, 'd' as u16,
        '%' as u16, 's' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16,
        'a' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, '-' as u16,
        '-' as u16, '-' as u16, '-' as u16, '\n' as u16, '\n' as u16, 0,
    ];
    static ITEM_FMT: [u16; 7] = [
        '%' as u16, 's' as u16, '\t' as u16, '%' as u16, 's' as u16, '\n' as u16, 0,
    ];

    let mut i: ::core::ffi::c_int = 0;
    if !SubCommand.is_null() {
        _ftprintf(
            (&raw mut _iob as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
            _T(UNHANDLED_FMT.as_ptr()),
            Command,
            SubCommand,
        );
    }
    _ftprintf(
        (&raw mut _iob as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
        _T(HANDLED_FMT.as_ptr()),
        Command,
    );
    i = 0;
    while i < HandlerListCount {
        _ftprintf(
            (&raw mut _iob as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
            _T(ITEM_FMT.as_ptr()),
            (*HandlersList.offset(i as isize)).Command,
            (*HandlersList.offset(i as isize)).Desc,
        );
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintErrorMessage(mut Error: DWORD) -> ::core::ffi::c_int {
    let mut String: *mut TCHAR = ::core::ptr::null_mut::<TCHAR>();
    /* Try to get textual error */
    if FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM) as DWORD,
        ::core::ptr::null::<::core::ffi::c_void>(),
        Error,
        0 as DWORD,
        &raw mut String as LPWSTR,
        0 as DWORD,
        ::core::ptr::null_mut::<::core::ffi::VaListImpl>(),
    ) != 0 as DWORD
    {
        /* And print it */
        let fmt: [u16; 11] = [
            'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16,
            ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0,
        ];
        _ftprintf(
            (&raw mut _iob as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
            fmt.as_ptr(),
            String,
        );
        LocalFree(String as HLOCAL);
    } else {
        /* Otherwise, just print the error number */
        let fmt: [u16; 11] = [
            'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ':' as u16,
            ' ' as u16, '%' as u16, 'd' as u16, '\n' as u16, 0,
        ];
        _ftprintf(
            (&raw mut _iob as *mut FILE).offset(2 as ::core::ffi::c_int as isize) as *mut FILE,
            fmt.as_ptr(),
            Error,
        );
    }
    return Error as ::core::ffi::c_int;
}
