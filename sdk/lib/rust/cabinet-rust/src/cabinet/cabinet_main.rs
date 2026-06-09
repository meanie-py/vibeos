use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn WARN(format: *const i8, ...);
}
extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn CreateFileA( lpFileName: LPCSTR, dwDesiredAccess: u32, dwShareMode: u32, lpSecurityAttributes: *mut SECURITY_ATTRIBUTES, dwCreationDisposition: u32, dwFlagsAndAttributes: u32, hTemplateFile: HANDLE, ) -> HANDLE;
    fn ReadFile( hFile: *mut core::ffi::c_void, lpBuffer: *mut core::ffi::c_void, nNumberOfBytesToRead: u32, lpNumberOfBytesRead: *mut u32, lpOverlapped: *mut core::ffi::c_void, ) -> i32;
    fn WriteFile(hFile: *mut core::ffi::c_void, lpBuffer: *const core::ffi::c_void, nNumberOfBytesToWrite: u32, lpNumberOfBytesWritten: *mut u32, lpOverlapped: *mut core::ffi::c_void) -> i32;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
    fn SetFilePointer( hFile: *mut core::ffi::c_void, lDistanceToMove: i32, lpDistanceToMoveHigh: *mut i32, dwMoveMethod: u32, ) -> u32;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn lstrcpyA(lpString1: *mut i8, lpString2: *const i8) -> *mut i8;
    fn strlen(s: *const i8) -> usize;
    fn lstrcmpiA(lpStr1: *const i8, lpStr2: *const i8) -> i32;
}

const E_INVALIDARG: i32 = 0x80070057u32 as i32;
const S_OK: i32 = 0;

unsafe extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetVersion(pdvi: *mut DLLVERSIONINFO) -> i32 {
    WARN(b"hmmm... not right version number \"5.1.1106.1\"?\n\0".as_ptr() as *const i8);
    if (*pdvi).cbSize as usize != core::mem::size_of::<DLLVERSIONINFO>() {
        return E_INVALIDARG;
    }
    (*pdvi).dwMajorVersion = 5;
    (*pdvi).dwMinorVersion = 1;
    (*pdvi).dwBuildNumber = 1106;
    (*pdvi).dwPlatformID = 1;
    return S_OK;
}

unsafe fn mem_alloc(cb: u32) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), 0, cb as usize)
}
unsafe fn mem_free(memory: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, memory);
}

