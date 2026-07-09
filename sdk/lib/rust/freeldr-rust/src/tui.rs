use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::*;
use crate::ui::{UiStatusBarFgColor, UiStatusBarBgColor, UiBackdropFgColor, UiBackdropBgColor, UiBackdropFillStyle, UiTitleBoxFgColor, UiTitleBoxBgColor, UiMessageBoxFgColor, UiMessageBoxBgColor, UiMenuFgColor, UiMenuBgColor, UiTextColor, UiSelectedTextColor, UiSelectedTextBgColor, UiEditBoxTextColor, UiEditBoxBgColor, UiShowTime, UiMenuBox, UiCenterMenu, UiUseSpecialEffects, UiScreenWidth, UiScreenHeight, UiTitleBoxTitleText, UiTimeText, UiMonthNames, UiProgressBar, UiVtbl};
use ntkrnl_sys::ddk::{PCSTR, PSTR, TIMEINFO, va_list};
use core::ffi::c_void;
use crate::tuimenu::{TuiDisplayMenu, TuiDrawMenu};

// tui.c file-local data rustconv dropped.
#[no_mangle] pub static mut TextVideoBuffer: PVOID = core::ptr::null_mut();
const TAG_TUI_SCREENBUFFER: ULONG = u32::from_be_bytes(*b"SiuT") as ULONG;
const TAG_TUI_PALETTE: ULONG = u32::from_be_bytes(*b"PiuT") as ULONG;

