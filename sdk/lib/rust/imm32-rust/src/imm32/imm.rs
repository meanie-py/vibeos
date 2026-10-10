use crate::imm32::ctf::CtfImeSetActiveContextAlways;
use crate::imm32::ctf::CtfImmIsTextFrameServiceDisabled;
use crate::imm32::ctf::CtfImmTIMActivate;
use crate::imm32::ctf::CtfImmTIMCreateInputContext;
use crate::imm32::ctf::CtfImmTIMDestroyInputContext;
use crate::imm32::ime::Imm32FindOrLoadImeDpi;
use crate::imm32::ime::Imm32ReleaseIME;
use crate::imm32::ime::ImmGetDefaultIMEWnd;
use crate::imm32::ime::ImmLoadIME;
use crate::imm32::ime::ImmLockImeDpi;
use crate::imm32::ime::ImmNotifyIME;
use crate::imm32::ime::ImmUnlockImeDpi;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_230B3B8A6763: [u16; 9] = [0x0030u16, 0x0030, 0x0030, 0x0030, 0x0030, 0x0034, 0x0030, 0x0039, 0x0000];
pub static RUSTCONV_WSTR_30DDABA0D544: [u16; 4] = [0x0031u16, 0x0030, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_4F11850FAC45: [u16; 5] = [0x0025u16, 0x0030, 0x0038, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_8AF36D3C73B6: [u16; 9] = [0x0049u16, 0x004D, 0x0045, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_AD263DD77A9A: [u16; 49] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0049, 0x004D, 0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_B479D0D4DBA3: [u16; 9] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0030, 0x0038, 0x006C, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_CAC534D4698B: [u16; 4] = [0x0031u16, 0x0032, 0x0033, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const REGKEY_KEYBOARD_LAYOUTS: [::core::ffi::c_ushort; 50] = unsafe {
    ::core::mem::transmute::<
        [u8; 100],
        [::core::ffi::c_ushort; 50],
    >(
        *b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0K\0e\0y\0b\0o\0a\0r\0d\0 \0L\0a\0y\0o\0u\0t\0s\0\0\0",
    )
};

#[no_mangle]
pub static mut ghImm32Inst: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut gpsi: PSERVERINFO = ::core::ptr::null::<tagSERVERINFO>() as *mut tagSERVERINFO;

#[no_mangle]
pub static mut gSharedInfo: SHAREDINFO = _SHAREDINFO {
    psi: ::core::ptr::null::<tagSERVERINFO>() as *mut tagSERVERINFO,
    aheList: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    pDispInfo: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    ulSharedDelta: 0,
    awmControl: [_WNDMSG {
        maxMsgs: 0,
        abMsgs: ::core::ptr::null::<::core::ffi::c_int>() as *mut ::core::ffi::c_int,
    }; 31],
    DefWindowMsgs: _WNDMSG {
        maxMsgs: 0,
        abMsgs: ::core::ptr::null::<::core::ffi::c_int>() as *mut ::core::ffi::c_int,
    },
    DefWindowSpecMsgs: _WNDMSG {
        maxMsgs: 0,
        abMsgs: ::core::ptr::null::<::core::ffi::c_int>() as *mut ::core::ffi::c_int,
    },
};

#[no_mangle]
pub static mut gfImmInitialized: BYTE = FALSE as BYTE;

#[no_mangle]
pub static mut gHighestUserAddress: ULONG_PTR = 0 as ULONG_PTR;

#[repr(C)]
pub struct SELECT_LAYOUT {
    pub hNewKL: HKL,
    pub hOldKL: HKL,
}
pub type LPSELECT_LAYOUT = *mut SELECT_LAYOUT;

pub const IMM32_UNITTEST_STR_0: *const u16 = RUSTCONV_WSTR_CAC534D4698B.as_ptr();
pub const IMM32_UNITTEST_STR_1: *const u16 = RUSTCONV_WSTR_30DDABA0D544.as_ptr();
pub const IMM32_UNITTEST_STR_2: *const u16 = RUSTCONV_WSTR_CAC534D4698B.as_ptr();
pub const IMM32_UNITTEST_STR_3: *const u16 = RUSTCONV_WSTR_30DDABA0D544.as_ptr();

unsafe extern "C" fn ImmInitializeGlobals(mut hMod: HMODULE) -> BOOL {
    let mut status: NTSTATUS = 0;
    let mut SysInfo: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();

    if !hMod.is_null() {
        ghImm32Inst = hMod;
    }

    if gfImmInitialized != 0 {
        return (TRUE) as BOOL;
    }

    status = RtlInitializeCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
    if ((status as ULONG) >> 30) == 3 as ULONG {
        return (FALSE) as BOOL;
    }

    status = NtQuerySystemInformation(
        SystemBasicInformation,
        core::ptr::addr_of_mut!(SysInfo) as PVOID,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if ((status as ULONG) >> 30) == 3 as ULONG {
        return (FALSE) as BOOL;
    }
    gHighestUserAddress = SysInfo.MaximumUserModeAddress;

    gfImmInitialized = (TRUE) as u8;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmRegisterClient(ptr: PSHAREDINFO, hMod: HINSTANCE) -> BOOL {
    gSharedInfo = *ptr;
    gpsi = gSharedInfo.psi;
    return ImmInitializeGlobals(hMod as HMODULE);
}

#[no_mangle]
pub unsafe extern "system" fn ImmLoadLayout(hKL: HKL, pImeInfoEx: PIMEINFOEX) -> BOOL {
    let mut cbData: DWORD;
    let mut dwType: DWORD = core::mem::zeroed();
    let mut hKey: HKEY = core::mem::zeroed();
    let mut error: LSTATUS;
    let mut szLayout: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
    let mut pszSubKey: LPCWSTR;

    /* Choose a key */
    if IS_IME_HKL!(hKL) || ((IS_CICERO_MODE!()) == false) || IS_16BIT_MODE!() {
        /* Non-Cicero? */
        StringCchPrintfW!(
            &raw mut szLayout as *mut WCHAR,
            _countof!(szLayout),
            RUSTCONV_WSTR_B479D0D4DBA3.as_ptr(),
            REGKEY_KEYBOARD_LAYOUTS,
            HandleToUlong!(hKL),
        );
        pszSubKey = &raw const szLayout as *const WCHAR as LPCWSTR;
    } else {
        /* Cicero */
        pszSubKey = RUSTCONV_WSTR_AD263DD77A9A.as_ptr();
    }

    /* Open the key */
    error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        pszSubKey,
        0,
        KEY_READ,
        &raw mut hKey,
    );
    if IS_ERROR_UNEXPECTEDLY!(error) {
        return (FALSE) as BOOL;
    }

    /* Load "IME File" value */
    cbData = ::core::mem::size_of::<[WCHAR; 80]>() as DWORD;
    error = RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_8AF36D3C73B6.as_ptr(),
        core::ptr::null_mut(),
        &raw mut dwType,
        &raw mut (*pImeInfoEx).wszImeFile as *mut WCHAR as LPBYTE,
        &raw mut cbData,
    );

    /* Avoid buffer overrun */
    (*pImeInfoEx).wszImeFile[(_countof!((*pImeInfoEx).wszImeFile) - 1) as usize] =
        UNICODE_NULL;

    RegCloseKey(hKey);

    if (error) as u32 != ERROR_SUCCESS || dwType != REG_SZ {
        return (FALSE) as BOOL; /* Failed */
    }

    (*pImeInfoEx).hkl = hKL;
    (*pImeInfoEx).fLoadFlag = 0;
    return Imm32LoadImeVerInfo(pImeInfoEx);
}

#[no_mangle]
pub unsafe extern "system" fn ImmFreeLayout(mut hKL: HKL) -> BOOL {
    let mut szKBD: [WCHAR; (KL_NAMELENGTH) as _] = [0; (KL_NAMELENGTH) as _];
    let mut iKL: UINT = 0;
    let mut cKLs: UINT = 0;
    let mut hOldKL: HKL = core::ptr::null_mut();
    let mut pList: *mut HKL = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut LangID: LANGID = 0;

    hOldKL = GetKeyboardLayout(0);

    if hKL == HKL_SWITCH_TO_NON_IME {
        if ((IS_IME_HKL!(hOldKL)) == false) {
            return (TRUE) as BOOL;
        }

        LangID = LANGIDFROMLCID!(GetSystemDefaultLCID()) as LANGID;

        cKLs = GetKeyboardLayoutList(0, core::ptr::null_mut());
        if cKLs != 0 {
            pList = ImmLocalAlloc(
                0,
                (cKLs as usize * core::mem::size_of::<HKL>()) as DWORD,
            ) as *mut HKL;
            if IS_NULL_UNEXPECTEDLY!(pList) {
                return (FALSE) as BOOL;
            }

            cKLs = GetKeyboardLayoutList((cKLs) as i32, pList);
            iKL = 0;
            while iKL < cKLs {
                if ((IS_IME_HKL!(*pList.add(iKL as usize))) == false) {
                    LangID = LOWORD!(*pList.add(iKL as usize)) as LANGID;
                    break;
                }
                iKL += 1;
            }

            ImmLocalFree!((pList).cast());
        }

        StringCchPrintfW!(
            (&raw mut szKBD) as *mut WCHAR,
            _countof!(szKBD),
            RUSTCONV_WSTR_4F11850FAC45.as_ptr(),
            LangID as core::ffi::c_uint,
        );
        if LoadKeyboardLayoutW((&raw mut szKBD) as *mut WCHAR as LPCWSTR, KLF_ACTIVATE).is_null() {
            LoadKeyboardLayoutW(
                RUSTCONV_WSTR_230B3B8A6763.as_ptr(),
                KLF_ACTIVATE | 0x200,
            );
        }
    } else if hKL == HKL_RELEASE_IME {
        RtlEnterCriticalSection(&raw mut gcsImeDpi);
        'retry: loop {
            pImeDpi = gpImeDpiList;
            while !pImeDpi.is_null() {
                if Imm32ReleaseIME((*pImeDpi).hKL) != 0 {
                    continue 'retry;
                }
                pImeDpi = (*pImeDpi).pNext as PIMEDPI;
            }
            break;
        }
        RtlLeaveCriticalSection(&raw mut gcsImeDpi);
    } else {
        if IS_IME_HKL!(hKL) && hKL != hOldKL {
            Imm32ReleaseIME(hKL);
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32SelectInputContext(mut hNewKL: HKL, mut hOldKL: HKL, mut hIMC: HIMC) {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut::<tagCLIENTIMC>();
    let mut pIC: LPINPUTCONTEXTDX = core::ptr::null_mut::<INPUTCONTEXTDX>();
    let mut pGL: LPGUIDELINE = core::ptr::null_mut::<tagGUIDELINE>();
    let mut pCI: LPCANDIDATEINFO = core::ptr::null_mut::<tagCANDIDATEINFO>();
    let mut pCS: LPCOMPOSITIONSTRING = core::ptr::null_mut::<tagCOMPOSITIONSTRING>();
    let mut LogFontA: LOGFONTA = core::mem::zeroed::<LOGFONTA>();
    let mut LogFontW: LOGFONTW = core::mem::zeroed::<LOGFONTW>();
    let mut fOldOpen: BOOL = 0;
    let mut bIsNewHKLIme: BOOL = TRUE as BOOL;
    let mut bIsOldHKLIme: BOOL = TRUE as BOOL;
    let mut bClientWide: BOOL = 0;
    let mut bNewDpiWide: BOOL = 0;
    let mut cbNewPrivate: DWORD = 0 as DWORD;
    let mut cbOldPrivate: DWORD = 0 as DWORD;
    let mut dwOldConversion: DWORD = 0;
    let mut dwOldSentence: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwNewSize: DWORD = 0;
    let mut pNewImeDpi: PIMEDPI = core::ptr::null_mut::<IMEDPI>();
    let mut pOldImeDpi: PIMEDPI = core::ptr::null_mut::<IMEDPI>();
    let mut hPrivate: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut pNewState: PIME_STATE = core::ptr::null_mut::<IME_STATE>();
    let mut pOldState: PIME_STATE = core::ptr::null_mut::<IME_STATE>();

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return;
    }

    pNewImeDpi = ImmLockImeDpi(hNewKL);

    if hNewKL != hOldKL {
        pOldImeDpi = ImmLockImeDpi(hOldKL);
    }

    if !pNewImeDpi.is_null() {
        cbNewPrivate = (*pNewImeDpi).ImeInfo.dwPrivateDataSize;
        (*pClientImc).uCodePage = (*pNewImeDpi).uCodePage;
    } else {
        (*pClientImc).uCodePage = CP_ACP as UINT;
    }

    if !pOldImeDpi.is_null() {
        cbOldPrivate = (*pOldImeDpi).ImeInfo.dwPrivateDataSize;
    }

    cbNewPrivate = max(cbNewPrivate, core::mem::size_of::<DWORD>() as DWORD);
    cbOldPrivate = max(cbOldPrivate, core::mem::size_of::<DWORD>() as DWORD);

    if (*pClientImc).hKL == hOldKL {
        if !pOldImeDpi.is_null() {
            if IS_IME_HKL!(hOldKL) != false {
                (*pOldImeDpi).ImeSelect.unwrap_unchecked()(hIMC, (FALSE) as i32);
            } else if IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == (false) {
                (*pOldImeDpi)
                    .CtfImeSelectEx
                    .unwrap_unchecked()(hIMC, (FALSE) as i32, hOldKL);
            }
        }
        (*pClientImc).hKL = core::ptr::null_mut::<HKL__>();
    }

    if CtfImmIsTextFrameServiceDisabled() != 0 && IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == (false) {
        bIsNewHKLIme = (IS_IME_HKL!(hNewKL) != false) as BOOL;
        bIsOldHKLIme = (IS_IME_HKL!(hOldKL) != false) as BOOL;
    }

    pIC = Imm32InternalLockIMC(hIMC, (FALSE) as BOOL) as LPINPUTCONTEXTDX;
    if pIC.is_null() {
        if !pNewImeDpi.is_null() {
            if IS_IME_HKL!(hNewKL) != false {
                (*pNewImeDpi).ImeSelect.unwrap_unchecked()(hIMC, (TRUE) as i32);
            } else if IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == (false) {
                (*pNewImeDpi)
                    .CtfImeSelectEx
                    .unwrap_unchecked()(hIMC, (TRUE) as i32, hNewKL);
            }

            (*pClientImc).hKL = hNewKL;
        }
    } else {
        dwOldConversion = (*pIC).__bindgen_anon_1.fdwConversion;
        dwOldSentence = (*pIC).__bindgen_anon_1.fdwSentence;
        fOldOpen = (*pIC).__bindgen_anon_1.fOpen;

        if !pNewImeDpi.is_null() {
            bClientWide = ((*pClientImc).dwFlags & CLIENTIMC_WIDE as DWORD) as BOOL;
            bNewDpiWide = ImeDpi_IsUnicode!(pNewImeDpi) as BOOL;
            if bClientWide != 0 && bNewDpiWide == 0 {
                if (*pIC).__bindgen_anon_1.fdwInit & INIT_LOGFONT as DWORD != 0 {
                    LogFontWideToAnsi(&raw mut (*pIC).__bindgen_anon_1.lfFont.W, &raw mut LogFontA);
                    (*pIC).__bindgen_anon_1.lfFont.A = LogFontA;
                }
                (*pClientImc).dwFlags &= !(CLIENTIMC_WIDE as DWORD);
            } else if bClientWide == 0 && bNewDpiWide != 0 {
                if (*pIC).__bindgen_anon_1.fdwInit & INIT_LOGFONT as DWORD != 0 {
                    LogFontAnsiToWide(&raw mut (*pIC).__bindgen_anon_1.lfFont.A, &raw mut LogFontW);
                    (*pIC).__bindgen_anon_1.lfFont.W = LogFontW;
                }
                (*pClientImc).dwFlags |= CLIENTIMC_WIDE as DWORD;
            }
        }

        if cbOldPrivate != cbNewPrivate {
            hPrivate = ImmReSizeIMCC((*pIC).__bindgen_anon_1.hPrivate, cbNewPrivate) as HANDLE;
            if hPrivate.is_null() {
                ImmDestroyIMCC((*pIC).__bindgen_anon_1.hPrivate);
                hPrivate = ImmCreateIMCC(cbNewPrivate) as HANDLE;
            }
            (*pIC).__bindgen_anon_1.hPrivate = hPrivate as HIMCC;
        }

        dwSize = ImmGetIMCCSize((*pIC).__bindgen_anon_1.hMsgBuf);
        if ImmGetIMCCLockCount((*pIC).__bindgen_anon_1.hMsgBuf) != 0
            || dwSize > MAX_IMCC_SIZE as DWORD
        {
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hMsgBuf);
            (*pIC).__bindgen_anon_1.hMsgBuf = ImmCreateIMCC(core::mem::size_of::<UINT>() as DWORD);
            (*pIC).__bindgen_anon_1.dwNumMsgBuf = 0 as DWORD;
        }

        dwSize = ImmGetIMCCSize((*pIC).__bindgen_anon_1.hGuideLine);
        dwNewSize = core::mem::size_of::<GUIDELINE>() as DWORD;
        if ImmGetIMCCLockCount((*pIC).__bindgen_anon_1.hGuideLine) != 0
            || dwSize < dwNewSize
            || dwSize > MAX_IMCC_SIZE as DWORD
        {
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hGuideLine);
            (*pIC).__bindgen_anon_1.hGuideLine = ImmCreateIMCC(dwNewSize);
            pGL = ImmLockIMCC((*pIC).__bindgen_anon_1.hGuideLine) as LPGUIDELINE;
            if !pGL.is_null() {
                (*pGL).dwSize = dwNewSize;
                ImmUnlockIMCC((*pIC).__bindgen_anon_1.hGuideLine);
            }
        }

        dwSize = ImmGetIMCCSize((*pIC).__bindgen_anon_1.hCandInfo);
        dwNewSize = core::mem::size_of::<CANDIDATEINFO>() as DWORD;
        if ImmGetIMCCLockCount((*pIC).__bindgen_anon_1.hCandInfo) != 0
            || dwSize < dwNewSize
            || dwSize > MAX_IMCC_SIZE as DWORD
        {
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hCandInfo);
            (*pIC).__bindgen_anon_1.hCandInfo = ImmCreateIMCC(dwNewSize);
            pCI = ImmLockIMCC((*pIC).__bindgen_anon_1.hCandInfo) as LPCANDIDATEINFO;
            if !pCI.is_null() {
                (*pCI).dwSize = dwNewSize;
                ImmUnlockIMCC((*pIC).__bindgen_anon_1.hCandInfo);
            }
        }

        dwSize = ImmGetIMCCSize((*pIC).__bindgen_anon_1.hCompStr);
        dwNewSize = core::mem::size_of::<COMPOSITIONSTRING>() as DWORD;
        if ImmGetIMCCLockCount((*pIC).__bindgen_anon_1.hCompStr) != 0
            || dwSize < dwNewSize
            || dwSize > MAX_IMCC_SIZE as DWORD
        {
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hCompStr);
            (*pIC).__bindgen_anon_1.hCompStr = ImmCreateIMCC(dwNewSize);
            pCS = ImmLockIMCC((*pIC).__bindgen_anon_1.hCompStr) as LPCOMPOSITIONSTRING;
            if !pCS.is_null() {
                (*pCS).dwSize = dwNewSize;
                ImmUnlockIMCC((*pIC).__bindgen_anon_1.hCompStr);
            }
        }

        if !pOldImeDpi.is_null() && bIsOldHKLIme != 0 {
            pOldState = Imm32FetchImeState(pIC, hOldKL);
            if !pOldState.is_null() {
                Imm32SaveImeStateSentence(pIC, pOldState, hOldKL);
            }
        }

        if !pNewImeDpi.is_null() && bIsNewHKLIme != 0 {
            pNewState = Imm32FetchImeState(pIC, hNewKL);
        }

        if pOldState != pNewState {
            if !pOldState.is_null() {
                (*pOldState).fOpen = ((*pIC).__bindgen_anon_1.fOpen != 0) as core::ffi::c_int as WORD;
                (*pOldState).dwConversion = (*pIC).__bindgen_anon_1.fdwConversion;
                (*pOldState).dwConversion &= !(IME_CMODE_EUDC as DWORD);
                (*pOldState).dwSentence = (*pIC).__bindgen_anon_1.fdwSentence;
                (*pOldState).dwInit = (*pIC).__bindgen_anon_1.fdwInit;
            }

            if !pNewState.is_null() {
                if (*pIC).dwChange & INPUTCONTEXTDX_CHANGE_FORCE_OPEN as DWORD != 0 {
                    (*pIC).dwChange &= !(INPUTCONTEXTDX_CHANGE_FORCE_OPEN as DWORD);
                    (*pIC).__bindgen_anon_1.fOpen = TRUE as BOOL;
                } else {
                    (*pIC).__bindgen_anon_1.fOpen = (*pNewState).fOpen as BOOL;
                }

                (*pIC).__bindgen_anon_1.fdwConversion = (*pNewState).dwConversion;
                (*pIC).__bindgen_anon_1.fdwConversion &= !(IME_CMODE_EUDC as DWORD);
                (*pIC).__bindgen_anon_1.fdwSentence = (*pNewState).dwSentence;
                (*pIC).__bindgen_anon_1.fdwInit = (*pNewState).dwInit;
            }
        }

        if !pNewState.is_null() {
            Imm32LoadImeStateSentence(pIC, pNewState, hNewKL);
        }

        if !pNewImeDpi.is_null() {
            if IS_IME_HKL!(hNewKL) != false {
                (*pNewImeDpi).ImeSelect.unwrap_unchecked()(hIMC, (TRUE) as i32);
            } else if IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == (false) {
                (*pNewImeDpi)
                    .CtfImeSelectEx
                    .unwrap_unchecked()(hIMC, (TRUE) as i32, hNewKL);
            }

            (*pClientImc).hKL = hNewKL;
        }

        (*pIC).dwChange = 0 as DWORD;
        if (*pIC).__bindgen_anon_1.fOpen != fOldOpen {
            (*pIC).dwChange |= INPUTCONTEXTDX_CHANGE_OPEN as DWORD;
        }
        if (*pIC).__bindgen_anon_1.fdwConversion != dwOldConversion {
            (*pIC).dwChange |= INPUTCONTEXTDX_CHANGE_CONVERSION as DWORD;
        }
        if (*pIC).__bindgen_anon_1.fdwSentence != dwOldSentence {
            (*pIC).dwChange |= INPUTCONTEXTDX_CHANGE_SENTENCE as DWORD;
        }

        ImmUnlockIMC(hIMC);
    }

    ImmUnlockImeDpi(pOldImeDpi);
    ImmUnlockImeDpi(pNewImeDpi);
    ImmUnlockClientImc(pClientImc);
}
unsafe extern "system" fn Imm32SelectContextProc(hIMC: HIMC, lParam: LPARAM) -> BOOL {
    let pSelect: LPSELECT_LAYOUT = lParam as LPSELECT_LAYOUT;
    Imm32SelectInputContext((*pSelect).hNewKL, (*pSelect).hOldKL, hIMC);
    return (TRUE) as BOOL;
}

unsafe extern "system" fn Imm32NotifyIMEProc(hIMC: HIMC, lParam: LPARAM) -> BOOL {
    ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, lParam as DWORD, 0);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmActivateLayout(hKL: HKL) -> BOOL {
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut hOldKL: HKL = core::ptr::null_mut();
    let mut lParam: LPARAM = 0;
    let mut hwndDefIME: HWND = core::ptr::null_mut();
    let mut SelectLayout: SELECT_LAYOUT = SELECT_LAYOUT {
        hNewKL: core::ptr::null_mut(),
        hOldKL: core::ptr::null_mut(),
    };

    hOldKL = GetKeyboardLayout(0);

    if hOldKL == hKL && ((*GetWin32ClientInfo!()).CI_flags & (CI_IMMACTIVATE as ULONG_PTR)) == 0 {
        return (TRUE) as BOOL;
    }

    ImmLoadIME(hKL);

    if hOldKL != hKL {
        pImeDpi = ImmLockImeDpi(hOldKL);
        if !pImeDpi.is_null() {
            if ((*pImeDpi).ImeInfo.fdwProperty & IME_PROP_COMPLETE_ON_UNSELECT) != 0 {
                lParam = CPS_COMPLETE as LPARAM;
            } else {
                lParam = CPS_CANCEL as LPARAM;
            }
            ImmUnlockImeDpi(pImeDpi);

            ImmEnumInputContext(0, Some(Imm32NotifyIMEProc), lParam);
        }

        hwndDefIME = ImmGetDefaultIMEWnd(core::ptr::null_mut());
        if IsWindow(hwndDefIME) != 0 {
            SendMessageW(hwndDefIME, WM_IME_SELECT, (FALSE) as WPARAM, hOldKL as LPARAM);
        }

        NtUserSetThreadLayoutHandles(hKL, hOldKL);
    }

    SelectLayout.hNewKL = hKL;
    SelectLayout.hOldKL = hOldKL;
    ImmEnumInputContext(0, Some(Imm32SelectContextProc), core::ptr::addr_of_mut!(SelectLayout) as LPARAM);

    if IsWindow(hwndDefIME) != 0 {
        SendMessageW(hwndDefIME, WM_IME_SELECT, (TRUE) as WPARAM, hKL as LPARAM);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmAssociateContext(hWnd: HWND, hIMC: HIMC) -> HIMC {
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut hwndFocus: HWND = core::ptr::null_mut();
    let mut dwValue: DWORD = 0;
    let mut hOldIMC: HIMC = core::ptr::null_mut();

    if !(IS_IMM_MODE!() != 0) {
        return core::ptr::null_mut();
    }

    pWnd = ValidateHwnd!((hWnd).cast()) as PWND;
    if IS_NULL_UNEXPECTEDLY!(pWnd) != false {
        return core::ptr::null_mut();
    }

    if !hIMC.is_null() && IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return core::ptr::null_mut();
    }

    hOldIMC = (*pWnd).hImc;
    if hOldIMC == hIMC {
        return hIMC;
    }

    dwValue = NtUserAssociateInputContext(hWnd, hIMC, 0);
    match dwValue {
        0 => return hOldIMC,
        1 => {
            hwndFocus = NtUserQueryWindow(hWnd, QUERY_WINDOW_FOCUS) as HWND;
            if hwndFocus == hWnd {
                ImmSetActiveContext(hWnd, hOldIMC, (FALSE) as BOOL);
                ImmSetActiveContext(hWnd, hIMC, (TRUE) as BOOL);
            }
            return hOldIMC;
        }
        _ => return core::ptr::null_mut(),
    };
}

#[no_mangle]
pub unsafe extern "system" fn ImmAssociateContextEx(
    hWnd: HWND,
    mut hIMC: HIMC,
    dwFlags: DWORD,
) -> BOOL {
    let mut hwndFocus: HWND;
    let mut pFocusWnd: PWND;
    let mut hOldIMC: HIMC = core::ptr::null_mut();
    let mut dwValue: DWORD;

    if IS_IMM_MODE!() == 0 {
        return (FALSE) as BOOL;
    }

    if !hIMC.is_null() && (dwFlags & IACE_DEFAULT) == 0 && IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    hwndFocus = NtUserQueryWindow(hWnd, QUERY_WINDOW_FOCUS) as HWND;
    pFocusWnd = ValidateHwnd!((hwndFocus).cast()) as PWND;
    if !pFocusWnd.is_null() {
        hOldIMC = (*pFocusWnd).hImc;
    }

    dwValue = NtUserAssociateInputContext(hWnd, hIMC, dwFlags);
    if dwValue == 0 {
        return (TRUE) as BOOL;
    } else if dwValue == 1 {
        pFocusWnd = ValidateHwnd!((hwndFocus).cast()) as PWND;
        if !pFocusWnd.is_null() {
            hIMC = (*pFocusWnd).hImc;
            if hIMC != hOldIMC {
                ImmSetActiveContext(hwndFocus, hOldIMC, (FALSE) as BOOL);
                ImmSetActiveContext(hwndFocus, hIMC, (TRUE) as BOOL);
            }
        }
        return (TRUE) as BOOL;
    } else {
        return (FALSE) as BOOL;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ImmCreateContext() -> HIMC {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut hIMC: HIMC = core::ptr::null_mut();

    if ((IS_IMM_MODE!()) == 0) {
        return core::ptr::null_mut();
    }

    pClientImc = ImmLocalAlloc(
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CLIENTIMC>() as DWORD,
    ) as PCLIENTIMC;
    if IS_NULL_UNEXPECTEDLY!(pClientImc) {
        return core::ptr::null_mut();
    }

    hIMC = NtUserCreateInputContext(pClientImc as ULONG_PTR) as HIMC;
    if IS_NULL_UNEXPECTEDLY!(hIMC) {
        ImmLocalFree!((pClientImc).cast());
        return core::ptr::null_mut();
    }

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*pClientImc).cs));

    (*pClientImc).dwCompatFlags = NtUserGetThreadState(THREADSTATE_IMECOMPATFLAGS) as DWORD;

    hIMC
}

unsafe extern "C" fn Imm32DestroyImeModeSaver(pIC: LPINPUTCONTEXTDX) {
    let mut pState: PIME_STATE;
    let mut pNext: PIME_STATE;
    let mut pSubState: PIME_SUBSTATE;
    let mut pSubNext: PIME_SUBSTATE;

    pState = (*pIC).pState as PIME_STATE;
    while !pState.is_null() {
        pNext = (*pState).pNext as PIME_STATE;

        pSubState = (*pState).pSubState as PIME_SUBSTATE;
        while !pSubState.is_null() {
            pSubNext = (*pSubState).pNext as PIME_SUBSTATE;
            ImmLocalFree!(pSubState as LPVOID);
            pSubState = pSubNext;
        }

        ImmLocalFree!(pState as LPVOID);
        pState = pNext;
    }

    (*pIC).pState = core::ptr::null_mut();
}

unsafe extern "C" fn Imm32DestroyInputContext(hIMC: HIMC, hKL: HKL, bKeep: BOOL) -> BOOL {
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXTDX = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pIMC: PIMC = core::ptr::null_mut();

    if hIMC.is_null() {
        return (FALSE) as BOOL;
    }

    if IS_IMM_MODE!() == 0 {
        return (FALSE) as BOOL;
    }

    pIMC = ValidateHandle((hIMC).cast(), TYPE_INPUTCONTEXT) as PIMC;
    if IS_NULL_UNEXPECTEDLY!(pIMC) != false {
        return (FALSE) as BOOL;
    }

    if (*pIMC).head.__bindgen_anon_1.pti != Imm32CurrentPti() {
        return (FALSE) as BOOL;
    }

    pClientImc = (*pIMC).dwClientImcData as PCLIENTIMC;
    'finish: {
        'quit: {
            if pClientImc.is_null() {
                break 'finish;
            }

            if ((*pClientImc).dwFlags & CLIENTIMC_LOCKED) != 0 && bKeep == 0 {
                return (FALSE) as BOOL;
            }

            if ((*pClientImc).dwFlags & CLIENTIMC_DESTROY) != 0 {
                return (TRUE) as BOOL;
            }

            InterlockedIncrement(core::ptr::addr_of_mut!((*pClientImc).cLockObj) as *mut core::ffi::c_long);

            if IS_NULL_UNEXPECTEDLY!((*pClientImc).hInputContext) != false {
                break 'quit;
            }

            pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
            if IS_NULL_UNEXPECTEDLY!(pIC) != false {
                ImmUnlockClientImc(pClientImc);
                return (FALSE) as BOOL;
            }

            CtfImmTIMDestroyInputContext(hIMC);

            if (*pClientImc).hKL == hKL {
                pImeDpi = ImmLockImeDpi(hKL);
                if !pImeDpi.is_null() {
                    if IS_IME_HKL!(hKL) != false {
                        (*pImeDpi).ImeSelect.unwrap_unchecked()(hIMC, (FALSE) as i32);
                    } else if IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == false {
                        (*pImeDpi).CtfImeSelectEx.unwrap_unchecked()(hIMC, (FALSE) as i32, hKL);
                    }

                    ImmUnlockImeDpi(pImeDpi);
                }

                (*pClientImc).hKL = core::ptr::null_mut();
            }

            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hPrivate);
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hMsgBuf);
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hGuideLine);
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hCandInfo);
            ImmDestroyIMCC((*pIC).__bindgen_anon_1.hCompStr);
            Imm32DestroyImeModeSaver(pIC);
            ImmUnlockIMC(hIMC);
        }

        // Quit:
        (*pClientImc).dwFlags |= CLIENTIMC_DESTROY;
        ImmUnlockClientImc(pClientImc);
    }

    // Finish:
    if bKeep != 0 {
        return (TRUE) as BOOL;
    }
    return NtUserDestroyInputContext(hIMC);
}

