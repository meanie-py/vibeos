use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::*;
use crate::ui::{UiStatusBarFgColor, UiStatusBarBgColor, UiBackdropFgColor, UiBackdropBgColor, UiBackdropFillStyle, UiTitleBoxFgColor, UiTitleBoxBgColor, UiMessageBoxFgColor, UiMessageBoxBgColor, UiMenuFgColor, UiMenuBgColor, UiTextColor, UiSelectedTextColor, UiSelectedTextBgColor, UiEditBoxTextColor, UiEditBoxBgColor, UiShowTime, UiMenuBox, UiCenterMenu, UiUseSpecialEffects, UiScreenWidth, UiScreenHeight, UiTitleBoxTitleText, UiTimeText, UiMonthNames, UiProgressBar, UiVtbl};
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;

extern "C" {
    fn strlen(s: *const i8) -> usize;
    fn RtlStringCbPrintfA(pszDest: PCHAR, cbDest: usize, pszFormat: PCHAR, ...) -> NTSTATUS;
    fn UiDrawText(X: ULONG, Y: ULONG, Text: PCHAR, Attr: UCHAR);
    fn MachConsKbHit() -> BOOLEAN;
    fn MachConsGetCh() -> ULONG;
    fn VideoCopyOffScreenBufferToVRAM();
    fn UiDrawBackdrop(Height: ULONG);
    fn UiGetScreenHeight() -> ULONG;
    fn ArcGetTime() -> *mut TIMEINFO;
    fn MachHwIdle();
    fn TuiUpdateDateTime();
}

#[no_mangle]
pub unsafe extern "C" fn TuiDisplayMenu(
    MenuHeader: PCSTR,
    MenuFooter: PCSTR,
    MenuItemList: *mut PCSTR,
    MenuItemCount: ULONG,
    DefaultMenuItem: ULONG,
    mut MenuTimeOut: LONG,
    SelectedMenuItem: PULONG,
    CanEscape: BOOLEAN,
    KeyPressFilter: UiMenuKeyPressFilterCallback,
    Context: PVOID,
) -> BOOLEAN {

    let mut MenuInformation: UI_MENU_INFO = core::mem::zeroed();
    let mut LastClockSecond: ULONG;
    let mut CurrentClockSecond: ULONG;
    let mut KeyPress: ULONG;

    // Drain any pending keystrokes left over from firmware/BdsDxe (UEFI),
    // otherwise the first MachConsKbHit() inside the loop will return
    // success and TuiProcessMenuKeyboardEvent disables the timeout.
    // Bounded so a buggy MachConsKbHit (e.g. UEFI ReadKeyStroke returning
    // success without producing a real key) can't infinite-loop us here.
    let mut drain_count = 0;
    while drain_count < 32 && MachConsKbHit() != 0 {
        let _ = MachConsGetCh();
        drain_count += 1;
    }

    if MenuTimeOut == 0 && KeyPressFilter.is_some() && MachConsKbHit() != 0 {
        KeyPress = MachConsGetCh();
        if KeyPress == KEY_EXTENDED {
            KeyPress = MachConsGetCh();
        }

        if KeyPressFilter.unwrap()(KeyPress, DefaultMenuItem, Context) != 0 {
            // C-exact: REASSIGN (C `MenuTimeOut = -1`) so a filter-accepted
            // boot key at timeout 0 switches to wait-indefinitely instead of
            // instantly returning the default item (the `let` shadow
            // discarded it).
            MenuTimeOut = -1i32 as LONG;
        }
    }

    if MenuTimeOut == 0 {
        if !SelectedMenuItem.is_null() {
            *SelectedMenuItem = DefaultMenuItem;
        }
        return 1u8;
    }

    MenuInformation.MenuHeader = MenuHeader;
    MenuInformation.MenuFooter = MenuFooter;
    MenuInformation.MenuItemList = MenuItemList;
    MenuInformation.MenuItemCount = MenuItemCount;
    MenuInformation.MenuTimeRemaining = MenuTimeOut;
    MenuInformation.SelectedMenuItem = DefaultMenuItem;
    MenuInformation.Context = Context;

    TuiCalcMenuBoxSize(core::ptr::addr_of_mut!(MenuInformation));

    (UiVtbl.DrawMenu.unwrap())(core::ptr::addr_of_mut!(MenuInformation));

    LastClockSecond = ((*ArcGetTime()).Second) as u32 ;

    loop {
        KeyPress = TuiProcessMenuKeyboardEvent(core::ptr::addr_of_mut!(MenuInformation), KeyPressFilter);

        if KeyPress == KEY_ENTER {
            break;
        }
        if CanEscape != 0 && KeyPress == KEY_ESC {
            return 0u8;
        }

        CurrentClockSecond = ((*ArcGetTime()).Second) as u32 ;
        if CurrentClockSecond != LastClockSecond {
            LastClockSecond = CurrentClockSecond;

            TuiUpdateDateTime();

            if MenuInformation.MenuTimeRemaining > 0 {
                MenuInformation.MenuTimeRemaining -= 1;
                TuiDrawMenuTimeout(core::ptr::addr_of_mut!(MenuInformation));
            } else if MenuInformation.MenuTimeRemaining == 0 {
                VideoCopyOffScreenBufferToVRAM();
                break;
            }
            VideoCopyOffScreenBufferToVRAM();
        }

        MachHwIdle();
    }

    if !SelectedMenuItem.is_null() {
        *SelectedMenuItem = MenuInformation.SelectedMenuItem;
    }
    return 1u8;
}

