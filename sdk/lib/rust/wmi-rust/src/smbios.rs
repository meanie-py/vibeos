use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    MmMapIoSpace, MmUnmapIoSpace, PHYSICAL_ADDRESS, PULONG64, ULONG64,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[repr(C)]
struct SMBIOS21_ENTRY_POINT {
    AnchorString: [i8; 4],
    Checksum: UCHAR,
    Length: UCHAR,
    MajorVersion: UCHAR,
    MinorVersion: UCHAR,
    MaxStructureSize: USHORT,
    EntryPointRevision: UCHAR,
    FormattedArea: [i8; 5],
    AnchorString2: [i8; 5],
    Checksum2: UCHAR,
    TableLength: USHORT,
    TableAddress: ULONG,
    NumberOfStructures: USHORT,
    BCDRevision: UCHAR,
}

#[repr(C)]
struct SMBIOS30_ENTRY_POINT {
    AnchorString: [i8; 5],
    Checksum: UCHAR,
    Length: UCHAR,
    MajorVersion: UCHAR,
    MinorVersion: UCHAR,
    Docref: UCHAR,
    Revision: UCHAR,
    Reserved: UCHAR,
    TableMaxSize: ULONG,
    TableAddress: ULONG64,
}

#[repr(C)]
struct MSSmBios_RawSMBiosTables {
    Used20CallingMethod: UCHAR,
    SmbiosMajorVersion: UCHAR,
    SmbiosMinorVersion: UCHAR,
    DmiRevision: UCHAR,
    Size: ULONG,
    SMBiosData: [UCHAR; 1],
}

#[repr(C)]
struct WNODE_ALL_DATA {
    WnodeHeader: WNODE_HEADER,
    DataBlockOffset: ULONG,
    InstanceCount: ULONG,
    OffsetInstanceNameOffsets: ULONG,
    FixedInstanceSize: ULONG,
}

