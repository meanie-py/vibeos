use super::super::term::*;
use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
use crate::consrv::{ConsoleAllocHeap, ConsoleFreeHeap};
use super::coninput::ConDrvWriteConsoleInput;

extern "C" {
    pub fn C_ASSERT(x: bool);
    pub fn RTL_FIELD_SIZE(t: usize, f: usize) -> usize;
}
// CONSOLE_SCREEN_BUFFER_* are plain-cdecl (no NTAPI) in the still-C condrv
// (text.c / conoutput.c) → extern "C".
extern "C" {
    pub fn CONSOLE_SCREEN_BUFFER_Destroy(Buffer: PCONSOLE_SCREEN_BUFFER);
    pub fn CONSOLE_SCREEN_BUFFER_Initialize( Buffer: *mut PCONSOLE_SCREEN_BUFFER, Console: PCONSOLE, Type: u32, Size: usize, ) -> i32;
}
// IS_FULL_WIDTH is a C macro in the original source; the consrv Rust shim
// (consrv_rust_shim.c) provides a real __stdcall implementation → "system".
extern "system" {
    pub fn IS_FULL_WIDTH(c: u16) -> u32;
}
// TermWriteStream/TermResizeTerminal/TermDrawRegion supplied by super::super::term::*

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

// `ConsoleOutputAnsiToUnicodeChar(Console, dWChar, sChar)` — text.c:31 macro.
// `MultiByteToWideChar((Console)->OutputCodePage, 0, (sChar), 1, (dWChar), 1)`.
#[inline]
pub(crate) unsafe fn ConsoleOutputAnsiToUnicodeChar(
    Console: PCONSOLE,
    dWChar: *mut u16,
    sChar: *const i8,
) {
    let _ = MultiByteToWideChar(
        (*Console).OutputCodePage,
        0,
        sChar,
        1,
        dWChar,
        1,
    );
}

// rect.h macros — pure expressions / wrappers, no real C symbol.
#[inline]
pub(crate) unsafe fn ConioIsRectEmpty(Rect: PSMALL_RECT) -> i32 {
    if (*Rect).Left > (*Rect).Right || (*Rect).Top > (*Rect).Bottom { 1 } else { 0 }
}

#[inline]
pub(crate) unsafe fn ConioRectHeight(Rect: PSMALL_RECT) -> i16 {
    if (*Rect).Top > (*Rect).Bottom { 0 } else { (*Rect).Bottom - (*Rect).Top + 1 }
}

#[inline]
pub(crate) unsafe fn ConioRectWidth(Rect: PSMALL_RECT) -> i16 {
    if (*Rect).Left > (*Rect).Right { 0 } else { (*Rect).Right - (*Rect).Left + 1 }
}

#[inline]
pub(crate) unsafe fn ConioInitRect(Rect: *mut SMALL_RECT, Top: i16, Left: i16, Bottom: i16, Right: i16) {
    (*Rect).Top = Top;
    (*Rect).Left = Left;
    (*Rect).Bottom = Bottom;
    (*Rect).Right = Right;
}

#[inline]
pub(crate) unsafe fn ConioGetIntersection(
    Intersection: *mut SMALL_RECT, Rect1: *const SMALL_RECT, Rect2: *const SMALL_RECT,
) -> u8 {
    if ConioIsRectEmpty(Rect1 as PSMALL_RECT) != 0 || ConioIsRectEmpty(Rect2 as PSMALL_RECT) != 0
        || (*Rect1).Top > (*Rect2).Bottom || (*Rect1).Left > (*Rect2).Right
        || (*Rect1).Bottom < (*Rect2).Top || (*Rect1).Right < (*Rect2).Left {
        ConioInitRect(Intersection, 0, -1, 0, -1);
        return 0;
    }
    ConioInitRect(
        Intersection,
        if (*Rect1).Top > (*Rect2).Top { (*Rect1).Top } else { (*Rect2).Top },
        if (*Rect1).Left > (*Rect2).Left { (*Rect1).Left } else { (*Rect2).Left },
        if (*Rect1).Bottom < (*Rect2).Bottom { (*Rect1).Bottom } else { (*Rect2).Bottom },
        if (*Rect1).Right < (*Rect2).Right { (*Rect1).Right } else { (*Rect2).Right },
    );
    1
}

#[inline]
pub(crate) unsafe fn ConioGetUnion(
    Union: *mut SMALL_RECT, Rect1: *const SMALL_RECT, Rect2: *const SMALL_RECT,
) -> u8 {
    if ConioIsRectEmpty(Rect1 as PSMALL_RECT) != 0 {
        if ConioIsRectEmpty(Rect2 as PSMALL_RECT) != 0 {
            ConioInitRect(Union, 0, -1, 0, -1);
        } else {
            *Union = *Rect2;
        }
    } else if ConioIsRectEmpty(Rect2 as PSMALL_RECT) != 0 {
        *Union = *Rect1;
    } else {
        ConioInitRect(
            Union,
            if (*Rect1).Top < (*Rect2).Top { (*Rect1).Top } else { (*Rect2).Top },
            if (*Rect1).Left < (*Rect2).Left { (*Rect1).Left } else { (*Rect2).Left },
            if (*Rect1).Bottom > (*Rect2).Bottom { (*Rect1).Bottom } else { (*Rect2).Bottom },
            if (*Rect1).Right > (*Rect2).Right { (*Rect1).Right } else { (*Rect2).Right },
        );
    }
    1
}

