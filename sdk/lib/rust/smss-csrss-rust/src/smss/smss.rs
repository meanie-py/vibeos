use ntdll_sys::nt::*;

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


const STATUS_WAIT_0: NTSTATUS = 0x00000000;
const STATUS_WAIT_1: NTSTATUS = 0x00000001;

// RtlGetProcessHeap is a TEB-inlined macro (NtCurrentTeb()->ProcessEnvironmentBlock->ProcessHeap).
#[inline(always)]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

// NtCurrentProcess is a pseudo-handle macro `(HANDLE)-1`.
#[inline(always)]
unsafe fn NtCurrentProcess() -> HANDLE {
    -1isize as HANDLE
}

// Sm* functions in sminit.c are NTAPI (stdcall on i386). The data
// globals are convention-free.
extern "C" {
    static mut SmpDefaultLibPath: _UNICODE_STRING;
    static mut SmpDefaultEnvironment: PVOID;
}
extern "system" {
    fn SmpSetProcessMuSessionId(ProcessHandle: HANDLE, MuSessionId: ULONG) -> NTSTATUS;
    fn SmpQueryRegistrySosOption() -> BOOLEAN;
    fn SmpSaveAndClearBootStatusData(BootOkay: *mut BOOLEAN, ShutdownOkay: *mut BOOLEAN)
        -> BOOLEAN;
    fn SmpRestoreBootStatusData(BootOkay: BOOLEAN, ShutdownOkay: BOOLEAN);
    fn SmpParseCommandLine(
        CommandLine: PUNICODE_STRING,
        Flags: *mut ULONG,
        FileName: *mut _UNICODE_STRING,
        Directory: *mut _UNICODE_STRING,
        Arguments: *mut _UNICODE_STRING,
    ) -> NTSTATUS;
    fn SmpLoadSubSystem(
        FileName: PUNICODE_STRING,
        Directory: PUNICODE_STRING,
        CommandLine: PUNICODE_STRING,
        MuSessionId: ULONG,
        ProcessId: PHANDLE,
        Flags: ULONG,
    ) -> NTSTATUS;
    fn SmConnectToSm(
        SbApiPortName: PUNICODE_STRING,
        SbApiPort: HANDLE,
        ImageType: ULONG,
        SmApiPort: PHANDLE,
    ) -> NTSTATUS;
    fn SmExecPgm(
        SmApiPort: HANDLE,
        ProcessInfo: PRTL_USER_PROCESS_INFORMATION,
        DebugFlag: BOOLEAN,
    ) -> NTSTATUS;
    fn SmpInit(InitialCommand: *mut _UNICODE_STRING, Handles: PHANDLE) -> NTSTATUS;
    fn SmpAcquirePrivilege(Privilege: ULONG, State: *mut PVOID) -> NTSTATUS;
    fn SmpReleasePrivilege(State: PVOID);
}
// DbgPrint is a real ntdll cdecl variadic.
extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> ULONG;
}

// RtlInitEmptyUnicodeString is FORCEINLINE in ndk/rtlfuncs.h.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(s: *mut _UNICODE_STRING, buf: *mut u16, len: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = len;
    (*s).Buffer = buf;
}
// SEH stubs (DUMMY_PSEH — dead code at runtime, just need to typecheck).
const EXCEPTION_EXECUTE_HANDLER: LONG = 1;

#[no_mangle]
pub static mut SmpSystemRoot: _UNICODE_STRING = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut AttachedSessionId: ULONG = 0xFFFFFFFF;

#[no_mangle]
pub static mut SmpDebug: BOOLEAN = 0;

#[no_mangle]
pub static mut SmpEnableDots: BOOLEAN = 0;

#[no_mangle]
pub static mut SmApiPort: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut SmpInitialCommandProcessId: HANDLE = core::ptr::null_mut();

// SMP_*_FLAG come from `ntdll_sys::nt::*` (bindgen of base/system/smss/smss.h):
// DEBUG=0x01 ASYNC=0x02 AUTOCHK=0x04 SUBSYSTEM=0x08 INVALID_PATH=0x10
// DEFERRED=0x20. The earlier local re-definitions here had SCRAMBLED values
// (SUBSYSTEM=0x02, ASYNC=0x04, AUTOCHK=0x10, …) that DISAGREED with the correct
// bindgen values used by smsubsys.rs/smloop.rs — so SmpLoadSubSystemsForMuSession
// passed SUBSYSTEM=0x08 while SmpExecuteCommand tested &0x02, the bit never
// matched, SmpLoadSubSystem was never called, csrss never launched and smss
// terminated → BUGCHECK 0xc000021a (task #18). Use the shared correct values.

const IMAGE_SUBSYSTEM_NATIVE: u16 = 1;
const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
// ndk/rtltypes.h:46,55. (These were 0x20000/0x100000: "RESERVE_1MB" was really
// setting the NX bit and "NX" an undefined bit, so subsystems never got the low 1MB.)
const RTL_USER_PROCESS_PARAMETERS_RESERVE_1MB: ULONG = 0x00000020;
const RTL_USER_PROCESS_PARAMETERS_NX: ULONG = 0x00020000;
const PROCESS_ALL_ACCESS: ULONG = 0x001F0FFF;
const SE_SHUTDOWN_PRIVILEGE: ULONG = 19;
const SE_LOAD_DRIVER_PRIVILEGE: ULONG = 10;
const STATUS_NO_TOKEN: NTSTATUS = 0xC000007C_u32 as NTSTATUS;
const STATUS_SYSTEM_PROCESS_TERMINATED: NTSTATUS = 0xC000021A_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000007B_u32 as NTSTATUS;
const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034_u32 as NTSTATUS;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001_u32 as NTSTATUS;
const FLG_DEBUG_INITIAL_COMMAND: ULONG = 0x00000004;
const FLG_DEBUG_INITIAL_COMMAND_EX: ULONG = 0x04000000;

