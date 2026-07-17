use super::super::{ToUnicodeEx};
use super::super::coninput::ConioProcessInputEvent;
use winsrv_sys::winsrv::*;

static mut KeyState: [u8; 256] = [0; 256];
static mut LastVirtualKey: UINT = 0;

pub(crate) unsafe fn ConioGetShiftState(KeyStatePtr: PBYTE, lParam: LPARAM) -> DWORD {
    let mut ssOut = 0u32;

    if *KeyStatePtr.offset(VK_CAPITAL as isize) & 0x01 != 0 {
        ssOut |= CAPSLOCK_ON;
    }

    if *KeyStatePtr.offset(VK_NUMLOCK as isize) & 0x01 != 0 {
        ssOut |= NUMLOCK_ON;
    }

    if *KeyStatePtr.offset(VK_SCROLL as isize) & 0x01 != 0 {
        ssOut |= SCROLLLOCK_ON;
    }

    if *KeyStatePtr.offset(VK_SHIFT as isize) & 0x80 != 0 {
        ssOut |= SHIFT_PRESSED;
    }

    if *KeyStatePtr.offset(VK_LCONTROL as isize) & 0x80 != 0 {
        ssOut |= LEFT_CTRL_PRESSED;
    }
    if *KeyStatePtr.offset(VK_RCONTROL as isize) & 0x80 != 0 {
        ssOut |= RIGHT_CTRL_PRESSED;
    }

    if *KeyStatePtr.offset(VK_LMENU as isize) & 0x80 != 0 {
        ssOut |= LEFT_ALT_PRESSED;
    }
    if *KeyStatePtr.offset(VK_RMENU as isize) & 0x80 != 0 {
        ssOut |= RIGHT_ALT_PRESSED;
    }

    if ((lParam >> 16) as u32) & KF_EXTENDED != 0 {
        ssOut |= ENHANCED_KEY;
    }

    ssOut
}

#[no_mangle]
pub unsafe extern "system" fn ConioProcessKey(Console: PCONSRV_CONSOLE, msg: *mut MSG) {
    let ShiftState: DWORD;
    let UnicodeChar: WCHAR;
    let VirtualKeyCode: UINT;
    let VirtualScanCode: UINT;
    let Down: BOOL;
    let Fake: BOOLEAN;
    let NotChar: BOOLEAN;

    let mut er: INPUT_RECORD = core::mem::zeroed();

    if Console.is_null() {
        return;
    }

    VirtualScanCode = ((((*msg).lParam >> 16) as u16) & 0xFF) as UINT;
    Down = ((*msg).message == WM_KEYDOWN || (*msg).message == WM_CHAR ||
            (*msg).message == WM_SYSKEYDOWN || (*msg).message == WM_SYSCHAR) as BOOL;

    GetKeyboardState(KeyState.as_mut_ptr());
    ShiftState = ConioGetShiftState(KeyState.as_mut_ptr(), (*msg).lParam);

    if (*msg).message == WM_CHAR || (*msg).message == WM_SYSCHAR {
        VirtualKeyCode = LastVirtualKey;
        UnicodeChar = (*msg).wParam as WCHAR;
    } else {
        let mut Chars: [WCHAR; 2] = [0; 2];
        let RetChars: INT;

        VirtualKeyCode = (*msg).wParam as UINT;
        RetChars = ToUnicodeEx(
            VirtualKeyCode,
            VirtualScanCode,
            KeyState.as_mut_ptr(),
            Chars.as_mut_ptr(),
            2,
            0,
            core::ptr::null_mut(),
        );
        UnicodeChar = if RetChars == 1 { Chars[0] } else { 0 };
    }

    Fake = (UnicodeChar != 0 &&
            ((*msg).message != WM_CHAR && (*msg).message != WM_SYSCHAR &&
             (*msg).message != WM_KEYUP && (*msg).message != WM_SYSKEYUP)) as BOOLEAN;
    NotChar = ((*msg).message != WM_CHAR && (*msg).message != WM_SYSCHAR) as BOOLEAN;
    if NotChar != 0 {
        LastVirtualKey = (*msg).wParam as UINT;
    }

    if Fake != 0 {
        return;
    }

    er.EventType = KEY_EVENT as u16;
    er.Event.KeyEvent.bKeyDown = Down as i32;
    er.Event.KeyEvent.wRepeatCount = 1;
    er.Event.KeyEvent.wVirtualKeyCode = VirtualKeyCode as u16;
    er.Event.KeyEvent.wVirtualScanCode = VirtualScanCode as u16;
    er.Event.KeyEvent.uChar.UnicodeChar = UnicodeChar;
    er.Event.KeyEvent.dwControlKeyState = ShiftState;

    ConioProcessInputEvent(Console, &mut er as *mut INPUT_RECORD);
}

#[no_mangle]
pub unsafe extern "system" fn ConioEffectiveCursorSize(Console: PCONSRV_CONSOLE, Scale: DWORD) -> DWORD {
    let Size = ((*(*Console).ActiveBuffer).CursorInfo.dwSize * Scale + 99) / 100;
    if !(*Console).LineBuffer.is_null() && (*Console).LineComplete == 0 && 
       (if (*Console).InsertMode != 0 { (*Console).LineInsertToggle == 0 } else { (*Console).LineInsertToggle != 0 }) {
        return if Size * 2 <= Scale { Size * 2 } else { Size / 2 };
    }
    Size
}