#[allow(unused_imports)] use crate::{ERR, FIXME, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_DISK; // partition.c: DBG_DEFAULT_CHANNEL(DISK)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use core::ffi::c_void;
extern "C" {
}

// --- partition.c file-local content (rustconv drops non-fn content). ---
const MaxDriveNumber: usize = 0xFF;
static mut DiskPartitionType: [PARTITION_STYLE; MaxDriveNumber + 1] =
    [PARTITION_STYLE_MBR; MaxDriveNumber + 1];

const XBOX_SIGNATURE_SECTOR: ULONGLONG = 3;
const XBOX_SIGNATURE: ULONG = (b'B' as ULONG)
    | ((b'R' as ULONG) << 8)
    | ((b'F' as ULONG) << 16)
    | ((b'R' as ULONG) << 24);
const FATX_DATA_PARTITION: ULONG = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct XBOX_PARTITION_TABLE_ENTRY {
    SectorCountBeforePartition: ULONG,
    PartitionSectorCount: ULONG,
    SystemIndicator: UCHAR,
}
static XboxPartitions: [XBOX_PARTITION_TABLE_ENTRY; 5] = [
    XBOX_PARTITION_TABLE_ENTRY { SectorCountBeforePartition: 0x0055F400, PartitionSectorCount: 0x0098F800, SystemIndicator: PARTITION_FAT32 },
    XBOX_PARTITION_TABLE_ENTRY { SectorCountBeforePartition: 0x00465400, PartitionSectorCount: 0x000FA000, SystemIndicator: PARTITION_FAT_16 },
    XBOX_PARTITION_TABLE_ENTRY { SectorCountBeforePartition: 0x00000400, PartitionSectorCount: 0x00177000, SystemIndicator: PARTITION_FAT_16 },
    XBOX_PARTITION_TABLE_ENTRY { SectorCountBeforePartition: 0x00177400, PartitionSectorCount: 0x00177000, SystemIndicator: PARTITION_FAT_16 },
    XBOX_PARTITION_TABLE_ENTRY { SectorCountBeforePartition: 0x002EE400, PartitionSectorCount: 0x00177000, SystemIndicator: PARTITION_FAT_16 },
];

extern "C" {
    fn UefiGetGptPartitionEntry( DriveNumber: UCHAR, PartitionNumber: ULONG, PartitionTableEntry: PPARTITION_TABLE_ENTRY, ) -> BOOLEAN;
}

