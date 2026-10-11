use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut force_mode: BOOL = 0;

static mut show_content: BOOL = 0;

// WINE_DEFAULT_DEBUG_CHANNEL(extrac32): debug-channel macro, no Rust declaration emitted.

unsafe extern "C" fn create_target_directory(Target: LPWSTR) {
    let mut dir: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut res: ::core::ffi::c_int = 0;

    strcpyW!(dir.as_mut_ptr(), Target);
    *PathFindFileNameW(dir.as_mut_ptr()) = 0 as WCHAR; /* Truncate file name */
    if PathIsDirectoryW(dir.as_mut_ptr() as LPCWSTR) == 0 {
        res = SHCreateDirectoryExW(core::ptr::null_mut(), dir.as_mut_ptr() as LPCWSTR, core::ptr::null_mut());
        if (res) as u32 != ERROR_SUCCESS && (res) as u32 != ERROR_ALREADY_EXISTS {
            WINE_ERR!(
                b"Can't create directory: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                wine_dbgstr_w(dir.as_ptr()),
            );
        }
    }
}
unsafe extern "system" fn ExtCabCallback(
    mut Context: PVOID,
    mut Notification: UINT,
    mut Param1: UINT_PTR,
    mut Param2: UINT_PTR,
) -> UINT {
    let mut pInfo: *mut FILE_IN_CABINET_INFO_W = ::core::ptr::null_mut::<FILE_IN_CABINET_INFO_W>();
    let mut pFilePaths: *mut FILEPATHS_W = ::core::ptr::null_mut::<FILEPATHS_W>();
    match Notification {
        SPFILENOTIFY_FILEINCABINET => {
            pInfo = Param1 as *mut FILE_IN_CABINET_INFO_W;
            if show_content != 0 {
                let mut ft: FILETIME = core::mem::zeroed();
                let mut st: SYSTEMTIME = core::mem::zeroed();
                let mut date: [CHAR; 12] = [0; 12];
                let mut time: [CHAR; 12] = [0; 12];
                let mut buf: [CHAR; (2 * MAX_PATH) as _] =
                    [0; (2 * MAX_PATH) as _];
                let mut count: ::core::ffi::c_int = 0;
                let mut dummy: DWORD = 0;

                /* DosDate and DosTime already represented at local time */
                DosDateTimeToFileTime((*pInfo).DosDate, (*pInfo).DosTime, &raw mut ft);
                FileTimeToSystemTime(&raw mut ft, &raw mut st);
                GetDateFormatA(
                    0 as LCID,
                    0 as DWORD,
                    &raw mut st,
                    b"MM'-'dd'-'yyyy\0" as *const u8 as LPCSTR,
                    &raw mut date as LPSTR,
                    ::core::mem::size_of::<[CHAR; 12]>() as ::core::ffi::c_int,
                );
                GetTimeFormatA(
                    0 as LCID,
                    0 as DWORD,
                    &raw mut st,
                    b"HH':'mm':'ss\0" as *const u8 as LPCSTR,
                    &raw mut time as LPSTR,
                    ::core::mem::size_of::<[CHAR; 12]>() as ::core::ffi::c_int,
                );
                count = wsprintfA(
                    &raw mut buf as LPSTR,
                    b"%s %s %c%c%c%c %15u %S\n\0" as *const u8 as LPCSTR,
                    &raw mut date as *mut CHAR,
                    &raw mut time as *mut CHAR,
                    if (*pInfo).DosAttribs as u32 & FILE_ATTRIBUTE_ARCHIVE != 0 {
                        'A' as i32
                    } else {
                        '-' as i32
                    },
                    if (*pInfo).DosAttribs as u32 & FILE_ATTRIBUTE_HIDDEN != 0 {
                        'H' as i32
                    } else {
                        '-' as i32
                    },
                    if (*pInfo).DosAttribs as u32 & FILE_ATTRIBUTE_READONLY != 0 {
                        'R' as i32
                    } else {
                        '-' as i32
                    },
                    if (*pInfo).DosAttribs as u32 & FILE_ATTRIBUTE_SYSTEM != 0 {
                        'S' as i32
                    } else {
                        '-' as i32
                    },
                    (*pInfo).FileSize,
                    (*pInfo).NameInCabinet,
                );
                WriteFile(
                    GetStdHandle(STD_OUTPUT_HANDLE),
                    &raw mut buf as *mut CHAR as LPCVOID,
                    count as DWORD,
                    &raw mut dummy,
                    ::core::ptr::null_mut::<_OVERLAPPED>(),
                );
                return FILEOP_SKIP as UINT;
            } else {
                lstrcpyW(
                    &raw mut (*pInfo).FullTargetName as LPWSTR,
                    Context as LPCWSTR,
                );
                lstrcatW(
                    &raw mut (*pInfo).FullTargetName as LPWSTR,
                    (*pInfo).NameInCabinet as LPCWSTR,
                );
                /* SetupIterateCabinet() doesn't create full path to target by itself,
                   so we should do it manually */
                create_target_directory(&raw mut (*pInfo).FullTargetName as LPWSTR);
                return FILEOP_DOIT as UINT;
            }
        }
        SPFILENOTIFY_FILEEXTRACTED => {
            pFilePaths = Param1 as *mut FILEPATHS_W;
            WINE_TRACE!(
                b"Extracted %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                wine_dbgstr_w((*pFilePaths).Target as *const WCHAR),
            );
            return NO_ERROR as UINT;
        }
        _ => {}
    }
    return NO_ERROR as UINT;
}

