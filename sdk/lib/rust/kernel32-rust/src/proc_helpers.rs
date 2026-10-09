//! proc.c — internal helper functions (hand-translated from C).
//!
//! Covers: StuffStdHandle, BuildSubSysCommandLine, BasepIsImageVersionOk,
//! BasepCheckWebBladeHashes, BasepSaveAppCertRegistryValue,
//! BasepConfigureAppCertDlls, BasepIsProcessAllowed,
//! BasepReplaceProcessThreadTokens, BasepSxsCloseHandles, BaseProcessStartup,
//! InitCommandLines.

use crate::substrate::*;

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ---------------------------------------------------------------------------
// StuffStdHandle
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn StuffStdHandle(
    ProcessHandle: HANDLE,
    StandardHandle: HANDLE,
    Address: PHANDLE,
) {
    let mut DuplicatedHandle: HANDLE = core::ptr::null_mut();
    let mut NumberOfBytesWritten: SIZE_T = 0;

    /* If there is no handle to duplicate, return immediately */
    if StandardHandle.is_null() {
        return;
    }

    /* Duplicate the handle */
    let Status = NtDuplicateObject(
        NtCurrentProcess(),
        StandardHandle,
        ProcessHandle,
        &mut DuplicatedHandle,
        0,
        0,
        DUPLICATE_SAME_ACCESS | DUPLICATE_SAME_ATTRIBUTES,
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    /* Write it */
    NtWriteVirtualMemory(
        ProcessHandle,
        Address as PVOID,
        &mut DuplicatedHandle as *mut HANDLE as PVOID,
        core::mem::size_of::<HANDLE>() as SIZE_T,
        &mut NumberOfBytesWritten,
    );
}

// ---------------------------------------------------------------------------
// BuildSubSysCommandLine
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BuildSubSysCommandLine(
    SubsystemName: LPCWSTR,
    ApplicationName: LPCWSTR,
    CommandLine: LPCWSTR,
    SubsysCommandLine: PUNICODE_STRING,
) -> BOOLEAN {
    let mut CommandLineString: UNICODE_STRING = core::mem::zeroed();
    let mut ApplicationNameString: UNICODE_STRING = core::mem::zeroed();

    /* Convert to unicode strings (note: C swaps the args here) */
    RtlInitUnicodeString(&mut CommandLineString, ApplicationName);
    RtlInitUnicodeString(&mut ApplicationNameString, CommandLine);

    /* Allocate buffer for the output string */
    let Length: ULONG = CommandLineString.MaximumLength as ULONG
        + ApplicationNameString.MaximumLength as ULONG
        + 32;
    let Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length as SIZE_T) as PWCHAR;
    RtlInitEmptyUnicodeString(SubsysCommandLine, Buffer, Length as USHORT);
    if Buffer.is_null() {
        /* Fail, no memory */
        BaseSetLastNTError(STATUS_NO_MEMORY);
        return FALSE as BOOLEAN;
    }

    /* Build the final subsystem command line */
    RtlAppendUnicodeToString(SubsysCommandLine, SubsystemName);
    RtlAppendUnicodeStringToString(SubsysCommandLine, &mut CommandLineString);
    let slash_c: [u16; 5] = [b' ' as u16, b'/' as u16, b'C' as u16, b' ' as u16, 0];
    RtlAppendUnicodeToString(SubsysCommandLine, slash_c.as_ptr());
    RtlAppendUnicodeStringToString(SubsysCommandLine, &mut ApplicationNameString);
    TRUE as BOOLEAN
}

// ---------------------------------------------------------------------------
// BasepIsImageVersionOk
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BasepIsImageVersionOk(
    ImageMajorVersion: ULONG,
    ImageMinorVersion: ULONG,
) -> BOOLEAN {
    /* Accept images for NT 3.1 or higher */
    if ImageMajorVersion > 3 || (ImageMajorVersion == 3 && ImageMinorVersion >= 10) {
        /* ReactOS-specific: accept images even if newer than our internal NT version. */
        // DPRINT1 dropped (NDEBUG)
        return TRUE as BOOLEAN;
    }

    FALSE as BOOLEAN
}

