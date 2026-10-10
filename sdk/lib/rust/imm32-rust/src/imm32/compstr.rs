use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(imm) is a debug-channel macro invocation, not a declaration; no Rust item is emitted for it.
// All CS_* macros are function-like and are inlined by the translated functions, so nothing is emitted here.
#[no_mangle]
pub unsafe extern "C" fn Imm32OpenICAndCS(hIMC: HIMC, ppIC: *mut LPINPUTCONTEXT, ppCS: *mut LPCOMPOSITIONSTRING) -> BOOL {
    let mut pIC: LPINPUTCONTEXT;
    let mut pCS: LPCOMPOSITIONSTRING;

    *ppIC = core::ptr::null_mut();
    *ppCS = core::ptr::null_mut();

    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXT;
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    pCS = ImmLockIMCC((*pIC).hCompStr) as LPCOMPOSITIONSTRING;
    if IS_NULL_UNEXPECTEDLY!(pCS) != false {
        ImmUnlockIMC(hIMC);
        return (FALSE) as BOOL;
    }

    *ppIC = pIC;
    *ppCS = pCS;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32CompStrAnsiToWide(
    mut psz: LPCSTR,
    mut cb: DWORD,
    mut lpBuf: LPWSTR,
    mut dwBufLen: DWORD,
    mut uCodePage: UINT,
) -> LONG {
    let mut ret: DWORD = MultiByteToWideChar(
        uCodePage,
        MB_PRECOMPOSED as DWORD,
        psz,
        (cb as usize).wrapping_div(::core::mem::size_of::<CHAR>()) as ::core::ffi::c_int,
        lpBuf,
        (dwBufLen as usize).wrapping_div(::core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
    ) as DWORD;
    if !lpBuf.is_null()
        && (ret.wrapping_add(1 as DWORD) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>())
            <= dwBufLen as usize
    {
        *lpBuf.offset(ret as isize) = UNICODE_NULL as WCHAR;
    }
    return (ret as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()) as LONG;
}

unsafe extern "C" fn Imm32CompStrWideToAnsi(psz: LPCWSTR, cb: DWORD, lpBuf: LPSTR, dwBufLen: DWORD, uCodePage: UINT) -> LONG {
    let mut ret: DWORD = WideCharToMultiByte(
        uCodePage,
        0,
        psz,
        (cb as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        lpBuf,
        (dwBufLen as usize).wrapping_div(core::mem::size_of::<CHAR>()) as ::core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    if !lpBuf.is_null()
        && (ret.wrapping_add(1 as DWORD) as usize)
            .wrapping_mul(core::mem::size_of::<CHAR>())
            <= dwBufLen as usize
    {
        *lpBuf.offset(ret as isize) = ANSI_NULL;
    }
    return (ret as usize).wrapping_mul(core::mem::size_of::<CHAR>()) as LONG;
}

unsafe extern "C" fn Imm32CompAttrWideToAnsi(src: *const BYTE, src_len: INT, text: LPCWSTR, str_len: INT, dst: LPBYTE, dst_len: INT, uCodePage: UINT) -> INT {
    let mut rc: INT;
    let mut i: INT;
    let mut j: INT = 0;
    let mut k: INT = 0;
    let mut len: INT;
    let mut str_len = str_len;
    let mut dst_len = dst_len;

    if src_len == 0 {
        return 0;
    }

    str_len /= core::mem::size_of::<WCHAR>() as INT;
    rc = WideCharToMultiByte(uCodePage, 0, text, str_len, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as INT;

    if dst_len != 0 {
        if dst_len > rc {
            dst_len = rc;
        }

        i = 0;
        'end: {
            while i < str_len {
                len = WideCharToMultiByte(uCodePage, 0, text.offset(i as isize), 1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as INT;
                while len > 0 {
                    let fresh = j;
                    j += 1;
                    *dst.offset(fresh as isize) = *src.offset(k as isize);

                    if dst_len <= j {
                        break 'end;
                    }
                    len -= 1;
                }
                i += 1;
                k += 1;
            }
        }
        rc = j;
    }

    return rc * core::mem::size_of::<BYTE>() as INT;
}

unsafe extern "C" fn Imm32CompAttrAnsiToWide(
    mut src: *const BYTE,
    mut src_len: INT,
    mut text: LPCSTR,
    mut str_len: INT,
    mut dst: LPBYTE,
    mut dst_len: INT,
    mut uCodePage: UINT,
) -> INT {
    let mut rc: INT = 0;
    let mut i: INT = 0;
    let mut j: INT = 0;

    if src_len == 0 {
        return 0;
    }

    str_len /= core::mem::size_of::<CHAR>() as INT;
    rc = MultiByteToWideChar(
        uCodePage,
        MB_PRECOMPOSED,
        text,
        str_len,
        core::ptr::null_mut(),
        0,
    );

    if dst_len != 0 {
        if dst_len > rc {
            dst_len = rc;
        }

        i = 0;
        while i < str_len {
            if !(IsDBCSLeadByteEx(uCodePage, *text.offset(i as isize) as BYTE) != 0
                && *text.offset((i + 1) as isize) != 0)
            {
                let fresh16 = j;
                j = j + 1;
                *dst.offset(fresh16 as isize) = *src.offset(i as isize);

                if dst_len <= j {
                    break;
                }
            }
            i += 1;
        }

        rc = j;
    }

    return rc * (core::mem::size_of::<BYTE>() as INT);
}

unsafe extern "C" fn Imm32CompClauseAnsiToWide(source: *const DWORD, slen: INT, text: LPCSTR, target: LPDWORD, mut tlen: INT, uCodePage: UINT) -> INT {
    let mut rc: INT;
    let mut i: INT;

    if slen == 0 {
        return 0;
    }

    if tlen != 0 {
        if tlen > slen {
            tlen = slen;
        }

        tlen /= core::mem::size_of::<DWORD>() as INT;

        i = 0;
        while i < tlen {
            *target.offset(i as isize) = MultiByteToWideChar(
                uCodePage,
                MB_PRECOMPOSED as DWORD,
                text,
                *source.offset(i as isize) as INT,
                core::ptr::null_mut(),
                0,
            ) as DWORD;
            i += 1;
        }

        rc = (core::mem::size_of::<DWORD>() as INT) * i;
    } else {
        rc = slen;
    }

    return rc;
}

unsafe extern "C" fn Imm32CompClauseWideToAnsi(source: *const DWORD, slen: INT, text: LPCWSTR, target: LPDWORD, mut tlen: INT, uCodePage: UINT) -> INT {
    let mut rc: INT;
    let mut i: INT;

    if slen == 0 {
        return 0;
    }

    if tlen != 0 {
        if tlen > slen {
            tlen = slen;
        }

        tlen /= core::mem::size_of::<DWORD>() as INT;

        i = 0;
        while i < tlen {
            *target.offset(i as isize) = WideCharToMultiByte(
                uCodePage,
                0 as DWORD,
                text,
                *source.offset(i as isize) as INT,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as DWORD;
            i += 1;
        }

        rc = (core::mem::size_of::<DWORD>() as INT) * i;
    } else {
        rc = slen;
    }

    return rc;
}

unsafe extern "C" fn CS_DoPrivate(hIMC: HIMC, pCS: *const COMPOSITIONSTRING, pBuffer: PVOID, dwBufLen: DWORD) -> DWORD {
    if CtfImmIsGuidMapEnable(hIMC) == 0 || (*pCS).dwPrivateSize < core::mem::size_of::<COMPSTR_PRIVATE>() as DWORD {
        return IMM_ERROR_GENERAL as DWORD;
    }

    /* Check boundary #1 (ReactOS only) */
    let mut dwPrivateOffset: DWORD = (*pCS).dwPrivateOffset;
    if dwPrivateOffset >= (*pCS).dwSize || dwPrivateOffset.wrapping_add((*pCS).dwPrivateSize) > (*pCS).dwSize {
        return IMM_ERROR_GENERAL as DWORD;
    }

    let pbCS: *const BYTE = pCS as *const BYTE;
    let pPrivate: *const COMPSTR_PRIVATE = pbCS.offset(dwPrivateOffset as isize) as *const COMPSTR_PRIVATE;
    let mut dwLen: DWORD = (*pPrivate).dwLen;
    let mut dwOffset: DWORD = (*pPrivate).dwOffset;

    /* Check boundary #2 (ReactOS only) */
    if dwPrivateOffset.wrapping_add(dwOffset) > (*pCS).dwPrivateSize
        || dwPrivateOffset.wrapping_add(dwOffset).wrapping_add(dwLen) > (*pCS).dwPrivateSize
    {
        return IMM_ERROR_GENERAL as DWORD;
    }

    if dwBufLen == 0 {
        return dwLen;
    }

    let mut ret: DWORD = min(dwBufLen, dwLen);
    CopyMemory!(
        pBuffer,
        (pPrivate as *const BYTE).offset(dwOffset as isize) as *const ::core::ffi::c_void,
        ret as usize,
    );
    return ret;
}

unsafe extern "C" fn Imm32GetCompStrA(mut hIMC: HIMC, mut pCS: *const COMPOSITIONSTRING, mut dwIndex: DWORD, mut lpBuf: LPVOID, mut dwBufLen: DWORD, mut bAnsiClient: BOOL, mut uCodePage: UINT) -> LONG {
    if bAnsiClient != 0 {
        match dwIndex {
            GCS_COMPREADSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_COMPREADATTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompReadAttrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompReadAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompReadAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwCompReadAttrOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_COMPREADCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompReadClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwCompReadClauseOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_COMPSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_COMPATTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompAttrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwCompAttrOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_COMPCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwCompClauseOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_CURSORPOS => {
                dwBufLen = (*pCS).dwCursorPos;
            }
            GCS_DELTASTART => {
                dwBufLen = (*pCS).dwDeltaStart;
            }
            GCS_RESULTREADSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwResultReadStrOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_RESULTREADCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultReadClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwResultReadClauseOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_RESULTSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwResultStrOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_RESULTCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize) as DWORD;
                    }
                    core::ptr::copy_nonoverlapping(
                        (pCS as LPCSTR).offset((*pCS).dwResultClauseOffset as isize),
                        lpBuf as LPSTR,
                        dwBufLen as usize,
                    );
                }
            }
            GCS_PRIVATE => {
                return CS_DoPrivate(hIMC, pCS, lpBuf, dwBufLen) as LONG;
            }
            _ => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"win32ss/user/imm32/compstr.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32GetCompStrA\0" as *const u8 as *const ::core::ffi::c_char,
                        306 as ::core::ffi::c_int,
                        b"0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
                        dwIndex,
                    );
                }
                return IMM_ERROR_GENERAL as LONG;
            }
        }
    } else {
        match dwIndex {
            GCS_COMPREADSTR => {
                dwBufLen = Imm32CompStrWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize) as LPCWSTR,
                    ((*pCS).dwCompReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
                    lpBuf as LPSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPREADATTR => {
                dwBufLen = Imm32CompAttrWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwCompReadAttrOffset as isize) as *const BYTE,
                    (*pCS).dwCompReadAttrLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize) as LPCWSTR,
                    ((*pCS).dwCompReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as INT,
                    lpBuf as LPBYTE,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPREADCLAUSE => {
                dwBufLen = Imm32CompClauseWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwCompReadClauseOffset as isize) as *const DWORD,
                    (*pCS).dwCompReadClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize) as LPCWSTR,
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPSTR => {
                dwBufLen = Imm32CompStrWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize) as LPCWSTR,
                    ((*pCS).dwCompStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
                    lpBuf as LPSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPATTR => {
                dwBufLen = Imm32CompAttrWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwCompAttrOffset as isize) as *const BYTE,
                    (*pCS).dwCompAttrLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize) as LPCWSTR,
                    ((*pCS).dwCompStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as INT,
                    lpBuf as LPBYTE,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPCLAUSE => {
                dwBufLen = Imm32CompClauseWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwCompClauseOffset as isize) as *const DWORD,
                    (*pCS).dwCompClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize) as LPCWSTR,
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_CURSORPOS => {
                dwBufLen = IchAnsiFromWide(
                    (*pCS).dwCursorPos as LONG,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize) as LPCWSTR,
                    uCodePage,
                ) as DWORD;
            }
            GCS_DELTASTART => {
                dwBufLen = IchAnsiFromWide(
                    (*pCS).dwDeltaStart as LONG,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize) as LPCWSTR,
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTREADSTR => {
                dwBufLen = Imm32CompStrWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwResultReadStrOffset as isize) as LPCWSTR,
                    ((*pCS).dwResultReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
                    lpBuf as LPSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTREADCLAUSE => {
                dwBufLen = Imm32CompClauseWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwResultReadClauseOffset as isize) as *const DWORD,
                    (*pCS).dwResultReadClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwResultReadStrOffset as isize) as LPCWSTR,
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTSTR => {
                dwBufLen = Imm32CompStrWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwResultStrOffset as isize) as LPCWSTR,
                    ((*pCS).dwResultStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
                    lpBuf as LPSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTCLAUSE => {
                dwBufLen = Imm32CompClauseWideToAnsi(
                    (pCS as LPCSTR).offset((*pCS).dwResultClauseOffset as isize) as *const DWORD,
                    (*pCS).dwResultClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwResultStrOffset as isize) as LPCWSTR,
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_PRIVATE => {
                return CS_DoPrivate(hIMC, pCS, lpBuf, dwBufLen) as LONG;
            }
            _ => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"win32ss/user/imm32/compstr.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32GetCompStrA\0" as *const u8 as *const ::core::ffi::c_char,
                        394 as ::core::ffi::c_int,
                        b"0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
                        dwIndex,
                    );
                }
                return IMM_ERROR_GENERAL as LONG;
            }
        }
    }
    return dwBufLen as LONG;
}

