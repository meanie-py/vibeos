use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, DRIVE_LAYOUT_INFORMATION, ExAllocatePool, ExFreePool, LARGE_INTEGER, PARTITION_INFORMATION, PDEVICE_OBJECT, PDRIVE_LAYOUT_INFORMATION, PRKEVENT, PSTRING, _LOADER_PARAMETER_BLOCK};
// freeldr_ddk_imports dropped (not in ddk): Type
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_SEEKMODE_SeekAbsolute as SeekAbsolute;
pub use ntkrnl_sys::ddk::_POOL_TYPE_NonPagedPool as NonPagedPool;
pub use ntkrnl_sys::ddk::EVENT_TYPE;

// ntstatus.h
pub const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC00000BBu32 as NTSTATUS;
pub const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as NTSTATUS;

extern "C" {
}
// freeldr.spec declares these cdecl — use extern "C" (NOT extern "system")
// to match the cdecl decoration (no `@N` suffix) emitted by freeldr.sys.
extern "C" {
    fn ArcSeek(FileId: usize, Position: *mut LARGE_INTEGER, SeekMode: u32) -> ARC_STATUS;
    fn ArcRead(FileId: usize, Buffer: *mut core::ffi::c_void, Length: u32, Count: *mut u32) -> ARC_STATUS;
    fn StallExecutionProcessor(MicroSeconds: ULONG);
}

// Ke* stubs MUST be extern "system" (NTAPI stdcall on i386 → `_Name@N`
// decoration). Callers in rosload's winldr/bootmgr declare these via the
// NTAPI macro and link expects the stdcall-decorated symbol.
#[no_mangle]
pub unsafe extern "system" fn KeInitializeEvent(
    Event: PRKEVENT,
    _Type: EVENT_TYPE,
    _State: BOOLEAN,
) {
    core::ptr::write_bytes(Event, 0, 1);
}

