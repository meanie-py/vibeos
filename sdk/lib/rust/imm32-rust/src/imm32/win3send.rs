use crate::imm32::utils::Imm32IsSystemJapaneseOrKorean;
use crate::imm32::ime::ImmConfigureIMEA;
use crate::imm32::ime::ImmConfigureIMEW;
use crate::imm32::ime::ImmEscapeW;
use crate::imm32::keymsg::ImmGetAppCompatFlags;
use crate::imm32::compstr::ImmGetCompositionStringA;
use crate::imm32::compstr::ImmGetCompositionStringW;
use crate::imm32::imm::ImmGetContext;
use crate::imm32::ime::ImmGetConversionListA;
use crate::imm32::ime::ImmGetConversionStatus;
use crate::imm32::ime::ImmGetDefaultIMEWnd;
use crate::imm32::ime::ImmGetOpenStatus;
use crate::imm32::imm::ImmGetSaveContext;
use crate::imm32::ime::ImmIsIME;
use crate::imm32::imm::ImmLockClientImc;
use crate::imm32::imm::ImmLockIMC;
use crate::imm32::ime::ImmLockImeDpi;
use crate::imm32::ime::ImmNotifyIME;
use crate::imm32::imm::ImmReleaseContext;
use crate::imm32::ime::ImmSetCompositionFontW;
use crate::imm32::ime::ImmSetCompositionWindow;
use crate::imm32::ime::ImmSetConversionStatus;
use crate::imm32::ime::ImmSetOpenStatus;
use crate::imm32::imm::ImmUnlockClientImc;
use crate::imm32::imm::ImmUnlockIMC;
use crate::imm32::ime::ImmUnlockImeDpi;
use crate::imm32::utils::LogFontAnsiToWide;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_816B184A1146: [u16; 8] = [0x0057u16, 0x0041, 0x004E, 0x0053, 0x0055, 0x004E, 0x0047, 0x0000];
pub static RUSTCONV_WSTR_FB1BE064B20C: [u16; 12] = [
    0x0049u16, 0x006E, 0x0070, 0x0075, 0x0074, 0x004D, 0x0065, 0x0074, 0x0068, 0x006F,
    0x0064, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const INPUT_METHOD_BASE: u32 = 100;

pub static g_MNTable: [[u16; 96]; 3] = [
    [
        0xA1A1, 0xA3A1, 0xA1A8, 0xA3A3, 0xA3A4, 0xA3A5, 0xA3A6, 0xA1AE,
        0xA3A8, 0xA3A9, 0xA3AA, 0xA3AB, 0xA3A7, 0xA3AD, 0xA3AE, 0xA3AF,
        0xA3B0, 0xA3B1, 0xA3B2, 0xA3B3, 0xA3B4, 0xA3B5, 0xA3B6, 0xA3B7,
        0xA3B8, 0xA3B9, 0xA3BA, 0xA3BB, 0xA3BC, 0xA3BD, 0xA3BE, 0xA3BF,
        0xA3C0, 0xA4B1, 0xA4D0, 0xA4BA, 0xA4B7, 0xA4A8, 0xA4A9, 0xA4BE,
        0xA4C7, 0xA4C1, 0xA4C3, 0xA4BF, 0xA4D3, 0xA4D1, 0xA4CC, 0xA4C2,
        0xA4C6, 0xA4B3, 0xA4A2, 0xA4A4, 0xA4B6, 0xA4C5, 0xA4BD, 0xA4B9,
        0xA4BC, 0xA4CB, 0xA4BB, 0xA3DB, 0xA1AC, 0xA3DD, 0xA3DE, 0xA3DF,
        0xA1A2, 0xA4B1, 0xA4D0, 0xA4BA, 0xA4B7, 0xA4A7, 0xA4A9, 0xA4BE,
        0xA4C7, 0xA4C1, 0xA4C3, 0xA4BF, 0xA4D3, 0xA4D1, 0xA4CC, 0xA4C0,
        0xA4C4, 0xA4B2, 0xA4A1, 0xA4A4, 0xA4B5, 0xA4C5, 0xA4BD, 0xA4B8,
        0xA4BC, 0xA4CB, 0xA4BB, 0xA3FB, 0xA3FC, 0xA3FD, 0xA1AD, 0x0000
    ],
    [
        0xA1A1, 0xA4B8, 0xA1A8, 0xA3A3, 0xA3A4, 0xA3A5, 0xA3A6, 0xA1AE,
        0xA3A8, 0xA3A9, 0xA3AA, 0xA3AB, 0xA4BC, 0xA3AD, 0xA3AE, 0xA4C7,
        0xA4BB, 0xA4BE, 0xA4B6, 0xA4B2, 0xA4CB, 0xA4D0, 0xA4C1, 0xA4C6,
        0xA4D2, 0xA4CC, 0xA3BA, 0xA4B2, 0xA3B2, 0xA3BD, 0xA3B3, 0xA3BF,
        0xA3C0, 0xA4A7, 0xA3A1, 0xA4AB, 0xA4AA, 0xA4BB, 0xA4A2, 0xA3AF,
        0xA1AF, 0xA3B8, 0xA3B4, 0xA3B5, 0xA3B6, 0xA3B1, 0xA3B0, 0xA3B9,
        0xA3BE, 0xA4BD, 0xA4C2, 0xA4A6, 0xA4C3, 0xA3B7, 0xA4B0, 0xA4BC,
        0xA4B4, 0xA3BC, 0xA4BA, 0xA3DB, 0xA3DC, 0xA3DD, 0xA3DE, 0xA3DF,
        0xA1AE, 0xA4B7, 0xA4CC, 0xA4C4, 0xA4D3, 0xA4C5, 0xA4BF, 0xA4D1,
        0xA4A4, 0xA4B1, 0xA4B7, 0xA4A1, 0xA4B8, 0xA4BE, 0xA4B5, 0xA4BA,
        0xA4BD, 0xA4B5, 0xA4C0, 0xA4A4, 0xA4C3, 0xA4A7, 0xA4C7, 0xA4A9,
        0xA4A1, 0xA4A9, 0xA4B1, 0xA3FB, 0xA3FC, 0xA3FD, 0xA1AD, 0x0000
    ],
    [
        0xA1A1, 0xA4A2, 0xA3AE, 0xA4B8, 0xA4AF, 0xA4AE, 0xA1B0, 0xA3AA,
        0xA1A2, 0xA1AD, 0xA1B1, 0xA3AB, 0xA4BC, 0xA3A9, 0xA3AE, 0xA4C7,
        0xA4BB, 0xA4BE, 0xA4B6, 0xA4B2, 0xA4CB, 0xA4D0, 0xA4C1, 0xA4C6,
        0xA4D2, 0xA4CC, 0xA3B4, 0xA4B2, 0xA3A7, 0xA1B5, 0xA3AE, 0xA3A1,
        0xA4AA, 0xA4A7, 0xA3BF, 0xA4BC, 0xA4AC, 0xA4A5, 0xA4AB, 0xA4C2,
        0xA3B0, 0xA3B7, 0xA3B1, 0xA3B2, 0xA3B3, 0xA1A8, 0xA3AD, 0xA3B8,
        0xA3B9, 0xA4BD, 0xA4B0, 0xA4A6, 0xA4AD, 0xA3B6, 0xA4A3, 0xA4BC,
        0xA4B4, 0xA3B5, 0xA4BA, 0xA3A8, 0xA3BA, 0xA1B4, 0xA3BD, 0xA3BB,
        0xA3AA, 0xA4B7, 0xA4CC, 0xA4C4, 0xA4D3, 0xA4C5, 0xA4BF, 0xA4D1,
        0xA4A4, 0xA4B1, 0xA4B7, 0xA4A1, 0xA4B8, 0xA4BE, 0xA4B5, 0xA4BA,
        0xA4BD, 0xA4B5, 0xA4C0, 0xA4A4, 0xA4C3, 0xA4A7, 0xA4C7, 0xA4A9,
        0xA4A1, 0xA4B1, 0xA4B1, 0xA3A5, 0xA3CC, 0xA3AF, 0xA1AD, 0x0000
    ]
];

unsafe extern "C" fn Imm32IsForegroundThread(hWnd: HWND) -> BOOL {
    let hwndFore: HWND = GetForegroundWindow();
    let dwTID1: DWORD = if IsWindow(hWnd) != 0 {
        GetWindowThreadProcessId(hWnd, core::ptr::null_mut::<DWORD>())
    } else {
        GetCurrentThreadId()
    };
    let dwTID2: DWORD = GetWindowThreadProcessId(hwndFore, core::ptr::null_mut::<DWORD>());
    return (dwTID1 == dwTID2) as BOOL;
}

unsafe extern "C" fn Imm32PostImsMessage(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    let hwndIme: HWND = ImmGetDefaultIMEWnd(hWnd);
    if hwndIme.is_null() {
        return (FALSE) as BOOL;
    }
    return PostMessageW(hwndIme, WM_IME_SYSTEM, wParam, lParam);
}

unsafe extern "C" fn Imm32SetCandidateWindow(hWnd: HWND, hIMC: HIMC, lpCandidate: PCANDIDATEFORM) -> BOOL {
    let pIC: PINPUTCONTEXT = ImmLockIMC(hIMC) as PINPUTCONTEXT;
    if pIC.is_null() {
        return (FALSE) as BOOL;
    }

    let mut ret: BOOL = 0;
    let dwCompatFlags: DWORD = ImmGetAppCompatFlags(hIMC);
    if (dwCompatFlags & (_IME_APP_COMPAT_DIRECT_IME_SYSTEM as DWORD)) != 0 {
        RtlCopyMemory!(
            (core::ptr::addr_of_mut!((*pIC).cfCandForm) as *mut CANDIDATEFORM)
                .offset((*lpCandidate).dwIndex as isize) as *mut core::ffi::c_void,
            lpCandidate as *const core::ffi::c_void,
            core::mem::size_of::<CANDIDATEFORM>(),
        );
        ret = Imm32PostImsMessage(
            hWnd,
            IMS_SETCANDFORM as WPARAM,
            (*lpCandidate).dwIndex as LPARAM,
        );
    } else {
        ret = ImmSetCandidateWindow(hIMC, lpCandidate);
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

unsafe extern "C" fn Imm32SetCompWindow(hWnd: HWND, hIMC: HIMC, lpCompForm: PCOMPOSITIONFORM) -> BOOL {
    let pIC: PINPUTCONTEXTDX = ImmLockIMC(hIMC) as PINPUTCONTEXTDX;
    if pIC.is_null() {
        return (FALSE) as BOOL;
    }

    let mut ret: BOOL;
    let dwCompatFlags: DWORD = ImmGetAppCompatFlags(hIMC);
    if (dwCompatFlags & _IME_APP_COMPAT_DIRECT_IME_SYSTEM) != 0 {
        RtlCopyMemory!(
            core::ptr::addr_of_mut!((*pIC).__bindgen_anon_1.cfCompForm) as *mut core::ffi::c_void,
            lpCompForm as *const core::ffi::c_void,
            core::mem::size_of::<COMPOSITIONFORM>(),
        );
        if (dwCompatFlags & _IME_APP_COMPAT_SPECIAL_IME) != 0 {
            ret = PostMessageW(hWnd, WM_IME_SYSTEM, (IMS_SETCOMPFORM) as WPARAM, 0);
        } else {
            ret = Imm32PostImsMessage(hWnd, (IMS_SETCOMPFORM) as WPARAM, 0);
        }
    } else {
        (*pIC).dwUIFlags |= _IME_UI_NO_COMPFORM;
        ret = ImmSetCompositionWindow(hIMC, lpCompForm);
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

unsafe extern "C" fn Imm32Get31ModeFrom40ModeK(fdwConversion: DWORD) -> DWORD {
    let mut flags: DWORD = 0 as DWORD;
    if fdwConversion & IME_CMODE_NATIVE as DWORD == 0 {
        flags |= IME_MODE_ALPHANUMERIC as DWORD;
    }
    if fdwConversion & IME_CMODE_FULLSHAPE as DWORD == 0 {
        flags |= _IME_MODE_KOR_SBCSCHAR as DWORD;
    }
    if fdwConversion & IME_CMODE_HANJACONVERT as DWORD != 0 {
        flags |= IME_MODE_HANJACONVERT as DWORD;
    }
    return flags;
}

unsafe extern "C" fn Imm32Get31ModeFrom40ModeJ(mut fdwConversion: DWORD) -> DWORD {
    let mut flags: DWORD = 0 as DWORD;
    if fdwConversion & IME_CMODE_NATIVE as DWORD != 0 {
        if fdwConversion & IME_CMODE_KATAKANA as DWORD != 0 {
            flags |= IME_MODE_KATAKANA as DWORD;
        } else {
            flags |= IME_MODE_HIRAGANA as DWORD;
        }
    } else {
        flags |= IME_MODE_ALPHANUMERIC as DWORD;
    }
    if fdwConversion & IME_CMODE_FULLSHAPE as DWORD != 0 {
        flags |= IME_MODE_DBCSCHAR as DWORD;
    } else {
        flags |= _IME_MODE_JPN_SBCSCHAR as DWORD;
    }
    if fdwConversion & IME_CMODE_ROMAN as DWORD != 0 {
        flags |= IME_MODE_ROMAN as DWORD;
    } else {
        flags |= IME_MODE_NOROMAN as DWORD;
    }
    if fdwConversion & IME_CMODE_CHARCODE as DWORD != 0 {
        flags |= IME_MODE_CODEINPUT as DWORD;
    } else {
        flags |= IME_MODE_NOCODEINPUT as DWORD;
    }
    return flags;
}

unsafe extern "C" fn Imm32TransGetMode(hIMC: HIMC) -> LRESULT {
    let mut fdwConversion: DWORD = 0 as DWORD;
    let mut fdwSentence: DWORD = 0;
    ImmGetConversionStatus(hIMC, core::ptr::addr_of_mut!(fdwConversion), core::ptr::addr_of_mut!(fdwSentence));
    return (Imm32Get31ModeFrom40ModeK(fdwConversion) | _IME_CMODE_EXTENDED as DWORD) as LRESULT;
}

unsafe extern "C" fn Imm32TransSetMode(hIMC: HIMC, pIme: PIMESTRUCT) -> DWORD {
    let mut fdwConversion: DWORD = 0 as DWORD;
    let mut fdwSentence: DWORD = 0;
    ImmGetConversionStatus(hIMC, core::ptr::addr_of_mut!(fdwConversion), core::ptr::addr_of_mut!(fdwSentence));

    let dw31Mode: DWORD = Imm32Get31ModeFrom40ModeK(fdwConversion);
    let wParam: WPARAM = (*pIme).wParam;
    if (wParam & (_IME_MODE_KOR_SBCSCHAR as WPARAM)) == 0 {
        fdwConversion |= IME_CMODE_FULLSHAPE as DWORD;
    }

    let mut dwNewFlags: DWORD = fdwConversion;
    let targetMask: DWORD = (IME_CMODE_NATIVE as DWORD)
        | (IME_CMODE_KATAKANA as DWORD)
        | (IME_CMODE_FULLSHAPE as DWORD)
        | (IME_CMODE_HANJACONVERT as DWORD);
    if (wParam & (IME_MODE_ALPHANUMERIC as WPARAM)) != 0 {
        dwNewFlags &= !targetMask;
    } else {
        dwNewFlags |= targetMask;
    }

    if ImmSetConversionStatus(hIMC, dwNewFlags, fdwSentence) == 0 {
        return 0 as DWORD;
    }
    return dw31Mode;
}

unsafe extern "C" fn Imm32TransCodeConvert(hIMC: HIMC, pIme: PIMESTRUCT) -> BOOL {
    let hKL: HKL = GetKeyboardLayout(0);
    let uSubFunc: UINT = HIWORD!((*pIme).wParam as DWORD) as UINT;
    match uSubFunc {
        IME_BANJAtoJUNJA | IME_JUNJAtoBANJA | IME_JOHABtoKS | IME_KStoJOHAB => {
            return ImmEscapeW(hKL, hIMC, uSubFunc, pIme as LPVOID) as BOOL;
        }
        _ => {
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn Imm32TransGetOpenK(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT, bAnsi: BOOL) -> LRESULT {
    let mut rc: RECT = core::mem::zeroed();
    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rc));
    let OldlParam2: LPARAM = (*pIme).lParam2;
    (*pIme).lParam2 = MAKELONG!(rc.top as WORD, rc.left as WORD) as LPARAM; /* Correct! */
    let hKL: HKL = GetKeyboardLayout(0);
    let result: LRESULT = ImmEscapeW(hKL, hIMC, IME_GETOPEN, pIme as LPVOID);
    (*pIme).lParam2 = OldlParam2;
    return result;
}

unsafe extern "C" fn Imm32TransGetOpenJ(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT, bAnsi: BOOL) -> BOOL {
    let fOpen: BOOL = ImmGetOpenStatus(hIMC);

    let mut cbCompStr: LONG;
    if bAnsi != 0 {
        cbCompStr = ImmGetCompositionStringA(hIMC, GCS_COMPSTR, core::ptr::null_mut(), 0);
    } else {
        cbCompStr = ImmGetCompositionStringW(hIMC, GCS_COMPSTR, core::ptr::null_mut(), 0);
    }

    (*pIme).wCount = (if cbCompStr > 0 { cbCompStr } else { 0 }) as UINT;
    return fOpen;
}

unsafe extern "C" fn Imm32TransSetOpenK(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT) -> LRESULT {
    let hKL: HKL = GetKeyboardLayout(0);
    return ImmEscapeW(hKL, hIMC, IME_SETOPEN, (pIme).cast());
}

unsafe extern "C" fn Imm32TransSetOpenJ(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT) -> BOOL {
    let fOldOpen: LRESULT = ImmGetOpenStatus(hIMC) as LRESULT;

    if Imm32IsForegroundThread(core::ptr::null_mut()) != 0 || !GetFocus().is_null() {
        ImmSetOpenStatus(hIMC, (*pIme).wParam as BOOL);
        return fOldOpen as BOOL;
    }

    let pIC: PINPUTCONTEXT = ImmLockIMC(hIMC) as PINPUTCONTEXT;
    if pIC.is_null() {
        return fOldOpen as BOOL;
    }

    let bRequestOpen: BOOL = ((*pIme).wParam != 0) as BOOL;
    if (*pIC).fOpen != bRequestOpen {
        (*pIC).fOpen = bRequestOpen;
        ImmNotifyIME(hIMC, NI_CONTEXTUPDATED, 0, (IMC_SETOPENSTATUS) as DWORD_PTR);
    }

    ImmUnlockIMC(hIMC);
    return fOldOpen as BOOL;
}

unsafe extern "C" fn Imm32TransConvertList(hIMC: HIMC, pIme: PIMESTRUCT) -> UINT {
    let cbIme: SIZE_T = GlobalSize(GlobalHandle(pIme as LPCVOID)) as SIZE_T;
    if ((*pIme).dchDest as SIZE_T) >= cbIme {
        return 0;
    }

    let hKL: HKL = GetKeyboardLayout(0);
    let pszSource: *const CHAR = (pIme as *const CHAR).offset((*pIme).dchSource as isize);
    let uBufLen: UINT = ImmGetConversionListA(
        hKL,
        hIMC,
        pszSource as LPCSTR,
        core::ptr::null_mut(),
        0,
        GCL_CONVERSION,
    ) as UINT;
    if uBufLen == 0 {
        return 0;
    }

    let hCandList: HGLOBAL = GlobalAlloc(GHND as UINT, uBufLen as SIZE_T);
    if hCandList.is_null() {
        return 0;
    }

    let pCL: PCANDIDATELIST = GlobalLock(hCandList) as PCANDIDATELIST;
    if pCL.is_null() {
        GlobalFree(hCandList);
        return 0;
    }

    let ret: UINT = ImmGetConversionListA(
        hKL,
        hIMC,
        pszSource as LPCSTR,
        pCL as LPCANDIDATELIST,
        uBufLen as DWORD,
        GCL_CONVERSION,
    ) as UINT;

    let mut wCount: UINT = 0;
    let mut pszDest: PCHAR = (pIme as PCHAR).offset((*pIme).dchDest as isize);
    let mut cbDestRemaining: SIZE_T = cbIme.wrapping_sub((*pIme).dchDest as SIZE_T);
    let mut i: DWORD = 0;
    while i < (*pCL).dwCount && cbDestRemaining > (2 as SIZE_T) {
        let pCandidate: *const CHAR = (pCL as *const CHAR)
            .offset(*(core::ptr::addr_of!((*pCL).dwOffset) as *const DWORD).add(i as usize) as isize);
        *pszDest = *pCandidate.offset(0 as isize);
        pszDest = pszDest.offset(1);
        *pszDest = *pCandidate.offset(1 as isize);
        pszDest = pszDest.offset(1);

        cbDestRemaining = cbDestRemaining.wrapping_sub(2 as SIZE_T);
        wCount = wCount.wrapping_add(2 as UINT);
        i = i.wrapping_add(1);
    }
    (*pIme).wCount = wCount;

    if cbDestRemaining > (0 as SIZE_T) {
        *pszDest = ANSI_NULL as CHAR;
    }

    GlobalUnlock(hCandList);
    GlobalFree(hCandList);
    return ret;
}

unsafe extern "C" fn Imm32TransHanjaMode(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT) -> LRESULT {
    let dwThreadId: DWORD = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD;
    if dwThreadId == 0 {
        return 0;
    }

    let mut hKL: HKL = GetKeyboardLayout(dwThreadId);
    let pImeDpi: PIMEDPI = ImmLockImeDpi(hKL);
    if pImeDpi.is_null() {
        return 0;
    }

    if ImeDpi_IsUnicode!(pImeDpi) != false {
        let offset: BYTE = HIBYTE!(HIWORD!((*pIme).wParam));
        let cbIme: SIZE_T = (core::mem::size_of::<tagIMESTRUCT>() as SIZE_T)
            .wrapping_add((*pIme).wCount as SIZE_T);
        let mut pMbcsCh: PCHAR = core::ptr::null_mut::<CHAR>();
        let mut MbcsCh: WORD = 0;

        /* SECURITY: Check boundary */
        if (offset as SIZE_T) > cbIme
            || cbIme - (offset as SIZE_T) < core::mem::size_of::<WORD>() as SIZE_T
        {
            ImmUnlockImeDpi(pImeDpi);
            return 0;
        }

        pMbcsCh = (pIme as PCHAR).add(offset as usize);
        RtlCopyMemory!(
            core::ptr::addr_of_mut!(MbcsCh) as *mut core::ffi::c_void,
            pMbcsCh as *const core::ffi::c_void,
            core::mem::size_of::<WORD>() as SIZE_T,
        );
        (*pIme).wParam = MAKELONG!(MbcsCh, HIWORD!((*pIme).wParam)) as WPARAM;

        let mut wch: WCHAR = UNICODE_NULL;
        if MultiByteToWideChar(
            CP_ACP,
            0,
            pMbcsCh as LPCSTR,
            2,
            core::ptr::addr_of_mut!(wch),
            1,
        ) == 0
        {
            ImmUnlockImeDpi(pImeDpi);
            return 0;
        }

        RtlCopyMemory!(
            pMbcsCh as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(wch) as *const core::ffi::c_void,
            core::mem::size_of::<WCHAR>() as SIZE_T,
        );
    }

    ImmUnlockImeDpi(pImeDpi);

    hKL = GetKeyboardLayout(0);
    let ret: LRESULT = ImmEscapeW(hKL, hIMC, IME_ESC_HANJA_MODE, pIme as LPVOID);
    if ret != 0 {
        SendMessageW(
            hWnd,
            WM_IME_NOTIFY,
            (IMN_OPENCANDIDATE) as WPARAM,
            1,
        );
    }

    return ret;
}

unsafe extern "C" fn Imm32TransGetLevel(hWnd: HWND) -> DWORD {
    let result: DWORD = NtUserGetAppImeLevel(hWnd);
    return if result != 0 { result } else { IME_RS_ERROR };
}

unsafe extern "C" fn Imm32TransSetLevel(hWnd: HWND, pIme: PIMESTRUCT) -> BOOL {
    let wParam: WPARAM = (*pIme).wParam;
    return (wParam != 0 && wParam <= 5 && NtUserSetAppImeLevel(hWnd, wParam as DWORD) != 0) as BOOL;
}

unsafe extern "C" fn Imm32TransGetMNTable(hIMC: HIMC, pIme: PIMESTRUCT) -> BOOL {
    static g_MNTable: [[WORD; 96]; 3] = [
        [
            0xA1A1, 0xA3A1, 0xA1A8, 0xA3A3, 0xA3A4, 0xA3A5, 0xA3A6, 0xA1AE,
            0xA3A8, 0xA3A9, 0xA3AA, 0xA3AB, 0xA3A7, 0xA3AD, 0xA3AE, 0xA3AF,
            0xA3B0, 0xA3B1, 0xA3B2, 0xA3B3, 0xA3B4, 0xA3B5, 0xA3B6, 0xA3B7,
            0xA3B8, 0xA3B9, 0xA3BA, 0xA3BB, 0xA3BC, 0xA3BD, 0xA3BE, 0xA3BF,
            0xA3C0, 0xA4B1, 0xA4D0, 0xA4BA, 0xA4B7, 0xA4A8, 0xA4A9, 0xA4BE,
            0xA4C7, 0xA4C1, 0xA4C3, 0xA4BF, 0xA4D3, 0xA4D1, 0xA4CC, 0xA4C2,
            0xA4C6, 0xA4B3, 0xA4A2, 0xA4A4, 0xA4B6, 0xA4C5, 0xA4BD, 0xA4B9,
            0xA4BC, 0xA4CB, 0xA4BB, 0xA3DB, 0xA1AC, 0xA3DD, 0xA3DE, 0xA3DF,
            0xA1A2, 0xA4B1, 0xA4D0, 0xA4BA, 0xA4B7, 0xA4A7, 0xA4A9, 0xA4BE,
            0xA4C7, 0xA4C1, 0xA4C3, 0xA4BF, 0xA4D3, 0xA4D1, 0xA4CC, 0xA4C0,
            0xA4C4, 0xA4B2, 0xA4A1, 0xA4A4, 0xA4B5, 0xA4C5, 0xA4BD, 0xA4B8,
            0xA4BC, 0xA4CB, 0xA4BB, 0xA3FB, 0xA3FC, 0xA3FD, 0xA1AD, 0x0000
        ],
        [
            0xA1A1, 0xA4B8, 0xA1A8, 0xA3A3, 0xA3A4, 0xA3A5, 0xA3A6, 0xA1AE,
            0xA3A8, 0xA3A9, 0xA3AA, 0xA3AB, 0xA4BC, 0xA3AD, 0xA3AE, 0xA4C7,
            0xA4BB, 0xA4BE, 0xA4B6, 0xA4B2, 0xA4CB, 0xA4D0, 0xA4C1, 0xA4C6,
            0xA4D2, 0xA4CC, 0xA3BA, 0xA4B2, 0xA3B2, 0xA3BD, 0xA3B3, 0xA3BF,
            0xA3C0, 0xA4A7, 0xA3A1, 0xA4AB, 0xA4AA, 0xA4BB, 0xA4A2, 0xA3AF,
            0xA1AF, 0xA3B8, 0xA3B4, 0xA3B5, 0xA3B6, 0xA3B1, 0xA3B0, 0xA3B9,
            0xA3BE, 0xA4BD, 0xA4C2, 0xA4A6, 0xA4C3, 0xA3B7, 0xA4B0, 0xA4BC,
            0xA4B4, 0xA3BC, 0xA4BA, 0xA3DB, 0xA3DC, 0xA3DD, 0xA3DE, 0xA3DF,
            0xA1AE, 0xA4B7, 0xA4CC, 0xA4C4, 0xA4D3, 0xA4C5, 0xA4BF, 0xA4D1,
            0xA4A4, 0xA4B1, 0xA4B7, 0xA4A1, 0xA4B8, 0xA4BE, 0xA4B5, 0xA4BA,
            0xA4BD, 0xA4B5, 0xA4C0, 0xA4A4, 0xA4C3, 0xA4A7, 0xA4C7, 0xA4A9,
            0xA4A1, 0xA4A9, 0xA4B1, 0xA3FB, 0xA3FC, 0xA3FD, 0xA1AD, 0x0000
        ],
        [
            0xA1A1, 0xA4A2, 0xA3AE, 0xA4B8, 0xA4AF, 0xA4AE, 0xA1B0, 0xA3AA,
            0xA1A2, 0xA1AD, 0xA1B1, 0xA3AB, 0xA4BC, 0xA3A9, 0xA3AE, 0xA4C7,
            0xA4BB, 0xA4BE, 0xA4B6, 0xA4B2, 0xA4CB, 0xA4D0, 0xA4C1, 0xA4C6,
            0xA4D2, 0xA4CC, 0xA3B4, 0xA4B2, 0xA3A7, 0xA1B5, 0xA3AE, 0xA3A1,
            0xA4AA, 0xA4A7, 0xA3BF, 0xA4BC, 0xA4AC, 0xA4A5, 0xA4AB, 0xA4C2,
            0xA3B0, 0xA3B7, 0xA3B1, 0xA3B2, 0xA3B3, 0xA1A8, 0xA3AD, 0xA3B8,
            0xA3B9, 0xA4BD, 0xA4B0, 0xA4A6, 0xA4AD, 0xA3B6, 0xA4A3, 0xA4BC,
            0xA4B4, 0xA3B5, 0xA4BA, 0xA3A8, 0xA3BA, 0xA1B4, 0xA3BD, 0xA3BB,
            0xA3AA, 0xA4B7, 0xA4CC, 0xA4C4, 0xA4D3, 0xA4C5, 0xA4BF, 0xA4D1,
            0xA4A4, 0xA4B1, 0xA4B7, 0xA4A1, 0xA4B8, 0xA4BE, 0xA4B5, 0xA4BA,
            0xA4BD, 0xA4B5, 0xA4C0, 0xA4A4, 0xA4C3, 0xA4A7, 0xA4C7, 0xA4A9,
            0xA4A1, 0xA4B1, 0xA4B1, 0xA3A5, 0xA3CC, 0xA3AF, 0xA1AD, 0x0000
        ]
    ];

    let mut nInputMethod: UINT = GetProfileIntW(RUSTCONV_WSTR_816B184A1146.as_ptr(), RUSTCONV_WSTR_FB1BE064B20C.as_ptr(), (INPUT_METHOD_BASE) as INT);
    let mut index: UINT = nInputMethod.wrapping_sub(INPUT_METHOD_BASE);
    if index >= _countof!(g_MNTable) {
        index = 0;
    }

    let mut pDst: PWORD = (*pIme).lParam1 as PWORD;
    RtlCopyMemory!(
        pDst as *mut core::ffi::c_void,
        core::ptr::addr_of!(g_MNTable[index as usize]) as *const core::ffi::c_void,
        core::mem::size_of_val(&g_MNTable[0]),
    );
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32TransMoveImeWindow(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT) -> LRESULT {
    if (*pIme).wParam == 2 as WPARAM {
        let mut pt: POINT = POINT {
            x: LOWORD!((*pIme).lParam1) as SHORT as LONG,
            y: HIWORD!((*pIme).lParam1) as SHORT as LONG,
        };
        ClientToScreen(hWnd, core::ptr::addr_of_mut!(pt));
        (*pIme).lParam1 = MAKELONG!(pt.x as WORD, pt.y as WORD) as LPARAM;
    }
    let mut hKL: HKL = GetKeyboardLayout(0 as DWORD);
    return ImmEscapeW(hKL, hIMC, IME_SETCONVERSIONWINDOW as UINT, pIme as LPVOID);
}

unsafe extern "C" fn Imm32SetCompFont(mut hWnd: HWND, mut hIMC: HIMC, mut plfW: PLOGFONTW) -> BOOL {
    let mut pIMC: PINPUTCONTEXT = ImmLockIMC(hIMC) as PINPUTCONTEXT;
    if pIMC.is_null() {
        return (FALSE) as BOOL;
    }

    let mut dwCompatFlags: DWORD = ImmGetAppCompatFlags(hIMC);
    let mut pClientImc: PCLIENTIMC = ImmLockClientImc(hIMC);
    if pClientImc.is_null() {
        ImmUnlockIMC(hIMC);
        return (FALSE) as BOOL;
    }

    let mut bIsUnicode: BOOL = ((*pClientImc).dwFlags & CLIENTIMC_WIDE as DWORD) as BOOL;
    ImmUnlockClientImc(pClientImc);

    let mut pCurrentFontW: *const LOGFONTW = core::ptr::null();
    let mut currentLF: LOGFONTW = core::mem::zeroed();
    if bIsUnicode != 0 {
        pCurrentFontW = core::ptr::addr_of_mut!((*pIMC).lfFont.W);
    } else {
        LogFontAnsiToWide(
            core::ptr::addr_of_mut!((*pIMC).lfFont.A),
            core::ptr::addr_of_mut!(currentLF),
        );
        pCurrentFontW = core::ptr::addr_of!(currentLF);
    }

    let mut ret: BOOL;
    if memcmp(
        pCurrentFontW as *const core::ffi::c_void,
        plfW as *const core::ffi::c_void,
        core::mem::offset_of!(LOGFONTW, lfFaceName) as usize,
    ) != 0
        || lstrcmpW(
            core::ptr::addr_of!((*pCurrentFontW).lfFaceName) as LPCWSTR,
            core::ptr::addr_of_mut!((*plfW).lfFaceName) as *mut WCHAR as LPCWSTR,
        ) != 0
    {
        if (dwCompatFlags & _IME_APP_COMPAT_DIRECT_IME_SYSTEM) != 0 {
            (*pIMC).lfFont.W = *plfW;

            if (dwCompatFlags & _IME_APP_COMPAT_DIRECT_IME_FONT) != 0 {
                ret = Imm32PostImsMessage(hWnd, (IMC_SETCOMPOSITIONFONT) as WPARAM, 0);
            } else {
                ret = PostMessageW(
                    hWnd,
                    WM_IME_SYSTEM,
                    (IMC_SETCOMPOSITIONFONT) as WPARAM,
                    0,
                );
            }
        } else {
            ret = ImmSetCompositionFontW(hIMC, plfW);
        }
    } else {
        ret = (TRUE) as i32;
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

unsafe extern "C" fn Imm32FixLogFont(plfW: PLOGFONTW, bVertical: BOOL) -> BOOL {
    if plfW.is_null() {
        return (FALSE) as BOOL;
    }

    /* SECURITY: Avoid buffer overrun */
    (*plfW).lfFaceName[(_countof!((*plfW).lfFaceName) - 1) as usize] = UNICODE_NULL;

    if bVertical != 0 {
        (*plfW).lfOrientation = 2700;
        (*plfW).lfEscapement = (*plfW).lfOrientation;

        if (*plfW).lfCharSet as ::core::ffi::c_int == (SHIFTJIS_CHARSET as i32)
            && (*plfW).lfFaceName[0 as usize] != 0x0040u16
        {
            let mut len: usize = wcslen(core::ptr::addr_of_mut!((*plfW).lfFaceName) as *mut WCHAR) as usize;
            if len >= (_countof!((*plfW).lfFaceName) - 1) as usize {
                return (FALSE) as BOOL;
            }

            // RtlMoveMemory(&lfFaceName[1], &lfFaceName[0], (len + 1) * sizeof(WCHAR)): DEST
            // first, a BYTE count. core::ptr::copy is (src, dst, ELEMENTS); the translation had
            // both the direction and the unit wrong, moving 2x the text the wrong way.
            core::ptr::copy(
                (core::ptr::addr_of_mut!((*plfW).lfFaceName) as *mut WCHAR).add(0 as usize),
                (core::ptr::addr_of_mut!((*plfW).lfFaceName) as *mut WCHAR).add(1 as usize),
                len + 1 as usize,
            );
            (*plfW).lfFaceName[0 as usize] = 0x0040u16;
        }
    } else {
        (*plfW).lfOrientation = 0;
        (*plfW).lfEscapement = (*plfW).lfOrientation;

        if (*plfW).lfCharSet as ::core::ffi::c_int == (SHIFTJIS_CHARSET as i32)
            && (*plfW).lfFaceName[0 as usize] == 0x0040u16
        {
            let mut len: usize = wcslen(core::ptr::addr_of_mut!((*plfW).lfFaceName) as *mut WCHAR) as usize;
            // RtlMoveMemory(&lfFaceName[0], &lfFaceName[1], len * sizeof(WCHAR)): see above.
            core::ptr::copy(
                (core::ptr::addr_of_mut!((*plfW).lfFaceName) as *mut WCHAR).add(1 as usize),
                (core::ptr::addr_of_mut!((*plfW).lfFaceName) as *mut WCHAR).add(0 as usize),
                len,
            );
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32SetFontForVertical(hWnd: HWND, hIMC: HIMC, pIC: PINPUTCONTEXTDX, bVertical: BOOL) -> BOOL {
    let mut lfW: LOGFONTW = core::mem::zeroed();
    if (*pIC).__bindgen_anon_1.fdwInit & INIT_LOGFONT != 0 {
        let pClientImc: PCLIENTIMC = ImmLockClientImc(hIMC);
        if pClientImc.is_null() {
            return (FALSE) as BOOL;
        }
        let bWide: BOOL = (((*pClientImc).dwFlags & CLIENTIMC_WIDE) != 0) as BOOL;
        ImmUnlockClientImc(pClientImc);

        if bWide != 0 {
            lfW = (*pIC).__bindgen_anon_1.lfFont.W;
        } else {
            LogFontAnsiToWide(core::ptr::addr_of_mut!((*pIC).__bindgen_anon_1.lfFont.A), core::ptr::addr_of_mut!(lfW));
        }
    } else {
        GetObjectW(
            GetStockObject((SYSTEM_FONT as i32)) as HANDLE,
            core::mem::size_of::<LOGFONTW>() as i32,
            core::ptr::addr_of_mut!(lfW) as LPVOID,
        );
    }

    if Imm32FixLogFont(core::ptr::addr_of_mut!(lfW), bVertical) == 0 {
        return (FALSE) as BOOL;
    }

    return Imm32SetCompFont(hWnd, hIMC, core::ptr::addr_of_mut!(lfW));
}

unsafe extern "C" fn Imm32TransSetConversionWindow(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT) -> BOOL {
    if Imm32IsForegroundThread(core::ptr::null_mut()) == 0 && GetFocus().is_null() {
        return (TRUE) as BOOL;
    }

    let pIC: PINPUTCONTEXTDX = ImmLockIMC(hIMC) as PINPUTCONTEXTDX;
    if pIC.is_null() {
        return (FALSE) as BOOL;
    }

    let mut cfComp: COMPOSITIONFORM = COMPOSITIONFORM {
        dwStyle: 0,
        ptCurrentPos: POINT { x: 0, y: 0 },
        rcArea: RECT { left: 0, top: 0, right: 0, bottom: 0 },
    };
    cfComp.dwStyle = 0 as DWORD;

    let mut pt: POINT = POINT {
        x: LOWORD!((*pIme).lParam1) as SHORT as LONG,
        y: HIWORD!((*pIme).lParam1) as SHORT as LONG,
    };
    let mut rcArea: RECT = RECT {
        left: LOWORD!((*pIme).lParam2) as SHORT as LONG,
        top: HIWORD!((*pIme).lParam2) as SHORT as LONG,
        right: LOWORD!((*pIme).lParam3) as SHORT as LONG,
        bottom: HIWORD!((*pIme).lParam3) as SHORT as LONG,
    };

    let wParam: WPARAM = (*pIme).wParam;
    let hwndIC: HWND = (*pIC).__bindgen_anon_1.hWnd;

    if (wParam & (MCW_HIDDEN as WPARAM)) != 0 {
        (*pIC).dwUIFlags |= _IME_UI_HIDDEN as DWORD;
        ScreenToClient(hwndIC, core::ptr::addr_of_mut!(pt));
        MapWindowPoints(
            core::ptr::null_mut(),
            hwndIC,
            core::ptr::addr_of_mut!(rcArea) as PPOINT,
            2 as UINT,
        );
    } else {
        (*pIC).dwUIFlags &= !(_IME_UI_HIDDEN as DWORD);
    }

    if (wParam & (MCW_WINDOW as WPARAM)) != 0 {
        let mut bUseCrossWindowLogic: BOOL = (TRUE) as i32;

        if LOWORD!(hWnd) == LOWORD!(hwndIC) {
            let pWndEntry: PVOID = if !hWnd.is_null() && !gpsi.is_null() {
                ValidateHwnd!((hWnd).cast())
            } else {
                core::ptr::null_mut()
            };
            let pWndICEntry: PVOID = if !hwndIC.is_null() && !gpsi.is_null() {
                ValidateHwnd!((hwndIC).cast())
            } else {
                core::ptr::null_mut()
            };

            if pWndEntry == pWndICEntry {
                bUseCrossWindowLogic = (FALSE) as i32;
            }
        }

        if bUseCrossWindowLogic != 0 {
            ClientToScreen(hWnd, core::ptr::addr_of_mut!(pt));
            ScreenToClient(hwndIC, core::ptr::addr_of_mut!(pt));

            if (wParam & (MCW_RECT as WPARAM)) != 0 {
                cfComp.dwStyle = CFS_RECT as DWORD;
                MapWindowPoints(
                    hWnd,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(rcArea) as PPOINT,
                    2 as UINT,
                );
                MapWindowPoints(
                    core::ptr::null_mut(),
                    hwndIC,
                    core::ptr::addr_of_mut!(rcArea) as PPOINT,
                    2 as UINT,
                );
            } else {
                cfComp.dwStyle = CFS_POINT as DWORD;
            }
        } else {
            if (wParam & (MCW_SCREEN as WPARAM)) != 0 {
                ScreenToClient(hwndIC, core::ptr::addr_of_mut!(pt));
                cfComp.dwStyle = (if (wParam & (MCW_RECT as WPARAM)) != 0 {
                    CFS_RECT
                } else {
                    CFS_POINT
                }) as DWORD;
                if (wParam & (MCW_RECT as WPARAM)) != 0 {
                    MapWindowPoints(
                        core::ptr::null_mut(),
                        hwndIC,
                        core::ptr::addr_of_mut!(rcArea) as PPOINT,
                        2 as UINT,
                    );
                }
            } else {
                cfComp.dwStyle = (if (wParam & (MCW_RECT as WPARAM)) != 0 {
                    CFS_RECT
                } else {
                    CFS_POINT
                }) as DWORD;
            }
        }
    }

    if (wParam & (MCW_VERTICAL as WPARAM)) != 0 {
        if ((*pIC).dwUIFlags & (_IME_UI_VERTICAL as DWORD)) == 0 {
            (*pIC).dwUIFlags |= _IME_UI_VERTICAL as DWORD;
            Imm32SetFontForVertical(hWnd, hIMC, pIC, (TRUE) as BOOL);
        }
    } else {
        if ((*pIC).dwUIFlags & (_IME_UI_VERTICAL as DWORD)) != 0 {
            (*pIC).dwUIFlags &= !(_IME_UI_VERTICAL as DWORD);
            Imm32SetFontForVertical(hWnd, hIMC, pIC, (FALSE) as BOOL);
        }
    }

    cfComp.ptCurrentPos = pt;
    cfComp.rcArea = rcArea;

    if ((*pIC).dwUIFlags & (_IME_UI_HIDDEN as DWORD)) != 0 {
        (*pIC).__bindgen_anon_1.cfCompForm.ptCurrentPos = pt;
        (*pIC).__bindgen_anon_1.cfCompForm.rcArea = rcArea;

        let mut i: DWORD = 0 as DWORD;
        while i < (MAX_CANDIDATEFORM as DWORD) {
            let pCF: PCANDIDATEFORM = core::ptr::addr_of_mut!((*pIC).__bindgen_anon_1.cfCandForm[i as usize]);
            if (*pCF).dwIndex == (1 as DWORD).wrapping_neg() {
                // continue: fall through to increment below
            } else {
                let mut cf: CANDIDATEFORM = CANDIDATEFORM {
                    dwIndex: 0,
                    dwStyle: 0,
                    ptCurrentPos: POINT { x: 0, y: 0 },
                    rcArea: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                };
                cf.dwIndex = i;
                cf.dwStyle = CFS_EXCLUDE as DWORD;
                cf.ptCurrentPos = pt;
                cf.rcArea = rcArea;
                Imm32SetCandidateWindow(hWnd, hIMC, core::ptr::addr_of_mut!(cf));
            }
            i = i.wrapping_add(1);
        }
    } else {
        Imm32SetCompWindow(hWnd, hIMC, core::ptr::addr_of_mut!(cfComp));
    }

    ImmUnlockIMC(hIMC);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32TransGetConversionMode(hIMC: HIMC) -> DWORD {
    let mut fdwConversion: DWORD = 0;
    let mut fdwSentence: DWORD = 0;
    ImmGetConversionStatus(hIMC, core::ptr::addr_of_mut!(fdwConversion), core::ptr::addr_of_mut!(fdwSentence));
    return Imm32Get31ModeFrom40ModeJ(fdwConversion);
}

unsafe extern "C" fn Imm32TransVKDBEMode(hIMC: HIMC, vKey: UINT) -> BOOL {
    let mut fdwConversion: DWORD = 0;
    let mut fdwSentence: DWORD = 0;
    let ret: BOOL = ImmGetConversionStatus(hIMC, core::ptr::addr_of_mut!(fdwConversion), core::ptr::addr_of_mut!(fdwSentence));
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    match vKey {
        VK_DBE_ALPHANUMERIC => {
            fdwConversion &= !(IME_CMODE_LANGUAGE as DWORD);
        }
        VK_DBE_KATAKANA => {
            fdwConversion |= IME_CMODE_LANGUAGE as DWORD;
        }
        VK_DBE_HIRAGANA => {
            fdwConversion = (fdwConversion & !(IME_CMODE_LANGUAGE as DWORD)) | (IME_CMODE_NATIVE as DWORD);
        }
        VK_DBE_SBCSCHAR => {
            fdwConversion &= !(IME_CMODE_FULLSHAPE as DWORD);
        }
        VK_DBE_DBCSCHAR => {
            fdwConversion |= IME_CMODE_FULLSHAPE as DWORD;
        }
        VK_DBE_ROMAN => {
            fdwConversion |= IME_CMODE_ROMAN as DWORD;
        }
        VK_DBE_NOROMAN => {
            fdwConversion &= !(IME_CMODE_ROMAN as DWORD);
        }
        VK_DBE_CODEINPUT => {
            fdwConversion |= IME_CMODE_CHARCODE as DWORD;
        }
        VK_DBE_NOCODEINPUT => {
            fdwConversion &= !(IME_CMODE_CHARCODE as DWORD);
        }
        _ => {}
    }

    return ImmSetConversionStatus(hIMC, fdwConversion, fdwSentence);
}

unsafe extern "C" fn Imm32TransSendVKey(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT, bAnsi: BOOL) -> BOOL {
    let wParam: WPARAM = (*pIme).wParam;

    if wParam >= (VK_DBE_DETERMINESTRING as WPARAM) {
        if wParam == (VK_DBE_DETERMINESTRING as WPARAM) {
            let hContext: HIMC = ImmGetContext(hWnd);
            let ret: BOOL = ImmNotifyIME(hContext, NI_COMPOSITIONSTR, CPS_COMPLETE, 0) as BOOL;
            ImmReleaseContext(hWnd, hContext);
            return ret;
        }

        if wParam > (VK_DBE_ENTERDLGCONVERSIONMODE as WPARAM) && (wParam == 0xFFFF || (wParam as LONG) == -1) {
            let wCount: UINT = (*pIme).wCount;
            if wCount == 28 || (240 <= wCount && wCount <= 253) {
                return (TRUE) as BOOL;
            }
        }

        return (FALSE) as BOOL;
    }

    if wParam >= (VK_DBE_CODEINPUT as WPARAM) {
        return Imm32TransVKDBEMode(hIMC, wParam as UINT);
    }

    if wParam == (VK_CONVERT) as _ {
        return ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_CONVERT, 0);
    }

    if wParam < (VK_DBE_ALPHANUMERIC as WPARAM) {
        return (FALSE) as BOOL;
    }

    if wParam <= (VK_DBE_NOROMAN as WPARAM) {
        return Imm32TransVKDBEMode(hIMC, wParam as UINT);
    }

    let hKL: HKL;
    let nImeConfig: UINT;
    if wParam == (VK_DBE_ENTERWORDREGISTERMODE as WPARAM) {
        hKL = GetKeyboardLayout(0);
        nImeConfig = IME_CONFIG_REGISTERWORD;
    } else if wParam == (VK_DBE_ENTERIMECONFIGMODE as WPARAM) {
        hKL = GetKeyboardLayout(0);
        nImeConfig = IME_CONFIG_GENERAL;
    } else {
        return ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_CANCEL, 0);
    }

    if bAnsi != 0 {
        return ImmConfigureIMEA(hKL, hWnd, nImeConfig, core::ptr::null_mut());
    }
    return ImmConfigureIMEW(hKL, hWnd, nImeConfig, core::ptr::null_mut());
}

unsafe extern "C" fn Imm32TransSetConversionMode(hIMC: HIMC, pIme: PIMESTRUCT) -> DWORD {
    let mut fdwOldConversion: DWORD = 0 as DWORD;
    let mut fdwSentence: DWORD = core::mem::zeroed();
    if ImmGetConversionStatus(
        hIMC,
        core::ptr::addr_of_mut!(fdwOldConversion),
        core::ptr::addr_of_mut!(fdwSentence),
    ) == 0
    {
        return 0 as DWORD;
    }

    let mode31: DWORD = Imm32Get31ModeFrom40ModeJ(fdwOldConversion);
    let wParam: WPARAM = (*pIme).wParam;
    let mut dwNewValues: DWORD = 0 as DWORD;
    let mut dwMask: DWORD = 0 as DWORD;

    let langFlags: DWORD = (IME_MODE_ALPHANUMERIC as DWORD)
        | (IME_MODE_KATAKANA as DWORD)
        | (IME_MODE_HIRAGANA as DWORD);
    if (wParam & (langFlags as WPARAM)) != 0 {
        dwMask |= IME_CMODE_LANGUAGE as DWORD;
        if (wParam & (IME_MODE_KATAKANA as WPARAM)) != 0 {
            dwNewValues |= IME_CMODE_LANGUAGE as DWORD;
        } else if (wParam & (IME_MODE_HIRAGANA as WPARAM)) != 0 {
            dwNewValues |= IME_CMODE_NATIVE as DWORD;
        } else {
            dwNewValues |= IME_CMODE_ALPHANUMERIC as DWORD;
        }
    }

    if (wParam & ((IME_MODE_DBCSCHAR as WPARAM) | (_IME_MODE_JPN_SBCSCHAR as WPARAM))) != 0 {
        dwMask |= IME_CMODE_FULLSHAPE as DWORD;
        if (wParam & (_IME_MODE_JPN_SBCSCHAR as WPARAM)) == 0 {
            dwNewValues |= IME_CMODE_FULLSHAPE as DWORD;
        }
    }

    if (wParam & ((IME_MODE_ROMAN as WPARAM) | (IME_MODE_NOROMAN as WPARAM))) != 0 {
        dwMask |= IME_CMODE_ROMAN as DWORD;
        if (wParam & (IME_MODE_ROMAN as WPARAM)) != 0 {
            dwNewValues |= IME_CMODE_ROMAN as DWORD;
        }
    }

    if (wParam & ((IME_MODE_CODEINPUT as WPARAM) | (IME_MODE_NOCODEINPUT as WPARAM))) != 0 {
        dwMask |= IME_CMODE_CHARCODE as DWORD;
        if (wParam & (IME_MODE_CODEINPUT as WPARAM)) != 0 {
            dwNewValues |= IME_CMODE_CHARCODE as DWORD;
        }
    }

    let fdwNewConversion: DWORD = (fdwOldConversion & !dwMask) | (dwNewValues & dwMask);

    if ImmSetConversionStatus(hIMC, fdwNewConversion, fdwSentence) == 0 {
        return 0 as DWORD;
    }

    return mode31;
}

unsafe extern "C" fn Imm32TransEnterWordRegisterMode(hWnd: HWND, pIme: PIMESTRUCT, bAnsi: BOOL) -> BOOL {
    let hKL: HKL = GetKeyboardLayout(0);
    if ImmIsIME(hKL) == 0 {
        return (FALSE) as BOOL;
    }

    let hReading: HGLOBAL = (*pIme).lParam1 as HGLOBAL;
    let hWord: HGLOBAL = (*pIme).lParam2 as HGLOBAL;
    let mut rwInfo: REGISTERWORDW = REGISTERWORDW {
        lpReading: core::ptr::null_mut(),
        lpWord: core::ptr::null_mut(),
    };
    if !hReading.is_null() {
        rwInfo.lpReading = GlobalLock(hReading) as LPWSTR;
    }
    if !hWord.is_null() {
        rwInfo.lpWord = GlobalLock(hWord) as LPWSTR;
    }

    let mut ret: BOOL = 0;
    if bAnsi != 0 {
        ret = ImmConfigureIMEA(
            hKL,
            hWnd,
            IME_CONFIG_REGISTERWORD as DWORD,
            core::ptr::addr_of_mut!(rwInfo) as PREGISTERWORDA as LPVOID,
        );
    } else {
        ret = ImmConfigureIMEW(
            hKL,
            hWnd,
            IME_CONFIG_REGISTERWORD as DWORD,
            core::ptr::addr_of_mut!(rwInfo) as LPVOID,
        );
    }

    if !rwInfo.lpReading.is_null() {
        GlobalUnlock(hReading);
    }
    if !rwInfo.lpWord.is_null() {
        GlobalUnlock(hWord);
    }

    return ret;
}

unsafe extern "C" fn Imm32TransSetConversionFontEx(hWnd: HWND, hIMC: HIMC, pIme: PIMESTRUCT, bAnsi: BOOL) -> BOOL {
    let pIC: PINPUTCONTEXTDX = ImmLockIMC(hIMC) as PINPUTCONTEXTDX;
    if pIC.is_null() {
        return (FALSE) as BOOL;
    }

    let pLogFont: PVOID = GlobalLock((*pIme).lParam1 as HGLOBAL) as PVOID;
    if pLogFont.is_null() {
        ImmUnlockIMC(hIMC);
        return (FALSE) as BOOL;
    }

    let mut lfW: LOGFONTW = core::mem::zeroed();
    if bAnsi != 0 {
        LogFontAnsiToWide(pLogFont as *const LOGFONTA, core::ptr::addr_of_mut!(lfW));
    } else {
        RtlCopyMemory!(
            core::ptr::addr_of_mut!(lfW) as *mut core::ffi::c_void,
            pLogFont as *const core::ffi::c_void,
            (core::mem::size_of::<LOGFONTW>()) as usize,
        );
    }

    GlobalUnlock((*pIme).lParam1 as HGLOBAL);

    let bFixed: BOOL = Imm32FixLogFont(
        core::ptr::addr_of_mut!(lfW),
        ((((*pIC).dwUIFlags & _IME_UI_VERTICAL) != 0) as BOOL),
    );
    ImmUnlockIMC(hIMC);

    if bFixed == 0 {
        return (FALSE) as BOOL;
    }
    return Imm32SetCompFont(hWnd, hIMC, core::ptr::addr_of_mut!(lfW));
}

unsafe extern "C" fn Imm32TranslateIMESubFunctions(hWnd: HWND, pIme: PIMESTRUCT, bAnsi: BOOL) -> LRESULT {
    let hImc: HIMC = ImmGetSaveContext(hWnd, 1 as DWORD);
    if hImc.is_null() {
        return 0 as LRESULT;
    }
    let wLang: WORD = PRIMARYLANGID!(LANGIDFROMLCID!(GetSystemDefaultLCID()));
    let bKorean: BOOL = ((wLang) as u32 == LANG_KOREAN) as BOOL;
    let bJapanese: BOOL = ((wLang) as u32 == LANG_JAPANESE) as BOOL;
    let fnc: UINT = (*pIme).fnc;
    match fnc {
        3 => {
            return 1 as LRESULT;
        }
        4 => {
            if bKorean != 0 {
                return Imm32TransSetOpenK(hWnd, hImc, pIme);
            }
            return Imm32TransSetOpenJ(hWnd, hImc, pIme) as LRESULT;
        }
        5 => {
            if bKorean != 0 {
                return Imm32TransGetOpenK(hWnd, hImc, pIme, bAnsi);
            }
            return Imm32TransGetOpenJ(hWnd, hImc, pIme, bAnsi) as LRESULT;
        }
        6 => {
            return 0 as LRESULT;
        }
        7 => {
            return 0xA03 as LRESULT;
        }
        8 => {
            if bKorean != 0 {
                return Imm32TransMoveImeWindow(hWnd, hImc, pIme);
            }
            return Imm32TransSetConversionWindow(hWnd, hImc, pIme) as LRESULT;
        }
        16 => {
            if bJapanese != 0 {
                return Imm32TransSetConversionMode(hImc, pIme) as LRESULT;
            }
        }
        17 => {
            if bKorean != 0 {
                return Imm32TransGetMode(hImc);
            }
            if bJapanese != 0 {
                return Imm32TransGetConversionMode(hImc) as LRESULT;
            }
        }
        18 => {
            if bKorean != 0 {
                return Imm32TransSetMode(hImc, pIme) as LRESULT;
            }
        }
        19 => {
            if bJapanese != 0 {
                return Imm32TransSendVKey(hWnd, hImc, pIme, bAnsi) as LRESULT;
            }
        }
        24 => {
            if bJapanese != 0 {
                return Imm32TransEnterWordRegisterMode(hWnd, pIme, bAnsi) as LRESULT;
            }
        }
        25 => {
            if bJapanese != 0 {
                return Imm32TransSetConversionFontEx(hWnd, hImc, pIme, bAnsi) as LRESULT;
            }
        }
        32 => {
            if bKorean != 0 {
                return (if Imm32TransCodeConvert(hImc, pIme) != 0 {
                    (*pIme).wParam
                } else {
                    0 as WPARAM
                }) as LRESULT;
            }
        }
        34 => {
            if bKorean != 0 {
                return Imm32TransConvertList(hImc, pIme) as LRESULT;
            }
        }
        48 => {
            if bKorean != 0 {
                return ImmEscapeW(
                    GetKeyboardLayout(0 as DWORD),
                    hImc,
                    0x30 as UINT,
                    pIme as LPVOID,
                );
            }
        }
        49 => {
            if bKorean != 0 {
                return Imm32TransHanjaMode(hWnd, hImc, pIme);
            }
        }
        64 => {
            if bKorean != 0 {
                return Imm32TransGetLevel(hWnd) as LRESULT;
            }
        }
        65 => {
            if bKorean != 0 {
                return Imm32TransSetLevel(hWnd, pIme) as LRESULT;
            }
        }
        66 => {
            if bKorean != 0 {
                return Imm32TransGetMNTable(hImc, pIme) as LRESULT;
            }
        }
        _ => {
            // ERR("Unknown fnc %d\n", fnc);
            let _ = fnc;
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn Imm32SendIMEMessageExAW(hWnd: HWND, hIME: HGLOBAL, bAnsi: BOOL) -> LRESULT {
    let mut pIme: PIMESTRUCT = GlobalLock(hIME) as PIMESTRUCT;
    if pIme.is_null() {
        return 0;
    }

    let mut ret: LRESULT = 0;
    let mut hKL: HKL = GetKeyboardLayout(0);
    if Imm32IsSystemJapaneseOrKorean() != 0 && ImmIsIME(hKL) != 0 {
        let mut hwndIme: HWND = ImmGetDefaultIMEWnd(hWnd);
        if IsWindow(hwndIme) != 0 {
            let mut hTargetWnd: HWND = if IsWindow(hWnd) != 0 {
                hWnd
            } else {
                GetFocus()
            };
            ret = Imm32TranslateIMESubFunctions(hTargetWnd, pIme, bAnsi);
        } else {
            ret = if (*pIme).fnc == IME_GETVERSION {
                IME_RS_INVALID as LRESULT
            } else {
                IME_RS_ERROR as LRESULT
            };
        }
    } else {
        (*pIme).wParam = (IME_RS_INVALID as WPARAM);
        ret = 0;
    }

    GlobalUnlock(hIME);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSendIMEMessageExA(hWnd: HWND, lParam: LPARAM) -> LRESULT {
    if false {
        return 0;
    }
    return Imm32SendIMEMessageExAW(hWnd, lParam as HGLOBAL, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmSendIMEMessageExW(hWnd: HWND, lParam: LPARAM) -> LRESULT {
    if cfg!(imm_win3_support) {
        return Imm32SendIMEMessageExAW(hWnd, lParam as HGLOBAL, (FALSE) as BOOL);
    } else {
        return 0;
    }
}
