use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoInvalidateDeviceRelations, KeAcquireInterruptSpinLock, KeInsertQueueDpc, KeReleaseInterruptSpinLock, KeWaitForSingleObject, PsTerminateSystemThread, va_list};
use core::ffi::c_void;

unsafe extern "C" fn AtaPortQueueEvent(
    mut PortData: PATAPORT_PORT_DATA,
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut FailedRequest: PATA_DEVICE_REQUEST,
    mut Action: ATA_PORT_ACTION,
) {
    if Action & (ACTION_PORT_RESET | ACTION_DEVICE_ERROR) != 0 {
        if Action & ACTION_DEVICE_ERROR != 0 {
            if !FailedRequest.is_null() {
                ASSERT_REQUEST!(FailedRequest);

                DevExt = CONTAINING_RECORD!((*FailedRequest).Device, ATAPORT_DEVICE_EXTENSION, Device);

                if (*FailedRequest).Flags & REQUEST_FLAG_INTERNAL != 0 {
                    Action &= !ACTION_DEVICE_ERROR;
                } else {
                    (*PortData).Worker.FailedRequest = FailedRequest;
                }
            } else {
                Action &= !ACTION_DEVICE_ERROR;
                Action |= ACTION_PORT_RESET;
            }
        }

        if (*PortData).ActiveSlotsBitmap & 1 != 0
            && (*(*PortData).Slots[0 as usize]).Flags & REQUEST_FLAG_INTERNAL != 0
        {
            let mut InternalRequest: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;

            if Action & ACTION_PORT_RESET != 0 {
                (*InternalRequest).SrbStatus = (SRB_STATUS_BUS_RESET) as u8;
            }

            (*PortData).ActiveSlotsBitmap &= !1;

            AtaReqStartCompletionDpc(InternalRequest);
        }

        (*PortData).Worker.PausedSlotsBitmap |= (*PortData).ActiveSlotsBitmap;
        (*PortData).ActiveSlotsBitmap = 0;

        (*PortData).AbortChannel.expect("non-null function pointer")(
            (*PortData).ChannelContext,
            (Action & ACTION_PORT_RESET != 0) as i32 as u8,
        );
    }

    if !DevExt.is_null() {
        _InterlockedOr(&raw mut (*DevExt).Worker.EventsPending, (Action) as i32);
    }
    _InterlockedOr(&raw mut (*PortData).Worker.EventsPending, (Action) as i32);

    if ((*PortData).InterruptFlags as u32 & PORT_INT_FLAG_IS_IO_ACTIVE) != 0 {
        _InterlockedAnd(&raw mut (*PortData).InterruptFlags, (!PORT_INT_FLAG_IS_IO_ACTIVE) as i32);
        KeInsertQueueDpc(&raw mut (*PortData).Worker.Dpc, core::ptr::null_mut(), core::ptr::null_mut());
    }
}

unsafe extern "C" fn AtaPortClearPortAction(
    Context: PATA_WORKER_CONTEXT,
    Action: ATA_PORT_ACTION,
) -> ATA_PORT_ACTION {
    return _InterlockedAnd(
        core::ptr::addr_of_mut!((*Context).EventsPending),
        (!(Action as ULONG)) as i32,
    ) as ATA_PORT_ACTION;
}

unsafe extern "C" fn AtaPortClearDeviceAction(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Action: ATA_PORT_ACTION,
) -> BOOLEAN {
    return ((_InterlockedAnd(
        &raw mut (*DevExt).Worker.EventsPending,
        !(Action as ULONG) as i32,
    ) as ULONG
        & Action as ULONG
        != 0) as i32) as BOOLEAN;
}

