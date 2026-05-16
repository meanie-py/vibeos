use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;

extern "C" {
    fn ATTR(fg: UCHAR, bg: UCHAR) -> UCHAR;
    fn TuiFillArea( Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, FillChar: UCHAR, Attr: UCHAR, );
    fn VideoCopyOffScreenBufferToVRAM();
}
extern "system" {
    fn TuiInitialize() -> BOOLEAN;
    fn TuiTextToColor(text: *const u8) -> UCHAR;
    fn UiInitProgressBar( Left: u32, Top: u32, Right: u32, Bottom: u32, ProgressText: PCSTR, );
}

pub unsafe extern "system" fn MiniTuiInitialize() -> BOOLEAN {

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

    *UiTitleBoxTitleText.as_mut_ptr() = 0u8;

    RtlStringCbCopyA(
        UiTimeText.as_mut_ptr(),
        core::mem::size_of_val(&UiTimeText),
        b"Seconds until highlighted choice will be started automatically:\0".as_ptr(),
    );

    1u8
}

unsafe fn MiniTuiDrawBackdrop(DrawHeight: ULONG) {

    TuiFillArea(
        0,
        0,
        UiScreenWidth - 1,
        DrawHeight - 1,
        UiBackdropFillStyle,
        ATTR(UiBackdropFgColor, UiBackdropBgColor),
    );

    VideoCopyOffScreenBufferToVRAM();
}

unsafe fn MiniTuiFadeInBackdrop() {
    MiniTuiDrawBackdrop(UiScreenHeight);
}

unsafe fn MiniTuiDrawStatusText(StatusText: PCSTR) {
    /* Minimal UI doesn't have a status bar */
}

pub unsafe extern "C" fn MiniTuiSetProgressBarText(
    ProgressText: PCSTR
) {
    let mut ProgressBarWidth: ULONG;
    let mut ProgressString: [u8; 256] = [0; 256];

    ProgressBarWidth = (*core::ptr::addr_of!(UiProgressBar)).Right - (*core::ptr::addr_of!(UiProgressBar)).Left + 1;

    RtlStringCbCopyA(ProgressString.as_mut_ptr(), core::mem::size_of_val(&ProgressString), ProgressText as *const u8);
    TuiTruncateStringEllipsis(ProgressString.as_mut_ptr(), ProgressBarWidth);

    TuiFillArea((*core::ptr::addr_of!(UiProgressBar)).Left, (*core::ptr::addr_of!(UiProgressBar)).Top,
                (*core::ptr::addr_of!(UiProgressBar)).Right,
                #[cfg(feature = "NTLDR_PROGRESSBAR")]
                (*core::ptr::addr_of!(UiProgressBar)).Bottom - 1,
                #[cfg(not(feature = "NTLDR_PROGRESSBAR"))]
                (*core::ptr::addr_of!(UiProgressBar)).Bottom - 2,
                b' ' as i8, ATTR(UiTextColor, UiMenuBgColor));

    TuiDrawCenteredText((*core::ptr::addr_of!(UiProgressBar)).Left, (*core::ptr::addr_of!(UiProgressBar)).Top,
                        (*core::ptr::addr_of!(UiProgressBar)).Right,
                        #[cfg(feature = "NTLDR_PROGRESSBAR")]
                        (*core::ptr::addr_of!(UiProgressBar)).Bottom - 1,
                        #[cfg(not(feature = "NTLDR_PROGRESSBAR"))]
                        (*core::ptr::addr_of!(UiProgressBar)).Bottom - 2,
                        ProgressString.as_mut_ptr(), ATTR(UiTextColor, UiMenuBgColor));
}

#[no_mangle]
pub unsafe extern "system" fn MiniTuiTickProgressBar(SubPercentTimes100: ULONG) {
    let mut ProgressBarWidth: ULONG;
    let mut FillCount: ULONG;

    ProgressBarWidth = UiProgressBar.Right - UiProgressBar.Left + 1;

    FillCount = ProgressBarWidth * SubPercentTimes100 / (100 * 100);

    if FillCount > 0 {
        TuiFillArea(
            UiProgressBar.Left,
            UiProgressBar.Bottom,
            UiProgressBar.Left + FillCount - 1,
            UiProgressBar.Bottom,
            0xDB,
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

pub unsafe fn MiniTuiDrawProgressBarCenter(
    ProgressText: PCSTR,
) {
    let Left: u32;
    let Top: u32;
    let Right: u32;
    let Bottom: u32;
    let Width: u32;
    let Height: u32;

    #[cfg(feature = "NTLDR_PROGRESSBAR")]
    {
        Height = 2;
        Width = UiScreenWidth;
        Left = 0;
        Top = UiScreenHeight - Height - 2;
    }

    #[cfg(not(feature = "NTLDR_PROGRESSBAR"))]
    {
        Height = 3;
        Width = UiScreenWidth - 4;
        Left = 2;
        Top = UiScreenHeight - Height - 3;
    }

    Right = Left + Width - 1;
    Bottom = Top + Height - 1;

    MiniTuiDrawProgressBar(Left, Top, Right, Bottom, ProgressText);
}

unsafe fn MiniTuiDrawProgressBar(
    Left: u32,
    Top: u32,
    Right: u32,
    Bottom: u32,
    ProgressText: PCSTR,
) {
    UiInitProgressBar(Left, Top, Right, Bottom, ProgressText);
}

unsafe fn MiniTuiDrawMenu(MenuInfo: PUI_MENU_INFO) {
    let mut i: ULONG;

    UiDrawBackdrop(UiGetScreenHeight());

    if !(*MenuInfo).MenuHeader.is_null() {
        ((*UiVtbl).DrawText)(
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

    ((*UiVtbl).DrawText)(
        0,
        (*MenuInfo).Bottom + 1,
        b"Use \x18 and \x19 to move the highlight to your choice.\0".as_ptr() as PCHAR,
        ATTR(UiMenuFgColor, UiMenuBgColor),
    );
    ((*UiVtbl).DrawText)(
        0,
        (*MenuInfo).Bottom + 2,
        b"Press ENTER to choose.\0".as_ptr() as PCHAR,
        ATTR(UiMenuFgColor, UiMenuBgColor),
    );

    if !(*MenuInfo).MenuFooter.is_null() {
        ((*UiVtbl).DrawText)(
            0,
            UiScreenHeight - 4,
            (*MenuInfo).MenuFooter,
            ATTR(UiMenuFgColor, UiMenuBgColor),
        );
    }

    VideoCopyOffScreenBufferToVRAM();
}


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
