use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAcquireCancelSpinLock, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoGetDeviceProperty, IoMarkIrpPending, IoReleaseCancelSpinLock, IoSetCancelRoutine, IoSetCompletionRoutine, KeReleaseSemaphore, KeWaitForSingleObject, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, RtlAssert, WORK_QUEUE_TYPE};
// bindgen enum members (faithful prefixed aliases)
use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue;
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyDriverKeyName as DevicePropertyDriverKeyName;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0;
// enum members for USB device/connection/hub-node speed (from psdk/usbioctl.h, usb200.h)
use crate::types::_USB_CONNECTION_STATUS_DeviceConnected as DeviceConnected;
use crate::types::_USB_CONNECTION_STATUS_DeviceHubNestedTooDeeply as DeviceHubNestedTooDeeply;
use crate::types::_USB_HUB_NODE_UsbHub as UsbHub;
use crate::types::_USB_DEVICE_SPEED_UsbLowSpeed as UsbLowSpeed;
use core::ffi::c_void;

extern "system" {
    fn URB_FROM_IRP(Irp: PIRP) -> PURB;
    fn USBH_HubCancelIdleIrp(HubExtension: PUSBHUB_FDO_EXTENSION, PendingIdleIrp: PIRP);
}
// wcschr is cdecl CRT (`_wcschr`), NOT NTAPI/stdcall — extern "system" emitted `_wcschr@8`
// (undefined). Resolves from the ntoskrnl import lib as cdecl.
extern "C" {
    fn wcschr(s: *const wchar_t, c: ::core::ffi::c_int) -> *mut wchar_t;
}


// PUSB_IDLE_CALLBACK_INFO is not emitted by bindgen (only the value type); usbioctl.h pointer typedef.
type PUSB_IDLE_CALLBACK_INFO = *mut USB_IDLE_CALLBACK_INFO;

// ---- file-scope preamble (translated) ----
// (no output — both `#define` macros are object-like with no value and produce no Rust declarations)

const USBD_PORT_ENABLED: u32 = 0x00000001;
const USBD_PORT_CONNECTED: u32 = 0x00000002;
const EVENT_INCREMENT: KPRIORITY = 1;

const USBD_MARK_DEVICE_BUSY: ULONG = 0x2; // sdk/include/ddk/hubbusif.h
const USBHUB_PDO_FLAG_PORT_RESSETING: ULONG = 0x00080000; // FIXED 2026-07-06: translation invented 0x80; real value is a (1<<N) define in usbhub.h
const USBHUB_PDO_FLAG_PORT_RESTORE_FAIL: ULONG = 0x00001000; // FIXED 2026-07-06: translation invented 0x100; real value is a (1<<N) define in usbhub.h

// USBD_STATUS_* from sdk/include/psdk/usb.h
const USBD_STATUS_INVALID_PARAMETER: USBD_STATUS = 0x80000300u32 as USBD_STATUS; // usb.h:192
const USBD_STATUS_NO_BANDWIDTH: USBD_STATUS = 0x80000700u32 as USBD_STATUS; // usb.h:195
const USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR: USBD_STATUS = 0xC0000F00u32 as USBD_STATUS; // usb.h:203

// STATUS_* from sdk/include/psdk/ntstatus.h (not in ntkrnl_sys::constants)
const STATUS_DEVICE_POWERED_OFF: NTSTATUS = 0x8000000Fu32 as NTSTATUS; // ntstatus.h:275
const STATUS_INVALID_DEVICE_STATE: NTSTATUS = 0xC0000184u32 as NTSTATUS; // ntstatus.h:715
const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as NTSTATUS; // ntstatus.h:277

// IOCTL_INTERNAL_USB_* — CTL_CODE(FILE_DEVICE_USB=FILE_DEVICE_UNKNOWN=0x22, func, method, 0)
// = (0x22<<16)|(func<<2)|method.  func/method from usbiodef.h + usbioctl.h.
// METHOD_NEITHER=3, METHOD_BUFFERED=0.
const IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO: ULONG = 0x0022000F; // func 3, NEITHER
const IOCTL_INTERNAL_USB_RESET_PORT: ULONG = 0x00220007; // func 1, NEITHER
const IOCTL_INTERNAL_USB_GET_PORT_STATUS: ULONG = 0x00220013; // func 4, NEITHER
const IOCTL_INTERNAL_USB_ENABLE_PORT: ULONG = 0x00220017; // func 5, NEITHER
const IOCTL_INTERNAL_USB_GET_HUB_COUNT: ULONG = 0x0022001B; // func 6, NEITHER
const IOCTL_INTERNAL_USB_CYCLE_PORT: ULONG = 0x0022001F; // func 7, NEITHER
const IOCTL_INTERNAL_USB_GET_HUB_NAME: ULONG = 0x00220020; // func 8, BUFFERED
const IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION: ULONG = 0x00220027; // func 9, NEITHER
const IOCTL_INTERNAL_USB_GET_BUS_INFO: ULONG = 0x00220420; // func 264, BUFFERED
const IOCTL_INTERNAL_USB_GET_PARENT_HUB_INFO: ULONG = 0x0022042C; // func 267, BUFFERED
const IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE: ULONG = 0x00220433; // func 268, NEITHER
const IOCTL_GET_HCD_DRIVERKEY_NAME: ULONG = 0x00220424; // HCD func 265, BUFFERED

