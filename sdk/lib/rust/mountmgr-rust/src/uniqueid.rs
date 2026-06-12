use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExUuidCreate, _KWAIT_REASON_Executive, KeReleaseSemaphore, KeWaitForSingleObject, RTL_QUERY_REGISTRY_TABLE, RtlQueryRegistryValues, RtlStringFromGUID, RtlWriteRegistryValue, UUID};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{KPROCESSOR_MODE, _LIST_ENTRY, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::mm::{PagedPool};

extern "C" {
    static mut DatabasePath: *mut u16;
}
extern "system" {
    fn ExAllocatePoolWithTag(PoolType: u32, NumberOfBytes: usize, Tag: u32) -> *mut core::ffi::c_void;
    fn ExFreePoolWithTag(P: *mut core::ffi::c_void, Tag: u32);
    fn AllocatePool(size: u32) -> *mut core::ffi::c_void;
    fn FreePool(ptr: *mut core::ffi::c_void);
    fn RtlCompareMemory(Source1: *const core::ffi::c_void, Source2: *const core::ffi::c_void, Length: usize) -> usize;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

type PRTL_QUERY_REGISTRY_ROUTINE = Option<
    unsafe extern "C" fn(
        *mut u16,
        u32,
        *mut core::ffi::c_void,
        u32,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> NTSTATUS,
>;

const UNICODE_NULL: u16 = 0;

const REG_BINARY: u32 = 3;

type PMOUNTDEV_UNIQUE_ID = *mut MOUNTDEV_UNIQUE_ID;

// CheckForNoDriveLetterEntry — re-integrated from the translation cache
// (.responses/mountmgr__uniqueid/); the stitcher dropped its body and left only a
// malformed extern-block decl. RTL_QUERY_REGISTRY_ROUTINE callback. (uniqueid.c)
#[no_mangle]
pub unsafe extern "system" fn CheckForNoDriveLetterEntry(
    ValueName: PWSTR,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    let EntryPresent: *mut u8 = EntryContext as *mut u8;
    let UniqueId: *mut MOUNTDEV_UNIQUE_ID = Context as *mut MOUNTDEV_UNIQUE_ID;

    if *ValueName != '#' as u16
        || ValueType != REG_BINARY
        || (*UniqueId).UniqueIdLength as u32 != ValueLength
    {
        return STATUS_SUCCESS;
    }

    if RtlCompareMemory(
        &raw const (*UniqueId).UniqueId as *const core::ffi::c_void,
        ValueData,
        ValueLength as usize,
    ) == ValueLength as usize
    {
        *EntryPresent = 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ChangeUniqueIdRoutine(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let OldUniqueId: PMOUNTDEV_UNIQUE_ID = Context as PMOUNTDEV_UNIQUE_ID;
    let NewUniqueId: PMOUNTDEV_UNIQUE_ID = EntryContext as PMOUNTDEV_UNIQUE_ID;
    if ValueType != REG_BINARY as ULONG || (*OldUniqueId).UniqueIdLength as ULONG != ValueLength {
        return STATUS_SUCCESS;
    }
    if RtlCompareMemory(
        (&raw mut (*OldUniqueId).UniqueId as *mut UCHAR) as *const core::ffi::c_void,
        ValueData as *const core::ffi::c_void,
        ValueLength as SIZE_T,
    ) == ValueLength as usize
    {
        RtlWriteRegistryValue(
            RTL_REGISTRY_ABSOLUTE as ULONG,
            DatabasePath as PCWSTR,
            ValueName as PCWSTR,
            REG_BINARY as ULONG,
            NewUniqueId as PVOID,
            (*NewUniqueId).UniqueIdLength as ULONG,
        );
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrUniqueIdChangeRoutine(
    DeviceExtension: *mut DEVICE_EXTENSION,
    OldUniqueId: *mut MOUNTDEV_UNIQUE_ID,
    NewUniqueId: *mut MOUNTDEV_UNIQUE_ID,
) {
    let mut Status: NTSTATUS = 0;
    let mut ResyncNeeded: BOOLEAN = 0;
    let mut DuplicateId: *mut UNIQUE_ID_REPLICATE = core::ptr::null_mut();
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] =
        core::mem::zeroed();
    let mut UniqueId: *mut MOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let mut NewDuplicateId: *mut MOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let mut ListHead: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut ReplicatedHead: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut NextReplicated: *mut LIST_ENTRY = core::ptr::null_mut();

    Status = WaitForRemoteDatabaseSemaphore(DeviceExtension);

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i32 as i32 as KPROCESSOR_MODE,
        FALSE as BOOLEAN,
        core::ptr::null_mut(),
    );

    RtlZeroMemory(
        &raw mut QueryTable as *mut core::ffi::c_void,
        core::mem::size_of_val(&QueryTable),
    );

    QueryTable[0].QueryRoutine = Some(core::mem::transmute::<
        unsafe extern "system" fn(*mut u16, u32, *mut core::ffi::c_void, u32, *mut core::ffi::c_void, *mut core::ffi::c_void) -> NTSTATUS,
        _, // per-arch RTL_QUERY_REGISTRY_ROUTINE slot ABI
    >(ChangeUniqueIdRoutine));

    QueryTable[0].EntryContext = NewUniqueId as *mut core::ffi::c_void;

    RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE as u32,
        DatabasePath as *const u16,
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
        OldUniqueId as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    ListHead = core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) as *mut LIST_ENTRY;
    NextEntry = (*ListHead).Flink as *mut LIST_ENTRY;

    while ListHead != NextEntry {
        let offset = core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry) as isize;
        DeviceInformation =
            (NextEntry as *mut u8).byte_offset(-offset) as *mut DEVICE_INFORMATION;

        if (*(*DeviceInformation).UniqueId).UniqueIdLength as i32
            == (*OldUniqueId).UniqueIdLength as i32
            && RtlCompareMemory(
                &raw mut (*OldUniqueId).UniqueId as *mut u8 as *const core::ffi::c_void,
                &raw mut (*(*DeviceInformation).UniqueId).UniqueId as *mut u8
                    as *const core::ffi::c_void,
                (*OldUniqueId).UniqueIdLength as usize,
            ) as u64
                == (*OldUniqueId).UniqueIdLength as u64
        {
            break;
        }

        NextEntry = (*NextEntry).Flink as *mut LIST_ENTRY;
    }

    if ListHead == NextEntry {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
            IO_NO_INCREMENT as i32,
            1i32,
            FALSE as BOOLEAN,
        );

        if Status >= 0 {
            ReleaseRemoteDatabaseSemaphore(DeviceExtension);
        }

        return;
    }

    if Status < 0 {
        ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
            IO_NO_INCREMENT as i32,
            1i32,
            FALSE as BOOLEAN,
        );
        return;
    }

    UniqueId = ExAllocatePoolWithTag(
        PagedPool,
        (*NewUniqueId).UniqueIdLength as usize
            + core::mem::size_of::<MOUNTDEV_UNIQUE_ID>(),
        1098149453u32,
    ) as *mut MOUNTDEV_UNIQUE_ID;

    if UniqueId.is_null() {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
            IO_NO_INCREMENT as i32,
            1i32,
            FALSE as BOOLEAN,
        );
        ReleaseRemoteDatabaseSemaphore(DeviceExtension);
        return;
    }

    ExFreePoolWithTag((*DeviceInformation).UniqueId as *mut core::ffi::c_void, 1098149453u32);
    (*DeviceInformation).UniqueId = UniqueId;
    (*UniqueId).UniqueIdLength = (*NewUniqueId).UniqueIdLength;
    RtlCopyMemory(
        &raw mut (*UniqueId).UniqueId as *mut u8 as *mut core::ffi::c_void,
        &raw mut (*NewUniqueId).UniqueId as *mut u8 as *const core::ffi::c_void,
        (*NewUniqueId).UniqueIdLength as usize,
    );

    ListHead = core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) as *mut LIST_ENTRY;
    NextEntry = (*ListHead).Flink as *mut LIST_ENTRY;

    while ListHead != NextEntry {
        let offset = core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry) as isize;
        DeviceInformation =
            (NextEntry as *mut u8).byte_offset(-offset) as *mut DEVICE_INFORMATION;
        ResyncNeeded = FALSE as BOOLEAN;

        ReplicatedHead = core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead)
            as *mut LIST_ENTRY;
        NextReplicated = (*ReplicatedHead).Flink as *mut LIST_ENTRY;

        while ReplicatedHead != NextReplicated {
            let dup_offset = core::mem::offset_of!(UNIQUE_ID_REPLICATE, ReplicatedUniqueIdsListEntry) as isize;
            DuplicateId =
                (NextReplicated as *mut u8).byte_offset(-dup_offset) as *mut UNIQUE_ID_REPLICATE;

            if (*(*DuplicateId).UniqueId).UniqueIdLength as i32
                == (*OldUniqueId).UniqueIdLength as i32
            {
                if RtlCompareMemory(
                    &raw mut (*(*DuplicateId).UniqueId).UniqueId as *mut u8
                        as *const core::ffi::c_void,
                    &raw mut (*OldUniqueId).UniqueId as *mut u8 as *const core::ffi::c_void,
                    (*OldUniqueId).UniqueIdLength as usize,
                ) as u64
                    == (*OldUniqueId).UniqueIdLength as u64
                {
                    NewDuplicateId = ExAllocatePoolWithTag(
                        PagedPool,
                        (*NewUniqueId).UniqueIdLength as usize
                            + core::mem::size_of::<MOUNTDEV_UNIQUE_ID>(),
                        1098149453u32,
                    ) as *mut MOUNTDEV_UNIQUE_ID;

                    if !NewDuplicateId.is_null() {
                        ResyncNeeded = TRUE as BOOLEAN;
                        ExFreePoolWithTag(
                            (*DuplicateId).UniqueId as *mut core::ffi::c_void,
                            1098149453u32,
                        );
                        (*DuplicateId).UniqueId = NewDuplicateId;
                        (*(*DuplicateId).UniqueId).UniqueIdLength =
                            (*NewUniqueId).UniqueIdLength;
                        RtlCopyMemory(
                            &raw mut (*NewDuplicateId).UniqueId as *mut u8
                                as *mut core::ffi::c_void,
                            &raw mut (*NewUniqueId).UniqueId as *mut u8
                                as *const core::ffi::c_void,
                            (*NewUniqueId).UniqueIdLength as usize,
                        );
                    }
                }
            }

            NextReplicated = (*NextReplicated).Flink as *mut LIST_ENTRY;
        }

        if ResyncNeeded != 0 {
            ChangeRemoteDatabaseUniqueId(DeviceInformation, OldUniqueId, NewUniqueId);
        }

        NextEntry = (*NextEntry).Flink as *mut LIST_ENTRY;
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
        IO_NO_INCREMENT as i32,
        1i32,
        FALSE as BOOLEAN,
    );
    ReleaseRemoteDatabaseSemaphore(DeviceExtension);
}
#[no_mangle]
pub unsafe extern "C" fn IsUniqueIdPresent(
    DeviceExtension: *mut DEVICE_EXTENSION,
    DatabaseEntry: *mut DATABASE_ENTRY,
) -> u8 {
    let mut NextEntry: *mut _LIST_ENTRY = core::ptr::null_mut();
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();

    if IsListEmpty(&raw mut (*DeviceExtension).DeviceListHead) {
        return 0; // FALSE
    }

    NextEntry = (*DeviceExtension).DeviceListHead.Flink;
    while NextEntry != &raw mut (*DeviceExtension).DeviceListHead {
        DeviceInformation = (NextEntry as *mut u8)
            .sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
            as *mut DEVICE_INFORMATION;

        if (*(*DeviceInformation).UniqueId).UniqueIdLength != (*DatabaseEntry).UniqueIdLength {
            NextEntry = (*NextEntry).Flink;
            continue;
        }

        if RtlCompareMemory(
            ((DatabaseEntry as usize).wrapping_add((*DatabaseEntry).UniqueIdOffset as usize))
                as *const core::ffi::c_void,
            &raw mut (*(*DeviceInformation).UniqueId).UniqueId as *const core::ffi::c_void,
            (*DatabaseEntry).UniqueIdLength as usize,
        ) == (*DatabaseEntry).UniqueIdLength as usize
        {
            return 1; // TRUE
        }

        NextEntry = (*NextEntry).Flink;
    }

    0 // FALSE
}
#[no_mangle]
pub unsafe extern "C" fn CreateNoDriveLetterEntry(UniqueId: *mut MOUNTDEV_UNIQUE_ID) {
    let mut Guid: UUID = UUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut String: *mut u16 = core::ptr::null_mut();
    let mut GuidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if !(ExUuidCreate(&raw mut Guid) >= 0) {
        return;
    }

    if !(RtlStringFromGUID(&raw mut Guid, &raw mut GuidString) >= 0) {
        return;
    }

    let size = (GuidString.Length as usize)
        .wrapping_add((2 as usize).wrapping_mul(core::mem::size_of::<u16>()));
    String = ExAllocatePoolWithTag(PagedPool, size, 1098149453u32) as *mut u16;
    if String.is_null() {
        ExFreePoolWithTag(GuidString.Buffer as *mut core::ffi::c_void, 0);
        return;
    }

    *String = '#' as u16;
    core::ptr::copy_nonoverlapping(
        GuidString.Buffer as *const u8,
        (String as *mut u8).add(core::mem::size_of::<u16>()),
        GuidString.Length as usize,
    );
    *String.add(GuidString.Length as usize / core::mem::size_of::<u16>()) = UNICODE_NULL;

    ExFreePoolWithTag(GuidString.Buffer as *mut core::ffi::c_void, 0);

    RtlWriteRegistryValue(
        RTL_REGISTRY_ABSOLUTE as u32,
        DatabasePath as *const u16,
        String as *const u16,
        REG_BINARY as u32,
        &raw mut (*UniqueId).UniqueId as *mut u8 as *mut core::ffi::c_void,
        (*UniqueId).UniqueIdLength as u32,
    );

    ExFreePoolWithTag(String as *mut core::ffi::c_void, 1098149453u32);
}

