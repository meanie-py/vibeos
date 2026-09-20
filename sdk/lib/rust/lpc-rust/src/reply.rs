use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _CLIENT_ID, HANDLE,
    KPROCESSOR_MODE,
    LARGE_INTEGER, POBJECT_TYPE,
    PCLIENT_ID,
    PETHREAD, PEPROCESS,
    PLARGE_INTEGER,
    PLPCP_CONNECTION_MESSAGE,
    PLPCP_DATA_INFO,
    PLPCP_MESSAGE,
    PLPCP_PORT_OBJECT,
    PORT_MESSAGE,
    PPORT_MESSAGE,
    _LPCP_MESSAGE, _LPCP_CONNECTION_MESSAGE,
    LpcpFreeToPortZone,
    LpcpGetMessageFromThread,
    LpcpGetDataInfoFromMessage,
    LpcpValidateClientPort,
    PsGetCurrentThread,
    PsLookupProcessThreadByCid,
    ObReferenceObjectByHandle,
};

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ProbeForRead(Address: PVOID, Length: usize, Alignment: ULONG);
    static mut LpcPortObjectType: PVOID;
    static mut LpcpLock: KGUARDED_MUTEX;
    fn LpcpGetMessageType(Message: PPORT_MESSAGE) -> ULONG;
    fn MmCopyVirtualMemory(
        SourceProcess: PVOID,
        SourceAddress: PVOID,
        TargetProcess: PVOID,
        TargetAddress: PVOID,
        BufferSize: usize,
        PreviousMode: KPROCESSOR_MODE,
        ReturnSize: *mut usize,
    ) -> NTSTATUS;
}
extern "C" {
    fn LpcpReceiveWait(Semaphore: PVOID, WaitMode: KPROCESSOR_MODE) -> NTSTATUS;
    fn ProbeForWritePointer(Address: *mut PVOID);
    fn ProbeForReadLargeInteger(Address: PLARGE_INTEGER);
}
// NTAPI (stdcall on i386) — defs in shim_exports/common_inlines are extern "system"
extern "system" {
    fn LpcpAllocateFromPortZone() -> PLPCP_MESSAGE;
    fn LpcpCompleteWait(Semaphore: PVOID);
}
extern "system" {
    fn PsGetCurrentProcess() -> PEPROCESS;
    fn InsertTailList(ListHead: PLIST_ENTRY, Entry: PLIST_ENTRY);
}

#[no_mangle]
pub unsafe extern "system" fn LpcpSaveDataInfoMessage(
    mut Port: PLPCP_PORT_OBJECT,
    Message: PLPCP_MESSAGE,
    LockFlags: ULONG,
) {
    let LockHeld = (LockFlags & LPCP_LOCK_HELD) != 0;

    // Acquire the lock
    if !LockHeld {
        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock) as *mut _);
    }

    // Check if the port we want is the connection port
    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) > LPCP_UNCONNECTED_PORT {
        // Use the connection port
        Port = (*Port).ConnectionPort;
        if Port.is_null() {
            // Release the lock and return
            if !LockHeld {
                KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock) as *mut _);
            }
            return;
        }
    }

    // Link the message
    InsertTailList(
        core::ptr::addr_of_mut!((*Port).LpcDataInfoChainHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry) as PLIST_ENTRY,
    );

    // Release the lock
    if !LockHeld {
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock) as *mut _);
    }
}

const LPCP_PORT_TYPE_MASK: ULONG = 0x0000000F;
const LPCP_UNCONNECTED_PORT: ULONG = 0x00000002;
const LPCP_CLIENT_PORT: ULONG = 0x00000004;
const LPCP_LOCK_HELD: ULONG = 1;
const LPCP_LOCK_RELEASE: ULONG = 2;
const LPCP_WAITABLE_PORT: ULONG = 0x20000000;

const LPC_CONNECTION_REQUEST: ULONG = 10;
const LPC_REPLY: ULONG = 2;
const LPC_REQUEST: ULONG = 1;

const PORT_ALL_ACCESS: u32 = 0x1F0001;

#[no_mangle]
pub unsafe extern "system" fn CONTAINING_RECORD_MSG(address: PLIST_ENTRY, offset: usize) -> PLPCP_MESSAGE {
    let address = opaque_ptr(address);
    (address as usize - offset) as PLPCP_MESSAGE
}

