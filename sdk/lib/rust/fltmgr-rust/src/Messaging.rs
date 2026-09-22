use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, ExAcquireFastMutex, ExGetPreviousMode, ExInitializeRundownProtection, ExReleaseFastMutex, _GENERIC_MAPPING, IRP, IoCreateDevice, IoCreateSymbolicLink, IoCsqInitializeEx, IoDeleteDevice, KeInitializeEvent, KeInitializeSemaphore, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, OBJECT_TYPE_INITIALIZER, ObCreateObject, ObCreateObjectType, ObInsertObject, ObMakeTemporaryObject, ObReferenceObjectByName, PDEVICE_OBJECT, PDRIVER_OBJECT, PFAST_MUTEX, PFILE_OBJECT, PHANDLE, PIO_CSQ, PIO_STACK_LOCATION, PIRP, PKIRQL, PLARGE_INTEGER, POBJECT_ATTRIBUTES, POBJECT_TYPE, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwClose, _OBJECT_TYPE, KPROCESSOR_MODE, _UNICODE_STRING, _DEVICE_OBJECT, EX_PUSH_LOCK, KPRIORITY};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;
// fltmgr-internal pushlock fns are defined #[no_mangle] in Object.rs; bring the
// names into scope here (their extern decls were removed in favor of the defs).
use crate::Object::{FltAcquirePushLockShared, FltReleasePushLock};

// All four are defined in crate::Object — import the real defs so ABI matches
// (FltObjectReference/Dereference are spec stdcall = extern "system"; the Fltp*
// pointer helpers are crate-internal cdecl). RtlAssert is the crate::glue no-op.
use crate::Object::{
    FltObjectReference, FltObjectDereference, FltpObjectPointerReference, FltpObjectPointerDereference,
};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut CommsDeviceName: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: ::core::ptr::null_mut::<WCHAR>(),
};

pub static mut CommsDeviceObject: PDEVICE_OBJECT =
    ::core::ptr::null::<_DEVICE_OBJECT>() as *mut _DEVICE_OBJECT;

pub static mut ServerPortObjectType: POBJECT_TYPE =
    ::core::ptr::null::<_OBJECT_TYPE>() as *mut _OBJECT_TYPE;

pub static mut ClientPortObjectType: POBJECT_TYPE =
    ::core::ptr::null::<_OBJECT_TYPE>() as *mut _OBJECT_TYPE;

