use ntdll_sys::nt::*;

// User-mode constants the rustconv translation referenced as if extern;
// they're macros in C and not in ntdll-sys bindgen output. Hand-pinned
// here. Values from <ntpsapi.h> / <ntstatus.h>.
const PROCESS_ALL_ACCESS: ACCESS_MASK = 0x001F0FFF;
const THREAD_ALL_ACCESS: ACCESS_MASK = 0x001F03FF;
const STATUS_WAIT_0: NTSTATUS = 0x00000000;
const STATUS_WAIT_1: NTSTATUS = 0x00000001;
const STATUS_NO_SUCH_PACKAGE: NTSTATUS = 0xC00000FEu32 as NTSTATUS;

#[inline]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// RtlCopyMemory is a memmove macro, not an export.
#[inline]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// RtlGetProcessHeap is a TEB inline (NtCurrentTeb()->ProcessEnvironmentBlock->ProcessHeap).
// Not exported; re-translate.
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

// _wcsicmp is a C CRT helper (cdecl, exported from libcntpr / ntdll).
extern "C" {
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
}

// RtlStringCbCatW is `static __inline NTSTATUS NTAPI` from
// ddk/ntstrsafe.h — not a real export. Inline equivalent that
// concatenates wide-char src onto NUL-terminated dst within cb_dst
// bytes. Used to assemble the initial command line.
#[inline(always)]
unsafe fn RtlStringCbCatW(dst: *mut u16, cb_dst: usize, src: *const u16) -> NTSTATUS {
    let max_chars = cb_dst / 2;
    if max_chars == 0 {
        return 0xC0000023u32 as NTSTATUS;
    } // STATUS_BUFFER_TOO_SMALL
    let mut dst_len = 0usize;
    while dst_len < max_chars && *dst.add(dst_len) != 0 {
        dst_len += 1;
    }
    if dst_len >= max_chars {
        return 0xC0000023u32 as NTSTATUS;
    }
    let mut i = 0usize;
    while *src.add(i) != 0 && dst_len + i + 1 < max_chars {
        *dst.add(dst_len + i) = *src.add(i);
        i += 1;
    }
    *dst.add(dst_len + i) = 0;
    if *src.add(i) != 0 {
        0xC0000023u32 as NTSTATUS
    } else {
        0
    }
}

// CONTAINING_RECORD is a C macro (`(type*)((char*)ptr - offsetof(type, field))`).
// Call sites pass `core::mem::offset_of!(T, Field)` as the third arg.
#[inline(always)]
unsafe fn CONTAINING_RECORD(
    ptr: PLIST_ENTRY,
    _ty: usize,
    field_offset: usize,
) -> *mut core::ffi::c_void {
    (ptr as *mut u8).wrapping_sub(field_offset) as *mut core::ffi::c_void
}

// Exported so cohabited C sminit.c (still on the rustconv translate
// queue) can read/write them via `extern` declarations.
#[no_mangle]
pub static mut SmpKnownSubSysLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpKnownSubSysHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut SmpWindowsSubSysProcess: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut SmpWindowsSubSysProcessId: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut RegPosixSingleInstance: BOOLEAN = 0u8;
static mut InitialCommandBuffer: [u16; 256] = [0; 256];

#[no_mangle]
pub unsafe extern "system" fn SmpCallCsrCreateProcess(
    SbApiMsg: PSB_API_MSG,
    MessageLength: USHORT,
    PortHandle: HANDLE,
) -> NTSTATUS {
    (*SbApiMsg).h.u2.ZeroInit = 0;
    (*SbApiMsg).h.u1.s1.DataLength = (MessageLength + 8) as i16;
    (*SbApiMsg).h.u1.s1.TotalLength = core::mem::size_of::<SB_API_MSG>() as i16;
    (*SbApiMsg).__bindgen_anon_1.__bindgen_anon_1.ApiNumber = SbpCreateProcess as SB_API_NUMBER;
    let mut Status = NtRequestWaitReplyPort(
        PortHandle,
        core::ptr::addr_of_mut!((*SbApiMsg).h) as *mut _,
        core::ptr::addr_of_mut!((*SbApiMsg).h) as *mut _,
    );
    if Status >= 0 {
        Status = (*SbApiMsg).__bindgen_anon_1.__bindgen_anon_1.ReturnValue;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpDereferenceSubsystem(SubSystem: PSMP_SUBSYSTEM) {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));

    (*SubSystem).ReferenceCount -= 1;
    if (*SubSystem).ReferenceCount == 0 && (*SubSystem).Terminating != 0 {
        if !(*SubSystem).Event.is_null() {
            NtClose((*SubSystem).Event);
        }
        if !(*SubSystem).ProcessHandle.is_null() {
            NtClose((*SubSystem).ProcessHandle);
        }
        if !(*SubSystem).SbApiPort.is_null() {
            NtClose((*SubSystem).SbApiPort);
        }
        RtlFreeHeap(SmpHeap, 0, SubSystem as PVOID);
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
}

#[no_mangle]
pub unsafe extern "system" fn SmpLocateKnownSubSysByCid(ClientId: PCLIENT_ID) -> PSMP_SUBSYSTEM {
    let mut Subsystem: PSMP_SUBSYSTEM = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));

    NextEntry = SmpKnownSubSysHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpKnownSubSysHead) {
        Subsystem = CONTAINING_RECORD(NextEntry, 0, core::mem::offset_of!(SMP_SUBSYSTEM, Entry))
            as PSMP_SUBSYSTEM;

        if *(core::ptr::addr_of!((*Subsystem).ClientId) as PULONGLONG) == *(ClientId as PULONGLONG)
            && (*Subsystem).Terminating == 0
        {
            (*Subsystem).ReferenceCount += 1;
            break;
        }

        Subsystem = core::ptr::null_mut();
        NextEntry = (*NextEntry).Flink;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
    Subsystem
}

#[no_mangle]
pub unsafe extern "system" fn SmpLocateKnownSubSysByType(
    MuSessionId: ULONG,
    ImageType: ULONG,
) -> PSMP_SUBSYSTEM {
    let mut Subsystem: PSMP_SUBSYSTEM = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));

    NextEntry = SmpKnownSubSysHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpKnownSubSysHead) {
        Subsystem = CONTAINING_RECORD(NextEntry, 0, core::mem::offset_of!(SMP_SUBSYSTEM, Entry))
            as PSMP_SUBSYSTEM;

        if (*Subsystem).ImageType == ImageType
            && (*Subsystem).Terminating == 0
            && (*Subsystem).MuSessionId == MuSessionId
        {
            (*Subsystem).ReferenceCount += 1;
            break;
        }

        Subsystem = core::ptr::null_mut();
        NextEntry = (*NextEntry).Flink;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
    Subsystem
}