#[no_mangle]
pub unsafe extern "system" fn SmpExecuteImage(
    FileName: PUNICODE_STRING,
    Directory: PUNICODE_STRING,
    CommandLine: PUNICODE_STRING,
    MuSessionId: ULONG,
    Flags: ULONG,
    ProcessInformation: PRTL_USER_PROCESS_INFORMATION,
) -> NTSTATUS {
    let ProcessInfo: PRTL_USER_PROCESS_INFORMATION;
    let mut LocalProcessInfo: RTL_USER_PROCESS_INFORMATION = core::mem::zeroed();
    let mut ProcessParameters: PRTL_USER_PROCESS_PARAMETERS = core::ptr::null_mut();

    ProcessInfo = if !ProcessInformation.is_null() {
        ProcessInformation
    } else {
        &mut LocalProcessInfo
    };

    let mut Status = RtlCreateProcessParameters(
        &mut ProcessParameters,
        FileName,
        if (*core::ptr::addr_of!(SmpDefaultLibPath)).Length != 0 {
            core::ptr::addr_of_mut!(SmpDefaultLibPath)
        } else {
            core::ptr::null_mut()
        },
        Directory,
        CommandLine,
        SmpDefaultEnvironment as PWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if Status < 0 {
        return Status;
    }

    (*ProcessInfo).Size = core::mem::size_of::<RTL_USER_PROCESS_INFORMATION>() as ULONG;

    if Flags & SMP_DEBUG_FLAG != 0 {
        (*ProcessParameters).DebugFlags = 1;
    } else {
        (*ProcessParameters).DebugFlags = SmpDebug as ULONG;
    }

    if Flags & SMP_SUBSYSTEM_FLAG != 0 {
        (*ProcessParameters).Flags |= RTL_USER_PROCESS_PARAMETERS_RESERVE_1MB;
    }

    (*ProcessParameters).Flags |= RTL_USER_PROCESS_PARAMETERS_NX;

    Status = RtlCreateUserProcess(
        FileName,
        OBJ_CASE_INSENSITIVE,
        ProcessParameters,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0u8,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ProcessInfo,
    );

    RtlDestroyProcessParameters(ProcessParameters);

    if Status < 0 {
        return Status;
    }

    Status = SmpSetProcessMuSessionId((*ProcessInfo).ProcessHandle, MuSessionId);

    if Flags & SMP_DEFERRED_FLAG != 0 {
        return Status;
    }

    if (*ProcessInfo).ImageInformation.SubSystemType == IMAGE_SUBSYSTEM_NATIVE.into() {
        NtResumeThread((*ProcessInfo).ThreadHandle, core::ptr::null_mut());
        if Flags & SMP_ASYNC_FLAG == 0 {
            NtWaitForSingleObject((*ProcessInfo).ThreadHandle, 0u8, core::ptr::null_mut());
        }

        NtClose((*ProcessInfo).ThreadHandle);
        NtClose((*ProcessInfo).ProcessHandle);
    } else {
        Status = STATUS_INVALID_IMAGE_FORMAT;
        NtTerminateProcess((*ProcessInfo).ProcessHandle, Status);
        NtWaitForSingleObject((*ProcessInfo).ThreadHandle, 0u8, core::ptr::null_mut());
        NtClose((*ProcessInfo).ThreadHandle);
        NtClose((*ProcessInfo).ProcessHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpInvokeAutoChk(
    FileName: PUNICODE_STRING,
    Directory: PUNICODE_STRING,
    Arguments: PUNICODE_STRING,
    Flags: ULONG,
) -> NTSTATUS {
    // The C's `CHAR MessageBuffer[256]` bounds the not-found message.
    const MESSAGE_CCH: usize = 256;
    let mut Destination: _UNICODE_STRING = core::mem::zeroed();
    let mut Buffer: [u16; 1024] = [0; 1024];
    let mut BootOkay: BOOLEAN = 0;
    let mut ShutdownOkay: BOOLEAN = 0;

    if SmpQueryRegistrySosOption() != 0 {
        SmpEnableDots = 0;
    }

    if Flags & SMP_INVALID_PATH != 0 {
        // Original C: RtlStringCbPrintfA(MessageBuffer, sizeof(MessageBuffer),
        // "%wZ program not found - skipping AUTOCHECK\r\n", FileName), then
        // ANSI -> Unicode for NtDisplayString. ntstrsafe's printf is a header
        // inline with no export, so build the identical Unicode text directly:
        // the file name followed by the fixed tail, bounded the same way
        // (sizeof(MessageBuffer) characters).
        RtlInitEmptyUnicodeString(
            &mut Destination,
            Buffer.as_mut_ptr(),
            (MESSAGE_CCH * core::mem::size_of::<u16>()) as u16,
        );
        RtlAppendUnicodeStringToString(&mut Destination, FileName);
        RtlAppendUnicodeToString(
            &mut Destination,
            w!(" program not found - skipping AUTOCHECK\r\n").as_ptr() as *const u16,
        );
        NtDisplayString(&mut Destination);
    } else {
        let BootState = SmpSaveAndClearBootStatusData(&mut BootOkay, &mut ShutdownOkay);

        RtlInitEmptyUnicodeString(
            &mut Destination,
            Buffer.as_mut_ptr(),
            (Buffer.len() * core::mem::size_of::<u16>()) as u16,
        );
        RtlAppendUnicodeStringToString(&mut Destination, FileName);
        RtlAppendUnicodeToString(&mut Destination, w!(" ").as_ptr() as *const u16);
        RtlAppendUnicodeStringToString(&mut Destination, Arguments);

        SmpExecuteImage(
            FileName,
            Directory,
            &mut Destination,
            0,
            Flags & !SMP_AUTOCHK_FLAG,
            core::ptr::null_mut(),
        );

        if BootState != 0 {
            SmpRestoreBootStatusData(BootOkay, ShutdownOkay);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpExecuteCommand(
    CommandLine: PUNICODE_STRING,
    MuSessionId: ULONG,
    ProcessId: PHANDLE,
    mut Flags: ULONG,
) -> NTSTATUS {
    let mut Arguments: _UNICODE_STRING = core::mem::zeroed();
    let mut Directory: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();

    if Flags & SMP_DEBUG_FLAG != 0 {
        return STATUS_SUCCESS;
    }

    let mut Status = SmpParseCommandLine(
        CommandLine,
        &mut Flags,
        &mut FileName,
        &mut Directory,
        &mut Arguments,
    );

    if Status < 0 {
        return Status;
    }

    if Flags & SMP_AUTOCHK_FLAG != 0 {
        Status = SmpInvokeAutoChk(&mut FileName, &mut Directory, &mut Arguments, Flags);
    } else if Flags & SMP_SUBSYSTEM_FLAG != 0 {
        Status = SmpLoadSubSystem(
            &mut FileName,
            &mut Directory,
            CommandLine,
            MuSessionId,
            ProcessId,
            Flags,
        );
    } else if Flags & SMP_INVALID_PATH != 0 {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
    } else {
        Status = SmpExecuteImage(
            &mut FileName,
            &mut Directory,
            CommandLine,
            MuSessionId,
            Flags,
            core::ptr::null_mut(),
        );
    }

    if !FileName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
    }
    if !Directory.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Directory.Buffer as PVOID);
    }
    if !Arguments.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Arguments.Buffer as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpExecuteInitialCommand(
    MuSessionId: ULONG,
    InitialCommand: PUNICODE_STRING,
    // C declared `IN HANDLE InitialCommandProcess` but passes it to NtDuplicateObject
    // (4th arg = PHANDLE/TargetHandle out-param). The C type is sloppy; we keep the
    // HANDLE ABI for compatibility with the ntdll.dll/csrsrv.dll consumers and cast
    // internally at the NtDuplicateObject call site.
    InitialCommandProcess: HANDLE,
    ReturnPid: PHANDLE,
) -> NTSTATUS {
    let mut ProcessInfo: RTL_USER_PROCESS_INFORMATION = core::mem::zeroed();
    let mut Arguments: _UNICODE_STRING = core::mem::zeroed();
    let mut Directory: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut Flags: ULONG = 0;

    if SmApiPort.is_null() {
        let Status = SmConnectToSm(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(SmApiPort),
        );
        if Status < 0 {
            crate::dbg_print!(b"[SMSS] SmConnectToSm FAILED st=%x\n\0", Status as u32); // perf phase marker
            return Status;
        }
    }

    let mut Status = SmpParseCommandLine(
        InitialCommand,
        &mut Flags,
        &mut FileName,
        &mut Directory,
        &mut Arguments,
    );

    if Flags & SMP_INVALID_PATH != 0 {
        if !FileName.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
        }
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    if Status < 0 {
        return Status;
    }

    Status = SmpExecuteImage(
        &mut FileName,
        &mut Directory,
        InitialCommand,
        MuSessionId,
        SMP_DEFERRED_FLAG,
        &mut ProcessInfo,
    );

    if !FileName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
    }
    if !Directory.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Directory.Buffer as PVOID);
    }
    if !Arguments.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Arguments.Buffer as PVOID);
    }

    if Status < 0 {
        return Status;
    }

    Status = NtDuplicateObject(
        NtCurrentProcess(),
        ProcessInfo.ProcessHandle,
        NtCurrentProcess(),
        InitialCommandProcess as PHANDLE,
        PROCESS_ALL_ACCESS,
        0,
        0,
    );

    if Status < 0 {
        NtTerminateProcess(ProcessInfo.ProcessHandle, Status);
        NtResumeThread(ProcessInfo.ThreadHandle, core::ptr::null_mut());
        NtClose(ProcessInfo.ThreadHandle);
        NtClose(ProcessInfo.ProcessHandle);
        return Status;
    }

    if !ReturnPid.is_null() {
        *ReturnPid = ProcessInfo.ClientId.UniqueProcess;
    }
    if MuSessionId == 0 {
        SmpInitialCommandProcessId = ProcessInfo.ClientId.UniqueProcess;
    }

    Status = SmExecPgm(SmApiPort, &mut ProcessInfo, 0u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpTerminate(
    Parameters: PULONG_PTR,
    ParameterMask: ULONG,
    ParameterCount: ULONG,
) -> NTSTATUS {
    let mut Old: BOOLEAN = 0;
    let mut Response: ULONG = 0;

    if RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, 1u8, 1u8, &mut Old) == STATUS_NO_TOKEN {
        RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, 1u8, 0u8, &mut Old);
    }

    let mut Status = NtRaiseHardError(
        STATUS_SYSTEM_PROCESS_TERMINATED,
        ParameterCount,
        ParameterMask,
        Parameters,
        OptionShutdownSystem,
        &mut Response,
    );

    // Exit with the real failure code in Parameters[1], not the hard-error call's own
    // status (upstream f255c9f046e).
    if ParameterCount >= 2 {
        Status = *Parameters.add(1) as NTSTATUS;
    }
    NtTerminateProcess(NtCurrentProcess(), Status)
}

