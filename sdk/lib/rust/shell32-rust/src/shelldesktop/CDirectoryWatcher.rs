// rustconv-assembly: class:CDirectoryWatcher
use crate::prelude::*;
// One declaration per external symbol: island.rs.
use crate::island::CloseHandle;
use crate::island::CreateFileW;
use crate::shelldesktop::CDirectoryList::CDirectoryList_AddPath_body;
use crate::shelldesktop::CDirectoryList::CDirectoryList_ContainsPath_body;
use crate::shelldesktop::CDirectoryList::CDirectoryList_DeletePath_body;
use crate::shelldesktop::CDirectoryList::CDirectoryList_RenamePath_body;
use crate::shelldesktop::CDirectoryList::CDirectoryList_ctor_2_body;
use crate::shelldesktop::CDirectoryWatcher_free::DirectoryWatcherThreadFuncAPC;
use crate::shelldesktop::CDirectoryWatcher_free::_AddDirectoryProcAPC;
use crate::shelldesktop::CDirectoryWatcher_free::_NotificationCompletion;
use crate::shelldesktop::CDirectoryWatcher_free::_RequestAllTerminationAPC;
use crate::shelldesktop::CDirectoryWatcher_free::_RequestTerminationAPC;
use crate::shelldesktop::CDirectoryWatcher_free::s_buffer;
use crate::shelldesktop::CDirectoryWatcher_free::s_fTerminateAllWatchers;
use crate::shelldesktop::CDirectoryWatcher_free::s_hThreadAPC;
#[repr(C)]
pub struct CDirectoryList {
    pub m_fRecursive: BOOL,
    pub m_items: CSimpleArray<CDirectoryItem>,
}

#[repr(C)]
pub struct CDirectoryItem {
    pub m_pszPath: LPWSTR,
}

// Deterministic COM scaffold for CDirectoryWatcher; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CDirectoryWatcher {
    pub m_hDirectory: HANDLE,
    pub m_szDirectoryPath: [WCHAR; 260],
    pub m_hNotifyWnd: HWND,
    pub m_fDead: BOOL,
    pub m_fRecursive: BOOL,
    pub m_dir_list: CDirectoryList,
    pub m_overlapped: OVERLAPPED,
}

/// `new CDirectoryWatcher` — operator new, the vptr store, then the constructor body.
pub unsafe fn CDirectoryWatcher_new(hNotifyWnd: HWND, pszDirectoryPath: LPCWSTR, fSubTree: BOOL) -> *mut CDirectoryWatcher {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CDirectoryWatcher>(),
    ) as *mut CDirectoryWatcher;
    if this.is_null() { return core::ptr::null_mut(); }
    CDirectoryWatcher_ctor_body(this, hNotifyWnd, pszDirectoryPath, fSubTree);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `CDirectoryWatcher_new`; freeing through any other allocator is a heap bug.
pub unsafe fn CDirectoryWatcher_delete(this: *mut CDirectoryWatcher) {
    if this.is_null() { return; }
    CDirectoryWatcher_dtor_body(this);
    HeapFree(GetProcessHeap(), 0, this);
}

// Deterministic C++ support closure selected from source identifiers.

pub const BUFFER_SIZE: UINT = 0x1000;

pub const ERROR_INVALID_FUNCTION: DWORD = 1;

pub const ERROR_OPERATION_ABORTED: DWORD = 995;

pub const FILE_ACTION_ADDED: DWORD = 0x00000001;

pub const FILE_ACTION_MODIFIED: DWORD = 0x00000003;

pub const FILE_ACTION_REMOVED: DWORD = 0x00000002;

pub const FILE_ACTION_RENAMED_NEW_NAME: DWORD = 0x00000005;

pub const FILE_ACTION_RENAMED_OLD_NAME: DWORD = 0x00000004;

pub const FILE_FLAG_BACKUP_SEMANTICS: DWORD = 0x02000000;

pub const FILE_FLAG_OVERLAPPED: DWORD = 0x40000000;

pub const FILE_LIST_DIRECTORY: DWORD = 0x00000001;

pub const FILE_NOTIFY_CHANGE_ATTRIBUTES: DWORD = 0x00000004;

