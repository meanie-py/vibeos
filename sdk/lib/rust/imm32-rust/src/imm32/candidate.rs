use crate::imm32::utils::Imm32MakeIMENotify;
use crate::imm32::imm::ImmLockClientImc;
use crate::imm32::imm::ImmLockIMC;
use crate::imm32::utils::ImmLockIMCC;
use crate::imm32::imm::ImmUnlockClientImc;
use crate::imm32::imm::ImmUnlockIMC;
use crate::imm32::utils::ImmUnlockIMCC;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The preamble contains one object-like constant-free macro (`ROUNDUP4` is function-like, so it is skipped) and the `WINE_DEFAULT_DEBUG_CHANNEL` macro invocation, which declares a debug-channel symbol that is only used for logging. Neither produces a Rust declaration that the functions depend on, so nothing is emitted here.
#[no_mangle]
pub unsafe extern "C" fn CandidateListWideToAnsi(
    pWideCL: *const CANDIDATELIST,
    pAnsiCL: LPCANDIDATELIST,
    dwBufLen: DWORD,
    uCodePage: UINT,
) -> DWORD {
    let mut bUsedDefault: BOOL = 0;
    let mut dwSize: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut cbGot: DWORD = 0;
    let mut cbLeft: DWORD = 0;
    let mut pbWide: *const BYTE = core::ptr::null();
    let mut pbAnsi: LPBYTE = core::ptr::null_mut();
    let mut pibOffsets: LPDWORD = core::ptr::null_mut();

    /* calculate total ansi size */
    if (*pWideCL).dwCount > 0 {
        dwSize = (core::mem::size_of::<CANDIDATELIST>() as DWORD).wrapping_add(
            (*pWideCL)
                .dwCount
                .wrapping_sub(1)
                .wrapping_mul(core::mem::size_of::<DWORD>() as DWORD),
        );
        dwIndex = 0;
        while dwIndex < (*pWideCL).dwCount {
            pbWide = (pWideCL as *const BYTE)
                .add(*(core::ptr::addr_of!((*pWideCL).dwOffset) as *const DWORD).add(dwIndex as usize) as usize);
            cbGot = WideCharToMultiByte(
                uCodePage,
                0,
                pbWide as LPCWSTR,
                -1,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(bUsedDefault),
            ) as DWORD;
            dwSize = dwSize.wrapping_add(cbGot);
            dwIndex = dwIndex.wrapping_add(1);
        }
    } else {
        dwSize = core::mem::size_of::<CANDIDATELIST>() as DWORD;
    }

    dwSize = ROUNDUP4!(dwSize);
    if dwBufLen == 0 {
        return dwSize;
    }
    if dwBufLen < dwSize {
        return 0;
    }

    /* store to ansi */
    (*pAnsiCL).dwSize = dwBufLen;
    (*pAnsiCL).dwStyle = (*pWideCL).dwStyle;
    (*pAnsiCL).dwCount = (*pWideCL).dwCount;
    (*pAnsiCL).dwSelection = (*pWideCL).dwSelection;
    (*pAnsiCL).dwPageStart = (*pWideCL).dwPageStart;
    (*pAnsiCL).dwPageSize = (*pWideCL).dwPageSize;

    pibOffsets = core::ptr::addr_of_mut!((*pAnsiCL).dwOffset) as LPDWORD;
    if (*pWideCL).dwCount > 0 {
        *pibOffsets.add(0 as usize) =
            (core::mem::size_of::<CANDIDATELIST>() as DWORD).wrapping_add(
                (*pWideCL)
                    .dwCount
                    .wrapping_sub(1)
                    .wrapping_mul(core::mem::size_of::<DWORD>() as DWORD),
            );
        cbLeft = dwBufLen.wrapping_sub(*pibOffsets.add(0 as usize));

        dwIndex = 0;
        while dwIndex < (*pWideCL).dwCount {
            pbWide = (pWideCL as *const BYTE)
                .add(*(core::ptr::addr_of!((*pWideCL).dwOffset) as *const DWORD).add(dwIndex as usize) as usize);
            pbAnsi = (pAnsiCL as LPBYTE).add(*pibOffsets.add(dwIndex as usize) as usize);

            /* convert to ansi */
            cbGot = WideCharToMultiByte(
                uCodePage,
                0,
                pbWide as LPCWSTR,
                -1,
                pbAnsi as LPSTR,
                cbLeft as core::ffi::c_int,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(bUsedDefault),
            ) as DWORD;
            cbLeft = cbLeft.wrapping_sub(cbGot);

            if dwIndex < (*pWideCL).dwCount.wrapping_sub(1) {
                *pibOffsets.add(dwIndex.wrapping_add(1) as usize) =
                    (*pibOffsets.add(dwIndex as usize)).wrapping_add(cbGot);
            }
            dwIndex = dwIndex.wrapping_add(1);
        }
    } else {
        *pibOffsets.add(0 as usize) =
            core::mem::size_of::<CANDIDATELIST>() as DWORD;
    }

    return dwBufLen;
}
#[no_mangle]
pub unsafe extern "C" fn CandidateListAnsiToWide(
    pAnsiCL: *const CANDIDATELIST,
    pWideCL: LPCANDIDATELIST,
    dwBufLen: DWORD,
    uCodePage: UINT,
) -> DWORD {
    let mut dwSize: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut cchGot: DWORD = 0;
    let mut cbGot: DWORD = 0;
    let mut cbLeft: DWORD = 0;
    let mut pbAnsi: *const BYTE = core::ptr::null();
    let mut pbWide: LPBYTE = core::ptr::null_mut();
    let mut pibOffsets: LPDWORD = core::ptr::null_mut();

    /* calculate total wide size */
    if (*pAnsiCL).dwCount > 0 as DWORD {
        dwSize = (core::mem::size_of::<CANDIDATELIST>() as DWORD).wrapping_add(
            ((*pAnsiCL).dwCount.wrapping_sub(1 as DWORD))
                .wrapping_mul(core::mem::size_of::<DWORD>() as DWORD),
        );
        dwIndex = 0 as DWORD;
        while dwIndex < (*pAnsiCL).dwCount {
            pbAnsi = (pAnsiCL as *const BYTE).offset(
                *(core::ptr::addr_of!((*pAnsiCL).dwOffset) as *const DWORD).offset(dwIndex as isize)
                    as isize,
            );
            cchGot = MultiByteToWideChar(
                uCodePage,
                MB_PRECOMPOSED as DWORD,
                pbAnsi as LPCSTR,
                -(1 as core::ffi::c_int),
                core::ptr::null_mut(),
                0 as core::ffi::c_int,
            ) as DWORD;
            dwSize = dwSize.wrapping_add(cchGot.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));
            dwIndex = dwIndex.wrapping_add(1);
        }
    } else {
        dwSize = core::mem::size_of::<CANDIDATELIST>() as DWORD;
    }

    dwSize = ROUNDUP4!(dwSize);
    if dwBufLen == 0 as DWORD {
        return dwSize;
    }
    if dwBufLen < dwSize {
        return 0 as DWORD;
    }

    /* store to wide */
    (*pWideCL).dwSize = dwBufLen;
    (*pWideCL).dwStyle = (*pAnsiCL).dwStyle;
    (*pWideCL).dwCount = (*pAnsiCL).dwCount;
    (*pWideCL).dwSelection = (*pAnsiCL).dwSelection;
    (*pWideCL).dwPageStart = (*pAnsiCL).dwPageStart;
    (*pWideCL).dwPageSize = (*pAnsiCL).dwPageSize;

    pibOffsets = core::ptr::addr_of_mut!((*pWideCL).dwOffset) as *mut DWORD as LPDWORD;
    if (*pAnsiCL).dwCount > 0 as DWORD {
        *pibOffsets.offset(0 as isize) =
            (core::mem::size_of::<CANDIDATELIST>() as DWORD).wrapping_add(
                ((*pWideCL).dwCount.wrapping_sub(1 as DWORD))
                    .wrapping_mul(core::mem::size_of::<DWORD>() as DWORD),
            );
        cbLeft = dwBufLen.wrapping_sub(*pibOffsets.offset(0 as isize));

        dwIndex = 0 as DWORD;
        while dwIndex < (*pAnsiCL).dwCount {
            pbAnsi = (pAnsiCL as *const BYTE).offset(
                *(core::ptr::addr_of!((*pAnsiCL).dwOffset) as *const DWORD).offset(dwIndex as isize)
                    as isize,
            );
            pbWide = (pWideCL as LPBYTE).offset(*pibOffsets.offset(dwIndex as isize) as isize);

            /* convert to wide */
            cchGot = MultiByteToWideChar(
                uCodePage,
                MB_PRECOMPOSED as DWORD,
                pbAnsi as LPCSTR,
                -(1 as core::ffi::c_int),
                pbWide as LPWSTR,
                (cbLeft / (core::mem::size_of::<WCHAR>() as DWORD)) as core::ffi::c_int,
            ) as DWORD;
            cbGot = cchGot.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
            cbLeft = cbLeft.wrapping_sub(cbGot);

            if dwIndex.wrapping_add(1 as DWORD) < (*pAnsiCL).dwCount {
                *pibOffsets.offset(dwIndex.wrapping_add(1 as DWORD) as isize) =
                    (*pibOffsets.offset(dwIndex as isize)).wrapping_add(cbGot);
            }
            dwIndex = dwIndex.wrapping_add(1);
        }
    } else {
        *pibOffsets.offset(0 as isize) = core::mem::size_of::<CANDIDATELIST>() as DWORD;
    }

    return dwBufLen;
}

