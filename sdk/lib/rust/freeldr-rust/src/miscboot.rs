#[allow(unused_imports)] use crate::{ERR};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_DISK; // miscboot.c: DBG_DEFAULT_CHANNEL(DISK)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCSTR, PSTR};
use core::ffi::c_void;
// ARC types/codes/openmode live in crate::ffi.
use crate::ffi::{ARC_STATUS, ESUCCESS, ENOEXEC, OpenReadOnly};

// arc.h enum _ARC_CODES — only the few used here.
const EINVAL: ARC_STATUS = 7;  // nolint: const-contradiction (ARC_STATUS enumerator; arc.h #undefs the C EINVAL)
const EIO: ARC_STATUS = 8; // nolint: const-contradiction (ARC_CODES enumerator, arc.h:40 — not errno EIO=5)

extern "C" {
    fn GetArgumentValue(Argc: u32, Argv: *mut PSTR, ArgName: PCSTR) -> PCSTR;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn UiShowMessageBoxesInArgv(Argc: u32, Argv: *mut PSTR);
    fn DriveMapGetBiosDriveNumber(Name: PCSTR) -> u8;
    fn atoi(s: PCSTR) -> i32;
    fn DissectArcPath(Path: PCSTR, FileName: *mut PCSTR, DriveNumber: *mut u8, PartitionNumber: *mut u32) -> BOOLEAN;
    fn UiMessageBox(fmt: PCSTR, ...);
    fn ConstructArcPath(Path: PSTR, FileName: PCSTR, DriveNumber: u8, PartitionNumber: u32);
    fn strrchr(s: PCSTR, c: i32) -> PSTR;
    fn strstr(haystack: PCSTR, needle: PCSTR) -> PCSTR;
    fn FrldrGetBootDrive() -> u8;
    fn FrldrGetBootPartition() -> u32;
    fn FsOpenFile(FileName: PCSTR, BootPath: PCSTR, OpenMode: u32, FileId: *mut u32) -> ARC_STATUS;
    fn ArcOpen(Path: PSTR, OpenMode: u32, FileId: *mut u32) -> ARC_STATUS;
    fn MachGetBootSectorLoadAddress(DriveNumber: u8) -> u32;
    fn ArcRead(FileId: u32, Buffer: *mut core::ffi::c_void, N: u32, Count: *mut u32) -> ARC_STATUS;
    fn ArcClose(FileId: u32) -> ARC_STATUS;
    fn UiUnInitialize(Status: PCSTR);
    fn IniCleanup();
    fn ChainLoadBiosBootSectorCode(DriveNumber: u8, PartitionNumber: u32);
}

// `UlongToPtr` is the Windows SDK inline. Cast directly.
#[inline(always)]
unsafe fn UlongToPtr(val: u32) -> *mut core::ffi::c_void {
    val as usize as *mut core::ffi::c_void
}

// ---- file-scope preamble (translated) ----
// No file-scope preamble declarations to translate.
// This file contains only includes, debug channel setup, and a single function definition.

