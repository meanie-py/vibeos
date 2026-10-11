use winedebug_rust::{TRACE, WARN};
use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::{InitializeObjectAttributes, SearchPathW};

// ---- no-op Wine debug shims (NDEBUG: TRACE/WARN/ERR/FIXME compile out) ----

const STATUS_FILE_IS_A_DIRECTORY: NTSTATUS = 0xC00000BAu32 as NTSTATUS;

// FilenameA2W and IntCheckForConsoleFileName are cdecl in C (fileutils.c /
// console.c — no WINAPI in their decls) → extern "C".
extern "C" {
    fn FilenameA2W(NameA: LPCSTR, alloc: BOOL) -> PWCHAR;
    fn IntCheckForConsoleFileName(pszName: LPCWSTR, dwDesiredAccess: DWORD) -> LPCWSTR;
}
extern "system" {
    static bIsFileApiAnsi: BOOL;
    fn OpenConsoleW(wsName: LPCWSTR, dwDesiredAccess: DWORD, bInheritHandle: BOOL,
        dwShareMode: DWORD) -> HANDLE;
    fn GetFullPathNameA(lpFileName: LPCSTR, nBufferLength: DWORD, lpBuffer: LPSTR,
        lpFilePart: *mut LPSTR) -> DWORD;
    fn GetDriveTypeA(lpRootPathName: LPCSTR) -> UINT;
    fn GetFileAttributesA(lpFileName: LPCSTR) -> DWORD;
    fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 200 as ULONG;

#[repr(C)]
pub struct _DEBUG_CHANNEL {
    pub name: *const u8,
    pub level: u32,
}
unsafe impl Sync for _DEBUG_CHANNEL {}

pub static kernel32file: _DEBUG_CHANNEL = _DEBUG_CHANNEL {
    name: b"kernel32file\0".as_ptr(),
    level: 0,
};

#[no_mangle]
pub unsafe extern "system" fn CreateFileA(
    lpFileName: LPCSTR,
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    dwCreationDisposition: DWORD,
    dwFlagsAndAttributes: DWORD,
    hTemplateFile: HANDLE,
) -> HANDLE {
    let FileNameW: PWCHAR;
    let FileHandle: HANDLE;

    TRACE!("CreateFileA(lpFileName %s)\n", lpFileName);

    FileNameW = FilenameA2W(lpFileName, FALSE as BOOL);
    if FileNameW.is_null() {
        return INVALID_HANDLE_VALUE;
    }

    FileHandle = CreateFileW(
        FileNameW as LPCWSTR,
        dwDesiredAccess,
        dwShareMode,
        lpSecurityAttributes,
        dwCreationDisposition,
        dwFlagsAndAttributes,
        hTemplateFile,
    );

    return FileHandle;
}

#[no_mangle]
pub unsafe extern "system" fn CreateFileW(
    lpFileName: LPCWSTR,
    mut dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    mut dwCreationDisposition: DWORD,
    dwFlagsAndAttributes: DWORD,
    hTemplateFile: HANDLE,
) -> HANDLE {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut NtPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut pszConsoleFileName: LPCWSTR = core::ptr::null();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut FileAttributes: ULONG = 0;
    let mut Flags: ULONG = 0;
    let mut EaBuffer: PVOID = core::ptr::null_mut();
    let mut EaLength: ULONG = 0;
    let mut TrailingBackslash: BOOLEAN = 0;

    if lpFileName.is_null() || *lpFileName.offset(0) == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return INVALID_HANDLE_VALUE;
    }

    TRACE!(b"CreateFileW(lpFileName %S)\n\0" as *const u8 as LPCSTR, lpFileName);

    match dwCreationDisposition {
        1 => { dwCreationDisposition = FILE_CREATE; }
        2 => { dwCreationDisposition = FILE_OVERWRITE_IF; }
        3 => { dwCreationDisposition = FILE_OPEN; }
        4 => { dwCreationDisposition = FILE_OPEN_IF; }
        5 => { dwCreationDisposition = FILE_OVERWRITE; }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return INVALID_HANDLE_VALUE;
        }
    }

    pszConsoleFileName = IntCheckForConsoleFileName(lpFileName, dwDesiredAccess);
    if !pszConsoleFileName.is_null() {
        return OpenConsoleW(
            pszConsoleFileName,
            dwDesiredAccess,
            if !lpSecurityAttributes.is_null() {
                (*lpSecurityAttributes).bInheritHandle
            } else {
                0
            },
            FILE_SHARE_READ | FILE_SHARE_WRITE,
        );
    }

    if dwFlagsAndAttributes & FILE_FLAG_OVERLAPPED == 0 {
        Flags |= FILE_SYNCHRONOUS_IO_NONALERT;
    }

    if dwFlagsAndAttributes & FILE_FLAG_WRITE_THROUGH != 0 {
        Flags |= FILE_WRITE_THROUGH;
    }

    if dwFlagsAndAttributes & FILE_FLAG_NO_BUFFERING != 0 {
        Flags |= FILE_NO_INTERMEDIATE_BUFFERING;
    }

    if dwFlagsAndAttributes & FILE_FLAG_RANDOM_ACCESS != 0 {
        Flags |= FILE_RANDOM_ACCESS;
    }

    if dwFlagsAndAttributes & FILE_FLAG_SEQUENTIAL_SCAN != 0 {
        Flags |= FILE_SEQUENTIAL_ONLY;
    }

    if dwFlagsAndAttributes & FILE_FLAG_DELETE_ON_CLOSE != 0 {
        Flags |= FILE_DELETE_ON_CLOSE;
        dwDesiredAccess |= DELETE;
    }

    if dwFlagsAndAttributes & FILE_FLAG_BACKUP_SEMANTICS != 0 {
        if dwDesiredAccess & GENERIC_ALL != 0 {
            Flags |= FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REMOTE_INSTANCE;
        } else {
            if dwDesiredAccess & GENERIC_READ != 0 {
                Flags |= FILE_OPEN_FOR_BACKUP_INTENT;
            }
            if dwDesiredAccess & GENERIC_WRITE != 0 {
                Flags |= FILE_OPEN_REMOTE_INSTANCE;
            }
        }
    } else {
        Flags |= FILE_NON_DIRECTORY_FILE;
    }

    if dwFlagsAndAttributes & FILE_FLAG_OPEN_REPARSE_POINT != 0 {
        Flags |= FILE_OPEN_REPARSE_POINT;
    }

    if dwFlagsAndAttributes & FILE_FLAG_OPEN_NO_RECALL != 0 {
        Flags |= FILE_OPEN_NO_RECALL;
    }

    FileAttributes = dwFlagsAndAttributes & (FILE_ATTRIBUTE_VALID_FLAGS & !FILE_ATTRIBUTE_DIRECTORY);

    dwDesiredAccess |= SYNCHRONIZE | FILE_READ_ATTRIBUTES;

    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut NtPathU,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        WARN!(b"Invalid path\n\0" as *const u8 as LPCSTR);
        SetLastError(ERROR_FILE_NOT_FOUND);
        return INVALID_HANDLE_VALUE;
    }

    TRACE!(b"NtPathU '%wZ'\n\0" as *const u8 as LPCSTR, &NtPathU);

    TrailingBackslash = 0;
    if NtPathU.Length as usize >= core::mem::size_of::<WCHAR>()
        && *NtPathU.Buffer.offset(
            (NtPathU.Length as usize).wrapping_div(core::mem::size_of::<WCHAR>()).wrapping_sub(1) as isize,
        ) != 0
    {
        TrailingBackslash = 1;
    }

    if !hTemplateFile.is_null() {
        let mut EaInformation: FILE_EA_INFORMATION = core::mem::zeroed();

        loop {
            Status = NtQueryInformationFile(
                hTemplateFile,
                &mut IoStatusBlock,
                &mut EaInformation as *mut _ as PVOID,
                core::mem::size_of::<FILE_EA_INFORMATION>() as ULONG,
                FileEaInformation,
            );

            if NT_SUCCESS(Status) && EaInformation.EaSize != 0 {
                EaBuffer = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    EaInformation.EaSize as SIZE_T,
                );

                if EaBuffer.is_null() {
                    RtlFreeHeap(
                        RtlGetProcessHeap(),
                        0,
                        NtPathU.Buffer as PVOID,
                    );
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    return INVALID_HANDLE_VALUE;
                }

                Status = NtQueryEaFile(
                    hTemplateFile,
                    &mut IoStatusBlock,
                    EaBuffer,
                    EaInformation.EaSize,
                    0,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    1,
                );

                if NT_SUCCESS(Status) {
                    EaLength = EaInformation.EaSize;
                    break;
                } else {
                    RtlFreeHeap(
                        RtlGetProcessHeap(),
                        0,
                        EaBuffer,
                    );
                    EaBuffer = core::ptr::null_mut();

                    if Status != STATUS_BUFFER_TOO_SMALL {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtPathU,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !lpSecurityAttributes.is_null() {
        if (*lpSecurityAttributes).bInheritHandle != 0 {
            ObjectAttributes.Attributes |= OBJ_INHERIT;
        }
        ObjectAttributes.SecurityDescriptor = (*lpSecurityAttributes).lpSecurityDescriptor as PVOID;
    }

    if dwFlagsAndAttributes & FILE_FLAG_POSIX_SEMANTICS == 0 {
        ObjectAttributes.Attributes |= OBJ_CASE_INSENSITIVE;
    }

    Status = NtCreateFile(
        &mut FileHandle,
        dwDesiredAccess,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FileAttributes,
        dwShareMode,
        dwCreationDisposition,
        Flags,
        EaBuffer,
        EaLength,
    );

    RtlFreeHeap(
        RtlGetProcessHeap(),
        0,
        NtPathU.Buffer as PVOID,
    );

    if !EaBuffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            EaBuffer,
        );
    }

    if !NT_SUCCESS(Status) {
        if Status == STATUS_OBJECT_NAME_COLLISION && dwCreationDisposition == FILE_CREATE {
            SetLastError(ERROR_FILE_EXISTS);
        } else if Status == STATUS_FILE_IS_A_DIRECTORY && TrailingBackslash != 0 {
            SetLastError(ERROR_PATH_NOT_FOUND);
        } else {
            BaseSetLastNTError(Status);
        }
        return INVALID_HANDLE_VALUE;
    }

    if dwCreationDisposition == FILE_OPEN_IF {
        SetLastError(
            if IoStatusBlock.Information == FILE_OPENED as ULONG_PTR {
                ERROR_ALREADY_EXISTS
            } else {
                ERROR_SUCCESS
            },
        );
    } else if dwCreationDisposition == FILE_OVERWRITE_IF {
        SetLastError(
            if IoStatusBlock.Information == FILE_OVERWRITTEN as ULONG_PTR {
                ERROR_ALREADY_EXISTS
            } else {
                ERROR_SUCCESS
            },
        );
    } else {
        SetLastError(ERROR_SUCCESS);
    }

    FileHandle
}

#[no_mangle]
pub unsafe extern "system" fn OpenFile(
    mut lpFileName: LPCSTR,
    lpReOpenBuff: LPOFSTRUCT,
    uStyle: UINT,
) -> HFILE {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileNameString: _UNICODE_STRING = core::mem::zeroed();
    let mut FileNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: ANSI_STRING = core::mem::zeroed();
    let mut PathNameW: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut errCode: NTSTATUS = 0;
    let mut FilePart: PWCHAR = core::ptr::null_mut();
    let mut Len: ULONG = 0;

    TRACE!(
        b"OpenFile('%s', lpReOpenBuff %p, uStyle %x)\n\0".as_ptr() as LPCSTR,
        lpFileName,
        lpReOpenBuff,
        uStyle,
    );

    if lpReOpenBuff.is_null() {
        return HFILE_ERROR;
    }

    (*lpReOpenBuff).nErrCode = 0;

    if uStyle & OF_REOPEN != 0 {
        lpFileName = (*lpReOpenBuff).szPathName.as_ptr() as LPCSTR;
    }

    if lpFileName.is_null() {
        return HFILE_ERROR;
    }

    if GetFullPathNameA(
        lpFileName,
        core::mem::size_of::<[CHAR; 128]>() as DWORD,
        (*lpReOpenBuff).szPathName.as_mut_ptr() as LPSTR,
        core::ptr::null_mut(),
    ) == 0
    {
        (*lpReOpenBuff).nErrCode = GetLastError() as WORD;
        return HFILE_ERROR;
    }

    if uStyle & OF_PARSE != 0 {
        (*lpReOpenBuff).fFixedDisk = (GetDriveTypeA(
            (*lpReOpenBuff).szPathName.as_ptr() as LPCSTR,
        ) != DRIVE_REMOVABLE) as BYTE;
        TRACE!(
            b"(%s): OF_PARSE, res = '%s'\n\0".as_ptr() as LPCSTR,
            lpFileName,
            (*lpReOpenBuff).szPathName.as_ptr() as LPCSTR,
        );
        return 0;
    }

    if (uStyle & OF_EXIST) != 0 && (uStyle & OF_CREATE) == 0 {
        let dwAttributes: DWORD = GetFileAttributesA(
            (*lpReOpenBuff).szPathName.as_ptr() as LPCSTR,
        );

        match dwAttributes {
            INVALID_FILE_ATTRIBUTES => {
                SetLastError(ERROR_FILE_NOT_FOUND);
                (*lpReOpenBuff).nErrCode = ERROR_FILE_NOT_FOUND as WORD;
                return -1;
            }
            FILE_ATTRIBUTE_DIRECTORY => {
                SetLastError(ERROR_ACCESS_DENIED);
                (*lpReOpenBuff).nErrCode = ERROR_ACCESS_DENIED as WORD;
                return -1;
            }
            _ => {
                (*lpReOpenBuff).cBytes = core::mem::size_of::<OFSTRUCT>() as BYTE;
                return 1;
            }
        }
    }

    (*lpReOpenBuff).cBytes = core::mem::size_of::<OFSTRUCT>() as BYTE;

    if (uStyle & OF_CREATE) == OF_CREATE {
        let Sharing: DWORD;
        match uStyle & 0x70 {
            OF_SHARE_EXCLUSIVE => {
                Sharing = 0;
            }
            OF_SHARE_DENY_WRITE => {
                Sharing = FILE_SHARE_READ;
            }
            OF_SHARE_DENY_READ => {
                Sharing = FILE_SHARE_WRITE;
            }
            OF_SHARE_DENY_NONE | OF_SHARE_COMPAT | _ => {
                Sharing = FILE_SHARE_READ | FILE_SHARE_WRITE;
            }
        }
        return CreateFileA(
            lpFileName,
            GENERIC_READ | GENERIC_WRITE,
            Sharing,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        ) as HFILE;
    }

    RtlInitAnsiString(&mut FileName, lpFileName as LPSTR);

    if bIsFileApiAnsi != 0 {
        RtlAnsiStringToUnicodeString(&mut FileNameU, &mut FileName, TRUE as BOOLEAN);
    } else {
        RtlOemStringToUnicodeString(&mut FileNameU, &mut FileName, TRUE as BOOLEAN);
    }

    Len = SearchPathW(
        core::ptr::null(),
        FileNameU.Buffer,
        core::ptr::null(),
        OFS_MAXPATHNAME as DWORD,
        PathNameW.as_mut_ptr(),
        &mut FilePart,
    ) as ULONG;

    RtlFreeUnicodeString(&mut FileNameU);

    if Len == 0 || Len > OFS_MAXPATHNAME as ULONG {
        (*lpReOpenBuff).nErrCode = GetLastError() as WORD;
        return HFILE_ERROR;
    }

    if uStyle & OF_DELETE != 0 {
        if DeleteFileW(PathNameW.as_ptr()) == 0 {
            (*lpReOpenBuff).nErrCode = GetLastError() as WORD;
            return HFILE_ERROR;
        }
        TRACE!(
            b"(%s): OF_DELETE return = OK\n\0".as_ptr() as LPCSTR,
            lpFileName,
        );
        return TRUE as HFILE;
    }

    FileName.Buffer = (*lpReOpenBuff).szPathName.as_mut_ptr();
    FileName.Length = 0;
    FileName.MaximumLength = OFS_MAXPATHNAME as USHORT;

    RtlInitUnicodeString(&mut FileNameU, PathNameW.as_ptr());

    if bIsFileApiAnsi != 0 {
        RtlUnicodeStringToAnsiString(&mut FileName, &mut FileNameU, FALSE as BOOLEAN);
    } else {
        RtlUnicodeStringToOemString(&mut FileName, &mut FileNameU, FALSE as BOOLEAN);
    }

    if RtlDosPathNameToNtPathName_U(
        PathNameW.as_ptr(),
        &mut FileNameString,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        return HFILE_ERROR;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_INHERIT;
    ObjectAttributes.ObjectName = &mut FileNameString;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    errCode = NtOpenFile(
        &mut FileHandle,
        GENERIC_READ | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ,
        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
    );

    RtlFreeHeap(
        RtlGetProcessHeap(),
        0,
        FileNameString.Buffer as PVOID,
    );

    (*lpReOpenBuff).nErrCode = RtlNtStatusToDosError(errCode) as WORD;

    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        return HFILE_ERROR;
    }

    if uStyle & OF_EXIST != 0 {
        NtClose(FileHandle);
        return 1;
    }

    return FileHandle as HFILE;
}

#[no_mangle]
pub unsafe extern "system" fn OpenDataFile(
    hFile: HANDLE,
    dwUnused: DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/client/file/create.c\0"
            .as_ptr() as *const i8,
        555i32,
        b"OpenDataFile\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/client/file/create.c\0"
                .as_ptr() as *const i8,
            555i32,
        );
    }
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn ReOpenFile(
    hOriginalFile: HANDLE,
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    dwFlags: DWORD,
) -> HANDLE {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/client/file/create.c\0"
            .as_ptr() as *const i8,
        569i32,
        b"ReOpenFile\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/client/file/create.c\0"
                .as_ptr() as *const i8,
            569i32,
        );
    }
    return INVALID_HANDLE_VALUE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/create.c
