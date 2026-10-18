use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use crate::close::LpcpClosePort;

use ntkrnl_sys::ddk::{


    _CLIENT_ID,
    ExInitializePagedLookasideList,
    _GENERIC_MAPPING,
    HANDLE, KPROCESSOR_MODE,
    _LPCP_NONPAGED_PORT_QUEUE, LPCP_NONPAGED_PORT_QUEUE,
    _LPCP_PORT_OBJECT, LPCP_PORT_OBJECT,
    LpcpDeletePort,
    LpcpGetPortFromThread,
    _OBJECT_TYPE_INITIALIZER, OBJECT_TYPE_INITIALIZER,
    ObCreateObjectType,
    _PAGED_LOOKASIDE_LIST, PAGED_LOOKASIDE_LIST,
    PETHREAD, _ETHREAD,
    PLPCP_PORT_OBJECT,
    POBJECT_TYPE,
    _PORT_MESSAGE, PORT_MESSAGE,
    PPORT_MESSAGE,
    RtlInitUnicodeString,
    _SECURITY_CLIENT_CONTEXT, SECURITY_CLIENT_CONTEXT,
    ObReferenceObjectByHandle,
    PsLookupProcessThreadByCid,
    ObReferenceObjectSafe,
    SeImpersonateClientEx,
    SeCreateClientSecurity,
    PORT_INFORMATION_CLASS,
    PULONG, PEPROCESS,
    ACCESS_MASK,
    _SECURITY_QUALITY_OF_SERVICE,
    PsGetCurrentThread,
    _UNICODE_STRING,
    _EVENT_TYPE_NotificationEvent,
};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ProbeForRead(address: PVOID, length: SIZE_T, alignment: ULONG);
}
extern "system" {
    fn SeDeleteClientSecurity(ClientContext: *mut _SECURITY_CLIENT_CONTEXT);
}

const PORT_CONNECT: ULONG = 0x0001;
const PORT_ALL_ACCESS: ULONG = 0x001F0001;

// N_ROUND_UP(PORT_MAXIMUM_MESSAGE_LENGTH + sizeof(LPCP_MESSAGE) + sizeof(LPCP_CONNECTION_MESSAGE), 16)
#[cfg(target_pointer_width = "32")]
const PORT_MAXIMUM_MESSAGE_LENGTH: usize = 256;
#[cfg(target_pointer_width = "64")]
const PORT_MAXIMUM_MESSAGE_LENGTH: usize = 512;

const LPCP_MAX_MESSAGE_SIZE: ULONG = {
    let raw = PORT_MAXIMUM_MESSAGE_LENGTH
        + core::mem::size_of::<ntkrnl_sys::ddk::_LPCP_MESSAGE>()
        + core::mem::size_of::<ntkrnl_sys::ddk::_LPCP_CONNECTION_MESSAGE>();
    ((raw + 15) & !15) as ULONG
};
const LPCP_PORT_TYPE_MASK: ULONG = 0x0000000F;
const LPCP_COMMUNICATION_PORT: ULONG = 0x00000003;
const LPCP_SECURITY_DYNAMIC: ULONG = 0x80000000;
// lpc.h. The port had 1, which is LPCP_THREAD_FLAG_IS_PORT: a client blocked with
// its port tagged into LpcReplyMessage could not be impersonated.
const LPCP_THREAD_FLAG_NO_IMPERSONATION: usize = 0x00000002;

#[no_mangle]
pub static mut LpcPortObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut LpcWaitablePortObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut LpcpMaxMessageSize: ULONG = 0;

#[no_mangle]
pub static mut LpcpMessagesLookaside: PAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut LpcpLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut LpcpTraceLevel: ULONG = 0;

#[no_mangle]
pub static mut LpcpNextMessageId: ULONG = 1;

#[no_mangle]
pub static mut LpcpNextCallbackId: ULONG = 1;

