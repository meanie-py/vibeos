use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn ReadProcessMemory( hProcess: HANDLE, lpBaseAddress: LPCVOID, lpBuffer: LPVOID, nSize: SIZE_T, lpNumberOfBytesRead: *mut SIZE_T, ) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// #define WIN32_NO_STATUS (flag, no value)
// #define NTOS_MODE_USER (flag, no value)
// #define NDEBUG (flag, no value)

pub const MAX_MODULES: usize = 0x2710; // Matches 10.000 modules

pub const INIT_MEMORY_SIZE: usize = 0x1000; // Matches 4kB

#[repr(C)]
pub struct _INTERNAL_ENUM_PAGE_FILES_CONTEXT {
    pub lpContext: LPVOID,
    pub pCallbackRoutine: PENUM_PAGE_FILE_CALLBACKA,
    pub dwErrCode: DWORD,
}
pub type INTERNAL_ENUM_PAGE_FILES_CONTEXT = _INTERNAL_ENUM_PAGE_FILES_CONTEXT;
pub type PINTERNAL_ENUM_PAGE_FILES_CONTEXT = *mut _INTERNAL_ENUM_PAGE_FILES_CONTEXT;

unsafe extern "system" fn FindDeviceDriver(
    ImageBase: PVOID,
    MatchingModule: PRTL_PROCESS_MODULE_INFORMATION,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut i: DWORD;
    let mut RequiredSize: DWORD = 0;
    let mut Information: PRTL_PROCESS_MODULES = core::ptr::null_mut();
    let mut Module: RTL_PROCESS_MODULE_INFORMATION = core::mem::zeroed();
    /* By default, to prevent too many reallocations, we already make room for 4 modules */
    let mut Size: DWORD = (core::mem::size_of::<RTL_PROCESS_MODULES>() as DWORD).wrapping_add(
        (3 as DWORD).wrapping_mul(core::mem::size_of::<RTL_PROCESS_MODULE_INFORMATION>() as DWORD),
    );

    loop {
        /* Allocate a buffer to hold modules information */
        Information = LocalAlloc(LMEM_FIXED as UINT, Size as SIZE_T) as PRTL_PROCESS_MODULES;
        if Information.is_null() {
            SetLastError(ERROR_NO_SYSTEM_RESOURCES as DWORD);
            return (FALSE) as BOOL;
        }

        /* Query information */
        Status = NtQuerySystemInformation(
            SystemModuleInformation,
            Information as PVOID,
            Size as ULONG,
            core::ptr::addr_of_mut!(RequiredSize),
        );
        if !NT_SUCCESS!(Status) {
            /* Free the current buffer */
            LocalFree(Information as HLOCAL);

            /* If it was not a length mismatch (ie, buffer too small), just leave */
            if Status != STATUS_INFO_LENGTH_MISMATCH {
                SetLastError(RtlNtStatusToDosError(Status) as DWORD);
                return (FALSE) as BOOL;
            }

            /* Try again with the required size */
            Size = RequiredSize;
            continue;
        }

        /* No modules returned? Leave */
        if (*Information).NumberOfModules == 0 {
            break;
        }

        /* Try to find which module matches the base address given */
        i = 0;
        while i < (*Information).NumberOfModules {
            Module = *(core::ptr::addr_of_mut!((*Information).Modules) as *mut RTL_PROCESS_MODULE_INFORMATION)
                .offset(i as isize);
            if Module.ImageBase == ImageBase {
                /* Copy the matching module and leave */
                memcpy(
                    MatchingModule as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(Module) as *const core::ffi::c_void,
                    core::mem::size_of::<RTL_PROCESS_MODULE_INFORMATION>() as size_t,
                );
                LocalFree(Information as HLOCAL);
                return (TRUE) as BOOL;
            }
            i = i.wrapping_add(1);
        }

        /* If we arrive here, it means we were not able to find matching base address */
        break;
    }

    /* Release and leave */
    LocalFree(Information as HLOCAL);
    SetLastError(ERROR_INVALID_HANDLE);

    return (FALSE) as BOOL;
}
unsafe extern "system" fn FindModule(
    mut hProcess: HANDLE,
    mut hModule: HMODULE,
    mut Module: PLDR_DATA_TABLE_ENTRY,
) -> BOOL {
    let mut Count: DWORD;
    let mut Status: NTSTATUS;
    let mut LoaderData: PPEB_LDR_DATA = core::ptr::null_mut();
    let mut ListHead: PLIST_ENTRY = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ProcInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();

    /* Query the process information to get its PEB address */
    Status = NtQueryInformationProcess(
        hProcess,
        ProcessBasicInformation,
        &raw mut ProcInfo as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    /* If no module was provided, get base as module */
    if hModule.is_null() {
        if ReadProcessMemory(
            hProcess,
            &raw mut (*ProcInfo.PebBaseAddress).ImageBaseAddress as LPCVOID,
            &raw mut hModule as LPVOID,
            core::mem::size_of::<HMODULE>() as SIZE_T,
            core::ptr::null_mut::<ULONG_PTR>(),
        ) == 0
        {
            return (FALSE) as BOOL;
        }
    }

    /* Read loader data address from PEB */
    if ReadProcessMemory(
        hProcess,
        &raw mut (*ProcInfo.PebBaseAddress).Ldr as LPCVOID,
        &raw mut LoaderData as LPVOID,
        // The PEB's Ldr POINTER, as the C's sizeof(LoaderData): sizing it as the
        // struct it points at overran this stack slot by the whole PEB_LDR_DATA.
        core::mem::size_of::<PPEB_LDR_DATA>() as SIZE_T,
        core::ptr::null_mut::<ULONG_PTR>(),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if LoaderData.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    /* Store list head address */
    ListHead = &raw mut (*LoaderData).InMemoryOrderModuleList as PLIST_ENTRY;

    /* Read first element in the modules list */
    if ReadProcessMemory(
        hProcess,
        &raw mut (*LoaderData).InMemoryOrderModuleList.Flink as LPCVOID,
        &raw mut ListEntry as LPVOID,
        // One Flink pointer (C: sizeof(ListEntry)), not a whole LIST_ENTRY.
        core::mem::size_of::<PLIST_ENTRY>() as SIZE_T,
        core::ptr::null_mut::<ULONG_PTR>(),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    Count = 0;

    /* Loop on the modules */
    while ListEntry != ListHead {
        /* Load module data */
        if ReadProcessMemory(
            hProcess,
            CONTAINING_RECORD!(ListEntry, LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks) as LPCVOID,
            Module as LPVOID,
            core::mem::size_of::<LDR_DATA_TABLE_ENTRY>() as SIZE_T,
            core::ptr::null_mut::<ULONG_PTR>(),
        ) == 0
        {
            return (FALSE) as BOOL;
        }

        /* Does that match the module we're looking for? */
        if (*Module).DllBase == hModule as PVOID {
            return TRUE as BOOL;
        }

        Count = Count.wrapping_add(1);
        if Count > MAX_MODULES as DWORD {
            break;
        }

        /* Get to next listed module */
        ListEntry = (*Module).InMemoryOrderLinks.Flink as PLIST_ENTRY;
    }

    SetLastError(ERROR_INVALID_HANDLE as DWORD);
    return (FALSE) as BOOL;
}

unsafe extern "system" fn CallBackConvertToAscii(
    pContext: LPVOID,
    pPageFileInfo: PENUM_PAGE_FILE_INFORMATION,
    lpFilename: LPCWSTR,
) -> BOOL {
    let mut Ret: BOOL;
    let mut Len: SIZE_T;
    let mut AnsiFileName: LPSTR;
    let Context: PINTERNAL_ENUM_PAGE_FILES_CONTEXT = pContext as PINTERNAL_ENUM_PAGE_FILES_CONTEXT;

    Len = wcslen(lpFilename as *const wchar_t) as SIZE_T;

    /* Alloc space for the ANSI string */
    AnsiFileName = LocalAlloc(
        LMEM_FIXED as UINT,
        Len.wrapping_mul(core::mem::size_of::<CHAR>() as SIZE_T)
            .wrapping_add(core::mem::size_of::<CHAR>() as SIZE_T),
    ) as LPSTR;
    if AnsiFileName.is_null() {
        (*Context).dwErrCode = RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES) as DWORD;
        return (FALSE) as BOOL;
    }

    /* Convert string to ANSI */
    if WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        lpFilename,
        -(1 as ::core::ffi::c_int),
        AnsiFileName,
        ((Len as usize).wrapping_mul(core::mem::size_of::<CHAR>() as usize)
            .wrapping_add(core::mem::size_of::<CHAR>() as usize))
            as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) == 0 as ::core::ffi::c_int
    {
        (*Context).dwErrCode = GetLastError() as DWORD;
        LocalFree(AnsiFileName as HLOCAL);
        return (FALSE) as BOOL;
    }

    /* And finally call "real" callback */
    Ret = (*Context).pCallbackRoutine.unwrap_unchecked()(
        (*Context).lpContext,
        pPageFileInfo,
        AnsiFileName as LPCSTR,
    );
    LocalFree(AnsiFileName as HLOCAL);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn EmptyWorkingSet(hProcess: HANDLE) -> BOOL {
    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
    let mut QuotaLimits: QUOTA_LIMITS = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));

    /* Query the working set */
    Status = NtQueryInformationProcess(
        hProcess,
        ProcessQuotaLimits,
        core::ptr::addr_of_mut!(QuotaLimits) as PVOID,
        core::mem::size_of::<QUOTA_LIMITS>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );

    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    /* Empty the working set */
    QuotaLimits.MinimumWorkingSetSize = (0 as SIZE_T).wrapping_sub(1);
    QuotaLimits.MaximumWorkingSetSize = (0 as SIZE_T).wrapping_sub(1);

    /* Set the working set */
    Status = NtSetInformationProcess(
        hProcess,
        ProcessQuotaLimits,
        core::ptr::addr_of_mut!(QuotaLimits) as PVOID,
        core::mem::size_of::<QUOTA_LIMITS>() as ULONG,
    );
    if !NT_SUCCESS!(Status) && Status != STATUS_PRIVILEGE_NOT_HELD {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumDeviceDrivers(
    lpImageBase: *mut LPVOID,
    cb: DWORD,
    lpcbNeeded: LPDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut NewSize: DWORD;
    let mut Count: DWORD = 0; // written before it is read; a closure borrows it below
    let mut Information: PRTL_PROCESS_MODULES = core::ptr::null_mut();
    /* By default, to prevent too many reallocations, we already make room for 4 modules */
    let mut Size: DWORD = (core::mem::size_of::<RTL_PROCESS_MODULES>()
        + (3 as usize) * core::mem::size_of::<RTL_PROCESS_MODULE_INFORMATION>()) as DWORD;

    'alloc: loop {
        'body: {
            /* Allocate a buffer to hold modules information */
            Information = LocalAlloc(LMEM_FIXED, Size as SIZE_T) as PRTL_PROCESS_MODULES;
            if Information.is_null() {
                SetLastError(ERROR_NO_SYSTEM_RESOURCES);
                return (FALSE) as BOOL;
            }

            /* Query information */
            Status = NtQuerySystemInformation(
                SystemModuleInformation,
                Information as PVOID,
                Size as ULONG,
                core::ptr::addr_of_mut!(Count),
            );
            /* In case of an error */
            if !NT_SUCCESS!(Status) {
                /* Save the amount of output modules */
                NewSize = (*Information).NumberOfModules as DWORD;
                /* And free buffer */
                LocalFree(Information as HLOCAL);

                /* If it was not a length mismatch (ie, buffer too small), just leave */
                if Status != STATUS_INFO_LENGTH_MISMATCH {
                    SetLastError(RtlNtStatusToDosError(Status) as DWORD);
                    return (FALSE) as BOOL;
                }

                /* Compute new size length */
                ASSERT!(Size as usize >= core::mem::size_of::<RTL_PROCESS_MODULES>());
                NewSize = (NewSize as usize).wrapping_mul(core::mem::size_of::<RTL_PROCESS_MODULE_INFORMATION>()) as DWORD;
                NewSize = (NewSize as usize).wrapping_add(core::mem::size_of::<ULONG>()) as DWORD;
                ASSERT!(NewSize as usize >= core::mem::size_of::<RTL_PROCESS_MODULES>());
                /* Check whether it is really bigger - otherwise, leave */
                if NewSize < Size {
                    ASSERT!(NewSize > Size);
                    SetLastError(RtlNtStatusToDosError(STATUS_INFO_LENGTH_MISMATCH) as DWORD);
                    return (FALSE) as BOOL;
                }

                /* Loop again with that new buffer */
                Size = NewSize;
                break 'body;
            }

            /* End of allocation loop */
            break 'alloc;
        }
        if !(TRUE != 0) {
            break 'alloc;
        }
    }

    // _SEH2_TRY { body } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { .. } _SEH2_END
    let seh = ntdll_sys::seh::try_except(
        &mut (),
        |_| {
            Count = 0;
            while Count < (*Information).NumberOfModules
                && (Count as usize) < (cb as usize) / core::mem::size_of::<LPVOID>()
            {
                *lpImageBase.offset(Count as isize) = (*core::ptr::addr_of!((*Information).Modules[0]).add(Count as usize)).ImageBase as LPVOID;
                Count = Count.wrapping_add(1);
            }

            *lpcbNeeded = ((*Information).NumberOfModules as usize)
                .wrapping_mul(core::mem::size_of::<LPVOID>()) as DWORD;
        },
        |_, _| ntdll_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    );
    if let Err(code) = seh {
        SetLastError(RtlNtStatusToDosError(code) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EnumProcesses(
    lpidProcess: *mut DWORD,
    cb: DWORD,
    lpcbNeeded: LPDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Size: DWORD = MAXSHORT;
    let mut Count: DWORD;
    let mut ProcInfo: PSYSTEM_PROCESS_INFORMATION;
    let mut ProcInfoArray: PSYSTEM_PROCESS_INFORMATION;

    /* First of all, query all the processes */
    'query: loop {
        'body: {
            ProcInfoArray = LocalAlloc(LMEM_FIXED as UINT, Size as SIZE_T) as PSYSTEM_PROCESS_INFORMATION;
            if ProcInfoArray.is_null() {
                return (FALSE) as BOOL;
            }

            Status = NtQuerySystemInformation(
                SystemProcessInformation,
                ProcInfoArray as PVOID,
                Size as ULONG,
                core::ptr::null_mut::<ULONG>(),
            );
            if Status == STATUS_INFO_LENGTH_MISMATCH {
                LocalFree(ProcInfoArray as HLOCAL);
                Size += MAXSHORT;
                break 'body;
            }

            // break out of the do-while entirely
            break 'query;
        }
        if (((TRUE)) == 0) {
            break;
        }
    }

    if !NT_SUCCESS!(Status) {
        LocalFree(ProcInfoArray as HLOCAL);
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    /* Then, loop to output data */
    Count = 0;
    ProcInfo = ProcInfoArray;

    // _SEH2_TRY { body } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { .. } _SEH2_END
    let seh = ntdll_sys::seh::try_except(
        &mut (),
        |_| {
        loop {
            /* It may sound weird, but actually MS only updated Count on
             * successful write. So, it cannot measure the amount of space needed!
             * This is really tricky.
             */
            if Count < cb / (core::mem::size_of::<DWORD>() as DWORD) {
                *lpidProcess.offset(Count as isize) = HandleToUlong!((*ProcInfo).UniqueProcessId) as DWORD;
                Count = Count.wrapping_add(1);
            }

            if (*ProcInfo).NextEntryOffset == 0 {
                break;
            }

            ProcInfo = (ProcInfo as ULONG_PTR).wrapping_add((*ProcInfo).NextEntryOffset as ULONG_PTR) as PSYSTEM_PROCESS_INFORMATION;

            if (((TRUE)) == 0) {
                break;
            }
        }

        *lpcbNeeded = Count.wrapping_mul(core::mem::size_of::<DWORD>() as DWORD);
        },
        |_, _| ntdll_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    );
    if let Err(code) = seh {
        SetLastError(RtlNtStatusToDosError(code) as DWORD);
        LocalFree(ProcInfoArray as HLOCAL);
        return (FALSE) as BOOL;
    }

    LocalFree(ProcInfoArray as HLOCAL);
    return (TRUE) as BOOL;
}

// LIST_MODULES_* (psapi.h).
const LIST_MODULES_32BIT: DWORD = 0x01;
const LIST_MODULES_64BIT: DWORD = 0x02;
const LIST_MODULES_ALL: DWORD = 0x03;

/// Vista. The filter picks 32- and/or 64-bit modules; LIST_MODULES_DEFAULT (0) is the
/// caller's own bitness. Every module a native process loads has the bitness of this
/// build — enumerating a WoW64 process's 32-bit list from 64-bit code is not supported
/// here — so asking only for the other bitness lists nothing, and anything else is
/// EnumProcessModules.
#[no_mangle]
pub unsafe extern "system" fn EnumProcessModulesEx(
    hProcess: HANDLE,
    lphModule: *mut HMODULE,
    cb: DWORD,
    lpcbNeeded: LPDWORD,
    dwFilterFlag: DWORD,
) -> BOOL {
    if dwFilterFlag & !LIST_MODULES_ALL != 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }
    let other = if core::mem::size_of::<usize>() == 8 { LIST_MODULES_32BIT } else { LIST_MODULES_64BIT };
    if dwFilterFlag == other {
        *lpcbNeeded = 0;
        return TRUE as BOOL;
    }
    EnumProcessModules(hProcess, lphModule, cb, lpcbNeeded)
}

#[no_mangle]
pub unsafe extern "system" fn EnumProcessModules(
    mut hProcess: HANDLE,
    mut lphModule: *mut HMODULE,
    mut cb: DWORD,
    mut lpcbNeeded: LPDWORD,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut NbOfModules: DWORD = 0;
    let mut Count: DWORD = 0;
    let mut LoaderData: PPEB_LDR_DATA = core::ptr::null_mut();
    let mut ListHead: PLIST_ENTRY = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ProcInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut CurrentModule: LDR_DATA_TABLE_ENTRY = core::mem::zeroed();

    /* Query the process information to get its PEB address */
    Status = NtQueryInformationProcess(
        hProcess,
        ProcessBasicInformation,
        &raw mut ProcInfo as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    if ProcInfo.PebBaseAddress.is_null() {
        SetLastError(RtlNtStatusToDosError(STATUS_PARTIAL_COPY) as DWORD);
        return (FALSE) as BOOL;
    }

    /* Read loader data address from PEB */
    if ReadProcessMemory(
        hProcess,
        &raw mut (*ProcInfo.PebBaseAddress).Ldr as LPCVOID,
        &raw mut LoaderData as LPVOID,
        // The PEB's Ldr POINTER, as the C's sizeof(LoaderData): sizing it as the
        // struct it points at overran this stack slot by the whole PEB_LDR_DATA.
        core::mem::size_of::<PPEB_LDR_DATA>() as SIZE_T,
        core::ptr::null_mut::<ULONG_PTR>(),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Store list head address */
    ListHead = &raw mut (*LoaderData).InLoadOrderModuleList as PLIST_ENTRY;

    /* Read first element in the modules list */
    if ReadProcessMemory(
        hProcess,
        &raw mut (*LoaderData).InLoadOrderModuleList.Flink as LPCVOID,
        &raw mut ListEntry as LPVOID,
        // One Flink pointer (C: sizeof(ListEntry)), not a whole LIST_ENTRY.
        core::mem::size_of::<PLIST_ENTRY>() as SIZE_T,
        core::ptr::null_mut::<ULONG_PTR>(),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    NbOfModules = cb / (core::mem::size_of::<HMODULE>() as DWORD);
    Count = 0;

    /* Loop on the modules */
    while ListEntry != ListHead {
        /* Load module data */
        if ReadProcessMemory(
            hProcess,
            CONTAINING_RECORD!(ListEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as LPCVOID,
            &raw mut CurrentModule as LPVOID,
            core::mem::size_of::<LDR_DATA_TABLE_ENTRY>() as SIZE_T,
            core::ptr::null_mut::<ULONG_PTR>(),
        ) == 0
        {
            return (FALSE) as BOOL;
        }

        /* Check if we can output module, do it if so */
        if Count < NbOfModules {
        // _SEH2_TRY { body } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { .. } _SEH2_END
            let seh = ntdll_sys::seh::try_except(
                &mut (),
                |_| *lphModule.offset(Count as isize) = CurrentModule.DllBase as HMODULE,
                |_, _| ntdll_sys::seh::EXCEPTION_EXECUTE_HANDLER,
            );
            if let Err(code) = seh {
                SetLastError(RtlNtStatusToDosError(code) as DWORD);
                return (FALSE) as BOOL;
            }
        }

        Count = Count.wrapping_add(1);
        if Count > MAX_MODULES as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
            return (FALSE) as BOOL;
        }

        /* Get to next listed module */
        ListEntry = CurrentModule.InLoadOrderLinks.Flink as PLIST_ENTRY;
    }

    // _SEH2_TRY { body } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { .. } _SEH2_END
    let seh = ntdll_sys::seh::try_except(
        &mut (),
        |_| *lpcbNeeded = Count.wrapping_mul(core::mem::size_of::<HMODULE>() as DWORD),
        |_, _| ntdll_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    );
    if let Err(code) = seh {
        SetLastError(RtlNtStatusToDosError(code) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetDeviceDriverBaseNameA(
    ImageBase: LPVOID,
    lpBaseName: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: SIZE_T;
    let mut LenWithNull: SIZE_T;
    let mut Module: RTL_PROCESS_MODULE_INFORMATION = core::mem::zeroed();

    /* Get the associated device driver to the base address */
    if FindDeviceDriver(ImageBase as PVOID, core::ptr::addr_of_mut!(Module)) == 0 {
        return 0;
    }

    /* And copy as much as possible to output buffer.
     * Try to add 1 to the len, to copy the null char as well.
     */
    LenWithNull = (strlen(
        (core::ptr::addr_of_mut!(Module.FullPathName) as *mut CHAR)
            .offset(Module.OffsetToFileName as isize) as *mut CHAR,
    ) as SIZE_T)
        + 1;
    Len = LenWithNull;
    if Len > nSize as SIZE_T {
        Len = nSize as SIZE_T;
    }

    memcpy(
        lpBaseName as *mut core::ffi::c_void,
        (core::ptr::addr_of_mut!(Module.FullPathName) as *mut CHAR)
            .offset(Module.OffsetToFileName as isize) as *mut CHAR
            as *const core::ffi::c_void,
        Len as usize,
    );
    /* In case we copied null char, remove it from final len */
    if Len == LenWithNull {
        Len = Len.wrapping_sub(1);
    }

    return Len as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetDeviceDriverFileNameA(
    ImageBase: LPVOID,
    lpFilename: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: SIZE_T;
    let mut LenWithNull: SIZE_T;
    let mut Module: RTL_PROCESS_MODULE_INFORMATION = core::mem::zeroed();

    /* Get the associated device driver to the base address */
    if FindDeviceDriver(ImageBase, core::ptr::addr_of_mut!(Module)) == 0 {
        return 0;
    }

    /* And copy as much as possible to output buffer.
     * Try to add 1 to the len, to copy the null char as well.
     */
    LenWithNull = strlen(core::ptr::addr_of!(Module.FullPathName) as *const CHAR) as SIZE_T + 1;
    Len = LenWithNull;
    if Len > nSize as SIZE_T {
        Len = nSize as SIZE_T;
    }

    memcpy(
        lpFilename as *mut core::ffi::c_void,
        core::ptr::addr_of!(Module.FullPathName) as *const CHAR as *const core::ffi::c_void,
        Len as size_t,
    );
    /* In case we copied null char, remove it from final len */
    if Len == LenWithNull {
        Len = Len.wrapping_sub(1);
    }

    return Len as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetDeviceDriverBaseNameW(
    ImageBase: LPVOID,
    lpBaseName: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD;
    let mut BaseName: LPSTR;

    /* Allocate internal buffer for conversion */
    BaseName = LocalAlloc(LMEM_FIXED as UINT, nSize as SIZE_T) as LPSTR;
    if BaseName == core::ptr::null_mut() {
        return 0;
    }

    /* Call A API */
    Len = GetDeviceDriverBaseNameA(ImageBase, BaseName, nSize);
    if Len == 0 {
        LocalFree(BaseName as HLOCAL);
        return 0;
    }

    /* And convert output */
    if MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        BaseName as LPCSTR,
        (if Len < nSize {
            Len.wrapping_add(1 as DWORD)
        } else {
            Len
        }) as ::core::ffi::c_int,
        lpBaseName,
        nSize as ::core::ffi::c_int,
    ) == 0 as ::core::ffi::c_int
    {
        LocalFree(BaseName as HLOCAL);
        return 0;
    }

    LocalFree(BaseName as HLOCAL);
    return Len;
}

#[no_mangle]
pub unsafe extern "system" fn GetDeviceDriverFileNameW(
    ImageBase: LPVOID,
    lpFilename: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD;
    let mut FileName: LPSTR;

    /* Allocate internal buffer for conversion */
    FileName = LocalAlloc(LMEM_FIXED as UINT, nSize as SIZE_T) as LPSTR;
    if FileName == core::ptr::null_mut() {
        return 0;
    }

    /* Call A API */
    Len = GetDeviceDriverFileNameA(ImageBase, FileName, nSize);
    if Len == 0 {
        LocalFree(FileName as HLOCAL);
        return 0;
    }

    /* And convert output */
    if MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        FileName as LPCSTR,
        (if Len < nSize {
            Len.wrapping_add(1 as DWORD)
        } else {
            Len
        }) as ::core::ffi::c_int,
        lpFilename,
        nSize as ::core::ffi::c_int,
    ) == 0
    {
        LocalFree(FileName as HLOCAL);
        return 0;
    }

    LocalFree(FileName as HLOCAL);
    return Len;
}

#[no_mangle]
pub unsafe extern "system" fn GetMappedFileNameA(
    hProcess: HANDLE,
    lpv: LPVOID,
    lpFilename: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD;
    let mut FileName: LPWSTR;

    DPRINT!("GetMappedFileNameA(%p, %p, %p, %lu)\n", hProcess, lpv, lpFilename, nSize);

    /* Allocate internal buffer for conversion */
    FileName = LocalAlloc(LMEM_FIXED as UINT, (nSize as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
    if FileName.is_null()
    {
        return 0 as DWORD;
    }

    /* Call W API */
    Len = GetMappedFileNameW(hProcess, lpv, FileName, nSize);

    /* And convert output */
    if WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        FileName as LPCWSTR,
        (if Len < nSize { Len + 1 as DWORD } else { Len }) as ::core::ffi::c_int,
        lpFilename,
        nSize as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) == 0 as ::core::ffi::c_int
    {
        Len = 0 as DWORD;
    }

    LocalFree(FileName as HLOCAL);
    return Len;
}
#[no_mangle]
pub unsafe extern "system" fn GetMappedFileNameW(
    mut hProcess: HANDLE,
    mut lpv: LPVOID,
    mut lpFilename: LPWSTR,
    mut nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD = 0;
    let mut OutSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;
    // The C's local `struct { MEMORY_SECTION_NAME; WCHAR CharBuffer[MAX_PATH]; } SectionName`.
    // The MS anonymous member puts MEMORY_SECTION_NAME's one field (ndk/mmtypes.h) directly in
    // the struct, ahead of the buffer the name's string points into.
    #[repr(C)]
    struct C2RustUnnamed_17 {
        SectionFileName: _UNICODE_STRING,
        CharBuffer: [WCHAR; MAX_PATH as usize],
    }
    let mut SectionName: C2RustUnnamed_17 = C2RustUnnamed_17 {
        SectionFileName: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null_mut::<WCHAR>(),
        },
        CharBuffer: [0; (MAX_PATH) as _],
    };
    DPRINT!(
        "GetMappedFileNameW(%p, %p, %p, %lu)\n",
        hProcess,
        lpv,
        lpFilename,
        nSize
    );
    if nSize == 0 as DWORD {
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        return 0 as DWORD;
    }
    Status = NtQueryVirtualMemory(
        hProcess,
        lpv as PVOID,
        MemorySectionName,
        &raw mut SectionName as PVOID,
        ::core::mem::size_of::<C2RustUnnamed_17>() as SIZE_T,
        &raw mut OutSize,
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return 0 as DWORD;
    }
    OutSize = (SectionName.SectionFileName.Length as usize)
        .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as SIZE_T;
    Len = OutSize as DWORD;
    if OutSize.wrapping_add(1 as SIZE_T) > nSize as SIZE_T {
        Len = nSize.wrapping_sub(1 as DWORD);
        OutSize = nSize as SIZE_T;
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
    } else {
        SetLastError(ERROR_SUCCESS as DWORD);
    }
    memcpy(
        lpFilename as *mut ::core::ffi::c_void,
        SectionName.SectionFileName.Buffer as *const ::core::ffi::c_void,
        (Len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    );
    *lpFilename.offset(Len as isize) = 0 as WCHAR;
    return OutSize as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleBaseNameA(
    hProcess: HANDLE,
    hModule: HMODULE,
    lpBaseName: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD = 0;
    let mut BaseName: PWSTR = core::ptr::null_mut::<WCHAR>();

    /* Allocate internal buffer for conversion */
    BaseName = LocalAlloc(
        LMEM_FIXED as UINT,
        (nSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if BaseName.is_null() {
        return 0 as DWORD;
    }

    /* Call W API */
    Len = GetModuleBaseNameW(hProcess, hModule, BaseName as LPWSTR, nSize);
    /* And convert output */
    if WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        BaseName as LPCWSTR,
        (if Len < nSize {
            Len.wrapping_add(1 as DWORD)
        } else {
            Len
        }) as core::ffi::c_int,
        lpBaseName,
        nSize as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) == 0 as core::ffi::c_int
    {
        Len = 0 as DWORD;
    }

    LocalFree(BaseName as HLOCAL);

    return Len;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleBaseNameW(
    hProcess: HANDLE,
    hModule: HMODULE,
    lpBaseName: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD;
    let mut Module: LDR_DATA_TABLE_ENTRY = core::mem::zeroed();

    /* Get the matching module */
    if FindModule(hProcess, hModule, core::ptr::addr_of_mut!(Module)) == 0 {
        return 0;
    }

    /* Get the maximum len we have/can write in given size */
    Len = (Module.BaseDllName.Length as usize)
        .wrapping_add(core::mem::size_of_val(&UNICODE_NULL)) as DWORD;
    if (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) < Len as usize {
        Len = (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
    }

    /* Read string */
    if ReadProcessMemory(
        hProcess,
        Module.BaseDllName.Buffer as LPCVOID,
        lpBaseName as LPVOID,
        Len as SIZE_T,
        core::ptr::null_mut(),
    ) == 0
    {
        return 0;
    }

    /* If we are at the end of the string, prepare to override to nullify string */
    if Len as usize
        == (Module.BaseDllName.Length as usize)
            .wrapping_add(core::mem::size_of_val(&UNICODE_NULL))
    {
        Len = (Len as usize).wrapping_sub(core::mem::size_of_val(&UNICODE_NULL)) as DWORD;
    }

    /* Nullify at the end if needed */
    if Len as usize >= (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) {
        if nSize != 0 {
            ASSERT!(nSize as usize >= core::mem::size_of_val(&UNICODE_NULL));
            *lpBaseName.offset((nSize - 1) as isize) = UNICODE_NULL;
        }
    }
    /* Otherwise, nullify at last written char */
    else {
        ASSERT!(
            (Len as usize).wrapping_add(core::mem::size_of::<WCHAR>())
                <= (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>())
        );
        *lpBaseName.offset((Len as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize) =
            UNICODE_NULL;
    }

    return (Len as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleFileNameExA(
    hProcess: HANDLE,
    hModule: HMODULE,
    lpFilename: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD;
    let mut Filename: PWSTR;

    /* Allocate internal buffer for conversion */
    Filename = LocalAlloc(
        LMEM_FIXED,
        (nSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if Filename.is_null() {
        return 0;
    }

    /* Call W API */
    Len = GetModuleFileNameExW(hProcess, hModule, Filename, nSize);
    /* And convert output */
    if WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        Filename as LPCWSTR,
        (if Len < nSize {
            Len.wrapping_add(1 as DWORD)
        } else {
            Len
        }) as ::core::ffi::c_int,
        lpFilename,
        nSize as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) == 0 as ::core::ffi::c_int
    {
        Len = 0 as DWORD;
    }

    LocalFree(Filename as HLOCAL);

    return Len;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleFileNameExW(
    hProcess: HANDLE,
    hModule: HMODULE,
    lpFilename: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    let mut Len: DWORD;
    let mut Module: LDR_DATA_TABLE_ENTRY = core::mem::zeroed();

    /* Get the matching module */
    if FindModule(hProcess, hModule, core::ptr::addr_of_mut!(Module)) == 0 {
        return 0;
    }

    /* Get the maximum len we have/can write in given size */
    Len = (Module.FullDllName.Length as DWORD).wrapping_add((core::mem::size_of_val(&UNICODE_NULL)) as DWORD);
    if (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) < Len as usize {
        Len = (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
    }

    /* Read string */
    if ReadProcessMemory(
        hProcess,
        Module.FullDllName.Buffer as LPCVOID,
        lpFilename as LPVOID,
        Len as SIZE_T,
        core::ptr::null_mut::<ULONG_PTR>(),
    ) == 0
    {
        return 0;
    }

    /* If we are at the end of the string, prepare to override to nullify string */
    if Len == (Module.FullDllName.Length as DWORD).wrapping_add((core::mem::size_of_val(&UNICODE_NULL)) as DWORD) {
        Len = Len.wrapping_sub((core::mem::size_of_val(&UNICODE_NULL)) as DWORD);
    }

    /* Nullify at the end if needed */
    if Len as usize >= (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) {
        if nSize != 0 {
            ASSERT!(nSize >= (core::mem::size_of_val(&UNICODE_NULL)) as DWORD);
            *lpFilename.offset(nSize.wrapping_sub((1) as DWORD) as isize) =
                UNICODE_NULL;
        }
    }
    /* Otherwise, nullify at last written char */
    else {
        ASSERT!(
            (Len as usize).wrapping_add(core::mem::size_of::<WCHAR>() as usize)
                <= (nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
        );
        *lpFilename.offset(
            (Len as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize) as isize,
        ) = UNICODE_NULL;
    }

    return (Len as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetModuleInformation(
    hProcess: HANDLE,
    hModule: HMODULE,
    lpmodinfo: LPMODULEINFO,
    cb: DWORD,
) -> BOOL {
    let mut LocalInfo: MODULEINFO = core::mem::zeroed();
    let mut Module: LDR_DATA_TABLE_ENTRY = core::mem::zeroed();

    /* Check output size */
    if (cb as usize) < core::mem::size_of::<MODULEINFO>() {
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }

    /* Get the matching module */
    if FindModule(hProcess, hModule, &raw mut Module) == 0 {
        return (FALSE) as BOOL;
    }

    /* Get a local copy first, to check for valid pointer once */
    LocalInfo.lpBaseOfDll = hModule as LPVOID;
    LocalInfo.SizeOfImage = Module.SizeOfImage as DWORD;
    LocalInfo.EntryPoint = Module.EntryPoint as LPVOID;

    /* Attempt to copy to output */
    // _SEH2_TRY { body } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { .. } _SEH2_END
    let seh = ntdll_sys::seh::try_except(
        &mut (),
        |_| {
            memcpy(
                lpmodinfo as *mut core::ffi::c_void,
                &raw mut LocalInfo as *const core::ffi::c_void,
                core::mem::size_of::<MODULEINFO>() as size_t,
            );
        },
        |_, _| ntdll_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    );
    if let Err(code) = seh {
        SetLastError(RtlNtStatusToDosError(code) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn InitializeProcessForWsWatch(hProcess: HANDLE) -> BOOL {
    let mut Status: NTSTATUS;

    /* Simply forward the call */
    Status = NtSetInformationProcess(hProcess,
                                     ProcessWorkingSetWatch,
                                     core::ptr::null_mut(),
                                     0);
    /* In case the function returns this, MS considers the call as a success */
    if NT_SUCCESS!(Status) || Status == STATUS_PORT_ALREADY_SET || Status == STATUS_ACCESS_DENIED
    {
        return (TRUE) as BOOL;
    }

    SetLastError(RtlNtStatusToDosError(Status));
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetWsChanges(
    hProcess: HANDLE,
    lpWatchInfo: PPSAPI_WS_WATCH_INFORMATION,
    cb: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;

    /* Simply forward the call */
    Status = NtQueryInformationProcess(
        hProcess,
        ProcessWorkingSetWatch,
        lpWatchInfo as LPVOID,
        cb,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessImageFileNameW(
    hProcess: HANDLE,
    lpImageFileName: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    let mut ImageFileName: PUNICODE_STRING = core::ptr::null_mut();
    let mut BufferSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;
    let mut Len: DWORD = 0;

    /* Allocate string big enough to hold name */
    BufferSize = (core::mem::size_of::<_UNICODE_STRING>() as usize)
        .wrapping_add((nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize))
        as SIZE_T;
    ImageFileName = LocalAlloc(LMEM_FIXED as UINT, BufferSize) as PUNICODE_STRING;
    if ImageFileName.is_null() {
        return 0 as DWORD;
    }

    /* Query name */
    Status = NtQueryInformationProcess(
        hProcess,
        ProcessImageFileName,
        ImageFileName as PVOID,
        BufferSize as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    /* Len mismatch => buffer too small */
    if Status == STATUS_INFO_LENGTH_MISMATCH as NTSTATUS {
        Status = STATUS_BUFFER_TOO_SMALL as NTSTATUS;
    }
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        LocalFree(ImageFileName as HLOCAL);
        return 0 as DWORD;
    }

    /* Copy name and null-terminate if possible */
    memcpy(
        lpImageFileName as *mut core::ffi::c_void,
        (*ImageFileName).Buffer as *const core::ffi::c_void,
        (*ImageFileName).Length as size_t,
    );
    Len = ((*ImageFileName).Length as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize)
        as DWORD;
    if Len < nSize {
        *lpImageFileName.offset(Len as isize) = UNICODE_NULL as WCHAR;
    }

    LocalFree(ImageFileName as HLOCAL);
    return Len;
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessImageFileNameA(
    hProcess: HANDLE,
    lpImageFileName: LPSTR,
    nSize: DWORD,
) -> DWORD {
    let mut ImageFileName: PUNICODE_STRING = core::ptr::null_mut();
    let mut BufferSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;
    let mut Len: DWORD = 0;

    /* Allocate string big enough to hold name */
    BufferSize = (core::mem::size_of::<_UNICODE_STRING>() as usize)
        .wrapping_add((nSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize))
        as SIZE_T;
    ImageFileName = LocalAlloc(LMEM_FIXED, BufferSize) as PUNICODE_STRING;
    if ImageFileName.is_null() {
        return 0;
    }

    /* Query name */
    Status = NtQueryInformationProcess(
        hProcess,
        ProcessImageFileName,
        ImageFileName as PVOID,
        BufferSize as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    /* Len mismatch => buffer too small */
    if Status == STATUS_INFO_LENGTH_MISMATCH {
        Status = STATUS_BUFFER_TOO_SMALL;
    }
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        LocalFree(ImageFileName as HLOCAL);
        return 0;
    }

    /* Copy name */
    Len = WideCharToMultiByte(
        CP_ACP,
        0 as DWORD,
        (*ImageFileName).Buffer as LPCWSTR,
        (*ImageFileName).Length as core::ffi::c_int,
        lpImageFileName,
        nSize as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    /* If conversion was successful, don't return len with added \0 */
    if Len != 0 {
        Len = (Len as core::ffi::c_ulonglong).wrapping_sub(
            core::mem::size_of::<CHAR>() as usize as core::ffi::c_ulonglong,
        ) as DWORD;
    }

    LocalFree(ImageFileName as HLOCAL);
    return Len;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPageFilesA(
    pCallbackRoutine: PENUM_PAGE_FILE_CALLBACKA,
    lpContext: LPVOID,
) -> BOOL {
    let mut Ret: BOOL;
    let mut Context: INTERNAL_ENUM_PAGE_FILES_CONTEXT = core::mem::zeroed();

    Context.dwErrCode = ERROR_SUCCESS;
    Context.lpContext = lpContext;
    Context.pCallbackRoutine = pCallbackRoutine;

    /* Call W with our own callback for W -> A conversions */
    Ret = EnumPageFilesW(
        Some(CallBackConvertToAscii),
        core::ptr::addr_of_mut!(Context) as LPVOID,
    );
    /* If we succeed but we have error code, fail and set error */
    if Ret != 0 && Context.dwErrCode != ERROR_SUCCESS {
        Ret = (FALSE as i32);
        SetLastError(Context.dwErrCode);
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPageFilesW(
    pCallbackRoutine: PENUM_PAGE_FILE_CALLBACKW,
    lpContext: LPVOID,
) -> BOOL {
    let mut Colon: PWSTR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut Size: DWORD = INIT_MEMORY_SIZE as DWORD;
    let mut Needed: DWORD = 0;
    let mut Information: ENUM_PAGE_FILE_INFORMATION = ENUM_PAGE_FILE_INFORMATION {
        cb: 0,
        Reserved: 0,
        TotalSize: 0,
        TotalInUse: 0,
        PeakUsage: 0,
    };
    let mut PageFileInfoArray: PSYSTEM_PAGEFILE_INFORMATION = core::ptr::null_mut();
    let mut PageFileInfo: PSYSTEM_PAGEFILE_INFORMATION = core::ptr::null_mut();

    /* First loop till we have all the information about page files */
    loop {
        PageFileInfoArray = LocalAlloc(LMEM_FIXED as UINT, Size as SIZE_T)
            as PSYSTEM_PAGEFILE_INFORMATION;
        if PageFileInfoArray.is_null() {
            SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES) as DWORD);
            return FALSE as BOOL;
        }

        Status = NtQuerySystemInformation(
            SystemPageFileInformation,
            PageFileInfoArray as PVOID,
            Size as ULONG,
            core::ptr::addr_of_mut!(Needed),
        );
        if NT_SUCCESS!(Status) {
            break;
        }

        LocalFree(PageFileInfoArray as HLOCAL);

        /* In case we have unexpected status, quit */
        if Status != STATUS_INFO_LENGTH_MISMATCH as NTSTATUS {
            SetLastError(RtlNtStatusToDosError(Status) as DWORD);
            return FALSE as BOOL;
        }

        /* If needed size is smaller than actual size, guess it's something to add to our current size */
        if Needed <= Size {
            Size = Size.wrapping_add(Needed);
        }
        /* Otherwise, take it as size to allocate */
        else {
            Size = Needed;
        }

        if !(TRUE != 0) {
            break;
        }
    }

    /* Start browsing all our entries */
    PageFileInfo = PageFileInfoArray;
    loop {
        /* Ensure we really have an entry */
        if (Needed as usize) < core::mem::size_of::<SYSTEM_PAGEFILE_INFORMATION>() {
            break;
        }

        /* Prepare structure to hand to the user */
        Information.Reserved = 0 as DWORD;
        Information.cb = core::mem::size_of::<ENUM_PAGE_FILE_INFORMATION>() as DWORD;
        Information.TotalSize = (*PageFileInfo).TotalSize as SIZE_T;
        Information.TotalInUse = (*PageFileInfo).TotalInUse as SIZE_T;
        Information.PeakUsage = (*PageFileInfo).PeakUsage as SIZE_T;

        /* Search for colon */
        Colon = wcschr(
            (*PageFileInfo).PageFileName.Buffer as *const WCHAR,
            0x003Au16,
        ) as PWSTR;
        /* If it's found and not at the begin of the string */
        if Colon != (0 as PWSTR)
            && Colon != (*PageFileInfo).PageFileName.Buffer
        {
            /* We can call the user callback routine with the colon */
            Colon = Colon.offset(-1);
            (pCallbackRoutine.unwrap_unchecked())(
                lpContext,
                core::ptr::addr_of_mut!(Information),
                Colon as LPCWSTR,
            );
        }

        /* If no next entry, then, it's over */
        if (*PageFileInfo).NextEntryOffset == 0 as ULONG
            || (*PageFileInfo).NextEntryOffset > Needed
        {
            break;
        }

        /* Jump to next entry while keeping accurate bytes left count */
        Needed -= (*PageFileInfo).NextEntryOffset;
        PageFileInfo = ((PageFileInfo as ULONG_PTR)
            .wrapping_add((*PageFileInfo).NextEntryOffset as ULONG_PTR))
            as PSYSTEM_PAGEFILE_INFORMATION;

        if !(TRUE != 0) {
            break;
        }
    }

    LocalFree(PageFileInfoArray as HLOCAL);
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPerformanceInfo(
    pPerformanceInformation: PPERFORMANCE_INFORMATION,
    cb: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut SystemBasicInfo: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
    let mut SystemPerfInfo: SYSTEM_PERFORMANCE_INFORMATION = core::mem::zeroed();
    let mut SystemFileCacheInfo: SYSTEM_FILECACHE_INFORMATION = core::mem::zeroed();
    let mut ProcInfoArray: PSYSTEM_PROCESS_INFORMATION = core::ptr::null_mut();
    let mut SystemProcInfo: PSYSTEM_PROCESS_INFORMATION = core::ptr::null_mut();
    let mut Size: DWORD = (INIT_MEMORY_SIZE as u32);
    let mut Needed: DWORD = 0;
    let mut ProcCount: DWORD = 0;
    let mut ThreadsCount: DWORD = 0;
    let mut HandleCount: DWORD = 0;

    /* Validate output buffer */
    if (cb as usize) < core::mem::size_of::<PERFORMANCE_INFORMATION>() {
        SetLastError(RtlNtStatusToDosError(STATUS_INFO_LENGTH_MISMATCH) as DWORD);
        return (FALSE) as BOOL;
    }

    /* First, gather as many information about the system as possible */
    Status = NtQuerySystemInformation(
        SystemBasicInformation,
        core::ptr::addr_of_mut!(SystemBasicInfo) as PVOID,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    Status = NtQuerySystemInformation(
        SystemPerformanceInformation,
        core::ptr::addr_of_mut!(SystemPerfInfo) as PVOID,
        core::mem::size_of::<SYSTEM_PERFORMANCE_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    Status = NtQuerySystemInformation(
        SystemFileCacheInformation,
        core::ptr::addr_of_mut!(SystemFileCacheInfo) as PVOID,
        core::mem::size_of::<SYSTEM_FILECACHE_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    /* Then loop till we have all the information about processes */
    loop {
        ProcInfoArray =
            LocalAlloc(LMEM_FIXED as UINT, Size as SIZE_T) as PSYSTEM_PROCESS_INFORMATION;
        if ProcInfoArray.is_null() {
            SetLastError(RtlNtStatusToDosError(STATUS_INSUFFICIENT_RESOURCES) as DWORD);
            return (FALSE) as BOOL;
        }

        Status = NtQuerySystemInformation(
            SystemProcessInformation,
            ProcInfoArray as PVOID,
            Size as ULONG,
            core::ptr::addr_of_mut!(Needed),
        );
        if NT_SUCCESS!(Status) {
            break;
        }

        LocalFree(ProcInfoArray as HLOCAL);

        /* In case we have unexpected status, quit */
        if Status != STATUS_INFO_LENGTH_MISMATCH {
            SetLastError(RtlNtStatusToDosError(Status) as DWORD);
            return (FALSE) as BOOL;
        }

        /* If needed size is smaller than actual size, guess it's something to add to our current size */
        if Needed <= Size {
            Size = Size.wrapping_add(Needed);
        }
        /* Otherwise, take it as size to allocate */
        else {
            Size = Needed;
        }

        if !((TRUE) != 0) {
            break;
        }
    }

    /* Start browsing all our entries */
    ProcCount = 0;
    HandleCount = 0;
    ThreadsCount = 0;
    SystemProcInfo = ProcInfoArray;
    loop {
        /* Ensure we really have an entry */
        if (Needed as usize) < core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() {
            break;
        }

        /* Sum procs, threads and handles */
        ProcCount = ProcCount.wrapping_add(1);
        ThreadsCount = ThreadsCount.wrapping_add((*SystemProcInfo).NumberOfThreads);
        HandleCount = HandleCount.wrapping_add((*SystemProcInfo).HandleCount);

        /* If no next entry, then, it's over */
        if (*SystemProcInfo).NextEntryOffset == 0
            || (*SystemProcInfo).NextEntryOffset > Needed
        {
            break;
        }

        /* Jump to next entry while keeping accurate bytes left count */
        Needed = Needed.wrapping_sub((*SystemProcInfo).NextEntryOffset);
        SystemProcInfo = (SystemProcInfo as ULONG_PTR)
            .wrapping_add((*SystemProcInfo).NextEntryOffset as ULONG_PTR)
            as PSYSTEM_PROCESS_INFORMATION;

        if !((TRUE) != 0) {
            break;
        }
    }

    LocalFree(ProcInfoArray as HLOCAL);

    /* Output data */
    (*pPerformanceInformation).CommitTotal = SystemPerfInfo.CommittedPages as SIZE_T;
    (*pPerformanceInformation).CommitLimit = SystemPerfInfo.CommitLimit as SIZE_T;
    (*pPerformanceInformation).CommitPeak = SystemPerfInfo.PeakCommitment as SIZE_T;
    (*pPerformanceInformation).PhysicalTotal = SystemBasicInfo.NumberOfPhysicalPages as SIZE_T;
    (*pPerformanceInformation).PhysicalAvailable = SystemPerfInfo.AvailablePages as SIZE_T;
    (*pPerformanceInformation).SystemCache =
        SystemFileCacheInfo.CurrentSizeIncludingTransitionInPages as SIZE_T;
    (*pPerformanceInformation).KernelNonpaged = SystemPerfInfo.NonPagedPoolPages as SIZE_T;
    (*pPerformanceInformation).PageSize = SystemBasicInfo.PageSize as SIZE_T;
    (*pPerformanceInformation).cb =
        core::mem::size_of::<PERFORMANCE_INFORMATION>() as DWORD;
    (*pPerformanceInformation).KernelTotal = SystemPerfInfo
        .PagedPoolPages
        .wrapping_add(SystemPerfInfo.NonPagedPoolPages) as SIZE_T;
    (*pPerformanceInformation).KernelPaged = SystemPerfInfo.PagedPoolPages as SIZE_T;
    (*pPerformanceInformation).HandleCount = HandleCount;
    (*pPerformanceInformation).ProcessCount = ProcCount;
    (*pPerformanceInformation).ThreadCount = ThreadsCount;

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessMemoryInfo(
    Process: HANDLE,
    ppsmemCounters: PPROCESS_MEMORY_COUNTERS,
    cb: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut Counters: VM_COUNTERS_EX = core::mem::zeroed();

    /* Validate output size
     * It can be either PROCESS_MEMORY_COUNTERS or PROCESS_MEMORY_COUNTERS_EX
     */
    if (cb as usize) < ::core::mem::size_of::<PROCESS_MEMORY_COUNTERS>() {
        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
        return (FALSE) as BOOL;
    }

    // _SEH2_TRY { body } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { .. } _SEH2_END. The body's
    // _SEH2_YIELD(return FALSE) cannot return through the closure: it records the result in
    // the state, and the caller returns it.
    let mut early: Option<BOOL> = None;
    let seh = ntdll_sys::seh::try_except(
        &mut early,
        |early| {
        (*ppsmemCounters).PeakPagefileUsage = 0 as SIZE_T;

        /* Query counters */
        Status = NtQueryInformationProcess(
            Process,
            ProcessVmCounters,
            core::ptr::addr_of_mut!(Counters) as PVOID,
            ::core::mem::size_of::<VM_COUNTERS_EX>() as ULONG,
            ::core::ptr::null_mut::<ULONG>(),
        );
        if NT_SUCCESS!(Status) == false {
            SetLastError(RtlNtStatusToDosError(Status) as DWORD);
            *early = Some((FALSE) as BOOL);
            return;
        }

        /* Properly set cb, according to what we received */
        if (cb as usize) >= ::core::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() {
            (*ppsmemCounters).cb = ::core::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as DWORD;
        } else {
            (*ppsmemCounters).cb = ::core::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as DWORD;
        }

        /* Output data */
        (*ppsmemCounters).PageFaultCount = Counters.PageFaultCount as DWORD;
        (*ppsmemCounters).PeakWorkingSetSize = Counters.PeakWorkingSetSize;
        (*ppsmemCounters).WorkingSetSize = Counters.WorkingSetSize;
        (*ppsmemCounters).QuotaPeakPagedPoolUsage = Counters.QuotaPeakPagedPoolUsage;
        (*ppsmemCounters).QuotaPagedPoolUsage = Counters.QuotaPagedPoolUsage;
        (*ppsmemCounters).QuotaPeakNonPagedPoolUsage = Counters.QuotaPeakNonPagedPoolUsage;
        (*ppsmemCounters).QuotaNonPagedPoolUsage = Counters.QuotaNonPagedPoolUsage;
        (*ppsmemCounters).PagefileUsage = Counters.PagefileUsage;
        (*ppsmemCounters).PeakPagefileUsage = Counters.PeakPagefileUsage;
        /* And if needed, additional field for _EX version */
        if (cb as usize) >= ::core::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() {
            (*(ppsmemCounters as PPROCESS_MEMORY_COUNTERS_EX)).PrivateUsage = Counters.PrivateUsage;
        }
        },
        |_, _| ntdll_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    );
    if let Err(code) = seh {
        SetLastError(RtlNtStatusToDosError(code) as DWORD);
        return (FALSE) as BOOL;
    }
    if let Some(ret) = early {
        return ret;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn QueryWorkingSet(hProcess: HANDLE, pv: PVOID, cb: DWORD) -> BOOL {
    let mut Status: NTSTATUS;

    /* Simply forward the call */
    Status = NtQueryVirtualMemory(
        hProcess,
        core::ptr::null_mut(),
        MemoryWorkingSetList,
        pv,
        (cb) as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn QueryWorkingSetEx(
    hProcess: HANDLE,
    pv: PVOID,
    cb: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;

    /* Simply forward the call */
    Status = NtQueryVirtualMemory(
        hProcess,
        core::ptr::null_mut(),
        MemoryWorkingSetExList,
        pv,
        cb as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        SetLastError(RtlNtStatusToDosError(Status) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
