use crate::eventlog::eventsource::GetEventSourceByName;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0C7319B93B0B: [u16; 12] = [
    0x0041u16, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_12A6BF85117F: [u16; 9] = [0x006Eu16, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_3D7301573997: [u16; 15] = [
    0x005Cu16, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C, 0x0045, 0x0076, 0x0065, 0x006E,
    0x0074, 0x004C, 0x006F, 0x0067, 0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut LogHandleListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut LogHandleListCs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

// #define NDEBUG (no value; no const emitted)

// static NTSTATUS ElfDeleteEventLogHandle(PIELF_HANDLE LogHandle);
// (function prototype; the function is emitted with its body elsewhere)

#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(lpParameter: LPVOID) -> DWORD {
    let mut Status: RPC_STATUS = 0;

    InitializeCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
    InitializeListHead(core::ptr::addr_of_mut!(LogHandleListHead));

    'quit: {
        Status = RpcServerUseProtseqEpW(RUSTCONV_WSTR_12A6BF85117F.as_ptr() as RPC_WSTR, 20, RUSTCONV_WSTR_3D7301573997.as_ptr() as RPC_WSTR, core::ptr::null_mut());
        if Status != RPC_S_OK {
            DPRINT!(b"RpcServerUseProtseqEpW() failed (Status %lx)\n\0".as_ptr() as *const i8, Status);
            break 'quit;
        }

        Status = RpcServerRegisterIf(eventlog_v0_0_s_ifspec, core::ptr::null_mut(), core::ptr::null_mut());
        if Status != RPC_S_OK {
            DPRINT!(b"RpcServerRegisterIf() failed (Status %lx)\n\0".as_ptr() as *const i8, Status);
            break 'quit;
        }

        Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, FALSE);
        if Status != RPC_S_OK {
            DPRINT!(b"RpcServerListen() failed (Status %lx)\n\0".as_ptr() as *const i8, Status);
        }

        EnterCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
        while !IsListEmpty(core::ptr::addr_of_mut!(LogHandleListHead)) {
            let mut LogHandle: IELF_HANDLE = CONTAINING_RECORD!(LogHandleListHead.Flink, LOGHANDLE, LogHandleListEntry) as IELF_HANDLE;
            ElfDeleteEventLogHandle(core::ptr::addr_of_mut!(LogHandle));
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
    }

    DeleteCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));

    return 0;
}

unsafe extern "C" fn ElfCreateEventLogHandle(
    LogHandle: *mut PLOGHANDLE,
    LogName: PUNICODE_STRING,
    Create: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut pLogHandle: PLOGHANDLE = core::ptr::null_mut();
    let mut currentLogFile: PLOGFILE = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut LogsActive: DWORD = 0;
    let mut pEventSource: PEVENTSOURCE = core::ptr::null_mut();

    *LogHandle = core::ptr::null_mut();

    i = (((*LogName).Length as usize + core::mem::size_of_val(&UNICODE_NULL))
        / core::mem::size_of::<WCHAR>()) as DWORD;
    pLogHandle = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::offset_of!(LOGHANDLE, szName)
            + (i as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PLOGHANDLE;
    if pLogHandle.is_null() {
        if DbgPrint(
            b"(%s:%d) Failed to allocate Heap!\n\0" as *const u8 as PCSTR,
            b"base/services/eventlog/rpc.c\0"
                as *const u8 as *const core::ffi::c_char,
            89 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/rpc.c\0"
                    as *const u8 as *const core::ffi::c_char,
                89 as core::ffi::c_int,
            );
        }
        return STATUS_NO_MEMORY;
    }

    StringCchCopyW(
        core::ptr::addr_of_mut!((*pLogHandle).szName) as STRSAFE_LPWSTR,
        i as size_t,
        (*LogName).Buffer as STRSAFE_LPCWSTR,
    );

    /* Get the number of Log Files the EventLog service found */
    // NOTE: We could just as well loop only once within the list of logs
    // and retrieve what the code below that calls LogfListItemByIndex, does!!
    LogsActive = LogfListItemCount();
    'done: {
        if LogsActive == 0 {
            if DbgPrint(
                b"(%s:%d) EventLog service reports no log files!\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/rpc.c\0"
                    as *const u8 as *const core::ffi::c_char,
                101 as core::ffi::c_int,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"base/services/eventlog/rpc.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    101 as core::ffi::c_int,
                );
            }
            Status = STATUS_UNSUCCESSFUL;
            break 'done;
        }

        /* If Creating, default to the Application Log in case we fail, as documented on MSDN */
        if Create != 0 {
            pEventSource = GetEventSourceByName((*LogName).Buffer);
            if !pEventSource.is_null() {
                (*pLogHandle).LogFile = (*pEventSource).LogFile;
            } else {
                (*pLogHandle).LogFile = LogfListItemByName(RUSTCONV_WSTR_0C7319B93B0B.as_ptr());
            }
        } else {
            (*pLogHandle).LogFile = core::ptr::null_mut();

            i = 1;
            while i <= LogsActive {
                currentLogFile = LogfListItemByIndex(i);

                if _wcsicmp((*LogName).Buffer, (*currentLogFile).LogName)
                    == 0
                {
                    (*pLogHandle).LogFile = currentLogFile;
                    break;
                }
                i = i.wrapping_add(1);
            }

            /* Use the application log if the desired log does not exist */
            if (*pLogHandle).LogFile.is_null() {
                (*pLogHandle).LogFile = LogfListItemByName(RUSTCONV_WSTR_0C7319B93B0B.as_ptr());
                if (*pLogHandle).LogFile.is_null() {
                    if DbgPrint(
                        b"(%s:%d) Application log is missing!\n\0" as *const u8 as PCSTR,
                        b"base/services/eventlog/rpc.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        145 as core::ffi::c_int,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                            b"base/services/eventlog/rpc.c\0"
                                as *const u8 as *const core::ffi::c_char,
                            145 as core::ffi::c_int,
                        );
                    }
                    Status = STATUS_UNSUCCESSFUL;
                    break 'done;
                }
            }

            /* Reset the current record */
            (*pLogHandle).CurrentRecord = 0;
        }

        if (*pLogHandle).LogFile.is_null() {
            Status = STATUS_UNSUCCESSFUL;
        }
    }

    /* Done: */
    if NT_SUCCESS!(Status) {
        /* Append log handle */
        EnterCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
        InsertTailList(
            core::ptr::addr_of_mut!(LogHandleListHead),
            core::ptr::addr_of_mut!((*pLogHandle).LogHandleListEntry),
        );
        LeaveCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
        *LogHandle = pLogHandle;
    } else {
        HeapFree(
            GetProcessHeap(),
            0,
            pLogHandle as PVOID,
        );
    }

    return Status;
}