unsafe extern "C" fn extract(cabfile: LPCWSTR, destdir: LPWSTR) {
    if SetupIterateCabinetW(
        cabfile,
        0 as DWORD,
        Some(ExtCabCallback),
        destdir as PVOID,
    ) == 0
    {
        WINE_ERR!(
            b"Could not extract cab file %s\n\0".as_ptr() as *const i8,
            wine_dbgstr_w(cabfile),
        );
    }
}

unsafe extern "C" fn copy_file(source: LPCWSTR, mut destination: LPCWSTR) {
    let mut destfile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* append source filename if destination is a directory */
    if PathIsDirectoryW(destination) != 0 {
        PathCombineW(
            &raw mut destfile as LPWSTR,
            destination,
            PathFindFileNameW(source) as LPCWSTR,
        );
        destination = &raw mut destfile as *mut WCHAR as LPCWSTR;
    }

    if PathFileExistsW(destination) != 0 && force_mode == 0 {
        static mut overwriteMsg: [WCHAR; 16] = [
    0x004Fu16, 0x0076, 0x0065, 0x0072, 0x0077, 0x0072, 0x0069, 0x0074, 0x0065, 0x0020,
    0x0022, 0x0025, 0x0073, 0x0022, 0x003F, 0x0000,
];
        static mut titleMsg: [WCHAR; 8] = [0x0045u16, 0x0078, 0x0074, 0x0072, 0x0061, 0x0063, 0x0074, 0x0000];
        let mut msg: [WCHAR; (MAX_PATH + 100) as _] =
            [0; (MAX_PATH + 100) as _];
        snprintfW(
            &raw mut msg as *mut WCHAR,
            (ARRAY_SIZE!(msg)) as _,
            &raw const overwriteMsg as *const WCHAR,
            destination,
        );
        if (MessageBoxW(
            core::ptr::null_mut(),
            &raw mut msg as *mut WCHAR as LPCWSTR,
            &raw const titleMsg as LPCWSTR,
            (MB_YESNO | MB_ICONWARNING) as UINT,
        )) as u32 != IDYES
        {
            return;
        }
    }

    WINE_TRACE!(
        b"copying %s to %s\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(source),
        wine_dbgstr_w(destination),
    );
    CopyFileW(source, destination, (FALSE) as BOOL);
}

