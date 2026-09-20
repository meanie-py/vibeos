use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    HANDLE, KPROCESSOR_MODE,
    PCLIENT_ID,
    PETHREAD, PKSEMAPHORE,
    PLPCP_DATA_INFO, PLPCP_MESSAGE, PLPCP_PORT_OBJECT,
    _LPCP_MESSAGE, _LPCP_DATA_INFO,
    PORT_MESSAGE, PPORT_MESSAGE,
    PsGetCurrentThread,
    LpcpFreeToPortZone, LpcpGetMessageFromThread,
    LpcpSetPortToThread,
    LpcpGetDataInfoFromMessage,
    ObReferenceObjectSafe,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// The fault-protected wrapper around LpcpMoveMessage (routes the payload copy
// through safe_copy_raw and returns NTSTATUS instead of raising). Defined in
// reply.rs, `#[no_mangle] pub`, so it is reachable by module path within this
// crate. Use it at exactly the _SEH2_TRY-wrapped LpcpMoveMessage sites below;
// the bare `LpcpMoveMessage` extern declared further down stays in use for
// the sites the C left unprotected.
use crate::reply::LpcpMoveMessageChecked;

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ProbeForRead(address: PVOID, length: usize, alignment: ULONG);
    fn ProbeForWrite(address: PVOID, length: usize, alignment: ULONG);

    static mut LpcPortObjectType: PVOID;
    static mut LpcpLock: KGUARDED_MUTEX;
    static mut LpcpNextMessageId: ULONG;

    fn LpcpGetMessageType(msg: PPORT_MESSAGE) -> ULONG;
    fn LpcpMoveMessage(
        dest: PPORT_MESSAGE,
        src: PPORT_MESSAGE,
        data: PVOID,
        msg_type: ULONG,
        cid: PVOID,
    );
    fn LpcpSaveDataInfoMessage(Port: PLPCP_PORT_OBJECT, Message: PLPCP_MESSAGE, LockFlags: ULONG);
    fn ObReferenceObjectByHandle(
        handle: HANDLE,
        access: ULONG,
        object_type: PVOID,
        mode: KPROCESSOR_MODE,
        object: *mut PVOID,
        info: PVOID,
    ) -> NTSTATUS;
}
extern "system" {
    fn KeEnterCriticalRegion();
    fn KeLeaveCriticalRegion();
    fn LpcpAllocateFromPortZone() -> PLPCP_MESSAGE;
    fn LpcpCompleteWait(semaphore: PVOID);
    fn LpcpReplyWait(semaphore: PVOID, mode: KPROCESSOR_MODE);
}

const LPCP_LOCK_HELD: ULONG = 1;
const LPCP_LOCK_RELEASE: ULONG = 2;
const LPCP_CONNECTION_PORT: ULONG = 0x00000001;
const LPCP_CLIENT_PORT: ULONG = 0x00000004;
const LPCP_COMMUNICATION_PORT: ULONG = 0x00000003;
const LPCP_PORT_TYPE_MASK: ULONG = 0x0000000F;
const LPCP_WAITABLE_PORT: ULONG = 0x20000000;

const LPC_DATAGRAM: ULONG = 0x03;
const LPC_REQUEST: ULONG = 1;
const LPC_CLIENT_DIED: ULONG = 0x06;
const LPC_PORT_CLOSED: ULONG = 0x05;
const LPC_EXCEPTION: ULONG = 0x07;
const LPC_DEBUG_EVENT: ULONG = 0x08;
const LPC_ERROR_EVENT: ULONG = 0x09;
const LPC_KERNELMODE_MESSAGE: ULONG = 0x8000;

const IO_NO_INCREMENT: LONG = 0;

