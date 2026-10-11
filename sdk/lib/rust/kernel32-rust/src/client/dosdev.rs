use winedebug_rust::{TRACE, WARN};
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{CSR_CREATE_API_NUMBER, InitializeObjectAttributes};

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// NDEBUG build: Wine TRACE/WARN compile to nothing (printf-style format +
// varargs), so model them as macros that discard their arguments.

extern "system" {
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
}

// `L"..."` wide-string literals from the C source. DeepSeek emitted them either as
// the invalid Rust token `L"..."` or as `b"...".as_ptr() as *const u16` (8-bit bytes
// reinterpreted as UTF-16 = garbage). These strings are pure ASCII, so build real
// NUL-terminated UTF-16 arrays at compile time and point at them.
const fn to_w<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut a = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        a[i] = b[i] as u16;
        i += 1;
    }
    a
}
// "\\GLOBAL??" = 9 chars + NUL
static GLOBAL_QQ_W: [u16; 10] = to_w("\\GLOBAL??");
// "user32.dll" = 10 chars + NUL
static USER32_DLL_W: [u16; 11] = to_w("user32.dll");
// "\\??" = 3 chars + NUL
static QQ_W: [u16; 4] = to_w("\\??");
// "SymbolicLink" = 12 chars + NUL
static SYMBOLIC_LINK_W: [u16; 13] = to_w("SymbolicLink");

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _SEH2_FINALLY {
    // This is a SEH2 finally block, represented as a struct for C compatibility
    // The actual implementation is handled by the compiler
}

pub const NDEBUG: i32 = 1;