unsafe extern "C" fn Imm32CreateInputContext(
    hIMC: HIMC,
    pIC: LPINPUTCONTEXT,
    pClientImc: PCLIENTIMC,
    hKL: HKL,
    fSelect: BOOL,
) -> BOOL {
    let mut dwIndex: DWORD;
    let mut cbPrivate: DWORD;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut pCS: LPCOMPOSITIONSTRING;
    let mut pCI: LPCANDIDATEINFO;
    let mut pGL: LPGUIDELINE;

    'quit: {
        /* Create IC components */
        (*pIC).hCompStr = ImmCreateIMCC((core::mem::size_of::<COMPOSITIONSTRING>()) as DWORD);
        (*pIC).hCandInfo = ImmCreateIMCC((core::mem::size_of::<CANDIDATEINFO>()) as DWORD);
        (*pIC).hGuideLine = ImmCreateIMCC((core::mem::size_of::<GUIDELINE>()) as DWORD);
        (*pIC).hMsgBuf = ImmCreateIMCC(core::mem::size_of::<UINT>() as DWORD);
        if IS_NULL_UNEXPECTEDLY!((*pIC).hCompStr)
            || IS_NULL_UNEXPECTEDLY!((*pIC).hCandInfo)
            || IS_NULL_UNEXPECTEDLY!((*pIC).hGuideLine)
            || IS_NULL_UNEXPECTEDLY!((*pIC).hMsgBuf)
        {
            break 'quit;
        }

        /* Initialize IC components */
        pCS = ImmLockIMCC((*pIC).hCompStr) as LPCOMPOSITIONSTRING;
        if IS_NULL_UNEXPECTEDLY!(pCS) {
            break 'quit;
        }
        (*pCS).dwSize = (core::mem::size_of::<COMPOSITIONSTRING>()) as DWORD;
        ImmUnlockIMCC((*pIC).hCompStr);

        pCI = ImmLockIMCC((*pIC).hCandInfo) as LPCANDIDATEINFO;
        if IS_NULL_UNEXPECTEDLY!(pCI) {
            break 'quit;
        }
        (*pCI).dwSize = (core::mem::size_of::<CANDIDATEINFO>()) as DWORD;
        ImmUnlockIMCC((*pIC).hCandInfo);

        pGL = ImmLockIMCC((*pIC).hGuideLine) as LPGUIDELINE;
        if IS_NULL_UNEXPECTEDLY!(pGL) {
            break 'quit;
        }
        (*pGL).dwSize = (core::mem::size_of::<GUIDELINE>()) as DWORD;
        ImmUnlockIMCC((*pIC).hGuideLine);

        (*pIC).dwNumMsgBuf = 0;
        (*pIC).fOpen = (FALSE) as i32;
        (*pIC).fdwSentence = 0;
        (*pIC).fdwConversion = (*pIC).fdwSentence;

        dwIndex = 0;
        while dwIndex < MAX_CANDIDATEFORM {
            (*pIC).cfCandForm[dwIndex as usize].dwIndex = IMM_INVALID_CANDFORM;
            dwIndex = dwIndex.wrapping_add(1);
        }

        /* Get private data size */
        pImeDpi = ImmLockImeDpi(hKL);
        if pImeDpi.is_null() {
            cbPrivate = core::mem::size_of::<DWORD>() as DWORD;
        } else {
            /* Update CLIENTIMC */
            (*pClientImc).uCodePage = (*pImeDpi).uCodePage;
            if ImeDpi_IsUnicode!(pImeDpi) {
                (*pClientImc).dwFlags |= CLIENTIMC_WIDE;
            }

            cbPrivate = (*pImeDpi).ImeInfo.dwPrivateDataSize;
            cbPrivate = max(cbPrivate, core::mem::size_of::<DWORD>() as DWORD); /* ensure minimum size, like Imm32SelectInputContext does */
        }

        /* Create private data */
        (*pIC).hPrivate = ImmCreateIMCC(cbPrivate);
        if IS_NULL_UNEXPECTEDLY!((*pIC).hPrivate) {
            break 'quit;
        }

        CtfImmTIMCreateInputContext(hIMC);

        if !pImeDpi.is_null() {
            /* Select the IME */
            if fSelect != 0 {
                if IS_IME_HKL!(hKL) {
                    (*pImeDpi).ImeSelect.unwrap_unchecked()(
                        hIMC,
                        (TRUE) as i32,
                    );
                } else if IS_CICERO_MODE!() && !IS_16BIT_MODE!() {
                    (*pImeDpi).CtfImeSelectEx.unwrap_unchecked()(
                        hIMC,
                        (TRUE) as i32,
                        hKL,
                    );
                }
            }

            /* Set HKL */
            (*pClientImc).hKL = hKL;

            ImmUnlockImeDpi(pImeDpi);
        }

        return (TRUE) as BOOL;
    }

    /* Fail: */
    if !pImeDpi.is_null() {
        ImmUnlockImeDpi(pImeDpi);
    }

    (*pIC).hMsgBuf = ImmDestroyIMCC((*pIC).hMsgBuf);
    (*pIC).hGuideLine = ImmDestroyIMCC((*pIC).hGuideLine);
    (*pIC).hCandInfo = ImmDestroyIMCC((*pIC).hCandInfo);
    (*pIC).hCompStr = ImmDestroyIMCC((*pIC).hCompStr);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32InternalLockIMC(hIMC: HIMC, fSelect: BOOL) -> LPINPUTCONTEXT {
    let mut hIC: HANDLE = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC;
    let mut LangID: WORD;
    let mut dwThreadId: DWORD;
    let mut hOldKL: HKL;
    let mut hNewKL: HKL;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();

    pClientImc = ImmLockClientImc(hIMC);
    if pClientImc.is_null() {
        return core::ptr::null_mut();
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*pClientImc).cs));

    let ok: BOOL = 'body: {
        if !(*pClientImc).hInputContext.is_null() {
            pIC = LocalLock((*pClientImc).hInputContext as HLOCAL) as LPINPUTCONTEXT;
            if IS_NULL_UNEXPECTEDLY!(pIC) != false {
                break 'body 0;
            }

            CtfImmTIMCreateInputContext(hIMC);
            break 'body 1;
        }

        dwThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD;
        if dwThreadId == GetCurrentThreadId() && IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == false {
            hOldKL = GetKeyboardLayout(0);
            LangID = LOWORD!(hOldKL as DWORD) as WORD;
            hNewKL = UlongToHandle!(MAKELONG!(LangID, LangID)) as HKL;

            pImeDpi = Imm32FindOrLoadImeDpi(hNewKL);
            if !pImeDpi.is_null() {
                CtfImmTIMActivate(hNewKL);
            }
        }

        if NtUserQueryInputContext(hIMC, QIC_DEFAULTWINDOWIME) == 0 {
            break 'body 0;
        }

        hIC = LocalAlloc(LHND as UINT, core::mem::size_of::<INPUTCONTEXTDX>() as SIZE_T) as HANDLE;
        pIC = LocalLock(hIC as HLOCAL) as LPINPUTCONTEXT;
        if IS_NULL_UNEXPECTEDLY!(pIC) != false {
            LocalFree(hIC as HLOCAL);
            break 'body 0;
        }
        (*pClientImc).hInputContext = hIC;

        hNewKL = GetKeyboardLayout(dwThreadId);
        if Imm32CreateInputContext(hIMC, pIC, pClientImc, hNewKL, fSelect) == 0 {
            LocalUnlock(hIC as HLOCAL);
            (*pClientImc).hInputContext = LocalFree(hIC as HLOCAL) as HANDLE;
            break 'body 0;
        }

        1
    };

    if ok != 0 {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pClientImc).cs));
        InterlockedIncrement(core::ptr::addr_of_mut!((*pClientImc).cLockObj) as *mut LONG);
        ImmUnlockClientImc(pClientImc);
        return pIC;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pClientImc).cs));
    ImmUnlockClientImc(pClientImc);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn ImmDestroyContext(hIMC: HIMC) -> BOOL {
    let hKL: HKL;

    if IS_IMM_MODE!() == 0 {
        return (FALSE) as BOOL;
    }

    if IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    hKL = GetKeyboardLayout(0);
    return Imm32DestroyInputContext(hIMC, hKL, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmLockClientImc(hImc: HIMC) -> PCLIENTIMC {
    let mut pIMC: PIMC = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();

    if IS_NULL_UNEXPECTEDLY!(hImc) != false {
        return core::ptr::null_mut();
    }

    pIMC = ValidateHandle((hImc).cast(), TYPE_INPUTCONTEXT) as PIMC;
    if pIMC.is_null() || Imm32CheckImcProcess(pIMC) == 0 {
        return core::ptr::null_mut();
    }

    pClientImc = (*pIMC).dwClientImcData as PCLIENTIMC;
    'finish: {
        if !pClientImc.is_null() {
            if (*pClientImc).dwFlags & CLIENTIMC_DESTROY != 0 {
                return core::ptr::null_mut();
            }
            break 'finish;
        }

        pClientImc = ImmLocalAlloc(
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<CLIENTIMC>() as DWORD,
        ) as PCLIENTIMC;
        if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
            return core::ptr::null_mut();
        }

        RtlInitializeCriticalSection(&raw mut (*pClientImc).cs);
        (*pClientImc).dwCompatFlags =
            NtUserGetThreadState(THREADSTATE_IMECOMPATFLAGS) as DWORD;

        if NtUserUpdateInputContext(hImc, UIC_CLIENTIMCDATA, pClientImc as DWORD_PTR) == 0 {
            ImmLocalFree!(pClientImc as PVOID);
            return core::ptr::null_mut();
        }

        (*pClientImc).dwFlags |= CLIENTIMC_LOCKED;
    }

    InterlockedIncrement(&raw mut (*pClientImc).cLockObj as *mut ::core::ffi::c_long);
    return pClientImc;
}

