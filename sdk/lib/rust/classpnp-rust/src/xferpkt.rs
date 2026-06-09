use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DelayedWorkQueue, IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildPartialMdl, IoFreeIrp, IoFreeMdl, IoFreeWorkItem, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoGetPagingIoPriority, IoMarkIrpPending, IoPagingPriorityHigh, IoPagingPriorityInvalid, IoReuseIrp, IoSetCompletionRoutine, IoSetHardErrorOrVerifyDevice, IoStartNextPacket, LARGE_INTEGER, MDL, OSVERSIONINFOEXW, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PKEVENT, POSVERSIONINFOW, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DEVICE_IO_CAPABILITY_DESCRIPTOR, RtlGetVersion, STORAGE_MINIPORT_DESCRIPTOR, StorageDeviceIoCapabilityProperty};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_CDB as CDB};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, IO_PAGING_PRIORITY, IO_STACK_LOCATION, IRP, KEVENT, OSVERSIONINFOW, PMDL, STORAGE_DEVICE_IO_CAPABILITY_DESCRIPTOR, STORAGE_PROPERTY_ID};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_LEVEL_INFORMATION};

extern "C" {
    fn TracePrint(level: u32, flag: u32, msg: *const i8, ...);
    fn max(a: i32, b: i32) -> i32;
}
extern "C" {
    fn NT_ASSERT(expr: i32)
    fn RtlULongAdd(a: u32, b: u32, result: *mut u32) -> i32;
    fn KeGetCurrentNodeNumber() -> u32;
    fn CreateStorageRequestBlock( srb_ptr: *mut *mut STORAGE_REQUEST_BLOCK, addr_type: u32, alloc_routine: Option<extern "system" fn(*mut c_void, u32, u32, u32) -> *mut STORAGE_REQUEST_BLOCK>, context: *mut c_void, count: u32, ... ) -> i32;
    fn HistoryInitializeRetryLogs(history: *mut SRB_HISTORY, count: u32);
    fn FREE_POOL(ptr: *mut c_void);
    fn DefaultStorageRequestBlockAllocateRoutine( ctx: *mut c_void, arg1: u32, arg2: u32, arg3: u32, ) -> *mut STORAGE_REQUEST_BLOCK;
    fn InterlockedIncrement(addend: *mut i32) -> i32;
    fn Int64ShrlMod32(val: i64, shift: u32) -> i64;
    fn REVERSE_BYTES(dst: *mut u32, src: *const u32);
    fn REVERSE_BYTES_QUAD(dst: *mut u64, src: *const i64);
    fn SrbGetCdb(srb: *mut core::ffi::c_void) -> *mut CDB;
    fn SrbSetCdbLength(srb: *mut core::ffi::c_void, length: u32);
    fn SrbSetDataBuffer(srb: *mut core::ffi::c_void, buf: *mut core::ffi::c_void);
    fn SrbSetDataTransferLength(srb: *mut core::ffi::c_void, len: u32);
    fn SrbSetQueueSortKey(srb: *mut core::ffi::c_void, key: u32);
    fn SrbSetOriginalRequest(srb: *mut core::ffi::c_void, irp: *mut IRP);
    fn SrbSetSenseInfoBuffer(srb: *mut core::ffi::c_void, buffer: *mut core::ffi::c_void);
    fn SrbSetTimeOutValue(srb: *mut core::ffi::c_void, timeout: u32);
    fn SrbAssignSrbFlags(srb: *mut core::ffi::c_void, flags: u32);
    fn SrbSetSrbFlags(srb: *mut core::ffi::c_void, flags: u32);
    fn DBGLOGFLUSHINFO(fdoData: *mut CLASS_PRIVATE_FDO_DATA, flag1: u8, flag2: u8, flag3: u8);
    fn SrbSetScsiStatus(srb: *mut SCSI_REQUEST_BLOCK, status: u8);
    fn SrbSetSenseInfoBufferLength(srb: *mut SCSI_REQUEST_BLOCK, length: u8);
    fn DBGLOGSENDPACKET(pkt: *mut c_void);
    fn HISTORYLOGSENDPACKET(pkt: *mut c_void);
    fn ClasspSrbSetOriginalIrp(srb: *mut c_void, original_irp: *mut c_void);
    fn ClasspIsIdleRequest(irp: PIRP) -> u8;
    fn SrbGetDataBuffer(srb: *mut c_void) -> *mut c_void;
    fn SrbGetDataTransferLength(srb: *mut c_void) -> u32;
    fn SrbSetRequestAttribute(srb: *mut core::ffi::c_void, attribute: u32);
    fn PAGED_CODE();
    fn ClasspReceiveWriteUsingTokenInformationTransferPacketDone(/* args unknown */);
    fn KeQueryHighestNodeNumber() -> u32;
    fn SimpleInitSlistHdr(Head: PSINGLE_LIST_ENTRY);
    fn SimplePushSlist(Head: PSINGLE_LIST_ENTRY, Entry: PSINGLE_LIST_ENTRY) -> PSINGLE_LIST_ENTRY;
    fn SimplePopSlist(Head: PSINGLE_LIST_ENTRY) -> PSINGLE_LIST_ENTRY;
    fn ClassAcquireRemoveLock(Fdo: PDEVICE_OBJECT, Irp: PIRP) -> ULONG;
    fn ClassReleaseRemoveLock(Fdo: PDEVICE_OBJECT, Irp: PIRP);
    fn IoQueueWorkItemEx(WorkItem: PIO_WORKITEM, WorkerRoutine: unsafe extern "system" fn(Fdo: PDEVICE_OBJECT, Context: PVOID), QueueType: u32, Context: PVOID) -> NTSTATUS;
    fn ClassGetDescriptor(DeviceObject: *mut DEVICE_OBJECT, PropertyId: *const STORAGE_PROPERTY_ID, Buffer: *mut *mut core::ffi::c_void) -> i32;
    fn NT_SUCCESS(status: i32) -> i32;
    fn ClassCompleteRequest(Fdo: PDEVICE_OBJECT, Irp: PIRP, PriorityBoost: i32);
    fn ClassReleaseQueue(Fdo: PDEVICE_OBJECT);
    fn ClasspCompleteIdleRequest(fdoExt: PFUNCTIONAL_DEVICE_EXTENSION);
    fn ClasspGetCurrentTime() -> LARGE_INTEGER;
    fn ClasspPerfIncrementSuccessfulIo(fdoExt: PFUNCTIONAL_DEVICE_EXTENSION);
    fn DBGCHECKRETURNEDPKT(pkt: PTRANSFER_PACKET);
    fn DBGLOGRETURNPACKET(pkt: PTRANSFER_PACKET);
    fn DequeueDeferredClientIrp(Fdo: PDEVICE_OBJECT) -> PIRP;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExt: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PSCSI_REQUEST_BLOCK);
    fn HISTORYLOGRETURNEDPACKET(pkt: PTRANSFER_PACKET);
    fn InterpretTransferPacketError(pkt: PTRANSFER_PACKET) -> BOOLEAN;
    fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN;
    fn IoSetMasterIrpStatus(Irp: PIRP, Status: NTSTATUS);
    fn MmPrepareMdlForReuse(Mdl: PMDL);
    fn PORT_ALLOCATED_SENSE_EX(fdoExt: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PSCSI_REQUEST_BLOCK) -> BOOLEAN;
    fn RetryTransferPacket(pkt: PTRANSFER_PACKET) -> BOOLEAN;
    fn ServiceTransferRequest(Fdo: PDEVICE_OBJECT, Irp: PIRP, Retry: BOOLEAN);
    fn StepLowMemRetry(pkt: PTRANSFER_PACKET) -> BOOLEAN;
    fn WriteULong64NoFence(Destination: *mut u64, Value: u64);
    fn SrbGetSenseInfoBuffer(Srb: PSCSI_REQUEST_BLOCK) -> *mut UCHAR;
    fn SrbGetSenseInfoBufferLength(Srb: PSCSI_REQUEST_BLOCK) -> ULONG;
    fn SrbGetSrbFlags(Srb: PSCSI_REQUEST_BLOCK) -> ULONG;
    fn TEST_FLAG(Flags: ULONG, Mask: ULONG) -> BOOLEAN;
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
}

// Constants (best-effort values)
const NonPagedPoolNx: u32 = 0x200;
const TRACE_LEVEL_ERROR: u32 = 0x01;
const TRACE_LEVEL_WARNING: u32 = 0x02;
const TRACE_FLAG_RW: u32 = 0x01;
const STATUS_INTEGER_OVERFLOW: i32 = 0xC0000095u32 as i32;
const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xC000009Au32 as i32;
const STATUS_SUCCESS: i32 = 0x00000000;
const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 1;
const SrbExDataTypeScsiCdb16: u32 = 1;
const SrbExDataTypeIoInfo: u32 = 2;

// Assume the following struct definitions exist (provided by caller)
// typedef struct _TRANSFER_PACKET { ... } TRANSFER_PACKET, *PTRANSFER_PACKET;
// typedef struct _DEVICE_OBJECT { ... } DEVICE_OBJECT, *PDEVICE_OBJECT;
// typedef struct _FUNCTIONAL_DEVICE_EXTENSION { ... } FUNCTIONAL_DEVICE_EXTENSION, *PFUNCTIONAL_DEVICE_EXTENSION;
// typedef struct _CLASS_PRIVATE_FDO_DATA { ... } CLASS_PRIVATE_FDO_DATA, *PCLASS_PRIVATE_FDO_DATA;
// typedef struct _STORAGE_ADAPTER_DESCRIPTOR { ... } STORAGE_ADAPTER_DESCRIPTOR, *PSTORAGE_ADAPTER_DESCRIPTOR;
// typedef struct _STORAGE_MINIPORT_DESCRIPTOR { ... } STORAGE_MINIPORT_DESCRIPTOR, *PSTORAGE_MINIPORT_DESCRIPTOR;
// typedef struct _SCSI_REQUEST_BLOCK { ... } SCSI_REQUEST_BLOCK, *PSCSI_REQUEST_BLOCK;
// typedef struct _STORAGE_REQUEST_BLOCK { ... } STORAGE_REQUEST_BLOCK, *PSTORAGE_REQUEST_BLOCK;
// typedef struct _SRB_HISTORY { ... } SRB_HISTORY, *PSRB_HISTORY;
// typedef struct _SRB_HISTORY_ITEM { ... } SRB_HISTORY_ITEM;

const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0x3; // typical value, verify
const SCSIOP_READ16: u8 = 0x88;
const SCSIOP_WRITE16: u8 = 0x8A;
const SCSIOP_READ: u8 = 0x28;
const SCSIOP_WRITE: u8 = 0x2A;
const IRP_PAGING_IO: u32 = 0x20;
const IRP_SYNCHRONOUS_PAGING_IO: u32 = 0x40;
const SRB_CLASS_FLAGS_PAGING: u32 = 0x10000;
const SRB_FLAGS_DATA_IN: u32 = 0x400;
const SRB_FLAGS_DATA_OUT: u32 = 0x800;
const SRB_FLAGS_ADAPTER_CACHE_ENABLE: u32 = 0x1000;
const SL_WRITE_THROUGH: u32 = 0x40;
const DEV_USE_16BYTE_CDB: u32 = 0x200;
const IRP_MJ_READ: u8 = 0x03;
const IRP_MJ_WRITE: u8 = 0x04;
const TRUE: u8 = 1;
const FALSE: u8 = 0;

const IRP_MJ_SCSI: u8 = 15;
const SENSE_BUFFER_SIZE_EX: u8 = 18;

const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0; // TODO: correct value
const SRB_SIMPLE_TAG_REQUEST: u32 = 0;          // TODO: correct value
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0; // TODO: correct value
const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0;       // TODO: correct value
const SCSIOP_MEDIUM_REMOVAL: u8 = 0;            // TODO: correct value
const NUM_LOCKMEDIAREMOVAL_RETRIES: u32 = 0;    // TODO: correct value

const SRB_SIMPLE_TAG_REQUEST: u32 = 0x01;
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x0100;
const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x0400;
const SCSIOP_MODE_SENSE: u8 = 0x1A;
const NUM_MODESENSE_RETRIES: u32 = 3;

#[allow(improper_ctypes)]

const REMOVE_COMPLETE: ULONG = 1;

const NonPagedPoolNxCacheAligned: u32 = 0; // placeholder
const CLASS_TAG_PRIVATE_DATA: u32 = 0; // placeholder
const StorageDeviceIoCapabilityProperty: STORAGE_PROPERTY_ID = 1; // placeholder
const VER_NT_DOMAIN_CONTROLLER: u16 = 2; // placeholder
const VER_NT_SERVER: u16 = 3; // placeholder
const MIN_WORKINGSET_TRANSFER_PACKETS_Client: u32 = 4; // placeholder
const MAX_WORKINGSET_TRANSFER_PACKETS_Client: u32 = 32; // placeholder
const MIN_WORKINGSET_TRANSFER_PACKETS_Server_LowerBound: u32 = 8; // placeholder
const MIN_WORKINGSET_TRANSFER_PACKETS_Server_UpperBound: u32 = 64; // placeholder
const MAX_WORKINGSET_TRANSFER_PACKETS_Server: u32 = 256; // placeholder
const MIN_INITIAL_TRANSFER_PACKETS: u32 = 2; // placeholder
const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0x30; // placeholder
const SrbExDataTypeScsiCdb16: u32 = 0x10; // placeholder
const SrbExDataTypeIoInfo: u32 = 0x11; // placeholder
const SRB_FUNCTION_EXECUTE_SCSI: u32 = 0x00; // placeholder
const SENSE_BUFFER_SIZE_EX: u32 = 18; // placeholder
const SRB_SIMPLE_TAG_REQUEST: u32 = 0x10; // placeholder
const NonPagedPoolNx: u32 = 0; // placeholder
const NTDDI_VERSION: u32 = 0x06030000; // placeholder
const NTDDI_WINBLUE: u32 = 0x06030000; // placeholder
const IS_NTDDI_WINBLUE: bool = NTDDI_VERSION >= NTDDI_WINBLUE;
const MAX_OUTSTANDING_IO_PER_LUN_DEFAULT: u32 = 1; // placeholder
const RTL_SIZEOF_THROUGH_FIELD_STORAGE_MINIPORT_DESCRIPTOR_ExtraIoInfoSupported: u32 = 0; // placeholder

// Forward declarations for unknown functions

