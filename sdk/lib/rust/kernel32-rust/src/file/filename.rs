use ntdll_sys::nt::*;
use crate::prelude::*;
#[allow(unused_imports)]
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

// kernel32-internal helpers + cross-file Win32 exports this file calls.
use crate::k32_externs::{
    RtlCopyMemory, CreateFileW, GetFileAttributesW, IsConsoleHandle,
    CSR_CREATE_API_NUMBER, Basep8BitStringToDynamicUnicodeString,
};
use crate::client::utils::bIsFileApiAnsi;

extern "system" {
    fn Basep8BitStringToStaticUnicodeString(String: LPCSTR) -> PUNICODE_STRING;
}

// FilenameA2W is cdecl in C (fileutils.c / kernel32.h: no WINAPI) → extern "C".
extern "C" {
    fn FilenameA2W(NameA: LPCSTR, alloc: BOOL) -> PWCHAR;
}

// BasepUnicodeStringTo8BitString is a PRTL_CONVERT_STRINGA fn-pointer GLOBAL in C
// (utils.c), called directly — declare it as a data symbol (no @N), not a function.
extern "C" {
    static BasepUnicodeStringTo8BitString:
        unsafe extern "system" fn(PANSI_STRING, PCUNICODE_STRING, BOOLEAN) -> NTSTATUS;
}

// ntstatus.h
pub const STATUS_FILE_IS_A_DIRECTORY: NTSTATUS = 0xC00000BAu32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn GetTempFileNameA(
    lpPathName: LPCSTR,
    lpPrefixString: LPCSTR,
    uUnique: UINT,
    lpTempFileName: LPSTR,
) -> UINT {
    let mut ID: UINT = 0;
    let mut Status: NTSTATUS = 0;
    let mut lpTempFileNameW: LPWSTR = core::ptr::null_mut();
    let mut lpPathNameW: PUNICODE_STRING = core::ptr::null_mut();
    let mut TempFileNameStringA: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut lpPrefixStringW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut TempFileNameStringW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    lpPathNameW = Basep8BitStringToStaticUnicodeString(lpPathName);
    if lpPathNameW.is_null() {
        return 0;
    }

    if Basep8BitStringToDynamicUnicodeString(&mut lpPrefixStringW, lpPrefixString) == 0 {
        return 0;
    }

    lpTempFileNameW = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (MAX_PATH as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPWSTR;
    if lpTempFileNameW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as u32);
        RtlFreeUnicodeString(&mut lpPrefixStringW);
        return 0;
    }

    ID = GetTempFileNameW(
        (*lpPathNameW).Buffer as LPCWSTR,
        lpPrefixStringW.Buffer as LPCWSTR,
        uUnique,
        lpTempFileNameW,
    );
    if ID != 0 {
        RtlInitUnicodeString(&mut TempFileNameStringW, lpTempFileNameW as PCWSTR);
        TempFileNameStringA.Buffer = lpTempFileName as PCHAR;
        TempFileNameStringA.MaximumLength = MAX_PATH as u16;

        Status = BasepUnicodeStringTo8BitString(
            &mut TempFileNameStringA,
            &mut TempFileNameStringW as PCUNICODE_STRING,
            0,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            ID = 0;
        }
    }

    RtlFreeUnicodeString(&mut lpPrefixStringW);
    RtlFreeHeap(RtlGetProcessHeap(), 0, lpTempFileNameW as PVOID);
    return ID;
}