// ---------------------------------------------------------------------------
// BasepCheckWebBladeHashes
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BasepCheckWebBladeHashes(FileHandle: HANDLE) -> NTSTATUS {
    let mut Hash: [CHAR; 16] = [0; 16];

    /* Get all the MD5 hashes */
    let Status = RtlComputeImportTableHash(FileHandle, Hash.as_mut_ptr(), 1);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Depending on suite, run a bsearch and block the appropriate ones (no-op) */
    // DPRINT1 cases dropped (NDEBUG)
    let _ = SharedUserData_SuiteMask();

    /* Actually, don't block anything, we're open source */
    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// BasepSaveAppCertRegistryValue
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BasepSaveAppCertRegistryValue(
    _List: PLIST_ENTRY,
    _ComponentName: PWCHAR,
    _DllName: PWCHAR,
) -> NTSTATUS {
    /* Pretty much the only thing this key is used for, is malware */
    // UNIMPLEMENTED
    STATUS_NOT_IMPLEMENTED
}

// ---------------------------------------------------------------------------
// BasepConfigureAppCertDlls
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BasepConfigureAppCertDlls(
    ValueName: PWSTR,
    _ValueType: ULONG,
    ValueData: PVOID,
    _ValueLength: ULONG,
    Context: PVOID,
    _EntryContext: PVOID,
) -> NTSTATUS {
    /* Add this to the certification list */
    BasepSaveAppCertRegistryValue(
        Context as PLIST_ENTRY,
        ValueName,
        ValueData as PWCHAR,
    )
}

// ---------------------------------------------------------------------------
// BasepIsProcessAllowed
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BasepIsProcessAllowed(ApplicationName: LPWSTR) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PWCHAR;
    let mut Length: UINT;
    let mut TrustLibrary: HMODULE;
    let mut Entry: PBASEP_APPCERT_ENTRY;
    let mut CertFlag: ULONG;
    let mut NextEntry: PLIST_ENTRY;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    // CertKey = L"\Registry\MACHINE\System\CurrentControlSet\Control\Session Manager\AppCertDlls"
    let cert_key_buf: [u16; 79] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'A' as u16,
        b'C' as u16, b'H' as u16, b'I' as u16, b'N' as u16, b'E' as u16, b'\\' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
        b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
        b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16,
        b'o' as u16, b'l' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b's' as u16,
        b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b' ' as u16, b'M' as u16,
        b'a' as u16, b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16,
        b'\\' as u16, b'A' as u16, b'p' as u16, b'p' as u16, b'C' as u16, b'e' as u16,
        b'r' as u16, b't' as u16, b'D' as u16, b'l' as u16, b'l' as u16, b's' as u16,
        0,
    ];
    let mut CertKey: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut CertKey, cert_key_buf.as_ptr());

    let mut KeyAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    KeyAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    KeyAttributes.ObjectName = &mut CertKey;
    KeyAttributes.Attributes = OBJ_CASE_INSENSITIVE;

    /* Try to initialize the certification subsystem */
    while g_AppCertInitialized == 0 {
        /* Defaults */
        Status = STATUS_SUCCESS;
        Buffer = core::ptr::null_mut();

        /* Acquire the lock while initializing and see if we lost a race */
        RtlEnterCriticalSection(&raw mut gcsAppCert);
        if g_AppCertInitialized != 0 {
            break;
        }

        /* On embedded, there is a special DLL */
        if (SharedUserData_SuiteMask() & VER_SUITE_EMBEDDEDNT) != 0 {
            /* Allocate a buffer for the name */
            Buffer = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                (MAX_PATH as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T
                    + core::mem::size_of::<WCHAR>() as SIZE_T, // sizeof(UNICODE_NULL)
            ) as PWCHAR;
            if Buffer.is_null() {
                /* Fail if no memory */
                Status = STATUS_NO_MEMORY;
            } else {
                /* sizeof("EmbdTrst.DLL") == 13 (bytes, incl NUL) */
                const EMBD_SZ: UINT = 13;
                /* Now get the system32 dir into our buffer, make sure it fits */
                Length = GetSystemDirectoryW(Buffer, MAX_PATH - EMBD_SZ);
                if Length != 0 && Length <= MAX_PATH - EMBD_SZ {
                    /* Add a slash if needed, and add the embedded cert DLL name */
                    if *Buffer.add((Length - 1) as usize) != b'\\' as u16 {
                        *Buffer.add(Length as usize) = b'\\' as u16;
                        Length += 1;
                    }
                    // L"EmbdTrst.DLL" + NUL == 13 u16 units
                    let embd: [u16; 13] = [
                        b'E' as u16, b'm' as u16, b'b' as u16, b'd' as u16, b'T' as u16,
                        b'r' as u16, b's' as u16, b't' as u16, b'.' as u16, b'D' as u16,
                        b'L' as u16, b'L' as u16, 0,
                    ];
                    RtlCopyMemory(
                        Buffer.add(Length as usize) as PVOID,
                        embd.as_ptr() as *const core::ffi::c_void as PVOID,
                        core::mem::size_of_val(&embd) as SIZE_T,
                    );

                    /* Try to load it */
                    TrustLibrary = LoadLibraryW(Buffer);
                    if !TrustLibrary.is_null() {
                        /* And extract the special function out of it */
                        let name = b"ImageOkToRunOnEmbeddedNT\0";
                        let proc = GetProcAddress(TrustLibrary, name.as_ptr() as LPCSTR);
                        fEmbeddedCertFunc =
                            core::mem::transmute::<FARPROC, PBASEP_APPCERT_EMBEDDED_FUNC>(proc);
                    }
                }

                /* If we didn't get this far, set a failure code */
                if (*(&raw const fEmbeddedCertFunc)).is_none() {
                    Status = STATUS_UNSUCCESSFUL;
                }
            }
        } else {
            /* Other systems have a registry entry for this */
            let Status1 = NtOpenKey(&mut KeyHandle, KEY_READ, &mut KeyAttributes);
            if NT_SUCCESS(Status1) {
                /* Close it, we'll query it through Rtl */
                NtClose(KeyHandle);

                /* Build the AppCert query table (element [0] + terminator).
                 * Built as a local because element [0] holds a non-const fn ptr
                 * and the address of a static (BasepAppCertDllsList). */
                let appcert_name: [u16; 12] = [
                    b'A' as u16, b'p' as u16, b'p' as u16, b'C' as u16, b'e' as u16,
                    b'r' as u16, b't' as u16, b'D' as u16, b'l' as u16, b'l' as u16,
                    b's' as u16, 0,
                ];
                let mut BasepAppCertTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
                // Bridge the cc type-name mismatch: BasepConfigureAppCertDlls is
                // extern "system"; the bindgen QueryRoutine field's fn type is the
                // same ABI per arch (stdcall on i386, C on amd64/arm64). Transmute
                // with the destination inferred (`_`) so it matches whatever the
                // field expects on each target.
                BasepAppCertTable[0].QueryRoutine = Some(core::mem::transmute::<
                    unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
                    _,
                >(
                    BasepConfigureAppCertDlls
                        as unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
                ));
                BasepAppCertTable[0].Flags = 1;
                BasepAppCertTable[0].Name = appcert_name.as_ptr();
                BasepAppCertTable[0].EntryContext =
                    &raw mut BasepAppCertDllsList as *mut LIST_ENTRY as PVOID;

                let sess_mgr: [u16; 16] = [
                    b'S' as u16, b'e' as u16, b's' as u16, b's' as u16, b'i' as u16,
                    b'o' as u16, b'n' as u16, b' ' as u16, b'M' as u16, b'a' as u16,
                    b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, 0,
                ];

                /* Do the query, which will call a special callback */
                Status = RtlQueryRegistryValues(
                    RTL_REGISTRY_CONTROL,
                    sess_mgr.as_ptr(),
                    BasepAppCertTable.as_mut_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                    Status = STATUS_SUCCESS;
                }
            }
        }

        /* Free any buffer if we had one */
        if !Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);
        }

        /* Check for errors, or a missing embedded/custom certification DLL */
        if !NT_SUCCESS(Status)
            || ((*(&raw const fEmbeddedCertFunc)).is_none() && IsListEmpty(&raw mut BasepAppCertDllsList))
        {
            /* The subsystem is not active on this machine, so give up */
            g_HaveAppCerts = FALSE as BOOLEAN;
            g_AppCertStatus = Status;
        } else {
            /* We have certification DLLs active, remember this */
            g_HaveAppCerts = TRUE as BOOLEAN;
        }

        /* We are done the initialization phase, release the lock */
        g_AppCertInitialized = TRUE as BOOLEAN;
        RtlLeaveCriticalSection(&raw mut gcsAppCert);
    }

    /* If there's no certification DLLs present, return the failure code */
    if g_HaveAppCerts == 0 {
        return g_AppCertStatus;
    }

    /* Otherwise, assume success and make sure we have *something* */
    debug_assert!((*(&raw const fEmbeddedCertFunc)).is_some() || !IsListEmpty(&raw mut BasepAppCertDllsList));
    Status = STATUS_SUCCESS;

    /* If the something is an embedded certification DLL, call it and return */
    if let Some(f) = fEmbeddedCertFunc {
        return f(ApplicationName);
    }

    /* Otherwise we have custom certification DLLs, parse them */
    NextEntry = BasepAppCertDllsList.Flink;
    CertFlag = 2;
    while NextEntry != &raw mut BasepAppCertDllsList as *mut LIST_ENTRY {
        /* Make sure the entry has a callback */
        Entry = (NextEntry as usize
            - core::mem::offset_of!(BASEP_APPCERT_ENTRY, Entry))
            as *mut BASEP_APPCERT_ENTRY;
        debug_assert!((*Entry).fPluginCertFunc.is_some());

        /* Call it and check if it failed */
        Status = ((*Entry).fPluginCertFunc.unwrap())(ApplicationName, 1);
        if !NT_SUCCESS(Status) {
            CertFlag = 3;
        }

        /* Move on */
        NextEntry = (*NextEntry).Flink;
    }

    /* Now loop them again */
    NextEntry = BasepAppCertDllsList.Flink;
    while NextEntry != &raw mut BasepAppCertDllsList as *mut LIST_ENTRY {
        /* Make sure the entry has a callback */
        Entry = (NextEntry as usize
            - core::mem::offset_of!(BASEP_APPCERT_ENTRY, Entry))
            as *mut BASEP_APPCERT_ENTRY;
        debug_assert!((*Entry).fPluginCertFunc.is_some());

        /* Call it, this time with the flag from the loop above */
        Status = ((*Entry).fPluginCertFunc.unwrap())(ApplicationName, CertFlag);

        /* NOTE: matches the C exactly — C does not advance NextEntry here. */
        NextEntry = (*NextEntry).Flink;
    }

    /* All done, return the status */
    Status
}

