#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread, ZwDeviceIoControlFile, ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::{STATUS_BUFFER_TOO_SMALL, STATUS_NOT_SUPPORTED, STATUS_INVALID_HANDLE};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

const UNICODE_NULL: WCHAR = 0;

// ERR/TRACE/WARN/FIXME — DbgPrint-style no-ops (release).
#[allow(unused_macros)]

// USERTAG_KBDTABLE 'tksU' (tags.h). C multi-char literal is big-endian, so the
// in-memory little-endian bytes are 'U','s','k','t'.
const USERTAG_KBDTABLE: u32 = u32::from_le_bytes(*b"Uskt");

// Keyboard device IOCTLs (ntddkbd.h):
// CTL_CODE(FILE_DEVICE_KEYBOARD=0x0b, fn, METHOD_BUFFERED=0, FILE_ANY_ACCESS=0)
//   = (0x0b << 16) | (fn << 2)
pub const IOCTL_KEYBOARD_QUERY_ATTRIBUTES: u32 = 0x000B_0000; // fn 0x0000
pub const IOCTL_KEYBOARD_SET_INDICATORS: u32 = 0x000B_0008; // fn 0x0002
pub const IOCTL_KEYBOARD_QUERY_INDICATORS: u32 = 0x000B_0040; // fn 0x0010
pub const IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION: u32 = 0x000B_0080; // fn 0x0020

// ---- key-state bit-array macros (input.h) ----
// GET_KS_BYTE(vk)=vk*2/8, GET_KS_DOWN_BIT(vk)=1<<((vk%4)*2),
// GET_KS_LOCK_BIT(vk)=1<<((vk%4)*2+1). Casts vk to u32 to accept any int type.
macro_rules! IS_KEY_DOWN {
    ($ks:expr, $vk:expr) => {{
        let vk = ($vk) as u32;
        if (*($ks).add((vk * 2 / 8) as usize) & (1u8 << ((vk % 4) * 2))) != 0 { 1i32 } else { 0i32 }
    }};
}
macro_rules! IS_KEY_LOCKED {
    ($ks:expr, $vk:expr) => {{
        let vk = ($vk) as u32;
        if (*($ks).add((vk * 2 / 8) as usize) & (1u8 << ((vk % 4) * 2 + 1))) != 0 { 1i32 } else { 0i32 }
    }};
}
macro_rules! SET_KEY_DOWN {
    ($ks:expr, $vk:expr, $down:expr $(,)?) => {{
        let vk = ($vk) as u32;
        let p = ($ks).add((vk * 2 / 8) as usize);
        let bit = 1u8 << ((vk % 4) * 2);
        *p = if ($down) != 0 { *p | bit } else { *p & !bit };
    }};
}
macro_rules! SET_KEY_LOCKED {
    ($ks:expr, $vk:expr, $locked:expr $(,)?) => {{
        let vk = ($vk) as u32;
        let p = ($ks).add((vk * 2 / 8) as usize);
        let bit = 1u8 << ((vk % 4) * 2 + 1);
        *p = if ($locked) != 0 { *p | bit } else { *p & !bit };
    }};
}

// LOWORD/HIWORD — bit extraction macros (windef.h).
#[inline(always)]
fn LOWORD(l: u32) -> u16 { (l & 0xffff) as u16 }
#[inline(always)]
fn HIWORD(l: u32) -> u16 { ((l >> 16) & 0xffff) as u16 }
// PRIMARYLANGID(l) = (WORD)(l) & 0x3ff
#[inline(always)]
fn PRIMARYLANGID(l: u16) -> u16 { l & 0x3ff }

// RtlZeroMemory.
#[inline(always)]
unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// UserHMGetHandle(obj) = (obj)->head.h (ntuser.h). head.h (HANDLE) is the first
// field of the bindgen-flattened THRDESKHEAD, at offset 0.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *const T) -> HWND {
    *(obj as *const HWND)
}

// EngGetTickCount32() = (ULONG)EngGetTickCount() (eng.h). EngGetTickCount is a
// win32k FASTCALL/stdcall export (`_EngGetTickCount@0`).
#[inline(always)]
unsafe fn EngGetTickCount32() -> u32 {
    EngGetTickCount() as u32
}

// gpsi->ptCursor accessor. tagSERVERINFO ends with __bindgen_padding_0
// representing the anonymous PERUSERSERVERINFO member (matches ntmouse.rs).
#[inline(always)]
unsafe fn gpsi_ptCursor() -> POINTL {
    let psi = (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO;
    (*psi).ptCursor
}

// (*pWnd).head.pti — bindgen flattened THRDESKHEAD (no `pti` field). pti sits at
// 2*sizeof(usize) (matches ntmouse.rs head_pti).
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// ---- win32k internal exports referenced by keyboard.c ----
// EngGetTickCount: `_EngGetTickCount@0` (stdcall). W32kGetDefaultKeyLayout:
// `_W32kGetDefaultKeyLayout` (cdecl, no @N).
extern "system" {
    fn EngGetTickCount() -> u32;
}
extern "C" {
    fn W32kGetDefaultKeyLayout() -> PKL;
}
// MsqPostMessage — FASTCALL (msgqueue.h).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR,) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR,) -> BOOL;
}

// ---- additional win32k internal exports referenced by keyboard.c/split2 ----
// SetLastNtError: `@SetLastNtError@4` (FASTCALL). SafeProbeForRead/Write:
// `_SafeProbe*@12` (stdcall). _MmCopyToCaller: cdecl `__MmCopyToCaller`
// (i686-gnu re-prepends `_` to the extern "C" name `_MmCopyToCaller`).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}
extern "C" {
    fn _MmCopyToCaller(dst: *mut c_void, src: *const c_void, len: ULONG) -> NTSTATUS;
    fn wcslen(s: *const u16) -> usize;
}
// co_HOOK_CallHooks / co_UserProcessHotKeys / IntCoalesceMouseMove come from the
// win32k_sys glob (already declared there).

// TAG_STRING ' RTS' (tags.h) -> little-endian bytes 'S','T','R',' '.
const TAG_STRING: u32 = u32::from_le_bytes(*b"STR "); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')

// ---- exported globals defined by keyboard.c (non-static -> #[no_mangle]) ----
// gafAsyncKeyState/gKeyboardInfo are the sole definitions (keyboard.c.obj `B`).
// The win32k_sys glob declares them extern; the local #[no_mangle] definition
// provides the symbol (same pattern as ime.rs gfIMEShowStatus).
#[no_mangle]
pub static mut gafAsyncKeyState: [BYTE; 256 * 2 / 8] = [0; 256 * 2 / 8];
#[no_mangle]
pub static mut gKeyboardInfo: KEYBOARD_ATTRIBUTES = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut gLanguageToggleKeyState: INT = 0;
#[no_mangle]
pub static mut gdwLanguageToggleKey: DWORD = 1;
#[no_mangle]
pub static mut gLayoutToggleKeyState: INT = 0;
#[no_mangle]
pub static mut gdwLayoutToggleKey: DWORD = 2;

// ---- file-private globals dropped by the translator (keyboard.c lines 15-17) ----
// These three are file-private (`static`/local-BSS in keyboard.c.obj).
static mut gafAsyncKeyStateRecentDown: [BYTE; 256 / 8] = [0; 256 / 8];
static mut gpKeyboardIndicatorTrans: PKEYBOARD_INDICATOR_TRANSLATION = core::ptr::null_mut();
static mut gIndicators: KEYBOARD_INDICATOR_PARAMETERS = KEYBOARD_INDICATOR_PARAMETERS { UnitId: 0, LedFlags: 0 };

#[no_mangle]
pub unsafe extern "system" fn InitKeyboardImpl() -> NTSTATUS {
    RtlZeroMemory(
        core::ptr::addr_of_mut!(gafAsyncKeyState) as *mut core::ffi::c_void,
        core::mem::size_of_val(&*(&raw const gafAsyncKeyState)),
    );
    RtlZeroMemory(
        core::ptr::addr_of_mut!(gafAsyncKeyStateRecentDown) as *mut core::ffi::c_void,
        core::mem::size_of_val(&*(&raw const gafAsyncKeyStateRecentDown)),
    );
    RtlZeroMemory(
        core::ptr::addr_of_mut!(gKeyboardInfo) as *mut core::ffi::c_void,
        core::mem::size_of_val(&*(&raw const gKeyboardInfo)),
    );
    gKeyboardInfo.KeyboardIdentifier.Type = 4;
    gKeyboardInfo.NumberOfFunctionKeys = 12;
    STATUS_SUCCESS
}

unsafe extern "system" fn IntKeyboardGetIndicatorTrans(
    hKeyboardDevice: HANDLE,
    ppIndicatorTrans: *mut PKEYBOARD_INDICATOR_TRANSLATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut dwSize: DWORD;
    let mut Block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut pRet: PKEYBOARD_INDICATOR_TRANSLATION;

    dwSize = core::mem::size_of::<KEYBOARD_INDICATOR_TRANSLATION>() as DWORD;

    pRet = ExAllocatePoolWithTag(PagedPool, dwSize as _, USERTAG_KBDTABLE)
        as PKEYBOARD_INDICATOR_TRANSLATION;

    while !pRet.is_null() {
        Status = ZwDeviceIoControlFile(
            hKeyboardDevice,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Block) as _,
            IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION,
            core::ptr::null_mut(),
            0,
            pRet as *mut core::ffi::c_void,
            dwSize,
        );

        if Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }

        ExFreePoolWithTag(pRet as *mut core::ffi::c_void, USERTAG_KBDTABLE);

        dwSize += core::mem::size_of::<KEYBOARD_INDICATOR_TRANSLATION>() as DWORD;

        pRet = ExAllocatePoolWithTag(PagedPool, dwSize as _, USERTAG_KBDTABLE)
            as PKEYBOARD_INDICATOR_TRANSLATION;
    }

    if pRet.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(pRet as *mut core::ffi::c_void, USERTAG_KBDTABLE);
        return Status;
    }

    *ppIndicatorTrans = pRet;
    Status
}