const _O_RDONLY: i32 = 0;
const _O_WRONLY: i32 = 1;
const _O_RDWR: i32 = 2;
const _O_ACCMODE: i32 = 3;
const _O_CREAT: i32 = 0x0100;
const _O_EXCL: i32 = 0x0400;
const _O_TRUNC: i32 = 0x0200;
unsafe fn fdi_open(pszFile: *mut i8, oflag: i32, pmode: i32) -> isize {
    let mut handle: HANDLE;
    let mut dwAccess: u32 = 0;
    let mut dwShareMode: u32 = 0;
    let mut dwCreateDisposition: u32;

    match oflag & _O_ACCMODE {
        _O_RDONLY => {
            dwAccess = ntdll_sys::nt::GENERIC_READ;
            dwShareMode = ntdll_sys::nt::FILE_SHARE_READ | ntdll_sys::nt::FILE_SHARE_DELETE;
        }
        _O_WRONLY => {
            dwAccess = ntdll_sys::nt::GENERIC_WRITE;
            dwShareMode = ntdll_sys::nt::FILE_SHARE_READ | ntdll_sys::nt::FILE_SHARE_WRITE | ntdll_sys::nt::FILE_SHARE_DELETE;
        }
        _O_RDWR => {
            dwAccess = ntdll_sys::nt::GENERIC_READ | ntdll_sys::nt::GENERIC_WRITE;
            dwShareMode = ntdll_sys::nt::FILE_SHARE_READ | ntdll_sys::nt::FILE_SHARE_WRITE | ntdll_sys::nt::FILE_SHARE_DELETE;
        }
        _ => {}
    }

    if (oflag & _O_CREAT) != 0 {
        dwCreateDisposition = ntdll_sys::nt::OPEN_ALWAYS;
        if (oflag & _O_EXCL) != 0 {
            dwCreateDisposition = ntdll_sys::nt::CREATE_NEW;
        } else if (oflag & _O_TRUNC) != 0 {
            dwCreateDisposition = ntdll_sys::nt::CREATE_ALWAYS;
        }
    } else {
        dwCreateDisposition = ntdll_sys::nt::OPEN_EXISTING;
        if (oflag & _O_TRUNC) != 0 {
            dwCreateDisposition = ntdll_sys::nt::TRUNCATE_EXISTING;
        }
    }

    handle = CreateFileA(
        pszFile,
        dwAccess,
        dwShareMode,
        core::ptr::null_mut(),
        dwCreateDisposition,
        0,
        core::ptr::null_mut(),
    );

    return handle as isize;
}
unsafe fn fdi_read(hf: isize, pv: *mut core::ffi::c_void, cb: u32) -> u32 {
    let handle: *mut core::ffi::c_void = hf as *mut core::ffi::c_void;
    let mut dwRead: u32 = 0;
    if ReadFile(handle, pv, cb, &raw mut dwRead, core::ptr::null_mut()) != 0 {
        return dwRead;
    }
    return 0;
}
unsafe fn fdi_write(hf: isize, pv: *mut core::ffi::c_void, cb: u32) -> u32 {
    let handle: *mut core::ffi::c_void = hf as *mut core::ffi::c_void;
    let mut dwWritten: u32 = 0;

    if WriteFile(handle, pv as *const core::ffi::c_void, cb, &raw mut dwWritten, core::ptr::null_mut()) != 0 {
        return dwWritten;
    }

    0
}
unsafe fn fdi_close(hf: isize) -> i32 {
    let handle: *mut core::ffi::c_void = hf as *mut core::ffi::c_void;
    if CloseHandle(handle) != 0 { 0 } else { -1 }
}
unsafe fn fdi_seek(hf: INT_PTR, dist: i32, seektype: i32) -> i32 {
    let handle = hf as *mut core::ffi::c_void;
    return SetFilePointer(handle, dist, core::ptr::null_mut(), seektype as u32) as i32;
}

unsafe fn fill_file_node(pNode: *mut FILELIST, szFilename: *const i8) {
    (*pNode).next = core::ptr::null_mut();
    (*pNode).DoExtract = 0;
    let len = strlen(szFilename);
    let alloc_size = len + 1;
    let heap = GetProcessHeap();
    let alloc = HeapAlloc(heap, 0, alloc_size);
    (*pNode).FileName = alloc as *mut i8;
    lstrcpyA((*pNode).FileName, szFilename);
}

