#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{
    PsGetCurrentThreadWin32Thread, ExAllocatePoolWithTag, ExFreePoolWithTag,
};

// Debug-channel macros — no kernel symbol, no-op.
macro_rules! ERR { ($($t:tt)*) => {{}}; }
macro_rules! TRACE { ($($t:tt)*) => {{}}; }
#[allow(unused_macros)]
macro_rules! WARN { ($($t:tt)*) => {{}}; }
#[allow(unused_macros)]
macro_rules! FIXME { ($($t:tt)*) => {{}}; }
#[allow(unused_macros)]
macro_rules! DPRINT1 { ($($t:tt)*) => {{}}; }

// BOOL is i32; bindgen's FALSE/TRUE are u32. Shadow them as BOOL.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// USERTAG_HOTKEY = 'khsU' (big-endian C 4-char literal) -> LE bytes reversed = "Ushk".
const USERTAG_HOTKEY: u32 = u32::from_le_bytes(*b"Ushk");

// PWND_BOTTOM = ((PWND)1) — macro, no symbol.
const PWND_BOTTOM: PWND = 1usize as PWND;

// --- FORCEINLINE / macro helpers (no exported kernel symbol) ---

// UserHMGetHandle(obj) = obj->head.h  (HANDLE at offset 0 of HEAD).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// IsWindowHotKey(pHK) = (pHK)->pti == NULL && (pHK)->id == IDHK_WNDKEY
#[inline(always)]
unsafe fn IsWindowHotKey(pHK: PHOT_KEY) -> bool {
    (*pHK).pti.is_null() && (*pHK).id == IDHK_WNDKEY as INT
}