#[no_mangle]
pub unsafe extern "C" fn LoadAndBootSector(
    Argc: u32,
    Argv: *mut PSTR,
    Envp: *mut PSTR,
) -> ARC_STATUS {
    let mut Status: ARC_STATUS;
    let mut ArgValue: PCSTR;
    let mut BootPath: PCSTR;
    let mut FileName: PCSTR;
    let mut BiosDriveNumber: u8 = 0;
    let mut PartitionNumber: u32 = 0;
    let LoadAddress: u32;
    let mut FileId: u32 = 0;
    let mut BytesRead: u32 = 0;
    let mut ArcPath: [i8; 260] = [0; 260];

    #[cfg(debug_assertions)]
    {
        ArgValue = GetArgumentValue(Argc, Argv, b"BootType\0".as_ptr() as PCSTR);
        if ArgValue.is_null() || *ArgValue == 0 || _stricmp(ArgValue, b"BootSector\0".as_ptr() as PCSTR) != 0 {
            ERR!("Unexpected boot type '%s', aborting\n", if !ArgValue.is_null() { ArgValue } else { b"n/a\0".as_ptr() as PCSTR });
            return EINVAL;
        }
    }

    UiShowMessageBoxesInArgv(Argc, Argv);

    BootPath = GetArgumentValue(Argc, Argv, b"BootPath\0".as_ptr() as PCSTR);
    if BootPath.is_null() || *BootPath == 0 {
        ArgValue = GetArgumentValue(Argc, Argv, b"BootDrive\0".as_ptr() as PCSTR);
        if !ArgValue.is_null() && *ArgValue != 0 {
            BiosDriveNumber = DriveMapGetBiosDriveNumber(ArgValue);

            PartitionNumber = 0;
            ArgValue = GetArgumentValue(Argc, Argv, b"BootPartition\0".as_ptr() as PCSTR);
            if !ArgValue.is_null() && *ArgValue != 0 {
                PartitionNumber = atoi(ArgValue) as u32;
            }
        } else {
            BootPath = GetArgumentValue(Argc, Argv, b"SystemPartition\0".as_ptr() as PCSTR);
        }
    }

    if !BootPath.is_null() && *BootPath != 0 {
        FileName = core::ptr::null();
        if DissectArcPath(
            BootPath,
            core::ptr::addr_of_mut!(FileName),
            core::ptr::addr_of_mut!(BiosDriveNumber),
            core::ptr::addr_of_mut!(PartitionNumber),
        ) == 0
            || (!FileName.is_null() && *FileName != 0)
        {
            UiMessageBox(b"Currently unsupported BootPath value:\n%s\0".as_ptr() as PCSTR, BootPath);
            return EINVAL;
        }
    } else {
        ConstructArcPath(ArcPath.as_mut_ptr(), b"\0".as_ptr() as PCSTR, BiosDriveNumber, PartitionNumber);
        let separator_pos = strrchr(ArcPath.as_ptr(), b'\\' as i32);
        *separator_pos = 0i8;
        BootPath = ArcPath.as_ptr();
    }

    FileName = core::ptr::null();
    if !strstr(BootPath, b")partition()\0".as_ptr() as PCSTR).is_null()
        || !strstr(BootPath, b")partition(0)\0".as_ptr() as PCSTR).is_null()
    {
    } else {
        FileName = GetArgumentValue(Argc, Argv, b"BootSectorFile\0".as_ptr() as PCSTR);
        if !FileName.is_null() && *FileName == 0 {
            FileName = core::ptr::null();
        }
    }

    if !FileName.is_null() {
        BiosDriveNumber = 0;
    }
    if BiosDriveNumber == 0 {
        BiosDriveNumber = FrldrGetBootDrive();
        PartitionNumber = FrldrGetBootPartition();
    }

    if !FileName.is_null() {
        Status = FsOpenFile(FileName, BootPath, OpenReadOnly, core::ptr::addr_of_mut!(FileId));
    } else {
        Status = ArcOpen(BootPath as PSTR, OpenReadOnly, core::ptr::addr_of_mut!(FileId));
    }
    if Status != ESUCCESS {
        UiMessageBox(
            b"Unable to open %s\0".as_ptr() as PCSTR,
            if !FileName.is_null() { FileName } else { BootPath },
        );
        return Status;
    }

    LoadAddress = MachGetBootSectorLoadAddress(BiosDriveNumber);

    Status = ArcRead(FileId, UlongToPtr(LoadAddress), 512, core::ptr::addr_of_mut!(BytesRead));
    ArcClose(FileId);
    if Status != ESUCCESS || BytesRead != 512 {
        let WhatFailed: PCSTR;

        if !FileName.is_null() {
            WhatFailed = b"boot sector file\0".as_ptr() as PCSTR;
        } else if PartitionNumber != 0 {
            WhatFailed = b"partition's boot sector\0".as_ptr() as PCSTR;
        } else {
            WhatFailed = b"MBR boot sector\0".as_ptr() as PCSTR;
        }

        UiMessageBox(b"Unable to load %s.\0".as_ptr() as PCSTR, WhatFailed);
        return EIO;
    }

    if *(UlongToPtr(LoadAddress + 0x1FE) as *const u16) != 0xAA55 {
        UiMessageBox(b"Invalid boot sector magic (0xAA55)\0".as_ptr() as PCSTR);
        return ENOEXEC;
    }

    UiUnInitialize(b"Booting...\0".as_ptr() as PCSTR);
    IniCleanup();

    // BIOS-only call (UEFI build doesn't have ChainLoadBiosBootSectorCode);
    // gate with the project's `uefi` feature.
    #[cfg(not(feature = "uefi"))]
    {
        ChainLoadBiosBootSectorCode(BiosDriveNumber, PartitionNumber);
    }

    return ESUCCESS;
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
// #if defined(_M_IX86) || defined(_M_AMD64)
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(DISK);
//
// /* FUNCTIONS ******************************************************************/
//
// /**
//  * @brief
//  * Loads and boots a disk MBR, a partition VBR or a file boot sector.
//  **/
// ARC_STATUS
// LoadAndBootSector(
//     _In_ ULONG Argc,
//     _In_ PCHAR Argv[],
//     _In_ PCHAR Envp[])
// {
//     ARC_STATUS Status;
//     PCSTR ArgValue;
//     PCSTR BootPath;
//     PCSTR FileName;
//     UCHAR BiosDriveNumber = 0;
//     ULONG PartitionNumber = 0;
//     ULONG LoadAddress;
//     ULONG FileId;
//     ULONG BytesRead;
//     CHAR ArcPath[MAX_PATH];
//
// #if DBG
//     /* Ensure the boot type is the one expected */
//     ArgValue = GetArgumentValue(Argc, Argv, "BootType");
//     if (!ArgValue || !*ArgValue || _stricmp(ArgValue, "BootSector") != 0)
//     {
//         ERR("Unexpected boot type '%s', aborting\n", ArgValue ? ArgValue : "n/a");
//         return EINVAL;
//     }
// #endif
//
//     /* Find all the message box settings and run them */
//     UiShowMessageBoxesInArgv(Argc, Argv);
//
//     /*
//      * Check whether we have a "BootPath" value (takes precedence
//      * over both "BootDrive" and "BootPartition").
//      */
//     BootPath = GetArgumentValue(Argc, Argv, "BootPath");
//     if (!BootPath || !*BootPath)
//     {
//         /* We don't have one, check whether we use "BootDrive" and "BootPartition" */
//
//         /* Retrieve the boot drive (optional, fall back to using default path otherwise) */
//         ArgValue = GetArgumentValue(Argc, Argv, "BootDrive");
//         if (ArgValue && *ArgValue)
//         {
//             BiosDriveNumber = DriveMapGetBiosDriveNumber(ArgValue);
//
//             /* Retrieve the boot partition (optional, fall back to zero otherwise) */
//             PartitionNumber = 0;
//             ArgValue = GetArgumentValue(Argc, Argv, "BootPartition");
//             if (ArgValue && *ArgValue)
//                 PartitionNumber = atoi(ArgValue);
//         }
//         else
//         {
//             /* Fall back to using the system partition as default path */
//             BootPath = GetArgumentValue(Argc, Argv, "SystemPartition");
//         }
//     }
//
//     /*
//      * The ARC "BootPath" value takes precedence over
//      * both the BiosDriveNumber and PartitionNumber options.
//      */
//     if (BootPath && *BootPath)
//     {
//         /*
//          * Retrieve the BIOS drive and partition numbers; verify also that the
//          * path is "valid" in the sense that it must not contain any file name.
//          */
//         FileName = NULL;
//         if (!DissectArcPath(BootPath, &FileName, &BiosDriveNumber, &PartitionNumber) ||
//             (FileName && *FileName))
//         {
//             UiMessageBox("Currently unsupported BootPath value:\n%s", BootPath);
//             return EINVAL;
//         }
//     }
//     else
//     {
//         /* We don't have one, so construct the corresponding ARC path */
//         ConstructArcPath(ArcPath, "", BiosDriveNumber, PartitionNumber);
//         *strrchr(ArcPath, '\\') = ANSI_NULL; // Trim the trailing path separator.
//         BootPath = ArcPath;
//     }
//
//     FileName = NULL;
//     if (strstr(BootPath, ")partition()") || strstr(BootPath, ")partition(0)"))
//     {
//         /*
//          * The partition specifier is zero i.e. the device is accessed
//          * in an unpartitioned fashion, do not retrieve a file name.
//          *
//          * NOTE: If we access a floppy drive, we would not have a
//          * partition specifier, and PartitionNumber would be == 0,
//          * so don't check explicitly for PartitionNumber because
//          * we want to retrieve a file name.
//          */
//     }
//     else
//     {
//         /* Retrieve the file name, if any, and normalize
//          * the pointer to make subsequent tests simpler */
//         FileName = GetArgumentValue(Argc, Argv, "BootSectorFile");
//         if (FileName && !*FileName)
//             FileName = NULL;
//     }
//
//
//     /* If we load a boot sector file, reset the drive number
//      * so as to use the original boot drive/partition */
//     if (FileName)
//         BiosDriveNumber = 0;
//     if (!BiosDriveNumber)
//     {
//         BiosDriveNumber = FrldrGetBootDrive();
//         PartitionNumber = FrldrGetBootPartition();
//     }
//
//
//     /* Open the boot sector file or the volume */
//     if (FileName)
//         Status = FsOpenFile(FileName, BootPath, OpenReadOnly, &FileId);
//     else
//         Status = ArcOpen((PSTR)BootPath, OpenReadOnly, &FileId);
//     if (Status != ESUCCESS)
//     {
//         UiMessageBox("Unable to open %s", FileName ? FileName : BootPath);
//         return Status;
//     }
//
//     LoadAddress = MachGetBootSectorLoadAddress(BiosDriveNumber);
//
//     /*
//      * Now try to load the boot sector: disk MBR (when PartitionNumber == 0),
//      * partition VBR or boot sector file. If this fails, abort.
//      */
//     Status = ArcRead(FileId, UlongToPtr(LoadAddress), 512, &BytesRead);
//     ArcClose(FileId);
//     if ((Status != ESUCCESS) || (BytesRead != 512))
//     {
//         PCSTR WhatFailed;
//
//         if (FileName)
//             WhatFailed = "boot sector file";
//         else if (PartitionNumber != 0)
//             WhatFailed = "partition's boot sector";
//         else
//             WhatFailed = "MBR boot sector";
//
//         UiMessageBox("Unable to load %s.", WhatFailed);
//         return EIO;
//     }
//
//     /* Check for validity */
//     if (*(USHORT*)UlongToPtr(LoadAddress + 0x1FE) != 0xAA55)
//     {
//         UiMessageBox("Invalid boot sector magic (0xAA55)");
//         return ENOEXEC;
//     }
//
//     UiUnInitialize("Booting...");
//     IniCleanup();
//
// #ifndef UEFIBOOT
//     /* Boot the loaded sector code */
//     ChainLoadBiosBootSectorCode(BiosDriveNumber, PartitionNumber);
// #endif
//     /* Must not return! */
//     return ESUCCESS;
// }
//
// #endif /* _M_IX86 || _M_AMD64 */
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
