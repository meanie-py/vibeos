use crate::imm32::ime::Imm32FindOrLoadImeDpi;
use crate::imm32::ime::ImmUnlockImeDpi;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "imm";

#[repr(C)]
pub struct ENUM_WORD_A2W {
    pub lpfnEnumProc: REGISTERWORDENUMPROCW,
    pub lpData: LPVOID,
    pub ret: UINT,
    pub uCodePage: UINT,
}
pub type LPENUM_WORD_A2W = *mut ENUM_WORD_A2W;

#[repr(C)]
pub struct ENUM_WORD_W2A {
    pub lpfnEnumProc: REGISTERWORDENUMPROCA,
    pub lpData: LPVOID,
    pub ret: UINT,
    pub uCodePage: UINT,
}
pub type LPENUM_WORD_W2A = *mut ENUM_WORD_W2A;


unsafe extern "system" fn Imm32EnumWordProcA2W(
    pszReadingA: LPCSTR,
    dwStyle: DWORD,
    pszRegisterA: LPCSTR,
    lpData: LPVOID,
) -> INT {
    let mut ret: INT = 0;
    let lpEnumData: LPENUM_WORD_A2W = lpData as LPENUM_WORD_A2W;
    let mut pszReadingW: LPWSTR = core::ptr::null_mut();
    let mut pszRegisterW: LPWSTR = core::ptr::null_mut();

    'Quit: {
        if !pszReadingA.is_null() {
            pszReadingW = Imm32WideFromAnsi((*lpEnumData).uCodePage, pszReadingA);
            if IS_NULL_UNEXPECTEDLY!(pszReadingW) != false {
                break 'Quit;
            }
        }

        if !pszRegisterA.is_null() {
            pszRegisterW = Imm32WideFromAnsi((*lpEnumData).uCodePage, pszRegisterA);
            if IS_NULL_UNEXPECTEDLY!(pszRegisterW) != false {
                break 'Quit;
            }
        }

        ret = (*lpEnumData)
            .lpfnEnumProc
            .unwrap_unchecked()(
            pszReadingW as LPCWSTR,
            dwStyle,
            pszRegisterW as LPCWSTR,
            (*lpEnumData).lpData,
        );
        (*lpEnumData).ret = ret as UINT;
    }

    ImmLocalFree!(pszReadingW as LPVOID);
    ImmLocalFree!(pszRegisterW as LPVOID);
    return ret;
}

