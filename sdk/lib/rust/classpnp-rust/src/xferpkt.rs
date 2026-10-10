use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildPartialMdl, IoFreeIrp, IoFreeMdl, IoFreeWorkItem, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoGetPagingIoPriority, IoMarkIrpPending, IoReuseIrp, IoSetCompletionRoutine, IoSetHardErrorOrVerifyDevice, IoStartNextPacket, LARGE_INTEGER, MDL, OSVERSIONINFOEXW, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PKEVENT, POSVERSIONINFOW, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DEVICE_IO_CAPABILITY_DESCRIPTOR, RtlGetVersion, STORAGE_MINIPORT_DESCRIPTOR};
// DelayedWorkQueue, IoPagingPriorityHigh, IoPagingPriorityInvalid, STORAGE_DEVICE_IO_CAPABILITY_PROPERTY as STORAGE_PROPERTY_ID
// are not exported from ntkrnl_sys::ddk — use local constants defined below.
// Helper to access SrbType/AddressType from _STORAGE_ADAPTER_DESCRIPTOR (extended SDK fields).
#[allow(non_snake_case, non_camel_case_types, dead_code)]
#[repr(C)]
struct LocalStorageAdapterDescriptor {
    Version: u32,
    Size: u32,
    MaximumTransferLength: u32,
    MaximumPhysicalPages: u32,
    AlignmentMask: u32,
    AdapterUsesPio: u8,
    AdapterScansDown: u8,
    CommandQueueing: u8,
    AcceleratedTransfer: u8,
    BusType: u8,
    BusMajorVersion: u16,
    BusMinorVersion: u16,
    SrbType: u8,
    AddressType: u8,
}
#[inline(always)]
unsafe fn adapter_desc_ext(p: *mut core::ffi::c_void) -> *mut LocalStorageAdapterDescriptor {
    p as *mut LocalStorageAdapterDescriptor
}
use core::ffi::c_void;
use crate::types::NUM_MODESENSE_RETRIES;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IRP_MJ_SCSI, SL_WRITE_THROUGH, TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING};
use crate::types::{CDB, CLASS_PRIVATE_FDO_DATA, ClasspReceiveWriteUsingTokenInformationTransferPacketDone, DEV_USE_16BYTE_CDB, DefaultStorageRequestBlockAllocateRoutine, FOUR_BYTE, FUNCTIONAL_DEVICE_EXTENSION, MAX_OUTSTANDING_IO_PER_LUN_DEFAULT, MAX_WORKINGSET_TRANSFER_PACKETS_Client, MAX_WORKINGSET_TRANSFER_PACKETS_Server, MIN_INITIAL_TRANSFER_PACKETS, MIN_WORKINGSET_TRANSFER_PACKETS_Client, MIN_WORKINGSET_TRANSFER_PACKETS_Server_LowerBound, MIN_WORKINGSET_TRANSFER_PACKETS_Server_UpperBound, NUM_DRIVECAPACITY_RETRIES, NUM_LOCKMEDIAREMOVAL_RETRIES, NUM_MODESELECT_RETRIES, NUM_WRITE_USING_TOKEN_RETRIES, PCDB, PCLASS_PRIVATE_FDO_DATA, PCLASS_WORKING_SET, PCOMMON_DEVICE_EXTENSION, PEIGHT_BYTE, PFOUR_BYTE, PFUNCTIONAL_DEVICE_EXTENSION, PNL_SLIST_HEADER, PPNL_SLIST_HEADER, PSCSI_REQUEST_BLOCK, PSRB_HISTORY, PSTORAGE_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK_HEADER, PTRANSFER_PACKET, READ_CAPACITY_DATA_EX, REMOVE_COMPLETE, SCSIOP_MEDIUM_REMOVAL, SCSIOP_MODE_SELECT, SCSIOP_READ, SCSIOP_READ16, SCSIOP_READ_CAPACITY, SCSIOP_READ_CAPACITY16, SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION, SCSIOP_WRITE, SCSIOP_WRITE16, SCSI_REQUEST_BLOCK, SENSE_DATA_EX, SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION, SRB_CLASS_FLAGS_PAGING, SRB_FLAGS_ADAPTER_CACHE_ENABLE, SRB_FLAGS_DATA_OUT, SRB_FLAGS_DONT_START_NEXT_PACKET, SRB_FUNCTION_EXECUTE_SCSI, SRB_HISTORY, SRB_HISTORY_ITEM, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_QUEUE_FROZEN, SRB_STATUS_SUCCESS, STORAGE_REQUEST_BLOCK, TRANSFER_PACKET, _CDB, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_REQUEST_BLOCK_HEADER};
use ntkrnl_sys::ddk::{CCHAR, CLONG, CSHORT, IO_PAGING_PRIORITY, IRP, KEVENT, PCCHAR, PCSTR, PMDL, PRTL_OSVERSIONINFOW, STORAGE_PROPERTY_ID, ULONG64, _MDL, _STORAGE_PROPERTY_ID_StorageDeviceProperty as StorageDeviceProperty};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IRP_PAGING_IO, IRP_SYNCHRONOUS_PAGING_IO, STATUS_INTEGER_OVERFLOW, STATUS_MORE_PROCESSING_REQUIRED, TRACE_LEVEL_VERBOSE, VER_NT_DOMAIN_CONTROLLER, VER_NT_SERVER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_GENERAL, TRACE_FLAG_IOCTL, TRACE_FLAG_RW};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx, NonPagedPoolNxCacheAligned};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_PRIVATE_DATA, SENSE_BUFFER_SIZE_EX};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::debug::{DbgLogFlushInfo, DbgLogSendPacket};
use crate::types::{ClassGetDescriptor, ClasspCompleteIdleRequest, CreateStorageRequestBlock, DequeueDeferredClientIrp, HistoryInitializeRetryLogs, HistoryLogSendPacket, InterpretTransferPacketError, RetryTransferPacket, ServiceTransferRequest, StepLowMemRetry};
use ntkrnl_sys::ddk::{ExFreePool};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ClassMaxInterleavePerCriticalIo};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::srblib::{SrbGetDataBuffer, SrbGetDataTransferLength, SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength, SrbGetSrbFlags, SrbSetQueueSortKey, SrbSetScsiStatus};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{max};
use ntkrnl_sys::ddk::RtlAssert;

extern "C" {
    fn DbgPrint(format: *const u8, ...);
    fn __int2c();
    
}
extern "system" {
    fn NT_ASSERT(condition: bool);
    fn SrbSetRequestAttribute(srb: *mut core::ffi::c_void, attribute: u8);
    fn SrbSetCdbLength(srb: *mut core::ffi::c_void, length: u8);
    fn SrbSetOriginalRequest(srb: *mut core::ffi::c_void, irp: PIRP);
    fn SrbSetSenseInfoBuffer(srb: *mut core::ffi::c_void, sense_info: *mut core::ffi::c_void);
    fn SrbSetSenseInfoBufferLength(srb: *mut core::ffi::c_void, length: u8);
    fn SrbSetTimeOutValue(srb: *mut core::ffi::c_void, timeout: u32);
    fn SrbSetDataBuffer(srb: *mut core::ffi::c_void, buffer: *mut core::ffi::c_void);
    fn SrbSetDataTransferLength(srb: *mut core::ffi::c_void, length: u32);
    fn SrbAssignSrbFlags(srb: *mut core::ffi::c_void, flags: u32);
    fn SrbGetCdb(srb: *mut core::ffi::c_void) -> *mut CDB;
}

const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0x01;
const SRB_SIMPLE_TAG_REQUEST: u32 = 0x20;
const SRB_FLAGS_DATA_IN: u32 = 0x40;
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x08;
const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x0100;
const SCSIOP_MODE_SENSE: u8 = 0x1A;

// Constants not exported from ntkrnl_sys::ddk
const DELAYED_WORK_QUEUE: u32 = 1; // _WORK_QUEUE_TYPE_DelayedWorkQueue
const IO_PAGING_PRIORITY_INVALID: u32 = 0; // _IO_PAGING_PRIORITY_IoPagingPriorityInvalid
const IO_PAGING_PRIORITY_HIGH: u32 = 2;    // _IO_PAGING_PRIORITY_IoPagingPriorityHigh
const STORAGE_DEVICE_IO_CAPABILITY_PROPERTY: u32 = 48; // _STORAGE_PROPERTY_ID_STORAGE_DEVICE_IO_CAPABILITY_PROPERTY as STORAGE_PROPERTY_ID

// SRB_FLAGS_PORT_DRIVER_ALLOCSENSE / SRB_FLAGS_FREE_SENSE_BUFFER
const SRB_FLAGS_PORT_DRIVER_ALLOCSENSE: u32 = 0x00200000;
const SRB_FLAGS_FREE_SENSE_BUFFER: u32 = 0x0400;

