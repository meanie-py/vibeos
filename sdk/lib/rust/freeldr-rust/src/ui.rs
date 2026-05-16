use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{Depth, Fill, PCSTR, va_list};
use core::ffi::c_void;

extern "C" {
    fn _vsnprintf(buffer: *mut i8, count: usize, format: PCSTR, ap: va_list) -> i32;
    fn va_start(ap: *mut va_list, param: *const core::ffi::c_void);
    fn va_end(ap: *mut va_list);
    fn strcpy(dest: PCHAR, src: PCHAR) -> PCHAR;
    fn strlen(s: PCHAR) -> usize;
    fn ATTR(fg: UCHAR, bg: UCHAR) -> UCHAR;
    fn FrLdrTempAlloc(Size: usize, Tag: ULONG) -> *mut core::ffi::c_void;
    fn FrLdrTempFree(Allocation: *mut core::ffi::c_void, Tag: ULONG);
    fn IniGetNumSectionItems(SectionId: ULONG_PTR) -> ULONG;
    fn IniGetSectionSettingValueSize(SectionId: ULONG_PTR, SettingIndex: ULONG) -> ULONG;
    fn IniReadSettingByNumber( SectionId: ULONG_PTR, SettingIndex: ULONG, SettingName: PCHAR, NameSize: ULONG, SettingValue: PCHAR, ValueSize: ULONG, ) -> BOOLEAN;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn GetNextArgumentValue(argc: ULONG, argv: *mut PCHAR, last_index: *mut ULONG, name: PCSTR) -> PCSTR;
    fn IniOpenSection(SectionName: PCSTR, SectionId: *mut usize) -> u8;
    fn IniReadSettingByName(SectionId: usize, SettingName: PCSTR, Buffer: *mut u8, BufferSize: usize) -> u8;
    fn MachVideoGetDisplaySize(Width: *mut u32, Height: *mut u32, Depth: *mut u32);
    fn MachVideoSetDisplayMode(DisplayMode: PCSTR, Query: u8) -> u32;
    fn RTL_NUMBER_OF(array: *const core::ffi::c_void) -> usize;
    fn TRACE(format: PCSTR, ...);
}