unsafe extern "system" fn IntKeyboardUpdateLeds(
    hKeyboardDevice: HANDLE,
    wVk: WORD,
    wScanCode: WORD,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut i: UINT;
    let mut LedFlag: USHORT = 0;
    let mut Block: _IO_STATUS_BLOCK = core::mem::zeroed();

    if gpKeyboardIndicatorTrans.is_null() {
        return STATUS_NOT_SUPPORTED;
    }

    match wVk as u32 {
        VK_CAPITAL => LedFlag = KEYBOARD_CAPS_LOCK_ON as USHORT,
        VK_NUMLOCK => LedFlag = KEYBOARD_NUM_LOCK_ON as USHORT,
        VK_SCROLL => LedFlag = KEYBOARD_SCROLL_LOCK_ON as USHORT,
        _ => {
            i = 0;
            while i < (*gpKeyboardIndicatorTrans).NumberOfIndicatorKeys as u32 {
                if (*gpKeyboardIndicatorTrans).IndicatorList.as_ptr().add(i as usize).read().MakeCode == wScanCode {
                    LedFlag = (*gpKeyboardIndicatorTrans).IndicatorList.as_ptr().add(i as usize).read().IndicatorFlags;
                    break;
                }
                i += 1;
            }
        }
    }

    if LedFlag != 0 {
        gIndicators.LedFlags ^= LedFlag;

        Status = ZwDeviceIoControlFile(
            hKeyboardDevice,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Block) as _,
            IOCTL_KEYBOARD_SET_INDICATORS,
            core::ptr::addr_of_mut!(gIndicators) as *mut _,
            core::mem::size_of_val(&*(&raw const gIndicators)) as u32,
            core::ptr::null_mut(),
            0,
        );

        return Status;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn UserInitKeyboard(hKeyboardDevice: HANDLE) {
    let mut Status: NTSTATUS;
    let mut Block: _IO_STATUS_BLOCK = core::mem::zeroed();

    IntKeyboardGetIndicatorTrans(
        hKeyboardDevice,
        core::ptr::addr_of_mut!(gpKeyboardIndicatorTrans),
    );

    Status = ZwDeviceIoControlFile(
        hKeyboardDevice,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Block) as _,
        IOCTL_KEYBOARD_QUERY_INDICATORS,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(gIndicators) as *mut _,
        core::mem::size_of_val(&*(&raw const gIndicators)) as u32,
    );

    if !NT_SUCCESS(Status) {
        gIndicators.LedFlags = 0;
        gIndicators.UnitId = 0;
    }

    SET_KEY_LOCKED!(
        (*(&raw mut gafAsyncKeyState)).as_mut_ptr(),
        VK_CAPITAL as i32,
        (gIndicators.LedFlags & KEYBOARD_CAPS_LOCK_ON as u16) as i32,
    );
    SET_KEY_LOCKED!(
        (*(&raw mut gafAsyncKeyState)).as_mut_ptr(),
        VK_NUMLOCK as i32,
        (gIndicators.LedFlags & KEYBOARD_NUM_LOCK_ON as u16) as i32,
    );
    SET_KEY_LOCKED!(
        (*(&raw mut gafAsyncKeyState)).as_mut_ptr(),
        VK_SCROLL as i32,
        (gIndicators.LedFlags & KEYBOARD_SCROLL_LOCK_ON as u16) as i32,
    );

    Status = ZwDeviceIoControlFile(
        hKeyboardDevice,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Block) as _,
        IOCTL_KEYBOARD_QUERY_ATTRIBUTES,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(gKeyboardInfo) as *mut _,
        core::mem::size_of_val(&*(&raw const gKeyboardInfo)) as u32,
    );

    if !NT_SUCCESS(Status) {}
}

unsafe fn IntSimplifyVk(wVk: WORD) -> WORD {
    match wVk as u32 {
        VK_LSHIFT | VK_RSHIFT => VK_SHIFT as WORD,
        VK_LCONTROL | VK_RCONTROL => VK_CONTROL as WORD,
        VK_LMENU | VK_RMENU => VK_MENU as WORD,
        _ => wVk,
    }
}

unsafe fn IntFixVk(wVk: WORD, bExt: BOOL) -> WORD {
    match wVk as u32 {
        VK_SHIFT => if bExt != 0 { VK_RSHIFT as WORD } else { VK_LSHIFT as WORD },
        VK_CONTROL => if bExt != 0 { VK_RCONTROL as WORD } else { VK_LCONTROL as WORD },
        VK_MENU => if bExt != 0 { VK_RMENU as WORD } else { VK_LMENU as WORD },
        _ => wVk,
    }
}

unsafe fn IntTranslateNumpadKey(wVk: WORD) -> WORD {
    match wVk as u32 {
        VK_INSERT => VK_NUMPAD0 as WORD,
        VK_END => VK_NUMPAD1 as WORD,
        VK_DOWN => VK_NUMPAD2 as WORD,
        VK_NEXT => VK_NUMPAD3 as WORD,
        VK_LEFT => VK_NUMPAD4 as WORD,
        VK_CLEAR => VK_NUMPAD5 as WORD,
        VK_RIGHT => VK_NUMPAD6 as WORD,
        VK_HOME => VK_NUMPAD7 as WORD,
        VK_UP => VK_NUMPAD8 as WORD,
        VK_PRIOR => VK_NUMPAD9 as WORD,
        VK_DELETE => VK_DECIMAL as WORD,
        _ => wVk,
    }
}

unsafe fn IntGetModBits(pKbdTbl: PKBDTABLES, pKeyState: PBYTE) -> DWORD {
    let mut i: DWORD = 0;
    let mut dwModBits: DWORD = 0;

    while (*(*(*pKbdTbl).pCharModifiers).pVkToBit.add(i as usize)).Vk != 0 {
        if IS_KEY_DOWN!(pKeyState, (*(*(*pKbdTbl).pCharModifiers).pVkToBit.add(i as usize)).Vk) != 0 {
            dwModBits |= (*(*(*pKbdTbl).pCharModifiers).pVkToBit.add(i as usize)).ModBits as DWORD;
        }
        i += 1;
    }

    TRACE!(b"Current Mod Bits: %lx\n\0".as_ptr() as *const i8, dwModBits);

    dwModBits
}

unsafe fn IntTranslateChar(
    wVirtKey: WORD,
    pKeyState: PBYTE,
    pbDead: PBOOL,
    pbLigature: PBOOL,
    pwcTranslatedChar: PWCHAR,
    pKbdTbl: PKBDTABLES,
) -> BOOL {
    let mut pVkToVchTbl: PVK_TO_WCHAR_TABLE;
    let mut pVkToVch: PVK_TO_WCHARS10;
    let mut i: DWORD;
    let mut dwModBits: DWORD;
    let mut dwVkModBits: DWORD;
    let mut dwModNumber: DWORD = 0;
    let mut wch: WCHAR;
    let bAltGr: BOOL;
    let wCaplokAttr: WORD;

    dwModBits = if !pKeyState.is_null() {
        IntGetModBits(pKbdTbl, pKeyState)
    } else {
        0
    };
    bAltGr = if !pKeyState.is_null()
        && ((*pKbdTbl).fLocaleFlags & KLLF_ALTGR) != 0
        && IS_KEY_DOWN!(pKeyState, VK_RMENU as WORD) != 0
    {
        1
    } else {
        0
    };
    wCaplokAttr = if bAltGr != 0 { CAPLOKALTGR as WORD } else { CAPLOK as WORD };

    TRACE!(b"TryToTranslate: %04x %x\n\0".as_ptr() as *const i8, wVirtKey as u32, dwModBits);

    if (dwModBits & (KBDALT | KBDCTRL)) == KBDALT {
        dwModBits &= !KBDALT;
    }

    if dwModBits > (*(*pKbdTbl).pCharModifiers).wMaxModBits as DWORD {
        TRACE!(
            b"dwModBits %x > wMaxModBits %x\n\0".as_ptr() as *const i8,
            dwModBits,
            (*(*pKbdTbl).pCharModifiers).wMaxModBits as u32,
        );
        return 0;
    }

    i = 0;
    while !(*(*pKbdTbl).pVkToWcharTable.add(i as usize)).pVkToWchars.is_null() {
        pVkToVchTbl = core::ptr::addr_of_mut!(*(*pKbdTbl).pVkToWcharTable.add(i as usize));
        pVkToVch = (*pVkToVchTbl).pVkToWchars as PVK_TO_WCHARS10;
        while (*pVkToVch).VirtualKey != 0 {
            if wVirtKey == ((*pVkToVch).VirtualKey as WORD & 0xFF) {
                dwVkModBits = dwModBits;

                if ((*pVkToVch).Attributes as WORD & wCaplokAttr) != 0
                    && !pKeyState.is_null()
                    && IS_KEY_LOCKED!(pKeyState, VK_CAPITAL as WORD) != 0
                {
                    dwVkModBits ^= KBDSHIFT;
                }

                if dwVkModBits > (*(*pKbdTbl).pCharModifiers).wMaxModBits as DWORD {
                    break;
                }

                dwModNumber = (*(*pKbdTbl).pCharModifiers).ModNumber.as_ptr().add(dwVkModBits as usize).read() as DWORD;
                if dwModNumber >= (*pVkToVchTbl).nModifications as DWORD {
                    TRACE!(
                        b"dwModNumber %u >= nModifications %u\n\0".as_ptr() as *const i8,
                        dwModNumber,
                        (*pVkToVchTbl).nModifications as u32,
                    );
                    break;
                }

                wch = (*pVkToVch).wch[dwModNumber as usize];
                if wch == WCH_NONE as WCHAR {
                    break;
                }

                *pbDead = if wch == WCH_DEAD as WCHAR { 1 } else { 0 };
                *pbLigature = if wch == WCH_LGTR as WCHAR { 1 } else { 0 };
                *pwcTranslatedChar = wch;

                TRACE!(
                    b"%lu %04x: dwModNumber %08x Char %04x\n\0".as_ptr() as *const i8,
                    i,
                    wVirtKey as u32,
                    dwModNumber,
                    wch as u32,
                );

                if *pbDead != 0 {
                    pVkToVch = ((pVkToVch as PBYTE).add((*pVkToVchTbl).cbSize as usize)) as PVK_TO_WCHARS10;
                    if (*pVkToVch).VirtualKey != 0xFF {
                        WARN!(b"Found dead key with no trailer in the table.\n\0".as_ptr() as *const i8);
                        WARN!(
                            b"VK: %04x, ADDR: %p\n\0".as_ptr() as *const i8,
                            wVirtKey as u32,
                            pVkToVch,
                        );
                        break;
                    }
                    *pwcTranslatedChar = (*pVkToVch).wch[dwModNumber as usize];
                }
                return 1;
            }
            pVkToVch = ((pVkToVch as PBYTE).add((*pVkToVchTbl).cbSize as usize)) as PVK_TO_WCHARS10;
        }
        i += 1;
    }

    if wVirtKey >= b'A' as WORD
        && wVirtKey <= b'Z' as WORD
        && !pKeyState.is_null()
        && IS_KEY_DOWN!(pKeyState, VK_CONTROL as WORD) != 0
        && IS_KEY_DOWN!(pKeyState, VK_MENU as WORD) == 0
    {
        *pwcTranslatedChar = (wVirtKey - b'A' as WORD) + 1;
        *pbDead = 0;
        *pbLigature = 0;
        return 1;
    }

    0
}