pub const FILE_NOTIFY_CHANGE_CREATION: DWORD = 0x00000040;

pub const FILE_NOTIFY_CHANGE_DIR_NAME: DWORD = 0x00000002;

pub const FILE_NOTIFY_CHANGE_FILE_NAME: DWORD = 0x00000001;

pub const FILE_NOTIFY_CHANGE_SIZE: DWORD = 0x00000008;

pub const FILE_SHARE_DELETE: DWORD = 0x00000004;

pub const FILE_SHARE_READ: DWORD = 0x00000001;

pub const FILE_SHARE_WRITE: DWORD = 0x00000002;

pub const INFINITE: DWORD = 0xFFFFFFFF;

pub const OPEN_EXISTING: DWORD = 3;

pub const SHCNE_ALLEVENTS: DWORD = 0x7FFFFFFF;

pub const SHCNE_CREATE: DWORD = 0x00000002;

pub const SHCNE_DELETE: DWORD = 0x00000004;

pub const SHCNE_INTERRUPT: DWORD = 0x80000000;

pub const SHCNE_MKDIR: DWORD = 0x00000008;

pub const SHCNE_RENAMEFOLDER: DWORD = 0x00020000;

pub const SHCNE_RENAMEITEM: DWORD = 0x00000001;

pub const SHCNE_RMDIR: DWORD = 0x00000010;

pub const SHCNE_UPDATEDIR: DWORD = 0x00001000;

pub const SHCNE_UPDATEITEM: DWORD = 0x00002000;

pub const SHCNF_FLUSH: DWORD = 0x1000;

pub const SHCNF_PATHW: DWORD = 0x0005;

extern "system" {
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GetLastError() -> DWORD;
    fn StrChrW(text: LPCWSTR, ch: WCHAR) -> LPWSTR;
}

extern "C" {
    fn _beginthreadex(security: *mut core::ffi::c_void, stack_size: UINT, start: Option<unsafe extern "system" fn(*mut core::ffi::c_void) -> UINT>, arglist: *mut core::ffi::c_void, initflag: UINT, thrdaddr: *mut UINT) -> usize;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

#[inline]
unsafe fn CopyMemory<D, S, N: CppComSize>(
    dst: *mut D, src: *const S, bytes: N) {
    let _ = memcpy(dst, src, bytes);
}
#[inline]
unsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {
    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());
}

use core::ffi::c_void;

pub unsafe fn NotifyFileSystemChange_body(wEventId: LONG, path1: LPCWSTR, path2: LPCWSTR) -> () {
    // C's usual arithmetic conversions promote `wEventId` (LONG) to unsigned for the
    // `|`, then convert the result back for the LONG parameter. Rust does neither
    // implicitly, so both steps are written out; the value is unchanged.
    SHChangeNotify(
        ((wEventId as DWORD) | SHCNE_INTERRUPT) as LONG,
        SHCNF_PATHW | SHCNF_FLUSH,
        path1.cast(),
        path2.cast(),
    );
}

pub unsafe fn CDirectoryWatcher_ctor_body(this: *mut CDirectoryWatcher, hNotifyWnd: HWND, pszDirectoryPath: LPCWSTR, fSubTree: BOOL) -> () {
    (*this).m_hNotifyWnd = hNotifyWnd;
    (*this).m_fDead = FALSE;
    (*this).m_fRecursive = fSubTree;
    // C++ initializer list: `m_dir_list(pszDirectoryPath, fSubTree)` — the TWO-argument
    // constructor, which is `_ctor_2_body`. `_ctor_body` is the default one and takes
    // only `this`. `.cast()` bridges this module's copy of the CDirectoryList layout to
    // the sibling module that owns the bodies (same repr, two nominal spellings).
    CDirectoryList_ctor_2_body(
        core::ptr::addr_of_mut!((*this).m_dir_list).cast(),
        pszDirectoryPath,
        fSubTree,
    );

    GetFullPathNameW(
        pszDirectoryPath,
        (*this).m_szDirectoryPath.len() as DWORD,
        (*this).m_szDirectoryPath.as_mut_ptr(),
        core::ptr::null_mut(),
    );

    (*this).m_hDirectory = CreateFileW(
        (*this).m_szDirectoryPath.as_ptr(),
        FILE_LIST_DIRECTORY,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
        core::ptr::null_mut(),
    );
}

