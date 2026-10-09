use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAllocateController, IoAllocateMdl, IoFreeController, IoFreeMdl, IoReleaseCancelSpinLock, KeAcquireInterruptSpinLock, KeInsertQueueDpc, KeReleaseInterruptSpinLock, KeWaitForSingleObject, MmBuildMdlForNonPagedPool, MmMapLockedPagesWithReservedMapping, MmUnmapReservedMapping, PDEVICE_OBJECT, PDMA_ADAPTER, PDMA_OPERATIONS, PIRP, PMDL, POWER_STATE, PSCATTER_GATHER_LIST};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// C: a zero-initialized global. c2rust spelled the zero through amd64's SLIST_HEADER layout
// (Alignment/Region), which i386 and arm64 do not share; zero is zero on all three.
#[no_mangle]
pub static mut AtapCompletionQueueList: SLIST_HEADER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut AtapCompletionDpc: KDPC = _KDPC {
    Type: 0,
    Importance: 0,
    Number: 0,
    DpcListEntry: _LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    DeferredRoutine: None,
    DeferredContext: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SystemArgument1: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SystemArgument2: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    DpcData: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
};


unsafe extern "C" fn AtaSrbStatusToNtStatus(mut SrbStatus: u8) -> i32 {
    match (SRB_STATUS!(SrbStatus)) as u32 {
        SRB_STATUS_SUCCESS => return STATUS_SUCCESS,
        SRB_STATUS_TIMEOUT => return STATUS_IO_TIMEOUT,
        SRB_STATUS_NO_DEVICE => return STATUS_DEVICE_DOES_NOT_EXIST,
        SRB_STATUS_DATA_OVERRUN => return STATUS_BUFFER_OVERFLOW,
        SRB_STATUS_SELECTION_TIMEOUT => return STATUS_DEVICE_NOT_CONNECTED,
        _ => {}
    }
    return STATUS_IO_DEVICE_ERROR;
}

unsafe extern "C" fn AtaDeviceCheckPowerState(Device: PATAPORT_IO_CONTEXT) {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut PowerState: POWER_STATE = POWER_STATE { SystemState: 0 };

    if (*Device).QueueFlags & (QUEUE_FLAG_FROZEN_POWER as i32) == 0 {
        return;
    }

    DevExt = CONTAINING_RECORD!(Device, ATAPORT_DEVICE_EXTENSION, Device);

    if (*DevExt).Common.DevicePowerState as u32 == PowerDeviceD0 as u32 {
        return;
    }

    if (*DevExt).Common.SystemPowerState as u32 != PowerSystemWorking as u32 {
        return;
    }

    INFO!(b"Powering up idle device\n\0".as_ptr() as *const i8);

    PowerState.DeviceState = (PowerDeviceD0) as _;
    Status = PoRequestPowerIrp(
        (*DevExt).Common.Self_,
        IRP_MN_SET_POWER,
        PowerState,
        None,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS!(Status) {
        ERR!(b"Failed to power up device '%s' %lx\n\0".as_ptr() as *const i8, (*DevExt).FriendlyName, Status);
    }
}

unsafe extern "C" fn AtaDeviceQueueEmptyEvent(Device: PATAPORT_IO_CONTEXT) {
    let mut Srb: PSCSI_REQUEST_BLOCK;

    KeSetEvent(
        &raw mut (*Device).QueueStoppedEvent,
        0,
        FALSE,
    );

    Srb = (*Device).QuiescenceSrb;
    if !Srb.is_null() {
        let Irp: PIRP = ((*Srb).OriginalRequest).cast();

        (*Device).QuiescenceSrb = core::ptr::null_mut();

        KeReleaseSpinLockFromDpcLevel((&raw mut (*Device).QueueLock).cast());

        (*Srb).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, (IO_DISK_INCREMENT) as CCHAR);

        KeAcquireSpinLockAtDpcLevel((&raw mut (*Device).QueueLock).cast());
    }
}

unsafe extern "C" fn AtaReqRequeueRequest(
    mut Device: PATAPORT_IO_CONTEXT,
    mut Request: PATA_DEVICE_REQUEST,
    mut ResultSrbStatus: PUCHAR,
) -> NTSTATUS {
    let mut SrbStatus: UCHAR = 0;
    SrbStatus = (SRB_STATUS!(*ResultSrbStatus)) as u8;
    if (SrbStatus as i32) as u32 != SRB_STATUS_BUSY && (SrbStatus as i32) as u32 != SRB_STATUS_INSUFFICIENT_RESOURCES {
        return (COMPLETE_IRP) as i32;
    }
    if (SrbStatus as i32) as u32 == SRB_STATUS_INSUFFICIENT_RESOURCES {
        if SRB_GET_FLAGS!((*Request).Srb) & (SRB_FLAG_LOW_MEM_RETRY) as ULONG_PTR != 0 {
            return (COMPLETE_IRP) as i32;
        }
        SRB_SET_FLAGS!((*Request).Srb, SRB_FLAG_LOW_MEM_RETRY);
    }
    if AtaReqDeviceQueueAddSrb(Device, (*Request).Srb) != STATUS_PENDING {
        *ResultSrbStatus = (*(*Request).Srb).SrbStatus as UCHAR;
        return (COMPLETE_IRP) as i32;
    }
    return (COMPLETE_NO_IRP) as i32;
}

unsafe extern "C" fn AtaReqPortQueueListDispatchNextRequest(
    PortData: PATAPORT_PORT_DATA,
) -> BOOLEAN {
    let mut Device: PATAPORT_IO_CONTEXT = core::ptr::null_mut();
    let mut Request: PATA_DEVICE_REQUEST = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();

    if IsListEmpty(&raw mut (*PortData).PortQueueList) != 0 {
        return FALSE;
    }

    Entry = RemoveHeadList(&raw mut (*PortData).PortQueueList);

    Request = CONTAINING_RECORD!(Entry, ATA_DEVICE_REQUEST, __bindgen_anon_1.PortEntry);
    ASSERT_REQUEST!(Request);

    Device = (*Request).Device as PATAPORT_IO_CONTEXT;

    (*Device).QueueFlags &= (!QUEUE_FLAG_FROZEN_SLOT) as i32;
    AtaReqDispatchRequest(Device, Request, FALSE);

    return TRUE;
}

unsafe extern "C" fn AtaReqReleaseResources(
    Device: PATAPORT_IO_CONTEXT,
    Request: PATA_DEVICE_REQUEST,
) {
    if (*Request).Flags & REQUEST_FLAG_HAS_SG_LIST != 0 {
        let DmaAdapter: PDMA_ADAPTER = (*(*Device).PortData).DmaAdapter;
        let DmaOperations: PDMA_OPERATIONS = (*DmaAdapter).DmaOperations as PDMA_OPERATIONS;
        (((*DmaOperations).PutScatterGatherList).unwrap_unchecked())(
            DmaAdapter,
            (*Request).SgList,
            ((*Request).Flags & REQUEST_FLAG_DATA_IN != 0) as i32 as u8,
        );
    } else if (*Request).Flags & REQUEST_FLAG_HAS_RESERVED_MAPPING != 0 {
        MmUnmapReservedMapping(
            (*(*Device).PortData).ReservedVaSpace,
            ATAPORT_TAG,
            (*Request).Mdl,
        );
        _InterlockedExchange(
            &raw mut (*(*Device).PortData).ReservedMappingLock,
            0,
        );
    }

    if (*Request).Flags & REQUEST_FLAG_HAS_MDL != 0 {
        IoFreeMdl((*Request).Mdl);
    }

    (*Request).Mdl = core::ptr::null_mut();
    (*Request).SgList = core::ptr::null_mut();

    (*Request).Flags &= !(REQUEST_FLAG_HAS_SG_LIST | REQUEST_FLAG_HAS_MDL | REQUEST_FLAG_HAS_RESERVED_MAPPING);
}

