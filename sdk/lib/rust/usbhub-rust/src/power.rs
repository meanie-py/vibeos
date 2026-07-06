use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAcquireCancelSpinLock, IoCancelIrp, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoReleaseCancelSpinLock, IoSetCompletionRoutine, KEVENT, KeInitializeEvent, KeReleaseSemaphore, KeWaitForSingleObject, LIST_ENTRY, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, POWER_STATE, PRKEVENT, _KWAIT_REASON_Suspended};
use ntkrnl_sys::ddk::_POWER_STATE_TYPE_DevicePowerState as DevicePowerState;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD1 as PowerDeviceD1;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceUnspecified as PowerDeviceUnspecified;
use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemHibernate as PowerSystemHibernate;
use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking;
use ntkrnl_sys::ddk::PREQUEST_POWER_COMPLETE;
use ntkrnl_sys::ddk::DEVICE_POWER_STATE;
use core::ffi::c_void;

extern "C" {
    fn USBH_GetRootHubExtension(HubExtension: PUSBHUB_FDO_EXTENSION) -> PUSBHUB_FDO_EXTENSION;
    fn USBH_Wait(MilliSeconds: ULONG);
    fn USBH_SubmitStatusChangeTransfer(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS;
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

extern "system" {
    fn PoRequestPowerIrp(
        DeviceObject: PDEVICE_OBJECT,
        MinorFunction: UCHAR,
        PowerState: POWER_STATE,
        CompletionFunction: PREQUEST_POWER_COMPLETE,
        Context: PVOID,
        Irp: *mut PIRP,
    ) -> NTSTATUS;
}

// USBH_CompletePowerIrp — translated but dropped during assembly. Reconstructed faithfully from
// drivers/usb/usbhub/power.c:18. NTAPI (usbhub.h:327) → extern "system" (matches the bindgen
// `extern "stdcall"` decl in types.rs + the @12 link decoration). Completes a power IRP and
// signals the hub's pending-request event when the last in-flight request drains.
#[no_mangle]
pub unsafe extern "system" fn USBH_CompletePowerIrp(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
    NtStatus: NTSTATUS,
) {
    (*Irp).IoStatus.__bindgen_anon_1.Status = NtStatus;
    PoStartNextPowerIrp(Irp);
    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID,
            EVENT_INCREMENT as LONG,
            0 as BOOLEAN,
        );
    }
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
}

// USBH_HubCancelWakeIrp — translated but dropped during assembly. Reconstructed faithfully from
// drivers/usb/usbhub/power.c:43. NTAPI (usbhub.h:364) → extern "system". Cancels the wait-wake
// IRP and, if this is the side that wins the FdoWaitWakeLock race, completes it as CANCELLED.
#[no_mangle]
pub unsafe extern "system" fn USBH_HubCancelWakeIrp(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) {
    const STATUS_CANCELLED: NTSTATUS = 0xC0000120u32 as i32;
    IoCancelIrp(Irp);
    if InterlockedExchange(core::ptr::addr_of_mut!((*HubExtension).FdoWaitWakeLock) as *mut LONG, 1) != 0 {
        PoStartNextPowerIrp(Irp);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    }
}

// ---- file-scope preamble (translated) ----
// (no output — both `#define`s are empty object-like macros with no constant value)

