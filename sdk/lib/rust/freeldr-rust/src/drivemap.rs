#[allow(unused_imports)] use crate::{TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_DISK; // drivemap.c: DBG_DEFAULT_CHANNEL(DISK)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;

extern "C" {
    fn atoi(s: *const i8) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn strtoul(s: PCSTR, endptr: *mut PCSTR, base: i32) -> u32;
    fn CacheInvalidateCacheData();
    fn ULongToPtr(ul: ULONG) -> PVOID;
    fn UlongToPtr(ul: ULONG) -> PVOID;
    fn IniGetNumSectionItems(SectionId: ULONG_PTR) -> ULONG;
    fn IniReadSettingByNumber( SectionId: ULONG_PTR, Index: ULONG, SettingName: PCHAR, SettingNameSize: ULONG, SettingValue: PCHAR, SettingValueSize: ULONG, ) -> u8;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn strcpy(dest: PCHAR, src: PCSTR) -> PCHAR;
    fn UiMessageBox(format: PCSTR, ...);
}

// drvmap.S labels (external linkage); taken by-address in
// DriveMapInstallInt13Handler. drvmap.S is i386-only asm (it has .code16
// with symbol arithmetic Clang can't handle); the entire INT13-handler
// install path is wrapped in `#ifdef _M_IX86` in drivemap.c. Mirror that
// gate so amd64 (no drvmap.S) doesn't reference these.
#[cfg(target_arch = "x86")]
extern "C" {
    static DriveMapInt13HandlerStart: PVOID;
    static DriveMapInt13HandlerEnd: PVOID;
    static mut DriveMapInt13HandlerMapList: DRIVE_MAP_LIST;
    static mut DriveMapOldInt13HandlerAddress: ULONG;
}

// ---- rustconv dropped file-local content from drivemap.c / drivemap.h, restored faithfully ----

// drivemap.h: #pragma pack(1) struct { UCHAR DriveMapCount; CHAR DriveMap[8]; }
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct DRIVE_MAP_LIST {
    pub DriveMapCount: UCHAR,
    pub DriveMap: [CHAR; 8],
}
pub type PDRIVE_MAP_LIST = *mut DRIVE_MAP_LIST;

// drivemap.c file-local globals (C external linkage, but unreferenced by other freeldr TUs).
static mut DriveMapInstalled: BOOLEAN = 0;
static mut OldInt13HandlerAddress: ULONG = 0;
static mut DriveMapHandlerAddress: ULONG = 0;
static mut DriveMapHandlerSegOff: ULONG = 0;

