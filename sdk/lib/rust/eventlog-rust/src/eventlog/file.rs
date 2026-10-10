use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut LogFileListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut LogFileListCs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};
#[no_mangle]
pub unsafe extern "C" fn LogfListInitialize() {
    InitializeCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
    InitializeListHead(core::ptr::addr_of_mut!(LogFileListHead));
}
#[no_mangle]
pub unsafe extern "C" fn LogfListItemByName(Name: LPCWSTR) -> PLOGFILE {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut Item: PLOGFILE;
    let mut Result: PLOGFILE = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));

    CurrentEntry = LogFileListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(LogFileListHead) {
        Item = ((CurrentEntry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(LOGFILE, ListEntry) as ULONG_PTR))
            as PLOGFILE;

        if !(*Item).LogName.is_null() && _wcsicmp((*Item).LogName, Name) == 0 {
            Result = Item;
            break;
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn LogfListItemByIndex(Index: DWORD) -> PLOGFILE {
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Result: PLOGFILE = core::ptr::null_mut();
    let mut i: DWORD = 1 as DWORD;

    EnterCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));

    CurrentEntry = (*core::ptr::addr_of_mut!(LogFileListHead)).Flink as PLIST_ENTRY;
    while CurrentEntry != core::ptr::addr_of_mut!(LogFileListHead) as PLIST_ENTRY {
        if i == Index {
            Result = (CurrentEntry as *mut u8)
                .wrapping_sub(core::mem::offset_of!(LOGFILE, ListEntry))
                as PLOGFILE;
            break;
        }

        CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
        i = i.wrapping_add(1);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn LogfListItemCount() -> DWORD {
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut i: DWORD = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));

    CurrentEntry = (*core::ptr::addr_of_mut!(LogFileListHead)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(LogFileListHead) {
        CurrentEntry = (*CurrentEntry).Flink;
        i += 1;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
    return i;
}

unsafe extern "C" fn LogfListAddItem(Item: PLOGFILE) {
    EnterCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
    InsertTailList(core::ptr::addr_of_mut!(LogFileListHead), core::ptr::addr_of_mut!((*Item).ListEntry));
    LeaveCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
}

unsafe extern "C" fn LogfListRemoveItem(Item: PLOGFILE) {
    EnterCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
    RemoveEntryList(core::ptr::addr_of_mut!((*Item).ListEntry));
    LeaveCriticalSection(core::ptr::addr_of_mut!(LogFileListCs));
}

unsafe extern "system" fn LogfpAlloc(Size: SIZE_T, Flags: ULONG, Tag: ULONG) -> PVOID {
    let _ = Tag;
    return RtlAllocateHeap(GetProcessHeap() as PVOID, Flags, Size);
}
unsafe extern "system" fn LogfpFree(mut Ptr: PVOID, mut Flags: ULONG, mut Tag: ULONG) {
    let _ = Tag; // UNREFERENCED_PARAMETER(Tag);
    RtlFreeHeap(GetProcessHeap(), Flags, Ptr);
}

unsafe extern "system" fn LogfpReadFile(
    LogFile: PEVTLOGFILE,
    FileOffset: PLARGE_INTEGER,
    Buffer: PVOID,
    Length: SIZE_T,
    ReadLength: PSIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut pLogFile: PLOGFILE = LogFile as PLOGFILE;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if !ReadLength.is_null() {
        *ReadLength = 0;
    }

    Status = NtReadFile(
        (*pLogFile).FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &raw mut IoStatusBlock,
        Buffer,
        Length as ULONG,
        FileOffset,
        core::ptr::null_mut(),
    );

    if !ReadLength.is_null() {
        *ReadLength = IoStatusBlock.Information;
    }

    return Status;
}
unsafe extern "system" fn LogfpWriteFile(
    mut LogFile: PEVTLOGFILE,
    mut FileOffset: PLARGE_INTEGER,
    mut Buffer: PVOID,
    mut Length: SIZE_T,
    mut WrittenLength: PSIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut pLogFile: PLOGFILE = LogFile as PLOGFILE;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if !WrittenLength.is_null() {
        *WrittenLength = 0;
    }

    Status = NtWriteFile(
        (*pLogFile).FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        Buffer,
        Length as ULONG,
        FileOffset,
        core::ptr::null_mut(),
    );

    if !WrittenLength.is_null() {
        *WrittenLength = IoStatusBlock.Information;
    }

    return Status;
}

unsafe extern "system" fn LogfpSetFileSize(
    LogFile: PEVTLOGFILE,
    FileSize: ULONG,
    OldFileSize: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let pLogFile: PLOGFILE = LogFile as PLOGFILE;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileEofInfo: FILE_END_OF_FILE_INFORMATION = core::mem::zeroed();
    let mut FileAllocInfo: FILE_ALLOCATION_INFORMATION = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(OldFileSize);

    FileEofInfo.EndOfFile.QuadPart = FileSize as LONGLONG;
    Status = NtSetInformationFile(
        (*pLogFile).FileHandle,
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::addr_of_mut!(FileEofInfo) as PVOID,
        core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as ULONG,
        FileEndOfFileInformation,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    FileAllocInfo.AllocationSize.QuadPart = FileSize as LONGLONG;
    Status = NtSetInformationFile(
        (*pLogFile).FileHandle,
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::addr_of_mut!(FileAllocInfo) as PVOID,
        core::mem::size_of::<FILE_ALLOCATION_INFORMATION>() as ULONG,
        FileAllocationInformation,
    );

    return Status;
}
unsafe extern "system" fn LogfpFlushFile(
    LogFile: PEVTLOGFILE,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
) -> NTSTATUS {
    let pLogFile: PLOGFILE = LogFile as PLOGFILE;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(FileOffset);
    UNREFERENCED_PARAMETER!(Length);

    return NtFlushBuffersFile((*pLogFile).FileHandle, core::ptr::addr_of_mut!(IoStatusBlock));
}
#[no_mangle]
pub unsafe extern "C" fn LogfCreate(
    LogFile: *mut PLOGFILE,
    LogName: PCWSTR,
    FileName: PUNICODE_STRING,
    MaxSize: ULONG,
    Retention: ULONG,
    Permanent: BOOLEAN,
    Backup: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileStdInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut pLogFile: PLOGFILE;
    let mut LogNameLen: SIZE_T;
    let mut CreateNew: BOOLEAN;

    pLogFile = LogfpAlloc(
        core::mem::size_of::<LOGFILE>() as SIZE_T,
        HEAP_ZERO_MEMORY,
        TAG_ELF as ULONG,
    ) as PLOGFILE;
    if pLogFile.is_null() {
        DPRINT1!(b"Cannot allocate heap!\n\0".as_ptr() as PCSTR);
        return STATUS_NO_MEMORY;
    }

    'quit: {
        LogNameLen = ((if !LogName.is_null() {
            wcslen(LogName)
        } else {
            0
        }) + 1) as SIZE_T;
        (*pLogFile).LogName = LogfpAlloc(
            LogNameLen * core::mem::size_of::<WCHAR>() as SIZE_T,
            HEAP_ZERO_MEMORY,
            0,
        ) as *mut WCHAR;
        if (*pLogFile).LogName.is_null() {
            DPRINT1!(b"Cannot allocate heap\n\0".as_ptr() as PCSTR);
            Status = STATUS_NO_MEMORY;
            break 'quit;
        }

        if !LogName.is_null() {
            StringCchCopyW((*pLogFile).LogName, LogNameLen as size_t, LogName);
        }

        // InitializeObjectAttributes(&ObjectAttributes, FileName, OBJ_CASE_INSENSITIVE, NULL, NULL)
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = FileName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        DPRINT!(
            b"Going to create or open %wZ\n\0".as_ptr() as PCSTR,
            FileName,
        );
        Status = NtCreateFile(
            core::ptr::addr_of_mut!((*pLogFile).FileHandle),
            if Backup != 0 {
                GENERIC_READ | SYNCHRONIZE
            } else {
                GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE
            },
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(IoStatusBlock),
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ,
            if Backup != 0 {
                FILE_OPEN
            } else {
                FILE_OPEN_IF
            },
            FILE_SYNCHRONOUS_IO_NONALERT,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"Cannot create file `%wZ' (Status 0x%08lx)\n\0".as_ptr() as PCSTR,
                FileName,
                Status,
            );
            break 'quit;
        }

        CreateNew = (IoStatusBlock.Information == FILE_CREATED as ULONG_PTR) as BOOLEAN;
        DPRINT!(
            b"%wZ %s successfully\n\0".as_ptr() as PCSTR,
            FileName,
            (if CreateNew != 0 {
                b"created\0".as_ptr()
            } else {
                b"opened\0".as_ptr()
            }) as PCSTR,
        );

        /*
         * Retrieve the log file size and check whether the file is not too large;
         * this log format only supports files of theoretical size < 0xFFFFFFFF .
         *
         * As it happens that, on Windows (and ReactOS), retrieving the End-Of-File
         * information using NtQueryInformationFile with the FileEndOfFileInformation
         * class is invalid (who knows why...), use instead the FileStandardInformation
         * class, and the EndOfFile member of the returned FILE_STANDARD_INFORMATION
         * structure will give the desired information.
         */
        Status = NtQueryInformationFile(
            (*pLogFile).FileHandle,
            core::ptr::addr_of_mut!(IoStatusBlock),
            core::ptr::addr_of_mut!(FileStdInfo) as PVOID,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
            FileStandardInformation,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"EventLog: NtQueryInformationFile failed (Status 0x%08lx)\n\0".as_ptr() as PCSTR,
                Status,
            );
            break 'quit;
        }
        if FileStdInfo.EndOfFile.u.HighPart != 0 {
            DPRINT1!(
                b"EventLog: Log `%wZ' is too large.\n\0".as_ptr() as PCSTR,
                FileName,
            );
            Status = STATUS_EVENTLOG_FILE_CORRUPT; // STATUS_FILE_TOO_LARGE;
            break 'quit;
        }

        DPRINT!(
            b"Initializing LogFile `%S'\n\0".as_ptr() as PCSTR,
            (*pLogFile).LogName,
        );

        Status = ElfCreateFile(
            core::ptr::addr_of_mut!((*pLogFile).LogFile),
            FileName,
            FileStdInfo.EndOfFile.u.LowPart as ULONG,
            MaxSize,
            Retention,
            CreateNew,
            Backup,
            Some(LogfpAlloc as unsafe extern "system" fn(SIZE_T, ULONG, ULONG) -> PVOID),
            Some(LogfpFree as unsafe extern "system" fn(PVOID, ULONG, ULONG) -> ()),
            Some(
                LogfpSetFileSize
                    as unsafe extern "system" fn(PEVTLOGFILE, ULONG, ULONG) -> NTSTATUS,
            ),
            Some(
                LogfpWriteFile
                    as unsafe extern "system" fn(
                        PEVTLOGFILE,
                        PLARGE_INTEGER,
                        PVOID,
                        SIZE_T,
                        PSIZE_T,
                    ) -> NTSTATUS,
            ),
            Some(
                LogfpReadFile
                    as unsafe extern "system" fn(
                        PEVTLOGFILE,
                        PLARGE_INTEGER,
                        PVOID,
                        SIZE_T,
                        PSIZE_T,
                    ) -> NTSTATUS,
            ),
            Some(
                LogfpFlushFile
                    as unsafe extern "system" fn(PEVTLOGFILE, PLARGE_INTEGER, ULONG) -> NTSTATUS,
            ),
        );
        if !NT_SUCCESS!(Status) {
            break 'quit;
        }

        (*pLogFile).Permanent = Permanent as BOOL;

        RtlInitializeResource(core::ptr::addr_of_mut!((*pLogFile).Lock));

        LogfListAddItem(pLogFile);
    }

    // Quit:
    if !NT_SUCCESS!(Status) {
        if !(*pLogFile).FileHandle.is_null() {
            NtClose((*pLogFile).FileHandle);
        }

        if !(*pLogFile).LogName.is_null() {
            LogfpFree(
                (*pLogFile).LogName as PVOID,
                0,
                0,
            );
        }

        LogfpFree(pLogFile as PVOID, 0, TAG_ELF as ULONG);
    } else {
        *LogFile = pLogFile;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn LogfClose(LogFile: PLOGFILE, ForceClose: BOOLEAN) {
    if LogFile.is_null() {
        return;
    }

    if ForceClose == 0 && (*LogFile).Permanent != 0 {
        return;
    }

    RtlAcquireResourceExclusive(&raw mut (*LogFile).Lock, TRUE as BOOLEAN);

    LogfListRemoveItem(LogFile);

    ElfCloseFile(&raw mut (*LogFile).LogFile);
    NtClose((*LogFile).FileHandle);
    LogfpFree((*LogFile).LogName as PVOID, 0, 0);

    RtlDeleteResource(&raw mut (*LogFile).Lock);

    LogfpFree(LogFile as PVOID, 0, TAG_ELF as ULONG);

    return;
}
#[no_mangle]
pub unsafe extern "C" fn LogfCloseAll() {
    EnterCriticalSection(&raw mut LogFileListCs);
    while IsListEmpty(&raw mut LogFileListHead) == false {
        LogfClose(
            ((LogFileListHead.Flink as ULONG_PTR)
                .wrapping_sub(&raw mut (*::core::ptr::null_mut::<LOGFILE>()).ListEntry as ULONG_PTR))
                as PLOGFILE,
            TRUE as BOOLEAN,
        );
    }
    LeaveCriticalSection(&raw mut LogFileListCs);
    DeleteCriticalSection(&raw mut LogFileListCs);
}
#[no_mangle]
pub unsafe extern "C" fn LogfClearFile(
    LogFile: PLOGFILE,
    BackupFileName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    /* Lock the log file exclusive */
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!((*LogFile).Lock), TRUE as BOOLEAN);

    'quit: {
        if (*BackupFileName).Length > 0 {
            /* Write a backup file */
            Status = LogfBackupFile(LogFile, BackupFileName);
            if !NT_SUCCESS!(Status) {
                DPRINT1!("LogfBackupFile failed (Status 0x%08lx)\n", Status);
                break 'quit;
            }
        }

        Status = ElfReCreateFile(core::ptr::addr_of_mut!((*LogFile).LogFile));
        if !NT_SUCCESS!(Status) {
            DPRINT1!("LogfInitializeNew failed (Status 0x%08lx)\n", Status);
        }
    }

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*LogFile).Lock));
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn LogfBackupFile(LogFile: PLOGFILE, BackupFileName: PUNICODE_STRING) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BackupLogFile: LOGFILE = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    // DPRINT is a no-op under NDEBUG; omitted.

    /* Lock the log file shared */
    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*LogFile).Lock), TRUE as BOOLEAN);

    // InitializeObjectAttributes(&ObjectAttributes, BackupFileName, OBJ_CASE_INSENSITIVE, NULL, NULL), expanded inline
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = BackupFileName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        core::ptr::addr_of_mut!(BackupLogFile.FileHandle),
        GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ,
        FILE_CREATE,
        FILE_WRITE_THROUGH | FILE_SYNCHRONOUS_IO_NONALERT,
        core::ptr::null_mut(),
        0,
    );

    'quit: {
        if !NT_SUCCESS!(Status) {
            // DPRINT("Cannot create backup file ...") omitted under NDEBUG
            break 'quit;
        }

        Status = ElfBackupFile(
            core::ptr::addr_of_mut!((*LogFile).LogFile),
            core::ptr::addr_of_mut!(BackupLogFile.LogFile),
        );
    }

    /* Close the backup file */
    if !BackupLogFile.FileHandle.is_null() {
        NtClose(BackupLogFile.FileHandle);
    }

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*LogFile).Lock));

    return Status;
}