pub unsafe fn CDirectoryWatcher_Create_body(this: *mut CDirectoryWatcher, hNotifyWnd: HWND, pszDirectoryPath: LPCWSTR, fSubTree: BOOL) -> *mut CDirectoryWatcher {
    // `this` is unused: C++ `Create` is a STATIC, but the worker signature carries a
    // receiver anyway (same shape as CMenuFocusManager's Acquire/ReleaseManager).
    let _ = this;
    // C++: `new CDirectoryWatcher(hNotifyWnd, pszDirectoryPath, fSubTree)` — allocate, then
    // run the 3-argument constructor. That is exactly `CDirectoryWatcher_new`.
    let mut pDirectoryWatcher = CDirectoryWatcher_new(hNotifyWnd, pszDirectoryPath, fSubTree);
    // The C++ dereferences `new`'s result unchecked. `operator new` IS malloc in this tree
    // (sdk/lib/cpprt), so that is a latent null-deref; the scaffold's `_new` reports failure
    // as NULL, and returning it here is what this function's own contract already means
    // (Create returns NULL on failure, and every caller tests for it).
    if pDirectoryWatcher.is_null() {
        return core::ptr::null_mut();
    }
    if (*pDirectoryWatcher).m_hDirectory == INVALID_HANDLE_VALUE {
        // C++: `delete pDirectoryWatcher` — destructor THEN free, not a bare free. The
        // destructor closes m_hDirectory; skipping it leaks a live directory handle.
        CDirectoryWatcher_delete(pDirectoryWatcher);
        pDirectoryWatcher = core::ptr::null_mut();
    }
    pDirectoryWatcher
}

pub unsafe fn CDirectoryWatcher_dtor_body(this: *mut CDirectoryWatcher) -> () {
    if (*this).m_hDirectory != INVALID_HANDLE_VALUE {
        CloseHandle((*this).m_hDirectory);
    }
}

pub unsafe fn ConvertActionToEvent_body(Action: DWORD, fDir: BOOL) -> DWORD {
    match Action {
        FILE_ACTION_ADDED => {
            return if fDir != 0 {
                SHCNE_MKDIR
            } else {
                SHCNE_CREATE
            };
        }
        FILE_ACTION_REMOVED => {
            return if fDir != 0 {
                SHCNE_RMDIR
            } else {
                SHCNE_DELETE
            };
        }
        FILE_ACTION_MODIFIED => {
            return if fDir != 0 {
                SHCNE_UPDATEDIR
            } else {
                SHCNE_UPDATEITEM
            };
        }
        FILE_ACTION_RENAMED_OLD_NAME => {}
        FILE_ACTION_RENAMED_NEW_NAME => {
            return if fDir != 0 {
                SHCNE_RENAMEFOLDER
            } else {
                SHCNE_RENAMEITEM
            };
        }
        _ => {}
    }
    0
}

