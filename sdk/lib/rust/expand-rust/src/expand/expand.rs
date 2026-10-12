use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
unsafe extern "C" fn myprintf_v(format: *const i8, ap_list: *mut ::core::ffi::c_void) -> i32 {
    let mut va: va_list;
    let mut tmp: [i8; 8192] = [0; 8192];
    let mut w: DWORD = 0;
    let mut len: i32;

    va = *(ap_list as *mut va_list);
    len = vsnprintf(
        addr_of_mut!(tmp) as *mut i8,
        core::mem::size_of::<[i8; 8192]>(),
        format,
        va,
    );
    if len > 0 {
        WriteFile(
            GetStdHandle(STD_ERROR_HANDLE),
            addr_of_mut!(tmp) as LPCVOID,
            len as DWORD,
            addr_of_mut!(w),
            core::ptr::null_mut(),
        );
    }
    return w as i32;
}



unsafe extern "system" fn set_outfile(
    mut context: PVOID,
    mut notification: UINT,
    mut param1: UINT_PTR,
    mut param2: UINT_PTR,
) -> UINT {
    let mut info: *mut FILE_IN_CABINET_INFO_A = param1 as *mut FILE_IN_CABINET_INFO_A;
    let mut buffer: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut basename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match notification {
        SPFILENOTIFY_FILEINCABINET => {
            let mut outfile: LPSTR = context as LPSTR;
            if *outfile.offset(0 as isize) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                SetLastError(ERROR_NOT_SUPPORTED as DWORD);
                return FILEOP_ABORT as UINT;
            }
            GetFullPathNameA(
                (*info).NameInCabinet as LPCSTR,
                core::mem::size_of::<[::core::ffi::c_char; (MAX_PATH) as _]>() as DWORD,
                core::ptr::addr_of_mut!(buffer) as LPSTR,
                core::ptr::addr_of_mut!(basename),
            );
            strcpy(outfile as *mut ::core::ffi::c_char, basename);
            return FILEOP_SKIP as UINT;
        }
        _ => return NO_ERROR as UINT,
    }
}

unsafe extern "system" fn extract_callback(
    context: PVOID,
    notification: UINT,
    param1: UINT_PTR,
    param2: UINT_PTR,
) -> UINT {
    let info: *mut FILE_IN_CABINET_INFO_A = param1 as *mut FILE_IN_CABINET_INFO_A;

    match notification {
        SPFILENOTIFY_FILEINCABINET => {
            let targetname: LPCSTR = context as LPCSTR;

            strcpy(
                core::ptr::addr_of_mut!((*info).FullTargetName) as *mut i8,
                targetname as *const i8,
            );
            return FILEOP_DOIT as UINT;
        }
        _ => return NO_ERROR as UINT,
    }
}