#[no_mangle]
pub unsafe extern "system" fn USBH_HubESDRecoverySetD3Completion(
    _DeviceObject: PDEVICE_OBJECT,
    _MinorFunction: UCHAR,
    _PowerState: POWER_STATE,
    Context: PVOID,
    _IoStatus: PIO_STATUS_BLOCK,
) {
    KeSetEvent(Context, EVENT_INCREMENT as LONG, FALSE as BOOLEAN);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubESDRecoverySetD0Completion(
    _DeviceObject: PDEVICE_OBJECT,
    _MinorFunction: UCHAR,
    _PowerState: POWER_STATE,
    Context: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
) {
    const USBHUB_FDO_FLAG_ESD_RECOVERING: u32 = 0x00000200; // FIXED 2026-07-06: translation invented 0x80000; real value is a (1<<N) define in usbhub.h

    let HubExtension: PUSBHUB_FDO_EXTENSION = Context as PUSBHUB_FDO_EXTENSION;

    if !HubExtension.is_null() && NT_SUCCESS((*IoStatus).__bindgen_anon_1.Status) {
        (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_ESD_RECOVERING as u32);
    } else if !HubExtension.is_null() {
        (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_ESD_RECOVERING as u32);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoDeferPoRequestCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    _MinorFunction: UCHAR,
    _PowerState: POWER_STATE,
    Context: PVOID,
    _IoStatus: PIO_STATUS_BLOCK,
) {
    let Extension: PUSBHUB_FDO_EXTENSION;
    let mut HubExtension: PUSBHUB_FDO_EXTENSION = core::ptr::null_mut();
    let PowerIrp: PIRP;
    let IoStack: PIO_STACK_LOCATION;

    Extension = Context as PUSBHUB_FDO_EXTENSION;

    PowerIrp = (*Extension).PowerIrp;

    if (*Extension).Common.ExtensionType == USBH_EXTENSION_TYPE_HUB {
        HubExtension = Context as PUSBHUB_FDO_EXTENSION;
    }

    IoStack = IoGetCurrentIrpStackLocation(PowerIrp);

    if (*IoStack).Parameters.Power.State.SystemState as u32 == PowerSystemWorking as u32
        && !HubExtension.is_null()
        && (*HubExtension).LowerPDO == (*HubExtension).RootHubPdo
    {
        (*HubExtension).SystemPowerState.SystemState = PowerSystemWorking;
        USBH_CheckIdleDeferred(HubExtension);
    }

    IoCopyCurrentIrpStackLocationToNext(PowerIrp);
    PoStartNextPowerIrp(PowerIrp);
    PoCallDriver((*Extension).LowerDevice, PowerIrp);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubSetD0(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS {
    const STATUS_INVALID_DEVICE_STATE: NTSTATUS = 0xC0000184u32 as NTSTATUS;
    const USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST: ULONG = 0x00001000; // FIXED 2026-07-06: translation invented 0x4; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_WAKEUP_START: ULONG = 0x00020000; // FIXED 2026-07-06: translation invented 0x10; real value is a (1<<N) define in usbhub.h

    let mut RootHubDevExt: PUSBHUB_FDO_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut PowerState: POWER_STATE = core::mem::zeroed();

    RootHubDevExt = USBH_GetRootHubExtension(HubExtension);

    if (*RootHubDevExt).SystemPowerState.SystemState != PowerSystemWorking {
        Status = STATUS_INVALID_DEVICE_STATE;
        return Status;
    }

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST != 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );
    }

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);

    PowerState.DeviceState = PowerDeviceD0;

    Status = PoRequestPowerIrp(
        (*HubExtension).LowerPDO,
        IRP_MN_SET_POWER as UCHAR,
        PowerState,
        core::mem::transmute::<_, PREQUEST_POWER_COMPLETE>(Some(USBH_HubESDRecoverySetD3Completion as unsafe extern "system" fn(PDEVICE_OBJECT, UCHAR, POWER_STATE, PVOID, PIO_STATUS_BLOCK))),
        core::ptr::addr_of_mut!(Event) as PVOID,
        core::ptr::null_mut(),
    );

    if Status == STATUS_PENDING {
        Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );
    }

    while (*HubExtension).HubFlags & USBHUB_FDO_FLAG_WAKEUP_START != 0 {
        USBH_Wait(10 as ULONG);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubStartESDRecovery(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    const STATUS_INVALID_DEVICE_STATE: NTSTATUS = 0xC0000184u32 as NTSTATUS;

    let mut RootHubDevExt: PUSBHUB_FDO_EXTENSION;
    let mut Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut PowerState: POWER_STATE = core::mem::zeroed();

    RootHubDevExt = USBH_GetRootHubExtension(HubExtension);

    if (*RootHubDevExt).SystemPowerState.SystemState as u32 != PowerSystemWorking as u32 {
        Status = STATUS_INVALID_DEVICE_STATE;
        (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_ESD_RECOVERING as u32);
        return Status;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        0 as BOOLEAN,
    );

    PowerState.DeviceState = PowerDeviceD3;

    Status = PoRequestPowerIrp(
        (*HubExtension).LowerPDO,
        IRP_MN_SET_POWER as UCHAR,
        PowerState,
        core::mem::transmute::<_, PREQUEST_POWER_COMPLETE>(Some(USBH_HubESDRecoverySetD3Completion as unsafe extern "system" fn(PDEVICE_OBJECT, UCHAR, POWER_STATE, PVOID, PIO_STATUS_BLOCK))),
        core::ptr::addr_of_mut!(Event) as PVOID,
        core::ptr::null_mut(),
    );
    if Status == STATUS_PENDING {
        Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );
    }
    if !NT_SUCCESS(Status) {
        (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_ESD_RECOVERING as u32);
        return Status;
    }

    PowerState.DeviceState = PowerDeviceD0;

    Status = PoRequestPowerIrp(
        (*HubExtension).LowerPDO,
        IRP_MN_SET_POWER as UCHAR,
        PowerState,
        core::mem::transmute::<_, PREQUEST_POWER_COMPLETE>(Some(USBH_HubESDRecoverySetD0Completion as unsafe extern "system" fn(PDEVICE_OBJECT, UCHAR, POWER_STATE, PVOID, PIO_STATUS_BLOCK))),
        HubExtension as PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) && Status != STATUS_PENDING {
        (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_ESD_RECOVERING as u32);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IdleCancelPowerHubWorker(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Context: PVOID,
) {
    let mut WorkItemIdlePower: PUSBHUB_IDLE_PORT_CANCEL_CONTEXT;
    let mut Irp: PIRP;

    WorkItemIdlePower = Context as PUSBHUB_IDLE_PORT_CANCEL_CONTEXT;

    if !HubExtension.is_null()
        && (*HubExtension).CurrentPowerState.DeviceState as u32 != PowerDeviceD0 as u32
        && (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED != 0
    {
        USBH_HubSetD0(HubExtension);
    }

    Irp = (*WorkItemIdlePower).Irp;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;

    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubQueuePortWakeIrps(
    mut HubExtension: PUSBHUB_FDO_EXTENSION,
    mut ListIrps: PLIST_ENTRY,
) {
    let mut PortDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut NumPorts: USHORT;
    let mut Port: USHORT;
    let mut WakeIrp: PIRP;
    let mut OldIrql: KIRQL = 0;

    NumPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts as USHORT;

    InitializeListHead(ListIrps);

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(OldIrql));

    Port = 0 as USHORT;
    while (Port as i32) < NumPorts as i32 {
        PortDevice = (*(*HubExtension).PortData.offset(Port as isize)).DeviceObject;

        if !PortDevice.is_null() {
            PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

            WakeIrp = (*PortExtension).PdoWaitWakeIrp;
            (*PortExtension).PdoWaitWakeIrp = core::ptr::null_mut();

            if !WakeIrp.is_null() {
                DbgBreakPoint();
            }
        }

        Port = Port.wrapping_add(1);
    }

    IoReleaseCancelSpinLock(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubCompleteQueuedPortWakeIrps(
    mut HubExtension: PUSBHUB_FDO_EXTENSION,
    mut ListIrps: PLIST_ENTRY,
    mut NtStatus: NTSTATUS,
) {
    while !IsListEmpty(ListIrps as *const LIST_ENTRY) {
        if DbgPrint(
            b"(%s:%d) USBH_HubCompleteQueuedPortWakeIrps: UNIMPLEMENTED. FIXME\n\0" as *const u8
                as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/power.c\0"
                as *const u8 as *const core::ffi::c_char,
            301 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/power.c\0"
                    as *const u8 as *const core::ffi::c_char,
                301 as core::ffi::c_int,
            );
        }
        DbgBreakPoint();
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubCompletePortWakeIrps(
    mut HubExtension: PUSBHUB_FDO_EXTENSION,
    mut NtStatus: NTSTATUS,
) {
    let mut ListIrps: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED != 0 {
        USBH_HubQueuePortWakeIrps(HubExtension, core::ptr::addr_of_mut!(ListIrps));

        USBH_HubCompleteQueuedPortWakeIrps(
            HubExtension,
            core::ptr::addr_of_mut!(ListIrps),
            NtStatus,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoPoRequestD0Completion(
    mut DeviceObject: PDEVICE_OBJECT,
    mut MinorFunction: UCHAR,
    mut PowerState: POWER_STATE,
    mut Context: PVOID,
    mut IoStatus: PIO_STATUS_BLOCK,
) {
    let mut HubExtension: PUSBHUB_FDO_EXTENSION;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    USBH_HubCompletePortWakeIrps(HubExtension, STATUS_SUCCESS);

    (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_WAKEUP_START as u32);

    if InterlockedDecrement(
        core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut core::ffi::c_long,
    ) == 0
    {
        KeSetEvent(
            core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID,
            EVENT_INCREMENT as KPRIORITY,
            FALSE as BOOLEAN,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_CompletePortWakeIrpsWorker(
    mut HubExtension: PUSBHUB_FDO_EXTENSION,
    mut Context: PVOID,
) {
    if DbgPrint(
        b"(%s:%d) USBH_CompletePortWakeIrpsWorker: UNIMPLEMENTED. FIXME\n\0" as *const u8 as *const i8,
        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/power.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        356 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/power.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            356 as ::core::ffi::c_int,
        );
    }
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoWWIrpIoCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {

    const USBHUB_FDO_FLAG_PENDING_WAKE_IRP: u32 = 0x00000080; // FIXED 2026-07-06: translation invented 0x400; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_WAKEUP_START: u32 = 0x00020000; // FIXED 2026-07-06: translation invented 0x800; real value is a (1<<N) define in usbhub.h
    const EVENT_INCREMENT: i32 = 1;
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;

    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;
    let mut PowerState: POWER_STATE = core::mem::zeroed();
    let WakeIrp: PIRP;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(OldIrql));

    (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_PENDING_WAKE_IRP as u32);

    WakeIrp = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*HubExtension).PendingWakeIrp) as *mut PVOID,
        core::ptr::null_mut(),
    ) as PIRP;

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID,
            EVENT_INCREMENT,
            0,
        );
    }

    IoReleaseCancelSpinLock(OldIrql);

    if !NT_SUCCESS(Status) {
        DbgBreakPoint();
    } else {
        PowerState.DeviceState = PowerDeviceD0;

        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_WAKEUP_START as u32;
        InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

        Status = STATUS_SUCCESS;

        PoRequestPowerIrp(
            (*HubExtension).LowerPDO,
            IRP_MN_SET_POWER as UCHAR,
            PowerState,
            core::mem::transmute::<_, PREQUEST_POWER_COMPLETE>(Some(USBH_FdoPoRequestD0Completion as unsafe extern "system" fn(PDEVICE_OBJECT, UCHAR, POWER_STATE, PVOID, PIO_STATUS_BLOCK))),
            HubExtension as PVOID,
            core::ptr::null_mut(),
        );
    }

    if WakeIrp.is_null() {
        let mut target: i32 = (*HubExtension).FdoWaitWakeLock;
        let old: i32 = target;
        target = 1;
        (*HubExtension).FdoWaitWakeLock = target;
        let _ = target;
        if old == 0 {
            Status = STATUS_MORE_PROCESSING_REQUIRED;
        }
    }

    if Status != STATUS_MORE_PROCESSING_REQUIRED {
        PoStartNextPowerIrp(Irp);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PowerIrpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    const USBHUB_FDO_FLAG_SET_D0_STATE: u32 = 0x00000400; // FIXED 2026-07-06: translation invented 0x800; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_HIBERNATE_STATE: u32 = 0x00400000; // FIXED 2026-07-06: translation invented 0x10000; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_DEVICE_STOPPED: u32 = 0x00000020; // FIXED 2026-07-06: translation invented 0x10; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_DO_ENUMERATION: u32 = 0x00100000; // FIXED 2026-07-06: translation invented 0x80; real value is a (1<<N) define in usbhub.h
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as i32;

    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut OldDeviceState: DEVICE_POWER_STATE;
    let mut Status: NTSTATUS;
    let mut PowerState: POWER_STATE;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    PowerState = (*IoStack).Parameters.Power.State;

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if !NT_SUCCESS(Status) {
        if PowerState.DeviceState == PowerDeviceD0 {
            PoStartNextPowerIrp(Irp);
            (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_SET_D0_STATE as u32);
        }
    } else if PowerState.DeviceState == PowerDeviceD0 {
        (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_SET_D0_STATE as u32);

        OldDeviceState = (*HubExtension).CurrentPowerState.DeviceState;
        (*HubExtension).CurrentPowerState.DeviceState = PowerDeviceD0;

        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_HIBERNATE_STATE != 0 {
            DbgBreakPoint();
        }

        (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_HIBERNATE_STATE as u32);

        if OldDeviceState == PowerDeviceD3 {
            DbgBreakPoint();
        }

        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0
            && (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DO_ENUMERATION != 0
        {
            USBH_SubmitStatusChangeTransfer(HubExtension);
        }

        if Status != STATUS_MORE_PROCESSING_REQUIRED {
            PoStartNextPowerIrp(Irp);
            return Status;
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoPower(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
    Minor: UCHAR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut PowerState: POWER_STATE;
    let mut DevicePwrState: POWER_STATE = core::mem::zeroed();
    let mut IsAllPortsD3: BOOLEAN;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut Port: ULONG;

    match Minor {
        IRP_MN_WAIT_WAKE => {
            IoCopyCurrentIrpStackLocationToNext(Irp);

            IoSetCompletionRoutine(
                Irp,
                core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(Some(USBH_FdoWWIrpIoCompletion as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS)),
                HubExtension as PVOID,
                TRUE as BOOLEAN,
                TRUE as BOOLEAN,
                TRUE as BOOLEAN,
            );

            PoStartNextPowerIrp(Irp);
            IoMarkIrpPending(Irp);
            PoCallDriver((*HubExtension).LowerDevice, Irp);

            return STATUS_PENDING;
        }
        IRP_MN_POWER_SEQUENCE => {}
        IRP_MN_SET_POWER => {
            IoStack = IoGetCurrentIrpStackLocation(Irp);
            PowerState = (*IoStack).Parameters.Power.State;

            if (*IoStack).Parameters.Power.Type as u32 == DevicePowerState as i32 as u32 {
                if (*HubExtension).CurrentPowerState.DeviceState as u32
                    == PowerState.DeviceState as u32
                {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    PoStartNextPowerIrp(Irp);
                    IoMarkIrpPending(Irp);
                    PoCallDriver((*HubExtension).LowerDevice, Irp);

                    return STATUS_PENDING;
                }

                match PowerState.DeviceState as u32 {
                    1 => {
                        if (*HubExtension).HubFlags as core::ffi::c_ulong
                            & USBHUB_FDO_FLAG_SET_D0_STATE as core::ffi::c_ulong
                            == 0
                        {
                            (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_NOT_D0_STATE
                                | USBHUB_FDO_FLAG_DEVICE_STOPPING)
                                as core::ffi::c_ulong;

                            (*HubExtension).HubFlags |=
                                USBHUB_FDO_FLAG_SET_D0_STATE as core::ffi::c_ulong;

                            IoCopyCurrentIrpStackLocationToNext(Irp);

                            IoSetCompletionRoutine(
                                Irp,
                                core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(Some(USBH_PowerIrpCompletion as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS)),
                                HubExtension as PVOID,
                                TRUE as BOOLEAN,
                                TRUE as BOOLEAN,
                                TRUE as BOOLEAN,
                            );
                        } else {
                            IoCopyCurrentIrpStackLocationToNext(Irp);
                            PoStartNextPowerIrp(Irp);
                        }

                        IoMarkIrpPending(Irp);
                        PoCallDriver((*HubExtension).LowerDevice, Irp);
                        return STATUS_PENDING;
                    }
                    2 | 3 | 4 => {
                        if (*HubExtension).ResetRequestCount != 0 {
                            IoCancelIrp((*HubExtension).ResetPortIrp);

                            KeWaitForSingleObject(
                                core::ptr::addr_of_mut!((*HubExtension).ResetEvent) as PVOID,
                                _KWAIT_REASON_Executive,
                                0i32 as i32 as KPROCESSOR_MODE,
                                FALSE as BOOLEAN,
                                core::ptr::null_mut(),
                            );
                        }

                        if (*HubExtension).HubFlags as core::ffi::c_ulong
                            & USBHUB_FDO_FLAG_DEVICE_STOPPED as core::ffi::c_ulong
                            == 0
                        {
                            (*HubExtension).HubFlags |= (USBHUB_FDO_FLAG_NOT_D0_STATE
                                | USBHUB_FDO_FLAG_DEVICE_STOPPING)
                                as core::ffi::c_ulong;

                            IoCancelIrp((*HubExtension).SCEIrp);

                            KeWaitForSingleObject(
                                core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent)
                                    as PVOID,
                                _KWAIT_REASON_Executive,
                                0i32 as i32 as KPROCESSOR_MODE,
                                FALSE as BOOLEAN,
                                core::ptr::null_mut(),
                            );
                        }

                        (*HubExtension).CurrentPowerState.DeviceState = PowerState.DeviceState;

                        if (*HubExtension).HubFlags as core::ffi::c_ulong
                            & USBHUB_FDO_FLAG_DO_SUSPENSE as core::ffi::c_ulong
                            != 0
                            && USBH_CheckIdleAbort(
                                HubExtension,
                                TRUE as BOOLEAN,
                                TRUE as BOOLEAN,
                            ) as i32
                                == TRUE as i32
                        {
                            (*HubExtension).HubFlags &= !(USBHUB_FDO_FLAG_NOT_D0_STATE
                                | USBHUB_FDO_FLAG_DEVICE_STOPPING)
                                as core::ffi::c_ulong;

                            (*HubExtension).CurrentPowerState.DeviceState = PowerDeviceD0;

                            USBH_SubmitStatusChangeTransfer(HubExtension);

                            PoStartNextPowerIrp(Irp);

                            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
                            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

                            (*HubExtension).HubFlags &=
                                !USBHUB_FDO_FLAG_DO_SUSPENSE as core::ffi::c_ulong;

                            KeReleaseSemaphore(
                                core::ptr::addr_of_mut!((*HubExtension).IdleSemaphore),
                                LOW_REALTIME_PRIORITY as KPRIORITY,
                                1 as LONG,
                                FALSE as BOOLEAN,
                            );

                            return STATUS_UNSUCCESSFUL;
                        }

                        IoCopyCurrentIrpStackLocationToNext(Irp);

                        IoSetCompletionRoutine(
                            Irp,
                            core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(Some(USBH_PowerIrpCompletion as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS)),
                            HubExtension as PVOID,
                            TRUE as BOOLEAN,
                            TRUE as BOOLEAN,
                            TRUE as BOOLEAN,
                        );

                        PoStartNextPowerIrp(Irp);
                        IoMarkIrpPending(Irp);
                        PoCallDriver((*HubExtension).LowerDevice, Irp);

                        if (*HubExtension).HubFlags as core::ffi::c_ulong
                            & USBHUB_FDO_FLAG_DO_SUSPENSE as core::ffi::c_ulong
                            != 0
                        {
                            (*HubExtension).HubFlags &=
                                !USBHUB_FDO_FLAG_DO_SUSPENSE as core::ffi::c_ulong;

                            KeReleaseSemaphore(
                                core::ptr::addr_of_mut!((*HubExtension).IdleSemaphore),
                                LOW_REALTIME_PRIORITY as KPRIORITY,
                                1 as LONG,
                                FALSE as BOOLEAN,
                            );
                        }

                        return STATUS_PENDING;
                    }
                    _ => {
                        DbgBreakPoint();
                    }
                }
            } else {
                let mut current_block_132: u64;
                if PowerState.SystemState as u32 != PowerSystemWorking as i32 as u32 {
                    (*USBH_GetRootHubExtension(HubExtension))
                        .SystemPowerState
                        .SystemState = PowerState.SystemState;
                }

                if PowerState.SystemState as u32 == PowerSystemHibernate as i32 as u32 {
                    (*HubExtension).HubFlags |=
                        USBHUB_FDO_FLAG_HIBERNATE_STATE as core::ffi::c_ulong;
                }

                PortData = (*HubExtension).PortData;

                IsAllPortsD3 = TRUE as BOOLEAN;

                if !PortData.is_null() && !(*HubExtension).HubDescriptor.is_null() {
                    Port = 0 as ULONG;
                    while Port
                        < (*(*HubExtension).HubDescriptor).bNumberOfPorts as core::ffi::c_ulong
                    {
                        PdoDevice = (*PortData.offset(Port as isize)).DeviceObject;

                        if !PdoDevice.is_null() {
                            PortExtension =
                                (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

                            if (*PortExtension).CurrentPowerState.DeviceState as u32
                                != PowerDeviceD3 as i32 as u32
                            {
                                IsAllPortsD3 = FALSE as BOOLEAN;
                                break;
                            }
                        }
                        Port = Port.wrapping_add(1);
                    }
                }

                if PowerState.SystemState as u32 == PowerSystemWorking as i32 as u32 {
                    DevicePwrState.DeviceState = PowerDeviceD0;
                    current_block_132 = 4691324637564808323;
                } else if (*HubExtension).HubFlags as core::ffi::c_ulong
                    & USBHUB_FDO_FLAG_PENDING_WAKE_IRP as core::ffi::c_ulong
                    != 0
                    || IsAllPortsD3 == 0
                {
                    DevicePwrState.DeviceState =
                        (*HubExtension).DeviceState[PowerState.SystemState as usize];

                    if DevicePwrState.DeviceState as u32 == PowerDeviceUnspecified as i32 as u32 {
                        current_block_132 = 10821978883506768420;
                    } else {
                        current_block_132 = 4691324637564808323;
                    }
                } else {
                    DevicePwrState.DeviceState = PowerDeviceD3;
                    current_block_132 = 4691324637564808323;
                }

                match current_block_132 {
                    4691324637564808323 => {
                        if DevicePwrState.DeviceState as u32
                            != (*HubExtension).CurrentPowerState.DeviceState as u32
                            && (*HubExtension).HubFlags as core::ffi::c_ulong
                                & USBHUB_FDO_FLAG_DEVICE_STARTED as core::ffi::c_ulong
                                != 0
                        {
                            (*HubExtension).PowerIrp = Irp;

                            IoMarkIrpPending(Irp);

                            if PoRequestPowerIrp(
                                (*HubExtension).LowerPDO,
                                IRP_MN_SET_POWER as UCHAR,
                                DevicePwrState,
                                core::mem::transmute::<_, PREQUEST_POWER_COMPLETE>(Some(USBH_FdoDeferPoRequestCompletion as unsafe extern "system" fn(PDEVICE_OBJECT, UCHAR, POWER_STATE, PVOID, PIO_STATUS_BLOCK))),
                                HubExtension as PVOID,
                                core::ptr::null_mut(),
                            ) == STATUS_PENDING
                            {
                                return STATUS_PENDING;
                            }

                            IoCopyCurrentIrpStackLocationToNext(Irp);
                            PoStartNextPowerIrp(Irp);
                            PoCallDriver((*HubExtension).LowerDevice, Irp);

                            return STATUS_PENDING;
                        }
                    }
                    _ => {}
                }

                (*HubExtension).SystemPowerState.SystemState = PowerState.SystemState;

                if PowerState.SystemState as u32 == PowerSystemWorking as i32 as u32 {
                    USBH_CheckIdleDeferred(HubExtension);
                }

                IoCopyCurrentIrpStackLocationToNext(Irp);
                PoStartNextPowerIrp(Irp);

                return PoCallDriver((*HubExtension).LowerDevice, Irp);
            }
        }
        IRP_MN_QUERY_POWER => {}
        _ => {}
    }

    IoCopyCurrentIrpStackLocationToNext(Irp);
    PoStartNextPowerIrp(Irp);
    Status = PoCallDriver((*HubExtension).LowerDevice, Irp);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoPower(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
    Minor: UCHAR,
) -> NTSTATUS {

    let Status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;

    match Minor {
        IRP_MN_WAIT_WAKE => {
            PoStartNextPowerIrp(Irp);
        }
        IRP_MN_POWER_SEQUENCE => {
            PoStartNextPowerIrp(Irp);
        }
        IRP_MN_SET_POWER => {
            PoStartNextPowerIrp(Irp);
        }
        IRP_MN_QUERY_POWER => {
            PoStartNextPowerIrp(Irp);
        }
        _ => {
            if DbgPrint(
                b"(%s:%d) USBHUB_PdoPower: unknown IRP_MN_POWER!\n\0".as_ptr() as *const i8,
                b"power.c\0".as_ptr() as *const i8,
                882i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"power.c\0".as_ptr() as *const i8,
                    882i32,
                );
            }
            PoStartNextPowerIrp(Irp);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Hub Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBHub power handling functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbhub.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBHUB_POWER
// #include "dbg_uhub.h"
//
// VOID
// NTAPI
// USBH_CompletePowerIrp(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                       IN PIRP Irp,
//                       IN NTSTATUS NtStatus)
// {
//     DPRINT("USBH_CompletePowerIrp: HubExtension - %p, Irp - %p, NtStatus - %lX\n",
//            HubExtension,
//            Irp,
//            NtStatus);
//
//     Irp->IoStatus.Status = NtStatus;
//
//     PoStartNextPowerIrp(Irp);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// VOID
// NTAPI
// USBH_HubCancelWakeIrp(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                       IN PIRP Irp)
// {
//     DPRINT("USBH_HubCancelWakeIrp: HubExtension - %p, Irp - %p\n",
//            HubExtension,
//            Irp);
//
//     IoCancelIrp(Irp);
//
//     if (InterlockedExchange((PLONG)&HubExtension->FdoWaitWakeLock, 1))
//     {
//         PoStartNextPowerIrp(Irp);
//         Irp->IoStatus.Status = STATUS_CANCELLED;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
// }
//
// VOID
// NTAPI
// USBH_HubESDRecoverySetD3Completion(IN PDEVICE_OBJECT DeviceObject,
//                                    IN UCHAR MinorFunction,
//                                    IN POWER_STATE PowerState,
//                                    IN PVOID Context,
//                                    IN PIO_STATUS_BLOCK IoStatus)
// {
//     DPRINT("USBH_HubESDRecoverySetD3Completion ... \n");
//
//     KeSetEvent((PRKEVENT)Context,
//                EVENT_INCREMENT,
//                FALSE);
// }
//
// VOID
// NTAPI
// USBH_HubESDRecoverySetD0Completion(IN PDEVICE_OBJECT DeviceObject,
//                                    IN UCHAR MinorFunction,
//                                    IN POWER_STATE PowerState,
//                                    IN PVOID Context,
//                                    IN PIO_STATUS_BLOCK IoStatus)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension = (PUSBHUB_FDO_EXTENSION)Context;
//
//     DPRINT("USBH_HubESDRecoverySetD0Completion: HubExtension - %p\n", HubExtension);
//
//     if (HubExtension && NT_SUCCESS(IoStatus->Status))
//     {
//         /* Clear the ESD recovery flag after successful power cycle */
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_ESD_RECOVERING;
//         DPRINT("USBH_HubESDRecoverySetD0Completion: ESD recovery completed\n");
//     }
//     else if (HubExtension)
//     {
//         /* If D0 transition failed, clear the flag anyway to allow retry */
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_ESD_RECOVERING;
//         DPRINT1("USBH_HubESDRecoverySetD0Completion: ESD recovery failed with status 0x%08lx\n",
//                 IoStatus->Status);
//     }
// }
//
// NTSTATUS
// NTAPI
// USBH_HubSetD0(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSBHUB_FDO_EXTENSION RootHubDevExt;
//     NTSTATUS Status;
//     KEVENT Event;
//     POWER_STATE PowerState;
//
//     DPRINT("USBH_HubSetD0: HubExtension - %p\n", HubExtension);
//
//     RootHubDevExt = USBH_GetRootHubExtension(HubExtension);
//
//     if (RootHubDevExt->SystemPowerState.SystemState != PowerSystemWorking)
//     {
//         Status = STATUS_INVALID_DEVICE_STATE;
//         return Status;
//     }
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST)
//     {
//         DPRINT("USBH_HubSetD0: HubFlags - %lX\n", HubExtension->HubFlags);
//
//         KeWaitForSingleObject(&HubExtension->IdleEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     PowerState.DeviceState = PowerDeviceD0;
//
//     Status = PoRequestPowerIrp(HubExtension->LowerPDO,
//                                IRP_MN_SET_POWER,
//                                PowerState,
//                                USBH_HubESDRecoverySetD3Completion,
//                                &Event,
//                                NULL);
//
//     if (Status == STATUS_PENDING)
//     {
//        Status = KeWaitForSingleObject(&Event,
//                                       Suspended,
//                                       KernelMode,
//                                       FALSE,
//                                       NULL);
//     }
//
//     while (HubExtension->HubFlags & USBHUB_FDO_FLAG_WAKEUP_START)
//     {
//         USBH_Wait(10);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_HubStartESDRecovery(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSBHUB_FDO_EXTENSION RootHubDevExt;
//     NTSTATUS Status;
//     KEVENT Event;
//     POWER_STATE PowerState;
//
//     DPRINT("USBH_HubStartESDRecovery: HubExtension - %p\n", HubExtension);
//
//     RootHubDevExt = USBH_GetRootHubExtension(HubExtension);
//
//     if (RootHubDevExt->SystemPowerState.SystemState != PowerSystemWorking)
//     {
//         Status = STATUS_INVALID_DEVICE_STATE;
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_ESD_RECOVERING;
//         return Status;
//     }
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* First, set the hub to D3 (power off) */
//     PowerState.DeviceState = PowerDeviceD3;
//
//     Status = PoRequestPowerIrp(HubExtension->LowerPDO,
//                                IRP_MN_SET_POWER,
//                                PowerState,
//                                USBH_HubESDRecoverySetD3Completion,
//                                &Event,
//                                NULL);
//     if (Status == STATUS_PENDING)
//     {
//         Status = KeWaitForSingleObject(&Event,
//                                        Suspended,
//                                        KernelMode,
//                                        FALSE,
//                                        NULL);
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_HubStartESDRecovery: Failed to set D3, Status - %lX\n", Status);
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_ESD_RECOVERING;
//         return Status;
//     }
//
//     /* Now set it back to D0 (power on) to complete the recovery */
//     PowerState.DeviceState = PowerDeviceD0;
//
//     Status = PoRequestPowerIrp(HubExtension->LowerPDO,
//                                IRP_MN_SET_POWER,
//                                PowerState,
//                                USBH_HubESDRecoverySetD0Completion,
//                                HubExtension,
//                                NULL);
//     if (!NT_SUCCESS(Status) && Status != STATUS_PENDING)
//     {
//         DPRINT1("USBH_HubStartESDRecovery: Failed to set D0, Status - %lX\n", Status);
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_ESD_RECOVERING;
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_IdleCancelPowerHubWorker(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                               IN PVOID Context)
// {
//     PUSBHUB_IDLE_PORT_CANCEL_CONTEXT WorkItemIdlePower;
//     PIRP Irp;
//
//     DPRINT("USBH_IdleCancelPowerHubWorker: ... \n");
//
//     WorkItemIdlePower = Context;
//
//     if (HubExtension &&
//         HubExtension->CurrentPowerState.DeviceState != PowerDeviceD0 &&
//         HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED)
//     {
//         USBH_HubSetD0(HubExtension);
//     }
//
//     Irp = WorkItemIdlePower->Irp;
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// VOID
// NTAPI
// USBH_HubQueuePortWakeIrps(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                           IN PLIST_ENTRY ListIrps)
// {
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     USHORT NumPorts;
//     USHORT Port;
//     PIRP WakeIrp;
//     KIRQL OldIrql;
//
//     DPRINT("USBH_HubQueuePortWakeIrps ... \n");
//
//     NumPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     InitializeListHead(ListIrps);
//
//     IoAcquireCancelSpinLock(&OldIrql);
//
//     for (Port = 0; Port < NumPorts; ++Port)
//     {
//         PortDevice = HubExtension->PortData[Port].DeviceObject;
//
//         if (PortDevice)
//         {
//             PortExtension = PortDevice->DeviceExtension;
//
//             WakeIrp = PortExtension->PdoWaitWakeIrp;
//             PortExtension->PdoWaitWakeIrp = NULL;
//
//             if (WakeIrp)
//             {
//                 DPRINT1("USBH_HubQueuePortWakeIrps: UNIMPLEMENTED. FIXME\n");
//                 DbgBreakPoint();
//             }
//         }
//     }
//
//     IoReleaseCancelSpinLock(OldIrql);
// }
//
// VOID
// NTAPI
// USBH_HubCompleteQueuedPortWakeIrps(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                    IN PLIST_ENTRY ListIrps,
//                                    IN NTSTATUS NtStatus)
// {
//     DPRINT("USBH_HubCompleteQueuedPortWakeIrps ... \n");
//
//     while (!IsListEmpty(ListIrps))
//     {
//         DPRINT1("USBH_HubCompleteQueuedPortWakeIrps: UNIMPLEMENTED. FIXME\n");
//         DbgBreakPoint();
//     }
// }
//
// VOID
// NTAPI
// USBH_HubCompletePortWakeIrps(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                              IN NTSTATUS NtStatus)
// {
//     LIST_ENTRY ListIrps;
//
//     DPRINT("USBH_HubCompletePortWakeIrps: NtStatus - %x\n", NtStatus);
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED)
//     {
//         USBH_HubQueuePortWakeIrps(HubExtension, &ListIrps);
//
//         USBH_HubCompleteQueuedPortWakeIrps(HubExtension,
//                                            &ListIrps,
//                                            NtStatus);
//     }
// }
//
// VOID
// NTAPI
// USBH_FdoPoRequestD0Completion(IN PDEVICE_OBJECT DeviceObject,
//                               IN UCHAR MinorFunction,
//                               IN POWER_STATE PowerState,
//                               IN PVOID Context,
//                               IN PIO_STATUS_BLOCK IoStatus)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//
//     DPRINT("USBH_FdoPoRequestD0Completion ... \n");
//
//     HubExtension = Context;
//
//     USBH_HubCompletePortWakeIrps(HubExtension, STATUS_SUCCESS);
//
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_WAKEUP_START;
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
// }
//
// VOID
// NTAPI
// USBH_CompletePortWakeIrpsWorker(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                 IN PVOID Context)
// {
//     DPRINT1("USBH_CompletePortWakeIrpsWorker: UNIMPLEMENTED. FIXME\n");
//     DbgBreakPoint();
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoWWIrpIoCompletion(IN PDEVICE_OBJECT DeviceObject,
//                           IN PIRP Irp,
//                           IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//     POWER_STATE PowerState;
//     PIRP WakeIrp;
//
//     DPRINT("USBH_FdoWWIrpIoCompletion: DeviceObject - %p, Irp - %p\n",
//             DeviceObject,
//             Irp);
//
//     HubExtension = Context;
//
//     Status = Irp->IoStatus.Status;
//
//     IoAcquireCancelSpinLock(&OldIrql);
//
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_PENDING_WAKE_IRP;
//
//     WakeIrp = InterlockedExchangePointer((PVOID *)&HubExtension->PendingWakeIrp,
//                                          NULL);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     IoReleaseCancelSpinLock(OldIrql);
//
//     DPRINT("USBH_FdoWWIrpIoCompletion: Status - %lX\n", Status);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_FdoWWIrpIoCompletion: DbgBreakPoint() \n");
//         DbgBreakPoint();
//     }
//     else
//     {
//         PowerState.DeviceState = PowerDeviceD0;
//
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_WAKEUP_START;
//         InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//         Status = STATUS_SUCCESS;
//
//         PoRequestPowerIrp(HubExtension->LowerPDO,
//                           IRP_MN_SET_POWER,
//                           PowerState,
//                           USBH_FdoPoRequestD0Completion,
//                           (PVOID)HubExtension,
//                           NULL);
//     }
//
//     if (!WakeIrp)
//     {
//         if (!InterlockedExchange(&HubExtension->FdoWaitWakeLock, 1))
//         {
//             Status = STATUS_MORE_PROCESSING_REQUIRED;
//         }
//     }
//
//     DPRINT("USBH_FdoWWIrpIoCompletion: Status - %lX\n", Status);
//
//     if (Status != STATUS_MORE_PROCESSING_REQUIRED)
//     {
//         PoStartNextPowerIrp(Irp);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PowerIrpCompletion(IN PDEVICE_OBJECT DeviceObject,
//                         IN PIRP Irp,
//                         IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PIO_STACK_LOCATION IoStack;
//     DEVICE_POWER_STATE OldDeviceState;
//     NTSTATUS Status;
//     POWER_STATE PowerState;
//
//     DPRINT("USBH_PowerIrpCompletion: DeviceObject - %p, Irp - %p\n",
//            DeviceObject,
//            Irp);
//
//     HubExtension = Context;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     PowerState = IoStack->Parameters.Power.State;
//
//     Status = Irp->IoStatus.Status;
//     DPRINT("USBH_PowerIrpCompletion: Status - %lX\n", Status);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (PowerState.DeviceState == PowerDeviceD0)
//         {
//             PoStartNextPowerIrp(Irp);
//             HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_SET_D0_STATE;
//         }
//     }
//     else if (PowerState.DeviceState == PowerDeviceD0)
//     {
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_SET_D0_STATE;
//
//         OldDeviceState = HubExtension->CurrentPowerState.DeviceState;
//         HubExtension->CurrentPowerState.DeviceState = PowerDeviceD0;
//
//         DPRINT("USBH_PowerIrpCompletion: OldDeviceState - %x\n", OldDeviceState);
//
//         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_HIBERNATE_STATE)
//         {
//             DPRINT1("USBH_PowerIrpCompletion: USBHUB_FDO_FLAG_HIBERNATE_STATE. FIXME\n");
//             DbgBreakPoint();
//         }
//
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_HIBERNATE_STATE;
//
//         if (OldDeviceState == PowerDeviceD3)
//         {
//             DPRINT1("USBH_PowerIrpCompletion: PowerDeviceD3. FIXME\n");
//             DbgBreakPoint();
//         }
//
//         if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED) &&
//             HubExtension->HubFlags & USBHUB_FDO_FLAG_DO_ENUMERATION)
//         {
//             USBH_SubmitStatusChangeTransfer(HubExtension);
//         }
//
//         DPRINT("USBH_PowerIrpCompletion: Status - %lX\n", Status);
//
//         if (Status != STATUS_MORE_PROCESSING_REQUIRED)
//         {
//             PoStartNextPowerIrp(Irp);
//             return Status;
//         }
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_FdoDeferPoRequestCompletion(IN PDEVICE_OBJECT DeviceObject,
//                                  IN UCHAR MinorFunction,
//                                  IN POWER_STATE PowerState,
//                                  IN PVOID Context,
//                                  IN PIO_STATUS_BLOCK IoStatus)
// {
//     PUSBHUB_FDO_EXTENSION Extension;
//     PUSBHUB_FDO_EXTENSION HubExtension = NULL;
//     PIRP PowerIrp;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT("USBH_FdoDeferPoRequestCompletion ... \n");
//
//     Extension = Context;
//
//     PowerIrp = Extension->PowerIrp;
//
//     if (Extension->Common.ExtensionType == USBH_EXTENSION_TYPE_HUB)
//     {
//         HubExtension = Context;
//     }
//
//     IoStack = IoGetCurrentIrpStackLocation(PowerIrp);
//
//     if (IoStack->Parameters.Power.State.SystemState == PowerSystemWorking &&
//         HubExtension && HubExtension->LowerPDO == HubExtension->RootHubPdo)
//     {
//         HubExtension->SystemPowerState.SystemState = PowerSystemWorking;
//         USBH_CheckIdleDeferred(HubExtension);
//     }
//
//     IoCopyCurrentIrpStackLocationToNext(PowerIrp);
//     PoStartNextPowerIrp(PowerIrp);
//     PoCallDriver(Extension->LowerDevice, PowerIrp);
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoPower(IN PUSBHUB_FDO_EXTENSION HubExtension,
//               IN PIRP Irp,
//               IN UCHAR Minor)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     POWER_STATE PowerState;
//     POWER_STATE DevicePwrState;
//     BOOLEAN IsAllPortsD3;
//     PUSBHUB_PORT_DATA PortData;
//     PDEVICE_OBJECT PdoDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     ULONG Port;
//
//     DPRINT_PWR("USBH_FdoPower: HubExtension - %p, Irp - %p, Minor - %X\n",
//                HubExtension,
//                Irp,
//                Minor);
//
//     switch (Minor)
//     {
//         case IRP_MN_WAIT_WAKE:
//             DPRINT_PWR("USBH_FdoPower: IRP_MN_WAIT_WAKE\n");
//
//             IoCopyCurrentIrpStackLocationToNext(Irp);
//
//             IoSetCompletionRoutine(Irp,
//                                    USBH_FdoWWIrpIoCompletion,
//                                    HubExtension,
//                                    TRUE,
//                                    TRUE,
//                                    TRUE);
//
//             PoStartNextPowerIrp(Irp);
//             IoMarkIrpPending(Irp);
//             PoCallDriver(HubExtension->LowerDevice, Irp);
//
//             return STATUS_PENDING;
//
//         case IRP_MN_POWER_SEQUENCE:
//             DPRINT_PWR("USBH_FdoPower: IRP_MN_POWER_SEQUENCE\n");
//             break;
//
//         case IRP_MN_SET_POWER:
//             DPRINT_PWR("USBH_FdoPower: IRP_MN_SET_POWER\n");
//
//             IoStack = IoGetCurrentIrpStackLocation(Irp);
//             DPRINT_PWR("USBH_FdoPower: IRP_MN_SET_POWER/DevicePowerState\n");
//             PowerState = IoStack->Parameters.Power.State;
//
//             if (IoStack->Parameters.Power.Type == DevicePowerState)
//             {
//                 DPRINT_PWR("USBH_FdoPower: PowerState - %x\n",
//                            PowerState.DeviceState);
//
//                 if (HubExtension->CurrentPowerState.DeviceState == PowerState.DeviceState)
//                 {
//                     IoCopyCurrentIrpStackLocationToNext(Irp);
//
//                     PoStartNextPowerIrp(Irp);
//                     IoMarkIrpPending(Irp);
//                     PoCallDriver(HubExtension->LowerDevice, Irp);
//
//                     return STATUS_PENDING;
//                 }
//
//                 switch (PowerState.DeviceState)
//                 {
//                     case PowerDeviceD0:
//                         if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_SET_D0_STATE))
//                         {
//                             HubExtension->HubFlags &= ~(USBHUB_FDO_FLAG_NOT_D0_STATE |
//                                                         USBHUB_FDO_FLAG_DEVICE_STOPPING);
//
//                             HubExtension->HubFlags |= USBHUB_FDO_FLAG_SET_D0_STATE;
//
//                             IoCopyCurrentIrpStackLocationToNext(Irp);
//
//                             IoSetCompletionRoutine(Irp,
//                                                    USBH_PowerIrpCompletion,
//                                                    HubExtension,
//                                                    TRUE,
//                                                    TRUE,
//                                                    TRUE);
//                         }
//                         else
//                         {
//                             IoCopyCurrentIrpStackLocationToNext(Irp);
//                             PoStartNextPowerIrp(Irp);
//                         }
//
//                         IoMarkIrpPending(Irp);
//                         PoCallDriver(HubExtension->LowerDevice, Irp);
//                         return STATUS_PENDING;
//
//                     case PowerDeviceD1:
//                     case PowerDeviceD2:
//                     case PowerDeviceD3:
//                         if (HubExtension->ResetRequestCount)
//                         {
//                             IoCancelIrp(HubExtension->ResetPortIrp);
//
//                             KeWaitForSingleObject(&HubExtension->ResetEvent,
//                                                   Executive,
//                                                   KernelMode,
//                                                   FALSE,
//                                                   NULL);
//                         }
//
//                         if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//                         {
//                             HubExtension->HubFlags |= (USBHUB_FDO_FLAG_NOT_D0_STATE |
//                                                        USBHUB_FDO_FLAG_DEVICE_STOPPING);
//
//                             IoCancelIrp(HubExtension->SCEIrp);
//
//                             KeWaitForSingleObject(&HubExtension->StatusChangeEvent,
//                                                   Executive,
//                                                   KernelMode,
//                                                   FALSE,
//                                                   NULL);
//                         }
//
//                         HubExtension->CurrentPowerState.DeviceState = PowerState.DeviceState;
//
//                         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DO_SUSPENSE &&
//                             USBH_CheckIdleAbort(HubExtension, TRUE, TRUE) == TRUE)
//                         {
//                             HubExtension->HubFlags &= ~(USBHUB_FDO_FLAG_NOT_D0_STATE |
//                                                         USBHUB_FDO_FLAG_DEVICE_STOPPING);
//
//                             HubExtension->CurrentPowerState.DeviceState = PowerDeviceD0;
//
//                             USBH_SubmitStatusChangeTransfer(HubExtension);
//
//                             PoStartNextPowerIrp(Irp);
//
//                             Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//                             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//                             HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_DO_SUSPENSE;
//
//                             KeReleaseSemaphore(&HubExtension->IdleSemaphore,
//                                                LOW_REALTIME_PRIORITY,
//                                                1,
//                                                FALSE);
//
//                             return STATUS_UNSUCCESSFUL;
//                         }
//
//                         IoCopyCurrentIrpStackLocationToNext(Irp);
//
//                         IoSetCompletionRoutine(Irp,
//                                                USBH_PowerIrpCompletion,
//                                                HubExtension,
//                                                TRUE,
//                                                TRUE,
//                                                TRUE);
//
//                         PoStartNextPowerIrp(Irp);
//                         IoMarkIrpPending(Irp);
//                         PoCallDriver(HubExtension->LowerDevice, Irp);
//
//                         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DO_SUSPENSE)
//                         {
//                             HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_DO_SUSPENSE;
//
//                             KeReleaseSemaphore(&HubExtension->IdleSemaphore,
//                                                LOW_REALTIME_PRIORITY,
//                                                1,
//                                                FALSE);
//                         }
//
//                         return STATUS_PENDING;
//
//                     default:
//                         DPRINT1("USBH_FdoPower: Unsupported PowerState.DeviceState\n");
//                         DbgBreakPoint();
//                         break;
//                 }
//             }
//             else
//             {
//                 if (PowerState.SystemState != PowerSystemWorking)
//                 {
//                     USBH_GetRootHubExtension(HubExtension)->SystemPowerState.SystemState =
//                                                             PowerState.SystemState;
//                 }
//
//                 if (PowerState.SystemState == PowerSystemHibernate)
//                 {
//                     HubExtension->HubFlags |= USBHUB_FDO_FLAG_HIBERNATE_STATE;
//                 }
//
//                 PortData = HubExtension->PortData;
//
//                 IsAllPortsD3 = TRUE;
//
//                 if (PortData && HubExtension->HubDescriptor)
//                 {
//                     for (Port = 0;
//                          Port < HubExtension->HubDescriptor->bNumberOfPorts;
//                          Port++)
//                     {
//                         PdoDevice = PortData[Port].DeviceObject;
//
//                         if (PdoDevice)
//                         {
//                             PortExtension = PdoDevice->DeviceExtension;
//
//                             if (PortExtension->CurrentPowerState.DeviceState != PowerDeviceD3)
//                             {
//                                 IsAllPortsD3 = FALSE;
//                                 break;
//                             }
//                         }
//                     }
//                 }
//
//                 if (PowerState.SystemState == PowerSystemWorking)
//                 {
//                     DevicePwrState.DeviceState = PowerDeviceD0;
//                 }
//                 else if (HubExtension->HubFlags & USBHUB_FDO_FLAG_PENDING_WAKE_IRP ||
//                          !IsAllPortsD3)
//                 {
//                     DevicePwrState.DeviceState = HubExtension->DeviceState[PowerState.SystemState];
//
//                     if (DevicePwrState.DeviceState == PowerDeviceUnspecified)
//                     {
//                         goto Exit;
//                     }
//                 }
//                 else
//                 {
//                     DevicePwrState.DeviceState = PowerDeviceD3;
//                 }
//
//                 if (DevicePwrState.DeviceState != HubExtension->CurrentPowerState.DeviceState &&
//                     HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED)
//                 {
//                     HubExtension->PowerIrp = Irp;
//
//                     IoMarkIrpPending(Irp);
//
//                     if (PoRequestPowerIrp(HubExtension->LowerPDO,
//                                           IRP_MN_SET_POWER,
//                                           DevicePwrState,
//                                           USBH_FdoDeferPoRequestCompletion,
//                                           (PVOID)HubExtension,
//                                           NULL) == STATUS_PENDING)
//                     {
//                         return STATUS_PENDING;
//                     }
//
//                     IoCopyCurrentIrpStackLocationToNext(Irp);
//                     PoStartNextPowerIrp(Irp);
//                     PoCallDriver(HubExtension->LowerDevice, Irp);
//
//                     return STATUS_PENDING;
//                 }
//
//             Exit:
//
//                 HubExtension->SystemPowerState.SystemState = PowerState.SystemState;
//
//                 if (PowerState.SystemState == PowerSystemWorking)
//                 {
//                     USBH_CheckIdleDeferred(HubExtension);
//                 }
//
//                 IoCopyCurrentIrpStackLocationToNext(Irp);
//                 PoStartNextPowerIrp(Irp);
//
//                 return PoCallDriver(HubExtension->LowerDevice, Irp);
//             }
//
//             break;
//
//         case IRP_MN_QUERY_POWER:
//             DPRINT_PWR("USBH_FdoPower: IRP_MN_QUERY_POWER\n");
//             break;
//
//         default:
//             DPRINT1("USBH_FdoPower: unknown IRP_MN_POWER!\n");
//             break;
//     }
//
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//     PoStartNextPowerIrp(Irp);
//     Status = PoCallDriver(HubExtension->LowerDevice, Irp);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoPower(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//               IN PIRP Irp,
//               IN UCHAR Minor)
// {
//     NTSTATUS Status = Irp->IoStatus.Status;
//
//     DPRINT_PWR("USBH_FdoPower: PortExtension - %p, Irp - %p, Minor - %X\n",
//                PortExtension,
//                Irp,
//                Minor);
//
//     switch (Minor)
//     {
//       case IRP_MN_WAIT_WAKE:
//           DPRINT_PWR("USBHUB_PdoPower: IRP_MN_WAIT_WAKE\n");
//           PoStartNextPowerIrp(Irp);
//           break;
//
//       case IRP_MN_POWER_SEQUENCE:
//           DPRINT_PWR("USBHUB_PdoPower: IRP_MN_POWER_SEQUENCE\n");
//           PoStartNextPowerIrp(Irp);
//           break;
//
//       case IRP_MN_SET_POWER:
//           DPRINT_PWR("USBHUB_PdoPower: IRP_MN_SET_POWER\n");
//           PoStartNextPowerIrp(Irp);
//           break;
//
//       case IRP_MN_QUERY_POWER:
//           DPRINT_PWR("USBHUB_PdoPower: IRP_MN_QUERY_POWER\n");
//           PoStartNextPowerIrp(Irp);
//           break;
//
//       default:
//           DPRINT1("USBHUB_PdoPower: unknown IRP_MN_POWER!\n");
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
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================