unsafe extern "C" fn Imm32GetCompStrW(mut hIMC: HIMC, mut pCS: *const COMPOSITIONSTRING, mut dwIndex: DWORD, mut lpBuf: LPVOID, mut dwBufLen: DWORD, mut bAnsiClient: BOOL, mut uCodePage: UINT) -> LONG {
    if bAnsiClient != 0 {
        match dwIndex {
            GCS_COMPREADSTR => {
                dwBufLen = Imm32CompStrAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize),
                    ((*pCS).dwCompReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD,
                    lpBuf as LPWSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPREADATTR => {
                dwBufLen = Imm32CompAttrAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwCompReadAttrOffset as isize) as *const BYTE,
                    (*pCS).dwCompReadAttrLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize),
                    ((*pCS).dwCompReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as INT,
                    lpBuf as LPBYTE,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPREADCLAUSE => {
                dwBufLen = Imm32CompClauseAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwCompReadClauseOffset as isize) as *const DWORD,
                    (*pCS).dwCompReadClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize),
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPSTR => {
                dwBufLen = Imm32CompStrAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize),
                    ((*pCS).dwCompStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD,
                    lpBuf as LPWSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPATTR => {
                dwBufLen = Imm32CompAttrAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwCompAttrOffset as isize) as *const BYTE,
                    (*pCS).dwCompAttrLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize),
                    ((*pCS).dwCompStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as INT,
                    lpBuf as LPBYTE,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_COMPCLAUSE => {
                dwBufLen = Imm32CompClauseAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwCompClauseOffset as isize) as *const DWORD,
                    (*pCS).dwCompClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize),
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_CURSORPOS => {
                dwBufLen = IchWideFromAnsi(
                    (*pCS).dwCursorPos as LONG,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize),
                    uCodePage,
                ) as DWORD;
            }
            GCS_DELTASTART => {
                dwBufLen = IchWideFromAnsi(
                    (*pCS).dwDeltaStart as LONG,
                    (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize),
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTREADSTR => {
                dwBufLen = Imm32CompStrAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwResultReadStrOffset as isize),
                    ((*pCS).dwResultReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD,
                    lpBuf as LPWSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTREADCLAUSE => {
                dwBufLen = Imm32CompClauseAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwResultReadClauseOffset as isize) as *const DWORD,
                    (*pCS).dwResultReadClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwResultReadStrOffset as isize),
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTSTR => {
                dwBufLen = Imm32CompStrAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwResultStrOffset as isize),
                    ((*pCS).dwResultStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD,
                    lpBuf as LPWSTR,
                    dwBufLen,
                    uCodePage,
                ) as DWORD;
            }
            GCS_RESULTCLAUSE => {
                dwBufLen = Imm32CompClauseAnsiToWide(
                    (pCS as LPCSTR).offset((*pCS).dwResultClauseOffset as isize) as *const DWORD,
                    (*pCS).dwResultClauseLen as INT,
                    (pCS as LPCSTR).offset((*pCS).dwResultStrOffset as isize),
                    lpBuf as LPDWORD,
                    dwBufLen as INT,
                    uCodePage,
                ) as DWORD;
            }
            GCS_PRIVATE => {
                return CS_DoPrivate(hIMC, pCS, lpBuf, dwBufLen) as LONG;
            }
            _ => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"win32ss/user/imm32/compstr.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32GetCompStrW\0" as *const u8 as *const ::core::ffi::c_char,
                        489 as ::core::ffi::c_int,
                        b"0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
                        dwIndex,
                    );
                }
                return IMM_ERROR_GENERAL as LONG;
            }
        }
    } else {
        match dwIndex {
            GCS_COMPREADSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwCompReadStrOffset as isize) as LPCWSTR
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_COMPREADATTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompReadAttrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompReadAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompReadAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwCompReadAttrOffset as isize)
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_COMPREADCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompReadClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwCompReadClauseOffset as isize)
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_COMPSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwCompStrOffset as isize) as LPCWSTR
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_COMPATTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompAttrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompAttrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwCompAttrOffset as isize)
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_COMPCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwCompClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwCompClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwCompClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwCompClauseOffset as isize)
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_CURSORPOS => {
                dwBufLen = (*pCS).dwCursorPos;
            }
            GCS_DELTASTART => {
                dwBufLen = (*pCS).dwDeltaStart;
            }
            GCS_RESULTREADSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultReadStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultReadStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwResultReadStrOffset as isize) as LPCWSTR
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_RESULTREADCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultReadClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultReadClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwResultReadClauseOffset as isize)
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_RESULTSTR => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultStrLen as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultStrLen as usize)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwResultStrOffset as isize) as LPCWSTR
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_RESULTCLAUSE => {
                if dwBufLen == 0 as DWORD {
                    dwBufLen = ((*pCS).dwResultClauseLen as usize)
                        .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                        as DWORD;
                } else {
                    if dwBufLen as usize
                        > ((*pCS).dwResultClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                    {
                        dwBufLen = ((*pCS).dwResultClauseLen as usize)
                            .wrapping_mul(::core::mem::size_of::<CHAR>() as usize)
                            as DWORD;
                    }
                    memcpy(
                        lpBuf as *mut ::core::ffi::c_void,
                        (pCS as LPCSTR).offset((*pCS).dwResultClauseOffset as isize)
                            as *const ::core::ffi::c_void,
                        dwBufLen as size_t,
                    );
                }
            }
            GCS_PRIVATE => {
                return CS_DoPrivate(hIMC, pCS, lpBuf, dwBufLen) as LONG;
            }
            _ => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"win32ss/user/imm32/compstr.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32GetCompStrW\0" as *const u8 as *const ::core::ffi::c_char,
                        549 as ::core::ffi::c_int,
                        b"0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
                        dwIndex,
                    );
                }
                return IMM_ERROR_GENERAL as LONG;
            }
        }
    }
    return dwBufLen as LONG;
}

