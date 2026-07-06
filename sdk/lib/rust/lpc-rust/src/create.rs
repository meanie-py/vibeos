use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    HANDLE, KPROCESSOR_MODE, _OBJECT_ATTRIBUTES, POBJECT_ATTRIBUTES, PHANDLE,
    PLPCP_PORT_OBJECT, PLPCP_NONPAGED_PORT_QUEUE,
    _LPCP_PORT_OBJECT, _LPCP_NONPAGED_PORT_QUEUE, LPCP_PORT_OBJECT,
    _UNICODE_STRING, _EVENT_TYPE_NotificationEvent,
    PETHREAD, PEPROCESS, ACCESS_MASK,
    PsGetCurrentThread,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG);
    fn KeInitializeSemaphore(Semaphore: PVOID, Count: LONG, Limit: LONG);
    fn PsGetCurrentProcessId() -> HANDLE;
    fn ObCloseHandle(Handle: HANDLE, AccessMode: KPROCESSOR_MODE) -> NTSTATUS;
    fn ObCreateObject(
        ProbeMode: KPROCESSOR_MODE,
        ObjectType: PVOID,
        ObjectAttributes: POBJECT_ATTRIBUTES,
        AccessMode: KPROCESSOR_MODE,
        Reserved: PVOID,
        ObjectSize: ULONG,
        PagedPoolCharge: ULONG,
        NonPagedPoolCharge: ULONG,
        Object: *mut PVOID,
    ) -> NTSTATUS;
    fn ObInsertObject(
        Object: PVOID,
        PassedAccessState: PVOID,
        DesiredAccess: ACCESS_MASK,
        AdditionalReferences: ULONG,
        ReferencedObject: *mut PVOID,
        Handle: PHANDLE,
    ) -> NTSTATUS;
    fn KeInitializeEvent(Event: PVOID, Type: ULONG, State: BOOLEAN);
    static LpcPortObjectType: PVOID;
    static LpcpMaxMessageSize: ULONG;
}
extern "system" {
    fn PsGetCurrentProcess() -> PEPROCESS;
}
extern "C" {
    fn RtlInitEmptyUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *mut WCHAR, MaximumLength: USHORT);
}

const MAXLONG: LONG = 0x7fffffff;
const LPCP_UNCONNECTED_PORT: ULONG = 0x00000002;
const LPCP_CONNECTION_PORT: ULONG = 0x00000001;
const LPCP_WAITABLE_PORT: ULONG = 0x20000000;
const PORT_ALL_ACCESS: ACCESS_MASK = 0x001F0001;
const FIELD_OFFSET_LPCP_MESSAGE_REQUEST: ULONG = core::mem::offset_of!(ntkrnl_sys::ddk::_LPCP_MESSAGE, Request) as ULONG;
const PORT_MESSAGE_SIZE: usize = core::mem::size_of::<ntkrnl_sys::ddk::PORT_MESSAGE>();

