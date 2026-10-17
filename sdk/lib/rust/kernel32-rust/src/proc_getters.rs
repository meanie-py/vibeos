use crate::substrate::*;

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

/* PUBLIC FUNCTIONS ***********************************************************/

// Upstream-sync 2026-09: GetActiveProcessorGroupCount/GetActiveProcessorCount/
// GetMaximumProcessorCount are genuinely NEW upstream code (proc.c, added in the
// 1,269-commit merge; no pre-merge C or Rust form existed). All three are UP-only fakes
// in the C original too -- `STUB; // FIXME: Real group support` -- so this is a
// faithful, not a lesser, port: there is only ever one group (THEONLYGROUP == 0) until
// real processor-group support lands.
const THEONLYGROUP: WORD = 0;

#[no_mangle]
pub unsafe extern "system" fn GetActiveProcessorGroupCount() -> WORD {
    STUB!(); // FIXME: Real group support
    1
}

unsafe fn GetProcessorCountInfo(GroupNumber: WORD, _Maximum: BOOL) -> DWORD {
    if GroupNumber != ALL_PROCESSOR_GROUPS as WORD && GroupNumber != THEONLYGROUP {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    // NB: the sysinfo island has its own _SYSTEM_INFO distinct from ntdll-sys's;
    // GetNativeSystemInfo takes the former, so name it explicitly.
    let mut info: crate::client::sysinfo::_SYSTEM_INFO = core::mem::zeroed();
    crate::client::sysinfo::GetNativeSystemInfo(&mut info);
    info.dwNumberOfProcessors
}

#[no_mangle]
pub unsafe extern "system" fn GetActiveProcessorCount(GroupNumber: WORD) -> DWORD {
    GetProcessorCountInfo(GroupNumber, FALSE)
}

#[no_mangle]
pub unsafe extern "system" fn GetMaximumProcessorCount(GroupNumber: WORD) -> DWORD {
    GetProcessorCountInfo(GroupNumber, TRUE)
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessAffinityMask(
    hProcess: HANDLE,
    lpProcessAffinityMask: PDWORD_PTR,
    lpSystemAffinityMask: PDWORD_PTR,
) -> BOOL {
    let mut ProcessInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();

    /* Query information on the process from the kernel */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessBasicInformation,
        &mut ProcessInfo as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Copy the affinity mask, and get the system one from our shared data */
    *lpProcessAffinityMask = ProcessInfo.AffinityMask as DWORD as DWORD_PTR;
    *lpSystemAffinityMask =
        (*BaseStaticServerData).SysInfo.ActiveProcessorsAffinityMask as DWORD as DWORD_PTR;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessAffinityMask(
    hProcess: HANDLE,
    dwProcessAffinityMask: DWORD_PTR,
) -> BOOL {
    /* Directly set the affinity mask */
    let Status = NtSetInformationProcess(
        hProcess,
        ProcessAffinityMask,
        &dwProcessAffinityMask as *const _ as PVOID,
        core::mem::size_of::<DWORD_PTR>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        /* Handle failure */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Everything was ok */
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessShutdownParameters(
    lpdwLevel: LPDWORD,
    lpdwFlags: LPDWORD,
) -> BOOL {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let ShutdownParametersRequest = core::ptr::addr_of_mut!(ApiMessage.Data.ShutdownParametersRequest);

    /* Ask CSRSS for shutdown information */
    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetProcessShutdownParam),
        core::mem::size_of::<BASE_GETSET_PROCESS_SHUTDOWN_PARAMS>() as ULONG,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        /* Return the failure from CSRSS */
        BaseSetLastNTError(ApiMessage.Status);
        return FALSE;
    }

    /* Get the data back */
    *lpdwLevel = (*ShutdownParametersRequest).ShutdownLevel;
    *lpdwFlags = (*ShutdownParametersRequest).ShutdownFlags;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessShutdownParameters(
    dwLevel: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let ShutdownParametersRequest = &mut ApiMessage.Data.ShutdownParametersRequest;

    /* Write the data into the CSRSS request and send it */
    (*ShutdownParametersRequest).ShutdownLevel = dwLevel;
    (*ShutdownParametersRequest).ShutdownFlags = dwFlags;
    CsrClientCallServer(
        &mut ApiMessage as *mut _ as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepSetProcessShutdownParam),
        core::mem::size_of::<BASE_GETSET_PROCESS_SHUTDOWN_PARAMS>() as ULONG,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        /* Return the failure from CSRSS */
        BaseSetLastNTError(ApiMessage.Status);
        return FALSE;
    }

    /* All went well */
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessWorkingSetSizeEx(
    hProcess: HANDLE,
    lpMinimumWorkingSetSize: PSIZE_T,
    lpMaximumWorkingSetSize: PSIZE_T,
    Flags: PDWORD,
) -> BOOL {
    let mut QuotaLimits: QUOTA_LIMITS_EX = core::mem::zeroed();

    /* Query the kernel about this */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessQuotaLimits,
        &mut QuotaLimits as *mut _ as PVOID,
        core::mem::size_of::<QUOTA_LIMITS_EX>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Return error */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Copy the quota information out */
    *lpMinimumWorkingSetSize = QuotaLimits.MinimumWorkingSetSize;
    *lpMaximumWorkingSetSize = QuotaLimits.MaximumWorkingSetSize;
    *Flags = QuotaLimits.Flags;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessWorkingSetSize(
    hProcess: HANDLE,
    lpMinimumWorkingSetSize: PSIZE_T,
    lpMaximumWorkingSetSize: PSIZE_T,
) -> BOOL {
    let mut Dummy: DWORD = 0;
    GetProcessWorkingSetSizeEx(
        hProcess,
        lpMinimumWorkingSetSize,
        lpMaximumWorkingSetSize,
        &mut Dummy,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessWorkingSetSizeEx(
    hProcess: HANDLE,
    dwMinimumWorkingSetSize: SIZE_T,
    dwMaximumWorkingSetSize: SIZE_T,
    Flags: DWORD,
) -> BOOL {
    let mut QuotaLimits: QUOTA_LIMITS_EX = core::mem::zeroed();
    let ReturnStatus: NTSTATUS;
    let Result: BOOL;
    let mut State: PVOID = core::ptr::null_mut();
    let mut Privilege: ULONG = SE_INC_BASE_PRIORITY_PRIVILEGE;

    /* Zero out the input structure */
    RtlZeroMemory(
        &mut QuotaLimits as *mut _ as PVOID,
        core::mem::size_of::<QUOTA_LIMITS_EX>() as SIZE_T,
    );

    /* Check if the caller sent any limits */
    if (dwMinimumWorkingSetSize != 0) && (dwMaximumWorkingSetSize != 0) {
        /* Write the quota information */
        QuotaLimits.MinimumWorkingSetSize = dwMinimumWorkingSetSize;
        QuotaLimits.MaximumWorkingSetSize = dwMaximumWorkingSetSize;
        QuotaLimits.Flags = Flags;

        /* Acquire the required privilege */
        let Status = RtlAcquirePrivilege(&mut Privilege, 1, 0, &mut State);

        /* Request the new quotas */
        ReturnStatus = NtSetInformationProcess(
            hProcess,
            ProcessQuotaLimits,
            &mut QuotaLimits as *mut _ as PVOID,
            core::mem::size_of::<QUOTA_LIMITS_EX>() as ULONG,
        );
        Result = NT_SUCCESS(ReturnStatus) as BOOL;
        if NT_SUCCESS(Status) {
            /* Release the privilege and set succes code */
            debug_assert!(!State.is_null());
            RtlReleasePrivilege(State);
            State = core::ptr::null_mut();
        }
        let _ = State;
    } else {
        /* No limits, fail the call */
        ReturnStatus = STATUS_INVALID_PARAMETER;
        Result = FALSE;
    }

    /* Return result code, set error code if this was a failure */
    if Result == 0 {
        BaseSetLastNTError(ReturnStatus);
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessWorkingSetSize(
    hProcess: HANDLE,
    dwMinimumWorkingSetSize: SIZE_T,
    dwMaximumWorkingSetSize: SIZE_T,
) -> BOOL {
    /* Call the newer API */
    SetProcessWorkingSetSizeEx(
        hProcess,
        dwMinimumWorkingSetSize,
        dwMaximumWorkingSetSize,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessTimes(
    hProcess: HANDLE,
    lpCreationTime: LPFILETIME,
    lpExitTime: LPFILETIME,
    lpKernelTime: LPFILETIME,
    lpUserTime: LPFILETIME,
) -> BOOL {
    let mut Kut: KERNEL_USER_TIMES = core::mem::zeroed();

    /* Query the times */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessTimes,
        &mut Kut as *mut _ as PVOID,
        core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Handle failure */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Copy all the times and return success */
    (*lpCreationTime).dwLowDateTime = Kut.CreateTime.u.LowPart as DWORD;
    (*lpCreationTime).dwHighDateTime = Kut.CreateTime.u.HighPart as DWORD;
    (*lpExitTime).dwLowDateTime = Kut.ExitTime.u.LowPart as DWORD;
    (*lpExitTime).dwHighDateTime = Kut.ExitTime.u.HighPart as DWORD;
    (*lpKernelTime).dwLowDateTime = Kut.KernelTime.u.LowPart as DWORD;
    (*lpKernelTime).dwHighDateTime = Kut.KernelTime.u.HighPart as DWORD;
    (*lpUserTime).dwLowDateTime = Kut.UserTime.u.LowPart as DWORD;
    (*lpUserTime).dwHighDateTime = Kut.UserTime.u.HighPart as DWORD;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentProcess() -> HANDLE {
    NtCurrentProcess() as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentThread() -> HANDLE {
    NtCurrentThread() as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentProcessId() -> DWORD {
    HandleToUlong((*NtCurrentTeb()).ClientId.UniqueProcess)
}

#[no_mangle]
pub unsafe extern "system" fn GetExitCodeProcess(
    hProcess: HANDLE,
    lpExitCode: LPDWORD,
) -> BOOL {
    let mut ProcessBasic: PROCESS_BASIC_INFORMATION = core::mem::zeroed();

    /* Ask the kernel */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessBasicInformation,
        &mut ProcessBasic as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* We failed, was this because this is a VDM process? */
        if BaseCheckForVDM(hProcess, lpExitCode) != FALSE {
            return TRUE;
        }

        /* Not a VDM process, fail the call */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Succes case, return the exit code */
    *lpExitCode = ProcessBasic.ExitStatus as DWORD;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessId(Process: HANDLE) -> DWORD {
    let mut ProcessBasic: PROCESS_BASIC_INFORMATION = core::mem::zeroed();

    /* Query the kernel */
    let Status = NtQueryInformationProcess(
        Process,
        ProcessBasicInformation,
        &mut ProcessBasic as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Handle failure */
        BaseSetLastNTError(Status);
        return 0;
    }

    /* Return the PID */
    ProcessBasic.UniqueProcessId as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn OpenProcess(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    dwProcessId: DWORD,
) -> HANDLE {
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ClientId: CLIENT_ID = core::mem::zeroed();

    /* Setup the input client ID structure */
    ClientId.UniqueProcess = UlongToHandle(dwProcessId);
    ClientId.UniqueThread = core::ptr::null_mut();

    /* This is needed just to define the inheritance flags */
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        core::ptr::null_mut(),
        if bInheritHandle != 0 { OBJ_INHERIT } else { 0 },
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    /* Now try to open the process */
    let Status = NtOpenProcess(
        &mut ProcessHandle,
        dwDesiredAccess,
        &mut ObjectAttributes,
        &mut ClientId,
    );
    if !NT_SUCCESS(Status) {
        /* Handle failure */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Otherwise return a handle to the process */
    ProcessHandle
}

#[no_mangle]
pub unsafe extern "system" fn RegisterWaitForInputIdle(
    lpfnRegisterWaitForInputIdle: WaitForInputIdleType,
) {
    /* Write the global function pointer */
    UserWaitForInputIdleRoutine = lpfnRegisterWaitForInputIdle;
}

#[no_mangle]
pub unsafe extern "system" fn GetStartupInfoW(lpStartupInfo: LPSTARTUPINFOW) {
    /* Get the process parameters */
    let Params: PRTL_USER_PROCESS_PARAMETERS = (*NtCurrentPeb()).ProcessParameters;

    /* Copy the data out of there */
    (*lpStartupInfo).cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
    (*lpStartupInfo).lpReserved = (*Params).ShellInfo.Buffer;
    (*lpStartupInfo).lpDesktop = (*Params).DesktopInfo.Buffer;
    (*lpStartupInfo).lpTitle = (*Params).WindowTitle.Buffer;
    (*lpStartupInfo).dwX = (*Params).StartingX;
    (*lpStartupInfo).dwY = (*Params).StartingY;
    (*lpStartupInfo).dwXSize = (*Params).CountX;
    (*lpStartupInfo).dwYSize = (*Params).CountY;
    (*lpStartupInfo).dwXCountChars = (*Params).CountCharsX;
    (*lpStartupInfo).dwYCountChars = (*Params).CountCharsY;
    (*lpStartupInfo).dwFillAttribute = (*Params).FillAttribute;
    (*lpStartupInfo).dwFlags = (*Params).WindowFlags;
    (*lpStartupInfo).wShowWindow = (*Params).ShowWindowFlags as WORD;
    (*lpStartupInfo).cbReserved2 = (*Params).RuntimeData.Length;
    (*lpStartupInfo).lpReserved2 = (*Params).RuntimeData.Buffer as LPBYTE;

    /* Check if the standard handles are being used for other features */
    if ((*lpStartupInfo).dwFlags
        & (STARTF_USESTDHANDLES | STARTF_USEHOTKEY | STARTF_SHELLPRIVATE))
        != 0
    {
        /* These are, so copy the standard handles too */
        (*lpStartupInfo).hStdInput = (*Params).StandardInput;
        (*lpStartupInfo).hStdOutput = (*Params).StandardOutput;
        (*lpStartupInfo).hStdError = (*Params).StandardError;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetStartupInfoA(lpStartupInfo: LPSTARTUPINFOA) {
    let mut TitleString: ANSI_STRING = core::mem::zeroed();
    let mut ShellString: ANSI_STRING = core::mem::zeroed();
    let mut DesktopString: ANSI_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    /* Get the cached information as well as the PEB parameters */
    let mut StartupInfo: LPSTARTUPINFOA = BaseAnsiStartupInfo;
    let Params: PRTL_USER_PROCESS_PARAMETERS = (*NtCurrentPeb()).ProcessParameters;

    /* Check if this is the first time we have to get the cached version */
    while StartupInfo.is_null() {
        /* Create new ANSI startup info */
        StartupInfo = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            core::mem::size_of::<STARTUPINFOA>() as SIZE_T,
        ) as LPSTARTUPINFOA;
        if !StartupInfo.is_null() {
            /* Zero out string pointers in case we fail to create them */
            (*StartupInfo).lpReserved = core::ptr::null_mut();
            (*StartupInfo).lpDesktop = core::ptr::null_mut();
            (*StartupInfo).lpTitle = core::ptr::null_mut();

            /* Set the size */
            (*StartupInfo).cb = core::mem::size_of::<STARTUPINFOA>() as DWORD;

            /* Copy what's already stored in the PEB */
            (*StartupInfo).dwX = (*Params).StartingX;
            (*StartupInfo).dwY = (*Params).StartingY;
            (*StartupInfo).dwXSize = (*Params).CountX;
            (*StartupInfo).dwYSize = (*Params).CountY;
            (*StartupInfo).dwXCountChars = (*Params).CountCharsX;
            (*StartupInfo).dwYCountChars = (*Params).CountCharsY;
            (*StartupInfo).dwFillAttribute = (*Params).FillAttribute;
            (*StartupInfo).dwFlags = (*Params).WindowFlags;
            (*StartupInfo).wShowWindow = (*Params).ShowWindowFlags as WORD;
            (*StartupInfo).cbReserved2 = (*Params).RuntimeData.Length;
            (*StartupInfo).lpReserved2 = (*Params).RuntimeData.Buffer as LPBYTE;
            (*StartupInfo).hStdInput = (*Params).StandardInput;
            (*StartupInfo).hStdOutput = (*Params).StandardOutput;
            (*StartupInfo).hStdError = (*Params).StandardError;

            /* Copy shell info string */
            Status = RtlUnicodeStringToAnsiString(
                &mut ShellString,
                &mut (*Params).ShellInfo,
                TRUE as BOOLEAN,
            );
            if NT_SUCCESS(Status) {
                /* Save it */
                (*StartupInfo).lpReserved = ShellString.Buffer;

                /* Copy desktop info string */
                Status = RtlUnicodeStringToAnsiString(
                    &mut DesktopString,
                    &mut (*Params).DesktopInfo,
                    TRUE as BOOLEAN,
                );
                if NT_SUCCESS(Status) {
                    /* Save it */
                    (*StartupInfo).lpDesktop = DesktopString.Buffer;

                    /* Copy window title string */
                    Status = RtlUnicodeStringToAnsiString(
                        &mut TitleString,
                        &mut (*Params).WindowTitle,
                        TRUE as BOOLEAN,
                    );
                    if NT_SUCCESS(Status) {
                        /* Save it */
                        (*StartupInfo).lpTitle = TitleString.Buffer;

                        /* We finished with the ANSI version, try to cache it */
                        if InterlockedCompareExchangePointer(
                            &raw mut BaseAnsiStartupInfo as *mut _ as *mut PVOID,
                            StartupInfo as PVOID,
                            core::ptr::null_mut(),
                        )
                        .is_null()
                        {
                            /* We were the first thread through, use the data */
                            break;
                        }

                        /* Someone beat us to it, we will use their data instead */
                        Status = STATUS_SUCCESS;

                        /* We're going to free our own stuff, but not raise */
                        RtlFreeAnsiString(&mut TitleString);
                    }
                    RtlFreeAnsiString(&mut DesktopString);
                }
                RtlFreeAnsiString(&mut ShellString);
            }
            RtlFreeHeap(RtlGetProcessHeap(), 0, StartupInfo as PVOID);

            /* Get the cached information again: either still NULL or set by another thread */
            StartupInfo = BaseAnsiStartupInfo;
        } else {
            /* No memory, fail */
            Status = STATUS_NO_MEMORY;
        }

        /* Raise an error unless we got here due to the race condition */
        if StartupInfo.is_null() {
            RtlRaiseStatus(Status);
        }
    }

    /* Now copy from the cached ANSI version */
    (*lpStartupInfo).cb = (*StartupInfo).cb;
    (*lpStartupInfo).lpReserved = (*StartupInfo).lpReserved;
    (*lpStartupInfo).lpDesktop = (*StartupInfo).lpDesktop;
    (*lpStartupInfo).lpTitle = (*StartupInfo).lpTitle;
    (*lpStartupInfo).dwX = (*StartupInfo).dwX;
    (*lpStartupInfo).dwY = (*StartupInfo).dwY;
    (*lpStartupInfo).dwXSize = (*StartupInfo).dwXSize;
    (*lpStartupInfo).dwYSize = (*StartupInfo).dwYSize;
    (*lpStartupInfo).dwXCountChars = (*StartupInfo).dwXCountChars;
    (*lpStartupInfo).dwYCountChars = (*StartupInfo).dwYCountChars;
    (*lpStartupInfo).dwFillAttribute = (*StartupInfo).dwFillAttribute;
    (*lpStartupInfo).dwFlags = (*StartupInfo).dwFlags;
    (*lpStartupInfo).wShowWindow = (*StartupInfo).wShowWindow;
    (*lpStartupInfo).cbReserved2 = (*StartupInfo).cbReserved2;
    (*lpStartupInfo).lpReserved2 = (*StartupInfo).lpReserved2;

    /* Check if the shell is hijacking the handles for other features */
    if ((*lpStartupInfo).dwFlags
        & (STARTF_USESTDHANDLES | STARTF_USEHOTKEY | STARTF_SHELLPRIVATE))
        != 0
    {
        /* It isn't, so we can return the raw values */
        (*lpStartupInfo).hStdInput = (*StartupInfo).hStdInput;
        (*lpStartupInfo).hStdOutput = (*StartupInfo).hStdOutput;
        (*lpStartupInfo).hStdError = (*StartupInfo).hStdError;
    } else {
        /* It is, so make sure nobody uses these as console handles */
        (*lpStartupInfo).hStdInput = INVALID_HANDLE_VALUE;
        (*lpStartupInfo).hStdOutput = INVALID_HANDLE_VALUE;
        (*lpStartupInfo).hStdError = INVALID_HANDLE_VALUE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FlushInstructionCache(
    hProcess: HANDLE,
    lpBaseAddress: LPCVOID,
    nSize: SIZE_T,
) -> BOOL {
    /* Call the native function */
    let Status = NtFlushInstructionCache(hProcess, lpBaseAddress as PVOID, nSize);
    if !NT_SUCCESS(Status) {
        /* Handle failure case */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* All good */
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn ExitProcess(uExitCode: UINT) -> ! {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let ExitProcessRequest = &mut ApiMessage.Data.ExitProcessRequest;

    debug_assert!(BaseRunningInServerProcess == 0);

    'try_blk: {
        /* Acquire the PEB lock */
        RtlAcquirePebLock();

        /* Kill all the threads */
        NtTerminateProcess(core::ptr::null_mut(), uExitCode as NTSTATUS);

        /* Unload all DLLs */
        LdrShutdownProcess();

        /* Notify Base Server of process termination */
        (*ExitProcessRequest).uExitCode = uExitCode;
        CsrClientCallServer(
            &mut ApiMessage as *mut _ as PCSR_API_MESSAGE,
            core::ptr::null_mut(),
            CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepExitProcess),
            core::mem::size_of::<BASE_EXIT_PROCESS>() as ULONG,
        );

        /* Now do it again */
        NtTerminateProcess(NtCurrentProcess(), uExitCode as NTSTATUS);

        let _ = &ApiMessage;
        break 'try_blk;
    }
    /* _SEH2_FINALLY: Release the PEB lock */
    RtlReleasePebLock();

    /* should never get here */
    debug_assert!(false);
    loop {}
}

#[no_mangle]
pub unsafe extern "system" fn TerminateProcess(
    hProcess: HANDLE,
    uExitCode: UINT,
) -> BOOL {
    /* Check if no handle was passed in */
    if hProcess.is_null() {
        /* Set error code */
        SetLastError(ERROR_INVALID_HANDLE);
    } else {
        /* Otherwise, try to terminate the process */
        let Status = NtTerminateProcess(hProcess, uExitCode as NTSTATUS);
        if NT_SUCCESS(Status) {
            return TRUE;
        }

        /* It failed, convert error code */
        BaseSetLastNTError(Status);
    }

    /* This is the failure path */
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn FatalAppExitA(uAction: UINT, lpMessageText: LPCSTR) {
    /* Initialize the string using the static TEB pointer */
    let MessageTextU: PUNICODE_STRING = &mut (*NtCurrentTeb()).StaticUnicodeString;
    let mut MessageText: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&mut MessageText, lpMessageText as LPSTR);

    /* Convert to unicode, or just exit normally if this failed */
    let Status = RtlAnsiStringToUnicodeString(MessageTextU, &mut MessageText, FALSE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        ExitProcess(0);
    }

    /* Call the Wide function */
    FatalAppExitW(uAction, (*MessageTextU).Buffer);
}

#[no_mangle]
pub unsafe extern "system" fn FatalAppExitW(uAction: UINT, lpMessageText: LPCWSTR) {
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();
    let mut Response: ULONG = 0;

    /* Setup the string to print out */
    RtlInitUnicodeString(&mut UnicodeString, lpMessageText);

    /* Display the hard error no matter what */
    let Status = NtRaiseHardError(
        STATUS_FATAL_APP_EXIT | HARDERROR_OVERRIDE_ERRORMODE as NTSTATUS,
        1,
        1,
        &mut UnicodeString as *mut _ as PULONG_PTR,
        OptionOk,
        &mut Response,
    );

    /* Give the user a chance to abort */
    if NT_SUCCESS(Status) && (Response == ResponseCancel) {
        return;
    }

    /* Otherwise kill the process */
    ExitProcess(0);
}

#[no_mangle]
pub unsafe extern "system" fn FatalExit(ExitCode: core::ffi::c_int) {
    /* On non-checked builds (DBG==0), just kill the process */
    ExitProcess(ExitCode as UINT);
}

#[no_mangle]
pub unsafe extern "system" fn GetPriorityClass(hProcess: HANDLE) -> DWORD {
    // C: `PROCESS_PRIORITY_CLASS DECLSPEC_ALIGN(4) PriorityClass`. The kernel
    // probes this info class with ULONG alignment; the bare two-byte struct is
    // only guaranteed alignment 1 (STATUS_DATATYPE_MISALIGNMENT if unlucky).
    #[repr(C, align(4))]
    struct AlignedPriorityClass(PROCESS_PRIORITY_CLASS);
    let mut PriorityClass = AlignedPriorityClass(core::mem::zeroed());

    /* Query the kernel */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessPriorityClass,
        &mut PriorityClass.0 as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_PRIORITY_CLASS>() as ULONG,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        /* Handle the conversion from NT to Win32 classes */
        return match PriorityClass.0.PriorityClass as ULONG {
            PROCESS_PRIORITY_CLASS_IDLE => IDLE_PRIORITY_CLASS,
            PROCESS_PRIORITY_CLASS_BELOW_NORMAL => BELOW_NORMAL_PRIORITY_CLASS,
            PROCESS_PRIORITY_CLASS_ABOVE_NORMAL => ABOVE_NORMAL_PRIORITY_CLASS,
            PROCESS_PRIORITY_CLASS_HIGH => HIGH_PRIORITY_CLASS,
            PROCESS_PRIORITY_CLASS_REALTIME => REALTIME_PRIORITY_CLASS,
            _ => NORMAL_PRIORITY_CLASS,
        };
    }

    /* Failure path */
    BaseSetLastNTError(Status);
    0
}

#[no_mangle]
pub unsafe extern "system" fn SetPriorityClass(
    hProcess: HANDLE,
    dwPriorityClass: DWORD,
) -> BOOL {
    let mut State: PVOID = core::ptr::null_mut();
    let mut PriorityClass: PROCESS_PRIORITY_CLASS = core::mem::zeroed();

    /* Handle conversion from Win32 to NT priority classes */
    match dwPriorityClass {
        IDLE_PRIORITY_CLASS => {
            PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_IDLE as UCHAR;
        }
        BELOW_NORMAL_PRIORITY_CLASS => {
            PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_BELOW_NORMAL as UCHAR;
        }
        NORMAL_PRIORITY_CLASS => {
            PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_NORMAL as UCHAR;
        }
        ABOVE_NORMAL_PRIORITY_CLASS => {
            PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_ABOVE_NORMAL as UCHAR;
        }
        HIGH_PRIORITY_CLASS => {
            PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_HIGH as UCHAR;
        }
        REALTIME_PRIORITY_CLASS => {
            /* Try to acquire the privilege. If it fails, just use HIGH */
            State = BasepIsRealtimeAllowed(TRUE as BOOLEAN);
            PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_HIGH as UCHAR;
            PriorityClass.PriorityClass += (!State.is_null()) as UCHAR;
        }
        _ => {
            /* Unrecognized priority classes don't make it to the kernel */
            SetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    }

    /* Send the request to the kernel, and don't touch the foreground flag */
    PriorityClass.Foreground = FALSE as BOOLEAN;
    let Status = NtSetInformationProcess(
        hProcess,
        ProcessPriorityClass,
        &mut PriorityClass as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_PRIORITY_CLASS>() as ULONG,
    );

    /* Release the privilege if we had it */
    if !State.is_null() {
        RtlReleasePrivilege(State);
    }
    if !NT_SUCCESS(Status) {
        /* Handle error path */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* All done */
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessVersion(ProcessId: DWORD) -> DWORD {
    let mut Version: DWORD = 0;
    let mut NtHeader: PIMAGE_NT_HEADERS;
    let DosHeader: PIMAGE_DOS_HEADER;
    let Peb: PPEB;
    let mut ProcessBasicInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut BaseAddress: PVOID = core::ptr::null_mut();
    let mut e_lfanew: ULONG = 0;
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut VersionData: [USHORT; 2] = [0; 2];
    let mut Result: BOOLEAN;

    /* We'll be accessing stuff that can fault, so protect everything with SEH */
    'try_blk: {
        /* It this an in-process or out-of-process request? */
        if (ProcessId == 0) || (GetCurrentProcessId() == ProcessId) {
            /* It's in-process, so just read our own header */
            NtHeader = RtlImageNtHeader((*NtCurrentPeb()).ImageBaseAddress);
            if NtHeader.is_null() {
                /* Unable to read the NT header, something is wrong here... */
                Status = STATUS_INVALID_IMAGE_FORMAT;
                break 'try_blk; /* goto Error (in-process: no Error label exists,
                                   but Status set; FINALLY closes no handle) */
            }

            /* Get the version straight out of the NT header */
            Version = MAKELONG(
                (*NtHeader).OptionalHeader.MinorSubsystemVersion as WORD,
                (*NtHeader).OptionalHeader.MajorSubsystemVersion as WORD,
            );
        } else {
            /* Out-of-process, so open it */
            'error: {
                ProcessHandle = OpenProcess(
                    PROCESS_VM_READ | PROCESS_QUERY_INFORMATION,
                    FALSE,
                    ProcessId,
                );
                if ProcessHandle.is_null() {
                    /* _SEH2_YIELD(return 0): close handle (NULL) and return */
                    return 0;
                }

                /* Try to find out where its PEB lives */
                Status = NtQueryInformationProcess(
                    ProcessHandle,
                    ProcessBasicInformation,
                    &mut ProcessBasicInfo as *mut _ as PVOID,
                    core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
                    core::ptr::null_mut(),
                );

                if !NT_SUCCESS(Status) {
                    break 'error;
                }
                Peb = ProcessBasicInfo.PebBaseAddress;

                /* Now that we have the PEB, read the image base address out of it */
                Result = ReadProcessMemory(
                    ProcessHandle,
                    &(*Peb).ImageBaseAddress as *const _ as LPCVOID,
                    &mut BaseAddress as *mut _ as LPVOID,
                    core::mem::size_of::<PVOID>() as SIZE_T,
                    core::ptr::null_mut(),
                ) as BOOLEAN;
                if Result == 0 {
                    break 'error;
                }

                /* Now read the e_lfanew (offset to NT header) from the base */
                DosHeader = BaseAddress as PIMAGE_DOS_HEADER;
                Result = ReadProcessMemory(
                    ProcessHandle,
                    core::ptr::addr_of!((*DosHeader).e_lfanew) as LPCVOID,
                    &mut e_lfanew as *mut _ as LPVOID,
                    core::mem::size_of::<ULONG>() as SIZE_T,
                    core::ptr::null_mut(),
                ) as BOOLEAN;
                if Result == 0 {
                    break 'error;
                }

                /* And finally, read the NT header itself by adding the offset */
                NtHeader =
                    ((BaseAddress as ULONG_PTR) + e_lfanew as ULONG_PTR) as PIMAGE_NT_HEADERS;
                Result = ReadProcessMemory(
                    ProcessHandle,
                    &(*NtHeader).OptionalHeader.MajorSubsystemVersion as *const _ as LPCVOID,
                    &mut VersionData as *mut _ as LPVOID,
                    core::mem::size_of::<[USHORT; 2]>() as SIZE_T,
                    core::ptr::null_mut(),
                ) as BOOLEAN;
                if Result == 0 {
                    break 'error;
                }

                /* Get the version straight out of the NT header */
                Version = MAKELONG(VersionData[0] as WORD, VersionData[1] as WORD);
            }
            /* Error: */
            /* If there was an error anywhere, set the last error */
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
            }
        }
    }
    /* _SEH2_FINALLY: Close the process handle */
    if !ProcessHandle.is_null() {
        CloseHandle(ProcessHandle);
    }

    /* And return the version data */
    Version
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessIoCounters(
    hProcess: HANDLE,
    lpIoCounters: PIO_COUNTERS,
) -> BOOL {
    /* Query the kernel. Structures are identical, so let it do the copy too. */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessIoCounters,
        lpIoCounters as PVOID,
        core::mem::size_of::<IO_COUNTERS>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Handle error path */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* All done */
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessPriorityBoost(
    hProcess: HANDLE,
    pDisablePriorityBoost: PBOOL,
) -> BOOL {
    let mut PriorityBoost: ULONG = 0;

    /* Query the kernel */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessPriorityBoost,
        &mut PriorityBoost as *mut _ as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        /* Convert from ULONG to a BOOL */
        *pDisablePriorityBoost = if PriorityBoost != 0 { TRUE } else { FALSE };
        return TRUE;
    }

    /* Handle error path */
    BaseSetLastNTError(Status);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessPriorityBoost(
    hProcess: HANDLE,
    bDisablePriorityBoost: BOOL,
) -> BOOL {
    /* Enforce that this is a BOOL, and send it to the kernel as a ULONG */
    let PriorityBoost: ULONG = if bDisablePriorityBoost != 0 {
        TRUE as ULONG
    } else {
        FALSE as ULONG
    };
    let Status = NtSetInformationProcess(
        hProcess,
        ProcessPriorityBoost,
        &PriorityBoost as *const _ as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        /* Handle error path */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* All done */
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessHandleCount(
    hProcess: HANDLE,
    pdwHandleCount: PDWORD,
) -> BOOL {
    let mut phc: ULONG = 0;

    /* Query the kernel */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessHandleCount,
        &mut phc as *mut _ as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        /* Copy the count and return success */
        *pdwHandleCount = phc;
        return TRUE;
    }

    /* Handle error path */
    BaseSetLastNTError(Status);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn IsWow64Process(
    hProcess: HANDLE,
    Wow64Process: PBOOL,
) -> BOOL {
    let mut pbi: ULONG_PTR = 0;

    /* Query the kernel */
    let Status = NtQueryInformationProcess(
        hProcess,
        ProcessWow64Information,
        &mut pbi as *mut _ as PVOID,
        core::mem::size_of::<ULONG_PTR>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Handle error path */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Enforce this is a BOOL, and return success */
    *Wow64Process = (pbi != 0) as BOOL;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetCommandLineA() -> LPSTR {
    BaseAnsiCommandLine.Buffer
}

#[no_mangle]
pub unsafe extern "system" fn GetCommandLineW() -> LPWSTR {
    BaseUnicodeCommandLine.Buffer
}

#[no_mangle]
pub unsafe extern "system" fn ReadProcessMemory(
    hProcess: HANDLE,
    lpBaseAddress: LPCVOID,
    lpBuffer: LPVOID,
    nSize: SIZE_T,
    lpNumberOfBytesRead: *mut SIZE_T,
) -> BOOL {
    let mut nSize = nSize;

    /* Do the read */
    let Status = NtReadVirtualMemory(
        hProcess,
        lpBaseAddress as PVOID,
        lpBuffer,
        nSize,
        &mut nSize,
    );

    /* In user-mode, this parameter is optional */
    if !lpNumberOfBytesRead.is_null() {
        *lpNumberOfBytesRead = nSize;
    }
    if !NT_SUCCESS(Status) {
        /* We failed */
        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Return success */
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn WriteProcessMemory(
    hProcess: HANDLE,
    lpBaseAddress: LPVOID,
    lpBuffer: LPCVOID,
    nSize: SIZE_T,
    lpNumberOfBytesWritten: *mut SIZE_T,
) -> BOOL {
    let mut nSize = nSize;
    let mut OldValue: ULONG = 0;
    let UnProtect: BOOLEAN;

    /* Set parameters for protect call */
    let mut RegionSize: SIZE_T = nSize;
    let mut Base: PVOID = lpBaseAddress;

    /* Check the current status */
    let mut Status = NtProtectVirtualMemory(
        hProcess,
        &mut Base,
        &mut RegionSize,
        PAGE_EXECUTE_READWRITE,
        &mut OldValue,
    );
    if NT_SUCCESS(Status) {
        /* Check if we are unprotecting */
        UnProtect = if OldValue
            & (PAGE_READWRITE
                | PAGE_WRITECOPY
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY)
            != 0
        {
            FALSE as BOOLEAN
        } else {
            TRUE as BOOLEAN
        };
        if UnProtect == 0 {
            /* Set the new protection */
            Status = NtProtectVirtualMemory(
                hProcess,
                &mut Base,
                &mut RegionSize,
                OldValue,
                &mut OldValue,
            );

            /* Write the memory */
            Status = NtWriteVirtualMemory(
                hProcess,
                lpBaseAddress,
                lpBuffer as LPVOID,
                nSize,
                &mut nSize,
            );

            /* In Win32, the parameter is optional, so handle this case */
            if !lpNumberOfBytesWritten.is_null() {
                *lpNumberOfBytesWritten = nSize;
            }

            if !NT_SUCCESS(Status) {
                /* We failed */
                BaseSetLastNTError(Status);
                return FALSE;
            }

            /* Flush the ITLB */
            NtFlushInstructionCache(hProcess, lpBaseAddress, nSize);
            TRUE
        } else {
            /* Check if we were read only */
            if OldValue & (PAGE_NOACCESS | PAGE_READONLY) != 0 {
                /* Restore protection and fail */
                NtProtectVirtualMemory(
                    hProcess,
                    &mut Base,
                    &mut RegionSize,
                    OldValue,
                    &mut OldValue,
                );
                BaseSetLastNTError(STATUS_ACCESS_VIOLATION);

                /* Note: This is what Windows returns and code depends on it */
                return STATUS_ACCESS_VIOLATION as BOOL;
            }

            /* Otherwise, do the write */
            Status = NtWriteVirtualMemory(
                hProcess,
                lpBaseAddress,
                lpBuffer as LPVOID,
                nSize,
                &mut nSize,
            );

            /* In Win32, the parameter is optional, so handle this case */
            if !lpNumberOfBytesWritten.is_null() {
                *lpNumberOfBytesWritten = nSize;
            }

            /* And restore the protection */
            NtProtectVirtualMemory(
                hProcess,
                &mut Base,
                &mut RegionSize,
                OldValue,
                &mut OldValue,
            );
            if !NT_SUCCESS(Status) {
                /* We failed */
                BaseSetLastNTError(STATUS_ACCESS_VIOLATION);

                /* Note: This is what Windows returns and code depends on it */
                return STATUS_ACCESS_VIOLATION as BOOL;
            }

            /* Flush the ITLB */
            NtFlushInstructionCache(hProcess, lpBaseAddress, nSize);
            TRUE
        }
    } else {
        /* We failed */
        BaseSetLastNTError(Status);
        FALSE
    }
}

#[no_mangle]
pub unsafe extern "system" fn ProcessIdToSessionId(
    dwProcessId: DWORD,
    pSessionId: PDWORD,
) -> BOOL {
    let mut SessionInformation: PROCESS_SESSION_INFORMATION = core::mem::zeroed();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ClientId: CLIENT_ID = core::mem::zeroed();
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    /* Do a quick check if the pointer is not writable */
    if IsBadWritePtr(pSessionId as LPVOID, core::mem::size_of::<DWORD>() as UINT_PTR) != 0 {
        /* Fail fast */
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    /* Open the process passed in by ID */
    ClientId.UniqueProcess = UlongToHandle(dwProcessId);
    ClientId.UniqueThread = core::ptr::null_mut();
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
        &mut ClientId,
    );
    if NT_SUCCESS(Status) {
        /* Query the session ID from the kernel */
        Status = NtQueryInformationProcess(
            ProcessHandle,
            ProcessSessionInformation,
            &mut SessionInformation as *mut _ as PVOID,
            core::mem::size_of::<PROCESS_SESSION_INFORMATION>() as ULONG,
            core::ptr::null_mut(),
        );

        /* Close the handle and check if we succeeded */
        NtClose(ProcessHandle);
        if NT_SUCCESS(Status) {
            /* Return the session ID */
            *pSessionId = SessionInformation.SessionId;
            return TRUE;
        }
    }

    /* Set error code and fail */
    BaseSetLastNTError(Status);
    FALSE
}