unsafe extern "system" fn IntToUnicodeEx(
    wVirtKey: UINT,
    wScanCode: UINT,
    pKeyState: PBYTE,
    pwszBuff: LPWSTR,
    cchBuff: i32,
    wFlags: UINT,
    pKbdTbl: PKBDTABLES,
) -> i32 {
    static mut wchDead: WCHAR = 0;
    let mut wchTranslatedChar: WCHAR = 0;
    let mut bDead: BOOL = 0;
    let mut bLigature: BOOL = 0;
    let mut iRet: i32 = 0;

    if IntTranslateChar(
        wVirtKey as WORD,
        pKeyState,
        &mut bDead,
        &mut bLigature,
        &mut wchTranslatedChar,
        pKbdTbl,
    ) == 0
    {
        return 0;
    }

    if bLigature != 0 {
        return 0;
    }

    if wchDead != 0 {
        if !(*pKbdTbl).pDeadKey.is_null() {
            let mut i: UINT = 0;
            while (*(*pKbdTbl).pDeadKey.offset(i as isize)).dwBoth != 0 {
                let wchFirst = ((*(*pKbdTbl).pDeadKey.offset(i as isize)).dwBoth >> 16) as WCHAR;
                let wchSecond = ((*(*pKbdTbl).pDeadKey.offset(i as isize)).dwBoth & 0xFFFF) as WCHAR;
                if wchFirst == wchDead && wchSecond == wchTranslatedChar {
                    wchTranslatedChar = (*(*pKbdTbl).pDeadKey.offset(i as isize)).wchComposed;
                    wchDead = 0;
                    bDead = 0;
                    break;
                }
                i += 1;
            }
        }
    }

    if wchDead != 0 {
        if cchBuff > iRet {
            *pwszBuff.offset(iRet as isize) = wchDead;
            iRet += 1;
        }
        bDead = 0;
    }

    if cchBuff > iRet {
        *pwszBuff.offset(iRet as isize) = wchTranslatedChar;
        iRet += 1;
    }

    wchDead = if bDead != 0 { wchTranslatedChar } else { 0 };

    if bDead != 0 {
        -iRet
    } else {
        iRet
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntVkToVsc(wVk: WORD, pKbdTbl: PKBDTABLES) -> WORD {
    IntVkToVsc_impl(wVk, pKbdTbl)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntVkToVsc(wVk: WORD, pKbdTbl: PKBDTABLES) -> WORD {
    IntVkToVsc_impl(wVk, pKbdTbl)
}

unsafe fn IntVkToVsc_impl(wVk: WORD, pKbdTbl: PKBDTABLES) -> WORD {
    let mut i: u32;

    /* Check standard keys first */
    i = 0;
    while i < (*pKbdTbl).bMaxVSCtoVK as u32 {
        if (*(*pKbdTbl).pusVSCtoVK.add(i as usize) & 0xFF) == wVk {
            return i as WORD;
        }
        i += 1;
    }

    /* Check extended keys now */
    i = 0;
    while (*(*pKbdTbl).pVSCtoVK_E0.add(i as usize)).Vsc != 0 {
        if ((*(*pKbdTbl).pVSCtoVK_E0.add(i as usize)).Vk & 0xFF) == wVk {
            return 0xE000u16 | (*(*pKbdTbl).pVSCtoVK_E0.add(i as usize)).Vsc as u16;
        }
        i += 1;
    }

    i = 0;
    while (*(*pKbdTbl).pVSCtoVK_E1.add(i as usize)).Vsc != 0 {
        if ((*(*pKbdTbl).pVSCtoVK_E1.add(i as usize)).Vk & 0xFF) == wVk {
            return 0xE100u16 | (*(*pKbdTbl).pVSCtoVK_E1.add(i as usize)).Vsc as u16;
        }
        i += 1;
    }

    /* Virtual key has not been found */
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntVscToVk(wScanCode: WORD, pKbdTbl: PKBDTABLES) -> WORD {
    IntVscToVk_impl(wScanCode, pKbdTbl)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntVscToVk(wScanCode: WORD, pKbdTbl: PKBDTABLES) -> WORD {
    IntVscToVk_impl(wScanCode, pKbdTbl)
}

unsafe fn IntVscToVk_impl(wScanCode: WORD, pKbdTbl: PKBDTABLES) -> WORD {
    let mut i: u32;
    let mut wVk: WORD = 0;

    if (wScanCode & 0xFF00) == 0xE000 {
        i = 0;
        while (*(*pKbdTbl).pVSCtoVK_E0.offset(i as isize)).Vsc != 0 {
            if (*(*pKbdTbl).pVSCtoVK_E0.offset(i as isize)).Vsc == (wScanCode & 0xFF) as u8 {
                wVk = (*(*pKbdTbl).pVSCtoVK_E0.offset(i as isize)).Vk;
            }
            i += 1;
        }
    } else if (wScanCode & 0xFF00) == 0xE100 {
        i = 0;
        while (*(*pKbdTbl).pVSCtoVK_E1.offset(i as isize)).Vsc != 0 {
            if (*(*pKbdTbl).pVSCtoVK_E1.offset(i as isize)).Vsc == (wScanCode & 0xFF) as u8 {
                wVk = (*(*pKbdTbl).pVSCtoVK_E1.offset(i as isize)).Vk;
            }
            i += 1;
        }
    } else if wScanCode < (*pKbdTbl).bMaxVSCtoVK as u16 {
        wVk = *(*pKbdTbl).pusVSCtoVK.offset(wScanCode as isize);
    }

    if wVk != 0xFF { wVk } else { 0 }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntVkToChar(wVk: WORD, pKbdTbl: PKBDTABLES) -> WCHAR {
    let mut wch: WCHAR = 0;
    let mut bDead: BOOL = 0;
    let mut bLigature: BOOL = 0;

    if IntTranslateChar(
        wVk,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(bDead),
        core::ptr::addr_of_mut!(bLigature),
        core::ptr::addr_of_mut!(wch),
        pKbdTbl,
    ) != 0
    {
        return wch;
    }

    0
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntVkToChar(wVk: WORD, pKbdTbl: PKBDTABLES) -> WCHAR {
    let mut wch: WCHAR = 0;
    let mut bDead: BOOL = 0;
    let mut bLigature: BOOL = 0;

    if IntTranslateChar(
        wVk,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(bDead),
        core::ptr::addr_of_mut!(bLigature),
        core::ptr::addr_of_mut!(wch),
        pKbdTbl,
    ) != 0
    {
        return wch;
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetAsyncKeyState(Key: INT) -> SHORT {
    let mut wRet: WORD = 0;

    if Key >= 0x100 || Key < 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    UserEnterExclusive();

    if (gafAsyncKeyState[(Key / 8) as usize] & (1 << (Key % 8))) != 0 {
        wRet |= 0x8000;
    }
    if (gafAsyncKeyStateRecentDown[(Key / 8) as usize] & (1 << (Key % 8))) != 0 {
        wRet |= 0x1;
    }
    gafAsyncKeyStateRecentDown[(Key / 8) as usize] &= !(1 << (Key % 8));

    UserLeave();

    wRet as SHORT
}

unsafe extern "system" fn UpdateAsyncKeyState(wVk: WORD, bIsDown: BOOL) {
    if bIsDown != 0 {
        if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), wVk) == 0 {
            let locked = IS_KEY_LOCKED!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), wVk);
            SET_KEY_LOCKED!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), wVk, (locked == 0) as u8);
        }

        SET_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), wVk, 1u8);
        gafAsyncKeyStateRecentDown[(wVk / 8) as usize] |= 1 << (wVk % 8);
    } else {
        SET_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), wVk, 0u8);
    }
}

