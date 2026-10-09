//! ORPHAN-BY-DESIGN: `ui/directui.c` is `#if 0` upstream; the freeldr
//! CMakeLists records directui.rs as deliberately not wired. 12 of its 13
//! exports (UiDisplayMenu, UiDrawBackdrop, …) are live in ui.rs; only
//! UiMessageBox is unique to this file, and nothing calls it.
//!
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{Depth, PCSTR};
use core::ffi::c_void;

extern "C" {
    fn ATTR(fg: u32, bg: u32) -> u32;
    fn MachVideoClearScreen(attr: u32);
    fn TuiDrawText2(X: u32, Y: u32, Length: u32, Text: PCSTR, Attr: u8);
    fn TuiDrawCenteredText( Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, TextString: PCSTR, Attr: UCHAR, );
    fn TuiPrintf(Format: PCSTR, ...) -> i32;
    fn MiniTuiDrawMenu(MenuInfo: PUI_MENU_INFO);
    // TEN parameters. tui.h and tuimenu.rs both say so; the extra ShowBootOptions
    // that used to sit at index 2 is UiDisplayMenu's, not TuiDisplayMenu's, and
    // forwarding it shifted every argument from MenuItemList on by one slot —
    // the callee read a u8 as its MenuItemList pointer.
    fn TuiDisplayMenu( MenuHeader: PCSTR, MenuFooter: PCSTR, MenuItemList: *const PCSTR, MenuItemCount: u32, DefaultMenuItem: u32, MenuTimeOut: i32, SelectedMenuItem: *mut u32, CanEscape: u8, KeyPressFilter: Option<UiMenuKeyPressFilterCallback>, Context: *mut core::ffi::c_void, ) -> u8;
    fn MachVideoSetDisplayMode(mode: *mut core::ffi::c_void, text: BOOLEAN);
    fn MachVideoGetDisplaySize(width: *mut ULONG, height: *mut ULONG, depth: *mut ULONG);
    fn UiGetScreenHeight() -> ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn UiInitialize(ShowUi: BOOLEAN) -> BOOLEAN {
    
    let mut Depth: ULONG = 0;

    if ShowUi == 0 {
        return 1u8;
    }

    MachVideoSetDisplayMode(core::ptr::null_mut(), 1u8);
    MachVideoGetDisplaySize(
        core::ptr::addr_of_mut!(UiScreenWidth),
        core::ptr::addr_of_mut!(UiScreenHeight),
        core::ptr::addr_of_mut!(Depth)
    );

    UiDrawBackdrop(UiGetScreenHeight());
    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn UiUnInitialize(BootText: PCSTR) {
    return;
}

#[no_mangle]
pub unsafe extern "C" fn UiDrawBackdrop(DrawHeight: ULONG) {
    
    const COLOR_WHITE: u32 = 15; // ui.h: 15 (7 is COLOR_GRAY)
    const COLOR_BLACK: u32 = 0;
    
    MachVideoClearScreen(ATTR(COLOR_WHITE, COLOR_BLACK));
}

#[no_mangle]
pub unsafe extern "C" fn UiDrawText(
    X: u32,
    Y: u32,
    Text: PCSTR,
    Attr: u8,
) {
    TuiDrawText2(X, Y, 0, Text, Attr);
}

#[no_mangle]
pub unsafe extern "C" fn UiDrawText2(
    X: u32,
    Y: u32,
    MaxNumChars: u32,
    Text: PCSTR,
    Attr: u8,
) {
    TuiDrawText2(X, Y, MaxNumChars, Text, Attr);
}

#[no_mangle]
pub unsafe extern "C" fn UiDrawCenteredText(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    TextString: PCSTR,
    Attr: UCHAR,
) {
    TuiDrawCenteredText(Left, Top, Right, Bottom, TextString, Attr);
}

#[no_mangle]
pub unsafe extern "C" fn UiDrawStatusText(StatusText: PCSTR) {
    return;
}

#[no_mangle]
pub unsafe extern "C" fn UiInfoBox(MessageText: PCSTR) {
    // `%s`, not MessageText-as-format: the text is DATA. noui.c uses
    // `printf("%s\n", …)` at every one of these sites for the same reason,
    // and a `%` in a message would otherwise consume slots nobody pushed.
    TuiPrintf(b"%s\0".as_ptr() as PCSTR, MessageText);
}

#[no_mangle]
pub unsafe extern "C" fn UiMessageBox(
    MessageText: PCSTR,
) {
    
    // `%s`, not MessageText-as-format: the text is DATA. noui.c uses
    // `printf("%s\n", …)` at every one of these sites for the same reason,
    // and a `%` in a message would otherwise consume slots nobody pushed.
    TuiPrintf(b"%s\0".as_ptr() as PCSTR, MessageText);
}

#[no_mangle]
pub unsafe extern "C" fn UiMessageBoxCritical(MessageText: PCSTR) {
    // `%s`, not MessageText-as-format: the text is DATA. noui.c uses
    // `printf("%s\n", …)` at every one of these sites for the same reason,
    // and a `%` in a message would otherwise consume slots nobody pushed.
    TuiPrintf(b"%s\0".as_ptr() as PCSTR, MessageText);
}

#[no_mangle]
pub unsafe extern "C" fn UiShowMessageBoxesInSection(
    SectionId: ULONG_PTR)
{
    return;
}

#[no_mangle]
pub unsafe extern "C" fn UiShowMessageBoxesInArgv(
    Argc: u32,
    Argv: *mut PCHAR,
) {
    return;
}

unsafe fn UiDrawMenu(
    MenuInfo: PUI_MENU_INFO)
{
    
    MiniTuiDrawMenu(MenuInfo);
}

#[no_mangle]
pub unsafe extern "C" fn UiDisplayMenu(
    MenuHeader: PCSTR,
    MenuFooter: PCSTR,
    MenuItemList: *const PCSTR,
    MenuItemCount: u32,
    DefaultMenuItem: u32,
    MenuTimeOut: i32,
    SelectedMenuItem: *mut u32,
    CanEscape: u8,
    KeyPressFilter: Option<UiMenuKeyPressFilterCallback>,
    Context: *mut core::ffi::c_void,
) -> u8 {

    let _ = ShowBootOptions; // TuiDisplayMenu does not take it (see the decl above)
    TuiDisplayMenu(
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


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Boot Loader
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            boot/freeldr/freeldr/ui/directui.c
//  * PURPOSE:         FreeLDR UI Routines
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// #if 0
// #include <freeldr.h>
//
// /* GLOBALS ********************************************************************/
//
// ULONG UiScreenWidth;
// ULONG UiScreenHeight;
// UCHAR UiMenuFgColor = COLOR_GRAY;
// UCHAR UiMenuBgColor = COLOR_BLACK;
// UCHAR UiTextColor = COLOR_GRAY;
// UCHAR UiSelectedTextColor = COLOR_BLACK;
// UCHAR UiSelectedTextBgColor = COLOR_GRAY;
// CHAR  UiTimeText[260] = "Seconds until highlighted choice will be started automatically:";
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN
// UiInitialize(IN BOOLEAN ShowUi)
// {
//     ULONG Depth;
//
//     /* Nothing to do */
//     if (!ShowUi) return TRUE;
//
//     /* Set mode and query size */
//     MachVideoSetDisplayMode(NULL, TRUE);
//     MachVideoGetDisplaySize(&UiScreenWidth, &UiScreenHeight, &Depth);
//
//     /* Clear the screen */
//     UiDrawBackdrop(UiGetScreenHeight());
//     return TRUE;
// }
//
// VOID
// UiUnInitialize(IN PCSTR BootText)
// {
//     /* Nothing to do */
//     return;
// }
//
// VOID
// UiDrawBackdrop(ULONG DrawHeight)
// {
//     /* Clear the screen */
//     MachVideoClearScreen(ATTR(COLOR_WHITE, COLOR_BLACK));
// }
//
// VOID
// UiDrawText(
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_ PCSTR Text,
//     _In_ UCHAR Attr)
// {
//     TuiDrawText2(X, Y, 0 /*(ULONG)strlen(Text)*/, Text, Attr);
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
//     TuiDrawText2(X, Y, MaxNumChars, Text, Attr);
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
//     TuiDrawCenteredText(Left, Top, Right, Bottom, TextString, Attr);
// }
//
// VOID
// UiDrawStatusText(IN PCSTR StatusText)
// {
//     return;
// }
//
// VOID
// UiInfoBox(
//     _In_ PCSTR MessageText)
// {
//     TuiPrintf(MessageText);
// }
//
// VOID
// UiMessageBox(
//     _In_ PCSTR MessageText)
// {
//     TuiPrintf(MessageText);
// }
//
// VOID
// UiMessageBoxCritical(
//     _In_ PCSTR MessageText)
// {
//     TuiPrintf(MessageText);
// }
//
// VOID
// UiShowMessageBoxesInSection(
//     IN ULONG_PTR SectionId)
// {
//     return;
// }
//
// VOID
// UiShowMessageBoxesInArgv(
//     IN ULONG Argc,
//     IN PCHAR Argv[])
// {
//     return;
// }
//
// VOID
// UiDrawMenu(
//     _In_ PUI_MENU_INFO MenuInfo)
// {
//     MiniTuiDrawMenu(MenuInfo);
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
//     return TuiDisplayMenu(MenuHeader,
//                           MenuFooter,
//                           ShowBootOptions,
//                           MenuItemList,
//                           MenuItemCount,
//                           DefaultMenuItem,
//                           MenuTimeOut,
//                           SelectedMenuItem,
//                           CanEscape,
//                           KeyPressFilter,
//                           Context);
// }
//
// #endif // _M_ARM
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
