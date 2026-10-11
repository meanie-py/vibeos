// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_LABEL_LENGTH: usize = 32;

pub const MAX_DRIVE_LENGTH: usize = 2;

#[no_mangle]
unsafe extern "C" fn ConFormatMessage_v(Stream: PCON_STREAM, MessageId: DWORD, ap_list: *mut ::core::ffi::c_void) {
    let mut arg_ptr: va_list = *(ap_list as *mut va_list);

    ConMsgPrintfV(
        Stream,
        FORMAT_MESSAGE_FROM_SYSTEM,
        core::ptr::null_mut(),
        MessageId,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(arg_ptr),
    );
}



unsafe extern "C" fn ConInString(lpInput: LPWSTR, dwLength: DWORD) {
    let mut dwOldMode: DWORD = 0;
    let mut dwRead: DWORD = 0 as DWORD;
    let mut hFile: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut p: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut pBuf: PCHAR = ::core::ptr::null_mut::<CHAR>();

    pBuf = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        dwLength.wrapping_sub(1 as DWORD) as SIZE_T,
    ) as PCHAR;

    hFile = GetStdHandle(STD_INPUT_HANDLE as DWORD);
    GetConsoleMode(hFile, &raw mut dwOldMode);

    SetConsoleMode(
        hFile,
        (ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT) as DWORD,
    );

    ReadFile(
        hFile,
        pBuf as PVOID,
        dwLength.wrapping_sub(1 as DWORD),
        &raw mut dwRead,
        ::core::ptr::null_mut::<_OVERLAPPED>(),
    );

    MultiByteToWideChar(
        GetConsoleCP(),
        0 as DWORD,
        pBuf as LPCSTR,
        dwRead as ::core::ffi::c_int,
        lpInput,
        dwLength.wrapping_sub(1 as DWORD) as ::core::ffi::c_int,
    );
    HeapFree(GetProcessHeap(), 0 as DWORD, pBuf as PVOID);

    p = lpInput;
    while *p != 0 {
        if *p == 0x000Du16 {
            *p = UNICODE_NULL;
            break;
        } else {
            p = p.offset(1);
        }
    }

    SetConsoleMode(hFile, dwOldMode);
}

unsafe extern "C" fn IsValidPathName(pszPath: LPCWSTR) -> BOOL {
    let mut szOldPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut bResult: BOOL = 0;

    GetCurrentDirectoryW(MAX_PATH, szOldPath.as_mut_ptr());
    bResult = SetCurrentDirectoryW(pszPath);

    SetCurrentDirectoryW(szOldPath.as_mut_ptr());

    return bResult;
}