#[no_mangle]
pub unsafe extern "system" fn LpcpFreeDataInfoMessage(
    Port: PLPCP_PORT_OBJECT,
    MessageId: ULONG,
    CallbackId: ULONG,
    ClientId: _CLIENT_ID,
) {
    let mut port = Port;
    let mut message: PLPCP_MESSAGE;
    let mut list_head: PLIST_ENTRY;
    let mut next_entry: PLIST_ENTRY;

    if ((*port).Flags & LPCP_PORT_TYPE_MASK) > LPCP_UNCONNECTED_PORT {
        port = (*port).ConnectionPort;
        if port.is_null() {
            return;
        }
    }

    list_head = core::ptr::addr_of_mut!((*port).LpcDataInfoChainHead);
    next_entry = (*list_head).Flink;
    while list_head != next_entry {
        message = CONTAINING_RECORD_MSG(
            next_entry,
            core::mem::offset_of!(_LPCP_MESSAGE, __bindgen_anon_1),
        );

        if ((*message).Request.MessageId == MessageId)
            && ((*message).Request.__bindgen_anon_1.ClientId.UniqueThread == ClientId.UniqueThread)
            && ((*message).Request.__bindgen_anon_1.ClientId.UniqueProcess == ClientId.UniqueProcess)
        {
            RemoveEntryList(core::ptr::addr_of_mut!((*message).__bindgen_anon_1.Entry));
            InitializeListHead(core::ptr::addr_of_mut!((*message).__bindgen_anon_1.Entry));
            LpcpFreeToPortZone(message, LPCP_LOCK_HELD);
            break;
        }

        next_entry = (*next_entry).Flink;
    }
}

#[no_mangle]
pub unsafe extern "system" fn LpcpFindDataInfoMessage(
    Port: PLPCP_PORT_OBJECT,
    MessageId: ULONG,
    ClientId: _CLIENT_ID,
) -> PLPCP_MESSAGE {
    let mut message: PLPCP_MESSAGE;
    let mut list_entry: PLIST_ENTRY;
    let mut port = Port;

    if ((*port).Flags & LPCP_PORT_TYPE_MASK) > LPCP_UNCONNECTED_PORT {
        port = (*port).ConnectionPort;
        if port.is_null() {
            return core::ptr::null_mut();
        }
    }

    list_entry = (*core::ptr::addr_of_mut!((*port).LpcDataInfoChainHead)).Flink;
    while list_entry != core::ptr::addr_of_mut!((*port).LpcDataInfoChainHead) {
        message = CONTAINING_RECORD_MSG(
            list_entry,
            core::mem::offset_of!(_LPCP_MESSAGE, __bindgen_anon_1),
        );

        if ((*message).Request.MessageId == MessageId)
            && ((*message).Request.__bindgen_anon_1.ClientId.UniqueProcess == ClientId.UniqueProcess)
            && ((*message).Request.__bindgen_anon_1.ClientId.UniqueThread == ClientId.UniqueThread)
        {
            return message;
        }

        list_entry = (*list_entry).Flink;
    }

    core::ptr::null_mut()
}

#[no_mangle]
/// The PORT_MESSAGE header fields both LpcpMoveMessage variants copy, and the
/// aligned payload length that follows. Split out so the checked and unchecked
/// forms cannot drift in the header while differing only in the payload copy.
#[inline]
unsafe fn lpcp_move_message_aligned_len(Destination: PPORT_MESSAGE) -> usize {
    let data_length = (*Destination).u1.s1.DataLength as usize;
    (data_length + core::mem::size_of::<ULONG>() - 1) & !(core::mem::size_of::<ULONG>() - 1)
}

/// `LpcpMoveMessage` with the payload copy FAULT-PROTECTED, returning the
/// status instead of raising.
///
/// Use at exactly the call sites the C wrapped in `_SEH2_TRY`, and nowhere
/// else. Those sites are not interchangeable with the bare ones: each has a
/// handler that frees the port-zone message and drops object references, and
/// several hold `LpcpLock` when they do it — a fault that escapes there does
/// not merely crash, it leaks the message and leaves the lock HELD.
///
/// The header fields are copied before the payload, exactly as the C does, so
/// a payload fault leaves the destination header already written — which is
/// also what the C leaves behind, because its TRY covers the whole call.
#[no_mangle]
pub unsafe extern "system" fn LpcpMoveMessageChecked(
    Destination: PPORT_MESSAGE,
    Origin: PPORT_MESSAGE,
    Data: PVOID,
    MessageType: ULONG,
    ClientId: PCLIENT_ID,
) -> NTSTATUS {
    LpcpMoveMessageHeader(Destination, Origin, MessageType, ClientId);
    let aligned_length = lpcp_move_message_aligned_len(Destination);
    match ntkrnl_sys::probe::safe_copy_raw(
        (Destination as usize + core::mem::size_of::<PORT_MESSAGE>()) as *mut core::ffi::c_void,
        Data as *const core::ffi::c_void,
        aligned_length,
    ) {
        Ok(()) => STATUS_SUCCESS,
        Err(st) => st,
    }
}