// head_pti(pWnd) — bindgen flattened THRDESKHEAD (no `pti` field). pti sits at
// 2*sizeof(usize) (= 8 on i386, 16 on amd64/arm64): after HANDLE h + DWORD cLockObj
// plus the head's tail padding on 64-bit.
#[inline(always)]
unsafe fn head_pti<T>(obj: *mut T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// ENHANCED_KEYBOARD(Id) = Id.Type in {2,4,7,8}
#[inline(always)]
fn ENHANCED_KEYBOARD(id: KEYBOARD_ID) -> bool {
    matches!(id.Type, 2 | 4 | 7 | 8)
}

// IS_KEY_DOWN(ks, vk): ((ks)[vk*2/8] & (1 << ((vk%4)*2))) ? TRUE : FALSE
#[inline(always)]
unsafe fn IS_KEY_DOWN(ks: PBYTE, vk: UINT) -> BOOL {
    let byte = *ks.add((vk * 2 / 8) as usize);
    if (byte & (1u8 << ((vk % 4) * 2))) != 0 { TRUE } else { FALSE }
}

// MAKELONG(a,b): (WORD)a | ((DWORD)(WORD)b << 16)
#[inline(always)]
fn MAKELONG(a: UINT, b: UINT) -> LONG {
    ((a & 0xffff) | ((b & 0xffff) << 16)) as LONG
}

// LOBYTE(w) / HIBYTE(w)
#[inline(always)]
fn LOBYTE(w: WPARAM) -> u32 { (w & 0xff) as u32 }
#[inline(always)]
fn HIBYTE(w: WPARAM) -> u32 { ((w >> 8) & 0xff) as u32 }

// ValidateHwndNoErr — real cross-TU export (FASTCALL on i386).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}

// ---- file-scope preamble (translated) ----
pub static mut gphkFirst: PHOT_KEY = core::ptr::null_mut();
pub static mut gfsModOnlyCandidate: UINT = 0;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn SetDebugHotKeys() {
    SetDebugHotKeys_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn SetDebugHotKeys() {
    SetDebugHotKeys_impl()
}

unsafe fn SetDebugHotKeys_impl() {
    let mut vk: UINT = VK_F12 as UINT;
    if !ENHANCED_KEYBOARD(gKeyboardInfo.KeyboardIdentifier) {
        vk = VK_SUBTRACT as UINT;
    }

    UserUnregisterHotKey(PWND_BOTTOM, IDHK_F12 as i32);
    UserUnregisterHotKey(PWND_BOTTOM, IDHK_SHIFTF12 as i32);
    UserRegisterHotKey(PWND_BOTTOM, IDHK_SHIFTF12 as core::ffi::c_int, MOD_SHIFT as UINT, vk);
    UserRegisterHotKey(PWND_BOTTOM, IDHK_F12 as core::ffi::c_int, 0, vk);

    TRACE!(b"Debugger hotkeys set up! If you see this you enabled Debug Prints. Congrats!\n\0".as_ptr());
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetModifiers(pKeyState: PBYTE) -> UINT {
    IntGetModifiers_impl(pKeyState)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetModifiers(pKeyState: PBYTE) -> UINT {
    IntGetModifiers_impl(pKeyState)
}

unsafe fn IntGetModifiers_impl(pKeyState: PBYTE) -> UINT {
    let mut fModifiers: UINT = 0;

    if IS_KEY_DOWN(pKeyState, VK_SHIFT) != 0 {
        fModifiers |= MOD_SHIFT;
    }

    if IS_KEY_DOWN(pKeyState, VK_CONTROL) != 0 {
        fModifiers |= MOD_CONTROL;
    }

    if IS_KEY_DOWN(pKeyState, VK_MENU) != 0 {
        fModifiers |= MOD_ALT;
    }

    if IS_KEY_DOWN(pKeyState, VK_LWIN) != 0 || IS_KEY_DOWN(pKeyState, VK_RWIN) != 0 {
        fModifiers |= MOD_WIN;
    }

    fModifiers
}

unsafe fn IntSwapModHKF(Input: UINT) -> UCHAR {
    ((Input & 2) | ((Input & 1) << 2) | ((Input >> 2) & 1)) as UCHAR
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UnregisterWindowHotKeys(pWnd: PWND) {
    UnregisterWindowHotKeys_impl(pWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UnregisterWindowHotKeys(pWnd: PWND) {
    UnregisterWindowHotKeys_impl(pWnd)
}

unsafe fn UnregisterWindowHotKeys_impl(pWnd: PWND) {
    let mut pHotKey: PHOT_KEY = gphkFirst;
    let mut phkNext: PHOT_KEY;
    let mut pLink: *mut PHOT_KEY = core::ptr::addr_of_mut!(gphkFirst);

    while !pHotKey.is_null() {
        phkNext = (*pHotKey).pNext;

        if (*pHotKey).pWnd == pWnd {
            *pLink = phkNext;
            ExFreePoolWithTag(pHotKey as *mut _, USERTAG_HOTKEY);
        } else {
            pLink = core::ptr::addr_of_mut!((*pHotKey).pNext);
        }

        pHotKey = phkNext;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UnregisterThreadHotKeys(pti: PTHREADINFO) {
    UnregisterThreadHotKeys_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UnregisterThreadHotKeys(pti: PTHREADINFO) {
    UnregisterThreadHotKeys_impl(pti)
}

unsafe fn UnregisterThreadHotKeys_impl(pti: PTHREADINFO) {
    let mut pHotKey: PHOT_KEY = gphkFirst;
    let mut phkNext: PHOT_KEY;
    let mut pLink: *mut PHOT_KEY = core::ptr::addr_of_mut!(gphkFirst);

    while !pHotKey.is_null() {
        phkNext = (*pHotKey).pNext;

        if (*pHotKey).pti == pti {
            *pLink = phkNext;
            ExFreePoolWithTag(pHotKey as *mut core::ffi::c_void, USERTAG_HOTKEY);
        } else {
            pLink = core::ptr::addr_of_mut!((*pHotKey).pNext);
        }

        pHotKey = phkNext;
    }
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn IsHotKey(fsModifiers: UINT, wVk: WORD) -> PHOT_KEY {
    let mut pHotKey: PHOT_KEY = gphkFirst;

    while !pHotKey.is_null() {
        if (*pHotKey).fsModifiers == fsModifiers && (*pHotKey).vk == wVk as UINT {
            return pHotKey;
        }

        pHotKey = (*pHotKey).pNext;
    }

    core::ptr::null_mut()
}

#[cfg(not(target_pointer_width = "32"))]
unsafe extern "C" fn IsHotKey(fsModifiers: UINT, wVk: WORD) -> PHOT_KEY {
    let mut pHotKey: PHOT_KEY = gphkFirst;

    while !pHotKey.is_null() {
        if (*pHotKey).fsModifiers == fsModifiers && (*pHotKey).vk == wVk as UINT {
            return pHotKey;
        }

        pHotKey = (*pHotKey).pNext;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn co_UserProcessHotKeys(wVk: WORD, bIsDown: BOOL) -> BOOL {
    let fModifiers: UINT;
    let mut pHotKey: PHOT_KEY = core::ptr::null_mut();
    let mut pWnd: PWND;
    let DoNotPostMsg: BOOL = FALSE;
    let mut IsModifier: BOOL = FALSE;

    if wVk == VK_SHIFT as WORD || wVk == VK_CONTROL as WORD || wVk == VK_MENU as WORD ||
        wVk == VK_LWIN as WORD || wVk == VK_RWIN as WORD
    {
        IsModifier = TRUE;
    }

    fModifiers = IntGetModifiers(core::ptr::addr_of_mut!(gafAsyncKeyState) as PBYTE);

    if bIsDown != 0 {
        if IsModifier != 0 {
            gfsModOnlyCandidate = fModifiers;
            return FALSE;
        } else {
            pHotKey = IsHotKey(fModifiers, wVk);
            gfsModOnlyCandidate = 0;
        }
    } else {
        if IsModifier != 0 {
            pHotKey = IsHotKey(gfsModOnlyCandidate, 0);
            gfsModOnlyCandidate = 0;
        } else {
            gfsModOnlyCandidate = 0;
            return FALSE;
        }
    }

    if !pHotKey.is_null() {
        if (*pHotKey).id == IDHK_SHIFTF12 || (*pHotKey).id == IDHK_F12 {
            if bIsDown != 0 {
                ERR!(b"Hot key pressed for Debug Activation!\n\0".as_ptr());
            }
            return DoNotPostMsg;
        }

        if (*pHotKey).id == IDHK_WINKEY {
            pWnd = ValidateHwndNoErr((*InputWindowStation).ShellWindow as _);
            if !pWnd.is_null() {
                UserPostMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_TASKLIST as WPARAM, 0);
                co_IntShellHookNotify(HSHELL_TASKMAN as WPARAM, 0, 0);
                return FALSE;
            }
        }

        if (*pHotKey).id == IDHK_SNAP_LEFT ||
            (*pHotKey).id == IDHK_SNAP_RIGHT ||
            (*pHotKey).id == IDHK_SNAP_UP ||
            (*pHotKey).id == IDHK_SNAP_DOWN
        {
            let topWnd: HWND = UserGetForegroundWindow();
            if !topWnd.is_null() {
                UserPostMessage(topWnd, WM_KEYDOWN, wVk as WPARAM, 0);
            }
            return TRUE;
        }

        if (*pHotKey).pWnd.is_null() {
            UserPostThreadMessage((*pHotKey).pti, WM_HOTKEY, (*pHotKey).id as WPARAM,
                                  MAKELONG(fModifiers, wVk as UINT) as LPARAM);
            return TRUE;
        } else {
            pWnd = (*pHotKey).pWnd;
            if pWnd == PWND_BOTTOM {
                if gpqForeground.is_null() {
                    return FALSE;
                }
                pWnd = (*gpqForeground).spwndFocus;
            }

            if !pWnd.is_null() {
                if pWnd == ValidateHwndNoErr((*InputWindowStation).ShellWindow as _) && (*pHotKey).id == SC_TASKLIST as i32 {
                    UserPostMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_TASKLIST as WPARAM, 0);
                    co_IntShellHookNotify(HSHELL_TASKMAN as WPARAM, 0, 0);
                } else if IsWindowHotKey(pHotKey) {
                    if bIsDown != 0 {
                        if !gpqForeground.is_null() && !(*gpqForeground).spwndActive.is_null() {
                            pWnd = (*gpqForeground).spwndActive;
                        }
                        UserPostMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND,
                                        SC_HOTKEY as WPARAM,
                                        UserHMGetHandle((*pHotKey).pWnd) as LPARAM);
                    }
                } else {
                    UserPostMessage(UserHMGetHandle(pWnd), WM_HOTKEY, (*pHotKey).id as WPARAM,
                                    MAKELONG(fModifiers, wVk as UINT) as LPARAM);
                }
                return TRUE;
            }
        }
    }
    FALSE
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DefWndGetHotKey(pWnd: PWND) -> UINT {
    DefWndGetHotKey_impl(pWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DefWndGetHotKey(pWnd: PWND) -> UINT {
    DefWndGetHotKey_impl(pWnd)
}

unsafe fn DefWndGetHotKey_impl(pWnd: PWND) -> UINT {
    let mut pHotKey: PHOT_KEY = gphkFirst;

    while !pHotKey.is_null() {
        if (*pHotKey).pWnd == pWnd
            && (*pHotKey).pti.is_null()
            && (*pHotKey).id == IDHK_WNDKEY as i32
        {
            let vk = (*pHotKey).vk as UINT;
            let mod_ = IntSwapModHKF((*pHotKey).fsModifiers as UINT) as UINT;
            return ((vk & 0xff) | ((mod_ & 0xff) << 8)) as UINT;
        }

        pHotKey = (*pHotKey).pNext;
    }

    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DefWndSetHotKey(pWnd: PWND, wParam: WPARAM) -> INT {
    DefWndSetHotKey_impl(pWnd, wParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DefWndSetHotKey(pWnd: PWND, wParam: WPARAM) -> INT {
    DefWndSetHotKey_impl(pWnd, wParam)
}

unsafe fn DefWndSetHotKey_impl(pWnd: PWND, wParam: WPARAM) -> INT {
    let fsModifiers: UINT = IntSwapModHKF(HIBYTE(wParam) as UINT) as UINT;
    let vk: UINT = LOBYTE(wParam) as UINT;
    let mut pHotKey: PHOT_KEY;
    let mut pLink: *mut PHOT_KEY;
    let mut iRet: INT = 1;

    // A hot key cannot be associated with a child window.
    if (*pWnd).style & WS_CHILD != 0 {
        return 0;
    }

    // VK_ESCAPE, VK_SPACE, VK_TAB and VK_PACKET are invalid hot keys.
    if vk == VK_ESCAPE as UINT
        || vk == VK_SPACE as UINT
        || vk == VK_TAB as UINT
        || vk == VK_PACKET as UINT
    {
        return -1;
    }

    if wParam != 0 {
        pHotKey = gphkFirst;
        while !pHotKey.is_null() {
            if (*pHotKey).fsModifiers as UINT == fsModifiers
                && (*pHotKey).vk as UINT == vk
                && (*pHotKey).pti.is_null()
                && (*pHotKey).id == IDHK_WNDKEY as _
            {
                if (*pHotKey).pWnd != pWnd {
                    iRet = 2; // Another window already has the same hot key.
                }
                break;
            }

            /* Move to the next entry */
            pHotKey = (*pHotKey).pNext;
        }
    }

    pHotKey = gphkFirst;
    pLink = core::ptr::addr_of_mut!(gphkFirst);
    while !pHotKey.is_null() {
        if (*pHotKey).pWnd == pWnd
            && (*pHotKey).pti.is_null()
            && (*pHotKey).id == IDHK_WNDKEY as _
        {
            /* This window has already hotkey registered */
            break;
        }

        /* Move to the next entry */
        pLink = core::ptr::addr_of_mut!((*pHotKey).pNext);
        pHotKey = (*pHotKey).pNext;
    }

    if wParam != 0 {
        if pHotKey.is_null() {
            /* Create new hotkey */
            pHotKey = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<HOT_KEY>() as _,
                USERTAG_HOTKEY,
            ) as PHOT_KEY;
            if pHotKey.is_null() {
                return 0;
            }

            (*pHotKey).pWnd = pWnd;
            (*pHotKey).id = IDHK_WNDKEY as _; // Don't care, these hot keys are unrelated to the hot keys set by RegisterHotKey
            (*pHotKey).pNext = gphkFirst;
            gphkFirst = pHotKey;
        }

        /* A window can only have one hot key. If the window already has a
           hot key associated with it, the new hot key replaces the old one. */
        (*pHotKey).pti = core::ptr::null_mut(); /* IsWindowHotKey */
        (*pHotKey).fsModifiers = fsModifiers as _;
        (*pHotKey).vk = vk as _;
    } else if !pHotKey.is_null() {
        /* Remove hotkey */
        *pLink = (*pHotKey).pNext;
        ExFreePoolWithTag(pHotKey as _, USERTAG_HOTKEY);
    }

    iRet
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserRegisterHotKey(
    pWnd: PWND,
    id: core::ffi::c_int,
    fsModifiers: UINT,
    vk: UINT,
) -> BOOL {
    let pHotKey: PHOT_KEY;
    let pHotKeyThread: PTHREADINFO;

    /* Find the hotkey thread */
    if pWnd.is_null() || pWnd == PWND_BOTTOM {
        pHotKeyThread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    } else {
        pHotKeyThread = head_pti(pWnd);
    }

    /* Ignore the VK_PACKET key since it is not a real keyboard input */
    if vk == VK_PACKET as UINT {
        return FALSE as BOOL;
    }

    /* Check whether we modify an existing hotkey */
    if !IsHotKey(fsModifiers, vk as WORD).is_null() {
        EngSetLastError(ERROR_HOTKEY_ALREADY_REGISTERED);
        return FALSE as BOOL;
    }

    /* Create a new hotkey */
    pHotKey = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<HOT_KEY>() as _,
        USERTAG_HOTKEY,
    ) as PHOT_KEY;
    if pHotKey.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE as BOOL;
    }

    (*pHotKey).pti = pHotKeyThread;
    (*pHotKey).pWnd = pWnd;
    (*pHotKey).fsModifiers = fsModifiers;
    (*pHotKey).vk = vk;
    (*pHotKey).id = id;

    /* Insert the hotkey into the global list */
    (*pHotKey).pNext = gphkFirst;
    gphkFirst = pHotKey;

    TRUE as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserRegisterHotKey(
    pWnd: PWND,
    id: core::ffi::c_int,
    fsModifiers: UINT,
    vk: UINT,
) -> BOOL {
    let pHotKey: PHOT_KEY;
    let pHotKeyThread: PTHREADINFO;

    /* Find the hotkey thread */
    if pWnd.is_null() || pWnd == PWND_BOTTOM {
        pHotKeyThread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    } else {
        pHotKeyThread = head_pti(pWnd);
    }

    /* Ignore the VK_PACKET key since it is not a real keyboard input */
    if vk == VK_PACKET as UINT {
        return FALSE as BOOL;
    }

    /* Check whether we modify an existing hotkey */
    if !IsHotKey(fsModifiers, vk as WORD).is_null() {
        EngSetLastError(ERROR_HOTKEY_ALREADY_REGISTERED);
        return FALSE as BOOL;
    }

    /* Create a new hotkey */
    pHotKey = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<HOT_KEY>() as _,
        USERTAG_HOTKEY,
    ) as PHOT_KEY;
    if pHotKey.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE as BOOL;
    }

    (*pHotKey).pti = pHotKeyThread;
    (*pHotKey).pWnd = pWnd;
    (*pHotKey).fsModifiers = fsModifiers;
    (*pHotKey).vk = vk;
    (*pHotKey).id = id;

    /* Insert the hotkey into the global list */
    (*pHotKey).pNext = gphkFirst;
    gphkFirst = pHotKey;

    TRUE as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserUnregisterHotKey(pWnd: PWND, id: i32) -> BOOL {
    UserUnregisterHotKey_impl(pWnd, id)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserUnregisterHotKey(pWnd: PWND, id: i32) -> BOOL {
    UserUnregisterHotKey_impl(pWnd, id)
}

unsafe fn UserUnregisterHotKey_impl(pWnd: PWND, id: i32) -> BOOL {
    let mut pHotKey: PHOT_KEY = gphkFirst;
    let mut phkNext: PHOT_KEY;
    let mut pLink: *mut PHOT_KEY = core::ptr::addr_of_mut!(gphkFirst);
    let mut bRet: BOOL = 0;

    while !pHotKey.is_null() {
        phkNext = (*pHotKey).pNext;

        if (*pHotKey).pWnd == pWnd && (*pHotKey).id == id {
            *pLink = phkNext;
            ExFreePoolWithTag(pHotKey as *mut _, USERTAG_HOTKEY);

            bRet = 1;
        } else {
            pLink = core::ptr::addr_of_mut!((*pHotKey).pNext);
        }

        pHotKey = phkNext;
    }

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRegisterHotKey(
    hWnd: HWND,
    id: core::ffi::c_int,
    fsModifiers: UINT,
    vk: UINT,
) -> BOOL {
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut bRet: BOOL = 0;

    if (fsModifiers & !(MOD_ALT | MOD_CONTROL | MOD_SHIFT | MOD_WIN)) != 0 {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    UserEnterExclusive();

    'cleanup: {
        if hWnd.is_null() {
            pWnd = core::ptr::null_mut();
        } else {
            pWnd = UserGetWindowObject(hWnd);
            if pWnd.is_null() {
                break 'cleanup;
            }

            if head_pti(pWnd) != gptiCurrent {
                EngSetLastError(ERROR_WINDOW_OF_OTHER_THREAD);
                break 'cleanup;
            }
        }

        bRet = UserRegisterHotKey(pWnd, id, fsModifiers, vk);
    }

    UserLeave();
    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUnregisterHotKey(hWnd: HWND, id: i32) -> BOOL {
    let mut bRet: BOOL = 0;
    let mut pWnd: PWND = core::ptr::null_mut();

    UserEnterExclusive();

    'cleanup: {
        if !hWnd.is_null() {
            pWnd = UserGetWindowObject(hWnd);
            if pWnd.is_null() {
                break 'cleanup;
            }
        }

        bRet = UserUnregisterHotKey(pWnd, id);
    }

    UserLeave();
    bRet
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          HotKey support
//  * FILE:             win32ss/user/ntuser/hotkey.c
//  * PROGRAMER:        Eric Kohl
//  */
//
// /*
//  * FIXME: Hotkey notifications are triggered by keyboard input (physical or programmatically)
//  * and since only desktops on WinSta0 can receive input in seems very wrong to allow
//  * windows/threads on destops not belonging to WinSta0 to set hotkeys (receive notifications).
//  *     -- Gunnar
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(UserHotkey);
//
// /* GLOBALS *******************************************************************/
//
// /*
//  * Hardcoded hotkeys. See http://ivanlef0u.fr/repo/windoz/VI20051005.html (DEAD_LINK)
//  * or https://web.archive.org/web/20170826161432/http://repo.meh.or.id/Windows/VI20051005.html .
//  *
//  * NOTE: The (Shift-)F12 keys are used only for the "UserDebuggerHotKey" setting
//  * which enables setting a key shortcut which, when pressed, establishes a
//  * breakpoint in the code being debugged:
//  * see https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc786263(v=ws.10)
//  * and https://flylib.com/books/en/4.441.1.33/1/ for more details.
//  * By default the key is VK-F12 on a 101-key keyboard, and is VK_SUBTRACT
//  * (hyphen / substract sign) on a 82-key keyboard.
//  */
// /*                       pti   pwnd  modifiers  vk      id  next */
// // HOT_KEY hkF12 =      {NULL, 1,    0,         VK_F12, IDHK_F12,      NULL};
// // HOT_KEY hkShiftF12 = {NULL, 1,    MOD_SHIFT, VK_F12, IDHK_SHIFTF12, &hkF12};
// // HOT_KEY hkWinKey =   {NULL, 1,    MOD_WIN,   0,      IDHK_WINKEY,   &hkShiftF12};
//
// PHOT_KEY gphkFirst = NULL;
// UINT gfsModOnlyCandidate;
//
// /* FUNCTIONS *****************************************************************/
//
// #define IsWindowHotKey(pHK) ( (pHK)->pti == NULL && (pHK)->id == IDHK_WNDKEY )
//
// VOID FASTCALL
// SetDebugHotKeys(VOID)
// {
//     UINT vk = VK_F12;
//     if (!ENHANCED_KEYBOARD(gKeyboardInfo.KeyboardIdentifier))
//         vk = VK_SUBTRACT;
//
//     UserUnregisterHotKey(PWND_BOTTOM, IDHK_F12);
//     UserUnregisterHotKey(PWND_BOTTOM, IDHK_SHIFTF12);
//     UserRegisterHotKey(PWND_BOTTOM, IDHK_SHIFTF12, MOD_SHIFT, vk);
//     UserRegisterHotKey(PWND_BOTTOM, IDHK_F12, 0, vk);
//
//     TRACE("Debugger hotkeys set up! If you see this you enabled Debug Prints. Congrats!\n");
// }
//
// /*
//  * IntGetModifiers
//  *
//  * Returns a value that indicates if the key is a modifier key, and
//  * which one.
//  */
// static
// UINT FASTCALL
// IntGetModifiers(PBYTE pKeyState)
// {
//     UINT fModifiers = 0;
//
//     if (IS_KEY_DOWN(pKeyState, VK_SHIFT))
//         fModifiers |= MOD_SHIFT;
//
//     if (IS_KEY_DOWN(pKeyState, VK_CONTROL))
//         fModifiers |= MOD_CONTROL;
//
//     if (IS_KEY_DOWN(pKeyState, VK_MENU))
//         fModifiers |= MOD_ALT;
//
//     if (IS_KEY_DOWN(pKeyState, VK_LWIN) || IS_KEY_DOWN(pKeyState, VK_RWIN))
//         fModifiers |= MOD_WIN;
//
//     return fModifiers;
// }
//
// /*
//  * IntSwapModHKF
//  *
//  * Maps to/from MOD_/HOTKEYF_ (swaps the SHIFT and ALT bits)
//  */
// static inline
// UCHAR
// IntSwapModHKF(UINT Input)
// {
//     return (Input & 2) | ((Input & 1) << 2) | ((Input >> 2) & 1);
// }
//
// /*
//  * UnregisterWindowHotKeys
//  *
//  * Removes hotkeys registered by specified window on its cleanup
//  */
// VOID FASTCALL
// UnregisterWindowHotKeys(PWND pWnd)
// {
//     PHOT_KEY pHotKey = gphkFirst, phkNext, *pLink = &gphkFirst;
//
//     while (pHotKey)
//     {
//         /* Save next ptr for later use */
//         phkNext = pHotKey->pNext;
//
//         /* Should we delete this hotkey? */
//         if (pHotKey->pWnd == pWnd)
//         {
//             /* Update next ptr for previous hotkey and free memory */
//             *pLink = phkNext;
//             ExFreePoolWithTag(pHotKey, USERTAG_HOTKEY);
//         }
//         else /* This hotkey will stay, use its next ptr */
//             pLink = &pHotKey->pNext;
//
//         /* Move to the next entry */
//         pHotKey = phkNext;
//     }
// }
//
// /*
//  * UnregisterThreadHotKeys
//  *
//  * Removes hotkeys registered by specified thread on its cleanup
//  */
// VOID FASTCALL
// UnregisterThreadHotKeys(PTHREADINFO pti)
// {
//     PHOT_KEY pHotKey = gphkFirst, phkNext, *pLink = &gphkFirst;
//
//     while (pHotKey)
//     {
//         /* Save next ptr for later use */
//         phkNext = pHotKey->pNext;
//
//         /* Should we delete this hotkey? */
//         if (pHotKey->pti == pti)
//         {
//             /* Update next ptr for previous hotkey and free memory */
//             *pLink = phkNext;
//             ExFreePoolWithTag(pHotKey, USERTAG_HOTKEY);
//         }
//         else /* This hotkey will stay, use its next ptr */
//             pLink = &pHotKey->pNext;
//
//         /* Move to the next entry */
//         pHotKey = phkNext;
//     }
// }
//
// /*
//  * IsHotKey
//  *
//  * Checks if given key and modificators have corresponding hotkey
//  */
// static PHOT_KEY FASTCALL
// IsHotKey(UINT fsModifiers, WORD wVk)
// {
//     PHOT_KEY pHotKey = gphkFirst;
//
//     while (pHotKey)
//     {
//         if (pHotKey->fsModifiers == fsModifiers &&
//             pHotKey->vk == wVk)
//         {
//             /* We have found it */
//             return pHotKey;
//         }
//
//         /* Move to the next entry */
//         pHotKey = pHotKey->pNext;
//     }
//
//     return NULL;
// }
//
// /*
//  * co_UserProcessHotKeys
//  *
//  * Sends WM_HOTKEY message if given keys are hotkey
//  */
// BOOL NTAPI
// co_UserProcessHotKeys(WORD wVk, BOOL bIsDown)
// {
//     UINT fModifiers;
//     PHOT_KEY pHotKey;
//     PWND pWnd;
//     BOOL DoNotPostMsg = FALSE;
//     BOOL IsModifier = FALSE;
//
//     if (wVk == VK_SHIFT || wVk == VK_CONTROL || wVk == VK_MENU ||
//         wVk == VK_LWIN || wVk == VK_RWIN)
//     {
//         /* Remember that this was a modifier */
//         IsModifier = TRUE;
//     }
//
//     fModifiers = IntGetModifiers(gafAsyncKeyState);
//
//     if (bIsDown)
//     {
//         if (IsModifier)
//         {
//             /* Modifier key down -- no hotkey trigger, but remember this */
//             gfsModOnlyCandidate = fModifiers;
//             return FALSE;
//         }
//         else
//         {
//             /* Regular key down -- check for hotkey, and reset mod candidates */
//             pHotKey = IsHotKey(fModifiers, wVk);
//             gfsModOnlyCandidate = 0;
//         }
//     }
//     else
//     {
//         if (IsModifier)
//         {
//             /* Modifier key up -- modifier-only keys are triggered here */
//             pHotKey = IsHotKey(gfsModOnlyCandidate, 0);
//             gfsModOnlyCandidate = 0;
//         }
//         else
//         {
//             /* Regular key up -- no hotkey, but reset mod-only candidates */
//             gfsModOnlyCandidate = 0;
//             return FALSE;
//         }
//     }
//
//     if (pHotKey)
//     {
//         TRACE("Hot key pressed (pWnd %p, id %d)\n", pHotKey->pWnd, pHotKey->id);
//
//         /* FIXME: See comment about "UserDebuggerHotKey" on top of this file. */
//         if (pHotKey->id == IDHK_SHIFTF12 || pHotKey->id == IDHK_F12)
//         {
//             if (bIsDown)
//             {
//                 ERR("Hot key pressed for Debug Activation! ShiftF12 = %d or F12 = %d\n",pHotKey->id == IDHK_SHIFTF12 , pHotKey->id == IDHK_F12);
//                 //DoNotPostMsg = co_ActivateDebugger(); // FIXME
//             }
//             return DoNotPostMsg;
//         }
//
//         /* WIN and F12 keys are not hardcoded here. See comments on top of this file. */
//         if (pHotKey->id == IDHK_WINKEY)
//         {
//             ASSERT(!bIsDown);
//             pWnd = ValidateHwndNoErr(InputWindowStation->ShellWindow);
//             if (pWnd)
//             {
//                TRACE("System Hot key Id %d Key %u\n", pHotKey->id, wVk );
//                UserPostMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_TASKLIST, 0);
//                co_IntShellHookNotify(HSHELL_TASKMAN, 0, 0);
//                return FALSE;
//             }
//         }
//         
//         if (pHotKey->id == IDHK_SNAP_LEFT ||
//             pHotKey->id == IDHK_SNAP_RIGHT ||
//             pHotKey->id == IDHK_SNAP_UP ||
//             pHotKey->id == IDHK_SNAP_DOWN)
//         {
//             HWND topWnd = UserGetForegroundWindow();
//             if (topWnd)
//             {
//                 UserPostMessage(topWnd, WM_KEYDOWN, wVk, 0);
//             }
//             return TRUE;
//         }
//
//         if (!pHotKey->pWnd)
//         {
//             TRACE("UPTM Hot key Id %d Key %u\n", pHotKey->id, wVk );
//             UserPostThreadMessage(pHotKey->pti, WM_HOTKEY, pHotKey->id, MAKELONG(fModifiers, wVk));
//             //ptiLastInput = pHotKey->pti;
//             return TRUE; /* Don't send any message */
//         }
//         else
//         {
//             pWnd = pHotKey->pWnd;
//             if (pWnd == PWND_BOTTOM)
//             {
//                 if (gpqForeground == NULL)
//                     return FALSE;
//
//                 pWnd = gpqForeground->spwndFocus;
//             }
//
//             if (pWnd)
//             {
//                 //  pWnd->head.rpdesk->pDeskInfo->spwndShell needs testing.
//                 if (pWnd == ValidateHwndNoErr(InputWindowStation->ShellWindow) && pHotKey->id == SC_TASKLIST)
//                 {
//                     UserPostMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_TASKLIST, 0);
//                     co_IntShellHookNotify(HSHELL_TASKMAN, 0, 0);
//                 }
//                 else if (IsWindowHotKey(pHotKey))
//                 {
//                     /* WM_SETHOTKEY notifies with WM_SYSCOMMAND, not WM_HOTKEY */
//                     if (bIsDown)
//                     {
//                         if (gpqForeground && gpqForeground->spwndActive)
//                             pWnd = gpqForeground->spwndActive;
//                         UserPostMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND,
//                                         SC_HOTKEY, (LPARAM)UserHMGetHandle(pHotKey->pWnd));
//                     }
//                 }
//                 else
//                 {
//                     TRACE("UPM Hot key Id %d Key %u\n", pHotKey->id, wVk );
//                     UserPostMessage(UserHMGetHandle(pWnd), WM_HOTKEY, pHotKey->id, MAKELONG(fModifiers, wVk));
//                 }
//                 //ptiLastInput = pWnd->head.pti;
//                 return TRUE; /* Don't send any message */
//             }
//         }
//     }
//     return FALSE;
// }
//
//
// /*
//  * DefWndGetHotKey --- GetHotKey message support
//  *
//  * Win: DWP_GetHotKey
//  */
// UINT FASTCALL
// DefWndGetHotKey(PWND pWnd)
// {
//     PHOT_KEY pHotKey = gphkFirst;
//
//     WARN("DefWndGetHotKey\n");
//
//     while (pHotKey)
//     {
//         if (pHotKey->pWnd == pWnd && IsWindowHotKey(pHotKey))
//         {
//             /* We have found it */
//             return MAKEWORD(pHotKey->vk, IntSwapModHKF(pHotKey->fsModifiers));
//         }
//
//         /* Move to the next entry */
//         pHotKey = pHotKey->pNext;
//     }
//
//     return 0;
// }
//
// /*
//  * DefWndSetHotKey --- SetHotKey message support
//  *
//  * Win: DWP_SetHotKey
//  */
// INT FASTCALL
// DefWndSetHotKey(PWND pWnd, WPARAM wParam)
// {
//     const UINT fsModifiers = IntSwapModHKF(HIBYTE(wParam));
//     const UINT vk = LOBYTE(wParam);
//     PHOT_KEY pHotKey, *pLink;
//     INT iRet = 1;
//
//     WARN("DefWndSetHotKey wParam 0x%x\n", wParam);
//
//     // A hot key cannot be associated with a child window.
//     if (pWnd->style & WS_CHILD)
//         return 0;
//
//     // VK_ESCAPE, VK_SPACE, VK_TAB and VK_PACKET are invalid hot keys.
//     if (vk == VK_ESCAPE || vk == VK_SPACE || vk == VK_TAB || vk == VK_PACKET)
//     {
//         return -1;
//     }
//
//     if (wParam)
//     {
//         pHotKey = gphkFirst;
//         while (pHotKey)
//         {
//             if (pHotKey->fsModifiers == fsModifiers &&
//                 pHotKey->vk == vk &&
//                 IsWindowHotKey(pHotKey))
//             {
//                 if (pHotKey->pWnd != pWnd)
//                     iRet = 2; // Another window already has the same hot key.
//                 break;
//             }
//
//             /* Move to the next entry */
//             pHotKey = pHotKey->pNext;
//         }
//     }
//
//     pHotKey = gphkFirst;
//     pLink = &gphkFirst;
//     while (pHotKey)
//     {
//         if (pHotKey->pWnd == pWnd && IsWindowHotKey(pHotKey))
//         {
//             /* This window has already hotkey registered */
//             break;
//         }
//
//         /* Move to the next entry */
//         pLink = &pHotKey->pNext;
//         pHotKey = pHotKey->pNext;
//     }
//
//     if (wParam)
//     {
//         if (!pHotKey)
//         {
//             /* Create new hotkey */
//             pHotKey = ExAllocatePoolWithTag(PagedPool, sizeof(HOT_KEY), USERTAG_HOTKEY);
//             if (pHotKey == NULL)
//                 return 0;
//
//             pHotKey->pWnd = pWnd;
//             pHotKey->id = IDHK_WNDKEY; // Don't care, these hot keys are unrelated to the hot keys set by RegisterHotKey
//             pHotKey->pNext = gphkFirst;
//             gphkFirst = pHotKey;
//         }
//
//         /* A window can only have one hot key. If the window already has a
//            hot key associated with it, the new hot key replaces the old one. */
//         pHotKey->pti = NULL; /* IsWindowHotKey */
//         pHotKey->fsModifiers = fsModifiers;
//         pHotKey->vk = vk;
//     }
//     else if (pHotKey)
//     {
//         /* Remove hotkey */
//         *pLink = pHotKey->pNext;
//         ExFreePoolWithTag(pHotKey, USERTAG_HOTKEY);
//     }
//
//     return iRet;
// }
//
//
// BOOL FASTCALL
// UserRegisterHotKey(PWND pWnd,
//                    int id,
//                    UINT fsModifiers,
//                    UINT vk)
// {
//     PHOT_KEY pHotKey;
//     PTHREADINFO pHotKeyThread;
//
//     /* Find the hotkey thread */
//     if (pWnd == NULL || pWnd == PWND_BOTTOM)
//     {
//         pHotKeyThread = PsGetCurrentThreadWin32Thread(); // gptiCurrent;
//     }
//     else
//     {
//         pHotKeyThread = pWnd->head.pti;
//     }
//
//     /* Ignore the VK_PACKET key since it is not a real keyboard input */
//     if (vk == VK_PACKET)
//         return FALSE;
//
//     /* Check whether we modify an existing hotkey */
//     if (IsHotKey(fsModifiers, vk))
//     {
//         EngSetLastError(ERROR_HOTKEY_ALREADY_REGISTERED);
//         WARN("Hotkey already exists\n");
//         return FALSE;
//     }
//
//     /* Create a new hotkey */
//     pHotKey = ExAllocatePoolWithTag(PagedPool, sizeof(HOT_KEY), USERTAG_HOTKEY);
//     if (pHotKey == NULL)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     pHotKey->pti = pHotKeyThread;
//     pHotKey->pWnd = pWnd;
//     pHotKey->fsModifiers = fsModifiers;
//     pHotKey->vk = vk;
//     pHotKey->id = id;
//
//     /* Insert the hotkey into the global list */
//     pHotKey->pNext = gphkFirst;
//     gphkFirst = pHotKey;
//
//     return TRUE;
// }
//
// BOOL FASTCALL
// UserUnregisterHotKey(PWND pWnd, int id)
// {
//     PHOT_KEY pHotKey = gphkFirst, phkNext, *pLink = &gphkFirst;
//     BOOL bRet = FALSE;
//
//     while (pHotKey)
//     {
//         /* Save next ptr for later use */
//         phkNext = pHotKey->pNext;
//
//         /* Should we delete this hotkey? */
//         if (pHotKey->pWnd == pWnd && pHotKey->id == id)
//         {
//             /* Update next ptr for previous hotkey and free memory */
//             *pLink = phkNext;
//             ExFreePoolWithTag(pHotKey, USERTAG_HOTKEY);
//
//             bRet = TRUE;
//         }
//         else
//         {
//             /* This hotkey will stay, use its next ptr */
//             pLink = &pHotKey->pNext;
//         }
//
//         /* Move to the next entry */
//         pHotKey = phkNext;
//     }
//
//     return bRet;
// }
//
//
// /* SYSCALLS *****************************************************************/
//
// BOOL __stdcall
// NtUserRegisterHotKey(HWND hWnd,
//                      int id,
//                      UINT fsModifiers,
//                      UINT vk)
// {
//     PWND pWnd = NULL;
//     BOOL bRet = FALSE;
//
//     TRACE("Enter NtUserRegisterHotKey\n");
//
//     // FIXME: Does Win2k3 support MOD_NOREPEAT?
//     if (fsModifiers & ~(MOD_ALT | MOD_CONTROL | MOD_SHIFT | MOD_WIN))
//     {
//         WARN("Invalid modifiers: %x\n", fsModifiers);
//         EngSetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
//     UserEnterExclusive();
//
//     /* Check the hotkey thread */
//     if (hWnd == NULL)
//     {
//         pWnd = NULL;
//     }
//     else
//     {
//         pWnd = UserGetWindowObject(hWnd);
//         if (!pWnd)
//             goto cleanup;
//
//         /* FIXME?? "Fix" wine msg "Window on another thread" test_hotkey */
//         if (pWnd->head.pti != gptiCurrent)
//         {
//             EngSetLastError(ERROR_WINDOW_OF_OTHER_THREAD);
//             WARN("Must be from the same Thread.\n");
//             goto cleanup;
//         }
//     }
//
//     bRet = UserRegisterHotKey(pWnd, id, fsModifiers, vk);
//
// cleanup:
//     TRACE("Leave NtUserRegisterHotKey, ret=%i\n", bRet);
//     UserLeave();
//     return bRet;
// }
//
// BOOL __stdcall
// NtUserUnregisterHotKey(HWND hWnd, int id)
// {
//     BOOL bRet = FALSE;
//     PWND pWnd = NULL;
//
//     TRACE("Enter NtUserUnregisterHotKey\n");
//     UserEnterExclusive();
//
//     /* Fail if the given window is invalid */
//     if (hWnd && !(pWnd = UserGetWindowObject(hWnd)))
//         goto cleanup;
//
//     bRet = UserUnregisterHotKey(pWnd, id);
//
// cleanup:
//     TRACE("Leave NtUserUnregisterHotKey, ret=%i\n", bRet);
//     UserLeave();
//     return bRet;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
