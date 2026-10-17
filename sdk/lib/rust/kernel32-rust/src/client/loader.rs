use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
pub use crate::file::filemap::CreateFileMappingW;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// All NT/Win32 record types (LIST_ENTRY, UNICODE_STRING, ANSI_STRING,
// LDR_DATA_TABLE_ENTRY, PEB/TEB, PROCESS_INFORMATION, IMAGE_NT_HEADERS, FARPROC,
// the integer typedefs) come from the bindgen `ntdll_sys::nt::*` glob so the
// Rtl*/Ldr*/Nt* calls bind against the real C layouts. Only the symbols the C
// file references that bindgen does NOT provide are declared here.

// STATUS_* codes missing from ntdll-sys (ntstatus.h cast-macros not bindgen-able).
pub const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000007Bu32 as NTSTATUS;
pub const STATUS_DLL_NOT_FOUND: NTSTATUS = 0xC0000135u32 as NTSTATUS;
pub const STATUS_ORDINAL_NOT_FOUND: NTSTATUS = 0xC0000138u32 as NTSTATUS;
pub const STATUS_ENTRYPOINT_NOT_FOUND: NTSTATUS = 0xC0000139u32 as NTSTATUS;

// kernel32.h: BasepGetModuleHandleExParameterValidation return codes.
pub const BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR: DWORD = 1;
pub const BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_SUCCESS: DWORD = 2;
pub const BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_CONTINUE: DWORD = 3;

// kernel32.h: tagLOADPARMS32 (LoadModule parameter block).
#[repr(C)]
pub struct LOADPARMS32 {
    pub lpEnvAddress: LPSTR,
    pub lpCmdLine: LPSTR,
    pub wMagicValue: WORD,
    pub wCmdShow: WORD,
    pub dwReserved: DWORD,
}

// ldrtypes.h: LDR_IS_DATAFILE(handle) = ((ULONG_PTR)handle) & 1
#[inline]
pub fn LDR_IS_DATAFILE<T>(handle: *mut T) -> ULONG_PTR {
    (handle as ULONG_PTR) & 1
}

// CONTAINING_RECORD(addr, type, field): recover the struct base from a member ptr.
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $ty:ty, $field:ident) => {
        ($addr as *mut u8).offset(-(core::mem::offset_of!($ty, $field) as isize)) as *mut $ty
    };
}

// winnt.h scalar helpers.
#[inline]
pub fn HIWORD(l: usize) -> u16 {
    ((l >> 16) & 0xFFFF) as u16
}
#[inline]
pub unsafe fn PtrToUlong(p: *mut c_void) -> ULONG {
    (p as usize) as ULONG
}

// RtlMoveMemory(dest, src, len) = memmove (overlap-safe). RtlZeroMemory/RtlCopyMemory
// live in k32_externs.
#[inline]
pub unsafe fn RtlMoveMemory(Destination: PVOID, Source: PVOID, Length: SIZE_T) {
    memmove(Destination, Source, Length as usize);
}
pub use crate::k32_externs::{RtlZeroMemory, RtlCopyMemory};

// PRTL_CONVERT_STRINGA: NTAPI fn-ptr typedef (kernel32.h).
pub type PRTL_CONVERT_STRINGA = Option<
    unsafe extern "system" fn(
        AnsiString: PANSI_STRING,
        UnicodeString: PCUNICODE_STRING,
        AllocateMemory: BOOLEAN,
    ) -> NTSTATUS,
>;

// proc.c's globals (cross-file). BasepExeLdrEntry + UserWaitForInputIdleRoutine are
// owned by k32_globals; BasepUnicodeStringTo8BitString is a fn-ptr global in utils.c.
pub use crate::k32_globals::{BasepExeLdrEntry, UserWaitForInputIdleRoutine};

extern "C" {
    pub static mut BasepUnicodeStringTo8BitString: PRTL_CONVERT_STRINGA;
}

extern "system" {
    // Base* helpers defined in still-C TUs (utils.c / path.c / appcache.c).
    pub fn Basep8BitStringToStaticUnicodeString(AnsiString: LPCSTR) -> PUNICODE_STRING;
    pub fn BaseComputeProcessDllPath(FullPath: LPWSTR, Environment: PVOID) -> LPWSTR;
    pub fn BasepLocateExeLdrEntry(
        Entry: PLDR_DATA_TABLE_ENTRY,
        Context: PVOID,
        StopEnumeration: *mut BOOLEAN,
    );

    // Cross-file Win32 exports defined in other kernel32 .c TUs.
    pub fn GetWindowsDirectoryA(lpBuffer: LPSTR, uSize: UINT) -> UINT;
    pub fn SearchPathA(
        lpPath: LPCSTR,
        lpFileName: LPCSTR,
        lpExtension: LPCSTR,
        nBufferLength: DWORD,
        lpBuffer: LPSTR,
        lpFilePart: *mut LPSTR,
    ) -> DWORD;
    pub fn UnmapViewOfFile(lpBaseAddress: LPCVOID) -> BOOL;
    pub fn CreateProcessA(
        lpApplicationName: LPCSTR,
        lpCommandLine: LPSTR,
        lpProcessAttributes: LPSECURITY_ATTRIBUTES,
        lpThreadAttributes: LPSECURITY_ATTRIBUTES,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: LPVOID,
        lpCurrentDirectory: LPCSTR,
        lpStartupInfo: LPSTARTUPINFOA,
        lpProcessInformation: LPPROCESS_INFORMATION,
    ) -> BOOL;
    // CreateFile == CreateFileA in the C (ANSI A-suffix elided by the macro).
    #[link_name = "CreateFileA"]
    pub fn CreateFile(
        lpFileName: LPCSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
}
pub use crate::k32_externs::{
    BaseSetLastNTError, SetLastError, GetLastError, RtlGetProcessHeap,
    CloseHandle, CreateFileW, ExitThread, SearchPathW, INVALID_HANDLE_VALUE,
};

#[no_mangle]
pub unsafe extern "system" fn BasepInitializeTermsrvFpns() -> NTSTATUS {
    UNIMPLEMENTED!();
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn BasepGetModuleHandleExParameterValidation(
    dwFlags: DWORD,
    lpwModuleName: LPCWSTR,
    phModule: *mut HMODULE,
) -> DWORD {
    // Set phModule to 0 if it's not a NULL pointer
    if !phModule.is_null() {
        *phModule = 0 as HMODULE;
    }

    // Check for invalid flags combination
    if (dwFlags & !(GET_MODULE_HANDLE_EX_FLAG_PIN |
                    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT |
                    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS)) != 0 ||
        ((dwFlags & GET_MODULE_HANDLE_EX_FLAG_PIN) != 0 &&
         (dwFlags & GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT) != 0) ||
         (lpwModuleName.is_null() && (dwFlags & GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS) != 0)
    {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER_1);
        return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR;
    }

    // Check 2nd parameter
    if phModule.is_null() {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER_2);
        return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR;
    }

    // Return what we have according to the module name
    if !lpwModuleName.is_null() {
        return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_CONTINUE;
    }

    // No name given, so put ImageBaseAddress there
    *phModule = (*NtCurrentPeb()).ImageBaseAddress as HMODULE;

    return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn BasepMapModuleHandle(
    hModule: HMODULE,
    AsDataFile: BOOLEAN,
) -> PVOID {
    if hModule.is_null() {
        return (*NtCurrentPeb()).ImageBaseAddress;
    }
    if LDR_IS_DATAFILE(hModule) != 0 && AsDataFile == 0 {
        return core::ptr::null_mut();
    }
    hModule as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn DisableThreadLibraryCalls(
    hLibModule: HMODULE,
) -> BOOL {
    let Status: NTSTATUS;

    /* Disable thread library calls */
    Status = LdrDisableThreadCalloutsForDll(hLibModule as *mut core::ffi::c_void);

    /* If it wasn't success - set last error and return failure */
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0i32; /* FALSE */
    }

    /* Return success */
    return 1i32; /* TRUE */
}

#[no_mangle]
pub unsafe extern "system" fn LoadLibraryA(
    lpLibFileName: LPCSTR,
) -> HINSTANCE {
    static TwainDllName: [i8; 13] = unsafe { core::mem::transmute(*b"twain_32.dll\0") };
    let mut PathBuffer: LPSTR;
    let mut Len: UINT;
    let mut Result: HINSTANCE;

    /* Treat twain_32.dll in a special way (what a surprise...) */
    if !lpLibFileName.is_null() && _strcmpi(lpLibFileName, TwainDllName.as_ptr()) == 0 {
        /* Allocate space for the buffer */
        PathBuffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            (MAX_PATH as UINT + core::mem::size_of::<i8>() as UINT) as SIZE_T,
        ) as LPSTR;
        if !PathBuffer.is_null() {
            /* Get windows dir in this buffer */
            Len = GetWindowsDirectoryA(PathBuffer, MAX_PATH);
            if Len != 0
                && (Len as UINT)
                    < ((MAX_PATH as UINT)
                        - (core::mem::size_of_val(&TwainDllName) as UINT)
                        - core::mem::size_of::<i8>() as UINT)
            {
                /* We successfully got windows directory. Concatenate twain_32.dll to it */
                *PathBuffer.offset(Len as isize) = '\\' as i8;
                strcpy(
                    PathBuffer.offset(Len as isize + 1),
                    TwainDllName.as_ptr(),
                );

                /* And recursively call ourselves with a new string */
                Result = LoadLibraryA(PathBuffer);

                /* If it was successful -  free memory and return result */
                if !Result.is_null() {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
                    return Result;
                }
            }

            /* Free allocated buffer */
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
        }
    }

    /* Call the Ex version of the API */
    return LoadLibraryExA(lpLibFileName, core::ptr::null_mut(), 0);
}