unsafe fn file_in_list(mut pNode: *mut FILELIST, szFilename: *const i8, pOut: *mut *mut FILELIST) -> i32 {
    while !pNode.is_null() {
        if lstrcmpiA((*pNode).FileName, szFilename) == 0 {
            if !pOut.is_null() {
                *pOut = pNode;
            }
            return 1;
        }
        pNode = (*pNode).next;
    }
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * cabinet.dll main
//  *
//  * Copyright 2002 Patrik Stridvall
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "config.h"
//
// #include <assert.h>
// #include <stdarg.h>
// #include <string.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "winerror.h"
// #define NO_SHLWAPI_REG
// #include "shlwapi.h"
// #undef NO_SHLWAPI_REG
//
// #include "cabinet.h"
//
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(cabinet);
//
//
// /***********************************************************************
//  * DllGetVersion (CABINET.2)
//  *
//  * Retrieves version information of the 'CABINET.DLL'
//  *
//  * PARAMS
//  *     pdvi [O] pointer to version information structure.
//  *
//  * RETURNS
//  *     Success: S_OK
//  *     Failure: E_INVALIDARG
//  *
//  * NOTES
//  *     Supposedly returns version from IE6SP1RP1
//  */
// HRESULT WINAPI DllGetVersion (DLLVERSIONINFO *pdvi)
// {
//   WARN("hmmm... not right version number \"5.1.1106.1\"?\n");
//
//   if (pdvi->cbSize != sizeof(DLLVERSIONINFO)) return E_INVALIDARG;
//
//   pdvi->dwMajorVersion = 5;
//   pdvi->dwMinorVersion = 1;
//   pdvi->dwBuildNumber = 1106;
//   pdvi->dwPlatformID = 1;
//
//   return S_OK;
// }
//
// /* FDI callback functions */
//
// static void * CDECL mem_alloc(ULONG cb)
// {
//     return HeapAlloc(GetProcessHeap(), 0, cb);
// }
//
// static void CDECL mem_free(void *memory)
// {
//     HeapFree(GetProcessHeap(), 0, memory);
// }
//
// static INT_PTR CDECL fdi_open(char *pszFile, int oflag, int pmode)
// {
//     HANDLE handle;
//     DWORD dwAccess = 0;
//     DWORD dwShareMode = 0;
//     DWORD dwCreateDisposition;
//
//     switch (oflag & _O_ACCMODE)
//     {
//         case _O_RDONLY:
//             dwAccess = GENERIC_READ;
//             dwShareMode = FILE_SHARE_READ | FILE_SHARE_DELETE;
//             break;
//         case _O_WRONLY:
//             dwAccess = GENERIC_WRITE;
//             dwShareMode = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
//             break;
//         case _O_RDWR:
//             dwAccess = GENERIC_READ | GENERIC_WRITE;
//             dwShareMode = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
//             break;
//     }
//
//     if (oflag & _O_CREAT)
//     {
//         dwCreateDisposition = OPEN_ALWAYS;
//         if (oflag & _O_EXCL) dwCreateDisposition = CREATE_NEW;
//         else if (oflag & _O_TRUNC) dwCreateDisposition = CREATE_ALWAYS;
//     }
//     else
//     {
//         dwCreateDisposition = OPEN_EXISTING;
//         if (oflag & _O_TRUNC) dwCreateDisposition = TRUNCATE_EXISTING;
//     }
//
//     handle = CreateFileA(pszFile, dwAccess, dwShareMode, NULL,
//                          dwCreateDisposition, 0, NULL);
//
//     return (INT_PTR) handle;
// }
//
// static UINT CDECL fdi_read(INT_PTR hf, void *pv, UINT cb)
// {
//     HANDLE handle = (HANDLE) hf;
//     DWORD dwRead;
//     
//     if (ReadFile(handle, pv, cb, &dwRead, NULL))
//         return dwRead;
//
//     return 0;
// }
//
// static UINT CDECL fdi_write(INT_PTR hf, void *pv, UINT cb)
// {
//     HANDLE handle = (HANDLE) hf;
//     DWORD dwWritten;
//
//     if (WriteFile(handle, pv, cb, &dwWritten, NULL))
//         return dwWritten;
//
//     return 0;
// }
//
// static int CDECL fdi_close(INT_PTR hf)
// {
//     HANDLE handle = (HANDLE) hf;
//     return CloseHandle(handle) ? 0 : -1;
// }
//
// static LONG CDECL fdi_seek(INT_PTR hf, LONG dist, int seektype)
// {
//     HANDLE handle = (HANDLE) hf;
//     return SetFilePointer(handle, dist, NULL, seektype);
// }
//
// static void fill_file_node(struct FILELIST *pNode, LPCSTR szFilename)
// {
//     pNode->next = NULL;
//     pNode->DoExtract = FALSE;
//
//     pNode->FileName = HeapAlloc(GetProcessHeap(), 0, strlen(szFilename) + 1);
//     lstrcpyA(pNode->FileName, szFilename);
// }
//
// static BOOL file_in_list(struct FILELIST *pNode, LPCSTR szFilename,
//                          struct FILELIST **pOut)
// {
//     while (pNode)
//     {
//         if (!lstrcmpiA(pNode->FileName, szFilename))
//         {
//             if (pOut)
//                 *pOut = pNode;
//
//             return TRUE;
//         }
//
//         pNode = pNode->next;
//     }
//
//     return FALSE;
// }
//
// static INT_PTR CDECL fdi_notify_extract(FDINOTIFICATIONTYPE fdint, PFDINOTIFICATION pfdin)
// {
//     switch (fdint)
//     {
//         case fdintCOPY_FILE:
//         {
//             struct FILELIST *fileList, *node = NULL;
//             SESSION *pDestination = pfdin->pv;
//             LPSTR szFullPath, szDirectory;
//             HANDLE hFile = 0;
//             DWORD dwSize;
//
//             dwSize = lstrlenA(pDestination->Destination) +
//                     lstrlenA("\\") + lstrlenA(pfdin->psz1) + 1;
//             szFullPath = HeapAlloc(GetProcessHeap(), 0, dwSize);
//
//             lstrcpyA(szFullPath, pDestination->Destination);
//             lstrcatA(szFullPath, "\\");
//             lstrcatA(szFullPath, pfdin->psz1);
//
//             /* pull out the destination directory string from the full path */
//             dwSize = strrchr(szFullPath, '\\') - szFullPath + 1;
//             szDirectory = HeapAlloc(GetProcessHeap(), 0, dwSize);
//             lstrcpynA(szDirectory, szFullPath, dwSize);
//
//             pDestination->FileSize += pfdin->cb;
//
//             if (pDestination->Operation & EXTRACT_FILLFILELIST)
//             {
//                 fileList = HeapAlloc(GetProcessHeap(), 0,
//                                      sizeof(struct FILELIST));
//
//                 fill_file_node(fileList, pfdin->psz1);
//                 fileList->DoExtract = TRUE;
//                 fileList->next = pDestination->FileList;
//                 pDestination->FileList = fileList;
//                 lstrcpyA(pDestination->CurrentFile, szFullPath);
//                 pDestination->FileCount++;
//             }
//
//             if ((pDestination->Operation & EXTRACT_EXTRACTFILES) ||
//                 file_in_list(pDestination->FilterList, pfdin->psz1, NULL))
//             {
// 		/* find the file node */
//                 file_in_list(pDestination->FileList, pfdin->psz1, &node);
//
//                 if (node && !node->DoExtract)
//                 {
//                     HeapFree(GetProcessHeap(), 0, szFullPath);
//                     HeapFree(GetProcessHeap(), 0, szDirectory);
//                     return 0;
//                 }
//
//                 /* create the destination directory if it doesn't exist */
//                 if (GetFileAttributesA(szDirectory) == INVALID_FILE_ATTRIBUTES)
//                 {
//                     char *ptr;
//
//                     for(ptr = szDirectory + strlen(pDestination->Destination)+1; *ptr; ptr++) {
//                         if(*ptr == '\\') {
//                             *ptr = 0;
//                             CreateDirectoryA(szDirectory, NULL);
//                             *ptr = '\\';
//                         }
//                     }
//                     CreateDirectoryA(szDirectory, NULL);
//                 }
//
//                 hFile = CreateFileA(szFullPath, GENERIC_READ | GENERIC_WRITE, 0, NULL,
//                                     CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
//
//                 if (hFile != INVALID_HANDLE_VALUE && node)
//                     node->DoExtract = FALSE;
//             }
//
//             HeapFree(GetProcessHeap(), 0, szFullPath);
//             HeapFree(GetProcessHeap(), 0, szDirectory);
//
//             return (INT_PTR) hFile;
//         }
//
//         case fdintCLOSE_FILE_INFO:
//         {
//             FILETIME ft;
//             FILETIME ftLocal;
//             HANDLE handle = (HANDLE) pfdin->hf;
//
//             if (!DosDateTimeToFileTime(pfdin->date, pfdin->time, &ft))
//                 return FALSE;
//
//             if (!LocalFileTimeToFileTime(&ft, &ftLocal))
//                 return FALSE;
//
//             if (!SetFileTime(handle, &ftLocal, 0, &ftLocal))
//                 return FALSE;
//
//             CloseHandle(handle);
//             return TRUE;
//         }
//
//         default:
//             return 0;
//     }
// }
//
// /***********************************************************************
//  * Extract (CABINET.3)
//  *
//  * Extracts the contents of the cabinet file to the specified
//  * destination.
//  *
//  * PARAMS
//  *   dest      [I/O] Controls the operation of Extract.  See NOTES.
//  *   szCabName [I] Filename of the cabinet to extract.
//  *
//  * RETURNS
//  *     Success: S_OK.
//  *     Failure: E_FAIL.
//  *
//  * NOTES
//  *   The following members of the dest struct control the operation
//  *   of Extract:
//  *       FileSize    [O] The size of all files extracted up to CurrentFile.
//  *       Error       [O] The error in case the extract operation fails.
//  *       FileList    [I] A linked list of filenames.  Extract only extracts
//  *                       files from the cabinet that are in this list.
//  *       FileCount   [O] Contains the number of files in FileList on
//  *                       completion.
//  *       Operation   [I] See Operation.
//  *       Destination [I] The destination directory.
//  *       CurrentFile [O] The last file extracted.
//  *       FilterList  [I] A linked list of files that should not be extracted.
//  *
//  *   Operation
//  *     If Operation contains EXTRACT_FILLFILELIST, then FileList will be
//  *     filled with all the files in the cabinet.  If Operation contains
//  *     EXTRACT_EXTRACTFILES, then only the files in the FileList will
//  *     be extracted from the cabinet.  EXTRACT_FILLFILELIST can be called
//  *     by itself, but EXTRACT_EXTRACTFILES must have a valid FileList
//  *     in order to succeed.  If Operation contains both EXTRACT_FILLFILELIST
//  *     and EXTRACT_EXTRACTFILES, then all the files in the cabinet
//  *     will be extracted.
//  */
// HRESULT WINAPI Extract(SESSION *dest, LPCSTR szCabName)
// {
//     HRESULT res = S_OK;
//     HFDI hfdi;
//     char *str, *end, *path = NULL, *name = NULL;
//
//     TRACE("(%p, %s)\n", dest, debugstr_a(szCabName));
//
//     hfdi = FDICreate(mem_alloc,
//                      mem_free,
//                      fdi_open,
//                      fdi_read,
//                      fdi_write,
//                      fdi_close,
//                      fdi_seek,
//                      cpuUNKNOWN,
//                      &dest->Error);
//
//     if (!hfdi)
//         return E_FAIL;
//
//     if (GetFileAttributesA(dest->Destination) == INVALID_FILE_ATTRIBUTES)
//     {
//         res = S_OK;
//         goto end;
//     }
//
//     /* split the cabinet name into path + name */
//     str = HeapAlloc(GetProcessHeap(), 0, lstrlenA(szCabName)+1);
//     if (!str)
//     {
//         res = E_OUTOFMEMORY;
//         goto end;
//     }
//     lstrcpyA(str, szCabName);
//
//     if ((end = strrchr(str, '\\')))
//     {
//         path = str;
//         end++;
//         name = HeapAlloc( GetProcessHeap(), 0, strlen(end) + 1 );
//         if (!name)
//         {
//             res = E_OUTOFMEMORY;
//             goto end;
//         }
//         strcpy( name, end );
//         *end = 0;
//     }
//     else
//     {
//         name = str;
//         path = NULL;
//     }
//
//     dest->FileSize = 0;
//
//     if (!FDICopy(hfdi, name, path, 0,
//          fdi_notify_extract, NULL, dest))
//         res = HRESULT_FROM_WIN32(GetLastError());
//
// end:
//     HeapFree(GetProcessHeap(), 0, path);
//     HeapFree(GetProcessHeap(), 0, name);
//     FDIDestroy(hfdi);
//     return res;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