unsafe extern "C" fn AtaPortFindDeviceForAction(
    PortData: PATAPORT_PORT_DATA,
    Action: ATA_PORT_ACTION,
) -> PATAPORT_DEVICE_EXTENSION {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Result: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION;
    let mut Entry: PSINGLE_LIST_ENTRY;
    let mut OldIrql: KIRQL;
    let mut OldLevel: KIRQL = core::mem::zeroed();

    ChanExt = CONTAINING_RECORD!(PortData, ATAPORT_CHANNEL_EXTENSION, PortData);

    KeAcquireSpinLock(&mut (*ChanExt).PdoListLock as *mut _ as *mut usize, &mut OldLevel);
    OldIrql = KeAcquireInterruptSpinLock((*PortData).InterruptObject);

    Entry = (*ChanExt).PdoList.Next;
    while !Entry.is_null() {
        DevExt = CONTAINING_RECORD!(Entry, ATAPORT_DEVICE_EXTENSION, ListEntry);

        if ((*DevExt).Device.AtaScsiAddress.PathId) as u32 != (*PortData).PortNumber {
            Entry = (*Entry).Next;
            continue;
        }

        if (*DevExt).ReportedMissing != 0 {
            Entry = (*Entry).Next;
            continue;
        }

        if (*DevExt).Worker.Flags & DEV_WORKER_FLAG_REMOVED != 0 {
            Entry = (*Entry).Next;
            continue;
        }

        if ((*DevExt).Worker.EventsPending as u32 & Action) != 0 {
            Result = DevExt;
            break;
        }

        Entry = (*Entry).Next;
    }

    if Result.is_null() {
        AtaPortClearPortAction(&mut (*PortData).Worker, Action);
    }

    KeReleaseInterruptSpinLock((*PortData).InterruptObject, OldIrql);
    KeReleaseSpinLock(&mut (*ChanExt).PdoListLock as *mut _ as *mut usize, OldLevel);

    Result
}

unsafe extern "C" fn AtaPortOnAsyncNotification(
    PortData: PATAPORT_PORT_DATA,
    DeviceBitmap: ULONG,
) {
    KeInsertQueueDpc(
        &raw mut (*PortData).Worker.NotificationDpc,
        UlongToPtr!(DeviceBitmap),
        core::ptr::null_mut(),
    );
}

unsafe extern "C" fn AtaPortOnResetNotification(
    mut PortData: PATAPORT_PORT_DATA,
    mut DeviceBitmap: ULONG,
) {
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION =
        ::core::ptr::null_mut::<ATAPORT_CHANNEL_EXTENSION>();
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = ATA_SCSI_ADDRESS {
        c2rust_unnamed: C2RustUnnamed_165 {
            Lun: 0,
            TargetId: 0,
            PathId: 0,
            IsValid: 0,
        },
    };
    let mut DevExt: PATAPORT_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<ATAPORT_DEVICE_EXTENSION>();
    _InterlockedOr(
        &raw mut (*PortData).Worker.EventsPending,
        (ACTION_ENUM_DEVICE
            | ACTION_DEVICE_CONFIG
            | ACTION_PORT_TIMING) as ::core::ffi::c_long,
    );
    ChanExt = CONTAINING_RECORD!(PortData, ATAPORT_CHANNEL_EXTENSION, PortData);
    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut Event: ATA_PORT_ACTION = 0 as ATA_PORT_ACTION;
        DevExt = AtaFdoFindNextDeviceByPath(
            ChanExt,
            &raw mut AtaScsiAddress,
            FALSE,
            ::core::ptr::null_mut(),
        );
        if DevExt.is_null() {
            break;
        }
        if DeviceBitmap
            & ((1) << AtaScsiAddress.c2rust_unnamed.TargetId as ULONG)
                as ULONG
            == 0
        {
            continue;
        }
        if (*DevExt).Device.DeviceFlags & DEVICE_PNP_STARTED != 0 {
            Event = (ACTION_ENUM_DEVICE | ACTION_DEVICE_CONFIG) as ATA_PORT_ACTION;
        } else {
            Event = ACTION_ENUM_DEVICE;
        }
        _InterlockedOr(
            &raw mut (*DevExt).Worker.EventsPending,
            Event as ::core::ffi::c_long,
        );
    }
}

unsafe extern "C" fn AtaPortOnRequestComplete(
    PortData: PATAPORT_PORT_DATA,
    mut CommandsCompleted: ULONG,
) {
    let mut Slot: ULONG = 0;

    (*PortData).ActiveSlotsBitmap &= !CommandsCompleted;

    while _BitScanForward(&mut Slot, CommandsCompleted) != 0 {
        let mut Request: PATA_DEVICE_REQUEST;

        CommandsCompleted &= !((1 << Slot) as ULONG);

        Request = (*PortData).Slots[Slot as usize];
        ASSERT_REQUEST!(Request);

        InterlockedPushEntrySList(&mut AtapCompletionQueueList, &mut (*Request).__bindgen_anon_1.CompletionEntry);
    }

    KeInsertQueueDpc(&mut AtapCompletionDpc, core::ptr::null_mut(), core::ptr::null_mut());
}

