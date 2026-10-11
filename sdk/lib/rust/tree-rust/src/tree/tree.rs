// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_041F8C928EFC: [u16; 2] = [0x007Cu16, 0x0000];
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_345B582FAAB7: [u16; 7] = [0x007Cu16, 0x0020, 0x0020, 0x0020, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_372DBC563AAB: [u16; 7] = [0x005Cu16, 0x002D, 0x002D, 0x002D, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_4D70F4A881C7: [u16; 2] = [0x003Au16, 0x0000];
pub static RUSTCONV_WSTR_77ECA80E282A: [u16; 3] = [0x002Eu16, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_8262504236A0: [u16; 7] = [0x2502u16, 0x0020, 0x0020, 0x0020, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A3AE38A2DDEC: [u16; 2] = [0x2502u16, 0x0000];
pub static RUSTCONV_WSTR_AB653EEA3FB6: [u16; 7] = [0x0020u16, 0x0020, 0x0020, 0x0020, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_C245AFF91259: [u16; 2] = [0x002Eu16, 0x0000];
pub static RUSTCONV_WSTR_C562B1A82C6D: [u16; 5] = [0x005Cu16, 0x002A, 0x002E, 0x002A, 0x0000];
pub static RUSTCONV_WSTR_DED4A41E9F23: [u16; 4] = [0x005Cu16, 0x002A, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_E2751960087C: [u16; 7] = [0x251Cu16, 0x2500, 0x2500, 0x2500, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_E474BB7CE79A: [u16; 6] = [0x0025u16, 0x0073, 0x003A, 0x002E, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_E912265E646E: [u16; 11] = [
    0x0020u16, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0000,
];
pub static RUSTCONV_WSTR_EBB16D97FA22: [u16; 7] = [0x002Bu16, 0x002D, 0x002D, 0x002D, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_FDEEDD469EBB: [u16; 7] = [0x2514u16, 0x2500, 0x2500, 0x2500, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut bShowFiles: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut bUseAscii: BOOL = (FALSE as i32);

pub const STR_MAX: i32 = 2048;

unsafe extern "C" fn HasSubFolder(strPath1: PCWSTR) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut FindFileData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();
    static mut strPath: [WCHAR; (STR_MAX) as usize] = [0; (STR_MAX) as usize];
    ZeroMemory!(
        core::ptr::addr_of_mut!(strPath) as *mut WCHAR as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (STR_MAX) as usize]>(),
    );

    wcscat(core::ptr::addr_of_mut!(strPath) as *mut WCHAR, strPath1);
    wcscat(core::ptr::addr_of_mut!(strPath) as *mut WCHAR, RUSTCONV_WSTR_DED4A41E9F23.as_ptr());

    hFind = FindFirstFileW(
        core::ptr::addr_of_mut!(strPath) as *mut WCHAR,
        core::ptr::addr_of_mut!(FindFileData),
    );
    'dowhile: loop {
        'body: {
            if (FindFileData.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY as DWORD)) != 0 {
                if wcscmp(
                    core::ptr::addr_of_mut!(FindFileData.cFileName) as *mut WCHAR,
                    RUSTCONV_WSTR_C245AFF91259.as_ptr(),
                ) == 0
                    || wcscmp(
                        core::ptr::addr_of_mut!(FindFileData.cFileName) as *mut WCHAR,
                        RUSTCONV_WSTR_77ECA80E282A.as_ptr(),
                    ) == 0
                {
                    break 'body;
                }

                ret = (TRUE as i32); // Sub-folder found
                break 'dowhile;
            }
        }
        if !(FindNextFileW(hFind, core::ptr::addr_of_mut!(FindFileData)) != 0) {
            break 'dowhile;
        }
    }

    FindClose(hFind);
    return ret;
}

unsafe extern "C" fn DrawTree(
    strPath: PCWSTR,
    arrFolder: *const WIN32_FIND_DATAW,
    szArr: size_t,
    width: UINT,
    prevLine: PCWSTR,
    drawfolder: BOOL,
) {
    let bHasSubFolder: BOOL = HasSubFolder(strPath);
    let mut i: UINT = 0;

    /* This will format the spaces required for correct formatting */
    i = 0;
    while (i as size_t) < szArr {
        let mut consoleOut: PWSTR = malloc(
            (STR_MAX as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
        ) as PWSTR;
        let mut j: UINT = 0;
        static mut str: [WCHAR; (STR_MAX) as usize] = [0; (STR_MAX) as usize];

        /* As we do not seem to have the _s functions properly set up, use the non-secure version for now */
        //wcscpy_s(consoleOut, STR_MAX, L"");
        //wcscpy_s(str, STR_MAX, L"");
        wcscpy(consoleOut, RUSTCONV_WSTR_96A296D224F2.as_ptr());
        wcscpy(&raw mut str as *mut WCHAR, RUSTCONV_WSTR_96A296D224F2.as_ptr());

        j = 0;
        while j < width.wrapping_sub(1) {
            /* If the previous line has '├' or '│' then the current line will
               add '│' to continue the connecting line */
            if *prevLine.offset(j as isize) == 0x251Cu16
                || *prevLine.offset(j as isize) == 0x2502u16
                || *prevLine.offset(j as isize) == 0x002Bu16
                || *prevLine.offset(j as isize) == 0x007Cu16
            {
                if bUseAscii == 0 {
                    wcscat(consoleOut, RUSTCONV_WSTR_A3AE38A2DDEC.as_ptr());
                } else {
                    wcscat(consoleOut, RUSTCONV_WSTR_041F8C928EFC.as_ptr());
                }
            } else {
                wcscat(consoleOut, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
            }
            j = j.wrapping_add(1);
        }

        if (szArr.wrapping_sub(1)) != i as size_t {
            if drawfolder != 0 {
                /* Add '├───Folder name' (\xC3\xC4\xC4\xC4 or \x251C\x2500\x2500\x2500) */
                if bUseAscii != 0 {
                    _swprintf(
                        &raw mut str as *mut WCHAR,
                        RUSTCONV_WSTR_EBB16D97FA22.as_ptr(),
                        core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                    );
                } else {
                    _swprintf(
                        &raw mut str as *mut WCHAR,
                        RUSTCONV_WSTR_E2751960087C.as_ptr(),
                        core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                    );
                }
            } else {
                if bHasSubFolder != 0 {
                    /* Add '│   FileName' (\xB3 or \x2502) */
                    // This line is added to connect the below-folder sub-structure
                    if bUseAscii != 0 {
                        _swprintf(
                            &raw mut str as *mut WCHAR,
                            RUSTCONV_WSTR_345B582FAAB7.as_ptr(),
                            core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                        );
                    } else {
                        _swprintf(
                            &raw mut str as *mut WCHAR,
                            RUSTCONV_WSTR_8262504236A0.as_ptr(),
                            core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                        );
                    }
                } else {
                    /* Add '    FileName' */
                    _swprintf(
                        &raw mut str as *mut WCHAR,
                        RUSTCONV_WSTR_AB653EEA3FB6.as_ptr(),
                        core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                    );
                }
            }
        } else {
            if drawfolder != 0 {
                /* '└───Folder name' (\xC0\xC4\xC4\xC4 or \x2514\x2500\x2500\x2500) */
                if bUseAscii != 0 {
                    _swprintf(
                        &raw mut str as *mut WCHAR,
                        RUSTCONV_WSTR_372DBC563AAB.as_ptr(),
                        core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                    );
                } else {
                    _swprintf(
                        &raw mut str as *mut WCHAR,
                        RUSTCONV_WSTR_FDEEDD469EBB.as_ptr(),
                        core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                    );
                }
            } else {
                if bHasSubFolder != 0 {
                    /* '│   FileName' (\xB3 or \x2502) */
                    if bUseAscii != 0 {
                        _swprintf(
                            &raw mut str as *mut WCHAR,
                            RUSTCONV_WSTR_345B582FAAB7.as_ptr(),
                            core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                        );
                    } else {
                        _swprintf(
                            &raw mut str as *mut WCHAR,
                            RUSTCONV_WSTR_8262504236A0.as_ptr(),
                            core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                        );
                    }
                } else {
                    /* '    FileName' */
                    _swprintf(
                        &raw mut str as *mut WCHAR,
                        RUSTCONV_WSTR_AB653EEA3FB6.as_ptr(),
                        core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
                    );
                }
            }
        }

        wcscat(consoleOut, &raw mut str as *mut WCHAR);
        ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), consoleOut);

        if drawfolder != 0 {
            let mut str_0: PWSTR = malloc(
                (STR_MAX as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            ) as PWSTR;
            ZeroMemory!(
                str_0 as *mut core::ffi::c_void,
                (STR_MAX as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            );

            wcscat(str_0, strPath);
            wcscat(str_0, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
            wcscat(
                str_0,
                core::ptr::addr_of!((*arrFolder.offset(i as isize)).cFileName) as *const WCHAR,
            );
            GetDirectoryStructure(str_0, width.wrapping_add(4), consoleOut as PCWSTR);

            free(str_0 as *mut core::ffi::c_void);
        }
        free(consoleOut as *mut core::ffi::c_void);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn GetDirectoryStructure(mut strPath: PWSTR, mut width: UINT, mut prevLine: PCWSTR) {
    let mut FindFileData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();
    /* Fill up with names of all sub-folders */
    let mut arrFolder: *mut WIN32_FIND_DATAW = core::ptr::null_mut();
    let mut arrFoldersz: UINT = 0;
    /* Fill up with names of all sub-folders */
    let mut arrFile: *mut WIN32_FIND_DATAW = core::ptr::null_mut();
    let mut arrFilesz: UINT = 0;

    ZeroMemory!(
        &raw mut FindFileData as *mut ::core::ffi::c_void,
        core::mem::size_of::<WIN32_FIND_DATAW>(),
    );

    {
        static mut tmp: [WCHAR; (STR_MAX) as usize] = [0; (STR_MAX) as usize];
        ZeroMemory!(
            &raw mut tmp as *mut WCHAR as *mut ::core::ffi::c_void,
            core::mem::size_of::<[WCHAR; (STR_MAX) as usize]>(),
        );
        wcscat(&raw mut tmp as *mut WCHAR, strPath);
        wcscat(&raw mut tmp as *mut WCHAR, RUSTCONV_WSTR_C562B1A82C6D.as_ptr());
        hFind = FindFirstFileW(
            &raw mut tmp as *mut WCHAR as LPCWSTR,
            &raw mut FindFileData,
        );
        //err = GetLastError();
    }

    if hFind == INVALID_HANDLE_VALUE {
        return;
    }

    loop {
        'body: {
            if (FindFileData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                if wcscmp(
                    &raw mut FindFileData.cFileName as *mut WCHAR,
                    RUSTCONV_WSTR_C245AFF91259.as_ptr(),
                ) == 0
                    || wcscmp(
                        &raw mut FindFileData.cFileName as *mut WCHAR,
                        RUSTCONV_WSTR_77ECA80E282A.as_ptr(),
                    ) == 0
                {
                    break 'body;
                }

                arrFoldersz = arrFoldersz.wrapping_add(1);
                arrFolder = realloc(
                    arrFolder as *mut ::core::ffi::c_void,
                    (arrFoldersz as usize) * core::mem::size_of::<WIN32_FIND_DATAW>(),
                ) as *mut WIN32_FIND_DATAW;

                if arrFolder.is_null() {
                    exit(-(1) as i32);
                }

                *arrFolder.offset((arrFoldersz - 1) as isize) = FindFileData;
            } else {
                arrFilesz = arrFilesz.wrapping_add(1);
                arrFile = realloc(
                    arrFile as *mut ::core::ffi::c_void,
                    (arrFilesz as usize) * core::mem::size_of::<WIN32_FIND_DATAW>(),
                ) as *mut WIN32_FIND_DATAW;

                if arrFile.is_null() {
                    exit(-(1) as i32);
                }

                *arrFile.offset((arrFilesz - 1) as isize) = FindFileData;
            }
        }
        if !(FindNextFileW(hFind, &raw mut FindFileData) != 0) {
            break;
        }
    }

    FindClose(hFind);

    if bShowFiles != 0 {
        /* Will free(arrFile) */
        DrawTree(
            strPath as PCWSTR,
            arrFile,
            arrFilesz as usize,
            width,
            prevLine,
            (FALSE) as BOOL,
        );
    }

    /* Will free(arrFile) */
    DrawTree(
        strPath as PCWSTR,
        arrFolder,
        arrFoldersz as usize,
        width,
        prevLine,
        (TRUE) as BOOL,
    );

    free(arrFolder as *mut ::core::ffi::c_void);
    free(arrFile as *mut ::core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut dwSerial: DWORD = 0;
    let mut t: WCHAR = 0;
    let mut strPath: PWSTR = core::ptr::null_mut();
    let mut sz: DWORD = 0;
    //PWSTR context = NULL;
    let mut driveLetter: PWSTR = core::ptr::null_mut();

    let mut i: ::core::ffi::c_int;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* Parse the command line */
    i = 1;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Du16
            || *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16
        {
            let sw: u32 = towlower(*(*argv.offset(i as isize)).offset(1 as isize)) as u32;
            if sw == 0x003Fu16 as u32 {
                /* Print help and exit after */
                ConResPuts(StdOut, IDS_USAGE);
                return 0;
            } else if sw == 0x0066u16 as u32 {
                bShowFiles = (TRUE as i32);
            } else if sw == 0x0061u16 as u32 {
                bUseAscii = (TRUE as i32);
            } else {
                /* default: break */
            }
        } else {
            /* This must be path to some folder */

            /* Set the current directory for this executable */
            let b: BOOL = SetCurrentDirectoryW(*argv.offset(i as isize));
            if b == (FALSE as i32) {
                ConResPuts(StdOut, IDS_NO_SUBDIRECTORIES);
                return 1;
            }
        }
        i += 1;
    }

    ConResPuts(StdOut, IDS_FOLDER_PATH);

    GetVolumeInformationW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(dwSerial),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    );
    ConResPrintf(
        StdOut,
        IDS_VOL_SERIAL,
        dwSerial >> 16,
        dwSerial & 0xffff,
    );

    /* get the buffer size */
    sz = GetCurrentDirectoryW(1, core::ptr::addr_of_mut!(t));
    /* must not return before calling delete[] */
    strPath = malloc((sz as usize) * core::mem::size_of::<WCHAR>()) as PWSTR;

    /* get the current directory */
    GetCurrentDirectoryW(sz, strPath);

    /* get the drive letter , must not return before calling delete[] */
    driveLetter = malloc((sz as usize) * core::mem::size_of::<WCHAR>()) as PWSTR;

    /* As we do not seem to have the _s functions properly set up, use the non-secure version for now */
    //wcscpy_s(driveLetter,sz,strPath);
    //wcstok_s(driveLetter,L":", &context); //parse for the drive letter
    wcscpy(driveLetter, strPath);
    wcstok(driveLetter, RUSTCONV_WSTR_4D70F4A881C7.as_ptr());

    ConPrintf(StdOut, RUSTCONV_WSTR_E474BB7CE79A.as_ptr(), driveLetter);

    free((driveLetter).cast());

    /* get the sub-directories within this current folder */
    GetDirectoryStructure(strPath, 1, RUSTCONV_WSTR_E912265E646E.as_ptr());

    free((strPath).cast());
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return 0;
}