// DEBUG_CHANNEL macro - typically expands to nothing in release builds
// _SEH2_FINALLY block - represents a structured exception handling finally block
// _SEH2_END - end of SEH2 block
#[no_mangle]
pub unsafe extern "C" fn IsGlobalDeviceMap(
    DirectoryHandle: HANDLE,
    IsGlobal: PBOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ReturnLength: DWORD = 0;
    let mut GlobalString: _UNICODE_STRING = core::mem::zeroed();
    let mut NameInfo: OBJECT_NAME_INFORMATION = core::mem::zeroed();
    let mut PNameInfo: *mut OBJECT_NAME_INFORMATION;

    if DirectoryHandle.is_null() || IsGlobal.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    PNameInfo = core::ptr::null_mut();
    loop {
        Status = NtQueryObject(
            DirectoryHandle,
            _OBJECT_INFORMATION_CLASS_ObjectNameInformation,
            &mut NameInfo as *mut OBJECT_NAME_INFORMATION as *mut core::ffi::c_void,
            0,
            &mut ReturnLength,
        );

        if NT_SUCCESS(Status) || Status == STATUS_INFO_LENGTH_MISMATCH {
            PNameInfo = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                ReturnLength as SIZE_T,
            ) as *mut OBJECT_NAME_INFORMATION;

            if PNameInfo.is_null() {
                Status = STATUS_NO_MEMORY;
                break;
            }

            Status = NtQueryObject(
                DirectoryHandle,
                _OBJECT_INFORMATION_CLASS_ObjectNameInformation,
                PNameInfo as *mut core::ffi::c_void,
                ReturnLength,
                &mut ReturnLength,
            );

            if NT_SUCCESS(Status) {
                RtlInitUnicodeString(&mut GlobalString, GLOBAL_QQ_W.as_ptr() as PCWSTR);
                *IsGlobal = if RtlEqualUnicodeString(
                    &GlobalString,
                    &(*PNameInfo).Name,
                    0,
                ) != 0
                {
                    1
                } else {
                    0
                };
                Status = STATUS_SUCCESS;
            }
        }
        break;
    }

    if !PNameInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, PNameInfo as PVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn FindSymbolicLinkEntry(
    NameToFind: PWSTR,
    NamesList: PWSTR,
    TotalEntries: DWORD,
    Found: PBOOLEAN,
) -> DWORD {
    let mut Current: WCHAR;
    let mut Entries: DWORD;
    let mut PartialNamesList: PWSTR;

    /* We need all parameters to be set */
    if NameToFind.is_null() || NamesList.is_null() || Found.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    /* Assume failure */
    *Found = 0;

    /* If no entries, job done, nothing found */
    if TotalEntries == 0 {
        return ERROR_SUCCESS;
    }

    /* Start browsing the names list */
    Entries = 0;
    PartialNamesList = NamesList;
    /* As long as we didn't find the name... */
    while wcscmp(NameToFind, PartialNamesList) != 0 {
        /* We chomped an entry! */
        Entries += 1;

        /* We're out of entries, bail out not to overrun */
        if Entries > TotalEntries {
            /*
             * Even though we found nothing,
             * the function ran fine
             */
            return ERROR_SUCCESS;
        }

        /* Jump to the next string */
        loop {
            Current = *PartialNamesList;
            PartialNamesList = PartialNamesList.offset(1);
            if Current == 0 {
                break;
            }
        }
    }

    /*
     * We're here because the loop stopped:
     * it means we found the name in the list
     */
    *Found = 1;
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DefineDosDeviceA(
    dwFlags: DWORD,
    lpDeviceName: LPCSTR,
    lpTargetPath: LPCSTR,
) -> BOOL {
    let mut Result: BOOL;
    let mut Status: NTSTATUS;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut TargetPathBuffer: PWSTR;
    let mut TargetPathU: _UNICODE_STRING = core::mem::zeroed();
    let DeviceNameU: PUNICODE_STRING;

    /* Convert DeviceName using static unicode string */
    RtlInitAnsiString(&mut AnsiString, lpDeviceName);
    DeviceNameU = &mut (*NtCurrentTeb()).StaticUnicodeString;
    Status = RtlAnsiStringToUnicodeString(DeviceNameU, &mut AnsiString, FALSE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        /*
         * If the static unicode string is too small,
         * it's because the name is too long...
         * so, return appropriate status!
         */
        if Status == STATUS_BUFFER_OVERFLOW {
            SetLastError(ERROR_FILENAME_EXCED_RANGE);
            return FALSE;
        }

        BaseSetLastNTError(Status);
        return FALSE;
    }

    /* Convert target path if existing */
    if !lpTargetPath.is_null() {
        RtlInitAnsiString(&mut AnsiString, lpTargetPath);
        Status = RtlAnsiStringToUnicodeString(&mut TargetPathU, &mut AnsiString, TRUE as BOOLEAN);
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return FALSE;
        }

        TargetPathBuffer = TargetPathU.Buffer;
    } else {
        TargetPathBuffer = core::ptr::null_mut();
    }

    /* Call W */
    Result = DefineDosDeviceW(dwFlags, (*DeviceNameU).Buffer, TargetPathBuffer);

    /* Free target path if allocated */
    if !TargetPathBuffer.is_null() {
        RtlFreeUnicodeString(&mut TargetPathU);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn DefineDosDeviceW(
    dwFlags: DWORD,
    lpDeviceName: LPCWSTR,
    lpTargetPath: LPCWSTR,
) -> BOOL {
    let mut ArgumentCount: ULONG;
    let mut BufferSize: ULONG;
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let DefineDosDeviceRequest: PBASE_DEFINE_DOS_DEVICE = &mut ApiMessage.Data.DefineDosDeviceRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;
    let mut NtTargetPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut hUser32: HMODULE;
    let mut dbcv: DEV_BROADCAST_VOLUME = core::mem::zeroed();
    let mut dwRecipients: DWORD;
    type BSM_type = unsafe extern "system" fn(DWORD, LPDWORD, UINT, WPARAM, LPARAM) -> LONG;
    let BSM_ptr: BSM_type;
    let LUIDDeviceMapsEnabled: BOOLEAN;
    let mut Letter: WCHAR;
    let mut wParam: WPARAM;

    /* Get status about local device mapping */
    LUIDDeviceMapsEnabled = (*BaseStaticServerData).LUIDDeviceMapsEnabled;

    /* Validate input & flags */
    if (dwFlags & 0xFFFFFFE0) != 0
        || ((dwFlags & DDD_EXACT_MATCH_ON_REMOVE) != 0
            && (dwFlags & DDD_REMOVE_DEFINITION) == 0)
        || (lpTargetPath.is_null()
            && (dwFlags & (DDD_LUID_BROADCAST_DRIVE | DDD_REMOVE_DEFINITION)) == 0)
        || ((dwFlags & DDD_LUID_BROADCAST_DRIVE) != 0
            && (lpDeviceName.is_null()
                || !lpTargetPath.is_null()
                || (dwFlags
                    & (DDD_NO_BROADCAST_SYSTEM
                        | DDD_EXACT_MATCH_ON_REMOVE
                        | DDD_RAW_TARGET_PATH))
                    != 0
                || LUIDDeviceMapsEnabled == 0))
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Initialize device unicode string to ease its use */
    RtlInitUnicodeString(&mut DeviceNameU, lpDeviceName);

    /* The buffer for CSR call will contain it */
    BufferSize = DeviceNameU.MaximumLength as ULONG;
    ArgumentCount = 1;

    /* If we don't have target path, use empty string */
    if lpTargetPath.is_null() {
        RtlInitUnicodeString(&mut NtTargetPathU, core::ptr::null());
    } else {
        /* Else, use it raw if asked to */
        if (dwFlags & DDD_RAW_TARGET_PATH) != 0 {
            RtlInitUnicodeString(&mut NtTargetPathU, lpTargetPath);
        } else {
            /* Otherwise, use it converted */
            if RtlDosPathNameToNtPathName_U(
                lpTargetPath,
                &mut NtTargetPathU,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == 0
            {
                WARN!("RtlDosPathNameToNtPathName_U() failed\n");
                BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
                return 0;
            }
        }

        /* This target path will be the second arg */
        ArgumentCount = 2;
        BufferSize += NtTargetPathU.MaximumLength as ULONG;
    }

    /* Allocate the capture buffer for our strings */
    CaptureBuffer = CsrAllocateCaptureBuffer(ArgumentCount, BufferSize);
    if CaptureBuffer.is_null() {
        if (dwFlags & DDD_RAW_TARGET_PATH) == 0 {
            RtlFreeUnicodeString(&mut NtTargetPathU);
        }

        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* Set the flags */
    (*DefineDosDeviceRequest).Flags = dwFlags;

    /* Allocate a buffer for the device name */
    (*DefineDosDeviceRequest).DeviceName.MaximumLength = CsrAllocateMessagePointer(
        CaptureBuffer,
        DeviceNameU.MaximumLength as ULONG,
        &mut (*DefineDosDeviceRequest).DeviceName.Buffer as *mut *mut u16 as *mut *mut core::ffi::c_void,
    ) as USHORT;
    /* And copy it while upcasing it */
    RtlUpcaseUnicodeString(
        &mut (*DefineDosDeviceRequest).DeviceName,
        &DeviceNameU,
        0,
    );

    /* If we have a target path, copy it too, and free it if allocated */
    if NtTargetPathU.Length != 0 {
        (*DefineDosDeviceRequest).TargetPath.MaximumLength = CsrAllocateMessagePointer(
            CaptureBuffer,
            NtTargetPathU.MaximumLength as ULONG,
            &mut (*DefineDosDeviceRequest).TargetPath.Buffer as *mut *mut u16 as *mut *mut core::ffi::c_void,
        ) as USHORT;
        RtlCopyUnicodeString(
            &mut (*DefineDosDeviceRequest).TargetPath,
            &NtTargetPathU,
        );

        if (dwFlags & DDD_RAW_TARGET_PATH) == 0 {
            RtlFreeUnicodeString(&mut NtTargetPathU);
        }
    }
    /* Otherwise, null initialize the string */
    else {
        RtlInitUnicodeString(&mut (*DefineDosDeviceRequest).TargetPath, core::ptr::null());
    }

    /* Finally,  call the server */
    CsrClientCallServer(
        &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, _BASESRV_API_NUMBER_BasepDefineDosDevice as ULONG),
        core::mem::size_of::<BASE_DEFINE_DOS_DEVICE>() as u32,
    );
    CsrFreeCaptureBuffer(CaptureBuffer);

    /* Return failure if any */
    if !NT_SUCCESS(ApiMessage.Status) {
        WARN!("CsrClientCallServer() failed (Status %lx)\n", ApiMessage.Status);
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    /* Here is the success path, we will always return true */

    /* Should broadcast the event? Only do if not denied and if drive letter */
    if (dwFlags & DDD_NO_BROADCAST_SYSTEM) == 0
        && DeviceNameU.Length == (2 * core::mem::size_of::<WCHAR>()) as u16
        && *DeviceNameU.Buffer.offset(1) as u16 == ':' as u16
    {
        /* Make sure letter is valid and there are no local device mappings */
        Letter = (RtlUpcaseUnicodeChar(*DeviceNameU.Buffer.offset(0)) as u16).wrapping_sub('A' as u16);
        if Letter < 26 && LUIDDeviceMapsEnabled == 0 {
            /* Rely on user32 for broadcasting */
            hUser32 = LoadLibraryW(USER32_DLL_W.as_ptr() as *const u16);
            if !hUser32.is_null() {
                /* Get the function pointer */
                BSM_ptr = core::mem::transmute::<FARPROC, BSM_type>(
                    GetProcAddress(hUser32, "BroadcastSystemMessageW\0".as_ptr() as *const i8),
                );
                if BSM_ptr as usize != 0 {
                    /* Set our target */
                    dwRecipients = BSM_APPLICATIONS;

                    /* And initialize our structure */
                    dbcv.dbcv_size = core::mem::size_of::<DEV_BROADCAST_VOLUME>() as u32;
                    dbcv.dbcv_devicetype = DBT_DEVTYP_VOLUME;
                    dbcv.dbcv_reserved = 0;

                    /* Set the volume which had the event */
                    dbcv.dbcv_unitmask = 1u32 << (Letter as u32);
                    dbcv.dbcv_flags = DBTF_NET as u16;

                    /* And properly set the event (removal or arrival?) */
                    wParam = if (dwFlags & DDD_REMOVE_DEFINITION) != 0 {
                        DBT_DEVICEREMOVECOMPLETE as WPARAM
                    } else {
                        DBT_DEVICEARRIVAL as WPARAM
                    };

                    /* And broadcast! */
                    BSM_ptr(
                        BSF_SENDNOTIFYMESSAGE | BSF_FLUSHDISK,
                        &mut dwRecipients,
                        WM_DEVICECHANGE,
                        wParam,
                        &mut dbcv as *mut DEV_BROADCAST_VOLUME as LPARAM,
                    );
                }

                /* We're done! */
                FreeLibrary(hUser32);
            }
        }
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn QueryDosDeviceA(
    lpDeviceName: LPCSTR,
    lpTargetPath: LPSTR,
    ucchMax: DWORD,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut CurrentPosition: USHORT;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut TargetPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceNameU: PUNICODE_STRING;
    let mut RetLength: DWORD;
    let mut CurrentLength: DWORD;
    let mut Length: DWORD;
    let mut DeviceNameBuffer: PWSTR;
    let mut TargetPathBuffer: PWSTR;

    /* If we want a specific device name, convert it */
    if !lpDeviceName.is_null() {
        /* Convert DeviceName using static unicode string */
        RtlInitAnsiString(&mut AnsiString, lpDeviceName);
        DeviceNameU = &mut (*NtCurrentTeb()).StaticUnicodeString;
        Status = RtlAnsiStringToUnicodeString(DeviceNameU, &mut AnsiString, 0);
        if !NT_SUCCESS(Status) {
            /*
             * If the static unicode string is too small,
             * it's because the name is too long...
             * so, return appropriate status!
             */
            if Status == STATUS_BUFFER_OVERFLOW {
                SetLastError(ERROR_FILENAME_EXCED_RANGE);
                return 0;
            }

            BaseSetLastNTError(Status);
            return 0;
        }

        DeviceNameBuffer = (*DeviceNameU).Buffer;
    } else {
        DeviceNameBuffer = core::ptr::null_mut();
    }

    /* Allocate the output buffer for W call */
    TargetPathBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        ((ucchMax as usize).wrapping_mul(core::mem::size_of::<u16>())) as SIZE_T,
    ) as PWSTR;
    if TargetPathBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* Call W */
    Length = QueryDosDeviceW(DeviceNameBuffer, TargetPathBuffer, ucchMax);
    /* We'll return that length in case of a success */
    RetLength = Length;

    /* Handle the case where we would fill output buffer completly */
    if Length != 0 && Length == ucchMax {
        /* This will be our work length (but not the one we return) */
        Length -= 1;
        /* Already 0 the last char */
        *lpTargetPath.offset(Length as isize) = 0i8;
    }

    /* If we had an output, start the convert loop */
    if Length != 0 {
        /*
         * We'll have to loop because TargetPathBuffer may contain
         * several strings (NULL separated)
         * We'll start at position 0
         */
        CurrentPosition = 0;
        while (CurrentPosition as DWORD) < Length {
            /* Get the maximum length */
            CurrentLength = min(
                Length - CurrentPosition as DWORD,
                (MAXUSHORT / 2) as DWORD,
            );

            /* Initialize our output string */
            AnsiString.Length = 0;
            AnsiString.MaximumLength = (CurrentLength + core::mem::size_of::<u8>() as DWORD) as u16;
            AnsiString.Buffer = &mut *lpTargetPath.offset(CurrentPosition as isize);

            /* Initialize input string that will be converted */
            TargetPathU.Length = (CurrentLength * core::mem::size_of::<u16>() as DWORD) as u16;
            TargetPathU.MaximumLength =
                (CurrentLength * core::mem::size_of::<u16>() as DWORD + core::mem::size_of::<u16>() as DWORD) as u16;
            TargetPathU.Buffer = &mut *TargetPathBuffer.offset(CurrentPosition as isize);

            /* Convert to ANSI */
            Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &TargetPathU, 0);
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                /* In case of a failure, forget about everything */
                RetLength = 0;

                break;
            }

            /* Move to the next string */
            CurrentPosition += CurrentLength as USHORT;
        }
    }

    /* Free our intermediate buffer and leave */
    RtlFreeHeap(RtlGetProcessHeap(), 0, TargetPathBuffer as *mut core::ffi::c_void);

    return RetLength;
}

#[no_mangle]
pub unsafe extern "system" fn QueryDosDeviceW(
    lpDeviceName: LPCWSTR,
    lpTargetPath: LPWSTR,
    ucchMax: DWORD,
) -> DWORD {
    let mut Ptr: PWSTR;
    let mut Buffer: PVOID;
    let mut Status: NTSTATUS;
    let mut i: USHORT;
    let mut TotalEntries: USHORT;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();
    let mut DeviceHandle: HANDLE = core::ptr::null_mut();
    let mut IsGlobal: BOOLEAN;
    let mut GlobalNeeded: BOOLEAN;
    let mut Found: BOOLEAN = 0;
    let mut DirInfo: POBJECT_DIRECTORY_INFORMATION;
    let NullEntry: OBJECT_DIRECTORY_INFORMATION = OBJECT_DIRECTORY_INFORMATION {
        Name: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
        TypeName: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
    };
    let mut ReturnLength: ULONG = 0;
    let mut NameLength: ULONG;
    let mut Length: ULONG = 0;
    let mut Context: ULONG;
    let mut BufferLength: ULONG;

    /* Open the '\??' directory */
    RtlInitUnicodeString(&mut UnicodeString, QQ_W.as_ptr() as LPCWSTR);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut UnicodeString,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenDirectoryObject(
        &mut DirectoryHandle,
        DIRECTORY_QUERY,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        WARN!(
            "NtOpenDirectoryObject() failed (Status %lx)\n\0".as_ptr() as LPCSTR,
            Status,
        );
        BaseSetLastNTError(Status);
        return 0;
    }

    Buffer = core::ptr::null_mut();
    let mut __finally_abnormal_termination: BOOLEAN = 0;
    let mut __finally_leave: BOOLEAN = 0;
    loop {
        __finally_leave = 0;
        if lpDeviceName != core::ptr::null_mut() {
            /* Open the lpDeviceName link object */
            RtlInitUnicodeString(&mut UnicodeString, lpDeviceName);
            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut UnicodeString,
                OBJ_CASE_INSENSITIVE,
                DirectoryHandle,
                core::ptr::null_mut(),
            );
            Status = NtOpenSymbolicLinkObject(
                &mut DeviceHandle,
                SYMBOLIC_LINK_QUERY,
                &mut ObjectAttributes,
            );
            if !NT_SUCCESS(Status) {
                WARN!(
                    "NtOpenSymbolicLinkObject() failed (Status %lx)\n\0".as_ptr() as LPCSTR,
                    Status,
                );
                __finally_leave = 1;
                break;
            }

            /*
             * Make sure we don't overrun the output buffer, so convert our DWORD
             * size to USHORT size properly
             */
            Length = if ucchMax <= MAXULONG / core::mem::size_of::<WCHAR>() as DWORD {
                ucchMax * core::mem::size_of::<WCHAR>() as DWORD
            } else {
                MAXULONG
            };

            /* Query link target */
            UnicodeString.Length = 0;
            UnicodeString.MaximumLength = if Length <= MAXUSHORT as ULONG {
                Length as USHORT
            } else {
                MAXUSHORT as USHORT
            };
            UnicodeString.Buffer = lpTargetPath;

            ReturnLength = 0;
            Status = NtQuerySymbolicLinkObject(
                DeviceHandle,
                &mut UnicodeString,
                &mut ReturnLength,
            );
            NtClose(DeviceHandle);
            if !NT_SUCCESS(Status) {
                WARN!(
                    "NtQuerySymbolicLinkObject() failed (Status %lx)\n\0".as_ptr() as LPCSTR,
                    Status,
                );
                __finally_leave = 1;
                break;
            }

            TRACE!(
                "ReturnLength: %lu\n\0".as_ptr() as LPCSTR,
                ReturnLength,
            );
            TRACE!(
                "TargetLength: %hu\n\0".as_ptr() as LPCSTR,
                UnicodeString.Length,
            );
            TRACE!(
                "Target: '%wZ'\n\0".as_ptr() as LPCSTR,
                &mut UnicodeString,
            );

            Length = ReturnLength / core::mem::size_of::<WCHAR>() as ULONG;
            /* Make sure we null terminate output buffer */
            if Length == 0 || *lpTargetPath.offset(Length as isize - 1) != 0 {
                if Length >= ucchMax {
                    TRACE!(
                        "Buffer is too small\n\0".as_ptr() as LPCSTR,
                    );
                    Status = STATUS_BUFFER_TOO_SMALL;
                    __finally_leave = 1;
                    break;
                }

                /* Append null-character */
                *lpTargetPath.offset(Length as isize) = 0;
                Length += 1;
            }

            if Length < ucchMax {
                /* Append null-character */
                *lpTargetPath.offset(Length as isize) = 0;
                Length += 1;
            }

            __finally_leave = 1;
            break;
        }

        /*
         * If LUID device maps are enabled,
         * ?? may not point to BaseNamedObjects
         * It may only be local DOS namespace.
         * And thus, it might be required to browse
         * Global?? for global devices
         */
        GlobalNeeded = 0;
        if (*BaseStaticServerData).LUIDDeviceMapsEnabled != 0 {
            /* Assume ?? == Global?? */
            IsGlobal = 1;
            /* Check if it's the case */
            Status = IsGlobalDeviceMap(DirectoryHandle, &mut IsGlobal);
            if NT_SUCCESS(Status) && IsGlobal == 0 {
                /* It's not, we'll have to browse Global?? too! */
                GlobalNeeded = 1;
            }
        }

        /*
         * Make sure we don't overrun the output buffer, so convert our DWORD
         * size to USHORT size properly
         */
        BufferLength = if ucchMax <= MAXULONG / core::mem::size_of::<WCHAR>() as DWORD {
            ucchMax * core::mem::size_of::<WCHAR>() as DWORD
        } else {
            MAXULONG
        };
        Length = 0;
        Ptr = lpTargetPath;

        Context = 0;
        TotalEntries = 0;

        /*
         * We'll query all entries at once, with a rather big buffer
         * If it's too small, we'll grow it by 2.
         * Limit the number of attempts to 3.
         */
        i = 0;
        while i < 3 {
            /* Allocate the query buffer */
            Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T);
            if Buffer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                __finally_leave = 1;
                break;
            }

            /* Perform the query */
            Status = NtQueryDirectoryObject(
                DirectoryHandle,
                Buffer,
                BufferLength,
                0,
                1,
                &mut Context,
                &mut ReturnLength,
            );
            /* Only failure accepted is: no more entries */
            if !NT_SUCCESS(Status) {
                if Status != STATUS_NO_MORE_ENTRIES {
                    __finally_leave = 1;
                    break;
                }

                /*
                 * Which is a success! But break out,
                 * it means our query returned no results
                 * so, nothing to parse.
                 */
                Status = STATUS_SUCCESS;
                break;
            }

            /* In case we had them all, start browsing for devices */
            if Status != STATUS_MORE_ENTRIES {
                DirInfo = Buffer as POBJECT_DIRECTORY_INFORMATION;

                /* Loop until we find the nul entry (terminating entry) */
                loop {
                    /* It's an entry full of zeroes */
                    if RtlCompareMemory(
                        &NullEntry as *const OBJECT_DIRECTORY_INFORMATION as *const core::ffi::c_void,
                        DirInfo as *const core::ffi::c_void,
                        core::mem::size_of::<OBJECT_DIRECTORY_INFORMATION>() as SIZE_T,
                    ) == core::mem::size_of::<OBJECT_DIRECTORY_INFORMATION>() as SIZE_T
                    {
                        break;
                    }

                    /* Only handle symlinks */
                    if wcscmp(
                        (*DirInfo).TypeName.Buffer,
                        SYMBOLIC_LINK_W.as_ptr() as LPCWSTR,
                    ) == 0
                    {
                        TRACE!(
                            "Name: '%wZ'\n\0".as_ptr() as LPCSTR,
                            &(*DirInfo).Name,
                        );

                        /* Get name length in chars to only comparisons */
                        NameLength = (*DirInfo).Name.Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG;

                        /* Make sure we don't overrun output buffer */
                        if Length > ucchMax
                            || NameLength > ucchMax - Length
                            || ucchMax - NameLength - Length < core::mem::size_of::<WCHAR>() as ULONG
                        {
                            Status = STATUS_BUFFER_TOO_SMALL;
                            __finally_leave = 1;
                            break;
                        }

                        /* Copy and NULL terminate string */
                        memcpy(
                            Ptr as *mut core::ffi::c_void,
                            (*DirInfo).Name.Buffer as *const core::ffi::c_void,
                            (*DirInfo).Name.Length as usize,
                        );
                        *Ptr.offset(NameLength as isize) = 0;

                        Ptr = Ptr.offset((NameLength + 1) as isize);
                        Length += NameLength + 1;

                        /*
                         * Keep the entries count, in case we would have to
                         * handle GLOBAL?? too
                         */
                        TotalEntries += 1;
                    }

                    /* Move to the next entry */
                    DirInfo = DirInfo.offset(1);
                }

                if __finally_leave != 0 {
                    break;
                }

                /*
                 * No need to loop again here, we got all the entries
                 * Note: we don't free the buffer here, because we may
                 * need it for GLOBAL??, so we save a few cycles here.
                 */
                break;
            }

            /* Failure path here, we'll need bigger buffer */
            RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
            Buffer = core::ptr::null_mut();

            /* We can't have bigger than that one, so leave */
            if BufferLength == MAXULONG {
                break;
            }

            /* Prevent any overflow while computing new size */
            if MAXULONG - BufferLength < BufferLength {
                BufferLength = MAXULONG;
            } else {
                BufferLength *= 2;
            }

            i += 1;
        }

        if __finally_leave != 0 {
            break;
        }

        /*
         * Out of the hot loop, but with more entries left?
         * that's an error case, leave here!
         */
        if Status == STATUS_MORE_ENTRIES {
            Status = STATUS_BUFFER_TOO_SMALL;
            __finally_leave = 1;
            break;
        }

        /* Now, if we had to handle GLOBAL??, go for it! */
        if (*BaseStaticServerData).LUIDDeviceMapsEnabled != 0 && NT_SUCCESS(Status) && GlobalNeeded != 0 {
            NtClose(DirectoryHandle);
            DirectoryHandle = core::ptr::null_mut();

            RtlInitUnicodeString(&mut UnicodeString, GLOBAL_QQ_W.as_ptr() as LPCWSTR);
            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut UnicodeString,
                OBJ_CASE_INSENSITIVE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            Status = NtOpenDirectoryObject(
                &mut DirectoryHandle,
                DIRECTORY_QUERY,
                &mut ObjectAttributes,
            );
            if !NT_SUCCESS(Status) {
                WARN!(
                    "NtOpenDirectoryObject() failed (Status %lx)\n\0".as_ptr() as LPCSTR,
                    Status,
                );
                __finally_leave = 1;
                break;
            }

            /*
             * We'll query all entries at once, with a rather big buffer
             * If it's too small, we'll grow it by 2.
             * Limit the number of attempts to 3.
             */
            i = 0;
            while i < 3 {
                /* If we had no buffer from previous attempt, allocate one */
                if Buffer.is_null() {
                    Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T);
                    if Buffer.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        __finally_leave = 1;
                        break;
                    }
                }

                /* Perform the query */
                Status = NtQueryDirectoryObject(
                    DirectoryHandle,
                    Buffer,
                    BufferLength,
                    0,
                    1,
                    &mut Context,
                    &mut ReturnLength,
                );
                /* Only failure accepted is: no more entries */
                if !NT_SUCCESS(Status) {
                    if Status != STATUS_NO_MORE_ENTRIES {
                        __finally_leave = 1;
                        break;
                    }

                    /*
                     * Which is a success! But break out,
                     * it means our query returned no results
                     * so, nothing to parse.
                     */
                    Status = STATUS_SUCCESS;
                    break;
                }

                /* In case we had them all, start browsing for devices */
                if Status != STATUS_MORE_ENTRIES {
                    DirInfo = Buffer as POBJECT_DIRECTORY_INFORMATION;

                    /* Loop until we find the nul entry (terminating entry) */
                    loop {
                        /* It's an entry full of zeroes */
                        if RtlCompareMemory(
                            &NullEntry as *const OBJECT_DIRECTORY_INFORMATION as *const core::ffi::c_void,
                            DirInfo as *const core::ffi::c_void,
                            core::mem::size_of::<OBJECT_DIRECTORY_INFORMATION>() as SIZE_T,
                        ) == core::mem::size_of::<OBJECT_DIRECTORY_INFORMATION>() as SIZE_T
                        {
                            break;
                        }

                        /* Only handle symlinks */
                        if wcscmp(
                            (*DirInfo).TypeName.Buffer,
                            SYMBOLIC_LINK_W.as_ptr() as LPCWSTR,
                        ) == 0
                        {
                            TRACE!(
                                "Name: '%wZ'\n\0".as_ptr() as LPCSTR,
                                &(*DirInfo).Name,
                            );

                            /*
                             * Now, we previously already browsed ??, and we
                             * don't want to devices twice, so we'll check
                             * the output buffer for duplicates.
                             * We'll add our entry only if we don't have already
                             * returned it.
                             */
                            if FindSymbolicLinkEntry(
                                (*DirInfo).Name.Buffer,
                                lpTargetPath,
                                TotalEntries as DWORD,
                                &mut Found,
                            ) == ERROR_SUCCESS
                                && Found == 0
                            {
                                /* Get name length in chars to only comparisons */
                                NameLength = (*DirInfo).Name.Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG;

                                /* Make sure we don't overrun output buffer */
                                if Length > ucchMax
                                    || NameLength > ucchMax - Length
                                    || ucchMax - NameLength - Length < core::mem::size_of::<WCHAR>() as ULONG
                                {
                                    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
                                    NtClose(DirectoryHandle);
                                    BaseSetLastNTError(STATUS_BUFFER_TOO_SMALL);
                                    return 0;
                                }

                                /* Copy and NULL terminate string */
                                memcpy(
                                    Ptr as *mut core::ffi::c_void,
                                    (*DirInfo).Name.Buffer as *const core::ffi::c_void,
                                    (*DirInfo).Name.Length as usize,
                                );
                                *Ptr.offset(NameLength as isize) = 0;

                                Ptr = Ptr.offset((NameLength + 1) as isize);
                                Length += NameLength + 1;
                            }
                        }

                        /* Move to the next entry */
                        DirInfo = DirInfo.offset(1);
                    }

                    if __finally_leave != 0 {
                        break;
                    }

                    /* No need to loop again here, we got all the entries */
                    break;
                }

                /* Failure path here, we'll need bigger buffer */
                RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
                Buffer = core::ptr::null_mut();

                /* We can't have bigger than that one, so leave */
                if BufferLength == MAXULONG {
                    break;
                }

                /* Prevent any overflow while computing new size */
                if MAXULONG - BufferLength < BufferLength {
                    BufferLength = MAXULONG;
                } else {
                    BufferLength *= 2;
                }

                i += 1;
            }

            if __finally_leave != 0 {
                break;
            }

            /*
             * Out of the hot loop, but with more entries left?
             * that's an error case, leave here!
             */
            if Status == STATUS_MORE_ENTRIES {
                Status = STATUS_BUFFER_TOO_SMALL;
                __finally_leave = 1;
                break;
            }
        }

        /* If we failed somewhere, just leave */
        if !NT_SUCCESS(Status) {
            __finally_leave = 1;
            break;
        }

        /* If we returned no entries, time to write the empty string */
        if Length == 0 {
            /* Unless output buffer is too small! */
            if ucchMax <= 0 {
                Status = STATUS_BUFFER_TOO_SMALL;
                __finally_leave = 1;
                break;
            }

            /* Emptry string is one char (terminator!) */
            *Ptr = 0;
            Ptr = Ptr.offset(1);
            Length = 1;
        }

        /*
         * If we have enough room, we need to double terminate the buffer:
         * that's a MULTI_SZ buffer, its end is marked by double NULL.
         * One was already added during the "copy string" process.
         * If we don't have enough room: that's a failure case.
         */
        if Length < ucchMax {
            *Ptr = 0;
            Ptr = Ptr.offset(1);
        } else {
            Status = STATUS_BUFFER_TOO_SMALL;
        }

        break;
    }

    /* _SEH2_FINALLY */
    if DirectoryHandle != core::ptr::null_mut() {
        NtClose(DirectoryHandle);
    }

    if !Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
    }

    if !NT_SUCCESS(Status) {
        Length = 0;
        BaseSetLastNTError(Status);
    }

    return Length;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/dosdev.c
