use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::CONTAINING_RECORD;

use ntkrnl_sys::ddk::{
    _DEVICE_OBJECT, _DRIVER_OBJECT, _EPROCESS, _FILE_OBJECT, _IRP,
    _FAST_IO_DISPATCH, _UNICODE_STRING,
    ACCESS_MASK, ExGetPreviousMode, FAST_IO_DISPATCH,
    HANDLE, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink,
    IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoRegisterShutdownNotification,
    IoReleaseCancelSpinLock, IoSetCancelRoutine, IoWMIRegistrationControl,
    KPROCESSOR_MODE, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PDRIVER_OBJECT,
    PFILE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP,
    POBJECT_ATTRIBUTES, PUNICODE_STRING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IofCompleteRequest(Irp: PIRP, PriorityBoost: i8);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IofCompleteRequest(Irp: PIRP, PriorityBoost: i8);
}

#[inline(always)]
unsafe fn IoCompleteRequest(Irp: PIRP, PriorityBoost: i8) {
    IofCompleteRequest(Irp, PriorityBoost);
}

extern "system" {
    fn WmipOpenGuidObjectByName(
        ObjectAttributes: POBJECT_ATTRIBUTES,
        DesiredAccess: ACCESS_MASK,
        AccessMode: KPROCESSOR_MODE,
        Handle: *mut HANDLE,
        Object: *mut PVOID,
    ) -> NTSTATUS;
}

const IO_NO_INCREMENT: i8 = 0;
const FILE_DEVICE_UNKNOWN: ULONG = 0x00000022;
const FILE_DEVICE_SECURE_OPEN: ULONG = 0x00000100;
const IRP_MJ_CREATE: usize = 0x00;
const IRP_MJ_CLOSE: usize = 0x02;
const IRP_MJ_DEVICE_CONTROL: usize = 0x0e;
const IRP_MJ_CLEANUP: usize = 0x12;
const IRP_MJ_SYSTEM_CONTROL: usize = 0x17;
const IRP_MJ_SHUTDOWN: usize = 0x10;
const IOCTL_WMI_REGISTER_GUIDS: ULONG = 0x228000;
const IOCTL_WMI_UNREGISTER_GUIDS: ULONG = 0x228004;
const IOCTL_WMI_RECEIVE_NOTIFICATIONS: ULONG = 0x228008;
const IOCTL_WMI_OPEN_GUID_FOR_EVENTS: ULONG = 0x22800c;
const IOCTL_WMI_SET_MARK: ULONG = 0x228010;
const IOCTL_WMI_TRACE_EVENT: ULONG = 0x228014;
const IOCTL_WMI_TRACE_USER_MESSAGE: ULONG = 0x228018;
const WMIREG_ACTION_REGISTER: ULONG = 1;
const SPECIFIC_RIGHTS_ALL: ACCESS_MASK = 0x0000ffff;
const DO_DEVICE_INITIALIZING: ULONG = 0x00000080;
const STATUS_CANCELLED: NTSTATUS = 0xC0000120u32 as i32;
const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC0000010u32 as i32;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as i32;

// Alignment matches C WMIREGINFOW (flexible array of WMIREGGUIDW contains ULONG_PTR)
#[cfg(target_pointer_width = "64")]
#[repr(C, align(8))]
struct WMIREGINFOW {
    BufferSize: ULONG,
    NextWmiRegInfo: ULONG,
    RegistryPath: ULONG,
    MofResourceName: ULONG,
    GuidCount: ULONG,
    WmiRegGuid: [u8; 0],
}

#[cfg(target_pointer_width = "32")]
#[repr(C, align(4))]
struct WMIREGINFOW {
    BufferSize: ULONG,
    NextWmiRegInfo: ULONG,
    RegistryPath: ULONG,
    MofResourceName: ULONG,
    GuidCount: ULONG,
    WmiRegGuid: [u8; 0],
}

type TRACEHANDLE = u64;

#[repr(C)]
struct WMIP_REGISTER_GUIDS {
    ObjectAttributes: POBJECT_ATTRIBUTES,
    Unknown04: ULONG,
    Unknown08: ULONG,
    Unknown0C: ULONG,
    Unknown10: ULONG,
    Unknown14: ULONG,
    RegInfo: WMIREGINFOW,
}

#[repr(C)]
struct WMIP_RESULT {
    Handle: HANDLE,
    Unknown04: ULONG,
    TraceHandle: TRACEHANDLE,
    Unknown10: BOOLEAN,
}

#[repr(C)]
struct WMI_GUID_OBJECT_ENTRY {
    Handle: HANDLE,
    Unknown04: ULONG,
}

#[repr(C)]
struct WMI_NOTIFICATION {
    NumberOfGuidObjects: ULONG,
    Unknown04: ULONG,
    Unknown08: ULONG,
    Unknown0C: ULONG,
    Unknown10: ULONG,
    Unknown14: ULONG,
    GuidObjects: [WMI_GUID_OBJECT_ENTRY; 0],
}

#[repr(C)]
struct WMI_OPEN_GUID_FOR_EVENTS {
    ObjectAttributes: POBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    Unknown08: ULONG,
    Unknown0C: ULONG,
}

#[repr(C)]
struct WMIP_RESULT2 {
    Unknown00: ULONG,
    Unknown04: ULONG,
    Handle: HANDLE,
    Unknown0C: ULONG,
}

#[repr(C)]
struct WMI_SET_MARK {
    Flags: ULONG,
    Mark: [u16; 1],
}

#[repr(C)]
struct WMIP_IRP_CONTEXT {
    GuidObjectListHead: LIST_ENTRY,
}

#[repr(C)]
struct WMIP_GUID_OBJECT {
    IrpLink: LIST_ENTRY,
    Irp: PIRP,
}

#[no_mangle]
pub static mut WmipServiceDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

#[no_mangle]
pub static mut WmipAdminDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

