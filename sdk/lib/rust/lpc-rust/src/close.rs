use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _ETHREAD, PETHREAD, PEPROCESS, PLPCP_MESSAGE, PLPCP_CONNECTION_MESSAGE,
    PLPCP_PORT_OBJECT, PLPCP_NONPAGED_PORT_QUEUE, HANDLE,
    _LPCP_MESSAGE, _LPCP_NONPAGED_PORT_QUEUE, _EPROCESS,
    ACCESS_MASK, LARGE_INTEGER, CLIENT_DIED_MSG,
    PORT_MESSAGE, PPORT_MESSAGE,
    LpcpGetMessageFromThread, LpcRequestPort, MmUnmapViewOfSection,
    PsGetCurrentThread, PsGetCurrentProcessId, KeDelayExecutionThread,
    KeReadStateSemaphore, KeReleaseSemaphore,
    _LPC_TYPE_LPC_CONNECTION_REQUEST, _LPC_TYPE_LPC_PORT_CLOSED,
    _SECURITY_CLIENT_CONTEXT,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    static mut LpcpLock: KGUARDED_MUTEX;
    static mut LpcpMessagesLookaside: ntkrnl_sys::ddk::PAGED_LOOKASIDE_LIST;
}
extern "system" {
    fn PsGetCurrentProcess() -> PEPROCESS;
    fn SeDeleteClientSecurity(ClientContext: *mut _SECURITY_CLIENT_CONTEXT);
}
extern "system" {
    fn ExFreeToPagedLookasideList(Lookaside: *mut ntkrnl_sys::ddk::PAGED_LOOKASIDE_LIST, Entry: PVOID);
}

const LPCP_LOCK_HELD: ULONG = 1;
const LPCP_LOCK_RELEASE: ULONG = 2;

const LPCP_PORT_TYPE_MASK: ULONG = 0x0000000F;
const LPCP_UNCONNECTED_PORT: ULONG = 0x00000002;
const LPCP_CONNECTION_PORT: ULONG = 0x00000001;
const LPCP_COMMUNICATION_PORT: ULONG = 0x00000003;
const LPCP_CLIENT_PORT: ULONG = 0x00000004;
const LPCP_NAME_DELETED: ULONG = 0x40000000;
const LPCP_SECURITY_DYNAMIC: ULONG = 0x80000000;

fn CONTAINING_RECORD<T>(address: PLIST_ENTRY, offset: usize) -> *mut T {
    (address as usize - offset) as *mut T
}

#[no_mangle]
pub unsafe extern "system" fn LpcExitThread(Thread: PETHREAD) {
    let Thread = opaque_ptr(Thread);
    let mut Message: PLPCP_MESSAGE;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if !IsListEmpty(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain)) {
        RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain));
    }

    (*Thread).__bindgen_anon_9.__bindgen_anon_1.set_LpcExitThreadCalled(1);
    (*Thread).LpcReplyMessageId = 0;

    Message = LpcpGetMessageFromThread(Thread);
    if !Message.is_null() {
        // C original (ntoskrnl/lpc/close.c): `/* FIXME: TODO */ ASSERT(FALSE);`
        // — the "thread exits still holding a reply message" case is an
        // unimplemented ReactOS TODO. ASSERT(FALSE) is DBG-gated, so the FREE
        // build (which is what ships and boots) compiles it out and continues,
        // leaking the message. The translation turned it into an UNCONDITIONAL
        // DbgBreakPoint() (the §9.65 anti-pattern: a debug macro rendered as an
        // always-on call) — on arm64, under the dense LPC traffic of full service
        // startup, a thread does exit with a pending message here, and the break
        // crashed the system (BUGCHECK during PspExitThread). Match the free
        // build: do not break. (Proper reclaim is the same TODO upstream carries.)
        let _ = Message;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
}

#[no_mangle]
pub unsafe extern "system" fn LpcpFreeToPortZone(
    Message: PLPCP_MESSAGE,
    LockFlags: ULONG,
) {
    let mut ConnectMessage: PLPCP_CONNECTION_MESSAGE;
    let mut ClientPort: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let LockHeld: BOOLEAN = if (LockFlags & LPCP_LOCK_HELD) != 0 { 1 } else { 0 };
    let ReleaseLock: BOOLEAN = if (LockFlags & LPCP_LOCK_RELEASE) != 0 { 1 } else { 0 };

    if LockHeld == 0 {
        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry)) {
        RemoveEntryList(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));
        InitializeListHead(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));
    }

    if !(*Message).RepliedToThread.is_null() {
        Thread = (*Message).RepliedToThread;
        (*Message).RepliedToThread = core::ptr::null_mut();
    }

    if (*Message).Request.u2.s2.Type == _LPC_TYPE_LPC_CONNECTION_REQUEST as i16 {
        ConnectMessage = (Message as usize + core::mem::size_of::<_LPCP_MESSAGE>()) as PLPCP_CONNECTION_MESSAGE;

        ClientPort = (*ConnectMessage).ClientPort;
        if !ClientPort.is_null() {
            (*ConnectMessage).ClientPort = core::ptr::null_mut();
        }
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if !Thread.is_null() {
        ObDereferenceObject(Thread as PVOID);
    }
    if !ClientPort.is_null() {
        ObDereferenceObject(ClientPort as PVOID);
    }

    ExFreeToPagedLookasideList(core::ptr::addr_of_mut!(LpcpMessagesLookaside), Message as PVOID);

    if (LockHeld != 0) && (ReleaseLock == 0) {
        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
    }
}