unsafe fn DriveMapIsValidDriveString(DriveString: PCSTR) -> u8 {
    let mut Index: u32;

    if (strlen(DriveString) < 3) ||
        ((*DriveString.offset(0) != b'f' as i8) && (*DriveString.offset(0) != b'F' as i8) &&
         (*DriveString.offset(0) != b'h' as i8) && (*DriveString.offset(0) != b'H' as i8)) ||
        ((*DriveString.offset(1) != b'd' as i8) && (*DriveString.offset(1) != b'D' as i8))
    {
        return 0u8;
    }

    Index = 2;
    while Index < strlen(DriveString) as u32 {
        if *DriveString.offset(Index as isize) < b'0' as i8 || *DriveString.offset(Index as isize) > b'9' as i8 {
            return 0u8;
        }
        Index += 1;
    }

    if (atoi(DriveString.offset(2)) < 0) || (atoi(DriveString.offset(2)) > 0xff) {
        return 0u8;
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn DriveMapGetBiosDriveNumber(DeviceName: PCSTR) -> u8 {
    let mut BiosDriveNumber: u8 = 0;

    TRACE!("DriveMapGetBiosDriveNumber(%s)\n", (DeviceName) as usize);

    if *DeviceName >= ((b'0') as i8) && *DeviceName <= ((b'9') as i8) {
        return strtoul(DeviceName, core::ptr::null_mut(), 0) as u8;
    }

    BiosDriveNumber = atoi(DeviceName.offset(2)) as u8;

    if (*DeviceName == ((b'h') as i8) || *DeviceName == ((b'H') as i8)) &&
       (*DeviceName.offset(1) == ((b'd') as i8) || *DeviceName.offset(1) == ((b'D') as i8)) {
        BiosDriveNumber |= 0x80;
    }

    BiosDriveNumber
}

// drivemap.c wraps DriveMapMapDrivesInSection / DriveMapInstallInt13Handler
// / DriveMapRemoveInt13Handler in `#ifdef _M_IX86` (they patch the 16-bit
// real-mode INT13 IVT via drvmap.S). bootmgr.c also only calls
// DriveMapMapDrivesInSection under `#ifdef _M_IX86 && !UEFIBOOT`. Mirror
// the C gate: i386-only. (DriveMapGetBiosDriveNumber stays arch-neutral.)
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn DriveMapMapDrivesInSection(SectionId: ULONG_PTR) {
    let mut SettingName: [i8; 80] = [0; 80];
    let mut SettingValue: [i8; 80] = [0; 80];
    let mut Drive1: [i8; 80] = [0; 80];
    let mut Drive2: [i8; 80] = [0; 80];
    let SectionItemCount: ULONG;
    let mut Index: ULONG;
    let mut Index2: ULONG;
    let mut DriveMapList: DRIVE_MAP_LIST = core::mem::zeroed();

    if SectionId == 0 {
        return;
    }

    RtlZeroMemory(
        &mut DriveMapList as *mut _ as PVOID,
        core::mem::size_of::<DRIVE_MAP_LIST>(),
    );

    SectionItemCount = IniGetNumSectionItems(SectionId);

    Index = 0;
    while Index < SectionItemCount {
        if IniReadSettingByNumber(
            SectionId,
            Index,
            SettingName.as_mut_ptr(),
            core::mem::size_of_val(&SettingName) as ULONG,
            SettingValue.as_mut_ptr(),
            core::mem::size_of_val(&SettingValue) as ULONG,
        ) != 0
        {
            if _stricmp(SettingName.as_ptr(), b"DriveMap\0".as_ptr() as PCSTR) == 0 {
                if DriveMapList.DriveMapCount >= 4 {
                    UiMessageBox(
                        b"Max DriveMap count exceeded in section [%s]:\n\n%s=%s\0".as_ptr() as PCSTR,
                        (*(SectionId as PINI_SECTION)).SectionName,
                        SettingName.as_ptr(),
                        SettingValue.as_ptr(),
                    );
                    Index += 1;
                    continue;
                }

                RtlZeroMemory(Drive1.as_mut_ptr() as PVOID, 80);
                RtlZeroMemory(Drive2.as_mut_ptr() as PVOID, 80);

                strcpy(Drive1.as_mut_ptr(), SettingValue.as_ptr());

                Index2 = 0;
                while Index2 < ((strlen(Drive1.as_ptr())) as u32) {
                    if Drive1[Index2 as usize] == b',' as i8 {
                        Drive1[Index2 as usize] = 0;
                        strcpy(Drive2.as_mut_ptr(), Drive1.as_ptr().offset((Index2 + 1) as isize));
                        break;
                    }
                    Index2 += 1;
                }

                if DriveMapIsValidDriveString(Drive1.as_ptr()) == 0
                    || DriveMapIsValidDriveString(Drive2.as_ptr()) == 0
                {
                    UiMessageBox(
                        b"Error in DriveMap setting in section [%s]:\n\n%s=%s\0".as_ptr() as PCSTR,
                        (*(SectionId as PINI_SECTION)).SectionName,
                        SettingName.as_ptr(),
                        SettingValue.as_ptr(),
                    );
                    Index += 1;
                    continue;
                }

                DriveMapList.DriveMap[(DriveMapList.DriveMapCount * 2) as usize] =
                    ((DriveMapGetBiosDriveNumber(Drive1.as_ptr())) as i8);
                DriveMapList.DriveMap[(DriveMapList.DriveMapCount * 2 + 1) as usize] =
                    ((DriveMapGetBiosDriveNumber(Drive2.as_ptr())) as i8);
                DriveMapList.DriveMapCount += 1;

                TRACE!("Mapping BIOS drive 0x%x to drive 0x%x\n", (DriveMapGetBiosDriveNumber(Drive1.as_ptr())) as usize, (DriveMapGetBiosDriveNumber(Drive2.as_ptr())) as usize);
            }
        }
        Index += 1;
    }

    if DriveMapList.DriveMapCount != 0 {
        TRACE!("Installing Int13 drive map for %d drives.\n", (DriveMapList.DriveMapCount) as usize);
        DriveMapInstallInt13Handler(&mut DriveMapList);
    } else {
        TRACE!("Removing any previously installed Int13 drive map.\n");
        DriveMapRemoveInt13Handler();
    }
}

#[cfg(target_arch = "x86")]
pub unsafe fn DriveMapInstallInt13Handler(DriveMap: PDRIVE_MAP_LIST) {
    let RealModeIVT = UlongToPtr(0x00000000) as *mut ULONG;
    let BiosLowMemorySize = ULongToPtr(0x00000413) as *mut USHORT;

    if DriveMapInstalled == 0 {
        OldInt13HandlerAddress = *RealModeIVT.offset(0x13);

        *BiosLowMemorySize = (*BiosLowMemorySize).wrapping_sub(1);

        DriveMapHandlerAddress = (*BiosLowMemorySize as ULONG) << 10;

        DriveMapHandlerSegOff = (DriveMapHandlerAddress << 12) & 0xffff0000;
    }

    RtlCopyMemory(
        &mut DriveMapInt13HandlerMapList as *mut _ as PVOID,
        DriveMap as PVOID,
        core::mem::size_of::<DRIVE_MAP_LIST>(),
    );

    DriveMapOldInt13HandlerAddress = OldInt13HandlerAddress;

    RtlCopyMemory(
        UlongToPtr(DriveMapHandlerAddress),
        &DriveMapInt13HandlerStart as *const _ as PVOID,
        (&DriveMapInt13HandlerEnd as *const _ as PUCHAR)
            .offset_from(&DriveMapInt13HandlerStart as *const _ as PUCHAR) as usize,
    );

    *RealModeIVT.offset(0x13) = DriveMapHandlerSegOff;

    CacheInvalidateCacheData();
    DriveMapInstalled = 1;
}

#[cfg(target_arch = "x86")]
unsafe fn DriveMapRemoveInt13Handler() {
    let RealModeIVT = 0x00000000 as *mut ULONG;
    let BiosLowMemorySize = 0x00000413 as *mut USHORT;

    if DriveMapInstalled != 0 {
        *RealModeIVT.offset(0x13) = OldInt13HandlerAddress;
        *BiosLowMemorySize = (*BiosLowMemorySize).wrapping_add(1);
        CacheInvalidateCacheData();
        DriveMapInstalled = 0u8;
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
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(DISK);
//
// #ifdef _M_IX86
//
// BOOLEAN      DriveMapInstalled = FALSE;    // Tells us if we have already installed our drive map int 13h handler code
// ULONG        OldInt13HandlerAddress = 0;   // Address of BIOS int 13h handler
// ULONG        DriveMapHandlerAddress = 0;   // Linear address of our drive map handler
// ULONG        DriveMapHandlerSegOff = 0;    // Segment:offset style address of our drive map handler
//
// #endif // _M_IX86
//
// BOOLEAN DriveMapIsValidDriveString(PCSTR DriveString)
// {
//     ULONG Index;
//
//     // Now verify that the user has given us appropriate strings
//     if ((strlen(DriveString) < 3) ||
//         ((DriveString[0] != 'f') && (DriveString[0] != 'F') &&
//          (DriveString[0] != 'h') && (DriveString[0] != 'H')) ||
//         ((DriveString[1] != 'd') && (DriveString[1] != 'D')))
//     {
//         return FALSE;
//     }
//
//     // Now verify that the user has given us appropriate numbers
//     // Make sure that only numeric characters were given
//     for (Index = 2; Index < strlen(DriveString); Index++)
//     {
//         if (DriveString[Index] < '0' || DriveString[Index] > '9')
//         {
//             return FALSE;
//         }
//     }
//
//     // Now make sure that they are not outrageous values (i.e. hd90874)
//     if ((atoi(&DriveString[2]) < 0) || (atoi(&DriveString[2]) > 0xff))
//     {
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// UCHAR DriveMapGetBiosDriveNumber(PCSTR DeviceName)
// {
//     UCHAR BiosDriveNumber = 0;
//
//     TRACE("DriveMapGetBiosDriveNumber(%s)\n", (DeviceName) as usize);
//
//     // If they passed in a number string then just
//     // convert it to decimal and return it
//     if (DeviceName[0] >= '0' && DeviceName[0] <= '9')
//     {
//         return (UCHAR)strtoul(DeviceName, NULL, 0);
//     }
//
//     // Convert the drive number string into a number
//     // 'hd1' = 1
//     BiosDriveNumber = atoi(&DeviceName[2]);
//
//     // If it's a hard disk then set the high bit
//     if ((DeviceName[0] == 'h' || DeviceName[0] == 'H') &&
//         (DeviceName[1] == 'd' || DeviceName[1] == 'D'))
//     {
//         BiosDriveNumber |= 0x80;
//     }
//
//     return BiosDriveNumber;
// }
//
// #ifdef _M_IX86
//
// VOID
// DriveMapMapDrivesInSection(
//     IN ULONG_PTR SectionId)
// {
//     CHAR           SettingName[80];
//     CHAR           SettingValue[80];
//     CHAR           Drive1[80];
//     CHAR           Drive2[80];
//     ULONG          SectionItemCount;
//     ULONG          Index;
//     ULONG          Index2;
//     DRIVE_MAP_LIST DriveMapList;
//
//     if (SectionId == 0)
//         return;
//
//     RtlZeroMemory(&DriveMapList, sizeof(DRIVE_MAP_LIST));
//
//     // Get the number of items in this section
//     SectionItemCount = IniGetNumSectionItems(SectionId);
//
//     // Loop through each one and check if its a DriveMap= setting
//     for (Index=0; Index<SectionItemCount; Index++)
//     {
//         // Get the next setting from the .ini file section
//         if (IniReadSettingByNumber(SectionId, Index, SettingName, sizeof(SettingName), SettingValue, sizeof(SettingValue)))
//         {
//             if (_stricmp(SettingName, "DriveMap") == 0)
//             {
//                 // Make sure we haven't exceeded the drive map max count
//                 if (DriveMapList.DriveMapCount >= 4)
//                 {
//                     UiMessageBox("Max DriveMap count exceeded in section [%s]:\n\n%s=%s", ((PINI_SECTION)SectionId)->SectionName, SettingName, SettingValue);
//                     continue;
//                 }
//
//                 RtlZeroMemory(Drive1, 80);
//                 RtlZeroMemory(Drive2, 80);
//
//                 strcpy(Drive1, SettingValue);
//
//                 // Parse the setting value and separate a string "hd0,hd1"
//                 // into two strings "hd0" and "hd1"
//                 for (Index2=0; Index2<strlen(Drive1); Index2++)
//                 {
//                     // Check if this character is the separater character (comma - ',')
//                     if (Drive1[Index2] == ',')
//                     {
//                         Drive1[Index2] = '\0';
//                         strcpy(Drive2, &Drive1[Index2+1]);
//                         break;
//                     }
//                 }
//
//                 // Make sure we got good values before we add them to the map
//                 if (!DriveMapIsValidDriveString(Drive1) || !DriveMapIsValidDriveString(Drive2))
//                 {
//                     UiMessageBox("Error in DriveMap setting in section [%s]:\n\n%s=%s", ((PINI_SECTION)SectionId)->SectionName, SettingName, SettingValue);
//                     continue;
//                 }
//
//                 // Add them to the map
//                 DriveMapList.DriveMap[(DriveMapList.DriveMapCount * 2)] = DriveMapGetBiosDriveNumber(Drive1);
//                 DriveMapList.DriveMap[(DriveMapList.DriveMapCount * 2)+1] = DriveMapGetBiosDriveNumber(Drive2);
//                 DriveMapList.DriveMapCount++;
//
//                 TRACE("Mapping BIOS drive 0x%x to drive 0x%x\n", (DriveMapGetBiosDriveNumber(Drive1)) as usize, (DriveMapGetBiosDriveNumber(Drive2)) as usize);
//             }
//         }
//     }
//
//     if (DriveMapList.DriveMapCount)
//     {
//         TRACE("Installing Int13 drive map for %d drives.\n", (DriveMapList.DriveMapCount) as usize);
//         DriveMapInstallInt13Handler(&DriveMapList);
//     }
//     else
//     {
//         TRACE("Removing any previously installed Int13 drive map.\n");
//         DriveMapRemoveInt13Handler();
//     }
// }
//
// VOID DriveMapInstallInt13Handler(PDRIVE_MAP_LIST DriveMap)
// {
//     ULONG*  RealModeIVT = (ULONG*)UlongToPtr(0x00000000);
//     USHORT* BiosLowMemorySize = (USHORT*)ULongToPtr(0x00000413);
//
//
//     if (!DriveMapInstalled)
//     {
//         // Get the old INT 13h handler address from the vector table
//         OldInt13HandlerAddress = RealModeIVT[0x13];
//
//         // Decrease the size of low memory
//         (*BiosLowMemorySize)--;
//
//         // Get linear address for drive map handler
//         DriveMapHandlerAddress = (ULONG)(*BiosLowMemorySize) << 10;
//
//         // Convert to segment:offset style address
//         DriveMapHandlerSegOff = (DriveMapHandlerAddress << 12) & 0xffff0000;
//     }
//
//     // Copy the drive map structure to the proper place
//     RtlCopyMemory(&DriveMapInt13HandlerMapList, DriveMap, sizeof(DRIVE_MAP_LIST));
//
//     // Set the address of the BIOS INT 13h handler
//     DriveMapOldInt13HandlerAddress = OldInt13HandlerAddress;
//
//     // Copy the code to our reserved area
//     RtlCopyMemory(UlongToPtr(DriveMapHandlerAddress),
//                   &DriveMapInt13HandlerStart,
//                   ((PUCHAR)&DriveMapInt13HandlerEnd - (PUCHAR)&DriveMapInt13HandlerStart));
//
//     // Update the IVT
//     RealModeIVT[0x13] = DriveMapHandlerSegOff;
//
//     CacheInvalidateCacheData();
//     DriveMapInstalled = TRUE;
// }
//
// VOID DriveMapRemoveInt13Handler(VOID)
// {
//     ULONG*  RealModeIVT = (ULONG*)0x00000000;
//     USHORT* BiosLowMemorySize = (USHORT*)0x00000413;
//
//
//     if (DriveMapInstalled)
//     {
//         // Get the old INT 13h handler address from the vector table
//         RealModeIVT[0x13] = OldInt13HandlerAddress;
//
//         // Increase the size of low memory
//         (*BiosLowMemorySize)++;
//
//         CacheInvalidateCacheData();
//         DriveMapInstalled = FALSE;
//     }
// }
//
// #endif // _M_IX86
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