unsafe extern "C" fn ElfCreateBackupLogHandle(
    LogHandle: *mut PLOGHANDLE,
    FileName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut pLogHandle: PLOGHANDLE;

    *LogHandle = core::ptr::null_mut();

    pLogHandle = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<LOGHANDLE>() as SIZE_T,
    ) as PLOGHANDLE;
    if pLogHandle.is_null() {
        if DbgPrint(
            b"(%s:%d) Failed to allocate Heap!\n\0" as *const u8 as PCSTR,
            b"base/services/eventlog/rpc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            192 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"base/services/eventlog/rpc.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                192 as ::core::ffi::c_int,
            );
        }
        return STATUS_NO_MEMORY;
    }

    /* Create the log file */
    Status = LogfCreate(
        &raw mut (*pLogHandle).LogFile,
        core::ptr::null::<WCHAR>(),
        FileName,
        0,
        0,
        FALSE as BOOLEAN,
        TRUE as BOOLEAN,
    );
    'Done: {
        if !NT_SUCCESS!(Status) {
            if DbgPrint(
                b"(%s:%d) Failed to create the log file! (Status 0x%08lx)\n\0" as *const u8
                    as PCSTR,
                b"base/services/eventlog/rpc.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                206 as ::core::ffi::c_int,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"base/services/eventlog/rpc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    206 as ::core::ffi::c_int,
                );
            }
            break 'Done;
        }

        /* Set the backup flag */
        (*pLogHandle).Flags |= LOG_HANDLE_BACKUP_FILE as ULONG;

        /* Reset the current record */
        (*pLogHandle).CurrentRecord = 0 as ULONG;
    }

    if NT_SUCCESS!(Status) {
        /* Append log handle */
        EnterCriticalSection(&raw mut LogHandleListCs);
        InsertTailList(
            &raw mut LogHandleListHead,
            &raw mut (*pLogHandle).LogHandleListEntry,
        );
        LeaveCriticalSection(&raw mut LogHandleListCs);
        *LogHandle = pLogHandle;
    } else {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pLogHandle as PVOID,
        );
    }

    return Status;
}

