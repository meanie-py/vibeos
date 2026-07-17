use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::*;
use crate::ui::{UiStatusBarFgColor, UiStatusBarBgColor, UiBackdropFgColor, UiBackdropBgColor, UiBackdropFillStyle, UiTitleBoxFgColor, UiTitleBoxBgColor, UiMessageBoxFgColor, UiMessageBoxBgColor, UiMenuFgColor, UiMenuBgColor, UiTextColor, UiSelectedTextColor, UiSelectedTextBgColor, UiEditBoxTextColor, UiEditBoxBgColor, UiShowTime, UiMenuBox, UiCenterMenu, UiUseSpecialEffects, UiScreenWidth, UiScreenHeight, UiTitleBoxTitleText, UiTimeText, UiMonthNames, UiProgressBar, UiVtbl};
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;
use crate::tuimenu::{TuiDrawMenuBox, TuiDrawMenuItem};

extern "C" {
    fn TuiFillArea( Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, FillChar: CHAR, Attr: UCHAR, );
    fn VideoCopyOffScreenBufferToVRAM();
    fn TuiInitialize() -> BOOLEAN;
    fn TuiTextToColor(text: *const u8) -> UCHAR;
    fn UiInitProgressBar( Left: u32, Top: u32, Right: u32, Bottom: u32, ProgressText: PCSTR, );
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiInitialize() -> BOOLEAN {

    if TuiInitialize() == 0 {
        return 0u8;
    }

    UiTextColor = TuiTextToColor(b"Default\0".as_ptr());

    UiStatusBarFgColor = UiTextColor;
    UiStatusBarBgColor = COLOR_BLACK;
    UiBackdropFgColor = UiTextColor;
    UiBackdropBgColor = COLOR_BLACK;
    UiBackdropFillStyle = b' ';
    UiTitleBoxFgColor = COLOR_WHITE;
    UiTitleBoxBgColor = COLOR_BLACK;
    UiMenuFgColor = UiTextColor;
    UiMenuBgColor = COLOR_BLACK;
    UiSelectedTextColor = COLOR_BLACK;
    UiSelectedTextBgColor = UiTextColor;

    UiShowTime = 0u8;
    UiMenuBox = 0u8;
    UiCenterMenu = 0u8;
    UiUseSpecialEffects = 0u8;

    *UiTitleBoxTitleText.as_mut_ptr() = (0u8) as i8 ;

    RtlStringCbCopyA(
        UiTimeText.as_mut_ptr(),
        core::mem::size_of_val(&UiTimeText),
        (b"Seconds until highlighted choice will be started automatically:\0".as_ptr()) as *const i8 ,
    );

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiDrawBackdrop(DrawHeight: ULONG) {

    TuiFillArea(
        0,
        0,
        UiScreenWidth - 1,
        DrawHeight - 1,
        (UiBackdropFillStyle) as i8 ,
        ATTR(UiBackdropFgColor, UiBackdropBgColor),
    );

    VideoCopyOffScreenBufferToVRAM();
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiFadeInBackdrop() {
    MiniTuiDrawBackdrop(UiScreenHeight);
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiDrawStatusText(_StatusText: PCSTR) {
    /* Minimal UI doesn't have a status bar */
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiSetProgressBarText(
    ProgressText: PCSTR
) {
    let ProgressBarWidth: ULONG;
    let mut ProgressString: [u8; 256] = [0; 256];

    ProgressBarWidth = UiProgressBar.Right - UiProgressBar.Left + 1;

    RtlStringCbCopyA(ProgressString.as_mut_ptr() as *mut i8, core::mem::size_of_val(&ProgressString), ProgressText as *const i8);
    TuiTruncateStringEllipsis((ProgressString.as_mut_ptr()) as *mut i8 , ProgressBarWidth);

    TuiFillArea(UiProgressBar.Left, UiProgressBar.Top,
                UiProgressBar.Right,
                UiProgressBar.Bottom - 2,
                b' ' as i8, ATTR(UiTextColor, UiMenuBgColor));

    TuiDrawCenteredText(UiProgressBar.Left, UiProgressBar.Top,
                        UiProgressBar.Right,
                        UiProgressBar.Bottom - 2,
                        (ProgressString.as_mut_ptr()) as *const i8 , ATTR(UiTextColor, UiMenuBgColor));
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiTickProgressBar(SubPercentTimes100: ULONG) {
    let ProgressBarWidth: ULONG;
    let FillCount: ULONG;

    ProgressBarWidth = UiProgressBar.Right - UiProgressBar.Left + 1;

    FillCount = ProgressBarWidth * SubPercentTimes100 / (100 * 100);

    if FillCount > 0 {
        TuiFillArea(
            UiProgressBar.Left,
            UiProgressBar.Bottom,
            UiProgressBar.Left + FillCount - 1,
            UiProgressBar.Bottom,
            0xDBu8 as i8,
            ATTR(UiTextColor, UiMenuBgColor),
        );
    }
    TuiFillArea(
        UiProgressBar.Left + FillCount,
        UiProgressBar.Bottom,
        UiProgressBar.Right,
        UiProgressBar.Bottom,
        b' ' as i8,
        ATTR(UiTextColor, UiMenuBgColor),
    );

    TuiUpdateDateTime();
    VideoCopyOffScreenBufferToVRAM();
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiDrawProgressBarCenter(
    ProgressText: PCSTR,
) {
    let Left: u32;
    let Top: u32;
    let Right: u32;
    let Bottom: u32;
    let Width: u32;
    let Height: u32;

    // NTLDR_PROGRESSBAR off (smoke path) → C `#else` variant only.
    // (rustconv emitted the #ifdef/#else as two brace blocks; decfg's
    //  single-line rule mis-split this multi-line one — resolved by hand.)
    Height = 3;
    Width = UiScreenWidth - 4;
    Left = 2;
    Top = UiScreenHeight - Height - 3;

    Right = Left + Width - 1;
    Bottom = Top + Height - 1;

    MiniTuiDrawProgressBar(Left, Top, Right, Bottom, ProgressText);
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiDrawProgressBar(
    Left: u32,
    Top: u32,
    Right: u32,
    Bottom: u32,
    ProgressText: PCSTR,
) {
    UiInitProgressBar(Left, Top, Right, Bottom, ProgressText);
}

#[no_mangle]
pub unsafe extern "C" fn MiniTuiDrawMenu(MenuInfo: PUI_MENU_INFO) {
    let mut i: ULONG;

    UiDrawBackdrop(UiGetScreenHeight());

    if !(*MenuInfo).MenuHeader.is_null() {
        (UiVtbl.DrawText.unwrap())(
            0,
            (*MenuInfo).Top - 2,
            (*MenuInfo).MenuHeader,
            ATTR(UiMenuFgColor, UiMenuBgColor),
        );
    }

    TuiDrawMenuBox(MenuInfo);

    i = 0;
    while i < (*MenuInfo).MenuItemCount {
        TuiDrawMenuItem(MenuInfo, i);
        i += 1;
    }

    (UiVtbl.DrawText.unwrap())(
        0,
        (*MenuInfo).Bottom + 1,
        b"Use \x18 and \x19 to move the highlight to your choice.\0".as_ptr() as PCHAR,
        ATTR(UiMenuFgColor, UiMenuBgColor),
    );
    (UiVtbl.DrawText.unwrap())(
        0,
        (*MenuInfo).Bottom + 2,
        b"Press ENTER to choose.\0".as_ptr() as PCHAR,
        ATTR(UiMenuFgColor, UiMenuBgColor),
    );

    if !(*MenuInfo).MenuFooter.is_null() {
        (UiVtbl.DrawText.unwrap())(
            0,
            UiScreenHeight - 4,
            (*MenuInfo).MenuFooter,
            ATTR(UiMenuFgColor, UiMenuBgColor),
        );
    }

    VideoCopyOffScreenBufferToVRAM();
}

// Tui* slots reused by MiniTuiVtbl (defined #[no_mangle] in tui.rs).
extern "C" {
    fn TuiUnInitialize();
    fn TuiDrawShadow(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG);
    fn TuiDrawBox(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, VertStyle: UCHAR, HorzStyle: UCHAR, Fill: BOOLEAN, Shadow: BOOLEAN, Attr: UCHAR);
    fn TuiDrawText(X: ULONG, Y: ULONG, Text: PCSTR, Attr: UCHAR);
    fn TuiDrawText2(X: ULONG, Y: ULONG, MaxNumChars: ULONG, Text: PCSTR, Attr: UCHAR);
    fn TuiDrawCenteredText(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, TextString: PCSTR, Attr: UCHAR);
    fn TuiUpdateDateTime();
    fn TuiMessageBox(MessageText: PCSTR);
    fn TuiMessageBoxCritical(MessageText: PCSTR);
    fn TuiEditBox(MessageText: PCSTR, EditTextBuffer: PCHAR, Length: ULONG) -> BOOLEAN;
    fn TuiTextToFillStyle(FillStyleText: PCSTR) -> UCHAR;
    fn TuiFadeOut();
    fn TuiDisplayMenu(MenuHeader: PCSTR, MenuFooter: PCSTR, MenuItemList: *mut PCSTR, MenuItemCount: ULONG, DefaultMenuItem: ULONG, MenuTimeOut: LONG, SelectedMenuItem: *mut ULONG, CanEscape: BOOLEAN, KeyPressFilter: UiMenuKeyPressFilterCallback, Context: PVOID) -> BOOLEAN;
}

// minitui.c:237  const UIVTBL MiniTuiVtbl = { ... } (header-exported, minitui.h)
#[no_mangle]
pub static MiniTuiVtbl: UIVTBL = UIVTBL {
    Initialize: Some(MiniTuiInitialize),
    UnInitialize: Some(TuiUnInitialize),
    DrawBackdrop: Some(MiniTuiDrawBackdrop),
    FillArea: Some(TuiFillArea),
    DrawShadow: Some(TuiDrawShadow),
    DrawBox: Some(TuiDrawBox),
    DrawText: Some(TuiDrawText),
    DrawText2: Some(TuiDrawText2),
    DrawCenteredText: Some(TuiDrawCenteredText),
    DrawStatusText: Some(MiniTuiDrawStatusText),
    UpdateDateTime: Some(TuiUpdateDateTime),
    MessageBox: Some(TuiMessageBox),
    MessageBoxCritical: Some(TuiMessageBoxCritical),
    DrawProgressBarCenter: Some(MiniTuiDrawProgressBarCenter),
    DrawProgressBar: Some(MiniTuiDrawProgressBar),
    SetProgressBarText: Some(MiniTuiSetProgressBarText),
    TickProgressBar: Some(MiniTuiTickProgressBar),
    EditBox: Some(TuiEditBox),
    TextToColor: Some(crate::tui::TuiTextToColor),
    TextToFillStyle: Some(TuiTextToFillStyle),
    FadeInBackdrop: Some(MiniTuiFadeInBackdrop),
    FadeOut: Some(TuiFadeOut),
    DisplayMenu: Some(TuiDisplayMenu),
    DrawMenu: Some(MiniTuiDrawMenu),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         FreeLoader
//  * FILE:            boot/freeldr/freeldr/ui/minitui.c
//  * PURPOSE:         Mini Text UI interface
//  * PROGRAMMERS:     Brian Palmer <brianp@sginet.com>
//  *                  Hervé Poussineau
//  */
//
// #include <freeldr.h>
//
// /* NTLDR or Vista+ BOOTMGR progress-bar style */
// // #define NTLDR_PROGRESSBAR
// // #define BTMGR_PROGRESSBAR /* Default style */
//
// BOOLEAN MiniTuiInitialize(VOID)
// {
//     /* Initialize main TUI */
//     if (!TuiInitialize())
//         return FALSE;
//
//     /* Override default settings with "Mini" TUI Theme */
//
//     UiTextColor = TuiTextToColor("Default");
//
//     UiStatusBarFgColor    = UiTextColor;
//     UiStatusBarBgColor    = COLOR_BLACK;
//     UiBackdropFgColor     = UiTextColor;
//     UiBackdropBgColor     = COLOR_BLACK;
//     UiBackdropFillStyle   = ' '; // TuiTextToFillStyle("None");
//     UiTitleBoxFgColor     = COLOR_WHITE;
//     UiTitleBoxBgColor     = COLOR_BLACK;
//     // UiMessageBoxFgColor   = COLOR_WHITE;
//     // UiMessageBoxBgColor   = COLOR_BLUE;
//     UiMenuFgColor         = UiTextColor;
//     UiMenuBgColor         = COLOR_BLACK;
//     UiSelectedTextColor   = COLOR_BLACK;
//     UiSelectedTextBgColor = UiTextColor;
//     // UiEditBoxTextColor    = COLOR_WHITE;
//     // UiEditBoxBgColor      = COLOR_BLACK;
//
//     UiShowTime          = FALSE;
//     UiMenuBox           = FALSE;
//     UiCenterMenu        = FALSE;
//     UiUseSpecialEffects = FALSE;
//
//     // TODO: Have a boolean to show/hide title box?
//     UiTitleBoxTitleText[0] = ANSI_NULL;
//
//     RtlStringCbCopyA(UiTimeText, sizeof(UiTimeText),
//                      "Seconds until highlighted choice will be started automatically:");
//
//     return TRUE;
// }
//
// VOID MiniTuiDrawBackdrop(ULONG DrawHeight)
// {
//     /* Fill in a black background */
//     TuiFillArea(0, 0, UiScreenWidth - 1, DrawHeight - 1,
//                 UiBackdropFillStyle,
//                 ATTR(UiBackdropFgColor, UiBackdropBgColor));
//
//     /* Update the screen buffer */
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// VOID MiniTuiFadeInBackdrop(VOID)
// {
//     /* No fade-in effect in MiniTui */
//     MiniTuiDrawBackdrop(UiScreenHeight);
// }
//
// VOID MiniTuiDrawStatusText(PCSTR StatusText)
// {
//     /* Minimal UI doesn't have a status bar */
// }
//
// /*static*/ VOID
// MiniTuiSetProgressBarText(
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
//                 UiProgressBar.Right,
// #ifdef NTLDR_PROGRESSBAR
//                 UiProgressBar.Bottom - 1,
// #else // BTMGR_PROGRESSBAR
//                 UiProgressBar.Bottom - 2, // One empty line between text and bar.
// #endif
//                 ' ', ATTR(UiTextColor, UiMenuBgColor));
//
//     /* Draw the "Loading..." text */
//     TuiDrawCenteredText(UiProgressBar.Left, UiProgressBar.Top,
//                         UiProgressBar.Right,
// #ifdef NTLDR_PROGRESSBAR
//                         UiProgressBar.Bottom - 1,
// #else // BTMGR_PROGRESSBAR
//                         UiProgressBar.Bottom - 2, // One empty line between text and bar.
// #endif
//                         ProgressString, ATTR(UiTextColor, UiMenuBgColor));
// }
//
// /*static*/ VOID
// MiniTuiTickProgressBar(
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
//     /* Fill the remaining with blanks */
//     TuiFillArea(UiProgressBar.Left + FillCount, UiProgressBar.Bottom,
//                 UiProgressBar.Right, UiProgressBar.Bottom,
//                 ' ', ATTR(UiTextColor, UiMenuBgColor));
//
//     TuiUpdateDateTime();
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// VOID
// MiniTuiDrawProgressBarCenter(
//     _In_ PCSTR ProgressText)
// {
//     ULONG Left, Top, Right, Bottom, Width, Height;
//
//     /* Build the coordinates and sizes */
// #ifdef NTLDR_PROGRESSBAR
//     Height = 2;
//     Width  = UiScreenWidth;
//     Left = 0;
//     Top  = UiScreenHeight - Height - 2;
// #else // BTMGR_PROGRESSBAR
//     Height = 3;
//     Width  = UiScreenWidth - 4;
//     Left = 2;
//     Top  = UiScreenHeight - Height - 3;
// #endif
//     Right  = Left + Width - 1;
//     Bottom = Top + Height - 1;
//
//     /* Draw the progress bar */
//     MiniTuiDrawProgressBar(Left, Top, Right, Bottom, ProgressText);
// }
//
// VOID
// MiniTuiDrawProgressBar(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR ProgressText)
// {
//     UiInitProgressBar(Left, Top, Right, Bottom, ProgressText);
// }
//
// VOID
// MiniTuiDrawMenu(
//     _In_ PUI_MENU_INFO MenuInfo)
// {
//     ULONG i;
//
//     /* Draw the backdrop */
//     UiDrawBackdrop(UiGetScreenHeight());
//
//     /* No GUI status bar text, just minimal text. Show the menu header. */
//     if (MenuInfo->MenuHeader)
//     {
//         UiVtbl.DrawText(0,
//                         MenuInfo->Top - 2,
//                         MenuInfo->MenuHeader,
//                         ATTR(UiMenuFgColor, UiMenuBgColor));
//     }
//
//     /* Draw the menu box */
//     TuiDrawMenuBox(MenuInfo);
//
//     /* Draw each line of the menu */
//     for (i = 0; i < MenuInfo->MenuItemCount; ++i)
//     {
//         TuiDrawMenuItem(MenuInfo, i);
//     }
//
//     /* Now tell the user how to choose */
//     UiVtbl.DrawText(0,
//                     MenuInfo->Bottom + 1,
//                     "Use \x18 and \x19 to move the highlight to your choice.",
//                     ATTR(UiMenuFgColor, UiMenuBgColor));
//     UiVtbl.DrawText(0,
//                     MenuInfo->Bottom + 2,
//                     "Press ENTER to choose.",
//                     ATTR(UiMenuFgColor, UiMenuBgColor));
//
//     /* And show the menu footer */
//     if (MenuInfo->MenuFooter)
//     {
//         UiVtbl.DrawText(0,
//                         UiScreenHeight - 4,
//                         MenuInfo->MenuFooter,
//                         ATTR(UiMenuFgColor, UiMenuBgColor));
//     }
//
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// const UIVTBL MiniTuiVtbl =
// {
//     MiniTuiInitialize,
//     TuiUnInitialize,
//     MiniTuiDrawBackdrop,
//     TuiFillArea,
//     TuiDrawShadow,
//     TuiDrawBox,
//     TuiDrawText,
//     TuiDrawText2,
//     TuiDrawCenteredText,
//     MiniTuiDrawStatusText,
//     TuiUpdateDateTime,
//     TuiMessageBox,
//     TuiMessageBoxCritical,
//     MiniTuiDrawProgressBarCenter,
//     MiniTuiDrawProgressBar,
//     MiniTuiSetProgressBarText,
//     MiniTuiTickProgressBar,
//     TuiEditBox,
//     TuiTextToColor,
//     TuiTextToFillStyle,
//     MiniTuiFadeInBackdrop,
//     TuiFadeOut,
//     TuiDisplayMenu,
//     MiniTuiDrawMenu,
// };
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
