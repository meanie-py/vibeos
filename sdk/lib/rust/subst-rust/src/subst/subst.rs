// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_16C15B3B6946: [u16; 13] = [
    0x0025u16, 0x0063, 0x003A, 0x005C, 0x003A, 0x0020, 0x003D, 0x003E, 0x0020, 0x0025,
    0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_660D31D81E7D: [u16; 3] = [0x002Fu16, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_C554B73F8939: [u16; 5] = [0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn PrintError(ErrCode: DWORD) {
    // DWORD dwLength = 0;
    let mut pMsgBuf: PWSTR = core::ptr::null_mut();

    /* Retrieve the message string without appending extra newlines */
    // dwLength =
    FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_MAX_WIDTH_MASK) as DWORD,
        core::ptr::null::<core::ffi::c_void>(),
        ErrCode,
        LANG_USER_DEFAULT as DWORD,
        &raw mut pMsgBuf as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    );
    if !pMsgBuf.is_null() /* && dwLength */ {
        ConResPrintf(
            &raw mut csStdErr,
            IDS_FAILED_WITH_ERRORCODE as UINT,
            ErrCode,
            pMsgBuf,
        );
        LocalFree(pMsgBuf as HLOCAL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn QuerySubstedDrive(
    mut DriveLetter: WCHAR,
    mut TargetPath: *mut PWSTR,
    mut Size: PULONG,
) -> ULONG {
    let mut Result: ULONG = ERROR_INVALID_DRIVE as ULONG;
    let mut Drive: [WCHAR; 3] = [0x0041u16, 0x003A, 0x0000];
    let mut dwSize: DWORD = 0;
    let mut CharCount: DWORD = 0 as DWORD;
    let mut lpTargetPath: PWSTR = core::ptr::null_mut();
    let mut tmp: PWSTR = core::ptr::null_mut();

    Drive[0 as usize] = DriveLetter;

    /* Check whether the user has given a pointer to a target path buffer */
    if TargetPath.is_null() {
        /* No, therefore use a local buffer */
        dwSize = MAX_PATH as DWORD;
        lpTargetPath = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (dwSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if lpTargetPath.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as ULONG;
        }
    } else {
        /* Just use the user-given pointer to a buffer; Size should point to a valid ULONG */
        if Size.is_null() {
            return ERROR_INVALID_PARAMETER as ULONG;
        }
        lpTargetPath = *TargetPath;
        dwSize = *Size as DWORD;
    }

    loop {
        /* Try querying DOS device information */
        CharCount = QueryDosDeviceW(
            core::ptr::addr_of_mut!(Drive) as *mut WCHAR,
            lpTargetPath,
            dwSize,
        );
        if CharCount == 0 {
            Result = GetLastError() as ULONG;
        }

        if !(CharCount == 0 && Result == ERROR_INSUFFICIENT_BUFFER as ULONG) {
            break;
        }

        /* Reallocate the buffer with double size */
        dwSize = dwSize.wrapping_mul(2 as DWORD);
        tmp = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            lpTargetPath as PVOID,
            (dwSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if tmp.is_null() {
            /* Memory problem, bail out */
            CharCount = 0 as DWORD;
            Result = ERROR_NOT_ENOUGH_MEMORY as ULONG;
            break;
        } else {
            /* Retry again */
            lpTargetPath = tmp;
        }
    }

    if CharCount != 0 {
        if wcsncmp(
            lpTargetPath as *const WCHAR,
            RUSTCONV_WSTR_C554B73F8939.as_ptr(),
            (4 as SIZE_T) as _,
        ) == 0
            && ((*lpTargetPath.add(4 as usize) >= 0x0041u16
                && *lpTargetPath.add(4 as usize) <= 0x005Au16)
                || (*lpTargetPath.add(4 as usize) >= 0x0061u16
                    && *lpTargetPath.add(4 as usize) <= 0x007Au16))
        {
            /* The drive exists and is SUBSTed */
            Result = ERROR_IS_SUBSTED as ULONG;
        }
    }

    if TargetPath.is_null() {
        /* Free the local buffer */
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpTargetPath as PVOID,
        );
    } else {
        /* Update the user-given pointers */
        *TargetPath = lpTargetPath;
        *Size = dwSize as ULONG;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "C" fn DumpSubstedDrives() {
    let mut DriveLetter: WCHAR = 0;
    let mut lpTargetPath: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut dwSize: DWORD = 0;
    let mut i: UCHAR = 0;

    dwSize = MAX_PATH;
    lpTargetPath = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if lpTargetPath.is_null() {
        return;
    }

    while i < 26 {
        DriveLetter = 0x0041u16 + i as WCHAR;
        if QuerySubstedDrive(DriveLetter, &raw mut lpTargetPath, &raw mut dwSize)
            == ERROR_IS_SUBSTED
        {
            ConPrintf(
                &raw mut csStdOut,
                RUSTCONV_WSTR_16C15B3B6946.as_ptr(),
                DriveLetter as core::ffi::c_uint,
                lpTargetPath.offset(4 as isize),
            );
        }

        i = i.wrapping_add(1);
    }

    HeapFree(
        GetProcessHeap(),
        0,
        lpTargetPath as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DeleteSubst(Drive: PWSTR) -> INT {
    let mut dwResult: DWORD = 0;
    'Quit: {
        if (wcslen(Drive) != 2) || (*Drive.add(1) != 0x003Au16)
        {
            dwResult = ERROR_INVALID_PARAMETER;
            break 'Quit;
        }

        if QuerySubstedDrive(*Drive.add(0), core::ptr::null_mut(), core::ptr::null_mut()) != ERROR_IS_SUBSTED
        {
            dwResult = ERROR_INVALID_PARAMETER;
            break 'Quit;
        }

        if ((DefineDosDeviceW(DDD_REMOVE_DEFINITION, Drive, core::ptr::null_mut())) == 0)
        {
            dwResult = GetLastError();
        }
        else
        {
            dwResult = ERROR_SUCCESS;
        }
    }

    match dwResult {
        ERROR_SUCCESS => {}

        // case ERROR_INVALID_DRIVE:
        ERROR_INVALID_PARAMETER => {
            ConResPrintf(StdErr, IDS_INVALID_PARAMETER2, Drive);
            return 1;
        }

        ERROR_ACCESS_DENIED => {
            ConResPrintf(StdErr, IDS_ACCESS_DENIED, Drive);
            return 1;
        }

        _ => {
            PrintError(GetLastError());
            return 1;
        }
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn AddSubst(Drive: PWSTR, Path: PWSTR) -> INT {
    let mut dwResult: DWORD = 0;
    let mut dwPathAttr: DWORD = 0;
    'Quit: {
        if (wcslen(Drive) != 2)
            || (*Drive.add(1 as usize) != 0x003Au16)
        {
            dwResult = ERROR_INVALID_PARAMETER;
            break 'Quit;
        }

        /*
         * Even if DefineDosDevice allows to map files to drive letters (yes yes!!)
         * it is not the purpose of SUBST to allow that. Therefore check whether
         * the given path exists and really is a path to a directory, and if not,
         * just fail with an error.
         */
        dwPathAttr = GetFileAttributesW(Path);
        if (dwPathAttr == INVALID_FILE_ATTRIBUTES)
            || !((dwPathAttr & FILE_ATTRIBUTE_DIRECTORY) != 0)
        {
            dwResult = ERROR_PATH_NOT_FOUND;
            break 'Quit;
        }

        /*
         * QuerySubstedDrive (via QueryDosDevice) returns ERROR_FILE_NOT_FOUND only
         * if there is no already existing drive mapping. For all other results
         * (existing drive, be it already subst'ed or not, or other errors...)
         * no attempt at defining a drive mapping should be done.
         */
        dwResult = QuerySubstedDrive(
            *Drive.add(0 as usize),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if dwResult != ERROR_FILE_NOT_FOUND {
            break 'Quit;
        }

        if DefineDosDeviceW(0, Drive, Path) == 0 {
            dwResult = GetLastError();
        } else {
            dwResult = ERROR_SUCCESS;
        }
    }

    // Quit:
    if dwResult == ERROR_SUCCESS {
        // break
    } else if dwResult == ERROR_INVALID_DRIVE || dwResult == ERROR_INVALID_PARAMETER {
        ConResPrintf(StdErr, IDS_INVALID_PARAMETER2, Drive);
        return 1;
    } else if dwResult == ERROR_IS_SUBSTED {
        ConResPuts(StdErr, IDS_DRIVE_ALREADY_SUBSTED);
        return 1;
    } else if dwResult == ERROR_FILE_NOT_FOUND || dwResult == ERROR_PATH_NOT_FOUND {
        ConResPrintf(StdErr, IDS_PATH_NOT_FOUND, Path);
        return 1;
    } else if dwResult == ERROR_ACCESS_DENIED {
        ConResPrintf(StdErr, IDS_ACCESS_DENIED, Path);
        return 1;
    } else {
        PrintError(GetLastError());
        return 1;
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut i: INT;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    i = 0;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_5462AED96D21.as_ptr()) == 0 {
            ConResPuts(StdOut, IDS_USAGE);
            return 0;
        }
        i += 1;
    }

    if argc < 3 {
        if argc >= 2 {
            ConResPrintf(StdErr, IDS_INVALID_PARAMETER, *argv.offset(1 as isize));
            return 1;
        }
        DumpSubstedDrives();
        return 0;
    }

    if argc > 3 {
        ConResPrintf(StdErr, IDS_INCORRECT_PARAMETER_COUNT, *argv.offset(3 as isize));
        return 1;
    }

    if _wcsicmp(*argv.offset(1 as isize), RUSTCONV_WSTR_660D31D81E7D.as_ptr()) == 0 {
        return DeleteSubst(*argv.offset(2 as isize) as PWSTR);
    }
    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_660D31D81E7D.as_ptr()) == 0 {
        return DeleteSubst(*argv.offset(1 as isize) as PWSTR);
    }
    return AddSubst(
        *argv.offset(1 as isize) as PWSTR,
        *argv.offset(2 as isize) as PWSTR,
    );
}