#[no_mangle]
pub unsafe extern "system" fn ImmUnlockClientImc(pClientImc: PCLIENTIMC) {
    let mut cLocks: LONG;
    let mut hInputContext: HANDLE;

    cLocks = InterlockedDecrement(&raw mut (*pClientImc).cLockObj as *mut LONG) as LONG;
    if cLocks != 0 || ((*pClientImc).dwFlags & CLIENTIMC_DESTROY) == 0 {
        return;
    }

    hInputContext = (*pClientImc).hInputContext;
    if !hInputContext.is_null() {
        LocalFree(hInputContext as HLOCAL);
    }

    RtlDeleteCriticalSection(&raw mut (*pClientImc).cs);
    ImmLocalFree!(pClientImc as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ImmGetSaveContext(hWnd: HWND, dwContextFlags: DWORD) -> HIMC {
    let mut hIMC: HIMC = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pWnd: PWND = core::ptr::null_mut();

    if IS_IMM_MODE!() == 0 {
        return core::ptr::null_mut();
    }

    'Quit: {
        if hWnd.is_null() {
            hIMC = NtUserGetThreadState(THREADSTATE_DEFAULTINPUTCONTEXT) as HIMC;
            break 'Quit;
        }

        pWnd = ValidateHwnd!((hWnd).cast()) as PWND;
        if IS_NULL_UNEXPECTEDLY!(pWnd) != false || IS_CROSS_PROCESS_HWND!(hWnd) != false {
            return core::ptr::null_mut();
        }

        hIMC = (*pWnd).hImc;
        if hIMC.is_null() && (dwContextFlags & 1) != 0 {
            hIMC = NtUserQueryWindow(hWnd, QUERY_WINDOW_DEFAULT_ICONTEXT) as HIMC;
        }
    }

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return core::ptr::null_mut();
    }

    if (dwContextFlags & 2) != 0
        && ((*pClientImc).dwFlags & CLIENTIMC_DISABLEIME) != 0
    {
        hIMC = core::ptr::null_mut();
    }

    ImmUnlockClientImc(pClientImc);
    return hIMC;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetContext(hWnd: HWND) -> HIMC {
    if IS_NULL_UNEXPECTEDLY!(hWnd) != false {
        return core::ptr::null_mut();
    }
    return ImmGetSaveContext(hWnd, 2);
}

#[no_mangle]
pub unsafe extern "system" fn ImmLockIMC(hIMC: HIMC) -> LPINPUTCONTEXT {
    return Imm32InternalLockIMC(hIMC, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn ImmUnlockIMC(hIMC: HIMC) -> BOOL {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return (FALSE) as BOOL;
    }

    if !(*pClientImc).hInputContext.is_null() {
        LocalUnlock((*pClientImc).hInputContext as HLOCAL);
    }

    InterlockedDecrement(core::ptr::addr_of_mut!((*pClientImc).cLockObj) as *mut core::ffi::c_long);
    ImmUnlockClientImc(pClientImc);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmReleaseContext(hWnd: HWND, hIMC: HIMC) -> BOOL {
    UNREFERENCED_PARAMETER!(hWnd);
    UNREFERENCED_PARAMETER!(hIMC);
    return (TRUE) as BOOL; // Do nothing. This is correct.
}

#[no_mangle]
pub unsafe extern "system" fn ImmEnumInputContext(
    dwThreadId: DWORD,
    lpfn: IMCENUMPROC,
    lParam: LPARAM,
) -> BOOL {
    let mut phList: *mut HIMC = core::ptr::null_mut();
    let mut dwIndex: DWORD;
    let mut dwCount: DWORD;
    let mut ret: BOOL = (TRUE) as i32;
    let mut hIMC: HIMC;

    if IS_IMM_MODE!() == 0 {
        return (FALSE) as BOOL;
    }

    dwCount = Imm32BuildHimcList(dwThreadId, core::ptr::addr_of_mut!(phList));
    if IS_ZERO_UNEXPECTEDLY!(dwCount) != false {
        return (FALSE) as BOOL;
    }

    dwIndex = 0;
    while dwIndex < dwCount {
        hIMC = *phList.add(dwIndex as usize);
        if !hIMC.is_null()
            && !gpsi.is_null()
            && !ValidateHandle(hIMC as HANDLE, TYPE_INPUTCONTEXT).is_null()
        {
            ret = (lpfn.unwrap_unchecked())(hIMC, lParam);
            if ret == 0 {
                break;
            }
        }
        dwIndex += 1;
    }

    ImmLocalFree!(phList as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetActiveContext(
    hWnd: HWND,
    hIMC: HIMC,
    fActive: BOOL,
) -> BOOL {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXTDX = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut hOldIMC: HIMC = core::ptr::null_mut();
    let mut hKL: HKL = core::ptr::null_mut();
    let mut fOpen: BOOL = (FALSE) as i32;
    let mut dwConversion: DWORD = 0;
    let mut dwShowFlags: DWORD = ISC_SHOWUIALL;
    let mut hwndDefIME: HWND = core::ptr::null_mut();

    if ((IS_IMM_MODE!()) == 0) {
        return (FALSE) as BOOL;
    }

    pClientImc = ImmLockClientImc(hIMC);

    if fActive == 0 {
        if !pClientImc.is_null() {
            (*pClientImc).dwFlags &= !CLIENTIMC_ACTIVE;
        }
    } else if !hIMC.is_null() {
        if IS_NULL_UNEXPECTEDLY!(pClientImc) {
            return (FALSE) as BOOL;
        }

        pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
        if IS_NULL_UNEXPECTEDLY!(pIC) {
            ImmUnlockClientImc(pClientImc);
            return (FALSE) as BOOL;
        }

        (*pIC).__bindgen_anon_1.hWnd = hWnd;
        (*pClientImc).dwFlags |= CLIENTIMC_ACTIVE;

        if (*pIC).dwUIFlags & _IME_UI_HIDDEN != 0 {
            dwShowFlags = (ISC_SHOWUIGUIDELINE | ISC_SHOWUIALLCANDIDATEWINDOW);
        }

        fOpen = (*pIC).__bindgen_anon_1.fOpen;
        dwConversion = (*pIC).__bindgen_anon_1.fdwConversion;

        ImmUnlockIMC(hIMC);
    } else {
        hOldIMC = ImmGetSaveContext(hWnd, 1);
        pIC = ImmLockIMC(hOldIMC) as LPINPUTCONTEXTDX;
        if !pIC.is_null() {
            (*pIC).__bindgen_anon_1.hWnd = hWnd;
            ImmUnlockIMC(hOldIMC);
        }
    }

    hKL = GetKeyboardLayout(0);
    if IS_CICERO_MODE!() && !IS_16BIT_MODE!() {
        CtfImeSetActiveContextAlways(hIMC, fActive, hWnd, hKL);
        hKL = GetKeyboardLayout(0);
    }

    pImeDpi = ImmLockImeDpi(hKL);
    if !pImeDpi.is_null() {
        if IS_IME_HKL!(hKL) {
            ((*pImeDpi).ImeSetActiveContext.unwrap_unchecked())(hIMC, fActive);
        }
        ImmUnlockImeDpi(pImeDpi);
    }

    if IsWindow(hWnd) != 0 {
        SendMessageW(hWnd, WM_IME_SETCONTEXT, (fActive) as WPARAM, (dwShowFlags) as LPARAM);
        if fActive != 0 {
            NtUserNotifyIMEStatus(hWnd, fOpen, dwConversion);
        }
    } else if fActive == 0 {
        hwndDefIME = ImmGetDefaultIMEWnd(core::ptr::null_mut());
        if !hwndDefIME.is_null() {
            SendMessageW(hwndDefIME, WM_IME_SETCONTEXT, 0, (dwShowFlags) as LPARAM);
        }
    }

    if !pClientImc.is_null() {
        ImmUnlockClientImc(pClientImc);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmWINNLSGetEnableStatus(mut hWnd: HWND) -> BOOL {
    if Imm32IsSystemJapaneseOrKorean() == 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return (FALSE) as BOOL;
    }

    return !ImmGetSaveContext(hWnd, 2 as DWORD).is_null() as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetActiveContextConsoleIME(hwnd: HWND, fFlag: BOOL) -> BOOL {
    let mut hIMC: HIMC = core::ptr::null_mut();
    hIMC = ImmGetContext(hwnd);
    if IS_NULL_UNEXPECTEDLY!(hIMC) != false {
        return (FALSE) as BOOL;
    }
    return ImmSetActiveContext(hwnd, hIMC, fFlag);
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyboardLayoutCP(wLangId: LANGID) -> UINT {
    let mut szText: [WCHAR; 8] = [0; 8];
    static mut s_wKeyboardLangIdCache: LANGID = 0;
    static mut s_uKeyboardLayoutCPCache: UINT = 0;

    if wLangId == s_wKeyboardLangIdCache {
        return s_uKeyboardLayoutCPCache;
    }

    if GetLocaleInfoW(
        wLangId as LCID,
        LOCALE_IDEFAULTANSICODEPAGE as LCTYPE,
        core::ptr::addr_of_mut!(szText) as LPWSTR,
        _countof!(szText) as ::core::ffi::c_int,
    ) == 0
    {
        return 0;
    }

    s_wKeyboardLangIdCache = wLangId;
    szText[(_countof!(szText) - 1) as usize] = UNICODE_NULL as WCHAR; /* Avoid buffer overrun */
    s_uKeyboardLayoutCPCache = wcstol(
        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
        core::ptr::null_mut(),
        10 as ::core::ffi::c_int,
    ) as UINT;
    return s_uKeyboardLayoutCPCache;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32UnitTest() {
    if 0 != 0 {
        let mut dwValue: DWORD = 0;
        let mut szText: [u16; 64] = [0; 64];

        Imm32StrToUInt(RUSTCONV_WSTR_CAC534D4698B.as_ptr(), core::ptr::addr_of_mut!(dwValue), 10);

        Imm32StrToUInt(RUSTCONV_WSTR_30DDABA0D544.as_ptr(), core::ptr::addr_of_mut!(dwValue), 16);

        Imm32UIntToStr(123, 10, szText.as_mut_ptr(), (_countof!(szText)) as USHORT);

        Imm32UIntToStr(0x100, 16, szText.as_mut_ptr(), (_countof!(szText)) as USHORT);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ImmDllInitialize(
    hDll: HINSTANCE,
    dwReason: ULONG,
    pReserved: PVOID,
) -> BOOL {
    let mut hKL: HKL = core::ptr::null_mut();
    let mut hIMC: HIMC = core::ptr::null_mut();

    match dwReason {
        DLL_PROCESS_ATTACH => {
            if ImmInitializeGlobals(hDll) == 0 {
                return (FALSE) as BOOL;
            }
            if User32InitializeImmEntryTable(IMM_INIT_MAGIC as DWORD) == 0 {
                return (FALSE) as BOOL;
            }
            Imm32UnitTest();
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {
            if ((IS_IMM_MODE!()) == 0) || (*NtCurrentTeb()).Win32ThreadInfo.is_null() {
                return (TRUE) as BOOL;
            }
            hKL = GetKeyboardLayout(0 as DWORD);
            hIMC = NtUserGetThreadState(THREADSTATE_DEFAULTINPUTCONTEXT as DWORD) as HIMC;
            Imm32DestroyInputContext(hIMC, hKL, (TRUE) as BOOL);
        }
        DLL_PROCESS_DETACH => {
            RtlDeleteCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