#[no_mangle]
pub unsafe extern "system" fn GetTempFileNameW(
    lpPathName: LPCWSTR,
    lpPrefixString: LPCWSTR,
    uUnique: UINT,
    lpTempFileName: LPWSTR,
) -> UINT {
    let mut Let: PUCHAR = core::ptr::null_mut();
    let mut TempFile: HANDLE = core::ptr::null_mut();
    let mut ID: UINT = 0;
    let mut Num: UINT = 0;
    let mut IDString: [UCHAR; 5] = [0; 5];
    let mut TempFileName: *mut WCHAR = core::ptr::null_mut();
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let GetTempFile: PBASE_GET_TEMP_FILE = &raw mut ApiMessage.Data.GetTempFileRequest;
    let mut FileAttributes: DWORD = 0;
    let mut LastError: DWORD = 0;
    let mut PathNameString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut PrefixString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    static Ext: [WCHAR; 5] = ['.' as WCHAR, 't' as WCHAR, 'm' as WCHAR, 'p' as WCHAR, UNICODE_NULL];

    RtlInitUnicodeString(&raw mut PathNameString, lpPathName);
    if PathNameString.Length == 0
        || *PathNameString.Buffer.offset(
            (PathNameString.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()).wrapping_sub(1) as isize,
        ) != '\\' as WCHAR
    {
        PathNameString.Length = (PathNameString.Length as u16).wrapping_add(core::mem::size_of::<WCHAR>() as u16);
    }

    if (PathNameString.Length as usize)
        > ((MAX_PATH - 3 - 4 - 4 - 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>())
    {
        SetLastError(ERROR_BUFFER_OVERFLOW);
        return 0;
    }

    if lpPathName != lpTempFileName as LPCWSTR {
        memmove(
            lpTempFileName as *mut core::ffi::c_void,
            PathNameString.Buffer as *const core::ffi::c_void,
            PathNameString.Length as usize,
        );
    }

    *lpTempFileName.offset(
        (PathNameString.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()).wrapping_sub(1) as isize,
    ) = UNICODE_NULL;
    FileAttributes = GetFileAttributesW(lpTempFileName);
    if FileAttributes == INVALID_FILE_ATTRIBUTES {
        *lpTempFileName.offset(
            (PathNameString.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()).wrapping_sub(1) as isize,
        ) = '\\' as WCHAR;
        *lpTempFileName.offset(
            (PathNameString.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
        ) = UNICODE_NULL;
        FileAttributes = GetFileAttributesW(lpTempFileName);
        if FileAttributes == INVALID_FILE_ATTRIBUTES {
            SetLastError(ERROR_DIRECTORY);
            return 0;
        }
    }

    if (FileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
        SetLastError(ERROR_DIRECTORY);
        return 0;
    }

    *lpTempFileName.offset(
        (PathNameString.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()).wrapping_sub(1) as isize,
    ) = '\\' as WCHAR;
    RtlInitUnicodeString(&raw mut PrefixString, lpPrefixString);
    if (PrefixString.Length as usize) > 3usize.wrapping_mul(core::mem::size_of::<WCHAR>()) {
        PrefixString.Length = 3usize.wrapping_mul(core::mem::size_of::<WCHAR>()) as u16;
    }

    TempFileName = lpTempFileName.offset(
        (PathNameString.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
    );
    memmove(
        TempFileName as *mut core::ffi::c_void,
        PrefixString.Buffer as *const core::ffi::c_void,
        PrefixString.Length as usize,
    );
    TempFileName = TempFileName.offset(
        (PrefixString.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
    );
    // Divergence from the C (still so on upstream master): it never rewinds TempFileName,
    // so every retry after a CREATE_NEW collision appends the next ID after the previous
    // "XXXX.tmp" instead of replacing it -- the names grow by 8 WCHARs per attempt and
    // walk off the end of the caller's MAX_PATH buffer. ole32's transacted-storage
    // scratch files hit it in Wine's storage32 test_locking (GetTempFileNameW faulted).
    let IdStart = TempFileName;

    loop {
        TempFileName = IdStart;
        if uUnique == 0 {
            CsrClientCallServer(
                &raw mut ApiMessage as PCSR_API_MESSAGE,
                core::ptr::null_mut(),
                CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetTempFile),
                core::mem::size_of::<BASE_GET_TEMP_FILE>() as ULONG,
            );
            if (*GetTempFile).UniqueID == 0 {
                Num = Num.wrapping_add(1);
                // C `continue` in a do-while still evaluates `while (Num & 0xFFFF)`;
                // a bare `continue` in this `loop` skipped it and could spin forever.
                if (Num & 0xFFFF) == 0 {
                    break;
                }
                continue;
            }
            ID = (*GetTempFile).UniqueID;
        } else {
            ID = uUnique;
        }

        RtlIntegerToChar(
            ID as ULONG,
            0x10,
            core::mem::size_of::<[UCHAR; 5]>() as ULONG,
            &raw mut IDString as *mut UCHAR as PCHAR,
        );
        Let = &raw mut IDString as PUCHAR;
        loop {
            let fresh0 = TempFileName;
            TempFileName = TempFileName.offset(1);
            *fresh0 = RtlAnsiCharToUnicodeChar(&raw mut Let);
            if *Let == 0 {
                break;
            }
        }

        memmove(
            TempFileName as *mut core::ffi::c_void,
            &raw const Ext as *const WCHAR as *const core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 5]>(),
        );

        if uUnique != 0 {
            return uUnique;
        }

        if RtlIsDosDeviceName_U(lpTempFileName) == 0 {
            TempFile = CreateFileW(
                lpTempFileName,
                GENERIC_READ,
                0,
                core::ptr::null_mut(),
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
                core::ptr::null_mut(),
            );
            if TempFile != INVALID_HANDLE_VALUE {
                NtClose(TempFile);
                return ID;
            }

            LastError = GetLastError();
            if LastError == ERROR_INVALID_PARAMETER
                || LastError == ERROR_CANNOT_MAKE
                || LastError == ERROR_WRITE_PROTECT
                || LastError == ERROR_NETWORK_ACCESS_DENIED
                || LastError == ERROR_DISK_FULL
                || LastError == ERROR_INVALID_NAME
                || LastError == ERROR_BAD_PATHNAME
                || LastError == ERROR_NO_INHERITANCE
                || LastError == ERROR_DISK_CORRUPT
                || (LastError == ERROR_ACCESS_DENIED
                    && (*NtCurrentTeb()).LastStatusValue != STATUS_FILE_IS_A_DIRECTORY as u32)
            {
                break;
            }
        }
        Num = Num.wrapping_add(1);
        if (Num & 0xFFFF) == 0 {
            break;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn SetFileShortNameW(
    hFile: HANDLE,
    lpShortName: LPCWSTR,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut NeededSize: ULONG = 0;
    let mut ShortName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileNameInfoBuf: PFILE_NAME_INFORMATION = core::ptr::null_mut();

    if IsConsoleHandle(hFile) {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }

    if lpShortName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    RtlInitUnicodeString(&raw mut ShortName, lpShortName);

    NeededSize = (core::mem::size_of::<FILE_NAME_INFORMATION>() as ULONG)
        .wrapping_add(ShortName.Length as ULONG)
        .wrapping_add(core::mem::size_of::<WCHAR>() as ULONG);

    FileNameInfoBuf = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        NeededSize as SIZE_T,
    ) as PFILE_NAME_INFORMATION;

    if FileNameInfoBuf.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE as BOOL;
    }

    (*FileNameInfoBuf).FileNameLength = ShortName.Length as ULONG;
    RtlCopyMemory(
        &raw mut (*FileNameInfoBuf).FileName as *mut ::core::ffi::c_void,
        ShortName.Buffer as *const ::core::ffi::c_void,
        ShortName.Length as SIZE_T,
    );

    Status = NtSetInformationFile(
        hFile,
        &raw mut IoStatusBlock,
        FileNameInfoBuf as PVOID,
        NeededSize,
        FileShortNameInformation,
    );

    RtlFreeHeap(
        RtlGetProcessHeap(),
        0 as ULONG,
        FileNameInfoBuf as PVOID,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetFileShortNameA(
    hFile: HANDLE,
    lpShortName: LPCSTR,
) -> BOOL {
    let mut ShortNameW: PWCHAR = core::ptr::null_mut();

    if hFile as usize & 0x10000003 == 0x3 {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return 0;
    }

    if lpShortName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    ShortNameW = FilenameA2W(lpShortName, 0);
    if ShortNameW.is_null() {
        return 0;
    }

    return SetFileShortNameW(hFile, ShortNameW as LPCWSTR);
}
#[no_mangle]
pub unsafe extern "system" fn CheckNameLegalDOS8Dot3W(
    lpName: LPCWSTR,
    lpOemName: LPSTR,
    OemNameSize: DWORD,
    pbNameContainsSpaces: PBOOL,
    pbNameLegal: PBOOL,
) -> BOOL {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut AnsiName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut NameContainsSpaces: BOOLEAN = 0;

    if lpName.is_null()
        || (lpOemName.is_null() && OemNameSize != 0)
        || pbNameLegal.is_null()
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !lpOemName.is_null() {
        AnsiName.Buffer = lpOemName;
        AnsiName.MaximumLength = (OemNameSize as USHORT).wrapping_mul(core::mem::size_of::<CHAR>() as USHORT);
        AnsiName.Length = 0;
    }

    RtlInitUnicodeString(&mut Name, lpName);

    *pbNameLegal = RtlIsNameLegalDOS8Dot3(
        &Name,
        if !lpOemName.is_null() { &mut AnsiName } else { core::ptr::null_mut() },
        &mut NameContainsSpaces,
    ) as BOOL;

    if *pbNameLegal != 0 && !pbNameContainsSpaces.is_null() {
        *pbNameContainsSpaces = NameContainsSpaces as BOOL;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn CheckNameLegalDOS8Dot3A(
    lpName: LPCSTR,
    lpOemName: LPSTR,
    OemNameSize: DWORD,
    pbNameContainsSpaces: PBOOL,
    pbNameLegal: PBOOL,
) -> BOOL {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut AnsiName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut AnsiInputName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;
    let mut NameContainsSpaces: BOOLEAN = 0;

    if lpName.is_null()
        || (lpOemName.is_null() && OemNameSize != 0)
        || pbNameLegal.is_null()
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !lpOemName.is_null() {
        AnsiName.Buffer = lpOemName;
        AnsiName.MaximumLength = (OemNameSize as USHORT).wrapping_mul(core::mem::size_of::<CHAR>() as USHORT);
        AnsiName.Length = 0;
    }

    RtlInitAnsiString(&mut AnsiInputName, lpName as LPSTR);
    if bIsFileApiAnsi != 0 {
        Status = RtlAnsiStringToUnicodeString(&mut Name, &mut AnsiInputName, 1);
    } else {
        Status = RtlOemStringToUnicodeString(&mut Name, &mut AnsiInputName, 1);
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    *pbNameLegal = RtlIsNameLegalDOS8Dot3(
        &Name,
        if !lpOemName.is_null() { &mut AnsiName } else { core::ptr::null_mut() },
        &mut NameContainsSpaces,
    ) as BOOL;

    if *pbNameLegal != 0 && !pbNameContainsSpaces.is_null() {
        *pbNameContainsSpaces = NameContainsSpaces as BOOL;
    }

    RtlFreeUnicodeString(&mut Name);

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/filename.c
//  * PURPOSE:         Directory functions
//  * PROGRAMMERS:     Ariadne (ariadne@xs4all.nl)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ******************************************************************/
//
// /* FUNCTIONS ****************************************************************/
//
// /***********************************************************************
//  *           GetTempFileNameA   (KERNEL32.@)
//  */
// UINT WINAPI
// GetTempFileNameA(IN LPCSTR lpPathName,
//                  IN LPCSTR lpPrefixString,
//                  IN UINT uUnique,
//                  OUT LPSTR lpTempFileName)
// {
//     UINT ID;
//     NTSTATUS Status;
//     LPWSTR lpTempFileNameW;
//     PUNICODE_STRING lpPathNameW;
//     ANSI_STRING TempFileNameStringA;
//     UNICODE_STRING lpPrefixStringW, TempFileNameStringW;
//
//     /* Convert strings */
//     lpPathNameW = Basep8BitStringToStaticUnicodeString(lpPathName);
//     if (!lpPathNameW)
//     {
//         return 0;
//     }
//
//     if (!Basep8BitStringToDynamicUnicodeString(&lpPrefixStringW, lpPrefixString))
//     {
//         return 0;
//     }
//
//     lpTempFileNameW = RtlAllocateHeap(RtlGetProcessHeap(), 0, MAX_PATH * sizeof(WCHAR));
//     if (!lpTempFileNameW)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         RtlFreeUnicodeString(&lpPrefixStringW);
//         return 0;
//     }
//
//     /* Call Unicode */
//     ID = GetTempFileNameW(lpPathNameW->Buffer, lpPrefixStringW.Buffer, uUnique, lpTempFileNameW);
//     if (ID)
//     {
//         RtlInitUnicodeString(&TempFileNameStringW, lpTempFileNameW);
//         TempFileNameStringA.Buffer = lpTempFileName;
//         TempFileNameStringA.MaximumLength = MAX_PATH;
//
//         Status = BasepUnicodeStringTo8BitString(&TempFileNameStringA, &TempFileNameStringW, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             ID = 0;
//         }
//     }
//
//     /* Cleanup */
//     RtlFreeUnicodeString(&lpPrefixStringW);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, lpTempFileNameW);
//     return ID;
//  }
//
//  /***********************************************************************
//   *           GetTempFileNameW   (KERNEL32.@)
//   */
// UINT WINAPI
// GetTempFileNameW(IN LPCWSTR lpPathName,
//                  IN LPCWSTR lpPrefixString,
//                  IN UINT uUnique,
//                  OUT LPWSTR lpTempFileName)
// {
//     PUCHAR Let;
//     HANDLE TempFile;
//     UINT ID, Num = 0;
//     UCHAR IDString[5];
//     WCHAR * TempFileName;
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_GET_TEMP_FILE GetTempFile = &ApiMessage.Data.GetTempFileRequest;
//     DWORD FileAttributes, LastError;
//     UNICODE_STRING PathNameString, PrefixString;
//     static const WCHAR Ext[] = { L'.', 't', 'm', 'p', UNICODE_NULL };
//
//     RtlInitUnicodeString(&PathNameString, lpPathName);
//     if (PathNameString.Length == 0 || PathNameString.Buffer[(PathNameString.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         PathNameString.Length += sizeof(WCHAR);
//     }
//
//     /* lpTempFileName must be able to contain: PathName, Prefix (3), number(4), .tmp(4) & \0(1)
//      * See: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-gettempfilenamea
//      */
//     if (PathNameString.Length > (MAX_PATH - 3 - 4 - 4 - 1) * sizeof(WCHAR))
//     {
//         SetLastError(ERROR_BUFFER_OVERFLOW);
//         return 0;
//     }
//
//     /* If PathName and TempFileName aren't the same buffer, move PathName to TempFileName */
//     if (lpPathName != lpTempFileName)
//     {
//         memmove(lpTempFileName, PathNameString.Buffer, PathNameString.Length);
//     }
//
//     /* PathName MUST BE a path. Check it */
//     lpTempFileName[(PathNameString.Length / sizeof(WCHAR)) - 1] = UNICODE_NULL;
//     FileAttributes = GetFileAttributesW(lpTempFileName);
//     if (FileAttributes == INVALID_FILE_ATTRIBUTES)
//     {
//         /* Append a '\' if necessary */
//         lpTempFileName[(PathNameString.Length / sizeof(WCHAR)) - 1] = L'\\';
//         lpTempFileName[PathNameString.Length / sizeof(WCHAR)] = UNICODE_NULL;
//         FileAttributes = GetFileAttributesW(lpTempFileName);
//         if (FileAttributes == INVALID_FILE_ATTRIBUTES)
//         {
//             SetLastError(ERROR_DIRECTORY);
//             return 0;
//         }
//     }
//     if (!(FileAttributes & FILE_ATTRIBUTE_DIRECTORY))
//     {
//         SetLastError(ERROR_DIRECTORY);
//         return 0;
//     }
//
//     /* Make sure not to mix path & prefix */
//     lpTempFileName[(PathNameString.Length / sizeof(WCHAR)) - 1] = L'\\';
//     RtlInitUnicodeString(&PrefixString, lpPrefixString);
//     if (PrefixString.Length > 3 * sizeof(WCHAR))
//     {
//         PrefixString.Length = 3 * sizeof(WCHAR);
//     }
//
//     /* Append prefix to path */
//     TempFileName = lpTempFileName + PathNameString.Length / sizeof(WCHAR);
//     memmove(TempFileName, PrefixString.Buffer, PrefixString.Length);
//     TempFileName += PrefixString.Length / sizeof(WCHAR);
//
//     /* Then, generate filename */
//     do
//     {
//         /* If user didn't gave any ID, ask Csrss to give one */
//         if (!uUnique)
//         {
//             CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                                 NULL,
//                                 CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepGetTempFile),
//                                 sizeof(*GetTempFile));
//             if (GetTempFile->UniqueID == 0)
//             {
//                 Num++;
//                 continue;
//             }
//
//             ID = GetTempFile->UniqueID;
//         }
//         else
//         {
//             ID = uUnique;
//         }
//
//         /* Convert that ID to wchar */
//         RtlIntegerToChar(ID, 0x10, sizeof(IDString), (PCHAR)IDString);
//         Let = IDString;
//         do
//         {
//             *(TempFileName++) = RtlAnsiCharToUnicodeChar(&Let);
//         } while (*Let != 0);
//
//         /* Append extension & UNICODE_NULL */
//         memmove(TempFileName, Ext, sizeof(Ext));
//
//         /* If user provided its ID, just return */
//         if (uUnique)
//         {
//             return uUnique;
//         }
//
//         /* Then, try to create file */
//         if (!RtlIsDosDeviceName_U(lpTempFileName))
//         {
//             TempFile = CreateFileW(lpTempFileName,
//                                    GENERIC_READ,
//                                    0,
//                                    NULL,
//                                    CREATE_NEW,
//                                    FILE_ATTRIBUTE_NORMAL,
//                                    0);
//             if (TempFile != INVALID_HANDLE_VALUE)
//             {
//                 NtClose(TempFile);
//                 DPRINT("Temp file: %S\n", lpTempFileName);
//                 return ID;
//             }
//
//             LastError = GetLastError();
//             /* There is no need to recover from those errors, they would hit next step */
//             if (LastError == ERROR_INVALID_PARAMETER || LastError == ERROR_CANNOT_MAKE ||
//                 LastError == ERROR_WRITE_PROTECT || LastError == ERROR_NETWORK_ACCESS_DENIED ||
//                 LastError == ERROR_DISK_FULL || LastError == ERROR_INVALID_NAME ||
//                 LastError == ERROR_BAD_PATHNAME || LastError == ERROR_NO_INHERITANCE ||
//                 LastError == ERROR_DISK_CORRUPT ||
//                 (LastError == ERROR_ACCESS_DENIED && NtCurrentTeb()->LastStatusValue != STATUS_FILE_IS_A_DIRECTORY))
//             {
//                 break;
//             }
//         }
//         Num++;
//     } while (Num & 0xFFFF);
//
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetFileShortNameW(
//     HANDLE hFile,
//     LPCWSTR lpShortName)
// {
//     NTSTATUS Status;
//     ULONG NeededSize;
//     UNICODE_STRING ShortName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PFILE_NAME_INFORMATION FileNameInfo;
//
//     if(IsConsoleHandle(hFile))
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if(!lpShortName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&ShortName, lpShortName);
//
//     NeededSize = sizeof(FILE_NAME_INFORMATION) + ShortName.Length + sizeof(WCHAR);
//     if(!(FileNameInfo = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, NeededSize)))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     FileNameInfo->FileNameLength = ShortName.Length;
//     RtlCopyMemory(FileNameInfo->FileName, ShortName.Buffer, ShortName.Length);
//
//     Status = NtSetInformationFile(hFile,
//                                   &IoStatusBlock, //out
//                                   FileNameInfo,
//                                   NeededSize,
//                                   FileShortNameInformation);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, FileNameInfo);
//     if(!NT_SUCCESS(Status))
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
// SetFileShortNameA(
//     HANDLE hFile,
//     LPCSTR lpShortName)
// {
//     PWCHAR ShortNameW;
//
//     if(IsConsoleHandle(hFile))
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if(!lpShortName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (!(ShortNameW = FilenameA2W(lpShortName, FALSE)))
//         return FALSE;
//
//     return SetFileShortNameW(hFile, ShortNameW);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CheckNameLegalDOS8Dot3W(
//     LPCWSTR lpName,
//     LPSTR lpOemName OPTIONAL,
//     DWORD OemNameSize OPTIONAL,
//     PBOOL pbNameContainsSpaces OPTIONAL,
//     PBOOL pbNameLegal
//     )
// {
//     UNICODE_STRING Name;
//     ANSI_STRING AnsiName;
//     BOOLEAN NameContainsSpaces;
//
//     if(lpName == NULL ||
//        (lpOemName == NULL && OemNameSize != 0) ||
//        pbNameLegal == NULL)
//     {
//       SetLastError(ERROR_INVALID_PARAMETER);
//       return FALSE;
//     }
//
//     if(lpOemName != NULL)
//     {
//       AnsiName.Buffer = lpOemName;
//       AnsiName.MaximumLength = (USHORT)OemNameSize * sizeof(CHAR);
//       AnsiName.Length = 0;
//     }
//
//     RtlInitUnicodeString(&Name, lpName);
//
//     *pbNameLegal = RtlIsNameLegalDOS8Dot3(&Name,
//                                           (lpOemName ? &AnsiName : NULL),
//                                           &NameContainsSpaces);
//     if (*pbNameLegal && pbNameContainsSpaces)
//         *pbNameContainsSpaces = NameContainsSpaces;
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
// CheckNameLegalDOS8Dot3A(
//     LPCSTR lpName,
//     LPSTR lpOemName OPTIONAL,
//     DWORD OemNameSize OPTIONAL,
//     PBOOL pbNameContainsSpaces OPTIONAL,
//     PBOOL pbNameLegal
//     )
// {
//     UNICODE_STRING Name;
//     ANSI_STRING AnsiName, AnsiInputName;
//     NTSTATUS Status;
//     BOOLEAN NameContainsSpaces;
//
//     if(lpName == NULL ||
//        (lpOemName == NULL && OemNameSize != 0) ||
//        pbNameLegal == NULL)
//     {
//       SetLastError(ERROR_INVALID_PARAMETER);
//       return FALSE;
//     }
//
//     if(lpOemName != NULL)
//     {
//       AnsiName.Buffer = lpOemName;
//       AnsiName.MaximumLength = (USHORT)OemNameSize * sizeof(CHAR);
//       AnsiName.Length = 0;
//     }
//
//     RtlInitAnsiString(&AnsiInputName, (LPSTR)lpName);
//     if(bIsFileApiAnsi)
//       Status = RtlAnsiStringToUnicodeString(&Name, &AnsiInputName, TRUE);
//     else
//       Status = RtlOemStringToUnicodeString(&Name, &AnsiInputName, TRUE);
//
//     if(!NT_SUCCESS(Status))
//     {
//       BaseSetLastNTError(Status);
//       return FALSE;
//     }
//
//     *pbNameLegal = RtlIsNameLegalDOS8Dot3(&Name,
//                                           (lpOemName ? &AnsiName : NULL),
//                                           &NameContainsSpaces);
//     if (*pbNameLegal && pbNameContainsSpaces)
//         *pbNameContainsSpaces = NameContainsSpaces;
//
//     RtlFreeUnicodeString(&Name);
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