unsafe fn LpcpMoveMessageHeader(
    Destination: PPORT_MESSAGE,
    Origin: PPORT_MESSAGE,
    MessageType: ULONG,
    ClientId: PCLIENT_ID,
) {
    let ClientId = opaque_ptr(ClientId);
    (*Destination).u1.Length = (*Origin).u1.Length;

    (*Destination).u2.s2.Type = if MessageType == 0 {
        (*Origin).u2.s2.Type
    } else {
        (MessageType & 0xFFFF) as i16
    };

    if !ClientId.is_null() {
        (*Destination).__bindgen_anon_1.ClientId.UniqueProcess = (*ClientId).UniqueProcess;
        (*Destination).__bindgen_anon_1.ClientId.UniqueThread = (*ClientId).UniqueThread;
    } else {
        (*Destination).__bindgen_anon_1.ClientId.UniqueProcess = (*Origin).__bindgen_anon_1.ClientId.UniqueProcess;
        (*Destination).__bindgen_anon_1.ClientId.UniqueThread = (*Origin).__bindgen_anon_1.ClientId.UniqueThread;
    }

    (*Destination).MessageId = (*Origin).MessageId;
    (*Destination).__bindgen_anon_2.ClientViewSize = (*Origin).__bindgen_anon_2.ClientViewSize;
}