unsafe fn co_CallLowLevelKeyboardHook(wVk: WORD, wScanCode: WORD, dwFlags: DWORD, bInjected: BOOL, dwTime: DWORD, dwExtraInfo: DWORD) -> LRESULT {
    let mut KbdHookData: KBDLLHOOKSTRUCT = core::mem::zeroed();
    let uMsg: UINT;

    KbdHookData.vkCode = wVk as DWORD;
    KbdHookData.scanCode = wScanCode as DWORD;
    KbdHookData.flags = 0;
    if dwFlags & KEYEVENTF_EXTENDEDKEY != 0 {
        KbdHookData.flags |= LLKHF_EXTENDED;
    }
    if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MENU as i32) != 0 {
        KbdHookData.flags |= LLKHF_ALTDOWN;
    }
    if dwFlags & KEYEVENTF_KEYUP != 0 {
        KbdHookData.flags |= LLKHF_UP;
    }
    if bInjected != 0 {
        KbdHookData.flags |= LLKHF_INJECTED;
    }
    KbdHookData.time = dwTime;
    KbdHookData.dwExtraInfo = dwExtraInfo as _;

    if dwFlags & KEYEVENTF_KEYUP != 0 {
        uMsg = WM_KEYUP;
    } else if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MENU as i32) != 0
        && IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_CONTROL as i32) == 0
    {
        uMsg = WM_SYSKEYDOWN;
    } else {
        uMsg = WM_KEYDOWN;
    }

    co_HOOK_CallHooks(
        WH_KEYBOARD_LL as i32,
        HC_ACTION as i32,
        uMsg as WPARAM,
        core::ptr::addr_of_mut!(KbdHookData) as LPARAM,
    )
}

