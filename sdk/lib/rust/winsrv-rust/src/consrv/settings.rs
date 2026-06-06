use winsrv_sys::nt::*;
use super::condrv::text::*;
use super::condrv::conoutput::*;
use super::condrv::coninput::*;
use super::condrv::console::*;
use super::console::*;
use super::handle::*;
use super::term::*;
use super::lineinput::*;
use super::history::*;
use super::init::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;

extern "C" {
    static s_Colors: [COLORREF; 16];
}

extern "system" {
    // NTSTATUS NTAPI — stdcall (`_ConDrvChangeScreenBufferAttributes@16` on i386).
    pub fn ConDrvChangeScreenBufferAttributes( Console: PCONSOLE, Buffer: PTEXTMODE_SCREEN_BUFFER, NewScreenAttrib: u16, NewPopupAttrib: u16, ) -> i32;
}
// condrv flip: cdecl — Term* are term.rs vtbl wrappers; ConioResizeBuffer is
// C plain-cdecl (NTSTATUS, no NTAPI). Match the extern "C" definitions.
// HistoryReshapeAllBuffers is plain `VOID` in C (no NTAPI) → cdecl; its
// definition in history.rs is `#[no_mangle] extern "C"` (`_HistoryReshapeAllBuffers`).
// It was previously declared `extern "system"` here (`_..@16`), an ABI/decoration
// mismatch that only linked via lld stdcall-fixup — now declared correctly.
extern "C" {
    pub fn TermSetCodePage( Console: PCONSRV_CONSOLE, CodePage: u32, ) -> u8;
    pub fn TermResizeTerminal(Console: PCONSRV_CONSOLE);
    pub fn ConioResizeBuffer( Console: PCONSOLE, Buffer: PTEXTMODE_SCREEN_BUFFER, Size: COORD, ) -> i32;
    pub fn HistoryReshapeAllBuffers( Console: PCONSRV_CONSOLE, HistoryBufferSize: u32, NumberOfHistoryBuffers: u32, HistoryNoDup: u32, );
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvApplyUserSettings(
    Console: PCONSRV_CONSOLE,
    ConsoleInfo: PCONSOLE_STATE_INFO,
) {
    let ActiveBuffer = (*Console).ActiveBuffer;

    (*Console).QuickEdit = if (*ConsoleInfo).QuickEdit != 0 { 1 } else { 0 };
    (*Console).InsertMode = if (*ConsoleInfo).InsertMode != 0 { 1 } else { 0 };
    
    HistoryReshapeAllBuffers(
        Console,
        (*ConsoleInfo).HistoryBufferSize,
        (*ConsoleInfo).NumberOfHistoryBuffers,
        (*ConsoleInfo).HistoryNoDup as u32,
    );

    core::ptr::copy_nonoverlapping(
        (*ConsoleInfo).ColorTable.as_ptr() as *const u8,
        (*Console).Colors.as_mut_ptr() as *mut u8,
        core::mem::size_of_val(&s_Colors),
    );

    (*ActiveBuffer).CursorInfo.bVisible = if (*ConsoleInfo).CursorSize != 0 { 1 } else { 0 };
    (*ActiveBuffer).CursorInfo.dwSize = {
        let size = (*ConsoleInfo).CursorSize;
        if size < 0 { 0 } else if size > 100 { 100 } else { size as u32 }
    };

    if ((*Console).OutputCodePage != (*ConsoleInfo).CodePage) &&
        IsValidCodePage((*ConsoleInfo).CodePage) != 0
    {
        if TermSetCodePage(Console, (*ConsoleInfo).CodePage) != 0 {
            (*Console).OutputCodePage = (*ConsoleInfo).CodePage;
            (*Console).InputCodePage = (*ConsoleInfo).CodePage;
        }
    }

    if (*ActiveBuffer).Header.Type as u32 == TEXTMODE_BUFFER {
        let Buffer = ActiveBuffer as PTEXTMODE_SCREEN_BUFFER;
        let BufSize = (*ConsoleInfo).ScreenBufferSize;

        if (*Console).FixedSize != 0 {
            if (*ConsoleInfo).WindowSize.X != (*ActiveBuffer).OldViewSize.X ||
                (*ConsoleInfo).WindowSize.Y != (*ActiveBuffer).OldViewSize.Y
            {
                (*ActiveBuffer).OldViewSize = (*ConsoleInfo).WindowSize;
            }

            if BufSize.X >= (*ActiveBuffer).OldViewSize.X && BufSize.Y >= (*ActiveBuffer).OldViewSize.Y {
                if BufSize.X != (*ActiveBuffer).OldScreenBufferSize.X ||
                    BufSize.Y != (*ActiveBuffer).OldScreenBufferSize.Y
                {
                    (*ActiveBuffer).OldScreenBufferSize = BufSize;
                }
            }
        } else {
            let mut SizeChanged = 0u8;

            if (*ConsoleInfo).WindowSize.X != (*ActiveBuffer).ViewSize.X ||
                (*ConsoleInfo).WindowSize.Y != (*ActiveBuffer).ViewSize.Y
            {
                (*ActiveBuffer).ViewSize = (*ConsoleInfo).WindowSize;
                SizeChanged = 1;
            }

            if BufSize.X != (*ActiveBuffer).ScreenBufferSize.X ||
                BufSize.Y != (*ActiveBuffer).ScreenBufferSize.Y
            {
                if NT_SUCCESS(ConioResizeBuffer(Console as PCONSOLE, Buffer, BufSize)) {
                    SizeChanged = 1;
                }
            }

            if SizeChanged != 0 {
                TermResizeTerminal(Console);
            }
        }

        ConDrvChangeScreenBufferAttributes(
            Console as PCONSOLE,
            Buffer,
            (*ConsoleInfo).ScreenAttributes,
            (*ConsoleInfo).PopupAttributes,
        );
    } else {
        if (*Console).FixedSize != 0 {
            if (*ConsoleInfo).WindowSize.X <= (*ActiveBuffer).ViewSize.X ||
                (*ConsoleInfo).WindowSize.Y <= (*ActiveBuffer).ViewSize.Y
            {
                (*ActiveBuffer).OldViewSize = (*ConsoleInfo).WindowSize;
            }
        } else {
            if (*ConsoleInfo).WindowSize.X <= (*ActiveBuffer).ViewSize.X ||
                (*ConsoleInfo).WindowSize.Y <= (*ActiveBuffer).ViewSize.Y
            {
                (*ActiveBuffer).ViewSize = (*ConsoleInfo).WindowSize;
            }
        }
    }
}