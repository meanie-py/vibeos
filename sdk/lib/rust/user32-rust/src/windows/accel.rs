use crate::ntwrapper::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::message::EnterCriticalSection;
use crate::windows::message::LeaveCriticalSection;
use crate::windows::cursoricon::LoadResource;
use win32k_sys::win32k::*;
use crate::internal::*;
use crate::winbase::*;
use crate::FIELD_OFFSET;
use core::ffi::c_void;

extern "system" {
    fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> HGLOBAL;
    fn LocalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn NtUserCreateAcceleratorTable(Entries: LPACCEL, EntriesCount: SIZE_T) -> HACCEL;
    fn NtUserCopyAcceleratorTable(Table: HACCEL, Entries: LPACCEL, EntriesCount: ULONG) -> ULONG;
    fn SizeofResource(hModule: HINSTANCE, hResInfo: HRSRC) -> DWORD;
    fn NtUserTranslateAccelerator(hWnd: HWND, hAccTable: HACCEL, lpMsg: LPMSG) -> core::ffi::c_int;
    fn FindResourceExW(hModule: HINSTANCE, lpType: LPCWSTR, lpName: LPCWSTR, wLanguage: u16) -> HRSRC;
    fn FindResourceExA(hModule: HINSTANCE, lpType: LPCSTR, lpName: LPCSTR, wLanguage: u16) -> HRSRC;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PE_ACCEL {
    pub fVirt: BYTE,
    pub pad0: BYTE,
    pub key: WORD,
    pub cmd: WORD,
    pub pad1: WORD,
}
pub type LPPE_ACCEL = *mut PE_ACCEL;

#[repr(C)]
pub struct _USER_ACCEL_CACHE_ENTRY {
    pub Next: *mut _USER_ACCEL_CACHE_ENTRY,
    pub Usage: ULONG_PTR,
    pub Object: HACCEL,
    pub Data: HGLOBAL,
}
pub type U32_ACCEL_CACHE_ENTRY = _USER_ACCEL_CACHE_ENTRY;

#[no_mangle]
pub static mut U32AccelCacheLock: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut U32AccelCache: *mut U32_ACCEL_CACHE_ENTRY = core::ptr::null_mut();

// Look up a handle or resource address in the cache. Returns a pointer to the
// list *link* that points to the matching entry (so callers can unlink without a
// doubly-linked list); on miss it points at the trailing NULL. (accel.c:60)
#[no_mangle]
pub unsafe extern "system" fn U32AccelCacheFind(
    Object: HANDLE,
    Data: HGLOBAL,
) -> *mut *mut U32_ACCEL_CACHE_ENTRY {
    let mut ppEntry: *mut *mut U32_ACCEL_CACHE_ENTRY = core::ptr::addr_of_mut!(U32AccelCache);
    while !(*ppEntry).is_null() {
        if (**ppEntry).Object as usize == Object as usize
            || (**ppEntry).Data as usize == Data as usize
        {
            break;
        }
        ppEntry = core::ptr::addr_of_mut!((**ppEntry).Next);
    }
    ppEntry
}

#[no_mangle]
pub unsafe extern "system" fn U32AccelCacheAdd(Object: HACCEL, Data: HGLOBAL) {
    let pEntry = LocalAlloc(
        LMEM_FIXED as u32,
        core::mem::size_of::<U32_ACCEL_CACHE_ENTRY>() as SIZE_T,
    ) as *mut U32_ACCEL_CACHE_ENTRY;

    if pEntry.is_null() {
        return;
    }

    (*pEntry).Usage = 1;
    (*pEntry).Object = Object;
    (*pEntry).Data = Data;

    (*pEntry).Next = U32AccelCache;
    U32AccelCache = pEntry;
}

#[no_mangle]
pub unsafe extern "system" fn U32LoadAccelerators(hInstance: HINSTANCE, hTableRes: HRSRC) -> HACCEL {
    let hAccTableData: HGLOBAL;
    let mut hAccTable: HACCEL = core::ptr::null_mut();
    let pEntry: *mut U32_ACCEL_CACHE_ENTRY;
    let pAccTableResData: *mut PE_ACCEL;
    let mut i: SIZE_T = 0;
    let mut j: SIZE_T;
    let pAccTableData: *mut ACCEL;

    /* load the accelerator table */
    hAccTableData = LoadResource(hInstance, hTableRes);

    /* failure */
    if hAccTableData.is_null() {
        return core::ptr::null_mut();
    }

    EnterCriticalSection((&raw mut U32AccelCacheLock).cast());

    'l_Leave: {
        /* see if this accelerator table has already been loaded */
        pEntry = *U32AccelCacheFind(core::ptr::null_mut(), hAccTableData);

        /* accelerator table already loaded */
        if !pEntry.is_null() {
            /* increment the reference count */
            (*pEntry).Usage += 1;

            /* return the existing object */
            hAccTable = (*pEntry).Object;

            /* success */
            break 'l_Leave;
        }

        /* determine the number of entries in the table */
        i = (SizeofResource(hInstance, hTableRes) as SIZE_T) / core::mem::size_of::<PE_ACCEL>() as SIZE_T;

        /* allocate the buffer for the table to be passed to Win32K */
        pAccTableData = LocalAlloc(LMEM_FIXED, i * core::mem::size_of::<ACCEL>() as SIZE_T) as *mut ACCEL;

        /* failure */
        if pAccTableData.is_null() {
            break 'l_Leave;
        }

        pAccTableResData = hAccTableData as *mut PE_ACCEL;

        /* copy the table */
        j = 0;
        while j < i {
            (*pAccTableData.add(j as usize)).fVirt = (*pAccTableResData.add(j as usize)).fVirt;
            (*pAccTableData.add(j as usize)).key = (*pAccTableResData.add(j as usize)).key;
            (*pAccTableData.add(j as usize)).cmd = (*pAccTableResData.add(j as usize)).cmd;
            j += 1;
        }
        (*pAccTableData.add((i - 1) as usize)).fVirt |= 0x80;

        /* create a new accelerator table object */
        hAccTable = NtUserCreateAcceleratorTable(pAccTableData, i);

        /* free the buffer */
        LocalFree(pAccTableData as HGLOBAL);

        /* failure */
        if hAccTable.is_null() {
            break 'l_Leave;
        }

        /* success - cache the object */
        U32AccelCacheAdd(hAccTable, pAccTableResData as HGLOBAL);
    }

    LeaveCriticalSection((&raw mut U32AccelCacheLock).cast());
    hAccTable
}
#[no_mangle]
pub unsafe extern "system" fn U32IsValidAccelMessage(uMsg: UINT) -> BOOL {
    match uMsg {
        WM_KEYDOWN | WM_KEYUP | WM_CHAR | WM_SYSCHAR | WM_SYSKEYDOWN | WM_SYSKEYUP => 1,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn DestroyAcceleratorTable(hAccel: HACCEL) -> BOOL {
    let mut nUsage: ULONG_PTR = 0;

    if hAccel.is_null() {
        return FALSE as BOOL;
    }

    EnterCriticalSection((core::ptr::addr_of_mut!(U32AccelCacheLock)).cast());

    let ppEntry = U32AccelCacheFind(hAccel as HANDLE, core::ptr::null_mut());

    if !(*ppEntry).is_null() {
        let pEntry = *ppEntry;

        (*pEntry).Usage = (*pEntry).Usage - 1;
        nUsage = (*pEntry).Usage;

        if nUsage == 0 {
            *ppEntry = (*pEntry).Next;
            LocalFree(pEntry as HGLOBAL);
        }
    }

    LeaveCriticalSection((core::ptr::addr_of_mut!(U32AccelCacheLock)).cast());

    if nUsage > 0 {
        return FALSE as BOOL;
    }

    NtUserDestroyAcceleratorTable(hAccel) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn LoadAcceleratorsW(hInstance: HINSTANCE, lpTableName: LPCWSTR) -> HACCEL {
    U32LoadAccelerators(
        hInstance,
        FindResourceExW(hInstance, RT_ACCELERATOR as LPCWSTR, lpTableName, 0),
    )
}

#[no_mangle]
pub unsafe extern "system" fn LoadAcceleratorsA(hInstance: HINSTANCE, lpTableName: LPCSTR) -> HACCEL {
    let Accel: HRSRC = FindResourceExA(hInstance, RT_ACCELERATOR as LPCSTR, lpTableName, 0);
    if Accel.is_null() {
        return core::ptr::null_mut();
    }

    U32LoadAccelerators(hInstance, Accel)
}

const RT_ACCELERATOR: usize = 9;

#[no_mangle]
pub unsafe extern "system" fn TranslateAcceleratorW(hWnd: HWND, hAccTable: HACCEL, lpMsg: LPMSG) -> core::ffi::c_int {
    if U32IsValidAccelMessage((*lpMsg).message) == 0 {
        return 0;
    }

    NtUserTranslateAccelerator(hWnd, hAccTable, lpMsg)
}

/* Spec-forwarded to NtUserCopyAcceleratorTable (user32.spec). (ntwrapper.h:147) */
#[no_mangle]
pub unsafe extern "system" fn CopyAcceleratorTableW(
    hAccelSrc: HACCEL,
    lpAccelDst: LPACCEL,
    cAccelEntries: core::ffi::c_int,
) -> core::ffi::c_int {
    NtUserCopyAcceleratorTable(hAccelSrc, lpAccelDst, cAccelEntries as ULONG) as core::ffi::c_int
}

/* Spec-forwarded to NtUserCreateAcceleratorTable (user32.spec). (ntwrapper.h:153) */
#[no_mangle]
pub unsafe extern "system" fn CreateAcceleratorTableW(
    lpaccl: LPACCEL,
    cEntries: core::ffi::c_int,
) -> HACCEL {
    NtUserCreateAcceleratorTable(lpaccl, cEntries as SIZE_T)
}

#[no_mangle]
pub unsafe extern "system" fn CopyAcceleratorTableA(
    hAccelSrc: HACCEL,
    lpAccelDst: LPACCEL,
    cAccelEntries: i32,
) -> i32 {
    let mut i: i32;

    let cAccelEntries = CopyAcceleratorTableW(hAccelSrc, lpAccelDst, cAccelEntries);

    if lpAccelDst.is_null() {
        return cAccelEntries;
    }

    i = 0;
    while i < cAccelEntries {
        if ((*lpAccelDst.offset(i as isize)).fVirt & FVIRTKEY as u8) == 0 {
            let nErrCode = RtlUnicodeToMultiByteN(
                core::ptr::addr_of_mut!((*lpAccelDst.offset(i as isize)).key) as PCHAR,
                core::mem::size_of_val(&(*lpAccelDst.offset(i as isize)).key) as ULONG,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*lpAccelDst.offset(i as isize)).key) as PWCHAR,
                core::mem::size_of_val(&(*lpAccelDst.offset(i as isize)).key) as ULONG,
            );

            if !NT_SUCCESS(nErrCode) {
                (*lpAccelDst.offset(i as isize)).key = 0;
            }
        }
        i += 1;
    }

    cAccelEntries
}

#[no_mangle]
pub unsafe extern "system" fn CreateAcceleratorTableA(lpaccl: LPACCEL, cEntries: core::ffi::c_int) -> HACCEL {
    let mut i: core::ffi::c_int;

    if cEntries == 0 || lpaccl.is_null() {
        return 0 as HACCEL;
    }

    i = 0;
    while i < cEntries {
        let entry = lpaccl.offset(i as isize);
        if (*entry).fVirt == 0 {
            let nErrCode = RtlMultiByteToUnicodeN(
                core::ptr::addr_of_mut!((*entry).key) as *mut u16,
                core::mem::size_of_val(&(*entry).key) as u32,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*entry).key) as *mut i8,
                core::mem::size_of_val(&(*entry).key) as u32,
            );

            if !NT_SUCCESS(nErrCode) {
                (*entry).key = -1i16 as u16;
            }
        }
        i += 1;
    }

    CreateAcceleratorTableW(lpaccl, cEntries)
}