#[no_mangle]
pub unsafe extern "system" fn FltCreateCommunicationPort(
    Filter: PFLT_FILTER,
    ServerPort: *mut PFLT_PORT,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ServerPortCookie: PVOID,
    ConnectNotifyCallback: PFLT_CONNECT_NOTIFY,
    DisconnectNotifyCallback: PFLT_DISCONNECT_NOTIFY,
    MessageNotifyCallback: PFLT_MESSAGE_NOTIFY,
    MaxConnections: LONG,
) -> NTSTATUS {

    let mut PortObject: PFLT_SERVER_PORT_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    /* The caller must allow at least one connection */
    if MaxConnections == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    /* The request must be for a kernel handle */
    if (*ObjectAttributes).Attributes as u32 & OBJ_KERNEL_HANDLE as u32 == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    /*
     * Get rundown protection on the target to stop the owner
     * from unloading whilst this port object is open.
     */
    Status = FltObjectReference(Filter as PVOID);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Create the server port object for this filter */
    Status = ObCreateObject(
        0i32 as i32 as KPROCESSOR_MODE,
        ServerPortObjectType,
        ObjectAttributes,
        0i32 as i32 as KPROCESSOR_MODE,
        core::ptr::null_mut(),
        core::mem::size_of::<FLT_SERVER_PORT_OBJECT>() as ULONG,
        0 as ULONG,
        0 as ULONG,
        core::ptr::addr_of_mut!(PortObject) as *mut PVOID,
    ) as NTSTATUS;
    if NT_SUCCESS(Status) {
        /* Zero out the struct */
        RtlZeroMemory(
            PortObject as PVOID,
            core::mem::size_of::<FLT_SERVER_PORT_OBJECT>(),
        );

        /* Increment the ref count on the target filter */
        FltpObjectPointerReference(Filter as PFLT_OBJECT);

        /* Setup the filter port object */
        (*PortObject).Filter = Filter;
        (*PortObject).ConnectNotify = ConnectNotifyCallback;
        (*PortObject).DisconnectNotify = DisconnectNotifyCallback;
        (*PortObject).MessageNotify = MessageNotifyCallback;
        (*PortObject).Cookie = ServerPortCookie;
        (*PortObject).MaxConnections = MaxConnections;

        /* Insert the object */
        Status = ObInsertObject(
            PortObject as PVOID,
            core::ptr::null_mut(),
            (STANDARD_RIGHTS_ALL | FILE_READ_DATA as u32) as ACCESS_MASK,
            0 as ULONG,
            core::ptr::null_mut(),
            ServerPort as PHANDLE,
        ) as NTSTATUS;
        if NT_SUCCESS(Status) {
            /* Lock the connection list */
            ExAcquireFastMutex(core::ptr::addr_of_mut!((*Filter).ConnectionList.mLock));

            /* Add the new port object to the connection list and increment the count */
            InsertTailList(
                core::ptr::addr_of_mut!((*Filter).ConnectionList.mList),
                core::ptr::addr_of_mut!((*PortObject).FilterLink),
            );
            (*Filter).ConnectionList.mCount = (*Filter).ConnectionList.mCount.wrapping_add(1);

            /* Unlock the connection list */
            ExReleaseFastMutex(core::ptr::addr_of_mut!((*Filter).ConnectionList.mLock));
        }
    }

    if !NT_SUCCESS(Status) {
        /* Allow the filter to be cleaned up */
        FltObjectDereference(Filter as PVOID);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FltCloseCommunicationPort(mut ServerPort: PFLT_PORT) {
    ZwClose(ServerPort as HANDLE);
}
#[no_mangle]
pub unsafe extern "system" fn FltCloseClientPort(
    mut Filter: PFLT_FILTER,
    mut ClientPort: *mut PFLT_PORT,
) {
    let mut Port: PFLT_PORT = core::ptr::null_mut();

    /* Protect against the handle being used whilst we're closing it */
    FltAcquirePushLockShared(core::ptr::addr_of_mut!((*Filter).PortLock));

    /* Store the port handle while we have the lock held */
    Port = *ClientPort;

    if !(*ClientPort).is_null() {
        /* Set the hadle to null */
        *ClientPort = core::ptr::null_mut();
    }

    /* Unlock the port */
    FltReleasePushLock(core::ptr::addr_of_mut!((*Filter).PortLock));

    if !Port.is_null() {
        /* Close the safe handle */
        ZwClose(Port as HANDLE);
    }
}
#[no_mangle]
pub unsafe extern "system" fn FltSendMessage(
    mut Filter: PFLT_FILTER,
    mut ClientPort: *mut PFLT_PORT,
    mut SenderBuffer: PVOID,
    mut SenderBufferLength: ULONG,
    mut ReplyBuffer: PVOID,
    mut ReplyLength: PULONG,
    mut Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let _ = Filter;
    let _ = ClientPort;
    let _ = SenderBuffer;
    let _ = SenderBufferLength;
    let _ = ReplyBuffer;
    let _ = ReplyLength;
    let _ = Timeout;
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn FltpMsgCreate(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {

    let mut StackPtr: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    /* Get the stack location */
    StackPtr = IoGetCurrentIrpStackLocation(Irp);

    if !((*StackPtr).MajorFunction as core::ffi::c_int == IRP_MJ_CREATE as core::ffi::c_int) {
        RtlAssert(
            b"StackPtr->MajorFunction == 0x00\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Messaging.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            232 as u32,
            core::ptr::null_mut::<i8>(),
        );
    } else {
    }

    /* Check if this is a caller wanting to connect */
    if (*StackPtr).MajorFunction as core::ffi::c_int == IRP_MJ_CREATE as core::ffi::c_int {
        /* Create the client port for this connection and exit */
        Status = CreateClientPort((*StackPtr).FileObject, Irp);
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status as NTSTATUS;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, 0);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FltpMsgDispatch(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;

    StackPtr = IoGetCurrentIrpStackLocation(Irp);

    if (*StackPtr).MajorFunction as i32 == IRP_MJ_CLOSE as i32 {
        Status = CloseClientPort((*StackPtr).FileObject, Irp);
    } else {
        Status = STATUS_NOT_IMPLEMENTED;
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, 0);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FltpServerPortClose(
    mut Process: PEPROCESS,
    mut Object: PVOID,
    mut GrantedAccess: ACCESS_MASK,
    mut ProcessHandleCount: ULONG_PTR,
    mut SystemHandleCount: ULONG_PTR,
) {

    let mut PortObject: PFLT_SERVER_PORT_OBJECT;
    let mut Lock: PFAST_MUTEX;

    /* Cast the object to a server port object */
    PortObject = Object as PFLT_SERVER_PORT_OBJECT;

    /* Lock the connection list */
    Lock = core::ptr::addr_of_mut!((*(*PortObject).Filter).ConnectionList.mLock) as PFAST_MUTEX;
    ExAcquireFastMutex(Lock);

    /* Remove the server port object from the list */
    RemoveEntryList(core::ptr::addr_of_mut!((*PortObject).FilterLink));

    /* Unlock the connection list */
    ExReleaseFastMutex(Lock);

    /* Remove the rundown protection we added to stop the owner from tearing us down */
    FltObjectDereference((*PortObject).Filter as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn FltpServerPortDelete(Object: PVOID) {
    let PortObject: PFLT_SERVER_PORT_OBJECT = Object as PFLT_SERVER_PORT_OBJECT;
    FltpObjectPointerDereference((*PortObject).Filter as PFLT_OBJECT);
}

#[no_mangle]
pub unsafe extern "system" fn FltpClientPortClose(
    mut Process: PEPROCESS,
    mut Object: PVOID,
    mut GrantedAccess: ACCESS_MASK,
    mut ProcessHandleCount: ULONG_PTR,
    mut SystemHandleCount: ULONG_PTR,
) {
    let mut PortObject: PFLT_PORT_OBJECT = Object as PFLT_PORT_OBJECT;
    if FltpDisconnectPort(PortObject) != 0 {
        InterlockedDecrement(
            core::ptr::addr_of_mut!((*(*PortObject).ServerPort).NumberOfConnections) as *mut core::ffi::c_long,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn FltpDisconnectPort(mut PortObject: PFLT_PORT_OBJECT) -> BOOLEAN {
    let mut Disconnected: BOOLEAN = 0 as BOOLEAN;

    /* Lock the port object while we disconnect it */
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*PortObject).Lock));

    /* Make sure we have a valid connection */
    if (*PortObject).Disconnected as core::ffi::c_int == 0 {
        /* Let any waiters know we're dusconnecing */
        KeSetEvent(
            core::ptr::addr_of_mut!((*PortObject).DisconnectEvent) as PVOID,
            0 as KPRIORITY,
            0 as BOOLEAN,
        );

        // cleanup everything in the message queue (PortObject->MsgQ.Csq)

        /* Set the disconnected state to true */
        (*PortObject).Disconnected = 1 as BOOLEAN;
        Disconnected = 1 as BOOLEAN;
    }

    /* Unlock and exit*/
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*PortObject).Lock));
    return Disconnected;
}

#[no_mangle]
pub unsafe extern "system" fn FltpClientPortDelete(Object: PVOID) {
    let PortObject: PFLT_PORT_OBJECT = Object as PFLT_PORT_OBJECT;
    ObDereferenceObject((*PortObject).ServerPort as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn FltpSetupCommunicationObjects(mut DriverObject: PDRIVER_OBJECT) -> NTSTATUS {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut SymLinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    let mut Mapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
        GenericRead: STANDARD_RIGHTS_READ as ACCESS_MASK,
        GenericWrite: STANDARD_RIGHTS_WRITE as ACCESS_MASK,
        GenericExecute: (STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE) as ACCESS_MASK,
        GenericAll: FLT_PORT_ALL_ACCESS as ACCESS_MASK,
    };

    /* Create the server comms object type */
    RtlZeroMemory(
        core::ptr::addr_of_mut!(ObjectTypeInitializer) as *mut core::ffi::c_void,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Name),
        core::mem::transmute::<[u8; 42], [u16; 21]>(
            *b"F\0i\0l\0t\0e\0r\0C\0o\0n\0n\0e\0c\0t\0i\0o\0n\0P\0o\0r\0t\0\0\0",
        )
        .as_ptr(),
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK as u32;
    ObjectTypeInitializer.GenericMapping = Mapping;
    ObjectTypeInitializer.PoolType = NonPagedPool;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge =
        core::mem::size_of::<FLT_SERVER_PORT_OBJECT>() as u32;
    ObjectTypeInitializer.ValidAccessMask = GENERIC_ALL as u32;
    ObjectTypeInitializer.CloseProcedure = core::mem::transmute(FltpServerPortClose as unsafe extern "system" fn(_, _, _, _, _));
    ObjectTypeInitializer.DeleteProcedure = core::mem::transmute(FltpServerPortDelete as unsafe extern "system" fn(_));
    Status = ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ServerPortObjectType),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Create the client comms object type */
    RtlZeroMemory(
        core::ptr::addr_of_mut!(ObjectTypeInitializer) as *mut core::ffi::c_void,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Name),
        core::mem::transmute::<[u8; 48], [u16; 24]>(
            *b"F\0i\0l\0t\0e\0r\0C\0o\0m\0m\0u\0n\0i\0c\0a\0t\0i\0o\0n\0P\0o\0r\0t\0\0\0",
        )
        .as_ptr(),
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK as u32;
    ObjectTypeInitializer.GenericMapping = Mapping;
    ObjectTypeInitializer.PoolType = NonPagedPool;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge =
        core::mem::size_of::<FLT_PORT_OBJECT>() as u32;
    ObjectTypeInitializer.ValidAccessMask = GENERIC_ALL as u32;
    ObjectTypeInitializer.CloseProcedure = core::mem::transmute(FltpClientPortClose as unsafe extern "system" fn(_, _, _, _, _));
    ObjectTypeInitializer.DeleteProcedure = core::mem::transmute(FltpClientPortDelete as unsafe extern "system" fn(_));
    Status = ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ClientPortObjectType),
    );
    if !NT_SUCCESS(Status) {
        // goto Quit
    } else {
        /* Create the device object */
        Status = IoCreateDevice(
            DriverObject,
            0,
            core::ptr::addr_of_mut!(CommsDeviceName),
            FILE_DEVICE_WPD,
            0,
            0,
            core::ptr::addr_of_mut!(CommsDeviceObject),
        );
        if NT_SUCCESS(Status) {
            /* Setup a symbolic link for the device */
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(SymLinkName),
                core::mem::transmute::<[u8; 44], [u16; 22]>(
                    *b"\\\0D\0o\0s\0D\0e\0v\0i\0c\0e\0s\0\\\0F\0l\0t\0M\0g\0r\0M\0s\0g\0\0\0",
                )
                .as_ptr(),
            );
            Status = IoCreateSymbolicLink(
                core::ptr::addr_of_mut!(SymLinkName),
                core::ptr::addr_of_mut!(CommsDeviceName),
            );
        }
    }

    // Quit:
    if !NT_SUCCESS(Status) {
        /* Something went wrong, undo */
        if !CommsDeviceObject.is_null() {
            IoDeleteDevice(CommsDeviceObject);
            CommsDeviceObject = core::ptr::null_mut();
        }
        if !ClientPortObjectType.is_null() {
            ObMakeTemporaryObject(ClientPortObjectType as *mut core::ffi::c_void);
            ObDereferenceObject(ClientPortObjectType as *mut core::ffi::c_void);
            ClientPortObjectType = core::ptr::null_mut();
        }

        if !ServerPortObjectType.is_null() {
            ObMakeTemporaryObject(ServerPortObjectType as *mut core::ffi::c_void);
            ObDereferenceObject(ServerPortObjectType as *mut core::ffi::c_void);
            ServerPortObjectType = core::ptr::null_mut();
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FltpAddMessageWaiter(
    Csq: PIO_CSQ,
    Irp: PIRP,
    InsertContext: PVOID,
) -> NTSTATUS {
    let mut MessageWaiterQueue: PFLT_MESSAGE_WAITER_QUEUE;

    /* Get the start of the waiter queue struct */
    MessageWaiterQueue = (Csq as usize)
        .wrapping_sub(core::mem::offset_of!(FLT_MESSAGE_WAITER_QUEUE, Csq))
        as *mut FLT_MESSAGE_WAITER_QUEUE;

    /* Insert the IRP at the end of the queue */
    InsertTailList(
        core::ptr::addr_of_mut!((*MessageWaiterQueue).WaiterQ.mList),
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
    );

    /* return success */
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn FltpRemoveMessageWaiter(mut Csq: PIO_CSQ, mut Irp: PIRP) {
    /* Remove the IRP from the queue */
    RemoveEntryList(core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
}

#[no_mangle]
pub unsafe extern "system" fn FltpGetNextMessageWaiter(
    Csq: PIO_CSQ,
    Irp: PIRP,
    PeekContext: PVOID,
) -> PIRP {
    let MessageWaiterQueue: PFLT_MESSAGE_WAITER_QUEUE;
    let mut NextIrp: PIRP = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;
    let mut IrpStack: PIO_STACK_LOCATION;

    /* Get the start of the waiter queue struct */
    MessageWaiterQueue = (Csq as usize
        - core::mem::offset_of!(FLT_MESSAGE_WAITER_QUEUE, Csq))
        as *mut FLT_MESSAGE_WAITER_QUEUE;

    /* Is the IRP valid? */
    if Irp.is_null() {
        /* Start peeking from the listhead */
        NextEntry = (*MessageWaiterQueue).WaiterQ.mList.Flink;
    } else {
        /* Start peeking from that IRP onwards */
        NextEntry = (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry.Flink;
    }

    /* Loop through the queue */
    while NextEntry != core::ptr::addr_of_mut!((*MessageWaiterQueue).WaiterQ.mList) {
        /* Store the next IRP in the list */
        NextIrp = (NextEntry as usize
            - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
            as *mut IRP;

        /* Did we supply a PeekContext on insert? */
        if PeekContext.is_null() {
            /* We already have the next IRP */
            break;
        } else {
            /* Get the stack of the next IRP */
            IrpStack = IoGetCurrentIrpStackLocation(NextIrp);

            /* Does the PeekContext match the object? */
            if (*IrpStack).FileObject == PeekContext as PFILE_OBJECT {
                /* We have a match */
                break;
            }

            /* Move to the next IRP */
            NextIrp = core::ptr::null_mut();
            NextEntry = (*NextEntry).Flink;
        }
    }

    return NextIrp;
}
#[no_mangle]
pub unsafe extern "system" fn FltpAcquireMessageWaiterLock(mut Csq: PIO_CSQ, mut Irql: PKIRQL) {
    let mut MessageWaiterQueue: PFLT_MESSAGE_WAITER_QUEUE = core::ptr::null_mut();

    let _ = Irql;

    /* Get the start of the waiter queue struct */
    MessageWaiterQueue = (Csq as usize).wrapping_sub(
        core::mem::offset_of!(FLT_MESSAGE_WAITER_QUEUE, Csq),
    ) as *mut FLT_MESSAGE_WAITER_QUEUE;

    /* Acquire the IRP queue lock */
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*MessageWaiterQueue).WaiterQ.mLock));
}
#[no_mangle]
pub unsafe extern "system" fn FltpReleaseMessageWaiterLock(mut Csq: PIO_CSQ, mut Irql: KIRQL) {
    let mut MessageWaiterQueue: PFLT_MESSAGE_WAITER_QUEUE = core::ptr::null_mut();

    let _ = Irql;

    /* Get the start of the waiter queue struct */
    MessageWaiterQueue = (Csq as usize)
        .wrapping_sub(core::mem::offset_of!(FLT_MESSAGE_WAITER_QUEUE, Csq))
        as *mut FLT_MESSAGE_WAITER_QUEUE;

    /* Release the IRP queue lock */
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*MessageWaiterQueue).WaiterQ.mLock));
}

#[no_mangle]
pub unsafe extern "system" fn FltpCancelMessageWaiter(Csq: PIO_CSQ, Irp: PIRP) {
    /* Cancel the IRP */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}
unsafe fn CreateClientPort(mut FileObject: PFILE_OBJECT, mut Irp: PIRP) -> NTSTATUS {
    const FLT_PORT_ALL_ACCESS: u32 = STANDARD_RIGHTS_ALL | 0x1ff;
    const STATUS_CONNECTION_COUNT_LIMIT: NTSTATUS = 0xC0000246u32 as NTSTATUS;

    let mut ServerPortObject: PFLT_SERVER_PORT_OBJECT = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FilterPortData: PFILTER_PORT_DATA;
    let mut ClientPortObject: PFLT_PORT_OBJECT = core::ptr::null_mut();
    let mut PortHandle: PFLT_PORT = core::ptr::null_mut();
    let mut PortCCB: PPORT_CCB = core::ptr::null_mut();
    let mut NumConns: LONG;
    let mut Status: NTSTATUS;

    /* We received the buffer via FilterConnectCommunicationPort, cast it back to its original form */
    FilterPortData = (*Irp).AssociatedIrp.SystemBuffer as PFILTER_PORT_DATA;

    /* Get a reference to the server port the filter created */
    Status = ObReferenceObjectByName(
        core::ptr::addr_of_mut!((*FilterPortData).PortName),
        0,
        core::ptr::null_mut(),
        FLT_PORT_ALL_ACCESS as ACCESS_MASK,
        ServerPortObjectType,
        ExGetPreviousMode(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ServerPortObject) as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Increment the number of connections on the server port */
    NumConns = InterlockedIncrement(
        core::ptr::addr_of_mut!((*ServerPortObject).NumberOfConnections),
    );
    if NumConns > (*ServerPortObject).MaxConnections {
        Status = STATUS_CONNECTION_COUNT_LIMIT;
    } else {
        /* Initialize a basic kernel handle request */
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        /* Now create the new client port object */
        Status = ObCreateObject(
            0i32 as KPROCESSOR_MODE,
            ClientPortObjectType,
            core::ptr::addr_of_mut!(ObjectAttributes),
            0i32 as KPROCESSOR_MODE,
            core::ptr::null_mut(),
            core::mem::size_of::<FLT_PORT_OBJECT>() as ULONG,
            0,
            0,
            core::ptr::addr_of_mut!(ClientPortObject) as *mut PVOID,
        );
        if NT_SUCCESS(Status) {
            /* Clear out the buffer */
            RtlZeroMemory(
                ClientPortObject as PVOID,
                core::mem::size_of::<FLT_PORT_OBJECT>(),
            );

            /* Initialize the locks */
            ExInitializeRundownProtection(core::ptr::addr_of_mut!((*ClientPortObject).MsgNotifRundownRef));
            ExInitializeFastMutex(core::ptr::addr_of_mut!((*ClientPortObject).Lock));

            /* Set the server port object this belongs to */
            (*ClientPortObject).ServerPort = ServerPortObject;

            /* Setup the message queue */
            Status = InitializeMessageWaiterQueue(core::ptr::addr_of_mut!((*ClientPortObject).MsgQ));
            if NT_SUCCESS(Status) {
                /* Create the CCB which we'll attach to the file object */
                PortCCB = CreatePortCCB(ClientPortObject);
                if PortCCB.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    /* Now insert the new client port into the object manager */
                    Status = ObInsertObject(
                        ClientPortObject as PVOID,
                        core::ptr::null_mut(),
                        FLT_PORT_ALL_ACCESS as ACCESS_MASK,
                        1,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(PortHandle) as PHANDLE,
                    );
                    if !NT_SUCCESS(Status) {
                        /* ObInsertObject() failed and already dereferenced ClientPortObject */
                        ClientPortObject = core::ptr::null_mut();
                    } else {
                        /* Add a reference to the filter to keep it alive while we do some work with it */
                        Status = FltObjectReference((*ServerPortObject).Filter as PVOID);
                        if NT_SUCCESS(Status) {
                            /* Invoke the callback to let the filter know we have a connection */
                            Status = ((*ServerPortObject).ConnectNotify.expect("non-null function pointer"))(
                                PortHandle,
                                (*ServerPortObject).Cookie,
                                core::ptr::null_mut(),
                                0,
                                core::ptr::addr_of_mut!((*ClientPortObject).Cookie),
                            );
                            if NT_SUCCESS(Status) {
                                /* Add the client port CCB to the file object */
                                (*FileObject).FsContext2 = PortCCB as PVOID;

                                /* Lock the port list on the filter and add this new port object to the list */
                                ExAcquireFastMutex(core::ptr::addr_of_mut!((*(*ServerPortObject).Filter).PortList.mLock));
                                InsertTailList(
                                    core::ptr::addr_of_mut!((*(*ServerPortObject).Filter).PortList.mList),
                                    core::ptr::addr_of_mut!((*ClientPortObject).FilterLink),
                                );
                                ExReleaseFastMutex(core::ptr::addr_of_mut!((*(*ServerPortObject).Filter).PortList.mLock));
                            }

                            /* We're done with the filter object, decremement the count */
                            FltObjectDereference((*ServerPortObject).Filter as PVOID);
                        }
                    }
                }
            }
        }
    }

    /* Quit: */
    if !NT_SUCCESS(Status) {
        if !ClientPortObject.is_null() {
            ObDereferenceObject(ClientPortObject as PVOID);
        }

        if !PortHandle.is_null() {
            ZwClose(PortHandle as HANDLE);
        } else if !ServerPortObject.is_null() {
            InterlockedDecrement(core::ptr::addr_of_mut!((*ServerPortObject).NumberOfConnections));
            ObDereferenceObject(ServerPortObject as PVOID);
        }

        if !PortCCB.is_null() {
            ExFreePoolWithTag(PortCCB as PVOID, FM_TAG_CCB);
        }
    }

    return Status;
}

unsafe fn CloseClientPort(mut FileObject: PFILE_OBJECT, mut Irp: PIRP) -> NTSTATUS {
    let mut Ccb: PFLT_CCB = core::ptr::null_mut();

    Ccb = (*FileObject).FsContext2 as PFLT_CCB;

    /* Remove the reference on the filter we added when we opened the port */
    ObDereferenceObject((*Ccb).Data.Port.as_ref().Port as PVOID);

    // FIXME: Free the CCB

    return STATUS_SUCCESS;
}
unsafe fn InitializeMessageWaiterQueue(
    MsgWaiterQueue: PFLT_MESSAGE_WAITER_QUEUE,
) -> NTSTATUS {
    const MAXLONG: i32 = 0x7fffffff;

    let mut Status: NTSTATUS;

    Status = IoCsqInitializeEx(
        core::ptr::addr_of_mut!((*MsgWaiterQueue).Csq),
        core::mem::transmute(FltpAddMessageWaiter as unsafe extern "system" fn(_, _, _) -> _),
        core::mem::transmute(FltpRemoveMessageWaiter as unsafe extern "system" fn(_, _)),
        core::mem::transmute(FltpGetNextMessageWaiter as unsafe extern "system" fn(_, _, _) -> _),
        core::mem::transmute(FltpAcquireMessageWaiterLock as unsafe extern "system" fn(_, _)),
        core::mem::transmute(FltpReleaseMessageWaiterLock as unsafe extern "system" fn(_, _)),
        core::mem::transmute(FltpCancelMessageWaiter as unsafe extern "system" fn(_, _)),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ExInitializeFastMutex(core::ptr::addr_of_mut!((*MsgWaiterQueue).WaiterQ.mLock));
    InitializeListHead(core::ptr::addr_of_mut!((*MsgWaiterQueue).WaiterQ.mList));
    (*MsgWaiterQueue).WaiterQ.mCount = 0;

    (*MsgWaiterQueue).MinimumWaiterLength = -(1 as i32) as u32;

    KeInitializeSemaphore(
        core::ptr::addr_of_mut!((*MsgWaiterQueue).Semaphore),
        0,
        MAXLONG,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*MsgWaiterQueue).Event),
        _EVENT_TYPE_NotificationEvent,
        0,
    );

    return STATUS_SUCCESS;
}

const FM_TAG_CCB: u32 = 0x62434d46; // 'FMCb'
unsafe fn CreatePortCCB(mut PortObject: PFLT_PORT_OBJECT) -> PPORT_CCB {
    let mut PortCCB: PPORT_CCB;

    /* Allocate a CCB struct to hold the client port object info */
    PortCCB = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<PORT_CCB>() as SIZE_T,
        FM_TAG_CCB as ULONG,
    ) as PPORT_CCB;
    if !PortCCB.is_null() {
        /* Initialize the structure */
        (*PortCCB).Port = PortObject;
        (*PortCCB).ReplyWaiterList.mCount = 0 as ULONG;
        ExInitializeFastMutex(core::ptr::addr_of_mut!((*PortCCB).ReplyWaiterList.mLock));
        KeInitializeEvent(
            core::ptr::addr_of_mut!((*PortCCB).ReplyWaiterList.mLock.Event),
            _EVENT_TYPE_SynchronizationEvent,
            0 as BOOLEAN,
        );
    }

    return PortCCB;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         Filesystem Filter Manager
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            drivers/filters/fltmgr/Messaging.c
// * PURPOSE:         Contains the routines to handle the comms layer
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
// #include <fltmgr_shared.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
// UNICODE_STRING CommsDeviceName = RTL_CONSTANT_STRING(L"\\FileSystem\\Filters\\FltMgrMsg");
// PDEVICE_OBJECT CommsDeviceObject;
//
// POBJECT_TYPE ServerPortObjectType;
// POBJECT_TYPE ClientPortObjectType;
//
// static
// BOOLEAN
// FltpDisconnectPort(
//      PFLT_PORT_OBJECT PortObject
// );
//
// static
// NTSTATUS
// CreateClientPort(
//      PFILE_OBJECT FileObject,
//      PIRP Irp
// );
//
// static
// NTSTATUS
// CloseClientPort(
//      PFILE_OBJECT FileObject,
//      PIRP Irp
// );
//
// static
// NTSTATUS
// InitializeMessageWaiterQueue(
//      PFLT_MESSAGE_WAITER_QUEUE MsgWaiterQueue
// );
//
// static
// PPORT_CCB
// CreatePortCCB(
//      PFLT_PORT_OBJECT PortObject
// );
//
//
//
// /* EXPORTED FUNCTIONS ******************************************************/
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// FLTAPI
// FltCreateCommunicationPort( PFLT_FILTER Filter,
//                             PFLT_PORT *ServerPort,
//                             POBJECT_ATTRIBUTES ObjectAttributes,
//                             PVOID ServerPortCookie,
//                             PFLT_CONNECT_NOTIFY ConnectNotifyCallback,
//                             PFLT_DISCONNECT_NOTIFY DisconnectNotifyCallback,
//                             PFLT_MESSAGE_NOTIFY MessageNotifyCallback,
//                             LONG MaxConnections)
// {
//     PFLT_SERVER_PORT_OBJECT PortObject;
//     NTSTATUS Status;
//
//     /* The caller must allow at least one connection */
//     if (MaxConnections == 0)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* The request must be for a kernel handle */
//     if (!(ObjectAttributes->Attributes & OBJ_KERNEL_HANDLE))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /*
//      * Get rundown protection on the target to stop the owner
//      * from unloading whilst this port object is open. It gets
//      * removed in the FltpServerPortClose callback
//      */
//     Status = FltObjectReference(Filter);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Create the server port object for this filter */
//     Status = ObCreateObject(KernelMode,
//                             ServerPortObjectType,
//                             ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             sizeof(FLT_SERVER_PORT_OBJECT),
//                             0,
//                             0,
//                             (PVOID *)&PortObject);
//     if (NT_SUCCESS(Status))
//     {
//         /* Zero out the struct */
//         RtlZeroMemory(PortObject, sizeof(FLT_SERVER_PORT_OBJECT));
//
//         /* Increment the ref count on the target filter */
//         FltpObjectPointerReference((PFLT_OBJECT)Filter);
//
//         /* Setup the filter port object */
//         PortObject->Filter = Filter;
//         PortObject->ConnectNotify = ConnectNotifyCallback;
//         PortObject->DisconnectNotify = DisconnectNotifyCallback;
//         PortObject->MessageNotify = MessageNotifyCallback;
//         PortObject->Cookie = ServerPortCookie;
//         PortObject->MaxConnections = MaxConnections;
//
//         /* Insert the object */
//         Status = ObInsertObject(PortObject,
//                                 NULL,
//                                 STANDARD_RIGHTS_ALL | FILE_READ_DATA,
//                                 0,
//                                 NULL,
//                                 (PHANDLE)ServerPort);
//         if (NT_SUCCESS(Status))
//         {
//             /* Lock the connection list */
//             ExAcquireFastMutex(&Filter->ConnectionList.mLock);
//
//             /* Add the new port object to the connection list and increment the count */
//             InsertTailList(&Filter->ConnectionList.mList, &PortObject->FilterLink);
//             Filter->ConnectionList.mCount++;
//
//             /* Unlock the connection list*/
//             ExReleaseFastMutex(&Filter->ConnectionList.mLock);
//         }
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Allow the filter to be cleaned up */
//         FltObjectDereference(Filter);
//     }
//
//     return Status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// VOID
// FLTAPI
// FltCloseCommunicationPort( PFLT_PORT ServerPort)
// {
//     /* Just close the handle to initiate the cleanup callbacks */
//     ZwClose(ServerPort);
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// VOID
// FLTAPI
// FltCloseClientPort( PFLT_FILTER Filter,
//                     PFLT_PORT *ClientPort)
// {
//     PFLT_PORT Port;
//
//     /* Protect against the handle being used whilst we're closing it */
//     FltAcquirePushLockShared(&Filter->PortLock);
//
//     /* Store the port handle while we have the lock held */
//     Port = *ClientPort;
//
//     if (*ClientPort)
//     {
//         /* Set the hadle to null */
//         *ClientPort = NULL;
//     }
//
//     /* Unlock the port */
//     FltReleasePushLock(&Filter->PortLock);
//
//     if (Port)
//     {
//         /* Close the safe handle */
//         ZwClose(Port);
//     }
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// FLTAPI
// FltSendMessage( PFLT_FILTER Filter,
//                 PFLT_PORT *ClientPort,
//                 PVOID SenderBuffer,
//                 ULONG SenderBufferLength,
//                 PVOID ReplyBuffer,
//                 PULONG ReplyLength,
//                 PLARGE_INTEGER Timeout)
// {
//     UNREFERENCED_PARAMETER(Filter);
//     UNREFERENCED_PARAMETER(ClientPort);
//     UNREFERENCED_PARAMETER(SenderBuffer);
//     UNREFERENCED_PARAMETER(SenderBufferLength);
//     UNREFERENCED_PARAMETER(ReplyBuffer);
//     UNREFERENCED_PARAMETER(ReplyLength);
//     UNREFERENCED_PARAMETER(Timeout);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* INTERNAL FUNCTIONS ******************************************************/
//
//
// NTSTATUS
// FltpMsgCreate( PDEVICE_OBJECT DeviceObject,
//                PIRP Irp)
// {
//     PIO_STACK_LOCATION StackPtr;
//     NTSTATUS Status;
//
//     /* Get the stack location */
//     StackPtr = IoGetCurrentIrpStackLocation(Irp);
//
//     FLT_ASSERT(StackPtr->MajorFunction == IRP_MJ_CREATE);
//
//     /* Check if this is a caller wanting to connect */
//     if (StackPtr->MajorFunction == IRP_MJ_CREATE)
//     {
//         /* Create the client port for this connection and exit */
//         Status = CreateClientPort(StackPtr->FileObject, Irp);
//     }
//     else
//     {
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     if (Status != STATUS_PENDING)
//     {
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, 0);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// FltpMsgDispatch( PDEVICE_OBJECT DeviceObject,
//                  PIRP Irp)
// {
//     PIO_STACK_LOCATION StackPtr;
//     NTSTATUS Status;
//
//     /* Get the stack location */
//     StackPtr = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Check if this is a caller wanting to connect */
//     if (StackPtr->MajorFunction == IRP_MJ_CLOSE)
//     {
//         /* Create the client port for this connection and exit */
//         Status = CloseClientPort(StackPtr->FileObject, Irp);
//     }
//     else
//     {
//         // We don't support anything else yet
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
//
//     if (Status != STATUS_PENDING)
//     {
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, 0);
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// FltpServerPortClose( PEPROCESS Process,
//                      PVOID Object,
//                      ACCESS_MASK GrantedAccess,
//                      ULONG ProcessHandleCount,
//                      ULONG SystemHandleCount)
// {
//     PFLT_SERVER_PORT_OBJECT PortObject;
//     PFAST_MUTEX Lock;
//
//     /* Cast the object to a server port object */
//     PortObject = (PFLT_SERVER_PORT_OBJECT)Object;
//
//     /* Lock the connection list */
//     Lock = &PortObject->Filter->ConnectionList.mLock;
//     ExAcquireFastMutex(Lock);
//
//     /* Remove the server port object from the list */
//     RemoveEntryList(&PortObject->FilterLink);
//
//     /* Unlock the connection list */
//     ExReleaseFastMutex(Lock);
//
//     /* Remove the rundown protection we added to stop the owner from tearing us down */
//     FltObjectDereference(PortObject->Filter);
// }
//
// VOID
// NTAPI
// FltpServerPortDelete(PVOID Object)
// {
//     /* Decrement the filter count we added in the create routine */
//     PFLT_SERVER_PORT_OBJECT PortObject = (PFLT_SERVER_PORT_OBJECT)Object;
//     FltpObjectPointerDereference((PFLT_OBJECT)PortObject->Filter);
//
// }
//
// VOID
// NTAPI
// FltpClientPortClose( PEPROCESS Process,
//                      PVOID Object,
//                      ACCESS_MASK GrantedAccess,
//                      ULONG ProcessHandleCount,
//                      ULONG SystemHandleCount)
// {
//     PFLT_PORT_OBJECT PortObject = (PFLT_PORT_OBJECT)Object;
//
//     if (FltpDisconnectPort(PortObject))
//     {
//         InterlockedDecrement(&PortObject->ServerPort->NumberOfConnections);
//     }
// }
//
// BOOLEAN
// FltpDisconnectPort( PFLT_PORT_OBJECT PortObject)
// {
//     BOOLEAN Disconnected = FALSE;
//
//     /* Lock the port object while we disconnect it */
//     ExAcquireFastMutex(&PortObject->Lock);
//
//     /* Make sure we have a valid connection */
//     if (PortObject->Disconnected == FALSE)
//     {
//         /* Let any waiters know we're dusconnecing */
//         KeSetEvent(&PortObject->DisconnectEvent, 0, 0);
//
//         // cleanup everything in the message queue (PortObject->MsgQ.Csq)
//
//         /* Set the disconnected state to true */
//         PortObject->Disconnected = TRUE;
//         Disconnected = TRUE;
//     }
//
//     /* Unlock and exit*/
//     ExReleaseFastMutex(&PortObject->Lock);
//     return Disconnected;
// }
//
// VOID
// NTAPI
// FltpClientPortDelete(PVOID Object)
// {
//     PFLT_PORT_OBJECT PortObject = (PFLT_PORT_OBJECT)Object;
//     ObDereferenceObject(PortObject->ServerPort);
// }
//
//
// NTSTATUS
// FltpSetupCommunicationObjects( PDRIVER_OBJECT DriverObject)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     UNICODE_STRING SymLinkName;
//     UNICODE_STRING Name;
//     NTSTATUS Status;
//
//     GENERIC_MAPPING Mapping =
//     {
//         STANDARD_RIGHTS_READ,
//         STANDARD_RIGHTS_WRITE,
//         STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//         FLT_PORT_ALL_ACCESS
//     };
//
//     /* Create the server comms object type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(OBJECT_TYPE_INITIALIZER));
//     RtlInitUnicodeString(&Name, L"FilterConnectionPort");
//     ObjectTypeInitializer.Length = sizeof(OBJECT_TYPE_INITIALIZER);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.GenericMapping = Mapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(FLT_SERVER_PORT_OBJECT);
//     ObjectTypeInitializer.ValidAccessMask = GENERIC_ALL;
//     ObjectTypeInitializer.CloseProcedure = FltpServerPortClose;
//     ObjectTypeInitializer.DeleteProcedure = FltpServerPortDelete;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ServerPortObjectType);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Create the client comms object type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(OBJECT_TYPE_INITIALIZER));
//     RtlInitUnicodeString(&Name, L"FilterCommunicationPort");
//     ObjectTypeInitializer.Length = sizeof(OBJECT_TYPE_INITIALIZER);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.GenericMapping = Mapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(FLT_PORT_OBJECT);
//     ObjectTypeInitializer.ValidAccessMask = GENERIC_ALL;
//     ObjectTypeInitializer.CloseProcedure = FltpClientPortClose;
//     ObjectTypeInitializer.DeleteProcedure = FltpClientPortDelete;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ClientPortObjectType);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Quit;
//     }
//
//     /* Create the device object */
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &CommsDeviceName,
//                             FILE_DEVICE_WPD,
//                             0,
//                             0,
//                             &CommsDeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         /* Setup a symbolic link for the device */
//         RtlInitUnicodeString(&SymLinkName, L"\\DosDevices\\FltMgrMsg");
//         Status = IoCreateSymbolicLink(&SymLinkName, &CommsDeviceName);
//     }
//
// Quit:
//     if (!NT_SUCCESS(Status))
//     {
//         /* Something went wrong, undo */
//         if (CommsDeviceObject)
//         {
//             IoDeleteDevice(CommsDeviceObject);
//             CommsDeviceObject = NULL;
//         }
//         if (ClientPortObjectType)
//         {
//             ObMakeTemporaryObject(ClientPortObjectType);
//             ObDereferenceObject(ClientPortObjectType);
//             ClientPortObjectType = NULL;
//         }
//
//         if (ServerPortObjectType)
//         {
//             ObMakeTemporaryObject(ServerPortObjectType);
//             ObDereferenceObject(ServerPortObjectType);
//             ServerPortObjectType = NULL;
//         }
//     }
//
//     return Status;
// }
//
// /* CSQ IRP CALLBACKS *******************************************************/
//
//
// NTSTATUS
// NTAPI
// FltpAddMessageWaiter( PIO_CSQ Csq,
//                       PIRP Irp,
//                       PVOID InsertContext)
// {
//     PFLT_MESSAGE_WAITER_QUEUE MessageWaiterQueue;
//
//     /* Get the start of the waiter queue struct */
//     MessageWaiterQueue = CONTAINING_RECORD(Csq,
//                                            FLT_MESSAGE_WAITER_QUEUE,
//                                            Csq);
//
//     /* Insert the IRP at the end of the queue */
//     InsertTailList(&MessageWaiterQueue->WaiterQ.mList,
//                    &Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//     /* return success */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// FltpRemoveMessageWaiter( PIO_CSQ Csq,
//                          PIRP Irp)
// {
//     /* Remove the IRP from the queue */
//     RemoveEntryList(&Irp->Tail.Overlay.__bindgen_anon_2.ListEntry);
// }
//
// PIRP
// NTAPI
// FltpGetNextMessageWaiter( PIO_CSQ Csq,
//                           PIRP Irp,
//                           PVOID PeekContext)
// {
//     PFLT_MESSAGE_WAITER_QUEUE MessageWaiterQueue;
//     PIRP NextIrp = NULL;
//     PLIST_ENTRY NextEntry;
//     PIO_STACK_LOCATION IrpStack;
//
//     /* Get the start of the waiter queue struct */
//     MessageWaiterQueue = CONTAINING_RECORD(Csq,
//                                            FLT_MESSAGE_WAITER_QUEUE,
//                                            Csq);
//
//     /* Is the IRP valid? */
//     if (Irp == NULL)
//     {
//         /* Start peeking from the listhead */
//         NextEntry = MessageWaiterQueue->WaiterQ.mList.Flink;
//     }
//     else
//     {
//         /* Start peeking from that IRP onwards */
//         NextEntry = Irp->Tail.Overlay.__bindgen_anon_2.ListEntry.Flink;
//     }
//
//     /* Loop through the queue */
//     while (NextEntry != &MessageWaiterQueue->WaiterQ.mList)
//     {
//         /* Store the next IRP in the list */
//         NextIrp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);
//
//         /* Did we supply a PeekContext on insert? */
//         if (!PeekContext)
//         {
//             /* We already have the next IRP */
//             break;
//         }
//         else
//         {
//             /* Get the stack of the next IRP */
//             IrpStack = IoGetCurrentIrpStackLocation(NextIrp);
//
//             /* Does the PeekContext match the object? */
//             if (IrpStack->FileObject == (PFILE_OBJECT)PeekContext)
//             {
//                 /* We have a match */
//                 break;
//             }
//
//             /* Move to the next IRP */
//             NextIrp = NULL;
//             NextEntry = NextEntry->Flink;
//         }
//     }
//
//     return NextIrp;
// }
//
// _Acquires_lock_(((PFLT_MESSAGE_WAITER_QUEUE)CONTAINING_RECORD(Csq, FLT_MESSAGE_WAITER_QUEUE, Csq))->WaiterQ.mLock)
// _IRQL_saves_global_(Irql, ((PFLT_MESSAGE_WAITER_QUEUE)CONTAINING_RECORD(Csq, DEVICE_EXTENSION, IrpQueue))->WaiterQ.mLock)
// _IRQL_raises_(DISPATCH_LEVEL)
// VOID
// NTAPI
// FltpAcquireMessageWaiterLock( PIO_CSQ Csq,
//                               PKIRQL Irql)
// {
//     PFLT_MESSAGE_WAITER_QUEUE MessageWaiterQueue;
//
//     UNREFERENCED_PARAMETER(Irql);
//
//     /* Get the start of the waiter queue struct */
//     MessageWaiterQueue = CONTAINING_RECORD(Csq,
//                                            FLT_MESSAGE_WAITER_QUEUE,
//                                            Csq);
//
//     /* Acquire the IRP queue lock */
//     ExAcquireFastMutex(&MessageWaiterQueue->WaiterQ.mLock);
// }
//
// _Releases_lock_(((PFLT_MESSAGE_WAITER_QUEUE)CONTAINING_RECORD(Csq, DEVICE_EXTENSION, IrpQueue))->WaiterQ.mLock)
// _IRQL_restores_global_(Irql, ((PFLT_MESSAGE_WAITER_QUEUE)CONTAINING_RECORD(Csq, DEVICE_EXTENSION, IrpQueue))->WaiterQ.mLock)
// _IRQL_requires_(DISPATCH_LEVEL)
// VOID
// NTAPI
// FltpReleaseMessageWaiterLock( PIO_CSQ Csq,
//                               KIRQL Irql)
// {
//     PFLT_MESSAGE_WAITER_QUEUE MessageWaiterQueue;
//
//     UNREFERENCED_PARAMETER(Irql);
//
//     /* Get the start of the waiter queue struct */
//     MessageWaiterQueue = CONTAINING_RECORD(Csq,
//                                            FLT_MESSAGE_WAITER_QUEUE,
//                                            Csq);
//
//     /* Release the IRP queue lock */
//     ExReleaseFastMutex(&MessageWaiterQueue->WaiterQ.mLock);
// }
//
// VOID
// NTAPI
// FltpCancelMessageWaiter( PIO_CSQ Csq,
//                          PIRP Irp)
// {
//     /* Cancel the IRP */
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
//
// /* PRIVATE FUNCTIONS ******************************************************/
//
// static
// NTSTATUS
// CreateClientPort( PFILE_OBJECT FileObject,
//                     PIRP Irp)
// {
//     PFLT_SERVER_PORT_OBJECT ServerPortObject = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PFILTER_PORT_DATA FilterPortData;
//     PFLT_PORT_OBJECT ClientPortObject = NULL;
//     PFLT_PORT PortHandle = NULL;
//     PPORT_CCB PortCCB = NULL;
//     //ULONG BufferLength;
//     LONG NumConns;
//     NTSTATUS Status;
//
//     /* We received the buffer via FilterConnectCommunicationPort, cast it back to its original form */
//     FilterPortData = Irp->AssociatedIrp.SystemBuffer;
//
//     /* Get a reference to the server port the filter created */
//     Status = ObReferenceObjectByName(&FilterPortData->PortName,
//                                      0,
//                                      0,
//                                      FLT_PORT_ALL_ACCESS,
//                                      ServerPortObjectType,
//                                      ExGetPreviousMode(),
//                                      0,
//                                      (PVOID *)&ServerPortObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Increment the number of connections on the server port */
//     NumConns = InterlockedIncrement(&ServerPortObject->NumberOfConnections);
//     if (NumConns > ServerPortObject->MaxConnections)
//     {
//         Status = STATUS_CONNECTION_COUNT_LIMIT;
//         goto Quit;
//     }
//
//     /* Initialize a basic kernel handle request */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Now create the new client port object */
//     Status = ObCreateObject(KernelMode,
//                             ClientPortObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             sizeof(FLT_PORT_OBJECT),
//                             0,
//                             0,
//                             (PVOID *)&ClientPortObject);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Quit;
//     }
//
//     /* Clear out the buffer */
//     RtlZeroMemory(ClientPortObject, sizeof(FLT_PORT_OBJECT));
//
//     /* Initialize the locks */
//     ExInitializeRundownProtection(&ClientPortObject->MsgNotifRundownRef);
//     ExInitializeFastMutex(&ClientPortObject->Lock);
//
//     /* Set the server port object this belongs to */
//     ClientPortObject->ServerPort = ServerPortObject;
//
//     /* Setup the message queue */
//     Status = InitializeMessageWaiterQueue(&ClientPortObject->MsgQ);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Quit;
//     }
//
//     /* Create the CCB which we'll attach to the file object */
//     PortCCB = CreatePortCCB(ClientPortObject);
//     if (PortCCB == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quit;
//     }
//
//     /* Now insert the new client port into the object manager */
//     Status = ObInsertObject(ClientPortObject, NULL, FLT_PORT_ALL_ACCESS, 1, NULL, (PHANDLE)&PortHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         /* ObInsertObject() failed and already dereferenced ClientPortObject */
//         ClientPortObject = NULL;
//         goto Quit;
//     }
//
//     /* Add a reference to the filter to keep it alive while we do some work with it */
//     Status = FltObjectReference(ServerPortObject->Filter);
//     if (NT_SUCCESS(Status))
//     {
//         /* Invoke the callback to let the filter know we have a connection */
//         Status = ServerPortObject->ConnectNotify(PortHandle,
//                                                  ServerPortObject->Cookie,
//                                                  NULL, //ConnectionContext
//                                                  0, //SizeOfContext
//                                                  &ClientPortObject->Cookie);
//         if (NT_SUCCESS(Status))
//         {
//             /* Add the client port CCB to the file object */
//             FileObject->FsContext2 = PortCCB;
//
//             /* Lock the port list on the filter and add this new port object to the list */
//             ExAcquireFastMutex(&ServerPortObject->Filter->PortList.mLock);
//             InsertTailList(&ServerPortObject->Filter->PortList.mList, &ClientPortObject->FilterLink);
//             ExReleaseFastMutex(&ServerPortObject->Filter->PortList.mLock);
//         }
//
//         /* We're done with the filter object, decremement the count */
//         FltObjectDereference(ServerPortObject->Filter);
//     }
//
//
// Quit:
//     if (!NT_SUCCESS(Status))
//     {
//         if (ClientPortObject)
//         {
//             ObDereferenceObject(ClientPortObject);
//         }
//
//         if (PortHandle)
//         {
//             ZwClose(PortHandle);
//         }
//         else if (ServerPortObject)
//         {
//             InterlockedDecrement(&ServerPortObject->NumberOfConnections);
//             ObDereferenceObject(ServerPortObject);
//         }
//
//         if (PortCCB)
//         {
//             ExFreePoolWithTag(PortCCB, FM_TAG_CCB);
//         }
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// CloseClientPort( PFILE_OBJECT FileObject,
//                  PIRP Irp)
// {
//     PFLT_CCB Ccb;
//
//     Ccb = (PFLT_CCB)FileObject->FsContext2;
//
//     /* Remove the reference on the filter we added when we opened the port */
//     ObDereferenceObject(Ccb->Data.Port.Port);
//
//     // FIXME: Free the CCB
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// InitializeMessageWaiterQueue( PFLT_MESSAGE_WAITER_QUEUE MsgWaiterQueue)
// {
//     NTSTATUS Status;
//
//     /* Setup the IRP queue */
//     Status = IoCsqInitializeEx(&MsgWaiterQueue->Csq,
//                                FltpAddMessageWaiter,
//                                FltpRemoveMessageWaiter,
//                                FltpGetNextMessageWaiter,
//                                FltpAcquireMessageWaiterLock,
//                                FltpReleaseMessageWaiterLock,
//                                FltpCancelMessageWaiter);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Initialize the waiter queue */
//     ExInitializeFastMutex(&MsgWaiterQueue->WaiterQ.mLock);
//     InitializeListHead(&MsgWaiterQueue->WaiterQ.mList);
//     MsgWaiterQueue->WaiterQ.mCount = 0;
//
//     /* We don't have a minimum waiter length */
//     MsgWaiterQueue->MinimumWaiterLength = (ULONG)-1;
//
//     /* Init the semaphore and event used for counting and signaling available IRPs */
//     KeInitializeSemaphore(&MsgWaiterQueue->Semaphore, 0, MAXLONG);
//     KeInitializeEvent(&MsgWaiterQueue->Event, NotificationEvent, FALSE);
//
//     return STATUS_SUCCESS;
// }
//
// static
// PPORT_CCB
// CreatePortCCB( PFLT_PORT_OBJECT PortObject)
// {
//     PPORT_CCB PortCCB;
//
//     /* Allocate a CCB struct to hold the client port object info */
//     PortCCB = ExAllocatePoolWithTag(NonPagedPool, sizeof(PORT_CCB), FM_TAG_CCB);
//     if (PortCCB)
//     {
//         /* Initialize the structure */
//         PortCCB->Port = PortObject;
//         PortCCB->ReplyWaiterList.mCount = 0;
//         ExInitializeFastMutex(&PortCCB->ReplyWaiterList.mLock);
//         KeInitializeEvent(&PortCCB->ReplyWaiterList.mLock.Event, SynchronizationEvent, 0);
//     }
//
//     return PortCCB;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