#[no_mangle]
pub static mut LpcpPortMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: READ_CONTROL | PORT_CONNECT,
    GenericWrite: DELETE | PORT_CONNECT,
    GenericExecute: 0,
    GenericAll: PORT_ALL_ACCESS,
};

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn LpcInitSystem() -> BOOLEAN {
    let mut object_type_initializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut name: _UNICODE_STRING = core::mem::zeroed();

    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    RtlZeroMemory(
        core::ptr::addr_of_mut!(object_type_initializer) as PVOID,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(name),
        w!("Port").as_ptr() as *mut u16,
    );
    object_type_initializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    object_type_initializer.DefaultNonPagedPoolCharge =
        core::mem::size_of::<LPCP_NONPAGED_PORT_QUEUE>() as ULONG;
    object_type_initializer.DefaultPagedPoolCharge = core::mem::offset_of!(_LPCP_PORT_OBJECT, WaitEvent) as ULONG;
    object_type_initializer.GenericMapping = LpcpPortMapping;
    object_type_initializer.PoolType = PagedPool;
    object_type_initializer.UseDefaultObject = 1u8;
    object_type_initializer.CloseProcedure = Some(LpcpClosePort);
    object_type_initializer.DeleteProcedure = Some(LpcpDeletePort);
    object_type_initializer.ValidAccessMask = PORT_ALL_ACCESS;
    object_type_initializer.InvalidAttributes = OBJ_VALID_ATTRIBUTES & !OBJ_CASE_INSENSITIVE;
    ObCreateObjectType(
        core::ptr::addr_of_mut!(name),
        core::ptr::addr_of_mut!(object_type_initializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(LpcPortObjectType),
    );

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(name),
        w!("WaitablePort").as_ptr() as *mut u16,
    );
    object_type_initializer.PoolType = NonPagedPool;
    object_type_initializer.DefaultNonPagedPoolCharge += core::mem::size_of::<LPCP_PORT_OBJECT>() as ULONG;
    object_type_initializer.DefaultPagedPoolCharge = 0;
    object_type_initializer.UseDefaultObject = 0u8;
    ObCreateObjectType(
        core::ptr::addr_of_mut!(name),
        core::ptr::addr_of_mut!(object_type_initializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(LpcWaitablePortObjectType),
    );

    LpcpMaxMessageSize = LPCP_MAX_MESSAGE_SIZE;
    ExInitializePagedLookasideList(
        core::ptr::addr_of_mut!(LpcpMessagesLookaside),
        None,
        None,
        0,
        LpcpMaxMessageSize as ntkrnl_sys::ddk::SIZE_T,
        0x4C63704D,
        32u16,
    );

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn LpcpValidateClientPort(
    ClientThread: PETHREAD,
    Port: PLPCP_PORT_OBJECT,
) -> BOOLEAN {
    let ClientThread = opaque_ptr(ClientThread);
    let ThreadPort = LpcpGetPortFromThread(ClientThread);
    if ThreadPort.is_null() {
        return 0u8;
    }

    if (Port == ThreadPort)
        || (Port == (*ThreadPort).ConnectionPort)
        || (Port == (*ThreadPort).ConnectedPort)
    {
        return 1u8;
    }

    if (((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_COMMUNICATION_PORT)
        && ((*Port).ConnectionPort == ThreadPort)
    {
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn NtImpersonateClientOfPort(
    PortHandle: HANDLE,
    ClientMessage: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut ClientId: _CLIENT_ID;
    let mut MessageId: ULONG;
    let mut Port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut ConnectedPort: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut ClientThread: PETHREAD = core::ptr::null_mut();
    let mut ClientContext: SECURITY_CLIENT_CONTEXT = core::mem::zeroed();

    if PreviousMode != 0i8 {
        // VALIDATE-FIRST. The C wraps this probe AND both capture reads in one
        // `_SEH2_TRY` whose handler is `_SEH2_YIELD(return
        // _SEH2_GetExceptionCode())` (port.c:142-152). The translation kept the
        // probe and the reads and dropped the handler, so under DUMMY_PSEH any
        // unprivileged caller passing a bad ClientMessage to this syscall
        // bugchecked the box. A range check proves where a buffer IS, never that
        // it stays mapped, so the CAPTURE has to be fault-protected too -- the MM
        // case proved that by moving the same bugcheck one frame along into
        // memcpy when only the probe was converted.
        //
        // Capturing the whole PORT_MESSAGE once is also strictly safer than the
        // C's two separate volatile field reads: those let a hostile caller
        // change MessageId after ClientId had been read. The probe covered
        // sizeof(PORT_MESSAGE) either way, so this reads no more than the C did.
        let captured: PORT_MESSAGE =
            // port.c:144 probes with sizeof(PVOID). probe_read_user would demand
            // align_of::<PORT_MESSAGE>(), which the `double DoNotUseThisField`
            // union member makes 8 on i386 -- rejecting addresses the C accepts.
            match ntkrnl_sys::probe::probe_read_generic(
                ClientMessage as *const PORT_MESSAGE,
                core::mem::size_of::<PVOID>(),
                PreviousMode,
            ) {
                Ok(v) => v,
                Err(st) => return st,
            };
        ClientId = captured.__bindgen_anon_1.ClientId;
        MessageId = captured.MessageId;
    } else {
        // Kernel-mode caller: no probe, but one fault-tolerant fetch of the
        // whole message rather than two raw field reads.
        let captured: PORT_MESSAGE = match ntkrnl_sys::probe::read_back(ClientMessage as *const PORT_MESSAGE) {
            Ok(v) => v,
            Err(st) => return st,
        };
        ClientId = captured.__bindgen_anon_1.ClientId;
        MessageId = captured.MessageId;
    }

    Status = ObReferenceObjectByHandle(
        PortHandle,
        PORT_ALL_ACCESS,
        LpcPortObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Port) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_COMMUNICATION_PORT {
        Status = STATUS_INVALID_PORT_HANDLE;
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    Status = PsLookupProcessThreadByCid(
        core::ptr::addr_of_mut!(ClientId),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ClientThread),
    );
    if Status < 0 {
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    ConnectedPort = (*Port).ConnectedPort;
    if ConnectedPort.is_null() || ObReferenceObjectSafe(ConnectedPort as PVOID) == 0u8 {
        ConnectedPort = core::ptr::null_mut();
        Status = STATUS_PORT_DISCONNECTED;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    if ((*ClientThread).__bindgen_anon_5.LpcReplyMessage as usize & LPCP_THREAD_FLAG_NO_IMPERSONATION) != 0 {
        Status = STATUS_ACCESS_DENIED;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    if ((*ClientThread).LpcReplyMessageId != MessageId) || (MessageId == 0) {
        Status = STATUS_REPLY_MESSAGE_MISMATCH;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    if LpcpValidateClientPort(ClientThread, Port) == 0u8 {
        Status = STATUS_REPLY_MESSAGE_MISMATCH;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if ((*ConnectedPort).Flags & LPCP_SECURITY_DYNAMIC) == 0 {
        Status = SeImpersonateClientEx(
            core::ptr::addr_of_mut!((*ConnectedPort).StaticSecurity),
            core::ptr::null_mut(),
        );
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    Status = SeCreateClientSecurity(
        ClientThread,
        core::ptr::addr_of_mut!((*ConnectedPort).SecurityQos),
        0u8,
        core::ptr::addr_of_mut!(ClientContext),
    );
    if Status < 0 {
        ObDereferenceObject(Port as PVOID);
        if !ConnectedPort.is_null() {
            ObDereferenceObject(ConnectedPort as PVOID);
        }
        if !ClientThread.is_null() {
            ObDereferenceObject(ClientThread as PVOID);
        }
        return Status;
    }

    Status = SeImpersonateClientEx(
        core::ptr::addr_of_mut!(ClientContext),
        core::ptr::null_mut(),
    );

    SeDeleteClientSecurity(core::ptr::addr_of_mut!(ClientContext));

    ObDereferenceObject(Port as PVOID);
    if !ConnectedPort.is_null() {
        ObDereferenceObject(ConnectedPort as PVOID);
    }
    if !ClientThread.is_null() {
        ObDereferenceObject(ClientThread as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryPortInformationProcess() -> NTSTATUS {
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationPort(
    PortHandle: HANDLE,
    PortInformationClass: PORT_INFORMATION_CLASS,
    PortInformation: PVOID,
    PortInformationLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let PortInformation = opaque_ptr(PortInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    STATUS_NOT_IMPLEMENTED
}