#[no_mangle]
pub unsafe extern "system" fn TranslateAcceleratorA(hWnd: HWND, hAccTable: HACCEL, lpMsg: LPMSG) -> core::ffi::c_int {
    match (*lpMsg).message {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            TranslateAcceleratorW(hWnd, hAccTable, lpMsg)
        }
        WM_CHAR | WM_SYSCHAR => {
            let mut msgW: MSG = *lpMsg;
            let ch: core::ffi::c_char = LOWORD((*lpMsg).wParam as DWORD) as core::ffi::c_char;
            let mut wch: u16 = 0;
            MultiByteToWideChar(CP_ACP, 0, &ch, 1, &mut wch, 1);
            msgW.wParam = MAKEWPARAM(wch, HIWORD((*lpMsg).wParam as DWORD));
            TranslateAcceleratorW(hWnd, hAccTable, &mut msgW)
        }
        _ => 0,
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 1998, 1999, 2000, 2001 ReactOS Team
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
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
//  */
// /*
//  * PROJECT:         ReactOS user32.dll
//  * FILE:            win32ss/user/user32/windows/accel.c
//  * PURPOSE:         Accelerator tables
//  * PROGRAMMER:      KJK::Hyperion <noog@libero.it>
//  * UPDATE HISTORY:
//  *      09/05/2001  CSH  Created
//  *      08/07/2003  KJK  Fully implemented
//  */
//
// #include <user32.h>
//
// /* this is the 8 byte accel struct used in Win32 resources (internal only) */
// typedef struct
// {
//     BYTE   fVirt;
//     BYTE   pad0;
//     WORD   key;
//     WORD   cmd;
//     WORD   pad1;
// } PE_ACCEL, *LPPE_ACCEL;
//
// /* Cache entry */
// typedef struct _USER_ACCEL_CACHE_ENTRY
// {
//  struct _USER_ACCEL_CACHE_ENTRY * Next;
//  ULONG_PTR Usage; /* how many times the table has been loaded */
//  HACCEL Object;   /* handle to the NtUser accelerator table object */
//  HGLOBAL Data;    /* base address of the resource data */
// }
// U32_ACCEL_CACHE_ENTRY;
//
// /* FUNCTIONS *****************************************************************/
//
// /* Lock guarding the cache */
// CRITICAL_SECTION U32AccelCacheLock;
//
// /* Cache */
// U32_ACCEL_CACHE_ENTRY * U32AccelCache = NULL;
//
// /* Look up a handle or resource address in the cache */
// U32_ACCEL_CACHE_ENTRY ** WINAPI U32AccelCacheFind(HANDLE Object, HGLOBAL Data)
// {
//  /*
//   to avoid using a double-link list and still allow elements to be removed,
//   return a pointer to the list link that points to the desired entry
//  */
//  U32_ACCEL_CACHE_ENTRY ** ppEntry = &U32AccelCache;
//
//  for(; *ppEntry; ppEntry = &((*ppEntry)->Next))
//   if((*ppEntry)->Object == Object || (*ppEntry)->Data == Data) break;
//
//  return ppEntry;
// }
//
// /* Allocate an entry and insert it into the cache */
// void WINAPI U32AccelCacheAdd(HACCEL Object, HGLOBAL Data)
// {
//  U32_ACCEL_CACHE_ENTRY * pEntry =
//   LocalAlloc(LMEM_FIXED, sizeof(U32_ACCEL_CACHE_ENTRY));
//
//  /* failed to allocate an entry - not critical */
//  if(pEntry == NULL) return;
//
//  /* initialize the entry */
//  pEntry->Usage = 1;
//  pEntry->Object = Object;
//  pEntry->Data = Data;
//
//  /* insert the entry into the cache */
//  pEntry->Next = U32AccelCache;
//  U32AccelCache = pEntry;
// }
//
// /* Create an accelerator table from a loaded resource */
// HACCEL WINAPI U32LoadAccelerators(HINSTANCE hInstance, HRSRC hTableRes)
// {
//  HGLOBAL hAccTableData;
//  HACCEL hAccTable = NULL;
//  U32_ACCEL_CACHE_ENTRY * pEntry;
//  PE_ACCEL * pAccTableResData;
//  SIZE_T i = 0;
//  SIZE_T j = 0;
//  ACCEL * pAccTableData;
//
//  /* load the accelerator table */
//  hAccTableData = LoadResource(hInstance, hTableRes);
//
//  /* failure */
//  if(hAccTableData == NULL) return NULL;
//
//  EnterCriticalSection(&U32AccelCacheLock);
//
//  /* see if this accelerator table has already been loaded */
//  pEntry = *U32AccelCacheFind(NULL, hAccTableData);
//
//  /* accelerator table already loaded */
//  if(pEntry)
//  {
//   /* increment the reference count */
//   ++ pEntry->Usage;
//
//   /* return the existing object */
//   hAccTable = pEntry->Object;
//
//   /* success */
//   goto l_Leave;
//  }
//
//  /* determine the number of entries in the table */
//  i = SizeofResource(hInstance, hTableRes) / sizeof(PE_ACCEL);
//
//  /* allocate the buffer for the table to be passed to Win32K */
//  pAccTableData = LocalAlloc(LMEM_FIXED, i * sizeof(ACCEL));
//
//  /* failure */
//  if(pAccTableData == NULL) goto l_Leave;
//
//  pAccTableResData = (PE_ACCEL *)hAccTableData;
//
//  /* copy the table */
//  for(j = 0; j < i; ++ j)
//  {
//   pAccTableData[j].fVirt = pAccTableResData[j].fVirt;
//   pAccTableData[j].key = pAccTableResData[j].key;
//   pAccTableData[j].cmd = pAccTableResData[j].cmd;
//  }
//  pAccTableData[i - 1].fVirt |= 0x80;
//
//  /* create a new accelerator table object */
//  hAccTable = NtUserCreateAcceleratorTable(pAccTableData, i);
//
//  /* free the buffer */
//  LocalFree(pAccTableData);
//
//  /* failure */
//  if(hAccTable == NULL) goto l_Leave;
//
//  /* success - cache the object */
//  U32AccelCacheAdd(hAccTable, pAccTableResData);
//
// l_Leave:
//  LeaveCriticalSection(&U32AccelCacheLock);
//  return hAccTable;
// }
//
// /* Checks if a message can be translated through an accelerator table */
// BOOL WINAPI U32IsValidAccelMessage(UINT uMsg)
// {
//  switch(uMsg)
//  {
//   case WM_KEYDOWN:
//   case WM_KEYUP:
//   case WM_CHAR:
//   case WM_SYSCHAR:
//   case WM_SYSKEYDOWN:
//   case WM_SYSKEYUP:
//    return TRUE;
//
//   default:
//    return FALSE;
//  }
// }
//
// /* WIN32 FUNCTIONS ***********************************************************/
//
// /*
//  * Dereference the specified accelerator table, removing it from the cache and
//  * deleting the associated NtUser object as appropriate
//  *
//  * @implemented
//  */
// BOOL WINAPI DestroyAcceleratorTable(HACCEL hAccel)
// {
//  U32_ACCEL_CACHE_ENTRY ** ppEntry;
//  ULONG_PTR nUsage = 0;
//
//  if (!hAccel)
//   return FALSE;
//
//  EnterCriticalSection(&U32AccelCacheLock);
//
//  /* see if this accelerator table has been cached */
//  ppEntry = U32AccelCacheFind(hAccel, NULL);
//
//  /* accelerator table cached */
//  if(*ppEntry)
//  {
//   U32_ACCEL_CACHE_ENTRY * pEntry = *ppEntry;
//
//   /* decrement the reference count */
//   nUsage = pEntry->Usage = pEntry->Usage - 1;
//
//   /* reference count now zero: destroy the cache entry */
//   if(nUsage == 0)
//   {
//    /* unlink the cache entry */
//    *ppEntry = pEntry->Next;
//
//    /* free the cache entry */
//    LocalFree(pEntry);
//   }
//  }
//
//  LeaveCriticalSection(&U32AccelCacheLock);
//
//  if(nUsage > 0) return FALSE;
//
//  /* destroy the object */
//  return NtUserDestroyAcceleratorTable(hAccel);
// }
//
//
// /*
//  * Create an accelerator table from a named resource
//  *
//  * @implemented
//  */
// HACCEL WINAPI LoadAcceleratorsW(HINSTANCE hInstance, LPCWSTR lpTableName)
// {
//  return U32LoadAccelerators
//  (
//   hInstance,
//   FindResourceExW(hInstance, (LPCWSTR) RT_ACCELERATOR, lpTableName, 0)
//  );
// }
//
//
// /*
//  * @implemented
//  */
// HACCEL WINAPI LoadAcceleratorsA(HINSTANCE hInstance, LPCSTR lpTableName)
// {
//   HRSRC Accel;
//
//   Accel = FindResourceExA(hInstance, (LPCSTR) RT_ACCELERATOR, lpTableName, 0);
//   if (NULL == Accel)
//     {
//       return NULL;
//     }
//
//   return U32LoadAccelerators(hInstance, Accel);
// }
//
// /*
//  * Translate a key press into a WM_COMMAND message
//  *
//  * @implemented
//  */
// int WINAPI TranslateAcceleratorW(HWND hWnd, HACCEL hAccTable, LPMSG lpMsg)
// {
//  if(!U32IsValidAccelMessage(lpMsg->message)) return 0;
//
//  return NtUserTranslateAccelerator(hWnd, hAccTable, lpMsg);
// }
//
//
// /*
//  * @implemented
//  */
// int WINAPI CopyAcceleratorTableA
// (
//  HACCEL hAccelSrc,
//  LPACCEL lpAccelDst, /* can be NULL */
//  int cAccelEntries
// )
// {
//    int i;
//
//    cAccelEntries = CopyAcceleratorTableW(hAccelSrc, lpAccelDst, cAccelEntries);
//
//    if (lpAccelDst == NULL) return cAccelEntries;
//
//    for(i = 0; i < cAccelEntries; ++ i)
//    if(!(lpAccelDst[i].fVirt & FVIRTKEY))
//    {
//       NTSTATUS nErrCode = RtlUnicodeToMultiByteN(
//          (PCHAR)&lpAccelDst[i].key,
//          sizeof(lpAccelDst[i].key),
//          NULL,
//          (PWCHAR)&lpAccelDst[i].key,
//          sizeof(lpAccelDst[i].key)
//          );
//
//       if(!NT_SUCCESS(nErrCode)) lpAccelDst[i].key = 0;
//    }
//
//    return cAccelEntries;
// }
//
//
// /*
//  * @implemented
//  */
// HACCEL WINAPI CreateAcceleratorTableA(LPACCEL lpaccl, int cEntries)
// {
//  int i;
//
//  if (!cEntries || !lpaccl) return (HACCEL)0;
//
//  for(i = 0; i < cEntries; ++ i)
//   if(!lpaccl[i].fVirt)
//   {
//    NTSTATUS nErrCode = RtlMultiByteToUnicodeN
//    (
//     (PWCHAR)&lpaccl[i].key,
//     sizeof(lpaccl[i].key),
//     NULL,
//     (PCHAR)&lpaccl[i].key,
//     sizeof(lpaccl[i].key)
//    );
//
//    if(!NT_SUCCESS(nErrCode)) lpaccl[i].key = -1;
//   }
//
//  return CreateAcceleratorTableW(lpaccl, cEntries);
// }
//
//
// /*
//  * @implemented
//  */
// int WINAPI TranslateAcceleratorA(HWND hWnd, HACCEL hAccTable, LPMSG lpMsg)
// {
//     switch (lpMsg->message)
//     {
//     case WM_KEYDOWN:
//     case WM_SYSKEYDOWN:
//         return TranslateAcceleratorW( hWnd, hAccTable, lpMsg );
//
//     case WM_CHAR:
//     case WM_SYSCHAR:
//         {
//             MSG msgW = *lpMsg;
//             char ch = LOWORD(lpMsg->wParam);
//             WCHAR wch;
//             MultiByteToWideChar(CP_ACP, 0, &ch, 1, &wch, 1);
//             msgW.wParam = MAKEWPARAM(wch, HIWORD(lpMsg->wParam));
//             return TranslateAcceleratorW( hWnd, hAccTable, &msgW );
//         }
//
//     default:
//         return 0;
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