unsafe extern "C" fn PromptYesNo() -> BOOL {
    let mut szOptions: [WCHAR; 4] = [0; 4];
    let mut szInput: [WCHAR; 16] = [0; 16];
    let mut bResult: BOOL = (FALSE as i32);

    LoadStringW(core::ptr::null_mut(), STRING_LABEL_OPTIONS, szOptions.as_mut_ptr(), ARRAYSIZE!(szOptions) as INT);

    loop {
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        ConResPuts(StdOut, STRING_LABEL_PROMPT);

        ConInString(szInput.as_mut_ptr(), ARRAYSIZE!(szInput) as DWORD);

        if towupper((szInput[0 as usize]) as _) as WCHAR == szOptions[0 as usize]
        {
            bResult = (TRUE as i32);
            break;
        }
        else if towupper((szInput[0 as usize]) as _) as WCHAR == szOptions[1 as usize]
        {
            bResult = (FALSE as i32);
            break;
        }

        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return bResult;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut szRootPath: [WCHAR; 4] = [0x0020u16, 0x003A, 0x005C, 0x0000];
    let mut szBuffer: [WCHAR; 80] = [0; 80];
    let mut szLabel: [WCHAR; 80] = [0; 80];
    let mut szOldLabel: [WCHAR; 80] = [0; 80];
    let mut dwSerialNr: DWORD = 0;
    let mut i: INT = 0;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* print help */
    if argc > 1
        && wcscmp(
            *argv.offset(1 as isize),
            RUSTCONV_WSTR_5462AED96D21.as_ptr(),
        ) == 0
    {
        ConResPuts(
            core::ptr::addr_of_mut!(csStdOut),
            STRING_LABEL_HELP as UINT,
        );
        return 0;
    }

    if argc > 1 {
        let mut len: size_t = 0 as size_t;
        i = 1 as INT;
        while i < argc {
            if i > 1 {
                len += 1;
            }
            len = len.wrapping_add(wcslen(*argv.offset(i as isize)));
            i += 1;
        }

        if len > (MAX_LABEL_LENGTH + MAX_DRIVE_LENGTH) as size_t {
            ConResPuts(
                core::ptr::addr_of_mut!(csStdOut),
                STRING_ERROR_INVALID_LABEL as UINT,
            );
            return 1;
        }

        i = 1 as INT;
        while i < argc {
            if i > 1 {
                wcscat(
                    core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
                    RUSTCONV_WSTR_8D71B3FAAB82.as_ptr(),
                );
            }
            wcscat(
                core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
                *argv.offset(i as isize),
            );
            i += 1;
        }
    }

    if wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR) > 0 as size_t {
        if szBuffer[1 as usize] == 0x003Au16 {
            szRootPath[0 as usize] =
                towupper((szBuffer[0 as usize]) as _) as WCHAR;
            wcscpy(
                core::ptr::addr_of_mut!(szLabel) as *mut WCHAR,
                (core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR)
                    .offset(2 as isize),
            );
        } else {
            wcscpy(
                core::ptr::addr_of_mut!(szLabel) as *mut WCHAR,
                core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
            );
        }
    }

    if wcslen(core::ptr::addr_of_mut!(szLabel) as *mut WCHAR) > MAX_LABEL_LENGTH as size_t {
        ConResPuts(
            core::ptr::addr_of_mut!(csStdOut),
            STRING_ERROR_INVALID_LABEL as UINT,
        );
        return 1;
    }

    if szRootPath[0 as usize] == 0x0020u16 {
        /* get label of current drive */
        let mut szCurPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        GetCurrentDirectoryW(
            MAX_PATH as DWORD,
            core::ptr::addr_of_mut!(szCurPath) as LPWSTR,
        );
        szRootPath[0 as usize] = szCurPath[0 as usize];
    }

    /* check root path */
    if IsValidPathName(core::ptr::addr_of_mut!(szRootPath) as *mut WCHAR as LPCWSTR) == 0 {
        ConResPuts(
            core::ptr::addr_of_mut!(csStdErr),
            STRING_ERROR_INVALID_DRIVE as UINT,
        );
        return 1;
    }

    if wcslen(core::ptr::addr_of_mut!(szLabel) as *mut WCHAR) == 0 as size_t {
        GetVolumeInformationW(
            core::ptr::addr_of_mut!(szRootPath) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(szOldLabel) as LPWSTR,
            ARRAYSIZE!(szOldLabel),
            core::ptr::addr_of_mut!(dwSerialNr),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<WCHAR>(),
            0 as DWORD,
        );

        /* print drive info */
        if szOldLabel[0 as usize] != UNICODE_NULL as WCHAR {
            ConResPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                STRING_LABEL_TEXT1 as UINT,
                towupper((szRootPath[0 as usize]) as _) as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(szOldLabel) as *mut WCHAR,
            );
        } else {
            ConResPrintf(
                core::ptr::addr_of_mut!(csStdOut),
                STRING_LABEL_TEXT2 as UINT,
                towupper((szRootPath[0 as usize]) as _) as ::core::ffi::c_int,
            );
        }

        /* print the volume serial number */
        ConResPrintf(
            core::ptr::addr_of_mut!(csStdOut),
            STRING_LABEL_TEXT3 as UINT,
            HIWORD!(dwSerialNr) as ::core::ffi::c_int,
            LOWORD!(dwSerialNr) as ::core::ffi::c_int,
        );

        ConResPuts(
            core::ptr::addr_of_mut!(csStdOut),
            STRING_LABEL_TEXT4 as UINT,
        );

        ConInString(
            core::ptr::addr_of_mut!(szLabel) as LPWSTR,
            ARRAYSIZE!(szLabel),
        );
        ConPuts(
            core::ptr::addr_of_mut!(csStdOut),
            RUSTCONV_WSTR_075DE2B906DB.as_ptr(),
        );

        if wcslen(core::ptr::addr_of_mut!(szLabel) as *mut WCHAR) == 0 as size_t {
            if PromptYesNo() == (FALSE as i32) {
                return 0;
            }
        }
    }

    if SetVolumeLabelW(
        core::ptr::addr_of_mut!(szRootPath) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szLabel) as *mut WCHAR as LPCWSTR,
    ) == 0
    {
        ConFormatMessage(
            core::ptr::addr_of_mut!(csStdOut),
            GetLastError(),
        );
        return 1;
    }

    return 0;
}
