use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{Fill, PCSTR};
use core::ffi::c_void;

extern "C" {
    fn printf(format: *const i8, ...) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn TRACE(fmt: *const u8, ...);
    fn MachConsGetCh();
}

unsafe fn NoUiInitialize() -> u8 {
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NoUiUnInitialize() {
}

unsafe fn NoUiDrawBackdrop(DrawHeight: ULONG) {
}

pub unsafe extern "C" fn NoUiFillArea(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    FillChar: CHAR,
    Attr: UCHAR,
) {
}

unsafe fn NoUiDrawShadow(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG) {
}

unsafe fn NoUiDrawBox(
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
}

unsafe fn NoUiDrawText(
    X: ULONG,
    Y: ULONG,
    Text: PCSTR,
    Attr: UCHAR,
) {
    printf(b"%s\n\0".as_ptr() as *const i8, Text);
}

unsafe fn NoUiDrawText2(
    X: u32,
    Y: u32,
    MaxNumChars: u32,
    Text: PCSTR,
    Attr: u8,
) {
    let mut MaxNumChars = MaxNumChars;
    if MaxNumChars == 0 {
        MaxNumChars = strlen(Text as *const i8) as u32;
    }
    printf(b"%*s\n\0".as_ptr() as *const i8, MaxNumChars, Text);
}

unsafe fn NoUiDrawCenteredText(
    Left: u32,
    Top: u32,
    Right: u32,
    Bottom: u32,
    TextString: PCSTR,
    Attr: u8,
) {
    printf(b"%s\n\0".as_ptr() as *const i8, TextString);
}

#[no_mangle]
pub unsafe extern "system" fn NoUiDrawStatusText(StatusText: PCSTR) {
    printf(b"%s\n\0".as_ptr() as *const i8, StatusText);
}

unsafe fn NoUiUpdateDateTime() {
}

unsafe fn NoUiMessageBox(MessageText: PCSTR) {
    NoUiMessageBoxCritical(MessageText);
}

unsafe fn NoUiMessageBoxCritical(MessageText: PCSTR) {
    #[cfg(feature = "UEFIBOOT")]
    {
        TRACE(b"NoUiMessageBox: %s\n\0".as_ptr(), MessageText);
        return;
    }
    #[cfg(not(feature = "UEFIBOOT"))]
    {
        printf(b"%s\n\0".as_ptr(), MessageText);
        printf(b"Press any key\n\0".as_ptr());
        MachConsGetCh();
    }
}

unsafe fn NoUiSetProgressBarText(ProgressText: PCSTR) {
}

unsafe fn NoUiTickProgressBar(
    SubPercentTimes100: ULONG)
{
}

unsafe fn NoUiDrawProgressBarCenter(
    ProgressText: PCSTR)
{
}

unsafe fn NoUiDrawProgressBar(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    ProgressText: PCSTR,
) {
}

#[no_mangle]
pub unsafe extern "system" fn NoUiEditBox(
    MessageText: PCSTR,
    EditTextBuffer: PCHAR,
    Length: ULONG,
) -> BOOLEAN {
    return 0u8;
}

unsafe fn NoUiTextToColor(ColorText: PCSTR) -> u8 {
    0
}

unsafe fn NoUiTextToFillStyle(FillStyleText: PCSTR) -> u8 {
    return 0;
}

unsafe fn NoUiFadeInBackdrop() {
}

unsafe fn NoUiFadeOut() {
}

#[no_mangle]
pub unsafe extern "system" fn NoUiDisplayMenu(
    MenuHeader: PCSTR,
    MenuFooter: PCSTR,
    MenuItemList: *const PCSTR,
    MenuItemCount: ULONG,
    DefaultMenuItem: ULONG,
    MenuTimeOut: LONG,
    SelectedMenuItem: PULONG,
    CanEscape: BOOLEAN,
    KeyPressFilter: UiMenuKeyPressFilterCallback,
    Context: PVOID,
) -> BOOLEAN {
    *SelectedMenuItem = DefaultMenuItem;
    1u8
}

unsafe fn NoUiDrawMenu(MenuInfo: PUI_MENU_INFO) {
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         FreeLoader
//  * FILE:            boot/freeldr/freeldr/ui/noui.c
//  * PURPOSE:         No Text UI interface
//  * PROGRAMMERS:     Hervé Poussineau
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(UI);
//
// BOOLEAN NoUiInitialize(VOID)
// {
//     return TRUE;
// }
//
// VOID NoUiUnInitialize(VOID)
// {
// }
//
// VOID NoUiDrawBackdrop(ULONG DrawHeight)
// {
// }
//
// VOID NoUiFillArea(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom, CHAR FillChar, UCHAR Attr)
// {
// }
//
// VOID NoUiDrawShadow(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom)
// {
// }
//
// VOID NoUiDrawBox(ULONG Left, ULONG Top, ULONG Right, ULONG Bottom, UCHAR VertStyle, UCHAR HorzStyle, BOOLEAN Fill, BOOLEAN Shadow, UCHAR Attr)
// {
// }
//
// VOID
// NoUiDrawText(
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_ PCSTR Text,
//     _In_ UCHAR Attr)
// {
//     printf("%s\n", Text);
// }
//
// VOID
// NoUiDrawText2(
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_opt_ ULONG MaxNumChars,
//     _In_reads_or_z_(MaxNumChars) PCSTR Text,
//     _In_ UCHAR Attr)
// {
//     if (MaxNumChars == 0)
//         MaxNumChars = (ULONG)strlen(Text);
//     printf("%*s\n", MaxNumChars, Text);
// }
//
// VOID
// NoUiDrawCenteredText(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR TextString,
//     _In_ UCHAR Attr)
// {
//     printf("%s\n", TextString);
// }
//
// VOID NoUiDrawStatusText(PCSTR StatusText)
// {
//     printf("%s\n", StatusText);
// }
//
// VOID NoUiUpdateDateTime(VOID)
// {
// }
//
// VOID
// NoUiMessageBox(
//     _In_ PCSTR MessageText)
// {
//     NoUiMessageBoxCritical(MessageText);
// }
//
// VOID
// NoUiMessageBoxCritical(
//     _In_ PCSTR MessageText)
// {
//     /*
//      * We have not yet displayed the user interface
//      * We are probably still reading the .ini file
//      * and have encountered an error. Just use printf()
//      * and return.
//      */
// #ifdef UEFIBOOT
//     /* Use debug serial output, not framebuffer printf */
//     TRACE("NoUiMessageBox: %s\n", MessageText);
//     return;
// #else
//     printf("%s\n", MessageText);
//     printf("Press any key\n");
//     MachConsGetCh();
// #endif
// }
//
// /* Loading Progress-Bar Functions ********************************************/
//
// VOID
// NoUiSetProgressBarText(
//     _In_ PCSTR ProgressText)
// {
// }
//
// VOID
// NoUiTickProgressBar(
//     _In_ ULONG SubPercentTimes100)
// {
// }
//
// VOID
// NoUiDrawProgressBarCenter(
//     _In_ PCSTR ProgressText)
// {
// }
//
// VOID
// NoUiDrawProgressBar(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ PCSTR ProgressText)
// {
// }
//
//
// BOOLEAN NoUiEditBox(PCSTR MessageText, PCHAR EditTextBuffer, ULONG Length)
// {
//     return FALSE;
// }
//
// UCHAR NoUiTextToColor(PCSTR ColorText)
// {
//     return 0;
// }
//
// UCHAR NoUiTextToFillStyle(PCSTR FillStyleText)
// {
//     return 0;
// }
//
// VOID NoUiFadeInBackdrop(VOID)
// {
// }
//
// VOID NoUiFadeOut(VOID)
// {
// }
//
// ///////////////////////////////////////////////////////////////////////////////////////
// //
// // Menu Functions
// //
// ///////////////////////////////////////////////////////////////////////////////////////
//
// BOOLEAN
// NoUiDisplayMenu(
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
//     *SelectedMenuItem = DefaultMenuItem;
//     return TRUE;
// }
//
// VOID
// NoUiDrawMenu(
//     _In_ PUI_MENU_INFO MenuInfo)
// {
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
