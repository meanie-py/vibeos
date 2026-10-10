#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_INIFILE; // ini_init.c: DBG_DEFAULT_CHANNEL(INIFILE)
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION};
use crate::frldr::{TAG_INI_FILE, FrLdrTempAlloc, FrLdrTempFree};
use core::ffi::c_void;

const ESUCCESS: ARC_STATUS = 0;
#[allow(non_upper_case_globals)]
const OpenReadOnly: u32 = 0;

extern "C" {
    fn FsOpenFile(FileName: *const i8, DeviceName: *const i8, OpenMode: u32, FileId: *mut u32) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: u32, Info: *mut FILEINFORMATION) -> ARC_STATUS;
    fn ArcClose(FileId: u32) -> ARC_STATUS;
    fn ArcRead(FileId: u32, Buffer: *mut c_void, N: u32, Count: *mut u32) -> ARC_STATUS;
    fn UiMessageBoxCritical(Message: *const i8);
    fn IniParseFile(IniFileData: *const i8, IniFileSize: u32) -> u8;
    // C: `CCHAR FrLdrBootPath[MAX_PATH]` — an ARRAY whose name decays to a
    // pointer at use sites. Declaring it `*const i8` in Rust would create a
    // pointer-shaped SLOT at the symbol's address and read the first 4 bytes
    // of the path string as a pointer (silent garbage). Use the accessor.
    fn FrLdrGetBootPath() -> *const i8;
}

#[no_mangle]
pub unsafe extern "C" fn IniFileInitialize() -> u8 {
    let mut FileInformation: FILEINFORMATION = core::mem::zeroed();
    let mut FileId: u32 = 0;
    let FreeLoaderIniFileData: *mut c_void;
    let FreeLoaderIniFileSize: u32;
    let mut Count: u32 = 0;
    let mut Status: ARC_STATUS;
    let Success: u8;

    TRACE!("IniFileInitialize()\n");

    let boot_path = FrLdrGetBootPath();

    Status = FsOpenFile(
        b"freeldr.ini\0".as_ptr() as *const i8,
        boot_path,
        OpenReadOnly,
        &mut FileId,
    );
    if Status != ESUCCESS {
        ERR!("Error while opening freeldr.ini, Status: %d\n", Status);

        Status = FsOpenFile(
            b"boot.ini\0".as_ptr() as *const i8,
            boot_path,
            OpenReadOnly,
            &mut FileId,
        );
        if Status != ESUCCESS {
            ERR!("Error while opening boot.ini, Status: %d\n", Status);
            UiMessageBoxCritical(b"Error opening freeldr.ini/boot.ini or file not found.\nYou need to re-install FreeLoader.\0".as_ptr() as *const i8);
            return 0u8;
        }
    }

    Status = ArcGetFileInformation(FileId, &mut FileInformation);
    if Status != ESUCCESS || FileInformation.EndingAddress.__bindgen_anon_1.HighPart != 0 {
        UiMessageBoxCritical(b"Error while getting informations about freeldr.ini.\nYou need to re-install FreeLoader.\0".as_ptr() as *const i8);
        ArcClose(FileId);
        return 0u8;
    }
    FreeLoaderIniFileSize = FileInformation.EndingAddress.__bindgen_anon_1.LowPart;

    FreeLoaderIniFileData = FrLdrTempAlloc(FreeLoaderIniFileSize as usize, TAG_INI_FILE);
    if FreeLoaderIniFileData.is_null() {
        UiMessageBoxCritical(b"Out of memory while loading freeldr.ini.\0".as_ptr() as *const i8);
        ArcClose(FileId);
        return 0u8;
    }

    Status = ArcRead(FileId, FreeLoaderIniFileData, FreeLoaderIniFileSize, &mut Count);
    if Status != ESUCCESS || Count != FreeLoaderIniFileSize {
        ERR!("Error while reading freeldr.ini, Status: %d\n", Status);
        UiMessageBoxCritical(b"Error while reading freeldr.ini.\0".as_ptr() as *const i8);
        ArcClose(FileId);
        FrLdrTempFree(FreeLoaderIniFileData, TAG_INI_FILE);
        return 0u8;
    }

    Success = IniParseFile(FreeLoaderIniFileData as *const i8, FreeLoaderIniFileSize);

    ArcClose(FileId);
    FrLdrTempFree(FreeLoaderIniFileData, TAG_INI_FILE);

    return Success;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 2009     Hervé Poussineau  <hpoussin@reactos.org>
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
// DBG_DEFAULT_CHANNEL(INIFILE);
//
// BOOLEAN IniFileInitialize(VOID)
// {
//     FILEINFORMATION FileInformation;
//     ULONG FileId; // File handle for freeldr.ini
//     PCHAR FreeLoaderIniFileData;
//     ULONG FreeLoaderIniFileSize, Count;
//     ARC_STATUS Status;
//     BOOLEAN Success;
//
//     TRACE("IniFileInitialize()\n");
//
//     /* Try to open freeldr.ini */
//     Status = FsOpenFile("freeldr.ini", FrLdrBootPath, OpenReadOnly, &FileId);
//     if (Status != ESUCCESS)
//     {
//         ERR("Error while opening freeldr.ini, Status: %d\n", Status);
//
//         /* Try to open boot.ini */
//         Status = FsOpenFile("boot.ini", FrLdrBootPath, OpenReadOnly, &FileId);
//         if (Status != ESUCCESS)
//         {
//             ERR("Error while opening boot.ini, Status: %d\n", Status);
//             UiMessageBoxCritical("Error opening freeldr.ini/boot.ini or file not found.\nYou need to re-install FreeLoader.");
//             return FALSE;
//         }
//     }
//
//     /* Get the file size */
//     Status = ArcGetFileInformation(FileId, &FileInformation);
//     if (Status != ESUCCESS || FileInformation.EndingAddress.HighPart != 0)
//     {
//         UiMessageBoxCritical("Error while getting informations about freeldr.ini.\nYou need to re-install FreeLoader.");
//         ArcClose(FileId);
//         return FALSE;
//     }
//     FreeLoaderIniFileSize = FileInformation.EndingAddress.LowPart;
//
//     /* Allocate memory to cache the whole freeldr.ini */
//     FreeLoaderIniFileData = FrLdrTempAlloc(FreeLoaderIniFileSize as usize, TAG_INI_FILE);
//     if (!FreeLoaderIniFileData)
//     {
//         UiMessageBoxCritical("Out of memory while loading freeldr.ini.");
//         ArcClose(FileId);
//         return FALSE;
//     }
//
//     /* Load freeldr.ini from the disk */
//     Status = ArcRead(FileId, FreeLoaderIniFileData, FreeLoaderIniFileSize, &Count);
//     if (Status != ESUCCESS || Count != FreeLoaderIniFileSize)
//     {
//         ERR("Error while reading freeldr.ini, Status: %d\n", Status);
//         UiMessageBoxCritical("Error while reading freeldr.ini.");
//         ArcClose(FileId);
//         FrLdrTempFree(FreeLoaderIniFileData, TAG_INI_FILE);
//         return FALSE;
//     }
//
//     /* Parse the .ini file data */
//     Success = IniParseFile(FreeLoaderIniFileData as *const i8, FreeLoaderIniFileSize);
//
//     /* Do some cleanup, and return */
//     ArcClose(FileId);
//     FrLdrTempFree(FreeLoaderIniFileData, TAG_INI_FILE);
//
//     return Success;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