unsafe extern "C" fn option_equal(str1: LPCSTR, str2: LPCSTR) -> BOOL {
    if *str1.offset(0 as isize) as i32 != '/' as i32
        && *str1.offset(0 as isize) as i32 != '-' as i32
    {
        return (FALSE) as BOOL;
    }
    return (lstrcmpA(str1.offset(1 as isize), str2) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: i32, argv: *mut *mut i8) -> i32 {
    let mut ret: i32 = 0;
    let mut infile: [i8; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut outfile: [i8; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut actual_name: [i8; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut outfile_basename: [i8; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut basename_index: *mut i8 = core::ptr::null_mut();
    let mut comp: u32 = 0;

    if argc < 3 {
        myprintf(b"Usage:\n\0".as_ptr() as *const i8);
        myprintf(b"\t%s infile outfile\n\0".as_ptr() as *const i8, *argv.offset(0));
        myprintf(b"\t%s /r infile\n\0".as_ptr() as *const i8, *argv.offset(0));
        return 1;
    }

    if argc == 3
        && (option_equal(*argv.offset(1), b"R\0".as_ptr() as *const i8) != 0
            || option_equal(*argv.offset(1), b"r\0".as_ptr() as *const i8) != 0)
    {
        GetFullPathNameA(
            *argv.offset(2),
            core::mem::size_of::<[i8; (MAX_PATH) as _]>() as u32,
            infile.as_mut_ptr(),
            core::ptr::null_mut(),
        );
    } else {
        GetFullPathNameA(
            *argv.offset(1),
            core::mem::size_of::<[i8; (MAX_PATH) as _]>() as u32,
            infile.as_mut_ptr(),
            core::ptr::null_mut(),
        );
    }

    if SetupGetFileCompressionInfoExA(
        infile.as_mut_ptr(),
        actual_name.as_mut_ptr(),
        core::mem::size_of::<[i8; (MAX_PATH) as _]>() as u32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(comp),
    ) == 0
    {
        myprintf(
            b"%s: can't open input file %s\n\0".as_ptr() as *const i8,
            *argv.offset(0),
            infile.as_mut_ptr(),
        );
        return 1;
    }

    if argc == 3
        && (option_equal(*argv.offset(1), b"R\0".as_ptr() as *const i8) != 0
            || option_equal(*argv.offset(1), b"r\0".as_ptr() as *const i8) != 0)
    {
        match comp {
            FILE_COMPRESSION_MSZIP => {
                outfile_basename[0] = 0;
                if SetupIterateCabinetA(
                    infile.as_mut_ptr(),
                    0,
                    Some(set_outfile),
                    (outfile_basename.as_mut_ptr()).cast(),
                ) == 0
                {
                    myprintf(
                        b"%s: can't determine original name\n\0".as_ptr() as *const i8,
                        *argv.offset(0),
                    );
                    return 1;
                }
                GetFullPathNameA(
                    infile.as_mut_ptr(),
                    core::mem::size_of::<[i8; (MAX_PATH) as _]>() as u32,
                    outfile.as_mut_ptr(),
                    core::ptr::addr_of_mut!(basename_index),
                );
                *basename_index = 0;
                strcat(outfile.as_mut_ptr(), outfile_basename.as_mut_ptr());
            }
            FILE_COMPRESSION_WINLZA => {
                GetExpandedNameA(infile.as_mut_ptr(), outfile_basename.as_mut_ptr());
            }
            _ => {
                myprintf(
                    b"%s: can't determine original\n\0".as_ptr() as *const i8,
                    *argv.offset(0),
                );
                return 1;
            }
        }
    } else {
        GetFullPathNameA(
            *argv.offset(2),
            core::mem::size_of::<[i8; (MAX_PATH) as _]>() as u32,
            outfile.as_mut_ptr(),
            core::ptr::null_mut(),
        );
    }

    if lstrcmpiA(infile.as_mut_ptr(), outfile.as_mut_ptr()) == 0 {
        myprintf(
            b"%s: can't expand file to itself\n\0".as_ptr() as *const i8,
            *argv.offset(0),
        );
        return 1;
    }

    match comp {
        FILE_COMPRESSION_MSZIP => {
            if SetupIterateCabinetA(
                infile.as_mut_ptr(),
                0,
                Some(extract_callback),
                (outfile.as_mut_ptr()).cast(),
            ) == 0
            {
                myprintf(
                    b"%s: cabinet extraction failed\n\0".as_ptr() as *const i8,
                    *argv.offset(0),
                );
                return 1;
            }
        }
        FILE_COMPRESSION_WINLZA => {
            let mut hin: i32 = 0;
            let mut hout: i32 = 0;
            let mut ofin: OFSTRUCT = core::mem::zeroed();
            let mut ofout: OFSTRUCT = core::mem::zeroed();
            let mut error: i32 = 0;

            hin = LZOpenFileA(
                infile.as_mut_ptr(),
                core::ptr::addr_of_mut!(ofin),
                (OF_READ) as WORD,
            );
            if hin < 0 {
                myprintf(
                    b"%s: can't open input file %s\n\0".as_ptr() as *const i8,
                    *argv.offset(0),
                    infile.as_mut_ptr(),
                );
                return 1;
            }
            hout = LZOpenFileA(
                outfile.as_mut_ptr(),
                core::ptr::addr_of_mut!(ofout),
                (OF_CREATE | OF_WRITE) as WORD,
            );
            if hout < 0 {
                LZClose(hin);
                myprintf(
                    b"%s: can't open output file %s\n\0".as_ptr() as *const i8,
                    *argv.offset(0),
                    outfile.as_mut_ptr(),
                );
                return 1;
            }
            error = LZCopy(hin, hout);

            LZClose(hin);
            LZClose(hout);

            if error < 0 {
                myprintf(
                    b"%s: LZCopy failed, error is %d\n\0".as_ptr() as *const i8,
                    *argv.offset(0),
                    error,
                );
                return 1;
            }
        }
        _ => {
            if CopyFileA(infile.as_mut_ptr(), outfile.as_mut_ptr(), (FALSE) as BOOL) == 0 {
                myprintf(
                    b"%s: CopyFileA failed\n\0".as_ptr() as *const i8,
                    *argv.offset(0),
                );
                return 1;
            }
        }
    }
    return ret;
}