#[no_mangle]
pub unsafe extern "system" fn USBH_SelectConfigOrInterfaceComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {

    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let TimeoutContext: PVOID;
    let mut PortData: PUSBHUB_PORT_DATA = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;
    let Urb: PURB;

    if (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }

    PortExtension = Context as PUSBHUB_PORT_PDO_EXTENSION;
    HubExtension = (*PortExtension).HubExtension;

    if !HubExtension.is_null() {
        PortData = (*HubExtension)
            .PortData
            .offset(((*PortExtension).PortNumber as i32 - 1) as isize)
            as PUSBHUB_PORT_DATA;
    }

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*PortExtension).PortTimeoutSpinLock) as PKSPIN_LOCK,
            core::ptr::addr_of_mut!(OldIrql),
        );

        TimeoutContext = (*PortExtension).BndwTimeoutContext;

        if !TimeoutContext.is_null() {
            DbgBreakPoint();
        }

        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*PortExtension).PortTimeoutSpinLock) as PKSPIN_LOCK,
            OldIrql,
        );

        (*PortExtension).PortPdoFlags &=
            !(USBHUB_PDO_FLAG_PORT_RESTORE_FAIL | USBHUB_PDO_FLAG_ALLOC_BNDW_FAILED) as u32;

        if !PortData.is_null()
            && (*PortData).ConnectionStatus as u32 != DeviceHubNestedTooDeeply as u32
        {
            (*PortData).ConnectionStatus = DeviceConnected;
        }
    } else {
        Urb = URB_FROM_IRP(Irp);
        if (*Urb).__bindgen_anon_1.UrbHeader.as_ref().Status == USBD_STATUS_NO_BANDWIDTH {
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoUrbFilter(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR;
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let DeviceObject: PDEVICE_OBJECT;
    let Urb: PURB;
    let Function: USHORT;
    let MaxPower: ULONG;
    let mut UrbStatus: USBD_STATUS = 0;
    let mut IsValidConfig: BOOLEAN;

    HubExtension = (*PortExtension).HubExtension;
    DeviceObject = (*PortExtension).Common.SelfDevice;
    let _ = DeviceObject;

    Urb = URB_FROM_IRP(Irp);

    if (*PortExtension).PortPdoFlags
        & (USBHUB_PDO_FLAG_PORT_RESTORE_FAIL | USBHUB_PDO_FLAG_PORT_RESSETING)
        != 0
    {
        (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Status = USBD_STATUS_INVALID_PARAMETER;
        USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    Function = (*Urb).__bindgen_anon_1.UrbHeader.as_ref().Function;

    let current_block_66: u64;
    match Function as u32 {
        URB_FUNCTION_SELECT_CONFIGURATION => {
            ConfigDescriptor = (*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref().ConfigurationDescriptor;

            if !ConfigDescriptor.is_null() {
                IsValidConfig = 1;

                if (*ConfigDescriptor).bDescriptorType as u32 != USB_CONFIGURATION_DESCRIPTOR_TYPE {
                    IsValidConfig = 0;
                    UrbStatus = USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR;
                }

                if ((*ConfigDescriptor).bLength as usize)
                    < core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>()
                {
                    IsValidConfig = 0;
                    UrbStatus = USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR;
                }

                if IsValidConfig == 0 {
                    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Status = UrbStatus;
                    USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
                    return STATUS_INVALID_PARAMETER;
                }

                MaxPower = (2 * (*ConfigDescriptor).MaxPower as i32) as ULONG;
                (*PortExtension).MaxPower = MaxPower;

                if (*HubExtension).MaxPowerPerPort < MaxPower {
                    (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_INSUFFICIENT_PWR;

                    DbgBreakPoint();

                    USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
                    return STATUS_INVALID_PARAMETER;
                }
            }
            current_block_66 = 5039492151439579437;
        }
        URB_FUNCTION_SELECT_INTERFACE => {
            current_block_66 = 5039492151439579437;
        }
        URB_FUNCTION_CONTROL_TRANSFER
        | URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER
        | URB_FUNCTION_ISOCH_TRANSFER => {
            if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_DELETE_PENDING != 0 {
                (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Status = USBD_STATUS_INVALID_PARAMETER;
                USBH_CompleteIrp(Irp, STATUS_DELETE_PENDING);
                return STATUS_DELETE_PENDING;
            }
            current_block_66 = 7343950298149844727;
        }
        URB_FUNCTION_GET_MS_FEATURE_DESCRIPTOR => {
            USBH_CompleteIrp(Irp, STATUS_NOT_IMPLEMENTED);
            return STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            current_block_66 = 7343950298149844727;
        }
    }

    match current_block_66 {
        7343950298149844727 => {}
        _ => {
            IoCopyCurrentIrpStackLocationToNext(Irp);
            IoSetCompletionRoutine(
                Irp,
                core::mem::transmute::<
                    unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS,
                    ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE,
                >(USBH_SelectConfigOrInterfaceComplete),
                PortExtension as *mut core::ffi::c_void,
                1,
                1,
                1,
            );
            return IoCallDriver((*HubExtension).RootHubPdo2, Irp);
        }
    }

    return USBH_PassIrp((*HubExtension).RootHubPdo2, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoIoctlSubmitUrb(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut Urb: PURB;
    let mut Status: NTSTATUS;

    DPRINT_IOCTL!(b"USBH_PdoIoctlSubmitUrb ... \n\0".as_ptr() as *const i8);

    HubExtension = (*PortExtension).HubExtension;

    Urb = (*IoGetCurrentIrpStackLocation(Irp))
        .Parameters
        .Others
        .Argument1 as PURB;

    if (*PortExtension).DeviceHandle.is_null() {
        (*Urb).__bindgen_anon_1.UrbHeader.as_mut().UsbdDeviceHandle = core::ptr::null_mut();
        Status = USBH_PassIrp((*HubExtension).RootHubPdo2, Irp);
    } else {
        (*Urb).__bindgen_anon_1.UrbHeader.as_mut().UsbdDeviceHandle = (*PortExtension).DeviceHandle as PVOID;
        Status = USBH_PdoUrbFilter(PortExtension, Irp);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoIoctlGetPortStatus(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut PortStatus: PULONG;
    let mut Status: NTSTATUS;

    HubExtension = (*PortExtension).HubExtension;

    InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount));

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*HubExtension).HubSemaphore) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    // ASSERT(PortExtension->PortNumber > 0);
    PortData = (*HubExtension)
        .PortData
        .offset(((*PortExtension).PortNumber as i32 - 1) as isize)
        as PUSBHUB_PORT_DATA;

    Status = USBH_SyncGetPortStatus(
        HubExtension,
        (*PortExtension).PortNumber,
        core::ptr::addr_of_mut!((*PortData).PortStatus),
        core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
    );

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    PortStatus = (*IoStack).Parameters.Others.Argument1 as PULONG;

    *PortStatus = 0;

    if (*PortExtension).Common.SelfDevice == (*PortData).DeviceObject {
        if (*PortData)
            .PortStatus
            .__bindgen_anon_1
            .as_ref()
            .PortStatus
            .Usb20PortStatus
            .as_ref()
            .__bindgen_anon_1
            .as_ref()
            .PortEnabledDisabled()
            != 0
        {
            *PortStatus |= USBD_PORT_ENABLED as core::ffi::c_ulong;
        }

        if (*PortData)
            .PortStatus
            .__bindgen_anon_1
            .as_ref()
            .PortStatus
            .Usb20PortStatus
            .as_ref()
            .__bindgen_anon_1
            .as_ref()
            .CurrentConnectStatus()
            != 0
        {
            *PortStatus |= USBD_PORT_CONNECTED as core::ffi::c_ulong;
        }
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).HubSemaphore),
        LOW_REALTIME_PRIORITY as KPRIORITY,
        1,
        0,
    );

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) == 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as *mut c_void,
            EVENT_INCREMENT,
            0,
        );
    }

    USBH_CompleteIrp(Irp, Status);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_ResetPortWorker(
    mut HubExtension: PUSBHUB_FDO_EXTENSION,
    mut Context: PVOID,
) {
    let mut WorkItemReset: PUSBHUB_RESET_PORT_CONTEXT = core::ptr::null_mut();
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION = core::ptr::null_mut();
    let mut DeviceHandle: PUSB_DEVICE_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut Port: USHORT = 0;

    WorkItemReset = Context as PUSBHUB_RESET_PORT_CONTEXT;

    PortExtension = (*WorkItemReset).PortExtension;

    if HubExtension.is_null() {
        Status = STATUS_UNSUCCESSFUL;
    } else {
        InterlockedIncrement(
            core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut core::ffi::c_long,
        );

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).HubSemaphore) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );

        Port = (*PortExtension).PortNumber;
        DeviceHandle = (*PortExtension).DeviceHandle;

        if !(Port as i32 > 0 as i32) {
            RtlAssert(
                b"Port > 0\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/ioctl.c\0"
                    as *const u8 as *const core::ffi::c_char as PVOID,
                337 as ULONG,
                core::ptr::null_mut(),
            );
        } else {
        }

        if (*PortExtension).Common.SelfDevice
            == (*(*HubExtension)
                .PortData
                .offset((Port as i32 - 1 as i32) as isize))
            .DeviceObject
            && !DeviceHandle.is_null()
        {
            USBD_RemoveDeviceEx(HubExtension, DeviceHandle, USBD_MARK_DEVICE_BUSY);

            Status = USBH_ResetDevice(HubExtension, Port, 1 as BOOLEAN, 0 as BOOLEAN);
        } else {
            Status = STATUS_INVALID_PARAMETER;
        }

        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*HubExtension).HubSemaphore),
            LOW_REALTIME_PRIORITY as KPRIORITY,
            1 as LONG,
            0 as BOOLEAN,
        );

        if InterlockedDecrement(
            core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut core::ffi::c_long,
        ) == 0
        {
            KeSetEvent(
                core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as *mut c_void,
                EVENT_INCREMENT,
                0 as BOOLEAN,
            );
        }
    }

    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_PORT_RESSETING as core::ffi::c_ulong;

    USBH_CompleteIrp((*WorkItemReset).Irp, Status);

    (*(*WorkItemReset).PortExtension).PortPdoFlags &=
        !USBHUB_PDO_FLAG_PORT_RESTORE_FAIL as core::ffi::c_ulong;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoIoctlResetPort(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    const USBHUB_PDO_FLAG_PORT_RESSETING: u32 = 0x00080000; // FIXED 2026-07-06: translation invented 0x40; real value is a (1<<N) define in usbhub.h // USBHUB_PDO_FLAG_PORT_RESSETING

    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut HubWorkItemBuffer: PUSBHUB_RESET_PORT_CONTEXT = core::ptr::null_mut();
    let mut HubIoWorkItem: PUSBHUB_IO_WORK_ITEM = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    HubExtension = (*PortExtension).HubExtension;

    // DPRINT("USBH_PdoIoctlResetPort ... \n");

    if (*PortExtension).PortPdoFlags as u32 & USBHUB_PDO_FLAG_PORT_RESSETING != 0 {
        Status = STATUS_UNSUCCESSFUL;
        USBH_CompleteIrp(Irp, Status);
        return Status;
    }

    Status = USBH_AllocateWorkItem(
        HubExtension,
        core::ptr::addr_of_mut!(HubIoWorkItem),
        core::mem::transmute::<
            unsafe extern "system" fn(PUSBHUB_FDO_EXTENSION, PVOID),
            PUSBHUB_WORKER_ROUTINE,
        >(USBH_ResetPortWorker),
        core::mem::size_of::<USBHUB_RESET_PORT_CONTEXT>() as _,
        core::ptr::addr_of_mut!(HubWorkItemBuffer) as *mut PVOID,
        DelayedWorkQueue,
    );

    if !NT_SUCCESS(Status) {
        Status = STATUS_UNSUCCESSFUL;
        USBH_CompleteIrp(Irp, Status);
        return Status;
    }

    (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_PORT_RESSETING;
    IoMarkIrpPending(Irp);

    (*HubWorkItemBuffer).PortExtension = PortExtension;
    (*HubWorkItemBuffer).Irp = Irp;

    Status = STATUS_PENDING;

    USBH_QueueWorkItem((*PortExtension).HubExtension, HubIoWorkItem);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PortIdleNotificationCancelRoutine(
    Device: PDEVICE_OBJECT,
    Irp: PIRP,
) {

    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION =
        core::ptr::null_mut::<_USBHUB_PORT_PDO_EXTENSION>();
    let mut HubExtension: PUSBHUB_FDO_EXTENSION = core::ptr::null_mut::<_USBHUB_FDO_EXTENSION>();
    let mut PendingIdleIrp: PIRP = core::ptr::null_mut();
    let mut HubIoWorkItem: PUSBHUB_IO_WORK_ITEM = core::ptr::null_mut::<_USBHUB_IO_WORK_ITEM>();
    let mut HubWorkItemBuffer: PUSBHUB_IDLE_PORT_CANCEL_CONTEXT =
        core::ptr::null_mut::<_USBHUB_IDLE_PORT_CANCEL_CONTEXT>();
    let mut Status: NTSTATUS;

    PortExtension = (*Device).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
    (*PortExtension).PortPdoFlags &= !(USBHUB_PDO_FLAG_IDLE_NOTIFICATION as u32);

    HubExtension = (*PortExtension).HubExtension;
    if HubExtension.is_null() {
        RtlAssert(
            b"HubExtension\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/ioctl.c\0".as_ptr()
                as *const i8 as PVOID,
            443 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    (*PortExtension).IdleNotificationIrp = core::ptr::null_mut();

    if (*HubExtension).HubFlags as u32 & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST as u32 != 0 {
        PendingIdleIrp = (*HubExtension).PendingIdleIrp;
        (*HubExtension).PendingIdleIrp = core::ptr::null_mut();
    }

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    if !PendingIdleIrp.is_null() {
        USBH_HubCancelIdleIrp(HubExtension, PendingIdleIrp);
    }

    if !((*HubExtension).CurrentPowerState.DeviceState as u32
        == PowerDeviceD0 as i32 as u32)
    {
        Status = USBH_AllocateWorkItem(
            HubExtension,
            core::ptr::addr_of_mut!(HubIoWorkItem),
            core::mem::transmute::<_, PUSBHUB_WORKER_ROUTINE>(
                USBH_IdleCancelPowerHubWorker as *const (),
            ),
            core::mem::size_of::<USBHUB_IDLE_PORT_CANCEL_CONTEXT>() as _,
            core::ptr::addr_of_mut!(HubWorkItemBuffer) as *mut PVOID,
            DelayedWorkQueue,
        );

        if NT_SUCCESS(Status) {
            (*HubWorkItemBuffer).Irp = Irp;
            USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
            return;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PortIdleNotificationRequest(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as NTSTATUS;
    const STATUS_NO_CALLBACK_ACTIVE: NTSTATUS = 0xC0000258u32 as NTSTATUS;
    const USBHUB_PDO_FLAG_IDLE_NOTIFICATION: u32 = 0x00400000; // FIXED 2026-07-06: translation invented 0x800; real value is a (1<<N) define in usbhub.h

    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut IdleCallbackInfo: PUSB_IDLE_CALLBACK_INFO;
    let Status: NTSTATUS;
    let mut Irql: KIRQL = 0;

    HubExtension = (*PortExtension).HubExtension;

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

    if !(*PortExtension).IdleNotificationIrp.is_null() {
        IoReleaseCancelSpinLock(Irql);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_BUSY;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return STATUS_DEVICE_BUSY;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    IdleCallbackInfo =
        (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PUSB_IDLE_CALLBACK_INFO;

    if IdleCallbackInfo.is_null() || (*IdleCallbackInfo).IdleCallback.is_none() {
        IoReleaseCancelSpinLock(Irql);

        Status = STATUS_NO_CALLBACK_ACTIVE;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

        return Status;
    }

    IoSetCancelRoutine(
        Irp,
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP),
            ntkrnl_sys::ddk::PDRIVER_CANCEL,
        >(USBH_PortIdleNotificationCancelRoutine),
    );

    if (*Irp).Cancel != 0 {
        if IoSetCancelRoutine(Irp, None).is_some() {
            IoReleaseCancelSpinLock(Irql);
            Status = STATUS_CANCELLED;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        } else {
            IoMarkIrpPending(Irp);
            IoReleaseCancelSpinLock(Irql);
            Status = STATUS_PENDING;
        }
    } else {
        (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_IDLE_NOTIFICATION as core::ffi::c_ulong;

        (*PortExtension).IdleNotificationIrp = Irp;
        IoMarkIrpPending(Irp);

        IoReleaseCancelSpinLock(Irql);
        Status = STATUS_PENDING;

        USBH_CheckIdleDeferred(HubExtension);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IoctlGetNodeName(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut ConnectionName: PUSB_NODE_CONNECTION_NAME;
    let mut PortDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut LengthSkip: size_t;
    let mut Buffer: PWCHAR;
    let mut BufferLength: ULONG;
    let mut BufferEnd: PWCHAR;
    let mut LengthReturned: ULONG_PTR;
    let mut LengthName: size_t;
    let mut Length: ULONG;
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Information: ULONG_PTR;

    Status = STATUS_SUCCESS;

    ConnectionName = (*Irp).AssociatedIrp.SystemBuffer as PUSB_NODE_CONNECTION_NAME;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Length = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    if (Length as usize) < core::mem::size_of::<USB_NODE_CONNECTION_NAME>() {
        Status = STATUS_BUFFER_TOO_SMALL;
        Information = (*Irp).IoStatus.Information as ULONG_PTR;
    } else if (*ConnectionName).ConnectionIndex == 0
        || (*ConnectionName).ConnectionIndex
            > (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG
    {
        Status = STATUS_INVALID_PARAMETER;
        Information = (*Irp).IoStatus.Information as ULONG_PTR;
    } else {
        PortDevice = (*(*HubExtension)
            .PortData
            .offset(((*ConnectionName).ConnectionIndex.wrapping_sub(1)) as isize))
        .DeviceObject;

        if PortDevice.is_null() {
            *(core::ptr::addr_of_mut!((*ConnectionName).NodeName) as *mut WCHAR).offset(0) =
                0 as WCHAR;
            (*ConnectionName).ActualLength =
                core::mem::size_of::<USB_NODE_CONNECTION_NAME>() as ULONG;

            Information = core::mem::size_of::<USB_NODE_CONNECTION_NAME>() as ULONG_PTR;
        } else {
            PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

            if (*PortExtension).PortPdoFlags as ULONG & USBHUB_PDO_FLAG_HUB_DEVICE as ULONG == 0
                || (*PortExtension).PortPdoFlags as ULONG
                    & USBHUB_PDO_FLAG_DEVICE_STARTED as ULONG
                    == 0
                || (*PortExtension).PortPdoFlags as ULONG
                    & USBHUB_PDO_FLAG_REG_DEV_INTERFACE as ULONG
                    == 0
            {
                *(core::ptr::addr_of_mut!((*ConnectionName).NodeName) as *mut WCHAR).offset(0) =
                    0 as WCHAR;
                (*ConnectionName).ActualLength =
                    core::mem::size_of::<USB_NODE_CONNECTION_NAME>() as ULONG;

                Information = core::mem::size_of::<USB_NODE_CONNECTION_NAME>() as ULONG_PTR;
            } else {
                Buffer = (*PortExtension).SymbolicLinkName.Buffer as PWCHAR;
                BufferLength = (*PortExtension).SymbolicLinkName.Length as ULONG;

                if !(*Buffer.offset(
                    (BufferLength as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
                ) as ::core::ffi::c_int
                    == 0 as WCHAR as ::core::ffi::c_int)
                {
                    RtlAssert(
                        b"Buffer[BufferLength / sizeof(WCHAR)] == UNICODE_NULL\0".as_ptr()
                            as *const i8 as PVOID,
                        b"ioctl.c\0".as_ptr() as *const i8 as PVOID,
                        631 as ULONG,
                        core::ptr::null_mut::<CHAR>(),
                    );
                } else {
                }

                LengthSkip = 0 as size_t;

                if *Buffer as ::core::ffi::c_int == '\\' as i32 {
                    BufferEnd =
                        wcschr(Buffer.offset(1) as *const wchar_t, '\\' as i32) as PWCHAR;

                    if !BufferEnd.is_null() {
                        LengthSkip = (BufferEnd.offset(1).offset_from(Buffer) as usize)
                            .wrapping_mul(core::mem::size_of::<WCHAR>())
                            as size_t;
                    } else {
                        LengthSkip = (*PortExtension).SymbolicLinkName.Length as size_t;
                    }
                }

                LengthName = (BufferLength as size_t).wrapping_sub(LengthSkip);

                (*ConnectionName).ActualLength = 0 as ULONG;

                core::ptr::write_bytes(
                    core::ptr::addr_of_mut!((*ConnectionName).NodeName) as *mut WCHAR
                        as *mut u8,
                    0,
                    (Length as usize).wrapping_sub(
                        core::mem::offset_of!(USB_NODE_CONNECTION_NAME, NodeName),
                    ),
                );

                LengthReturned = (core::mem::size_of::<USB_NODE_CONNECTION_NAME>())
                    .wrapping_add(LengthName as usize)
                    as ULONG_PTR;

                if (Length as ULONG_PTR) < LengthReturned {
                    *(core::ptr::addr_of_mut!((*ConnectionName).NodeName) as *mut WCHAR)
                        .offset(0) = 0 as WCHAR;
                    (*ConnectionName).ActualLength = LengthReturned as ULONG;

                    Information = core::mem::size_of::<USB_NODE_CONNECTION_NAME>() as ULONG_PTR;
                } else {
                    core::ptr::copy_nonoverlapping(
                        Buffer.offset(
                            (LengthSkip as usize).wrapping_div(core::mem::size_of::<WCHAR>())
                                as isize,
                        ) as *const u8,
                        (core::ptr::addr_of_mut!((*ConnectionName).NodeName) as *mut WCHAR)
                            .offset(0) as *mut u8,
                        LengthName,
                    );

                    (*ConnectionName).ActualLength = LengthReturned as ULONG;

                    Status = STATUS_SUCCESS;
                    Information = LengthReturned;
                }
            }
        }
    }

    (*Irp).IoStatus.Information = Information as _;
    USBH_CompleteIrp(Irp, Status);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IoctlGetNodeInformation(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut NodeInfo: PUSB_NODE_INFORMATION = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION;
    let mut BufferLength: ULONG;
    let mut Status: NTSTATUS;
    let HubIsBusPowered: BOOLEAN;

    Status = STATUS_SUCCESS;

    NodeInfo = (*Irp).AssociatedIrp.SystemBuffer as PUSB_NODE_INFORMATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    BufferLength = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    RtlZeroMemory(
        (*Irp).AssociatedIrp.SystemBuffer as *mut core::ffi::c_void,
        BufferLength as usize,
    );

    if (BufferLength as usize) < core::mem::size_of::<USB_NODE_INFORMATION>() {
        Status = STATUS_BUFFER_TOO_SMALL;
        USBH_CompleteIrp(Irp, Status);
        return Status;
    }

    (*NodeInfo).NodeType = UsbHub;

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*(*NodeInfo).u.HubInformation.as_mut()).HubDescriptor) as *mut core::ffi::c_void,
        (*HubExtension).HubDescriptor as *const core::ffi::c_void,
        core::mem::size_of::<USB_HUB_DESCRIPTOR>(),
    );

    HubIsBusPowered = USBH_HubIsBusPowered(
        (*HubExtension).Common.SelfDevice,
        (*HubExtension).HubConfigDescriptor,
    );

    (*NodeInfo).u.HubInformation.as_mut().HubIsBusPowered = HubIsBusPowered;

    (*Irp).IoStatus.Information = core::mem::size_of::<USB_NODE_INFORMATION>() as _;

    USBH_CompleteIrp(Irp, Status);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IoctlGetHubCapabilities(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    const USBHUB_FDO_FLAG_USB20_HUB: u32 = 0x00008000; // FIXED 2026-07-06: translation invented 0x40; real value is a (1<<N) define in usbhub.h // USBHUB_FDO_FLAG_USB20_HUB

    let mut Capabilities: PUSB_HUB_CAPABILITIES = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION;
    let mut BufferLength: ULONG;
    let mut Length: ULONG;
    let mut HubCaps: USB_HUB_CAPABILITIES = core::mem::zeroed();

    Capabilities = (*Irp).AssociatedIrp.SystemBuffer as PUSB_HUB_CAPABILITIES;

    HubCaps.set_HubIs2xCapable(
        (((*HubExtension).HubFlags as core::ffi::c_ulong
            & USBHUB_FDO_FLAG_USB20_HUB as core::ffi::c_ulong
            == USBHUB_FDO_FLAG_USB20_HUB as core::ffi::c_ulong) as core::ffi::c_int)
            as ULONG,
    );

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    BufferLength = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    if BufferLength == 0 {
        (*Irp).IoStatus.Information = BufferLength as _;
        USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if BufferLength as usize <= core::mem::size_of::<USB_HUB_CAPABILITIES>() {
        Length = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;
    } else {
        Length = core::mem::size_of::<USB_HUB_CAPABILITIES>() as ULONG;
    }

    RtlZeroMemory(Capabilities as *mut core::ffi::c_void, BufferLength as usize);
    RtlCopyMemory(
        Capabilities as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(HubCaps) as *const core::ffi::c_void,
        Length as usize,
    );

    (*Irp).IoStatus.Information = Length as _;

    USBH_CompleteIrp(Irp, STATUS_SUCCESS);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IoctlGetNodeConnectionAttributes(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut Attributes: PUSB_NODE_CONNECTION_ATTRIBUTES;
    let ConnectionIndex: ULONG;
    let NumPorts: ULONG;
    let mut Status: NTSTATUS;
    let PortData: PUSBHUB_PORT_DATA;
    let IoStack: PIO_STACK_LOCATION;
    let BufferLength: ULONG;

    // DPRINT("USBH_IoctlGetNodeConnectionAttributes ... \n");

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    BufferLength = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    loop {
        if (BufferLength as usize) < core::mem::size_of::<USB_NODE_CONNECTION_ATTRIBUTES>() {
            Status = STATUS_BUFFER_TOO_SMALL;
            break;
        }

        Attributes = (*Irp).AssociatedIrp.SystemBuffer as PUSB_NODE_CONNECTION_ATTRIBUTES;

        ConnectionIndex = (*Attributes).ConnectionIndex;
        RtlZeroMemory(Attributes as *mut core::ffi::c_void, BufferLength as usize);
        (*Attributes).ConnectionIndex = ConnectionIndex;

        Status = STATUS_INVALID_PARAMETER;

        NumPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG;

        if NumPorts == 0 || ConnectionIndex == 0 || ConnectionIndex > NumPorts {
            break;
        }

        PortData = (*HubExtension).PortData.offset((ConnectionIndex - 1) as isize);

        (*Attributes).ConnectionStatus = (*PortData).ConnectionStatus;
        (*Attributes).PortAttributes = (*PortData).PortAttributes;

        (*Irp).IoStatus.Information =
            core::mem::size_of::<USB_NODE_CONNECTION_ATTRIBUTES>() as _;
        Status = STATUS_SUCCESS;
        break;
    }

    USBH_CompleteIrp(Irp, Status);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IoctlGetNodeConnectionInformation(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
    IsExt: BOOLEAN,
) -> NTSTATUS {

    let mut PortData: PUSBHUB_PORT_DATA;
    let mut BufferLength: ULONG;
    let mut Info: PUSB_NODE_CONNECTION_INFORMATION_EX;
    let mut ConnectionIndex: ULONG;
    let mut NumPorts: ULONG;
    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    BufferLength = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    if BufferLength < core::mem::offset_of!(USB_NODE_CONNECTION_INFORMATION_EX, PipeList) as ULONG {
        Status = STATUS_BUFFER_TOO_SMALL;
    } else {
        Info = (*Irp).AssociatedIrp.SystemBuffer as PUSB_NODE_CONNECTION_INFORMATION_EX;

        ConnectionIndex = (*Info).ConnectionIndex;
        RtlZeroMemory(Info as PVOID, BufferLength as usize);
        (*Info).ConnectionIndex = ConnectionIndex;

        Status = STATUS_INVALID_PARAMETER;

        NumPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG;

        if !(NumPorts == 0 || ConnectionIndex == 0 || ConnectionIndex > NumPorts) {
            PortData = (*HubExtension).PortData.offset((ConnectionIndex - 1) as isize);
            DeviceObject = (*PortData).DeviceObject;

            if DeviceObject.is_null() {
                (*Info).ConnectionStatus = (*PortData).ConnectionStatus;

                (*Irp).IoStatus.Information =
                    core::mem::offset_of!(USB_NODE_CONNECTION_INFORMATION_EX, PipeList) as _;
                Status = STATUS_SUCCESS;
            } else {
                PortExtension = (*DeviceObject).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

                (*Info).ConnectionStatus = (*PortData).ConnectionStatus;

                (*Info).DeviceIsHub = (((*PortExtension).PortPdoFlags as u32
                    & USBHUB_PDO_FLAG_HUB_DEVICE as u32)
                    == USBHUB_PDO_FLAG_HUB_DEVICE as u32) as BOOLEAN;

                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*Info).DeviceDescriptor) as PVOID,
                    core::ptr::addr_of!((*PortExtension).DeviceDescriptor) as PVOID,
                    core::mem::size_of::<USB_DEVICE_DESCRIPTOR>(),
                );

                if !(*PortExtension).DeviceHandle.is_null() {
                    Status = USBD_GetDeviceInformationEx(
                        PortExtension,
                        HubExtension,
                        Info,
                        BufferLength,
                        (*PortExtension).DeviceHandle,
                    );
                } else {
                    Status = STATUS_SUCCESS;
                }

                if NT_SUCCESS(Status) {
                    if IsExt == 0 {
                        /* IOCTL_USB_GET_NODE_CONNECTION_INFORMATION request reports
                           only low and full speed connections. Info->Speed member
                           is Info->LowSpeed in the non-EX version of the structure */

                        (*Info).Speed = ((*Info).Speed as i32 == UsbLowSpeed as i32) as UCHAR;
                    }

                    (*Irp).IoStatus.Information = (core::mem::size_of::<USB_NODE_CONNECTION_INFORMATION_EX>()
                        + ((*Info).NumberOfOpenPipes as usize - 1)
                            * core::mem::size_of::<USB_PIPE_INFO>())
                        as _;
                } else if !(Status != STATUS_BUFFER_TOO_SMALL) {
                    (*Irp).IoStatus.Information =
                        core::mem::offset_of!(USB_NODE_CONNECTION_INFORMATION_EX, PipeList)
                            as _;
                    Status = STATUS_SUCCESS;
                }
            }
        }
    }

    USBH_CompleteIrp(Irp, Status);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IoctlGetNodeConnectionDriverKeyName(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut PortData: PUSBHUB_PORT_DATA = core::ptr::null_mut();
    let mut PortDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Length: ULONG;
    let mut ResultLength: ULONG;
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut BufferLength: ULONG;
    let mut KeyName: PUSB_NODE_CONNECTION_DRIVERKEY_NAME;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    BufferLength = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    if (BufferLength as usize) < core::mem::size_of::<USB_NODE_CONNECTION_DRIVERKEY_NAME>()
        || (*(*HubExtension).HubDescriptor).bNumberOfPorts == 0
    {
        Status = STATUS_BUFFER_TOO_SMALL;
    } else {
        KeyName = (*Irp).AssociatedIrp.SystemBuffer as PUSB_NODE_CONNECTION_DRIVERKEY_NAME;
        Status = STATUS_INVALID_PARAMETER;

        PortData = (*HubExtension)
            .PortData
            .offset(((*KeyName).ConnectionIndex as u32).wrapping_sub(1) as isize)
            as PUSBHUB_PORT_DATA;
        PortDevice = (*PortData).DeviceObject;

        if !PortDevice.is_null() {
            PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

            if (*PortExtension).PortPdoFlags as u32 & USBHUB_PDO_FLAG_ENUMERATED as u32 == 0 {
                Status = STATUS_INVALID_DEVICE_STATE;
            } else {
                ResultLength = (BufferLength as usize)
                    .wrapping_sub(core::mem::size_of::<USB_NODE_CONNECTION_DRIVERKEY_NAME>())
                    as ULONG;

                Status = IoGetDeviceProperty(
                    PortDevice,
                    DevicePropertyDriverKeyName,
                    (BufferLength as usize)
                        .wrapping_sub(core::mem::size_of::<USB_NODE_CONNECTION_DRIVERKEY_NAME>())
                        as ULONG,
                    core::ptr::addr_of_mut!((*KeyName).DriverKeyName) as PVOID,
                    core::ptr::addr_of_mut!(ResultLength),
                );

                if Status == STATUS_BUFFER_TOO_SMALL {
                    Status = STATUS_SUCCESS;
                }

                Length = (ResultLength as usize)
                    .wrapping_add(core::mem::size_of::<USB_NODE_CONNECTION_DRIVERKEY_NAME>())
                    as ULONG;
                (*KeyName).ActualLength = Length;

                if BufferLength < Length {
                    *(core::ptr::addr_of_mut!((*KeyName).DriverKeyName) as *mut WCHAR)
                        .offset(0) = 0 as WCHAR;
                    (*Irp).IoStatus.Information =
                        core::mem::size_of::<USB_NODE_CONNECTION_DRIVERKEY_NAME>() as _;
                } else {
                    (*Irp).IoStatus.Information = Length as _;
                }
            }
        }
    }

    USBH_CompleteIrp(Irp, Status);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IoctlGetDescriptor(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut BufferLength: ULONG;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut UsbRequest: PUSB_DESCRIPTOR_REQUEST;
    let mut PortDevice: PDEVICE_OBJECT;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut Urb: *mut _URB_CONTROL_TRANSFER;
    let mut Status: NTSTATUS;
    let mut RequestBufferLength: ULONG;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut NumPorts: ULONG;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    BufferLength = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;

    if (BufferLength as usize) < core::mem::size_of::<USB_DESCRIPTOR_REQUEST>() {
        Status = STATUS_BUFFER_TOO_SMALL;
    } else {
        UsbRequest = (*Irp).AssociatedIrp.SystemBuffer as PUSB_DESCRIPTOR_REQUEST;
        RequestBufferLength = (*UsbRequest).SetupPacket.wLength as ULONG;

        if RequestBufferLength
            > BufferLength.wrapping_sub(core::mem::offset_of!(USB_DESCRIPTOR_REQUEST, Data) as ULONG)
        {
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            Status = STATUS_INVALID_PARAMETER;

            NumPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG;

            if !(NumPorts == 0
                || (*UsbRequest).ConnectionIndex == 0
                || (*UsbRequest).ConnectionIndex > NumPorts)
            {
                PortData = (*HubExtension)
                    .PortData
                    .offset(((*UsbRequest).ConnectionIndex.wrapping_sub(1)) as isize);
                PortDevice = (*PortData).DeviceObject;

                if !PortDevice.is_null() {
                    PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;

                    if (*UsbRequest).SetupPacket.bmRequest as u32
                        == USB_CONFIGURATION_DESCRIPTOR_TYPE
                        && RequestBufferLength as usize
                            == core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>()
                    {
                        Status = STATUS_SUCCESS;

                        RtlCopyMemory(
                            core::ptr::addr_of_mut!((*UsbRequest).Data) as *mut UCHAR
                                as *mut core::ffi::c_void,
                            core::ptr::addr_of_mut!((*PortExtension).ConfigDescriptor)
                                as *const core::ffi::c_void,
                            core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>(),
                        );

                        (*Irp).IoStatus.Information = (core::mem::size_of::<USB_DESCRIPTOR_REQUEST>()
                            - core::mem::size_of::<UCHAR>()
                            + core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>())
                            as _;
                    } else {
                        Urb = ExAllocatePoolWithTag(
                            NonPagedPool,
                            core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as SIZE_T,
                            USB_HUB_TAG as ULONG,
                        ) as *mut _URB_CONTROL_TRANSFER;

                        if Urb.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                        } else {
                            RtlZeroMemory(
                                Urb as *mut core::ffi::c_void,
                                core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>(),
                            );

                            (*Urb).Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE as USHORT;
                            (*Urb).Hdr.Length =
                                core::mem::size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as USHORT;

                            (*Urb).TransferBuffer =
                                core::ptr::addr_of_mut!((*UsbRequest).Data) as *mut UCHAR as PVOID;
                            (*Urb).TransferBufferLength = RequestBufferLength;
                            (*Urb).TransferBufferMDL = core::ptr::null_mut();
                            (*Urb).UrbLink = core::ptr::null_mut();

                            RtlCopyMemory(
                                core::ptr::addr_of_mut!((*Urb).SetupPacket) as *mut UCHAR
                                    as *mut core::ffi::c_void,
                                core::ptr::addr_of_mut!((*UsbRequest).SetupPacket)
                                    as *const core::ffi::c_void,
                                core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
                            );

                            Status = USBH_SyncSubmitUrb(
                                (*PortExtension).Common.SelfDevice,
                                Urb as PURB,
                            );

                            (*Irp).IoStatus.Information = ((core::mem::size_of::<USB_DESCRIPTOR_REQUEST>()
                                - core::mem::size_of::<UCHAR>())
                                + (*Urb).TransferBufferLength as usize)
                                as _;

                            ExFreePoolWithTag(Urb as PVOID, USB_HUB_TAG as ULONG);
                        }
                    }
                }
            }
        }
    }

    USBH_CompleteIrp(Irp, Status);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_DeviceControl(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as NTSTATUS;
    const USBHUB_FDO_FLAG_DEVICE_STARTED: u32 = 0x00000001; // FIXED 2026-07-06: translation invented 0x4; real value is a (1<<N) define in usbhub.h // USBHUB_FDO_FLAG_DEVICE_STARTED
    const USBHUB_FDO_FLAG_DEVICE_STOPPED: u32 = 0x00000020; // FIXED 2026-07-06: translation invented 0x8; real value is a (1<<N) define in usbhub.h // USBHUB_FDO_FLAG_DEVICE_STOPPED

    const IOCTL_USB_GET_HUB_CAPABILITIES: u32 = 2229308;
    const IOCTL_USB_HUB_CYCLE_PORT: u32 = 2229316;
    const IOCTL_USB_GET_NODE_INFORMATION: u32 = 2229256;
    const IOCTL_USB_GET_NODE_CONNECTION_INFORMATION: u32 = 2229260;
    const IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX: u32 = 2229320;
    const IOCTL_USB_GET_NODE_CONNECTION_ATTRIBUTES: u32 = 2229312;
    const IOCTL_USB_GET_NODE_CONNECTION_NAME: u32 = 2229268;
    const IOCTL_USB_GET_NODE_CONNECTION_DRIVERKEY_NAME: u32 = 2229280;
    const IOCTL_USB_GET_DESCRIPTOR_FROM_NODE_CONNECTION: u32 = 2229264;
    const IOCTL_KS_PROPERTY: u32 = 3080195;

    let mut Status: NTSTATUS = STATUS_DEVICE_BUSY;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut ControlCode: ULONG;
    let mut IsCheckHubIdle: BOOLEAN = 0 as BOOLEAN;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    ControlCode = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    if (*HubExtension).CurrentPowerState.DeviceState as u32 != PowerDeviceD0 as u32
        && (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED != 0
    {
        IsCheckHubIdle = 1 as BOOLEAN;
        USBH_HubSetD0(HubExtension);
    }

    match ControlCode {
        IOCTL_USB_GET_HUB_CAPABILITIES => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetHubCapabilities(HubExtension, Irp);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_USB_HUB_CYCLE_PORT => {
            DbgBreakPoint();
        }
        IOCTL_USB_GET_NODE_INFORMATION => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetNodeInformation(HubExtension, Irp);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_USB_GET_NODE_CONNECTION_INFORMATION => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetNodeConnectionInformation(HubExtension, Irp, 0 as BOOLEAN);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetNodeConnectionInformation(HubExtension, Irp, 1 as BOOLEAN);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_USB_GET_NODE_CONNECTION_ATTRIBUTES => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetNodeConnectionAttributes(HubExtension, Irp);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_USB_GET_NODE_CONNECTION_NAME => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetNodeName(HubExtension, Irp);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_USB_GET_NODE_CONNECTION_DRIVERKEY_NAME => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetNodeConnectionDriverKeyName(HubExtension, Irp);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_USB_GET_DESCRIPTOR_FROM_NODE_CONNECTION => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED == 0 {
                Status = USBH_IoctlGetDescriptor(HubExtension, Irp);
            } else {
                USBH_CompleteIrp(Irp, Status);
            }
        }
        IOCTL_KS_PROPERTY => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
            USBH_CompleteIrp(Irp, Status);
        }
        _ => {
            Status = USBH_PassIrp((*HubExtension).RootHubPdo, Irp);
        }
    }

    if IsCheckHubIdle != 0 {
        USBH_CheckHubIdle(HubExtension);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoInternalControl(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let mut Status: NTSTATUS = STATUS_NOT_SUPPORTED;
    let ControlCode: ULONG;
    let IoStack: PIO_STACK_LOCATION;
    let HubCount: PULONG;

    HubExtension = (*PortExtension).HubExtension;

    if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_NOT_CONNECTED != 0 {
        Status = STATUS_DEVICE_NOT_CONNECTED;
        USBH_CompleteIrp(Irp, Status);
        return Status;
    }

    if (*PortExtension).CurrentPowerState.DeviceState != PowerDeviceD0 {
        Status = STATUS_DEVICE_POWERED_OFF;
        USBH_CompleteIrp(Irp, Status);
        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    ControlCode = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    if ControlCode == IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO {
        HubExtension = (*PortExtension).RootHubExtension;
    }

    if HubExtension.is_null() {
        Status = STATUS_DEVICE_BUSY;
        USBH_CompleteIrp(Irp, Status);
        return Status;
    }

    match ControlCode {
        IOCTL_INTERNAL_USB_SUBMIT_URB => {
            return USBH_PdoIoctlSubmitUrb(PortExtension, Irp);
        }

        IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION => {
            return USBH_PortIdleNotificationRequest(PortExtension, Irp);
        }

        IOCTL_INTERNAL_USB_GET_PORT_STATUS => {
            return USBH_PdoIoctlGetPortStatus(PortExtension, Irp);
        }

        IOCTL_INTERNAL_USB_RESET_PORT => {
            return USBH_PdoIoctlResetPort(PortExtension, Irp);
        }

        IOCTL_INTERNAL_USB_ENABLE_PORT => {
            DbgBreakPoint();
        }

        IOCTL_INTERNAL_USB_CYCLE_PORT => {
            DbgBreakPoint();
        }

        IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE => {
            *((*IoStack).Parameters.Others.Argument1 as *mut PVOID) =
                (*PortExtension).DeviceHandle as PVOID;
            Status = STATUS_SUCCESS;
        }

        IOCTL_INTERNAL_USB_GET_HUB_COUNT => {
            if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE == 0 {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                HubCount = (*IoStack).Parameters.Others.Argument1 as PULONG;
                *HubCount = (*HubCount).wrapping_add(1);
                Status = USBH_SyncGetHubCount((*HubExtension).LowerDevice, HubCount);
            }
        }

        IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO => {
            if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE == 0 {
                DbgBreakPoint();
                Status = STATUS_SUCCESS;
                *((*IoStack).Parameters.Others.Argument1 as *mut PVOID) =
                    core::ptr::null_mut();
                USBH_CompleteIrp(Irp, Status);
            } else {
                return USBH_PassIrp((*HubExtension).RootHubPdo, Irp);
            }
        }

        IOCTL_INTERNAL_USB_GET_HUB_NAME => {
            DbgBreakPoint();
        }

        IOCTL_GET_HCD_DRIVERKEY_NAME => {
            DbgBreakPoint();
        }

        IOCTL_INTERNAL_USB_GET_BUS_INFO => {
            DbgBreakPoint();
        }

        IOCTL_INTERNAL_USB_GET_PARENT_HUB_INFO => {
            DbgBreakPoint();
        }

        _ => {}
    }

    USBH_CompleteIrp(Irp, Status);
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Hub Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBHub I/O control functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbhub.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBHUB_IOCTL
// #include "dbg_uhub.h"
//
// NTSTATUS
// NTAPI
// USBH_SelectConfigOrInterfaceComplete(IN PDEVICE_OBJECT DeviceObject,
//                                      IN PIRP Irp,
//                                      IN PVOID Context)
// {
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PVOID TimeoutContext; // PUSBHUB_BANDWIDTH_TIMEOUT_CONTEXT
//     PUSBHUB_PORT_DATA PortData = NULL;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//     PURB Urb;
//
//     DPRINT("USBH_SelectConfigOrInterfaceComplete ... \n");
//
//     if (Irp->PendingReturned)
//     {
//         IoMarkIrpPending(Irp);
//     }
//
//     PortExtension = Context;
//     HubExtension = PortExtension->HubExtension;
//
//     ASSERT(PortExtension->PortNumber > 0);
//
//     if (HubExtension)
//     {
//         PortData = &HubExtension->PortData[PortExtension->PortNumber - 1];
//     }
//
//     Status = Irp->IoStatus.Status;
//
//     if (NT_SUCCESS(Irp->IoStatus.Status))
//     {
//         KeAcquireSpinLock(&PortExtension->PortTimeoutSpinLock, &OldIrql);
//
//         TimeoutContext = PortExtension->BndwTimeoutContext;
//
//         if (TimeoutContext)
//         {
//             DPRINT1("USBH_SelectConfigOrInterfaceComplete: TimeoutContext != NULL. FIXME\n");
//             DbgBreakPoint();
//         }
//
//         KeReleaseSpinLock(&PortExtension->PortTimeoutSpinLock, OldIrql);
//
//         PortExtension->PortPdoFlags &= ~(USBHUB_PDO_FLAG_PORT_RESTORE_FAIL |
//                                          USBHUB_PDO_FLAG_ALLOC_BNDW_FAILED);
//
//         if (PortData && PortData->ConnectionStatus != DeviceHubNestedTooDeeply)
//         {
//             PortData->ConnectionStatus = DeviceConnected;
//         }
//     }
//     else
//     {
//         Urb = URB_FROM_IRP(Irp);
//         DPRINT1("USBH_SelectConfigOrInterfaceComplete: Status = 0x%lx, UsbdStatus = 0x%lx\n", Status, Urb->UrbHeader.Status);
//         if (Urb->UrbHeader.Status == USBD_STATUS_NO_BANDWIDTH)
//         {
//             DPRINT1("USBH_SelectConfigOrInterfaceComplete: USBD_STATUS_NO_BANDWIDTH. FIXME\n");
//             /*DbgBreakPoint();*/ /* disabled due to CORE-16384, seems to be continuable */
//         }
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoUrbFilter(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                   IN PIRP Irp)
// {
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigDescriptor;
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PDEVICE_OBJECT DeviceObject;
//     PURB Urb;
//     USHORT Function;
//     ULONG MaxPower;
//     USBD_STATUS UrbStatus;
//     BOOLEAN IsValidConfig;
//
//     HubExtension = PortExtension->HubExtension;
//     DeviceObject = PortExtension->Common.SelfDevice;
//
//     Urb = URB_FROM_IRP(Irp);
//
//     DPRINT_IOCTL!("USBH_PdoUrbFilter: Device - %p, Irp - %p, Urb - %p\n",
//                  DeviceObject,
//                  Irp,
//                  Urb);
//
//     if (PortExtension->PortPdoFlags & (USBHUB_PDO_FLAG_PORT_RESTORE_FAIL |
//                                        USBHUB_PDO_FLAG_PORT_RESSETING))
//     {
//         Urb->UrbHeader.Status = USBD_STATUS_INVALID_PARAMETER;
//         USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Function = Urb->UrbHeader.Function;
//
//     switch (Function)
//     {
//         case URB_FUNCTION_SELECT_CONFIGURATION:
//         {
//             ConfigDescriptor = Urb->UrbSelectConfiguration.ConfigurationDescriptor;
//
//             if (ConfigDescriptor)
//             {
//                 IsValidConfig = TRUE;
//
//                 if (ConfigDescriptor->bDescriptorType != USB_CONFIGURATION_DESCRIPTOR_TYPE)
//                 {
//                     DPRINT1("USBH_PdoUrbFilter: Not valid Cfg. bDescriptorType\n");
//                     IsValidConfig = FALSE;
//                     UrbStatus = USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR;
//                 }
//
//                 if (ConfigDescriptor->bLength < sizeof(USB_CONFIGURATION_DESCRIPTOR))
//                 {
//                     DPRINT1("USBH_PdoUrbFilter: Size Cfg. descriptor is too small\n");
//                     IsValidConfig = FALSE;
//                     UrbStatus = USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR;
//                 }
//
//                 if (!IsValidConfig)
//                 {
//                     Urb->UrbHeader.Status = UrbStatus;
//                     USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
//                     return STATUS_INVALID_PARAMETER;
//                 }
//
//                 MaxPower = 2 * ConfigDescriptor->MaxPower;
//                 PortExtension->MaxPower = MaxPower;
//
//                 if (HubExtension->MaxPowerPerPort < MaxPower)
//                 {
//                     PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_INSUFFICIENT_PWR;
//
//                     DPRINT1("USBH_PdoUrbFilter: USBH_InvalidatePortDeviceState() UNIMPLEMENTED. FIXME\n");
//                     DbgBreakPoint();
//
//                     USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
//                     return STATUS_INVALID_PARAMETER;
//                 }
//             }
//         }
//
//         /* fall through */
//
//         case URB_FUNCTION_SELECT_INTERFACE:
//         {
//             IoCopyCurrentIrpStackLocationToNext(Irp);
//
//             IoSetCompletionRoutine(Irp,
//                                    USBH_SelectConfigOrInterfaceComplete,
//                                    PortExtension,
//                                    TRUE,
//                                    TRUE,
//                                    TRUE);
//
//             return IoCallDriver(HubExtension->RootHubPdo2, Irp);
//         }
//
//         case URB_FUNCTION_CONTROL_TRANSFER:
//         case URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER:
//         case URB_FUNCTION_ISOCH_TRANSFER:
//         {
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_DELETE_PENDING)
//             {
//                 Urb->UrbHeader.Status = USBD_STATUS_INVALID_PARAMETER;
//                 USBH_CompleteIrp(Irp, STATUS_DELETE_PENDING);
//                 return STATUS_DELETE_PENDING;
//             }
//
//             break;
//         }
//
//         case URB_FUNCTION_GET_MS_FEATURE_DESCRIPTOR:
//             DPRINT1("USBH_PdoUrbFilter: URB_FUNCTION_GET_MS_FEATURE_DESCRIPTOR UNIMPLEMENTED. FIXME\n");
//             USBH_CompleteIrp(Irp, STATUS_NOT_IMPLEMENTED);
//             return STATUS_NOT_IMPLEMENTED;
//
//         default:
//             break;
//     }
//
//     return USBH_PassIrp(HubExtension->RootHubPdo2, Irp);
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoIoctlSubmitUrb(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                        IN PIRP Irp)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PURB Urb;
//     NTSTATUS Status;
//
//     DPRINT_IOCTL!("USBH_PdoIoctlSubmitUrb ... \n");
//
//     HubExtension = PortExtension->HubExtension;
//
//     Urb = URB_FROM_IRP(Irp);
//
//     if (PortExtension->DeviceHandle == NULL)
//     {
//         Urb->UrbHeader.UsbdDeviceHandle = NULL;
//         Status = USBH_PassIrp(HubExtension->RootHubPdo2, Irp);
//     }
//     else
//     {
//         Urb->UrbHeader.UsbdDeviceHandle = PortExtension->DeviceHandle;
//         Status = USBH_PdoUrbFilter(PortExtension, Irp);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoIoctlGetPortStatus(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                            IN PIRP Irp)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PUSBHUB_PORT_DATA PortData;
//     PIO_STACK_LOCATION IoStack;
//     PULONG PortStatus;
//     NTSTATUS Status;
//
//     DPRINT("USBH_PdoIoctlGetPortStatus ... \n");
//
//     HubExtension = PortExtension->HubExtension;
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->HubSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     ASSERT(PortExtension->PortNumber > 0);
//     PortData = &HubExtension->PortData[PortExtension->PortNumber - 1];
//
//     Status = USBH_SyncGetPortStatus(HubExtension,
//                                     PortExtension->PortNumber,
//                                     &PortData->PortStatus,
//                                     sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     PortStatus = IoStack->Parameters.Others.Argument1;
//
//     *PortStatus = 0;
//
//     if (PortExtension->Common.SelfDevice == PortData->DeviceObject)
//     {
//         if (PortData->PortStatus.PortStatus.Usb20PortStatus.PortEnabledDisabled)
//         {
//             *PortStatus |= USBD_PORT_ENABLED;
//         }
//
//         if (PortData->PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus)
//         {
//             *PortStatus |= USBD_PORT_CONNECTED;
//         }
//     }
//
//     KeReleaseSemaphore(&HubExtension->HubSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
//     USBH_CompleteIrp(Irp, Status);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_ResetPortWorker(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                      IN PVOID Context)
// {
//     PUSBHUB_RESET_PORT_CONTEXT WorkItemReset;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PUSB_DEVICE_HANDLE DeviceHandle;
//     NTSTATUS Status;
//     USHORT Port;
//
//     DPRINT("USBH_ResetPortWorker ... \n");
//
//     WorkItemReset = Context;
//
//     PortExtension = WorkItemReset->PortExtension;
//
//     if (!HubExtension)
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         goto Exit;
//     }
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->HubSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     Port = PortExtension->PortNumber;
//     DeviceHandle = PortExtension->DeviceHandle;
//
//     ASSERT(Port > 0);
//
//     if (PortExtension->Common.SelfDevice == HubExtension->PortData[Port-1].DeviceObject &&
//         DeviceHandle != NULL)
//     {
//         USBD_RemoveDeviceEx(HubExtension,
//                             DeviceHandle,
//                             USBD_MARK_DEVICE_BUSY);
//
//         Status = USBH_ResetDevice(HubExtension,
//                                   Port,
//                                   TRUE,
//                                   FALSE);
//     }
//     else
//     {
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     KeReleaseSemaphore(&HubExtension->HubSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent,
//                    EVENT_INCREMENT,
//                    FALSE);
//     }
//
// Exit:
//
//     PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_PORT_RESSETING;
//
//     USBH_CompleteIrp(WorkItemReset->Irp, Status);
//
//     WorkItemReset->PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_PORT_RESTORE_FAIL;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoIoctlResetPort(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                        IN PIRP Irp)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PUSBHUB_RESET_PORT_CONTEXT HubWorkItemBuffer;
//     PUSBHUB_IO_WORK_ITEM HubIoWorkItem;
//     NTSTATUS Status;
//
//     HubExtension = PortExtension->HubExtension;
//
//     DPRINT("USBH_PdoIoctlResetPort ... \n");
//
//     if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_PORT_RESSETING)
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         USBH_CompleteIrp(Irp, Status);
//         return Status;
//     }
//
//     Status = USBH_AllocateWorkItem(HubExtension,
//                                    &HubIoWorkItem,
//                                    USBH_ResetPortWorker,
//                                    sizeof(USBHUB_RESET_PORT_CONTEXT),
//                                    (PVOID *)&HubWorkItemBuffer,
//                                    DelayedWorkQueue);
//
//     if (!NT_SUCCESS(Status))
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         USBH_CompleteIrp(Irp, Status);
//         return Status;
//     }
//
//     PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_PORT_RESSETING;
//     IoMarkIrpPending(Irp);
//
//     HubWorkItemBuffer->PortExtension = PortExtension;
//     HubWorkItemBuffer->Irp = Irp;
//
//     Status = STATUS_PENDING;
//
//     USBH_QueueWorkItem(PortExtension->HubExtension, HubIoWorkItem);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_PortIdleNotificationCancelRoutine(IN PDEVICE_OBJECT Device,
//                                        IN PIRP Irp)
// {
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PIRP PendingIdleIrp = NULL;
//     PUSBHUB_IO_WORK_ITEM HubIoWorkItem;
//     PUSBHUB_IDLE_PORT_CANCEL_CONTEXT HubWorkItemBuffer;
//     NTSTATUS Status;
//
//     DPRINT("USBH_PortIdleNotificationCancelRoutine ... \n");
//
//     PortExtension = Device->DeviceExtension;
//     PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_IDLE_NOTIFICATION;
//
//     HubExtension = PortExtension->HubExtension;
//     ASSERT(HubExtension);
//
//     PortExtension->IdleNotificationIrp = NULL;
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST)
//     {
//         PendingIdleIrp = HubExtension->PendingIdleIrp;
//         HubExtension->PendingIdleIrp = NULL;
//     }
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     if (PendingIdleIrp)
//     {
//         USBH_HubCancelIdleIrp(HubExtension, PendingIdleIrp);
//     }
//
//     if (HubExtension->CurrentPowerState.DeviceState == PowerDeviceD0)
//     {
//         goto ErrorExit;
//     }
//
//     Status = USBH_AllocateWorkItem(HubExtension,
//                                    &HubIoWorkItem,
//                                    USBH_IdleCancelPowerHubWorker,
//                                    sizeof(USBHUB_IDLE_PORT_CANCEL_CONTEXT),
//                                    (PVOID *)&HubWorkItemBuffer,
//                                    DelayedWorkQueue);
//
//     if (NT_SUCCESS(Status))
//     {
//         HubWorkItemBuffer->Irp = Irp;
//         USBH_QueueWorkItem(HubExtension, HubIoWorkItem);
//         return;
//     }
//
// ErrorExit:
//
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// NTSTATUS
// NTAPI
// USBH_PortIdleNotificationRequest(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                                  IN PIRP Irp)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PIO_STACK_LOCATION IoStack;
//     PUSB_IDLE_CALLBACK_INFO IdleCallbackInfo;
//     NTSTATUS Status;
//     KIRQL Irql;
//
//     DPRINT("USBH_PortIdleNotificationRequest ... \n");
//
//     HubExtension = PortExtension->HubExtension;
//
//     IoAcquireCancelSpinLock(&Irql);
//
//     if (PortExtension->IdleNotificationIrp)
//     {
//         IoReleaseCancelSpinLock(Irql);
//         Irp->IoStatus.Status = STATUS_DEVICE_BUSY;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_DEVICE_BUSY;
//     }
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     IdleCallbackInfo = IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//     if (!IdleCallbackInfo || !IdleCallbackInfo->IdleCallback)
//     {
//         IoReleaseCancelSpinLock(Irql);
//
//         Status = STATUS_NO_CALLBACK_ACTIVE;
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//         return Status;
//     }
//
//     IoSetCancelRoutine(Irp, USBH_PortIdleNotificationCancelRoutine);
//
//     if (Irp->Cancel)
//     {
//         if (IoSetCancelRoutine(Irp, NULL))
//         {
//             IoReleaseCancelSpinLock(Irql);
//             Status = STATUS_CANCELLED;
//             Irp->IoStatus.Status = STATUS_CANCELLED;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         }
//         else
//         {
//             IoMarkIrpPending(Irp);
//             IoReleaseCancelSpinLock(Irql);
//             Status = STATUS_PENDING;
//         }
//     }
//     else
//     {
//         PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_IDLE_NOTIFICATION;
//
//         PortExtension->IdleNotificationIrp = Irp;
//         IoMarkIrpPending(Irp);
//
//         IoReleaseCancelSpinLock(Irql);
//         Status = STATUS_PENDING;
//
//         DPRINT("USBH_PortIdleNotificationRequest: IdleNotificationIrp - %p\n",
//                PortExtension->IdleNotificationIrp);
//
//         USBH_CheckIdleDeferred(HubExtension);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_IoctlGetNodeName(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                       IN PIRP Irp)
// {
//     PUSB_NODE_CONNECTION_NAME ConnectionName;
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     size_t LengthSkip;
//     PWCHAR Buffer;
//     ULONG BufferLength;
//     PWCHAR BufferEnd;
//     ULONG_PTR LengthReturned;
//     size_t LengthName;
//     ULONG Length;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     ULONG_PTR Information;
//
//     DPRINT("USBH_IoctlGetNodeName ... \n");
//
//     Status = STATUS_SUCCESS;
//
//     ConnectionName = Irp->AssociatedIrp.SystemBuffer;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Length = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     if (Length < sizeof(USB_NODE_CONNECTION_NAME))
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         Information = Irp->IoStatus.Information;
//         goto Exit;
//     }
//
//     if (ConnectionName->ConnectionIndex == 0 ||
//         ConnectionName->ConnectionIndex > HubExtension->HubDescriptor->bNumberOfPorts)
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         Information = Irp->IoStatus.Information;
//         goto Exit;
//     }
//
//     PortDevice = HubExtension->PortData[ConnectionName->ConnectionIndex - 1].DeviceObject;
//
//     if (!PortDevice)
//     {
//         ConnectionName->NodeName[0] = 0;
//         ConnectionName->ActualLength = sizeof(USB_NODE_CONNECTION_NAME);
//
//         Information = sizeof(USB_NODE_CONNECTION_NAME);
//         goto Exit;
//     }
//
//     PortExtension = PortDevice->DeviceExtension;
//
//     if (!(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE) ||
//         !(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_DEVICE_STARTED) ||
//         !(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_REG_DEV_INTERFACE))
//     {
//         ConnectionName->NodeName[0] = 0;
//         ConnectionName->ActualLength = sizeof(USB_NODE_CONNECTION_NAME);
//
//         Information = sizeof(USB_NODE_CONNECTION_NAME);
//         goto Exit;
//     }
//
//     Buffer = PortExtension->SymbolicLinkName.Buffer;
//     BufferLength = PortExtension->SymbolicLinkName.Length;
//
//     ASSERT(Buffer[BufferLength / sizeof(WCHAR)] == UNICODE_NULL);
//
//     LengthSkip = 0;
//
//     if (*Buffer == L'\\')
//     {
//         BufferEnd = wcschr(Buffer + 1, L'\\');
//
//         if (BufferEnd != NULL)
//         {
//             LengthSkip = (BufferEnd + 1 - Buffer) * sizeof(WCHAR);
//         }
//         else
//         {
//             LengthSkip = PortExtension->SymbolicLinkName.Length;
//         }
//     }
//
//     LengthName = BufferLength - LengthSkip;
//
//     ConnectionName->ActualLength = 0;
//
//     RtlZeroMemory(ConnectionName->NodeName,
//                   Length - FIELD_OFFSET(USB_NODE_CONNECTION_NAME, NodeName));
//
//     LengthReturned = sizeof(USB_NODE_CONNECTION_NAME) + LengthName;
//
//     if (Length < LengthReturned)
//     {
//         ConnectionName->NodeName[0] = 0;
//         ConnectionName->ActualLength = LengthReturned;
//
//         Information = sizeof(USB_NODE_CONNECTION_NAME);
//         goto Exit;
//     }
//
//     RtlCopyMemory(&ConnectionName->NodeName[0],
//                   &Buffer[LengthSkip / sizeof(WCHAR)],
//                   LengthName);
//
//     ConnectionName->ActualLength = LengthReturned;
//
//     Status = STATUS_SUCCESS;
//     Information = LengthReturned;
//
// Exit:
//     Irp->IoStatus.Information = Information;
//     USBH_CompleteIrp(Irp, Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_IoctlGetNodeInformation(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                              IN PIRP Irp)
// {
//     PUSB_NODE_INFORMATION NodeInfo;
//     PIO_STACK_LOCATION IoStack;
//     ULONG BufferLength;
//     NTSTATUS Status;
//     BOOLEAN HubIsBusPowered;
//
//     DPRINT("USBH_IoctlGetNodeInformation ... \n");
//
//     Status = STATUS_SUCCESS;
//
//     NodeInfo = Irp->AssociatedIrp.SystemBuffer;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     BufferLength = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     RtlZeroMemory(Irp->AssociatedIrp.SystemBuffer, BufferLength);
//
//     if (BufferLength < sizeof(USB_NODE_INFORMATION))
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         USBH_CompleteIrp(Irp, Status);
//         return Status;
//     }
//
//     NodeInfo->NodeType = UsbHub;
//
//     RtlCopyMemory(&NodeInfo->u.HubInformation.HubDescriptor,
//                   HubExtension->HubDescriptor,
//                   sizeof(USB_HUB_DESCRIPTOR));
//
//     HubIsBusPowered = USBH_HubIsBusPowered(HubExtension->Common.SelfDevice,
//                                            HubExtension->HubConfigDescriptor);
//
//     NodeInfo->u.HubInformation.HubIsBusPowered = HubIsBusPowered;
//
//     Irp->IoStatus.Information = sizeof(USB_NODE_INFORMATION);
//
//     USBH_CompleteIrp(Irp, Status);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_IoctlGetHubCapabilities(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                              IN PIRP Irp)
// {
//     PUSB_HUB_CAPABILITIES Capabilities;
//     PIO_STACK_LOCATION IoStack;
//     ULONG BufferLength;
//     ULONG Length;
//     USB_HUB_CAPABILITIES HubCaps;
//
//     DPRINT("USBH_IoctlGetHubCapabilities ... \n");
//
//     Capabilities = Irp->AssociatedIrp.SystemBuffer;
//
//     HubCaps.HubIs2xCapable = (HubExtension->HubFlags & USBHUB_FDO_FLAG_USB20_HUB) ==
//                               USBHUB_FDO_FLAG_USB20_HUB;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     BufferLength = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     if (BufferLength == 0)
//     {
//         Irp->IoStatus.Information = BufferLength;
//         USBH_CompleteIrp(Irp, STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (BufferLength <= sizeof(HubCaps))
//     {
//         Length = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//     }
//     else
//     {
//         Length = sizeof(HubCaps);
//     }
//
//     RtlZeroMemory(Capabilities, BufferLength);
//     RtlCopyMemory(Capabilities, &HubCaps, Length);
//
//     Irp->IoStatus.Information = Length;
//
//     USBH_CompleteIrp(Irp, STATUS_SUCCESS);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBH_IoctlGetNodeConnectionAttributes(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                       IN PIRP Irp)
// {
//     PUSB_NODE_CONNECTION_ATTRIBUTES Attributes;
//     ULONG ConnectionIndex;
//     ULONG NumPorts;
//     NTSTATUS Status;
//     PUSBHUB_PORT_DATA PortData;
//     PIO_STACK_LOCATION IoStack;
//     ULONG BufferLength;
//
//     DPRINT("USBH_IoctlGetNodeConnectionAttributes ... \n");
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     BufferLength = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     if (BufferLength < sizeof(USB_NODE_CONNECTION_ATTRIBUTES))
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Exit;
//     }
//
//     Attributes = Irp->AssociatedIrp.SystemBuffer;
//
//     ConnectionIndex = Attributes->ConnectionIndex;
//     RtlZeroMemory(Attributes, BufferLength);
//     Attributes->ConnectionIndex = ConnectionIndex;
//
//     Status = STATUS_INVALID_PARAMETER;
//
//     NumPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     if (NumPorts == 0 ||
//         ConnectionIndex == 0 ||
//         ConnectionIndex > NumPorts)
//     {
//         goto Exit;
//     }
//
//     PortData = HubExtension->PortData + (ConnectionIndex - 1);
//
//     Attributes->ConnectionStatus = PortData->ConnectionStatus;
//     Attributes->PortAttributes = PortData->PortAttributes;
//
//     Irp->IoStatus.Information = sizeof(USB_NODE_CONNECTION_ATTRIBUTES);
//     Status = STATUS_SUCCESS;
//
// Exit:
//
//     USBH_CompleteIrp(Irp, Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_IoctlGetNodeConnectionInformation(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                        IN PIRP Irp,
//                                        IN BOOLEAN IsExt)
// {
//     PUSBHUB_PORT_DATA PortData;
//     ULONG BufferLength;
//     PUSB_NODE_CONNECTION_INFORMATION_EX Info;
//     ULONG ConnectionIndex;
//     ULONG NumPorts;
//     NTSTATUS Status;
//     PDEVICE_OBJECT DeviceObject;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT("USBH_IoctlGetNodeConnectionInformation ... \n");
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     BufferLength = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     if (BufferLength < (ULONG)FIELD_OFFSET(USB_NODE_CONNECTION_INFORMATION_EX,
//                                            PipeList))
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Exit;
//     }
//
//     Info = Irp->AssociatedIrp.SystemBuffer;
//
//     ConnectionIndex = Info->ConnectionIndex;
//     RtlZeroMemory(Info, BufferLength);
//     Info->ConnectionIndex = ConnectionIndex;
//
//     Status = STATUS_INVALID_PARAMETER;
//
//     NumPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     if (NumPorts == 0 ||
//         ConnectionIndex == 0 ||
//         ConnectionIndex > NumPorts)
//     {
//         goto Exit;
//     }
//
//     PortData = HubExtension->PortData + (ConnectionIndex - 1);
//     DeviceObject = PortData->DeviceObject;
//
//     if (!DeviceObject)
//     {
//         Info->ConnectionStatus = PortData->ConnectionStatus;
//
//         Irp->IoStatus.Information = FIELD_OFFSET(USB_NODE_CONNECTION_INFORMATION_EX,
//                                                  PipeList);
//         Status = STATUS_SUCCESS;
//         goto Exit;
//     }
//
//     PortExtension = DeviceObject->DeviceExtension;
//
//     Info->ConnectionStatus = PortData->ConnectionStatus;
//
//     Info->DeviceIsHub = (PortExtension->PortPdoFlags &
//                          USBHUB_PDO_FLAG_HUB_DEVICE) ==
//                          USBHUB_PDO_FLAG_HUB_DEVICE;
//
//     RtlCopyMemory(&Info->DeviceDescriptor,
//                   &PortExtension->DeviceDescriptor,
//                   sizeof(USB_DEVICE_DESCRIPTOR));
//
//     if (PortExtension->DeviceHandle)
//     {
//         Status = USBD_GetDeviceInformationEx(PortExtension,
//                                              HubExtension,
//                                              Info,
//                                              BufferLength,
//                                              PortExtension->DeviceHandle);
//     }
//     else
//     {
//         Status = STATUS_SUCCESS;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         if (!IsExt)
//         {
//             /* IOCTL_USB_GET_NODE_CONNECTION_INFORMATION request reports
//                only low and full speed connections. Info->Speed member
//                is Info->LowSpeed in the non-EX version of the structure */
//
//             Info->Speed = (Info->Speed == UsbLowSpeed);
//         }
//
//         Irp->IoStatus.Information = sizeof(USB_NODE_CONNECTION_INFORMATION_EX) +
//                                     (Info->NumberOfOpenPipes - 1) * sizeof(USB_PIPE_INFO);
//         goto Exit;
//     }
//
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         goto Exit;
//     }
//
//     Irp->IoStatus.Information = FIELD_OFFSET(USB_NODE_CONNECTION_INFORMATION_EX,
//                                              PipeList);
//     Status = STATUS_SUCCESS;
//
// Exit:
//     USBH_CompleteIrp(Irp, Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_IoctlGetNodeConnectionDriverKeyName(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                                          IN PIRP Irp)
// {
//     PUSBHUB_PORT_DATA PortData;
//     PDEVICE_OBJECT PortDevice;
//     ULONG Length;
//     ULONG ResultLength;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     ULONG BufferLength;
//     PUSB_NODE_CONNECTION_DRIVERKEY_NAME KeyName;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//
//     DPRINT("USBH_IoctlGetNodeConnectionDriverKeyName ... \n");
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     BufferLength = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     if (BufferLength < sizeof(USB_NODE_CONNECTION_DRIVERKEY_NAME) ||
//         HubExtension->HubDescriptor->bNumberOfPorts == 0)
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Exit;
//     }
//
//     KeyName = Irp->AssociatedIrp.SystemBuffer;
//     Status = STATUS_INVALID_PARAMETER;
//
//     PortData = &HubExtension->PortData[KeyName->ConnectionIndex - 1];
//     PortDevice = PortData->DeviceObject;
//
//     if (!PortDevice)
//     {
//         goto Exit;
//     }
//
//     PortExtension = PortDevice->DeviceExtension;
//
//     if (!(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_ENUMERATED))
//     {
//         Status = STATUS_INVALID_DEVICE_STATE;
//         goto Exit;
//     }
//
//     ResultLength = BufferLength - sizeof(USB_NODE_CONNECTION_DRIVERKEY_NAME);
//
//     Status = IoGetDeviceProperty(PortDevice,
//                                  DevicePropertyDriverKeyName,
//                                  BufferLength - sizeof(USB_NODE_CONNECTION_DRIVERKEY_NAME),
//                                  &KeyName->DriverKeyName,
//                                  &ResultLength);
//
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         Status = STATUS_SUCCESS;
//     }
//
//     Length = ResultLength + sizeof(USB_NODE_CONNECTION_DRIVERKEY_NAME);
//     KeyName->ActualLength = Length;
//
//     if (BufferLength < Length)
//     {
//         KeyName->DriverKeyName[0] = 0;
//         Irp->IoStatus.Information = sizeof(USB_NODE_CONNECTION_DRIVERKEY_NAME);
//     }
//     else
//     {
//         Irp->IoStatus.Information = Length;
//     }
//
// Exit:
//     USBH_CompleteIrp(Irp, Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_IoctlGetDescriptor(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                         IN PIRP Irp)
// {
//     ULONG BufferLength;
//     PUSBHUB_PORT_DATA PortData;
//     PUSB_DESCRIPTOR_REQUEST UsbRequest;
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     struct _URB_CONTROL_TRANSFER * Urb;
//     NTSTATUS Status;
//     ULONG RequestBufferLength;
//     PIO_STACK_LOCATION IoStack;
//     ULONG NumPorts;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     BufferLength = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     DPRINT("USBH_IoctlGetDescriptor: BufferLength - %x\n", BufferLength);
//
//     if (BufferLength < sizeof(USB_DESCRIPTOR_REQUEST))
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Exit;
//     }
//
//     UsbRequest = Irp->AssociatedIrp.SystemBuffer;
//     RequestBufferLength = UsbRequest->SetupPacket.wLength;
//
//     if (RequestBufferLength > BufferLength -
//                               FIELD_OFFSET(USB_DESCRIPTOR_REQUEST, Data))
//     {
//         DPRINT("USBH_IoctlGetDescriptor: RequestBufferLength - %x\n",
//                RequestBufferLength);
//
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Exit;
//     }
//
//     Status = STATUS_INVALID_PARAMETER;
//
//     NumPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     if (NumPorts == 0 ||
//         UsbRequest->ConnectionIndex == 0 ||
//         UsbRequest->ConnectionIndex > NumPorts)
//     {
//         goto Exit;
//     }
//
//     PortData = HubExtension->PortData + (UsbRequest->ConnectionIndex - 1);
//     PortDevice = PortData->DeviceObject;
//
//     if (!PortDevice)
//     {
//         goto Exit;
//     }
//
//     PortExtension = PortDevice->DeviceExtension;
//
//     if (UsbRequest->SetupPacket.bmRequest == USB_CONFIGURATION_DESCRIPTOR_TYPE &&
//         RequestBufferLength == sizeof(USB_CONFIGURATION_DESCRIPTOR))
//     {
//         Status = STATUS_SUCCESS;
//
//         RtlCopyMemory(&UsbRequest->Data[0],
//                       &PortExtension->ConfigDescriptor,
//                       sizeof(USB_CONFIGURATION_DESCRIPTOR));
//
//         Irp->IoStatus.Information = sizeof(USB_DESCRIPTOR_REQUEST) - sizeof(UCHAR) +
//                                     sizeof(USB_CONFIGURATION_DESCRIPTOR);
//         goto Exit;
//     }
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (!Urb)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Exit;
//     }
//
//     RtlZeroMemory(Urb, sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST));
//
//     Urb->Hdr.Function = URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE;
//     Urb->Hdr.Length = sizeof(struct _URB_CONTROL_DESCRIPTOR_REQUEST);
//
//     Urb->TransferBuffer = &UsbRequest->Data[0];
//     Urb->TransferBufferLength = RequestBufferLength;
//     Urb->TransferBufferMDL = NULL;
//     Urb->UrbLink = NULL;
//
//     RtlCopyMemory(Urb->SetupPacket,
//                   &UsbRequest->SetupPacket,
//                   sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     Status = USBH_SyncSubmitUrb(PortExtension->Common.SelfDevice,
//                                 (PURB)Urb);
//
//     Irp->IoStatus.Information = (sizeof(USB_DESCRIPTOR_REQUEST) - sizeof(UCHAR)) +
//                                 Urb->TransferBufferLength;
//
//     ExFreePoolWithTag(Urb, USB_HUB_TAG);
//
// Exit:
//     USBH_CompleteIrp(Irp, Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_DeviceControl(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                    IN PIRP Irp)
// {
//     NTSTATUS Status = STATUS_DEVICE_BUSY;
//     PIO_STACK_LOCATION IoStack;
//     ULONG ControlCode;
//     BOOLEAN IsCheckHubIdle = FALSE;
//
//     DPRINT("USBH_DeviceControl: HubExtension - %p, Irp - %p\n",
//            HubExtension,
//            Irp);
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     ControlCode = IoStack->Parameters.DeviceIoControl.IoControlCode;
//     DPRINT("USBH_DeviceControl: ControlCode - %lX\n", ControlCode);
//
//     if ((HubExtension->CurrentPowerState.DeviceState != PowerDeviceD0) &&
//         (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED))
//     {
//         IsCheckHubIdle = TRUE;
//         USBH_HubSetD0(HubExtension);
//     }
//
//     switch (ControlCode)
//     {
//         case IOCTL_USB_GET_HUB_CAPABILITIES:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_HUB_CAPABILITIES\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetHubCapabilities(HubExtension, Irp);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_USB_HUB_CYCLE_PORT:
//             DPRINT1("USBH_DeviceControl: IOCTL_USB_HUB_CYCLE_PORT UNIMPLEMENTED. FIXME\n");
//             DbgBreakPoint();
//             break;
//
//         case IOCTL_USB_GET_NODE_INFORMATION:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_NODE_INFORMATION\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetNodeInformation(HubExtension, Irp);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_USB_GET_NODE_CONNECTION_INFORMATION:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_NODE_CONNECTION_INFORMATION\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetNodeConnectionInformation(HubExtension,
//                                                                 Irp,
//                                                                 FALSE);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetNodeConnectionInformation(HubExtension,
//                                                                 Irp,
//                                                                 TRUE);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_USB_GET_NODE_CONNECTION_ATTRIBUTES:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_NODE_CONNECTION_ATTRIBUTES\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetNodeConnectionAttributes(HubExtension, Irp);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_USB_GET_NODE_CONNECTION_NAME:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_NODE_CONNECTION_NAME\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetNodeName(HubExtension, Irp);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_USB_GET_NODE_CONNECTION_DRIVERKEY_NAME:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_NODE_CONNECTION_DRIVERKEY_NAME\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetNodeConnectionDriverKeyName(HubExtension, Irp);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_USB_GET_DESCRIPTOR_FROM_NODE_CONNECTION:
//             DPRINT("USBH_DeviceControl: IOCTL_USB_GET_DESCRIPTOR_FROM_NODE_CONNECTION\n");
//             if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STOPPED))
//             {
//                 Status = USBH_IoctlGetDescriptor(HubExtension, Irp);
//                 break;
//             }
//
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         case IOCTL_KS_PROPERTY:
//             DPRINT("USBH_DeviceControl: IOCTL_KS_PROPERTY\n");
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//             USBH_CompleteIrp(Irp, Status);
//             break;
//
//         default:
//             DPRINT1("USBH_DeviceControl: Unhandled IOCTL_ - %lX\n", ControlCode);
//             Status = USBH_PassIrp(HubExtension->RootHubPdo, Irp);
//             break;
//     }
//
//     if (IsCheckHubIdle)
//     {
//         USBH_CheckHubIdle(HubExtension);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoInternalControl(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                         IN PIRP Irp)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     NTSTATUS Status = STATUS_NOT_SUPPORTED;
//     ULONG ControlCode;
//     PIO_STACK_LOCATION IoStack;
//     PULONG HubCount;
//
//     DPRINT_IOCTL!("USBH_PdoInternalControl: PortExtension - %p, Irp - %p\n",
//                  PortExtension,
//                  Irp);
//
//     HubExtension = PortExtension->HubExtension;
//
//     if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_NOT_CONNECTED)
//     {
//         Status = STATUS_DEVICE_NOT_CONNECTED;
//         goto Exit;
//     }
//
//     if (PortExtension->CurrentPowerState.DeviceState != PowerDeviceD0)
//     {
//         Status = STATUS_DEVICE_POWERED_OFF;
//         goto Exit;
//     }
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     ControlCode = IoStack->Parameters.DeviceIoControl.IoControlCode;
//
//     if (ControlCode == IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO)
//     {
//         HubExtension = PortExtension->RootHubExtension;
//         DPRINT("USBH_PdoInternalControl: HubExtension - %p\n", HubExtension);
//     }
//
//     if (!HubExtension)
//     {
//         Status = STATUS_DEVICE_BUSY;
//         goto Exit;
//     }
//
//     switch (ControlCode)
//     {
//         case IOCTL_INTERNAL_USB_SUBMIT_URB:
//             return USBH_PdoIoctlSubmitUrb(PortExtension, Irp);
//
//         case IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION\n");
//             return USBH_PortIdleNotificationRequest(PortExtension, Irp);
//
//         case IOCTL_INTERNAL_USB_GET_PORT_STATUS:
//             DPRINT("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_GET_PORT_STATUS\n");
//             return USBH_PdoIoctlGetPortStatus(PortExtension, Irp);
//
//         case IOCTL_INTERNAL_USB_RESET_PORT:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_RESET_PORT\n");
//             return USBH_PdoIoctlResetPort(PortExtension, Irp);
//
//         case IOCTL_INTERNAL_USB_ENABLE_PORT:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_ENABLE_PORT\n");
//             DbgBreakPoint();
//             break;
//
//         case IOCTL_INTERNAL_USB_CYCLE_PORT:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_CYCLE_PORT\n");
//             DbgBreakPoint();
//             break;
//
//         case IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE:
//             DPRINT("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE\n");
//             *(PVOID *)IoStack->Parameters.Others.Argument1 = PortExtension->DeviceHandle;
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IOCTL_INTERNAL_USB_GET_HUB_COUNT:
//             DPRINT("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_GET_HUB_COUNT. PortPdoFlags - %lX\n",
//                    PortExtension->PortPdoFlags);
//
//             if (!(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE))
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             HubCount = IoStack->Parameters.Others.Argument1;
//
//             ++*HubCount;
//
//             Status = USBH_SyncGetHubCount(HubExtension->LowerDevice,
//                                           HubCount);
//
//             DPRINT("USBH_PdoInternalControl: *HubCount - %x\n", *HubCount);
//             break;
//
//         case IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO:
//             DPRINT("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO. PortPdoFlags - %lX\n",
//                    PortExtension->PortPdoFlags);
//
//             if (!(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE))
//             {
//                 DbgBreakPoint();
//                 Status = STATUS_SUCCESS;
//
//                 *(PVOID *)IoStack->Parameters.Others.Argument1 = NULL;
//
//                 USBH_CompleteIrp(Irp, Status);
//                 break;
//             }
//
//             ASSERT(HubExtension->RootHubPdo);
//             return USBH_PassIrp(HubExtension->RootHubPdo, Irp);
//
//         case IOCTL_INTERNAL_USB_GET_HUB_NAME:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_GET_HUB_NAME\n");
//             DbgBreakPoint();
//             break;
//
//         case IOCTL_GET_HCD_DRIVERKEY_NAME:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_GET_HCD_DRIVERKEY_NAME\n");
//             DbgBreakPoint();
//             break;
//
//         case IOCTL_INTERNAL_USB_GET_BUS_INFO:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_GET_BUS_INFO\n");
//             DbgBreakPoint();
//             break;
//
//         case IOCTL_INTERNAL_USB_GET_PARENT_HUB_INFO:
//             DPRINT1("USBH_PdoInternalControl: IOCTL_INTERNAL_USB_GET_PARENT_HUB_INFO\n");
//             DbgBreakPoint();
//             break;
//
//         default:
//             DPRINT1("USBH_PdoInternalControl: unhandled IOCTL_ - %lX\n", ControlCode);
//             break;
//     }
//
// Exit:
//     USBH_CompleteIrp(Irp, Status);
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================