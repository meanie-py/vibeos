//! vista.c — QueryFullProcessImageName, CreateSymbolicLink, GetFinalPathNameByHandleA,
//! recovery/bandwidth/MUI stubs.

use crate::common::*;
use core::ffi::c_void;
use core::mem::offset_of;

const FILE_DIRECTORY_FILE: ULONG = 0x0000_0001;
const FILE_NON_DIRECTORY_FILE: ULONG = 0x0000_0040;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x0000_0020;
const FILE_OPEN_REPARSE_POINT: ULONG = 0x0020_0000;
const FILE_ATTRIBUTE_NORMAL: ULONG = 0x0000_0080;
const FILE_CREATE: ULONG = 0x0000_0002;
const FILE_WRITE_ATTRIBUTES: ACCESS_MASK = 0x0000_0100;
const DELETE: ACCESS_MASK = 0x0001_0000;
const SYNCHRONIZE: ACCESS_MASK = 0x0010_0000;

// ===========================================================================
// QueryFullProcessImageNameW / A
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn QueryFullProcessImageNameW(
    hProcess: HANDLE,
    dwFlags: DWORD,
    lpExeName: LPWSTR,
    lpdwSize: PDWORD,
) -> BOOL {
    if dwFlags & !PROCESS_NAME_NATIVE != 0 {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER_2);
        return 0;
    }

    let dwBufferSize =
        core::mem::size_of::<UNICODE_STRING>() as DWORD + *lpdwSize * core::mem::size_of::<WCHAR>() as DWORD;
    let pBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, dwBufferSize as SIZE_T) as PUNICODE_STRING;
    if pBuffer.is_null() {
        BaseSetLastNTError(STATUS_NO_MEMORY);
        return 0;
    }

    let mut bRet: BOOL = 0;
    let mut Status = NtQueryInformationProcess(
        hProcess,
        if dwFlags & PROCESS_NAME_NATIVE != 0 {
            ProcessImageFileName
        } else {
            ProcessImageFileNameWin32
        },
        pBuffer as PVOID,
        dwBufferSize,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        let dwCch = (*pBuffer).Length as DWORD / core::mem::size_of::<WCHAR>() as DWORD;
        if dwCch >= *lpdwSize {
            BaseSetLastNTError(STATUS_BUFFER_TOO_SMALL);
        } else {
            RtlCopyMemory(
                lpExeName as PVOID,
                (*pBuffer).Buffer as *const c_void,
                (*pBuffer).Length as SIZE_T,
            );
            *lpExeName.add(dwCch as usize) = UNICODE_NULL;
            *lpdwSize = dwCch;
            bRet = TRUE as BOOL;
        }
    } else {
        if Status == STATUS_INFO_LENGTH_MISMATCH {
            Status = STATUS_BUFFER_TOO_SMALL;
        }
        BaseSetLastNTError(Status);
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, pBuffer as PVOID);
    bRet
}