//  * PURPOSE:         Directory functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  *                  Removed use of SearchPath (not used by Windows)
//  *                  18/08/2002: CreateFileW mess cleaned up (KJK::Hyperion)
//  *                  24/08/2002: removed superfluous DPRINTs (KJK::Hyperion)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// DEBUG_CHANNEL(kernel32file);
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// HANDLE WINAPI CreateFileA (LPCSTR			lpFileName,
// 			    DWORD			dwDesiredAccess,
// 			    DWORD			dwShareMode,
// 			    LPSECURITY_ATTRIBUTES	lpSecurityAttributes,
// 			    DWORD			dwCreationDisposition,
// 			    DWORD			dwFlagsAndAttributes,
// 			    HANDLE			hTemplateFile)
// {
//    PWCHAR FileNameW;
//    HANDLE FileHandle;
//
//    TRACE!("CreateFileA(lpFileName %s)\n",lpFileName);
//
//    if (!(FileNameW = FilenameA2W(lpFileName, FALSE)))
//       return INVALID_HANDLE_VALUE;
//
//    FileHandle = CreateFileW (FileNameW,
// 			     dwDesiredAccess,
// 			     dwShareMode,
// 			     lpSecurityAttributes,
// 			     dwCreationDisposition,
// 			     dwFlagsAndAttributes,
// 			     hTemplateFile);
//
//    return FileHandle;
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE WINAPI CreateFileW (LPCWSTR			lpFileName,
// 			    DWORD			dwDesiredAccess,
// 			    DWORD			dwShareMode,
// 			    LPSECURITY_ATTRIBUTES	lpSecurityAttributes,
// 			    DWORD			dwCreationDisposition,
// 			    DWORD			dwFlagsAndAttributes,
// 			    HANDLE			hTemplateFile)
// {
//    OBJECT_ATTRIBUTES ObjectAttributes;
//    IO_STATUS_BLOCK IoStatusBlock;
//    UNICODE_STRING NtPathU;
//    LPCWSTR pszConsoleFileName;
//    HANDLE FileHandle;
//    NTSTATUS Status;
//    ULONG FileAttributes, Flags = 0;
//    PVOID EaBuffer = NULL;
//    ULONG EaLength = 0;
//    BOOLEAN TrailingBackslash;
//
//    if (!lpFileName || !lpFileName[0])
//    {
//        SetLastError( ERROR_PATH_NOT_FOUND );
//        return INVALID_HANDLE_VALUE;
//    }
//
//    TRACE!("CreateFileW(lpFileName %S)\n",lpFileName);
//
//    /* validate & translate the creation disposition */
//    switch (dwCreationDisposition)
//      {
//       case CREATE_NEW:
// 	dwCreationDisposition = FILE_CREATE;
// 	break;
//
//       case CREATE_ALWAYS:
// 	dwCreationDisposition = FILE_OVERWRITE_IF;
// 	break;
//
//       case OPEN_EXISTING:
// 	dwCreationDisposition = FILE_OPEN;
// 	break;
//
//       case OPEN_ALWAYS:
// 	dwCreationDisposition = FILE_OPEN_IF;
// 	break;
//
//       case TRUNCATE_EXISTING:
// 	dwCreationDisposition = FILE_OVERWRITE;
//         break;
//
//       default:
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return (INVALID_HANDLE_VALUE);
//      }
//
//    /* check for console input/output */
//    pszConsoleFileName = IntCheckForConsoleFileName(lpFileName, dwDesiredAccess);
//    if (pszConsoleFileName)
//    {
//       return OpenConsoleW(pszConsoleFileName,
//                           dwDesiredAccess,
//                           lpSecurityAttributes ? lpSecurityAttributes->bInheritHandle : FALSE,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE);
//    }
//
//   /* validate & translate the flags */
//
//    /* translate the flags that need no validation */
//    if (!(dwFlagsAndAttributes & FILE_FLAG_OVERLAPPED))
//    {
//       /* yes, nonalert is correct! apc's are not delivered
//       while waiting for file io to complete */
//       Flags |= FILE_SYNCHRONOUS_IO_NONALERT;
//    }
//
//    if(dwFlagsAndAttributes & FILE_FLAG_WRITE_THROUGH)
//       Flags |= FILE_WRITE_THROUGH;
//
//    if(dwFlagsAndAttributes & FILE_FLAG_NO_BUFFERING)
//       Flags |= FILE_NO_INTERMEDIATE_BUFFERING;
//
//    if(dwFlagsAndAttributes & FILE_FLAG_RANDOM_ACCESS)
//       Flags |= FILE_RANDOM_ACCESS;
//
//    if(dwFlagsAndAttributes & FILE_FLAG_SEQUENTIAL_SCAN)
//       Flags |= FILE_SEQUENTIAL_ONLY;
//
//    if(dwFlagsAndAttributes & FILE_FLAG_DELETE_ON_CLOSE)
//    {
//       Flags |= FILE_DELETE_ON_CLOSE;
//       dwDesiredAccess |= DELETE;
//    }
//
//    if(dwFlagsAndAttributes & FILE_FLAG_BACKUP_SEMANTICS)
//    {
//       if(dwDesiredAccess & GENERIC_ALL)
//          Flags |= FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REMOTE_INSTANCE;
//       else
//       {
//          if(dwDesiredAccess & GENERIC_READ)
//             Flags |= FILE_OPEN_FOR_BACKUP_INTENT;
//
//          if(dwDesiredAccess & GENERIC_WRITE)
//             Flags |= FILE_OPEN_REMOTE_INSTANCE;
//       }
//    }
//    else
//       Flags |= FILE_NON_DIRECTORY_FILE;
//
//    if(dwFlagsAndAttributes & FILE_FLAG_OPEN_REPARSE_POINT)
//       Flags |= FILE_OPEN_REPARSE_POINT;
//
//    if(dwFlagsAndAttributes & FILE_FLAG_OPEN_NO_RECALL)
//       Flags |= FILE_OPEN_NO_RECALL;
//
//    FileAttributes = (dwFlagsAndAttributes & (FILE_ATTRIBUTE_VALID_FLAGS & ~FILE_ATTRIBUTE_DIRECTORY));
//
//    /* handle may always be waited on and querying attributes are always allowed */
//    dwDesiredAccess |= SYNCHRONIZE | FILE_READ_ATTRIBUTES;
//
//    /* FILE_FLAG_POSIX_SEMANTICS is handled later */
//
//    /* validate & translate the filename */
//    if (!RtlDosPathNameToNtPathName_U (lpFileName,
// 				      &NtPathU,
// 				      NULL,
// 				      NULL))
//    {
//      WARN!("Invalid path\n");
//      SetLastError(ERROR_FILE_NOT_FOUND);
//      return INVALID_HANDLE_VALUE;
//    }
//
//    TRACE!("NtPathU \'%wZ\'\n", &NtPathU);
//
//    TrailingBackslash = FALSE;
//    if (NtPathU.Length >= sizeof(WCHAR) &&
//        NtPathU.Buffer[NtPathU.Length / sizeof(WCHAR) - 1])
//    {
//       TrailingBackslash = TRUE;
//    }
//
//    if (hTemplateFile != NULL)
//    {
//       FILE_EA_INFORMATION EaInformation;
//
//       for (;;)
//       {
//          /* try to get the size of the extended attributes, if we fail just continue
//             creating the file without copying the attributes! */
//          Status = NtQueryInformationFile(hTemplateFile,
//                                          &IoStatusBlock,
//                                          &EaInformation,
//                                          sizeof(FILE_EA_INFORMATION),
//                                          FileEaInformation);
//          if (NT_SUCCESS(Status) && (EaInformation.EaSize != 0))
//          {
//             /* there's extended attributes to read, let's give it a try */
//             EaBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        0,
//                                        EaInformation.EaSize);
//             if (EaBuffer == NULL)
//             {
//                RtlFreeHeap(RtlGetProcessHeap(),
//                            0,
//                            NtPathU.Buffer);
//
//                /* the template file handle is valid and has extended attributes,
//                   however we seem to lack some memory here. We should fail here! */
//                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//                return INVALID_HANDLE_VALUE;
//             }
//
//             Status = NtQueryEaFile(hTemplateFile,
//                                    &IoStatusBlock,
//                                    EaBuffer,
//                                    EaInformation.EaSize,
//                                    FALSE,
//                                    NULL,
//                                    0,
//                                    NULL,
//                                    TRUE);
//
//             if (NT_SUCCESS(Status))
//             {
//                /* we successfully read the extended attributes, break the loop
//                   and continue */
//                EaLength = EaInformation.EaSize;
//                break;
//             }
//             else
//             {
//                RtlFreeHeap(RtlGetProcessHeap(),
//                            0,
//                            EaBuffer);
//                EaBuffer = NULL;
//
//                if (Status != STATUS_BUFFER_TOO_SMALL)
//                {
//                   /* unless we just allocated not enough memory, break the loop
//                      and just continue without copying extended attributes */
//                   break;
//                }
//             }
//          }
//          else
//          {
//             /* we either failed to get the size of the extended attributes or
//                they're empty, just continue as there's no need to copy
//                attributes */
//             break;
//          }
//       }
//    }
//
//    /* build the object attributes */
//    InitializeObjectAttributes(&ObjectAttributes,
//                               &NtPathU,
//                               0,
//                               NULL,
//                               NULL);
//
//    if (lpSecurityAttributes)
//    {
//       if(lpSecurityAttributes->bInheritHandle)
//          ObjectAttributes.Attributes |= OBJ_INHERIT;
//
//       ObjectAttributes.SecurityDescriptor = lpSecurityAttributes->lpSecurityDescriptor;
//    }
//
//    if(!(dwFlagsAndAttributes & FILE_FLAG_POSIX_SEMANTICS))
//     ObjectAttributes.Attributes |= OBJ_CASE_INSENSITIVE;
//
//    /* perform the call */
//    Status = NtCreateFile (&FileHandle,
// 			  dwDesiredAccess,
// 			  &ObjectAttributes,
// 			  &IoStatusBlock,
// 			  NULL,
// 			  FileAttributes,
// 			  dwShareMode,
// 			  dwCreationDisposition,
// 			  Flags,
// 			  EaBuffer,
// 			  EaLength);
//
//    RtlFreeHeap(RtlGetProcessHeap(),
//                0,
//                NtPathU.Buffer);
//
//    /* free the extended attributes buffer if allocated */
//    if (EaBuffer != NULL)
//    {
//       RtlFreeHeap(RtlGetProcessHeap(),
//                   0,
//                   EaBuffer);
//    }
//
//    /* error */
//    if (!NT_SUCCESS(Status))
//    {
//       /* In the case file creation was rejected due to CREATE_NEW flag
//        * was specified and file with that name already exists, correct
//        * last error is ERROR_FILE_EXISTS and not ERROR_ALREADY_EXISTS.
//        * Note: RtlNtStatusToDosError is not the subject to blame here.
//        */
//       if (Status == STATUS_OBJECT_NAME_COLLISION &&
//           dwCreationDisposition == FILE_CREATE)
//       {
//          SetLastError( ERROR_FILE_EXISTS );
//       }
//       else if (Status == STATUS_FILE_IS_A_DIRECTORY &&
//                TrailingBackslash)
//       {
//          SetLastError(ERROR_PATH_NOT_FOUND);
//       }
//       else
//       {
//          BaseSetLastNTError (Status);
//       }
//
//       return INVALID_HANDLE_VALUE;
//    }
//
//   /*
//   create with OPEN_ALWAYS (FILE_OPEN_IF) returns info = FILE_OPENED or FILE_CREATED
//   create with CREATE_ALWAYS (FILE_OVERWRITE_IF) returns info = FILE_OVERWRITTEN or FILE_CREATED
//   */
//   if (dwCreationDisposition == FILE_OPEN_IF)
//   {
//     SetLastError(IoStatusBlock.Information == FILE_OPENED ? ERROR_ALREADY_EXISTS : ERROR_SUCCESS);
//   }
//   else if (dwCreationDisposition == FILE_OVERWRITE_IF)
//   {
//     SetLastError(IoStatusBlock.Information == FILE_OVERWRITTEN ? ERROR_ALREADY_EXISTS : ERROR_SUCCESS);
//   }
//   else
//   {
//     SetLastError(ERROR_SUCCESS);
//   }
//
//   return FileHandle;
// }
//
// /*
//  * @implemented
//  */
// HFILE WINAPI
// OpenFile(LPCSTR lpFileName,
// 	 LPOFSTRUCT lpReOpenBuff,
// 	 UINT uStyle)
// {
// 	OBJECT_ATTRIBUTES ObjectAttributes;
// 	IO_STATUS_BLOCK IoStatusBlock;
// 	UNICODE_STRING FileNameString;
// 	UNICODE_STRING FileNameU;
// 	ANSI_STRING FileName;
// 	WCHAR PathNameW[MAX_PATH];
// 	HANDLE FileHandle = NULL;
// 	NTSTATUS errCode;
// 	PWCHAR FilePart;
// 	ULONG Len;
//
// 	TRACE!("OpenFile('%s', lpReOpenBuff %p, uStyle %x)\n", lpFileName, lpReOpenBuff, uStyle);
//
// 	if (lpReOpenBuff == NULL)
// 	{
// 		return HFILE_ERROR;
// 	}
//
//     lpReOpenBuff->nErrCode = 0;
//
// 	if (uStyle & OF_REOPEN) lpFileName = lpReOpenBuff->szPathName;
//
// 	if (!lpFileName)
// 	{
// 		return HFILE_ERROR;
// 	}
//
// 	if (!GetFullPathNameA(lpFileName,
// 						  sizeof(lpReOpenBuff->szPathName),
// 						  lpReOpenBuff->szPathName,
// 						  NULL))
// 	{
// 	    lpReOpenBuff->nErrCode = (WORD)GetLastError();
// 		return HFILE_ERROR;
// 	}
//
//     if (uStyle & OF_PARSE)
//     {
//         lpReOpenBuff->fFixedDisk = (GetDriveTypeA(lpReOpenBuff->szPathName) != DRIVE_REMOVABLE);
//         TRACE!("(%s): OF_PARSE, res = '%s'\n", lpFileName, lpReOpenBuff->szPathName);
//         return 0;
//     }
//
//     if ((uStyle & OF_EXIST) && !(uStyle & OF_CREATE))
//     {
//         DWORD dwAttributes = GetFileAttributesA(lpReOpenBuff->szPathName);
//
//         switch (dwAttributes)
//         {
//             case INVALID_FILE_ATTRIBUTES: /* File does not exist */
//                 SetLastError(ERROR_FILE_NOT_FOUND);
//                 lpReOpenBuff->nErrCode = (WORD) ERROR_FILE_NOT_FOUND;
//                 return -1;
//
//             case FILE_ATTRIBUTE_DIRECTORY:
//                 SetLastError(ERROR_ACCESS_DENIED);
//                 lpReOpenBuff->nErrCode = (WORD) ERROR_ACCESS_DENIED;
//                 return -1;
//
//             default:
//                 lpReOpenBuff->cBytes = sizeof(OFSTRUCT);
//                 return 1;
//         }
//     }
//     lpReOpenBuff->cBytes = sizeof(OFSTRUCT);
// 	if ((uStyle & OF_CREATE) == OF_CREATE)
// 	{
// 		DWORD Sharing;
// 		switch (uStyle & 0x70)
// 		{
// 			case OF_SHARE_EXCLUSIVE: Sharing = 0; break;
// 			case OF_SHARE_DENY_WRITE: Sharing = FILE_SHARE_READ; break;
// 			case OF_SHARE_DENY_READ: Sharing = FILE_SHARE_WRITE; break;
// 			case OF_SHARE_DENY_NONE:
// 			case OF_SHARE_COMPAT:
// 			default:
// 				Sharing = FILE_SHARE_READ | FILE_SHARE_WRITE;
// 		}
// 		return (HFILE)(ULONG_PTR) CreateFileA (lpFileName,
// 		                            GENERIC_READ | GENERIC_WRITE,
// 		                            Sharing,
// 		                            NULL,
// 		                            CREATE_ALWAYS,
// 		                            FILE_ATTRIBUTE_NORMAL,
// 		                            0);
// 	}
//
// 	RtlInitAnsiString (&FileName, (LPSTR)lpFileName);
//
// 	/* convert ansi (or oem) string to unicode */
// 	if (bIsFileApiAnsi)
// 		RtlAnsiStringToUnicodeString (&FileNameU, &FileName, TRUE);
// 	else
// 		RtlOemStringToUnicodeString (&FileNameU, &FileName, TRUE);
//
// 	Len = SearchPathW (NULL,
// 	                   FileNameU.Buffer,
//         	           NULL,
// 	                   OFS_MAXPATHNAME,
// 	                   PathNameW,
//         	           &FilePart);
//
// 	RtlFreeUnicodeString(&FileNameU);
//
// 	if (Len == 0 || Len > OFS_MAXPATHNAME)
// 	{
// 		lpReOpenBuff->nErrCode = (WORD)GetLastError();
// 		return HFILE_ERROR;
// 	}
//
//     if (uStyle & OF_DELETE)
//     {
//         if (!DeleteFileW(PathNameW))
// 		{
// 			lpReOpenBuff->nErrCode = (WORD)GetLastError();
// 			return HFILE_ERROR;
// 		}
//         TRACE!("(%s): OF_DELETE return = OK\n", lpFileName);
//         return TRUE;
//     }
//
// 	FileName.Buffer = lpReOpenBuff->szPathName;
// 	FileName.Length = 0;
// 	FileName.MaximumLength = OFS_MAXPATHNAME;
//
// 	RtlInitUnicodeString(&FileNameU, PathNameW);
//
// 	/* convert unicode string to ansi (or oem) */
// 	if (bIsFileApiAnsi)
// 		RtlUnicodeStringToAnsiString (&FileName, &FileNameU, FALSE);
// 	else
// 		RtlUnicodeStringToOemString (&FileName, &FileNameU, FALSE);
//
// 	if (!RtlDosPathNameToNtPathName_U (PathNameW,
// 					   &FileNameString,
// 					   NULL,
// 					   NULL))
// 	{
// 		return HFILE_ERROR;
// 	}
//
// 	// FILE_SHARE_READ
// 	// FILE_NO_INTERMEDIATE_BUFFERING
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileNameString,
//                                OBJ_CASE_INSENSITIVE | OBJ_INHERIT,
//                                NULL,
//                                NULL);
//
// 	errCode = NtOpenFile (&FileHandle,
// 	                      GENERIC_READ | SYNCHRONIZE,
// 	                      &ObjectAttributes,
// 	                      &IoStatusBlock,
// 	                      FILE_SHARE_READ,
// 	                      FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
//
// 	RtlFreeHeap(RtlGetProcessHeap(), 0, FileNameString.Buffer);
//
// 	lpReOpenBuff->nErrCode = (WORD)RtlNtStatusToDosError(errCode);
//
// 	if (!NT_SUCCESS(errCode))
// 	{
// 		BaseSetLastNTError (errCode);
// 		return HFILE_ERROR;
// 	}
//
// 	if (uStyle & OF_EXIST)
// 	{
// 		NtClose(FileHandle);
// 		return (HFILE)1;
// 	}
//
// 	return (HFILE)(ULONG_PTR)FileHandle;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// OpenDataFile(HANDLE hFile, DWORD dwUnused)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// HANDLE
// WINAPI
// ReOpenFile(IN HANDLE hOriginalFile,
//            IN DWORD dwDesiredAccess,
//            IN DWORD dwShareMode,
//            IN DWORD dwFlags)
// {
//    STUB;
//    return INVALID_HANDLE_VALUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