unsafe extern "C" fn ElfGetLogHandleEntryByHandle(EventLogHandle: IELF_HANDLE) -> PLOGHANDLE {
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Handle: PLOGHANDLE = core::ptr::null_mut();
    let mut pLogHandle: PLOGHANDLE = core::ptr::null_mut();
    EnterCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
    CurrentEntry = (*core::ptr::addr_of_mut!(LogHandleListHead)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(LogHandleListHead) {
        Handle = CONTAINING_RECORD!(CurrentEntry, LOGHANDLE, LogHandleListEntry);
        CurrentEntry = (*CurrentEntry).Flink;

        if Handle == EventLogHandle as PLOGHANDLE {
            pLogHandle = Handle;
            break;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
    return pLogHandle;
}

unsafe extern "C" fn ElfDeleteEventLogHandle(LogHandle: PIELF_HANDLE) -> NTSTATUS {
    let mut pLogHandle: PLOGHANDLE;

    pLogHandle = ElfGetLogHandleEntryByHandle(*LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));
    RemoveEntryList(core::ptr::addr_of_mut!((*pLogHandle).LogHandleListEntry));
    LeaveCriticalSection(core::ptr::addr_of_mut!(LogHandleListCs));

    LogfClose((*pLogHandle).LogFile, FALSE as BOOLEAN);

    HeapFree(GetProcessHeap(), 0 as DWORD, pLogHandle as PVOID);

    *LogHandle = core::ptr::null_mut();

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrClearELFW(
    LogHandle: IELF_HANDLE,
    BackupFileName: PRPC_UNICODE_STRING,
) -> NTSTATUS {
    let mut pLogHandle: PLOGHANDLE;

    DPRINT!("ElfrClearELFW()\n");

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    /* Fail, if the log file is a backup file */
    if ((*pLogHandle).Flags & LOG_HANDLE_BACKUP_FILE) != 0 {
        return STATUS_INVALID_HANDLE;
    }

    return LogfClearFile(
        (*pLogHandle).LogFile,
        BackupFileName as PUNICODE_STRING,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ElfrBackupELFW(
    LogHandle: IELF_HANDLE,
    BackupFileName: PRPC_UNICODE_STRING,
) -> NTSTATUS {
    let mut pLogHandle: PLOGHANDLE = core::ptr::null_mut();

    DPRINT!("ElfrBackupELFW()\n");

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    return LogfBackupFile((*pLogHandle).LogFile, BackupFileName as PUNICODE_STRING);
}

#[no_mangle]
pub unsafe extern "system" fn ElfrCloseEL(LogHandle: PIELF_HANDLE) -> NTSTATUS {
    ElfDeleteEventLogHandle(LogHandle)
}

#[no_mangle]
pub unsafe extern "system" fn ElfrDeregisterEventSource(LogHandle: PIELF_HANDLE) -> NTSTATUS {
    ElfDeleteEventLogHandle(LogHandle)
}

#[no_mangle]
pub unsafe extern "system" fn ElfrNumberOfRecords(
    LogHandle: IELF_HANDLE,
    NumberOfRecords: PULONG,
) -> NTSTATUS {
    let mut pLogHandle: PLOGHANDLE = core::ptr::null_mut();
    let mut pLogFile: PLOGFILE = core::ptr::null_mut();
    let mut OldestRecordNumber: ULONG;
    let mut CurrentRecordNumber: ULONG;

    DPRINT!("ElfrNumberOfRecords()\n");

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    if NumberOfRecords.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    pLogFile = (*pLogHandle).LogFile;

    /* Lock the log file shared */
    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*pLogFile).Lock), TRUE as BOOLEAN);

    OldestRecordNumber = ElfGetOldestRecord(core::ptr::addr_of_mut!((*pLogFile).LogFile));
    CurrentRecordNumber = ElfGetCurrentRecord(core::ptr::addr_of_mut!((*pLogFile).LogFile));

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*pLogFile).Lock));

    DPRINT!("Oldest: %lu  Current: %lu\n", OldestRecordNumber, CurrentRecordNumber);

    if OldestRecordNumber == 0 {
        /* OldestRecordNumber == 0 when the log is empty */
        *NumberOfRecords = 0;
    } else {
        /* The log contains events */
        *NumberOfRecords = CurrentRecordNumber.wrapping_sub(OldestRecordNumber);
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrOldestRecord(
    LogHandle: IELF_HANDLE,
    OldestRecordNumber: PULONG,
) -> NTSTATUS {
    let mut pLogHandle: PLOGHANDLE = core::ptr::null_mut();
    let mut pLogFile: PLOGFILE = core::ptr::null_mut();

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    if OldestRecordNumber.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    pLogFile = (*pLogHandle).LogFile;

    /* Lock the log file shared */
    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*pLogFile).Lock), TRUE as BOOLEAN);

    *OldestRecordNumber = ElfGetOldestRecord(core::ptr::addr_of_mut!((*pLogFile).LogFile));

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*pLogFile).Lock));

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrChangeNotify(
    LogHandle: IELF_HANDLE,
    ClientId: RPC_CLIENT_ID,
    Event: ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn ElfrOpenELW(
    UNCServerName: EVENTLOG_HANDLE_W,
    ModuleName: PRPC_UNICODE_STRING,
    RegModuleName: PRPC_UNICODE_STRING,
    MajorVersion: ULONG,
    MinorVersion: ULONG,
    LogHandle: PIELF_HANDLE,
) -> NTSTATUS {
    if (MajorVersion != 1) || (MinorVersion != 1) {
        return STATUS_INVALID_PARAMETER;
    }

    /* RegModuleName must be an empty string */
    if (*RegModuleName).Length > 0 {
        return STATUS_INVALID_PARAMETER;
    }

    /* FIXME: UNCServerName must specify the server */

    /* FIXME: Must verify that caller has read access */

    return ElfCreateEventLogHandle(
        LogHandle as *mut PLOGHANDLE,
        ModuleName as PUNICODE_STRING,
        FALSE as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ElfrRegisterEventSourceW(
    UNCServerName: EVENTLOG_HANDLE_W,
    ModuleName: PRPC_UNICODE_STRING,
    RegModuleName: PRPC_UNICODE_STRING,
    MajorVersion: ULONG,
    MinorVersion: ULONG,
    LogHandle: PIELF_HANDLE,
) -> NTSTATUS {
    DPRINT!("ElfrRegisterEventSourceW()\n");

    if (MajorVersion != 1) || (MinorVersion != 1) {
        return STATUS_INVALID_PARAMETER;
    }

    /* RegModuleName must be an empty string */
    if (*RegModuleName).Length as i32 > 0 {
        return STATUS_INVALID_PARAMETER;
    }

    DPRINT!("ModuleName: %wZ\n", ModuleName);

    /* FIXME: UNCServerName must specify the server or empty for local */

    /* FIXME: Must verify that caller has write access */

    return ElfCreateEventLogHandle(
        LogHandle as *mut PLOGHANDLE,
        ModuleName as PUNICODE_STRING,
        TRUE as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ElfrOpenBELW(
    UNCServerName: EVENTLOG_HANDLE_W,
    BackupFileName: PRPC_UNICODE_STRING,
    MajorVersion: ULONG,
    MinorVersion: ULONG,
    LogHandle: PIELF_HANDLE,
) -> NTSTATUS {
    DPRINT!("ElfrOpenBELW(%wZ)\n", BackupFileName);

    if (MajorVersion != 1) || (MinorVersion != 1) {
        return STATUS_INVALID_PARAMETER;
    }

    /* FIXME: UNCServerName must specify the server */

    /* FIXME: Must verify that caller has read access */

    return ElfCreateBackupLogHandle(
        LogHandle as *mut PLOGHANDLE,
        BackupFileName as PUNICODE_STRING,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ElfrReadELW(
    LogHandle: IELF_HANDLE,
    ReadFlags: ULONG,
    RecordOffset: ULONG,
    NumberOfBytesToRead: RULONG,
    Buffer: PBYTE,
    NumberOfBytesRead: PULONG,
    MinNumberOfBytesNeeded: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut pLogHandle: PLOGHANDLE;
    let mut RecordNumber: ULONG;

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    if Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* If sequential read, retrieve the CurrentRecord from this log handle */
    if (ReadFlags & (EVENTLOG_SEQUENTIAL_READ as ULONG)) != 0 {
        RecordNumber = (*pLogHandle).CurrentRecord;
    } else {
        // (ReadFlags & EVENTLOG_SEEK_READ)
        RecordNumber = RecordOffset;
    }

    Status = LogfReadEvents(
        (*pLogHandle).LogFile,
        ReadFlags,
        core::ptr::addr_of_mut!(RecordNumber),
        NumberOfBytesToRead as ULONG,
        Buffer,
        NumberOfBytesRead,
        MinNumberOfBytesNeeded,
        FALSE as BOOLEAN,
    );

    /* Update the handle's CurrentRecord if success */
    if NT_SUCCESS!(Status) {
        (*pLogHandle).CurrentRecord = RecordNumber;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn ElfrIntReportEventW(
    LogHandle: IELF_HANDLE,
    Time: ULONG,
    EventType: USHORT,
    EventCategory: USHORT,
    EventID: ULONG,
    SourceName: PRPC_UNICODE_STRING,
    NumStrings: USHORT,
    DataSize: ULONG,
    ComputerName: PRPC_UNICODE_STRING,
    UserSID: PRPC_SID,
    Strings: *mut PRPC_UNICODE_STRING,
    Data: PBYTE,
    Flags: USHORT,
    RecordNumber: PULONG,
    TimeWritten: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut pLogHandle: PLOGHANDLE;
    let mut LocalSourceName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalComputerName: _UNICODE_STRING = core::mem::zeroed();
    let mut LogBuffer: PEVENTLOGRECORD;
    let mut i: USHORT;
    let mut RecSize: SIZE_T = 0;
    let mut dwStringsSize: ULONG = 0;
    let mut dwUserSidLength: ULONG = 0;
    let mut lpStrings: PWSTR;
    let mut str: PWSTR;

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    /* Flags must be 0 */
    if Flags != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    i = 0 as USHORT;
    while (i as core::ffi::c_int) < NumStrings as core::ffi::c_int {
        match EventType as ULONG {
            EVENTLOG_SUCCESS => {
                DbgPrint(
                    b"(%s:%d) Success: %wZ\n\0" as *const u8 as PCSTR,
                    b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
                    0 as core::ffi::c_int,
                    *Strings.offset(i as isize),
                );
            }
            EVENTLOG_ERROR_TYPE => {
                DbgPrint(
                    b"(%s:%d) Error: %wZ\n\0" as *const u8 as PCSTR,
                    b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
                    0 as core::ffi::c_int,
                    *Strings.offset(i as isize),
                );
            }
            EVENTLOG_WARNING_TYPE => {
                DbgPrint(
                    b"(%s:%d) Warning: %wZ\n\0" as *const u8 as PCSTR,
                    b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
                    0 as core::ffi::c_int,
                    *Strings.offset(i as isize),
                );
            }
            EVENTLOG_INFORMATION_TYPE => {
                DbgPrint(
                    b"(%s:%d) Info: %wZ\n\0" as *const u8 as PCSTR,
                    b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
                    0 as core::ffi::c_int,
                    *Strings.offset(i as isize),
                );
            }
            EVENTLOG_AUDIT_SUCCESS => {
                DbgPrint(
                    b"(%s:%d) Audit Success: %wZ\n\0" as *const u8 as PCSTR,
                    b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
                    0 as core::ffi::c_int,
                    *Strings.offset(i as isize),
                );
            }
            EVENTLOG_AUDIT_FAILURE => {
                DbgPrint(
                    b"(%s:%d) Audit Failure: %wZ\n\0" as *const u8 as PCSTR,
                    b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
                    0 as core::ffi::c_int,
                    *Strings.offset(i as isize),
                );
            }
            _ => {
                DbgPrint(
                    b"(%s:%d) Type %hu: %wZ\n\0" as *const u8 as PCSTR,
                    b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
                    0 as core::ffi::c_int,
                    EventType as core::ffi::c_int,
                    *Strings.offset(i as isize),
                );
            }
        }
        dwStringsSize = dwStringsSize.wrapping_add(
            (**Strings.offset(i as isize)).Length as ULONG
                + core::mem::size_of_val(&UNICODE_NULL) as ULONG,
        );
        i = i.wrapping_add(1);
    }

    lpStrings = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        dwStringsSize as SIZE_T,
    ) as PWSTR;
    if lpStrings.is_null() {
        DbgPrint(
            b"(%s:%d) Failed to allocate heap\n\0" as *const u8 as PCSTR,
            b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
            0 as core::ffi::c_int,
        );
        return STATUS_NO_MEMORY;
    }

    str = lpStrings;
    i = 0 as USHORT;
    while (i as core::ffi::c_int) < NumStrings as core::ffi::c_int {
        RtlCopyMemory!(
            str as PVOID,
            (**Strings.offset(i as isize)).Buffer as PVOID,
            (**Strings.offset(i as isize)).Length as SIZE_T,
        );
        str = str.offset(
            ((**Strings.offset(i as isize)).Length as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
        );
        *str = UNICODE_NULL as WCHAR;
        str = str.offset(1);
        i = i.wrapping_add(1);
    }

    if !UserSID.is_null() {
        dwUserSidLength = (core::mem::offset_of!(SID, SubAuthority)
            + (*UserSID).SubAuthorityCount as usize * core::mem::size_of::<ULONG>())
            as ULONG;
    }

    if !SourceName.is_null() && !(*SourceName).Buffer.is_null() {
        LocalSourceName = *(SourceName as PUNICODE_STRING);
    } else {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(LocalSourceName),
            core::ptr::addr_of_mut!((*pLogHandle).szName) as *mut WCHAR as PCWSTR,
        );
    }

    LocalComputerName = *(ComputerName as PUNICODE_STRING);

    LogBuffer = LogfAllocAndBuildNewRecord(
        core::ptr::addr_of_mut!(RecSize),
        Time,
        EventType,
        EventCategory,
        EventID,
        core::ptr::addr_of_mut!(LocalSourceName),
        core::ptr::addr_of_mut!(LocalComputerName),
        dwUserSidLength,
        UserSID as PSID,
        NumStrings,
        lpStrings,
        DataSize,
        Data as PVOID,
    );
    if LogBuffer.is_null() {
        DbgPrint(
            b"(%s:%d) LogfAllocAndBuildNewRecord failed!\n\0" as *const u8 as PCSTR,
            b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
            0 as core::ffi::c_int,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpStrings as PVOID,
        );
        return STATUS_NO_MEMORY;
    }

    Status = LogfWriteRecord((*pLogHandle).LogFile, LogBuffer, RecSize);
    if !NT_SUCCESS!(Status) {
        DbgPrint(
            b"(%s:%d) ERROR writing to event log `%S' (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
            b"rpc.c\0" as *const u8 as *const core::ffi::c_char,
            0 as core::ffi::c_int,
            (*(*pLogHandle).LogFile).LogName,
            Status,
        );
    }

    if NT_SUCCESS!(Status) {
        /* Retrieve the two fields that were set by LogfWriteRecord into the record */
        if !RecordNumber.is_null() {
            *RecordNumber = (*LogBuffer).RecordNumber as ULONG;
        }
        if !TimeWritten.is_null() {
            *TimeWritten = (*LogBuffer).TimeWritten as ULONG;
        }
    }

    LogfFreeRecord(LogBuffer);

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        lpStrings as PVOID,
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrReportEventW(
    LogHandle: IELF_HANDLE,
    Time: ULONG,
    EventType: USHORT,
    EventCategory: USHORT,
    EventID: ULONG,
    NumStrings: USHORT,
    DataSize: ULONG,
    ComputerName: PRPC_UNICODE_STRING,
    UserSID: PRPC_SID,
    Strings: *mut PRPC_UNICODE_STRING,
    Data: PBYTE,
    Flags: USHORT,
    RecordNumber: PULONG,
    TimeWritten: PULONG,
) -> NTSTATUS {
    return ElfrIntReportEventW(
        LogHandle,
        Time,
        EventType,
        EventCategory,
        EventID,
        core::ptr::null_mut(),
        NumStrings,
        DataSize,
        ComputerName,
        UserSID,
        Strings,
        Data,
        Flags,
        RecordNumber,
        TimeWritten,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ElfrClearELFA(
    LogHandle: IELF_HANDLE,
    BackupFileName: PRPC_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BackupFileNameW: _UNICODE_STRING = core::mem::zeroed();

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(BackupFileNameW),
        BackupFileName as PANSI_STRING,
        TRUE as BOOLEAN,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    Status = ElfrClearELFW(
        LogHandle,
        core::ptr::addr_of_mut!(BackupFileNameW) as PRPC_UNICODE_STRING,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(BackupFileNameW));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrBackupELFA(
    LogHandle: IELF_HANDLE,
    BackupFileName: PRPC_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BackupFileNameW: _UNICODE_STRING = core::mem::zeroed();

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(BackupFileNameW),
        BackupFileName as PANSI_STRING,
        TRUE as BOOLEAN,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    Status = ElfrBackupELFW(
        LogHandle,
        core::ptr::addr_of_mut!(BackupFileNameW) as PRPC_UNICODE_STRING,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(BackupFileNameW));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrOpenELA(
    UNCServerName: EVENTLOG_HANDLE_A,
    ModuleName: PRPC_STRING,
    RegModuleName: PRPC_STRING,
    MajorVersion: ULONG,
    MinorVersion: ULONG,
    LogHandle: PIELF_HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ModuleNameW: _UNICODE_STRING = core::mem::zeroed();

    if (MajorVersion != 1) || (MinorVersion != 1) {
        return STATUS_INVALID_PARAMETER;
    }

    /* RegModuleName must be an empty string */
    if (*RegModuleName).Length > 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(ModuleNameW),
        ModuleName as PANSI_STRING,
        TRUE as BOOLEAN,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    /* FIXME: Must verify that caller has read access */

    Status = ElfCreateEventLogHandle(
        LogHandle as *mut PLOGHANDLE,
        core::ptr::addr_of_mut!(ModuleNameW),
        FALSE as BOOLEAN,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ModuleNameW));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrRegisterEventSourceA(
    UNCServerName: EVENTLOG_HANDLE_A,
    ModuleName: PRPC_STRING,
    RegModuleName: PRPC_STRING,
    MajorVersion: ULONG,
    MinorVersion: ULONG,
    LogHandle: PIELF_HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ModuleNameW: _UNICODE_STRING = core::mem::zeroed();

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(ModuleNameW),
        ModuleName as PANSI_STRING,
        TRUE as BOOLEAN,
    );
    if !NT_SUCCESS!(Status) {
        if DbgPrint(
            b"(%s:%d) RtlAnsiStringToUnicodeString failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            b"rpc.c\0".as_ptr() as *const i8,
            865,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"rpc.c\0".as_ptr() as *const i8,
                865,
            );
        }
        return Status;
    }

    /* RegModuleName must be an empty string */
    if (*RegModuleName).Length > 0 {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(ModuleNameW));
        return STATUS_INVALID_PARAMETER;
    }

    if (MajorVersion != 1) || (MinorVersion != 1) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(ModuleNameW));
        return STATUS_INVALID_PARAMETER;
    }

    /* FIXME: Must verify that caller has write access */

    Status = ElfCreateEventLogHandle(
        LogHandle as *mut PLOGHANDLE,
        core::ptr::addr_of_mut!(ModuleNameW),
        TRUE as BOOLEAN,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ModuleNameW));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrOpenBELA(
    UNCServerName: EVENTLOG_HANDLE_A,
    BackupFileName: PRPC_STRING,
    MajorVersion: ULONG,
    MinorVersion: ULONG,
    LogHandle: PIELF_HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BackupFileNameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(BackupFileNameW),
        BackupFileName as PANSI_STRING,
        TRUE as BOOLEAN,
    );
    if !NT_SUCCESS!(Status) {
        DbgPrint(
            b"(%s:%d) RtlAnsiStringToUnicodeString failed (Status 0x%08lx)\n\0".as_ptr() as PCSTR,
            b"rpc.c\0".as_ptr() as *const core::ffi::c_char,
            914 as core::ffi::c_int,
            Status,
        );
        return Status;
    }

    if (MajorVersion != 1) || (MinorVersion != 1) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(BackupFileNameW));
        return STATUS_INVALID_PARAMETER;
    }

    /* FIXME: UNCServerName must specify the server */

    /* FIXME: Must verify that caller has read access */

    Status = ElfCreateBackupLogHandle(
        LogHandle as *mut PLOGHANDLE,
        core::ptr::addr_of_mut!(BackupFileNameW),
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(BackupFileNameW));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrReadELA(
    LogHandle: IELF_HANDLE,
    ReadFlags: ULONG,
    RecordOffset: ULONG,
    NumberOfBytesToRead: RULONG,
    Buffer: PBYTE,
    NumberOfBytesRead: PULONG,
    MinNumberOfBytesNeeded: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut pLogHandle: PLOGHANDLE;
    let mut RecordNumber: ULONG;

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    if Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* If sequential read, retrieve the CurrentRecord from this log handle */
    if (ReadFlags & EVENTLOG_SEQUENTIAL_READ as ULONG) != 0 {
        RecordNumber = (*pLogHandle).CurrentRecord;
    } else {
        // (ReadFlags & EVENTLOG_SEEK_READ)
        RecordNumber = RecordOffset;
    }

    Status = LogfReadEvents(
        (*pLogHandle).LogFile,
        ReadFlags,
        core::ptr::addr_of_mut!(RecordNumber),
        NumberOfBytesToRead,
        Buffer,
        NumberOfBytesRead,
        MinNumberOfBytesNeeded,
        TRUE as BOOLEAN,
    );

    /* Update the handle's CurrentRecord if success */
    if NT_SUCCESS!(Status) {
        (*pLogHandle).CurrentRecord = RecordNumber;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrReportEventA(
    LogHandle: IELF_HANDLE,
    Time: ULONG,
    EventType: USHORT,
    EventCategory: USHORT,
    EventID: ULONG,
    NumStrings: USHORT,
    DataSize: ULONG,
    ComputerName: PRPC_STRING,
    UserSID: PRPC_SID,
    Strings: *mut PRPC_STRING,
    Data: PBYTE,
    Flags: USHORT,
    RecordNumber: PULONG,
    TimeWritten: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut ComputerNameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut StringsArrayW: *mut PUNICODE_STRING = core::ptr::null_mut();
    let mut i: USHORT = 0;

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(ComputerNameW) as PUNICODE_STRING,
        ComputerName as PANSI_STRING,
        TRUE as BOOLEAN,
    );
    if !(Status >= 0) {
        return Status;
    }

    'done: {
        if NumStrings != 0 {
            StringsArrayW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                (NumStrings as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<PUNICODE_STRING>() as SIZE_T),
            ) as *mut PUNICODE_STRING;
            if StringsArrayW.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'done;
            }

            i = 0 as USHORT;
            while (i as i32) < (NumStrings as i32) {
                if !(*Strings.offset(i as isize)).is_null() {
                    *StringsArrayW.offset(i as isize) = HeapAlloc(
                        GetProcessHeap(),
                        HEAP_ZERO_MEMORY as DWORD,
                        core::mem::size_of::<_UNICODE_STRING>() as SIZE_T,
                    ) as PUNICODE_STRING;
                    if (*StringsArrayW.offset(i as isize)).is_null() {
                        Status = STATUS_NO_MEMORY;
                        break;
                    }

                    Status = RtlAnsiStringToUnicodeString(
                        *StringsArrayW.offset(i as isize),
                        *Strings.offset(i as isize) as PANSI_STRING,
                        TRUE as BOOLEAN,
                    );
                }

                if !(Status >= 0) {
                    break;
                }
                i = i.wrapping_add(1);
            }
        }

        if Status >= 0 {
            Status = ElfrReportEventW(
                LogHandle,
                Time,
                EventType,
                EventCategory,
                EventID,
                NumStrings,
                DataSize,
                core::ptr::addr_of_mut!(ComputerNameW) as PRPC_UNICODE_STRING,
                UserSID,
                StringsArrayW as *mut PRPC_UNICODE_STRING,
                Data,
                Flags,
                RecordNumber,
                TimeWritten,
            );
        }
    }

    // Done:
    if !StringsArrayW.is_null() {
        i = 0 as USHORT;
        while (i as i32) < (NumStrings as i32) {
            if !(*StringsArrayW.offset(i as isize)).is_null()
                && !(**StringsArrayW.offset(i as isize)).Buffer.is_null()
            {
                RtlFreeUnicodeString(*StringsArrayW.offset(i as isize));
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    *StringsArrayW.offset(i as isize) as PVOID,
                );
            }
            i = i.wrapping_add(1);
        }

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            StringsArrayW as PVOID,
        );
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ComputerNameW));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrRegisterClusterSvc(BindingHandle: handle_t) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn ElfrDeregisterClusterSvc(BindingHandle: handle_t) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn ElfrWriteClusterEvents(BindingHandle: handle_t) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn ElfrGetLogInformation(
    mut LogHandle: IELF_HANDLE,
    mut InfoLevel: ULONG,
    mut Buffer: PBYTE,
    mut cbBufSize: ULONG,
    mut pcbBytesNeeded: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut pLogHandle: PLOGHANDLE = core::ptr::null_mut();
    let mut pLogFile: PLOGFILE = core::ptr::null_mut();

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    pLogFile = (*pLogHandle).LogFile;

    /* Lock the log file shared */
    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*pLogFile).Lock), TRUE as BOOLEAN);

    match InfoLevel {
        EVENTLOG_FULL_INFO => {
            let mut efi: LPEVENTLOG_FULL_INFORMATION = Buffer as LPEVENTLOG_FULL_INFORMATION;

            *pcbBytesNeeded = core::mem::size_of::<EVENTLOG_FULL_INFORMATION>() as ULONG;
            if (cbBufSize as usize) < core::mem::size_of::<EVENTLOG_FULL_INFORMATION>() as usize {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                (*efi).dwFull = ((ElfGetFlags(core::ptr::addr_of_mut!((*pLogFile).LogFile))
                    & ELF_LOGFILE_LOGFULL_WRITTEN as ULONG)
                    != 0) as ::core::ffi::c_int as DWORD;
            }
        }
        _ => {
            Status = STATUS_INVALID_LEVEL;
        }
    }

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*pLogFile).Lock));

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn ElfrFlushEL(mut LogHandle: IELF_HANDLE) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut pLogHandle: PLOGHANDLE = core::ptr::null_mut();
    let mut pLogFile: PLOGFILE = core::ptr::null_mut();

    pLogHandle = ElfGetLogHandleEntryByHandle(LogHandle);
    if pLogHandle.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    pLogFile = (*pLogHandle).LogFile;

    /* Lock the log file exclusive */
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*pLogFile).Lock), TRUE as BOOLEAN);

    Status = ElfFlushFile(core::ptr::addr_of_mut!((*pLogFile).LogFile));

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*pLogFile).Lock));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ElfrReportEventAndSourceW(
    LogHandle: IELF_HANDLE,
    Time: ULONG,
    EventType: USHORT,
    EventCategory: USHORT,
    EventID: ULONG,
    SourceName: PRPC_UNICODE_STRING,
    NumStrings: USHORT,
    DataSize: ULONG,
    ComputerName: PRPC_UNICODE_STRING,
    UserSID: PRPC_SID,
    Strings: *mut PRPC_UNICODE_STRING,
    Data: PBYTE,
    Flags: USHORT,
    RecordNumber: PULONG,
    TimeWritten: PULONG,
) -> NTSTATUS {
    /* Call the helper function. The event source is specified by the caller. */
    return ElfrIntReportEventW(
        LogHandle,
        Time,
        EventType,
        EventCategory,
        EventID,
        SourceName,
        NumStrings,
        DataSize,
        ComputerName,
        UserSID,
        Strings,
        Data,
        Flags,
        RecordNumber,
        TimeWritten,
    );
}
#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, len) as *mut core::ffi::c_void;
}
#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0 as DWORD, ptr as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn IELF_HANDLE_rundown(mut LogHandle: IELF_HANDLE) {
    // Close the handle
    ElfDeleteEventLogHandle(core::ptr::addr_of_mut!(LogHandle)); // ElfrCloseEL(&LogHandle);
}