#[no_mangle]
pub unsafe extern "system" fn SmpLoadSubSystem(
    FileName: PUNICODE_STRING,
    Directory: PUNICODE_STRING,
    CommandLine: PUNICODE_STRING,
    mut MuSessionId: ULONG,
    ProcessId: PHANDLE,
    Flags: ULONG,
) -> NTSTATUS {
    let mut Subsystem: PSMP_SUBSYSTEM;
    let mut NewSubsystem: PSMP_SUBSYSTEM;
    let mut KnownSubsystem: PSMP_SUBSYSTEM = core::ptr::null_mut();
    let mut SubSysProcessId: HANDLE;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SbApiMsg: SB_API_MSG = core::mem::zeroed();
    let mut ProcessInformation: RTL_USER_PROCESS_INFORMATION = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut State: PVOID = core::ptr::null_mut();

    if Flags & SMP_INVALID_PATH != 0 {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    if Flags & 0x80 != 0 {
        MuSessionId = 0;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
    loop {
        Subsystem = SmpLocateKnownSubSysByType(MuSessionId, 0xFFFFFFFF);
        if Subsystem.is_null() {
            break;
        }
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));

        NtWaitForSingleObject((*Subsystem).Event, 0u8, core::ptr::null_mut());

        RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
        SmpDereferenceSubsystem(Subsystem);
    }

    if Flags & SMP_POSIX_FLAG != 0 {
        Subsystem = SmpLocateKnownSubSysByType(MuSessionId, IMAGE_SUBSYSTEM_POSIX_CUI);
    } else if Flags & SMP_OS2_FLAG != 0 {
        Subsystem = SmpLocateKnownSubSysByType(MuSessionId, IMAGE_SUBSYSTEM_OS2_CUI);
    } else {
        Subsystem = core::ptr::null_mut();
    }

    if !Subsystem.is_null() {
        SmpDereferenceSubsystem(Subsystem);
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
        return STATUS_SUCCESS;
    }

    NewSubsystem = RtlAllocateHeap(
        SmpHeap,
        SmBaseTag as ULONG,
        core::mem::size_of::<SMP_SUBSYSTEM>().try_into().unwrap(),
    ) as PSMP_SUBSYSTEM;
    if NewSubsystem.is_null() {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
        return STATUS_NO_MEMORY;
    }

    (*NewSubsystem).ReferenceCount = 1;
    (*NewSubsystem).MuSessionId = MuSessionId;
    (*NewSubsystem).ImageType = 0xFFFFFFFF;
    (*NewSubsystem).Terminating = 0u8;
    (*NewSubsystem).ProcessHandle = core::ptr::null_mut();
    (*NewSubsystem).Event = core::ptr::null_mut();
    (*NewSubsystem).PortHandle = core::ptr::null_mut();
    (*NewSubsystem).SbApiPort = core::ptr::null_mut();

    Status = NtCreateEvent(
        core::ptr::addr_of_mut!((*NewSubsystem).Event),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );
    if Status < 0 {
        RtlFreeHeap(SmpHeap, 0, NewSubsystem as PVOID);
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
        return STATUS_NO_MEMORY;
    }

    InsertTailList(
        core::ptr::addr_of_mut!(SmpKnownSubSysHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY,
    );
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));

    if Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG) != 0 {
        KnownSubsystem = SmpLocateKnownSubSysByType(MuSessionId, IMAGE_SUBSYSTEM_WINDOWS_GUI);
        if KnownSubsystem.is_null() {
            Status = STATUS_UNSUCCESSFUL;
            'Quickie2: {
                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }

        let CreateProcess =
            core::ptr::addr_of_mut!(SbApiMsg.__bindgen_anon_1.__bindgen_anon_1.u.CreateProcess);
        (*CreateProcess).__bindgen_anon_1.In.ImageName = FileName;
        (*CreateProcess).__bindgen_anon_1.In.CurrentDirectory = Directory;
        (*CreateProcess).__bindgen_anon_1.In.CommandLine = CommandLine;
        (*CreateProcess).__bindgen_anon_1.In.DllPath = if SmpDefaultLibPath.Length != 0 {
            core::ptr::addr_of_mut!(SmpDefaultLibPath)
        } else {
            core::ptr::null_mut()
        };
        (*CreateProcess).__bindgen_anon_1.In.Flags = Flags | SMP_DEFERRED_FLAG;
        (*CreateProcess).__bindgen_anon_1.In.DebugFlags = SmpDebug as ULONG;
        Status = SmpCallCsrCreateProcess(
            core::ptr::addr_of_mut!(SbApiMsg),
            core::mem::size_of::<SB_CREATE_PROCESS_MSG>() as u16,
            (*KnownSubsystem).SbApiPort,
        );
        if Status < 0 {
            'Quickie2: {
                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }

        ProcessInformation.ProcessHandle = (*CreateProcess).__bindgen_anon_1.Out.ProcessHandle;
        ProcessInformation.ThreadHandle = (*CreateProcess).__bindgen_anon_1.Out.ThreadHandle;
        ProcessInformation.ClientId = (*CreateProcess).__bindgen_anon_1.Out.ClientId;
        ProcessInformation.ImageInformation.SubSystemType =
            (*CreateProcess).__bindgen_anon_1.Out.SubsystemType;
    } else {
        Status = SmpExecuteImage(
            FileName,
            Directory,
            CommandLine,
            MuSessionId,
            Flags | SMP_DEFERRED_FLAG,
            core::ptr::addr_of_mut!(ProcessInformation),
        );
        if Status < 0 {
            'Quickie2: {
                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }
    }

    (*NewSubsystem).ProcessHandle = ProcessInformation.ProcessHandle;
    (*NewSubsystem).ClientId = ProcessInformation.ClientId;

    if ProcessInformation.ImageInformation.SubSystemType == IMAGE_SUBSYSTEM_NATIVE {
        SubSysProcessId = ProcessInformation.ClientId.UniqueProcess;
        if !ProcessId.is_null() && (*ProcessId).is_null() {
            *ProcessId = SubSysProcessId;
        }

        if MuSessionId == 0 {
            SmpWindowsSubSysProcessId = SubSysProcessId;
            SmpWindowsSubSysProcess = ProcessInformation.ProcessHandle;
        }
    } else {
        let CreateSession =
            core::ptr::addr_of_mut!(SbApiMsg.__bindgen_anon_1.__bindgen_anon_1.u.CreateSession);
        (*CreateSession).ProcessInfo = ProcessInformation;
        (*CreateSession).DbgSessionId = 0;
        *(core::ptr::addr_of_mut!((*CreateSession).DbgUiClientId) as PULONGLONG) = 0;

        Subsystem = SmpLocateKnownSubSysByType(
            MuSessionId,
            ProcessInformation.ImageInformation.SubSystemType,
        );
        if Subsystem.is_null() {
            Status = STATUS_NO_SUCH_PACKAGE;
            'Quickie: {
                if AttachedSessionId == 0xFFFFFFFF || (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG) != 0)
                {
                } else {
                    Status = SmpAcquirePrivilege(
                        SE_LOAD_DRIVER_PRIVILEGE,
                        core::ptr::addr_of_mut!(State),
                    );
                    if Status >= 0 {
                        Status = NtSetSystemInformation(
                            SystemSessionDetach,
                            core::ptr::addr_of_mut!(AttachedSessionId) as PVOID,
                            core::mem::size_of::<ULONG>() as ULONG,
                        );
                        if Status >= 0 {
                            AttachedSessionId = 0xFFFFFFFF;
                        }
                        SmpReleasePrivilege(State);
                    }
                }

                NtTerminateProcess(ProcessInformation.ProcessHandle, Status);
                NtClose(ProcessInformation.ThreadHandle);

                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }

        Status = NtDuplicateObject(
            NtCurrentProcess(),
            ProcessInformation.ProcessHandle,
            (*Subsystem).ProcessHandle,
            core::ptr::addr_of_mut!((*CreateSession).ProcessInfo.ProcessHandle),
            PROCESS_ALL_ACCESS,
            0,
            0,
        );
        if Status < 0 {
            'Quickie: {
                if AttachedSessionId == 0xFFFFFFFF || (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG) != 0)
                {
                } else {
                    Status = SmpAcquirePrivilege(
                        SE_LOAD_DRIVER_PRIVILEGE,
                        core::ptr::addr_of_mut!(State),
                    );
                    if Status >= 0 {
                        Status = NtSetSystemInformation(
                            SystemSessionDetach,
                            core::ptr::addr_of_mut!(AttachedSessionId) as PVOID,
                            core::mem::size_of::<ULONG>() as ULONG,
                        );
                        if Status >= 0 {
                            AttachedSessionId = 0xFFFFFFFF;
                        }
                        SmpReleasePrivilege(State);
                    }
                }

                NtTerminateProcess(ProcessInformation.ProcessHandle, Status);
                NtClose(ProcessInformation.ThreadHandle);

                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }

        Status = NtDuplicateObject(
            NtCurrentProcess(),
            ProcessInformation.ThreadHandle,
            (*Subsystem).ProcessHandle,
            core::ptr::addr_of_mut!((*CreateSession).ProcessInfo.ThreadHandle),
            THREAD_ALL_ACCESS,
            0,
            0,
        );
        if Status < 0 {
            'Quickie: {
                if AttachedSessionId == 0xFFFFFFFF || (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG) != 0)
                {
                } else {
                    Status = SmpAcquirePrivilege(
                        SE_LOAD_DRIVER_PRIVILEGE,
                        core::ptr::addr_of_mut!(State),
                    );
                    if Status >= 0 {
                        Status = NtSetSystemInformation(
                            SystemSessionDetach,
                            core::ptr::addr_of_mut!(AttachedSessionId) as PVOID,
                            core::mem::size_of::<ULONG>() as ULONG,
                        );
                        if Status >= 0 {
                            AttachedSessionId = 0xFFFFFFFF;
                        }
                        SmpReleasePrivilege(State);
                    }
                }

                NtTerminateProcess(ProcessInformation.ProcessHandle, Status);
                NtClose(ProcessInformation.ThreadHandle);

                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }

        (*CreateSession).SessionId = SmpAllocateSessionId(Subsystem, core::ptr::null_mut());

        SbApiMsg.__bindgen_anon_1.__bindgen_anon_1.ReturnValue = STATUS_SUCCESS;
        SbApiMsg.h.u2.ZeroInit = 0;
        SbApiMsg.h.u1.s1.DataLength = (core::mem::size_of::<SB_CREATE_SESSION_MSG>() + 8) as i16;
        SbApiMsg.h.u1.s1.TotalLength = core::mem::size_of::<SB_API_MSG>() as i16;
        SbApiMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = SbpCreateSession as SB_API_NUMBER;
        Status = NtRequestWaitReplyPort(
            (*Subsystem).SbApiPort,
            core::ptr::addr_of_mut!(SbApiMsg.h) as *mut _,
            core::ptr::addr_of_mut!(SbApiMsg.h) as *mut _,
        );
        if Status >= 0 {
            Status = SbApiMsg.__bindgen_anon_1.__bindgen_anon_1.ReturnValue;
        }
        if Status < 0 {
            SmpDeleteSession((*CreateSession).SessionId);
            'Quickie: {
                if AttachedSessionId == 0xFFFFFFFF || (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG) != 0)
                {
                } else {
                    Status = SmpAcquirePrivilege(
                        SE_LOAD_DRIVER_PRIVILEGE,
                        core::ptr::addr_of_mut!(State),
                    );
                    if Status >= 0 {
                        Status = NtSetSystemInformation(
                            SystemSessionDetach,
                            core::ptr::addr_of_mut!(AttachedSessionId) as PVOID,
                            core::mem::size_of::<ULONG>() as ULONG,
                        );
                        if Status >= 0 {
                            AttachedSessionId = 0xFFFFFFFF;
                        }
                        SmpReleasePrivilege(State);
                    }
                }

                NtTerminateProcess(ProcessInformation.ProcessHandle, Status);
                NtClose(ProcessInformation.ThreadHandle);

                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }
    }

    Status = NtResumeThread(ProcessInformation.ThreadHandle, core::ptr::null_mut());
    if Status < 0 {
        'Quickie: {
            if AttachedSessionId == 0xFFFFFFFF || (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG) != 0) {
            } else {
                Status =
                    SmpAcquirePrivilege(SE_LOAD_DRIVER_PRIVILEGE, core::ptr::addr_of_mut!(State));
                if Status >= 0 {
                    Status = NtSetSystemInformation(
                        SystemSessionDetach,
                        core::ptr::addr_of_mut!(AttachedSessionId) as PVOID,
                        core::mem::size_of::<ULONG>() as ULONG,
                    );
                    if Status >= 0 {
                        AttachedSessionId = 0xFFFFFFFF;
                    }
                    SmpReleasePrivilege(State);
                }
            }

            NtTerminateProcess(ProcessInformation.ProcessHandle, Status);
            NtClose(ProcessInformation.ThreadHandle);

            RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
            if !Subsystem.is_null() {
                SmpDereferenceSubsystem(Subsystem);
            }
            if !KnownSubsystem.is_null() {
                SmpDereferenceSubsystem(KnownSubsystem);
            }
            if Status < 0 {
                RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                SmpDereferenceSubsystem(NewSubsystem);
            }
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
            return Status;
        }
    }

    if MuSessionId != 0 {
        Timeout.QuadPart = -600000000;
        Status =
            NtWaitForSingleObject((*NewSubsystem).Event, 0u8, core::ptr::addr_of_mut!(Timeout));

        if SmpCheckDuplicateMuSessionId(MuSessionId) == 0 {
            return STATUS_DELETE_PENDING;
        }

        if Status != STATUS_WAIT_0 {
            'Quickie: {
                if AttachedSessionId == 0xFFFFFFFF || (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG) != 0)
                {
                } else {
                    Status = SmpAcquirePrivilege(
                        SE_LOAD_DRIVER_PRIVILEGE,
                        core::ptr::addr_of_mut!(State),
                    );
                    if Status >= 0 {
                        Status = NtSetSystemInformation(
                            SystemSessionDetach,
                            core::ptr::addr_of_mut!(AttachedSessionId) as PVOID,
                            core::mem::size_of::<ULONG>() as ULONG,
                        );
                        if Status >= 0 {
                            AttachedSessionId = 0xFFFFFFFF;
                        }
                        SmpReleasePrivilege(State);
                    }
                }

                NtTerminateProcess(ProcessInformation.ProcessHandle, Status);
                NtClose(ProcessInformation.ThreadHandle);

                RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                if !Subsystem.is_null() {
                    SmpDereferenceSubsystem(Subsystem);
                }
                if !KnownSubsystem.is_null() {
                    SmpDereferenceSubsystem(KnownSubsystem);
                }
                if Status < 0 {
                    RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
                    NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
                    SmpDereferenceSubsystem(NewSubsystem);
                }
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
                return Status;
            }
        }
    } else {
        NtWaitForSingleObject((*NewSubsystem).Event, 0u8, core::ptr::null_mut());
    }

    NtClose(ProcessInformation.ThreadHandle);
    Status = STATUS_SUCCESS;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
    if !Subsystem.is_null() {
        SmpDereferenceSubsystem(Subsystem);
    }
    if !KnownSubsystem.is_null() {
        SmpDereferenceSubsystem(KnownSubsystem);
    }
    if Status < 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*NewSubsystem).Entry) as PLIST_ENTRY);
        NtSetEvent((*NewSubsystem).Event, core::ptr::null_mut());
        SmpDereferenceSubsystem(NewSubsystem);
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(SmpKnownSubSysLock));
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpLoadSubSystemsForMuSession(
    MuSessionId: *mut ULONG,
    ProcessId: PHANDLE,
    InitialCommand: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Status2: NTSTATUS;
    let mut RegEntry: PSMP_REGISTRY_VALUE;
    let mut DestinationString: _UNICODE_STRING = core::mem::zeroed();
    let mut NtPath: _UNICODE_STRING = core::mem::zeroed();
    let mut NextEntry: PLIST_ENTRY;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut State: PVOID = core::ptr::null_mut();

    SmpTranslateSystemPartitionInformation();

    NextEntry = SmpSetupExecuteList.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpSetupExecuteList) {
        RegEntry = CONTAINING_RECORD(
            NextEntry,
            0,
            core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry),
        ) as PSMP_REGISTRY_VALUE;
        SmpExecuteCommand(
            core::ptr::addr_of_mut!((*RegEntry).Name),
            0,
            core::ptr::null_mut(),
            0,
        );
        NextEntry = (*NextEntry).Flink;
    }

    NextEntry = SmpSubSystemList.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpSubSystemList) {
        RegEntry = CONTAINING_RECORD(
            NextEntry,
            0,
            core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry),
        ) as PSMP_REGISTRY_VALUE;

        static KMODE: [u16; 6] = [
            b'K' as u16,
            b'm' as u16,
            b'o' as u16,
            b'd' as u16,
            b'e' as u16,
            0,
        ];
        if _wcsicmp((*RegEntry).Name.Buffer, KMODE.as_ptr()) == 0 {
            if RtlDosPathNameToNtPathName_U(
                (*RegEntry).Value.Buffer,
                core::ptr::addr_of_mut!(NtPath),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == 0
            {
                Status = STATUS_OBJECT_PATH_SYNTAX_BAD;
            } else {
                Status =
                    SmpAcquirePrivilege(SE_LOAD_DRIVER_PRIVILEGE, core::ptr::addr_of_mut!(State));
                if Status >= 0 {
                    Status = NtSetSystemInformation(
                        SystemSessionCreate,
                        MuSessionId as PVOID,
                        core::mem::size_of::<ULONG>() as ULONG,
                    );
                    if Status < 0 {
                        SmpReleasePrivilege(State);
                        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPath.Buffer as PVOID);
                        return Status;
                    }
                    AttachedSessionId = *MuSessionId;

                    static WIN32K_PATH: [u16; 32] = [
                        b'\\' as u16,
                        b'S' as u16,
                        b'y' as u16,
                        b's' as u16,
                        b't' as u16,
                        b'e' as u16,
                        b'm' as u16,
                        b'R' as u16,
                        b'o' as u16,
                        b'o' as u16,
                        b't' as u16,
                        b'\\' as u16,
                        b'S' as u16,
                        b'y' as u16,
                        b's' as u16,
                        b't' as u16,
                        b'e' as u16,
                        b'm' as u16,
                        b'3' as u16,
                        b'2' as u16,
                        b'\\' as u16,
                        b'w' as u16,
                        b'i' as u16,
                        b'n' as u16,
                        b'3' as u16,
                        b'2' as u16,
                        b'k' as u16,
                        b'.' as u16,
                        b's' as u16,
                        b'y' as u16,
                        b's' as u16,
                        0,
                    ];
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!(DestinationString),
                        WIN32K_PATH.as_ptr(),
                    );
                    Status = NtSetSystemInformation(
                        SystemExtendServiceTableInformation,
                        core::ptr::addr_of_mut!(DestinationString) as PVOID,
                        core::mem::size_of::<_UNICODE_STRING>() as ULONG,
                    );
                    RtlFreeHeap(RtlGetProcessHeap(), 0, NtPath.Buffer as PVOID);
                    SmpReleasePrivilege(State);
                    if Status < 0 {
                        return Status;
                    }
                }
            }
        }

        NextEntry = (*NextEntry).Flink;
    }

    NextEntry = SmpSubSystemsToLoad.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpSubSystemsToLoad) {
        RegEntry = CONTAINING_RECORD(
            NextEntry,
            0,
            core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry),
        ) as PSMP_REGISTRY_VALUE;

        static DEBUG: [u16; 6] = [
            b'D' as u16,
            b'e' as u16,
            b'b' as u16,
            b'u' as u16,
            b'g' as u16,
            0,
        ];
        if _wcsicmp((*RegEntry).Name.Buffer, DEBUG.as_ptr()) == 0 {
            Status = SmpExecuteCommand(
                core::ptr::addr_of_mut!((*RegEntry).Value),
                *MuSessionId,
                ProcessId,
                SMP_DEBUG_FLAG | SMP_SUBSYSTEM_FLAG,
            );
        } else {
            Status = SmpExecuteCommand(
                core::ptr::addr_of_mut!((*RegEntry).Value),
                *MuSessionId,
                ProcessId,
                SMP_SUBSYSTEM_FLAG,
            );
        }
        if Status < 0 {
            return Status;
        }

        NextEntry = (*NextEntry).Flink;
    }

    NextEntry = SmpExecuteList.Blink;
    if NextEntry != core::ptr::addr_of_mut!(SmpExecuteList) {
        RegEntry = CONTAINING_RECORD(
            NextEntry,
            0,
            core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry),
        ) as PSMP_REGISTRY_VALUE;

        *InitialCommand = (*RegEntry).Name;
        Timeout.QuadPart = -50000000;
        NtDelayExecution(0u8, core::ptr::addr_of_mut!(Timeout));
    } else {
        static WINLOGON: [u16; 13] = [
            b'w' as u16,
            b'i' as u16,
            b'n' as u16,
            b'l' as u16,
            b'o' as u16,
            b'g' as u16,
            b'o' as u16,
            b'n' as u16,
            b'.' as u16,
            b'e' as u16,
            b'x' as u16,
            b'e' as u16,
            0,
        ];
        RtlInitUnicodeString(InitialCommand, WINLOGON.as_ptr());
        InitialCommandBuffer[0] = 0;

        static DEBUGGER: [u16; 9] = [
            b'D' as u16,
            b'e' as u16,
            b'b' as u16,
            b'u' as u16,
            b'g' as u16,
            b'g' as u16,
            b'e' as u16,
            b'r' as u16,
            0,
        ];
        Status2 = LdrQueryImageFileExecutionOptions(
            InitialCommand,
            DEBUGGER.as_ptr(),
            REG_SZ,
            InitialCommandBuffer.as_mut_ptr() as PVOID,
            (core::mem::size_of::<[u16; 256]>() - (*InitialCommand).Length as usize) as ULONG,
            core::ptr::null_mut(),
        );
        if Status2 >= 0 && InitialCommandBuffer[0] != 0 {
            RtlStringCbCatW(
                InitialCommandBuffer.as_mut_ptr(),
                core::mem::size_of::<[u16; 256]>(),
                b" \0".as_ptr() as *const u16,
            );
            RtlStringCbCatW(
                InitialCommandBuffer.as_mut_ptr(),
                core::mem::size_of::<[u16; 256]>(),
                (*InitialCommand).Buffer,
            );
            RtlInitUnicodeString(InitialCommand, InitialCommandBuffer.as_ptr());
        }
    }

    NextEntry = SmpExecuteList.Flink;
    while NextEntry != core::ptr::addr_of_mut!(SmpExecuteList) {
        RegEntry = CONTAINING_RECORD(
            NextEntry,
            0,
            core::mem::offset_of!(SMP_REGISTRY_VALUE, Entry),
        ) as PSMP_REGISTRY_VALUE;
        SmpExecuteCommand(
            core::ptr::addr_of_mut!((*RegEntry).Name),
            *MuSessionId,
            core::ptr::null_mut(),
            0,
        );
        NextEntry = (*NextEntry).Flink;
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/smsubsys.c
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
// RTL_CRITICAL_SECTION SmpKnownSubSysLock;
// LIST_ENTRY SmpKnownSubSysHead;
// HANDLE SmpWindowsSubSysProcess;
// HANDLE SmpWindowsSubSysProcessId;
// BOOLEAN RegPosixSingleInstance;
// WCHAR InitialCommandBuffer[256];
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// SmpCallCsrCreateProcess(IN PSB_API_MSG SbApiMsg,
//                         IN USHORT MessageLength,
//                         IN HANDLE PortHandle)
// {
//     NTSTATUS Status;
//
//     /* Initialize the header and send the message to CSRSS */
//     SbApiMsg->h.u2.ZeroInit = 0;
//     SbApiMsg->h.u1.s1.DataLength = MessageLength + 8;
//     SbApiMsg->h.u1.s1.TotalLength = sizeof(SB_API_MSG);
//     SbApiMsg->ApiNumber = SbpCreateProcess;
//     Status = NtRequestWaitReplyPort(PortHandle, &SbApiMsg->h, &SbApiMsg->h);
//     if (NT_SUCCESS(Status)) Status = SbApiMsg->ReturnValue;
//     return Status;
// }
//
// VOID
// NTAPI
// SmpDereferenceSubsystem(IN PSMP_SUBSYSTEM SubSystem)
// {
//     /* Acquire the database lock while we (potentially) destroy this subsystem */
//     RtlEnterCriticalSection(&SmpKnownSubSysLock);
//
//     /* Drop the reference and see if it's terminating */
//     if (!(--SubSystem->ReferenceCount) && (SubSystem->Terminating))
//     {
//         /* Close all handles and free it */
//         if (SubSystem->Event) NtClose(SubSystem->Event);
//         if (SubSystem->ProcessHandle) NtClose(SubSystem->ProcessHandle);
//         if (SubSystem->SbApiPort) NtClose(SubSystem->SbApiPort);
//         RtlFreeHeap(SmpHeap, 0, SubSystem);
//     }
//
//     /* Release the database lock */
//     RtlLeaveCriticalSection(&SmpKnownSubSysLock);
// }
//
// PSMP_SUBSYSTEM
// NTAPI
// SmpLocateKnownSubSysByCid(IN PCLIENT_ID ClientId)
// {
//     PSMP_SUBSYSTEM Subsystem = NULL;
//     PLIST_ENTRY NextEntry;
//
//     /* Lock the subsystem database */
//     RtlEnterCriticalSection(&SmpKnownSubSysLock);
//
//     /* Loop each subsystem in the database */
//     NextEntry = SmpKnownSubSysHead.Flink;
//     while (NextEntry != &SmpKnownSubSysHead)
//     {
//         /* Check if this one matches the client ID and is still valid */
//         Subsystem = CONTAINING_RECORD(NextEntry, SMP_SUBSYSTEM, Entry);
//         if ((*(PULONGLONG)&Subsystem->ClientId == *(PULONGLONG)ClientId) &&
//             !(Subsystem->Terminating))
//         {
//             /* Add a reference and return it */
//             Subsystem->ReferenceCount++;
//             break;
//         }
//
//         /* Reset the current pointer and keep searching */
//         Subsystem = NULL;
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the lock and return the subsystem we found */
//     RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//     return Subsystem;
// }
//
// PSMP_SUBSYSTEM
// NTAPI
// SmpLocateKnownSubSysByType(IN ULONG MuSessionId,
//                            IN ULONG ImageType)
// {
//     PSMP_SUBSYSTEM Subsystem = NULL;
//     PLIST_ENTRY NextEntry;
//
//     /* Lock the subsystem database */
//     RtlEnterCriticalSection(&SmpKnownSubSysLock);
//
//     /* Loop each subsystem in the database */
//     NextEntry = SmpKnownSubSysHead.Flink;
//     while (NextEntry != &SmpKnownSubSysHead)
//     {
//         /* Check if this one matches the image and uID, and is still valid */
//         Subsystem = CONTAINING_RECORD(NextEntry, SMP_SUBSYSTEM, Entry);
//         if ((Subsystem->ImageType == ImageType) &&
//             !(Subsystem->Terminating) &&
//             (Subsystem->MuSessionId == MuSessionId))
//         {
//             /* Return it referenced for the caller */
//             Subsystem->ReferenceCount++;
//             break;
//         }
//
//         /* Reset the current pointer and keep searching */
//         Subsystem = NULL;
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Release the lock and return the subsystem we found */
//     RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//     return Subsystem;
// }
//
// NTSTATUS
// NTAPI
// SmpLoadSubSystem(IN PUNICODE_STRING FileName,
//                  IN PUNICODE_STRING Directory,
//                  IN PUNICODE_STRING CommandLine,
//                  IN ULONG MuSessionId,
//                  OUT PHANDLE ProcessId,
//                  IN ULONG Flags)
// {
//     PSMP_SUBSYSTEM Subsystem, NewSubsystem, KnownSubsystem = NULL;
//     HANDLE SubSysProcessId;
//     NTSTATUS Status = STATUS_SUCCESS;
//     SB_API_MSG SbApiMsg;
//     RTL_USER_PROCESS_INFORMATION ProcessInformation;
//     LARGE_INTEGER Timeout;
//     PVOID State;
//     PSB_CREATE_PROCESS_MSG CreateProcess = &SbApiMsg.u.CreateProcess;
//     PSB_CREATE_SESSION_MSG CreateSession = &SbApiMsg.u.CreateSession;
//
//     /* Make sure this is a found subsystem */
//     if (Flags & SMP_INVALID_PATH)
//     {
//         DPRINT1("SMSS: Unable to find subsystem - %wZ\n", FileName);
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /* Don't use a session if the flag is set */
//     if (Flags & 0x80) MuSessionId = 0;
//
//     /* Lock the subsystems while we do a look up */
//     RtlEnterCriticalSection(&SmpKnownSubSysLock);
//     while (TRUE)
//     {
//         /* Check if we found a subsystem not yet fully initialized */
//         Subsystem = SmpLocateKnownSubSysByType(MuSessionId, -1);
//         if (!Subsystem) break;
//         RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//
//         /* Wait on it to initialize */
//         NtWaitForSingleObject(Subsystem->Event, FALSE, NULL);
//
//         /* Dereference it and try the next one */
//         RtlEnterCriticalSection(&SmpKnownSubSysLock);
//         SmpDereferenceSubsystem(Subsystem);
//     }
//
//     /* Check if this is a POSIX subsystem */
//     if (Flags & SMP_POSIX_FLAG)
//     {
//         /* Do we already have it? */
//         Subsystem = SmpLocateKnownSubSysByType(MuSessionId, IMAGE_SUBSYSTEM_POSIX_CUI);
//     }
//     else if (Flags & SMP_OS2_FLAG)
//     {
//         /* This is an OS/2 subsystem, do we we already have it? */
//         Subsystem = SmpLocateKnownSubSysByType(MuSessionId, IMAGE_SUBSYSTEM_OS2_CUI);
//     }
//
//     /* Check if we already have one of the optional subsystems for the session */
//     if (Subsystem)
//     {
//         /* Dereference and return, no work to do */
//         SmpDereferenceSubsystem(Subsystem);
//         RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//         return STATUS_SUCCESS;
//     }
//
//     /* Allocate a new subsystem! */
//     NewSubsystem = RtlAllocateHeap(SmpHeap, SmBaseTag, sizeof(SMP_SUBSYSTEM));
//     if (!NewSubsystem)
//     {
//         RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Initialize its header and reference count */
//     NewSubsystem->ReferenceCount = 1;
//     NewSubsystem->MuSessionId = MuSessionId;
//     NewSubsystem->ImageType = -1;
//
//     /* Clear out all the other data for now */
//     NewSubsystem->Terminating = FALSE;
//     NewSubsystem->ProcessHandle = NULL;
//     NewSubsystem->Event = NULL;
//     NewSubsystem->PortHandle = NULL;
//     NewSubsystem->SbApiPort = NULL;
//
//     /* Create the event we'll be waiting on for initialization */
//     Status = NtCreateEvent(&NewSubsystem->Event,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            NotificationEvent,
//                            FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* This failed, bail out */
//         RtlFreeHeap(SmpHeap, 0, NewSubsystem);
//         RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Insert the subsystem and release the lock. It can now be found */
//     InsertTailList(&SmpKnownSubSysHead, &NewSubsystem->Entry);
//     RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//
//     /* The OS/2 and POSIX subsystems are actually Windows applications! */
//     if (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG))
//     {
//         /* Locate the Windows subsystem for this session */
//         KnownSubsystem = SmpLocateKnownSubSysByType(MuSessionId,
//                                                     IMAGE_SUBSYSTEM_WINDOWS_GUI);
//         if (!KnownSubsystem)
//         {
//             DPRINT1("SMSS: SmpLoadSubSystem - SmpLocateKnownSubSysByType Failed\n");
//             goto Quickie2;
//         }
//
//         /* Fill out all the process details and call CSRSS to launch it */
//         CreateProcess->In.ImageName = FileName;
//         CreateProcess->In.CurrentDirectory = Directory;
//         CreateProcess->In.CommandLine = CommandLine;
//         CreateProcess->In.DllPath = SmpDefaultLibPath.Length ?
//                                     &SmpDefaultLibPath : NULL;
//         CreateProcess->In.Flags = Flags | SMP_DEFERRED_FLAG;
//         CreateProcess->In.DebugFlags = SmpDebug;
//         Status = SmpCallCsrCreateProcess(&SbApiMsg,
//                                          sizeof(*CreateProcess),
//                                          KnownSubsystem->SbApiPort);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Handle failures */
//             DPRINT1("SMSS: SmpLoadSubSystem - SmpCallCsrCreateProcess Failed with  Status %lx\n",
//                     Status);
//             goto Quickie2;
//         }
//
//         /* Save the process information we'll need for the create session */
//         ProcessInformation.ProcessHandle = CreateProcess->Out.ProcessHandle;
//         ProcessInformation.ThreadHandle = CreateProcess->Out.ThreadHandle;
//         ProcessInformation.ClientId = CreateProcess->Out.ClientId;
//         ProcessInformation.ImageInformation.SubSystemType = CreateProcess->Out.SubsystemType;
//     }
//     else
//     {
//         /* This must be CSRSS itself, so just launch it and that's it */
//         DPRINT1("SMSS: SmpLoadSubSystem - calling SmpExecuteImage for '%wZ'\n", FileName);
//         Status = SmpExecuteImage(FileName,
//                                  Directory,
//                                  CommandLine,
//                                  MuSessionId,
//                                  Flags | SMP_DEFERRED_FLAG,
//                                  &ProcessInformation);
//         DPRINT1("SMSS: SmpLoadSubSystem - SmpExecuteImage returned 0x%lx\n", Status);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Handle failures */
//             DPRINT1("SMSS: SmpLoadSubSystem - SmpExecuteImage Failed with  Status %lx\n",
//                     Status);
//             goto Quickie2;
//         }
//     }
//
//     /* Fill out the handle and client ID in the subsystem structure now */
//     NewSubsystem->ProcessHandle = ProcessInformation.ProcessHandle;
//     NewSubsystem->ClientId = ProcessInformation.ClientId;
//
//     /* Check if we launched a native image or a subsystem-backed image */
//     if (ProcessInformation.ImageInformation.SubSystemType == IMAGE_SUBSYSTEM_NATIVE)
//     {
//         /* This must be CSRSS itself, since it's a native subsystem image */
//         SubSysProcessId = ProcessInformation.ClientId.UniqueProcess;
//         if ((ProcessId) && !(*ProcessId)) *ProcessId = SubSysProcessId;
//
//         /* Was this the initial CSRSS on Session 0? */
//         if (!MuSessionId)
//         {
//             /* Then save it in the global variables */
//             SmpWindowsSubSysProcessId = SubSysProcessId;
//             SmpWindowsSubSysProcess = ProcessInformation.ProcessHandle;
//         }
//         ASSERT(NT_SUCCESS(Status));
//     }
//     else
//     {
//         /* This is the POSIX or OS/2 subsystem process, copy its information */
//         CreateSession->ProcessInfo = ProcessInformation;
//
//         CreateSession->DbgSessionId = 0;
//         *(PULONGLONG)&CreateSession->DbgUiClientId = 0;
//
//         /* This should find CSRSS because they are POSIX or OS/2 subsystems */
//         Subsystem = SmpLocateKnownSubSysByType(MuSessionId,
//                                                ProcessInformation.ImageInformation.SubSystemType);
//         if (!Subsystem)
//         {
//             /* Odd failure -- but handle it anyway */
//             Status = STATUS_NO_SUCH_PACKAGE;
//             DPRINT1("SMSS: SmpLoadSubSystem - SmpLocateKnownSubSysByType Failed with  Status %lx for sessionid %lu\n",
//                     Status,
//                     MuSessionId);
//             goto Quickie;
//         }
//
//         /* Duplicate the parent process handle for the subsystem to have */
//         Status = NtDuplicateObject(NtCurrentProcess(),
//                                    ProcessInformation.ProcessHandle,
//                                    Subsystem->ProcessHandle,
//                                    &CreateSession->ProcessInfo.ProcessHandle,
//                                    PROCESS_ALL_ACCESS,
//                                    0,
//                                    0);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail since this is critical */
//             DPRINT1("SMSS: SmpLoadSubSystem - NtDuplicateObject Failed with  Status %lx for sessionid %lu\n",
//                     Status,
//                     MuSessionId);
//             goto Quickie;
//         }
//
//         /* Duplicate the initial thread handle for the subsystem to have */
//         Status = NtDuplicateObject(NtCurrentProcess(),
//                                    ProcessInformation.ThreadHandle,
//                                    Subsystem->ProcessHandle,
//                                    &CreateSession->ProcessInfo.ThreadHandle,
//                                    THREAD_ALL_ACCESS,
//                                    0,
//                                    0);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail since this is critical */
//             DPRINT1("SMSS: SmpLoadSubSystem - NtDuplicateObject Failed with  Status %lx for sessionid %lu\n",
//                     Status,
//                     MuSessionId);
//             goto Quickie;
//         }
//
//         /* Allocate an internal Session ID for this subsystem */
//         CreateSession->SessionId = SmpAllocateSessionId(Subsystem, NULL);
//
//         /* Send the create session message to the subsystem */
//         SbApiMsg.ReturnValue = STATUS_SUCCESS;
//         SbApiMsg.h.u2.ZeroInit = 0;
//         SbApiMsg.h.u1.s1.DataLength = sizeof(SB_CREATE_SESSION_MSG) + 8;
//         SbApiMsg.h.u1.s1.TotalLength = sizeof(SB_API_MSG);
//         SbApiMsg.ApiNumber = SbpCreateSession;
//         Status = NtRequestWaitReplyPort(Subsystem->SbApiPort,
//                                         &SbApiMsg.h,
//                                         &SbApiMsg.h);
//         if (NT_SUCCESS(Status)) Status = SbApiMsg.ReturnValue;
//         if (!NT_SUCCESS(Status))
//         {
//             /* Delete the session and handle failure if the LPC call failed */
//             SmpDeleteSession(CreateSession->SessionId);
//             DPRINT1("SMSS: SmpLoadSubSystem - NtRequestWaitReplyPort Failed with  Status %lx for sessionid %lu\n",
//                     Status,
//                     MuSessionId);
//             goto Quickie;
//         }
//     }
//
//     /* Okay, everything looks good to go, initialize this subsystem now! */
//     DPRINT1("SMSS: SmpLoadSubSystem - resuming thread and waiting for subsystem connect (session %lu)\n", MuSessionId);
//     Status = NtResumeThread(ProcessInformation.ThreadHandle, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* That didn't work -- back out of everything */
//         DPRINT1("SMSS: SmpLoadSubSystem - NtResumeThread failed Status %lx\n", Status);
//         goto Quickie;
//     }
//
//     /* Check if this was the subsystem for a different session */
//     if (MuSessionId)
//     {
//         /* Wait up to 60 seconds for it to initialize */
//         Timeout.QuadPart = -600000000;
//         Status = NtWaitForSingleObject(NewSubsystem->Event, FALSE, &Timeout);
//
//         /* Timeout is done -- does this session still exist? */
//         if (!SmpCheckDuplicateMuSessionId(MuSessionId))
//         {
//             /* Nope, it died. Cleanup should've ocurred in a different path. */
//             DPRINT1("SMSS: SmpLoadSubSystem - session deleted\n");
//             return STATUS_DELETE_PENDING;
//         }
//
//         /* Check if we timed our or there was another error with the wait */
//         if (Status != STATUS_WAIT_0)
//         {
//             /* Something is wrong with the subsystem, so back out of everything */
//             DPRINT1("SMSS: SmpLoadSubSystem - Timeout waiting for subsystem connect with Status %lx for sessionid %lu\n",
//                     Status,
//                     MuSessionId);
//             goto Quickie;
//         }
//     }
//     else
//     {
//         /* This a session 0 subsystem, just wait for it to initialize */
//         NtWaitForSingleObject(NewSubsystem->Event, FALSE, NULL);
//     }
//
//     /* Subsystem is created, resumed, and initialized. Close handles and exit */
//     NtClose(ProcessInformation.ThreadHandle);
//     Status = STATUS_SUCCESS;
//     goto Quickie2;
//
// Quickie:
//     /* This is the failure path. First check if we need to detach from session */
//     if ((AttachedSessionId == -1) || (Flags & (SMP_POSIX_FLAG | SMP_OS2_FLAG)))
//     {
//         /* We were not attached, or did not launch subsystems that required it */
//         DPRINT1("SMSS: Did not detach from Session Space: SessionId=%x Flags=%x Status=%x\n",
//                 AttachedSessionId,
//                 Flags | SMP_DEFERRED_FLAG,
//                 Status);
//     }
//     else
//     {
//         /* Get the privilege we need for detachment */
//         Status = SmpAcquirePrivilege(SE_LOAD_DRIVER_PRIVILEGE, &State);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We can't detach without it */
//             DPRINT1("SMSS: Did not detach from Session Space: SessionId=%x Flags=%x Status=%x\n",
//                     AttachedSessionId,
//                     Flags | SMP_DEFERRED_FLAG,
//                     Status);
//         }
//         else
//         {
//             /* Now detach from the session */
//             Status = NtSetSystemInformation(SystemSessionDetach,
//                                             &AttachedSessionId,
//                                             sizeof(AttachedSessionId));
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Failed to detach. Note the DPRINT1 has a typo in Windows */
//                 DPRINT1("SMSS: SmpStartCsr, Couldn't Detach from Session Space. Status=%x\n", Status);
//                 ASSERT(NT_SUCCESS(Status));
//             }
//             else
//             {
//                 /* Detachment worked, reset our attached session ID */
//                 AttachedSessionId = -1;
//             }
//
//             /* And release the privilege we acquired */
//             SmpReleasePrivilege(State);
//         }
//     }
//
//     /* Since this is the failure path, terminate the subsystem process */
//     NtTerminateProcess(ProcessInformation.ProcessHandle, Status);
//     NtClose(ProcessInformation.ThreadHandle);
//
// Quickie2:
//     /* This is the cleanup path -- first dereference our subsystems */
//     RtlEnterCriticalSection(&SmpKnownSubSysLock);
//     if (Subsystem) SmpDereferenceSubsystem(Subsystem);
//     if (KnownSubsystem) SmpDereferenceSubsystem(KnownSubsystem);
//
//     /* In the failure case, destroy the new subsystem we just created */
//     if (!NT_SUCCESS(Status))
//     {
//         RemoveEntryList(&NewSubsystem->Entry);
//         NtSetEvent(NewSubsystem->Event, NULL);
//         SmpDereferenceSubsystem(NewSubsystem);
//     }
//
//     /* Finally, we're all done! */
//     RtlLeaveCriticalSection(&SmpKnownSubSysLock);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpLoadSubSystemsForMuSession(IN PULONG MuSessionId,
//                               OUT PHANDLE ProcessId,
//                               IN PUNICODE_STRING InitialCommand)
// {
//     NTSTATUS Status = STATUS_SUCCESS, Status2;
//     PSMP_REGISTRY_VALUE RegEntry;
//     UNICODE_STRING DestinationString, NtPath;
//     PLIST_ENTRY NextEntry;
//     LARGE_INTEGER Timeout;
//     PVOID State;
//
//     /* Write a few last registry keys with the boot partition information */
//     SmpTranslateSystemPartitionInformation();
//
//     /* Process "SetupExecute" values */
//     NextEntry = SmpSetupExecuteList.Flink;
//     while (NextEntry != &SmpSetupExecuteList)
//     {
//         /* Execute each one and move on */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         SmpExecuteCommand(&RegEntry->Name, 0, NULL, 0);
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Now process the subsystems */
//     NextEntry = SmpSubSystemList.Flink;
//     while (NextEntry != &SmpSubSystemList)
//     {
//         /* Get the entry and check if this is the special Win32k entry */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         if (_wcsicmp(RegEntry->Name.Buffer, L"Kmode") == 0)
//         {
//             /* Translate it */
//             if (!RtlDosPathNameToNtPathName_U(RegEntry->Value.Buffer,
//                                               &NtPath,
//                                               NULL,
//                                               NULL))
//             {
//                 Status = STATUS_OBJECT_PATH_SYNTAX_BAD;
//                 DPRINT1("Failed: %lx\n", Status);
//             }
//             else
//             {
//                 /* Get the driver privilege */
//                 Status = SmpAcquirePrivilege(SE_LOAD_DRIVER_PRIVILEGE, &State);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Create the new session */
//                     ASSERT(AttachedSessionId == -1);
//                     Status = NtSetSystemInformation(SystemSessionCreate,
//                                                     MuSessionId,
//                                                     sizeof(*MuSessionId));
//                     if (!NT_SUCCESS(Status))
//                     {
//                         DPRINT1("SMSS: Session space creation failed\n");
//                         SmpReleasePrivilege(State);
//                         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPath.Buffer);
//                         return Status;
//                     }
//                     AttachedSessionId = *MuSessionId;
//
//                     /*
//                      * Start Win32k.sys on this session. Use a hardcoded value
//                      * instead of the Kmode one...
//                      */
//                     RtlInitUnicodeString(&DestinationString,
//                                          L"\\SystemRoot\\System32\\win32k.sys");
//                     Status = NtSetSystemInformation(SystemExtendServiceTableInformation,
//                                                     &DestinationString,
//                                                     sizeof(DestinationString));
//                     RtlFreeHeap(RtlGetProcessHeap(), 0, NtPath.Buffer);
//                     SmpReleasePrivilege(State);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         DPRINT1("SMSS: Load of WIN32K failed.\n");
//                         return Status;
//                     }
//                 }
//             }
//         }
//
//         /* Next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Now parse the required subsystem list */
//     NextEntry = SmpSubSystemsToLoad.Flink;
//     while (NextEntry != &SmpSubSystemsToLoad)
//     {
//         /* Get each entry and check if it's the internal debug or not */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         DPRINT1("SMSS: Loading subsystem '%wZ' => command='%wZ'\n",
//                 &RegEntry->Name, &RegEntry->Value);
//         if (_wcsicmp(RegEntry->Name.Buffer, L"Debug") == 0)
//         {
//             /* Load the internal debug system */
//             Status = SmpExecuteCommand(&RegEntry->Value,
//                                        *MuSessionId,
//                                        ProcessId,
//                                        SMP_DEBUG_FLAG | SMP_SUBSYSTEM_FLAG);
//         }
//         else
//         {
//             /* Load the required subsystem */
//             Status = SmpExecuteCommand(&RegEntry->Value,
//                                        *MuSessionId,
//                                        ProcessId,
//                                        SMP_SUBSYSTEM_FLAG);
//         }
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("SMSS: Subsystem execute failed (%wZ)\n", &RegEntry->Value);
//             return Status;
//         }
//
//         /* Move to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Process the "Execute" list now */
//     NextEntry = SmpExecuteList.Blink;
//     if (NextEntry != &SmpExecuteList)
//     {
//         /* Get the custom initial command */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//
//         /* Write the initial command and wait for 5 seconds (why??!) */
//         *InitialCommand = RegEntry->Name;
//         Timeout.QuadPart = -50000000;
//         NtDelayExecution(FALSE, &Timeout);
//     }
//     else
//     {
//         /* Use the default Winlogon initial command */
//         RtlInitUnicodeString(InitialCommand, L"winlogon.exe");
//         InitialCommandBuffer[0] = UNICODE_NULL;
//
//         /* Check if there's a debugger for Winlogon */
//         Status2 = LdrQueryImageFileExecutionOptions(InitialCommand,
//                                                     L"Debugger",
//                                                     REG_SZ,
//                                                     InitialCommandBuffer,
//                                                     sizeof(InitialCommandBuffer) -
//                                                     InitialCommand->Length,
//                                                     NULL);
//         if ((NT_SUCCESS(Status2)) && (InitialCommandBuffer[0]))
//         {
//             /* Put the debugger string with the Winlogon string */
//             RtlStringCbCatW(InitialCommandBuffer, sizeof(InitialCommandBuffer), L" ");
//             RtlStringCbCatW(InitialCommandBuffer, sizeof(InitialCommandBuffer), InitialCommand->Buffer);
//             RtlInitUnicodeString(InitialCommand, InitialCommandBuffer);
//         }
//     }
//
//     /* Finally check if there was a custom initial command */
//     NextEntry = SmpExecuteList.Flink;
//     while (NextEntry != &SmpExecuteList)
//     {
//         /* Execute each one */
//         RegEntry = CONTAINING_RECORD(NextEntry, SMP_REGISTRY_VALUE, Entry);
//         SmpExecuteCommand(&RegEntry->Name, *MuSessionId, NULL, 0);
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
