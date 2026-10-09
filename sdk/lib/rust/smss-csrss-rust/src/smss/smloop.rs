use ntdll_sys::nt::*;

const PROCESS_ALL_ACCESS: ACCESS_MASK = 0x001F0FFF;
const THREAD_ALL_ACCESS: ACCESS_MASK = 0x001F03FF;

// CONTAINING_RECORD + InitializeObjectAttributes are C macros, not
// exports. Inline equivalents.
#[inline(always)]
unsafe fn CONTAINING_RECORD(address: PVOID, field_offset: usize) -> PVOID {
    (address as *mut u8).wrapping_sub(field_offset) as PVOID
}

#[inline(always)]
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

#[repr(C)]
struct SMP_CLIENT_CONTEXT {
    Subsystem: PSMP_SUBSYSTEM,
    ProcessHandle: HANDLE,
    PortHandle: HANDLE,
    Reserved: PVOID,
}

type PSMP_CLIENT_CONTEXT = *mut SMP_CLIENT_CONTEXT;

type PSM_API_HANDLER = Option<
    unsafe extern "system" fn(
        SmApiMsg: PSM_API_MSG,
        ClientContext: PSMP_CLIENT_CONTEXT,
        SmApiPort: HANDLE,
    ) -> NTSTATUS,
>;

static mut SmTotalApiThreads: LONG = 0;
static mut SmUniqueProcessId: HANDLE = core::ptr::null_mut();