unsafe extern "C" fn get_extrac_args(mut cmdline: LPWSTR, mut pargc: *mut ::core::ffi::c_int) -> *mut LPWSTR {
    let _ = (OUTSIDE_ARG, INSIDE_ARG, INSIDE_QUOTED_ARG);
    let mut state: ::core::ffi::c_int;
    let mut str: LPWSTR;
    let mut argc: ::core::ffi::c_int;
    let mut argv: *mut LPWSTR;
    let mut max_argc: ::core::ffi::c_int = 16;
    let mut new_arg: BOOL;

    WINE_TRACE!(b"cmdline: %s\n\0".as_ptr() as *const ::core::ffi::c_char, wine_dbgstr_w(cmdline));
    str = HeapAlloc(GetProcessHeap(), 0 as DWORD, (strlenW!(cmdline) as SIZE_T + 1) * core::mem::size_of::<WCHAR>() as SIZE_T) as LPWSTR;
    if str.is_null() {
        return ::core::ptr::null_mut::<LPWSTR>();
    }
    strcpyW!(str, cmdline);
    argv = HeapAlloc(GetProcessHeap(), 0 as DWORD, ((max_argc + 1) as SIZE_T).wrapping_mul(core::mem::size_of::<LPWSTR>() as SIZE_T)) as *mut LPWSTR;
    if argv.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, str as PVOID);
        return ::core::ptr::null_mut::<LPWSTR>();
    }

    /* Split command line to separate arg-strings and fill argv */
    state = OUTSIDE_ARG;
    argc = 0;
    while *str != 0 {
        new_arg = (FALSE) as i32;
        /* Check character */
        if isspaceW!(*str) != 0 {          /* white space */
            if state == INSIDE_ARG {
                state = OUTSIDE_ARG;
                *str = 0 as WCHAR;
            }
        } else if *str == '"' as WCHAR {  /* double quote */
            match state {
                INSIDE_QUOTED_ARG => {
                    state = OUTSIDE_ARG;
                    *str = 0 as WCHAR;
                }
                INSIDE_ARG => {
                    *str = 0 as WCHAR;
                    /* Fall through */
                    str = str.offset(1);
                    if *str == 0 {
                        continue;
                    }
                    state = INSIDE_QUOTED_ARG;
                    new_arg = (TRUE) as i32;
                }
                OUTSIDE_ARG => {
                    str = str.offset(1);
                    if *str == 0 {
                        continue;
                    }
                    state = INSIDE_QUOTED_ARG;
                    new_arg = (TRUE) as i32;
                }
                _ => {}
            }
        } else {                          /* regular character */
            if state == OUTSIDE_ARG {
                state = INSIDE_ARG;
                new_arg = (TRUE) as i32;
            }
        }

        /* Add new argv entry, if need */
        if new_arg != 0 {
            if argc >= max_argc - 1 {
                /* Realloc argv here because there always should be
                   at least one reserved cell for terminating NULL */
                max_argc *= 2;
                argv = HeapReAlloc(GetProcessHeap(), 0 as DWORD, argv as PVOID,
                        ((max_argc + 1) as SIZE_T).wrapping_mul(core::mem::size_of::<LPWSTR>() as SIZE_T)) as *mut LPWSTR;
                if argv.is_null() {
                    HeapFree(GetProcessHeap(), 0 as DWORD, str as PVOID);
                    return ::core::ptr::null_mut::<LPWSTR>();
                }
            }
            *argv.offset(argc as isize) = str;
            argc += 1;
        }

        str = str.offset(1);
    }

    *argv.offset(argc as isize) = ::core::ptr::null_mut::<WCHAR>();
    *pargc = argc;

    if TRACE_ON!(__wine_dbch_extrac32) != 0 {
        let mut i: ::core::ffi::c_int;
        i = 0;
        while i < argc {
            WINE_TRACE!(b"arg %d: %s\n\0".as_ptr() as *const ::core::ffi::c_char, i, wine_dbgstr_w(*argv.offset(i as isize)));
            i += 1;
        }
    }
    return argv;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    prev: HINSTANCE,
    cmdline: LPWSTR,
    show: i32,
) -> i32 {
    let mut argv: *mut LPWSTR;
    let mut argc: i32 = 0;
    let mut i: i32 = 0;
    let mut check: WCHAR = 0;
    let mut cmd: WCHAR = 0;
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut cabfile: LPCWSTR = core::ptr::null();

    path[0] = 0;

    /* Do not use CommandLineToArgvW() or __wgetmainargs() to parse
     * command line for this program. It should treat each quote as argument
     * delimiter. This doesn't match with behavior of mentioned functions.
     * Do not use args provided by wmain() for the same reason.
     */
    argv = get_extrac_args(cmdline, core::ptr::addr_of_mut!(argc));

    if argv.is_null() {
        WINE_ERR!(b"Command line parsing failed\n\0".as_ptr() as *const i8);
        return 0;
    }

    /* Parse arguments */
    i = 0;
    'for_loop: while i < argc {
        'body: {
            /* Get cabfile */
            if (*(*argv.offset(i as isize)).offset(0) as i32 != '/' as i32)
                && (*(*argv.offset(i as isize)).offset(0) as i32 != '-' as i32)
            {
                if cabfile.is_null() {
                    cabfile = *argv.offset(i as isize) as LPCWSTR;
                    break 'body;
                } else {
                    break 'for_loop;
                }
            }
            /* Get parameters for commands */
            check = toupperW!(*(*argv.offset(i as isize)).offset(1)) as WCHAR;
            match check as i32 {
                65 => {
                    WINE_FIXME!(b"/A not implemented\n\0".as_ptr() as *const i8);
                }
                89 => {
                    force_mode = (TRUE) as i32;
                }
                76 => {
                    if (i + 1) >= argc {
                        return 0;
                    }
                    i += 1;
                    if GetFullPathNameW(
                        *argv.offset(i as isize) as LPCWSTR,
                        MAX_PATH,
                        core::ptr::addr_of_mut!(path) as LPWSTR,
                        core::ptr::null_mut(),
                    ) == 0
                    {
                        return 0;
                    }
                }
                67 | 69 | 68 => {
                    if cmd != 0 {
                        return 0;
                    }
                    cmd = check;
                }
                _ => {
                    return 0;
                }
            }
        }
        i += 1;
    }

    if cabfile.is_null() {
        return 0;
    }

    if cmd == ('C' as WCHAR) {
        if (i + 1) != argc {
            return 0;
        }
        if GetFullPathNameW(
            *argv.offset(i as isize) as LPCWSTR,
            MAX_PATH,
            core::ptr::addr_of_mut!(path) as LPWSTR,
            core::ptr::null_mut(),
        ) == 0
        {
            return 0;
        }
    } else if cmd == 0 {
        /* Use extraction by default if names of required files presents */
        cmd = if i < argc { 'E' as WCHAR } else { 'D' as WCHAR };
    }

    if (cmd == ('E' as WCHAR)) && (path[0] == 0) {
        GetCurrentDirectoryW(
            MAX_PATH,
            core::ptr::addr_of_mut!(path) as LPWSTR,
        );
    }

    PathAddBackslashW(core::ptr::addr_of_mut!(path) as LPWSTR);

    /* Execute the specified command */
    match cmd as i32 {
        67 => {
            /* Copy file */
            copy_file(cabfile, core::ptr::addr_of_mut!(path) as LPCWSTR);
        }
        68 => {
            /* Display CAB archive */
            show_content = (TRUE) as i32;
            /* Fall through */
            /* Extract CAB archive */
            extract(cabfile, core::ptr::addr_of_mut!(path) as LPWSTR);
        }
        69 => {
            /* Extract CAB archive */
            extract(cabfile, core::ptr::addr_of_mut!(path) as LPWSTR);
        }
        _ => {}
    }
    return 0;
}
