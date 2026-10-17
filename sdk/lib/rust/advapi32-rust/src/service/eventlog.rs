use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::externs::HeapAlloc;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "system" {
    fn ElfrNumberOfRecords(hEventLog: HANDLE, NumberOfRecords: PULONG) -> NTSTATUS;
    fn SetLastError(dwErrCode: DWORD);
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> i32;
}

// Register/Report/Deregister used to go through service/eventlog_seh.c's native-SEH thunks,
// which are a pass-through on i386 (no native SEH there). csrss died of it: winsrv's
// UserServerHardError -> ReportEventW while the EventLog service was exiting raised
// RPC_S_CALL_FAILED (0x6be) through the unguarded stub, BUGCHECK 0xF4 (2026-10-17, i386).
// They now use rpc_try like every other Elfr* call here, which works on all three arches.

// ---- file-scope preamble (translated) ----
static mut EmptyStringU: RPC_UNICODE_STRING = RPC_UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: (&[0u16] as *const u16) as PWSTR,
};

static mut EmptyStringA: RPC_STRING = RPC_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: b"\0".as_ptr() as PSTR,
};
#[no_mangle]
pub unsafe extern "system" fn EVENTLOG_HANDLE_A_bind(UNCServerName: EVENTLOG_HANDLE_A) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: RPC_CSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS;

    TRACE!(b"EVENTLOG_HANDLE_A_bind() called\n\0".as_ptr());

    status = RpcStringBindingComposeA(
        core::ptr::null_mut(),
        b"ncacn_np\0".as_ptr() as RPC_CSTR,
        UNCServerName as RPC_CSTR,
        b"\\pipe\\EventLog\0".as_ptr() as RPC_CSTR,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        ERR!("RpcStringBindingCompose returned 0x%x\n", status);
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingA(pszStringBinding, &mut hBinding);
    if status != RPC_S_OK {
        ERR!("RpcBindingFromStringBinding returned 0x%x\n", status);
    }

    status = RpcStringFreeA(&mut pszStringBinding);
    if status != RPC_S_OK {
        ERR!("RpcStringFree returned 0x%x\n", status);
    }

    hBinding
}
#[no_mangle]
pub unsafe extern "system" fn EVENTLOG_HANDLE_A_unbind(UNCServerName: EVENTLOG_HANDLE_A, mut hBinding: handle_t) {
    let status: RPC_STATUS;

    status = RpcBindingFree(core::ptr::addr_of_mut!(hBinding));
    if status != RPC_S_OK {
    }
}
#[no_mangle]
pub unsafe extern "system" fn EVENTLOG_HANDLE_W_bind(UNCServerName: EVENTLOG_HANDLE_W) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: RPC_WSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS;

    TRACE!(b"EVENTLOG_HANDLE_W_bind() called\n\0".as_ptr());

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        w!("ncacn_np").as_ptr() as *mut u16,
        UNCServerName,
        w!("\\pipe\\EventLog").as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != RPC_S_OK {
        ERR!("RpcStringBindingCompose returned 0x%x\n", status);
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != RPC_S_OK {
        ERR!("RpcBindingFromStringBinding returned 0x%x\n", status);
    }

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != RPC_S_OK {
        ERR!("RpcStringFree returned 0x%x\n", status);
    }

    hBinding
}
#[no_mangle]
pub unsafe extern "system" fn EVENTLOG_HANDLE_W_unbind(
    UNCServerName: EVENTLOG_HANDLE_W,
    mut hBinding: handle_t,
) {
    let status: RPC_STATUS;

    status = RpcBindingFree(&mut hBinding);
    if status != RPC_S_OK {
    }
}