unsafe extern "C" fn ImmSetCompositionStringAW(hIMC: HIMC, dwIndex: DWORD, pComp: LPVOID, dwCompLen: DWORD, pRead: LPVOID, dwReadLen: DWORD, bAnsiAPI: BOOL) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut bAnsiClient: BOOL = 0;
    let mut pCompNew: LPVOID = core::ptr::null_mut();
    let mut pReadNew: LPVOID = core::ptr::null_mut();
    let mut dwThreadId: DWORD = 0;
    let mut cbCompNew: DWORD = 0;
    let mut cbReadNew: DWORD = 0;
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pCS: LPCOMPOSITIONSTRING = core::ptr::null_mut();
    let mut hKL: HKL = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut uCodePage: UINT = 0;
    let mut pRS: LPRECONVERTSTRING = core::ptr::null_mut();

    dwThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD;
    if dwThreadId != GetCurrentThreadId() {
        return (FALSE) as BOOL;
    }

    hKL = GetKeyboardLayout(dwThreadId);
    pImeDpi = ImmLockImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }

    uCodePage = (*pImeDpi).uCodePage;
    bAnsiClient = (ImeDpi_IsUnicode!(pImeDpi) == false) as BOOL;

    match dwIndex {
        SCS_SETSTR | SCS_CHANGEATTR | SCS_CHANGECLAUSE => {}
        SCS_SETRECONVERTSTRING | SCS_QUERYRECONVERTSTRING => {
            if ((*pImeDpi).ImeInfo.fdwSCSCaps & SCS_CAP_SETRECONVERTSTRING) != 0 {
            } else {
                if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR as i32) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"win32ss/user/imm32/compstr.c\0"
                            .as_ptr() as *const i8,
                        b"ImmSetCompositionStringAW\0".as_ptr() as *const i8,
                        602,
                        b"0x%X\n\0".as_ptr() as *const i8,
                        dwIndex,
                    );
                }
                ImmUnlockImeDpi(pImeDpi);
                return (FALSE) as BOOL;
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR as i32) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"win32ss/user/imm32/compstr.c\0"
                        .as_ptr() as *const i8,
                    b"ImmSetCompositionStringAW\0".as_ptr() as *const i8,
                    602,
                    b"0x%X\n\0".as_ptr() as *const i8,
                    dwIndex,
                );
            }
            ImmUnlockImeDpi(pImeDpi);
            return (FALSE) as BOOL;
        }
    }

    if bAnsiAPI == bAnsiClient || (pComp.is_null() && pRead.is_null()) {
        ret = (*pImeDpi)
            .ImeSetCompositionString
            .unwrap_unchecked()(
            hIMC,
            dwIndex,
            pComp as LPCVOID,
            dwCompLen,
            pRead as LPCVOID,
            dwReadLen,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    if Imm32OpenICAndCS(hIMC, &raw mut pIC, &raw mut pCS) == 0 {
        ImmUnlockImeDpi(pImeDpi);
        return (FALSE) as BOOL;
    }

    'quit: {
        match dwIndex {
            SCS_SETSTR => {
                if !pComp.is_null() {
                    if bAnsiClient != 0 {
                        cbCompNew = Imm32CompStrWideToAnsi(
                            pComp as LPCWSTR,
                            dwCompLen,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        Imm32CompStrWideToAnsi(
                            pComp as LPCWSTR,
                            dwCompLen,
                            pCompNew as LPSTR,
                            cbCompNew,
                            uCodePage,
                        );
                    } else {
                        cbCompNew = Imm32CompStrAnsiToWide(
                            pComp as LPCSTR,
                            dwCompLen,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        Imm32CompStrAnsiToWide(
                            pComp as LPCSTR,
                            dwCompLen,
                            pCompNew as LPWSTR,
                            cbCompNew,
                            uCodePage,
                        );
                    }
                }

                if !pRead.is_null() {
                    if bAnsiClient != 0 {
                        cbReadNew = Imm32CompStrWideToAnsi(
                            pRead as LPCWSTR,
                            dwReadLen,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        Imm32CompStrWideToAnsi(
                            pRead as LPCWSTR,
                            dwReadLen,
                            pReadNew as LPSTR,
                            cbReadNew,
                            uCodePage,
                        );
                    } else {
                        cbReadNew = Imm32CompStrAnsiToWide(
                            pRead as LPCSTR,
                            dwReadLen,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        Imm32CompStrAnsiToWide(
                            pRead as LPCSTR,
                            dwReadLen,
                            pReadNew as LPWSTR,
                            cbReadNew,
                            uCodePage,
                        );
                    }
                }
            }

            SCS_CHANGEATTR => {
                if !pComp.is_null() {
                    if bAnsiClient != 0 {
                        cbCompNew = Imm32CompAttrWideToAnsi(
                            pComp as *const BYTE,
                            dwCompLen as INT,
                            CS_StrW!(pCS, CompStr),
                            (CS_SizeW!(pCS, CompStr)) as INT,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        Imm32CompAttrWideToAnsi(
                            pComp as *const BYTE,
                            dwCompLen as INT,
                            CS_StrW!(pCS, CompStr),
                            (CS_SizeW!(pCS, CompStr)) as INT,
                            pCompNew as LPBYTE,
                            cbCompNew as INT,
                            uCodePage,
                        );
                    } else {
                        cbCompNew = Imm32CompAttrAnsiToWide(
                            pComp as *const BYTE,
                            dwCompLen as INT,
                            CS_StrA!(pCS, CompStr),
                            (CS_SizeA!(pCS, CompStr)) as INT,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        Imm32CompAttrAnsiToWide(
                            pComp as *const BYTE,
                            dwCompLen as INT,
                            CS_StrA!(pCS, CompStr),
                            (CS_SizeA!(pCS, CompStr)) as INT,
                            pCompNew as LPBYTE,
                            cbCompNew as INT,
                            uCodePage,
                        );
                    }
                }

                if !pRead.is_null() {
                    if bAnsiClient != 0 {
                        cbReadNew = Imm32CompAttrWideToAnsi(
                            pRead as *const BYTE,
                            dwReadLen as INT,
                            CS_StrW!(pCS, CompReadStr),
                            (CS_SizeW!(pCS, CompReadStr)) as INT,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        Imm32CompAttrWideToAnsi(
                            pRead as *const BYTE,
                            dwReadLen as INT,
                            CS_StrW!(pCS, CompReadStr),
                            (CS_SizeW!(pCS, CompReadStr)) as INT,
                            pReadNew as LPBYTE,
                            cbReadNew as INT,
                            uCodePage,
                        );
                    } else {
                        cbReadNew = Imm32CompAttrAnsiToWide(
                            pRead as *const BYTE,
                            dwReadLen as INT,
                            CS_StrA!(pCS, CompReadStr),
                            (CS_SizeA!(pCS, CompReadStr)) as INT,
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        Imm32CompAttrAnsiToWide(
                            pRead as *const BYTE,
                            dwReadLen as INT,
                            CS_StrA!(pCS, CompReadStr),
                            (CS_SizeA!(pCS, CompReadStr)) as INT,
                            pReadNew as LPBYTE,
                            cbReadNew as INT,
                            uCodePage,
                        );
                    }
                }
            }

            SCS_CHANGECLAUSE => {
                if !pComp.is_null() {
                    if bAnsiClient != 0 {
                        cbCompNew = Imm32CompClauseWideToAnsi(
                            pComp as *const DWORD,
                            dwCompLen as INT,
                            CS_StrW!(pCS, CompStr),
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        Imm32CompClauseWideToAnsi(
                            pComp as *const DWORD,
                            dwCompLen as INT,
                            CS_StrW!(pCS, CompStr),
                            pCompNew as LPDWORD,
                            cbCompNew as INT,
                            uCodePage,
                        );
                    } else {
                        cbCompNew = Imm32CompClauseAnsiToWide(
                            pComp as *const DWORD,
                            dwCompLen as INT,
                            CS_StrA!(pCS, CompStr),
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        Imm32CompClauseAnsiToWide(
                            pComp as *const DWORD,
                            dwCompLen as INT,
                            CS_StrA!(pCS, CompStr),
                            pCompNew as LPDWORD,
                            cbCompNew as INT,
                            uCodePage,
                        );
                    }
                }

                if !pRead.is_null() {
                    if bAnsiClient != 0 {
                        cbReadNew = Imm32CompClauseWideToAnsi(
                            pRead as *const DWORD,
                            dwReadLen as INT,
                            CS_StrW!(pCS, CompReadStr),
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        Imm32CompClauseWideToAnsi(
                            pRead as *const DWORD,
                            dwReadLen as INT,
                            CS_StrW!(pCS, CompReadStr),
                            pReadNew as LPDWORD,
                            cbReadNew as INT,
                            uCodePage,
                        );
                    } else {
                        cbReadNew = Imm32CompClauseAnsiToWide(
                            pRead as *const DWORD,
                            dwReadLen as INT,
                            CS_StrA!(pCS, CompReadStr),
                            core::ptr::null_mut(),
                            0,
                            uCodePage,
                        ) as DWORD;
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        Imm32CompClauseAnsiToWide(
                            pRead as *const DWORD,
                            dwReadLen as INT,
                            CS_StrA!(pCS, CompReadStr),
                            pReadNew as LPDWORD,
                            cbReadNew as INT,
                            uCodePage,
                        );
                    }
                }
            }

            SCS_SETRECONVERTSTRING | SCS_QUERYRECONVERTSTRING => {
                if !pComp.is_null() {
                    if bAnsiClient != 0 {
                        cbCompNew = Imm32ReconvertAnsiFromWide(
                            core::ptr::null_mut(),
                            pComp as *const RECONVERTSTRING,
                            uCodePage,
                        );
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        pRS = pCompNew as LPRECONVERTSTRING;
                        (*pRS).dwSize = cbCompNew;
                        (*pRS).dwVersion = 0;
                        Imm32ReconvertAnsiFromWide(
                            pRS,
                            pComp as *const RECONVERTSTRING,
                            uCodePage,
                        );
                    } else {
                        cbCompNew = Imm32ReconvertWideFromAnsi(
                            core::ptr::null_mut(),
                            pComp as *const RECONVERTSTRING,
                            uCodePage,
                        );
                        pCompNew = ImmLocalAlloc(0, cbCompNew);
                        if IS_NULL_UNEXPECTEDLY!(pCompNew) != false {
                            break 'quit;
                        }
                        pRS = pCompNew as LPRECONVERTSTRING;
                        (*pRS).dwSize = cbCompNew;
                        (*pRS).dwVersion = 0;
                        Imm32ReconvertWideFromAnsi(
                            pRS,
                            pComp as *const RECONVERTSTRING,
                            uCodePage,
                        );
                    }
                }

                if !pRead.is_null() {
                    if bAnsiClient != 0 {
                        cbReadNew = Imm32ReconvertAnsiFromWide(
                            core::ptr::null_mut(),
                            pRead as *const RECONVERTSTRING,
                            uCodePage,
                        );
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        pRS = pReadNew as LPRECONVERTSTRING;
                        (*pRS).dwSize = cbReadNew;
                        (*pRS).dwVersion = 0;
                        Imm32ReconvertAnsiFromWide(
                            pRS,
                            pRead as *const RECONVERTSTRING,
                            uCodePage,
                        );
                    } else {
                        cbReadNew = Imm32ReconvertWideFromAnsi(
                            core::ptr::null_mut(),
                            pRead as *const RECONVERTSTRING,
                            uCodePage,
                        );
                        pReadNew = ImmLocalAlloc(0, cbReadNew);
                        if IS_NULL_UNEXPECTEDLY!(pReadNew) != false {
                            break 'quit;
                        }
                        pRS = pReadNew as LPRECONVERTSTRING;
                        (*pRS).dwSize = cbReadNew;
                        (*pRS).dwVersion = 0;
                        Imm32ReconvertWideFromAnsi(
                            pRS,
                            pRead as *const RECONVERTSTRING,
                            uCodePage,
                        );
                    }
                }
            }

            _ => {}
        }

        ImmUnlockIMCC((*pIC).hCompStr);
        pCS = core::ptr::null_mut();
        ImmUnlockIMC(hIMC);
        pIC = core::ptr::null_mut();

        ret = (*pImeDpi)
            .ImeSetCompositionString
            .unwrap_unchecked()(
            hIMC,
            dwIndex,
            pCompNew as LPCVOID,
            cbCompNew,
            pReadNew as LPCVOID,
            cbReadNew,
        );

        if dwIndex == SCS_QUERYRECONVERTSTRING {
            if !pComp.is_null() {
                if bAnsiClient != 0 {
                    ret = Imm32ReconvertWideFromAnsi(
                        pComp as LPRECONVERTSTRING,
                        pCompNew as *const RECONVERTSTRING,
                        uCodePage,
                    ) as BOOL;
                } else {
                    ret = Imm32ReconvertAnsiFromWide(
                        pComp as LPRECONVERTSTRING,
                        pCompNew as *const RECONVERTSTRING,
                        uCodePage,
                    ) as BOOL;
                }
            }

            if !pRead.is_null() {
                if bAnsiClient != 0 {
                    ret = Imm32ReconvertWideFromAnsi(
                        pRead as LPRECONVERTSTRING,
                        pReadNew as *const RECONVERTSTRING,
                        uCodePage,
                    ) as BOOL;
                } else {
                    ret = Imm32ReconvertAnsiFromWide(
                        pRead as LPRECONVERTSTRING,
                        pReadNew as *const RECONVERTSTRING,
                        uCodePage,
                    ) as BOOL;
                }
            }
        }
    }

    // Quit:
    if !pCS.is_null() {
        ImmUnlockIMCC((*pIC).hCompStr);
    }
    if !pIC.is_null() {
        ImmUnlockIMC(hIMC);
    }
    ImmLocalFree!(pCompNew);
    ImmLocalFree!(pReadNew);
    ImmUnlockImeDpi(pImeDpi);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCompositionStringA(hIMC: HIMC, dwIndex: DWORD, lpBuf: LPVOID, dwBufLen: DWORD) -> LONG {
    let mut ret: LONG = 0;
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pCS: LPCOMPOSITIONSTRING = core::ptr::null_mut();
    let mut bAnsiClient: BOOL = 0;
    let mut uCodePage: UINT = 0;

    if dwBufLen != 0 && IS_NULL_UNEXPECTEDLY!(lpBuf) != false {
        return 0;
    }

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return 0;
    }

    bAnsiClient = ((*pClientImc).dwFlags & CLIENTIMC_WIDE == 0) as i32 as BOOL;
    uCodePage = (*pClientImc).uCodePage;
    ImmUnlockClientImc(pClientImc);

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return 0;
    }

    pCS = ImmLockIMCC((*pIC).hCompStr) as LPCOMPOSITIONSTRING;
    if IS_NULL_UNEXPECTEDLY!(pCS) != false {
        ImmUnlockIMC(hIMC);
        return 0;
    }

    ret = Imm32GetCompStrA(hIMC, pCS, dwIndex, lpBuf, dwBufLen, bAnsiClient, uCodePage);
    ImmUnlockIMCC((*pIC).hCompStr);
    ImmUnlockIMC(hIMC);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCompositionStringW(hIMC: HIMC, dwIndex: DWORD, lpBuf: LPVOID, dwBufLen: DWORD) -> LONG {
    let mut ret: LONG = 0;
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pCS: LPCOMPOSITIONSTRING = core::ptr::null_mut();
    let mut bAnsiClient: BOOL = 0;
    let mut uCodePage: UINT = 0;

    if dwBufLen != 0 && IS_NULL_UNEXPECTEDLY!(lpBuf) != false {
        return 0;
    }

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return 0;
    }

    bAnsiClient = (((*pClientImc).dwFlags & CLIENTIMC_WIDE) == 0) as BOOL;
    uCodePage = (*pClientImc).uCodePage;
    ImmUnlockClientImc(pClientImc);

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return 0;
    }

    pCS = ImmLockIMCC((*pIC).hCompStr) as LPCOMPOSITIONSTRING;
    if IS_NULL_UNEXPECTEDLY!(pCS) != false {
        ImmUnlockIMC(hIMC);
        return 0;
    }

    ret = Imm32GetCompStrW(
        hIMC,
        pCS as *const COMPOSITIONSTRING,
        dwIndex,
        lpBuf,
        dwBufLen,
        bAnsiClient,
        uCodePage,
    );
    ImmUnlockIMCC((*pIC).hCompStr);
    ImmUnlockIMC(hIMC);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetCompositionStringA(
    hIMC: HIMC,
    dwIndex: DWORD,
    lpComp: LPVOID,
    dwCompLen: DWORD,
    lpRead: LPVOID,
    dwReadLen: DWORD,
) -> BOOL {
    return ImmSetCompositionStringAW(hIMC, dwIndex, lpComp, dwCompLen, lpRead, dwReadLen, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetCompositionStringW(
    hIMC: HIMC,
    dwIndex: DWORD,
    lpComp: LPVOID,
    dwCompLen: DWORD,
    lpRead: LPVOID,
    dwReadLen: DWORD,
) -> BOOL {
    return ImmSetCompositionStringAW(hIMC, dwIndex, lpComp, dwCompLen, lpRead, dwReadLen, (FALSE) as BOOL);
}
