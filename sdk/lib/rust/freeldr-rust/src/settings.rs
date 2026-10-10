use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::PCSTR;
use core::ffi::c_void;
extern "C" {
}

extern "C" {
    fn RtlStringCbCopyNA(pszDest: PCHAR, cbDest: usize, pszSrc: PCSTR, cbToCopy: usize) -> NTSTATUS;
    fn _strupr(s: PCHAR) -> PCHAR;
    fn atoi(s: PCSTR) -> i32;
    fn strcspn(s1: PCSTR, s2: PCSTR) -> usize;
    fn strstr(s1: PCSTR, s2: PCSTR) -> PCHAR;
    fn strtoul(nptr: PCSTR, endptr: *mut PCHAR, base: i32) -> u32;
    fn strtoull(nptr: PCSTR, endptr: *mut PCHAR, base: i32) -> u64;
    // Canonical freeldr inifile signatures (match inifile.rs/inifile.h):
    //   PLIST_ENTRY IniGetFileSectionListHead(VOID);
    //   BOOLEAN     IniOpenSection(PCSTR, ULONG_PTR*);
    //   BOOLEAN     IniReadSettingByName(ULONG_PTR, PCSTR, PCHAR, ULONG);
    // ULONG_PTR is layout-compatible with `*mut c_void` on all targets.
    fn IniGetFileSectionListHead() -> PLIST_ENTRY;
    fn IniOpenSection(Section: PCSTR, SectionOut: *mut *mut c_void) -> BOOLEAN;
    fn IniReadSettingByName(Section: *mut c_void, Name: PCSTR, Buffer: PCHAR, BufferSize: ULONG) -> BOOLEAN;
    fn UiMessageBoxCritical(MessageText: PCSTR);
    // ramdisk.rs owns these (#[no_mangle] pub static mut).
    static mut gInitRamDiskBase: PVOID;
    static mut gInitRamDiskSize: ULONG;
}

// ---- file-scope preamble (translated) ----
// settings.c: static CCHAR DebugString[256], static CCHAR VideoOptions[128],
// static CCHAR DefaultOs[256], BOOTMGRINFO BootMgrInfo = {NULL,NULL,NULL,-1,0};
// BootMgrInfo is the public global (extern in settings.h) → #[no_mangle].
// The three CCHAR buffers are file-static; not exported. Keep `pub static mut`
// so the Rust SettingsRsModule can address them, but no #[no_mangle].
pub static mut DebugString: [CCHAR; 256] = [0; 256];
pub static mut VideoOptions: [CCHAR; 128] = [0; 128];
pub static mut DefaultOs: [CCHAR; 256] = [0; 256];
#[no_mangle]
pub static mut BootMgrInfo: BOOTMGRINFO = BOOTMGRINFO {
    DebugString: core::ptr::null(),
    VideoOptions: core::ptr::null(),
    DefaultOs: core::ptr::null(),
    TimeOut: -1,
    FrLdrSection: core::ptr::null_mut(),
};

