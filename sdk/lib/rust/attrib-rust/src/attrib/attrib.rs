// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_48D07AA50093: [u16; 19] = [
    0x0025u16, 0x0063, 0x0020, 0x0020, 0x0025, 0x0063, 0x0025, 0x0063, 0x0025, 0x0063,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_77ECA80E282A: [u16; 3] = [0x002Eu16, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9E6B93EF3A85: [u16; 3] = [0x002Fu16, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_B0232478EA10: [u16; 3] = [0x002Fu16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_BCF3913447D5: [u16; 4] = [0x0025u16, 0x0073, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_C245AFF91259: [u16; 2] = [0x002Eu16, 0x0000];
pub static RUSTCONV_WSTR_EE82350A57BF: [u16; 30] = [
    0x0044u16, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079, 0x0020,
    0x006C, 0x0065, 0x0076, 0x0065, 0x006C, 0x0020, 0x0074, 0x006F, 0x006F, 0x0020,
    0x0064, 0x0065, 0x0065, 0x0070, 0x003A, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F503AE040218: [u16; 6] = [0x0020u16, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut ALL_FILES_PATTERN: [::core::ffi::c_ushort; 4] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_ushort; 4]>(*b"*\0.\0*\0\0\0") };

#[no_mangle]
pub static mut StdOutScreen: CON_SCREEN = unsafe {
    _CON_SCREEN {
        Stream: (&raw const csStdOut as PCON_STREAM).cast(),
        csbi: _CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: _COORD { X: 0, Y: 0 },
            dwCursorPosition: _COORD { X: 0, Y: 0 },
            wAttributes: 0,
            srWindow: _SMALL_RECT {
                Left: 0,
                Top: 0,
                Right: 0,
                Bottom: 0,
            },
            dwMaximumWindowSize: _COORD { X: 0, Y: 0 },
        },
        cci: _CONSOLE_CURSOR_INFO {
            dwSize: 0,
            bVisible: 0,
        },
    }
};

// NOTE: INIT_CON_SCREEN is not in the provided preamble; its expansion must be supplied by the crate.

#[repr(C)]
pub struct _ATTRIBS_MASKS {
    pub dwMask: DWORD,
    pub dwAttrib: DWORD,
}
pub type ATTRIBS_MASKS = _ATTRIBS_MASKS;
pub type PATTRIBS_MASKS = *mut _ATTRIBS_MASKS;

pub const ENUM_RECURSE: DWORD = 0x01;

pub const ENUM_DIRECTORIES: DWORD = 0x02;

pub type PENUMFILES_CALLBACK = unsafe extern "C" fn(
    pFindData: PWIN32_FIND_DATAW,
    pszFullName: PCWSTR,
    Context: PVOID,
) -> BOOL;

#[repr(C)]
pub struct _ENUMFILES_CTX {
    /* Fixed data */
    pub FileName: PCWSTR,
    pub Flags: DWORD,

    /* Callback invoked on each enumerated file/directory */
    pub Callback: PENUMFILES_CALLBACK,
    pub Context: PVOID,

    /* Dynamic data */
    pub findData: WIN32_FIND_DATAW,
    pub uReparseLevel: ULONG,

    /* The full path buffer the function will act recursively */
    // PWSTR FullPath; // Use a relocated buffer once long paths become supported!
    pub cchBuffer: usize, // Buffer size
    pub FullPathBuffer: [WCHAR; (MAX_PATH as usize) + 5], // MAX_PATH + _countof(L"\\" L"*.*")
}
pub type ENUMFILES_CTX = _ENUMFILES_CTX;
pub type PENUMFILES_CTX = *mut _ENUMFILES_CTX;

#[no_mangle]
unsafe extern "C" fn ErrorMessage_v(dwErrorCode: DWORD, pszMsg: PCWSTR, ap_list: *mut ::core::ffi::c_void) {
    let mut Len: INT = 0;
    let mut arg_ptr: va_list = *(ap_list as *mut va_list);
    if dwErrorCode == ERROR_SUCCESS {
        return;
    }

    Len = ConMsgPrintfV(
        StdErr,
        FORMAT_MESSAGE_FROM_SYSTEM,
        core::ptr::null::<core::ffi::c_void>(),
        dwErrorCode,
        LANG_USER_DEFAULT,
        &raw mut arg_ptr,
    );

    /* Fall back just in case the error is not defined */
    if Len <= 0 {
        ConResPrintf(StdErr, STRING_CONSOLE_ERROR, dwErrorCode);
    }
    /* Display the extra optional message if necessary */
    if !pszMsg.is_null() {
        ConPrintf(StdErr, RUSTCONV_WSTR_F503AE040218.as_ptr(), pszMsg);
    }
}



unsafe extern "C" fn PrintAttributes(pFindData: PWIN32_FIND_DATAW, pszFullName: PCWSTR, Context: PVOID) -> BOOL {
    let dwAttributes: DWORD = (*pFindData).dwFileAttributes;

    UNREFERENCED_PARAMETER!(Context);

    ConPrintf(
        StdOut,
        RUSTCONV_WSTR_48D07AA50093.as_ptr(),
        (if (dwAttributes & FILE_ATTRIBUTE_ARCHIVE) != 0 { 0x0041u16 } else { 0x0020u16 }) as core::ffi::c_uint,
        (if (dwAttributes & FILE_ATTRIBUTE_SYSTEM) != 0 { 0x0053u16 } else { 0x0020u16 }) as core::ffi::c_uint,
        (if (dwAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 { 0x0048u16 } else { 0x0020u16 }) as core::ffi::c_uint,
        (if (dwAttributes & FILE_ATTRIBUTE_READONLY) != 0 { 0x0052u16 } else { 0x0020u16 }) as core::ffi::c_uint,
        pszFullName,
    );

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ChangeAttributes(mut pFindData: PWIN32_FIND_DATAW, mut pszFullName: PCWSTR, mut Context: PVOID) -> BOOL {
    let mut AttribsMasks: PATTRIBS_MASKS = Context as PATTRIBS_MASKS;
    let mut dwAttributes: DWORD = 0;
    dwAttributes =
        ((*pFindData).dwFileAttributes & !(*AttribsMasks).dwMask) | (*AttribsMasks).dwAttrib;
    return SetFileAttributesW(pszFullName, dwAttributes);
}

unsafe extern "C" fn EnumFilesWorker(mut EnumCtx: PENUMFILES_CTX, mut offFilePart: size_t) -> BOOL {
    let mut bFound: BOOL = (FALSE) as i32;
    let mut hRes: HRESULT;
    let mut hFind: HANDLE;
    let mut findFileName: PWSTR = &raw mut (*EnumCtx).findData.cFileName as PWSTR;
    let mut pFilePart: PWSTR = (&raw mut (*EnumCtx).FullPathBuffer as *mut WCHAR).add(offFilePart);
    let mut cchRemaining: size_t = (*EnumCtx).cchBuffer.wrapping_sub(offFilePart);

    /* Recurse over all subdirectories */
    if ((*EnumCtx).Flags & ENUM_RECURSE) != 0 {
        /* Append '*.*' */
        hRes = StringCchCopyW(pFilePart, cchRemaining, (ALL_FILES_PATTERN).as_ptr());
        if hRes != S_OK {
            if hRes == STRSAFE_E_INSUFFICIENT_BUFFER {
                // TODO: If this fails, try to reallocate EnumCtx->FullPathBuffer by
                // increasing its size by _countof(EnumCtx->findData.cFileName) + 1
                // to satisfy this copy, as well as the one made in the loop below.
            }
            // else
            ConPrintf(&raw mut csStdErr, RUSTCONV_WSTR_EE82350A57BF.as_ptr(), &raw mut (*EnumCtx).FullPathBuffer as *mut WCHAR);
            return (FALSE) as BOOL;
        }

        hFind = FindFirstFileW(&raw mut (*EnumCtx).FullPathBuffer as *mut WCHAR, &raw mut (*EnumCtx).findData);
        if hFind == INVALID_HANDLE_VALUE {
            let mut Error: DWORD = GetLastError();
            if (Error != ERROR_DIRECTORY)
                && (Error != ERROR_SHARING_VIOLATION)
                && (Error != ERROR_FILE_NOT_FOUND)
            {
                ErrorMessage(Error, &raw mut (*EnumCtx).FullPathBuffer as *mut WCHAR);
            }
            return (FALSE) as BOOL;
        }

        loop {
            'body: {
                let mut bIsReparse: BOOL = 0;
                let mut offNewFilePart: size_t = 0;

                if ((*EnumCtx).findData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
                    break 'body;
                }

                if wcscmp(findFileName as *const u16, RUSTCONV_WSTR_C245AFF91259.as_ptr()) == 0
                    || wcscmp(findFileName as *const u16, RUSTCONV_WSTR_77ECA80E282A.as_ptr()) == 0
                {
                    break 'body;
                }

                /* Allow at most 2 levels of reparse points / symbolic links */
                bIsReparse = (((*EnumCtx).findData.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0) as BOOL;
                if bIsReparse != 0 {
                    if (*EnumCtx).uReparseLevel < 2 {
                        (*EnumCtx).uReparseLevel = (*EnumCtx).uReparseLevel.wrapping_add(1);
                    } else {
                        break 'body;
                    }
                }

                hRes = StringCchPrintfExW!(
                    pFilePart,
                    cchRemaining,
                    core::ptr::null_mut(),
                    &raw mut offNewFilePart,
                    0,
                    RUSTCONV_WSTR_BCF3913447D5.as_ptr(),
                    findFileName,
                );
                /* Offset to the new file name part */
                offNewFilePart = (*EnumCtx).cchBuffer.wrapping_sub(offNewFilePart);

                bFound |= EnumFilesWorker(EnumCtx, offNewFilePart);

                /* Recalculate the file part pointer and the number of characters
                 * remaining: the buffer may have been enlarged and relocated. */
                pFilePart = (&raw mut (*EnumCtx).FullPathBuffer as *mut WCHAR).add(offFilePart);
                cchRemaining = (*EnumCtx).cchBuffer.wrapping_sub(offFilePart);

                /* If we went through a reparse point / symbolic link, decrease level */
                if bIsReparse != 0 {
                    (*EnumCtx).uReparseLevel = (*EnumCtx).uReparseLevel.wrapping_sub(1);
                }
            }
            if FindNextFileW(hFind, &raw mut (*EnumCtx).findData) == 0 {
                break;
            }
        }
        FindClose(hFind);
    }

    /* Append the file name pattern to search for */
    hRes = StringCchCopyW(pFilePart, cchRemaining, (*EnumCtx).FileName);

    /* Search in the current directory */
    hFind = FindFirstFileW(&raw mut (*EnumCtx).FullPathBuffer as *mut WCHAR, &raw mut (*EnumCtx).findData);
    if hFind == INVALID_HANDLE_VALUE {
        return bFound;
    }

    loop {
        'body2: {
            let mut bIsDir: BOOL = (((*EnumCtx).findData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0) as BOOL;
            let mut bExactMatch: BOOL = (_wcsicmp(findFileName as *const u16, (*EnumCtx).FileName) == 0) as BOOL;

            if bIsDir != 0 && ((*EnumCtx).Flags & ENUM_DIRECTORIES) == 0 && bExactMatch == 0 {
                break 'body2;
            }

            if wcscmp(findFileName as *const u16, RUSTCONV_WSTR_C245AFF91259.as_ptr()) == 0
                || wcscmp(findFileName as *const u16, RUSTCONV_WSTR_77ECA80E282A.as_ptr()) == 0
            {
                break 'body2;
            }

            /* If we recursively enumerate files excluding directories,
             * exclude any directory from the enumeration. */
            if bIsDir != 0
                && ((*EnumCtx).Flags & ENUM_DIRECTORIES) == 0
                && ((*EnumCtx).Flags & ENUM_RECURSE) != 0
            {
                break 'body2;
            }

            StringCchCopyW(pFilePart, cchRemaining, findFileName);
            /* bFound = */ ((*EnumCtx).Callback)(
                &raw mut (*EnumCtx).findData,
                &raw mut (*EnumCtx).FullPathBuffer as *mut WCHAR as PCWSTR,
                (*EnumCtx).Context,
            );
            bFound = (TRUE) as i32;
        }
        if FindNextFileW(hFind, &raw mut (*EnumCtx).findData) == 0 {
            break;
        }
    }
    FindClose(hFind);

    return bFound;
}

unsafe extern "C" fn AttribEnumFiles(
    pszPath: PCWSTR,
    pszFile: PCWSTR,
    fFlags: DWORD,
    AttribsMasks: PATTRIBS_MASKS,
) -> BOOL {
    let mut EnumContext: ENUMFILES_CTX = core::mem::zeroed();
    let mut offFilePart: size_t = 0;
    let mut hRes: HRESULT = 0;

    EnumContext.FileName = pszFile;
    EnumContext.Flags = fFlags;
    EnumContext.Callback = (if (*AttribsMasks).dwMask == 0 {
        PrintAttributes as unsafe extern "C" fn(PWIN32_FIND_DATAW, PCWSTR, PVOID) -> BOOL
    } else {
        ChangeAttributes as unsafe extern "C" fn(PWIN32_FIND_DATAW, PCWSTR, PVOID) -> BOOL
    }) as PENUMFILES_CALLBACK;
    EnumContext.Context = (if (*AttribsMasks).dwMask == 0 {
        core::ptr::null_mut::<_ATTRIBS_MASKS>()
    } else {
        AttribsMasks
    }) as PVOID;

    /* Prepare the full file path buffer */
    EnumContext.cchBuffer = _countof!(EnumContext.FullPathBuffer) as size_t;
    hRes = StringCchCopyExW(
        core::ptr::addr_of_mut!(EnumContext.FullPathBuffer) as STRSAFE_LPWSTR,
        EnumContext.cchBuffer,
        pszPath as STRSAFE_LPCWSTR,
        core::ptr::null_mut::<STRSAFE_LPWSTR>(),
        core::ptr::addr_of_mut!(offFilePart),
        0 as STRSAFE_DWORD,
    );
    if hRes != S_OK {
        return (FALSE) as BOOL;
    }

    /* Offset to the file name part */
    offFilePart = EnumContext.cchBuffer.wrapping_sub(offFilePart);
    if EnumContext.FullPathBuffer[offFilePart.wrapping_sub(1 as size_t) as usize]
        != 0x005Cu16
    {
        EnumContext.FullPathBuffer[offFilePart as usize] = 0x005Cu16;
        EnumContext.FullPathBuffer[offFilePart.wrapping_add(1 as size_t) as usize] =
            UNICODE_NULL as WCHAR;
        offFilePart = offFilePart.wrapping_add(1);
    }

    return EnumFilesWorker(core::ptr::addr_of_mut!(EnumContext), offFilePart);
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 1;
    let mut dwEnumFlags: DWORD = 0;
    let mut AttribsMasks: ATTRIBS_MASKS = ATTRIBS_MASKS { dwMask: 0, dwAttrib: 0 };
    let mut bFound: BOOL = (FALSE) as i32;
    let mut pszFileName: PWSTR = core::ptr::null_mut();
    let mut szFilePath: [WCHAR; (MAX_PATH + 2) as _] =
        [0; (MAX_PATH + 2) as _];

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* Check for options and file specifications */
    i = 1;
    while i < argc {
        if *(*argv.offset(i as isize)) == 0x002Fu16 {
            /* Print help and bail out if needed */
            if wcscmp(*argv.offset(i as isize), RUSTCONV_WSTR_5462AED96D21.as_ptr()) == 0 {
                ConResPuts(StdOut, STRING_ATTRIB_HELP);
                return 0;
            } else
            /* Retrieve the enumeration modes */
            if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_9E6B93EF3A85.as_ptr()) == 0 {
                dwEnumFlags |= ENUM_RECURSE;
            } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_B0232478EA10.as_ptr()) == 0 {
                dwEnumFlags |= ENUM_DIRECTORIES;
            } else {
                /* Unknown option */
                ConResPrintf(StdErr, STRING_ERROR_INVALID_PARAM_FORMAT, *argv.offset(i as isize));
                return -1;
            }
        } else
        /* Build attributes and mask */
        if (*(*argv.offset(i as isize)) == 0x002Bu16)
            || (*(*argv.offset(i as isize)) == 0x002Du16)
        {
            let mut bAdd: BOOL = (*(*argv.offset(i as isize)) == 0x002Bu16) as BOOL;

            if wcslen(*argv.offset(i as isize)) != 2 {
                ConResPrintf(StdErr, STRING_ERROR_INVALID_PARAM_FORMAT, *argv.offset(i as isize));
                return -1;
            }

            match towupper((*(*argv.offset(i as isize)).offset(1 as isize)) as _) as u16 {
                0x0041u16 => {
                    AttribsMasks.dwMask |= FILE_ATTRIBUTE_ARCHIVE;
                    if bAdd != 0 {
                        AttribsMasks.dwAttrib |= FILE_ATTRIBUTE_ARCHIVE;
                    } else {
                        AttribsMasks.dwAttrib &= !FILE_ATTRIBUTE_ARCHIVE;
                    }
                }

                0x0053u16 => {
                    AttribsMasks.dwMask |= FILE_ATTRIBUTE_SYSTEM;
                    if bAdd != 0 {
                        AttribsMasks.dwAttrib |= FILE_ATTRIBUTE_SYSTEM;
                    } else {
                        AttribsMasks.dwAttrib &= !FILE_ATTRIBUTE_SYSTEM;
                    }
                }

                0x0048u16 => {
                    AttribsMasks.dwMask |= FILE_ATTRIBUTE_HIDDEN;
                    if bAdd != 0 {
                        AttribsMasks.dwAttrib |= FILE_ATTRIBUTE_HIDDEN;
                    } else {
                        AttribsMasks.dwAttrib &= !FILE_ATTRIBUTE_HIDDEN;
                    }
                }

                0x0052u16 => {
                    AttribsMasks.dwMask |= FILE_ATTRIBUTE_READONLY;
                    if bAdd != 0 {
                        AttribsMasks.dwAttrib |= FILE_ATTRIBUTE_READONLY;
                    } else {
                        AttribsMasks.dwAttrib &= !FILE_ATTRIBUTE_READONLY;
                    }
                }

                _ => {
                    ConResPrintf(StdErr, STRING_ERROR_INVALID_PARAM_FORMAT, *argv.offset(i as isize));
                    return -1;
                }
            }
        } else {
            /* At least one file specification found */
            bFound = (TRUE) as i32;
        }
        i += 1;
    }

    /* If no file specification was found, operate on all files of the current directory */
    if bFound == 0 {
        GetCurrentDirectoryW(
            _countof!(szFilePath) - 2,
            core::ptr::addr_of_mut!(szFilePath) as *mut WCHAR,
        );
        pszFileName = (ALL_FILES_PATTERN).as_mut_ptr();

        bFound = AttribEnumFiles(
            core::ptr::addr_of_mut!(szFilePath) as *mut WCHAR as PCWSTR,
            pszFileName as PCWSTR,
            dwEnumFlags,
            core::ptr::addr_of_mut!(AttribsMasks),
        );
        if bFound == 0 {
            ConResPrintf(StdOut, STRING_FILE_NOT_FOUND, pszFileName);
        }

        return 0;
    }

    /* Operate on each file specification */
    i = 1;
    while i < argc {
        'body: {
            /* Skip options */
            if *(*argv.offset(i as isize)) == 0x002Fu16
                || *(*argv.offset(i as isize)) == 0x002Bu16
                || *(*argv.offset(i as isize)) == 0x002Du16
            {
                break 'body;
            }

            GetFullPathNameW(
                *argv.offset(i as isize),
                _countof!(szFilePath) - 2,
                core::ptr::addr_of_mut!(szFilePath) as *mut WCHAR,
                core::ptr::addr_of_mut!(pszFileName),
            );
            if !pszFileName.is_null() {
                /* Move the file part so as to separate and NULL-terminate the directory */
                MoveMemory!(
                    pszFileName.offset(1 as isize) as *mut core::ffi::c_void,
                    pszFileName as *const core::ffi::c_void,
                    core::mem::size_of::<[WCHAR; (MAX_PATH + 2) as _]>()
                        - (((pszFileName.offset_from(core::ptr::addr_of_mut!(szFilePath) as *mut WCHAR)
                            + 1 as isize) as usize)
                            * core::mem::size_of::<WCHAR>()),
                );
                *pszFileName = UNICODE_NULL;
                pszFileName = pszFileName.offset(1);
            } else {
                pszFileName = RUSTCONV_WSTR_96A296D224F2.as_ptr() as PWSTR;
            }

            bFound = AttribEnumFiles(
                core::ptr::addr_of_mut!(szFilePath) as *mut WCHAR as PCWSTR,
                pszFileName as PCWSTR,
                dwEnumFlags,
                core::ptr::addr_of_mut!(AttribsMasks),
            );
            if bFound == 0 {
                ConResPrintf(StdOut, STRING_FILE_NOT_FOUND, *argv.offset(i as isize));
            }
        }
        i += 1;
    }

    return 0;
}