unsafe fn SnapWindow(mut hWnd: HWND) {
    let mut hbm: HBITMAP = core::ptr::null_mut();
    let hbmOld: HBITMAP;
    let mut hdc: HDC = core::ptr::null_mut();
    let hdcMem: HDC;
    let mut scd: SETCLIPBDATA = core::mem::zeroed();
    let cx: INT;
    let cy: INT;
    let mut pWnd: PWND = core::ptr::null_mut();

    'cleanup: {
        if hWnd.is_null() {
            hWnd = IntGetDesktopWindow();
        }

        pWnd = UserGetWindowObject(hWnd);
        if pWnd.is_null() {
            break 'cleanup;
        }

        hdc = UserGetDCEx(pWnd, core::ptr::null_mut(), DCX_USESTYLE | DCX_WINDOW);
        if hdc.is_null() {
            break 'cleanup;
        }

        cx = (*pWnd).rcWindow.right - (*pWnd).rcWindow.left;
        cy = (*pWnd).rcWindow.bottom - (*pWnd).rcWindow.top;

        hbm = NtGdiCreateCompatibleBitmap(hdc, cx, cy);
        if hbm.is_null() {
            break 'cleanup;
        }

        hdcMem = NtGdiCreateCompatibleDC(hdc);
        if hdcMem.is_null() {
            break 'cleanup;
        }

        hbmOld = NtGdiSelectBitmap(hdcMem, hbm);
        NtGdiBitBlt(hdcMem, 0, 0, cx, cy, hdc, 0, 0, SRCCOPY, CLR_INVALID, 0);
        NtGdiSelectBitmap(hdcMem, hbmOld);
        IntGdiDeleteDC(hdcMem, 0i32);

        if UserOpenClipboard(core::ptr::null_mut()) != 0 {
            UserEmptyClipboard();
            scd.fIncSerialNumber = 1;
            scd.fGlobalHandle = 0;
            if !UserSetClipboardData(CF_BITMAP, hbm as *mut c_void, &mut scd).is_null() {
                hbm = core::ptr::null_mut();
            }
            UserCloseClipboard();
        }
    }

    if !hbm.is_null() {
        GreDeleteObject(hbm as *mut c_void);
    }
    if !hdc.is_null() {
        UserReleaseDC(pWnd, hdc, 0i32);
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetNextKL(
    mut pKL: PKL,
    bNext: BOOL,
    bSameLang: BOOL,
) -> PKL {
    let pFirstKL: PKL = pKL;
    let LangID: LANGID = LOWORD((*pKL).hkl as u32) as LANGID;

    loop {
        pKL = if bNext != 0 { (*pKL).pklNext } else { (*pKL).pklPrev };

        if ((*pKL).dwKL_Flags & KL_UNLOAD) == 0
            && bSameLang == ((LangID == LOWORD((*pKL).hkl as u32) as LANGID) as BOOL)
        {
            return pKL;
        }

        if pKL == pFirstKL {
            break;
        }
    }

    pFirstKL
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetNextKL(
    mut pKL: PKL,
    bNext: BOOL,
    bSameLang: BOOL,
) -> PKL {
    let pFirstKL: PKL = pKL;
    let LangID: LANGID = LOWORD((*pKL).hkl as u32) as LANGID;

    loop {
        pKL = if bNext != 0 { (*pKL).pklNext } else { (*pKL).pklPrev };

        if ((*pKL).dwKL_Flags & KL_UNLOAD) == 0
            && bSameLang == ((LangID == LOWORD((*pKL).hkl as u32) as LANGID) as BOOL)
        {
            return pKL;
        }

        if pKL == pFirstKL {
            break;
        }
    }

    pFirstKL
}

unsafe fn IntLanguageToggle(
    pFocusQueue: PUSER_MESSAGE_QUEUE,
    bSameLang: BOOL,
    nKeyState: INT,
) {
    let mut pWnd: PWND;
    let pti: PTHREADINFO;
    let mut pkl: PKL;
    let mut wParam: WPARAM = 0;

    if pFocusQueue.is_null() {
        ERR!("IntLanguageToggle(): NULL pFocusQueue\n");
        return;
    }
    pWnd = (*pFocusQueue).spwndFocus;
    if pWnd.is_null() {
        pWnd = (*pFocusQueue).spwndActive;
    }
    if pWnd.is_null() {
        return;
    }

    pti = head_pti(pWnd);
    pkl = (*pti).KeyboardLayout;

    if nKeyState == INPUTLANGCHANGE_FORWARD as INT {
        pkl = IntGetNextKL(pkl, 1, bSameLang);
    } else if nKeyState == INPUTLANGCHANGE_BACKWARD as INT {
        pkl = IntGetNextKL(pkl, 0, bSameLang);
    }

    if gSystemFS & (*pkl).dwFontSigs != 0 {
        wParam |= INPUTLANGCHANGE_SYSCHARSET as WPARAM;
    }

    UserPostMessage(
        UserHMGetHandle(pWnd),
        WM_INPUTLANGCHANGEREQUEST,
        wParam,
        (*pkl).hkl as LPARAM,
    );
}

unsafe fn IntCheckLanguageToggle(
    pFocusQueue: PUSER_MESSAGE_QUEUE,
    bIsDown: BOOL,
    wVk: WORD,
    pKeyState: PINT,
) -> BOOL {
    if bIsDown != 0 {
        if wVk == VK_LSHIFT as WORD {
            *pKeyState = INPUTLANGCHANGE_FORWARD as INT;
        } else if wVk == VK_RSHIFT as WORD {
            *pKeyState = INPUTLANGCHANGE_BACKWARD as INT;
        } else if wVk == 0 && IS_KEY_DOWN!((*(&raw const gafAsyncKeyState)).as_ptr(), VK_LSHIFT) != 0 {
            *pKeyState = INPUTLANGCHANGE_FORWARD as INT;
        } else if wVk == 0 && IS_KEY_DOWN!((*(&raw const gafAsyncKeyState)).as_ptr(), VK_RSHIFT) != 0 {
            *pKeyState = INPUTLANGCHANGE_BACKWARD as INT;
        } else {
            return 0;
        }
    } else {
        if *pKeyState == 0 {
            return 0;
        }

        IntLanguageToggle(
            pFocusQueue,
            (pKeyState == core::ptr::addr_of_mut!(gLayoutToggleKeyState)) as BOOL,
            *pKeyState,
        );
        *pKeyState = 0;
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntTranslateKbdMessage(lpMsg: LPMSG, flags: UINT) -> BOOL {
    IntTranslateKbdMessage_impl(lpMsg, flags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntTranslateKbdMessage(lpMsg: LPMSG, flags: UINT) -> BOOL {
    IntTranslateKbdMessage_impl(lpMsg, flags)
}

unsafe fn IntTranslateKbdMessage_impl(lpMsg: LPMSG, flags: UINT) -> BOOL {
    let pti: PTHREADINFO;
    let mut cch: INT = 0;
    let mut i: INT;
    let mut wch: [WCHAR; 3] = [0; 3];
    let mut NewMsg: MSG = core::mem::zeroed();
    let pKbdTbl: PKBDTABLES;
    let mut bResult: BOOL = 0;

    match (*lpMsg).message {
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {}
        _ => return 0,
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if (*pti).KeyboardLayout.is_null() {
        let pDefKL: PKL = W32kGetDefaultKeyLayout();
        UserAssignmentLock(
            core::ptr::addr_of_mut!((*pti).KeyboardLayout) as *mut PVOID,
            pDefKL as PVOID,
        );
        if !pDefKL.is_null() {
            (*(*pti).pClientInfo).hKL = (*pDefKL).hkl;
            pKbdTbl = (*(*pDefKL).spkf).pKbdTbl;
        } else {
            (*(*pti).pClientInfo).hKL = core::ptr::null_mut();
            pKbdTbl = core::ptr::null_mut();
        }
    } else {
        pKbdTbl = (*(*(*pti).KeyboardLayout).spkf).pKbdTbl;
    }

    if pKbdTbl.is_null() {
        return 0;
    }

    if (*lpMsg).message != WM_KEYDOWN && (*lpMsg).message != WM_SYSKEYDOWN {
        return 0;
    }

    NewMsg.pt = gpsi_ptCursor();
    NewMsg.hwnd = (*lpMsg).hwnd;
    NewMsg.time = EngGetTickCount32();

    if (*lpMsg).wParam == VK_PACKET as _ {
        NewMsg.message = if (*lpMsg).message == WM_KEYDOWN {
            WM_CHAR
        } else {
            WM_SYSCHAR
        };
        NewMsg.wParam = HIWORD((*lpMsg).lParam as u32) as _;
        NewMsg.lParam = LOWORD((*lpMsg).lParam as u32) as isize as LPARAM;
        MsqPostMessage(pti, &mut NewMsg, 0, QS_KEY, 0, 0);
        return 1;
    }

    cch = IntToUnicodeEx(
        (*lpMsg).wParam as UINT,
        (HIWORD((*lpMsg).lParam as u32) & 0xFF) as UINT,
        (*(*pti).MessageQueue).afKeyState.as_mut_ptr(),
        wch.as_mut_ptr(),
        (core::mem::size_of_val(&wch) / core::mem::size_of_val(&wch[0])) as i32,
        0,
        pKbdTbl,
    );

    if cch != 0 {
        if cch > 0 {
            NewMsg.message = if (*lpMsg).message == WM_KEYDOWN {
                WM_CHAR
            } else {
                WM_SYSCHAR
            };
        } else {
            cch = -cch;
            NewMsg.message = if (*lpMsg).message == WM_KEYDOWN {
                WM_DEADCHAR
            } else {
                WM_SYSDEADCHAR
            };
        }
        NewMsg.lParam = (*lpMsg).lParam;

        i = 0;
        while i < cch {
            NewMsg.wParam = wch[i as usize] as _;
            MsqPostMessage(pti, &mut NewMsg, 0, QS_KEY, 0, 0);
            i += 1;
        }
        bResult = 1;
    }

    bResult
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetKeyboardType(dwTypeFlag: DWORD) -> DWORD {
    match dwTypeFlag {
        0 => gKeyboardInfo.KeyboardIdentifier.Type as DWORD,
        1 => gKeyboardInfo.KeyboardIdentifier.Subtype as DWORD,
        2 => gKeyboardInfo.NumberOfFunctionKeys as DWORD,
        _ => 0,
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetKeyboardType(dwTypeFlag: DWORD) -> DWORD {
    match dwTypeFlag {
        0 => gKeyboardInfo.KeyboardIdentifier.Type as DWORD,
        1 => gKeyboardInfo.KeyboardIdentifier.Subtype as DWORD,
        2 => gKeyboardInfo.NumberOfFunctionKeys as DWORD,
        _ => 0,
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserVkKeyScanEx(
    wch: WCHAR,
    dwhkl: HKL,
    bUsehKL: BOOL,
) -> DWORD {
    let pKbdTbl: PKBDTABLES;
    let mut pVkToWchTbl: PVK_TO_WCHAR_TABLE;
    let mut pVkToWch: PVK_TO_WCHARS10;
    let mut pKl: PKL = core::ptr::null_mut();
    let mut i: DWORD;
    let mut dwModBits: DWORD;
    let mut dwModNumber: DWORD;
    let mut Ret: DWORD = (-1i32) as DWORD;

    UserEnterShared();

    if bUsehKL != 0 {
        if !dwhkl.is_null() {
            pKl = UserHklToKbl(dwhkl);
        }
    } else {
        pKl = (*(PsGetCurrentThreadWin32Thread() as PTHREADINFO)).KeyboardLayout;
    }

    'exit: loop {
        if pKl.is_null() {
            break 'exit;
        }

        pKbdTbl = (*(*pKl).spkf).pKbdTbl;

        i = 0;
        while !(*(*pKbdTbl).pVkToWcharTable.offset(i as isize)).pVkToWchars.is_null() {
            pVkToWchTbl = &mut *(*pKbdTbl).pVkToWcharTable.offset(i as isize);
            pVkToWch = (*pVkToWchTbl).pVkToWchars as PVK_TO_WCHARS10;

            while (*pVkToWch).VirtualKey != 0 {
                dwModNumber = 0;
                while dwModNumber < (*pVkToWchTbl).nModifications as DWORD {
                    if (*pVkToWch).wch[dwModNumber as usize] == wch {
                        dwModBits = (*(*pKbdTbl).pCharModifiers).ModNumber.as_ptr().add(dwModNumber as usize).read() as DWORD;
                        Ret = (dwModBits << 8) | ((*pVkToWch).VirtualKey as DWORD & 0xFF);
                        break 'exit;
                    }
                    dwModNumber += 1;
                }
                pVkToWch = ((pVkToWch as *mut BYTE).offset((*pVkToWchTbl).cbSize as isize)) as PVK_TO_WCHARS10;
            }
            i += 1;
        }

        break 'exit;
    }

    UserLeave();
    Ret
}

// MAKELPARAM(l, h) = (WORD)l | ((WORD)h << 16)
#[inline(always)]
fn MAKELPARAM(l: WORD, h: WORD) -> LPARAM {
    (((l as u32) | ((h as u32) << 16)) as i32) as LPARAM
}

// RtlCopyMemory.
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// ====================================================================
// keyboard.c body functions the translator did not emit (ProcessKeyEvent
// from keyboard.c part1; UserSendKeyboardInput, UserProcessKeyboardInput,
// IntMapVirtualKeyEx, NtUserMapVirtualKeyEx, NtUserToUnicodeEx,
// NtUserGetKeyNameText from keyboard.split2.c). Hand-translated verbatim.
// ====================================================================

// ProcessKeyEvent — `_ProcessKeyEvent@24` (NTAPI stdcall).
#[no_mangle]
pub unsafe extern "system" fn ProcessKeyEvent(wVk: WORD, wScanCode: WORD, dwFlags: DWORD, bInjected: BOOL, dwTime: DWORD, dwExtraInfo: DWORD) -> BOOL {
    ProcessKeyEvent_impl(wVk, wScanCode, dwFlags, bInjected, dwTime, dwExtraInfo)
}

unsafe fn ProcessKeyEvent_impl(wVk: WORD, wScanCode: WORD, dwFlags: DWORD, bInjected: BOOL, dwTime: DWORD, dwExtraInfo: DWORD) -> BOOL {
    let mut wSimpleVk: WORD;
    let wFixedVk: WORD;
    let wVk2: WORD;
    let pFocusQueue: PUSER_MESSAGE_QUEUE;
    let mut pti: PTHREADINFO;
    let bExt: BOOL = if dwFlags & KEYEVENTF_EXTENDEDKEY != 0 { 1 } else { 0 };
    let bIsDown: BOOL = if dwFlags & KEYEVENTF_KEYUP != 0 { 0 } else { 1 };
    let bPacket: BOOL = if dwFlags & KEYEVENTF_UNICODE != 0 { 1 } else { 0 };
    let bWasSimpleDown: BOOL;
    let mut bPostMsg: BOOL = 1;
    let bIsSimpleDown: BOOL;
    let mut Msg: MSG = core::mem::zeroed();
    static mut bMenuDownRecently: BOOL = 0;
    let mut bLangToggled: BOOL = 0;
    let mut bExt = bExt;

    /* Get virtual key without shifts (VK_(L|R)* -> VK_*) */
    wSimpleVk = IntSimplifyVk(wVk);

    if PRIMARYLANGID(gusLanguageID) == LANG_JAPANESE as u16 {
        /* Japanese special! */
        if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_SHIFT) != 0 {
            if wSimpleVk == VK_OEM_ATTN as WORD {
                wSimpleVk = VK_CAPITAL as WORD;
            } else if wSimpleVk == VK_OEM_COPY as WORD {
                wSimpleVk = VK_OEM_FINISH as WORD;
            }
        }
    }

    bWasSimpleDown = IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), wSimpleVk);

    /* Update key without shifts */
    wVk2 = IntFixVk(wSimpleVk, (bExt == 0) as BOOL);
    bIsSimpleDown = if bIsDown != 0 || IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), wVk2) != 0 { 1 } else { 0 };
    UpdateAsyncKeyState(wSimpleVk, bIsSimpleDown);

    if bIsDown != 0 {
        /* Update keyboard LEDs */
        IntKeyboardUpdateLeds(ghKeyboardDevice, wSimpleVk, wScanCode);
    }

    /* Call WH_KEYBOARD_LL hook */
    if co_CallLowLevelKeyboardHook(wVk, wScanCode, dwFlags, bInjected, dwTime, dwExtraInfo) != 0 {
        ERR!("Kbd msg dropped by WH_KEYBOARD_LL hook\n");
        bPostMsg = 0;
    }

    /* Check if this is a hotkey */
    if co_UserProcessHotKeys(wSimpleVk, bIsDown) != 0 {
        TRACE!("HotKey Processed\n");
        bPostMsg = 0;
    }

    wFixedVk = IntFixVk(wSimpleVk, bExt); /* LSHIFT + EXT = RSHIFT */
    if wSimpleVk == VK_SHIFT as WORD {
        bExt = 0;
    }

    /* If we have a focus queue, post a keyboard message */
    pFocusQueue = IntGetFocusMessageQueue();

    // Bounded drop trace (2026-07-07 doom-playable triage): a key with no
    // focus queue vanishes silently — make the drop visible.
    if pFocusQueue.is_null() {
        static mut KEY_NOQ_TRACE: u32 = 0;
        if KEY_NOQ_TRACE < 8 {
            KEY_NOQ_TRACE += 1;
            extern "C" {
                fn DbgPrint(Format: *const u8, ...) -> u32;
            }
            DbgPrint(b"[WIN32K] KEYDROP: no focus queue (vk=0x%x)\n\0".as_ptr(), wVk as u32);
        }
    }

    /* If it is F10 or ALT is down and CTRL is up, it's a system key */
    if wVk == VK_F10 as WORD
        || (wSimpleVk == VK_MENU as WORD && bMenuDownRecently != 0)
        || (IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MENU) != 0
            && IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_CONTROL) == 0)
        || (!pFocusQueue.is_null() && (*pFocusQueue).spwndFocus.is_null())
    {
        bMenuDownRecently = 0;
        if bIsDown != 0 {
            Msg.message = WM_SYSKEYDOWN;
            if wSimpleVk == VK_MENU as WORD {
                bMenuDownRecently = 1;
            }
        } else {
            Msg.message = WM_SYSKEYUP;
        }
    } else if bIsDown != 0 {
        Msg.message = WM_KEYDOWN;
    } else {
        Msg.message = WM_KEYUP;
    }

    /* Update async state of not simplified vk here. */
    UpdateAsyncKeyState(wFixedVk, bIsDown);

    /* Alt-Tab/Esc Check. */
    if bIsSimpleDown != 0 && bWasSimpleDown == 0
        && IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MENU) != 0
        && IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_CONTROL) == 0
        && (wVk == VK_ESCAPE as WORD || wVk == VK_TAB as WORD)
    {
        TRACE!("Alt-Tab/Esc Pressed\n");
    }

    /* Check Language/Layout Toggle */
    if gdwLanguageToggleKey == 1 || gdwLanguageToggleKey == 2 {
        if wSimpleVk == VK_SHIFT as WORD {
            let targetKey: UINT = if gdwLanguageToggleKey == 1 { VK_LMENU } else { VK_CONTROL };
            if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), targetKey) != 0 {
                bLangToggled = IntCheckLanguageToggle(pFocusQueue, bIsDown, wVk, core::ptr::addr_of_mut!(gLanguageToggleKeyState));
            }
        } else if (wSimpleVk == VK_MENU as WORD && gdwLanguageToggleKey == 1)
            || (wSimpleVk == VK_CONTROL as WORD && gdwLanguageToggleKey == 2)
        {
            if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_SHIFT) != 0 {
                bLangToggled = IntCheckLanguageToggle(pFocusQueue, bIsDown, 0, core::ptr::addr_of_mut!(gLanguageToggleKeyState));
            }
        }
    }
    if bLangToggled == 0 && (gdwLayoutToggleKey == 1 || gdwLayoutToggleKey == 2) {
        if wSimpleVk == VK_SHIFT as WORD {
            let targetKey: UINT = if gdwLayoutToggleKey == 1 { VK_LMENU } else { VK_CONTROL };
            if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), targetKey) != 0 {
                IntCheckLanguageToggle(pFocusQueue, bIsDown, wVk, core::ptr::addr_of_mut!(gLayoutToggleKeyState));
            }
        } else if (wSimpleVk == VK_MENU as WORD && gdwLayoutToggleKey == 1)
            || (wSimpleVk == VK_CONTROL as WORD && gdwLayoutToggleKey == 2)
        {
            if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_SHIFT) != 0 {
                IntCheckLanguageToggle(pFocusQueue, bIsDown, 0, core::ptr::addr_of_mut!(gLayoutToggleKeyState));
            }
        }
    }

    if bIsDown != 0 && wVk == VK_SNAPSHOT as WORD {
        if !pFocusQueue.is_null()
            && IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MENU) != 0
            && IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_CONTROL) == 0
        {
            let hwnd = if !(*pFocusQueue).spwndActive.is_null() {
                UserHMGetHandle((*pFocusQueue).spwndActive)
            } else {
                core::ptr::null_mut()
            };
            SnapWindow(hwnd);
        } else {
            SnapWindow(core::ptr::null_mut());
        }
    } else if !pFocusQueue.is_null() && bPostMsg != 0 {
        let mut Wnd: PWND = (*pFocusQueue).spwndFocus;

        pti = (*pFocusQueue).ptiKeyboard;

        if Wnd.is_null() && !(*pFocusQueue).spwndActive.is_null() {
            Wnd = (*pFocusQueue).spwndActive;
        }
        if !Wnd.is_null() {
            pti = head_pti(Wnd);
        }

        /* Init message */
        Msg.hwnd = if !Wnd.is_null() { UserHMGetHandle(Wnd) } else { core::ptr::null_mut() };
        Msg.wParam = (wFixedVk & 0xFF) as _;
        Msg.lParam = MAKELPARAM(1, wScanCode);
        Msg.time = dwTime;
        Msg.pt = gpsi_ptCursor();

        if Msg.message == WM_KEYDOWN || Msg.message == WM_SYSKEYDOWN {
            if (Msg.wParam == VK_SHIFT as _ || Msg.wParam == VK_CONTROL as _ || Msg.wParam == VK_MENU as _)
                && IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), Msg.wParam as u32) == 0
            {
                ERR!("Set last input\n");
            }
        }

        /* If it is VK_PACKET, high word of wParam is used for wchar */
        if bPacket == 0 {
            if bExt != 0 {
                Msg.lParam |= (KF_EXTENDED << 16) as LPARAM;
            }
            if IS_KEY_DOWN!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MENU) != 0 {
                Msg.lParam |= (KF_ALTDOWN << 16) as LPARAM;
            }
            if bWasSimpleDown != 0 {
                Msg.lParam |= (KF_REPEAT << 16) as LPARAM;
            }
            if bIsDown == 0 {
                Msg.lParam |= (KF_UP << 16) as LPARAM;
            }
            if (*pFocusQueue).QF_flags & QF_DIALOGACTIVE != 0 {
                Msg.lParam |= (KF_DLGMODE << 16) as LPARAM;
            }
            if !(*pFocusQueue).MenuOwner.is_null() {
                Msg.lParam |= (KF_MENUMODE << 16) as LPARAM;
            }
        }

        if (*pFocusQueue).QF_flags & QF_MOUSEMOVED != 0 {
            IntCoalesceMouseMove(pti);
        }

        /* Post a keyboard message */
        MsqPostMessage(pti, &mut Msg, 1, QS_KEY, 0, dwExtraInfo as LONG_PTR);
    }
    1
}

