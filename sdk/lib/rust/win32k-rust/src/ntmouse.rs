#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread};
use core::ffi::c_void;

// Debug-channel macros — no kernel symbol, no-op.
#[allow(unused_macros)]
#[allow(unused_macros)]
#[allow(unused_macros)]

// BOOL is i32; bindgen's FALSE/TRUE are u32. Shadow them as BOOL.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// UserHMGetHandle(obj) = obj->head.h  (HANDLE/HWND at offset 0 of HEAD).
// FORCEINLINE in user32/win32k headers — no exported symbol.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// SafeProbeForRead/SafeProbeForWrite — safe_user_io.h helpers. STDCALL
// (`_SafeProbeForRead@12` on i386) => extern "system". SIZE_T length, matches
// console.rs/icm.rs canonical signature.
extern "system" {
    fn SafeProbeForRead(Address: *const c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: *mut c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// SetLastNtError — FASTCALL (`@SetLastNtError@4`) on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

// EngGetTickCount32() = (ULONG)EngGetTickCount().  (eng.h)
#[inline(always)]
unsafe fn EngGetTickCount32() -> ULONG {
    EngGetTickCount() as ULONG
}

// NT_SUCCESS(x): (NTSTATUS)(x) >= 0
#[inline(always)]
fn NT_SUCCESS(x: NTSTATUS) -> bool {
    (x as i32) >= 0
}

// RtlZeroMemory / RtlCopyMemory — compiler-intrinsic memset/memcpy (no symbol).
#[inline(always)]
unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// ARRAYSIZE(arr) used only on the fixed 64-entry move history.
#[inline(always)]
fn ARRAYSIZE(_p: *const c_void) -> usize { 64 }

// IS_KEY_DOWN(ks, vk): ((ks)[vk*2/8] & (1 << ((vk%4)*2))) ? TRUE : FALSE
#[inline(always)]
unsafe fn IS_KEY_DOWN(ks: *const u8, vk: u32) -> BOOL {
    let byte = *ks.add((vk * 2 / 8) as usize);
    if (byte & (1u8 << ((vk % 4) * 2))) != 0 { TRUE } else { FALSE }
}

// SET_KEY_DOWN(ks, vk, down): set/clear the down bit for vk.
#[inline(always)]
unsafe fn SET_KEY_DOWN(ks: *mut u8, vk: u32, down: i32) {
    let p = ks.add((vk * 2 / 8) as usize);
    let bit = 1u8 << ((vk % 4) * 2);
    *p = if down != 0 { *p | bit } else { *p & !bit };
}

// MAKEWPARAM(l, h) = (WORD)l | ((WORD)h << 16)  -> WPARAM
#[inline(always)]
fn MAKEWPARAM(l: WORD, h: WORD) -> WPARAM {
    ((l as u32) | ((h as u32) << 16)) as WPARAM
}
// MAKELPARAM(l, h) = (WORD)l | ((WORD)h << 16)  -> LPARAM
#[inline(always)]
fn MAKELPARAM(l: WORD, h: WORD) -> LPARAM {
    (((l as u32) | ((h as u32) << 16)) as i32) as LPARAM
}

// RECTL_vSetRect — rect.h FORCEINLINE (no exported symbol).
#[inline(always)]
unsafe fn RECTL_vSetRect(prcl: *mut RECTL, left: i32, top: i32, right: i32, bottom: i32) {
    (*prcl).left = left;
    (*prcl).top = top;
    (*prcl).right = right;
    (*prcl).bottom = bottom;
}

// gpsi->ptCursor accessor. tagSERVERINFO ends with __bindgen_padding_0
// representing the anonymous PERUSERSERVERINFO member; cast that tail to access
// ptCursor (matches timer.rs/mouse.rs).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// head_pti(obj) — bindgen flattened THRDESKHEAD (no `pti` field). pti sits at
// 2*sizeof(usize) (= 8 on i386, 16 on amd64/arm64): after HANDLE h + DWORD
// cLockObj plus the head's tail padding on 64-bit. (matches timer.rs/event.rs)
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// SystemTimerProc — CALLBACK in timer.c (timer.h: VOID CALLBACK SystemTimerProc(...)).
// Passed as a TIMERPROC function pointer to IntSetTimer. TIMERPROC is
// `extern "stdcall"` on i386 / `extern "C"` elsewhere — match it exactly so the
// fn item unifies with the Option<extern fn> param type.
extern "system" {
    fn SystemTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD);
}

// ---- file-scope preamble (translated) ----
pub static mut gMouseHistoryOfMoves: [MOUSEMOVEPOINT; 64] = unsafe { core::mem::zeroed() };
pub static mut gcMouseHistoryOfMoves: INT = 0;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetMouseButtonsState() -> WORD {
    UserGetMouseButtonsState_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetMouseButtonsState() -> WORD {
    UserGetMouseButtonsState_impl()
}

unsafe fn UserGetMouseButtonsState_impl() -> WORD {
    let mut wRet: WORD = 0;

    wRet = (*IntGetSysCursorInfo()).ButtonsDown as WORD;

    if IS_KEY_DOWN((*(&raw const gafAsyncKeyState)).as_ptr(), VK_SHIFT) != 0 {
        wRet |= MK_SHIFT as WORD;
    }
    if IS_KEY_DOWN((*(&raw const gafAsyncKeyState)).as_ptr(), VK_CONTROL) != 0 {
        wRet |= MK_CONTROL as WORD;
    }

    wRet
}
#[no_mangle]
pub unsafe extern "system" fn UserProcessMouseInput(mid: PMOUSE_INPUT_DATA) {
    let mut mi: MOUSEINPUT = core::mem::zeroed();

    mi.dx = (*mid).LastX;
    mi.dy = (*mid).LastY;
    mi.mouseData = 0;
    mi.dwFlags = 0;
    mi.time = 0;
    mi.dwExtraInfo = (*mid).ExtraInformation as ULONG_PTR;

    if mi.dx != 0 || mi.dy != 0 {
        mi.dwFlags |= MOUSEEVENTF_MOVE;
    }

    if (*mid).Flags as u32 & MOUSE_MOVE_ABSOLUTE != 0 {
        mi.dwFlags |= MOUSEEVENTF_ABSOLUTE;
    }
    if (*mid).Flags as u32 & MOUSE_VIRTUAL_DESKTOP != 0 {
        mi.dwFlags |= MOUSEEVENTF_VIRTUALDESK;
    }

    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_LEFT_BUTTON_DOWN != 0 {
        mi.dwFlags |= MOUSEEVENTF_LEFTDOWN;
    }
    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_LEFT_BUTTON_UP != 0 {
        mi.dwFlags |= MOUSEEVENTF_LEFTUP;
    }

    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_MIDDLE_BUTTON_DOWN != 0 {
        mi.dwFlags |= MOUSEEVENTF_MIDDLEDOWN;
    }
    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_MIDDLE_BUTTON_UP != 0 {
        mi.dwFlags |= MOUSEEVENTF_MIDDLEUP;
    }

    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_RIGHT_BUTTON_DOWN != 0 {
        mi.dwFlags |= MOUSEEVENTF_RIGHTDOWN;
    }
    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_RIGHT_BUTTON_UP != 0 {
        mi.dwFlags |= MOUSEEVENTF_RIGHTUP;
    }

    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_BUTTON_4_DOWN != 0 {
        mi.dwFlags |= MOUSEEVENTF_XDOWN;
        mi.mouseData |= XBUTTON1 as DWORD;
    }
    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_BUTTON_4_UP != 0 {
        mi.dwFlags |= MOUSEEVENTF_XUP;
        mi.mouseData |= XBUTTON1 as DWORD;
    }

    if mi.dwFlags & (MOUSE_BUTTON_4_DOWN | MOUSE_BUTTON_4_UP) != 0 {
        UserSendMouseInput(&mut mi, FALSE as BOOL);
        RtlZeroMemory(&mut mi as *mut _ as *mut _, core::mem::size_of::<MOUSEINPUT>());
    }

    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_BUTTON_5_DOWN != 0 {
        mi.mouseData |= XBUTTON2 as DWORD;
        mi.dwFlags |= MOUSEEVENTF_XDOWN;
    }
    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_BUTTON_5_UP != 0 {
        mi.mouseData |= XBUTTON2 as DWORD;
        mi.dwFlags |= MOUSEEVENTF_XUP;
    }

    if mi.dwFlags & (MOUSE_BUTTON_5_DOWN | MOUSE_BUTTON_5_UP) != 0 {
        UserSendMouseInput(&mut mi, FALSE as BOOL);
        RtlZeroMemory(&mut mi as *mut _ as *mut _, core::mem::size_of::<MOUSEINPUT>());
    }

    if (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags as u32 & MOUSE_WHEEL != 0 {
        mi.mouseData = (*mid).__bindgen_anon_1.__bindgen_anon_1.ButtonData as DWORD;
        mi.dwFlags |= MOUSEEVENTF_WHEEL;
    }

    if mi.dwFlags != 0 {
        UserSendMouseInput(&mut mi, FALSE as BOOL);
    }
}

unsafe fn IntFixMouseInputButtons(dwFlags: DWORD) -> DWORD {
    let mut dwNewFlags: DWORD;

    if gspv.bMouseBtnSwap == 0 {
        return dwFlags;
    }

    dwNewFlags = dwFlags & !(MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP |
                             MOUSEEVENTF_RIGHTDOWN | MOUSEEVENTF_RIGHTUP);

    if dwFlags & MOUSEEVENTF_LEFTDOWN != 0 {
        dwNewFlags |= MOUSEEVENTF_RIGHTDOWN;
    }
    if dwFlags & MOUSEEVENTF_LEFTUP != 0 {
        dwNewFlags |= MOUSEEVENTF_RIGHTUP;
    }
    if dwFlags & MOUSEEVENTF_RIGHTDOWN != 0 {
        dwNewFlags |= MOUSEEVENTF_LEFTDOWN;
    }
    if dwFlags & MOUSEEVENTF_RIGHTUP != 0 {
        dwNewFlags |= MOUSEEVENTF_LEFTUP;
    }

    dwNewFlags
}
#[no_mangle]
pub unsafe extern "system" fn UserSendMouseInput(pmi: *mut MOUSEINPUT, bInjected: BOOL) -> BOOL {
    let mut ptCursor: POINTL;
    let pCurInfo: PSYSTEM_CURSORINFO;
    let mut Msg: MSG = core::mem::zeroed();
    let dwFlags: DWORD;

    pCurInfo = IntGetSysCursorInfo();
    ptCursor = (*gpsi_per_user()).ptCursor;
    dwFlags = IntFixMouseInputButtons((*pmi).dwFlags);

    gppiInputProvider = (*(PsGetCurrentThreadWin32Thread() as PTHREADINFO)).ppi;

    if (*pmi).dwFlags & MOUSEEVENTF_MOVE != 0 {
        if (*pmi).dwFlags & MOUSEEVENTF_ABSOLUTE == 0 {
            ptCursor.x += (*pmi).dx;
            ptCursor.y += (*pmi).dy;
        } else if (*pmi).dwFlags & MOUSEEVENTF_VIRTUALDESK != 0 {
            ptCursor.x = (*pmi).dx * UserGetSystemMetrics(SM_CXVIRTUALSCREEN as ULONG) >> 16;
            ptCursor.y = (*pmi).dy * UserGetSystemMetrics(SM_CYVIRTUALSCREEN as ULONG) >> 16;
        } else {
            ptCursor.x = (*pmi).dx * UserGetSystemMetrics(SM_CXSCREEN as ULONG) >> 16;
            ptCursor.y = (*pmi).dy * UserGetSystemMetrics(SM_CYSCREEN as ULONG) >> 16;
        }
    }

    Msg.wParam = UserGetMouseButtonsState() as WPARAM;
    Msg.lParam = MAKELPARAM(ptCursor.x as WORD, ptCursor.y as WORD);
    Msg.pt = ptCursor;
    Msg.time = (*pmi).time;
    if Msg.time == 0 {
        Msg.time = EngGetTickCount32();
    }

    gMouseHistoryOfMoves[gcMouseHistoryOfMoves as usize].x = ptCursor.x;
    gMouseHistoryOfMoves[gcMouseHistoryOfMoves as usize].y = ptCursor.y;
    gMouseHistoryOfMoves[gcMouseHistoryOfMoves as usize].time = Msg.time;
    gMouseHistoryOfMoves[gcMouseHistoryOfMoves as usize].dwExtraInfo = (*pmi).dwExtraInfo;
    gcMouseHistoryOfMoves += 1;
    if gcMouseHistoryOfMoves as usize == ARRAYSIZE((*(&raw const gMouseHistoryOfMoves)).as_ptr() as *const _) as usize {
        gcMouseHistoryOfMoves = 0;
    }

    if dwFlags & MOUSEEVENTF_MOVE != 0 {
        UserSetCursorPos(ptCursor.x, ptCursor.y, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    }

    if IS_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_SHIFT) != 0 {
        (*pCurInfo).ButtonsDown |= MK_SHIFT as UINT;
    } else {
        (*pCurInfo).ButtonsDown &= !(MK_SHIFT as UINT);
    }

    if IS_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_CONTROL) != 0 {
        (*pCurInfo).ButtonsDown |= MK_CONTROL as UINT;
    } else {
        (*pCurInfo).ButtonsDown &= !(MK_CONTROL as UINT);
    }

    if dwFlags & MOUSEEVENTF_LEFTDOWN != 0 {
        SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_LBUTTON, 1);
        Msg.message = WM_LBUTTONDOWN;
        (*pCurInfo).ButtonsDown |= MK_LBUTTON as UINT;
        Msg.wParam |= MK_LBUTTON as WPARAM;
        co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    } else if dwFlags & MOUSEEVENTF_LEFTUP != 0 {
        SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_LBUTTON, 0);
        Msg.message = WM_LBUTTONUP;
        (*pCurInfo).ButtonsDown &= !(MK_LBUTTON as UINT);
        Msg.wParam &= !(MK_LBUTTON as WPARAM);
        co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    }

    if dwFlags & MOUSEEVENTF_MIDDLEDOWN != 0 {
        SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MBUTTON, 1);
        Msg.message = WM_MBUTTONDOWN;
        (*pCurInfo).ButtonsDown |= MK_MBUTTON as UINT;
        Msg.wParam |= MK_MBUTTON as WPARAM;
        co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    } else if dwFlags & MOUSEEVENTF_MIDDLEUP != 0 {
        SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_MBUTTON, 0);
        Msg.message = WM_MBUTTONUP;
        (*pCurInfo).ButtonsDown &= !(MK_MBUTTON as UINT);
        Msg.wParam &= !(MK_MBUTTON as WPARAM);
        co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    }

    if dwFlags & MOUSEEVENTF_RIGHTDOWN != 0 {
        SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_RBUTTON, 1);
        Msg.message = WM_RBUTTONDOWN;
        (*pCurInfo).ButtonsDown |= MK_RBUTTON as UINT;
        Msg.wParam |= MK_RBUTTON as WPARAM;
        co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    } else if dwFlags & MOUSEEVENTF_RIGHTUP != 0 {
        SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_RBUTTON, 0);
        Msg.message = WM_RBUTTONUP;
        (*pCurInfo).ButtonsDown &= !(MK_RBUTTON as UINT);
        Msg.wParam &= !(MK_RBUTTON as WPARAM);
        co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    }

    if (dwFlags & (MOUSEEVENTF_XDOWN | MOUSEEVENTF_XUP) != 0) &&
       (dwFlags & MOUSEEVENTF_WHEEL != 0) {
        return 0;
    }

    if dwFlags & MOUSEEVENTF_XDOWN != 0 {
        Msg.message = WM_XBUTTONDOWN;
        if (*pmi).mouseData & XBUTTON1 != 0 {
            SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_XBUTTON1, 1);
            (*pCurInfo).ButtonsDown |= MK_XBUTTON1 as UINT;
            Msg.wParam |= MAKEWPARAM(MK_XBUTTON1 as WORD, XBUTTON1 as WORD) as WPARAM;
            co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
        }
        if (*pmi).mouseData & XBUTTON2 != 0 {
            SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_XBUTTON2, 1);
            (*pCurInfo).ButtonsDown |= MK_XBUTTON2 as UINT;
            Msg.wParam |= MAKEWPARAM(MK_XBUTTON2 as WORD, XBUTTON2 as WORD) as WPARAM;
            co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
        }
    } else if dwFlags & MOUSEEVENTF_XUP != 0 {
        Msg.message = WM_XBUTTONUP;
        if (*pmi).mouseData & XBUTTON1 != 0 {
            SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_XBUTTON1, 0);
            (*pCurInfo).ButtonsDown &= !(MK_XBUTTON1 as UINT);
            Msg.wParam &= !(MK_XBUTTON1 as WPARAM);
            Msg.wParam |= MAKEWPARAM(0, XBUTTON1 as WORD) as WPARAM;
            co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
        }
        if (*pmi).mouseData & XBUTTON2 != 0 {
            SET_KEY_DOWN((*(&raw mut gafAsyncKeyState)).as_mut_ptr(), VK_XBUTTON2, 0);
            (*pCurInfo).ButtonsDown &= !(MK_XBUTTON2 as UINT);
            Msg.wParam &= !(MK_XBUTTON2 as WPARAM);
            Msg.wParam |= MAKEWPARAM(0, XBUTTON2 as WORD) as WPARAM;
            co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
        }
    }

    if dwFlags & MOUSEEVENTF_WHEEL != 0 {
        Msg.message = WM_MOUSEWHEEL;
        Msg.wParam = MAKEWPARAM((*pCurInfo).ButtonsDown as WORD, (*pmi).mouseData as WORD) as WPARAM;
        co_MsqInsertMouseMessage(&mut Msg, bInjected as DWORD, (*pmi).dwExtraInfo, 1);
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntRemoveTrackMouseEvent(pDesk: PDESKTOP) {
    IntRemoveTrackMouseEvent_impl(pDesk)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntRemoveTrackMouseEvent(pDesk: PDESKTOP) {
    IntRemoveTrackMouseEvent_impl(pDesk)
}

#[inline]
unsafe fn IntRemoveTrackMouseEvent_impl(pDesk: PDESKTOP) {

    /* Generate a leave message */
    if (*pDesk).dwDTFlags & DF_TME_LEAVE != 0 {
        let uMsg = if (*pDesk).htEx != HTCLIENT as _ {
            WM_NCMOUSELEAVE
        } else {
            WM_MOUSELEAVE
        };
        UserPostMessage(
            UserHMGetHandle((*pDesk).spwndTrack as *mut core::ffi::c_void) as _,
            uMsg,
            0,
            0,
        );
    }

    /* Kill the timer */
    if (*pDesk).dwDTFlags & DF_TME_HOVER != 0 {
        IntKillTimer((*pDesk).spwndTrack, ID_EVENT_SYSTIMER_MOUSEHOVER as _, TRUE);
    }

    /* Reset state */
    (*pDesk).dwDTFlags &= !(DF_TME_LEAVE | DF_TME_HOVER);
    (*pDesk).spwndTrack = core::ptr::null_mut();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntQueryTrackMouseEvent(
    lpEventTrack: LPTRACKMOUSEEVENT,
) -> BOOL {
    IntQueryTrackMouseEvent_impl(lpEventTrack)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntQueryTrackMouseEvent(
    lpEventTrack: LPTRACKMOUSEEVENT,
) -> BOOL {
    IntQueryTrackMouseEvent_impl(lpEventTrack)
}

unsafe fn IntQueryTrackMouseEvent_impl(
    lpEventTrack: LPTRACKMOUSEEVENT,
) -> BOOL {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let pDesk: PDESKTOP = (*pti).rpdesk;

    RtlZeroMemory(
        lpEventTrack as *mut core::ffi::c_void,
        core::mem::size_of::<TRACKMOUSEEVENT>() as _,
    );
    (*lpEventTrack).cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as _;

    if (*pDesk).dwDTFlags & (DF_TME_LEAVE | DF_TME_HOVER) != 0
        && !(*pDesk).spwndTrack.is_null()
        && (*pti).MessageQueue == (*head_pti((*pDesk).spwndTrack)).MessageQueue
    {
        if (*pDesk).htEx != HTCLIENT as _ {
            (*lpEventTrack).dwFlags |= TME_NONCLIENT;
        }

        if (*pDesk).dwDTFlags & DF_TME_LEAVE != 0 {
            (*lpEventTrack).dwFlags |= TME_LEAVE;
        }

        if (*pDesk).dwDTFlags & DF_TME_HOVER != 0 {
            (*lpEventTrack).dwFlags |= TME_HOVER;
            (*lpEventTrack).dwHoverTime = (*pDesk).dwMouseHoverTime;
        }
        (*lpEventTrack).hwndTrack = UserHMGetHandle((*pDesk).spwndTrack);
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntTrackMouseEvent(lpEventTrack: LPTRACKMOUSEEVENT) -> BOOL {
    IntTrackMouseEvent_impl(lpEventTrack)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntTrackMouseEvent(lpEventTrack: LPTRACKMOUSEEVENT) -> BOOL {
    IntTrackMouseEvent_impl(lpEventTrack)
}

unsafe fn IntTrackMouseEvent_impl(lpEventTrack: LPTRACKMOUSEEVENT) -> BOOL {
    let pDesk: PDESKTOP;
    let pti: PTHREADINFO;
    let pWnd: PWND;
    let point: POINTL;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    pDesk = (*pti).rpdesk;

    pWnd = UserGetWindowObject((*lpEventTrack).hwndTrack);
    if pWnd.is_null() {
        return FALSE as BOOL;
    }

    if (*pDesk).spwndTrack != pWnd
        || (((*pDesk).htEx != HTCLIENT as _) as i32
            ^ ((((*lpEventTrack).dwFlags & TME_NONCLIENT) != 0) as i32))
            != 0
    {
        if (*lpEventTrack).dwFlags & TME_LEAVE != 0
            && (*lpEventTrack).dwFlags & TME_CANCEL == 0
        {
            UserPostMessage(
                (*lpEventTrack).hwndTrack,
                if (*lpEventTrack).dwFlags & TME_NONCLIENT != 0 {
                    WM_NCMOUSELEAVE
                } else {
                    WM_MOUSELEAVE
                },
                0,
                0,
            );
        }
        TRACE!(
            b"IntTrackMouseEvent spwndTrack %p pwnd %p\n\0".as_ptr() as PCHAR,
            (*pDesk).spwndTrack,
            pWnd,
        );
        return TRUE as BOOL;
    }

    if (*lpEventTrack).dwFlags & TME_CANCEL != 0 {
        if (*lpEventTrack).dwFlags & TME_LEAVE != 0 {
            (*pDesk).dwDTFlags &= !DF_TME_LEAVE;
        }

        if (*lpEventTrack).dwFlags & TME_HOVER != 0 {
            if (*pDesk).dwDTFlags & DF_TME_HOVER != 0 {
                IntKillTimer(pWnd, ID_EVENT_SYSTIMER_MOUSEHOVER as _, TRUE);
                (*pDesk).dwDTFlags &= !DF_TME_HOVER;
            }
        }
    } else {
        if (*lpEventTrack).dwFlags & TME_LEAVE != 0 {
            (*pDesk).dwDTFlags |= DF_TME_LEAVE;
        }

        if (*lpEventTrack).dwFlags & TME_HOVER != 0 {
            (*pDesk).dwDTFlags |= DF_TME_HOVER;

            if (*lpEventTrack).dwHoverTime == 0
                || (*lpEventTrack).dwHoverTime == HOVER_DEFAULT
            {
                (*pDesk).dwMouseHoverTime = gspv.iMouseHoverTime as _;
            } else {
                (*pDesk).dwMouseHoverTime = (*lpEventTrack).dwHoverTime;
            }
            IntSetTimer(
                pWnd,
                ID_EVENT_SYSTIMER_MOUSEHOVER as _,
                (*pDesk).dwMouseHoverTime,
                Some(SystemTimerProc),
                TMRF_SYSTEM as _,
            );
            point = (*head_pti(pWnd)).ptLast;
            RECTL_vSetRect(
                core::ptr::addr_of_mut!((*pDesk).rcMouseHover),
                point.x - gspv.iMouseHoverWidth / 2,
                point.y - gspv.iMouseHoverHeight / 2,
                point.x + gspv.iMouseHoverWidth / 2,
                point.y + gspv.iMouseHoverHeight / 2,
            );
        }
    }
    TRUE as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserTrackMouseEvent(
    lpEventTrack: LPTRACKMOUSEEVENT,
) -> BOOL {
    let mut SafeTME: TRACKMOUSEEVENT = core::mem::zeroed();
    let mut bRet: BOOL = 0;
    let Status: NTSTATUS;

    {
        let __SehStatus = SafeProbeForRead(
            lpEventTrack as *mut core::ffi::c_void,
            core::mem::size_of::<TRACKMOUSEEVENT>() as _,
            1,
        );
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            return 0;
        }
    }
    RtlCopyMemory(
        core::ptr::addr_of_mut!(SafeTME) as *mut core::ffi::c_void,
        lpEventTrack as *const core::ffi::c_void,
        core::mem::size_of::<TRACKMOUSEEVENT>() as usize,
    );

    if SafeTME.cbSize != core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if SafeTME.dwFlags
        & !(TME_CANCEL | TME_QUERY | TME_NONCLIENT | TME_LEAVE | TME_HOVER)
        != 0
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    UserEnterExclusive();

    if SafeTME.dwFlags & TME_QUERY != 0 {
        bRet = IntQueryTrackMouseEvent(core::ptr::addr_of_mut!(SafeTME));
        Status = SafeProbeForWrite(
            lpEventTrack as *mut core::ffi::c_void,
            core::mem::size_of::<TRACKMOUSEEVENT>() as _,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            bRet = 0;
        } else {
            RtlCopyMemory(
                lpEventTrack as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeTME) as *const core::ffi::c_void,
                core::mem::size_of::<TRACKMOUSEEVENT>() as usize,
            );
        }
    } else {
        bRet = IntTrackMouseEvent(core::ptr::addr_of_mut!(SafeTME));
    }

    UserLeave();
    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMouseMovePointsEx(
    cbSize: UINT,
    lpptIn: LPMOUSEMOVEPOINT,
    lpptOut: LPMOUSEMOVEPOINT,
    nBufPoints: core::ffi::c_int,
    resolution: DWORD,
) -> DWORD {
    let mut Safeppt: MOUSEMOVEPOINT = core::mem::zeroed();
    let mut iRet: INT = -1;

    if (cbSize != core::mem::size_of::<MOUSEMOVEPOINT>() as UINT)
        || (nBufPoints < 0)
        || (nBufPoints > 64)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1i32 as DWORD;
    }

    if lpptIn.is_null() || (lpptOut.is_null() && nBufPoints != 0) {
        EngSetLastError(ERROR_NOACCESS);
        return -1i32 as DWORD;
    }

    {
        let __SehStatus = SafeProbeForRead(lpptIn as *mut _, cbSize as _, 1);
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            return -1i32 as DWORD;
        }
    }
    RtlCopyMemory(
        core::ptr::addr_of_mut!(Safeppt) as *mut _,
        lpptIn as *const _,
        cbSize as usize,
    );

    UserEnterShared();

    iRet = gcMouseHistoryOfMoves;

    loop {
        if Safeppt.x == 0 && Safeppt.y == 0 {
            break;
        }
        if gMouseHistoryOfMoves[iRet as usize].x == Safeppt.x
            && gMouseHistoryOfMoves[iRet as usize].y == Safeppt.y
        {
            if Safeppt.time != 0 {
                if Safeppt.time == gMouseHistoryOfMoves[iRet as usize].time {
                    break;
                } else {
                    iRet -= 1;
                    if iRet < 0 {
                        iRet = 63;
                    }
                    if iRet == gcMouseHistoryOfMoves {
                        break;
                    }
                    continue;
                }
            }
            break;
        }
        iRet -= 1;
        if iRet < 0 {
            iRet = 63;
        }
        if iRet == gcMouseHistoryOfMoves {
            break;
        }
    }

    'cleanup: {
        match resolution {
            GMMP_USE_DISPLAY_POINTS => {
                if nBufPoints != 0 {
                    let __SehStatus = SafeProbeForWrite(lpptOut as *mut _, cbSize as _, 1);
                    if !NT_SUCCESS(__SehStatus) {
                        SetLastNtError(__SehStatus);
                        iRet = -1;
                        break 'cleanup;
                    }
                }
                iRet = nBufPoints;
            }
            GMMP_USE_HIGH_RESOLUTION_POINTS => {}
            _ => {
                EngSetLastError(ERROR_POINT_NOT_FOUND);
                iRet = -1;
            }
        }
    }

    UserLeave();
    iRet as DWORD
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          Mouse functions
//  * FILE:             win32ss/user/ntuser/mouse.c
//  * PROGRAMERS:       Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *                   Rafal Harabien (rafalh@reactos.org)
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
// DBG_DEFAULT_CHANNEL(UserInput);
//
// MOUSEMOVEPOINT gMouseHistoryOfMoves[64];
// INT gcMouseHistoryOfMoves = 0;
//
// /*
//  * UserGetMouseButtonsState
//  *
//  * Returns bitfield of MK_* flags used in mouse messages
//  */
// WORD FASTCALL
// UserGetMouseButtonsState(VOID)
// {
//     WORD wRet = 0;
//
//     wRet = IntGetSysCursorInfo()->ButtonsDown;
//
//     if (IS_KEY_DOWN(gafAsyncKeyState, VK_SHIFT)) wRet |= MK_SHIFT;
//     if (IS_KEY_DOWN(gafAsyncKeyState, VK_CONTROL)) wRet |= MK_CONTROL;
//
//     return wRet;
// }
//
// /*
//  * UserProcessMouseInput
//  *
//  * Process raw mouse input data
//  */
// VOID NTAPI
// UserProcessMouseInput(PMOUSE_INPUT_DATA mid)
// {
//     MOUSEINPUT mi;
//
//     /* Convert MOUSE_INPUT_DATA to MOUSEINPUT. First init all fields. */
//     mi.dx = mid->LastX;
//     mi.dy = mid->LastY;
//     mi.mouseData = 0;
//     mi.dwFlags = 0;
//     mi.time = 0;
//     mi.dwExtraInfo = mid->ExtraInformation;
//
//     /* Mouse position */
//     if (mi.dx != 0 || mi.dy != 0)
//         mi.dwFlags |= MOUSEEVENTF_MOVE;
//
//     /* Flags for absolute move */
//     if (mid->Flags & MOUSE_MOVE_ABSOLUTE)
//         mi.dwFlags |= MOUSEEVENTF_ABSOLUTE;
//     if (mid->Flags & MOUSE_VIRTUAL_DESKTOP)
//         mi.dwFlags |= MOUSEEVENTF_VIRTUALDESK;
//
//     /* Left button */
//     if (mid->ButtonFlags & MOUSE_LEFT_BUTTON_DOWN)
//         mi.dwFlags |= MOUSEEVENTF_LEFTDOWN;
//     if (mid->ButtonFlags & MOUSE_LEFT_BUTTON_UP)
//         mi.dwFlags |= MOUSEEVENTF_LEFTUP;
//
//     /* Middle button */
//     if (mid->ButtonFlags & MOUSE_MIDDLE_BUTTON_DOWN)
//         mi.dwFlags |= MOUSEEVENTF_MIDDLEDOWN;
//     if (mid->ButtonFlags & MOUSE_MIDDLE_BUTTON_UP)
//         mi.dwFlags |= MOUSEEVENTF_MIDDLEUP;
//
//     /* Right button */
//     if (mid->ButtonFlags & MOUSE_RIGHT_BUTTON_DOWN)
//         mi.dwFlags |= MOUSEEVENTF_RIGHTDOWN;
//     if (mid->ButtonFlags & MOUSE_RIGHT_BUTTON_UP)
//         mi.dwFlags |= MOUSEEVENTF_RIGHTUP;
//
//     /* Note: Next buttons use mouseData field so they cannot be sent in one call */
//
//     /* Button 4 */
//     if (mid->ButtonFlags & MOUSE_BUTTON_4_DOWN)
//     {
//         mi.dwFlags |= MOUSEEVENTF_XDOWN;
//         mi.mouseData |= XBUTTON1;
//     }
//     if (mid->ButtonFlags & MOUSE_BUTTON_4_UP)
//     {
//         mi.dwFlags |= MOUSEEVENTF_XUP;
//         mi.mouseData |= XBUTTON1;
//     }
//
//     /* If mouseData is used by button 4, send input and clear mi */
//     if (mi.dwFlags & (MOUSE_BUTTON_4_DOWN | MOUSE_BUTTON_4_UP))
//     {
//         UserSendMouseInput(&mi, FALSE);
//         RtlZeroMemory(&mi, sizeof(mi));
//     }
//
//     /* Button 5 */
//     if (mid->ButtonFlags & MOUSE_BUTTON_5_DOWN)
//     {
//         mi.mouseData |= XBUTTON2;
//         mi.dwFlags |= MOUSEEVENTF_XDOWN;
//     }
//     if (mid->ButtonFlags & MOUSE_BUTTON_5_UP)
//     {
//         mi.mouseData |= XBUTTON2;
//         mi.dwFlags |= MOUSEEVENTF_XUP;
//     }
//
//     /* If mouseData is used by button 5, send input and clear mi */
//     if (mi.dwFlags & (MOUSE_BUTTON_5_DOWN | MOUSE_BUTTON_5_UP))
//     {
//         UserSendMouseInput(&mi, FALSE);
//         RtlZeroMemory(&mi, sizeof(mi));
//     }
//
//     /* Mouse wheel */
//     if (mid->ButtonFlags & MOUSE_WHEEL)
//     {
//         mi.mouseData = mid->ButtonData;
//         mi.dwFlags |= MOUSEEVENTF_WHEEL;
//     }
//
//     /* If something has changed, send input to user */
//     if (mi.dwFlags)
//         UserSendMouseInput(&mi, FALSE);
// }
//
// /*
//  * IntFixMouseInputButtons
//  *
//  * Helper function for supporting mouse button swap function
//  */
// DWORD
// IntFixMouseInputButtons(DWORD dwFlags)
// {
//     DWORD dwNewFlags;
//
//     if (!gspv.bMouseBtnSwap)
//         return dwFlags;
//
//     /* Buttons other than left and right are not affected */
//     dwNewFlags = dwFlags & ~(MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP |
//                              MOUSEEVENTF_RIGHTDOWN | MOUSEEVENTF_RIGHTUP);
//
//     /* Swap buttons */
//     if (dwFlags & MOUSEEVENTF_LEFTDOWN)
//         dwNewFlags |= MOUSEEVENTF_RIGHTDOWN;
//     if (dwFlags & MOUSEEVENTF_LEFTUP)
//         dwNewFlags |= MOUSEEVENTF_RIGHTUP;
//     if (dwFlags & MOUSEEVENTF_RIGHTDOWN)
//         dwNewFlags |= MOUSEEVENTF_LEFTDOWN;
//     if (dwFlags & MOUSEEVENTF_RIGHTUP)
//         dwNewFlags |= MOUSEEVENTF_LEFTUP;
//
//     return dwNewFlags;
// }
//
// /*
//  * UserSendMouseInput
//  *
//  * Process mouse input from input devices and SendInput API
//  */
// BOOL NTAPI
// UserSendMouseInput(MOUSEINPUT *pmi, BOOL bInjected)
// {
//     POINT ptCursor;
//     PSYSTEM_CURSORINFO pCurInfo;
//     MSG Msg;
//     DWORD dwFlags;
//
//     ASSERT(pmi);
//
//     pCurInfo = IntGetSysCursorInfo();
//     ptCursor = gpsi->ptCursor;
//     dwFlags = IntFixMouseInputButtons(pmi->dwFlags);
//
//     gppiInputProvider = ((PTHREADINFO)PsGetCurrentThreadWin32Thread())->ppi;
//
//     if (pmi->dwFlags & MOUSEEVENTF_MOVE)
//     {
//         /* Mouse has changes position */
//         if (!(pmi->dwFlags & MOUSEEVENTF_ABSOLUTE))
//         {
//             /* Relative move */
//             ptCursor.x += pmi->dx;
//             ptCursor.y += pmi->dy;
//         }
//         else if (pmi->dwFlags & MOUSEEVENTF_VIRTUALDESK)
//         {
//             /* Absolute move in virtual screen units */
//             ptCursor.x = pmi->dx * UserGetSystemMetrics(SM_CXVIRTUALSCREEN) >> 16;
//             ptCursor.y = pmi->dy * UserGetSystemMetrics(SM_CYVIRTUALSCREEN) >> 16;
//         }
//         else
//         {
//             /* Absolute move in primary monitor units */
//             ptCursor.x = pmi->dx * UserGetSystemMetrics(SM_CXSCREEN) >> 16;
//             ptCursor.y = pmi->dy * UserGetSystemMetrics(SM_CYSCREEN) >> 16;
//         }
//     }
//
//     /* Init message fields */
//     Msg.wParam = UserGetMouseButtonsState();
//     Msg.lParam = MAKELPARAM(ptCursor.x, ptCursor.y);
//     Msg.pt = ptCursor;
//     Msg.time = pmi->time;
//     if (!Msg.time)
//     {
//         Msg.time = EngGetTickCount32();
//     }
//
//     /* Do GetMouseMovePointsEx FIFO. */
//     gMouseHistoryOfMoves[gcMouseHistoryOfMoves].x = ptCursor.x;
//     gMouseHistoryOfMoves[gcMouseHistoryOfMoves].y = ptCursor.y;
//     gMouseHistoryOfMoves[gcMouseHistoryOfMoves].time = Msg.time;
//     gMouseHistoryOfMoves[gcMouseHistoryOfMoves].dwExtraInfo = pmi->dwExtraInfo;
//     if (++gcMouseHistoryOfMoves == ARRAYSIZE(gMouseHistoryOfMoves))
//        gcMouseHistoryOfMoves = 0; // 0 - 63 is 64, FIFO forwards.
//
//     /* Update cursor position */
//     if (dwFlags & MOUSEEVENTF_MOVE)
//     {
//         UserSetCursorPos(ptCursor.x, ptCursor.y, bInjected as DWORD, pmi->dwExtraInfo, TRUE);
//     }
//
//     if (IS_KEY_DOWN(gafAsyncKeyState, VK_SHIFT))
//         pCurInfo->ButtonsDown |= MK_SHIFT;
//     else
//         pCurInfo->ButtonsDown &= ~MK_SHIFT;
//
//     if (IS_KEY_DOWN(gafAsyncKeyState, VK_CONTROL))
//         pCurInfo->ButtonsDown |= MK_CONTROL;
//     else
//         pCurInfo->ButtonsDown &= ~MK_CONTROL;
//
//     /* Left button */
//     if (dwFlags & MOUSEEVENTF_LEFTDOWN)
//     {
//         SET_KEY_DOWN(gafAsyncKeyState, VK_LBUTTON, TRUE);
//         Msg.message = WM_LBUTTONDOWN;
//         pCurInfo->ButtonsDown |= MK_LBUTTON;
//         Msg.wParam |= MK_LBUTTON;
//         co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//     }
//     else if (dwFlags & MOUSEEVENTF_LEFTUP)
//     {
//         SET_KEY_DOWN(gafAsyncKeyState, VK_LBUTTON, FALSE);
//         Msg.message = WM_LBUTTONUP;
//         pCurInfo->ButtonsDown &= ~MK_LBUTTON;
//         Msg.wParam &= ~MK_LBUTTON;
//         co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//     }
//
//     /* Middle button */
//     if (dwFlags & MOUSEEVENTF_MIDDLEDOWN)
//     {
//         SET_KEY_DOWN(gafAsyncKeyState, VK_MBUTTON, TRUE);
//         Msg.message = WM_MBUTTONDOWN;
//         pCurInfo->ButtonsDown |= MK_MBUTTON;
//         Msg.wParam |= MK_MBUTTON;
//         co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//     }
//     else if (dwFlags & MOUSEEVENTF_MIDDLEUP)
//     {
//         SET_KEY_DOWN(gafAsyncKeyState, VK_MBUTTON, FALSE);
//         Msg.message = WM_MBUTTONUP;
//         pCurInfo->ButtonsDown &= ~MK_MBUTTON;
//         Msg.wParam &= ~MK_MBUTTON;
//         co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//     }
//
//     /* Right button */
//     if (dwFlags & MOUSEEVENTF_RIGHTDOWN)
//     {
//         SET_KEY_DOWN(gafAsyncKeyState, VK_RBUTTON, TRUE);
//         Msg.message = WM_RBUTTONDOWN;
//         pCurInfo->ButtonsDown |= MK_RBUTTON;
//         Msg.wParam |= MK_RBUTTON;
//         co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//     }
//     else if (dwFlags & MOUSEEVENTF_RIGHTUP)
//     {
//         SET_KEY_DOWN(gafAsyncKeyState, VK_RBUTTON, FALSE);
//         Msg.message = WM_RBUTTONUP;
//         pCurInfo->ButtonsDown &= ~MK_RBUTTON;
//         Msg.wParam &= ~MK_RBUTTON;
//         co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//     }
//
//     if((dwFlags & (MOUSEEVENTF_XDOWN | MOUSEEVENTF_XUP)) &&
//        (dwFlags & MOUSEEVENTF_WHEEL))
//     {
//         /* Fail because both types of events use the mouseData field */
//         WARN("Invalid flags!\n");
//         return FALSE;
//     }
//
//     /* X-Button (4 or 5) */
//     if (dwFlags & MOUSEEVENTF_XDOWN)
//     {
//         Msg.message = WM_XBUTTONDOWN;
//         if (pmi->mouseData & XBUTTON1)
//         {
//             SET_KEY_DOWN(gafAsyncKeyState, VK_XBUTTON1, TRUE);
//             pCurInfo->ButtonsDown |= MK_XBUTTON1;
//             Msg.wParam |= MAKEWPARAM(MK_XBUTTON1, XBUTTON1);
//             co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//         }
//         if (pmi->mouseData & XBUTTON2)
//         {
//             SET_KEY_DOWN(gafAsyncKeyState, VK_XBUTTON2, TRUE);
//             pCurInfo->ButtonsDown |= MK_XBUTTON2;
//             Msg.wParam |= MAKEWPARAM(MK_XBUTTON2, XBUTTON2);
//             co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//         }
//     }
//     else if (dwFlags & MOUSEEVENTF_XUP)
//     {
//         Msg.message = WM_XBUTTONUP;
//         if(pmi->mouseData & XBUTTON1)
//         {
//             SET_KEY_DOWN(gafAsyncKeyState, VK_XBUTTON1, FALSE);
//             pCurInfo->ButtonsDown &= ~MK_XBUTTON1;
//             Msg.wParam &= ~MK_XBUTTON1;
//             Msg.wParam |= MAKEWPARAM(0, XBUTTON2);
//             co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//         }
//         if (pmi->mouseData & XBUTTON2)
//         {
//             SET_KEY_DOWN(gafAsyncKeyState, VK_XBUTTON2, FALSE);
//             pCurInfo->ButtonsDown &= ~MK_XBUTTON2;
//             Msg.wParam &= ~MK_XBUTTON2;
//             Msg.wParam |= MAKEWPARAM(0, XBUTTON2);
//             co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//         }
//     }
//
//     /* Mouse wheel */
//     if (dwFlags & MOUSEEVENTF_WHEEL)
//     {
//         Msg.message = WM_MOUSEWHEEL;
//         Msg.wParam = MAKEWPARAM(pCurInfo->ButtonsDown, pmi->mouseData);
//         co_MsqInsertMouseMessage(&Msg, bInjected, pmi->dwExtraInfo, TRUE);
//     }
//
//     return TRUE;
// }
//
// VOID
// FASTCALL
// IntRemoveTrackMouseEvent(
//     PDESKTOP pDesk)
// {
//     /* Generate a leave message */
//     if (pDesk->dwDTFlags & DF_TME_LEAVE)
//     {
//         UINT uMsg = (pDesk->htEx != HTCLIENT) ? WM_NCMOUSELEAVE : WM_MOUSELEAVE;
//         UserPostMessage(UserHMGetHandle(pDesk->spwndTrack), uMsg, 0, 0);
//     }
//     /* Kill the timer */
//     if (pDesk->dwDTFlags & DF_TME_HOVER)
//         IntKillTimer(pDesk->spwndTrack, ID_EVENT_SYSTIMER_MOUSEHOVER, TRUE);
//
//     /* Reset state */
//     pDesk->dwDTFlags &= ~(DF_TME_LEAVE|DF_TME_HOVER);
//     pDesk->spwndTrack = NULL;
// }
//
// BOOL
// FASTCALL
// IntQueryTrackMouseEvent(
//     LPTRACKMOUSEEVENT lpEventTrack)
// {
//     PDESKTOP pDesk;
//     PTHREADINFO pti;
//
//     pti = PsGetCurrentThreadWin32Thread();
//     pDesk = pti->rpdesk;
//
//     /* Always cleared with size set and return true. */
//     RtlZeroMemory(lpEventTrack , sizeof(TRACKMOUSEEVENT));
//     lpEventTrack->cbSize = sizeof(TRACKMOUSEEVENT);
//
//     if (pDesk->dwDTFlags & (DF_TME_LEAVE | DF_TME_HOVER) &&
//         pDesk->spwndTrack &&
//         pti->MessageQueue == pDesk->spwndTrack->head.pti->MessageQueue)
//     {
//         if (pDesk->htEx != HTCLIENT)
//             lpEventTrack->dwFlags |= TME_NONCLIENT;
//
//         if (pDesk->dwDTFlags & DF_TME_LEAVE)
//             lpEventTrack->dwFlags |= TME_LEAVE;
//
//         if (pDesk->dwDTFlags & DF_TME_HOVER)
//         {
//             lpEventTrack->dwFlags |= TME_HOVER;
//             lpEventTrack->dwHoverTime = pDesk->dwMouseHoverTime;
//         }
//         lpEventTrack->hwndTrack = UserHMGetHandle(pDesk->spwndTrack);
//     }
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// IntTrackMouseEvent(
//     LPTRACKMOUSEEVENT lpEventTrack)
// {
//     PDESKTOP pDesk;
//     PTHREADINFO pti;
//     PWND pWnd;
//     POINT point;
//
//     pti = PsGetCurrentThreadWin32Thread();
//     pDesk = pti->rpdesk;
//
//     if (!(pWnd = UserGetWindowObject(lpEventTrack->hwndTrack)))
//         return FALSE;
//
//     if ( pDesk->spwndTrack != pWnd ||
//             (pDesk->htEx != HTCLIENT) ^ !!(lpEventTrack->dwFlags & TME_NONCLIENT) )
//     {
//         if ( lpEventTrack->dwFlags & TME_LEAVE && !(lpEventTrack->dwFlags & TME_CANCEL) )
//         {
//             UserPostMessage( lpEventTrack->hwndTrack,
//                              lpEventTrack->dwFlags & TME_NONCLIENT ? WM_NCMOUSELEAVE : WM_MOUSELEAVE,
//                              0, 0);
//         }
//         TRACE("IntTrackMouseEvent spwndTrack %p pwnd %p\n", pDesk->spwndTrack, pWnd);
//         return TRUE;
//     }
//
//     /* Tracking spwndTrack same as pWnd */
//     if (lpEventTrack->dwFlags & TME_CANCEL) // Canceled mode.
//     {
//         if (lpEventTrack->dwFlags & TME_LEAVE)
//             pDesk->dwDTFlags &= ~DF_TME_LEAVE;
//
//         if (lpEventTrack->dwFlags & TME_HOVER)
//         {
//             if (pDesk->dwDTFlags & DF_TME_HOVER)
//             {   // Kill hover timer.
//                 IntKillTimer(pWnd, ID_EVENT_SYSTIMER_MOUSEHOVER, TRUE);
//                 pDesk->dwDTFlags &= ~DF_TME_HOVER;
//             }
//         }
//     }
//     else // Not Canceled.
//     {
//         if (lpEventTrack->dwFlags & TME_LEAVE)
//             pDesk->dwDTFlags |= DF_TME_LEAVE;
//
//         if (lpEventTrack->dwFlags & TME_HOVER)
//         {
//             pDesk->dwDTFlags |= DF_TME_HOVER;
//
//             if (!lpEventTrack->dwHoverTime || lpEventTrack->dwHoverTime == HOVER_DEFAULT)
//                 pDesk->dwMouseHoverTime = gspv.iMouseHoverTime; // use the system default hover time-out.
//             else
//                 pDesk->dwMouseHoverTime = lpEventTrack->dwHoverTime;
//             // Start timer for the hover period.
//             IntSetTimer(pWnd, ID_EVENT_SYSTIMER_MOUSEHOVER, pDesk->dwMouseHoverTime, SystemTimerProc, TMRF_SYSTEM);
//             // Get windows thread message points.
//             point = pWnd->head.pti->ptLast;
//             // Set desktop mouse hover from the system default hover rectangle.
//             RECTL_vSetRect(&pDesk->rcMouseHover,
//                            point.x - gspv.iMouseHoverWidth  / 2,
//                            point.y - gspv.iMouseHoverHeight / 2,
//                            point.x + gspv.iMouseHoverWidth  / 2,
//                            point.y + gspv.iMouseHoverHeight / 2);
//         }
//     }
//     return TRUE;
// }
//
// BOOL
// __stdcall
// NtUserTrackMouseEvent(
//     LPTRACKMOUSEEVENT lpEventTrack)
// {
//     TRACKMOUSEEVENT SafeTME;
//     BOOL bRet = FALSE;
//     NTSTATUS Status;
//
//     TRACE("Enter NtUserTrackMouseEvent\n");
//
//     {
//         NTSTATUS __SehStatus = SafeProbeForRead(lpEventTrack, sizeof(TRACKMOUSEEVENT), 1);
//         if (!NT_SUCCESS(__SehStatus))
//         {
//             SetLastNtError(__SehStatus);
//             return FALSE;
//         }
//     }
//     RtlCopyMemory(&SafeTME, lpEventTrack, sizeof(TRACKMOUSEEVENT));
//
//     if (SafeTME.cbSize != sizeof(TRACKMOUSEEVENT))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (SafeTME.dwFlags & ~(TME_CANCEL | TME_QUERY | TME_NONCLIENT | TME_LEAVE | TME_HOVER) )
//     {
//         EngSetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
//     UserEnterExclusive();
//
//     if (SafeTME.dwFlags & TME_QUERY)
//     {
//         bRet = IntQueryTrackMouseEvent(&SafeTME);
//         Status = SafeProbeForWrite(lpEventTrack, sizeof(TRACKMOUSEEVENT), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             bRet = FALSE;
//         }
//         else
//         {
//             RtlCopyMemory(lpEventTrack, &SafeTME, sizeof(TRACKMOUSEEVENT));
//         }
//     }
//     else
//     {
//         bRet = IntTrackMouseEvent(&SafeTME);
//     }
//
//     TRACE("Leave NtUserTrackMouseEvent, ret=%i\n", bRet);
//     UserLeave();
//     return bRet;
// }
//
//
// DWORD
// __stdcall
// NtUserGetMouseMovePointsEx(
//     UINT cbSize,
//     LPMOUSEMOVEPOINT lpptIn,
//     LPMOUSEMOVEPOINT lpptOut,
//     int nBufPoints,
//     DWORD resolution)
// {
//     MOUSEMOVEPOINT Safeppt;
//     //BOOL Hit;
//     INT iRet = -1;
//
//     TRACE("Enter NtUserGetMouseMovePointsEx\n");
//
//     if ((cbSize != sizeof(MOUSEMOVEPOINT)) || (nBufPoints < 0) || (nBufPoints > 64))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return (DWORD)-1;
//     }
//
//     if (!lpptIn || (!lpptOut && nBufPoints))
//     {
//         EngSetLastError(ERROR_NOACCESS);
//         return (DWORD)-1;
//     }
//
//     {
//         NTSTATUS __SehStatus = SafeProbeForRead(lpptIn, cbSize, 1);
//         if (!NT_SUCCESS(__SehStatus))
//         {
//             SetLastNtError(__SehStatus);
//             return (DWORD)-1;
//         }
//     }
//     RtlCopyMemory(&Safeppt, lpptIn, cbSize);
//
//     UserEnterShared();
//
//     // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmousemovepointsex
//     // This explains the math issues in transforming points.
//     iRet = gcMouseHistoryOfMoves; // FIFO is forward so retrieve backward.
//     //Hit = FALSE;
//     do
//     {
//         if (Safeppt.x == 0 && Safeppt.y == 0)
//             break; // No test.
//         // Finds the point, it returns the last nBufPoints prior to and including the supplied point.
//         if (gMouseHistoryOfMoves[iRet].x == Safeppt.x && gMouseHistoryOfMoves[iRet].y == Safeppt.y)
//         {
//             if (Safeppt.time) // Now test time and it seems to be absolute.
//             {
//                 if (Safeppt.time == gMouseHistoryOfMoves[iRet].time)
//                 {
//                     //Hit = TRUE;
//                     break;
//                 }
//                 else
//                 {
//                     if (--iRet < 0) iRet = 63;
//                     continue;
//                 }
//             }
//             //Hit = TRUE;
//             break;
//         }
//         if (--iRet < 0) iRet = 63;
//     }
//     while (iRet != gcMouseHistoryOfMoves);
//
//     switch(resolution)
//     {
//         case GMMP_USE_DISPLAY_POINTS:
//             if (nBufPoints)
//             {
//                 {
//                     NTSTATUS __SehStatus = SafeProbeForWrite(lpptOut, cbSize, 1);
//                     if (!NT_SUCCESS(__SehStatus))
//                     {
//                         SetLastNtError(__SehStatus);
//                         iRet = -1;
//                         goto cleanup;
//                     }
//                 }
//             }
//             iRet = nBufPoints;
//             break;
//         case GMMP_USE_HIGH_RESOLUTION_POINTS:
//             break;
//         default:
//             EngSetLastError(ERROR_POINT_NOT_FOUND);
//             iRet = -1;
//     }
//
// cleanup:
//     TRACE("Leave NtUserGetMouseMovePointsEx, ret=%i\n", iRet);
//     UserLeave();
//     return (DWORD)iRet;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
