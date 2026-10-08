use crate::prelude::*;
use ntkrnl_sys::ddk::{_IO_STATUS_BLOCK, IRP, PDEVICE_OBJECT, PDEVICE_POWER_STATE, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, POWER_STATE};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

unsafe extern "C" fn AtaPowerCompleteIrp(
    CommonExt: PATAPORT_COMMON_EXTENSION,
    Irp: PIRP,
    Status: NTSTATUS,
) -> NTSTATUS {
    PoStartNextPowerIrp(Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

    IoReleaseRemoveLock(&mut (*CommonExt).RemoveLock, (Irp).cast());

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AtaDeviceFlushPowerIrpQueue(
    DevExt: PATAPORT_DEVICE_EXTENSION,
) {
    loop {
        let mut Entry: PLIST_ENTRY;
        let mut PowerIrp: PIRP;
        let mut OldLevel: KIRQL = core::mem::zeroed();

        KeAcquireSpinLock((&mut (*DevExt).Device.QueueLock as *mut _ as *mut usize).cast(), &mut OldLevel);
        if IsListEmpty(&mut (*DevExt).PowerIrpQueueList) != 0 {
            Entry = core::ptr::null_mut();
        } else {
            Entry = RemoveHeadList(&mut (*DevExt).PowerIrpQueueList);
        }
        KeReleaseSpinLock((&mut (*DevExt).Device.QueueLock as *mut _ as *mut usize).cast(), OldLevel);

        if Entry.is_null() {
            break;
        }

        PowerIrp = CONTAINING_RECORD!(Entry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);

        AtaPowerCompleteIrp(&mut (*DevExt).Common, PowerIrp, STATUS_NO_SUCH_DEVICE);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaDeviceGetFlushCacheCommand(
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> UCHAR {
    let mut Command: u8 = 0;

    if !IS_ATAPI!(&(*DevExt).Device) {
        if ((*DevExt).Device.DeviceFlags & DEVICE_LBA48) != 0
            && AtaDevHasFlushCacheExt(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0
        {
            Command = (IDE_COMMAND_FLUSH_CACHE_EXT) as u8;
        } else if AtaDevHasFlushCache(&raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData) != 0 {
            Command = (IDE_COMMAND_FLUSH_CACHE) as u8;
        }
    }

    return Command;
}

unsafe extern "C" fn AtaDeviceCompletePowerIrp(
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) {
    let mut PowerState: POWER_STATE = POWER_STATE {
        SystemState: PowerSystemUnspecified,
    };
    PowerState.DeviceState = (*DevExt).Common.DevicePowerState;
    PoSetPowerState(
        (*DevExt).Common.Self_,
        DevicePowerState,
        PowerState,
    );
    AtaPowerCompleteIrp(&mut (*DevExt).Common, Irp, STATUS_SUCCESS);
}

unsafe extern "C" fn AtaDeviceFlushCache(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;
    let mut Command: UCHAR;

    Command = AtaDeviceGetFlushCacheCommand((DevExt).cast());
    if Command == 0 {
        return STATUS_SUCCESS;
    }

    (*Request).Flags = 0;
    (*Request).TimeOut = 30;
    RtlZeroMemory!(
        &raw mut (*Request).__bindgen_anon_2.TaskFile as *mut core::ffi::c_void,
        core::mem::size_of::<ATA_TASKFILE>(),
    );
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = Command;

    return AtaPortSendRequest(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceSetIdleMode(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;

    (*Request).Flags = 0;
    (*Request).TimeOut = 20;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_2.Feature = 0;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_IDLE_IMMEDIATE) as u8;

    return AtaPortSendRequest(PortData, DevExt);
}

unsafe extern "C" fn AtaDeviceSetStandbyMode(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let Request: PATA_DEVICE_REQUEST = &raw mut (*PortData).Worker.InternalRequest;

    (*Request).Flags = 0;
    (*Request).TimeOut = 20;
    (*Request).__bindgen_anon_2.TaskFile.__bindgen_anon_1.Command = (IDE_COMMAND_STANDBY_IMMEDIATE) as u8;

    return AtaPortSendRequest(PortData, DevExt);
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortDeviceProcessPowerChange(
    PortData: *mut ATAPORT_PORT_DATA,
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY;
    let mut PowerIrp: PIRP;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut NewState: DEVICE_POWER_STATE;
    let mut OldLevel: KIRQL = 0;
    let mut Status: NTSTATUS;

    KeAcquireSpinLock((&raw mut (*DevExt).Device.QueueLock).cast(), &raw mut OldLevel);
    if IsListEmpty(&raw mut (*DevExt).PowerIrpQueueList) != 0 {
        Entry = core::ptr::null_mut();
    } else {
        Entry = RemoveHeadList(&raw mut (*DevExt).PowerIrpQueueList);
    }
    KeReleaseSpinLock((&raw mut (*DevExt).Device.QueueLock).cast(), OldLevel);
    if Entry.is_null() {
        return STATUS_SUCCESS;
    }

    PowerIrp = CONTAINING_RECORD!(Entry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);

    IoStack = IoGetCurrentIrpStackLocation(PowerIrp);

    NewState = (*IoStack).Parameters.Power.State.DeviceState;
    if NewState as u32 == (*DevExt).Common.DevicePowerState as u32 {
        AtaDeviceCompletePowerIrp(DevExt, PowerIrp);
        return STATUS_SUCCESS;
    }

    if NewState as u32 != PowerDeviceD3 as u32 {
        AtaReqThawQueue(DevExt, QUEUE_FLAG_FROZEN_POWER);

        Status = AtaDeviceSetIdleMode(PortData, DevExt);
        if Status == STATUS_ADAPTER_HARDWARE_ERROR {
            return Status;
        }

        _InterlockedOr(&raw mut (*DevExt).Worker.EventsPending, (ACTION_DEVICE_CONFIG) as i32);
        _InterlockedOr(&raw mut (*PortData).Worker.EventsPending, (ACTION_DEVICE_CONFIG) as i32);
    } else {
        AtaReqFreezeQueue(DevExt, QUEUE_FLAG_FROZEN_POWER);

        Status = AtaDeviceFlushCache(PortData, DevExt);
        if Status == STATUS_ADAPTER_HARDWARE_ERROR {
            return Status;
        }

        Status = AtaDeviceSetStandbyMode(PortData, DevExt);
        if Status == STATUS_ADAPTER_HARDWARE_ERROR {
            return Status;
        }
    }

    AtaDeviceCompletePowerIrp(DevExt, PowerIrp);

    (*DevExt).Common.DevicePowerState = NewState;
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortCheckDevicePowerState(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if (*DevExt).Common.DevicePowerState == (PowerDeviceD0) as _ {
        return STATUS_SUCCESS;
    }

    INFO!(
        b"CH %lu: Powering up idle device '%s'\n\0".as_ptr() as *const i8,
        (*PortData).PortNumber,
        (*DevExt).FriendlyName.as_ptr(),
    );

    Status = AtaDeviceSetIdleMode(PortData, DevExt);
    if !NT_SUCCESS!(Status) {
        /*
         * If the power on device has failed, the next command can still work.
         * The media access will result in a transition from the PM1:Idle/PM2:Standby state
         * to the PM0:Active state by the ATA spec.
         */
    } else {
        (*DevExt).Common.DevicePowerState = (PowerDeviceD0) as _;
    }

    return Status;
}
unsafe extern "C" fn AtaPdoQueuePowerIrp(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) {
    let mut OldLevel: KIRQL = 0;

    KeAcquireSpinLock(
        (&mut (*DevExt).Device.QueueLock as *mut _ as *mut usize).cast(),
        &mut OldLevel,
    );
    InsertTailList(
        &mut (*DevExt).PowerIrpQueueList,
        &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
    );
    KeReleaseSpinLock(
        (&mut (*DevExt).Device.QueueLock as *mut _ as *mut usize).cast(),
        OldLevel,
    );

    AtaDeviceQueueEvent((*DevExt).Device.PortData, (DevExt).cast(), ACTION_DEVICE_POWER);
}

unsafe extern "system" fn AtaPdoPowerUpCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let DevExt: *mut ATAPORT_DEVICE_EXTENSION = Context as *mut ATAPORT_DEVICE_EXTENSION;
    let Status: NTSTATUS;

    UNREFERENCED_PARAMETER!(DeviceObject);

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    INFO!("Powering up device, lower driver status %lx\n", Status);

    if !NT_SUCCESS!(Status) {
        PoStartNextPowerIrp(Irp);

        IoReleaseRemoveLock(&mut (*DevExt).Common.RemoveLock, (Irp).cast());
        return STATUS_CONTINUE_COMPLETION;
    }

    AtaPdoQueuePowerIrp((DevExt).cast(), Irp);

    return STATUS_MORE_PROCESSING_REQUIRED;
}

unsafe extern "C" fn AtaPdoPowerSetDevicePowerState(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut NewState: DEVICE_POWER_STATE = (PowerDeviceD0) as _;
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    NewState = (*IoStack).Parameters.Power.State.DeviceState;
    IoMarkIrpPending(Irp);
    if (NewState as u32) < (*DevExt).Common.DevicePowerState as u32 {
        let mut ChanExt: PATAPORT_CHANNEL_EXTENSION = ((*DevExt).Common.__bindgen_anon_1.FdoExt).cast();
        IoCopyCurrentIrpStackLocationToNext(Irp);
        IoSetCompletionRoutine(
            Irp,
            Some(AtaPdoPowerUpCompletionRoutine as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS),
            DevExt as PVOID,
            TRUE,
            TRUE,
            TRUE,
        );
        PoCallDriver((*ChanExt).Common.Self_, Irp);
    } else {
        INFO!(b"Powering down device\n\0".as_ptr() as *const i8);
        AtaPdoQueuePowerIrp((DevExt).cast(), Irp);
    }
    return STATUS_PENDING;
}

unsafe extern "system" fn AtaPdoPowerSetSysStateWithDevStateComplete(
    DeviceObject: PDEVICE_OBJECT,
    MinorFunction: u8,
    PowerState: POWER_STATE,
    Context: *mut core::ffi::c_void,
    IoStatus: PIO_STATUS_BLOCK,
) {
    let DevExt = (*DeviceObject).DeviceExtension as *mut ATAPORT_DEVICE_EXTENSION;
    let Irp = Context as PIRP;

    UNREFERENCED_PARAMETER!(MinorFunction);

    if (*DevExt).Common.SystemPowerState as u32 != PowerState.SystemState as u32 {
        (*DevExt).Common.SystemPowerState = PowerState.SystemState;
        PoSetPowerState(DeviceObject, PowerState.SystemState, PowerState);
    }

    AtaPowerCompleteIrp(
        core::ptr::addr_of_mut!((*DevExt).Common),
        Irp,
        (*IoStatus).__bindgen_anon_1.Status,
    );
}

unsafe extern "C" fn AtaPdoPowerGetDevStateForSysState(
    SystemState: SYSTEM_POWER_STATE,
    DeviceState: PDEVICE_POWER_STATE,
) {
    if SystemState == PowerSystemWorking {
        *DeviceState = (PowerDeviceD0) as _;
    } else {
        *DeviceState = PowerDeviceD3;
    }
}

unsafe extern "C" fn AtaPdoPowerSetSystemPowerState(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut NewPowerState: POWER_STATE = POWER_STATE {
        SystemState: PowerSystemWorking,
    };
    let mut NewSystemState: SYSTEM_POWER_STATE = PowerSystemWorking;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    NewSystemState = (*IoStack).Parameters.Power.State.SystemState;

    IoMarkIrpPending(Irp);

    AtaPdoPowerGetDevStateForSysState(NewSystemState, &raw mut NewPowerState.DeviceState);
    Status = PoRequestPowerIrp(
        (*DevExt).Common.Self_ as *mut _DEVICE_OBJECT,
        IRP_MN_SET_POWER,
        NewPowerState,
        Some(
            AtaPdoPowerSetSysStateWithDevStateComplete,
        ),
        Irp as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };

        IoStatusBlock.__bindgen_anon_1.Status = Status;
        AtaPdoPowerSetSysStateWithDevStateComplete(
            (*DevExt).Common.Self_,
            IRP_MN_SET_POWER,
            NewPowerState,
            (Irp).cast(),
            &raw mut IoStatusBlock,
        );
    }

    return Status;
}

unsafe extern "C" fn AtaPdoPower(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    UNREFERENCED_PARAMETER!(AtaPdoPowerSetSystemPowerState);
    UNREFERENCED_PARAMETER!(AtaPdoPowerGetDevStateForSysState);
    UNREFERENCED_PARAMETER!(AtaPdoPowerSetSysStateWithDevStateComplete);
    UNREFERENCED_PARAMETER!(AtaPdoPowerSetDevicePowerState);
    UNREFERENCED_PARAMETER!(AtaPdoPowerUpCompletionRoutine);
    UNREFERENCED_PARAMETER!(AtaPdoQueuePowerIrp);

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    PoStartNextPowerIrp(Irp);
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
    return Status;
}

unsafe extern "C" fn AtaFdoPower(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = IoAcquireRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        PoStartNextPowerIrp(Irp);

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);
        return Status;
    }

    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    Status = PoCallDriver((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

    IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn AtaDispatchPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if IS_FDO!((*DeviceObject).DeviceExtension) != false {
        return AtaFdoPower(((*DeviceObject).DeviceExtension).cast(), Irp);
    } else {
        return AtaPdoPower(((*DeviceObject).DeviceExtension).cast(), Irp);
    }
}