extern "C" {
    fn va_start(ap: *mut va_list, last: ...);
    fn _vsnprintf(buf: *mut i8, size: usize, fmt: PCSTR, ap: va_list) -> i32;
    fn va_end(ap: va_list);
    // machine.h: VOID (*ConsPutChar)(int Ch) — canonical i32 (the
    // machpc.rs #[no_mangle] def matches machine.h; the rustconv `i8`
    // decl was the outlier).
    fn MachConsPutChar(c: i32);
    fn strlen(s: *const i8) -> usize;
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
    fn MachVideoClearScreen(attr: u8);
    fn MachVideoHideShowTextCursor(show: BOOLEAN);
    fn MachVideoSetTextCursorPosition(x: u32, y: u32);
    fn VideoAllocateOffScreenBuffer() -> *mut core::ffi::c_void;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn VideoCopyOffScreenBufferToVRAM();
    fn UiDrawText(X: u32, Y: u32, Text: PCSTR, Attr: UCHAR);
    fn UiInitProgressBar(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, ProgressText: PCSTR);
    fn MachVideoIsPaletteFixed() -> BOOLEAN;
    fn UiGetScreenHeight() -> ULONG;
    fn VideoFadeIn(palette: PPALETTE_ENTRY, count: ULONG);
    fn VideoSavePaletteState(palette: PPALETTE_ENTRY, count: ULONG);
    fn VideoSetAllColorsToBlack(count: ULONG);
    fn MachConsGetCh() -> i8;
    fn MachConsKbHit() -> u8;
    fn MachHwIdle();
    fn UiDrawStatusText(Text: PCSTR);
    fn MachBeep();
    fn UiDrawText2(X: i32, Y: i32, MaxNumChars: i32, Text: PCSTR, Attr: i32);
    fn UiFillArea(Left: i32, Top: i32, Right: i32, Bottom: i32, FillChar: i8, Attr: i32);
    fn memmove(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}

#[no_mangle]
// C-variadic def unstable on stable rustc; callers keep their variadic
// extern decl, cdecl caller-cleans → non-variadic def is ABI-safe
// (debug.rs precedent). Emit the format string verbatim (no %-expansion;
// freeldr console text is cosmetic to the `shell ready` smoke marker).
pub unsafe extern "C" fn TuiPrintf(Format: PCSTR) -> i32 {
    let mut i: i32 = 0;
    while *Format.offset(i as isize) != 0 {
        MachConsPutChar(*Format.offset(i as isize) as i32);
        i += 1;
    }
    i
}

// freeldr.h `#define printf TuiPrintf` — rustconv expanded the macro to
// `extern { fn printf(...) }` + call sites instead of resolving to
// TuiPrintf. Real cdecl alias forwarding to TuiPrintf (non-variadic,
// same verbatim-format degradation; cdecl caller-cleans → ABI-safe).
// UNIQUE: printf is a C macro, never a real symbol.
#[no_mangle]
pub unsafe extern "C" fn printf(Format: PCSTR) -> i32 {
    TuiPrintf(Format)
}

// `puts` — LLVM lowers `printf("...\n")` (constant string, trailing \n)
// to a `puts` call. Faithful: emit the string then a newline. cdecl,
// UNIQUE (no C/asm freeldr puts; it's a libc-fold artifact).
#[no_mangle]
pub unsafe extern "C" fn puts(Str: PCSTR) -> i32 {
    let mut i: i32 = 0;
    while *Str.offset(i as isize) != 0 {
        MachConsPutChar(*Str.offset(i as isize) as i32);
        i += 1;
    }
    MachConsPutChar(b'\n' as i32);
    i + 1
}

// pcvideo.c:1038 `#define TRACE_printf(Format,...) do { TRACE(Format,...);
// printf(Format,...); } while(0)`. TRACE is a no-op in the NDEBUG
// freeldr build; printf == TuiPrintf. Non-variadic cdecl def emitting
// the format verbatim (same degradation as TuiPrintf/the keystone
// TRACE). Defined here (declared `extern` variadic in pcvideo.rs) so
// the variadic call sites compile; cdecl caller-cleans → ABI-safe.
// UNIQUE (C macro, no symbol).
#[no_mangle]
pub unsafe extern "C" fn TRACE_printf(Format: PCSTR) {
    TuiPrintf(Format);
}

#[no_mangle]
pub unsafe extern "C" fn TuiTruncateStringEllipsis(
    StringText: PSTR,
    MaxChars: u32,
) {

    if strlen(StringText as *const i8) > MaxChars as usize {
        strcpy(
            StringText.offset((MaxChars - 3) as isize),
            b"...\0".as_ptr() as *const i8,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawText(
    X: u32,
    Y: u32,
    Text: PCSTR,
    Attr: u8,
) {
    TuiDrawText2(X, Y, 0, Text, Attr);
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawText2(
    X: u32,
    Y: u32,
    MaxNumChars: u32,
    Text: PCSTR,
    Attr: u8,
) {
    let ScreenMemory = TextVideoBuffer as *mut u8;
    let mut i: u32;
    let mut j: u32;

    if ScreenMemory.is_null() {
        return;
    }

    if (X >= UiScreenWidth) || (Y >= UiScreenHeight) {
        return;
    }

    i = X;
    j = 0;
    while *Text.offset(j as isize) != 0
        && i < UiScreenWidth
        && (if MaxNumChars > 0 { j < MaxNumChars } else { 1u8 != 0 })
    {
        *ScreenMemory.offset((((Y * 2) * UiScreenWidth) + (i * 2)) as isize) =
            *Text.offset(j as isize) as u8;
        *ScreenMemory.offset((((Y * 2) * UiScreenWidth) + (i * 2) + 1) as isize) = Attr;
        i += 1;
        j += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawCenteredText(
    Left: u32,
    Top: u32,
    Right: u32,
    Bottom: u32,
    TextString: PCSTR,
    Attr: u8,
) {
    let TextLength: usize;
    let mut Index: usize;
    let mut LastIndex: usize;
    let mut LineBreakCount: u32;
    let mut BoxWidth: u32;
    let BoxHeight: u32;
    let RealLeft: u32;
    let mut RealTop: u32;
    let mut X: u32;
    let mut Y: u32;
    let mut Temp: [u8; 2] = [0; 2];

    TextLength = strlen(TextString);

    LineBreakCount = 0;
    BoxWidth = 0;
    LastIndex = 0;
    Index = 0;
    while Index < TextLength {
        if *TextString.offset(Index as isize) == ((b'\n') as i8) {
            LastIndex = Index;
            LineBreakCount += 1;
        } else {
            if (Index - LastIndex) > BoxWidth as usize {
                BoxWidth = (Index - LastIndex) as u32;
            }
        }
        Index += 1;
    }

    BoxHeight = LineBreakCount + 1;

    RealLeft = (Left + Right - BoxWidth + 1) / 2;
    RealTop = (Top + Bottom - BoxHeight + 1) / 2;

    LastIndex = 0;
    Index = 0;
    while Index < TextLength {
        if *TextString.offset(Index as isize) == ((b'\n') as i8) {
            RealTop += 1;
            LastIndex = 0;
        } else {
            X = RealLeft + LastIndex as u32;
            Y = RealTop;
            LastIndex += 1;
            Temp[0] = ((*TextString.offset(Index as isize)) as u8);
            Temp[1] = 0;
            TuiDrawText(X, Y, ((Temp.as_ptr()) as *const i8), Attr);
        }
        Index += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiInitialize() -> BOOLEAN {
    MachVideoHideShowTextCursor(0u8);
    MachVideoSetTextCursorPosition(0, 0);
    MachVideoClearScreen(ATTR(COLOR_GRAY, COLOR_BLACK));

    TextVideoBuffer = VideoAllocateOffScreenBuffer();
    if TextVideoBuffer.is_null() {
        return 0u8;
    }

    UiStatusBarFgColor = COLOR_BLACK;
    UiStatusBarBgColor = COLOR_CYAN;
    UiBackdropFgColor = COLOR_WHITE;
    UiBackdropBgColor = COLOR_BLUE;
    UiBackdropFillStyle = MEDIUM_FILL;
    UiTitleBoxFgColor = COLOR_WHITE;
    UiTitleBoxBgColor = COLOR_RED;
    UiMessageBoxFgColor = COLOR_WHITE;
    UiMessageBoxBgColor = COLOR_BLUE;
    UiMenuFgColor = COLOR_WHITE;
    UiMenuBgColor = COLOR_BLUE;
    UiTextColor = COLOR_YELLOW;
    UiSelectedTextColor = COLOR_BLACK;
    UiSelectedTextBgColor = COLOR_GRAY;
    UiEditBoxTextColor = COLOR_WHITE;
    UiEditBoxBgColor = COLOR_BLACK;

    UiShowTime = 1u8;
    UiMenuBox = 1u8;
    UiCenterMenu = 1u8;
    UiUseSpecialEffects = 0u8;

    RtlStringCbCopyA(
        UiTitleBoxTitleText.as_mut_ptr(),
        core::mem::size_of_val(&UiTitleBoxTitleText),
        ((b"Boot Menu\0".as_ptr()) as *const i8),
    );

    RtlStringCbCopyA(
        UiTimeText.as_mut_ptr(),
        core::mem::size_of_val(&UiTimeText),
        ((b"[Time Remaining: %d]\0".as_ptr()) as *const i8),
    );

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn TuiUnInitialize() {
    if TextVideoBuffer.is_null() {
        return;
    }

    if UiUseSpecialEffects != 0 {
        TuiFadeOut();
    } else {
        MachVideoSetDisplayMode(core::ptr::null_mut(), 0u8);
    }

    VideoFreeOffScreenBuffer();
    TextVideoBuffer = core::ptr::null_mut();

    MachVideoClearScreen(ATTR(COLOR_GRAY, COLOR_BLACK));
    MachVideoSetTextCursorPosition(0, 0);
    MachVideoHideShowTextCursor(1u8);
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawBackdrop(DrawHeight: ULONG) {

    TuiFillArea(
        0,
        TUI_TITLE_BOX_CHAR_HEIGHT,
        UiScreenWidth - 1,
        DrawHeight - 2,
        ((UiBackdropFillStyle) as i8),
        ATTR(UiBackdropFgColor, UiBackdropBgColor),
    );

    TuiDrawBox(
        0,
        0,
        UiScreenWidth - 1,
        TUI_TITLE_BOX_CHAR_HEIGHT - 1,
        D_VERT,
        D_HORZ,
        1,
        0,
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor),
    );

    TuiDrawText(
        2,
        1,
        VERSION,
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor),
    );

    TuiDrawText(
        2,
        2,
        BY_AUTHOR,
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor),
    );

    TuiDrawText(
        2,
        3,
        AUTHOR_EMAIL,
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor),
    );

    TuiDrawText(
        UiScreenWidth - 16,
        3,
        b"F8 for Options\0".as_ptr() as PCSTR,
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor),
    );

    TuiDrawText(
        (UiScreenWidth - strlen((UiTitleBoxTitleText).as_ptr() as *const i8) as ULONG) / 2,
        2,
        (UiTitleBoxTitleText).as_ptr() as *const i8,
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor),
    );

    TuiUpdateDateTime();
    VideoCopyOffScreenBufferToVRAM();
}

#[no_mangle]
pub unsafe extern "C" fn TuiFillArea(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, FillChar: CHAR, Attr: UCHAR) {
    let mut ScreenMemory = TextVideoBuffer as PUCHAR;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut Right = Right;
    let mut Bottom = Bottom;

    if ScreenMemory.is_null() {
        return;
    }

    if (Left >= UiScreenWidth) || (Top >= UiScreenHeight) {
        return;
    }
    if Right >= UiScreenWidth {
        Right = UiScreenWidth - 1;
    }
    if Bottom >= UiScreenHeight {
        Bottom = UiScreenHeight - 1;
    }

    i = Top;
    while i <= Bottom {
        j = Left;
        while j <= Right {
            *ScreenMemory.offset((((i * 2) * UiScreenWidth) + (j * 2)) as isize) = FillChar as UCHAR;
            *ScreenMemory.offset((((i * 2) * UiScreenWidth) + (j * 2) + 1) as isize) = Attr;
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawShadow(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG) {
    let ScreenMemory = TextVideoBuffer as *mut u8;
    let mut i: ULONG;
    let RightShadow: u8;
    let DoubleRightShadow: u8;
    let BottomShadow: u8;
    let DoubleWidth: u8;
    let mut Right = Right;
    let mut Bottom = Bottom;

    if ScreenMemory.is_null() {
        return;
    }

    RightShadow = if Right < (UiScreenWidth - 1) { 1 } else { 0 };
    DoubleRightShadow = if (Right + 1) < (UiScreenWidth - 1) { 1 } else { 0 };
    BottomShadow = if Bottom < (UiScreenHeight - 1) { 1 } else { 0 };
    DoubleWidth = if UiScreenHeight < 34 { 1 } else { 0 };

    Right = min(Right, UiScreenWidth - 1);
    Bottom = min(Bottom, UiScreenHeight - 1);

    if BottomShadow != 0 {
        i = Left + if DoubleWidth != 0 { 2 } else { 1 };
        while i <= Right {
            *ScreenMemory.offset(((((Bottom + 1) * 2) * UiScreenWidth) + (i * 2) + 1) as isize) =
                ATTR(COLOR_GRAY, COLOR_BLACK);
            i += 1;
        }
    }

    if RightShadow != 0 {
        i = Top + 1;
        while i <= Bottom {
            *ScreenMemory.offset((((i * 2) * UiScreenWidth) + ((Right + 1) * 2) + 1) as isize) =
                ATTR(COLOR_GRAY, COLOR_BLACK);
            i += 1;
        }
    }
    if DoubleWidth != 0 && DoubleRightShadow != 0 {
        i = Top + 1;
        while i <= Bottom {
            *ScreenMemory.offset((((i * 2) * UiScreenWidth) + ((Right + 2) * 2) + 1) as isize) =
                ATTR(COLOR_GRAY, COLOR_BLACK);
            i += 1;
        }
    }

    if RightShadow != 0 && BottomShadow != 0 {
        *ScreenMemory.offset(((((Bottom + 1) * 2) * UiScreenWidth) + ((Right + 1) * 2) + 1) as isize) =
            ATTR(COLOR_GRAY, COLOR_BLACK);
    }
    if DoubleWidth != 0 && DoubleRightShadow != 0 && BottomShadow != 0 {
        *ScreenMemory.offset(((((Bottom + 1) * 2) * UiScreenWidth) + ((Right + 2) * 2) + 1) as isize) =
            ATTR(COLOR_GRAY, COLOR_BLACK);
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawBoxTopLine(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    VertStyle: UCHAR,
    HorzStyle: UCHAR,
    Attr: UCHAR,
) {
    let ULCorner: UCHAR;
    let URCorner: UCHAR;

    if HorzStyle == HORZ {
        if VertStyle == VERT {
            ULCorner = UL;
            URCorner = UR;
        } else {
            ULCorner = VD_UL;
            URCorner = VD_UR;
        }
    } else {
        if VertStyle == VERT {
            ULCorner = HD_UL;
            URCorner = HD_UR;
        } else {
            ULCorner = D_UL;
            URCorner = D_UR;
        }
    }

    TuiFillArea(Left, Top, Left, Top, ULCorner as CHAR, Attr);
    TuiFillArea(Left.wrapping_add(1), Top, Right.wrapping_sub(1), Top, HorzStyle as CHAR, Attr);
    TuiFillArea(Right, Top, Right, Top, URCorner as CHAR, Attr);
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawBoxBottomLine(
    Left: ULONG,
    Bottom: ULONG,
    Right: ULONG,
    VertStyle: UCHAR,
    HorzStyle: UCHAR,
    Attr: UCHAR,
) {
    let LLCorner: UCHAR;
    let LRCorner: UCHAR;

    if HorzStyle == HORZ {
        if VertStyle == VERT {
            LLCorner = LL;
            LRCorner = LR;
        } else {
            LLCorner = VD_LL;
            LRCorner = VD_LR;
        }
    } else {
        if VertStyle == VERT {
            LLCorner = HD_LL;
            LRCorner = HD_LR;
        } else {
            LLCorner = D_LL;
            LRCorner = D_LR;
        }
    }

    TuiFillArea(Left, Bottom, Left, Bottom, LLCorner as CHAR, Attr);
    TuiFillArea(Left + 1, Bottom, Right - 1, Bottom, HorzStyle as CHAR, Attr);
    TuiFillArea(Right, Bottom, Right, Bottom, LRCorner as CHAR, Attr);
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawBox(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    VertStyle: UCHAR,
    HorzStyle: UCHAR,
    Fill: BOOLEAN,
    Shadow: BOOLEAN,
    Attr: UCHAR,
) {
    if Fill != 0 {
        TuiFillArea(Left, Top, Right, Bottom, b' ' as CHAR, Attr);
    }

    TuiDrawBoxTopLine(Left, Top, Right, VertStyle, HorzStyle, Attr);

    TuiFillArea(Left, Top.wrapping_add(1), Left, Bottom.wrapping_sub(1), VertStyle as CHAR, Attr);
    TuiFillArea(Right, Top.wrapping_add(1), Right, Bottom.wrapping_sub(1), VertStyle as CHAR, Attr);

    TuiDrawBoxBottomLine(Left, Bottom, Right, VertStyle, HorzStyle, Attr);

    if Shadow != 0 {
        TuiDrawShadow(Left, Top, Right, Bottom);
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawStatusText(StatusText: PCSTR) {

    let attr = ATTR(UiStatusBarFgColor, UiStatusBarBgColor);
    
    TuiDrawText(0, UiScreenHeight - 1, b" \0".as_ptr() as PCSTR, attr);
    TuiDrawText(1, UiScreenHeight - 1, StatusText, attr);

    let text_len = strlen(StatusText);
    let mut i = text_len + 1;
    while i < UiScreenWidth as usize {
        TuiDrawText(i as u32, UiScreenHeight - 1, b" \0".as_ptr() as PCSTR, attr);
        i += 1;
    }

    VideoCopyOffScreenBufferToVRAM();
}

#[no_mangle]
pub unsafe extern "C" fn TuiUpdateDateTime() {
    let mut TimeInfo: *mut TIMEINFO;
    let mut DayPostfix: PCSTR;
    let mut PMHour: u8 = 0u8;
    let mut Buffer: [i8; 40] = [0; 40];

    if UiShowTime == 0 {
        return;
    }

    TimeInfo = ArcGetTime();
    if (*TimeInfo).Year < 1 || 9999 < (*TimeInfo).Year ||
        (*TimeInfo).Month < 1 || 12 < (*TimeInfo).Month ||
        (*TimeInfo).Day < 1 || 31 < (*TimeInfo).Day ||
        23 < (*TimeInfo).Hour ||
        59 < (*TimeInfo).Minute ||
        59 < (*TimeInfo).Second
    {
        return;
    }

    if 1 == (*TimeInfo).Day || 21 == (*TimeInfo).Day || 31 == (*TimeInfo).Day {
        DayPostfix = b"st\0".as_ptr() as PCSTR;
    } else if 2 == (*TimeInfo).Day || 22 == (*TimeInfo).Day {
        DayPostfix = b"nd\0".as_ptr() as PCSTR;
    } else if 3 == (*TimeInfo).Day || 23 == (*TimeInfo).Day {
        DayPostfix = b"rd\0".as_ptr() as PCSTR;
    } else {
        DayPostfix = b"th\0".as_ptr() as PCSTR;
    }

    RtlStringCbPrintfA(
        Buffer.as_mut_ptr(),
        core::mem::size_of_val(&Buffer),
        b"%s %d%s %d\0".as_ptr() as PCSTR,
        UiMonthNames.0[((*TimeInfo).Month - 1) as usize],
        (*TimeInfo).Day as u32,
        DayPostfix,
        (*TimeInfo).Year as u32
    );

    TuiDrawText(
        UiScreenWidth - strlen(Buffer.as_ptr()) as u32 - 2,
        1,
        Buffer.as_ptr(),
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor)
    );

    if (*TimeInfo).Hour > 12 {
        (*TimeInfo).Hour -= 12;
        PMHour = 1u8;
    }
    if (*TimeInfo).Hour == 0 {
        (*TimeInfo).Hour = 12;
    }

    RtlStringCbPrintfA(
        Buffer.as_mut_ptr(),
        core::mem::size_of_val(&Buffer),
        b"  %d:%02d:%02d %s\0".as_ptr() as PCSTR,
        (*TimeInfo).Hour as u32,
        (*TimeInfo).Minute as u32,
        (*TimeInfo).Second as u32,
        if PMHour != 0 { b"PM\0".as_ptr() } else { b"AM\0".as_ptr() } as PCSTR
    );

    TuiDrawText(
        UiScreenWidth - strlen(Buffer.as_ptr()) as u32 - 2,
        2,
        Buffer.as_ptr(),
        ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor)
    );
}

unsafe extern "C" fn TuiSaveScreen() -> PUCHAR {
    let Buffer: PUCHAR;
    let ScreenMemory: PUCHAR = TextVideoBuffer as PUCHAR;
    let mut i: ULONG;

    Buffer = FrLdrTempAlloc(
        ((UiScreenWidth * UiScreenHeight * 2) as usize),
        TAG_TUI_SCREENBUFFER,
    ) as *mut u8;
    if Buffer.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    while i < (UiScreenWidth * UiScreenHeight * 2) {
        *Buffer.add(i as usize) = *ScreenMemory.add(i as usize);
        i += 1;
    }

    Buffer
}

#[no_mangle]
pub unsafe extern "C" fn TuiRestoreScreen(Buffer: PUCHAR) {
    let ScreenMemory = TextVideoBuffer as PUCHAR;
    let mut i: ULONG;

    if Buffer.is_null() {
        return;
    }

    i = 0;
    while i < (UiScreenWidth * UiScreenHeight * 2) {
        *ScreenMemory.offset(i as isize) = *Buffer.offset(i as isize);
        i += 1;
    }

    FrLdrTempFree(Buffer as PVOID, TAG_TUI_SCREENBUFFER);

    VideoCopyOffScreenBufferToVRAM();
}

unsafe extern "C" fn TuiDrawMsgBoxCommon(
    MessageText: PCSTR,
    MsgBoxRect: *mut SMALL_RECT,
) {

    let mut width: i32 = 8;
    let mut height: i32 = 1;
    let mut curline: i32 = 0;
    let mut k: i32;
    let mut i: usize;
    let mut j: usize;
    let x1: i32;
    let x2: i32;
    let y1: i32;
    let y2: i32;
    let mut temp: [i8; 260] = [0; 260];

    let msglen = strlen(MessageText);

    i = 0;
    while i < msglen {
        if *MessageText.add(i) == b'\n' as i8 {
            height += 1;
        }
        i += 1;
    }

    i = 0;
    j = 0;
    k = 0;
    while i < height as usize {
        while *MessageText.add(j) != b'\n' as i8 && *MessageText.add(j) != 0 {
            j += 1;
            k += 1;
        }

        if k > width {
            width = k;
        }

        k = 0;
        j += 1;
        i += 1;
    }

    width += 4;
    height += 5;

    x1 = (((UiScreenWidth - ((min(width, ((UiScreenWidth) as i32))) as u32)) / 2) as i32);
    if UiCenterMenu != 0 && height <= ((UiScreenHeight - TUI_TITLE_BOX_CHAR_HEIGHT - 1) as i32) {
        y1 = (((UiScreenHeight + TUI_TITLE_BOX_CHAR_HEIGHT - 1 - ((height) as u32)) / 2) as i32);
    } else {
        y1 = (((UiScreenHeight - ((min(height, ((UiScreenHeight) as i32))) as u32)) / 2) as i32);
    }
    x2 = x1 + width - 1;
    y2 = y1 + height - 1;

    (*MsgBoxRect).Left = x1 as i16;
    (*MsgBoxRect).Right = x2 as i16;
    (*MsgBoxRect).Top = y1 as i16;
    (*MsgBoxRect).Bottom = y2 as i16;

    TuiDrawBox(
        x1 as ULONG,
        y1 as ULONG,
        x2 as ULONG,
        y2 as ULONG,
        D_VERT,
        D_HORZ,
        1u8,
        1u8,
        ((UiMessageBoxFgColor & 0xF) | ((UiMessageBoxBgColor & 0xF) << 4)) as UCHAR,
    );

    i = 0;
    j = 0;
    while i < msglen + 1 {
        if *MessageText.add(i) == b'\n' as i8 || *MessageText.add(i) == 0 {
            temp[j] = 0;
            j = 0;
            UiDrawText(
                (x1 + 2) as u32,
                (y1 + 1 + curline) as u32,
                temp.as_ptr(),
                ((UiMessageBoxFgColor & 0xF) | ((UiMessageBoxBgColor & 0xF) << 4)) as u8,
            );
            curline += 1;
        } else {
            temp[j] = *MessageText.add(i);
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiMessageBox(
    MessageText: PCSTR,
) {

    let ScreenBuffer: PVOID;

    ScreenBuffer = ((TuiSaveScreen()) as *mut c_void);

    TuiMessageBoxCritical(MessageText);

    TuiRestoreScreen(ScreenBuffer as PUCHAR);
}

#[no_mangle]
pub unsafe extern "C" fn TuiMessageBoxCritical(
    MessageText: PCSTR,
) {

    let mut BoxRect: SMALL_RECT = core::mem::zeroed();
    let mut key: i8;

    TuiDrawMsgBoxCommon(MessageText, &mut BoxRect);

    UiDrawText((((BoxRect.Left + BoxRect.Right) / 2 - 3) as u32),
               (BoxRect.Bottom - 2) as u32,
               b"   OK   \0".as_ptr() as PCSTR,
               ATTR(0, 7));

    UiDrawStatusText(b"Press ENTER to continue\0".as_ptr() as PCSTR);

    VideoCopyOffScreenBufferToVRAM();

    loop {
        if MachConsKbHit() != 0 {
            key = MachConsGetCh();
            if key == 0 {
                key = MachConsGetCh();
            }

            if (key == 13) || (key == 32) || (key == 27) {
                break;
            }
        }

        TuiUpdateDateTime();

        VideoCopyOffScreenBufferToVRAM();

        MachHwIdle();
    }
}

unsafe extern "C" fn TuiSetProgressBarText(
    ProgressText: PCSTR,
) {
    let ProgressBarWidth: ULONG;
    let mut ProgressString: [CHAR; 256] = [0; 256];

    ProgressBarWidth = UiProgressBar.Right - UiProgressBar.Left + 1;

    RtlStringCbCopyA(ProgressString.as_mut_ptr() as *mut i8, core::mem::size_of_val(&ProgressString), ProgressText as *const i8);
    TuiTruncateStringEllipsis(ProgressString.as_mut_ptr(), ProgressBarWidth);

    TuiFillArea(UiProgressBar.Left, UiProgressBar.Top,
                UiProgressBar.Right, UiProgressBar.Bottom - 1,
                b' ' as CHAR, ATTR(UiTextColor, UiMenuBgColor));

    TuiDrawCenteredText(UiProgressBar.Left, UiProgressBar.Top,
                        UiProgressBar.Right, UiProgressBar.Bottom - 1,
                        ProgressString.as_ptr(), ATTR(UiTextColor, UiMenuBgColor));
}

unsafe extern "C" fn TuiTickProgressBar(SubPercentTimes100: ULONG) {
    let mut ProgressBarWidth: ULONG;
    let mut FillCount: ULONG;

    ProgressBarWidth = UiProgressBar.Right.wrapping_sub(UiProgressBar.Left).wrapping_add(1);

    FillCount = ProgressBarWidth.wrapping_mul(SubPercentTimes100).wrapping_div(100 * 100);

    if FillCount > 0 {
        TuiFillArea(
            UiProgressBar.Left,
            UiProgressBar.Bottom,
            UiProgressBar.Left.wrapping_add(FillCount).wrapping_sub(1),
            UiProgressBar.Bottom,
            0xDBu8 as CHAR,
            ATTR(UiTextColor, UiMenuBgColor)
        );
    }
    TuiFillArea(
        UiProgressBar.Left.wrapping_add(FillCount),
        UiProgressBar.Bottom,
        UiProgressBar.Right,
        UiProgressBar.Bottom,
        0xB2u8 as CHAR,
        ATTR(UiTextColor, UiMenuBgColor)
    );

    TuiUpdateDateTime();
    VideoCopyOffScreenBufferToVRAM();
}

unsafe extern "C" fn TuiDrawProgressBarCenter(ProgressText: PCSTR) {
    let mut Left: ULONG;
    let mut Top: ULONG;
    let mut Right: ULONG;
    let mut Bottom: ULONG;
    let Width: ULONG;
    let Height: ULONG;

    Height = 2;
    Width = 50;
    Left = (UiScreenWidth - Width) / 2;
    Top = (UiScreenHeight - Height + 4) / 2;
    Right = Left + Width - 1;
    Bottom = Top + Height - 1;

    Left -= 2;
    Right += 2;
    Top -= 1;
    Bottom += 1;

    TuiDrawProgressBar(Left, Top, Right, Bottom, ProgressText);
}

unsafe extern "C" fn TuiDrawProgressBar(
    mut Left: ULONG,
    mut Top: ULONG,
    mut Right: ULONG,
    mut Bottom: ULONG,
    ProgressText: PCSTR,
) {

    TuiDrawBox(
        Left,
        Top,
        Right,
        Bottom,
        VERT,
        HORZ,
        1u8,
        1u8,
        ATTR(UiMenuFgColor, UiMenuBgColor),
    );

    Left += 2;
    Right -= 2;
    Top += 1;
    Bottom -= 1;

    UiInitProgressBar(Left, Top, Right, Bottom, ProgressText);
}

#[no_mangle]
pub unsafe extern "C" fn TuiTextToColor(ColorText: PCSTR) -> u8 {
    struct ColorEntry {
        ColorName: PCSTR,
        ColorValue: u8,
    }
    unsafe impl Sync for ColorEntry {}
    
    static COLORS: [ColorEntry; 16] = [
        ColorEntry { ColorName: b"Black\0".as_ptr() as PCSTR, ColorValue: COLOR_BLACK },
        ColorEntry { ColorName: b"Blue\0".as_ptr() as PCSTR, ColorValue: COLOR_BLUE },
        ColorEntry { ColorName: b"Green\0".as_ptr() as PCSTR, ColorValue: COLOR_GREEN },
        ColorEntry { ColorName: b"Cyan\0".as_ptr() as PCSTR, ColorValue: COLOR_CYAN },
        ColorEntry { ColorName: b"Red\0".as_ptr() as PCSTR, ColorValue: COLOR_RED },
        ColorEntry { ColorName: b"Magenta\0".as_ptr() as PCSTR, ColorValue: COLOR_MAGENTA },
        ColorEntry { ColorName: b"Brown\0".as_ptr() as PCSTR, ColorValue: COLOR_BROWN },
        ColorEntry { ColorName: b"Gray\0".as_ptr() as PCSTR, ColorValue: COLOR_GRAY },
        ColorEntry { ColorName: b"DarkGray\0".as_ptr() as PCSTR, ColorValue: COLOR_DARKGRAY },
        ColorEntry { ColorName: b"LightBlue\0".as_ptr() as PCSTR, ColorValue: COLOR_LIGHTBLUE },
        ColorEntry { ColorName: b"LightGreen\0".as_ptr() as PCSTR, ColorValue: COLOR_LIGHTGREEN },
        ColorEntry { ColorName: b"LightCyan\0".as_ptr() as PCSTR, ColorValue: COLOR_LIGHTCYAN },
        ColorEntry { ColorName: b"LightRed\0".as_ptr() as PCSTR, ColorValue: COLOR_LIGHTRED },
        ColorEntry { ColorName: b"LightMagenta\0".as_ptr() as PCSTR, ColorValue: COLOR_LIGHTMAGENTA },
        ColorEntry { ColorName: b"Yellow\0".as_ptr() as PCSTR, ColorValue: COLOR_YELLOW },
        ColorEntry { ColorName: b"White\0".as_ptr() as PCSTR, ColorValue: COLOR_WHITE },
    ];
    
    
    if _stricmp(ColorText, b"Default\0".as_ptr() as PCSTR) == 0 {
        return MachDefaultTextColor;
    }
    
    let mut i: u32 = 0;
    while i < COLORS.len() as u32 {
        if _stricmp(ColorText, COLORS[i as usize].ColorName) == 0 {
            return COLORS[i as usize].ColorValue;
        }
        i += 1;
    }
    
    COLOR_BLACK
}

#[no_mangle]
pub unsafe extern "C" fn TuiTextToFillStyle(FillStyleText: PCSTR) -> u8 {
    #[repr(C)]
    struct FillStyleEntry {
        FillStyleName: PCSTR,
        FillStyleValue: u8,
    }
    unsafe impl Sync for FillStyleEntry {}

    static FillStyles: [FillStyleEntry; 4] = [
        FillStyleEntry { FillStyleName: b"None\0".as_ptr() as PCSTR, FillStyleValue: b' ' },
        FillStyleEntry { FillStyleName: b"Light\0".as_ptr() as PCSTR, FillStyleValue: LIGHT_FILL },
        FillStyleEntry { FillStyleName: b"Medium\0".as_ptr() as PCSTR, FillStyleValue: MEDIUM_FILL },
        FillStyleEntry { FillStyleName: b"Dark\0".as_ptr() as PCSTR, FillStyleValue: DARK_FILL },
    ];

    for i in 0..FillStyles.len() {
        if _stricmp(FillStyleText, FillStyles[i].FillStyleName) == 0 {
            return FillStyles[i].FillStyleValue;
        }
    }

    LIGHT_FILL
}

#[no_mangle]
pub unsafe extern "C" fn TuiFadeInBackdrop() {

    let mut TuiFadePalette: PPALETTE_ENTRY = core::ptr::null_mut();

    if UiUseSpecialEffects != 0 && MachVideoIsPaletteFixed() == 0 {
        TuiFadePalette = FrLdrTempAlloc(
            core::mem::size_of::<PALETTE_ENTRY>() * 64,
            TAG_TUI_PALETTE
        ) as PPALETTE_ENTRY;

        if !TuiFadePalette.is_null() {
            VideoSavePaletteState(TuiFadePalette, 64);
            VideoSetAllColorsToBlack(64);
        }
    }

    TuiDrawBackdrop(UiGetScreenHeight());

    if UiUseSpecialEffects != 0 && MachVideoIsPaletteFixed() == 0 && !TuiFadePalette.is_null() {
        VideoFadeIn(TuiFadePalette, 64);
        FrLdrTempFree(TuiFadePalette as *mut VOID, TAG_TUI_PALETTE);
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiFadeOut() {
    let mut TuiFadePalette: PPALETTE_ENTRY = core::ptr::null_mut();

    if UiUseSpecialEffects != 0 && MachVideoIsPaletteFixed() == 0 {
        TuiFadePalette = FrLdrTempAlloc(
            core::mem::size_of::<PALETTE_ENTRY>() * 64,
            TAG_TUI_PALETTE
        ) as PPALETTE_ENTRY;

        if !TuiFadePalette.is_null() {
            VideoSavePaletteState(TuiFadePalette, 64);
        }
    }

    if UiUseSpecialEffects != 0 && MachVideoIsPaletteFixed() == 0 && !TuiFadePalette.is_null() {
        VideoFadeOut(64);
    }

    MachVideoSetDisplayMode(core::ptr::null_mut(), 0u8);

    if UiUseSpecialEffects != 0 && MachVideoIsPaletteFixed() == 0 && !TuiFadePalette.is_null() {
        VideoRestorePaletteState(TuiFadePalette, 64);
        FrLdrTempFree(TuiFadePalette as *mut core::ffi::c_void, TAG_TUI_PALETTE);
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiEditBox(MessageText: PCSTR, EditTextBuffer: PCHAR, Length: ULONG) -> BOOLEAN {

    let mut key: i8;
    let mut Extended: BOOLEAN;
    let EditBoxLine: i32;
    let EditBoxStartX: ULONG;
    let EditBoxEndX: ULONG;
    let mut EditBoxCursorX: i32;
    let mut EditBoxTextLength: ULONG;
    let mut EditBoxTextPosition: ULONG;
    let mut EditBoxTextDisplayIndex: i32;
    let ReturnCode: BOOLEAN;
    let mut BoxRect: SMALL_RECT = core::mem::zeroed();
    let ScreenBuffer: PVOID;

    ScreenBuffer = ((TuiSaveScreen()) as *mut c_void);

    TuiDrawMsgBoxCommon(MessageText, &mut BoxRect);

    EditBoxTextLength = strlen(EditTextBuffer) as ULONG;
    EditBoxTextLength = min(EditBoxTextLength as i32, (Length - 1) as i32) as ULONG;
    EditBoxTextPosition = 0;
    EditBoxLine = BoxRect.Bottom as i32 - 2;
    EditBoxStartX = BoxRect.Left as ULONG + 3;
    EditBoxEndX = BoxRect.Right as ULONG - 3;

    UiFillArea(EditBoxStartX as i32, EditBoxLine, EditBoxEndX as i32, EditBoxLine, b' ' as i8, ((ATTR(UiEditBoxTextColor, UiEditBoxBgColor)) as i32));
    UiDrawText2(EditBoxStartX as i32, EditBoxLine, (EditBoxEndX - EditBoxStartX + 1) as i32, EditTextBuffer, ((ATTR(UiEditBoxTextColor, UiEditBoxBgColor)) as i32));

    EditBoxCursorX = EditBoxStartX as i32;
    MachVideoSetTextCursorPosition(EditBoxCursorX as u32, EditBoxLine as u32);
    MachVideoHideShowTextCursor(1u8);

    UiDrawStatusText(b"Press ENTER to continue, or ESC to cancel\0".as_ptr() as PCSTR);

    VideoCopyOffScreenBufferToVRAM();

    loop {
        if MachConsKbHit() != 0 {
            Extended = 0u8;
            key = MachConsGetCh();
            if key == ((KEY_EXTENDED) as i8) {
                Extended = 1u8;
                key = MachConsGetCh();
            }

            if key == ((KEY_ENTER) as i8) {
                ReturnCode = 1u8;
                break;
            } else if key == ((KEY_ESC) as i8) {
                ReturnCode = 0u8;
                break;
            } else if key == ((KEY_BACKSPACE) as i8) {
                if (EditBoxTextLength > 0) && (EditBoxTextPosition > 0) &&
                   (EditBoxTextPosition <= EditBoxTextLength) {
                    EditBoxTextPosition -= 1;
                    memmove(
                        EditTextBuffer.offset(EditBoxTextPosition as isize) as *mut core::ffi::c_void,
                        EditTextBuffer.offset((EditBoxTextPosition + 1) as isize) as *const core::ffi::c_void,
                        (EditBoxTextLength - EditBoxTextPosition) as usize
                    );
                    EditBoxTextLength -= 1;
                    *EditTextBuffer.offset(EditBoxTextLength as isize) = 0;
                } else {
                    MachBeep();
                }
            } else if Extended != 0 && key == ((KEY_DELETE) as i8) {
                if (EditBoxTextLength > 0) &&
                   (EditBoxTextPosition < EditBoxTextLength) {
                    memmove(
                        EditTextBuffer.offset(EditBoxTextPosition as isize) as *mut core::ffi::c_void,
                        EditTextBuffer.offset((EditBoxTextPosition + 1) as isize) as *const core::ffi::c_void,
                        (EditBoxTextLength - EditBoxTextPosition) as usize
                    );
                    EditBoxTextLength -= 1;
                    *EditTextBuffer.offset(EditBoxTextLength as isize) = 0;
                } else {
                    MachBeep();
                }
            } else if Extended != 0 && key == ((KEY_HOME) as i8) {
                EditBoxTextPosition = 0;
            } else if Extended != 0 && key == ((KEY_END) as i8) {
                EditBoxTextPosition = EditBoxTextLength;
            } else if Extended != 0 && key == ((KEY_RIGHT) as i8) {
                if EditBoxTextPosition < EditBoxTextLength {
                    EditBoxTextPosition += 1;
                } else {
                    MachBeep();
                }
            } else if Extended != 0 && key == ((KEY_LEFT) as i8) {
                if EditBoxTextPosition > 0 {
                    EditBoxTextPosition -= 1;
                } else {
                    MachBeep();
                }
            } else if Extended == 0 {
                if (EditBoxTextLength < Length - 1) &&
                   (EditBoxTextPosition < Length - 1) {
                    memmove(
                        EditTextBuffer.offset((EditBoxTextPosition + 1) as isize) as *mut core::ffi::c_void,
                        EditTextBuffer.offset(EditBoxTextPosition as isize) as *const core::ffi::c_void,
                        (EditBoxTextLength - EditBoxTextPosition) as usize
                    );
                    *EditTextBuffer.offset(EditBoxTextPosition as isize) = key;
                    EditBoxTextPosition += 1;
                    EditBoxTextLength += 1;
                    *EditTextBuffer.offset(EditBoxTextLength as isize) = 0;
                } else {
                    MachBeep();
                }
            } else {
                MachBeep();
            }
        }

        UiFillArea(EditBoxStartX as i32, EditBoxLine, EditBoxEndX as i32, EditBoxLine, b' ' as i8, ((ATTR(UiEditBoxTextColor, UiEditBoxBgColor)) as i32));

        if EditBoxTextPosition > (EditBoxEndX - EditBoxStartX) {
            EditBoxTextDisplayIndex = (EditBoxTextPosition - (EditBoxEndX - EditBoxStartX)) as i32;
            EditBoxCursorX = EditBoxEndX as i32;
        } else {
            EditBoxTextDisplayIndex = 0;
            EditBoxCursorX = EditBoxStartX as i32 + EditBoxTextPosition as i32;
        }
        UiDrawText2(EditBoxStartX as i32, EditBoxLine, (EditBoxEndX - EditBoxStartX + 1) as i32, EditTextBuffer.offset(EditBoxTextDisplayIndex as isize), ((ATTR(UiEditBoxTextColor, UiEditBoxBgColor)) as i32));

        MachVideoSetTextCursorPosition(EditBoxCursorX as u32, EditBoxLine as u32);

        TuiUpdateDateTime();

        VideoCopyOffScreenBufferToVRAM();

        MachHwIdle();
    }

    MachVideoHideShowTextCursor(0u8);

    TuiRestoreScreen(((ScreenBuffer) as *mut u8));

    ReturnCode
}

// tui.c:1233  const UIVTBL TuiVtbl = { Tui* ... }  (header-exported, tui.h)
#[no_mangle]
pub static TuiVtbl: UIVTBL = UIVTBL {
    Initialize: Some(TuiInitialize),
    UnInitialize: Some(TuiUnInitialize),
    DrawBackdrop: Some(TuiDrawBackdrop),
    FillArea: Some(TuiFillArea),
    DrawShadow: Some(TuiDrawShadow),
    DrawBox: Some(TuiDrawBox),
    DrawText: Some(TuiDrawText),
    DrawText2: Some(TuiDrawText2),
    DrawCenteredText: Some(TuiDrawCenteredText),
    DrawStatusText: Some(TuiDrawStatusText),
    UpdateDateTime: Some(TuiUpdateDateTime),
    MessageBox: Some(TuiMessageBox),
    MessageBoxCritical: Some(TuiMessageBoxCritical),
    DrawProgressBarCenter: Some(TuiDrawProgressBarCenter),
    DrawProgressBar: Some(TuiDrawProgressBar),
    SetProgressBarText: Some(TuiSetProgressBarText),
    TickProgressBar: Some(TuiTickProgressBar),
    EditBox: Some(TuiEditBox),
    TextToColor: Some(TuiTextToColor),
    TextToFillStyle: Some(TuiTextToFillStyle),
    FadeInBackdrop: Some(TuiFadeInBackdrop),
    FadeOut: Some(TuiFadeOut),
    DisplayMenu: Some(TuiDisplayMenu),
    DrawMenu: Some(TuiDrawMenu),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003  Brian Palmer  <brianp@sginet.com>
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// typedef struct _SMALL_RECT
// {
//     SHORT Left;
//     SHORT Top;
//     SHORT Right;
//     SHORT Bottom;
// } SMALL_RECT, *PSMALL_RECT;
//
// PVOID TextVideoBuffer = NULL;
//
// /* GENERIC TUI UTILS *********************************************************/
//
// /*
//  * TuiPrintf()
//  * Prints formatted text to the screen.
//  */
// INT
// TuiPrintf(
//     _In_ PCSTR Format, ...)
// {
//     INT i;
//     INT Length;
//     va_list ap;
//     CHAR Buffer[512];
//
//     va_start(ap, Format);
//     Length = _vsnprintf(Buffer, sizeof(Buffer), Format, ap);
//     va_end(ap);
//
//     if (Length == -1)
//         Length = (INT)sizeof(Buffer);
//
//     for (i = 0; i < Length; i++)
//     {
//         MachConsPutChar(Buffer[i]);
//     }
//
//     return Length;
// }
//
// VOID
// TuiTruncateStringEllipsis(
//     _Inout_z_ PSTR StringText,
//     _In_ ULONG MaxChars)
// {
//     /* If it's too large, just add some ellipsis past the maximum */
//     if (strlen(StringText) > MaxChars)
//         strcpy(&StringText[MaxChars - 3], "...");
// }
//
// /*
//  * DrawText()
//  * Displays a string on a single screen line.
//  * This function assumes coordinates are zero-based.
//  */
// VOID
// TuiDrawText(
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_ PCSTR Text,
//     _In_ UCHAR Attr)
// {
//     TuiDrawText2(X, Y, 0 /*(ULONG)strlen(Text)*/, Text, Attr);
// }
//
// /*
//  * DrawText2()
//  * Displays a string on a single screen line.
//  * This function assumes coordinates are zero-based.
//  * MaxNumChars is the maximum number of characters to display.
//  * If MaxNumChars == 0, then display the whole string.
//  */
// VOID
// TuiDrawText2(
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_opt_ ULONG MaxNumChars,
//     _In_reads_or_z_(MaxNumChars) PCSTR Text,
//     _In_ UCHAR Attr)
// {
//     PUCHAR ScreenMemory = (PUCHAR)TextVideoBuffer;
//     ULONG i, j;
//
//     /* Bail out if no video buffer (e.g. UEFI with no GOP) */
//     if (!ScreenMemory)
//         return;
//
//     /* Don't display anything if we are out of the screen */
//     if ((X >= UiScreenWidth) || (Y >= UiScreenHeight))
//         return;
//
//     /* Draw the text, not exceeding the width */
//     for (i = X, j = 0; Text[j] && i < UiScreenWidth && (MaxNumChars > 0 ? j < MaxNumChars : TRUE); i++, j++)
//     {
//         ScreenMemory[((Y*2)*UiScreenWidth)+(i*2)]   = (UCHAR)Text[j];
//         ScreenMemory[((Y*2)*UiScreenWidth)+(i*2)+1] = Attr;
//     }
// }
//
// VOID
// TuiDrawCenteredText(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR TextString,
//     _In_ UCHAR Attr)
// {
//     SIZE_T TextLength;
//     SIZE_T Index, LastIndex;
//     ULONG  LineBreakCount;
//     ULONG  BoxWidth, BoxHeight;
//     ULONG  RealLeft, RealTop;
//     ULONG  X, Y;
//     CHAR   Temp[2];
//
//     /* Query text length */
//     TextLength = strlen(TextString);
//
//     /* Count the new lines and the box width */
//     LineBreakCount = 0;
//     BoxWidth = 0;
//     LastIndex = 0;
//     for (Index = 0; Index < TextLength; Index++)
//     {
//         /* Scan for new lines */
//         if (TextString[Index] == '\n')
//         {
//             /* Remember the new line */
//             LastIndex = Index;
//             LineBreakCount++;
//         }
//         else
//         {
//             /* Check for new larger box width */
//             if ((Index - LastIndex) > BoxWidth)
//             {
//                 /* Update it */
//                 BoxWidth = (ULONG)(Index - LastIndex);
//             }
//         }
//     }
//
//     /* Base the box height on the number of lines */
//     BoxHeight = LineBreakCount + 1;
//
//     /*
//      * Create the centered coordinates.
//      * Here, the Left/Top/Right/Bottom rectangle is a hint, around
//      * which we center the "real" text rectangle RealLeft/RealTop.
//      */
//     RealLeft = (Left + Right - BoxWidth + 1) / 2;
//     RealTop  = (Top + Bottom - BoxHeight + 1) / 2;
//
//     /* Now go for a second scan */
//     LastIndex = 0;
//     for (Index = 0; Index < TextLength; Index++)
//     {
//         /* Look for new lines again */
//         if (TextString[Index] == '\n')
//         {
//             /* Update where the text should start */
//             RealTop++;
//             LastIndex = 0;
//         }
//         else
//         {
//             /* We've got a line of text to print, do it */
//             X = (ULONG)(RealLeft + LastIndex);
//             Y = RealTop;
//             LastIndex++;
//             Temp[0] = TextString[Index];
//             Temp[1] = 0;
//             TuiDrawText(X, Y, Temp, Attr);
//         }
//     }
// }
//
// /* FULL TUI THEME ************************************************************/
//
// #define TAG_TUI_SCREENBUFFER 'SiuT'
// #define TAG_TUI_PALETTE      'PiuT'
//
// extern UCHAR MachDefaultTextColor;
//
// BOOLEAN TuiInitialize(VOID)
// {
//     MachVideoHideShowTextCursor(FALSE);
//     MachVideoSetTextCursorPosition(0, 0);
//     MachVideoClearScreen(ATTR(COLOR_GRAY, COLOR_BLACK));
//
//     TextVideoBuffer = VideoAllocateOffScreenBuffer();
//     if (TextVideoBuffer == NULL)
//     {
//         return FALSE;
//     }
//
//     /* Load default settings with "Full" TUI Theme */
//
//     UiStatusBarFgColor    = COLOR_BLACK;
//     UiStatusBarBgColor    = COLOR_CYAN;
//     UiBackdropFgColor     = COLOR_WHITE;
//     UiBackdropBgColor     = COLOR_BLUE;
//     UiBackdropFillStyle   = MEDIUM_FILL;
//     UiTitleBoxFgColor     = COLOR_WHITE;
//     UiTitleBoxBgColor     = COLOR_RED;
//     UiMessageBoxFgColor   = COLOR_WHITE;
//     UiMessageBoxBgColor   = COLOR_BLUE;
//     UiMenuFgColor         = COLOR_WHITE;
//     UiMenuBgColor         = COLOR_BLUE;
//     UiTextColor           = COLOR_YELLOW;
//     UiSelectedTextColor   = COLOR_BLACK;
//     UiSelectedTextBgColor = COLOR_GRAY;
//     UiEditBoxTextColor    = COLOR_WHITE;
//     UiEditBoxBgColor      = COLOR_BLACK;
//
//     UiShowTime          = TRUE;
//     UiMenuBox           = TRUE;
//     UiCenterMenu        = TRUE;
//     UiUseSpecialEffects = FALSE;
//
//     // TODO: Have a boolean to show/hide title box?
//     RtlStringCbCopyA(UiTitleBoxTitleText, sizeof(UiTitleBoxTitleText),
//                      "Boot Menu");
//
//     RtlStringCbCopyA(UiTimeText, sizeof(UiTimeText),
//                      "[Time Remaining: %d]");
//
//     return TRUE;
// }
//
// VOID TuiUnInitialize(VOID)
// {
//     /* Do nothing if already uninitialized */
//     if (!TextVideoBuffer)
//         return;
//
//     if (UiUseSpecialEffects)
//     {
//         TuiFadeOut();
//     }
//     else
//     {
//         MachVideoSetDisplayMode(NULL, FALSE);
//     }
//
//     VideoFreeOffScreenBuffer();
//     TextVideoBuffer = NULL;
//
//     MachVideoClearScreen(ATTR(COLOR_GRAY, COLOR_BLACK));
//     MachVideoSetTextCursorPosition(0, 0);
//     MachVideoHideShowTextCursor(TRUE);
// }
//
// VOID TuiDrawBackdrop(ULONG DrawHeight)
// {
//     /* Fill in the background (excluding title box & status bar) */
//     TuiFillArea(0,
//                 TUI_TITLE_BOX_CHAR_HEIGHT,
//                 UiScreenWidth - 1,
//                 DrawHeight - 2,
//                 UiBackdropFillStyle,
//                 ATTR(UiBackdropFgColor, UiBackdropBgColor));
//
//     /* Draw the title box */
//     TuiDrawBox(0,
//                0,
//                UiScreenWidth - 1,
//                TUI_TITLE_BOX_CHAR_HEIGHT - 1,
//                D_VERT,
//                D_HORZ,
//                TRUE,
//                FALSE,
//                ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
//
//     /* Draw version text */
//     TuiDrawText(2,
//                 1,
//                 VERSION,
//                 ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
//
//     /* Draw copyright */
//     TuiDrawText(2,
//                 2,
//                 BY_AUTHOR,
//                 ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
//     TuiDrawText(2,
//                 3,
//                 AUTHOR_EMAIL,
//                 ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
//
//     /* Draw help text */
//     TuiDrawText(UiScreenWidth - 16, 3,
//                 /*"F1 for Help"*/ "F8 for Options",
//                 ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
//
//     /* Draw title text */
//     TuiDrawText((UiScreenWidth - (ULONG)strlen(UiTitleBoxTitleText)) / 2,
//                 2,
//                 UiTitleBoxTitleText,
//                 ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
//
//     /* Update the date & time */
//     TuiUpdateDateTime();
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// /*
//  * FillArea()
//  * This function assumes coordinates are zero-based
//  */
// VOID TuiFillArea(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom, CHAR FillChar, UCHAR Attr /* Color Attributes */)
// {
//     PUCHAR ScreenMemory = (PUCHAR)TextVideoBuffer;
//     ULONG  i, j;
//
//     /* Bail out if no video buffer (e.g. UEFI with no GOP) */
//     if (!ScreenMemory)
//         return;
//
//     /* Clip the area to the screen */
//     if ((Left >= UiScreenWidth) || (Top >= UiScreenHeight))
//     {
//         return;
//     }
//     if (Right >= UiScreenWidth)
//         Right = UiScreenWidth - 1;
//     if (Bottom >= UiScreenHeight)
//         Bottom = UiScreenHeight - 1;
//
//     /* Loop through each line and column and fill it in */
//     for (i = Top; i <= Bottom; ++i)
//     {
//         for (j = Left; j <= Right; ++j)
//         {
//             ScreenMemory[((i*2)*UiScreenWidth)+(j*2)] = (UCHAR)FillChar;
//             ScreenMemory[((i*2)*UiScreenWidth)+(j*2)+1] = Attr;
//         }
//     }
// }
//
// /*
//  * DrawShadow()
//  * This function assumes coordinates are zero-based
//  */
// VOID TuiDrawShadow(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom)
// {
//     PUCHAR ScreenMemory = (PUCHAR)TextVideoBuffer;
//     ULONG  i;
//     BOOLEAN RightShadow, DoubleRightShadow, BottomShadow, DoubleWidth;
//
//     /* Bail out if no video buffer (e.g. UEFI with no GOP) */
//     if (!ScreenMemory)
//         return;
//
//     RightShadow = (Right < (UiScreenWidth - 1));
//     DoubleRightShadow = ((Right + 1) < (UiScreenWidth - 1));
//     BottomShadow = (Bottom < (UiScreenHeight - 1));
//     DoubleWidth = (UiScreenHeight < 34);
//
//     /* Cap the right and bottom borders */
//     Right = min(Right, UiScreenWidth - 1);
//     Bottom = min(Bottom, UiScreenHeight - 1);
//
//     /* Shade the bottom of the area */
//     if (BottomShadow)
//     {
//         i = Left + (DoubleWidth ? 2 : 1);
//         for (; i <= Right; ++i)
//         {
//             ScreenMemory[(((Bottom+1)*2)*UiScreenWidth)+(i*2)+1] = ATTR(COLOR_GRAY, COLOR_BLACK);
//         }
//     }
//
//     /* Shade the right of the area */
//     if (RightShadow)
//     {
//         for (i = Top + 1; i <= Bottom; ++i)
//         {
//             ScreenMemory[((i*2)*UiScreenWidth)+((Right+1)*2)+1] = ATTR(COLOR_GRAY, COLOR_BLACK);
//         }
//     }
//     if (DoubleWidth && DoubleRightShadow)
//     {
//         for (i = Top + 1; i <= Bottom; ++i)
//         {
//             ScreenMemory[((i*2)*UiScreenWidth)+((Right+2)*2)+1] = ATTR(COLOR_GRAY, COLOR_BLACK);
//         }
//     }
//
//     /* Shade the bottom right corner */
//     if (RightShadow && BottomShadow)
//     {
//         ScreenMemory[(((Bottom+1)*2)*UiScreenWidth)+((Right+1)*2)+1] = ATTR(COLOR_GRAY, COLOR_BLACK);
//     }
//     if (DoubleWidth && DoubleRightShadow && BottomShadow)
//     {
//         ScreenMemory[(((Bottom+1)*2)*UiScreenWidth)+((Right+2)*2)+1] = ATTR(COLOR_GRAY, COLOR_BLACK);
//     }
// }
//
// /*
//  * DrawBox()
//  * This function assumes coordinates are zero-based
//  */
// VOID
// TuiDrawBoxTopLine(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ UCHAR VertStyle,
//     _In_ UCHAR HorzStyle,
//     _In_ UCHAR Attr)
// {
//     UCHAR ULCorner, URCorner;
//
//     /* Calculate the corner values */
//     if (HorzStyle == HORZ)
//     {
//         if (VertStyle == VERT)
//         {
//             ULCorner = UL;
//             URCorner = UR;
//         }
//         else // VertStyle == D_VERT
//         {
//             ULCorner = VD_UL;
//             URCorner = VD_UR;
//         }
//     }
//     else // HorzStyle == D_HORZ
//     {
//         if (VertStyle == VERT)
//         {
//             ULCorner = HD_UL;
//             URCorner = HD_UR;
//         }
//         else // VertStyle == D_VERT
//         {
//             ULCorner = D_UL;
//             URCorner = D_UR;
//         }
//     }
//
//     TuiFillArea(Left, Top, Left, Top, ULCorner, Attr);
//     TuiFillArea(Left+1, Top, Right-1, Top, HorzStyle, Attr);
//     TuiFillArea(Right, Top, Right, Top, URCorner, Attr);
// }
//
// VOID
// TuiDrawBoxBottomLine(
//     _In_ ULONG Left,
//     _In_ ULONG Bottom,
//     _In_ ULONG Right,
//     _In_ UCHAR VertStyle,
//     _In_ UCHAR HorzStyle,
//     _In_ UCHAR Attr)
// {
//     UCHAR LLCorner, LRCorner;
//
//     /* Calculate the corner values */
//     if (HorzStyle == HORZ)
//     {
//         if (VertStyle == VERT)
//         {
//             LLCorner = LL;
//             LRCorner = LR;
//         }
//         else // VertStyle == D_VERT
//         {
//             LLCorner = VD_LL;
//             LRCorner = VD_LR;
//         }
//     }
//     else // HorzStyle == D_HORZ
//     {
//         if (VertStyle == VERT)
//         {
//             LLCorner = HD_LL;
//             LRCorner = HD_LR;
//         }
//         else // VertStyle == D_VERT
//         {
//             LLCorner = D_LL;
//             LRCorner = D_LR;
//         }
//     }
//
//     TuiFillArea(Left, Bottom, Left, Bottom, LLCorner, Attr);
//     TuiFillArea(Left+1, Bottom, Right-1, Bottom, HorzStyle, Attr);
//     TuiFillArea(Right, Bottom, Right, Bottom, LRCorner, Attr);
// }
//
// VOID
// TuiDrawBox(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ UCHAR VertStyle,
//     _In_ UCHAR HorzStyle,
//     _In_ BOOLEAN Fill,
//     _In_ BOOLEAN Shadow,
//     _In_ UCHAR Attr)
// {
//     /* Fill in the box background */
//     if (Fill)
//         TuiFillArea(Left, Top, Right, Bottom, ' ', Attr);
//
//     /* Fill in the top horizontal line */
//     TuiDrawBoxTopLine(Left, Top, Right, VertStyle, HorzStyle, Attr);
//
//     /* Fill in the vertical left and right lines */
//     TuiFillArea(Left, Top+1, Left, Bottom-1, VertStyle, Attr);
//     TuiFillArea(Right, Top+1, Right, Bottom-1, VertStyle, Attr);
//
//     /* Fill in the bottom horizontal line */
//     TuiDrawBoxBottomLine(Left, Bottom, Right, VertStyle, HorzStyle, Attr);
//
//     /* Draw the shadow */
//     if (Shadow)
//         TuiDrawShadow(Left, Top, Right, Bottom);
// }
//
// VOID TuiDrawStatusText(PCSTR StatusText)
// {
//     SIZE_T    i;
//
//     TuiDrawText(0, UiScreenHeight-1, " ", ATTR(UiStatusBarFgColor, UiStatusBarBgColor));
//     TuiDrawText(1, UiScreenHeight-1, StatusText, ATTR(UiStatusBarFgColor, UiStatusBarBgColor));
//
//     for (i=strlen(StatusText)+1; i<UiScreenWidth; i++)
//     {
//         TuiDrawText((ULONG)i, UiScreenHeight-1, " ", ATTR(UiStatusBarFgColor, UiStatusBarBgColor));
//     }
//
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// VOID TuiUpdateDateTime(VOID)
// {
//     TIMEINFO* TimeInfo;
//     PCSTR   DayPostfix;
//     BOOLEAN PMHour = FALSE;
//     CHAR Buffer[40];
//
//     /* Don't draw the time if this has been disabled */
//     if (!UiShowTime) return;
//
//     TimeInfo = ArcGetTime();
//     if (TimeInfo->Year < 1 || 9999 < TimeInfo->Year ||
//         TimeInfo->Month < 1 || 12 < TimeInfo->Month ||
//         TimeInfo->Day < 1 || 31 < TimeInfo->Day ||
//         23 < TimeInfo->Hour ||
//         59 < TimeInfo->Minute ||
//         59 < TimeInfo->Second)
//     {
//         /* This happens on QEmu sometimes. We just skip updating. */
//         return;
//     }
//
//     /* Get the day postfix */
//     if (1 == TimeInfo->Day || 21 == TimeInfo->Day || 31 == TimeInfo->Day)
//         DayPostfix = "st";
//     else if (2 == TimeInfo->Day || 22 == TimeInfo->Day)
//         DayPostfix = "nd";
//     else if (3 == TimeInfo->Day || 23 == TimeInfo->Day)
//         DayPostfix = "rd";
//     else
//         DayPostfix = "th";
//
//     /* Build the date string in format: "MMMM dx yyyy" */
//     RtlStringCbPrintfA(Buffer, sizeof(Buffer),
//                        "%s %d%s %d",
//                        UiMonthNames[TimeInfo->Month - 1],
//                        TimeInfo->Day,
//                        DayPostfix,
//                        TimeInfo->Year);
//
//     /* Draw the date */
//     TuiDrawText(UiScreenWidth - (ULONG)strlen(Buffer) - 2, 1,
//                 Buffer, ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
//
//     /* Get the hour and change from 24-hour mode to 12-hour */
//     if (TimeInfo->Hour > 12)
//     {
//         TimeInfo->Hour -= 12;
//         PMHour = TRUE;
//     }
//     if (TimeInfo->Hour == 0)
//     {
//         TimeInfo->Hour = 12;
//     }
//
//     /* Build the time string in format: "h:mm:ss tt" */
//     RtlStringCbPrintfA(Buffer, sizeof(Buffer),
//                        "  %d:%02d:%02d %s",
//                        TimeInfo->Hour,
//                        TimeInfo->Minute,
//                        TimeInfo->Second,
//                        PMHour ? "PM" : "AM");
//
//     /* Draw the time */
//     TuiDrawText(UiScreenWidth - (ULONG)strlen(Buffer) - 2, 2,
//                 Buffer, ATTR(UiTitleBoxFgColor, UiTitleBoxBgColor));
// }
//
// _Ret_maybenull_
// __drv_allocatesMem(Mem)
// PUCHAR
// TuiSaveScreen(VOID)
// {
//     PUCHAR Buffer;
//     PUCHAR ScreenMemory = (PUCHAR)TextVideoBuffer;
//     ULONG i;
//
//     /* Allocate the buffer */
//     Buffer = FrLdrTempAlloc(UiScreenWidth * UiScreenHeight * 2,
//                             TAG_TUI_SCREENBUFFER);
//     if (!Buffer)
//         return NULL;
//
//     /* Loop through each cell and copy it */
//     for (i=0; i < (UiScreenWidth * UiScreenHeight * 2); i++)
//     {
//         Buffer[i] = ScreenMemory[i];
//     }
//
//     return Buffer;
// }
//
// VOID
// TuiRestoreScreen(
//     _In_opt_ __drv_freesMem(Mem) PUCHAR Buffer)
// {
//     PUCHAR ScreenMemory = (PUCHAR)TextVideoBuffer;
//     ULONG i;
//
//     if (!Buffer)
//         return;
//
//     /* Loop through each cell and copy it */
//     for (i=0; i < (UiScreenWidth * UiScreenHeight * 2); i++)
//     {
//         ScreenMemory[i] = Buffer[i];
//     }
//
//     /* Free the buffer */
//     FrLdrTempFree(Buffer, TAG_TUI_SCREENBUFFER);
//
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// static VOID
// TuiDrawMsgBoxCommon(
//     _In_ PCSTR MessageText,
//     _Out_ PSMALL_RECT MsgBoxRect)
// {
//     INT width = 8;
//     INT height = 1;
//     INT curline = 0;
//     INT k;
//     size_t i, j;
//     INT x1, x2, y1, y2;
//     CHAR temp[260];
//
//     /* Find the height */
//     for (i = 0; i < strlen(MessageText); i++)
//     {
//         if (MessageText[i] == '\n')
//             height++;
//     }
//
//     /* Find the width */
//     for (i = j = k = 0; i < height; i++)
//     {
//         while ((MessageText[j] != '\n') && (MessageText[j] != ANSI_NULL))
//         {
//             j++;
//             k++;
//         }
//
//         if (k > width)
//             width = k;
//
//         k = 0;
//         j++;
//     }
//
//     /* Account for the message box margins & bottom button/edit box */
//     width  += 4; // Border & space on left and right.
//     height += 5; // Border on top and bottom, plus 3 lines for button/edit box.
//
//     /* Calculate the centered box area, also ensuring that the top-left
//      * corner is always visible if the borders are partly off-screen */
//     x1 = (UiScreenWidth - min(width, UiScreenWidth)) / 2;
//     if (UiCenterMenu && (height <= UiScreenHeight - TUI_TITLE_BOX_CHAR_HEIGHT - 1))
//     {
//         /* Exclude the header and the status bar */
//         // y1 = (UiScreenHeight - TUI_TITLE_BOX_CHAR_HEIGHT - 1 - height) / 2
//         //      + TUI_TITLE_BOX_CHAR_HEIGHT;
//         y1 = (UiScreenHeight + TUI_TITLE_BOX_CHAR_HEIGHT - 1 - height) / 2;
//     }
//     else
//     {
//         y1 = (UiScreenHeight - min(height, UiScreenHeight)) / 2;
//     }
//     x2 = x1 + width - 1;
//     y2 = y1 + height - 1;
//
//     MsgBoxRect->Left = x1; MsgBoxRect->Right  = x2;
//     MsgBoxRect->Top  = y1; MsgBoxRect->Bottom = y2;
//
//
//     /* Draw the box */
//     TuiDrawBox(x1, y1, x2, y2, D_VERT, D_HORZ, TRUE, TRUE,
//                ATTR(UiMessageBoxFgColor, UiMessageBoxBgColor));
//
//     /* Draw the text */
//     for (i = j = 0; i < strlen(MessageText) + 1; i++)
//     {
//         if ((MessageText[i] == '\n') || (MessageText[i] == ANSI_NULL))
//         {
//             temp[j] = 0;
//             j = 0;
//             UiDrawText(x1 + 2, y1 + 1 + curline, temp,
//                        ATTR(UiMessageBoxFgColor, UiMessageBoxBgColor));
//             curline++;
//         }
//         else
//         {
//             temp[j++] = MessageText[i];
//         }
//     }
// }
//
// VOID
// TuiMessageBox(
//     _In_ PCSTR MessageText)
// {
//     PVOID ScreenBuffer;
//
//     /* Save the screen contents */
//     ScreenBuffer = TuiSaveScreen();
//
//     /* Display the message box */
//     TuiMessageBoxCritical(MessageText);
//
//     /* Restore the screen contents */
//     TuiRestoreScreen(ScreenBuffer);
// }
//
// VOID
// TuiMessageBoxCritical(
//     _In_ PCSTR MessageText)
// {
//     SMALL_RECT BoxRect;
//     CHAR key;
//
//     /* Draw the common parts of the message box */
//     TuiDrawMsgBoxCommon(MessageText, &BoxRect);
//
//     /* Draw centered OK button */
//     UiDrawText((BoxRect.Left + BoxRect.Right) / 2 - 3,
//                BoxRect.Bottom - 2,
//                "   OK   ",
//                ATTR(COLOR_BLACK, COLOR_GRAY));
//
//     /* Draw status text */
//     UiDrawStatusText("Press ENTER to continue");
//
//     VideoCopyOffScreenBufferToVRAM();
//
//     for (;;)
//     {
//         if (MachConsKbHit())
//         {
//             key = MachConsGetCh();
//             if (key == KEY_EXTENDED)
//                 key = MachConsGetCh();
//
//             if ((key == KEY_ENTER) || (key == KEY_SPACE) || (key == KEY_ESC))
//                 break;
//         }
//
//         TuiUpdateDateTime();
//
//         VideoCopyOffScreenBufferToVRAM();
//
//         MachHwIdle();
//     }
// }
//
// static VOID
// TuiSetProgressBarText(
//     _In_ PCSTR ProgressText)
// {
//     ULONG ProgressBarWidth;
//     CHAR ProgressString[256];
//
//     /* Make sure the progress bar is enabled */
//     ASSERT(UiProgressBar.Show);
//
//     /* Calculate the width of the bar proper */
//     ProgressBarWidth = UiProgressBar.Right - UiProgressBar.Left + 1;
//
//     /* First make sure the progress bar text fits */
//     RtlStringCbCopyA(ProgressString, sizeof(ProgressString), ProgressText);
//     TuiTruncateStringEllipsis(ProgressString, ProgressBarWidth);
//
//     /* Clear the text area */
//     TuiFillArea(UiProgressBar.Left, UiProgressBar.Top,
//                 UiProgressBar.Right, UiProgressBar.Bottom - 1,
//                 ' ', ATTR(UiTextColor, UiMenuBgColor));
//
//     /* Draw the "Loading..." text */
//     TuiDrawCenteredText(UiProgressBar.Left, UiProgressBar.Top,
//                         UiProgressBar.Right, UiProgressBar.Bottom - 1,
//                         ProgressString, ATTR(UiTextColor, UiMenuBgColor));
// }
//
// static VOID
// TuiTickProgressBar(
//     _In_ ULONG SubPercentTimes100)
// {
//     ULONG ProgressBarWidth;
//     ULONG FillCount;
//
//     /* Make sure the progress bar is enabled */
//     ASSERT(UiProgressBar.Show);
//
//     ASSERT(SubPercentTimes100 <= (100 * 100));
//
//     /* Calculate the width of the bar proper */
//     ProgressBarWidth = UiProgressBar.Right - UiProgressBar.Left + 1;
//
//     /* Compute fill count */
//     // FillCount = (ProgressBarWidth * Position) / Range;
//     FillCount = ProgressBarWidth * SubPercentTimes100 / (100 * 100);
//
//     /* Fill the progress bar */
//     /* Draw the percent complete -- Use the fill character */
//     if (FillCount > 0)
//     {
//         TuiFillArea(UiProgressBar.Left, UiProgressBar.Bottom,
//                     UiProgressBar.Left + FillCount - 1, UiProgressBar.Bottom,
//                     '\xDB', ATTR(UiTextColor, UiMenuBgColor));
//     }
//     /* Fill the remaining with shadow blanks */
//     TuiFillArea(UiProgressBar.Left + FillCount, UiProgressBar.Bottom,
//                 UiProgressBar.Right, UiProgressBar.Bottom,
//                 '\xB2', ATTR(UiTextColor, UiMenuBgColor));
//
//     TuiUpdateDateTime();
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// static VOID
// TuiDrawProgressBar(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR ProgressText);
//
// static VOID
// TuiDrawProgressBarCenter(
//     _In_ PCSTR ProgressText)
// {
//     ULONG Left, Top, Right, Bottom, Width, Height;
//
//     /* Build the coordinates and sizes */
//     Height = 2;
//     Width  = 50; // Allow for 50 "bars"
//     Left = (UiScreenWidth - Width) / 2;
//     Top  = (UiScreenHeight - Height + 4) / 2;
//     Right  = Left + Width - 1;
//     Bottom = Top + Height - 1;
//
//     /* Inflate to include the box margins */
//     Left -= 2;
//     Right += 2;
//     Top -= 1;
//     Bottom += 1;
//
//     /* Draw the progress bar */
//     TuiDrawProgressBar(Left, Top, Right, Bottom, ProgressText);
// }
//
// static VOID
// TuiDrawProgressBar(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR ProgressText)
// {
//     /* Draw the box */
//     TuiDrawBox(Left, Top, Right, Bottom,
//                VERT, HORZ, TRUE, TRUE,
//                ATTR(UiMenuFgColor, UiMenuBgColor));
//
//     /* Exclude the box margins */
//     Left += 2;
//     Right -= 2;
//     Top += 1;
//     Bottom -= 1;
//
//     UiInitProgressBar(Left, Top, Right, Bottom, ProgressText);
// }
//
// UCHAR TuiTextToColor(PCSTR ColorText)
// {
//     static const struct
//     {
//         PCSTR ColorName;
//         UCHAR ColorValue;
//     } Colors[] =
//     {
//         {"Black"  , COLOR_BLACK  },
//         {"Blue"   , COLOR_BLUE   },
//         {"Green"  , COLOR_GREEN  },
//         {"Cyan"   , COLOR_CYAN   },
//         {"Red"    , COLOR_RED    },
//         {"Magenta", COLOR_MAGENTA},
//         {"Brown"  , COLOR_BROWN  },
//         {"Gray"   , COLOR_GRAY   },
//         {"DarkGray"    , COLOR_DARKGRAY    },
//         {"LightBlue"   , COLOR_LIGHTBLUE   },
//         {"LightGreen"  , COLOR_LIGHTGREEN  },
//         {"LightCyan"   , COLOR_LIGHTCYAN   },
//         {"LightRed"    , COLOR_LIGHTRED    },
//         {"LightMagenta", COLOR_LIGHTMAGENTA},
//         {"Yellow"      , COLOR_YELLOW      },
//         {"White"       , COLOR_WHITE       },
//     };
//     ULONG i;
//
//     if (_stricmp(ColorText, "Default") == 0)
//         return MachDefaultTextColor;
//
//     for (i = 0; i < RTL_NUMBER_OF(Colors); ++i)
//     {
//         if (_stricmp(ColorText, Colors[i].ColorName) == 0)
//             return Colors[i].ColorValue;
//     }
//
//     return COLOR_BLACK;
// }
//
// UCHAR TuiTextToFillStyle(PCSTR FillStyleText)
// {
//     static const struct
//     {
//         PCSTR FillStyleName;
//         UCHAR FillStyleValue;
//     } FillStyles[] =
//     {
//         {"None"  , ' '},
//         {"Light" , LIGHT_FILL },
//         {"Medium", MEDIUM_FILL},
//         {"Dark"  , DARK_FILL  },
//     };
//     ULONG i;
//
//     for (i = 0; i < RTL_NUMBER_OF(FillStyles); ++i)
//     {
//         if (_stricmp(FillStyleText, FillStyles[i].FillStyleName) == 0)
//             return FillStyles[i].FillStyleValue;
//     }
//
//     return LIGHT_FILL;
// }
//
// VOID TuiFadeInBackdrop(VOID)
// {
//     PPALETTE_ENTRY TuiFadePalette = NULL;
//
//     if (UiUseSpecialEffects && ! MachVideoIsPaletteFixed())
//     {
//         TuiFadePalette = (PPALETTE_ENTRY)FrLdrTempAlloc(sizeof(PALETTE_ENTRY) * 64,
//                                                         TAG_TUI_PALETTE);
//
//         if (TuiFadePalette != NULL)
//         {
//             VideoSavePaletteState(TuiFadePalette, 64);
//             VideoSetAllColorsToBlack(64);
//         }
//     }
//
//     // Draw the backdrop and title box
//     TuiDrawBackdrop(UiGetScreenHeight());
//
//     if (UiUseSpecialEffects && ! MachVideoIsPaletteFixed() && TuiFadePalette != NULL)
//     {
//         VideoFadeIn(TuiFadePalette, 64);
//         FrLdrTempFree(TuiFadePalette, TAG_TUI_PALETTE);
//     }
// }
//
// VOID TuiFadeOut(VOID)
// {
//     PPALETTE_ENTRY TuiFadePalette = NULL;
//
//     if (UiUseSpecialEffects && ! MachVideoIsPaletteFixed())
//     {
//         TuiFadePalette = (PPALETTE_ENTRY)FrLdrTempAlloc(sizeof(PALETTE_ENTRY) * 64,
//                                                         TAG_TUI_PALETTE);
//
//         if (TuiFadePalette != NULL)
//         {
//             VideoSavePaletteState(TuiFadePalette, 64);
//         }
//     }
//
//     if (UiUseSpecialEffects && ! MachVideoIsPaletteFixed() && TuiFadePalette != NULL)
//     {
//         VideoFadeOut(64);
//     }
//
//     MachVideoSetDisplayMode(NULL, FALSE);
//
//     if (UiUseSpecialEffects && ! MachVideoIsPaletteFixed() && TuiFadePalette != NULL)
//     {
//         VideoRestorePaletteState(TuiFadePalette, 64);
//         FrLdrTempFree(TuiFadePalette, TAG_TUI_PALETTE);
//     }
//
// }
//
// BOOLEAN TuiEditBox(PCSTR MessageText, PCHAR EditTextBuffer, ULONG Length)
// {
//     CHAR    key;
//     BOOLEAN Extended;
//     INT     EditBoxLine;
//     ULONG   EditBoxStartX, EditBoxEndX;
//     INT     EditBoxCursorX;
//     ULONG   EditBoxTextLength, EditBoxTextPosition;
//     INT     EditBoxTextDisplayIndex;
//     BOOLEAN ReturnCode;
//     SMALL_RECT BoxRect;
//     PVOID ScreenBuffer;
//
//     /* Save the screen contents */
//     ScreenBuffer = TuiSaveScreen();
//
//     /* Draw the common parts of the message box */
//     TuiDrawMsgBoxCommon(MessageText, &BoxRect);
//
//     EditBoxTextLength = (ULONG)strlen(EditTextBuffer);
//     EditBoxTextLength = min(EditBoxTextLength, Length - 1);
//     EditBoxTextPosition = 0;
//     EditBoxLine = BoxRect.Bottom - 2;
//     EditBoxStartX = BoxRect.Left + 3;
//     EditBoxEndX = BoxRect.Right - 3;
//
//     // Draw the edit box background and the text
//     UiFillArea(EditBoxStartX, EditBoxLine, EditBoxEndX, EditBoxLine, ' ', ATTR(UiEditBoxTextColor, UiEditBoxBgColor));
//     UiDrawText2(EditBoxStartX, EditBoxLine, EditBoxEndX - EditBoxStartX + 1, EditTextBuffer, ATTR(UiEditBoxTextColor, UiEditBoxBgColor));
//
//     // Show the cursor
//     EditBoxCursorX = EditBoxStartX;
//     MachVideoSetTextCursorPosition(EditBoxCursorX, EditBoxLine);
//     MachVideoHideShowTextCursor(TRUE);
//
//     // Draw status text
//     UiDrawStatusText("Press ENTER to continue, or ESC to cancel");
//
//     VideoCopyOffScreenBufferToVRAM();
//
//     //
//     // Enter the text. Please keep in mind that the default input mode
//     // of the edit boxes is in insertion mode, that is, you can insert
//     // text without erasing the existing one.
//     //
//     for (;;)
//     {
//         if (MachConsKbHit())
//         {
//             Extended = FALSE;
//             key = MachConsGetCh();
//             if (key == KEY_EXTENDED)
//             {
//                 Extended = TRUE;
//                 key = MachConsGetCh();
//             }
//
//             if (key == KEY_ENTER)
//             {
//                 ReturnCode = TRUE;
//                 break;
//             }
//             else if (key == KEY_ESC)
//             {
//                 ReturnCode = FALSE;
//                 break;
//             }
//             else if (key == KEY_BACKSPACE) // Remove a character
//             {
//                 if ( (EditBoxTextLength > 0) && (EditBoxTextPosition > 0) &&
//                      (EditBoxTextPosition <= EditBoxTextLength) )
//                 {
//                     EditBoxTextPosition--;
//                     memmove(EditTextBuffer + EditBoxTextPosition,
//                             EditTextBuffer + EditBoxTextPosition + 1,
//                             EditBoxTextLength - EditBoxTextPosition);
//                     EditBoxTextLength--;
//                     EditTextBuffer[EditBoxTextLength] = 0;
//                 }
//                 else
//                 {
//                     MachBeep();
//                 }
//             }
//             else if (Extended && key == KEY_DELETE) // Remove a character
//             {
//                 if ( (EditBoxTextLength > 0) &&
//                      (EditBoxTextPosition < EditBoxTextLength) )
//                 {
//                     memmove(EditTextBuffer + EditBoxTextPosition,
//                             EditTextBuffer + EditBoxTextPosition + 1,
//                             EditBoxTextLength - EditBoxTextPosition);
//                     EditBoxTextLength--;
//                     EditTextBuffer[EditBoxTextLength] = 0;
//                 }
//                 else
//                 {
//                     MachBeep();
//                 }
//             }
//             else if (Extended && key == KEY_HOME) // Go to the start of the buffer
//             {
//                 EditBoxTextPosition = 0;
//             }
//             else if (Extended && key == KEY_END) // Go to the end of the buffer
//             {
//                 EditBoxTextPosition = EditBoxTextLength;
//             }
//             else if (Extended && key == KEY_RIGHT) // Go right
//             {
//                 if (EditBoxTextPosition < EditBoxTextLength)
//                     EditBoxTextPosition++;
//                 else
//                     MachBeep();
//             }
//             else if (Extended && key == KEY_LEFT) // Go left
//             {
//                 if (EditBoxTextPosition > 0)
//                     EditBoxTextPosition--;
//                 else
//                     MachBeep();
//             }
//             else if (!Extended) // Add this key to the buffer
//             {
//                 if ( (EditBoxTextLength   < Length - 1) &&
//                      (EditBoxTextPosition < Length - 1) )
//                 {
//                     memmove(EditTextBuffer + EditBoxTextPosition + 1,
//                             EditTextBuffer + EditBoxTextPosition,
//                             EditBoxTextLength - EditBoxTextPosition);
//                     EditTextBuffer[EditBoxTextPosition] = key;
//                     EditBoxTextPosition++;
//                     EditBoxTextLength++;
//                     EditTextBuffer[EditBoxTextLength] = 0;
//                 }
//                 else
//                 {
//                     MachBeep();
//                 }
//             }
//             else
//             {
//                 MachBeep();
//             }
//         }
//
//         // Draw the edit box background
//         UiFillArea(EditBoxStartX, EditBoxLine, EditBoxEndX, EditBoxLine, ' ', ATTR(UiEditBoxTextColor, UiEditBoxBgColor));
//
//         // Fill the text in
//         if (EditBoxTextPosition > (EditBoxEndX - EditBoxStartX))
//         {
//             EditBoxTextDisplayIndex = EditBoxTextPosition - (EditBoxEndX - EditBoxStartX);
//             EditBoxCursorX = EditBoxEndX;
//         }
//         else
//         {
//             EditBoxTextDisplayIndex = 0;
//             EditBoxCursorX = EditBoxStartX + EditBoxTextPosition;
//         }
//         UiDrawText2(EditBoxStartX, EditBoxLine, EditBoxEndX - EditBoxStartX + 1, &EditTextBuffer[EditBoxTextDisplayIndex], ATTR(UiEditBoxTextColor, UiEditBoxBgColor));
//
//         // Move the cursor
//         MachVideoSetTextCursorPosition(EditBoxCursorX, EditBoxLine);
//
//         TuiUpdateDateTime();
//
//         VideoCopyOffScreenBufferToVRAM();
//
//         MachHwIdle();
//     }
//
//     // Hide the cursor again
//     MachVideoHideShowTextCursor(FALSE);
//
//     /* Restore the screen contents */
//     TuiRestoreScreen(ScreenBuffer);
//
//     return ReturnCode;
// }
//
// const UIVTBL TuiVtbl =
// {
//     TuiInitialize,
//     TuiUnInitialize,
//     TuiDrawBackdrop,
//     TuiFillArea,
//     TuiDrawShadow,
//     TuiDrawBox,
//     TuiDrawText,
//     TuiDrawText2,
//     TuiDrawCenteredText,
//     TuiDrawStatusText,
//     TuiUpdateDateTime,
//     TuiMessageBox,
//     TuiMessageBoxCritical,
//     TuiDrawProgressBarCenter,
//     TuiDrawProgressBar,
//     TuiSetProgressBarText,
//     TuiTickProgressBar,
//     TuiEditBox,
//     TuiTextToColor,
//     TuiTextToFillStyle,
//     TuiFadeInBackdrop,
//     TuiFadeOut,
//     TuiDisplayMenu,
//     TuiDrawMenu,
// };
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