#[no_mangle]
pub unsafe extern "system" fn LpcpDestroyPortQueue(
    Port: PLPCP_PORT_OBJECT,
    Destroy: BOOLEAN,
) {
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut Thread: PETHREAD;
    let mut Message: PLPCP_MESSAGE;
    let mut ConnectionPort: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut ConnectMessage: PLPCP_CONNECTION_MESSAGE;
    let mut MessageQueue: PLPCP_NONPAGED_PORT_QUEUE;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if (((*Port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_UNCONNECTED_PORT) &&
       !(*Port).ConnectedPort.is_null() {
        (*(*Port).ConnectedPort).ConnectedPort = core::ptr::null_mut();
        ConnectionPort = (*(*Port).ConnectedPort).ConnectionPort;
        if !ConnectionPort.is_null() {
            (*(*Port).ConnectedPort).ConnectionPort = core::ptr::null_mut();
        }
    }

    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CONNECTION_PORT {
        (*Port).Flags |= LPCP_NAME_DELETED;
    }

    ListHead = core::ptr::addr_of_mut!((*Port).LpcReplyChainHead);
    NextEntry = (*ListHead).Flink;
    while !NextEntry.is_null() && (NextEntry != ListHead) {
        Thread = CONTAINING_RECORD::<_ETHREAD>(
            NextEntry,
            core::mem::offset_of!(_ETHREAD, __bindgen_anon_1)
        );

        if (*Thread).__bindgen_anon_9.__bindgen_anon_1.LpcExitThreadCalled() != 0 {
            break;
        }

        NextEntry = (*NextEntry).Flink;

        RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain));
        InitializeListHead(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.LpcReplyChain));

        if KeReadStateSemaphore(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_4.LpcReplySemaphore)) == 0 {
            Message = LpcpGetMessageFromThread(Thread);
            if !Message.is_null() {
                if (*Message).Request.u2.s2.Type == _LPC_TYPE_LPC_CONNECTION_REQUEST as i16 {
                    ConnectMessage = (Message as usize + core::mem::size_of::<_LPCP_MESSAGE>()) as PLPCP_CONNECTION_MESSAGE;

                    if !(*ConnectMessage).SectionToMap.is_null() {
                        ObDereferenceObject((*ConnectMessage).SectionToMap);
                    }
                }

                (*Thread).__bindgen_anon_5.LpcReplyMessage = core::ptr::null_mut();

                LpcpFreeToPortZone(Message, LPCP_LOCK_HELD);
                NextEntry = (*Port).LpcReplyChainHead.Flink;
            }

            (*Thread).LpcReplyMessageId = 0;
            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*Thread).__bindgen_anon_4.LpcReplySemaphore),
                0,
                1,
                0u8
            );
        }
    }

    InitializeListHead(core::ptr::addr_of_mut!((*Port).LpcReplyChainHead));

    while !(*Port).MsgQueue.ReceiveHead.Flink.is_null() &&
          !IsListEmpty(core::ptr::addr_of_mut!((*Port).MsgQueue.ReceiveHead)) {
        Message = CONTAINING_RECORD::<_LPCP_MESSAGE>(
            (*Port).MsgQueue.ReceiveHead.Flink,
            core::mem::offset_of!(_LPCP_MESSAGE, __bindgen_anon_1)
        );

        RemoveEntryList(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));
        InitializeListHead(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));

        LpcpFreeToPortZone(Message, LPCP_LOCK_HELD);
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    if !ConnectionPort.is_null() {
        ObDereferenceObject(ConnectionPort as PVOID);
    }

    if Destroy != 0 {
        if !(*Port).MsgQueue.Semaphore.is_null() {
            MessageQueue = CONTAINING_RECORD::<_LPCP_NONPAGED_PORT_QUEUE>(
                (*Port).MsgQueue.Semaphore as PLIST_ENTRY,
                core::mem::offset_of!(_LPCP_NONPAGED_PORT_QUEUE, Semaphore)
            );
            ExFreePoolWithTag(MessageQueue as PVOID, u32::from_be_bytes(*b"troP"));
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn LpcpClosePort(
    _Process: PEPROCESS,
    Object: PVOID,
    _GrantedAccess: ACCESS_MASK,
    _ProcessHandleCount: ULONG_PTR,
    SystemHandleCount: ULONG_PTR,
) {
    let _Process = opaque_ptr(_Process);
    let Object = opaque_ptr(Object);
    let Port: PLPCP_PORT_OBJECT = Object as PLPCP_PORT_OBJECT;

    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CONNECTION_PORT {
        match SystemHandleCount {
            0 => {
                LpcpDestroyPortQueue(Port, 1u8);
            }
            1 => {
                LpcpDestroyPortQueue(Port, 0u8);
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn LpcpFreePortClientSecurity(Port: PLPCP_PORT_OBJECT) {
    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CLIENT_PORT {
        if ((*Port).Flags & LPCP_SECURITY_DYNAMIC) == 0 {
            if !(*Port).StaticSecurity.ClientToken.is_null() {
                SeDeleteClientSecurity(core::ptr::addr_of_mut!((*Port).StaticSecurity));
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn LpcpDeletePort(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut Thread: PETHREAD;
    let Port: PLPCP_PORT_OBJECT = ObjectBody as PLPCP_PORT_OBJECT;
    let mut ConnectionPort: PLPCP_PORT_OBJECT;
    let mut Message: PLPCP_MESSAGE;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut Pid: HANDLE;
    let mut ClientDiedMsg: CLIENT_DIED_MSG = core::mem::zeroed();

    Timeout.QuadPart = -1000000;

    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_COMMUNICATION_PORT {
        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

        Thread = (*Port).ClientThread;
        if !Thread.is_null() {
            (*Port).ClientThread = core::ptr::null_mut();

            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
            ObDereferenceObject(Thread as PVOID);
        } else {
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
        }
    }

    if ((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CLIENT_PORT {
        ClientDiedMsg.h.u1.s1.TotalLength = core::mem::size_of::<CLIENT_DIED_MSG>() as i16;
        ClientDiedMsg.h.u1.s1.DataLength = core::mem::size_of::<LARGE_INTEGER>() as i16;
        ClientDiedMsg.h.u2.ZeroInit = 0;
        ClientDiedMsg.h.u2.s2.Type = _LPC_TYPE_LPC_PORT_CLOSED as i16;
        ClientDiedMsg.CreateTime = (*(PsGetCurrentProcess() as *mut _EPROCESS)).CreateTime;

        loop {
            if LpcRequestPort(Port as PVOID, core::ptr::addr_of_mut!(ClientDiedMsg.h)) != STATUS_NO_MEMORY {
                break;
            }

            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of_mut!(Timeout));
        }
    }

    LpcpDestroyPortQueue(Port, 1u8);

    if !(*Port).ClientSectionBase.is_null() || !(*Port).ServerSectionBase.is_null() {
        if !(*Port).ClientSectionBase.is_null() {
            MmUnmapViewOfSection(
                (*Port).__bindgen_anon_1.MappingProcess,
                (*Port).ClientSectionBase
            );
        }

        if !(*Port).ServerSectionBase.is_null() {
            MmUnmapViewOfSection(
                (*Port).__bindgen_anon_1.MappingProcess,
                (*Port).ServerSectionBase
            );
        }

        ObDereferenceObject((*Port).__bindgen_anon_1.MappingProcess as PVOID);
        (*Port).__bindgen_anon_1.MappingProcess = core::ptr::null_mut();
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

    ConnectionPort = (*Port).ConnectionPort;
    if !ConnectionPort.is_null() {
        Pid = PsGetCurrentProcessId();

        ListHead = core::ptr::addr_of_mut!((*ConnectionPort).LpcDataInfoChainHead);
        NextEntry = (*ListHead).Flink;
        while NextEntry != ListHead {
            Message = CONTAINING_RECORD::<_LPCP_MESSAGE>(
                NextEntry,
                core::mem::offset_of!(_LPCP_MESSAGE, __bindgen_anon_1)
            );
            NextEntry = (*NextEntry).Flink;

            if Port == ConnectionPort {
                RemoveEntryList(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));
                InitializeListHead(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));
                LpcpFreeToPortZone(Message, LPCP_LOCK_HELD);

                NextEntry = (*ListHead).Flink;
            } else if ((*Message).Request.__bindgen_anon_1.ClientId.UniqueProcess == Pid) &&
                      (((*Message).SenderPort == Port) ||
                       ((*Message).SenderPort == (*Port).ConnectedPort) ||
                       ((*Message).SenderPort == ConnectionPort)) {
                RemoveEntryList(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));
                InitializeListHead(core::ptr::addr_of_mut!((*Message).__bindgen_anon_1.Entry));
                LpcpFreeToPortZone(Message, LPCP_LOCK_HELD);

                NextEntry = (*ListHead).Flink;
            }
        }

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));

        if ConnectionPort != Port {
            ObDereferenceObject(ConnectionPort as PVOID);
        }

        if (((*Port).Flags & LPCP_PORT_TYPE_MASK) == LPCP_CONNECTION_PORT) &&
           !(*ConnectionPort).__bindgen_anon_1.ServerProcess.is_null() {
            ObDereferenceObject((*ConnectionPort).__bindgen_anon_1.ServerProcess as PVOID);
            (*ConnectionPort).__bindgen_anon_1.ServerProcess = core::ptr::null_mut();
        }
    } else {
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(LpcpLock));
    }

    LpcpFreePortClientSecurity(Port);
}