#[no_mangle]
pub unsafe extern "system" fn UiInitialize(ShowUi: u8) -> u8 {
    let mut UiDisplayMode: u32;
    let mut UiMinimal: u8 = 0u8;
    let mut SectionId: usize = 0;
    let mut Depth: u32 = 0;
    let mut SettingText: [u8; 260] = [0; 260];

    if ShowUi == 0 {
        if ((*UiVtbl).Initialize.unwrap())() == 0 {
            MachVideoSetDisplayMode(core::ptr::null(), 0u8);
            return 0u8;
        }
        return 1u8;
    }

    TRACE(b"Initializing User Interface.\n\0".as_ptr() as PCSTR);
    TRACE(b"Reading UI settings from [Display] section.\n\0".as_ptr() as PCSTR);

    if IniOpenSection(b"Display\0".as_ptr() as PCSTR, &mut SectionId) == 0 {
        SectionId = 0;
    }

    SettingText[0] = 0;
    if SectionId != 0 && IniReadSettingByName(SectionId, b"DisplayMode\0".as_ptr() as PCSTR, SettingText.as_mut_ptr(), core::mem::size_of_val(&SettingText)) == 0 {
        SettingText[0] = 0;
    }
    UiDisplayMode = MachVideoSetDisplayMode(SettingText.as_ptr() as PCSTR, 1u8);
    MachVideoGetDisplaySize(&mut UiScreenWidth, &mut UiScreenHeight, &mut Depth);

    if SectionId != 0 && IniReadSettingByName(SectionId, b"MinimalUI\0".as_ptr() as PCSTR, SettingText.as_mut_ptr(), core::mem::size_of_val(&SettingText)) != 0 {
        UiMinimal = if _stricmp(SettingText.as_ptr() as PCSTR, b"Yes\0".as_ptr() as PCSTR) == 0 { 1u8 } else { 0u8 };
    }

    if UiDisplayMode == VideoGraphicsMode {
        MachVideoSetDisplayMode(core::ptr::null(), 1u8);
        UiDisplayMode = VideoTextMode;
    }

    UiVtbl = if UiMinimal != 0 { MiniTuiVtbl } else { TuiVtbl };

    if ((*UiVtbl).Initialize.unwrap())() == 0 {
        MachVideoSetDisplayMode(core::ptr::null(), 0u8);
        #[cfg(feature = "UEFIBOOT")]
        {
            TRACE(b"TUI init failed, falling back to NoUI\n\0".as_ptr() as PCSTR);
            UiVtbl = SavedNoUiVtbl;
        }
        #[cfg(not(feature = "UEFIBOOT"))]
        {
            return 0u8;
        }
    }

    if SectionId != 0 {
        #[repr(C)]
        struct Setting {
            SettingName: PCSTR,
            SettingVar: *mut core::ffi::c_void,
            SettingSize: usize,
            SettingType: u8,
        }

        let Settings: [Setting; 21] = [
            Setting { SettingName: b"TitleText\0".as_ptr() as PCSTR, SettingVar: &mut UiTitleBoxTitleText as *mut _ as *mut core::ffi::c_void, SettingSize: core::mem::size_of_val(&UiTitleBoxTitleText), SettingType: 0 },
            Setting { SettingName: b"TimeText\0".as_ptr() as PCSTR, SettingVar: &mut UiTimeText as *mut _ as *mut core::ffi::c_void, SettingSize: core::mem::size_of_val(&UiTimeText), SettingType: 0 },
            Setting { SettingName: b"ShowTime\0".as_ptr() as PCSTR, SettingVar: &mut UiShowTime as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 1 },
            Setting { SettingName: b"MenuBox\0".as_ptr() as PCSTR, SettingVar: &mut UiMenuBox as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 1 },
            Setting { SettingName: b"CenterMenu\0".as_ptr() as PCSTR, SettingVar: &mut UiCenterMenu as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 1 },
            Setting { SettingName: b"SpecialEffects\0".as_ptr() as PCSTR, SettingVar: &mut UiUseSpecialEffects as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 1 },
            Setting { SettingName: b"BackdropColor\0".as_ptr() as PCSTR, SettingVar: &mut UiBackdropBgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"BackdropTextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiBackdropFgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"StatusBarColor\0".as_ptr() as PCSTR, SettingVar: &mut UiStatusBarBgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"StatusBarTextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiStatusBarFgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"TitleBoxColor\0".as_ptr() as PCSTR, SettingVar: &mut UiTitleBoxBgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"TitleBoxTextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiTitleBoxFgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"MessageBoxColor\0".as_ptr() as PCSTR, SettingVar: &mut UiMessageBoxBgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"MessageBoxTextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiMessageBoxFgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"MenuColor\0".as_ptr() as PCSTR, SettingVar: &mut UiMenuBgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"MenuTextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiMenuFgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"TextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiTextColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"SelectedColor\0".as_ptr() as PCSTR, SettingVar: &mut UiSelectedTextBgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"SelectedTextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiSelectedTextColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"EditBoxColor\0".as_ptr() as PCSTR, SettingVar: &mut UiEditBoxBgColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
            Setting { SettingName: b"EditBoxTextColor\0".as_ptr() as PCSTR, SettingVar: &mut UiEditBoxTextColor as *mut _ as *mut core::ffi::c_void, SettingSize: 0, SettingType: 2 },
        ];

        for i in 0..21 {
            if IniReadSettingByName(SectionId, Settings[i].SettingName, SettingText.as_mut_ptr(), core::mem::size_of_val(&SettingText)) == 0 {
                continue;
            }

            match Settings[i].SettingType {
                0 => {
                    RtlStringCbCopyA(Settings[i].SettingVar as *mut u8, Settings[i].SettingSize, SettingText.as_ptr());
                },
                1 => {
                    *(Settings[i].SettingVar as PBOOLEAN) = if _stricmp(SettingText.as_ptr() as PCSTR, b"Yes\0".as_ptr() as PCSTR) == 0 { 1u8 } else { 0u8 };
                },
                2 => {
                    *(Settings[i].SettingVar as *mut u8) = UiTextToColor(SettingText.as_ptr() as PCSTR);
                },
                3 => {
                    *(Settings[i].SettingVar as *mut u8) = UiTextToFillStyle(SettingText.as_ptr() as PCSTR);
                },
                _ => {},
            }
        }
    }

    UiFadeInBackdrop();

    TRACE(b"UiInitialize() returning TRUE.\n\0".as_ptr() as PCSTR);
    1u8
}

pub unsafe fn UiUnInitialize(BootText: PCSTR) {
    UiDrawBackdrop(UiGetScreenHeight());
    UiDrawStatusText(BootText);
    UiInfoBox(BootText);

    ((*UiVtbl).UnInitialize)();
}

pub unsafe fn UiDrawBackdrop(DrawHeight: ULONG) {
    ((*UiVtbl).DrawBackdrop)(DrawHeight);
}

pub unsafe extern "system" fn UiFillArea(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    FillChar: CHAR,
    Attr: UCHAR,
) {
    ((*UiVtbl).FillArea)(Left, Top, Right, Bottom, FillChar, Attr);
}

unsafe fn UiDrawShadow(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG) {
    ((*UiVtbl).DrawShadow)(Left, Top, Right, Bottom);
}

pub unsafe fn UiDrawBox(
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
    (UiVtbl.DrawBox)(Left, Top, Right, Bottom, VertStyle, HorzStyle, Fill, Shadow, Attr);
}

pub unsafe fn UiDrawText(
    X: u32,
    Y: u32,
    Text: PCSTR,
    Attr: u8,
) {
    ((*UiVtbl).DrawText)(X, Y, Text, Attr);
}

pub unsafe fn UiDrawText2(
    X: u32,
    Y: u32,
    MaxNumChars: u32,
    Text: PCSTR,
    Attr: u8,
) {
    ((*UiVtbl).DrawText2)(X, Y, MaxNumChars, Text, Attr);
}

pub unsafe extern "system" fn UiDrawCenteredText(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    TextString: PCSTR,
    Attr: UCHAR,
) {
    (UiVtbl.DrawCenteredText)(Left, Top, Right, Bottom, TextString, Attr);
}

pub unsafe fn UiDrawStatusText(StatusText: PCSTR) {
    ((*UiVtbl).DrawStatusText)(StatusText);
}

unsafe fn UiUpdateDateTime() {
    (UiVtbl.UpdateDateTime.unwrap())();
}

unsafe fn UiInfoBox(MessageText: PCSTR) {

    let TextLength = strlen(MessageText);

    let mut LineBreakCount: ULONG = 0;
    let mut BoxWidth: ULONG = 0;
    let mut LastIndex: usize = 0;
    
    for Index in 0..TextLength {
        if *MessageText.add(Index) == b'\n' {
            LastIndex = Index;
            LineBreakCount += 1;
        } else {
            if (Index - LastIndex) > BoxWidth as usize {
                BoxWidth = (Index - LastIndex) as ULONG;
            }
        }
    }

    BoxWidth += 6;
    let BoxHeight = LineBreakCount + 4;

    let Left = (UiScreenWidth / 2) - (BoxWidth / 2);
    let Top = (UiScreenHeight / 2) - (BoxHeight / 2);
    let Right = (UiScreenWidth / 2) + (BoxWidth / 2);
    let Bottom = (UiScreenHeight / 2) + (BoxHeight / 2);

    UiDrawBox(
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

    UiDrawCenteredText(
        Left,
        Top,
        Right,
        Bottom,
        MessageText,
        ATTR(UiTextColor, UiMenuBgColor),
    );
}

unsafe fn UiMessageBox(Format: PCSTR, mut args: ...) {
    
    let mut ap: va_list = core::mem::zeroed();
    let mut Buffer: [i8; 1024] = [0; 1024];
    
    va_start(&mut ap as *mut va_list, Format as *const core::ffi::c_void);
    _vsnprintf(Buffer.as_mut_ptr(), 1024 - 1, Format, ap);
    ((*core::ptr::addr_of!(UiVtbl)).MessageBox)(Buffer.as_mut_ptr());
    va_end(&mut ap as *mut va_list);
}

pub unsafe fn UiMessageBoxCritical(MessageText: PCSTR) {
    ((*UiVtbl).MessageBoxCritical)(MessageText);
}

#[no_mangle]
pub unsafe extern "system" fn UiTextToColor(ColorText: PCSTR) -> u8 {
    ((*UiVtbl).TextToColor.unwrap())(ColorText)
}

unsafe fn UiTextToFillStyle(FillStyleText: PCSTR) -> u8 {
    (UiVtbl.TextToFillStyle)(FillStyleText)
}

pub unsafe fn UiInitProgressBar(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    ProgressText: PCSTR,
) {
    UiProgressBar.Left = Left;
    UiProgressBar.Top = Top;
    UiProgressBar.Right = Right;
    UiProgressBar.Bottom = Bottom;

    UiSetProgressBarSubset(0, 100);
    UiProgressBar.Indicator.Count = 0;
    UiProgressBar.Indicator.Expected = 25;
    UiProgressBar.Indicator.Percentage = 0;

    UiProgressBar.Show = 1u8;

    (UiVtbl.SetProgressBarText.unwrap())(ProgressText);
    (UiVtbl.TickProgressBar.unwrap())(0);
}

unsafe fn UiIndicateProgress() {
    let mut Percentage: ULONG;

    UiProgressBar.Indicator.Count = UiProgressBar.Indicator.Count.wrapping_add(1);

    Percentage = 100u32.wrapping_mul(UiProgressBar.Indicator.Count)
        .wrapping_div(UiProgressBar.Indicator.Expected);
    Percentage = if Percentage < 99 { Percentage } else { 99 };

    if Percentage != UiProgressBar.Indicator.Percentage {
        UiProgressBar.Indicator.Percentage = Percentage;
        UiUpdateProgressBar(Percentage, core::ptr::null());
    }
}

pub unsafe fn UiSetProgressBarSubset(
    Floor: ULONG,
    Ceiling: ULONG,
) {
    UiProgressBar.State.Floor = Floor * 100;
    UiProgressBar.State.Bias = Ceiling - Floor;
}

unsafe fn UiUpdateProgressBar(
    Percentage: ULONG,
    ProgressText: PCSTR,
) {
    let mut TotalProgress: ULONG;

    if UiProgressBar.Show == 0 {
        return;
    }

    if !ProgressText.is_null() {
        UiSetProgressBarText(ProgressText);
    }

    TotalProgress = UiProgressBar.State.Floor + (Percentage * UiProgressBar.State.Bias);

    ((*UiVtbl).TickProgressBar)(TotalProgress);
}

unsafe fn UiSetProgressBarText(
    ProgressText: PCSTR,
) {
    if UiProgressBar.Show == 0 {
        return;
    }

    ((*UiVtbl).SetProgressBarText)(ProgressText);
}

unsafe fn UiDrawProgressBarCenter(
    ProgressText: PCSTR,
)
{
    ((*UiVtbl).DrawProgressBarCenter)(ProgressText);
}

#[no_mangle]
pub unsafe extern "system" fn UiDrawProgressBar(
    Left: u32,
    Top: u32,
    Right: u32,
    Bottom: u32,
    ProgressText: PCSTR,
) {
    ((*UiVtbl).DrawProgressBar)(Left, Top, Right, Bottom, ProgressText);
}

unsafe fn UiEscapeString(String: PCHAR) {
    let mut Idx: ULONG = 0;
    
    while Idx < strlen(String) {
        if *String.add(Idx as usize) == b'\\' as i8 
            && *String.add((Idx + 1) as usize) == b'n' as i8 
        {
            *String.add(Idx as usize) = b'\n' as i8;
            strcpy(
                String.add((Idx + 1) as usize),
                String.add((Idx + 2) as usize)
            );
        }
        Idx += 1;
    }
}

unsafe fn UiShowMessageBoxesInSection(SectionId: ULONG_PTR) {

    let mut Idx: ULONG;
    let mut SettingName: [i8; 80] = [0; 80];
    let mut SettingValue: [i8; 80] = [0; 80];
    let mut MessageBoxText: PCHAR;
    let mut MessageBoxTextSize: ULONG;

    if SectionId == 0 {
        return;
    }

    Idx = 0;
    while Idx < IniGetNumSectionItems(SectionId) {
        IniReadSettingByNumber(
            SectionId,
            Idx,
            SettingName.as_mut_ptr(),
            core::mem::size_of_val(&SettingName) as ULONG,
            SettingValue.as_mut_ptr(),
            core::mem::size_of_val(&SettingValue) as ULONG,
        );
        if _stricmp(SettingName.as_ptr(), b"MessageBox\0".as_ptr() as PCSTR) != 0 {
            Idx += 1;
            continue;
        }

        MessageBoxTextSize = IniGetSectionSettingValueSize(SectionId, Idx);

        MessageBoxText = FrLdrTempAlloc(MessageBoxTextSize as usize, TAG_UI_TEXT) as PCHAR;
        if MessageBoxText.is_null() {
            Idx += 1;
            continue;
        }

        IniReadSettingByNumber(
            SectionId,
            Idx,
            SettingName.as_mut_ptr(),
            core::mem::size_of_val(&SettingName) as ULONG,
            MessageBoxText,
            MessageBoxTextSize,
        );

        UiEscapeString(MessageBoxText);

        UiMessageBox(MessageBoxText);

        FrLdrTempFree(MessageBoxText as *mut core::ffi::c_void, TAG_UI_TEXT);

        Idx += 1;
    }
}

unsafe fn UiShowMessageBoxesInArgv(
    Argc: ULONG,
    Argv: *mut PCHAR,
) {

    let mut LastIndex: ULONG = 0;
    let mut ArgValue: PCSTR;
    let mut MessageBoxText: PCHAR;
    let MessageBoxTextSize: usize;

    loop {
        ArgValue = GetNextArgumentValue(Argc, Argv, &mut LastIndex, b"MessageBox\0".as_ptr() as PCSTR);
        if ArgValue.is_null() {
            break;
        }

        MessageBoxTextSize = (strlen(ArgValue) + 1) * core::mem::size_of::<i8>();

        MessageBoxText = FrLdrTempAlloc(MessageBoxTextSize, TAG_UI_TEXT) as PCHAR;
        if MessageBoxText.is_null() {
            LastIndex += 1;
            continue;
        }

        strcpy(MessageBoxText, ArgValue);

        UiEscapeString(MessageBoxText);

        UiMessageBox(b"%s\0".as_ptr() as PCSTR, MessageBoxText);

        FrLdrTempFree(MessageBoxText as *mut core::ffi::c_void, TAG_UI_TEXT);

        LastIndex += 1;
    }
}

unsafe fn UiDisplayMenu(
    MenuHeader: PCSTR,
    MenuFooter: PCSTR,
    MenuItemList: *mut PCSTR,
    MenuItemCount: ULONG,
    DefaultMenuItem: ULONG,
    MenuTimeOut: LONG,
    SelectedMenuItem: PULONG,
    CanEscape: BOOLEAN,
    KeyPressFilter: UiMenuKeyPressFilterCallback,
    Context: PVOID,
) -> BOOLEAN {
    ((*UiVtbl).DisplayMenu)(
        MenuHeader,
        MenuFooter,
        MenuItemList,
        MenuItemCount,
        DefaultMenuItem,
        MenuTimeOut,
        SelectedMenuItem,
        CanEscape,
        KeyPressFilter,
        Context,
    )
}

pub unsafe extern "system" fn UiFadeInBackdrop() {
    ((*UiVtbl).FadeInBackdrop.unwrap())();
}

pub unsafe fn UiFadeOut() {
    ((*UiVtbl).FadeOut.unwrap())();
}

#[no_mangle]
pub unsafe extern "system" fn UiEditBox(
    MessageText: PCSTR,
    EditTextBuffer: PCHAR,
    Length: ULONG,
) -> BOOLEAN {
    ((*UiVtbl).EditBox)(MessageText, EditTextBuffer, Length)
}

#[no_mangle]
pub unsafe extern "system" fn UiResetForSOS() {
    #[cfg(target_arch = "arm")]
    {
        UiInitialize(1u8);
    }
    #[cfg(all(not(target_arch = "arm"), not(feature = "UEFIBOOT")))]
    {
        (UiVtbl.UnInitialize.unwrap())();
        UiVtbl = MiniTuiVtbl;
        (UiVtbl.Initialize.unwrap())();
    }
    UiProgressBar.Show = 0u8;
}

unsafe fn UiGetScreenHeight() -> ULONG {
    UiScreenHeight
}

unsafe fn UiGetMenuBgColor() -> UCHAR {
    UiMenuBgColor
}


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
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(UI);
//
// UCHAR UiStatusBarFgColor;       // Status bar foreground color
// UCHAR UiStatusBarBgColor;       // Status bar background color
// UCHAR UiBackdropFgColor;        // Backdrop foreground color
// UCHAR UiBackdropBgColor;        // Backdrop background color
// UCHAR UiBackdropFillStyle;      // Backdrop fill style
// UCHAR UiTitleBoxFgColor;        // Title box foreground color
// UCHAR UiTitleBoxBgColor;        // Title box background color
// UCHAR UiMessageBoxFgColor;      // Message box foreground color
// UCHAR UiMessageBoxBgColor;      // Message box background color
// UCHAR UiMenuFgColor;            // Menu foreground color
// UCHAR UiMenuBgColor;            // Menu background color
// UCHAR UiTextColor;              // Normal text color
// UCHAR UiSelectedTextColor;      // Selected text color
// UCHAR UiSelectedTextBgColor;    // Selected text background color
// UCHAR UiEditBoxTextColor;       // Edit box text color
// UCHAR UiEditBoxBgColor;         // Edit box text background color
//
// BOOLEAN UiShowTime;             // Whether to draw the time
// BOOLEAN UiMenuBox;              // Whether to draw a box around the menu
// BOOLEAN UiCenterMenu;           // Whether to use a centered or left-aligned menu
// BOOLEAN UiUseSpecialEffects;    // Whether to use fade effects
//
// CHAR UiTitleBoxTitleText[260] = "Boot Menu";    // Title box's title text
// CHAR UiTimeText[260] = "[Time Remaining: %d]";
//
// const PCSTR UiMonthNames[12] = { "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December" };
//
// #define TAG_UI_TEXT 'xTiU'
//
// ULONG UiScreenWidth;    // Screen Width
// ULONG UiScreenHeight;   // Screen Height
//
// /*
//  * Loading progress bar, based on the NTOS Inbv one.
//  * Supports progress within sub-ranges, used when loading
//  * with an unknown number of steps.
//  */
// UI_PROGRESS_BAR UiProgressBar = {{0}};
//
// static const UIVTBL SavedNoUiVtbl =
// {
//     NoUiInitialize,
//     NoUiUnInitialize,
//     NoUiDrawBackdrop,
//     NoUiFillArea,
//     NoUiDrawShadow,
//     NoUiDrawBox,
//     NoUiDrawText,
//     NoUiDrawText2,
//     NoUiDrawCenteredText,
//     NoUiDrawStatusText,
//     NoUiUpdateDateTime,
//     NoUiMessageBox,
//     NoUiMessageBoxCritical,
//     NoUiDrawProgressBarCenter,
//     NoUiDrawProgressBar,
//     NoUiSetProgressBarText,
//     NoUiTickProgressBar,
//     NoUiEditBox,
//     NoUiTextToColor,
//     NoUiTextToFillStyle,
//     NoUiFadeInBackdrop,
//     NoUiFadeOut,
//     NoUiDisplayMenu,
//     NoUiDrawMenu,
// };
//
// UIVTBL UiVtbl =
// {
//     NoUiInitialize,
//     NoUiUnInitialize,
//     NoUiDrawBackdrop,
//     NoUiFillArea,
//     NoUiDrawShadow,
//     NoUiDrawBox,
//     NoUiDrawText,
//     NoUiDrawText2,
//     NoUiDrawCenteredText,
//     NoUiDrawStatusText,
//     NoUiUpdateDateTime,
//     NoUiMessageBox,
//     NoUiMessageBoxCritical,
//     NoUiDrawProgressBarCenter,
//     NoUiDrawProgressBar,
//     NoUiSetProgressBarText,
//     NoUiTickProgressBar,
//     NoUiEditBox,
//     NoUiTextToColor,
//     NoUiTextToFillStyle,
//     NoUiFadeInBackdrop,
//     NoUiFadeOut,
//     NoUiDisplayMenu,
//     NoUiDrawMenu,
// };
//
// BOOLEAN UiInitialize(BOOLEAN ShowUi)
// {
//     VIDEODISPLAYMODE UiDisplayMode; // Tells us if we are in text or graphics mode
//     BOOLEAN UiMinimal = FALSE;      // Tells us if we are using a minimal console-like UI
//     ULONG_PTR SectionId;
//     ULONG Depth;
//     CHAR  SettingText[260];
//
//     if (!ShowUi)
//     {
//         if (!UiVtbl.Initialize())
//         {
//             MachVideoSetDisplayMode(NULL, FALSE);
//             return FALSE;
//         }
//         return TRUE;
//     }
//
//     TRACE("Initializing User Interface.\n");
//     TRACE("Reading UI settings from [Display] section.\n");
//
//     /* Open the [Display] section */
//     if (!IniOpenSection("Display", &SectionId))
//         SectionId = 0;
//
//     /* Select the video mode */
//     SettingText[0] = '\0';
//     if ((SectionId != 0) && !IniReadSettingByName(SectionId, "DisplayMode", SettingText, sizeof(SettingText)))
//     {
//         SettingText[0] = '\0';
//     }
//     UiDisplayMode = MachVideoSetDisplayMode(SettingText, TRUE);
//     MachVideoGetDisplaySize(&UiScreenWidth, &UiScreenHeight, &Depth);
//
//     /* Select the UI */
//     if ((SectionId != 0) && IniReadSettingByName(SectionId, "MinimalUI", SettingText, sizeof(SettingText)))
//     {
//         UiMinimal = (_stricmp(SettingText, "Yes") == 0);
//     }
//
//     if (UiDisplayMode == VideoGraphicsMode)
// #if 0 // We don't support a GUI mode yet.
//         UiVtbl = GuiVtbl;
// #else
//     {
//         // Switch back to text mode.
//         MachVideoSetDisplayMode(NULL, TRUE);
//         UiDisplayMode = VideoTextMode;
//     }
// #endif
//     else // if (UiDisplayMode == VideoTextMode)
//         UiVtbl = (UiMinimal ? MiniTuiVtbl : TuiVtbl);
//
//     /* Load the UI and initialize its default settings */
//     if (!UiVtbl.Initialize())
//     {
//         MachVideoSetDisplayMode(NULL, FALSE);
// #ifdef UEFIBOOT
//         /* On UEFI without video (no GOP), fall back to NoUI
//          * so RunLoader can continue loading the OS headless */
//         TRACE("TUI init failed, falling back to NoUI\n");
//         UiVtbl = SavedNoUiVtbl;
// #else
//         return FALSE;
// #endif
//     }
//
//     /* Load the user UI settings */
//     if (SectionId != 0)
//     {
//         static const struct
//         {
//             PCSTR SettingName;
//             PVOID SettingVar;
//             SIZE_T SettingSize OPTIONAL; // Must be non-zero only for text buffers.
//             UCHAR SettingType; // 0: Text, 1: Yes/No, 2: Color, 3: Fill style
//         } Settings[] =
//         {
//             {"TitleText", &UiTitleBoxTitleText, sizeof(UiTitleBoxTitleText), 0},
//             {"TimeText" , &UiTimeText, sizeof(UiTimeText), 0},
//
//             {"ShowTime"      , &UiShowTime         , 0, 1},
//             {"MenuBox"       , &UiMenuBox          , 0, 1},
//             {"CenterMenu"    , &UiCenterMenu       , 0, 1},
//             {"SpecialEffects", &UiUseSpecialEffects, 0, 1},
//
//             {"BackdropColor"      , &UiBackdropBgColor    , 0, 2},
//             {"BackdropTextColor"  , &UiBackdropFgColor    , 0, 2},
//             {"StatusBarColor"     , &UiStatusBarBgColor   , 0, 2},
//             {"StatusBarTextColor" , &UiStatusBarFgColor   , 0, 2},
//             {"TitleBoxColor"      , &UiTitleBoxBgColor    , 0, 2},
//             {"TitleBoxTextColor"  , &UiTitleBoxFgColor    , 0, 2},
//             {"MessageBoxColor"    , &UiMessageBoxBgColor  , 0, 2},
//             {"MessageBoxTextColor", &UiMessageBoxFgColor  , 0, 2},
//             {"MenuColor"          , &UiMenuBgColor        , 0, 2},
//             {"MenuTextColor"      , &UiMenuFgColor        , 0, 2},
//             {"TextColor"          , &UiTextColor          , 0, 2},
//             {"SelectedColor"      , &UiSelectedTextBgColor, 0, 2},
//             {"SelectedTextColor"  , &UiSelectedTextColor  , 0, 2},
//             {"EditBoxColor"       , &UiEditBoxBgColor     , 0, 2},
//             {"EditBoxTextColor"   , &UiEditBoxTextColor   , 0, 2},
//
//             {"BackdropFillStyle", &UiBackdropFillStyle, 0, 3},
//         };
//         ULONG i;
//
//         for (i = 0; i < RTL_NUMBER_OF(Settings); ++i)
//         {
//             if (!IniReadSettingByName(SectionId, Settings[i].SettingName, SettingText, sizeof(SettingText)))
//                 continue;
//
//             switch (Settings[i].SettingType)
//             {
//             case 0: // Text
//                 RtlStringCbCopyA((PCHAR)Settings[i].SettingVar,
//                                  Settings[i].SettingSize, SettingText);
//                 break;
//             case 1: // Yes/No
//                 *(PBOOLEAN)Settings[i].SettingVar = (_stricmp(SettingText, "Yes") == 0);
//                 break;
//             case 2: // Color
//                 *(PUCHAR)Settings[i].SettingVar = UiTextToColor(SettingText);
//                 break;
//             case 3: // Fill style
//                 *(PUCHAR)Settings[i].SettingVar = UiTextToFillStyle(SettingText);
//                 break;
//             default:
//                 break;
//             }
//         }
//     }
//
//     /* Draw the backdrop and fade it in if special effects are enabled */
//     UiFadeInBackdrop();
//
//     TRACE("UiInitialize() returning TRUE.\n");
//     return TRUE;
// }
//
// VOID UiUnInitialize(PCSTR BootText)
// {
//     UiDrawBackdrop(UiGetScreenHeight());
//     UiDrawStatusText(BootText);
//     UiInfoBox(BootText);
//
//     UiVtbl.UnInitialize();
// }
//
// VOID UiDrawBackdrop(ULONG DrawHeight)
// {
//     UiVtbl.DrawBackdrop(DrawHeight);
// }
//
// VOID UiFillArea(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom, CHAR FillChar, UCHAR Attr /* Color Attributes */)
// {
//     UiVtbl.FillArea(Left, Top, Right, Bottom, FillChar, Attr);
// }
//
// VOID UiDrawShadow(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom)
// {
//     UiVtbl.DrawShadow(Left, Top, Right, Bottom);
// }
//
// VOID UiDrawBox(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom, UCHAR VertStyle, UCHAR HorzStyle, BOOLEAN Fill, BOOLEAN Shadow, UCHAR Attr)
// {
//     UiVtbl.DrawBox(Left, Top, Right, Bottom, VertStyle, HorzStyle, Fill, Shadow, Attr);
// }
//
// VOID
// UiDrawText(
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_ PCSTR Text,
//     _In_ UCHAR Attr)
// {
//     UiVtbl.DrawText(X, Y, Text, Attr);
// }
//
// VOID
// UiDrawText2(
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_opt_ ULONG MaxNumChars,
//     _In_reads_or_z_(MaxNumChars) PCSTR Text,
//     _In_ UCHAR Attr)
// {
//     UiVtbl.DrawText2(X, Y, MaxNumChars, Text, Attr);
// }
//
// VOID
// UiDrawCenteredText(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR TextString,
//     _In_ UCHAR Attr)
// {
//     UiVtbl.DrawCenteredText(Left, Top, Right, Bottom, TextString, Attr);
// }
//
// VOID UiDrawStatusText(PCSTR StatusText)
// {
//     UiVtbl.DrawStatusText(StatusText);
// }
//
// VOID UiUpdateDateTime(VOID)
// {
//     UiVtbl.UpdateDateTime();
// }
//
// VOID
// UiInfoBox(
//     _In_ PCSTR MessageText)
// {
//     SIZE_T TextLength;
//     ULONG  BoxWidth;
//     ULONG  BoxHeight;
//     ULONG  LineBreakCount;
//     SIZE_T Index;
//     SIZE_T LastIndex;
//     ULONG  Left;
//     ULONG  Top;
//     ULONG  Right;
//     ULONG  Bottom;
//
//     TextLength = strlen(MessageText);
//
//     /* Count the new lines and the box width */
//     LineBreakCount = 0;
//     BoxWidth = 0;
//     LastIndex = 0;
//     for (Index=0; Index<TextLength; Index++)
//     {
//         if (MessageText[Index] == '\n')
//         {
//             LastIndex = Index;
//             LineBreakCount++;
//         }
//         else
//         {
//             if ((Index - LastIndex) > BoxWidth)
//             {
//                 BoxWidth = (ULONG)(Index - LastIndex);
//             }
//         }
//     }
//
//     /* Calc the box width & height */
//     BoxWidth += 6;
//     BoxHeight = LineBreakCount + 4;
//
//     /* Calc the box coordinates */
//     Left = (UiScreenWidth / 2) - (BoxWidth / 2);
//     Top  = (UiScreenHeight / 2) - (BoxHeight / 2);
//     Right  = (UiScreenWidth / 2) + (BoxWidth / 2);
//     Bottom = (UiScreenHeight / 2) + (BoxHeight / 2);
//
//     /* Draw the box */
//     UiDrawBox(Left,
//               Top,
//               Right,
//               Bottom,
//               VERT,
//               HORZ,
//               TRUE,
//               TRUE,
//               ATTR(UiMenuFgColor, UiMenuBgColor));
//
//     /* Draw the text */
//     UiDrawCenteredText(Left, Top, Right, Bottom, MessageText, ATTR(UiTextColor, UiMenuBgColor));
// }
//
// VOID
// UiMessageBox(
//     _In_ PCSTR Format, ...)
// {
//     va_list ap;
//     CHAR Buffer[1024];
//
//     va_start(ap, Format);
//     _vsnprintf(Buffer, sizeof(Buffer) - sizeof(CHAR), Format, ap);
//     UiVtbl.MessageBox(Buffer);
//     va_end(ap);
// }
//
// VOID
// UiMessageBoxCritical(
//     _In_ PCSTR MessageText)
// {
//     UiVtbl.MessageBoxCritical(MessageText);
// }
//
// UCHAR UiTextToColor(PCSTR ColorText)
// {
//     return UiVtbl.TextToColor(ColorText);
// }
//
// UCHAR UiTextToFillStyle(PCSTR FillStyleText)
// {
//     return UiVtbl.TextToFillStyle(FillStyleText);
// }
//
// VOID
// UiInitProgressBar(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR ProgressText)
// {
//     /* Progress bar area */
//     UiProgressBar.Left = Left;
//     UiProgressBar.Top  = Top;
//     UiProgressBar.Right  = Right;
//     UiProgressBar.Bottom = Bottom;
//     // UiProgressBar.Width = Right - Left + 1;
//
//     /* Set the progress bar ranges */
//     UiSetProgressBarSubset(0, 100);
//     UiProgressBar.Indicator.Count = 0;
//     UiProgressBar.Indicator.Expected = 25;
//     UiProgressBar.Indicator.Percentage = 0;
//
//     /* Enable the progress bar */
//     UiProgressBar.Show = TRUE;
//
//     /* Initial drawing: set the "Loading..." text and the original position */
//     UiVtbl.SetProgressBarText(ProgressText);
//     UiVtbl.TickProgressBar(0);
// }
//
// VOID
// UiIndicateProgress(VOID)
// {
//     ULONG Percentage;
//
//     /* Increase progress */
//     UiProgressBar.Indicator.Count++;
//
//     /* Compute the new percentage - Don't go over 100% */
//     Percentage = 100 * UiProgressBar.Indicator.Count /
//                        UiProgressBar.Indicator.Expected;
//     Percentage = min(Percentage, 99);
//
//     if (Percentage != UiProgressBar.Indicator.Percentage)
//     {
//         /* Percentage has changed, update the progress bar */
//         UiProgressBar.Indicator.Percentage = Percentage;
//         UiUpdateProgressBar(Percentage, NULL);
//     }
// }
//
// VOID
// UiSetProgressBarSubset(
//     _In_ ULONG Floor,
//     _In_ ULONG Ceiling)
// {
//     /* Sanity checks */
//     ASSERT(Floor < Ceiling);
//     ASSERT(Ceiling <= 100);
//
//     /* Update the progress bar state */
//     UiProgressBar.State.Floor = Floor * 100;
//     // UiProgressBar.State.Ceiling = Ceiling * 100;
//     UiProgressBar.State.Bias = Ceiling - Floor;
// }
//
// VOID
// UiUpdateProgressBar(
//     _In_ ULONG Percentage,
//     _In_opt_ PCSTR ProgressText)
// {
//     ULONG TotalProgress;
//
//     /* Make sure the progress bar is enabled */
//     if (!UiProgressBar.Show)
//         return;
//
//     /* Set the progress text if specified */
//     if (ProgressText)
//         UiSetProgressBarText(ProgressText);
//
//     /* Compute the total progress and tick the progress bar */
//     TotalProgress = UiProgressBar.State.Floor + (Percentage * UiProgressBar.State.Bias);
//     // TotalProgress /= (100 * 100);
//
//     UiVtbl.TickProgressBar(TotalProgress);
// }
//
// VOID
// UiSetProgressBarText(
//     _In_ PCSTR ProgressText)
// {
//     /* Make sure the progress bar is enabled */
//     if (!UiProgressBar.Show)
//         return;
//
//     UiVtbl.SetProgressBarText(ProgressText);
// }
//
// VOID
// UiDrawProgressBarCenter(
//     _In_ PCSTR ProgressText)
// {
//     UiVtbl.DrawProgressBarCenter(ProgressText);
// }
//
// VOID
// UiDrawProgressBar(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR ProgressText)
// {
//     UiVtbl.DrawProgressBar(Left, Top, Right, Bottom, ProgressText);
// }
//
// static VOID
// UiEscapeString(PCHAR String)
// {
//     ULONG    Idx;
//
//     for (Idx=0; Idx<strlen(String); Idx++)
//     {
//         // Escape the new line characters
//         if (String[Idx] == '\\' && String[Idx+1] == 'n')
//         {
//             // Escape the character
//             String[Idx] = '\n';
//
//             // Move the rest of the string up
//             strcpy(&String[Idx+1], &String[Idx+2]);
//         }
//     }
// }
//
// VOID
// UiShowMessageBoxesInSection(
//     IN ULONG_PTR SectionId)
// {
//     ULONG Idx;
//     CHAR  SettingName[80];
//     CHAR  SettingValue[80];
//     PCHAR MessageBoxText;
//     ULONG MessageBoxTextSize;
//
//     if (SectionId == 0)
//         return;
//
//     /* Find all the message box settings and run them */
//     for (Idx = 0; Idx < IniGetNumSectionItems(SectionId); Idx++)
//     {
//         IniReadSettingByNumber(SectionId, Idx, SettingName, sizeof(SettingName), SettingValue, sizeof(SettingValue));
//         if (_stricmp(SettingName, "MessageBox") != 0)
//             continue;
//
//         /* Get the real length of the MessageBox text */
//         MessageBoxTextSize = IniGetSectionSettingValueSize(SectionId, Idx);
//         // if (MessageBoxTextSize <= 0)
//             // continue;
//
//         /* Allocate enough memory to hold the text */
//         MessageBoxText = FrLdrTempAlloc(MessageBoxTextSize, TAG_UI_TEXT);
//         if (!MessageBoxText)
//             continue;
//
//         /* Get the MessageBox text */
//         IniReadSettingByNumber(SectionId, Idx, SettingName, sizeof(SettingName), MessageBoxText, MessageBoxTextSize);
//
//         /* Fix it up */
//         UiEscapeString(MessageBoxText);
//
//         /* Display it */
//         UiMessageBox(MessageBoxText);
//
//         /* Free the memory */
//         FrLdrTempFree(MessageBoxText, TAG_UI_TEXT);
//     }
// }
//
// VOID
// UiShowMessageBoxesInArgv(
//     IN ULONG Argc,
//     IN PCHAR Argv[])
// {
//     ULONG LastIndex;
//     PCSTR ArgValue;
//     PCHAR MessageBoxText;
//     SIZE_T MessageBoxTextSize;
//
//     /* Find all the message box settings and run them */
//     for (LastIndex = 0;
//          (ArgValue = GetNextArgumentValue(Argc, Argv, &LastIndex, "MessageBox")) != NULL;
//          ++LastIndex)
//     {
//         /* Get the real length of the MessageBox text */
//         MessageBoxTextSize = (strlen(ArgValue) + 1) * sizeof(CHAR);
//
//         /* Allocate enough memory to hold the text */
//         MessageBoxText = FrLdrTempAlloc(MessageBoxTextSize, TAG_UI_TEXT);
//         if (!MessageBoxText)
//             continue;
//
//         /* Get the MessageBox text */
//         strcpy(MessageBoxText, ArgValue);
//
//         /* Fix it up */
//         UiEscapeString(MessageBoxText);
//
//         /* Display it */
//         UiMessageBox(MessageBoxText);
//
//         /* Free the memory */
//         FrLdrTempFree(MessageBoxText, TAG_UI_TEXT);
//     }
// }
//
// BOOLEAN
// UiDisplayMenu(
//     IN PCSTR MenuHeader,
//     IN PCSTR MenuFooter OPTIONAL,
//     IN PCSTR MenuItemList[],
//     IN ULONG MenuItemCount,
//     IN ULONG DefaultMenuItem,
//     IN LONG MenuTimeOut,
//     OUT PULONG SelectedMenuItem,
//     IN BOOLEAN CanEscape,
//     IN UiMenuKeyPressFilterCallback KeyPressFilter OPTIONAL,
//     IN PVOID Context OPTIONAL)
// {
//     return UiVtbl.DisplayMenu(MenuHeader, MenuFooter,
//                               MenuItemList, MenuItemCount, DefaultMenuItem,
//                               MenuTimeOut, SelectedMenuItem, CanEscape,
//                               KeyPressFilter, Context);
// }
//
// VOID UiFadeInBackdrop(VOID)
// {
//     UiVtbl.FadeInBackdrop();
// }
//
// VOID UiFadeOut(VOID)
// {
//     UiVtbl.FadeOut();
// }
//
// BOOLEAN UiEditBox(PCSTR MessageText, PCHAR EditTextBuffer, ULONG Length)
// {
//     return UiVtbl.EditBox(MessageText, EditTextBuffer, Length);
// }
//
// VOID
// UiResetForSOS(VOID)
// {
// #ifdef _M_ARM
//     /* Re-initialize the UI */
//     UiInitialize(TRUE);
// #elif defined(UEFIBOOT)
//     /* On UEFI without video (no GOP), keep current vtbl (NoUI).
//      * Switching to MiniTui would hang on framebuffer printf. */
// #else
//     /* Reset the UI and switch to MiniTui */
//     UiVtbl.UnInitialize();
//     UiVtbl = MiniTuiVtbl;
//     UiVtbl.Initialize();
// #endif
//     /* Disable the progress bar */
//     UiProgressBar.Show = FALSE;
// }
//
// ULONG
// UiGetScreenHeight(VOID)
// {
//     return UiScreenHeight;
// }
//
// UCHAR
// UiGetMenuBgColor(VOID)
// {
//     return UiMenuBgColor;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
