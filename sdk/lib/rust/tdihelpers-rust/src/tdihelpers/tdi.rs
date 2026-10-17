use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, FILE_FULL_EA_INFORMATION, _IO_STATUS_BLOCK, IoAllocateMdl, IoBuildDeviceIoControlRequest, IoFreeMdl, IoGetRelatedDeviceObject, KEVENT, KeInitializeEvent, KeWaitForSingleObject, MmProbeAndLockPages, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_FULL_EA_INFORMATION, PFILE_OBJECT, PHANDLE, PIO_COMPLETION_ROUTINE, PIO_STATUS_BLOCK, PIRP, PKEVENT, PLARGE_INTEGER, PMDL, PUNICODE_STRING, ZwClose, ZwCreateFile};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static TdiTransportAddress: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"TransportAddress\0")
};

pub static TdiConnectionContext: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"ConnectionContext\0")
};

pub const TDI_TRANSPORT_ADDRESS_LENGTH: usize =
    (::core::mem::size_of::<[::core::ffi::c_char; 17]>() as usize).wrapping_sub(1 as usize);

pub const TDI_CONNECTION_CONTEXT_LENGTH: usize =
    (::core::mem::size_of::<[::core::ffi::c_char; 18]>() as usize).wrapping_sub(1 as usize);