pub unsafe fn CDirectoryWatcher_ProcessNotification_body(this: *mut CDirectoryWatcher) -> () {
    let mut pInfo: *mut FILE_NOTIFY_INFORMATION = s_buffer.as_mut_ptr() as *mut FILE_NOTIFY_INFORMATION;
    let mut szName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szTempPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwEvent: DWORD;
    let mut cbName: DWORD;
    let mut fDir: BOOL;

    szPath[0 as usize] = 0 as u16;
    szTempPath[0 as usize] = 0 as u16;

    loop {
        cbName = (*pInfo).FileNameLength;
        if (core::mem::size_of_val(&szName) - core::mem::size_of::<u16>()) < cbName as usize {
            break;
        }

        ZeroMemory(szName.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of_val(&szName));
        CopyMemory(
            szName.as_mut_ptr() as *mut core::ffi::c_void,
            (*pInfo).FileName.as_ptr() as *const core::ffi::c_void,
            cbName as usize,
        );

        lstrcpynW(
            szPath.as_mut_ptr() as *mut u16,
            (*this).m_szDirectoryPath.as_ptr() as *const u16,
            szPath.len() as i32,
        );
        PathAppendW(
            szPath.as_mut_ptr() as *mut u16,
            szName.as_ptr() as *const u16,
        );

        if !StrChrW(szPath.as_ptr() as *const u16, 0x007Eu16).is_null() {
            if GetLongPathNameW(
                szPath.as_ptr() as *const u16,
                szName.as_mut_ptr() as *mut u16,
                szName.len() as u32,
            ) != 0
                && PathIsRelativeW(szName.as_ptr() as *const u16) == 0
            {
                lstrcpynW(
                    szPath.as_mut_ptr() as *mut u16,
                    szName.as_ptr() as *const u16,
                    szPath.len() as i32,
                );
            }
        }

        fDir = PathIsDirectoryW(szPath.as_ptr() as *const u16);
        dwEvent = ConvertActionToEvent_body((*pInfo).Action, fDir);

        if fDir == 0
            && dwEvent == SHCNE_DELETE
            && CDirectoryList_ContainsPath_body(core::ptr::addr_of_mut!((*this).m_dir_list).cast(), szPath.as_ptr() as *const u16) != 0
        {
            fDir = TRUE;
            dwEvent = SHCNE_RMDIR;
        }

        match dwEvent {
            SHCNE_MKDIR => {
                if PathIsDirectoryW(szPath.as_ptr() as *const u16) == 0
                    || CDirectoryList_AddPath_body(core::ptr::addr_of_mut!((*this).m_dir_list).cast(), szPath.as_ptr() as *const u16) == 0
                {
                    dwEvent = 0;
                }
            }
            SHCNE_CREATE => {
                if PathFileExistsW(szPath.as_ptr() as *const u16) == 0
                    || PathIsDirectoryW(szPath.as_ptr() as *const u16) != 0
                {
                    dwEvent = 0;
                }
            }
            SHCNE_RENAMEFOLDER => {
                if PathIsDirectoryW(szPath.as_ptr() as *const u16) == 0
                    || CDirectoryList_RenamePath_body(core::ptr::addr_of_mut!((*this).m_dir_list).cast(),
                        szTempPath.as_ptr() as *const u16,
                        szPath.as_ptr() as *const u16,
                    ) == 0
                {
                    dwEvent = 0;
                }
            }
            SHCNE_RENAMEITEM => {
                if PathFileExistsW(szPath.as_ptr() as *const u16) == 0
                    || PathIsDirectoryW(szPath.as_ptr() as *const u16) != 0
                {
                    dwEvent = 0;
                }
            }
            SHCNE_RMDIR => {
                if PathIsDirectoryW(szPath.as_ptr() as *const u16) != 0
                    || CDirectoryList_DeletePath_body(core::ptr::addr_of_mut!((*this).m_dir_list).cast(), szPath.as_ptr() as *const u16) == 0
                {
                    dwEvent = 0;
                }
            }
            SHCNE_DELETE => {
                if PathFileExistsW(szPath.as_ptr() as *const u16) != 0 {
                    dwEvent = 0;
                }
            }
            _ => {}
        }

        if dwEvent != 0 {
            if (*pInfo).Action == FILE_ACTION_RENAMED_NEW_NAME {
                NotifyFileSystemChange_body(
                    dwEvent as LONG,
                    szTempPath.as_ptr() as *const u16,
                    szPath.as_ptr() as *const u16,
                );
            } else {
                NotifyFileSystemChange_body(
                    dwEvent as LONG,
                    szPath.as_ptr() as *const u16,
                    core::ptr::null(),
                );
            }
        } else if (*pInfo).Action == FILE_ACTION_RENAMED_OLD_NAME {
            lstrcpynW(
                szTempPath.as_mut_ptr() as *mut u16,
                szPath.as_ptr() as *const u16,
                MAX_PATH as i32,
            );
        }

        if (*pInfo).NextEntryOffset == 0 {
            break;
        }

        pInfo = (pInfo as *mut u8).add((*pInfo).NextEntryOffset as usize) as *mut FILE_NOTIFY_INFORMATION;
    }
}

