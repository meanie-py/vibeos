use super::vdm_types::*;
use core::ffi::c_void;
use ntdll_sys::nt::*;

// CsrGetClientThread is a CSR macro (csrsrv.h: TEB->CsrClientThread), inlined
// in C — NOT a real symbol. Re-translate as an inline (same as proc.rs).
#[inline]
unsafe fn CsrGetClientThread() -> PCSR_THREAD {
    (*NtCurrentTeb()).CsrClientThread as PCSR_THREAD
}

// ---- file-scope preamble (translated) ----
pub static mut FirstVDM: BOOLEAN = TRUE as BOOLEAN;
pub static mut VDMConsoleListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut DosCriticalSection: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut WowCriticalSection: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };

unsafe fn BaseSrvCreateConsoleRecord() -> PVDM_CONSOLE_RECORD {
    let ConsoleRecord: PVDM_CONSOLE_RECORD = RtlAllocateHeap(
        BaseSrvHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<VDM_CONSOLE_RECORD>() as _,
    ) as PVDM_CONSOLE_RECORD;
    if ConsoleRecord.is_null() {
        return core::ptr::null_mut();
    }

    (*ConsoleRecord).ConsoleHandle = core::ptr::null_mut();
    (*ConsoleRecord).ProcessHandle = core::ptr::null_mut();
    (*ConsoleRecord).ClientEvent = core::ptr::null_mut();
    (*ConsoleRecord).ServerEvent = core::ptr::null_mut();
    (*ConsoleRecord).ReenterCount = 0;
    (*ConsoleRecord).CurrentDirs = core::ptr::null_mut();
    (*ConsoleRecord).CurDirsLength = 0;
    (*ConsoleRecord).SessionId = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead));

    ConsoleRecord
}

#[no_mangle]
pub unsafe extern "C" fn BaseSrvGetConsoleRecord(
    ConsoleHandle: HANDLE,
    Record: *mut PVDM_CONSOLE_RECORD,
) -> NTSTATUS {
    let mut CurrentRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();

    if ConsoleHandle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let mut i: PLIST_ENTRY = VDMConsoleListHead.Flink;
    while i != core::ptr::addr_of_mut!(VDMConsoleListHead) {
        CurrentRecord = (i as *mut u8)
            .offset(-(core::mem::offset_of!(_VDM_CONSOLE_RECORD, Entry) as isize))
            as PVDM_CONSOLE_RECORD;
        if (*CurrentRecord).ConsoleHandle == ConsoleHandle {
            break;
        }
        i = (*i).Flink;
    }

    if i == core::ptr::addr_of_mut!(VDMConsoleListHead) {
        CurrentRecord = core::ptr::null_mut();
    }

    *Record = CurrentRecord;
    if !CurrentRecord.is_null() {
        STATUS_SUCCESS
    } else {
        STATUS_NOT_FOUND
    }
}

unsafe fn BaseSrvDestroyConsoleRecord(ConsoleRecord: PVDM_CONSOLE_RECORD) {
    if (*ConsoleRecord).CurrentDirs != core::ptr::null_mut() {
        RtlFreeHeap(BaseSrvHeap, 0, (*ConsoleRecord).CurrentDirs as _);
        (*ConsoleRecord).CurrentDirs = core::ptr::null_mut();
        (*ConsoleRecord).CurDirsLength = 0;
    }

    if !(*ConsoleRecord).ProcessHandle.is_null() {
        NtClose((*ConsoleRecord).ProcessHandle);
    }

    if !(*ConsoleRecord).ServerEvent.is_null() {
        NtClose((*ConsoleRecord).ServerEvent);
    }

    RtlFreeHeap(BaseSrvHeap, 0, ConsoleRecord as _);
}

#[no_mangle]
pub unsafe extern "C" fn GetConsoleRecordBySessionId(
    TaskId: ULONG,
    Record: *mut PVDM_CONSOLE_RECORD,
) -> NTSTATUS {
    let mut i: PLIST_ENTRY;
    let mut CurrentRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();

    i = VDMConsoleListHead.Flink;
    while i != core::ptr::addr_of_mut!(VDMConsoleListHead) {
        CurrentRecord = (i as *mut u8)
            .offset(-(core::mem::offset_of!(_VDM_CONSOLE_RECORD, Entry) as isize))
            as PVDM_CONSOLE_RECORD;
        if (*CurrentRecord).SessionId == TaskId {
            break;
        }
        i = (*i).Flink;
    }

    if i == core::ptr::addr_of_mut!(VDMConsoleListHead) {
        CurrentRecord = core::ptr::null_mut();
    }

    *Record = CurrentRecord;
    if !CurrentRecord.is_null() {
        STATUS_SUCCESS
    } else {
        STATUS_NOT_FOUND
    }
}

unsafe fn GetNextDosSesId() -> ULONG {
    // Renamed from `SessionId` — that shadows the `SessionId` static imported
    // via `use ntdll_sys::nt::*`.
    let mut SessionIdLocal: ULONG;
    let mut i: PLIST_ENTRY;
    let mut CurrentRecord: PVDM_CONSOLE_RECORD;
    let mut Found: BOOLEAN;

    SessionIdLocal = 1;
    while SessionIdLocal != 0 {
        Found = 0u8;

        i = VDMConsoleListHead.Flink;
        while i != core::ptr::addr_of_mut!(VDMConsoleListHead) {
            CurrentRecord = (i as *mut u8).sub(core::mem::offset_of!(_VDM_CONSOLE_RECORD, Entry))
                as PVDM_CONSOLE_RECORD;
            if (*CurrentRecord).SessionId == SessionIdLocal {
                Found = 1u8;
            }
            i = (*i).Flink;
        }

        if Found == 0 {
            break;
        }

        SessionIdLocal += 1;
    }

    SessionIdLocal
}

unsafe fn BaseSrvIsVdmAllowed() -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut VdmAllowed: BOOLEAN = 1u8;
    let mut RootKey: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut MachineKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ValueBuffer: [UCHAR;
        core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + core::mem::size_of::<ULONG>()] =
        [0; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + core::mem::size_of::<ULONG>()];
    let ValueInfo: PKEY_VALUE_PARTIAL_INFORMATION =
        ValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    let mut ActualSize: ULONG = 0;

    /* Initialize the unicode strings */
    let machine_name: &[u16] = &[
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C, 0x004D,
        0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x0000,
    ];
    RtlInitUnicodeString(&mut MachineKeyName, machine_name.as_ptr());
    RtlInitUnicodeString(&mut KeyName, VDM_POLICY_KEY_NAME_BUF.as_ptr());
    RtlInitUnicodeString(&mut ValueName, VDM_DISALLOWED_VALUE_NAME_BUF.as_ptr());

    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    Attributes.RootDirectory = core::ptr::null_mut();
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.ObjectName = &mut MachineKeyName;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Open the local machine key */
    Status = NtOpenKey(&mut RootKey, KEY_READ, &mut Attributes);
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    Attributes.RootDirectory = RootKey;
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.ObjectName = &mut KeyName;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Open the policy key in the local machine hive, if it exists */
    if NT_SUCCESS(NtOpenKey(&mut KeyHandle, KEY_READ, &mut Attributes)) {
        if NT_SUCCESS(NtQueryValueKey(
            KeyHandle,
            &mut ValueName,
            KeyValuePartialInformation,
            ValueInfo as *mut core::ffi::c_void,
            core::mem::size_of_val(&ValueBuffer) as ULONG,
            &mut ActualSize,
        )) {
            if *((*ValueInfo).Data.as_ptr() as *const ULONG) != 0 {
                VdmAllowed = 0u8;
            }
        }

        NtClose(KeyHandle);
    }

    /* Close the local machine key */
    NtClose(RootKey);

    /* If it's disabled system-wide, there's no need to check the user key */
    if VdmAllowed == 0 {
        return 0u8;
    }

    /* Open the current user key of the client */
    if CsrImpersonateClient(core::ptr::null_mut()) == 0 {
        return VdmAllowed;
    }
    Status = RtlOpenCurrentUser(KEY_READ, &mut RootKey);
    CsrRevertToSelf();

    /* If that fails, return the system-wide setting */
    if !NT_SUCCESS(Status) {
        return VdmAllowed;
    }

    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    Attributes.RootDirectory = RootKey;
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.ObjectName = &mut KeyName;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Open the policy key in the current user hive, if it exists */
    if NT_SUCCESS(NtOpenKey(&mut KeyHandle, KEY_READ, &mut Attributes)) {
        if NT_SUCCESS(NtQueryValueKey(
            KeyHandle,
            &mut ValueName,
            KeyValuePartialInformation,
            ValueInfo as *mut core::ffi::c_void,
            core::mem::size_of_val(&ValueBuffer) as ULONG,
            &mut ActualSize,
        )) {
            if *((*ValueInfo).Data.as_ptr() as *const ULONG) != 0 {
                VdmAllowed = 0u8;
            }
        }

        NtClose(KeyHandle);
    }

    VdmAllowed
}

unsafe fn BaseSrvCreatePairWaitHandles(ServerEvent: PHANDLE, ClientEvent: PHANDLE) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = NtCreateEvent(
        ServerEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtDuplicateObject(
        -1isize as *mut core::ffi::c_void,
        *ServerEvent,
        (*(*CsrGetClientThread()).Process).ProcessHandle,
        ClientEvent,
        0,
        0,
        DUPLICATE_SAME_ATTRIBUTES | DUPLICATE_SAME_ACCESS,
    );

    if !NT_SUCCESS(Status) {
        NtClose(*ServerEvent);
    }
    Status
}

unsafe extern "system" fn BaseSrvDestroyPairWaitHandles(ServerEvent: HANDLE, ClientEvent: HANDLE) {
    if !ServerEvent.is_null() {
        NtClose(ServerEvent);
    }
    if !ClientEvent.is_null() {
        NtDuplicateObject(
            (*(*CsrGetClientThread()).Process).ProcessHandle,
            ClientEvent,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
    }
}

unsafe fn BaseSrvFreeVDMInfo(CommandInfo: PVDM_COMMAND_INFO) {
    if !(*CommandInfo).CmdLine.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).CmdLine as _);
    }
    if !(*CommandInfo).AppName.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).AppName as _);
    }
    if !(*CommandInfo).PifFile.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).PifFile as _);
    }
    if !(*CommandInfo).CurDirectory.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).CurDirectory as _);
    }
    if !(*CommandInfo).Env.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).Env as _);
    }
    if !(*CommandInfo).Desktop.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).Desktop as _);
    }
    if !(*CommandInfo).Title.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).Title as _);
    }
    if !(*CommandInfo).Reserved.is_null() {
        RtlFreeHeap(BaseSrvHeap, 0, (*CommandInfo).Reserved as _);
    }

    RtlFreeHeap(BaseSrvHeap, 0, CommandInfo as _);
}

#[no_mangle]
pub unsafe extern "C" fn BaseSrvCleanupVDMResources(CsrProcess: PCSR_PROCESS) {
    let ProcessId = HandleToUlong((*CsrProcess).ClientId.UniqueProcess);
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD;
    let mut DosRecord: PVDM_DOS_RECORD;
    let mut i: PLIST_ENTRY;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    i = VDMConsoleListHead.Flink;
    while i != core::ptr::addr_of_mut!(VDMConsoleListHead) {
        ConsoleRecord =
            (i as usize - core::mem::offset_of!(_VDM_CONSOLE_RECORD, Entry)) as PVDM_CONSOLE_RECORD;
        i = (*i).Flink;

        if (*ConsoleRecord).ProcessId == ProcessId {
            if !(*ConsoleRecord).ServerEvent.is_null() {
                NtClose((*ConsoleRecord).ServerEvent);
                (*ConsoleRecord).ServerEvent = core::ptr::null_mut();
            }

            while !IsListEmpty(core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead)) {
                DosRecord = ((*ConsoleRecord).DosListHead.Flink as usize
                    - core::mem::offset_of!(_VDM_DOS_RECORD, Entry))
                    as PVDM_DOS_RECORD;

                if !(*DosRecord).ServerEvent.is_null() {
                    NtSetEvent((*DosRecord).ServerEvent, core::ptr::null_mut());
                    NtClose((*DosRecord).ServerEvent);
                    (*DosRecord).ServerEvent = core::ptr::null_mut();
                }

                if !(*DosRecord).CommandInfo.is_null() {
                    BaseSrvFreeVDMInfo((*DosRecord).CommandInfo);
                }
                RemoveEntryList(core::ptr::addr_of_mut!((*DosRecord).Entry));
                RtlFreeHeap(BaseSrvHeap, 0, DosRecord as _);
            }

            RemoveEntryList(core::ptr::addr_of_mut!((*ConsoleRecord).Entry));
            BaseSrvDestroyConsoleRecord(ConsoleRecord);
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));
}

