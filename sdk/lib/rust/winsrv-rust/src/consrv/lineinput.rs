use super::{GetConsoleInputBufferMode, GetType};
use super::term::*;
use super::popup::PPOPUP_WINDOW;
use winsrv_sys::winsrv::*;

// Forward declarations for functions from other modules.
// Term* macros are provided by super::term::* (imported above);
// GetType/GetConsoleInputBufferMode are bindgen-resolved.
extern "C" {
    // history.rs defines the History* helpers cdecl (§9.30 cc-match rule)
 fn HistoryFindEntryByPrefix( Console: PCONSRV_CONSOLE, ExeName: PUNICODE_STRING, Prefix: *const _UNICODE_STRING, EntryFound: *mut _UNICODE_STRING, ) -> BOOL;
}
extern "C" {
    // history.rs defines the History* helpers cdecl (§9.30 cc-match rule)
 fn HistoryDisplayCurrentHistory( Console: PCONSRV_CONSOLE, ExeName: PUNICODE_STRING, ) -> PPOPUP_WINDOW;
}
extern "C" {
    // history.rs defines the History* helpers cdecl (§9.30 cc-match rule)
 fn HistoryDeleteCurrentBuffer(Console: PCONSRV_CONSOLE, ExeName: PUNICODE_STRING);
}
extern "C" {
    // history.rs defines the History* helpers cdecl (§9.30 cc-match rule)
 fn HistoryGetCurrentEntry( Console: PCONSRV_CONSOLE, ExeName: PUNICODE_STRING, Entry: *mut _UNICODE_STRING, );
}
extern "C" {
    // history.rs defines the History* helpers cdecl (§9.30 cc-match rule)
 fn HistoryRecallHistory( Console: PCONSRV_CONSOLE, ExeName: PUNICODE_STRING, Offset: i32, Entry: *mut _UNICODE_STRING, ) -> BOOL;
}
extern "system" {
    pub fn HistoryAddEntry( Console: PCONSRV_CONSOLE, ExeName: PUNICODE_STRING, Entry: *const _UNICODE_STRING, );
    pub fn DestroyPopupWindow(Popup: PPOPUP_WINDOW);
}

static mut Popup: PPOPUP_WINDOW = core::ptr::null_mut();

pub(crate) unsafe fn LineInputSetPos(Console: PCONSRV_CONSOLE, NewPos: u32) {
    // `LinePosVal` collides with winsrv_sys::winsrv::LinePosVal (_BUS_DATA_TYPE_Pos
    // enum variant glob-imported via use winsrv_sys::winsrv::*).
    // Reference the parameter as `NewPos` instead.
    if NewPos != (*Console).LinePos && (GetConsoleInputBufferMode(Console) & ENABLE_ECHO_INPUT as u16) != 0 {
        let Buffer = (*Console).ActiveBuffer;
        let OldCursorX = (*Buffer).CursorPosition.X;
        let OldCursorY = (*Buffer).CursorPosition.Y;
        let mut XY = OldCursorY as i32 * (*Buffer).ScreenBufferSize.X as i32 + OldCursorX as i32;

        XY += NewPos as i32 - (*Console).LinePos as i32;
        if XY < 0 {
            XY = 0;
        } else if XY >= (*Buffer).ScreenBufferSize.Y as i32 * (*Buffer).ScreenBufferSize.X as i32 {
            XY = (*Buffer).ScreenBufferSize.Y as i32 * (*Buffer).ScreenBufferSize.X as i32 - 1;
        }

        (*Buffer).CursorPosition.X = (XY % (*Buffer).ScreenBufferSize.X as i32) as i16;
        (*Buffer).CursorPosition.Y = (XY / (*Buffer).ScreenBufferSize.X as i32) as i16;
        TermSetScreenInfo(Console, Buffer, OldCursorX, OldCursorY);
    }

    (*Console).LinePos = NewPos;
}