pub unsafe fn CDirectoryWatcher_ReadCompletion_body(this: *mut CDirectoryWatcher, dwErrorCode: DWORD, dwNumberOfBytesTransfered: DWORD) -> () {
    if dwErrorCode == ERROR_INVALID_FUNCTION {
        return;
    }

    if dwErrorCode == ERROR_OPERATION_ABORTED {
        if CDirectoryWatcher_IsDead_body(this) != 0 {
            CDirectoryWatcher_delete(this);
        }
        return;
    }

    if CDirectoryWatcher_IsDead_body(this) != 0 {
        CDirectoryWatcher_delete(this);
        return;
    }

    if dwNumberOfBytesTransfered == 0 {
        NotifyFileSystemChange_body(
            SHCNE_UPDATEDIR as LONG,
            (*this).m_szDirectoryPath.as_ptr(),
            core::ptr::null(),
        );
    } else {
        CDirectoryWatcher_ProcessNotification_body(this);
    }

    CDirectoryWatcher_RestartWatching_body(this);
}

pub unsafe fn GetFilterFromEvents_body(fEvents: DWORD) -> DWORD {
    let _ = fEvents;
    (FILE_NOTIFY_CHANGE_FILE_NAME |
     FILE_NOTIFY_CHANGE_DIR_NAME |
     FILE_NOTIFY_CHANGE_CREATION |
     FILE_NOTIFY_CHANGE_SIZE |
     FILE_NOTIFY_CHANGE_ATTRIBUTES)
}

pub unsafe fn CDirectoryWatcher_RestartWatching_body(this: *mut CDirectoryWatcher) -> BOOL {
    assert!(this != core::ptr::null_mut());
    if CDirectoryWatcher_IsDead_body(this) != 0 {
        CDirectoryWatcher_dtor_body(this);
        return FALSE;
    }

    ZeroMemory(s_buffer.as_mut_ptr().cast::<core::ffi::c_void>(), core::mem::size_of_val(&s_buffer));
    ZeroMemory(core::ptr::addr_of_mut!((*this).m_overlapped).cast::<core::ffi::c_void>(), core::mem::size_of_val(&(*this).m_overlapped));
    (*this).m_overlapped.hEvent = this as HANDLE;

    let dwFilter: DWORD = GetFilterFromEvents_body(SHCNE_ALLEVENTS);
    if ReadDirectoryChangesW(
        (*this).m_hDirectory,
        s_buffer.as_mut_ptr().cast(),
        core::mem::size_of_val(&s_buffer) as DWORD,
        (*this).m_fRecursive,
        dwFilter,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*this).m_overlapped),
        // Nullable callback: the island types it as `Option<fn…>`.
        Some(_NotificationCompletion),
    ) == 0 {
        return FALSE;
    }

    return TRUE;
}

pub unsafe fn CDirectoryWatcher_CreateAPCThread_body(this: *mut CDirectoryWatcher) -> BOOL {
    if s_hThreadAPC != core::ptr::null_mut() {
        return TRUE;
    }

    let mut tid: core::ffi::c_uint = 0;

    s_fTerminateAllWatchers = FALSE;
    // `_beginthreadex` returns `uintptr_t`; the handle slot is a HANDLE.
    s_hThreadAPC = _beginthreadex(
        core::ptr::null_mut(),
        0,
        // Nullable callback: the island types it as `Option<fn…>`.
        Some(DirectoryWatcherThreadFuncAPC),
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(tid),
    ) as *mut core::ffi::c_void;

    return (s_hThreadAPC != core::ptr::null_mut()) as BOOL;
}

pub unsafe fn CDirectoryWatcher_RequestAddWatcher_body(this: *mut CDirectoryWatcher) -> BOOL {
    assert!(!this.is_null());

    if CDirectoryWatcher_CreateAPCThread_body(this) == 0 {
        return FALSE;
    }

    QueueUserAPC(Some(_AddDirectoryProcAPC), s_hThreadAPC, this as ULONG_PTR);
    return TRUE;
}

pub unsafe fn CDirectoryWatcher_RequestTermination_body(this: *mut CDirectoryWatcher) -> BOOL {
    assert!(!this.is_null());
    if s_hThreadAPC != core::ptr::null_mut() {
        QueueUserAPC(Some(_RequestTerminationAPC), s_hThreadAPC, this as ULONG_PTR);
        return TRUE;
    }

    return FALSE;
}

