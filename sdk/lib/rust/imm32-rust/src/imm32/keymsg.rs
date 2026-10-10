use crate::imm32::ctf::CtfImeProcessCicHotkey;
use crate::imm32::ctf::CtfImmIsCiceroStartedInThread;
use crate::imm32::ctf::CtfImmSetLangBand;
use crate::imm32::imm::ImmEnumInputContext;
use crate::imm32::compstr::ImmGetCompositionStringA;
use crate::imm32::compstr::ImmGetCompositionStringW;
use crate::imm32::imm::ImmGetContext;
use crate::imm32::ime::ImmGetOpenStatus;
use crate::imm32::ime::ImmIsIME;
use crate::imm32::imm::ImmLockClientImc;
use crate::imm32::imm::ImmLockIMC;
use crate::imm32::ime::ImmLockImeDpi;
use crate::imm32::ime::ImmNotifyIME;
use crate::imm32::imm::ImmReleaseContext;
use crate::imm32::ime::ImmSetConversionStatus;
use crate::imm32::ime::ImmSetOpenStatus;
use crate::imm32::imm::ImmUnlockClientImc;
use crate::imm32::imm::ImmUnlockIMC;
use crate::imm32::ime::ImmUnlockImeDpi;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
}

#[allow(non_snake_case, unused_assignments, unused_mut, unused_variables)]

#[allow(non_snake_case, unused_assignments, unused_mut, unused_variables)]
static acbData: [size_t; 7 * 2] = [
    core::mem::size_of::<COMPOSITIONFORM>(),
    core::mem::size_of::<CANDIDATEFORM>(),
    core::mem::size_of::<LOGFONTW>(),
    core::mem::size_of::<RECONVERTSTRING>(),
    core::mem::size_of::<RECONVERTSTRING>(),
    core::mem::size_of::<IMECHARPOSITION>(),
    core::mem::size_of::<RECONVERTSTRING>(),
    core::mem::size_of::<COMPOSITIONFORM>(),
    core::mem::size_of::<CANDIDATEFORM>(),
    core::mem::size_of::<LOGFONTA>(),
    core::mem::size_of::<RECONVERTSTRING>(),
    core::mem::size_of::<RECONVERTSTRING>(),
    core::mem::size_of::<IMECHARPOSITION>(),
    core::mem::size_of::<RECONVERTSTRING>(),
];
unsafe extern "C" fn Imm32ImeNonImeToggle(hIMC: HIMC, hKL: HKL, hWnd: HWND, bNowIME: BOOL, LangID: LANGID) -> BOOL {
    let mut hOldKL: HKL = core::ptr::null_mut();
    let mut LayoutList: [HKL; 32] = [core::ptr::null_mut(); 32];
    let mut hFoundKL: HKL = core::ptr::null_mut();
    let mut iLayout: UINT;
    let mut nLayoutCount: UINT;

    /* Get the previous layout */
    hOldKL = NtUserGetThreadState(THREADSTATE_OLDKEYBOARDLAYOUT as DWORD) as HKL;

    /* Get the layout list */
    nLayoutCount = GetKeyboardLayoutList(
        _countof!(LayoutList) as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(LayoutList) as *mut HKL,
    );

    /* Is there hOldKL in the list in the specified language ID? */
    if !hOldKL.is_null()
        && (LangID == 0 || LOWORD!(hOldKL as DWORD) as LANGID == LangID)
    {
        iLayout = 0 as UINT;
        while iLayout < nLayoutCount {
            if LayoutList[iLayout as usize] == hOldKL {
                hFoundKL = hOldKL;
                break;
            }
            iLayout += 1;
        }
    }

    if hFoundKL.is_null() {
        /* Is there the keyboard layout of another kind in LangID? */
        iLayout = 0 as UINT;
        while iLayout < nLayoutCount {
            /* Same kind? -> continue (skip) */
            if bNowIME != ImmIsIME(LayoutList[iLayout as usize]) {
                if LangID == 0 as LANGID
                    || LangID == LOWORD!(LayoutList[iLayout as usize] as DWORD) as LANGID
                {
                    hFoundKL = LayoutList[iLayout as usize];
                    break;
                }
            }
            iLayout += 1;
        }
    }

    if !hFoundKL.is_null() && hKL != hFoundKL {
        /* Found and different layout */
        PostMessageW(
            hWnd,
            WM_INPUTLANGCHANGEREQUEST as UINT,
            1 as WPARAM,
            hFoundKL as LPARAM,
        );
    }

    return ImmIsIME(hFoundKL);
}

