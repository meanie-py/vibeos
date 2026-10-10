// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn IsTextNonZeroASCII(pText: LPCVOID, mut dwSize: DWORD) -> BOOL {
    let mut pch: *const core::ffi::c_schar = pText as *const core::ffi::c_schar;
    while {
        let fresh = dwSize;
        dwSize = dwSize.wrapping_sub(1);
        fresh > 0
    } {
        if (*pch as core::ffi::c_int) <= 0 {
            return (FALSE) as BOOL;
        }

        pch = pch.offset(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn AnalyzeEncoding(pBytes: *const BYTE, dwSize: DWORD) -> ENCODING {
    let mut flags: INT = (IS_TEXT_UNICODE_STATISTICS | IS_TEXT_UNICODE_REVERSE_STATISTICS) as i32;

    if IsTextNonZeroASCII(pBytes as LPCVOID, dwSize) != 0 {
        return ENCODING_DEFAULT;
    }

    if IsTextUnicode(pBytes as LPCVOID, dwSize as INT, core::ptr::addr_of_mut!(flags)) != 0 {
        return ENCODING_UTF16LE;
    }

    if ((flags as u32) & IS_TEXT_UNICODE_REVERSE_MASK) == IS_TEXT_UNICODE_REVERSE_STATISTICS {
        return ENCODING_UTF16BE;
    }

    /* is it UTF-8? */
    if MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, pBytes as LPCSTR, dwSize as INT, core::ptr::null_mut(), 0) != 0 {
        return ENCODING_UTF8;
    }

    ENCODING_ANSI
}

unsafe extern "C" fn ReplaceNewLines(pszNew: LPWSTR, cchNew: SIZE_T, pszOld: LPCWSTR, cchOld: SIZE_T) {
    let mut bPrevCR: BOOL = (FALSE) as i32;
    let mut ichNew: SIZE_T;
    let mut ichOld: SIZE_T;

    ichNew = 0;
    ichOld = ichNew;
    while ichOld < cchOld {
        let ch: WCHAR = *pszOld.add((ichOld) as _);

        if ch == 0x000Au16 {
            if bPrevCR == 0 {
                *pszNew.add((ichNew) as _) = 0x000Du16;
                ichNew += 1;
                *pszNew.add((ichNew) as _) = 0x000Au16;
                ichNew += 1;
            }
        } else if ch == '\r' as u16 {
            *pszNew.add((ichNew) as _) = 0x000Du16;
            ichNew += 1;
            *pszNew.add((ichNew) as _) = 0x000Au16;
            ichNew += 1;
        } else {
            *pszNew.add((ichNew) as _) = ch;
            ichNew += 1;
        }

        bPrevCR = (ch == 0x000Du16) as BOOL;
        ichOld += 1;
    }

    *pszNew.add((ichNew) as _) = UNICODE_NULL;
    assert!(ichNew == cchNew);
}

unsafe extern "C" fn ProcessNewLinesAndNulls(
    phLocal: *mut HLOCAL,
    ppszText: *mut LPWSTR,
    pcchText: *mut SIZE_T,
    piEoln: *mut EOLN,
) -> BOOL {
    let mut ich: SIZE_T = 0;
    let mut cchText: SIZE_T = *pcchText;
    let mut adwEolnCount: [SIZE_T; 3] = [
        0,
        0,
        0,
    ];
    let mut cNonCRLFs: SIZE_T = 0;
    let mut cchNew: SIZE_T = 0;
    let mut pszText: LPWSTR = *ppszText;
    let mut pszNew: LPWSTR = core::ptr::null_mut();
    let mut iEoln: EOLN;
    let mut bPrevCR: BOOL = (FALSE) as i32;
    let mut hLocal: HLOCAL = core::ptr::null_mut();

    /* Replace '\0' with SPACE. Count newlines. */
    ich = 0;
    while ich < cchText {
        let mut ch: WCHAR = *pszText.offset(ich as isize);
        if ch == UNICODE_NULL {
            *pszText.offset(ich as isize) = 0x0020u16;
        }

        if ch == 0x000Au16 {
            if bPrevCR != 0 {
                adwEolnCount[EOLN_CR as usize] =
                    adwEolnCount[EOLN_CR as usize].wrapping_sub(1);
                adwEolnCount[EOLN_CRLF as usize] =
                    adwEolnCount[EOLN_CRLF as usize].wrapping_add(1);
            } else {
                adwEolnCount[EOLN_LF as usize] =
                    adwEolnCount[EOLN_LF as usize].wrapping_add(1);
            }
        } else if ch == '\r' as WCHAR {
            adwEolnCount[EOLN_CR as usize] =
                adwEolnCount[EOLN_CR as usize].wrapping_add(1);
        }

        bPrevCR = (ch == 0x000Du16) as BOOL;
        ich = ich.wrapping_add(1);
    }

    /* Choose the newline code */
    if adwEolnCount[EOLN_CR as usize]
        > adwEolnCount[EOLN_CRLF as usize]
    {
        iEoln = EOLN_CR;
    } else if adwEolnCount[EOLN_LF as usize]
        > adwEolnCount[EOLN_CRLF as usize]
    {
        iEoln = EOLN_LF;
    } else {
        iEoln = EOLN_CRLF;
    }

    cNonCRLFs = adwEolnCount[EOLN_CR as usize]
        .wrapping_add(adwEolnCount[EOLN_LF as usize]);
    if cNonCRLFs != 0 {
        /* Allocate a buffer for EM_SETHANDLE */
        cchNew = cchText.wrapping_add(cNonCRLFs);
        hLocal = LocalAlloc(
            LMEM_MOVEABLE as UINT,
            cchNew
                .wrapping_add(1 as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        );
        if hLocal.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return (FALSE) as BOOL; /* Failure */
        }

        pszNew = LocalLock(hLocal) as LPWSTR;
        if pszNew.is_null() {
            LocalFree(hLocal);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return (FALSE) as BOOL; /* Failure */
        }

        ReplaceNewLines(pszNew, cchNew, pszText as LPCWSTR, cchText);

        /* Replace with new data */
        LocalUnlock(*phLocal);
        LocalFree(*phLocal);
        *phLocal = hLocal;
        *ppszText = pszNew;
        *pcchText = cchNew;
    }

    *piEoln = iEoln;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ReadText(
    hFile: HANDLE,
    pencFile: *mut ENCODING,
    piEoln: *mut EOLN,
) -> HLOCAL {
    let mut pBytes: LPBYTE = core::ptr::null_mut();
    let mut pszText: LPWSTR = core::ptr::null_mut();
    let mut pszNewText: LPWSTR = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;
    let mut dwPos: DWORD = 0;
    let mut i: SIZE_T = 0;
    let mut cchText: SIZE_T = 0;
    let mut cbContent: SIZE_T = 0;
    let mut bSuccess: BOOL = (FALSE) as i32;
    let mut encFile: ENCODING = ENCODING_ANSI;
    let mut iCodePage: UINT = 0;
    let mut hMapping: HANDLE = INVALID_HANDLE_VALUE;
    let mut hNewLocal: HLOCAL = core::ptr::null_mut();

    'done: {
        dwSize = GetFileSize(hFile, core::ptr::null_mut());
        if dwSize == INVALID_FILE_SIZE {
            break 'done;
        }

        if dwSize == 0 {
            hNewLocal = LocalAlloc(
                LMEM_MOVEABLE as UINT,
                core::mem::size_of_val(&UNICODE_NULL) as SIZE_T,
            );
            if hNewLocal.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'done;
            }

            pszNewText = LocalLock(hNewLocal) as LPWSTR;
            if pszNewText.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'done;
            }

            *pszNewText = UNICODE_NULL;
            LocalUnlock(hNewLocal);

            *piEoln = EOLN_CRLF;
            *pencFile = ENCODING_DEFAULT;
            return hNewLocal;
        }

        hMapping = CreateFileMappingW(
            hFile,
            core::ptr::null_mut(),
            PAGE_READONLY as DWORD,
            0 as DWORD,
            0 as DWORD,
            core::ptr::null(),
        );
        if hMapping.is_null() {
            break 'done;
        }

        pBytes = MapViewOfFile(
            hMapping,
            FILE_MAP_READ as DWORD,
            0 as DWORD,
            0 as DWORD,
            dwSize as SIZE_T,
        ) as LPBYTE;
        if pBytes.is_null() {
            break 'done;
        }

        /* Look for Byte Order Marks */
        dwPos = 0;
        if (dwSize >= 2)
            && (*pBytes.offset(0 as isize) == 0xFF)
            && (*pBytes.offset(1 as isize) == 0xFE)
        {
            encFile = ENCODING_UTF16LE;
            dwPos = dwPos.wrapping_add(2);
        } else if (dwSize >= 2)
            && (*pBytes.offset(0 as isize) == 0xFE)
            && (*pBytes.offset(1 as isize) == 0xFF)
        {
            encFile = ENCODING_UTF16BE;
            dwPos = dwPos.wrapping_add(2);
        } else if (dwSize >= 3)
            && (*pBytes.offset(0 as isize) == 0xEF)
            && (*pBytes.offset(1 as isize) == 0xBB)
            && (*pBytes.offset(2 as isize) == 0xBF)
        {
            encFile = ENCODING_UTF8BOM;
            dwPos = dwPos.wrapping_add(3);
        } else {
            encFile = AnalyzeEncoding(pBytes as *const BYTE, dwSize);
        }

        /* switch (encFile) */
        if encFile == ENCODING_UTF16BE || encFile == ENCODING_UTF16LE {
            /* Allocate the buffer for EM_SETHANDLE */
            pszText = pBytes.add(dwPos as usize) as LPWSTR;
            cchText = ((dwSize.wrapping_sub(dwPos) as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>())) as _;
            if cchText
                >= ((MAXLONG as usize).wrapping_div(core::mem::size_of::<WCHAR>())) as _
            {
                SetLastError(ERROR_FILE_TOO_LARGE as DWORD);
                break 'done;
            }

            hNewLocal = LocalAlloc(
                LMEM_MOVEABLE as UINT,
                (cchText.wrapping_add((1 as usize) as _))
                    .wrapping_mul((core::mem::size_of::<WCHAR>()) as _),
            );
            if hNewLocal.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'done;
            }

            pszNewText = LocalLock(hNewLocal) as LPWSTR;
            if pszNewText.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'done;
            }

            CopyMemory!(
                pszNewText as *mut core::ffi::c_void,
                pszText as *const core::ffi::c_void,
                cchText.wrapping_mul((core::mem::size_of::<WCHAR>()) as _),
            );

            if encFile == ENCODING_UTF16BE {
                /* big endian; Swap bytes */
                let mut tmp: BYTE = 0;
                let mut pb: *mut BYTE = pszNewText as *mut BYTE;
                i = 0;
                while i < cchText.wrapping_mul((2 as usize) as _) {
                    tmp = *pb.offset(i as isize);
                    *pb.offset(i as isize) =
                        *pb.offset(i.wrapping_add((1 as usize) as _) as isize);
                    *pb.offset(i.wrapping_add((1 as usize) as _) as isize) = tmp;
                    i = i.wrapping_add((2 as usize) as _);
                }
            }
        } else if encFile == ENCODING_ANSI
            || encFile == ENCODING_UTF8
            || encFile == ENCODING_UTF8BOM
        {
            iCodePage = (if encFile == ENCODING_UTF8 || encFile == ENCODING_UTF8BOM
            {
                CP_UTF8
            } else {
                CP_ACP
            }) as UINT;
            cbContent = dwSize.wrapping_sub(dwPos) as SIZE_T;
            if cbContent
                >= ((MAXLONG as usize).wrapping_div(core::mem::size_of::<WCHAR>())) as _
            {
                SetLastError(ERROR_FILE_TOO_LARGE as DWORD);
                break 'done;
            }

            /* Allocate the buffer for EM_SETHANDLE */
            hNewLocal = LocalAlloc(
                LMEM_MOVEABLE as UINT,
                (cbContent.wrapping_add((1 as usize) as _))
                    .wrapping_mul((core::mem::size_of::<WCHAR>()) as _),
            );
            if hNewLocal.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'done;
            }

            pszNewText = LocalLock(hNewLocal) as LPWSTR;
            if pszNewText.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                break 'done;
            }

            /* Do conversion */
            cchText = 0;
            if cbContent > (0 as usize) as _ {
                cchText = MultiByteToWideChar(
                    iCodePage,
                    0 as DWORD,
                    pBytes.add(dwPos as usize) as LPCSTR,
                    cbContent as core::ffi::c_int,
                    pszNewText,
                    cbContent as core::ffi::c_int,
                ) as SIZE_T;
                if cchText == 0 {
                    break 'done;
                }
            }
        } else {
            // C: `DEFAULT_UNREACHABLE` (`__builtin_unreachable()` under GCC/clang). Every
            // ENCODING value has an arm above; an out-of-range one fails the read instead of
            // being undefined behaviour that writes through a NULL `pszNewText`.
            break 'done;
        }

        *pszNewText.add(cchText as usize) = UNICODE_NULL;

        if ProcessNewLinesAndNulls(
            core::ptr::addr_of_mut!(hNewLocal),
            core::ptr::addr_of_mut!(pszNewText),
            core::ptr::addr_of_mut!(cchText),
            piEoln,
        ) == 0
        {
            break 'done;
        }

        *pencFile = encFile;
        bSuccess = (TRUE) as i32;
    }

    /* done: */
    if !pBytes.is_null() {
        UnmapViewOfFile(pBytes as LPCVOID);
    }
    if hMapping != INVALID_HANDLE_VALUE {
        CloseHandle(hMapping);
    }
    if !pszNewText.is_null() {
        LocalUnlock(hNewLocal);
    }
    if bSuccess == 0 && !hNewLocal.is_null() {
        LocalFree(hNewLocal);
        hNewLocal = core::ptr::null_mut();
    }
    return hNewLocal;
}

