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
    fn ObCreateObject( object_attributes_access_mode: KPROCESSOR_MODE, object_type: PVOID, object_attributes: PVOID, access_mode: KPROCESSOR_MODE, reserved: PVOID, object_body_size: ULONG, page_pool_charge: ULONG, non_paged_pool_charge: ULONG, object: *mut PVOID, ) -> NTSTATUS;
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
        // VALIDATE-FIRST, same shape as port.rs's NtImpersonateClientOfPort. The C
        // wraps ProbeForWriteHandle(PortHandle) + ProbeForRead(ObjectAttributes) +
        // ProbeForReadUnicodeString(ObjectName) (create.c:66-81) in ONE
        // `_SEH2_TRY` whose handler is `_SEH2_YIELD(return
        // _SEH2_GetExceptionCode())`. The translation kept the probes/captures and
        // dropped the handler, so under DUMMY_PSEH a hostile PortHandle,
        // ObjectAttributes, or ObjectName reaching NtCreatePort /
        // NtCreateWaitablePort from an unprivileged caller bugchecked the kernel.
        //
        // (a) WRONG-DIRECTION PROBE, fixed here: the C probes PortHandle with
        // ProbeForWriteHandle (create.c:68) -- a range check PLUS a real
        // read-modify-write touch, because `*PortHandle = Handle` is stored at the
        // end of this function (create.rs:255, deliberately left untouched -- see
        // that site's own comment). The translation substituted a read-only
        // ProbeForRead, so a read-only-but-valid user page sailed past this probe
        // and only faulted later at the store, far from any diagnostic.
        // `probe_write_handle` performs the real write-probe the C did.
        if let Err(st) = ntkrnl_sys::probe::probe_write_generic::<HANDLE>(
            PortHandle as *mut HANDLE,
            // ProbeForWriteHandle is range-only and PRESERVES the caller's
            // value (`*p = *p`). probe_write_handle writes NULL, which is
            // ProbeAndZeroHandle's contract -- a different macro, and it would
            // clobber the caller's OUT variable on every failure path.
            1,
            PreviousMode,
        ) {
            return st;
        }

        if let Err(st) = ntkrnl_sys::probe::probe_user_buffer(
            ObjectAttributes as *const core::ffi::c_void,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            return st;
        }

        // C: inside the __try, whose handler returns the code.
        ObjectName = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*ObjectAttributes).ObjectName)) {
            Ok(v) => v,
            Err(st) => return st,
        };
        if !ObjectName.is_null() {
            // (b) SPURIOUS ALIGNMENT CHECK, fixed here: the C's
            // ProbeForReadUnicodeString (probe.h) is a RANGE check only -- its own
            // comment says "Alignment of the pointers is not verified". The
            // translation instead ran a real ProbeForRead with
            // align_of::<UNICODE_STRING>(), which rejects a misaligned-but-valid
            // ObjectName with STATUS_DATATYPE_MISALIGNMENT where upstream would
            // have accepted it -- a behaviour change the translation introduced,
            // not one the C ever had. Alignment 1 here reproduces the C's
            // range-only check; the capture below is a separate fault-protected
            // copy, matching create.c:75's `*(const volatile UNICODE_STRING*)` read
            // (volatile dropped deliberately -- the copy is already fault-protected
            // and nothing here re-reads the source after this point).
            if let Err(st) = ntkrnl_sys::probe::probe_user_buffer(
                ObjectName as *const core::ffi::c_void,
                core::mem::size_of::<_UNICODE_STRING>(),
                1,
                PreviousMode,
            ) {
                return st;
            }
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                &mut CapturedObjectName as *mut _ as *mut core::ffi::c_void,
                ObjectName as *const core::ffi::c_void,
                core::mem::size_of::<_UNICODE_STRING>(),
            ) {
                return st;
            }
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
            fn Arm64UartPutChar(c: u8);
        }
        let u = Arm64UartKernelBase as *mut ULONG;
        if !u.is_null() {
            let m = b"[LPC-CREATE] PID=\0";
            for &ch in &m[..m.len()-1] {
                Arm64UartPutChar(ch as u8);
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
                    Arm64UartPutChar(buf[bi] as u8);
                }
            }
            let m = b" Port=\0";
            for &ch in &m[..m.len()-1] {
                Arm64UartPutChar(ch as u8);
            }
            if !CapturedObjectName.Buffer.is_null() && CapturedObjectName.Length > 0 {
                for ii in 0..(CapturedObjectName.Length as usize / core::mem::size_of::<WCHAR>()).min(120) {
                    let ch = if PreviousMode != KernelMode {
                        let ptr = CapturedObjectName.Buffer.add(ii);
                        *ptr
                    } else {
                        *CapturedObjectName.Buffer.add(ii)
                    };
                    Arm64UartPutChar((ch & 0xFF) as u8);
                }
            } else {
                let m = b"(unnamed)\0";
                for &ch in &m[..m.len()-1] {
                    Arm64UartPutChar(ch as u8);
                }
            }
            Arm64UartPutChar(b'\r' as u8);
            Arm64UartPutChar(b'\n' as u8);
        }
    }

    Status = ObCreateObject(
        PreviousMode,
        LpcPortObjectType,
        (ObjectAttributes) as _,
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
        // C wraps only this store in `_SEH2_TRY` (create.c:242-251); the
        // handler is JUST `Status = _SEH2_GetExceptionCode();` -- NO
        // `_SEH2_YIELD(return ...)`, no `goto`. It deliberately FALLS
        // THROUGH to `return Status;` at create.c:258, the same statement
        // the success path reaches. There is no handle to close and no
        // cleanup to run: PortHandle was already probed for WRITE up front
        // (create.rs:118-131, via probe_write_generic -- the
        // ProbeForWriteHandle equivalent), so this is a bare
        // fault-tolerant store with no re-probe, matching the C, which has
        // no additional ProbeForWrite inside this TRY either.
        //
        // DO NOT turn this into an early return or a goto-style jump. It
        // only LOOKS equivalent to "propagate the fault status and return"
        // because `Status` is the very next statement either way -- but the
        // C's shape is a genuine fall-through, and a later edit that adds
        // anything else after this `if` block would silently skip it if
        // this were rewritten as an early return.
        if let Err(s) = ntkrnl_sys::probe::safe_copy_raw(
            PortHandle as *mut core::ffi::c_void,
            core::ptr::addr_of!(Handle) as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
        ) {
            Status = s;
        }
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