unsafe extern "system" fn Imm32EnumWordProcW2A(
    pszReadingW: LPCWSTR,
    dwStyle: DWORD,
    pszRegisterW: LPCWSTR,
    lpData: LPVOID,
) -> INT {
    let mut ret: INT = 0;
    let lpEnumData: LPENUM_WORD_W2A = lpData as LPENUM_WORD_W2A;
    let mut pszReadingA: LPSTR = core::ptr::null_mut();
    let mut pszRegisterA: LPSTR = core::ptr::null_mut();

    'quit: {
        if !pszReadingW.is_null() {
            pszReadingA = Imm32AnsiFromWide((*lpEnumData).uCodePage, pszReadingW);
            if IS_NULL_UNEXPECTEDLY!(pszReadingW) != false {
                break 'quit;
            }
        }

        if !pszRegisterW.is_null() {
            pszRegisterA = Imm32AnsiFromWide((*lpEnumData).uCodePage, pszRegisterW);
            if IS_NULL_UNEXPECTEDLY!(pszRegisterA) != false {
                break 'quit;
            }
        }

        ret = (*lpEnumData).lpfnEnumProc.unwrap_unchecked()(
            pszReadingA as LPCSTR,
            dwStyle,
            pszRegisterA as LPCSTR,
            (*lpEnumData).lpData,
        );
        (*lpEnumData).ret = ret as UINT;
    }

    ImmLocalFree!(pszReadingA as LPVOID);
    ImmLocalFree!(pszRegisterA as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmEnumRegisterWordA(
    hKL: HKL,
    lpfnEnumProc: REGISTERWORDENUMPROCA,
    lpszReading: LPCSTR,
    dwStyle: DWORD,
    lpszRegister: LPCSTR,
    lpData: LPVOID,
) -> UINT {
    let mut ret: UINT = 0;
    let mut pszReadingW: LPWSTR = core::ptr::null_mut();
    let mut pszRegisterW: LPWSTR = core::ptr::null_mut();
    let mut EnumDataW2A: ENUM_WORD_W2A = ENUM_WORD_W2A {
        lpfnEnumProc: None,
        lpData: core::ptr::null_mut(),
        ret: 0,
        uCodePage: 0,
    };
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return 0;
    }

    if ImeDpi_IsUnicode!(pImeDpi) == false {
        /* No conversion needed */
        ret = (*pImeDpi)
            .ImeEnumRegisterWord
            .unwrap_unchecked()(
            core::mem::transmute::<REGISTERWORDENUMPROCA, LPVOID>(lpfnEnumProc),
            lpszReading as LPCVOID,
            dwStyle,
            lpszRegister as LPCVOID,
            lpData,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    'quit: {
        if !lpszReading.is_null() {
            pszReadingW = Imm32WideFromAnsi((*pImeDpi).uCodePage, lpszReading);
            if IS_NULL_UNEXPECTEDLY!(pszReadingW) != false {
                break 'quit;
            }
        }

        if !lpszRegister.is_null() {
            pszRegisterW = Imm32WideFromAnsi((*pImeDpi).uCodePage, lpszRegister);
            if IS_NULL_UNEXPECTEDLY!(pszRegisterW) != false {
                break 'quit;
            }
        }

        EnumDataW2A.lpfnEnumProc = lpfnEnumProc;
        EnumDataW2A.lpData = lpData;
        EnumDataW2A.ret = 0;
        EnumDataW2A.uCodePage = (*pImeDpi).uCodePage;
        (*pImeDpi)
            .ImeEnumRegisterWord
            .unwrap_unchecked()(
            core::mem::transmute::<
                Option<unsafe extern "system" fn(LPCWSTR, DWORD, LPCWSTR, LPVOID) -> INT>,
                LPVOID,
            >(Some(
                Imm32EnumWordProcW2A,
            )),
            pszReadingW as LPCVOID,
            dwStyle,
            pszRegisterW as LPCVOID,
            core::ptr::addr_of_mut!(EnumDataW2A) as LPVOID,
        );
        ret = EnumDataW2A.ret;
    }

    // Quit:
    ImmLocalFree!((pszReadingW).cast());
    ImmLocalFree!((pszRegisterW).cast());
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmEnumRegisterWordW(
    hKL: HKL,
    lpfnEnumProc: REGISTERWORDENUMPROCW,
    lpszReading: LPCWSTR,
    dwStyle: DWORD,
    lpszRegister: LPCWSTR,
    lpData: LPVOID,
) -> UINT {
    let mut ret: UINT = 0 as UINT;
    let mut pszReadingA: LPSTR = core::ptr::null_mut();
    let mut pszRegisterA: LPSTR = core::ptr::null_mut();
    let mut EnumDataA2W: ENUM_WORD_A2W = core::mem::zeroed();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return 0 as UINT;
    }

    if ImeDpi_IsUnicode!(pImeDpi) != false {
        ret = (*pImeDpi)
            .ImeEnumRegisterWord
            .unwrap_unchecked()(
            ::core::mem::transmute::<REGISTERWORDENUMPROCW, LPVOID>(lpfnEnumProc),
            lpszReading as LPCVOID,
            dwStyle,
            lpszRegister as LPCVOID,
            lpData,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    'quit: {
        if !lpszReading.is_null() {
            pszReadingA = Imm32AnsiFromWide((*pImeDpi).uCodePage, lpszReading);
            if IS_NULL_UNEXPECTEDLY!(pszReadingA) != false {
                break 'quit;
            }
        }

        if !lpszRegister.is_null() {
            pszRegisterA = Imm32AnsiFromWide((*pImeDpi).uCodePage, lpszRegister);
            if IS_NULL_UNEXPECTEDLY!(pszRegisterA) != false {
                break 'quit;
            }
        }

        EnumDataA2W.lpfnEnumProc = lpfnEnumProc;
        EnumDataA2W.lpData = lpData;
        EnumDataA2W.ret = 0 as UINT;
        EnumDataA2W.uCodePage = (*pImeDpi).uCodePage;
        (*pImeDpi)
            .ImeEnumRegisterWord
            .unwrap_unchecked()(
            ::core::mem::transmute::<
                Option<unsafe extern "system" fn(LPCSTR, DWORD, LPCSTR, LPVOID) -> INT>,
                LPVOID,
            >(Some(
                Imm32EnumWordProcA2W,
            )),
            pszReadingA as LPCVOID,
            dwStyle,
            pszRegisterA as LPCVOID,
            core::ptr::addr_of_mut!(EnumDataA2W) as LPVOID,
        );
        ret = EnumDataA2W.ret;
    }

    ImmLocalFree!((pszReadingA).cast());
    ImmLocalFree!((pszRegisterA).cast());
    ImmUnlockImeDpi(pImeDpi);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetRegisterWordStyleA(
    hKL: HKL,
    nItem: UINT,
    lpStyleBuf: LPSTYLEBUFA,
) -> UINT {
    let mut iItem: UINT = 0;
    let mut ret: UINT = 0;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut pDestA: LPSTYLEBUFA = core::ptr::null_mut();
    let mut pSrcW: LPSTYLEBUFW = core::ptr::null_mut();
    let mut pNewStylesW: LPSTYLEBUFW = core::ptr::null_mut();
    let mut cchW: size_t = 0;
    let mut cchA: INT = 0;

    'quit: {
        pImeDpi = Imm32FindOrLoadImeDpi(hKL);
        if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
            return 0;
        }

        if ImeDpi_IsUnicode!(pImeDpi) == false {
            /* No conversion needed */
            ret = (*pImeDpi)
                .ImeGetRegisterWordStyle
                .unwrap_unchecked()(nItem, lpStyleBuf as LPVOID);
            break 'quit;
        }

        if nItem > 0 {
            pNewStylesW = ImmLocalAlloc(
                0,
                (nItem as usize).wrapping_mul(core::mem::size_of::<STYLEBUFW>())
                    as DWORD,
            ) as LPSTYLEBUFW;
            if IS_NULL_UNEXPECTEDLY!(pNewStylesW) != false {
                break 'quit;
            }
        }

        ret = (*pImeDpi)
            .ImeGetRegisterWordStyle
            .unwrap_unchecked()(nItem, pNewStylesW as LPVOID);

        if nItem > 0 {
            /* lpStyleBuf <-- pNewStylesW */
            iItem = 0;
            while iItem < ret {
                pSrcW = pNewStylesW.add(iItem as usize);
                pDestA = lpStyleBuf.add(iItem as usize);
                (*pDestA).dwStyle = (*pSrcW).dwStyle;
                StringCchLengthW(
                    core::ptr::addr_of_mut!((*pSrcW).szDescription) as *mut WCHAR as STRSAFE_LPCWSTR,
                    (_countof!((*pSrcW).szDescription)) as _,
                    core::ptr::addr_of_mut!(cchW),
                );
                cchA = WideCharToMultiByte(
                    (*pImeDpi).uCodePage,
                    0,
                    core::ptr::addr_of_mut!((*pSrcW).szDescription) as *mut WCHAR as LPCWSTR,
                    cchW as INT,
                    core::ptr::addr_of_mut!((*pDestA).szDescription) as LPSTR,
                    _countof!((*pDestA).szDescription) as INT,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                ) as INT;
                if cchA as usize
                    > (_countof!((*pDestA).szDescription).wrapping_sub((1 as usize) as _)) as _
                {
                    cchA = _countof!((*pDestA).szDescription)
                        .wrapping_sub((1 as usize) as _) as INT;
                }
                (*pDestA).szDescription[cchA as usize] = ANSI_NULL;
                iItem = iItem.wrapping_add(1);
            }
        }
    }

    // Quit:
    ImmLocalFree!((pNewStylesW).cast());
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetRegisterWordStyleW(
    hKL: HKL,
    nItem: UINT,
    lpStyleBuf: LPSTYLEBUFW,
) -> UINT {
    let mut iItem: UINT = 0;
    let mut ret: UINT = 0;
    let mut pImeDpi: PIMEDPI;
    let mut pSrcA: LPSTYLEBUFA;
    let mut pNewStylesA: LPSTYLEBUFA = core::ptr::null_mut();
    let mut pDestW: LPSTYLEBUFW;
    let mut cchA: size_t = 0;
    let mut cchW: INT;

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return 0;
    }

    'Quit: {
        if ImeDpi_IsUnicode!(pImeDpi) != false {
            /* No conversion needed */
            ret = (*pImeDpi)
                .ImeGetRegisterWordStyle
                .unwrap_unchecked()(nItem, lpStyleBuf as LPVOID);
            break 'Quit;
        }

        if nItem > 0 {
            pNewStylesA = ImmLocalAlloc(
                0,
                (nItem as usize).wrapping_mul(core::mem::size_of::<STYLEBUFA>())
                    as DWORD,
            ) as LPSTYLEBUFA;
            if IS_NULL_UNEXPECTEDLY!(pNewStylesA) != false {
                break 'Quit;
            }
        }

        ret = (*pImeDpi)
            .ImeGetRegisterWordStyle
            .unwrap_unchecked()(nItem, pNewStylesA as LPVOID);

        if nItem > 0 {
            /* lpStyleBuf <-- pNewStylesA */
            iItem = 0;
            while iItem < ret {
                pSrcA = pNewStylesA.offset(iItem as isize);
                pDestW = lpStyleBuf.offset(iItem as isize);
                (*pDestW).dwStyle = (*pSrcA).dwStyle;
                StringCchLengthA(
                    core::ptr::addr_of_mut!((*pSrcA).szDescription) as *mut CHAR as STRSAFE_LPCSTR,
                    _countof!((*pSrcA).szDescription) as usize,
                    core::ptr::addr_of_mut!(cchA),
                );
                cchW = MultiByteToWideChar(
                    (*pImeDpi).uCodePage,
                    MB_PRECOMPOSED,
                    core::ptr::addr_of_mut!((*pSrcA).szDescription) as *mut CHAR as LPCSTR,
                    cchA as INT,
                    core::ptr::addr_of_mut!((*pDestW).szDescription) as *mut WCHAR as LPWSTR,
                    _countof!((*pDestW).szDescription) as INT,
                ) as INT;
                if cchW > _countof!((*pDestW).szDescription) as INT - 1 {
                    cchW = _countof!((*pDestW).szDescription) as INT - 1;
                }
                (*pDestW).szDescription[cchW as usize] = UNICODE_NULL as WCHAR;
                iItem = iItem.wrapping_add(1);
            }
        }
    }

    // Quit:
    ImmLocalFree!(pNewStylesA as PVOID);
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmRegisterWordA(
    hKL: HKL,
    lpszReading: LPCSTR,
    dwStyle: DWORD,
    lpszRegister: LPCSTR,
) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut pImeDpi: PIMEDPI;
    let mut pszReadingW: LPWSTR = core::ptr::null_mut();
    let mut pszRegisterW: LPWSTR = core::ptr::null_mut();

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }

    if ImeDpi_IsUnicode!(pImeDpi) == false {
        /* No conversion needed */
        ret = (*pImeDpi)
            .ImeRegisterWord
            .unwrap_unchecked()(
            lpszReading as LPCVOID,
            dwStyle,
            lpszRegister as LPCVOID,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    'quit: {
        if !lpszReading.is_null() {
            pszReadingW = Imm32WideFromAnsi((*pImeDpi).uCodePage, lpszReading);
            if IS_NULL_UNEXPECTEDLY!(pszReadingW) != false {
                break 'quit;
            }
        }

        if !lpszRegister.is_null() {
            pszRegisterW = Imm32WideFromAnsi((*pImeDpi).uCodePage, lpszRegister);
            if IS_NULL_UNEXPECTEDLY!(pszRegisterW) != false {
                break 'quit;
            }
        }

        ret = (*pImeDpi)
            .ImeRegisterWord
            .unwrap_unchecked()(
            pszReadingW as LPCVOID,
            dwStyle,
            pszRegisterW as LPCVOID,
        );
    }

    ImmLocalFree!((pszReadingW).cast());
    ImmLocalFree!((pszRegisterW).cast());
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmRegisterWordW(
    hKL: HKL,
    lpszReading: LPCWSTR,
    dwStyle: DWORD,
    lpszRegister: LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut pszReadingA: LPSTR = core::ptr::null_mut();
    let mut pszRegisterA: LPSTR = core::ptr::null_mut();

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }

    if ImeDpi_IsUnicode!(pImeDpi) != false {
        ret = (*pImeDpi).ImeRegisterWord.unwrap_unchecked()(
            lpszReading as LPCVOID,
            dwStyle,
            lpszRegister as LPCVOID,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    'quit: {
        if !lpszReading.is_null() {
            pszReadingA = Imm32AnsiFromWide((*pImeDpi).uCodePage, lpszReading);
            if IS_NULL_UNEXPECTEDLY!(pszReadingA) != false {
                break 'quit;
            }
        }

        if !lpszRegister.is_null() {
            pszRegisterA = Imm32AnsiFromWide((*pImeDpi).uCodePage, lpszRegister);
            if IS_NULL_UNEXPECTEDLY!(pszRegisterA) != false {
                break 'quit;
            }
        }

        ret = (*pImeDpi).ImeRegisterWord.unwrap_unchecked()(
            pszReadingA as LPCVOID,
            dwStyle,
            pszRegisterA as LPCVOID,
        );
    }

    ImmLocalFree!((pszReadingA).cast());
    ImmLocalFree!((pszRegisterA).cast());
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmUnregisterWordA(
    hKL: HKL,
    lpszReading: LPCSTR,
    dwStyle: DWORD,
    lpszUnregister: LPCSTR,
) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut pImeDpi: PIMEDPI;
    let mut pszReadingW: LPWSTR = core::ptr::null_mut();
    let mut pszUnregisterW: LPWSTR = core::ptr::null_mut();

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }

    if ImeDpi_IsUnicode!(pImeDpi) == false {
        /* No conversion needed */
        ret = (*pImeDpi).ImeUnregisterWord.unwrap_unchecked()(
            lpszReading as LPCVOID,
            dwStyle,
            lpszUnregister as LPCVOID,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    'quit: {
        if !lpszReading.is_null() {
            pszReadingW = Imm32WideFromAnsi((*pImeDpi).uCodePage, lpszReading);
            if IS_NULL_UNEXPECTEDLY!(pszReadingW) != false {
                break 'quit;
            }
        }

        if !lpszUnregister.is_null() {
            pszUnregisterW = Imm32WideFromAnsi((*pImeDpi).uCodePage, lpszUnregister);
            if IS_NULL_UNEXPECTEDLY!(pszUnregisterW) != false {
                break 'quit;
            }
        }

        ret = (*pImeDpi).ImeUnregisterWord.unwrap_unchecked()(
            pszReadingW as LPCVOID,
            dwStyle,
            pszUnregisterW as LPCVOID,
        );
    }

    ImmLocalFree!(pszReadingW as LPVOID);
    ImmLocalFree!(pszUnregisterW as LPVOID);
    ImmUnlockImeDpi(pImeDpi);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmUnregisterWordW(
    hKL: HKL,
    lpszReading: LPCWSTR,
    dwStyle: DWORD,
    lpszUnregister: LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut pszReadingA: LPSTR = core::ptr::null_mut();
    let mut pszUnregisterA: LPSTR = core::ptr::null_mut();

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }

    if ImeDpi_IsUnicode!(pImeDpi) != false {
        ret = (*pImeDpi)
            .ImeUnregisterWord
            .unwrap_unchecked()(
            lpszReading as LPCVOID,
            dwStyle,
            lpszUnregister as LPCVOID,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    'quit: {
        if !lpszReading.is_null() {
            pszReadingA = Imm32AnsiFromWide((*pImeDpi).uCodePage, lpszReading);
            if IS_NULL_UNEXPECTEDLY!(pszReadingA) != false {
                break 'quit;
            }
        }

        if !lpszUnregister.is_null() {
            pszUnregisterA = Imm32AnsiFromWide((*pImeDpi).uCodePage, lpszUnregister);
            if IS_NULL_UNEXPECTEDLY!(pszUnregisterA) != false {
                break 'quit;
            }
        }

        ret = (*pImeDpi)
            .ImeUnregisterWord
            .unwrap_unchecked()(
            pszReadingA as LPCVOID,
            dwStyle,
            pszUnregisterA as LPCVOID,
        );
    }

    ImmLocalFree!(pszReadingA as LPVOID);
    ImmLocalFree!(pszUnregisterA as LPVOID);
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}
