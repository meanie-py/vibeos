use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn AppendBackslash(lpPath: LPWSTR) -> LPWSTR;
    fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
    fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    fn wcschr(s: *const WCHAR, c: i32) -> LPWSTR;
    fn wcslen(s: *const WCHAR) -> usize;
}
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;

// L"\*.*"
static WILDCARD_ALL_W: [u16; 5] = [b'\\' as u16, b'*' as u16, b'.' as u16, b'*' as u16, 0];
// L"."
static DOT_W: [u16; 2] = [b'.' as u16, 0];
// L".."
static DOT_DOT_W: [u16; 3] = [b'.' as u16, b'.' as u16, 0];
// L"\"
static BACKSLASH_W: [u16; 2] = [b'\\' as u16, 0];

#[no_mangle]
pub unsafe extern "system" fn CopyProfileDirectoryA(
    lpSourcePath: LPCSTR,
    lpDestinationPath: LPCSTR,
    dwFlags: DWORD,
) -> BOOL {
    let mut SrcPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DstPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bResult: BOOL = 0;
    
    if RtlCreateUnicodeStringFromAsciiz(&mut SrcPath, lpSourcePath as LPSTR) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    
    if RtlCreateUnicodeStringFromAsciiz(&mut DstPath, lpDestinationPath as LPSTR) == 0 {
        RtlFreeUnicodeString(&mut SrcPath);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    
    bResult = CopyProfileDirectoryW(SrcPath.Buffer, DstPath.Buffer, dwFlags);
    
    RtlFreeUnicodeString(&mut DstPath);
    RtlFreeUnicodeString(&mut SrcPath);
    
    bResult
}

#[no_mangle]
pub unsafe extern "system" fn CopyProfileDirectoryW(
    lpSourcePath: LPCWSTR,
    lpDestinationPath: LPCWSTR,
    _dwFlags: DWORD,
) -> BOOL {
    CopyDirectory(lpDestinationPath, lpSourcePath)
}
#[no_mangle]
pub unsafe extern "C" fn CopyDirectory(lpDestinationPath: LPCWSTR, lpSourcePath: LPCWSTR) -> BOOL {
    let mut szFileName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szFullSrcName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szFullDstName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut FindFileData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut lpSrcPtr: LPWSTR;
    let mut lpDstPtr: LPWSTR;
    let mut hFind: *mut core::ffi::c_void;

    wcscpy(szFileName.as_mut_ptr(), lpSourcePath);
    wcscat(szFileName.as_mut_ptr(), WILDCARD_ALL_W.as_ptr() as *const u16);

    hFind = FindFirstFileW(szFileName.as_ptr(), &mut FindFileData);
    if hFind == INVALID_HANDLE_VALUE {
        crate::DbgPrint(
            c"[USERENV] CopyDirectory: FindFirstFileW(%ws) failed (error %lu)\n".as_ptr(),
            szFileName.as_ptr(),
            GetLastError(),
        );
        return 0;
    }

    wcscpy(szFullSrcName.as_mut_ptr(), lpSourcePath);
    lpSrcPtr = AppendBackslash(szFullSrcName.as_mut_ptr());

    wcscpy(szFullDstName.as_mut_ptr(), lpDestinationPath);
    lpDstPtr = AppendBackslash(szFullDstName.as_mut_ptr());

    loop {
        if wcscmp(FindFileData.cFileName.as_ptr(), DOT_W.as_ptr() as *const u16) != 0 &&
           wcscmp(FindFileData.cFileName.as_ptr(), DOT_DOT_W.as_ptr() as *const u16) != 0
        {
            wcscpy(lpSrcPtr, FindFileData.cFileName.as_ptr());
            wcscpy(lpDstPtr, FindFileData.cFileName.as_ptr());

            if (FindFileData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                if CreateDirectoryExW(szFullSrcName.as_ptr(), szFullDstName.as_ptr(), core::ptr::null_mut()) == 0 {
                    if GetLastError() != ERROR_ALREADY_EXISTS {
                        crate::DbgPrint(
                            c"[USERENV] CopyDirectory: CreateDirectoryExW(%ws -> %ws) failed (error %lu)\n".as_ptr(),
                            szFullSrcName.as_ptr(),
                            szFullDstName.as_ptr(),
                            GetLastError(),
                        );
                        FindClose(hFind);
                        return 0;
                    }
                }

                if CopyDirectory(szFullDstName.as_ptr(), szFullSrcName.as_ptr()) == 0 {
                    FindClose(hFind);
                    return 0;
                }
            } else {
                if CopyFileW(szFullSrcName.as_ptr(), szFullDstName.as_ptr(), 0) == 0 {
                    crate::DbgPrint(
                        c"[USERENV] CopyDirectory: CopyFileW(%ws -> %ws) failed (error %lu)\n".as_ptr(),
                        szFullSrcName.as_ptr(),
                        szFullDstName.as_ptr(),
                        GetLastError(),
                    );
                    FindClose(hFind);
                    return 0;
                }
            }
        }

        if FindNextFileW(hFind, &mut FindFileData) == 0 {
            break;
        }
    }

    FindClose(hFind);
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn CreateDirectoryPath(
    mut lpPathName: LPCWSTR,
    mut lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> BOOL {
    let mut szPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut Ptr: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut dwError: DWORD = 0;

    if lpPathName.is_null() || *lpPathName as i32 == 0 {
        return TRUE;
    }

    if CreateDirectoryW(lpPathName, lpSecurityAttributes) != 0 {
        return TRUE;
    }

    dwError = GetLastError();
    if dwError == ERROR_ALREADY_EXISTS as u32 {
        return TRUE;
    }

    wcscpy(
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR,
        lpPathName as *const WCHAR,
    );

    if wcslen(core::ptr::addr_of_mut!(szPath) as *const WCHAR) > 3
        && szPath[1] as i32 == ':' as i32
        && szPath[2] as i32 == '\\' as i32
    {
        Ptr = (core::ptr::addr_of_mut!(szPath) as *mut WCHAR).offset(3) as LPWSTR;
    } else {
        Ptr = core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPWSTR;
    }

    while !Ptr.is_null() {
        Ptr = wcschr(Ptr as *const WCHAR, '\\' as i32) as LPWSTR;
        if !Ptr.is_null() {
            *Ptr = 0 as WCHAR;
        }

        if CreateDirectoryW(
            core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
            lpSecurityAttributes,
        ) == 0
        {
            dwError = GetLastError();
            if dwError != ERROR_ALREADY_EXISTS as u32 {
                return FALSE;
            }
        }

        if !Ptr.is_null() {
            *Ptr = '\\' as i32 as WCHAR;
            Ptr = Ptr.offset(1);
        }
    }

    return TRUE;
}
unsafe extern "C" fn RecursiveRemoveDir(lpPath: LPCWSTR) -> i32 {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE;
    let mut bResult: i32;

    wcscpy(szPath.as_mut_ptr(), lpPath);
    wcscat(szPath.as_mut_ptr(), WILDCARD_ALL_W.as_ptr() as *const u16);

    hFind = FindFirstFileW(szPath.as_ptr() as LPCWSTR, &mut FindData);
    if hFind == INVALID_HANDLE_VALUE {
        return 0;
    }

    bResult = 1;
    loop {
        if wcscmp(FindData.cFileName.as_ptr(), DOT_W.as_ptr() as *const u16) != 0
            && wcscmp(FindData.cFileName.as_ptr(), DOT_DOT_W.as_ptr() as *const u16) != 0
        {
            wcscpy(szPath.as_mut_ptr(), lpPath);
            wcscat(szPath.as_mut_ptr(), BACKSLASH_W.as_ptr() as *const u16);
            wcscat(szPath.as_mut_ptr(), FindData.cFileName.as_ptr());

            if FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                if RecursiveRemoveDir(szPath.as_ptr() as LPCWSTR) == 0 {
                    bResult = 0;
                    break;
                }

                if FindData.dwFileAttributes & FILE_ATTRIBUTE_READONLY != 0 {
                    SetFileAttributesW(
                        szPath.as_ptr() as LPCWSTR,
                        FindData.dwFileAttributes & !FILE_ATTRIBUTE_READONLY,
                    );
                }

                if RemoveDirectoryW(szPath.as_ptr() as LPCWSTR) == 0 {
                    bResult = 0;
                    break;
                }
            } else {
                if FindData.dwFileAttributes & (FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM) != 0
                {
                    SetFileAttributesW(szPath.as_ptr() as LPCWSTR, FILE_ATTRIBUTE_NORMAL);
                }

                if DeleteFileW(szPath.as_ptr() as LPCWSTR) == 0 {
                    bResult = 0;
                    break;
                }
            }
        }

        if FindNextFileW(hFind, &mut FindData) == 0 {
            if GetLastError() != ERROR_NO_MORE_FILES {
                bResult = 0;
                break;
            }

            break;
        }
    }

    FindClose(hFind);

    bResult
}
#[no_mangle]
pub unsafe extern "C" fn RemoveDirectoryPath(lpPathName: LPCWSTR) -> BOOL {
    if RecursiveRemoveDir(lpPathName) == 0 {
        return FALSE;
    }
    return RemoveDirectoryW(lpPathName);
}
// enforce_import_abi: moved 10 import decl(s) into the extern block their
// exporting DLL's .spec requires: CopyFileW->system, CreateDirectoryExW->system, CreateDirectoryW->system, DeleteFileW->system, FindClose->system, FindFirstFileW->system, FindNextFileW->system, GetLastError->system, RemoveDirectoryW->system, SetFileAttributesW->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "system" {
    fn CopyFileW(lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL) -> BOOL;
    fn CreateDirectoryExW(lpTemplateDirectory: LPCWSTR, lpNewDirectory: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES) -> BOOL;
    fn FindClose(hFindFile: *mut core::ffi::c_void) -> BOOL;
    fn FindFirstFileW(lpFileName: LPCWSTR, lpFindFileData: *mut WIN32_FIND_DATAW) -> *mut core::ffi::c_void;
    fn FindNextFileW(hFindFile: *mut core::ffi::c_void, lpFindFileData: *mut WIN32_FIND_DATAW) -> BOOL;
    fn GetLastError() -> u32;
    fn CreateDirectoryW(lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES) -> BOOL;
    fn DeleteFileW(lpFileName: LPCWSTR) -> i32;
    fn RemoveDirectoryW(lpPathName: LPCWSTR) -> i32;
    fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: u32) -> i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2004 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/userenv/directory.c
//  * PURPOSE:         User profile code
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ***************************************************************/
//
// BOOL
// WINAPI
// CopyProfileDirectoryA(LPCSTR lpSourcePath,
//                       LPCSTR lpDestinationPath,
//                       DWORD dwFlags)
// {
//     UNICODE_STRING SrcPath;
//     UNICODE_STRING DstPath;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&SrcPath,
//                                           (LPSTR)lpSourcePath))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&DstPath,
//                                           (LPSTR)lpDestinationPath))
//     {
//         RtlFreeUnicodeString(&SrcPath);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     bResult = CopyProfileDirectoryW(SrcPath.Buffer,
//                                     DstPath.Buffer,
//                                     dwFlags);
//
//     RtlFreeUnicodeString(&DstPath);
//     RtlFreeUnicodeString(&SrcPath);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// CopyProfileDirectoryW(LPCWSTR lpSourcePath,
//                       LPCWSTR lpDestinationPath,
//                       DWORD dwFlags)
// {
//     /* FIXME: dwFlags are ignored! */
//     return CopyDirectory(lpDestinationPath, lpSourcePath);
// }
//
//
// BOOL
// CopyDirectory(LPCWSTR lpDestinationPath,
//               LPCWSTR lpSourcePath)
// {
//     WCHAR szFileName[MAX_PATH];
//     WCHAR szFullSrcName[MAX_PATH];
//     WCHAR szFullDstName[MAX_PATH];
//     WIN32_FIND_DATAW FindFileData;
//     LPWSTR lpSrcPtr;
//     LPWSTR lpDstPtr;
//     HANDLE hFind;
//
//     DPRINT("CopyDirectory (%S, %S) called\n",
//             lpDestinationPath, lpSourcePath);
//
//     wcscpy(szFileName, lpSourcePath);
//     wcscat(szFileName, L"\\*.*");
//
//     hFind = FindFirstFileW(szFileName,
//                            &FindFileData);
//     if (hFind == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("Error: %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     wcscpy(szFullSrcName, lpSourcePath);
//     lpSrcPtr = AppendBackslash(szFullSrcName);
//
//     wcscpy(szFullDstName, lpDestinationPath);
//     lpDstPtr = AppendBackslash(szFullDstName);
//
//     for (;;)
//     {
//         if (wcscmp(FindFileData.cFileName, L".") &&
//             wcscmp(FindFileData.cFileName, L".."))
//         {
//             wcscpy(lpSrcPtr, FindFileData.cFileName);
//             wcscpy(lpDstPtr, FindFileData.cFileName);
//
//             if (FindFileData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
//             {
//                 DPRINT("Create directory: %S\n", szFullDstName);
//                 if (!CreateDirectoryExW(szFullSrcName, szFullDstName, NULL))
//                 {
//                     if (GetLastError() != ERROR_ALREADY_EXISTS)
//                     {
//                         DPRINT1("Error: %lu\n", GetLastError());
//
//                         FindClose(hFind);
//                         return FALSE;
//                     }
//                 }
//
//                 if (!CopyDirectory(szFullDstName, szFullSrcName))
//                 {
//                     DPRINT1("Error: %lu\n", GetLastError());
//
//                     FindClose(hFind);
//                     return FALSE;
//                 }
//             }
//             else
//             {
//                 DPRINT("Copy file: %S -> %S\n", szFullSrcName, szFullDstName);
//                 if (!CopyFileW(szFullSrcName, szFullDstName, FALSE))
//                 {
//                     DPRINT1("Error: %lu\n", GetLastError());
//
//                     FindClose(hFind);
//                     return FALSE;
//                 }
//             }
//         }
//
//         if (!FindNextFileW(hFind, &FindFileData))
//         {
//             if (GetLastError() != ERROR_NO_MORE_FILES)
//             {
//                 DPRINT1("Error: %lu\n", GetLastError());
//             }
//
//             break;
//         }
//     }
//
//     FindClose(hFind);
//
//     DPRINT("CopyDirectory() done\n");
//
//     return TRUE;
// }
//
//
// BOOL
// CreateDirectoryPath(LPCWSTR lpPathName,
//                     LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     WCHAR szPath[MAX_PATH];
//     LPWSTR Ptr;
//     DWORD dwError;
//
//     DPRINT("CreateDirectoryPath() called\n");
//
//     if (lpPathName == NULL || *lpPathName == 0)
//         return TRUE;
//
//     if (CreateDirectoryW(lpPathName,
//                          lpSecurityAttributes))
//         return TRUE;
//
//     dwError = GetLastError();
//     if (dwError == ERROR_ALREADY_EXISTS)
//         return TRUE;
//
//     wcscpy(szPath, lpPathName);
//
//     if (wcslen(szPath) > 3 && szPath[1] == ':' && szPath[2] == '\\')
//     {
//         Ptr = &szPath[3];
//     }
//     else
//     {
//         Ptr = szPath;
//     }
//
//     while (Ptr != NULL)
//     {
//         Ptr = wcschr(Ptr, L'\\');
//         if (Ptr != NULL)
//             *Ptr = 0;
//
//         DPRINT("CreateDirectory(%S)\n", szPath);
//         if (!CreateDirectoryW(szPath,
//                               lpSecurityAttributes))
//         {
//             dwError = GetLastError();
//             if (dwError != ERROR_ALREADY_EXISTS)
//                 return FALSE;
//         }
//
//         if (Ptr != NULL)
//         {
//             *Ptr = L'\\';
//             Ptr++;
//         }
//     }
//
//     DPRINT("CreateDirectoryPath() done\n");
//
//     return TRUE;
// }
//
//
// static
// BOOL
// RecursiveRemoveDir(LPCWSTR lpPath)
// {
//     WCHAR szPath[MAX_PATH];
//     WIN32_FIND_DATAW FindData;
//     HANDLE hFind;
//     BOOL bResult;
//
//     wcscpy(szPath, lpPath);
//     wcscat(szPath, L"\\*.*");
//     DPRINT("Search path: '%S'\n", szPath);
//
//     hFind = FindFirstFileW(szPath,
//                            &FindData);
//     if (hFind == INVALID_HANDLE_VALUE)
//         return FALSE;
//
//     bResult = TRUE;
//     while (TRUE)
//     {
//         if (wcscmp(FindData.cFileName, L".") &&
//             wcscmp(FindData.cFileName, L".."))
//         {
//             wcscpy(szPath, lpPath);
//             wcscat(szPath, L"\\");
//             wcscat(szPath, FindData.cFileName);
//             DPRINT("File name: '%S'\n", szPath);
//
//             if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
//             {
//                 DPRINT("Delete directory: '%S'\n", szPath);
//
//                 if (!RecursiveRemoveDir(szPath))
//                 {
//                     bResult = FALSE;
//                     break;
//                 }
//
//                 if (FindData.dwFileAttributes & FILE_ATTRIBUTE_READONLY)
//                 {
//                     SetFileAttributesW(szPath,
//                                        FindData.dwFileAttributes & ~FILE_ATTRIBUTE_READONLY);
//                 }
//
//                 if (!RemoveDirectoryW(szPath))
//                 {
//                     bResult = FALSE;
//                     break;
//                 }
//             }
//             else
//             {
//                 DPRINT("Delete file: '%S'\n", szPath);
//
//                 if (FindData.dwFileAttributes & (FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM))
//                 {
//                     SetFileAttributesW(szPath,
//                                        FILE_ATTRIBUTE_NORMAL);
//                 }
//
//                 if (!DeleteFileW(szPath))
//                 {
//                     bResult = FALSE;
//                     break;
//                 }
//             }
//         }
//
//         if (!FindNextFileW(hFind, &FindData))
//         {
//             if (GetLastError() != ERROR_NO_MORE_FILES)
//             {
//                 DPRINT1("Error: %lu\n", GetLastError());
//                 bResult = FALSE;
//                 break;
//             }
//
//             break;
//         }
//     }
//
//     FindClose(hFind);
//
//     return bResult;
// }
//
//
// BOOL
// RemoveDirectoryPath(LPCWSTR lpPathName)
// {
//     if (!RecursiveRemoveDir(lpPathName))
//         return FALSE;
//
//     DPRINT("Delete directory: '%S'\n", lpPathName);
//     return RemoveDirectoryW(lpPathName);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