//  * PURPOSE:         Dos device functions
//  * PROGRAMMER:      Ariadne (ariadne@xs4all.nl)
//  *                  Pierre Schweitzer
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
// #include <dbt.h>
// DEBUG_CHANNEL(kernel32file);
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// IsGlobalDeviceMap(
//     HANDLE DirectoryHandle,
//     PBOOLEAN IsGlobal)
// {
//     NTSTATUS Status;
//     DWORD ReturnLength;
//     UNICODE_STRING GlobalString;
//     OBJECT_NAME_INFORMATION NameInfo, *PNameInfo;
//
//     /* We need both parameters */
//     if (DirectoryHandle == 0 || IsGlobal == NULL)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     PNameInfo = NULL;
//     _SEH2_TRY
//     {
//         /* Query handle information */
//         Status = NtQueryObject(DirectoryHandle,
//                                ObjectNameInformation,
//                                &NameInfo,
//                                0,
//                                &ReturnLength);
//         /* Only failure we tolerate is length mismatch */
//         if (NT_SUCCESS(Status) || Status == STATUS_INFO_LENGTH_MISMATCH)
//         {
//             /* Allocate big enough buffer */
//             PNameInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, ReturnLength);
//             if (PNameInfo == NULL)
//             {
//                 Status = STATUS_NO_MEMORY;
//                 _SEH2_LEAVE;
//             }
//
//             /* Query again handle information */
//             Status = NtQueryObject(DirectoryHandle,
//                                    ObjectNameInformation,
//                                    PNameInfo,
//                                    ReturnLength,
//                                    &ReturnLength);
//
//             /*
//              * If it succeed, check we have Global??
//              * If so, return success
//              */
//             if (NT_SUCCESS(Status))
//             {
//                 RtlInitUnicodeString(&GlobalString, L"\\GLOBAL??");
//                 *IsGlobal = RtlEqualUnicodeString(&GlobalString, &PNameInfo->Name, FALSE);
//                 Status = STATUS_SUCCESS;
//             }
//         }
//     }
//     _SEH2_FINALLY
//     {
//         if (PNameInfo != NULL)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, PNameInfo);
//         }
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// FindSymbolicLinkEntry(
//     PWSTR NameToFind,
//     PWSTR NamesList,
//     DWORD TotalEntries,
//     PBOOLEAN Found)
// {
//     WCHAR Current;
//     DWORD Entries;
//     PWSTR PartialNamesList;
//
//     /* We need all parameters to be set */
//     if (NameToFind == NULL || NamesList == NULL || Found == NULL)
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Assume failure */
//     *Found = FALSE;
//
//     /* If no entries, job done, nothing found */
//     if (TotalEntries == 0)
//     {
//         return ERROR_SUCCESS;
//     }
//
//     /* Start browsing the names list */
//     Entries = 0;
//     PartialNamesList = NamesList;
//     /* As long as we didn't find the name... */
//     while (wcscmp(NameToFind, PartialNamesList) != 0)
//     {
//         /* We chomped an entry! */
//         ++Entries;
//
//         /* We're out of entries, bail out not to overrun */
//         if (Entries > TotalEntries)
//         {
//             /*
//              * Even though we found nothing,
//              * the function ran fine
//              */
//             return ERROR_SUCCESS;
//         }
//
//         /* Jump to the next string */
//         do
//         {
//             Current = *PartialNamesList;
//             ++PartialNamesList;
//         } while (Current != UNICODE_NULL);
//     }
//
//     /*
//      * We're here because the loop stopped:
//      * it means we found the name in the list
//      */
//     *Found = TRUE;
//     return ERROR_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DefineDosDeviceA(
//     DWORD dwFlags,
//     LPCSTR lpDeviceName,
//     LPCSTR lpTargetPath
//     )
// {
//     BOOL Result;
//     NTSTATUS Status;
//     ANSI_STRING AnsiString;
//     PWSTR TargetPathBuffer;
//     UNICODE_STRING TargetPathU;
//     PUNICODE_STRING DeviceNameU;
//
//     /* Convert DeviceName using static unicode string */
//     RtlInitAnsiString(&AnsiString, lpDeviceName);
//     DeviceNameU = &NtCurrentTeb()->StaticUnicodeString;
//     Status = RtlAnsiStringToUnicodeString(DeviceNameU, &AnsiString, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         /*
//          * If the static unicode string is too small,
//          * it's because the name is too long...
//          * so, return appropriate status!
//          */
//         if (Status == STATUS_BUFFER_OVERFLOW)
//         {
//             SetLastError(ERROR_FILENAME_EXCED_RANGE);
//             return FALSE;
//         }
//
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Convert target path if existing */
//     if (lpTargetPath != NULL)
//     {
//         RtlInitAnsiString(&AnsiString, lpTargetPath);
//         Status = RtlAnsiStringToUnicodeString(&TargetPathU, &AnsiString, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         TargetPathBuffer = TargetPathU.Buffer;
//     }
//     else
//     {
//         TargetPathBuffer = NULL;
//     }
//
//     /* Call W */
//     Result = DefineDosDeviceW(dwFlags, DeviceNameU->Buffer, TargetPathBuffer);
//
//     /* Free target path if allocated */
//     if (TargetPathBuffer != NULL)
//     {
//         RtlFreeUnicodeString(&TargetPathU);
//     }
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DefineDosDeviceW(
//     DWORD dwFlags,
//     LPCWSTR lpDeviceName,
//     LPCWSTR lpTargetPath
//     )
// {
//     ULONG ArgumentCount;
//     ULONG BufferSize;
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_DEFINE_DOS_DEVICE DefineDosDeviceRequest = &ApiMessage.Data.DefineDosDeviceRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//     UNICODE_STRING NtTargetPathU;
//     UNICODE_STRING DeviceNameU;
//     HANDLE hUser32;
//     DEV_BROADCAST_VOLUME dbcv;
//     DWORD dwRecipients;
//     typedef long (WINAPI *BSM_type)(DWORD, LPDWORD, UINT, WPARAM, LPARAM);
//     BSM_type BSM_ptr;
//     BOOLEAN LUIDDeviceMapsEnabled;
//     WCHAR Letter;
//     WPARAM wParam;
//
//     /* Get status about local device mapping */
//     LUIDDeviceMapsEnabled = BaseStaticServerData->LUIDDeviceMapsEnabled;
//
//     /* Validate input & flags */
//     if ((dwFlags & 0xFFFFFFE0) ||
//         ((dwFlags & DDD_EXACT_MATCH_ON_REMOVE) &&
//         !(dwFlags & DDD_REMOVE_DEFINITION)) ||
//         (lpTargetPath == NULL && !(dwFlags & (DDD_LUID_BROADCAST_DRIVE | DDD_REMOVE_DEFINITION))) ||
//         ((dwFlags & DDD_LUID_BROADCAST_DRIVE) &&
//          (lpDeviceName == NULL || lpTargetPath != NULL || dwFlags & (DDD_NO_BROADCAST_SYSTEM | DDD_EXACT_MATCH_ON_REMOVE | DDD_RAW_TARGET_PATH) || !LUIDDeviceMapsEnabled)))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Initialize device unicode string to ease its use */
//     RtlInitUnicodeString(&DeviceNameU, lpDeviceName);
//
//     /* The buffer for CSR call will contain it */
//     BufferSize = DeviceNameU.MaximumLength;
//     ArgumentCount = 1;
//
//     /* If we don't have target path, use empty string */
//     if (lpTargetPath == NULL)
//     {
//         RtlInitUnicodeString(&NtTargetPathU, NULL);
//     }
//     else
//     {
//         /* Else, use it raw if asked to */
//         if (dwFlags & DDD_RAW_TARGET_PATH)
//         {
//             RtlInitUnicodeString(&NtTargetPathU, lpTargetPath);
//         }
//         else
//         {
//             /* Otherwise, use it converted */
//             if (!RtlDosPathNameToNtPathName_U(lpTargetPath,
//                                               &NtTargetPathU,
//                                               NULL,
//                                               NULL))
//             {
//                 WARN("RtlDosPathNameToNtPathName_U() failed\n");
//                 BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
//                 return FALSE;
//             }
//         }
//
//         /* This target path will be the second arg */
//         ArgumentCount = 2;
//         BufferSize += NtTargetPathU.MaximumLength;
//     }
//
//     /* Allocate the capture buffer for our strings */
//     CaptureBuffer = CsrAllocateCaptureBuffer(ArgumentCount,
//                                              BufferSize);
//     if (CaptureBuffer == NULL)
//     {
//         if (!(dwFlags & DDD_RAW_TARGET_PATH))
//         {
//             RtlFreeUnicodeString(&NtTargetPathU);
//         }
//
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Set the flags */
//     DefineDosDeviceRequest->Flags = dwFlags;
//
//     /* Allocate a buffer for the device name */
//     DefineDosDeviceRequest->DeviceName.MaximumLength = CsrAllocateMessagePointer(CaptureBuffer,
//                                                                                  DeviceNameU.MaximumLength,
//                                                                                  (PVOID*)&DefineDosDeviceRequest->DeviceName.Buffer);
//     /* And copy it while upcasing it */
//     RtlUpcaseUnicodeString(&DefineDosDeviceRequest->DeviceName, &DeviceNameU, FALSE);
//
//     /* If we have a target path, copy it too, and free it if allocated */
//     if (NtTargetPathU.Length != 0)
//     {
//         DefineDosDeviceRequest->TargetPath.MaximumLength = CsrAllocateMessagePointer(CaptureBuffer,
//                                                                                      NtTargetPathU.MaximumLength,
//                                                                                      (PVOID*)&DefineDosDeviceRequest->TargetPath.Buffer);
//         RtlCopyUnicodeString(&DefineDosDeviceRequest->TargetPath, &NtTargetPathU);
//
//         if (!(dwFlags & DDD_RAW_TARGET_PATH))
//         {
//             RtlFreeUnicodeString(&NtTargetPathU);
//         }
//     }
//     /* Otherwise, null initialize the string */
//     else
//     {
//         RtlInitUnicodeString(&DefineDosDeviceRequest->TargetPath, NULL);
//     }
//
//     /* Finally,  call the server */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepDefineDosDevice),
//                         sizeof(*DefineDosDeviceRequest));
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     /* Return failure if any */
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         WARN("CsrClientCallServer() failed (Status %lx)\n", ApiMessage.Status);
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     /* Here is the success path, we will always return true */
//
//     /* Should broadcast the event? Only do if not denied and if drive letter */
//     if (!(dwFlags & DDD_NO_BROADCAST_SYSTEM) &&
//         DeviceNameU.Length == 2 * sizeof(WCHAR) &&
//         DeviceNameU.Buffer[1] == L':')
//     {
//         /* Make sure letter is valid and there are no local device mappings */
//         Letter = RtlUpcaseUnicodeChar(DeviceNameU.Buffer[0]) - L'A';
//         if (Letter < 26 && !LUIDDeviceMapsEnabled)
//         {
//             /* Rely on user32 for broadcasting */
//             hUser32 = LoadLibraryW(L"user32.dll");
//             if (hUser32 != 0)
//             {
//                 /* Get the function pointer */
//                 BSM_ptr = (BSM_type)GetProcAddress(hUser32, "BroadcastSystemMessageW");
//                 if (BSM_ptr)
//                 {
//                     /* Set our target */
//                     dwRecipients = BSM_APPLICATIONS;
//
//                     /* And initialize our structure */
//                     dbcv.dbcv_size = sizeof(DEV_BROADCAST_VOLUME);
//                     dbcv.dbcv_devicetype = DBT_DEVTYP_VOLUME;
//                     dbcv.dbcv_reserved = 0;
//
//                     /* Set the volume which had the event */
//                     dbcv.dbcv_unitmask = 1 << Letter;
//                     dbcv.dbcv_flags = DBTF_NET;
//
//                     /* And properly set the event (removal or arrival?) */
//                     wParam = (dwFlags & DDD_REMOVE_DEFINITION) ? DBT_DEVICEREMOVECOMPLETE : DBT_DEVICEARRIVAL;
//
//                     /* And broadcast! */
//                     BSM_ptr(BSF_SENDNOTIFYMESSAGE | BSF_FLUSHDISK,
//                             &dwRecipients,
//                             WM_DEVICECHANGE,
//                             wParam,
//                             (LPARAM)&dbcv);
//                 }
//
//                 /* We're done! */
//                 FreeLibrary(hUser32);
//             }
//         }
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// QueryDosDeviceA(
//     LPCSTR lpDeviceName,
//     LPSTR lpTargetPath,
//     DWORD ucchMax
//     )
// {
//     NTSTATUS Status;
//     USHORT CurrentPosition;
//     ANSI_STRING AnsiString;
//     UNICODE_STRING TargetPathU;
//     PUNICODE_STRING DeviceNameU;
//     DWORD RetLength, CurrentLength, Length;
//     PWSTR DeviceNameBuffer, TargetPathBuffer;
//
//     /* If we want a specific device name, convert it */
//     if (lpDeviceName != NULL)
//     {
//         /* Convert DeviceName using static unicode string */
//         RtlInitAnsiString(&AnsiString, lpDeviceName);
//         DeviceNameU = &NtCurrentTeb()->StaticUnicodeString;
//         Status = RtlAnsiStringToUnicodeString(DeviceNameU, &AnsiString, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             /*
//              * If the static unicode string is too small,
//              * it's because the name is too long...
//              * so, return appropriate status!
//              */
//             if (Status == STATUS_BUFFER_OVERFLOW)
//             {
//                 SetLastError(ERROR_FILENAME_EXCED_RANGE);
//                 return FALSE;
//             }
//
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         DeviceNameBuffer = DeviceNameU->Buffer;
//     }
//     else
//     {
//         DeviceNameBuffer = NULL;
//     }
//
//     /* Allocate the output buffer for W call */
//     TargetPathBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, ucchMax * sizeof(WCHAR));
//     if (TargetPathBuffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     /* Call W */
//     Length = QueryDosDeviceW(DeviceNameBuffer, TargetPathBuffer, ucchMax);
//     /* We'll return that length in case of a success */
//     RetLength = Length;
//
//     /* Handle the case where we would fill output buffer completly */
//     if (Length != 0 && Length == ucchMax)
//     {
//         /* This will be our work length (but not the one we return) */
//         --Length;
//         /* Already 0 the last char */
//         lpTargetPath[Length] = ANSI_NULL;
//     }
//
//     /* If we had an output, start the convert loop */
//     if (Length != 0)
//     {
//         /*
//          * We'll have to loop because TargetPathBuffer may contain
//          * several strings (NULL separated)
//          * We'll start at position 0
//          */
//         CurrentPosition = 0;
//         while (CurrentPosition < Length)
//         {
//             /* Get the maximum length */
//             CurrentLength = min(Length - CurrentPosition, MAXUSHORT / 2);
//
//             /* Initialize our output string */
//             AnsiString.Length = 0;
//             AnsiString.MaximumLength = CurrentLength + sizeof(ANSI_NULL);
//             AnsiString.Buffer = &lpTargetPath[CurrentPosition];
//
//             /* Initialize input string that will be converted */
//             TargetPathU.Length = CurrentLength * sizeof(WCHAR);
//             TargetPathU.MaximumLength = CurrentLength * sizeof(WCHAR) + sizeof(UNICODE_NULL);
//             TargetPathU.Buffer = &TargetPathBuffer[CurrentPosition];
//
//             /* Convert to ANSI */
//             Status = RtlUnicodeStringToAnsiString(&AnsiString, &TargetPathU, FALSE);
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 /* In case of a failure, forget about everything */
//                 RetLength = 0;
//
//                 goto Leave;
//             }
//
//             /* Move to the next string */
//             CurrentPosition += CurrentLength;
//         }
//     }
//
// Leave:
//     /* Free our intermediate buffer and leave */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, TargetPathBuffer);
//
//     return RetLength;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// QueryDosDeviceW(
//     LPCWSTR lpDeviceName,
//     LPWSTR lpTargetPath,
//     DWORD ucchMax
//     )
// {
//     PWSTR Ptr;
//     PVOID Buffer;
//     NTSTATUS Status;
//     USHORT i, TotalEntries;
//     UNICODE_STRING UnicodeString;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE DirectoryHandle, DeviceHandle;
//     BOOLEAN IsGlobal, GlobalNeeded, Found;
//     POBJECT_DIRECTORY_INFORMATION DirInfo;
//     OBJECT_DIRECTORY_INFORMATION NullEntry = {{0}};
//     ULONG ReturnLength, NameLength, Length, Context, BufferLength;
//
//     /* Open the '\??' directory */
//     RtlInitUnicodeString(&UnicodeString, L"\\??");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &UnicodeString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenDirectoryObject(&DirectoryHandle,
//                                    DIRECTORY_QUERY,
//                                    &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN("NtOpenDirectoryObject() failed (Status %lx)\n", Status);
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     Buffer = NULL;
//     _SEH2_TRY
//     {
//         if (lpDeviceName != NULL)
//         {
//             /* Open the lpDeviceName link object */
//             RtlInitUnicodeString(&UnicodeString, lpDeviceName);
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &UnicodeString,
//                                        OBJ_CASE_INSENSITIVE,
//                                        DirectoryHandle,
//                                        NULL);
//             Status = NtOpenSymbolicLinkObject(&DeviceHandle,
//                                               SYMBOLIC_LINK_QUERY,
//                                               &ObjectAttributes);
//             if (!NT_SUCCESS(Status))
//             {
//                 WARN("NtOpenSymbolicLinkObject() failed (Status %lx)\n", Status);
//                 _SEH2_LEAVE;
//             }
//
//             /*
//              * Make sure we don't overrun the output buffer, so convert our DWORD
//              * size to USHORT size properly
//              */
//             Length = (ucchMax <= MAXULONG / sizeof(WCHAR)) ? (ucchMax * sizeof(WCHAR)) : MAXULONG;
//
//             /* Query link target */
//             UnicodeString.Length = 0;
//             UnicodeString.MaximumLength = Length <= MAXUSHORT ? Length : MAXUSHORT;
//             UnicodeString.Buffer = lpTargetPath;
//
//             ReturnLength = 0;
//             Status = NtQuerySymbolicLinkObject(DeviceHandle,
//                                                &UnicodeString,
//                                                &ReturnLength);
//             NtClose(DeviceHandle);
//             if (!NT_SUCCESS(Status))
//             {
//                 WARN("NtQuerySymbolicLinkObject() failed (Status %lx)\n", Status);
//                 _SEH2_LEAVE;
//             }
//
//             TRACE("ReturnLength: %lu\n", ReturnLength);
//             TRACE("TargetLength: %hu\n", UnicodeString.Length);
//             TRACE("Target: '%wZ'\n", &UnicodeString);
//
//             Length = ReturnLength / sizeof(WCHAR);
//             /* Make sure we null terminate output buffer */
//             if (Length == 0 || lpTargetPath[Length - 1] != UNICODE_NULL)
//             {
//                 if (Length >= ucchMax)
//                 {
//                     TRACE("Buffer is too small\n");
//                     Status = STATUS_BUFFER_TOO_SMALL;
//                     _SEH2_LEAVE;
//                 }
//
//                 /* Append null-character */
//                 lpTargetPath[Length] = UNICODE_NULL;
//                 Length++;
//             }
//
//             if (Length < ucchMax)
//             {
//                 /* Append null-character */
//                 lpTargetPath[Length] = UNICODE_NULL;
//                 Length++;
//             }
//
//             _SEH2_LEAVE;
//         }
//
//         /*
//          * If LUID device maps are enabled,
//          * ?? may not point to BaseNamedObjects
//          * It may only be local DOS namespace.
//          * And thus, it might be required to browse
//          * Global?? for global devices
//          */
//         GlobalNeeded = FALSE;
//         if (BaseStaticServerData->LUIDDeviceMapsEnabled)
//         {
//             /* Assume ?? == Global?? */
//             IsGlobal = TRUE;
//             /* Check if it's the case */
//             Status = IsGlobalDeviceMap(DirectoryHandle, &IsGlobal);
//             if (NT_SUCCESS(Status) && !IsGlobal)
//             {
//                 /* It's not, we'll have to browse Global?? too! */
//                 GlobalNeeded = TRUE;
//             }
//         }
//
//         /*
//          * Make sure we don't overrun the output buffer, so convert our DWORD
//          * size to USHORT size properly
//          */
//         BufferLength = (ucchMax <= MAXULONG / sizeof(WCHAR)) ? (ucchMax * sizeof(WCHAR)) : MAXULONG;
//         Length = 0;
//         Ptr = lpTargetPath;
//
//         Context = 0;
//         TotalEntries = 0;
//
//         /*
//          * We'll query all entries at once, with a rather big buffer
//          * If it's too small, we'll grow it by 2.
//          * Limit the number of attempts to 3.
//          */
//         for (i = 0; i < 3; ++i)
//         {
//             /* Allocate the query buffer */
//             Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength);
//             if (Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 _SEH2_LEAVE;
//             }
//
//             /* Perform the query */
//             Status = NtQueryDirectoryObject(DirectoryHandle,
//                                             Buffer,
//                                             BufferLength,
//                                             FALSE,
//                                             TRUE,
//                                             &Context,
//                                             &ReturnLength);
//             /* Only failure accepted is: no more entries */
//             if (!NT_SUCCESS(Status))
//             {
//                 if (Status != STATUS_NO_MORE_ENTRIES)
//                 {
//                     _SEH2_LEAVE;
//                 }
//
//                 /*
//                  * Which is a success! But break out,
//                  * it means our query returned no results
//                  * so, nothing to parse.
//                  */
//                 Status = STATUS_SUCCESS;
//                 break;
//             }
//
//             /* In case we had them all, start browsing for devices */
//             if (Status != STATUS_MORE_ENTRIES)
//             {
//                 DirInfo = Buffer;
//
//                 /* Loop until we find the nul entry (terminating entry) */
//                 while (TRUE)
//                 {
//                     /* It's an entry full of zeroes */
//                     if (RtlCompareMemory(&NullEntry, DirInfo, sizeof(NullEntry)) == sizeof(NullEntry))
//                     {
//                         break;
//                     }
//
//                     /* Only handle symlinks */
//                     if (!wcscmp(DirInfo->TypeName.Buffer, L"SymbolicLink"))
//                     {
//                         TRACE("Name: '%wZ'\n", &DirInfo->Name);
//
//                         /* Get name length in chars to only comparisons */
//                         NameLength = DirInfo->Name.Length / sizeof(WCHAR);
//
//                         /* Make sure we don't overrun output buffer */
//                         if (Length > ucchMax ||
//                             NameLength > ucchMax - Length ||
//                             ucchMax - NameLength - Length < sizeof(WCHAR))
//                         {
//                             Status = STATUS_BUFFER_TOO_SMALL;
//                             _SEH2_LEAVE;
//                         }
//
//                         /* Copy and NULL terminate string */
//                         memcpy(Ptr, DirInfo->Name.Buffer, DirInfo->Name.Length);
//                         Ptr[NameLength] = UNICODE_NULL;
//
//                         Ptr += (NameLength + 1);
//                         Length += (NameLength + 1);
//
//                         /*
//                          * Keep the entries count, in case we would have to
//                          * handle GLOBAL?? too
//                          */
//                         ++TotalEntries;
//                     }
//
//                     /* Move to the next entry */
//                     ++DirInfo;
//                 }
//
//                 /*
//                  * No need to loop again here, we got all the entries
//                  * Note: we don't free the buffer here, because we may
//                  * need it for GLOBAL??, so we save a few cycles here.
//                  */
//                 break;
//             }
//
//             /* Failure path here, we'll need bigger buffer */
//             RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//             Buffer = NULL;
//
//             /* We can't have bigger than that one, so leave */
//             if (BufferLength == MAXULONG)
//             {
//                 break;
//             }
//
//             /* Prevent any overflow while computing new size */
//             if (MAXULONG - BufferLength < BufferLength)
//             {
//                 BufferLength = MAXULONG;
//             }
//             else
//             {
//                 BufferLength *= 2;
//             }
//         }
//
//         /*
//          * Out of the hot loop, but with more entries left?
//          * that's an error case, leave here!
//          */
//         if (Status == STATUS_MORE_ENTRIES)
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//             _SEH2_LEAVE;
//         }
//
//         /* Now, if we had to handle GLOBAL??, go for it! */
//         if (BaseStaticServerData->LUIDDeviceMapsEnabled && NT_SUCCESS(Status) && GlobalNeeded)
//         {
//             NtClose(DirectoryHandle);
//             DirectoryHandle = 0;
//
//             RtlInitUnicodeString(&UnicodeString, L"\\GLOBAL??");
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &UnicodeString,
//                                        OBJ_CASE_INSENSITIVE,
//                                        NULL,
//                                        NULL);
//             Status = NtOpenDirectoryObject(&DirectoryHandle,
//                                            DIRECTORY_QUERY,
//                                            &ObjectAttributes);
//             if (!NT_SUCCESS(Status))
//             {
//                 WARN("NtOpenDirectoryObject() failed (Status %lx)\n", Status);
//                 _SEH2_LEAVE;
//             }
//
//             /*
//              * We'll query all entries at once, with a rather big buffer
//              * If it's too small, we'll grow it by 2.
//              * Limit the number of attempts to 3.
//              */
//             for (i = 0; i < 3; ++i)
//             {
//                 /* If we had no buffer from previous attempt, allocate one */
//                 if (Buffer == NULL)
//                 {
//                     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength);
//                     if (Buffer == NULL)
//                     {
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                         _SEH2_LEAVE;
//                     }
//                 }
//
//                 /* Perform the query */
//                 Status = NtQueryDirectoryObject(DirectoryHandle,
//                                                 Buffer,
//                                                 BufferLength,
//                                                 FALSE,
//                                                 TRUE,
//                                                 &Context,
//                                                 &ReturnLength);
//                 /* Only failure accepted is: no more entries */
//                 if (!NT_SUCCESS(Status))
//                 {
//                     if (Status != STATUS_NO_MORE_ENTRIES)
//                     {
//                         _SEH2_LEAVE;
//                     }
//
//                     /*
//                      * Which is a success! But break out,
//                      * it means our query returned no results
//                      * so, nothing to parse.
//                      */
//                     Status = STATUS_SUCCESS;
//                     break;
//                 }
//
//                 /* In case we had them all, start browsing for devices */
//                 if (Status != STATUS_MORE_ENTRIES)
//                 {
//                     DirInfo = Buffer;
//
//                     /* Loop until we find the nul entry (terminating entry) */
//                     while (TRUE)
//                     {
//                         /* It's an entry full of zeroes */
//                         if (RtlCompareMemory(&NullEntry, DirInfo, sizeof(NullEntry)) == sizeof(NullEntry))
//                         {
//                             break;
//                         }
//
//                         /* Only handle symlinks */
//                         if (!wcscmp(DirInfo->TypeName.Buffer, L"SymbolicLink"))
//                         {
//                             TRACE("Name: '%wZ'\n", &DirInfo->Name);
//
//                             /*
//                              * Now, we previously already browsed ??, and we
//                              * don't want to devices twice, so we'll check
//                              * the output buffer for duplicates.
//                              * We'll add our entry only if we don't have already
//                              * returned it.
//                              */
//                             if (FindSymbolicLinkEntry(DirInfo->Name.Buffer,
//                                                       lpTargetPath,
//                                                       TotalEntries,
//                                                       &Found) == ERROR_SUCCESS &&
//                                 !Found)
//                             {
//                                 /* Get name length in chars to only comparisons */
//                                 NameLength = DirInfo->Name.Length / sizeof(WCHAR);
//
//                                 /* Make sure we don't overrun output buffer */
//                                 if (Length > ucchMax ||
//                                     NameLength > ucchMax - Length ||
//                                     ucchMax - NameLength - Length < sizeof(WCHAR))
//                                 {
//                                     RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//                                     NtClose(DirectoryHandle);
//                                     BaseSetLastNTError(STATUS_BUFFER_TOO_SMALL);
//                                     return 0;
//                                 }
//
//                                 /* Copy and NULL terminate string */
//                                 memcpy(Ptr, DirInfo->Name.Buffer, DirInfo->Name.Length);
//                                 Ptr[NameLength] = UNICODE_NULL;
//
//                                 Ptr += (NameLength + 1);
//                                 Length += (NameLength + 1);
//                             }
//                         }
//
//                         /* Move to the next entry */
//                         ++DirInfo;
//                     }
//
//                     /* No need to loop again here, we got all the entries */
//                     break;
//                 }
//
//                 /* Failure path here, we'll need bigger buffer */
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//                 Buffer = NULL;
//
//                 /* We can't have bigger than that one, so leave */
//                 if (BufferLength == MAXULONG)
//                 {
//                     break;
//                 }
//
//                 /* Prevent any overflow while computing new size */
//                 if (MAXULONG - BufferLength < BufferLength)
//                 {
//                     BufferLength = MAXULONG;
//                 }
//                 else
//                 {
//                     BufferLength *= 2;
//                 }
//             }
//
//             /*
//              * Out of the hot loop, but with more entries left?
//              * that's an error case, leave here!
//              */
//             if (Status == STATUS_MORE_ENTRIES)
//             {
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 _SEH2_LEAVE;
//             }
//         }
//
//         /* If we failed somewhere, just leave */
//         if (!NT_SUCCESS(Status))
//         {
//             _SEH2_LEAVE;
//         }
//
//         /* If we returned no entries, time to write the empty string */
//         if (Length == 0)
//         {
//             /* Unless output buffer is too small! */
//             if (ucchMax <= 0)
//             {
//                 Status = STATUS_BUFFER_TOO_SMALL;
//                 _SEH2_LEAVE;
//             }
//
//             /* Emptry string is one char (terminator!) */
//             *Ptr = UNICODE_NULL;
//             ++Ptr;
//             Length = 1;
//         }
//
//         /*
//          * If we have enough room, we need to double terminate the buffer:
//          * that's a MULTI_SZ buffer, its end is marked by double NULL.
//          * One was already added during the "copy string" process.
//          * If we don't have enough room: that's a failure case.
//          */
//         if (Length < ucchMax)
//         {
//             *Ptr = UNICODE_NULL;
//             ++Ptr;
//         }
//         else
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//     }
//     _SEH2_FINALLY
//     {
//         if (DirectoryHandle != 0)
//         {
//             NtClose(DirectoryHandle);
//         }
//
//         if (Buffer != NULL)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             Length = 0;
//             BaseSetLastNTError(Status);
//         }
//     }
//     _SEH2_END;
//
//     return Length;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
