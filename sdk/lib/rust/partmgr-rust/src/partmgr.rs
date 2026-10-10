// ---- crate-internal glue (private structs, GUID externs, no-op DDK shims) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::guids::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CREATE_DISK, DEVICE_RELATIONS, DISK_GEOMETRY_EX, DRIVE_LAYOUT_INFORMATION, DRIVE_LAYOUT_INFORMATION_EX, ExFreePool, IoAttachDeviceToDeviceStack, IoCreateDevice, IoCreateDisk, IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, IoInvalidateDeviceRelations, IoReadPartitionTableEx, IoRegisterDeviceInterface, IoReportTargetDeviceChangeAsynchronous, IoSetDeviceInterfaceState, IoWritePartitionTableEx, KeInitializeEvent, PARTITION_INFORMATION, PARTITION_INFORMATION_EX, PCREATE_DISK, PDEVICE_OBJECT, PDEVICE_RELATIONS, PDRIVER_OBJECT, PDRIVE_LAYOUT_INFORMATION, PDRIVE_LAYOUT_INFORMATION_EX, PIO_STACK_LOCATION, PIRP, PPARTITION_INFORMATION, PPARTITION_INFORMATION_EX, PUNICODE_STRING, RtlFreeUnicodeString, RtlInitUnicodeString, STORAGE_DEVICE_NUMBER, _EVENT_TYPE_SynchronizationEvent, TARGET_DEVICE_CUSTOM_NOTIFICATION, UINT32};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{CREATE_DISK_MBR, DISK_PARTITION_INFO, FILE_OBJECT, GUID, PARTITION_STYLE, PCSTR, PDRIVER_ADD_DEVICE, PDRIVER_DISPATCH, PDRIVER_UNLOAD, RtlAssert, _CREATE_DISK, _DPFLTR_TYPE_DPFLTR_DISK_ID as DPFLTR_DISK_ID, _DRIVE_LAYOUT_INFORMATION, _DRIVE_LAYOUT_INFORMATION_EX, _LARGE_INTEGER, _PARTITION_INFORMATION, _PARTITION_INFORMATION_EX__bindgen_ty_1, _PARTITION_INFORMATION_MBR, _STORAGE_DEVICE_NUMBER, _UNICODE_STRING};

