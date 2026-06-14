use crate::substrate::*;

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }

/* ReactOS-specific name STARTF_SHELLPRIVATE == STARTF_HASSHELLDATA (0x400);
 * not present in the substrate, defined here from the C header value. */
const STARTF_SHELLPRIVATE: u32 = 0x00000400;

#[no_mangle]
pub unsafe extern "system" fn BasePushProcessParameters(
    ParameterFlags: ULONG,
    ProcessHandle: HANDLE,
    RemotePeb: PPEB,
    ApplicationPathName: LPCWSTR,
    lpCurrentDirectory: LPWSTR,
    lpCommandLine: LPWSTR,
    lpEnvironment: LPVOID,
    StartupInfo: LPSTARTUPINFOW,
    CreationFlags: DWORD,
    InheritHandles: BOOL,
    ImageSubsystem: ULONG,
    AppCompatData: PVOID,
    AppCompatDataSize: ULONG,
) -> BOOLEAN {
    let mut FullPath = [0u16; MAX_PATH as usize + 5];
    let mut Remaining: PWCHAR = core::ptr::null_mut();
    let DllPathString: PWCHAR;
    let mut ScanChar: PWCHAR;
    let mut ProcessParameters: PRTL_USER_PROCESS_PARAMETERS = core::ptr::null_mut();
    let mut RemoteParameters: PRTL_USER_PROCESS_PARAMETERS;
    let mut RemoteAppCompatData: PVOID;
    let mut DllPath: UNICODE_STRING = core::mem::zeroed();
    let mut ImageName: UNICODE_STRING = core::mem::zeroed();
    let mut CommandLine: UNICODE_STRING = core::mem::zeroed();
    let mut CurrentDirectory: UNICODE_STRING = core::mem::zeroed();
    let mut Desktop: UNICODE_STRING = core::mem::zeroed();
    let mut Shell: UNICODE_STRING = core::mem::zeroed();
    let mut Runtime: UNICODE_STRING = core::mem::zeroed();
    let mut Title: UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let EnviroSize: ULONG;
    let mut Size: SIZE_T;
    let mut HavePebLock: BOOLEAN = FALSE as BOOLEAN;
    let mut Result: BOOLEAN = FALSE as BOOLEAN;
    let Peb: PPEB = NtCurrentPeb();

    /* Empty wide string for the L"" cases */
    let empty_w: [u16; 1] = [0];

    /* Local environment working pointer (the C reassigns the LPVOID param) */
    let mut lpEnvironment = lpEnvironment;

    /* Get the full path name */
    Size = GetFullPathNameW(
        ApplicationPathName,
        MAX_PATH + 4,
        FullPath.as_mut_ptr(),
        &mut Remaining,
    ) as SIZE_T;
    if (Size != 0) && (Size <= (MAX_PATH as SIZE_T + 4)) {
        /* Get the DLL Path */
        DllPathString = BaseComputeProcessDllPath(FullPath.as_mut_ptr(), lpEnvironment);
        if DllPathString.is_null() {
            /* Fail */
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return FALSE as BOOLEAN;
        }

        /* Initialize Strings */
        RtlInitUnicodeString(&mut DllPath, DllPathString);
        RtlInitUnicodeString(&mut ImageName, FullPath.as_ptr());
    } else {
        /* Couldn't get the path name. Just take the original path */
        DllPathString = BaseComputeProcessDllPath(ApplicationPathName as LPWSTR, lpEnvironment);
        if DllPathString.is_null() {
            /* Fail */
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return FALSE as BOOLEAN;
        }

        /* Initialize Strings */
        RtlInitUnicodeString(&mut DllPath, DllPathString);
        RtlInitUnicodeString(&mut ImageName, ApplicationPathName);
    }

    /* Initialize Strings */
    RtlInitUnicodeString(&mut CommandLine, lpCommandLine);
    RtlInitUnicodeString(&mut CurrentDirectory, lpCurrentDirectory);

    /* Initialize more Strings from the Startup Info */
    if !(*StartupInfo).lpDesktop.is_null() {
        RtlInitUnicodeString(&mut Desktop, (*StartupInfo).lpDesktop);
    } else {
        RtlInitUnicodeString(&mut Desktop, empty_w.as_ptr());
    }
    if !(*StartupInfo).lpReserved.is_null() {
        RtlInitUnicodeString(&mut Shell, (*StartupInfo).lpReserved);
    } else {
        RtlInitUnicodeString(&mut Shell, empty_w.as_ptr());
    }
    if !(*StartupInfo).lpTitle.is_null() {
        RtlInitUnicodeString(&mut Title, (*StartupInfo).lpTitle);
    } else {
        RtlInitUnicodeString(&mut Title, ApplicationPathName);
    }

    /* This one is special because the length can differ */
    Runtime.Buffer = (*StartupInfo).lpReserved2 as LPWSTR;
    Runtime.Length = (*StartupInfo).cbReserved2;
    Runtime.MaximumLength = (*StartupInfo).cbReserved2;

    /* Enforce no app compat data if the pointer was NULL */
    let mut AppCompatDataSize = AppCompatDataSize;
    if AppCompatData.is_null() {
        AppCompatDataSize = 0;
    }

    /* Create the Parameter Block */
    ProcessParameters = core::ptr::null_mut();

    'quickie: {
        Status = RtlCreateProcessParameters(
            &mut ProcessParameters,
            &mut ImageName,
            &mut DllPath,
            if !lpCurrentDirectory.is_null() {
                &mut CurrentDirectory
            } else {
                core::ptr::null_mut()
            },
            &mut CommandLine,
            lpEnvironment as *mut u16,
            &mut Title,
            &mut Desktop,
            &mut Shell,
            &mut Runtime,
        );
        if !NT_SUCCESS(Status) {
            /* FailPath */
            BaseSetLastNTError(Status);
            Result = FALSE as BOOLEAN;
            break 'quickie;
        }

        /* Clear the current directory handle if not inheriting */
        if InheritHandles == 0 {
            (*ProcessParameters).CurrentDirectory.Handle = core::ptr::null_mut();
        }

        /* Check if the user passed in an environment */
        if !lpEnvironment.is_null() {
            /* We should've made it part of the parameters block, enforce this */
            debug_assert!((*ProcessParameters).Environment as PVOID == lpEnvironment);
            lpEnvironment = (*ProcessParameters).Environment as LPVOID;
        } else {
            /* The user did not, so use the one from the current PEB */
            HavePebLock = TRUE as BOOLEAN;
            RtlAcquirePebLock();
            lpEnvironment = (*(*Peb).ProcessParameters).Environment as LPVOID;
        }

        /* Save pointer and start lookup */
        ScanChar = lpEnvironment as PWCHAR;
        if !lpEnvironment.is_null() {
            /* Find the environment size */
            /* while (*ScanChar++) while (*ScanChar++); */
            while {
                let c = *ScanChar;
                ScanChar = ScanChar.add(1);
                c != 0
            } {
                while {
                    let c = *ScanChar;
                    ScanChar = ScanChar.add(1);
                    c != 0
                } {}
            }
            EnviroSize = (ScanChar as ULONG_PTR - lpEnvironment as ULONG_PTR) as ULONG;

            /* Allocate and Initialize new Environment Block */
            Size = EnviroSize as SIZE_T;
            (*ProcessParameters).Environment = core::ptr::null_mut();
            Status = NtAllocateVirtualMemory(
                ProcessHandle,
                &mut (*ProcessParameters).Environment as *mut PWSTR as *mut PVOID,
                0,
                &mut Size,
                MEM_COMMIT,
                PAGE_READWRITE,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                Result = FALSE as BOOLEAN;
                break 'quickie;
            }

            /* Write the Environment Block */
            Status = NtWriteVirtualMemory(
                ProcessHandle,
                (*ProcessParameters).Environment as PVOID,
                lpEnvironment,
                EnviroSize as SIZE_T,
                core::ptr::null_mut(),
            );

            /* No longer need the PEB lock anymore */
            if HavePebLock != 0 {
                /* Release it */
                RtlReleasePebLock();
                HavePebLock = FALSE as BOOLEAN;
            }

            /* Check if the write failed */
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                Result = FALSE as BOOLEAN;
                break 'quickie;
            }
        }

        /* Write new parameters */
        (*ProcessParameters).StartingX = (*StartupInfo).dwX;
        (*ProcessParameters).StartingY = (*StartupInfo).dwY;
        (*ProcessParameters).CountX = (*StartupInfo).dwXSize;
        (*ProcessParameters).CountY = (*StartupInfo).dwYSize;
        (*ProcessParameters).CountCharsX = (*StartupInfo).dwXCountChars;
        (*ProcessParameters).CountCharsY = (*StartupInfo).dwYCountChars;
        (*ProcessParameters).FillAttribute = (*StartupInfo).dwFillAttribute;
        (*ProcessParameters).WindowFlags = (*StartupInfo).dwFlags;
        (*ProcessParameters).ShowWindowFlags = (*StartupInfo).wShowWindow as ULONG;

        /* Write the handles only if we have to */
        if (*StartupInfo).dwFlags
            & (STARTF_USESTDHANDLES | STARTF_USEHOTKEY | STARTF_SHELLPRIVATE)
            != 0
        {
            (*ProcessParameters).StandardInput = (*StartupInfo).hStdInput;
            (*ProcessParameters).StandardOutput = (*StartupInfo).hStdOutput;
            (*ProcessParameters).StandardError = (*StartupInfo).hStdError;
        }

        /* Use Special Flags for ConDllInitialize in Kernel32 */
        if CreationFlags & DETACHED_PROCESS != 0 {
            (*ProcessParameters).ConsoleHandle = HANDLE_DETACHED_PROCESS;
        } else if CreationFlags & CREATE_NEW_CONSOLE != 0 {
            (*ProcessParameters).ConsoleHandle = HANDLE_CREATE_NEW_CONSOLE;
        } else if CreationFlags & CREATE_NO_WINDOW != 0 {
            (*ProcessParameters).ConsoleHandle = HANDLE_CREATE_NO_WINDOW;
        } else {
            /* Inherit our Console Handle */
            (*ProcessParameters).ConsoleHandle = (*(*Peb).ProcessParameters).ConsoleHandle;

            /* Make sure that the shell isn't trampling on our handles first */
            if (*StartupInfo).dwFlags
                & (STARTF_USESTDHANDLES | STARTF_USEHOTKEY | STARTF_SHELLPRIVATE)
                == 0
            {
                /* Copy the handle if we are inheriting or if it's a console handle */
                if (InheritHandles != 0)
                    || IsConsoleHandle((*(*Peb).ProcessParameters).StandardInput)
                {
                    (*ProcessParameters).StandardInput =
                        (*(*Peb).ProcessParameters).StandardInput;
                }
                if (InheritHandles != 0)
                    || IsConsoleHandle((*(*Peb).ProcessParameters).StandardOutput)
                {
                    (*ProcessParameters).StandardOutput =
                        (*(*Peb).ProcessParameters).StandardOutput;
                }
                if (InheritHandles != 0)
                    || IsConsoleHandle((*(*Peb).ProcessParameters).StandardError)
                {
                    (*ProcessParameters).StandardError =
                        (*(*Peb).ProcessParameters).StandardError;
                }
            }
        }

        /* Also set the Console Flag */
        if (CreationFlags & CREATE_NEW_PROCESS_GROUP != 0)
            && (CreationFlags & CREATE_NEW_CONSOLE == 0)
        {
            (*ProcessParameters).ConsoleFlags = 1;
        }

        /* Check if there's a .local file present */
        if ParameterFlags & 1 != 0 {
            (*ProcessParameters).Flags |= RTL_USER_PROCESS_PARAMETERS_LOCAL_DLL_PATH;
        }

        /* Check if we failed to open the IFEO key */
        if ParameterFlags & 2 != 0 {
            (*ProcessParameters).Flags |= RTL_USER_PROCESS_PARAMETERS_IMAGE_KEY_MISSING;
        }

        /* Allocate memory for the parameter block */
        Size = (*ProcessParameters).Length as SIZE_T;
        RemoteParameters = core::ptr::null_mut();
        Status = NtAllocateVirtualMemory(
            ProcessHandle,
            &mut RemoteParameters as *mut PRTL_USER_PROCESS_PARAMETERS as *mut PVOID,
            0,
            &mut Size,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Result = FALSE as BOOLEAN;
            break 'quickie;
        }

        /* Set the allocated size */
        (*ProcessParameters).MaximumLength = Size as ULONG;

        /* Handle some Parameter Flags */
        (*ProcessParameters).Flags |= if CreationFlags & PROFILE_USER != 0 {
            RTL_USER_PROCESS_PARAMETERS_PROFILE_USER
        } else {
            0
        };
        (*ProcessParameters).Flags |= if CreationFlags & PROFILE_KERNEL != 0 {
            RTL_USER_PROCESS_PARAMETERS_PROFILE_KERNEL
        } else {
            0
        };
        (*ProcessParameters).Flags |= if CreationFlags & PROFILE_SERVER != 0 {
            RTL_USER_PROCESS_PARAMETERS_PROFILE_SERVER
        } else {
            0
        };
        (*ProcessParameters).Flags |= (*(*Peb).ProcessParameters).Flags
            & RTL_USER_PROCESS_PARAMETERS_DISABLE_HEAP_CHECKS;

        /* Write the Parameter Block */
        Status = NtWriteVirtualMemory(
            ProcessHandle,
            RemoteParameters as PVOID,
            ProcessParameters as PVOID,
            (*ProcessParameters).Length as SIZE_T,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Result = FALSE as BOOLEAN;
            break 'quickie;
        }

        /* Write the PEB Pointer */
        Status = NtWriteVirtualMemory(
            ProcessHandle,
            &mut (*RemotePeb).ProcessParameters as *mut _ as PVOID,
            &mut RemoteParameters as *mut _ as PVOID,
            core::mem::size_of::<PVOID>() as SIZE_T,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Result = FALSE as BOOLEAN;
            break 'quickie;
        }

        /* Check if there's any app compat data to write */
        RemoteAppCompatData = core::ptr::null_mut();
        if !AppCompatData.is_null() {
            /* Allocate some space for the application compatibility data */
            Size = AppCompatDataSize as SIZE_T;
            Status = NtAllocateVirtualMemory(
                ProcessHandle,
                &mut RemoteAppCompatData,
                0,
                &mut Size,
                MEM_COMMIT,
                PAGE_READWRITE,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                Result = FALSE as BOOLEAN;
                break 'quickie;
            }

            /* Write the application compatibility data */
            Status = NtWriteVirtualMemory(
                ProcessHandle,
                RemoteAppCompatData,
                AppCompatData,
                AppCompatDataSize as SIZE_T,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                Result = FALSE as BOOLEAN;
                break 'quickie;
            }
        }

        /* Write the PEB Pointer to the app compat data (might be NULL) */
        Status = NtWriteVirtualMemory(
            ProcessHandle,
            &mut (*RemotePeb).pShimData as *mut _ as PVOID,
            &mut RemoteAppCompatData as *mut _ as PVOID,
            core::mem::size_of::<PVOID>() as SIZE_T,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Result = FALSE as BOOLEAN;
            break 'quickie;
        }

        /* Now write Peb->ImageSubSystem */
        if ImageSubsystem != 0 {
            let mut ImageSubsystem = ImageSubsystem;
            NtWriteVirtualMemory(
                ProcessHandle,
                &mut (*RemotePeb).ImageSubsystem as *mut _ as PVOID,
                &mut ImageSubsystem as *mut _ as PVOID,
                core::mem::size_of::<ULONG>() as SIZE_T,
                core::ptr::null_mut(),
            );
        }

        /* Success path */
        Result = TRUE as BOOLEAN;
    }

    /* Quickie: Cleanup */
    if HavePebLock != 0 {
        RtlReleasePebLock();
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, DllPath.Buffer as PVOID);
    if !ProcessParameters.is_null() {
        RtlDestroyProcessParameters(ProcessParameters);
    }
    Result
}