unsafe extern "C" fn AtaPortMarkDeviceFailed(
    PortData: *mut PATAPORT_PORT_DATA,
    DevExt: *mut PATAPORT_DEVICE_EXTENSION,
) {
    (*(*DevExt)).Worker.ResetRetryCount = (*(*DevExt)).Worker.ResetRetryCount.wrapping_add(1);
    if (*(*DevExt)).Worker.ResetRetryCount >= 4 {
        (*(*DevExt)).Worker.Flags |= DEV_WORKER_FLAG_REMOVED;
        (*(*PortData)).Worker.BadDeviceBitmap |= 1
            << (*(*DevExt)).Device.AtaScsiAddress.TargetId as ULONG;
    }
}

unsafe extern "C" fn AtaPortEnumeratePort(PortData: *mut ATAPORT_PORT_DATA) -> NTSTATUS {
    let mut ChanExt: *mut ATAPORT_CHANNEL_EXTENSION;
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = core::mem::zeroed();

    _InterlockedOr(
        core::ptr::addr_of_mut!((*PortData).InterruptFlags),
        (PORT_INT_FLAG_IGNORE_LINK_IRQ) as i32,
    );
    (*PortData).Worker.DeviceCount = (((*PortData).EnumerateChannel).unwrap_unchecked())((*PortData).ChannelContext);
    _InterlockedAnd(
        core::ptr::addr_of_mut!((*PortData).InterruptFlags),
        (!PORT_INT_FLAG_IGNORE_LINK_IRQ) as i32,
    );

    INFO!(
        b"CH %lu: Detected %lu devices\n\0".as_ptr() as *const i8,
        (*PortData).PortNumber,
        (*PortData).Worker.DeviceCount,
    );

    if AtaPortClearPortAction(
        core::ptr::addr_of_mut!((*PortData).Worker),
        ACTION_ENUM_PORT,
    ) != 0
    {
        (*PortData).Worker.Flags |= WORKER_FLAG_COMPLETE_PORT_ENUM_EVENT;
    }

    ChanExt = CONTAINING_RECORD!(PortData, ATAPORT_CHANNEL_EXTENSION, PortData);

    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut DevExt: *mut ATAPORT_DEVICE_EXTENSION;

        DevExt = AtaFdoFindNextDeviceByPath(
            ChanExt,
            core::ptr::addr_of_mut!(AtaScsiAddress),
            FALSE,
            core::ptr::null_mut(),
        );
        if DevExt.is_null() {
            break;
        }

        if (AtaScsiAddress.TargetId) as u32 >= (*PortData).Worker.DeviceCount {
            (*DevExt).Worker.Flags |= DEV_WORKER_FLAG_REMOVED;
        }
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPortResetPort(PortData: PATAPORT_PORT_DATA) -> NTSTATUS {
    _InterlockedOr(
        core::ptr::addr_of_mut!((*PortData).InterruptFlags),
        (PORT_INT_FLAG_IGNORE_LINK_IRQ) as i32,
    );
    (((*PortData).ResetChannel).unwrap_unchecked())((*PortData).ChannelContext);
    _InterlockedAnd(
        core::ptr::addr_of_mut!((*PortData).InterruptFlags),
        (!PORT_INT_FLAG_IGNORE_LINK_IRQ) as i32,
    );

    AtaPortEnumeratePort(PortData);
    AtaPortOnResetNotification(PortData, MAXULONG);

    AtaPortClearPortAction(
        core::ptr::addr_of_mut!((*PortData).Worker),
        ACTION_PORT_RESET,
    );

    AtaPortSelectTimings(PortData, TRUE);

    (*PortData).Worker.ResetRetryCount = (*PortData).Worker.ResetRetryCount.wrapping_add(1);
    if (*PortData).Worker.ResetRetryCount >= 10 {
        (*PortData).Worker.BadDeviceBitmap = MAXULONG;
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPortEnumerateDevice(
    PortData: PATAPORT_PORT_DATA,
    Action: ATA_PORT_ACTION,
) -> NTSTATUS {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();

    if Action == ACTION_ENUM_DEVICE_NEW {
        DevExt = (*PortData).Worker.EnumDevExt;
        AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_PORT_BUSY);
    } else {
        DevExt = AtaPortFindDeviceForAction(PortData, Action);
        if DevExt.is_null() {
            return STATUS_SUCCESS;
        }
    }

    if (*PortData).Worker.BadDeviceBitmap
        & ((1) << (*DevExt).Device.AtaScsiAddress.TargetId as ULONG)
        != 0
    {
        (*DevExt).Worker.EnumStatus = (DEV_STATUS_NO_DEVICE) as i32;
    } else {
        (*DevExt).Worker.EnumStatus = (AtaPortIdentifyDevice(PortData, DevExt)) as i32;
        if ((*DevExt).Worker.EnumStatus) as u32 == DEV_STATUS_FAILED {
            AtaPortMarkDeviceFailed((PortData).cast(), (DevExt).cast());
            return STATUS_ADAPTER_HARDWARE_ERROR;
        }
    }

    if ((*DevExt).Worker.EnumStatus) as u32 != DEV_STATUS_SAME_DEVICE
        && (*DevExt).Device.DeviceFlags & DEVICE_UNINITIALIZED == 0
    {
        (*DevExt).Worker.Flags |= DEV_WORKER_FLAG_REMOVED;
    }

    if AtaPortClearDeviceAction(DevExt, Action) != 0 {
        if Action == ACTION_ENUM_DEVICE_NEW {
            AtaPortClearPortAction(&mut (*PortData).Worker, Action);
        }
        KeSetEvent(
            &mut (*DevExt).Worker.EnumerationEvent,
            0,
            FALSE,
        );
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPortRecoveryFromError(mut PortData: PATAPORT_PORT_DATA) -> NTSTATUS {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    DevExt = AtaPortFindDeviceForAction(PortData, ACTION_DEVICE_ERROR);
    if DevExt.is_null() {
        return STATUS_SUCCESS;
    }
    Status = AtaPortDeviceProcessError(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        AtaPortMarkDeviceFailed((PortData).cast(), (DevExt).cast());
        return Status;
    }
    AtaPortClearDeviceAction(DevExt, ACTION_DEVICE_ERROR);
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPortConfigureDevice(
    mut PortData: PATAPORT_PORT_DATA,
) -> NTSTATUS {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    DevExt = AtaPortFindDeviceForAction(PortData, ACTION_DEVICE_CONFIG);
    if DevExt.is_null() {
        return STATUS_SUCCESS;
    }

    Status = AtaPortDeviceProcessConfig(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        AtaPortMarkDeviceFailed((PortData).cast(), (DevExt).cast());
        return Status;
    }

    if AtaPortClearDeviceAction(DevExt, ACTION_DEVICE_CONFIG) != 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*DevExt).Worker.ConfigureEvent),
            0,
            FALSE,
        );
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPortDeviceChangePower(mut PortData: PATAPORT_PORT_DATA) -> NTSTATUS {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    DevExt = AtaPortFindDeviceForAction(PortData, ACTION_DEVICE_POWER);
    if DevExt.is_null() {
        return STATUS_SUCCESS;
    }

    Status = AtaPortDeviceProcessPowerChange(PortData, DevExt);
    if Status == STATUS_ADAPTER_HARDWARE_ERROR {
        AtaPortMarkDeviceFailed((PortData).cast(), (DevExt).cast());
        return Status;
    }

    AtaPortClearDeviceAction(DevExt, ACTION_DEVICE_POWER);
    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPortSetTransferMode(PortData: PATAPORT_PORT_DATA) -> NTSTATUS {
    AtaPortSelectTimings(PortData, FALSE);
    AtaPortClearPortAction(&raw mut (*PortData).Worker, ACTION_PORT_TIMING);

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaPortGetNextEvent(
    PortData: PATAPORT_PORT_DATA,
    Action: *mut ATA_PORT_ACTION,
) -> BOOLEAN {
    let mut OldIrql: KIRQL;
    let mut i: ULONG;
    let mut EventIndex: ULONG;
    let mut Success: BOOLEAN;

    OldIrql = KeAcquireInterruptSpinLock((*PortData).InterruptObject);

    Success = _BitScanForward(
        &raw mut EventIndex,
        (*PortData).Worker.EventsPending as ::core::ffi::c_ulong,
    ) as BOOLEAN;
    if Success == 0 {
        let mut Request: PATA_DEVICE_REQUEST;

        _InterlockedOr(
            &raw mut (*PortData).InterruptFlags,
            PORT_INT_FLAG_IS_IO_ACTIVE as ::core::ffi::c_long,
        );

        Request = (*PortData).Worker.FailedRequest;
        if !Request.is_null()
            && (*PortData).Worker.PausedSlotsBitmap
                & ((1 as ::core::ffi::c_int) << (*Request).Slot) as ULONG
                == 0
        {
            AtaReqStartCompletionDpc(Request);
        }
        (*PortData).Worker.FailedRequest = ::core::ptr::null_mut::<_ATA_DEVICE_REQUEST>();

        i = 0 as ULONG;
        while i < MAX_SLOTS as ULONG {
            if !((*PortData).Worker.PausedSlotsBitmap
                & ((1 as ::core::ffi::c_int) << i) as ULONG
                == 0)
            {
                Request = (*PortData).Slots[i as usize];
                ASSERT_REQUEST!(!Request.is_null());

                (*Request).SrbStatus = SRB_STATUS_BUSY as UCHAR;
                (*Request).__bindgen_anon_3.__bindgen_anon_1.InternalState = REQUEST_STATE_REQUEUE as UCHAR;
                AtaReqStartCompletionDpc(Request);
            }
            i = i.wrapping_add(1);
        }

        (*PortData).Worker.PausedSlotsBitmap = 0 as ULONG;
    }

    KeReleaseInterruptSpinLock((*PortData).InterruptObject, OldIrql);

    *Action = ((1 as ::core::ffi::c_int) << EventIndex) as ATA_PORT_ACTION;
    return Success;
}

unsafe extern "C" fn AtaPortRunStateMachine(PortData: PATAPORT_PORT_DATA) {
    let mut Action: ATA_PORT_ACTION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    loop {
        if AtaPortGetNextEvent(PortData, &mut Action) == 0 {
            break;
        }

        if (*PortData).Worker.StateLoopCount.wrapping_add(1) > 1500 {
            (*PortData).Worker.StateLoopCount = (*PortData).Worker.StateLoopCount.wrapping_add(1);
        }

        match Action {
            ACTION_PORT_RESET => {
                Status = AtaPortResetPort(PortData);
            }
            ACTION_ENUM_PORT => {
                Status = AtaPortEnumeratePort(PortData);
            }
            ACTION_ENUM_DEVICE | ACTION_ENUM_DEVICE_NEW => {
                Status = AtaPortEnumerateDevice(PortData, Action);
            }
            ACTION_PORT_TIMING => {
                Status = AtaPortSetTransferMode(PortData);
            }
            ACTION_DEVICE_CONFIG => {
                Status = AtaPortConfigureDevice(PortData);
            }
            ACTION_DEVICE_ERROR => {
                Status = AtaPortRecoveryFromError(PortData);
            }
            ACTION_DEVICE_POWER => {
                Status = AtaPortDeviceChangePower(PortData);
            }
            _ => {
                ASSERT!(FALSE);
                UNREACHABLE!();
            }
        }

        if Status == STATUS_ADAPTER_HARDWARE_ERROR {
            _InterlockedOr(
                &mut (*PortData).Worker.EventsPending,
                (ACTION_PORT_RESET) as i32,
            );
        }
    }
}

unsafe extern "C" fn AtaPortWaitForIdle(PortData: PATAPORT_PORT_DATA) {
    let mut OldIrql: KIRQL = 0;
    let mut DoWait: BOOLEAN = FALSE;

    KeAcquireSpinLock((&raw mut (*PortData).QueueLock).cast(), &raw mut OldIrql);
    KeClearEvent(&raw mut (*PortData).QueueStoppedEvent);
    if ((AtaPortQueueEmpty(PortData)) == 0) {
        (*PortData).QueueFlags |= PORT_QUEUE_FLAG_SIGNAL_STOP;
        DoWait = TRUE;
    }
    KeReleaseSpinLock((&raw mut (*PortData).QueueLock).cast(), OldIrql);
    if ((DoWait) != 0) {
        KeWaitForSingleObject(
            &raw mut (*PortData).QueueStoppedEvent as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
    }
}

unsafe extern "C" fn AtaPortEnterStateMachine(mut PortData: PATAPORT_PORT_DATA) {
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION =
        ::core::ptr::null_mut::<_ATAPORT_CHANNEL_EXTENSION>();
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = _ATA_SCSI_ADDRESS {
        c2rust_unnamed: _C2RustUnnamed_165 {
            Lun: 0,
            TargetId: 0,
            PathId: 0,
            IsValid: 0,
        },
    };
    ChanExt = CONTAINING_RECORD!(PortData, ATAPORT_CHANNEL_EXTENSION, PortData);
    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION =
            ::core::ptr::null_mut::<_ATAPORT_DEVICE_EXTENSION>();
        let mut PowerIdleCounter: PULONG = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
        DevExt = AtaFdoFindNextDeviceByPath(
            ChanExt,
            &raw mut AtaScsiAddress,
            FALSE,
            AtaPortWorkerThread as *mut core::ffi::c_void,
        );
        if DevExt.is_null() {
            break;
        }
        (*DevExt).Worker.Flags = DEV_WORKER_FLAG_HOLD_REFERENCE;
        (*DevExt).Worker.ResetRetryCount = 0;
        if (*DevExt).Device.QueueFlags as u32 & QUEUE_FLAG_FROZEN_REMOVED != 0 {
            (*DevExt).Worker.Flags |= DEV_WORKER_FLAG_REMOVED;
            continue;
        }
        AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_PORT_BUSY);
        PowerIdleCounter = (*DevExt).Device.PowerIdleCounter;
        if !PowerIdleCounter.is_null() {
            PoSetDeviceBusy(PowerIdleCounter);
        }
    }
    AtaPortWaitForIdle(PortData);
    (*PortData).Worker.Flags = 0;
    (*PortData).Worker.BadDeviceBitmap = 0;
    (*PortData).Worker.ResetRetryCount = 0;
    (*PortData).Worker.StateLoopCount = 0;
}

unsafe extern "C" fn AtaPortExitStateMachine(mut PortData: PATAPORT_PORT_DATA) {
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION = core::ptr::null_mut();
    let mut AtaScsiAddress: ATA_SCSI_ADDRESS = core::mem::zeroed();

    ChanExt = CONTAINING_RECORD!(PortData, ATAPORT_CHANNEL_EXTENSION, PortData);

    AtaScsiAddress.AsULONG = 0;
    loop {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
        let mut OldIrql: KIRQL = 0;

        DevExt = AtaFdoFindNextDeviceByPath(ChanExt, &mut AtaScsiAddress, TRUE, core::ptr::null_mut());
        if DevExt.is_null() {
            break;
        }

        if (*DevExt).Worker.Flags & DEV_WORKER_FLAG_REMOVED != 0 {
            KeAcquireSpinLock(&mut (*ChanExt).PdoListLock, &mut OldIrql);
            if (*DevExt).RemovalPending == 0 {
                (*PortData).Worker.Flags |= WORKER_FLAG_NEED_RESCAN;
            }
            (*DevExt).RemovalPending = TRUE;
            KeReleaseSpinLock(&mut (*ChanExt).PdoListLock, OldIrql);

            AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_REMOVED);
            AtaReqFlushDeviceQueue(&mut (*DevExt).Device);

            AtaDeviceFlushPowerIrpQueue((DevExt).cast());
            KeSetEvent(&mut (*DevExt).Worker.ConfigureEvent, 0, FALSE);

            (*DevExt).Worker.EnumStatus = (DEV_STATUS_NO_DEVICE) as i32;
            KeSetEvent(&mut (*DevExt).Worker.EnumerationEvent, 0, FALSE);
        } else {
            AtaReqThawQueue(DevExt, QUEUE_FLAG_FROZEN_PORT_BUSY);
        }

        if (*DevExt).Worker.Flags & DEV_WORKER_FLAG_HOLD_REFERENCE != 0 {
            (*DevExt).Worker.Flags &= !DEV_WORKER_FLAG_HOLD_REFERENCE;
            IoReleaseRemoveLock(&mut (*DevExt).Common.RemoveLock, AtaPortWorkerThread);
        }
    }

    if (*PortData).Worker.Flags & WORKER_FLAG_COMPLETE_PORT_ENUM_EVENT != 0 {
        (*PortData).Worker.Flags &= !WORKER_FLAG_NEED_RESCAN;

        KeSetEvent(&mut (*PortData).Worker.EnumerationEvent, 0, FALSE);
    }

    if (*PortData).Worker.Flags & WORKER_FLAG_NEED_RESCAN != 0 {
        IoInvalidateDeviceRelations((*ChanExt).Pdo, BusRelations);
    }
}

unsafe extern "C" fn AtaPortWorkerClearSignal(PortData: PATAPORT_PORT_DATA) {
    let mut OldIrql: KIRQL = 0;
    KeAcquireSpinLock((&raw mut (*PortData).Worker.Lock).cast(), &raw mut OldIrql);
    KeClearEvent(&raw mut (*PortData).Worker.ThreadEvent);
    KeReleaseSpinLock((&raw mut (*PortData).Worker.Lock).cast(), OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn AtaPortWorkerThread(StartContext: *mut core::ffi::c_void) {
    let PortData: *mut PATAPORT_PORT_DATA = StartContext as *mut PATAPORT_PORT_DATA;

    loop {
        KeWaitForSingleObject(
            &raw mut (*(*PortData)).Worker.ThreadEvent as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        if (*(*PortData)).PortFlags & PORT_FLAG_EXIT_THREAD != 0 {
            break;
        }
        AtaPortWorkerClearSignal((PortData).cast());
        AtaPortEnterStateMachine((PortData).cast());
        AtaPortRunStateMachine((PortData).cast());
        AtaPortExitStateMachine((PortData).cast());
    }

    PsTerminateSystemThread(STATUS_SUCCESS);
}

#[no_mangle]
pub unsafe extern "C" fn AtaPortNotification(
    NotificationType: PORT_NOTIFICATION_TYPE,
    PortContext: PVOID,
    mut args: ...
) {
    let PortData: PATAPORT_PORT_DATA = PortContext as PATAPORT_PORT_DATA;
    let mut ap: ::core::ffi::VaListImpl;
    ap = args.clone();

    match NotificationType as ::core::ffi::c_uint {
        0 => {
            AtaPortOnRequestComplete(PortData, ap.arg::<ULONG>());
        }
        1 => {
            AtaPortOnResetNotification(PortData, ap.arg::<ULONG>());
        }
        2 => {
            if !((*PortData).InterruptFlags & PORT_INT_FLAG_IGNORE_LINK_IRQ as LONG != 0) {
                AtaPortQueueEvent(
                    PortData,
                    ::core::ptr::null_mut::<_ATAPORT_DEVICE_EXTENSION>(),
                    ::core::ptr::null_mut::<_ATA_DEVICE_REQUEST>(),
                    ACTION_PORT_RESET,
                );
            } else {
                TRACE!(
                    b"CH %lu: Ignore link IRQ\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*PortData).PortNumber,
                );
            }
        }
        3 => {
            AtaPortQueueEvent(
                PortData,
                ::core::ptr::null_mut::<_ATAPORT_DEVICE_EXTENSION>(),
                ap.arg::<PATA_DEVICE_REQUEST>(),
                ACTION_DEVICE_ERROR,
            );
        }
        4 => {
            AtaPortOnAsyncNotification(PortData, ap.arg::<ULONG>());
        }
        _ => {
            ERR!(
                b"CH %lu: Unsupported notification %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*PortData).PortNumber,
                NotificationType as ::core::ffi::c_uint,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortCompleteInternalRequest(
    Request: *mut ATA_DEVICE_REQUEST,
) -> ATA_COMPLETION_ACTION {
    let Device: *mut ATAPORT_IO_CONTEXT = (*Request).Device as *mut ATAPORT_IO_CONTEXT;
    let PortData: *mut ATAPORT_PORT_DATA = (*Device).PortData;

    KeSetEvent(
        &raw mut (*PortData).Worker.CompletionEvent,
        (IO_NO_INCREMENT) as KPRIORITY,
        FALSE,
    );

    return COMPLETE_IRP;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortSendRequest(
    PortData: *mut ATAPORT_PORT_DATA,
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: *mut ATA_DEVICE_REQUEST = &mut (*PortData).Worker.InternalRequest;
    let mut OldIrql: KIRQL = 0;

    if (*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0 {
        INFO!(
            b"CH %lu: Send CDB %u %02x:%02x:%02x:%02x:%02x:%02x:%02x\n\0".as_ptr() as *const i8,
            (*PortData).PortNumber,
            (*DevExt).Device.AtaScsiAddress.TargetId,
            (*Request).Cdb[0],
            (*Request).Cdb[1],
            (*Request).Cdb[2],
            (*Request).Cdb[3],
            (*Request).Cdb[4],
            (*Request).Cdb[5],
            (*Request).Cdb[6],
        );
    } else {
        INFO!(
            b"CH %lu: Send TF %u %02x:%02x:%02x:%02x:%02x:%02x:%02x\n\0".as_ptr() as *const i8,
            (*PortData).PortNumber,
            (*DevExt).Device.AtaScsiAddress.TargetId,
            (*Request).TaskFile.Command,
            (*Request).TaskFile.Feature,
            (*Request).TaskFile.SectorCount,
            (*Request).TaskFile.LowLba,
            (*Request).TaskFile.MidLba,
            (*Request).TaskFile.HighLba,
            (*Request).TaskFile.DriveSelect,
        );
    }

    KeClearEvent(&mut (*PortData).Worker.CompletionEvent);

    (*Request).Flags |= REQUEST_FLAG_INTERNAL;
    (*Request).Device = &mut (*DevExt).Device as *mut _ as PATA_IO_CONTEXT_COMMON;

    (*PortData).Worker.OldRequest = (*PortData).Slots[0];
    (*PortData).Slots[0] = Request;

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &mut OldIrql);

    KeAcquireSpinLockAtDpcLevel(&mut (*PortData).QueueLock);
    (*PortData).ActiveTimersBitmap |= 1 << 0;
    KeReleaseSpinLockFromDpcLevel(&mut (*PortData).QueueLock);

    AtaReqSendRequest(Request);

    KeLowerIrql(OldIrql);

    KeWaitForSingleObject(
        &mut (*PortData).Worker.CompletionEvent,
        _KWAIT_REASON_Executive,
        (0i32) as KPROCESSOR_MODE,
        FALSE,
        core::ptr::null_mut(),
    );

    KeAcquireSpinLock(&mut (*PortData).QueueLock, &mut OldIrql);
    (*PortData).ActiveTimersBitmap = 0;
    KeReleaseSpinLock(&mut (*PortData).QueueLock, OldIrql);

    (*PortData).Slots[0] = (*PortData).Worker.OldRequest;

    if ((*Request).SrbStatus) as u32 == SRB_STATUS_TIMEOUT
        || ((*Request).SrbStatus) as u32 == SRB_STATUS_BUS_RESET
    {
        return STATUS_ADAPTER_HARDWARE_ERROR;
    }

    if ((*Request).SrbStatus) as u32 == SRB_STATUS_SUCCESS {
        return STATUS_SUCCESS;
    }

    return STATUS_IO_DEVICE_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortTimeout(
    PortData: *mut ATAPORT_PORT_DATA,
    Slot: u32,
) {
    let mut Request: *mut ATA_DEVICE_REQUEST = core::ptr::null_mut();
    let mut DevExt: *mut ATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();

    Request = (*PortData).Slots[Slot as usize];
    ASSERT_REQUEST!(!Request.is_null());

    DevExt = CONTAINING_RECORD!(
        (*Request).Device,
        ATAPORT_DEVICE_EXTENSION,
        Device,
    );

    if (*Request).Flags & REQUEST_FLAG_PACKET_COMMAND != 0 {
    } else {
    }

    (*Request).SrbStatus = (SRB_STATUS_TIMEOUT) as u8;

    (*Request).Output.__bindgen_anon_1.Status = (IDE_STATUS_ERROR) as u8;
    (*Request).Output.__bindgen_anon_2.Error = (IDE_ERROR_COMMAND_ABORTED) as u8;

    AtaPortQueueEvent(
        PortData,
        core::ptr::null_mut(),
        Request,
        ACTION_PORT_RESET | ACTION_DEVICE_ERROR,
    );
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortSignalWorkerThread(PortData: *mut PATAPORT_PORT_DATA) {
    let mut OldIrql: KIRQL = 0;
    KeAcquireSpinLock((&raw mut (*(*PortData)).Worker.Lock).cast(), &raw mut OldIrql);
    KeSetEvent(
        &raw mut (*(*PortData)).Worker.ThreadEvent,
        (IO_NO_INCREMENT) as KPRIORITY,
        FALSE,
    );
    KeReleaseSpinLock((&raw mut (*(*PortData)).Worker.Lock).cast(), OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn AtaPortWorkerSignalDpc(
    Dpc: PKDPC,
    DeferredContext: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {
    let PortData: *mut ATAPORT_PORT_DATA = DeferredContext as *mut ATAPORT_PORT_DATA;

    UNREFERENCED_PARAMETER!(Dpc);
    UNREFERENCED_PARAMETER!(SystemArgument1);
    UNREFERENCED_PARAMETER!(SystemArgument2);

    AtaPortSignalWorkerThread((PortData).cast());
}
#[no_mangle]
pub unsafe extern "C" fn AtaDeviceQueueEvent(
    PortData: *mut ATAPORT_PORT_DATA,
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    Action: ATA_PORT_ACTION,
) {
    let mut OldIrql: KIRQL;

    OldIrql = KeAcquireInterruptSpinLock((*PortData).InterruptObject);
    AtaPortQueueEvent(PortData, DevExt, core::ptr::null_mut(), Action);
    KeReleaseInterruptSpinLock((*PortData).InterruptObject, OldIrql);
}