// Constants (placeholders, exact values to be provided by caller)
const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0x00000103;
const SRB_STATUS_SUCCESS: UCHAR = 0x01;
const SRB_STATUS_QUEUE_FROZEN: UCHAR = 0x40;
const SRB_FLAGS_DONT_START_NEXT_PACKET: ULONG = 0x02; // guessing
const IRP_PAGING_IO: ULONG = 0x08;
const IoPagingPriorityHigh: u32 = 2;
const IoPagingPriorityInvalid: u32 = 0;
const ClassMaxInterleavePerCriticalIo: u32 = 8; // guessing
#[no_mangle]
pub unsafe extern "C" fn InitializeTransferPackets(Fdo: *mut DEVICE_OBJECT) -> i32 {
    PAGED_CODE();

    let commonExt = (*Fdo).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let fdoExt = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let adapterDesc = (*(*commonExt).PartitionZeroExtension).AdapterDescriptor;

    let mut devIoCapabilityDesc: *mut STORAGE_DEVICE_IO_CAPABILITY_DESCRIPTOR = core::ptr::null_mut();
    let mut propertyId: STORAGE_PROPERTY_ID = core::mem::zeroed();
    let mut osVersionInfo: OSVERSIONINFOEXW = core::mem::zeroed();
    let hwMaxPages: u32;
    let arraySize: u32;
    let mut index: u32;
    let mut maxOutstandingIOPerLUN: u32 = 0;
    let mut minWorkingSetTransferPackets: u32;
    let mut maxWorkingSetTransferPackets: u32;
    let mut status: i32 = STATUS_SUCCESS;

    NT_ASSERT((*adapterDesc).MaximumTransferLength != 0 as i32)

    hwMaxPages = if (*adapterDesc).MaximumPhysicalPages != 0 { (*adapterDesc).MaximumPhysicalPages - 1 } else { 0 };

    (*fdoData).HwMaxXferLen = core::cmp::min((*adapterDesc).MaximumTransferLength, hwMaxPages << PAGE_SHIFT);
    (*fdoData).HwMaxXferLen = core::cmp::max((*fdoData).HwMaxXferLen, PAGE_SIZE);

    arraySize = KeQueryHighestNodeNumber() + 1;

    (*fdoData).FreeTransferPacketsLists =
        ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
                              core::mem::size_of::<PNL_SLIST_HEADER>() * arraySize as usize,
                              CLASS_TAG_PRIVATE_DATA) as *mut PNL_SLIST_HEADER;

    if (*fdoData).FreeTransferPacketsLists.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        return status;
    }

    for index in 0..arraySize {
        let list_ptr = &raw mut (*(*fdoData).FreeTransferPacketsLists.add(index as usize)).SListHeader;
        InitializeSListHead(list_ptr);
        (*(*fdoData).FreeTransferPacketsLists.add(index as usize)).NumTotalTransferPackets = 0;
        (*(*fdoData).FreeTransferPacketsLists.add(index as usize)).NumFreeTransferPackets = 0;
    }

    InitializeListHead(&raw mut (*fdoData).AllTransferPacketsList as *mut LIST_ENTRY);

    osVersionInfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as u32;
    status = RtlGetVersion(&raw mut osVersionInfo as *mut OSVERSIONINFOW);

    NT_ASSERT(NT_SUCCESS(status) != 0 as i32)

    propertyId = StorageDeviceIoCapabilityProperty;
    status = ClassGetDescriptor((*(*fdoExt).CommonExtension).LowerDeviceObject,
                                &raw const propertyId,
                                &raw mut devIoCapabilityDesc as *mut *mut core::ffi::c_void);

    if NT_SUCCESS(status) != 0 && !devIoCapabilityDesc.is_null() {
        maxOutstandingIOPerLUN = (*devIoCapabilityDesc).LunMaxIoCount;
        FREE_POOL(devIoCapabilityDesc as *mut core::ffi::c_void);

        #[cfg(debug_assertions)]
        {
            (*fdoData).MaxOutstandingIOPerLUN = maxOutstandingIOPerLUN;
        }
    } else {
        maxOutstandingIOPerLUN = MAX_OUTSTANDING_IO_PER_LUN_DEFAULT;

        #[cfg(debug_assertions)]
        {
            (*fdoData).MaxOutstandingIOPerLUN = 0;
        }
    }

    status = STATUS_SUCCESS;

    if (u16::from(osVersionInfo.wProductType) != VER_NT_DOMAIN_CONTROLLER) &&
        (u16::from(osVersionInfo.wProductType) != VER_NT_SERVER) {
        // Client SKU
        minWorkingSetTransferPackets = MIN_WORKINGSET_TRANSFER_PACKETS_Client;
        maxWorkingSetTransferPackets = core::cmp::max(MAX_WORKINGSET_TRANSFER_PACKETS_Client,
                                                        2 * maxOutstandingIOPerLUN);
    } else {
        // Server SKU
        minWorkingSetTransferPackets =
            core::cmp::max(MIN_WORKINGSET_TRANSFER_PACKETS_Server_LowerBound,
                            core::cmp::min(MIN_WORKINGSET_TRANSFER_PACKETS_Server_UpperBound,
                                           maxOutstandingIOPerLUN));
        maxWorkingSetTransferPackets = core::cmp::max(MAX_WORKINGSET_TRANSFER_PACKETS_Server,
                                                        2 * maxOutstandingIOPerLUN);
    }

    (*fdoData).LocalMinWorkingSetTransferPackets = minWorkingSetTransferPackets;
    (*fdoData).LocalMaxWorkingSetTransferPackets = maxWorkingSetTransferPackets;

    if !(*(*commonExt).DriverExtension).WorkingSet.is_null() {
        let workingSet = (*(*commonExt).DriverExtension).WorkingSet;
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

    for index in 0..arraySize {
        while (*(*fdoData).FreeTransferPacketsLists.add(index as usize)).NumFreeTransferPackets < MIN_INITIAL_TRANSFER_PACKETS {
            let pkt = NewTransferPacket(Fdo);
            if !pkt.is_null() {
                InterlockedIncrement(&raw mut (*(*fdoData).FreeTransferPacketsLists.add(index as usize)).NumTotalTransferPackets as *mut i32);
                (*pkt).AllocateNode = index;
                EnqueueFreeTransferPacket(Fdo, pkt);
            } else {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }
        }
        (*(*fdoData).FreeTransferPacketsLists.add(index as usize)).DbgPeakNumTransferPackets =
            (*(*fdoData).FreeTransferPacketsLists.add(index as usize)).NumTotalTransferPackets;
    }

    if NT_SUCCESS(status) != 0 {
        if (*(*fdoExt).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
            let mut ByteSize: u32 = 0;

            if IS_NTDDI_WINBLUE {
                if !(*fdoExt).MiniportDescriptor.is_null() &&
                    (*(*fdoExt).MiniportDescriptor).Size >= RTL_SIZEOF_THROUGH_FIELD_STORAGE_MINIPORT_DESCRIPTOR_ExtraIoInfoSupported &&
                    (*(*fdoExt).MiniportDescriptor).ExtraIoInfoSupported == 1 {
                    status = CreateStorageRequestBlock(
                        &raw mut (*fdoData).SrbTemplate as *mut *mut STORAGE_REQUEST_BLOCK,
                        (*(*fdoExt).AdapterDescriptor).AddressType,
                        DefaultStorageRequestBlockAllocateRoutine,
                        &raw mut ByteSize,
                        2,
                        SrbExDataTypeScsiCdb16,
                        SrbExDataTypeIoInfo
                    );
                } else {
                    status = CreateStorageRequestBlock(
                        &raw mut (*fdoData).SrbTemplate as *mut *mut STORAGE_REQUEST_BLOCK,
                        (*(*fdoExt).AdapterDescriptor).AddressType,
                        DefaultStorageRequestBlockAllocateRoutine,
                        &raw mut ByteSize,
                        1,
                        SrbExDataTypeScsiCdb16
                    );
                }
            } else {
                status = CreateStorageRequestBlock(
                    &raw mut (*fdoData).SrbTemplate as *mut *mut STORAGE_REQUEST_BLOCK,
                    (*(*fdoExt).AdapterDescriptor).AddressType,
                    DefaultStorageRequestBlockAllocateRoutine,
                    &raw mut ByteSize,
                    1,
                    SrbExDataTypeScsiCdb16
                );
            }

            if NT_SUCCESS(status) != 0 {
                (*(*fdoData).SrbTemplate as *mut STORAGE_REQUEST_BLOCK).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
            } else {
                NT_ASSERT(0 as i32)
            }
        } else {
            (*fdoData).SrbTemplate = ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                u32::from_le_bytes(*b"-brs")
            ) as *mut core::ffi::c_void;
            if (*fdoData).SrbTemplate.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                RtlZeroMemory((*fdoData).SrbTemplate, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                (*(*fdoData).SrbTemplate as *mut SCSI_REQUEST_BLOCK).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32;
                (*(*fdoData).SrbTemplate as *mut SCSI_REQUEST_BLOCK).Function = SRB_FUNCTION_EXECUTE_SCSI;
            }
        }
    }

    if status == STATUS_SUCCESS {
        SrbSetRequestAttribute((*fdoData).SrbTemplate, SRB_SIMPLE_TAG_REQUEST);
        SrbSetSenseInfoBufferLength((*fdoData).SrbTemplate, SENSE_BUFFER_SIZE_EX);
        SrbSetCdbLength((*fdoData).SrbTemplate, 10);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DestroyAllTransferPackets(Fdo: PDEVICE_OBJECT) {
    let fdoExt = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let mut pkt;
    let mut index;
    let mut arraySize;

    PAGED_CODE();

    if !(*fdoData).FreeTransferPacketsLists.is_null() {
        NT_ASSERT(IsListEmpty(core::ptr::addr_of_mut!((*fdoData).DeferredClientIrpList)) as i32);

        arraySize = KeQueryHighestNodeNumber() + 1;
        index = 0;
        while index < arraySize {
            pkt = DequeueFreeTransferPacketEx(Fdo, 0, index);
            while !pkt.is_null() {
                DestroyTransferPacket(pkt);
                InterlockedDecrement(core::ptr::addr_of_mut!((*fdoData).FreeTransferPacketsLists[index].NumTotalTransferPackets) as *mut i32);
                pkt = DequeueFreeTransferPacketEx(Fdo, 0, index);
            }

            NT_ASSERT(if (*fdoData).FreeTransferPacketsLists[index].NumTotalTransferPackets == 0  { 1 } else { 0 } as i32)

            index += 1;
        }
    }

    FREE_POOL((*fdoData).SrbTemplate as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn NewTransferPacket(Fdo: *mut DEVICE_OBJECT) -> *mut TRANSFER_PACKET {
    let fdoExt = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
    let mut newPkt: *mut TRANSFER_PACKET = core::ptr::null_mut();
    let mut transferLength: u32 = u32::MAX;
    let mut status: i32 = STATUS_SUCCESS;

    if NT_SUCCESS(status) {
        status = RtlULongAdd((*fdoData).HwMaxXferLen, PAGE_SIZE as u32, &mut transferLength as *mut u32);
        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_RW,
                b"Integer overflow in calculating transfer packet size.\0".as_ptr() as *const i8,
            );
            status = STATUS_INTEGER_OVERFLOW;
        }
    }

    if NT_SUCCESS(status) {
        newPkt = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<TRANSFER_PACKET>(),
            u32::from_ne_bytes(*b"pnPC"),
        ) as *mut TRANSFER_PACKET;
        if newPkt.is_null() {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"Failed to allocate transfer packet.\0".as_ptr() as *const i8,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            RtlZeroMemory(newPkt as *mut c_void, core::mem::size_of::<TRANSFER_PACKET>());
            (*newPkt).AllocateNode = KeGetCurrentNodeNumber();

            if (*(*fdoExt).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                // NTDDI_VERSION >= NTDDI_WINBLUE assumed for this block
                if ((*fdoExt).MiniportDescriptor != core::ptr::null_mut())
                    && ((*(*fdoExt).MiniportDescriptor).Size
                        >= (core::mem::offset_of!(STORAGE_MINIPORT_DESCRIPTOR, ExtraIoInfoSupported)
                            + core::mem::size_of::<u8>()))
                    && ((*(*fdoExt).MiniportDescriptor).ExtraIoInfoSupported == 1)
                {
                    status = CreateStorageRequestBlock(
                        core::ptr::addr_of_mut!((*newPkt).Srb) as *mut *mut STORAGE_REQUEST_BLOCK,
                        (*(*fdoExt).AdapterDescriptor).AddressType,
                        Some(DefaultStorageRequestBlockAllocateRoutine),
                        core::ptr::null_mut(),
                        2u32,
                        SrbExDataTypeScsiCdb16,
                        SrbExDataTypeIoInfo,
                    );
                } else {
                    status = CreateStorageRequestBlock(
                        core::ptr::addr_of_mut!((*newPkt).Srb) as *mut *mut STORAGE_REQUEST_BLOCK,
                        (*(*fdoExt).AdapterDescriptor).AddressType,
                        Some(DefaultStorageRequestBlockAllocateRoutine),
                        core::ptr::null_mut(),
                        1u32,
                        SrbExDataTypeScsiCdb16,
                    );
                }
            } else {
                (*newPkt).Srb = ExAllocatePoolWithTag(
                    NonPagedPoolNx,
                    core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                    u32::from_ne_bytes(*b"-brs"),
                ) as *mut c_void;
                if ((*newPkt).Srb).is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                }
            }

            if status != STATUS_SUCCESS {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_RW,
                    b"Failed to allocate SRB.\0".as_ptr() as *const i8,
                );
                FREE_POOL(newPkt as *mut c_void);
                newPkt = core::ptr::null_mut();
            }
        }
    }

    if NT_SUCCESS(status) && !newPkt.is_null() {
        (*newPkt).Irp = IoAllocateIrp((*Fdo).StackSize, 0);
        if ((*newPkt).Irp).is_null() {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"Failed to allocate IRP for transfer packet.\0".as_ptr() as *const i8,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if NT_SUCCESS(status) && !newPkt.is_null() {
        NT_ASSERT((transferLength != u32::MAX) as i32);
        (*newPkt).PartialMdl = IoAllocateMdl(core::ptr::null_mut(), transferLength, 0, 0, core::ptr::null_mut());
        if ((*newPkt).PartialMdl).is_null() {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"Failed to allocate MDL for transfer packet.\0".as_ptr() as *const i8,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            NT_ASSERT(
                ((*(*newPkt).PartialMdl).Size
                    >= (core::mem::size_of::<MDL>()
                        + BYTES_TO_PAGES((*fdoData).HwMaxXferLen) as u16
                            * core::mem::size_of::<PFN_NUMBER>() as u16) as i16)
                    .into(),
            );
        }
    }

    if NT_SUCCESS(status)
        && ((*fdoData).InterpretSenseInfo != core::ptr::null_mut())
        && !newPkt.is_null()
    {
        let mut historyByteCount: u32 = 0;

        NT_ASSERT((*(*fdoData).InterpretSenseInfo).HistoryCount != 0 as i32)
        // _Analysis_assume_((*fdoData).InterpretSenseInfo->HistoryCount != 0);

        historyByteCount = core::mem::size_of::<SRB_HISTORY_ITEM>() as u32
            * (*(*fdoData).InterpretSenseInfo).HistoryCount;
        historyByteCount += core::mem::size_of::<SRB_HISTORY>() as u32 - core::mem::size_of::<SRB_HISTORY_ITEM>() as u32;

        (*newPkt).RetryHistory = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            historyByteCount as usize,
            u32::from_ne_bytes(*b"hrPC"),
        ) as *mut SRB_HISTORY;

        if ((*newPkt).RetryHistory).is_null() {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"Failed to allocate MDL for transfer packet.\0".as_ptr() as *const i8,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            HistoryInitializeRetryLogs(
                (*newPkt).RetryHistory,
                (*(*fdoData).InterpretSenseInfo).HistoryCount,
            );
        }
    }

    if NT_SUCCESS(status) && !newPkt.is_null() {
        let mut oldIrql: KIRQL = 0;
        (*newPkt).Fdo = Fdo;
        if cfg!(debug_assertions) {
            (*newPkt).DbgPktId = InterlockedIncrement(
                core::ptr::addr_of_mut!((*fdoData).DbgMaxPktId) as *mut i32,
            );
        }
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*fdoData).SpinLock),
            &mut oldIrql as *mut KIRQL,
        );
        InsertTailList(
            core::ptr::addr_of_mut!((*fdoData).AllTransferPacketsList),
            core::ptr::addr_of_mut!((*newPkt).AllPktsListEntry),
        );
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock), oldIrql);
    } else {
        if !newPkt.is_null() {
            FREE_POOL((*newPkt).RetryHistory as *mut c_void);
            (*newPkt).RetryHistory = core::ptr::null_mut();
            if !((*newPkt).PartialMdl).is_null() {
                IoFreeMdl((*newPkt).PartialMdl);
            }
            if !((*newPkt).Irp).is_null() {
                IoFreeIrp((*newPkt).Irp);
            }
            if !((*newPkt).Srb).is_null() {
                FREE_POOL((*newPkt).Srb as *mut c_void);
            }
            FREE_POOL(newPkt as *mut c_void);
            newPkt = core::ptr::null_mut();
        }
    }

    newPkt
}
#[no_mangle]
pub unsafe extern "C" fn DestroyTransferPacket(Pkt: *mut TRANSFER_PACKET) {
    let fdoExt = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let mut oldIrql: KIRQL;

    NT_ASSERT((*Pkt).SlistEntry.Next.is_null() as i32);
    //    NT_ASSERT(!Pkt->OriginalIrp as i32)

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock), core::ptr::addr_of_mut!(oldIrql));

    /*
     *  Delete the packet from our all-packets queue.
     */
    NT_ASSERT(!IsListEmpty(core::ptr::addr_of!((*Pkt).AllPktsListEntry)) as i32)
    NT_ASSERT(!IsListEmpty(core::ptr::addr_of!((*fdoData).AllTransferPacketsList)) as i32)
    RemoveEntryList(core::ptr::addr_of_mut!((*Pkt).AllPktsListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*Pkt).AllPktsListEntry));

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock), oldIrql);

    IoFreeMdl((*Pkt).PartialMdl);
    IoFreeIrp((*Pkt).Irp);
    FREE_POOL((*Pkt).RetryHistory as *mut core::ffi::c_void);
    FREE_POOL((*Pkt).Srb as *mut core::ffi::c_void);
    FREE_POOL(Pkt as *mut core::ffi::c_void);
}
#[no_mangle] 
 pub unsafe extern "C" fn EnqueueFreeTransferPacket(Fdo: PDEVICE_OBJECT, Pkt: PTRANSFER_PACKET) {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;
    let mut workItem: PIO_WORKITEM;
    let mut isRemoved: ULONG;
    let mut pktToDelete: PTRANSFER_PACKET;

    NT_ASSERT((*Pkt).SlistEntry.Next.is_null() as i32);

    let allocateNode: ULONG = (*Pkt).AllocateNode;

    InterlockedPushEntrySList(
        core::ptr::addr_of_mut!((*fdoData).FreeTransferPacketsLists[allocateNode as usize].SListHeader),
        core::ptr::addr_of_mut!((*Pkt).SlistEntry)
    );
    InterlockedIncrement(
        core::ptr::addr_of_mut!((*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumFreeTransferPackets) as *mut i32
    );

    if (*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumFreeTransferPackets >=
        (*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets {
        
        if (*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets >
            (*fdoData).LocalMaxWorkingSetTransferPackets {

            workItem = IoAllocateWorkItem(Fdo);
            isRemoved = ClassAcquireRemoveLock(Fdo, workItem as PIRP);

            if !workItem.is_null() && isRemoved == 0 {
                let fmt = b"EnqueueFreeTransferPacket: Device (%p), queuing work item to clean up free transfer packets.\n\0"
                    .as_ptr() as *const i8;
                TracePrint(4 /* TRACE_LEVEL_INFORMATION */, 0x1 /* TRACE_FLAG_GENERAL */, fmt, Fdo);

                IoQueueWorkItemEx(
                    workItem,
                    CleanupTransferPacketToWorkingSetSizeWorker,
                    DelayedWorkQueue,
                    (allocateNode as usize) as *mut core::ffi::c_void
                );
            } else {
                if !workItem.is_null() {
                    IoFreeWorkItem(workItem);
                }
                if isRemoved != REMOVE_COMPLETE {
                    ClassReleaseRemoveLock(Fdo, workItem as PIRP);
                }
                let fmt_err = b"EnqueueFreeTransferPacket: Device (%p), Failed to allocate memory for the work item.\n\0"
                    .as_ptr() as *const i8;
                TracePrint(1 /* TRACE_LEVEL_ERROR */, 0x1 /* TRACE_FLAG_GENERAL */, fmt_err, Fdo);

                CleanupTransferPacketToWorkingSetSize(Fdo, 1 /* TRUE */, allocateNode);
            }
        }

        if (*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets >
            (*fdoData).LocalMinWorkingSetTransferPackets {

            let fmt_exit = b"Exiting stress, lazily freeing one of %d/%d packets from node %d.\n\0"
                .as_ptr() as *const i8;
            TracePrint(4 /* TRACE_LEVEL_INFORMATION */, 0x4 /* TRACE_FLAG_RW */,
                fmt_exit,
                (*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets,
                (*fdoData).LocalMinWorkingSetTransferPackets,
                allocateNode);

            pktToDelete = core::ptr::null_mut();
            KeAcquireSpinLock(
                core::ptr::addr_of_mut!((*fdoData).SpinLock),
                core::ptr::addr_of_mut!(oldIrql)
            );

            if ((*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumFreeTransferPackets >=
                (*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets) &&
                ((*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets >
                (*fdoData).LocalMinWorkingSetTransferPackets) {

                pktToDelete = DequeueFreeTransferPacketEx(Fdo, 0 /* FALSE */, allocateNode);
                if !pktToDelete.is_null() {
                    InterlockedDecrement(
                        core::ptr::addr_of_mut!((*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets) as *mut i32
                    );
                } else {
                    let fmt_unlikely = b"Extremely unlikely condition (non-fatal): %d packets dequeued at once for Fdo %p. NumTotalTransferPackets=%d (2). Node=%d\n\0"
                        .as_ptr() as *const i8;
                    TracePrint(4 /* TRACE_LEVEL_INFORMATION */, 0x4 /* TRACE_FLAG_RW */,
                        fmt_unlikely,
                        (*fdoData).LocalMinWorkingSetTransferPackets,
                        Fdo,
                        (*fdoData).FreeTransferPacketsLists[allocateNode as usize].NumTotalTransferPackets,
                        allocateNode);
                }
            }
            KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock), oldIrql);

            if !pktToDelete.is_null() {
                DestroyTransferPacket(pktToDelete);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DequeueFreeTransferPacket(Fdo: *mut DEVICE_OBJECT, AllocIfNeeded: u8) -> *mut TRANSFER_PACKET {
    DequeueFreeTransferPacketEx(Fdo, AllocIfNeeded, KeGetCurrentNodeNumber())
}
#[no_mangle]
pub unsafe extern "C" fn DequeueFreeTransferPacketEx(
    Fdo: *mut DEVICE_OBJECT,
    AllocIfNeeded: u8,
    Node: u32,
) -> *mut TRANSFER_PACKET {
    let fdo_ext_ptr = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdo_data = (*fdo_ext_ptr).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;

    let list_slot = (*fdo_data).FreeTransferPacketsLists.add(Node as usize);
    let slist_entry = InterlockedPopEntrySList(&raw mut (*list_slot).SListHeader as *mut _);

    let pkt: *mut TRANSFER_PACKET;

    if !slist_entry.is_null() {
        // SlistEntry is now taken out of the list, clear its Next pointer.
        (*slist_entry).Next = core::ptr::null_mut();

        // CONTAINING_RECORD
        let offset = core::mem::offset_of!(TRANSFER_PACKET, SlistEntry);
        pkt = ((slist_entry as usize).wrapping_sub(offset)) as *mut TRANSFER_PACKET;

        InterlockedDecrement(&raw mut (*list_slot).NumFreeTransferPackets as *mut i32);

        // Reset history data on dequeue.
        HISTORYINITIALIZERETRYLOGS(pkt);
    } else {
        if AllocIfNeeded != 0 {
            // We ran out of lookaside packets; allocate an extra one.
            pkt = NewTransferPacket(Fdo);
            if !pkt.is_null() {
                InterlockedIncrement(&raw mut (*list_slot).NumTotalTransferPackets as *mut i32);
                let current_peak = (*list_slot).DbgPeakNumTransferPackets;
                let total = (*list_slot).NumTotalTransferPackets;
                (*list_slot).DbgPeakNumTransferPackets = max(current_peak, total);
            } else {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_RW,
                    "DequeueFreeTransferPacket: packet allocation failed\0".as_ptr() as *const i8,
                );
            }
        } else {
            pkt = core::ptr::null_mut();
        }
    }

    pkt
}
#[no_mangle]
pub unsafe extern "C" fn SetupReadWriteTransferPacket(
    Pkt: *mut TRANSFER_PACKET,
    Buf: *mut core::ffi::c_void,
    Len: u32,
    DiskLocation: LARGE_INTEGER,
    OriginalIrp: *mut IRP,
) {
    let fdoExt: *mut FUNCTIONAL_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension: *mut COMMON_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let origCurSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(OriginalIrp);
    let majorFunc: u8 = (*origCurSp).MajorFunction;
    let mut logicalBlockAddr: LARGE_INTEGER = core::mem::zeroed();
    let numTransferBlocks: u32;
    let pCdb: *mut CDB;
    let srbLength: u32;
    let timeoutValue: u32 = (*fdoExt).TimeOutValue;

    logicalBlockAddr.QuadPart = Int64ShrlMod32(DiskLocation.QuadPart, (*fdoExt).SectorShift.into());
    numTransferBlocks = Len >> (*fdoExt).SectorShift;

    (*(*Pkt).DriverUsesStartIO) = if (*(*commonExtension).DriverExtension).InitData.ClassStartIo.is_null() { 0 } else { 1 };

    if *(*((*fdoExt).AdapterDescriptor)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK  {
        srbLength = (*((*fdoData).SrbTemplate as *mut STORAGE_REQUEST_BLOCK)).SrbLength;
        NT_ASSERT((((*((*Pkt).Srb as *mut STORAGE_REQUEST_BLOCK)).SrbLength >= srbLength) as u8).into());
    } else {
        srbLength = (*(*fdoData).SrbTemplate).Length;
    }
    core::ptr::copy_nonoverlapping((*fdoData).SrbTemplate as *const u8, (*Pkt).Srb as *mut u8, srbLength as usize);
    SrbSetDataBuffer((*Pkt).Srb, Buf);
    SrbSetDataTransferLength((*Pkt).Srb, Len);
    SrbSetQueueSortKey((*Pkt).Srb, logicalBlockAddr.__bindgen_anon_1.LowPart);
    if logicalBlockAddr.QuadPart > 0xFFFFFFFFi64 {
        SrbSetQueueSortKey((*Pkt).Srb, 0xFFFFFFFF);
    }
    SrbSetOriginalRequest((*Pkt).Srb, (*Pkt).Irp);
    SrbSetSenseInfoBuffer((*Pkt).Srb, core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as *mut core::ffi::c_void);
    SrbSetTimeOutValue((*Pkt).Srb, timeoutValue);

    pCdb = SrbGetCdb((*Pkt).Srb);
    if !pCdb.is_null() {
        if ((*fdoExt).DeviceFlags & DEV_USE_16BYTE_CDB) != 0 {
            REVERSE_BYTES_QUAD(core::ptr::addr_of_mut!((*pCdb).CDB16.LogicalBlock) as *mut u64, core::ptr::addr_of!(logicalBlockAddr) as *const i64);
            REVERSE_BYTES(core::ptr::addr_of_mut!((*pCdb).CDB16.TransferLength) as *mut u32, core::ptr::addr_of!(numTransferBlocks) as *const u32);
            (*pCdb).CDB16.OperationCode = if majorFunc == IRP_MJ_READ { SCSIOP_READ16 } else { SCSIOP_WRITE16 };
            SrbSetCdbLength((*Pkt).Srb, 16);
        } else {
            let low_part_ptr = core::ptr::addr_of!(logicalBlockAddr.__bindgen_anon_1.LowPart) as *const u8;
            (*pCdb).CDB10.LogicalBlockByte0 = *low_part_ptr.add(3);
            (*pCdb).CDB10.LogicalBlockByte1 = *low_part_ptr.add(2);
            (*pCdb).CDB10.LogicalBlockByte2 = *low_part_ptr.add(1);
            (*pCdb).CDB10.LogicalBlockByte3 = *low_part_ptr.add(0);
            let num_blocks_ptr = core::ptr::addr_of!(numTransferBlocks) as *const u8;
            (*pCdb).CDB10.TransferBlocksMsb = *num_blocks_ptr.add(1);
            (*pCdb).CDB10.TransferBlocksLsb = *num_blocks_ptr.add(0);
            (*pCdb).CDB10.OperationCode = if majorFunc == IRP_MJ_READ { SCSIOP_READ } else { SCSIOP_WRITE };
        }
    }

    SrbAssignSrbFlags((*Pkt).Srb, (*fdoExt).SrbFlags);
    if ((*(*OriginalIrp).Flags & IRP_PAGING_IO) != 0) || ((*(*OriginalIrp).Flags & IRP_SYNCHRONOUS_PAGING_IO) != 0) {
        SrbSetSrbFlags((*Pkt).Srb, SRB_CLASS_FLAGS_PAGING);
    }
    SrbSetSrbFlags((*Pkt).Srb, if majorFunc == IRP_MJ_READ { SRB_FLAGS_DATA_IN } else { SRB_FLAGS_DATA_OUT });

    if (majorFunc == IRP_MJ_WRITE) && ((*(*origCurSp).Flags & SL_WRITE_THROUGH) != 0) && !pCdb.is_null() {
        (*pCdb).CDB10.ForceUnitAccess = (*fdoExt).CdbForceUnitAccess;
    } else {
        SrbSetSrbFlags((*Pkt).Srb, SRB_FLAGS_ADAPTER_CACHE_ENABLE);
    }

    (*Pkt).BufPtrCopy = Buf;
    (*Pkt).BufLenCopy = Len;
    (*Pkt).TargetLocationCopy = DiskLocation;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = (*fdoData).MaxNumberOfIoRetries;
    (*Pkt).SyncEventPtr = core::ptr::null_mut();
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 1;

    (*Pkt).NumIoTimeoutRetries = (*fdoData).MaxNumberOfIoRetries;
    (*Pkt).NumThinProvisioningRetries = 0;

    if !pCdb.is_null() {
        DBGLOGFLUSHINFO(fdoData, 1, (*pCdb).CDB10.ForceUnitAccess, 0);
    } else {
        DBGLOGFLUSHINFO(fdoData, 1, 0, 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SubmitTransferPacket(Pkt: *mut TRANSFER_PACKET) -> NTSTATUS {
    let commonExtension = (*(*Pkt).Fdo).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let nextDevObj = (*commonExtension).LowerDeviceObject;
    let fdoExtension = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let mut idleRequest: u8 = 0;
    let nextSp: *mut IO_STACK_LOCATION;

    let irp = (*Pkt).Irp;
    let cond = ((*irp).CurrentLocation as u32 == (*irp).StackCount as u32 + 1) as u8;
    NT_ASSERT(cond.into() as i32)

    IoReuseIrp(irp, STATUS_NOT_SUPPORTED);

    nextSp = IoGetNextIrpStackLocation(irp);
    (*nextSp).MajorFunction = IRP_MJ_SCSI;
    (*nextSp).Parameters.Scsi.Srb = (*Pkt).Srb as *mut SCSI_REQUEST_BLOCK;

    SrbSetScsiStatus((*Pkt).Srb as *mut SCSI_REQUEST_BLOCK, 0);
    (*(*Pkt).Srb).SrbStatus = 0;
    SrbSetSenseInfoBufferLength((*Pkt).Srb as *mut SCSI_REQUEST_BLOCK, SENSE_BUFFER_SIZE_EX);

    if (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0 {
        let origCurSp = IoGetCurrentIrpStackLocation((*Pkt).OriginalIrp);
        (*nextSp).Flags = (*origCurSp).Flags;
    }

    if (*Pkt).UsePartialMdl == 0 {
        (*irp).MdlAddress = (*(*Pkt).OriginalIrp).MdlAddress;
    } else {
        IoBuildPartialMdl(
            (*(*Pkt).OriginalIrp).MdlAddress,
            (*Pkt).PartialMdl,
            SrbGetDataBuffer((*Pkt).Srb as *mut c_void),
            SrbGetDataTransferLength((*Pkt).Srb as *mut c_void)
        );
        (*irp).MdlAddress = (*Pkt).PartialMdl;
    }

    DBGLOGSENDPACKET(Pkt as *mut c_void);
    HISTORYLOGSENDPACKET(Pkt as *mut c_void);

    ClasspSrbSetOriginalIrp((*Pkt).Srb as *mut c_void, (*Pkt).OriginalIrp as *mut c_void);

    if (*fdoData).IdlePrioritySupported != 0 {
        idleRequest = ClasspIsIdleRequest((*Pkt).OriginalIrp);
        if idleRequest != 0 {
            InterlockedIncrement(core::ptr::addr_of_mut!((*fdoData).ActiveIdleIoCount) as *mut i32);
        } else {
            InterlockedIncrement(core::ptr::addr_of_mut!((*fdoData).ActiveIoCount) as *mut i32);
        }
    }

    IoSetCompletionRoutine(irp, Some(TransferPktComplete), Pkt as *mut c_void, 1, 1, 1);
    return IoCallDriver(nextDevObj, irp);
}

#[no_mangle]
pub unsafe extern "system" fn TransferPktComplete(
    NullFdo: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let _ = NullFdo; // UNREFERENCED_PARAMETER

    let pkt = Context as *mut TRANSFER_PACKET;
    let fdoExt = (*pkt).Fdo as *mut DEVICE_OBJECT; // actually PDEVICE_OBJECT
    let fdoExt = (*fdoExt).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;

    let mut packetDone: BOOLEAN = 0;
    let mut idleRequest: BOOLEAN = 0;
    let mut transferLength: ULONG;
    let mut completionTime: LARGE_INTEGER;
    let mut lastIoCompletionTime: u64;

    // DBG spew
    DBGLOGRETURNPACKET(pkt);
    DBGCHECKRETURNEDPKT(pkt);
    HISTORYLOGRETURNEDPACKET(pkt);

    completionTime = ClasspGetCurrentTime();

    #[cfg(target_pointer_width = "64")]
    {
        // Windows 64-bit non-ReactOS path using fence-free reads/writes
        lastIoCompletionTime = ReadULong64NoFence(
            &(*fdoData).LastIoCompletionTime.QuadPart as *const i64 as *const u64,
        );
        WriteULong64NoFence(
            &mut (*fdoData).LastIoCompletionTime.QuadPart as *mut i64 as *mut u64,
            completionTime.QuadPart as u64, // LARGE_INTEGER.QuadPart is i64, but we cast to u64
        );
    }

    if (*fdoData).IdlePrioritySupported == 1 {
        idleRequest = ClasspIsIdleRequest((*pkt).OriginalIrp);
        if idleRequest != 0 {
            InterlockedDecrement(&(*fdoData).ActiveIdleIoCount as *mut i32);
            NT_ASSERT(((*fdoData).ActiveIdleIoCount >= 0) as i32);
        } else {
            (*fdoData).LastNonIdleIoTime = completionTime;
            InterlockedDecrement(&(*fdoData).ActiveIoCount as *mut i32);
            NT_ASSERT(((*fdoData).ActiveIoCount >= 0) as i32);
        }
    }

    if (*pkt).UsePartialMdl != 0 {
        MmPrepareMdlForReuse((*pkt).PartialMdl);
    }

    if SRB_STATUS((*pkt).Srb.SrbStatus) == SRB_STATUS_SUCCESS {
        NT_ASSERT(NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) as i32);
        transferLength = SrbGetDataTransferLength((*pkt).Srb);
        (*fdoData).LoggedTURFailureSinceLastIO = 0;
        NT_ASSERT(!PORT_ALLOCATED_SENSE_EX(fdoExt, (*pkt).Srb) as i32);
        InterlockedExchangeAdd(
            &mut (*pkt).OriginalIrp.IoStatus.Information as *mut ULONG as *mut LONG,
            transferLength as LONG,
        );

        if ((*pkt).InLowMemRetry != 0)
            || ((*pkt).DriverUsesStartIO != 0 && (*pkt).LowMemRetry_remainingBufLen > 0)
        {
            packetDone = StepLowMemRetry(pkt);
        } else {
            packetDone = 1;
        }
    } else {
        let mut shouldRetry: BOOLEAN;

        if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) != 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
        }

        if (*pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0 {
            IoMarkIrpPending((*pkt).OriginalIrp);
        }

        shouldRetry = InterpretTransferPacketError(pkt);

        if (*pkt).Srb.SrbStatus & SRB_STATUS_QUEUE_FROZEN != 0 {
            ClassReleaseQueue((*pkt).Fdo);
        }

        if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) != 0 {
            NT_ASSERT((shouldRetry == 0) as i32);
            InterlockedExchangeAdd(
                &mut (*pkt).OriginalIrp.IoStatus.Information as *mut ULONG as *mut LONG,
                SrbGetDataTransferLength((*pkt).Srb) as LONG,
            );

            if ((*pkt).InLowMemRetry != 0)
                || ((*pkt).DriverUsesStartIO != 0 && (*pkt).LowMemRetry_remainingBufLen > 0)
            {
                packetDone = StepLowMemRetry(pkt);
            } else {
                packetDone = 1;
            }
        } else {
            if shouldRetry != 0 && (*pkt).NumRetries > 0 {
                packetDone = RetryTransferPacket(pkt);
            } else if shouldRetry != 0 && !(*pkt).RetryHistory.is_null() {
                packetDone = RetryTransferPacket(pkt);
            } else {
                packetDone = 1;
            }
        }
    }

    if packetDone != 0 {
        let numPacketsRemaining: LONG;
        let deferredIrp: PIRP;
        let Fdo = (*pkt).Fdo;
        let mut uniqueAddr: UCHAR = 0;

        ClassAcquireRemoveLock(Fdo, &raw mut uniqueAddr as *mut UCHAR as *mut c_void);

        if PORT_ALLOCATED_SENSE_EX(fdoExt, (*pkt).Srb) != 0 {
            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, b"Freeing port-allocated sense buffer for pkt %ph.\0".as_ptr() as *const i8, pkt);
            FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExt, (*pkt).Srb);
            SrbSetSenseInfoBuffer((*pkt).Srb, &raw mut (*pkt).SrbErrorSenseData as *mut UCHAR);
            SrbSetSenseInfoBufferLength((*pkt).Srb, (core::mem::size_of_val(&(*pkt).SrbErrorSenseData) as ULONG).try_into().unwrap());
        } else {
            NT_ASSERT((SrbGetSenseInfoBuffer((*pkt).Srb) == &raw mut (*pkt).SrbErrorSenseData as *mut UCHAR) as i32);
            NT_ASSERT((SrbGetSenseInfoBufferLength((*pkt).Srb) <= core::mem::size_of_val(&(*pkt).SrbErrorSenseData) as ULONG) as i32);
        }

        core::ptr::write_bytes(
            &raw mut (*pkt).SrbErrorSenseData as *mut _ as *mut u8,
            0,
            core::mem::size_of_val(&(*pkt).SrbErrorSenseData),
        );

        IoSetMasterIrpStatus((*pkt).OriginalIrp, (*Irp).IoStatus.__bindgen_anon_1.Status);

        if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
            if IoIsErrorUserInduced((*Irp).IoStatus.__bindgen_anon_1.Status) != 0
                && (*pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0
                && !(*pkt).OriginalIrp.Tail.Overlay.Thread.is_null()
            {
                IoSetHardErrorOrVerifyDevice((*pkt).OriginalIrp, Fdo);
            }
        }

        numPacketsRemaining = InterlockedDecrement(
            &mut (*(*pkt).OriginalIrp).Tail.Overlay.DriverContext[0] as *mut PVOID as *mut LONG,
        );

        if numPacketsRemaining > 0 {
            // More pieces remain; do nothing
        } else {
            NT_ASSERT((numPacketsRemaining == 0) as i32);
            if (*pkt).CompleteOriginalIrpWhenLastPacketCompletes != 0 {
                let priority: IO_PAGING_PRIORITY = if TEST_FLAG(
                    (*(*pkt).OriginalIrp).Flags,
                    IRP_PAGING_IO,
                ) != 0
                {
                    IoGetPagingIoPriority((*pkt).OriginalIrp)
                } else {
                    IoPagingPriorityInvalid
                };
                let mut oldIrql: KIRQL;

                if NT_SUCCESS((*(*pkt).OriginalIrp).IoStatus.__bindgen_anon_1.Status) != 0 {
                    NT_ASSERT(
                        ((*(*pkt).OriginalIrp).IoStatus.Information as ULONG
                            == IoGetCurrentIrpStackLocation((*pkt).OriginalIrp)
                                .Parameters
                                .Read
                                .Length) as i32,
                    );
                    ClasspPerfIncrementSuccessfulIo(fdoExt);
                }

                ClassReleaseRemoveLock(Fdo, (*pkt).OriginalIrp);

                if (*pkt).Irp.PendingReturned != 0 {
                    IoMarkIrpPending((*pkt).OriginalIrp);
                }

                ClassCompleteRequest(Fdo, (*pkt).OriginalIrp, IO_DISK_INCREMENT.into());

                if priority == IoPagingPriorityHigh {
                    KeAcquireSpinLock(&mut (*fdoData).SpinLock, &mut oldIrql);

                    if (*fdoData).MaxInterleavedNormalIo < ClassMaxInterleavePerCriticalIo {
                        (*fdoData).MaxInterleavedNormalIo = 0;
                    } else {
                        (*fdoData).MaxInterleavedNormalIo -= ClassMaxInterleavePerCriticalIo;
                    }

                    (*fdoData).NumHighPriorityPagingIo -= 1;

                    if (*fdoData).NumHighPriorityPagingIo == 0 {
                        let mut period: LARGE_INTEGER;

                        KeQuerySystemTime(&mut (*fdoData).ThrottleStopTime);

                        period.QuadPart = (*fdoData).ThrottleStopTime.QuadPart
                            - (*fdoData).ThrottleStartTime.QuadPart;
                        (*fdoData).LongestThrottlePeriod.QuadPart = if period.QuadPart
                            > (*fdoData).LongestThrottlePeriod.QuadPart
                        {
                            period.QuadPart
                        } else {
                            (*fdoData).LongestThrottlePeriod.QuadPart
                        };
                    }

                    KeReleaseSpinLock(&mut (*fdoData).SpinLock, oldIrql);
                }

                if idleRequest != 0 {
                    ClasspCompleteIdleRequest(fdoExt);
                }

                if (*fdoExt).CommonExtension.DriverExtension.InitData.ClassStartIo as ULONG_PTR
                    != 0
                {
                    if TEST_FLAG(
                        SrbGetSrbFlags((*pkt).Srb),
                        SRB_FLAGS_DONT_START_NEXT_PACKET,
                    ) != 0
                    {
                        TracePrint(
                            TRACE_LEVEL_INFORMATION,
                            TRACE_FLAG_RW,
                            b"SRB_FLAGS_DONT_START_NEXT_PACKET should never be set here (?)\0"
                                .as_ptr() as *const i8,
                        );
                    } else {
                        KeRaiseIrql(DISPATCH_LEVEL, &mut oldIrql);
                        IoStartNextPacket(Fdo, 1);
                        KeLowerIrql(oldIrql);
                    }
                }
            }
        }

        if !(*pkt).SyncEventPtr.is_null() {
            KeSetEvent((*pkt).SyncEventPtr, 0, 0);
            (*pkt).SyncEventPtr = core::ptr::null_mut();
        }

        if !(*pkt).ContinuationRoutine.is_null() {
            let cont = (*pkt).ContinuationRoutine;
            cont((*pkt).ContinuationContext);
            (*pkt).ContinuationRoutine = core::ptr::null_mut();
        }

        // Free packet
        (*pkt).UsePartialMdl = 0;
        (*pkt).InLowMemRetry = 0;
        EnqueueFreeTransferPacket(Fdo, pkt);

        deferredIrp = DequeueDeferredClientIrp(Fdo);
        if !deferredIrp.is_null() {
            ServiceTransferRequest(Fdo, deferredIrp, 1);
        }

        ClassReleaseRemoveLock(Fdo, &raw mut uniqueAddr as *mut UCHAR as *mut c_void);
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}
#[no_mangle]
pub unsafe extern "C" fn SetupEjectionTransferPacket(
    Pkt: *mut TRANSFER_PACKET,
    PreventMediaRemoval: u8,
    SyncEventPtr: *mut KEVENT,
    OriginalIrp: *mut IRP,
) {
    let fdoExt = (*((*Pkt).Fdo)).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
    let pCdb: *mut CDB;
    let srbLength: u32;

    // PAGED_CODE();

    if (*(*fdoExt).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbLength = (*(fdoData.SrbTemplate as *mut STORAGE_REQUEST_BLOCK)).SrbLength;
        // NT_ASSERT((*( (*Pkt).Srb as *mut STORAGE_REQUEST_BLOCK)).SrbLength >= srbLength);
    } else {
        srbLength = (*fdoData.SrbTemplate).Length;
    }

    core::ptr::copy_nonoverlapping(
        fdoData.SrbTemplate as *const u8,
        (*Pkt).Srb as *mut u8,
        srbLength as usize,
    );

    SrbSetRequestAttribute(((*Pkt).Srb) as *mut core::ffi::c_void, SRB_SIMPLE_TAG_REQUEST);
    SrbSetCdbLength(((*Pkt).Srb) as *mut core::ffi::c_void, 6);
    SrbSetOriginalRequest(((*Pkt).Srb) as *mut core::ffi::c_void, (*Pkt).Irp);
    SrbSetSenseInfoBuffer(
        ((*Pkt).Srb) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as *mut core::ffi::c_void,
    );
    SrbSetSenseInfoBufferLength(
        ((*Pkt).Srb) as *mut core::ffi::c_void,
        (core::mem::size_of_val(&(*Pkt).SrbErrorSenseData) as u32).try_into().unwrap(),
    );
    SrbSetTimeOutValue(((*Pkt).Srb) as *mut core::ffi::c_void, (*fdoExt).TimeOutValue);

    SrbAssignSrbFlags(
        ((*Pkt).Srb) as *mut core::ffi::c_void,
        (*fdoExt).SrbFlags | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE,
    );

    pCdb = SrbGetCdb(((*Pkt).Srb) as *mut core::ffi::c_void);
    if !pCdb.is_null() {
        (*pCdb).MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
        (*pCdb).MEDIA_REMOVAL.Prevent = PreventMediaRemoval;
    }

    (*Pkt).BufPtrCopy = core::ptr::null_mut();
    (*Pkt).BufLenCopy = 0;

    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_LOCKMEDIAREMOVAL_RETRIES;
    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0;
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
    PAGED_CODE();

    let fdoExt: *mut FUNCTIONAL_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension as _;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let pCdb: *mut CDB;
    let srbLength: u32;

    if (*(*fdoExt).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        let srb_template_cast = (*fdoData).SrbTemplate as *mut STORAGE_REQUEST_BLOCK;
        srbLength = (*srb_template_cast).SrbLength;
        let srb_cast = (*Pkt).Srb as *mut STORAGE_REQUEST_BLOCK;
        NT_ASSERT((*srb_cast).SrbLength >= srbLength as i32)
    } else {
        srbLength = (*(*fdoData).SrbTemplate).Length;
    }

    RtlCopyMemory(
        (*Pkt).Srb as *mut _,
        (*fdoData).SrbTemplate as *const _,
        srbLength as usize,
    );

    SrbSetRequestAttribute((*Pkt).Srb, SRB_SIMPLE_TAG_REQUEST);
    SrbSetCdbLength((*Pkt).Srb, 6u32);
    SrbSetOriginalRequest((*Pkt).Srb, (*Pkt).Irp);
    SrbSetSenseInfoBuffer((*Pkt).Srb, core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as _);
    SrbSetSenseInfoBufferLength((*Pkt).Srb, (core::mem::size_of_val(&(*Pkt).SrbErrorSenseData) as u32).try_into().unwrap());
    SrbSetTimeOutValue((*Pkt).Srb, (*fdoExt).TimeOutValue);
    SrbSetDataBuffer((*Pkt).Srb, ModeSenseBuffer);
    SrbSetDataTransferLength((*Pkt).Srb, ModeSenseBufferLen as u32);

    let flags = (*fdoExt).SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE;
    SrbAssignSrbFlags((*Pkt).Srb, flags);

    pCdb = SrbGetCdb((*Pkt).Srb);
    if !pCdb.is_null() {
        (*pCdb).MODE_SENSE.OperationCode = SCSIOP_MODE_SENSE;
        (*pCdb).MODE_SENSE.PageCode = PageMode;
        (*pCdb).MODE_SENSE.SubPageCode = SubPage;
        (*pCdb).MODE_SENSE.Pc = PageControl;
        (*pCdb).MODE_SENSE.AllocationLength = ModeSenseBufferLen;
    }

    (*Pkt).BufPtrCopy = ModeSenseBuffer;
    (*Pkt).BufLenCopy = ModeSenseBufferLen;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_MODESENSE_RETRIES;
    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0;
}

#[no_mangle]
pub unsafe extern "C" fn SetupModeSelectTransferPacket(
    Pkt: *mut TRANSFER_PACKET,
    SyncEventPtr: PKEVENT,
    ModeSelectBuffer: *mut core::ffi::c_void,
    ModeSelectBufferLen: u8,
    SavePages: u8,
    OriginalIrp: PIRP,
) {
    const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0; // replace with correct value
    const SRB_FLAGS_DATA_OUT: u32 = 0; // replace with correct value
    const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0; // replace with correct value
    const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0; // replace with correct value
    const NUM_MODESELECT_RETRIES: u32 = 0; // replace with correct value
    const SRB_SIMPLE_TAG_REQUEST: u32 = 0; // replace with correct value
    const SCSIOP_MODE_SELECT: u8 = 0x55; // MODE SELECT (6) operation code

    let fdo_ptr = (*Pkt).Fdo;
    let fdoExt = (*fdo_ptr).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;

    let srb_length: u32;
    let adapter_desc = (*fdoExt).AdapterDescriptor;
    if (*adapter_desc).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        let srb_template_block = &*(*fdoData).SrbTemplate as *const STORAGE_REQUEST_BLOCK;
        srb_length = (*srb_template_block).SrbLength;
        let pkt_srb_block = &*((*Pkt).Srb as *const STORAGE_REQUEST_BLOCK);
        NT_ASSERT((*pkt_srb_block).SrbLength >= srb_length as i32)
    } else {
        srb_length = (*(*fdoData).SrbTemplate).Length;
    }

    core::ptr::copy_nonoverlapping(
        (*fdoData).SrbTemplate as *const u8,
        (*Pkt).Srb as *mut u8,
        srb_length as usize,
    );

    SrbSetRequestAttribute((*Pkt).Srb, SRB_SIMPLE_TAG_REQUEST);
    SrbSetCdbLength((*Pkt).Srb, 6);
    SrbSetOriginalRequest((*Pkt).Srb, (*Pkt).Irp);
    SrbSetSenseInfoBuffer((*Pkt).Srb, core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData).cast::<u8>());
    SrbSetSenseInfoBufferLength((*Pkt).Srb, (core::mem::size_of_val(&&raw const (*Pkt).SrbErrorSenseData) as u32).try_into().unwrap());
    SrbSetTimeOutValue((*Pkt).Srb, (*fdoExt).TimeOutValue);
    SrbSetDataBuffer((*Pkt).Srb, ModeSelectBuffer);
    SrbSetDataTransferLength((*Pkt).Srb, ModeSelectBufferLen as u32);

    SrbAssignSrbFlags(
        (*Pkt).Srb,
        (*fdoExt).SrbFlags | SRB_FLAGS_DATA_OUT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE,
    );

    let pCdb = SrbGetCdb((*Pkt).Srb);
    if !pCdb.is_null() {
        (*pCdb).MODE_SELECT.OperationCode = SCSIOP_MODE_SELECT;
        (*pCdb).MODE_SELECT.SPBit = SavePages;
        (*pCdb).MODE_SELECT.PFBit = 1;
        (*pCdb).MODE_SELECT.ParameterListLength = ModeSelectBufferLen;
    }

    (*Pkt).BufPtrCopy = ModeSelectBuffer;
    (*Pkt).BufLenCopy = ModeSelectBufferLen as u32;
    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_MODESELECT_RETRIES;
    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0u8;
}

// Extern declarations for unknown functions and constants

// Placeholder constants (actual values must be provided by caller)
const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0; // placeholder
const SCSIOP_READ_CAPACITY16: u8 = 0;
const SCSIOP_READ_CAPACITY: u8 = 0;
const NUM_DRIVECAPACITY_RETRIES: u32 = 0;
#[no_mangle]
pub unsafe extern "C" fn SetupDriveCapacityTransferPacket(
    Pkt: *mut TRANSFER_PACKET,
    ReadCapacityBuffer: *mut core::ffi::c_void,
    ReadCapacityBufferLen: u32,
    SyncEventPtr: PKEVENT,
    OriginalIrp: PIRP,
    Use16ByteCdb: i32, // BOOLEAN
) {
    let fdo_ext = (*(*Pkt).Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdo_data = (*fdo_ext).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
    let timeout_value = (*fdo_ext).TimeOutValue;

    let srb_template = (*fdo_data).SrbTemplate as *mut STORAGE_REQUEST_BLOCK;
    let srb_length: u32;
    if (*(*fdo_ext).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srb_length = (*srb_template).SrbLength;
        NT_ASSERT(((*Pkt).Srb as *mut STORAGE_REQUEST_BLOCK).SrbLength >= srb_length);
    } else {
        srb_length = (*srb_template).Length;
    }

    core::ptr::copy_nonoverlapping(
        (*fdo_data).SrbTemplate as *const core::ffi::c_void,
        (*Pkt).Srb as *mut core::ffi::c_void,
        srb_length as usize,
    );

    SrbSetRequestAttribute((*Pkt).Srb as *mut core::ffi::c_void, SRB_SIMPLE_TAG_REQUEST);
    SrbSetOriginalRequest((*Pkt).Srb as *mut core::ffi::c_void, (*Pkt).Irp);
    SrbSetSenseInfoBuffer(
        (*Pkt).Srb as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*Pkt).SrbErrorSenseData) as *mut core::ffi::c_void,
    );
    SrbSetSenseInfoBufferLength(
        (*Pkt).Srb as *mut core::ffi::c_void,
        (core::mem::size_of_val(&(*Pkt).SrbErrorSenseData) as u32).try_into().unwrap(),
    );

    SrbSetTimeOutValue((*Pkt).Srb as *mut core::ffi::c_void, timeout_value);
    SrbSetDataBuffer((*Pkt).Srb as *mut core::ffi::c_void, ReadCapacityBuffer);
    SrbSetDataTransferLength((*Pkt).Srb as *mut core::ffi::c_void, ReadCapacityBufferLen);

    SrbAssignSrbFlags(
        (*Pkt).Srb as *mut core::ffi::c_void,
        (*fdo_ext).SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE,
    );

    let p_cdb = SrbGetCdb((*Pkt).Srb as *mut core::ffi::c_void);
    if !p_cdb.is_null() {
        if Use16ByteCdb != 0 {
            NT_ASSERT(ReadCapacityBufferLen as usize >= core::mem::size_of::<READ_CAPACITY_DATA_EX>());
            SrbSetCdbLength((*Pkt).Srb as *mut core::ffi::c_void, 16);
            // Write OperationCode at offset 0 of CDB16
            *((*p_cdb).CDB16 as *mut u8) = SCSIOP_READ_CAPACITY16;
            REVERSE_BYTES(
                core::ptr::addr_of_mut!((*p_cdb).CDB16.TransferLength) as *mut core::ffi::c_void,
                core::ptr::addr_of!(ReadCapacityBufferLen) as *const core::ffi::c_void,
            );
            // AsByte[1] = 0x10
            *(((*p_cdb).AsByte) as *mut u8).add(1) = 0x10;
        } else {
            SrbSetCdbLength((*Pkt).Srb as *mut core::ffi::c_void, 10);
            // Write OperationCode at offset 0 of CDB10
            *((*p_cdb).CDB10 as *mut u8) = SCSIOP_READ_CAPACITY;
        }
    }

    (*Pkt).BufPtrCopy = ReadCapacityBuffer;
    (*Pkt).BufLenCopy = ReadCapacityBufferLen;

    (*Pkt).OriginalIrp = OriginalIrp;
    (*Pkt).NumRetries = NUM_DRIVECAPACITY_RETRIES;
    // #if !defined(__REACTOS__) && NTDDI_VERSION >= NTDDI_WINBLUE
    (*Pkt).NumIoTimeoutRetries = NUM_DRIVECAPACITY_RETRIES;

    (*Pkt).SyncEventPtr = SyncEventPtr;
    (*Pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0u8; // FALSE
}

#[no_mangle]
pub unsafe extern "system" fn CleanupTransferPacketToWorkingSetSizeWorker(
    Fdo: *mut core::ffi::c_void,
    Context: *mut core::ffi::c_void,
    IoWorkItem: PIO_WORKITEM
) {

    let node = Context as u32;

    PAGED_CODE();

    CleanupTransferPacketToWorkingSetSize(Fdo as *mut DEVICE_OBJECT, 0i32, node);

    //
    // Release the remove lock acquired in EnqueueFreeTransferPacket
    //
    ClassReleaseRemoveLock(Fdo as *mut DEVICE_OBJECT, IoWorkItem as PIRP);

    if !IoWorkItem.is_null() {
        IoFreeWorkItem(IoWorkItem);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CleanupTransferPacketToWorkingSetSize(
    Fdo: PDEVICE_OBJECT,
    LimitNumPktToDelete: i32,
    Node: u32,
) {
    let fdoExt = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData as *mut CLASS_PRIVATE_FDO_DATA;
    let mut oldIrql: KIRQL;
    let mut pktList: SINGLE_LIST_ENTRY = core::mem::zeroed();
    let mut slistEntry: PSINGLE_LIST_ENTRY;
    let mut pktToDelete: *mut TRANSFER_PACKET;
    let mut requiredNumPktToDelete = (*fdoData).FreeTransferPacketsLists[Node as usize].NumTotalTransferPackets -
                                     (*fdoData).LocalMaxWorkingSetTransferPackets;

    if LimitNumPktToDelete != 0 {
        requiredNumPktToDelete = if requiredNumPktToDelete > MAX_CLEANUP_TRANSFER_PACKETS_AT_ONCE {
            MAX_CLEANUP_TRANSFER_PACKETS_AT_ONCE
        } else {
            requiredNumPktToDelete
        };
    }

    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW,
        b"CleanupTransferPacketToWorkingSetSize (%p): Exiting stress, block freeing %d packets.\0".as_ptr() as *const i8,
        Fdo, requiredNumPktToDelete);

    SimpleInitSlistHdr(&mut pktList as *mut SINGLE_LIST_ENTRY);
    KeAcquireSpinLock(&mut (*fdoData).SpinLock, &mut oldIrql);
    while (*fdoData).FreeTransferPacketsLists[Node as usize].NumFreeTransferPackets >=
           (*fdoData).FreeTransferPacketsLists[Node as usize].NumTotalTransferPackets &&
           (*fdoData).FreeTransferPacketsLists[Node as usize].NumTotalTransferPackets >
           (*fdoData).LocalMaxWorkingSetTransferPackets &&
           {
               let orig = requiredNumPktToDelete;
               requiredNumPktToDelete -= 1;
               orig != 0
           }
    {
        pktToDelete = DequeueFreeTransferPacketEx(Fdo, 0, Node);
        if !pktToDelete.is_null() {
            SimplePushSlist(&mut pktList as *mut SINGLE_LIST_ENTRY,
                            &mut (*pktToDelete).SlistEntry as *mut SINGLE_LIST_ENTRY);
            InterlockedDecrement(
                &mut (*fdoData).FreeTransferPacketsLists[Node as usize].NumTotalTransferPackets as *mut i32
            );
        } else {
            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW,
                b"Extremely unlikely condition (non-fatal): %d packets dequeued at once for Fdo %p. NumTotalTransferPackets=%d (1). Node=%d\0".as_ptr() as *const i8,
                (*fdoData).LocalMaxWorkingSetTransferPackets,
                Fdo,
                (*fdoData).FreeTransferPacketsLists[Node as usize].NumTotalTransferPackets,
                Node);
            break;
        }
    }
    KeReleaseSpinLock(&mut (*fdoData).SpinLock, oldIrql);

    slistEntry = SimplePopSlist(&mut pktList as *mut SINGLE_LIST_ENTRY);
    while !slistEntry.is_null() {
        pktToDelete = (slistEntry as *mut u8).offset(
            -(core::mem::offset_of!(TRANSFER_PACKET, SlistEntry) as isize)
        ) as *mut TRANSFER_PACKET;
        DestroyTransferPacket(pktToDelete);
        slistEntry = SimplePopSlist(&mut pktList as *mut SINGLE_LIST_ENTRY);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClasspSetupReceiveWriteUsingTokenInformationTransferPacket(
    offload_write_context: *mut OFFLOAD_WRITE_CONTEXT,
    pkt: *mut TRANSFER_PACKET,
    length: u32,
    receive_write_using_token_information_buffer: *mut u8,
    original_irp: *mut IRP,
    list_identifier: u32,
) {

    const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 0x00; // actual value unknown
    const SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION: u8 = 0x00;
    const SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION: u8 = 0x00;
    const NUM_WRITE_USING_TOKEN_RETRIES: u32 = 3; // example
    const TRACE_LEVEL_VERBOSE: u32 = 5;
    const TRACE_FLAG_IOCTL: u32 = 0x100;

    let fdo_ext: *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdo_data: *mut CLASS_PRIVATE_FDO_DATA;
    let pc_db: *mut CDB;
    let srb_length: u32;

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspSetupReceiveWriteUsingTokenInformationTransferPacket (%p): Entering function. Irp %p\n\0".as_ptr() as *const i8,
        (*pkt).Fdo,
        original_irp,
    );

    fdo_ext = (*(*pkt).Fdo).DeviceExtension;
    fdo_data = (*fdo_ext).PrivateFdoData;

    if (*(*fdo_ext).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srb_length = (*(*(fdo_data as *mut STORAGE_REQUEST_BLOCK)).SrbTemplate).SrbLength;
        // NT_ASSERT replacement
        debug_assert!(
            (*(*pkt).Srb as *mut STORAGE_REQUEST_BLOCK).SrbLength >= srb_length,
            "SrbLength check failed"
        );
    } else {
        srb_length = (*(*fdo_data).SrbTemplate).Length;
    }

    core::ptr::copy_nonoverlapping(
        (*fdo_data).SrbTemplate as *const u8,
        (*pkt).Srb as *mut u8,
        srb_length as usize,
    );

    SrbSetRequestAttribute((*pkt).Srb, SRB_SIMPLE_TAG_REQUEST);
    SrbSetCdbLength((*pkt).Srb, 16);
    SrbSetOriginalRequest((*pkt).Srb, (*pkt).Irp);
    SrbSetSenseInfoBuffer(
        (*pkt).Srb,
        core::ptr::addr_of_mut!((*pkt).SrbErrorSenseData) as *mut core::ffi::c_void,
    );
    SrbSetSenseInfoBufferLength(
        (*pkt).Srb,
        (core::mem::size_of_val(&(*(*pkt)).SrbErrorSenseData) as u32).try_into().unwrap(),
    );
    SrbSetTimeOutValue((*pkt).Srb, (*fdo_ext).TimeOutValue);
    SrbSetDataBuffer((*pkt).Srb, receive_write_using_token_information_buffer as *mut core::ffi::c_void);
    SrbSetDataTransferLength((*pkt).Srb, length);

    SrbAssignSrbFlags(
        (*pkt).Srb,
        (*fdo_ext).SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE,
    );

    pc_db = SrbGetCdb((*pkt).Srb);
    if !pc_db.is_null() {
        (*(*pc_db).RECEIVE_TOKEN_INFORMATION).OperationCode = SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION;
        (*(*pc_db).RECEIVE_TOKEN_INFORMATION).ServiceAction = SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION;

        REVERSE_BYTES(
            &mut (*(*pc_db).RECEIVE_TOKEN_INFORMATION).AllocationLength as *mut u32 as *mut core::ffi::c_void,
            &length as *const u32 as *const core::ffi::c_void,
        );
        REVERSE_BYTES(
            &mut (*(*pc_db).RECEIVE_TOKEN_INFORMATION).ListIdentifier as *mut u32 as *mut core::ffi::c_void,
            &list_identifier as *const u32 as *const core::ffi::c_void,
        );
    }

    (*pkt).BufPtrCopy = receive_write_using_token_information_buffer;
    (*pkt).BufLenCopy = length;

    (*pkt).OriginalIrp = original_irp;
    (*pkt).NumRetries = NUM_WRITE_USING_TOKEN_RETRIES;
    (*pkt).CompleteOriginalIrpWhenLastPacketCompletes = 0; // FALSE

    (*pkt).ContinuationRoutine = ClasspReceiveWriteUsingTokenInformationTransferPacketDone as *mut core::ffi::c_void;
    (*pkt).ContinuationContext = offload_write_context as *mut core::ffi::c_void;

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspSetupReceiveWriteUsingTokenInformationTransferPacket (%p): Exiting function with Irp %p\n\0".as_ptr() as *const i8,
        (*pkt).Fdo,
        original_irp,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1991 - 2010
//
// Module Name:
//
//     xferpkt.c
//
// Abstract:
//
//     Packet routines for CLASSPNP
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #include "classp.h"
// #include "debug.h"
//
// #ifdef DEBUG_USE_WPP
// #include "xferpkt.tmh"
// #endif
//
// #ifdef ALLOC_PRAGMA
//     #pragma alloc_text(PAGE, InitializeTransferPackets)
//     #pragma alloc_text(PAGE, DestroyAllTransferPackets)
//     #pragma alloc_text(PAGE, SetupEjectionTransferPacket)
//     #pragma alloc_text(PAGE, SetupModeSenseTransferPacket)
//     #pragma alloc_text(PAGE, CleanupTransferPacketToWorkingSetSizeWorker)
//     #pragma alloc_text(PAGE, ClasspSetupPopulateTokenTransferPacket)
// #endif
//
// /*
//  *  InitializeTransferPackets
//  *
//  *      Allocate/initialize TRANSFER_PACKETs and related resources.
//  */
// NTSTATUS InitializeTransferPackets(PDEVICE_OBJECT Fdo)
// {
//     PCOMMON_DEVICE_EXTENSION commonExt = Fdo->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PSTORAGE_ADAPTER_DESCRIPTOR adapterDesc = commonExt->PartitionZeroExtension->AdapterDescriptor;
//     PSTORAGE_DEVICE_IO_CAPABILITY_DESCRIPTOR devIoCapabilityDesc = NULL;
//     STORAGE_PROPERTY_ID propertyId;
//     OSVERSIONINFOEXW osVersionInfo;
//     ULONG hwMaxPages;
//     ULONG arraySize;
//     ULONG index;
//     ULONG maxOutstandingIOPerLUN;
//     ULONG minWorkingSetTransferPackets;
//     ULONG maxWorkingSetTransferPackets;
//
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     //  Precompute the maximum transfer length
//     //
//     NT_ASSERT(adapterDesc->MaximumTransferLength as i32)
//
//     hwMaxPages = adapterDesc->MaximumPhysicalPages ? adapterDesc->MaximumPhysicalPages-1 : 0;
//
//     fdoData->HwMaxXferLen = MIN(adapterDesc->MaximumTransferLength, hwMaxPages << PAGE_SHIFT);
//     fdoData->HwMaxXferLen = MAX(fdoData->HwMaxXferLen, PAGE_SIZE);
//
//     //
//     // Allocate per-node free packet lists
//     //
//     arraySize = KeQueryHighestNodeNumber() + 1;
//     fdoData->FreeTransferPacketsLists =
//         ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                               sizeof(PNL_SLIST_HEADER) * arraySize,
//                               CLASS_TAG_PRIVATE_DATA);
//
//     if (fdoData->FreeTransferPacketsLists == NULL) {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         return status;
//     }
//
//     for (index = 0; index < arraySize; index++) {
//         InitializeSListHead(&(fdoData->FreeTransferPacketsLists[index].SListHeader));
//         fdoData->FreeTransferPacketsLists[index].NumTotalTransferPackets = 0;
//         fdoData->FreeTransferPacketsLists[index].NumFreeTransferPackets = 0;
//     }
//
//     InitializeListHead(&fdoData->AllTransferPacketsList);
//
//     //
//     // Set the packet threshold numbers based on the Windows Client or Server SKU.
//     //
//
//     osVersionInfo.dwOSVersionInfoSize = sizeof(OSVERSIONINFOEXW);
//     status = RtlGetVersion((POSVERSIONINFOW) &osVersionInfo);
//
//     NT_ASSERT( NT_SUCCESS(status) as i32)
//
//     //
//     // Retrieve info on IO capability supported by port drivers
//     //
//
//     propertyId = StorageDeviceIoCapabilityProperty;
//     status = ClassGetDescriptor(fdoExt->CommonExtension.LowerDeviceObject,
//                                 &propertyId,
//                                 (PVOID *)&devIoCapabilityDesc);
//
//     if (NT_SUCCESS(status) && (devIoCapabilityDesc != NULL)) {
//         maxOutstandingIOPerLUN = devIoCapabilityDesc->LunMaxIoCount;
//         FREE_POOL(devIoCapabilityDesc);
//
// #if DBG
//         fdoData->MaxOutstandingIOPerLUN = maxOutstandingIOPerLUN;
// #endif
//
//     } else {
//         maxOutstandingIOPerLUN = MAX_OUTSTANDING_IO_PER_LUN_DEFAULT;
//
// #if DBG
//         fdoData->MaxOutstandingIOPerLUN = 0;
// #endif
//
//     }
//
//     //
//     // StorageDeviceIoCapabilityProperty support is optional so
//     // ignore any failures.
//     //
//
//     status = STATUS_SUCCESS;
//
//
//     if ((osVersionInfo.wProductType != VER_NT_DOMAIN_CONTROLLER) &&
//         (osVersionInfo.wProductType != VER_NT_SERVER)) {
//
//         // this is Client SKU
//
//             minWorkingSetTransferPackets = MIN_WORKINGSET_TRANSFER_PACKETS_Client;
//
//         // Note: the reason we use max here is to guarantee a reasonable large max number
//         // in the case where the port driver may return a very small supported outstanding
//         // IOs. For example, even EMMC drive only reports 1 outstanding IO supported, we
//         // may still want to set this value to be at least
//         // MAX_WORKINGSET_TRANSFER_PACKETS_Client.
//         maxWorkingSetTransferPackets = max(MAX_WORKINGSET_TRANSFER_PACKETS_Client,
//                                            2 * maxOutstandingIOPerLUN);
//
//     } else {
//
//         // this is Server SKU
//         // Note: the addition max here to make sure we set the min to be at least
//         // MIN_WORKINGSET_TRANSFER_PACKETS_Server_LowerBound no matter what maxOutstandingIOPerLUN
//         // reported. We shouldn't set this value to be smaller than client system.
//         // In other words, the minWorkingSetTransferPackets for server will always between
//         // MIN_WORKINGSET_TRANSFER_PACKETS_Server_LowerBound and MIN_WORKINGSET_TRANSFER_PACKETS_Server_UpperBound
//
//         minWorkingSetTransferPackets =
//             max(MIN_WORKINGSET_TRANSFER_PACKETS_Server_LowerBound,
//                 min(MIN_WORKINGSET_TRANSFER_PACKETS_Server_UpperBound,
//                     maxOutstandingIOPerLUN));
//
//         maxWorkingSetTransferPackets = max(MAX_WORKINGSET_TRANSFER_PACKETS_Server,
//                                            2 * maxOutstandingIOPerLUN);
//     }
//
//
//     fdoData->LocalMinWorkingSetTransferPackets = minWorkingSetTransferPackets;
//     fdoData->LocalMaxWorkingSetTransferPackets = maxWorkingSetTransferPackets;
//
//     //
//     //  Allow class driver to override the settings
//     //
//     if (commonExt->DriverExtension->WorkingSet != NULL) {
//         PCLASS_WORKING_SET workingSet = commonExt->DriverExtension->WorkingSet;
//
//         // override only if non-zero
//         if (workingSet->XferPacketsWorkingSetMinimum != 0)
//         {
//             fdoData->LocalMinWorkingSetTransferPackets = workingSet->XferPacketsWorkingSetMinimum;
//             // adjust maximum upwards if needed
//             if (fdoData->LocalMaxWorkingSetTransferPackets < fdoData->LocalMinWorkingSetTransferPackets)
//             {
//                 fdoData->LocalMaxWorkingSetTransferPackets = fdoData->LocalMinWorkingSetTransferPackets;
//             }
//         }
//         // override only if non-zero
//         if (workingSet->XferPacketsWorkingSetMaximum != 0)
//         {
//             fdoData->LocalMaxWorkingSetTransferPackets = workingSet->XferPacketsWorkingSetMaximum;
//             // adjust minimum downwards if needed
//             if (fdoData->LocalMinWorkingSetTransferPackets > fdoData->LocalMaxWorkingSetTransferPackets)
//             {
//                 fdoData->LocalMinWorkingSetTransferPackets = fdoData->LocalMaxWorkingSetTransferPackets;
//             }
//         }
//         // that's all the adjustments required/allowed
//     } // end working set size special code
//
//     for (index = 0; index < arraySize; index++) {
//         while (fdoData->FreeTransferPacketsLists[index].NumFreeTransferPackets < MIN_INITIAL_TRANSFER_PACKETS){
//             PTRANSFER_PACKET pkt = NewTransferPacket(Fdo);
//             if (pkt) {
//                 InterlockedIncrement((volatile LONG *)&(fdoData->FreeTransferPacketsLists[index].NumTotalTransferPackets));
//                 pkt->AllocateNode = index;
//                 EnqueueFreeTransferPacket(Fdo, pkt);
//             } else {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//         }
//         fdoData->FreeTransferPacketsLists[index].DbgPeakNumTransferPackets = fdoData->FreeTransferPacketsLists[index].NumTotalTransferPackets;
//     }
//
//     //
//     //  Pre-initialize our SCSI_REQUEST_BLOCK template with all
//     //  the constant fields.  This will save a little time for each xfer.
//     //  NOTE: a CdbLength field of 10 may not always be appropriate
//     //
//
//     if (NT_SUCCESS(status))  {
//         if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//             ULONG ByteSize = 0;
//
//     #if (NTDDI_VERSION >= NTDDI_WINBLUE)
//             if ((fdoExt->MiniportDescriptor != NULL) &&
//                 (fdoExt->MiniportDescriptor->Size >= RTL_SIZEOF_THROUGH_FIELD(STORAGE_MINIPORT_DESCRIPTOR, ExtraIoInfoSupported)) &&
//                 (fdoExt->MiniportDescriptor->ExtraIoInfoSupported == TRUE)) {
//                 status = CreateStorageRequestBlock((PSTORAGE_REQUEST_BLOCK *)&fdoData->SrbTemplate,
//                                                     fdoExt->AdapterDescriptor->AddressType,
//                                                     DefaultStorageRequestBlockAllocateRoutine,
//                                                     &ByteSize,
//                                                     2,
//                                                     SrbExDataTypeScsiCdb16,
//                                                     SrbExDataTypeIoInfo
//                                                   );
//             } else {
//                 status = CreateStorageRequestBlock((PSTORAGE_REQUEST_BLOCK *)&fdoData->SrbTemplate,
//                                                     fdoExt->AdapterDescriptor->AddressType,
//                                                     DefaultStorageRequestBlockAllocateRoutine,
//                                                     &ByteSize,
//                                                     1,
//                                                     SrbExDataTypeScsiCdb16
//                                                   );
//             }
//     #else
//             status = CreateStorageRequestBlock((PSTORAGE_REQUEST_BLOCK *)&fdoData->SrbTemplate,
//                                                 fdoExt->AdapterDescriptor->AddressType,
//                                                 DefaultStorageRequestBlockAllocateRoutine,
//                                                 &ByteSize,
//                                                 1,
//                                                 SrbExDataTypeScsiCdb16
//                                                 );
//     #endif
//             if (NT_SUCCESS(status)) {
//                 ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//             } else {
//                 NT_ASSERT(FALSE as i32)
//             }
//         } else {
//             fdoData->SrbTemplate = ExAllocatePoolWithTag(NonPagedPoolNx, sizeof(SCSI_REQUEST_BLOCK), '-brs');
//             if (fdoData->SrbTemplate == NULL) {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//             } else {
//                 RtlZeroMemory(fdoData->SrbTemplate, sizeof(SCSI_REQUEST_BLOCK));
//                 fdoData->SrbTemplate->Length = sizeof(SCSI_REQUEST_BLOCK);
//                 fdoData->SrbTemplate->Function = SRB_FUNCTION_EXECUTE_SCSI;
//             }
//         }
//     }
//
//     if (status == STATUS_SUCCESS) {
//         SrbSetRequestAttribute(fdoData->SrbTemplate, SRB_SIMPLE_TAG_REQUEST);
//         SrbSetSenseInfoBufferLength(fdoData->SrbTemplate, SENSE_BUFFER_SIZE_EX);
//         SrbSetCdbLength(fdoData->SrbTemplate, 10);
//     }
//
//     return status;
// }
//
//
// VOID DestroyAllTransferPackets(PDEVICE_OBJECT Fdo)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     TRANSFER_PACKET *pkt;
//     ULONG index;
//     ULONG arraySize;
//
//     PAGED_CODE();
//
//     //
//     // fdoData->FreeTransferPacketsLists could be NULL if
//     // there was an error during start device.
//     //
//     if (fdoData->FreeTransferPacketsLists != NULL) {
//
//         NT_ASSERT(IsListEmpty(&fdoData->DeferredClientIrpList) as i32)
//
//         arraySize = KeQueryHighestNodeNumber() + 1;
//         for (index = 0; index < arraySize; index++) {
//             pkt = DequeueFreeTransferPacketEx(Fdo, FALSE, index);
//             while (pkt) {
//                 DestroyTransferPacket(pkt);
//                 InterlockedDecrement((volatile LONG *)&(fdoData->FreeTransferPacketsLists[index].NumTotalTransferPackets));
//                 pkt = DequeueFreeTransferPacketEx(Fdo, FALSE, index);
//             }
//
//             NT_ASSERT(fdoData->FreeTransferPacketsLists[index].NumTotalTransferPackets == 0 as i32)
//         }
//     }
//
//     FREE_POOL(fdoData->SrbTemplate);
// }
//
// __drv_allocatesMem(Mem)
// #ifdef _MSC_VER
// #pragma warning(suppress:28195) // This function may not allocate memory in some error cases.
// #endif
// PTRANSFER_PACKET NewTransferPacket(PDEVICE_OBJECT Fdo)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PTRANSFER_PACKET newPkt = NULL;
//     ULONG transferLength = (ULONG)-1;
//     NTSTATUS status = STATUS_SUCCESS;
//
//     if (NT_SUCCESS(status)) {
//         status = RtlULongAdd(fdoData->HwMaxXferLen, PAGE_SIZE, &transferLength);
//         if (!NT_SUCCESS(status)) {
//
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_RW, "Integer overflow in calculating transfer packet size."));
//             status = STATUS_INTEGER_OVERFLOW;
//         }
//     }
//
//     /*
//      *  Allocate the actual packet.
//      */
//     if (NT_SUCCESS(status)) {
//         newPkt = ExAllocatePoolWithTag(NonPagedPoolNx, sizeof(TRANSFER_PACKET), 'pnPC');
//         if (newPkt == NULL) {
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_RW, "Failed to allocate transfer packet."));
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         } else {
//             RtlZeroMemory(newPkt, sizeof(TRANSFER_PACKET));
//             newPkt->AllocateNode = KeGetCurrentNodeNumber();
//             if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
// #if (NTDDI_VERSION >= NTDDI_WINBLUE)
//                 if ((fdoExt->MiniportDescriptor != NULL) &&
//                     (fdoExt->MiniportDescriptor->Size >= RTL_SIZEOF_THROUGH_FIELD(STORAGE_MINIPORT_DESCRIPTOR, ExtraIoInfoSupported)) &&
//                     (fdoExt->MiniportDescriptor->ExtraIoInfoSupported == TRUE)) {
//                     status = CreateStorageRequestBlock((PSTORAGE_REQUEST_BLOCK *)&newPkt->Srb,
//                                         fdoExt->AdapterDescriptor->AddressType,
//                                         DefaultStorageRequestBlockAllocateRoutine,
//                                         NULL,
//                                         2,
//                                         SrbExDataTypeScsiCdb16,
//                                         SrbExDataTypeIoInfo
//                                         );
//                 } else {
//                     status = CreateStorageRequestBlock((PSTORAGE_REQUEST_BLOCK *)&newPkt->Srb,
//                                         fdoExt->AdapterDescriptor->AddressType,
//                                         DefaultStorageRequestBlockAllocateRoutine,
//                                         NULL,
//                                         1,
//                                         SrbExDataTypeScsiCdb16
//                                         );
//                 }
// #else
//                 status = CreateStorageRequestBlock((PSTORAGE_REQUEST_BLOCK *)&newPkt->Srb,
//                                     fdoExt->AdapterDescriptor->AddressType,
//                                     DefaultStorageRequestBlockAllocateRoutine,
//                                     NULL,
//                                     1,
//                                     SrbExDataTypeScsiCdb16
//                                     );
// #endif
//             } else {
// #ifdef _MSC_VER
// #pragma prefast(suppress:6014, "The allocated memory that Pkt->Srb points to will be freed in DestroyTransferPacket().")
// #endif
//                 newPkt->Srb = ExAllocatePoolWithTag(NonPagedPoolNx, sizeof(SCSI_REQUEST_BLOCK), '-brs');
//                 if (newPkt->Srb == NULL) {
//                     status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//
//             }
//
//             if (status != STATUS_SUCCESS)
//             {
//                 TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_RW, "Failed to allocate SRB."));
//                 FREE_POOL(newPkt);
//             }
//         }
//     }
//
//     /*
//      *  Allocate Irp for the packet.
//      */
//     if (NT_SUCCESS(status) && newPkt != NULL) {
//         newPkt->Irp = IoAllocateIrp(Fdo->StackSize, FALSE);
//         if (newPkt->Irp == NULL) {
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_RW, "Failed to allocate IRP for transfer packet."));
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     /*
//      * Allocate a MDL.  Add one page to the length to insure an extra page
//      * entry is allocated if the buffer does not start on page boundaries.
//      */
//     if (NT_SUCCESS(status) && newPkt != NULL) {
//
//         NT_ASSERT(transferLength != (ULONG)-1 as i32)
//
//         newPkt->PartialMdl = IoAllocateMdl(NULL,
//                                            transferLength,
//                                            FALSE,
//                                            FALSE,
//                                            NULL);
//         if (newPkt->PartialMdl == NULL) {
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_RW, "Failed to allocate MDL for transfer packet."));
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         } else {
//             NT_ASSERT(newPkt->PartialMdl->Size >= (CSHORT)(sizeof(MDL) + BYTES_TO_PAGES(fdoData->HwMaxXferLen) * sizeof(PFN_NUMBER)) as i32)
//         }
//
//     }
//
//     /*
//      * Allocate per-packet retry history, if required
//      */
//     if (NT_SUCCESS(status) &&
//         (fdoData->InterpretSenseInfo != NULL) &&
//         (newPkt != NULL)
//         ) {
//         // attempt to allocate also the history
//         ULONG historyByteCount = 0;
//
//         // SAL annotation and ClassInitializeEx() should both catch this case
//         NT_ASSERT(fdoData->InterpretSenseInfo->HistoryCount != 0 as i32)
//         _Analysis_assume_(fdoData->InterpretSenseInfo->HistoryCount != 0);
//
//         historyByteCount = sizeof(SRB_HISTORY_ITEM) * fdoData->InterpretSenseInfo->HistoryCount;
//         historyByteCount += sizeof(SRB_HISTORY) - sizeof(SRB_HISTORY_ITEM);
//
//         newPkt->RetryHistory = (PSRB_HISTORY)ExAllocatePoolWithTag(NonPagedPoolNx, historyByteCount, 'hrPC');
//
//         if (newPkt->RetryHistory == NULL) {
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_RW, "Failed to allocate MDL for transfer packet."));
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         } else {
//             // call this routine directly once since it's the first initialization of
//             // the structure and the internal maximum count field is not yet setup.
//             HistoryInitializeRetryLogs(newPkt->RetryHistory, fdoData->InterpretSenseInfo->HistoryCount);
//         }
//     }
//
//     /*
//      *  Enqueue the packet in our static AllTransferPacketsList
//      *  (just so we can find it during debugging if its stuck somewhere).
//      */
//     if (NT_SUCCESS(status) && newPkt != NULL)
//     {
//         KIRQL oldIrql;
//         newPkt->Fdo = Fdo;
// #if DBG
//         newPkt->DbgPktId = InterlockedIncrement((volatile LONG *)&fdoData->DbgMaxPktId);
// #endif
//         KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//         InsertTailList(&fdoData->AllTransferPacketsList, &newPkt->AllPktsListEntry);
//         KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
//
//     } else {
//         // free any resources acquired above (in reverse order)
//         if (newPkt != NULL) {
//             FREE_POOL(newPkt->RetryHistory);
//             if (newPkt->PartialMdl != NULL) { IoFreeMdl(newPkt->PartialMdl); }
//             if (newPkt->Irp        != NULL) { IoFreeIrp(newPkt->Irp);        }
//             if (newPkt->Srb        != NULL) { FREE_POOL(newPkt->Srb);        }
//             FREE_POOL(newPkt);
//         }
//     }
//
//     return newPkt;
// }
//
//
// /*
//  *  DestroyTransferPacket
//  *
//  */
// VOID DestroyTransferPacket( __drv_freesMem(mem) PTRANSFER_PACKET Pkt)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     KIRQL oldIrql;
//
//     NT_ASSERT(!Pkt->SlistEntry.Next as i32)
// //    NT_ASSERT(!Pkt->OriginalIrp as i32)
//
//     KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//
//     /*
//      *  Delete the packet from our all-packets queue.
//      */
//     NT_ASSERT(!IsListEmpty(&Pkt->AllPktsListEntry) as i32)
//     NT_ASSERT(!IsListEmpty(&fdoData->AllTransferPacketsList) as i32)
//     RemoveEntryList(&Pkt->AllPktsListEntry);
//     InitializeListHead(&Pkt->AllPktsListEntry);
//
//     KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
//
//     IoFreeMdl(Pkt->PartialMdl);
//     IoFreeIrp(Pkt->Irp);
//     FREE_POOL(Pkt->RetryHistory);
//     FREE_POOL(Pkt->Srb);
//     FREE_POOL(Pkt);
// }
//
//
// VOID EnqueueFreeTransferPacket(PDEVICE_OBJECT Fdo, __drv_aliasesMem PTRANSFER_PACKET Pkt)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     ULONG allocateNode;
//     KIRQL oldIrql;
//
//     NT_ASSERT(!Pkt->SlistEntry.Next as i32)
//
//     allocateNode = Pkt->AllocateNode;
//     InterlockedPushEntrySList(&(fdoData->FreeTransferPacketsLists[allocateNode].SListHeader), &Pkt->SlistEntry);
//     InterlockedIncrement((volatile LONG *)&(fdoData->FreeTransferPacketsLists[allocateNode].NumFreeTransferPackets));
//
//     /*
//      *  If the total number of packets is larger than LocalMinWorkingSetTransferPackets,
//      *  that means that we've been in stress.  If all those packets are now
//      *  free, then we are now out of stress and can free the extra packets.
//      *  Attempt to free down to LocalMaxWorkingSetTransferPackets immediately, and
//      *  down to LocalMinWorkingSetTransferPackets lazily (one at a time).
//      *  However, since we're at DPC, do this is a work item. If the device is removed
//      *  or we are unable to allocate the work item, do NOT free more than
//      *  MAX_CLEANUP_TRANSFER_PACKETS_AT_ONCE. Subsequent IO completions will end up freeing
//      *  up the rest, even if it is MAX_CLEANUP_TRANSFER_PACKETS_AT_ONCE at a time.
//      */
//     if (fdoData->FreeTransferPacketsLists[allocateNode].NumFreeTransferPackets >=
//         fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets) {
//
//         /*
//          *  1.  Immediately snap down to our UPPER threshold.
//          */
//         if (fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets >
//             fdoData->LocalMaxWorkingSetTransferPackets) {
//
//             ULONG isRemoved;
//             PIO_WORKITEM workItem = NULL;
//
//             workItem = IoAllocateWorkItem(Fdo);
//
//             //
//             // Acquire a remove lock in order to make sure the device object and its
//             // private data structures will exist when the workitem fires.
//             // The remove lock will be released by the workitem (CleanupTransferPacketToWorkingSetSize).
//             //
//             isRemoved = ClassAcquireRemoveLock(Fdo, (PIRP)workItem);
//
//             if (workItem && !isRemoved) {
//
//                 TracePrint((TRACE_LEVEL_INFORMATION,
//                             TRACE_FLAG_GENERAL,
//                             "EnqueueFreeTransferPacket: Device (%p), queuing work item to clean up free transfer packets.\n",
//                             Fdo));
//
//                 //
//                 // Queue a work item to trim down the total number of transfer packets to with the
//                 // working size.
//                 //
//                 IoQueueWorkItemEx(workItem, CleanupTransferPacketToWorkingSetSizeWorker, DelayedWorkQueue, (PVOID)(ULONG_PTR)allocateNode);
//
//             } else {
//
//                 if (workItem) {
//                     IoFreeWorkItem(workItem);
//                 }
//
//                 if (isRemoved != REMOVE_COMPLETE) {
//                     ClassReleaseRemoveLock(Fdo, (PIRP)workItem);
//                 }
//
//                 TracePrint((TRACE_LEVEL_ERROR,
//                             TRACE_FLAG_GENERAL,
//                             "EnqueueFreeTransferPacket: Device (%p), Failed to allocate memory for the work item.\n",
//                             Fdo));
//
//                 CleanupTransferPacketToWorkingSetSize(Fdo, TRUE, allocateNode);
//             }
//         }
//
//         /*
//          *  2.  Lazily work down to our LOWER threshold (by only freeing one packet at a time).
//          */
//         if (fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets >
//             fdoData->LocalMinWorkingSetTransferPackets){
//             /*
//              *  Check the counter again with lock held.  This eliminates a race condition
//              *  while still allowing us to not grab the spinlock in the common codepath.
//              *
//              *  Note that the spinlock does not synchronize with threads dequeuing free
//              *  packets to send (DequeueFreeTransferPacket does that with a lightweight
//              *  interlocked exchange); the spinlock prevents multiple threads in this function
//              *  from deciding to free too many extra packets at once.
//              */
//             PTRANSFER_PACKET pktToDelete = NULL;
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, "Exiting stress, lazily freeing one of %d/%d packets from node %d.",
//                 fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets,
//                 fdoData->LocalMinWorkingSetTransferPackets,
//                 allocateNode));
//
//             KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//             if ((fdoData->FreeTransferPacketsLists[allocateNode].NumFreeTransferPackets >=
//                 fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets) &&
//                 (fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets >
//                 fdoData->LocalMinWorkingSetTransferPackets)){
//
//                 pktToDelete = DequeueFreeTransferPacketEx(Fdo, FALSE, allocateNode);
//                 if (pktToDelete) {
//                     InterlockedDecrement((volatile LONG *)&(fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets));
//                 } else {
//                     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW,
//                         "Extremely unlikely condition (non-fatal): %d packets dequeued at once for Fdo %p. NumTotalTransferPackets=%d (2). Node=%d",
//                         fdoData->LocalMinWorkingSetTransferPackets,
//                         Fdo,
//                         fdoData->FreeTransferPacketsLists[allocateNode].NumTotalTransferPackets,
//                         allocateNode));
//                 }
//             }
//             KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
//
//             if (pktToDelete) {
//                 DestroyTransferPacket(pktToDelete);
//             }
//         }
//
//     }
//
// }
//
// PTRANSFER_PACKET DequeueFreeTransferPacket(PDEVICE_OBJECT Fdo, BOOLEAN AllocIfNeeded)
// {
//     return DequeueFreeTransferPacketEx(Fdo, AllocIfNeeded, KeGetCurrentNodeNumber());
// }
//
// PTRANSFER_PACKET DequeueFreeTransferPacketEx(
//      PDEVICE_OBJECT Fdo,
//      BOOLEAN AllocIfNeeded,
//      ULONG Node)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PTRANSFER_PACKET pkt;
//     PSLIST_ENTRY slistEntry;
//
//     slistEntry = InterlockedPopEntrySList(&(fdoData->FreeTransferPacketsLists[Node].SListHeader));
//
//     if (slistEntry) {
//         slistEntry->Next = NULL;
//         pkt = CONTAINING_RECORD(slistEntry, TRANSFER_PACKET, SlistEntry);
//         InterlockedDecrement((volatile LONG *)&(fdoData->FreeTransferPacketsLists[Node].NumFreeTransferPackets));
//
//         // when dequeuing the packet, also reset the history data
//         HISTORYINITIALIZERETRYLOGS(pkt);
//
//     } else {
//         if (AllocIfNeeded) {
//             /*
//              *  We are in stress and have run out of lookaside packets.
//              *  In order to service the current transfer,
//              *  allocate an extra packet.
//              *  We will free it lazily when we are out of stress.
//              */
//             pkt = NewTransferPacket(Fdo);
//             if (pkt) {
//                 InterlockedIncrement((volatile LONG *)&fdoData->FreeTransferPacketsLists[Node].NumTotalTransferPackets);
//                 fdoData->FreeTransferPacketsLists[Node].DbgPeakNumTransferPackets =
//                     max(fdoData->FreeTransferPacketsLists[Node].DbgPeakNumTransferPackets,
//                         fdoData->FreeTransferPacketsLists[Node].NumTotalTransferPackets);
//             } else {
//                 TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_RW, "DequeueFreeTransferPacket: packet allocation failed"));
//             }
//         } else {
//             pkt = NULL;
//         }
//     }
//
//     return pkt;
// }
//
//
//
// /*
//  *  SetupReadWriteTransferPacket
//  *
//  *        This function is called once to set up the first attempt to send a packet.
//  *        It is not called before a retry, as SRB fields may be modified for the retry.
//  *
//  *      Set up the Srb of the TRANSFER_PACKET for the transfer.
//  *        The Irp is set up in SubmitTransferPacket because it must be reset
//  *        for each packet submission.
//  */
// VOID SetupReadWriteTransferPacket(  PTRANSFER_PACKET Pkt,
//                                     PVOID Buf,
//                                     ULONG Len,
//                                     LARGE_INTEGER DiskLocation,
//                                     PIRP OriginalIrp)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Pkt->Fdo->DeviceExtension;
//     PCOMMON_DEVICE_EXTENSION commonExtension = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PIO_STACK_LOCATION origCurSp = IoGetCurrentIrpStackLocation(OriginalIrp);
//     UCHAR majorFunc = origCurSp->MajorFunction;
//     LARGE_INTEGER logicalBlockAddr;
//     ULONG numTransferBlocks;
//     PCDB pCdb;
//     ULONG srbLength;
//     ULONG timeoutValue = fdoExt->TimeOutValue;
//
//     logicalBlockAddr.QuadPart = Int64ShrlMod32(DiskLocation.QuadPart, fdoExt->SectorShift);
//     numTransferBlocks = Len >> fdoExt->SectorShift;
//
//     /*
//      * This field is useful when debugging, since low-memory conditions are
//      * handled differently for CDROM (which is the only driver using StartIO)
//      */
//     Pkt->DriverUsesStartIO = (commonExtension->DriverExtension->InitData.ClassStartIo != NULL);
//
//     /*
//      *  Slap the constant SRB fields in from our pre-initialized template.
//      *  We'll then only have to fill in the unique fields for this transfer.
//      *  Tell lower drivers to sort the SRBs by the logical block address
//      *  so that disk seeks are minimized.
//      */
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//     SrbSetDataBuffer(Pkt->Srb, Buf);
//     SrbSetDataTransferLength(Pkt->Srb, Len);
//     SrbSetQueueSortKey(Pkt->Srb, logicalBlockAddr.LowPart);
//     if (logicalBlockAddr.QuadPart > 0xFFFFFFFF) {
//         //
//         // If the requested LBA is more than max ULONG set the
//         // QueueSortKey to the maximum value, so that these
//         // requests can be added towards the end of the queue.
//         //
//
//         SrbSetQueueSortKey(Pkt->Srb, 0xFFFFFFFF);
//     }
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//
//
//     SrbSetTimeOutValue(Pkt->Srb, timeoutValue);
//
//     /*
//      *  Arrange values in CDB in big-endian format.
//      */
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         if (TEST_FLAG(fdoExt->DeviceFlags, DEV_USE_16BYTE_CDB)) {
//             REVERSE_BYTES_QUAD(&pCdb->CDB16.LogicalBlock, &logicalBlockAddr);
//             REVERSE_BYTES(&pCdb->CDB16.TransferLength, &numTransferBlocks);
//             pCdb->CDB16.OperationCode = (majorFunc==IRP_MJ_READ) ? SCSIOP_READ16 : SCSIOP_WRITE16;
//             SrbSetCdbLength(Pkt->Srb, 16);
//         } else {
//             pCdb->CDB10.LogicalBlockByte0 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte3;
//             pCdb->CDB10.LogicalBlockByte1 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte2;
//             pCdb->CDB10.LogicalBlockByte2 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte1;
//             pCdb->CDB10.LogicalBlockByte3 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte0;
//             pCdb->CDB10.TransferBlocksMsb = ((PFOUR_BYTE)&numTransferBlocks)->Byte1;
//             pCdb->CDB10.TransferBlocksLsb = ((PFOUR_BYTE)&numTransferBlocks)->Byte0;
//             pCdb->CDB10.OperationCode = (majorFunc==IRP_MJ_READ) ? SCSIOP_READ : SCSIOP_WRITE;
//         }
//     }
//
//     /*
//      *  Set SRB and IRP flags
//      */
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags);
//     if (TEST_FLAG(OriginalIrp->Flags, IRP_PAGING_IO) ||
//         TEST_FLAG(OriginalIrp->Flags, IRP_SYNCHRONOUS_PAGING_IO)){
//         SrbSetSrbFlags(Pkt->Srb, SRB_CLASS_FLAGS_PAGING);
//     }
//     SrbSetSrbFlags(Pkt->Srb, (majorFunc==IRP_MJ_READ) ? SRB_FLAGS_DATA_IN : SRB_FLAGS_DATA_OUT);
//
//     /*
//      *  Allow caching only if this is not a write-through request.
//      *  If write-through and caching is enabled on the device, force
//      *  media access.
//      *  Ignore SL_WRITE_THROUGH for reads; it's only set because the file handle was opened with WRITE_THROUGH.
//      */
//     if ((majorFunc == IRP_MJ_WRITE) && TEST_FLAG(origCurSp->Flags, SL_WRITE_THROUGH) && pCdb) {
//         pCdb->CDB10.ForceUnitAccess = fdoExt->CdbForceUnitAccess;
//     } else {
//         SrbSetSrbFlags(Pkt->Srb, SRB_FLAGS_ADAPTER_CACHE_ENABLE);
//     }
//
//     /*
//      *  Remember the buf and len in the SRB because miniports
//      *  can overwrite SRB.DataTransferLength and we may need it again
//      *  for the retry.
//      */
//     Pkt->BufPtrCopy = Buf;
//     Pkt->BufLenCopy = Len;
//     Pkt->TargetLocationCopy = DiskLocation;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = fdoData->MaxNumberOfIoRetries;
//     Pkt->SyncEventPtr = NULL;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = TRUE;
// #if !defined(__REACTOS__) && NTDDI_VERSION >= NTDDI_WINBLUE
//     Pkt->NumIoTimeoutRetries = fdoData->MaxNumberOfIoRetries;
//     Pkt->NumThinProvisioningRetries = 0;
// #endif
//
//
//     if (pCdb) {
//         DBGLOGFLUSHINFO(fdoData, TRUE, (BOOLEAN)(pCdb->CDB10.ForceUnitAccess), FALSE);
//     } else {
//         DBGLOGFLUSHINFO(fdoData, TRUE, FALSE, FALSE);
//     }
// }
//
//
// /*
//  *  SubmitTransferPacket
//  *
//  *        Set up the IRP for the TRANSFER_PACKET submission and send it down.
//  */
// NTSTATUS SubmitTransferPacket(PTRANSFER_PACKET Pkt)
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = Pkt->Fdo->DeviceExtension;
//     PDEVICE_OBJECT nextDevObj = commonExtension->LowerDeviceObject;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     BOOLEAN idleRequest = FALSE;
//     PIO_STACK_LOCATION nextSp;
//
//     NT_ASSERT(Pkt->Irp->CurrentLocation == Pkt->Irp->StackCount+1 as i32)
//
//     /*
//      *  Attach the SRB to the IRP.
//      *  The reused IRP's stack location has to be rewritten for each retry
//      *  call because IoCompleteRequest clears the stack locations.
//      */
//     IoReuseIrp(Pkt->Irp, STATUS_NOT_SUPPORTED);
//
//
//     nextSp = IoGetNextIrpStackLocation(Pkt->Irp);
//     nextSp->MajorFunction = IRP_MJ_SCSI;
//     nextSp->Parameters.Scsi.Srb = (PSCSI_REQUEST_BLOCK)Pkt->Srb;
//
//     SrbSetScsiStatus(Pkt->Srb, 0);
//     Pkt->Srb->SrbStatus = 0;
//     SrbSetSenseInfoBufferLength(Pkt->Srb, SENSE_BUFFER_SIZE_EX);
//
//     if (Pkt->CompleteOriginalIrpWhenLastPacketCompletes) {
//         /*
//          *  Only dereference the "original IRP"'s stack location
//          *  if its a real client irp (as opposed to a static irp
//          *  we're using just for result status for one of the non-IO scsi commands).
//          *
//          *  For read/write, propagate the storage-specific IRP stack location flags
//          *  (e.g. SL_OVERRIDE_VERIFY_VOLUME, SL_WRITE_THROUGH).
//          */
//         PIO_STACK_LOCATION origCurSp = IoGetCurrentIrpStackLocation(Pkt->OriginalIrp);
//         nextSp->Flags = origCurSp->Flags;
//     }
//
//     //
//     // If the request is not split, we can use the original IRP MDL.  If the
//     // request needs to be split, we need to use a partial MDL.  The partial MDL
//     // is needed because more than one driver might be mapping the same MDL
//     // and this causes problems.
//     //
//     if (Pkt->UsePartialMdl == FALSE) {
//         Pkt->Irp->MdlAddress = Pkt->OriginalIrp->MdlAddress;
//     } else {
//         IoBuildPartialMdl(Pkt->OriginalIrp->MdlAddress, Pkt->PartialMdl, SrbGetDataBuffer(Pkt->Srb), SrbGetDataTransferLength(Pkt->Srb));
//         Pkt->Irp->MdlAddress = Pkt->PartialMdl;
//     }
//
//
//     DBGLOGSENDPACKET(Pkt);
//     HISTORYLOGSENDPACKET(Pkt);
//
//     //
//     // Set the original irp here for SFIO.
//     //
//     ClasspSrbSetOriginalIrp(Pkt->Srb, (PVOID) (Pkt->OriginalIrp));
//
//     //
//     // No need to lock for IdlePrioritySupported, since it will
//     // be modified only at initialization time.
//     //
//     if (fdoData->IdlePrioritySupported == TRUE) {
//         idleRequest = ClasspIsIdleRequest(Pkt->OriginalIrp);
//         if (idleRequest) {
//             InterlockedIncrement(&fdoData->ActiveIdleIoCount);
//         } else {
//             InterlockedIncrement(&fdoData->ActiveIoCount);
//         }
//     }
//
//     IoSetCompletionRoutine(Pkt->Irp, TransferPktComplete, Pkt, TRUE, TRUE, TRUE);
//     return IoCallDriver(nextDevObj, Pkt->Irp);
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// TransferPktComplete(IN PDEVICE_OBJECT NullFdo, IN PIRP Irp, IN PVOID Context)
// {
//     PTRANSFER_PACKET pkt = (PTRANSFER_PACKET)Context;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     BOOLEAN packetDone = FALSE;
//     BOOLEAN idleRequest = FALSE;
//     ULONG transferLength;
//     LARGE_INTEGER completionTime;
//     ULONGLONG lastIoCompletionTime;
//
//     UNREFERENCED_PARAMETER(NullFdo);
//
//     /*
//      *  Put all the assertions and spew in here so we don't have to look at them.
//      */
//     DBGLOGRETURNPACKET(pkt);
//     DBGCHECKRETURNEDPKT(pkt);
//     HISTORYLOGRETURNEDPACKET(pkt);
//
//
//     completionTime = ClasspGetCurrentTime();
//
//     //
//     // Record the time at which the last IO completed while snapping the old
//     // value to be used later. This can occur on multiple threads and hence
//     // could be overwritten with an older value. This is OK because this value
//     // is maintained as a heuristic.
//     //
//
// #ifdef _WIN64
// #ifndef __REACTOS__
//     lastIoCompletionTime = ReadULong64NoFence((volatile ULONG64*)&fdoData->LastIoCompletionTime.QuadPart);
//     WriteULong64NoFence((volatile ULONG64*)&fdoData->LastIoCompletionTime.QuadPart,
//                         completionTime.QuadPart);
// #else
//     lastIoCompletionTime = *(volatile ULONG64*)&fdoData->LastIoCompletionTime.QuadPart;
//     *((volatile ULONG64*)&fdoData->LastIoCompletionTime.QuadPart) = completionTime.QuadPart;
// #endif
// #else
//     lastIoCompletionTime = InterlockedExchangeNoFence64((volatile LONG64*)&fdoData->LastIoCompletionTime.QuadPart,
//                                                         completionTime.QuadPart);
// #endif
//
//     if (fdoData->IdlePrioritySupported == TRUE) {
//         idleRequest = ClasspIsIdleRequest(pkt->OriginalIrp);
//         if (idleRequest) {
//             InterlockedDecrement(&fdoData->ActiveIdleIoCount);
//             NT_ASSERT(fdoData->ActiveIdleIoCount >= 0 as i32)
//         } else {
//             fdoData->LastNonIdleIoTime = completionTime;
//             InterlockedDecrement(&fdoData->ActiveIoCount);
//             NT_ASSERT(fdoData->ActiveIoCount >= 0 as i32)
//         }
//     }
//
//     //
//     // If partial MDL was used, unmap the pages.  When the packet is retried, the
//     // MDL will be recreated.  If the packet is done, the MDL will be ready to be reused.
//     //
//     if (pkt->UsePartialMdl) {
//         MmPrepareMdlForReuse(pkt->PartialMdl);
//     }
//
//     if (SRB_STATUS(pkt->Srb->SrbStatus) == SRB_STATUS_SUCCESS) {
//
//         NT_ASSERT(NT_SUCCESS(Irp->IoStatus.Status) as i32)
//
//         transferLength = SrbGetDataTransferLength(pkt->Srb);
//
//         fdoData->LoggedTURFailureSinceLastIO = FALSE;
//
//         /*
//          *  The port driver should not have allocated a sense buffer
//          *  if the SRB succeeded.
//          */
//         NT_ASSERT(!PORT_ALLOCATED_SENSE_EX(fdoExt, pkt->Srb) as i32)
//
//         /*
//          *  Add this packet's transferred length to the original IRP's.
//          */
//         InterlockedExchangeAdd((PLONG)&pkt->OriginalIrp->IoStatus.Information,
//                               (LONG)transferLength);
//
//
//         if ((pkt->InLowMemRetry) ||
//             (pkt->DriverUsesStartIO && pkt->LowMemRetry_remainingBufLen > 0)) {
//             packetDone = StepLowMemRetry(pkt);
//         } else {
//             packetDone = TRUE;
//         }
//
//     }
//     else {
//         /*
//          *  The packet failed.  We may retry it if possible.
//          */
//         BOOLEAN shouldRetry;
//
//         /*
//          *  Make sure IRP status matches SRB error status (since we propagate it).
//          */
//         if (NT_SUCCESS(Irp->IoStatus.Status)){
//             Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//         }
//
//         /*
//          *  The packet failed.
//          *  So when sending the packet down we either saw either an error or STATUS_PENDING,
//          *  and so we returned STATUS_PENDING for the original IRP.
//          *  So now we must mark the original irp pending to match that, _regardless_ of
//          *  whether we actually switch threads here by retrying.
//          *  (We also have to mark the irp pending if the lower driver marked the irp pending;
//          *   that is dealt with farther down).
//          */
//         if (pkt->CompleteOriginalIrpWhenLastPacketCompletes){
//             IoMarkIrpPending(pkt->OriginalIrp);
//         }
//
//         /*
//          *  Interpret the SRB error (to a meaningful IRP status)
//          *  and determine if we should retry this packet.
//          *  This call looks at the returned SENSE info to figure out what to do.
//          */
//         shouldRetry = InterpretTransferPacketError(pkt);
//
//         /*
//          *  If the SRB queue is locked-up, release it.
//          *  Do this after calling the error handler.
//          */
//         if (pkt->Srb->SrbStatus & SRB_STATUS_QUEUE_FROZEN){
//             ClassReleaseQueue(pkt->Fdo);
//         }
//
//
//         if (NT_SUCCESS(Irp->IoStatus.Status)){
//             /*
//              *  The error was recovered above in the InterpretTransferPacketError() call.
//              */
//
//             NT_ASSERT(!shouldRetry as i32)
//
//             /*
//              *  In the case of a recovered error,
//              *  add the transfer length to the original Irp as we would in the success case.
//              */
//             InterlockedExchangeAdd((PLONG)&pkt->OriginalIrp->IoStatus.Information,
//                                   (LONG)SrbGetDataTransferLength(pkt->Srb));
//
//             if ((pkt->InLowMemRetry) ||
//                 (pkt->DriverUsesStartIO && pkt->LowMemRetry_remainingBufLen > 0)) {
//                 packetDone = StepLowMemRetry(pkt);
//             } else {
//                 packetDone = TRUE;
//             }
//         } else {
//             if (shouldRetry && (pkt->NumRetries > 0)){
//                 packetDone = RetryTransferPacket(pkt);
//             } else if (shouldRetry && (pkt->RetryHistory != NULL)){
//                 // don't limit retries if class driver has custom interpretation routines
//                 packetDone = RetryTransferPacket(pkt);
//             } else {
//                 packetDone = TRUE;
//             }
//         }
//     }
//
//     /*
//      *  If the packet is completed, put it back in the free list.
//      *  If it is the last packet servicing the original request, complete the original irp.
//      */
//     if (packetDone){
//         LONG numPacketsRemaining;
//         PIRP deferredIrp;
//         PDEVICE_OBJECT Fdo = pkt->Fdo;
//         UCHAR uniqueAddr = 0;
//
//         /*
//          *  In case a remove is pending, bump the lock count so we don't get freed
//          *  right after we complete the original irp.
//          */
//         ClassAcquireRemoveLock(Fdo, (PVOID)&uniqueAddr);
//
//
//         /*
//          *  Sometimes the port driver can allocates a new 'sense' buffer
//          *  to report transfer errors, e.g. when the default sense buffer
//          *  is too small.  If so, it is up to us to free it.
//          *  Now that we're done using the sense info, free it if appropriate.
//          *  Then clear the sense buffer so it doesn't pollute future errors returned in this packet.
//          */
//         if (PORT_ALLOCATED_SENSE_EX(fdoExt, pkt->Srb)) {
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, "Freeing port-allocated sense buffer for pkt %ph.", pkt));
//             FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExt, pkt->Srb);
//             SrbSetSenseInfoBuffer(pkt->Srb, &pkt->SrbErrorSenseData);
//             SrbSetSenseInfoBufferLength(pkt->Srb, sizeof(pkt->SrbErrorSenseData));
//         } else {
//             NT_ASSERT(SrbGetSenseInfoBuffer(pkt->Srb) == &pkt->SrbErrorSenseData as i32)
//             NT_ASSERT(SrbGetSenseInfoBufferLength(pkt->Srb) <= sizeof(pkt->SrbErrorSenseData) as i32)
//         }
//
//         RtlZeroMemory(&pkt->SrbErrorSenseData, sizeof(pkt->SrbErrorSenseData));
//
//         /*
//          *  Call IoSetMasterIrpStatus to set appropriate status
//          *  for the Master IRP.
//          */
//         IoSetMasterIrpStatus(pkt->OriginalIrp, Irp->IoStatus.Status);
//
//         if (!NT_SUCCESS(Irp->IoStatus.Status)){
//             /*
//              *  If the original I/O originated in user space (i.e. it is thread-queued),
//              *  and the error is user-correctable (e.g. media is missing, for removable media),
//              *  alert the user.
//              *  Since this is only one of possibly several packets completing for the original IRP,
//              *  we may do this more than once for a single request.  That's ok; this allows
//              *  us to test each returned status with IoIsErrorUserInduced().
//              */
//             if (IoIsErrorUserInduced(Irp->IoStatus.Status) &&
//                 pkt->CompleteOriginalIrpWhenLastPacketCompletes &&
//                 pkt->OriginalIrp->Tail.Overlay.Thread){
//
//                 IoSetHardErrorOrVerifyDevice(pkt->OriginalIrp, Fdo);
//             }
//         }
//
//         /*
//          *  We use a field in the original IRP to count
//          *  down the transfer pieces as they complete.
//          */
//         numPacketsRemaining = InterlockedDecrement(
//             (PLONG)&pkt->OriginalIrp->Tail.Overlay.DriverContext[0]);
//
//         if (numPacketsRemaining > 0){
//             /*
//              *  More transfer pieces remain for the original request.
//              *  Wait for them to complete before completing the original irp.
//              */
//         } else {
//
//             /*
//              *  All the transfer pieces are done.
//              *  Complete the original irp if appropriate.
//              */
//             NT_ASSERT(numPacketsRemaining == 0 as i32)
//             if (pkt->CompleteOriginalIrpWhenLastPacketCompletes){
//
//                 IO_PAGING_PRIORITY priority = (TEST_FLAG(pkt->OriginalIrp->Flags, IRP_PAGING_IO)) ? IoGetPagingIoPriority(pkt->OriginalIrp) : IoPagingPriorityInvalid;
//                 KIRQL oldIrql;
//
//                 if (NT_SUCCESS(pkt->OriginalIrp->IoStatus.Status)){
//                     NT_ASSERT((ULONG)pkt->OriginalIrp->IoStatus.Information ==  IoGetCurrentIrpStackLocation(pkt->OriginalIrp)->Parameters.Read.Length as i32)
//                     ClasspPerfIncrementSuccessfulIo(fdoExt);
//                 }
//                 ClassReleaseRemoveLock(Fdo, pkt->OriginalIrp);
//
//                 /*
//                  *  We submitted all the downward irps, including this last one, on the thread
//                  *  that the OriginalIrp came in on.  So the OriginalIrp is completing on a
//                  *  different thread iff this last downward irp is completing on a different thread.
//                  *  If BlkCache is loaded, for example, it will often complete
//                  *  requests out of the cache on the same thread, therefore not marking the downward
//                  *  irp pending and not requiring us to do so here.  If the downward request is completing
//                  *  on the same thread, then by not marking the OriginalIrp pending we can save an APC
//                  *  and get extra perf benefit out of BlkCache.
//                  *  Note that if the packet ever cycled due to retry or LowMemRetry,
//                  *  we set the pending bit in those codepaths.
//                  */
//                 if (pkt->Irp->PendingReturned){
//                     IoMarkIrpPending(pkt->OriginalIrp);
//                 }
//
//
//                 ClassCompleteRequest(Fdo, pkt->OriginalIrp, IO_DISK_INCREMENT);
//
//                 //
//                 // Drop the count only after completing the request, to give
//                 // Mm some amount of time to issue its next critical request
//                 //
//
//                 if (priority == IoPagingPriorityHigh)
//                 {
//                     KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//
//                     if (fdoData->MaxInterleavedNormalIo < ClassMaxInterleavePerCriticalIo)
//                     {
//                         fdoData->MaxInterleavedNormalIo = 0;
//                     } else {
//                         fdoData->MaxInterleavedNormalIo -= ClassMaxInterleavePerCriticalIo;
//                     }
//
//                     fdoData->NumHighPriorityPagingIo--;
//
//                     if (fdoData->NumHighPriorityPagingIo == 0)
//                     {
//                         LARGE_INTEGER period;
//
//                         //
//                         // Exiting throttle mode
//                         //
//
//                         KeQuerySystemTime(&fdoData->ThrottleStopTime);
//
//                         period.QuadPart = fdoData->ThrottleStopTime.QuadPart - fdoData->ThrottleStartTime.QuadPart;
//                         fdoData->LongestThrottlePeriod.QuadPart = max(fdoData->LongestThrottlePeriod.QuadPart, period.QuadPart);
//                     }
//
//                     KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
//                 }
//
//                 if (idleRequest) {
//                     ClasspCompleteIdleRequest(fdoExt);
//                 }
//
//                 /*
//                  *  We may have been called by one of the class drivers (e.g. cdrom)
//                  *  via the legacy API ClassSplitRequest.
//                  *  This is the only case for which the packet engine is called for an FDO
//                  *  with a StartIo routine; in that case, we have to call IoStartNextPacket
//                  *  now that the original irp has been completed.
//                  */
//                 if (fdoExt->CommonExtension.DriverExtension->InitData.ClassStartIo) {
//                     if (TEST_FLAG(SrbGetSrbFlags(pkt->Srb), SRB_FLAGS_DONT_START_NEXT_PACKET)){
//                         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, "SRB_FLAGS_DONT_START_NEXT_PACKET should never be set here (?)"));
//                     } else {
//                         KeRaiseIrql(DISPATCH_LEVEL, &oldIrql);
//                         IoStartNextPacket(Fdo, TRUE); // yes, some IO is now cancellable
//                         KeLowerIrql(oldIrql);
//                     }
//                 }
//             }
//         }
//
//         /*
//          *  If the packet was synchronous, write the final result back to the issuer's status buffer
//          *  and signal his event.
//          */
//         if (pkt->SyncEventPtr){
//             KeSetEvent(pkt->SyncEventPtr, 0, FALSE);
//             pkt->SyncEventPtr = NULL;
//         }
//
//         /*
//          *  If the operation isn't a normal read/write, but needs to do more
//          *  operation-specific processing, call the operation's continuation
//          *  routine.  The operation may create and queue another transfer packet
//          *  within this routine, but pkt is still freed after returning from the
//          *  continuation routine.
//          */
//         if (pkt->ContinuationRoutine != NULL){
//             pkt->ContinuationRoutine(pkt->ContinuationContext);
//             pkt->ContinuationRoutine = NULL;
//         }
//
//         /*
//          *  Free the completed packet.
//          */
//         pkt->UsePartialMdl = FALSE;
// //        pkt->OriginalIrp = NULL;
//         pkt->InLowMemRetry = FALSE;
//         EnqueueFreeTransferPacket(Fdo, pkt);
//
//         /*
//          *  Now that we have freed some resources,
//          *  try again to send one of the previously deferred irps.
//          */
//         deferredIrp = DequeueDeferredClientIrp(Fdo);
//         if (deferredIrp){
//             ServiceTransferRequest(Fdo, deferredIrp, TRUE);
//         }
//
//         ClassReleaseRemoveLock(Fdo, (PVOID)&uniqueAddr);
//     }
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// /*
//  *  SetupEjectionTransferPacket
//  *
//  *      Set up a transferPacket for a synchronous Ejection Control transfer.
//  */
// VOID SetupEjectionTransferPacket(   TRANSFER_PACKET *Pkt,
//                                         BOOLEAN PreventMediaRemoval,
//                                         PKEVENT SyncEventPtr,
//                                         PIRP OriginalIrp)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//
//     PAGED_CODE();
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetCdbLength(Pkt->Srb, 6);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//     SrbSetTimeOutValue(Pkt->Srb, fdoExt->TimeOutValue);
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         pCdb->MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
//         pCdb->MEDIA_REMOVAL.Prevent = PreventMediaRemoval;
//     }
//
//     Pkt->BufPtrCopy = NULL;
//     Pkt->BufLenCopy = 0;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_LOCKMEDIAREMOVAL_RETRIES;
//     Pkt->SyncEventPtr = SyncEventPtr;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
// }
//
//
// /*
//  *  SetupModeSenseTransferPacket
//  *
//  *      Set up a transferPacket for a synchronous Mode Sense transfer.
//  */
// VOID SetupModeSenseTransferPacket(TRANSFER_PACKET *Pkt,
//                                   PKEVENT SyncEventPtr,
//                                   PVOID ModeSenseBuffer,
//                                   UCHAR ModeSenseBufferLen,
//                                   UCHAR PageMode,
//                                   UCHAR SubPage,
//                                   PIRP OriginalIrp,
//                                   UCHAR PageControl)
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//
//     PAGED_CODE();
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetCdbLength(Pkt->Srb, 6);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//     SrbSetTimeOutValue(Pkt->Srb, fdoExt->TimeOutValue);
//     SrbSetDataBuffer(Pkt->Srb, ModeSenseBuffer);
//     SrbSetDataTransferLength(Pkt->Srb, ModeSenseBufferLen);
//
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         pCdb->MODE_SENSE.OperationCode = SCSIOP_MODE_SENSE;
//         pCdb->MODE_SENSE.PageCode = PageMode;
//         pCdb->MODE_SENSE.SubPageCode = SubPage;
//         pCdb->MODE_SENSE.Pc = PageControl;
//         pCdb->MODE_SENSE.AllocationLength = (UCHAR)ModeSenseBufferLen;
//     }
//
//     Pkt->BufPtrCopy = ModeSenseBuffer;
//     Pkt->BufLenCopy = ModeSenseBufferLen;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_MODESENSE_RETRIES;
//     Pkt->SyncEventPtr = SyncEventPtr;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
// }
//
// /*
//  *  SetupModeSelectTransferPacket
//  *
//  *      Set up a transferPacket for a synchronous Mode Select transfer.
//  */
// VOID SetupModeSelectTransferPacket(TRANSFER_PACKET *Pkt,
//                                   PKEVENT SyncEventPtr,
//                                   PVOID ModeSelectBuffer,
//                                   UCHAR ModeSelectBufferLen,
//                                   BOOLEAN SavePages,
//                                   PIRP OriginalIrp)
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetCdbLength(Pkt->Srb, 6);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//     SrbSetTimeOutValue(Pkt->Srb, fdoExt->TimeOutValue);
//     SrbSetDataBuffer(Pkt->Srb, ModeSelectBuffer);
//     SrbSetDataTransferLength(Pkt->Srb, ModeSelectBufferLen);
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DATA_OUT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         pCdb->MODE_SELECT.OperationCode = SCSIOP_MODE_SELECT;
//         pCdb->MODE_SELECT.SPBit = SavePages;
//         pCdb->MODE_SELECT.PFBit = 1;
//         pCdb->MODE_SELECT.ParameterListLength = (UCHAR)ModeSelectBufferLen;
//     }
//
//     Pkt->BufPtrCopy = ModeSelectBuffer;
//     Pkt->BufLenCopy = ModeSelectBufferLen;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_MODESELECT_RETRIES;
//     Pkt->SyncEventPtr = SyncEventPtr;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
// }
//
//
// /*
//  *  SetupDriveCapacityTransferPacket
//  *
//  *      Set up a transferPacket for a synchronous Drive Capacity transfer.
//  */
// VOID SetupDriveCapacityTransferPacket(   TRANSFER_PACKET *Pkt,
//                                         PVOID ReadCapacityBuffer,
//                                         ULONG ReadCapacityBufferLen,
//                                         PKEVENT SyncEventPtr,
//                                         PIRP OriginalIrp,
//                                         BOOLEAN Use16ByteCdb)
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Pkt->Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//     ULONG timeoutValue = fdoExt->TimeOutValue;
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//
//
//     SrbSetTimeOutValue(Pkt->Srb, timeoutValue);
//     SrbSetDataBuffer(Pkt->Srb, ReadCapacityBuffer);
//     SrbSetDataTransferLength(Pkt->Srb, ReadCapacityBufferLen);
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         if (Use16ByteCdb == TRUE) {
//             NT_ASSERT(ReadCapacityBufferLen >= sizeof(READ_CAPACITY_DATA_EX) as i32)
//             SrbSetCdbLength(Pkt->Srb, 16);
//             pCdb->CDB16.OperationCode = SCSIOP_READ_CAPACITY16;
//             REVERSE_BYTES(&pCdb->CDB16.TransferLength, &ReadCapacityBufferLen);
//             pCdb->AsByte[1] = 0x10; // Service Action
//         } else {
//             SrbSetCdbLength(Pkt->Srb, 10);
//             pCdb->CDB10.OperationCode = SCSIOP_READ_CAPACITY;
//         }
//     }
//
//     Pkt->BufPtrCopy = ReadCapacityBuffer;
//     Pkt->BufLenCopy = ReadCapacityBufferLen;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_DRIVECAPACITY_RETRIES;
// #if !defined(__REACTOS__) && NTDDI_VERSION >= NTDDI_WINBLUE
//     Pkt->NumIoTimeoutRetries = NUM_DRIVECAPACITY_RETRIES;
// #endif
//
//     Pkt->SyncEventPtr = SyncEventPtr;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
// }
//
//
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CleanupTransferPacketToWorkingSetSizeWorker(
//      PVOID Fdo,
//      PVOID Context,
//      PIO_WORKITEM IoWorkItem
//     )
// {
//     ULONG node = (ULONG) (ULONG_PTR)Context;
//
//     PAGED_CODE();
//
//     CleanupTransferPacketToWorkingSetSize((PDEVICE_OBJECT)Fdo, FALSE, node);
//
//     //
//     // Release the remove lock acquired in EnqueueFreeTransferPacket
//     //
//     ClassReleaseRemoveLock((PDEVICE_OBJECT)Fdo, (PIRP)IoWorkItem);
//
//     if (IoWorkItem != NULL) {
//         IoFreeWorkItem(IoWorkItem);
//     }
// }
//
//
// VOID
// CleanupTransferPacketToWorkingSetSize(
//      PDEVICE_OBJECT Fdo,
//      BOOLEAN LimitNumPktToDelete,
//      ULONG Node
//     )
//
// /*
// Routine Description:
//
//     This function frees the resources for the free transfer packets attempting
//     to bring them down within the working set size.
//
// Arguments:
//     Fdo: The FDO that represents the device whose transfer packet size needs to be trimmed.
//     LimitNumPktToDelete: Flag to indicate if the number of packets freed in one call should be capped.
//     Node: NUMA node transfer packet is associated with.
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//     KIRQL oldIrql;
//     SINGLE_LIST_ENTRY pktList;
//     PSINGLE_LIST_ENTRY slistEntry;
//     PTRANSFER_PACKET pktToDelete;
//     ULONG requiredNumPktToDelete = fdoData->FreeTransferPacketsLists[Node].NumTotalTransferPackets -
//                                    fdoData->LocalMaxWorkingSetTransferPackets;
//
//     if (LimitNumPktToDelete) {
//         requiredNumPktToDelete = MIN(requiredNumPktToDelete, MAX_CLEANUP_TRANSFER_PACKETS_AT_ONCE);
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, "CleanupTransferPacketToWorkingSetSize (%p): Exiting stress, block freeing %d packets.", Fdo, requiredNumPktToDelete));
//
//     /*
//      *  Check the counter again with lock held.  This eliminates a race condition
//      *  while still allowing us to not grab the spinlock in the common codepath.
//      *
//      *  Note that the spinlock does not synchronize with threads dequeuing free
//      *  packets to send (DequeueFreeTransferPacket does that with a lightweight
//      *  interlocked exchange); the spinlock prevents multiple threads in this function
//      *  from deciding to free too many extra packets at once.
//      */
//     SimpleInitSlistHdr(&pktList);
//     KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//     while ((fdoData->FreeTransferPacketsLists[Node].NumFreeTransferPackets >= fdoData->FreeTransferPacketsLists[Node].NumTotalTransferPackets) &&
//            (fdoData->FreeTransferPacketsLists[Node].NumTotalTransferPackets > fdoData->LocalMaxWorkingSetTransferPackets) &&
//            (requiredNumPktToDelete--)){
//
//         pktToDelete = DequeueFreeTransferPacketEx(Fdo, FALSE, Node);
//         if (pktToDelete){
//             SimplePushSlist(&pktList,
//                             (PSINGLE_LIST_ENTRY)&pktToDelete->SlistEntry);
//             InterlockedDecrement((volatile LONG *)&fdoData->FreeTransferPacketsLists[Node].NumTotalTransferPackets);
//         } else {
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW,
//                 "Extremely unlikely condition (non-fatal): %d packets dequeued at once for Fdo %p. NumTotalTransferPackets=%d (1). Node=%d",
//                 fdoData->LocalMaxWorkingSetTransferPackets,
//                 Fdo,
//                 fdoData->FreeTransferPacketsLists[Node].NumTotalTransferPackets,
//                 Node));
//             break;
//         }
//     }
//     KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
//
//     slistEntry = SimplePopSlist(&pktList);
//     while (slistEntry) {
//         pktToDelete = CONTAINING_RECORD(slistEntry, TRANSFER_PACKET, SlistEntry);
//         DestroyTransferPacket(pktToDelete);
//         slistEntry = SimplePopSlist(&pktList);
//     }
//
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// _IRQL_requires_min_(PASSIVE_LEVEL)
// _IRQL_requires_same_
// VOID
// ClasspSetupPopulateTokenTransferPacket(
//      __drv_aliasesMem POFFLOAD_READ_CONTEXT OffloadReadContext,
//      PTRANSFER_PACKET Pkt,
//      ULONG Length,
//      PUCHAR PopulateTokenBuffer,
//      PIRP OriginalIrp,
//      ULONG ListIdentifier
//     )
//
// /*++
//
// Routine description:
//
//     This routine is called once to set up a packet for PopulateToken.
//     It builds up the SRB by setting the appropriate fields.
//
// Arguments:
//
//     Pkt - The transfer packet to be sent down to the lower driver
//     SyncEventPtr - The event that gets signaled once the IRP contained in the packet completes
//     Length - Length of the buffer being sent as part of the command
//     PopulateTokenBuffer - The buffer that contains the LBA ranges information for the PopulateToken operation
//     OriginalIrp - The Io request to be processed
//     ListIdentifier - The identifier that will be used to correlate a subsequent command to retrieve the token
//
// Return Value:
//
//     Nothing
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
//     PCLASS_PRIVATE_FDO_DATA fdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupPopulateTokenTransferPacket (%p): Entering function. Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     fdoExt = Pkt->Fdo->DeviceExtension;
//     fdoData = fdoExt->PrivateFdoData;
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetCdbLength(Pkt->Srb, 16);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//     SrbSetTimeOutValue(Pkt->Srb, fdoExt->TimeOutValue);
//     SrbSetDataBuffer(Pkt->Srb, PopulateTokenBuffer);
//     SrbSetDataTransferLength(Pkt->Srb, Length);
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DATA_OUT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         pCdb->TOKEN_OPERATION.OperationCode = SCSIOP_POPULATE_TOKEN;
//         pCdb->TOKEN_OPERATION.ServiceAction = SERVICE_ACTION_POPULATE_TOKEN;
//
//         REVERSE_BYTES(&pCdb->TOKEN_OPERATION.ListIdentifier, &ListIdentifier);
//         REVERSE_BYTES(&pCdb->TOKEN_OPERATION.ParameterListLength, &Length);
//     }
//
//     Pkt->BufPtrCopy = PopulateTokenBuffer;
//     Pkt->BufLenCopy = Length;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_POPULATE_TOKEN_RETRIES;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
//
//     Pkt->ContinuationRoutine = ClasspPopulateTokenTransferPacketDone;
//     Pkt->ContinuationContext = OffloadReadContext;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupPopulateTokenTransferPacket (%p): Exiting function with Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// _IRQL_requires_min_(PASSIVE_LEVEL)
// _IRQL_requires_same_
// VOID
// ClasspSetupReceivePopulateTokenInformationTransferPacket(
//      POFFLOAD_READ_CONTEXT OffloadReadContext,
//      PTRANSFER_PACKET Pkt,
//      ULONG Length,
//      PUCHAR ReceivePopulateTokenInformationBuffer,
//      PIRP OriginalIrp,
//      ULONG ListIdentifier
//     )
//
// /*++
//
// Routine description:
//
//     This routine is called once to set up a packet for read token retrieval.
//     It builds up the SRB by setting the appropriate fields.
//
// Arguments:
//
//     Pkt - The transfer packet to be sent down to the lower driver
//     Length - Length of the buffer being sent as part of the command
//     ReceivePopulateTokenInformationBuffer - The buffer into which the target will pass back the token
//     OriginalIrp - The Io request to be processed
//     ListIdentifier - The identifier that will be used to correlate this command with its corresponding previous populate token operation
//
// Return Value:
//
//     Nothing
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
//     PCLASS_PRIVATE_FDO_DATA fdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupReceivePopulateTokenInformationTransferPacket (%p): Entering function. Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     fdoExt = Pkt->Fdo->DeviceExtension;
//     fdoData = fdoExt->PrivateFdoData;
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetCdbLength(Pkt->Srb, 16);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//     SrbSetTimeOutValue(Pkt->Srb, fdoExt->TimeOutValue);
//     SrbSetDataBuffer(Pkt->Srb, ReceivePopulateTokenInformationBuffer);
//     SrbSetDataTransferLength(Pkt->Srb, Length);
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         pCdb->RECEIVE_TOKEN_INFORMATION.OperationCode = SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION;
//         pCdb->RECEIVE_TOKEN_INFORMATION.ServiceAction = SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION;
//
//         REVERSE_BYTES(&pCdb->RECEIVE_TOKEN_INFORMATION.ListIdentifier, &ListIdentifier);
//         REVERSE_BYTES(&pCdb->RECEIVE_TOKEN_INFORMATION.AllocationLength, &Length);
//     }
//
//     Pkt->BufPtrCopy = ReceivePopulateTokenInformationBuffer;
//     Pkt->BufLenCopy = Length;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_POPULATE_TOKEN_RETRIES;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
//
//     Pkt->ContinuationRoutine = ClasspReceivePopulateTokenInformationTransferPacketDone;
//     Pkt->ContinuationContext = OffloadReadContext;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupReceivePopulateTokenInformationTransferPacket (%p): Exiting function with Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// _IRQL_requires_min_(PASSIVE_LEVEL)
// _IRQL_requires_same_
// VOID
// ClasspSetupWriteUsingTokenTransferPacket(
//      __drv_aliasesMem POFFLOAD_WRITE_CONTEXT OffloadWriteContext,
//      PTRANSFER_PACKET Pkt,
//      ULONG Length,
//      PUCHAR WriteUsingTokenBuffer,
//      PIRP OriginalIrp,
//      ULONG ListIdentifier
//     )
//
// /*++
//
// Routine description:
//
//     This routine is called once to set up a packet for WriteUsingToken.
//     It builds up the SRB by setting the appropriate fields. It is not called
//     before a retry as the SRB fields may be modified for the retry.
//
//     The IRP is set up in SubmitTransferPacket because it must be reset for
//     each packet submission.
//
// Arguments:
//
//     Pkt - The transfer packet to be sent down to the lower driver
//     Length - Length of the buffer being sent as part of the command
//     WriteUsingTokenBuffer - The buffer that contains the read token and the write LBA ranges information for the WriteUsingToken operation
//     OriginalIrp - The Io request to be processed
//     ListIdentifier - The identifier that will be used to correlate a subsequent command to retrieve extended results in case of command failure
//
// Return Value:
//
//     Nothing
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
//     PCLASS_PRIVATE_FDO_DATA fdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupWriteUsingTokenTransferPacket (%p): Entering function. Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     fdoExt = Pkt->Fdo->DeviceExtension;
//     fdoData = fdoExt->PrivateFdoData;
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetCdbLength(Pkt->Srb, 16);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//     SrbSetTimeOutValue(Pkt->Srb, fdoExt->TimeOutValue);
//     SrbSetDataBuffer(Pkt->Srb, WriteUsingTokenBuffer);
//     SrbSetDataTransferLength(Pkt->Srb, Length);
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DATA_OUT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         pCdb->TOKEN_OPERATION.OperationCode = SCSIOP_WRITE_USING_TOKEN;
//         pCdb->TOKEN_OPERATION.ServiceAction = SERVICE_ACTION_WRITE_USING_TOKEN;
//
//         REVERSE_BYTES(&pCdb->TOKEN_OPERATION.ParameterListLength, &Length);
//         REVERSE_BYTES(&pCdb->TOKEN_OPERATION.ListIdentifier, &ListIdentifier);
//     }
//
//     Pkt->BufPtrCopy = WriteUsingTokenBuffer;
//     Pkt->BufLenCopy = Length;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_WRITE_USING_TOKEN_RETRIES;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
//
//     Pkt->ContinuationRoutine = ClasspWriteUsingTokenTransferPacketDone;
//     Pkt->ContinuationContext = OffloadWriteContext;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupWriteUsingTokenTransferPacket (%p): Exiting function with Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// _IRQL_requires_min_(PASSIVE_LEVEL)
// _IRQL_requires_same_
// VOID
// ClasspSetupReceiveWriteUsingTokenInformationTransferPacket(
//      POFFLOAD_WRITE_CONTEXT OffloadWriteContext,
//      PTRANSFER_PACKET Pkt,
//      ULONG Length,
//      PUCHAR ReceiveWriteUsingTokenInformationBuffer,
//      PIRP OriginalIrp,
//      ULONG ListIdentifier
//     )
//
// /*++
//
// Routine description:
//
//     This routine is called once to set up a packet for extended results for
//     WriteUsingToken operation. It builds up the SRB by setting the appropriate fields.
//
// Arguments:
//
//     Pkt - The transfer packet to be sent down to the lower driver
//     SyncEventPtr - The event that gets signaled once the IRP contained in the packet completes
//     Length - Length of the buffer being sent as part of the command
//     ReceiveWriteUsingTokenInformationBuffer - The buffer into which the target will pass back the extended results
//     OriginalIrp - The Io request to be processed
//     ListIdentifier - The identifier that will be used to correlate this command with its corresponding previous write using token operation
//
// Return Value:
//
//     Nothing
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
//     PCLASS_PRIVATE_FDO_DATA fdoData;
//     PCDB pCdb;
//     ULONG srbLength;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupReceiveWriteUsingTokenInformationTransferPacket (%p): Entering function. Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     fdoExt = Pkt->Fdo->DeviceExtension;
//     fdoData = fdoExt->PrivateFdoData;
//
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbLength = ((PSTORAGE_REQUEST_BLOCK) fdoData->SrbTemplate)->SrbLength;
//         NT_ASSERT(((PSTORAGE_REQUEST_BLOCK) Pkt->Srb)->SrbLength >= srbLength as i32)
//     } else {
//         srbLength = fdoData->SrbTemplate->Length;
//     }
//     RtlCopyMemory(Pkt->Srb, fdoData->SrbTemplate, srbLength); // copies _contents_ of SRB blocks
//
//     SrbSetRequestAttribute(Pkt->Srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbSetCdbLength(Pkt->Srb, 16);
//     SrbSetOriginalRequest(Pkt->Srb, Pkt->Irp);
//     SrbSetSenseInfoBuffer(Pkt->Srb, &Pkt->SrbErrorSenseData);
//     SrbSetSenseInfoBufferLength(Pkt->Srb, sizeof(Pkt->SrbErrorSenseData));
//     SrbSetTimeOutValue(Pkt->Srb, fdoExt->TimeOutValue);
//     SrbSetDataBuffer(Pkt->Srb, ReceiveWriteUsingTokenInformationBuffer);
//     SrbSetDataTransferLength(Pkt->Srb, Length);
//
//     SrbAssignSrbFlags(Pkt->Srb, fdoExt->SrbFlags | SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     pCdb = SrbGetCdb(Pkt->Srb);
//     if (pCdb) {
//         pCdb->RECEIVE_TOKEN_INFORMATION.OperationCode = SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION;
//         pCdb->RECEIVE_TOKEN_INFORMATION.ServiceAction = SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION;
//
//         REVERSE_BYTES(&pCdb->RECEIVE_TOKEN_INFORMATION.AllocationLength, &Length);
//         REVERSE_BYTES(&pCdb->RECEIVE_TOKEN_INFORMATION.ListIdentifier, &ListIdentifier);
//     }
//
//     Pkt->BufPtrCopy = ReceiveWriteUsingTokenInformationBuffer;
//     Pkt->BufLenCopy = Length;
//
//     Pkt->OriginalIrp = OriginalIrp;
//     Pkt->NumRetries = NUM_WRITE_USING_TOKEN_RETRIES;
//     Pkt->CompleteOriginalIrpWhenLastPacketCompletes = FALSE;
//
//     Pkt->ContinuationRoutine = (PCONTINUATION_ROUTINE) ClasspReceiveWriteUsingTokenInformationTransferPacketDone;
//     Pkt->ContinuationContext = OffloadWriteContext;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspSetupReceiveWriteUsingTokenInformationTransferPacket (%p): Exiting function with Irp %p\n",
//                 Pkt->Fdo,
//                 OriginalIrp));
//
//     return;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