#[repr(C)]
union WNODE_HEADER_U1 {
    HistoricalContext: u64,
    VersionLinkage: WNODE_HEADER_VERSION_LINKAGE,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct WNODE_HEADER_VERSION_LINKAGE {
    Version: ULONG,
    Linkage: ULONG,
}

#[repr(C)]
union WNODE_HEADER_U2 {
    CountLost: ULONG,
    KernelHandle: PVOID,
    TimeStamp: LARGE_INTEGER,
}

#[repr(C)]
struct WNODE_HEADER {
    BufferSize: ULONG,
    ProviderId: ULONG,
    u1: WNODE_HEADER_U1,
    u2: WNODE_HEADER_U2,
    Guid: GUID,
    ClientContext: ULONG,
    Flags: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct GUID {
    Data1: ULONG,
    Data2: USHORT,
    Data3: USHORT,
    Data4: [UCHAR; 8],
}

const WNODE_FLAG_FIXED_INSTANCE_SIZE: ULONG = 0x10;

#[no_mangle]
pub static MSSmBios_RawSMBiosTables_GUID: GUID = GUID {
    Data1: 0x8f680850,
    Data2: 0xa584,
    Data3: 0x11d1,
    Data4: [0xbf, 0x38, 0x00, 0xa0, 0xc9, 0x06, 0x29, 0x10],
};

#[no_mangle]
pub unsafe extern "system" fn GetEntryPointData(
    EntryPointAddress: *const UCHAR,
    TableAddress: PULONG64,
    TableSize: *mut ULONG,
    BiosTablesHeader: *mut MSSmBios_RawSMBiosTables,
) -> BOOLEAN {
    let TableSize = opaque_ptr(TableSize);
    let BiosTablesHeader = opaque_ptr(BiosTablesHeader);
    let EntryPoint21 = EntryPointAddress as *const SMBIOS21_ENTRY_POINT;
    let EntryPoint30 = EntryPointAddress as *const SMBIOS30_ENTRY_POINT;
    let mut Checksum: UCHAR;
    let mut i: ULONG;

    if core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint21).AnchorString[0])) == b'_' as i8 &&
       core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint21).AnchorString[1])) == b'S' as i8 &&
       core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint21).AnchorString[2])) == b'M' as i8 &&
       core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint21).AnchorString[3])) == b'_' as i8 {
        
        if (*EntryPoint21).Length > 32 {
            return 0u8;
        }

        Checksum = 0;
        for i in 0..(*EntryPoint21).Length as usize {
            Checksum = Checksum.wrapping_add(*EntryPointAddress.add(i));
        }

        if Checksum != 0 {
            return 0u8;
        }

        *TableAddress = (*EntryPoint21).TableAddress as ULONG64;
        *TableSize = (*EntryPoint21).TableLength as ULONG;
        (*BiosTablesHeader).Used20CallingMethod = 0;
        (*BiosTablesHeader).SmbiosMajorVersion = (*EntryPoint21).MajorVersion;
        (*BiosTablesHeader).SmbiosMinorVersion = (*EntryPoint21).MinorVersion;
        (*BiosTablesHeader).DmiRevision = 2;
        (*BiosTablesHeader).Size = (*EntryPoint21).TableLength as ULONG;
        return 1u8;
    }

    if core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint30).AnchorString[0])) == b'_' as i8 &&
       core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint30).AnchorString[1])) == b'S' as i8 &&
       core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint30).AnchorString[2])) == b'M' as i8 &&
       core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint30).AnchorString[3])) == b'3' as i8 &&
       core::ptr::read_unaligned(core::ptr::addr_of!((*EntryPoint30).AnchorString[4])) == b'_' as i8 {
        
        if (*EntryPoint30).Length > 32 {
            return 0u8;
        }

        Checksum = 0;
        for i in 0..(*EntryPoint30).Length as usize {
            Checksum = Checksum.wrapping_add(*EntryPointAddress.add(i));
        }

        if Checksum != 0 {
            return 0u8;
        }

        *TableAddress = (*EntryPoint30).TableAddress;
        *TableSize = (*EntryPoint30).TableMaxSize;
        (*BiosTablesHeader).Used20CallingMethod = 0;
        (*BiosTablesHeader).SmbiosMajorVersion = (*EntryPoint30).MajorVersion;
        (*BiosTablesHeader).SmbiosMinorVersion = (*EntryPoint30).MinorVersion;
        (*BiosTablesHeader).DmiRevision = 3;
        (*BiosTablesHeader).Size = (*EntryPoint30).TableMaxSize;
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn WmipGetRawSMBiosTableData(
    OutTableData: *mut PVOID,
    OutDataSize: *mut ULONG,
) -> NTSTATUS {
    let OutTableData = opaque_ptr(OutTableData);
    let OutDataSize = opaque_ptr(OutDataSize);
    const SearchSize: SIZE_T = 0x10000;
    const HeaderSize: ULONG = core::mem::size_of::<MSSmBios_RawSMBiosTables>() as ULONG;
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let EntryPointMapping: *mut UCHAR;
    let mut BiosTablesHeader: MSSmBios_RawSMBiosTables = core::mem::zeroed();
    let BiosTables: PVOID;
    let TableMapping: PVOID;
    let mut Offset: ULONG;
    let mut TableSize: ULONG = 0;
    let mut TableAddress: ULONG64 = 0;

    PhysicalAddress.QuadPart = 0xF0000;

    EntryPointMapping = MmMapIoSpace(PhysicalAddress, SearchSize as _, ntkrnl_sys::ddk::_MEMORY_CACHING_TYPE_MmCached as _) as *mut UCHAR;
    if EntryPointMapping.is_null() {
        return 0xC0000001u32 as NTSTATUS;
    }

    Offset = 0;
    while Offset <= (0x10000 - 32) {
        if GetEntryPointData(
            EntryPointMapping.add(Offset as usize),
            &mut TableAddress,
            &mut TableSize,
            &mut BiosTablesHeader,
        ) != 0 {
            break;
        }
        Offset += 16;
    }

    MmUnmapIoSpace(EntryPointMapping as PVOID, SearchSize as _);

    if TableAddress == 0 {
        return 0xC0000225u32 as NTSTATUS;
    }

    if !OutTableData.is_null() {
        BiosTables = ExAllocatePoolWithTag(
            PagedPool,
            (HeaderSize + TableSize) as _,
            0x534D5442,
        );
        if BiosTables.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            &BiosTablesHeader as *const MSSmBios_RawSMBiosTables as *const u8,
            BiosTables as *mut u8,
            HeaderSize as usize,
        );

        PhysicalAddress.QuadPart = TableAddress as i64;

        TableMapping = MmMapIoSpace(PhysicalAddress, TableSize as _, ntkrnl_sys::ddk::_MEMORY_CACHING_TYPE_MmCached as _);
        if TableMapping.is_null() {
            ExFreePoolWithTag(BiosTables, 0x534D5442);
            return 0xC0000001u32 as NTSTATUS;
        }

        core::ptr::copy_nonoverlapping(
            TableMapping as *const u8,
            (BiosTables as *mut u8).add(HeaderSize as usize),
            TableSize as usize,
        );

        MmUnmapIoSpace(TableMapping, TableSize as _);

        *OutTableData = BiosTables;
    }

    *OutDataSize = HeaderSize + TableSize;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipQueryRawSMBiosTables(
    InOutBufferSize: *mut ULONG,
    OutBuffer: PVOID,
) -> NTSTATUS {
    let InOutBufferSize = opaque_ptr(InOutBufferSize);
    let OutBuffer = opaque_ptr(OutBuffer);
    let mut Status: NTSTATUS;
    let mut TableData: PVOID = core::ptr::null_mut();
    let mut TableSize: ULONG = 0;
    let ResultSize: ULONG;
    let AllData: *mut WNODE_ALL_DATA;

    Status = WmipGetRawSMBiosTableData(
        if !OutBuffer.is_null() { &mut TableData } else { core::ptr::null_mut() },
        &mut TableSize,
    );
    if Status < 0 {
        return Status;
    }

    ResultSize = core::mem::size_of::<WNODE_ALL_DATA>() as ULONG + TableSize;

    if !OutBuffer.is_null() && *InOutBufferSize != 0 {
        if *InOutBufferSize < ResultSize {
            return 0xC0000023u32 as NTSTATUS;
        }

        AllData = OutBuffer as *mut WNODE_ALL_DATA;
        (*AllData).WnodeHeader.BufferSize = ResultSize;
        (*AllData).WnodeHeader.ProviderId = 0;
        (*AllData).WnodeHeader.u1.VersionLinkage = WNODE_HEADER_VERSION_LINKAGE { Version: 0, Linkage: 0 };
        (*AllData).WnodeHeader.u2.KernelHandle = core::ptr::null_mut();
        (*AllData).WnodeHeader.Guid = MSSmBios_RawSMBiosTables_GUID;
        (*AllData).WnodeHeader.Flags = WNODE_FLAG_FIXED_INSTANCE_SIZE;
        (*AllData).DataBlockOffset = core::mem::size_of::<WNODE_ALL_DATA>() as ULONG;
        (*AllData).InstanceCount = 1;
        (*AllData).FixedInstanceSize = TableSize;

        core::ptr::copy_nonoverlapping(
            TableData as *const u8,
            (AllData as *mut u8).add(core::mem::size_of::<WNODE_ALL_DATA>()),
            TableSize as usize,
        );
    }

    *InOutBufferSize = ResultSize;

    if !TableData.is_null() {
        ExFreePoolWithTag(TableData, 0x534D5442);
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/wmi/smbios.c
//  * PURPOSE:         I/O Windows Management Instrumentation (WMI) Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <wmiguid.h>
// #include <wmidata.h>
// #include <wmistr.h>
//
// #include "wmip.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* FUNCTIONS *****************************************************************/
//
// typedef struct _SMBIOS21_ENTRY_POINT
// {
//  	CHAR AnchorString[4];
//  	UCHAR Checksum;
//  	UCHAR Length;
//  	UCHAR MajorVersion;
//  	UCHAR MinorVersion;
//  	USHORT MaxStructureSize;
//  	UCHAR EntryPointRevision;
//  	CHAR FormattedArea[5];
//  	CHAR AnchorString2[5];
//  	UCHAR Checksum2;
//  	USHORT TableLength;
//  	ULONG TableAddress;
//  	USHORT NumberOfStructures;
//  	UCHAR BCDRevision;
// } SMBIOS21_ENTRY_POINT, *PSMBIOS21_ENTRY_POINT;
//
// typedef struct _SMBIOS30_ENTRY_POINT
// {
//  	CHAR AnchorString[5];
//  	UCHAR Checksum;
//  	UCHAR Length;
//  	UCHAR MajorVersion;
//  	UCHAR MinorVersion;
//  	UCHAR Docref;
//     UCHAR Revision;
//     UCHAR Reserved;
//     ULONG TableMaxSize;
//     ULONG64 TableAddress;
// } SMBIOS30_ENTRY_POINT, *PSMBIOS30_ENTRY_POINT;
//
// static
// BOOLEAN
// GetEntryPointData(
//     _In_ const UCHAR *EntryPointAddress,
//     _Out_ PULONG64 TableAddress,
//     _Out_ PULONG TableSize,
//     _Out_ PMSSmBios_RawSMBiosTables BiosTablesHeader)
// {
//     PSMBIOS21_ENTRY_POINT EntryPoint21;
//     PSMBIOS30_ENTRY_POINT EntryPoint30;
//     UCHAR Checksum;
//     ULONG i;
//
//     /* Check for SMBIOS 2.1 entry point */
//     EntryPoint21 = (PSMBIOS21_ENTRY_POINT)EntryPointAddress;
//     if (RtlEqualMemory(EntryPoint21->AnchorString, "_SM_", 4))
//     {
//         if (EntryPoint21->Length > 32)
//             return FALSE;
//
//         /* Calculate the checksum */
//         Checksum = 0;
//         for (i = 0; i < EntryPoint21->Length; i++)
//         {
//             Checksum += EntryPointAddress[i];
//         }
//
//         if (Checksum != 0)
//             return FALSE;
//
//         *TableAddress = EntryPoint21->TableAddress;
//         *TableSize = EntryPoint21->TableLength;
//         BiosTablesHeader->Used20CallingMethod = 0;
//         BiosTablesHeader->SmbiosMajorVersion = EntryPoint21->MajorVersion;
//         BiosTablesHeader->SmbiosMinorVersion = EntryPoint21->MinorVersion;
//         BiosTablesHeader->DmiRevision = 2;
//         BiosTablesHeader->Size = EntryPoint21->TableLength;
//         return TRUE;
//     }
//
//     /* Check for SMBIOS 3.0 entry point */
//     EntryPoint30 = (PSMBIOS30_ENTRY_POINT)EntryPointAddress;
//     if (RtlEqualMemory(EntryPoint30->AnchorString, "_SM3_", 5))
//     {
//         if (EntryPoint30->Length > 32)
//             return FALSE;
//
//         /* Calculate the checksum */
//         Checksum = 0;
//         for (i = 0; i < EntryPoint30->Length; i++)
//         {
//             Checksum += EntryPointAddress[i];
//         }
//
//         if (Checksum != 0)
//             return FALSE;
//
//         *TableAddress = EntryPoint30->TableAddress;
//         *TableSize = EntryPoint30->TableMaxSize;
//         BiosTablesHeader->Used20CallingMethod = 0;
//         BiosTablesHeader->SmbiosMajorVersion = EntryPoint30->MajorVersion;
//         BiosTablesHeader->SmbiosMinorVersion = EntryPoint30->MinorVersion;
//         BiosTablesHeader->DmiRevision = 3;
//         BiosTablesHeader->Size = EntryPoint30->TableMaxSize;
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// _At_(*OutTableData, __drv_allocatesMem(Mem))
// NTSTATUS
// NTAPI
// WmipGetRawSMBiosTableData(
//     _Outptr_opt_result_buffer_(*OutDataSize) PVOID *OutTableData,
//     _Out_ PULONG OutDataSize)
// {
//     static const SIZE_T SearchSize = 0x10000;
//     static const ULONG HeaderSize = FIELD_OFFSET(MSSmBios_RawSMBiosTables, SMBiosData);
//     PHYSICAL_ADDRESS PhysicalAddress;
//     PUCHAR EntryPointMapping;
//     MSSmBios_RawSMBiosTables BiosTablesHeader;
//     PVOID BiosTables, TableMapping;
//     ULONG Offset, TableSize;
//     ULONG64 TableAddress = 0;
//
//     /* This is where the range for the entry point starts */
//     PhysicalAddress.QuadPart = 0xF0000;
//
//     /* Map the range into the system address space */
//     EntryPointMapping = MmMapIoSpace(PhysicalAddress, SearchSize, MmCached);
//     if (EntryPointMapping == NULL)
//     {
//         DPRINT1("Failed to map range for SMBIOS entry point\n");
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* Loop the table memory in 16 byte steps */
//     for (Offset = 0; Offset <= (0x10000 - 32); Offset += 16)
//     {
//         /* Check if we have an entry point here and get it's data */
//         if (GetEntryPointData(EntryPointMapping + Offset,
//                               &TableAddress,
//                               &TableSize,
//                               &BiosTablesHeader))
//         {
//             break;
//         }
//     }
//
//     /* Unmap the entry point */
//     MmUnmapIoSpace(EntryPointMapping, SearchSize);
//
//     /* Did we find anything */
//     if (TableAddress == 0)
//     {
//         DPRINT1("Could not find the SMBIOS entry point\n");
//         return STATUS_NOT_FOUND;
//     }
//
//     /* Check if the caller asked for the buffer */
//     if (OutTableData != NULL)
//     {
//         /* Allocate a buffer for the result */
//         BiosTables = ExAllocatePoolWithTag(PagedPool,
//                                            HeaderSize + TableSize,
//                                            'BTMS');
//         if (BiosTables == NULL)
//         {
//             DPRINT1("Failed to allocate %lu bytes for the SMBIOS table\n",
//                     HeaderSize + TableSize);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Copy the header */
//         RtlCopyMemory(BiosTables, &BiosTablesHeader, HeaderSize);
//
//         /* This is where the table is */
//         PhysicalAddress.QuadPart = TableAddress;
//
//         /* Map the table into the system address space */
//         TableMapping = MmMapIoSpace(PhysicalAddress, TableSize, MmCached);
//         if (TableMapping == NULL)
//         {
//             ExFreePoolWithTag(BiosTables, 'BTMS');
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* Copy the table */
//         RtlCopyMemory((PUCHAR)BiosTables + HeaderSize, TableMapping, TableSize);
//
//         /* Unmap the table */
//         MmUnmapIoSpace(TableMapping, TableSize);
//
//         *OutTableData = BiosTables;
//     }
//
//     *OutDataSize = HeaderSize + TableSize;
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// WmipQueryRawSMBiosTables(
//     _Inout_ ULONG *InOutBufferSize,
//     _Out_opt_ PVOID OutBuffer)
// {
//     NTSTATUS Status;
//     PVOID TableData = NULL;
//     ULONG TableSize, ResultSize;
//     PWNODE_ALL_DATA AllData;
//
//     /* Get the table data */
//     Status = WmipGetRawSMBiosTableData(OutBuffer ? &TableData : NULL, &TableSize);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WmipGetRawSMBiosTableData failed: 0x%08lx\n", Status);
//         return Status;
//     }
//
//     ResultSize = sizeof(WNODE_ALL_DATA) + TableSize;
//
//     /* Check if the caller provided a buffer */
//     if ((OutBuffer != NULL) && (*InOutBufferSize != 0))
//     {
//         /* Check if the buffer is large enough */
//         if (*InOutBufferSize < ResultSize)
//         {
//             DPRINT1("Buffer too small. Got %lu, need %lu\n",
//                     *InOutBufferSize, ResultSize);
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         /// FIXME: most of this is fubar
//         AllData = OutBuffer;
//         AllData->WnodeHeader.BufferSize = ResultSize;
//         AllData->WnodeHeader.ProviderId = 0;
//         AllData->WnodeHeader.Version = 0;
//         AllData->WnodeHeader.Linkage = 0; // last entry
//         //AllData->WnodeHeader.CountLost;
//         AllData->WnodeHeader.KernelHandle = NULL;
//         //AllData->WnodeHeader.TimeStamp;
//         AllData->WnodeHeader.Guid = MSSmBios_RawSMBiosTables_GUID;
//         //AllData->WnodeHeader.ClientContext;
//         AllData->WnodeHeader.Flags = WNODE_FLAG_FIXED_INSTANCE_SIZE;
//         AllData->DataBlockOffset = sizeof(WNODE_ALL_DATA);
//         AllData->InstanceCount = 1;
//         //AllData->OffsetInstanceNameOffsets;
//         AllData->FixedInstanceSize = TableSize;
//
//         RtlCopyMemory(AllData + 1, TableData, TableSize);
//     }
//
//     /* Set the size */
//     *InOutBufferSize = ResultSize;
//
//     /* Free the table buffer */
//     if (TableData != NULL)
//     {
//         ExFreePoolWithTag(TableData, 'BTMS');
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<SMBIOS21_ENTRY_POINT>() == 32);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, AnchorString) == 0);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, Checksum) == 4);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, Length) == 5);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, MajorVersion) == 6);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, MinorVersion) == 7);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, MaxStructureSize) == 8);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, EntryPointRevision) == 10);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, FormattedArea) == 11);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, AnchorString2) == 16);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, Checksum2) == 21);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, TableLength) == 22);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, TableAddress) == 24);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, NumberOfStructures) == 28);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, BCDRevision) == 30);
    assert!(core::mem::size_of::<SMBIOS30_ENTRY_POINT>() == 24);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, AnchorString) == 0);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Checksum) == 5);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Length) == 6);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, MajorVersion) == 7);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, MinorVersion) == 8);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Docref) == 9);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Revision) == 10);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Reserved) == 11);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, TableMaxSize) == 12);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, TableAddress) == 16);
    assert!(core::mem::size_of::<WNODE_ALL_DATA>() == 64);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, WnodeHeader) == 0);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, DataBlockOffset) == 48);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, InstanceCount) == 52);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, OffsetInstanceNameOffsets) == 56);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, FixedInstanceSize) == 60);
    assert!(core::mem::size_of::<MSSmBios_RawSMBiosTables>() == 12);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, Used20CallingMethod) == 0);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, SmbiosMajorVersion) == 1);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, SmbiosMinorVersion) == 2);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, DmiRevision) == 3);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, Size) == 4);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, SMBiosData) == 8);
    assert!(core::mem::size_of::<GUID>() == 16);
    assert!(core::mem::offset_of!(GUID, Data1) == 0);
    assert!(core::mem::offset_of!(GUID, Data2) == 4);
    assert!(core::mem::offset_of!(GUID, Data3) == 6);
    assert!(core::mem::offset_of!(GUID, Data4) == 8);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<SMBIOS21_ENTRY_POINT>() == 32);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, AnchorString) == 0);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, Checksum) == 4);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, Length) == 5);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, MajorVersion) == 6);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, MinorVersion) == 7);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, MaxStructureSize) == 8);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, EntryPointRevision) == 10);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, FormattedArea) == 11);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, AnchorString2) == 16);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, Checksum2) == 21);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, TableLength) == 22);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, TableAddress) == 24);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, NumberOfStructures) == 28);
    assert!(core::mem::offset_of!(SMBIOS21_ENTRY_POINT, BCDRevision) == 30);
    assert!(core::mem::size_of::<SMBIOS30_ENTRY_POINT>() == 24);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, AnchorString) == 0);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Checksum) == 5);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Length) == 6);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, MajorVersion) == 7);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, MinorVersion) == 8);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Docref) == 9);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Revision) == 10);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, Reserved) == 11);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, TableMaxSize) == 12);
    assert!(core::mem::offset_of!(SMBIOS30_ENTRY_POINT, TableAddress) == 16);
    assert!(core::mem::size_of::<WNODE_ALL_DATA>() == 64);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, WnodeHeader) == 0);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, DataBlockOffset) == 48);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, InstanceCount) == 52);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, OffsetInstanceNameOffsets) == 56);
    assert!(core::mem::offset_of!(WNODE_ALL_DATA, FixedInstanceSize) == 60);
    assert!(core::mem::size_of::<MSSmBios_RawSMBiosTables>() == 12);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, Used20CallingMethod) == 0);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, SmbiosMajorVersion) == 1);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, SmbiosMinorVersion) == 2);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, DmiRevision) == 3);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, Size) == 4);
    assert!(core::mem::offset_of!(MSSmBios_RawSMBiosTables, SMBiosData) == 8);
    assert!(core::mem::size_of::<GUID>() == 16);
    assert!(core::mem::offset_of!(GUID, Data1) == 0);
    assert!(core::mem::offset_of!(GUID, Data2) == 4);
    assert!(core::mem::offset_of!(GUID, Data3) == 6);
    assert!(core::mem::offset_of!(GUID, Data4) == 8);
};
// END OFFSET CHECKS
