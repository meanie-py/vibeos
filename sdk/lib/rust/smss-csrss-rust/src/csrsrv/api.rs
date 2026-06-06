use super::srv_inlines::*;
use ntdll_sys::nt::*;
pub static mut CsrClientThreadSetup: Option<unsafe extern "system" fn() -> BOOLEAN> = None;
#[no_mangle]
pub static mut CsrApiPortName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
pub static mut CsrpStaticThreadCount: ULONG = 0;
pub static mut CsrpDynamicThreadTotal: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn CsrCallServerFromServer(
    ReceiveMsg: PCSR_API_MESSAGE,
    ReplyMsg: PCSR_API_MESSAGE,
) -> NTSTATUS {
    let mut ServerId: ULONG = 0;
    let mut ServerDll: PCSR_SERVER_DLL = core::ptr::null_mut();
    let mut ApiId: ULONG = 0;
    let mut ReplyCode: CSR_REPLY_CODE = CsrReplyImmediately;

    ServerId = (((*ReceiveMsg).__bindgen_anon_1.__bindgen_anon_1.ApiNumber) >> 16) & 0xFFFF;

    if (ServerId >= CSR_SERVER_DLL_MAX) || {
        ServerDll = *CsrLoadedServerDll.as_ptr().add(ServerId as usize);
        ServerDll.is_null()
    } {
        (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_ILLEGAL_FUNCTION;
        return STATUS_ILLEGAL_FUNCTION;
    } else {
        ApiId = (((*ReceiveMsg).__bindgen_anon_1.__bindgen_anon_1.ApiNumber) & 0xFFFF)
            .wrapping_sub((*ServerDll).ApiBase);

        if (ApiId >= (*ServerDll).HighestApiSupported)
            || (!(*ServerDll).ValidTable.is_null()
                && *(*ServerDll).ValidTable.add(ApiId as usize) == 0)
        {
            (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_ILLEGAL_FUNCTION;
            return STATUS_ILLEGAL_FUNCTION;
        }
    }

    let dispatch_fn = (*ServerDll).DispatchTable.add(ApiId as usize);
    if let Some(func) = *dispatch_fn {
        (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = func(ReceiveMsg, &raw mut ReplyCode);
    } else {
        (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_ACCESS_VIOLATION;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrApiHandleConnectionRequest(
    ApiMessage: PCSR_API_MESSAGE,
) -> NTSTATUS {
    let mut CsrThread: PCSR_THREAD = core::ptr::null_mut();
    let mut CsrProcess: PCSR_PROCESS = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let ConnectInfo: *mut CSR_API_CONNECTINFO =
        &raw mut (*ApiMessage).__bindgen_anon_1.ConnectionInfo;
    let mut AllowConnection: BOOLEAN = 0u8;
    let mut RemotePortView: REMOTE_PORT_VIEW = core::mem::zeroed();
    let mut ServerPort: HANDLE = core::ptr::null_mut();

    CsrAcquireProcessLock();

    CsrThread = CsrLocateThreadByClientId(
        core::ptr::null_mut(),
        &raw mut (*ApiMessage).Header.__bindgen_anon_1.ClientId,
    );

    if !CsrThread.is_null() {
        CsrProcess = (*CsrThread).Process;
        if !CsrProcess.is_null() {
            CsrLockedReferenceProcess(CsrProcess);

            Status = CsrSrvAttachSharedSection(CsrProcess, ConnectInfo);
            if Status >= 0 {
                (*ConnectInfo).DebugFlags = CsrDebug;
                AllowConnection = 1u8;
            }

            CsrLockedDereferenceProcess(CsrProcess);
        }
    }

    CsrReleaseProcessLock();

    RemotePortView.Length = core::mem::size_of::<REMOTE_PORT_VIEW>() as ULONG;
    RemotePortView.ViewSize = 0;
    RemotePortView.ViewBase = core::ptr::null_mut();

    (*ConnectInfo).ServerProcessId = (*NtCurrentTeb()).ClientId.UniqueProcess;

    Status = NtAcceptConnectPort(
        &raw mut ServerPort,
        if AllowConnection != 0 {
            (*CsrProcess).SequenceNumber as PVOID
        } else {
            core::ptr::null_mut()
        },
        &raw mut (*ApiMessage).Header,
        AllowConnection,
        core::ptr::null_mut(),
        &raw mut RemotePortView,
    );

    if Status >= 0 && AllowConnection != 0 {
        (*CsrProcess).ClientPort = ServerPort;
        (*CsrProcess).ClientViewBase = RemotePortView.ViewBase as ULONG_PTR;
        (*CsrProcess).ClientViewBounds =
            (RemotePortView.ViewBase as ULONG_PTR) + (RemotePortView.ViewSize as ULONG_PTR);

        Status = NtCompleteConnectPort(ServerPort);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrpCheckRequestThreads() -> NTSTATUS {
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();
    let Status: NTSTATUS;

    if InterlockedDecrementUL(&raw mut CsrpStaticThreadCount) == 0 {
        if CsrpDynamicThreadTotal < CsrMaxApiRequestThreads {
            Status = RtlCreateUserThread(
                NtCurrentProcess(),
                core::ptr::null_mut(),
                1u8,
                0,
                0,
                0,
                core::mem::transmute(CsrApiRequestThread as usize),
                core::ptr::null_mut(),
                &raw mut hThread,
                &raw mut ClientId,
            );

            if Status >= 0 {
                InterlockedIncrementUL(&raw mut CsrpStaticThreadCount);
                InterlockedIncrementUL(&raw mut CsrpDynamicThreadTotal);

                if CsrAddStaticServerThread(hThread, &raw mut ClientId, CsrThreadIsServerThread)
                    != core::ptr::null_mut()
                {
                    NtResumeThread(hThread, core::ptr::null_mut());
                } else {
                    InterlockedDecrementUL(&raw mut CsrpStaticThreadCount);
                    InterlockedDecrementUL(&raw mut CsrpDynamicThreadTotal);
                    NtTerminateThread(hThread, 0);
                    NtClose(hThread);
                    return STATUS_UNSUCCESSFUL;
                }
            }
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrApiRequestThread(Parameter: PVOID) -> NTSTATUS {
    let Teb: *mut _TEB = NtCurrentTeb();
    let CurrentThread: PCSR_THREAD;
    let mut CsrThread: PCSR_THREAD;
    let mut Status: NTSTATUS;
    let mut ReplyCode: CSR_REPLY_CODE = CsrReplyImmediately;
    let mut ReplyMsg: PCSR_API_MESSAGE;
    let mut ReceiveMsg: CSR_API_MESSAGE = core::mem::zeroed();
    let mut CsrProcess: PCSR_PROCESS = core::ptr::null_mut();
    let mut HardErrorMsg: PHARDERROR_MSG;
    let mut PortContext: PVOID = core::ptr::null_mut();
    let mut ServerDll: PCSR_SERVER_DLL;
    let mut ClientDiedMsg: PCLIENT_DIED_MSG;
    let mut DebugMessage: PDBGKM_MSG;
    let mut ServerId: ULONG = 0;
    let mut ApiId: ULONG = 0;
    let mut MessageType: ULONG = 0;
    let mut i: ULONG;
    let mut ReplyPort: HANDLE;

    crate::dbg_print!(b"[CSRSRV] ApiRequestThread: enter param=%p\n\0", Parameter);

    ReplyMsg = core::ptr::null_mut();
    ReplyPort = CsrApiPort;

    #[cfg(target_arch = "aarch64")]
    {
        // ARM64: skip CsrConnectToUser
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let mut connect_attempts: u32 = 0;
        while CsrConnectToUser().is_null() {
            connect_attempts += 1;
            crate::dbg_print!(
                b"[CSRSRV] ApiRequestThread: CsrConnectToUser null, attempt %u\n\0",
                connect_attempts
            );
            let mut TimeOut: LARGE_INTEGER = core::mem::zeroed();
            TimeOut.QuadPart = -30 * 1000 * 1000 * 10;
            (*Teb).Win32ClientInfo[0] = 0;
            NtDelayExecution(0u8, &raw mut TimeOut);
        }
        crate::dbg_print!(b"[CSRSRV] ApiRequestThread: CsrConnectToUser OK\n\0");
    }

    CurrentThread = (*Teb).CsrClientThread as PCSR_THREAD;

    if !Parameter.is_null() {
        Status = NtSetEvent(Parameter as HANDLE, core::ptr::null_mut());
        InterlockedIncrementUL(&raw mut CsrpStaticThreadCount);
        InterlockedIncrementUL(&raw mut CsrpDynamicThreadTotal);
        crate::dbg_print!(
            b"[CSRSRV] ApiRequestThread: NtSetEvent st=0x%lx, entering loop\n\0",
            Status
        );
    }

    let mut loop_iter: u32 = 0;
    loop {
        loop_iter += 1;
        if loop_iter <= 5 {
            crate::dbg_print!(
                b"[CSRSRV] ApiReqThr: iter %u, calling NtReplyWaitReceivePort port=%p\n\0",
                loop_iter,
                ReplyPort
            );
        }
        (*Teb).RealClientId = (*Teb).ClientId;

        Status = NtReplyWaitReceivePort(
            ReplyPort,
            &raw mut PortContext,
            if !ReplyMsg.is_null() {
                &raw mut (*ReplyMsg).Header
            } else {
                core::ptr::null_mut()
            },
            &raw mut ReceiveMsg.Header,
        );

        if loop_iter <= 5 {
            crate::dbg_print!(
                b"[CSRSRV] ApiReqThr: iter %u, NtReplyWaitReceivePort st=0x%lx\n\0",
                loop_iter,
                Status
            );
        }
        if Status != STATUS_SUCCESS {
            if Status < 0 {
                ReplyMsg = core::ptr::null_mut();
                ReplyPort = CsrApiPort;
                continue;
            } else {
                continue;
            }
        }

        (*Teb).RealClientId = ReceiveMsg.Header.__bindgen_anon_1.ClientId;
        MessageType = ReceiveMsg.Header.u2.s2.Type as ULONG;

        if MessageType == LPC_CONNECTION_REQUEST {
            CsrApiHandleConnectionRequest(&raw mut ReceiveMsg);
            ReplyMsg = core::ptr::null_mut();
            ReplyPort = CsrApiPort;
            continue;
        }

        CsrAcquireProcessLock();

        CsrThread = CsrLocateThreadByClientId(
            &raw mut CsrProcess,
            &raw mut ReceiveMsg.Header.__bindgen_anon_1.ClientId,
        );

        if CsrThread.is_null() {
            CsrReleaseProcessLock();

            if MessageType == LPC_EXCEPTION {
                ReplyMsg = &raw mut ReceiveMsg;
                ReplyPort = CsrApiPort;
                (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = DBG_CONTINUE;
            } else if MessageType == LPC_PORT_CLOSED || MessageType == LPC_CLIENT_DIED {
                ReplyMsg = core::ptr::null_mut();
                ReplyPort = CsrApiPort;
            } else if MessageType == LPC_ERROR_EVENT {
                HardErrorMsg = &raw mut ReceiveMsg as PHARDERROR_MSG;
                (*HardErrorMsg).Response = ResponseNotHandled;

                CsrpCheckRequestThreads();
                if CsrpStaticThreadCount != 0 {
                    i = 0;
                    while i < CSR_SERVER_DLL_MAX {
                        ServerDll = *CsrLoadedServerDll.as_ptr().add(i as usize);
                        if !ServerDll.is_null() && (*ServerDll).HardErrorCallback.is_some() {
                            ((*ServerDll).HardErrorCallback.unwrap())(
                                core::ptr::null_mut(),
                                HardErrorMsg,
                            );
                            if (*HardErrorMsg).Response != ResponseNotHandled {
                                break;
                            }
                        }
                        i += 1;
                    }
                }

                InterlockedIncrementUL(&raw mut CsrpStaticThreadCount);

                if (*HardErrorMsg).Response == 0xFFFFFFFF {
                    ReplyMsg = core::ptr::null_mut();
                    ReplyPort = CsrApiPort;
                } else {
                    ReplyMsg = &raw mut ReceiveMsg;
                    ReplyPort = CsrApiPort;
                }
            } else if MessageType == LPC_REQUEST {
                ReplyMsg = &raw mut ReceiveMsg;
                ReplyPort = CsrApiPort;
                (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_ILLEGAL_FUNCTION;
            } else if MessageType == LPC_DATAGRAM {
                ServerId =
                    ((ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber) >> 16) & 0xFFFF;
                ServerDll = core::ptr::null_mut();

                if (ServerId >= CSR_SERVER_DLL_MAX) || {
                    ServerDll = *CsrLoadedServerDll.as_ptr().add(ServerId as usize);
                    ServerDll.is_null()
                } {
                    ReplyMsg = core::ptr::null_mut();
                    ReplyPort = CsrApiPort;
                    continue;
                }

                ApiId = ((ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber) & 0xFFFF)
                    .wrapping_sub((*ServerDll).ApiBase);

                if ApiId >= (*ServerDll).HighestApiSupported {
                    ReplyPort = CsrApiPort;
                    ReplyMsg = core::ptr::null_mut();
                    continue;
                }

                ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_SUCCESS;

                CsrpCheckRequestThreads();
                ReplyMsg = core::ptr::null_mut();
                ReplyPort = CsrApiPort;

                let dispatch_fn = (*ServerDll).DispatchTable.add(ApiId as usize);
                if let Some(func) = *dispatch_fn {
                    func(&raw mut ReceiveMsg, &raw mut ReplyCode);
                }

                InterlockedIncrementUL(&raw mut CsrpStaticThreadCount);
            } else {
                ReplyMsg = core::ptr::null_mut();
                ReplyPort = CsrApiPort;
            }

            continue;
        }

        if MessageType != LPC_REQUEST {
            if MessageType == LPC_CLIENT_DIED {
                ClientDiedMsg = &raw mut ReceiveMsg as PCLIENT_DIED_MSG;
                if (*ClientDiedMsg).CreateTime.QuadPart == (*CsrThread).CreateTime.QuadPart {
                    ReplyPort = (*(*CsrThread).Process).ClientPort;
                    CsrLockedReferenceThread(CsrThread);
                    CsrDestroyThread(&raw mut ReceiveMsg.Header.__bindgen_anon_1.ClientId);

                    if (*CsrProcess).ThreadCount == 1 {
                        CsrDestroyProcess(&raw mut (*CsrThread).ClientId, 0);
                    }

                    CsrLockedDereferenceThread(CsrThread);
                }

                CsrReleaseProcessLock();
                ReplyMsg = core::ptr::null_mut();
                ReplyPort = CsrApiPort;
                continue;
            }

            CsrLockedReferenceThread(CsrThread);
            CsrReleaseProcessLock();

            if MessageType == LPC_EXCEPTION {
                NtTerminateProcess((*CsrProcess).ProcessHandle, STATUS_ABANDONED);
                CsrDestroyProcess(
                    &raw mut ReceiveMsg.Header.__bindgen_anon_1.ClientId,
                    STATUS_ABANDONED,
                );
                DebugMessage = &raw mut ReceiveMsg as PDBGKM_MSG;
                (*DebugMessage).ReturnedStatus = DBG_CONTINUE;
                ReplyMsg = &raw mut ReceiveMsg;
                ReplyPort = CsrApiPort;
                CsrDereferenceThread(CsrThread);
            } else if MessageType == LPC_ERROR_EVENT {
                HardErrorMsg = &raw mut ReceiveMsg as PHARDERROR_MSG;
                (*HardErrorMsg).Response = ResponseNotHandled;

                CsrpCheckRequestThreads();
                if CsrpStaticThreadCount != 0 {
                    i = 0;
                    while i < CSR_SERVER_DLL_MAX {
                        ServerDll = *CsrLoadedServerDll.as_ptr().add(i as usize);
                        if !ServerDll.is_null() && (*ServerDll).HardErrorCallback.is_some() {
                            ((*ServerDll).HardErrorCallback.unwrap())(CsrThread, HardErrorMsg);
                            if (*HardErrorMsg).Response != ResponseNotHandled {
                                break;
                            }
                        }
                        i += 1;
                    }
                }

                InterlockedIncrementUL(&raw mut CsrpStaticThreadCount);

                if (*HardErrorMsg).Response == 0xFFFFFFFF {
                    ReplyMsg = core::ptr::null_mut();
                    ReplyPort = CsrApiPort;
                } else {
                    CsrDereferenceThread(CsrThread);
                    ReplyMsg = &raw mut ReceiveMsg;
                    ReplyPort = CsrApiPort;
                }
            } else {
                CsrDereferenceThread(CsrThread);
                ReplyMsg = core::ptr::null_mut();
            }

            continue;
        }

        CsrLockedReferenceThread(CsrThread);
        CsrReleaseProcessLock();

        let raw_api_number = ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber;
        ServerId = ((raw_api_number) >> 16) & 0xFFFF;
        ServerDll = core::ptr::null_mut();

        crate::dbg_print!(
            b"[CSRSRV] LPC_REQ: ApiNumber=0x%lx ServerId=%lu MsgType=%lu\n\0",
            raw_api_number,
            ServerId,
            MessageType
        );

        if (ServerId >= CSR_SERVER_DLL_MAX) || {
            ServerDll = *CsrLoadedServerDll.as_ptr().add(ServerId as usize);
            ServerDll.is_null()
        } {
            crate::dbg_print!(
                b"[CSRSRV] LPC_REQ: BAD ServerId=%lu (MAX=%lu) ServerDll=%p\n\0",
                ServerId,
                CSR_SERVER_DLL_MAX,
                ServerDll
            );
            ReplyPort = CsrApiPort;
            ReplyMsg = &raw mut ReceiveMsg;
            (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_ILLEGAL_FUNCTION;
            CsrDereferenceThread(CsrThread);
            continue;
        }

        ApiId = ((raw_api_number) & 0xFFFF).wrapping_sub((*ServerDll).ApiBase);

        crate::dbg_print!(
            b"[CSRSRV] LPC_REQ: ServerDll=%p ApiBase=%lu HiApi=%lu ApiId=%lu DispatchTable=%p\n\0",
            ServerDll,
            (*ServerDll).ApiBase,
            (*ServerDll).HighestApiSupported,
            ApiId,
            (*ServerDll).DispatchTable
        );

        if ApiId >= (*ServerDll).HighestApiSupported {
            crate::dbg_print!(
                b"[CSRSRV] LPC_REQ: ApiId %lu >= HiApi %lu, ILLEGAL_FUNCTION\n\0",
                ApiId,
                (*ServerDll).HighestApiSupported
            );
            ReplyPort = CsrApiPort;
            ReplyMsg = &raw mut ReceiveMsg;
            (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_ILLEGAL_FUNCTION;
            CsrDereferenceThread(CsrThread);
            continue;
        }

        ReplyMsg = &raw mut ReceiveMsg;
        ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_SUCCESS;
        ReplyPort = (*(*CsrThread).Process).ClientPort;

        if !ReceiveMsg
            .__bindgen_anon_1
            .__bindgen_anon_1
            .CsrCaptureData
            .is_null()
        {
            if CsrCaptureArguments(CsrThread, &raw mut ReceiveMsg) == 0 {
                CsrDereferenceThread(CsrThread);
                continue;
            }
        }

        CsrpCheckRequestThreads();
        (*Teb).CsrClientThread = CsrThread as PVOID;
        ReplyCode = CsrReplyImmediately;

        let dispatch_fn = (*ServerDll).DispatchTable.add(ApiId as usize);
        let dispatch_target: *mut core::ffi::c_void = *(dispatch_fn as *mut *mut core::ffi::c_void);
        crate::dbg_print!(
            b"[CSRSRV] LPC_REQ: dispatch_fn_slot=%p target=%p\n\0",
            dispatch_fn,
            dispatch_target
        );
        if let Some(func) = *dispatch_fn {
            let st = func(&raw mut ReceiveMsg, &raw mut ReplyCode);
            crate::dbg_print!(
                b"[CSRSRV] LPC_REQ: dispatch returned st=0x%lx ReplyCode=%lu\n\0",
                st,
                ReplyCode
            );
            (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.Status = st;
        } else {
            crate::dbg_print!(b"[CSRSRV] LPC_REQ: dispatch fn is None (NULL slot)\n\0");
        }

        InterlockedIncrementUL(&raw mut CsrpStaticThreadCount);
        (*Teb).CsrClientThread = CurrentThread as PVOID;

        if ReplyCode == CsrReplyAlreadySent {
            if !ReceiveMsg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .CsrCaptureData
                .is_null()
            {
                CsrReleaseCapturedArguments(&raw mut ReceiveMsg);
            }
            ReplyMsg = core::ptr::null_mut();
            ReplyPort = CsrApiPort;
            CsrDereferenceThread(CsrThread);
        } else if ReplyCode == CsrReplyDeadClient {
            NtReplyPort(ReplyPort, &raw mut (*ReplyMsg).Header);
            ReplyMsg = core::ptr::null_mut();
            ReplyPort = CsrApiPort;
            CsrDereferenceThread(CsrThread);
        } else if ReplyCode == CsrReplyPending {
            ReplyMsg = core::ptr::null_mut();
            ReplyPort = CsrApiPort;
        } else {
            if !ReceiveMsg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .CsrCaptureData
                .is_null()
            {
                CsrReleaseCapturedArguments(&raw mut ReceiveMsg);
            }
            CsrDereferenceThread(CsrThread);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CsrApiPortInitialize() -> NTSTATUS {
    let Size: ULONG;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut hRequestEvent: HANDLE = core::ptr::null_mut();
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut ServerThread: PCSR_THREAD;
    crate::dbg_print!(b"[CSRSRV] ApiPortInit: enter\n\0");

    Size = CsrDirectoryName.Length as ULONG + core::mem::size_of::<[u16; 10]>() as ULONG;

    CsrApiPortName.Buffer =
        RtlAllocateHeap(CsrHeap, 0, (Size as usize).try_into().unwrap()) as PWSTR;
    if CsrApiPortName.Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    CsrApiPortName.Length = 0;
    CsrApiPortName.MaximumLength = Size as USHORT;
    RtlAppendUnicodeStringToString(&raw mut CsrApiPortName, &raw mut CsrDirectoryName);
    static BACKSLASH: [u16; 2] = [b'\\' as u16, 0];
    RtlAppendUnicodeToString(&raw mut CsrApiPortName, BACKSLASH.as_ptr());
    static PORT_NAME: [u16; 9] = [
        b'A' as u16,
        b'p' as u16,
        b'i' as u16,
        b'P' as u16,
        b'o' as u16,
        b'r' as u16,
        b't' as u16,
        0,
        0,
    ];
    RtlAppendUnicodeToString(&raw mut CsrApiPortName, PORT_NAME.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut CsrApiPortName;
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreatePort(
        &raw mut CsrApiPort,
        &raw mut ObjectAttributes,
        core::mem::size_of::<CSR_API_CONNECTINFO>() as ULONG,
        core::mem::size_of::<CSR_API_MESSAGE>() as ULONG,
        16 * PAGE_SIZE as ULONG,
    );

    crate::dbg_print!(b"[CSRSRV] ApiPortInit: NtCreatePort st=0x%lx\n\0", Status);
    if Status >= 0 {
        Status = NtCreateEvent(
            &raw mut hRequestEvent,
            EVENT_ALL_ACCESS,
            core::ptr::null_mut(),
            _EVENT_TYPE_SynchronizationEvent,
            0u8,
        );
        crate::dbg_print!(
            b"[CSRSRV] ApiPortInit: NtCreateEvent st=0x%lx h=%p\n\0",
            Status,
            hRequestEvent
        );

        if Status >= 0 {
            Status = RtlCreateUserThread(
                NtCurrentProcess(),
                core::ptr::null_mut(),
                1u8,
                0,
                0,
                0,
                core::mem::transmute(CsrApiRequestThread as usize),
                hRequestEvent as PVOID,
                &raw mut hThread,
                &raw mut ClientId,
            );
            crate::dbg_print!(
                b"[CSRSRV] ApiPortInit: RtlCreateUserThread st=0x%lx h=%p\n\0",
                Status,
                hThread
            );

            if Status >= 0 {
                let added =
                    CsrAddStaticServerThread(hThread, &raw mut ClientId, CsrThreadIsServerThread);
                crate::dbg_print!(
                    b"[CSRSRV] ApiPortInit: CsrAddStaticServerThread -> %p (CsrRootProcess=%p)\n\0",
                    added,
                    CsrRootProcess
                );

                ListHead = &raw mut (*CsrRootProcess).ThreadList;
                NextEntry = (*ListHead).Flink;
                crate::dbg_print!(
                    b"[CSRSRV] ApiPortInit: resume loop start ListHead=%p Flink=%p\n\0",
                    ListHead,
                    NextEntry
                );

                let mut iter_n: u32 = 0;
                while NextEntry != ListHead {
                    ServerThread = ((NextEntry as usize) - core::mem::offset_of!(_CSR_THREAD, Link))
                        as PCSR_THREAD;
                    crate::dbg_print!(
                        b"[CSRSRV] ApiPortInit: iter %u ServerThread=%p Flags=0x%x hT=%p\n\0",
                        iter_n,
                        ServerThread,
                        (*ServerThread).Flags,
                        (*ServerThread).ThreadHandle
                    );

                    Status = NtResumeThread((*ServerThread).ThreadHandle, core::ptr::null_mut());
                    crate::dbg_print!(b"[CSRSRV] ApiPortInit: NtResumeThread st=0x%lx\n\0", Status);

                    if (*ServerThread).Flags & CsrThreadIsServerThread != 0 {
                        Status = NtWaitForSingleObject(hRequestEvent, 0u8, core::ptr::null_mut());
                        crate::dbg_print!(
                            b"[CSRSRV] ApiPortInit: NtWaitForSingleObject st=0x%lx\n\0",
                            Status
                        );
                    }

                    NextEntry = (*NextEntry).Flink;
                    iter_n += 1;
                }

                crate::dbg_print!(
                    b"[CSRSRV] ApiPortInit: resume loop done iters=%u\n\0",
                    iter_n
                );
                NtClose(hRequestEvent);
            }
        }
    }

    crate::dbg_print!(b"[CSRSRV] ApiPortInit: exit st=0x%lx\n\0", Status);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrConnectToUser() -> PCSR_THREAD {
    let mut Status: NTSTATUS;
    let mut DllName: ANSI_STRING = core::mem::zeroed();
    let mut TempName: _UNICODE_STRING = core::mem::zeroed();
    let mut hUser32: HANDLE = core::ptr::null_mut();
    let mut StartupName: ANSI_STRING = core::mem::zeroed();
    let Teb: *mut _TEB = NtCurrentTeb();
    let CsrThread: PCSR_THREAD;
    let Connected: BOOLEAN;

    if CsrClientThreadSetup.is_none() {
        static DLL_STR: [u8; 7] = [b'u', b's', b'e', b'r', b'3', b'2', 0];
        DllName.Buffer = DLL_STR.as_ptr() as *mut i8;
        DllName.Length = 6;
        DllName.MaximumLength = 7;
        RtlAnsiStringToUnicodeString(&raw mut TempName, &raw mut DllName, 1u8);
        Status = LdrGetDllHandle(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut TempName,
            &raw mut hUser32,
        );
        RtlFreeUnicodeString(&raw mut TempName);

        if Status >= 0 {
            static STARTUP_STR: [u8; 18] = [
                b'C', b'l', b'i', b'e', b'n', b't', b'T', b'h', b'r', b'e', b'a', b'd', b'S', b'e',
                b't', b'u', b'p', 0,
            ];
            StartupName.Buffer = STARTUP_STR.as_ptr() as *mut i8;
            StartupName.Length = 17;
            StartupName.MaximumLength = 18;
            Status = LdrGetProcedureAddress(
                hUser32,
                &raw mut StartupName,
                0,
                &raw mut CsrClientThreadSetup as *mut _ as *mut PVOID,
            );
        }
    }

    if let Some(setup_fn) = CsrClientThreadSetup {
        Connected = setup_fn();
    } else {
        Connected = 0u8;
    }

    if Connected == 0 {
        return core::ptr::null_mut();
    }

    CsrAcquireProcessLock();
    CsrThread = CsrLocateThreadInProcess(core::ptr::null_mut(), &raw mut (*Teb).ClientId);
    CsrReleaseProcessLock();
    if !CsrThread.is_null() {
        (*Teb).CsrClientThread = CsrThread as PVOID;
    }

    CsrThread
}

#[no_mangle]
pub unsafe extern "system" fn CsrQueryApiPort() -> HANDLE {
    crate::dbg_print!(b"[CSRSRV] CsrQueryApiPort -> %p\n\0", CsrApiPort);
    CsrApiPort
}

#[no_mangle]
pub unsafe extern "system" fn CsrCaptureArguments(
    CsrThread: PCSR_THREAD,
    ApiMessage: PCSR_API_MESSAGE,
) -> BOOLEAN {
    let CsrProcess: PCSR_PROCESS = (*CsrThread).Process;
    let ClientCaptureBuffer: PCSR_CAPTURE_BUFFER;
    let mut ServerCaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();
    let EndOfClientBuffer: ULONG_PTR;
    let SizeOfBufferThroughOffsetsArray: usize;
    let BufferDistance: usize;
    let Length: ULONG;
    let PointerCount: ULONG;
    let mut OffsetPointer: PULONG_PTR;
    let _CurrentOffset: ULONG_PTR;

    ClientCaptureBuffer = (*ApiMessage)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .CsrCaptureData;

    if ((ClientCaptureBuffer as ULONG_PTR) < (*CsrProcess).ClientViewBase)
        || (((ClientCaptureBuffer as ULONG_PTR)
            + core::mem::offset_of!(_CSR_CAPTURE_BUFFER, PointerOffsetsArray) as ULONG_PTR)
            >= (*CsrProcess).ClientViewBounds)
    {
        (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 0u8;
    }

    Length = (*ClientCaptureBuffer).Size;
    EndOfClientBuffer = (ClientCaptureBuffer as ULONG_PTR) + (Length as ULONG_PTR);

    if (EndOfClientBuffer < (ClientCaptureBuffer as ULONG_PTR))
        || (EndOfClientBuffer >= (*CsrProcess).ClientViewBounds)
    {
        (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 0u8;
    }

    PointerCount = (*ClientCaptureBuffer).PointerCount;
    SizeOfBufferThroughOffsetsArray =
        core::mem::offset_of!(_CSR_CAPTURE_BUFFER, PointerOffsetsArray)
            + (PointerCount as usize * core::mem::size_of::<PVOID>());

    if (PointerCount > 0xFFFF) || (SizeOfBufferThroughOffsetsArray > Length as usize) {
        (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 0u8;
    }

    ServerCaptureBuffer = RtlAllocateHeap(
        CsrHeap,
        HEAP_ZERO_MEMORY,
        (Length as usize).try_into().unwrap(),
    ) as PCSR_CAPTURE_BUFFER;

    if ServerCaptureBuffer.is_null() {
        (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        return 0u8;
    }

    core::ptr::copy_nonoverlapping(
        ClientCaptureBuffer as *const u8,
        ServerCaptureBuffer as *mut u8,
        Length as usize,
    );

    (*ServerCaptureBuffer).Size = Length;
    (*ServerCaptureBuffer).PointerCount = PointerCount;

    BufferDistance = (ServerCaptureBuffer as usize).wrapping_sub(ClientCaptureBuffer as usize);

    OffsetPointer = (*ServerCaptureBuffer).PointerOffsetsArray.as_mut_ptr();
    let mut count = PointerCount;
    while count > 0 {
        let CurrentOffset = *OffsetPointer;

        if CurrentOffset != 0 {
            if ((CurrentOffset & (core::mem::size_of::<PVOID>() as ULONG_PTR - 1)) != 0)
                || (CurrentOffset
                    < core::mem::offset_of!(
                        _CSR_API_MESSAGE,
                        __bindgen_anon_1.__bindgen_anon_1.Data.ApiMessageData
                    ) as ULONG_PTR)
                || (CurrentOffset >= core::mem::size_of::<_CSR_API_MESSAGE>().try_into().unwrap())
            {
                (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                break;
            }

            let ptr_location = (CurrentOffset as usize) + (ApiMessage as usize);
            let pointed_value = *(ptr_location as *const ULONG_PTR);

            if (pointed_value
                >= ((ClientCaptureBuffer as ULONG_PTR)
                    + SizeOfBufferThroughOffsetsArray as ULONG_PTR))
                && (pointed_value
                    <= (EndOfClientBuffer
                        .wrapping_sub(core::mem::size_of::<PVOID>().try_into().unwrap())))
            {
                *(ptr_location as *mut ULONG_PTR) =
                    pointed_value.wrapping_add(BufferDistance.try_into().unwrap());
            } else {
                (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                break;
            }
        }

        OffsetPointer = OffsetPointer.add(1);
        count -= 1;
    }

    if (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status != STATUS_SUCCESS {
        RtlFreeHeap(CsrHeap, 0, ServerCaptureBuffer as PVOID);
        return 0u8;
    } else {
        (*ServerCaptureBuffer).PreviousCaptureBuffer = ClientCaptureBuffer;
        (*ApiMessage)
            .__bindgen_anon_1
            .__bindgen_anon_1
            .CsrCaptureData = ServerCaptureBuffer;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CsrReleaseCapturedArguments(ApiMessage: PCSR_API_MESSAGE) {
    let ServerCaptureBuffer: PCSR_CAPTURE_BUFFER;
    let ClientCaptureBuffer: PCSR_CAPTURE_BUFFER;
    let BufferDistance: usize;
    let PointerCount: ULONG;
    let mut OffsetPointer: PULONG_PTR;
    let _CurrentOffset: ULONG_PTR;

    ServerCaptureBuffer = (*ApiMessage)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .CsrCaptureData;

    if ServerCaptureBuffer.is_null() {
        return;
    }

    ClientCaptureBuffer = (*ServerCaptureBuffer).PreviousCaptureBuffer;
    (*ServerCaptureBuffer).PreviousCaptureBuffer = core::ptr::null_mut();
    (*ApiMessage)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .CsrCaptureData = ClientCaptureBuffer;

    BufferDistance = (ServerCaptureBuffer as usize).wrapping_sub(ClientCaptureBuffer as usize);

    PointerCount = (*ServerCaptureBuffer).PointerCount;
    OffsetPointer = (*ServerCaptureBuffer).PointerOffsetsArray.as_mut_ptr();

    let mut count = PointerCount;
    while count > 0 {
        let CurrentOffset = *OffsetPointer;

        if CurrentOffset != 0 {
            let ptr_location = (CurrentOffset as usize) + (ApiMessage as usize);
            *(ptr_location as *mut ULONG_PTR) = (*(ptr_location as *const ULONG_PTR))
                .wrapping_sub(BufferDistance.try_into().unwrap());
        }

        OffsetPointer = OffsetPointer.add(1);
        count -= 1;
    }

    core::ptr::copy_nonoverlapping(
        ServerCaptureBuffer as *const u8,
        ClientCaptureBuffer as *mut u8,
        (*ServerCaptureBuffer).Size as usize,
    );

    RtlFreeHeap(CsrHeap, 0, ServerCaptureBuffer as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn CsrValidateMessageBuffer(
    ApiMessage: PCSR_API_MESSAGE,
    Buffer: *mut PVOID,
    ElementCount: ULONG,
    ElementSize: ULONG,
) -> BOOLEAN {
    let CaptureBuffer: PCSR_CAPTURE_BUFFER = (*ApiMessage)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .CsrCaptureData;
    let BufferDistance: usize = (Buffer as usize).wrapping_sub(ApiMessage as usize);
    let PointerCount: ULONG;
    let mut OffsetPointer: PULONG_PTR;

    if Buffer.is_null()
        || ElementSize == 0
        || (ElementCount as u64) * (ElementSize as u64) > 0xFFFFFFFF
    {
        return 0u8;
    }

    if (*Buffer).is_null() && ElementCount == 0 {
        return 1u8;
    }

    if CaptureBuffer.is_null() {
        if (*NtCurrentTeb()).ClientId.UniqueProcess
            == (*ApiMessage).Header.__bindgen_anon_1.ClientId.UniqueProcess
        {
            return 1u8;
        }
    } else {
        if (((*CaptureBuffer).Size as usize)
            .wrapping_sub((*Buffer as usize).wrapping_sub(CaptureBuffer as usize)))
            >= ((ElementCount as usize) * (ElementSize as usize))
        {
            PointerCount = (*CaptureBuffer).PointerCount;
            OffsetPointer = (*CaptureBuffer).PointerOffsetsArray.as_mut_ptr();
            let mut count = PointerCount;
            while count > 0 {
                if *OffsetPointer == BufferDistance.try_into().unwrap() {
                    return 1u8;
                }
                OffsetPointer = OffsetPointer.add(1);
                count -= 1;
            }
        }
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn CsrValidateMessageString(
    ApiMessage: PCSR_API_MESSAGE,
    MessageString: *mut PWSTR,
) -> BOOLEAN {
    if MessageString.is_null() {
        return 0u8;
    }

    let str_ptr = *MessageString;
    if str_ptr.is_null() {
        return 0u8;
    }

    let mut len = 0usize;
    let mut ptr = str_ptr;
    while *ptr != 0 {
        len += 1;
        ptr = ptr.add(1);
    }

    CsrValidateMessageBuffer(
        ApiMessage,
        MessageString as *mut PVOID,
        (len + 1) as ULONG,
        core::mem::size_of::<WCHAR>() as ULONG,
    )
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Client/Server Runtime SubSystem
//  * FILE:            subsystems/win32/csrsrv/api.c
//  * PURPOSE:         CSR Server DLL API LPC Implementation
//  *                  "\Windows\ApiPort" port process management functions
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "srv.h"
//
// #include <ndk/kefuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// BOOLEAN (*CsrClientThreadSetup)(VOID) = NULL;
// UNICODE_STRING CsrApiPortName;
// volatile ULONG CsrpStaticThreadCount;
// volatile ULONG CsrpDynamicThreadTotal;
// extern ULONG CsrMaxApiRequestThreads;
//
// /* FUNCTIONS ******************************************************************/
//
// /*++
//  * @name CsrCallServerFromServer
//  * @implemented NT4
//  *
//  * The CsrCallServerFromServer routine calls a CSR API from within a server.
//  * It avoids using LPC messages since the request isn't coming from a client.
//  *
//  * @param ReceiveMsg
//  *        Pointer to the CSR API Message to send to the server.
//  *
//  * @param ReplyMsg
//  *        Pointer to the CSR API Message to receive from the server.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_ILLEGAL_FUNCTION
//  *         if the ApiNumber is invalid, or STATUS_ACCESS_VIOLATION if there
//  *         was a problem executing the API.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrCallServerFromServer(IN PCSR_API_MESSAGE ReceiveMsg,
//                         IN OUT PCSR_API_MESSAGE ReplyMsg)
// {
//     ULONG ServerId;
//     PCSR_SERVER_DLL ServerDll = NULL;
//     ULONG ApiId;
//     CSR_REPLY_CODE ReplyCode = CsrReplyImmediately;
//
//     /* Get the Server ID */
//     ServerId = CSR_API_NUMBER_TO_SERVER_ID(ReceiveMsg->ApiNumber);
//
//     /* Make sure that the ID is within limits, and the Server DLL loaded */
//     if ((ServerId >= CSR_SERVER_DLL_MAX) ||
//         (!(ServerDll = CsrLoadedServerDll[ServerId])))
//     {
//         /* We are beyond the Maximum Server ID */
// #ifdef CSR_DBG
//         DPRINT1("CSRSS: %lx is invalid ServerDllIndex (%08x)\n",
//                 ServerId, ServerDll);
//         if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//         ReplyMsg->Status = STATUS_ILLEGAL_FUNCTION;
//         return STATUS_ILLEGAL_FUNCTION;
//     }
//     else
//     {
//         /* Get the API ID, normalized with our Base ID */
//         ApiId = CSR_API_NUMBER_TO_API_ID(ReceiveMsg->ApiNumber) - ServerDll->ApiBase;
//
//         /* Make sure that the ID is within limits, and the entry exists */
//         if ((ApiId >= ServerDll->HighestApiSupported) ||
//             ((ServerDll->ValidTable) && !(ServerDll->ValidTable[ApiId])))
//         {
//             /* We are beyond the Maximum API ID, or it doesn't exist */
// #ifdef CSR_DBG
//             DPRINT1("API: %d\n", ApiId);
//             DPRINT1("CSRSS: %lx (%s) is invalid ApiTableIndex for %Z or is an "
//                     "invalid API to call from the server.\n",
//                     ApiId,
//                     ((ServerDll->NameTable) && (ServerDll->NameTable[ApiId])) ?
//                     ServerDll->NameTable[ApiId] : "*** UNKNOWN ***",
//                     &ServerDll->Name);
//             if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//             ReplyMsg->Status = STATUS_ILLEGAL_FUNCTION;
//             return STATUS_ILLEGAL_FUNCTION;
//         }
//     }
//
// #ifdef CSR_DBG
//     if (CsrDebug & 2)
//     {
//         DPRINT1("CSRSS: %s Api Request received from server process\n",
//                 ServerDll->NameTable[ApiId]);
//     }
// #endif
//
//     /* Validation complete, start SEH */
//     _SEH2_TRY
//     {
//         /* Call the API, get the reply code and return the result */
//         ReplyMsg->Status = ServerDll->DispatchTable[ApiId](ReceiveMsg, &ReplyCode);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* If we got an exception, return access violation */
//         ReplyMsg->Status = STATUS_ACCESS_VIOLATION;
//     }
//     _SEH2_END;
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrApiHandleConnectionRequest
//  *
//  * The CsrApiHandleConnectionRequest routine handles and accepts a new
//  * connection request to the CSR API LPC Port.
//  *
//  * @param ApiMessage
//  *        Pointer to the incoming CSR API Message which contains the
//  *        connection request.
//  *
//  * @return STATUS_SUCCESS in case of success, or status code which caused
//  *         the routine to error.
//  *
//  * @remarks This routine is responsible for attaching the Shared Section to
//  *          new clients connecting to CSR.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrApiHandleConnectionRequest(IN PCSR_API_MESSAGE ApiMessage)
// {
//     PCSR_THREAD CsrThread = NULL;
//     PCSR_PROCESS CsrProcess = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PCSR_API_CONNECTINFO ConnectInfo = &ApiMessage->ConnectionInfo;
//     BOOLEAN AllowConnection = FALSE;
//     REMOTE_PORT_VIEW RemotePortView;
//     HANDLE ServerPort;
//
//     /* Acquire the Process Lock */
//     CsrAcquireProcessLock();
//
//     /* Lookup the CSR Thread */
//     CsrThread = CsrLocateThreadByClientId(NULL, &ApiMessage->Header.ClientId);
//
//     /* Check if we have a thread */
//     if (CsrThread)
//     {
//         /* Get the Process and make sure we have it as well */
//         CsrProcess = CsrThread->Process;
//         if (CsrProcess)
//         {
//             /* Reference the Process */
//             CsrLockedReferenceProcess(CsrProcess);
//
//             /* Attach the Shared Section */
//             Status = CsrSrvAttachSharedSection(CsrProcess, ConnectInfo);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Allow the connection and return debugging flag */
//                 ConnectInfo->DebugFlags = CsrDebug;
//                 AllowConnection = TRUE;
//             }
//
//             /* Dereference the Process */
//             CsrLockedDereferenceProcess(CsrProcess);
//         }
//     }
//
//     /* Release the Process Lock */
//     CsrReleaseProcessLock();
//
//     /* Setup the Port View Structure */
//     RemotePortView.Length = sizeof(REMOTE_PORT_VIEW);
//     RemotePortView.ViewSize = 0;
//     RemotePortView.ViewBase = NULL;
//
//     /* Save the Process ID */
//     ConnectInfo->ServerProcessId = NtCurrentTeb()->ClientId.UniqueProcess;
//
//     /* Accept the Connection */
//     ASSERT(!AllowConnection || CsrProcess);
//     Status = NtAcceptConnectPort(&ServerPort,
//                                  AllowConnection ? UlongToPtr(CsrProcess->SequenceNumber) : 0,
//                                  &ApiMessage->Header,
//                                  AllowConnection,
//                                  NULL,
//                                  &RemotePortView);
//     if (!NT_SUCCESS(Status))
//     {
//          DPRINT1("CSRSS: NtAcceptConnectPort - failed.  Status == %X\n", Status);
//     }
//     else if (AllowConnection)
//     {
//         if (CsrDebug & 2)
//         {
//             DPRINT1("CSRSS: ClientId: %lx.%lx has ClientView: Base=%p, Size=%lx\n",
//                     ApiMessage->Header.ClientId.UniqueProcess,
//                     ApiMessage->Header.ClientId.UniqueThread,
//                     RemotePortView.ViewBase,
//                     RemotePortView.ViewSize);
//         }
//
//         /* Set some Port Data in the Process */
//         CsrProcess->ClientPort = ServerPort;
//         CsrProcess->ClientViewBase = (ULONG_PTR)RemotePortView.ViewBase;
//         CsrProcess->ClientViewBounds = (ULONG_PTR)((ULONG_PTR)RemotePortView.ViewBase +
//                                                    (ULONG_PTR)RemotePortView.ViewSize);
//
//         /* Complete the connection */
//         Status = NtCompleteConnectPort(ServerPort);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("CSRSS: NtCompleteConnectPort - failed.  Status == %X\n", Status);
//         }
//     }
//     else
//     {
//         DPRINT1("CSRSS: Rejecting Connection Request from ClientId: %lx.%lx\n",
//                 ApiMessage->Header.ClientId.UniqueProcess,
//                 ApiMessage->Header.ClientId.UniqueThread);
//     }
//
//     /* Return status to caller */
//     return Status;
// }
//
// /*++
//  * @name CsrpCheckRequestThreads
//  *
//  * The CsrpCheckRequestThreads routine checks if there are no more threads
//  * to handle CSR API Requests, and creates a new thread if possible, to
//  * avoid starvation.
//  *
//  * @param None.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL
//  *         if a new thread couldn't be created.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrpCheckRequestThreads(VOID)
// {
//     HANDLE hThread;
//     CLIENT_ID ClientId;
//     NTSTATUS Status;
//
//     /* Decrease the count, and see if we're out */
//     if (InterlockedDecrementUL(&CsrpStaticThreadCount) == 0)
//     {
//         /* Check if we've still got space for a Dynamic Thread */
//         if (CsrpDynamicThreadTotal < CsrMaxApiRequestThreads)
//         {
//             /* Create a new dynamic thread */
//             Status = RtlCreateUserThread(NtCurrentProcess(),
//                                          NULL,
//                                          TRUE,
//                                          0,
//                                          0,
//                                          0,
//                                          (PVOID)CsrApiRequestThread,
//                                          NULL,
//                                          &hThread,
//                                          &ClientId);
//             /* Check success */
//             if (NT_SUCCESS(Status))
//             {
//                 /* Increase the thread counts */
//                 InterlockedIncrementUL(&CsrpStaticThreadCount);
//                 InterlockedIncrementUL(&CsrpDynamicThreadTotal);
//
//                 /* Add a new server thread */
//                 if (CsrAddStaticServerThread(hThread,
//                                              &ClientId,
//                                              CsrThreadIsServerThread))
//                 {
//                     /* Activate it */
//                     NtResumeThread(hThread, NULL);
//                 }
//                 else
//                 {
//                     /* Failed to create a new static thread */
//                     InterlockedDecrementUL(&CsrpStaticThreadCount);
//                     InterlockedDecrementUL(&CsrpDynamicThreadTotal);
//
//                     /* Terminate it */
//                     DPRINT1("Failing\n");
//                     NtTerminateThread(hThread, 0);
//                     NtClose(hThread);
//
//                     /* Return */
//                     return STATUS_UNSUCCESSFUL;
//                 }
//             }
//         }
//     }
//
//     /* Success */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrApiRequestThread
//  *
//  * The CsrApiRequestThread routine handles incoming messages or connection
//  * requests on the CSR API LPC Port.
//  *
//  * @param Parameter
//  *        System-default user-defined parameter. Unused.
//  *
//  * @return The thread exit code, if the thread is terminated.
//  *
//  * @remarks Before listening on the port, the routine will first attempt
//  *          to connect to the user subsystem.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrApiRequestThread(IN PVOID Parameter)
// {
//     PTEB Teb = NtCurrentTeb();
// #ifndef _M_ARM64
//     LARGE_INTEGER TimeOut;
// #endif
//     PCSR_THREAD CurrentThread, CsrThread;
//     NTSTATUS Status;
//     CSR_REPLY_CODE ReplyCode;
//     PCSR_API_MESSAGE ReplyMsg;
//     CSR_API_MESSAGE ReceiveMsg;
//     PCSR_PROCESS CsrProcess;
//     PHARDERROR_MSG HardErrorMsg;
//     PVOID PortContext;
//     PCSR_SERVER_DLL ServerDll;
//     PCLIENT_DIED_MSG ClientDiedMsg;
//     PDBGKM_MSG DebugMessage;
//     ULONG ServerId, ApiId, MessageType, i;
//     HANDLE ReplyPort;
//
//     /* Setup LPC loop port and message */
//     ReplyMsg = NULL;
//     ReplyPort = CsrApiPort;
//
// #ifdef _M_ARM64
//     /* ARM64: Skip CsrConnectToUser loop — user32.dll is not loaded in csrss
//      * during early boot. Proceed immediately so the event is signaled and
//      * the main thread can continue to CsrSbApiPortInitialize + SmConnectToSm. */
// #else
//     /* Connect to user32 */
//     while (!CsrConnectToUser())
//     {
//         /* Set up the timeout for the connect (30 seconds) */
//         TimeOut.QuadPart = -30 * 1000 * 1000 * 10;
//
//         /* Keep trying until we get a response */
//         Teb->Win32ClientInfo[0] = 0;
//         NtDelayExecution(FALSE, &TimeOut);
//     }
// #endif
//
//     /* Get our thread */
//     CurrentThread = Teb->CsrClientThread;
//
//     /* If we got an event... */
//     if (Parameter)
//     {
//         /* Set it, to let stuff waiting on us load */
//         Status = NtSetEvent((HANDLE)Parameter, NULL);
//         ASSERT(NT_SUCCESS(Status));
//
//         /* Increase the Thread Counts */
//         InterlockedIncrementUL(&CsrpStaticThreadCount);
//         InterlockedIncrementUL(&CsrpDynamicThreadTotal);
//     }
//
//     /* Now start the loop */
//     while (TRUE)
//     {
//         /* Make sure the real CID is set */
//         Teb->RealClientId = Teb->ClientId;
//
// #ifdef CSR_DBG
//         /* Debug check */
//         if (Teb->CountOfOwnedCriticalSections)
//         {
//             DPRINT1("CSRSRV: FATAL ERROR. CsrThread is Idle while holding %lu critical sections\n",
//                     Teb->CountOfOwnedCriticalSections);
//             DPRINT1("CSRSRV: Last Receive Message %lx ReplyMessage %lx\n",
//                     &ReceiveMsg, ReplyMsg);
//             DbgBreakPoint();
//         }
// #endif
//
//         /* Wait for a message to come through */
//         Status = NtReplyWaitReceivePort(ReplyPort,
//                                         &PortContext,
//                                         &ReplyMsg->Header,
//                                         &ReceiveMsg.Header);
//
//         /* Check if we didn't get success */
//         if (Status != STATUS_SUCCESS)
//         {
//             /* Was it a failure or another success code? */
//             if (!NT_SUCCESS(Status))
//             {
// #ifdef CSR_DBG
//                 /* Check for specific status cases */
//                 if ((Status != STATUS_INVALID_CID) &&
//                     (Status != STATUS_UNSUCCESSFUL) &&
//                     ((Status != STATUS_INVALID_HANDLE) || (ReplyPort == CsrApiPort)))
//                 {
//                     /* Notify the debugger */
//                     DPRINT1("CSRSS: ReceivePort failed - Status == %X\n", Status);
//                     DPRINT1("CSRSS: ReplyPortHandle %lx CsrApiPort %lx\n", ReplyPort, CsrApiPort);
//                 }
// #endif
//
//                 /* We failed big time, so start out fresh */
//                 ReplyMsg = NULL;
//                 ReplyPort = CsrApiPort;
//                 continue;
//             }
//             else
//             {
//                 /* A strange "success" code, just try again */
//                 DPRINT1("NtReplyWaitReceivePort returned \"success\" status 0x%x\n", Status);
//                 continue;
//             }
//         }
//
//         // ASSERT(ReceiveMsg.Header.u1.s1.TotalLength >= sizeof(PORT_MESSAGE));
//         // ASSERT(ReceiveMsg.Header.u1.s1.TotalLength <  sizeof(ReceiveMsg));
//
//         /* Use whatever Client ID we got */
//         Teb->RealClientId = ReceiveMsg.Header.ClientId;
//
//         /* Get the Message Type */
//         MessageType = ReceiveMsg.Header.u2.s2.Type;
//
//         /* Handle connection requests */
//         if (MessageType == LPC_CONNECTION_REQUEST)
//         {
//             /* Handle the Connection Request */
//             CsrApiHandleConnectionRequest(&ReceiveMsg);
//
//             ReplyMsg = NULL;
//             ReplyPort = CsrApiPort;
//             continue;
//         }
//
//         /* It's some other kind of request. Get the lock for the lookup */
//         CsrAcquireProcessLock();
//
//         /* Now do the lookup to get the CSR_THREAD */
//         CsrThread = CsrLocateThreadByClientId(&CsrProcess,
//                                               &ReceiveMsg.Header.ClientId);
//
//         /* Did we find a thread? */
//         if (!CsrThread)
//         {
//             /* This wasn't a CSR Thread, release lock */
//             CsrReleaseProcessLock();
//
//             /* If this was an exception, handle it */
//             if (MessageType == LPC_EXCEPTION)
//             {
//                 ReplyMsg = &ReceiveMsg;
//                 ReplyPort = CsrApiPort;
//                 ReplyMsg->Status = DBG_CONTINUE;
//             }
//             else if (MessageType == LPC_PORT_CLOSED ||
//                      MessageType == LPC_CLIENT_DIED)
//             {
//                 /* The Client or Port are gone, loop again */
//                 ReplyMsg = NULL;
//                 ReplyPort = CsrApiPort;
//             }
//             else if (MessageType == LPC_ERROR_EVENT)
//             {
//                 /* If it's a hard error, handle this too */
//                 HardErrorMsg = (PHARDERROR_MSG)&ReceiveMsg;
//
//                 /* Default it to unhandled */
//                 HardErrorMsg->Response = ResponseNotHandled;
//
//                 /* Check if there are free api threads */
//                 CsrpCheckRequestThreads();
//                 if (CsrpStaticThreadCount)
//                 {
//                     /* Loop every Server DLL */
//                     for (i = 0; i < CSR_SERVER_DLL_MAX; i++)
//                     {
//                         /* Get the Server DLL */
//                         ServerDll = CsrLoadedServerDll[i];
//
//                         /* Check if it's valid and if it has a Hard Error Callback */
//                         if ((ServerDll) && (ServerDll->HardErrorCallback))
//                         {
//                             /* Call it */
//                             ServerDll->HardErrorCallback(NULL /* == CsrThread */, HardErrorMsg);
//
//                             /* If it's handled, get out of here */
//                             if (HardErrorMsg->Response != ResponseNotHandled) break;
//                         }
//                     }
//                 }
//
//                 /* Increase the thread count */
//                 InterlockedIncrementUL(&CsrpStaticThreadCount);
//
//                 /* If the response was 0xFFFFFFFF, we'll ignore it */
//                 if (HardErrorMsg->Response == 0xFFFFFFFF)
//                 {
//                     ReplyMsg = NULL;
//                     ReplyPort = CsrApiPort;
//                 }
//                 else
//                 {
//                     ReplyMsg = &ReceiveMsg;
//                     ReplyPort = CsrApiPort;
//                 }
//             }
//             else if (MessageType == LPC_REQUEST)
//             {
//                 /* This is an API Message coming from a non-CSR Thread */
//                 ReplyMsg = &ReceiveMsg;
//                 ReplyPort = CsrApiPort;
//                 ReplyMsg->Status = STATUS_ILLEGAL_FUNCTION;
//             }
//             else if (MessageType == LPC_DATAGRAM)
//             {
//                 /* This is an API call, get the Server ID */
//                 ServerId = CSR_API_NUMBER_TO_SERVER_ID(ReceiveMsg.ApiNumber);
//
//                 /* Make sure that the ID is within limits, and the Server DLL loaded */
//                 ServerDll = NULL;
//                 if ((ServerId >= CSR_SERVER_DLL_MAX) ||
//                     (!(ServerDll = CsrLoadedServerDll[ServerId])))
//                 {
//                     /* We are beyond the Maximum Server ID */
// #ifdef CSR_DBG
//                     DPRINT1("CSRSS: %lx is invalid ServerDllIndex (%08x)\n",
//                             ServerId, ServerDll);
//                     if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//                     ReplyMsg = NULL;
//                     ReplyPort = CsrApiPort;
//                     continue;
//                 }
//
//                 /* Get the API ID, normalized with our Base ID */
//                 ApiId = CSR_API_NUMBER_TO_API_ID(ReceiveMsg.ApiNumber) - ServerDll->ApiBase;
//
//                 /* Make sure that the ID is within limits, and the entry exists */
//                 if (ApiId >= ServerDll->HighestApiSupported)
//                 {
//                     /* We are beyond the Maximum API ID, or it doesn't exist */
// #ifdef CSR_DBG
//                     DPRINT1("CSRSS: %lx is invalid ApiTableIndex for %Z\n",
//                             CSR_API_NUMBER_TO_API_ID(ReceiveMsg.ApiNumber),
//                             &ServerDll->Name);
// #endif
//                     ReplyPort = CsrApiPort;
//                     ReplyMsg = NULL;
//                     continue;
//                 }
//
// #ifdef CSR_DBG
//                 if (CsrDebug & 2)
//                 {
//                     DPRINT1("[%02x] CSRSS: [%02x,%02x] - %s Api called from %08x\n",
//                             Teb->ClientId.UniqueThread,
//                             ReceiveMsg.Header.ClientId.UniqueProcess,
//                             ReceiveMsg.Header.ClientId.UniqueThread,
//                             ServerDll->NameTable[ApiId],
//                             NULL);
//                 }
// #endif
//
//                 /* Assume success */
//                 ReceiveMsg.Status = STATUS_SUCCESS;
//
//                 /* Validation complete, start SEH */
//                 _SEH2_TRY
//                 {
//                     /* Make sure we have enough threads */
//                     CsrpCheckRequestThreads();
//
//                     /* Call the API and get the reply code */
//                     ReplyMsg = NULL;
//                     ReplyPort = CsrApiPort;
//                     ServerDll->DispatchTable[ApiId](&ReceiveMsg, &ReplyCode);
//
//                     /* Increase the static thread count */
//                     InterlockedIncrementUL(&CsrpStaticThreadCount);
//                 }
//                 _SEH2_EXCEPT(CsrUnhandledExceptionFilter(_SEH2_GetExceptionInformation()))
//                 {
//                     ReplyMsg = NULL;
//                     ReplyPort = CsrApiPort;
//                 }
//                 _SEH2_END;
//             }
//             else
//             {
//                 /* Some other ignored message type */
//                 ReplyMsg = NULL;
//                 ReplyPort = CsrApiPort;
//             }
//
//             /* Keep going */
//             continue;
//         }
//
//         /* We have a valid thread, was this an LPC Request? */
//         if (MessageType != LPC_REQUEST)
//         {
//             /* It's not an API, check if the client died */
//             if (MessageType == LPC_CLIENT_DIED)
//             {
//                 /* Get the information and check if it matches our thread */
//                 ClientDiedMsg = (PCLIENT_DIED_MSG)&ReceiveMsg;
//                 if (ClientDiedMsg->CreateTime.QuadPart == CsrThread->CreateTime.QuadPart)
//                 {
//                     /* Now we reply to the dying client */
//                     ReplyPort = CsrThread->Process->ClientPort;
//
//                     /* Reference the thread */
//                     CsrLockedReferenceThread(CsrThread);
//
//                     /* Destroy the thread in the API Message */
//                     CsrDestroyThread(&ReceiveMsg.Header.ClientId);
//
//                     /* Check if the thread was actually ourselves */
//                     if (CsrProcess->ThreadCount == 1)
//                     {
//                         /* Kill the process manually here */
//                         CsrDestroyProcess(&CsrThread->ClientId, 0);
//                     }
//
//                     /* Remove our extra reference */
//                     CsrLockedDereferenceThread(CsrThread);
//                 }
//
//                 /* Release the lock and keep looping */
//                 CsrReleaseProcessLock();
//
//                 ReplyMsg = NULL;
//                 ReplyPort = CsrApiPort;
//                 continue;
//             }
//
//             /* Reference the thread and release the lock */
//             CsrLockedReferenceThread(CsrThread);
//             CsrReleaseProcessLock();
//
//             /* Check if this was an exception */
//             if (MessageType == LPC_EXCEPTION)
//             {
//                 /* Kill the process */
//                 NtTerminateProcess(CsrProcess->ProcessHandle, STATUS_ABANDONED);
//
//                 /* Destroy it from CSR */
//                 CsrDestroyProcess(&ReceiveMsg.Header.ClientId, STATUS_ABANDONED);
//
//                 /* Return a Debug Message */
//                 DebugMessage = (PDBGKM_MSG)&ReceiveMsg;
//                 DebugMessage->ReturnedStatus = DBG_CONTINUE;
//                 ReplyMsg = &ReceiveMsg;
//                 ReplyPort = CsrApiPort;
//
//                 /* Remove our extra reference */
//                 CsrDereferenceThread(CsrThread);
//             }
//             else if (MessageType == LPC_ERROR_EVENT)
//             {
//                 /* If it's a hard error, handle this too */
//                 HardErrorMsg = (PHARDERROR_MSG)&ReceiveMsg;
//
//                 /* Default it to unhandled */
//                 HardErrorMsg->Response = ResponseNotHandled;
//
//                 /* Check if there are free api threads */
//                 CsrpCheckRequestThreads();
//                 if (CsrpStaticThreadCount)
//                 {
//                     /* Loop every Server DLL */
//                     for (i = 0; i < CSR_SERVER_DLL_MAX; i++)
//                     {
//                         /* Get the Server DLL */
//                         ServerDll = CsrLoadedServerDll[i];
//
//                         /* Check if it's valid and if it has a Hard Error Callback */
//                         if ((ServerDll) && (ServerDll->HardErrorCallback))
//                         {
//                             /* Call it */
//                             ServerDll->HardErrorCallback(CsrThread, HardErrorMsg);
//
//                             /* If it's handled, get out of here */
//                             if (HardErrorMsg->Response != ResponseNotHandled) break;
//                         }
//                     }
//                 }
//
//                 /* Increase the thread count */
//                 InterlockedIncrementUL(&CsrpStaticThreadCount);
//
//                 /* If the response was 0xFFFFFFFF, we'll ignore it */
//                 if (HardErrorMsg->Response == 0xFFFFFFFF)
//                 {
//                     ReplyMsg = NULL;
//                     ReplyPort = CsrApiPort;
//                 }
//                 else
//                 {
//                     CsrDereferenceThread(CsrThread);
//                     ReplyMsg = &ReceiveMsg;
//                     ReplyPort = CsrApiPort;
//                 }
//             }
//             else
//             {
//                 /* Something else */
//                 CsrDereferenceThread(CsrThread);
//                 ReplyMsg = NULL;
//             }
//
//             /* Keep looping */
//             continue;
//         }
//
//         /* We got an API Request */
//         CsrLockedReferenceThread(CsrThread);
//         CsrReleaseProcessLock();
//
//         /* This is an API call, get the Server ID */
//         ServerId = CSR_API_NUMBER_TO_SERVER_ID(ReceiveMsg.ApiNumber);
//
//         /* Make sure that the ID is within limits, and the Server DLL loaded */
//         ServerDll = NULL;
//         if ((ServerId >= CSR_SERVER_DLL_MAX) ||
//             (!(ServerDll = CsrLoadedServerDll[ServerId])))
//         {
//             /* We are beyond the Maximum Server ID */
// #ifdef CSR_DBG
//             DPRINT1("CSRSS: %lx is invalid ServerDllIndex (%08x)\n",
//                     ServerId, ServerDll);
//             if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//             ReplyPort = CsrApiPort;
//             ReplyMsg = &ReceiveMsg;
//             ReplyMsg->Status = STATUS_ILLEGAL_FUNCTION;
//             CsrDereferenceThread(CsrThread);
//             continue;
//         }
//
//         /* Get the API ID, normalized with our Base ID */
//         ApiId = CSR_API_NUMBER_TO_API_ID(ReceiveMsg.ApiNumber) - ServerDll->ApiBase;
//
//         /* Make sure that the ID is within limits, and the entry exists */
//         if (ApiId >= ServerDll->HighestApiSupported)
//         {
// #ifdef CSR_DBG
//             /* We are beyond the Maximum API ID, or it doesn't exist */
//             DPRINT1("CSRSS: %lx is invalid ApiTableIndex for %Z\n",
//                     CSR_API_NUMBER_TO_API_ID(ReceiveMsg.ApiNumber),
//                     &ServerDll->Name);
// #endif
//             ReplyPort = CsrApiPort;
//             ReplyMsg = &ReceiveMsg;
//             ReplyMsg->Status = STATUS_ILLEGAL_FUNCTION;
//             CsrDereferenceThread(CsrThread);
//             continue;
//         }
//
// #ifdef CSR_DBG
//         if (CsrDebug & 2)
//         {
//             DPRINT1("[%02x] CSRSS: [%02x,%02x] - %s Api called from %08x, Process %08x - %08x\n",
//                     Teb->ClientId.UniqueThread,
//                     ReceiveMsg.Header.ClientId.UniqueProcess,
//                     ReceiveMsg.Header.ClientId.UniqueThread,
//                     ServerDll->NameTable[ApiId],
//                     CsrThread,
//                     CsrThread->Process,
//                     CsrProcess);
//         }
// #endif
//
//         /* Assume success */
//         ReplyMsg = &ReceiveMsg;
//         ReceiveMsg.Status = STATUS_SUCCESS;
//
//         /* Now we reply to a particular client */
//         ReplyPort = CsrThread->Process->ClientPort;
//
//         /* Check if there's a capture buffer */
//         if (ReceiveMsg.CsrCaptureData)
//         {
//             /* Capture the arguments */
//             if (!CsrCaptureArguments(CsrThread, &ReceiveMsg))
//             {
//                 /* Ignore this message if we failed to get the arguments */
//                 CsrDereferenceThread(CsrThread);
//                 continue;
//             }
//         }
//
//         /* Validation complete, start SEH */
//         _SEH2_TRY
//         {
//             /* Make sure we have enough threads */
//             CsrpCheckRequestThreads();
//
//             Teb->CsrClientThread = CsrThread;
//
//             /* Call the API, get the reply code and return the result */
//             ReplyCode = CsrReplyImmediately;
//             ReplyMsg->Status = ServerDll->DispatchTable[ApiId](&ReceiveMsg, &ReplyCode);
//
//             /* Increase the static thread count */
//             InterlockedIncrementUL(&CsrpStaticThreadCount);
//
//             Teb->CsrClientThread = CurrentThread;
//
//             if (ReplyCode == CsrReplyAlreadySent)
//             {
//                 if (ReceiveMsg.CsrCaptureData)
//                 {
//                     CsrReleaseCapturedArguments(&ReceiveMsg);
//                 }
//                 ReplyMsg = NULL;
//                 ReplyPort = CsrApiPort;
//                 CsrDereferenceThread(CsrThread);
//             }
//             else if (ReplyCode == CsrReplyDeadClient)
//             {
//                 /* Reply to the death message */
//                 NTSTATUS Status2;
//                 Status2 = NtReplyPort(ReplyPort, &ReplyMsg->Header);
//                 if (!NT_SUCCESS(Status2))
//                     DPRINT1("CSRSS: Error while replying to the death message, Status 0x%lx\n", Status2);
//
//                 /* Reply back to the API port now */
//                 ReplyMsg = NULL;
//                 ReplyPort = CsrApiPort;
//                 CsrDereferenceThread(CsrThread);
//             }
//             else if (ReplyCode == CsrReplyPending)
//             {
//                 ReplyMsg = NULL;
//                 ReplyPort = CsrApiPort;
//             }
//             else
//             {
//                 if (ReceiveMsg.CsrCaptureData)
//                 {
//                     CsrReleaseCapturedArguments(&ReceiveMsg);
//                 }
//                 CsrDereferenceThread(CsrThread);
//             }
//         }
//         _SEH2_EXCEPT(CsrUnhandledExceptionFilter(_SEH2_GetExceptionInformation()))
//         {
//             ReplyMsg = NULL;
//             ReplyPort = CsrApiPort;
//         }
//         _SEH2_END;
//     }
//
//     /* We're out of the loop for some reason, terminate! */
//     NtTerminateThread(NtCurrentThread(), Status);
//     return Status;
// }
//
// /*++
//  * @name CsrApiPortInitialize
//  *
//  * The CsrApiPortInitialize routine initializes the LPC Port used for
//  * communications with the Client/Server Runtime (CSR) and initializes the
//  * static thread that will handle connection requests and APIs.
//  *
//  * @param None
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrApiPortInitialize(VOID)
// {
//     ULONG Size;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE hRequestEvent, hThread;
//     CLIENT_ID ClientId;
//     PLIST_ENTRY ListHead, NextEntry;
//     PCSR_THREAD ServerThread;
//
//     /* Calculate how much space we'll need for the Port Name */
//     Size = CsrDirectoryName.Length + sizeof(CSR_PORT_NAME) + sizeof(WCHAR);
//
//     /* Create the buffer for it */
//     CsrApiPortName.Buffer = RtlAllocateHeap(CsrHeap, 0, Size);
//     if (!CsrApiPortName.Buffer) return STATUS_NO_MEMORY;
//
//     /* Setup the rest of the empty string */
//     CsrApiPortName.Length = 0;
//     CsrApiPortName.MaximumLength = (USHORT)Size;
//     RtlAppendUnicodeStringToString(&CsrApiPortName, &CsrDirectoryName);
//     RtlAppendUnicodeToString(&CsrApiPortName, UNICODE_PATH_SEP);
//     RtlAppendUnicodeToString(&CsrApiPortName, CSR_PORT_NAME);
//     if (CsrDebug & 1)
//     {
//         DPRINT1("CSRSS: Creating %wZ port and associated threads\n", &CsrApiPortName);
//         DPRINT1("CSRSS: sizeof( CONNECTINFO ) == %ld  sizeof( API_MSG ) == %ld\n",
//                 sizeof(CSR_API_CONNECTINFO), sizeof(CSR_API_MESSAGE));
//     }
//
//     /* FIXME: Create a Security Descriptor */
//
//     /* Initialize the Attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &CsrApiPortName,
//                                0,
//                                NULL,
//                                NULL /* FIXME: Use the Security Descriptor */);
//
//     /* Create the Port Object */
//     Status = NtCreatePort(&CsrApiPort,
//                           &ObjectAttributes,
//                           sizeof(CSR_API_CONNECTINFO),
//                           sizeof(CSR_API_MESSAGE),
//                           16 * PAGE_SIZE);
//     if (NT_SUCCESS(Status))
//     {
//         /* Create the event the Port Thread will use */
//         Status = NtCreateEvent(&hRequestEvent,
//                                EVENT_ALL_ACCESS,
//                                NULL,
//                                SynchronizationEvent,
//                                FALSE);
//         if (NT_SUCCESS(Status))
//         {
//             /* Create the Request Thread */
//             Status = RtlCreateUserThread(NtCurrentProcess(),
//                                          NULL,
//                                          TRUE,
//                                          0,
//                                          0,
//                                          0,
//                                          (PVOID)CsrApiRequestThread,
//                                          (PVOID)hRequestEvent,
//                                          &hThread,
//                                          &ClientId);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Add this as a static thread to CSRSRV */
//                 CsrAddStaticServerThread(hThread, &ClientId, CsrThreadIsServerThread);
//
//                 /* Get the Thread List Pointers */
//                 ListHead = &CsrRootProcess->ThreadList;
//                 NextEntry = ListHead->Flink;
//
//                 /* Start looping the list */
//                 while (NextEntry != ListHead)
//                 {
//                     /* Get the Thread */
//                     ServerThread = CONTAINING_RECORD(NextEntry, CSR_THREAD, Link);
//
//                     /* Start it up */
//                     Status = NtResumeThread(ServerThread->ThreadHandle, NULL);
//
//                     /* Is this a Server Thread? */
//                     if (ServerThread->Flags & CsrThreadIsServerThread)
//                     {
//                         /* If so, then wait for it to initialize */
//                         Status = NtWaitForSingleObject(hRequestEvent, FALSE, NULL);
//                         ASSERT(NT_SUCCESS(Status));
//                     }
//
//                     /* Next thread */
//                     NextEntry = NextEntry->Flink;
//                 }
//
//                 /* We don't need this anymore */
//                 NtClose(hRequestEvent);
//             }
//         }
//     }
//
//     /* Return */
//     return Status;
// }
//
// /*++
//  * @name CsrConnectToUser
//  * @implemented NT4
//  *
//  * The CsrConnectToUser connects to the User subsystem.
//  *
//  * @param None
//  *
//  * @return A pointer to the CSR Thread
//  *
//  * @remarks None.
//  *
//  *--*/
// PCSR_THREAD
// NTAPI
// CsrConnectToUser(VOID)
// {
//     NTSTATUS Status;
//     ANSI_STRING DllName;
//     UNICODE_STRING TempName;
//     HANDLE hUser32;
//     STRING StartupName;
//     PTEB Teb = NtCurrentTeb();
//     PCSR_THREAD CsrThread;
//     BOOLEAN Connected;
//
//     /* Check if we didn't already find it */
//     if (!CsrClientThreadSetup)
//     {
//         /* Get the DLL Handle for user32.dll */
//         RtlInitAnsiString(&DllName, "user32");
//         RtlAnsiStringToUnicodeString(&TempName, &DllName, TRUE);
//         Status = LdrGetDllHandle(NULL,
//                                  NULL,
//                                  &TempName,
//                                  &hUser32);
//         RtlFreeUnicodeString(&TempName);
//
//         /* If we got the handle, get the Client Thread Startup Entrypoint */
//         if (NT_SUCCESS(Status))
//         {
//             RtlInitAnsiString(&StartupName,"ClientThreadSetup");
//             Status = LdrGetProcedureAddress(hUser32,
//                                             &StartupName,
//                                             0,
//                                             (PVOID)&CsrClientThreadSetup);
//         }
//     }
//
//     /* Connect to user32 */
//     if (CsrClientThreadSetup)
//     {
//         _SEH2_TRY
//         {
//             Connected = CsrClientThreadSetup();
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Connected = FALSE;
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         Connected = FALSE;
//     }
//
//     if (!Connected)
//     {
//         DPRINT1("CSRSS: CsrConnectToUser failed\n");
//         return NULL;
//     }
//
//     /* Save pointer to this thread in TEB */
//     CsrAcquireProcessLock();
//     CsrThread = CsrLocateThreadInProcess(NULL, &Teb->ClientId);
//     CsrReleaseProcessLock();
//     if (CsrThread) Teb->CsrClientThread = CsrThread;
//
//     /* Return it */
//     return CsrThread;
// }
//
// /*++
//  * @name CsrQueryApiPort
//  * @implemented NT4
//  *
//  * The CsrQueryApiPort routine returns a handle to the CSR API LPC port.
//  *
//  * @param None.
//  *
//  * @return A handle to the port.
//  *
//  * @remarks None.
//  *
//  *--*/
// HANDLE
// NTAPI
// CsrQueryApiPort(VOID)
// {
//     return CsrApiPort;
// }
//
// /*++
//  * @name CsrCaptureArguments
//  * @implemented NT5.1
//  *
//  * The CsrCaptureArguments routine validates a CSR Capture Buffer and
//  * re-captures it into a server CSR Capture Buffer.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread performing the validation.
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message containing the Capture Buffer
//  *        that needs to be validated.
//  *
//  * @return TRUE if validation succeeded, FALSE otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// CsrCaptureArguments(IN PCSR_THREAD CsrThread,
//                     IN PCSR_API_MESSAGE ApiMessage)
// {
//     PCSR_PROCESS CsrProcess = CsrThread->Process;
//     PCSR_CAPTURE_BUFFER ClientCaptureBuffer, ServerCaptureBuffer = NULL;
//     ULONG_PTR EndOfClientBuffer;
//     SIZE_T SizeOfBufferThroughOffsetsArray;
//     SIZE_T BufferDistance;
//     ULONG Length;
//     ULONG PointerCount;
//     PULONG_PTR OffsetPointer;
//     ULONG_PTR CurrentOffset;
//
//     /* Get the buffer we got from whoever called NTDLL */
//     ClientCaptureBuffer = ApiMessage->CsrCaptureData;
//
//     /* Use SEH to validate and capture the client buffer */
//     _SEH2_TRY
//     {
//         /* Check whether at least the buffer's header is inside our mapped section */
//         if (  ((ULONG_PTR)ClientCaptureBuffer < CsrProcess->ClientViewBase) ||
//              (((ULONG_PTR)ClientCaptureBuffer + FIELD_OFFSET(CSR_CAPTURE_BUFFER, PointerOffsetsArray))
//                     >= CsrProcess->ClientViewBounds) )
//         {
// #ifdef CSR_DBG
//             DPRINT1("*** CSRSS: CaptureBuffer outside of ClientView 1\n");
//             if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//             /* Return failure */
//             ApiMessage->Status = STATUS_INVALID_PARAMETER;
//             _SEH2_YIELD(return FALSE);
//         }
//
//         /* Capture the buffer length */
//         Length = ((volatile CSR_CAPTURE_BUFFER*)ClientCaptureBuffer)->Size;
//
//         /*
//          * Now check if the remaining of the buffer is inside our mapped section.
//          * Take also care for any possible wrap-around of the buffer end-address.
//          */
//         EndOfClientBuffer = (ULONG_PTR)ClientCaptureBuffer + Length;
//         if ( (EndOfClientBuffer < (ULONG_PTR)ClientCaptureBuffer) ||
//              (EndOfClientBuffer >= CsrProcess->ClientViewBounds) )
//         {
// #ifdef CSR_DBG
//             DPRINT1("*** CSRSS: CaptureBuffer outside of ClientView 2\n");
//             if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//             /* Return failure */
//             ApiMessage->Status = STATUS_INVALID_PARAMETER;
//             _SEH2_YIELD(return FALSE);
//         }
//
//         /* Capture the pointer count */
//         PointerCount = ((volatile CSR_CAPTURE_BUFFER*)ClientCaptureBuffer)->PointerCount;
//
//         /*
//          * Check whether the total buffer size and the pointer count are consistent
//          * -- the array of offsets must be contained inside the buffer.
//          */
//         SizeOfBufferThroughOffsetsArray =
//             FIELD_OFFSET(CSR_CAPTURE_BUFFER, PointerOffsetsArray) +
//                 (PointerCount * sizeof(PVOID));
//         if ( (PointerCount > MAXUSHORT) ||
//              (SizeOfBufferThroughOffsetsArray > Length) )
//         {
// #ifdef CSR_DBG
//             DPRINT1("*** CSRSS: CaptureBuffer %p has bad length\n", ClientCaptureBuffer);
//             if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//             /* Return failure */
//             ApiMessage->Status = STATUS_INVALID_PARAMETER;
//             _SEH2_YIELD(return FALSE);
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
// #ifdef CSR_DBG
//         DPRINT1("*** CSRSS: Took exception during capture %x\n", _SEH2_GetExceptionCode());
//         if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//         /* Return failure */
//         ApiMessage->Status = STATUS_INVALID_PARAMETER;
//         _SEH2_YIELD(return FALSE);
//     }
//     _SEH2_END;
//
//     /* We validated the client buffer, now allocate the server buffer */
//     ServerCaptureBuffer = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, Length);
//     if (!ServerCaptureBuffer)
//     {
//         /* We're out of memory */
//         ApiMessage->Status = STATUS_NO_MEMORY;
//         return FALSE;
//     }
//
//     /*
//      * Copy the client's buffer and ensure we use the correct buffer length
//      * and pointer count we captured and used for validation earlier on.
//      */
//     _SEH2_TRY
//     {
//         RtlMoveMemory(ServerCaptureBuffer, ClientCaptureBuffer, Length);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
// #ifdef CSR_DBG
//         DPRINT1("*** CSRSS: Took exception during capture %x\n", _SEH2_GetExceptionCode());
//         if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//         /* Failure, free the buffer and return */
//         RtlFreeHeap(CsrHeap, 0, ServerCaptureBuffer);
//         ApiMessage->Status = STATUS_INVALID_PARAMETER;
//         _SEH2_YIELD(return FALSE);
//     }
//     _SEH2_END;
//
//     ServerCaptureBuffer->Size = Length;
//     ServerCaptureBuffer->PointerCount = PointerCount;
//
//     /* Calculate the difference between our buffer and the client's */
//     BufferDistance = (ULONG_PTR)ServerCaptureBuffer - (ULONG_PTR)ClientCaptureBuffer;
//
//     /*
//      * All the pointer offsets correspond to pointers that point
//      * to the server data buffer instead of the client one.
//      */
//     // PointerCount  = ServerCaptureBuffer->PointerCount;
//     OffsetPointer = ServerCaptureBuffer->PointerOffsetsArray;
//     while (PointerCount--)
//     {
//         CurrentOffset = *OffsetPointer;
//
//         if (CurrentOffset != 0)
//         {
//             /*
//              * Check whether the offset is pointer-aligned and whether
//              * it points inside CSR_API_MESSAGE::Data.ApiMessageData.
//              */
//             if ( ((CurrentOffset & (sizeof(PVOID)-1)) != 0) ||
//                  (CurrentOffset < FIELD_OFFSET(CSR_API_MESSAGE, Data.ApiMessageData)) ||
//                  (CurrentOffset >= sizeof(CSR_API_MESSAGE)) )
//             {
// #ifdef CSR_DBG
//                 DPRINT1("*** CSRSS: CaptureBuffer MessagePointer outside of message\n");
//                 if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//                 /* Invalid pointer, fail */
//                 ApiMessage->Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Get the pointer corresponding to the offset */
//             CurrentOffset += (ULONG_PTR)ApiMessage;
//
//             /* Validate the bounds of the current pointed pointer */
//             if ( (*(PULONG_PTR)CurrentOffset >= ((ULONG_PTR)ClientCaptureBuffer +
//                                                  SizeOfBufferThroughOffsetsArray)) &&
//                  (*(PULONG_PTR)CurrentOffset <= (EndOfClientBuffer - sizeof(PVOID))) )
//             {
//                 /* Modify the pointed pointer to take into account its new position */
//                 *(PULONG_PTR)CurrentOffset += BufferDistance;
//             }
//             else
//             {
// #ifdef CSR_DBG
//                 DPRINT1("*** CSRSS: CaptureBuffer MessagePointer outside of ClientView\n");
//                 if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//                 /* Invalid pointer, fail */
//                 ApiMessage->Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//         }
//
//         ++OffsetPointer;
//     }
//
//     /* Check if we got success */
//     if (ApiMessage->Status != STATUS_SUCCESS)
//     {
//         /* Failure, free the buffer and return */
//         RtlFreeHeap(CsrHeap, 0, ServerCaptureBuffer);
//         return FALSE;
//     }
//     else
//     {
//         /* Success, save the previous buffer and use the server capture buffer */
//         ServerCaptureBuffer->PreviousCaptureBuffer = ClientCaptureBuffer;
//         ApiMessage->CsrCaptureData = ServerCaptureBuffer;
//     }
//
//     /* Success */
//     return TRUE;
// }
//
// /*++
//  * @name CsrReleaseCapturedArguments
//  * @implemented NT5.1
//  *
//  * The CsrReleaseCapturedArguments routine releases a Capture Buffer
//  * that was previously captured with CsrCaptureArguments.
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message containing the Capture Buffer
//  *        that needs to be released.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// CsrReleaseCapturedArguments(IN PCSR_API_MESSAGE ApiMessage)
// {
//     PCSR_CAPTURE_BUFFER ServerCaptureBuffer, ClientCaptureBuffer;
//     SIZE_T BufferDistance;
//     ULONG PointerCount;
//     PULONG_PTR OffsetPointer;
//     ULONG_PTR CurrentOffset;
//
//     /* Get the server capture buffer */
//     ServerCaptureBuffer = ApiMessage->CsrCaptureData;
//
//     /* Do not continue if there is no captured buffer */
//     if (!ServerCaptureBuffer) return;
//
//     /* If there is one, get the corresponding client capture buffer */
//     ClientCaptureBuffer = ServerCaptureBuffer->PreviousCaptureBuffer;
//
//     /* Free the previous one and use again the client capture buffer */
//     ServerCaptureBuffer->PreviousCaptureBuffer = NULL;
//     ApiMessage->CsrCaptureData = ClientCaptureBuffer;
//
//     /* Calculate the difference between our buffer and the client's */
//     BufferDistance = (ULONG_PTR)ServerCaptureBuffer - (ULONG_PTR)ClientCaptureBuffer;
//
//     /*
//      * All the pointer offsets correspond to pointers that point
//      * to the client data buffer instead of the server one (reverse
//      * the logic of CsrCaptureArguments()).
//      */
//     PointerCount  = ServerCaptureBuffer->PointerCount;
//     OffsetPointer = ServerCaptureBuffer->PointerOffsetsArray;
//     while (PointerCount--)
//     {
//         CurrentOffset = *OffsetPointer;
//
//         if (CurrentOffset != 0)
//         {
//             /* Get the pointer corresponding to the offset */
//             CurrentOffset += (ULONG_PTR)ApiMessage;
//
//             /* Modify the pointed pointer to take into account its new position */
//             *(PULONG_PTR)CurrentOffset -= BufferDistance;
//         }
//
//         ++OffsetPointer;
//     }
//
//     /* Copy the data back into the client buffer */
//     _SEH2_TRY
//     {
//         RtlMoveMemory(ClientCaptureBuffer, ServerCaptureBuffer, ServerCaptureBuffer->Size);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
// #ifdef CSR_DBG
//         DPRINT1("*** CSRSS: Took exception during release %x\n", _SEH2_GetExceptionCode());
//         if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//         /* Return failure */
//         ApiMessage->Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Free our allocated buffer */
//     RtlFreeHeap(CsrHeap, 0, ServerCaptureBuffer);
// }
//
// /*++
//  * @name CsrValidateMessageBuffer
//  * @implemented NT5.1
//  *
//  * The CsrValidateMessageBuffer routine validates a captured message buffer
//  * present in the CSR Api Message
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message containing the CSR Capture Buffer.
//  *
//  * @param Buffer
//  *        Pointer to the message buffer to validate.
//  *
//  * @param ElementCount
//  *        Number of elements contained in the message buffer.
//  *
//  * @param ElementSize
//  *        Size of each element.
//  *
//  * @return TRUE if validation succeeded, FALSE otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// CsrValidateMessageBuffer(IN PCSR_API_MESSAGE ApiMessage,
//                          IN PVOID *Buffer,
//                          IN ULONG ElementCount,
//                          IN ULONG ElementSize)
// {
//     PCSR_CAPTURE_BUFFER CaptureBuffer = ApiMessage->CsrCaptureData;
//     SIZE_T BufferDistance = (ULONG_PTR)Buffer - (ULONG_PTR)ApiMessage;
//     ULONG PointerCount;
//     PULONG_PTR OffsetPointer;
//
//     /*
//      * Check whether we have a valid buffer pointer, elements
//      * of non-trivial size and that we don't overflow.
//      */
//     if (!Buffer || ElementSize == 0 ||
//         (ULONGLONG)ElementCount * ElementSize > (ULONGLONG)MAXULONG)
//     {
//         return FALSE;
//     }
//
//     /* Check if didn't get a buffer and there aren't any arguments to check */
//     // if (!*Buffer && (ElementCount * ElementSize == 0))
//     if (!*Buffer && ElementCount == 0) // Here ElementSize != 0 therefore only ElementCount can be == 0
//         return TRUE;
//
//     /* Check if we have no capture buffer */
//     if (!CaptureBuffer)
//     {
//         /* In this case, succeed only if the caller is CSRSS */
//         if (NtCurrentTeb()->ClientId.UniqueProcess ==
//             ApiMessage->Header.ClientId.UniqueProcess)
//         {
//             return TRUE;
//         }
//     }
//     else
//     {
//         /* Make sure that there is still space left in the capture buffer */
//         if ((CaptureBuffer->Size - (ULONG_PTR)*Buffer + (ULONG_PTR)CaptureBuffer) >=
//             (ElementCount * ElementSize))
//         {
//             /* Perform the validation test */
//             PointerCount  = CaptureBuffer->PointerCount;
//             OffsetPointer = CaptureBuffer->PointerOffsetsArray;
//             while (PointerCount--)
//             {
//                 /*
//                  * Find in the array, the pointer offset (from the
//                  * API message) that corresponds to the buffer.
//                  */
//                 if (*OffsetPointer == BufferDistance)
//                 {
//                     return TRUE;
//                 }
//                 ++OffsetPointer;
//             }
//         }
//     }
//
//     /* Failure */
// #ifdef CSR_DBG
//     DPRINT1("CSRSRV: Bad message buffer %p\n", ApiMessage);
//     if (NtCurrentPeb()->BeingDebugged) DbgBreakPoint();
// #endif
//     return FALSE;
// }
//
// /*++
//  * @name CsrValidateMessageString
//  * @implemented NT5.1
//  *
//  * The CsrValidateMessageString validates a captured Wide-Character String
//  * present in a CSR API Message.
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message containing the CSR Capture Buffer.
//  *
//  * @param MessageString
//  *        Pointer to the buffer containing the string to validate.
//  *
//  * @return TRUE if validation succeeded, FALSE otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// CsrValidateMessageString(IN PCSR_API_MESSAGE ApiMessage,
//                          IN PWSTR *MessageString)
// {
//     if (MessageString)
//     {
//         return CsrValidateMessageBuffer(ApiMessage,
//                                         (PVOID*)MessageString,
//                                         wcslen(*MessageString) + 1,
//                                         sizeof(WCHAR));
//     }
//     else
//     {
//         return FALSE;
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