#[no_mangle]
pub unsafe extern "system" fn LpcRequestPort(
    PortObject: PVOID,
    LpcMessage: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortObject = opaque_ptr(PortObject);
    let Port = PortObject as PLPCP_PORT_OBJECT;
    let mut QueuePort: PLPCP_PORT_OBJECT;
    let mut ConnectionPort: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut MessageType: ULONG;
    let Message: PLPCP_MESSAGE;
    let PreviousMode = KeGetPreviousMode();
    let Thread = PsGetCurrentThread();

    if (*LpcMessage).u2.s2.Type != 0 {
        MessageType = LpcpGetMessageType(LpcMessage);

        if MessageType < LPC_DATAGRAM || MessageType > LPC_CLIENT_DIED {
            return STATUS_INVALID_PARAMETER;
        }

        if PreviousMode == 0i8 && ((*LpcMessage).u2.s2.Type & (LPC_KERNELMODE_MESSAGE as i16)) != 0 {
            MessageType |= LPC_KERNELMODE_MESSAGE;
        }
    } else {
        MessageType = LPC_DATAGRAM;
    }

    if (*LpcMessage).u2.s2.DataInfoOffset != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*LpcMessage).u1.s1.TotalLength as ULONG) > (*Port).MaxMessageLength as ULONG
        || ((*LpcMessage).u1.s1.TotalLength as ULONG) <= ((*LpcMessage).u1.s1.DataLength as ULONG)
    {
        return STATUS_PORT_MESSAGE_TOO_LONG;
    }

    Message = LpcpAllocateFromPortZone();
    if Message.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*Message).RepliedToThread = core::ptr::null_mut();
    (*Message).PortContext = core::ptr::null_mut();

    LpcpMoveMessage(
        core::ptr::addr_of_mut!((*Message).Request),
        LpcMessage,
        LpcMessage.offset(1) as PVOID,
        MessageType,
        core::ptr::addr_of_mut!((*Thread).Cid) as PVOID,
    );

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_CONNECTION_PORT {
        QueuePort = (*Port).ConnectedPort;
        if !QueuePort.is_null() {
            if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CLIENT_PORT {
                (*Message).PortContext = (*QueuePort).PortContext;
                ConnectionPort = (*Port).ConnectionPort;
                QueuePort = ConnectionPort;
                if ConnectionPort.is_null() {
                    LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                    return STATUS_PORT_DISCONNECTED;
                }
            } else if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_COMMUNICATION_PORT {
                ConnectionPort = (*Port).ConnectionPort;
                QueuePort = ConnectionPort;
                if ConnectionPort.is_null() {
                    LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                    return STATUS_PORT_DISCONNECTED;
                }
            }

            if !ConnectionPort.is_null() {
                ObReferenceObject(ConnectionPort as PVOID);
            }
        }
    } else {
        QueuePort = PortObject as PLPCP_PORT_OBJECT;
    }

    if !QueuePort.is_null() {
        (*Message).Request.MessageId = LpcpNextMessageId;
        LpcpNextMessageId = LpcpNextMessageId.wrapping_add(1);
        if LpcpNextMessageId == 0 {
            LpcpNextMessageId = 1;
        }
        (*Message).Request.__bindgen_anon_2.CallbackId = 0;

        (*Thread).LpcReplyMessageId = 0;

        InsertTailList(
            core::ptr::addr_of_mut!((*QueuePort).MsgQueue.ReceiveHead),
            core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry),
        );

        KeEnterCriticalRegion();
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        LpcpCompleteWait((*QueuePort).MsgQueue.Semaphore as PVOID);

        if ((*QueuePort).Flags & LPCP_WAITABLE_PORT) != 0 {
            KeSetEvent(core::ptr::addr_of_mut!((*QueuePort).WaitEvent) as PVOID, IO_NO_INCREMENT, 0u8);
        }

        KeLeaveCriticalRegion();

        if !ConnectionPort.is_null() {
            ObDereferenceObject(ConnectionPort as PVOID);
        }
        return STATUS_SUCCESS;
    }

    LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
    if !ConnectionPort.is_null() {
        ObDereferenceObject(ConnectionPort as PVOID);
    }
    return STATUS_PORT_DISCONNECTED;
}