pub(crate) unsafe fn LineInputEdit(
    Console: PCONSRV_CONSOLE,
    NumToDelete: u32,
    NumToInsert: u32,
    Insertion: *const u16,
) {
    let LinePosVal: u32 = (*Console).LinePos;
    let NewSize: u32 = (*Console).LineSize - NumToDelete + NumToInsert;

    if GetType((*Console).ActiveBuffer) != TEXTMODE_BUFFER {
        return;
    }
    let ActiveBuffer = (*Console).ActiveBuffer as PTEXTMODE_SCREEN_BUFFER;

    if NewSize + 2 > (*Console).LineMaxSize {
        return;
    }

    core::ptr::copy(
        (*Console).LineBuffer.add((LinePosVal + NumToDelete) as usize),
        (*Console).LineBuffer.add((LinePosVal + NumToInsert) as usize),
        ((*Console).LineSize - (LinePosVal + NumToDelete)) as usize,
    );
    if !Insertion.is_null() {
        core::ptr::copy_nonoverlapping(
            Insertion,
            (*Console).LineBuffer.add(LinePosVal as usize),
            NumToInsert as usize,
        );
    }

    if (GetConsoleInputBufferMode(Console) & ENABLE_ECHO_INPUT as u16) != 0 {
        if LinePosVal < NewSize {
            TermWriteStream(
                Console,
                ActiveBuffer,
                (*Console).LineBuffer.add(LinePosVal as usize),
                NewSize - LinePosVal,
                1,
            );
        }
        let mut i = NewSize;
        while i < (*Console).LineSize {
            static SPACE: [u16; 1] = [b' ' as u16];
            TermWriteStream(Console, ActiveBuffer, SPACE.as_ptr() as *mut u16, 1, 1);
            i += 1;
        }
        (*Console).LinePos = i;
    }

    (*Console).LineSize = NewSize;
    LineInputSetPos(Console, LinePosVal + NumToInsert);
}

pub(crate) unsafe fn LineInputRecallHistory(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
    Offset: i32,
) {
    let mut Entry: _UNICODE_STRING = core::mem::zeroed();

    if HistoryRecallHistory(Console, ExeName, Offset, &mut Entry) == 0 {
        return;
    }

    LineInputSetPos(Console, 0);
    LineInputEdit(
        Console,
        (*Console).LineSize,
        Entry.Length as u32 / core::mem::size_of::<u16>() as u32,
        Entry.Buffer,
    );
}