// ---------------------------------------------------------------------------
// BasepReplaceProcessThreadTokens
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BasepReplaceProcessThreadTokens(
    TokenHandle: HANDLE,
    ProcessHandle: HANDLE,
    ThreadHandle: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let name = b"SaferiReplaceProcessThreadTokens";
    let mut SaferiReplaceProcessThreadTokens: ANSI_STRING = core::mem::zeroed();
    SaferiReplaceProcessThreadTokens.Buffer = name.as_ptr() as PCHAR;
    SaferiReplaceProcessThreadTokens.Length = name.len() as USHORT;
    SaferiReplaceProcessThreadTokens.MaximumLength = (name.len() + 1) as USHORT;

    /* Enter the application certification lock */
    RtlEnterCriticalSection(&raw mut gcsAppCert);

    /* Check if we already know the function */
    if let Some(f) = g_SaferReplaceProcessThreadTokens {
        /* Call it */
        Status = if f(TokenHandle, ProcessHandle, ThreadHandle) != 0 {
            STATUS_SUCCESS
        } else {
            STATUS_UNSUCCESSFUL
        };
    } else {
        /* Check if the app certification DLL isn't loaded */
        if gSaferHandle.is_null()
            || gSaferHandle == (-1isize as HMODULE)
            || gSaferHandle == (-2isize as HMODULE)
        {
            /* Then we can't call the function */
            Status = STATUS_ENTRYPOINT_NOT_FOUND;
        } else {
            /* We have the DLL, find the address of the Safer function */
            Status = LdrGetProcedureAddress(
                gSaferHandle as PVOID,
                &mut SaferiReplaceProcessThreadTokens,
                0,
                &raw mut g_SaferReplaceProcessThreadTokens as *mut PSAFER_REPLACE_PROCESS_THREAD_TOKENS
                    as *mut PVOID,
            );
            if NT_SUCCESS(Status) {
                /* Found it, now call it */
                let f = g_SaferReplaceProcessThreadTokens.unwrap();
                Status = if f(TokenHandle, ProcessHandle, ThreadHandle) != 0 {
                    STATUS_SUCCESS
                } else {
                    STATUS_UNSUCCESSFUL
                };
            } else {
                /* Couldn't find it, so this must be an unsupported DLL */
                LdrUnloadDll(gSaferHandle as PVOID);
                gSaferHandle = core::ptr::null_mut();
                Status = STATUS_ENTRYPOINT_NOT_FOUND;
            }
        }
    }

    /* Release the lock and return the result */
    RtlLeaveCriticalSection(&raw mut gcsAppCert);
    Status
}