unsafe extern "C" fn TdiCall(
    mut Irp: PIRP,
    mut DeviceObject: PDEVICE_OBJECT,
    mut Event: PKEVENT,
    mut Iosb: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;

    DPRINT!("Called\n");

    DPRINT!("Irp->UserEvent = %p\n", (*Irp).UserEvent);

    Status = IoCallDriver(DeviceObject, Irp);
    DPRINT!("IoCallDriver: %08x\n", Status);

    if (Status == STATUS_PENDING) && (!Event.is_null()) {
        DPRINT!("Waiting on transport.\n");
        KeWaitForSingleObject(
            Event as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as i8,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = (*Iosb).__bindgen_anon_1.Status;
    }

    DPRINT!("Status (0x%X).\n", Status);

    return Status;
}

unsafe extern "C" fn TdiOpenDevice(
    mut DeviceName: PUNICODE_STRING,
    mut EaLength: ULONG,
    mut EaInfo: PFILE_FULL_EA_INFORMATION,
    mut ShareType: ULONG,
    mut Handle: PHANDLE,
    mut Object: *mut PFILE_OBJECT,
) -> NTSTATUS {
    let mut Attr: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut ShareAccess: ULONG = 0;

    DPRINT!("Called. DeviceName (%wZ, %u)\n", DeviceName, ShareType);

    /* Determine the share access */
    if ShareType != (AFD_SHARE_REUSE as ULONG) {
        /* Exclusive access */
        ShareAccess = (0 as ULONG);
    } else {
        /* Shared access */
        ShareAccess = ((FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG);
    }

    /* InitializeObjectAttributes(&Attr, DeviceName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL) */
    Attr.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    Attr.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>() as HANDLE;
    Attr.ObjectName = DeviceName;
    Attr.Attributes = ((OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG);
    Attr.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;
    Attr.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;

    Status = ZwCreateFile(
        Handle,                                                                  /* Return file handle */
        ((GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE) as ACCESS_MASK), /* Desired access */
        core::ptr::addr_of_mut!(Attr),                                           /* Object attributes */
        core::ptr::addr_of_mut!(Iosb),                                           /* IO status */
        core::ptr::null_mut(),                                                  /* Initial allocation size */
        (FILE_ATTRIBUTE_NORMAL as ULONG),                                       /* File attributes */
        ShareAccess,                                                             /* Share access */
        (FILE_OPEN_IF as ULONG),                                       /* Create disposition */
        (0 as ULONG),                                       /* Create options */
        EaInfo as PVOID,                                                         /* EA buffer */
        EaLength,                                                                /* EA length */
    );
    if NT_SUCCESS!(Status) {
        Status = ObReferenceObjectByHandle(
            *Handle,                                                             /* Handle to open file */
            ((GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE) as ACCESS_MASK), /* Access mode */
            IoFileObjectType as POBJECT_TYPE,                                    /* Object type */
            (0i32 as KPROCESSOR_MODE),                         /* Access mode */
            Object as *mut PVOID,                                                /* Pointer to object */
            core::ptr::null_mut::<_OBJECT_HANDLE_INFORMATION>(),                 /* Handle information */
        );
        if !NT_SUCCESS!(Status) {
            DPRINT!("ObReferenceObjectByHandle() failed with status (0x%X).\n", Status);
            ZwClose(*Handle);
        } else {
            DPRINT!("Got handle (%p)  Object (%p)\n", *Handle, *Object);
        }
    } else {
        DPRINT!("ZwCreateFile() failed with status (0x%X)\n", Status);
    }

    if !NT_SUCCESS!(Status) {
        *Handle = (INVALID_HANDLE_VALUE as *mut core::ffi::c_void);
        *Object = core::ptr::null_mut::<_FILE_OBJECT>();
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TdiOpenAddressFile(
    mut DeviceName: PUNICODE_STRING,
    mut Name: PTRANSPORT_ADDRESS,
    mut ShareType: ULONG,
    mut AddressHandle: PHANDLE,
    mut AddressObject: *mut PFILE_OBJECT,
) -> NTSTATUS {
    let mut EaInfo: PFILE_FULL_EA_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut EaLength: ULONG;
    let mut Address: PTRANSPORT_ADDRESS;

    DPRINT!("Called. DeviceName (%wZ)  Name (%p)\n", DeviceName, Name);

    /* EaName must be 0-terminated, even though TDI_TRANSPORT_ADDRESS_LENGTH does *not* include the 0 */
    EaLength = (core::mem::size_of::<FILE_FULL_EA_INFORMATION>() as ULONG)
        + (TDI_TRANSPORT_ADDRESS_LENGTH as ULONG)
        + (TaLengthOfTransportAddress(Name) as ULONG)
        + (1 as ULONG);
    EaInfo = ExAllocatePoolWithTag(
        NonPagedPool as _,
        EaLength as SIZE_T,
        TAG_AFD_EA_INFO as ULONG,
    ) as PFILE_FULL_EA_INFORMATION;
    if EaInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory!(EaInfo as PVOID, EaLength as SIZE_T);
    (*EaInfo).EaNameLength = TDI_TRANSPORT_ADDRESS_LENGTH as UCHAR;
    /* Don't copy the terminating 0; we have already zeroed it */
    RtlCopyMemory!(
        core::ptr::addr_of_mut!((*EaInfo).EaName) as *mut CHAR as *mut core::ffi::c_void,
        core::ptr::addr_of!(TdiTransportAddress) as *const core::ffi::c_void,
        TDI_TRANSPORT_ADDRESS_LENGTH as SIZE_T,
    );
    (*EaInfo).EaValueLength = core::mem::size_of::<TA_IP_ADDRESS>() as USHORT;
    Address = ((core::ptr::addr_of_mut!((*EaInfo).EaName)) as *mut CHAR)
        .offset(TDI_TRANSPORT_ADDRESS_LENGTH as isize + 1 as isize)
        as PTRANSPORT_ADDRESS; /* 0-terminated */
    TaCopyTransportAddressInPlace(Address, Name);

    Status = TdiOpenDevice(
        DeviceName,
        EaLength,
        EaInfo,
        ShareType,
        AddressHandle,
        AddressObject,
    );
    ExFreePoolWithTag(EaInfo as PVOID, TAG_AFD_EA_INFO as ULONG);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TdiQueryMaxDatagramLength(
    FileObject: PFILE_OBJECT,
    MaxDatagramLength: PUINT,
) -> NTSTATUS {
    let mut Mdl: PMDL = core::ptr::null_mut();
    let mut Buffer: PTDI_MAX_DATAGRAM_INFO;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<TDI_MAX_DATAGRAM_INFO>() as SIZE_T,
        TAG_AFD_DATA_BUFFER as ULONG,
    ) as PTDI_MAX_DATAGRAM_INFO;

    if Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    Mdl = IoAllocateMdl(
        Buffer as PVOID,
        core::mem::size_of::<TDI_MAX_DATAGRAM_INFO>() as ULONG,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    );
    if Mdl.is_null() {
        ExFreePoolWithTag(Buffer as PVOID, TAG_AFD_DATA_BUFFER as ULONG);
        return STATUS_NO_MEMORY;
    }

    // _SEH2_TRY/_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) as a real handler (ntkrnl_sys::seh);
    // the C's except arm is dead under the kernel's DUMMY_PSEH.
    let mut __seh = ();
    if let Err(code) = ntkrnl_sys::seh::try_except(
        &mut __seh,
        |_| {
            MmProbeAndLockPages(Mdl, KernelMode as KPROCESSOR_MODE, IoModifyAccess);
        },
        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    ) {
        Status = code;
    }

    if !NT_SUCCESS!(Status) {
        DPRINT!("Failed to lock pages\n");
        IoFreeMdl(Mdl);
        ExFreePoolWithTag(Buffer as PVOID, TAG_AFD_DATA_BUFFER as ULONG);
        return Status;
    }

    Status = TdiQueryInformation(
        FileObject,
        (TDI_QUERY_MAX_DATAGRAM_INFO) as LONG,
        Mdl,
    );
    if !NT_SUCCESS!(Status) {
        ExFreePoolWithTag(Buffer as PVOID, TAG_AFD_DATA_BUFFER as ULONG);
        return Status;
    }

    *MaxDatagramLength = (*Buffer).MaxDatagramSize as ::core::ffi::c_uint;

    ExFreePoolWithTag(Buffer as PVOID, TAG_AFD_DATA_BUFFER as ULONG);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn TdiOpenConnectionEndpointFile(
    DeviceName: PUNICODE_STRING,
    ConnectionHandle: PHANDLE,
    ConnectionObject: *mut PFILE_OBJECT,
) -> NTSTATUS {
    let mut EaInfo: PFILE_FULL_EA_INFORMATION;
    let mut ContextArea: *mut PVOID;
    let mut Status: NTSTATUS;
    let mut EaLength: ULONG;

    DPRINT!("Called. DeviceName (%wZ)\n", DeviceName);

    /* EaName must be 0-terminated, even though TDI_TRANSPORT_ADDRESS_LENGTH does *not* include the 0 */
    EaLength = (core::mem::size_of::<FILE_FULL_EA_INFORMATION>() as ULONG)
        + (TDI_CONNECTION_CONTEXT_LENGTH) as u32
        + (core::mem::size_of::<PVOID>() as ULONG)
        + 1;

    EaInfo = ExAllocatePoolWithTag(
        NonPagedPool,
        EaLength as SIZE_T,
        TAG_AFD_EA_INFO,
    ) as PFILE_FULL_EA_INFORMATION;
    if EaInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory!(EaInfo as *mut ::core::ffi::c_void, EaLength as SIZE_T);
    (*EaInfo).EaNameLength = TDI_CONNECTION_CONTEXT_LENGTH as UCHAR;
    /* Don't copy the terminating 0; we have already zeroed it */
    RtlCopyMemory!(
        core::ptr::addr_of_mut!((*EaInfo).EaName) as *mut ::core::ffi::c_void,
        core::ptr::addr_of!(TdiConnectionContext) as *const ::core::ffi::c_void,
        TDI_CONNECTION_CONTEXT_LENGTH as SIZE_T,
    );
    (*EaInfo).EaValueLength = core::mem::size_of::<PVOID>() as USHORT;
    ContextArea = (core::ptr::addr_of_mut!((*EaInfo).EaName) as *mut CHAR)
        .offset(TDI_CONNECTION_CONTEXT_LENGTH as isize + 1 as isize)
        as *mut PVOID; /* 0-terminated */
    /* FIXME: Allocate context area */
    *ContextArea = core::ptr::null_mut();
    Status = TdiOpenDevice(
        DeviceName,
        EaLength,
        EaInfo,
        AFD_SHARE_UNIQUE,
        ConnectionHandle,
        ConnectionObject,
    );
    ExFreePoolWithTag(EaInfo as PVOID, TAG_AFD_EA_INFO);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TdiConnect(
    Irp: *mut PIRP,
    ConnectionObject: PFILE_OBJECT,
    ConnectionCallInfo: PTDI_CONNECTION_INFORMATION,
    ConnectionReturnInfo: PTDI_CONNECTION_INFORMATION,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;

    DPRINT!("Called\n");

    if ConnectionObject.is_null() {
        DPRINT!("Bad connection object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(ConnectionObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_CONNECT,
        DeviceObject,
        ConnectionObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if (*Irp).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildConnect!(
        *Irp,
        DeviceObject,
        ConnectionObject,
        CompletionRoutine,
        CompletionContext,
        core::ptr::null_mut(),
        ConnectionCallInfo,
        ConnectionReturnInfo,
    );

    return TdiCall(*Irp, DeviceObject, core::ptr::null_mut(), core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn TdiAssociateAddressFile(
    AddressHandle: HANDLE,
    ConnectionObject: PFILE_OBJECT,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP = core::ptr::null_mut();

    DPRINT!("Called. AddressHandle (%p)  ConnectionObject (%p)\n", AddressHandle, ConnectionObject);

    if ConnectionObject.is_null() {
        DPRINT!("Bad connection object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(ConnectionObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_ASSOCIATE_ADDRESS,
        DeviceObject,
        ConnectionObject,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(Iosb),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildAssociateAddress!(
        Irp,
        DeviceObject,
        ConnectionObject,
        None,
        core::ptr::null_mut(),
        AddressHandle,
    );

    return TdiCall(
        Irp,
        DeviceObject,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(Iosb),
    );
}
#[no_mangle]
pub unsafe extern "C" fn TdiDisassociateAddressFile(
    ConnectionObject: PFILE_OBJECT,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP = core::ptr::null_mut();

    DPRINT!("Called. ConnectionObject (%p)\n", ConnectionObject);

    if ConnectionObject.is_null() {
        DPRINT!("Bad connection object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(ConnectionObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_DISASSOCIATE_ADDRESS,
        DeviceObject,
        ConnectionObject,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(Iosb),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildDisassociateAddress!(
        Irp,
        DeviceObject,
        ConnectionObject,
        None,
        core::ptr::null_mut(),
    );

    return TdiCall(
        Irp,
        DeviceObject,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(Iosb),
    );
}
#[no_mangle]
pub unsafe extern "C" fn TdiListen(
    Irp: *mut PIRP,
    ConnectionObject: PFILE_OBJECT,
    RequestConnectionInfo: *mut PTDI_CONNECTION_INFORMATION,
    ReturnConnectionInfo: *mut PTDI_CONNECTION_INFORMATION,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    DPRINT!("Called\n");

    if ConnectionObject.is_null() {
        DPRINT!("Bad connection object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(ConnectionObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_LISTEN,
        DeviceObject,
        ConnectionObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if (*Irp).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildListen!(
        *Irp,
        DeviceObject,
        ConnectionObject,
        CompletionRoutine,
        CompletionContext,
        0,
        *RequestConnectionInfo,
        *ReturnConnectionInfo,
    );

    TdiCall(
        *Irp,
        DeviceObject,
        core::ptr::null_mut(), // Don't wait for completion
        core::ptr::null_mut(),
    );

    return STATUS_PENDING;
}
#[no_mangle]
pub unsafe extern "C" fn TdiAccept(
    Irp: *mut PIRP,
    AcceptConnectionObject: PFILE_OBJECT,
    RequestConnectionInfo: PTDI_CONNECTION_INFORMATION,
    ReturnConnectionInfo: PTDI_CONNECTION_INFORMATION,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;

    DPRINT!("Called\n");

    if AcceptConnectionObject.is_null() {
        DPRINT!("Bad connection object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(AcceptConnectionObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_LISTEN,
        DeviceObject,
        AcceptConnectionObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if (*Irp).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildAccept!(
        *Irp,
        DeviceObject,
        AcceptConnectionObject,
        CompletionRoutine,
        CompletionContext,
        RequestConnectionInfo,
        ReturnConnectionInfo,
    );

    TdiCall(
        *Irp,
        DeviceObject,
        core::ptr::null_mut(), // Don't wait for completion
        core::ptr::null_mut(),
    );

    return STATUS_PENDING;
}
#[no_mangle]
pub unsafe extern "C" fn TdiSetEventHandler(
    FileObject: PFILE_OBJECT,
    EventType: LONG,
    Handler: PVOID,
    Context: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP = core::ptr::null_mut();

    DPRINT!("Called\n");

    if FileObject.is_null() {
        DPRINT!("Bad file object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_SET_EVENT_HANDLER,
        DeviceObject,
        FileObject,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(Iosb),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildSetEventHandler!(
        Irp,
        DeviceObject,
        FileObject,
        None,
        core::ptr::null_mut(),
        EventType,
        Handler,
        Context,
    );

    return TdiCall(
        Irp,
        DeviceObject,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(Iosb),
    );
}
#[no_mangle]
pub unsafe extern "C" fn TdiQueryDeviceControl(
    FileObject: PFILE_OBJECT,
    IoControlCode: ULONG,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    Return: PULONG,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP;

    if FileObject.is_null() {
        DPRINT!("Bad file object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, FALSE);

    Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        DeviceObject,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        FALSE,
        &mut Event,
        &mut Iosb,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = TdiCall(Irp, DeviceObject, &mut Event, &mut Iosb);

    if !Return.is_null() {
        *Return = Iosb.Information as ULONG;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TdiQueryInformation(
    FileObject: PFILE_OBJECT,
    QueryType: LONG,
    MdlBuffer: PMDL,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP;

    if FileObject.is_null() {
        DPRINT!("Bad file object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_NotificationEvent, FALSE);

    Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_QUERY_INFORMATION,
        DeviceObject,
        ConnectionObject,
        &raw mut Event,
        &raw mut Iosb,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildQueryInformation!(
        Irp,
        DeviceObject,
        FileObject,
        None,
        core::ptr::null_mut(),
        QueryType,
        MdlBuffer,
    );

    return TdiCall(Irp, DeviceObject, &raw mut Event, &raw mut Iosb);
}
#[no_mangle]
pub unsafe extern "C" fn TdiQueryInformationEx(
    FileObject: PFILE_OBJECT,
    Entity: ULONG,
    Instance: ULONG,
    Class: ULONG,
    Type: ULONG,
    Id: ULONG,
    OutputBuffer: PVOID,
    OutputLength: PULONG,
) -> NTSTATUS {
    let mut QueryInfo: TCP_REQUEST_QUERY_INFORMATION_EX = core::mem::zeroed();

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(QueryInfo) as PVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>(),
    );
    (QueryInfo.ID.toi_entity.tei_entity) = Entity;
    (QueryInfo.ID.toi_entity.tei_instance) = Instance;
    (QueryInfo.ID.toi_class) = Class;
    (QueryInfo.ID.toi_type) = Type;
    (QueryInfo.ID.toi_id) = Id;

    return TdiQueryDeviceControl(
        FileObject,
        IOCTL_TCP_QUERY_INFORMATION_EX as ULONG,
        core::ptr::addr_of_mut!(QueryInfo) as PVOID,
        core::mem::size_of::<TCP_REQUEST_QUERY_INFORMATION_EX>() as ULONG,
        OutputBuffer,
        *OutputLength,
        OutputLength,
    );
}
#[no_mangle]
pub unsafe extern "C" fn TdiQueryAddress(
    FileObject: PFILE_OBJECT,
    Address: PULONG,
) -> NTSTATUS {
    let mut i: UINT = 0;
    let mut Entities: *mut TDIEntityID = core::ptr::null_mut();
    let mut EntityCount: ULONG = 0;
    let mut EntityType: ULONG = 0;
    let mut SnmpInfo: IPSNMPInfo = core::mem::zeroed();
    let mut IpAddress: PIPADDR_ENTRY = core::ptr::null_mut();
    let mut BufferSize: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    DPRINT!("Called\n");

    BufferSize = (core::mem::size_of::<TDIEntityID>() as ULONG) * 20;
    Entities = ExAllocatePoolWithTag(
        NonPagedPool,
        BufferSize as SIZE_T,
        TAG_AFD_TRANSPORT_ADDRESS,
    ) as *mut TDIEntityID;
    if Entities.is_null() {
        DPRINT!("Insufficient resources.\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Query device for supported entities */

    Status = TdiQueryInformationEx(
        FileObject,                          /* File object */
        GENERIC_ENTITY,              /* Entity */
        TL_INSTANCE,              /* Instance */
        INFO_CLASS_GENERIC,              /* Entity class */
        INFO_TYPE_PROVIDER,              /* Entity type */
        ENTITY_LIST_ID,              /* Entity id */
        Entities as PVOID,                   /* Output buffer */
        core::ptr::addr_of_mut!(BufferSize), /* Output buffer size */
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!(
            "Unable to get list of supported entities (Status = 0x%X).\n",
            Status
        );
        ExFreePoolWithTag(Entities as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
        return Status;
    }

    /* Locate an IP entity */
    EntityCount = BufferSize / (core::mem::size_of::<TDIEntityID>() as ULONG);

    DPRINT!("EntityCount = %u\n", EntityCount);

    i = 0;
    while (i as ULONG) < EntityCount {
        if (*Entities.offset(i as isize)).tei_entity == CL_NL_ENTITY {
            /* Query device for entity type */

            BufferSize = core::mem::size_of::<ULONG>() as ULONG;
            Status = TdiQueryInformationEx(
                FileObject,                             /* File object */
                CL_NL_ENTITY,                 /* Entity */
                (*Entities.offset(i as isize)).tei_instance, /* Instance */
                INFO_CLASS_GENERIC,                 /* Entity class */
                INFO_TYPE_PROVIDER,                 /* Entity type */
                ENTITY_TYPE_ID,                 /* Entity id */
                core::ptr::addr_of_mut!(EntityType) as PVOID, /* Output buffer */
                core::ptr::addr_of_mut!(BufferSize),    /* Output buffer size */
            );
            if !NT_SUCCESS!(Status) || (EntityType != CL_NL_IP) {
                DPRINT!("Unable to get entity of type IP (Status = 0x%X).\n", Status);
                break;
            }

            /* Query device for SNMP information */

            BufferSize = core::mem::size_of::<IPSNMPInfo>() as ULONG;
            Status = TdiQueryInformationEx(
                FileObject,                             /* File object */
                CL_NL_ENTITY,                 /* Entity */
                (*Entities.offset(i as isize)).tei_instance, /* Instance */
                INFO_CLASS_PROTOCOL,                 /* Entity class */
                INFO_TYPE_PROVIDER,                 /* Entity type */
                IP_MIB_STATS_ID,                 /* Entity id */
                core::ptr::addr_of_mut!(SnmpInfo) as PVOID, /* Output buffer */
                core::ptr::addr_of_mut!(BufferSize),    /* Output buffer size */
            );
            if !NT_SUCCESS!(Status) || (SnmpInfo.ipsi_numaddr == 0) {
                DPRINT!(
                    "Unable to get SNMP information or no IP addresses available (Status = 0x%X).\n",
                    Status
                );
                break;
            }

            /* Query device for all IP addresses */

            if SnmpInfo.ipsi_numaddr != 0 {
                BufferSize = (SnmpInfo.ipsi_numaddr as usize)
                    .wrapping_mul(core::mem::size_of::<IPADDR_ENTRY>() as usize)
                    as ULONG;
                IpAddress = ExAllocatePoolWithTag(
                    NonPagedPool,
                    BufferSize as SIZE_T,
                    TAG_AFD_SNMP_ADDRESS_INFO,
                ) as PIPADDR_ENTRY;
                if IpAddress.is_null() {
                    DPRINT!("Insufficient resources.\n");
                    break;
                }

                Status = TdiQueryInformationEx(
                    FileObject,                             /* File object */
                    CL_NL_ENTITY,                 /* Entity */
                    (*Entities.offset(i as isize)).tei_instance, /* Instance */
                    INFO_CLASS_PROTOCOL,                 /* Entity class */
                    INFO_TYPE_PROVIDER,                 /* Entity type */
                    IP_MIB_ADDRTABLE_ENTRY_ID,                 /* Entity id */
                    IpAddress as PVOID,                     /* Output buffer */
                    core::ptr::addr_of_mut!(BufferSize),    /* Output buffer size */
                );
                if !NT_SUCCESS!(Status) {
                    DPRINT!("Unable to get IP address (Status = 0x%X).\n", Status);
                    ExFreePoolWithTag(IpAddress as PVOID, TAG_AFD_SNMP_ADDRESS_INFO);
                    break;
                }

                if SnmpInfo.ipsi_numaddr != 1 {
                    /* Skip loopback address */
                    *Address = DN2H!((*IpAddress.offset(1 as isize)).Addr);
                } else {
                    /* Select the first address returned */
                    *Address = DN2H!((*IpAddress).Addr);
                }

                ExFreePoolWithTag(IpAddress as PVOID, TAG_AFD_SNMP_ADDRESS_INFO);
            } else {
                Status = STATUS_UNSUCCESSFUL;
                break;
            }
        }
        i = i.wrapping_add(1);
    }

    ExFreePoolWithTag(Entities as PVOID, TAG_AFD_TRANSPORT_ADDRESS);

    DPRINT!("Leaving\n");

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TdiSend(
    Irp: *mut PIRP,
    TransportObject: PFILE_OBJECT,
    Flags: USHORT,
    Buffer: PCHAR,
    BufferLength: UINT,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Mdl: PMDL;

    if TransportObject.is_null() {
        DPRINT!("Bad transport object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(TransportObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_SEND,
        DeviceObject,
        TransportObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if (*Irp).is_null() {
        DPRINT!("Insufficient resources.\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("Allocating irp for %p:%u\n", Buffer, BufferLength);

    Mdl = IoAllocateMdl(
        Buffer as PVOID,            /* Virtual address */
        BufferLength,               /* Length of buffer */
        FALSE,     /* Not secondary */
        FALSE,     /* Don't charge quota */
        core::ptr::null_mut(),      /* Don't use IRP */
    );
    if Mdl.is_null() {
        DPRINT!("Insufficient resources.\n");
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // _SEH2_TRY/_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) as a real handler (ntkrnl_sys::seh);
    // the C's except arm is dead under the kernel's DUMMY_PSEH.
    let mut __seh = ();
    if ntkrnl_sys::seh::try_except(
        &mut __seh,
        |_| {
            MmProbeAndLockPages(Mdl, (**Irp).RequestorMode, IoReadAccess);
        },
        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    )
    .is_err()
    {
        DPRINT!("MmProbeAndLockPages() failed.\n");
        IoFreeMdl(Mdl);
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("AFD>>> Got an MDL: %p\n", Mdl);

    TdiBuildSend!(
        *Irp,
        DeviceObject,
        TransportObject,
        CompletionRoutine,
        CompletionContext,
        Mdl,
        Flags,
        BufferLength,
    );

    /* Does not block...  The MDL is deleted in the receive completion
       routine. */
    return TdiCall(*Irp, DeviceObject, core::ptr::null_mut(), core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn TdiReceive(
    Irp: *mut PIRP,
    TransportObject: PFILE_OBJECT,
    Flags: USHORT,
    Buffer: PCHAR,
    BufferLength: UINT,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Mdl: PMDL;

    if TransportObject.is_null() {
        DPRINT!("Bad transport object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(TransportObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_RECEIVE,
        DeviceObject,
        TransportObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if (*Irp).is_null() {
        DPRINT!("Insufficient resources.\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("Allocating irp for %p:%u\n", Buffer, BufferLength);

    Mdl = IoAllocateMdl(
        Buffer as PVOID,            // Virtual address
        BufferLength as ULONG,      // Length of buffer
        FALSE,     // Not secondary
        FALSE,     // Don't charge quota
        core::ptr::null_mut(),      // Don't use IRP
    );
    if Mdl.is_null() {
        DPRINT!("Insufficient resources.\n");
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // _SEH2_TRY/_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) as a real handler (ntkrnl_sys::seh);
    // the C's except arm is dead under the kernel's DUMMY_PSEH.
    let mut __seh = ();
    if ntkrnl_sys::seh::try_except(
        &mut __seh,
        |_| {
            DPRINT!("probe and lock\n");
            MmProbeAndLockPages(Mdl, (**Irp).RequestorMode, IoModifyAccess);
            DPRINT!("probe and lock done\n");
        },
        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    )
    .is_err()
    {
        DPRINT!("MmProbeAndLockPages() failed.\n");
        IoFreeMdl(Mdl);
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("AFD>>> Got an MDL: %p\n", Mdl);

    TdiBuildReceive!(
        *Irp,
        DeviceObject,
        TransportObject,
        CompletionRoutine,
        CompletionContext,
        Mdl,
        Flags,
        BufferLength,
    );

    TdiCall(*Irp, DeviceObject, core::ptr::null_mut(), core::ptr::null_mut());
    // Does not block...  The MDL is deleted in the receive completion
    // routine.

    return STATUS_PENDING;
}
#[no_mangle]
pub unsafe extern "C" fn TdiReceiveDatagram(
    Irp: *mut PIRP,
    TransportObject: PFILE_OBJECT,
    Flags: USHORT,
    Buffer: PCHAR,
    BufferLength: UINT,
    Addr: PTDI_CONNECTION_INFORMATION,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Mdl: PMDL = core::ptr::null_mut();

    if TransportObject.is_null() {
        DPRINT!("Bad tranport object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DeviceObject = IoGetRelatedDeviceObject(TransportObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_RECEIVE_DATAGRAM,
        DeviceObject,
        TransportObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if (*Irp).is_null() {
        DPRINT!("Insufficient resources.\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("Allocating irp for %p:%u\n", Buffer, BufferLength);

    // Virtual address, length of buffer, not secondary, don't charge quota, don't use IRP
    Mdl = IoAllocateMdl(
        Buffer as PVOID,
        BufferLength as ULONG,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    );
    if Mdl.is_null() {
        DPRINT!("Insufficient resources.\n");
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // _SEH2_TRY/_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) as a real handler (ntkrnl_sys::seh);
    // the C's except arm is dead under the kernel's DUMMY_PSEH.
    let mut __seh = ();
    if ntkrnl_sys::seh::try_except(
        &mut __seh,
        |_| {
            MmProbeAndLockPages(Mdl, (**Irp).RequestorMode, IoModifyAccess);
        },
        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    )
    .is_err()
    {
        DPRINT!("MmProbeAndLockPages() failed.\n");
        IoFreeMdl(Mdl);
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("AFD>>> Got an MDL: %p\n", Mdl);

    TdiBuildReceiveDatagram!(
        *Irp,
        DeviceObject,
        TransportObject,
        CompletionRoutine,
        CompletionContext,
        Mdl,
        BufferLength,
        Addr,
        Addr,
        Flags,
    );

    TdiCall(
        *Irp,
        DeviceObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    // Does not block... The MDL is deleted in the receive completion routine.

    return STATUS_PENDING;
}
#[no_mangle]
pub unsafe extern "C" fn TdiSendDatagram(
    Irp: *mut PIRP,
    TransportObject: PFILE_OBJECT,
    Buffer: PCHAR,
    BufferLength: UINT,
    Addr: PTDI_CONNECTION_INFORMATION,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Mdl: PMDL = core::ptr::null_mut();

    if TransportObject.is_null() {
        DPRINT!("Bad transport object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DPRINT!("Called(TransportObject %p)\n", TransportObject);

    DeviceObject = IoGetRelatedDeviceObject(TransportObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    if BufferLength == 0 {
        DPRINT!("Succeeding send with length 0.\n");
        return STATUS_SUCCESS;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_SEND_DATAGRAM,
        DeviceObject,
        TransportObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if (*Irp).is_null() {
        DPRINT!("Insufficient resources.\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("Allocating irp for %p:%u\n", Buffer, BufferLength);

    Mdl = IoAllocateMdl(
        Buffer as PVOID,
        BufferLength as ULONG,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    );

    if Mdl.is_null() {
        DPRINT!("Insufficient resources.\n");
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // _SEH2_TRY/_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) as a real handler (ntkrnl_sys::seh);
    // the C's except arm is dead under the kernel's DUMMY_PSEH.
    let mut __seh = ();
    if ntkrnl_sys::seh::try_except(
        &mut __seh,
        |_| {
            MmProbeAndLockPages(Mdl, (**Irp).RequestorMode, IoReadAccess);
        },
        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    )
    .is_err()
    {
        DPRINT!("MmProbeAndLockPages() failed.\n");
        IoFreeMdl(Mdl);
        IoCompleteRequest(*Irp, IO_NO_INCREMENT);
        *Irp = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DPRINT!("AFD>>> Got an MDL: %p\n", Mdl);

    TdiBuildSendDatagram!(
        *Irp,
        DeviceObject,
        TransportObject,
        CompletionRoutine,
        CompletionContext,
        Mdl,
        BufferLength,
        Addr,
    );

    /* Does not block...  The MDL is deleted in the send completion
       routine. */
    return TdiCall(
        *Irp,
        DeviceObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn TdiDisconnect(
    Irp: *mut PIRP,
    TransportObject: PFILE_OBJECT,
    Time: PLARGE_INTEGER,
    Flags: USHORT,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    CompletionContext: PVOID,
    RequestConnectionInfo: PTDI_CONNECTION_INFORMATION,
    ReturnConnectionInfo: PTDI_CONNECTION_INFORMATION,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;

    if TransportObject.is_null() {
        DPRINT!("Bad transport object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    DPRINT!("Called(TransportObject %p)\n", TransportObject);

    DeviceObject = IoGetRelatedDeviceObject(TransportObject);
    if DeviceObject.is_null() {
        DPRINT!("Bad device object.\n");
        return STATUS_INVALID_PARAMETER;
    }

    *Irp = TdiBuildInternalDeviceControlIrp!(
        TDI_DISCONNECT,
        DeviceObject,
        TransportObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if (*Irp).is_null() {
        DPRINT!("Insufficient resources.\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    TdiBuildDisconnect!(
        *Irp,
        DeviceObject,
        TransportObject,
        CompletionRoutine,
        CompletionContext,
        Time,
        Flags,
        RequestConnectionInfo,
        ReturnConnectionInfo,
    );

    TdiCall(*Irp, DeviceObject, core::ptr::null_mut(), core::ptr::null_mut());

    return STATUS_PENDING;
}
