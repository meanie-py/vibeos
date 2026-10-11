// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1652C1041F58: [u16; 4] = [0x0053u16, 0x0048, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_17D32AC93B88: [u16; 5] = [0x002Eu16, 0x0053, 0x0050, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_47F4508116AD: [u16; 10] = [0x005Cu16, 0x0025, 0x0030, 0x0035, 0x006C, 0x0075, 0x002E, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_B2FCAC26DCDA: [u16; 5] = [0x002Eu16, 0x0053, 0x0048, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_D575D22E81A4: [u16; 4] = [0x0053u16, 0x0050, 0x004C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GlobalJobList: SKIPLIST = _SKIPLIST {
    Head: _SKIPLIST_NODE {
        Distance: [0; 16],
        Next: [::core::ptr::null::<_SKIPLIST_NODE>() as *mut _SKIPLIST_NODE; 16],
        Element: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    },
    MaximumLevel: 0,
    NodeCount: 0,
    AllocateRoutine: None,
    CompareRoutine: None,
    FreeRoutine: None,
};

static mut _dwLastJobID: DWORD = 0;

static mut dwJobInfo1Offsets: [DWORD; 7] = [
    core::mem::offset_of!(JOB_INFO_1W, pPrinterName) as DWORD,
    core::mem::offset_of!(JOB_INFO_1W, pMachineName) as DWORD,
    core::mem::offset_of!(JOB_INFO_1W, pUserName) as DWORD,
    core::mem::offset_of!(JOB_INFO_1W, pDocument) as DWORD,
    core::mem::offset_of!(JOB_INFO_1W, pDatatype) as DWORD,
    core::mem::offset_of!(JOB_INFO_1W, pStatus) as DWORD,
    MAXDWORD as DWORD,
];

static mut dwJobInfo2Offsets: [DWORD; 11] = [
    core::mem::offset_of!(JOB_INFO_2W, pPrinterName) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pMachineName) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pUserName) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pDocument) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pNotifyName) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pDatatype) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pPrintProcessor) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pParameters) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pDriverName) as DWORD,
    core::mem::offset_of!(JOB_INFO_2W, pStatus) as DWORD,
    MAXDWORD as DWORD,
];

unsafe extern "C" fn _EqualStrings(pwszA: PCWSTR, pwszB: PCWSTR) -> BOOL {
    if pwszA.is_null() && pwszB.is_null() {
        return TRUE as BOOL;
    }
    if !pwszA.is_null() && pwszB.is_null() {
        return FALSE as BOOL;
    }
    if pwszA.is_null() && !pwszB.is_null() {
        return FALSE as BOOL;
    }
    return (wcscmp(pwszA, pwszB) == 0) as BOOL;
}

unsafe extern "C" fn _GetNextJobID(dwJobID: PDWORD) -> BOOL {
    _dwLastJobID = _dwLastJobID.wrapping_add(1);
    while !LookupElementSkiplist(
        core::ptr::addr_of_mut!(GlobalJobList),
        core::ptr::addr_of_mut!(_dwLastJobID) as PVOID,
        core::ptr::null_mut::<DWORD>(),
    )
    .is_null()
    {
        // This ID is already taken. Try the next one.
        _dwLastJobID = _dwLastJobID.wrapping_add(1);
    }
    if !IS_VALID_JOB_ID!(_dwLastJobID) {
        return FALSE as BOOL;
    }
    *dwJobID = _dwLastJobID;
    return TRUE as BOOL;
}
unsafe extern "system" fn _GlobalJobListCompareRoutine(FirstStruct: PVOID, SecondStruct: PVOID) -> i32 {
    let A: PLOCAL_JOB = FirstStruct as PLOCAL_JOB;
    let B: PLOCAL_JOB = SecondStruct as PLOCAL_JOB;

    return (*A).dwJobID.wrapping_sub((*B).dwJobID) as i32;
}

