use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::*;
use crate::ui::{UiStatusBarFgColor, UiStatusBarBgColor, UiBackdropFgColor, UiBackdropBgColor, UiBackdropFillStyle, UiTitleBoxFgColor, UiTitleBoxBgColor, UiMessageBoxFgColor, UiMessageBoxBgColor, UiMenuFgColor, UiMenuBgColor, UiTextColor, UiSelectedTextColor, UiSelectedTextBgColor, UiEditBoxTextColor, UiEditBoxBgColor, UiShowTime, UiMenuBox, UiCenterMenu, UiUseSpecialEffects, UiScreenWidth, UiScreenHeight, UiTitleBoxTitleText, UiTimeText, UiMonthNames, UiProgressBar, UiVtbl};
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;

extern "C" {
    fn printf(fmt: *const i8, ...) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn MachConsGetCh();
}

#[no_mangle]
pub unsafe extern "C" fn NoUiInitialize() -> u8 {
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn NoUiUnInitialize() {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawBackdrop(_DrawHeight: ULONG) {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiFillArea(
    _Left: ULONG,
    _Top: ULONG,
    _Right: ULONG,
    _Bottom: ULONG,
    _FillChar: CHAR,
    _Attr: UCHAR,
) {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawShadow(_Left: ULONG, _Top: ULONG, _Right: ULONG, _Bottom: ULONG) {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawBox(
    _Left: ULONG,
    _Top: ULONG,
    _Right: ULONG,
    _Bottom: ULONG,
    _VertStyle: UCHAR,
    _HorzStyle: UCHAR,
    _Fill: BOOLEAN,
    _Shadow: BOOLEAN,
    _Attr: UCHAR,
) {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawText(
    _X: ULONG,
    _Y: ULONG,
    Text: PCSTR,
    _Attr: UCHAR,
) {
    printf(b"%s\n\0".as_ptr() as *const i8, Text);
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawText2(
    _X: u32,
    _Y: u32,
    MaxNumChars: u32,
    Text: PCSTR,
    _Attr: u8,
) {
    let mut MaxNumChars = MaxNumChars;
    if MaxNumChars == 0 {
        MaxNumChars = strlen(Text as *const i8) as u32;
    }
    printf(b"%*s\n\0".as_ptr() as *const i8, MaxNumChars, Text);
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawCenteredText(
    _Left: u32,
    _Top: u32,
    _Right: u32,
    _Bottom: u32,
    TextString: PCSTR,
    _Attr: u8,
) {
    printf(b"%s\n\0".as_ptr() as *const i8, TextString);
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawStatusText(StatusText: PCSTR) {
    printf(b"%s\n\0".as_ptr() as *const i8, StatusText);
}

#[no_mangle]
pub unsafe extern "C" fn NoUiUpdateDateTime() {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiMessageBox(MessageText: PCSTR) {
    NoUiMessageBoxCritical(MessageText);
}

#[no_mangle]
pub unsafe extern "C" fn NoUiMessageBoxCritical(MessageText: PCSTR) {
    // noui.c:100: on UEFI (`#ifdef UEFIBOOT`, this crate's `uefi` feature) the
    // message goes to the debug serial port and the loader carries on. Waiting
    // for a key there hangs a headless machine, which has no one to press it.
    #[cfg(feature = "uefi")]
    {
        crate::frldr::dbgcon_str(b"[FLDR] ");
        crate::frldr::dbgcon_str(core::ffi::CStr::from_ptr(MessageText as *const core::ffi::c_char).to_bytes());
        crate::frldr::dbgcon_str(b"\r\n");
    }
    #[cfg(not(feature = "uefi"))]
    {
        printf((b"%s\n\0".as_ptr()) as *const i8, MessageText);
        printf((b"Press any key\n\0".as_ptr()) as *const i8);
        MachConsGetCh();
    }
}

#[no_mangle]
pub unsafe extern "C" fn NoUiSetProgressBarText(_ProgressText: PCSTR) {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiTickProgressBar(
    _SubPercentTimes100: ULONG)
{
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawProgressBarCenter(
    _ProgressText: PCSTR)
{
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawProgressBar(
    _Left: ULONG,
    _Top: ULONG,
    _Right: ULONG,
    _Bottom: ULONG,
    _ProgressText: PCSTR,
) {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiEditBox(
    _MessageText: PCSTR,
    _EditTextBuffer: PCHAR,
    _Length: ULONG,
) -> BOOLEAN {
    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn NoUiTextToColor(_ColorText: PCSTR) -> u8 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn NoUiTextToFillStyle(_FillStyleText: PCSTR) -> u8 {
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn NoUiFadeInBackdrop() {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiFadeOut() {
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDisplayMenu(
    _MenuHeader: PCSTR,
    _MenuFooter: PCSTR,
    _MenuItemList: *mut PCSTR,
    _MenuItemCount: ULONG,
    DefaultMenuItem: ULONG,
    _MenuTimeOut: LONG,
    SelectedMenuItem: PULONG,
    _CanEscape: BOOLEAN,
    _KeyPressFilter: UiMenuKeyPressFilterCallback,
    _Context: PVOID,
) -> BOOLEAN {
    *SelectedMenuItem = DefaultMenuItem;
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn NoUiDrawMenu(_MenuInfo: PUI_MENU_INFO) {
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
//     TRACE("NoUiMessageBox: %s\n", (MessageText) as usize);
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
