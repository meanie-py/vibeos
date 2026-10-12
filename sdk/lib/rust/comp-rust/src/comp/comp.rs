// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_14BBCEB1CBCC: [u16; 33] = [
    0x0043u16, 0x0061, 0x006E, 0x0027, 0x0074, 0x0020, 0x0067, 0x0065, 0x0074, 0x0020,
    0x0066, 0x0072, 0x0065, 0x0065, 0x0020, 0x006D, 0x0065, 0x006D, 0x006F, 0x0072,
    0x0079, 0x0020, 0x0066, 0x006F, 0x0072, 0x0020, 0x0042, 0x0075, 0x0066, 0x0066,
    0x0032, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_76CFE90FF2FC: [u16; 3] = [0x0072u16, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_BD955B5BBED1: [u16; 33] = [
    0x0043u16, 0x0061, 0x006E, 0x0027, 0x0074, 0x0020, 0x0067, 0x0065, 0x0074, 0x0020,
    0x0066, 0x0072, 0x0065, 0x0065, 0x0020, 0x006D, 0x0065, 0x006D, 0x006F, 0x0072,
    0x0079, 0x0020, 0x0066, 0x006F, 0x0072, 0x0020, 0x0042, 0x0075, 0x0066, 0x0066,
    0x0031, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const STRBUF: usize = 1024;
#[no_mangle]
pub unsafe extern "C" fn GetBuff(buff: PBYTE, in_: *mut FILE) -> size_t {
    return fread(buff as *mut core::ffi::c_void, core::mem::size_of::<BYTE>() as size_t, STRBUF as size_t, in_) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn FileSize(mut fd: *mut FILE) -> INT {
    let mut result: INT = -1;
    if fseek(fd, 0, SEEK_END as i32) == 0 && {
        result = ftell((fd).cast()) as INT;
        result != -1
    } {
        /* Restoring file pointer */
        rewind(fd);
    }
    return result;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *mut WCHAR) -> INT {
    /* File pointers */
    let mut fp1: *mut FILE = core::ptr::null_mut();
    let mut fp2: *mut FILE = core::ptr::null_mut();

    let mut i: usize;
    let mut BufLen1: usize;
    let mut BufLen2: usize;
    let mut Buff1: PBYTE = core::ptr::null_mut();
    let mut Buff2: PBYTE = core::ptr::null_mut();
    // File paths
    let mut File1: [WCHAR; (_MAX_PATH + 1) as _] =
        [0; (_MAX_PATH + 1) as _];
    let mut File2: [WCHAR; (_MAX_PATH + 1) as _] =
        [0; (_MAX_PATH + 1) as _];
    // /A switch, /L switch
    let mut bAscii: BOOL = (FALSE) as i32;
    let mut bLineNos: BOOL = (FALSE) as i32;
    let mut LineNumber: UINT;
    let mut Offset: UINT;
    let mut FileSizeFile1: INT;
    let mut FileSizeFile2: INT;
    let mut NumberOfOptions: INT = 0;
    let mut FilesOK: INT = 1;
    let mut Status: INT = (EXIT_SUCCESS as i32);

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* Parse command line for options */
    i = 1 as usize;
    while i < argc as usize {
        if *(*argv.offset(i as isize)).offset(0 as isize)
            == 0x002Fu16
        {
            match towlower(
                *(*argv.offset(i as isize)).offset(1 as isize),
            ) {
                0x0061u16 => {
                    bAscii = (TRUE) as i32;
                    NumberOfOptions += 1;
                }
                0x006Cu16 => {
                    bLineNos = (TRUE) as i32;
                    NumberOfOptions += 1;
                }
                0x003Fu16 => {
                    ConResPuts(StdOut, IDS_HELP);
                    return (EXIT_SUCCESS) as INT;
                }
                _ => {
                    ConResPrintf(
                        StdErr,
                        IDS_INVALIDSWITCH,
                        *(*argv.offset(i as isize)).offset(1 as isize)
                            as ::core::ffi::c_int,
                    );
                    ConResPuts(StdOut, IDS_HELP);
                    return (EXIT_FAILURE) as INT;
                }
            }
        }
        i = i.wrapping_add(1);
    }

    if argc - NumberOfOptions == 3 {
        wcsncpy(
            core::ptr::addr_of_mut!(File1) as *mut WCHAR,
            *argv.offset((1 + NumberOfOptions) as isize),
            (_MAX_PATH) as _,
        );
        wcsncpy(
            core::ptr::addr_of_mut!(File2) as *mut WCHAR,
            *argv.offset((2 + NumberOfOptions) as isize),
            (_MAX_PATH) as _,
        );
    } else {
        ConResPuts(StdErr, IDS_BADSYNTAX);
        return (EXIT_FAILURE) as INT;
    }

    'cleanup: {
        Buff1 = malloc(STRBUF) as PBYTE;
        if Buff1.is_null() {
            ConPuts(StdErr, RUSTCONV_WSTR_BD955B5BBED1.as_ptr());
            Status = (EXIT_FAILURE as i32);
            break 'cleanup;
        }

        Buff2 = malloc(STRBUF) as PBYTE;
        if Buff2.is_null() {
            ConPuts(StdErr, RUSTCONV_WSTR_14BBCEB1CBCC.as_ptr());
            Status = (EXIT_FAILURE as i32);
            break 'cleanup;
        }

        fp1 = (_wfopen(
            core::ptr::addr_of_mut!(File1) as *mut WCHAR,
            RUSTCONV_WSTR_76CFE90FF2FC.as_ptr(),
        )).cast();
        if fp1.is_null() {
            ConResPrintf(
                StdErr,
                IDS_FILEERROR,
                core::ptr::addr_of_mut!(File1) as *mut WCHAR,
            );
            Status = (EXIT_FAILURE as i32);
            break 'cleanup;
        }
        fp2 = (_wfopen(
            core::ptr::addr_of_mut!(File2) as *mut WCHAR,
            RUSTCONV_WSTR_76CFE90FF2FC.as_ptr(),
        )).cast();
        if fp2.is_null() {
            ConResPrintf(
                StdErr,
                IDS_FILEERROR,
                core::ptr::addr_of_mut!(File2) as *mut WCHAR,
            );
            Status = (EXIT_FAILURE as i32);
            break 'cleanup;
        }

        ConResPrintf(
            StdOut,
            IDS_COMPARING,
            core::ptr::addr_of_mut!(File1) as *mut WCHAR,
            core::ptr::addr_of_mut!(File2) as *mut WCHAR,
        );

        FileSizeFile1 = FileSize(fp1);
        if FileSizeFile1 == -1 {
            ConResPrintf(
                StdErr,
                IDS_FILESIZEERROR,
                core::ptr::addr_of_mut!(File1) as *mut WCHAR,
            );
            Status = (EXIT_FAILURE as i32);
            break 'cleanup;
        }

        FileSizeFile2 = FileSize(fp2);
        if FileSizeFile2 == -1 {
            ConResPrintf(
                StdErr,
                IDS_FILESIZEERROR,
                core::ptr::addr_of_mut!(File2) as *mut WCHAR,
            );
            Status = (EXIT_FAILURE as i32);
            break 'cleanup;
        }

        if FileSizeFile1 != FileSizeFile2 {
            ConResPuts(StdOut, IDS_SIZEDIFFERS);
            Status = (EXIT_FAILURE as i32);
            break 'cleanup;
        }

        LineNumber = 1;
        Offset = 0;
        while 1 != 0 {
            BufLen1 = GetBuff(Buff1, fp1);
            BufLen2 = GetBuff(Buff2, fp2);

            if ferror(fp1) != 0 || ferror(fp2) != 0 {
                ConResPuts(StdErr, IDS_READERROR);
                Status = (EXIT_FAILURE as i32);
                break 'cleanup;
            }

            if BufLen1 == 0 && BufLen2 == 0 {
                break;
            }

            assert!(BufLen1 == BufLen2);
            i = 0 as usize;
            while i < BufLen1 {
                if *Buff1.offset(i as isize) != *Buff2.offset(i as isize) {
                    FilesOK = 0;

                    /* Reporting here a mismatch */
                    if bLineNos != 0 {
                        ConResPrintf(StdOut, IDS_MISMATCHLINE, LineNumber);
                    } else {
                        ConResPrintf(StdOut, IDS_MISMATCHOFFSET, Offset);
                    }

                    if bAscii != 0 {
                        ConResPrintf(
                            StdOut,
                            IDS_ASCIIDIFF,
                            1,
                            *Buff1.offset(i as isize) as ::core::ffi::c_int,
                        );
                        ConResPrintf(
                            StdOut,
                            IDS_ASCIIDIFF,
                            2,
                            *Buff2.offset(i as isize) as ::core::ffi::c_int,
                        );
                    } else {
                        ConResPrintf(
                            StdOut,
                            IDS_HEXADECIMALDIFF,
                            1,
                            *Buff1.offset(i as isize) as ::core::ffi::c_int,
                        );
                        ConResPrintf(
                            StdOut,
                            IDS_HEXADECIMALDIFF,
                            2,
                            *Buff2.offset(i as isize) as ::core::ffi::c_int,
                        );
                    }
                }

                Offset = Offset.wrapping_add(1);

                if *Buff1.offset(i as isize) == b'\n' {
                    LineNumber = LineNumber.wrapping_add(1);
                }
                i = i.wrapping_add(1);
            }
        }

        if FilesOK != 0 {
            ConResPuts(StdOut, IDS_MATCH);
        }
    }

    /* Cleanup */
    if !fp2.is_null() {
        fclose(fp2);
    }
    if !fp1.is_null() {
        fclose(fp1);
    }

    if !Buff2.is_null() {
        free(Buff2 as *mut ::core::ffi::c_void);
    }
    if !Buff1.is_null() {
        free(Buff1 as *mut ::core::ffi::c_void);
    }

    return Status;
}