unsafe extern "system" fn SmpCreateForeignSession(
    _SmApiMsg: PSM_API_MSG,
    _ClientContext: PSMP_CLIENT_CONTEXT,
    _SmApiPort: HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe extern "system" fn SmpSessionComplete(
    _SmApiMsg: PSM_API_MSG,
    _ClientContext: PSMP_CLIENT_CONTEXT,
    _SmApiPort: HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe extern "system" fn SmpTerminateForeignSession(
    _SmApiMsg: PSM_API_MSG,
    _ClientContext: PSMP_CLIENT_CONTEXT,
    _SmApiPort: HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe extern "system" fn SmpExecPgm(
    SmApiMsg: PSM_API_MSG,
    _ClientContext: PSMP_CLIENT_CONTEXT,
    _SmApiPort: HANDLE,
) -> NTSTATUS {
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let SmExecPgm: PSM_EXEC_PGM_MSG;
    let mut ProcessInformation: RTL_USER_PROCESS_INFORMATION;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenProcess(
        &mut ProcessHandle,
        PROCESS_DUP_HANDLE,
        &mut ObjectAttributes,
        core::ptr::addr_of_mut!((*SmApiMsg).h.__bindgen_anon_1.ClientId),
    );
    if Status < 0 {
        return Status;
    }

    SmExecPgm = &mut (*SmApiMsg).u.ExecPgm;
    ProcessInformation = (*SmExecPgm).ProcessInformation;

    Status = NtDuplicateObject(
        ProcessHandle,
        (*SmExecPgm).ProcessInformation.ProcessHandle,
        NtCurrentProcess(),
        &mut ProcessInformation.ProcessHandle,
        PROCESS_ALL_ACCESS,
        0,
        0,
    );
    if Status < 0 {
        NtClose(ProcessHandle);
        return Status;
    }

    Status = NtDuplicateObject(
        ProcessHandle,
        (*SmExecPgm).ProcessInformation.ThreadHandle,
        NtCurrentProcess(),
        &mut ProcessInformation.ThreadHandle,
        THREAD_ALL_ACCESS,
        0,
        0,
    );
    if Status < 0 {
        NtClose(ProcessInformation.ProcessHandle);
        NtClose(ProcessHandle);
        return Status;
    }

    NtClose(ProcessHandle);
    return SmpSbCreateSession(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut ProcessInformation,
        0,
        if (*SmExecPgm).DebugFlag != 0 {
            &(*SmApiMsg).h.__bindgen_anon_1.ClientId as *const _ as *mut _
        } else {
            core::ptr::null_mut()
        },
    );
}

unsafe extern "system" fn SmpLoadDeferedSubsystem(
    SmApiMsg: PSM_API_MSG,
    ClientContext: PSMP_CLIENT_CONTEXT,
    _SmApiPort: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_OBJECT_NAME_NOT_FOUND;
    let SmLoadDefered: PSM_LOAD_DEFERED_SUBSYSTEM_MSG = &mut (*SmApiMsg).u.LoadDefered;
    let mut DeferedSubsystem: _UNICODE_STRING = core::mem::zeroed();
    let mut MuSessionId: ULONG = 0;
    let mut NextEntry: PLIST_ENTRY;
    let mut RegEntry: PSMP_REGISTRY_VALUE;

    if (*SmLoadDefered).Length <= 0
        || (*SmLoadDefered).Length
            > (core::mem::size_of_val(&(*SmLoadDefered).Buffer) as LONG)
                .try_into()
                .unwrap()
    {
        return STATUS_INVALID_PARAMETER;
    }

    DeferedSubsystem.Length = (*SmLoadDefered).Length as USHORT;
    DeferedSubsystem.MaximumLength = DeferedSubsystem.Length;
    DeferedSubsystem.Buffer = (*SmLoadDefered).Buffer.as_mut_ptr();

    SmpGetProcessMuSessionId((*ClientContext).ProcessHandle, &mut MuSessionId);
    if SmpCheckDuplicateMuSessionId(MuSessionId) == 0 {
        return Status;
    }

    NextEntry = SmpSubSystemsToDefer.Flink;
    while NextEntry != &raw const SmpSubSystemsToDefer as *const _ as PLIST_ENTRY {
        RegEntry = CONTAINING_RECORD(
            NextEntry as PVOID,
            core::mem::offset_of!(_SMP_REGISTRY_VALUE, Entry),
        ) as PSMP_REGISTRY_VALUE;
        if RtlEqualUnicodeString(&(*RegEntry).Name, &DeferedSubsystem, 1u8) != 0 {
            Status = SmpExecuteCommand(
                &mut (*RegEntry).Value,
                MuSessionId,
                core::ptr::null_mut(),
                SMP_SUBSYSTEM_FLAG,
            );
            break;
        }
        NextEntry = (*NextEntry).Flink;
    }

    return Status;
}

unsafe extern "system" fn SmpStartCsr(
    SmApiMsg: PSM_API_MSG,
    _ClientContext: PSMP_CLIENT_CONTEXT,
    _SmApiPort: HANDLE,
) -> NTSTATUS {
    let SmStartCsr: PSM_START_CSR_MSG = &mut (*SmApiMsg).u.StartCsr;
    let mut InitialCommand: _UNICODE_STRING = core::mem::zeroed();
    let mut InitialCommandProcess: HANDLE = core::ptr::null_mut();
    let mut InitialCommandProcessId: HANDLE = core::ptr::null_mut();
    let mut WindowsSubSysProcessId: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = SmpLoadSubSystemsForMuSession(
        &mut (*SmStartCsr).MuSessionId,
        &mut WindowsSubSysProcessId,
        &mut InitialCommand,
    );
    if Status < 0 {
        return Status;
    }

    if (*SmStartCsr).Length != 0 {
        InitialCommand.Length = (*SmStartCsr).Length as u16;
        InitialCommand.MaximumLength = (*SmStartCsr).Length as u16;
        InitialCommand.Buffer = (*SmStartCsr).Buffer.as_mut_ptr();
    }

    Status = SmpExecuteInitialCommand(
        (*SmStartCsr).MuSessionId,
        &mut InitialCommand,
        &mut InitialCommandProcess as *mut HANDLE as HANDLE,
        &mut InitialCommandProcessId,
    );
    if Status < 0 {
        return Status;
    }

    NtClose(InitialCommandProcess);

    (*SmStartCsr).WindowsSubSysProcessId = WindowsSubSysProcessId;
    (*SmStartCsr).SmpInitialCommandProcessId = InitialCommandProcessId;

    return STATUS_SUCCESS;
}

unsafe extern "system" fn SmpStopCsr(
    _SmApiMsg: PSM_API_MSG,
    _ClientContext: PSMP_CLIENT_CONTEXT,
    _SmApiPort: HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

static mut SmpApiDispatch: [PSM_API_HANDLER; 7] = [
    Some(SmpCreateForeignSession),
    Some(SmpSessionComplete),
    Some(SmpTerminateForeignSession),
    Some(SmpExecPgm),
    Some(SmpLoadDeferedSubsystem),
    Some(SmpStartCsr),
    Some(SmpStopCsr),
];

unsafe extern "system" fn SmpHandleConnectionRequest(
    _SmApiPort: HANDLE,
    SbApiMsg: PSB_API_MSG,
) -> NTSTATUS {
    let mut Accept: BOOLEAN = 1u8;
    let mut PortHandle: HANDLE = core::ptr::null_mut();
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut SessionIdLocal: ULONG = 0;
    let mut SubsystemPort: _UNICODE_STRING = core::mem::zeroed();
    let ClientContext: PSMP_CLIENT_CONTEXT;
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PortView: REMOTE_PORT_VIEW = core::mem::zeroed();
    let mut SecurityQos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let CidSubsystem: PSMP_SUBSYSTEM;
    let TypeSubsystem: PSMP_SUBSYSTEM;

    SecurityQos.ImpersonationLevel = SecurityIdentification;
    SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
    SecurityQos.EffectiveOnly = 1u8;

    if (*SbApiMsg).h.__bindgen_anon_1.ClientId.UniqueProcess == SmUniqueProcessId {
        ProcessHandle = core::ptr::null_mut();
        SessionIdLocal = 0;
    } else {
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        Status = NtOpenProcess(
            &mut ProcessHandle,
            PROCESS_QUERY_INFORMATION,
            &mut ObjectAttributes,
            core::ptr::addr_of_mut!((*SbApiMsg).h.__bindgen_anon_1.ClientId),
        );
        if Status < 0 {
            Accept = 0u8;
        }

        SmpGetProcessMuSessionId(ProcessHandle, &mut SessionIdLocal);
    }

    CidSubsystem = SmpLocateKnownSubSysByCid(core::ptr::addr_of_mut!(
        (*SbApiMsg).h.__bindgen_anon_1.ClientId
    ));
    if !CidSubsystem.is_null() && Accept != 0 {
        TypeSubsystem = SmpLocateKnownSubSysByType(
            SessionIdLocal,
            (*SbApiMsg).__bindgen_anon_1.ConnectionInfo.SubsystemType,
        );
        if TypeSubsystem == CidSubsystem {
            Accept = 0u8;
        } else {
            (*CidSubsystem).ImageType = (*SbApiMsg).__bindgen_anon_1.ConnectionInfo.SubsystemType;
        }

        if !TypeSubsystem.is_null() {
            SmpDereferenceSubsystem(TypeSubsystem);
        }
    }

    if Accept != 0 {
        ClientContext = RtlAllocateHeap(
            SmpHeap,
            0,
            core::mem::size_of::<SMP_CLIENT_CONTEXT>()
                .try_into()
                .unwrap(),
        ) as PSMP_CLIENT_CONTEXT;
        if !ClientContext.is_null() {
            (*ClientContext).ProcessHandle = ProcessHandle;
            (*ClientContext).Subsystem = CidSubsystem;
            (*ClientContext).Reserved = core::ptr::null_mut();
            (*ClientContext).PortHandle = core::ptr::null_mut();
        } else {
            Accept = 0u8;
        }
    } else {
        ClientContext = SbApiMsg as PSMP_CLIENT_CONTEXT;
    }

    PortView.Length = core::mem::size_of::<REMOTE_PORT_VIEW>() as ULONG;
    Status = NtAcceptConnectPort(
        &mut PortHandle,
        ClientContext as PVOID,
        core::ptr::addr_of_mut!((*SbApiMsg).h),
        Accept,
        core::ptr::null_mut(),
        &mut PortView,
    );
    if Accept == 0 || Status < 0 {
        if ClientContext != SbApiMsg as PVOID as PSMP_CLIENT_CONTEXT {
            RtlFreeHeap(SmpHeap, 0, ClientContext as PVOID);
        }
        if !ProcessHandle.is_null() {
            NtClose(ProcessHandle);
        }
        if !CidSubsystem.is_null() {
            SmpDereferenceSubsystem(CidSubsystem);
        }
        return Status;
    }

    if !ClientContext.is_null() {
        (*ClientContext).PortHandle = PortHandle;
    }
    if !CidSubsystem.is_null() {
        (*CidSubsystem).PortHandle = PortHandle;
    }

    Status = NtCompleteConnectPort(PortHandle);
    if Status >= 0 && !CidSubsystem.is_null() {
        (*SbApiMsg).__bindgen_anon_1.ConnectionInfo.SbApiPortName[119] = 0u16;
        RtlCreateUnicodeString(
            &mut SubsystemPort,
            (*SbApiMsg)
                .__bindgen_anon_1
                .ConnectionInfo
                .SbApiPortName
                .as_ptr(),
        );
        Status = NtConnectPort(
            &mut (*CidSubsystem).SbApiPort,
            &mut SubsystemPort,
            &mut SecurityQos,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        RtlFreeUnicodeString(&mut SubsystemPort);

        NtSetEvent((*CidSubsystem).Event, core::ptr::null_mut());
    } else if !CidSubsystem.is_null() {
        (*CidSubsystem).PortHandle = core::ptr::null_mut();
    }

    if !CidSubsystem.is_null() {
        SmpDereferenceSubsystem(CidSubsystem);
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SmpApiLoop(Parameter: PVOID) -> ULONG {
    let SmApiPort: HANDLE = Parameter as HANDLE;
    let mut Status: NTSTATUS;
    let mut ClientContext: PSMP_CLIENT_CONTEXT = core::ptr::null_mut();
    let mut ReplyMsg: PSM_API_MSG = core::ptr::null_mut();
    let mut RequestMsg: SM_API_MSG = core::mem::zeroed();
    let mut ProcessInformation: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();

    core::sync::atomic::AtomicI32::from_ptr(&raw const SmTotalApiThreads as *const _ as *mut i32)
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst);

    RtlSetThreadIsCritical(1u8, core::ptr::null_mut(), 1u8);

    NtQueryInformationProcess(
        NtCurrentProcess(),
        ProcessBasicInformation,
        &mut ProcessInformation as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    SmUniqueProcessId = ProcessInformation.UniqueProcessId as HANDLE;

    loop {
        Status = NtReplyWaitReceivePort(
            SmApiPort,
            &mut ClientContext as *mut _ as *mut PVOID,
            if !ReplyMsg.is_null() {
                &mut (*ReplyMsg).h
            } else {
                core::ptr::null_mut()
            },
            &mut RequestMsg.h,
        );
        if Status == STATUS_NO_MEMORY {
            Timeout.QuadPart = -50000000i64;
            NtDelayExecution(0u8, &mut Timeout);
            continue;
        }

        match RequestMsg.h.u2.s2.Type as u32 {
            x if x == LPC_CONNECTION_REQUEST => {
                SmpHandleConnectionRequest(SmApiPort, &mut RequestMsg as *mut _ as PSB_API_MSG);
                ReplyMsg = core::ptr::null_mut();
            }
            x if x == LPC_PORT_CLOSED => {
                ReplyMsg = core::ptr::null_mut();
            }
            _ => {
                if ClientContext.is_null() {
                    ReplyMsg = core::ptr::null_mut();
                    continue;
                }

                RequestMsg.ReturnValue = STATUS_PENDING;

                if RequestMsg.ApiNumber >= SmpMaxApiNumber {
                    Status = STATUS_NOT_IMPLEMENTED;
                } else if RequestMsg.ApiNumber <= SmpTerminateForeignSessionApi
                    && (*ClientContext).Subsystem.is_null()
                {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    Status = (SmpApiDispatch[RequestMsg.ApiNumber as usize].unwrap())(
                        &mut RequestMsg,
                        ClientContext,
                        SmApiPort,
                    );
                }

                RequestMsg.ReturnValue = Status;
                ReplyMsg = &mut RequestMsg;
            }
        }
    }
    0
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/smloop.c
//  * PURPOSE:         Main SMSS Code
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "smss.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// typedef struct _SMP_CLIENT_CONTEXT
// {
//     PSMP_SUBSYSTEM Subsystem;
//     HANDLE ProcessHandle;
//     HANDLE PortHandle;
//     PVOID Reserved;
// } SMP_CLIENT_CONTEXT, *PSMP_CLIENT_CONTEXT;
//
// typedef
// NTSTATUS
// (NTAPI *PSM_API_HANDLER)(
//     _In_ PSM_API_MSG SmApiMsg,
//     _In_ PSMP_CLIENT_CONTEXT ClientContext,
//     _In_ HANDLE SmApiPort);
//
// volatile LONG SmTotalApiThreads;
// HANDLE SmUniqueProcessId;
//
// /* API HANDLERS ***************************************************************/
//
// NTSTATUS
// NTAPI
// SmpCreateForeignSession(IN PSM_API_MSG SmApiMsg,
//                         IN PSMP_CLIENT_CONTEXT ClientContext,
//                         IN HANDLE SmApiPort)
// {
//     DPRINT1("%s is not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// SmpSessionComplete(IN PSM_API_MSG SmApiMsg,
//                    IN PSMP_CLIENT_CONTEXT ClientContext,
//                    IN HANDLE SmApiPort)
// {
//     DPRINT1("%s is not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// SmpTerminateForeignSession(IN PSM_API_MSG SmApiMsg,
//                            IN PSMP_CLIENT_CONTEXT ClientContext,
//                            IN HANDLE SmApiPort)
// {
//     DPRINT1("%s is not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// SmpExecPgm(IN PSM_API_MSG SmApiMsg,
//            IN PSMP_CLIENT_CONTEXT ClientContext,
//            IN HANDLE SmApiPort)
// {
//     HANDLE ProcessHandle;
//     NTSTATUS Status;
//     PSM_EXEC_PGM_MSG SmExecPgm;
//     RTL_USER_PROCESS_INFORMATION ProcessInformation;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Open the client process */
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     Status = NtOpenProcess(&ProcessHandle,
//                            PROCESS_DUP_HANDLE,
//                            &ObjectAttributes,
//                            &SmApiMsg->h.ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT1("SmExecPgm: NtOpenProcess Failed %lx\n", Status);
//         return Status;
//     }
//
//     /* Copy the process information out of the message */
//     SmExecPgm = &SmApiMsg->u.ExecPgm;
//     ProcessInformation = SmExecPgm->ProcessInformation;
//
//     /* Duplicate the process handle */
//     Status = NtDuplicateObject(ProcessHandle,
//                                SmExecPgm->ProcessInformation.ProcessHandle,
//                                NtCurrentProcess(),
//                                &ProcessInformation.ProcessHandle,
//                                PROCESS_ALL_ACCESS,
//                                0,
//                                0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Close the handle and fail */
//         NtClose(ProcessHandle);
//         DPRINT1("SmExecPgm: NtDuplicateObject (Process) Failed %lx\n", Status);
//         return Status;
//     }
//
//     /* Duplicate the thread handle */
//     Status = NtDuplicateObject(ProcessHandle,
//                                SmExecPgm->ProcessInformation.ThreadHandle,
//                                NtCurrentProcess(),
//                                &ProcessInformation.ThreadHandle,
//                                THREAD_ALL_ACCESS,
//                                0,
//                                0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Close both handles and fail */
//         NtClose(ProcessInformation.ProcessHandle);
//         NtClose(ProcessHandle);
//         DPRINT1("SmExecPgm: NtDuplicateObject (Thread) Failed %lx\n", Status);
//         return Status;
//     }
//
//     /* Close the process handle and call the internal client API */
//     NtClose(ProcessHandle);
//     return SmpSbCreateSession(NULL,
//                               NULL,
//                               &ProcessInformation,
//                               0,
//                               SmExecPgm->DebugFlag ? &SmApiMsg->h.ClientId : NULL);
// }
//
// NTSTATUS
// NTAPI
// SmpLoadDeferedSubsystem(
//     _In_ PSM_API_MSG SmApiMsg,
//     _In_ PSMP_CLIENT_CONTEXT ClientContext,
//     _In_ HANDLE SmApiPort)
// {
//     NTSTATUS Status = STATUS_OBJECT_NAME_NOT_FOUND;
//     PSM_LOAD_DEFERED_SUBSYSTEM_MSG SmLoadDefered = &SmApiMsg->u.LoadDefered;
//     UNICODE_STRING DeferedSubsystem;
//     ULONG MuSessionId;
//     PLIST_ENTRY NextEntry;
//     PSMP_REGISTRY_VALUE RegEntry;
//
//     /* Validate DeferedSubsystem's length */
//     if ((SmLoadDefered->Length <= 0) ||
//         (SmLoadDefered->Length > sizeof(SmLoadDefered->Buffer)))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get the name of the subsystem to start */
//     DeferedSubsystem.Length = (USHORT)SmLoadDefered->Length;
//     DeferedSubsystem.MaximumLength = DeferedSubsystem.Length;
//     DeferedSubsystem.Buffer = SmLoadDefered->Buffer;
//
//     /* Find a subsystem responsible for this session */
//     SmpGetProcessMuSessionId(ClientContext->ProcessHandle, &MuSessionId);
//     if (!SmpCheckDuplicateMuSessionId(MuSessionId))
//     {
//         DPRINT1("SMSS: Deferred subsystem load (%wZ) for MuSessionId %u, status=0x%x\n",
//                 &DeferedSubsystem, MuSessionId, Status);
//         return Status;
//     }
//
//     /* Now process the deferred subsystems list */
//     for (NextEntry = SmpSubSystemsToDefer.Flink;
//          NextEntry != &SmpSubSystemsToDefer;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get each entry and check if it's the subsystem we are looking for */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         if (RtlEqualUnicodeString(&RegEntry->Name, &DeferedSubsystem, TRUE))
//         {
//             // TODO: One may want to extra-flag the command for
//             // specific POSIX or OS2 processing...
//
//             /* Load the deferred subsystem */
//             Status = SmpExecuteCommand(&RegEntry->Value,
//                                        MuSessionId,
//                                        NULL,
//                                        SMP_SUBSYSTEM_FLAG);
//             if (!NT_SUCCESS(Status))
//                 DPRINT1("SMSS: Subsystem execute failed (%wZ)\n", &RegEntry->Value);
//
//             break;
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpStartCsr(IN PSM_API_MSG SmApiMsg,
//             IN PSMP_CLIENT_CONTEXT ClientContext,
//             IN HANDLE SmApiPort)
// {
//     PSM_START_CSR_MSG SmStartCsr = &SmApiMsg->u.StartCsr;
//     UNICODE_STRING InitialCommand;
//     HANDLE InitialCommandProcess, InitialCommandProcessId, WindowsSubSysProcessId;
//     NTSTATUS Status;
//
//     Status = SmpLoadSubSystemsForMuSession(&SmStartCsr->MuSessionId,
//                                            &WindowsSubSysProcessId,
//                                            &InitialCommand);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: SmpLoadSubSystemsForMuSession failed with status 0x%08x\n", Status);
//         return Status;
//     }
//
//     if (SmStartCsr->Length)
//     {
//         InitialCommand.Length = InitialCommand.MaximumLength = SmStartCsr->Length;
//         InitialCommand.Buffer = SmStartCsr->Buffer;
//     }
//
//     Status = SmpExecuteInitialCommand(SmStartCsr->MuSessionId,
//                                       &InitialCommand,
//                                       &InitialCommandProcess,
//                                       &InitialCommandProcessId);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: SmpExecuteInitialCommand failed with status 0x%08x\n", Status);
//         /* FIXME: undo effects of SmpLoadSubSystemsForMuSession */
//         ASSERT(FALSE);
//         return Status;
//     }
//
//     NtClose(InitialCommandProcess);
//
//     SmStartCsr->WindowsSubSysProcessId = WindowsSubSysProcessId;
//     SmStartCsr->SmpInitialCommandProcessId = InitialCommandProcessId;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpStopCsr(IN PSM_API_MSG SmApiMsg,
//            IN PSMP_CLIENT_CONTEXT ClientContext,
//            IN HANDLE SmApiPort)
// {
//     DPRINT1("%s is not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// PSM_API_HANDLER SmpApiDispatch[SmpMaxApiNumber - SmpCreateForeignSessionApi] =
// {
//     SmpCreateForeignSession,
//     SmpSessionComplete,
//     SmpTerminateForeignSession,
//     SmpExecPgm,
//     SmpLoadDeferedSubsystem,
//     SmpStartCsr,
//     SmpStopCsr
// };
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// SmpHandleConnectionRequest(IN HANDLE SmApiPort,
//                            IN PSB_API_MSG SbApiMsg)
// {
//     BOOLEAN Accept = TRUE;
//     HANDLE PortHandle, ProcessHandle;
//     ULONG SessionId;
//     UNICODE_STRING SubsystemPort;
//     PSMP_CLIENT_CONTEXT ClientContext;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     REMOTE_PORT_VIEW PortView;
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//     PSMP_SUBSYSTEM CidSubsystem, TypeSubsystem;
//
//     /* Initialize QoS data */
//     SecurityQos.ImpersonationLevel = SecurityIdentification;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
//     SecurityQos.EffectiveOnly = TRUE;
//
//     /* Check if this is SM connecting to itself */
//     if (SbApiMsg->h.ClientId.UniqueProcess == SmUniqueProcessId)
//     {
//         /* No need to get any handle -- assume session 0 */
//         ProcessHandle = NULL;
//         SessionId = 0;
//     }
//     else
//     {
//         /* Reference the foreign process */
//         InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//         Status = NtOpenProcess(&ProcessHandle,
//                                PROCESS_QUERY_INFORMATION,
//                                &ObjectAttributes,
//                                &SbApiMsg->h.ClientId);
//         if (!NT_SUCCESS(Status)) Accept = FALSE;
//
//         /* Get its session ID */
//         SmpGetProcessMuSessionId(ProcessHandle, &SessionId);
//     }
//
//     /* See if we already know about the caller's subsystem */
//     CidSubsystem = SmpLocateKnownSubSysByCid(&SbApiMsg->h.ClientId);
//     if ((CidSubsystem) && (Accept))
//     {
//         /* Check if we already have a subsystem for this kind of image */
//         TypeSubsystem = SmpLocateKnownSubSysByType(SessionId,
//                                                    SbApiMsg->ConnectionInfo.SubsystemType);
//         if (TypeSubsystem == CidSubsystem)
//         {
//             /* Someone is trying to take control of an existing subsystem, fail */
//             Accept = FALSE;
//             DPRINT1("SMSS: Connection from SubSystem rejected\n");
//             DPRINT1("SMSS: Image type already being served\n");
//         }
//         else
//         {
//             /* Set this image type as the type for this subsystem */
//             CidSubsystem->ImageType = SbApiMsg->ConnectionInfo.SubsystemType;
//         }
//
//         /* Drop the reference we had acquired */
//         if (TypeSubsystem) SmpDereferenceSubsystem(TypeSubsystem);
//     }
//
//     /* Check if we'll be accepting the connection */
//     if (Accept)
//     {
//         /* We will, so create a client context for it */
//         ClientContext = RtlAllocateHeap(SmpHeap, 0, sizeof(SMP_CLIENT_CONTEXT));
//         if (ClientContext)
//         {
//             ClientContext->ProcessHandle = ProcessHandle;
//             ClientContext->Subsystem = CidSubsystem;
//             ClientContext->Reserved = NULL;
//             ClientContext->PortHandle = NULL;
//         }
//         else
//         {
//             /* Failed to allocate a client context, so reject the connection */
//             DPRINT1("Rejecting connection due to lack of memory\n");
//             Accept = FALSE;
//         }
//     }
//     else
//     {
//         /* Use a bogus context since we're going to reject the message */
//         ClientContext = (PSMP_CLIENT_CONTEXT)SbApiMsg;
//     }
//
//     /* Now send the actual accept reply (which could be a rejection) */
//     PortView.Length = sizeof(PortView);
//     Status = NtAcceptConnectPort(&PortHandle,
//                                  ClientContext,
//                                  &SbApiMsg->h,
//                                  Accept,
//                                  NULL,
//                                  &PortView);
//     if (!(Accept) || !(NT_SUCCESS(Status)))
//     {
//         /* Close the process handle, reference the subsystem, and exit */
//         DPRINT1("Accept failed or rejected: %lx\n", Status);
//         if (ClientContext != (PVOID)SbApiMsg) RtlFreeHeap(SmpHeap, 0, ClientContext);
//         if (ProcessHandle) NtClose(ProcessHandle);
//         if (CidSubsystem) SmpDereferenceSubsystem(CidSubsystem);
//         return Status;
//     }
//
//     /* Save the port handle now that we've accepted it */
//     if (ClientContext) ClientContext->PortHandle = PortHandle;
//     if (CidSubsystem) CidSubsystem->PortHandle = PortHandle;
//
//     /* Complete the port connection */
//     Status = NtCompleteConnectPort(PortHandle);
//     if ((NT_SUCCESS(Status)) && (CidSubsystem))
//     {
//         /* This was an actual subsystem, so connect back to it */
//         SbApiMsg->ConnectionInfo.SbApiPortName[119] = UNICODE_NULL;
//         RtlCreateUnicodeString(&SubsystemPort,
//                                SbApiMsg->ConnectionInfo.SbApiPortName);
//         Status = NtConnectPort(&CidSubsystem->SbApiPort,
//                                &SubsystemPort,
//                                &SecurityQos,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("SMSS: Connect back to Sb %wZ failed %lx\n", &SubsystemPort, Status);
//         }
//         RtlFreeUnicodeString(&SubsystemPort);
//
//         /* Now that we're connected, signal the event handle */
//         NtSetEvent(CidSubsystem->Event, NULL);
//     }
//     else if (CidSubsystem)
//     {
//         /* We failed to complete the connection, so clear the port handle */
//         DPRINT1("Completing the connection failed: %lx\n", Status);
//         CidSubsystem->PortHandle = NULL;
//     }
//
//     /* Dereference the subsystem and return the result */
//     if (CidSubsystem) SmpDereferenceSubsystem(CidSubsystem);
//     return Status;
// }
//
// ULONG
// NTAPI
// SmpApiLoop(IN PVOID Parameter)
// {
//     HANDLE SmApiPort = (HANDLE)Parameter;
//     NTSTATUS Status;
//     PSMP_CLIENT_CONTEXT ClientContext;
//     PSM_API_MSG ReplyMsg = NULL;
//     SM_API_MSG RequestMsg;
//     PROCESS_BASIC_INFORMATION ProcessInformation;
//     LARGE_INTEGER Timeout;
//
//     /* Increase the number of API threads for throttling code for later */
//     _InterlockedExchangeAdd(&SmTotalApiThreads, 1);
//
//     /* Mark us critical */
//     RtlSetThreadIsCritical(TRUE, NULL, TRUE);
//
//     /* Set the PID of the SM process itself for later checking */
//     NtQueryInformationProcess(NtCurrentProcess(),
//                               ProcessBasicInformation,
//                               &ProcessInformation,
//                               sizeof(ProcessInformation),
//                               NULL);
//     SmUniqueProcessId = (HANDLE)ProcessInformation.UniqueProcessId;
//
//     /* Now process incoming messages */
//     while (TRUE)
//     {
//         /* Begin waiting on a request */
//         Status = NtReplyWaitReceivePort(SmApiPort,
//                                         (PVOID*)&ClientContext,
//                                         &ReplyMsg->h,
//                                         &RequestMsg.h);
//         if (Status == STATUS_NO_MEMORY)
//         {
//             /* Ran out of memory, so do a little timeout and try again */
//             if (ReplyMsg) DPRINT1("SMSS: Failed to reply to calling thread, retrying.\n");
//             Timeout.QuadPart = -50000000;
//             NtDelayExecution(FALSE, &Timeout);
//             continue;
//         }
//
//         /* Check what kind of request we received */
//         switch (RequestMsg.h.u2.s2.Type)
//         {
//             /* A new connection */
//             case LPC_CONNECTION_REQUEST:
//                 /* Create the right structures for it */
//                 SmpHandleConnectionRequest(SmApiPort, (PSB_API_MSG)&RequestMsg);
//                 ReplyMsg = NULL;
//                 break;
//
//             /* A closed connection */
//             case LPC_PORT_CLOSED:
//                 /* Destroy any state we had for this client */
//                 DPRINT1("Port closed\n");
//                 //if (ClientContext) SmpPushDeferredClientContext(ClientContext);
//                 ReplyMsg = NULL;
//                 break;
//
//             /* An actual API message */
//             default:
//                 if (!ClientContext)
//                 {
//                     ReplyMsg = NULL;
//                     break;
//                 }
//
//                 RequestMsg.ReturnValue = STATUS_PENDING;
//
//                 /* Check if the API is valid */
//                 if (RequestMsg.ApiNumber >= SmpMaxApiNumber)
//                 {
//                     /* It isn't, fail */
//                     DPRINT1("Invalid API: %lx\n", RequestMsg.ApiNumber);
//                     Status = STATUS_NOT_IMPLEMENTED;
//                 }
//                 else if ((RequestMsg.ApiNumber <= SmpTerminateForeignSessionApi) &&
//                          !(ClientContext->Subsystem))
//                 {
//                     /* It's valid, but doesn't have a subsystem with it */
//                     DPRINT1("Invalid session API\n");
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//                 else
//                 {
//                     /* It's totally okay, so call the dispatcher for it */
//                     Status = SmpApiDispatch[RequestMsg.ApiNumber](&RequestMsg,
//                                                                   ClientContext,
//                                                                   SmApiPort);
//                 }
//
//                 /* Write the result value and return the message back */
//                 RequestMsg.ReturnValue = Status;
//                 ReplyMsg = &RequestMsg;
//                 break;
//         }
//     }
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
