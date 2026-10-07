use crate::usetup::console::AllocConsole;
use crate::usetup::console::FillConsoleOutputAttribute;
use crate::usetup::console::FillConsoleOutputCharacterA;
use crate::usetup::console::FlushConsoleInputBuffer;
use crate::usetup::console::FreeConsole;
use crate::usetup::console::GetConsoleScreenBufferInfo;
use crate::usetup::console::GetStdHandle;
use crate::usetup::console::PeekConsoleInput;
use crate::usetup::console::ReadConsoleInput;
use crate::usetup::console::SetConsoleCursorInfo;
use crate::usetup::console::SetConsoleCursorPosition;
use crate::usetup::console::WriteConsole;
use crate::usetup::console::WriteConsoleOutputCharacterA;
use crate::usetup::console::WriteConsoleOutputCharacterW;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut StdInput: HANDLE = NULL;

#[no_mangle]
pub static mut StdOutput: HANDLE = NULL;

#[no_mangle]
pub static mut xScreen: SHORT = 0 as SHORT;

#[no_mangle]
pub static mut yScreen: SHORT = 0 as SHORT;
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_Init() -> i32 {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    if AllocConsole() == 0 {
        return (FALSE) as i32;
    }
    StdInput = GetStdHandle(STD_INPUT_HANDLE);
    StdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
    if GetConsoleScreenBufferInfo(StdOutput, &mut csbi) == 0 {
        FreeConsole();
        return (FALSE) as i32;
    }
    xScreen = csbi.dwSize.X;
    yScreen = csbi.dwSize.Y;
    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ConInKey(Buffer: PINPUT_RECORD) {
    let mut Read: DWORD = 0;
    loop {
        ReadConsoleInput(StdInput, Buffer, 1, &mut Read);
        if ((*Buffer).EventType) as u32 == KEY_EVENT
            && ((*Buffer).Event.KeyEvent.bKeyDown) as u32 != FALSE
        {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ConInKeyPeek(
    Buffer: *mut INPUT_RECORD,
) -> BOOLEAN {
    let mut Read: DWORD = 0;

    loop {
        if PeekConsoleInput(StdInput, Buffer, 1, &mut Read) == 0 {
            return (FALSE) as BOOLEAN;
        }
        if Read == 0 {
            return (FALSE) as BOOLEAN;
        }

        ReadConsoleInput(StdInput, Buffer, 1, &mut Read);

        if ((*Buffer).EventType) as u32 == KEY_EVENT
            && ((*Buffer).Event.KeyEvent.bKeyDown) as u32 != FALSE
        {
            return (TRUE) as BOOLEAN;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ConOutChar(c: CHAR) {
    let mut Written: DWORD = 0;
    WriteConsole(
        StdOutput,
        core::ptr::addr_of!(c) as *const core::ffi::c_void,
        1,
        core::ptr::addr_of_mut!(Written),
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ConOutPuts(szText: LPCSTR) {
    let mut Written: u32 = 0;
    WriteConsole(
        StdOutput,
        (szText).cast(),
        strlen(szText) as u32,
        &mut Written,
        core::ptr::null_mut(),
    );
    WriteConsole(
        StdOutput,
        (b"\n\0".as_ptr() as *const i8).cast(),
        1,
        &mut Written,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ConOutPrintfV(
    szFormat: LPCSTR,
    args: va_list,
) {
    let mut szOut: [i8; 256] = [0; 256];
    let mut dwWritten: u32 = 0;

    vsprintf(
        szOut.as_mut_ptr(),
        szFormat,
        args,
    );

    WriteConsole(
        StdOutput,
        szOut.as_ptr() as *const core::ffi::c_void,
        strlen(szOut.as_ptr()) as u32,
        &mut dwWritten,
        core::ptr::null_mut(),
    );
}
extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn CONSOLE_ConOutPrintf(szFormat: LPCSTR, ...);
}

/// `CONSOLE_ConOutPrintf(szFormat, ...)`, defined through varargs-rust's naked thunk:
/// `a.get(0)` is `szFormat`, `a.va(1)` is the `va_list` C's `va_start(arg_ptr, szFormat)`
/// would have produced (`va_end` is a no-op on these ABIs).
mod CONSOLE_ConOutPrintf_export {
    use super::*;
    varargs_rust::va_entry!(CONSOLE_ConOutPrintf => |a| -> () {
        let szFormat = a.get(0) as LPCSTR;
        CONSOLE_ConOutPrintfV(szFormat, a.va(1) as va_list);
    });
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_Flush() -> i32 {
    return FlushConsoleInputBuffer(StdInput);
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_GetCursorXY(x: PSHORT, y: PSHORT) {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();

    GetConsoleScreenBufferInfo(StdOutput, &mut csbi);

    *x = csbi.dwCursorPosition.X;
    *y = csbi.dwCursorPosition.Y;
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_GetCursorX() -> SHORT {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    GetConsoleScreenBufferInfo(StdOutput, &mut csbi);
    return csbi.dwCursorPosition.X;
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_GetCursorY() -> SHORT {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    GetConsoleScreenBufferInfo(StdOutput, &mut csbi);
    return csbi.dwCursorPosition.Y;
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetCursorType(bInsert: BOOL, bVisible: BOOL) {
    let mut cci: CONSOLE_CURSOR_INFO = CONSOLE_CURSOR_INFO {
        dwSize: 0,
        bVisible: 0,
    };
    cci.dwSize = if bInsert != 0 {
        10
    } else {
        99
    };
    cci.bVisible = bVisible;
    SetConsoleCursorInfo(StdOutput, core::ptr::addr_of_mut!(cci));
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetCursorXY(x: SHORT, y: SHORT) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    coPos.X = x;
    coPos.Y = y;
    SetConsoleCursorPosition(StdOutput, coPos);
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ClearScreen() {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: u32 = 0;

    coPos.X = 0;
    coPos.Y = 0;

    FillConsoleOutputAttribute(
        StdOutput,
        FOREGROUND_BLUE | BACKGROUND_BLUE,
        (xScreen as i32 * yScreen as i32) as u32,
        coPos,
        &mut Written,
    );

    FillConsoleOutputCharacterA(
        StdOutput,
        (' ' as i32) as CHAR,
        (xScreen as i32 * yScreen as i32) as u32,
        coPos,
        &mut Written,
    );

    FillConsoleOutputAttribute(
        StdOutput,
        (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as WORD,
        (xScreen as i32 * yScreen as i32) as u32,
        coPos,
        &mut Written,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_InvertTextXY(
    x: SHORT,
    y: SHORT,
    col: SHORT,
    row: SHORT,
) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;

    coPos.Y = y;
    while (coPos.Y as i32) < (y as i32 + row as i32) {
        coPos.X = x;

        FillConsoleOutputAttribute(
            StdOutput,
            FOREGROUND_BLUE | (BACKGROUND_WHITE) as u16,
            col as DWORD,
            coPos,
            &mut Written,
        );

        coPos.Y += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_NormalTextXY(
    x: SHORT,
    y: SHORT,
    col: SHORT,
    row: SHORT,
) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    coPos.Y = y;
    while (coPos.Y as i32) < (y as i32 + row as i32) {
        coPos.X = x;
        FillConsoleOutputAttribute(
            StdOutput,
            (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as WORD,
            col as DWORD,
            coPos,
            core::ptr::addr_of_mut!(Written),
        );
        coPos.Y += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetTextXY(x: SHORT, y: SHORT, Text: LPCSTR) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;

    coPos.X = x;
    coPos.Y = y;

    WriteConsoleOutputCharacterA(
        StdOutput,
        Text,
        strlen(Text) as ULONG,
        coPos,
        &mut Written,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ClearTextXY(x: SHORT, y: SHORT, Length: SHORT) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    coPos.X = x;
    coPos.Y = y;
    FillConsoleOutputCharacterA(
        StdOutput,
        ' ' as i32 as CHAR,
        Length as DWORD,
        coPos,
        &raw mut Written,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetInputTextXY(
    x: SHORT,
    y: SHORT,
    len: SHORT,
    Text: LPCWSTR,
) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Length: SHORT = 0;
    let mut Written: DWORD = 0;

    coPos.X = x;
    coPos.Y = y;

    Length = wcslen(Text) as SHORT;
    if Length > len - 1 {
        Length = len - 1;
    }

    FillConsoleOutputAttribute(
        StdOutput,
        (BACKGROUND_WHITE) as WORD,
        len as DWORD,
        coPos,
        &mut Written,
    );

    WriteConsoleOutputCharacterW(
        StdOutput,
        Text,
        Length as ULONG,
        coPos,
        &mut Written,
    );

    coPos.X += Length;
    if len > Length {
        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as CHAR,
            (len - Length) as DWORD,
            coPos,
            &mut Written,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetUnderlinedTextXY(
    x: SHORT,
    y: SHORT,
    Text: LPCSTR,
) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Length: DWORD = 0;
    let mut Written: DWORD = 0;

    coPos.X = x;
    coPos.Y = y;

    Length = strlen(Text) as ULONG as DWORD;

    WriteConsoleOutputCharacterA(
        StdOutput,
        Text,
        Length,
        coPos,
        core::ptr::addr_of_mut!(Written),
    );

    coPos.Y += 1;

    FillConsoleOutputCharacterA(
        StdOutput,
        CharDoubleHorizontalLine,
        Length,
        coPos,
        core::ptr::addr_of_mut!(Written),
    );
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetStatusTextXV(
    x: SHORT,
    fmt: LPCSTR,
    args: va_list,
) {
    let mut nLength: INT;
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    // C leaves Buffer uninitialized and memsets only its first min(sizeof, xScreen) bytes.
    let mut Buffer: [CHAR; 128] = [0; 128];

    memset(
        Buffer.as_mut_ptr() as *mut core::ffi::c_void,
        ' ' as i32,
        min(core::mem::size_of::<[CHAR; 128]>(), xScreen as usize),
    );
    nLength = vsprintf(
        Buffer.as_mut_ptr().offset(x as isize),
        (fmt as *const core::ffi::c_char).cast(),
        args,
    );

    Buffer[(x as INT + nLength) as usize] = ' ' as CHAR;

    coPos.X = 0;
    coPos.Y = yScreen - 1;
    FillConsoleOutputAttribute(
        StdOutput,
        (BACKGROUND_WHITE) as WORD,
        xScreen as DWORD,
        coPos,
        &mut Written,
    );
    WriteConsoleOutputCharacterA(
        StdOutput,
        Buffer.as_ptr() as LPCSTR,
        min(core::mem::size_of::<[CHAR; 128]>(), xScreen as usize) as DWORD,
        coPos,
        &mut Written,
    );
}

extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn CONSOLE_SetStatusTextX(x: SHORT, fmt: LPCSTR, ...);
}

/// `CONSOLE_SetStatusTextX(x, fmt, ...)`, defined through varargs-rust's naked thunk:
/// `a.get(i)` is fixed argument `i`, `a.va(2)` is the `va_list` of `va_start(ap, fmt)`.
mod CONSOLE_SetStatusTextX_export {
    use super::*;
    varargs_rust::va_entry!(CONSOLE_SetStatusTextX => |a| -> () {
        let x = a.get(0) as SHORT;
        let fmt = a.get(1) as LPCSTR;
        CONSOLE_SetStatusTextXV(x, fmt, a.va(2) as va_list);
    });
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetStatusTextV(fmt: LPCSTR, args: va_list) {
    CONSOLE_SetStatusTextXV(0, fmt, args);
}

extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn CONSOLE_SetStatusText(fmt: LPCSTR, ...);
}

/// `CONSOLE_SetStatusText(fmt, ...)`, defined through varargs-rust's naked thunk:
/// `a.get(0)` is `fmt`, `a.va(1)` is the `va_list` of `va_start(ap, fmt)`.
mod CONSOLE_SetStatusText_export {
    use super::*;
    varargs_rust::va_entry!(CONSOLE_SetStatusText => |a| -> () {
        let fmt = a.get(0) as LPCSTR;
        CONSOLE_SetStatusTextV(fmt, a.va(1) as va_list);
    });
}

unsafe extern "C" fn CONSOLE_ClearStatusTextX(mut x: SHORT, mut Length: SHORT) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    coPos.X = x;
    coPos.Y = (yScreen as i32 - 1) as SHORT;
    FillConsoleOutputCharacterA(
        StdOutput,
        (' ' as i32) as CHAR,
        Length as DWORD,
        coPos,
        &raw mut Written,
    );
}

extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn CONSOLE_SetStatusTextAutoFitX(x: SHORT, fmt: LPCSTR, ...);
}

/// `CONSOLE_SetStatusTextAutoFitX(x, fmt, ...)`, defined through varargs-rust's naked thunk:
/// `a.get(i)` is fixed argument `i`, `a.va(2)` is the `va_list` of `va_start(ap, fmt)`.
mod CONSOLE_SetStatusTextAutoFitX_export {
    use super::*;
    varargs_rust::va_entry!(CONSOLE_SetStatusTextAutoFitX => |a| -> () {
        let x = a.get(0) as SHORT;
        let fmt = a.get(1) as LPCSTR;
        let mut Buffer: [CHAR; 128] = [0; 128];
        let mut Length: DWORD;

        vsprintf(Buffer.as_mut_ptr(), fmt, a.va(2) as va_list);

        Length = strlen(Buffer.as_ptr()) as ULONG as DWORD;

        // C: `Length + x <= 79` -- DWORD + SHORT is unsigned 32-bit arithmetic.
        if Length.wrapping_add(x as DWORD) <= 79 {
            CONSOLE_SetStatusTextX(x, Buffer.as_ptr());
        } else {
            // C: `79 - Length` (DWORD) passed for a SHORT parameter: truncated.
            CONSOLE_SetStatusTextX((79 as DWORD).wrapping_sub(Length) as SHORT, Buffer.as_ptr());
        }
    });
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetInvertedTextXY(x: SHORT, y: SHORT, Text: LPCSTR) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Length: DWORD = 0;
    let mut Written: DWORD = 0;

    coPos.X = x;
    coPos.Y = y;

    Length = strlen(Text) as ULONG as DWORD;

    FillConsoleOutputAttribute(
        StdOutput,
        (FOREGROUND_BLUE | (BACKGROUND_WHITE) as u16) as WORD,
        Length,
        coPos,
        &mut Written,
    );

    WriteConsoleOutputCharacterA(
        StdOutput,
        Text,
        Length,
        coPos,
        &mut Written,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetHighlightedTextXY(
    x: SHORT,
    y: SHORT,
    Text: LPCSTR,
) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Length: DWORD = 0;
    let mut Written: DWORD = 0;

    coPos.X = x;
    coPos.Y = y;

    Length = strlen(Text) as ULONG as DWORD;

    FillConsoleOutputAttribute(
        StdOutput,
        (FOREGROUND_WHITE | (FOREGROUND_INTENSITY) as u32 | (BACKGROUND_BLUE) as u32) as WORD,
        Length,
        coPos,
        &mut Written,
    );

    WriteConsoleOutputCharacterA(
        StdOutput,
        Text,
        Length,
        coPos,
        &mut Written,
    );
}
extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn CONSOLE_PrintTextXY(x: SHORT, y: SHORT, fmt: LPCSTR, ...);
}

/// `CONSOLE_PrintTextXY(x, y, fmt, ...)`, defined through varargs-rust's naked thunk:
/// `a.get(i)` is fixed argument `i`, `a.va(3)` is the `va_list` of `va_start(ap, fmt)`.
mod CONSOLE_PrintTextXY_export {
    use super::*;
    varargs_rust::va_entry!(CONSOLE_PrintTextXY => |a| -> () {
        let x = a.get(0) as SHORT;
        let y = a.get(1) as SHORT;
        let fmt = a.get(2) as LPCSTR;
        let mut buffer: [CHAR; 512] = [0; 512];
        let mut coPos: COORD = COORD { X: 0, Y: 0 };
        let mut Written: DWORD = 0;

        vsprintf(buffer.as_mut_ptr(), fmt, a.va(3) as va_list);

        coPos.X = x;
        coPos.Y = y;

        WriteConsoleOutputCharacterA(
            StdOutput,
            buffer.as_ptr(),
            strlen(buffer.as_ptr()) as ULONG,
            coPos,
            &mut Written,
        );
    });
}
extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn CONSOLE_PrintTextXYN(x: SHORT, y: SHORT, len: SHORT, fmt: LPCSTR, ...);
}

/// `CONSOLE_PrintTextXYN(x, y, len, fmt, ...)`, defined through varargs-rust's naked thunk:
/// `a.get(i)` is fixed argument `i`, `a.va(4)` is the `va_list` of `va_start(ap, fmt)`.
mod CONSOLE_PrintTextXYN_export {
    use super::*;
    varargs_rust::va_entry!(CONSOLE_PrintTextXYN => |a| -> () {
        let x = a.get(0) as SHORT;
        let y = a.get(1) as SHORT;
        let len = a.get(2) as SHORT;
        let fmt = a.get(3) as LPCSTR;
        let mut buffer: [CHAR; 512] = [0; 512];
        let mut coPos: COORD = COORD { X: 0, Y: 0 };
        let mut Length: SHORT;
        let mut Written: DWORD = 0;

        vsprintf(buffer.as_mut_ptr(), fmt, a.va(4) as va_list);

        coPos.X = x;
        coPos.Y = y;

        Length = strlen(buffer.as_ptr()) as SHORT;
        if Length as i32 > len as i32 - 1 {
            Length = (len as i32 - 1) as SHORT;
        }

        WriteConsoleOutputCharacterA(
            StdOutput,
            buffer.as_ptr(),
            Length as DWORD,
            coPos,
            &mut Written,
        );

        coPos.X = (coPos.X as i32 + Length as i32) as SHORT;

        if len as i32 > Length as i32 {
            FillConsoleOutputCharacterA(
                StdOutput,
                b' ' as CHAR,
                (len as i32 - Length as i32) as DWORD,
                coPos,
                &mut Written,
            );
        }
    });
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_SetStyledText(x: SHORT, y: SHORT, Flags: INT, Text: LPCSTR) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };

    coPos.X = x;
    coPos.Y = y;

    if Flags as u32 & TEXT_TYPE_STATUS != 0 {
        coPos.X = x;
        coPos.Y = yScreen - 1;
    } else {
        coPos.X = x;
        coPos.Y = y;
    }

    if Flags as u32 & TEXT_ALIGN_CENTER != 0 {
        coPos.X = (xScreen - strlen(Text) as SHORT) / 2;
    } else if Flags as u32 & TEXT_ALIGN_RIGHT != 0 {
        coPos.X = coPos.X - strlen(Text) as SHORT;

        if Flags as u32 & TEXT_PADDING_SMALL != 0 {
            coPos.X -= 1;
        } else if Flags as u32 & TEXT_PADDING_MEDIUM != 0 {
            coPos.X -= 2;
        } else if Flags as u32 & TEXT_PADDING_BIG != 0 {
            coPos.X -= 3;
        }
    } else {
        if Flags as u32 & TEXT_PADDING_SMALL != 0 {
            coPos.X += 1;
        } else if Flags as u32 & TEXT_PADDING_MEDIUM != 0 {
            coPos.X += 2;
        } else if Flags as u32 & TEXT_PADDING_BIG != 0 {
            coPos.X += 3;
        }
    }

    if Flags as u32 & TEXT_TYPE_STATUS != 0 {
        CONSOLE_SetStatusTextX(coPos.X, Text);
    } else {
        if Flags as u32 & TEXT_STYLE_HIGHLIGHT != 0 {
            CONSOLE_SetHighlightedTextXY(coPos.X, coPos.Y, Text);
        } else if Flags as u32 & TEXT_STYLE_UNDERLINE != 0 {
            CONSOLE_SetUnderlinedTextXY(coPos.X, coPos.Y, Text);
        } else {
            CONSOLE_SetTextXY(coPos.X, coPos.Y, Text);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CONSOLE_ClearStyledText(x: SHORT, y: SHORT, Flags: INT, Length: SHORT) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };

    coPos.X = x;
    coPos.Y = y;

    if Flags as u32 & TEXT_TYPE_STATUS != 0 {
        coPos.X = x;
        coPos.Y = (yScreen as i32 - 1) as SHORT;
    } else {
        coPos.X = x;
        coPos.Y = y;
    }

    if Flags as u32 & TEXT_ALIGN_CENTER != 0 {
        coPos.X = ((xScreen as i32 - Length as i32) / 2) as SHORT;
    } else if Flags as u32 & TEXT_ALIGN_RIGHT != 0 {
        coPos.X = (coPos.X as i32 - Length as i32) as SHORT;

        if Flags as u32 & TEXT_PADDING_SMALL != 0 {
            coPos.X = (coPos.X as i32 - 1) as SHORT;
        } else if Flags as u32 & TEXT_PADDING_MEDIUM != 0 {
            coPos.X = (coPos.X as i32 - 2) as SHORT;
        } else if Flags as u32 & TEXT_PADDING_BIG != 0 {
            coPos.X = (coPos.X as i32 - 3) as SHORT;
        }
    } else {
        if Flags as u32 & TEXT_PADDING_SMALL != 0 {
            coPos.X = (coPos.X as i32 + 1) as SHORT;
        } else if Flags as u32 & TEXT_PADDING_MEDIUM != 0 {
            coPos.X = (coPos.X as i32 + 2) as SHORT;
        } else if Flags as u32 & TEXT_PADDING_BIG != 0 {
            coPos.X = (coPos.X as i32 + 3) as SHORT;
        }
    }

    if Flags as u32 & TEXT_TYPE_STATUS != 0 {
        CONSOLE_ClearStatusTextX(coPos.X, Length);
    } else if Flags as u32 & TEXT_STYLE_UNDERLINE != 0 {
        CONSOLE_ClearTextXY(coPos.X, coPos.Y, Length);
        CONSOLE_ClearTextXY(
            coPos.X,
            (coPos.Y as i32 + 1) as SHORT,
            Length,
        );
    } else {
        CONSOLE_ClearTextXY(coPos.X, coPos.Y, Length);
    }
}