extern "C" {
    fn RtlULongAdd(augend: u32, addend: u32, result: *mut u32) -> NTSTATUS;
    fn StorDebugPrint(debug_print_level: i32, debug_print_flags: u32, debug_message: PCCHAR, ...);
    fn Int64ShrlMod32(value: u64, shift: u32) -> u64;
    fn ClasspSrbSetOriginalIrp(srb: PSTORAGE_REQUEST_BLOCK_HEADER, irp: PIRP);
    fn ClasspIsIdleRequest(irp: PIRP) -> BOOLEAN;
    fn DbgLogReturnPacket(pkt: *mut TRANSFER_PACKET);
    fn DbgCheckReturnedPkt(pkt: *mut TRANSFER_PACKET);
    fn HistoryLogReturnedPacket(pkt: *mut TRANSFER_PACKET);
    fn MmPrepareMdlForReuse(mdl: PMDL);
    fn IoSetMasterIrpStatus(irp: PIRP, status: NTSTATUS);
    fn ClasspGetCurrentTime() -> LARGE_INTEGER;
    fn PORT_ALLOCATED_SENSE_EX(fdo_extension: PFUNCTIONAL_DEVICE_EXTENSION, srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> BOOLEAN;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdo_extension: PFUNCTIONAL_DEVICE_EXTENSION, srb: PSTORAGE_REQUEST_BLOCK_HEADER);
    fn SimpleInitSlistHdr(header: *mut SINGLE_LIST_ENTRY);
    fn SimplePushSlist(header: *mut SINGLE_LIST_ENTRY, entry: PSINGLE_LIST_ENTRY);
    fn SimplePopSlist(header: *mut SINGLE_LIST_ENTRY) -> PSINGLE_LIST_ENTRY;
    
    static SharedSystemTime: LARGE_INTEGER;
}
unsafe extern "system" {
    fn KeQueryHighestNodeNumber() -> USHORT;
    fn KeGetCurrentNodeNumber() -> USHORT;
    fn IoQueueWorkItemEx(io_work_item: PIO_WORKITEM, worker_routine: *const c_void, queue_type: u32, context: PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTransferPackets(Fdo: PDEVICE_OBJECT) -> NTSTATUS {
    let commonExt: PCOMMON_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let adapterDesc: PSTORAGE_ADAPTER_DESCRIPTOR =
        (*(*commonExt).PartitionZeroExtension).AdapterDescriptor;
    let mut devIoCapabilityDesc: PSTORAGE_DEVICE_IO_CAPABILITY_DESCRIPTOR =
        core::ptr::null_mut();
    let mut propertyId: STORAGE_PROPERTY_ID = StorageDeviceProperty;
    let mut osVersionInfo: OSVERSIONINFOEXW = OSVERSIONINFOEXW {
        dwOSVersionInfoSize: 0,
        dwMajorVersion: 0,
        dwMinorVersion: 0,
        dwBuildNumber: 0,
        dwPlatformId: 0,
        szCSDVersion: [0; 128],
        wServicePackMajor: 0,
        wServicePackMinor: 0,
        wSuiteMask: 0,
        wProductType: 0,
        wReserved: 0,
    };
    let mut hwMaxPages: ULONG;
    let mut arraySize: ULONG;
    let mut index: ULONG;
    let mut maxOutstandingIOPerLUN: ULONG;
    let mut minWorkingSetTransferPackets: ULONG;
    let mut maxWorkingSetTransferPackets: ULONG;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0"
                    as *const u8 as *const i8 as PCSTR,
                63,
                b"0\0" as *const u8 as PCSTR,
            );
            __int2c();
        }
    }

    // Precompute maximum transfer length
    if (*adapterDesc).MaximumTransferLength == 0 {
        DbgPrint(
            (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
            b"drivers/storage/class/classpnp/xferpkt.c\0"
                as *const u8 as *const i8 as PCSTR,
            68,
            b"adapterDesc->MaximumTransferLength\0" as *const u8 as PCSTR,
        );
        __int2c();
    }

    hwMaxPages = if (*adapterDesc).MaximumPhysicalPages != 0 {
        (*adapterDesc).MaximumPhysicalPages.wrapping_sub(1) as ULONG
    } else {
        0
    };

    (*fdoData).HwMaxXferLen = core::cmp::min(
        (*adapterDesc).MaximumTransferLength,
        (hwMaxPages << PAGE_SHIFT) as ULONG,
    );
    (*fdoData).HwMaxXferLen = core::cmp::max((*fdoData).HwMaxXferLen, PAGE_SIZE as ULONG);

    // Allocate per-node free packet lists
    arraySize = KeQueryHighestNodeNumber() as ULONG + 1;
    (*fdoData).FreeTransferPacketsLists = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        (core::mem::size_of::<PNL_SLIST_HEADER>() as SIZE_T).wrapping_mul(arraySize as SIZE_T),
        CLASS_TAG_PRIVATE_DATA,
    ) as PPNL_SLIST_HEADER;

    if (*fdoData).FreeTransferPacketsLists.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        return status;
    }

    index = 0;
    while index < arraySize {
        InitializeSListHead(
            (&raw mut (*(*fdoData).FreeTransferPacketsLists.offset(index as isize)).SListHeader).cast(),
        );
        (*(*fdoData).FreeTransferPacketsLists.offset(index as isize)).NumTotalTransferPackets = 0;
        (*(*fdoData).FreeTransferPacketsLists.offset(index as isize)).NumFreeTransferPackets = 0;
        index = index.wrapping_add(1);
    }

    InitializeListHead(&raw mut (*fdoData).AllTransferPacketsList);

    osVersionInfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as ULONG;
    status = RtlGetVersion(&raw mut osVersionInfo as PRTL_OSVERSIONINFOW);
    if !NT_SUCCESS(status) {
        DbgPrint(
            (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
            b"drivers/storage/class/classpnp/xferpkt.c\0"
                as *const u8 as *const i8 as PCSTR,
            104,
            b"NT_SUCCESS(status)\0" as *const u8 as PCSTR,
        );
        __int2c();
    }

    // Retrieve IO capability descriptor
    propertyId = STORAGE_DEVICE_IO_CAPABILITY_PROPERTY as STORAGE_PROPERTY_ID;
    status = ClassGetDescriptor(
        (*fdoExt).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
        &raw mut propertyId,
        &raw mut devIoCapabilityDesc as *mut PVOID,
    );

    if NT_SUCCESS(status) && !devIoCapabilityDesc.is_null() {
        maxOutstandingIOPerLUN = (*devIoCapabilityDesc).LunMaxIoCount;
        ExFreePool(devIoCapabilityDesc as PVOID);
        devIoCapabilityDesc = core::ptr::null_mut();
        #[cfg(DBG)]
        {
            (*fdoData).MaxOutstandingIOPerLUN = maxOutstandingIOPerLUN;
        }
    } else {
        maxOutstandingIOPerLUN = MAX_OUTSTANDING_IO_PER_LUN_DEFAULT;
        #[cfg(DBG)]
        {
            (*fdoData).MaxOutstandingIOPerLUN = 0;
        }
    }
    status = STATUS_SUCCESS;

    // Compute working set sizes based on SKU
    if osVersionInfo.wProductType != (VER_NT_DOMAIN_CONTROLLER) as u8
        && osVersionInfo.wProductType != (VER_NT_SERVER) as u8
    {
        // Client SKU
        minWorkingSetTransferPackets = MIN_WORKINGSET_TRANSFER_PACKETS_Client;
        maxWorkingSetTransferPackets = core::cmp::max(
            MAX_WORKINGSET_TRANSFER_PACKETS_Client,
            2u32.wrapping_mul(maxOutstandingIOPerLUN),
        );
    } else {
        // Server SKU
        minWorkingSetTransferPackets = core::cmp::max(
            MIN_WORKINGSET_TRANSFER_PACKETS_Server_LowerBound,
            core::cmp::min(
                MIN_WORKINGSET_TRANSFER_PACKETS_Server_UpperBound,
                maxOutstandingIOPerLUN,
            ),
        );
        maxWorkingSetTransferPackets = core::cmp::max(
            MAX_WORKINGSET_TRANSFER_PACKETS_Server,
            2u32.wrapping_mul(maxOutstandingIOPerLUN),
        );
    }

    (*fdoData).LocalMinWorkingSetTransferPackets = minWorkingSetTransferPackets;
    (*fdoData).LocalMaxWorkingSetTransferPackets = maxWorkingSetTransferPackets;

    // Class driver override
    if !(*(*commonExt).DriverExtension).WorkingSet.is_null() {
        let workingSet: PCLASS_WORKING_SET = (*(*commonExt).DriverExtension).WorkingSet;
        if (*workingSet).XferPacketsWorkingSetMinimum != 0 {
            (*fdoData).LocalMinWorkingSetTransferPackets = (*workingSet).XferPacketsWorkingSetMinimum;
            if (*fdoData).LocalMaxWorkingSetTransferPackets < (*fdoData).LocalMinWorkingSetTransferPackets {
                (*fdoData).LocalMaxWorkingSetTransferPackets = (*fdoData).LocalMinWorkingSetTransferPackets;
            }
        }
        if (*workingSet).XferPacketsWorkingSetMaximum != 0 {
            (*fdoData).LocalMaxWorkingSetTransferPackets = (*workingSet).XferPacketsWorkingSetMaximum;
            if (*fdoData).LocalMinWorkingSetTransferPackets > (*fdoData).LocalMaxWorkingSetTransferPackets {
                (*fdoData).LocalMinWorkingSetTransferPackets = (*fdoData).LocalMaxWorkingSetTransferPackets;
            }
        }
    }

    // Pre-create initial transfer packets per node
    index = 0;
    while index < arraySize {
        while (*(*fdoData).FreeTransferPacketsLists.offset(index as isize)).NumFreeTransferPackets
            < MIN_INITIAL_TRANSFER_PACKETS
        {
            let pkt: PTRANSFER_PACKET = NewTransferPacket(Fdo);
            if !pkt.is_null() {
                InterlockedIncrement(
                    &raw mut (*(*fdoData).FreeTransferPacketsLists.offset(index as isize))
                        .NumTotalTransferPackets as *mut i32,
                );
                (*pkt).AllocateNode = index;
                EnqueueFreeTransferPacket(Fdo, pkt);
            } else {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }
        }
        (*(*fdoData).FreeTransferPacketsLists.offset(index as isize)).DbgPeakNumTransferPackets =
            (*(*fdoData).FreeTransferPacketsLists.offset(index as isize)).NumTotalTransferPackets;
        index = index.wrapping_add(1);
    }

    // Pre-initialize SRB template
    if NT_SUCCESS(status) {
        if (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
            let mut ByteSize: ULONG = 0;
            status = CreateStorageRequestBlock(
                &raw mut (*fdoData).SrbTemplate as *mut PSTORAGE_REQUEST_BLOCK,
                (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).AddressType as USHORT,
                Some(DefaultStorageRequestBlockAllocateRoutine),
                &raw mut ByteSize,
                1,
                SrbExDataTypeScsiCdb16,
            );
            if NT_SUCCESS(status) {
                let srb = (*fdoData).SrbTemplate as PSTORAGE_REQUEST_BLOCK;
                (*srb).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI as ULONG;
            } else {
                if FALSE == 0 {
                    DbgPrint(
                        (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
                        b"drivers/storage/class/classpnp/xferpkt.c\0"
                            as *const u8 as *const i8 as PCSTR,
                        264,
                        b"FALSE\0" as *const u8 as PCSTR,
                    );
                    __int2c();
                }
            }
        } else {
            (*fdoData).SrbTemplate = ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<SCSI_REQUEST_BLOCK>() as SIZE_T,
                761426547u32,
            ) as PSTORAGE_REQUEST_BLOCK_HEADER;
            if (*fdoData).SrbTemplate.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                RtlZeroMemory(
                    (*fdoData).SrbTemplate as *mut c_void,
                    core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                );
                (*(*fdoData).SrbTemplate).Length =
                    core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                (*(*fdoData).SrbTemplate).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
            }
        }
    }

    if status == STATUS_SUCCESS {
        SrbSetRequestAttribute(
            (*fdoData).SrbTemplate as PVOID,
            (SRB_SIMPLE_TAG_REQUEST) as _,
        );
        SrbSetSenseInfoBufferLength(
            (*fdoData).SrbTemplate as PVOID,
            ((SENSE_BUFFER_SIZE_EX) as u32) as _,
        );
        SrbSetCdbLength((*fdoData).SrbTemplate as PVOID, 10);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DestroyAllTransferPackets(Fdo: PDEVICE_OBJECT) {
    let fdoExt = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;

    // PAGED_CODE check
    if (KeGetCurrentIrql() as u32) > (APC_LEVEL as u32) {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8).cast(),
            KeGetCurrentIrql() as i32,
        );
        DbgPrint(
            (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8).cast(),
            b"drivers/storage/class/classpnp/xferpkt.c\0".as_ptr() as *const i8,
            296,
            b"0\0".as_ptr() as *const i8,
        );
        __int2c();
    }

    if !(*fdoData).FreeTransferPacketsLists.is_null() {
        if !IsListEmpty(&raw mut (*fdoData).DeferredClientIrpList) {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0".as_ptr() as *const i8,
                304,
                b"IsListEmpty(&fdoData->DeferredClientIrpList)\0".as_ptr() as *const i8,
            );
            __int2c();
        }

        let arraySize = KeQueryHighestNodeNumber() as ULONG + 1;
        let mut index = 0u32;
        while index < arraySize {
            let mut pkt = DequeueFreeTransferPacketEx(Fdo, 0u8, index) as *mut TRANSFER_PACKET;
            while !pkt.is_null() {
                DestroyTransferPacket(pkt);
                InterlockedDecrement(
                    &raw mut (*(*fdoData).FreeTransferPacketsLists.offset(index as isize))
                        .NumTotalTransferPackets as *mut i32,
                );
                pkt = DequeueFreeTransferPacketEx(Fdo, 0u8, index) as *mut TRANSFER_PACKET;
            }
            if (*(*fdoData).FreeTransferPacketsLists.offset(index as isize))
                .NumTotalTransferPackets != 0
            {
                DbgPrint(
                    (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8).cast(),
                    b"drivers/storage/class/classpnp/xferpkt.c\0".as_ptr() as *const i8,
                    315,
                    b"fdoData->FreeTransferPacketsLists[index].NumTotalTransferPackets == 0\0".as_ptr() as *const i8,
                );
                __int2c();
            }
            index = index.wrapping_add(1);
        }
    }

    if !(*fdoData).SrbTemplate.is_null() {
        ExFreePool((*fdoData).SrbTemplate as *mut ::core::ffi::c_void);
        (*fdoData).SrbTemplate = core::ptr::null_mut::<_STORAGE_REQUEST_BLOCK_HEADER>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn NewTransferPacket(Fdo: PDEVICE_OBJECT) -> PTRANSFER_PACKET {
    let fdoExt = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let mut newPkt: PTRANSFER_PACKET = core::ptr::null_mut();
    let mut transferLength: ULONG = u32::MAX; // (ULONG)-1
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // integer overflow check for transferLength
    if status >= 0 {
        status = RtlULongAdd((*fdoData).HwMaxXferLen, PAGE_SIZE as ULONG, &mut transferLength);
        if status < 0 {
            StorDebugPrint(
                2 as i32,
                TRACE_FLAG_RW,
                b"Integer overflow in calculating transfer packet size.\0" as *const u8 as *const i8 as PCCHAR,
            );
            status = STATUS_INTEGER_OVERFLOW;
        }
    }

    // allocate the transfer packet itself
    if status >= 0 {
        newPkt = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<TRANSFER_PACKET>() as SIZE_T,
            1886277699u32, // tag 'pnPC'
        ) as PTRANSFER_PACKET;
        if newPkt.is_null() {
            StorDebugPrint(
                3 as i32,
                TRACE_FLAG_RW,
                b"Failed to allocate transfer packet.\0" as *const u8 as *const i8 as PCCHAR,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            core::ptr::write_bytes(newPkt as *mut u8, 0, core::mem::size_of::<TRANSFER_PACKET>());
            (*newPkt).AllocateNode = KeGetCurrentNodeNumber() as ULONG;

            if (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                status = CreateStorageRequestBlock(
                    &mut (*newPkt).Srb as *mut _ as *mut PSTORAGE_REQUEST_BLOCK,
                    (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).AddressType as u16,
                    Some(DefaultStorageRequestBlockAllocateRoutine as unsafe extern "C" fn(CLONG) -> PVOID),
                    core::ptr::null_mut::<ULONG>(),
                    1u32,
                    SrbExDataTypeScsiCdb16 as i32,
                );
            } else {
                (*newPkt).Srb = ExAllocatePoolWithTag(
                    NonPagedPoolNx,
                    core::mem::size_of::<SCSI_REQUEST_BLOCK>() as SIZE_T,
                    761426547u32, // tag '-brs'
                ) as PSTORAGE_REQUEST_BLOCK_HEADER;
                if (*newPkt).Srb.is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                }
            }

            if status != STATUS_SUCCESS {
                StorDebugPrint(
                    3 as i32,
                    TRACE_FLAG_RW,
                    b"Failed to allocate SRB.\0" as *const u8 as *const i8 as PCCHAR,
                );
                FREE_POOL((newPkt).cast());
                newPkt = core::ptr::null_mut();
            }
        }
    }

    // allocate an IRP for the packet
    if status >= 0 && !newPkt.is_null() {
        (*newPkt).Irp = IoAllocateIrp((*Fdo).StackSize, 0u8);
        if (*newPkt).Irp.is_null() {
            StorDebugPrint(
                3 as i32,
                TRACE_FLAG_RW,
                b"Failed to allocate IRP for transfer packet.\0" as *const u8 as *const i8 as PCCHAR,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    // allocate an MDL with an extra page entry
    if status >= 0 && !newPkt.is_null() {
        // NT_ASSERT equivalent from c2rust
        if !(transferLength != u32::MAX) {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0" as *const u8 as *const i8,
                421 as i32,
                b"transferLength != (ULONG)-1\0" as *const u8 as *const i8,
            );
            __int2c();
        }

        (*newPkt).PartialMdl = IoAllocateMdl(
            core::ptr::null_mut(),
            transferLength,
            0u8,
            0u8,
            core::ptr::null_mut::<IRP>(),
        );
        if (*newPkt).PartialMdl.is_null() {
            StorDebugPrint(
                3 as i32,
                TRACE_FLAG_RW,
                b"Failed to allocate MDL for transfer packet.\0" as *const u8 as *const i8 as PCCHAR,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            // NT_ASSERT for MDL size
            if !((*(*newPkt).PartialMdl).Size as i32
                >= (core::mem::size_of::<MDL>() + BYTES_TO_PAGES(((*fdoData).HwMaxXferLen) as usize) * core::mem::size_of::<PFN_NUMBER>()) as CSHORT as i32)
            {
                DbgPrint(
                    (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
                    b"drivers/storage/class/classpnp/xferpkt.c\0" as *const u8 as *const i8,
                    432 as i32,
                    b"newPkt->PartialMdl->Size >= (CSHORT)(sizeof(MDL) + BYTES_TO_PAGES(fdoData->HwMaxXferLen) * sizeof(PFN_NUMBER))\0" as *const u8 as *const i8,
                );
                __int2c();
            }
        }
    }

    // allocate per-packet retry history if needed
    if status >= 0 && !(*fdoData).InterpretSenseInfo.is_null() && !newPkt.is_null() {
        let mut historyByteCount: ULONG = 0;

        // NT_ASSERT(HistoryCount != 0)
        if !((*(*fdoData).InterpretSenseInfo).HistoryCount != 0) {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0" as *const u8 as *const i8,
                448 as i32,
                b"fdoData->InterpretSenseInfo->HistoryCount != 0\0" as *const u8 as *const i8,
            );
            __int2c();
        }

        historyByteCount = (core::mem::size_of::<SRB_HISTORY_ITEM>() as u32)
            .wrapping_mul((*(*fdoData).InterpretSenseInfo).HistoryCount as u32);
        historyByteCount = (historyByteCount as u64)
            .wrapping_add((core::mem::size_of::<SRB_HISTORY>() - core::mem::size_of::<SRB_HISTORY_ITEM>()) as u64) as ULONG;

        (*newPkt).RetryHistory = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            historyByteCount as SIZE_T,
            1752322115u32, // tag 'hrPC'
        ) as PSRB_HISTORY;

        if (*newPkt).RetryHistory.is_null() {
            StorDebugPrint(
                3 as i32,
                TRACE_FLAG_RW,
                b"Failed to allocate MDL for transfer packet.\0" as *const u8 as *const i8 as PCCHAR,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            HistoryInitializeRetryLogs(
                (*newPkt).RetryHistory,
                (*(*fdoData).InterpretSenseInfo).HistoryCount,
            );
        }
    }

    // enqueue the packet or clean up on failure
    if status >= 0 && !newPkt.is_null() {
        let mut oldIrql: KIRQL = 0;
        (*newPkt).Fdo = Fdo;
        (*newPkt).DbgPktId = InterlockedIncrement(&raw mut (*fdoData).DbgMaxPktId as *mut i32) as ULONG;

        KeAcquireSpinLock(&mut (*fdoData).SpinLock as *mut _ as *mut usize, &mut oldIrql);
        InsertTailList(
            &raw mut (*fdoData).AllTransferPacketsList,
            &raw mut (*newPkt).AllPktsListEntry,
        );
        KeReleaseSpinLock(&mut (*fdoData).SpinLock as *mut _ as *mut usize, oldIrql);
    } else {
        // free all resources in reverse order
        if !newPkt.is_null() {
            FREE_POOL(((*newPkt).RetryHistory).cast());
            if !(*newPkt).PartialMdl.is_null() {
                IoFreeMdl((*newPkt).PartialMdl);
            }
            if !(*newPkt).Irp.is_null() {
                IoFreeIrp((*newPkt).Irp);
            }
            if !(*newPkt).Srb.is_null() {
                FREE_POOL(((*newPkt).Srb).cast());
            }
            FREE_POOL((newPkt).cast());
            newPkt = core::ptr::null_mut();
        }
    }

    newPkt
}
#[no_mangle]
pub unsafe extern "C" fn DestroyTransferPacket(pkt: PTRANSFER_PACKET) {
    let fdo_ext = (*(*pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdo_data = (*fdo_ext).PrivateFdoData;
    let mut old_irql: KIRQL = 0;

    ntkrnl_sys::ASSERT!((*pkt).SlistEntry.Next.is_null());

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*fdo_data).SpinLock)).cast(),
        core::ptr::addr_of_mut!(old_irql),
    );

    ntkrnl_sys::ASSERT!(!IsListEmpty(core::ptr::addr_of!((*pkt).AllPktsListEntry)));
    ntkrnl_sys::ASSERT!(!IsListEmpty(core::ptr::addr_of!((*fdo_data).AllTransferPacketsList)));

    RemoveEntryList(core::ptr::addr_of_mut!((*pkt).AllPktsListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*pkt).AllPktsListEntry));

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdo_data).SpinLock)).cast(), old_irql);

    IoFreeMdl((*pkt).PartialMdl);
    IoFreeIrp((*pkt).Irp);

    if !(*pkt).RetryHistory.is_null() {
        ExFreePool((*pkt).RetryHistory as *mut core::ffi::c_void);
        (*pkt).RetryHistory = core::ptr::null_mut();
    }
    if !(*pkt).Srb.is_null() {
        ExFreePool((*pkt).Srb as *mut core::ffi::c_void);
        (*pkt).Srb = core::ptr::null_mut();
    }
    if !pkt.is_null() {
        ExFreePool(pkt as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn EnqueueFreeTransferPacket(Fdo: PDEVICE_OBJECT, Pkt: PTRANSFER_PACKET) {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let allocateNode: ULONG;
    let mut oldIrql: KIRQL = 0;

    // NT_ASSERT(!Pkt->SlistEntry.Next)
    if !(*Pkt).SlistEntry.Next.is_null() {
        // In debug builds, NT_ASSERT would break; for translation we just keep the check.
        // If it's a macro that always evaluates, we can keep it as a call.
        // For safety, we'll leave it as a panic if not null.
        RtlAssert(
            b"!Pkt->SlistEntry.Next\0".as_ptr() as *mut core::ffi::c_void,
            b"xferpkt.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }

    allocateNode = (*Pkt).AllocateNode;

    InterlockedPushEntrySList(
        core::ptr::addr_of_mut!((*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).SListHeader).cast(),
        (core::ptr::addr_of_mut!((*Pkt).SlistEntry)).cast(),
    );
    InterlockedIncrement(
        core::ptr::addr_of_mut!((*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumFreeTransferPackets)
            as *mut LONG,
    );

    if (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumFreeTransferPackets
        >= (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets
    {
        // 1. Immediate snap down to UPPER threshold
        if (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets
            > (*fdoData).LocalMaxWorkingSetTransferPackets
        {
            let isRemoved: ULONG;
            let workItem: PIO_WORKITEM = IoAllocateWorkItem(Fdo);

            isRemoved = ClassAcquireRemoveLockEx(Fdo, workItem as PVOID, b"\0".as_ptr(), 0);

            if !workItem.is_null() && isRemoved == 0 {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_GENERAL,
                    b"EnqueueFreeTransferPacket: Device (%p), queuing work item to clean up free transfer packets.\n\0"
                        .as_ptr() as PCHAR,
                    Fdo,
                );

                IoQueueWorkItemEx(
                    workItem,
                    CleanupTransferPacketToWorkingSetSizeWorker as unsafe extern "system" fn(PVOID, PVOID, PIO_WORKITEM) as *const c_void,
                    DELAYED_WORK_QUEUE,
                    allocateNode as ULONG_PTR as PVOID,
                );
            } else {
                if !workItem.is_null() {
                    IoFreeWorkItem(workItem);
                }
                if isRemoved != REMOVE_COMPLETE {
                    ClassReleaseRemoveLock(Fdo, workItem as PIRP);
                }

                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"EnqueueFreeTransferPacket: Device (%p), Failed to allocate memory for the work item.\n\0"
                        .as_ptr() as PCHAR,
                    Fdo,
                );

                CleanupTransferPacketToWorkingSetSize(Fdo, 1u8, allocateNode);
            }
        }

        // 2. Lazily work down to LOWER threshold
        if (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets
            > (*fdoData).LocalMinWorkingSetTransferPackets
        {
            let mut pktToDelete: PTRANSFER_PACKET = core::ptr::null_mut();

            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_RW,
                b"Exiting stress, lazily freeing one of %d/%d packets from node %d.\0".as_ptr() as PCHAR,
                (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets,
                (*fdoData).LocalMinWorkingSetTransferPackets,
                allocateNode,
            );

            KeAcquireSpinLock(
                core::ptr::addr_of_mut!((*fdoData).SpinLock) as *mut _ as *mut usize,
                core::ptr::addr_of_mut!(oldIrql),
            );

            if (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumFreeTransferPackets
                >= (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets
                && (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets
                    > (*fdoData).LocalMinWorkingSetTransferPackets
            {
                pktToDelete = DequeueFreeTransferPacketEx(Fdo, 0u8, allocateNode);
                if !pktToDelete.is_null() {
                    InterlockedDecrement(
                        core::ptr::addr_of_mut!(
                            (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets
                        ) as *mut LONG,
                    );
                } else {
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_RW,
                        b"Extremely unlikely condition (non-fatal): %d packets dequeued at once for Fdo %p. NumTotalTransferPackets=%d (2). Node=%d\0"
                            .as_ptr() as PCHAR,
                        (*fdoData).LocalMinWorkingSetTransferPackets,
                        Fdo,
                        (*(*fdoData).FreeTransferPacketsLists.offset((allocateNode) as isize)).NumTotalTransferPackets,
                        allocateNode,
                    );
                }
            }

            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*fdoData).SpinLock) as *mut _ as *mut usize,
                oldIrql,
            );

            if !pktToDelete.is_null() {
                DestroyTransferPacket(pktToDelete);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DequeueFreeTransferPacket(
    Fdo: PDEVICE_OBJECT,
    AllocIfNeeded: BOOLEAN,
) -> PTRANSFER_PACKET {
    DequeueFreeTransferPacketEx(Fdo, AllocIfNeeded, KeGetCurrentNodeNumber() as ULONG)
}
#[no_mangle]
pub unsafe extern "C" fn DequeueFreeTransferPacketEx(
    Fdo: PDEVICE_OBJECT,
    AllocIfNeeded: BOOLEAN,
    Node: ULONG,
) -> PTRANSFER_PACKET {
    let fdoExt = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let mut pkt: PTRANSFER_PACKET = core::ptr::null_mut();

    let slistEntry = InterlockedPopEntrySList(
        (core::ptr::addr_of_mut!((*(*fdoData).FreeTransferPacketsLists.offset(Node as isize)).SListHeader)).cast(),
    );

    if !slistEntry.is_null() {
        (*slistEntry).Next = core::ptr::null_mut::<SLIST_ENTRY>();
        pkt = (slistEntry as ULONG_PTR)
            .wrapping_sub(
                core::mem::offset_of!(TRANSFER_PACKET, SlistEntry) as ULONG_PTR,
            ) as PTRANSFER_PACKET;
        InterlockedDecrement(
            core::ptr::addr_of_mut!(
                (*(*fdoData).FreeTransferPacketsLists.offset(Node as isize)).NumFreeTransferPackets
            ) as *mut LONG,
        );

        // when dequeuing the packet, also reset the history data
        if !(*pkt).RetryHistory.is_null() {
            HistoryInitializeRetryLogs(
                (*pkt).RetryHistory,
                (*(*pkt).RetryHistory).TotalHistoryCount,
            );
        }
    } else if AllocIfNeeded != 0 {
        pkt = NewTransferPacket(Fdo);
        if !pkt.is_null() {
            InterlockedIncrement(
                core::ptr::addr_of_mut!(
                    (*(*fdoData).FreeTransferPacketsLists.offset(Node as isize)).NumTotalTransferPackets
                ) as *mut LONG,
            );
            let list = &mut (*(*fdoData).FreeTransferPacketsLists.offset(Node as isize));
            let peak = core::ptr::addr_of_mut!(list.DbgPeakNumTransferPackets);
            if list.NumTotalTransferPackets > *peak {
                *peak = list.NumTotalTransferPackets;
            }
        } else {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"DequeueFreeTransferPacket: packet allocation failed\0".as_ptr() as PCHAR,
            );
        }
    } else {
        pkt = core::ptr::null_mut::<TRANSFER_PACKET>();
    }

    pkt
}
#[no_mangle]
pub unsafe extern "C" fn SetupReadWriteTransferPacket(
    Pkt: PTRANSFER_PACKET,
    Buf: PVOID,
    Len: ULONG,
    DiskLocation: LARGE_INTEGER,
    OriginalIrp: PIRP,
) {
    let fdoExt = (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension = (*(*Pkt).Fdo).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let origCurSp = IoGetCurrentIrpStackLocation(OriginalIrp);
    let majorFunc = (*origCurSp).MajorFunction;
    let mut logicalBlockAddr: LARGE_INTEGER = core::mem::zeroed();
    let timeoutValue = (*fdoExt).TimeOutValue;
    logicalBlockAddr.QuadPart = Int64ShrlMod32(
        DiskLocation.QuadPart as u64,
        (*fdoExt).SectorShift as u32,
    ) as i64;
    let mut numTransferBlocks = Len >> (*fdoExt).SectorShift;

    (*Pkt).DriverUsesStartIO = if (*(*commonExtension).DriverExtension)
        .InitData
        .ClassStartIo
        .is_none()
    {
        0u8
    } else {
        1u8
    };

    let srbLength: ULONG;
    if (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbLength = (*((*fdoData).SrbTemplate as PVOID as PSTORAGE_REQUEST_BLOCK)).SrbLength;
        if !((*((*Pkt).Srb as PVOID as PSTORAGE_REQUEST_BLOCK)).SrbLength >= srbLength) {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0"
                    .as_ptr() as *const i8,
                752i32,
                b"((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength\0"
                    .as_ptr() as *const i8,
            );
            __int2c();
        }
    } else {
        srbLength = (*(*fdoData).SrbTemplate).Length as ULONG;
    }

    RtlCopyMemory(
        (*Pkt).Srb as *mut core::ffi::c_void,
        (*fdoData).SrbTemplate as *const core::ffi::c_void,
        srbLength as usize,
    );
    SrbSetDataBuffer((*Pkt).Srb as PVOID, Buf);
    SrbSetDataTransferLength((*Pkt).Srb as PVOID, Len);
    SrbSetQueueSortKey(
        (*Pkt).Srb as PVOID,
        logicalBlockAddr.__bindgen_anon_1.LowPart,
    );
    if logicalBlockAddr.QuadPart > 0xFFFFFFFFi64 {
        SrbSetQueueSortKey((*Pkt).Srb as PVOID, 0xFFFFFFFFu32);
    }
    SrbSetOriginalRequest((*Pkt).Srb as PVOID, ((*Pkt).Irp as PVOID).cast());
    SrbSetSenseInfoBuffer(
        (*Pkt).Srb as PVOID,
        &raw mut (*Pkt).SrbErrorSenseData as PVOID,
    );
    SrbSetTimeOutValue((*Pkt).Srb as PVOID, timeoutValue);

    let pCdb = SrbGetCdb((*Pkt).Srb as PVOID);
    if !pCdb.is_null() {
        if TEST_FLAG(((*fdoExt).DeviceFlags) as u32, DEV_USE_16BYTE_CDB) != 0u8 {
            let val1 = &raw mut (*pCdb).CDB16.as_mut().LogicalBlock as PEIGHT_BYTE;
            let val2 = &raw mut logicalBlockAddr as PEIGHT_BYTE;
            (*val1).__bindgen_anon_1.as_mut().Byte7 = (*val2).__bindgen_anon_1.as_ref().Byte0;
            (*val1).__bindgen_anon_1.as_mut().Byte6 = (*val2).__bindgen_anon_1.as_ref().Byte1;
            (*val1).__bindgen_anon_1.as_mut().Byte5 = (*val2).__bindgen_anon_1.as_ref().Byte2;
            (*val1).__bindgen_anon_1.as_mut().Byte4 = (*val2).__bindgen_anon_1.as_ref().Byte3;
            (*val1).__bindgen_anon_1.as_mut().Byte3 = (*val2).__bindgen_anon_1.as_ref().Byte4;
            (*val1).__bindgen_anon_1.as_mut().Byte2 = (*val2).__bindgen_anon_1.as_ref().Byte5;
            (*val1).__bindgen_anon_1.as_mut().Byte1 = (*val2).__bindgen_anon_1.as_ref().Byte6;
            (*val1).__bindgen_anon_1.as_mut().Byte0 = (*val2).__bindgen_anon_1.as_ref().Byte7;

            let val1_0 = &raw mut (*pCdb).CDB16.as_mut().TransferLength as PFOUR_BYTE;
            let val2_0 = &raw mut numTransferBlocks as PFOUR_BYTE;
            (*val1_0).__bindgen_anon_1.as_mut().Byte3 = (*val2_0).__bindgen_anon_1.as_ref().Byte0;
            (*val1_0).__bindgen_anon_1.as_mut().Byte2 = (*val2_0).__bindgen_anon_1.as_ref().Byte1;
            (*val1_0).__bindgen_anon_1.as_mut().Byte1 = (*val2_0).__bindgen_anon_1.as_ref().Byte2;
            (*val1_0).__bindgen_anon_1.as_mut().Byte0 = (*val2_0).__bindgen_anon_1.as_ref().Byte3;

            (*pCdb).CDB16.as_mut().OperationCode = if majorFunc as i32 == (IRP_MJ_READ) as i32 {
                SCSIOP_READ16 as UCHAR
            } else {
                SCSIOP_WRITE16 as UCHAR
            };
            SrbSetCdbLength((*Pkt).Srb as PVOID, 16u8);
        } else {
            let p_lo =
                &raw const logicalBlockAddr.__bindgen_anon_1.LowPart as *const FOUR_BYTE;
            (*pCdb).CDB10.as_mut().LogicalBlockByte0 = (*p_lo).__bindgen_anon_1.as_ref().Byte3;
            (*pCdb).CDB10.as_mut().LogicalBlockByte1 = (*p_lo).__bindgen_anon_1.as_ref().Byte2;
            (*pCdb).CDB10.as_mut().LogicalBlockByte2 = (*p_lo).__bindgen_anon_1.as_ref().Byte1;
            (*pCdb).CDB10.as_mut().LogicalBlockByte3 = (*p_lo).__bindgen_anon_1.as_ref().Byte0;
            let p_nb = &raw const numTransferBlocks as *const FOUR_BYTE;
            (*pCdb).CDB10.as_mut().TransferBlocksMsb = (*p_nb).__bindgen_anon_1.as_ref().Byte1;
            (*pCdb).CDB10.as_mut().TransferBlocksLsb = (*p_nb).__bindgen_anon_1.as_ref().Byte0;
            (*pCdb).CDB10.as_mut().OperationCode = if majorFunc as i32 == (IRP_MJ_READ) as i32 {
                SCSIOP_READ as UCHAR
            } else {
                SCSIOP_WRITE as UCHAR
            };
        }
    }

    SrbAssignSrbFlags((*Pkt).Srb as PVOID, (*fdoExt).SrbFlags);
    if TEST_FLAG((*OriginalIrp).Flags, IRP_PAGING_IO) != 0u8
        || TEST_FLAG((*OriginalIrp).Flags, IRP_SYNCHRONOUS_PAGING_IO) != 0u8
    {
        SrbSetSrbFlags(
            (*Pkt).Srb as PVOID,
            SRB_CLASS_FLAGS_PAGING as ULONG,
        );
    }
    SrbSetSrbFlags(
        (*Pkt).Srb as PVOID,
        if majorFunc as i32 == (IRP_MJ_READ) as i32 {
            SRB_FLAGS_DATA_IN as ULONG
        } else {
            SRB_FLAGS_DATA_OUT as ULONG
        },
    );

    if majorFunc as i32 == (IRP_MJ_WRITE) as i32
        && TEST_FLAG(((*origCurSp).Flags) as u32, SL_WRITE_THROUGH) != 0u8
        && !pCdb.is_null()
    {
        (*pCdb)
            .CDB10
            .as_mut().set_ForceUnitAccess((*fdoExt).CdbForceUnitAccess as UCHAR);
    } else {
        SrbSetSrbFlags(
            (*Pkt).Srb as PVOID,
            SRB_FLAGS_ADAPTER_CACHE_ENABLE as ULONG,
        );
    }

    (*Pkt).BufPtrCopy = Buf as PUCHAR;
    (*Pkt).BufLenCopy = Len;
    (*Pkt).TargetLocationCopy = DiskLocation;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = (*fdoData).MaxNumberOfIoRetries as ULONG;
    (*Pkt).SyncEventPtr = core::ptr::null_mut::<KEVENT>();
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = TRUE as BOOLEAN;

    if !pCdb.is_null() {
        DbgLogFlushInfo(
            fdoData,
            TRUE as BOOLEAN,
            (*pCdb).CDB10.as_ref().ForceUnitAccess() as BOOLEAN,
            FALSE as BOOLEAN,
        );
    } else {
        DbgLogFlushInfo(
            fdoData,
            TRUE as BOOLEAN,
            FALSE as BOOLEAN,
            FALSE as BOOLEAN,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn SubmitTransferPacket(Pkt: PTRANSFER_PACKET) -> NTSTATUS {
    let commonExtension = (*(*Pkt).Fdo).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let nextDevObj = (*commonExtension).LowerDeviceObject;
    let fdoExtension = (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let mut idleRequest: BOOLEAN = 0u8;

    if !((*(*Pkt).Irp).CurrentLocation as i32 == (*(*Pkt).Irp).StackCount as i32 + 1) {
        DbgPrint(
            (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR).cast(),
            b"drivers/storage/class/classpnp/xferpkt.c\0".as_ptr()
                as *const ::core::ffi::c_char,
            859i32,
            b"Pkt->Irp->CurrentLocation == Pkt->Irp->StackCount+1\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        __int2c();
    }

    IoReuseIrp((*Pkt).Irp, STATUS_NOT_SUPPORTED);

    let nextSp = IoGetNextIrpStackLocation((*Pkt).Irp);
    (*nextSp).MajorFunction = IRP_MJ_SCSI as u8;
    (*nextSp).Parameters.Scsi.Srb = (*Pkt).Srb as PVOID as *mut _;

    SrbSetScsiStatus((*Pkt).Srb as PVOID, 0u8);
    (*(*Pkt).Srb).SrbStatus = 0u8;
    SrbSetSenseInfoBufferLength((*Pkt).Srb as PVOID, ((SENSE_BUFFER_SIZE_EX as u8) as u32) as _);

    if (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0 {
        let origCurSp = IoGetCurrentIrpStackLocation((*Pkt).OriginalIrp);
        (*nextSp).Flags = (*origCurSp).Flags;
    }

    if (*Pkt).UsePartialMdl as i32 == (FALSE) as i32 {
        (*(*Pkt).Irp).MdlAddress = (*(*Pkt).OriginalIrp).MdlAddress;
    } else {
        IoBuildPartialMdl(
            (*(*Pkt).OriginalIrp).MdlAddress as PMDL,
            (*Pkt).PartialMdl,
            SrbGetDataBuffer((*Pkt).Srb as PVOID),
            SrbGetDataTransferLength((*Pkt).Srb as PVOID),
        );
        (*(*Pkt).Irp).MdlAddress = (*Pkt).PartialMdl as *mut _MDL;
    }

    DbgLogSendPacket(Pkt as *mut TRANSFER_PACKET);
    if !(*Pkt).RetryHistory.is_null() {
        HistoryLogSendPacket(Pkt as *mut TRANSFER_PACKET);
    }

    ClasspSrbSetOriginalIrp((*Pkt).Srb, (*Pkt).OriginalIrp as PVOID as PIRP);

    if (*fdoData).IdlePrioritySupported as i32 == (TRUE) as i32 {
        idleRequest = ClasspIsIdleRequest((*Pkt).OriginalIrp);
        if idleRequest != 0 {
            InterlockedIncrement(&raw mut (*fdoData).ActiveIdleIoCount as *mut i32);
        } else {
            InterlockedIncrement(&raw mut (*fdoData).ActiveIoCount as *mut i32);
        }
    }

    IoSetCompletionRoutine(
        (*Pkt).Irp,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS,
            _,
        >(TransferPktComplete)),
        Pkt as PVOID,
        1u8,
        1u8,
        1u8,
    );

    IoCallDriver(nextDevObj, (*Pkt).Irp)
}

#[no_mangle]
pub unsafe extern "system" fn TransferPktComplete(
    NullFdo: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let _ = NullFdo; // UNREFERENCED_PARAMETER
    let pkt: PTRANSFER_PACKET = Context as PTRANSFER_PACKET;
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*(*pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let mut packetDone: BOOLEAN = 0u8;
    let mut idleRequest: BOOLEAN = 0u8;
    let mut transferLength: ULONG;
    let mut completionTime: LARGE_INTEGER = LARGE_INTEGER {
        __bindgen_anon_1: ::core::mem::zeroed(),
    };
    let mut lastIoCompletionTime: ULONGLONG;

    DbgLogReturnPacket(pkt);
    DbgCheckReturnedPkt(pkt);
    if !(*pkt).RetryHistory.is_null() {
        HistoryLogReturnedPacket(pkt);
    }

    completionTime = ClasspGetCurrentTime();

    // Snapshot and update LastIoCompletionTime. _WIN64 (__REACTOS__): a plain
    // volatile 64-bit read + write. 32-bit: InterlockedExchangeNoFence64 -- two
    // plain 32-bit halves there could pair the high word of one completion time
    // with the low word of another.
    #[cfg(target_pointer_width = "64")]
    {
        lastIoCompletionTime = core::ptr::read_volatile(
            &raw mut (*fdoData).LastIoCompletionTime.QuadPart as *mut ULONG64,
        );
        core::ptr::write_volatile(
            &raw mut (*fdoData).LastIoCompletionTime.QuadPart as *mut ULONG64,
            completionTime.QuadPart as ULONG64,
        );
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        let last = &*(&raw mut (*fdoData).LastIoCompletionTime.QuadPart
            as *const core::sync::atomic::AtomicI64);
        lastIoCompletionTime =
            last.swap(completionTime.QuadPart as i64, core::sync::atomic::Ordering::Relaxed) as ULONG64;
    }

    if (*fdoData).IdlePrioritySupported == 1u8 {
        idleRequest = ClasspIsIdleRequest((*pkt).OriginalIrp);
        if idleRequest != 0 {
            InterlockedDecrement(&raw mut (*fdoData).ActiveIdleIoCount as *mut LONG);
            NT_ASSERT((*fdoData).ActiveIdleIoCount >= 0);
        } else {
            (*fdoData).LastNonIdleIoTime = completionTime;
            InterlockedDecrement(&raw mut (*fdoData).ActiveIoCount as *mut LONG);
            NT_ASSERT((*fdoData).ActiveIoCount >= 0);
        }
    }

    // Reuse partial MDL if used
    if (*pkt).UsePartialMdl != 0 {
        MmPrepareMdlForReuse((*pkt).PartialMdl);
    }

    let srb_status_masked =
        (*(*pkt).Srb).SrbStatus
            & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as u8;
    if srb_status_masked == (SRB_STATUS_SUCCESS) as u8 {
        NT_ASSERT((*Irp).IoStatus.__bindgen_anon_1.Status >= 0);
        transferLength = SrbGetDataTransferLength((*pkt).Srb as PVOID);
        (*fdoData).LoggedTURFailureSinceLastIO = 0u8;
        NT_ASSERT(PORT_ALLOCATED_SENSE_EX(fdoExt, (*pkt).Srb) == 0);
        InterlockedExchangeAdd(
            &raw mut (*(*pkt).OriginalIrp).IoStatus.Information as *mut LONG,
            transferLength as LONG,
        );
        if (*pkt).InLowMemRetry != 0
            || ((*pkt).DriverUsesStartIO != 0 && (*pkt).LowMemRetry_remainingBufLen > 0)
        {
            packetDone = StepLowMemRetry(pkt);
        } else {
            packetDone = 1u8;
        }
    } else {
        // The packet failed, possibly retried
        let mut shouldRetry: BOOLEAN = 0u8;

        if (*Irp).IoStatus.__bindgen_anon_1.Status >= 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
        }

        if (*pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0 {
            IoMarkIrpPending((*pkt).OriginalIrp);
        }

        shouldRetry = InterpretTransferPacketError(pkt);

        if (*(*pkt).Srb).SrbStatus & (SRB_STATUS_QUEUE_FROZEN) as u8 != 0 {
            ClassReleaseQueue((*pkt).Fdo);
        }

        if (*Irp).IoStatus.__bindgen_anon_1.Status >= 0 {
            NT_ASSERT(shouldRetry == 0);
            InterlockedExchangeAdd(
                &raw mut (*(*pkt).OriginalIrp).IoStatus.Information as *mut LONG,
                SrbGetDataTransferLength((*pkt).Srb as PVOID) as LONG,
            );
            if (*pkt).InLowMemRetry != 0
                || ((*pkt).DriverUsesStartIO != 0
                    && (*pkt).LowMemRetry_remainingBufLen > 0)
            {
                packetDone = StepLowMemRetry(pkt);
            } else {
                packetDone = 1u8;
            }
        } else if shouldRetry != 0 && (*pkt).NumRetries > 0 {
            packetDone = RetryTransferPacket(pkt);
        } else if shouldRetry != 0 && !(*pkt).RetryHistory.is_null() {
            packetDone = RetryTransferPacket(pkt);
        } else {
            packetDone = 1u8;
        }
    }

    if packetDone != 0 {
        let mut numPacketsRemaining: LONG;
        let mut deferredIrp: PIRP = core::ptr::null_mut();
        let fdo: PDEVICE_OBJECT = (*pkt).Fdo;
        let mut uniqueAddr: UCHAR = 0;

        ClassAcquireRemoveLock(fdo, (&raw mut uniqueAddr as PVOID).cast());

        if PORT_ALLOCATED_SENSE_EX(fdoExt, (*pkt).Srb) != 0 {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_RW,
                b"Freeing port-allocated sense buffer for pkt %ph.\0".as_ptr() as PCCHAR,
                pkt,
            );
            FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExt, (*pkt).Srb);
            SrbSetSenseInfoBuffer(
                (*pkt).Srb as PVOID,
                &raw mut (*pkt).SrbErrorSenseData as PVOID,
            );
            SrbSetSenseInfoBufferLength(
                (*pkt).Srb as PVOID,
                ((core::mem::size_of::<SENSE_DATA_EX>() as UCHAR) as u32) as _,
            );
        } else {
            NT_ASSERT(
                SrbGetSenseInfoBuffer((*pkt).Srb as PVOID)
                    == (&raw mut (*pkt).SrbErrorSenseData as PVOID),
            );
            NT_ASSERT(
                SrbGetSenseInfoBufferLength((*pkt).Srb as PVOID) as usize
                    <= core::mem::size_of::<SENSE_DATA_EX>(),
            );
        }

        RtlZeroMemory(
            &raw mut (*pkt).SrbErrorSenseData as *mut core::ffi::c_void,
            core::mem::size_of::<SENSE_DATA_EX>(),
        );

        IoSetMasterIrpStatus((*pkt).OriginalIrp, (*Irp).IoStatus.__bindgen_anon_1.Status);

        if !((*Irp).IoStatus.__bindgen_anon_1.Status >= 0) {
            if IoIsErrorUserInduced((*Irp).IoStatus.__bindgen_anon_1.Status) != 0
                && (*pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0
                && !(*(*pkt).OriginalIrp).Tail.Overlay.Thread.is_null()
            {
                IoSetHardErrorOrVerifyDevice((*pkt).OriginalIrp, fdo);
            }
        }

        numPacketsRemaining = InterlockedDecrement(
            &raw mut (*(*pkt).OriginalIrp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] as *mut PVOID as *mut LONG,
        ) as LONG;

        if numPacketsRemaining > 0 {
            // more packets remain
        } else {
            NT_ASSERT(numPacketsRemaining == 0);
            if (*pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0 {
                let priority: IO_PAGING_PRIORITY = if (*(*pkt).OriginalIrp).Flags & IRP_PAGING_IO as ULONG != 0 {
                    IoGetPagingIoPriority((*pkt).OriginalIrp)
                } else {
                    IO_PAGING_PRIORITY_INVALID as IO_PAGING_PRIORITY
                };
                let mut oldIrql: KIRQL = 0;

                if (*(*pkt).OriginalIrp).IoStatus.__bindgen_anon_1.Status >= 0 {
                    NT_ASSERT(
                        (*(*pkt).OriginalIrp).IoStatus.Information as ULONG
                            == (*IoGetCurrentIrpStackLocation((*pkt).OriginalIrp))
                                .Parameters
                                .Read
                                .Length,
                    );
                    ClasspPerfIncrementSuccessfulIo(fdoExt);
                }

                ClassReleaseRemoveLock(fdo, (*pkt).OriginalIrp);

                if (*(*pkt).Irp).PendingReturned != 0 {
                    IoMarkIrpPending((*pkt).OriginalIrp);
                }

                ClassCompleteRequest(fdo, (*pkt).OriginalIrp, (IO_DISK_INCREMENT as CCHAR) as _);

                if priority == IO_PAGING_PRIORITY_HIGH as IO_PAGING_PRIORITY {
                    KeAcquireSpinLock((&raw mut (*fdoData).SpinLock).cast(), &raw mut oldIrql);

                    if (*fdoData).MaxInterleavedNormalIo < ClassMaxInterleavePerCriticalIo {
                        (*fdoData).MaxInterleavedNormalIo = 0;
                    } else {
                        (*fdoData).MaxInterleavedNormalIo -= ClassMaxInterleavePerCriticalIo;
                    }
                    (*fdoData).NumHighPriorityPagingIo = (*fdoData)
                        .NumHighPriorityPagingIo
                        .wrapping_sub(1);
                    if (*fdoData).NumHighPriorityPagingIo == 0 {
                        // Inline KeQuerySystemTime
                        (*fdoData).ThrottleStopTime.QuadPart =
                            core::ptr::read_volatile(
                                core::ptr::addr_of!(SharedSystemTime.QuadPart),
                            );
                        let period_quad = (*fdoData).ThrottleStopTime.QuadPart
                            - (*fdoData).ThrottleStartTime.QuadPart;
                        (*fdoData).LongestThrottlePeriod.QuadPart =
                            max(
                                (*fdoData).LongestThrottlePeriod.QuadPart,
                                period_quad,
                            );
                    }

                    KeReleaseSpinLock((&raw mut (*fdoData).SpinLock).cast(), oldIrql);
                }

                if idleRequest != 0 {
                    ClasspCompleteIdleRequest(fdoExt);
                }

                if !(*(*fdoExt).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension)
                    .InitData
                    .ClassStartIo
                    .is_none()
                {
                    if SrbGetSrbFlags((*pkt).Srb as PVOID) as ULONG
                        & SRB_FLAGS_DONT_START_NEXT_PACKET
                        != 0
                    {
                        TracePrint(
                            TRACE_LEVEL_INFORMATION,
                            TRACE_FLAG_RW,
                            b"SRB_FLAGS_DONT_START_NEXT_PACKET should never be set here (?)\0"
                                .as_ptr() as PCCHAR,
                        );
                    } else {
                        KeRaiseIrql(DISPATCH_LEVEL, &raw mut oldIrql);
                        IoStartNextPacket(fdo, 1u8);
                        KeLowerIrql(oldIrql);
                    }
                }
            }
        }

        if !(*pkt).SyncEventPtr.is_null() {
            KeSetEvent(((*pkt).SyncEventPtr as PKEVENT).cast(), 0, 0u8);
            (*pkt).SyncEventPtr = core::ptr::null_mut();
        }

        if (*pkt).ContinuationRoutine.is_some() {
            ((*pkt).ContinuationRoutine.expect("non-null function pointer"))(
                (*pkt).ContinuationContext,
            );
            (*pkt).ContinuationRoutine = None;
        }

        (*pkt).UsePartialMdl = 0u8;
        (*pkt).InLowMemRetry = 0u8;

        EnqueueFreeTransferPacket(fdo, pkt);

        deferredIrp = DequeueDeferredClientIrp(fdo);
        if !deferredIrp.is_null() {
            ServiceTransferRequest(fdo, deferredIrp, 1u8);
        }

        ClassReleaseRemoveLock(fdo, (&raw mut uniqueAddr as PVOID).cast());
    }

    STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "C" fn SetupEjectionTransferPacket(
    mut Pkt: *mut TRANSFER_PACKET,
    mut PreventMediaRemoval: BOOLEAN,
    mut SyncEventPtr: PKEVENT,
    mut OriginalIrp: PIRP,
) {
    let mut fdoExt: PFUNCTIONAL_DEVICE_EXTENSION =
        (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let mut pCdb: PCDB = ::core::ptr::null_mut::<_CDB>();
    let mut srbLength: ULONG = 0;
    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR).cast(),
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                1325 as ::core::ffi::c_int,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
            __int2c();
        } else {
        };
    }
    if (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).SrbType as u32
        == SRB_TYPE_STORAGE_REQUEST_BLOCK
    {
        srbLength = (*((*fdoData).SrbTemplate as PVOID as PSTORAGE_REQUEST_BLOCK)).SrbLength;
        if !((*((*Pkt).Srb as PVOID as PSTORAGE_REQUEST_BLOCK)).SrbLength >= srbLength) {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                1329 as ::core::ffi::c_int,
                b"((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
            __int2c();
        } else {
        };
    } else {
        srbLength = (*(*fdoData).SrbTemplate).Length as ULONG;
    }
    core::ptr::copy_nonoverlapping(
        (*fdoData).SrbTemplate as *const u8,
        (*Pkt).Srb as *mut u8,
        srbLength as usize,
    );
    SrbSetRequestAttribute((*Pkt).Srb as PVOID, ((SRB_SIMPLE_TAG_REQUEST as UCHAR) as u32) as _);
    SrbSetCdbLength((*Pkt).Srb as PVOID, 6 as UCHAR);
    SrbSetOriginalRequest((*Pkt).Srb as PVOID, ((*Pkt).Irp as PVOID).cast());
    SrbSetSenseInfoBuffer(
        (*Pkt).Srb as PVOID,
        &raw mut (*Pkt).SrbErrorSenseData as PVOID,
    );
    SrbSetSenseInfoBufferLength(
        (*Pkt).Srb as PVOID,
        ((::core::mem::size_of::<SENSE_DATA_EX>() as UCHAR) as u32) as _,
    );
    SrbSetTimeOutValue((*Pkt).Srb as PVOID, (*fdoExt).TimeOutValue);
    SrbAssignSrbFlags(
        (*Pkt).Srb as PVOID,
        (*fdoExt).SrbFlags
            | SRB_FLAGS_DISABLE_SYNCH_TRANSFER as ULONG
            | SRB_FLAGS_NO_QUEUE_FREEZE as ULONG,
    );
    pCdb = SrbGetCdb((*Pkt).Srb as PVOID);
    if !pCdb.is_null() {
        (*pCdb).MEDIA_REMOVAL.as_mut().OperationCode = SCSIOP_MEDIUM_REMOVAL as UCHAR;
        (*pCdb)
            .MEDIA_REMOVAL
            .as_mut().set_Prevent(PreventMediaRemoval as UCHAR as UCHAR);
    }
    (*Pkt).BufPtrCopy = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    (*Pkt).BufLenCopy = 0 as ULONG;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_LOCKMEDIAREMOVAL_RETRIES as ULONG;
    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = FALSE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn SetupModeSenseTransferPacket(
    Pkt: *mut TRANSFER_PACKET,
    SyncEventPtr: PKEVENT,
    ModeSenseBuffer: *mut core::ffi::c_void,
    ModeSenseBufferLen: u8,
    PageMode: u8,
    SubPage: u8,
    OriginalIrp: PIRP,
    PageControl: u8,
) {
    PAGED_CODE!();

    let fdoExt: *mut FUNCTIONAL_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension as _;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let pCdb: *mut CDB;
    let srbLength: u32;

    if (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        let srb_template_cast = (*fdoData).SrbTemplate as *mut STORAGE_REQUEST_BLOCK;
        srbLength = (*srb_template_cast).SrbLength;
        let srb_cast = (*Pkt).Srb as *mut STORAGE_REQUEST_BLOCK;
        NT_ASSERT((*srb_cast).SrbLength >= srbLength);
    } else {
        srbLength = ((*(*fdoData).SrbTemplate).Length) as u32;
    }

    RtlCopyMemory(
        (*Pkt).Srb as *mut _,
        (*fdoData).SrbTemplate as *const _,
        srbLength as usize,
    );

    SrbSetRequestAttribute(((*Pkt).Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
    SrbSetCdbLength(((*Pkt).Srb).cast(), 6u8);
    SrbSetOriginalRequest(((*Pkt).Srb).cast(), (*Pkt).Irp);
    SrbSetSenseInfoBuffer(((*Pkt).Srb).cast(), core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as _);
    SrbSetSenseInfoBufferLength(((*Pkt).Srb).cast(), (core::mem::size_of_val(&(*Pkt).SrbErrorSenseData) as u32) as _);
    SrbSetTimeOutValue(((*Pkt).Srb).cast(), (*fdoExt).TimeOutValue);
    SrbSetDataBuffer(((*Pkt).Srb).cast(), ModeSenseBuffer);
    SrbSetDataTransferLength(((*Pkt).Srb).cast(), ModeSenseBufferLen as u32);

    let flags = (*fdoExt).SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE;
    SrbAssignSrbFlags(((*Pkt).Srb).cast(), flags);

    pCdb = SrbGetCdb(((*Pkt).Srb).cast());
    if !pCdb.is_null() {
        (*pCdb).MODE_SENSE.as_mut().OperationCode = SCSIOP_MODE_SENSE;
        (*pCdb).MODE_SENSE.as_mut().set_PageCode(PageMode);
        (*pCdb).MODE_SENSE.as_mut().SubPageCode = SubPage;
        (*pCdb).MODE_SENSE.as_mut().set_Pc(PageControl);
        (*pCdb).MODE_SENSE.as_mut().AllocationLength = ModeSenseBufferLen;
    }

    (*Pkt).BufPtrCopy = (ModeSenseBuffer).cast();
    (*Pkt).BufLenCopy = (ModeSenseBufferLen) as u32;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_MODESENSE_RETRIES;
    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0;
}
#[no_mangle]
pub unsafe extern "C" fn SetupModeSelectTransferPacket(
    Pkt: *mut TRANSFER_PACKET,
    SyncEventPtr: PKEVENT,
    ModeSelectBuffer: PVOID,
    ModeSelectBufferLen: u8,
    SavePages: u8,
    OriginalIrp: PIRP,
) {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let mut pCdb: PCDB = ::core::ptr::null_mut::<_CDB>();
    let srbLength: u32;
    if (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbLength = (*((*fdoData).SrbTemplate as *mut STORAGE_REQUEST_BLOCK)).SrbLength;
        if !((*((*Pkt).Srb as *mut STORAGE_REQUEST_BLOCK)).SrbLength >= srbLength) {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR).cast(),
                b"drivers/storage/class/classpnp/xferpkt.c\0".as_ptr() as *const i8,
                1440i32,
                b"((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    } else {
        srbLength = (*(*fdoData).SrbTemplate).Length as u32;
    }
    RtlCopyMemory(
        (*Pkt).Srb as *mut ::core::ffi::c_void,
        (*fdoData).SrbTemplate as *const ::core::ffi::c_void,
        srbLength as usize,
    );
    SrbSetRequestAttribute((*Pkt).Srb as PVOID, ((SRB_SIMPLE_TAG_REQUEST as u8) as u32) as _);
    SrbSetCdbLength((*Pkt).Srb as PVOID, 6u8);
    SrbSetOriginalRequest((*Pkt).Srb as PVOID, ((*Pkt).Irp as PVOID).cast());
    SrbSetSenseInfoBuffer(
        (*Pkt).Srb as PVOID,
        ::core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as PVOID,
    );
    SrbSetSenseInfoBufferLength(
        (*Pkt).Srb as PVOID,
        ((::core::mem::size_of::<SENSE_DATA_EX>() as u8) as u32) as _,
    );
    SrbSetTimeOutValue((*Pkt).Srb as PVOID, (*fdoExt).TimeOutValue);
    SrbSetDataBuffer((*Pkt).Srb as PVOID, ModeSelectBuffer);
    SrbSetDataTransferLength((*Pkt).Srb as PVOID, ModeSelectBufferLen as u32);
    SrbAssignSrbFlags(
        (*Pkt).Srb as PVOID,
        (*fdoExt).SrbFlags
            | SRB_FLAGS_DATA_OUT as u32
            | SRB_FLAGS_DISABLE_SYNCH_TRANSFER as u32
            | SRB_FLAGS_NO_QUEUE_FREEZE as u32,
    );
    pCdb = SrbGetCdb((*Pkt).Srb as PVOID);
    if !pCdb.is_null() {
        (*pCdb).MODE_SELECT.as_mut().OperationCode = SCSIOP_MODE_SELECT as u8;
        (*pCdb).MODE_SELECT.as_mut().set_SPBit(SavePages as u8);
        (*pCdb).MODE_SELECT.as_mut().set_PFBit(1u8);
        (*pCdb).MODE_SELECT.as_mut().ParameterListLength = ModeSelectBufferLen;
    }
    (*Pkt).BufPtrCopy = ModeSelectBuffer as PUCHAR;
    (*Pkt).BufLenCopy = ModeSelectBufferLen as u32;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_MODESELECT_RETRIES;
    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0u8;
}
#[no_mangle]
pub unsafe extern "C" fn SetupDriveCapacityTransferPacket(
    Pkt: *mut TRANSFER_PACKET,
    ReadCapacityBuffer: PVOID,
    ReadCapacityBufferLen: ULONG,
    SyncEventPtr: PKEVENT,
    OriginalIrp: PIRP,
    Use16ByteCdb: BOOLEAN,
) {
    let fdoExt = (*((*Pkt).Fdo)).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let timeoutValue = (*fdoExt).TimeOutValue;
    let srbLength: ULONG;

    if (*adapter_desc_ext((*fdoExt).AdapterDescriptor as *mut _)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbLength = (*((*fdoData).SrbTemplate as PSTORAGE_REQUEST_BLOCK)).SrbLength;
        if !((*((*Pkt).Srb as PSTORAGE_REQUEST_BLOCK)).SrbLength >= srbLength) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr(),
                b"drivers/storage/class/classpnp/xferpkt.c\0"
                    .as_ptr(),
                1495i32,
                b"((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength\0".as_ptr(),
            );
            __int2c();
        }
    } else {
        srbLength = (*(*fdoData).SrbTemplate).Length as ULONG;
    }

    core::ptr::copy_nonoverlapping(
        (*fdoData).SrbTemplate as *const u8,
        (*Pkt).Srb as *mut u8,
        srbLength as usize,
    );

    SrbSetRequestAttribute((*Pkt).Srb as PVOID, ((SRB_SIMPLE_TAG_REQUEST as UCHAR) as u32) as _);
    SrbSetOriginalRequest((*Pkt).Srb as PVOID, ((*Pkt).Irp as PVOID).cast());
    SrbSetSenseInfoBuffer(
        (*Pkt).Srb as PVOID,
        core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as PVOID,
    );
    SrbSetSenseInfoBufferLength(
        (*Pkt).Srb as PVOID,
        ((core::mem::size_of::<SENSE_DATA_EX>() as UCHAR) as u32) as _,
    );
    SrbSetTimeOutValue((*Pkt).Srb as PVOID, timeoutValue);
    SrbSetDataBuffer((*Pkt).Srb as PVOID, ReadCapacityBuffer);
    SrbSetDataTransferLength((*Pkt).Srb as PVOID, ReadCapacityBufferLen);
    SrbAssignSrbFlags(
        (*Pkt).Srb as PVOID,
        (*fdoExt).SrbFlags
            | SRB_FLAGS_DATA_IN as ULONG
            | SRB_FLAGS_DISABLE_SYNCH_TRANSFER as ULONG
            | SRB_FLAGS_NO_QUEUE_FREEZE as ULONG,
    );

    let pCdb = SrbGetCdb((*Pkt).Srb as PVOID);
    if !pCdb.is_null() {
        if Use16ByteCdb == 1u8 {
            if !(ReadCapacityBufferLen as usize >= core::mem::size_of::<READ_CAPACITY_DATA_EX>()) {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr(),
                    b"drivers/storage/class/classpnp/xferpkt.c\0".as_ptr(),
                    1516i32,
                    b"ReadCapacityBufferLen >= sizeof(READ_CAPACITY_DATA_EX)\0".as_ptr(),
                );
                __int2c();
            }
            SrbSetCdbLength((*Pkt).Srb as PVOID, 16u8);
            (*pCdb).CDB16.as_mut().OperationCode = SCSIOP_READ_CAPACITY16 as UCHAR;

            // REVERSE_BYTES expansion
            let target = core::ptr::addr_of_mut!((*pCdb).CDB16.as_mut().TransferLength) as *mut u8;
            let source = &ReadCapacityBufferLen as *const ULONG as *const u8;
            core::ptr::write(target.add(3), core::ptr::read(source.add(0)));
            core::ptr::write(target.add(2), core::ptr::read(source.add(1)));
            core::ptr::write(target.add(1), core::ptr::read(source.add(2)));
            core::ptr::write(target.add(0), core::ptr::read(source.add(3)));

            (*pCdb).AsByte.as_mut()[1] = 0x10u8;
        } else {
            SrbSetCdbLength((*Pkt).Srb as PVOID, 10u8);
            (*pCdb).CDB10.as_mut().OperationCode = SCSIOP_READ_CAPACITY as UCHAR;
        }
    }

    (*Pkt).BufPtrCopy = ReadCapacityBuffer as PUCHAR;
    (*Pkt).BufLenCopy = ReadCapacityBufferLen;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_DRIVECAPACITY_RETRIES as ULONG;
    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0u8;
}
#[no_mangle]
pub unsafe extern "system" fn CleanupTransferPacketToWorkingSetSizeWorker(
    Fdo: PVOID,
    Context: PVOID,
    IoWorkItem: PIO_WORKITEM,
) {
    let node: ULONG = Context as ULONG_PTR as ULONG;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const u8,
            (KeGetCurrentIrql()) as i32,
        );
        // ASSERT always fires (c2rust pattern: if (0 == 0) which is always true)
        __int2c();
    }

    CleanupTransferPacketToWorkingSetSize(Fdo as PDEVICE_OBJECT, 0u8, node);

    // Release the remove lock acquired in EnqueueFreeTransferPacket
    ClassReleaseRemoveLock(Fdo as PDEVICE_OBJECT, IoWorkItem as PIRP);

    if !IoWorkItem.is_null() {
        IoFreeWorkItem(IoWorkItem);
    }
}

// Required externs for this function – placed here as they are otherwise missing.
// The caller should ensure these are visible or the compilation unit must have them defined.
#[no_mangle]
pub unsafe extern "C" fn CleanupTransferPacketToWorkingSetSize(
    Fdo: PDEVICE_OBJECT,
    LimitNumPktToDelete: BOOLEAN,
    Node: ULONG,
) {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;
    let mut pktList = SINGLE_LIST_ENTRY {
        Next: core::ptr::null_mut::<SINGLE_LIST_ENTRY>(),
    };
    let mut slistEntry: PSINGLE_LIST_ENTRY = core::ptr::null_mut::<SINGLE_LIST_ENTRY>();
    let mut pktToDelete: PTRANSFER_PACKET = core::ptr::null_mut::<TRANSFER_PACKET>();
    let list = &(*(*fdoData).FreeTransferPacketsLists.offset((Node) as isize));
    let mut requiredNumPktToDelete = list.NumTotalTransferPackets.wrapping_sub((*fdoData).LocalMaxWorkingSetTransferPackets);
    if LimitNumPktToDelete != 0 {
        if requiredNumPktToDelete > 8192 {
            requiredNumPktToDelete = 8192;
        }
    }
    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_RW,
        b"CleanupTransferPacketToWorkingSetSize (%p): Exiting stress, block freeing %d packets.\0"
            .as_ptr() as PCHAR,
        Fdo as *const core::ffi::c_void,
        requiredNumPktToDelete as core::ffi::c_ulong,
    );
    SimpleInitSlistHdr(&raw mut pktList);
    KeAcquireSpinLock((&raw mut (*fdoData).SpinLock).cast(), &raw mut oldIrql);
    while (*(*fdoData).FreeTransferPacketsLists.offset((Node) as isize)).NumFreeTransferPackets
        >= (*(*fdoData).FreeTransferPacketsLists.offset((Node) as isize)).NumTotalTransferPackets
        && (*(*fdoData).FreeTransferPacketsLists.offset((Node) as isize)).NumTotalTransferPackets
            > (*fdoData).LocalMaxWorkingSetTransferPackets
        && {
            let non_zero = requiredNumPktToDelete != 0;
            requiredNumPktToDelete = requiredNumPktToDelete.wrapping_sub(1);
            non_zero
        }
    {
        pktToDelete = DequeueFreeTransferPacketEx(Fdo, 0u8, Node);
        if !pktToDelete.is_null() {
            SimplePushSlist(
                &raw mut pktList,
                &raw mut (*pktToDelete).SlistEntry as PSINGLE_LIST_ENTRY,
            );
            InterlockedDecrement(
                &raw mut (*(*fdoData).FreeTransferPacketsLists.offset((Node) as isize))
                    .NumTotalTransferPackets as *mut LONG,
            );
        } else {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_RW,
                b"Extremely unlikely condition (non-fatal): %d packets dequeued at once for Fdo %p. NumTotalTransferPackets=%d (1). Node=%d\0"
                    .as_ptr() as PCHAR,
                (*fdoData).LocalMaxWorkingSetTransferPackets as core::ffi::c_ulong,
                Fdo as *const core::ffi::c_void,
                (*(*fdoData).FreeTransferPacketsLists.offset((Node) as isize)).NumTotalTransferPackets as core::ffi::c_ulong,
                Node as core::ffi::c_ulong,
            );
            break;
        }
    }
    KeReleaseSpinLock((&raw mut (*fdoData).SpinLock).cast(), oldIrql);
    slistEntry = SimplePopSlist(&raw mut pktList) as PSINGLE_LIST_ENTRY;
    while !slistEntry.is_null() {
        pktToDelete = ((slistEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(TRANSFER_PACKET, SlistEntry) as ULONG_PTR,
        )) as PTRANSFER_PACKET;
        DestroyTransferPacket(pktToDelete);
        slistEntry = SimplePopSlist(&raw mut pktList) as PSINGLE_LIST_ENTRY;
    }
}

unsafe fn ClasspSetupReceiveWriteUsingTokenInformationTransferPacket(
    OffloadWriteContext: *mut core::ffi::c_void,
    Pkt: *mut TRANSFER_PACKET,
    Length: u32,
    ReceiveWriteUsingTokenInformationBuffer: *mut u8,
    OriginalIrp: *mut IRP,
    ListIdentifier: u32,
) {
    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspSetupReceiveWriteUsingTokenInformationTransferPacket (%p): Entering function. Irp %p\n\0"
            .as_ptr() as *const i8,
        (*Pkt).Fdo,
        OriginalIrp,
    );

    let fdoExt = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let adapter_desc = (*fdoExt).AdapterDescriptor;
    let srb_length: u32;

    if (*adapter_desc_ext(adapter_desc as *mut _)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srb_length = (*((*fdoData).SrbTemplate as PVOID as *mut STORAGE_REQUEST_BLOCK)).SrbLength;
        NT_ASSERT(
            (*((*Pkt).Srb as PVOID as *mut STORAGE_REQUEST_BLOCK)).SrbLength >= srb_length,
        );
    } else {
        srb_length = ((*(*fdoData).SrbTemplate).Length) as u32;
    }

    core::ptr::copy_nonoverlapping(
        (*fdoData).SrbTemplate as *const u8,
        (*Pkt).Srb as *mut u8,
        srb_length as usize,
    );

    SrbSetRequestAttribute(((*Pkt).Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
    SrbSetCdbLength(((*Pkt).Srb).cast(), 16);
    SrbSetOriginalRequest(((*Pkt).Srb).cast(), (*Pkt).Irp);
    SrbSetSenseInfoBuffer(
        ((*Pkt).Srb).cast(),
        core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as *mut core::ffi::c_void,
    );
    SrbSetSenseInfoBufferLength(
        ((*Pkt).Srb).cast(),
        (core::mem::size_of_val(&(*Pkt).SrbErrorSenseData) as u32) as _,
    );
    SrbSetTimeOutValue(((*Pkt).Srb).cast(), (*fdoExt).TimeOutValue);
    SrbSetDataBuffer(((*Pkt).Srb).cast(), (ReceiveWriteUsingTokenInformationBuffer).cast());
    SrbSetDataTransferLength(((*Pkt).Srb).cast(), Length);

    SrbAssignSrbFlags(
        ((*Pkt).Srb).cast(),
        (*fdoExt).SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE,
    );

    let pCdb = SrbGetCdb(((*Pkt).Srb).cast());

    if !pCdb.is_null() {
        let mut length = Length;
        let mut list_identifier = ListIdentifier;

        (*pCdb).RECEIVE_TOKEN_INFORMATION.as_mut().OperationCode = (SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION) as u8;
        (*pCdb).RECEIVE_TOKEN_INFORMATION.as_mut().set_ServiceAction((SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION) as u8);

        REVERSE_BYTES(
            core::ptr::addr_of_mut!((*pCdb).RECEIVE_TOKEN_INFORMATION.as_mut().AllocationLength) as *mut u32,
            &length as *const u32,
        );
        REVERSE_BYTES(
            core::ptr::addr_of_mut!((*pCdb).RECEIVE_TOKEN_INFORMATION.as_mut().ListIdentifier) as *mut u32,
            &list_identifier as *const u32,
        );
    }

    (*Pkt).BufPtrCopy = ReceiveWriteUsingTokenInformationBuffer;
    (*Pkt).BufLenCopy = Length;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_WRITE_USING_TOKEN_RETRIES;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0u8;

    (*Pkt).ContinuationRoutine = Some(core::mem::transmute(ClasspReceiveWriteUsingTokenInformationTransferPacketDone as unsafe extern "C" fn(*mut _)));

    (*Pkt).ContinuationContext = OffloadWriteContext;

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspSetupReceiveWriteUsingTokenInformationTransferPacket (%p): Exiting function with Irp %p\n\0"
            .as_ptr() as *const i8,
        (*Pkt).Fdo,
        OriginalIrp,
    );
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    
}
extern "system" {
    fn ClassAcquireRemoveLockEx(DeviceObject: PDEVICE_OBJECT, Tag: PVOID, File: *const u8, Line: ULONG) -> ULONG;
    fn ClassReleaseRemoveLock(Device: PDEVICE_OBJECT, Irp: PIRP);
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn KeGetCurrentIrql() -> KIRQL;
}
