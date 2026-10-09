use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::{RtlCopyMemory, RtlZeroMemory, InitializeObjectAttributes};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
#[derive(PartialEq)]
pub enum _FIND_DATA_TYPE {
    FindFile = 1,
    FindStream = 2,
}
pub type FIND_DATA_TYPE = _FIND_DATA_TYPE;
use _FIND_DATA_TYPE::{FindFile, FindStream};

extern "system" {
    fn Basep8BitStringToStaticUnicodeString(String: LPCSTR) -> PUNICODE_STRING;
}

// BasepUnicodeStringTo8BitString is a PRTL_CONVERT_STRINGA fn-pointer GLOBAL in C
// (utils.c), called directly — declare it as a data symbol (no @N), not a function.
extern "C" {
    static BasepUnicodeStringTo8BitString:
        unsafe extern "system" fn(PANSI_STRING, PCUNICODE_STRING, BOOLEAN) -> NTSTATUS;
}

// RtlStringCbCopyNW is FORCEINLINE in ntstrsafe.h (no export) → use the crate's
// faithful Rust implementation in the ntstrsafe helper module.
use crate::ntstrsafe::RtlStringCbCopyNW;

#[repr(C)]
#[derive(Copy, Clone)]
pub union _DIR_INFORMATION {
    pub DirInfo: PVOID,
    pub FullDirInfo: PFILE_FULL_DIR_INFORMATION,
    pub BothDirInfo: PFILE_BOTH_DIR_INFORMATION,
}
pub type DIR_INFORMATION = _DIR_INFORMATION;

#[repr(C)]
pub struct _FIND_FILE_DATA {
    pub Handle: HANDLE,
    pub InfoLevel: FINDEX_INFO_LEVELS,
    pub SearchOp: FINDEX_SEARCH_OPS,
    pub HasMoreData: BOOLEAN,
    pub NextDirInfo: DIR_INFORMATION,
    pub Buffer: [BYTE; FIND_DATA_SIZE],
}
pub type FIND_FILE_DATA = _FIND_FILE_DATA;
pub type PFIND_FILE_DATA = *mut _FIND_FILE_DATA;

#[repr(C)]
pub struct _FIND_STREAM_DATA {
    pub InfoLevel: STREAM_INFO_LEVELS,
    pub FileStreamInfo: PFILE_STREAM_INFORMATION,
    pub CurrentInfo: PFILE_STREAM_INFORMATION,
}
pub type FIND_STREAM_DATA = _FIND_STREAM_DATA;
pub type PFIND_STREAM_DATA = *mut _FIND_STREAM_DATA;

#[repr(C)]
pub struct _FIND_DATA_HANDLE {
    pub Type: FIND_DATA_TYPE,
    pub Lock: RTL_CRITICAL_SECTION,
    pub u: _FIND_DATA_HANDLE_u,
}
#[repr(C)]
pub union _FIND_DATA_HANDLE_u {
    pub FindFileData: PFIND_FILE_DATA,
    pub FindStreamData: PFIND_STREAM_DATA,
}
pub type FIND_DATA_HANDLE = _FIND_DATA_HANDLE;
pub type PFIND_DATA_HANDLE = *mut _FIND_DATA_HANDLE;