// UserSendKeyboardInput — `_UserSendKeyboardInput@8` (NTAPI stdcall).
#[no_mangle]
pub unsafe extern "system" fn UserSendKeyboardInput(pKbdInput: *mut KEYBDINPUT, bInjected: BOOL) -> BOOL {
    UserSendKeyboardInput_impl(pKbdInput, bInjected)
}

unsafe fn UserSendKeyboardInput_impl(pKbdInput: *mut KEYBDINPUT, bInjected: BOOL) -> BOOL {
    let wScanCode: WORD;
    let mut wScanCode = (*pKbdInput).wScan;
    let mut wVk: WORD;
    let mut pKl: PKL = core::ptr::null_mut();
    let pKbdTbl: PKBDTABLES;
    let pFocusQueue: PUSER_MESSAGE_QUEUE;
    let dwTime: DWORD;
    let bExt: BOOL = if (*pKbdInput).dwFlags & KEYEVENTF_EXTENDEDKEY != 0 { 1 } else { 0 };
    let _ = wScanCode;

    gppiInputProvider = (*(PsGetCurrentThreadWin32Thread() as PTHREADINFO)).ppi;

    /* Find the target thread whose locale is in effect */
    pFocusQueue = IntGetFocusMessageQueue();

    if !pFocusQueue.is_null() && !(*pFocusQueue).ptiKeyboard.is_null() {
        pKl = (*(*pFocusQueue).ptiKeyboard).KeyboardLayout;
    }

    if pKl.is_null() {
        pKl = W32kGetDefaultKeyLayout();
    }
    if pKl.is_null() {
        ERR!("No keyboard layout!\n");
        return 0;
    }

    pKbdTbl = (*(*pKl).spkf).pKbdTbl;

    wScanCode = (*pKbdInput).wScan;

    if (*pKbdInput).dwFlags & KEYEVENTF_UNICODE != 0 {
        wVk = VK_PACKET as WORD;
    } else {
        wScanCode &= 0x7F;
        if (*pKbdInput).dwFlags & KEYEVENTF_SCANCODE != 0 {
            wVk = IntVscToVk(wScanCode | (if bExt != 0 { 0xE000 } else { 0 }), pKbdTbl);
            if wVk == 0 {
                wVk = 0xFF;
            }
        } else {
            wVk = (*pKbdInput).wVk;
        }
        wVk &= 0xFF;
    }

    if (*pKbdInput).time != 0 {
        dwTime = (*pKbdInput).time;
    } else {
        dwTime = EngGetTickCount32();
    }

    if wVk == VK_RMENU as WORD && ((*pKbdTbl).fLocaleFlags & KLLF_ALTGR) != 0 {
        ProcessKeyEvent(VK_LCONTROL as WORD, 0, (*pKbdInput).dwFlags & KEYEVENTF_KEYUP, bInjected, dwTime, 0);
    }

    ProcessKeyEvent(wVk, wScanCode, (*pKbdInput).dwFlags, bInjected, dwTime, (*pKbdInput).dwExtraInfo as DWORD)
}