unsafe extern "C" fn WriteEncodedText(
    hFile: HANDLE,
    pszText: LPCWSTR,
    dwTextLen: DWORD,
    encFile: ENCODING,
) -> BOOL {
    let mut pBytes: LPBYTE = core::ptr::null_mut();
    let mut pAllocBuffer: LPBYTE = core::ptr::null_mut();
    let mut dwPos: DWORD = 0;
    let mut dwByteCount: DWORD = 0;
    let mut buffer: [BYTE; 1024] = [0; 1024];
    let mut iCodePage: UINT = 0;
    let mut dwDummy: DWORD = 0;
    let mut i: DWORD = 0;
    let mut bSuccess: BOOL = (FALSE) as i32;
    let mut iBufferSize: ::core::ffi::c_int = 0;
    let mut iRequiredBytes: ::core::ffi::c_int = 0;
    let mut b: BYTE = 0;

    'done: {
        while dwPos < dwTextLen {
            match encFile as ::core::ffi::c_int {
                ENCODING_UTF16LE => {
                    pBytes = pszText.add(dwPos as usize) as LPBYTE;
                    dwByteCount = dwTextLen
                        .wrapping_sub(dwPos)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD);
                    dwPos = dwTextLen;
                }
                ENCODING_UTF16BE => {
                    dwByteCount = dwTextLen
                        .wrapping_sub(dwPos)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD);
                    if dwByteCount > ::core::mem::size_of_val(&buffer) as DWORD {
                        dwByteCount = ::core::mem::size_of_val(&buffer) as DWORD;
                    }

                    memcpy(
                        buffer.as_mut_ptr() as *mut ::core::ffi::c_void,
                        pszText.add(dwPos as usize) as *const ::core::ffi::c_void,
                        dwByteCount as usize,
                    );
                    i = 0;
                    while i < dwByteCount {
                        b = buffer[i.wrapping_add(0) as usize];
                        buffer[i.wrapping_add(0) as usize] =
                            buffer[i.wrapping_add(1) as usize];
                        buffer[i.wrapping_add(1) as usize] = b;
                        i = i.wrapping_add(2);
                    }
                    // C: `&buffer[dwPos]` -- a WCHAR index into the BYTE buffer just filled
                    // from offset 0. Past the first 512-WCHAR chunk it wrote stack bytes
                    // beyond `buffer` into the file; the swapped chunk starts at `buffer`.
                    pBytes = buffer.as_mut_ptr() as LPBYTE;
                    dwPos = dwPos
                        .wrapping_add(dwByteCount / (::core::mem::size_of::<WCHAR>() as DWORD));
                }
                ENCODING_ANSI | ENCODING_UTF8 | ENCODING_UTF8BOM => {
                    if encFile as ::core::ffi::c_int == ENCODING_UTF8 as ::core::ffi::c_int
                        || encFile as ::core::ffi::c_int
                            == ENCODING_UTF8BOM as ::core::ffi::c_int
                    {
                        iCodePage = CP_UTF8 as UINT;
                    } else {
                        iCodePage = CP_ACP as UINT;
                    }

                    iRequiredBytes = WideCharToMultiByte(
                        iCodePage,
                        0,
                        pszText.add(dwPos as usize) as LPCWSTR,
                        dwTextLen.wrapping_sub(dwPos) as ::core::ffi::c_int,
                        ::core::ptr::null_mut(),
                        0,
                        ::core::ptr::null(),
                        ::core::ptr::null_mut(),
                    );
                    if iRequiredBytes <= 0 {
                        break 'done;
                    } else if (iRequiredBytes as usize)
                        < ::core::mem::size_of_val(&buffer)
                    {
                        pBytes = buffer.as_mut_ptr() as LPBYTE;
                        iBufferSize = ::core::mem::size_of_val(&buffer) as ::core::ffi::c_int;
                    } else {
                        pAllocBuffer = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            iRequiredBytes as SIZE_T,
                        ) as LPBYTE;
                        if pAllocBuffer.is_null() {
                            return (FALSE) as BOOL;
                        }
                        pBytes = pAllocBuffer;
                        iBufferSize = iRequiredBytes;
                    }

                    dwByteCount = WideCharToMultiByte(
                        iCodePage,
                        0,
                        pszText.add(dwPos as usize) as LPCWSTR,
                        dwTextLen.wrapping_sub(dwPos) as ::core::ffi::c_int,
                        pBytes as LPSTR,
                        iBufferSize,
                        ::core::ptr::null(),
                        ::core::ptr::null_mut(),
                    ) as DWORD;
                    if dwByteCount == 0 {
                        break 'done;
                    }

                    dwPos = dwTextLen;
                }
                _ => {
                    break 'done;
                }
            }

            if WriteFile(
                hFile,
                pBytes as LPCVOID,
                dwByteCount,
                ::core::ptr::addr_of_mut!(dwDummy),
                ::core::ptr::null_mut(),
            ) == 0
            {
                break 'done;
            }

            /* free the buffer, if we have allocated one */
            if !pAllocBuffer.is_null() {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    pAllocBuffer as PVOID,
                );
                pAllocBuffer = core::ptr::null_mut();
            }
        }
        bSuccess = (TRUE) as i32;
    }

    if !pAllocBuffer.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pAllocBuffer as PVOID,
        );
    }
    return bSuccess;
}
#[no_mangle]
pub unsafe extern "C" fn WriteText(
    hFile: HANDLE,
    pszText: LPCWSTR,
    dwTextLen: DWORD,
    encFile: ENCODING,
    iEoln: EOLN,
) -> BOOL {
    let mut wcBom: WCHAR = 0;
    let mut pszLF: LPCWSTR = RUSTCONV_WSTR_075DE2B906DB.as_ptr();
    let mut dwPos: DWORD = 0;
    let mut dwNext: DWORD = 0;

    /* Write the proper byte order marks if not ANSI or UTF-8 without BOM */
    if encFile as ::core::ffi::c_int != ENCODING_ANSI as ::core::ffi::c_int
        && encFile as ::core::ffi::c_int != ENCODING_UTF8 as ::core::ffi::c_int
    {
        wcBom = 0xFEFF as WCHAR;
        if WriteEncodedText(
            hFile,
            core::ptr::addr_of_mut!(wcBom) as LPCWSTR,
            1,
            encFile,
        ) == 0
        {
            return (FALSE) as BOOL;
        }
    }

    dwPos = 0;

    /* pszText eoln are always \r\n */
    loop {
        /* Find the next eoln */
        dwNext = dwPos;
        while dwNext < dwTextLen {
            if *pszText.offset(dwNext as isize) as ::core::ffi::c_int == '\r' as i32
                && *pszText.offset(dwNext.wrapping_add(1) as isize)
                    as ::core::ffi::c_int
                    == '\n' as i32
            {
                break;
            }
            dwNext = dwNext.wrapping_add(1);
        }

        if dwNext != dwTextLen {
            match iEoln as ::core::ffi::c_uint {
                EOLN_LF => {
                    /* Write text (without eoln) */
                    if WriteEncodedText(
                        hFile,
                        pszText.offset(dwPos as isize) as LPCWSTR,
                        dwNext.wrapping_sub(dwPos),
                        encFile,
                    ) == 0
                    {
                        return (FALSE) as BOOL;
                    }
                    /* Write eoln */
                    if WriteEncodedText(hFile, pszLF, 1, encFile) == 0 {
                        return (FALSE) as BOOL;
                    }
                }
                EOLN_CR => {
                    /* Write text (including \r as eoln) */
                    if WriteEncodedText(
                        hFile,
                        pszText.offset(dwPos as isize) as LPCWSTR,
                        dwNext.wrapping_sub(dwPos).wrapping_add(1),
                        encFile,
                    ) == 0
                    {
                        return (FALSE) as BOOL;
                    }
                }
                EOLN_CRLF => {
                    /* Write text (including \r\n as eoln) */
                    if WriteEncodedText(
                        hFile,
                        pszText.offset(dwPos as isize) as LPCWSTR,
                        dwNext.wrapping_sub(dwPos).wrapping_add(2),
                        encFile,
                    ) == 0
                    {
                        return (FALSE) as BOOL;
                    }
                }
                _ => return (FALSE) as i32,
            }
        } else {
            /* Write text (without eoln, since this is the end of the file) */
            if WriteEncodedText(
                hFile,
                pszText.offset(dwPos as isize) as LPCWSTR,
                dwNext.wrapping_sub(dwPos),
                encFile,
            ) == 0
            {
                return (FALSE) as BOOL;
            }
        }

        /* Skip \r\n */
        dwPos = dwNext.wrapping_add(2);
        if !(dwPos < dwTextLen) {
            break;
        }
    }

    return (TRUE) as BOOL;
}