unsafe extern "system" fn _PrinterJobListCompareRoutine(
    FirstStruct: PVOID,
    SecondStruct: PVOID,
) -> core::ffi::c_int {
    let A: PLOCAL_JOB = FirstStruct as PLOCAL_JOB;
    let B: PLOCAL_JOB = SecondStruct as PLOCAL_JOB;
    let mut iComparison: core::ffi::c_int;
    let mut ftSubmittedA: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut ftSubmittedB: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };

    // First compare the priorities to determine the order.
    // The job with a higher priority shall come first.
    iComparison = (*A).dwPriority.wrapping_sub((*B).dwPriority) as core::ffi::c_int;
    if iComparison != 0 {
        return iComparison;
    }

    // Both have the same priority, so go by creation time.
    if SystemTimeToFileTime(
        core::ptr::addr_of_mut!((*A).stSubmitted),
        core::ptr::addr_of_mut!(ftSubmittedA),
    ) == 0
    {
        return 0;
    }

    if SystemTimeToFileTime(
        core::ptr::addr_of_mut!((*B).stSubmitted),
        core::ptr::addr_of_mut!(ftSubmittedB),
    ) == 0
    {
        return 0;
    }

    return CompareFileTime(
        core::ptr::addr_of_mut!(ftSubmittedA),
        core::ptr::addr_of_mut!(ftSubmittedB),
    ) as core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn GetJobFilePath(mut pwszExtension: PCWSTR, mut dwJobID: DWORD, mut pwszOutput: PWSTR) -> DWORD {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/jobs.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetJobFilePath\0" as *const u8 as *const ::core::ffi::c_char,
            148 as ::core::ffi::c_int,
            b"GetJobFilePath(%S, %lu, %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            pwszExtension,
            dwJobID,
            pwszOutput,
        );
    }
    if !pwszOutput.is_null() {
        CopyMemory!(
            pwszOutput as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(wszJobDirectory) as *const ::core::ffi::c_void,
            (cchJobDirectory as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
        );
        _swprintf(
            pwszOutput.offset(cchJobDirectory as isize),
            RUSTCONV_WSTR_47F4508116AD.as_ptr(),
            dwJobID,
            pwszExtension,
        );
    }
    return ((cchJobDirectory as usize)
        .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 11]>() as usize))
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeGlobalJobList() -> BOOL {
    let wszPath: [WCHAR; 11] = [
    0x005Cu16, 0x003F, 0x003F, 0x003F, 0x003F, 0x003F, 0x002E, 0x0053, 0x0048, 0x0044,
    0x0000,
];
    let cchPath: DWORD = (_countof!(wszPath) - 1) as DWORD;
    let mut dwErrorCode: DWORD = 0;
    let mut dwJobID: DWORD = 0;
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut pJob: PLOCAL_JOB = core::ptr::null_mut();
    let mut p: PWSTR = core::ptr::null_mut();
    let mut wszFullPath: [WCHAR; MAX_PATH as usize] = core::mem::zeroed();
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();

    // This one is incremented in _GetNextJobID.
    _dwLastJobID = 0;

    // Initialize an empty list for all jobs of all local printers.
    // We will search it by Job ID (supply a pointer to a DWORD in LookupElementSkiplist).
    let dll_free_spl_mem_fn = DllFreeSplMem as unsafe extern "system" fn(PVOID) -> BOOL;
    InitializeSkiplist(
        core::ptr::addr_of_mut!(GlobalJobList),
        Some(DllAllocSplMem),
        Some(_GlobalJobListCompareRoutine),
        Some(core::ptr::read(
            core::ptr::addr_of!(dll_free_spl_mem_fn) as *const unsafe extern "system" fn(PVOID),
        )),
    );

    // Construct the full path search pattern.
    CopyMemory!(
        core::ptr::addr_of_mut!(wszFullPath) as *mut WCHAR as *mut core::ffi::c_void,
        core::ptr::addr_of!(wszJobDirectory) as *const WCHAR as *const core::ffi::c_void,
        (cchJobDirectory as usize) * core::mem::size_of::<WCHAR>(),
    );
    CopyMemory!(
        (core::ptr::addr_of_mut!(wszFullPath) as *mut WCHAR).offset(cchJobDirectory as isize)
            as *mut core::ffi::c_void,
        core::ptr::addr_of!(wszPath) as *const WCHAR as *const core::ffi::c_void,
        ((cchPath + 1) as usize) * core::mem::size_of::<WCHAR>(),
    );

    'cleanup: {
        // Use the search pattern to look for unfinished jobs serialized in shadow files (.SHD)
        hFind = FindFirstFileW(
            core::ptr::addr_of_mut!(wszFullPath) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(FindData),
        );
        if hFind == INVALID_HANDLE_VALUE {
            // No unfinished jobs found.
            dwErrorCode = ERROR_SUCCESS;
            break 'cleanup;
        }

        loop {
            'body: {
                // Skip possible subdirectories.
                if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    break 'body;
                }

                // Extract the Job ID and verify the file name format at the same time.
                // This includes all valid names (like "00005.SHD") and excludes invalid ones (like "10ABC.SHD").
                dwJobID = wcstoul(
                    core::ptr::addr_of_mut!(FindData.cFileName) as *mut WCHAR,
                    core::ptr::addr_of_mut!(p),
                    10,
                ) as DWORD;
                if !IS_VALID_JOB_ID!(dwJobID) {
                    break 'body;
                }

                if _wcsicmp(p, RUSTCONV_WSTR_B2FCAC26DCDA.as_ptr()) != 0 {
                    break 'body;
                }

                // This shadow file has a valid name. Construct the full path and try to load it.
                GetJobFilePath(
                    RUSTCONV_WSTR_1652C1041F58.as_ptr(),
                    dwJobID,
                    core::ptr::addr_of_mut!(wszFullPath) as *mut WCHAR as PWSTR,
                );
                pJob = ReadJobShadowFile(
                    core::ptr::addr_of_mut!(wszFullPath) as *mut WCHAR as PCWSTR,
                );
                if pJob.is_null() {
                    break 'body;
                }

                // Add it to the Global Job List.
                if InsertElementSkiplist(
                    core::ptr::addr_of_mut!(GlobalJobList),
                    pJob as PVOID,
                ) == 0
                {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    break 'cleanup;
                }

                // Add it to the Printer's Job List.
                if InsertElementSkiplist(
                    core::ptr::addr_of_mut!((*(*pJob).pPrinter).JobList),
                    pJob as PVOID,
                ) == 0
                {
                    dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                    break 'cleanup;
                }
            }
            // do-while condition (reached by normal completion or `continue`)
            if !(FindNextFileW(
                hFind,
                core::ptr::addr_of_mut!(FindData),
            ) != 0)
            {
                break;
            }
        }

        dwErrorCode = ERROR_SUCCESS;
    }

    // Outside the loop
    if !hFind.is_null() {
        FindClose(hFind);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitializePrinterJobList(mut pPrinter: PLOCAL_PRINTER) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/jobs.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"InitializePrinterJobList\0" as *const u8 as *const ::core::ffi::c_char,
            249 as ::core::ffi::c_int,
            b"InitializePrinterJobList(%p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            pPrinter,
        );
    }
    InitializeSkiplist(
        &raw mut (*pPrinter).JobList,
        Some(DllAllocSplMem as unsafe extern "system" fn(DWORD) -> PVOID),
        Some(
            _PrinterJobListCompareRoutine
                as unsafe extern "system" fn(PVOID, PVOID) -> ::core::ffi::c_int,
        ),
        ::core::mem::transmute::<Option<unsafe extern "system" fn(PVOID) -> BOOL>, PSKIPLIST_FREE_ROUTINE>(
            Some(DllFreeSplMem as unsafe extern "system" fn(PVOID) -> BOOL),
        ),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CreateJob(pPrinterHandle: PLOCAL_PRINTER_HANDLE) -> DWORD {
    let wszDoubleBackslash: [u16; 2] = [0x005Cu16, 0x0000];
    let cchDoubleBackslash: DWORD = _countof!(wszDoubleBackslash) as DWORD - 1 as DWORD;

    let mut cchMachineName: DWORD = 0;
    let mut cchUserName: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut pJob: PLOCAL_JOB = core::ptr::null_mut();
    let mut hServerBinding: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut pwszBinding: RPC_WSTR = core::ptr::null_mut();
    let mut pwszMachineName: RPC_WSTR = core::ptr::null_mut();

    'Cleanup: {
        // Create a new job.
        pJob = DllAllocSplMem(core::mem::size_of::<LOCAL_JOB>() as DWORD) as PLOCAL_JOB;
        if pJob.is_null() {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'Cleanup;
        }

        // Reserve an ID for this job.
        if _GetNextJobID(&raw mut (*pJob).dwJobID) == 0 {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'Cleanup;
        }

        // Copy over defaults to the LOCAL_JOB structure.
        (*pJob).pPrinter = (*pPrinterHandle).pPrinter;
        (*pJob).pPrintProcessor = (*(*pPrinterHandle).pPrinter).pPrintProcessor;
        (*pJob).dwPriority = DEF_PRIORITY as DWORD;
        (*pJob).dwStatus = JOB_STATUS_SPOOLING as DWORD;
        (*pJob).pwszDatatype = AllocSplStr((*pPrinterHandle).pwszDatatype as PCWSTR);
        (*pJob).pwszDocumentName = AllocSplStr(core::ptr::addr_of!(wszDefaultDocumentName) as PCWSTR);
        (*pJob).pDevMode = DuplicateDevMode((*pPrinterHandle).pDevMode);
        GetSystemTime(&raw mut (*pJob).stSubmitted);

        // Get the user name for the Job.
        cchUserName = (UNLEN + 1) as DWORD;
        (*pJob).pwszUserName = DllAllocSplMem(
            (cchUserName as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        if GetUserNameW((*pJob).pwszUserName as LPWSTR, &raw mut cchUserName) == 0 {
            dwErrorCode = GetLastError();
            break 'Cleanup;
        }

        // FIXME: For now, pwszNotifyName equals pwszUserName.
        (*pJob).pwszNotifyName = AllocSplStr((*pJob).pwszUserName as PCWSTR);

        // Get the name of the machine that submitted the Job over RPC.
        dwErrorCode = RpcBindingServerFromClient(core::ptr::null_mut(), &raw mut hServerBinding) as DWORD;
        if dwErrorCode != RPC_S_OK as DWORD {
            break 'Cleanup;
        }

        dwErrorCode = RpcBindingToStringBindingW(hServerBinding, &raw mut pwszBinding) as DWORD;
        if dwErrorCode != RPC_S_OK as DWORD {
            break 'Cleanup;
        }

        dwErrorCode = RpcStringBindingParseW(
            pwszBinding,
            core::ptr::null_mut::<RPC_WSTR>(),
            core::ptr::null_mut::<RPC_WSTR>(),
            &raw mut pwszMachineName,
            core::ptr::null_mut::<RPC_WSTR>(),
            core::ptr::null_mut::<RPC_WSTR>(),
        ) as DWORD;
        if dwErrorCode != RPC_S_OK as DWORD {
            break 'Cleanup;
        }

        cchMachineName = wcslen(pwszMachineName as *const WCHAR) as DWORD;
        (*pJob).pwszMachineName = DllAllocSplMem(
            (cchMachineName
                .wrapping_add(cchDoubleBackslash)
                .wrapping_add(1 as DWORD) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        ) as PWSTR;
        CopyMemory!(
            (*pJob).pwszMachineName as *mut core::ffi::c_void,
            core::ptr::addr_of!(wszDoubleBackslash) as *const core::ffi::c_void,
            (cchDoubleBackslash as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        CopyMemory!(
            (*pJob).pwszMachineName.offset(cchDoubleBackslash as isize) as *mut WCHAR as *mut core::ffi::c_void,
            pwszMachineName as *const core::ffi::c_void,
            (cchMachineName.wrapping_add(1 as DWORD) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        );

        // Add the job to the Global Job List.
        if InsertElementSkiplist(&raw mut GlobalJobList, pJob as PVOID) == 0 {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'Cleanup;
        }

        // Add the job at the end of the Printer's Job List.
        // As all new jobs are created with default priority, we can be sure that it would always be inserted at the end.
        if InsertTailElementSkiplist(&raw mut (*(*pJob).pPrinter).JobList, pJob as PVOID) == 0 {
            dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'Cleanup;
        }

        // We were successful!
        (*pPrinterHandle).set_bStartedDoc(TRUE as BOOL);
        (*pPrinterHandle).pJob = pJob;
        dwErrorCode = ERROR_SUCCESS as DWORD;

        // Don't let the cleanup routine free this.
        pJob = core::ptr::null_mut();
    }

    // Cleanup:
    if !pJob.is_null() {
        DllFreeSplMem(pJob as PVOID);
    }

    if !pwszMachineName.is_null() {
        RpcStringFreeW(&raw mut pwszMachineName);
    }

    if !pwszBinding.is_null() {
        RpcStringFreeW(&raw mut pwszBinding);
    }

    if !hServerBinding.is_null() {
        RpcBindingFree(&raw mut hServerBinding);
    }

    return dwErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn LocalAddJob(hPrinter: HANDLE, Level: DWORD, pData: LPBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD) -> BOOL {
    let mut AddJobInfo1: ADDJOB_INFO_1W = core::mem::zeroed();
    let mut dwErrorCode: DWORD = 0;
    let mut pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();

    'cleanup: {
        // Check if this is a printer handle.
        if (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // This handle must not have started a job yet!
        if !(*pPrinterHandle).pJob.is_null() {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        // Check if this is the right structure level.
        if Level != 1 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'cleanup;
        }

        // Check if the printer is set to do direct printing.
        // The Job List isn't used in this case.
        if (*(*pPrinterHandle).pPrinter).dwAttributes & PRINTER_ATTRIBUTE_DIRECT != 0 {
            dwErrorCode = ERROR_INVALID_ACCESS;
            break 'cleanup;
        }

        // Check if the supplied buffer is large enough.
        *pcbNeeded = (core::mem::size_of::<ADDJOB_INFO_1W>() as DWORD).wrapping_add(
            GetJobFilePath(
                RUSTCONV_WSTR_D575D22E81A4.as_ptr(),
                0,
                core::ptr::null_mut(),
            ) as DWORD,
        );
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'cleanup;
        }

        // All requirements are met - create a new job.
        dwErrorCode = CreateJob(pPrinterHandle);
        if dwErrorCode != ERROR_SUCCESS {
            break 'cleanup;
        }

        // Mark that this job was started with AddJob (so that it can be scheduled for printing with ScheduleJob).
        (*(*pPrinterHandle).pJob).set_bAddedJob(TRUE as BOOL);

        // Return a proper ADDJOB_INFO_1W structure.
        AddJobInfo1.JobId = (*(*pPrinterHandle).pJob).dwJobID;
        AddJobInfo1.Path = pData
            .offset(core::mem::size_of::<ADDJOB_INFO_1W>() as isize)
            as PWSTR;

        CopyMemory!(
            pData as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(AddJobInfo1) as *const core::ffi::c_void,
            core::mem::size_of::<ADDJOB_INFO_1W>(),
        );
        GetJobFilePath(
            RUSTCONV_WSTR_D575D22E81A4.as_ptr(),
            AddJobInfo1.JobId,
            AddJobInfo1.Path,
        );
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

unsafe extern "C" fn _LocalGetJobLevel1(
    mut pJob: PLOCAL_JOB,
    mut ppJobInfo: *mut PJOB_INFO_1W,
    mut ppJobInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
) {
    let mut cbDatatype: DWORD;
    let mut cbDocumentName: DWORD = 0 as DWORD;
    let mut cbMachineName: DWORD;
    let mut cbPrinterName: DWORD;
    let mut cbStatus: DWORD = 0 as DWORD;
    let mut cbUserName: DWORD = 0 as DWORD;
    let mut pwszStrings: [PCWSTR; 6 as usize] =
        [core::ptr::null(); 6 as usize];

    // Calculate the string lengths.
    if ppJobInfo.is_null() {
        cbDatatype = (wcslen((*pJob).pwszDatatype) as DWORD
            + 1 as DWORD)
            * core::mem::size_of::<WCHAR>() as DWORD;
        cbMachineName = (wcslen((*pJob).pwszMachineName) as DWORD
            + 1 as DWORD)
            * core::mem::size_of::<WCHAR>() as DWORD;
        cbPrinterName = (wcslen((*(*pJob).pPrinter).pwszPrinterName) as DWORD
            + 1 as DWORD)
            * core::mem::size_of::<WCHAR>() as DWORD;

        // These values are optional.
        if !(*pJob).pwszDocumentName.is_null() {
            cbDocumentName = (wcslen((*pJob).pwszDocumentName) as DWORD
                + 1 as DWORD)
                * core::mem::size_of::<WCHAR>() as DWORD;
        }

        if !(*pJob).pwszStatus.is_null() {
            cbStatus = (wcslen((*pJob).pwszStatus) as DWORD
                + 1 as DWORD)
                * core::mem::size_of::<WCHAR>() as DWORD;
        }

        if !(*pJob).pwszUserName.is_null() {
            cbUserName = (wcslen((*pJob).pwszUserName) as DWORD
                + 1 as DWORD)
                * core::mem::size_of::<WCHAR>() as DWORD;
        }

        *pcbNeeded += core::mem::size_of::<JOB_INFO_1W>() as DWORD
            + cbDatatype
            + cbDocumentName
            + cbMachineName
            + cbPrinterName
            + cbStatus
            + cbUserName;
        return;
    }

    // Set the general fields.
    (**ppJobInfo).JobId = (*pJob).dwJobID;
    (**ppJobInfo).Status = (*pJob).dwStatus;
    (**ppJobInfo).Priority = (*pJob).dwPriority;
    (**ppJobInfo).TotalPages = (*pJob).dwTotalPages;
    (**ppJobInfo).PagesPrinted = (*pJob).dwPagesPrinted;
    CopyMemory!(
        core::ptr::addr_of_mut!((**ppJobInfo).Submitted) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*pJob).stSubmitted) as *const core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>() as usize,
    );

    // Position in JOB_INFO_1W is the 1-based index of the job in the processing queue.
    // Retrieve this through the element index of the job in the Printer's Job List.
    if LookupElementSkiplist(
        core::ptr::addr_of_mut!((*(*pJob).pPrinter).JobList),
        pJob as PVOID,
        core::ptr::addr_of_mut!((**ppJobInfo).Position),
    )
    .is_null()
    {
        return;
    }

    // Make the index 1-based.
    (**ppJobInfo).Position = (**ppJobInfo).Position.wrapping_add(1);

    // Set the pPrinterName field.
    pwszStrings[0 as usize] =
        (*(*pJob).pPrinter).pwszPrinterName as PCWSTR;

    // Set the pMachineName field.
    pwszStrings[1 as usize] = (*pJob).pwszMachineName as PCWSTR;

    // Set the pUserName field.
    pwszStrings[2 as usize] = (*pJob).pwszUserName as PCWSTR;

    // Set the pDocument field.
    pwszStrings[3 as usize] = (*pJob).pwszDocumentName as PCWSTR;

    // Set the pDatatype field.
    pwszStrings[4 as usize] = (*pJob).pwszDatatype as PCWSTR;

    // Set the pStatus field.
    pwszStrings[5 as usize] = (*pJob).pwszStatus as PCWSTR;

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppJobInfoEnd = PackStrings(
        core::ptr::addr_of_mut!(pwszStrings) as *mut PCWSTR,
        *ppJobInfo as PBYTE,
        core::ptr::addr_of_mut!(dwJobInfo1Offsets) as *mut DWORD,
        *ppJobInfoEnd,
    );
    *ppJobInfo = (*ppJobInfo).add(1);
}

unsafe extern "C" fn _LocalGetJobLevel2(
    mut pJob: PLOCAL_JOB,
    mut ppJobInfo: *mut PJOB_INFO_2W,
    mut ppJobInfoEnd: *mut PBYTE,
    mut pcbNeeded: PDWORD,
) {
    let mut cbDatatype: DWORD = 0;
    let mut cbDevMode: DWORD = 0;
    let mut cbDocumentName: DWORD = 0;
    let mut cbDriverName: DWORD = 0;
    let mut cbMachineName: DWORD = 0;
    let mut cbNotifyName: DWORD = 0;
    let mut cbPrinterName: DWORD = 0;
    let mut cbPrintProcessor: DWORD = 0;
    let mut cbPrintProcessorParameters: DWORD = 0;
    let mut cbStatus: DWORD = 0;
    let mut cbUserName: DWORD = 0;
    let mut ftNow: FILETIME = core::mem::zeroed();
    let mut ftSubmitted: FILETIME = core::mem::zeroed();
    let mut pwszStrings: [PCWSTR; 10] =
        [core::ptr::null(); 10];
    let mut uliNow: ULARGE_INTEGER = core::mem::zeroed();
    let mut uliSubmitted: ULARGE_INTEGER = core::mem::zeroed();

    // Calculate the string lengths.
    cbDevMode = ((*(*pJob).pDevMode).dmSize as DWORD)
        + ((*(*pJob).pDevMode).dmDriverExtra as DWORD);

    if ppJobInfo.is_null() {
        cbDatatype = ((wcslen((*pJob).pwszDatatype) as DWORD)
            + 1)
            * (core::mem::size_of::<WCHAR>() as DWORD);
        cbDriverName = ((wcslen((*(*pJob).pPrinter).pwszPrinterDriver) as DWORD)
            + 1)
            * (core::mem::size_of::<WCHAR>() as DWORD);
        cbMachineName = ((wcslen((*pJob).pwszMachineName) as DWORD)
            + 1)
            * (core::mem::size_of::<WCHAR>() as DWORD);
        cbPrinterName = ((wcslen((*(*pJob).pPrinter).pwszPrinterName) as DWORD)
            + 1)
            * (core::mem::size_of::<WCHAR>() as DWORD);
        cbPrintProcessor = ((wcslen((*(*pJob).pPrintProcessor).pwszName) as DWORD)
            + 1)
            * (core::mem::size_of::<WCHAR>() as DWORD);

        // These values are optional.
        if !(*pJob).pwszDocumentName.is_null() {
            cbDocumentName = ((wcslen((*pJob).pwszDocumentName) as DWORD)
                + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD);
        }

        if !(*pJob).pwszNotifyName.is_null() {
            cbNotifyName = ((wcslen((*pJob).pwszNotifyName) as DWORD)
                + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD);
        }

        if !(*pJob).pwszPrintProcessorParameters.is_null() {
            cbPrintProcessorParameters = ((wcslen(
                (*pJob).pwszPrintProcessorParameters,
            ) as DWORD)
                + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD);
        }

        if !(*pJob).pwszStatus.is_null() {
            cbStatus = ((wcslen((*pJob).pwszStatus) as DWORD)
                + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD);
        }

        if !(*pJob).pwszUserName.is_null() {
            cbUserName = ((wcslen((*pJob).pwszUserName) as DWORD)
                + 1)
                * (core::mem::size_of::<WCHAR>() as DWORD);
        }

        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (core::mem::size_of::<JOB_INFO_2W>() as DWORD)
                .wrapping_add(cbDatatype)
                .wrapping_add(cbDevMode)
                .wrapping_add(cbDocumentName)
                .wrapping_add(cbDriverName)
                .wrapping_add(cbMachineName)
                .wrapping_add(cbNotifyName)
                .wrapping_add(cbPrinterName)
                .wrapping_add(cbPrintProcessor)
                .wrapping_add(cbPrintProcessorParameters)
                .wrapping_add(cbStatus)
                .wrapping_add(cbUserName),
        );
        return;
    }

    // Set the general fields.
    (**ppJobInfo).JobId = (*pJob).dwJobID;
    (**ppJobInfo).Status = (*pJob).dwStatus;
    (**ppJobInfo).Priority = (*pJob).dwPriority;
    (**ppJobInfo).StartTime = (*pJob).dwStartTime;
    (**ppJobInfo).UntilTime = (*pJob).dwUntilTime;
    (**ppJobInfo).TotalPages = (*pJob).dwTotalPages;
    (**ppJobInfo).PagesPrinted = (*pJob).dwPagesPrinted;
    CopyMemory!(
        &raw mut (**ppJobInfo).Submitted as *mut ::core::ffi::c_void,
        &raw mut (*pJob).stSubmitted as *const ::core::ffi::c_void,
        core::mem::size_of::<SYSTEMTIME>(),
    );

    // Time in JOB_INFO_2W is the number of milliseconds elapsed since the job was submitted. Calculate this time.
    if SystemTimeToFileTime(&raw mut (*pJob).stSubmitted, &raw mut ftSubmitted) == 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/printing/providers/localspl/jobs.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"_LocalGetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                587 as ::core::ffi::c_int,
                b"SystemTimeToFileTime failed with error %lu!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                GetLastError(),
            );
        }
        return;
    }

    GetSystemTimeAsFileTime(&raw mut ftNow);
    uliSubmitted.__bindgen_anon_1.LowPart = ftSubmitted.dwLowDateTime;
    uliSubmitted.__bindgen_anon_1.HighPart = ftSubmitted.dwHighDateTime;
    uliNow.__bindgen_anon_1.LowPart = ftNow.dwLowDateTime;
    uliNow.__bindgen_anon_1.HighPart = ftNow.dwHighDateTime;
    (**ppJobInfo).Time = uliNow
        .QuadPart
        .wrapping_sub(uliSubmitted.QuadPart)
        .wrapping_div(10000) as DWORD;

    // Position in JOB_INFO_2W is the 1-based index of the job in the processing queue.
    // Retrieve this through the element index of the job in the Printer's Job List.
    if LookupElementSkiplist(
        &raw mut (*(*pJob).pPrinter).JobList,
        pJob as PVOID,
        &raw mut (**ppJobInfo).Position,
    )
    .is_null()
    {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/printing/providers/localspl/jobs.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"_LocalGetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                602 as ::core::ffi::c_int,
                b"pJob could not be located in the Printer's Job List!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return;
    }

    // Make the index 1-based.
    (**ppJobInfo).Position = (**ppJobInfo).Position.wrapping_add(1);

    // FIXME!
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int))
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"win32ss/printing/providers/localspl/jobs.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"_LocalGetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
            610 as ::core::ffi::c_int,
            b"Setting pSecurityDescriptor and Size to 0 for now!\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    (**ppJobInfo).pSecurityDescriptor = core::ptr::null_mut() as PSECURITY_DESCRIPTOR;
    (**ppJobInfo).Size = 0;

    // Set the pDevMode field (and copy the DevMode).
    *ppJobInfoEnd = (*ppJobInfoEnd).offset(-(cbDevMode as isize));
    CopyMemory!(
        *ppJobInfoEnd as *mut ::core::ffi::c_void,
        (*pJob).pDevMode as *const ::core::ffi::c_void,
        cbDevMode as usize,
    );
    (**ppJobInfo).pDevMode = *ppJobInfoEnd as PDEVMODEW as LPDEVMODEW;

    // Set the pPrinterName field.
    pwszStrings[0] = (*(*pJob).pPrinter).pwszPrinterName as PCWSTR;

    // Set the pMachineName field.
    pwszStrings[1] = (*pJob).pwszMachineName as PCWSTR;

    // Set the pUserName field.
    pwszStrings[2] = (*pJob).pwszUserName as PCWSTR;

    // Set the pDocument field.
    pwszStrings[3] = (*pJob).pwszDocumentName as PCWSTR;

    // Set the pNotifyName field.
    pwszStrings[4] = (*pJob).pwszNotifyName as PCWSTR;

    // Set the pDatatype field.
    pwszStrings[5] = (*pJob).pwszDatatype as PCWSTR;

    // Set the pPrintProcessor field.
    pwszStrings[6] = (*(*pJob).pPrintProcessor).pwszName as PCWSTR;

    // Set the pParameters field.
    pwszStrings[7] = (*pJob).pwszPrintProcessorParameters as PCWSTR;

    // Set the pDriverName field.
    pwszStrings[8] = (*(*pJob).pPrinter).pwszPrinterDriver as PCWSTR;

    // Set the pStatus field.
    pwszStrings[9] = (*pJob).pwszStatus as PCWSTR;

    // Finally copy the structure and advance to the next one in the output buffer.
    *ppJobInfoEnd = PackStrings(
        &raw mut pwszStrings as *mut PCWSTR,
        *ppJobInfo as PBYTE,
        &raw mut dwJobInfo2Offsets as *mut DWORD,
        *ppJobInfoEnd,
    );
    *ppJobInfo = (*ppJobInfo).offset(1);
}