#[no_mangle]
pub unsafe extern "system" fn LpcRequestWaitReplyPort(
    PortObject: PVOID,
    LpcRequest: PPORT_MESSAGE,
    LpcReply: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortObject = opaque_ptr(PortObject);
    let mut Status = STATUS_SUCCESS;
    let PreviousMode = KeGetPreviousMode();
    let Thread = PsGetCurrentThread();
    let Port = PortObject as PLPCP_PORT_OBJECT;
    let mut QueuePort: PLPCP_PORT_OBJECT;
    let mut ReplyPort: PLPCP_PORT_OBJECT;
    let mut ConnectionPort: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut MessageType: u16;
    let Message: PLPCP_MESSAGE;
    let Callback: BOOLEAN;
    let Semaphore: PKSEMAPHORE;

    if (*Thread).__bindgen_anon_9.__bindgen_anon_1.LpcExitThreadCalled() != 0 {
        return STATUS_THREAD_IS_TERMINATING;
    }

    MessageType = LpcpGetMessageType(LpcRequest) as u16;
    match MessageType as ULONG {
        0 => {
            MessageType = LPC_REQUEST as u16;
            Callback = 0u8;
        }
        LPC_REQUEST => {
            Callback = 1u8;
        }
        LPC_CLIENT_DIED | LPC_PORT_CLOSED | LPC_EXCEPTION | LPC_DEBUG_EVENT | LPC_ERROR_EVENT => {
            Callback = 0u8;
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    (*LpcRequest).u2.s2.Type = MessageType as i16;

    if ((*LpcRequest).u1.s1.TotalLength as ULONG) > (*Port).MaxMessageLength as ULONG
        || ((*LpcRequest).u1.s1.TotalLength as ULONG) <= ((*LpcRequest).u1.s1.DataLength as ULONG)
    {
        return STATUS_PORT_MESSAGE_TOO_LONG;
    }

    Message = LpcpAllocateFromPortZone();
    if Message.is_null() {
        return STATUS_NO_MEMORY;
    }

    if Callback != 0 {
        Semaphore = core::ptr::null_mut();
        return STATUS_NOT_IMPLEMENTED;
    } else {
        LpcpMoveMessage(
            core::ptr::addr_of_mut!((*Message).Request),
            LpcRequest,
            LpcRequest.offset(1) as PVOID,
            0,
            core::ptr::addr_of_mut!((*Thread).Cid) as PVOID,
        );

        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

        (*Message).PortContext = core::ptr::null_mut();

        if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_CONNECTION_PORT {
            QueuePort = (*Port).ConnectedPort;
            if QueuePort.is_null() {
                LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                return STATUS_PORT_DISCONNECTED;
            }

            ReplyPort = QueuePort;

            if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CLIENT_PORT {
                (*Message).PortContext = (*QueuePort).PortContext;
                ConnectionPort = (*Port).ConnectionPort;
                QueuePort = ConnectionPort;
                if ConnectionPort.is_null() {
                    LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                    return STATUS_PORT_DISCONNECTED;
                }
            } else if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_COMMUNICATION_PORT {
                ConnectionPort = (*Port).ConnectionPort;
                QueuePort = ConnectionPort;
                if ConnectionPort.is_null() {
                    LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                    return STATUS_PORT_DISCONNECTED;
                }
            }

            if !ConnectionPort.is_null() {
                ObReferenceObject(ConnectionPort as PVOID);
            }
        } else {
            QueuePort = Port;
            ReplyPort = Port;
        }

        (*Message).RepliedToThread = core::ptr::null_mut();
        (*Message).SenderPort = Port;

        (*Message).Request.MessageId = LpcpNextMessageId;
        LpcpNextMessageId = LpcpNextMessageId.wrapping_add(1);
        if LpcpNextMessageId == 0 {
            LpcpNextMessageId = 1;
        }
        (*Message).Request.__bindgen_anon_2.CallbackId = 0;

        (*Thread).LpcReplyMessageId = (*Message).Request.MessageId;
        (*Thread).__bindgen_anon_5.LpcReplyMessage = core::ptr::null_mut();

        InsertTailList(
            core::ptr::addr_of_mut!((*QueuePort).MsgQueue.ReceiveHead),
            core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry),
        );
        InsertTailList(
            core::ptr::addr_of_mut!((*ReplyPort).LpcReplyChainHead),
            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain),
        );
        LpcpSetPortToThread(Thread, Port);

        KeEnterCriticalRegion();
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        Semaphore = (*QueuePort).MsgQueue.Semaphore as PKSEMAPHORE;

        if ((*QueuePort).Flags & LPCP_WAITABLE_PORT) != 0 {
            KeSetEvent(core::ptr::addr_of_mut!((*QueuePort).WaitEvent) as PVOID, IO_NO_INCREMENT, 0u8);
        }
    }

    LpcpCompleteWait(Semaphore as PVOID);
    KeLeaveCriticalRegion();

    LpcpReplyWait(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID, PreviousMode);

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    let Message = LpcpGetMessageFromThread(Thread);
    (*Thread).__bindgen_anon_5.LpcReplyMessage = core::ptr::null_mut();
    (*Thread).LpcReplyMessageId = 0;

    if !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain)) {
        RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain));
        InitializeListHead(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain));
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if Status == STATUS_SUCCESS {
        if !Message.is_null() {
            LpcpMoveMessage(
                LpcReply,
                core::ptr::addr_of_mut!((*Message).Request),
                core::ptr::addr_of_mut!((*Message).Request).offset(1) as PVOID,
                0,
                core::ptr::null_mut(),
            );

            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

            if !(*Message).RepliedToThread.is_null() {
                ObDereferenceObject((*Message).RepliedToThread as PVOID);
                (*Message).RepliedToThread = core::ptr::null_mut();
            }

            LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
        } else {
            Status = STATUS_LPC_REPLY_LOST;
        }
    } else {
        if !Message.is_null() {
            LpcpFreeToPortZone(Message, 0);
        }
    }

    if !ConnectionPort.is_null() {
        ObDereferenceObject(ConnectionPort as PVOID);
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn NtRequestPort(
    PortHandle: HANDLE,
    LpcRequest: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let mut Status: NTSTATUS;
    let PreviousMode = KeGetPreviousMode();
    let Thread = PsGetCurrentThread();
    let mut CapturedLpcRequest: PORT_MESSAGE;
    let mut Port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut QueuePort: PLPCP_PORT_OBJECT;
    let mut ConnectionPort: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let MessageType: ULONG;
    let Message: PLPCP_MESSAGE;

    if PreviousMode != 0i8 {
        // VALIDATE-FIRST. The C wraps the probe AND the capture in one
        // `_SEH2_TRY` returning `_SEH2_GetExceptionCode()` (send.c:456-466); the
        // translation kept both and dropped the handler, so under DUMMY_PSEH a
        // bad LpcRequest to this syscall bugchecked the box. A range check says
        // where a buffer IS, not that it stays mapped, so the capture is
        // fault-protected too.
        //
        // NOT `probe_read_user`, deliberately: that derives alignment from
        // `align_of::<PORT_MESSAGE>()`, which is 8 on 64-bit, where the C asks
        // for `sizeof(ULONG)` = 4. Tightening a probe is not a safe default --
        // it rejects addresses upstream accepts, which is a bug the translation
        // already introduced once elsewhere in this crate. Spell the C's
        // alignment out.
        if let Err(st) = ntkrnl_sys::probe::probe_user_buffer(
            LpcRequest as *const core::ffi::c_void,
            core::mem::size_of::<PORT_MESSAGE>(),
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            return st;
        }
        // Staged through MaybeUninit rather than written straight into
        // CapturedLpcRequest: a raw-pointer write does not initialise the
        // binding as far as the borrow checker is concerned (rustc rejected the
        // direct version), and only a COMPLETE copy initialises a PORT_MESSAGE
        // -- a fault may have copied a prefix, which must never be read as a
        // value. Zeroing first would also work and would cost a memset on the
        // hottest syscall in the system.
        let mut Staging = core::mem::MaybeUninit::<PORT_MESSAGE>::uninit();
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            Staging.as_mut_ptr() as *mut core::ffi::c_void,
            LpcRequest as *const core::ffi::c_void,
            core::mem::size_of::<PORT_MESSAGE>(),
        ) {
            return st;
        }
        CapturedLpcRequest = Staging.assume_init();
    } else {
        CapturedLpcRequest = *LpcRequest;
    }

    // From the CAPTURED copy, as the C does (send.c:481). This used to read
    // `(*LpcRequest).u2.s2.Type` -- a SECOND dereference of the raw user
    // pointer, and two bugs rather than one: a hostile caller can change Type
    // between the capture and this read (the capture exists precisely to stop
    // that), and it faults on the SUCCESS path, outside the `PreviousMode`
    // guard, so it was reachable by kernel callers too.
    MessageType = CapturedLpcRequest.u2.s2.Type as ULONG | LPC_DATAGRAM;

    if CapturedLpcRequest.u2.s2.DataInfoOffset != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if ((CapturedLpcRequest.u1.s1.DataLength as ULONG)
        .wrapping_add(core::mem::size_of::<PORT_MESSAGE>() as ULONG))
        > (CapturedLpcRequest.u1.s1.TotalLength as ULONG)
    {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        PortHandle,
        0,
        LpcPortObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Port) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if (CapturedLpcRequest.u1.s1.TotalLength as ULONG) > (*Port).MaxMessageLength as ULONG
        || (CapturedLpcRequest.u1.s1.TotalLength as ULONG) <= (CapturedLpcRequest.u1.s1.DataLength as ULONG)
    {
        ObDereferenceObject(Port as PVOID);
        return STATUS_PORT_MESSAGE_TOO_LONG;
    }

    Message = LpcpAllocateFromPortZone();
    if Message.is_null() {
        ObDereferenceObject(Port as PVOID);
        return STATUS_NO_MEMORY;
    }

    // FAULT-PROTECTED (send.c:522-538, one _SEH2_TRY over just this copy).
    // The C's handler is:
    //     LpcpFreeToPortZone(Message, 0);
    //     ObDereferenceObject(Port);
    //     _SEH2_YIELD(return _SEH2_GetExceptionCode());
    // -- a plain `return`, so the shape here matches: on failure we free and
    // dereference and return immediately, same as every other early-out in
    // this function. The free flag is bare `0`, NOT LPCP_LOCK_HELD |
    // LPCP_LOCK_RELEASE -- LpcpLock has not been acquired yet at this point
    // in NtRequestPort, so that combination would release a lock we don't
    // hold.
    Status = LpcpMoveMessageChecked(
        core::ptr::addr_of_mut!((*Message).Request),
        core::ptr::addr_of_mut!(CapturedLpcRequest),
        LpcRequest.offset(1) as PVOID,
        MessageType,
        core::ptr::addr_of_mut!((*Thread).Cid) as PCLIENT_ID,
    );
    if Status < 0 {
        LpcpFreeToPortZone(Message, 0);
        ObDereferenceObject(Port as PVOID);
        return Status;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    (*Message).PortContext = core::ptr::null_mut();

    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_CONNECTION_PORT {
        QueuePort = (*Port).ConnectedPort;
        if QueuePort.is_null() {
            LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
            ObDereferenceObject(Port as PVOID);
            return STATUS_PORT_DISCONNECTED;
        }

        if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CLIENT_PORT {
            (*Message).PortContext = (*QueuePort).PortContext;
            ConnectionPort = (*Port).ConnectionPort;
            QueuePort = ConnectionPort;
            if ConnectionPort.is_null() {
                LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                ObDereferenceObject(Port as PVOID);
                return STATUS_PORT_DISCONNECTED;
            }
        } else if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_COMMUNICATION_PORT {
            ConnectionPort = (*Port).ConnectionPort;
            QueuePort = ConnectionPort;
            if ConnectionPort.is_null() {
                LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                ObDereferenceObject(Port as PVOID);
                return STATUS_PORT_DISCONNECTED;
            }
        }

        if !ConnectionPort.is_null() {
            ObReferenceObject(ConnectionPort as PVOID);
        }
    } else {
        QueuePort = Port;
    }

    if !QueuePort.is_null() && ObReferenceObjectSafe(QueuePort as PVOID) != 0 {
        (*Message).SenderPort = Port;

        (*Message).Request.MessageId = LpcpNextMessageId;
        LpcpNextMessageId = LpcpNextMessageId.wrapping_add(1);
        if LpcpNextMessageId == 0 {
            LpcpNextMessageId = 1;
        }
        (*Message).Request.__bindgen_anon_2.CallbackId = 0;

        (*Thread).LpcReplyMessageId = 0;

        InsertTailList(
            core::ptr::addr_of_mut!((*QueuePort).MsgQueue.ReceiveHead),
            core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry),
        );

        KeEnterCriticalRegion();
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        LpcpCompleteWait((*QueuePort).MsgQueue.Semaphore as PVOID);

        if ((*QueuePort).Flags & LPCP_WAITABLE_PORT) != 0 {
            KeSetEvent(core::ptr::addr_of_mut!((*QueuePort).WaitEvent) as PVOID, IO_NO_INCREMENT, 0u8);
        }

        KeLeaveCriticalRegion();

        if !ConnectionPort.is_null() {
            ObDereferenceObject(ConnectionPort as PVOID);
        }
        ObDereferenceObject(QueuePort as PVOID);
        ObDereferenceObject(Port as PVOID);
        return STATUS_SUCCESS;
    }

    Status = STATUS_PORT_DISCONNECTED;

    if !Message.is_null() {
        LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
    }

    ObDereferenceObject(Port as PVOID);
    if !ConnectionPort.is_null() {
        ObDereferenceObject(ConnectionPort as PVOID);
    }
    return Status;
}