unsafe extern "C" fn TuiCalcMenuBoxSize(MenuInfo: *mut UI_MENU_INFO) {
    let mut i: u32;
    let mut Width: u32 = 0;
    let Height: u32;
    let mut Length: u32;

    Height = (*MenuInfo).MenuItemCount + 2;

    i = 0;
    while i < (*MenuInfo).MenuItemCount {
        if !(*(*MenuInfo).MenuItemList.add(i as usize)).is_null() {
            Length = strlen(*(*MenuInfo).MenuItemList.add(i as usize) ) as u32;
            Width = max(Width, Length);
        }
        i += 1;
    }

    Width += 10;

    if UiCenterMenu != 0 {
        (*MenuInfo).Left = (UiScreenWidth - min(Width, UiScreenWidth)) / 2;
        if Height <= UiScreenHeight - TUI_TITLE_BOX_CHAR_HEIGHT - 1 {
            (*MenuInfo).Top = (UiScreenHeight + TUI_TITLE_BOX_CHAR_HEIGHT - 1 - Height) / 2;
        } else {
            (*MenuInfo).Top = (UiScreenHeight - min(Height, UiScreenHeight)) / 2;
        }
    } else {
        (*MenuInfo).Left = -1i32 as u32;
        (*MenuInfo).Top = 4;
    }

    (*MenuInfo).Right = (*MenuInfo).Left + Width - 1;
    (*MenuInfo).Bottom = (*MenuInfo).Top + Height - 1;
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawMenu(MenuInfo: PUI_MENU_INFO) {

    let mut i: ULONG;

    UiDrawBackdrop(UiGetScreenHeight());

    TuiDrawMenuBox(MenuInfo);

    i = 0;
    while i < (*MenuInfo).MenuItemCount {
        TuiDrawMenuItem(MenuInfo, i);
        i = i.wrapping_add(1);
    }

    (UiVtbl.DrawStatusText.unwrap())(b"Use \x18 and \x19 to select, then press ENTER.\0".as_ptr() as PCHAR);

    VideoCopyOffScreenBufferToVRAM();
}

unsafe extern "C" fn TuiDrawMenuTimeout(MenuInfo: PUI_MENU_INFO) {
    let Length: ULONG;
    let mut MenuLineText: [CHAR; 80] = [0; 80];

    if (*MenuInfo).MenuTimeRemaining >= 0 {
        let mut ptr: PCHAR = (UiTimeText).as_mut_ptr() as *mut i8;
        while {
            ptr = strchr(ptr, '%' as i32);
            !ptr.is_null() && *ptr.offset(1) != 'd' as i8
        } {
            ptr = ptr.offset(1);
            if *ptr != 0 {
                ptr = ptr.offset(1);
            }
        }

        if !ptr.is_null() {
            RtlStringCbPrintfA(
                MenuLineText.as_mut_ptr(),
                core::mem::size_of_val(&MenuLineText),
                b"%.*s%d%s\0".as_ptr() as PCHAR,
                ptr.offset_from((UiTimeText).as_ptr() as *const i8) as i32,
                UiTimeText.as_ptr(),
                (*MenuInfo).MenuTimeRemaining,
                ptr.offset(2),
            );
        } else {
            RtlStringCbPrintfA(
                MenuLineText.as_mut_ptr(),
                core::mem::size_of_val(&MenuLineText),
                b"%s %d\0".as_ptr() as PCHAR,
                UiTimeText.as_ptr(),
                (*MenuInfo).MenuTimeRemaining,
            );
        }

        Length = strlen(MenuLineText.as_ptr()) as ULONG;
    } else {
        Length = 0;
    }

    if UiCenterMenu != 0 {
        TuiFillArea(
            0,
            (*MenuInfo).Bottom,
            if UiMenuBox != 0 {
                (*MenuInfo).Left - 1
            } else {
                (*MenuInfo).Right
            },
            (*MenuInfo).Bottom,
            (UiBackdropFillStyle) as i8 ,
            ATTR(UiBackdropFgColor, UiBackdropBgColor),
        );

        if UiMenuBox != 0 {
            TuiDrawBoxBottomLine(
                (*MenuInfo).Left,
                (*MenuInfo).Bottom,
                (*MenuInfo).Right,
                D_VERT,
                D_HORZ,
                ATTR(UiMenuFgColor, UiMenuBgColor),
            );
        }

        if Length > 0 {
            UiDrawText(
                (*MenuInfo).Right - Length - 1,
                (*MenuInfo).Bottom,
                MenuLineText.as_ptr() as *mut i8,
                ATTR(UiMenuFgColor, UiMenuBgColor),
            );
        }
    } else {
        if Length > 0 {
            UiDrawText(
                0,
                (*MenuInfo).Bottom + 4,
                MenuLineText.as_ptr() as *mut i8,
                ATTR(UiMenuFgColor, UiMenuBgColor),
            );
        }

        TuiFillArea(
            Length,
            (*MenuInfo).Bottom + 4,
            if Length != 0 { Length + 1 } else { UiScreenWidth - 1 },
            (*MenuInfo).Bottom + 4,
            (UiBackdropFillStyle) as i8 ,
            ATTR(UiBackdropFgColor, UiBackdropBgColor),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawMenuBox(MenuInfo: PUI_MENU_INFO) {
    if UiMenuBox != 0 {
        UiDrawBox(
            (*MenuInfo).Left,
            (*MenuInfo).Top,
            (*MenuInfo).Right,
            (*MenuInfo).Bottom,
            D_VERT,
            D_HORZ,
            0u8,
            1u8,
            ATTR(UiMenuFgColor, UiMenuBgColor),
        );
    }

    TuiUpdateDateTime();
    TuiDrawMenuTimeout(MenuInfo);
}

#[no_mangle]
pub unsafe extern "C" fn TuiDrawMenuItem(
    MenuInfo: PUI_MENU_INFO,
    MenuItemNumber: ULONG,
) {
    let SpaceLeft: ULONG;
    let SpaceRight: ULONG;
    let Attribute: UCHAR;
    let mut MenuLineText: [CHAR; 80] = [0; 80];

    if (*(*MenuInfo).MenuItemList.add(MenuItemNumber as usize)).is_null() {
        if UiMenuBox != 0 {
            UiDrawText(
                (*MenuInfo).Left,
                (*MenuInfo).Top + 1 + MenuItemNumber,
                b"\xC7\0".as_ptr() as PCHAR,
                ATTR(UiMenuFgColor, UiMenuBgColor),
            );
        }

        RtlZeroMemory(MenuLineText.as_mut_ptr() as PVOID, core::mem::size_of_val(&MenuLineText));
        core::ptr::write_bytes(
            MenuLineText.as_mut_ptr() as *mut u8,
            0xC4u8,
            min(core::mem::size_of_val(&MenuLineText) as ULONG, (*MenuInfo).Right - (*MenuInfo).Left - 1) as usize,
        );

        UiDrawText(
            (*MenuInfo).Left + 1,
            (*MenuInfo).Top + 1 + MenuItemNumber,
            MenuLineText.as_mut_ptr(),
            ATTR(UiMenuFgColor, UiMenuBgColor),
        );

        if UiMenuBox != 0 {
            UiDrawText(
                (*MenuInfo).Right,
                (*MenuInfo).Top + 1 + MenuItemNumber,
                b"\xB6\0".as_ptr() as PCHAR,
                ATTR(UiMenuFgColor, UiMenuBgColor),
            );
        }

        return;
    }

    if UiCenterMenu != 0 {
        let SpaceTotal: ULONG =
            ((*MenuInfo).Right - (*MenuInfo).Left - 2) -
            strlen(*(*MenuInfo).MenuItemList.add(MenuItemNumber as usize) ) as ULONG;
        SpaceLeft = (SpaceTotal / 2) + 1;
        SpaceRight = (SpaceTotal - SpaceLeft) + 1;
    } else {
        SpaceLeft = 4;
        SpaceRight = 0;
    }

    RtlStringCbPrintfA(
        MenuLineText.as_mut_ptr(),
        core::mem::size_of_val(&MenuLineText),
        b"%*s%s%*s\0".as_ptr() as PCHAR,
        SpaceLeft,
        b"\0".as_ptr() as PCHAR,
        *(*MenuInfo).MenuItemList.add(MenuItemNumber as usize) ,
        SpaceRight,
        b"\0".as_ptr() as PCHAR,
    );

    if MenuItemNumber == (*MenuInfo).SelectedMenuItem {
        Attribute = ATTR(UiSelectedTextColor, UiSelectedTextBgColor);
    } else {
        Attribute = ATTR(UiTextColor, UiMenuBgColor);
    }

    UiDrawText(
        (*MenuInfo).Left + 1,
        (*MenuInfo).Top + 1 + MenuItemNumber,
        MenuLineText.as_mut_ptr(),
        Attribute,
    );
}

unsafe extern "C" fn TuiProcessMenuKeyboardEvent(
    MenuInfo: PUI_MENU_INFO,
    KeyPressFilter: UiMenuKeyPressFilterCallback,
) -> ULONG {

    let mut KeyEvent: ULONG = 0;
    let mut Selected: ULONG;
    let Count: ULONG;

    if MachConsKbHit() == 0 {
        return 0;
    }

    if (*MenuInfo).MenuTimeRemaining != -1 {
        (*MenuInfo).MenuTimeRemaining = -1;
        TuiDrawMenuTimeout(MenuInfo);
    }

    KeyEvent = MachConsGetCh();
    if KeyEvent == KEY_EXTENDED {
        KeyEvent = MachConsGetCh();
    }

    if !KeyPressFilter.is_none() &&
        KeyPressFilter.unwrap()(KeyEvent, (*MenuInfo).SelectedMenuItem, (*MenuInfo).Context) != 0
    {
        (UiVtbl.DrawMenu.unwrap())(MenuInfo);
        return 0;
    }

    if (KeyEvent == KEY_UP) || (KeyEvent == KEY_DOWN) ||
       (KeyEvent == KEY_HOME) || (KeyEvent == KEY_END)
    {
        Selected = (*MenuInfo).SelectedMenuItem;
        Count = (*MenuInfo).MenuItemCount - 1;

        if (KeyEvent == KEY_UP) && (Selected > 0) {
            (*MenuInfo).SelectedMenuItem -= 1;
            TuiDrawMenuItem(MenuInfo, Selected);
            Selected -= 1;

            if (Selected > 0) &&
                (*(*MenuInfo).MenuItemList.offset(Selected as isize)).is_null()
            {
                (*MenuInfo).SelectedMenuItem -= 1;
            }
        } else if ((KeyEvent == KEY_UP) && (Selected == 0)) ||
                  (KeyEvent == KEY_END)
        {
            (*MenuInfo).SelectedMenuItem = Count;
            TuiDrawMenuItem(MenuInfo, Selected);
        } else if (KeyEvent == KEY_DOWN) && (Selected < Count) {
            (*MenuInfo).SelectedMenuItem += 1;
            TuiDrawMenuItem(MenuInfo, Selected);
            Selected += 1;

            if (Selected < Count) &&
                (*(*MenuInfo).MenuItemList.offset(Selected as isize)).is_null()
            {
                (*MenuInfo).SelectedMenuItem += 1;
            }
        } else if ((KeyEvent == KEY_DOWN) && (Selected == Count)) ||
                  (KeyEvent == KEY_HOME)
        {
            (*MenuInfo).SelectedMenuItem = 0;
            TuiDrawMenuItem(MenuInfo, Selected);
        }

        TuiDrawMenuItem(MenuInfo, (*MenuInfo).SelectedMenuItem);
        VideoCopyOffScreenBufferToVRAM();
    }

    return KeyEvent;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         FreeLoader
//  * FILE:            boot/freeldr/freeldr/ui/tuimenu.c
//  * PURPOSE:         Text UI Menu Functions
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Brian Palmer (brianp@sginet.com)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <freeldr.h>
//
// /* FUNCTIONS *****************************************************************/
//
// static VOID
// TuiCalcMenuBoxSize(
//     _In_ PUI_MENU_INFO MenuInfo);
//
// static ULONG
// TuiProcessMenuKeyboardEvent(
//     _In_ PUI_MENU_INFO MenuInfo,
//     _In_ UiMenuKeyPressFilterCallback KeyPressFilter);
//
// static VOID
// TuiDrawMenuTimeout(
//     _In_ PUI_MENU_INFO MenuInfo);
//
// BOOLEAN
// TuiDisplayMenu(
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
//     UI_MENU_INFO MenuInformation;
//     ULONG LastClockSecond;
//     ULONG CurrentClockSecond;
//     ULONG KeyPress;
//
//     /*
//      * Before taking any default action if there is no timeout,
//      * check whether the supplied key filter callback function
//      * may handle a specific user keypress. If it does, the
//      * timeout is cancelled.
//      */
//     if (!MenuTimeOut && KeyPressFilter && MachConsKbHit())
//     {
//         /* Get the key (get the extended key if needed) */
//         KeyPress = MachConsGetCh();
//         if (KeyPress == KEY_EXTENDED)
//             KeyPress = MachConsGetCh();
//
//         /*
//          * Call the supplied key filter callback function to see
//          * if it is going to handle this keypress.
//          */
//         if (KeyPressFilter(KeyPress, DefaultMenuItem, Context))
//         {
//             /* It processed the key character, cancel the timeout */
//             MenuTimeOut = -1;
//         }
//     }
//
//     /* Check if there is no timeout */
//     if (!MenuTimeOut)
//     {
//         /* Return the default selected item */
//         if (SelectedMenuItem) *SelectedMenuItem = DefaultMenuItem;
//         return TRUE;
//     }
//
//     /* Setup the MENU_INFO structure */
//     MenuInformation.MenuHeader = MenuHeader;
//     MenuInformation.MenuFooter = MenuFooter;
//     MenuInformation.MenuItemList = MenuItemList;
//     MenuInformation.MenuItemCount = MenuItemCount;
//     MenuInformation.MenuTimeRemaining = MenuTimeOut;
//     MenuInformation.SelectedMenuItem = DefaultMenuItem;
//     MenuInformation.Context = Context;
//
//     /* Calculate the size of the menu box */
//     TuiCalcMenuBoxSize(&MenuInformation);
//
//     /* Draw the menu */
//     UiVtbl.DrawMenu(&MenuInformation);
//
//     /* Get the current second of time */
//     LastClockSecond = ArcGetTime()->Second;
//
//     /* Process keys */
//     while (TRUE)
//     {
//         /* Process key presses */
//         KeyPress = TuiProcessMenuKeyboardEvent(&MenuInformation, KeyPressFilter);
//
//         /* Check for ENTER or ESC */
//         if (KeyPress == KEY_ENTER) break;
//         if (CanEscape && KeyPress == KEY_ESC) return FALSE;
//
//         /* Get the updated time, and check if more than a second has elapsed */
//         CurrentClockSecond = ArcGetTime()->Second;
//         if (CurrentClockSecond != LastClockSecond)
//         {
//             /* Update the time information */
//             LastClockSecond = CurrentClockSecond;
//
//             // FIXME: Theme-specific
//             /* Update the date & time */
//             TuiUpdateDateTime();
//
//             /* If there is a countdown, update it */
//             if (MenuInformation.MenuTimeRemaining > 0)
//             {
//                 MenuInformation.MenuTimeRemaining--;
//                 TuiDrawMenuTimeout(&MenuInformation);
//             }
//             else if (MenuInformation.MenuTimeRemaining == 0)
//             {
//                 /* A timeout occurred, exit this loop and return selection */
//                 VideoCopyOffScreenBufferToVRAM();
//                 break;
//             }
//             VideoCopyOffScreenBufferToVRAM();
//         }
//
//         MachHwIdle();
//     }
//
//     /* Return the selected item */
//     if (SelectedMenuItem) *SelectedMenuItem = MenuInformation.SelectedMenuItem;
//     return TRUE;
// }
//
// static VOID
// TuiCalcMenuBoxSize(
//     _In_ PUI_MENU_INFO MenuInfo)
// {
//     ULONG i;
//     ULONG Width = 0;
//     ULONG Height;
//     ULONG Length;
//
//     /* Height is the menu item count plus 2 (top border & bottom border) */
//     Height = MenuInfo->MenuItemCount + 2;
//
//     /* Loop every item */
//     for (i = 0; i < MenuInfo->MenuItemCount; ++i)
//     {
//         /* Get the string length and make it become the new width if necessary */
//         if (MenuInfo->MenuItemList[i])
//         {
//             Length = (ULONG)strlen(MenuInfo->MenuItemList[i]);
//             Width = max(Width, Length);
//         }
//     }
//
//     /* Allow room for left & right borders, plus 4 spaces on each side */
//     Width += 10;
//
//     /* Check if we're drawing a centered menu */
//     if (UiCenterMenu)
//     {
//         /* Calculate the centered menu box area, also ensuring that the top-left
//          * corner is always visible if the borders are partly off-screen */
//         MenuInfo->Left = (UiScreenWidth - min(Width, UiScreenWidth)) / 2;
//         if (Height <= UiScreenHeight - TUI_TITLE_BOX_CHAR_HEIGHT - 1)
//         {
//             /* Exclude the header and the status bar */
//             // MenuInfo->Top = (UiScreenHeight - TUI_TITLE_BOX_CHAR_HEIGHT - 1 - Height) / 2
//             //                 + TUI_TITLE_BOX_CHAR_HEIGHT;
//             MenuInfo->Top = (UiScreenHeight + TUI_TITLE_BOX_CHAR_HEIGHT - 1 - Height) / 2;
//         }
//         else
//         {
//             MenuInfo->Top = (UiScreenHeight - min(Height, UiScreenHeight)) / 2;
//         }
//     }
//     else
//     {
//         /* Put the menu in the default left-corner position */
//         MenuInfo->Left = -1;
//         MenuInfo->Top = 4;
//     }
//
//     /* The other margins are the same */
//     MenuInfo->Right = MenuInfo->Left + Width - 1;
//     MenuInfo->Bottom = MenuInfo->Top + Height - 1;
// }
//
// VOID
// TuiDrawMenu(
//     _In_ PUI_MENU_INFO MenuInfo)
// {
//     ULONG i;
//
//     // FIXME: Theme-specific
//     /* Draw the backdrop */
//     UiDrawBackdrop(UiGetScreenHeight());
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
//     // FIXME: Theme-specific
//     /* Update the status bar */
//     UiVtbl.DrawStatusText("Use \x18 and \x19 to select, then press ENTER.");
//
//     VideoCopyOffScreenBufferToVRAM();
// }
//
// static VOID
// TuiDrawMenuTimeout(
//     _In_ PUI_MENU_INFO MenuInfo)
// {
//     ULONG Length;
//     CHAR MenuLineText[80];
//
//     /* If there is a timeout, draw the time remaining */
//     if (MenuInfo->MenuTimeRemaining >= 0)
//     {
//         /* Find whether the time text string is escaped
//          * with %d for specific countdown insertion. */
//         PCHAR ptr = UiTimeText;
//         while ((ptr = strchr(ptr, '%')) && (ptr[1] != 'd'))
//         {
//             /* Ignore any following character (including a following
//              * '%' that would be escaped), thus skip two characters.
//              * If this is the last character, ignore it and stop. */
//             if (*++ptr)
//                 ++ptr;
//         }
//         ASSERT(!ptr || (ptr[0] == '%' && ptr[1] == 'd'));
//
//         if (ptr)
//         {
//             /* Copy the time text string up to the '%d' insertion point and
//              * skip it, add the remaining time and the rest of the string. */
//             RtlStringCbPrintfA(MenuLineText, sizeof(MenuLineText),
//                                "%.*s%d%s",
//                                ptr - UiTimeText, UiTimeText,
//                                MenuInfo->MenuTimeRemaining,
//                                ptr + 2);
//         }
//         else
//         {
//             /* Copy the time text string, append a separating blank,
//              * and add the remaining time. */
//             RtlStringCbPrintfA(MenuLineText, sizeof(MenuLineText),
//                                "%s %d",
//                                UiTimeText,
//                                MenuInfo->MenuTimeRemaining);
//         }
//
//         Length = (ULONG)strlen(MenuLineText);
//     }
//     else
//     {
//         /* Erase the timeout with blanks */
//         Length = 0;
//     }
//
//     /**
//      * How to pad/fill:
//      *
//      *  Center  Box     What to do:
//      *  0       0 or 1  Pad on the right with blanks.
//      *  1       0       Pad on the left with blanks.
//      *  1       1       Pad on the left with blanks + box bottom border.
//      **/
//
//     if (UiCenterMenu)
//     {
//         /* In boxed menu mode, pad on the left with blanks and box border,
//          * otherwise, pad over all the box length until its right edge. */
//         TuiFillArea(0,
//                     MenuInfo->Bottom,
//                     UiMenuBox
//                         ? MenuInfo->Left - 1 /* Left side of the box bottom */
//                         : MenuInfo->Right,   /* Left side + all box length  */
//                     MenuInfo->Bottom,
//                     UiBackdropFillStyle,
//                     ATTR(UiBackdropFgColor, UiBackdropBgColor));
//
//         if (UiMenuBox)
//         {
//             /* Fill with box bottom border */
//             TuiDrawBoxBottomLine(MenuInfo->Left,
//                                  MenuInfo->Bottom,
//                                  MenuInfo->Right,
//                                  D_VERT,
//                                  D_HORZ,
//                                  ATTR(UiMenuFgColor, UiMenuBgColor));
//
//             /* In centered boxed menu mode, the timeout string
//              * does not go past the right border, in principle... */
//         }
//
//         if (Length > 0)
//         {
//             /* Display the timeout at the bottom-right part of the menu */
//             UiDrawText(MenuInfo->Right - Length - 1,
//                        MenuInfo->Bottom,
//                        MenuLineText,
//                        ATTR(UiMenuFgColor, UiMenuBgColor));
//         }
//     }
//     else
//     {
//         if (Length > 0)
//         {
//             /* Display the timeout under the menu directly */
//             UiDrawText(0,
//                        MenuInfo->Bottom + 4,
//                        MenuLineText,
//                        ATTR(UiMenuFgColor, UiMenuBgColor));
//         }
//
//         /* Pad on the right with blanks, to erase
//          * characters when the string length decreases. */
//         TuiFillArea(Length,
//                     MenuInfo->Bottom + 4,
//                     Length ? (Length + 1) : (UiScreenWidth - 1),
//                     MenuInfo->Bottom + 4,
//                     UiBackdropFillStyle,
//                     ATTR(UiBackdropFgColor, UiBackdropBgColor)
//                     );
//     }
// }
//
// VOID
// TuiDrawMenuBox(
//     _In_ PUI_MENU_INFO MenuInfo)
// {
//     // FIXME: Theme-specific
//     /* Draw the menu box if requested */
//     if (UiMenuBox)
//     {
//         UiDrawBox(MenuInfo->Left,
//                   MenuInfo->Top,
//                   MenuInfo->Right,
//                   MenuInfo->Bottom,
//                   D_VERT,
//                   D_HORZ,
//                   FALSE,    // Filled
//                   TRUE,     // Shadow
//                   ATTR(UiMenuFgColor, UiMenuBgColor));
//     }
//
//     /* Update the date & time */
//     TuiUpdateDateTime();
//     TuiDrawMenuTimeout(MenuInfo);
// }
//
// VOID
// TuiDrawMenuItem(
//     _In_ PUI_MENU_INFO MenuInfo,
//     _In_ ULONG MenuItemNumber)
// {
//     ULONG SpaceLeft;
//     ULONG SpaceRight;
//     UCHAR Attribute;
//     CHAR MenuLineText[80];
//
//     /* If this is a separator */
//     if (MenuInfo->MenuItemList[MenuItemNumber] == NULL)
//     {
//         // FIXME: Theme-specific
//         /* Draw its left box corner */
//         if (UiMenuBox)
//         {
//             UiDrawText(MenuInfo->Left,
//                        MenuInfo->Top + 1 + MenuItemNumber,
//                        "\xC7",
//                        ATTR(UiMenuFgColor, UiMenuBgColor));
//         }
//
//         /* Make it a separator line and use menu colors */
//         RtlZeroMemory(MenuLineText, sizeof(MenuLineText));
//         RtlFillMemory(MenuLineText,
//                       min(sizeof(MenuLineText), (MenuInfo->Right - MenuInfo->Left - 1)),
//                       0xC4);
//
//         /* Draw the item */
//         UiDrawText(MenuInfo->Left + 1,
//                    MenuInfo->Top + 1 + MenuItemNumber,
//                    MenuLineText,
//                    ATTR(UiMenuFgColor, UiMenuBgColor));
//
//         // FIXME: Theme-specific
//         /* Draw its right box corner */
//         if (UiMenuBox)
//         {
//             UiDrawText(MenuInfo->Right,
//                        MenuInfo->Top + 1 + MenuItemNumber,
//                        "\xB6",
//                        ATTR(UiMenuFgColor, UiMenuBgColor));
//         }
//
//         /* We are done */
//         return;
//     }
//
//     /* This is not a separator */
//     ASSERT(MenuInfo->MenuItemList[MenuItemNumber]);
//
//     /* Check if using centered menu */
//     if (UiCenterMenu)
//     {
//         /*
//          * We will want the string centered so calculate
//          * how many spaces will be to the left and right.
//          */
//         ULONG SpaceTotal =
//             (MenuInfo->Right - MenuInfo->Left - 2) -
//             (ULONG)strlen(MenuInfo->MenuItemList[MenuItemNumber]);
//         SpaceLeft  = (SpaceTotal / 2) + 1;
//         SpaceRight = (SpaceTotal - SpaceLeft) + 1;
//     }
//     else
//     {
//         /* Simply left-align it */
//         SpaceLeft  = 4;
//         SpaceRight = 0;
//     }
//
//     /* Format the item text string */
//     RtlStringCbPrintfA(MenuLineText, sizeof(MenuLineText),
//                        "%*s%s%*s",
//                        SpaceLeft, "",   // Left padding
//                        MenuInfo->MenuItemList[MenuItemNumber],
//                        SpaceRight, ""); // Right padding
//
//     if (MenuItemNumber == MenuInfo->SelectedMenuItem)
//     {
//         /* If this is the selected item, use the selected colors */
//         Attribute = ATTR(UiSelectedTextColor, UiSelectedTextBgColor);
//     }
//     else
//     {
//         /* Normal item colors */
//         Attribute = ATTR(UiTextColor, UiMenuBgColor);
//     }
//
//     /* Draw the item */
//     UiDrawText(MenuInfo->Left + 1,
//                MenuInfo->Top + 1 + MenuItemNumber,
//                MenuLineText,
//                Attribute);
// }
//
// static ULONG
// TuiProcessMenuKeyboardEvent(
//     _In_ PUI_MENU_INFO MenuInfo,
//     _In_ UiMenuKeyPressFilterCallback KeyPressFilter)
// {
//     ULONG KeyEvent = 0;
//     ULONG Selected, Count;
//
//     /* Check for a keypress */
//     if (!MachConsKbHit())
//         return 0; // None, bail out
//
//     /* Check if the timeout is not already complete */
//     if (MenuInfo->MenuTimeRemaining != -1)
//     {
//         /* Cancel it and remove it */
//         MenuInfo->MenuTimeRemaining = -1;
//         TuiDrawMenuTimeout(MenuInfo);
//     }
//
//     /* Get the key (get the extended key if needed) */
//     KeyEvent = MachConsGetCh();
//     if (KeyEvent == KEY_EXTENDED)
//         KeyEvent = MachConsGetCh();
//
//     /*
//      * Call the supplied key filter callback function to see
//      * if it is going to handle this keypress.
//      */
//     if (KeyPressFilter &&
//         KeyPressFilter(KeyEvent, MenuInfo->SelectedMenuItem, MenuInfo->Context))
//     {
//         /* It processed the key character, so redraw and exit */
//         UiVtbl.DrawMenu(MenuInfo);
//         return 0;
//     }
//
//     /* Process the key */
//     if ((KeyEvent == KEY_UP  ) || (KeyEvent == KEY_DOWN) ||
//         (KeyEvent == KEY_HOME) || (KeyEvent == KEY_END ))
//     {
//         /* Get the current selected item and count */
//         Selected = MenuInfo->SelectedMenuItem;
//         Count = MenuInfo->MenuItemCount - 1;
//
//         /* Check the key and change the selected menu item */
//         if ((KeyEvent == KEY_UP) && (Selected > 0))
//         {
//             /* Deselect previous item and go up */
//             MenuInfo->SelectedMenuItem--;
//             TuiDrawMenuItem(MenuInfo, Selected);
//             Selected--;
//
//             /* Skip past any separators */
//             if ((Selected > 0) &&
//                 (MenuInfo->MenuItemList[Selected] == NULL))
//             {
//                 MenuInfo->SelectedMenuItem--;
//             }
//         }
//         else if ( ((KeyEvent == KEY_UP) && (Selected == 0)) ||
//                    (KeyEvent == KEY_END) )
//         {
//             /* Go to the end */
//             MenuInfo->SelectedMenuItem = Count;
//             TuiDrawMenuItem(MenuInfo, Selected);
//         }
//         else if ((KeyEvent == KEY_DOWN) && (Selected < Count))
//         {
//             /* Deselect previous item and go down */
//             MenuInfo->SelectedMenuItem++;
//             TuiDrawMenuItem(MenuInfo, Selected);
//             Selected++;
//
//             /* Skip past any separators */
//             if ((Selected < Count) &&
//                 (MenuInfo->MenuItemList[Selected] == NULL))
//             {
//                 MenuInfo->SelectedMenuItem++;
//             }
//         }
//         else if ( ((KeyEvent == KEY_DOWN) && (Selected == Count)) ||
//                    (KeyEvent == KEY_HOME) )
//         {
//             /* Go to the beginning */
//             MenuInfo->SelectedMenuItem = 0;
//             TuiDrawMenuItem(MenuInfo, Selected);
//         }
//
//         /* Select new item and update video buffer */
//         TuiDrawMenuItem(MenuInfo, MenuInfo->SelectedMenuItem);
//         VideoCopyOffScreenBufferToVRAM();
//     }
//
//     /* Return the pressed key */
//     return KeyEvent;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