// condrv flip: conoutput.rs (sibling) calls this as plain-cdecl
// `_TEXTMODE_BUFFER_Initialize` (NTSTATUS, no NTAPI) — export it.
#[no_mangle]
pub unsafe extern "C" fn TEXTMODE_BUFFER_Initialize(
    Buffer: *mut PCONSOLE_SCREEN_BUFFER,
    Console: PCONSOLE,
    _ProcessHandle: *mut core::ffi::c_void,
    TextModeInfo: PTEXTMODE_BUFFER_INFO,
) -> i32 {
    let mut Status: i32 = STATUS_SUCCESS;
    let mut NewBuffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    if Console.is_null() || Buffer.is_null() || TextModeInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*TextModeInfo).ScreenBufferSize.X == 0) || ((*TextModeInfo).ScreenBufferSize.Y == 0) {
        return STATUS_INVALID_PARAMETER;
    }

    *Buffer = core::ptr::null_mut();

    Status = CONSOLE_SCREEN_BUFFER_Initialize(
        &mut NewBuffer as *mut PTEXTMODE_SCREEN_BUFFER as *mut PCONSOLE_SCREEN_BUFFER,
        Console,
        TEXTMODE_BUFFER,
        core::mem::size_of::<TEXTMODE_SCREEN_BUFFER>(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*NewBuffer).Buffer = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        ((*TextModeInfo).ScreenBufferSize.X as usize)
            * ((*TextModeInfo).ScreenBufferSize.Y as usize)
            * core::mem::size_of::<CHAR_INFO>(),
    ) as *mut CHAR_INFO;
    if (*NewBuffer).Buffer.is_null() {
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*NewBuffer).ScreenBufferSize = (*TextModeInfo).ScreenBufferSize;
    (*NewBuffer).OldScreenBufferSize = (*NewBuffer).ScreenBufferSize;

    (*NewBuffer).ViewSize.X = core::cmp::min(
        core::cmp::max((*TextModeInfo).ViewSize.X, 1),
        (*NewBuffer).ScreenBufferSize.X,
    );
    (*NewBuffer).ViewSize.Y = core::cmp::min(
        core::cmp::max((*TextModeInfo).ViewSize.Y, 1),
        (*NewBuffer).ScreenBufferSize.Y,
    );
    (*NewBuffer).OldViewSize = (*NewBuffer).ViewSize;

    (*NewBuffer).ViewOrigin.X = 0;
    (*NewBuffer).ViewOrigin.Y = 0;
    (*NewBuffer).VirtualY = 0;

    (*NewBuffer).CursorBlinkOn = 0u8;
    (*NewBuffer).ForceCursorOff = 0u8;
    (*NewBuffer).CursorInfo.bVisible = if (*TextModeInfo).IsCursorVisible != 0 && (*TextModeInfo).CursorSize != 0 { 1 } else { 0 };
    (*NewBuffer).CursorInfo.dwSize = core::cmp::min(core::cmp::max((*TextModeInfo).CursorSize, 0), 100);

    (*NewBuffer).ScreenDefaultAttrib = (*TextModeInfo).ScreenAttrib & !(COMMON_LVB_SBCSDBCS as u16);
    (*NewBuffer).PopupDefaultAttrib = (*TextModeInfo).PopupAttrib & !(COMMON_LVB_SBCSDBCS as u16);

    (*NewBuffer).CursorPosition.Y = 0;
    while (*NewBuffer).CursorPosition.Y < (*NewBuffer).ScreenBufferSize.Y {
        ClearLineBuffer(NewBuffer);
        (*NewBuffer).CursorPosition.Y += 1;
    }
    (*NewBuffer).CursorPosition.X = 0;
    (*NewBuffer).CursorPosition.Y = 0;

    (*NewBuffer).Mode = (ENABLE_PROCESSED_OUTPUT | ENABLE_WRAP_AT_EOL_OUTPUT as u16) as u16;

    *Buffer = NewBuffer as PCONSOLE_SCREEN_BUFFER;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn TEXTMODE_BUFFER_Destroy(Buffer: PCONSOLE_SCREEN_BUFFER) {
    let Buff = Buffer as PTEXTMODE_SCREEN_BUFFER;

    (*Buffer).Header.Type = SCREEN_BUFFER;

    ConsoleFreeHeap((*Buff).Buffer as *mut core::ffi::c_void);

    CONSOLE_SCREEN_BUFFER_Destroy(Buffer);
}

#[no_mangle]
pub unsafe extern "C" fn ConioCoordToPointer(Buff: PTEXTMODE_SCREEN_BUFFER, X: u32, Y: u32) -> PCHAR_INFO {
    (*Buff).Buffer.offset(
        (((Y + (*Buff).VirtualY as u32) % (*Buff).ScreenBufferSize.Y as u32) * (*Buff).ScreenBufferSize.X as u32 + X) as isize
    )
}