unsafe fn DiskReadBootRecord(
    DriveNumber: UCHAR,
    LogicalSectorNumber: ULONGLONG,
    BootRecord: PMASTER_BOOT_RECORD,
) -> BOOLEAN {
    let mut Index: ULONG;

    if MachDiskReadLogicalSectors(DriveNumber, LogicalSectorNumber, 1, DiskReadBuffer) == 0 {
        return 0u8;
    }
    core::ptr::copy_nonoverlapping(
        DiskReadBuffer as *const u8,
        BootRecord as *mut u8,
        core::mem::size_of::<MASTER_BOOT_RECORD>(),
    );

    TRACE!("Dumping partition table for drive 0x%x:\n", (DriveNumber) as usize);
    TRACE!("Boot record logical start sector = %d\n", (LogicalSectorNumber) as usize);
    TRACE!("sizeof(MASTER_BOOT_RECORD) = 0x%x.\n", (core::mem::size_of::<MASTER_BOOT_RECORD>()) as usize);

    Index = 0;
    while Index < 4 {
        TRACE!("-------------------------------------------\n");
        TRACE!("Partition %d\n", (Index + 1) as usize);
        TRACE!("BootIndicator: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].BootIndicator) as usize);
        TRACE!("StartHead: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].StartHead) as usize);
        TRACE!("StartSector (Plus 2 cylinder bits): 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].StartSector) as usize);
        TRACE!("StartCylinder: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].StartCylinder) as usize);
        TRACE!("SystemIndicator: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].SystemIndicator) as usize);
        TRACE!("EndHead: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].EndHead) as usize);
        TRACE!("EndSector (Plus 2 cylinder bits): 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].EndSector) as usize);
        TRACE!("EndCylinder: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].EndCylinder) as usize);
        TRACE!("SectorCountBeforePartition: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].SectorCountBeforePartition) as usize);
        TRACE!("PartitionSectorCount: 0x%x\n", ((*BootRecord).PartitionTable[Index as usize].PartitionSectorCount) as usize);
        Index += 1;
    }

    if (*BootRecord).MasterBootRecordMagic == 0xaa55 { 1u8 } else { 0u8 }
}

unsafe fn DiskGetFirstPartitionEntry(
    MasterBootRecord: PMASTER_BOOT_RECORD,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY
) -> BOOLEAN {
    let _Index: ULONG;

    for Index in 0..4 {
        if ((*MasterBootRecord).PartitionTable[Index as usize].SystemIndicator != PARTITION_ENTRY_UNUSED) &&
           ((*MasterBootRecord).PartitionTable[Index as usize].SystemIndicator != PARTITION_EXTENDED) &&
           ((*MasterBootRecord).PartitionTable[Index as usize].SystemIndicator != PARTITION_XINT13_EXTENDED) {
            RtlCopyMemory(
                PartitionTableEntry as *mut core::ffi::c_void,
                core::ptr::addr_of!((*MasterBootRecord).PartitionTable[Index as usize]) as *const core::ffi::c_void,
                core::mem::size_of::<PARTITION_TABLE_ENTRY>() as SIZE_T
            );
            return 1u8;
        }
    }

    return 0u8;
}

unsafe fn DiskGetFirstExtendedPartitionEntry(
    MasterBootRecord: PMASTER_BOOT_RECORD,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
) -> BOOLEAN {
    let mut Index: ULONG;

    Index = 0;
    while Index < 4 {
        if ((*MasterBootRecord).PartitionTable[Index as usize].SystemIndicator == PARTITION_EXTENDED)
            || ((*MasterBootRecord).PartitionTable[Index as usize].SystemIndicator == PARTITION_XINT13_EXTENDED)
        {
            RtlCopyMemory(
                PartitionTableEntry as *mut core::ffi::c_void,
                core::ptr::addr_of!((*MasterBootRecord).PartitionTable[Index as usize]) as *const core::ffi::c_void,
                core::mem::size_of::<PARTITION_TABLE_ENTRY>() as SIZE_T,
            );
            return 1u8;
        }
        Index += 1;
    }

    return 0u8;
}

unsafe fn DiskGetActivePartitionEntry(
    DriveNumber: UCHAR,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
    ActivePartition: PULONG,
) -> BOOLEAN {
    let mut BootablePartitionCount: ULONG = 0;
    let mut CurrentPartitionNumber: ULONG;
    let mut Index: ULONG;
    let mut MasterBootRecord: MASTER_BOOT_RECORD = core::mem::zeroed();
    let mut ThisPartitionTableEntry: PPARTITION_TABLE_ENTRY;

    *ActivePartition = 0;

    if DiskReadBootRecord(DriveNumber, 0, &mut MasterBootRecord) == 0 {
        return 0u8;
    }

    CurrentPartitionNumber = 0;
    Index = 0;
    while Index < 4 {
        ThisPartitionTableEntry = &mut MasterBootRecord.PartitionTable[Index as usize];

        if (*ThisPartitionTableEntry).SystemIndicator != PARTITION_ENTRY_UNUSED &&
            (*ThisPartitionTableEntry).SystemIndicator != PARTITION_EXTENDED &&
            (*ThisPartitionTableEntry).SystemIndicator != PARTITION_XINT13_EXTENDED
        {
            CurrentPartitionNumber += 1;

            if (*ThisPartitionTableEntry).BootIndicator == 0x80 {
                BootablePartitionCount += 1;
                *ActivePartition = CurrentPartitionNumber;

                core::ptr::copy_nonoverlapping(
                    ThisPartitionTableEntry as *const u8,
                    PartitionTableEntry as *mut u8,
                    core::mem::size_of::<PARTITION_TABLE_ENTRY>(),
                );
            }
        }

        Index += 1;
    }

    if BootablePartitionCount == 0 {
        ERR!("No bootable (active) partitions found.\n");
        return 0u8;
    } else if BootablePartitionCount != 1 {
        ERR!("Too many bootable (active) partitions found.\n");
        return 0u8;
    }

    return 1u8;
}

unsafe fn DiskGetMbrPartitionEntry(
    DriveNumber: UCHAR,
    PartitionNumber: ULONG,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
) -> BOOLEAN {
    let mut MasterBootRecord: MASTER_BOOT_RECORD = core::mem::zeroed();
    let mut ExtendedPartitionTableEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();
    let ExtendedPartitionNumber: ULONG;
    let mut ExtendedPartitionOffset: ULONG;
    let mut Index: ULONG;
    let mut CurrentPartitionNumber: ULONG;
    let mut ThisPartitionTableEntry: PPARTITION_TABLE_ENTRY;

    if DiskReadBootRecord(DriveNumber, 0, core::ptr::addr_of_mut!(MasterBootRecord)) == 0 {
        return 0u8;
    }

    CurrentPartitionNumber = 0;
    Index = 0;
    while Index < 4 {
        ThisPartitionTableEntry = core::ptr::addr_of_mut!(MasterBootRecord.PartitionTable[Index as usize]);

        if (*ThisPartitionTableEntry).SystemIndicator != PARTITION_ENTRY_UNUSED &&
            (*ThisPartitionTableEntry).SystemIndicator != PARTITION_EXTENDED &&
            (*ThisPartitionTableEntry).SystemIndicator != PARTITION_XINT13_EXTENDED
        {
            CurrentPartitionNumber += 1;
        }

        if PartitionNumber == CurrentPartitionNumber {
            RtlCopyMemory(
                PartitionTableEntry as *mut core::ffi::c_void,
                ThisPartitionTableEntry as *const core::ffi::c_void,
                core::mem::size_of::<PARTITION_TABLE_ENTRY>() as SIZE_T,
            );
            return 1u8;
        }

        Index += 1;
    }

    ExtendedPartitionNumber = PartitionNumber - CurrentPartitionNumber - 1;

    ExtendedPartitionOffset = 0;

    Index = 0;
    while Index <= ExtendedPartitionNumber {
        if DiskGetFirstExtendedPartitionEntry(
            core::ptr::addr_of_mut!(MasterBootRecord),
            core::ptr::addr_of_mut!(ExtendedPartitionTableEntry),
        ) == 0
        {
            return 0u8;
        }

        ExtendedPartitionTableEntry.SectorCountBeforePartition += ExtendedPartitionOffset;
        if ExtendedPartitionOffset == 0 {
            ExtendedPartitionOffset = ExtendedPartitionTableEntry.SectorCountBeforePartition;
        }

        if DiskReadBootRecord(
            DriveNumber,
            ExtendedPartitionTableEntry.SectorCountBeforePartition as ULONGLONG,
            core::ptr::addr_of_mut!(MasterBootRecord),
        ) == 0
        {
            return 0u8;
        }

        if DiskGetFirstPartitionEntry(
            core::ptr::addr_of_mut!(MasterBootRecord),
            PartitionTableEntry,
        ) == 0
        {
            return 0u8;
        }

        (*PartitionTableEntry).SectorCountBeforePartition += ExtendedPartitionTableEntry.SectorCountBeforePartition;

        Index += 1;
    }

    return 1u8;
}

unsafe fn DiskGetBrfrPartitionEntry(
    DriveNumber: UCHAR,
    PartitionNumber: ULONG,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
) -> BOOLEAN {
    if PartitionNumber >= 1 
        && PartitionNumber <= (core::mem::size_of_val(&XboxPartitions) / core::mem::size_of::<XBOX_PARTITION_TABLE_ENTRY>()) as ULONG
        && MachDiskReadLogicalSectors(DriveNumber, XBOX_SIGNATURE_SECTOR, 1, DiskReadBuffer) != 0
    {
        if *(DiskReadBuffer as PULONG) != XBOX_SIGNATURE {
            return 0u8;
        }

        RtlZeroMemory(PartitionTableEntry as PVOID, core::mem::size_of::<PARTITION_TABLE_ENTRY>());
        (*PartitionTableEntry).SystemIndicator = XboxPartitions[(PartitionNumber - 1) as usize].SystemIndicator;
        (*PartitionTableEntry).SectorCountBeforePartition = XboxPartitions[(PartitionNumber - 1) as usize].SectorCountBeforePartition;
        (*PartitionTableEntry).PartitionSectorCount = XboxPartitions[(PartitionNumber - 1) as usize].PartitionSectorCount;
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn DiskDetectPartitionType(DriveNumber: UCHAR) {
    let mut MasterBootRecord: MASTER_BOOT_RECORD = core::mem::zeroed();
    let mut Index: ULONG;
    let mut PartitionCount: ULONG = 0;
    let mut ThisPartitionTableEntry: PPARTITION_TABLE_ENTRY;
    let mut GPTProtect: BOOLEAN = 0u8;
    let mut PartitionTableEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();

    if DiskReadBootRecord(DriveNumber, 0, &mut MasterBootRecord) != 0 {
        DiskPartitionType[DriveNumber as usize] = PARTITION_STYLE_MBR;

        Index = 0;
        while Index < 4 {
            ThisPartitionTableEntry = &mut MasterBootRecord.PartitionTable[Index as usize];

            if (*ThisPartitionTableEntry).SystemIndicator != PARTITION_ENTRY_UNUSED {
                PartitionCount += 1;

                if Index == 0 && (*ThisPartitionTableEntry).SystemIndicator == PARTITION_GPT {
                    GPTProtect = 1u8;
                }
            }
            Index += 1;
        }

        if PartitionCount == 1 && GPTProtect != 0 {
            DiskPartitionType[DriveNumber as usize] = PARTITION_STYLE_GPT;
        }
        TRACE!("Drive 0x%X partition type %s\n", (DriveNumber) as usize, (if DiskPartitionType[DriveNumber as usize] == PARTITION_STYLE_MBR {
                b"MBR\0".as_ptr() as PCHAR
            } else {
                b"GPT\0".as_ptr() as PCHAR
            }) as usize);
        return;
    }

    if DiskGetBrfrPartitionEntry(DriveNumber, FATX_DATA_PARTITION, &mut PartitionTableEntry) != 0 {
        DiskPartitionType[DriveNumber as usize] = PARTITION_STYLE_BRFR;
        TRACE!("Drive 0x%X partition type Xbox-BRFR\n", (DriveNumber) as usize);
        return;
    }

    DiskPartitionType[DriveNumber as usize] = PARTITION_STYLE_RAW;
    TRACE!("Drive 0x%X partition type unknown\n", (DriveNumber) as usize);
}

#[no_mangle]
pub unsafe extern "C" fn DiskGetBootPartitionEntry(
    DriveNumber: UCHAR,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
    BootPartition: PULONG,
) -> BOOLEAN {
    match DiskPartitionType[DriveNumber as usize] {
        PARTITION_STYLE_MBR => {
            return DiskGetActivePartitionEntry(DriveNumber, PartitionTableEntry, BootPartition);
        }
        PARTITION_STYLE_GPT => {
            FIXME!("DiskGetBootPartitionEntry() unimplemented for GPT\n");
            return 0u8;
        }
        PARTITION_STYLE_RAW => {
            FIXME!("DiskGetBootPartitionEntry() unimplemented for RAW\n");
            return 0u8;
        }
        PARTITION_STYLE_BRFR => {
            if DiskGetBrfrPartitionEntry(DriveNumber, FATX_DATA_PARTITION, PartitionTableEntry) != 0 {
                *BootPartition = FATX_DATA_PARTITION;
                return 1u8;
            }
            return 0u8;
        }
        _ => {
            ERR!("Drive 0x%X partition type = %d, should not happen!\n", (DriveNumber) as usize, (DiskPartitionType[DriveNumber as usize]) as usize);
            ntkrnl_sys::ASSERT!(false);
        }
    }
    0u8
}

#[no_mangle]
pub unsafe extern "C" fn DiskGetPartitionEntry(
    DriveNumber: UCHAR,
    PartitionNumber: ULONG,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
) -> BOOLEAN {
    match DiskPartitionType[DriveNumber as usize] {
        PARTITION_STYLE_MBR => {
            return DiskGetMbrPartitionEntry(DriveNumber, PartitionNumber, PartitionTableEntry);
        }
        PARTITION_STYLE_GPT => {
            #[cfg(feature = "uefi")]
            {
                return UefiGetGptPartitionEntry(DriveNumber, PartitionNumber, PartitionTableEntry);
            }
            #[cfg(not(feature = "uefi"))]
            {
                FIXME!("DiskGetPartitionEntry() unimplemented for GPT\n");
                return 0u8;
            }
        }
        PARTITION_STYLE_RAW => {
            FIXME!("DiskGetPartitionEntry() unimplemented for RAW\n");
            return 0u8;
        }
        PARTITION_STYLE_BRFR => {
            return DiskGetBrfrPartitionEntry(DriveNumber, PartitionNumber, PartitionTableEntry);
        }
        _ => {
            ERR!("Drive 0x%X partition type = %d, should not happen!\n", (DriveNumber) as usize, (DiskPartitionType[DriveNumber as usize]) as usize);
            ntkrnl_sys::ASSERT!(false);
        }
    }
    0u8
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
// #ifndef _M_ARM
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(DISK);
//
// #define MaxDriveNumber 0xFF
// static PARTITION_STYLE DiskPartitionType[MaxDriveNumber + 1];
//
// /* BRFR signature at disk offset 0x600 */
// #define XBOX_SIGNATURE_SECTOR 3
// #define XBOX_SIGNATURE        ('B' | ('R' << 8) | ('F' << 16) | ('R' << 24))
//
// /* Default hardcoded partition number to boot from Xbox disk */
// #define FATX_DATA_PARTITION 1
//
// static struct
// {
//     ULONG SectorCountBeforePartition;
//     ULONG PartitionSectorCount;
//     UCHAR SystemIndicator;
// } XboxPartitions[] =
// {
//     /* This is in the \Device\Harddisk0\Partition.. order used by the Xbox kernel */
//     { 0x0055F400, 0x0098F800, PARTITION_FAT32  }, /* Store , E: */
//     { 0x00465400, 0x000FA000, PARTITION_FAT_16 }, /* System, C: */
//     { 0x00000400, 0x00177000, PARTITION_FAT_16 }, /* Cache1, X: */
//     { 0x00177400, 0x00177000, PARTITION_FAT_16 }, /* Cache2, Y: */
//     { 0x002EE400, 0x00177000, PARTITION_FAT_16 }  /* Cache3, Z: */
// };
//
// static BOOLEAN
// DiskReadBootRecord(
//     IN UCHAR DriveNumber,
//     IN ULONGLONG LogicalSectorNumber,
//     OUT PMASTER_BOOT_RECORD BootRecord)
// {
//     ULONG Index;
//
//     /* Read master boot record */
//     if (!MachDiskReadLogicalSectors(DriveNumber, LogicalSectorNumber, 1, DiskReadBuffer))
//     {
//         return FALSE;
//     }
//     RtlCopyMemory(BootRecord, DiskReadBuffer, sizeof(MASTER_BOOT_RECORD));
//
//     TRACE("Dumping partition table for drive 0x%x:\n", (DriveNumber) as usize);
//     TRACE("Boot record logical start sector = %d\n", (LogicalSectorNumber) as usize);
//     TRACE("sizeof(MASTER_BOOT_RECORD) = 0x%x.\n", (sizeof(MASTER_BOOT_RECORD)) as usize);
//
//     for (Index = 0; Index < 4; Index++)
//     {
//         TRACE("-------------------------------------------\n");
//         TRACE("Partition %d\n", ((Index + 1)) as usize);
//         TRACE("BootIndicator: 0x%x\n", (BootRecord->PartitionTable[Index].BootIndicator) as usize);
//         TRACE("StartHead: 0x%x\n", (BootRecord->PartitionTable[Index].StartHead) as usize);
//         TRACE("StartSector (Plus 2 cylinder bits): 0x%x\n", (BootRecord->PartitionTable[Index].StartSector) as usize);
//         TRACE("StartCylinder: 0x%x\n", (BootRecord->PartitionTable[Index].StartCylinder) as usize);
//         TRACE("SystemIndicator: 0x%x\n", (BootRecord->PartitionTable[Index].SystemIndicator) as usize);
//         TRACE("EndHead: 0x%x\n", (BootRecord->PartitionTable[Index].EndHead) as usize);
//         TRACE("EndSector (Plus 2 cylinder bits): 0x%x\n", (BootRecord->PartitionTable[Index].EndSector) as usize);
//         TRACE("EndCylinder: 0x%x\n", (BootRecord->PartitionTable[Index].EndCylinder) as usize);
//         TRACE("SectorCountBeforePartition: 0x%x\n", (BootRecord->PartitionTable[Index].SectorCountBeforePartition) as usize);
//         TRACE("PartitionSectorCount: 0x%x\n", (BootRecord->PartitionTable[Index].PartitionSectorCount) as usize);
//     }
//
//     /* Check the partition table magic value */
//     return (BootRecord->MasterBootRecordMagic == 0xaa55);
// }
//
// static BOOLEAN
// DiskGetFirstPartitionEntry(
//     IN PMASTER_BOOT_RECORD MasterBootRecord,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry)
// {
//     ULONG Index;
//
//     for (Index = 0; Index < 4; Index++)
//     {
//         /* Check the system indicator. If it's not an extended or unused partition then we're done. */
//         if ((MasterBootRecord->PartitionTable[Index].SystemIndicator != PARTITION_ENTRY_UNUSED) &&
//             (MasterBootRecord->PartitionTable[Index].SystemIndicator != PARTITION_EXTENDED) &&
//             (MasterBootRecord->PartitionTable[Index].SystemIndicator != PARTITION_XINT13_EXTENDED))
//         {
//             RtlCopyMemory(PartitionTableEntry, &MasterBootRecord->PartitionTable[Index], sizeof(PARTITION_TABLE_ENTRY));
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// static BOOLEAN
// DiskGetFirstExtendedPartitionEntry(
//     IN PMASTER_BOOT_RECORD MasterBootRecord,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry)
// {
//     ULONG Index;
//
//     for (Index = 0; Index < 4; Index++)
//     {
//         /* Check the system indicator. If it an extended partition then we're done. */
//         if ((MasterBootRecord->PartitionTable[Index].SystemIndicator == PARTITION_EXTENDED) ||
//             (MasterBootRecord->PartitionTable[Index].SystemIndicator == PARTITION_XINT13_EXTENDED))
//         {
//             RtlCopyMemory(PartitionTableEntry, &MasterBootRecord->PartitionTable[Index], sizeof(PARTITION_TABLE_ENTRY));
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// static BOOLEAN
// DiskGetActivePartitionEntry(
//     IN UCHAR DriveNumber,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry,
//     OUT PULONG ActivePartition)
// {
//     ULONG BootablePartitionCount = 0;
//     ULONG CurrentPartitionNumber;
//     ULONG Index;
//     MASTER_BOOT_RECORD MasterBootRecord;
//     PPARTITION_TABLE_ENTRY ThisPartitionTableEntry;
//
//     *ActivePartition = 0;
//
//     /* Read master boot record */
//     if (!DiskReadBootRecord(DriveNumber, 0, &MasterBootRecord))
//     {
//         return FALSE;
//     }
//
//     CurrentPartitionNumber = 0;
//     for (Index = 0; Index < 4; Index++)
//     {
//         ThisPartitionTableEntry = &MasterBootRecord.PartitionTable[Index];
//
//         if (ThisPartitionTableEntry->SystemIndicator != PARTITION_ENTRY_UNUSED &&
//             ThisPartitionTableEntry->SystemIndicator != PARTITION_EXTENDED &&
//             ThisPartitionTableEntry->SystemIndicator != PARTITION_XINT13_EXTENDED)
//         {
//             CurrentPartitionNumber++;
//
//             /* Test if this is the bootable partition */
//             if (ThisPartitionTableEntry->BootIndicator == 0x80)
//             {
//                 BootablePartitionCount++;
//                 *ActivePartition = CurrentPartitionNumber;
//
//                 /* Copy the partition table entry */
//                 RtlCopyMemory(PartitionTableEntry,
//                               ThisPartitionTableEntry,
//                               sizeof(PARTITION_TABLE_ENTRY));
//             }
//         }
//     }
//
//     /* Make sure there was only one bootable partition */
//     if (BootablePartitionCount == 0)
//     {
//         ERR("No bootable (active) partitions found.\n");
//         return FALSE;
//     }
//     else if (BootablePartitionCount != 1)
//     {
//         ERR("Too many bootable (active) partitions found.\n");
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static BOOLEAN
// DiskGetMbrPartitionEntry(
//     IN UCHAR DriveNumber,
//     IN ULONG PartitionNumber,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry)
// {
//     MASTER_BOOT_RECORD MasterBootRecord;
//     PARTITION_TABLE_ENTRY ExtendedPartitionTableEntry;
//     ULONG ExtendedPartitionNumber;
//     ULONG ExtendedPartitionOffset;
//     ULONG Index;
//     ULONG CurrentPartitionNumber;
//     PPARTITION_TABLE_ENTRY ThisPartitionTableEntry;
//
//     /* Read master boot record */
//     if (!DiskReadBootRecord(DriveNumber, 0, &MasterBootRecord))
//     {
//         return FALSE;
//     }
//
//     CurrentPartitionNumber = 0;
//     for (Index = 0; Index < 4; Index++)
//     {
//         ThisPartitionTableEntry = &MasterBootRecord.PartitionTable[Index];
//
//         if (ThisPartitionTableEntry->SystemIndicator != PARTITION_ENTRY_UNUSED &&
//             ThisPartitionTableEntry->SystemIndicator != PARTITION_EXTENDED &&
//             ThisPartitionTableEntry->SystemIndicator != PARTITION_XINT13_EXTENDED)
//         {
//             CurrentPartitionNumber++;
//         }
//
//         if (PartitionNumber == CurrentPartitionNumber)
//         {
//             RtlCopyMemory(PartitionTableEntry, ThisPartitionTableEntry, sizeof(PARTITION_TABLE_ENTRY));
//             return TRUE;
//         }
//     }
//
//     /*
//      * They want an extended partition entry so we will need
//      * to loop through all the extended partitions on the disk
//      * and return the one they want.
//      */
//     ExtendedPartitionNumber = PartitionNumber - CurrentPartitionNumber - 1;
//
//     /*
//      * Set the initial relative starting sector to 0.
//      * This is because extended partition starting
//      * sectors a numbered relative to their parent.
//      */
//     ExtendedPartitionOffset = 0;
//
//     for (Index = 0; Index <= ExtendedPartitionNumber; Index++)
//     {
//         /* Get the extended partition table entry */
//         if (!DiskGetFirstExtendedPartitionEntry(&MasterBootRecord, &ExtendedPartitionTableEntry))
//         {
//             return FALSE;
//         }
//
//         /* Adjust the relative starting sector of the partition */
//         ExtendedPartitionTableEntry.SectorCountBeforePartition += ExtendedPartitionOffset;
//         if (ExtendedPartitionOffset == 0)
//         {
//             /* Set the start of the parrent extended partition */
//             ExtendedPartitionOffset = ExtendedPartitionTableEntry.SectorCountBeforePartition;
//         }
//         /* Read the partition boot record */
//         if (!DiskReadBootRecord(DriveNumber, ExtendedPartitionTableEntry.SectorCountBeforePartition, &MasterBootRecord))
//         {
//             return FALSE;
//         }
//
//         /* Get the first real partition table entry */
//         if (!DiskGetFirstPartitionEntry(&MasterBootRecord, PartitionTableEntry))
//         {
//             return FALSE;
//         }
//
//         /* Now correct the start sector of the partition */
//         PartitionTableEntry->SectorCountBeforePartition += ExtendedPartitionTableEntry.SectorCountBeforePartition;
//     }
//
//     /*
//      * When we get here we should have the correct entry already
//      * stored in PartitionTableEntry, so just return TRUE.
//      */
//     return TRUE;
// }
//
// static BOOLEAN
// DiskGetBrfrPartitionEntry(
//     IN UCHAR DriveNumber,
//     IN ULONG PartitionNumber,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry)
// {
//     /*
//      * Get partition entry of an Xbox-standard BRFR partitioned disk.
//      */
//     if (PartitionNumber >= 1 && PartitionNumber <= sizeof(XboxPartitions) / sizeof(XboxPartitions[0]) &&
//         MachDiskReadLogicalSectors(DriveNumber, XBOX_SIGNATURE_SECTOR, 1, DiskReadBuffer))
//     {
//         if (*((PULONG)DiskReadBuffer) != XBOX_SIGNATURE)
//         {
//             /* No magic Xbox partitions */
//             return FALSE;
//         }
//
//         RtlZeroMemory(PartitionTableEntry, sizeof(PARTITION_TABLE_ENTRY));
//         PartitionTableEntry->SystemIndicator = XboxPartitions[PartitionNumber - 1].SystemIndicator;
//         PartitionTableEntry->SectorCountBeforePartition = XboxPartitions[PartitionNumber - 1].SectorCountBeforePartition;
//         PartitionTableEntry->PartitionSectorCount = XboxPartitions[PartitionNumber - 1].PartitionSectorCount;
//         return TRUE;
//     }
//
//     /* Partition does not exist */
//     return FALSE;
// }
//
// VOID
// DiskDetectPartitionType(
//     IN UCHAR DriveNumber)
// {
//     MASTER_BOOT_RECORD MasterBootRecord;
//     ULONG Index;
//     ULONG PartitionCount = 0;
//     PPARTITION_TABLE_ENTRY ThisPartitionTableEntry;
//     BOOLEAN GPTProtect = FALSE;
//     PARTITION_TABLE_ENTRY PartitionTableEntry;
//
//     /* Probe for Master Boot Record */
//     if (DiskReadBootRecord(DriveNumber, 0, &MasterBootRecord))
//     {
//         DiskPartitionType[DriveNumber] = PARTITION_STYLE_MBR;
//
//         /* Check for GUID Partition Table */
//         for (Index = 0; Index < 4; Index++)
//         {
//             ThisPartitionTableEntry = &MasterBootRecord.PartitionTable[Index];
//
//             if (ThisPartitionTableEntry->SystemIndicator != PARTITION_ENTRY_UNUSED)
//             {
//                 PartitionCount++;
//
//                 if (Index == 0 && ThisPartitionTableEntry->SystemIndicator == PARTITION_GPT)
//                 {
//                     GPTProtect = TRUE;
//                 }
//             }
//         }
//
//         if (PartitionCount == 1 && GPTProtect)
//         {
//             DiskPartitionType[DriveNumber] = PARTITION_STYLE_GPT;
//         }
//         TRACE("Drive 0x%X partition type %s\n", (DriveNumber) as usize, (DiskPartitionType[DriveNumber] == PARTITION_STYLE_MBR ? "MBR" : "GPT") as usize);
//         return;
//     }
//
//     /* Probe for Xbox-BRFR partitioning */
//     if (DiskGetBrfrPartitionEntry(DriveNumber, FATX_DATA_PARTITION, &PartitionTableEntry))
//     {
//         DiskPartitionType[DriveNumber] = PARTITION_STYLE_BRFR;
//         TRACE("Drive 0x%X partition type Xbox-BRFR\n", (DriveNumber) as usize);
//         return;
//     }
//
//     /* Failed to detect partitions, assume partitionless disk */
//     DiskPartitionType[DriveNumber] = PARTITION_STYLE_RAW;
//     TRACE("Drive 0x%X partition type unknown\n", (DriveNumber) as usize);
// }
//
// BOOLEAN
// DiskGetBootPartitionEntry(
//     IN UCHAR DriveNumber,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry,
//     OUT PULONG BootPartition)
// {
//     switch (DiskPartitionType[DriveNumber])
//     {
//         case PARTITION_STYLE_MBR:
//         {
//             return DiskGetActivePartitionEntry(DriveNumber, PartitionTableEntry, BootPartition);
//         }
//         case PARTITION_STYLE_GPT:
//         {
//             FIXME("DiskGetBootPartitionEntry() unimplemented for GPT\n");
//             return FALSE;
//         }
//         case PARTITION_STYLE_RAW:
//         {
//             FIXME("DiskGetBootPartitionEntry() unimplemented for RAW\n");
//             return FALSE;
//         }
//         case PARTITION_STYLE_BRFR:
//         {
//             if (DiskGetBrfrPartitionEntry(DriveNumber, FATX_DATA_PARTITION, PartitionTableEntry))
//             {
//                 *BootPartition = FATX_DATA_PARTITION;
//                 return TRUE;
//             }
//             return FALSE;
//         }
//         default:
//         {
//             ERR("Drive 0x%X partition type = %d, should not happen!\n", (DriveNumber) as usize, (DiskPartitionType[DriveNumber]) as usize);
//             ASSERT(FALSE);
//         }
//     }
//     return FALSE;
// }
//
// BOOLEAN
// DiskGetPartitionEntry(
//     IN UCHAR DriveNumber,
//     IN ULONG PartitionNumber,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry)
// {
//     switch (DiskPartitionType[DriveNumber])
//     {
//         case PARTITION_STYLE_MBR:
//         {
//             return DiskGetMbrPartitionEntry(DriveNumber, PartitionNumber, PartitionTableEntry);
//         }
//         case PARTITION_STYLE_GPT:
//         {
// #ifdef UEFIBOOT
//             BOOLEAN UefiGetGptPartitionEntry(
//                 IN UCHAR DriveNumber,
//                 IN ULONG PartitionNumber,
//                 OUT PPARTITION_TABLE_ENTRY PartitionTableEntry);
//             return UefiGetGptPartitionEntry(DriveNumber, PartitionNumber, PartitionTableEntry);
// #else
//             FIXME("DiskGetPartitionEntry() unimplemented for GPT\n");
//             return FALSE;
// #endif
//         }
//         case PARTITION_STYLE_RAW:
//         {
//             FIXME("DiskGetPartitionEntry() unimplemented for RAW\n");
//             return FALSE;
//         }
//         case PARTITION_STYLE_BRFR:
//         {
//             return DiskGetBrfrPartitionEntry(DriveNumber, PartitionNumber, PartitionTableEntry);
//         }
//         default:
//         {
//             ERR("Drive 0x%X partition type = %d, should not happen!\n", (DriveNumber) as usize, (DiskPartitionType[DriveNumber]) as usize);
//             ASSERT(FALSE);
//         }
//     }
//     return FALSE;
// }
//
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