/// `LpcpVerifyMessageDataInfo` with every user read FAULT-PROTECTED.
///
/// The plain version below walks `Message + DataInfoOffset`, which is BEYOND
/// the `sizeof(PORT_MESSAGE)` its callers probe. In the C that is safe only
/// because the user-mode call sits inside `_SEH2_TRY` (send.c:721-743); the
/// KernelMode call at send.c:748 is deliberately unprotected. So this exists
/// alongside the original rather than replacing it, exactly as
/// `LpcpMoveMessageChecked` does — protecting the kernel-mode path would
/// convert a loud crash into a silent wrong answer.
///
/// THE CHECK ORDER IS LOAD-BEARING and is copied from the C. The header
/// bounds tests run BEFORE anything past the header is touched, so a malformed
/// message still returns `STATUS_INVALID_PARAMETER` rather than
/// `STATUS_ACCESS_VIOLATION`. Probing the whole `TotalLength` extent up front
/// would be simpler and WRONG: it changes the answer upstream gives for
/// hostile input, which is a divergence dressed as hardening.
#[no_mangle]
pub unsafe extern "system" fn LpcpVerifyMessageDataInfoChecked(
    Message: PPORT_MESSAGE,
    NumberOfDataEntries: *mut ULONG,
    PreviousMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let NumberOfDataEntries = opaque_ptr(NumberOfDataEntries);

    // One protected capture of the header. The C re-reads DataInfoOffset from
    // user memory here deliberately (send.c:846) as a re-validation rather
    // than trusting the earlier capture, so this re-reads too — just safely.
    let hdr = match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(
        Message as *const PORT_MESSAGE,
        core::mem::size_of::<ULONG>(),
        PreviousMode,
    ) {
        Ok(v) => v,
        Err(st) => return st,
    };

    if hdr.u2.s2.DataInfoOffset == 0 {
        *NumberOfDataEntries = 0;
        return STATUS_SUCCESS;
    }

    let total = hdr.u1.s1.TotalLength as ULONG;
    let off = hdr.u2.s2.DataInfoOffset as ULONG;
    let hdr_size = core::mem::size_of::<PORT_MESSAGE>() as ULONG;
    let di_size = core::mem::size_of::<ntkrnl_sys::ddk::_LPCP_DATA_INFO>() as ULONG;

    if (total < hdr_size + di_size) || (off < hdr_size) || (off > total - di_size) {
        return STATUS_INVALID_PARAMETER;
    }

    // Only now is anything past the header touched, and it is bounded by the
    // tests above: off is >= sizeof(PORT_MESSAGE) and <= total - sizeof(DATA_INFO).
    let data_info = (Message as *mut u8).add(off as usize) as PLPCP_DATA_INFO;
    let di = match ntkrnl_sys::probe::probe_read_generic::<ntkrnl_sys::ddk::_LPCP_DATA_INFO>(
        data_info as *const ntkrnl_sys::ddk::_LPCP_DATA_INFO,
        core::mem::size_of::<ULONG>(),
        PreviousMode,
    ) {
        Ok(v) => v,
        Err(st) => return st,
    };

    let end_of_entries = (core::ptr::addr_of!((*data_info).Entries) as *mut UCHAR).add(
        di.NumberOfEntries as usize
            * core::mem::size_of::<ntkrnl_sys::ddk::_LPCP_DATA_INFO__bindgen_ty_1>(),
    );
    if (end_of_entries > (Message as *mut UCHAR).add(total as usize))
        || (end_of_entries < (Message as *mut UCHAR))
    {
        return STATUS_INVALID_PARAMETER;
    }

    *NumberOfDataEntries = di.NumberOfEntries;
    STATUS_SUCCESS
}