unsafe fn BaseSrvCopyCommand(
    CheckVdmRequest: PBASE_CHECK_VDM,
    DosRecord: PVDM_DOS_RECORD,
) -> BOOLEAN {
    let mut Success: BOOLEAN = 0u8;
    let CommandInfo: PVDM_COMMAND_INFO;

    CommandInfo = RtlAllocateHeap(
        BaseSrvHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<VDM_COMMAND_INFO>() as _,
    ) as PVDM_COMMAND_INFO;
    if CommandInfo.is_null() {
        return 0u8;
    }

    (*CommandInfo).TaskId = (*CheckVdmRequest).iTask;
    (*CommandInfo).ExitCode = (*DosRecord).ExitCode;
    (*CommandInfo).CodePage = (*CheckVdmRequest).CodePage;
    (*CommandInfo).StdIn = (*CheckVdmRequest).StdIn;
    (*CommandInfo).StdOut = (*CheckVdmRequest).StdOut;
    (*CommandInfo).StdErr = (*CheckVdmRequest).StdErr;

    'cleanup: {
        (*CommandInfo).CmdLine = RtlAllocateHeap(
            BaseSrvHeap,
            HEAP_ZERO_MEMORY,
            (*CheckVdmRequest).CmdLen as _,
        ) as _;
        if (*CommandInfo).CmdLine.is_null() {
            break 'cleanup;
        }
        core::ptr::copy(
            (*CheckVdmRequest).CmdLine as *const u8,
            (*CommandInfo).CmdLine as *mut u8,
            (*CheckVdmRequest).CmdLen as usize,
        );

        (*CommandInfo).AppName = RtlAllocateHeap(
            BaseSrvHeap,
            HEAP_ZERO_MEMORY,
            (*CheckVdmRequest).AppLen as _,
        ) as _;
        if (*CommandInfo).AppName.is_null() {
            break 'cleanup;
        }
        core::ptr::copy(
            (*CheckVdmRequest).AppName as *const u8,
            (*CommandInfo).AppName as *mut u8,
            (*CheckVdmRequest).AppLen as usize,
        );

        if (*CheckVdmRequest).PifLen != 0 {
            (*CommandInfo).PifFile = RtlAllocateHeap(
                BaseSrvHeap,
                HEAP_ZERO_MEMORY,
                (*CheckVdmRequest).PifLen as _,
            ) as _;
            if (*CommandInfo).PifFile.is_null() {
                break 'cleanup;
            }
            core::ptr::copy(
                (*CheckVdmRequest).PifFile as *const u8,
                (*CommandInfo).PifFile as *mut u8,
                (*CheckVdmRequest).PifLen as usize,
            );
        } else {
            (*CommandInfo).PifFile = core::ptr::null_mut();
        }

        if (*CheckVdmRequest).CurDirectoryLen != 0 {
            (*CommandInfo).CurDirectory = RtlAllocateHeap(
                BaseSrvHeap,
                HEAP_ZERO_MEMORY,
                (*CheckVdmRequest).CurDirectoryLen as _,
            ) as _;
            if (*CommandInfo).CurDirectory.is_null() {
                break 'cleanup;
            }
            core::ptr::copy(
                (*CheckVdmRequest).CurDirectory as *const u8,
                (*CommandInfo).CurDirectory as *mut u8,
                (*CheckVdmRequest).CurDirectoryLen as usize,
            );
        } else {
            (*CommandInfo).CurDirectory = core::ptr::null_mut();
        }

        (*CommandInfo).Env = RtlAllocateHeap(
            BaseSrvHeap,
            HEAP_ZERO_MEMORY,
            (*CheckVdmRequest).EnvLen as _,
        ) as _;
        if (*CommandInfo).Env.is_null() {
            break 'cleanup;
        }
        core::ptr::copy(
            (*CheckVdmRequest).Env as *const u8,
            (*CommandInfo).Env as *mut u8,
            (*CheckVdmRequest).EnvLen as usize,
        );

        (*CommandInfo).EnvLen = (*CheckVdmRequest).EnvLen;
        core::ptr::copy(
            (*CheckVdmRequest).StartupInfo as *const u8,
            core::ptr::addr_of_mut!((*CommandInfo).StartupInfo) as *mut u8,
            core::mem::size_of::<STARTUPINFOA>(),
        );

        if (*CheckVdmRequest).DesktopLen != 0 {
            (*CommandInfo).Desktop = RtlAllocateHeap(
                BaseSrvHeap,
                HEAP_ZERO_MEMORY,
                (*CheckVdmRequest).DesktopLen as _,
            ) as _;
            if (*CommandInfo).Desktop.is_null() {
                break 'cleanup;
            }
            core::ptr::copy(
                (*CheckVdmRequest).Desktop as *const u8,
                (*CommandInfo).Desktop as *mut u8,
                (*CheckVdmRequest).DesktopLen as usize,
            );
        } else {
            (*CommandInfo).Desktop = core::ptr::null_mut();
        }

        (*CommandInfo).DesktopLen = (*CheckVdmRequest).DesktopLen;

        if (*CheckVdmRequest).TitleLen != 0 {
            (*CommandInfo).Title = RtlAllocateHeap(
                BaseSrvHeap,
                HEAP_ZERO_MEMORY,
                (*CheckVdmRequest).TitleLen as _,
            ) as _;
            if (*CommandInfo).Title.is_null() {
                break 'cleanup;
            }
            core::ptr::copy(
                (*CheckVdmRequest).Title as *const u8,
                (*CommandInfo).Title as *mut u8,
                (*CheckVdmRequest).TitleLen as usize,
            );
        } else {
            (*CommandInfo).Title = core::ptr::null_mut();
        }

        (*CommandInfo).TitleLen = (*CheckVdmRequest).TitleLen;

        if (*CheckVdmRequest).ReservedLen != 0 {
            (*CommandInfo).Reserved = RtlAllocateHeap(
                BaseSrvHeap,
                HEAP_ZERO_MEMORY,
                (*CheckVdmRequest).ReservedLen as _,
            ) as _;
            if (*CommandInfo).Reserved.is_null() {
                break 'cleanup;
            }
            core::ptr::copy(
                (*CheckVdmRequest).Reserved as *const u8,
                (*CommandInfo).Reserved as *mut u8,
                (*CheckVdmRequest).ReservedLen as usize,
            );
        } else {
            (*CommandInfo).Reserved = core::ptr::null_mut();
        }

        (*CommandInfo).ReservedLen = (*CheckVdmRequest).ReservedLen;

        (*CommandInfo).CmdLen = (*CheckVdmRequest).CmdLen;
        (*CommandInfo).AppLen = (*CheckVdmRequest).AppLen;
        (*CommandInfo).PifLen = (*CheckVdmRequest).PifLen;
        (*CommandInfo).CurDirectoryLen = (*CheckVdmRequest).CurDirectoryLen;
        (*CommandInfo).VDMState = (*DosRecord).State;

        (*DosRecord).CommandInfo = CommandInfo;

        Success = 1u8;
    }

    if Success == 0u8 {
        BaseSrvFreeVDMInfo(CommandInfo);
    }

    Success
}