unsafe extern "C" fn ImmGetCandidateListAW(
    hIMC: HIMC,
    dwIndex: DWORD,
    lpCandList: LPCANDIDATELIST,
    dwBufLen: DWORD,
    bAnsi: BOOL,
) -> DWORD {
    let mut dwSize: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut uCodePage: UINT = 0;
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pCI: LPCANDIDATEINFO = core::ptr::null_mut();
    let mut pCL: LPCANDIDATELIST = core::ptr::null_mut();

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) {
        return 0;
    }

    uCodePage = (*pClientImc).uCodePage;
    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) {
        ImmUnlockClientImc(pClientImc);
        return 0;
    }

    pCI = ImmLockIMCC((*pIC).hCandInfo) as LPCANDIDATEINFO;
    if IS_NULL_UNEXPECTEDLY!(pCI) {
        ImmUnlockIMC(hIMC);
        ImmUnlockClientImc(pClientImc);
        return 0;
    }

    'quit: {
        if ((*pCI).dwSize as usize) < core::mem::size_of::<CANDIDATEINFO>() as usize {
            break 'quit;
        }

        if (*pCI).dwCount <= dwIndex {
            break 'quit;
        }

        /* get required size */
        pCL = (pCI as LPBYTE).offset((*pCI).dwOffset[dwIndex as usize] as isize) as LPCANDIDATELIST;
        if bAnsi != 0 {
            if (*pClientImc).dwFlags & CLIENTIMC_WIDE != 0 {
                dwSize = CandidateListAnsiToWide(
                    pCL as *const CANDIDATELIST,
                    core::ptr::null_mut(),
                    0,
                    uCodePage,
                );
            } else {
                dwSize = (*pCL).dwSize;
            }
        } else {
            if (*pClientImc).dwFlags & CLIENTIMC_WIDE != 0 {
                dwSize = (*pCL).dwSize;
            } else {
                dwSize = CandidateListWideToAnsi(
                    pCL as *const CANDIDATELIST,
                    core::ptr::null_mut(),
                    0,
                    uCodePage,
                );
            }
        }

        if dwBufLen != 0 && dwSize != 0 {
            if lpCandList.is_null() || dwBufLen < dwSize {
                break 'quit;
            }

            /* store */
            if bAnsi != 0 {
                if (*pClientImc).dwFlags & CLIENTIMC_WIDE != 0 {
                    CandidateListAnsiToWide(
                        pCL as *const CANDIDATELIST,
                        lpCandList,
                        dwSize,
                        uCodePage,
                    );
                } else {
                    RtlCopyMemory!(
                        lpCandList as *mut ::core::ffi::c_void,
                        pCL as *const ::core::ffi::c_void,
                        dwSize as usize,
                    );
                }
            } else {
                if (*pClientImc).dwFlags & CLIENTIMC_WIDE != 0 {
                    RtlCopyMemory!(
                        lpCandList as *mut ::core::ffi::c_void,
                        pCL as *const ::core::ffi::c_void,
                        dwSize as usize,
                    );
                } else {
                    CandidateListWideToAnsi(
                        pCL as *const CANDIDATELIST,
                        lpCandList,
                        dwSize,
                        uCodePage,
                    );
                }
            }
        }

        ret = dwSize;
    }

    ImmUnlockIMCC((*pIC).hCandInfo);
    ImmUnlockIMC(hIMC);
    ImmUnlockClientImc(pClientImc);
    return ret;
}