/// UNPROTECTED by design — the KernelMode path. See
/// `LpcpVerifyMessageDataInfoChecked` above; keep the two in step.
#[no_mangle]
pub unsafe extern "system" fn LpcpVerifyMessageDataInfo(
    Message: PPORT_MESSAGE,
    NumberOfDataEntries: *mut ULONG,
) -> NTSTATUS {
    let NumberOfDataEntries = opaque_ptr(NumberOfDataEntries);
    let DataInfo: PLPCP_DATA_INFO;
    let EndOfEntries: *mut UCHAR;

    if (*Message).u2.s2.DataInfoOffset == 0 {
        *NumberOfDataEntries = 0;
        return STATUS_SUCCESS;
    }

    if (((*Message).u1.s1.TotalLength as ULONG)
        < (core::mem::size_of::<PORT_MESSAGE>() as ULONG + core::mem::size_of::<ntkrnl_sys::ddk::_LPCP_DATA_INFO>() as ULONG))
        || (((*Message).u2.s2.DataInfoOffset as ULONG) < core::mem::size_of::<PORT_MESSAGE>() as ULONG)
        || (((*Message).u2.s2.DataInfoOffset as ULONG)
            > (((*Message).u1.s1.TotalLength as ULONG) - core::mem::size_of::<ntkrnl_sys::ddk::_LPCP_DATA_INFO>() as ULONG))
    {
        return STATUS_INVALID_PARAMETER;
    }

    DataInfo = LpcpGetDataInfoFromMessage(Message);

    EndOfEntries = (core::ptr::addr_of!((*DataInfo).Entries) as *mut UCHAR)
        .add((*DataInfo).NumberOfEntries as usize * core::mem::size_of::<ntkrnl_sys::ddk::_LPCP_DATA_INFO__bindgen_ty_1>());
    if (EndOfEntries > ((Message as *mut UCHAR).add((*Message).u1.s1.TotalLength as usize)))
        || (EndOfEntries < (Message as *mut UCHAR))
    {
        return STATUS_INVALID_PARAMETER;
    }

    *NumberOfDataEntries = (*DataInfo).NumberOfEntries;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NtRequestWaitReplyPort(
    PortHandle: HANDLE,
    LpcRequest: PPORT_MESSAGE,
    LpcReply: PPORT_MESSAGE,
) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let mut Status: NTSTATUS;
    let mut CapturedLpcRequest: PORT_MESSAGE;
    let mut NumberOfDataEntries: ULONG = 0;
    let mut Port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut QueuePort: PLPCP_PORT_OBJECT;
    let mut ReplyPort: PLPCP_PORT_OBJECT;
    let mut ConnectionPort: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let Message: PLPCP_MESSAGE;
    let PreviousMode = KeGetPreviousMode();
    let Thread = PsGetCurrentThread();
    let Callback: BOOLEAN;
    let Semaphore: PKSEMAPHORE;
    let MessageType: ULONG;
    let DataInfo: PLPCP_DATA_INFO;

    if (*Thread).__bindgen_anon_9.__bindgen_anon_1.LpcExitThreadCalled() != 0 {
        return STATUS_THREAD_IS_TERMINATING;
    }

    if PreviousMode != 0i8 {
        // VALIDATE-FIRST. One `_SEH2_TRY` in the C (send.c:721-743) covers the
        // read probe, the capture, the write probe and the
        // LpcpVerifyMessageDataInfo call, with `_SEH2_YIELD(return
        // _SEH2_GetExceptionCode())` as its handler. The translation kept all
        // four and dropped the handler, so under DUMMY_PSEH a bad LpcRequest or
        // LpcReply to this syscall -- the one every CSR/LPC call in the system
        // goes through -- bugchecked the box.
        //
        // Alignment is spelled out rather than taken from `align_of`: the C asks
        // for `sizeof(ULONG)` = 4, and `align_of::<PORT_MESSAGE>()` is 8 on
        // 64-bit. A tighter probe rejects addresses upstream accepts, which is a
        // defect, not a hardening.
        if let Err(st) = ntkrnl_sys::probe::probe_user_buffer(
            LpcRequest as *const core::ffi::c_void,
            core::mem::size_of::<PORT_MESSAGE>(),
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            return st;
        }
        // Staged through MaybeUninit rather than written straight into
        // CapturedLpcRequest: a raw-pointer write does not initialise the
        // binding as far as the borrow checker is concerned (rustc rejected the
        // direct version), and only a COMPLETE copy initialises a PORT_MESSAGE
        // -- a fault may have copied a prefix, which must never be read as a
        // value. Zeroing first would also work and would cost a memset on the
        // hottest syscall in the system.
        let mut Staging = core::mem::MaybeUninit::<PORT_MESSAGE>::uninit();
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            Staging.as_mut_ptr() as *mut core::ffi::c_void,
            LpcRequest as *const core::ffi::c_void,
            core::mem::size_of::<PORT_MESSAGE>(),
        ) {
            return st;
        }
        CapturedLpcRequest = Staging.assume_init();
        if let Err(st) = ntkrnl_sys::probe::probe_user_buffer(
            LpcReply as *const core::ffi::c_void,
            core::mem::size_of::<PORT_MESSAGE>(),
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            return st;
        }

        // NOW PROTECTED. This walks `Message + DataInfoOffset`, beyond the
        // sizeof(PORT_MESSAGE) probed above, and in the C is safe only because
        // it sits inside this TRY (send.c:731). The Checked variant does the
        // same work with every user read fault-protected, in the C's check
        // ORDER so a malformed message still answers STATUS_INVALID_PARAMETER
        // rather than STATUS_ACCESS_VIOLATION. The KernelMode call below keeps
        // the unprotected version, because the C leaves that one bare too.
        Status = LpcpVerifyMessageDataInfoChecked(
            LpcRequest,
            core::ptr::addr_of_mut!(NumberOfDataEntries),
            PreviousMode,
        );
        if Status < 0 {
            return Status;
        }
    } else {
        CapturedLpcRequest = *LpcRequest;
        Status = LpcpVerifyMessageDataInfo(LpcRequest, core::ptr::addr_of_mut!(NumberOfDataEntries));
        if Status < 0 {
            return Status;
        }
    }

    CapturedLpcRequest.u2.s2.Type &= !0x4000;

    if LpcpGetMessageType(core::ptr::addr_of_mut!(CapturedLpcRequest)) == LPC_REQUEST {
        Callback = 1u8;
    } else if LpcpGetMessageType(core::ptr::addr_of_mut!(CapturedLpcRequest)) != 0 {
        return STATUS_INVALID_PARAMETER;
    } else {
        CapturedLpcRequest.u2.s2.Type |= LPC_REQUEST as i16;
        Callback = 0u8;
    }

    MessageType = CapturedLpcRequest.u2.s2.Type as ULONG;

    if ((CapturedLpcRequest.u1.s1.DataLength as ULONG) + core::mem::size_of::<PORT_MESSAGE>() as ULONG)
        > (CapturedLpcRequest.u1.s1.TotalLength as ULONG)
    {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObReferenceObjectByHandle(
        PortHandle,
        0,
        LpcPortObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Port) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    if (CapturedLpcRequest.u1.s1.TotalLength as ULONG) > (*Port).MaxMessageLength as ULONG
        || (CapturedLpcRequest.u1.s1.TotalLength as ULONG) <= (CapturedLpcRequest.u1.s1.DataLength as ULONG)
    {
        ObDereferenceObject(Port as PVOID);
        return STATUS_PORT_MESSAGE_TOO_LONG;
    }

    Message = LpcpAllocateFromPortZone();
    if Message.is_null() {
        ObDereferenceObject(Port as PVOID);
        return STATUS_NO_MEMORY;
    }

    if Callback != 0 {
        Semaphore = core::ptr::null_mut();
        return STATUS_NOT_IMPLEMENTED;
    } else {
        // FAULT-PROTECTED as ONE unit (send.c:843-876, one _SEH2_TRY over the
        // DataInfoOffset re-read, the NumberOfEntries read, AND the move).
        // The C's handler is:
        //     DPRINT1(...);
        //     LpcpFreeToPortZone(Message, 0);
        //     ObDereferenceObject(Port);
        //     _SEH2_YIELD(return _SEH2_GetExceptionCode());
        // -- same shape as NtRequestPort's site: a plain early return, free
        // flag bare `0` because LpcpLock is not held yet here either.
        //
        // Both raw-pointer reads below sit inside the C's TRY too (the
        // re-read of LpcRequest->u2.s2.DataInfoOffset, distinct from the
        // already-captured CapturedLpcRequest, and DataInfo->NumberOfEntries,
        // which LpcpGetDataInfoFromMessage points PAST the probed
        // sizeof(PORT_MESSAGE) header) -- so a bare deref of either would
        // reopen the same bugcheck the move itself was protected against.
        // Alignment is spelled out as sizeof(ULONG) = 4 for both, matching
        // the C's plain-ULONG-sized field reads, not align_of::<PORT_MESSAGE>().
        let DataInfoOffset = match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(
            LpcRequest as *const PORT_MESSAGE,
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            Ok(msg) => msg.u2.s2.DataInfoOffset,
            Err(st) => {
                LpcpFreeToPortZone(Message, 0);
                ObDereferenceObject(Port as PVOID);
                return st;
            }
        };
        if DataInfoOffset != 0 {
            DataInfo = LpcpGetDataInfoFromMessage(LpcRequest);
            let NumberOfEntries = match ntkrnl_sys::probe::probe_read_generic::<ULONG>(
                core::ptr::addr_of!((*DataInfo).NumberOfEntries),
                core::mem::size_of::<ULONG>(),
                PreviousMode,
            ) {
                Ok(n) => n,
                Err(st) => {
                    LpcpFreeToPortZone(Message, 0);
                    ObDereferenceObject(Port as PVOID);
                    return st;
                }
            };
            if NumberOfEntries != NumberOfDataEntries {
                LpcpFreeToPortZone(Message, 0);
                ObDereferenceObject(Port as PVOID);
                return STATUS_INVALID_PARAMETER;
            }
        }

        Status = LpcpMoveMessageChecked(
            core::ptr::addr_of_mut!((*Message).Request),
            core::ptr::addr_of_mut!(CapturedLpcRequest),
            LpcRequest.offset(1) as PVOID,
            MessageType,
            core::ptr::addr_of_mut!((*Thread).Cid) as PCLIENT_ID,
        );
        if Status < 0 {
            LpcpFreeToPortZone(Message, 0);
            ObDereferenceObject(Port as PVOID);
            return Status;
        }

        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

        (*Message).PortContext = core::ptr::null_mut();

        if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_CONNECTION_PORT {
            QueuePort = (*Port).ConnectedPort;
            if QueuePort.is_null() {
                LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                ObDereferenceObject(Port as PVOID);
                return STATUS_PORT_DISCONNECTED;
            }

            ReplyPort = QueuePort;

            if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CLIENT_PORT {
                (*Message).PortContext = (*QueuePort).PortContext;
            }

            if ((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_COMMUNICATION_PORT {
                ConnectionPort = (*Port).ConnectionPort;
                QueuePort = ConnectionPort;
                if ConnectionPort.is_null() {
                    LpcpFreeToPortZone(Message, LPCP_LOCK_HELD | LPCP_LOCK_RELEASE);
                    ObDereferenceObject(Port as PVOID);
                    return STATUS_PORT_DISCONNECTED;
                }
            }

            if !ConnectionPort.is_null() {
                ObReferenceObject(ConnectionPort as PVOID);
            }
        } else {
            QueuePort = Port;
            ReplyPort = Port;
        }

        (*Message).RepliedToThread = core::ptr::null_mut();
        (*Message).SenderPort = Port;

        (*Message).Request.MessageId = LpcpNextMessageId;
        LpcpNextMessageId = LpcpNextMessageId.wrapping_add(1);
        if LpcpNextMessageId == 0 {
            LpcpNextMessageId = 1;
        }
        (*Message).Request.__bindgen_anon_2.CallbackId = 0;

        (*Thread).LpcReplyMessageId = (*Message).Request.MessageId;
        (*Thread).__bindgen_anon_5.LpcReplyMessage = core::ptr::null_mut();

        InsertTailList(
            core::ptr::addr_of_mut!((*QueuePort).MsgQueue.ReceiveHead),
            core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry),
        );
        InsertTailList(
            core::ptr::addr_of_mut!((*ReplyPort).LpcReplyChainHead),
            core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain),
        );
        LpcpSetPortToThread(Thread, Port);

        KeEnterCriticalRegion();
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        Semaphore = (*QueuePort).MsgQueue.Semaphore as PKSEMAPHORE;

        if ((*QueuePort).Flags & LPCP_WAITABLE_PORT) != 0 {
            KeSetEvent(core::ptr::addr_of_mut!((*QueuePort).WaitEvent) as PVOID, IO_NO_INCREMENT, 0u8);
        }
    }

    LpcpCompleteWait(Semaphore as PVOID);
    KeLeaveCriticalRegion();

    LpcpReplyWait(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID, PreviousMode);

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    let Message = LpcpGetMessageFromThread(Thread);
    (*Thread).__bindgen_anon_5.LpcReplyMessage = core::ptr::null_mut();
    (*Thread).LpcReplyMessageId = 0;

    if !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain)) {
        RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain));
        InitializeListHead(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain));
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if Status == STATUS_SUCCESS {
        if !Message.is_null() {
            // FAULT-PROTECTED (send.c:1000-1013), and its handler's shape is
            // DELIBERATELY DIFFERENT from the other two sites in this file.
            // The C:
            //     _SEH2_TRY { LpcpMoveMessage(LpcReply, ...); }
            //     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            //     {
            //         DPRINT1("Got exception!\n");
            //         Status = _SEH2_GetExceptionCode();
            //     }
            //     _SEH2_END;
            // -- NO _SEH2_YIELD(return ...) in the handler. It only sets
            // Status and FALLS THROUGH into the message-disposal code right
            // below (LpcpSaveDataInfoMessage / LpcpFreeToPortZone). This copy
            // writes INTO user memory (LpcReply) from a port-zone message
            // that must still be disposed of either way -- an early `return`
            // here, unlike at the other two sites, would LEAK the port-zone
            // message. So: capture the checked call's status into `Status`
            // and keep going into the same disposal logic the success path
            // uses; do NOT turn this into a `return Err(...)`.
            Status = LpcpMoveMessageChecked(
                LpcReply,
                core::ptr::addr_of_mut!((*Message).Request),
                core::ptr::addr_of_mut!((*Message).Request).offset(1) as PVOID,
                0,
                core::ptr::null_mut(),
            );

            if (LpcpGetMessageType(core::ptr::addr_of_mut!((*Message).Request)) == LPC_REQUEST)
                && ((*Message).Request.u2.s2.DataInfoOffset != 0)
            {
                LpcpSaveDataInfoMessage(Port, Message, 0);
            } else {
                LpcpFreeToPortZone(Message, 0);
            }
        } else {
            Status = STATUS_LPC_REPLY_LOST;
        }
    } else {
        if !Message.is_null() {
            LpcpFreeToPortZone(Message, 0);
        }
    }

    ObDereferenceObject(Port as PVOID);
    if !ConnectionPort.is_null() {
        ObDereferenceObject(ConnectionPort as PVOID);
    }
    return Status;
}