#[no_mangle]
pub unsafe extern "C" fn HasNoDriveLetterEntry(
    UniqueId: PMOUNTDEV_UNIQUE_ID,
) -> i32 {
    let mut EntryPresent: i32 = 0;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: None,
        Flags: 0,
        Name: core::ptr::null::<u16>(),
        EntryContext: core::ptr::null_mut::<core::ffi::c_void>(),
        DefaultType: 0,
        DefaultData: core::ptr::null_mut::<core::ffi::c_void>(),
        DefaultLength: 0,
    }; 2];

    core::ptr::write_bytes(
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE as *mut core::ffi::c_void,
        0,
        core::mem::size_of_val(&QueryTable),
    );

    QueryTable[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS, _>(CheckForNoDriveLetterEntry));
    QueryTable[0].EntryContext = &raw mut EntryPresent as *mut core::ffi::c_void;

    RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        DatabasePath as *const u16,
        QueryTable.as_mut_ptr(),
        UniqueId as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    return EntryPresent;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateReplicatedUniqueIds(
    mut DeviceInformation: PDEVICE_INFORMATION,
    mut DatabaseEntry: PDATABASE_ENTRY,
) {
    let mut NextEntry: PLIST_ENTRY;
    let mut ReplicatedUniqueId: PUNIQUE_ID_REPLICATE;
    let mut NewEntry: PUNIQUE_ID_REPLICATE;

    NextEntry = (*DeviceInformation).ReplicatedUniqueIdsListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead) {
        ReplicatedUniqueId = ((NextEntry as usize).wrapping_sub(
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<UNIQUE_ID_REPLICATE>()).ReplicatedUniqueIdsListEntry) as usize
        )) as PUNIQUE_ID_REPLICATE;

        if (*(*ReplicatedUniqueId).UniqueId).UniqueIdLength == (*DatabaseEntry).UniqueIdLength {
            if RtlCompareMemory(
                core::ptr::addr_of!((*(*ReplicatedUniqueId).UniqueId).UniqueId) as *const core::ffi::c_void,
                (DatabaseEntry as usize + (*DatabaseEntry).UniqueIdOffset as usize) as *const core::ffi::c_void,
                (*(*ReplicatedUniqueId).UniqueId).UniqueIdLength as usize,
            ) == (*(*ReplicatedUniqueId).UniqueId).UniqueIdLength as usize {
                break;
            }
        }
        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry != core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead) {
        return;
    }

    NewEntry = AllocatePool(core::mem::size_of::<UNIQUE_ID_REPLICATE>() as u32) as PUNIQUE_ID_REPLICATE;
    if NewEntry.is_null() {
        return;
    }

    (*NewEntry).UniqueId = AllocatePool(
        ((*DatabaseEntry).UniqueIdLength as usize + core::mem::size_of::<MOUNTDEV_UNIQUE_ID>()) as u32
    ) as PMOUNTDEV_UNIQUE_ID;
    if (*NewEntry).UniqueId.is_null() {
        FreePool(NewEntry as *mut core::ffi::c_void);
        return;
    }

    (*(*NewEntry).UniqueId).UniqueIdLength = (*DatabaseEntry).UniqueIdLength;
    core::ptr::copy_nonoverlapping(
        (DatabaseEntry as usize + (*DatabaseEntry).UniqueIdOffset as usize) as *const core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*NewEntry).UniqueId).UniqueId) as *mut core::ffi::c_void,
        (*DatabaseEntry).UniqueIdLength as usize,
    );
    InsertTailList(
        core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead),
        core::ptr::addr_of_mut!((*NewEntry).ReplicatedUniqueIdsListEntry),
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2011 ReactOS Team
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
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA.
//  *
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/filesystem/mountmgr/uniqueid.c
//  * PURPOSE:          Mount Manager - Unique ID
//  * PROGRAMMER:       Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// #include "mntmgr.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// ChangeUniqueIdRoutine(IN PWSTR ValueName,
//                       IN ULONG ValueType,
//                       IN PVOID ValueData,
//                       IN ULONG ValueLength,
//                       IN PVOID Context,
//                       IN PVOID EntryContext)
// {
//     PMOUNTDEV_UNIQUE_ID OldUniqueId = Context;
//     PMOUNTDEV_UNIQUE_ID NewUniqueId = EntryContext;
//
//     /* Validate parameters not to corrupt registry */
//     if ((ValueType != REG_BINARY) ||
//         (OldUniqueId->UniqueIdLength != ValueLength))
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (RtlCompareMemory(OldUniqueId->UniqueId, ValueData, ValueLength) == ValueLength)
//     {
//         /* Write new data */
//         RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                               DatabasePath,
//                               ValueName,
//                               REG_BINARY,
//                               NewUniqueId,
//                               NewUniqueId->UniqueIdLength);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrUniqueIdChangeRoutine(IN PDEVICE_EXTENSION DeviceExtension,
//                               IN PMOUNTDEV_UNIQUE_ID OldUniqueId,
//                               IN PMOUNTDEV_UNIQUE_ID NewUniqueId)
// {
//     NTSTATUS Status;
//     BOOLEAN ResyncNeeded;
//     PUNIQUE_ID_REPLICATE DuplicateId;
//     PDEVICE_INFORMATION DeviceInformation;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//     PMOUNTDEV_UNIQUE_ID UniqueId, NewDuplicateId;
//     PLIST_ENTRY ListHead, NextEntry, ReplicatedHead, NextReplicated;
//
//     /* Synchronise with remote databases */
//     Status = WaitForRemoteDatabaseSemaphore(DeviceExtension);
//     KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = ChangeUniqueIdRoutine;
//     QueryTable[0].EntryContext = NewUniqueId;
//
//     /* Write new data */
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            DatabasePath,
//                            QueryTable,
//                            OldUniqueId,
//                            NULL);
//
//     /* Browse all the devices to find the one that
//      * owns the old unique ID
//      */
//     ListHead = &(DeviceExtension->DeviceListHead);
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry,
//                                               DEVICE_INFORMATION,
//                                               DeviceListEntry);
//
//         if (DeviceInformation->UniqueId->UniqueIdLength == OldUniqueId->UniqueIdLength &&
//             RtlCompareMemory(OldUniqueId->UniqueId,
//                              DeviceInformation->UniqueId->UniqueId,
//                              OldUniqueId->UniqueIdLength) == OldUniqueId->UniqueIdLength)
//         {
//             break;
//         }
//
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* If we didn't find any release everything and quit */
//     if (ListHead == NextEntry)
//     {
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT,
//                            1, FALSE);
//
//         if (NT_SUCCESS(Status))
//         {
//             ReleaseRemoteDatabaseSemaphore(DeviceExtension);
//         }
//
//         return;
//     }
//
//     /* If lock failed, then, just update this database */
//     if (!NT_SUCCESS(Status))
//     {
//         ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT,
//                            1, FALSE);
//         return;
//     }
//
//     /* Allocate new unique ID */
//     UniqueId = AllocatePool(NewUniqueId->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//     if (!UniqueId)
//     {
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT,
//                            1, FALSE);
//         ReleaseRemoteDatabaseSemaphore(DeviceExtension);
//         return;
//     }
//
//     /* Release old one */
//     FreePool(DeviceInformation->UniqueId);
//     /* And set new one */
//     DeviceInformation->UniqueId = UniqueId;
//     UniqueId->UniqueIdLength = NewUniqueId->UniqueIdLength;
//     RtlCopyMemory(UniqueId->UniqueId, NewUniqueId->UniqueId, NewUniqueId->UniqueIdLength);
//
//     /* Now, check if it's required to update replicated unique IDs as well */
//     ListHead = &(DeviceExtension->DeviceListHead);
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry,
//                                               DEVICE_INFORMATION,
//                                               DeviceListEntry);
//         ResyncNeeded = FALSE;
//
//         ReplicatedHead = &(DeviceInformation->ReplicatedUniqueIdsListHead);
//         NextReplicated = ReplicatedHead->Flink;
//         while (ReplicatedHead != NextReplicated)
//         {
//             DuplicateId = CONTAINING_RECORD(NextReplicated,
//                                             UNIQUE_ID_REPLICATE,
//                                             ReplicatedUniqueIdsListEntry);
//
//             if (DuplicateId->UniqueId->UniqueIdLength == OldUniqueId->UniqueIdLength)
//             {
//                 if (RtlCompareMemory(DuplicateId->UniqueId->UniqueId,
//                                      OldUniqueId->UniqueId,
//                                      OldUniqueId->UniqueIdLength) == OldUniqueId->UniqueIdLength)
//                 {
//                     /* It was our old unique ID */
//                     NewDuplicateId = AllocatePool(NewUniqueId->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//                     if (NewDuplicateId)
//                     {
//                         /* Update it */
//                         ResyncNeeded = TRUE;
//                         FreePool(DuplicateId->UniqueId);
//
//                         DuplicateId->UniqueId = NewDuplicateId;
//                         DuplicateId->UniqueId->UniqueIdLength = NewUniqueId->UniqueIdLength;
//                         RtlCopyMemory(NewDuplicateId->UniqueId, NewUniqueId->UniqueId, NewUniqueId->UniqueIdLength);
//                     }
//                 }
//             }
//
//             NextReplicated = NextReplicated->Flink;
//         }
//
//         /* If resync is required on this device, do it */
//         if (ResyncNeeded)
//         {
//             ChangeRemoteDatabaseUniqueId(DeviceInformation, OldUniqueId, NewUniqueId);
//         }
//
//         NextEntry = NextEntry->Flink;
//     }
//
//     KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//     ReleaseRemoteDatabaseSemaphore(DeviceExtension);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// IsUniqueIdPresent(IN PDEVICE_EXTENSION DeviceExtension,
//                   IN PDATABASE_ENTRY DatabaseEntry)
// {
//     PLIST_ENTRY NextEntry;
//     PDEVICE_INFORMATION DeviceInformation;
//
//     /* If no device, no unique ID (O'rly?!)
//      * ./)/).
//      * (°-°)
//      * (___) ORLY?
//      *  " "
//      */
//     if (IsListEmpty(&(DeviceExtension->DeviceListHead)))
//     {
//         return FALSE;
//     }
//
//     /* Now we know that we have devices, find the one */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry,
//                                               DEVICE_INFORMATION,
//                                               DeviceListEntry);
//
//         if (DeviceInformation->UniqueId->UniqueIdLength != DatabaseEntry->UniqueIdLength)
//         {
//             continue;
//         }
//
//         /* It's matching! */
//         if (RtlCompareMemory((PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset),
//                              DeviceInformation->UniqueId->UniqueId,
//                              DatabaseEntry->UniqueIdLength) == DatabaseEntry->UniqueIdLength)
//         {
//             return TRUE;
//         }
//     }
//
//     /* No luck... */
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// CreateNoDriveLetterEntry(IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     UUID Guid;
//     PWCHAR String;
//     UNICODE_STRING GuidString;
//
//     /* Entry with no drive letter are made that way:
//      * Instead of having a path with the letter,
//      * you have GUID with the unique ID.
//      */
//     if (!NT_SUCCESS(ExUuidCreate(&Guid)))
//     {
//         return;
//     }
//
//     /* Convert to string */
//     if (!NT_SUCCESS(RtlStringFromGUID(&Guid, &GuidString)))
//     {
//         return;
//     }
//
//     /* No letter entries must start with #, so allocate a proper string */
//     String = AllocatePool(GuidString.Length + 2 * sizeof(WCHAR));
//     if (!String)
//     {
//         ExFreePoolWithTag(GuidString.Buffer, 0);
//         return;
//     }
//
//     /* Write the complete string */
//     String[0] = L'#';
//     RtlCopyMemory(String + 1, GuidString.Buffer, GuidString.Length);
//     String[GuidString.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Don't need that one anymore */
//     ExFreePoolWithTag(GuidString.Buffer, 0);
//
//     /* Write the entry */
//     RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                           DatabasePath,
//                           String,
//                           REG_BINARY,
//                           UniqueId->UniqueId,
//                           UniqueId->UniqueIdLength);
//
//     FreePool(String);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI

// driver-internal cdecl decls (types_*.rs authority = extern "C").
unsafe extern "C" {
    fn WaitForRemoteDatabaseSemaphore(DeviceExtension: *mut DEVICE_EXTENSION) -> NTSTATUS;
    fn ReleaseRemoteDatabaseSemaphore(DeviceExtension: *mut DEVICE_EXTENSION);
    fn ReconcileThisDatabaseWithMaster( DeviceExtension: *mut DEVICE_EXTENSION, DeviceInformation: *mut DEVICE_INFORMATION, );
    fn ChangeRemoteDatabaseUniqueId( DeviceInformation: *mut DEVICE_INFORMATION, OldUniqueId: *mut MOUNTDEV_UNIQUE_ID, NewUniqueId: *mut MOUNTDEV_UNIQUE_ID, );
}
// CheckForNoDriveLetterEntry(IN PWSTR ValueName,
//                            IN ULONG ValueType,
//                            IN PVOID ValueData,
//                            IN ULONG ValueLength,
//                            IN PVOID Context,
//                            IN PVOID EntryContext)
// {
//     PBOOLEAN EntryPresent = EntryContext;
//     PMOUNTDEV_UNIQUE_ID UniqueId = Context;
//
//     /* Check if matches no drive letter entry */
//     if (ValueName[0] != L'#' || ValueType != REG_BINARY ||
//         UniqueId->UniqueIdLength != ValueLength)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Compare unique ID */
//     if (RtlCompareMemory(UniqueId->UniqueId, ValueData, ValueLength) == ValueLength)
//     {
//         *EntryPresent = TRUE;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// HasNoDriveLetterEntry(IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     BOOLEAN EntryPresent = FALSE;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].QueryRoutine = CheckForNoDriveLetterEntry;
//     QueryTable[0].EntryContext = &EntryPresent;
//
//     RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                            DatabasePath,
//                            QueryTable,
//                            UniqueId,
//                            NULL);
//
//     return EntryPresent;
// }
//
// /*
//  * @implemented
//  */
// VOID
// UpdateReplicatedUniqueIds(IN PDEVICE_INFORMATION DeviceInformation, IN PDATABASE_ENTRY DatabaseEntry)
// {
//     PLIST_ENTRY NextEntry;
//     PUNIQUE_ID_REPLICATE ReplicatedUniqueId, NewEntry;
//
//     /* Browse all the device replicated unique IDs */
//     for (NextEntry = DeviceInformation->ReplicatedUniqueIdsListHead.Flink;
//          NextEntry != &(DeviceInformation->ReplicatedUniqueIdsListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         ReplicatedUniqueId = CONTAINING_RECORD(NextEntry,
//                                                UNIQUE_ID_REPLICATE,
//                                                ReplicatedUniqueIdsListEntry);
//
//         if (ReplicatedUniqueId->UniqueId->UniqueIdLength != DatabaseEntry->UniqueIdLength)
//         {
//             continue;
//         }
//
//         /* If we find the UniqueId to update, break */
//         if (RtlCompareMemory(ReplicatedUniqueId->UniqueId->UniqueId,
//                              (PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset),
//                              ReplicatedUniqueId->UniqueId->UniqueIdLength) == ReplicatedUniqueId->UniqueId->UniqueIdLength)
//         {
//             break;
//         }
//     }
//
//     /* We found the unique ID, no need to continue */
//     if (NextEntry != &(DeviceInformation->ReplicatedUniqueIdsListHead))
//     {
//         return;
//     }
//
//     /* Allocate a new entry for unique ID */
//     NewEntry = AllocatePool(sizeof(UNIQUE_ID_REPLICATE));
//     if (!NewEntry)
//     {
//         return;
//     }
//
//     /* Allocate the unique ID */
//     NewEntry->UniqueId = AllocatePool(DatabaseEntry->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//     if (!NewEntry->UniqueId)
//     {
//         FreePool(NewEntry);
//         return;
//     }
//
//     /* Copy */
//     NewEntry->UniqueId->UniqueIdLength = DatabaseEntry->UniqueIdLength;
//     RtlCopyMemory(NewEntry->UniqueId->UniqueId,
//                   (PVOID)((ULONG_PTR)DatabaseEntry + DatabaseEntry->UniqueIdOffset),
//                   DatabaseEntry->UniqueIdLength);
//     /* And insert into replicated unique IDs list */
//     InsertTailList(&DeviceInformation->ReplicatedUniqueIdsListHead, &NewEntry->ReplicatedUniqueIdsListEntry);
//
//     return;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