unsafe extern "C" fn ReadRecord(
    mut LogFile: PEVTLOGFILE,
    mut RecordNumber: ULONG,
    mut Record: PEVENTLOGRECORD,
    mut BufSize: SIZE_T,
    mut BytesRead: PSIZE_T,
    mut BytesNeeded: PSIZE_T,
    mut Ansi: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut UnicodeBuffer: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut Src: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut Dst: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut StringA: ANSI_STRING = core::mem::zeroed();
    let mut StringW: _UNICODE_STRING = core::mem::zeroed();
    let mut SrcPtr: PVOID = core::ptr::null_mut();
    let mut DstPtr: PVOID = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut dwPadding: DWORD = 0;
    let mut dwRecordLength: DWORD = 0;
    let mut pLength: PDWORD = core::ptr::null_mut();

    if Ansi == 0 {
        return ElfReadRecord(
            LogFile,
            RecordNumber,
            Record,
            BufSize as usize,
            BytesRead as *mut usize,
            BytesNeeded as *mut usize,
        );
    }

    if !BytesRead.is_null() {
        *BytesRead = 0;
    }

    if !BytesNeeded.is_null() {
        *BytesNeeded = 0;
    }

    UnicodeBuffer = LogfpAlloc(
        BufSize,
        HEAP_ZERO_MEMORY as ULONG,
        TAG_ELF_BUF as ULONG,
    ) as PEVENTLOGRECORD;
    if UnicodeBuffer.is_null() {
        if DbgPrint(
            b"(%s:%d) Alloc failed!\n\0".as_ptr() as PCSTR,
            b"base/services/eventlog/file.c\0"
                .as_ptr() as *const core::ffi::c_char,
            589 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"base/services/eventlog/file.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                589 as core::ffi::c_int,
            );
        }
        return STATUS_NO_MEMORY;
    }

    'quit: {
        Status = ElfReadRecord(
            LogFile,
            RecordNumber,
            UnicodeBuffer,
            BufSize as usize,
            BytesRead as *mut usize,
            BytesNeeded as *mut usize,
        );
        if !NT_SUCCESS!(Status) {
            break 'quit;
        }

        Src = UnicodeBuffer;
        Dst = Record;

        (*Dst).Reserved = (*Src).Reserved;
        (*Dst).RecordNumber = (*Src).RecordNumber;
        (*Dst).TimeGenerated = (*Src).TimeGenerated;
        (*Dst).TimeWritten = (*Src).TimeWritten;
        (*Dst).EventID = (*Src).EventID;
        (*Dst).EventType = (*Src).EventType;
        (*Dst).EventCategory = (*Src).EventCategory;
        (*Dst).NumStrings = (*Src).NumStrings;
        (*Dst).UserSidLength = (*Src).UserSidLength;
        (*Dst).DataLength = (*Src).DataLength;

        SrcPtr = ((Src as ULONG_PTR)
            .wrapping_add(core::mem::size_of::<EVENTLOGRECORD>() as ULONG_PTR))
            as PVOID;
        DstPtr = ((Dst as ULONG_PTR)
            .wrapping_add(core::mem::size_of::<EVENTLOGRECORD>() as ULONG_PTR))
            as PVOID;

        /* Convert the module name */
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringW), SrcPtr as PCWSTR);
        Status = RtlUnicodeStringToAnsiString(
            core::ptr::addr_of_mut!(StringA),
            core::ptr::addr_of_mut!(StringW) as PCUNICODE_STRING,
            TRUE as BOOLEAN,
        );
        if NT_SUCCESS!(Status) {
            RtlCopyMemory!(
                DstPtr,
                StringA.Buffer as *const core::ffi::c_void,
                StringA.MaximumLength as usize,
            );
            DstPtr = ((DstPtr as ULONG_PTR).wrapping_add(StringA.MaximumLength as ULONG_PTR))
                as PVOID;

            RtlFreeAnsiString(core::ptr::addr_of_mut!(StringA));
        } else {
            RtlZeroMemory!(
                DstPtr,
                (StringW.MaximumLength as usize) / core::mem::size_of::<WCHAR>(),
            );
            DstPtr = ((DstPtr as ULONG_PTR).wrapping_add(
                (StringW.MaximumLength as ULONG_PTR) / (core::mem::size_of::<WCHAR>() as ULONG_PTR),
            )) as PVOID;
        }
        SrcPtr = ((SrcPtr as ULONG_PTR).wrapping_add(StringW.MaximumLength as ULONG_PTR)) as PVOID;

        /* Convert the computer name */
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringW), SrcPtr as PCWSTR);
        Status = RtlUnicodeStringToAnsiString(
            core::ptr::addr_of_mut!(StringA),
            core::ptr::addr_of_mut!(StringW) as PCUNICODE_STRING,
            TRUE as BOOLEAN,
        );
        if NT_SUCCESS!(Status) {
            RtlCopyMemory!(
                DstPtr,
                StringA.Buffer as *const core::ffi::c_void,
                StringA.MaximumLength as usize,
            );
            DstPtr = ((DstPtr as ULONG_PTR).wrapping_add(StringA.MaximumLength as ULONG_PTR))
                as PVOID;

            RtlFreeAnsiString(core::ptr::addr_of_mut!(StringA));
        } else {
            RtlZeroMemory!(
                DstPtr,
                (StringW.MaximumLength as usize) / core::mem::size_of::<WCHAR>(),
            );
            DstPtr = ((DstPtr as ULONG_PTR).wrapping_add(
                (StringW.MaximumLength as ULONG_PTR) / (core::mem::size_of::<WCHAR>() as ULONG_PTR),
            )) as PVOID;
        }

        /* Add the padding and the User SID */
        dwPadding = (core::mem::size_of::<ULONG>() as ULONG_PTR).wrapping_sub(
            ((DstPtr as ULONG_PTR).wrapping_sub(Dst as ULONG_PTR))
                % (core::mem::size_of::<ULONG>() as ULONG_PTR),
        ) as DWORD;
        RtlZeroMemory!(DstPtr, dwPadding as usize);

        SrcPtr = ((Src as ULONG_PTR).wrapping_add((*Src).UserSidOffset as ULONG_PTR)) as PVOID;
        DstPtr = ((DstPtr as ULONG_PTR).wrapping_add(dwPadding as ULONG_PTR)) as PVOID;

        (*Dst).UserSidOffset = ((DstPtr as ULONG_PTR).wrapping_sub(Dst as ULONG_PTR)) as DWORD;
        RtlCopyMemory!(
            DstPtr,
            SrcPtr as *const core::ffi::c_void,
            (*Src).UserSidLength as usize,
        );

        /* Convert the strings */
        SrcPtr = ((Src as ULONG_PTR).wrapping_add((*Src).StringOffset as ULONG_PTR)) as PVOID;
        DstPtr = ((DstPtr as ULONG_PTR).wrapping_add((*Src).UserSidLength as ULONG_PTR)) as PVOID;
        (*Dst).StringOffset = ((DstPtr as ULONG_PTR).wrapping_sub(Dst as ULONG_PTR)) as DWORD;

        i = 0;
        while i < (*Dst).NumStrings as DWORD {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(StringW), SrcPtr as PCWSTR);
            Status = RtlUnicodeStringToAnsiString(
                core::ptr::addr_of_mut!(StringA),
                core::ptr::addr_of_mut!(StringW) as PCUNICODE_STRING,
                TRUE as BOOLEAN,
            );
            if NT_SUCCESS!(Status) {
                RtlCopyMemory!(
                    DstPtr,
                    StringA.Buffer as *const core::ffi::c_void,
                    StringA.MaximumLength as usize,
                );
                DstPtr = ((DstPtr as ULONG_PTR).wrapping_add(StringA.MaximumLength as ULONG_PTR))
                    as PVOID;

                RtlFreeAnsiString(core::ptr::addr_of_mut!(StringA));
            } else {
                RtlZeroMemory!(
                    DstPtr,
                    (StringW.MaximumLength as usize) / core::mem::size_of::<WCHAR>(),
                );
                DstPtr = ((DstPtr as ULONG_PTR).wrapping_add(
                    (StringW.MaximumLength as ULONG_PTR)
                        / (core::mem::size_of::<WCHAR>() as ULONG_PTR),
                )) as PVOID;
            }
            SrcPtr = ((SrcPtr as ULONG_PTR).wrapping_add(StringW.MaximumLength as ULONG_PTR))
                as PVOID;
            i = i.wrapping_add(1);
        }

        /* Copy the binary data */
        SrcPtr = ((Src as ULONG_PTR).wrapping_add((*Src).DataOffset as ULONG_PTR)) as PVOID;
        (*Dst).DataOffset = ((DstPtr as ULONG_PTR).wrapping_sub(Dst as ULONG_PTR)) as DWORD;
        RtlCopyMemory!(
            DstPtr,
            SrcPtr as *const core::ffi::c_void,
            (*Src).DataLength as usize,
        );
        DstPtr = ((DstPtr as ULONG_PTR).wrapping_add((*Src).DataLength as ULONG_PTR)) as PVOID;

        /* Add the padding */
        dwPadding = (core::mem::size_of::<ULONG>() as ULONG_PTR).wrapping_sub(
            ((DstPtr as ULONG_PTR).wrapping_sub(Dst as ULONG_PTR))
                % (core::mem::size_of::<ULONG>() as ULONG_PTR),
        ) as DWORD;
        RtlZeroMemory!(DstPtr, dwPadding as usize);

        /* Set the record length at the beginning and the end of the record */
        dwRecordLength = ((DstPtr as ULONG_PTR)
            .wrapping_add(dwPadding as ULONG_PTR)
            .wrapping_add(core::mem::size_of::<ULONG>() as ULONG_PTR)
            .wrapping_sub(Dst as ULONG_PTR)) as DWORD;
        (*Dst).Length = dwRecordLength;
        pLength = ((DstPtr as ULONG_PTR).wrapping_add(dwPadding as ULONG_PTR)) as PDWORD;
        *pLength = dwRecordLength;

        if !BytesRead.is_null() {
            *BytesRead = dwRecordLength as ULONG_PTR;
        }

        Status = STATUS_SUCCESS;
    }

    LogfpFree(
        UnicodeBuffer as PVOID,
        0 as ULONG,
        TAG_ELF_BUF as ULONG,
    );

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn LogfReadEvents(LogFile: PLOGFILE, Flags: ULONG, RecordNumber: PULONG, BufSize: ULONG, Buffer: PBYTE, BytesRead: PULONG, BytesNeeded: PULONG, Ansi: BOOLEAN) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut RecNum: ULONG;
    let mut ReadLength: SIZE_T = core::mem::zeroed();
    let mut NeededSize: SIZE_T = core::mem::zeroed();
    let mut BufferUsage: ULONG;

    /* EVENTLOG_SEQUENTIAL_READ and EVENTLOG_SEEK_READ are mutually exclusive */
    if (Flags & EVENTLOG_SEQUENTIAL_READ) != 0 && (Flags & EVENTLOG_SEEK_READ) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if (Flags & EVENTLOG_SEQUENTIAL_READ) == 0 && (Flags & EVENTLOG_SEEK_READ) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    /* EVENTLOG_FORWARDS_READ and EVENTLOG_BACKWARDS_READ are mutually exclusive */
    if (Flags & EVENTLOG_FORWARDS_READ) != 0 && (Flags & EVENTLOG_BACKWARDS_READ) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if (Flags & EVENTLOG_FORWARDS_READ) == 0 && (Flags & EVENTLOG_BACKWARDS_READ) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if Buffer.is_null() || BytesRead.is_null() || BytesNeeded.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* In seek read mode, a record number of 0 is invalid */
    if (Flags & EVENTLOG_SEQUENTIAL_READ) == 0 && *RecordNumber == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    /* Lock the log file shared */
    RtlAcquireResourceShared(core::ptr::addr_of_mut!((*LogFile).Lock), (TRUE) as BOOLEAN);

    /*
     * In sequential read mode, a record number of 0 means we need
     * to determine where to start the read operation. Otherwise
     * we just use the provided record number.
     */
    if (Flags & EVENTLOG_SEQUENTIAL_READ) != 0 && *RecordNumber == 0 {
        if (Flags & EVENTLOG_FORWARDS_READ) != 0 {
            *RecordNumber = ElfGetOldestRecord(core::ptr::addr_of_mut!((*LogFile).LogFile));
        } else {
            *RecordNumber = ElfGetCurrentRecord(core::ptr::addr_of_mut!((*LogFile).LogFile))
                .wrapping_sub(1);
        }
    }

    RecNum = *RecordNumber;

    *BytesRead = 0;
    *BytesNeeded = 0;

    BufferUsage = 0;

    'quit: {
        loop {
            Status = ReadRecord(
                core::ptr::addr_of_mut!((*LogFile).LogFile),
                RecNum,
                Buffer.add(BufferUsage as usize) as PEVENTLOGRECORD,
                BufSize.wrapping_sub(BufferUsage) as SIZE_T,
                core::ptr::addr_of_mut!(ReadLength),
                core::ptr::addr_of_mut!(NeededSize),
                Ansi,
            );
            if Status == STATUS_NOT_FOUND {
                if BufferUsage == 0 {
                    Status = STATUS_END_OF_FILE;
                    break 'quit;
                } else {
                    break;
                }
            } else if Status == STATUS_BUFFER_TOO_SMALL {
                if BufferUsage == 0 {
                    *BytesNeeded = NeededSize as ULONG;
                    // Status = STATUS_BUFFER_TOO_SMALL;
                    break 'quit;
                } else {
                    break;
                }
            } else if !NT_SUCCESS!(Status) {
                DbgPrint(
                    b"(%s:%d) ElfReadRecord failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
                    b"base/services/eventlog/file.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    822 as ::core::ffi::c_int,
                    Status,
                );
                break 'quit;
            }

            /* Go to the next event record */
            /*
             * NOTE: This implicitly supposes that all the other record numbers
             * are consecutive (and do not jump than more than one unit); but if
             * it is not the case, then we would prefer here to call some
             * "get_next_record_number" function.
             */
            if (Flags & EVENTLOG_FORWARDS_READ) != 0 {
                RecNum = RecNum.wrapping_add(1);
            } else {
                RecNum = RecNum.wrapping_sub(1);
            }

            BufferUsage = (BufferUsage as SIZE_T).wrapping_add(ReadLength) as ULONG;
            if !(BufferUsage <= BufSize) {
                break;
            }
        }

        *BytesRead = BufferUsage;
        *RecordNumber = RecNum;

        Status = STATUS_SUCCESS;
    }

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*LogFile).Lock));

    if !NT_SUCCESS!(Status) {
        DbgPrint(
            b"(%s:%d) LogfReadEvents failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
            b"base/services/eventlog/file.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            852 as ::core::ffi::c_int,
            Status,
        );
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn LogfWriteRecord(mut LogFile: PLOGFILE, mut Record: PEVENTLOGRECORD, mut BufSize: SIZE_T) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();

    if Record.is_null() || BufSize < core::mem::size_of::<EVENTLOGRECORD>() as SIZE_T {
        return STATUS_INVALID_PARAMETER;
    }

    /* Lock the log file exclusive */
    RtlAcquireResourceExclusive(
        core::ptr::addr_of_mut!((*LogFile).Lock),
        TRUE as BOOLEAN,
    );

    /*
     * Retrieve the record written time now, that will also be compared
     * with the existing events timestamps in case the log is wrapping.
     */
    NtQuerySystemTime(core::ptr::addr_of_mut!(SystemTime));
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of_mut!(SystemTime),
        core::ptr::addr_of_mut!((*Record).TimeWritten),
    );

    Status = ElfWriteRecord(
        core::ptr::addr_of_mut!((*LogFile).LogFile),
        Record,
        BufSize as usize,
    );
    if Status == STATUS_LOG_FILE_FULL {
        /* The event log file is full, queue a message box for the user and exit */
        // TODO!
        if DbgPrint(
            b"(%s:%d) Log file `%S' is full!\n\0".as_ptr() as PCSTR,
            b"base/services/eventlog/file.c\0"
                .as_ptr() as *const core::ffi::c_char,
            885 as core::ffi::c_int,
            (*LogFile).LogName,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"base/services/eventlog/file.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                885 as core::ffi::c_int,
            );
        }
    }

    /* Unlock the log file */
    RtlReleaseResource(core::ptr::addr_of_mut!((*LogFile).Lock));

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn LogfAllocAndBuildNewRecord(pRecSize: PSIZE_T, Time: ULONG, wType: USHORT, wCategory: USHORT, dwEventId: ULONG, SourceName: PUNICODE_STRING, ComputerName: PUNICODE_STRING, dwSidLength: ULONG, pUserSid: PSID, wNumStrings: USHORT, pStrings: PWSTR, dwDataSize: ULONG, pRawData: PVOID) -> PEVENTLOGRECORD {
    let mut RecSize: SIZE_T = 0;
    let mut SourceNameSize: SIZE_T = 0;
    let mut ComputerNameSize: SIZE_T = 0;
    let mut StringLen: SIZE_T = 0;
    let mut Buffer: PBYTE = core::ptr::null_mut();
    let mut pRec: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut str: PWSTR = core::ptr::null_mut();
    let mut i: UINT = 0;
    let mut pos: UINT = 0;

    SourceNameSize = if !SourceName.is_null() && !(*SourceName).Buffer.is_null() {
        (*SourceName).Length as SIZE_T
    } else {
        0 as SIZE_T
    };
    ComputerNameSize = if !ComputerName.is_null() && !(*ComputerName).Buffer.is_null() {
        (*ComputerName).Length as SIZE_T
    } else {
        0 as SIZE_T
    };

    RecSize = (core::mem::size_of::<EVENTLOGRECORD>() as SIZE_T)
        + SourceNameSize
        + ComputerNameSize
        + (2 as SIZE_T) * (core::mem::size_of_val(&UNICODE_NULL) as SIZE_T);

    /* Align on DWORD boundary for the SID */
    RecSize = ROUND_UP!(RecSize, core::mem::size_of::<ULONG>() as SIZE_T);

    RecSize += dwSidLength as SIZE_T;

    /* Add the sizes for the strings array */

    i = 0 as UINT;
    str = pStrings;
    while i < wNumStrings as UINT {
        StringLen = (wcslen(str) as SIZE_T + 1 as SIZE_T) as SIZE_T; // str must be != NULL
        RecSize += StringLen * (core::mem::size_of::<WCHAR>() as SIZE_T);
        str = str.add(StringLen as usize);
        i += 1;
    }

    /* Add the data size */
    RecSize += dwDataSize as SIZE_T;

    /* Align on DWORD boundary for the full structure */
    RecSize = ROUND_UP!(RecSize, core::mem::size_of::<ULONG>() as SIZE_T);

    /* Size of the trailing 'Length' member */
    RecSize += core::mem::size_of::<ULONG>() as SIZE_T;

    Buffer = RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY as ULONG, RecSize) as PBYTE;
    if Buffer.is_null() {
        DbgPrint(b"Cannot allocate heap!\n\0".as_ptr() as PCSTR);
        return core::ptr::null_mut();
    }

    pRec = Buffer as PEVENTLOGRECORD;
    (*pRec).Length = RecSize as DWORD;
    (*pRec).Reserved = LOGFILE_SIGNATURE as DWORD;

    /*
     * Do not assign here any precomputed record number to the event record.
     * The true record number will be assigned atomically and sequentially in
     * LogfWriteRecord, so that all the event records will have consistent and
     * unique record numbers.
     */
    (*pRec).RecordNumber = 0 as DWORD;

    /*
     * Set the generated time, and temporarily set the written time
     * with the generated time.
     */
    (*pRec).TimeGenerated = Time as DWORD;
    (*pRec).TimeWritten = Time as DWORD;

    (*pRec).EventID = dwEventId as DWORD;
    (*pRec).EventType = wType as WORD;
    (*pRec).EventCategory = wCategory as WORD;

    pos = core::mem::size_of::<EVENTLOGRECORD>() as UINT;

    /* NOTE: Equivalents of RtlStringCbCopyUnicodeString calls */
    if SourceNameSize != 0 {
        StringCbCopyNW(
            Buffer.add(pos as usize) as PWSTR,
            (SourceNameSize + core::mem::size_of_val(&UNICODE_NULL) as SIZE_T) as _,
            (*SourceName).Buffer,
            (SourceNameSize) as _,
        );
    }
    pos = (pos as SIZE_T + SourceNameSize + core::mem::size_of_val(&UNICODE_NULL) as SIZE_T) as UINT;
    if ComputerNameSize != 0 {
        StringCbCopyNW(
            Buffer.add(pos as usize) as PWSTR,
            (ComputerNameSize + core::mem::size_of_val(&UNICODE_NULL) as SIZE_T) as _,
            (*ComputerName).Buffer,
            (ComputerNameSize) as _,
        );
    }
    pos = (pos as SIZE_T + ComputerNameSize + core::mem::size_of_val(&UNICODE_NULL) as SIZE_T) as UINT;

    /* Align on DWORD boundary for the SID */
    pos = ROUND_UP!(pos as SIZE_T, core::mem::size_of::<ULONG>() as SIZE_T) as UINT;

    (*pRec).UserSidLength = 0 as DWORD;
    (*pRec).UserSidOffset = 0 as DWORD;
    if dwSidLength != 0 {
        RtlCopyMemory!(
            Buffer.add(pos as usize) as PVOID,
            pUserSid as PVOID,
            dwSidLength as SIZE_T,
        );
        (*pRec).UserSidLength = dwSidLength as DWORD;
        (*pRec).UserSidOffset = pos as DWORD;
        pos = (pos as SIZE_T + dwSidLength as SIZE_T) as UINT;
    }

    (*pRec).StringOffset = pos as DWORD;
    i = 0 as UINT;
    str = pStrings;
    while i < wNumStrings as UINT {
        StringLen = (wcslen(str) as SIZE_T + 1 as SIZE_T) as SIZE_T; // str must be != NULL
        StringCchCopyW(
            Buffer.add(pos as usize) as PWSTR,
            StringLen as usize,
            str,
        );
        str = str.add(StringLen as usize);
        pos = (pos as SIZE_T + StringLen * (core::mem::size_of::<WCHAR>() as SIZE_T)) as UINT;
        i += 1;
    }
    (*pRec).NumStrings = wNumStrings as WORD;

    (*pRec).DataLength = 0 as DWORD;
    (*pRec).DataOffset = 0 as DWORD;
    if dwDataSize != 0 {
        RtlCopyMemory!(
            Buffer.add(pos as usize) as PVOID,
            pRawData,
            dwDataSize as SIZE_T,
        );
        (*pRec).DataLength = dwDataSize as DWORD;
        (*pRec).DataOffset = pos as DWORD;
        pos = (pos as SIZE_T + dwDataSize as SIZE_T) as UINT;
    }

    /* Align on DWORD boundary for the full structure */
    pos = ROUND_UP!(pos as SIZE_T, core::mem::size_of::<ULONG>() as SIZE_T) as UINT;

    /* Initialize the trailing 'Length' member */
    *(Buffer.add(pos as usize) as PDWORD) = RecSize as DWORD;

    *pRecSize = RecSize;
    return pRec;
}
#[no_mangle]
pub unsafe extern "C" fn LogfReportEvent(
    mut wType: USHORT,
    mut wCategory: USHORT,
    mut dwEventId: ULONG,
    mut wNumStrings: USHORT,
    mut pStrings: PWSTR,
    mut dwDataSize: ULONG,
    mut pRawData: PVOID,
) {
    let mut Status: NTSTATUS;
    let mut SourceName: _UNICODE_STRING = core::mem::zeroed();
    let mut ComputerName: _UNICODE_STRING = core::mem::zeroed();
    let mut LogBuffer: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Time: ULONG = 0;
    let mut RecSize: SIZE_T = 0;
    let mut dwComputerNameLength: DWORD;
    let mut szComputerName: [WCHAR; MAX_COMPUTERNAME_LENGTH as usize + 1] =
        [0; MAX_COMPUTERNAME_LENGTH as usize + 1];

    if EventLogSource.is_null() {
        return;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(SourceName),
        core::ptr::addr_of_mut!((*EventLogSource).szName) as *mut WCHAR as PCWSTR,
    );

    dwComputerNameLength = ARRAYSIZE!(szComputerName) as DWORD;
    if GetComputerNameW(
        szComputerName.as_mut_ptr(),
        core::ptr::addr_of_mut!(dwComputerNameLength),
    ) == 0
    {
        szComputerName[0 as usize] = 0x0000u16;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ComputerName),
        szComputerName.as_mut_ptr() as PCWSTR,
    );

    NtQuerySystemTime(core::ptr::addr_of_mut!(SystemTime));
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of_mut!(SystemTime),
        core::ptr::addr_of_mut!(Time),
    );

    LogBuffer = LogfAllocAndBuildNewRecord(
        core::ptr::addr_of_mut!(RecSize),
        Time,
        wType,
        wCategory,
        dwEventId,
        core::ptr::addr_of_mut!(SourceName),
        core::ptr::addr_of_mut!(ComputerName),
        0 as ULONG,
        core::ptr::null_mut(),
        wNumStrings,
        pStrings,
        dwDataSize,
        pRawData,
    );
    if LogBuffer.is_null() {
        DPRINT1!(b"LogfAllocAndBuildNewRecord failed!\n\0" as *const u8 as PCSTR);
        return;
    }

    Status = LogfWriteRecord((*EventLogSource).LogFile, LogBuffer, RecSize);
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"ERROR writing to event log `%S' (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
            (*(*EventLogSource).LogFile).LogName,
            Status,
        );
    }

    LogfFreeRecord(LogBuffer);
}