unsafe extern "C" fn AtaReqCompleteRequest(mut Request: PATA_DEVICE_REQUEST) {
    let mut Device: PATAPORT_IO_CONTEXT = (*Request).Device as PATAPORT_IO_CONTEXT;
    let mut Srb: PSCSI_REQUEST_BLOCK;
    let mut Irp: PIRP;
    let mut SrbStatus: UCHAR;
    let mut CompletionAction: ATA_COMPLETION_ACTION = COMPLETE_IRP;

    if ((*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState) as u32 != REQUEST_STATE_NOT_STARTED {
        if (*Request).Complete.is_some() {
            if (*Request).Flags & REQUEST_FLAG_INTERNAL != 0 {
                (*Request).Complete.unwrap_unchecked()(Request);
                return;
            } else {
                KeAcquireSpinLockAtDpcLevel((&raw mut (*Device).QueueLock).cast());
                CompletionAction = (*Request).Complete.unwrap_unchecked()(Request);
                KeReleaseSpinLockFromDpcLevel((&raw mut (*Device).QueueLock).cast());
            }
        }
        if (*(*Device).PortData).PortFlags & PORT_FLAG_IS_SIMPLEX != 0 {
            IoFreeController((*(*Device).PortData).HwSyncObject);
        }
    }

    AtaReqReleaseResources(Device, Request);

    Srb = (*Request).Srb;
    Irp = (*Request).Irp;
    (*Irp).IoStatus.Information = (*Request).DataTransferLength as ULONG_PTR;

    KeAcquireSpinLockAtDpcLevel((&raw mut (*Device).QueueLock).cast());

    (*Device).FreeRequestsBitmap |= (*Request).Tag;

    if ((*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState) as u32 != REQUEST_STATE_NOT_STARTED {
        let mut PortData: PATAPORT_PORT_DATA = (*Device).PortData;
        KeAcquireSpinLockAtDpcLevel((&raw mut (*PortData).QueueLock).cast());

        (*PortData).ActiveTimersBitmap &= !(1 << (*Request).Slot);

        if (*Request).Flags & REQUEST_EXCLUSIVE_ACCESS_FLAGS != 0 {
            (*PortData).QueueFlags &= !PORT_QUEUE_FLAG_EXCLUSIVE_MODE;
        }

        (*PortData).FreeSlotsBitmap |= 1 << (*Request).Slot;
        (*PortData).Slots[(*Request).Slot as usize] = core::ptr::null_mut();
        if (*Request).Flags & REQUEST_FLAG_NCQ != 0 {
            (*PortData).AllocatedSlots -= 1;
        } else {
            (*PortData).AllocatedSlots += 1;
        }

        (*PortData).AllocateSlot.unwrap_unchecked()(
            (*PortData).ChannelContext,
            Request,
            FALSE,
        );

        if (*PortData).QueueFlags & PORT_QUEUE_FLAG_SIGNAL_STOP != 0 && AtaPortQueueEmpty(PortData) != 0 {
            (*PortData).QueueFlags &= !PORT_QUEUE_FLAG_SIGNAL_STOP;
            KeSetEvent(
                &raw mut (*PortData).QueueStoppedEvent,
                0,
                FALSE,
            );
        }

        if AtaReqPortQueueListDispatchNextRequest(PortData) == 0 {
            KeReleaseSpinLockFromDpcLevel((&raw mut (*PortData).QueueLock).cast());
        }
    }

    if ((*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState) as u32 == REQUEST_STATE_NOT_STARTED || ((*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState) as u32 == REQUEST_STATE_REQUEUE {
        CompletionAction = (AtaReqRequeueRequest(Device, Request, &raw mut (*Request).SrbStatus)) as u32;
    }

    if CompletionAction == COMPLETE_IRP {
        (*Device).Statistics.RequestsCompleted = (*Device).Statistics.RequestsCompleted.wrapping_add(1);
    }

    if ((*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState) as u32 == REQUEST_STATE_FREEZE_QUEUE && (*Srb).SrbFlags & SRB_FLAGS_NO_QUEUE_FREEZE == 0 {
        (*Device).QueueFlags |= (QUEUE_FLAG_FROZEN_QUEUE_FREEZE) as i32;
        (*Request).SrbStatus |= (SRB_STATUS_QUEUE_FROZEN) as u8;
        INFO!("Freeze queue\n");
    }

    if (*Device).QueueFlags & (QUEUE_FLAG_SIGNAL_STOP as i32) != 0 && (*Device).FreeRequestsBitmap == (*Device).MaxRequestsBitmap {
        (*Device).QueueFlags &= (!QUEUE_FLAG_SIGNAL_STOP) as i32;
        AtaDeviceQueueEmptyEvent(Device);
    }

    SrbStatus = (*Request).SrbStatus;

    if AtaReqDeviceQueueDispatchNextRequest(Device) == 0 {
        KeReleaseSpinLockFromDpcLevel((&raw mut (*Device).QueueLock).cast());
    }

    if CompletionAction == COMPLETE_IRP {
        if SRB_STATUS!(SrbStatus) == (SRB_STATUS_INSUFFICIENT_RESOURCES as i32) {
            (*Srb).SrbStatus = (SRB_STATUS_INTERNAL_ERROR) as u8;
            (*Srb).__bindgen_anon_1.InternalStatus = (STATUS_INSUFFICIENT_RESOURCES) as u32;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            (*Srb).SrbStatus = SrbStatus;
            (*Irp).IoStatus.__bindgen_anon_1.Status = AtaSrbStatusToNtStatus(SrbStatus);
        }

        IoCompleteRequest(Irp, (IO_DISK_INCREMENT) as CCHAR);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqSetFixedSenseData(
    Srb: PSCSI_REQUEST_BLOCK,
    SenseCode: SCSI_SENSE_CODE,
) -> UCHAR {
    let mut SenseData: SENSE_DATA = core::mem::zeroed();

    if (*Srb).SenseInfoBuffer.is_null()
        || (*Srb).SenseInfoBufferLength as i32 == 0
    {
        return SenseCode.SrbStatus;
    }

    (*Srb).ScsiStatus = (SCSISTAT_CHECK_CONDITION) as u8;

    RtlZeroMemory!(
        &mut SenseData as *mut SENSE_DATA as *mut core::ffi::c_void,
        core::mem::size_of::<SENSE_DATA>(),
    );
    // Named setters: the hand-placed `_bitfield_1.set(0, 1, ..)` put Valid in bit 0 and the
    // error code in bits 1-7, where SENSE_DATA has ErrorCode:7 then Valid:1.
    SenseData.set_Valid(1);
    SenseData.set_ErrorCode(SCSI_SENSE_ERRORCODE_FIXED_CURRENT as UCHAR);
    SenseData.set_SenseKey(SenseCode.SenseKey as UCHAR);
    SenseData.AdditionalSenseCode = SenseCode.AdditionalSenseCode;
    SenseData.AdditionalSenseCodeQualifier = SenseCode.AdditionalSenseCodeQualifier;
    SenseData.AdditionalSenseLength = (core::mem::size_of::<SENSE_DATA>()
        - RTL_SIZEOF_THROUGH_FIELD!(SENSE_DATA, AdditionalSenseLength))
        as UCHAR;

    RtlCopyMemory!(
        (*Srb).SenseInfoBuffer,
        &SenseData as *const SENSE_DATA as *const core::ffi::c_void,
        min(
            (*Srb).SenseInfoBufferLength as usize,
            core::mem::size_of::<SENSE_DATA>(),
        ),
    );

    return SenseCode.SrbStatus | (SRB_STATUS_AUTOSENSE_VALID) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqSetLbaInformation(
    Srb: PSCSI_REQUEST_BLOCK,
    Lba: u64,
) {
    let mut SenseData: PSENSE_DATA;

    if ((*Srb).SenseInfoBuffer.is_null())
        || ((*Srb).SenseInfoBufferLength == 0)
    {
        return;
    }

    SenseData = ((*Srb).SenseInfoBuffer).cast();

    if ((RTL_CONTAINS_FIELD!(SenseData, (*Srb).SenseInfoBufferLength, Information)) != 0) {
        (*SenseData).Information[0] = (Lba >> 24) as u8;
        (*SenseData).Information[1] = (Lba >> 16) as u8;
        (*SenseData).Information[2] = (Lba >> 8) as u8;
        (*SenseData).Information[3] = (Lba >> 0) as u8;
    }

    // TODO: D_SENSE (fixed or descriptor format) is not supported yet
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqAllocateMdl(Request: PATA_DEVICE_REQUEST) -> BOOLEAN {
    let mut Mdl: PMDL = core::ptr::null_mut();

    Mdl = IoAllocateMdl(
        (*Request).DataBuffer,
        (*Request).DataTransferLength,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    );
    if Mdl.is_null() {
        return FALSE;
    }

    MmBuildMdlForNonPagedPool(Mdl);

    (*Request).Mdl = Mdl;
    (*Request).Flags |= REQUEST_FLAG_HAS_MDL;

    return TRUE;
}

unsafe extern "C" fn AtaReqStartIo(Request: PATA_DEVICE_REQUEST) {
    let mut Device: PATAPORT_IO_CONTEXT = (*Request).Device as PATAPORT_IO_CONTEXT;
    let mut PortData: PATAPORT_PORT_DATA = (*Device).PortData as PATAPORT_PORT_DATA;
    let mut OldIrql: KIRQL = 0;

    (*PortData).PrepareIo.unwrap_unchecked()((*PortData).ChannelContext, Request);

    OldIrql = KeAcquireInterruptSpinLock((*PortData).InterruptObject);

    if (*PortData).InterruptFlags & PORT_INT_FLAG_IS_IO_ACTIVE as LONG != 0
        || (*Request).Flags & REQUEST_FLAG_INTERNAL as ULONG != 0
    {
        (*PortData).ActiveSlotsBitmap |= 1 << (*Request).Slot;

        if (*PortData).StartIo.unwrap_unchecked()(
            (*PortData).ChannelContext,
            Request,
        ) == 0
        {
            (*PortData).TimerCount[(*Request).Slot as usize] = (*Request).TimeOut as LONG;
        }

        KeReleaseInterruptSpinLock((*PortData).InterruptObject, OldIrql);
    } else {
        let mut Device_0: PATAPORT_IO_CONTEXT = (*Request).Device as PATAPORT_IO_CONTEXT;

        _InterlockedOr(
            &raw mut (*Device_0).QueueFlags as *mut ::core::ffi::c_long,
            QUEUE_FLAG_FROZEN_PORT_BUSY as ::core::ffi::c_long,
        );

        KeReleaseInterruptSpinLock((*PortData).InterruptObject, OldIrql);

        (*Request).SrbStatus = SRB_STATUS_BUSY as UCHAR;
        (*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState = REQUEST_STATE_REQUEUE as UCHAR;

        AtaReqStartCompletionDpc(Request);
    }
}

unsafe extern "system" fn AtaReqPreparePrdTable(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, SgList: PSCATTER_GATHER_LIST, Context: PVOID) {
    let Request: PATA_DEVICE_REQUEST = Context as PATA_DEVICE_REQUEST;
    let Device: PATAPORT_IO_CONTEXT = (*Request).Device as PATAPORT_IO_CONTEXT;
    let PortData: PATAPORT_PORT_DATA = (*Device).PortData as PATAPORT_PORT_DATA;

    UNREFERENCED_PARAMETER!(DeviceObject);
    UNREFERENCED_PARAMETER!(Irp);

    (*Request).SgList = SgList;
    (*Request).Flags |= REQUEST_FLAG_HAS_SG_LIST;

    (((*PortData).PreparePrdTable).unwrap_unchecked())((*PortData).ChannelContext, Request, SgList);
    AtaReqStartIo(Request);
}

unsafe extern "C" fn AtaReqMapBuffer(
    Device: PATAPORT_IO_CONTEXT,
    Request: PATA_DEVICE_REQUEST,
) -> PVOID {
    let mut PortData: PATAPORT_PORT_DATA;
    let mut BaseAddress: PVOID;
    let mut PagesNeeded: ULONG;

    BaseAddress = MmGetSystemAddressForMdlSafe((*Request).Mdl, HighPagePriority);
    if !BaseAddress.is_null() {
        return BaseAddress;
    }

    PortData = (*Device).PortData;
    if (*PortData).ReservedVaSpace.is_null() {
        return core::ptr::null_mut();
    }

    PagesNeeded = (ADDRESS_AND_SIZE_TO_SPAN_PAGES(
        MmGetMdlVirtualAddress((*Request).Mdl),
        (MmGetMdlByteCount((*Request).Mdl)) as _,
    )) as _;
    if PagesNeeded > ATA_RESERVED_PAGES {
        return core::ptr::null_mut();
    }

    if _InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*PortData).ReservedMappingLock),
        1,
        0,
    ) == 0
    {
        BaseAddress = MmMapLockedPagesWithReservedMapping(
            (*PortData).ReservedVaSpace,
            ATAPORT_TAG,
            (*Request).Mdl,
            MmCached,
        );
        if !BaseAddress.is_null() {
            (*Request).Flags |= REQUEST_FLAG_HAS_RESERVED_MAPPING;
        } else {
            _InterlockedExchange(
                core::ptr::addr_of_mut!((*PortData).ReservedMappingLock),
                0,
            );
        }
    }

    BaseAddress
}

unsafe extern "C" fn AtaReqGetScatterGatherList(
    PortData: PATAPORT_PORT_DATA,
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    let DmaAdapter: PDMA_ADAPTER = (*PortData).DmaAdapter;
    let DmaOperations: PDMA_OPERATIONS = (*DmaAdapter).DmaOperations;
    let Status: NTSTATUS;

    Status = (*DmaOperations).GetScatterGatherList.unwrap_unchecked()(
        DmaAdapter,
        (*PortData).ChannelObject,
        (*Request).Mdl,
        (*Request).DataBuffer,
        (*Request).DataTransferLength,
        Some(
            AtaReqPreparePrdTable,
        ),
        Request as PVOID,
        ((*Request).Flags & REQUEST_FLAG_DATA_IN as ULONG != 0) as BOOLEAN,
    );
    if NT_SUCCESS!(Status) {
        return TRUE;
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqSendRequest(Request: PATA_DEVICE_REQUEST) {
    let Device: PATAPORT_IO_CONTEXT = (*Request).Device as PATAPORT_IO_CONTEXT;
    let mut BaseAddress: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Offset: ULONG_PTR = 0;

    if ((*Device).DeviceFlags as ULONG & DEVICE_PIO_VIA_DMA as ULONG) != 0 {
        (*Request).Flags |= REQUEST_FLAG_PROGRAM_DMA as ULONG;
    }

    if ((*Request).Flags as ULONG & REQUEST_FLAG_DMA as ULONG) != 0 {
        ASSERT!(((*Request).Flags as ULONG & REQUEST_FLAG_PROGRAM_DMA as ULONG));
    }

    if ((*Request).Flags as ULONG & REQUEST_FLAG_NO_KEEP_AWAKE as ULONG) == 0 {
        let PowerIdleCounter: PULONG = (*Device).PowerIdleCounter;
        if !PowerIdleCounter.is_null() {
            PoSetDeviceBusy(PowerIdleCounter);
        }
    }

    'CompleteNoMemory: loop {
        if ((*Request).Flags as ULONG & REQUEST_FLAG_HAS_LOCAL_BUFFER as ULONG) != 0 {
            if ((*Request).Flags as ULONG & REQUEST_FLAG_PROGRAM_DMA as ULONG) != 0 {
                (*(*(*Device).PortData).LocalSgList.Elements.as_mut_ptr()
                    .add(0 as usize))
                    .Length = (*Request).DataTransferLength;
                AtaReqPreparePrdTable(
                    core::ptr::null_mut::<core::ffi::c_void>() as PDEVICE_OBJECT,
                    core::ptr::null_mut::<core::ffi::c_void>() as PIRP,
                    core::ptr::addr_of_mut!((*(*Device).PortData).LocalSgList),
                    Request as PVOID,
                );
            } else {
                (*Request).DataBuffer = (*Device).LocalBuffer;
                AtaReqStartIo(Request);
            }
            return;
        }

        if ((*Request).Flags as ULONG & (REQUEST_FLAG_DATA_IN as ULONG | REQUEST_FLAG_DATA_OUT as ULONG)) == 0 {
            AtaReqStartIo(Request);
            return;
        }

        if ((*Request).Flags as ULONG & REQUEST_FLAG_PROGRAM_DMA as ULONG) != 0 {
            if AtaReqGetScatterGatherList((*Device).PortData, Request) != 0 {
                return;
            }
            if ((*Device).DeviceFlags as ULONG & DEVICE_PIO_VIA_DMA as ULONG) != 0 {
                break 'CompleteNoMemory;
            }
            if AtaReqDmaTransferToPioTransfer(Request) == 0 {
                break 'CompleteNoMemory;
            }
        }

        BaseAddress = AtaReqMapBuffer(Device, Request);
        if BaseAddress.is_null() {
            break 'CompleteNoMemory;
        }

        Offset = (BaseAddress as ULONG_PTR)
            .wrapping_add((*Request).DataBuffer as ULONG_PTR)
            .wrapping_sub(MmGetMdlVirtualAddress((*Request).Mdl) as ULONG_PTR);
        (*Request).DataBuffer = Offset as PVOID;

        AtaReqStartIo(Request);
        return;
    }

    (*Request).SrbStatus = SRB_STATUS_INSUFFICIENT_RESOURCES as UCHAR;
    (*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState = REQUEST_STATE_REQUEUE as UCHAR;
    AtaReqStartCompletionDpc(Request);
}

unsafe extern "system" fn AtaReqCallSendRequestSerialized(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, MapRegisterBase: PVOID, Context: PVOID) -> IO_ALLOCATION_ACTION {
    let Request: PATA_DEVICE_REQUEST = Context as PATA_DEVICE_REQUEST;

    UNREFERENCED_PARAMETER!(DeviceObject);
    UNREFERENCED_PARAMETER!(Irp);
    UNREFERENCED_PARAMETER!(MapRegisterBase);

    AtaReqSendRequest(Request);
    return KeepObject;
}

#[no_mangle]
pub unsafe extern "system" fn AtaReqCompletionDpc(Dpc: *mut _KDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID) {
    let mut CurrentEntry: PSLIST_ENTRY;
    let mut NextEntry: PSLIST_ENTRY;

    UNREFERENCED_PARAMETER!(Dpc);
    UNREFERENCED_PARAMETER!(DeferredContext);
    UNREFERENCED_PARAMETER!(SystemArgument1);
    UNREFERENCED_PARAMETER!(SystemArgument2);

    CurrentEntry = ExInterlockedFlushSList(&mut AtapCompletionQueueList);
    while !CurrentEntry.is_null() {
        let Request: *mut ATA_DEVICE_REQUEST;

        NextEntry = (*CurrentEntry).Next;

        Request = CONTAINING_RECORD!(CurrentEntry, ATA_DEVICE_REQUEST, __bindgen_anon_1.CompletionEntry);
        ASSERT_REQUEST!(Request);

        AtaReqCompleteRequest(Request);

        CurrentEntry = NextEntry;
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqStartCompletionDpc(Request: PATA_DEVICE_REQUEST) {
    ASSERT_REQUEST!(Request);

    InterlockedPushEntrySList((&raw mut AtapCompletionQueueList).cast(), (&raw mut (*Request).__bindgen_anon_1.CompletionEntry).cast());
    KeInsertQueueDpc(&mut AtapCompletionDpc, core::ptr::null_mut(), core::ptr::null_mut());
}

unsafe extern "C" fn AtaReqAllocateSlot(
    Device: PATAPORT_IO_CONTEXT,
    PortData: PATAPORT_PORT_DATA,
    Request: PATA_DEVICE_REQUEST,
) -> BOOLEAN {
    let mut SlotMask: ULONG;
    let mut SlotNumber: ULONG = 0;
    let mut SlotsBitmap: ULONG;

    if (*PortData).QueueFlags & PORT_QUEUE_FLAG_EXCLUSIVE_MODE != 0 {
        return FALSE;
    }

    if (*Request).Flags & REQUEST_EXCLUSIVE_ACCESS_FLAGS != 0 {
        if (*PortData).AllocatedSlots != 0 {
            return FALSE;
        }
    }

    if (*Request).Flags & REQUEST_FLAG_NCQ != 0 {
        let mut QueueDepth: ULONG;

        if (*PortData).AllocatedSlots < 0 {
            return FALSE;
        }

        QueueDepth = (*Device).__bindgen_anon_1.TransportFlags & DEVICE_QUEUE_DEPTH_MASK;
        QueueDepth >>= DEVICE_QUEUE_DEPTH_SHIFT;

        if ((*PortData).AllocatedSlots) as u32 >= QueueDepth {
            return FALSE;
        }
    } else {
        if (*PortData).AllocatedSlots > 0 {
            return FALSE;
        }
    }

    SlotsBitmap = (*PortData).FreeSlotsBitmap;

    SlotMask = !(0xFFFFFFFFu32 >> (MAX_SLOTS as u32 - (((*PortData).LastUsedSlot + 1)) as u32));
    if SlotsBitmap & SlotMask == 0 {
        SlotMask = 0xFFFFFFFF;
    }

    if _BitScanForward(&mut SlotNumber, SlotsBitmap & SlotMask) == 0 {
        return FALSE;
    }

    if ((*PortData).AllocateSlot.unwrap_unchecked())((*PortData).ChannelContext, Request, TRUE) == 0 {
        return FALSE;
    }

    if (*Request).Flags & REQUEST_FLAG_NCQ != 0 {
        (*PortData).AllocatedSlots += 1;
    } else {
        (*PortData).AllocatedSlots -= 1;
    }

    if (*Request).Flags & REQUEST_EXCLUSIVE_ACCESS_FLAGS != 0 {
        (*PortData).QueueFlags |= PORT_QUEUE_FLAG_EXCLUSIVE_MODE;
    }

    (*Request).Slot = SlotNumber;

    (*PortData).LastUsedSlot = SlotNumber;
    (*PortData).FreeSlotsBitmap &= !(1 << SlotNumber);
    (*PortData).Slots[SlotNumber as usize] = Request;
    return TRUE;
}

unsafe extern "C" fn AtaReqDispatchRequest(
    Device: PATAPORT_IO_CONTEXT,
    Request: PATA_DEVICE_REQUEST,
    DoReleaseDeviceQueueLock: BOOLEAN,
) {
    let PortData: PATAPORT_PORT_DATA = (*Device).PortData;
    let mut Success: BOOLEAN;

    Success = AtaReqAllocateSlot(Device, PortData, Request);
    if Success != 0 {
        (*PortData).ActiveTimersBitmap |= (1 << (*Request).Slot) as ULONG;
    } else {
        InsertTailList(
            &raw mut (*PortData).PortQueueList,
            &raw mut (*Request).__bindgen_anon_1.PortEntry,
        );

        (*Device).QueueFlags |= (QUEUE_FLAG_FROZEN_SLOT) as i32;
    }

    KeReleaseSpinLockFromDpcLevel((&raw mut (*PortData).QueueLock).cast());

    if DoReleaseDeviceQueueLock != 0 {
        KeReleaseSpinLockFromDpcLevel((&raw mut (*Device).QueueLock).cast());
    }

    if Success != 0 {
        if (*PortData).PortFlags & PORT_FLAG_IS_SIMPLEX != 0 {
            IoAllocateController(
                (*PortData).HwSyncObject,
                (*PortData).ChannelObject,
                Some(AtaReqCallSendRequestSerialized as unsafe extern "system" fn(_, _, _, _) -> _),
                Request as PVOID,
            );
        } else {
            AtaReqSendRequest(Request);
        }
    }
}

unsafe extern "C" fn AtaReqTranslateRequest(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) -> ULONG {
    let mut SrbStatus: UCHAR;

    (*Request).Srb = Srb;
    (*Request).DataBuffer = (*Srb).DataBuffer;
    (*Request).DataTransferLength = (*Srb).DataTransferLength;
    (*Request).Irp = (*Srb).OriginalRequest as PIRP;
    (*Request).Mdl = (*((*Srb).OriginalRequest as PIRP)).MdlAddress;
    (*Request).TimeOut = (*Srb).TimeOutValue;
    (*Request).Complete = None;
    (*Request).Flags = 0;
    (*Request).__bindgen_anon_3.State = 0;

    match ((*Srb).Function as i32) as u32 {
        SRB_FUNCTION_EXECUTE_SCSI => {
            SrbStatus = AtaReqExecuteScsi(DevExt, Request, Srb);
        }
        SRB_FUNCTION_IO_CONTROL => {
            SrbStatus = AtaReqSmartIoControl(DevExt, Request, Srb);
        }
        SRB_FUNCTION_SHUTDOWN | SRB_FUNCTION_FLUSH => {
            SrbStatus = (SRB_STATUS_SUCCESS) as u8;
        }
        _ => {
            UNREACHABLE!();
        }
    }

    return SrbStatus as ULONG;
}

unsafe extern "C" fn AtaReqStartRequest(
    Device: PATAPORT_IO_CONTEXT,
    Request: PATA_DEVICE_REQUEST,
    Srb: PSCSI_REQUEST_BLOCK,
) {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION;
    let mut SrbStatus: UCHAR;

    DevExt = CONTAINING_RECORD!(Device, ATAPORT_DEVICE_EXTENSION, Device);

    SrbStatus = (AtaReqTranslateRequest(DevExt, Request, Srb)) as u8;

    if (SrbStatus) as u32 != SRB_STATUS_PENDING {
        KeReleaseSpinLockFromDpcLevel((&raw mut (*Device).QueueLock).cast());

        (*Request).SrbStatus = SrbStatus;
        (*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState = (REQUEST_STATE_NOT_STARTED) as u8;

        AtaReqStartCompletionDpc(Request);
    } else {
        (*Request).Flags |= (*Srb).SrbFlags & REQUEST_FLAG_NO_KEEP_AWAKE;

        KeAcquireSpinLockAtDpcLevel((&raw mut (*(*Device).PortData).QueueLock).cast());

        AtaReqDispatchRequest(Device, Request, TRUE);
    }
}

unsafe extern "C" fn AtaReqRemovePortRequest(
    mut Device: PATAPORT_IO_CONTEXT,
) -> PATA_DEVICE_REQUEST {
    let mut PortData: PATAPORT_PORT_DATA = (*Device).PortData as PATAPORT_PORT_DATA;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Request: PATA_DEVICE_REQUEST = core::ptr::null_mut();
    let mut Result: PATA_DEVICE_REQUEST = core::ptr::null_mut();
    if (*Device).QueueFlags as u32 & QUEUE_FLAG_FROZEN_SLOT == 0 {
        return core::ptr::null_mut();
    }
    (*Device).QueueFlags &= (!QUEUE_FLAG_FROZEN_SLOT) as i32;
    KeAcquireSpinLockAtDpcLevel((&raw mut (*PortData).QueueLock).cast());
    Entry = (*PortData).PortQueueList.Flink as PLIST_ENTRY;
    while Entry != &raw mut (*PortData).PortQueueList {
        Request = CONTAINING_RECORD!(Entry, ATA_DEVICE_REQUEST, __bindgen_anon_1.PortEntry);
        ASSERT_REQUEST!(Request);
        if (*Request).Device != Device as PATA_IO_CONTEXT_COMMON {
            Entry = (*Entry).Flink as PLIST_ENTRY;
        } else {
            RemoveEntryList(&raw mut (*Request).__bindgen_anon_1.PortEntry);
            Result = Request;
            break;
        }
    }
    KeReleaseSpinLockFromDpcLevel((&raw mut (*PortData).QueueLock).cast());
    return Result;
}

unsafe extern "C" fn AtaReqDeviceQueueRemoveEntry(
    Device: PATAPORT_IO_CONTEXT,
    QueueEntry: PREQUEST_QUEUE_ENTRY,
) {
    RemoveEntryList(&raw mut (*QueueEntry).ListEntry);
}

unsafe extern "system" fn AtaReqDeviceQueueCancelIo(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let mut Device: PATAPORT_IO_CONTEXT;
    let mut OldLevel: KIRQL = core::mem::zeroed();
    let mut Srb: PSCSI_REQUEST_BLOCK;
    let mut QueueEntry: PREQUEST_QUEUE_ENTRY;

    UNREFERENCED_PARAMETER!(DeviceObject);

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    QueueEntry = QUEUE_ENTRY_FROM_IRP!(Irp);
    Device = ((*QueueEntry).Context).cast();

    KeAcquireSpinLock((&raw mut (*Device).QueueLock).cast(), &raw mut OldLevel);
    AtaReqDeviceQueueRemoveEntry(Device, QueueEntry);
    KeReleaseSpinLock((&raw mut (*Device).QueueLock).cast(), OldLevel);

    Srb = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.Scsi.Srb.cast();

    (*Srb).SrbStatus = (SRB_STATUS_ABORTED) as u8;
    (*Srb).__bindgen_anon_1.InternalStatus = (STATUS_CANCELLED) as u32;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
}

unsafe extern "C" fn AtaReqDeviceQueueInsertSrb(
    mut Device: PATAPORT_IO_CONTEXT,
    mut Srb: PSCSI_REQUEST_BLOCK,
    mut Irp: PIRP,
) {
    let mut Entry: PLIST_ENTRY;
    let mut QueueEntry: PREQUEST_QUEUE_ENTRY;
    let mut SortKey: ULONG;

    SortKey = (*Srb).__bindgen_anon_1.QueueSortKey;

    Entry = (*Device).DeviceQueueList.Flink;
    while Entry != &raw mut (*Device).DeviceQueueList {
        QueueEntry = CONTAINING_RECORD!(Entry, REQUEST_QUEUE_ENTRY, ListEntry);

        if (*QueueEntry).SortKey > SortKey {
            break;
        }
        Entry = (*Entry).Flink;
    }

    QueueEntry = QUEUE_ENTRY_FROM_IRP!(Irp);
    (*QueueEntry).Context = (Device).cast();
    (*QueueEntry).SortKey = SortKey;

    InsertTailList(Entry, &raw mut (*QueueEntry).ListEntry);
}

unsafe extern "C" fn AtaReqDeviceQueueAddSrb(
    mut Device: PATAPORT_IO_CONTEXT,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut Irp: PIRP = (*Srb).OriginalRequest as PIRP;
    if (*Device).QueueFlags & (QUEUE_FLAG_FROZEN_REMOVED as i32) != 0 {
        (*Srb).SrbStatus = (SRB_STATUS_NO_DEVICE) as u8;
        return STATUS_NO_SUCH_DEVICE;
    }
    AtaReqDeviceQueueInsertSrb(Device, Srb, Irp);
    AtaDeviceCheckPowerState(Device);
    IoSetCancelRoutine(Irp, Some(AtaReqDeviceQueueCancelIo as unsafe extern "system" fn(_, _)));
    if (*Irp).Cancel as ::core::ffi::c_int != 0 && IoSetCancelRoutine(Irp, None).is_some() {
        AtaReqDeviceQueueRemoveEntry(
            Device,
            QUEUE_ENTRY_FROM_IRP!(Irp),
        );
        (*Srb).SrbStatus = (SRB_STATUS_ABORTED) as u8;
        (*Srb).__bindgen_anon_1.InternalStatus = (STATUS_CANCELLED) as u32;
        return STATUS_CANCELLED;
    }
    return STATUS_PENDING;
}

unsafe extern "C" fn AtaReqGetTagForRequest(
    Device: PATAPORT_IO_CONTEXT,
    Srb: PSCSI_REQUEST_BLOCK,
    Tag: PULONG,
) -> UCHAR {
    if (*Device).QueueFlags & (QUEUE_FLAGS_FROZEN & !(QUEUE_FLAG_FROZEN_QUEUE_LOCK | QUEUE_FLAG_FROZEN_QUEUE_FREEZE)) as i32 != 0 {
        return FALSE;
    }
    if (*Device).QueueFlags & QUEUE_FLAG_FROZEN_QUEUE_FREEZE as i32 != 0
        && (*Srb).SrbFlags & SRB_FLAGS_BYPASS_FROZEN_QUEUE == 0
    {
        return FALSE;
    }
    if (*Device).QueueFlags & QUEUE_FLAG_FROZEN_QUEUE_LOCK as i32 != 0
        && (*Srb).SrbFlags & SRB_FLAGS_BYPASS_LOCKED_QUEUE == 0
    {
        return FALSE;
    }
    return _BitScanForward(Tag, (*Device).FreeRequestsBitmap);
}

unsafe extern "C" fn AtaReqAllocateRequestFromTag(
    mut Device: PATAPORT_IO_CONTEXT,
    mut Tag: ULONG,
) -> PATA_DEVICE_REQUEST {
    let mut Request: PATA_DEVICE_REQUEST = core::ptr::null_mut();
    ASSERT!((*Device).FreeRequestsBitmap & ((1 as ULONG) << Tag) != 0);
    (*Device).FreeRequestsBitmap &= !((1 as ULONG) << Tag);
    Request = (*Device).Requests.offset(Tag as isize);
    ASSERT_REQUEST!(Request);
    (*Request).Tag = (1 as ULONG) << Tag;
    return Request;
}

unsafe extern "C" fn AtaReqDeviceQueueDispatchNextRequest(
    Device: PATAPORT_IO_CONTEXT,
) -> BOOLEAN {
    let mut QueueEntry: PREQUEST_QUEUE_ENTRY = core::ptr::null_mut();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut Request: PATA_DEVICE_REQUEST = core::ptr::null_mut();
    let mut Tag: ULONG = 0;

    if (*Device).FreeRequestsBitmap == 0 {
        return FALSE;
    }

    if (*Device).QueueFlags as u32 & (QUEUE_FLAG_FROZEN_QUEUE_FREEZE | QUEUE_FLAG_FROZEN_QUEUE_LOCK) != 0
        && (*Device).QueueFlags as u32 & QUEUE_FLAGS_FROZEN_NOT_BYPASS == 0
    {
        let mut Entry: PLIST_ENTRY = (*Device).DeviceQueueList.Flink;
        while Entry != &raw mut (*Device).DeviceQueueList {
            QueueEntry = CONTAINING_RECORD!(Entry, REQUEST_QUEUE_ENTRY, ListEntry);

            Irp = IRP_FROM_QUEUE_ENTRY!(QueueEntry);
            Srb = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

            if ((AtaReqGetTagForRequest(Device, Srb, &raw mut Tag)) == 0) {
                Entry = (*Entry).Flink;
                continue;
            }

            AtaReqDeviceQueueRemoveEntry(Device, QueueEntry);

            if IoSetCancelRoutine(Irp, None).is_none() {
                InitializeListHead(&raw mut (*QueueEntry).ListEntry);
                Entry = (*Entry).Flink;
                continue;
            }

            Request = AtaReqAllocateRequestFromTag(Device, Tag);

            AtaReqStartRequest(Device, Request, Srb);
            return TRUE;
        }
    } else {
        let mut Entry: PLIST_ENTRY = core::ptr::null_mut();

        if (*Device).QueueFlags as u32 & QUEUE_FLAGS_FROZEN != 0 {
            return FALSE;
        }

        NT_VERIFY!(_BitScanForward(&raw mut Tag, (*Device).FreeRequestsBitmap));

        Entry = (*Device).DeviceQueueList.Flink;
        while Entry != &raw mut (*Device).DeviceQueueList {
            QueueEntry = CONTAINING_RECORD!(Entry, REQUEST_QUEUE_ENTRY, ListEntry);

            Irp = IRP_FROM_QUEUE_ENTRY!(QueueEntry);
            Srb = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

            AtaReqDeviceQueueRemoveEntry(Device, QueueEntry);

            if IoSetCancelRoutine(Irp, None).is_none() {
                InitializeListHead(&raw mut (*QueueEntry).ListEntry);
                Entry = (*Entry).Flink;
                continue;
            }

            Request = AtaReqAllocateRequestFromTag(Device, Tag);

            AtaReqStartRequest(Device, Request, Srb);
            return TRUE;
        }
    }

    return FALSE;
}

unsafe extern "C" fn AtaReqStartSrb(
    Device: PATAPORT_IO_CONTEXT,
    Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut OldIrql: KIRQL = 0;
    let mut Status: NTSTATUS;
    let mut Request: PATA_DEVICE_REQUEST;
    let mut Tag: ULONG = 0;

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &mut OldIrql);
    KeAcquireSpinLockAtDpcLevel(&mut (*Device).QueueLock);

    (*Device).Statistics.RequestsStarted = (*Device).Statistics.RequestsStarted.wrapping_add(1);

    if AtaReqGetTagForRequest(Device, Srb, &mut Tag) == 0 {
        Status = AtaReqDeviceQueueAddSrb(Device, Srb);

        KeReleaseSpinLockFromDpcLevel(&mut (*Device).QueueLock);
    } else {
        Request = AtaReqAllocateRequestFromTag(Device, Tag);
        AtaReqStartRequest(Device, Request, Srb);

        Status = STATUS_PENDING;
    }

    KeLowerIrql(OldIrql);

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqFreezeQueue(
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    ReasonFlags: u32,
) {
    let Device: *mut ATAPORT_IO_CONTEXT = core::ptr::addr_of_mut!((*DevExt).Device);
    let mut OldLevel: KIRQL = 0;

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*Device).QueueLock)).cast(), &mut OldLevel);
    _InterlockedOr(core::ptr::addr_of_mut!((*Device).QueueFlags), (ReasonFlags) as i32);
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*Device).QueueLock)).cast(), OldLevel);
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqThawQueue(
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    ReasonFlags: u32,
) {
    let Device: *mut ATAPORT_IO_CONTEXT = core::ptr::addr_of_mut!((*DevExt).Device);
    let mut PortData: *mut ATAPORT_PORT_DATA;
    let mut OldIrql: KIRQL = core::mem::zeroed();

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &mut OldIrql);
    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Device).QueueLock)).cast());

    if (*Device).FreeRequestsBitmap != (*Device).MaxRequestsBitmap {
        AtaDeviceCheckPowerState(Device);
    }

    PortData = (*Device).PortData;
    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*PortData).QueueLock)).cast());
    if ((AtaReqPortQueueListDispatchNextRequest(PortData)) == 0) {
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*PortData).QueueLock)).cast());
    }

    _InterlockedAnd(core::ptr::addr_of_mut!((*Device).QueueFlags), (!ReasonFlags) as i32);

    if ((AtaReqDeviceQueueDispatchNextRequest(Device)) == 0) {
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Device).QueueLock)).cast());
    }

    KeLowerIrql(OldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqWaitForOutstandingIoToComplete(
    Device: PATAPORT_IO_CONTEXT,
    Srb: PSCSI_REQUEST_BLOCK,
) {
    let mut OldIrql: KIRQL = 0;
    let mut DoWait: BOOLEAN;
    let mut Request: PATA_DEVICE_REQUEST;

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &mut OldIrql);
    KeAcquireSpinLockAtDpcLevel(&mut (*Device).QueueLock);

    KeClearEvent(&mut (*Device).QueueStoppedEvent);

    if !Srb.is_null() {
        (*Device).QuiescenceSrb = Srb;
    }

    if (*Device).FreeRequestsBitmap != (*Device).MaxRequestsBitmap {
        (*Device).QueueFlags |= (QUEUE_FLAG_SIGNAL_STOP) as i32;
        DoWait = Srb.is_null() as BOOLEAN;
    } else {
        AtaDeviceQueueEmptyEvent(Device);

        KeSetEvent(
            &mut (*Device).QueueStoppedEvent,
            0,
            FALSE,
        );
        DoWait = FALSE;
    }

    Request = AtaReqRemovePortRequest(Device);

    KeReleaseSpinLockFromDpcLevel(&mut (*Device).QueueLock);
    KeLowerIrql(OldIrql);

    if !Request.is_null() {
        (*Request).SrbStatus = (SRB_STATUS_BUSY) as u8;
        (*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState = (REQUEST_STATE_NOT_STARTED) as u8;

        KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &mut OldIrql);
        AtaReqCompleteRequest(Request);
        KeLowerIrql(OldIrql);
    }

    if ((DoWait) == 0) {
        return;
    }

    KeWaitForSingleObject(
        (&raw mut (*Device).QueueStoppedEvent).cast(),
        _KWAIT_REASON_Executive,
        (0i32) as KPROCESSOR_MODE,
        FALSE,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn AtaReqFlushDeviceQueue(Device: PATAPORT_IO_CONTEXT) {
    let mut Request: PATA_DEVICE_REQUEST;
    let mut OldLevel: KIRQL = 0;
    let mut Entry: PLIST_ENTRY;

    KeAcquireSpinLock(&mut (*Device).QueueLock, &mut OldLevel);

    Entry = (*Device).DeviceQueueList.Flink;
    while Entry != &mut (*Device).DeviceQueueList {
        let mut QueueEntry: PREQUEST_QUEUE_ENTRY;
        let mut Irp: PIRP;
        let mut Srb: PSCSI_REQUEST_BLOCK;

        QueueEntry = CONTAINING_RECORD!(Entry, REQUEST_QUEUE_ENTRY, ListEntry);

        AtaReqDeviceQueueRemoveEntry(Device, QueueEntry);

        Irp = IRP_FROM_QUEUE_ENTRY!(QueueEntry);
        if IoSetCancelRoutine(Irp, None).is_some() {
            InitializeListHead(&mut (*QueueEntry).ListEntry);
            Entry = (*Entry).Flink;
            continue;
        }

        KeReleaseSpinLock(&mut (*Device).QueueLock, OldLevel);

        Srb = ((*IoGetCurrentIrpStackLocation(Irp)).Parameters.Scsi.Srb).cast();

        (*Srb).SrbStatus = (SRB_STATUS_ABORTED) as u8;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        (*Irp).IoStatus.Information = ((*Srb).DataTransferLength) as _;
        IoCompleteRequest(Irp, (IO_DISK_INCREMENT) as CCHAR);

        KeAcquireSpinLock(&mut (*Device).QueueLock, &mut OldLevel);
        Entry = (*Entry).Flink;
    }

    Request = AtaReqRemovePortRequest(Device);

    KeReleaseSpinLock(&mut (*Device).QueueLock, OldLevel);

    if !Request.is_null() {
        let mut OldIrql: KIRQL = 0;

        (*Request).SrbStatus = (SRB_STATUS_ABORTED) as u8;
        (*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState = (REQUEST_STATE_NOT_STARTED) as u8;

        KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &mut OldIrql);
        AtaReqCompleteRequest(Request);
        KeLowerIrql(OldIrql);
    }
}

unsafe extern "C" fn AtaPdoHandleAttachReleaseDevice(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut Status: ULONG = 0;
    let mut SrbStatus: ULONG = 0;

    if ((*Srb).Function) as u32 == SRB_FUNCTION_RELEASE_DEVICE {
        (*DevExt).DeviceClaimed = FALSE;
        SrbStatus = SRB_STATUS_SUCCESS;
        Status = (STATUS_SUCCESS) as u32;
    } else if (*DevExt).DeviceClaimed != 0 {
        SrbStatus = SRB_STATUS_BUSY;
        Status = (STATUS_DEVICE_BUSY) as u32;
    } else {
        (*DevExt).DeviceClaimed = TRUE;
        (*Srb).DataBuffer = (*DevExt).Common.Self_ as PVOID;
        SrbStatus = SRB_STATUS_SUCCESS;
        Status = (STATUS_SUCCESS) as u32;
    }

    (*Srb).SrbStatus = SrbStatus as UCHAR;
    return Status as NTSTATUS;
}

unsafe extern "C" fn AtaPdoHandleQuiesceDevice(
    mut Device: PATAPORT_IO_CONTEXT,
    mut Irp: PIRP,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    IoMarkIrpPending(Irp);
    AtaReqWaitForOutstandingIoToComplete(Device, Srb);
    (*Srb).SrbStatus = (SRB_STATUS_PENDING) as u8;
    return STATUS_PENDING;
}

unsafe extern "C" fn AtaPdoHandleIoControl(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Status: *mut NTSTATUS,
) -> BOOLEAN {
    let SrbControl: PSRB_IO_CONTROL = (*Srb).DataBuffer as PSRB_IO_CONTROL;

    PAGED_CODE!();

    match (*SrbControl).ControlCode {
        IOCTL_SCSI_MINIPORT_SMART_VERSION => {
            *Status = AtaPdoHandleMiniportSmartVersion(DevExt, Srb);
        }
        IOCTL_SCSI_MINIPORT_IDENTIFY => {
            *Status = AtaPdoHandleMiniportIdentify(DevExt, Srb);
        }
        IOCTL_SCSI_MINIPORT_DISABLE_SMART
        | IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE
        | IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTO_OFFLINE
        | IOCTL_SCSI_MINIPORT_ENABLE_SMART
        | IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS
        | IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS
        | IOCTL_SCSI_MINIPORT_READ_SMART_LOG
        | IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS
        | IOCTL_SCSI_MINIPORT_RETURN_STATUS
        | IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES
        | IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG => {
            return FALSE;
        }
        _ => {
            (*Srb).SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8;
            *Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    return TRUE;
}

unsafe extern "C" fn AtaPdoDispatchScsi(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Srb: PSCSI_REQUEST_BLOCK;

    Srb = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

    Status = IoAcquireRemoveLock(&mut (*DevExt).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Srb).SrbStatus = (SRB_STATUS_NO_DEVICE) as u8;
        Status = STATUS_NO_SUCH_DEVICE;

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

        return Status;
    }

    match (*Srb).Function as u32 {
        SRB_FUNCTION_CLAIM_DEVICE | SRB_FUNCTION_RELEASE_DEVICE => {
            Status = AtaPdoHandleAttachReleaseDevice(DevExt, Srb);
        }

        SRB_FUNCTION_LOCK_QUEUE => {
            AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_QUEUE_LOCK);

            (*Srb).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
            Status = STATUS_SUCCESS;
        }

        SRB_FUNCTION_FLUSH_QUEUE => {
            AtaReqFlushDeviceQueue(&mut (*DevExt).Device);
            let mut Reason: ULONG;
            if ((*Srb).Function) as u32 == SRB_FUNCTION_UNLOCK_QUEUE {
                Reason = QUEUE_FLAG_FROZEN_QUEUE_LOCK;
            } else {
                Reason = QUEUE_FLAG_FROZEN_QUEUE_FREEZE;
            }
            AtaReqThawQueue(DevExt, Reason);

            (*Srb).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
            Status = STATUS_SUCCESS;
        }

        SRB_FUNCTION_UNLOCK_QUEUE | SRB_FUNCTION_RELEASE_QUEUE => {
            let mut Reason: ULONG;
            if ((*Srb).Function) as u32 == SRB_FUNCTION_UNLOCK_QUEUE {
                Reason = QUEUE_FLAG_FROZEN_QUEUE_LOCK;
            } else {
                Reason = QUEUE_FLAG_FROZEN_QUEUE_FREEZE;
            }
            AtaReqThawQueue(DevExt, Reason);

            (*Srb).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
            Status = STATUS_SUCCESS;
        }

        SRB_FUNCTION_QUIESCE_DEVICE => {
            Status = AtaPdoHandleQuiesceDevice(&mut (*DevExt).Device, Irp, Srb);
        }

        SRB_FUNCTION_IO_CONTROL => {
            if ((AtaPdoHandleIoControl(DevExt, Srb, &mut Status)) != 0) {
                // break out of match
            } else {
                let mut AtaScsiAddress: ATA_SCSI_ADDRESS;
                IoMarkIrpPending(Irp);

                AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;
                (*Srb).PathId = AtaScsiAddress.PathId;
                (*Srb).TargetId = AtaScsiAddress.TargetId;
                (*Srb).Lun = AtaScsiAddress.Lun;

                (*Srb).SrbExtension = core::ptr::null_mut();

                Status = AtaReqStartSrb(&mut (*DevExt).Device, Srb);
            }
        }

        SRB_FUNCTION_SHUTDOWN | SRB_FUNCTION_FLUSH | SRB_FUNCTION_EXECUTE_SCSI => {
            let mut AtaScsiAddress: ATA_SCSI_ADDRESS;
            IoMarkIrpPending(Irp);

            AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;
            (*Srb).PathId = AtaScsiAddress.PathId;
            (*Srb).TargetId = AtaScsiAddress.TargetId;
            (*Srb).Lun = AtaScsiAddress.Lun;

            (*Srb).SrbExtension = core::ptr::null_mut();

            Status = AtaReqStartSrb(&mut (*DevExt).Device, Srb);
        }

        _ => {
            (*Srb).SrbStatus = (SRB_STATUS_INVALID_REQUEST) as u8;
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
    }

    IoReleaseRemoveLock(&mut (*DevExt).Common.RemoveLock, (Irp).cast());

    return Status;
}

unsafe extern "C" fn AtaFdoDispatchScsi(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Srb: PSCSI_REQUEST_BLOCK;

    UNREFERENCED_PARAMETER!(ChanExt);

    PAGED_CODE!();

    Srb = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

    (*Srb).SrbStatus = (SRB_STATUS_NO_DEVICE) as u8;
    Status = STATUS_NO_SUCH_DEVICE;

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn AtaDispatchScsi(DeviceObject: *mut _DEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    if IS_FDO!((*DeviceObject).DeviceExtension) != false {
        return AtaFdoDispatchScsi(((*DeviceObject).DeviceExtension).cast(), Irp);
    } else {
        return AtaPdoDispatchScsi(((*DeviceObject).DeviceExtension).cast(), Irp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn AtaPortIoTimer(
    DeviceObject: PDEVICE_OBJECT,
    Context: *mut core::ffi::c_void,
) {
    let PortData: PATAPORT_PORT_DATA = Context as _;
    let mut Slot: u32 = 0;
    let mut ActiveTimersBitmap: u32;

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*PortData).QueueLock));

    ActiveTimersBitmap = (*PortData).ActiveTimersBitmap;
    while _BitScanForward(
        core::ptr::addr_of_mut!(Slot),
        ActiveTimersBitmap as core::ffi::c_ulong,
    ) as core::ffi::c_int
        != 0
    {
        ActiveTimersBitmap &= !((1 as u32) << Slot);

        if (*PortData).TimerCount[Slot as usize] > 0
            && {
                (*PortData).TimerCount[Slot as usize] -= 1;
                (*PortData).TimerCount[Slot as usize] == 0
            }
        {
            let OldIrql: KIRQL = KeAcquireInterruptSpinLock((*PortData).InterruptObject);

            if (*PortData).ActiveSlotsBitmap & ((1 as u32) << Slot) != 0 {
                AtaPortTimeout(PortData, Slot);
            }

            KeReleaseInterruptSpinLock((*PortData).InterruptObject, OldIrql);
        }
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*PortData).QueueLock));
}