// condrv flip: terminal.rs declares this cdecl `_LineInputKeyDown` (C: VOID, no NTAPI).
#[no_mangle]
pub unsafe extern "C" fn LineInputKeyDown(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
    KeyEvent: *mut KEY_EVENT_RECORD,
) {
    let mut LinePosVal: u32 = (*Console).LinePos;
    let mut Entry: _UNICODE_STRING = core::mem::zeroed();

    match (*KeyEvent).wVirtualKeyCode as u32 {
        VK_ESCAPE => {
            LineInputSetPos(Console, 0);
            LineInputEdit(Console, (*Console).LineSize, 0, core::ptr::null());

            if !Popup.is_null() {
                DestroyPopupWindow(Popup);
                Popup = core::ptr::null_mut();
            }
            return;
        }

        VK_HOME => {
            LineInputSetPos(Console, 0);
            if ((*KeyEvent).dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0 {
                LineInputEdit(Console, LinePosVal, 0, core::ptr::null());
            }
            return;
        }

        VK_END => {
            if ((*KeyEvent).dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0 {
                LineInputEdit(Console, (*Console).LineSize - LinePosVal, 0, core::ptr::null());
            } else {
                LineInputSetPos(Console, (*Console).LineSize);
            }
            return;
        }

        VK_LEFT => {
            if ((*KeyEvent).dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0 {
                while LinePosVal > 0 && *(*Console).LineBuffer.add((LinePosVal - 1) as usize) == b' ' as u16 {
                    LinePosVal -= 1;
                }
                while LinePosVal > 0 && *(*Console).LineBuffer.add((LinePosVal - 1) as usize) != b' ' as u16 {
                    LinePosVal -= 1;
                }
            } else {
                LinePosVal -= if LinePosVal > 0 { 1u32 } else { 0u32 };
            }
            LineInputSetPos(Console, LinePosVal);
            return;
        }

        VK_RIGHT | VK_F1 => {
            if ((*KeyEvent).dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0 {
                while LinePosVal < (*Console).LineSize
                    && *(*Console).LineBuffer.add(LinePosVal as usize) != b' ' as u16
                {
                    LinePosVal += 1;
                }
                while LinePosVal < (*Console).LineSize
                    && *(*Console).LineBuffer.add(LinePosVal as usize) == b' ' as u16
                {
                    LinePosVal += 1;
                }
                LineInputSetPos(Console, LinePosVal);
            } else {
                HistoryGetCurrentEntry(Console, ExeName, &mut Entry);
                if LinePosVal < (*Console).LineSize {
                    LineInputSetPos(Console, LinePosVal + 1);
                } else if (LinePosVal as usize * core::mem::size_of::<u16>()) < Entry.Length as usize {
                    LineInputEdit(Console, 0, 1, Entry.Buffer.add(LinePosVal as usize));
                }
            }
            return;
        }

        VK_INSERT => {
            (*Console).LineInsertToggle = if (*Console).LineInsertToggle != 0 { 0 } else { 1 };
            TermSetCursorInfo(Console, (*Console).ActiveBuffer);
            return;
        }

        VK_DELETE => {
            if LinePosVal != (*Console).LineSize {
                LineInputEdit(Console, 1, 0, core::ptr::null());
            }
            return;
        }

        VK_PRIOR => {
            LineInputRecallHistory(Console, ExeName, -(0xFFFF_u16 as i32));
            return;
        }

        VK_NEXT => {
            LineInputRecallHistory(Console, ExeName, 0xFFFF_u16 as i32);
            return;
        }

        VK_UP | VK_F5 => {
            LineInputRecallHistory(
                Console,
                ExeName,
                if (*Console).LineUpPressed != 0 { -1 } else { 0 },
            );
            (*Console).LineUpPressed = 1;
            return;
        }

        VK_DOWN => {
            LineInputRecallHistory(Console, ExeName, 1);
            return;
        }

        VK_F3 => {
            HistoryGetCurrentEntry(Console, ExeName, &mut Entry);
            if (LinePosVal as usize * core::mem::size_of::<u16>()) < Entry.Length as usize {
                let InsertSize = (Entry.Length as u32 / core::mem::size_of::<u16>() as u32) - LinePosVal;
                let DeleteSize = core::cmp::min((*Console).LineSize - LinePosVal, InsertSize);
                LineInputEdit(
                    Console,
                    DeleteSize,
                    InsertSize,
                    Entry.Buffer.add(LinePosVal as usize),
                );
            }
            return;
        }

        VK_F6 => {
            (*KeyEvent).uChar.UnicodeChar = 26;
        }

        VK_F7 => {
            if ((*KeyEvent).dwControlKeyState & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED)) != 0 {
                HistoryDeleteCurrentBuffer(Console, ExeName);
            } else {
                if !Popup.is_null() {
                    DestroyPopupWindow(Popup);
                }
                Popup = HistoryDisplayCurrentHistory(Console, ExeName);
            }
            return;
        }

        VK_F8 => {
            Entry.Length = ((*Console).LinePos as u16)
                .wrapping_mul(core::mem::size_of::<u16>() as u16);
            Entry.Buffer = (*Console).LineBuffer;

            let mut EntryFound: _UNICODE_STRING = core::mem::zeroed();
            if HistoryFindEntryByPrefix(Console, ExeName, &Entry, &mut EntryFound) != 0 {
                LineInputEdit(
                    Console,
                    (*Console).LineSize - LinePosVal,
                    (EntryFound.Length as u32 / core::mem::size_of::<u16>() as u32) - LinePosVal,
                    EntryFound.Buffer.add(LinePosVal as usize),
                );
                LineInputSetPos(Console, LinePosVal);
            }

            return;
        }

        _ => return,
    }

    if (*KeyEvent).uChar.UnicodeChar == b'\x08' as u16
        && (GetConsoleInputBufferMode(Console) & ENABLE_PROCESSED_INPUT as u16) != 0
    {
        if LinePosVal > 0 {
            LineInputSetPos(Console, LinePosVal - 1);
            LineInputEdit(Console, 1, 0, core::ptr::null());
        }
    } else if (*KeyEvent).uChar.UnicodeChar == b'\r' as u16 {
        if (GetConsoleInputBufferMode(Console) & ENABLE_ECHO_INPUT as u16) != 0 {
            Entry.Length = ((*Console).LineSize as u16)
                .wrapping_mul(core::mem::size_of::<u16>() as u16);
            Entry.MaximumLength = Entry.Length;
            Entry.Buffer = (*Console).LineBuffer;
            HistoryAddEntry(Console, ExeName, &Entry);
        }

        LineInputSetPos(Console, (*Console).LineSize);
        *(*Console).LineBuffer.add((*Console).LineSize as usize) = b'\r' as u16;
        (*Console).LineSize += 1;
        if GetType((*Console).ActiveBuffer) == TEXTMODE_BUFFER
            && (GetConsoleInputBufferMode(Console) & ENABLE_ECHO_INPUT as u16) != 0
        {
            static CR: [u16; 1] = [b'\r' as u16];
            TermWriteStream(
                Console,
                (*Console).ActiveBuffer as PTEXTMODE_SCREEN_BUFFER,
                CR.as_ptr() as *mut u16,
                1,
                1,
            );
        }

        if (GetConsoleInputBufferMode(Console) & ENABLE_PROCESSED_INPUT as u16) != 0
            && (*Console).LineSize < (*Console).LineMaxSize
        {
            *(*Console).LineBuffer.add((*Console).LineSize as usize) = b'\n' as u16;
            (*Console).LineSize += 1;
            if GetType((*Console).ActiveBuffer) == TEXTMODE_BUFFER
                && (GetConsoleInputBufferMode(Console) & ENABLE_ECHO_INPUT as u16) != 0
            {
                static LF: [u16; 1] = [b'\n' as u16];
                TermWriteStream(
                    Console,
                    (*Console).ActiveBuffer as PTEXTMODE_SCREEN_BUFFER,
                    LF.as_ptr() as *mut u16,
                    1,
                    1,
                );
            }
        }
        (*Console).LineComplete = 1;
        (*Console).LinePos = 0;
    } else if (*KeyEvent).uChar.UnicodeChar != 0 {
        if (*KeyEvent).uChar.UnicodeChar < 0x20
            && ((*Console).LineWakeupMask & (1 << (*KeyEvent).uChar.UnicodeChar)) != 0
        {
            *(*Console).LineBuffer.add((*Console).LineSize as usize) = b' ' as u16;
            (*Console).LineSize += 1;
            *(*Console).LineBuffer.add((*Console).LinePos as usize) =
                (*KeyEvent).uChar.UnicodeChar;
            (*Console).LineComplete = 1;
            (*Console).LinePos = 0;
        } else {
            let Overstrike =
                (*Console).LineInsertToggle == 0 && (*Console).LinePos != (*Console).LineSize;
            LineInputEdit(
                Console,
                if Overstrike { 1 } else { 0 },
                1,
                core::ptr::addr_of!((*KeyEvent).uChar.UnicodeChar),
            );
        }
    }
}