pub unsafe fn CDirectoryWatcher_RequestAllWatchersTermination_body(this: *mut CDirectoryWatcher) -> () {
    if s_hThreadAPC.is_null() {
        return;
    }

    QueueUserAPC(
        Some(_RequestAllTerminationAPC),
        s_hThreadAPC,
        core::ptr::null_mut::<core::ffi::c_void>() as ULONG_PTR,
    );
}

pub unsafe fn CDirectoryWatcher_QuitWatching_body(this: *mut CDirectoryWatcher) -> () {
    assert!(this != core::ptr::null_mut());
    unsafe {
        (*this).m_fDead = TRUE;
        (*this).m_hNotifyWnd = core::ptr::null_mut();
        CancelIo((*this).m_hDirectory);
    }
}

pub unsafe fn CDirectoryWatcher_IsDead_body(this: *mut CDirectoryWatcher) -> BOOL {
    if (*this).m_hNotifyWnd != core::ptr::null_mut() && IsWindow((*this).m_hNotifyWnd) == 0 {
        (*this).m_hNotifyWnd = core::ptr::null_mut();
        (*this).m_fDead = TRUE;
        CancelIo((*this).m_hDirectory);
    }
    (*this).m_fDead
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     Shell change notification
//  * COPYRIGHT:   Copyright 2020 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
// #include "shelldesktop.h"
// #include "CDirectoryWatcher.h"
// #include <process.h>     // for _beginthreadex
// #include <assert.h>      // for assert
//
// WINE_DEFAULT_DEBUG_CHANNEL(shcn);
//
// // Notify filesystem change
// static inline void
// NotifyFileSystemChange(LONG wEventId, LPCWSTR path1, LPCWSTR path2)
// {
//     SHChangeNotify(wEventId | SHCNE_INTERRUPT, SHCNF_PATHW | SHCNF_FLUSH, path1, path2);
// }
// /*static*/ CDirectoryWatcher *
// CDirectoryWatcher::Create(HWND hNotifyWnd, LPCWSTR pszDirectoryPath, BOOL fSubTree)
// {
//     CDirectoryWatcher *pDirectoryWatcher =
//         new CDirectoryWatcher(hNotifyWnd, pszDirectoryPath, fSubTree);
//     if (pDirectoryWatcher->m_hDirectory == INVALID_HANDLE_VALUE)
//     {
//         ERR("CreateFileW failed\n");
//         delete pDirectoryWatcher;
//         pDirectoryWatcher = NULL;
//     }
//     return pDirectoryWatcher;
// }
// static DWORD
// ConvertActionToEvent(DWORD Action, BOOL fDir)
// {
//     switch (Action)
//     {
//         case FILE_ACTION_ADDED:
//             return (fDir ? SHCNE_MKDIR : SHCNE_CREATE);
//         case FILE_ACTION_REMOVED:
//             return (fDir ? SHCNE_RMDIR : SHCNE_DELETE);
//         case FILE_ACTION_MODIFIED:
//             return (fDir ? SHCNE_UPDATEDIR : SHCNE_UPDATEITEM);
//         case FILE_ACTION_RENAMED_OLD_NAME:
//             break;
//         case FILE_ACTION_RENAMED_NEW_NAME:
//             return (fDir ? SHCNE_RENAMEFOLDER : SHCNE_RENAMEITEM);
//         default:
//             break;
//     }
//     return 0;
// }
// void CDirectoryWatcher::ProcessNotification()
// {
//     PFILE_NOTIFY_INFORMATION pInfo = (PFILE_NOTIFY_INFORMATION)s_buffer;
//     WCHAR szName[MAX_PATH], szPath[MAX_PATH], szTempPath[MAX_PATH];
//     DWORD dwEvent, cbName;
//     BOOL fDir;
//     TRACE("CDirectoryWatcher::ProcessNotification: enter\n");
//
//     // for each entry in s_buffer
//     szPath[0] = szTempPath[0] = 0;
//     for (;;)
//     {
//         // get name (relative from m_szDirectoryPath)
//         cbName = pInfo->FileNameLength;
//         if (sizeof(szName) - sizeof(UNICODE_NULL) < cbName)
//         {
//             ERR("pInfo->FileName is longer than szName\n");
//             break;
//         }
//         // NOTE: FILE_NOTIFY_INFORMATION.FileName is not null-terminated.
//         ZeroMemory(szName, sizeof(szName));
//         CopyMemory(szName, pInfo->FileName, cbName);
//
//         // get full path
//         lstrcpynW(szPath, m_szDirectoryPath, _countof(szPath));
//         PathAppendW(szPath, szName);
//
//         // convert to long pathname if it contains '~'
//         if (StrChrW(szPath, L'~') != NULL)
//         {
//             if (GetLongPathNameW(szPath, szName, _countof(szName)) &&
//                 !PathIsRelativeW(szName))
//             {
//                 lstrcpynW(szPath, szName, _countof(szPath));
//             }
//         }
//
//         // convert action to event
//         fDir = PathIsDirectoryW(szPath);
//         dwEvent = ConvertActionToEvent(pInfo->Action, fDir);
//
//         // convert SHCNE_DELETE to SHCNE_RMDIR if the path is a directory
//         if (!fDir && (dwEvent == SHCNE_DELETE) && m_dir_list.ContainsPath(szPath))
//         {
//             fDir = TRUE;
//             dwEvent = SHCNE_RMDIR;
//         }
//
//         // update m_dir_list
//         switch (dwEvent)
//         {
//             case SHCNE_MKDIR:
//                 if (!PathIsDirectoryW(szPath) || !m_dir_list.AddPath(szPath))
//                     dwEvent = 0;
//                 break;
//             case SHCNE_CREATE:
//                 if (!PathFileExistsW(szPath) || PathIsDirectoryW(szPath))
//                     dwEvent = 0;
//                 break;
//             case SHCNE_RENAMEFOLDER:
//                 if (!PathIsDirectoryW(szPath) || !m_dir_list.RenamePath(szTempPath, szPath))
//                     dwEvent = 0;
//                 break;
//             case SHCNE_RENAMEITEM:
//                 if (!PathFileExistsW(szPath) || PathIsDirectoryW(szPath))
//                     dwEvent = 0;
//                 break;
//             case SHCNE_RMDIR:
//                 if (PathIsDirectoryW(szPath) || !m_dir_list.DeletePath(szPath))
//                     dwEvent = 0;
//                 break;
//             case SHCNE_DELETE:
//                 if (PathFileExistsW(szPath))
//                     dwEvent = 0;
//                 break;
//         }
//
//         if (dwEvent != 0)
//         {
//             // notify
//             if (pInfo->Action == FILE_ACTION_RENAMED_NEW_NAME)
//                 NotifyFileSystemChange(dwEvent, szTempPath, szPath);
//             else
//                 NotifyFileSystemChange(dwEvent, szPath, NULL);
//         }
//         else if (pInfo->Action == FILE_ACTION_RENAMED_OLD_NAME)
//         {
//             // save path for next FILE_ACTION_RENAMED_NEW_NAME
//             lstrcpynW(szTempPath, szPath, MAX_PATH);
//         }
//
//         if (pInfo->NextEntryOffset == 0)
//             break; // there is no next entry
//
//         // go next entry
//         pInfo = (PFILE_NOTIFY_INFORMATION)((LPBYTE)pInfo + pInfo->NextEntryOffset);
//     }
//
//     TRACE("CDirectoryWatcher::ProcessNotification: leave\n");
// }
// void CDirectoryWatcher::ReadCompletion(DWORD dwErrorCode, DWORD dwNumberOfBytesTransfered)
// {
//     // If the FSD doesn't support directory change notifications, there's no
//     // no need to retry and requeue notification
//     if (dwErrorCode == ERROR_INVALID_FUNCTION)
//     {
//         ERR("ERROR_INVALID_FUNCTION\n");
//         return;
//     }
//
//     // Also, if the notify operation was canceled (like, user moved to another
//     // directory), then, don't requeue notification.
//     if (dwErrorCode == ERROR_OPERATION_ABORTED)
//     {
//         TRACE("ERROR_OPERATION_ABORTED\n");
//         if (IsDead())
//             delete this;
//         return;
//     }
//
//     // is this watch dead?
//     if (IsDead())
//     {
//         TRACE("IsDead()\n");
//         delete this;
//         return;
//     }
//
//     // This likely means overflow, so force whole directory refresh.
//     if (dwNumberOfBytesTransfered == 0)
//     {
//         // do notify a SHCNE_UPDATEDIR
//         NotifyFileSystemChange(SHCNE_UPDATEDIR, m_szDirectoryPath, NULL);
//     }
//     else
//     {
//         // do notify
//         ProcessNotification();
//     }
//
//     // restart a watch
//     RestartWatching();
// }
// static DWORD
// GetFilterFromEvents(DWORD fEvents)
// {
//     // FIXME
//     return (FILE_NOTIFY_CHANGE_FILE_NAME |
//             FILE_NOTIFY_CHANGE_DIR_NAME |
//             FILE_NOTIFY_CHANGE_CREATION |
//             FILE_NOTIFY_CHANGE_SIZE |
//             FILE_NOTIFY_CHANGE_ATTRIBUTES);
// }
// BOOL CDirectoryWatcher::RestartWatching()
// {
//     assert(this != NULL);
//
//     if (IsDead())
//     {
//         delete this;
//         return FALSE; // the watch is dead
//     }
//
//     // initialize the buffer and the overlapped
//     ZeroMemory(s_buffer, sizeof(s_buffer));
//     ZeroMemory(&m_overlapped, sizeof(m_overlapped));
//     m_overlapped.hEvent = (HANDLE)this;
//
//     // start the directory watch
//     DWORD dwFilter = GetFilterFromEvents(SHCNE_ALLEVENTS);
//     if (!ReadDirectoryChangesW(m_hDirectory, s_buffer, sizeof(s_buffer),
//                                m_fRecursive, dwFilter, NULL,
//                                &m_overlapped, _NotificationCompletion))
//     {
//         ERR("ReadDirectoryChangesW for '%S' failed (error: %ld)\n",
//             m_szDirectoryPath, GetLastError());
//         return FALSE; // failure
//     }
//
//     return TRUE; // success
// }
// BOOL CDirectoryWatcher::CreateAPCThread()
// {
//     if (s_hThreadAPC != NULL)
//         return TRUE;
//
//     unsigned tid;
//     s_fTerminateAllWatchers = FALSE;
//     s_hThreadAPC = (HANDLE)_beginthreadex(NULL, 0, DirectoryWatcherThreadFuncAPC,
//                                           NULL, 0, &tid);
//     return s_hThreadAPC != NULL;
// }
// BOOL CDirectoryWatcher::RequestAddWatcher()
// {
//     assert(this != NULL);
//
//     // create an APC thread for directory watching
//     if (!CreateAPCThread())
//         return FALSE;
//
//     // request adding the watch
//     QueueUserAPC(_AddDirectoryProcAPC, s_hThreadAPC, (ULONG_PTR)this);
//     return TRUE;
// }
// BOOL CDirectoryWatcher::RequestTermination()
// {
//     assert(this != NULL);
//
//     if (s_hThreadAPC)
//     {
//         QueueUserAPC(_RequestTerminationAPC, s_hThreadAPC, (ULONG_PTR)this);
//         return TRUE;
//     }
//
//     return FALSE;
// }
// /*static*/ void CDirectoryWatcher::RequestAllWatchersTermination()
// {
//     if (!s_hThreadAPC)
//         return;
//
//     // request termination of all directory watches
//     QueueUserAPC(_RequestAllTerminationAPC, s_hThreadAPC, (ULONG_PTR)NULL);
// }
// void CDirectoryWatcher::QuitWatching()
// {
//     assert(this != NULL);
//
//     m_fDead = TRUE;
//     m_hNotifyWnd = NULL;
//     CancelIo(m_hDirectory);
// }
// BOOL CDirectoryWatcher::IsDead()
// {
//     if (m_hNotifyWnd && !::IsWindow(m_hNotifyWnd))
//     {
//         m_hNotifyWnd = NULL;
//         m_fDead = TRUE;
//         CancelIo(m_hDirectory);
//     }
//     return m_fDead;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