unsafe fn CmdLineParse(CmdLine: PCSTR) {
    let mut Setting: PCHAR;
    let mut Length: usize;
    let mut Offset: ULONG_PTR = 0;

    Setting = strstr(CmdLine, b"debug=\0".as_ptr() as PCSTR);
    if !Setting.is_null() {
        Setting = Setting.offset((core::mem::size_of::<[u8; 7]>() - core::mem::size_of::<u8>()) as isize);
        Length = strcspn(Setting as PCSTR, b" \t\0".as_ptr() as PCSTR);

        RtlStringCbCopyNA(
            DebugString.as_mut_ptr(),
            core::mem::size_of_val(&DebugString),
            Setting as PCSTR,
            Length,
        );
        _strupr(DebugString.as_mut_ptr());

        Setting = DebugString.as_mut_ptr();
        while *Setting != 0 {
            if *Setting == b';' as i8 {
                *Setting = b' ' as i8;
            }
            Setting = Setting.offset(1);
        }

        BootMgrInfo.DebugString = DebugString.as_ptr() as PCSTR;
    }

    Setting = strstr(CmdLine, b"video=\0".as_ptr() as PCSTR);
    if !Setting.is_null() {
        Setting = Setting.offset((core::mem::size_of::<[u8; 7]>() - core::mem::size_of::<u8>()) as isize);
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(not(any(feature = "sarch_xbox", feature = "sarch_pc98")))]
        if Setting.is_null() {
            Setting = strstr(CmdLine, b"vga=\0".as_ptr() as PCSTR);
            if !Setting.is_null() {
                Setting = Setting.offset((core::mem::size_of::<[u8; 5]>() - core::mem::size_of::<u8>()) as isize);
            }
        }
    }
    if !Setting.is_null() {
        Length = strcspn(Setting as PCSTR, b" \t\0".as_ptr() as PCSTR);

        RtlStringCbCopyNA(
            VideoOptions.as_mut_ptr(),
            core::mem::size_of_val(&VideoOptions),
            Setting as PCSTR,
            Length,
        );
        BootMgrInfo.VideoOptions = VideoOptions.as_ptr() as PCSTR;
    }

    Setting = strstr(CmdLine, b"timeout=\0".as_ptr() as PCSTR);
    if !Setting.is_null() {
        BootMgrInfo.TimeOut = atoi(
            Setting.offset((core::mem::size_of::<[u8; 9]>() - core::mem::size_of::<u8>()) as isize) as PCSTR
        );
    }

    Setting = strstr(CmdLine, b"defaultos=\0".as_ptr() as PCSTR);
    if !Setting.is_null() {
        Setting = Setting.offset((core::mem::size_of::<[u8; 11]>() - core::mem::size_of::<u8>()) as isize);
        Length = strcspn(Setting as PCSTR, b" \t\0".as_ptr() as PCSTR);

        RtlStringCbCopyNA(
            DefaultOs.as_mut_ptr(),
            core::mem::size_of_val(&DefaultOs),
            Setting as PCSTR,
            Length,
        );
        BootMgrInfo.DefaultOs = DefaultOs.as_ptr() as PCSTR;
    }

    Setting = strstr(CmdLine, b"rdbase=\0".as_ptr() as PCSTR);
    if !Setting.is_null() {
        gInitRamDiskBase = strtoull(
            Setting.offset((core::mem::size_of::<[u8; 8]>() - core::mem::size_of::<u8>()) as isize) as PCSTR,
            core::ptr::null_mut(),
            0,
        ) as ULONG_PTR as PVOID;
    }

    Setting = strstr(CmdLine, b"rdsize=\0".as_ptr() as PCSTR);
    if !Setting.is_null() {
        gInitRamDiskSize = strtoul(
            Setting.offset((core::mem::size_of::<[u8; 8]>() - core::mem::size_of::<u8>()) as isize) as PCSTR,
            core::ptr::null_mut(),
            0,
        );
    }

    Setting = strstr(CmdLine, b"rdoffset=\0".as_ptr() as PCSTR);
    if !Setting.is_null() {
        Offset = strtoul(
            Setting.offset((core::mem::size_of::<[u8; 10]>() - core::mem::size_of::<u8>()) as isize) as PCSTR,
            core::ptr::null_mut(),
            0,
        ) as ULONG_PTR;
    }

    gInitRamDiskBase = (gInitRamDiskBase as ULONG_PTR + Offset) as PVOID;
}