#[no_mangle]
pub unsafe extern "C" fn SmpUnhandledExceptionFilter(ExceptionInfo: PEXCEPTION_POINTERS) -> LONG {
    let mut Parameters: [ULONG_PTR; 4] = [0; 4];
    let mut ErrorString: _UNICODE_STRING = core::mem::zeroed();

    crate::dbg_print!(
        b"SMSS: Unhandled exception - Status == %x  IP == %p\n\0",
        (*(*ExceptionInfo).ExceptionRecord).ExceptionCode,
        (*(*ExceptionInfo).ExceptionRecord).ExceptionAddress,
    );
    // Upstream 5178c3913ed: the address is [1] and R/W is [0], printed only for the codes
    // that carry them, and the break happens only under a debugger -- an unconditional
    // DbgBreakPoint in the filter is itself unhandled on a debugger-free boot.
    let Record = (*ExceptionInfo).ExceptionRecord;
    let Code = (*Record).ExceptionCode as NTSTATUS;
    if Code == STATUS_IN_PAGE_ERROR && (*Record).NumberParameters >= 3 {
        crate::dbg_print!(
            b"      Memory Address: %x  Read/Write: %x  I/O Error: %x\n\0",
            (*Record).ExceptionInformation[1],
            (*Record).ExceptionInformation[0],
            (*Record).ExceptionInformation[2],
        );
    } else if (Code == STATUS_ACCESS_VIOLATION
        || Code == STATUS_GUARD_PAGE_VIOLATION
        || Code == STATUS_STACK_OVERFLOW
        || Code == STATUS_IN_PAGE_ERROR)
        && (*Record).NumberParameters >= 2
    {
        crate::dbg_print!(
            b"      Memory Address: %x  Read/Write: %x\n\0",
            (*Record).ExceptionInformation[1],
            (*Record).ExceptionInformation[0],
        );
    }
    if (*NtCurrentPeb()).BeingDebugged != 0 {
        DbgBreakPoint();
    }

    static ERROR_MSG: [u16; 42] = [
        b'U' as u16,
        b'n' as u16,
        b'h' as u16,
        b'a' as u16,
        b'n' as u16,
        b'd' as u16,
        b'l' as u16,
        b'e' as u16,
        b'd' as u16,
        b' ' as u16,
        b'E' as u16,
        b'x' as u16,
        b'c' as u16,
        b'e' as u16,
        b'p' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b' ' as u16,
        b'i' as u16,
        b'n' as u16,
        b' ' as u16,
        b'S' as u16,
        b'e' as u16,
        b's' as u16,
        b's' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b' ' as u16,
        b'M' as u16,
        b'a' as u16,
        b'n' as u16,
        b'a' as u16,
        b'g' as u16,
        b'e' as u16,
        b'r' as u16,
        0,
        0,
        0,
        0,
    ];
    RtlInitUnicodeString(&mut ErrorString, ERROR_MSG.as_ptr());
    Parameters[0] = core::ptr::addr_of!(ErrorString) as ULONG_PTR;
    Parameters[1] = (*(*ExceptionInfo).ExceptionRecord).ExceptionCode as ULONG_PTR;
    Parameters[2] = (*(*ExceptionInfo).ExceptionRecord).ExceptionAddress as ULONG_PTR;
    Parameters[3] = (*ExceptionInfo).ContextRecord as ULONG_PTR;
    SmpTerminate(Parameters.as_mut_ptr(), 1, 4);

    EXCEPTION_EXECUTE_HANDLER
}

