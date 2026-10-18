// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_260DAF524DD6: [u16; 17] = [
    0x000Au16, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D, 0x002D,
    0x002D, 0x0020, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_287E9076D428: [u16; 5] = [0x0025u16, 0x006C, 0x0064, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_3106C82989F1: [u16; 5] = [0x002Fu16, 0x006F, 0x0066, 0x0066, 0x0000];
pub static RUSTCONV_WSTR_7CE3C618A5D8: [u16; 9] = [0x002Fu16, 0x006F, 0x0066, 0x0066, 0x006C, 0x0069, 0x006E, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_8A28804AA482: [u16; 6] = [0x005Bu16, 0x0025, 0x006C, 0x0064, 0x005D, 0x0000];
pub static RUSTCONV_WSTR_997B10FEC0D2: [u16; 3] = [0x003Au16, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_D877BF4E5023: [u16; 2] = [0x0072u16, 0x0000];
pub static RUSTCONV_WSTR_FB47CCC40FE6: [u16; 3] = [0x0025u16, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut bInvertSearch: BOOL = (FALSE) as i32;

static mut bCountLines: BOOL = (FALSE) as i32;

static mut bDisplayLineNumbers: BOOL = (FALSE) as i32;

static mut bIgnoreCase: BOOL = (FALSE) as i32;

static mut bDoNotSkipOfflineFiles: BOOL = (FALSE) as i32;

pub const FIND_LINE_BUFFER_SIZE: usize = 4096;

unsafe extern "C" fn StrStrCase(
    mut pszStr: PCWSTR,
    mut pszSearch: PCWSTR,
    mut b_ignore_case: BOOL,
) -> PWSTR {
    if b_ignore_case != 0 {
        let mut LocaleId: LCID = 0;
        let mut cch1: UINT = 0;
        let mut cch2: UINT = 0;
        let mut i: INT = 0;

        LocaleId = GetThreadLocale();

        cch1 = wcslen(pszStr) as UINT;
        cch2 = wcslen(pszSearch) as UINT;

        if cch2 == (0) {
            return pszStr as PWSTR;
        }
        // Divergence from the C: `i <= cch1 - cch2` is unsigned, so a search string longer
        // than the line wrapped to ~4G and CompareStringW was handed offsets past the line's
        // end. A longer needle simply cannot match.
        if cch2 > cch1 {
            return ::core::ptr::null_mut::<WCHAR>();
        }

        i = (0) as INT;
        while (i as UINT) <= cch1.wrapping_sub(cch2) {
            if (CompareStringW(
                LocaleId,
                (NORM_IGNORECASE) as DWORD,
                pszStr.offset(i as isize),
                cch2 as ::core::ffi::c_int,
                pszSearch,
                cch2 as ::core::ffi::c_int,
            )) as u32 == (CSTR_EQUAL)
            {
                return pszStr.offset(i as isize) as PWSTR;
            }
            i += 1;
        }
        return ::core::ptr::null_mut::<WCHAR>();
    } else {
        return wcsstr(pszStr, pszSearch) as PWSTR;
    }
}

unsafe extern "C" fn FindString(
    pStream: *mut FILE,
    pszFilePath: PCWSTR,
    pszSearchString: PCWSTR,
) -> ::core::ffi::c_int {
    let mut lLineCount: LONG = 0;
    let mut lLineNumber: LONG = 0;
    let mut bSubstringFound: BOOL;
    let mut iReturnValue: ::core::ffi::c_int = 1;
    let mut szLineBuffer: [WCHAR; FIND_LINE_BUFFER_SIZE] = [0; FIND_LINE_BUFFER_SIZE];

    if !pszFilePath.is_null() {
        /* Print the file's header */
        ConPrintf(
            &raw mut csStdOut,
            RUSTCONV_WSTR_260DAF524DD6.as_ptr(),
            pszFilePath,
            if bCountLines != 0 {
                RUSTCONV_WSTR_997B10FEC0D2.as_ptr()
            } else {
                RUSTCONV_WSTR_075DE2B906DB.as_ptr()
            },
        );
    }

    /* Loop through every line in the file */
    // FIXME: What if the string we search for crosses the boundary of our szLineBuffer ?
    while !fgetws(
        &raw mut szLineBuffer as *mut WCHAR,
        (_countof!(szLineBuffer)) as i32,
        pStream,
    )
    .is_null()
    {
        lLineNumber += 1;

        bSubstringFound = (StrStrCase(
            &raw mut szLineBuffer as *mut WCHAR as PCWSTR,
            pszSearchString,
            bIgnoreCase,
        ) != core::ptr::null_mut()) as ::core::ffi::c_int as BOOL;

        /* Check if this line can be counted */
        if bSubstringFound != bInvertSearch {
            iReturnValue = 0;

            if bCountLines != 0 {
                lLineCount += 1;
            } else {
                /* Display the line number if needed */
                if bDisplayLineNumbers != 0 {
                    ConPrintf(
                        &raw mut csStdOut,
                        RUSTCONV_WSTR_8A28804AA482.as_ptr(),
                        lLineNumber,
                    );
                }
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_FB47CCC40FE6.as_ptr(),
                    &raw mut szLineBuffer as *mut WCHAR,
                );
            }
        }
    }

    if bCountLines != 0 {
        /* Print the matching line count */
        ConPrintf(
            &raw mut csStdOut,
            RUSTCONV_WSTR_287E9076D428.as_ptr(),
            lLineCount,
        );
    }

    return iReturnValue;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(
    argc: ::core::ffi::c_int,
    argv: *mut *mut WCHAR,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut iReturnValue: ::core::ffi::c_int = 2;
    let mut iSearchedStringIndex: ::core::ffi::c_int = -1;
    let mut bFoundFileParameter: BOOL = (FALSE) as i32;
    let mut hFindFile: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut FindData: WIN32_FIND_DATAW = ::core::mem::zeroed::<WIN32_FIND_DATAW>();
    let mut pOpenedFile: *mut FILE = ::core::ptr::null_mut();
    let mut ptr: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut szFullFilePath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if argc == 1 {
        /* If no argument were provided by the user, display program usage and exit */
        ConResPuts(&raw mut csStdOut, IDS_USAGE);
        return 0;
    }

    /* Parse the command line arguments */
    i = 1;
    while i < argc {
        /* Check if this argument contains a switch */
        if wcslen(*argv.offset(i as isize)) == 2
            && *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16
        {
            match towupper(
                (*(*argv.offset(i as isize)).offset(1 as isize)) as _,
            ) as u16
            {
                0x003Fu16 => {
                    ConResPuts(&raw mut csStdOut, IDS_USAGE);
                    return 0;
                }
                0x0056u16 => {
                    bInvertSearch = (TRUE) as i32;
                }
                0x0043u16 => {
                    bCountLines = (TRUE) as i32;
                }
                0x004Eu16 => {
                    bDisplayLineNumbers = (TRUE) as i32;
                }
                0x0049u16 => {
                    bIgnoreCase = (TRUE) as i32;
                }
                _ => {
                    /* Report invalid switch error */
                    ConResPuts(&raw mut csStdErr, IDS_INVALID_SWITCH);
                    return 2;
                }
            }
        } else if wcslen(*argv.offset(i as isize)) > 2
            && *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16
        {
            /* Check if this parameter is /OFF or /OFFLINE */
            if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_3106C82989F1.as_ptr()) == 0
                || _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_7CE3C618A5D8.as_ptr()) == 0
            {
                bDoNotSkipOfflineFiles = (TRUE) as i32;
            } else {
                /* Report invalid switch error */
                ConResPuts(&raw mut csStdErr, IDS_INVALID_SWITCH);
                return 2;
            }
        } else {
            if iSearchedStringIndex == -1 {
                iSearchedStringIndex = i;
            } else {
                /* There's a file specified in the parameters, no need to read from __iob_func().add(0) */
                bFoundFileParameter = (TRUE) as i32;
            }
        }
        i += 1;
    }

    if iSearchedStringIndex == -1 {
        /* User didn't provide the string to search for, display program usage and exit */
        ConResPuts(&raw mut csStdErr, IDS_USAGE);
        return 2;
    }

    if bFoundFileParameter != 0 {
        /* After the command line arguments were parsed, iterate through them again to get the filenames */
        i = 1;
        while i < argc {
            /* If the value is a switch or the searched string, continue */
            if !((wcslen(*argv.offset(i as isize)) > 0
                && *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16)
                || i == iSearchedStringIndex)
            {
                hFindFile = FindFirstFileW(
                    *argv.offset(i as isize) as LPCWSTR,
                    &raw mut FindData,
                );
                if hFindFile == INVALID_HANDLE_VALUE {
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_NO_SUCH_FILE,
                        *argv.offset(i as isize),
                    );
                } else {
                    loop {
                        'body: {
                            /* Check if the file contains offline attribute and should be skipped */
                            if (FindData.dwFileAttributes & FILE_ATTRIBUTE_OFFLINE) != 0
                                && bDoNotSkipOfflineFiles == 0
                            {
                                break 'body;
                            }

                            /* Skip directory */
                            // FIXME: Implement recursivity?
                            if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                                break 'body;
                            }

                            /*
                             * Build the full file path from the file specification pattern.
                             *
                             * Note that we could use GetFullPathName() instead, however
                             * we want to keep compatibility with Windows' find.exe utility
                             * that does not use this function as it keeps the file name
                             * directly based on the pattern.
                             */
                            ptr = wcsrchr(
                                *argv.offset(i as isize),
                                0x005Cu16,
                            ) as PWCHAR; // Check for last directory.
                            if ptr.is_null() {
                                ptr = wcsrchr(
                                    *argv.offset(i as isize),
                                    0x003Au16,
                                ) as PWCHAR; // Check for drive.
                            }
                            if !ptr.is_null() {
                                /* The pattern contains a drive or directory part: keep it and concatenate the full file name */
                                StringCchCopyNW(
                                    &raw mut szFullFilePath as STRSAFE_LPWSTR,
                                    (_countof!(szFullFilePath)) as _,
                                    *argv.offset(i as isize) as STRSAFE_LPCWSTR,
                                    ptr.offset(1 as isize)
                                        .offset_from(*argv.offset(i as isize))
                                        as ::core::ffi::c_longlong
                                        as size_t,
                                );
                                StringCchCatW(
                                    &raw mut szFullFilePath as STRSAFE_LPWSTR,
                                    (_countof!(szFullFilePath)) as _,
                                    &raw mut FindData.cFileName as *mut WCHAR as STRSAFE_LPCWSTR,
                                );
                            } else {
                                /* The pattern does not contain any drive or directory part: just copy the full file name */
                                StringCchCopyW(
                                    &raw mut szFullFilePath as STRSAFE_LPWSTR,
                                    (_countof!(szFullFilePath)) as _,
                                    &raw mut FindData.cFileName as *mut WCHAR as STRSAFE_LPCWSTR,
                                );
                            }

                            // FIXME: Windows' find.exe supports searching inside binary files.
                            pOpenedFile = _wfopen(
                                &raw mut szFullFilePath as *mut WCHAR,
                                RUSTCONV_WSTR_D877BF4E5023.as_ptr(),
                            ) as *mut FILE;
                            if pOpenedFile.is_null() {
                                ConResPrintf(
                                    &raw mut csStdErr,
                                    IDS_CANNOT_OPEN,
                                    &raw mut szFullFilePath as *mut WCHAR,
                                );
                                break 'body;
                            }

                            /* NOTE: Convert the file path to uppercase for formatting */
                            if FindString(
                                pOpenedFile,
                                _wcsupr(&raw mut szFullFilePath as *mut wchar_t) as PCWSTR,
                                *argv.offset(iSearchedStringIndex as isize) as PCWSTR,
                            ) == 0
                            {
                                iReturnValue = 0;
                            } else if iReturnValue != 0 {
                                iReturnValue = 1;
                            }

                            fclose(pOpenedFile);
                        }
                        if !(FindNextFileW(hFindFile, &raw mut FindData) != 0) {
                            break;
                        }
                    }

                    FindClose(hFindFile);
                }
            }
            i += 1;
        }
    } else {
        iReturnValue = FindString(
            (__iob_func() as *mut FILE).offset(0 as ::core::ffi::c_int as isize) as *mut FILE,
            ::core::ptr::null::<WCHAR>(),
            *argv.offset(iSearchedStringIndex as isize) as PCWSTR,
        );
    }

    return iReturnValue;
}