unsafe fn BaseSrvFillCommandInfo(
    CommandInfo: PVDM_COMMAND_INFO,
    Message: PBASE_GET_NEXT_VDM_COMMAND,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    (*Message).iTask = (*CommandInfo).TaskId;
    (*Message).StdIn = (*CommandInfo).StdIn;
    (*Message).StdOut = (*CommandInfo).StdOut;
    (*Message).StdErr = (*CommandInfo).StdErr;
    (*Message).CodePage = (*CommandInfo).CodePage;
    (*Message).dwCreationFlags = (*CommandInfo).CreationFlags;
    (*Message).ExitCode = (*CommandInfo).ExitCode;
    (*Message).CurrentDrive = (*CommandInfo).CurrentDrive;
    (*Message).VDMState = (*CommandInfo).VDMState;
    (*Message).fComingFromBat = (*CommandInfo).ComingFromBat as u32;

    if (*Message).CmdLen >= (*CommandInfo).CmdLen {
        core::ptr::copy(
            (*CommandInfo).CmdLine as *const u8,
            (*Message).CmdLine as *mut u8,
            (*CommandInfo).CmdLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).CmdLen = (*CommandInfo).CmdLen;

    if (*Message).AppLen >= (*CommandInfo).AppLen {
        core::ptr::copy(
            (*CommandInfo).AppName as *const u8,
            (*Message).AppName as *mut u8,
            (*CommandInfo).AppLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).AppLen = (*CommandInfo).AppLen;

    if (*Message).PifLen >= (*CommandInfo).PifLen {
        core::ptr::copy(
            (*CommandInfo).PifFile as *const u8,
            (*Message).PifFile as *mut u8,
            (*CommandInfo).PifLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).PifLen = (*CommandInfo).PifLen;

    if (*Message).CurDirectoryLen >= (*CommandInfo).CurDirectoryLen {
        core::ptr::copy(
            (*CommandInfo).CurDirectory as *const u8,
            (*Message).CurDirectory as *mut u8,
            (*CommandInfo).CurDirectoryLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).CurDirectoryLen = (*CommandInfo).CurDirectoryLen;

    if (*Message).EnvLen >= (*CommandInfo).EnvLen {
        core::ptr::copy(
            (*CommandInfo).Env as *const u8,
            (*Message).Env as *mut u8,
            (*CommandInfo).EnvLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).EnvLen = (*CommandInfo).EnvLen;

    core::ptr::copy(
        core::ptr::addr_of!((*CommandInfo).StartupInfo) as *const u8,
        (*Message).StartupInfo as *mut u8,
        core::mem::size_of::<STARTUPINFOA>() as _,
    );

    if (*Message).DesktopLen >= (*CommandInfo).DesktopLen {
        core::ptr::copy(
            (*CommandInfo).Desktop as *const u8,
            (*Message).Desktop as *mut u8,
            (*CommandInfo).DesktopLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).DesktopLen = (*CommandInfo).DesktopLen;

    if (*Message).TitleLen >= (*CommandInfo).TitleLen {
        core::ptr::copy(
            (*CommandInfo).Title as *const u8,
            (*Message).Title as *mut u8,
            (*CommandInfo).TitleLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).TitleLen = (*CommandInfo).TitleLen;

    if (*Message).ReservedLen >= (*CommandInfo).ReservedLen {
        core::ptr::copy(
            (*CommandInfo).Reserved as *const u8,
            (*Message).Reserved as *mut u8,
            (*CommandInfo).ReservedLen as usize,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }
    (*Message).ReservedLen = (*CommandInfo).ReservedLen;

    Status
}

#[no_mangle]
pub unsafe extern "C" fn BaseInitializeVDM() {
    InitializeListHead(core::ptr::addr_of_mut!(VDMConsoleListHead));

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(WowCriticalSection));
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvCheckVDM(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let CheckVdmRequest: PBASE_CHECK_VDM =
        core::ptr::addr_of_mut!((*(ApiMessage as PBASE_API_MESSAGE)).Data.CheckVDMRequest);
    let mut CriticalSection: PRTL_CRITICAL_SECTION = core::ptr::null_mut();
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();
    let mut DosRecord: PVDM_DOS_RECORD = core::ptr::null_mut();
    let mut NewConsoleRecord: BOOLEAN = 0u8;
    let mut NewDosRecord: BOOLEAN = 0u8;

    if BaseSrvIsVdmAllowed() == 0 {
        return STATUS_VDM_DISALLOWED;
    }

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*CheckVdmRequest).CmdLine) as *mut PVOID,
        (*CheckVdmRequest).CmdLen as _,
        core::mem::size_of_val(&*(*CheckVdmRequest).CmdLine) as _,
    ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*CheckVdmRequest).AppName) as *mut PVOID,
            (*CheckVdmRequest).AppLen as _,
            core::mem::size_of_val(&*(*CheckVdmRequest).AppName) as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*CheckVdmRequest).PifFile) as *mut PVOID,
            (*CheckVdmRequest).PifLen as _,
            core::mem::size_of_val(&*(*CheckVdmRequest).PifFile) as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*CheckVdmRequest).CurDirectory) as *mut PVOID,
            (*CheckVdmRequest).CurDirectoryLen as _,
            core::mem::size_of_val(&*(*CheckVdmRequest).CurDirectory) as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*CheckVdmRequest).Desktop) as *mut PVOID,
            (*CheckVdmRequest).DesktopLen as _,
            core::mem::size_of_val(&*(*CheckVdmRequest).Desktop) as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*CheckVdmRequest).Title) as *mut PVOID,
            (*CheckVdmRequest).TitleLen as _,
            core::mem::size_of_val(&*(*CheckVdmRequest).Title) as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*CheckVdmRequest).Reserved) as *mut PVOID,
            (*CheckVdmRequest).ReservedLen as _,
            core::mem::size_of_val(&*(*CheckVdmRequest).Reserved) as _,
        ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    CriticalSection = if (*CheckVdmRequest).BinaryType != BINARY_TYPE_SEPARATE_WOW {
        core::ptr::addr_of_mut!(DosCriticalSection)
    } else {
        core::ptr::addr_of_mut!(WowCriticalSection)
    };

    RtlEnterCriticalSection(CriticalSection);

    'cleanup: {
        if (*CheckVdmRequest).BinaryType != BINARY_TYPE_SEPARATE_WOW {
            Status = BaseSrvGetConsoleRecord(
                (*CheckVdmRequest).ConsoleHandle,
                core::ptr::addr_of_mut!(ConsoleRecord),
            );
            if !NT_SUCCESS(Status) {
                ConsoleRecord = BaseSrvCreateConsoleRecord();
                if ConsoleRecord.is_null() {
                    Status = STATUS_NO_MEMORY;
                    break 'cleanup;
                }

                (*ConsoleRecord).ConsoleHandle = (*CheckVdmRequest).ConsoleHandle;
                if (*ConsoleRecord).ConsoleHandle.is_null() {
                    (*ConsoleRecord).SessionId = GetNextDosSesId();
                } else {
                    (*ConsoleRecord).SessionId = 0;
                }

                NewConsoleRecord = 1u8;
            }

            if NewConsoleRecord == 0 {
                DosRecord = ((*ConsoleRecord).DosListHead.Flink as *mut u8)
                    .sub(core::mem::offset_of!(_VDM_DOS_RECORD, Entry))
                    as PVDM_DOS_RECORD;

                if (*DosRecord).State != VDM_READY {
                    DosRecord = RtlAllocateHeap(
                        BaseSrvHeap,
                        HEAP_ZERO_MEMORY,
                        core::mem::size_of::<VDM_DOS_RECORD>() as _,
                    ) as PVDM_DOS_RECORD;
                    if DosRecord.is_null() {
                        Status = STATUS_NO_MEMORY;
                        break 'cleanup;
                    }

                    NewDosRecord = 1u8;
                }
            } else {
                DosRecord = RtlAllocateHeap(
                    BaseSrvHeap,
                    HEAP_ZERO_MEMORY,
                    core::mem::size_of::<VDM_DOS_RECORD>() as _,
                ) as PVDM_DOS_RECORD;
                if DosRecord.is_null() {
                    Status = STATUS_NO_MEMORY;
                    break 'cleanup;
                }

                NewDosRecord = 1u8;
            }

            (*DosRecord).State = VDM_NOT_READY;
            (*DosRecord).ExitCode = 0;

            if BaseSrvCopyCommand(CheckVdmRequest, DosRecord) == 0 {
                Status = STATUS_NO_MEMORY;
                break 'cleanup;
            }

            if NewDosRecord != 0 {
                InsertHeadList(
                    core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead),
                    core::ptr::addr_of_mut!((*DosRecord).Entry),
                );
            }

            if NewConsoleRecord == 0 {
                Status = BaseSrvCreatePairWaitHandles(
                    core::ptr::addr_of_mut!((*DosRecord).ServerEvent),
                    core::ptr::addr_of_mut!((*DosRecord).ClientEvent),
                );
                if !NT_SUCCESS(Status) {
                    break 'cleanup;
                }

                (*CheckVdmRequest).WaitObjectForParent = (*DosRecord).ClientEvent;
            }

            if !(*ConsoleRecord).ServerEvent.is_null() {
                NtSetEvent((*ConsoleRecord).ServerEvent, core::ptr::null_mut());
            }

            if NewConsoleRecord != 0 {
                InsertTailList(
                    core::ptr::addr_of_mut!(VDMConsoleListHead),
                    core::ptr::addr_of_mut!((*ConsoleRecord).Entry),
                );
            }

            (*CheckVdmRequest).iTask = (*ConsoleRecord).SessionId;
            (*CheckVdmRequest).VDMState = if NewConsoleRecord != 0 {
                VDM_NOT_LOADED
            } else {
                VDM_READY
            };
            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    if !NT_SUCCESS(Status) {
        if NewDosRecord != 0 {
            BaseSrvDestroyPairWaitHandles((*DosRecord).ServerEvent, (*DosRecord).ClientEvent);

            RtlFreeHeap(BaseSrvHeap, 0, DosRecord as PVOID);
            DosRecord = core::ptr::null_mut();
        }

        if NewConsoleRecord != 0 {
            RtlFreeHeap(BaseSrvHeap, 0, ConsoleRecord as PVOID);
            ConsoleRecord = core::ptr::null_mut();
        }
    }

    RtlLeaveCriticalSection(CriticalSection);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvUpdateVDMEntry(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let UpdateVdmEntryRequest: PBASE_UPDATE_VDM_ENTRY = core::ptr::addr_of_mut!(
        (*(ApiMessage as PBASE_API_MESSAGE))
            .Data
            .UpdateVDMEntryRequest
    );
    let CriticalSection: PRTL_CRITICAL_SECTION;
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();
    let mut DosRecord: PVDM_DOS_RECORD;

    CriticalSection = if (*UpdateVdmEntryRequest).BinaryType != BINARY_TYPE_SEPARATE_WOW {
        core::ptr::addr_of_mut!(DosCriticalSection)
    } else {
        core::ptr::addr_of_mut!(WowCriticalSection)
    };

    RtlEnterCriticalSection(CriticalSection);

    'cleanup: {
        if (*UpdateVdmEntryRequest).BinaryType != BINARY_TYPE_SEPARATE_WOW {
            if (*UpdateVdmEntryRequest).iTask != 0 {
                Status = GetConsoleRecordBySessionId(
                    (*UpdateVdmEntryRequest).iTask,
                    core::ptr::addr_of_mut!(ConsoleRecord),
                );
            } else {
                Status = BaseSrvGetConsoleRecord(
                    (*UpdateVdmEntryRequest).ConsoleHandle,
                    core::ptr::addr_of_mut!(ConsoleRecord),
                );
            }

            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }

            DosRecord = ((*ConsoleRecord).DosListHead.Flink as *mut u8)
                .offset(-(core::mem::offset_of!(_VDM_DOS_RECORD, Entry) as isize))
                as PVDM_DOS_RECORD;

            match (*UpdateVdmEntryRequest).EntryIndex {
                VdmEntryUndo => {
                    NtClose((*DosRecord).ServerEvent);
                    (*DosRecord).ClientEvent = core::ptr::null_mut();
                    (*DosRecord).ServerEvent = core::ptr::null_mut();

                    if ((*UpdateVdmEntryRequest).VDMCreationState as ULONG
                        & (VDM_UNDO_PARTIAL | VDM_UNDO_FULL))
                        != 0
                    {
                        if !(*DosRecord).CommandInfo.is_null() {
                            BaseSrvFreeVDMInfo((*DosRecord).CommandInfo);
                        }
                        RemoveEntryList(core::ptr::addr_of_mut!((*DosRecord).Entry));
                        RtlFreeHeap(BaseSrvHeap, 0, DosRecord as _);

                        if (*ConsoleRecord).DosListHead.Flink
                            == core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead)
                        {
                            RemoveEntryList(core::ptr::addr_of_mut!((*ConsoleRecord).Entry));
                            BaseSrvDestroyConsoleRecord(ConsoleRecord);
                        }
                    }

                    Status = STATUS_SUCCESS;
                }

                VdmEntryUpdateProcess => {
                    Status = NtDuplicateObject(
                        (*(*CsrGetClientThread()).Process).ProcessHandle,
                        (*UpdateVdmEntryRequest).VDMProcessHandle,
                        NtCurrentProcess(),
                        core::ptr::addr_of_mut!((*ConsoleRecord).ProcessHandle),
                        0,
                        0,
                        DUPLICATE_SAME_ATTRIBUTES | DUPLICATE_SAME_ACCESS,
                    );
                    if !NT_SUCCESS(Status) {
                        break 'cleanup;
                    }

                    Status = BaseSrvCreatePairWaitHandles(
                        core::ptr::addr_of_mut!((*DosRecord).ServerEvent),
                        core::ptr::addr_of_mut!((*DosRecord).ClientEvent),
                    );
                    if !NT_SUCCESS(Status) {
                        break 'cleanup;
                    }

                    (*UpdateVdmEntryRequest).WaitObjectForParent = (*DosRecord).ClientEvent;
                }

                VdmEntryUpdateControlCHandler => {
                    Status = STATUS_NOT_IMPLEMENTED;
                }

                _ => {
                    Status = STATUS_INVALID_PARAMETER;
                }
            }
        } else {
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    RtlLeaveCriticalSection(CriticalSection);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvGetNextVDMCommand(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let GetNextVdmCommandRequest: PBASE_GET_NEXT_VDM_COMMAND = core::ptr::addr_of_mut!(
        (*(ApiMessage as PBASE_API_MESSAGE))
            .Data
            .GetNextVDMCommandRequest
    );
    let CriticalSection: PRTL_CRITICAL_SECTION;
    let mut i: PLIST_ENTRY;
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();
    let mut DosRecord: PVDM_DOS_RECORD;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).CmdLine) as *mut PVOID,
        (*GetNextVdmCommandRequest).CmdLen as _,
        core::mem::size_of::<core::ffi::c_char>() as _,
    ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).AppName) as *mut PVOID,
            (*GetNextVdmCommandRequest).AppLen as _,
            core::mem::size_of::<core::ffi::c_char>() as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).PifFile) as *mut PVOID,
            (*GetNextVdmCommandRequest).PifLen as _,
            core::mem::size_of::<core::ffi::c_char>() as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).CurDirectory) as *mut PVOID,
            (*GetNextVdmCommandRequest).CurDirectoryLen as _,
            core::mem::size_of::<core::ffi::c_char>() as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).Env) as *mut PVOID,
            (*GetNextVdmCommandRequest).EnvLen as _,
            core::mem::size_of::<core::ffi::c_char>() as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).Desktop) as *mut PVOID,
            (*GetNextVdmCommandRequest).DesktopLen as _,
            core::mem::size_of::<core::ffi::c_char>() as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).Title) as *mut PVOID,
            (*GetNextVdmCommandRequest).TitleLen as _,
            core::mem::size_of::<core::ffi::c_char>() as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).Reserved) as *mut PVOID,
            (*GetNextVdmCommandRequest).ReservedLen as _,
            core::mem::size_of::<core::ffi::c_char>() as _,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetNextVdmCommandRequest).StartupInfo) as *mut PVOID,
            1,
            core::mem::size_of::<STARTUPINFOA>() as _,
        ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    CriticalSection = if ((*GetNextVdmCommandRequest).VDMState as ULONG & VDM_FLAG_WOW) != 0 {
        core::ptr::addr_of_mut!(WowCriticalSection)
    } else {
        core::ptr::addr_of_mut!(DosCriticalSection)
    };

    RtlEnterCriticalSection(CriticalSection);

    'cleanup: {
        if ((*GetNextVdmCommandRequest).VDMState as ULONG & VDM_FLAG_WOW) != 0 {
            Status = STATUS_NOT_IMPLEMENTED;
            break 'cleanup;
        }

        if (*GetNextVdmCommandRequest).iTask != 0 {
            Status = GetConsoleRecordBySessionId(
                (*GetNextVdmCommandRequest).iTask,
                core::ptr::addr_of_mut!(ConsoleRecord),
            );
        } else {
            Status = BaseSrvGetConsoleRecord(
                (*GetNextVdmCommandRequest).ConsoleHandle,
                core::ptr::addr_of_mut!(ConsoleRecord),
            );
        }

        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        (*GetNextVdmCommandRequest).iTask = (*ConsoleRecord).SessionId;
        (*GetNextVdmCommandRequest).WaitObjectForVDM = core::ptr::null_mut();

        if ((*GetNextVdmCommandRequest).VDMState as ULONG & VDM_GET_FIRST_COMMAND) != 0 {
            if (*ConsoleRecord).DosListHead.Flink
                == core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead)
            {
                Status = STATUS_INVALID_PARAMETER;
                break 'cleanup;
            }

            DosRecord = ((*ConsoleRecord).DosListHead.Flink as *mut u8)
                .offset(-(core::mem::offset_of!(_VDM_DOS_RECORD, Entry) as isize))
                as PVDM_DOS_RECORD;

            if (*DosRecord).CommandInfo.is_null() {
                Status = STATUS_INVALID_PARAMETER;
                break 'cleanup;
            }

            if (*ConsoleRecord).ConsoleHandle.is_null() {
                (*ConsoleRecord).ConsoleHandle = (*GetNextVdmCommandRequest).ConsoleHandle;
            }

            Status = BaseSrvFillCommandInfo((*DosRecord).CommandInfo, GetNextVdmCommandRequest);
            break 'cleanup;
        }

        if ((*GetNextVdmCommandRequest).VDMState as ULONG
            & (VDM_FLAG_FIRST_TASK | VDM_FLAG_RETRY | VDM_FLAG_NESTED_TASK))
            == 0
        {
            i = (*ConsoleRecord).DosListHead.Flink;
            DosRecord = core::ptr::null_mut();
            while i != core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead) {
                DosRecord = (i as *mut u8)
                    .offset(-(core::mem::offset_of!(_VDM_DOS_RECORD, Entry) as isize))
                    as PVDM_DOS_RECORD;
                if (*DosRecord).State == VDM_NOT_READY {
                    break;
                }
                i = (*i).Flink;
            }

            if i == core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead) {
                Status = STATUS_INVALID_PARAMETER;
                break 'cleanup;
            }

            (*DosRecord).ExitCode = (*GetNextVdmCommandRequest).ExitCode;
            (*DosRecord).State = VDM_READY;

            NtSetEvent((*DosRecord).ServerEvent, core::ptr::null_mut());
            NtClose((*DosRecord).ServerEvent);
            (*DosRecord).ServerEvent = core::ptr::null_mut();
        }

        i = (*ConsoleRecord).DosListHead.Flink;
        DosRecord = core::ptr::null_mut();
        while i != core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead) {
            DosRecord = (i as *mut u8)
                .offset(-(core::mem::offset_of!(_VDM_DOS_RECORD, Entry) as isize))
                as PVDM_DOS_RECORD;
            if (*DosRecord).State == VDM_NOT_READY && !(*DosRecord).CommandInfo.is_null() {
                break;
            }
            i = (*i).Flink;
        }

        if i != core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead) {
            if ((*GetNextVdmCommandRequest).VDMState as ULONG & VDM_GET_ENVIRONMENT) != 0 {
                if (*GetNextVdmCommandRequest).EnvLen < (*(*DosRecord).CommandInfo).EnvLen {
                    (*GetNextVdmCommandRequest).EnvLen = (*(*DosRecord).CommandInfo).EnvLen;
                    Status = STATUS_BUFFER_OVERFLOW;
                    break 'cleanup;
                }

                core::ptr::copy(
                    (*(*DosRecord).CommandInfo).Env as *const u8,
                    (*GetNextVdmCommandRequest).Env as *mut u8,
                    (*(*DosRecord).CommandInfo).EnvLen as usize,
                );

                (*GetNextVdmCommandRequest).EnvLen = (*(*DosRecord).CommandInfo).EnvLen;
            } else {
                Status = BaseSrvFillCommandInfo((*DosRecord).CommandInfo, GetNextVdmCommandRequest);
                if !NT_SUCCESS(Status) {
                    break 'cleanup;
                }

                BaseSrvFreeVDMInfo((*DosRecord).CommandInfo);
                (*DosRecord).CommandInfo = core::ptr::null_mut();
                (*DosRecord).State = VDM_NOT_READY;
            }

            Status = STATUS_SUCCESS;
            break 'cleanup;
        }

        (*GetNextVdmCommandRequest).WaitObjectForVDM = core::ptr::null_mut();

        if ((*GetNextVdmCommandRequest).VDMState as ULONG & VDM_FLAG_DONT_WAIT) == 0
            || ((*GetNextVdmCommandRequest).VDMState as ULONG & VDM_FLAG_RETRY) == 0
        {
            if !(*ConsoleRecord).ServerEvent.is_null() {
                NtResetEvent((*ConsoleRecord).ServerEvent, core::ptr::null_mut());
            } else {
                Status = BaseSrvCreatePairWaitHandles(
                    core::ptr::addr_of_mut!((*ConsoleRecord).ServerEvent),
                    core::ptr::addr_of_mut!((*ConsoleRecord).ClientEvent),
                );
                if !NT_SUCCESS(Status) {
                    break 'cleanup;
                }
            }

            (*GetNextVdmCommandRequest).WaitObjectForVDM = (*ConsoleRecord).ClientEvent;
        }
    }

    RtlLeaveCriticalSection(CriticalSection);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvExitVDM(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let ExitVdmRequest: PBASE_EXIT_VDM =
        core::ptr::addr_of_mut!((*(ApiMessage as PBASE_API_MESSAGE)).Data.ExitVDMRequest);
    let CriticalSection: PRTL_CRITICAL_SECTION;
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();
    let mut DosRecord: PVDM_DOS_RECORD;

    CriticalSection = if (*ExitVdmRequest).iWowTask == 0 {
        core::ptr::addr_of_mut!(DosCriticalSection)
    } else {
        core::ptr::addr_of_mut!(WowCriticalSection)
    };

    RtlEnterCriticalSection(CriticalSection);

    'cleanup: {
        if (*ExitVdmRequest).iWowTask == 0 {
            Status = BaseSrvGetConsoleRecord(
                (*ExitVdmRequest).ConsoleHandle,
                core::ptr::addr_of_mut!(ConsoleRecord),
            );
            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }

            if !(*ConsoleRecord).ServerEvent.is_null() {
                (*ExitVdmRequest).WaitObjectForVDM = (*ConsoleRecord).ClientEvent;
            }

            if !(*ConsoleRecord).ServerEvent.is_null() {
                NtClose((*ConsoleRecord).ServerEvent);
                (*ConsoleRecord).ServerEvent = core::ptr::null_mut();
            }

            while !IsListEmpty(core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead)) {
                DosRecord = ((*ConsoleRecord).DosListHead.Flink as *mut u8)
                    .offset(-(core::mem::offset_of!(_VDM_DOS_RECORD, Entry) as isize))
                    as PVDM_DOS_RECORD;

                if !(*DosRecord).ServerEvent.is_null() {
                    NtSetEvent((*DosRecord).ServerEvent, core::ptr::null_mut());
                    NtClose((*DosRecord).ServerEvent);
                    (*DosRecord).ServerEvent = core::ptr::null_mut();
                }

                if !(*DosRecord).CommandInfo.is_null() {
                    BaseSrvFreeVDMInfo((*DosRecord).CommandInfo);
                }
                RemoveEntryList(core::ptr::addr_of_mut!((*DosRecord).Entry));
                RtlFreeHeap(BaseSrvHeap, 0, DosRecord as _);
            }

            RemoveEntryList(core::ptr::addr_of_mut!((*ConsoleRecord).Entry));
            BaseSrvDestroyConsoleRecord(ConsoleRecord);
        } else {
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    RtlLeaveCriticalSection(CriticalSection);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvIsFirstVDM(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let IsFirstVDMRequest: PBASE_IS_FIRST_VDM =
        core::ptr::addr_of_mut!((*(ApiMessage as PBASE_API_MESSAGE)).Data.IsFirstVDMRequest);

    (*IsFirstVDMRequest).FirstVDM = FirstVDM as u32;

    FirstVDM = 0u8;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvGetVDMExitCode(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let GetVDMExitCodeRequest: PBASE_GET_VDM_EXIT_CODE = core::ptr::addr_of_mut!(
        (*(ApiMessage as PBASE_API_MESSAGE))
            .Data
            .GetVDMExitCodeRequest
    );
    let mut i: PLIST_ENTRY;
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();
    let mut DosRecord: PVDM_DOS_RECORD = core::ptr::null_mut();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    'cleanup: {
        Status =
            BaseSrvGetConsoleRecord((*GetVDMExitCodeRequest).ConsoleHandle, &mut ConsoleRecord);
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        i = (*ConsoleRecord).DosListHead.Flink;
        while i != core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead) {
            DosRecord = (i as *mut u8)
                .offset(-(core::mem::offset_of!(_VDM_DOS_RECORD, Entry) as isize))
                as PVDM_DOS_RECORD;
            if (*DosRecord).ClientEvent == (*GetVDMExitCodeRequest).hParent {
                break;
            }
            i = (*i).Flink;
        }

        if i == core::ptr::addr_of_mut!((*ConsoleRecord).DosListHead) {
            Status = STATUS_NOT_FOUND;
            break 'cleanup;
        }

        if (*DosRecord).State != VDM_READY {
            (*GetVDMExitCodeRequest).ExitCode = STATUS_PENDING as _;
            break 'cleanup;
        }

        (*GetVDMExitCodeRequest).ExitCode = (*DosRecord).ExitCode;

        if !(*DosRecord).CommandInfo.is_null() {
            BaseSrvFreeVDMInfo((*DosRecord).CommandInfo);
        }
        RemoveEntryList(core::ptr::addr_of_mut!((*DosRecord).Entry));
        RtlFreeHeap(BaseSrvHeap, 0, DosRecord as _);
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvSetReenterCount(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let SetReenterCountRequest: PBASE_SET_REENTER_COUNT = core::ptr::addr_of_mut!(
        (*(ApiMessage as PBASE_API_MESSAGE))
            .Data
            .SetReenterCountRequest
    );
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    'cleanup: {
        Status = BaseSrvGetConsoleRecord(
            (*SetReenterCountRequest).ConsoleHandle,
            core::ptr::addr_of_mut!(ConsoleRecord),
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        if (*SetReenterCountRequest).fIncDec == VDM_INC_REENTER_COUNT {
            (*ConsoleRecord).ReenterCount += 1;
        } else if (*SetReenterCountRequest).fIncDec == VDM_DEC_REENTER_COUNT {
            (*ConsoleRecord).ReenterCount -= 1;
            if !(*ConsoleRecord).ServerEvent.is_null() {
                NtSetEvent((*ConsoleRecord).ServerEvent, core::ptr::null_mut());
            }
        } else {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvSetVDMCurDirs(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let VDMCurrentDirsRequest: PBASE_GETSET_VDM_CURDIRS = core::ptr::addr_of_mut!(
        (*(ApiMessage as PBASE_API_MESSAGE))
            .Data
            .VDMCurrentDirsRequest
    );
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();
    let mut Buffer: PCHAR;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*VDMCurrentDirsRequest).lpszzCurDirs) as *mut PVOID,
        (*VDMCurrentDirsRequest).cchCurDirs as _,
        core::mem::size_of::<CHAR>() as _,
    ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    'cleanup: {
        Status = BaseSrvGetConsoleRecord(
            (*VDMCurrentDirsRequest).ConsoleHandle,
            core::ptr::addr_of_mut!(ConsoleRecord),
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        if (*ConsoleRecord).CurrentDirs.is_null() {
            Buffer = RtlAllocateHeap(
                BaseSrvHeap,
                HEAP_ZERO_MEMORY,
                (*VDMCurrentDirsRequest).cchCurDirs as _,
            ) as PCHAR;
        } else {
            Buffer = RtlReAllocateHeap(
                BaseSrvHeap,
                HEAP_ZERO_MEMORY,
                (*ConsoleRecord).CurrentDirs as PVOID,
                (*VDMCurrentDirsRequest).cchCurDirs as _,
            ) as PCHAR;
        }

        if Buffer.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'cleanup;
        }

        (*ConsoleRecord).CurrentDirs = Buffer;
        (*ConsoleRecord).CurDirsLength = (*VDMCurrentDirsRequest).cchCurDirs;

        core::ptr::copy(
            (*VDMCurrentDirsRequest).lpszzCurDirs as *const u8,
            (*ConsoleRecord).CurrentDirs as *mut u8,
            (*VDMCurrentDirsRequest).cchCurDirs as usize,
        );
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvGetVDMCurDirs(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let VDMCurrentDirsRequest: PBASE_GETSET_VDM_CURDIRS = core::ptr::addr_of_mut!(
        (*(ApiMessage as PBASE_API_MESSAGE))
            .Data
            .VDMCurrentDirsRequest
    );
    let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*VDMCurrentDirsRequest).lpszzCurDirs) as *mut PVOID,
        (*VDMCurrentDirsRequest).cchCurDirs as _,
        core::mem::size_of::<core::ffi::c_char>() as _,
    ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    'cleanup: {
        Status = BaseSrvGetConsoleRecord(
            (*VDMCurrentDirsRequest).ConsoleHandle,
            core::ptr::addr_of_mut!(ConsoleRecord),
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        (*VDMCurrentDirsRequest).cchCurDirs = (*ConsoleRecord).CurDirsLength;

        if (*VDMCurrentDirsRequest).cchCurDirs < (*ConsoleRecord).CurDirsLength {
            Status = STATUS_BUFFER_TOO_SMALL;
            break 'cleanup;
        }

        core::ptr::copy(
            (*ConsoleRecord).CurrentDirs as *const u8,
            (*VDMCurrentDirsRequest).lpszzCurDirs as *mut u8,
            (*ConsoleRecord).CurDirsLength as usize,
        );
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(DosCriticalSection));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvBatNotification(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvRegisterWowExec(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvRefreshIniFileMapping(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Base API Server DLL
//  * FILE:            subsystems/win/basesrv/vdm.c
//  * PURPOSE:         Virtual DOS Machines (VDM) Support
//  * PROGRAMMERS:     Hermes Belusca-Maito (hermes.belusca@sfr.fr)
//  *                  Aleksandar Andrejevic <theflash AT sdf DOT lonestar DOT org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "basesrv.h"
// #include "vdm.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// BOOLEAN FirstVDM = TRUE;
// LIST_ENTRY VDMConsoleListHead;
// RTL_CRITICAL_SECTION DosCriticalSection;
// RTL_CRITICAL_SECTION WowCriticalSection;
//
// /* HELPER FUNCTIONS ***********************************************************/
//
// PVDM_CONSOLE_RECORD BaseSrvCreateConsoleRecord(VOID)
// {
//     PVDM_CONSOLE_RECORD ConsoleRecord;
//
//     ConsoleRecord = RtlAllocateHeap(BaseSrvHeap, HEAP_ZERO_MEMORY,
//                                     sizeof(VDM_CONSOLE_RECORD));
//     if (ConsoleRecord == NULL)
//         return NULL;
//
//     /* Initialize the console record */
//     ConsoleRecord->ConsoleHandle = NULL;
//     ConsoleRecord->ProcessHandle = NULL;
//     ConsoleRecord->ServerEvent = ConsoleRecord->ClientEvent = NULL;
//     ConsoleRecord->ReenterCount = 0;
//     ConsoleRecord->CurrentDirs = NULL;
//     ConsoleRecord->CurDirsLength = 0;
//     ConsoleRecord->SessionId = 0;
//     InitializeListHead(&ConsoleRecord->DosListHead);
//
//     return ConsoleRecord;
// }
//
// NTSTATUS BaseSrvGetConsoleRecord(HANDLE ConsoleHandle, PVDM_CONSOLE_RECORD *Record)
// {
//     PLIST_ENTRY i;
//     PVDM_CONSOLE_RECORD CurrentRecord = NULL;
//
//     /* NULL is not a valid console handle */
//     if (ConsoleHandle == NULL) return STATUS_INVALID_PARAMETER;
//
//     /* Search for a record that has the same console handle */
//     for (i = VDMConsoleListHead.Flink; i != &VDMConsoleListHead; i = i->Flink)
//     {
//         CurrentRecord = CONTAINING_RECORD(i, VDM_CONSOLE_RECORD, Entry);
//         if (CurrentRecord->ConsoleHandle == ConsoleHandle) break;
//     }
//
//     /* Check if nothing was found */
//     if (i == &VDMConsoleListHead) CurrentRecord = NULL;
//
//     *Record = CurrentRecord;
//     return CurrentRecord ? STATUS_SUCCESS : STATUS_NOT_FOUND;
// }
//
// VOID BaseSrvDestroyConsoleRecord(PVDM_CONSOLE_RECORD ConsoleRecord)
// {
//     if (ConsoleRecord->CurrentDirs != NULL)
//     {
//         /* Free the current directories */
//         RtlFreeHeap(BaseSrvHeap, 0, ConsoleRecord->CurrentDirs);
//         ConsoleRecord->CurrentDirs = NULL;
//         ConsoleRecord->CurDirsLength = 0;
//     }
//
//     /* Close the process handle */
//     if (ConsoleRecord->ProcessHandle)
//         NtClose(ConsoleRecord->ProcessHandle);
//
//     /* Close the event handle */
//     if (ConsoleRecord->ServerEvent)
//         NtClose(ConsoleRecord->ServerEvent);
//
//     /* Remove the console record */
//     // RemoveEntryList(&ConsoleRecord->Entry);
//     RtlFreeHeap(BaseSrvHeap, 0, ConsoleRecord);
// }
//
// NTSTATUS GetConsoleRecordBySessionId(ULONG TaskId, PVDM_CONSOLE_RECORD *Record)
// {
//     PLIST_ENTRY i;
//     PVDM_CONSOLE_RECORD CurrentRecord = NULL;
//
//     /* Search for a record that has the same console handle */
//     for (i = VDMConsoleListHead.Flink; i != &VDMConsoleListHead; i = i->Flink)
//     {
//         CurrentRecord = CONTAINING_RECORD(i, VDM_CONSOLE_RECORD, Entry);
//         if (CurrentRecord->SessionId == TaskId) break;
//     }
//
//     /* Check if nothing was found */
//     if (i == &VDMConsoleListHead) CurrentRecord = NULL;
//
//     *Record = CurrentRecord;
//     return CurrentRecord ? STATUS_SUCCESS : STATUS_NOT_FOUND;
// }
//
// ULONG GetNextDosSesId(VOID)
// {
//     ULONG SessionId;
//     PLIST_ENTRY i;
//     PVDM_CONSOLE_RECORD CurrentRecord = NULL;
//     BOOLEAN Found;
//
//     /* Search for an available session ID */
//     for (SessionId = 1; SessionId != 0; SessionId++)
//     {
//         Found = FALSE;
//
//         /* Check if the ID is already in use */
//         for (i = VDMConsoleListHead.Flink; i != &VDMConsoleListHead; i = i->Flink)
//         {
//             CurrentRecord = CONTAINING_RECORD(i, VDM_CONSOLE_RECORD, Entry);
//             if (CurrentRecord->SessionId == SessionId) Found = TRUE;
//         }
//
//         /* If not, we found one */
//         if (!Found) break;
//     }
//
//     ASSERT(SessionId != 0);
//
//     /* Return the session ID */
//     return SessionId;
// }
//
// BOOLEAN BaseSrvIsVdmAllowed(VOID)
// {
//     NTSTATUS Status;
//     BOOLEAN VdmAllowed = TRUE;
//     HANDLE RootKey, KeyHandle;
//     UNICODE_STRING KeyName, ValueName, MachineKeyName;
//     OBJECT_ATTRIBUTES Attributes;
//     UCHAR ValueBuffer[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + sizeof(ULONG)];
//     PKEY_VALUE_PARTIAL_INFORMATION ValueInfo = (PKEY_VALUE_PARTIAL_INFORMATION)ValueBuffer;
//     ULONG ActualSize;
//
//     /* Initialize the unicode strings */
//     RtlInitUnicodeString(&MachineKeyName, L"\\Registry\\Machine");
//     RtlInitUnicodeString(&KeyName, VDM_POLICY_KEY_NAME);
//     RtlInitUnicodeString(&ValueName, VDM_DISALLOWED_VALUE_NAME);
//
//     InitializeObjectAttributes(&Attributes,
//                                &MachineKeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open the local machine key */
//     Status = NtOpenKey(&RootKey, KEY_READ, &Attributes);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     InitializeObjectAttributes(&Attributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                RootKey,
//                                NULL);
//
//     /* Open the policy key in the local machine hive, if it exists */
//     if (NT_SUCCESS(NtOpenKey(&KeyHandle, KEY_READ, &Attributes)))
//     {
//         /* Read the value, if it's set */
//         if (NT_SUCCESS(NtQueryValueKey(KeyHandle,
//                                        &ValueName,
//                                        KeyValuePartialInformation,
//                                        ValueInfo,
//                                        sizeof(ValueBuffer),
//                                        &ActualSize)))
//         {
//             if (*((PULONG)ValueInfo->Data))
//             {
//                 /* The VDM has been disabled in the registry */
//                 VdmAllowed = FALSE;
//             }
//         }
//
//         NtClose(KeyHandle);
//     }
//
//     /* Close the local machine key */
//     NtClose(RootKey);
//
//     /* If it's disabled system-wide, there's no need to check the user key */
//     if (!VdmAllowed) return FALSE;
//
//     /* Open the current user key of the client */
//     if (!CsrImpersonateClient(NULL)) return VdmAllowed;
//     Status = RtlOpenCurrentUser(KEY_READ, &RootKey);
//     CsrRevertToSelf();
//
//     /* If that fails, return the system-wide setting */
//     if (!NT_SUCCESS(Status)) return VdmAllowed;
//
//     InitializeObjectAttributes(&Attributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                RootKey,
//                                NULL);
//
//     /* Open the policy key in the current user hive, if it exists */
//     if (NT_SUCCESS(NtOpenKey(&KeyHandle, KEY_READ, &Attributes)))
//     {
//         /* Read the value, if it's set */
//         if (NT_SUCCESS(NtQueryValueKey(KeyHandle,
//                                        &ValueName,
//                                        KeyValuePartialInformation,
//                                        ValueInfo,
//                                        sizeof(ValueBuffer),
//                                        &ActualSize)))
//         {
//             if (*((PULONG)ValueInfo->Data))
//             {
//                 /* The VDM has been disabled in the registry */
//                 VdmAllowed = FALSE;
//             }
//         }
//
//         NtClose(KeyHandle);
//     }
//
//     return VdmAllowed;
// }
//
// NTSTATUS BaseSrvCreatePairWaitHandles(PHANDLE ServerEvent, PHANDLE ClientEvent)
// {
//     NTSTATUS Status;
//
//     /* Create the event */
//     Status = NtCreateEvent(ServerEvent, EVENT_ALL_ACCESS, NULL, NotificationEvent, FALSE);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Duplicate the event into the client process */
//     Status = NtDuplicateObject(NtCurrentProcess(),
//                                *ServerEvent,
//                                CsrGetClientThread()->Process->ProcessHandle,
//                                ClientEvent,
//                                0,
//                                0,
//                                DUPLICATE_SAME_ATTRIBUTES | DUPLICATE_SAME_ACCESS);
//
//     if (!NT_SUCCESS(Status)) NtClose(*ServerEvent);
//     return Status;
// }
//
// VOID BaseSrvDestroyPairWaitHandles(HANDLE ServerEvent, HANDLE ClientEvent)
// {
//     if (ServerEvent) NtClose(ServerEvent);
//     if (ClientEvent)
//     {
//         /* Close the remote handle */
//         NtDuplicateObject(CsrGetClientThread()->Process->ProcessHandle,
//                           ClientEvent,
//                           NULL,
//                           NULL,
//                           0,
//                           0,
//                           DUPLICATE_CLOSE_SOURCE);
//     }
// }
//
// /* WOW SUPPORT FUNCTIONS ******************************************************/
//
// /* DOS SUPPORT FUNCTIONS ******************************************************/
//
// VOID BaseSrvFreeVDMInfo(PVDM_COMMAND_INFO CommandInfo)
// {
//     /* Free the allocated structure members */
//     if (CommandInfo->CmdLine != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->CmdLine);
//     if (CommandInfo->AppName != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->AppName);
//     if (CommandInfo->PifFile != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->PifFile);
//     if (CommandInfo->CurDirectory != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->CurDirectory);
//     if (CommandInfo->Env != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->Env);
//     if (CommandInfo->Desktop != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->Desktop);
//     if (CommandInfo->Title != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->Title);
//     if (CommandInfo->Reserved != NULL) RtlFreeHeap(BaseSrvHeap, 0, CommandInfo->Reserved);
//
//     /* Free the structure itself */
//     RtlFreeHeap(BaseSrvHeap, 0, CommandInfo);
// }
//
// VOID BaseSrvCleanupVDMResources(IN PCSR_PROCESS CsrProcess)
// {
//     ULONG ProcessId = HandleToUlong(CsrProcess->ClientId.UniqueProcess);
//     PVDM_CONSOLE_RECORD ConsoleRecord = NULL;
//     PVDM_DOS_RECORD DosRecord;
//     PLIST_ENTRY i;
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(&DosCriticalSection);
//
//     /* Search for a record that has the same process handle */
//     i = VDMConsoleListHead.Flink;
//     while (i != &VDMConsoleListHead)
//     {
//         ConsoleRecord = CONTAINING_RECORD(i, VDM_CONSOLE_RECORD, Entry);
//         i = i->Flink;
//
//         if (ConsoleRecord->ProcessId == ProcessId)
//         {
//             if (ConsoleRecord->ServerEvent)
//             {
//                 NtClose(ConsoleRecord->ServerEvent);
//                 ConsoleRecord->ServerEvent = NULL;
//             }
//
//             /* Cleanup the DOS records */
//             while (!IsListEmpty(&ConsoleRecord->DosListHead))
//             {
//                 DosRecord = CONTAINING_RECORD(ConsoleRecord->DosListHead.Flink,
//                                               VDM_DOS_RECORD, Entry);
//
//                 /* Set the event and close it */
//                 if (DosRecord->ServerEvent)
//                 {
//                     NtSetEvent(DosRecord->ServerEvent, NULL);
//                     NtClose(DosRecord->ServerEvent);
//                     DosRecord->ServerEvent = NULL;
//                 }
//
//                 /* Remove the DOS entry */
//                 if (DosRecord->CommandInfo) BaseSrvFreeVDMInfo(DosRecord->CommandInfo);
//                 RemoveEntryList(&DosRecord->Entry);
//                 RtlFreeHeap(BaseSrvHeap, 0, DosRecord);
//             }
//
//             /* Remove the console record */
//             RemoveEntryList(&ConsoleRecord->Entry);
//             BaseSrvDestroyConsoleRecord(ConsoleRecord);
//         }
//     }
//
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(&DosCriticalSection);
// }
//
// BOOLEAN BaseSrvCopyCommand(PBASE_CHECK_VDM CheckVdmRequest, PVDM_DOS_RECORD DosRecord)
// {
//     BOOLEAN Success = FALSE;
//     PVDM_COMMAND_INFO CommandInfo = NULL;
//
//     /* Allocate the command information structure */
//     CommandInfo = (PVDM_COMMAND_INFO)RtlAllocateHeap(BaseSrvHeap,
//                                                      HEAP_ZERO_MEMORY,
//                                                      sizeof(VDM_COMMAND_INFO));
//     if (CommandInfo == NULL) return FALSE;
//
//     /* Fill the structure */
//     CommandInfo->TaskId = CheckVdmRequest->iTask;
//     CommandInfo->ExitCode = DosRecord->ExitCode;
//     CommandInfo->CodePage = CheckVdmRequest->CodePage;
//     CommandInfo->StdIn = CheckVdmRequest->StdIn;
//     CommandInfo->StdOut = CheckVdmRequest->StdOut;
//     CommandInfo->StdErr = CheckVdmRequest->StdErr;
//
//     /* Allocate memory for the command line */
//     CommandInfo->CmdLine = RtlAllocateHeap(BaseSrvHeap,
//                                            HEAP_ZERO_MEMORY,
//                                            CheckVdmRequest->CmdLen);
//     if (CommandInfo->CmdLine == NULL) goto Cleanup;
//
//     /* Copy the command line */
//     RtlMoveMemory(CommandInfo->CmdLine, CheckVdmRequest->CmdLine, CheckVdmRequest->CmdLen);
//
//     /* Allocate memory for the application name */
//     CommandInfo->AppName = RtlAllocateHeap(BaseSrvHeap,
//                                            HEAP_ZERO_MEMORY,
//                                            CheckVdmRequest->AppLen);
//     if (CommandInfo->AppName == NULL) goto Cleanup;
//
//     /* Copy the application name */
//     RtlMoveMemory(CommandInfo->AppName, CheckVdmRequest->AppName, CheckVdmRequest->AppLen);
//
//     /* Allocate memory for the PIF file name */
//     if (CheckVdmRequest->PifLen != 0)
//     {
//         CommandInfo->PifFile = RtlAllocateHeap(BaseSrvHeap,
//                                                HEAP_ZERO_MEMORY,
//                                                CheckVdmRequest->PifLen);
//         if (CommandInfo->PifFile == NULL) goto Cleanup;
//
//         /* Copy the PIF file name */
//         RtlMoveMemory(CommandInfo->PifFile, CheckVdmRequest->PifFile, CheckVdmRequest->PifLen);
//     }
//     else CommandInfo->PifFile = NULL;
//
//     /* Allocate memory for the current directory */
//     if (CheckVdmRequest->CurDirectoryLen != 0)
//     {
//         CommandInfo->CurDirectory = RtlAllocateHeap(BaseSrvHeap,
//                                                     HEAP_ZERO_MEMORY,
//                                                     CheckVdmRequest->CurDirectoryLen);
//         if (CommandInfo->CurDirectory == NULL) goto Cleanup;
//
//         /* Copy the current directory */
//         RtlMoveMemory(CommandInfo->CurDirectory,
//                       CheckVdmRequest->CurDirectory,
//                       CheckVdmRequest->CurDirectoryLen);
//     }
//     else CommandInfo->CurDirectory = NULL;
//
//     /* Allocate memory for the environment block */
//     CommandInfo->Env = RtlAllocateHeap(BaseSrvHeap,
//                                        HEAP_ZERO_MEMORY,
//                                        CheckVdmRequest->EnvLen);
//     if (CommandInfo->Env == NULL) goto Cleanup;
//
//     /* Copy the environment block */
//     RtlMoveMemory(CommandInfo->Env, CheckVdmRequest->Env, CheckVdmRequest->EnvLen);
//
//     CommandInfo->EnvLen = CheckVdmRequest->EnvLen;
//     RtlMoveMemory(&CommandInfo->StartupInfo,
//                   CheckVdmRequest->StartupInfo,
//                   sizeof(STARTUPINFOA));
//
//     /* Allocate memory for the desktop */
//     if (CheckVdmRequest->DesktopLen != 0)
//     {
//         CommandInfo->Desktop = RtlAllocateHeap(BaseSrvHeap,
//                                                HEAP_ZERO_MEMORY,
//                                                CheckVdmRequest->DesktopLen);
//         if (CommandInfo->Desktop == NULL) goto Cleanup;
//
//         /* Copy the desktop name */
//         RtlMoveMemory(CommandInfo->Desktop, CheckVdmRequest->Desktop, CheckVdmRequest->DesktopLen);
//     }
//     else CommandInfo->Desktop = NULL;
//
//     CommandInfo->DesktopLen = CheckVdmRequest->DesktopLen;
//
//     /* Allocate memory for the title */
//     if (CheckVdmRequest->TitleLen != 0)
//     {
//         CommandInfo->Title = RtlAllocateHeap(BaseSrvHeap,
//                                              HEAP_ZERO_MEMORY,
//                                              CheckVdmRequest->TitleLen);
//         if (CommandInfo->Title == NULL) goto Cleanup;
//
//         /* Copy the title */
//         RtlMoveMemory(CommandInfo->Title, CheckVdmRequest->Title, CheckVdmRequest->TitleLen);
//     }
//     else CommandInfo->Title = NULL;
//
//     CommandInfo->TitleLen = CheckVdmRequest->TitleLen;
//
//     /* Allocate memory for the reserved field */
//     if (CheckVdmRequest->ReservedLen != 0)
//     {
//         CommandInfo->Reserved = RtlAllocateHeap(BaseSrvHeap,
//                                                 HEAP_ZERO_MEMORY,
//                                                 CheckVdmRequest->ReservedLen);
//         if (CommandInfo->Reserved == NULL) goto Cleanup;
//
//         /* Copy the reserved field */
//         RtlMoveMemory(CommandInfo->Reserved,
//                       CheckVdmRequest->Reserved,
//                       CheckVdmRequest->ReservedLen);
//     }
//     else CommandInfo->Reserved = NULL;
//
//     CommandInfo->ReservedLen = CheckVdmRequest->ReservedLen;
//
//     CommandInfo->CmdLen = CheckVdmRequest->CmdLen;
//     CommandInfo->AppLen = CheckVdmRequest->AppLen;
//     CommandInfo->PifLen = CheckVdmRequest->PifLen;
//     CommandInfo->CurDirectoryLen = CheckVdmRequest->CurDirectoryLen;
//     CommandInfo->VDMState = DosRecord->State;
//     // TODO: Set CommandInfo->CurrentDrive
//     // TODO: Set CommandInfo->ComingFromBat
//
//     /* Set the DOS record's command structure */
//     DosRecord->CommandInfo = CommandInfo;
//
//     /* The operation was successful */
//     Success = TRUE;
//
// Cleanup:
//     /* If it wasn't successful, free the memory */
//     if (!Success) BaseSrvFreeVDMInfo(CommandInfo);
//
//     return Success;
// }
//
// NTSTATUS BaseSrvFillCommandInfo(PVDM_COMMAND_INFO CommandInfo,
//                                 PBASE_GET_NEXT_VDM_COMMAND Message)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Copy the data */
//     Message->iTask = CommandInfo->TaskId;
//     Message->StdIn = CommandInfo->StdIn;
//     Message->StdOut = CommandInfo->StdOut;
//     Message->StdErr = CommandInfo->StdErr;
//     Message->CodePage = CommandInfo->CodePage;
//     Message->dwCreationFlags = CommandInfo->CreationFlags;
//     Message->ExitCode = CommandInfo->ExitCode;
//     Message->CurrentDrive = CommandInfo->CurrentDrive;
//     Message->VDMState = CommandInfo->VDMState;
//     Message->fComingFromBat = CommandInfo->ComingFromBat;
//
//     if (Message->CmdLen >= CommandInfo->CmdLen)
//     {
//         /* Copy the command line */
//         RtlMoveMemory(Message->CmdLine, CommandInfo->CmdLine, CommandInfo->CmdLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->CmdLen = CommandInfo->CmdLen;
//
//     if (Message->AppLen >= CommandInfo->AppLen)
//     {
//         /* Copy the application name */
//         RtlMoveMemory(Message->AppName, CommandInfo->AppName, CommandInfo->AppLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->AppLen = CommandInfo->AppLen;
//
//     if (Message->PifLen >= CommandInfo->PifLen)
//     {
//         /* Copy the PIF file name */
//         RtlMoveMemory(Message->PifFile, CommandInfo->PifFile, CommandInfo->PifLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->PifLen = CommandInfo->PifLen;
//
//     if (Message->CurDirectoryLen >= CommandInfo->CurDirectoryLen)
//     {
//         /* Copy the current directory */
//         RtlMoveMemory(Message->CurDirectory, CommandInfo->CurDirectory, CommandInfo->CurDirectoryLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->CurDirectoryLen = CommandInfo->CurDirectoryLen;
//
//     if (Message->EnvLen >= CommandInfo->EnvLen)
//     {
//         /* Copy the environment */
//         RtlMoveMemory(Message->Env, CommandInfo->Env, CommandInfo->EnvLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->EnvLen = CommandInfo->EnvLen;
//
//     /* Copy the startup info */
//     RtlMoveMemory(Message->StartupInfo,
//                   &CommandInfo->StartupInfo,
//                   sizeof(STARTUPINFOA));
//
//     if (Message->DesktopLen >= CommandInfo->DesktopLen)
//     {
//         /* Copy the desktop name */
//         RtlMoveMemory(Message->Desktop, CommandInfo->Desktop, CommandInfo->DesktopLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->DesktopLen = CommandInfo->DesktopLen;
//
//     if (Message->TitleLen >= CommandInfo->TitleLen)
//     {
//         /* Copy the title */
//         RtlMoveMemory(Message->Title, CommandInfo->Title, CommandInfo->TitleLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->TitleLen = CommandInfo->TitleLen;
//
//     if (Message->ReservedLen >= CommandInfo->ReservedLen)
//     {
//         /* Copy the reserved parameter */
//         RtlMoveMemory(Message->Reserved, CommandInfo->Reserved, CommandInfo->ReservedLen);
//     }
//     else Status = STATUS_INVALID_PARAMETER;
//     Message->ReservedLen = CommandInfo->ReservedLen;
//
//     return Status;
// }
//
// VOID BaseInitializeVDM(VOID)
// {
//     /* Initialize the list head */
//     InitializeListHead(&VDMConsoleListHead);
//
//     /* Initialize the critical sections */
//     RtlInitializeCriticalSection(&DosCriticalSection);
//     RtlInitializeCriticalSection(&WowCriticalSection);
// }
//
// /* PUBLIC SERVER APIS *********************************************************/
//
// NTSTATUS NTAPI BaseSrvCheckVDM(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status;
//     PBASE_CHECK_VDM CheckVdmRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.CheckVDMRequest;
//     PRTL_CRITICAL_SECTION CriticalSection = NULL;
//     PVDM_CONSOLE_RECORD ConsoleRecord = NULL;
//     PVDM_DOS_RECORD DosRecord = NULL;
//     BOOLEAN NewConsoleRecord = FALSE;
//     BOOLEAN NewDosRecord = FALSE;
//
//     /* Don't do anything if the VDM has been disabled in the registry */
//     if (!BaseSrvIsVdmAllowed()) return STATUS_VDM_DISALLOWED;
//
//     /* Validate the message buffers */
//     if (!CsrValidateMessageBuffer(ApiMessage,
//                                   (PVOID*)&CheckVdmRequest->CmdLine,
//                                   CheckVdmRequest->CmdLen,
//                                   sizeof(*CheckVdmRequest->CmdLine))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&CheckVdmRequest->AppName,
//                                      CheckVdmRequest->AppLen,
//                                      sizeof(*CheckVdmRequest->AppName))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&CheckVdmRequest->PifFile,
//                                      CheckVdmRequest->PifLen,
//                                      sizeof(*CheckVdmRequest->PifFile))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&CheckVdmRequest->CurDirectory,
//                                      CheckVdmRequest->CurDirectoryLen,
//                                      sizeof(*CheckVdmRequest->CurDirectory))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&CheckVdmRequest->Desktop,
//                                      CheckVdmRequest->DesktopLen,
//                                      sizeof(*CheckVdmRequest->Desktop))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&CheckVdmRequest->Title,
//                                      CheckVdmRequest->TitleLen,
//                                      sizeof(*CheckVdmRequest->Title))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&CheckVdmRequest->Reserved,
//                                      CheckVdmRequest->ReservedLen,
//                                      sizeof(*CheckVdmRequest->Reserved)))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     CriticalSection = (CheckVdmRequest->BinaryType != BINARY_TYPE_SEPARATE_WOW)
//                       ? &DosCriticalSection
//                       : &WowCriticalSection;
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(CriticalSection);
//
//     /* Check if this is a DOS or WOW VDM */
//     if (CheckVdmRequest->BinaryType != BINARY_TYPE_SEPARATE_WOW)
//     {
//         /* Get the console record */
//         Status = BaseSrvGetConsoleRecord(CheckVdmRequest->ConsoleHandle,
//                                          &ConsoleRecord);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Allocate a new console record */
//             ConsoleRecord = BaseSrvCreateConsoleRecord();
//             if (ConsoleRecord == NULL)
//             {
//                 Status = STATUS_NO_MEMORY;
//                 goto Cleanup;
//             }
//
//             /* Initialize the console record */
//             ConsoleRecord->ConsoleHandle = CheckVdmRequest->ConsoleHandle;
//             if (ConsoleRecord->ConsoleHandle == NULL)
//             {
//                 /* The parent doesn't have a console, get a new session ID */
//                 ConsoleRecord->SessionId = GetNextDosSesId();
//             }
//             else
//             {
//                 /* No session ID is needed */
//                 ConsoleRecord->SessionId = 0;
//             }
//
//             /* Remember that the console record was allocated here */
//             NewConsoleRecord = TRUE;
//         }
//
//         if (!NewConsoleRecord)
//         {
//             /* Get the primary DOS record */
//             DosRecord = (PVDM_DOS_RECORD)CONTAINING_RECORD(ConsoleRecord->DosListHead.Flink,
//                                                            VDM_DOS_RECORD, Entry);
//
//             if (DosRecord->State != VDM_READY) // == VDM_NOT_READY
//             {
//                 /* Allocate a new DOS record */
//                 DosRecord = (PVDM_DOS_RECORD)RtlAllocateHeap(BaseSrvHeap,
//                                                              HEAP_ZERO_MEMORY,
//                                                              sizeof(VDM_DOS_RECORD));
//                 if (DosRecord == NULL)
//                 {
//                     Status = STATUS_NO_MEMORY;
//                     goto Cleanup;
//                 }
//
//                 /* Remember that the DOS record was allocated here */
//                 NewDosRecord = TRUE;
//             }
//         }
//         else
//         {
//             /* Allocate a new DOS record */
//             DosRecord = (PVDM_DOS_RECORD)RtlAllocateHeap(BaseSrvHeap,
//                                                          HEAP_ZERO_MEMORY,
//                                                          sizeof(VDM_DOS_RECORD));
//             if (DosRecord == NULL)
//             {
//                 Status = STATUS_NO_MEMORY;
//                 goto Cleanup;
//             }
//
//             /* Remember that the DOS record was allocated here */
//             NewDosRecord = TRUE;
//         }
//
//         /* Initialize the DOS record */
//         DosRecord->State = VDM_NOT_READY;
//         DosRecord->ExitCode = 0;
//
//         /* Translate the input structure into a VDM command structure and set it in the DOS record */
//         if (!BaseSrvCopyCommand(CheckVdmRequest, DosRecord))
//         {
//             /* The only possibility is that an allocation failure occurred */
//             Status = STATUS_NO_MEMORY;
//             goto Cleanup;
//         }
//
//         if (NewDosRecord)
//         {
//             /* Add the DOS record */
//             InsertHeadList(&ConsoleRecord->DosListHead, &DosRecord->Entry);
//         }
//
//         if (!NewConsoleRecord)
//         {
//             Status = BaseSrvCreatePairWaitHandles(&DosRecord->ServerEvent, &DosRecord->ClientEvent);
//             if (!NT_SUCCESS(Status)) goto Cleanup;
//
//             /* Return the client event handle */
//             CheckVdmRequest->WaitObjectForParent = DosRecord->ClientEvent;
//         }
//
//         // FIXME: We may notify ONLY if ConsoleRecord->nReEntrancy is > 0
//         // in case NewConsoleRecord == FALSE AND NewDosRecord == TRUE.
//         if (ConsoleRecord->ServerEvent)
//         {
//             /* Signal the session event */
//             NtSetEvent(ConsoleRecord->ServerEvent, NULL);
//         }
//
//         if (NewConsoleRecord)
//         {
//             /* Add the console record */
//             InsertTailList(&VDMConsoleListHead, &ConsoleRecord->Entry);
//         }
//
//         CheckVdmRequest->iTask = ConsoleRecord->SessionId;
//         CheckVdmRequest->VDMState = NewConsoleRecord ? VDM_NOT_LOADED : VDM_READY;
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         // TODO: NOT IMPLEMENTED
//         UNIMPLEMENTED;
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
//
// Cleanup:
//     /* Check if it failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free the DOS record if it was allocated here */
//         if (NewDosRecord)
//         {
//             ASSERT(DosRecord != NULL);
//
//             BaseSrvDestroyPairWaitHandles(DosRecord->ServerEvent,
//                                           DosRecord->ClientEvent);
//
//             RtlFreeHeap(BaseSrvHeap, 0, DosRecord);
//             DosRecord = NULL;
//         }
//
//         /* Free the console record if it was allocated here */
//         if (NewConsoleRecord)
//         {
//             ASSERT(ConsoleRecord != NULL);
//
//             RtlFreeHeap(BaseSrvHeap, 0, ConsoleRecord);
//             ConsoleRecord = NULL;
//         }
//     }
//
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(CriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvUpdateVDMEntry(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status;
//     PBASE_UPDATE_VDM_ENTRY UpdateVdmEntryRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.UpdateVDMEntryRequest;
//     PRTL_CRITICAL_SECTION CriticalSection = NULL;
//     PVDM_CONSOLE_RECORD ConsoleRecord = NULL;
//     PVDM_DOS_RECORD DosRecord = NULL;
//
//     CriticalSection = (UpdateVdmEntryRequest->BinaryType != BINARY_TYPE_SEPARATE_WOW)
//                       ? &DosCriticalSection
//                       : &WowCriticalSection;
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(CriticalSection);
//
//     /* Check if this is a DOS or WOW VDM */
//     if (UpdateVdmEntryRequest->BinaryType != BINARY_TYPE_SEPARATE_WOW)
//     {
//         if (UpdateVdmEntryRequest->iTask != 0)
//         {
//             /* Get the console record using the task ID */
//             Status = GetConsoleRecordBySessionId(UpdateVdmEntryRequest->iTask,
//                                                  &ConsoleRecord);
//         }
//         else
//         {
//             /* Get the console record using the console handle */
//             Status = BaseSrvGetConsoleRecord(UpdateVdmEntryRequest->ConsoleHandle,
//                                              &ConsoleRecord);
//         }
//
//         if (!NT_SUCCESS(Status)) goto Cleanup;
//
//         /* Get the primary DOS record */
//         DosRecord = (PVDM_DOS_RECORD)CONTAINING_RECORD(ConsoleRecord->DosListHead.Flink,
//                                                        VDM_DOS_RECORD, Entry);
//
//         switch (UpdateVdmEntryRequest->EntryIndex)
//         {
//             case VdmEntryUndo:
//             {
//                 /* Close the server event handle, the client will close the client handle */
//                 NtClose(DosRecord->ServerEvent);
//                 DosRecord->ServerEvent = DosRecord->ClientEvent = NULL;
//
//                 if (UpdateVdmEntryRequest->VDMCreationState & (VDM_UNDO_PARTIAL | VDM_UNDO_FULL))
//                 {
//                     /* Remove the DOS record */
//                     if (DosRecord->CommandInfo) BaseSrvFreeVDMInfo(DosRecord->CommandInfo);
//                     RemoveEntryList(&DosRecord->Entry);
//                     RtlFreeHeap(BaseSrvHeap, 0, DosRecord);
//
//                     /*
//                      * Since this is an undo, if that was the only DOS record the VDM
//                      * won't even start, so the console record should be removed too.
//                      */
//                     if (ConsoleRecord->DosListHead.Flink == &ConsoleRecord->DosListHead)
//                     {
//                         RemoveEntryList(&ConsoleRecord->Entry);
//                         BaseSrvDestroyConsoleRecord(ConsoleRecord);
//                     }
//                 }
//
//                 /* It was successful */
//                 Status = STATUS_SUCCESS;
//
//                 break;
//             }
//
//             case VdmEntryUpdateProcess:
//             {
//                 /* Duplicate the VDM process handle */
//                 Status = NtDuplicateObject(CsrGetClientThread()->Process->ProcessHandle,
//                                            UpdateVdmEntryRequest->VDMProcessHandle,
//                                            NtCurrentProcess(),
//                                            &ConsoleRecord->ProcessHandle,
//                                            0,
//                                            0,
//                                            DUPLICATE_SAME_ATTRIBUTES | DUPLICATE_SAME_ACCESS);
//                 if (!NT_SUCCESS(Status)) goto Cleanup;
//
//                 //
//                 // FIXME! Should we always do the following??
//                 //
//
//                 /* Create a pair of handles to one event object */
//                 Status = BaseSrvCreatePairWaitHandles(&DosRecord->ServerEvent,
//                                                       &DosRecord->ClientEvent);
//                 if (!NT_SUCCESS(Status)) goto Cleanup;
//
//                 /* Return the client event handle */
//                 UpdateVdmEntryRequest->WaitObjectForParent = DosRecord->ClientEvent;
//
//                 break;
//             }
//
//             case VdmEntryUpdateControlCHandler:
//             {
//                 // TODO: NOT IMPLEMENTED
//                 DPRINT1("BaseSrvUpdateVDMEntry: VdmEntryUpdateControlCHandler not implemented\n");
//                 Status = STATUS_NOT_IMPLEMENTED;
//
//                 break;
//             }
//
//             default:
//             {
//                 /* Invalid */
//                 Status = STATUS_INVALID_PARAMETER;
//             }
//         }
//     }
//     else
//     {
//         // TODO: NOT IMPLEMENTED
//         UNIMPLEMENTED;
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
//
// Cleanup:
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(CriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvGetNextVDMCommand(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status;
//     PBASE_GET_NEXT_VDM_COMMAND GetNextVdmCommandRequest =
//         &((PBASE_API_MESSAGE)ApiMessage)->Data.GetNextVDMCommandRequest;
//     PRTL_CRITICAL_SECTION CriticalSection;
//     PLIST_ENTRY i = NULL;
//     PVDM_CONSOLE_RECORD ConsoleRecord = NULL;
//     PVDM_DOS_RECORD DosRecord = NULL;
//
//     /* Validate the message buffers */
//     if (!CsrValidateMessageBuffer(ApiMessage,
//                                   (PVOID*)&GetNextVdmCommandRequest->CmdLine,
//                                   GetNextVdmCommandRequest->CmdLen,
//                                   sizeof(*GetNextVdmCommandRequest->CmdLine))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->AppName,
//                                      GetNextVdmCommandRequest->AppLen,
//                                      sizeof(*GetNextVdmCommandRequest->AppName))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->PifFile,
//                                      GetNextVdmCommandRequest->PifLen,
//                                      sizeof(*GetNextVdmCommandRequest->PifFile))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->CurDirectory,
//                                      GetNextVdmCommandRequest->CurDirectoryLen,
//                                      sizeof(*GetNextVdmCommandRequest->CurDirectory))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->Env,
//                                      GetNextVdmCommandRequest->EnvLen,
//                                      sizeof(*GetNextVdmCommandRequest->Env))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->Desktop,
//                                      GetNextVdmCommandRequest->DesktopLen,
//                                      sizeof(*GetNextVdmCommandRequest->Desktop))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->Title,
//                                      GetNextVdmCommandRequest->TitleLen,
//                                      sizeof(*GetNextVdmCommandRequest->Title))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->Reserved,
//                                      GetNextVdmCommandRequest->ReservedLen,
//                                      sizeof(*GetNextVdmCommandRequest->Reserved))
//         || !CsrValidateMessageBuffer(ApiMessage,
//                                      (PVOID*)&GetNextVdmCommandRequest->StartupInfo,
//                                      1,
//                                      sizeof(STARTUPINFOA)))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     CriticalSection = (GetNextVdmCommandRequest->VDMState & VDM_FLAG_WOW)
//                       ? &WowCriticalSection
//                       : &DosCriticalSection;
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(CriticalSection);
//
//     if (GetNextVdmCommandRequest->VDMState & VDM_FLAG_WOW)
//     {
//         // TODO: WOW SUPPORT NOT IMPLEMENTED
//         UNIMPLEMENTED;
//         Status = STATUS_NOT_IMPLEMENTED;
//         goto Cleanup;
//     }
//     // else if (!(GetNextVdmCommandRequest->VDMState & VDM_FLAG_WOW))
//     {
//         if (GetNextVdmCommandRequest->iTask != 0)
//         {
//             /* Get the console record using the task ID */
//             Status = GetConsoleRecordBySessionId(GetNextVdmCommandRequest->iTask,
//                                                  &ConsoleRecord);
//         }
//         else
//         {
//             /* Get the console record using the console handle */
//             Status = BaseSrvGetConsoleRecord(GetNextVdmCommandRequest->ConsoleHandle,
//                                              &ConsoleRecord);
//         }
//
//         /* Make sure we found the console record */
//         if (!NT_SUCCESS(Status)) goto Cleanup;
//
//         /* Return the session ID */
//         GetNextVdmCommandRequest->iTask = ConsoleRecord->SessionId;
//         GetNextVdmCommandRequest->WaitObjectForVDM = NULL;
//
//         if (GetNextVdmCommandRequest->VDMState & VDM_GET_FIRST_COMMAND)
//         {
//             /* Check if the DOS record list is empty */
//             if (ConsoleRecord->DosListHead.Flink == &ConsoleRecord->DosListHead)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 goto Cleanup;
//             }
//
//             /* Get the first DOS record */
//             DosRecord = CONTAINING_RECORD(ConsoleRecord->DosListHead.Flink, VDM_DOS_RECORD, Entry);
//
//             /* Make sure its command information is still there */
//             if (DosRecord->CommandInfo == NULL)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 goto Cleanup;
//             }
//
//             /* Check if the console handle hasn't been set yet */
//             if (ConsoleRecord->ConsoleHandle == NULL)
//             {
//                 /* Set it now */
//                 ConsoleRecord->ConsoleHandle = GetNextVdmCommandRequest->ConsoleHandle;
//             }
//
//             /* Fill the command information */
//             Status = BaseSrvFillCommandInfo(DosRecord->CommandInfo, GetNextVdmCommandRequest);
//             goto Cleanup;
//         }
//
//         /* Check if we should set the state of a running DOS record to ready */
//         if (!(GetNextVdmCommandRequest->VDMState
//             & (VDM_FLAG_FIRST_TASK | VDM_FLAG_RETRY | VDM_FLAG_NESTED_TASK)))
//         {
//             /* Search for a DOS record that is currently running */
//             for (i = ConsoleRecord->DosListHead.Flink; i != &ConsoleRecord->DosListHead; i = i->Flink)
//             {
//                 DosRecord = CONTAINING_RECORD(i, VDM_DOS_RECORD, Entry);
//                 if (DosRecord->State == VDM_NOT_READY) break;
//             }
//
//             /* Check if we found any */
//             if (i == &ConsoleRecord->DosListHead)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 goto Cleanup;
//             }
//
//             /* Set the exit code */
//             DosRecord->ExitCode = GetNextVdmCommandRequest->ExitCode;
//
//             /* Update the VDM state */
//             DosRecord->State = VDM_READY;
//
//             /* Notify all waiting threads that the task is finished */
//             NtSetEvent(DosRecord->ServerEvent, NULL);
//             NtClose(DosRecord->ServerEvent);
//             DosRecord->ServerEvent = NULL;
//         }
//
//         /* Search for a DOS record that is currently running and has command information */
//         for (i = ConsoleRecord->DosListHead.Flink; i != &ConsoleRecord->DosListHead; i = i->Flink)
//         {
//             DosRecord = CONTAINING_RECORD(i, VDM_DOS_RECORD, Entry);
//             if ((DosRecord->State == VDM_NOT_READY) && (DosRecord->CommandInfo != NULL)) break;
//         }
//
//         /* Check if we found any */
//         if (i != &ConsoleRecord->DosListHead)
//         {
//             ASSERT(DosRecord->CommandInfo != NULL);
//
//             /* Check if the caller only wants environment data */
//             if (GetNextVdmCommandRequest->VDMState & VDM_GET_ENVIRONMENT)
//             {
//                 if (GetNextVdmCommandRequest->EnvLen < DosRecord->CommandInfo->EnvLen)
//                 {
//                     /* Not enough space was reserved */
//                     GetNextVdmCommandRequest->EnvLen = DosRecord->CommandInfo->EnvLen;
//                     Status = STATUS_BUFFER_OVERFLOW;
//                     goto Cleanup;
//                 }
//
//                 /* Copy the environment data */
//                 RtlMoveMemory(GetNextVdmCommandRequest->Env,
//                               DosRecord->CommandInfo->Env,
//                               DosRecord->CommandInfo->EnvLen);
//
//                 /* Return the actual size to the caller */
//                 GetNextVdmCommandRequest->EnvLen = DosRecord->CommandInfo->EnvLen;
//             }
//             else
//             {
//                 /* Fill the command information */
//                 Status = BaseSrvFillCommandInfo(DosRecord->CommandInfo, GetNextVdmCommandRequest);
//                 if (!NT_SUCCESS(Status)) goto Cleanup;
//
//                 /* Free the command information, it's no longer needed */
//                 BaseSrvFreeVDMInfo(DosRecord->CommandInfo);
//                 DosRecord->CommandInfo = NULL;
//
//                 /* Update the VDM state */
//                 DosRecord->State = VDM_NOT_READY;
//             }
//
//             Status = STATUS_SUCCESS;
//             goto Cleanup;
//         }
//     }
//
//     GetNextVdmCommandRequest->WaitObjectForVDM = NULL;
//
//     /*
//      * There is no command yet. Prepare for waiting if we asked so,
//      * and if we were not retrying a request.
//      */
//     if (!(GetNextVdmCommandRequest->VDMState & VDM_FLAG_DONT_WAIT) ||
//         !(GetNextVdmCommandRequest->VDMState & VDM_FLAG_RETRY))
//     {
//         if (ConsoleRecord->ServerEvent)
//         {
//             /* Reset the event */
//             NtResetEvent(ConsoleRecord->ServerEvent, NULL);
//         }
//         else
//         {
//             /* Create a pair of wait handles */
//             Status = BaseSrvCreatePairWaitHandles(&ConsoleRecord->ServerEvent,
//                                                   &ConsoleRecord->ClientEvent);
//             if (!NT_SUCCESS(Status)) goto Cleanup;
//         }
//
//         /* Return the client event handle */
//         GetNextVdmCommandRequest->WaitObjectForVDM = ConsoleRecord->ClientEvent;
//     }
//
// Cleanup:
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(CriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvExitVDM(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status;
//     PBASE_EXIT_VDM ExitVdmRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.ExitVDMRequest;
//     PRTL_CRITICAL_SECTION CriticalSection = NULL;
//     PVDM_CONSOLE_RECORD ConsoleRecord = NULL;
//     PVDM_DOS_RECORD DosRecord;
//
//     CriticalSection = (ExitVdmRequest->iWowTask == 0)
//                       ? &DosCriticalSection
//                       : &WowCriticalSection;
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(CriticalSection);
//
//     if (ExitVdmRequest->iWowTask == 0)
//     {
//         /* Get the console record */
//         Status = BaseSrvGetConsoleRecord(ExitVdmRequest->ConsoleHandle, &ConsoleRecord);
//         if (!NT_SUCCESS(Status)) goto Cleanup;
//
//         if (ConsoleRecord->ServerEvent)
//             ExitVdmRequest->WaitObjectForVDM = ConsoleRecord->ClientEvent;
//
//         // NOTE: The following is the same as in BaseSrvCleanupVDMResources.
//
//         if (ConsoleRecord->ServerEvent)
//         {
//             NtClose(ConsoleRecord->ServerEvent);
//             ConsoleRecord->ServerEvent = NULL;
//         }
//
//         /* Cleanup the DOS records */
//         while (!IsListEmpty(&ConsoleRecord->DosListHead))
//         {
//             DosRecord = CONTAINING_RECORD(ConsoleRecord->DosListHead.Flink,
//                                           VDM_DOS_RECORD, Entry);
//
//             /* Set the event and close it */
//             if (DosRecord->ServerEvent)
//             {
//                 NtSetEvent(DosRecord->ServerEvent, NULL);
//                 NtClose(DosRecord->ServerEvent);
//                 DosRecord->ServerEvent = NULL;
//             }
//
//             /* Remove the DOS entry */
//             if (DosRecord->CommandInfo) BaseSrvFreeVDMInfo(DosRecord->CommandInfo);
//             RemoveEntryList(&DosRecord->Entry);
//             RtlFreeHeap(BaseSrvHeap, 0, DosRecord);
//         }
//
//         /* Remove the console record */
//         RemoveEntryList(&ConsoleRecord->Entry);
//         BaseSrvDestroyConsoleRecord(ConsoleRecord);
//     }
//     else
//     {
//         // TODO: NOT IMPLEMENTED
//         UNIMPLEMENTED;
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
//
// Cleanup:
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(CriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvIsFirstVDM(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     PBASE_IS_FIRST_VDM IsFirstVDMRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.IsFirstVDMRequest;
//
//     /* Return the result */
//     IsFirstVDMRequest->FirstVDM = FirstVDM;
//
//     /* Clear the first VDM flag */
//     FirstVDM = FALSE;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI BaseSrvGetVDMExitCode(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status;
//     PBASE_GET_VDM_EXIT_CODE GetVDMExitCodeRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.GetVDMExitCodeRequest;
//     PLIST_ENTRY i = NULL;
//     PVDM_CONSOLE_RECORD ConsoleRecord = NULL;
//     PVDM_DOS_RECORD DosRecord = NULL;
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(&DosCriticalSection);
//
//     /* Get the console record */
//     Status = BaseSrvGetConsoleRecord(GetVDMExitCodeRequest->ConsoleHandle, &ConsoleRecord);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Search for a DOS record that has the same parent process handle */
//     for (i = ConsoleRecord->DosListHead.Flink; i != &ConsoleRecord->DosListHead; i = i->Flink)
//     {
//         DosRecord = CONTAINING_RECORD(i, VDM_DOS_RECORD, Entry);
//         if (DosRecord->ClientEvent == GetVDMExitCodeRequest->hParent) break;
//     }
//
//     /* Check if no DOS record was found */
//     if (i == &ConsoleRecord->DosListHead)
//     {
//         Status = STATUS_NOT_FOUND;
//         goto Cleanup;
//     }
//
//     /* Check if this task is still running */
//     if (DosRecord->State != VDM_READY)
//     {
//         GetVDMExitCodeRequest->ExitCode = STATUS_PENDING;
//         goto Cleanup;
//     }
//
//     /* Return the exit code */
//     GetVDMExitCodeRequest->ExitCode = DosRecord->ExitCode;
//
//     // FIXME: We may just change DosRecord->State to VDM_READY in some cases...
//
//     /* Since this is a zombie task record, remove it */
//     if (DosRecord->CommandInfo) BaseSrvFreeVDMInfo(DosRecord->CommandInfo);
//     RemoveEntryList(&DosRecord->Entry);
//     RtlFreeHeap(BaseSrvHeap, 0, DosRecord);
//
// Cleanup:
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(&DosCriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvSetReenterCount(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PBASE_SET_REENTER_COUNT SetReenterCountRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.SetReenterCountRequest;
//     PVDM_CONSOLE_RECORD ConsoleRecord;
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(&DosCriticalSection);
//
//     /* Get the console record */
//     Status = BaseSrvGetConsoleRecord(SetReenterCountRequest->ConsoleHandle, &ConsoleRecord);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     if (SetReenterCountRequest->fIncDec == VDM_INC_REENTER_COUNT)
//     {
//         ConsoleRecord->ReenterCount++;
//     }
//     else if (SetReenterCountRequest->fIncDec == VDM_DEC_REENTER_COUNT)
//     {
//         ConsoleRecord->ReenterCount--;
//         if (ConsoleRecord->ServerEvent)
//             NtSetEvent(ConsoleRecord->ServerEvent, NULL);
//     }
//     else
//     {
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
// Cleanup:
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(&DosCriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvSetVDMCurDirs(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status;
//     PBASE_GETSET_VDM_CURDIRS VDMCurrentDirsRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.VDMCurrentDirsRequest;
//     PVDM_CONSOLE_RECORD ConsoleRecord;
//     PCHAR Buffer = NULL;
//
//     /* Validate the input buffer */
//     if (!CsrValidateMessageBuffer(ApiMessage,
//                                   (PVOID*)&VDMCurrentDirsRequest->lpszzCurDirs,
//                                   VDMCurrentDirsRequest->cchCurDirs,
//                                   sizeof(*VDMCurrentDirsRequest->lpszzCurDirs)))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(&DosCriticalSection);
//
//     /* Find the console record */
//     Status = BaseSrvGetConsoleRecord(VDMCurrentDirsRequest->ConsoleHandle, &ConsoleRecord);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     if (ConsoleRecord->CurrentDirs == NULL)
//     {
//         /* Allocate memory for the current directory information */
//         Buffer = RtlAllocateHeap(BaseSrvHeap,
//                                  HEAP_ZERO_MEMORY,
//                                  VDMCurrentDirsRequest->cchCurDirs);
//     }
//     else
//     {
//         /* Resize the amount of allocated memory */
//         Buffer = RtlReAllocateHeap(BaseSrvHeap,
//                                    HEAP_ZERO_MEMORY,
//                                    ConsoleRecord->CurrentDirs,
//                                    VDMCurrentDirsRequest->cchCurDirs);
//     }
//
//     if (Buffer == NULL)
//     {
//         /* Allocation failed */
//         Status = STATUS_NO_MEMORY;
//         goto Cleanup;
//     }
//
//     /* Update the console record */
//     ConsoleRecord->CurrentDirs = Buffer;
//     ConsoleRecord->CurDirsLength = VDMCurrentDirsRequest->cchCurDirs;
//
//     /* Copy the data */
//     RtlMoveMemory(ConsoleRecord->CurrentDirs,
//                   VDMCurrentDirsRequest->lpszzCurDirs,
//                   VDMCurrentDirsRequest->cchCurDirs);
//
// Cleanup:
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(&DosCriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvGetVDMCurDirs(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     NTSTATUS Status;
//     PBASE_GETSET_VDM_CURDIRS VDMCurrentDirsRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.VDMCurrentDirsRequest;
//     PVDM_CONSOLE_RECORD ConsoleRecord;
//
//     /* Validate the output buffer */
//     if (!CsrValidateMessageBuffer(ApiMessage,
//                                   (PVOID*)&VDMCurrentDirsRequest->lpszzCurDirs,
//                                   VDMCurrentDirsRequest->cchCurDirs,
//                                   sizeof(*VDMCurrentDirsRequest->lpszzCurDirs)))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Enter the critical section */
//     RtlEnterCriticalSection(&DosCriticalSection);
//
//     /* Find the console record */
//     Status = BaseSrvGetConsoleRecord(VDMCurrentDirsRequest->ConsoleHandle, &ConsoleRecord);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Return the actual size of the current directory information */
//     VDMCurrentDirsRequest->cchCurDirs = ConsoleRecord->CurDirsLength;
//
//     /* Check if the buffer is large enough */
//     if (VDMCurrentDirsRequest->cchCurDirs < ConsoleRecord->CurDirsLength)
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Cleanup;
//     }
//
//     /* Copy the data */
//     RtlMoveMemory(VDMCurrentDirsRequest->lpszzCurDirs,
//                   ConsoleRecord->CurrentDirs,
//                   ConsoleRecord->CurDirsLength);
//
// Cleanup:
//     /* Leave the critical section */
//     RtlLeaveCriticalSection(&DosCriticalSection);
//
//     return Status;
// }
//
// NTSTATUS NTAPI BaseSrvBatNotification(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS NTAPI BaseSrvRegisterWowExec(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS NTAPI BaseSrvRefreshIniFileMapping(PCSR_API_MESSAGE ApiMessage, PCSR_REPLY_CODE ReplyCode)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