#[no_mangle]
pub static mut WmipFastIoDispatch: FAST_IO_DISPATCH = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn WmipOpenCloseCleanup(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _DeviceObject = opaque_ptr(_DeviceObject);
    let Irp = opaque_ptr(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmiTraceEvent(_InputBuffer: PVOID, _PreviousMode: KPROCESSOR_MODE) -> NTSTATUS {
    let _InputBuffer = opaque_ptr(_InputBuffer);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmiTraceUserMessage(_InputBuffer: PVOID, _InputBufferLength: ULONG) -> NTSTATUS {
    let _InputBuffer = opaque_ptr(_InputBuffer);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipCaptureGuidObjectAttributes(
    GuidObjectAttributes: POBJECT_ATTRIBUTES,
    CapturedObjectAttributes: POBJECT_ATTRIBUTES,
    CapturedObjectName: PUNICODE_STRING,
    ObjectNameBuffer: PWSTR,
    AccessMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let GuidObjectAttributes = opaque_ptr(GuidObjectAttributes);
    let CapturedObjectAttributes = opaque_ptr(CapturedObjectAttributes);
    let CapturedObjectName = opaque_ptr(CapturedObjectName);
    let ObjectNameBuffer = opaque_ptr(ObjectNameBuffer);
    ntkrnl_sys::ASSERT!(AccessMode != 0);

    // C: the whole capture is one _SEH2_TRY whose handler returns the exception
    // code; each probe + copy here is the Result form of that.
    match ntkrnl_sys::probe::probe_read_generic(
        GuidObjectAttributes as *const _OBJECT_ATTRIBUTES,
        core::mem::size_of::<PVOID>(),
        AccessMode,
    ) {
        Ok(attributes) => *CapturedObjectAttributes = attributes,
        Err(status) => return status,
    }

    match ntkrnl_sys::probe::probe_read_generic(
        (*CapturedObjectAttributes).ObjectName as *const _UNICODE_STRING,
        core::mem::size_of::<PVOID>(),
        AccessMode,
    ) {
        Ok(name) => *CapturedObjectName = name,
        Err(status) => return status,
    }

    if (*CapturedObjectName).Length != 45 * core::mem::size_of::<u16>() as u16 {
        return STATUS_INVALID_PARAMETER;
    }

    if let Err(status) = ntkrnl_sys::probe::safe_copy_from_user(
        ObjectNameBuffer.cast(),
        (*CapturedObjectName).Buffer as *const core::ffi::c_void,
        (*CapturedObjectName).Length as usize,
        core::mem::size_of::<u16>(),
        AccessMode,
    ) {
        return status;
    }

    (*CapturedObjectName).Buffer = ObjectNameBuffer;
    // Divergence: the C writes `GuidObjectAttributes->ObjectName` — the CALLER'S
    // user-mode struct — under a "Fix pointers" comment. That hands user mode a
    // kernel stack address and leaves the captured attributes pointing at the
    // user's UNICODE_STRING, which WmipOpenGuidObjectByName then re-reads. The
    // captured copy is the one meant.
    (*CapturedObjectAttributes).ObjectName = CapturedObjectName;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipRegisterGuids(
    _DeviceObject: PDEVICE_OBJECT,
    Buffer: PVOID,
    InputLength: ULONG,
    OutputLength: *mut ULONG,
) -> NTSTATUS {
    let _DeviceObject = opaque_ptr(_DeviceObject);
    let Buffer = opaque_ptr(Buffer);
    let OutputLength = opaque_ptr(OutputLength);
    let RegisterGuids = Buffer as *mut WMIP_REGISTER_GUIDS;
    let Result = Buffer as *mut WMIP_RESULT;
    let mut LocalObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LocalObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalObjectNameBuffer: [u16; 46] = [0; 46];
    let mut GuidObjectHandle: HANDLE = core::ptr::null_mut();
    let mut GuidObject: PVOID = core::ptr::null_mut();

    if InputLength < core::mem::size_of::<WMIP_REGISTER_GUIDS>() as ULONG
        || (*RegisterGuids).RegInfo.BufferSize
            > InputLength - core::mem::offset_of!(WMIP_REGISTER_GUIDS, RegInfo) as ULONG
    {
        return STATUS_UNSUCCESSFUL;
    }

    if (*RegisterGuids).RegInfo.GuidCount == 0 || (*RegisterGuids).RegInfo.GuidCount > 0x10000 {
        return STATUS_UNSUCCESSFUL;
    }

    let PreviousMode = ExGetPreviousMode();
    let Status = WmipCaptureGuidObjectAttributes(
        (*RegisterGuids).ObjectAttributes,
        &mut LocalObjectAttributes,
        &mut LocalObjectName,
        LocalObjectNameBuffer.as_mut_ptr(),
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    let Status = WmipOpenGuidObjectByName(
        &mut LocalObjectAttributes,
        SPECIFIC_RIGHTS_ALL,
        PreviousMode,
        &mut GuidObjectHandle,
        &mut GuidObject,
    );
    if Status < 0 {
        return Status;
    }

    ObDereferenceObject(GuidObject);

    (*Result).Handle = GuidObjectHandle;
    (*Result).TraceHandle = 0;
    *OutputLength = 24;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipUnregisterGuids(
    _Buffer: PVOID,
    _InputLength: ULONG,
    _OutputLength: *mut ULONG,
) -> NTSTATUS {
    let _Buffer = opaque_ptr(_Buffer);
    let _OutputLength = opaque_ptr(_OutputLength);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipClearIrpObjectList(Irp: PIRP) {
    let Irp = opaque_ptr(Irp);
    let IrpContext = (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] as *mut WMIP_IRP_CONTEXT;

    let mut ListEntry = (*IrpContext).GuidObjectListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*IrpContext).GuidObjectListHead) {
        let GuidObject = CONTAINING_RECORD!(ListEntry, WMIP_GUID_OBJECT, IrpLink);

        ntkrnl_sys::ASSERT!((*GuidObject).Irp == Irp);
        (*GuidObject).Irp = core::ptr::null_mut();

        let NextEntry = (*ListEntry).Flink;
        RemoveEntryList(ListEntry);
        ListEntry = NextEntry;
    }
}

#[no_mangle]
pub unsafe extern "system" fn WmipNotificationIrpCancel(_DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let _DeviceObject = opaque_ptr(_DeviceObject);
    let Irp = opaque_ptr(Irp);
    WmipClearIrpObjectList(Irp);

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
}

#[no_mangle]
pub unsafe extern "system" fn WmipInitializeIrpContext(IrpContext: *mut WMIP_IRP_CONTEXT) {
    let IrpContext = opaque_ptr(IrpContext);
    InitializeListHead(core::ptr::addr_of_mut!((*IrpContext).GuidObjectListHead));
}

#[no_mangle]
pub unsafe extern "system" fn WmipReceiveNotifications(
    Irp: PIRP,
    Buffer: PVOID,
    InputLength: ULONG,
    OutputLength: *mut ULONG,
) -> NTSTATUS {
    let Irp = opaque_ptr(Irp);
    let Buffer = opaque_ptr(Buffer);
    let OutputLength = opaque_ptr(OutputLength);
    if InputLength < core::mem::size_of::<WMI_NOTIFICATION>() as ULONG || *OutputLength < 0x38 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let IrpContext = (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] as *mut WMIP_IRP_CONTEXT;
    WmipInitializeIrpContext(IrpContext);

    IoSetCancelRoutine(Irp, core::mem::transmute(WmipNotificationIrpCancel as usize));

    if (*Irp).Cancel != 0 && IoSetCancelRoutine(Irp, None).is_some() {
        STATUS_CANCELLED
    } else {
        IoMarkIrpPending(Irp);
        STATUS_PENDING
    }
}

#[no_mangle]
pub unsafe extern "system" fn WmipOpenGuidForEvents(
    Buffer: PVOID,
    InputLength: ULONG,
    OutputLength: *mut ULONG,
) -> NTSTATUS {
    let Buffer = opaque_ptr(Buffer);
    let OutputLength = opaque_ptr(OutputLength);
    let OpenGuidForEvents = Buffer as *mut WMI_OPEN_GUID_FOR_EVENTS;
    let Result = Buffer as *mut WMIP_RESULT2;
    let mut LocalObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LocalObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalObjectNameBuffer: [u16; 46] = [0; 46];
    let mut GuidObjectHandle: HANDLE = core::ptr::null_mut();
    let mut GuidObject: PVOID = core::ptr::null_mut();

    if InputLength != core::mem::size_of::<WMI_OPEN_GUID_FOR_EVENTS>() as ULONG
        || *OutputLength != core::mem::size_of::<WMIP_RESULT2>() as ULONG
    {
        return STATUS_UNSUCCESSFUL;
    }

    let PreviousMode = ExGetPreviousMode();
    let Status = WmipCaptureGuidObjectAttributes(
        (*OpenGuidForEvents).ObjectAttributes,
        &mut LocalObjectAttributes,
        &mut LocalObjectName,
        LocalObjectNameBuffer.as_mut_ptr(),
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    let Status = WmipOpenGuidObjectByName(
        &mut LocalObjectAttributes,
        (*OpenGuidForEvents).DesiredAccess,
        PreviousMode,
        &mut GuidObjectHandle,
        &mut GuidObject,
    );
    if Status < 0 {
        return Status;
    }

    (*Result).Handle = GuidObjectHandle;

    ObDereferenceObject(GuidObject);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmiSetMark(_Buffer: *mut WMI_SET_MARK, _Length: ULONG) -> NTSTATUS {
    let _Buffer = opaque_ptr(_Buffer);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    let IoControlCode = (*IoStackLocation).Parameters.DeviceIoControl.IoControlCode;
    let Buffer = (*Irp).AssociatedIrp.SystemBuffer;
    let InputLength = (*IoStackLocation).Parameters.DeviceIoControl.InputBufferLength;
    let mut OutputLength = (*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength;

    let Status = match IoControlCode {
        IOCTL_WMI_REGISTER_GUIDS => {
            WmipRegisterGuids(DeviceObject, Buffer, InputLength, &mut OutputLength)
        }

        IOCTL_WMI_UNREGISTER_GUIDS => {
            WmipUnregisterGuids(Buffer, InputLength, &mut OutputLength)
        }

        IOCTL_WMI_RECEIVE_NOTIFICATIONS => {
            WmipReceiveNotifications(Irp, Buffer, InputLength, &mut OutputLength)
        }

        0x228168 => STATUS_SUCCESS,

        IOCTL_WMI_OPEN_GUID_FOR_EVENTS => {
            WmipOpenGuidForEvents(Buffer, InputLength, &mut OutputLength)
        }

        IOCTL_WMI_SET_MARK => {
            if InputLength < core::mem::offset_of!(WMI_SET_MARK, Mark) as ULONG {
                STATUS_UNSUCCESSFUL
            } else {
                WmiSetMark(Buffer as *mut WMI_SET_MARK, InputLength)
            }
        }

        _ => STATUS_INVALID_DEVICE_REQUEST,
    };

    if Status == STATUS_PENDING {
        return Status;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = if Status >= 0 { OutputLength as _ } else { 0 };

    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn WmipSystemControl(
    _DeviceObject: PDEVICE_OBJECT,
    _Irp: PIRP,
) -> NTSTATUS {
    let _DeviceObject = opaque_ptr(_DeviceObject);
    let _Irp = opaque_ptr(_Irp);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmipShutdown(_DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let _DeviceObject = opaque_ptr(_DeviceObject);
    let Irp = opaque_ptr(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipFastIoDeviceControl(
    _FileObject: PFILE_OBJECT,
    _Wait: BOOLEAN,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    _OutputBuffer: PVOID,
    _OutputBufferLength: ULONG,
    IoControlCode: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    _DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let _FileObject = opaque_ptr(_FileObject);
    let InputBuffer = opaque_ptr(InputBuffer);
    let _OutputBuffer = opaque_ptr(_OutputBuffer);
    let IoStatus = opaque_ptr(IoStatus);
    let _DeviceObject = opaque_ptr(_DeviceObject);
    if IoControlCode == IOCTL_WMI_TRACE_EVENT {
        if InputBufferLength < 0x30 {
            return 0;
        }

        (*IoStatus).__bindgen_anon_1.Status = WmiTraceEvent(InputBuffer, ExGetPreviousMode());
        return 1;
    } else if IoControlCode == IOCTL_WMI_TRACE_USER_MESSAGE {
        if InputBufferLength < 0x30 {
            return 0;
        }

        (*IoStatus).__bindgen_anon_1.Status = WmiTraceUserMessage(InputBuffer, InputBufferLength);
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn WmipDockUndockEventCallback(
    _NotificationStructure: PVOID,
    _Context: PVOID,
) -> NTSTATUS {
    let _NotificationStructure = opaque_ptr(_NotificationStructure);
    let _Context = opaque_ptr(_Context);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmipDriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let _RegistryPath = opaque_ptr(_RegistryPath);
    static SERVICE_DEVICE_NAME_BUF: [u16; 21] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
        b'e' as u16, b'\\' as u16, b'W' as u16, b'M' as u16, b'I' as u16, b'D' as u16,
        b'a' as u16, b't' as u16, b'a' as u16, b'D' as u16, b'e' as u16, b'v' as u16,
        b'i' as u16, b'c' as u16, b'e' as u16,
    ];
    #[no_mangle]
    pub static mut SERVICE_DEVICE_NAME: _UNICODE_STRING = _UNICODE_STRING {
        Length: 42,
        MaximumLength: 42,
        Buffer: SERVICE_DEVICE_NAME_BUF.as_ptr() as *mut u16,
    };

    static SERVICE_DOS_DEVICE_NAME_BUF: [u16; 25] = [
        b'\\' as u16, b'D' as u16, b'o' as u16, b's' as u16, b'D' as u16, b'e' as u16,
        b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
        b'W' as u16, b'M' as u16, b'I' as u16, b'D' as u16, b'a' as u16, b't' as u16,
        b'a' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
        b'e' as u16,
    ];
    #[no_mangle]
    pub static mut SERVICE_DOS_DEVICE_NAME: _UNICODE_STRING = _UNICODE_STRING {
        Length: 50,
        MaximumLength: 50,
        Buffer: SERVICE_DOS_DEVICE_NAME_BUF.as_ptr() as *mut u16,
    };

    static ADMIN_DEVICE_NAME_BUF: [u16; 22] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
        b'e' as u16, b'\\' as u16, b'W' as u16, b'M' as u16, b'I' as u16, b'A' as u16,
        b'd' as u16, b'm' as u16, b'i' as u16, b'n' as u16, b'D' as u16, b'e' as u16,
        b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
    ];
    #[no_mangle]
    pub static mut ADMIN_DEVICE_NAME: _UNICODE_STRING = _UNICODE_STRING {
        Length: 44,
        MaximumLength: 44,
        Buffer: ADMIN_DEVICE_NAME_BUF.as_ptr() as *mut u16,
    };

    static ADMIN_DOS_DEVICE_NAME_BUF: [u16; 26] = [
        b'\\' as u16, b'D' as u16, b'o' as u16, b's' as u16, b'D' as u16, b'e' as u16,
        b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
        b'W' as u16, b'M' as u16, b'I' as u16, b'A' as u16, b'd' as u16, b'm' as u16,
        b'i' as u16, b'n' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16,
        b'c' as u16, b'e' as u16,
    ];
    #[no_mangle]
    pub static mut ADMIN_DOS_DEVICE_NAME: _UNICODE_STRING = _UNICODE_STRING {
        Length: 52,
        MaximumLength: 52,
        Buffer: ADMIN_DOS_DEVICE_NAME_BUF.as_ptr() as *mut u16,
    };

    let mut Status = IoCreateDevice(
        DriverObject,
        0,
        &raw mut SERVICE_DEVICE_NAME,
        FILE_DEVICE_UNKNOWN,
        FILE_DEVICE_SECURE_OPEN,
        0u8,
        &raw mut WmipServiceDeviceObject,
    );
    if Status < 0 {
        return Status;
    }

    Status = IoCreateSymbolicLink(&raw mut SERVICE_DOS_DEVICE_NAME, &raw mut SERVICE_DEVICE_NAME);
    if Status < 0 {
        IoDeleteDevice(WmipServiceDeviceObject);
        return Status;
    }

    Status = IoCreateDevice(
        DriverObject,
        0,
        &raw mut ADMIN_DEVICE_NAME,
        FILE_DEVICE_UNKNOWN,
        FILE_DEVICE_SECURE_OPEN,
        0u8,
        &raw mut WmipAdminDeviceObject,
    );
    if Status < 0 {
        IoDeleteDevice(WmipServiceDeviceObject);
        IoDeleteSymbolicLink(&raw mut SERVICE_DOS_DEVICE_NAME);
        return Status;
    }

    Status = IoCreateSymbolicLink(&raw mut ADMIN_DOS_DEVICE_NAME, &raw mut ADMIN_DEVICE_NAME);
    if Status < 0 {
        IoDeleteSymbolicLink(&raw mut SERVICE_DOS_DEVICE_NAME);
        IoDeleteDevice(WmipServiceDeviceObject);
        IoDeleteDevice(WmipAdminDeviceObject);
        return Status;
    }

    // SAFETY: extern "system" maps to the correct ABI per-platform (stdcall on i386, C on amd64/arm64).
    // Transmute is needed because Rust's type system treats extern "system" differently from
    // bindgen's extern "stdcall" (i386) / extern "C" (amd64) even though the ABI is identical.
    (*DriverObject).MajorFunction[IRP_MJ_CREATE] = core::mem::transmute(WmipOpenCloseCleanup as usize);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE] = core::mem::transmute(WmipOpenCloseCleanup as usize);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL] = core::mem::transmute(WmipIoControl as usize);
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP] = core::mem::transmute(WmipOpenCloseCleanup as usize);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL] = core::mem::transmute(WmipSystemControl as usize);
    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN] = core::mem::transmute(WmipShutdown as usize);

    core::ptr::write_bytes(&raw mut WmipFastIoDispatch as *mut FAST_IO_DISPATCH as *mut u8, 0, core::mem::size_of::<FAST_IO_DISPATCH>());
    WmipFastIoDispatch.SizeOfFastIoDispatch = core::mem::size_of::<FAST_IO_DISPATCH>() as ULONG;
    WmipFastIoDispatch.FastIoDeviceControl = core::mem::transmute(WmipFastIoDeviceControl as usize);
    (*DriverObject).FastIoDispatch = &raw mut WmipFastIoDispatch;

    IoWMIRegistrationControl(WmipServiceDeviceObject, WMIREG_ACTION_REGISTER);

    IoRegisterShutdownNotification(WmipServiceDeviceObject);

    (*WmipServiceDeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
    (*WmipAdminDeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/wmi/wmidrv.c
//  * PURPOSE:         I/O Windows Management Instrumentation (WMI) Support
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <wmistr.h>
// #include <wmiioctl.h>
// #include "wmip.h"
//
// #define NDEBUG
// #include <debug.h>
//
// // FIXME: these should go to a shared header
// typedef struct _WMIP_REGISTER_GUIDS
// {
//     POBJECT_ATTRIBUTES ObjectAttributes;
//     ULONG Unknown04;
//     ULONG Unknown08;
//     ULONG Unknown0C;
//     ULONG Unknown10;
//     ULONG Unknown14;
//
//     WMIREGINFOW RegInfo;
// } WMIP_REGISTER_GUIDS, *PWMIP_REGISTER_GUIDS;
//
// typedef struct _WMIP_RESULT
// {
//     HANDLE Handle;
//     ULONG Unknown04;
//     TRACEHANDLE TraceHandle;
//     BOOLEAN Unknown10;
// } WMIP_RESULT, *PWMIP_RESULT;
//
// typedef struct _WMI_UNREGISTER_GUID
// {
//     GUID Guid;
//     ULONG Unknown10;
//     ULONG Unknown14;
//     ULONG Unknown18;
//     ULONG Unknown1C;
// } WMI_UNREGISTER_GUID, *PWMI_UNREGISTER_GUID;
//
// typedef struct _WMI_GUID_OBJECT_ENTRY
// {
//     HANDLE Handle;
//     ULONG Unknown04;
// } WMI_GUID_OBJECT_ENTRY, *PWMI_GUID_OBJECT_ENTRY;
//
// typedef struct _WMI_NOTIFICATION
// {
//     ULONG NumberOfGuidObjects;
//     ULONG Unknown04;
//     ULONG Unknown08;
//     ULONG Unknown0C;
//     ULONG Unknown10;
//     ULONG Unknown14;
//     WMI_GUID_OBJECT_ENTRY GuidObjects[0];
// } WMI_NOTIFICATION, *PWMI_NOTIFICATION;
//
// typedef struct _WMI_SET_MARK
// {
//     ULONG Flags;
//     WCHAR Mark[1];
// } WMI_SET_MARK, *PWMI_SET_MARK;
//
// PDEVICE_OBJECT WmipServiceDeviceObject;
// PDEVICE_OBJECT WmipAdminDeviceObject;
// FAST_IO_DISPATCH WmipFastIoDispatch;
//
//
// /* FUNCTIONS *****************************************************************/
//
// DRIVER_DISPATCH WmipOpenCloseCleanup;
// DRIVER_DISPATCH WmipIoControl;
// DRIVER_DISPATCH WmipSystemControl;
// DRIVER_DISPATCH WmipShutdown;
//
// NTSTATUS
// NTAPI
// WmipOpenCloseCleanup(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _Inout_ PIRP Irp)
// {
//     PAGED_CODE();
//
//     /* No work to do, just return success */
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// WmiTraceEvent(
//     PVOID InputBuffer,
//     KPROCESSOR_MODE PreviousMode)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// WmiTraceUserMessage(
//     PVOID InputBuffer,
//     ULONG InputBufferLength)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// WmipCaptureGuidObjectAttributes(
//     _In_ POBJECT_ATTRIBUTES GuidObjectAttributes,
//     _Out_ POBJECT_ATTRIBUTES CapuredObjectAttributes,
//     _Out_ PUNICODE_STRING CapturedObjectName,
//     _Out_ PWSTR ObjectNameBuffer,
//     _In_ KPROCESSOR_MODE AccessMode)
// {
//     ASSERT(AccessMode != KernelMode);
//
//     _SEH2_TRY
//     {
//         /* Probe and copy the object attributes structure */
//         ProbeForRead(GuidObjectAttributes,
//                      sizeof(OBJECT_ATTRIBUTES),
//                      sizeof(PVOID));
//         *CapuredObjectAttributes = *GuidObjectAttributes;
//
//         /* Probe and copy the object name UNICODE_STRING */
//         ProbeForRead(CapuredObjectAttributes->ObjectName,
//                      sizeof(UNICODE_STRING),
//                      sizeof(PVOID));
//         *CapturedObjectName = *CapuredObjectAttributes->ObjectName;
//
//         /* Check if the object name has the expected length */
//         if (CapturedObjectName->Length != 45 * sizeof(WCHAR))
//         {
//             _SEH2_YIELD(return STATUS_INVALID_PARAMETER);
//         }
//
//         /* Probe and copy the object name buffer */
//         ProbeForRead(CapturedObjectName->Buffer,
//                      CapturedObjectName->Length,
//                      sizeof(WCHAR));
//         RtlCopyMemory(ObjectNameBuffer,
//                       CapturedObjectName->Buffer,
//                       CapturedObjectName->Length);
//
//         /* Fix pointers */
//         CapturedObjectName->Buffer = ObjectNameBuffer;
//         GuidObjectAttributes->ObjectName = CapturedObjectName;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DPRINT1("Got exception!\n");
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// WmipRegisterGuids(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_ PVOID Buffer,
//     _In_ ULONG InputLength,
//     _Inout_ PULONG OutputLength)
// {
//     PWMIP_REGISTER_GUIDS RegisterGuids = (PWMIP_REGISTER_GUIDS)Buffer;
//     PWMIP_RESULT Result = (PWMIP_RESULT)Buffer;
//     OBJECT_ATTRIBUTES LocalObjectAttributes;
//     UNICODE_STRING LocalObjectName;
//     WCHAR LocalObjectNameBuffer[45 + 1];
//     KPROCESSOR_MODE PreviousMode;
//     HANDLE GuidObjectHandle;
//     PVOID GuidObject;
//     NTSTATUS Status;
//
//     /* Make sure the input buffer is large enough */
//     if ((InputLength < sizeof(WMIP_REGISTER_GUIDS)) ||
//         (RegisterGuids->RegInfo.BufferSize >
//          (InputLength - FIELD_OFFSET(WMIP_REGISTER_GUIDS, RegInfo))))
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* Make sure we have a resonable GUID count */
//     if ((RegisterGuids->RegInfo.GuidCount == 0) ||
//         (RegisterGuids->RegInfo.GuidCount > 0x10000))
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* Capture object attributes */
//     PreviousMode = ExGetPreviousMode();
//     Status = WmipCaptureGuidObjectAttributes(RegisterGuids->ObjectAttributes,
//                                              &LocalObjectAttributes,
//                                              &LocalObjectName,
//                                              LocalObjectNameBuffer,
//                                              PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WmipCaptureGuidObjectAttributes failed: 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Open a new GUID object */
//     Status = WmipOpenGuidObjectByName(&LocalObjectAttributes,
//                                       SPECIFIC_RIGHTS_ALL,
//                                       PreviousMode,
//                                       &GuidObjectHandle,
//                                       &GuidObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WmipOpenGuidObjectByName failed: 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Dereference the GUID object */
//     ObDereferenceObject(GuidObject);
//
//     /* Return the handle (user mode will close it) */
//     Result->Handle = GuidObjectHandle;
//     Result->TraceHandle = 0;
//     *OutputLength = 24;
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// WmipUnregisterGuids(
//     _In_ PVOID Buffer,
//     _In_ ULONG InputLength,
//     _Inout_ PULONG OutputLength)
// {
//     /* For now we have nothing to do */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// WmipClearIrpObjectList(
//     _In_ PIRP Irp)
// {
//     PWMIP_IRP_CONTEXT IrpContext;
//     PLIST_ENTRY ListEntry;
//     PWMIP_GUID_OBJECT GuidObject;
//
//     /* Get the IRP context */
//     IrpContext = (PWMIP_IRP_CONTEXT)Irp->Tail.Overlay.DriverContext;
//
//     /* Loop all GUID objects attached to this IRP */
//     for (ListEntry = IrpContext->GuidObjectListHead.Flink;
//          ListEntry != &IrpContext->GuidObjectListHead;
//          ListEntry = ListEntry->Flink)
//     {
//         /* Get the GUID object */
//         GuidObject = CONTAINING_RECORD(ListEntry, WMIP_GUID_OBJECT, IrpLink);
//
//         /* Make sure the IRP matches and clear it */
//         ASSERT(GuidObject->Irp == Irp);
//         GuidObject->Irp = NULL;
//
//         /* Remove the entry */
//         RemoveEntryList(ListEntry);
//     }
// }
//
// VOID
// NTAPI
// WmipNotificationIrpCancel(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _Inout_ PIRP Irp)
// {
//     /* Clear the list */
//     WmipClearIrpObjectList(Irp);
//
//     /* Release the cancel spin lock */
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     /* Set the status to cancelled and complete the IRP */
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// static
// VOID
// WmipInitializeIrpContext(
//     PWMIP_IRP_CONTEXT IrpContext)
// {
//     /* Initialize the list head for GUID objects */
//     InitializeListHead(&IrpContext->GuidObjectListHead);
// }
//
// static
// NTSTATUS
// WmipReceiveNotifications(
//     _Inout_ PIRP Irp,
//     _In_ PVOID Buffer,
//     _In_ ULONG InputLength,
//     _Inout_ PULONG OutputLength)
// {
//     PWMI_NOTIFICATION Notification;
//     PWMIP_IRP_CONTEXT IrpContext;
//     NTSTATUS Status;
//
//     //__debugbreak();
//     if ((InputLength < sizeof(WMI_NOTIFICATION)) || (*OutputLength < 0x38))
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /// FIXME: For now we don't do any actual work, but simply pretend we are
//     /// waiting for notifications. We won't ever deliver any though.
//     Notification = (PWMI_NOTIFICATION)Buffer;
//     DBG_UNREFERENCED_LOCAL_VARIABLE(Notification);
//
//     // loop all objects
//         // reference the object
//             // on failure, fail the whole request
//
//     // loop all objects
//         // update the irp (synchronization!)
//             // if we had one before complete the old irp with an error
//
//     /* Get the IRP context and initialize it */
//     IrpContext = (PWMIP_IRP_CONTEXT)Irp->Tail.Overlay.DriverContext;
//     WmipInitializeIrpContext(IrpContext);
//
//     // loop all objects
//         // insert the objects into the IRP list
//
//     /* Set our cancel routine for cleanup */
//     IoSetCancelRoutine(Irp, WmipNotificationIrpCancel);
//
//     /* Check if the IRP is already being cancelled */
//     if (Irp->Cancel && IoSetCancelRoutine(Irp, NULL))
//     {
//         Status = STATUS_CANCELLED;
//     }
//     else
//     {
//         /* Mark the IRP as pending */
//         IoMarkIrpPending(Irp);
//         Status = STATUS_PENDING;
//     }
//
//     return Status;
// }
//
// typedef struct _WMI_OPEN_GUID_FOR_EVENTS
// {
//     POBJECT_ATTRIBUTES ObjectAttributes;
//     ACCESS_MASK DesiredAccess;
//     ULONG Unknown08;
//     ULONG Unknown0C;
// } WMI_OPEN_GUID_FOR_EVENTS, *PWMI_OPEN_GUID_FOR_EVENTS;
//
// typedef struct _WMIP_RESULT2
// {
//     ULONG Unknown00;
//     ULONG Unknown04;
//     HANDLE Handle;
//     ULONG Unknown0C;
// } WMIP_RESULT2, *PWMIP_RESULT2;
//
// static
// NTSTATUS
// WmipOpenGuidForEvents(
//     PVOID Buffer,
//     ULONG InputLength,
//     PULONG OutputLength)
// {
//     PWMI_OPEN_GUID_FOR_EVENTS OpenGuidForEvents = Buffer;
//     PWMIP_RESULT2 Result = (PWMIP_RESULT2)Buffer;
//     OBJECT_ATTRIBUTES LocalObjectAttributes;
//     UNICODE_STRING LocalObjectName;
//     WCHAR LocalObjectNameBuffer[45 + 1];
//     KPROCESSOR_MODE PreviousMode;
//     HANDLE GuidObjectHandle;
//     PVOID GuidObject;
//     NTSTATUS Status;
//
//     if ((InputLength != sizeof(WMI_OPEN_GUID_FOR_EVENTS)) ||
//         (*OutputLength != sizeof(WMIP_RESULT2)))
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* Capture object attributes */
//     PreviousMode = ExGetPreviousMode();
//     Status = WmipCaptureGuidObjectAttributes(OpenGuidForEvents->ObjectAttributes,
//                                              &LocalObjectAttributes,
//                                              &LocalObjectName,
//                                              LocalObjectNameBuffer,
//                                              PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ProbeAndCaptureGuidObjectAttributes failed: 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Open a new GUID object */
//     Status = WmipOpenGuidObjectByName(&LocalObjectAttributes,
//                                       OpenGuidForEvents->DesiredAccess,
//                                       PreviousMode,
//                                       &GuidObjectHandle,
//                                       &GuidObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WmipOpenGuidObjectByName failed: 0x%lx\n", Status);
//         return Status;
//     }
//
//     Result->Handle = GuidObjectHandle;
//
//     ObDereferenceObject(GuidObject);
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// WmiSetMark(
//     PWMI_SET_MARK Buffer,
//     ULONG Length)
// {
//     UNIMPLEMENTED;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// WmipIoControl(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _Inout_ PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStackLocation;
//     ULONG IoControlCode;
//     PVOID Buffer;
//     ULONG InputLength, OutputLength;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the current stack location */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Get the io control parameters */
//     IoControlCode = IoStackLocation->Parameters.DeviceIoControl.IoControlCode;
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//     InputLength = IoStackLocation->Parameters.DeviceIoControl.InputBufferLength;
//     OutputLength = IoStackLocation->Parameters.DeviceIoControl.OutputBufferLength;
//
//     switch (IoControlCode)
//     {
//
//         case IOCTL_WMI_REGISTER_GUIDS:
//         {
//             Status = WmipRegisterGuids(DeviceObject,
//                                        Buffer,
//                                        InputLength,
//                                        &OutputLength);
//             break;
//         }
//
//         case IOCTL_WMI_UNREGISTER_GUIDS:
//         {
//             Status = WmipUnregisterGuids(Buffer,
//                                          InputLength,
//                                          &OutputLength);
//             break;
//         }
//
//         case IOCTL_WMI_RECEIVE_NOTIFICATIONS:
//         {
//             Status = WmipReceiveNotifications(Irp,
//                                               Buffer,
//                                               InputLength,
//                                               &OutputLength);
//             break;
//         }
//
//         case 0x228168:
//         {
//             DPRINT1("IOCTL 0x228168 is unimplemented, ignoring\n");
//             Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case IOCTL_WMI_OPEN_GUID_FOR_EVENTS:
//         {
//             Status = WmipOpenGuidForEvents(Buffer, InputLength, &OutputLength);
//             break;
//         }
//
//         case IOCTL_WMI_SET_MARK:
//         {
//             if (InputLength < FIELD_OFFSET(WMI_SET_MARK, Mark))
//             {
//                 Status = STATUS_UNSUCCESSFUL;
//                 break;
//             }
//
//             Status = WmiSetMark(Buffer, InputLength);
//             break;
//         }
//
//         default:
//             DPRINT1("Unsupported yet IOCTL: 0x%lx\n", IoControlCode);
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//             __debugbreak();
//             break;
//     }
//
//     if (Status == STATUS_PENDING)
//         return Status;
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = NT_SUCCESS(Status) ? OutputLength : 0;
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// WmipSystemControl(
//   _Inout_  PDEVICE_OBJECT DeviceObject,
//   _Inout_  PIRP Irp)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// WmipShutdown(
//   _Inout_  PDEVICE_OBJECT DeviceObject,
//   _Inout_  PIRP Irp)
// {
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_SUCCESS;
// }
//
// _Function_class_(FAST_IO_DEVICE_CONTROL)
// _IRQL_requires_same_
// BOOLEAN
// NTAPI
// WmipFastIoDeviceControl(
//     _In_ PFILE_OBJECT FileObject,
//     _In_ BOOLEAN Wait,
//     _In_opt_ PVOID InputBuffer,
//     _In_ ULONG InputBufferLength,
//     _Out_opt_ PVOID OutputBuffer,
//     _In_ ULONG OutputBufferLength,
//     _In_ ULONG IoControlCode,
//     _Out_ PIO_STATUS_BLOCK IoStatus,
//     _In_ PDEVICE_OBJECT DeviceObject)
// {
//     PAGED_CODE();
//
//     if (IoControlCode == IOCTL_WMI_TRACE_EVENT)
//     {
//         if (InputBufferLength < 0x30)
//         {
//             DPRINT1("Buffer too small\n");
//             return FALSE;
//         }
//
//         IoStatus->Status = WmiTraceEvent(InputBuffer, ExGetPreviousMode());
//         return TRUE;
//     }
//     else if (IoControlCode == IOCTL_WMI_TRACE_USER_MESSAGE)
//     {
//         if (InputBufferLength < 0x30)
//         {
//             DPRINT1("Buffer too small\n");
//             return FALSE;
//         }
//
//         IoStatus->Status = WmiTraceUserMessage(InputBuffer, InputBufferLength);
//         return TRUE;
//     }
//
//     DPRINT1("Invalid io control code for fast dispatch: 0x%lx\n", IoControlCode);
//     return FALSE;
// }
//
// NTSTATUS
// NTAPI
// WmipDockUndockEventCallback(
//     _In_ PVOID NotificationStructure,
//     _Inout_opt_ PVOID Context)
// {
//     UNIMPLEMENTED_DBGBREAK();
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// _Function_class_(DRIVER_INITIALIZE)
// _IRQL_requires_same_
// NTSTATUS
// NTAPI
// WmipDriverEntry(
//     _In_ PDRIVER_OBJECT DriverObject,
//     _In_ PUNICODE_STRING RegistryPath)
// {
//     static UNICODE_STRING ServiceDeviceName = RTL_CONSTANT_STRING(L"\\Device\\WMIDataDevice");
//     static UNICODE_STRING ServiceDosDeviceName = RTL_CONSTANT_STRING(L"\\DosDevices\\WMIDataDevice");
//     static UNICODE_STRING AdminDeviceName = RTL_CONSTANT_STRING(L"\\Device\\WMIAdminDevice");
//     static UNICODE_STRING AdminDosDeviceName = RTL_CONSTANT_STRING(L"\\DosDevices\\WMIAdminDevice");
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Create the service device object */
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &ServiceDeviceName,
//                             FILE_DEVICE_UNKNOWN,
//                             FILE_DEVICE_SECURE_OPEN,
//                             0,
//                             &WmipServiceDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create service device: 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Create a symbolic link for the service device */
//     Status = IoCreateSymbolicLink(&ServiceDosDeviceName, &ServiceDeviceName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoCreateSymbolicLink() failed: 0x%lx\n", Status);
//         IoDeleteDevice(WmipServiceDeviceObject);
//         return Status;
//     }
//
//     /* Create the admin device object */
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &AdminDeviceName,
//                             FILE_DEVICE_UNKNOWN,
//                             FILE_DEVICE_SECURE_OPEN,
//                             0,
//                             &WmipAdminDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create admin device: 0x%lx\n", Status);
//         IoDeleteDevice(WmipServiceDeviceObject);
//         IoDeleteSymbolicLink(&ServiceDosDeviceName);
//         return Status;
//     }
//
//     /* Create a symbolic link for the admin device */
//     Status = IoCreateSymbolicLink(&AdminDosDeviceName, &AdminDeviceName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoCreateSymbolicLink() failed: 0x%lx\n", Status);
//         IoDeleteSymbolicLink(&ServiceDosDeviceName);
//         IoDeleteDevice(WmipServiceDeviceObject);
//         IoDeleteDevice(WmipAdminDeviceObject);
//         return Status;
//     }
//
//     /* Initialize dispatch routines */
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = WmipOpenCloseCleanup;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = WmipOpenCloseCleanup;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = WmipIoControl;
//     DriverObject->MajorFunction[IRP_MJ_CLEANUP] = WmipOpenCloseCleanup;
//     DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = WmipSystemControl;
//     DriverObject->MajorFunction[IRP_MJ_SHUTDOWN] = WmipShutdown;
//
//     /* Initialize fast dispatch */
//     RtlZeroMemory(&WmipFastIoDispatch, sizeof(WmipFastIoDispatch));
//     WmipFastIoDispatch.SizeOfFastIoDispatch = sizeof(WmipFastIoDispatch);
//     WmipFastIoDispatch.FastIoDeviceControl = WmipFastIoDeviceControl;
//     DriverObject->FastIoDispatch = &WmipFastIoDispatch;
//
//     /* Register the WMI service device */
//     IoWMIRegistrationControl(WmipServiceDeviceObject, WMIREG_ACTION_REGISTER);
//
//     /* Register a shutdown notification */
//     IoRegisterShutdownNotification(WmipServiceDeviceObject);
//
//     /* Initialization is done */
//     WmipServiceDeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//     WmipAdminDeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     return STATUS_SUCCESS;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<WMIP_REGISTER_GUIDS>() == 56);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, ObjectAttributes) == 0);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown04) == 8);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown08) == 12);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown0C) == 16);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown10) == 20);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown14) == 24);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, RegInfo) == 32);
    assert!(core::mem::size_of::<WMIP_RESULT>() == 32);
    assert!(core::mem::offset_of!(WMIP_RESULT, Handle) == 0);
    assert!(core::mem::offset_of!(WMIP_RESULT, Unknown04) == 8);
    assert!(core::mem::offset_of!(WMIP_RESULT, TraceHandle) == 16);
    assert!(core::mem::offset_of!(WMIP_RESULT, Unknown10) == 24);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<WMIP_REGISTER_GUIDS>() == 44);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, ObjectAttributes) == 0);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown04) == 4);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown08) == 8);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown0C) == 12);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown10) == 16);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, Unknown14) == 20);
    assert!(core::mem::offset_of!(WMIP_REGISTER_GUIDS, RegInfo) == 24);
    assert!(core::mem::size_of::<WMIP_RESULT>() == 24);
    assert!(core::mem::offset_of!(WMIP_RESULT, Handle) == 0);
    assert!(core::mem::offset_of!(WMIP_RESULT, Unknown04) == 4);
    assert!(core::mem::offset_of!(WMIP_RESULT, TraceHandle) == 8);
    assert!(core::mem::offset_of!(WMIP_RESULT, Unknown10) == 16);
};
// END OFFSET CHECKS
