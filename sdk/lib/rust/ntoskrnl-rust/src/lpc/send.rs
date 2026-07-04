use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    HANDLE, KPROCESSOR_MODE,
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
extern "C" {
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
        ProbeForRead(LpcRequest as PVOID, core::mem::size_of::<PORT_MESSAGE>(), core::mem::size_of::<ULONG>() as ULONG);
        CapturedLpcRequest = core::ptr::read_volatile(LpcRequest);
    } else {
        CapturedLpcRequest = *LpcRequest;
    }

    MessageType = (*LpcRequest).u2.s2.Type as ULONG | LPC_DATAGRAM;

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

    LpcpMoveMessage(
        core::ptr::addr_of_mut!((*Message).Request),
        core::ptr::addr_of_mut!(CapturedLpcRequest),
        LpcRequest.offset(1) as PVOID,
        MessageType,
        core::ptr::addr_of_mut!((*Thread).Cid) as PVOID,
    );

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
        ProbeForRead(LpcRequest as PVOID, core::mem::size_of::<PORT_MESSAGE>(), core::mem::size_of::<ULONG>() as ULONG);
        CapturedLpcRequest = core::ptr::read_volatile(LpcRequest);

        ProbeForWrite(LpcReply as PVOID, core::mem::size_of::<PORT_MESSAGE>(), core::mem::size_of::<ULONG>() as ULONG);

        Status = LpcpVerifyMessageDataInfo(LpcRequest, core::ptr::addr_of_mut!(NumberOfDataEntries));
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
        if (*LpcRequest).u2.s2.DataInfoOffset != 0 {
            DataInfo = LpcpGetDataInfoFromMessage(LpcRequest);
            if (*DataInfo).NumberOfEntries != NumberOfDataEntries {
                LpcpFreeToPortZone(Message, 0);
                ObDereferenceObject(Port as PVOID);
                return STATUS_INVALID_PARAMETER;
            }
        }

        LpcpMoveMessage(
            core::ptr::addr_of_mut!((*Message).Request),
            core::ptr::addr_of_mut!(CapturedLpcRequest),
            LpcRequest.offset(1) as PVOID,
            MessageType,
            core::ptr::addr_of_mut!((*Thread).Cid) as PVOID,
        );

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
            LpcpMoveMessage(
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
