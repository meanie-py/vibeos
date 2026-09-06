use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;
use crate::oslist::OperatingSystemItem;

extern "C" {
    fn DbgPrint(fmt: *const i8, ...) -> i32;
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
    fn strcat(dest: *mut i8, src: *const i8) -> *mut i8;
    // ui.h: VOID UiDrawText(ULONG X, ULONG Y, PCSTR Text, UCHAR Attr);
    fn UiDrawText(x: ULONG, y: ULONG, text: PCSTR, attr: UCHAR);
    // ui.rs / noui.rs co-located fns — declared `pub extern "C"`.
    fn UiDisplayMenu(
        MenuHeader: PCSTR,
        MenuFooter: PCSTR,
        MenuItemList: *const PCSTR,
        MenuItemCount: ULONG,
        DefaultMenuItem: ULONG,
        MenuTimeOut: LONG,
        SelectedMenuItem: PULONG,
        CanEscape: BOOLEAN,
        KeyPressFilter: *mut c_void,
        Context: PVOID,
    ) -> BOOLEAN;
    fn UiDrawBackdrop(DrawHeight: ULONG);
    fn UiEditBox(MessageText: PCSTR, EditTextBuffer: PCHAR, Length: ULONG) -> BOOLEAN;
    fn UiGetScreenHeight() -> ULONG;
    fn UiGetMenuBgColor() -> UCHAR;
    // PCHAR, not PCSTR: the C splits this buffer in place with NULs.
    fn DbgParseDebugChannels(Channels: PCHAR);
}

// ATTR(fg,bg) is the (UCHAR)(((bg)<<4)|((fg)&0xF)) macro from ui.h. Translate
// to a const fn for typecheck purity.
#[inline(always)]
const fn ATTR(fg: UCHAR, bg: UCHAR) -> UCHAR {
    (bg << 4) | (fg & 0xF)
}

// ---- file-scope preamble (translated) ----
// Raw `*const i8` is !Sync; wrap so each pub static can sit in static storage.
#[repr(transparent)]
pub struct PcstrSync(pub PCSTR);
unsafe impl Sync for PcstrSync {}
#[repr(transparent)]
pub struct PcstrArraySync<const N: usize>(pub [PCSTR; N]);
unsafe impl<const N: usize> Sync for PcstrArraySync<N> {}

pub static OptionsMenuList: PcstrArraySync<16> = PcstrArraySync([
    b"Safe Mode\0".as_ptr() as PCSTR,
    b"Safe Mode with Networking\0".as_ptr() as PCSTR,
    b"Safe Mode with Command Prompt\0".as_ptr() as PCSTR,

    core::ptr::null(),

    b"Enable Boot Logging\0".as_ptr() as PCSTR,
    b"Enable VGA Mode\0".as_ptr() as PCSTR,
    b"Last Known Good Configuration\0".as_ptr() as PCSTR,
    b"Directory Services Restore Mode\0".as_ptr() as PCSTR,
    b"Debugging Mode\0".as_ptr() as PCSTR,
    b"FreeLdr debugging\0".as_ptr() as PCSTR,

    core::ptr::null(),

    b"Start ReactOS normally\0".as_ptr() as PCSTR,
    b"Edit Boot Command Line (F10)\0".as_ptr() as PCSTR,
    b"Custom Boot\0".as_ptr() as PCSTR,
    b"Reboot\0".as_ptr() as PCSTR,
    core::ptr::null(),
]);

pub static FrldrDbgMsg: PcstrSync = PcstrSync(b"Enable FreeLdr debug channels\nAcceptable syntax: [level1]#channel1[,[level2]#channel2]\nlevel can be one of: trace,warn,fixme,err\n  if the level is omitted all levels\n  are enabled for the specified channel\n# can be either + or -\nchannel can be one of the following:\n  all,warning,memory,filesystem,inifile,ui,disk,cache,registry,\n  reactos,linux,hwdetect,windows,peloader,scsiport,heap\nExamples:\n  trace+windows,trace+reactos\n  +hwdetect,err-disk\n  +peloader\nNOTE: all letters must be lowercase, no spaces allowed.\0".as_ptr() as PCSTR);