#[no_mangle]
pub unsafe extern "system" fn KeSetTimeIncrement(
    _MaxIncrement: ULONG,
    _MinIncrement: ULONG,
) {
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IoAssignDriveLetters(
    LoaderBlock: *mut _LOADER_PARAMETER_BLOCK,
    NtDeviceName: PSTRING,
    NtSystemPath: PUCHAR,
    NtSystemPathString: PSTRING,
) {
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IoSetPartitionInformation(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: u32,
    PartitionNumber: u32,
    PartitionType: u32,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "C" fn IopReadBootRecord(
    DeviceObject: PDEVICE_OBJECT,
    LogicalSectorNumber: u64,
    SectorSize: u32,
    BootRecord: *mut core::ffi::c_void,
) -> i32 {

    let FileId = DeviceObject as usize;
    let mut Position = LARGE_INTEGER { QuadPart: (LogicalSectorNumber * SectorSize as u64) as i64 };
    let mut BytesRead: u32 = 0;
    let mut Status: ARC_STATUS;

    Status = ArcSeek(FileId, &mut Position, SeekAbsolute);
    if Status != ESUCCESS {
        return STATUS_IO_DEVICE_ERROR;
    }

    Status = ArcRead(FileId, BootRecord, SectorSize, &mut BytesRead);
    if Status != ESUCCESS || BytesRead != SectorSize {
        return STATUS_IO_DEVICE_ERROR;
    }

    STATUS_SUCCESS
}

unsafe fn IopCopyPartitionRecord(
    ReturnRecognizedPartitions: BOOLEAN,
    SectorSize: ULONG,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
    PartitionEntry: *mut PARTITION_INFORMATION,
) -> BOOLEAN {
    let IsRecognized: BOOLEAN;

    IsRecognized = 1u8;
    if IsRecognized == 0 && ReturnRecognizedPartitions != 0 {
        return 0u8;
    }

    (*PartitionEntry).StartingOffset.QuadPart = 
        ((*PartitionTableEntry).SectorCountBeforePartition as ULONGLONG)
            .wrapping_mul(SectorSize as ULONGLONG) as LONGLONG;
    (*PartitionEntry).PartitionLength.QuadPart = 
        ((*PartitionTableEntry).PartitionSectorCount as ULONGLONG)
            .wrapping_mul(SectorSize as ULONGLONG) as LONGLONG;
    (*PartitionEntry).HiddenSectors = 0;
    (*PartitionEntry).PartitionNumber = 0;
    (*PartitionEntry).PartitionType = (*PartitionTableEntry).SystemIndicator;
    (*PartitionEntry).BootIndicator = if (*PartitionTableEntry).BootIndicator & 0x80 != 0 { 1u8 } else { 0u8 };
    (*PartitionEntry).RecognizedPartition = IsRecognized;
    (*PartitionEntry).RewritePartition = 0u8;

    1u8
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IoReadPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    ReturnRecognizedPartitions: BOOLEAN,
    PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let mut MasterBootRecord: PMASTER_BOOT_RECORD;
    let mut Partitions: PDRIVE_LAYOUT_INFORMATION;
    let mut NbPartitions: ULONG;
    let mut i: ULONG;
    let mut Size: ULONG;

    *PartitionBuffer = core::ptr::null_mut();

    if SectorSize < core::mem::size_of::<MASTER_BOOT_RECORD>() as ULONG {
        return STATUS_NOT_SUPPORTED;
    }

    MasterBootRecord = ExAllocatePool(NonPagedPool, SectorSize) as PMASTER_BOOT_RECORD;
    if MasterBootRecord.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = IopReadBootRecord(DeviceObject, 0, SectorSize, MasterBootRecord as *mut core::ffi::c_void);
    if !NT_SUCCESS(Status) {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return Status;
    }

    if (*MasterBootRecord).MasterBootRecordMagic != 0xaa55 {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    NbPartitions = 0;
    i = 0;
    while i < 4 {
        NbPartitions += 1;

        if (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_EXTENDED ||
           (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_XINT13_EXTENDED {
            unimplemented!();
        }
        i += 1;
    }

    if NbPartitions == 0 {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    Size = core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION, PartitionEntry) as ULONG +
           NbPartitions * core::mem::size_of::<PARTITION_INFORMATION>() as ULONG;
    Partitions = ExAllocatePool(NonPagedPool, Size) as PDRIVE_LAYOUT_INFORMATION;
    if Partitions.is_null() {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return STATUS_NO_MEMORY;
    }

    NbPartitions = 0;
    i = 0;
    while i < 4 {
        if IopCopyPartitionRecord(
            ReturnRecognizedPartitions,
            SectorSize,
            &mut (*MasterBootRecord).PartitionTable[i as usize],
            &mut (*Partitions).PartitionEntry[NbPartitions as usize],
        ) != 0 {
            (*Partitions).PartitionEntry[NbPartitions as usize].PartitionNumber = NbPartitions + 1;
            NbPartitions += 1;
        }

        if (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_EXTENDED ||
           (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_XINT13_EXTENDED {
            unimplemented!();
        }
        i += 1;
    }

    (*Partitions).PartitionCount = NbPartitions;
    (*Partitions).Signature = (*MasterBootRecord).Signature;
    ExFreePool(MasterBootRecord as *mut core::ffi::c_void);

    *PartitionBuffer = Partitions;
    STATUS_SUCCESS
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IoReadPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    ReturnRecognizedPartitions: BOOLEAN,
    PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {

    let Status: NTSTATUS;
    let MasterBootRecord: PMASTER_BOOT_RECORD;
    let Partitions: PDRIVE_LAYOUT_INFORMATION;
    let mut NbPartitions: ULONG;
    let mut i: ULONG;
    let Size: ULONG;

    *PartitionBuffer = core::ptr::null_mut();

    if SectorSize < core::mem::size_of::<MASTER_BOOT_RECORD>() as ULONG {
        return STATUS_NOT_SUPPORTED;
    }

    MasterBootRecord = ExAllocatePool(NonPagedPool, SectorSize as _) as PMASTER_BOOT_RECORD;
    if MasterBootRecord.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = IopReadBootRecord(DeviceObject, 0, SectorSize, MasterBootRecord as *mut core::ffi::c_void);
    if !NT_SUCCESS(Status) {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return Status;
    }

    if (*MasterBootRecord).MasterBootRecordMagic != 0xaa55 {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    NbPartitions = 0;
    i = 0;
    while i < 4 {
        NbPartitions += 1;

        if (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_EXTENDED ||
           (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_XINT13_EXTENDED {
            unimplemented!();
        }
        i += 1;
    }

    if NbPartitions == 0 {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    Size = core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION, PartitionEntry) as ULONG +
           NbPartitions * core::mem::size_of::<PARTITION_INFORMATION>() as ULONG;
    Partitions = ExAllocatePool(NonPagedPool, Size as _) as PDRIVE_LAYOUT_INFORMATION;
    if Partitions.is_null() {
        ExFreePool(MasterBootRecord as *mut core::ffi::c_void);
        return STATUS_NO_MEMORY;
    }

    NbPartitions = 0;
    i = 0;
    while i < 4 {
        if IopCopyPartitionRecord(
            ReturnRecognizedPartitions,
            SectorSize,
            &mut (*MasterBootRecord).PartitionTable[i as usize],
            &mut (*Partitions).PartitionEntry[NbPartitions as usize],
        ) != 0 {
            (*Partitions).PartitionEntry[NbPartitions as usize].PartitionNumber = NbPartitions + 1;
            NbPartitions += 1;
        }

        if (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_EXTENDED ||
           (*MasterBootRecord).PartitionTable[i as usize].SystemIndicator == PARTITION_XINT13_EXTENDED {
            unimplemented!();
        }
        i += 1;
    }

    (*Partitions).PartitionCount = NbPartitions;
    (*Partitions).Signature = (*MasterBootRecord).Signature;
    ExFreePool(MasterBootRecord as *mut core::ffi::c_void);

    *PartitionBuffer = Partitions;
    STATUS_SUCCESS
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IoWritePartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: u32,
    SectorsPerTrack: u32,
    NumberOfHeads: u32,
    PartitionBuffer: PDRIVE_LAYOUT_INFORMATION,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KeStallExecutionProcessor(
    MicroSeconds: ULONG,
) {
    StallExecutionProcessor(MicroSeconds);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/arch/i386/ntoskrnl.c
//  * PURPOSE:         NTOS glue routines for the MINIHAL library
//  * PROGRAMMERS:     Hervé Poussineau  <hpoussin@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <freeldr.h>
// #include <ntoskrnl.h>
//
// #ifndef UNIMPLEMENTED
// #define UNIMPLEMENTED ASSERT(FALSE)
// #endif
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// KeInitializeEvent(
//     IN PRKEVENT Event,
//     IN EVENT_TYPE Type,
//     IN BOOLEAN State)
// {
//     RtlZeroMemory(Event, sizeof(*Event));
// }
//
// VOID
// NTAPI
// KeSetTimeIncrement(
//     IN ULONG MaxIncrement,
//     IN ULONG MinIncrement)
// {
// }
//
// VOID
// FASTCALL
// IoAssignDriveLetters(
//     IN struct _LOADER_PARAMETER_BLOCK *LoaderBlock,
//     IN PSTRING NtDeviceName,
//     OUT PUCHAR NtSystemPath,
//     OUT PSTRING NtSystemPathString)
// {
// }
//
// NTSTATUS
// FASTCALL
// IoSetPartitionInformation(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN ULONG SectorSize,
//     IN ULONG PartitionNumber,
//     IN ULONG PartitionType)
// {
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// #ifndef _M_AMD64
// NTSTATUS
// NTAPI
// IopReadBootRecord(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN ULONGLONG LogicalSectorNumber,
//     IN ULONG SectorSize,
//     OUT PMASTER_BOOT_RECORD BootRecord)
// {
//     ULONG_PTR FileId = (ULONG_PTR)DeviceObject;
//     LARGE_INTEGER Position;
//     ULONG BytesRead;
//     ARC_STATUS Status;
//
//     Position.QuadPart = LogicalSectorNumber * SectorSize;
//     Status = ArcSeek(FileId, &Position, SeekAbsolute);
//     if (Status != ESUCCESS)
//         return STATUS_IO_DEVICE_ERROR;
//
//     Status = ArcRead(FileId, BootRecord, SectorSize, &BytesRead);
//     if (Status != ESUCCESS || BytesRead != SectorSize)
//         return STATUS_IO_DEVICE_ERROR;
//
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// IopCopyPartitionRecord(
//     IN BOOLEAN ReturnRecognizedPartitions,
//     IN ULONG SectorSize,
//     IN PPARTITION_TABLE_ENTRY PartitionTableEntry,
//     OUT PARTITION_INFORMATION *PartitionEntry)
// {
//     BOOLEAN IsRecognized;
//
//     IsRecognized = TRUE; /* FIXME */
//     if (!IsRecognized && ReturnRecognizedPartitions)
//         return FALSE;
//
//     PartitionEntry->StartingOffset.QuadPart = (ULONGLONG)PartitionTableEntry->SectorCountBeforePartition * SectorSize;
//     PartitionEntry->PartitionLength.QuadPart = (ULONGLONG)PartitionTableEntry->PartitionSectorCount * SectorSize;
//     PartitionEntry->HiddenSectors = 0;
//     PartitionEntry->PartitionNumber = 0; /* Will be filled later */
//     PartitionEntry->PartitionType = PartitionTableEntry->SystemIndicator;
//     PartitionEntry->BootIndicator = (PartitionTableEntry->BootIndicator & 0x80) ? TRUE : FALSE;
//     PartitionEntry->RecognizedPartition = IsRecognized;
//     PartitionEntry->RewritePartition = FALSE;
//
//     return TRUE;
// }
//
// NTSTATUS
// FASTCALL
// IoReadPartitionTable(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN ULONG SectorSize,
//     IN BOOLEAN ReturnRecognizedPartitions,
//     OUT PDRIVE_LAYOUT_INFORMATION *PartitionBuffer)
// {
//     NTSTATUS Status;
//     PMASTER_BOOT_RECORD MasterBootRecord;
//     PDRIVE_LAYOUT_INFORMATION Partitions;
//     ULONG NbPartitions, i, Size;
//
//     *PartitionBuffer = NULL;
//
//     if (SectorSize < sizeof(MASTER_BOOT_RECORD))
//         return STATUS_NOT_SUPPORTED;
//
//     MasterBootRecord = ExAllocatePool(NonPagedPool, SectorSize);
//     if (!MasterBootRecord)
//         return STATUS_NO_MEMORY;
//
//     /* Read disk MBR */
//     Status = IopReadBootRecord(DeviceObject, 0, SectorSize, MasterBootRecord);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePool(MasterBootRecord);
//         return Status;
//     }
//
//     /* Check validity of boot record */
//     if (MasterBootRecord->MasterBootRecordMagic != 0xaa55)
//     {
//         ExFreePool(MasterBootRecord);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     /* Count number of partitions */
//     NbPartitions = 0;
//     for (i = 0; i < 4; i++)
//     {
//         NbPartitions++;
//
//         if (MasterBootRecord->PartitionTable[i].SystemIndicator == PARTITION_EXTENDED ||
//             MasterBootRecord->PartitionTable[i].SystemIndicator == PARTITION_XINT13_EXTENDED)
//         {
//             /* FIXME: unhandled case; count number of partitions */
//             UNIMPLEMENTED;
//         }
//     }
//
//     if (NbPartitions == 0)
//     {
//         ExFreePool(MasterBootRecord);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     /* Allocation space to store partitions */
//     Size = crate::FIELD_OFFSET!(DRIVE_LAYOUT_INFORMATION, PartitionEntry) +
//            NbPartitions * sizeof(PARTITION_INFORMATION);
//     Partitions = ExAllocatePool(NonPagedPool, Size);
//     if (!Partitions)
//     {
//         ExFreePool(MasterBootRecord);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Count number of partitions */
//     NbPartitions = 0;
//     for (i = 0; i < 4; i++)
//     {
//         if (IopCopyPartitionRecord(ReturnRecognizedPartitions,
//                                    SectorSize,
//                                    &MasterBootRecord->PartitionTable[i],
//                                    &Partitions->PartitionEntry[NbPartitions]))
//         {
//             Partitions->PartitionEntry[NbPartitions].PartitionNumber = NbPartitions + 1;
//             NbPartitions++;
//         }
//
//         if (MasterBootRecord->PartitionTable[i].SystemIndicator == PARTITION_EXTENDED ||
//             MasterBootRecord->PartitionTable[i].SystemIndicator == PARTITION_XINT13_EXTENDED)
//         {
//             /* FIXME: unhandled case; copy partitions */
//             UNIMPLEMENTED;
//         }
//     }
//
//     Partitions->PartitionCount = NbPartitions;
//     Partitions->Signature = MasterBootRecord->Signature;
//     ExFreePool(MasterBootRecord);
//
//     *PartitionBuffer = Partitions;
//     return STATUS_SUCCESS;
// }
// #endif // _M_AMD64
//
// NTSTATUS
// FASTCALL
// IoWritePartitionTable(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN ULONG SectorSize,
//     IN ULONG SectorsPerTrack,
//     IN ULONG NumberOfHeads,
//     IN PDRIVE_LAYOUT_INFORMATION PartitionBuffer)
// {
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// VOID
// NTAPI
// KeStallExecutionProcessor(
//     IN ULONG MicroSeconds)
// {
//     StallExecutionProcessor(MicroSeconds);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
