use crate::types::*;
use crate::glue::*;
use crate::cb;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_CAPABILITIES, IoCopyCurrentIrpStackLocationToNext, IoCsqInsertIrp, IoCsqRemoveNextIrp, IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoReleaseCancelSpinLock, IoSetCancelRoutine, KeInsertQueueDpc, LARGE_INTEGER, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, POWER_STATE, PRKDPC};
use core::ffi::c_void;

extern "system" {
    fn USBPORT_FlushController(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_Wait(MiniPortExtension: PVOID, MillisecondsTimeout: ULONG) -> NTSTATUS;
    fn USBPORT_MiniportInterrupts(FdoDevice: PDEVICE_OBJECT, Enable: BOOLEAN);
    fn USBPORT_NukeAllEndpoints(FdoDevice: PDEVICE_OBJECT);
}
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// (no translatable declarations)

const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CompletePdoWaitWake(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PdoDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION = core::ptr::null_mut();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    PdoDevice = (*FdoExtension).RootHubPdo;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    Irp = (*PdoExtension).WakeIrp;

    if !Irp.is_null() && !IoSetCancelRoutine(Irp, None).is_none() {
        (*PdoExtension).WakeIrp = core::ptr::null_mut();
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
            OldIrql,
        );

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = (0 as _);
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

        return;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HcWakeDpc(
    Dpc: PRKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    USBPORT_CompletePdoWaitWake(DeferredContext as PDEVICE_OBJECT);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HcQueueWakeDpc(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    KeInsertQueueDpc(
        core::ptr::addr_of_mut!((*FdoExtension).HcWakeDpc),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CompletePendingIdleIrp(PdoDevice: PDEVICE_OBJECT) {
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Irp: PIRP;

    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Irp = IoCsqRemoveNextIrp(
        core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsq),
        core::ptr::null_mut(),
    );

    if !Irp.is_null() {
        InterlockedDecrement(core::ptr::addr_of_mut!((*FdoExtension).IdleLockCounter));

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DoSetPowerD0(FdoDevice: PDEVICE_OBJECT) {
    return;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SuspendController(FdoDevice: PDEVICE_OBJECT) {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Packet: PUSBPORT_REGISTRATION_PACKET = core::ptr::null_mut();

    if DbgPrint(
        b"(%s:%d) USBPORT_SuspendController \n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbport/power.c\0"
            as *const u8 as *const core::ffi::c_char,
        117 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/power.c\0"
                as *const u8 as *const core::ffi::c_char,
            117 as core::ffi::c_int,
        );
    }

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    (*FdoExtension).TimerFlags |= USBPORT_TMFLAG_RH_SUSPENDED as core::ffi::c_ulong;

    USBPORT_FlushController(FdoDevice);

    if (*FdoExtension).Flags as core::ffi::c_ulong & USBPORT_FLAG_HC_SUSPEND as core::ffi::c_ulong
        != 0
    {
        return;
    }

    (*FdoExtension).TimerFlags |= USBPORT_TMFLAG_HC_SUSPENDED as core::ffi::c_ulong;

    if (*FdoExtension).MiniPortFlags as core::ffi::c_ulong
        & USBPORT_MPFLAG_INTERRUPTS_ENABLED as core::ffi::c_ulong
        != 0
    {
        (*FdoExtension).MiniPortFlags |= USBPORT_MPFLAG_SUSPENDED as core::ffi::c_ulong;
        USBPORT_Wait(FdoDevice as PVOID, 10 as ULONG);
        ((*Packet).SuspendController.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
        );
    }

    (*FdoExtension).Flags |= USBPORT_FLAG_HC_SUSPEND as core::ffi::c_ulong;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ResumeController(FdoDevice: PDEVICE_OBJECT) -> NTSTATUS {

    const USBPORT_FLAG_HC_SUSPEND: u32 = 0x00000100; // FIXED 2026-07-05: translation invented 0x10000; verified vs usbport.h/usbmport.h
    const USBPORT_TMFLAG_HC_SUSPENDED: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USBPORT_TMFLAG_RH_SUSPENDED: u32 = 0x00000008;
    const USBPORT_TMFLAG_HC_RESUME: u32 = 0x00000004; // FIXED 2026-07-05: translation invented 0x10; verified vs usbport.h/usbmport.h
    const USBPORT_MPFLAG_SUSPENDED: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h

    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut OldIrql: KIRQL = 0;
    let MpStatus: MPSTATUS;

    // DPRINT1("USBPORT_ResumeController: ... \n");

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet) as PUSBPORT_REGISTRATION_PACKET;

    if (*FdoExtension).Flags & USBPORT_FLAG_HC_SUSPEND == 0 {
        return Status;
    }

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize), core::ptr::addr_of_mut!(OldIrql));

    (*FdoExtension).TimerFlags &= !(USBPORT_TMFLAG_HC_SUSPENDED | USBPORT_TMFLAG_RH_SUSPENDED);

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize), OldIrql);

    if (*FdoExtension).MiniPortFlags & USBPORT_MPFLAG_SUSPENDED == 0 {
        (*FdoExtension).Flags &= !USBPORT_FLAG_HC_SUSPEND;
        return Status;
    }

    (*FdoExtension).MiniPortFlags &= !USBPORT_MPFLAG_SUSPENDED;

    if ((*Packet).ResumeController.expect("non-null function pointer")((*FdoExtension).MiniPortExt)) == 0 {
        Status = (USBPORT_Wait(FdoDevice as PVOID, 100 as ULONG) as i32);

        (*FdoExtension).Flags &= !USBPORT_FLAG_HC_SUSPEND;
        return Status;
    }

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize), core::ptr::addr_of_mut!(OldIrql));
    (*FdoExtension).TimerFlags |= USBPORT_TMFLAG_HC_SUSPENDED | USBPORT_TMFLAG_HC_RESUME;
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize), OldIrql);

    USBPORT_MiniportInterrupts(FdoDevice, 0 as BOOLEAN);

    (*Packet).StopController.expect("non-null function pointer")((*FdoExtension).MiniPortExt, 1 as BOOLEAN);

    USBPORT_NukeAllEndpoints(FdoDevice);

    RtlZeroMemory((*FdoExtension).MiniPortExt as PVOID, (*Packet).MiniPortExtensionSize as usize);

    RtlZeroMemory((*FdoExtension).UsbPortResources.StartVA as PVOID, (*Packet).MiniPortResourcesSize as usize);

    (*FdoExtension).UsbPortResources.IsChirpHandled = 1 as BOOLEAN;

    MpStatus = (*Packet).StartController.expect("non-null function pointer")(
        (*FdoExtension).MiniPortExt,
        core::ptr::addr_of_mut!((*FdoExtension).UsbPortResources),
    );

    (*FdoExtension).UsbPortResources.IsChirpHandled = 0 as BOOLEAN;

    if MpStatus == 0 {
        USBPORT_MiniportInterrupts(FdoDevice, 1 as BOOLEAN);
    }

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize), core::ptr::addr_of_mut!(OldIrql));

    (*FdoExtension).TimerFlags &= !(USBPORT_TMFLAG_HC_SUSPENDED | USBPORT_TMFLAG_HC_RESUME | USBPORT_TMFLAG_RH_SUSPENDED);

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize), OldIrql);

    Status = (USBPORT_Wait(FdoDevice as PVOID, 100 as ULONG) as i32);

    (*FdoExtension).Flags &= !USBPORT_FLAG_HC_SUSPEND;

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PdoDevicePowerState(
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut State: POWER_STATE;

    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    State = (*IoStack).Parameters.Power.State;

    // DPRINT1("USBPORT_PdoDevicePowerState: Irp - %p, State - %x\n", Irp, State.DeviceState);

    if State.DeviceState as u32 == PowerDeviceD0 as u32 {
        if (*FdoExtension).CommonExtension.DevicePowerState as u32 == PowerDeviceD0 as u32 {
            // FIXME FdoExtension->Flags
            while (*FdoExtension).SetPowerLockCounter != 0 {
                USBPORT_Wait(FdoDevice as PVOID, 10 as ULONG);
            }

            USBPORT_ResumeController(FdoDevice);

            (*PdoExtension).CommonExtension.DevicePowerState = PowerDeviceD0;

            USBPORT_CompletePdoWaitWake(FdoDevice);
            USBPORT_CompletePendingIdleIrp(PdoDevice);
        } else {
            // DPRINT1("USBPORT_PdoDevicePowerState: FdoExtension->Flags - %lx\n", FdoExtension->Flags);

            DbgBreakPoint();
            Status = STATUS_UNSUCCESSFUL;
        }
    } else if State.DeviceState as u32 == PowerDeviceD1 as u32
        || State.DeviceState as u32 == PowerDeviceD2 as u32
        || State.DeviceState as u32 == PowerDeviceD3 as u32
    {
        (*FdoExtension).TimerFlags |= USBPORT_TMFLAG_WAKE as ULONG;
        USBPORT_SuspendController(FdoDevice);
        (*PdoExtension).CommonExtension.DevicePowerState = State.DeviceState;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CancelPendingWakeIrp(
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION = core::ptr::null_mut();

    IoReleaseCancelSpinLock((*Irp).CancelIrql);
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoExtension = (*(*PdoExtension).FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    if (*PdoExtension).WakeIrp == Irp {
        (*PdoExtension).WakeIrp = core::ptr::null_mut();
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
        OldIrql,
    );

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PdoPower(
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    const USBPORT_FLAG_HC_STARTED: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USBPORT_TMFLAG_WAKE: u32 = 0x00000020; // FIXED 2026-07-05: translation invented 0x8; verified vs usbport.h/usbmport.h
    const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as NTSTATUS;

    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let FdoDevice: PDEVICE_OBJECT;
    let IoStack: PIO_STACK_LOCATION;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;

    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    match (*IoStack).MinorFunction {
        IRP_MN_WAIT_WAKE => {
            if (*FdoExtension).Flags & USBPORT_FLAG_HC_STARTED == 0 {
                /* The device does not support wake-up */
                Status = STATUS_NOT_SUPPORTED;
            } else {
                KeAcquireSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
                    core::ptr::addr_of_mut!(OldIrql),
                );

                IoSetCancelRoutine(Irp, cb!(USBPORT_CancelPendingWakeIrp));

                /* Check if the IRP has been cancelled */
                let mut cancelled = false;
                if (*Irp).Cancel != 0 {
                    if !IoSetCancelRoutine(Irp, None).is_none() {
                        /* IRP has been cancelled, release cancel spinlock */
                        KeReleaseSpinLock(
                            (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
                            OldIrql,
                        );

                        /* IRP is cancelled */
                        Status = STATUS_CANCELLED;
                        cancelled = true;
                    }
                }

                if !cancelled {
                    if (*PdoExtension).WakeIrp.is_null() {
                        /* The driver received the IRP
                           and is waiting for the device to signal wake-up. */
                        IoMarkIrpPending(Irp);
                        (*PdoExtension).WakeIrp = Irp;

                        KeReleaseSpinLock(
                            (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
                            OldIrql,
                        );
                        return STATUS_PENDING;
                    } else {
                        /* An IRP_MN_WAIT_WAKE request is already pending */
                        if !IoSetCancelRoutine(Irp, None).is_none() {
                            KeReleaseSpinLock(
                                (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
                                OldIrql,
                            );
                            PoStartNextPowerIrp(Irp);
                            Status = STATUS_DEVICE_BUSY;
                        } else {
                            DbgBreakPoint();
                            KeReleaseSpinLock(
                                (core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock) as *mut usize),
                                OldIrql,
                            );
                            return Status;
                        }
                    }
                }
            }
        }

        IRP_MN_POWER_SEQUENCE => {
            PoStartNextPowerIrp(Irp);
        }

        IRP_MN_SET_POWER => {
            if (*IoStack).Parameters.Power.Type == (DevicePowerState as i32 as u32) {
                Status = USBPORT_PdoDevicePowerState(PdoDevice, Irp);
                PoStartNextPowerIrp(Irp);
            } else {
                if (*IoStack).Parameters.Power.State.SystemState as u32
                    == PowerSystemWorking as u32
                {
                    (*FdoExtension).TimerFlags |= USBPORT_TMFLAG_WAKE;
                } else {
                    (*FdoExtension).TimerFlags &= !USBPORT_TMFLAG_WAKE;
                }

                Status = STATUS_SUCCESS;

                PoStartNextPowerIrp(Irp);
            }
        }

        IRP_MN_QUERY_POWER => {
            Status = STATUS_SUCCESS;
            PoStartNextPowerIrp(Irp);
        }

        _ => {
            PoStartNextPowerIrp(Irp);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HcWake(
    mut FdoDevice: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBPORT_HcWake: UNIMPLEMENTED. FIXME. \n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbport/power.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        465 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbport/power.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            465 as ::core::ffi::c_int,
        );
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DevicePowerState(
    mut FdoDevice: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBPORT_DevicePowerState: UNIMPLEMENTED. FIXME. \n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbport/power.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        474 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbport/power.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            474 as ::core::ffi::c_int,
        );
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SystemPowerState(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBPORT_SystemPowerState: UNIMPLEMENTED. FIXME. \n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbport/power.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        483 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbport/power.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            483 as ::core::ffi::c_int,
        );
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FdoPower(
    mut FdoDevice: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).MinorFunction {
        IRP_MN_WAIT_WAKE => {
            Status = USBPORT_HcWake(FdoDevice, Irp);
            return Status;
        }
        IRP_MN_POWER_SEQUENCE => {}
        IRP_MN_SET_POWER => {
            if (*IoStack).Parameters.Power.Type as u32 == DevicePowerState as i32 as u32 {
                Status = USBPORT_DevicePowerState(FdoDevice, Irp);
            } else {
                Status = USBPORT_SystemPowerState(FdoDevice, Irp);
            }

            if Status != STATUS_PENDING {
                // break;
            } else {
                return Status;
            }
        }
        IRP_MN_QUERY_POWER => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }
        _ => {}
    }

    IoCopyCurrentIrpStackLocationToNext(Irp);
    PoStartNextPowerIrp(Irp);
    return PoCallDriver((*FdoExtension).CommonExtension.LowerDevice, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DoIdleNotificationCallback(Context: PVOID) {
    let mut IoStack: PIO_STACK_LOCATION;
    let FdoDevice: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut NextIrp: PIRP;
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let IdleQueueItem: PTIMER_WORK_QUEUE_ITEM;
    let PdoDevice: PDEVICE_OBJECT;
    let mut IdleCallbackInfo: PUSB_IDLE_CALLBACK_INFO;
    let mut OldIrql: KIRQL = 0;

    IdleQueueItem = Context as PTIMER_WORK_QUEUE_ITEM;

    FdoDevice = (*IdleQueueItem).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    PdoDevice = (*FdoExtension).RootHubPdo;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    // KeQuerySystemTime(&CurrentTime) expanded inline
    *((core::ptr::addr_of_mut!(CurrentTime)) as *mut u64) = *(SharedSystemTimePtr() as *mut u64);

    if (*FdoExtension).IdleTime.QuadPart == 0
        || ((CurrentTime.QuadPart - (*FdoExtension).IdleTime.QuadPart) / 10000) >= 500
    {
        if (*PdoExtension).CommonExtension.DevicePowerState as u32
            == PowerDeviceD0 as u32
            && (*FdoExtension).CommonExtension.DevicePowerState as u32
                == PowerDeviceD0 as u32
        {
            NextIrp = IoCsqRemoveNextIrp(
                core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsq),
                core::ptr::null_mut(),
            );

            if !NextIrp.is_null() {
                IoStack = IoGetCurrentIrpStackLocation(NextIrp);
                IdleCallbackInfo = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer
                    as PUSB_IDLE_CALLBACK_INFO;

                if !IdleCallbackInfo.is_null() && (*IdleCallbackInfo).IdleCallback.is_some() {
                    ((*IdleCallbackInfo).IdleCallback.expect("non-null function pointer"))(
                        (*IdleCallbackInfo).IdleContext,
                    );
                }

                if (*NextIrp).Cancel != 0 {
                    InterlockedDecrement(
                        core::ptr::addr_of_mut!((*FdoExtension).IdleLockCounter) as *mut i32,
                    );

                    (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
                    (*NextIrp).IoStatus.Information = 0;
                    IoCompleteRequest(NextIrp, IO_NO_INCREMENT as i8);
                } else {
                    IoCsqInsertIrp(
                        core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsq),
                        NextIrp,
                        core::ptr::null_mut(),
                    );
                }
            }
        }
    }

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    (*FdoExtension).TimerFlags &= !(USBPORT_TMFLAG_IDLE_QUEUEITEM_ON as u32);
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock) as *mut usize),
        OldIrql,
    );

    ExFreePoolWithTag(IdleQueueItem as PVOID, USB_PORT_TAG as u32);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_IdleNotification(
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut LockCounter: LONG;
    let mut Status: NTSTATUS = STATUS_PENDING;

    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    LockCounter =
        InterlockedIncrement(core::ptr::addr_of_mut!((*FdoExtension).IdleLockCounter)) as LONG;

    if LockCounter != 0 {
        if Status != STATUS_PENDING {
            InterlockedDecrement(core::ptr::addr_of_mut!((*FdoExtension).IdleLockCounter));

            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

            return Status;
        }

        Status = STATUS_DEVICE_BUSY;
    }

    if Status != STATUS_PENDING {
        InterlockedDecrement(core::ptr::addr_of_mut!((*FdoExtension).IdleLockCounter));

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

        return Status;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PENDING;
    IoMarkIrpPending(Irp);

    *(core::ptr::addr_of_mut!((*FdoExtension).IdleTime) as *mut u64) =
        *(0xFFFFFFFFFF000014u64 as usize as *const u64); // KeQuerySystemTime: SharedUserData->SystemTime

    IoCsqInsertIrp(
        core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsq),
        Irp,
        core::ptr::null_mut(),
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AdjustDeviceCapabilities(
    FdoDevice: PDEVICE_OBJECT,
    PdoDevice: PDEVICE_OBJECT,
) {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let Capabilities: PDEVICE_CAPABILITIES;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    Capabilities = core::ptr::addr_of_mut!((*PdoExtension).Capabilities) as PDEVICE_CAPABILITIES;

    RtlCopyMemory(
        Capabilities as *mut core::ffi::c_void,
        core::ptr::addr_of!((*FdoExtension).Capabilities) as *const core::ffi::c_void,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );

    (*Capabilities).set_DeviceD1(0);
    (*Capabilities).set_DeviceD2(1);

    (*Capabilities).set_Removable(0);
    (*Capabilities).set_UniqueID(0);

    (*Capabilities).set_WakeFromD0(1);
    (*Capabilities).set_WakeFromD1(0);
    (*Capabilities).set_WakeFromD2(1);
    (*Capabilities).set_WakeFromD3(0);

    (*Capabilities).Address = 0;
    (*Capabilities).UINumber = 0;

    if (*Capabilities).SystemWake as u32 == PowerSystemUnspecified as i32 as u32 {
        (*Capabilities).SystemWake = PowerSystemWorking;
    }

    (*Capabilities).DeviceWake = PowerDeviceD2;

    (*Capabilities).DeviceState[PowerSystemSleeping1 as i32 as usize] = PowerDeviceD3;
    (*Capabilities).DeviceState[PowerSystemSleeping2 as i32 as usize] = PowerDeviceD3;
    (*Capabilities).DeviceState[PowerSystemSleeping3 as i32 as usize] = PowerDeviceD3;
    (*Capabilities).DeviceState[PowerSystemHibernate as i32 as usize] = PowerDeviceD3;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort power handling functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// VOID
// NTAPI
// USBPORT_CompletePdoWaitWake(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PIRP Irp;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_CompletePdoWaitWake: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     PdoDevice = FdoExtension->RootHubPdo;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->PowerWakeSpinLock, &OldIrql);
//
//     Irp = PdoExtension->WakeIrp;
//
//     if (Irp && IoSetCancelRoutine(Irp, NULL))
//     {
//         PdoExtension->WakeIrp = NULL;
//         KeReleaseSpinLock(&FdoExtension->PowerWakeSpinLock, OldIrql);
//
//         DPRINT("USBPORT_CompletePdoWaitWake: Complete Irp - %p\n", Irp);
//
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//         return;
//     }
//
//     KeReleaseSpinLock(&FdoExtension->PowerWakeSpinLock, OldIrql);
// }
//
// VOID
// NTAPI
// USBPORT_HcWakeDpc(IN PRKDPC Dpc,
//                   IN PVOID DeferredContext,
//                   IN PVOID SystemArgument1,
//                   IN PVOID SystemArgument2)
// {
//     DPRINT("USBPORT_HcWakeDpc: ... \n");
//     USBPORT_CompletePdoWaitWake((PDEVICE_OBJECT)DeferredContext);
// }
//
// VOID
// NTAPI
// USBPORT_HcQueueWakeDpc(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_HcQueueWakeDpc: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     KeInsertQueueDpc(&FdoExtension->HcWakeDpc, NULL, NULL);
// }
//
// VOID
// NTAPI
// USBPORT_CompletePendingIdleIrp(IN PDEVICE_OBJECT PdoDevice)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PIRP Irp;
//
//     DPRINT("USBPORT_CompletePendingIdleIrp: ... \n");
//
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Irp = IoCsqRemoveNextIrp(&FdoExtension->IdleIoCsq, 0);
//
//     if (Irp)
//     {
//         InterlockedDecrement(&FdoExtension->IdleLockCounter);
//
//         DPRINT("USBPORT_CompletePendingIdleIrp: Complete Irp - %p\n", Irp);
//
//         Irp->IoStatus.Status = STATUS_CANCELLED;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_DoSetPowerD0(IN PDEVICE_OBJECT FdoDevice)
// {
//     DPRINT("USBPORT_DoSetPowerD0: FIXME!\n");
//     return;
// }
//
// VOID
// NTAPI
// USBPORT_SuspendController(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//
//     DPRINT1("USBPORT_SuspendController \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     FdoExtension->TimerFlags |= USBPORT_TMFLAG_RH_SUSPENDED;
//
//     USBPORT_FlushController(FdoDevice);
//
//     if (FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND)
//     {
//         return;
//     }
//
//     FdoExtension->TimerFlags |= USBPORT_TMFLAG_HC_SUSPENDED;
//
//     if (FdoExtension->MiniPortFlags & USBPORT_MPFLAG_INTERRUPTS_ENABLED)
//     {
//         FdoExtension->MiniPortFlags |= USBPORT_MPFLAG_SUSPENDED;
//
//         USBPORT_Wait(FdoDevice, 10);
//         Packet->SuspendController(FdoExtension->MiniPortExt);
//     }
//
//     FdoExtension->Flags |= USBPORT_FLAG_HC_SUSPEND;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_ResumeController(IN PDEVICE_OBJECT FdoDevice)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     KIRQL OldIrql;
//     MPSTATUS MpStatus;
//
//     DPRINT1("USBPORT_ResumeController: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     if (!(FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND))
//     {
//         return Status;
//     }
//
//     KeAcquireSpinLock(&FdoExtension->TimerFlagsSpinLock, &OldIrql);
//
//     FdoExtension->TimerFlags &= ~(USBPORT_TMFLAG_HC_SUSPENDED |
//                                   USBPORT_TMFLAG_RH_SUSPENDED);
//
//     KeReleaseSpinLock(&FdoExtension->TimerFlagsSpinLock, OldIrql);
//
//     if (!(FdoExtension->MiniPortFlags & USBPORT_MPFLAG_SUSPENDED))
//     {
//         FdoExtension->Flags &= ~USBPORT_FLAG_HC_SUSPEND;
//         return Status;
//     }
//
//     FdoExtension->MiniPortFlags &= ~USBPORT_MPFLAG_SUSPENDED;
//
//     if (!Packet->ResumeController(FdoExtension->MiniPortExt))
//     {
//         Status = USBPORT_Wait(FdoDevice, 100);
//
//         FdoExtension->Flags &= ~USBPORT_FLAG_HC_SUSPEND;
//         return Status;
//     }
//
//     KeAcquireSpinLock(&FdoExtension->TimerFlagsSpinLock, &OldIrql);
//     FdoExtension->TimerFlags |= (USBPORT_TMFLAG_HC_SUSPENDED |
//                                  USBPORT_TMFLAG_HC_RESUME);
//     KeReleaseSpinLock(&FdoExtension->TimerFlagsSpinLock, OldIrql);
//
//     USBPORT_MiniportInterrupts(FdoDevice, FALSE);
//
//     Packet->StopController(FdoExtension->MiniPortExt, 1);
//
//     USBPORT_NukeAllEndpoints(FdoDevice);
//
//     RtlZeroMemory(FdoExtension->MiniPortExt, Packet->MiniPortExtensionSize);
//
//     RtlZeroMemory((PVOID)FdoExtension->UsbPortResources.StartVA,
//                   Packet->MiniPortResourcesSize);
//
//     FdoExtension->UsbPortResources.IsChirpHandled = TRUE;
//
//     MpStatus = Packet->StartController(FdoExtension->MiniPortExt,
//                                        &FdoExtension->UsbPortResources);
//
//     FdoExtension->UsbPortResources.IsChirpHandled = FALSE;
//
//     if (!MpStatus)
//     {
//         USBPORT_MiniportInterrupts(FdoDevice, TRUE);
//     }
//
//     KeAcquireSpinLock(&FdoExtension->TimerFlagsSpinLock, &OldIrql);
//
//     FdoExtension->TimerFlags &= ~(USBPORT_TMFLAG_HC_SUSPENDED |
//                                   USBPORT_TMFLAG_HC_RESUME |
//                                   USBPORT_TMFLAG_RH_SUSPENDED);
//
//     KeReleaseSpinLock(&FdoExtension->TimerFlagsSpinLock, OldIrql);
//
//     Status = USBPORT_Wait(FdoDevice, 100);
//
//     FdoExtension->Flags &= ~USBPORT_FLAG_HC_SUSPEND;
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_PdoDevicePowerState(IN PDEVICE_OBJECT PdoDevice,
//                             IN PIRP Irp)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status = STATUS_SUCCESS;
//     POWER_STATE State;
//
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     State = IoStack->Parameters.Power.State;
//
//     DPRINT1("USBPORT_PdoDevicePowerState: Irp - %p, State - %x\n",
//             Irp,
//             State.DeviceState);
//
//     if (State.DeviceState == PowerDeviceD0)
//     {
//         if (FdoExtension->CommonExtension.DevicePowerState == PowerDeviceD0)
//         {
//             // FIXME FdoExtension->Flags
//             while (FdoExtension->SetPowerLockCounter)
//             {
//                 USBPORT_Wait(FdoDevice, 10);
//             }
//
//             USBPORT_ResumeController(FdoDevice);
//
//             PdoExtension->CommonExtension.DevicePowerState = PowerDeviceD0;
//
//             USBPORT_CompletePdoWaitWake(FdoDevice);
//             USBPORT_CompletePendingIdleIrp(PdoDevice);
//         }
//         else
//         {
//             DPRINT1("USBPORT_PdoDevicePowerState: FdoExtension->Flags - %lx\n",
//                     FdoExtension->Flags);
//
//             DbgBreakPoint();
//             Status = STATUS_UNSUCCESSFUL;
//         }
//     }
//     else if (State.DeviceState  == PowerDeviceD1 ||
//              State.DeviceState  == PowerDeviceD2 ||
//              State.DeviceState  == PowerDeviceD3)
//     {
//         FdoExtension->TimerFlags |= USBPORT_TMFLAG_WAKE;
//         USBPORT_SuspendController(FdoDevice);
//         PdoExtension->CommonExtension.DevicePowerState = State.DeviceState;
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBPORT_CancelPendingWakeIrp(IN PDEVICE_OBJECT PdoDevice,
//                              IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     KIRQL OldIrql;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//
//     DPRINT("USBPORT_CancelPendingWakeIrp: ... \n");
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoExtension = PdoExtension->FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->PowerWakeSpinLock, &OldIrql);
//
//     if (PdoExtension->WakeIrp == Irp)
//     {
//         PdoExtension->WakeIrp = NULL;
//     }
//
//     KeReleaseSpinLock(&FdoExtension->PowerWakeSpinLock, OldIrql);
//
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_PdoPower(IN PDEVICE_OBJECT PdoDevice,
//                  IN PIRP Irp)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PIO_STACK_LOCATION IoStack;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_PdoPower: Irp - %p\n", Irp);
//
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     Status = Irp->IoStatus.Status;
//
//     switch (IoStack->MinorFunction)
//     {
//       case IRP_MN_WAIT_WAKE:
//           DPRINT("USBPORT_PdoPower: IRP_MN_WAIT_WAKE\n");
//
//           if (!(FdoExtension->Flags & USBPORT_FLAG_HC_STARTED))
//           {
//               /* The device does not support wake-up */
//               Status = STATUS_NOT_SUPPORTED;
//               break;
//           }
//
//           KeAcquireSpinLock(&FdoExtension->PowerWakeSpinLock, &OldIrql);
//
//           IoSetCancelRoutine(Irp, USBPORT_CancelPendingWakeIrp);
//
//           /* Check if the IRP has been cancelled */
//           if (Irp->Cancel)
//           {
//               if (IoSetCancelRoutine(Irp, NULL))
//               {
//                   /* IRP has been cancelled, release cancel spinlock */
//                   KeReleaseSpinLock(&FdoExtension->PowerWakeSpinLock, OldIrql);
//
//                   DPRINT("USBPORT_PdoPower: IRP_MN_WAIT_WAKE - STATUS_CANCELLED\n");
//
//                   /* IRP is cancelled */
//                   Status = STATUS_CANCELLED;
//                   break;
//               }
//           }
//
//           if (!PdoExtension->WakeIrp)
//           {
//               /* The driver received the IRP
//                  and is waiting for the device to signal wake-up. */
//
//               DPRINT("USBPORT_PdoPower: IRP_MN_WAIT_WAKE - No WakeIrp\n");
//
//               IoMarkIrpPending(Irp);
//               PdoExtension->WakeIrp = Irp;
//
//               KeReleaseSpinLock(&FdoExtension->PowerWakeSpinLock, OldIrql);
//               return STATUS_PENDING;
//           }
//           else
//           {
//               /* An IRP_MN_WAIT_WAKE request is already pending and must be
//                  completed or canceled before another IRP_MN_WAIT_WAKE request
//                  can be issued. */
//
//               if (IoSetCancelRoutine(Irp, NULL))
//               {
//                   DPRINT("USBPORT_PdoPower: IRP_MN_WAIT_WAKE - STATUS_DEVICE_BUSY\n");
//
//                   KeReleaseSpinLock(&FdoExtension->PowerWakeSpinLock, OldIrql);
//                   PoStartNextPowerIrp(Irp);
//                   Status = STATUS_DEVICE_BUSY;
//                   break;
//               }
//               else
//               {
//                   ASSERT(FALSE);
//                   KeReleaseSpinLock(&FdoExtension->PowerWakeSpinLock, OldIrql);
//                   return Status;
//               }
//           }
//
//       case IRP_MN_POWER_SEQUENCE:
//           DPRINT("USBPORT_PdoPower: IRP_MN_POWER_SEQUENCE\n");
//           PoStartNextPowerIrp(Irp);
//           break;
//
//       case IRP_MN_SET_POWER:
//           DPRINT("USBPORT_PdoPower: IRP_MN_SET_POWER\n");
//
//           if (IoStack->Parameters.Power.Type == DevicePowerState)
//           {
//               DPRINT("USBPORT_PdoPower: IRP_MN_SET_POWER/DevicePowerState\n");
//               Status = USBPORT_PdoDevicePowerState(PdoDevice, Irp);
//               PoStartNextPowerIrp(Irp);
//               break;
//           }
//
//           DPRINT("USBPORT_PdoPower: IRP_MN_SET_POWER/SystemPowerState \n");
//
//           if (IoStack->Parameters.Power.State.SystemState == PowerSystemWorking)
//           {
//               FdoExtension->TimerFlags |= USBPORT_TMFLAG_WAKE;
//           }
//           else
//           {
//               FdoExtension->TimerFlags &= ~USBPORT_TMFLAG_WAKE;
//           }
//
//           Status = STATUS_SUCCESS;
//
//           PoStartNextPowerIrp(Irp);
//           break;
//
//       case IRP_MN_QUERY_POWER:
//           DPRINT("USBPORT_PdoPower: IRP_MN_QUERY_POWER\n");
//           Status = STATUS_SUCCESS;
//           PoStartNextPowerIrp(Irp);
//           break;
//
//       default:
//           DPRINT1("USBPORT_PdoPower: unknown IRP_MN_POWER!\n");
//           PoStartNextPowerIrp(Irp);
//           break;
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HcWake(IN PDEVICE_OBJECT FdoDevice,
//                IN PIRP Irp)
// {
//     DPRINT1("USBPORT_HcWake: UNIMPLEMENTED. FIXME. \n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_DevicePowerState(IN PDEVICE_OBJECT FdoDevice,
//                          IN PIRP Irp)
// {
//     DPRINT1("USBPORT_DevicePowerState: UNIMPLEMENTED. FIXME. \n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_SystemPowerState(IN PDEVICE_OBJECT FdoDevice,
//                          IN PIRP Irp)
// {
//     DPRINT1("USBPORT_SystemPowerState: UNIMPLEMENTED. FIXME. \n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_FdoPower(IN PDEVICE_OBJECT FdoDevice,
//                  IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_FdoPower: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     switch (IoStack->MinorFunction)
//     {
//       case IRP_MN_WAIT_WAKE:
//           DPRINT("USBPORT_FdoPower: IRP_MN_WAIT_WAKE\n");
//           Status = USBPORT_HcWake(FdoDevice, Irp);
//           return Status;
//
//       case IRP_MN_POWER_SEQUENCE:
//           DPRINT("USBPORT_FdoPower: IRP_MN_POWER_SEQUENCE\n");
//           break;
//
//       case IRP_MN_SET_POWER:
//           DPRINT("USBPORT_FdoPower: IRP_MN_SET_POWER\n");
//           if (IoStack->Parameters.Power.Type == DevicePowerState)
//           {
//               Status = USBPORT_DevicePowerState(FdoDevice, Irp);
//           }
//           else
//           {
//               Status = USBPORT_SystemPowerState(FdoDevice, Irp);
//           }
//
//           if (Status != STATUS_PENDING)
//               break;
//
//           return Status;
//
//       case IRP_MN_QUERY_POWER:
//           DPRINT("USBPORT_FdoPower: IRP_MN_QUERY_POWER\n");
//           Irp->IoStatus.Status = STATUS_SUCCESS;
//           break;
//
//       default:
//           DPRINT1("USBPORT_FdoPower: unknown IRP_MN_POWER!\n");
//           break;
//     }
//
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//     PoStartNextPowerIrp(Irp);
//     return PoCallDriver(FdoExtension->CommonExtension.LowerDevice, Irp);
// }
//
// VOID
// NTAPI
// USBPORT_DoIdleNotificationCallback(IN PVOID Context)
// {
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PIRP NextIrp;
//     LARGE_INTEGER CurrentTime = {{0, 0}};
//     PTIMER_WORK_QUEUE_ITEM IdleQueueItem;
//     PDEVICE_OBJECT PdoDevice;
//     PUSB_IDLE_CALLBACK_INFO IdleCallbackInfo;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_DoIdleNotificationCallback \n");
//
//     IdleQueueItem = Context;
//
//     FdoDevice = IdleQueueItem->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     PdoDevice = FdoExtension->RootHubPdo;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     KeQuerySystemTime(&CurrentTime);
//
//     if ((FdoExtension->IdleTime.QuadPart == 0) ||
//         (((CurrentTime.QuadPart - FdoExtension->IdleTime.QuadPart) / 10000) >= 500))
//     {
//         if (PdoExtension->CommonExtension.DevicePowerState == PowerDeviceD0 &&
//             FdoExtension->CommonExtension.DevicePowerState == PowerDeviceD0)
//         {
//             NextIrp = IoCsqRemoveNextIrp(&FdoExtension->IdleIoCsq, NULL);
//
//             if (NextIrp)
//             {
//                 IoStack = IoGetCurrentIrpStackLocation(NextIrp);
//                 IdleCallbackInfo = IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//                 if (IdleCallbackInfo && IdleCallbackInfo->IdleCallback)
//                 {
//                     IdleCallbackInfo->IdleCallback(IdleCallbackInfo->IdleContext);
//                 }
//
//                 if (NextIrp->Cancel)
//                 {
//                     InterlockedDecrement(&FdoExtension->IdleLockCounter);
//
//                     NextIrp->IoStatus.Status = STATUS_CANCELLED;
//                     NextIrp->IoStatus.Information = 0;
//                     IoCompleteRequest(NextIrp, IO_NO_INCREMENT);
//                 }
//                 else
//                 {
//                     IoCsqInsertIrp(&FdoExtension->IdleIoCsq, NextIrp, NULL);
//                 }
//             }
//         }
//     }
//
//     KeAcquireSpinLock(&FdoExtension->TimerFlagsSpinLock, &OldIrql);
//     FdoExtension->TimerFlags &= ~USBPORT_TMFLAG_IDLE_QUEUEITEM_ON;
//     KeReleaseSpinLock(&FdoExtension->TimerFlagsSpinLock, OldIrql);
//
//     ExFreePoolWithTag(IdleQueueItem, USB_PORT_TAG);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_IdleNotification(IN PDEVICE_OBJECT PdoDevice,
//                          IN PIRP Irp)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     LONG LockCounter;
//     NTSTATUS Status = STATUS_PENDING;
//
//     DPRINT("USBPORT_IdleNotification: Irp - %p\n", Irp);
//
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     LockCounter = InterlockedIncrement(&FdoExtension->IdleLockCounter);
//
//     if (LockCounter != 0)
//     {
//         if (Status != STATUS_PENDING)
//         {
//             InterlockedDecrement(&FdoExtension->IdleLockCounter);
//
//             Irp->IoStatus.Status = Status;
//             Irp->IoStatus.Information = 0;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//             return Status;
//         }
//
//         Status = STATUS_DEVICE_BUSY;
//     }
//
//     if (Status != STATUS_PENDING)
//     {
//         InterlockedDecrement(&FdoExtension->IdleLockCounter);
//
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//         return Status;
//     }
//
//     Irp->IoStatus.Status = STATUS_PENDING;
//     IoMarkIrpPending(Irp);
//
//     KeQuerySystemTime(&FdoExtension->IdleTime);
//
//     IoCsqInsertIrp(&FdoExtension->IdleIoCsq, Irp, 0);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBPORT_AdjustDeviceCapabilities(IN PDEVICE_OBJECT FdoDevice,
//                                  IN PDEVICE_OBJECT PdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_CAPABILITIES Capabilities;
//
//     DPRINT("USBPORT_AdjustDeviceCapabilities: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     PdoExtension = PdoDevice->DeviceExtension;
//     Capabilities = &PdoExtension->Capabilities;
//
//     RtlCopyMemory(Capabilities,
//                   &FdoExtension->Capabilities,
//                   sizeof(DEVICE_CAPABILITIES));
//
//     Capabilities->DeviceD1 = FALSE;
//     Capabilities->DeviceD2 = TRUE;
//
//     Capabilities->Removable = FALSE;
//     Capabilities->UniqueID = FALSE;
//
//     Capabilities->WakeFromD0 = TRUE;
//     Capabilities->WakeFromD1 = FALSE;
//     Capabilities->WakeFromD2 = TRUE;
//     Capabilities->WakeFromD3 = FALSE;
//
//     Capabilities->Address = 0;
//     Capabilities->UINumber = 0;
//
//     if (Capabilities->SystemWake == PowerSystemUnspecified)
//         Capabilities->SystemWake = PowerSystemWorking;
//
//     Capabilities->DeviceWake = PowerDeviceD2;
//
//     Capabilities->DeviceState[PowerSystemSleeping1] = PowerDeviceD3;
//     Capabilities->DeviceState[PowerSystemSleeping2] = PowerDeviceD3;
//     Capabilities->DeviceState[PowerSystemSleeping3] = PowerDeviceD3;
//     Capabilities->DeviceState[PowerSystemHibernate] = PowerDeviceD3;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