unsafe extern "C" fn Imm32CImeNonImeToggle(hIMC: HIMC, hKL: HKL, hWnd: HWND, LangID: LANGID) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut fOpen: BOOL;

    if IS_NULL_UNEXPECTEDLY!(hWnd) != false {
        return (FALSE) as BOOL;
    }

    if LOWORD!(hKL as DWORD_PTR) as LANGID != LangID || ImmIsIME(hKL) == 0 {
        Imm32ImeNonImeToggle(hIMC, hKL, hWnd, (FALSE) as BOOL, LangID);
        return (TRUE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (TRUE) as BOOL;
    }

    fOpen = (*pIC).fOpen;
    ImmUnlockIMC(hIMC);

    if fOpen != 0 {
        Imm32ImeNonImeToggle(hIMC, hKL, hWnd, (TRUE) as BOOL, 0);
    } else {
        ImmSetOpenStatus(hIMC, (TRUE) as BOOL);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32CShapeToggle(hIMC: HIMC, hKL: HKL, hWnd: HWND) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut fOpen: BOOL;
    let mut dwConversion: DWORD = 0;
    let mut dwSentence: DWORD = 0;

    if hWnd.is_null() || ImmIsIME(hKL) == 0 {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (TRUE) as BOOL;
    }

    fOpen = (*pIC).fOpen;
    if fOpen != 0 {
        dwConversion = (*pIC).fdwConversion ^ IME_CMODE_FULLSHAPE;
        dwSentence = (*pIC).fdwSentence;
    }

    ImmUnlockIMC(hIMC);

    if fOpen != 0 {
        ImmSetConversionStatus(hIMC, dwConversion, dwSentence);
    } else {
        ImmSetOpenStatus(hIMC, (TRUE) as BOOL);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32CSymbolToggle(hIMC: HIMC, hKL: HKL, hWnd: HWND) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut fOpen: BOOL;
    let mut dwConversion: DWORD = 0;
    let mut dwSentence: DWORD = 0;

    if hWnd.is_null() || ImmIsIME(hKL) == 0 {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (TRUE) as BOOL;
    }

    fOpen = (*pIC).fOpen;
    if fOpen != 0 {
        dwConversion = (*pIC).fdwConversion ^ IME_CMODE_SYMBOL;
        dwSentence = (*pIC).fdwSentence;
    }

    ImmUnlockIMC(hIMC);

    if fOpen != 0 {
        ImmSetConversionStatus(hIMC, dwConversion, dwSentence);
    } else {
        ImmSetOpenStatus(hIMC, (TRUE) as BOOL);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32JCloseOpen(mut hIMC: HIMC, mut hKL: HKL, mut hWnd: HWND) -> BOOL {
    let mut fOpen: BOOL = 0;
    let mut pIC: LPINPUTCONTEXTDX = core::ptr::null_mut::<INPUTCONTEXTDX>();
    if (LOWORD!(hKL as usize as u32) as u32) == (LANGID_JAPANESE as u32)
        && ImmIsIME(hKL) != 0
    {
        fOpen = ImmGetOpenStatus(hIMC);
        ImmSetOpenStatus(hIMC, (fOpen == 0) as BOOL);
        return (TRUE) as BOOL;
    }
    if Imm32ImeNonImeToggle(hIMC, hKL, hWnd, (FALSE) as BOOL, LANGID_JAPANESE as LANGID) != 0 {
        pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
        if !pIC.is_null() {
            (*pIC).dwChange |= INPUTCONTEXTDX_CHANGE_FORCE_OPEN as DWORD;
            ImmUnlockIMC(hIMC);
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32KShapeToggle(hIMC: HIMC) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut dwConversion: DWORD = 0;
    let mut dwSentence: DWORD = 0;
    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }
    dwConversion = (*pIC).fdwConversion ^ IME_CMODE_FULLSHAPE;
    dwSentence = (*pIC).fdwSentence;
    ImmSetConversionStatus(hIMC, dwConversion, dwSentence);
    if (*pIC).fdwConversion & (IME_CMODE_FULLSHAPE | IME_CMODE_NATIVE) != 0 {
        ImmSetOpenStatus(hIMC, (TRUE) as BOOL);
    } else {
        ImmSetOpenStatus(hIMC, (FALSE) as BOOL);
    }
    ImmUnlockIMC(hIMC);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32KHanjaConvert(hIMC: HIMC) -> BOOL {
    let mut pIC: LPINPUTCONTEXT;
    let mut dwConversion: DWORD;
    let mut dwSentence: DWORD;

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    dwConversion = ((*pIC).fdwConversion ^ IME_CMODE_HANJACONVERT);
    dwSentence = (*pIC).fdwSentence;
    ImmUnlockIMC(hIMC);

    ImmSetConversionStatus(hIMC, dwConversion, dwSentence);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32KEnglish(hIMC: HIMC) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut dwConversion: DWORD = 0;
    let mut dwSentence: DWORD = 0;
    let mut fOpen: BOOL = 0;
    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }
    dwConversion = (*pIC).fdwConversion ^ IME_CMODE_NATIVE;
    dwSentence = (*pIC).fdwSentence;
    ImmSetConversionStatus(hIMC, dwConversion, dwSentence);
    fOpen = (((*pIC).fdwConversion & (IME_CMODE_FULLSHAPE | IME_CMODE_NATIVE)) != 0) as BOOL;
    ImmSetOpenStatus(hIMC, fOpen);
    ImmUnlockIMC(hIMC);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32ProcessHotKey(
    mut hWnd: HWND,
    mut hIMC: HIMC,
    mut hKL: HKL,
    mut dwHotKeyID: DWORD,
) -> BOOL {
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut::<IMEDPI>();
    let mut ret: BOOL = 0;

    if !hIMC.is_null() && IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    match dwHotKeyID {
        IME_CHOTKEY_IME_NONIME_TOGGLE => {
            return Imm32CImeNonImeToggle(hIMC, hKL, hWnd, (LANGID_CHINESE_SIMPLIFIED) as LANGID);
        }
        IME_CHOTKEY_SHAPE_TOGGLE => {
            return Imm32CShapeToggle(hIMC, hKL, hWnd);
        }
        IME_CHOTKEY_SYMBOL_TOGGLE => {
            return Imm32CSymbolToggle(hIMC, hKL, hWnd);
        }
        IME_JHOTKEY_CLOSE_OPEN => {
            return Imm32JCloseOpen(hIMC, hKL, hWnd);
        }
        IME_KHOTKEY_SHAPE_TOGGLE => {
            return Imm32KShapeToggle(hIMC);
        }
        IME_KHOTKEY_HANJACONVERT => {
            return Imm32KHanjaConvert(hIMC);
        }
        IME_KHOTKEY_ENGLISH => {
            return Imm32KEnglish(hIMC);
        }
        IME_THOTKEY_IME_NONIME_TOGGLE => {
            return Imm32CImeNonImeToggle(hIMC, hKL, hWnd, (LANGID_CHINESE_TRADITIONAL) as LANGID);
        }
        IME_THOTKEY_SHAPE_TOGGLE => {
            return Imm32CShapeToggle(hIMC, hKL, hWnd);
        }
        IME_THOTKEY_SYMBOL_TOGGLE => {
            return Imm32CSymbolToggle(hIMC, hKL, hWnd);
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/user/imm32/keymsg.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32ProcessHotKey\0" as *const u8 as *const ::core::ffi::c_char,
                    293 as ::core::ffi::c_int,
                    b"0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
                    dwHotKeyID,
                );
            }
        }
    }

    if dwHotKeyID < IME_HOTKEY_PRIVATE_FIRST || IME_HOTKEY_PRIVATE_LAST < dwHotKeyID {
        return (FALSE) as BOOL;
    }

    pImeDpi = ImmLockImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }

    ret = (*pImeDpi).ImeEscape.unwrap_unchecked()(
        hIMC,
        IME_ESC_PRIVATE_HOTKEY as UINT,
        &raw mut dwHotKeyID as LPVOID,
    ) as BOOL;
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

unsafe extern "C" fn ImmIsUIMessageAW(
    mut hWndIME: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
    mut bAnsi: BOOL,
) -> BOOL {
    match msg {
        WM_IME_STARTCOMPOSITION | WM_IME_ENDCOMPOSITION | WM_IME_COMPOSITION | WM_IME_SETCONTEXT | WM_IME_NOTIFY | WM_IME_COMPOSITIONFULL | WM_IME_SELECT | WM_IME_SYSTEM => {}
        _ => return (FALSE) as i32,
    }
    if IS_NULL_UNEXPECTEDLY!(hWndIME) {
        return (TRUE) as BOOL;
    }
    if bAnsi != 0 {
        SendMessageA(hWndIME, msg, wParam, lParam);
    } else {
        SendMessageW(hWndIME, msg, wParam, lParam);
    }
    return (TRUE) as BOOL;
}

unsafe extern "system" fn Imm32SendNotificationProc(hIMC: HIMC, lParam: LPARAM) -> BOOL {
    let mut hWnd: HWND;
    let mut pIC: LPINPUTCONTEXTDX;

    UNREFERENCED_PARAMETER!(lParam);

    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
    if IS_NULL_UNEXPECTEDLY!(pIC) {
        return (TRUE) as BOOL;
    }

    hWnd = (*pIC).__bindgen_anon_1.hWnd;
    'quit: {
        if IsWindow(hWnd) == 0 {
            break 'quit;
        }

        if ((*pIC).dwChange & INPUTCONTEXTDX_CHANGE_OPEN) != 0 {
            SendMessageW(hWnd, WM_IME_NOTIFY, (IMN_SETOPENSTATUS) as WPARAM, 0);
        }
        if ((*pIC).dwChange & INPUTCONTEXTDX_CHANGE_CONVERSION) != 0 {
            SendMessageW(hWnd, WM_IME_NOTIFY, (IMN_SETCONVERSIONMODE) as WPARAM, 0);
        }
        if ((*pIC).dwChange & (INPUTCONTEXTDX_CHANGE_OPEN | INPUTCONTEXTDX_CHANGE_CONVERSION)) != 0 {
            NtUserNotifyIMEStatus(hWnd, (*pIC).__bindgen_anon_1.fOpen, (*pIC).__bindgen_anon_1.fdwConversion);
        }
        if ((*pIC).dwChange & INPUTCONTEXTDX_CHANGE_SENTENCE) != 0 {
            SendMessageW(hWnd, WM_IME_NOTIFY, (IMN_SETSENTENCEMODE) as WPARAM, 0);
        }
    }
    // Quit:
    (*pIC).dwChange = 0;
    ImmUnlockIMC(hIMC); // ??? Windows doesn't unlock here
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32SendNotification(mut bProcess: BOOL) -> BOOL {
    return ImmEnumInputContext(
        (if bProcess != 0 {
            -(1 as ::core::ffi::c_int)
        } else {
            0 as ::core::ffi::c_int
        }) as DWORD,
        Some(Imm32SendNotificationProc),
        0 as LPARAM,
    );
}

unsafe extern "C" fn Imm32ProcessRequest(
    mut hIMC: HIMC,
    mut pWnd: PWND,
    mut dwCommand: DWORD,
    mut pData: LPVOID,
    mut bAnsiAPI: BOOL,
) -> LRESULT {
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    let mut dwCharPos: DWORD = 0;
    let mut cchCompStr: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut pCS: LPVOID = core::ptr::null_mut();
    let mut pTempData: LPVOID = pData;
    let mut pRS: LPRECONVERTSTRING = core::ptr::null_mut();
    let mut pICP: LPIMECHARPOSITION = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut uCodePage: UINT = CP_ACP;
    let mut bAnsiWnd: BOOL =
        (((*pWnd).state & (WNDS_ANSIWINDOWPROC as DWORD)) != 0) as ::core::ffi::c_int;

    if dwCommand == 0 || dwCommand > IMR_DOCUMENTFEED {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0)
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/keymsg.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32ProcessRequest\0" as *const u8 as *const ::core::ffi::c_char,
                402 as ::core::ffi::c_int,
                b"Out of boundary\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    dwSize = acbData[((bAnsiAPI as DWORD).wrapping_mul(7 as DWORD))
        .wrapping_add(dwCommand)
        .wrapping_sub(1 as DWORD) as usize] as DWORD;
    if !pData.is_null() && IsBadWritePtr(pData as PVOID, dwSize as UINT_PTR) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0)
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/keymsg.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32ProcessRequest\0" as *const u8 as *const ::core::ffi::c_char,
                409 as ::core::ffi::c_int,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    /* Sanity check */
    match dwCommand {
        IMR_RECONVERTSTRING | IMR_DOCUMENTFEED => {
            pRS = pData as LPRECONVERTSTRING;
            if !pRS.is_null()
                && ((*pRS).dwVersion != 0
                    || ((*pRS).dwSize as usize)
                        < core::mem::size_of::<RECONVERTSTRING>())
            {
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0)
                {
                    let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_1,
                        __dbch_1,
                        b"win32ss/user/imm32/keymsg.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32ProcessRequest\0" as *const u8 as *const ::core::ffi::c_char,
                        420 as ::core::ffi::c_int,
                        b"Invalid pRS\n\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                return 0;
            }
        }
        IMR_CONFIRMRECONVERTSTRING => {
            pRS = pData as LPRECONVERTSTRING;
            if pRS.is_null() || (*pRS).dwVersion != 0 {
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0)
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"win32ss/user/imm32/keymsg.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32ProcessRequest\0" as *const u8 as *const ::core::ffi::c_char,
                        429 as ::core::ffi::c_int,
                        b"Invalid pRS\n\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                return 0;
            }
        }
        _ => {
            if IS_NULL_UNEXPECTEDLY!(pData) != false {
                return 0;
            }
        }
    }

    pClientImc = ImmLockClientImc(hIMC);
    if !pClientImc.is_null() {
        uCodePage = (*pClientImc).uCodePage;
        ImmUnlockClientImc(pClientImc);
    }

    /* Prepare */
    'doit: {
        match dwCommand {
            IMR_COMPOSITIONFONT => {
                if bAnsiAPI == bAnsiWnd {
                    break 'doit; /* No conversion needed */
                }

                if bAnsiWnd != 0 {
                    pTempData = ImmLocalAlloc(
                        0 as DWORD,
                        core::mem::size_of::<LOGFONTA>() as DWORD,
                    );
                } else {
                    pTempData = ImmLocalAlloc(
                        0 as DWORD,
                        core::mem::size_of::<LOGFONTW>() as DWORD,
                    );
                }

                if IS_NULL_UNEXPECTEDLY!(pTempData) != false {
                    return 0;
                }
            }

            IMR_RECONVERTSTRING | IMR_CONFIRMRECONVERTSTRING | IMR_DOCUMENTFEED => {
                if bAnsiAPI == bAnsiWnd || pData.is_null() {
                    break 'doit; /* No conversion needed */
                }

                if bAnsiWnd != 0 {
                    ret = Imm32ReconvertAnsiFromWide(
                        core::ptr::null_mut::<tagRECONVERTSTRING>(),
                        pData as *const RECONVERTSTRING,
                        uCodePage,
                    );
                } else {
                    ret = Imm32ReconvertWideFromAnsi(
                        core::ptr::null_mut::<tagRECONVERTSTRING>(),
                        pData as *const RECONVERTSTRING,
                        uCodePage,
                    );
                }

                pTempData = ImmLocalAlloc(
                    0 as DWORD,
                    ret.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD),
                );
                if IS_NULL_UNEXPECTEDLY!(pTempData) != false {
                    return 0;
                }

                pRS = pTempData as LPRECONVERTSTRING;
                (*pRS).dwSize = ret;
                (*pRS).dwVersion = 0;

                if dwCommand == IMR_CONFIRMRECONVERTSTRING {
                    if bAnsiWnd != 0 {
                        ret = Imm32ReconvertAnsiFromWide(
                            pTempData as LPRECONVERTSTRING,
                            pData as *const RECONVERTSTRING,
                            uCodePage,
                        );
                    } else {
                        ret = Imm32ReconvertWideFromAnsi(
                            pTempData as LPRECONVERTSTRING,
                            pData as *const RECONVERTSTRING,
                            uCodePage,
                        );
                    }
                }
            }

            IMR_QUERYCHARPOSITION => {
                if bAnsiAPI == bAnsiWnd {
                    break 'doit; /* No conversion needed */
                }

                pICP = pData as LPIMECHARPOSITION;
                dwCharPos = (*pICP).dwCharPos;

                if bAnsiAPI != 0 {
                    cchCompStr = ImmGetCompositionStringA(
                        hIMC,
                        GCS_COMPSTR as DWORD,
                        core::ptr::null_mut(),
                        0 as DWORD,
                    ) as DWORD;
                    if IS_ZERO_UNEXPECTEDLY!(cchCompStr) != false {
                        return 0;
                    }

                    pCS = ImmLocalAlloc(
                        0 as DWORD,
                        (cchCompStr.wrapping_add(1 as DWORD) as usize)
                            .wrapping_mul(core::mem::size_of::<CHAR>())
                            as DWORD,
                    );
                    if IS_NULL_UNEXPECTEDLY!(pCS) != false {
                        return 0;
                    }

                    ImmGetCompositionStringA(
                        hIMC,
                        GCS_COMPSTR as DWORD,
                        pCS,
                        cchCompStr,
                    );
                    (*pICP).dwCharPos = IchWideFromAnsi(
                        (*pICP).dwCharPos as LONG,
                        pCS as LPCSTR,
                        uCodePage,
                    ) as DWORD;
                } else {
                    cchCompStr = ImmGetCompositionStringW(
                        hIMC,
                        GCS_COMPSTR as DWORD,
                        core::ptr::null_mut(),
                        0 as DWORD,
                    ) as DWORD;
                    if IS_ZERO_UNEXPECTEDLY!(cchCompStr) != false {
                        return 0;
                    }

                    pCS = ImmLocalAlloc(
                        0 as DWORD,
                        (cchCompStr.wrapping_add(1 as DWORD) as usize)
                            .wrapping_mul(core::mem::size_of::<WCHAR>())
                            as DWORD,
                    );
                    if IS_NULL_UNEXPECTEDLY!(pCS) != false {
                        return 0;
                    }

                    ImmGetCompositionStringW(
                        hIMC,
                        GCS_COMPSTR as DWORD,
                        pCS,
                        cchCompStr,
                    );
                    (*pICP).dwCharPos = IchAnsiFromWide(
                        (*pICP).dwCharPos as LONG,
                        pCS as LPCWSTR,
                        uCodePage,
                    ) as DWORD;
                }

                ImmLocalFree!(pCS);
            }

            _ => {
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int))
                    != 0)
                {
                    let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_3: __wine_debug_class = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl_3,
                        __dbch_3,
                        b"win32ss/user/imm32/keymsg.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32ProcessRequest\0" as *const u8 as *const ::core::ffi::c_char,
                        527 as ::core::ffi::c_int,
                        b"0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
                        dwCommand,
                    );
                }
            }
        }
    }

    /* DoIt: The main task */
    'quit: {
        hWnd = (*pWnd).head.__bindgen_anon_1.__bindgen_anon_1.h as HWND;
        if bAnsiWnd != 0 {
            ret = SendMessageA(
                hWnd,
                WM_IME_REQUEST as UINT,
                dwCommand as WPARAM,
                pTempData as LPARAM,
            ) as DWORD;
        } else {
            ret = SendMessageW(
                hWnd,
                WM_IME_REQUEST as UINT,
                dwCommand as WPARAM,
                pTempData as LPARAM,
            ) as DWORD;
        }

        if bAnsiAPI == bAnsiWnd {
            break 'quit; /* No conversion needed */
        }

        /* Get back to caller */
        'sw3: {
            match dwCommand {
                IMR_COMPOSITIONFONT => {
                    if bAnsiAPI != 0 {
                        LogFontWideToAnsi(pTempData as *const LOGFONTW, pData as LPLOGFONTA);
                    } else {
                        LogFontAnsiToWide(pTempData as *const LOGFONTA, pData as LPLOGFONTW);
                    }
                }

                IMR_RECONVERTSTRING | IMR_DOCUMENTFEED => {
                    if ret == 0 {
                        break 'sw3;
                    }

                    if ret < core::mem::size_of::<RECONVERTSTRING>() as DWORD {
                        ret = 0;
                        break 'sw3;
                    }

                    if !pTempData.is_null() {
                        if bAnsiWnd != 0 {
                            ret = Imm32ReconvertWideFromAnsi(
                                pData as LPRECONVERTSTRING,
                                pTempData as *const RECONVERTSTRING,
                                uCodePage,
                            );
                        } else {
                            ret = Imm32ReconvertAnsiFromWide(
                                pData as LPRECONVERTSTRING,
                                pTempData as *const RECONVERTSTRING,
                                uCodePage,
                            );
                        }
                    }
                }

                IMR_QUERYCHARPOSITION => {
                    (*pICP).dwCharPos = dwCharPos;
                }

                _ => {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int)
                            << (__WINE_DBCL_WARN as ::core::ffi::c_int))
                        != 0)
                    {
                        let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_4: __wine_debug_class = __WINE_DBCL_WARN;
                        ros_dbg_log(
                            __dbcl_4,
                            __dbch_4,
                            b"win32ss/user/imm32/keymsg.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"Imm32ProcessRequest\0" as *const u8 as *const ::core::ffi::c_char,
                            576 as ::core::ffi::c_int,
                            b"0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
                            dwCommand,
                        );
                    }
                }
            }
        }
    }

    /* Quit: */
    if pTempData != pData {
        HeapFree(ghImmHeap, 0 as DWORD, pTempData as PVOID);
    }
    return ret as LRESULT;
}