#[no_mangle]
pub unsafe extern "C" fn ClearLineBuffer(Buff: PTEXTMODE_SCREEN_BUFFER) {
    let mut Ptr = ConioCoordToPointer(Buff, 0 as u32, (*Buff).CursorPosition.Y as u32);
    let mut LinePos: i16 = 0;

    while LinePos < (*Buff).ScreenBufferSize.X {
        (*Ptr).Char.UnicodeChar = ' ' as u16;
        (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
        LinePos += 1;
        Ptr = Ptr.offset(1);
    }
}

pub(crate) unsafe fn ConioComputeUpdateRect(
    Buff: PTEXTMODE_SCREEN_BUFFER,
    UpdateRect: PSMALL_RECT,
    Start: PCOORD,
    Length: u32,
) {
    if (*Buff).ScreenBufferSize.X as u32 <= (*Start).X as u32 + Length {
        (*UpdateRect).Left = 0;
        (*UpdateRect).Right = (*Buff).ScreenBufferSize.X - 1;
    } else {
        (*UpdateRect).Left = (*Start).X;
        (*UpdateRect).Right = (*Start).X + Length as i16 - 1;
    }
    (*UpdateRect).Top = (*Start).Y;
    (*UpdateRect).Bottom = (*Start).Y + (((*Start).X as u32 + Length - 1) / (*Buff).ScreenBufferSize.X as u32) as i16;
    if (*Buff).ScreenBufferSize.Y <= (*UpdateRect).Bottom {
        (*UpdateRect).Bottom = (*Buff).ScreenBufferSize.Y - 1;
    }
}

pub(crate) unsafe fn ConioCopyRegion(
    ScreenBuffer: PTEXTMODE_SCREEN_BUFFER,
    SrcRegion: PSMALL_RECT,
    DstOrigin: PCOORD)
{
    let Width: u32;
    let mut Height: u32;
    let mut SY: i16;
    let mut DY: i16;
    let mut YDelta: i16;
    let mut PtrSrc: PCHAR_INFO;
    let mut PtrDst: PCHAR_INFO;

    if ConioIsRectEmpty(SrcRegion) != 0 {
        return;
    }

    if ((*SrcRegion).Left == (*DstOrigin).X) && ((*SrcRegion).Top == (*DstOrigin).Y) {
        return;
    }

    SY = (*SrcRegion).Top;
    DY = (*DstOrigin).Y;
    YDelta = 1;
    if SY < DY {
        SY = (*SrcRegion).Bottom;
        DY = (*DstOrigin).Y + ((*SrcRegion).Bottom - (*SrcRegion).Top);
        YDelta = -1;
    } else {
        YDelta = 1;
    }

    Width = ConioRectWidth(SrcRegion) as u32;
    Height = ConioRectHeight(SrcRegion) as u32;

    while Height > 0 {
        Height -= 1;
        
        PtrSrc = ConioCoordToPointer(ScreenBuffer, (*SrcRegion).Left as u32, SY as u32);
        PtrDst = ConioCoordToPointer(ScreenBuffer, (*DstOrigin).X as u32, DY as u32);

        core::ptr::copy(PtrSrc, PtrDst, (Width * core::mem::size_of::<CHAR_INFO>() as u32 / core::mem::size_of::<CHAR_INFO>() as u32) as usize);

        SY += YDelta;
        DY += YDelta;
    }
}

pub(crate) unsafe fn ConioFillRegion(
    ScreenBuffer: PTEXTMODE_SCREEN_BUFFER,
    Region: PSMALL_RECT,
    ExcludeRegion: PSMALL_RECT,
    FillChar: CHAR_INFO,
) {
    let mut X: i16;
    let mut Y: i16;
    let mut Ptr: PCHAR_INFO;

    if ConioIsRectEmpty(Region) != 0 {
        return;
    }

    let ExcludeRegion = if !ExcludeRegion.is_null() && ConioIsRectEmpty(ExcludeRegion) != 0 {
        core::ptr::null_mut()
    } else {
        ExcludeRegion
    };

    Y = (*Region).Top;
    while Y <= (*Region).Bottom {
        Ptr = ConioCoordToPointer(ScreenBuffer, (*Region).Left as u32, Y as u32);
        X = (*Region).Left;
        while X <= (*Region).Right {
            if ExcludeRegion.is_null() ||
                !(X >= (*ExcludeRegion).Left && X <= (*ExcludeRegion).Right &&
                  Y >= (*ExcludeRegion).Top  && Y <= (*ExcludeRegion).Bottom)
            {
                *Ptr = FillChar;
            }

            Ptr = Ptr.offset(1);
            X += 1;
        }
        Y += 1;
    }
}

// condrv flip: settings.c (still C) calls `_ConioResizeBuffer` (cdecl) — export.
#[no_mangle]
pub unsafe extern "C" fn ConioResizeBuffer(
    Console: PCONSOLE,
    ScreenBuffer: PTEXTMODE_SCREEN_BUFFER,
    Size: COORD,
) -> i32 {
    let Buffer: PCHAR_INFO;
    let mut Ptr: PCHAR_INFO;
    let mut Offset: usize = 0;
    let mut CurrentAttribute: u16;
    let mut CurrentY: u16;
    let OldBuffer: PCHAR_INFO;
    let mut i: u32;
    let mut diff: u32;

    if Size.X == 0 || Size.Y == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if Size.X < (*ScreenBuffer).ViewSize.X || Size.Y < (*ScreenBuffer).ViewSize.Y {
        return STATUS_INVALID_PARAMETER;
    }

    if Size.X == (*ScreenBuffer).ScreenBufferSize.X && Size.Y == (*ScreenBuffer).ScreenBufferSize.Y {
        return STATUS_SUCCESS;
    }

    if (*Console).FixedSize != 0 {
        (*ScreenBuffer).OldScreenBufferSize = Size;
        return STATUS_NOT_SUPPORTED;
    }

    Buffer = ConsoleAllocHeap(HEAP_ZERO_MEMORY, (Size.X as u32 * Size.Y as u32 * core::mem::size_of::<CHAR_INFO>() as u32) as usize) as PCHAR_INFO;
    if Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    OldBuffer = (*ScreenBuffer).Buffer;

    CurrentY = 0;
    while (CurrentY as i16) < (*ScreenBuffer).ScreenBufferSize.Y && (CurrentY as i16) < Size.Y {
        Ptr = ConioCoordToPointer(ScreenBuffer, 0 as u32, CurrentY as u32);

        if Size.X <= (*ScreenBuffer).ScreenBufferSize.X {
            core::ptr::copy_nonoverlapping(Ptr, Buffer.add(Offset), Size.X as usize);
            Offset += Size.X as usize;

            Ptr = Buffer.add(Offset - 1);
            if (*Ptr).Attributes & (COMMON_LVB_LEADING_BYTE as u16) != 0 {
                (*Ptr).Char.UnicodeChar = ' ' as u16;
                (*Ptr).Attributes &= !(COMMON_LVB_SBCSDBCS as u16);
            }
        } else {
            core::ptr::copy_nonoverlapping(Ptr, Buffer.add(Offset), (*ScreenBuffer).ScreenBufferSize.X as usize);
            Offset += (*ScreenBuffer).ScreenBufferSize.X as usize;

            CurrentAttribute = (*ConioCoordToPointer(ScreenBuffer, (*ScreenBuffer).ScreenBufferSize.X as u32 - 1, CurrentY as u32)).Attributes;
            CurrentAttribute &= !(COMMON_LVB_SBCSDBCS as u16);

            diff = Size.X as u32 - (*ScreenBuffer).ScreenBufferSize.X as u32;

            i = 0;
            while i < diff {
                Ptr = Buffer.add(Offset);
                (*Ptr).Char.UnicodeChar = ' ' as u16;
                (*Ptr).Attributes = CurrentAttribute;
                Offset += 1;
                i += 1;
            }
        }
        CurrentY += 1;
    }

    if Size.Y > (*ScreenBuffer).ScreenBufferSize.Y {
        diff = Size.X as u32 * (Size.Y as u32 - (*ScreenBuffer).ScreenBufferSize.Y as u32);

        i = 0;
        while i < diff {
            Ptr = Buffer.add(Offset);
            (*Ptr).Char.UnicodeChar = ' ' as u16;
            (*Ptr).Attributes = (*ScreenBuffer).ScreenDefaultAttrib;
            Offset += 1;
            i += 1;
        }
    }

    InterlockedExchangePointer(core::ptr::addr_of_mut!((*ScreenBuffer).Buffer) as *mut *mut core::ffi::c_void, Buffer as *mut core::ffi::c_void);
    ConsoleFreeHeap(OldBuffer as *mut core::ffi::c_void);
    (*ScreenBuffer).ScreenBufferSize = Size;
    (*ScreenBuffer).OldScreenBufferSize = Size;
    (*ScreenBuffer).VirtualY = 0;

    (*ScreenBuffer).CursorPosition.X = min((*ScreenBuffer).CursorPosition.X, Size.X - 1);
    (*ScreenBuffer).CursorPosition.Y = min((*ScreenBuffer).CursorPosition.Y, Size.Y - 1);
    (*ScreenBuffer).ViewOrigin.X = min((*ScreenBuffer).ViewOrigin.X, Size.X - (*ScreenBuffer).ViewSize.X);
    (*ScreenBuffer).ViewOrigin.Y = min((*ScreenBuffer).ViewOrigin.Y, Size.Y - (*ScreenBuffer).ViewSize.Y);

    if (*Console).InputBuffer.Mode & (ENABLE_WINDOW_INPUT as u16) != 0 {
        let mut NumEventsWritten: u32 = 0;
        let mut er: INPUT_RECORD = core::mem::zeroed();

        er.EventType = WINDOW_BUFFER_SIZE_EVENT as u16;
        er.Event.WindowBufferSizeEvent.dwSize = (*ScreenBuffer).ScreenBufferSize;

        ConDrvWriteConsoleInput(
            Console,
            core::ptr::addr_of_mut!((*Console).InputBuffer),
            1u8,
            core::ptr::addr_of_mut!(er),
            1,
            core::ptr::addr_of_mut!(NumEventsWritten),
        );
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvChangeScreenBufferAttributes(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    NewScreenAttrib: u16,
    NewPopupAttrib: u16,
) -> i32 {
    let mut X: u16;
    let mut Y: u16;
    let mut Ptr: PCHAR_INFO;

    let mut TopLeft: COORD = core::mem::zeroed();
    let NumCodesToWrite: u32;
    let OldScreenAttrib: u16;
    let OldPopupAttrib: u16;

    if Console.is_null() || Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let NewScreenAttrib = NewScreenAttrib & !(COMMON_LVB_SBCSDBCS as u16);
    let NewPopupAttrib = NewPopupAttrib & !(COMMON_LVB_SBCSDBCS as u16);

    NumCodesToWrite = ((*Buffer).ScreenBufferSize.X as u32) * ((*Buffer).ScreenBufferSize.Y as u32);
    OldScreenAttrib = (*Buffer).ScreenDefaultAttrib;
    OldPopupAttrib = (*Buffer).PopupDefaultAttrib;

    Y = 0;
    while Y < (*Buffer).ScreenBufferSize.Y as u16 {
        Ptr = ConioCoordToPointer(Buffer, 0 as u32, Y as u32);
        X = 0;
        while X < (*Buffer).ScreenBufferSize.X as u16 {
            if ((*Ptr).Attributes & 0x0F) == (OldScreenAttrib & 0x0F) {
                (*Ptr).Attributes = ((*Ptr).Attributes & 0xFFF0) | (NewScreenAttrib & 0x0F);
            }
            if ((*Ptr).Attributes & 0x0F) == (OldPopupAttrib & 0x0F) {
                (*Ptr).Attributes = ((*Ptr).Attributes & 0xFFF0) | (NewPopupAttrib & 0x0F);
            }

            if ((*Ptr).Attributes & 0xF0) == (OldScreenAttrib & 0xF0) {
                (*Ptr).Attributes = ((*Ptr).Attributes & 0xFF0F) | (NewScreenAttrib & 0xF0);
            }
            if ((*Ptr).Attributes & 0xF0) == (OldPopupAttrib & 0xF0) {
                (*Ptr).Attributes = ((*Ptr).Attributes & 0xFF0F) | (NewPopupAttrib & 0xF0);
            }

            Ptr = Ptr.offset(1);
            X += 1;
        }
        Y += 1;
    }

    (*Buffer).ScreenDefaultAttrib = NewScreenAttrib;
    (*Buffer).PopupDefaultAttrib = NewPopupAttrib;

    if Buffer as PCONSOLE_SCREEN_BUFFER == (*Console).ActiveBuffer {
        let mut UpdateRect: SMALL_RECT = core::mem::zeroed();
        ConioComputeUpdateRect(Buffer, &mut UpdateRect, &mut TopLeft, NumCodesToWrite);
        TermDrawRegion(Console as PCONSRV_CONSOLE, &mut UpdateRect);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvReadConsoleOutput(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    Unicode: u8,
    CharInfo: PCHAR_INFO,
    ReadRegion: PSMALL_RECT,
) -> i32 {
    let mut X: i16;
    let mut Y: i16;
    let mut ScreenBuffer: SMALL_RECT = core::mem::zeroed();
    let mut CurCharInfo: PCHAR_INFO;
    let mut CapturedReadRegion: SMALL_RECT;
    let mut Ptr: PCHAR_INFO;

    if Console.is_null() || Buffer.is_null() || CharInfo.is_null() || ReadRegion.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    CapturedReadRegion = *ReadRegion;

    ConioInitRect(
        &mut ScreenBuffer,
        0,
        0,
        (*Buffer).ScreenBufferSize.Y - 1,
        (*Buffer).ScreenBufferSize.X - 1,
    );
    if ConioGetIntersection(&mut CapturedReadRegion, &CapturedReadRegion, &ScreenBuffer) == 0 {
        return STATUS_SUCCESS;
    }

    CurCharInfo = CharInfo;

    Y = CapturedReadRegion.Top;
    while Y <= CapturedReadRegion.Bottom {
        Ptr = ConioCoordToPointer(Buffer, CapturedReadRegion.Left as u32, Y as u32);
        X = CapturedReadRegion.Left;
        while X <= CapturedReadRegion.Right {
            if Unicode != 0 {
                (*CurCharInfo).Char.UnicodeChar = (*Ptr).Char.UnicodeChar;
            } else {
                WideCharToMultiByte(
                    (*Console).OutputCodePage,
                    0,
                    core::ptr::addr_of!((*Ptr).Char.UnicodeChar),
                    1,
                    core::ptr::addr_of_mut!((*CurCharInfo).Char.AsciiChar),
                    1,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
            #[cfg(not(feature = "win8"))]
            {
                (*CurCharInfo).Attributes = (*Ptr).Attributes & !(COMMON_LVB_SBCSDBCS as u16);
            }
            #[cfg(feature = "win8")]
            {
                (*CurCharInfo).Attributes = (*Ptr).Attributes;
            }
            Ptr = Ptr.offset(1);
            CurCharInfo = CurCharInfo.offset(1);
            X += 1;
        }
        Y += 1;
    }

    *ReadRegion = CapturedReadRegion;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvWriteConsoleOutput(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    Unicode: u8,
    CharInfo: PCHAR_INFO,
    WriteRegion: PSMALL_RECT,
) -> i32 {

    if Console.is_null() || Buffer.is_null() || CharInfo.is_null() || WriteRegion.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let mut CapturedWriteRegion = *WriteRegion;
    let mut ScreenBuffer: SMALL_RECT = core::mem::zeroed();

    ConioInitRect(
        &mut ScreenBuffer,
        0,
        0,
        (*Buffer).ScreenBufferSize.Y - 1,
        (*Buffer).ScreenBufferSize.X - 1,
    );

    if ConioGetIntersection(&mut CapturedWriteRegion, &CapturedWriteRegion, &ScreenBuffer) == 0 {
        return STATUS_SUCCESS;
    }

    let mut CurCharInfo = CharInfo;

    let mut Y = CapturedWriteRegion.Top;
    while Y <= CapturedWriteRegion.Bottom {
        let mut Ptr = ConioCoordToPointer(Buffer, CapturedWriteRegion.Left as u32, Y as u32);
        let mut X = CapturedWriteRegion.Left;
        while X <= CapturedWriteRegion.Right {
            if Unicode != 0 {
                (*Ptr).Char.UnicodeChar = (*CurCharInfo).Char.UnicodeChar;
            } else {
                ConsoleOutputAnsiToUnicodeChar(
                    Console,
                    &mut (*Ptr).Char.UnicodeChar,
                    &(*CurCharInfo).Char.AsciiChar,
                );
            }
            (*Ptr).Attributes = (*CurCharInfo).Attributes;
            Ptr = Ptr.offset(1);
            CurCharInfo = CurCharInfo.offset(1);
            X += 1;
        }
        Y += 1;
    }

    TermDrawRegion(Console as PCONSRV_CONSOLE, &mut CapturedWriteRegion);

    *WriteRegion = CapturedWriteRegion;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvWriteConsoleOutputVDM(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    CharInfo: PCHAR_CELL,
    CharInfoSize: COORD,
    WriteRegion: PSMALL_RECT,
) -> i32 {
    let mut X: i16;
    let mut Y: i16;
    let mut ScreenBuffer: SMALL_RECT = core::mem::zeroed();
    let mut CurCharInfo: PCHAR_CELL;
    let mut CapturedWriteRegion: SMALL_RECT = core::mem::zeroed();
    let mut Ptr: PCHAR_INFO;

    if Console.is_null() || Buffer.is_null() || CharInfo.is_null() || WriteRegion.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    CapturedWriteRegion = *WriteRegion;

    ConioInitRect(
        core::ptr::addr_of_mut!(ScreenBuffer),
        0,
        0,
        (*Buffer).ScreenBufferSize.Y - 1,
        (*Buffer).ScreenBufferSize.X - 1,
    );
    if ConioGetIntersection(
        core::ptr::addr_of_mut!(CapturedWriteRegion),
        core::ptr::addr_of_mut!(CapturedWriteRegion),
        core::ptr::addr_of_mut!(ScreenBuffer),
    ) == 0
    {
        return STATUS_SUCCESS;
    }

    Y = CapturedWriteRegion.Top;
    while Y <= CapturedWriteRegion.Bottom {
        CurCharInfo = CharInfo.offset((Y as i32 * CharInfoSize.X as i32 + CapturedWriteRegion.Left as i32) as isize);

        Ptr = ConioCoordToPointer(Buffer, CapturedWriteRegion.Left as u32, Y as u32);
        X = CapturedWriteRegion.Left;
        while X <= CapturedWriteRegion.Right {
            ConsoleOutputAnsiToUnicodeChar(
                Console,
                core::ptr::addr_of_mut!((*Ptr).Char.UnicodeChar),
                core::ptr::addr_of_mut!((*CurCharInfo).Char),
            );
            (*Ptr).Attributes = (*CurCharInfo).Attributes as u16;
            Ptr = Ptr.offset(1);
            CurCharInfo = CurCharInfo.offset(1);
            X += 1;
        }
        Y += 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvWriteConsole(
    Console: PCONSOLE,
    ScreenBuffer: PTEXTMODE_SCREEN_BUFFER,
    Unicode: u8,
    StringBuffer: *mut core::ffi::c_void,
    NumCharsToWrite: u32,
    NumCharsWritten: *mut u32,
) -> i32 {
    let mut Status: i32 = STATUS_SUCCESS;
    let mut Buffer: *mut u16 = core::ptr::null_mut();
    let mut Written: u32 = 0;
    let Length: u32;

    if Console.is_null() || ScreenBuffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*Console).ConsolePaused != 0 {
        return STATUS_PENDING;
    }

    if Unicode != 0 {
        Buffer = StringBuffer as *mut u16;
    } else {
        Length = MultiByteToWideChar(
            (*Console).OutputCodePage,
            0,
            StringBuffer as *const i8,
            NumCharsToWrite as i32,
            core::ptr::null_mut(),
            0,
        ) as u32;
        Buffer = ConsoleAllocHeap(0, (Length as usize) * core::mem::size_of::<u16>()) as *mut u16;
        if !Buffer.is_null() {
            MultiByteToWideChar(
                (*Console).OutputCodePage,
                0,
                StringBuffer as *const i8,
                NumCharsToWrite as i32,
                Buffer,
                Length as i32,
            );
        } else {
            Status = STATUS_NO_MEMORY;
        }
    }

    if !Buffer.is_null() {
        if NT_SUCCESS(Status) {
            Status = TermWriteStream(
                Console as PCONSRV_CONSOLE,
                ScreenBuffer,
                Buffer,
                NumCharsToWrite,
                1,
            );
            if NT_SUCCESS(Status) {
                Written = NumCharsToWrite;
            }
        }

        if Unicode == 0 {
            ConsoleFreeHeap(Buffer as *mut core::ffi::c_void);
        }
    }

    if !NumCharsWritten.is_null() {
        *NumCharsWritten = Written;
    }

    Status
}

pub(crate) unsafe fn IntReadConsoleOutputStringChars(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    StringBuffer: *mut core::ffi::c_void,
    Unicode: u8,
    NumCodesToRead: u32,
    ReadCoord: PCOORD,
    NumCodesRead: *mut u32,
) -> i32 {

    let CodeSize: u32;
    let mut ReadBuffer: LPBYTE = StringBuffer as LPBYTE;
    let mut X: i16;
    let mut Y: i16;
    let mut XStart: i16 = (*ReadCoord).X;
    let mut nNumChars: u32 = 0;
    let mut Ptr: PCHAR_INFO;
    let bCJK: u8 = (*Console).IsCJK;
    let mut NumCodesToRead = NumCodesToRead;

    CodeSize = if Unicode != 0 {
        core::mem::size_of::<u16>() as u32
    } else {
        core::mem::size_of::<u8>() as u32
    };

    Y = (*ReadCoord).Y;
    'outer: loop {
        if Y >= (*Buffer).ScreenBufferSize.Y {
            break;
        }

        Ptr = ConioCoordToPointer(Buffer, XStart as u32, Y as u32);
        X = XStart;
        loop {
            if X >= (*Buffer).ScreenBufferSize.X {
                break;
            }

            if nNumChars >= NumCodesToRead {
                break 'outer;
            }

            if bCJK != 0 && ((*Ptr).Attributes & (COMMON_LVB_TRAILING_BYTE as u16)) != 0 {
                if NumCodesToRead > 0 {
                    NumCodesToRead -= 1;
                }
                Ptr = Ptr.offset(1);
                X += 1;
                continue;
            }

            if Unicode != 0 {
                *(ReadBuffer as *mut u16) = (*Ptr).Char.UnicodeChar;
            } else {
                WideCharToMultiByte(
                    (*Console).OutputCodePage,
                    0,
                    core::ptr::addr_of!((*Ptr).Char.UnicodeChar),
                    1,
                    ReadBuffer as *mut i8,
                    1,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            }

            Ptr = Ptr.offset(1);

            ReadBuffer = ReadBuffer.offset(CodeSize as isize);
            nNumChars += 1;

            X += 1;
        }

        // Restart at the beginning of the next line
        XStart = 0;
        Y += 1;
    }

    if !NumCodesRead.is_null() {
        *NumCodesRead = nNumChars;
    }

    STATUS_SUCCESS
}

pub(crate) unsafe fn IntReadConsoleOutputStringAttributes(
    _Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    StringBuffer: PWORD,
    NumCodesToRead: u32,
    ReadCoord: PCOORD,
    NumCodesRead: *mut u32,
) -> i32 {
    let mut XStart = (*ReadCoord).X;
    let mut nNumChars: u32 = 0;
    let mut StringBufferPtr = StringBuffer;

    let mut Y = (*ReadCoord).Y;
    'outer: while Y < (*Buffer).ScreenBufferSize.Y {
        let mut Ptr = ConioCoordToPointer(Buffer, XStart as u32, Y as u32);
        let mut X = XStart;
        while X < (*Buffer).ScreenBufferSize.X {
            if nNumChars >= NumCodesToRead {
                break 'outer;
            }

            *StringBufferPtr = (*Ptr).Attributes;
            Ptr = Ptr.offset(1);

            StringBufferPtr = StringBufferPtr.offset(1);
            nNumChars += 1;

            X += 1;
        }
        XStart = 0;
        Y += 1;
    }

    if !NumCodesRead.is_null() {
        *NumCodesRead = nNumChars;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvReadConsoleOutputString(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    CodeType: u32,
    StringBuffer: *mut core::ffi::c_void,
    NumCodesToRead: u32,
    ReadCoord: PCOORD,
    NumCodesRead: *mut u32,
) -> i32 {

    if Console.is_null() || Buffer.is_null() || ReadCoord.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !NumCodesRead.is_null() {
        *NumCodesRead = 0;
    }

    if StringBuffer.is_null() || NumCodesToRead == 0 {
        return STATUS_SUCCESS;
    }

    if (*ReadCoord).X < 0 || (*ReadCoord).X >= (*Buffer).ScreenBufferSize.X ||
       (*ReadCoord).Y < 0 || (*ReadCoord).Y >= (*Buffer).ScreenBufferSize.Y {
        return STATUS_SUCCESS;
    }

    let mut NumCodesToRead = NumCodesToRead;
    let max_codes = ((*Buffer).ScreenBufferSize.X as u32) * ((*Buffer).ScreenBufferSize.Y as u32);
    if NumCodesToRead > max_codes {
        NumCodesToRead = max_codes;
    }

    match CodeType {
        CODE_ASCII => {
            return IntReadConsoleOutputStringChars(
                Console,
                Buffer,
                StringBuffer,
                0u8,
                NumCodesToRead,
                ReadCoord,
                NumCodesRead,
            );
        }

        CODE_UNICODE => {
            return IntReadConsoleOutputStringChars(
                Console,
                Buffer,
                StringBuffer,
                1u8,
                NumCodesToRead,
                ReadCoord,
                NumCodesRead,
            );
        }

        CODE_ATTRIBUTE => {
            return IntReadConsoleOutputStringAttributes(
                Console,
                Buffer,
                StringBuffer as PWORD,
                NumCodesToRead,
                ReadCoord,
                NumCodesRead,
            );
        }

        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }
}

pub(crate) unsafe fn IntWriteConsoleOutputStringChars(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    StringBuffer: *mut core::ffi::c_void,
    Unicode: u8,
    NumCodesToWrite: u32,
    WriteCoord: PCOORD,
    NumCodesWritten: *mut u32,
) -> i32 {

    let mut Status = STATUS_SUCCESS;
    let mut WriteBuffer: *mut u16 = core::ptr::null_mut();
    let mut tmpString: *mut u16 = core::ptr::null_mut();
    let Length: u32;
    let mut X: i16;
    let mut Y: i16;
    let mut XStart: i16 = (*WriteCoord).X;
    let mut nNumChars: u32 = 0;
    let mut Ptr: PCHAR_INFO;
    let bCJK: u8 = (*Console).IsCJK;
    let mut NumCodesToWrite = NumCodesToWrite;

    'Quit: {
        if Unicode != 0 {
            WriteBuffer = StringBuffer as *mut u16;
        } else {
            Length = MultiByteToWideChar(
                (*Console).OutputCodePage,
                0,
                StringBuffer as *const i8,
                NumCodesToWrite as i32,
                core::ptr::null_mut(),
                0,
            ) as u32;
            tmpString = ConsoleAllocHeap(0, (Length as usize) * core::mem::size_of::<u16>()) as *mut u16;
            if tmpString.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'Quit;
            }

            MultiByteToWideChar(
                (*Console).OutputCodePage,
                0,
                StringBuffer as *const i8,
                NumCodesToWrite as i32,
                tmpString,
                Length as i32,
            );

            NumCodesToWrite = Length;
            WriteBuffer = tmpString;
        }

        Y = (*WriteCoord).Y;
        while Y < (*Buffer).ScreenBufferSize.Y {
            Ptr = ConioCoordToPointer(Buffer, XStart as u32, Y as u32);
            X = XStart;
            while X < (*Buffer).ScreenBufferSize.X {
                if nNumChars >= NumCodesToWrite {
                    break 'Quit;
                }

                if bCJK != 0 && IS_FULL_WIDTH(*WriteBuffer) != 0 {
                    if X >= (*Buffer).ScreenBufferSize.X - 1 {
                        break;
                    }

                    (*Ptr).Char.UnicodeChar = *WriteBuffer;
                    (*Ptr).Attributes &= !(COMMON_LVB_SBCSDBCS as u16);
                    (*Ptr).Attributes |= COMMON_LVB_LEADING_BYTE as u16 ;
                    Ptr = Ptr.offset(1);

                    (*Ptr).Char.UnicodeChar = b' ' as u16;
                    (*Ptr).Attributes &= !(COMMON_LVB_SBCSDBCS as u16);
                    (*Ptr).Attributes |= COMMON_LVB_TRAILING_BYTE as u16 ;
                } else {
                    (*Ptr).Char.UnicodeChar = *WriteBuffer;
                }

                Ptr = Ptr.offset(1);

                WriteBuffer = WriteBuffer.offset(1);
                nNumChars += 1;
                X += 1;
            }
            // Restart at the beginning of the next line (C: `XStart = 0;`)
            XStart = 0;
            Y += 1;
        }
    }

    if !tmpString.is_null() {
        ConsoleFreeHeap(tmpString as *mut core::ffi::c_void);
    }

    if !NumCodesWritten.is_null() {
        *NumCodesWritten = nNumChars;
    }

    Status
}

pub(crate) unsafe fn IntWriteConsoleOutputStringAttribute(
    _Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    StringBuffer: PWORD,
    NumCodesToWrite: u32,
    WriteCoord: PCOORD,
    NumCodesWritten: *mut u32,
) -> i32 {
    let mut XStart: i16 = (*WriteCoord).X;
    let mut nNumChars: u32 = 0;
    let mut Ptr: PCHAR_INFO;
    let mut StringBufferPtr = StringBuffer;

    'outer: for Y in (*WriteCoord).Y..(*Buffer).ScreenBufferSize.Y {
        Ptr = ConioCoordToPointer(Buffer, XStart as u32, Y as u32);
        for _X in XStart..(*Buffer).ScreenBufferSize.X {
            if nNumChars >= NumCodesToWrite {
                break 'outer;
            }

            (*Ptr).Attributes &= COMMON_LVB_SBCSDBCS as u16 ;
            (*Ptr).Attributes |= *StringBufferPtr & !(COMMON_LVB_SBCSDBCS as u16);

            Ptr = Ptr.offset(1);

            StringBufferPtr = StringBufferPtr.offset(1);
            nNumChars += 1;
        }
        XStart = 0;
    }

    if !NumCodesWritten.is_null() {
        *NumCodesWritten = nNumChars;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvWriteConsoleOutputString(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    CodeType: u32,
    StringBuffer: *mut core::ffi::c_void,
    NumCodesToWrite: u32,
    WriteCoord: PCOORD,
    NumCodesWritten: *mut u32,
) -> i32 {

    if Console.is_null() || Buffer.is_null() || WriteCoord.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !NumCodesWritten.is_null() {
        *NumCodesWritten = 0;
    }

    if StringBuffer.is_null() || NumCodesToWrite == 0 {
        return STATUS_SUCCESS;
    }

    if (*WriteCoord).X < 0 || (*WriteCoord).X >= (*Buffer).ScreenBufferSize.X ||
       (*WriteCoord).Y < 0 || (*WriteCoord).Y >= (*Buffer).ScreenBufferSize.Y {
        return STATUS_SUCCESS;
    }

    let NumCodesToWrite = core::cmp::min(NumCodesToWrite, (*Buffer).ScreenBufferSize.X as u32 * (*Buffer).ScreenBufferSize.Y as u32);

    let Status = match CodeType {
        CODE_ASCII => {
            IntWriteConsoleOutputStringChars(
                Console,
                Buffer,
                StringBuffer,
                0u8,
                NumCodesToWrite,
                WriteCoord,
                NumCodesWritten,
            )
        }

        CODE_UNICODE => {
            IntWriteConsoleOutputStringChars(
                Console,
                Buffer,
                StringBuffer,
                1u8,
                NumCodesToWrite,
                WriteCoord,
                NumCodesWritten,
            )
        }

        CODE_ATTRIBUTE => {
            IntWriteConsoleOutputStringAttribute(
                Console,
                Buffer,
                StringBuffer as PWORD,
                NumCodesToWrite,
                WriteCoord,
                NumCodesWritten,
            )
        }

        _ => return STATUS_INVALID_PARAMETER,
    };

    if Buffer as PCONSOLE_SCREEN_BUFFER == (*Console).ActiveBuffer {
        let mut UpdateRect: SMALL_RECT = core::mem::zeroed();
        ConioComputeUpdateRect(Buffer, &mut UpdateRect, WriteCoord, NumCodesToWrite);
        TermDrawRegion(Console as PCONSRV_CONSOLE, &mut UpdateRect);
    }

    Status
}

// condrv flip: popup.c (still C) calls `_ConDrvFillConsoleOutput@28` (NTAPI).
#[no_mangle]
pub unsafe extern "system" fn ConDrvFillConsoleOutput(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    CodeType: u32,
    Code: CODE_ELEMENT,
    NumCodesToWrite: u32,
    WriteCoord: PCOORD,
    NumCodesWritten: *mut u32,
) -> i32 {
    let mut X: i16;
    let mut Y: i16;
    let mut XStart: i16;
    let mut nNumChars: u32 = 0;
    let mut Ptr: PCHAR_INFO;
    let mut bLead: u8;
    let mut bFullwidth: u8;

    if Console.is_null() || Buffer.is_null() || WriteCoord.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !NumCodesWritten.is_null() {
        *NumCodesWritten = 0;
    }

    if NumCodesToWrite == 0 {
        return STATUS_SUCCESS;
    }

    if (*WriteCoord).X < 0 || (*WriteCoord).X >= (*Buffer).ScreenBufferSize.X ||
       (*WriteCoord).Y < 0 || (*WriteCoord).Y >= (*Buffer).ScreenBufferSize.Y {
        return STATUS_SUCCESS;
    }

    let NumCodesToWrite = core::cmp::min(NumCodesToWrite, (*Buffer).ScreenBufferSize.X as u32 * (*Buffer).ScreenBufferSize.Y as u32);

    let mut Code = Code;
    if CodeType == CODE_ASCII {
        let mut tmp: CODE_ELEMENT = core::mem::zeroed();
        ConsoleOutputAnsiToUnicodeChar(Console, core::ptr::addr_of_mut!(tmp.UnicodeChar), core::ptr::addr_of!(Code.AsciiChar));
        Code = tmp;
    }

    XStart = (*WriteCoord).X;

    X = XStart;
    Y = (*WriteCoord).Y;
    bLead = 1;
    bFullwidth = 0;
    if (*Console).IsCJK != 0 {
        bFullwidth = IS_FULL_WIDTH(Code.UnicodeChar) as u8;
        if X > 0 {
            Ptr = ConioCoordToPointer(Buffer, X as u32 - 1 , Y as u32);
            if (*Ptr).Attributes & (COMMON_LVB_LEADING_BYTE as u16) != 0 {
                (*Ptr).Char.UnicodeChar = ' ' as u16;
                (*Ptr).Attributes &= !(COMMON_LVB_SBCSDBCS as u16);
            }
        }
    }

    'outer: loop {
        Y = (*WriteCoord).Y;
        while Y < (*Buffer).ScreenBufferSize.Y {
            Ptr = ConioCoordToPointer(Buffer, XStart as u32, Y as u32);
            X = XStart;
            while X < (*Buffer).ScreenBufferSize.X {
                if nNumChars >= NumCodesToWrite {
                    break 'outer;
                }

                match CodeType {
                    CODE_ASCII | CODE_UNICODE => {
                        (*Ptr).Char.UnicodeChar = Code.UnicodeChar;
                        (*Ptr).Attributes &= !(COMMON_LVB_SBCSDBCS as u16);
                        if bFullwidth != 0 {
                            if bLead != 0 {
                                (*Ptr).Attributes |= COMMON_LVB_LEADING_BYTE as u16 ;
                            } else {
                                (*Ptr).Attributes |= COMMON_LVB_TRAILING_BYTE as u16 ;
                            }
                        }
                        bLead = if bLead != 0 { 0 } else { 1 };
                    }
                    CODE_ATTRIBUTE => {
                        (*Ptr).Attributes &= COMMON_LVB_SBCSDBCS as u16 ;
                        (*Ptr).Attributes |= Code.Attribute & !(COMMON_LVB_SBCSDBCS as u16);
                    }
                    _ => {}
                }

                Ptr = Ptr.offset(1);
                nNumChars += 1;
                X += 1;
            }
            XStart = 0;
            Y += 1;
        }
        break;
    }

    if (nNumChars & 1) & bFullwidth as u32 != 0 {
        if X as i32 + Y as i32 * (*Buffer).ScreenBufferSize.X as i32 > 0 {
            Ptr = ConioCoordToPointer(Buffer, X as u32 - 1 , Y as u32);
            (*Ptr).Char.UnicodeChar = ' ' as u16;
            (*Ptr).Attributes &= !(COMMON_LVB_SBCSDBCS as u16);
        }
    }

    if !NumCodesWritten.is_null() {
        *NumCodesWritten = nNumChars;
    }

    if Buffer as PCONSOLE_SCREEN_BUFFER == (*Console).ActiveBuffer {
        let mut UpdateRect: SMALL_RECT = core::mem::zeroed();
        ConioComputeUpdateRect(Buffer, core::ptr::addr_of_mut!(UpdateRect), WriteCoord, nNumChars);
        TermDrawRegion(Console as PCONSRV_CONSOLE, core::ptr::addr_of_mut!(UpdateRect));
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvGetConsoleScreenBufferInfo(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    ScreenBufferSize: PCOORD,
    CursorPosition: PCOORD,
    ViewOrigin: PCOORD,
    ViewSize: PCOORD,
    MaximumViewSize: PCOORD,
    Attributes: PWORD,
) -> NTSTATUS {

    let mut LargestWindowSize: COORD = core::mem::zeroed();

    if Console.is_null() || Buffer.is_null() || ScreenBufferSize.is_null() ||
        CursorPosition.is_null() || ViewOrigin.is_null() || ViewSize.is_null() ||
        MaximumViewSize.is_null() || Attributes.is_null()
    {
        return STATUS_INVALID_PARAMETER;
    }

    *ScreenBufferSize = (*Buffer).ScreenBufferSize;
    *CursorPosition = (*Buffer).CursorPosition;
    *ViewOrigin = (*Buffer).ViewOrigin;
    *ViewSize = (*Buffer).ViewSize;
    *Attributes = (*Buffer).ScreenDefaultAttrib;

    TermGetLargestConsoleWindowSize(Console as PCONSRV_CONSOLE, &mut LargestWindowSize);
    LargestWindowSize.X = min(LargestWindowSize.X, (*Buffer).ScreenBufferSize.X);
    LargestWindowSize.Y = min(LargestWindowSize.Y, (*Buffer).ScreenBufferSize.Y);
    *MaximumViewSize = LargestWindowSize;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleTextAttribute(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    Attributes: WORD,
) -> NTSTATUS {
    if Console.is_null() || Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    (*Buffer).ScreenDefaultAttrib = Attributes & !(COMMON_LVB_SBCSDBCS as u16);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleScreenBufferSize(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    Size: PCOORD,
) -> i32 {
    let Status: i32;

    if Console.is_null() || Buffer.is_null() || Size.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ConioResizeBuffer(Console, Buffer, *Size);
    if NT_SUCCESS(Status) {
        TermResizeTerminal(Console as PCONSRV_CONSOLE);
    }

    Status
}

pub unsafe fn ConDrvScrollConsoleScreenBuffer(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    Unicode: u8,
    ScrollRectangle: PSMALL_RECT,
    UseClipRectangle: u8,
    ClipRectangle: PSMALL_RECT,
    DestinationOrigin: PCOORD,
    FillChar: CHAR_INFO,
) -> i32 {
    let mut CapturedDestinationOrigin: COORD = core::mem::zeroed();
    let mut ScreenBuffer: SMALL_RECT = core::mem::zeroed();
    let mut CapturedClipRectangle: SMALL_RECT = core::mem::zeroed();
    let mut SrcRegion: SMALL_RECT = core::mem::zeroed();
    let mut DstRegion: SMALL_RECT = core::mem::zeroed();
    let mut UpdateRegion: SMALL_RECT = core::mem::zeroed();

    if Console.is_null() || Buffer.is_null() || ScrollRectangle.is_null() ||
        (UseClipRectangle != 0 && ClipRectangle.is_null()) || DestinationOrigin.is_null()
    {
        return STATUS_INVALID_PARAMETER;
    }

    CapturedDestinationOrigin = *DestinationOrigin;

    ConioInitRect(core::ptr::addr_of_mut!(ScreenBuffer), 0, 0,
                  (*Buffer).ScreenBufferSize.Y - 1,
                  (*Buffer).ScreenBufferSize.X - 1);
    if ConioGetIntersection(core::ptr::addr_of_mut!(SrcRegion), ScrollRectangle, core::ptr::addr_of_mut!(ScreenBuffer)) == 0
    {
        return STATUS_SUCCESS;
    }

    if (*ScrollRectangle).Left < 0 {
        CapturedDestinationOrigin.X -= (*ScrollRectangle).Left;
    }
    if (*ScrollRectangle).Top < 0 {
        CapturedDestinationOrigin.Y -= (*ScrollRectangle).Top;
    }

    if UseClipRectangle != 0
    {
        CapturedClipRectangle = *ClipRectangle;
        if ConioGetIntersection(core::ptr::addr_of_mut!(CapturedClipRectangle), core::ptr::addr_of_mut!(CapturedClipRectangle), core::ptr::addr_of_mut!(ScreenBuffer)) == 0
        {
            return STATUS_SUCCESS;
        }
    }
    else
    {
        CapturedClipRectangle = ScreenBuffer;
    }

    if ConioGetIntersection(core::ptr::addr_of_mut!(UpdateRegion), core::ptr::addr_of_mut!(SrcRegion), core::ptr::addr_of_mut!(CapturedClipRectangle)) == 0
    {
        return STATUS_SUCCESS;
    }

    ConioInitRect(core::ptr::addr_of_mut!(DstRegion),
                  CapturedDestinationOrigin.Y,
                  CapturedDestinationOrigin.X,
                  CapturedDestinationOrigin.Y + ConioRectHeight(core::ptr::addr_of_mut!(SrcRegion)) - 1,
                  CapturedDestinationOrigin.X + ConioRectWidth(core::ptr::addr_of_mut!(SrcRegion)) - 1);

    if ConioGetIntersection(core::ptr::addr_of_mut!(DstRegion), core::ptr::addr_of_mut!(DstRegion), core::ptr::addr_of_mut!(CapturedClipRectangle)) != 0
    {
        SrcRegion.Left   += DstRegion.Left - CapturedDestinationOrigin.X;
        SrcRegion.Top    += DstRegion.Top  - CapturedDestinationOrigin.Y;
        SrcRegion.Right  = SrcRegion.Left + (DstRegion.Right - DstRegion.Left);
        SrcRegion.Bottom = SrcRegion.Top  + (DstRegion.Bottom - DstRegion.Top);

        CapturedDestinationOrigin.X = DstRegion.Left;
        CapturedDestinationOrigin.Y = DstRegion.Top;
        ConioCopyRegion(Buffer, core::ptr::addr_of_mut!(SrcRegion), core::ptr::addr_of_mut!(CapturedDestinationOrigin));
    }

    let mut FillChar = FillChar;
    if Unicode == 0
    {
        let mut tmp: u16 = 0;
        ConsoleOutputAnsiToUnicodeChar(Console, core::ptr::addr_of_mut!(tmp), core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(FillChar.Char)).AsciiChar));
        (*core::ptr::addr_of_mut!(FillChar.Char)).UnicodeChar = tmp;
    }
    FillChar.Attributes &= !(COMMON_LVB_SBCSDBCS as u16);

    ConioFillRegion(Buffer, core::ptr::addr_of_mut!(UpdateRegion), core::ptr::addr_of_mut!(DstRegion), FillChar);

    if Buffer as PCONSOLE_SCREEN_BUFFER == (*Console).ActiveBuffer
    {
        ConioGetUnion(core::ptr::addr_of_mut!(UpdateRegion), core::ptr::addr_of_mut!(UpdateRegion), core::ptr::addr_of_mut!(DstRegion));
        if ConioGetIntersection(core::ptr::addr_of_mut!(UpdateRegion), core::ptr::addr_of_mut!(UpdateRegion), core::ptr::addr_of_mut!(CapturedClipRectangle)) != 0
        {
            TermDrawRegion(Console as PCONSRV_CONSOLE, core::ptr::addr_of_mut!(UpdateRegion));
        }
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleWindowInfo(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    Absolute: u8,
    WindowRect: PSMALL_RECT,
) -> i32 {
    let mut CapturedWindowRect: SMALL_RECT;
    let mut LargestWindowSize: COORD;

    if Console.is_null() || Buffer.is_null() || WindowRect.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    CapturedWindowRect = *WindowRect;

    if Absolute == 0 {
        CapturedWindowRect.Left += (*Buffer).ViewOrigin.X;
        CapturedWindowRect.Top += (*Buffer).ViewOrigin.Y;
        CapturedWindowRect.Right += (*Buffer).ViewOrigin.X + (*Buffer).ViewSize.X - 1;
        CapturedWindowRect.Bottom += (*Buffer).ViewOrigin.Y + (*Buffer).ViewSize.Y - 1;
    }

    if ConioIsRectEmpty(core::ptr::addr_of_mut!(CapturedWindowRect)) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    LargestWindowSize = core::mem::zeroed();
    TermGetLargestConsoleWindowSize(Console as PCONSRV_CONSOLE, core::ptr::addr_of_mut!(LargestWindowSize));
    LargestWindowSize.X = min(LargestWindowSize.X, (*Buffer).ScreenBufferSize.X);
    LargestWindowSize.Y = min(LargestWindowSize.Y, (*Buffer).ScreenBufferSize.Y);
    if (ConioRectWidth(core::ptr::addr_of_mut!(CapturedWindowRect)) > LargestWindowSize.X) ||
       (ConioRectHeight(core::ptr::addr_of_mut!(CapturedWindowRect)) > LargestWindowSize.Y) {
        return STATUS_INVALID_PARAMETER;
    }

    if CapturedWindowRect.Left < 0 {
        CapturedWindowRect.Right -= CapturedWindowRect.Left;
        CapturedWindowRect.Left = 0;
    }
    if CapturedWindowRect.Top < 0 {
        CapturedWindowRect.Bottom -= CapturedWindowRect.Top;
        CapturedWindowRect.Top = 0;
    }

    CapturedWindowRect.Right = min(CapturedWindowRect.Right, (*Buffer).ScreenBufferSize.X);
    CapturedWindowRect.Bottom = min(CapturedWindowRect.Bottom, (*Buffer).ScreenBufferSize.Y);

    (*Buffer).ViewOrigin.X = CapturedWindowRect.Left;
    (*Buffer).ViewOrigin.Y = CapturedWindowRect.Top;

    (*Buffer).ViewSize.X = ConioRectWidth(core::ptr::addr_of_mut!(CapturedWindowRect));
    (*Buffer).ViewSize.Y = ConioRectHeight(core::ptr::addr_of_mut!(CapturedWindowRect));

    TermResizeTerminal(Console as PCONSRV_CONSOLE);

    STATUS_SUCCESS
}