#[no_mangle]
pub unsafe extern "C" fn LoadSettings(CmdLine: PCSTR) {

    if !CmdLine.is_null() {
        CmdLineParse(CmdLine);
        return;
    } else if IsListEmpty(IniGetFileSectionListHead() as *const LIST_ENTRY) {
        return;
    }

    let mut FoundLoaderSection: u8 = 0u8;
    let LoaderSections: [PCSTR; 4] = [
        b"FreeLoader\0".as_ptr() as PCSTR,
        b"Boot Loader\0".as_ptr() as PCSTR,
        b"FlexBoot\0".as_ptr() as PCSTR,
        b"MultiBoot\0".as_ptr() as PCSTR,
    ];

    if BootMgrInfo.FrLdrSection.is_null() {
        let mut i: u32 = 0;
        while i < 4 {
            let Section: PCSTR = LoaderSections[i as usize];

            if IniOpenSection(Section, core::ptr::addr_of_mut!(BootMgrInfo.FrLdrSection)) != 0 {
                FoundLoaderSection = 1u8;
                break;
            }
            i += 1;
        }

        if FoundLoaderSection == 0 {
            UiMessageBoxCritical(b"Bootloader Section not found in freeldr.ini\0".as_ptr() as PCSTR);
            return;
        }
    }

    if IniReadSettingByName(BootMgrInfo.FrLdrSection, b"Debug\0".as_ptr() as PCSTR,
                            DebugString.as_mut_ptr(), core::mem::size_of_val(&DebugString) as ULONG) != 0 {
        BootMgrInfo.DebugString = DebugString.as_ptr() as PCSTR;
    }

    if BootMgrInfo.TimeOut < 0 {
        let mut TimeOutText: [i8; 20] = [0; 20];
        BootMgrInfo.TimeOut = -1;
        if IniReadSettingByName(BootMgrInfo.FrLdrSection, b"TimeOut\0".as_ptr() as PCSTR,
                                TimeOutText.as_mut_ptr(), core::mem::size_of_val(&TimeOutText) as ULONG) != 0 {
            BootMgrInfo.TimeOut = atoi(TimeOutText.as_ptr());
        }
    }

    if BootMgrInfo.DefaultOs.is_null() || *BootMgrInfo.DefaultOs == 0 {
        if IniReadSettingByName(BootMgrInfo.FrLdrSection, b"DefaultOS\0".as_ptr() as PCSTR,
                                DefaultOs.as_mut_ptr(), core::mem::size_of_val(&DefaultOs) as ULONG) != 0 {
            BootMgrInfo.DefaultOs = DefaultOs.as_ptr() as PCSTR;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetBootMgrInfo() -> PBOOTMGRINFO {
    core::ptr::addr_of_mut!(BootMgrInfo)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     BSD - See COPYING.ARM in the top level directory
//  *              or MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Command-line parsing and global settings management
//  * COPYRIGHT:   Copyright 2008-2010 ReactOS Portable Systems Group <ros.arm@reactos.org>
//  *              Copyright 2015-2024 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
//
// /* GLOBALS ********************************************************************/
//
// static CCHAR DebugString[256];
// static CCHAR VideoOptions[128];
// static CCHAR DefaultOs[256];
// BOOTMGRINFO BootMgrInfo = {NULL, NULL, NULL, -1, 0};
//
// /* FUNCTIONS ******************************************************************/
//
// static VOID
// CmdLineParse(
//     _In_ PCSTR CmdLine)
// {
//     PCHAR Setting;
//     size_t Length;
//     ULONG_PTR Offset = 0;
//
//     /*
//      * Get the debug string, in the following format:
//      * "debug=option1=XXX;option2=YYY;..."
//      * and translate it into the format:
//      * "OPTION1=XXX OPTION2=YYY ..."
//      */
//     Setting = strstr(CmdLine, "debug=");
//     if (Setting)
//     {
//         /* Check if there are more command-line parameters following */
//         Setting += sizeof("debug=") - sizeof(ANSI_NULL);
//         Length = strcspn(Setting, " \t");
//
//         /* Copy the debug string and upcase it */
//         RtlStringCbCopyNA(DebugString, sizeof(DebugString), Setting, Length);
//         _strupr(DebugString);
//
//         /* Replace all separators ';' by spaces */
//         Setting = DebugString;
//         while (*Setting)
//         {
//             if (*Setting == ';') *Setting = ' ';
//             Setting++;
//         }
//
//         BootMgrInfo.DebugString = DebugString;
//     }
//
//     /* Get the video options */
//     Setting = strstr(CmdLine, "video=");
//     if (Setting)
//     {
//         Setting += sizeof("video=") - sizeof(ANSI_NULL);
//     }
// #if (defined(_M_IX86) || defined(_M_AMD64)) && !defined(SARCH_XBOX) && !defined(SARCH_PC98)
//     else
//     {
//         Setting = strstr(CmdLine, "vga=");
//         if (Setting)
//             Setting += sizeof("vga=") - sizeof(ANSI_NULL);
//     }
// #endif
//     if (Setting)
//     {
//         /* Check if there are more command-line parameters following */
//         Length = strcspn(Setting, " \t");
//
//         /* Copy the string */
//         RtlStringCbCopyNA(VideoOptions, sizeof(VideoOptions), Setting, Length);
//         BootMgrInfo.VideoOptions = VideoOptions;
//     }
//
//     /* Get the timeout */
//     Setting = strstr(CmdLine, "timeout=");
//     if (Setting)
//     {
//         BootMgrInfo.TimeOut = atoi(Setting +
//                                    sizeof("timeout=") - sizeof(ANSI_NULL));
//     }
//
//     /* Get the default OS */
//     Setting = strstr(CmdLine, "defaultos=");
//     if (Setting)
//     {
//         /* Check if there are more command-line parameters following */
//         Setting += sizeof("defaultos=") - sizeof(ANSI_NULL);
//         Length = strcspn(Setting, " \t");
//
//         /* Copy the default OS */
//         RtlStringCbCopyNA(DefaultOs, sizeof(DefaultOs), Setting, Length);
//         BootMgrInfo.DefaultOs = DefaultOs;
//     }
//
//     /* Get the ramdisk base address */
//     Setting = strstr(CmdLine, "rdbase=");
//     if (Setting)
//     {
//         gInitRamDiskBase =
//             (PVOID)(ULONG_PTR)strtoull(Setting +
//                                        sizeof("rdbase=") - sizeof(ANSI_NULL),
//                                        NULL, 0);
//     }
//
//     /* Get the ramdisk size */
//     Setting = strstr(CmdLine, "rdsize=");
//     if (Setting)
//     {
//         gInitRamDiskSize = strtoul(Setting +
//                                    sizeof("rdsize=") - sizeof(ANSI_NULL),
//                                    NULL, 0);
//     }
//
//     /* Get the ramdisk offset */
//     Setting = strstr(CmdLine, "rdoffset=");
//     if (Setting)
//     {
//         Offset = strtoul(Setting +
//                          sizeof("rdoffset=") - sizeof(ANSI_NULL),
//                          NULL, 0);
//     }
//
//     /* Fix it up */
//     gInitRamDiskBase = (PVOID)((ULONG_PTR)gInitRamDiskBase + Offset);
// }
//
// VOID
// LoadSettings(
//     _In_opt_ PCSTR CmdLine)
// {
//     /* Pre-initialization: The settings originate from the command-line.
//      * Main initialization: Overwrite them if needed with those from freeldr.ini */
//     if (CmdLine)
//     {
//         CmdLineParse(CmdLine);
//         return;
//     }
//     else if (IsListEmpty(IniGetFileSectionListHead()))
//     {
//         // ERR("LoadSettings() called but no freeldr.ini\n");
//         return;
//     }
//
//     BOOLEAN FoundLoaderSection = FALSE;
//     PCSTR LoaderSections[] = {
//         "FreeLoader",
//         "Boot Loader",
//         "FlexBoot",
//         "MultiBoot",
//     };
//
//     /* Search for the first section in LoaderSections and load the settings, 
//      * prioritizing the order in the file.
//      * If a section is already loaded, skip further checks. */
//     if (!BootMgrInfo.FrLdrSection)
//     {
//         for (ULONG i = 0; i < sizeof(LoaderSections) / sizeof(LoaderSections[0]); i++)
//         {
//             PCSTR Section = LoaderSections[i];
//
//             if (IniOpenSection(Section, &BootMgrInfo.FrLdrSection))
//             {
//                 FoundLoaderSection = TRUE;
//                 break;
//             }
//         }
//
//         if (!FoundLoaderSection)
//         {
//             UiMessageBoxCritical("Bootloader Section not found in freeldr.ini");
//             return;
//         }
//     }
//
//     /* Get the debug string. Always override it with the one from freeldr.ini */
//     if (IniReadSettingByName(BootMgrInfo.FrLdrSection, "Debug",
//                              DebugString, sizeof(DebugString)))
//     {
//         BootMgrInfo.DebugString = DebugString;
//     }
//
//     /* Get the timeout. Keep the existing one if it is valid,
//      * otherwise retrieve it from freeldr.ini */
//     if (BootMgrInfo.TimeOut < 0)
//     {
//         CHAR TimeOutText[20];
//         BootMgrInfo.TimeOut = -1;
//         if (IniReadSettingByName(BootMgrInfo.FrLdrSection, "TimeOut",
//                                  TimeOutText, sizeof(TimeOutText)))
//         {
//             BootMgrInfo.TimeOut = atoi(TimeOutText);
//         }
//     }
//
//     /* Get the default OS */
//     if (!BootMgrInfo.DefaultOs || !*BootMgrInfo.DefaultOs)
//     {
//         if (IniReadSettingByName(BootMgrInfo.FrLdrSection, "DefaultOS",
//                                  DefaultOs, sizeof(DefaultOs)))
//         {
//             BootMgrInfo.DefaultOs = DefaultOs;
//         }
//     }
// }
//
// PBOOTMGRINFO GetBootMgrInfo(VOID)
// {
//     return &BootMgrInfo;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