unsafe extern "C" fn ImmRequestMessageAW(
    hIMC: HIMC,
    wParam: WPARAM,
    lParam: LPARAM,
    bAnsi: BOOL,
) -> LRESULT {
    let mut ret: LRESULT = 0;
    let mut pIC: LPINPUTCONTEXT;
    let mut hWnd: HWND;
    let mut pWnd: PWND = core::ptr::null_mut();

    if IS_NULL_UNEXPECTEDLY!(hIMC) != false || IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as LRESULT;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as LRESULT;
    }

    hWnd = (*pIC).hWnd;
    if !hWnd.is_null() {
        pWnd = ValidateHwnd!((hWnd).cast()) as PWND;
    }

    if !pWnd.is_null() && (*pWnd).head.__bindgen_anon_1.pti == Imm32CurrentPti() {
        ret = Imm32ProcessRequest(hIMC, pWnd, wParam as DWORD, lParam as LPVOID, bAnsi);
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmIsUIMessageA(
    hWndIME: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    return ImmIsUIMessageAW(hWndIME, msg, wParam, lParam, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmIsUIMessageW(hWndIME: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    return ImmIsUIMessageAW(hWndIME, msg, wParam, lParam, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetHotKey(
    dwHotKey: DWORD,
    lpuModifiers: LPUINT,
    lpuVKey: LPUINT,
    lphKL: LPHKL,
) -> BOOL {
    if !lpuModifiers.is_null() && !lpuVKey.is_null() {
        return NtUserGetImeHotKey(dwHotKey, lpuModifiers, lpuVKey, lphKL);
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmWINNLSGetIMEHotkey(hwndIme: HWND) -> UINT {
    UNREFERENCED_PARAMETER!(hwndIme);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSimulateHotKey(mut hWnd: HWND, mut dwHotKeyID: DWORD) -> BOOL {
    let mut hIMC: HIMC;
    let mut dwThreadId: DWORD;
    let mut hKL: HKL;
    let mut ret: BOOL;

    hIMC = ImmGetContext(hWnd);
    dwThreadId = GetWindowThreadProcessId(hWnd, core::ptr::null_mut());
    hKL = GetKeyboardLayout(dwThreadId);
    ret = Imm32ProcessHotKey(hWnd, hIMC, hKL, dwHotKeyID);
    ImmReleaseContext(hWnd, hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetVirtualKey(mut hWnd: HWND) -> UINT {
    let mut hIMC: HIMC;
    let mut pIC: LPINPUTCONTEXTDX;
    let mut ret: UINT = VK_PROCESSKEY;

    hIMC = ImmGetContext(hWnd);
    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return ret;
    }

    if (*pIC).bNeedsTrans != 0 {
        ret = (*pIC).nVKey;
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetAppCompatFlags(hIMC: HIMC) -> DWORD {
    let pClientIMC: PCLIENTIMC = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientIMC) != false {
        return 0;
    }

    let dwCompatFlags: DWORD = (*pClientIMC).dwCompatFlags;
    ImmUnlockClientImc(pClientIMC);
    return dwCompatFlags | g_aimm_compat_flags;
}

#[no_mangle]
pub unsafe extern "system" fn ImmProcessKey(
    mut hWnd: HWND,
    mut hKL: HKL,
    mut vKey: UINT,
    mut lParam: LPARAM,
    mut dwHotKeyID: DWORD,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut hIMC: HIMC = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXTDX = core::ptr::null_mut();
    let mut KeyState: [BYTE; 256] = [0; 256];
    let mut bLowWordOnly: BOOL = (FALSE) as i32;
    let mut bSkipThisKey: BOOL = (FALSE) as i32;
    let mut bHotKeyDone: BOOL = (TRUE) as i32;

    /* Process the key by the IME */
    hIMC = ImmGetContext(hWnd);
    pImeDpi = ImmLockImeDpi(hKL);
    if !pImeDpi.is_null() {
        pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
        if !pIC.is_null() {
            if (LOBYTE!(vKey) as UINT) == VK_PACKET
                && ((*pImeDpi).ImeInfo.fdwProperty & IME_PROP_ACCEPT_WIDE_VKEY) == 0
            {
                if ImeDpi_IsUnicode!(pImeDpi) != false {
                    bLowWordOnly = (TRUE) as i32;
                } else {
                    if (*pIC).__bindgen_anon_1.fOpen != 0 {
                        ret |= IPHK_SKIPTHISKEY;
                    }
                    bSkipThisKey = (TRUE) as i32;
                }
            }

            if bSkipThisKey == 0
                && GetKeyboardState(core::ptr::addr_of_mut!(KeyState) as PBYTE) != 0
            {
                let vk: UINT = if bLowWordOnly != 0 {
                    LOWORD!(vKey) as UINT
                } else {
                    vKey
                };
                if (*pImeDpi).ImeProcessKey.unwrap_unchecked()(
                    hIMC,
                    vk,
                    lParam,
                    core::ptr::addr_of_mut!(KeyState) as LPBYTE,
                ) != 0
                {
                    (*pIC).bNeedsTrans = (TRUE) as i32;
                    (*pIC).nVKey = vKey;
                    ret |= IPHK_PROCESSBYIME;
                }
            }

            ImmUnlockIMC(hIMC);
        }

        ImmUnlockImeDpi(pImeDpi);
    }

    /* Process the hot-key if necessary */
    if CtfImmIsCiceroStartedInThread() == 0 {
        /* Not Cicero? */
        /* Process IMM IME hotkey */
        if dwHotKeyID == INVALID_HOTKEY_ID
            || Imm32ProcessHotKey(hWnd, hIMC, hKL, dwHotKeyID) == 0
        {
            bHotKeyDone = (FALSE) as i32;
        }
    } else if CtfImeProcessCicHotkey(hIMC, vKey, lParam) == 0 {
        /* CTF IME not processed the hotkey? */
        /* Process IMM IME hotkey */
        if IS_IME_HKL!(hKL) == false
            || (dwHotKeyID == INVALID_HOTKEY_ID
                || Imm32ProcessHotKey(hWnd, hIMC, hKL, dwHotKeyID) == 0)
        {
            bHotKeyDone = (FALSE) as i32;
        }
    }

    if bHotKeyDone != 0
        && ((vKey != VK_KANJI) || (dwHotKeyID != IME_JHOTKEY_CLOSE_OPEN))
    {
        ret |= IPHK_HOTKEY;
    }

    if (ret & IPHK_PROCESSBYIME) != 0
        && (ImmGetAppCompatFlags(hIMC) & _IME_APP_COMPAT_PROCESS_BY_IME) != 0
    {
        /* The key has been processed by IME's ImeProcessKey */
        let wLangID: LANGID = LANGIDFROMLCID!(GetSystemDefaultLCID()) as LANGID;
        if (PRIMARYLANGID!(wLangID) as UINT) == LANG_KOREAN
            && ((vKey == VK_PROCESSKEY) || (ret & IPHK_HOTKEY) != 0)
        {
            /* Korean don't want VK_PROCESSKEY and IME hot-keys */
        } else {
            /* Add WM_KEYDOWN:VK_PROCESSKEY message */
            ImmTranslateMessage(hWnd, WM_KEYDOWN, (VK_PROCESSKEY) as WPARAM, lParam);

            ret &= !IPHK_PROCESSBYIME;
            ret |= IPHK_SKIPTHISKEY;
        }
    }

    ImmReleaseContext(hWnd, hIMC);
    return ret; /* Returns IPHK_... flags */
}

#[no_mangle]
pub unsafe extern "system" fn ImmSystemHandler(
    hIMC: HIMC,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match wParam {
        v if v == IMS_SENDNOTIFICATION as WPARAM => {
            Imm32SendNotification(lParam as BOOL);
            return 0;
        }
        v if v == IMS_COMPLETECOMPSTR as WPARAM => {
            ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_COMPLETE, 0);
            return 0;
        }
        v if v == IMS_SETLANGBAND as WPARAM || v == IMS_UNSETLANGBAND as WPARAM => {
            return CtfImmSetLangBand(lParam as HWND, (wParam == (IMS_SETLANGBAND as WPARAM)) as i32);
        }
        _ => {
            WARN!(b"%p\n\0".as_ptr() as *const i8, wParam);
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ImmGenerateMessage(hIMC: HIMC) -> BOOL {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pMsgs: LPTRANSMSG = core::ptr::null_mut();
    let mut pTrans: LPTRANSMSG = core::ptr::null_mut();
    let mut pItem: LPTRANSMSG = core::ptr::null_mut();
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut dwIndex: DWORD;
    let mut dwCount: DWORD;
    let mut cbTrans: DWORD;
    let mut hMsgBuf: HIMCC = core::ptr::null_mut();
    let mut bAnsi: BOOL;

    'quit: {
        if IS_CROSS_THREAD_HIMC!(hIMC) != false {
            return (FALSE) as BOOL;
        }

        pClientImc = ImmLockClientImc(hIMC);
        if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
            return (FALSE) as BOOL;
        }

        bAnsi = (((*pClientImc).dwFlags & CLIENTIMC_WIDE) == 0) as BOOL;
        ImmUnlockClientImc(pClientImc);

        pIC = ImmLockIMC(hIMC);
        if IS_NULL_UNEXPECTEDLY!(pIC) != false {
            return (FALSE) as BOOL;
        }

        dwCount = (*pIC).dwNumMsgBuf;
        if dwCount == 0 {
            break 'quit;
        }

        hMsgBuf = (*pIC).hMsgBuf;
        pMsgs = ImmLockIMCC(hMsgBuf) as LPTRANSMSG;
        if IS_NULL_UNEXPECTEDLY!(pMsgs) != false {
            break 'quit;
        }

        cbTrans = (dwCount as usize)
            .wrapping_mul(core::mem::size_of::<TRANSMSG>() as usize)
            as DWORD;
        pTrans = ImmLocalAlloc(0, cbTrans) as LPTRANSMSG;
        if IS_NULL_UNEXPECTEDLY!(pTrans) != false {
            break 'quit;
        }

        RtlCopyMemory!(
            pTrans as *mut core::ffi::c_void,
            pMsgs as *const core::ffi::c_void,
            cbTrans as usize,
        );

        if (*GetWin32ClientInfo!()).dwExpWinVer < _WIN32_WINNT_NT4 {
            let mut LangID: LANGID = LANGIDFROMLCID!(GetSystemDefaultLCID());
            let mut wLang: WORD = PRIMARYLANGID!(LangID);

            if (wLang as core::ffi::c_int) as u32 == LANG_JAPANESE
                || ((wLang as core::ffi::c_int) as u32 == LANG_KOREAN
                    && NtUserGetAppImeLevel((*pIC).hWnd) == 3)
            {
                dwCount = WINNLSTranslateMessage(
                    dwCount as INT,
                    pTrans as PTRANSMSG,
                    hIMC,
                    bAnsi,
                    wLang,
                );
            }
        }

        hWnd = (*pIC).hWnd;
        pItem = pTrans;
        dwIndex = 0;
        while dwIndex < dwCount {
            SendMessageW(hWnd, (*pItem).message, (*pItem).wParam, (*pItem).lParam);
            dwIndex = dwIndex.wrapping_add(1);
            pItem = pItem.offset(1);
        }

        UNREFERENCED_PARAMETER!(bAnsi);
    }

    ImmLocalFree!((pTrans).cast());
    if !hMsgBuf.is_null() && !pMsgs.is_null() {
        ImmUnlockIMCC(hMsgBuf);
    }
    (*pIC).dwNumMsgBuf = 0;
    ImmUnlockIMC(hIMC);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn ImmPostMessages(
    mut hwnd: HWND,
    mut hIMC: HIMC,
    mut dwCount: DWORD,
    mut lpTransMsg: LPTRANSMSG,
) {
    let mut dwIndex: DWORD;
    let mut pClientImc: PCLIENTIMC;
    let mut pNewTransMsg: LPTRANSMSG = lpTransMsg;
    let mut pItem: LPTRANSMSG;
    let mut bAnsi: BOOL;

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return;
    }

    bAnsi = ((*pClientImc).dwFlags & CLIENTIMC_WIDE as DWORD == 0) as ::core::ffi::c_int as BOOL;
    ImmUnlockClientImc(pClientImc);

    if (*GetWin32ClientInfo!()).dwExpWinVer < _WIN32_WINNT_NT4 as DWORD {
        let mut LangID: LANGID = LANGIDFROMLCID!(GetSystemDefaultLCID());
        let mut Lang: WORD = PRIMARYLANGID!(LangID);

        if Lang as ::core::ffi::c_int == LANG_JAPANESE as ::core::ffi::c_int
            || (Lang as ::core::ffi::c_int == LANG_KOREAN as ::core::ffi::c_int
                && NtUserGetAppImeLevel(hwnd) == 3 as DWORD)
        {
            let mut cbTransMsg: DWORD = (dwCount as usize)
                .wrapping_mul(core::mem::size_of::<TRANSMSG>() as usize)
                as DWORD;
            pNewTransMsg = ImmLocalAlloc(0 as DWORD, cbTransMsg) as LPTRANSMSG;
            if !pNewTransMsg.is_null() {
                RtlCopyMemory!(
                    pNewTransMsg as LPVOID,
                    lpTransMsg as LPVOID,
                    cbTransMsg as usize,
                );
                dwCount = WINNLSTranslateMessage(
                    dwCount as ::core::ffi::c_int,
                    pNewTransMsg as PTRANSMSG,
                    hIMC,
                    bAnsi,
                    Lang,
                ) as DWORD;
            } else {
                pNewTransMsg = lpTransMsg;
            }
        }
    }

    pItem = pNewTransMsg;
    dwIndex = 0 as DWORD;
    while dwIndex < dwCount {
        if bAnsi != 0 {
            PostMessageA(hwnd, (*pItem).message, (*pItem).wParam, (*pItem).lParam);
        } else {
            PostMessageW(hwnd, (*pItem).message, (*pItem).wParam, (*pItem).lParam);
        }
        dwIndex = dwIndex.wrapping_add(1);
        pItem = pItem.offset(1);
    }

    if pNewTransMsg != lpTransMsg {
        ImmLocalFree!(pNewTransMsg as LPVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ImmTranslateMessage(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lKeyData: LPARAM,
) -> BOOL {
    const MSG_COUNT: usize = 0x100;
    let mut ret: BOOL = (FALSE) as i32;
    let mut kret: INT = 0;
    let mut pIC: LPINPUTCONTEXTDX;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut pList: LPTRANSMSGLIST = core::ptr::null_mut();
    let mut pTransMsg: LPTRANSMSG;
    let mut abKeyState: [BYTE; 256] = [0; 256];
    let mut hIMC: HIMC;
    let mut hKL: HKL;
    let mut vk: UINT = 0;
    let mut dwThreadId: DWORD;
    let mut dwCount: DWORD;
    let mut cbList: DWORD;
    let mut wch: WCHAR = 0;
    let mut wChar: WORD = 0;

    /* filter the message */
    match msg {
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {}
        _ => return (FALSE) as i32,
    }

    hIMC = ImmGetContext(hwnd);
    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        ImmReleaseContext(hwnd, hIMC);
        return (FALSE) as BOOL;
    }

    'quit: {
        if (*pIC).bNeedsTrans == 0 {
            /* directly post them */
            dwCount = (*pIC).__bindgen_anon_1.dwNumMsgBuf;
            if dwCount == 0 {
                break 'quit;
            }

            pTransMsg = ImmLockIMCC((*pIC).__bindgen_anon_1.hMsgBuf) as LPTRANSMSG;
            if !pTransMsg.is_null() {
                ImmPostMessages(hwnd, hIMC, dwCount, pTransMsg);
                ImmUnlockIMCC((*pIC).__bindgen_anon_1.hMsgBuf);
                ret = (TRUE) as i32;
            }
            (*pIC).__bindgen_anon_1.dwNumMsgBuf = 0; /* done */
            break 'quit;
        }
        (*pIC).bNeedsTrans = (FALSE) as i32; /* clear the flag */

        dwThreadId = GetWindowThreadProcessId(hwnd, core::ptr::null_mut());
        hKL = GetKeyboardLayout(dwThreadId);
        pImeDpi = ImmLockImeDpi(hKL);
        if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
            break 'quit;
        }

        if GetKeyboardState(core::ptr::addr_of_mut!(abKeyState) as PBYTE) == 0 {
            /* get keyboard ON/OFF status */
            break 'quit;
        }

        /* convert a virtual key if IME_PROP_KBD_CHAR_FIRST */
        vk = (*pIC).nVKey;
        if ((*pImeDpi).ImeInfo.fdwProperty & IME_PROP_KBD_CHAR_FIRST) != 0 {
            if ImeDpi_IsUnicode!(pImeDpi) != false {
                wch = 0 as WCHAR;
                kret = ToUnicode(
                    vk,
                    HIWORD!(lKeyData) as UINT,
                    core::ptr::addr_of_mut!(abKeyState) as *mut BYTE,
                    core::ptr::addr_of_mut!(wch),
                    1,
                    0,
                ) as INT;
                if kret == 1 {
                    vk = MAKELONG!(LOBYTE!(vk), wch) as UINT;
                }
            } else {
                wChar = 0 as WORD;
                kret = ToAsciiEx(
                    vk,
                    HIWORD!(lKeyData) as UINT,
                    core::ptr::addr_of_mut!(abKeyState) as *mut BYTE,
                    core::ptr::addr_of_mut!(wChar),
                    0,
                    hKL,
                ) as INT;
                if kret > 0 {
                    if (((vk as BYTE) as INT)) as u32 == VK_PACKET {
                        vk |= (wChar as UINT) << 8;
                    } else {
                        vk = MAKEWORD!(vk, wChar) as UINT;
                    }
                }
            }
        }

        /* allocate a list */
        cbList = (core::mem::offset_of!(TRANSMSGLIST, TransMsg)
            + MSG_COUNT * core::mem::size_of::<TRANSMSG>()) as DWORD;
        pList = ImmLocalAlloc(0, cbList) as LPTRANSMSGLIST;
        if IS_NULL_UNEXPECTEDLY!(pList) != false {
            break 'quit;
        }

        /* use IME conversion engine and convert the list */
        (*pList).uMsgCount = MSG_COUNT as UINT;
        kret = (*pImeDpi).ImeToAsciiEx.unwrap_unchecked()(
            vk,
            HIWORD!(lKeyData) as UINT,
            core::ptr::addr_of_mut!(abKeyState) as LPBYTE,
            pList,
            0,
            hIMC,
        ) as INT;
        if kret <= 0 {
            break 'quit;
        }

        /* post them */
        if kret <= MSG_COUNT as INT {
            ImmPostMessages(
                hwnd,
                hIMC,
                kret as DWORD,
                core::ptr::addr_of_mut!((*pList).TransMsg) as LPTRANSMSG,
            );
            ret = (TRUE) as i32;
        } else {
            pTransMsg = ImmLockIMCC((*pIC).__bindgen_anon_1.hMsgBuf) as LPTRANSMSG;
            if IS_NULL_UNEXPECTEDLY!(pTransMsg) != false {
                break 'quit;
            }
            ImmPostMessages(hwnd, hIMC, kret as DWORD, pTransMsg);
            ImmUnlockIMCC((*pIC).__bindgen_anon_1.hMsgBuf);
        }
    }

    /* Quit: */
    ImmLocalFree!((pList).cast());
    ImmUnlockImeDpi(pImeDpi);
    ImmUnlockIMC(hIMC);
    ImmReleaseContext(hwnd, hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmRequestMessageA(
    hIMC: HIMC,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    return ImmRequestMessageAW(hIMC, wParam, lParam, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmRequestMessageW(
    hIMC: HIMC,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    return ImmRequestMessageAW(hIMC, wParam, lParam, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmCallImeConsoleIME(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    puVK: LPUINT,
) -> DWORD {
    let mut dwThreadId: DWORD;
    let mut ret: DWORD = 0;
    let mut hKL: HKL;
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut hIMC: HIMC;
    let mut pImeDpi: PIMEDPI;
    let mut uVK: UINT;
    let mut pIMC: PIMC;

    if !(uMsg == WM_KEYDOWN
        || uMsg == WM_KEYUP
        || uMsg == WM_SYSKEYDOWN
        || uMsg == WM_SYSKEYUP)
    {
        return 0;
    }

    dwThreadId = GetWindowThreadProcessId(hWnd, core::ptr::null_mut());
    hKL = GetKeyboardLayout(dwThreadId);

    if !hWnd.is_null() && !gpsi.is_null() {
        pWnd = ValidateHwndNoErr!((hWnd).cast()) as PWND;
    }
    if IS_NULL_UNEXPECTEDLY!(pWnd) != false {
        return 0;
    }

    hIMC = ImmGetContext(hWnd);
    if IS_NULL_UNEXPECTEDLY!(hIMC) != false {
        return 0;
    }

    *puVK = (wParam & 0xFF) as UINT;
    uVK = *puVK;

    pIMC = ValidateHandleNoErr((hIMC).cast(), TYPE_INPUTCONTEXT) as PIMC;
    if IS_NULL_UNEXPECTEDLY!(pIMC) != false {
        return 0;
    }

    pImeDpi = ImmLockImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return 0;
    }

    'quit: {
        if (lParam & MAKELPARAM!(0, KF_UP)) != 0
            && ((*pImeDpi).ImeInfo.fdwProperty & IME_PROP_IGNORE_UPKEYS) != 0
        {
            break 'quit;
        }

        let vk_is_listed = uVK == VK_DBE_ROMAN
            || uVK == VK_DBE_NOROMAN
            || uVK == VK_DBE_HIRAGANA
            || uVK == VK_DBE_KATAKANA
            || uVK == VK_DBE_CODEINPUT
            || uVK == VK_DBE_NOCODEINPUT
            || uVK == VK_DBE_ENTERWORDREGISTERMODE
            || uVK == VK_DBE_ENTERIMECONFIGMODE;

        if !vk_is_listed {
            if uMsg == WM_SYSKEYDOWN || uMsg == WM_SYSKEYUP {
                if uVK != VK_MENU && uVK != VK_F10 {
                    break 'quit;
                }
            }

            if ((*pImeDpi).ImeInfo.fdwProperty & IME_PROP_NEED_ALTKEY) == 0 {
                if uVK == VK_MENU
                    || (lParam & MAKELPARAM!(0, KF_ALTDOWN)) != 0
                {
                    break 'quit;
                }
            }
        }

        ret = ImmProcessKey(hWnd, hKL, uVK, lParam, INVALID_HOTKEY_ID);
    }

    ImmUnlockImeDpi(pImeDpi);
    return ret;
}