pub const FIND_DATA_SIZE: usize = 0x4000;
pub const FIND_DEVICE_HANDLE: HANDLE = 0x1 as HANDLE;
unsafe fn CopyDeviceFindData(
    lpFindFileData: LPWIN32_FIND_DATAW,
    lpFileName: LPCWSTR,
    DeviceNameInfo: ULONG,
) {
    let DeviceName: LPCWSTR;
    let Length: SIZE_T;

    // _SEH2_TRY
    {
        /* DeviceNameInfo == { USHORT Offset; USHORT Length } */
        Length = (DeviceNameInfo & 0xFFFF) as SIZE_T;
        DeviceName = ((lpFileName as ULONG_PTR) + ((DeviceNameInfo >> 16) & 0xFFFF) as ULONG_PTR) as LPCWSTR;

        /* Return the data */
        RtlZeroMemory(lpFindFileData as *mut _, core::mem::size_of::<WIN32_FIND_DATAW>() as SIZE_T);
        (*lpFindFileData).dwFileAttributes = FILE_ATTRIBUTE_ARCHIVE;
        RtlStringCbCopyNW(
            (*lpFindFileData).cFileName.as_mut_ptr(),
            core::mem::size_of_val(&(*lpFindFileData).cFileName),
            DeviceName,
            Length as usize,
        );
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // Exception handler - empty
    }
    // _SEH2_END

    return;
}
unsafe fn CopyFindData(
    lpFindFileData: LPWIN32_FIND_DATAW,
    fInfoLevelId: FINDEX_INFO_LEVELS,
    DirInfo: DIR_INFORMATION,
) {
    // ULARGE_INTEGER_2_FILETIME macro expanded inline
    macro_rules! ULARGE_INTEGER_2_FILETIME {
        ($ft:expr, $ul:expr) => {
            ($ft).dwHighDateTime = ($ul).u.HighPart as DWORD;
            ($ft).dwLowDateTime = ($ul).u.LowPart;
        };
    }

    // _SEH2_TRY block
    {
        RtlZeroMemory(
            lpFindFileData as *mut _,
            core::mem::size_of::<WIN32_FIND_DATAW>() as SIZE_T,
        );

        (*lpFindFileData).dwFileAttributes = (*DirInfo.FullDirInfo).FileAttributes;

        ULARGE_INTEGER_2_FILETIME!(
            (*lpFindFileData).ftCreationTime,
            (*DirInfo.FullDirInfo).CreationTime
        );
        ULARGE_INTEGER_2_FILETIME!(
            (*lpFindFileData).ftLastAccessTime,
            (*DirInfo.FullDirInfo).LastAccessTime
        );
        ULARGE_INTEGER_2_FILETIME!(
            (*lpFindFileData).ftLastWriteTime,
            (*DirInfo.FullDirInfo).LastWriteTime
        );

        (*lpFindFileData).nFileSizeHigh = (*DirInfo.FullDirInfo).EndOfFile.u.HighPart as DWORD;
        (*lpFindFileData).nFileSizeLow = (*DirInfo.FullDirInfo).EndOfFile.u.LowPart;

        if (*DirInfo.FullDirInfo).FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            (*lpFindFileData).dwReserved0 = (*DirInfo.FullDirInfo).EaSize;
        } else {
            (*lpFindFileData).dwReserved0 = 0;
        }

        (*lpFindFileData).dwReserved1 = 0;

        if fInfoLevelId == FindExInfoStandard {
            RtlStringCbCopyNW(
                (*lpFindFileData).cFileName.as_mut_ptr(),
                core::mem::size_of_val(&(*lpFindFileData).cFileName),
                (*DirInfo.BothDirInfo).FileName.as_ptr(),
                (*DirInfo.BothDirInfo).FileNameLength as usize,
            );

            RtlStringCbCopyNW(
                (*lpFindFileData).cAlternateFileName.as_mut_ptr(),
                core::mem::size_of_val(&(*lpFindFileData).cAlternateFileName),
                (*DirInfo.BothDirInfo).ShortName.as_ptr(),
                (*DirInfo.BothDirInfo).ShortNameLength as usize,
            );
        } else if fInfoLevelId == FindExInfoBasic {
            RtlStringCbCopyNW(
                (*lpFindFileData).cFileName.as_mut_ptr(),
                core::mem::size_of_val(&(*lpFindFileData).cFileName),
                (*DirInfo.FullDirInfo).FileName.as_ptr(),
                (*DirInfo.FullDirInfo).FileNameLength as usize,
            );

            (*lpFindFileData).cAlternateFileName[0] = 0u16; // UNICODE_NULL
        } else {
            // ASSERT(FALSE) - invalid InfoLevelId
            // In release builds this is a no-op
        }
    }
}
unsafe fn CopyStreamData(
    FindStreamData: *mut FIND_STREAM_DATA,
    lpFindStreamData: *mut WIN32_FIND_STREAM_DATA,
) {
    // _SEH2_TRY
    {
        // ASSERT(FindStreamData->CurrentInfo);
        match (*FindStreamData).InfoLevel {
            FindStreamInfoStandard => {
                let StreamNameLen: ULONG = min(
                    (*FindStreamData).CurrentInfo.as_ref().unwrap().StreamNameLength,
                    (core::mem::size_of::<[WCHAR; 260]>() - core::mem::size_of::<WCHAR>()) as ULONG,
                );

                RtlZeroMemory(
                    lpFindStreamData as *mut _,
                    core::mem::size_of::<WIN32_FIND_STREAM_DATA>() as SIZE_T,
                );

                (*lpFindStreamData).StreamSize.QuadPart =
                    (*FindStreamData).CurrentInfo.as_ref().unwrap().StreamSize.QuadPart;
                RtlCopyMemory(
                    (*lpFindStreamData).cStreamName.as_mut_ptr() as *mut _,
                    (*FindStreamData).CurrentInfo.as_ref().unwrap().StreamName.as_ptr() as *const _,
                    StreamNameLen as SIZE_T,
                );
                (*lpFindStreamData).cStreamName[(StreamNameLen / core::mem::size_of::<WCHAR>() as ULONG) as usize] = UNICODE_NULL;
            }
            _ => {
                // ASSERT(FALSE);
            }
        }
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
    }
    // _SEH2_END

    return;
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstFileA(
    lpFileName: LPCSTR,
    lpFindFileData: LPWIN32_FIND_DATAA,
) -> HANDLE {
    let hSearch: HANDLE;
    let mut Status: NTSTATUS;
    let mut Ansi: ANSI_STRING = core::mem::zeroed();
    let mut UTF8: _UNICODE_STRING = core::mem::zeroed();
    let lpFileNameW: PUNICODE_STRING;
    let mut FindFileDataW: WIN32_FIND_DATAW = core::mem::zeroed();

    lpFileNameW = Basep8BitStringToStaticUnicodeString(lpFileName);
    if lpFileNameW.is_null() {
        return INVALID_HANDLE_VALUE;
    }

    hSearch = FindFirstFileExW(
        (*lpFileNameW).Buffer,
        FindExInfoStandard,
        &mut FindFileDataW as *mut WIN32_FIND_DATAW as *mut c_void,
        FindExSearchNameMatch,
        core::ptr::null_mut(),
        0,
    );
    if hSearch == INVALID_HANDLE_VALUE {
        return INVALID_HANDLE_VALUE;
    }

    RtlCopyMemory(
        lpFindFileData as *mut c_void,
        &FindFileDataW as *const WIN32_FIND_DATAW as *const c_void,
        core::mem::offset_of!(WIN32_FIND_DATAA, cFileName) as SIZE_T,
    );

    RtlInitUnicodeString(&mut UTF8, FindFileDataW.cFileName.as_ptr());
    Ansi.Buffer = (*lpFindFileData).cFileName.as_mut_ptr();
    Ansi.Length = 0;
    Ansi.MaximumLength = core::mem::size_of::<[u8; 260]>() as u16;
    Status = BasepUnicodeStringTo8BitString(&mut Ansi, &UTF8, 0);
    if !NT_SUCCESS(Status) {
        FindClose(hSearch);
        BaseSetLastNTError(Status);
        return INVALID_HANDLE_VALUE;
    }

    RtlInitUnicodeString(&mut UTF8, FindFileDataW.cAlternateFileName.as_ptr());
    Ansi.Buffer = (*lpFindFileData).cAlternateFileName.as_mut_ptr();
    Ansi.Length = 0;
    Ansi.MaximumLength = core::mem::size_of::<[u8; 14]>() as u16;
    Status = BasepUnicodeStringTo8BitString(&mut Ansi, &UTF8, 0);
    if !NT_SUCCESS(Status) {
        FindClose(hSearch);
        BaseSetLastNTError(Status);
        return INVALID_HANDLE_VALUE;
    }

    hSearch
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstFileW(
    lpFileName: LPCWSTR,
    lpFindFileData: LPWIN32_FIND_DATAW,
) -> HANDLE {
    return FindFirstFileExW(
        lpFileName,
        FindExInfoStandard,
        lpFindFileData as LPVOID,
        FindExSearchNameMatch,
        core::ptr::null_mut(),
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FindNextFileA(
    hFindFile: HANDLE,
    lpFindFileData: LPWIN32_FIND_DATAA,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Ansi: ANSI_STRING = core::mem::zeroed();
    let mut UTF8: _UNICODE_STRING = core::mem::zeroed();
    let mut FindFileDataW: WIN32_FIND_DATAW = core::mem::zeroed();

    if FindNextFileW(hFindFile, &mut FindFileDataW) == 0 {
        return 0;
    }

    RtlCopyMemory(
        lpFindFileData as *mut core::ffi::c_void,
        &FindFileDataW as *const WIN32_FIND_DATAW as *const core::ffi::c_void,
        core::mem::offset_of!(WIN32_FIND_DATAA, cFileName) as SIZE_T,
    );

    RtlInitUnicodeString(&mut UTF8, FindFileDataW.cFileName.as_ptr());
    Ansi.Buffer = (*lpFindFileData).cFileName.as_mut_ptr();
    Ansi.Length = 0;
    Ansi.MaximumLength = core::mem::size_of::<[u8; 260]>() as u16; // sizeof(lpFindFileData->cFileName)
    Status = BasepUnicodeStringTo8BitString(&mut Ansi, &UTF8, 0);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    RtlInitUnicodeString(&mut UTF8, FindFileDataW.cAlternateFileName.as_ptr());
    Ansi.Buffer = (*lpFindFileData).cAlternateFileName.as_mut_ptr();
    Ansi.Length = 0;
    Ansi.MaximumLength = core::mem::size_of::<[u8; 14]>() as u16; // sizeof(lpFindFileData->cAlternateFileName)
    Status = BasepUnicodeStringTo8BitString(&mut Ansi, &UTF8, 0);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn FindNextFileW(
    hFindFile: HANDLE,
    lpFindFileData: LPWIN32_FIND_DATAW,
) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut FoundFile: DIR_INFORMATION = DIR_INFORMATION { DirInfo: core::ptr::null_mut() };

    // TRACE("FindNextFileW(%p, 0x%p)\n", hFindFile, lpFindFileData);

    if hFindFile != FIND_DEVICE_HANDLE {
        let FindDataHandle: PFIND_DATA_HANDLE = hFindFile as PFIND_DATA_HANDLE;
        let FindFileData: PFIND_FILE_DATA;
        let InfoLevel: FINDEX_INFO_LEVELS;
        let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut DirInfo: DIR_INFORMATION = DIR_INFORMATION { DirInfo: core::ptr::null_mut() };
        let mut NextDirInfo: DIR_INFORMATION = DIR_INFORMATION { DirInfo: core::ptr::null_mut() };

        if hFindFile.is_null()
            || hFindFile == INVALID_HANDLE_VALUE
            || (*FindDataHandle).Type != FindFile
        {
            SetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }

        RtlEnterCriticalSection(&mut (*FindDataHandle).Lock);

        FindFileData = (*FindDataHandle).u.FindFileData;
        InfoLevel = (*FindFileData).InfoLevel;

        loop {
            if (*FindFileData).NextDirInfo.DirInfo.is_null() {
                Status = NtQueryDirectoryFile(
                    (*FindFileData).Handle,
                    core::ptr::null_mut(),
                    None,
                    core::ptr::null_mut(),
                    &mut IoStatusBlock,
                    (*FindFileData).Buffer.as_mut_ptr() as *mut core::ffi::c_void,
                    core::mem::size_of_val(&(*FindFileData).Buffer) as ULONG,
                    if InfoLevel == FindExInfoStandard {
                        FileBothDirectoryInformation
                    } else {
                        FileFullDirectoryInformation
                    },
                    0,
                    core::ptr::null_mut(),
                    0,
                );
                if Status == STATUS_BUFFER_OVERFLOW {
                    (*FindFileData).HasMoreData = 1u8;
                    Status = STATUS_SUCCESS;
                } else {
                    if !NT_SUCCESS(Status) {
                        break;
                    }
                    (*FindFileData).HasMoreData = 0u8;
                }

                (*FindFileData).NextDirInfo.DirInfo = (*FindFileData).Buffer.as_mut_ptr() as *mut core::ffi::c_void;
            }

            DirInfo = (*FindFileData).NextDirInfo;

            if (*DirInfo.FullDirInfo).NextEntryOffset != 0 {
                let BufferEnd: ULONG_PTR = ((*FindFileData).Buffer.as_ptr() as ULONG_PTR)
                    + core::mem::size_of_val(&(*FindFileData).Buffer) as ULONG_PTR;
                let pFileName: PWSTR;

                NextDirInfo.DirInfo = {
                    let tmp = (*FindFileData).NextDirInfo.DirInfo;
                    (*FindFileData).NextDirInfo.DirInfo = (DirInfo.DirInfo as ULONG_PTR
                        + (*DirInfo.FullDirInfo).NextEntryOffset as ULONG_PTR)
                        as *mut core::ffi::c_void;
                    tmp
                };

                pFileName = if InfoLevel == FindExInfoStandard {
                    (*NextDirInfo.BothDirInfo).FileName.as_mut_ptr()
                } else {
                    (*NextDirInfo.FullDirInfo).FileName.as_mut_ptr()
                };

                if BufferEnd < NextDirInfo.DirInfo as ULONG_PTR
                    || BufferEnd
                        < (&raw mut (*NextDirInfo.FullDirInfo).FileNameLength as ULONG_PTR)
                            + core::mem::size_of_val(&(*NextDirInfo.FullDirInfo).FileNameLength)
                                as ULONG_PTR
                    || BufferEnd
                        <= (pFileName as ULONG_PTR
                            + (*NextDirInfo.FullDirInfo).FileNameLength as ULONG_PTR)
                {
                    (*FindFileData).NextDirInfo.DirInfo = core::ptr::null_mut();
                }
            } else {
                (*FindFileData).NextDirInfo.DirInfo = core::ptr::null_mut();
            }

            if ((*FindFileData).SearchOp != FindExSearchLimitToDirectories)
                || ((*DirInfo.FullDirInfo).FileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
            {
                FoundFile = DirInfo;
            }

            if FoundFile.DirInfo.is_null()
                && ((*FindFileData).NextDirInfo.DirInfo.is_null() == false
                    || (*FindFileData).HasMoreData != 0)
            {
                continue;
            }
            break;
        }

        if !FoundFile.DirInfo.is_null() {
            CopyFindData(lpFindFileData, InfoLevel, FoundFile);
        }

        RtlLeaveCriticalSection(&mut (*FindDataHandle).Lock);
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    } else if FoundFile.DirInfo.is_null() {
        SetLastError(ERROR_NO_MORE_FILES);
        return 0;
    }

    return 1;
}
#[no_mangle]
#[allow(unreachable_patterns)] // faithful to C: defensive `default:` arm on FindDataHandle->Type
pub unsafe extern "system" fn FindClose(hFindFile: HANDLE) -> BOOL {
    // TRACE("FindClose(hFindFile %p)\n", hFindFile);

    if hFindFile == FIND_DEVICE_HANDLE {
        return 1;
    }

    if hFindFile.is_null() || hFindFile == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Protect with SEH against closing attempts on invalid handles. */
    let mut __try_result: i32 = 0;
    let mut __except_code: NTSTATUS = 0;
    let mut __try_entered: bool = false;
    let mut __except_entered: bool = false;

    // _SEH2_TRY
    __try_entered = true;
    {
        let FindDataHandle: PFIND_DATA_HANDLE = hFindFile as PFIND_DATA_HANDLE;

        match (*FindDataHandle).Type {
            FindFile => {
                RtlEnterCriticalSection(&mut (*FindDataHandle).Lock);
                NtClose((*(*FindDataHandle).u.FindFileData).Handle);
                RtlLeaveCriticalSection(&mut (*FindDataHandle).Lock);
                RtlDeleteCriticalSection(&mut (*FindDataHandle).Lock);
            }

            FindStream => {
                RtlEnterCriticalSection(&mut (*FindDataHandle).Lock);
                if !(*(*FindDataHandle).u.FindStreamData).FileStreamInfo.is_null() {
                    RtlFreeHeap(
                        RtlGetProcessHeap(),
                        0,
                        (*(*FindDataHandle).u.FindStreamData).FileStreamInfo as PVOID,
                    );
                }
                RtlLeaveCriticalSection(&mut (*FindDataHandle).Lock);
                RtlDeleteCriticalSection(&mut (*FindDataHandle).Lock);
            }

            _ => {
                SetLastError(ERROR_INVALID_HANDLE);
                // _SEH2_YIELD(return FALSE);
                __try_result = 0;
                __try_entered = false;
                return 0;
            }
        }

        RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle as PVOID);
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    if __try_entered {
        // No exception occurred
    } else {
        __except_entered = true;
        __except_code = _SEH2_GetExceptionCode();
        {
            BaseSetLastNTError(__except_code);
            // _SEH2_YIELD(return FALSE);
            return 0;
        }
    }
    // _SEH2_END

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstFileExA(
    lpFileName: LPCSTR,
    fInfoLevelId: FINDEX_INFO_LEVELS,
    lpFindFileData: LPVOID,
    fSearchOp: FINDEX_SEARCH_OPS,
    lpSearchFilter: LPVOID,
    dwAdditionalFlags: DWORD,
) -> HANDLE {
    let hSearch: HANDLE;
    let mut Status: NTSTATUS;
    let mut Ansi: ANSI_STRING = core::mem::zeroed();
    let mut UTF8: _UNICODE_STRING = core::mem::zeroed();
    let lpFileNameW: PUNICODE_STRING;
    let mut FindFileDataW: WIN32_FIND_DATAW = core::mem::zeroed();
    let lpFindFileDataA: LPWIN32_FIND_DATAA = lpFindFileData as LPWIN32_FIND_DATAA;

    if (fInfoLevelId != FindExInfoStandard && fInfoLevelId != FindExInfoBasic)
        || fSearchOp == FindExSearchLimitToDevices
        || (dwAdditionalFlags & !FIND_FIRST_EX_CASE_SENSITIVE) != 0
    {
        SetLastError(
            if fSearchOp == FindExSearchLimitToDevices {
                ERROR_NOT_SUPPORTED
            } else {
                ERROR_INVALID_PARAMETER
            },
        );
        return INVALID_HANDLE_VALUE;
    }

    lpFileNameW = Basep8BitStringToStaticUnicodeString(lpFileName);
    if lpFileNameW.is_null() {
        return INVALID_HANDLE_VALUE;
    }

    hSearch = FindFirstFileExW(
        (*lpFileNameW).Buffer,
        fInfoLevelId,
        &mut FindFileDataW as *mut WIN32_FIND_DATAW as LPVOID,
        fSearchOp,
        lpSearchFilter,
        dwAdditionalFlags,
    );
    if hSearch == INVALID_HANDLE_VALUE {
        return INVALID_HANDLE_VALUE;
    }

    RtlCopyMemory(
        lpFindFileDataA as *mut core::ffi::c_void,
        &FindFileDataW as *const WIN32_FIND_DATAW as *const core::ffi::c_void,
        core::mem::offset_of!(WIN32_FIND_DATAA, cFileName) as SIZE_T,
    );

    RtlInitUnicodeString(&mut UTF8, FindFileDataW.cFileName.as_ptr());
    Ansi.Buffer = (*lpFindFileDataA).cFileName.as_mut_ptr();
    Ansi.Length = 0;
    Ansi.MaximumLength = core::mem::size_of::<[CHAR; 260]>() as u16;
    Status = BasepUnicodeStringTo8BitString(&mut Ansi, &UTF8, 0);
    if !NT_SUCCESS(Status) {
        FindClose(hSearch);
        BaseSetLastNTError(Status);
        return INVALID_HANDLE_VALUE;
    }

    if fInfoLevelId != FindExInfoBasic {
        RtlInitUnicodeString(&mut UTF8, FindFileDataW.cAlternateFileName.as_ptr());
        Ansi.Buffer = (*lpFindFileDataA).cAlternateFileName.as_mut_ptr();
        Ansi.Length = 0;
        Ansi.MaximumLength = core::mem::size_of::<[CHAR; 14]>() as u16;
        Status = BasepUnicodeStringTo8BitString(&mut Ansi, &UTF8, 0);
        if !NT_SUCCESS(Status) {
            FindClose(hSearch);
            BaseSetLastNTError(Status);
            return INVALID_HANDLE_VALUE;
        }
    } else {
        (*lpFindFileDataA).cAlternateFileName[0] = 0i8;
    }

    hSearch
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstFileExW(
    lpFileName: LPCWSTR,
    fInfoLevelId: FINDEX_INFO_LEVELS,
    lpFindFileData: LPVOID,
    fSearchOp: FINDEX_SEARCH_OPS,
    lpSearchFilter: LPVOID,
    dwAdditionalFlags: DWORD,
) -> HANDLE {
    // UNREFERENCED_PARAMETER(lpSearchFilter);
    // TRACE("FindFirstFileExW(lpFileName %S)\n", lpFileName);

    if (fInfoLevelId != FindExInfoStandard && fInfoLevelId != FindExInfoBasic)
        || fSearchOp == FindExSearchLimitToDevices
        || (dwAdditionalFlags & !FIND_FIRST_EX_CASE_SENSITIVE) != 0
    {
        SetLastError(if fSearchOp == FindExSearchLimitToDevices {
            ERROR_NOT_SUPPORTED
        } else {
            ERROR_INVALID_PARAMETER
        });
        return INVALID_HANDLE_VALUE;
    }

    if fSearchOp == FindExSearchNameMatch || fSearchOp == FindExSearchLimitToDirectories {
        let Win32FindData: LPWIN32_FIND_DATAW = lpFindFileData as LPWIN32_FIND_DATAW;
        let FindDataHandle: PFIND_DATA_HANDLE;
        let FindFileData: PFIND_FILE_DATA;

        let mut NtPath: _UNICODE_STRING = core::mem::zeroed();
        let mut FilePattern: _UNICODE_STRING = core::mem::zeroed();
        let mut FileName: _UNICODE_STRING = core::mem::zeroed();
        let NtPathBuffer: PWSTR;
        let mut RelativePath: RTL_RELATIVE_NAME_U = core::mem::zeroed();
        let mut DeviceNameInfo: ULONG = 0;

        let mut Status: NTSTATUS;
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut hDirectory: HANDLE = core::ptr::null_mut();

        let mut HadADot: BOOLEAN = 0;

        // C: `DECLSPEC_ALIGN(4) BYTE DirectoryInfo[FIND_DATA_SIZE]` —
        // NtQueryDirectoryFile requires a ULONG-aligned buffer, and a bare
        // byte array is only guaranteed alignment 1.
        #[repr(C, align(4))]
        struct AlignedDirectoryInfo([BYTE; FIND_DATA_SIZE]);
        let mut DirectoryInfo = AlignedDirectoryInfo([0; FIND_DATA_SIZE]);
        let mut DirInfo: DIR_INFORMATION = DIR_INFORMATION {
            DirInfo: DirectoryInfo.0.as_mut_ptr() as PVOID,
        };

        RtlInitUnicodeString(&mut FileName, lpFileName);
        if FileName.Length != 0
            && *FileName.Buffer.offset(FileName.Length as isize / core::mem::size_of::<WCHAR>() as isize - 1) as u16 == '.' as u16
        {
            HadADot = 1;
        }

        if RtlDosPathNameToNtPathName_U(
            lpFileName,
            &mut NtPath,
            &mut FilePattern.Buffer as *mut PWSTR as *mut PCWSTR,
            &mut RelativePath,
        ) == 0
        {
            SetLastError(ERROR_PATH_NOT_FOUND);
            return INVALID_HANDLE_VALUE;
        }

        // DPRINT("lpFileName = '%S'\n", lpFileName);
        // DPRINT("FilePattern.Buffer = '%S'\n", FilePattern.Buffer);
        // DPRINT("RelativePath.RelativeName = '%wZ'\n", &RelativePath.RelativeName);
        // DPRINT("NtPath.Buffer = '%S'\n", NtPath.Buffer);
        // DPRINT("NtPath - Before = '%wZ'\n", &NtPath);

        NtPathBuffer = NtPath.Buffer;

        if FilePattern.Buffer.is_null() || *FilePattern.Buffer == 0 {
            DeviceNameInfo = RtlIsDosDeviceName_U(lpFileName);
            if DeviceNameInfo != 0 {
                RtlReleaseRelativeName(&mut RelativePath);
                RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer as PVOID);

                CopyDeviceFindData(Win32FindData, lpFileName, DeviceNameInfo);
                return FIND_DEVICE_HANDLE;
            }
        }

        if !FilePattern.Buffer.is_null() {
            FilePattern.Length = NtPath.Length
                - ((FilePattern.Buffer as ULONG_PTR).wrapping_sub(NtPath.Buffer as ULONG_PTR)) as USHORT;
        } else {
            FilePattern.Length = 0;
        }
        FilePattern.MaximumLength = FilePattern.Length;

        if RelativePath.RelativeName.Length != 0
            && RelativePath.RelativeName.Buffer != FilePattern.Buffer
        {
            if !FilePattern.Buffer.is_null() {
                NtPath.MaximumLength = ((FilePattern.Buffer as ULONG_PTR)
                    .wrapping_sub(RelativePath.RelativeName.Buffer as ULONG_PTR))
                    as USHORT;
                NtPath.Length = NtPath.MaximumLength;
                NtPath.Buffer = RelativePath.RelativeName.Buffer;
            }
        } else {
            RelativePath.ContainingDirectory = core::ptr::null_mut();
            if !FilePattern.Buffer.is_null() {
                NtPath.MaximumLength = ((FilePattern.Buffer as ULONG_PTR)
                    .wrapping_sub(NtPath.Buffer as ULONG_PTR))
                    as USHORT;
                NtPath.Length = NtPath.MaximumLength;
            }
        }

        // DPRINT("NtPath - After = '%wZ'\n", &NtPath);
        // DPRINT("FilePattern = '%wZ'\n", &FilePattern);
        // DPRINT("RelativeTo = 0x%p\n", RelativePath.ContainingDirectory);

        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut NtPath,
            if (dwAdditionalFlags & FIND_FIRST_EX_CASE_SENSITIVE) != 0 {
                0
            } else {
                OBJ_CASE_INSENSITIVE
            },
            RelativePath.ContainingDirectory,
            core::ptr::null_mut(),
        );

        Status = NtOpenFile(
            &mut hDirectory,
            FILE_LIST_DIRECTORY | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
        );

        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut RelativePath);
            RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer as PVOID);

            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_OBJECT_PATH_NOT_FOUND;
            } else if Status == STATUS_OBJECT_TYPE_MISMATCH {
                Status = STATUS_OBJECT_PATH_NOT_FOUND;
            }

            BaseSetLastNTError(Status);
            return INVALID_HANDLE_VALUE;
        }

        if FilePattern.Length == 0 {
            NtClose(hDirectory);
            RtlReleaseRelativeName(&mut RelativePath);
            RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer as PVOID);

            SetLastError(ERROR_FILE_NOT_FOUND);
            return INVALID_HANDLE_VALUE;
        }

        // C: RtlCompareMemory(FilePattern.Buffer, L"*.*", 6) -- the WIDE literal. Compared
        // against the narrow b"*.*" the 6 bytes never matched, "*.*" went through the DOS
        // wildcard rewrite instead (as `<"*`), and that rejects "." and "..": FindFirstFileW
        // on any empty directory failed with ERROR_FILE_NOT_FOUND, so userenv could not copy
        // the Default User profile and GUI setup stopped at "CopySystemProfile() failed".
        static STAR_DOT_STAR_W: [u16; 3] = [b'*' as u16, b'.' as u16, b'*' as u16];
        if FilePattern.Length == 6
            && RtlCompareMemory(
                FilePattern.Buffer as PVOID,
                STAR_DOT_STAR_W.as_ptr() as PVOID,
                6,
            ) == 6
        {
            FilePattern.Length = 2;
        } else {
            let mut PatternIndex: USHORT = 0;
            while (PatternIndex as usize) < (FilePattern.Length as usize / core::mem::size_of::<WCHAR>()) {
                if PatternIndex > 0 {
                    if *FilePattern.Buffer.offset(PatternIndex as isize) as u16 == '.' as u16
                        && *FilePattern.Buffer.offset(PatternIndex as isize - 1) as u16 == '*' as u16
                    {
                        *FilePattern.Buffer.offset(PatternIndex as isize - 1) = '<' as u16 as WCHAR;
                    }
                }

                if *FilePattern.Buffer.offset(PatternIndex as isize) as u16 == '?' as u16 {
                    *FilePattern.Buffer.offset(PatternIndex as isize) = '>' as u16 as WCHAR;
                    if PatternIndex > 0 {
                        if *FilePattern.Buffer.offset(PatternIndex as isize - 1) as u16 == '.' as u16 {
                            *FilePattern.Buffer.offset(PatternIndex as isize - 1) = '\"' as u16 as WCHAR;
                        }
                    }
                } else if *FilePattern.Buffer.offset(PatternIndex as isize) as u16 == '*' as u16 {
                    if PatternIndex > 0 {
                        if *FilePattern.Buffer.offset(PatternIndex as isize - 1) as u16 == '.' as u16 {
                            *FilePattern.Buffer.offset(PatternIndex as isize - 1) = '\"' as u16 as WCHAR;
                        }
                    }
                }

                PatternIndex += 1;
            }

            if HadADot != 0 {
                if *FilePattern
                    .Buffer
                    .offset(FilePattern.Length as isize / core::mem::size_of::<WCHAR>() as isize - 1)
                    as u16
                    == '*' as u16
                {
                    *FilePattern
                        .Buffer
                        .offset(FilePattern.Length as isize / core::mem::size_of::<WCHAR>() as isize - 1) =
                        '<' as u16 as WCHAR;
                }
            }
        }

        Status = NtQueryDirectoryFile(
            hDirectory,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            DirInfo.DirInfo,
            core::mem::size_of_val(&DirectoryInfo) as ULONG,
            if fInfoLevelId == FindExInfoStandard {
                FileBothDirectoryInformation
            } else {
                FileFullDirectoryInformation
            },
            1,
            &mut FilePattern,
            1,
        );

        RtlReleaseRelativeName(&mut RelativePath);
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer as PVOID);

        if !NT_SUCCESS(Status) {
            NtClose(hDirectory);
            BaseSetLastNTError(Status);
            return INVALID_HANDLE_VALUE;
        }

        // ASSERT(DirInfo.FullDirInfo->NextEntryOffset == 0);

        CopyFindData(Win32FindData, fInfoLevelId, DirInfo);

        FindDataHandle = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (core::mem::size_of::<FIND_DATA_HANDLE>() + core::mem::size_of::<FIND_FILE_DATA>()) as SIZE_T,
        ) as PFIND_DATA_HANDLE;
        if FindDataHandle.is_null() {
            NtClose(hDirectory);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return INVALID_HANDLE_VALUE;
        }

        (*FindDataHandle).Type = FindFile;
        // C: (PFIND_FILE_DATA)(FindDataHandle + 1) — advance by sizeof(FIND_DATA_HANDLE),
        // THEN cast. Offsetting after the cast advances by sizeof(FIND_FILE_DATA) (~16KB),
        // placing FindFileData (and its Buffer) past the allocation → heap overrun.
        (*FindDataHandle).u.FindFileData =
            FindDataHandle.offset(1) as PFIND_FILE_DATA;
        FindFileData = (*FindDataHandle).u.FindFileData;

        (*FindFileData).Handle = hDirectory;
        (*FindFileData).InfoLevel = fInfoLevelId;
        (*FindFileData).SearchOp = fSearchOp;
        (*FindFileData).HasMoreData = 0;
        (*FindFileData).NextDirInfo.DirInfo = core::ptr::null_mut();

        Status = RtlInitializeCriticalSection(&mut (*FindDataHandle).Lock);
        if !NT_SUCCESS(Status) {
            NtClose(hDirectory);
            RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle as PVOID);

            BaseSetLastNTError(Status);
            return INVALID_HANDLE_VALUE;
        }

        return FindDataHandle as HANDLE;
    } else {
        SetLastError(ERROR_NOT_SUPPORTED);
        return INVALID_HANDLE_VALUE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstStreamW(
    lpFileName: LPCWSTR,
    InfoLevel: STREAM_INFO_LEVELS,
    lpFindStreamData: LPVOID,
    dwFlags: DWORD,
) -> HANDLE {
    let mut FindDataHandle: *mut FIND_DATA_HANDLE = core::ptr::null_mut();
    let FindStreamData: *mut FIND_STREAM_DATA;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut NtFilePath: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut BufferSize: ULONG = 0;

    if dwFlags != 0 || InfoLevel != FindStreamInfoStandard || lpFindStreamData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    if RtlDosPathNameToNtPathName_U(lpFileName, &mut NtFilePath, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return INVALID_HANDLE_VALUE;
    }

    'Cleanup: {
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtFilePath,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtCreateFile(
        &mut FileHandle,
        0,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        0,
        FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN,
        0,
        core::ptr::null_mut(),
        0,
    );
    if !NT_SUCCESS(Status) {
        break 'Cleanup;
    }

    FindDataHandle = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<FIND_DATA_HANDLE>() + core::mem::size_of::<FIND_STREAM_DATA>()) as SIZE_T,
    ) as *mut FIND_DATA_HANDLE;
    if FindDataHandle.is_null() {
        Status = STATUS_NO_MEMORY;
        break 'Cleanup;
    }

    (*FindDataHandle).Type = FindStream;
    (*FindDataHandle).u.FindStreamData = FindDataHandle.add(1) as *mut FIND_STREAM_DATA;
    FindStreamData = (*FindDataHandle).u.FindStreamData;

    (*FindStreamData).InfoLevel = InfoLevel;
    (*FindStreamData).FileStreamInfo = core::ptr::null_mut();
    (*FindStreamData).CurrentInfo = core::ptr::null_mut();

    Status = RtlInitializeCriticalSection(&mut (*FindDataHandle).Lock);
    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle as *mut _);
        break 'Cleanup;
    }

    loop {
        BufferSize += 0x1000;

        if (*FindStreamData).FileStreamInfo.is_null() {
            (*FindStreamData).FileStreamInfo = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                BufferSize as SIZE_T,
            ) as *mut FILE_STREAM_INFORMATION;
            if (*FindStreamData).FileStreamInfo.is_null() {
                Status = STATUS_NO_MEMORY;
                break;
            }
        } else {
            let pfsi: *mut FILE_STREAM_INFORMATION;

            pfsi = RtlReAllocateHeap(
                RtlGetProcessHeap(),
                0,
                (*FindStreamData).FileStreamInfo as *mut _,
                BufferSize as SIZE_T,
            ) as *mut FILE_STREAM_INFORMATION;
            if pfsi.is_null() {
                Status = STATUS_NO_MEMORY;
                break;
            }

            (*FindStreamData).FileStreamInfo = pfsi;
        }

        Status = NtQueryInformationFile(
            FileHandle,
            &mut IoStatusBlock,
            (*FindStreamData).FileStreamInfo as *mut _,
            BufferSize,
            FileStreamInformation,
        );

        if Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }
    }

    if NT_SUCCESS(Status) {
        (*FindStreamData).CurrentInfo = (*FindStreamData).FileStreamInfo;
        CopyStreamData(FindStreamData, lpFindStreamData as *mut WIN32_FIND_STREAM_DATA);

        Status = STATUS_SUCCESS;
    } else {
        if !(*FindStreamData).FileStreamInfo.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*FindStreamData).FileStreamInfo as *mut _);
        }

        RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle as *mut _);
    }
    } // 'Cleanup

    // Cleanup:
    if !FileHandle.is_null() {
        NtClose(FileHandle);
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, NtFilePath.Buffer as *mut _);

    if NT_SUCCESS(Status) {
        return FindDataHandle as HANDLE;
    } else {
        BaseSetLastNTError(Status);
        return INVALID_HANDLE_VALUE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FindNextStreamW(
    hFindStream: HANDLE,
    lpFindStreamData: LPVOID,
) -> BOOL {
    let FindDataHandle: PFIND_DATA_HANDLE = hFindStream as PFIND_DATA_HANDLE;
    let FindStreamData: PFIND_STREAM_DATA;

    if hFindStream.is_null()
        || hFindStream == INVALID_HANDLE_VALUE
        || (*FindDataHandle).Type != FindStream
    {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    RtlEnterCriticalSection(&mut (*FindDataHandle).Lock);

    FindStreamData = (*FindDataHandle).u.FindStreamData;

    /* Select next stream if possible */
    if (*(*FindStreamData).CurrentInfo).NextEntryOffset != 0
    {
        (*FindStreamData).CurrentInfo = ((*FindStreamData).CurrentInfo as ULONG_PTR)
            .wrapping_add((*(*FindStreamData).CurrentInfo).NextEntryOffset as ULONG_PTR)
            as PFILE_STREAM_INFORMATION;

        /* Return the information */
        CopyStreamData(FindStreamData, lpFindStreamData as *mut WIN32_FIND_STREAM_DATA);

        RtlLeaveCriticalSection(&mut (*FindDataHandle).Lock);
        return 1;
    }
    else
    {
        RtlLeaveCriticalSection(&mut (*FindDataHandle).Lock);

        SetLastError(ERROR_HANDLE_EOF);
        return 0;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/find.c
//  * PURPOSE:         Find functions
//  * PROGRAMMERS:     Ariadne (ariadne@xs4all.nl)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  *                  Hermes Belusca-Maito
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
// #include <ntstrsafe.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(kernel32file);
//
//
// /* TYPES **********************************************************************/
//
// #define FIND_DATA_SIZE      0x4000
// #define FIND_DEVICE_HANDLE  ((HANDLE)0x1)
//
// typedef enum _FIND_DATA_TYPE
// {
//     FindFile   = 1,
//     FindStream = 2
// } FIND_DATA_TYPE;
//
// /*
//  * FILE_FULL_DIR_INFORMATION and FILE_BOTH_DIR_INFORMATION structures layout.
//  *
//  *
//  *  struct FILE_FULL_DIR_INFORMATION   |   struct FILE_BOTH_DIR_INFORMATION
//  * ------------------------------------+---------------------------------------
//  *     ULONG NextEntryOffset;          |       ULONG NextEntryOffset;
//  *     ULONG FileIndex;                |       ULONG FileIndex;
//  *     LARGE_INTEGER CreationTime;     |       LARGE_INTEGER CreationTime;
//  *     LARGE_INTEGER LastAccessTime;   |       LARGE_INTEGER LastAccessTime;
//  *     LARGE_INTEGER LastWriteTime;    |       LARGE_INTEGER LastWriteTime;
//  *     LARGE_INTEGER ChangeTime;       |       LARGE_INTEGER ChangeTime;
//  *     LARGE_INTEGER EndOfFile;        |       LARGE_INTEGER EndOfFile;
//  *     LARGE_INTEGER AllocationSize;   |       LARGE_INTEGER AllocationSize;
//  *     ULONG FileAttributes;           |       ULONG FileAttributes;
//  *     ULONG FileNameLength;           |       ULONG FileNameLength;
//  *     ULONG EaSize;                   |       ULONG EaSize;
//  * ------------------------------------+---------------------------------------
//  *     WCHAR FileName[1];              |       CCHAR ShortNameLength;
//  *                                     |       WCHAR ShortName[12];
//  *                                     |       WCHAR FileName[1];
//  *
//  * Therefore we can use pointers to FILE_FULL_DIR_INFORMATION when one doesn't
//  * want to refer to the ShortName* fields and FileName (useful for implementing
//  * the FindExInfoBasic functionality for FindFirstFileEx), however a cast to
//  * FILE_BOTH_DIR_INFORMATION is required when one wants to use FileName and
//  * ShortName* fields (needed for the FindExInfoStandard functionality).
//  *
//  */
// typedef union _DIR_INFORMATION
// {
//     PVOID DirInfo;
//     PFILE_FULL_DIR_INFORMATION FullDirInfo;
//     PFILE_BOTH_DIR_INFORMATION BothDirInfo;
// } DIR_INFORMATION;
//
// typedef struct _FIND_FILE_DATA
// {
//     HANDLE Handle;
//     FINDEX_INFO_LEVELS InfoLevel;
//     FINDEX_SEARCH_OPS SearchOp;
//
//     /*
//      * For handling STATUS_BUFFER_OVERFLOW errors emitted by
//      * NtQueryDirectoryFile in the FindNextFile function.
//      */
//     BOOLEAN HasMoreData;
//
//     /*
//      * "Pointer" to the next file info structure in the buffer.
//      * The type is defined by the 'InfoLevel' parameter.
//      */
//     DIR_INFORMATION NextDirInfo;
//
//     BYTE Buffer[FIND_DATA_SIZE];
// } FIND_FILE_DATA, *PFIND_FILE_DATA;
//
// typedef struct _FIND_STREAM_DATA
// {
//     STREAM_INFO_LEVELS InfoLevel;
//     PFILE_STREAM_INFORMATION FileStreamInfo;
//     PFILE_STREAM_INFORMATION CurrentInfo;
// } FIND_STREAM_DATA, *PFIND_STREAM_DATA;
//
// typedef struct _FIND_DATA_HANDLE
// {
//     FIND_DATA_TYPE Type;
//     RTL_CRITICAL_SECTION Lock;
//
//     /*
//      * Pointer to the following finding data, located at
//      * (this + 1). The type is defined by the 'Type' parameter.
//      */
//     union
//     {
//         PFIND_FILE_DATA FindFileData;
//         PFIND_STREAM_DATA FindStreamData;
//     } u;
//
// } FIND_DATA_HANDLE, *PFIND_DATA_HANDLE;
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// static VOID
// CopyDeviceFindData(OUT LPWIN32_FIND_DATAW lpFindFileData,
//                    IN LPCWSTR lpFileName,
//                    IN ULONG DeviceNameInfo)
// {
//     LPCWSTR DeviceName;
//     SIZE_T Length;
//
//     _SEH2_TRY
//     {
//         /* DeviceNameInfo == { USHORT Offset; USHORT Length } */
//         Length     =  (SIZE_T)(DeviceNameInfo & 0xFFFF);
//         DeviceName = (LPCWSTR)((ULONG_PTR)lpFileName + ((DeviceNameInfo >> 16) & 0xFFFF));
//
//         /* Return the data */
//         RtlZeroMemory(lpFindFileData, sizeof(*lpFindFileData));
//         lpFindFileData->dwFileAttributes = FILE_ATTRIBUTE_ARCHIVE;
//         RtlStringCbCopyNW(lpFindFileData->cFileName,
//                           sizeof(lpFindFileData->cFileName),
//                           DeviceName, Length);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return;
// }
//
// static VOID
// CopyFindData(OUT LPWIN32_FIND_DATAW lpFindFileData,
//              IN FINDEX_INFO_LEVELS fInfoLevelId,
//              IN DIR_INFORMATION DirInfo)
// {
// #define ULARGE_INTEGER_2_FILETIME(ft, ul) \
// do { \
//     (ft).dwHighDateTime = (ul).u.HighPart; \
//     (ft).dwLowDateTime  = (ul).u.LowPart ; \
// } while(0)
//
//     _SEH2_TRY
//     {
//         RtlZeroMemory(lpFindFileData, sizeof(*lpFindFileData));
//
//         lpFindFileData->dwFileAttributes = DirInfo.FullDirInfo->FileAttributes;
//
//         ULARGE_INTEGER_2_FILETIME(lpFindFileData->ftCreationTime, DirInfo.FullDirInfo->CreationTime);
//         ULARGE_INTEGER_2_FILETIME(lpFindFileData->ftLastAccessTime, DirInfo.FullDirInfo->LastAccessTime);
//         ULARGE_INTEGER_2_FILETIME(lpFindFileData->ftLastWriteTime, DirInfo.FullDirInfo->LastWriteTime);
//
//         lpFindFileData->nFileSizeHigh = DirInfo.FullDirInfo->EndOfFile.u.HighPart;
//         lpFindFileData->nFileSizeLow = DirInfo.FullDirInfo->EndOfFile.u.LowPart;
//
//         /* dwReserved0 contains the NTFS reparse point tag, if any. */
//         if (DirInfo.FullDirInfo->FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT)
//             lpFindFileData->dwReserved0 = DirInfo.FullDirInfo->EaSize;
//         else
//             lpFindFileData->dwReserved0 = 0;
//
//         /* Unused dwReserved1 field */
//         lpFindFileData->dwReserved1 = 0;
//
//         if (fInfoLevelId == FindExInfoStandard)
//         {
//             RtlStringCbCopyNW(lpFindFileData->cFileName,
//                               sizeof(lpFindFileData->cFileName),
//                               DirInfo.BothDirInfo->FileName,
//                               DirInfo.BothDirInfo->FileNameLength);
//
//             RtlStringCbCopyNW(lpFindFileData->cAlternateFileName,
//                               sizeof(lpFindFileData->cAlternateFileName),
//                               DirInfo.BothDirInfo->ShortName,
//                               DirInfo.BothDirInfo->ShortNameLength);
//         }
//         else if (fInfoLevelId == FindExInfoBasic)
//         {
//             RtlStringCbCopyNW(lpFindFileData->cFileName,
//                               sizeof(lpFindFileData->cFileName),
//                               DirInfo.FullDirInfo->FileName,
//                               DirInfo.FullDirInfo->FileNameLength);
//
//             lpFindFileData->cAlternateFileName[0] = UNICODE_NULL;
//         }
//         else
//         {
//             /* Invalid InfoLevelId */
//             ASSERT(FALSE);
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return;
// }
//
// static VOID
// CopyStreamData(IN OUT PFIND_STREAM_DATA FindStreamData,
//                OUT PWIN32_FIND_STREAM_DATA lpFindStreamData)
// {
//     _SEH2_TRY
//     {
//         ASSERT(FindStreamData->CurrentInfo);
//
//         switch (FindStreamData->InfoLevel)
//         {
//             case FindStreamInfoStandard:
//             {
//                 ULONG StreamNameLen = min(FindStreamData->CurrentInfo->StreamNameLength,
//                                           sizeof(lpFindStreamData->cStreamName) - sizeof(WCHAR));
//
//                 RtlZeroMemory(lpFindStreamData, sizeof(*lpFindStreamData));
//
//                 lpFindStreamData->StreamSize.QuadPart = FindStreamData->CurrentInfo->StreamSize.QuadPart;
//                 RtlCopyMemory(lpFindStreamData->cStreamName,
//                               FindStreamData->CurrentInfo->StreamName,
//                               StreamNameLen);
//                 lpFindStreamData->cStreamName[StreamNameLen / sizeof(WCHAR)] = UNICODE_NULL;
//
//                 break;
//             }
//
//             default:
//             {
//                 /* Invalid InfoLevel */
//                 ASSERT(FALSE);
//                 break;
//             }
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     return;
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// FindFirstFileA(IN LPCSTR lpFileName,
//                OUT LPWIN32_FIND_DATAA lpFindFileData)
// {
//     HANDLE hSearch;
//     NTSTATUS Status;
//     ANSI_STRING Ansi;
//     UNICODE_STRING UTF8;
//     PUNICODE_STRING lpFileNameW;
//     WIN32_FIND_DATAW FindFileDataW;
//
//     lpFileNameW = Basep8BitStringToStaticUnicodeString(lpFileName);
//     if (!lpFileNameW) return INVALID_HANDLE_VALUE;
//
//     hSearch = FindFirstFileExW(lpFileNameW->Buffer,
//                                FindExInfoStandard,
//                                &FindFileDataW,
//                                FindExSearchNameMatch,
//                                NULL, 0);
//     if (hSearch == INVALID_HANDLE_VALUE) return INVALID_HANDLE_VALUE;
//
//     RtlCopyMemory(lpFindFileData,
//                   &FindFileDataW,
//                   FIELD_OFFSET(WIN32_FIND_DATAA, cFileName));
//
//     RtlInitUnicodeString(&UTF8, FindFileDataW.cFileName);
//     Ansi.Buffer = lpFindFileData->cFileName;
//     Ansi.Length = 0;
//     Ansi.MaximumLength = sizeof(lpFindFileData->cFileName);
//     Status = BasepUnicodeStringTo8BitString(&Ansi, &UTF8, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         FindClose(hSearch);
//         BaseSetLastNTError(Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     RtlInitUnicodeString(&UTF8, FindFileDataW.cAlternateFileName);
//     Ansi.Buffer = lpFindFileData->cAlternateFileName;
//     Ansi.Length = 0;
//     Ansi.MaximumLength = sizeof(lpFindFileData->cAlternateFileName);
//     Status = BasepUnicodeStringTo8BitString(&Ansi, &UTF8, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         FindClose(hSearch);
//         BaseSetLastNTError(Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     return hSearch;
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// FindFirstFileW(IN LPCWSTR lpFileName,
//                OUT LPWIN32_FIND_DATAW lpFindFileData)
// {
//     return FindFirstFileExW(lpFileName,
//                             FindExInfoStandard,
//                             lpFindFileData,
//                             FindExSearchNameMatch,
//                             NULL, 0);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindNextFileA(IN HANDLE hFindFile,
//               OUT LPWIN32_FIND_DATAA lpFindFileData)
// {
//     NTSTATUS Status;
//     ANSI_STRING Ansi;
//     UNICODE_STRING UTF8;
//     WIN32_FIND_DATAW FindFileDataW;
//
//     if (!FindNextFileW(hFindFile, &FindFileDataW))
//         return FALSE;
//
//     RtlCopyMemory(lpFindFileData,
//                   &FindFileDataW,
//                   FIELD_OFFSET(WIN32_FIND_DATAA, cFileName));
//
//     RtlInitUnicodeString(&UTF8, FindFileDataW.cFileName);
//     Ansi.Buffer = lpFindFileData->cFileName;
//     Ansi.Length = 0;
//     Ansi.MaximumLength = sizeof(lpFindFileData->cFileName);
//     Status = BasepUnicodeStringTo8BitString(&Ansi, &UTF8, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&UTF8, FindFileDataW.cAlternateFileName);
//     Ansi.Buffer = lpFindFileData->cAlternateFileName;
//     Ansi.Length = 0;
//     Ansi.MaximumLength = sizeof(lpFindFileData->cAlternateFileName);
//     Status = BasepUnicodeStringTo8BitString(&Ansi, &UTF8, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindNextFileW(IN HANDLE hFindFile,
//               OUT LPWIN32_FIND_DATAW lpFindFileData)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     DIR_INFORMATION FoundFile = {NULL};
//
//     TRACE("FindNextFileW(%p, 0x%p)\n", hFindFile, lpFindFileData);
//
//     if (hFindFile != FIND_DEVICE_HANDLE)
//     {
//         PFIND_DATA_HANDLE FindDataHandle = (PFIND_DATA_HANDLE)hFindFile;
//         PFIND_FILE_DATA FindFileData;
//         FINDEX_INFO_LEVELS InfoLevel;
//         IO_STATUS_BLOCK IoStatusBlock;
//         DIR_INFORMATION DirInfo = {NULL}, NextDirInfo = {NULL};
//
//         if (hFindFile == NULL || hFindFile == INVALID_HANDLE_VALUE ||
//             FindDataHandle->Type != FindFile)
//         {
//             SetLastError(ERROR_INVALID_HANDLE);
//             return FALSE;
//         }
//
//         RtlEnterCriticalSection(&FindDataHandle->Lock);
//
//         FindFileData = FindDataHandle->u.FindFileData;
//         InfoLevel = FindFileData->InfoLevel;
//
//         do
//         {
//             if (FindFileData->NextDirInfo.DirInfo == NULL)
//             {
//                 Status = NtQueryDirectoryFile(FindFileData->Handle,
//                                               NULL, NULL, NULL,
//                                               &IoStatusBlock,
//                                               &FindFileData->Buffer,
//                                               sizeof(FindFileData->Buffer),
//                                               (InfoLevel == FindExInfoStandard
//                                                           ? FileBothDirectoryInformation
//                                                           : FileFullDirectoryInformation),
//                                               FALSE,
//                                               NULL, /* Use the file pattern from the first call */
//                                               FALSE);
//                 if (Status == STATUS_BUFFER_OVERFLOW)
//                 {
//                     FindFileData->HasMoreData = TRUE;
//                     Status = STATUS_SUCCESS;
//                 }
//                 else
//                 {
//                     if (!NT_SUCCESS(Status)) break;
//                     FindFileData->HasMoreData = FALSE;
//                 }
//
//                 FindFileData->NextDirInfo.DirInfo = &FindFileData->Buffer;
//             }
//
//             DirInfo = FindFileData->NextDirInfo;
//
//             if (DirInfo.FullDirInfo->NextEntryOffset != 0)
//             {
//                 ULONG_PTR BufferEnd = (ULONG_PTR)&FindFileData->Buffer + sizeof(FindFileData->Buffer);
//                 PWSTR pFileName;
//
//                 NextDirInfo.DirInfo = FindFileData->NextDirInfo.DirInfo =
//                     (PVOID)((ULONG_PTR)DirInfo.DirInfo + DirInfo.FullDirInfo->NextEntryOffset);
//
//                 pFileName = (InfoLevel == FindExInfoStandard
//                                         ? NextDirInfo.BothDirInfo->FileName
//                                         : NextDirInfo.FullDirInfo->FileName);
//
//                 /* Be paranoid and make sure that the next entry is completely there */
//                 if (BufferEnd < (ULONG_PTR)NextDirInfo.DirInfo ||
//                     BufferEnd < (ULONG_PTR)&NextDirInfo.FullDirInfo->FileNameLength + sizeof(NextDirInfo.FullDirInfo->FileNameLength) ||
//                     BufferEnd <= (ULONG_PTR)((ULONG_PTR)pFileName + NextDirInfo.FullDirInfo->FileNameLength))
//                 {
//                     FindFileData->NextDirInfo.DirInfo = NULL;
//                 }
//             }
//             else
//             {
//                 FindFileData->NextDirInfo.DirInfo = NULL;
//             }
//
//             if ((FindFileData->SearchOp != FindExSearchLimitToDirectories) ||
//                 (DirInfo.FullDirInfo->FileAttributes & FILE_ATTRIBUTE_DIRECTORY))
//             {
//                 FoundFile = DirInfo;
//             }
//         } while ( FoundFile.DirInfo == NULL && (FindFileData->NextDirInfo.DirInfo || FindFileData->HasMoreData) );
//
//         if (FoundFile.DirInfo != NULL)
//         {
//             /* Return the information */
//             CopyFindData(lpFindFileData, InfoLevel, FoundFile);
//         }
//
//         RtlLeaveCriticalSection(&FindDataHandle->Lock);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//     else if (FoundFile.DirInfo == NULL)
//     {
//         SetLastError(ERROR_NO_MORE_FILES);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindClose(HANDLE hFindFile)
// {
//     TRACE("FindClose(hFindFile %p)\n", hFindFile);
//
//     if (hFindFile == FIND_DEVICE_HANDLE)
//         return TRUE;
//
//     if (!hFindFile || hFindFile == INVALID_HANDLE_VALUE)
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Protect with SEH against closing attempts on invalid handles. */
//     _SEH2_TRY
//     {
//         PFIND_DATA_HANDLE FindDataHandle = (PFIND_DATA_HANDLE)hFindFile;
//
//         switch (FindDataHandle->Type)
//         {
//             case FindFile:
//             {
//                 RtlEnterCriticalSection(&FindDataHandle->Lock);
//                 NtClose(FindDataHandle->u.FindFileData->Handle);
//                 RtlLeaveCriticalSection(&FindDataHandle->Lock);
//                 RtlDeleteCriticalSection(&FindDataHandle->Lock);
//                 break;
//             }
//
//             case FindStream:
//             {
//                 RtlEnterCriticalSection(&FindDataHandle->Lock);
//                 if (FindDataHandle->u.FindStreamData->FileStreamInfo != NULL)
//                 {
//                     RtlFreeHeap(RtlGetProcessHeap(), 0,
//                                 FindDataHandle->u.FindStreamData->FileStreamInfo);
//                 }
//                 RtlLeaveCriticalSection(&FindDataHandle->Lock);
//                 RtlDeleteCriticalSection(&FindDataHandle->Lock);
//                 break;
//             }
//
//             default:
//             {
//                 SetLastError(ERROR_INVALID_HANDLE);
//                 _SEH2_YIELD(return FALSE);
//             }
//         }
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         BaseSetLastNTError(_SEH2_GetExceptionCode());
//         _SEH2_YIELD(return FALSE);
//     }
//     _SEH2_END;
//
//     return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// HANDLE
// WINAPI
// FindFirstFileExA(IN LPCSTR lpFileName,
//                  IN FINDEX_INFO_LEVELS fInfoLevelId,
//                  OUT LPVOID lpFindFileData,
//                  IN FINDEX_SEARCH_OPS fSearchOp,
//                  LPVOID lpSearchFilter,
//                  IN DWORD dwAdditionalFlags)
// {
//     HANDLE hSearch;
//     NTSTATUS Status;
//     ANSI_STRING Ansi;
//     UNICODE_STRING UTF8;
//     PUNICODE_STRING lpFileNameW;
//     WIN32_FIND_DATAW FindFileDataW;
//     LPWIN32_FIND_DATAA lpFindFileDataA = (LPWIN32_FIND_DATAA)lpFindFileData;
//
//     if ((fInfoLevelId != FindExInfoStandard && fInfoLevelId != FindExInfoBasic) ||
//         fSearchOp == FindExSearchLimitToDevices ||
//         dwAdditionalFlags & ~FIND_FIRST_EX_CASE_SENSITIVE /* only supported flag for now */)
//     {
//         SetLastError(fSearchOp == FindExSearchLimitToDevices
//                                 ? ERROR_NOT_SUPPORTED
//                                 : ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     lpFileNameW = Basep8BitStringToStaticUnicodeString(lpFileName);
//     if (!lpFileNameW) return INVALID_HANDLE_VALUE;
//
//     hSearch = FindFirstFileExW(lpFileNameW->Buffer,
//                                fInfoLevelId,
//                                &FindFileDataW,
//                                fSearchOp,
//                                lpSearchFilter,
//                                dwAdditionalFlags);
//     if (hSearch == INVALID_HANDLE_VALUE) return INVALID_HANDLE_VALUE;
//
//     RtlCopyMemory(lpFindFileDataA,
//                   &FindFileDataW,
//                   FIELD_OFFSET(WIN32_FIND_DATAA, cFileName));
//
//     RtlInitUnicodeString(&UTF8, FindFileDataW.cFileName);
//     Ansi.Buffer = lpFindFileDataA->cFileName;
//     Ansi.Length = 0;
//     Ansi.MaximumLength = sizeof(lpFindFileDataA->cFileName);
//     Status = BasepUnicodeStringTo8BitString(&Ansi, &UTF8, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         FindClose(hSearch);
//         BaseSetLastNTError(Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     if (fInfoLevelId != FindExInfoBasic)
//     {
//         RtlInitUnicodeString(&UTF8, FindFileDataW.cAlternateFileName);
//         Ansi.Buffer = lpFindFileDataA->cAlternateFileName;
//         Ansi.Length = 0;
//         Ansi.MaximumLength = sizeof(lpFindFileDataA->cAlternateFileName);
//         Status = BasepUnicodeStringTo8BitString(&Ansi, &UTF8, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             FindClose(hSearch);
//             BaseSetLastNTError(Status);
//             return INVALID_HANDLE_VALUE;
//         }
//     }
//     else
//     {
//         lpFindFileDataA->cAlternateFileName[0] = ANSI_NULL;
//     }
//
//     return hSearch;
// }
//
//
// /*
//  * @unimplemented
//  */
// HANDLE
// WINAPI
// FindFirstFileExW(IN LPCWSTR lpFileName,
//                  IN FINDEX_INFO_LEVELS fInfoLevelId,
//                  OUT LPVOID lpFindFileData,
//                  IN FINDEX_SEARCH_OPS fSearchOp,
//                  LPVOID lpSearchFilter,
//                  IN DWORD dwAdditionalFlags)
// {
//     UNREFERENCED_PARAMETER(lpSearchFilter);
//
//     TRACE("FindFirstFileExW(lpFileName %S)\n", lpFileName);
//
//     if ((fInfoLevelId != FindExInfoStandard && fInfoLevelId != FindExInfoBasic) ||
//         fSearchOp == FindExSearchLimitToDevices ||
//         dwAdditionalFlags & ~FIND_FIRST_EX_CASE_SENSITIVE /* only supported flag for now */)
//     {
//         SetLastError(fSearchOp == FindExSearchLimitToDevices
//                                 ? ERROR_NOT_SUPPORTED
//                                 : ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     if (fSearchOp == FindExSearchNameMatch ||
//         fSearchOp == FindExSearchLimitToDirectories)
//     {
//         LPWIN32_FIND_DATAW Win32FindData = (LPWIN32_FIND_DATAW)lpFindFileData;
//         PFIND_DATA_HANDLE FindDataHandle;
//         PFIND_FILE_DATA FindFileData;
//
//         UNICODE_STRING NtPath, FilePattern, FileName;
//         PWSTR NtPathBuffer;
//         RTL_RELATIVE_NAME_U RelativePath;
//         ULONG DeviceNameInfo = 0;
//
//         NTSTATUS Status;
//         OBJECT_ATTRIBUTES ObjectAttributes;
//         IO_STATUS_BLOCK IoStatusBlock;
//         HANDLE hDirectory = NULL;
//
//         BOOLEAN HadADot = FALSE;
//
//         /*
//          * May represent many FILE_BOTH_DIR_INFORMATION
//          * or many FILE_FULL_DIR_INFORMATION structures.
//          * NOTE: NtQueryDirectoryFile requires the buffer to be ULONG-aligned
//          */
//         DECLSPEC_ALIGN(4) BYTE DirectoryInfo[FIND_DATA_SIZE];
//         DIR_INFORMATION DirInfo = { .DirInfo = &DirectoryInfo };
//
//         RtlInitUnicodeString(&FileName, lpFileName);
//         if (FileName.Length != 0 && FileName.Buffer[FileName.Length / sizeof(WCHAR) - 1] == L'.')
//         {
//             HadADot = TRUE;
//         }
//
//         if (!RtlDosPathNameToNtPathName_U(lpFileName,
//                                           &NtPath,
//                                           (PCWSTR*)&FilePattern.Buffer,
//                                           &RelativePath))
//         {
//             SetLastError(ERROR_PATH_NOT_FOUND);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         DPRINT("lpFileName = '%S'\n", lpFileName);
//         DPRINT("FilePattern.Buffer = '%S'\n", FilePattern.Buffer);
//         DPRINT("RelativePath.RelativeName = '%wZ'\n", &RelativePath.RelativeName);
//         DPRINT("NtPath.Buffer = '%S'\n", NtPath.Buffer);
//         DPRINT("NtPath - Before = '%wZ'\n", &NtPath);
//
//         /* Save the buffer pointer for later, we need to free it! */
//         NtPathBuffer = NtPath.Buffer;
//
//         /*
//          * Contrary to what Windows does, check NOW whether or not
//          * lpFileName is a DOS driver. Therefore we don't have to
//          * write broken code to check that.
//          */
//         if (!FilePattern.Buffer || !*FilePattern.Buffer)
//         {
//             /* No file pattern specified, or DOS device */
//
//             DeviceNameInfo = RtlIsDosDeviceName_U(lpFileName);
//             if (DeviceNameInfo != 0)
//             {
//                 RtlReleaseRelativeName(&RelativePath);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer);
//
//                 /* OK, it's really a DOS device */
//                 CopyDeviceFindData(Win32FindData, lpFileName, DeviceNameInfo);
//                 return FIND_DEVICE_HANDLE;
//             }
//         }
//
//         /* A file pattern was specified, or it was not a DOS device */
//
//         /* If there is a file pattern then determine its length */
//         if (FilePattern.Buffer != NULL)
//         {
//             FilePattern.Length = NtPath.Length -
//                 (USHORT)((ULONG_PTR)FilePattern.Buffer - (ULONG_PTR)NtPath.Buffer);
//         }
//         else
//         {
//             FilePattern.Length = 0;
//         }
//         FilePattern.MaximumLength = FilePattern.Length;
//
//         if (RelativePath.RelativeName.Length != 0 &&
//             RelativePath.RelativeName.Buffer != FilePattern.Buffer)
//         {
//             if (FilePattern.Buffer != NULL)
//             {
//                 /* This is a relative path to RelativePath.ContainingDirectory, adjust NtPath! */
//                 NtPath.Length = NtPath.MaximumLength =
//                     (USHORT)((ULONG_PTR)FilePattern.Buffer - (ULONG_PTR)RelativePath.RelativeName.Buffer);
//                 NtPath.Buffer = RelativePath.RelativeName.Buffer;
//             }
//         }
//         else
//         {
//             /* This is an absolute path, NtPath receives the full path */
//             RelativePath.ContainingDirectory = NULL;
//             if (FilePattern.Buffer != NULL)
//             {
//                 NtPath.Length = NtPath.MaximumLength =
//                     (USHORT)((ULONG_PTR)FilePattern.Buffer - (ULONG_PTR)NtPath.Buffer);
//             }
//         }
//
//         DPRINT("NtPath - After = '%wZ'\n", &NtPath);
//         DPRINT("FilePattern = '%wZ'\n", &FilePattern);
//         DPRINT("RelativeTo = 0x%p\n", RelativePath.ContainingDirectory);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &NtPath,
//                                    (dwAdditionalFlags & FIND_FIRST_EX_CASE_SENSITIVE) ? 0 : OBJ_CASE_INSENSITIVE,
//                                    RelativePath.ContainingDirectory,
//                                    NULL);
//
//         Status = NtOpenFile(&hDirectory,
//                             FILE_LIST_DIRECTORY | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
//
//         if (!NT_SUCCESS(Status))
//         {
//             RtlReleaseRelativeName(&RelativePath);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer);
//
//             /* Adjust the last error codes */
//             if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 Status = STATUS_OBJECT_PATH_NOT_FOUND;
//             else if (Status == STATUS_OBJECT_TYPE_MISMATCH)
//                 Status = STATUS_OBJECT_PATH_NOT_FOUND;
//
//             BaseSetLastNTError(Status);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         /*
//          * Fail if there is not any file pattern,
//          * since we are not looking for a device.
//          */
//         if (FilePattern.Length == 0)
//         {
//             NtClose(hDirectory);
//             RtlReleaseRelativeName(&RelativePath);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer);
//
//             SetLastError(ERROR_FILE_NOT_FOUND);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         /* Change pattern: "*.*" --> "*" */
//         if (FilePattern.Length == 6 &&
//             RtlCompareMemory(FilePattern.Buffer, L"*.*", 6) == 6)
//         {
//             FilePattern.Length = 2;
//         }
//         else
//         {
//             /* Translate wildcard from "real" world to DOS world for lower interpretation */
//             USHORT PatternIndex = 0;
//             while (PatternIndex < FilePattern.Length / sizeof(WCHAR))
//             {
//                 if (PatternIndex > 0)
//                 {
//                     if (FilePattern.Buffer[PatternIndex] == L'.' &&
//                         FilePattern.Buffer[PatternIndex - 1] == L'*')
//                     {
//                         FilePattern.Buffer[PatternIndex - 1] = L'<';
//                     }
//                 }
//
//                 if (FilePattern.Buffer[PatternIndex] == L'?')
//                 {
//                     FilePattern.Buffer[PatternIndex] = L'>';
//                     if (PatternIndex > 0)
//                     {
//                         if (FilePattern.Buffer[PatternIndex - 1] == L'.')
//                         {
//                             FilePattern.Buffer[PatternIndex - 1] = L'\"';
//                         }
//                     }
//                 }
//                 else if (FilePattern.Buffer[PatternIndex] == L'*')
//                 {
//                     if (PatternIndex > 0)
//                     {
//                         if (FilePattern.Buffer[PatternIndex - 1] == L'.')
//                         {
//                             FilePattern.Buffer[PatternIndex - 1] = L'\"';
//                         }
//                     }
//                 }
//
//                 PatternIndex++;
//             }
//
//             /* Handle partial wc if our last dot was eaten */
//             if (HadADot)
//             {
//                 if (FilePattern.Buffer[FilePattern.Length / sizeof(WCHAR) - 1] == L'*')
//                 {
//                     FilePattern.Buffer[FilePattern.Length / sizeof(WCHAR) - 1] = L'<';
//                 }
//             }
//         }
//
//         Status = NtQueryDirectoryFile(hDirectory,
//                                       NULL, NULL, NULL,
//                                       &IoStatusBlock,
//                                       DirInfo.DirInfo, // == &DirectoryInfo
//                                       sizeof(DirectoryInfo),
//                                       (fInfoLevelId == FindExInfoStandard
//                                                      ? FileBothDirectoryInformation
//                                                      : FileFullDirectoryInformation),
//                                       TRUE, /* Return a single entry */
//                                       &FilePattern,
//                                       TRUE);
//
//         RtlReleaseRelativeName(&RelativePath);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathBuffer);
//
//         if (!NT_SUCCESS(Status))
//         {
//             NtClose(hDirectory);
//             BaseSetLastNTError(Status);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         ASSERT(DirInfo.FullDirInfo->NextEntryOffset == 0);
//
//         /* Return the information */
//         CopyFindData(Win32FindData, fInfoLevelId, DirInfo);
//
//         /*
//          * Initialization of the search handle.
//          */
//         FindDataHandle = RtlAllocateHeap(RtlGetProcessHeap(),
//                                          HEAP_ZERO_MEMORY,
//                                          sizeof(FIND_DATA_HANDLE) +
//                                              sizeof(FIND_FILE_DATA));
//         if (!FindDataHandle)
//         {
//             NtClose(hDirectory);
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         FindDataHandle->Type = FindFile;
//         FindDataHandle->u.FindFileData = (PFIND_FILE_DATA)(FindDataHandle + 1);
//         FindFileData = FindDataHandle->u.FindFileData;
//
//         FindFileData->Handle = hDirectory;
//         FindFileData->InfoLevel = fInfoLevelId;
//         FindFileData->SearchOp = fSearchOp;
//         FindFileData->HasMoreData = FALSE;
//         FindFileData->NextDirInfo.DirInfo = NULL;
//
//         /* The critical section must always be initialized */
//         Status = RtlInitializeCriticalSection(&FindDataHandle->Lock);
//         if (!NT_SUCCESS(Status))
//         {
//             NtClose(hDirectory);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle);
//
//             BaseSetLastNTError(Status);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         return (HANDLE)FindDataHandle;
//     }
//     else
//     {
//         SetLastError(ERROR_NOT_SUPPORTED);
//         return INVALID_HANDLE_VALUE;
//     }
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// FindFirstStreamW(IN LPCWSTR lpFileName,
//                  IN STREAM_INFO_LEVELS InfoLevel,
//                  OUT LPVOID lpFindStreamData,
//                  IN DWORD dwFlags)
// {
//     PFIND_DATA_HANDLE FindDataHandle = NULL;
//     PFIND_STREAM_DATA FindStreamData;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING NtFilePath;
//     HANDLE FileHandle = NULL;
//     NTSTATUS Status;
//     ULONG BufferSize = 0;
//
//     if (dwFlags != 0 || InfoLevel != FindStreamInfoStandard ||
//         lpFindStreamData == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     /* Validate and translate the filename */
//     if (!RtlDosPathNameToNtPathName_U(lpFileName,
//                                       &NtFilePath,
//                                       NULL, NULL))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     /* Open the file */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtFilePath,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtCreateFile(&FileHandle,
//                           0,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL, 0,
//                           FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           FILE_OPEN,
//                           0, NULL, 0);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /*
//      * Initialization of the search handle.
//      */
//     FindDataHandle = RtlAllocateHeap(RtlGetProcessHeap(),
//                                      HEAP_ZERO_MEMORY,
//                                      sizeof(FIND_DATA_HANDLE) +
//                                          sizeof(FIND_STREAM_DATA));
//     if (!FindDataHandle)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Cleanup;
//     }
//
//     FindDataHandle->Type = FindStream;
//     FindDataHandle->u.FindStreamData = (PFIND_STREAM_DATA)(FindDataHandle + 1);
//     FindStreamData = FindDataHandle->u.FindStreamData;
//
//     FindStreamData->InfoLevel = InfoLevel;
//     FindStreamData->FileStreamInfo = NULL;
//     FindStreamData->CurrentInfo = NULL;
//
//     /* The critical section must always be initialized */
//     Status = RtlInitializeCriticalSection(&FindDataHandle->Lock);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle);
//         goto Cleanup;
//     }
//
//     /* Capture all information about the streams */
//     do
//     {
//         BufferSize += 0x1000;
//
//         if (FindStreamData->FileStreamInfo == NULL)
//         {
//             FindStreamData->FileStreamInfo = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                               HEAP_ZERO_MEMORY,
//                                                               BufferSize);
//             if (FindStreamData->FileStreamInfo == NULL)
//             {
//                 Status = STATUS_NO_MEMORY;
//                 break;
//             }
//         }
//         else
//         {
//             PFILE_STREAM_INFORMATION pfsi;
//
//             pfsi = RtlReAllocateHeap(RtlGetProcessHeap(),
//                                      0, // HEAP_ZERO_MEMORY,
//                                      FindStreamData->FileStreamInfo,
//                                      BufferSize);
//             if (pfsi == NULL)
//             {
//                 Status = STATUS_NO_MEMORY;
//                 break;
//             }
//
//             FindStreamData->FileStreamInfo = pfsi;
//         }
//
//         Status = NtQueryInformationFile(FileHandle,
//                                         &IoStatusBlock,
//                                         FindStreamData->FileStreamInfo,
//                                         BufferSize,
//                                         FileStreamInformation);
//
//     } while (Status == STATUS_BUFFER_TOO_SMALL);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Select the first stream and return the information */
//         FindStreamData->CurrentInfo = FindStreamData->FileStreamInfo;
//         CopyStreamData(FindStreamData, lpFindStreamData);
//
//         /* All done */
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         if (FindStreamData->FileStreamInfo)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, FindStreamData->FileStreamInfo);
//         }
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FindDataHandle);
//     }
//
// Cleanup:
//     if (FileHandle) NtClose(FileHandle);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NtFilePath.Buffer);
//
//     if (NT_SUCCESS(Status))
//     {
//         return (HANDLE)FindDataHandle;
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//         return INVALID_HANDLE_VALUE;
//     }
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindNextStreamW(IN HANDLE hFindStream,
//                 OUT LPVOID lpFindStreamData)
// {
//     PFIND_DATA_HANDLE FindDataHandle = (PFIND_DATA_HANDLE)hFindStream;
//     PFIND_STREAM_DATA FindStreamData;
//
//     if (hFindStream == NULL || hFindStream == INVALID_HANDLE_VALUE ||
//         FindDataHandle->Type != FindStream)
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     RtlEnterCriticalSection(&FindDataHandle->Lock);
//
//     FindStreamData = FindDataHandle->u.FindStreamData;
//
//     /* Select next stream if possible */
//     if (FindStreamData->CurrentInfo->NextEntryOffset != 0)
//     {
//         FindStreamData->CurrentInfo = (PFILE_STREAM_INFORMATION)((ULONG_PTR)FindStreamData->CurrentInfo +
//                                                                  FindStreamData->CurrentInfo->NextEntryOffset);
//
//         /* Return the information */
//         CopyStreamData(FindStreamData, lpFindStreamData);
//
//         RtlLeaveCriticalSection(&FindDataHandle->Lock);
//         return TRUE;
//     }
//     else
//     {
//         RtlLeaveCriticalSection(&FindDataHandle->Lock);
//
//         SetLastError(ERROR_HANDLE_EOF);
//         return FALSE;
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