/// UserRawInputProcessKeyboardInput — the WM_INPUT half of a keyboard packet (upstream #8819).
unsafe fn UserRawInputProcessKeyboardInput(
    pDeviceInfo: PINPUT_DEVICE_INFO,
    pKbdInputData: *mut KEYBOARD_INPUT_DATA,
    wScanCode: WORD,
    wVk: WORD,
) {
    let mut pti: PTHREADINFO = core::ptr::null_mut();
    let mut hwndTarget: HWND = core::ptr::null_mut();
    let mut wParam: WPARAM = 0;
    let mut kb: RAWKEYBOARD = core::mem::zeroed();
    let mut Msg: MSG = core::mem::zeroed();
    let bIsDown = (*pKbdInputData).Flags & KEY_BREAK as u16 == 0;

    if crate::rawinput::UserGetRawInputTarget(RIM_TYPEKEYBOARD, &mut pti, &mut hwndTarget, &mut wParam) == 0 {
        return;
    }

    kb.MakeCode = wScanCode & 0x7F;
    kb.Flags = if bIsDown { RI_KEY_MAKE } else { RI_KEY_BREAK } as USHORT;
    kb.VKey = wVk & 0xFF; // Note: wVk is simplified by msg queue
    kb.Message = if bIsDown { WM_KEYDOWN } else { WM_KEYUP };
    if (*pKbdInputData).Flags & KEY_E1 as u16 != 0 {
        kb.Flags |= RI_KEY_E1 as USHORT;
    }
    if (*pKbdInputData).Flags & KEY_E0 as u16 != 0 {
        kb.Flags |= RI_KEY_E0 as USHORT;
    }
    kb.ExtraInformation = (*pKbdInputData).ExtraInformation;

    let hRawInput = crate::rawinput::UserCreateRawInput(
        pti,
        RIM_TYPEKEYBOARD,
        pDeviceInfo as HANDLE,
        wParam,
        &kb as *const RAWKEYBOARD as *const c_void,
        core::mem::size_of::<RAWKEYBOARD>() as UINT,
    );
    if hRawInput.is_null() {
        return;
    }

    Msg.hwnd = hwndTarget;
    Msg.message = WM_INPUT;
    Msg.wParam = wParam;
    Msg.lParam = hRawInput as LPARAM;
    Msg.time = EngGetTickCount32();
    let pt = gpsi_ptCursor();
    Msg.pt.x = pt.x;
    Msg.pt.y = pt.y;

    if MsqPostMessage(pti, &mut Msg, 1, QS_RAWINPUT, 0, 0) == 0 {
        crate::rawinput::UserFreeRawInput((*pti).MessageQueue, hRawInput);
    }
}

// UserProcessKeyboardInput — `_UserProcessKeyboardInput@8` (NTAPI stdcall).
#[no_mangle]
pub unsafe extern "system" fn UserProcessKeyboardInput(
    pDeviceInfo: PINPUT_DEVICE_INFO,
    pKbdInputData: *mut KEYBOARD_INPUT_DATA,
) {
    UserProcessKeyboardInput_impl(pDeviceInfo, pKbdInputData)
}

unsafe fn UserProcessKeyboardInput_impl(pDeviceInfo: PINPUT_DEVICE_INFO, pKbdInputData: *mut KEYBOARD_INPUT_DATA) {
    let mut wScanCode: WORD;
    let mut wVk: WORD;
    let mut pKl: PKL = core::ptr::null_mut();
    let pKbdTbl: PKBDTABLES;
    let pFocusQueue: PUSER_MESSAGE_QUEUE;

    /* Calculate scan code with prefix */
    wScanCode = (*pKbdInputData).MakeCode & 0x7F;
    if (*pKbdInputData).Flags & KEY_E0 as u16 != 0 {
        wScanCode |= 0xE000u16;
    }
    if (*pKbdInputData).Flags & KEY_E1 as u16 != 0 {
        wScanCode |= 0xE100u16;
    }

    pFocusQueue = IntGetFocusMessageQueue();

    if !pFocusQueue.is_null() && !(*pFocusQueue).ptiKeyboard.is_null() {
        pKl = (*(*pFocusQueue).ptiKeyboard).KeyboardLayout;
    }

    if pKl.is_null() {
        pKl = W32kGetDefaultKeyLayout();
    }
    if pKl.is_null() {
        // Silent key drop — make it loud (2026-07-07 doom-playable triage).
        extern "C" {
            fn DbgPrint(Format: *const u8, ...) -> u32;
        }
        DbgPrint(b"[WIN32K] KEYDROP: NO keyboard layout (sc=0x%x)\n\0".as_ptr(), wScanCode as u32);
        return;
    }

    pKbdTbl = (*(*pKl).spkf).pKbdTbl;

    wVk = IntVscToVk(wScanCode, pKbdTbl);

    if wVk == 0 {
        // vsc->vk mapping failed — silent drop otherwise (2026-07-07 triage).
        static mut KEY_NOVK_TRACE: u32 = 0;
        if KEY_NOVK_TRACE < 32 {
            KEY_NOVK_TRACE += 1;
            extern "C" {
                fn DbgPrint(Format: *const u8, ...) -> u32;
            }
            DbgPrint(
                b"[WIN32K] KEYDROP: no VK for sc=0x%x (make=0x%x flags=0x%x)\n\0".as_ptr(),
                wScanCode as u32,
                (*pKbdInputData).MakeCode as u32,
                (*pKbdInputData).Flags as u32,
            );
        }
    }

    if crate::rawinput::RawInputEnabled == TRUE as BOOLEAN {
        UserRawInputProcessKeyboardInput(pDeviceInfo, pKbdInputData, wScanCode, wVk);
    }

    if wVk != 0 {
        let mut KbdInput: KEYBDINPUT = core::mem::zeroed();

        /* Support numlock */
        if (wVk & KBDNUMPAD as u16) != 0 && IS_KEY_LOCKED!((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_NUMLOCK) != 0 {
            wVk = IntTranslateNumpadKey(wVk & 0xFF);
        }

        KbdInput.wVk = wVk & 0xFF;
        KbdInput.wScan = wScanCode & 0x7F;
        KbdInput.dwFlags = 0;
        if (*pKbdInputData).Flags & KEY_BREAK as u16 != 0 {
            KbdInput.dwFlags |= KEYEVENTF_KEYUP;
        }

        if wVk & KBDEXT as u16 != 0 {
            KbdInput.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        }
        if wVk == VK_RSHIFT as u16 {
            KbdInput.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        }

        KbdInput.time = 0;
        KbdInput.dwExtraInfo = (*pKbdInputData).ExtraInformation as _;
        UserSendKeyboardInput(&mut KbdInput, 0);

        /* E1 keys don't have break code */
        if (*pKbdInputData).Flags & KEY_E1 as u16 != 0 {
            KbdInput.dwFlags |= KEYEVENTF_KEYUP;
            UserSendKeyboardInput(&mut KbdInput, 0);
        }
    }
}

// IntMapVirtualKeyEx — static helper (private).
unsafe fn IntMapVirtualKeyEx(uCode: UINT, Type: UINT, pKbdTbl: PKBDTABLES) -> UINT {
    let mut uRet: UINT = 0;
    let mut uCode = uCode;

    match Type {
        MAPVK_VK_TO_VSC => {
            uCode = IntFixVk(uCode as WORD, 0) as UINT;
            uRet = IntVkToVsc(uCode as WORD, pKbdTbl) as UINT;
            if uRet > 0xFF {
                uRet = 0;
            }
        }
        MAPVK_VSC_TO_VK => {
            uRet = (IntVscToVk(uCode as WORD, pKbdTbl) & 0xFF) as UINT;
            uRet = IntSimplifyVk(uRet as WORD) as UINT;
        }
        MAPVK_VK_TO_CHAR => {
            uRet = IntVkToChar(uCode as WORD, pKbdTbl) as UINT;
        }
        MAPVK_VSC_TO_VK_EX => {
            uRet = (IntVscToVk(uCode as WORD, pKbdTbl) & 0xFF) as UINT;
        }
        MAPVK_VK_TO_VSC_EX => {
            uRet = IntVkToVsc(uCode as WORD, pKbdTbl) as UINT;
        }
        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ERR!("Wrong type value\n");
        }
    }

    uRet
}

// NtUserMapVirtualKeyEx — `_NtUserMapVirtualKeyEx@16` (NTAPI stdcall).
#[no_mangle]
pub unsafe extern "system" fn NtUserMapVirtualKeyEx(uCode: UINT, uType: UINT, keyboardId: DWORD, dwhkl: HKL) -> UINT {
    NtUserMapVirtualKeyEx_impl(uCode, uType, keyboardId, dwhkl)
}

