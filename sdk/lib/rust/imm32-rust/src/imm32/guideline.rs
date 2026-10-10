use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn ImmGetGuideLineAW(
    hIMC: HIMC,
    dwIndex: DWORD,
    lpBuf: LPVOID,
    dwBufLen: DWORD,
    bAnsi: BOOL,
) -> DWORD {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pGuideLine: LPGUIDELINE = core::ptr::null_mut();
    let mut cb: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut pvStr: LPVOID = core::ptr::null_mut();
    let mut pvPrivate: LPVOID = core::ptr::null_mut();
    let mut bUsedDefault: BOOL = 0;
    let mut uCodePage: UINT = 0;

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return 0;
    }

    uCodePage = (*pClientImc).uCodePage;

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        ImmUnlockClientImc(pClientImc);
        return 0;
    }

    pGuideLine = ImmLockIMCC((*pIC).hGuideLine) as LPGUIDELINE;
    if IS_NULL_UNEXPECTEDLY!(pGuideLine) != false {
        ImmUnlockIMC(hIMC);
        ImmUnlockClientImc(pClientImc);
        return 0;
    }

    'quit: {
        if dwIndex == GGL_LEVEL {
            ret = (*pGuideLine).dwLevel;
            break 'quit;
        }

        if dwIndex == GGL_INDEX {
            ret = (*pGuideLine).dwIndex;
            break 'quit;
        }

        if dwIndex == GGL_STRING {
            pvStr = (pGuideLine as LPBYTE).add((*pGuideLine).dwStrOffset as usize) as LPVOID;

            /* get size */
            if bAnsi != 0 {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) != 0 {
                    cb = WideCharToMultiByte(
                        uCodePage,
                        0 as DWORD,
                        pvStr as LPCWSTR,
                        (*pGuideLine).dwStrLen as i32,
                        core::ptr::null_mut(),
                        0 as i32,
                        core::ptr::null(),
                        core::ptr::addr_of_mut!(bUsedDefault),
                    ) as DWORD;
                } else {
                    cb = (*pGuideLine).dwStrLen
                        * (core::mem::size_of::<CHAR>() as DWORD);
                }
            } else {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) != 0 {
                    cb = (*pGuideLine).dwStrLen
                        * (core::mem::size_of::<WCHAR>() as DWORD);
                } else {
                    cb = (MultiByteToWideChar(
                        uCodePage,
                        MB_PRECOMPOSED as DWORD,
                        pvStr as LPCSTR,
                        (*pGuideLine).dwStrLen as i32,
                        core::ptr::null_mut(),
                        0 as i32,
                    ) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
                }
            }

            if dwBufLen == 0 {
                ret = cb;
                break 'quit;
            }
            if cb == 0 || lpBuf.is_null() || dwBufLen < cb {
                ret = 0; /* Error */
                break 'quit;
            }

            /* store to buffer */
            if bAnsi != 0 {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) != 0 {
                    ret = WideCharToMultiByte(
                        uCodePage,
                        0 as DWORD,
                        pvStr as LPCWSTR,
                        (*pGuideLine).dwStrLen as i32,
                        lpBuf as LPSTR,
                        dwBufLen as i32,
                        core::ptr::null(),
                        core::ptr::addr_of_mut!(bUsedDefault),
                    ) as DWORD;
                    break 'quit;
                }
            } else {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) == 0 {
                    ret = (MultiByteToWideChar(
                        uCodePage,
                        MB_PRECOMPOSED as DWORD,
                        pvStr as LPCSTR,
                        (*pGuideLine).dwStrLen as i32,
                        lpBuf as LPWSTR,
                        dwBufLen as i32,
                    ) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
                    break 'quit;
                }
            }

            RtlCopyMemory!(lpBuf, pvStr, cb as usize);
            ret = cb;
            break 'quit;
        }

        if dwIndex == GGL_PRIVATE {
            pvPrivate = (pGuideLine as LPBYTE).add((*pGuideLine).dwPrivateOffset as usize) as LPVOID;

            /* get size */
            if bAnsi != 0 {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) != 0
                    && (*pGuideLine).dwIndex == GL_ID_REVERSECONVERSION
                {
                    cb = CandidateListWideToAnsi(
                        pvPrivate as *const CANDIDATELIST,
                        core::ptr::null_mut(),
                        0 as DWORD,
                        uCodePage,
                    );
                } else {
                    cb = (*pGuideLine).dwPrivateSize;
                }
            } else {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) == 0
                    && (*pGuideLine).dwIndex == GL_ID_REVERSECONVERSION
                {
                    cb = CandidateListAnsiToWide(
                        pvPrivate as *const CANDIDATELIST,
                        core::ptr::null_mut(),
                        0 as DWORD,
                        uCodePage,
                    );
                } else {
                    cb = (*pGuideLine).dwPrivateSize;
                }
            }

            if dwBufLen == 0 {
                ret = cb;
                break 'quit;
            }
            if cb == 0 || lpBuf.is_null() || dwBufLen < cb {
                ret = 0; /* Error */
                break 'quit;
            }

            /* store to buffer */
            if bAnsi != 0 {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) != 0
                    && (*pGuideLine).dwIndex == GL_ID_REVERSECONVERSION
                {
                    ret = CandidateListWideToAnsi(
                        pvPrivate as *const CANDIDATELIST,
                        lpBuf as LPCANDIDATELIST,
                        cb,
                        uCodePage,
                    );
                    break 'quit;
                }
            } else {
                if ((*pClientImc).dwFlags & CLIENTIMC_WIDE) == 0
                    && (*pGuideLine).dwIndex == GL_ID_REVERSECONVERSION
                {
                    ret = CandidateListAnsiToWide(
                        pvPrivate as *const CANDIDATELIST,
                        lpBuf as LPCANDIDATELIST,
                        cb,
                        uCodePage,
                    );
                    break 'quit;
                }
            }

            RtlCopyMemory!(lpBuf, pvPrivate, cb as usize);
            ret = cb;
            break 'quit;
        }
    }

    ImmUnlockIMCC((*pIC).hGuideLine);
    ImmUnlockIMC(hIMC);
    ImmUnlockClientImc(pClientImc);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetGuideLineA(
    hIMC: HIMC,
    dwIndex: DWORD,
    lpBuf: LPSTR,
    dwBufLen: DWORD,
) -> DWORD {
    return ImmGetGuideLineAW(hIMC, dwIndex, lpBuf as LPVOID, dwBufLen, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetGuideLineW(hIMC: HIMC, dwIndex: DWORD, lpBuf: LPWSTR, dwBufLen: DWORD) -> DWORD {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/guideline.c\0"
                as *const u8 as *const i8,
            b"ImmGetGuideLineW\0" as *const u8 as *const i8,
            225i32,
            b"(%p, %lu, %p, %lu)\n\0" as *const u8 as *const i8,
            hIMC,
            dwIndex,
            lpBuf,
            dwBufLen,
        );
    }
    return ImmGetGuideLineAW(hIMC, dwIndex, lpBuf as LPVOID, dwBufLen, (FALSE) as BOOL);
}