#[no_mangle]
pub unsafe extern "system" fn QueryFullProcessImageNameA(
    hProcess: HANDLE,
    dwFlags: DWORD,
    lpExeName: LPSTR,
    lpdwSize: PDWORD,
) -> BOOL {
    let mut dwSize = *lpdwSize;
    let pszFullName = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (dwSize as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PWSTR;
    if pszFullName.is_null() {
        BaseSetLastNTError(STATUS_NO_MEMORY);
        return 0;
    }

    let mut bRet: BOOL = 0;
    if QueryFullProcessImageNameW(hProcess, dwFlags, pszFullName, &mut dwSize) != 0 {
        let iCch = WideCharToMultiByte(
            CP_ACP,
            WC_NO_BEST_FIT_CHARS,
            pszFullName,
            (dwSize + 1) as core::ffi::c_int,
            lpExeName,
            *lpdwSize as core::ffi::c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if iCch != 0 {
            *lpdwSize = (iCch - 1) as DWORD;
            bRet = TRUE as BOOL;
        }
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, pszFullName as PVOID);
    bRet
}

// ===========================================================================
// Application recovery / restart — unimplemented stubs
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn GetApplicationRecoveryCallback(
    _hProcess: HANDLE,
    _pRecoveryCallback: PVOID,
    _ppvParameter: *mut PVOID,
    _dwPingInterval: PDWORD,
    _dwFlags: PDWORD,
) -> HRESULT {
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn GetApplicationRestart(
    _hProcess: HANDLE,
    _pwzCommandline: PWSTR,
    _pcchSize: PDWORD,
    _pdwFlags: PDWORD,
) -> HRESULT {
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ApplicationRecoveryFinished(_bSuccess: BOOL) {}
#[no_mangle]
pub unsafe extern "system" fn ApplicationRecoveryInProgress(_pbCancelled: PBOOL) -> HRESULT {
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn RegisterApplicationRecoveryCallback(
    _pRecoveryCallback: PVOID,
    _pvParameter: PVOID,
    _dwPingInterval: DWORD,
    _dwFlags: DWORD,
) -> HRESULT {
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn RegisterApplicationRestart(
    _pwzCommandline: PCWSTR,
    _dwFlags: DWORD,
) -> HRESULT {
    E_FAIL
}

// ===========================================================================
// CreateSymbolicLinkW / A
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn CreateSymbolicLinkW(
    lpSymlinkFileName: LPCWSTR,
    mut lpTargetFileName: LPCWSTR,
    dwFlags: DWORD,
) -> BOOLEAN {
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hSymlink: HANDLE = core::ptr::null_mut();
    let mut SymlinkFileName: UNICODE_STRING = core::mem::zeroed();
    let mut TargetFileName: UNICODE_STRING = core::mem::zeroed();
    let mut bAllocatedTarget: BOOLEAN = FALSE as BOOLEAN;
    let mut bRelativePath: BOOLEAN = FALSE as BOOLEAN;
    let mut lpTargetFullFileName: LPWSTR = core::ptr::null_mut();
    let pReparseData;
    let dwCreateOptions: ULONG;
    let mut dwErr: DWORD;

    if lpSymlinkFileName.is_null()
        || lpTargetFileName.is_null()
        || (dwFlags | SYMBOLIC_LINK_FLAG_DIRECTORY) != SYMBOLIC_LINK_FLAG_DIRECTORY
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOLEAN;
    }

    dwCreateOptions = if dwFlags & SYMBOLIC_LINK_FLAG_DIRECTORY != 0 {
        FILE_DIRECTORY_FILE
    } else {
        FILE_NON_DIRECTORY_FILE
    };

    'cleanup: loop {
        match RtlDetermineDosPathNameType_U(lpTargetFileName) {
            RtlPathTypeUnknown | RtlPathTypeRooted | RtlPathTypeRelative => {
                bRelativePath = TRUE as BOOLEAN;
                RtlInitUnicodeString(&mut TargetFileName, lpTargetFileName);
            }
            other => {
                if other == RtlPathTypeDriveRelative {
                    let mut FilePart: LPWSTR = core::ptr::null_mut();
                    let cchTargetFullFileName =
                        GetFullPathNameW(lpTargetFileName, 0, core::ptr::null_mut(), &mut FilePart);
                    if cchTargetFullFileName == 0 {
                        dwErr = GetLastError();
                        break 'cleanup;
                    }
                    lpTargetFullFileName = RtlAllocateHeap(
                        RtlGetProcessHeap(),
                        0,
                        (cchTargetFullFileName as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
                    ) as LPWSTR;
                    if lpTargetFullFileName.is_null() {
                        dwErr = ERROR_NOT_ENOUGH_MEMORY;
                        break 'cleanup;
                    }
                    if GetFullPathNameW(
                        lpTargetFileName,
                        cchTargetFullFileName,
                        lpTargetFullFileName,
                        &mut FilePart,
                    ) == 0
                    {
                        dwErr = GetLastError();
                        break 'cleanup;
                    }
                    lpTargetFileName = lpTargetFullFileName;
                }
                // UncAbsolute / DriveAbsolute / LocalDevice / RootLocalDevice / default
                // (+ DriveRelative fallthrough)
                if RtlDosPathNameToNtPathName_U(
                    lpTargetFileName,
                    &mut TargetFileName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) == 0
                {
                    // NOTE: faithful to the C (sets bAllocatedTarget on the failure path).
                    bAllocatedTarget = TRUE as BOOLEAN;
                    dwErr = ERROR_INVALID_PARAMETER;
                    break 'cleanup;
                }
            }
        }

        let cbPrintName: usize = wcslen(lpTargetFileName) * core::mem::size_of::<WCHAR>();
        let cbReparseData: usize =
            offset_of!(REPARSE_DATA_BUFFER, __bindgen_anon_1.SymbolicLinkReparseBuffer.PathBuffer)
                + TargetFileName.Length as usize
                + cbPrintName;
        pReparseData =
            RtlAllocateHeap(RtlGetProcessHeap(), 0, cbReparseData as SIZE_T) as PREPARSE_DATA_BUFFER;
        if pReparseData.is_null() {
            dwErr = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        let mut pBufTail = (*pReparseData)
            .__bindgen_anon_1
            .SymbolicLinkReparseBuffer
            .PathBuffer
            .as_mut_ptr() as *mut u8;

        (*pReparseData).ReparseTag = IO_REPARSE_TAG_SYMLINK as ULONG;
        (*pReparseData).ReparseDataLength = (cbReparseData as USHORT).wrapping_sub(
            offset_of!(REPARSE_DATA_BUFFER, __bindgen_anon_1) as USHORT,
        );
        (*pReparseData).Reserved = 0;

        (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameOffset = 0;
        (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameLength =
            TargetFileName.Length;
        pBufTail = pBufTail.add(
            (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameOffset as usize,
        );
        RtlCopyMemory(
            pBufTail as PVOID,
            TargetFileName.Buffer as *const c_void,
            TargetFileName.Length as SIZE_T,
        );

        (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameOffset =
            (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.SubstituteNameLength;
        (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameLength =
            cbPrintName as USHORT;
        pBufTail = pBufTail.add(
            (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.PrintNameOffset as usize,
        );
        RtlCopyMemory(pBufTail as PVOID, lpTargetFileName as *const c_void, cbPrintName as SIZE_T);

        (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.Flags = 0;
        if bRelativePath != 0 {
            (*pReparseData).__bindgen_anon_1.SymbolicLinkReparseBuffer.Flags |= 1;
        }

        if RtlDosPathNameToNtPathName_U(
            lpSymlinkFileName,
            &mut SymlinkFileName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            dwErr = ERROR_PATH_NOT_FOUND;
            // pReparseData freed in cleanup below
            return finish_symlink(
                hSymlink,
                &mut SymlinkFileName,
                bAllocatedTarget,
                &mut TargetFileName,
                lpTargetFullFileName,
                pReparseData,
                dwErr,
            );
        }

        ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.ObjectName = &mut SymlinkFileName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;

        let mut Status = NtCreateFile(
            &mut hSymlink,
            FILE_WRITE_ATTRIBUTES | DELETE | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL,
            0,
            FILE_CREATE,
            FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT | dwCreateOptions,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            dwErr = RtlNtStatusToDosError(Status);
            return finish_symlink(
                hSymlink,
                &mut SymlinkFileName,
                bAllocatedTarget,
                &mut TargetFileName,
                lpTargetFullFileName,
                pReparseData,
                dwErr,
            );
        }

        Status = NtFsControlFile(
            hSymlink,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            FSCTL_SET_REPARSE_POINT,
            pReparseData as PVOID,
            cbReparseData as ULONG,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            let mut DispInfo: FILE_DISPOSITION_INFORMATION = core::mem::zeroed();
            DispInfo.DeleteFileA = TRUE as BOOLEAN;
            NtSetInformationFile(
                hSymlink,
                &mut IoStatusBlock,
                &mut DispInfo as *mut FILE_DISPOSITION_INFORMATION as PVOID,
                core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as ULONG,
                FileDispositionInformation,
            );
            dwErr = RtlNtStatusToDosError(Status);
            return finish_symlink(
                hSymlink,
                &mut SymlinkFileName,
                bAllocatedTarget,
                &mut TargetFileName,
                lpTargetFullFileName,
                pReparseData,
                dwErr,
            );
        }

        dwErr = NO_ERROR;
        return finish_symlink(
            hSymlink,
            &mut SymlinkFileName,
            bAllocatedTarget,
            &mut TargetFileName,
            lpTargetFullFileName,
            pReparseData,
            dwErr,
        );
    }

    // 'cleanup break target: pReparseData not yet allocated on these paths
    finish_symlink(
        hSymlink,
        &mut SymlinkFileName,
        bAllocatedTarget,
        &mut TargetFileName,
        lpTargetFullFileName,
        core::ptr::null_mut(),
        dwErr,
    )
}

#[inline]
unsafe fn finish_symlink(
    hSymlink: HANDLE,
    SymlinkFileName: PUNICODE_STRING,
    bAllocatedTarget: BOOLEAN,
    TargetFileName: PUNICODE_STRING,
    lpTargetFullFileName: LPWSTR,
    pReparseData: PREPARSE_DATA_BUFFER,
    dwErr: DWORD,
) -> BOOLEAN {
    if !hSymlink.is_null() {
        NtClose(hSymlink);
    }
    RtlFreeUnicodeString(SymlinkFileName);
    if bAllocatedTarget != 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*TargetFileName).Buffer as PVOID);
    }
    if !lpTargetFullFileName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, lpTargetFullFileName as PVOID);
    }
    if !pReparseData.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pReparseData as PVOID);
    }
    if dwErr != 0 {
        SetLastError(dwErr);
        return FALSE as BOOLEAN;
    }
    TRUE as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn CreateSymbolicLinkA(
    lpSymlinkFileName: LPCSTR,
    lpTargetFileName: LPCSTR,
    dwFlags: DWORD,
) -> BOOLEAN {
    if lpSymlinkFileName.is_null() || lpTargetFileName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOLEAN;
    }

    let SymlinkW = FilenameA2W(lpSymlinkFileName, FALSE as BOOL);
    if SymlinkW.is_null() {
        return FALSE as BOOLEAN;
    }
    let TargetW = FilenameA2W(lpTargetFileName, TRUE as BOOL);
    if TargetW.is_null() {
        return FALSE as BOOLEAN;
    }

    let Ret = CreateSymbolicLinkW(SymlinkW, TargetW, dwFlags);

    RtlFreeHeap(RtlGetProcessHeap(), 0, SymlinkW as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, TargetW as PVOID);

    Ret
}

// ===========================================================================
// GetFinalPathNameByHandleA
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn GetFinalPathNameByHandleA(
    hFile: HANDLE,
    lpszFilePath: LPSTR,
    cchFilePath: DWORD,
    dwFlags: DWORD,
) -> DWORD {
    let mut FilePathW: [WCHAR; 260] = [0; 260];
    let mut FilePathU: UNICODE_STRING = core::mem::zeroed();

    if cchFilePath != 0 && cchFilePath as usize > FilePathW.len() {
        FilePathU.Length = 0;
        FilePathU.MaximumLength = (cchFilePath as USHORT) * core::mem::size_of::<WCHAR>() as USHORT;
        FilePathU.Buffer =
            RtlAllocateHeap(RtlGetProcessHeap(), 0, FilePathU.MaximumLength as SIZE_T) as PWSTR;
        if FilePathU.Buffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    } else {
        FilePathU.Length = 0;
        FilePathU.MaximumLength = core::mem::size_of_val(&FilePathW) as USHORT;
        FilePathU.Buffer = FilePathW.as_mut_ptr();
    }

    let PrevLastError = GetLastError();
    SetLastError(ERROR_SUCCESS);

    let mut Ret =
        crate::finalpath::GetFinalPathNameByHandleW(hFile, FilePathU.Buffer, cchFilePath, dwFlags);

    if GetLastError() == ERROR_SUCCESS {
        SetLastError(PrevLastError);
        Ret = FilenameU2A_FitOrFail(lpszFilePath, cchFilePath as INT, &mut FilePathU);
    }

    if FilePathU.Buffer != FilePathW.as_mut_ptr() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FilePathU.Buffer as PVOID);
    }

    Ret
}

// ===========================================================================
// Bandwidth reservation / OpenFileById — unimplemented stubs
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn SetFileBandwidthReservation(
    _hFile: HANDLE,
    _nPeriodMilliseconds: DWORD,
    _nBytesPerPeriod: DWORD,
    _bDiscardable: BOOL,
    _lpTransferSize: LPDWORD,
    _lpNumOutstandingRequests: LPDWORD,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn GetFileBandwidthReservation(
    _hFile: HANDLE,
    _lpPeriodMilliseconds: LPDWORD,
    _lpBytesPerPeriod: LPDWORD,
    _pDiscardable: LPBOOL,
    _lpTransferSize: LPDWORD,
    _lpNumOutstandingRequests: LPDWORD,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn OpenFileById(
    _hFile: HANDLE,
    _lpFileID: PVOID,
    _dwDesiredAccess: DWORD,
    _dwShareMode: DWORD,
    _lpSecurityAttributes: PVOID,
    _dwFlags: DWORD,
) -> HANDLE {
    (-1isize) as HANDLE // INVALID_HANDLE_VALUE
}

// ===========================================================================
// MUI / UI-language — unimplemented stubs
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn GetFileMUIInfo(
    _dwFlags: DWORD,
    _pcwszFilePath: PCWSTR,
    _pFileMUIInfo: PVOID,
    _pcbFileMUIInfo: *mut DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn GetFileMUIPath(
    _dwFlags: DWORD,
    _pcwszFilePath: PCWSTR,
    _pwszLanguage: PWSTR,
    _pcchLanguage: PULONG,
    _pwszFileMUIPath: PWSTR,
    _pcchFileMUIPath: PULONG,
    _pululEnumerator: *mut ULONGLONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn GetSystemPreferredUILanguages(
    _dwFlags: DWORD,
    _pulNumLanguages: PULONG,
    _pwszLanguagesBuffer: PWSTR,
    _pcchLanguagesBuffer: PULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn GetThreadPreferredUILanguages(
    _dwFlags: DWORD,
    _pulNumLanguages: PULONG,
    _pwszLanguagesBuffer: PWSTR,
    _pcchLanguagesBuffer: PULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn GetThreadUILanguage() -> LANGID {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn GetUILanguageInfo(
    _dwFlags: DWORD,
    _pwmszLanguage: PCWSTR,
    _pwszFallbackLanguages: PWSTR,
    _pcchFallbackLanguages: PDWORD,
    _pdwAttributes: PDWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn GetUserPreferredUILanguages(
    _dwFlags: DWORD,
    _pulNumLanguages: PULONG,
    _pwszLanguagesBuffer: PWSTR,
    _pcchLanguagesBuffer: PULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
#[no_mangle]
pub unsafe extern "system" fn SetThreadPreferredUILanguages(
    _dwFlags: DWORD,
    _pwszLanguagesBuffer: PCWSTR,
    _pulNumLanguages: PULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

// ===========================================================================
// CompareStringOrdinal — added by the 2026-09-20 upstream sync
// ===========================================================================
// Upstream added this to vista.c and exported it from kernel32_vista.spec after our fork.
// Ported here rather than building their vista.c, which this crate already owns.
// Lengths are in CHARACTERS on both sides of the RtlCompareUnicodeStrings call.
use core::ffi::c_int;
// Upstream calls ntdll's RtlCompareUnicodeStrings here, but that export is
// `-version=0x600+` in ntdll.spec and this tree builds ntdll at 0x502, so the importlib
// does not carry it. The comparison itself is small, so it is inlined rather than
// widening ntdll's export surface: ordinal (code-unit) order, optionally upcased, then
// shorter-is-less. RtlUpcaseUnicodeChar is exported unversioned.
extern "system" {
    fn RtlUpcaseUnicodeChar(SourceCharacter: WCHAR) -> WCHAR;
}

unsafe fn compare_unicode_strings(
    s1: *const WCHAR,
    l1: usize,
    s2: *const WCHAR,
    l2: usize,
    ignore_case: bool,
) -> LONG {
    let n = core::cmp::min(l1, l2);
    for i in 0..n {
        let (mut a, mut b) = (*s1.add(i), *s2.add(i));
        if a != b && ignore_case {
            a = RtlUpcaseUnicodeChar(a);
            b = RtlUpcaseUnicodeChar(b);
        }
        if a != b {
            return a as LONG - b as LONG;
        }
    }
    l1 as LONG - l2 as LONG
}

const CSTR_LESS_THAN: c_int = 1;
const CSTR_EQUAL: c_int = 2;
const CSTR_GREATER_THAN: c_int = 3;

#[no_mangle]
pub unsafe extern "system" fn CompareStringOrdinal(
    str1: *const WCHAR,
    mut len1: c_int,
    str2: *const WCHAR,
    mut len2: c_int,
    ignore_case: BOOL,
) -> c_int {
    if str1.is_null() || str2.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if len1 < 0 {
        len1 = lstrlenW(str1);
    }
    if len2 < 0 {
        len2 = lstrlenW(str2);
    }
    let ret = compare_unicode_strings(str1, len1 as usize, str2, len2 as usize, ignore_case != 0);
    if ret < 0 {
        CSTR_LESS_THAN
    } else if ret > 0 {
        CSTR_GREATER_THAN
    } else {
        CSTR_EQUAL
    }
}