#[no_mangle]
pub unsafe extern "system" fn LocalGetJob(mut hPrinter: HANDLE, mut JobId: DWORD, mut Level: DWORD, mut pStart: PBYTE, mut cbBuf: DWORD, mut pcbNeeded: LPDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pEnd: PBYTE = pStart.add(cbBuf as usize);
    let mut pHandle: PLOCAL_HANDLE;
    let mut pJob: PLOCAL_JOB;
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE;

    'Cleanup: {
        // Check if this is a printer handle.
        pHandle = hPrinter as PLOCAL_HANDLE;
        if (*pHandle).HandleType != HandleType_Printer as _ {
            dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
            break 'Cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // Get the desired job.
        pJob = LookupElementSkiplist(
            &raw mut GlobalJobList,
            &raw mut JobId as PVOID,
            ::core::ptr::null_mut::<DWORD>(),
        ) as PLOCAL_JOB;
        if pJob.is_null() || (*pJob).pPrinter != (*pPrinterHandle).pPrinter {
            dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
            break 'Cleanup;
        }

        if Level > 2 as DWORD {
            // The caller supplied an invalid level for GetJob.
            dwErrorCode = ERROR_INVALID_LEVEL as DWORD;
            break 'Cleanup;
        }

        // Count the required buffer size.
        *pcbNeeded = 0 as DWORD;

        if Level == 1 as DWORD {
            _LocalGetJobLevel1(
                pJob,
                ::core::ptr::null_mut::<PJOB_INFO_1W>(),
                ::core::ptr::null_mut::<PBYTE>(),
                pcbNeeded,
            );
        } else if Level == 2 as DWORD {
            _LocalGetJobLevel2(
                pJob,
                ::core::ptr::null_mut::<PJOB_INFO_2W>(),
                ::core::ptr::null_mut::<PBYTE>(),
                pcbNeeded,
            );
        }

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER as DWORD;
            break 'Cleanup;
        }

        // Copy over the Job information.
        pEnd = pStart.add(*pcbNeeded as usize);

        if Level == 1 as DWORD {
            _LocalGetJobLevel1(
                pJob,
                &raw mut pStart as *mut PJOB_INFO_1W,
                &raw mut pEnd,
                ::core::ptr::null_mut::<DWORD>(),
            );
        } else if Level == 2 as DWORD {
            _LocalGetJobLevel2(
                pJob,
                &raw mut pStart as *mut PJOB_INFO_2W,
                &raw mut pEnd,
                ::core::ptr::null_mut::<DWORD>(),
            );
        }

        dwErrorCode = ERROR_SUCCESS as DWORD;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

unsafe extern "C" fn _LocalSetJobLevel1(
    pPrinterHandle: PLOCAL_PRINTER_HANDLE,
    pJob: PLOCAL_JOB,
    pJobInfo: PJOB_INFO_1W,
) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    'Cleanup: {
        // First check the validity of the input before changing anything.
        if FindDatatype((*pJob).pPrintProcessor, (*pJobInfo).pDatatype as PCWSTR) == 0 {
            dwErrorCode = ERROR_INVALID_DATATYPE;
            break 'Cleanup;
        }

        // Check if the datatype has changed.
        if _EqualStrings((*pJob).pwszDatatype as PCWSTR, (*pJobInfo).pDatatype as PCWSTR) == 0 {
            // Use the new value.
            if ReallocSplStr(
                core::ptr::addr_of_mut!((*pJob).pwszDatatype),
                (*pJobInfo).pDatatype as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }
        }

        // Check if the document name has changed. An empty string is permitted here!
        if _EqualStrings((*pJob).pwszDocumentName as PCWSTR, (*pJobInfo).pDocument as PCWSTR) == 0 {
            // Use the new value.
            if ReallocSplStr(
                core::ptr::addr_of_mut!((*pJob).pwszDocumentName),
                (*pJobInfo).pDocument as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }
        }

        // Check if the status message has changed. An empty string is permitted here!
        if _EqualStrings((*pJob).pwszStatus as PCWSTR, (*pJobInfo).pStatus as PCWSTR) == 0 {
            // Use the new value.
            if ReallocSplStr(
                core::ptr::addr_of_mut!((*pJob).pwszStatus),
                (*pJobInfo).pStatus as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }
        }

        // Check if the user name has changed. An empty string is permitted here!
        if _EqualStrings((*pJob).pwszUserName as PCWSTR, (*pJobInfo).pUserName as PCWSTR) == 0 {
            // The new user name doesn't need to exist, so no additional verification is required.

            // Use the new value.
            if ReallocSplStr(
                core::ptr::addr_of_mut!((*pJob).pwszUserName),
                (*pJobInfo).pUserName as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'Cleanup;
            }
        }

        // Check if the priority has changed.
        if (*pJob).dwPriority != (*pJobInfo).Priority && IS_VALID_PRIORITY!((*pJobInfo).Priority) {
            // Set the new priority.
            (*pJob).dwPriority = (*pJobInfo).Priority;

            // Remove and reinsert the job in the Printer's Job List.
            // The Compare function will be used to find the right position now considering the new priority.
            DeleteElementSkiplist(core::ptr::addr_of_mut!((*(*pJob).pPrinter).JobList), pJob as PVOID);
            InsertElementSkiplist(core::ptr::addr_of_mut!((*(*pJob).pPrinter).JobList), pJob as PVOID);
        }

        // Check if the status flags have changed.
        if (*pJob).dwStatus != (*pJobInfo).Status {
            // Only add status flags that make sense.
            if ((*pJobInfo).Status & JOB_STATUS_PAUSED) != 0 {
                (*pJob).dwStatus |= JOB_STATUS_PAUSED;
            }

            if ((*pJobInfo).Status & JOB_STATUS_ERROR) != 0 {
                (*pJob).dwStatus |= JOB_STATUS_ERROR;
            }

            if ((*pJobInfo).Status & JOB_STATUS_OFFLINE) != 0 {
                (*pJob).dwStatus |= JOB_STATUS_OFFLINE;
            }

            if ((*pJobInfo).Status & JOB_STATUS_PAPEROUT) != 0 {
                (*pJob).dwStatus |= JOB_STATUS_PAPEROUT;
            }
        }

        dwErrorCode = ERROR_SUCCESS;
    }
    return dwErrorCode;
}