// smss.exe `_main`. Under `smss_exe_only` this is the binary's `#[no_mangle]`
// entry (csrss.rs is gated out in that feature set, so there is no `_main`
// collision). Under default features it stays a private mangled symbol so the
// archive can coexist with csrss::_main in the workspace cargo check.
#[cfg_attr(feature = "smss_exe_only", no_mangle)]
#[cfg_attr(not(feature = "smss_exe_only"), allow(dead_code))]
pub unsafe extern "C" fn _main(
    _argc: i32,
    _argv: *const *const i8,
    _envp: *const *const i8,
    DebugFlag: ULONG,
) -> NTSTATUS {
    let mut SetBasePriority: KPRIORITY = 11;
    let mut Parameters: [ULONG_PTR; 4] = [0; 4];
    let mut Handles: [HANDLE; 2] = [core::ptr::null_mut(); 2];
    let mut State: PVOID = core::ptr::null_mut();
    let mut Flags: ULONG = 0;
    let mut ProcessInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut DbgString: _UNICODE_STRING = core::mem::zeroed();
    let mut InitialCommand: _UNICODE_STRING = core::mem::zeroed();

    crate::dbg_print!(b"[SMSS] _main entered\n\0");

    RtlSetProcessIsCritical(1u8, core::ptr::null_mut(), 0u8);
    RtlSetThreadIsCritical(1u8, core::ptr::null_mut(), 0u8);

    let mut Status = NtSetInformationProcess(
        NtCurrentProcess(),
        ProcessBasePriority,
        core::ptr::addr_of_mut!(SetBasePriority) as PVOID,
        core::mem::size_of::<KPRIORITY>() as ULONG,
    );

    if DebugFlag != 0 {
        SmpDebug = 1;
    }

    Parameters[0] = core::ptr::addr_of!(DbgString) as ULONG_PTR;
    Parameters[1] = 0;
    Parameters[2] = 0;
    Parameters[3] = 0;

    Status = SmpInit(&mut InitialCommand, Handles.as_mut_ptr());
    if Status < 0 {
        static INIT_MSG: [u16; 31] = [
            b'S' as u16,
            b'e' as u16,
            b's' as u16,
            b's' as u16,
            b'i' as u16,
            b'o' as u16,
            b'n' as u16,
            b' ' as u16,
            b'M' as u16,
            b'a' as u16,
            b'n' as u16,
            b'a' as u16,
            b'g' as u16,
            b'e' as u16,
            b'r' as u16,
            b' ' as u16,
            b'I' as u16,
            b'n' as u16,
            b'i' as u16,
            b't' as u16,
            b'i' as u16,
            b'a' as u16,
            b'l' as u16,
            b'i' as u16,
            b'z' as u16,
            b'a' as u16,
            b't' as u16,
            b'i' as u16,
            b'o' as u16,
            b'n' as u16,
         0,];
        RtlInitUnicodeString(&mut DbgString, INIT_MSG.as_ptr());
        crate::dbg_print!(b"[SMSS] SmpInit FAILED st=%x\n\0", Status as u32); // perf phase marker
        Parameters[1] = Status as ULONG_PTR;
        return SmpTerminate(Parameters.as_mut_ptr(), 1, 4);
    }

    Status = NtQuerySystemInformation(
        SystemFlagsInformation,
        core::ptr::addr_of_mut!(Flags) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );

    if Flags & (FLG_DEBUG_INITIAL_COMMAND | FLG_DEBUG_INITIAL_COMMAND_EX) != 0 {
        crate::dbg_print!(b"Global Flags Set to SMSS Debugging: Not yet supported\n\0");
    }

    Status = SmpExecuteInitialCommand(
        0,
        &mut InitialCommand,
        &mut Handles[1] as *mut HANDLE as HANDLE,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        static EXEC_MSG: [u16; 40] = [
            b'S' as u16,
            b'e' as u16,
            b's' as u16,
            b's' as u16,
            b'i' as u16,
            b'o' as u16,
            b'n' as u16,
            b' ' as u16,
            b'M' as u16,
            b'a' as u16,
            b'n' as u16,
            b'a' as u16,
            b'g' as u16,
            b'e' as u16,
            b'r' as u16,
            b' ' as u16,
            b'E' as u16,
            b'x' as u16,
            b'e' as u16,
            b'c' as u16,
            b'u' as u16,
            b't' as u16,
            b'e' as u16,
            b'I' as u16,
            b'n' as u16,
            b'i' as u16,
            b't' as u16,
            b'i' as u16,
            b'a' as u16,
            b'l' as u16,
            b'C' as u16,
            b'o' as u16,
            b'm' as u16,
            b'm' as u16,
            b'a' as u16,
            b'n' as u16,
            b'd' as u16,
            0,
            0,
            0,
        ];
        RtlInitUnicodeString(&mut DbgString, EXEC_MSG.as_ptr());
        Parameters[1] = Status as ULONG_PTR;
        return SmpTerminate(Parameters.as_mut_ptr(), 1, 4);
    }

    Status = SmpAcquirePrivilege(SE_LOAD_DRIVER_PRIVILEGE, &mut State);
    if AttachedSessionId != 0xFFFFFFFF {
        Status = NtSetSystemInformation(
            SystemSessionDetach,
            core::ptr::addr_of_mut!(AttachedSessionId) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        AttachedSessionId = 0xFFFFFFFF;
    }
    SmpReleasePrivilege(State);

    Status = NtWaitForMultipleObjects(2, Handles.as_mut_ptr(), WaitAny, 0u8, core::ptr::null_mut());

    if Status == STATUS_WAIT_0 {
        static WIN_SUBSYS: [u16; 18] = [
            b'W' as u16,
            b'i' as u16,
            b'n' as u16,
            b'd' as u16,
            b'o' as u16,
            b'w' as u16,
            b's' as u16,
            b' ' as u16,
            b'S' as u16,
            b'u' as u16,
            b'b' as u16,
            b'S' as u16,
            b'y' as u16,
            b's' as u16,
            b't' as u16,
            b'e' as u16,
            b'm' as u16,
         0,];
        RtlInitUnicodeString(&mut DbgString, WIN_SUBSYS.as_ptr());
        Status = NtQueryInformationProcess(
            Handles[0],
            ProcessBasicInformation,
            core::ptr::addr_of_mut!(ProcessInfo) as PVOID,
            core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
            core::ptr::null_mut(),
        );
    } else {
        static WIN_LOGON: [u16; 22] = [
            b'W' as u16,
            b'i' as u16,
            b'n' as u16,
            b'd' as u16,
            b'o' as u16,
            b'w' as u16,
            b's' as u16,
            b' ' as u16,
            b'L' as u16,
            b'o' as u16,
            b'g' as u16,
            b'o' as u16,
            b'n' as u16,
            b' ' as u16,
            b'P' as u16,
            b'r' as u16,
            b'o' as u16,
            b'c' as u16,
            b'e' as u16,
            b's' as u16,
            b's' as u16,
         0,];
        RtlInitUnicodeString(&mut DbgString, WIN_LOGON.as_ptr());
        if Status == STATUS_WAIT_1 {
            Status = NtQueryInformationProcess(
                Handles[1],
                ProcessBasicInformation,
                core::ptr::addr_of_mut!(ProcessInfo) as PVOID,
                core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
                core::ptr::null_mut(),
            );
        } else {
            ProcessInfo.ExitStatus = Status;
            Status = STATUS_SUCCESS;
        }
    }

    if Status >= 0 {
        Parameters[1] = ProcessInfo.ExitStatus as ULONG_PTR;
    } else {
        Parameters[1] = STATUS_UNSUCCESSFUL as ULONG_PTR;
    }

    SmpTerminate(Parameters.as_mut_ptr(), 1, 4)
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/smss.c
//  * PURPOSE:         Main SMSS Code
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "smss.h"
//
// #include <pseh/pseh2.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// UNICODE_STRING SmpSystemRoot;
// ULONG AttachedSessionId = -1;
// BOOLEAN SmpDebug, SmpEnableDots;
// HANDLE SmApiPort;
// HANDLE SmpInitialCommandProcessId;
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// SmpExecuteImage(IN PUNICODE_STRING FileName,
//                 IN PUNICODE_STRING Directory,
//                 IN PUNICODE_STRING CommandLine,
//                 IN ULONG MuSessionId,
//                 IN ULONG Flags,
//                 IN PRTL_USER_PROCESS_INFORMATION ProcessInformation)
// {
//     PRTL_USER_PROCESS_INFORMATION ProcessInfo;
//     NTSTATUS Status;
//     RTL_USER_PROCESS_INFORMATION LocalProcessInfo;
//     PRTL_USER_PROCESS_PARAMETERS ProcessParameters;
//
//     /* Use the input process information if we have it, otherwise use local */
//     ProcessInfo = ProcessInformation;
//     if (!ProcessInfo) ProcessInfo = &LocalProcessInfo;
//
//     /* Create parameters for the target process */
//     Status = RtlCreateProcessParameters(&ProcessParameters,
//                                         FileName,
//                                         SmpDefaultLibPath.Length ?
//                                         &SmpDefaultLibPath : NULL,
//                                         Directory,
//                                         CommandLine,
//                                         SmpDefaultEnvironment,
//                                         NULL,
//                                         NULL,
//                                         NULL,
//                                         0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* This is a pretty bad failure. ASSERT on checked builds and exit */
//         ASSERTMSG("RtlCreateProcessParameters failed.\n", NT_SUCCESS(Status));
//         DPRINT1("SMSS: RtlCreateProcessParameters failed for %wZ - Status == %lx\n",
//                 FileName, Status);
//         return Status;
//     }
//
//     /* Set the size field as required */
//     ProcessInfo->Size = sizeof(*ProcessInfo);
//
//     /* Check if the debug flag was requested */
//     if (Flags & SMP_DEBUG_FLAG)
//     {
//         /* Write it in the process parameters */
//         ProcessParameters->DebugFlags = 1;
//     }
//     else
//     {
//         /* Otherwise inherit the flag that was passed to SMSS itself */
//         ProcessParameters->DebugFlags = SmpDebug;
//     }
//
//     /* Subsystems get the first 1MB of memory reserved for DOS/IVT purposes */
//     if (Flags & SMP_SUBSYSTEM_FLAG)
//     {
//         ProcessParameters->Flags |= RTL_USER_PROCESS_PARAMETERS_RESERVE_1MB;
//     }
//
//     /* And always force NX for anything that SMSS launches */
//     ProcessParameters->Flags |= RTL_USER_PROCESS_PARAMETERS_NX;
//
//     /* Now create the process in suspended state */
//     Status = RtlCreateUserProcess(FileName,
//                                   OBJ_CASE_INSENSITIVE,
//                                   ProcessParameters,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   FALSE,
//                                   NULL,
//                                   NULL,
//                                   ProcessInfo);
//     RtlDestroyProcessParameters(ProcessParameters);
//     if (!NT_SUCCESS(Status))
//     {
//         /* If we couldn't create it, fail back to the caller */
//         DPRINT1("SMSS: Failed load of %wZ - Status  == %lx\n",
//                 FileName, Status);
//         return Status;
//     }
//
//     /* Associate a session with this process */
//     Status = SmpSetProcessMuSessionId(ProcessInfo->ProcessHandle, MuSessionId);
//
//     /* If the application is deferred (suspended), there's nothing to do */
//     if (Flags & SMP_DEFERRED_FLAG) return Status;
//
//     /* Otherwise, get ready to start it, but make sure it's a native app */
//     if (ProcessInfo->ImageInformation.SubSystemType == IMAGE_SUBSYSTEM_NATIVE)
//     {
//         /* Resume it */
//         NtResumeThread(ProcessInfo->ThreadHandle, NULL);
//         if (!(Flags & SMP_ASYNC_FLAG))
//         {
//             /* Block on it unless Async was requested */
//             NtWaitForSingleObject(ProcessInfo->ThreadHandle, FALSE, NULL);
//         }
//
//         /* It's up and running now, close our handles */
//         NtClose(ProcessInfo->ThreadHandle);
//         NtClose(ProcessInfo->ProcessHandle);
//     }
//     else
//     {
//         /* This image is invalid, so kill it, close our handles, and fail */
//         Status = STATUS_INVALID_IMAGE_FORMAT;
//         NtTerminateProcess(ProcessInfo->ProcessHandle, Status);
//         NtWaitForSingleObject(ProcessInfo->ThreadHandle, FALSE, NULL);
//         NtClose(ProcessInfo->ThreadHandle);
//         NtClose(ProcessInfo->ProcessHandle);
//         DPRINT1("SMSS: Not an NT image - %wZ\n", FileName);
//     }
//
//     /* Return the outcome of the process create */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpInvokeAutoChk(IN PUNICODE_STRING FileName,
//                  IN PUNICODE_STRING Directory,
//                  IN PUNICODE_STRING Arguments,
//                  IN ULONG Flags)
// {
//     ANSI_STRING MessageString;
//     CHAR MessageBuffer[256];
//     UNICODE_STRING Destination;
//     WCHAR Buffer[1024];
//     BOOLEAN BootState, BootOkay, ShutdownOkay;
//
//     /* Check if autochk should show dots (if the user booted with /SOS) */
//     if (SmpQueryRegistrySosOption()) SmpEnableDots = FALSE;
//
//     /* Make sure autochk was actually found */
//     if (Flags & SMP_INVALID_PATH)
//     {
//         /* It wasn't, so create an error message to print on the screen */
//         RtlStringCbPrintfA(MessageBuffer,
//                            sizeof(MessageBuffer),
//                            "%wZ program not found - skipping AUTOCHECK\r\n",
//                            FileName);
//         RtlInitAnsiString(&MessageString, MessageBuffer);
//         if (NT_SUCCESS(RtlAnsiStringToUnicodeString(&Destination,
//                                                     &MessageString,
//                                                     TRUE)))
//         {
//             /* And show it */
//             NtDisplayString(&Destination);
//             RtlFreeUnicodeString(&Destination);
//         }
//     }
//     else
//     {
//         /* Autochk is there, so record the BSD state */
//         BootState = SmpSaveAndClearBootStatusData(&BootOkay, &ShutdownOkay);
//
//         /* Build the path to autochk and place its arguments */
//         RtlInitEmptyUnicodeString(&Destination, Buffer, sizeof(Buffer));
//         RtlAppendUnicodeStringToString(&Destination, FileName);
//         RtlAppendUnicodeToString(&Destination, L" ");
//         RtlAppendUnicodeStringToString(&Destination, Arguments);
//
//         /* Execute it */
//         SmpExecuteImage(FileName,
//                         Directory,
//                         &Destination,
//                         0,
//                         Flags & ~SMP_AUTOCHK_FLAG,
//                         NULL);
//
//         /* Restore the BSD state */
//         if (BootState) SmpRestoreBootStatusData(BootOkay, ShutdownOkay);
//     }
//
//     /* We're all done! */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpExecuteCommand(IN PUNICODE_STRING CommandLine,
//                   IN ULONG MuSessionId,
//                   OUT PHANDLE ProcessId,
//                   IN ULONG Flags)
// {
//     NTSTATUS Status;
//     UNICODE_STRING Arguments, Directory, FileName;
//
//     /* There's no longer a debugging subsystem */
//     if (Flags & SMP_DEBUG_FLAG) return STATUS_SUCCESS;
//
//     /* Parse the command line to see what execution flags are requested */
//     Status = SmpParseCommandLine(CommandLine,
//                                  &Flags,
//                                  &FileName,
//                                  &Directory,
//                                  &Arguments);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail if we couldn't do that */
//         DPRINT1("SMSS: SmpParseCommandLine( %wZ ) failed - Status == %lx\n",
//                 CommandLine, Status);
//         return Status;
//     }
//
//     /* Check if autochk is requested */
//     if (Flags & SMP_AUTOCHK_FLAG)
//     {
//         /* Run it */
//         Status = SmpInvokeAutoChk(&FileName, &Directory, &Arguments, Flags);
//     }
//     else if (Flags & SMP_SUBSYSTEM_FLAG)
//     {
//         Status = SmpLoadSubSystem(&FileName,
//                                   &Directory,
//                                   CommandLine,
//                                   MuSessionId,
//                                   ProcessId,
//                                   Flags);
//     }
//     else if (Flags & SMP_INVALID_PATH)
//     {
//         /* An invalid image was specified, fail */
//         DPRINT1("SMSS: Image file (%wZ) not found\n", &FileName);
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//     else
//     {
//         /* An actual image name was present, execute it */
//         Status = SmpExecuteImage(&FileName,
//                                  &Directory,
//                                  CommandLine,
//                                  MuSessionId,
//                                  Flags,
//                                  NULL);
//     }
//
//     /* Free all the token parameters */
//     if (FileName.Buffer)  RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//     if (Directory.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, Directory.Buffer);
//     if (Arguments.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, Arguments.Buffer);
//
//     /* Return to the caller */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Command '%wZ' failed - Status == %x\n",
//                 CommandLine, Status);
//     }
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpExecuteInitialCommand(IN ULONG MuSessionId,
//                          IN PUNICODE_STRING InitialCommand,
//                          IN HANDLE InitialCommandProcess,
//                          OUT PHANDLE ReturnPid)
// {
//     NTSTATUS Status;
//     RTL_USER_PROCESS_INFORMATION ProcessInfo;
//     UNICODE_STRING Arguments, Directory, FileName;
//     ULONG Flags = 0;
//
//     /* Check if we haven't yet connected to ourselves */
//     if (!SmApiPort)
//     {
//         /* Connect to ourselves, as a client */
//         Status = SmConnectToSm(NULL, NULL, 0, &SmApiPort);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("SMSS: Unable to connect to SM - Status == %lx\n", Status);
//             return Status;
//         }
//     }
//
//     /* Parse the initial command line */
//     Status = SmpParseCommandLine(InitialCommand,
//                                  &Flags,
//                                  &FileName,
//                                  &Directory,
//                                  &Arguments);
//     if (Flags & SMP_INVALID_PATH)
//     {
//         /* Fail if it doesn't exist */
//         DPRINT1("SMSS: Initial command image (%wZ) not found\n", &FileName);
//         if (FileName.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /* And fail if any other reason is also true */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: SmpParseCommandLine( %wZ ) failed - Status == %lx\n",
//                 InitialCommand, Status);
//         return Status;
//     }
//
//     /* Execute the initial command, but defer its full execution */
//     Status = SmpExecuteImage(&FileName,
//                              &Directory,
//                              InitialCommand,
//                              MuSessionId,
//                              SMP_DEFERRED_FLAG,
//                              &ProcessInfo);
//
//     /* Free all the token parameters */
//     if (FileName.Buffer)  RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//     if (Directory.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, Directory.Buffer);
//     if (Arguments.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, Arguments.Buffer);
//
//     /* Bail out if we couldn't execute the initial command */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Now duplicate the handle to this process */
//     Status = NtDuplicateObject(NtCurrentProcess(),
//                                ProcessInfo.ProcessHandle,
//                                NtCurrentProcess(),
//                                InitialCommandProcess,
//                                PROCESS_ALL_ACCESS,
//                                0,
//                                0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Kill it utterly if duplication failed */
//         DPRINT1("SMSS: DupObject Failed. Status == %lx\n", Status);
//         NtTerminateProcess(ProcessInfo.ProcessHandle, Status);
//         NtResumeThread(ProcessInfo.ThreadHandle, NULL);
//         NtClose(ProcessInfo.ThreadHandle);
//         NtClose(ProcessInfo.ProcessHandle);
//         return Status;
//     }
//
//     /* Return PID to the caller, and set this as the initial command PID */
//     if (ReturnPid) *ReturnPid = ProcessInfo.ClientId.UniqueProcess;
//     if (!MuSessionId) SmpInitialCommandProcessId = ProcessInfo.ClientId.UniqueProcess;
//
//     /* Now call our server execution function to wrap up its initialization */
//     Status = SmExecPgm(SmApiPort, &ProcessInfo, FALSE);
//     if (!NT_SUCCESS(Status)) DPRINT1("SMSS: SmExecPgm Failed. Status == %lx\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// SmpTerminate(IN PULONG_PTR Parameters,
//              IN ULONG ParameterMask,
//              IN ULONG ParameterCount)
// {
//     NTSTATUS Status;
//     BOOLEAN Old;
//     ULONG Response;
//
//     /* Give the shutdown privilege to the thread */
//     if (RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, TRUE, TRUE, &Old) ==
//         STATUS_NO_TOKEN)
//     {
//         /* Thread doesn't have a token, give it to the entire process */
//         RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, TRUE, FALSE, &Old);
//     }
//
//     /* Take down the process/machine with a hard error */
//     Status = NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED,
//                               ParameterCount,
//                               ParameterMask,
//                               Parameters,
//                               OptionShutdownSystem,
//                               &Response);
//
//     /* Terminate the process if the hard error didn't already */
//     return NtTerminateProcess(NtCurrentProcess(), Status);
// }
//
// LONG
// SmpUnhandledExceptionFilter(IN PEXCEPTION_POINTERS ExceptionInfo)
// {
//     ULONG_PTR Parameters[4];
//     UNICODE_STRING ErrorString;
//
//     /* Print and breakpoint into the debugger */
//     DbgPrint("SMSS: Unhandled exception - Status == %x  IP == %p\n",
//              ExceptionInfo->ExceptionRecord->ExceptionCode,
//              ExceptionInfo->ExceptionRecord->ExceptionAddress);
//     DbgPrint("      Memory Address: %x  Read/Write: %x\n",
//              ExceptionInfo->ExceptionRecord->ExceptionInformation[0],
//              ExceptionInfo->ExceptionRecord->ExceptionInformation[1]);
//     DbgBreakPoint();
//
//     /* Build the hard error and terminate */
//     RtlInitUnicodeString(&ErrorString, L"Unhandled Exception in Session Manager");
//     Parameters[0] = (ULONG_PTR)&ErrorString;
//     Parameters[1] = ExceptionInfo->ExceptionRecord->ExceptionCode;
//     Parameters[2] = (ULONG_PTR)ExceptionInfo->ExceptionRecord->ExceptionAddress;
//     Parameters[3] = (ULONG_PTR)ExceptionInfo->ContextRecord;
//     SmpTerminate(Parameters, 1, RTL_NUMBER_OF(Parameters));
//
//     /* We should never get here */
//     ASSERT(FALSE);
//     return EXCEPTION_EXECUTE_HANDLER;
// }
//
// NTSTATUS
// __cdecl
// _main(IN INT argc,
//       IN PCHAR argv[],
//       IN PCHAR envp[],
//       IN ULONG DebugFlag)
// {
//     NTSTATUS Status;
//     KPRIORITY SetBasePriority;
//     ULONG_PTR Parameters[4];
//     HANDLE Handles[2];
//     PVOID State;
//     ULONG Flags;
//     PROCESS_BASIC_INFORMATION ProcessInfo;
//     UNICODE_STRING DbgString, InitialCommand;
//
//     /* Boot marker — proves SMSS user-mode code is running */
//     DbgPrint("[SMSS] _main entered\n");
//
//     /* Make us critical */
//     RtlSetProcessIsCritical(TRUE, NULL, FALSE);
//     RtlSetThreadIsCritical(TRUE, NULL, FALSE);
//
//     /* Raise our priority */
//     SetBasePriority = 11;
//     Status = NtSetInformationProcess(NtCurrentProcess(),
//                                      ProcessBasePriority,
//                                      (PVOID)&SetBasePriority,
//                                      sizeof(SetBasePriority));
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Save the debug flag if it was passed */
//     if (DebugFlag) SmpDebug = DebugFlag != 0;
//
//     /* Build the hard error parameters */
//     Parameters[0] = (ULONG_PTR)&DbgString;
//     Parameters[1] = Parameters[2] = Parameters[3] = 0;
//
//     /* Enter SEH so we can terminate correctly if anything goes wrong */
//     _SEH2_TRY
//     {
//         /* Initialize SMSS */
//         Status = SmpInit(&InitialCommand, &Handles[0]);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("SMSS: SmpInit return failure - Status == %x\n", Status);
//             RtlInitUnicodeString(&DbgString, L"Session Manager Initialization");
//             Parameters[1] = Status;
//             _SEH2_LEAVE;
//         }
//
//         /* Get the global flags */
//         Status = NtQuerySystemInformation(SystemFlagsInformation,
//                                           &Flags,
//                                           sizeof(Flags),
//                                           NULL);
//         ASSERT(NT_SUCCESS(Status));
//
//         /* Before executing the initial command check if the debug flag is on */
//         if (Flags & (FLG_DEBUG_INITIAL_COMMAND | FLG_DEBUG_INITIAL_COMMAND_EX))
//         {
//             /* SMSS should launch ntsd with a few parameters at this point */
//             DPRINT1("Global Flags Set to SMSS Debugging: Not yet supported\n");
//         }
//
//         /* Execute the initial command (Winlogon.exe) */
//         Status = SmpExecuteInitialCommand(0, &InitialCommand, &Handles[1], NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail and raise a hard error */
//             DPRINT1("SMSS: Execute Initial Command failed\n");
//             RtlInitUnicodeString(&DbgString,
//                                  L"Session Manager ExecuteInitialCommand");
//             Parameters[1] = Status;
//             _SEH2_LEAVE;
//         }
//
//         /*  Check if we're already attached to a session */
//         Status = SmpAcquirePrivilege(SE_LOAD_DRIVER_PRIVILEGE, &State);
//         if (AttachedSessionId != -1)
//         {
//             /* Detach from it, we should be in no session right now */
//             Status = NtSetSystemInformation(SystemSessionDetach,
//                                             &AttachedSessionId,
//                                             sizeof(AttachedSessionId));
//             ASSERT(NT_SUCCESS(Status));
//             AttachedSessionId = -1;
//         }
//         SmpReleasePrivilege(State);
//
//         /* Wait on either CSRSS or Winlogon to die */
//         Status = NtWaitForMultipleObjects(RTL_NUMBER_OF(Handles),
//                                           Handles,
//                                           WaitAny,
//                                           FALSE,
//                                           NULL);
//         if (Status == STATUS_WAIT_0)
//         {
//             /* CSRSS is dead, get exit code and prepare for the hard error */
//             RtlInitUnicodeString(&DbgString, L"Windows SubSystem");
//             Status = NtQueryInformationProcess(Handles[0],
//                                                ProcessBasicInformation,
//                                                &ProcessInfo,
//                                                sizeof(ProcessInfo),
//                                                NULL);
//             DPRINT1("SMSS: Windows subsystem terminated when it wasn't supposed to.\n");
//         }
//         else
//         {
//             /* The initial command is dead or we have another failure */
//             RtlInitUnicodeString(&DbgString, L"Windows Logon Process");
//             if (Status == STATUS_WAIT_1)
//             {
//                 /* Winlogon.exe got terminated, get its exit code */
//                 Status = NtQueryInformationProcess(Handles[1],
//                                                    ProcessBasicInformation,
//                                                    &ProcessInfo,
//                                                    sizeof(ProcessInfo),
//                                                    NULL);
//             }
//             else
//             {
//                 /* Something else satisfied our wait, so set the wait status */
//                 ProcessInfo.ExitStatus = Status;
//                 Status = STATUS_SUCCESS;
//             }
//             DPRINT1("SMSS: Initial command '%wZ' terminated when it wasn't supposed to. ExitStatus=0x%lx\n",
//                     &InitialCommand, ProcessInfo.ExitStatus);
//         }
//
//         /* Check if NtQueryInformationProcess was successful */
//         if (NT_SUCCESS(Status))
//         {
//             /* Then we must have a valid exit status in the structure, use it */
//             Parameters[1] = ProcessInfo.ExitStatus;
//         }
//         else
//         {
//             /* We really don't know what happened, so set a generic error */
//             Parameters[1] = STATUS_UNSUCCESSFUL;
//         }
//     }
//     _SEH2_EXCEPT(SmpUnhandledExceptionFilter(_SEH2_GetExceptionInformation()))
//     {
//         /* The filter should never return here */
//         ASSERT(FALSE);
//     }
//     _SEH2_END;
//
//     /* Something in the init loop failed, terminate SMSS */
//     return SmpTerminate(Parameters, 1, RTL_NUMBER_OF(Parameters));
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