#[no_mangle]
pub unsafe extern "system" fn ElfBackupEventLogFileA(
    hEventLog: HANDLE,
    BackupFileNameA: PANSI_STRING,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if BackupFileNameA.is_null() || (*BackupFileNameA).Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| ElfrBackupELFA(hEventLog, BackupFileNameA as PRPC_STRING)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BackupEventLogA(
    hEventLog: HANDLE,
    lpBackupFileName: LPCSTR,
) -> BOOL {
    let Success: BOOL;
    let Status: NTSTATUS;
    let mut BackupFileNameA: ANSI_STRING = core::mem::zeroed();
    let mut BackupFileNameW: _UNICODE_STRING = core::mem::zeroed();

    if lpBackupFileName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    RtlInitAnsiString(&mut BackupFileNameA, lpBackupFileName as PSTR);

    Status = RtlAnsiStringToUnicodeString(&mut BackupFileNameW, &mut BackupFileNameA, 1u8);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    Success = BackupEventLogW(hEventLog, BackupFileNameW.Buffer);

    RtlFreeUnicodeString(&mut BackupFileNameW);

    Success
}

#[no_mangle]
pub unsafe extern "system" fn ElfBackupEventLogFileW(
    hEventLog: HANDLE,
    BackupFileNameU: PUNICODE_STRING,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if BackupFileNameU.is_null() || (*BackupFileNameU).Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| ElfrBackupELFW(hEventLog, BackupFileNameU as PRPC_UNICODE_STRING)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BackupEventLogW(
    hEventLog: HANDLE,
    lpBackupFileName: LPCWSTR,
) -> BOOL {
    let Status: NTSTATUS;
    let mut BackupFileName: _UNICODE_STRING = core::mem::zeroed();

    if lpBackupFileName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if RtlDosPathNameToNtPathName_U(
        lpBackupFileName,
        &mut BackupFileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Status = ElfBackupEventLogFileW(hEventLog, &mut BackupFileName);

    RtlFreeHeap(RtlGetProcessHeap(), 0, BackupFileName.Buffer as PVOID);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ElfClearEventLogFileA(
    hEventLog: HANDLE,
    BackupFileNameA: PANSI_STRING,
) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| ElfrClearELFA(hEventLog, BackupFileNameA as PRPC_STRING)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ClearEventLogA(
    hEventLog: HANDLE,
    lpBackupFileName: LPCSTR,
) -> BOOL {
    let mut BackupFileNameA: ANSI_STRING = core::mem::zeroed();
    let mut BackupFileNameW: _UNICODE_STRING = core::mem::zeroed();

    if lpBackupFileName.is_null() {
        RtlInitUnicodeString(&mut BackupFileNameW, core::ptr::null_mut());
    } else {
        RtlInitAnsiString(&mut BackupFileNameA, lpBackupFileName);

        let Status = RtlAnsiStringToUnicodeString(&mut BackupFileNameW, &mut BackupFileNameA, 1u8);
        if !NT_SUCCESS(Status) {
            SetLastError(RtlNtStatusToDosError(Status));
            return 0;
        }
    }

    let Success = ClearEventLogW(hEventLog, BackupFileNameW.Buffer);

    RtlFreeUnicodeString(&mut BackupFileNameW);

    Success
}

#[no_mangle]
pub unsafe extern "system" fn ElfClearEventLogFileW(
    hEventLog: HANDLE,
    BackupFileNameU: PUNICODE_STRING,
) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| ElfrClearELFW(hEventLog, BackupFileNameU as PRPC_UNICODE_STRING)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ClearEventLogW(
    hEventLog: HANDLE,
    lpBackupFileName: LPCWSTR,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut BackupFileName: _UNICODE_STRING = core::mem::zeroed();

    if lpBackupFileName.is_null() {
        RtlInitUnicodeString(&mut BackupFileName, core::ptr::null_mut());
    } else {
        if RtlDosPathNameToNtPathName_U(
            lpBackupFileName,
            &mut BackupFileName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    Status = ElfClearEventLogFileW(hEventLog, &mut BackupFileName);

    if !lpBackupFileName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, BackupFileName.Buffer as PVOID);
    }

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ElfCloseEventLog(mut hEventLog: HANDLE) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| ElfrCloseEL(core::ptr::addr_of_mut!(hEventLog))).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CloseEventLog(hEventLog: HANDLE) -> BOOL {
    let Status: NTSTATUS = ElfCloseEventLog(hEventLog);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn ElfDeregisterEventSource(hEventLog: HANDLE) -> NTSTATUS {
    let mut hEventLog = hEventLog;
    let Status: NTSTATUS;

    Status = rpc_try(|| advapi32_sys::ElfrDeregisterEventSource(&mut hEventLog))
        .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn DeregisterEventSource(hEventLog: HANDLE) -> BOOL {
    let Status: NTSTATUS = ElfDeregisterEventSource(hEventLog);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetEventLogInformation(
    hEventLog: HANDLE,
    dwInfoLevel: DWORD,
    lpBuffer: LPVOID,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPDWORD,
) -> BOOL {
    let Status: NTSTATUS;

    if dwInfoLevel != EVENTLOG_FULL_INFO {
        SetLastError(ERROR_INVALID_LEVEL);
        return 0;
    }

    Status = rpc_try(|| ElfrGetLogInformation(
        hEventLog,
        dwInfoLevel,
        lpBuffer as LPBYTE,
        cbBufSize,
        pcbBytesNeeded,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ElfNumberOfRecords(
    hEventLog: HANDLE,
    NumberOfRecords: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if NumberOfRecords.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| ElfrNumberOfRecords(hEventLog, NumberOfRecords)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn GetNumberOfEventLogRecords(
    hEventLog: HANDLE,
    NumberOfRecords: PDWORD,
) -> BOOL {
    let Status: NTSTATUS = ElfNumberOfRecords(hEventLog, NumberOfRecords);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn ElfOldestRecord(hEventLog: HANDLE, OldestRecordNumber: PULONG) -> NTSTATUS {
    let Status: NTSTATUS;

    if OldestRecordNumber.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| ElfrOldestRecord(hEventLog, OldestRecordNumber)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn GetOldestEventLogRecord(
    hEventLog: HANDLE,
    OldestRecord: PDWORD,
) -> BOOL {
    let Status: NTSTATUS = ElfOldestRecord(hEventLog, OldestRecord);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE as BOOL;
    }

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn ElfChangeNotify(hEventLog: HANDLE, hEvent: HANDLE) -> NTSTATUS {
    let Status: NTSTATUS;
    let ClientId: _CLIENT_ID = (*NtCurrentTeb()).ClientId;
    let mut RpcClientId: RPC_CLIENT_ID = core::mem::zeroed();

    RpcClientId.UniqueProcess = HandleToUlong(ClientId.UniqueProcess);
    RpcClientId.UniqueThread = HandleToUlong(ClientId.UniqueThread);

    Status = rpc_try(|| ElfrChangeNotify(hEventLog, RpcClientId, HandleToUlong(hEvent))).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NotifyChangeEventLog(hEventLog: HANDLE, hEvent: HANDLE) -> BOOL {
    let Status: NTSTATUS = ElfChangeNotify(hEventLog, hEvent);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE as BOOL;
    }

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn ElfOpenBackupEventLogA(
    UNCServerNameA: PANSI_STRING,
    BackupFileNameA: PANSI_STRING,
    phEventLog: PHANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut pUNCServerName: PSTR = core::ptr::null_mut();

    if phEventLog.is_null()
        || BackupFileNameA.is_null()
        || (*BackupFileNameA).Length == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    if !UNCServerNameA.is_null() && (*UNCServerNameA).Length != 0 {
        pUNCServerName = (*UNCServerNameA).Buffer as PSTR;
    }

    *phEventLog = core::ptr::null_mut();

    Status = rpc_try(|| ElfrOpenBELA(
        pUNCServerName,
        BackupFileNameA as PRPC_STRING,
        1,
        1,
        phEventLog as *mut *mut core::ffi::c_void,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn OpenBackupEventLogA(
    lpUNCServerName: LPCSTR,
    lpFileName: LPCSTR,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let LogHandle: HANDLE;
    let mut UNCServerNameA: ANSI_STRING = core::mem::zeroed();
    let mut UNCServerNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut FileNameA: ANSI_STRING = core::mem::zeroed();
    let mut FileNameW: _UNICODE_STRING = core::mem::zeroed();

    /* Convert the server name to unicode */
    if lpUNCServerName.is_null() {
        RtlInitUnicodeString(&mut UNCServerNameW, core::ptr::null_mut());
    } else {
        RtlInitAnsiString(&mut UNCServerNameA, lpUNCServerName);

        Status = RtlAnsiStringToUnicodeString(&mut UNCServerNameW, &mut UNCServerNameA, 1u8);
        if !NT_SUCCESS(Status) {
            SetLastError(RtlNtStatusToDosError(Status));
            return core::ptr::null_mut();
        }
    }

    /* Convert the file name to unicode */
    if lpFileName.is_null() {
        RtlInitUnicodeString(&mut FileNameW, core::ptr::null_mut());
    } else {
        RtlInitAnsiString(&mut FileNameA, lpFileName);

        Status = RtlAnsiStringToUnicodeString(&mut FileNameW, &mut FileNameA, 1u8);
        if !NT_SUCCESS(Status) {
            RtlFreeUnicodeString(&mut UNCServerNameW);
            SetLastError(RtlNtStatusToDosError(Status));
            return core::ptr::null_mut();
        }
    }

    /* Call the unicode function */
    LogHandle = OpenBackupEventLogW(UNCServerNameW.Buffer, FileNameW.Buffer);

    /* Free the unicode strings */
    RtlFreeUnicodeString(&mut UNCServerNameW);
    RtlFreeUnicodeString(&mut FileNameW);

    LogHandle
}

#[no_mangle]
pub unsafe extern "system" fn ElfOpenBackupEventLogW(
    UNCServerNameU: PUNICODE_STRING,
    BackupFileNameU: PUNICODE_STRING,
    phEventLog: PHANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut pUNCServerName: PWSTR = core::ptr::null_mut();

    if phEventLog.is_null()
        || BackupFileNameU.is_null()
        || (*BackupFileNameU).Length == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    if !UNCServerNameU.is_null() && (*UNCServerNameU).Length != 0 {
        pUNCServerName = (*UNCServerNameU).Buffer;
    }

    *phEventLog = core::ptr::null_mut();

    Status = rpc_try(|| ElfrOpenBELW(
        pUNCServerName,
        BackupFileNameU as *mut _,
        1,
        1,
        phEventLog as *mut _,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn OpenBackupEventLogW(
    lpUNCServerName: LPCWSTR,
    lpFileName: LPCWSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut UNCServerName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();

    if lpFileName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut FileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    RtlInitUnicodeString(&mut UNCServerName, lpUNCServerName);

    Status = ElfOpenBackupEventLogW(&mut UNCServerName, &mut FileName, &mut hEventLog);

    if !FileName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
    }

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return core::ptr::null_mut();
    }

    hEventLog
}

#[no_mangle]
pub unsafe extern "system" fn ElfOpenEventLogA(
    UNCServerNameA: PANSI_STRING,
    SourceNameA: PANSI_STRING,
    phEventLog: PHANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut pUNCServerName: PSTR = core::ptr::null_mut();

    if phEventLog.is_null() || SourceNameA.is_null() || (*SourceNameA).Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if !UNCServerNameA.is_null() && (*UNCServerNameA).Length != 0 {
        pUNCServerName = (*UNCServerNameA).Buffer;
    }

    *phEventLog = core::ptr::null_mut();

    Status = rpc_try(|| ElfrOpenELA(
        pUNCServerName,
        SourceNameA as PRPC_STRING,
        core::ptr::addr_of_mut!(EmptyStringA),
        1,
        1,
        phEventLog as *mut IELF_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn OpenEventLogA(
    lpUNCServerName: LPCSTR,
    lpSourceName: LPCSTR,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut UNCServerName: ANSI_STRING = core::mem::zeroed();
    let mut SourceName: ANSI_STRING = core::mem::zeroed();

    RtlInitAnsiString(&mut UNCServerName, lpUNCServerName as PSTR);
    RtlInitAnsiString(&mut SourceName, lpSourceName as PSTR);

    Status = ElfOpenEventLogA(&mut UNCServerName, &mut SourceName, &mut hEventLog);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return core::ptr::null_mut();
    }

    hEventLog
}

#[no_mangle]
pub unsafe extern "system" fn ElfOpenEventLogW(
    UNCServerNameU: PUNICODE_STRING,
    SourceNameU: PUNICODE_STRING,
    phEventLog: PHANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut pUNCServerName: PWSTR = core::ptr::null_mut();

    if phEventLog.is_null() || SourceNameU.is_null() || (*SourceNameU).Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if !UNCServerNameU.is_null() && (*UNCServerNameU).Length != 0 {
        pUNCServerName = (*UNCServerNameU).Buffer;
    }

    *phEventLog = core::ptr::null_mut();

    Status = rpc_try(|| ElfrOpenELW(
        pUNCServerName,
        SourceNameU as PRPC_UNICODE_STRING,
        core::ptr::addr_of_mut!(EmptyStringU),
        1,
        1,
        phEventLog as *mut *mut core::ffi::c_void,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn OpenEventLogW(
    lpUNCServerName: LPCWSTR,
    lpSourceName: LPCWSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut UNCServerName: _UNICODE_STRING = core::mem::zeroed();
    let mut SourceName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut UNCServerName, lpUNCServerName as PWSTR);
    RtlInitUnicodeString(&mut SourceName, lpSourceName as PWSTR);

    Status = ElfOpenEventLogW(&mut UNCServerName, &mut SourceName, &mut hEventLog);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return core::ptr::null_mut();
    }

    hEventLog
}

#[no_mangle]
pub unsafe extern "system" fn ElfReadEventLogA(
    hEventLog: HANDLE,
    ReadFlags: ULONG,
    RecordOffset: ULONG,
    Buffer: LPVOID,
    NumberOfBytesToRead: ULONG,
    NumberOfBytesRead: PULONG,
    MinNumberOfBytesNeeded: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Flags: ULONG;

    if Buffer.is_null() || NumberOfBytesRead.is_null() || MinNumberOfBytesNeeded.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Flags = ReadFlags & (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ);
    if Flags == (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ) {
        return STATUS_INVALID_PARAMETER;
    }

    Flags = ReadFlags & (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ);
    if Flags == (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ) {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| ElfrReadELA(
        hEventLog,
        ReadFlags,
        RecordOffset,
        NumberOfBytesToRead,
        Buffer as *mut u8,
        NumberOfBytesRead,
        MinNumberOfBytesNeeded,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ReadEventLogA(
    hEventLog: HANDLE,
    dwReadFlags: DWORD,
    dwRecordOffset: DWORD,
    lpBuffer: LPVOID,
    nNumberOfBytesToRead: DWORD,
    pnBytesRead: *mut DWORD,
    pnMinNumberOfBytesNeeded: *mut DWORD,
) -> BOOL {
    let Status: NTSTATUS;

    Status = ElfReadEventLogA(
        hEventLog,
        dwReadFlags,
        dwRecordOffset,
        lpBuffer,
        nNumberOfBytesToRead,
        pnBytesRead,
        pnMinNumberOfBytesNeeded,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ElfReadEventLogW(
    hEventLog: HANDLE,
    ReadFlags: ULONG,
    RecordOffset: ULONG,
    Buffer: LPVOID,
    NumberOfBytesToRead: ULONG,
    NumberOfBytesRead: PULONG,
    MinNumberOfBytesNeeded: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Flags: ULONG;

    if Buffer.is_null() || NumberOfBytesRead.is_null() || MinNumberOfBytesNeeded.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Flags = ReadFlags & (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ);
    if Flags == (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ) {
        return STATUS_INVALID_PARAMETER;
    }

    Flags = ReadFlags & (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ);
    if Flags == (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ) {
        return STATUS_INVALID_PARAMETER;
    }

    Status = rpc_try(|| ElfrReadELW(
        hEventLog,
        ReadFlags,
        RecordOffset,
        NumberOfBytesToRead,
        Buffer as *mut u8,
        NumberOfBytesRead,
        MinNumberOfBytesNeeded,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ReadEventLogW(
    hEventLog: HANDLE,
    dwReadFlags: DWORD,
    dwRecordOffset: DWORD,
    lpBuffer: LPVOID,
    nNumberOfBytesToRead: DWORD,
    pnBytesRead: *mut DWORD,
    pnMinNumberOfBytesNeeded: *mut DWORD,
) -> BOOL {
    let Status: NTSTATUS;

    Status = ElfReadEventLogW(
        hEventLog,
        dwReadFlags,
        dwRecordOffset,
        lpBuffer,
        nNumberOfBytesToRead,
        pnBytesRead,
        pnMinNumberOfBytesNeeded,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ElfRegisterEventSourceA(
    UNCServerNameA: PANSI_STRING,
    SourceNameA: PANSI_STRING,
    phEventLog: PHANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut pUNCServerName: PSTR = core::ptr::null_mut();

    if phEventLog.is_null() || SourceNameA.is_null() || (*SourceNameA).Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if !UNCServerNameA.is_null() && (*UNCServerNameA).Length != 0 {
        pUNCServerName = (*UNCServerNameA).Buffer;
    }

    *phEventLog = core::ptr::null_mut();

    Status = rpc_try(|| ElfrRegisterEventSourceA(
        pUNCServerName,
        SourceNameA as PRPC_STRING,
        core::ptr::addr_of_mut!(EmptyStringA),
        1,
        1,
        phEventLog as *mut IELF_HANDLE,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RegisterEventSourceA(
    lpUNCServerName: LPCSTR,
    lpSourceName: LPCSTR,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut UNCServerName: ANSI_STRING = core::mem::zeroed();
    let mut SourceName: ANSI_STRING = core::mem::zeroed();

    TRACE!(b"%s, %s\n\0".as_ptr() as LPCSTR, lpUNCServerName, lpSourceName);

    RtlInitAnsiString(&mut UNCServerName, lpUNCServerName as PSTR);
    RtlInitAnsiString(&mut SourceName, lpSourceName as PSTR);

    Status = ElfRegisterEventSourceA(&mut UNCServerName, &mut SourceName, &mut hEventLog);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return core::ptr::null_mut();
    }

    hEventLog
}

#[no_mangle]
pub unsafe extern "system" fn ElfRegisterEventSourceW(
    UNCServerNameU: PUNICODE_STRING,
    SourceNameU: PUNICODE_STRING,
    phEventLog: PHANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut pUNCServerName: PWSTR = core::ptr::null_mut();

    if phEventLog.is_null() || SourceNameU.is_null() || (*SourceNameU).Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if !UNCServerNameU.is_null() && (*UNCServerNameU).Length != 0 {
        pUNCServerName = (*UNCServerNameU).Buffer;
    }

    *phEventLog = core::ptr::null_mut();

    Status = rpc_try(|| {
        advapi32_sys::ElfrRegisterEventSourceW(
            pUNCServerName,
            SourceNameU as PRPC_UNICODE_STRING,
            core::ptr::addr_of_mut!(EmptyStringU),
            1,
            1,
            phEventLog as *mut IELF_HANDLE,
        )
    })
    .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RegisterEventSourceW(
    lpUNCServerName: LPCWSTR,
    lpSourceName: LPCWSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut UNCServerName: _UNICODE_STRING = core::mem::zeroed();
    let mut SourceName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut UNCServerName, lpUNCServerName as PWSTR);
    RtlInitUnicodeString(&mut SourceName, lpSourceName as PWSTR);

    Status = ElfRegisterEventSourceW(&mut UNCServerName, &mut SourceName, &mut hEventLog);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return core::ptr::null_mut();
    }

    hEventLog
}

#[no_mangle]
pub unsafe extern "system" fn ElfReportEventA(
    hEventLog: HANDLE,
    EventType: USHORT,
    EventCategory: USHORT,
    EventID: ULONG,
    UserSID: PSID,
    NumStrings: USHORT,
    DataSize: ULONG,
    Strings: *mut PANSI_STRING,
    Data: PVOID,
    Flags: USHORT,
    RecordNumber: PULONG,
    TimeWritten: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Time: ULONG = 0;
    let mut dwSize: ULONG;
    let mut ComputerName: ANSI_STRING = core::mem::zeroed();
    let mut szComputerName: [CHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] =
        [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];

    dwSize = szComputerName.len() as ULONG;
    GetComputerNameA(szComputerName.as_mut_ptr(), &mut dwSize);
    RtlInitAnsiString(&mut ComputerName, szComputerName.as_ptr());

    NtQuerySystemTime(&mut SystemTime);
    RtlTimeToSecondsSince1970(&mut SystemTime, &mut Time);

    Status = rpc_try(|| ElfrReportEventA(
        hEventLog,
        Time,
        EventType,
        EventCategory,
        EventID,
        NumStrings,
        DataSize,
        core::ptr::addr_of_mut!(ComputerName) as PRPC_STRING,
        UserSID as PRPC_SID,
        Strings as *mut PRPC_STRING,
        Data as *mut u8,
        Flags,
        RecordNumber,
        TimeWritten,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ReportEventA(
    hEventLog: HANDLE,
    wType: WORD,
    wCategory: WORD,
    dwEventID: DWORD,
    lpUserSid: PSID,
    wNumStrings: WORD,
    dwDataSize: DWORD,
    lpStrings: *mut LPCSTR,
    lpRawData: LPVOID,
) -> BOOL {

    let Status: NTSTATUS;
    let mut i: WORD;

    let Strings: *mut PANSI_STRING = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((wNumStrings as usize) * core::mem::size_of::<PANSI_STRING>()) as SIZE_T,
    ) as *mut PANSI_STRING;

    if Strings.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    i = 0;
    while i < wNumStrings {
        *Strings.add(i as usize) = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (core::mem::size_of::<ANSI_STRING>()) as SIZE_T,
        ) as PANSI_STRING;
        if !(*Strings.add(i as usize)).is_null() {
            RtlInitAnsiString(*Strings.add(i as usize), *lpStrings.add(i as usize) as PSTR);
        }
        i += 1;
    }

    Status = ElfReportEventA(
        hEventLog,
        wType,
        wCategory,
        dwEventID,
        lpUserSid,
        wNumStrings,
        dwDataSize,
        Strings,
        lpRawData,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    i = 0;
    while i < wNumStrings {
        if !(*Strings.add(i as usize)).is_null() {
            HeapFree(GetProcessHeap(), 0, *Strings.add(i as usize) as LPVOID);
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, Strings as LPVOID);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ElfReportEventW(
    hEventLog: HANDLE,
    EventType: USHORT,
    EventCategory: USHORT,
    EventID: ULONG,
    UserSID: PSID,
    NumStrings: USHORT,
    DataSize: ULONG,
    Strings: *mut PUNICODE_STRING,
    Data: PVOID,
    Flags: USHORT,
    RecordNumber: PULONG,
    TimeWritten: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Time: ULONG = 0;
    let mut dwSize: ULONG;
    let mut ComputerName: _UNICODE_STRING = core::mem::zeroed();
    let mut szComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] =
        [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];

    dwSize = szComputerName.len() as ULONG;
    GetComputerNameW(szComputerName.as_mut_ptr(), &mut dwSize);
    RtlInitUnicodeString(&mut ComputerName, szComputerName.as_mut_ptr());

    NtQuerySystemTime(&mut SystemTime);
    RtlTimeToSecondsSince1970(&mut SystemTime, &mut Time);

    Status = rpc_try(|| {
        advapi32_sys::ElfrReportEventW(
            hEventLog,
            Time,
            EventType,
            EventCategory,
            EventID,
            NumStrings,
            DataSize,
            &mut ComputerName as *mut _UNICODE_STRING as PRPC_UNICODE_STRING,
            UserSID as PRPC_SID,
            Strings as *mut PRPC_UNICODE_STRING,
            Data as *mut u8,
            Flags,
            RecordNumber,
            TimeWritten,
        )
    })
    .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ReportEventW(
    hEventLog: HANDLE,
    wType: WORD,
    wCategory: WORD,
    dwEventID: DWORD,
    lpUserSid: PSID,
    wNumStrings: WORD,
    dwDataSize: DWORD,
    lpStrings: *mut LPCWSTR,
    lpRawData: LPVOID,
) -> BOOL {
    let Status: NTSTATUS;
    let Strings: *mut PUNICODE_STRING;
    let mut i: WORD;

    TRACE!(
        b"%p, %u, %u, %lu, %p, %u, %lu, %p, %p\n\0".as_ptr() as LPCSTR,
        hEventLog,
        wType as u32,
        wCategory as u32,
        dwEventID,
        lpUserSid,
        wNumStrings as u32,
        dwDataSize,
        lpStrings,
        lpRawData,
    );

    Strings = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((wNumStrings as usize) * core::mem::size_of::<PUNICODE_STRING>()) as SIZE_T,
    ) as *mut PUNICODE_STRING;
    if Strings.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    i = 0;
    while i < wNumStrings {
        *Strings.add(i as usize) = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (core::mem::size_of::<_UNICODE_STRING>()) as SIZE_T,
        ) as PUNICODE_STRING;
        if !(*Strings.add(i as usize)).is_null() {
            RtlInitUnicodeString(*Strings.add(i as usize), *lpStrings.add(i as usize) as PWSTR);
        }
        i += 1;
    }

    Status = ElfReportEventW(
        hEventLog,
        wType,
        wCategory,
        dwEventID,
        lpUserSid,
        wNumStrings,
        dwDataSize,
        Strings,
        lpRawData,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    i = 0;
    while i < wNumStrings {
        if !(*Strings.add(i as usize)).is_null() {
            HeapFree(GetProcessHeap(), 0, *Strings.add(i as usize) as LPVOID);
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, Strings as LPVOID);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ElfReportEventAndSourceW(
    hEventLog: HANDLE,
    Time: ULONG,
    ComputerName: PUNICODE_STRING,
    EventType: USHORT,
    EventCategory: USHORT,
    EventID: ULONG,
    UserSID: PSID,
    SourceName: PUNICODE_STRING,
    NumStrings: USHORT,
    DataSize: ULONG,
    Strings: *mut PUNICODE_STRING,
    Data: PVOID,
    Flags: USHORT,
    RecordNumber: PULONG,
    TimeWritten: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| ElfrReportEventAndSourceW(
        hEventLog,
        Time,
        EventType,
        EventCategory,
        EventID,
        SourceName as PRPC_UNICODE_STRING,
        NumStrings,
        DataSize,
        ComputerName as PRPC_UNICODE_STRING,
        UserSID as PRPC_SID,
        Strings as *mut PRPC_UNICODE_STRING,
        Data as PBYTE,
        Flags,
        RecordNumber,
        TimeWritten,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ElfFlushEventLog(hEventLog: HANDLE) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = rpc_try(|| ElfrFlushEL(hEventLog)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Win32 advapi functions
//  *
//  * Copyright 1995 Sven Verdoolaege
//  * Copyright 1998 Juergen Schmied
//  * Copyright 2003 Mike Hearn
//  * Copyright 2007 Hervé Poussineau
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <advapi32.h>
//
// #include <ndk/kefuncs.h>
// #include <eventlogrpc_c.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// static RPC_UNICODE_STRING EmptyStringU = { 0, 0, L"" };
// static RPC_STRING EmptyStringA = { 0, 0, "" };
//
//
// /* FUNCTIONS *****************************************************************/
//
// handle_t __RPC_USER
// EVENTLOG_HANDLE_A_bind(EVENTLOG_HANDLE_A UNCServerName)
// {
//     handle_t hBinding = NULL;
//     RPC_CSTR pszStringBinding;
//     RPC_STATUS status;
//
//     TRACE("EVENTLOG_HANDLE_A_bind() called\n");
//
//     status = RpcStringBindingComposeA(NULL,
//                                       (RPC_CSTR)"ncacn_np",
//                                       (RPC_CSTR)UNCServerName,
//                                       (RPC_CSTR)"\\pipe\\EventLog",
//                                       NULL,
//                                       &pszStringBinding);
//     if (status)
//     {
//         ERR("RpcStringBindingCompose returned 0x%x\n", status);
//         return NULL;
//     }
//
//     /* Set the binding handle that will be used to bind to the server. */
//     status = RpcBindingFromStringBindingA(pszStringBinding,
//                                           &hBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFromStringBinding returned 0x%x\n", status);
//     }
//
//     status = RpcStringFreeA(&pszStringBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcStringFree returned 0x%x\n", status);
//     }
//
//     return hBinding;
// }
//
//
// void __RPC_USER
// EVENTLOG_HANDLE_A_unbind(EVENTLOG_HANDLE_A UNCServerName,
//                          handle_t hBinding)
// {
//     RPC_STATUS status;
//
//     TRACE("EVENTLOG_HANDLE_A_unbind() called\n");
//
//     status = RpcBindingFree(&hBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFree returned 0x%x\n", status);
//     }
// }
//
//
// handle_t __RPC_USER
// EVENTLOG_HANDLE_W_bind(EVENTLOG_HANDLE_W UNCServerName)
// {
//     handle_t hBinding = NULL;
//     RPC_WSTR pszStringBinding;
//     RPC_STATUS status;
//
//     TRACE("EVENTLOG_HANDLE_W_bind() called\n");
//
//     status = RpcStringBindingComposeW(NULL,
//                                       L"ncacn_np",
//                                       UNCServerName,
//                                       L"\\pipe\\EventLog",
//                                       NULL,
//                                       &pszStringBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcStringBindingCompose returned 0x%x\n", status);
//         return NULL;
//     }
//
//     /* Set the binding handle that will be used to bind to the server. */
//     status = RpcBindingFromStringBindingW(pszStringBinding,
//                                           &hBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFromStringBinding returned 0x%x\n", status);
//     }
//
//     status = RpcStringFreeW(&pszStringBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcStringFree returned 0x%x\n", status);
//     }
//
//     return hBinding;
// }
//
//
// void __RPC_USER
// EVENTLOG_HANDLE_W_unbind(EVENTLOG_HANDLE_W UNCServerName,
//                          handle_t hBinding)
// {
//     RPC_STATUS status;
//
//     TRACE("EVENTLOG_HANDLE_W_unbind() called\n");
//
//     status = RpcBindingFree(&hBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFree returned 0x%x\n", status);
//     }
// }
//
//
// /******************************************************************************
//  * BackupEventLogA [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfBackupEventLogFileA(IN HANDLE hEventLog,
//                        IN PANSI_STRING BackupFileNameA)
// {
//     NTSTATUS Status;
//
//     if (!BackupFileNameA || (BackupFileNameA->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         Status = ElfrBackupELFA(hEventLog,
//                                 (PRPC_STRING)BackupFileNameA);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// BackupEventLogA(IN HANDLE hEventLog,
//                 IN LPCSTR lpBackupFileName)
// {
//     BOOL Success;
//     NTSTATUS Status;
//     ANSI_STRING BackupFileNameA;
//     UNICODE_STRING BackupFileNameW;
//
//     TRACE("%p, %s\n", hEventLog, lpBackupFileName);
//
//     if (lpBackupFileName == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlInitAnsiString(&BackupFileNameA, lpBackupFileName);
//
//     Status = RtlAnsiStringToUnicodeString(&BackupFileNameW,
//                                           &BackupFileNameA,
//                                           TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     Success = BackupEventLogW(hEventLog,
//                               BackupFileNameW.Buffer);
//
//     RtlFreeUnicodeString(&BackupFileNameW);
//
//     return Success;
// }
//
//
// /******************************************************************************
//  * BackupEventLogW [ADVAPI32.@]
//  *
//  * PARAMS
//  *   hEventLog        []
//  *   lpBackupFileName []
//  */
// NTSTATUS
// NTAPI
// ElfBackupEventLogFileW(IN HANDLE hEventLog,
//                        IN PUNICODE_STRING BackupFileNameU)
// {
//     NTSTATUS Status;
//
//     if (!BackupFileNameU || (BackupFileNameU->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         Status = ElfrBackupELFW(hEventLog,
//                                 (PRPC_UNICODE_STRING)BackupFileNameU);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// BackupEventLogW(IN HANDLE hEventLog,
//                 IN LPCWSTR lpBackupFileName)
// {
//     NTSTATUS Status;
//     UNICODE_STRING BackupFileName;
//
//     TRACE("%p, %s\n", hEventLog, debugstr_w(lpBackupFileName));
//
//     if (lpBackupFileName == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (!RtlDosPathNameToNtPathName_U(lpBackupFileName, &BackupFileName,
//                                       NULL, NULL))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Status = ElfBackupEventLogFileW(hEventLog, &BackupFileName);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, BackupFileName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * ClearEventLogA [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfClearEventLogFileA(IN HANDLE hEventLog,
//                       IN PANSI_STRING BackupFileNameA)
// {
//     NTSTATUS Status;
//
//     RpcTryExcept
//     {
//         Status = ElfrClearELFA(hEventLog,
//                                (PRPC_STRING)BackupFileNameA);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// ClearEventLogA(IN HANDLE hEventLog,
//                IN LPCSTR lpBackupFileName)
// {
//     BOOL Success;
//     NTSTATUS Status;
//     ANSI_STRING BackupFileNameA;
//     UNICODE_STRING BackupFileNameW;
//
//     TRACE("%p, %s\n", hEventLog, lpBackupFileName);
//
//     if (lpBackupFileName == NULL)
//     {
//         RtlInitUnicodeString(&BackupFileNameW, NULL);
//     }
//     else
//     {
//         RtlInitAnsiString(&BackupFileNameA, lpBackupFileName);
//
//         Status = RtlAnsiStringToUnicodeString(&BackupFileNameW,
//                                               &BackupFileNameA,
//                                               TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//     }
//
//     Success = ClearEventLogW(hEventLog,
//                              BackupFileNameW.Buffer);
//
//     RtlFreeUnicodeString(&BackupFileNameW);
//
//     return Success;
// }
//
//
// /******************************************************************************
//  * ClearEventLogW [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfClearEventLogFileW(IN HANDLE hEventLog,
//                       IN PUNICODE_STRING BackupFileNameU)
// {
//     NTSTATUS Status;
//
//     RpcTryExcept
//     {
//         Status = ElfrClearELFW(hEventLog,
//                                (PRPC_UNICODE_STRING)BackupFileNameU);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// ClearEventLogW(IN HANDLE hEventLog,
//                IN LPCWSTR lpBackupFileName)
// {
//     NTSTATUS Status;
//     UNICODE_STRING BackupFileName;
//
//     TRACE("%p, %s\n", hEventLog, debugstr_w(lpBackupFileName));
//
//     if (lpBackupFileName == NULL)
//     {
//         RtlInitUnicodeString(&BackupFileName, NULL);
//     }
//     else
//     {
//         if (!RtlDosPathNameToNtPathName_U(lpBackupFileName, &BackupFileName,
//                                           NULL, NULL))
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//         }
//     }
//
//     Status = ElfClearEventLogFileW(hEventLog, &BackupFileName);
//
//     if (lpBackupFileName != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, BackupFileName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * CloseEventLog [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfCloseEventLog(IN HANDLE hEventLog)
// {
//     NTSTATUS Status;
//
//     RpcTryExcept
//     {
//         Status = ElfrCloseEL(&hEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// CloseEventLog(IN HANDLE hEventLog)
// {
//     NTSTATUS Status;
//
//     TRACE("%p\n", hEventLog);
//
//     Status = ElfCloseEventLog(hEventLog);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * DeregisterEventSource [ADVAPI32.@]
//  * Closes a handle to the specified event log
//  *
//  * PARAMS
//  *    hEventLog [I] Handle to event log
//  *
//  * RETURNS STD
//  */
// NTSTATUS
// NTAPI
// ElfDeregisterEventSource(IN HANDLE hEventLog)
// {
//     NTSTATUS Status;
//
//     RpcTryExcept
//     {
//         Status = ElfrDeregisterEventSource(&hEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// DeregisterEventSource(IN HANDLE hEventLog)
// {
//     NTSTATUS Status;
//
//     TRACE("%p\n", hEventLog);
//
//     Status = ElfDeregisterEventSource(hEventLog);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * GetEventLogInformation [ADVAPI32.@]
//  *
//  * PARAMS
//  *   hEventLog      [I] Handle to event log
//  *   dwInfoLevel    [I] Level of event log information to return
//  *   lpBuffer       [O] Buffer that receives the event log information
//  *   cbBufSize      [I] Size of the lpBuffer buffer
//  *   pcbBytesNeeded [O] Required buffer size
//  */
// BOOL WINAPI
// GetEventLogInformation(IN HANDLE hEventLog,
//                        IN DWORD dwInfoLevel,
//                        OUT LPVOID lpBuffer,
//                        IN DWORD cbBufSize,
//                        OUT LPDWORD pcbBytesNeeded)
// {
//     NTSTATUS Status;
//
//     if (dwInfoLevel != EVENTLOG_FULL_INFO)
//     {
//         SetLastError(ERROR_INVALID_LEVEL);
//         return FALSE;
//     }
//
//     RpcTryExcept
//     {
//         Status = ElfrGetLogInformation(hEventLog,
//                                        dwInfoLevel,
//                                        (LPBYTE)lpBuffer,
//                                        cbBufSize,
//                                        pcbBytesNeeded);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * GetNumberOfEventLogRecords [ADVAPI32.@]
//  *
//  * PARAMS
//  *   hEventLog       []
//  *   NumberOfRecords []
//  */
// NTSTATUS
// NTAPI
// ElfNumberOfRecords(IN HANDLE hEventLog,
//                    OUT PULONG NumberOfRecords)
// {
//     NTSTATUS Status;
//
//     if (!NumberOfRecords)
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         Status = ElfrNumberOfRecords(hEventLog, NumberOfRecords);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// GetNumberOfEventLogRecords(IN HANDLE hEventLog,
//                            OUT PDWORD NumberOfRecords)
// {
//     NTSTATUS Status;
//
//     TRACE("%p, %p\n", hEventLog, NumberOfRecords);
//
//     Status = ElfNumberOfRecords(hEventLog, NumberOfRecords);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * GetOldestEventLogRecord [ADVAPI32.@]
//  *
//  * PARAMS
//  *   hEventLog    []
//  *   OldestRecord []
//  */
// NTSTATUS
// NTAPI
// ElfOldestRecord(IN HANDLE hEventLog,
//                 OUT PULONG OldestRecordNumber)
// {
//     NTSTATUS Status;
//
//     if (!OldestRecordNumber)
//         return STATUS_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         Status = ElfrOldestRecord(hEventLog, OldestRecordNumber);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// GetOldestEventLogRecord(IN HANDLE hEventLog,
//                         OUT PDWORD OldestRecord)
// {
//     NTSTATUS Status;
//
//     TRACE("%p, %p\n", hEventLog, OldestRecord);
//
//     Status = ElfOldestRecord(hEventLog, OldestRecord);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * NotifyChangeEventLog [ADVAPI32.@]
//  *
//  * PARAMS
//  *   hEventLog []
//  *   hEvent    []
//  */
// NTSTATUS
// NTAPI
// ElfChangeNotify(IN HANDLE hEventLog,
//                 IN HANDLE hEvent)
// {
//     NTSTATUS Status;
//     CLIENT_ID ClientId = NtCurrentTeb()->ClientId;
//     RPC_CLIENT_ID RpcClientId;
//
//     RpcClientId.UniqueProcess = HandleToUlong(ClientId.UniqueProcess);
//     RpcClientId.UniqueThread  = HandleToUlong(ClientId.UniqueThread);
//
//     RpcTryExcept
//     {
//         Status = ElfrChangeNotify(hEventLog, RpcClientId, HandleToUlong(hEvent));
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// NotifyChangeEventLog(IN HANDLE hEventLog,
//                      IN HANDLE hEvent)
// {
//     NTSTATUS Status;
//
//     TRACE("%p, %p\n", hEventLog, hEvent);
//
//     Status = ElfChangeNotify(hEventLog, hEvent);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * OpenBackupEventLogA [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfOpenBackupEventLogA(IN PANSI_STRING UNCServerNameA,
//                        IN PANSI_STRING BackupFileNameA,
//                        OUT PHANDLE phEventLog)
// {
//     NTSTATUS Status;
//     PSTR pUNCServerName = NULL;
//
//     if (!phEventLog || !BackupFileNameA || (BackupFileNameA->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     if (UNCServerNameA && (UNCServerNameA->Length != 0))
//         pUNCServerName = UNCServerNameA->Buffer;
//
//     *phEventLog = NULL;
//
//     RpcTryExcept
//     {
//         Status = ElfrOpenBELA(pUNCServerName,
//                               (PRPC_STRING)BackupFileNameA,
//                               1, 1,
//                               (IELF_HANDLE*)phEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// HANDLE WINAPI
// OpenBackupEventLogA(IN LPCSTR lpUNCServerName,
//                     IN LPCSTR lpFileName)
// {
//     NTSTATUS Status;
//     HANDLE LogHandle;
//     ANSI_STRING UNCServerNameA;
//     UNICODE_STRING UNCServerNameW;
//     ANSI_STRING FileNameA;
//     UNICODE_STRING FileNameW;
//
//     TRACE("%s, %s\n", lpUNCServerName, lpFileName);
//
//     /* Convert the server name to unicode */
//     if (lpUNCServerName == NULL)
//     {
//         RtlInitUnicodeString(&UNCServerNameW, NULL);
//     }
//     else
//     {
//         RtlInitAnsiString(&UNCServerNameA, lpUNCServerName);
//
//         Status = RtlAnsiStringToUnicodeString(&UNCServerNameW,
//                                               &UNCServerNameA,
//                                               TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastError(RtlNtStatusToDosError(Status));
//             return NULL;
//         }
//     }
//
//     /* Convert the file name to unicode */
//     if (lpFileName == NULL)
//     {
//         RtlInitUnicodeString(&FileNameW, NULL);
//     }
//     else
//     {
//         RtlInitAnsiString(&FileNameA, lpFileName);
//
//         Status = RtlAnsiStringToUnicodeString(&FileNameW,
//                                               &FileNameA,
//                                               TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             RtlFreeUnicodeString(&UNCServerNameW);
//             SetLastError(RtlNtStatusToDosError(Status));
//             return NULL;
//         }
//     }
//
//     /* Call the unicode function */
//     LogHandle = OpenBackupEventLogW(UNCServerNameW.Buffer,
//                                     FileNameW.Buffer);
//
//     /* Free the unicode strings */
//     RtlFreeUnicodeString(&UNCServerNameW);
//     RtlFreeUnicodeString(&FileNameW);
//
//     return LogHandle;
// }
//
//
// /******************************************************************************
//  * OpenBackupEventLogW [ADVAPI32.@]
//  *
//  * PARAMS
//  *   lpUNCServerName []
//  *   lpFileName      []
//  */
// NTSTATUS
// NTAPI
// ElfOpenBackupEventLogW(IN PUNICODE_STRING UNCServerNameU,
//                        IN PUNICODE_STRING BackupFileNameU,
//                        OUT PHANDLE phEventLog)
// {
//     NTSTATUS Status;
//     PWSTR pUNCServerName = NULL;
//
//     if (!phEventLog || !BackupFileNameU || (BackupFileNameU->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     if (UNCServerNameU && (UNCServerNameU->Length != 0))
//         pUNCServerName = UNCServerNameU->Buffer;
//
//     *phEventLog = NULL;
//
//     RpcTryExcept
//     {
//         Status = ElfrOpenBELW(pUNCServerName,
//                               (PRPC_UNICODE_STRING)BackupFileNameU,
//                               1,
//                               1,
//                               (IELF_HANDLE*)phEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// HANDLE WINAPI
// OpenBackupEventLogW(IN LPCWSTR lpUNCServerName,
//                     IN LPCWSTR lpFileName)
// {
//     NTSTATUS Status;
//     HANDLE hEventLog;
//     UNICODE_STRING UNCServerName, FileName;
//
//     TRACE("%s, %s\n", debugstr_w(lpUNCServerName), debugstr_w(lpFileName));
//
//     if (lpFileName == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     if (!RtlDosPathNameToNtPathName_U(lpFileName, &FileName,
//                                       NULL, NULL))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     RtlInitUnicodeString(&UNCServerName, lpUNCServerName);
//
//     Status = ElfOpenBackupEventLogW(&UNCServerName, &FileName, &hEventLog);
//
//     if (FileName.Buffer != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return NULL;
//     }
//
//     return hEventLog;
// }
//
//
// /******************************************************************************
//  * OpenEventLogA [ADVAPI32.@]
//  *
//  * Opens a handle to the specified event log.
//  *
//  * PARAMS
//  *  lpUNCServerName [I] UNC name of the server on which the event log is
//  *                      opened.
//  *  lpSourceName    [I] Name of the log.
//  *
//  * RETURNS
//  *  Success: Handle to an event log.
//  *  Failure: NULL
//  */
// NTSTATUS
// NTAPI
// ElfOpenEventLogA(IN PANSI_STRING UNCServerNameA,
//                  IN PANSI_STRING SourceNameA,
//                  OUT PHANDLE phEventLog)
// {
//     NTSTATUS Status;
//     PSTR pUNCServerName = NULL;
//
//     if (!phEventLog || !SourceNameA || (SourceNameA->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     if (UNCServerNameA && (UNCServerNameA->Length != 0))
//         pUNCServerName = UNCServerNameA->Buffer;
//
//     *phEventLog = NULL;
//
//     RpcTryExcept
//     {
//         Status = ElfrOpenELA(pUNCServerName,
//                              (PRPC_STRING)SourceNameA,
//                              &EmptyStringA,
//                              1,
//                              1,
//                              (IELF_HANDLE*)phEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// HANDLE WINAPI
// OpenEventLogA(IN LPCSTR lpUNCServerName,
//               IN LPCSTR lpSourceName)
// {
//     NTSTATUS Status;
//     HANDLE hEventLog;
//     ANSI_STRING UNCServerName, SourceName;
//
//     TRACE("%s, %s\n", lpUNCServerName, lpSourceName);
//
//     RtlInitAnsiString(&UNCServerName, lpUNCServerName);
//     RtlInitAnsiString(&SourceName, lpSourceName);
//
//     Status = ElfOpenEventLogA(&UNCServerName, &SourceName, &hEventLog);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return NULL;
//     }
//
//     return hEventLog;
// }
//
//
// /******************************************************************************
//  * OpenEventLogW [ADVAPI32.@]
//  *
//  * PARAMS
//  *   lpUNCServerName []
//  *   lpSourceName    []
//  */
// NTSTATUS
// NTAPI
// ElfOpenEventLogW(IN PUNICODE_STRING UNCServerNameU,
//                  IN PUNICODE_STRING SourceNameU,
//                  OUT PHANDLE phEventLog)
// {
//     NTSTATUS Status;
//     PWSTR pUNCServerName = NULL;
//
//     if (!phEventLog || !SourceNameU || (SourceNameU->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     if (UNCServerNameU && (UNCServerNameU->Length != 0))
//         pUNCServerName = UNCServerNameU->Buffer;
//
//     *phEventLog = NULL;
//
//     RpcTryExcept
//     {
//         Status = ElfrOpenELW(pUNCServerName,
//                              (PRPC_UNICODE_STRING)SourceNameU,
//                              &EmptyStringU,
//                              1,
//                              1,
//                              (IELF_HANDLE*)phEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// HANDLE WINAPI
// OpenEventLogW(IN LPCWSTR lpUNCServerName,
//               IN LPCWSTR lpSourceName)
// {
//     NTSTATUS Status;
//     HANDLE hEventLog;
//     UNICODE_STRING UNCServerName, SourceName;
//
//     TRACE("%s, %s\n", debugstr_w(lpUNCServerName), debugstr_w(lpSourceName));
//
//     RtlInitUnicodeString(&UNCServerName, lpUNCServerName);
//     RtlInitUnicodeString(&SourceName, lpSourceName);
//
//     Status = ElfOpenEventLogW(&UNCServerName, &SourceName, &hEventLog);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return NULL;
//     }
//
//     return hEventLog;
// }
//
//
// /******************************************************************************
//  * ReadEventLogA [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfReadEventLogA(IN HANDLE hEventLog,
//                  IN ULONG ReadFlags,
//                  IN ULONG RecordOffset,
//                  OUT LPVOID Buffer,
//                  IN ULONG NumberOfBytesToRead,
//                  OUT PULONG NumberOfBytesRead,
//                  OUT PULONG MinNumberOfBytesNeeded)
// {
//     NTSTATUS Status;
//     ULONG Flags;
//
//     if (!Buffer || !NumberOfBytesRead || !MinNumberOfBytesNeeded)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Flags = ReadFlags & (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ);
//     if (Flags == (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Flags = ReadFlags & (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ);
//     if (Flags == (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     RpcTryExcept
//     {
//         Status = ElfrReadELA(hEventLog,
//                              ReadFlags,
//                              RecordOffset,
//                              NumberOfBytesToRead,
//                              Buffer,
//                              NumberOfBytesRead,
//                              MinNumberOfBytesNeeded);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// ReadEventLogA(IN HANDLE hEventLog,
//               IN DWORD dwReadFlags,
//               IN DWORD dwRecordOffset,
//               OUT LPVOID lpBuffer,
//               IN DWORD nNumberOfBytesToRead,
//               OUT DWORD *pnBytesRead,
//               OUT DWORD *pnMinNumberOfBytesNeeded)
// {
//     NTSTATUS Status;
//
//     TRACE("%p, %lu, %lu, %p, %lu, %p, %p\n",
//         hEventLog, dwReadFlags, dwRecordOffset, lpBuffer,
//         nNumberOfBytesToRead, pnBytesRead, pnMinNumberOfBytesNeeded);
//
//     Status = ElfReadEventLogA(hEventLog,
//                               dwReadFlags,
//                               dwRecordOffset,
//                               lpBuffer,
//                               nNumberOfBytesToRead,
//                               pnBytesRead,
//                               pnMinNumberOfBytesNeeded);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * ReadEventLogW [ADVAPI32.@]
//  *
//  * PARAMS
//  *   hEventLog                []
//  *   dwReadFlags              []
//  *   dwRecordOffset           []
//  *   lpBuffer                 []
//  *   nNumberOfBytesToRead     []
//  *   pnBytesRead              []
//  *   pnMinNumberOfBytesNeeded []
//  */
// NTSTATUS
// NTAPI
// ElfReadEventLogW(IN HANDLE hEventLog,
//                  IN ULONG ReadFlags,
//                  IN ULONG RecordOffset,
//                  OUT LPVOID Buffer,
//                  IN ULONG NumberOfBytesToRead,
//                  OUT PULONG NumberOfBytesRead,
//                  OUT PULONG MinNumberOfBytesNeeded)
// {
//     NTSTATUS Status;
//     ULONG Flags;
//
//     if (!Buffer || !NumberOfBytesRead || !MinNumberOfBytesNeeded)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Flags = ReadFlags & (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ);
//     if (Flags == (EVENTLOG_SEQUENTIAL_READ | EVENTLOG_SEEK_READ))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Flags = ReadFlags & (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ);
//     if (Flags == (EVENTLOG_FORWARDS_READ | EVENTLOG_BACKWARDS_READ))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     RpcTryExcept
//     {
//         Status = ElfrReadELW(hEventLog,
//                              ReadFlags,
//                              RecordOffset,
//                              NumberOfBytesToRead,
//                              Buffer,
//                              NumberOfBytesRead,
//                              MinNumberOfBytesNeeded);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// ReadEventLogW(IN HANDLE hEventLog,
//               IN DWORD dwReadFlags,
//               IN DWORD dwRecordOffset,
//               OUT LPVOID lpBuffer,
//               IN DWORD nNumberOfBytesToRead,
//               OUT DWORD *pnBytesRead,
//               OUT DWORD *pnMinNumberOfBytesNeeded)
// {
//     NTSTATUS Status;
//
//     TRACE("%p, %lu, %lu, %p, %lu, %p, %p\n",
//         hEventLog, dwReadFlags, dwRecordOffset, lpBuffer,
//         nNumberOfBytesToRead, pnBytesRead, pnMinNumberOfBytesNeeded);
//
//     Status = ElfReadEventLogW(hEventLog,
//                               dwReadFlags,
//                               dwRecordOffset,
//                               lpBuffer,
//                               nNumberOfBytesToRead,
//                               pnBytesRead,
//                               pnMinNumberOfBytesNeeded);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * RegisterEventSourceA [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfRegisterEventSourceA(IN PANSI_STRING UNCServerNameA,
//                         IN PANSI_STRING SourceNameA,
//                         OUT PHANDLE phEventLog)
// {
//     NTSTATUS Status;
//     PSTR pUNCServerName = NULL;
//
//     if (!phEventLog || !SourceNameA || (SourceNameA->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     if (UNCServerNameA && (UNCServerNameA->Length != 0))
//         pUNCServerName = UNCServerNameA->Buffer;
//
//     *phEventLog = NULL;
//
//     RpcTryExcept
//     {
//         Status = ElfrRegisterEventSourceA(pUNCServerName,
//                                           (PRPC_STRING)SourceNameA,
//                                           &EmptyStringA,
//                                           1,
//                                           1,
//                                           (IELF_HANDLE*)phEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// HANDLE WINAPI
// RegisterEventSourceA(IN LPCSTR lpUNCServerName,
//                      IN LPCSTR lpSourceName)
// {
//     NTSTATUS Status;
//     HANDLE hEventLog;
//     ANSI_STRING UNCServerName, SourceName;
//
//     TRACE("%s, %s\n", lpUNCServerName, lpSourceName);
//
//     RtlInitAnsiString(&UNCServerName, lpUNCServerName);
//     RtlInitAnsiString(&SourceName, lpSourceName);
//
//     Status = ElfRegisterEventSourceA(&UNCServerName, &SourceName, &hEventLog);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return NULL;
//     }
//
//     return hEventLog;
// }
//
//
// /******************************************************************************
//  * RegisterEventSourceW [ADVAPI32.@]
//  * Returns a registered handle to an event log
//  *
//  * PARAMS
//  *   lpUNCServerName [I] Server name for source
//  *   lpSourceName    [I] Source name for registered handle
//  *
//  * RETURNS
//  *    Success: Handle
//  *    Failure: NULL
//  */
// NTSTATUS
// NTAPI
// ElfRegisterEventSourceW(IN PUNICODE_STRING UNCServerNameU,
//                         IN PUNICODE_STRING SourceNameU,
//                         OUT PHANDLE phEventLog)
// {
//     NTSTATUS Status;
//     PWSTR pUNCServerName = NULL;
//
//     if (!phEventLog || !SourceNameU || (SourceNameU->Length == 0))
//         return STATUS_INVALID_PARAMETER;
//
//     if (UNCServerNameU && (UNCServerNameU->Length != 0))
//         pUNCServerName = UNCServerNameU->Buffer;
//
//     *phEventLog = NULL;
//
//     RpcTryExcept
//     {
//         Status = ElfrRegisterEventSourceW(pUNCServerName,
//                                           (PRPC_UNICODE_STRING)SourceNameU,
//                                           &EmptyStringU,
//                                           1,
//                                           1,
//                                           (IELF_HANDLE*)phEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// HANDLE WINAPI
// RegisterEventSourceW(IN LPCWSTR lpUNCServerName,
//                      IN LPCWSTR lpSourceName)
// {
//     NTSTATUS Status;
//     HANDLE hEventLog;
//     UNICODE_STRING UNCServerName, SourceName;
//
//     TRACE("%s, %s\n", debugstr_w(lpUNCServerName), debugstr_w(lpSourceName));
//
//     RtlInitUnicodeString(&UNCServerName, lpUNCServerName);
//     RtlInitUnicodeString(&SourceName, lpSourceName);
//
//     Status = ElfRegisterEventSourceW(&UNCServerName, &SourceName, &hEventLog);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return NULL;
//     }
//
//     return hEventLog;
// }
//
//
// /******************************************************************************
//  * ReportEventA [ADVAPI32.@]
//  */
// NTSTATUS
// NTAPI
// ElfReportEventA(IN HANDLE hEventLog,
//                 IN USHORT EventType,
//                 IN USHORT EventCategory,
//                 IN ULONG EventID,
//                 IN PSID UserSID,
//                 IN USHORT NumStrings,
//                 IN ULONG DataSize,
//                 IN PANSI_STRING* Strings,
//                 IN PVOID Data,
//                 IN USHORT Flags,
//                 IN OUT PULONG RecordNumber,
//                 IN OUT PULONG TimeWritten)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER SystemTime;
//     ULONG Time;
//     ULONG dwSize;
//     ANSI_STRING ComputerName;
//     CHAR szComputerName[MAX_COMPUTERNAME_LENGTH + 1];
//
//     dwSize = ARRAYSIZE(szComputerName);
//     GetComputerNameA(szComputerName, &dwSize);
//     RtlInitAnsiString(&ComputerName, szComputerName);
//
//     NtQuerySystemTime(&SystemTime);
//     RtlTimeToSecondsSince1970(&SystemTime, &Time);
//
//     RpcTryExcept
//     {
//         Status = ElfrReportEventA(hEventLog,
//                                   Time,
//                                   EventType,
//                                   EventCategory,
//                                   EventID,
//                                   NumStrings,
//                                   DataSize,
//                                   (PRPC_STRING)&ComputerName,
//                                   (PRPC_SID)UserSID,
//                                   (PRPC_STRING*)Strings,
//                                   Data,
//                                   Flags,
//                                   RecordNumber,
//                                   TimeWritten);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// ReportEventA(IN HANDLE hEventLog,
//              IN WORD wType,
//              IN WORD wCategory,
//              IN DWORD dwEventID,
//              IN PSID lpUserSid,
//              IN WORD wNumStrings,
//              IN DWORD dwDataSize,
//              IN LPCSTR *lpStrings,
//              IN LPVOID lpRawData)
// {
//     NTSTATUS Status;
//     PANSI_STRING *Strings;
//     WORD i;
//
//     TRACE("%p, %u, %u, %lu, %p, %u, %lu, %p, %p\n",
//           hEventLog, wType, wCategory, dwEventID, lpUserSid,
//           wNumStrings, dwDataSize, lpStrings, lpRawData);
//
//     Strings = HeapAlloc(GetProcessHeap(),
//                         HEAP_ZERO_MEMORY,
//                         wNumStrings * sizeof(PANSI_STRING));
//     if (!Strings)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     for (i = 0; i < wNumStrings; i++)
//     {
//         Strings[i] = HeapAlloc(GetProcessHeap(),
//                                HEAP_ZERO_MEMORY,
//                                sizeof(ANSI_STRING));
//         if (Strings[i])
//         {
//             RtlInitAnsiString(Strings[i], lpStrings[i]);
//         }
//     }
//
//     Status = ElfReportEventA(hEventLog,
//                              wType,
//                              wCategory,
//                              dwEventID,
//                              lpUserSid,
//                              wNumStrings,
//                              dwDataSize,
//                              Strings,
//                              lpRawData,
//                              0,
//                              NULL,
//                              NULL);
//
//     for (i = 0; i < wNumStrings; i++)
//     {
//         if (Strings[i] != NULL)
//             HeapFree(GetProcessHeap(), 0, Strings[i]);
//     }
//
//     HeapFree(GetProcessHeap(), 0, Strings);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * ReportEventW [ADVAPI32.@]
//  *
//  * PARAMS
//  *   hEventLog   []
//  *   wType       []
//  *   wCategory   []
//  *   dwEventID   []
//  *   lpUserSid   []
//  *   wNumStrings []
//  *   dwDataSize  []
//  *   lpStrings   []
//  *   lpRawData   []
//  */
// NTSTATUS
// NTAPI
// ElfReportEventW(IN HANDLE hEventLog,
//                 IN USHORT EventType,
//                 IN USHORT EventCategory,
//                 IN ULONG EventID,
//                 IN PSID UserSID,
//                 IN USHORT NumStrings,
//                 IN ULONG DataSize,
//                 IN PUNICODE_STRING* Strings,
//                 IN PVOID Data,
//                 IN USHORT Flags,
//                 IN OUT PULONG RecordNumber,
//                 IN OUT PULONG TimeWritten)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER SystemTime;
//     ULONG Time;
//     ULONG dwSize;
//     UNICODE_STRING ComputerName;
//     WCHAR szComputerName[MAX_COMPUTERNAME_LENGTH + 1];
//
//     dwSize = ARRAYSIZE(szComputerName);
//     GetComputerNameW(szComputerName, &dwSize);
//     RtlInitUnicodeString(&ComputerName, szComputerName);
//
//     NtQuerySystemTime(&SystemTime);
//     RtlTimeToSecondsSince1970(&SystemTime, &Time);
//
//     RpcTryExcept
//     {
//         Status = ElfrReportEventW(hEventLog,
//                                   Time,
//                                   EventType,
//                                   EventCategory,
//                                   EventID,
//                                   NumStrings,
//                                   DataSize,
//                                   (PRPC_UNICODE_STRING)&ComputerName,
//                                   (PRPC_SID)UserSID,
//                                   (PRPC_UNICODE_STRING*)Strings,
//                                   Data,
//                                   Flags,
//                                   RecordNumber,
//                                   TimeWritten);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// BOOL WINAPI
// ReportEventW(IN HANDLE hEventLog,
//              IN WORD wType,
//              IN WORD wCategory,
//              IN DWORD dwEventID,
//              IN PSID lpUserSid,
//              IN WORD wNumStrings,
//              IN DWORD dwDataSize,
//              IN LPCWSTR *lpStrings,
//              IN LPVOID lpRawData)
// {
//     NTSTATUS Status;
//     PUNICODE_STRING *Strings;
//     WORD i;
//
//     TRACE("%p, %u, %u, %lu, %p, %u, %lu, %p, %p\n",
//           hEventLog, wType, wCategory, dwEventID, lpUserSid,
//           wNumStrings, dwDataSize, lpStrings, lpRawData);
//
//     Strings = HeapAlloc(GetProcessHeap(),
//                         HEAP_ZERO_MEMORY,
//                         wNumStrings * sizeof(PUNICODE_STRING));
//     if (!Strings)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     for (i = 0; i < wNumStrings; i++)
//     {
//         Strings[i] = HeapAlloc(GetProcessHeap(),
//                                HEAP_ZERO_MEMORY,
//                                sizeof(UNICODE_STRING));
//         if (Strings[i])
//         {
//             RtlInitUnicodeString(Strings[i], lpStrings[i]);
//         }
//     }
//
//     Status = ElfReportEventW(hEventLog,
//                              wType,
//                              wCategory,
//                              dwEventID,
//                              lpUserSid,
//                              wNumStrings,
//                              dwDataSize,
//                              Strings,
//                              lpRawData,
//                              0,
//                              NULL,
//                              NULL);
//
//     for (i = 0; i < wNumStrings; i++)
//     {
//         if (Strings[i] != NULL)
//             HeapFree(GetProcessHeap(), 0, Strings[i]);
//     }
//
//     HeapFree(GetProcessHeap(), 0, Strings);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// NTSTATUS
// NTAPI
// ElfReportEventAndSourceW(IN HANDLE hEventLog,
//                          IN ULONG Time,
//                          IN PUNICODE_STRING ComputerName,
//                          IN USHORT EventType,
//                          IN USHORT EventCategory,
//                          IN ULONG EventID,
//                          IN PSID UserSID,
//                          IN PUNICODE_STRING SourceName,
//                          IN USHORT NumStrings,
//                          IN ULONG DataSize,
//                          IN PUNICODE_STRING* Strings,
//                          IN PVOID Data,
//                          IN USHORT Flags,
//                          IN OUT PULONG RecordNumber,
//                          IN OUT PULONG TimeWritten)
// {
//     NTSTATUS Status;
//
//     RpcTryExcept
//     {
//         Status = ElfrReportEventAndSourceW(hEventLog,
//                                            Time,
//                                            EventType,
//                                            EventCategory,
//                                            EventID,
//                                            (PRPC_UNICODE_STRING)SourceName,
//                                            NumStrings,
//                                            DataSize,
//                                            (PRPC_UNICODE_STRING)ComputerName,
//                                            (PRPC_SID)UserSID,
//                                            (PRPC_UNICODE_STRING*)Strings,
//                                            (PBYTE)Data,
//                                            Flags,
//                                            RecordNumber,
//                                            TimeWritten);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ElfFlushEventLog(IN HANDLE hEventLog)
// {
//     NTSTATUS Status;
//
//     RpcTryExcept
//     {
//         Status = ElfrFlushEL(hEventLog);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = I_RpcMapWin32Status(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