unsafe fn NtUserMapVirtualKeyEx_impl(uCode: UINT, uType: UINT, _keyboardId: DWORD, dwhkl: HKL) -> UINT {
    let mut pKbdTbl: PKBDTABLES = core::ptr::null_mut();
    let mut ret: UINT = 0;

    UserEnterShared();

    if dwhkl.is_null() {
        let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        if !pti.is_null() && !(*pti).KeyboardLayout.is_null() {
            pKbdTbl = (*(*(*pti).KeyboardLayout).spkf).pKbdTbl;
        }
    } else {
        let pKl: PKL = UserHklToKbl(dwhkl);
        if !pKl.is_null() {
            pKbdTbl = (*(*pKl).spkf).pKbdTbl;
        }
    }

    if !pKbdTbl.is_null() {
        ret = IntMapVirtualKeyEx(uCode, uType, pKbdTbl);
    }

    UserLeave();
    ret
}

// NtUserToUnicodeEx — `_NtUserToUnicodeEx@28` (NTAPI stdcall).
#[no_mangle]
pub unsafe extern "system" fn NtUserToUnicodeEx(
    wVirtKey: UINT, wScanCode: UINT, pKeyStateUnsafe: PBYTE,
    pwszBuffUnsafe: LPWSTR, cchBuff: INT, wFlags: UINT, dwhkl: HKL,
) -> INT {
    NtUserToUnicodeEx_impl(wVirtKey, wScanCode, pKeyStateUnsafe, pwszBuffUnsafe, cchBuff, wFlags, dwhkl)
}

unsafe fn NtUserToUnicodeEx_impl(
    wVirtKey: UINT, wScanCode: UINT, pKeyStateUnsafe: PBYTE,
    pwszBuffUnsafe: LPWSTR, cchBuff: INT, wFlags: UINT, dwhkl: HKL,
) -> INT {
    let pti: PTHREADINFO;
    let mut afKeyState: [BYTE; 256 * 2 / 8] = [0; 256 * 2 / 8];
    let mut pwszBuff: PWCHAR = core::ptr::null_mut();
    let mut iRet: INT = 0;
    let mut pKl: PKL = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    /* Return 0 if SC_KEY_UP bit is set */
    if wScanCode & SC_KEY_UP != 0 || wVirtKey >= 0x100 {
        ERR!("Invalid parameter\n");
        return 0;
    }

    Status = SafeProbeForRead(pKeyStateUnsafe as PVOID, (256 * core::mem::size_of::<BYTE>()) as _, 1);
    if !NT_SUCCESS(Status) {
        ERR!("Cannot copy key state\n");
        SetLastNtError(Status);
        return 0;
    } else {
        /* Probe and copy key state to smaller bitmap. The probe is validate-only
         * (USE_DUMMY_PSEH) and the C had probe AND all 512 loads inside one
         * _SEH2_TRY, so the array comes across in ONE fault-tolerant copy and the
         * loop then reads kernel memory. */
        let mut SafeKeyState: [u8; 256] = [0; 256];
        if !crate::copy::capture_from_user(
            SafeKeyState.as_mut_ptr() as *mut core::ffi::c_void,
            pKeyStateUnsafe as *const core::ffi::c_void,
            SafeKeyState.len(),
        ) {
            ERR!("Cannot copy key state\n");
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            return 0;
        }
        let mut i: INT = 0;
        while i < 256 {
            if SafeKeyState[i as usize] & KS_DOWN_BIT as u8 != 0 {
                SET_KEY_DOWN!(afKeyState.as_mut_ptr(), i, 1);
            }
            if SafeKeyState[i as usize] & KS_LOCK_BIT as u8 != 0 {
                SET_KEY_LOCKED!(afKeyState.as_mut_ptr(), i, 1);
            }
            i += 1;
        }
    }

    pwszBuff = ExAllocatePoolWithTag(NonPagedPool, (core::mem::size_of::<WCHAR>() as INT * cchBuff) as _, TAG_STRING) as PWCHAR;
    if pwszBuff.is_null() {
        ERR!("ExAllocatePoolWithTag failed\n");
        return 0;
    }
    RtlZeroMemory(pwszBuff as *mut c_void, core::mem::size_of::<WCHAR>() * cchBuff as usize);

    UserEnterExclusive(); // Note: We modify wchDead static variable

    if !dwhkl.is_null() {
        pKl = UserHklToKbl(dwhkl);
    }

    if pKl.is_null() {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        pKl = (*pti).KeyboardLayout;
    }

    if !pKl.is_null() {
        iRet = IntToUnicodeEx(
            wVirtKey, wScanCode, afKeyState.as_mut_ptr(),
            pwszBuff, cchBuff, wFlags, (*(*pKl).spkf).pKbdTbl,
        );

        if iRet != 0 {
            Status = _MmCopyToCaller(pwszBuffUnsafe as *mut c_void, pwszBuff as *const c_void, (cchBuff * core::mem::size_of::<WCHAR>() as INT) as ULONG);
        }
    } else {
        ERR!("No keyboard layout ?!\n");
        Status = STATUS_INVALID_HANDLE;
    }

    ExFreePoolWithTag(pwszBuff as *mut c_void, TAG_STRING);

    if !NT_SUCCESS(Status) {
        iRet = 0;
        SetLastNtError(Status);
    }

    UserLeave();
    iRet
}

// NtUserGetKeyNameText — `_NtUserGetKeyNameText@12` (NTAPI stdcall).
#[no_mangle]
pub unsafe extern "system" fn NtUserGetKeyNameText(lParam: LONG, lpString: LPWSTR, cchSize: INT) -> DWORD {
    NtUserGetKeyNameText_impl(lParam, lpString, cchSize)
}

unsafe fn NtUserGetKeyNameText_impl(lParam: LONG, lpString: LPWSTR, cchSize: INT) -> DWORD {
    let pti: PTHREADINFO;
    let mut i: DWORD;
    let mut dwRet: DWORD = 0;
    let mut cchKeyName: SIZE_T;
    let mut wScanCode: WORD = ((lParam >> 16) & 0xFF) as WORD;
    let mut bExtKey: BOOL = if (HIWORD(lParam as u32) as u32 & KF_EXTENDED) != 0 { 1 } else { 0 };
    let pKbdTbl: PKBDTABLES;
    let mut pKeyNames: *mut VSC_LPWSTR; // = NULL
    let mut pKeyName: *const WCHAR = core::ptr::null();
    let mut KeyNameBuf: [WCHAR; 2] = [0; 2];
    let Status: NTSTATUS;

    'cleanup: {
        UserEnterShared();

        /* Get current keyboard layout */
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        pKbdTbl = if !pti.is_null() { (*(*(*pti).KeyboardLayout).spkf).pKbdTbl } else { core::ptr::null_mut() };

        if pKbdTbl.is_null() || cchSize < 1 {
            ERR!("Invalid parameter\n");
            break 'cleanup;
        }

        /* "Do not care" flag */
        if lParam & LP_DO_NOT_CARE_BIT as LONG != 0 {
            if wScanCode == SCANCODE_RSHIFT as WORD && bExtKey == 0 {
                wScanCode = SCANCODE_LSHIFT as WORD;
            } else if wScanCode == SCANCODE_CTRL as WORD || wScanCode == SCANCODE_ALT as WORD {
                bExtKey = 0;
            }
        }

        pKeyNames = if bExtKey != 0 { (*pKbdTbl).pKeyNamesExt } else { (*pKbdTbl).pKeyNames };

        i = 0;
        while !(*pKeyNames.add(i as usize)).pwsz.is_null() {
            if (*pKeyNames.add(i as usize)).vsc as WORD == wScanCode {
                pKeyName = (*pKeyNames.add(i as usize)).pwsz;
                break;
            }
            i += 1;
        }

        if pKeyName.is_null() {
            let wVk: WORD = IntVscToVk(wScanCode, pKbdTbl);

            if wVk != 0 {
                KeyNameBuf[0] = IntVkToChar(wVk, pKbdTbl);
                KeyNameBuf[1] = 0;
                if KeyNameBuf[0] != 0 {
                    pKeyName = KeyNameBuf.as_ptr();
                }
            }
        }

        if !pKeyName.is_null() {
            cchKeyName = wcslen(pKeyName) as SIZE_T;
            if cchKeyName > (cchSize as SIZE_T - 1) {
                cchKeyName = cchSize as SIZE_T - 1; // Don't count '\0'
            }

            Status = SafeProbeForWrite(lpString as PVOID, ((cchKeyName + 1) * core::mem::size_of::<WCHAR>() as SIZE_T) as _, 1);
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
            } else {
                // Probe is validate-only; the C had probe AND both stores in one
                // _SEH2_TRY. RtlCopyMemory into a user buffer is a plain memcpy.
                let nul: WCHAR = UNICODE_NULL;
                if crate::copy::publish_to_user(
                    lpString as *mut core::ffi::c_void,
                    pKeyName as *const core::ffi::c_void,
                    cchKeyName as usize * core::mem::size_of::<WCHAR>(),
                ) && crate::copy::publish_to_user(
                    lpString.add(cchKeyName as usize) as *mut core::ffi::c_void,
                    core::ptr::addr_of!(nul) as *const core::ffi::c_void,
                    core::mem::size_of::<WCHAR>(),
                ) {
                    dwRet = cchKeyName as DWORD;
                } else {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                }
            }
        } else {
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    UserLeave();
    dwRet
}