#[no_mangle]
pub unsafe extern "system" fn LpcpMoveMessage(
    Destination: PPORT_MESSAGE,
    Origin: PPORT_MESSAGE,
    Data: PVOID,
    MessageType: ULONG,
    ClientId: PCLIENT_ID,
) {
    let Data = opaque_ptr(Data);
    LpcpMoveMessageHeader(Destination, Origin, MessageType, ClientId);

    let aligned_length = lpcp_move_message_aligned_len(Destination);
    // RAW on purpose. The C protects only SOME LpcpMoveMessage call sites with
    // _SEH2_TRY; the rest run bare and fault. Making this copy fault-tolerant
    // for everyone would turn those bare sites from a loud crash into a SILENT
    // partial copy, which is strictly worse. Sites the C did protect call
    // LpcpMoveMessageChecked below instead.
    core::ptr::copy_nonoverlapping(
        Data as *const u8,
        (Destination as usize + core::mem::size_of::<PORT_MESSAGE>()) as *mut u8,
        aligned_length,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NtReplyPort(
    PortHandle: HANDLE,
    ReplyMessage: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let mut status: NTSTATUS;
    let previous_mode = KeGetPreviousMode();
    let mut captured_reply_message: PORT_MESSAGE = core::mem::zeroed();
    let mut port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut message: PLPCP_MESSAGE;
    let thread = PsGetCurrentThread();
    let mut wakeup_thread: PETHREAD = core::ptr::null_mut();

    if previous_mode != 0i8 {
        // VALIDATE-FIRST. The C wraps this probe AND the capture read in one
        // `_SEH2_TRY` whose handler is `_SEH2_YIELD(return
        // _SEH2_GetExceptionCode())` (reply.c:209-217). The translation kept
        // the probe and the read and dropped the handler, so under
        // DUMMY_PSEH an unprivileged caller passing a bad ReplyMessage to
        // this syscall bugchecked the box. A range check proves where a
        // buffer IS, never that it stays mapped, so the CAPTURE has to be
        // fault-protected too -- converting only the probe just moves the
        // same bugcheck one frame along into the raw dereference below.
        //
        // The C reads via `*(volatile PORT_MESSAGE*)`, which was dropped by
        // the translation; probe_read_user's single fetch restores that
        // once-only semantics (no TOCTOU window for the caller to mutate the
        // message between separate field reads).
        captured_reply_message =
            match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(
                ReplyMessage as *const PORT_MESSAGE,
                // reply.c uses sizeof(ULONG); align_of::<PORT_MESSAGE>() is 8.
                core::mem::size_of::<ULONG>(),
                previous_mode,
            ) {
                Ok(v) => v,
                Err(st) => return st,
            };
    } else {
        captured_reply_message = *ReplyMessage;
    }

    if (captured_reply_message.u1.s1.DataLength as ULONG
        + core::mem::size_of::<PORT_MESSAGE>() as ULONG)
        > captured_reply_message.u1.s1.TotalLength as ULONG
    {
        return STATUS_INVALID_PARAMETER;
    }

    if captured_reply_message.MessageId == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    status = ObReferenceObjectByHandle(
        PortHandle,
        0,
        LpcPortObjectType as POBJECT_TYPE,
        previous_mode,
        core::ptr::addr_of_mut!(port) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if (captured_reply_message.u1.s1.TotalLength as ULONG > (*port).MaxMessageLength as ULONG)
        || (captured_reply_message.u1.s1.TotalLength as ULONG
            <= captured_reply_message.u1.s1.DataLength as ULONG)
    {
        ObDereferenceObject(port as PVOID);
        return STATUS_PORT_MESSAGE_TOO_LONG;
    }

    status = PsLookupProcessThreadByCid(
        core::ptr::addr_of_mut!(captured_reply_message.__bindgen_anon_1.ClientId),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(wakeup_thread),
    );
    if status < 0 {
        ObDereferenceObject(port as PVOID);
        return status;
    }

    message = LpcpAllocateFromPortZone();
    if message.is_null() {
        ObDereferenceObject(wakeup_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return STATUS_NO_MEMORY;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if ((*wakeup_thread).LpcReplyMessageId != captured_reply_message.MessageId)
        || (!LpcpGetMessageFromThread(wakeup_thread).is_null()
            && (LpcpGetMessageType(core::ptr::addr_of_mut!(
                (*LpcpGetMessageFromThread(wakeup_thread)).Request
            )) != LPC_REQUEST))
    {
        LpcpFreeToPortZone(message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
        ObDereferenceObject(wakeup_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return STATUS_REPLY_MESSAGE_MISMATCH;
    }

    // FAULT-PROTECTED: reply.c:293-310 wraps this call in _SEH2_TRY whose
    // handler frees the port-zone message WITH LpcpLock HELD AND RELEASED,
    // drops two object references, and returns the exception code. The
    // translation dropped the handler, so a bad ReplyMessage payload here did
    // not merely crash -- it crashed while holding LpcpLock. Note the free
    // flags: LPCP_LOCK_HELD | LPCP_LOCK_RELEASE is how the lock gets released
    // on this path, so an early `return` that skipped it would deadlock every
    // later LPC call rather than fail one.
    {
        let st = LpcpMoveMessageChecked(
            core::ptr::addr_of_mut!((*message).Request),
            core::ptr::addr_of_mut!(captured_reply_message),
            (ReplyMessage as usize + core::mem::size_of::<PORT_MESSAGE>()) as PVOID,
            LPC_REPLY,
            core::ptr::null_mut(),
        );
        if st < 0 {
            LpcpFreeToPortZone(message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
            ObDereferenceObject(wakeup_thread as PVOID);
            ObDereferenceObject(port as PVOID);
            return st;
        }
    }

    ObReferenceObject(wakeup_thread as PVOID);
    (*message).RepliedToThread = wakeup_thread;

    (*wakeup_thread).LpcReplyMessageId = 0;
    (*wakeup_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;

    if ((*wakeup_thread).__bindgen_anon_9.__bindgen_anon_1.LpcExitThreadCalled() == 0)
        && !IsListEmpty(core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_1.LpcReplyChain))
    {
        RemoveEntryList(core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_1.LpcReplyChain));
        InitializeListHead(core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_1.LpcReplyChain));
    }

    if ((*thread).__bindgen_anon_9.__bindgen_anon_1.LpcReceivedMsgIdValid() != 0)
        && ((*thread).__bindgen_anon_6.LpcReceivedMessageId == captured_reply_message.MessageId)
    {
        (*thread).__bindgen_anon_6.LpcReceivedMessageId = 0;
        (*thread).__bindgen_anon_9.__bindgen_anon_1.set_LpcReceivedMsgIdValid(0);
    }

    LpcpFreeDataInfoMessage(
        port,
        captured_reply_message.MessageId,
        captured_reply_message.__bindgen_anon_2.CallbackId,
        captured_reply_message.__bindgen_anon_1.ClientId,
    );

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
    LpcpCompleteWait(core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);

    ObDereferenceObject(wakeup_thread as PVOID);
    ObDereferenceObject(port as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtReplyWaitReceivePortEx(
    PortHandle: HANDLE,
    PortContext: *mut PVOID,
    ReplyMessage: PPORT_MESSAGE,
    ReceiveMessage: PPORT_MESSAGE,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let PortContext = opaque_ptr(PortContext);
    let Timeout = opaque_ptr(Timeout);
    let mut status: NTSTATUS;
    let previous_mode = KeGetPreviousMode();
    let mut wait_mode = previous_mode;
    let mut captured_reply_message: PORT_MESSAGE = core::mem::zeroed();
    let mut captured_timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut receive_port: PLPCP_PORT_OBJECT;
    let mut connection_port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut message: PLPCP_MESSAGE;
    let thread = PsGetCurrentThread();
    let mut wakeup_thread: PETHREAD = core::ptr::null_mut();
    let mut connect_message: PLPCP_CONNECTION_MESSAGE;
    let mut connection_info_length: ULONG;

    if previous_mode != 0i8 {
        // VALIDATE-FIRST, all three probes. The C wraps PortContext's probe,
        // ReplyMessage's probe+capture, AND Timeout's probe+capture in ONE
        // `_SEH2_TRY` (reply.c:387-408) whose handler is
        // `_SEH2_YIELD(return _SEH2_GetExceptionCode())`. The translation kept
        // the probes and reads and dropped the handler, so under DUMMY_PSEH a
        // bad PortContext/ReplyMessage/Timeout from an unprivileged caller
        // bugchecked the box. A range check proves where a buffer IS, never
        // that it stays mapped, so the CAPTURE reads must be fault-protected
        // too -- converting only the probes just moves the same bugcheck one
        // frame along into the raw dereferences below. Each site returns the
        // same way the shared C handler did: propagate the status, no cleanup.
        if !PortContext.is_null() {
            if let Err(st) = ntkrnl_sys::probe::probe_user_buffer(
                PortContext as *const core::ffi::c_void,
                core::mem::size_of::<PVOID>(),
                core::mem::align_of::<PVOID>(),
                previous_mode,
            ) {
                return st;
            }
        }

        if !ReplyMessage.is_null() {
            // C reads via `*(volatile PORT_MESSAGE*)`, dropped by the
            // translation; probe_read_user's single fetch restores that
            // once-only semantics.
            captured_reply_message =
                match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(
                ReplyMessage as *const PORT_MESSAGE,
                // reply.c uses sizeof(ULONG); align_of::<PORT_MESSAGE>() is 8.
                core::mem::size_of::<ULONG>(),
                previous_mode,
            ) {
                    Ok(v) => v,
                    Err(st) => return st,
                };
        }

        if !Timeout.is_null() {
            captured_timeout =
                match ntkrnl_sys::probe::probe_read_generic::<LARGE_INTEGER>(
                Timeout as *const LARGE_INTEGER,
                // ProbeForReadLargeInteger is a RANGE CHECK ONLY.
                1,
                previous_mode,
            ) {
                    Ok(v) => v,
                    Err(st) => return st,
                };
        }
    } else {
        if (*thread).__bindgen_anon_7.__bindgen_anon_1.SystemThread() != 0 {
            wait_mode = 1i8; // UserMode
        }

        if !ReplyMessage.is_null() {
            captured_reply_message = *ReplyMessage;
        }
    }

    if !ReplyMessage.is_null() {
        if (captured_reply_message.u1.s1.DataLength as ULONG
            + core::mem::size_of::<PORT_MESSAGE>() as ULONG)
            > captured_reply_message.u1.s1.TotalLength as ULONG
        {
            return STATUS_INVALID_PARAMETER;
        }

        if captured_reply_message.MessageId == 0 {
            return STATUS_INVALID_PARAMETER;
        }
    }

    status = ObReferenceObjectByHandle(
        PortHandle,
        0,
        LpcPortObjectType as POBJECT_TYPE,
        previous_mode,
        core::ptr::addr_of_mut!(port) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if !ReplyMessage.is_null() {
        if (captured_reply_message.u1.s1.TotalLength as ULONG > (*port).MaxMessageLength as ULONG)
            || (captured_reply_message.u1.s1.TotalLength as ULONG
                <= captured_reply_message.u1.s1.DataLength as ULONG)
        {
            ObDereferenceObject(port as PVOID);
            return STATUS_PORT_MESSAGE_TOO_LONG;
        }
    }

    if ((*port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_CLIENT_PORT {
        if (*port).ConnectionPort == port {
            connection_port = port;
            receive_port = port;
            ObReferenceObject(connection_port as PVOID);
        } else {
            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

            connection_port = (*port).ConnectionPort;
            receive_port = (*port).ConnectionPort;
            if connection_port.is_null() {
                KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
                ObDereferenceObject(port as PVOID);
                return STATUS_PORT_DISCONNECTED;
            }

            ObReferenceObject(connection_port as PVOID);
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        }
    } else {
        receive_port = port;
    }

    if !ReplyMessage.is_null() {
        status = PsLookupProcessThreadByCid(
            core::ptr::addr_of_mut!(captured_reply_message.__bindgen_anon_1.ClientId),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(wakeup_thread),
        );
        if status < 0 {
            ObDereferenceObject(port as PVOID);
            if !connection_port.is_null() {
                ObDereferenceObject(connection_port as PVOID);
            }
            return status;
        }

        message = LpcpAllocateFromPortZone();
        if message.is_null() {
            if !connection_port.is_null() {
                ObDereferenceObject(connection_port as PVOID);
            }
            ObDereferenceObject(wakeup_thread as PVOID);
            ObDereferenceObject(port as PVOID);
            return STATUS_NO_MEMORY;
        }

        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

        if ((*wakeup_thread).LpcReplyMessageId != captured_reply_message.MessageId)
            || (!LpcpGetMessageFromThread(wakeup_thread).is_null()
                && (LpcpGetMessageType(core::ptr::addr_of_mut!(
                    (*LpcpGetMessageFromThread(wakeup_thread)).Request
                )) != LPC_REQUEST))
        {
            LpcpFreeToPortZone(message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
            if !connection_port.is_null() {
                ObDereferenceObject(connection_port as PVOID);
            }
            ObDereferenceObject(wakeup_thread as PVOID);
            ObDereferenceObject(port as PVOID);
            return STATUS_REPLY_MESSAGE_MISMATCH;
        }

        // FAULT-PROTECTED: reply.c:538-556. Same shape as the NtReplyPort site
        // above and NOT the same cleanup -- this handler additionally drops
        // connection_port, CONDITIONALLY, before the other two. Copied from the
        // C rather than assumed symmetric with its neighbour.
        {
            let st = LpcpMoveMessageChecked(
                core::ptr::addr_of_mut!((*message).Request),
                core::ptr::addr_of_mut!(captured_reply_message),
                (ReplyMessage as usize + core::mem::size_of::<PORT_MESSAGE>()) as PVOID,
                LPC_REPLY,
                core::ptr::null_mut(),
            );
            if st < 0 {
                LpcpFreeToPortZone(message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                if !connection_port.is_null() {
                    ObDereferenceObject(connection_port as PVOID);
                }
                ObDereferenceObject(wakeup_thread as PVOID);
                ObDereferenceObject(port as PVOID);
                return st;
            }
        }

        ObReferenceObject(wakeup_thread as PVOID);
        (*message).RepliedToThread = wakeup_thread;

        (*wakeup_thread).LpcReplyMessageId = 0;
        (*wakeup_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;

        if ((*wakeup_thread).__bindgen_anon_9.__bindgen_anon_1.LpcExitThreadCalled() == 0)
            && !IsListEmpty(core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_1.LpcReplyChain))
        {
            RemoveEntryList(
                core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_1.LpcReplyChain),
            );
            InitializeListHead(
                core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_1.LpcReplyChain),
            );
        }

        if ((*thread).__bindgen_anon_9.__bindgen_anon_1.LpcReceivedMsgIdValid() != 0)
            && ((*thread).__bindgen_anon_6.LpcReceivedMessageId == captured_reply_message.MessageId)
        {
            (*thread).__bindgen_anon_6.LpcReceivedMessageId = 0;
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_LpcReceivedMsgIdValid(0);
        }

        LpcpFreeDataInfoMessage(
            port,
            captured_reply_message.MessageId,
            captured_reply_message.__bindgen_anon_2.CallbackId,
            captured_reply_message.__bindgen_anon_1.ClientId,
        );

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        LpcpCompleteWait(core::ptr::addr_of_mut!((*wakeup_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);

        ObDereferenceObject(wakeup_thread as PVOID);
    }

    status = LpcpReceiveWait((*receive_port).MsgQueue.Semaphore as PVOID, wait_mode);
    if status != STATUS_SUCCESS {
        if !connection_port.is_null() {
            ObDereferenceObject(connection_port as PVOID);
        }
        ObDereferenceObject(port as PVOID);
        return status;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if IsListEmpty(core::ptr::addr_of_mut!((*receive_port).MsgQueue.ReceiveHead)) {
        if ((*receive_port).Flags & LPCP_WAITABLE_PORT) != 0 {
            KeClearEvent(core::ptr::addr_of_mut!((*receive_port).WaitEvent) as PVOID);
        }

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        if !connection_port.is_null() {
            ObDereferenceObject(connection_port as PVOID);
        }
        ObDereferenceObject(port as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    message = CONTAINING_RECORD_MSG(
        RemoveHeadList(core::ptr::addr_of_mut!((*receive_port).MsgQueue.ReceiveHead)),
        core::mem::offset_of!(_LPCP_MESSAGE, __bindgen_anon_1),
    );

    if IsListEmpty(core::ptr::addr_of_mut!((*receive_port).MsgQueue.ReceiveHead)) {
        if ((*receive_port).Flags & LPCP_WAITABLE_PORT) != 0 {
            KeClearEvent(core::ptr::addr_of_mut!((*receive_port).WaitEvent) as PVOID);
        }
    }

    InitializeListHead(core::ptr::addr_of_mut!((*message).__bindgen_anon_1.Entry));

    (*thread).__bindgen_anon_6.LpcReceivedMessageId = (*message).Request.MessageId;
    (*thread).__bindgen_anon_9.__bindgen_anon_1.set_LpcReceivedMsgIdValid(1);

    // FAULT-PROTECTED DISPATCH. reply.c:644-713 wraps this whole if/else-if in
    // one `_SEH2_TRY`, and its handler is the unusual one: `Status =
    // _SEH2_GetExceptionCode();` and NOTHING ELSE -- no return, no goto. It
    // FALLS THROUGH to the free-or-unlock tail immediately below, which is
    // what releases LpcpLock. An early return here would leave the lock HELD
    // and deadlock every later LPC call in the system rather than fail this
    // one, so the labeled block exists to reproduce the fall-through exactly:
    // `break 'dispatch` is the C's handler, and the tail after it is the C's
    // code after `_SEH2_END`.
    //
    // Note the tail's behaviour depends on `message`, which this block MUTATES
    // (it nulls it once ownership has moved). A fault leaves `message` at
    // whatever value it had reached, and the tail then frees or merely unlocks
    // accordingly -- that is the C's behaviour too, and it is why the status
    // must be recorded rather than returned.
    'dispatch: {
        if LpcpGetMessageType(core::ptr::addr_of_mut!((*message).Request)) == LPC_CONNECTION_REQUEST {
            connect_message = (message as usize + core::mem::size_of::<_LPCP_MESSAGE>())
                as PLPCP_CONNECTION_MESSAGE;

            connection_info_length = (*message).Request.u1.s1.DataLength as ULONG
                - core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>() as ULONG;

            // The C stores the header into user memory and then patches two of
            // its fields in place, three separate user writes. Build the final
            // header LOCALLY and write it once: same resulting bytes, one
            // fault point instead of three, and no window in which the user
            // can observe a half-patched header.
            let mut staged: PORT_MESSAGE = (*message).Request;
            staged.u1.s1.TotalLength = (core::mem::size_of::<PORT_MESSAGE>()
                + connection_info_length as usize) as i16;
            staged.u1.s1.DataLength = connection_info_length as i16;

            message = core::ptr::null_mut();

            if let Err(st) = ntkrnl_sys::probe::safe_copy_to_user(
                ReceiveMessage as *mut core::ffi::c_void,
                core::ptr::addr_of!(staged) as *const core::ffi::c_void,
                core::mem::size_of::<PORT_MESSAGE>(),
                core::mem::size_of::<ULONG>(),
                previous_mode,
            ) {
                status = st;
                break 'dispatch;
            }

            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                (ReceiveMessage as usize + core::mem::size_of::<PORT_MESSAGE>())
                    as *mut core::ffi::c_void,
                (connect_message as usize + core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>())
                    as *const core::ffi::c_void,
                connection_info_length as usize,
            ) {
                status = st;
                break 'dispatch;
            }

            if !PortContext.is_null() {
                let null_ctx: PVOID = core::ptr::null_mut();
                if let Err(st) = ntkrnl_sys::probe::safe_copy_to_user(
                    PortContext as *mut core::ffi::c_void,
                    core::ptr::addr_of!(null_ctx) as *const core::ffi::c_void,
                    core::mem::size_of::<PVOID>(),
                    1,
                    previous_mode,
                ) {
                    status = st;
                    break 'dispatch;
                }
            }
        } else if LpcpGetMessageType(core::ptr::addr_of_mut!((*message).Request)) != LPC_REPLY {
            let st = LpcpMoveMessageChecked(
                ReceiveMessage,
                core::ptr::addr_of_mut!((*message).Request),
                (core::ptr::addr_of_mut!((*message).Request) as usize
                    + core::mem::size_of::<PORT_MESSAGE>()) as PVOID,
                0,
                core::ptr::null_mut(),
            );
            if st < 0 {
                status = st;
                break 'dispatch;
            }

            if !PortContext.is_null() {
                let ctx: PVOID = (*message).PortContext;
                if let Err(st) = ntkrnl_sys::probe::safe_copy_to_user(
                    PortContext as *mut core::ffi::c_void,
                    core::ptr::addr_of!(ctx) as *const core::ffi::c_void,
                    core::mem::size_of::<PVOID>(),
                    1,
                    previous_mode,
                ) {
                    status = st;
                    break 'dispatch;
                }
            }

            if (*message).Request.u2.s2.DataInfoOffset != 0 {
                LpcpSaveDataInfoMessage(port, message, LPCP_LOCK_HELD);
                message = core::ptr::null_mut();
            }
        }
    }

    if !message.is_null() {
        LpcpFreeToPortZone(message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
    } else {
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
    }

    if !connection_port.is_null() {
        ObDereferenceObject(connection_port as PVOID);
    }
    ObDereferenceObject(port as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtReplyWaitReceivePort(
    PortHandle: HANDLE,
    PortContext: *mut PVOID,
    ReplyMessage: PPORT_MESSAGE,
    ReceiveMessage: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let PortContext = opaque_ptr(PortContext);
    NtReplyWaitReceivePortEx(
        PortHandle,
        PortContext,
        ReplyMessage,
        ReceiveMessage,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtReplyWaitReplyPort(
    PortHandle: HANDLE,
    ReplyMessage: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn LpcpCopyRequestData(
    Write: BOOLEAN,
    PortHandle: HANDLE,
    Message: PPORT_MESSAGE,
    Index: ULONG,
    Buffer: PVOID,
    BufferLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let Buffer = opaque_ptr(Buffer);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut status: NTSTATUS;
    let previous_mode = KeGetPreviousMode();
    let mut captured_message: PORT_MESSAGE = core::mem::zeroed();
    let mut port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut client_thread: PETHREAD = core::ptr::null_mut();
    let mut local_return_length: usize = 0;
    let mut info_message: PLPCP_MESSAGE;
    let mut data_info: PLPCP_DATA_INFO;
    let mut data_info_base_address: PVOID;

    if previous_mode != 0i8 {
        // VALIDATE-FIRST. The C wraps this probe AND the capture read in one
        // `_SEH2_TRY` whose handler is `_SEH2_YIELD(return
        // _SEH2_GetExceptionCode())` (reply.c:794-801). The translation kept
        // the probe and the read and dropped the handler, so under
        // DUMMY_PSEH an unprivileged caller passing a bad Message to
        // NtReadRequestData/NtWriteRequestData (which funnel through this
        // helper) bugchecked the box. A range check proves where a buffer IS,
        // never that it stays mapped, so the CAPTURE has to be
        // fault-protected too -- converting only the probe just moves the
        // same bugcheck one frame along into the raw dereference below.
        //
        // The C reads via `*(volatile PORT_MESSAGE*)`, dropped by the
        // translation; probe_read_user's single fetch restores that
        // once-only semantics.
        captured_message =
            match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(
                Message as *const PORT_MESSAGE,
                // reply.c:796 uses sizeof(PVOID) here, unlike the sizeof(ULONG)
                // sites above -- copied from the C, not assumed uniform.
                core::mem::size_of::<PVOID>(),
                previous_mode,
            ) {
                Ok(v) => v,
                Err(st) => return st,
            };
    } else {
        captured_message = *Message;
    }

    if captured_message.u2.s2.DataInfoOffset == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    status = ObReferenceObjectByHandle(
        PortHandle,
        PORT_ALL_ACCESS,
        LpcPortObjectType as POBJECT_TYPE,
        previous_mode,
        core::ptr::addr_of_mut!(port) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    status = PsLookupProcessThreadByCid(
        core::ptr::addr_of_mut!(captured_message.__bindgen_anon_1.ClientId),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(client_thread),
    );
    if status < 0 {
        ObDereferenceObject(port as PVOID);
        return status;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if ((*client_thread).LpcReplyMessageId != captured_message.MessageId)
        || (captured_message.MessageId == 0)
    {
        status = STATUS_REPLY_MESSAGE_MISMATCH;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(client_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    if LpcpValidateClientPort(client_thread, port) == 0 {
        status = STATUS_REPLY_MESSAGE_MISMATCH;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(client_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    info_message = LpcpFindDataInfoMessage(
        port,
        captured_message.MessageId,
        captured_message.__bindgen_anon_1.ClientId,
    );
    if info_message.is_null() {
        status = STATUS_INVALID_PARAMETER;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(client_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    data_info = LpcpGetDataInfoFromMessage(core::ptr::addr_of_mut!((*info_message).Request));

    if Index >= (*data_info).NumberOfEntries {
        status = STATUS_INVALID_PARAMETER;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(client_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    // `Entries` is a C flexible array (lpc.h:74 declares `Entries[1]` alongside
    // `NumberOfEntries`), so bindgen's `[_; 1]` makes `Entries[Index]` bounds-check
    // and PANIC for any Index >= 1 -- exactly the values the guard above admits.
    let entry = core::ptr::addr_of!((*data_info).Entries)
        .cast::<ntkrnl_sys::ddk::_LPCP_DATA_INFO__bindgen_ty_1>()
        .add(Index as usize);

    if BufferLength > (*entry).DataLength {
        status = STATUS_INVALID_PARAMETER;
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        ObDereferenceObject(client_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    data_info_base_address = (*entry).BaseAddress;

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if Write != 0 {
        status = MmCopyVirtualMemory(
            PsGetCurrentProcess() as PVOID,
            Buffer,
            (*client_thread).ThreadsProcess as PVOID,
            data_info_base_address,
            BufferLength as usize,
            previous_mode,
            core::ptr::addr_of_mut!(local_return_length),
        );
    } else {
        status = MmCopyVirtualMemory(
            (*client_thread).ThreadsProcess as PVOID,
            data_info_base_address,
            PsGetCurrentProcess() as PVOID,
            Buffer,
            BufferLength as usize,
            previous_mode,
            core::ptr::addr_of_mut!(local_return_length),
        );
    }

    if status < 0 {
        ObDereferenceObject(client_thread as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    if !ReturnLength.is_null() {
        // FAULT-PROTECTED, and the handler SWALLOWS -- the third distinct shape
        // in this file. reply.c:932-941 wraps this single store in _SEH2_TRY
        // and its handler is a DPRINT1 and nothing else: Status is deliberately
        // left untouched and execution continues to Cleanup. So a caller that
        // passes a bad ReturnLength still gets the operation's real status,
        // because the data WAS copied and only the courtesy length write
        // failed. Propagating the error here would turn a successful copy into
        // a reported failure, which is why `let _ =` is correct rather than
        // lazy -- the C decided this, not us.
        let staged_len = local_return_length as ULONG;
        let _ = ntkrnl_sys::probe::safe_copy_to_user(
            ReturnLength as *mut core::ffi::c_void,
            core::ptr::addr_of!(staged_len) as *const core::ffi::c_void,
            core::mem::size_of::<ULONG>(),
            core::mem::size_of::<ULONG>(),
            previous_mode,
        );
    }

    ObDereferenceObject(client_thread as PVOID);
    ObDereferenceObject(port as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtReadRequestData(
    PortHandle: HANDLE,
    Message: PPORT_MESSAGE,
    Index: ULONG,
    Buffer: PVOID,
    BufferLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let Buffer = opaque_ptr(Buffer);
    let ReturnLength = opaque_ptr(ReturnLength);
    LpcpCopyRequestData(0, PortHandle, Message, Index, Buffer, BufferLength, ReturnLength)
}

#[no_mangle]
pub unsafe extern "system" fn NtWriteRequestData(
    PortHandle: HANDLE,
    Message: PPORT_MESSAGE,
    Index: ULONG,
    Buffer: PVOID,
    BufferLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let Buffer = opaque_ptr(Buffer);
    let ReturnLength = opaque_ptr(ReturnLength);
    LpcpCopyRequestData(1, PortHandle, Message, Index, Buffer, BufferLength, ReturnLength)
}