extern "C" {
    // KeGetCurrentIrql comes from ntkrnl_sys::ke (FORCEINLINE on amd64/arm64, not an
    // export there) — declared above; the LLM's local phantom extern is removed.
    fn IssueSyncIoControlRequest(
        IoControlCode: u32,
        DeviceObject: PDEVICE_OBJECT,
        InputBuffer: *mut core::ffi::c_void,
        InputBufferLength: u32,
        OutputBuffer: *mut core::ffi::c_void,
        OutputBufferLength: u32,
        Wait: u8, /* BOOLEAN */
    ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// (LLM emitted bogus `static mut PartMgrPower/PartMgrShutdownFlush: Option<DRIVER_DISPATCH>`
// here, duplicating the real dispatch functions of the same name below — dropped.)
#[no_mangle]
pub unsafe extern "C" fn PartMgrConvertExtendedToLayout(
    LayoutEx: PDRIVE_LAYOUT_INFORMATION_EX,
) -> PDRIVE_LAYOUT_INFORMATION {
    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    .as_ptr() as PVOID,
                27 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    // ASSERT(LayoutEx)
    if LayoutEx.is_null() {
        RtlAssert(
            b"LayoutEx\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            29 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    if (*LayoutEx).PartitionStyle != PARTITION_STYLE_MBR {
        // ASSERT(FALSE)
        if 0 == 0 {
            RtlAssert(
                b"FALSE\0".as_ptr() as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    .as_ptr() as PVOID,
                33 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
        return core::ptr::null_mut::<DRIVE_LAYOUT_INFORMATION>();
    }

    let layout_size: usize = 8usize.wrapping_add(
        ((*LayoutEx).PartitionCount as usize)
            .wrapping_mul(core::mem::size_of::<PARTITION_INFORMATION>()),
    );

    let layout: PDRIVE_LAYOUT_INFORMATION = ExAllocatePoolWithTag(
        PagedPool,
        layout_size as SIZE_T,
        TAG_PARTMGR as ULONG,
    ) as PDRIVE_LAYOUT_INFORMATION;

    if layout.is_null() {
        return core::ptr::null_mut::<DRIVE_LAYOUT_INFORMATION>();
    }

    (*layout).Signature = (*LayoutEx).__bindgen_anon_1.Mbr.Signature;
    (*layout).PartitionCount = (*LayoutEx).PartitionCount;

    let mut i: u32 = 0;
    while (i as u64) < ((*LayoutEx).PartitionCount) as u64 {
        let partition = &mut *(*layout).PartitionEntry.as_mut_ptr().offset(i as isize);
        let partition_ex = &* (*LayoutEx).PartitionEntry.as_ptr().offset(i as isize);

        (*partition).StartingOffset = (*partition_ex).StartingOffset;
        (*partition).PartitionLength = (*partition_ex).PartitionLength;
        (*partition).RewritePartition = (*partition_ex).RewritePartition;
        (*partition).PartitionNumber = (*partition_ex).PartitionNumber;
        (*partition).PartitionType = (*partition_ex).__bindgen_anon_1.Mbr.PartitionType;
        (*partition).BootIndicator = (*partition_ex).__bindgen_anon_1.Mbr.BootIndicator;
        (*partition).RecognizedPartition = (*partition_ex).__bindgen_anon_1.Mbr.RecognizedPartition;
        (*partition).HiddenSectors = (*partition_ex).__bindgen_anon_1.Mbr.HiddenSectors;

        i = i.wrapping_add(1);
    }

    layout
}
#[no_mangle]
pub unsafe extern "C" fn PartMgrConvertLayoutToExtended(
    Layout: PDRIVE_LAYOUT_INFORMATION,
) -> PDRIVE_LAYOUT_INFORMATION_EX {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            (KeGetCurrentIrql()) as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    .as_ptr() as PVOID,
                77u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if Layout.is_null() {
        RtlAssert(
            b"Layout != NULL\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            79u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    let layoutSize = core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry)
        + (*Layout).PartitionCount as usize * core::mem::size_of::<PARTITION_INFORMATION_EX>();
    let layoutEx = ExAllocatePoolUninitialized(PagedPool, layoutSize as _, TAG_PARTMGR)
        as PDRIVE_LAYOUT_INFORMATION_EX;
    if layoutEx.is_null() {
        return core::ptr::null_mut();
    }
    (*layoutEx).PartitionStyle = PARTITION_STYLE_MBR as ULONG;
    (*layoutEx).PartitionCount = (*Layout).PartitionCount;
    (*layoutEx).__bindgen_anon_1.Mbr.Signature = (*Layout).Signature;
    for i in 0..(*Layout).PartitionCount {
        let part = &*(*Layout).PartitionEntry.as_ptr().add(i as usize);
        let entry_ptr = (*layoutEx).PartitionEntry.as_mut_ptr().add(i as usize);
        *entry_ptr = PARTITION_INFORMATION_EX {
            PartitionStyle: PARTITION_STYLE_MBR,
            StartingOffset: (*part).StartingOffset,
            PartitionLength: (*part).PartitionLength,
            RewritePartition: (*part).RewritePartition,
            PartitionNumber: (*part).PartitionNumber,
            __bindgen_anon_1: _PARTITION_INFORMATION_EX__bindgen_ty_1 {
                Mbr: _PARTITION_INFORMATION_MBR {
                    PartitionType: (*part).PartitionType,
                    BootIndicator: (*part).BootIndicator,
                    RecognizedPartition: (*part).RecognizedPartition,
                    HiddenSectors: (*part).HiddenSectors,
                },
            },
        };
    }
    layoutEx
}
#[no_mangle]
pub unsafe extern "C" fn PartMgrIsDiskSuperFloppy(FdoExtension: PFDO_EXTENSION) -> BOOLEAN {
    // PAGED_CODE expansion
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            (KeGetCurrentIrql()) as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            140u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // Original assertions
    if !( (*FdoExtension).LayoutValid != 0 && !(*FdoExtension).LayoutCache.is_null() ) {
        RtlAssert(
            b"FdoExtension->LayoutValid && FdoExtension->LayoutCache\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            142u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if (*FdoExtension).DiskData.PartitionStyle as u32
        != (*(*FdoExtension).LayoutCache).PartitionStyle as u32
    {
        RtlAssert(
            b"FdoExtension->DiskData.PartitionStyle == FdoExtension->LayoutCache->PartitionStyle\0"
                .as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            145u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // Style must be MBR
    if (*FdoExtension).DiskData.PartitionStyle as u32 != PARTITION_STYLE_MBR as u32 {
        return 0u8;
    }
    // Only one partition
    if (*(*FdoExtension).LayoutCache).PartitionCount != 1u32 {
        return 0u8;
    }

    // Get pointer to the single partition entry
    let PartitionInfo: PPARTITION_INFORMATION_EX =
        (&raw mut (*(*FdoExtension).LayoutCache).PartitionEntry).cast();

    // PartitionStyle assertion
    if (*FdoExtension).DiskData.PartitionStyle as u32 != (*PartitionInfo).PartitionStyle as u32 {
        RtlAssert(
            b"FdoExtension->DiskData.PartitionStyle == PartitionInfo->PartitionStyle\0"
                .as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            153u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // Partition must start at beginning of disk (offset 0, hidden sectors 0)
    if !((*PartitionInfo).StartingOffset.QuadPart == 0i64
        && (*PartitionInfo).__bindgen_anon_1.Mbr.HiddenSectors == 0u32)
    {
        return 0u8;
    }

    // MBR signature assert
    if (*FdoExtension).DiskData.__bindgen_anon_1.Mbr.as_ref().Signature as u32
        != (*(*FdoExtension).LayoutCache).__bindgen_anon_1.Mbr.Signature as u32
    {
        RtlAssert(
            b"FdoExtension->DiskData.Mbr.Signature == FdoExtension->LayoutCache->Mbr.Signature\0"
                .as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            163u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // Warn if signature not 1
    if (*FdoExtension).DiskData.__bindgen_anon_1.Mbr.as_ref().Signature != 1u32 {
        DbgPrintEx(
            DPFLTR_DISK_ID as u32 as ULONG,
            DPFLTR_WARNING_LEVEL as ULONG,
            b"(%s:%d) Super-Floppy disk %lu signature %08x != 1!\n\0".as_ptr() as PCSTR,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as *const core::ffi::c_char,
            167i32,
            (*FdoExtension).DiskData.DeviceNumber,
            (*FdoExtension).DiskData.__bindgen_anon_1.Mbr.as_ref().Signature,
        );
    }

    // Check partition properties: recognized, type FAT16, not bootable
    if (*PartitionInfo).__bindgen_anon_1.Mbr.RecognizedPartition as i32 != (TRUE) as i32
        || (*PartitionInfo).__bindgen_anon_1.Mbr.PartitionType as i32 != (PARTITION_FAT_16) as i32
        || (*PartitionInfo).__bindgen_anon_1.Mbr.BootIndicator as i32 != (FALSE) as i32
    {
        let rec_str_1: *const core::ffi::c_char;
        if (*PartitionInfo).__bindgen_anon_1.Mbr.RecognizedPartition as i32 != 0 {
            rec_str_1 = b"TRUE\0".as_ptr() as *const core::ffi::c_char;
        } else {
            rec_str_1 = b"FALSE\0".as_ptr() as *const core::ffi::c_char;
        }
        let boot_str_1: *const core::ffi::c_char;
        if (*PartitionInfo).__bindgen_anon_1.Mbr.BootIndicator as i32 != 0 {
            boot_str_1 = b"TRUE\0".as_ptr() as *const core::ffi::c_char;
        } else {
            boot_str_1 = b"FALSE\0".as_ptr() as *const core::ffi::c_char;
        }
        DbgPrintEx(
            DPFLTR_DISK_ID as u32 as ULONG,
            DPFLTR_WARNING_LEVEL as ULONG,
            b"(%s:%d) Super-Floppy disk %lu does not return default settings!\n    RecognizedPartition = %s, expected TRUE\n    PartitionType = 0x%02x, expected 0x04 (PARTITION_FAT_16)\n    BootIndicator = %s, expected FALSE\n\0"
                .as_ptr() as PCSTR,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as *const core::ffi::c_char,
            182i32,
            (*FdoExtension).DiskData.DeviceNumber,
            rec_str_1,
            (*PartitionInfo).__bindgen_anon_1.Mbr.PartitionType as i32,
            boot_str_1,
        );
    }

    // Partition size should match disk size
    if (*PartitionInfo).PartitionLength.QuadPart as u64 != (*FdoExtension).DiskData.DiskSize {
        DbgPrintEx(
            DPFLTR_DISK_ID as u32 as ULONG,
            DPFLTR_WARNING_LEVEL as ULONG,
            b"(%s:%d) PartitionLength = %I64u is different from DiskSize = %I64u\n\0"
                .as_ptr() as PCSTR,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as *const core::ffi::c_char,
            189i32,
            (*PartitionInfo).PartitionLength.QuadPart,
            (*FdoExtension).DiskData.DiskSize,
        );
    }

    return 1u8;
}
#[no_mangle]
pub unsafe extern "C" fn PartMgrUpdatePartitionDevices(
    FdoExtension: PFDO_EXTENSION,
    NewLayout: PDRIVE_LAYOUT_INFORMATION_EX,
) {
    let mut totalPartitions: u32 = 0;

    // Clear partition numbers
    let count = (*NewLayout).PartitionCount;
    for i in 0..count {
        let entry = &mut *(*NewLayout).PartitionEntry.as_mut_ptr().add(i as usize);
        entry.PartitionNumber = 0;
    }

    // Iterate over old partition list
    let mut prevEntry = &raw mut (*FdoExtension).PartitionList;
    let mut curEntry = (*FdoExtension).PartitionList.Next;
    while !curEntry.is_null() {
        let partExt = (curEntry as usize - core::mem::offset_of!(PARTITION_EXTENSION, ListEntry)) as PPARTITION_EXTENSION;
        let mut partNumber: u32 = 0;
        let mut found: u8 = 0;
        let mut partEntry: PPARTITION_INFORMATION_EX = core::ptr::null_mut();

        // Search old partition inside new layout
        for i in 0..count {
            partEntry = (*NewLayout).PartitionEntry.as_mut_ptr().add(i as usize);
            if (*NewLayout).PartitionStyle == PARTITION_STYLE_MBR {
                let mbr_type = (*partEntry).__bindgen_anon_1.Mbr.PartitionType;
                if mbr_type == (PARTITION_ENTRY_UNUSED) as u8 || IsContainerPartition(mbr_type) {
                    continue;
                }
            }
            partNumber = partNumber.wrapping_add(1);
            if (*partEntry).PartitionNumber != 0 {
                continue;
            }
            if (*partEntry).StartingOffset.QuadPart != ((*partExt).StartingOffset) as i64
                || (*partEntry).PartitionLength.QuadPart != ((*partExt).PartitionLength) as i64
            {
                continue;
            }
            found = 1;
            break;
        }

        if found != 0 {
            // Update metadata
            if (*NewLayout).PartitionStyle == PARTITION_STYLE_MBR {
                (*partExt).__bindgen_anon_1.Mbr.as_mut().PartitionType = (*partEntry).__bindgen_anon_1.Mbr.PartitionType;
                (*partExt).__bindgen_anon_1.Mbr.as_mut().BootIndicator = (*partEntry).__bindgen_anon_1.Mbr.BootIndicator;
            } else {
                (*partExt).__bindgen_anon_1.Gpt.as_mut().PartitionType = (*partEntry).__bindgen_anon_1.Gpt.PartitionType;
                (*partExt).__bindgen_anon_1.Gpt.as_mut().PartitionId = (*partEntry).__bindgen_anon_1.Gpt.PartitionId;
                (*partExt).__bindgen_anon_1.Gpt.as_mut().Attributes = (*partEntry).__bindgen_anon_1.Gpt.Attributes as u64;
                core::ptr::copy_nonoverlapping(
                    &raw const (*partEntry).__bindgen_anon_1.Gpt.Name as *const _,
                    &raw mut (*partExt).__bindgen_anon_1.Gpt.as_mut().Name as *mut _,
                    36, // element count (u16) — size_of here copied 2x too much
                );
            }
            (*partExt).OnDiskNumber = partNumber;
            (*partEntry).PartitionNumber = partNumber;
            totalPartitions = totalPartitions.wrapping_add(1);
            // Advance to next entry
            prevEntry = curEntry;
            curEntry = (*curEntry).Next;
        } else {
            // Remove device from list
            (*prevEntry).Next = (*curEntry).Next;
            curEntry = prevEntry;
            (*partExt).Attached = 0;
            if (*partExt).IsEnumerated == 0 {
                PartitionHandleRemove(partExt, 1);
            }
            prevEntry = curEntry;
            curEntry = (*curEntry).Next;
        }
    }

    let mut partNumber: u32 = 0;
    let mut pdoNumber: u32 = 1;

    // Process new partitions
    for i in 0..count {
        let partEntry = (*NewLayout).PartitionEntry.as_mut_ptr().add(i as usize);
        if (*NewLayout).PartitionStyle == PARTITION_STYLE_MBR {
            let mbr_type = (*partEntry).__bindgen_anon_1.Mbr.PartitionType;
            if mbr_type == (PARTITION_ENTRY_UNUSED) as u8 || IsContainerPartition(mbr_type) {
                continue;
            }
        }
        partNumber = partNumber.wrapping_add(1);
        if (*partEntry).PartitionNumber != 0 {
            continue;
        }

        // Find first free PDO index
        let mut curEntry2 = (*FdoExtension).PartitionList.Next;
        while !curEntry2.is_null() {
            let partExt2 = (curEntry2 as usize - core::mem::offset_of!(PARTITION_EXTENSION, ListEntry)) as PPARTITION_EXTENSION;
            if (*partExt2).DetectedNumber == pdoNumber {
                // Restart from the head. The C resets the cursor inside a `for` whose
                // step then runs, so its "restart" never re-checks the first entry and
                // can hand out a PDO number that entry already holds.
                curEntry2 = (*FdoExtension).PartitionList.Next;
                pdoNumber = pdoNumber.wrapping_add(1);
                continue;
            }
            curEntry2 = (*curEntry2).Next;
        }

        (*partEntry).PartitionNumber = partNumber;

        let mut partitionDevice: PDEVICE_OBJECT = core::ptr::null_mut();
        let status = PartitionCreateDevice(
            (*FdoExtension).DeviceObject,
            partEntry,
            pdoNumber,
            (*NewLayout).PartitionStyle,
            &raw mut partitionDevice,
        );
        if !NT_SUCCESS(status) {
            (*partEntry).PartitionNumber = 0;
            continue;
        }

        // Mark removable if parent device is removable
        if (*(*FdoExtension).LowerDevice).Characteristics as u32 & FILE_REMOVABLE_MEDIA as u32 != 0 {
            (*partitionDevice).Characteristics |= FILE_REMOVABLE_MEDIA as u32;
        }

        totalPartitions = totalPartitions.wrapping_add(1);

        // Insert into sorted list
        curEntry = (*FdoExtension).PartitionList.Next;
        prevEntry = core::ptr::null_mut();
        while !curEntry.is_null() {
            let curPart = (curEntry as usize - core::mem::offset_of!(PARTITION_EXTENSION, ListEntry)) as PPARTITION_EXTENSION;
            if (*curPart).OnDiskNumber >= partNumber {
                break;
            }
            prevEntry = curEntry;
            curEntry = (*curPart).ListEntry.Next;
        }

        let partExt = (*partitionDevice).DeviceExtension as PPARTITION_EXTENSION;
        if !prevEntry.is_null() {
            (*partExt).ListEntry.Next = (*prevEntry).Next;
            (*prevEntry).Next = &raw mut (*partExt).ListEntry;
        } else {
            (*partExt).ListEntry.Next = (*FdoExtension).PartitionList.Next;
            (*FdoExtension).PartitionList.Next = &raw mut (*partExt).ListEntry;
        }
        (*partExt).Attached = 1;
    }

    (*FdoExtension).EnumeratedPartitionsTotal = totalPartitions;
}
#[no_mangle]
pub unsafe extern "C" fn PartMgrGetDriveLayout(
    FdoExtension: PFDO_EXTENSION,
    DriveLayout: *mut PDRIVE_LAYOUT_INFORMATION_EX,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *mut core::ffi::c_void,
                b"drivers/storage/partmgr/partmgr.c\0"
                    as *const u8 as *mut core::ffi::c_void,
                411u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if (*FdoExtension).LayoutValid != 0 {
        *DriveLayout = (*FdoExtension).LayoutCache;
        return STATUS_SUCCESS;
    }
    let mut layoutEx: PDRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let mut status: NTSTATUS = IoReadPartitionTableEx(
        (*FdoExtension).LowerDevice,
        core::ptr::addr_of_mut!(layoutEx),
    );
    if !NT_SUCCESS(status) {
        return status;
    }
    if !(*FdoExtension).LayoutCache.is_null() {
        ExFreePool((*FdoExtension).LayoutCache as *mut core::ffi::c_void);
    }
    (*FdoExtension).LayoutCache = layoutEx;
    (*FdoExtension).LayoutValid = 1u8;
    (*FdoExtension).DiskData.PartitionStyle = (*layoutEx).PartitionStyle;
    if (*FdoExtension).DiskData.PartitionStyle as u32 == PARTITION_STYLE_MBR as u32 {
        (*FdoExtension)
            .DiskData
            .__bindgen_anon_1
            .Mbr
            .as_mut().Signature = (*layoutEx).__bindgen_anon_1.Mbr.Signature as u32;
    } else {
        (*FdoExtension)
            .DiskData
            .__bindgen_anon_1
            .Gpt
            .as_mut().DiskId = (*layoutEx).__bindgen_anon_1.Gpt.DiskId;
    }
    (*FdoExtension).IsSuperFloppy = PartMgrIsDiskSuperFloppy(FdoExtension);
    *DriveLayout = layoutEx;
    status
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskGetDriveGeometryEx(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as PVOID,
            456 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }

    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let geometryEx: PDISK_GEOMETRY_EX_INTERNAL =
        (*Irp).AssociatedIrp.SystemBuffer as PDISK_GEOMETRY_EX_INTERNAL;
    let outBufferLength: ULONG = (*ioStack).Parameters.DeviceIoControl.OutputBufferLength;
    let mut status: NTSTATUS;

    status = IssueSyncIoControlRequest(
        IOCTL_DISK_GET_DRIVE_GEOMETRY_EX as UINT32,
        (*FdoExtension).LowerDevice,
        core::ptr::null_mut(),
        0u32,
        geometryEx as PVOID,
        outBufferLength,
        0u8,  // FALSE as BOOLEAN
    );

    if !(status >= 0) {
        return status;
    }

    // Check if DISK_PARTITION_INFO fits
    if outBufferLength >= 56u32 {
        PartMgrAcquireLayoutLock(FdoExtension);

        (*geometryEx).Partition.SizeOfPartitionInfo =
            core::mem::size_of::<DISK_PARTITION_INFO>() as ULONG;
        (*geometryEx).Partition.PartitionStyle = (*FdoExtension).DiskData.PartitionStyle;

        match (*geometryEx).Partition.PartitionStyle as u32 {
            0 => {
                (*geometryEx).Partition.__bindgen_anon_1.Mbr.Signature =
                    (*FdoExtension).DiskData.__bindgen_anon_1.Mbr.as_ref().Signature as ULONG;
            }
            1 => {
                (*geometryEx).Partition.__bindgen_anon_1.Gpt.DiskId =
                    (*FdoExtension).DiskData.__bindgen_anon_1.Gpt.as_ref().DiskId;
            }
            _ => {
                RtlZeroMemory(
                    &raw mut (*geometryEx).Partition as *mut core::ffi::c_void,
                    (core::mem::size_of::<DISK_PARTITION_INFO>() as u64) as usize,
                );
            }
        }

        PartMgrReleaseLayoutLock(FdoExtension);
    }

    // Set IoStatus.Information to min(outBufferLength, sizeof(DISK_GEOMETRY_EX_INTERNAL))
    let info: usize = if (outBufferLength as usize) < core::mem::size_of::<DISK_GEOMETRY_EX_INTERNAL>() {
        outBufferLength as usize
    } else {
        core::mem::size_of::<DISK_GEOMETRY_EX_INTERNAL>()
    };

    (*Irp).IoStatus.Information = (info) as _;
    status
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskGetPartitionInfo(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let partInfo: PPARTITION_INFORMATION = (*Irp).AssociatedIrp.SystemBuffer as PPARTITION_INFORMATION;
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    as *const u8 as *const i8 as PVOID,
                519 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<_PARTITION_INFORMATION>() as SIZE_T)) == 0 {
        return STATUS_BUFFER_TOO_SMALL;
    }
    PartMgrAcquireLayoutLock(FdoExtension);
    *partInfo = _PARTITION_INFORMATION {
        StartingOffset: _LARGE_INTEGER {
            QuadPart: 0i64,
        },
        PartitionLength: _LARGE_INTEGER {
            QuadPart: (*FdoExtension).DiskData.DiskSize as i64,
        },
        HiddenSectors: 0u32,
        PartitionNumber: 0u32,
        PartitionType: (PARTITION_ENTRY_UNUSED) as u8,
        BootIndicator: 0u8,
        RecognizedPartition: 0u8,
        RewritePartition: 0u8,
    };
    PartMgrReleaseLayoutLock(FdoExtension);
    (*Irp).IoStatus.Information = (core::mem::size_of::<_PARTITION_INFORMATION>() as _);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskGetPartitionInfoEx(FdoExtension: PFDO_EXTENSION, Irp: PIRP) -> NTSTATUS {
    let partInfoEx = (*Irp).AssociatedIrp.SystemBuffer as *mut PARTITION_INFORMATION_EX;
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const u8 as *const i8 as *mut c_void,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as *const u8 as *mut c_void,
            554,
            core::ptr::null_mut(),
        );
    }
    if VerifyIrpOutBufferSize(Irp, (core::mem::size_of::<PARTITION_INFORMATION_EX>()) as usize) == 0 {
        return STATUS_BUFFER_TOO_SMALL;
    }
    PartMgrAcquireLayoutLock(FdoExtension);
    *partInfoEx = core::mem::zeroed();
    (*partInfoEx).PartitionStyle = (*FdoExtension).DiskData.PartitionStyle;
    (*partInfoEx).PartitionLength.QuadPart = ((*FdoExtension).DiskData.DiskSize) as i64;
    PartMgrReleaseLayoutLock(FdoExtension);
    (*Irp).IoStatus.Information = (core::mem::size_of::<PARTITION_INFORMATION_EX>() as _);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskGetDriveLayout(
    FdoExtension: *mut FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as u32 > (APC_LEVEL) as u32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8).cast(),
            KeGetCurrentIrql() as u32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/storage/partmgr/partmgr.c\0"
                .as_ptr() as *const i8 as PVOID,
            582,
            core::ptr::null_mut(),
        );
    }

    PartMgrAcquireLayoutLock(FdoExtension);

    let mut layoutEx: *mut DRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let status = PartMgrGetDriveLayout(FdoExtension, &mut layoutEx);

    if !NT_SUCCESS(status) {
        PartMgrReleaseLayoutLock(FdoExtension);
        return status;
    }

    if (*layoutEx).PartitionStyle != PARTITION_STYLE_MBR {
        PartMgrReleaseLayoutLock(FdoExtension);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let mut size = core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION, PartitionEntry) as usize;
    size += (*layoutEx).PartitionCount as usize * core::mem::size_of::<PARTITION_INFORMATION>();

    if VerifyIrpOutBufferSize(Irp, (size) as usize) == 0 {
        PartMgrReleaseLayoutLock(FdoExtension);
        return STATUS_BUFFER_TOO_SMALL;
    }

    let partitionList = PartMgrConvertExtendedToLayout(layoutEx);

    PartMgrReleaseLayoutLock(FdoExtension);

    if partitionList.is_null() {
        (*Irp).IoStatus.Information = 0;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        partitionList as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        size,
    );
    ExFreePoolWithTag(partitionList as PVOID, TAG_PARTMGR);

    (*Irp).IoStatus.Information = (size as _);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskGetDriveLayoutEx(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    PartMgrAcquireLayoutLock(FdoExtension);
    let mut layoutEx: *mut _DRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let mut status = PartMgrGetDriveLayout(FdoExtension, &mut layoutEx);
    if !NT_SUCCESS(status) {
        PartMgrReleaseLayoutLock(FdoExtension);
        return status;
    }
    let size: usize = 48 + ((*layoutEx).PartitionCount as usize) * core::mem::size_of::<PARTITION_INFORMATION_EX>();
    if VerifyIrpOutBufferSize(Irp, (size) as usize) == 0 {
        PartMgrReleaseLayoutLock(FdoExtension);
        return STATUS_BUFFER_TOO_SMALL;
    }
    // `size` is BYTES, as the C's RtlCopyMemory counts. Typed `*const _` made this copy
    // `size` whole DRIVE_LAYOUT_INFORMATION_EX structs (192x): diskpart's first 624-byte
    // layout query wrote ~117 KB through the caller's system buffer (BUGCHECK 0x19).
    core::ptr::copy_nonoverlapping(
        layoutEx as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        size,
    );
    PartMgrReleaseLayoutLock(FdoExtension);
    (*Irp).IoStatus.Information = (size as usize) as _;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskSetDriveLayout(FdoExtension: PFDO_EXTENSION, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE() expansion
    if unsafe { KeGetCurrentIrql() } as i32 > (APC_LEVEL) as i32 {
        unsafe {
            DbgPrint(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as PCSTR,
                KeGetCurrentIrql() as i32,
            );
            if 0 == 0 {
                RtlAssert(
                    b"0\0".as_ptr() as *const core::ffi::c_char as PVOID,
                    b"drivers/storage/partmgr/partmgr.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
                    672 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            }
        }
    }

    let layoutInfo = (*Irp).AssociatedIrp.SystemBuffer as PDRIVE_LAYOUT_INFORMATION;

    if unsafe { VerifyIrpInBufferSize(Irp, (core::mem::size_of::<_DRIVE_LAYOUT_INFORMATION>() as SIZE_T)) } == 0u8 {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let layoutSize: usize = core::mem::offset_of!(_DRIVE_LAYOUT_INFORMATION, PartitionEntry)
        + ((*layoutInfo).PartitionCount as usize).wrapping_mul(core::mem::size_of::<PARTITION_INFORMATION>());

    if unsafe { VerifyIrpInBufferSize(Irp, (layoutSize as SIZE_T)) } == 0u8 {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let layoutEx: PDRIVE_LAYOUT_INFORMATION_EX = unsafe { PartMgrConvertLayoutToExtended(layoutInfo) };
    if layoutEx.is_null() {
        (*Irp).IoStatus.Information = (0 as _);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    unsafe { PartMgrAcquireLayoutLock(FdoExtension) };

    if ((*FdoExtension).IsSuperFloppy != 0u8) && ((*layoutEx).PartitionCount > 1) {
        unsafe { PartMgrReleaseLayoutLock(FdoExtension) };
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    unsafe { PartMgrUpdatePartitionDevices(FdoExtension, layoutEx) };

    let status: NTSTATUS = unsafe {
        IoWritePartitionTableEx(
            (*FdoExtension).LowerDevice,
            layoutEx as *mut _DRIVE_LAYOUT_INFORMATION_EX,
        )
    };

    if status >= 0 {
        // Update layout cache
        if !(*FdoExtension).LayoutCache.is_null() {
            unsafe { ExFreePool((*FdoExtension).LayoutCache as PVOID) };
        }
        (*FdoExtension).LayoutCache = layoutEx;
        (*FdoExtension).LayoutValid = 1u8;

        // Copy updated partition numbers back to caller
        for i in 0..(*layoutInfo).PartitionCount {
            let part = unsafe { core::ptr::addr_of_mut!((*layoutInfo).PartitionEntry).cast::<PARTITION_INFORMATION>().offset(i as isize) };
            unsafe {
                (*part).PartitionNumber = (*core::ptr::addr_of!((*layoutEx).PartitionEntry).cast::<PARTITION_INFORMATION_EX>().offset(i as isize)).PartitionNumber;
            }
        }

        (*FdoExtension).IsSuperFloppy = unsafe { PartMgrIsDiskSuperFloppy(FdoExtension) };
    } else {
        (*FdoExtension).LayoutValid = 0u8;
    }

    unsafe { PartMgrReleaseLayoutLock(FdoExtension) };

    unsafe { IoInvalidateDeviceRelations((*FdoExtension).PhysicalDiskDO, BusRelations) };

    let mut notification: TARGET_DEVICE_CUSTOM_NOTIFICATION = TARGET_DEVICE_CUSTOM_NOTIFICATION {
        Version: 0,
        Size: 0,
        Event: GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0u8; 8] },
        FileObject: core::ptr::null_mut(),
        NameBufferOffset: 0,
        CustomDataBuffer: [0u8; 1],
    };

    notification.Event = GUID_IO_DISK_LAYOUT_CHANGE;
    notification.Version = 1;
    notification.Size = 36; // FIELD_OFFSET(TARGET_DEVICE_CUSTOM_NOTIFICATION, CustomDataBuffer)
    notification.FileObject = core::ptr::null_mut();
    notification.NameBufferOffset = -1i32;

    unsafe {
        IoReportTargetDeviceChangeAsynchronous(
            (*FdoExtension).PhysicalDiskDO,
            &raw mut notification as PVOID,
            None,
            core::ptr::null_mut(),
        );
    }

    (*Irp).IoStatus.Information = (layoutSize as _);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskSetDriveLayoutEx(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    as *const u8 as PVOID,
                767 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    let layoutUser: PDRIVE_LAYOUT_INFORMATION_EX = (*Irp).AssociatedIrp.SystemBuffer as PDRIVE_LAYOUT_INFORMATION_EX;
    let layoutEx: PDRIVE_LAYOUT_INFORMATION_EX;
    let status: NTSTATUS;

    if VerifyIrpInBufferSize(Irp, (core::mem::size_of::<DRIVE_LAYOUT_INFORMATION_EX>() as SIZE_T)) == 0 {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let mut layoutSize: size_t = 48; // FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry[0])
    layoutSize = (layoutSize as u64).wrapping_add(
        ((*layoutUser).PartitionCount as usize)
            .wrapping_mul(core::mem::size_of::<PARTITION_INFORMATION_EX>())
            as u64,
    ) as size_t;

    if VerifyIrpInBufferSize(Irp, (layoutSize as SIZE_T)) == 0 {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    layoutEx = ExAllocatePoolWithTag(PagedPool, layoutSize as SIZE_T, TAG_PARTMGR as ULONG) as PDRIVE_LAYOUT_INFORMATION_EX;
    if layoutEx.is_null() {
        (*Irp).IoStatus.Information = (0 as _);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopyMemory(layoutEx as PVOID, layoutUser as *const c_void, layoutSize as SIZE_T);
    PartMgrAcquireLayoutLock(FdoExtension);

    if (*FdoExtension).IsSuperFloppy as i32 != 0 &&
        ((*layoutEx).PartitionStyle != (PARTITION_STYLE_MBR as u32 as u64) as u32 ||
            (*layoutEx).PartitionCount > (1 as u64) as u32)
    {
        PartMgrReleaseLayoutLock(FdoExtension);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if (*layoutEx).PartitionCount == (0 as u64) as u32 {
        let mut createDisk: CREATE_DISK = CREATE_DISK {
            PartitionStyle: PARTITION_STYLE_MBR,
            __bindgen_anon_1: ntkrnl_sys::ddk::_CREATE_DISK__bindgen_ty_1 {
                Mbr: CREATE_DISK_MBR { Signature: 0 },
            },
        };
        createDisk.PartitionStyle = (*layoutEx).PartitionStyle as PARTITION_STYLE;
        if createDisk.PartitionStyle as u32 == PARTITION_STYLE_MBR as i32 as u32 {
            createDisk.__bindgen_anon_1.Mbr.Signature = (*layoutEx).__bindgen_anon_1.Mbr.Signature;
        } else if createDisk.PartitionStyle as u32 == PARTITION_STYLE_GPT as i32 as u32 {
            createDisk.__bindgen_anon_1.Gpt.DiskId = (*layoutEx).__bindgen_anon_1.Gpt.DiskId;
        }
        status = IoCreateDisk((*FdoExtension).LowerDevice, &raw mut createDisk);
    } else {
        PartMgrUpdatePartitionDevices(FdoExtension, layoutEx);
        status = IoWritePartitionTableEx((*FdoExtension).LowerDevice, layoutEx as *mut _DRIVE_LAYOUT_INFORMATION_EX);
        if NT_SUCCESS(status) {
            let mut i: UINT32 = 0;
            while (i as u64) < ((*layoutEx).PartitionCount) as u64 {
                let part: PPARTITION_INFORMATION_EX = (&raw mut (*layoutEx).PartitionEntry as *mut PARTITION_INFORMATION_EX).offset(i as isize) as PPARTITION_INFORMATION_EX;
                (*part).PartitionNumber = (*((&raw mut (*layoutEx).PartitionEntry as *mut PARTITION_INFORMATION_EX).offset(i as isize))).PartitionNumber;
                i = i.wrapping_add(1);
            }
        }
    }

    if NT_SUCCESS(status) {
        if !(*FdoExtension).LayoutCache.is_null() {
            ExFreePool((*FdoExtension).LayoutCache as PVOID);
        }
        (*FdoExtension).LayoutCache = layoutEx;
        (*FdoExtension).LayoutValid = 1u8; // TRUE
        (*FdoExtension).IsSuperFloppy = PartMgrIsDiskSuperFloppy(FdoExtension);
    } else {
        (*FdoExtension).LayoutValid = 0u8; // FALSE
    }

    PartMgrReleaseLayoutLock(FdoExtension);
    IoInvalidateDeviceRelations((*FdoExtension).PhysicalDiskDO, BusRelations);

    let mut notification: TARGET_DEVICE_CUSTOM_NOTIFICATION = TARGET_DEVICE_CUSTOM_NOTIFICATION {
        Version: 0,
        Size: 0,
        Event: GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0u8; 8],
        },
        FileObject: core::ptr::null_mut::<FILE_OBJECT>(),
        NameBufferOffset: 0,
        CustomDataBuffer: [0u8; 1],
    };
    notification.Event = GUID_IO_DISK_LAYOUT_CHANGE;
    notification.Version = 1 as USHORT;
    notification.Size = 36 as USHORT;
    notification.FileObject = core::ptr::null_mut::<FILE_OBJECT>();
    notification.NameBufferOffset = -(1 as i32) as LONG;
    IoReportTargetDeviceChangeAsynchronous(
        (*FdoExtension).PhysicalDiskDO,
        &raw mut notification as PVOID,
        None, // Callback: PDEVICE_CHANGE_COMPLETE_CALLBACK
        core::ptr::null_mut(),
    );
    (*Irp).IoStatus.Information = (layoutSize as _);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskUpdateProperties(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            (KeGetCurrentIrql()) as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const u8 as *const core::ffi::c_char as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0".as_ptr() as *const u8 as *const core::ffi::c_char as PVOID,
                885u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    PartMgrAcquireLayoutLock(FdoExtension);
    (*FdoExtension).LayoutValid = 0u8;
    PartMgrReleaseLayoutLock(FdoExtension);
    IoInvalidateDeviceRelations((*FdoExtension).PhysicalDiskDO, BusRelations);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskCreateDisk(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr()
                as *const u8 as *const i8,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/partmgr/partmgr.c\0".as_ptr()
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                902u32,
                (core::ptr::null_mut::<core::ffi::c_char>() as *mut core::ffi::c_void).cast(),
            );
        }
    }

    let createDisk: PCREATE_DISK = (*Irp).AssociatedIrp.SystemBuffer as PCREATE_DISK;
    if VerifyIrpInBufferSize(Irp, (core::mem::size_of::<_CREATE_DISK>() as SIZE_T)) == 0u8 {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    PartMgrAcquireLayoutLock(FdoExtension);

    let status = IoCreateDisk((*FdoExtension).LowerDevice, createDisk as *mut _CREATE_DISK);
    (*FdoExtension).LayoutValid = 0u8; // FALSE

    PartMgrReleaseLayoutLock(FdoExtension);

    IoInvalidateDeviceRelations((*FdoExtension).PhysicalDiskDO, BusRelations);
    status
}
#[no_mangle]
pub unsafe extern "C" fn FdoIoctlDiskDeleteDriveLayout(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE assertion: IRQL must be <= APC_LEVEL
    const APC_LEVEL: u8 = 1;
    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() <= APC_LEVEL);

    PartMgrAcquireLayoutLock(FdoExtension);

    let mut createDisk: CREATE_DISK = core::mem::zeroed();
    createDisk.PartitionStyle = PARTITION_STYLE_RAW;

    let status = IoCreateDisk((*FdoExtension).LowerDevice, core::ptr::addr_of_mut!(createDisk));

    (*FdoExtension).LayoutValid = 0u8; // FALSE

    PartMgrReleaseLayoutLock(FdoExtension);

    IoInvalidateDeviceRelations((*FdoExtension).PhysicalDiskDO, BusRelations);

    status
}
#[no_mangle]
pub unsafe extern "C" fn FdoHandleStartDevice(
    FdoExtension: *mut _FDO_EXTENSION,
    _Irp: PIRP,
) -> NTSTATUS {
    // Declare IssueSyncIoControlRequest if needed (assume extern available)

    let mut deviceNumber: _STORAGE_DEVICE_NUMBER = _STORAGE_DEVICE_NUMBER {
        DeviceType: 0,
        DeviceNumber: 0,
        PartitionNumber: 0,
    };

    let mut status: NTSTATUS = IssueSyncIoControlRequest(
        0x2D1080u32, // IOCTL_STORAGE_GET_DEVICE_NUMBER (typical value)
        (*FdoExtension).LowerDevice,
        core::ptr::null_mut(),
        0u32,
        &raw mut deviceNumber as *mut core::ffi::c_void,
        core::mem::size_of::<_STORAGE_DEVICE_NUMBER>() as u32,
        0u8, // FALSE
    );
    if !NT_SUCCESS(status) {
        DbgPrintEx(
            DPFLTR_DISK_ID as u32,
            DPFLTR_ERROR_LEVEL as u32,
            b"[PARTMGR] FdoHandleStartDevice: GET_DEVICE_NUMBER failed 0x%x\n\0".as_ptr()
                as *const i8,
            status,
        );
        return status;
    }

    (*FdoExtension).DiskData.DeviceNumber = deviceNumber.DeviceNumber as u32;

    let mut interfaceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };

    status = IoRegisterDeviceInterface(
        (*FdoExtension).PhysicalDiskDO,
        &raw const GUID_DEVINTERFACE_DISK,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        &raw mut interfaceName,
    );
    if !NT_SUCCESS(status) {
        DbgPrintEx(
            DPFLTR_DISK_ID as u32,
            DPFLTR_ERROR_LEVEL as u32,
            b"(%s:%d) Failed to register GUID_DEVINTERFACE_DISK, status %x\n\0".as_ptr() as *const i8,
            b"drivers/storage/partmgr/partmgr.c\0".as_ptr() as *const i8,
            976i32,
            status,
        );
        return status;
    }

    DbgPrintEx(
        DPFLTR_DISK_ID as u32,
        DPFLTR_INFO_LEVEL as u32,
        b"(%s:%d) Disk interface %wZ\n\0".as_ptr() as *const i8,
        b"drivers/storage/partmgr/partmgr.c\0".as_ptr() as *const i8,
        980i32,
        &raw mut interfaceName,
    );

    (*FdoExtension).DiskInterfaceName = interfaceName;
    status = IoSetDeviceInterfaceState(&raw mut interfaceName, 1u8); // TRUE
    if !NT_SUCCESS(status) {
        RtlFreeUnicodeString(&raw mut interfaceName);
        RtlInitUnicodeString(
            &raw mut (*FdoExtension).DiskInterfaceName,
            core::ptr::null::<u16>(),
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn PartMgrRefreshDiskData(
    FdoExtension: PFDO_EXTENSION,
) -> NTSTATUS {
    let irql = KeGetCurrentIrql();
    if irql > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            irql as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    as *const u8 as PVOID,
                1007,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    let mut geometryEx: DISK_GEOMETRY_EX = core::mem::zeroed();
    let status = IssueSyncIoControlRequest(
        IOCTL_DISK_GET_DRIVE_GEOMETRY_EX as u32,
        (*FdoExtension).LowerDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(geometryEx).cast(),
        core::mem::size_of::<DISK_GEOMETRY_EX>() as u32,
        0u8,
    );
    if !NT_SUCCESS(status) {
        return status;
    }
    (*FdoExtension).DiskData.DiskSize = geometryEx.DiskSize.QuadPart as u64;
    (*FdoExtension).DiskData.BytesPerSector = geometryEx.Geometry.BytesPerSector;
    let mut layoutEx: PDRIVE_LAYOUT_INFORMATION_EX =
        core::ptr::null_mut::<_DRIVE_LAYOUT_INFORMATION_EX>();
    let status = PartMgrGetDriveLayout(FdoExtension, core::ptr::addr_of_mut!(layoutEx));
    if !NT_SUCCESS(status) {
        return status;
    }
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FdoHandleDeviceRelations(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    // Corrected from the LLM's fabricated values (review: DISK_ID=9, INFO=3, TRACE=2,
    // 'MtrP' not 'PMGP') — diagnostic-only (DbgPrintEx component/level + pool tag).
    const DPFLTR_DISK_ID: u32 = 9;
    const DPFLTR_INFO_LEVEL: u32 = 3;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const TAG_PARTMGR: u32 = 0x4d747250; // 'MtrP'

    let ioStack = IoGetCurrentIrpStackLocation(Irp);
    let type_0 = (*ioStack).Parameters.QueryDeviceRelations.Type;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as u32,
        );
        if (0 as i32) == 0 {
            RtlAssert(
                &0u8 as *const u8 as PVOID,
                &0u8 as *const u8 as PVOID,
                1043u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    if type_0 == BusRelations {
        PartMgrAcquireLayoutLock(FdoExtension);

        let status = PartMgrRefreshDiskData(FdoExtension);
        if !(status >= 0) {
            PartMgrReleaseLayoutLock(FdoExtension);
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            (*Irp).IoStatus.Information = 0 as _;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        DbgPrintEx(
            DPFLTR_DISK_ID,
            DPFLTR_INFO_LEVEL,
            b"(%s:%d) Partition style %u\n\0" as *const u8 as *const i8,
            b"drivers/storage/partmgr/partmgr.c\0"
                as *const u8 as *const i8,
            1059i32,
            (*FdoExtension).DiskData.PartitionStyle as u32,
        );

        let mut layoutEx: PDRIVE_LAYOUT_INFORMATION_EX =
            core::ptr::null_mut::<_DRIVE_LAYOUT_INFORMATION_EX>();
        PartMgrGetDriveLayout(FdoExtension, core::ptr::addr_of_mut!(layoutEx));
        PartMgrUpdatePartitionDevices(FdoExtension, layoutEx);

        DbgPrintEx(
            DPFLTR_DISK_ID,
            DPFLTR_TRACE_LEVEL,
            b"(%s:%d) Reporting %u partitions\n\0" as *const u8 as *const i8,
            b"drivers/storage/partmgr/partmgr.c\0"
                as *const u8 as *const i8,
            1069i32,
            (*FdoExtension).EnumeratedPartitionsTotal,
        );

        let mut deviceRelations: PDEVICE_RELATIONS = ExAllocatePoolWithTag(
            PagedPool,
            ((core::mem::size_of::<DEVICE_RELATIONS>() as u32)
                + (core::mem::size_of::<PDEVICE_OBJECT>() as u32
                    * ((*FdoExtension).EnumeratedPartitionsTotal as u32).wrapping_sub(1u32))) as usize,
            TAG_PARTMGR,
        ) as PDEVICE_RELATIONS;

        if deviceRelations.is_null() {
            PartMgrReleaseLayoutLock(FdoExtension);
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            (*Irp).IoStatus.Information = 0 as _;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        (*deviceRelations).Count = 0u32;

        let mut curEntry: PSINGLE_LIST_ENTRY = ((*FdoExtension).PartitionList.Next).cast();
        while !curEntry.is_null() {
            let partExt = (curEntry as usize)
                .wrapping_sub(
                    core::mem::offset_of!(PARTITION_EXTENSION, ListEntry),
                ) as PPARTITION_EXTENSION;

            (*partExt).IsEnumerated = 1u8;

            let count = &mut (*deviceRelations).Count;
            *core::ptr::addr_of_mut!((*deviceRelations).Objects).cast::<PDEVICE_OBJECT>().add(*count as usize) = (*partExt).DeviceObject;
            *count = *count + 1;

            ObReferenceObject((*partExt).DeviceObject as PVOID);
            curEntry = ((*partExt).ListEntry.Next).cast();
        }

        if (*deviceRelations).Count != (*FdoExtension).EnumeratedPartitionsTotal {
            RtlAssert(
                b"deviceRelations->Count == FdoExtension->EnumeratedPartitionsTotal\0"
                    as *const u8 as *const i8 as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    as *const u8 as *const i8 as PVOID,
                1104u32,
                core::ptr::null_mut::<i8>(),
            );
        }

        PartMgrReleaseLayoutLock(FdoExtension);

        (*Irp).IoStatus.Information = (deviceRelations as _);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }

    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver((*FdoExtension).LowerDevice, Irp)
}
#[no_mangle]
pub unsafe extern "C" fn FdoHandleRemoveDevice(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if true {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    as *const u8 as *const i8 as PVOID,
                1123u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if !(*FdoExtension).DiskInterfaceName.Buffer.is_null() {
        IoSetDeviceInterfaceState(
            &raw mut (*FdoExtension).DiskInterfaceName,
            0u8,
        );
        RtlFreeUnicodeString(&raw mut (*FdoExtension).DiskInterfaceName);
        RtlInitUnicodeString(
            &raw mut (*FdoExtension).DiskInterfaceName,
            core::ptr::null::<u16>(),
        );
    }
    IoSkipCurrentIrpStackLocation(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    let status = IoCallDriver((*FdoExtension).LowerDevice, Irp);
    IoDetachDevice((*FdoExtension).LowerDevice);
    IoDeleteDevice((*FdoExtension).DeviceObject);
    status
}
#[no_mangle]
pub unsafe extern "C" fn FdoHandleSurpriseRemoval(
    FdoExtension: PFDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0"
                .as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if (0 as i32) == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0"
                    .as_ptr() as *const i8 as PVOID,
                1149u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let mut curEntry: PSINGLE_LIST_ENTRY = ((*FdoExtension).PartitionList.Next).cast();
    while !curEntry.is_null() {
        let partExt: PPARTITION_EXTENSION = (curEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(PARTITION_EXTENSION, ListEntry) as ULONG_PTR,
        ) as PPARTITION_EXTENSION;
        if (*partExt).IsEnumerated != 0u8 {
            PartitionHandleRemove(partExt, 1u8);
        }
        curEntry = (*curEntry).Next;
    }

    IoSkipCurrentIrpStackLocation(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    return IoCallDriver((*FdoExtension).LowerDevice, Irp);
}
#[no_mangle]
pub unsafe extern "system" fn PartMgrAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let current_irql = KeGetCurrentIrql();
    if current_irql > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            (current_irql) as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/storage/partmgr/partmgr.c\0".as_ptr() as PVOID,
                1183 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    let mut deviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    let mut status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<FDO_EXTENSION>() as ULONG,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        FILE_DEVICE_MASS_STORAGE as ULONG,
        (FILE_AUTOGENERATED_DEVICE_NAME | FILE_DEVICE_SECURE_OPEN) as ULONG,
        0u8,
        &mut deviceObject,
    );

    if !(status >= 0) {
        DbgPrintEx(
            DPFLTR_DISK_ID as ULONG,
            DPFLTR_ERROR_LEVEL as ULONG,
            b"(%s:%d) Failed to create FDO 0x%x\n\0".as_ptr() as *const i8,
            b"drivers/storage/partmgr/partmgr.c\0".as_ptr() as *const i8,
            1201 as i32,
            status,
        );
        return status;
    }

    let deviceExtension = &mut *((*deviceObject).DeviceExtension as *mut FDO_EXTENSION);
    core::ptr::write_bytes(
        deviceExtension as *mut FDO_EXTENSION as *mut u8,
        0u8,
        core::mem::size_of::<FDO_EXTENSION>(),
    );
    (*deviceExtension).IsFDO = 1u8;
    (*deviceExtension).DeviceObject = deviceObject;
    (*deviceExtension).LowerDevice = IoAttachDeviceToDeviceStack(deviceObject, PhysicalDeviceObject);
    if (*deviceExtension).LowerDevice.is_null() {
        IoDeleteDevice(deviceObject);
        return STATUS_DEVICE_REMOVED;
    }
    (*deviceExtension).PhysicalDiskDO = PhysicalDeviceObject;
    KeInitializeEvent(
        &raw mut (*deviceExtension).SyncEvent,
        _EVENT_TYPE_SynchronizationEvent,
        1u8,
    );
    (*deviceObject).DeviceType = (*(*deviceExtension).LowerDevice).DeviceType;
    (*deviceObject).Flags |= DO_DIRECT_IO | DO_POWER_PAGABLE;
    (*deviceObject).Flags &= !DO_DEVICE_INITIALIZING;
    STATUS_SUCCESS
}

pub unsafe extern "system" fn PartMgrDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let fdoExtension: PFDO_EXTENSION = (*DeviceObject).DeviceExtension as PFDO_EXTENSION;
    let mut status: NTSTATUS;

    // INFO("IRP_MJ_DEVICE_CONTROL %p Irp %p IOCTL %x isFdo: %u\n",
    //       DeviceObject, Irp, ioStack->Parameters.DeviceIoControl.IoControlCode, fdoExtension->IsFDO);
    DbgPrintEx(
        DPFLTR_DISK_ID as u32,
        DPFLTR_INFO_LEVEL as u32,
        (b"(%s:%d) IRP_MJ_DEVICE_CONTROL %p Irp %p IOCTL %x isFdo: %u\n\0" as *const u8 as *const u8).cast(),
        b"drivers/storage/partmgr/partmgr.c\0"
            as *const u8 as *const i8,
        1245i32,
        DeviceObject,
        Irp,
        (*ioStack).Parameters.DeviceIoControl.IoControlCode,
        (*fdoExtension).IsFDO as i32,
    );

    if (*fdoExtension).IsFDO == 0 {
        return PartitionHandleDeviceControl(DeviceObject, Irp);
    }

    match (*ioStack).Parameters.DeviceIoControl.IoControlCode {
        458912 => {
            status = FdoIoctlDiskGetDriveGeometryEx(fdoExtension, Irp);
        }
        475140 => {
            status = FdoIoctlDiskGetPartitionInfo(fdoExtension, Irp);
        }
        458824 => {
            status = FdoIoctlDiskGetPartitionInfoEx(fdoExtension, Irp);
        }
        475148 => {
            status = FdoIoctlDiskGetDriveLayout(fdoExtension, Irp);
        }
        458832 => {
            status = FdoIoctlDiskGetDriveLayoutEx(fdoExtension, Irp);
        }
        507920 => {
            status = FdoIoctlDiskSetDriveLayout(fdoExtension, Irp);
        }
        507988 => {
            status = FdoIoctlDiskSetDriveLayoutEx(fdoExtension, Irp);
        }
        459072 => {
            status = FdoIoctlDiskUpdateProperties(fdoExtension, Irp);
        }
        507992 => {
            status = FdoIoctlDiskCreateDisk(fdoExtension, Irp);
        }
        508160 => {
            status = FdoIoctlDiskDeleteDriveLayout(fdoExtension, Irp);
        }
        _ => return ForwardIrpAndForget(DeviceObject, Irp),
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return status;
}

#[no_mangle]
pub unsafe extern "system" fn PartMgrPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            (KeGetCurrentIrql()) as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/partmgr/partmgr.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                1311u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let fdoExtension = (*DeviceObject).DeviceExtension as PFDO_EXTENSION;
    let ioStack = IoGetCurrentIrpStackLocation(Irp);

    DbgPrintEx(
        DPFLTR_DISK_ID,
        DPFLTR_INFO_LEVEL,
        b"(%s:%d) IRP_MJ_PNP %p Irp %p %s isFDO: %u\n\0" as *const u8 as *const i8,
        b"drivers/storage/partmgr/partmgr.c\0"
            as *const u8 as *const i8,
        1317i32,
        DeviceObject,
        Irp,
        GetIRPMinorFunctionString((*ioStack).MinorFunction),
        (*fdoExtension).IsFDO as u32,
    );

    if (*fdoExtension).IsFDO == 0 {
        return PartitionHandlePnp(DeviceObject, Irp);
    }

    match (*ioStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            let mut status;
            if IoForwardIrpSynchronously((*fdoExtension).LowerDevice, Irp) == 0 {
                status = STATUS_UNSUCCESSFUL;
            } else {
                status = FdoHandleStartDevice(fdoExtension, Irp);
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return status;
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            return FdoHandleDeviceRelations(fdoExtension, Irp);
        }
        IRP_MN_SURPRISE_REMOVAL => {
            return FdoHandleSurpriseRemoval(fdoExtension, Irp);
        }
        IRP_MN_REMOVE_DEVICE => {
            return FdoHandleRemoveDevice(fdoExtension, Irp);
        }
        IRP_MN_QUERY_STOP_DEVICE
        | IRP_MN_QUERY_REMOVE_DEVICE
        | IRP_MN_CANCEL_STOP_DEVICE
        | IRP_MN_CANCEL_REMOVE_DEVICE
        | IRP_MN_STOP_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }
        _ => {}
    }

    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver((*fdoExtension).LowerDevice, Irp);
}

pub unsafe extern "system" fn PartMgrReadWrite(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let partExt = (*DeviceObject).DeviceExtension as PPARTITION_EXTENSION;
    let ioStack = IoGetCurrentIrpStackLocation(Irp);
    if (*partExt).IsFDO == 0 {
        if (*partExt).IsEnumerated == 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return STATUS_DEVICE_DOES_NOT_EXIST;
        } else {
            (*ioStack).Parameters.Read.ByteOffset.QuadPart =
                ((*ioStack).Parameters.Read.ByteOffset.QuadPart).wrapping_add(((*partExt).StartingOffset) as i64);
        }
    }
    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver((*partExt).LowerDevice, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn PartMgrPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let partExt = (*DeviceObject).DeviceExtension as *mut _PARTITION_EXTENSION;
    let ioStack = IoGetCurrentIrpStackLocation(Irp);
    PoStartNextPowerIrp(Irp);
    if (*partExt).IsFDO == 0 {
        let status;
        if (*partExt).IsEnumerated == 0 {
            status = STATUS_DEVICE_DOES_NOT_EXIST;
        } else if (*ioStack).MinorFunction == IRP_MN_SET_POWER
            || (*ioStack).MinorFunction == IRP_MN_QUERY_POWER
        {
            status = STATUS_SUCCESS;
        } else {
            status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return status;
    } else {
        IoSkipCurrentIrpStackLocation(Irp);
        return PoCallDriver((*partExt).LowerDevice, Irp);
    }
}
#[no_mangle]
pub unsafe extern "system" fn PartMgrShutdownFlush(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let partExt = (*DeviceObject).DeviceExtension as PPARTITION_EXTENSION;
    let lowerDevice: PDEVICE_OBJECT;
    if (*partExt).IsFDO == 0 {
        if (*partExt).IsEnumerated == 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return STATUS_DEVICE_DOES_NOT_EXIST;
        } else {
            let fdoExtension = (*(*partExt).LowerDevice).DeviceExtension as PFDO_EXTENSION;
            lowerDevice = (*fdoExtension).LowerDevice;
        }
    } else {
        lowerDevice = (*partExt).LowerDevice;
    }
    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver(lowerDevice, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn PartMgrUnload(DriverObject: PDRIVER_OBJECT) {
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    // Dispatch routines are extern "system" (correct NTAPI per arch: cdecl on amd64/
    // arm64, stdcall on i386). The bindgen dispatch-field types are nominally extern
    // "C", so transmute fn-item -> field type (ABI-identical per arch; pcix idiom, see
    // [[feedback_cohab_fnptr_transmute_and_inline_macros]]). The prior `Some(.. as fn)
    // as PDRIVER_*` was an illegal `as`-cast between Option types (E0605).
    use core::mem::transmute as xm;
    (*DriverObject).DriverUnload = Some(xm::<*const (), _>(PartMgrUnload as *const ()));
    (*(*DriverObject).DriverExtension).AddDevice = Some(xm::<*const (), _>(PartMgrAddDevice as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(xm::<*const (), _>(ForwardIrpAndForget as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(xm::<*const (), _>(ForwardIrpAndForget as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = Some(xm::<*const (), _>(PartMgrReadWrite as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = Some(xm::<*const (), _>(PartMgrReadWrite as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(xm::<*const (), _>(PartMgrDeviceControl as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(xm::<*const (), _>(PartMgrPnp as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN as usize] = Some(xm::<*const (), _>(PartMgrShutdownFlush as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_FLUSH_BUFFERS as usize] = Some(xm::<*const (), _>(PartMgrShutdownFlush as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(xm::<*const (), _>(PartMgrPower as *const ()));
    STATUS_SUCCESS
}