unsafe extern "C" fn ImmGetCandidateListCountAW(
    hIMC: HIMC,
    lpdwListCount: LPDWORD,
    bAnsi: BOOL,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut cbGot: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut uCodePage: UINT = 0;
    let mut pCI: *const CANDIDATEINFO = core::ptr::null();
    let mut pb: *const BYTE = core::ptr::null();
    let mut pCL: *const CANDIDATELIST = core::ptr::null();
    let mut pdwOffsets: *const DWORD = core::ptr::null();

    if IS_NULL_UNEXPECTEDLY!(lpdwListCount) != false {
        return 0;
    }

    *lpdwListCount = 0;

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return 0;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        ImmUnlockClientImc(pClientImc);
        return 0;
    }

    uCodePage = (*pClientImc).uCodePage;

    pCI = ImmLockIMCC((*pIC).hCandInfo) as *const CANDIDATEINFO;
    if IS_NULL_UNEXPECTEDLY!(pCI) != false {
        ImmUnlockIMC(hIMC);
        ImmUnlockClientImc(pClientImc);
        return 0;
    }

    'quit: {
        if ((*pCI).dwSize as usize) < core::mem::size_of::<CANDIDATEINFO>() {
            break 'quit;
        }

        *lpdwListCount = (*pCI).dwCount;

        /* calculate total size of candidate lists */
        if bAnsi != 0 {
            if (*pClientImc).dwFlags & CLIENTIMC_WIDE != 0 {
                ret = ROUNDUP4!((*pCI).dwPrivateSize);
                pdwOffsets = core::ptr::addr_of!((*pCI).dwOffset) as *const DWORD;
                dwIndex = 0;
                while dwIndex < (*pCI).dwCount {
                    pb = (pCI as *const BYTE).add(*pdwOffsets.add(dwIndex as usize) as usize);
                    pCL = pb as *const CANDIDATELIST;
                    cbGot = CandidateListWideToAnsi(
                        pCL,
                        core::ptr::null_mut(),
                        0,
                        uCodePage,
                    );
                    ret = ret.wrapping_add(cbGot);
                    dwIndex = dwIndex.wrapping_add(1);
                }
            } else {
                ret = (*pCI).dwSize;
            }
        } else {
            if (*pClientImc).dwFlags & CLIENTIMC_WIDE != 0 {
                ret = (*pCI).dwSize;
            } else {
                ret = ROUNDUP4!((*pCI).dwPrivateSize);
                pdwOffsets = core::ptr::addr_of!((*pCI).dwOffset) as *const DWORD;
                dwIndex = 0;
                while dwIndex < (*pCI).dwCount {
                    pb = (pCI as *const BYTE).add(*pdwOffsets.add(dwIndex as usize) as usize);
                    pCL = pb as *const CANDIDATELIST;
                    cbGot = CandidateListAnsiToWide(
                        pCL,
                        core::ptr::null_mut(),
                        0,
                        uCodePage,
                    );
                    ret = ret.wrapping_add(cbGot);
                    dwIndex = dwIndex.wrapping_add(1);
                }
            }
        }
    }

    // Quit:
    ImmUnlockIMCC((*pIC).hCandInfo);
    ImmUnlockIMC(hIMC);
    ImmUnlockClientImc(pClientImc);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCandidateListA(
    mut hIMC: HIMC,
    mut dwIndex: DWORD,
    mut lpCandList: LPCANDIDATELIST,
    mut dwBufLen: DWORD,
) -> DWORD {
    return ImmGetCandidateListAW(hIMC, dwIndex, lpCandList, dwBufLen, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCandidateListCountA(
    hIMC: HIMC,
    lpdwListCount: LPDWORD,
) -> DWORD {
    return ImmGetCandidateListCountAW(hIMC, lpdwListCount, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCandidateListCountW(
    hIMC: HIMC,
    lpdwListCount: LPDWORD,
) -> DWORD {
    return ImmGetCandidateListCountAW(hIMC, lpdwListCount, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCandidateListW(
    hIMC: HIMC,
    dwIndex: DWORD,
    lpCandList: LPCANDIDATELIST,
    dwBufLen: DWORD,
) -> DWORD {
    return ImmGetCandidateListAW(hIMC, dwIndex, lpCandList, dwBufLen, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCandidateWindow(hIMC: HIMC, dwIndex: DWORD, lpCandidate: LPCANDIDATEFORM) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pCF: LPCANDIDATEFORM = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/candidate.c\0"
                as *const u8 as *const i8,
            b"ImmGetCandidateWindow\0" as *const u8 as *const i8,
            405i32,
            b"(%p, %lu, %p)\n\0" as *const u8 as *const i8,
            hIMC,
            dwIndex,
            lpCandidate,
        );
    }

    if dwIndex >= MAX_CANDIDATEFORM {
        if (*__wine_dbch___default).flags as i32
            & (1i32) << __WINE_DBCL_ERR as i32
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/candidate.c\0"
                    as *const u8 as *const i8,
                b"ImmGetCandidateWindow\0" as *const u8 as *const i8,
                409i32,
                b"Out of boundary\n\0" as *const u8 as *const i8,
            );
        }
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        if (*__wine_dbch___default).flags as i32
            & (1i32) << __WINE_DBCL_ERR as i32
            != 0
            && __wine_dbg_get_channel_flags(__wine_dbch___default) as i32
                & (1i32) << __WINE_DBCL_ERR as i32
                != 0
        {
            wine_dbg_printf(
                b"err:(%s:%d) %s was NULL\n\0" as *const u8 as *const i8,
                b"win32ss/user/imm32/candidate.c\0"
                    as *const u8 as *const i8,
                414i32,
                b"pIC\0" as *const u8 as *const i8,
            );
        }
        return (FALSE) as BOOL;
    }

    pCF = (core::ptr::addr_of_mut!((*pIC).cfCandForm) as *mut CANDIDATEFORM)
        .offset(dwIndex as isize) as *mut CANDIDATEFORM as LPCANDIDATEFORM;
    if (*pCF).dwIndex != IMM_INVALID_CANDFORM {
        *lpCandidate = *pCF;
        ret = (TRUE) as i32;
    }

    ImmUnlockIMC(hIMC);

    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"win32ss/user/imm32/candidate.c\0"
                as *const u8 as *const i8,
            b"ImmGetCandidateWindow\0" as *const u8 as *const i8,
            425i32,
            b"ret: %d\n\0" as *const u8 as *const i8,
            ret,
        );
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetCandidateWindow(
    hIMC: HIMC,
    lpCandidate: LPCANDIDATEFORM,
) -> BOOL {
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();

    if (*lpCandidate).dwIndex >= MAX_CANDIDATEFORM {
        return (FALSE) as BOOL;
    }

    if IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    hWnd = (*pIC).hWnd;
    (*pIC).cfCandForm[(*lpCandidate).dwIndex as usize] = *lpCandidate;

    ImmUnlockIMC(hIMC);

    Imm32MakeIMENotify(
        hIMC,
        hWnd,
        NI_CONTEXTUPDATED,
        0,
        (IMC_SETCANDIDATEPOS) as DWORD_PTR,
        IMN_SETCANDIDATEPOS,
        ((1 << ((*lpCandidate).dwIndex as BYTE as u32)) as DWORD) as DWORD_PTR,
    );
    return (TRUE) as BOOL;
}