// ---------------------------------------------------------------------------
// BasepSxsCloseHandles
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BasepSxsCloseHandles(Handles: PBASE_MSG_SXS_HANDLES) {
    /* Sanity checks */
    debug_assert!(!Handles.is_null());
    debug_assert!(
        (*Handles).Process.is_null() || (*Handles).Process == NtCurrentProcess()
    );

    /* Close the file handle */
    if !(*Handles).File.is_null() {
        let Status = NtClose((*Handles).File);
        debug_assert!(NT_SUCCESS(Status));
        let _ = Status;
    }

    /* Close the section handle */
    if !(*Handles).Section.is_null() {
        let Status = NtClose((*Handles).Section);
        debug_assert!(NT_SUCCESS(Status));
        let _ = Status;
    }

    /* Unmap the section view */
    if (*Handles).ViewBase.QuadPart != 0 {
        let Status = NtUnmapViewOfSection(
            NtCurrentProcess(),
            (*Handles).ViewBase.QuadPart as ULONG_PTR as PVOID,
        );
        debug_assert!(NT_SUCCESS(Status));
        let _ = Status;
    }
}

// ---------------------------------------------------------------------------
// BaseProcessStartup (DECLSPEC_NORETURN)
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn BaseProcessStartup(
    lpStartAddress: PPROCESS_START_ROUTINE,
) -> ! {
    let mut start = lpStartAddress;

    /* Set our Start Address */
    NtSetInformationThread(
        NtCurrentThread(),
        ThreadQuerySetWin32StartAddress,
        &mut start as *mut PPROCESS_START_ROUTINE as PVOID,
        core::mem::size_of::<PPROCESS_START_ROUTINE>() as ULONG,
    );

    /* Call the Start Routine under the real top-level __try/__except */
    crate::client::except::run_under_top_level_handler(move || {
        ExitThread((lpStartAddress.unwrap())())
    })
}

// ---------------------------------------------------------------------------
// InitCommandLines
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn InitCommandLines() {
    /* Read the UNICODE_STRING from the PEB */
    BaseUnicodeCommandLine = (*(*NtCurrentPeb()).ProcessParameters).CommandLine;

    /* Convert to ANSI_STRING for the *A callers */
    let Status = RtlUnicodeStringToAnsiString(
        &raw mut BaseAnsiCommandLine,
        &raw mut BaseUnicodeCommandLine,
        TRUE as BOOLEAN,
    );
    if !NT_SUCCESS(Status) {
        RtlInitEmptyAnsiString(&raw mut BaseAnsiCommandLine, core::ptr::null_mut(), 0);
    }
}