pub const NO_OPTION: u32 = 0;
pub const SAFE_MODE: u32 = 1;
pub const SAFE_MODE_WITH_NETWORKING: u32 = 2;
pub const SAFE_MODE_WITH_COMMAND_PROMPT: u32 = 3;
pub const LAST_KNOWN_GOOD_CONFIGURATION: u32 = 4;
pub const DIRECTORY_SERVICES_RESTORE_MODE: u32 = 5;

static mut BootOptionChoice: u32 = NO_OPTION;
static mut BootLogging: u8 = 0u8;
static mut VgaMode: u8 = 0u8;
static mut DebuggingMode: u8 = 0u8;

#[no_mangle]
pub unsafe extern "C" fn DoOptionsMenu(OperatingSystem: *mut OperatingSystemItem) {
    let mut SelectedMenuItem: ULONG = 0;
    let mut DebugChannelString: [CHAR; 100] = [0; 100];

    if UiDisplayMenu(
        b"Select an option:\0".as_ptr() as PCSTR,
        core::ptr::null(),
        OptionsMenuList.0.as_ptr(),
        (core::mem::size_of_val(&OptionsMenuList.0) / core::mem::size_of::<PCSTR>()) as ULONG,
        11,
        -1 as LONG,
        &mut SelectedMenuItem,
        1u8,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        return;
    }

    UiDrawBackdrop(UiGetScreenHeight());

    match SelectedMenuItem {
        0 => {
            BootOptionChoice = SAFE_MODE;
            BootLogging = 1u8;
        }
        1 => {
            BootOptionChoice = SAFE_MODE_WITH_NETWORKING;
            BootLogging = 1u8;
        }
        2 => {
            BootOptionChoice = SAFE_MODE_WITH_COMMAND_PROMPT;
            BootLogging = 1u8;
        }
        4 => {
            BootLogging = 1u8;
        }
        5 => {
            VgaMode = 1u8;
        }
        6 => {
            BootOptionChoice = LAST_KNOWN_GOOD_CONFIGURATION;
        }
        7 => {
            BootOptionChoice = DIRECTORY_SERVICES_RESTORE_MODE;
        }
        8 => {
            DebuggingMode = 1u8;
        }
        9 => {
            DebugChannelString[0] = 0;
            if UiEditBox(
                FrldrDbgMsg.0,
                DebugChannelString.as_mut_ptr(),
                (core::mem::size_of_val(&DebugChannelString) / core::mem::size_of::<CHAR>()) as ULONG,
            ) != 0
            {
                DbgParseDebugChannels(DebugChannelString.as_mut_ptr());
            }
        }
        11 => {
            BootOptionChoice = NO_OPTION;
            BootLogging = 0u8;
            VgaMode = 0u8;
            DebuggingMode = 0u8;
        }
        #[cfg(HAS_OPTION_MENU_EDIT_CMDLINE)]
        12 => {
            EditOperatingSystemEntry(OperatingSystem);
        }
        #[cfg(HAS_OPTION_MENU_CUSTOM_BOOT)]
        13 => {
            OptionMenuCustomBoot();
        }
        #[cfg(HAS_OPTION_MENU_REBOOT)]
        14 => {
            OptionMenuReboot();
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn DisplayBootTimeOptions() {

    const COLOR_LIGHTBLUE: i32 = 9;
    const ANSI_NULL: i8 = 0;

    let mut BootOptions: [i8; 260] = [0; 260];

    match BootOptionChoice {
        1 => {
            strcpy(BootOptions.as_mut_ptr(), OptionsMenuList.0[0]);
        }
        2 => {
            strcpy(BootOptions.as_mut_ptr(), OptionsMenuList.0[1]);
        }
        3 => {
            strcpy(BootOptions.as_mut_ptr(), OptionsMenuList.0[2]);
        }
        4 => {
            strcpy(BootOptions.as_mut_ptr(), OptionsMenuList.0[6]);
        }
        5 => {
            strcpy(BootOptions.as_mut_ptr(), OptionsMenuList.0[7]);
        }
        _ => {
            BootOptions[0] = ANSI_NULL;
        }
    }

    if BootLogging != 0 {
        if BootOptionChoice != 1 && BootOptionChoice != 2 && BootOptionChoice != 3 {
            if BootOptions[0] != ANSI_NULL {
                strcat(BootOptions.as_mut_ptr(), b", \0".as_ptr() as *const i8);
            }
            strcat(BootOptions.as_mut_ptr(), OptionsMenuList.0[4]);
        }
    }

    if VgaMode != 0 {
        if BootOptions[0] != ANSI_NULL {
            strcat(BootOptions.as_mut_ptr(), b", \0".as_ptr() as *const i8);
        }
        strcat(BootOptions.as_mut_ptr(), OptionsMenuList.0[5]);
    }

    if DebuggingMode != 0 {
        if BootOptions[0] != ANSI_NULL {
            strcat(BootOptions.as_mut_ptr(), b", \0".as_ptr() as *const i8);
        }
        strcat(BootOptions.as_mut_ptr(), OptionsMenuList.0[8]);
    }

    UiDrawText(
        0,
        UiGetScreenHeight() - 2,
        BootOptions.as_ptr() as PCSTR,
        ATTR(COLOR_LIGHTBLUE as UCHAR, UiGetMenuBgColor()),
    );
}

#[no_mangle]
pub unsafe extern "C" fn AppendBootTimeOptions(BootOptions: PCHAR) {
    match BootOptionChoice {
        SAFE_MODE => {
            strcat(BootOptions, b" /SAFEBOOT:MINIMAL /SOS /NOGUIBOOT\0".as_ptr() as PCHAR);
        }

        SAFE_MODE_WITH_NETWORKING => {
            strcat(BootOptions, b" /SAFEBOOT:NETWORK /SOS /NOGUIBOOT\0".as_ptr() as PCHAR);
        }

        SAFE_MODE_WITH_COMMAND_PROMPT => {
            strcat(BootOptions, b" /SAFEBOOT:MINIMAL(ALTERNATESHELL) /SOS /NOGUIBOOT\0".as_ptr() as PCHAR);
        }

        LAST_KNOWN_GOOD_CONFIGURATION => {
            DbgPrint(b"Last known good configuration is not supported yet!\n\0".as_ptr() as *const i8);
        }

        DIRECTORY_SERVICES_RESTORE_MODE => {
            strcat(BootOptions, b" /SAFEBOOT:DSREPAIR /SOS\0".as_ptr() as PCHAR);
        }

        _ => {}
    }

    if BootLogging != 0 {
        strcat(BootOptions, b" /BOOTLOG\0".as_ptr() as PCHAR);
    }

    if VgaMode != 0 {
        strcat(BootOptions, b" /BASEVIDEO\0".as_ptr() as PCHAR);
    }

    if DebuggingMode != 0 {
        strcat(BootOptions, b" /DEBUG\0".as_ptr() as PCHAR);
    }
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
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// PCSTR OptionsMenuList[] =
// {
//     "Safe Mode",
//     "Safe Mode with Networking",
//     "Safe Mode with Command Prompt",
//
//     NULL,
//
//     "Enable Boot Logging",
//     "Enable VGA Mode",
//     "Last Known Good Configuration",
//     "Directory Services Restore Mode",
//     "Debugging Mode",
//     "FreeLdr debugging",
//
//     NULL,
//
//     "Start ReactOS normally",
// #ifdef HAS_OPTION_MENU_EDIT_CMDLINE
//     "Edit Boot Command Line (F10)",
// #endif
// #ifdef HAS_OPTION_MENU_CUSTOM_BOOT
//     "Custom Boot",
// #endif
// #ifdef HAS_OPTION_MENU_REBOOT
//     "Reboot",
// #endif
// };
//
// const
// PCSTR FrldrDbgMsg = "Enable FreeLdr debug channels\n"
//                     "Acceptable syntax: [level1]#channel1[,[level2]#channel2]\n"
//                     "level can be one of: trace,warn,fixme,err\n"
//                     "  if the level is omitted all levels\n"
//                     "  are enabled for the specified channel\n"
//                     "# can be either + or -\n"
//                     "channel can be one of the following:\n"
//                     "  all,warning,memory,filesystem,inifile,ui,disk,cache,registry,\n"
//                     "  reactos,linux,hwdetect,windows,peloader,scsiport,heap\n"
//                     "Examples:\n"
//                     "  trace+windows,trace+reactos\n"
//                     "  +hwdetect,err-disk\n"
//                     "  +peloader\n"
//                     "NOTE: all letters must be lowercase, no spaces allowed.";
//
// /* The boot options are mutually exclusive */
// enum BootOption
// {
//     NO_OPTION = 0,
//
//     SAFE_MODE,
//     SAFE_MODE_WITH_NETWORKING,
//     SAFE_MODE_WITH_COMMAND_PROMPT,
//
//     LAST_KNOWN_GOOD_CONFIGURATION,
//     DIRECTORY_SERVICES_RESTORE_MODE,
// };
//
// static enum BootOption BootOptionChoice = NO_OPTION;
// static BOOLEAN BootLogging = FALSE;
// static BOOLEAN VgaMode = FALSE;
// static BOOLEAN DebuggingMode = FALSE;
//
// /* FUNCTIONS ******************************************************************/
//
// VOID DoOptionsMenu(IN OperatingSystemItem* OperatingSystem)
// {
//     ULONG SelectedMenuItem;
//     CHAR  DebugChannelString[100];
//
//     if (!UiDisplayMenu("Select an option:", NULL,
//                        OptionsMenuList,
//                        sizeof(OptionsMenuList) / sizeof(OptionsMenuList.0[0]),
//                        11, // Use "Start ReactOS normally" as default; see the switch below.
//                        -1,
//                        &SelectedMenuItem,
//                        TRUE,
//                        NULL, NULL))
//     {
//         /* The user pressed ESC */
//         return;
//     }
//
//     /* Clear the backdrop */
//     UiDrawBackdrop(UiGetScreenHeight());
//
//     switch (SelectedMenuItem)
//     {
//         case 0: // Safe Mode
//             BootOptionChoice = SAFE_MODE;
//             BootLogging = TRUE;
//             break;
//         case 1: // Safe Mode with Networking
//             BootOptionChoice = SAFE_MODE_WITH_NETWORKING;
//             BootLogging = TRUE;
//             break;
//         case 2: // Safe Mode with Command Prompt
//             BootOptionChoice = SAFE_MODE_WITH_COMMAND_PROMPT;
//             BootLogging = TRUE;
//             break;
//         // case 3: // Separator
//         //     break;
//         case 4: // Enable Boot Logging
//             BootLogging = TRUE;
//             break;
//         case 5: // Enable VGA Mode
//             VgaMode = TRUE;
//             break;
//         case 6: // Last Known Good Configuration
//             BootOptionChoice = LAST_KNOWN_GOOD_CONFIGURATION;
//             break;
//         case 7: // Directory Services Restore Mode
//             BootOptionChoice = DIRECTORY_SERVICES_RESTORE_MODE;
//             break;
//         case 8: // Debugging Mode
//             DebuggingMode = TRUE;
//             break;
//         case 9: // FreeLdr debugging
//             DebugChannelString[0] = 0;
//             if (UiEditBox(FrldrDbgMsg,
//                           DebugChannelString,
//                           sizeof(DebugChannelString) / sizeof(DebugChannelString[0])))
//             {
//                 DbgParseDebugChannels(DebugChannelString);
//             }
//             break;
//         // case 10: // Separator
//         //     break;
//         case 11: // Start ReactOS normally
//             // Reset all the parameters to their default values.
//             BootOptionChoice = NO_OPTION;
//             BootLogging = FALSE;
//             VgaMode = FALSE;
//             DebuggingMode = FALSE;
//             break;
// #ifdef HAS_OPTION_MENU_EDIT_CMDLINE
//         case 12: // Edit command line
//             EditOperatingSystemEntry(OperatingSystem);
//             break;
// #endif
// #ifdef HAS_OPTION_MENU_CUSTOM_BOOT
//         case 13: // Custom Boot
//             OptionMenuCustomBoot();
//             break;
// #endif
// #ifdef HAS_OPTION_MENU_REBOOT
//         case 14: // Reboot
//             OptionMenuReboot();
//             break;
// #endif
//     }
// }
//
// VOID DisplayBootTimeOptions(VOID)
// {
//     CHAR BootOptions[260];
//
//     switch (BootOptionChoice)
//     {
//         case SAFE_MODE:
//             strcpy(BootOptions, OptionsMenuList.0[0]);
//             break;
//
//         case SAFE_MODE_WITH_NETWORKING:
//             strcpy(BootOptions, OptionsMenuList.0[1]);
//             break;
//
//         case SAFE_MODE_WITH_COMMAND_PROMPT:
//             strcpy(BootOptions, OptionsMenuList.0[2]);
//             break;
//
//         case LAST_KNOWN_GOOD_CONFIGURATION:
//             strcpy(BootOptions, OptionsMenuList.0[6]);
//             break;
//
//         case DIRECTORY_SERVICES_RESTORE_MODE:
//             strcpy(BootOptions, OptionsMenuList.0[7]);
//             break;
//
//         default:
//             BootOptions[0] = ANSI_NULL;
//             break;
//     }
//
//     if (BootLogging)
//     {
//         if ( (BootOptionChoice != SAFE_MODE) &&
//              (BootOptionChoice != SAFE_MODE_WITH_NETWORKING) &&
//              (BootOptionChoice != SAFE_MODE_WITH_COMMAND_PROMPT) )
//         {
//             if (BootOptions[0] != ANSI_NULL)
//                 strcat(BootOptions, ", ");
//             strcat(BootOptions, OptionsMenuList.0[4]);
//         }
//     }
//
//     if (VgaMode)
//     {
//         if (BootOptions[0] != ANSI_NULL)
//             strcat(BootOptions, ", ");
//         strcat(BootOptions, OptionsMenuList.0[5]);
//     }
//
//     if (DebuggingMode)
//     {
//         if (BootOptions[0] != ANSI_NULL)
//             strcat(BootOptions, ", ");
//         strcat(BootOptions, OptionsMenuList.0[8]);
//     }
//
//     /* Display the chosen boot options */
//     UiDrawText(0,
//                UiGetScreenHeight() - 2,
//                BootOptions,
//                ATTR(COLOR_LIGHTBLUE, UiGetMenuBgColor()));
// }
//
// VOID AppendBootTimeOptions(PCHAR BootOptions)
// {
//     switch (BootOptionChoice)
//     {
//         case SAFE_MODE:
//             strcat(BootOptions, " /SAFEBOOT:MINIMAL /SOS /NOGUIBOOT");
//             break;
//
//         case SAFE_MODE_WITH_NETWORKING:
//             strcat(BootOptions, " /SAFEBOOT:NETWORK /SOS /NOGUIBOOT");
//             break;
//
//         case SAFE_MODE_WITH_COMMAND_PROMPT:
//             strcat(BootOptions, " /SAFEBOOT:MINIMAL(ALTERNATESHELL) /SOS /NOGUIBOOT");
//             break;
//
//         case LAST_KNOWN_GOOD_CONFIGURATION:
//             DbgPrint("Last known good configuration is not supported yet!\n");
//             break;
//
//         case DIRECTORY_SERVICES_RESTORE_MODE:
//             strcat(BootOptions, " /SAFEBOOT:DSREPAIR /SOS");
//             break;
//
//         default:
//             break;
//     }
//
//     if (BootLogging)
//         strcat(BootOptions, " /BOOTLOG");
//
//     if (VgaMode)
//         strcat(BootOptions, " /BASEVIDEO");
//
//     if (DebuggingMode)
//         strcat(BootOptions, " /DEBUG");
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