#[no_mangle]
pub unsafe extern "system" fn LoadLibraryExA(
    lpLibFileName: LPCSTR,
    hFile: HANDLE,
    dwFlags: DWORD,
) -> HINSTANCE {
    let FileNameW: PUNICODE_STRING;

    /* Convert file name to unicode */
    FileNameW = Basep8BitStringToStaticUnicodeString(lpLibFileName);
    if FileNameW.is_null() {
        return core::ptr::null_mut();
    }

    /* And call W version of the API */
    return LoadLibraryExW((*FileNameW).Buffer, hFile, dwFlags);
}

#[no_mangle]
pub unsafe extern "system" fn LoadLibraryW(
    lpLibFileName: LPCWSTR,
) -> HINSTANCE {
    LoadLibraryExW(lpLibFileName, core::ptr::null_mut(), 0)
}
unsafe fn BasepLoadLibraryAsDatafile(
    Path: PWSTR,
    Name: LPCWSTR,
    hModule: *mut HMODULE,
) -> NTSTATUS {
    let mut FilenameW: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut hMapping: HANDLE;
    let mut Status: NTSTATUS;
    let mut lpBaseAddress: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ViewSize: SIZE_T = 0;

    *hModule = core::ptr::null_mut();

    if SearchPathW(
        Path,
        Name,
        crate::w!(".DLL\0").as_ptr() as LPCWSTR,
        (core::mem::size_of::<[u16; MAX_PATH as usize]>() / core::mem::size_of::<u16>()) as u32,
        FilenameW.as_mut_ptr(),
        core::ptr::null_mut(),
    ) == 0
    {
        return (*NtCurrentTeb()).LastStatusValue as NTSTATUS;
    }

    hFile = CreateFileW(
        FilenameW.as_ptr(),
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_DELETE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE {
        return (*NtCurrentTeb()).LastStatusValue as NTSTATUS;
    }

    hMapping = CreateFileMappingW(
        hFile,
        core::ptr::null_mut(),
        PAGE_READONLY,
        0,
        0,
        core::ptr::null_mut(),
    );

    CloseHandle(hFile);

    if hMapping.is_null() {
        return (*NtCurrentTeb()).LastStatusValue as NTSTATUS;
    }

    Status = NtMapViewOfSection(
        hMapping,
        NtCurrentProcess(),
        &mut lpBaseAddress,
        0,
        0,
        core::ptr::null_mut(),
        &mut ViewSize,
        ViewShare,
        0,
        PAGE_READONLY,
    );

    CloseHandle(hMapping);

    if !NT_SUCCESS(Status) {
        return (*NtCurrentTeb()).LastStatusValue as NTSTATUS;
    }

    if RtlImageNtHeader(lpBaseAddress).is_null() {
        UnmapViewOfFile(lpBaseAddress);
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    *hModule = (lpBaseAddress as ULONG_PTR | 1) as HMODULE;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn LoadLibraryExW(
    lpLibFileName: LPCWSTR,
    hFile: HANDLE,
    dwFlags: DWORD,
) -> HINSTANCE {
    let mut DllName: UNICODE_STRING = core::mem::zeroed();
    let mut hInst: HINSTANCE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut SearchPath: PWSTR;
    let mut DllCharacteristics: ULONG = 0;
    let mut FreeString: BOOL = 0;

    /* Check for any flags LdrLoadDll might be interested in */
    if (dwFlags & DONT_RESOLVE_DLL_REFERENCES) != 0 {
        /* Tell LDR to treat it as an EXE */
        DllCharacteristics = IMAGE_FILE_EXECUTABLE_IMAGE as ULONG;
    }

    /* Build up a unicode dll name from null-terminated string */
    RtlInitUnicodeString(&mut DllName, lpLibFileName as LPWSTR);

    /* Lazy-initialize BasepExeLdrEntry */
    if BasepExeLdrEntry.is_null() {
        // BasepLocateExeLdrEntry has the NTAPI ABI (== PLDR_ENUM_CALLBACK), but a
        // nominally distinct extern type — transmute the fn-ptr to the callback type.
        let cb: PLDR_ENUM_CALLBACK = core::mem::transmute(BasepLocateExeLdrEntry as *const ());
        LdrEnumerateLoadedModules(0, cb, (*NtCurrentPeb()).ImageBaseAddress);
    }

    /* Check if that module is our exe*/
    if !BasepExeLdrEntry.is_null()
        && (dwFlags & LOAD_LIBRARY_AS_DATAFILE) == 0
        && DllName.Length == (*BasepExeLdrEntry).FullDllName.Length
    {
        /* Lengths match and it's not a datafile, so perform name comparison */
        if RtlEqualUnicodeString(&DllName, &(*BasepExeLdrEntry).FullDllName, 1) != 0 {
            /* That's us! */
            return (*BasepExeLdrEntry).DllBase as HINSTANCE;
        }
    }

    /* Check for trailing spaces and remove them if necessary */
    if *DllName.Buffer.offset((DllName.Length as usize / core::mem::size_of::<WCHAR>()) as isize - 1) == ' ' as u16 {
        RtlCreateUnicodeString(&mut DllName, lpLibFileName as LPWSTR);
        while DllName.Length > core::mem::size_of::<WCHAR>() as u16
            && *DllName.Buffer.offset((DllName.Length as usize / core::mem::size_of::<WCHAR>()) as isize - 1) == ' ' as u16
        {
            DllName.Length -= core::mem::size_of::<WCHAR>() as u16;
        }
        *DllName.Buffer.offset((DllName.Length as usize / core::mem::size_of::<WCHAR>()) as isize) = UNICODE_NULL;
        FreeString = 1;
    }

    /* Compute the load path: the LOAD_LIBRARY_SEARCH_* order when those flags (passed
     * here, or the SetDefaultDllDirectories default) apply, else the legacy order. */
    SearchPath = match super::dlldirs::search_flags_path(DllName.Buffer, dwFlags) {
        Ok(p) => p,
        Err(e) => {
            BaseSetLastNTError(e);
            if FreeString != 0 {
                RtlFreeUnicodeString(&mut DllName);
            }
            return core::ptr::null_mut();
        }
    };
    if SearchPath.is_null() {
        SearchPath = BaseComputeProcessDllPath(
            if (dwFlags & LOAD_WITH_ALTERED_SEARCH_PATH) != 0 {
                DllName.Buffer
            } else {
                core::ptr::null_mut()
            },
            core::ptr::null_mut(),
        );
    }
    if SearchPath.is_null() {
        /* Getting DLL path failed, so set last error, free mem and return */
        BaseSetLastNTError(STATUS_NO_MEMORY);
        if FreeString != 0 {
            RtlFreeUnicodeString(&mut DllName);
        }
        return core::ptr::null_mut();
    }

    /* SEH try block. `_SEH2_YIELD(goto done)` => `break 'done`
     * (the cleanup tail follows the block). */
    'done: {
        if (dwFlags & LOAD_LIBRARY_AS_DATAFILE) != 0 {
            /* If the image is loaded as a datafile, try to get its handle */
            Status = LdrGetDllHandleEx(0, SearchPath, core::ptr::null_mut(), &mut DllName, &mut hInst as *mut HINSTANCE as *mut PVOID);
            if !NT_SUCCESS(Status) {
                /* It's not loaded yet - so load it up */
                Status = BasepLoadLibraryAsDatafile(SearchPath, DllName.Buffer, &mut hInst);
            }
            /* _SEH2_YIELD - jump to done, skipping the normal LdrLoadDll path */
            break 'done;
        }

        /* Call the API Properly */
        Status = LdrLoadDll(
            SearchPath,
            &mut DllCharacteristics,
            &mut DllName,
            &mut hInst as *mut HINSTANCE as *mut PVOID,
        );
    }

    // _SEH2_EXCEPT handling - if exception occurred, use exception code
    // For simplicity, we assume no exception occurred in this transliteration
    // The actual SEH handling would need __try/__except which doesn't exist in Rust

    // done:
    /* Free SearchPath buffer */
    RtlFreeHeap(RtlGetProcessHeap(), 0, SearchPath as PVOID);

    /* Free DllName string if it was dynamically allocated */
    if FreeString != 0 {
        RtlFreeUnicodeString(&mut DllName);
    }

    /* Set last error in failure case */
    if !NT_SUCCESS(Status) {
        // DPRINT1("LoadLibraryExW(%ls) failing with status %lx\n", lpLibFileName, Status);
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Return loaded module handle */
    hInst
}

#[no_mangle]
pub unsafe extern "system" fn GetProcAddress(
    hModule: HMODULE,
    lpProcName: LPCSTR,
) -> FARPROC {
    let mut ProcedureName: ANSI_STRING = core::mem::zeroed();
    let mut ProcNamePtr: *mut ANSI_STRING = core::ptr::null_mut();
    let mut fnExp: FARPROC = None;
    let mut Status: NTSTATUS;
    let mut hMapped: PVOID;
    let mut Ordinal: ULONG = 0;

    if (lpProcName as usize) > MAXUSHORT as usize {
        /* Look up by name */
        RtlInitAnsiString(&mut ProcedureName, lpProcName as LPSTR);
        ProcNamePtr = &mut ProcedureName;
    } else {
        /* Look up by ordinal */
        Ordinal = PtrToUlong(lpProcName as *mut c_void);
    }

    /* Map provided handle */
    hMapped = BasepMapModuleHandle(hModule, 0);

    /* Get the proc address */
    Status = LdrGetProcedureAddress(
        hMapped,
        ProcNamePtr,
        Ordinal,
        &mut fnExp as *mut FARPROC as *mut PVOID,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return None;
    }

    /* Check for a special case when returned pointer is
       the same as image's base address */
    if fnExp.map_or(core::ptr::null_mut(), |f| f as *mut c_void) == hMapped {
        /* Set correct error code */
        if HIWORD(lpProcName as usize) != 0 {
            BaseSetLastNTError(STATUS_ENTRYPOINT_NOT_FOUND);
        } else {
            BaseSetLastNTError(STATUS_ORDINAL_NOT_FOUND);
        }

        return None;
    }

    /* All good, return procedure pointer */
    return fnExp;
}

#[no_mangle]
pub unsafe extern "system" fn FreeLibrary(hLibModule: HINSTANCE) -> BOOL {
    let mut Status: NTSTATUS;
    let NtHeaders: PIMAGE_NT_HEADERS;

    if LDR_IS_DATAFILE(hLibModule) != 0 {
        /* This is a LOAD_LIBRARY_AS_DATAFILE module, check if it's a valid one */
        NtHeaders = RtlImageNtHeader((hLibModule as ULONG_PTR & !1) as *mut core::ffi::c_void);

        if !NtHeaders.is_null() {
            /* Unmap view */
            Status = NtUnmapViewOfSection(NtCurrentProcess(), (hLibModule as ULONG_PTR & !1) as *mut core::ffi::c_void);

            /* Unload alternate resource module */
            LdrUnloadAlternateResourceModule(hLibModule as PVOID);
        } else {
            Status = STATUS_INVALID_IMAGE_FORMAT;
        }
    } else {
        /* Just unload it */
        Status = LdrUnloadDll(hLibModule as *mut core::ffi::c_void);
    }

    /* Check what kind of status we got */
    if !NT_SUCCESS(Status) {
        /* Set last error */
        BaseSetLastNTError(Status);

        /* Return failure */
        return 0; // FALSE
    }

    /* Return success */
    return 1; // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn FreeLibraryAndExitThread(
    hLibModule: HMODULE,
    dwExitCode: DWORD,
) {
    if LDR_IS_DATAFILE(hLibModule) != 0 {
        /* This is a LOAD_LIBRARY_AS_DATAFILE module */
        if !RtlImageNtHeader((hLibModule as usize & !1) as *mut core::ffi::c_void).is_null() {
            /* Unmap view */
            NtUnmapViewOfSection(
                NtCurrentProcess(),
                (hLibModule as usize & !1) as *mut core::ffi::c_void,
            );

            /* Unload alternate resource module */
            LdrUnloadAlternateResourceModule(hLibModule as PVOID);
        }
    } else {
        /* Just unload it */
        LdrUnloadDll(hLibModule as *mut core::ffi::c_void);
    }

    /* Exit thread */
    ExitThread(dwExitCode);
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleFileNameA(
    hModule: HINSTANCE,
    lpFilename: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut FilenameW: UNICODE_STRING = core::mem::zeroed();
    let mut FilenameA: ANSI_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Length: DWORD = 0;
    let mut LengthToCopy: DWORD;

    /* Allocate a unicode buffer */
    FilenameW.Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (nSize as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
    ) as PWSTR;
    if FilenameW.Buffer.is_null() {
        BaseSetLastNTError(STATUS_NO_MEMORY);
        return 0;
    }

    /* Call unicode API */
    FilenameW.Length = (GetModuleFileNameW(hModule, FilenameW.Buffer, nSize) as USHORT)
        * core::mem::size_of::<WCHAR>() as USHORT;
    FilenameW.MaximumLength = FilenameW.Length + core::mem::size_of::<WCHAR>() as USHORT;

    if FilenameW.Length != 0 {
        /* Convert to ansi string */
        Status = (BasepUnicodeStringTo8BitString.unwrap())(&mut FilenameA, &FilenameW, 1);
        if !NT_SUCCESS(Status) {
            /* Set last error, free string and return failure */
            BaseSetLastNTError(Status);
            RtlFreeUnicodeString(&mut FilenameW);
            return 0;
        }

        /* Calculate size to copy */
        Length = if nSize < FilenameA.Length as DWORD {
            nSize
        } else {
            FilenameA.Length as DWORD
        };

        /* Include terminating zero */
        if nSize > Length {
            LengthToCopy = Length + 1;
        } else {
            LengthToCopy = nSize;
        }

        /* Now copy back to the caller amount he asked */
        core::ptr::copy_nonoverlapping(
            FilenameA.Buffer as *const u8,
            lpFilename as *mut u8,
            LengthToCopy as usize,
        );

        /* Free ansi filename */
        RtlFreeAnsiString(&mut FilenameA);
    }

    /* Free unicode filename */
    RtlFreeHeap(RtlGetProcessHeap(), 0, FilenameW.Buffer as PVOID);

    /* Return length copied */
    return Length;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleFileNameW(
    hModule: HINSTANCE,
    lpFilename: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    let mut ModuleListHead: PLIST_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut Module: PLDR_DATA_TABLE_ENTRY;
    let mut Length: ULONG = 0;
    let mut Cookie: ULONG_PTR = 0;
    let Peb: PPEB;

    let hModule = BasepMapModuleHandle(hModule, 0);

    /* Upscale nSize from chars to bytes */
    let mut nSize = nSize * core::mem::size_of::<WCHAR>() as DWORD;

    /* SEH try block */
    {
        /* We don't use per-thread cur dir now */
        //PRTL_PERTHREAD_CURDIR PerThreadCurdir = (PRTL_PERTHREAD_CURDIR)teb->NtTib.SubSystemTib;

        Peb = NtCurrentPeb();

        /* Acquire a loader lock */
        LdrLockLoaderLock(
            LDR_LOCK_LOADER_LOCK_FLAG_RAISE_ON_ERRORS,
            core::ptr::null_mut(),
            &mut Cookie,
        );

        /* Traverse the module list */
        ModuleListHead = &mut (*((*Peb).Ldr)).InLoadOrderModuleList;
        Entry = (*ModuleListHead).Flink;
        while Entry != ModuleListHead {
            Module = CONTAINING_RECORD!(
                Entry,
                LDR_DATA_TABLE_ENTRY,
                InLoadOrderLinks
            );

            /* Check if this is the requested module */
            if (*Module).DllBase == hModule {
                /* Calculate size to copy */
                Length = min(
                    nSize,
                    (*Module).FullDllName.MaximumLength as ULONG,
                );

                /* Copy contents */
                RtlMoveMemory(
                    lpFilename as *mut core::ffi::c_void,
                    (*Module).FullDllName.Buffer as *mut core::ffi::c_void,
                    Length as SIZE_T,
                );

                /* Subtract a terminating zero */
                if Length == (*Module).FullDllName.MaximumLength as ULONG {
                    Length -= core::mem::size_of::<WCHAR>() as ULONG;
                }

                /* Break out of the loop */
                break;
            }

            /* Advance to the next entry */
            Entry = (*Entry).Flink;
        }
    }

    /* Release the loader lock */
    LdrUnlockLoaderLock(
        LDR_UNLOCK_LOADER_LOCK_FLAG_RAISE_ON_ERRORS,
        Cookie,
    );

    return Length / core::mem::size_of::<WCHAR>() as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleHandleForUnicodeString(
    ModuleName: PUNICODE_STRING,
) -> HMODULE {
    let mut Status: NTSTATUS;
    let mut Module: PVOID = core::ptr::null_mut();
    let DllPath: LPWSTR;

    /* Try to get a handle with a magic value of 1 for DllPath */
    Status = LdrGetDllHandle(1 as LPWSTR, core::ptr::null_mut(), ModuleName, &mut Module);

    /* If that succeeded - we're done */
    if NT_SUCCESS(Status) {
        return Module as HMODULE;
    }

    /* If not, then the path should be computed */
    DllPath = BaseComputeProcessDllPath(core::ptr::null_mut(), core::ptr::null_mut());
    if DllPath.is_null() {
        Status = STATUS_NO_MEMORY;
    } else {
        // The C wraps this in _SEH2_TRY/_SEH2_EXCEPT. The translation used to run the
        // except arm unconditionally, which overwrote Status with the DUMMY_PSEH
        // `_SEH2_GetExceptionCode()` (0 = STATUS_SUCCESS), and it then dropped the C's whole
        // tail: DllPath leaked on every miss and no last error was set, though
        // BasepGetModuleHandleExW's caller relies on it ("Last error is already set") --
        // GetModuleHandleW of an unloaded module returned NULL with a stale GetLastError.
        Status = LdrGetDllHandle(DllPath, core::ptr::null_mut(), ModuleName, &mut Module);
    }

    /* Free the DllPath */
    RtlFreeHeap(RtlGetProcessHeap(), 0, DllPath as PVOID);

    /* In case of error set last win32 error and return NULL */
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        Module = core::ptr::null_mut();
    }

    /* Return module */
    Module as HMODULE
}

#[no_mangle]
pub unsafe extern "system" fn BasepGetModuleHandleExW(
    NoLock: BOOLEAN,
    dwPublicFlags: DWORD,
    lpwModuleName: LPCWSTR,
    phModule: *mut HMODULE,
) -> BOOLEAN {
    let mut Cookie: ULONG_PTR = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Status2: NTSTATUS = STATUS_SUCCESS;
    let mut hModule: HANDLE = core::ptr::null_mut();
    let mut ModuleNameU: UNICODE_STRING = core::mem::zeroed();
    let mut dwValid: DWORD = 0;
    let Redirected: BOOLEAN = 0; // FIXME

    /* Validate parameters */
    dwValid = BasepGetModuleHandleExParameterValidation(dwPublicFlags, lpwModuleName, phModule);
    // ASSERT(dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_CONTINUE);

    /* Acquire lock if necessary */
    if NoLock == 0 {
        Status = LdrLockLoaderLock(0, core::ptr::null_mut(), &mut Cookie);
        if !NT_SUCCESS(Status) {
            /* Fail */
            BaseSetLastNTError(Status);
            if !phModule.is_null() {
                *phModule = core::ptr::null_mut();
            }
            return if NT_SUCCESS(Status) { 1 } else { 0 };
        }
    }

    /* The body down to `quickie:` lives in 'quickie; `goto quickie` =>
     * `break 'quickie` (the cleanup tail follows the block). */
    'quickie: {
    if (dwPublicFlags & GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS) == 0 {
        /* Create a unicode string out of module name */
        RtlInitUnicodeString(&mut ModuleNameU, lpwModuleName);

        // FIXME: Do some redirected DLL stuff?
        if Redirected != 0 {
            // UNIMPLEMENTED;
        }

        if hModule.is_null() {
            hModule = GetModuleHandleForUnicodeString(&mut ModuleNameU) as HANDLE;
            if hModule.is_null() {
                /* Last error is already set, so just return failure by setting status */
                Status = STATUS_DLL_NOT_FOUND;
                break 'quickie;
            }
        }
    } else {
        /* Perform Pc to file header to get module instance */
        hModule = RtlPcToFileHeader(lpwModuleName as *mut core::ffi::c_void, &mut hModule as *mut HANDLE as *mut *mut core::ffi::c_void) as HANDLE;

        /* Check if it succeeded */
        if hModule.is_null() {
            /* Set "dll not found" status and quit */
            Status = STATUS_DLL_NOT_FOUND;
            break 'quickie;
        }
    }

    /* Check if changing reference is not forbidden */
    if (dwPublicFlags & GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT) == 0 {
        /* Add reference to this DLL */
        Status = LdrAddRefDll(
            if (dwPublicFlags & GET_MODULE_HANDLE_EX_FLAG_PIN) != 0 {
                LDR_ADDREF_DLL_PIN
            } else {
                0
            },
            hModule,
        );
    }
    } // 'quickie

    // quickie:
    {
        /* Set last error in case of failure */
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
        }

        /* Unlock loader lock if it was acquired */
        if NoLock == 0 {
            Status2 = LdrUnlockLoaderLock(0, Cookie);
            // ASSERT(NT_SUCCESS(Status2));
        }

        /* Set the module handle to the caller */
        if !phModule.is_null() {
            *phModule = hModule as HMODULE;
        }

        /* Return TRUE on success and FALSE otherwise */
        return if NT_SUCCESS(Status) { 1 } else { 0 };
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleHandleA(lpModuleName: LPCSTR) -> HMODULE {
    let ModuleNameW: PUNICODE_STRING;
    let pTeb: PTEB = NtCurrentTeb();

    /* Check if we have no name to convert */
    if lpModuleName.is_null() {
        return (*((*pTeb).ProcessEnvironmentBlock)).ImageBaseAddress as HMODULE;
    }

    /* Convert module name to unicode */
    ModuleNameW = Basep8BitStringToStaticUnicodeString(lpModuleName);

    /* Call W version if conversion was successful */
    if !ModuleNameW.is_null() {
        return GetModuleHandleW((*ModuleNameW).Buffer);
    }

    /* Return failure */
    return 0 as HMODULE;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE {
    let mut hModule: HMODULE = core::ptr::null_mut();
    let Success: BOOLEAN;

    /* If current module is requested - return it right away */
    if lpModuleName.is_null() {
        return (*NtCurrentPeb()).ImageBaseAddress as HMODULE;
    }

    /* Use common helper routine */
    Success = BasepGetModuleHandleExW(
        1 as BOOLEAN, // TRUE
        GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        lpModuleName,
        &mut hModule as *mut HMODULE,
    );

    /* If it wasn't successful - return NULL */
    if Success == 0 {
        hModule = core::ptr::null_mut();
    }

    /* Return the handle */
    return hModule;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleHandleExW(
    dwFlags: DWORD,
    lpwModuleName: LPCWSTR,
    phModule: *mut HMODULE,
) -> BOOL {
    let dwValid: DWORD;
    let Ret: BOOL;

    dwValid = BasepGetModuleHandleExParameterValidation(dwFlags, lpwModuleName, phModule);

    if dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR {
        return 0;
    }

    if dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_SUCCESS {
        return 1;
    }

    Ret = BasepGetModuleHandleExW(0, dwFlags, lpwModuleName, phModule) as BOOL;

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleHandleExA(
    dwFlags: DWORD,
    lpModuleName: LPCSTR,
    phModule: *mut HMODULE,
) -> BOOL {
    let lpModuleNameW: PUNICODE_STRING;
    let dwValid: DWORD;
    let Ret: BOOL;

    /* Validate parameters */
    dwValid = BasepGetModuleHandleExParameterValidation(dwFlags, lpModuleName as LPCWSTR, phModule);

    /* If result is invalid parameter - return failure */
    if dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR {
        return 0;
    }

    /* If result is 2, there is no need to do anything - return success. */
    if dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_SUCCESS {
        return 1;
    }

    /* Check if we don't need to convert the name */
    if (dwFlags & GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS) != 0 {
        /* Call the extended version of the API without conversion */
        Ret = BasepGetModuleHandleExW(
            0,
            dwFlags,
            lpModuleName as LPCWSTR,
            phModule,
        ) as BOOL;
    } else {
        /* Convert module name to unicode */
        lpModuleNameW = Basep8BitStringToStaticUnicodeString(lpModuleName);

        /* Return FALSE if conversion failed */
        if lpModuleNameW.is_null() {
            return 0;
        }

        /* Call the extended version of the API */
        Ret = BasepGetModuleHandleExW(
            0,
            dwFlags,
            (*lpModuleNameW).Buffer,
            phModule,
        ) as BOOL;
    }

    /* Return result */
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn LoadModule(
    lpModuleName: LPCSTR,
    lpParameterBlock: LPVOID,
) -> DWORD {
    let mut StartupInfo: STARTUPINFOA = core::mem::zeroed();
    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let LoadParams: *mut LOADPARMS32;
    let mut FileName: [i8; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let CommandLine: LPSTR;
    let mut Length: DWORD;
    let mut Error: DWORD;
    let mut ProcessStatus: BOOL;
    let mut AnsiStr: ANSI_STRING = core::mem::zeroed();
    let mut UnicStr: UNICODE_STRING = core::mem::zeroed();
    let mut PathType: RTL_PATH_TYPE;
    let mut Handle: HANDLE;

    LoadParams = lpParameterBlock as *mut LOADPARMS32;

    /* Check load parameters */
    if (*LoadParams).dwReserved != 0 || (*LoadParams).wMagicValue != 2 {
        /* Fail with invalid param error */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0;
    }

    /* Search path */
    Length = SearchPathA(
        core::ptr::null_mut(),
        lpModuleName,
        b".exe\0".as_ptr() as LPCSTR,
        MAX_PATH as DWORD,
        FileName.as_mut_ptr(),
        core::ptr::null_mut(),
    );

    /* Check if path was found */
    if Length != 0 && Length < MAX_PATH as DWORD {
        /* Build StartupInfo */
        RtlZeroMemory(
            &mut StartupInfo as *mut STARTUPINFOA as *mut core::ffi::c_void,
            core::mem::size_of::<STARTUPINFOA>() as SIZE_T,
        );

        StartupInfo.cb = core::mem::size_of::<STARTUPINFOA>() as DWORD;
        StartupInfo.dwFlags = STARTF_USESHOWWINDOW;
        StartupInfo.wShowWindow = (*LoadParams).wCmdShow;

        /* Allocate command line buffer */
        CommandLine = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((*(*LoadParams).lpCmdLine) as DWORD + Length + 2) as SIZE_T,
        ) as LPSTR;

        /* Put module name there, then a space, and then copy provided command line,
           and null-terminate it */
        RtlCopyMemory(
            CommandLine as *mut core::ffi::c_void,
            FileName.as_ptr() as *const core::ffi::c_void,
            Length as SIZE_T,
        );
        *CommandLine.offset(Length as isize) = ' ' as i8;
        RtlCopyMemory(
            CommandLine.offset(Length as isize + 1) as *mut core::ffi::c_void,
            (*LoadParams).lpCmdLine.offset(1) as *const core::ffi::c_void,
            (*(*LoadParams).lpCmdLine) as SIZE_T,
        );
        *CommandLine.offset(Length as isize + 1 + (*(*LoadParams).lpCmdLine) as isize) = 0;

        /* Create the process */
        ProcessStatus = CreateProcessA(
            FileName.as_ptr(),
            CommandLine,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            (*LoadParams).lpEnvAddress as LPVOID,
            core::ptr::null_mut(),
            &mut StartupInfo,
            &mut ProcessInformation,
        );

        /* Free the command line buffer */
        RtlFreeHeap(RtlGetProcessHeap(), 0, CommandLine as *mut core::ffi::c_void);

        if ProcessStatus == 0 {
            /* Creating process failed, return right error code */
            Error = GetLastError();
            match Error {
                ERROR_BAD_EXE_FORMAT => {
                    return ERROR_BAD_FORMAT;
                }
                ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => {
                    return Error;
                }
                _ => {}
            }

            /* Return 0 otherwise */
            return 0;
        }

        /* Wait up to 30 seconds for the process to become idle */
        if (*(&raw const UserWaitForInputIdleRoutine)).is_some() {
            (UserWaitForInputIdleRoutine.unwrap())(ProcessInformation.hProcess, 30000);
        }

        /* Close handles */
        NtClose(ProcessInformation.hThread);
        NtClose(ProcessInformation.hProcess);

        /* Return magic success value (33) */
        return 33;
    }

    /* The path was not found, create an ansi string from
        the module name and convert it to unicode */
    RtlInitAnsiString(&mut AnsiStr, lpModuleName);
    if !NT_SUCCESS(RtlAnsiStringToUnicodeString(&mut UnicStr, &mut AnsiStr, 1)) {
        return ERROR_FILE_NOT_FOUND;
    }

    /* Determine path type */
    PathType = RtlDetermineDosPathNameType_U(UnicStr.Buffer);

    /* Free the unicode module name */
    RtlFreeUnicodeString(&mut UnicStr);

    /* If it's a relative path, return file not found */
    if PathType == RtlPathTypeRelative {
        return ERROR_FILE_NOT_FOUND;
    }

    /* If not, try to open it */
    Handle = CreateFile(
        lpModuleName,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    if Handle != INVALID_HANDLE_VALUE {
        /* Opening file succeeded for some reason, close the handle and return file not found anyway */
        CloseHandle(Handle);
        return ERROR_FILE_NOT_FOUND;
    }

    /* Return last error which CreateFile set during an attempt to open it */
    return GetLastError();
}

#[no_mangle]
pub unsafe extern "system" fn DelayLoadFailureHook(
    pszDllName: LPCSTR,
    pszProcName: LPCSTR,
) -> FARPROC {
    STUB!();
    None
}

#[no_mangle]
pub unsafe extern "system" fn UTRegister(
    hModule: HMODULE,
    lpsz16BITDLL: LPSTR,
    lpszInitName: LPSTR,
    lpszProcName: LPSTR,
    ppfn32Thunk: *mut FARPROC,
    pfnUT32CallBack: FARPROC,
    lpBuff: LPVOID,
) -> BOOL {
    STUB!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn UTUnRegister(hModule: HMODULE) {
    STUB!();
}

#[no_mangle]
pub unsafe extern "system" fn BaseQueryModuleData(
    ModuleName: LPSTR,
    Unknown_: LPSTR,
    Unknown2: PVOID,
    Unknown3: PVOID,
    Unknown4: PVOID,
) -> BOOL {
    // DPRINT1("BaseQueryModuleData called: %s %s %p %p %p\n",
    //         ModuleName,
    //         Unknown,
    //         Unknown2,
    //         Unknown3,
    //         Unknown4);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn BaseProcessInitPostImport() -> NTSTATUS {
    // DPRINT("Post-init called\n");

    /* Check if this is a terminal server */
    if (*SharedUserData).SuiteMask & 0x00000010 != 0 {
        /* Initialize TS pointers */
        return BasepInitializeTermsrvFpns();
    }

    /* FIXME: Initialize TS pointers */
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT: See COPYING in the top level directory
//  * PROJECT  : ReactOS system libraries
//  * MODULE   : kernel32.dll
//  * FILE     : reactos/dll/win32/kernel32/misc/ldr.c
//  * AUTHOR   : Aleksey Bragin <aleksey@reactos.org>
//  */
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ****************************************************************/
//
// NTSTATUS
// WINAPI
// BasepInitializeTermsrvFpns(VOID)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// DWORD
// WINAPI
// BasepGetModuleHandleExParameterValidation(DWORD dwFlags,
//                                           LPCWSTR lpwModuleName,
//                                           HMODULE *phModule)
// {
//     /* Set phModule to 0 if it's not a NULL pointer */
//     if (phModule) *phModule = 0;
//
//     /* Check for invalid flags combination */
//     if (dwFlags & ~(GET_MODULE_HANDLE_EX_FLAG_PIN |
//                     GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT |
//                     GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS) ||
//         ((dwFlags & GET_MODULE_HANDLE_EX_FLAG_PIN) &&
//          (dwFlags & GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT)) ||
//          (!lpwModuleName && (dwFlags & GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS))
//         )
//     {
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER_1);
//         return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR;
//     }
//
//     /* Check 2nd parameter */
//     if (!phModule)
//     {
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER_2);
//         return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR;
//     }
//
//     /* Return what we have according to the module name */
//     if (lpwModuleName)
//     {
//         return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_CONTINUE;
//     }
//
//     /* No name given, so put ImageBaseAddress there */
//     *phModule = (HMODULE)NtCurrentPeb()->ImageBaseAddress;
//
//     return BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_SUCCESS;
// }
//
// PVOID
// WINAPI
// BasepMapModuleHandle(HMODULE hModule, BOOLEAN AsDataFile)
// {
//     /* If no handle is provided - use current image base address */
//     if (!hModule) return NtCurrentPeb()->ImageBaseAddress;
//
//     /* Check if it's a normal or a datafile one */
//     if (LDR_IS_DATAFILE(hModule) && !AsDataFile)
//         return NULL;
//
//     /* It's a normal DLL, just return its handle */
//     return hModule;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DisableThreadLibraryCalls(
//     IN HMODULE hLibModule)
// {
//     NTSTATUS Status;
//
//     /* Disable thread library calls */
//     Status = LdrDisableThreadCalloutsForDll((PVOID)hLibModule);
//
//     /* If it wasn't success - set last error and return failure */
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// HINSTANCE
// WINAPI
// DECLSPEC_HOTPATCH
// LoadLibraryA(LPCSTR lpLibFileName)
// {
//     static const CHAR TwainDllName[] = "twain_32.dll";
//     LPSTR PathBuffer;
//     UINT Len;
//     HINSTANCE Result;
//
//     /* Treat twain_32.dll in a special way (what a surprise...) */
//     if (lpLibFileName && !_strcmpi(lpLibFileName, TwainDllName))
//     {
//         /* Allocate space for the buffer */
//         PathBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, MAX_PATH + sizeof(ANSI_NULL));
//         if (PathBuffer)
//         {
//             /* Get windows dir in this buffer */
//             Len = GetWindowsDirectoryA(PathBuffer, MAX_PATH);
//             if ((Len != 0) && (Len < (MAX_PATH - sizeof(TwainDllName) - sizeof('\\'))))
//             {
//                 /* We successfully got windows directory. Concatenate twain_32.dll to it */
//                 PathBuffer[Len] = '\\';
//                 strcpy(&PathBuffer[Len + 1], TwainDllName);
//
//                 /* And recursively call ourselves with a new string */
//                 Result = LoadLibraryA(PathBuffer);
//
//                 /* If it was successful -  free memory and return result */
//                 if (Result)
//                 {
//                     RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//                     return Result;
//                 }
//             }
//
//             /* Free allocated buffer */
//             RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//         }
//     }
//
//     /* Call the Ex version of the API */
//     return LoadLibraryExA(lpLibFileName, NULL, 0);
// }
//
// /*
//  * @implemented
//  */
// HINSTANCE
// WINAPI
// DECLSPEC_HOTPATCH
// LoadLibraryExA(LPCSTR lpLibFileName,
//                HANDLE hFile,
//                DWORD dwFlags)
// {
//     PUNICODE_STRING FileNameW;
//
//     /* Convert file name to unicode */
//     if (!(FileNameW = Basep8BitStringToStaticUnicodeString(lpLibFileName)))
//         return NULL;
//
//     /* And call W version of the API */
//     return LoadLibraryExW(FileNameW->Buffer, hFile, dwFlags);
// }
//
// /*
//  * @implemented
//  */
// HINSTANCE
// WINAPI
// DECLSPEC_HOTPATCH
// LoadLibraryW(LPCWSTR lpLibFileName)
// {
//     /* Call Ex version of the API */
//     return LoadLibraryExW(lpLibFileName, NULL, 0);
// }
//
//
// static
// NTSTATUS
// BasepLoadLibraryAsDatafile(PWSTR Path, LPCWSTR Name, HMODULE *hModule)
// {
//     WCHAR FilenameW[MAX_PATH];
//     HANDLE hFile = INVALID_HANDLE_VALUE;
//     HANDLE hMapping;
//     NTSTATUS Status;
//     PVOID lpBaseAddress = NULL;
//     SIZE_T ViewSize = 0;
//     //PUNICODE_STRING OriginalName;
//     //UNICODE_STRING dotDLL = RTL_CONSTANT_STRING(L".DLL");
//
//     /* Zero out handle value */
//     *hModule = 0;
//
//     DPRINT("BasepLoadLibraryAsDatafile(%S %S %p)\n", Path, Name, hModule);
//
//     /*Status = RtlDosApplyFileIsolationRedirection_Ustr(TRUE,
//                                                       Name,
//                                                       &dotDLL,
//                                                       RedirName,
//                                                       RedirName2,
//                                                       &OriginalName2,
//                                                       NULL,
//                                                       NULL,
//                                                       NULL);*/
//
//     /* Try to search for it */
//     if (!SearchPathW(Path,
//                      Name,
//                      L".DLL",
//                      sizeof(FilenameW) / sizeof(FilenameW[0]),
//                      FilenameW,
//                      NULL))
//     {
//         /* Return last status value directly */
//         return NtCurrentTeb()->LastStatusValue;
//     }
//
//     /* Open this file we found */
//     hFile = CreateFileW(FilenameW,
//                         GENERIC_READ,
//                         FILE_SHARE_READ | FILE_SHARE_DELETE,
//                         NULL,
//                         OPEN_EXISTING,
//                         0,
//                         0);
//
//     /* If opening failed - return last status value */
//     if (hFile == INVALID_HANDLE_VALUE) return NtCurrentTeb()->LastStatusValue;
//
//     /* Create file mapping */
//     hMapping = CreateFileMappingW(hFile, NULL, PAGE_READONLY, 0, 0, NULL);
//
//     /* Close the file handle */
//     CloseHandle(hFile);
//
//     /* If creating file mapping failed - return last status value */
//     if (!hMapping) return NtCurrentTeb()->LastStatusValue;
//
//     /* Map view of section */
//     Status = NtMapViewOfSection(hMapping,
//                                 NtCurrentProcess(),
//                                 &lpBaseAddress,
//                                 0,
//                                 0,
//                                 0,
//                                 &ViewSize,
//                                 ViewShare,
//                                 0,
//                                 PAGE_READONLY);
//
//     /* Close handle to the section */
//     CloseHandle(hMapping);
//
//     /* If mapping view of section failed - return last status value */
//     if (!NT_SUCCESS(Status)) return NtCurrentTeb()->LastStatusValue;
//
//     /* Make sure it's a valid PE file */
//     if (!RtlImageNtHeader(lpBaseAddress))
//     {
//         /* Unmap the view and return failure status */
//         UnmapViewOfFile(lpBaseAddress);
//         return STATUS_INVALID_IMAGE_FORMAT;
//     }
//
//     /* Set low bit of handle to indicate datafile module */
//     *hModule = (HMODULE)((ULONG_PTR)lpBaseAddress | 1);
//
//     /* Load alternate resource module */
//     //LdrLoadAlternateResourceModule(*hModule, FilenameW);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// HINSTANCE
// WINAPI
// DECLSPEC_HOTPATCH
// LoadLibraryExW(LPCWSTR lpLibFileName,
//                HANDLE hFile,
//                DWORD dwFlags)
// {
//     UNICODE_STRING DllName;
//     HINSTANCE hInst;
//     NTSTATUS Status;
//     PWSTR SearchPath;
//     ULONG DllCharacteristics = 0;
//     BOOL FreeString = FALSE;
//
//     /* Check for any flags LdrLoadDll might be interested in */
//     if (dwFlags & DONT_RESOLVE_DLL_REFERENCES)
//     {
//         /* Tell LDR to treat it as an EXE */
//         DllCharacteristics = IMAGE_FILE_EXECUTABLE_IMAGE;
//     }
//
//     /* Build up a unicode dll name from null-terminated string */
//     RtlInitUnicodeString(&DllName, (LPWSTR)lpLibFileName);
//
//     /* Lazy-initialize BasepExeLdrEntry */
//     if (!BasepExeLdrEntry)
//         LdrEnumerateLoadedModules(0, BasepLocateExeLdrEntry, NtCurrentPeb()->ImageBaseAddress);
//
//     /* Check if that module is our exe*/
//     if (BasepExeLdrEntry && !(dwFlags & LOAD_LIBRARY_AS_DATAFILE) &&
//         DllName.Length == BasepExeLdrEntry->FullDllName.Length)
//     {
//         /* Lengths match and it's not a datafile, so perform name comparison */
//         if (RtlEqualUnicodeString(&DllName, &BasepExeLdrEntry->FullDllName, TRUE))
//         {
//             /* That's us! */
//             return BasepExeLdrEntry->DllBase;
//         }
//     }
//
//     /* Check for trailing spaces and remove them if necessary */
//     if (DllName.Buffer[DllName.Length/sizeof(WCHAR) - 1] == L' ')
//     {
//         RtlCreateUnicodeString(&DllName, (LPWSTR)lpLibFileName);
//         while (DllName.Length > sizeof(WCHAR) &&
//             DllName.Buffer[DllName.Length/sizeof(WCHAR) - 1] == L' ')
//         {
//             DllName.Length -= sizeof(WCHAR);
//         }
//         DllName.Buffer[DllName.Length/sizeof(WCHAR)] = UNICODE_NULL;
//         FreeString = TRUE;
//     }
//
//     /* Compute the load path */
//     SearchPath = BaseComputeProcessDllPath((dwFlags & LOAD_WITH_ALTERED_SEARCH_PATH) ?
//                                            DllName.Buffer : NULL,
//                                            NULL);
//     if (!SearchPath)
//     {
//         /* Getting DLL path failed, so set last error, free mem and return */
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//         if (FreeString) RtlFreeUnicodeString(&DllName);
//         return NULL;
//     }
//
//     _SEH2_TRY
//     {
//         if (dwFlags & LOAD_LIBRARY_AS_DATAFILE)
//         {
//             /* If the image is loaded as a datafile, try to get its handle */
//             Status = LdrGetDllHandleEx(0, SearchPath, NULL, &DllName, (PVOID*)&hInst);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* It's not loaded yet - so load it up */
//                 Status = BasepLoadLibraryAsDatafile(SearchPath, DllName.Buffer, &hInst);
//             }
//             _SEH2_YIELD(goto done;)
//         }
//
//         /* Call the API Properly */
//         Status = LdrLoadDll(SearchPath,
//                             &DllCharacteristics,
//                             &DllName,
//                             (PVOID*)&hInst);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     } _SEH2_END;
//
//
// done:
//     /* Free SearchPath buffer */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, SearchPath);
//
//     /* Free DllName string if it was dynamically allocated */
//     if (FreeString) RtlFreeUnicodeString(&DllName);
//
//     /* Set last error in failure case */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LoadLibraryExW(%ls) failing with status %lx\n", lpLibFileName, Status);
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Return loaded module handle */
//     return hInst;
// }
//
//
// /*
//  * @implemented
//  */
// FARPROC
// WINAPI
// GetProcAddress(HMODULE hModule, LPCSTR lpProcName)
// {
//     ANSI_STRING ProcedureName, *ProcNamePtr = NULL;
//     FARPROC fnExp = NULL;
//     NTSTATUS Status;
//     PVOID hMapped;
//     ULONG Ordinal = 0;
//
//     if ((ULONG_PTR)lpProcName > MAXUSHORT)
//     {
//         /* Look up by name */
//         RtlInitAnsiString(&ProcedureName, (LPSTR)lpProcName);
//         ProcNamePtr = &ProcedureName;
//     }
//     else
//     {
//         /* Look up by ordinal */
//         Ordinal = PtrToUlong(lpProcName);
//     }
//
//     /* Map provided handle */
//     hMapped = BasepMapModuleHandle(hModule, FALSE);
//
//     /* Get the proc address */
//     Status = LdrGetProcedureAddress(hMapped,
//                                     ProcNamePtr,
//                                     Ordinal,
//                                     (PVOID*)&fnExp);
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Check for a special case when returned pointer is
//        the same as image's base address */
//     if (fnExp == hMapped)
//     {
//         /* Set correct error code */
//         if (HIWORD(lpProcName) != 0)
//             BaseSetLastNTError(STATUS_ENTRYPOINT_NOT_FOUND);
//         else
//             BaseSetLastNTError(STATUS_ORDINAL_NOT_FOUND);
//
//         return NULL;
//     }
//
//     /* All good, return procedure pointer */
//     return fnExp;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// FreeLibrary(HINSTANCE hLibModule)
// {
//     NTSTATUS Status;
//     PIMAGE_NT_HEADERS NtHeaders;
//
//     if (LDR_IS_DATAFILE(hLibModule))
//     {
//         /* This is a LOAD_LIBRARY_AS_DATAFILE module, check if it's a valid one */
//         NtHeaders = RtlImageNtHeader((PVOID)((ULONG_PTR)hLibModule & ~1));
//
//         if (NtHeaders)
//         {
//             /* Unmap view */
//             Status = NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)((ULONG_PTR)hLibModule & ~1));
//
//             /* Unload alternate resource module */
//             LdrUnloadAlternateResourceModule(hLibModule as PVOID);
//         }
//         else
//             Status = STATUS_INVALID_IMAGE_FORMAT;
//     }
//     else
//     {
//         /* Just unload it */
//         Status = LdrUnloadDll((PVOID)hLibModule);
//     }
//
//     /* Check what kind of status we got */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Set last error */
//         BaseSetLastNTError(Status);
//
//         /* Return failure */
//         return FALSE;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// FreeLibraryAndExitThread(HMODULE hLibModule,
//                          DWORD dwExitCode)
// {
//
//     if (LDR_IS_DATAFILE(hLibModule))
//     {
//         /* This is a LOAD_LIBRARY_AS_DATAFILE module */
//         if (RtlImageNtHeader((PVOID)((ULONG_PTR)hLibModule & ~1)))
//         {
//             /* Unmap view */
//             NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)((ULONG_PTR)hLibModule & ~1));
//
//             /* Unload alternate resource module */
//             LdrUnloadAlternateResourceModule(hLibModule as PVOID);
//         }
//     }
//     else
//     {
//         /* Just unload it */
//         LdrUnloadDll((PVOID)hLibModule);
//     }
//
//     /* Exit thread */
//     ExitThread(dwExitCode);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetModuleFileNameA(HINSTANCE hModule,
//                    LPSTR lpFilename,
//                    DWORD nSize)
// {
//     UNICODE_STRING FilenameW;
//     ANSI_STRING FilenameA;
//     NTSTATUS Status;
//     DWORD Length = 0, LengthToCopy;
//
//     /* Allocate a unicode buffer */
//     FilenameW.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, nSize * sizeof(WCHAR));
//     if (!FilenameW.Buffer)
//     {
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//         return 0;
//     }
//
//     /* Call unicode API */
//     FilenameW.Length = (USHORT)GetModuleFileNameW(hModule, FilenameW.Buffer, nSize) * sizeof(WCHAR);
//     FilenameW.MaximumLength = FilenameW.Length + sizeof(WCHAR);
//
//     if (FilenameW.Length)
//     {
//         /* Convert to ansi string */
//         Status = BasepUnicodeStringTo8BitString(&FilenameA, &FilenameW, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Set last error, free string and return failure */
//             BaseSetLastNTError(Status);
//             RtlFreeUnicodeString(&FilenameW);
//             return 0;
//         }
//
//         /* Calculate size to copy */
//         Length = min(nSize, FilenameA.Length);
//
//         /* Include terminating zero */
//         if (nSize > Length)
//             LengthToCopy = Length + 1;
//         else
//             LengthToCopy = nSize;
//
//         /* Now copy back to the caller amount he asked */
//         RtlMoveMemory(lpFilename, FilenameA.Buffer, LengthToCopy);
//
//         /* Free ansi filename */
//         RtlFreeAnsiString(&FilenameA);
//     }
//
//     /* Free unicode filename */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, FilenameW.Buffer);
//
//     /* Return length copied */
//     return Length;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetModuleFileNameW(HINSTANCE hModule,
//                    LPWSTR lpFilename,
//                    DWORD nSize)
// {
//     PLIST_ENTRY ModuleListHead, Entry;
//     PLDR_DATA_TABLE_ENTRY Module;
//     ULONG Length = 0;
//     ULONG_PTR Cookie;
//     PPEB Peb;
//
//     hModule = BasepMapModuleHandle(hModule, FALSE);
//
//     /* Upscale nSize from chars to bytes */
//     nSize *= sizeof(WCHAR);
//
//     _SEH2_TRY
//     {
//         /* We don't use per-thread cur dir now */
//         //PRTL_PERTHREAD_CURDIR PerThreadCurdir = (PRTL_PERTHREAD_CURDIR)teb->NtTib.SubSystemTib;
//
//         Peb = NtCurrentPeb ();
//
//         /* Acquire a loader lock */
//         LdrLockLoaderLock(LDR_LOCK_LOADER_LOCK_FLAG_RAISE_ON_ERRORS, NULL, &Cookie);
//
//         /* Traverse the module list */
//         ModuleListHead = &Peb->Ldr->InLoadOrderModuleList;
//         Entry = ModuleListHead->Flink;
//         while (Entry != ModuleListHead)
//         {
//             Module = CONTAINING_RECORD(Entry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//
//             /* Check if this is the requested module */
//             if (Module->DllBase == (PVOID)hModule)
//             {
//                 /* Calculate size to copy */
//                 Length = min(nSize, Module->FullDllName.MaximumLength);
//
//                 /* Copy contents */
//                 RtlMoveMemory(lpFilename, Module->FullDllName.Buffer, Length);
//
//                 /* Subtract a terminating zero */
//                 if (Length == Module->FullDllName.MaximumLength)
//                     Length -= sizeof(WCHAR);
//
//                 /* Break out of the loop */
//                 break;
//             }
//
//             /* Advance to the next entry */
//             Entry = Entry->Flink;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         BaseSetLastNTError(_SEH2_GetExceptionCode());
//         Length = 0;
//     } _SEH2_END
//
//     /* Release the loader lock */
//     LdrUnlockLoaderLock(LDR_UNLOCK_LOADER_LOCK_FLAG_RAISE_ON_ERRORS, Cookie);
//
//     return Length / sizeof(WCHAR);
// }
//
// HMODULE
// WINAPI
// GetModuleHandleForUnicodeString(PUNICODE_STRING ModuleName)
// {
//     NTSTATUS Status;
//     PVOID Module;
//     LPWSTR DllPath;
//
//     /* Try to get a handle with a magic value of 1 for DllPath */
//     Status = LdrGetDllHandle((LPWSTR)1, NULL, ModuleName, &Module);
//
//     /* If that succeeded - we're done */
//     if (NT_SUCCESS(Status)) return Module;
//
//     /* If not, then the path should be computed */
//     DllPath = BaseComputeProcessDllPath(NULL, NULL);
//     if (!DllPath)
//     {
//         Status = STATUS_NO_MEMORY;
//     }
//     else
//     {
//         _SEH2_TRY
//         {
//             Status = LdrGetDllHandle(DllPath, NULL, ModuleName, &Module);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Fail with the SEH error */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Free the DllPath */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, DllPath);
//
//     /* In case of error set last win32 error and return NULL */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failure acquiring DLL module '%wZ' handle, Status 0x%08X\n", ModuleName, Status);
//         BaseSetLastNTError(Status);
//         Module = 0;
//     }
//
//     /* Return module */
//     return (HMODULE)Module;
// }
//
// BOOLEAN
// WINAPI
// BasepGetModuleHandleExW(BOOLEAN NoLock, DWORD dwPublicFlags, LPCWSTR lpwModuleName, HMODULE *phModule)
// {
//     ULONG_PTR Cookie;
//     NTSTATUS Status = STATUS_SUCCESS, Status2;
//     HANDLE hModule = NULL;
//     UNICODE_STRING ModuleNameU;
//     DWORD dwValid;
//     BOOLEAN Redirected = FALSE; // FIXME
//
//     /* Validate parameters */
//     dwValid = BasepGetModuleHandleExParameterValidation(dwPublicFlags, lpwModuleName, phModule);
//     ASSERT(dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_CONTINUE);
//
//     /* Acquire lock if necessary */
//     if (!NoLock)
//     {
//         Status = LdrLockLoaderLock(0, NULL, &Cookie);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             BaseSetLastNTError(Status);
//             if (phModule) *phModule = NULL;
//             return NT_SUCCESS(Status);
//         }
//     }
//
//     if (!(dwPublicFlags & GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS))
//     {
//         /* Create a unicode string out of module name */
//         RtlInitUnicodeString(&ModuleNameU, lpwModuleName);
//
//         // FIXME: Do some redirected DLL stuff?
//         if (Redirected)
//         {
//             UNIMPLEMENTED;
//         }
//
//         if (!hModule)
//         {
//             hModule = GetModuleHandleForUnicodeString(&ModuleNameU);
//             if (!hModule)
//             {
//                 /* Last error is already set, so just return failure by setting status */
//                 Status = STATUS_DLL_NOT_FOUND;
//                 goto quickie;
//             }
//         }
//     }
//     else
//     {
//         /* Perform Pc to file header to get module instance */
//         hModule = (HMODULE)RtlPcToFileHeader((PVOID)lpwModuleName,
//                                              (PVOID*)&hModule);
//
//         /* Check if it succeeded */
//         if (!hModule)
//         {
//             /* Set "dll not found" status and quit */
//             Status = STATUS_DLL_NOT_FOUND;
//             goto quickie;
//         }
//     }
//
//     /* Check if changing reference is not forbidden */
//     if (!(dwPublicFlags & GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT))
//     {
//         /* Add reference to this DLL */
//         Status = LdrAddRefDll((dwPublicFlags & GET_MODULE_HANDLE_EX_FLAG_PIN) ? LDR_ADDREF_DLL_PIN : 0,
//                               hModule);
//     }
//
// quickie:
//     /* Set last error in case of failure */
//     if (!NT_SUCCESS(Status))
//         BaseSetLastNTError(Status);
//
//     /* Unlock loader lock if it was acquired */
//     if (!NoLock)
//     {
//         Status2 = LdrUnlockLoaderLock(0, Cookie);
//         ASSERT(NT_SUCCESS(Status2));
//     }
//
//     /* Set the module handle to the caller */
//     if (phModule) *phModule = hModule;
//
//     /* Return TRUE on success and FALSE otherwise */
//     return NT_SUCCESS(Status);
// }
//
// /*
//  * @implemented
//  */
// HMODULE
// WINAPI
// DECLSPEC_HOTPATCH
// GetModuleHandleA(LPCSTR lpModuleName)
// {
//     PUNICODE_STRING ModuleNameW;
//     PTEB pTeb = NtCurrentTeb();
//
//     /* Check if we have no name to convert */
//     if (!lpModuleName)
//         return ((HMODULE)pTeb->ProcessEnvironmentBlock->ImageBaseAddress);
//
//     /* Convert module name to unicode */
//     ModuleNameW = Basep8BitStringToStaticUnicodeString(lpModuleName);
//
//     /* Call W version if conversion was successful */
//     if (ModuleNameW)
//         return GetModuleHandleW(ModuleNameW->Buffer);
//
//     /* Return failure */
//     return 0;
// }
//
//
// /*
//  * @implemented
//  */
// HMODULE
// WINAPI
// GetModuleHandleW(LPCWSTR lpModuleName)
// {
//     HMODULE hModule;
//     BOOLEAN Success;
//
//     /* If current module is requested - return it right away */
//     if (!lpModuleName)
//         return ((HMODULE)NtCurrentPeb()->ImageBaseAddress);
//
//     /* Use common helper routine */
//     Success = BasepGetModuleHandleExW(TRUE,
//                                       GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
//                                       lpModuleName,
//                                       &hModule);
//
//     /* If it wasn't successful - return NULL */
//     if (!Success) hModule = NULL;
//
//     /* Return the handle */
//     return hModule;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetModuleHandleExW(IN DWORD dwFlags,
//                    IN LPCWSTR lpwModuleName  OPTIONAL,
//                    OUT HMODULE* phModule)
// {
//     DWORD dwValid;
//     BOOL Ret;
//
//     /* Validate parameters */
//     dwValid = BasepGetModuleHandleExParameterValidation(dwFlags, lpwModuleName, phModule);
//
//     /* If result is invalid parameter - return failure */
//     if (dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR) return FALSE;
//
//     /* If result is 2, there is no need to do anything - return success. */
//     if (dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_SUCCESS) return TRUE;
//
//     /* Use common helper routine */
//     Ret = BasepGetModuleHandleExW(FALSE,
//                                   dwFlags,
//                                   lpwModuleName,
//                                   phModule);
//
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetModuleHandleExA(IN DWORD dwFlags,
//                    IN LPCSTR lpModuleName OPTIONAL,
//                    OUT HMODULE* phModule)
// {
//     PUNICODE_STRING lpModuleNameW;
//     DWORD dwValid;
//     BOOL Ret;
//
//     /* Validate parameters */
//     dwValid = BasepGetModuleHandleExParameterValidation(dwFlags, (LPCWSTR)lpModuleName, phModule);
//
//     /* If result is invalid parameter - return failure */
//     if (dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_ERROR) return FALSE;
//
//     /* If result is 2, there is no need to do anything - return success. */
//     if (dwValid == BASEP_GET_MODULE_HANDLE_EX_PARAMETER_VALIDATION_SUCCESS) return TRUE;
//
//     /* Check if we don't need to convert the name */
//     if (dwFlags & GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS)
//     {
//         /* Call the extended version of the API without conversion */
//         Ret = BasepGetModuleHandleExW(FALSE,
//                                       dwFlags,
//                                       (LPCWSTR)lpModuleName,
//                                       phModule);
//     }
//     else
//     {
//         /* Convert module name to unicode */
//         lpModuleNameW = Basep8BitStringToStaticUnicodeString(lpModuleName);
//
//         /* Return FALSE if conversion failed */
//         if (!lpModuleNameW) return FALSE;
//
//         /* Call the extended version of the API */
//         Ret = BasepGetModuleHandleExW(FALSE,
//                                       dwFlags,
//                                       lpModuleNameW->Buffer,
//                                       phModule);
//     }
//
//     /* Return result */
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// LoadModule(LPCSTR lpModuleName,
//            LPVOID lpParameterBlock)
// {
//     STARTUPINFOA StartupInfo;
//     PROCESS_INFORMATION ProcessInformation;
//     LOADPARMS32 *LoadParams;
//     char FileName[MAX_PATH];
//     LPSTR CommandLine;
//     DWORD Length, Error;
//     BOOL ProcessStatus;
//     ANSI_STRING AnsiStr;
//     UNICODE_STRING UnicStr;
//     RTL_PATH_TYPE PathType;
//     HANDLE Handle;
//
//     LoadParams = (LOADPARMS32*)lpParameterBlock;
//
//     /* Check load parameters */
//     if (LoadParams->dwReserved || LoadParams->wMagicValue != 2)
//     {
//         /* Fail with invalid param error */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return 0;
//     }
//
//     /* Search path */
//     Length = SearchPathA(NULL, lpModuleName, ".exe", MAX_PATH, FileName, NULL);
//
//     /* Check if path was found */
//     if (Length && Length < MAX_PATH)
//     {
//         /* Build StartupInfo */
//         RtlZeroMemory(&StartupInfo, sizeof(StartupInfo));
//
//         StartupInfo.cb = sizeof(STARTUPINFOA);
//         StartupInfo.dwFlags = STARTF_USESHOWWINDOW;
//         StartupInfo.wShowWindow = LoadParams->wCmdShow;
//
//         /* Allocate command line buffer */
//         CommandLine = RtlAllocateHeap(RtlGetProcessHeap(),
//                                       HEAP_ZERO_MEMORY,
//                                       (ULONG)LoadParams->lpCmdLine[0] + Length + 2);
//
//         /* Put module name there, then a space, and then copy provided command line,
//            and null-terminate it */
//         RtlCopyMemory(CommandLine, FileName, Length);
//         CommandLine[Length] = ' ';
//         RtlCopyMemory(&CommandLine[Length + 1], &LoadParams->lpCmdLine[1], (ULONG)LoadParams->lpCmdLine[0]);
//         CommandLine[Length + 1 + (ULONG)LoadParams->lpCmdLine[0]] = 0;
//
//         /* Create the process */
//         ProcessStatus = CreateProcessA(FileName,
//                                        CommandLine,
//                                        NULL,
//                                        NULL,
//                                        FALSE,
//                                        0,
//                                        LoadParams->lpEnvAddress,
//                                        NULL,
//                                        &StartupInfo,
//                                        &ProcessInformation);
//
//         /* Free the command line buffer */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, CommandLine);
//
//         if (!ProcessStatus)
//         {
//             /* Creating process failed, return right error code */
//             Error = GetLastError();
//             switch(Error)
//             {
//             case ERROR_BAD_EXE_FORMAT:
//                return ERROR_BAD_FORMAT;
//
//             case ERROR_FILE_NOT_FOUND:
//             case ERROR_PATH_NOT_FOUND:
//                 return Error;
//             }
//
//             /* Return 0 otherwise */
//             return 0;
//         }
//
//         /* Wait up to 30 seconds for the process to become idle */
//         if (UserWaitForInputIdleRoutine)
//         {
//             UserWaitForInputIdleRoutine(ProcessInformation.hProcess, 30000);
//         }
//
//         /* Close handles */
//         NtClose(ProcessInformation.hThread);
//         NtClose(ProcessInformation.hProcess);
//
//         /* Return magic success value (33) */
//         return 33;
//     }
//
//     /* The path was not found, create an ansi string from
//         the module name and convert it to unicode */
//     RtlInitAnsiString(&AnsiStr, lpModuleName);
//     if (!NT_SUCCESS(RtlAnsiStringToUnicodeString(&UnicStr,&AnsiStr,TRUE)))
//         return ERROR_FILE_NOT_FOUND;
//
//     /* Determine path type */
//     PathType = RtlDetermineDosPathNameType_U(UnicStr.Buffer);
//
//     /* Free the unicode module name */
//     RtlFreeUnicodeString(&UnicStr);
//
//     /* If it's a relative path, return file not found */
//     if (PathType == RtlPathTypeRelative)
//         return ERROR_FILE_NOT_FOUND;
//
//     /* If not, try to open it */
//     Handle = CreateFile(lpModuleName,
//                         GENERIC_READ,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         NULL,
//                         OPEN_EXISTING,
//                         FILE_ATTRIBUTE_NORMAL,
//                         NULL);
//
//     if (Handle != INVALID_HANDLE_VALUE)
//     {
//         /* Opening file succeeded for some reason, close the handle and return file not found anyway */
//         CloseHandle(Handle);
//         return ERROR_FILE_NOT_FOUND;
//     }
//
//     /* Return last error which CreateFile set during an attempt to open it */
//     return GetLastError();
// }
//
// /*
//  * @unimplemented
//  */
// FARPROC WINAPI DelayLoadFailureHook(LPCSTR pszDllName, LPCSTR pszProcName)
// {
//     STUB;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI UTRegister( HMODULE hModule, LPSTR lpsz16BITDLL,
//                         LPSTR lpszInitName, LPSTR lpszProcName,
//                         FARPROC *ppfn32Thunk, FARPROC pfnUT32CallBack,
//                         LPVOID lpBuff )
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// VOID WINAPI UTUnRegister( HMODULE hModule )
// {
//     STUB;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// BaseQueryModuleData(IN LPSTR ModuleName,
//                     IN LPSTR Unknown,
//                     IN PVOID Unknown2,
//                     IN PVOID Unknown3,
//                     IN PVOID Unknown4)
// {
//     DPRINT1("BaseQueryModuleData called: %s %s %p %p %p\n",
//             ModuleName,
//             Unknown,
//             Unknown2,
//             Unknown3,
//             Unknown4);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// BaseProcessInitPostImport(VOID)
// {
//     DPRINT("Post-init called\n");
//
//     /* Check if this is a terminal server */
//     if (SharedUserData->SuiteMask & VER_SUITE_TERMINAL)
//     {
//         /* Initialize TS pointers */
//         return BasepInitializeTermsrvFpns();
//     }
//
//     /* FIXME: Initialize TS pointers */
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