unsafe extern "C" fn _LocalSetJobLevel2(
    mut pPrinterHandle: PLOCAL_PRINTER_HANDLE,
    mut pJob: PLOCAL_JOB,
    mut pJobInfo: PJOB_INFO_2W,
) -> DWORD {
    let mut dwErrorCode: DWORD = 0;
    let mut pPrintProcessor: PLOCAL_PRINT_PROCESSOR = core::ptr::null_mut();

    'Cleanup: {
        // First check the validity of the input before changing anything.
        pPrintProcessor = FindPrintProcessor((*pJobInfo).pPrintProcessor as PCWSTR);
        if pPrintProcessor.is_null() {
            dwErrorCode = ERROR_UNKNOWN_PRINTPROCESSOR as DWORD;
            break 'Cleanup;
        }

        if FindDatatype(pPrintProcessor, (*pJobInfo).pDatatype as PCWSTR) == 0 {
            dwErrorCode = ERROR_INVALID_DATATYPE as DWORD;
            break 'Cleanup;
        }

        // Check if the datatype has changed.
        if _EqualStrings(
            (*pJob).pwszDatatype as PCWSTR,
            (*pJobInfo).pDatatype as PCWSTR,
        ) == 0
        {
            // Use the new value.
            if ReallocSplStr(
                &raw mut (*pJob).pwszDatatype,
                (*pJobInfo).pDatatype as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"win32ss/printing/providers/localspl/jobs.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"_LocalSetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                        844 as ::core::ffi::c_int,
                        b"ReallocSplStr failed, last error is %lu!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        GetLastError(),
                    );
                }
                break 'Cleanup;
            }
        }

        // Check if the document name has changed. An empty string is permitted here!
        if _EqualStrings(
            (*pJob).pwszDocumentName as PCWSTR,
            (*pJobInfo).pDocument as PCWSTR,
        ) == 0
        {
            // Use the new value.
            if ReallocSplStr(
                &raw mut (*pJob).pwszDocumentName,
                (*pJobInfo).pDocument as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"win32ss/printing/providers/localspl/jobs.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"_LocalSetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                        856 as ::core::ffi::c_int,
                        b"ReallocSplStr failed, last error is %lu!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        GetLastError(),
                    );
                }
                break 'Cleanup;
            }
        }

        // Check if the notify name has changed. An empty string is permitted here!
        if _EqualStrings(
            (*pJob).pwszNotifyName as PCWSTR,
            (*pJobInfo).pNotifyName as PCWSTR,
        ) == 0
        {
            // The new notify name doesn't need to exist, so no additional verification is required.

            // Use the new value.
            if ReallocSplStr(
                &raw mut (*pJob).pwszNotifyName,
                (*pJobInfo).pNotifyName as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_1,
                        __dbch_1,
                        b"win32ss/printing/providers/localspl/jobs.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"_LocalSetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                        870 as ::core::ffi::c_int,
                        b"ReallocSplStr failed, last error is %lu!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        GetLastError(),
                    );
                }
                break 'Cleanup;
            }
        }

        // Check if the Print Processor Parameters have changed. An empty string is permitted here!
        if _EqualStrings(
            (*pJob).pwszPrintProcessorParameters as PCWSTR,
            (*pJobInfo).pParameters as PCWSTR,
        ) == 0
        {
            // Use the new value.
            if ReallocSplStr(
                &raw mut (*pJob).pwszPrintProcessorParameters,
                (*pJobInfo).pParameters as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"win32ss/printing/providers/localspl/jobs.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"_LocalSetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                        882 as ::core::ffi::c_int,
                        b"ReallocSplStr failed, last error is %lu!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        GetLastError(),
                    );
                }
                break 'Cleanup;
            }
        }

        // Check if the Status Message has changed. An empty string is permitted here!
        if _EqualStrings(
            (*pJob).pwszStatus as PCWSTR,
            (*pJobInfo).pStatus as PCWSTR,
        ) == 0
        {
            // Use the new value.
            if ReallocSplStr(
                &raw mut (*pJob).pwszStatus,
                (*pJobInfo).pStatus as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_3,
                        __dbch_3,
                        b"win32ss/printing/providers/localspl/jobs.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"_LocalSetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                        894 as ::core::ffi::c_int,
                        b"ReallocSplStr failed, last error is %lu!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        GetLastError(),
                    );
                }
                break 'Cleanup;
            }
        }

        // Check if the user name has changed. An empty string is permitted here!
        if _EqualStrings(
            (*pJob).pwszUserName as PCWSTR,
            (*pJobInfo).pUserName as PCWSTR,
        ) == 0
        {
            // The new user name doesn't need to exist, so no additional verification is required.

            // Use the new value.
            if ReallocSplStr(
                &raw mut (*pJob).pwszUserName,
                (*pJobInfo).pUserName as PCWSTR,
            ) == 0
            {
                dwErrorCode = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_4,
                        __dbch_4,
                        b"win32ss/printing/providers/localspl/jobs.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"_LocalSetJobLevel2\0" as *const u8 as *const ::core::ffi::c_char,
                        908 as ::core::ffi::c_int,
                        b"ReallocSplStr failed, last error is %lu!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        GetLastError(),
                    );
                }
                break 'Cleanup;
            }
        }

        // Check if the priority has changed.
        if (*pJob).dwPriority != (*pJobInfo).Priority
            && (IS_VALID_PRIORITY!((*pJobInfo).Priority) != false)
        {
            // Set the new priority.
            (*pJob).dwPriority = (*pJobInfo).Priority;

            // Remove and reinsert the job in the Printer's Job List.
            // The Compare function will be used to find the right position now considering the new priority.
            DeleteElementSkiplist(
                &raw mut (*(*pJob).pPrinter).JobList,
                pJob as PVOID,
            );
            InsertElementSkiplist(
                &raw mut (*(*pJob).pPrinter).JobList,
                pJob as PVOID,
            );
        }

        // Check if the status flags have changed.
        if (*pJob).dwStatus != (*pJobInfo).Status {
            // Only add status flags that make sense.
            if (*pJobInfo).Status & JOB_STATUS_PAUSED as DWORD != 0 {
                (*pJob).dwStatus |= JOB_STATUS_PAUSED as DWORD;
            }

            if (*pJobInfo).Status & JOB_STATUS_ERROR as DWORD != 0 {
                (*pJob).dwStatus |= JOB_STATUS_ERROR as DWORD;
            }

            if (*pJobInfo).Status & JOB_STATUS_OFFLINE as DWORD != 0 {
                (*pJob).dwStatus |= JOB_STATUS_OFFLINE as DWORD;
            }

            if (*pJobInfo).Status & JOB_STATUS_PAPEROUT as DWORD != 0 {
                (*pJob).dwStatus |= JOB_STATUS_PAPEROUT as DWORD;
            }
        }

        dwErrorCode = ERROR_SUCCESS as DWORD;
    }

    return dwErrorCode;
}
#[no_mangle]
pub unsafe extern "system" fn LocalSetJob(mut hPrinter: HANDLE, mut JobId: DWORD, mut Level: DWORD, mut pJobInfo: PBYTE, mut Command: DWORD) -> BOOL {
    let mut dwErrorCode: DWORD = ERROR_SUCCESS as DWORD;
    let mut pHandle: PLOCAL_HANDLE = core::ptr::null_mut::<_LOCAL_HANDLE>();
    let mut pJob: PLOCAL_JOB = core::ptr::null_mut::<_LOCAL_JOB>();
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut::<_LOCAL_PRINTER_HANDLE>();
    let mut wszFullPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/jobs.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalSetJob\0" as *const u8 as *const ::core::ffi::c_char,
            957 as ::core::ffi::c_int,
            b"LocalSetJob(%p, %lu, %lu, %p, %lu)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hPrinter,
            JobId,
            Level,
            pJobInfo,
            Command,
        );
    }
    pHandle = hPrinter as PLOCAL_HANDLE;
    if pHandle.is_null()
        || (*pHandle).HandleType as ::core::ffi::c_uint
            != HandleType_Printer as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        dwErrorCode = ERROR_INVALID_HANDLE as DWORD;
    } else {
        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;
        pJob = LookupElementSkiplist(
            &raw mut GlobalJobList,
            &raw mut JobId as PVOID,
            core::ptr::null_mut::<DWORD>(),
        ) as PLOCAL_JOB;
        if pJob.is_null() || (*pJob).pPrinter != (*pPrinterHandle).pPrinter {
            dwErrorCode = ERROR_INVALID_PARAMETER as DWORD;
        } else {
            if Level == 1 as DWORD {
                dwErrorCode = _LocalSetJobLevel1(pPrinterHandle, pJob, pJobInfo as PJOB_INFO_1W);
            } else if Level == 2 as DWORD {
                dwErrorCode = _LocalSetJobLevel2(pPrinterHandle, pJob, pJobInfo as PJOB_INFO_2W);
            }
            if !(dwErrorCode != ERROR_SUCCESS as DWORD) {
                if (*(*pPrinterHandle).pPrinter).dwAttributes
                    & PRINTER_ATTRIBUTE_DIRECT as DWORD
                    == 0
                {
                    GetJobFilePath(
                        RUSTCONV_WSTR_1652C1041F58.as_ptr(),
                        JobId,
                        &raw mut wszFullPath as PWSTR,
                    );
                    WriteJobShadowFile(&raw mut wszFullPath as PWSTR, pJob);
                }
                if Command != 0 {
                    if Command == JOB_CONTROL_SENT_TO_PRINTER as DWORD {
                        if !(*pJob).hPrintProcessor.is_null() {
                            (*(*pJob).pPrintProcessor)
                                .pfnControlPrintProcessor
                                .unwrap_unchecked()(
                                (*pJob).hPrintProcessor,
                                JOB_CONTROL_CANCEL as DWORD,
                            );
                        }
                        FreeJob(pJob);
                    } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl_0,
                            __dbch_0,
                            b"win32ss/printing/providers/localspl/jobs.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"LocalSetJob\0" as *const u8 as *const ::core::ffi::c_char,
                            1012 as ::core::ffi::c_int,
                            b"Unimplemented SetJob Command: %lu!\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            Command,
                        );
                    }
                }
            }
        }
    }
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS as DWORD) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalEnumJobs(hPrinter: HANDLE, FirstJob: DWORD, NoJobs: DWORD, Level: DWORD, mut pStart: PBYTE, cbBuf: DWORD, pcbNeeded: LPDWORD, pcReturned: LPDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut i: DWORD;
    let mut pEnd: PBYTE;
    let mut pHandle: PLOCAL_HANDLE;
    let mut pJob: PLOCAL_JOB;
    let mut pFirstJobNode: PSKIPLIST_NODE;
    let mut pNode: PSKIPLIST_NODE;
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE;

    'Cleanup: {
        // Check if this is a printer handle.
        pHandle = hPrinter as PLOCAL_HANDLE;
        if (*pHandle).HandleType != HandleType_Printer {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'Cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // Check the level.
        if Level > 2 {
            dwErrorCode = ERROR_INVALID_LEVEL;
            break 'Cleanup;
        }

        // Begin counting.
        *pcbNeeded = 0;
        *pcReturned = 0;

        // Lookup the node of the first job requested by the caller in the Printer's Job List.
        pFirstJobNode = LookupNodeByIndexSkiplist(
            core::ptr::addr_of_mut!((*(*pPrinterHandle).pPrinter).JobList),
            FirstJob,
        );

        // Count the required buffer size and the number of jobs.
        i = 0;
        pNode = pFirstJobNode;

        while i < NoJobs && !pNode.is_null() {
            pJob = (*pNode).Element as PLOCAL_JOB;

            if Level == 1 {
                _LocalGetJobLevel1(
                    pJob,
                    core::ptr::null_mut::<PJOB_INFO_1W>(),
                    core::ptr::null_mut::<PBYTE>(),
                    pcbNeeded,
                );
            } else if Level == 2 {
                _LocalGetJobLevel2(
                    pJob,
                    core::ptr::null_mut::<PJOB_INFO_2W>(),
                    core::ptr::null_mut::<PBYTE>(),
                    pcbNeeded,
                );
            }

            // We stop either when there are no more jobs in the list or when the caller didn't request more, whatever comes first.
            i = i.wrapping_add(1);
            pNode = (*pNode).Next[0 as usize] as PSKIPLIST_NODE;
        }

        // Check if the supplied buffer is large enough.
        if cbBuf < *pcbNeeded {
            dwErrorCode = ERROR_INSUFFICIENT_BUFFER;
            break 'Cleanup;
        }

        // Copy over the Job information.
        i = 0;
        pNode = pFirstJobNode;
        pEnd = pStart.offset(*pcbNeeded as isize);

        while i < NoJobs && !pNode.is_null() {
            pJob = (*pNode).Element as PLOCAL_JOB;

            if Level == 1 {
                _LocalGetJobLevel1(
                    pJob,
                    core::ptr::addr_of_mut!(pStart) as *mut PJOB_INFO_1W,
                    core::ptr::addr_of_mut!(pEnd),
                    core::ptr::null_mut::<DWORD>(),
                );
            } else if Level == 2 {
                _LocalGetJobLevel2(
                    pJob,
                    core::ptr::addr_of_mut!(pStart) as *mut PJOB_INFO_2W,
                    core::ptr::addr_of_mut!(pEnd),
                    core::ptr::null_mut::<DWORD>(),
                );
            }

            // We stop either when there are no more jobs in the list or when the caller didn't request more, whatever comes first.
            i = i.wrapping_add(1);
            pNode = (*pNode).Next[0 as usize] as PSKIPLIST_NODE;
        }

        *pcReturned = i;
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LocalScheduleJob(hPrinter: HANDLE, mut dwJobID: DWORD) -> BOOL {
    let mut dwAttributes: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut pJob: PLOCAL_JOB = core::ptr::null_mut();
    let mut pHandle: PLOCAL_HANDLE = hPrinter as PLOCAL_HANDLE;
    let mut pPrinterHandle: PLOCAL_PRINTER_HANDLE = core::ptr::null_mut();
    let mut wszFullPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/printing/providers/localspl/jobs.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LocalScheduleJob\0" as *const u8 as *const ::core::ffi::c_char,
            1122 as ::core::ffi::c_int,
            b"LocalScheduleJob(%p, %lu)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hPrinter,
            dwJobID,
        );
    }

    'cleanup: {
        // Check if this is a printer handle.
        if (*pHandle).HandleType as ::core::ffi::c_uint
            != HandleType_Printer as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            dwErrorCode = ERROR_INVALID_HANDLE;
            break 'cleanup;
        }

        pPrinterHandle = (*pHandle).pSpecificHandle as PLOCAL_PRINTER_HANDLE;

        // Check if the Job ID is valid.
        pJob = LookupElementSkiplist(
            core::ptr::addr_of_mut!(GlobalJobList),
            core::ptr::addr_of_mut!(dwJobID) as PVOID,
            core::ptr::null_mut::<DWORD>(),
        ) as PLOCAL_JOB;
        if pJob.is_null() || (*pJob).pPrinter != (*pPrinterHandle).pPrinter {
            dwErrorCode = ERROR_INVALID_PARAMETER;
            break 'cleanup;
        }

        // Check if this Job was started with AddJob.
        if (*pJob).bAddedJob() == 0 {
            dwErrorCode = ERROR_SPL_NO_ADDJOB;
            break 'cleanup;
        }

        // Construct the full path to the spool file.
        GetJobFilePath(
            RUSTCONV_WSTR_D575D22E81A4.as_ptr(),
            dwJobID,
            core::ptr::addr_of_mut!(wszFullPath) as PWSTR,
        );

        // Check if it exists.
        dwAttributes = GetFileAttributesW(
            core::ptr::addr_of_mut!(wszFullPath) as *mut WCHAR as LPCWSTR,
        );
        if dwAttributes == INVALID_FILE_ATTRIBUTES
            || dwAttributes & FILE_ATTRIBUTE_DIRECTORY != 0
        {
            dwErrorCode = ERROR_SPOOL_FILE_NOT_FOUND;
            break 'cleanup;
        }

        // Spooling is finished at this point.
        (*pJob).dwStatus &= !JOB_STATUS_SPOOLING;

        // Write the job data into the shadow file.
        wcscpy(
            wcsrchr(
                core::ptr::addr_of_mut!(wszFullPath) as *mut WCHAR,
                0x002Eu16,
            ),
            RUSTCONV_WSTR_B2FCAC26DCDA.as_ptr(),
        );
        WriteJobShadowFile(core::ptr::addr_of_mut!(wszFullPath) as PWSTR, pJob);

        // Create the thread for performing the printing process.
        hThread = CreateThread(
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            0,
            core::mem::transmute::<
                Option<unsafe extern "system" fn(PLOCAL_JOB) -> DWORD>,
                LPTHREAD_START_ROUTINE,
            >(Some(
                PrintingThreadProc as unsafe extern "system" fn(PLOCAL_JOB) -> DWORD,
            )),
            pJob as PVOID,
            0,
            core::ptr::null_mut::<DWORD>(),
        );
        if hThread.is_null() {
            dwErrorCode = GetLastError();
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LocalScheduleJob\0" as *const u8 as *const ::core::ffi::c_char,
                    1171 as ::core::ffi::c_int,
                    b"CreateThread failed with error %lu!\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    dwErrorCode,
                );
            }
            break 'cleanup;
        }

        // We don't need the thread handle. Keeping it open blocks the thread from terminating.
        CloseHandle(hThread);

        // ScheduleJob has done its job. The rest happens inside the thread.
        dwErrorCode = ERROR_SUCCESS;
    }

    // Cleanup:
    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ReadJobShadowFile(pwszFilePath: PCWSTR) -> PLOCAL_JOB {
    let mut cbFileSize: DWORD = 0;
    let mut cbRead: DWORD = 0;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut pJob: PLOCAL_JOB = core::ptr::null_mut();
    let mut pReturnValue: PLOCAL_JOB = core::ptr::null_mut();
    let mut pPrinter: PLOCAL_PRINTER = core::ptr::null_mut();
    let mut pPrintProcessor: PLOCAL_PRINT_PROCESSOR = core::ptr::null_mut();
    let mut pShadowFile: PSHD_HEADER = core::ptr::null_mut();
    let mut pwszPrinterName: PWSTR = core::ptr::null_mut();
    let mut pwszPrintProcessor: PWSTR = core::ptr::null_mut();

    'cleanup: {
        // Try to open the file.
        hFile = CreateFileW(
            pwszFilePath as LPCWSTR,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const i8,
                    b"ReadJobShadowFile\0" as *const u8 as *const i8,
                    1206 as i32,
                    b"CreateFileW failed with error %lu for file \"%S\"!\n\0" as *const u8 as *const i8,
                    GetLastError(),
                    pwszFilePath,
                );
            }
            break 'cleanup;
        }

        // Get its file size (small enough for a single DWORD) and allocate memory for all of it.
        cbFileSize = GetFileSize(hFile, core::ptr::null_mut());
        pShadowFile = DllAllocSplMem(cbFileSize) as PSHD_HEADER;
        if pShadowFile.is_null() {
            if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const i8,
                    b"ReadJobShadowFile\0" as *const u8 as *const i8,
                    1215 as i32,
                    b"DllAllocSplMem failed for file \"%S\"!\n\0" as *const u8 as *const i8,
                    pwszFilePath,
                );
            }
            break 'cleanup;
        }

        // Read the entire file.
        if ReadFile(
            hFile,
            pShadowFile as PVOID,
            cbFileSize,
            core::ptr::addr_of_mut!(cbRead),
            core::ptr::null_mut(),
        ) == 0
        {
            if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const i8,
                    b"ReadJobShadowFile\0" as *const u8 as *const i8,
                    1222 as i32,
                    b"ReadFile failed with error %lu for file \"%S\"!\n\0" as *const u8 as *const i8,
                    GetLastError(),
                    pwszFilePath,
                );
            }
            break 'cleanup;
        }

        // Check signature and header size.
        if (*pShadowFile).dwSignature != SHD_WIN2003_SIGNATURE
            || (*pShadowFile).cbHeader as usize != core::mem::size_of::<SHD_HEADER>() as usize
        {
            if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const i8,
                    b"ReadJobShadowFile\0" as *const u8 as *const i8,
                    1229 as i32,
                    b"Signature or Header Size mismatch for file \"%S\"!\n\0" as *const u8 as *const i8,
                    pwszFilePath,
                );
            }
            break 'cleanup;
        }

        // Retrieve the associated printer from the list.
        pwszPrinterName = (pShadowFile as ULONG_PTR)
            .wrapping_add((*pShadowFile).offPrinterName as ULONG_PTR) as PWSTR;
        pPrinter = LookupElementSkiplist(
            core::ptr::addr_of_mut!(PrinterList),
            core::ptr::addr_of_mut!(pwszPrinterName) as PVOID,
            core::ptr::null_mut(),
        ) as PLOCAL_PRINTER;
        if pPrinter.is_null() {
            if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_4,
                    __dbch_4,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const i8,
                    b"ReadJobShadowFile\0" as *const u8 as *const i8,
                    1238 as i32,
                    b"Shadow file \"%S\" references a non-existing printer \"%S\"!\n\0" as *const u8 as *const i8,
                    pwszFilePath,
                    pwszPrinterName,
                );
            }
            break 'cleanup;
        }

        // Retrieve the associated Print Processor from the list.
        pwszPrintProcessor = (pShadowFile as ULONG_PTR)
            .wrapping_add((*pShadowFile).offPrintProcessor as ULONG_PTR) as PWSTR;
        pPrintProcessor = FindPrintProcessor(pwszPrintProcessor as PCWSTR);
        if pPrintProcessor.is_null() {
            if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
                let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_5: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_5,
                    __dbch_5,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const i8,
                    b"ReadJobShadowFile\0" as *const u8 as *const i8,
                    1247 as i32,
                    b"Shadow file \"%S\" references a non-existing Print Processor \"%S\"!\n\0" as *const u8 as *const i8,
                    pwszFilePath,
                    pwszPrintProcessor,
                );
            }
            break 'cleanup;
        }

        // Create a new job structure and copy over the relevant fields.
        pJob = DllAllocSplMem(core::mem::size_of::<LOCAL_JOB>() as DWORD) as PLOCAL_JOB;
        if pJob.is_null() {
            if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32)) != 0 {
                let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_6: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_6,
                    __dbch_6,
                    b"win32ss/printing/providers/localspl/jobs.c\0"
                        as *const u8 as *const i8,
                    b"ReadJobShadowFile\0" as *const u8 as *const i8,
                    1255 as i32,
                    b"DllAllocSplMem failed for file \"%S\"!\n\0" as *const u8 as *const i8,
                    pwszFilePath,
                );
            }
            break 'cleanup;
        }

        (*pJob).dwJobID = (*pShadowFile).dwJobID;
        (*pJob).dwPriority = (*pShadowFile).dwPriority;
        (*pJob).dwStartTime = (*pShadowFile).dwStartTime;
        (*pJob).dwTotalPages = (*pShadowFile).dwTotalPages;
        (*pJob).dwUntilTime = (*pShadowFile).dwUntilTime;
        (*pJob).pPrinter = pPrinter;
        (*pJob).pPrintProcessor = pPrintProcessor;
        (*pJob).pDevMode = DuplicateDevMode(
            (pShadowFile as ULONG_PTR).wrapping_add((*pShadowFile).offDevMode as ULONG_PTR) as PDEVMODEW,
        );
        (*pJob).pwszDatatype = AllocSplStr(
            (pShadowFile as ULONG_PTR).wrapping_add((*pShadowFile).offDatatype as ULONG_PTR) as PCWSTR,
        );
        (*pJob).pwszMachineName = AllocSplStr(
            (pShadowFile as ULONG_PTR).wrapping_add((*pShadowFile).offMachineName as ULONG_PTR) as PCWSTR,
        );
        CopyMemory!(
            core::ptr::addr_of_mut!((*pJob).stSubmitted) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*pShadowFile).stSubmitted) as *const core::ffi::c_void,
            core::mem::size_of::<SYSTEMTIME>() as size_t,
        );

        // Copy the optional values.
        if (*pShadowFile).offDocumentName != 0 {
            (*pJob).pwszDocumentName = AllocSplStr(
                (pShadowFile as ULONG_PTR).wrapping_add((*pShadowFile).offDocumentName as ULONG_PTR) as PCWSTR,
            );
        }

        if (*pShadowFile).offNotifyName != 0 {
            (*pJob).pwszNotifyName = AllocSplStr(
                (pShadowFile as ULONG_PTR).wrapping_add((*pShadowFile).offNotifyName as ULONG_PTR) as PCWSTR,
            );
        }

        if (*pShadowFile).offPrintProcessorParameters != 0 {
            (*pJob).pwszPrintProcessorParameters = AllocSplStr(
                (pShadowFile as ULONG_PTR)
                    .wrapping_add((*pShadowFile).offPrintProcessorParameters as ULONG_PTR) as PCWSTR,
            );
        }

        if (*pShadowFile).offUserName != 0 {
            (*pJob).pwszUserName = AllocSplStr(
                (pShadowFile as ULONG_PTR).wrapping_add((*pShadowFile).offUserName as ULONG_PTR) as PCWSTR,
            );
        }

        // Jobs read from shadow files were always added using AddJob.
        (*pJob).set_bAddedJob(TRUE as BOOL);

        pReturnValue = pJob;
    }

    // Cleanup:
    if !pShadowFile.is_null() {
        DllFreeSplMem(pShadowFile as PVOID);
    }

    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }

    return pReturnValue;
}
#[no_mangle]
pub unsafe extern "C" fn WriteJobShadowFile(pwszFilePath: PWSTR, pJob: PLOCAL_JOB) -> BOOL {
    let mut bReturnValue: BOOL = FALSE as BOOL;
    let mut cbDatatype: DWORD = (wcslen((*pJob).pwszDatatype) as DWORD)
        .wrapping_add(1)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    let mut cbDevMode: DWORD = ((*(*pJob).pDevMode).dmSize as DWORD)
        .wrapping_add((*(*pJob).pDevMode).dmDriverExtra as DWORD);
    let mut cbDocumentName: DWORD = 0;
    let mut cbFileSize: DWORD = 0;
    let mut cbMachineName: DWORD = (wcslen((*pJob).pwszMachineName) as DWORD)
        .wrapping_add(1)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    let mut cbNotifyName: DWORD = 0;
    let mut cbPrinterDriver: DWORD =
        (wcslen((*(*pJob).pPrinter).pwszPrinterDriver) as DWORD)
            .wrapping_add(1)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    let mut cbPrinterName: DWORD =
        (wcslen((*(*pJob).pPrinter).pwszPrinterName) as DWORD)
            .wrapping_add(1)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    let mut cbPrintProcessor: DWORD =
        (wcslen((*(*pJob).pPrintProcessor).pwszName) as DWORD)
            .wrapping_add(1)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    let mut cbPrintProcessorParameters: DWORD = 0;
    let mut cbUserName: DWORD = 0;
    let mut cbWritten: DWORD = 0;
    let mut dwCurrentOffset: DWORD = 0;
    let mut hSHDFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut hSPLFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut pShadowFile: PSHD_HEADER = core::ptr::null_mut();

    'cleanup: {
        // Try to open the SHD file.
        hSHDFile = CreateFileW(
            pwszFilePath,
            GENERIC_WRITE,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            0,
            core::ptr::null_mut(),
        );
        if hSHDFile == INVALID_HANDLE_VALUE {
            break 'cleanup;
        }

        // Calculate the lengths of the optional values and the total size of the shadow file.
        if !(*pJob).pwszDocumentName.is_null() {
            cbDocumentName = (wcslen((*pJob).pwszDocumentName) as DWORD)
                .wrapping_add(1)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        }

        if !(*pJob).pwszNotifyName.is_null() {
            cbNotifyName = (wcslen((*pJob).pwszNotifyName) as DWORD)
                .wrapping_add(1)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        }

        if !(*pJob).pwszPrintProcessorParameters.is_null() {
            cbPrintProcessorParameters =
                (wcslen((*pJob).pwszPrintProcessorParameters) as DWORD)
                    .wrapping_add(1)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        }

        if !(*pJob).pwszUserName.is_null() {
            cbUserName = (wcslen((*pJob).pwszUserName) as DWORD)
                .wrapping_add(1)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        }

        cbFileSize = (core::mem::size_of::<SHD_HEADER>() as DWORD)
            .wrapping_add(cbDatatype)
            .wrapping_add(cbDocumentName)
            .wrapping_add(cbDevMode)
            .wrapping_add(cbMachineName)
            .wrapping_add(cbNotifyName)
            .wrapping_add(cbPrinterDriver)
            .wrapping_add(cbPrinterName)
            .wrapping_add(cbPrintProcessor)
            .wrapping_add(cbPrintProcessorParameters)
            .wrapping_add(cbUserName);

        // Allocate memory for it.
        pShadowFile = DllAllocSplMem(cbFileSize) as PSHD_HEADER;
        if pShadowFile.is_null() {
            break 'cleanup;
        }

        // Fill out the shadow file header information.
        (*pShadowFile).dwSignature = SHD_WIN2003_SIGNATURE;
        (*pShadowFile).cbHeader = core::mem::size_of::<SHD_HEADER>() as DWORD;

        // Copy the values.
        (*pShadowFile).dwJobID = (*pJob).dwJobID;
        (*pShadowFile).dwPriority = (*pJob).dwPriority;
        (*pShadowFile).dwStartTime = (*pJob).dwStartTime;
        (*pShadowFile).dwTotalPages = (*pJob).dwTotalPages;
        (*pShadowFile).dwUntilTime = (*pJob).dwUntilTime;
        CopyMemory!(
            core::ptr::addr_of_mut!((*pShadowFile).stSubmitted) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*pJob).stSubmitted) as *const core::ffi::c_void,
            core::mem::size_of::<SYSTEMTIME>() as usize,
        );

        // Determine the file size of the .SPL file
        wcscpy(
            wcsrchr(pwszFilePath, 0x002Eu16),
            RUSTCONV_WSTR_17D32AC93B88.as_ptr(),
        );
        hSPLFile = CreateFileW(
            pwszFilePath,
            GENERIC_READ,
            (FILE_SHARE_READ | FILE_SHARE_WRITE),
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if hSPLFile != INVALID_HANDLE_VALUE {
            (*pShadowFile).dwSPLSize =
                GetFileSize(hSPLFile, core::ptr::null_mut());
        }

        // Add the extra values that are stored as offsets in the shadow file.
        // The first value begins right after the shadow file header.
        dwCurrentOffset = core::mem::size_of::<SHD_HEADER>() as DWORD;

        CopyMemory!(
            (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
            (*pJob).pwszDatatype as *const core::ffi::c_void,
            cbDatatype as usize,
        );
        (*pShadowFile).offDatatype = dwCurrentOffset;
        dwCurrentOffset = dwCurrentOffset.wrapping_add(cbDatatype);

        CopyMemory!(
            (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
            (*pJob).pDevMode as *const core::ffi::c_void,
            cbDevMode as usize,
        );
        (*pShadowFile).offDevMode = dwCurrentOffset;
        dwCurrentOffset = dwCurrentOffset.wrapping_add(cbDevMode);

        // offDriverName is only written, but automatically determined through offPrinterName when reading.
        CopyMemory!(
            (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
            (*(*pJob).pPrinter).pwszPrinterDriver as *const core::ffi::c_void,
            cbPrinterDriver as usize,
        );
        (*pShadowFile).offDriverName = dwCurrentOffset;
        dwCurrentOffset = dwCurrentOffset.wrapping_add(cbPrinterDriver);

        CopyMemory!(
            (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
            (*pJob).pwszMachineName as *const core::ffi::c_void,
            cbMachineName as usize,
        );
        (*pShadowFile).offMachineName = dwCurrentOffset;
        dwCurrentOffset = dwCurrentOffset.wrapping_add(cbMachineName);

        CopyMemory!(
            (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
            (*(*pJob).pPrinter).pwszPrinterName as *const core::ffi::c_void,
            cbPrinterName as usize,
        );
        (*pShadowFile).offPrinterName = dwCurrentOffset;
        dwCurrentOffset = dwCurrentOffset.wrapping_add(cbPrinterName);

        CopyMemory!(
            (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
            (*(*pJob).pPrintProcessor).pwszName as *const core::ffi::c_void,
            cbPrintProcessor as usize,
        );
        (*pShadowFile).offPrintProcessor = dwCurrentOffset;
        dwCurrentOffset = dwCurrentOffset.wrapping_add(cbPrintProcessor);

        // Copy the optional values.
        if cbDocumentName != 0 {
            CopyMemory!(
                (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
                (*pJob).pwszDocumentName as *const core::ffi::c_void,
                cbDocumentName as usize,
            );
            (*pShadowFile).offDocumentName = dwCurrentOffset;
            dwCurrentOffset = dwCurrentOffset.wrapping_add(cbDocumentName);
        }

        if cbNotifyName != 0 {
            CopyMemory!(
                (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
                (*pJob).pwszNotifyName as *const core::ffi::c_void,
                cbNotifyName as usize,
            );
            (*pShadowFile).offNotifyName = dwCurrentOffset;
            dwCurrentOffset = dwCurrentOffset.wrapping_add(cbNotifyName);
        }

        if cbPrintProcessorParameters != 0 {
            CopyMemory!(
                (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
                (*pJob).pwszPrintProcessorParameters as *const core::ffi::c_void,
                cbPrintProcessorParameters as usize,
            );
            (*pShadowFile).offPrintProcessorParameters = dwCurrentOffset;
            dwCurrentOffset = dwCurrentOffset.wrapping_add(cbPrintProcessorParameters);
        }

        if cbUserName != 0 {
            CopyMemory!(
                (pShadowFile as PBYTE).offset(dwCurrentOffset as isize) as *mut core::ffi::c_void,
                (*pJob).pwszUserName as *const core::ffi::c_void,
                cbUserName as usize,
            );
            (*pShadowFile).offUserName = dwCurrentOffset;
            dwCurrentOffset = dwCurrentOffset.wrapping_add(cbUserName);
        }

        // Write the file.
        if WriteFile(
            hSHDFile,
            pShadowFile as LPCVOID,
            cbFileSize,
            core::ptr::addr_of_mut!(cbWritten),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'cleanup;
        }

        bReturnValue = TRUE as BOOL;
    }

    // Cleanup:
    if !pShadowFile.is_null() {
        DllFreeSplMem(pShadowFile as PVOID);
    }

    if hSHDFile != INVALID_HANDLE_VALUE {
        CloseHandle(hSHDFile);
    }

    if hSPLFile != INVALID_HANDLE_VALUE {
        CloseHandle(hSPLFile);
    }

    return bReturnValue;
}
#[no_mangle]
pub unsafe extern "C" fn FreeJob(pJob: PLOCAL_JOB) {
    let mut pwszSHDFile: PWSTR;

    // Remove the Job from both Job Lists.
    DeleteElementSkiplist(core::ptr::addr_of_mut!((*(*pJob).pPrinter).JobList), pJob as PVOID);
    DeleteElementSkiplist(core::ptr::addr_of_mut!(GlobalJobList), pJob as PVOID);

    // Try to delete the corresponding .SHD file.
    pwszSHDFile = DllAllocSplMem(GetJobFilePath(
        RUSTCONV_WSTR_1652C1041F58.as_ptr(),
        0,
        core::ptr::null_mut::<WCHAR>(),
    )) as PWSTR;
    if !pwszSHDFile.is_null()
        && GetJobFilePath(RUSTCONV_WSTR_1652C1041F58.as_ptr(), (*pJob).dwJobID, pwszSHDFile) != 0
    {
        DeleteFileW(pwszSHDFile as LPCWSTR);
    }

    // Free memory for the mandatory fields.
    DllFreeSplMem((*pJob).pDevMode as PVOID);
    DllFreeSplStr((*pJob).pwszDatatype);
    DllFreeSplStr((*pJob).pwszDocumentName);
    DllFreeSplStr((*pJob).pwszMachineName);
    DllFreeSplStr((*pJob).pwszNotifyName);
    DllFreeSplStr((*pJob).pwszUserName);

    // Free memory for the optional fields if they are present.
    if !(*pJob).pwszOutputFile.is_null() {
        DllFreeSplStr((*pJob).pwszOutputFile);
    }

    if !(*pJob).pwszPrintProcessorParameters.is_null() {
        DllFreeSplStr((*pJob).pwszPrintProcessorParameters);
    }

    if !(*pJob).pwszStatus.is_null() {
        DllFreeSplStr((*pJob).pwszStatus);
    }

    // Finally free the job structure itself.
    DllFreeSplMem(pJob as PVOID);
}