#[no_mangle]
pub unsafe extern "system" fn LpcpInitializePortQueue(Port: PLPCP_PORT_OBJECT) -> NTSTATUS {
    let message_queue = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_LPCP_NONPAGED_PORT_QUEUE>(),
        u32::from_be_bytes(*b"troP"),
    ) as PLPCP_NONPAGED_PORT_QUEUE;

    if message_queue.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeSemaphore(
        core::ptr::addr_of_mut!((*message_queue).Semaphore) as PVOID,
        0,
        MAXLONG,
    );
    (*message_queue).BackPointer = Port;

    (*Port).MsgQueue.Semaphore = core::ptr::addr_of_mut!((*message_queue).Semaphore) as *mut _;
    InitializeListHead(core::ptr::addr_of_mut!((*Port).MsgQueue.ReceiveHead));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LpcpCreatePort(
    PortHandle: PHANDLE,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    MaxConnectionInfoLength: ULONG,
    MaxMessageLength: ULONG,
    MaxPoolUsage: ULONG,
    Waitable: BOOLEAN,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = KeGetPreviousMode();
    let mut CapturedObjectName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let ObjectName: *mut _UNICODE_STRING;
    let mut Port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitEmptyUnicodeString(&mut CapturedObjectName, core::ptr::null_mut(), 0);

    if PreviousMode != KernelMode {
        ProbeForRead(PortHandle as PVOID, core::mem::size_of::<HANDLE>(), core::mem::align_of::<HANDLE>() as ULONG);
        ProbeForRead(ObjectAttributes as PVOID, core::mem::size_of::<_OBJECT_ATTRIBUTES>(), core::mem::size_of::<ULONG>() as ULONG);

        ObjectName = (*ObjectAttributes).ObjectName;
        if !ObjectName.is_null() {
            ProbeForRead(ObjectName as PVOID, core::mem::size_of::<_UNICODE_STRING>(), core::mem::align_of::<_UNICODE_STRING>() as ULONG);
            CapturedObjectName = *ObjectName;
        }
    } else {
        ObjectName = (*ObjectAttributes).ObjectName;
        if !ObjectName.is_null() {
            CapturedObjectName = *ObjectName;
        }
    }

    if CapturedObjectName.Length == 0 {
        CapturedObjectName.Buffer = core::ptr::null_mut();
    }

    #[cfg(target_arch = "aarch64")]
    {
        extern "C" {
            static Arm64UartKernelBase: PVOID;
        }
        let u = Arm64UartKernelBase as *mut ULONG;
        if !u.is_null() {
            let m = b"[LPC-CREATE] PID=\0";
            for &ch in &m[..m.len()-1] {
                *u = ch as ULONG;
            }
            {
                let pid = PsGetCurrentProcessId() as ULONG;
                let mut buf = [0u8; 12];
                let mut pos = 0;
                if pid == 0 {
                    buf[pos] = b'0';
                    pos += 1;
                } else {
                    let mut t = [0u8; 12];
                    let mut tp = 0;
                    let mut p = pid;
                    while p > 0 {
                        t[tp] = b'0' + (p % 10) as u8;
                        p /= 10;
                        tp += 1;
                    }
                    while tp > 0 {
                        tp -= 1;
                        buf[pos] = t[tp];
                        pos += 1;
                    }
                }
                for bi in 0..pos {
                    *u = buf[bi] as ULONG;
                }
            }
            let m = b" Port=\0";
            for &ch in &m[..m.len()-1] {
                *u = ch as ULONG;
            }
            if !CapturedObjectName.Buffer.is_null() && CapturedObjectName.Length > 0 {
                for ii in 0..(CapturedObjectName.Length as usize / core::mem::size_of::<WCHAR>()).min(120) {
                    let ch = if PreviousMode != KernelMode {
                        let ptr = CapturedObjectName.Buffer.add(ii);
                        *ptr
                    } else {
                        *CapturedObjectName.Buffer.add(ii)
                    };
                    *u = (ch & 0xFF) as ULONG;
                }
            } else {
                let m = b"(unnamed)\0";
                for &ch in &m[..m.len()-1] {
                    *u = ch as ULONG;
                }
            }
            *u = b'\r' as ULONG;
            *u = b'\n' as ULONG;
        }
    }

    Status = ObCreateObject(
        PreviousMode,
        LpcPortObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<LPCP_PORT_OBJECT>() as ULONG,
        0,
        0,
        &mut Port as *mut _ as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    core::ptr::write_bytes(Port as *mut u8, 0, core::mem::size_of::<LPCP_PORT_OBJECT>());
    (*Port).ConnectionPort = Port;
    (*Port).Creator = (*PsGetCurrentThread()).Cid;
    InitializeListHead(core::ptr::addr_of_mut!((*Port).LpcDataInfoChainHead));
    InitializeListHead(core::ptr::addr_of_mut!((*Port).LpcReplyChainHead));

    if CapturedObjectName.Buffer.is_null() {
        (*Port).Flags = LPCP_UNCONNECTED_PORT;
        (*Port).ConnectedPort = Port;
        (*Port).__bindgen_anon_1.ServerProcess = core::ptr::null_mut();
    } else {
        (*Port).Flags = LPCP_CONNECTION_PORT;
        (*Port).__bindgen_anon_1.ServerProcess = PsGetCurrentProcess();
        ObReferenceObject((*Port).__bindgen_anon_1.ServerProcess as PVOID);
    }

    if Waitable != 0 {
        (*Port).Flags |= LPCP_WAITABLE_PORT;
    }

    Status = LpcpInitializePortQueue(Port);
    if Status < 0 {
        ObDereferenceObject(Port as PVOID);
        return Status;
    }

    if ((*Port).Flags & LPCP_WAITABLE_PORT) != 0 {
        KeInitializeEvent(
            core::ptr::addr_of_mut!((*Port).WaitEvent) as PVOID,
            _EVENT_TYPE_NotificationEvent,
            0u8,
        );
    }

    (*Port).MaxMessageLength = (LpcpMaxMessageSize - FIELD_OFFSET_LPCP_MESSAGE_REQUEST) as ULONG;
    (*Port).MaxConnectionInfoLength = ((*Port).MaxMessageLength as usize - PORT_MESSAGE_SIZE - core::mem::size_of::<ntkrnl_sys::ddk::LPCP_CONNECTION_MESSAGE>()) as ULONG;

    if (*Port).MaxConnectionInfoLength < MaxConnectionInfoLength {
        ObDereferenceObject(Port as PVOID);
        return STATUS_INVALID_PARAMETER_3;
    } else if (*Port).MaxMessageLength < MaxMessageLength {
        ObDereferenceObject(Port as PVOID);
        return STATUS_INVALID_PARAMETER_4;
    }

    (*Port).MaxMessageLength = MaxMessageLength;

    Status = ObInsertObject(
        Port as PVOID,
        core::ptr::null_mut(),
        PORT_ALL_ACCESS,
        0,
        core::ptr::null_mut(),
        &mut Handle,
    );

    if Status >= 0 {
        *PortHandle = Handle;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreatePort(
    PortHandle: PHANDLE,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    MaxConnectInfoLength: ULONG,
    MaxDataLength: ULONG,
    MaxPoolUsage: ULONG,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    LpcpCreatePort(
        PortHandle,
        ObjectAttributes,
        MaxConnectInfoLength,
        MaxDataLength,
        MaxPoolUsage,
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateWaitablePort(
    PortHandle: PHANDLE,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    MaxConnectInfoLength: ULONG,
    MaxDataLength: ULONG,
    MaxPoolUsage: ULONG,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    LpcpCreatePort(
        PortHandle,
        ObjectAttributes,
        MaxConnectInfoLength,
        MaxDataLength,
        MaxPoolUsage,
        1u8,
    )
}
