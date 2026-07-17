use winsrv_sys::nt::*;
use super::condrv::text::*;
use super::GetType;
use super::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;

// popup.h:15 — typedef struct _POPUP_WINDOW (not included by bindgen
// since popup.h isn't pulled by alias.c, the bindgen TU). Hand-written
// here so popup.rs/history.rs/lineinput.rs see it; pub-re-exported.
pub type PPOPUP_INPUT_ROUTINE = Option<unsafe extern "system" fn()>;

#[repr(C)]
pub struct _POPUP_WINDOW {
    pub ListEntry: LIST_ENTRY,
    pub ScreenBuffer: PTEXTMODE_SCREEN_BUFFER,
    pub Origin: COORD,
    pub Size: COORD,
    pub OldContents: PCHAR_INFO,
    pub PopupInputRoutine: PPOPUP_INPUT_ROUTINE,
}
pub type POPUP_WINDOW = _POPUP_WINDOW;
pub type PPOPUP_WINDOW = *mut _POPUP_WINDOW;

pub(crate) unsafe fn DrawBox(
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    xLeft: SHORT,
    yTop: SHORT,
    Width: SHORT,
    Height: SHORT,
) {
    let mut coPos: COORD = core::mem::zeroed();
    let mut Written: DWORD = 0;
    let mut Code: CODE_ELEMENT = core::mem::zeroed();

    coPos.X = xLeft;
    Code.Attribute = (*Buffer).PopupDefaultAttrib;
    coPos.Y = yTop;
    while coPos.Y < yTop + Height {
        ConDrvFillConsoleOutput(
            (*Buffer).Header.Console,
            Buffer,
            CODE_ATTRIBUTE,
            Code,
            Width as ULONG,
            &mut coPos as *mut COORD,
            &mut Written,
        );
        coPos.Y += 1;
    }

    coPos.X = xLeft;
    coPos.Y = yTop;
    Code.AsciiChar = 0xDAu8 as i8;
    ConDrvFillConsoleOutput(
        (*Buffer).Header.Console,
        Buffer,
        CODE_ASCII,
        Code,
        1,
        &mut coPos as *mut COORD,
        &mut Written,
    );

    coPos.X = xLeft + 1;
    coPos.Y = yTop;
    Code.AsciiChar = 0xC4u8 as i8;
    ConDrvFillConsoleOutput(
        (*Buffer).Header.Console,
        Buffer,
        CODE_ASCII,
        Code,
        (Width - 2) as ULONG,
        &mut coPos as *mut COORD,
        &mut Written,
    );

    coPos.X = xLeft + Width - 1;
    coPos.Y = yTop;
    Code.AsciiChar = 0xBFu8 as i8;
    ConDrvFillConsoleOutput(
        (*Buffer).Header.Console,
        Buffer,
        CODE_ASCII,
        Code,
        1,
        &mut coPos as *mut COORD,
        &mut Written,
    );

    coPos.Y = yTop + 1;
    while coPos.Y < yTop + Height - 1 {
        coPos.X = xLeft;
        Code.AsciiChar = 0xB3u8 as i8;
        ConDrvFillConsoleOutput(
            (*Buffer).Header.Console,
            Buffer,
            CODE_ASCII,
            Code,
            1,
            &mut coPos as *mut COORD,
            &mut Written,
        );

        coPos.X = xLeft + 1;
        Code.AsciiChar = b' ' as i8;
        ConDrvFillConsoleOutput(
            (*Buffer).Header.Console,
            Buffer,
            CODE_ASCII,
            Code,
            (Width - 2) as ULONG,
            &mut coPos as *mut COORD,
            &mut Written,
        );

        coPos.X = xLeft + Width - 1;
        Code.AsciiChar = 0xB3u8 as i8;
        ConDrvFillConsoleOutput(
            (*Buffer).Header.Console,
            Buffer,
            CODE_ASCII,
            Code,
            1,
            &mut coPos as *mut COORD,
            &mut Written,
        );

        coPos.Y += 1;
    }

    coPos.X = xLeft;
    coPos.Y = yTop + Height - 1;
    Code.AsciiChar = 0xC0u8 as i8;
    ConDrvFillConsoleOutput(
        (*Buffer).Header.Console,
        Buffer,
        CODE_ASCII,
        Code,
        1,
        &mut coPos as *mut COORD,
        &mut Written,
    );

    coPos.X = xLeft + 1;
    coPos.Y = yTop + Height - 1;
    Code.AsciiChar = 0xC4u8 as i8;
    ConDrvFillConsoleOutput(
        (*Buffer).Header.Console,
        Buffer,
        CODE_ASCII,
        Code,
        (Width - 2) as ULONG,
        &mut coPos as *mut COORD,
        &mut Written,
    );

    coPos.X = xLeft + Width - 1;
    coPos.Y = yTop + Height - 1;
    Code.AsciiChar = 0xD9u8 as i8;
    ConDrvFillConsoleOutput(
        (*Buffer).Header.Console,
        Buffer,
        CODE_ASCII,
        Code,
        1,
        &mut coPos as *mut COORD,
        &mut Written,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CreatePopupWindow(
    Console: PCONSRV_CONSOLE,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
    xLeft: SHORT,
    yTop: SHORT,
    Width: SHORT,
    Height: SHORT,
) -> PPOPUP_WINDOW {
    let Buffer: PTEXTMODE_SCREEN_BUFFER;
    let Popup: PPOPUP_WINDOW;
    let mut Region: SMALL_RECT = core::mem::zeroed();

    if (*ScreenBuffer).Header.Console as PCONSOLE != Console as PCONSOLE {
        return core::ptr::null_mut();
    }

    if GetType(ScreenBuffer) != TEXTMODE_BUFFER {
        return core::ptr::null_mut();
    }

    Buffer = ScreenBuffer as PTEXTMODE_SCREEN_BUFFER;

    Popup = ConsoleAllocHeap(HEAP_ZERO_MEMORY, core::mem::size_of::<POPUP_WINDOW>()) as PPOPUP_WINDOW;
    if Popup.is_null() {
        return core::ptr::null_mut();
    }

    (*Popup).ScreenBuffer = Buffer;
    (*Popup).Origin.X = xLeft;
    (*Popup).Origin.Y = yTop;
    (*Popup).Size.X = Width;
    (*Popup).Size.Y = Height;

    (*Popup).OldContents = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        ((*Popup).Size.X as usize) * ((*Popup).Size.Y as usize) * core::mem::size_of::<CHAR_INFO>(),
    ) as PCHAR_INFO;
    if (*Popup).OldContents.is_null() {
        ConsoleFreeHeap(Popup as PVOID);
        return core::ptr::null_mut();
    }

    Region.Left = (*Popup).Origin.X;
    Region.Top = (*Popup).Origin.Y;
    Region.Right = (*Popup).Origin.X + (*Popup).Size.X - 1;
    Region.Bottom = (*Popup).Origin.Y + (*Popup).Size.Y - 1;
    ConDrvReadConsoleOutput(
        (*Buffer).Header.Console,
        Buffer,
        1u8,
        (*Popup).OldContents,
        &mut Region as *mut SMALL_RECT,
    );

    DrawBox(Buffer, xLeft, yTop, Width, Height);

    InsertTailList(
        core::ptr::addr_of_mut!((*Console).PopupWindows) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Popup).ListEntry) as PLIST_ENTRY,
    );

    Popup
}

#[no_mangle]
pub unsafe extern "system" fn DestroyPopupWindow(Popup: PPOPUP_WINDOW) {
    let mut Region: SMALL_RECT = core::mem::zeroed();

    if Popup.is_null() {
        return;
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*Popup).ListEntry) as PLIST_ENTRY);

    Region.Left = (*Popup).Origin.X;
    Region.Top = (*Popup).Origin.Y;
    Region.Right = (*Popup).Origin.X + (*Popup).Size.X - 1;
    Region.Bottom = (*Popup).Origin.Y + (*Popup).Size.Y - 1;
    ConDrvWriteConsoleOutput(
        (*(*Popup).ScreenBuffer).Header.Console,
        (*Popup).ScreenBuffer,
        1u8,
        (*Popup).OldContents,
        &mut Region as *mut SMALL_RECT,
    );

    ConsoleFreeHeap((*Popup).OldContents as PVOID);
    ConsoleFreeHeap(Popup as PVOID);
}