use crate::prelude::*;
use core::ffi::c_void;

// CreateDefaultSecurityDescriptor is a userenv-internal cdecl helper (misc.c, no WINAPI) that
// misc.rs defines as `#[no_mangle] extern "C"`. Declare it cdecl here — the grind put it in the
// extern "system" block below, i.e. a stdcall `@0` reference misc's cdecl def cannot satisfy.
extern "C" {
    fn CreateDefaultSecurityDescriptor() -> PSECURITY_DESCRIPTOR;
}

extern "system" {
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    fn SetEvent(hEvent: HANDLE) -> i32;
    fn OpenEventW(dwDesiredAccess: u32, bInheritHandle: BOOL, lpName: LPCWSTR) -> HANDLE;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn CreateMutexW( lpMutexAttributes: *mut SECURITY_ATTRIBUTES, bInitialOwner: BOOL, lpName: LPCWSTR, ) -> HANDLE;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: u32) -> u32;
    fn SetLastError(dwErrCode: u32);
    fn ReleaseMutex(hMutex: HANDLE) -> BOOL;
    fn FreeLibraryAndExitThread(hLibModule: HMODULE, dwExitCode: DWORD) -> ();
    fn GetModuleHandleExW(dwFlags: DWORD, lpModuleName: LPCWSTR, phModule: *mut HMODULE) -> BOOL;
    fn ResetEvent(hEvent: HANDLE) -> BOOL;
    fn WaitForMultipleObjects(nCount: DWORD, lpHandles: *mut HANDLE, bWaitAll: BOOL, dwMilliseconds: DWORD) -> DWORD;
    fn CreateEventW( lpEventAttributes: *mut SECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE;
    fn CreateThread( lpThreadAttributes: *mut SECURITY_ATTRIBUTES, dwStackSize: u32, lpStartAddress: Option<unsafe extern "system" fn(LPVOID) -> u32>, lpParameter: LPVOID, dwCreationFlags: u32, lpThreadId: *mut u32, ) -> HANDLE;
    fn LocalAlloc(uFlags: u32, uBytes: usize) -> HLOCAL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szLocalGPApplied: [WCHAR; 44] = unsafe {
    ::core::mem::transmute::<
        [u8; 88],
        [WCHAR; 44],
    >(
        *b"u\0s\0e\0r\0e\0n\0v\0:\0 \0U\0s\0e\0r\0 \0G\0r\0o\0u\0p\0 \0P\0o\0l\0i\0c\0y\0 \0h\0a\0s\0 \0b\0e\0e\0n\0 \0a\0p\0p\0l\0i\0e\0d\0\0\0",
    )
};

static mut szLocalGPMutex: [WCHAR; 27] = unsafe {
    ::core::mem::transmute::<[u8; 54], [WCHAR; 27]>(
        *b"u\0s\0e\0r\0e\0n\0v\0:\0 \0u\0s\0e\0r\0 \0p\0o\0l\0i\0c\0y\0 \0m\0u\0t\0e\0x\0\0\0",
    )
};

static mut szLocalGPRefreshEvent: [WCHAR; 35] = unsafe {
    ::core::mem::transmute::<
        [u8; 70],
        [WCHAR; 35],
    >(
        *b"u\0s\0e\0r\0e\0n\0v\0:\0 \0u\0s\0e\0r\0 \0p\0o\0l\0i\0c\0y\0 \0r\0e\0f\0r\0e\0s\0h\0 \0e\0v\0e\0n\0t\0\0\0",
    )
};

static mut szLocalGPForceRefreshEvent: [WCHAR; 41] = unsafe {
    ::core::mem::transmute::<
        [u8; 82],
        [WCHAR; 41],
    >(
        *b"u\0s\0e\0r\0e\0n\0v\0:\0 \0u\0s\0e\0r\0 \0p\0o\0l\0i\0c\0y\0 \0f\0o\0r\0c\0e\0 \0r\0e\0f\0r\0e\0s\0h\0 \0e\0v\0e\0n\0t\0\0\0",
    )
};

static mut szLocalGPDoneEvent: [WCHAR; 43] = unsafe {
    ::core::mem::transmute::<
        [u8; 86],
        [WCHAR; 43],
    >(
        *b"u\0s\0e\0r\0e\0n\0v\0:\0 \0U\0s\0e\0r\0 \0P\0o\0l\0i\0c\0y\0 \0F\0o\0r\0e\0g\0r\0o\0u\0n\0d\0 \0D\0o\0n\0e\0 \0E\0v\0e\0n\0t\0\0\0",
    )
};

static mut szMachineGPApplied: [WCHAR; 54] = unsafe {
    ::core::mem::transmute::<
        [u8; 108],
        [WCHAR; 54],
    >(
        *b"G\0l\0o\0b\0a\0l\0\\\0u\0s\0e\0r\0e\0n\0v\0:\0 \0M\0a\0c\0h\0i\0n\0e\0 \0G\0r\0o\0u\0p\0 \0P\0o\0l\0i\0c\0y\0 \0h\0a\0s\0 \0b\0e\0e\0n\0 \0a\0p\0p\0l\0i\0e\0d\0\0\0",
    )
};

static mut szMachineGPMutex: [WCHAR; 37] = unsafe {
    ::core::mem::transmute::<
        [u8; 74],
        [WCHAR; 37],
    >(
        *b"G\0l\0o\0b\0a\0l\0\\\0u\0s\0e\0r\0e\0n\0v\0:\0 \0m\0a\0c\0h\0i\0n\0e\0 \0p\0o\0l\0i\0c\0y\0 \0m\0u\0t\0e\0x\0\0\0",
    )
};

static mut szMachineGPRefreshEvent: [WCHAR; 45] = unsafe {
    ::core::mem::transmute::<
        [u8; 90],
        [WCHAR; 45],
    >(
        *b"G\0l\0o\0b\0a\0l\0\\\0u\0s\0e\0r\0e\0n\0v\0:\0 \0m\0a\0c\0h\0i\0n\0e\0 \0p\0o\0l\0i\0c\0y\0 \0r\0e\0f\0r\0e\0s\0h\0 \0e\0v\0e\0n\0t\0\0\0",
    )
};

static mut szMachineGPForceRefreshEvent: [WCHAR; 51] = unsafe {
    ::core::mem::transmute::<
        [u8; 102],
        [WCHAR; 51],
    >(
        *b"G\0l\0o\0b\0a\0l\0\\\0u\0s\0e\0r\0e\0n\0v\0:\0 \0m\0a\0c\0h\0i\0n\0e\0 \0p\0o\0l\0i\0c\0y\0 \0f\0o\0r\0c\0e\0 \0r\0e\0f\0r\0e\0s\0h\0 \0e\0v\0e\0n\0t\0\0\0",
    )
};

static mut szMachineGPDoneEvent: [WCHAR; 53] = unsafe {
    ::core::mem::transmute::<
        [u8; 106],
        [WCHAR; 53],
    >(
        *b"G\0l\0o\0b\0a\0l\0\\\0u\0s\0e\0r\0e\0n\0v\0:\0 \0M\0a\0c\0h\0i\0n\0e\0 \0P\0o\0l\0i\0c\0y\0 \0F\0o\0r\0e\0g\0r\0o\0u\0n\0d\0 \0D\0o\0n\0e\0 \0E\0v\0e\0n\0t\0\0\0",
    )
};

static mut GPNotifyLock: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

static mut NotificationList: PGP_NOTIFY = ::core::ptr::null::<_GP_NOTIFY>() as *mut _GP_NOTIFY;

static mut GPNotificationAction: GP_ACTION = gpaUpdate;

static mut hNotificationThread: HANDLE = core::ptr::null_mut();

static mut hNotificationThreadEvent: HANDLE = core::ptr::null_mut();

static mut hLocalGPAppliedEvent: HANDLE = core::ptr::null_mut();

static mut hMachineGPAppliedEvent: HANDLE = core::ptr::null_mut();

#[repr(C)]
pub struct _GP_NOTIFY {
    pub Next: *mut _GP_NOTIFY,
    pub hEvent: HANDLE,
    pub bMachine: BOOL,
}
pub type PGP_NOTIFY = *mut _GP_NOTIFY;

pub type GP_ACTION = u32;
pub const gpaUpdate: u32 = 0;
pub const gpaTerminate: u32 = 1;
#[no_mangle]
pub unsafe extern "C" fn InitializeGPNotifications() {
    InitializeCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));
}
#[no_mangle]
pub unsafe extern "C" fn UninitializeGPNotifications() {
    EnterCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));

    if !hNotificationThread.is_null() {
        GPNotificationAction = gpaTerminate;
        SetEvent(hNotificationThreadEvent);

        LeaveCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));

        WaitForSingleObject(hNotificationThread, INFINITE);

        EnterCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));

        if !hNotificationThread.is_null() {
            CloseHandle(hNotificationThread);
            hNotificationThread = core::ptr::null_mut();
        }
    }

    if !hNotificationThreadEvent.is_null() {
        CloseHandle(hNotificationThreadEvent);
        hNotificationThreadEvent = core::ptr::null_mut();
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));

    DeleteCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));
}
unsafe extern "C" fn NotifyGPEvents(mut bMachine: BOOL) {
    let mut Notify: PGP_NOTIFY = NotificationList;
    while !Notify.is_null() {
        if (*Notify).bMachine == bMachine {
            SetEvent((*Notify).hEvent);
        }
        Notify = (*Notify).Next;
    }
}
unsafe extern "system" fn GPNotificationThreadProc(mut lpParameter: LPVOID) -> DWORD {
    let mut hModule: HMODULE = core::ptr::null_mut();
    let mut WaitResult: DWORD;
    let mut WaitCount: DWORD;
    let mut WaitHandles: [HANDLE; 3] = [core::ptr::null_mut(); 3];

    if GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        hInstance as LPCWSTR,
        &mut hModule,
    ) != 0
    {
        EnterCriticalSection(&mut GPNotifyLock);

        WaitHandles[0] = hNotificationThreadEvent;
        loop {
            if NotificationList == core::ptr::null_mut() {
                break;
            }

            WaitCount = 2;
            WaitHandles[1] = hMachineGPAppliedEvent;

            if hLocalGPAppliedEvent != core::ptr::null_mut() {
                WaitHandles[2] = hLocalGPAppliedEvent;
                WaitCount += 1;
            }

            LeaveCriticalSection(&mut GPNotifyLock);

            WaitResult = WaitForMultipleObjects(
                WaitCount,
                &mut WaitHandles as *mut HANDLE,
                0,
                INFINITE,
            );

            EnterCriticalSection(&mut GPNotifyLock);

            if WaitResult != u32::MAX {
                if WaitResult == WAIT_OBJECT_0 {
                    ResetEvent(hNotificationThreadEvent);

                    if GPNotificationAction as u32 == 1u32 {
                        break;
                    }
                } else if WaitResult == WAIT_OBJECT_0 + 1 || WaitResult == WAIT_OBJECT_0 + 2 {
                    if NotificationList != core::ptr::null_mut() {
                        NotifyGPEvents((WaitResult == WAIT_OBJECT_0 + 1) as BOOL);
                    }
                } else if WaitResult == WAIT_ABANDONED_0 + 2 {
                    if hLocalGPAppliedEvent != core::ptr::null_mut() {
                        CloseHandle(hLocalGPAppliedEvent);
                        hLocalGPAppliedEvent = core::ptr::null_mut();
                    }
                } else if WaitResult == WAIT_ABANDONED_0 || WaitResult == WAIT_ABANDONED_0 + 1 {
                    break;
                } else {
                    break;
                }

                if NotificationList == core::ptr::null_mut() {
                    break;
                }
            } else {
                break;
            }
        }

        CloseHandle(hNotificationThread);
        CloseHandle(hNotificationThreadEvent);
        hNotificationThread = core::ptr::null_mut();
        hNotificationThreadEvent = core::ptr::null_mut();

        if hLocalGPAppliedEvent != core::ptr::null_mut() {
            CloseHandle(hLocalGPAppliedEvent);
            hLocalGPAppliedEvent = core::ptr::null_mut();
        }
        if hMachineGPAppliedEvent != core::ptr::null_mut() {
            CloseHandle(hMachineGPAppliedEvent);
            hMachineGPAppliedEvent = core::ptr::null_mut();
        }

        LeaveCriticalSection(&mut GPNotifyLock);

        FreeLibraryAndExitThread(hModule, 0);
    }

    1
}
unsafe extern "C" fn CreateGPEvent(
    mut bMachine: BOOL,
    mut lpSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> HANDLE {
    let mut hEvent: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut SecurityAttributes: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
        nLength: 0,
        lpSecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        bInheritHandle: 0,
    };
    SecurityAttributes.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD;
    SecurityAttributes.lpSecurityDescriptor = lpSecurityDescriptor as LPVOID;
    SecurityAttributes.bInheritHandle = FALSE as BOOL;
    hEvent = CreateEventW(
        &raw mut SecurityAttributes,
        TRUE,
        FALSE,
        if bMachine != 0 {
            &raw const szMachineGPApplied as LPCWSTR
        } else {
            &raw const szLocalGPApplied as LPCWSTR
        },
    );
    return hEvent;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterGPNotification(mut hEvent: HANDLE, mut bMachine: BOOL) -> BOOL {
    let mut Notify: *mut _GP_NOTIFY;
    let mut lpSecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut Ret: BOOL = 0;

    EnterCriticalSection(&mut GPNotifyLock);

    'cleanup: {
    if hNotificationThreadEvent.is_null() {
        hNotificationThreadEvent = CreateEventW(core::ptr::null_mut(), 1, 0, core::ptr::null());
        if hNotificationThreadEvent.is_null() {
            break 'cleanup;
        }
    }

    if hMachineGPAppliedEvent.is_null() {
        lpSecurityDescriptor = CreateDefaultSecurityDescriptor();
        if lpSecurityDescriptor.is_null() {
            break 'cleanup;
        }

        hMachineGPAppliedEvent = CreateGPEvent(1, lpSecurityDescriptor);
        if hMachineGPAppliedEvent.is_null() {
            break 'cleanup;
        }
    }

    if bMachine == 0 && hLocalGPAppliedEvent.is_null() {
        if lpSecurityDescriptor.is_null() {
            lpSecurityDescriptor = CreateDefaultSecurityDescriptor();
            if lpSecurityDescriptor.is_null() {
                break 'cleanup;
            }
        }

        hLocalGPAppliedEvent = CreateGPEvent(0, lpSecurityDescriptor);
        if hLocalGPAppliedEvent.is_null() {
            break 'cleanup;
        }
    }

    if hNotificationThread.is_null() {
        hNotificationThread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(GPNotificationThreadProc),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
    }

    if !hNotificationThread.is_null() {
        Notify = LocalAlloc(LMEM_FIXED, core::mem::size_of::<_GP_NOTIFY>()) as *mut _GP_NOTIFY;
        if !Notify.is_null() {
            (*Notify).Next = NotificationList;
            (*Notify).hEvent = hEvent;
            (*Notify).bMachine = bMachine;

            NotificationList = Notify;

            GPNotificationAction = gpaUpdate;
            SetEvent(hNotificationThreadEvent);

            Ret = 1;
        }
    }

    } // 'cleanup

    LeaveCriticalSection(&mut GPNotifyLock);

    if !lpSecurityDescriptor.is_null() {
        LocalFree(lpSecurityDescriptor);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterGPNotification(hEvent: HANDLE) -> i32 {
    let mut Notify: *mut _GP_NOTIFY = core::ptr::null_mut();
    let mut NotifyLink: *mut *mut _GP_NOTIFY = core::ptr::null_mut();
    let mut Ret: i32 = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));

    Notify = NotificationList;
    NotifyLink = core::ptr::addr_of_mut!(NotificationList);

    while !Notify.is_null() {
        if (*Notify).hEvent == hEvent {
            *NotifyLink = (*Notify).Next;
            LocalFree(Notify as HLOCAL);

            if !hNotificationThread.is_null() && !hNotificationThreadEvent.is_null() {
                GPNotificationAction = 0;
                SetEvent(hNotificationThreadEvent);
            }

            Ret = 1;
            break;
        }

        NotifyLink = core::ptr::addr_of_mut!((*Notify).Next);
        Notify = (*Notify).Next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(GPNotifyLock));

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RefreshPolicy(mut bMachine: BOOL) -> BOOL {
    let mut hEvent: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Ret: BOOL = 1;
    hEvent = OpenEventW(
        EVENT_MODIFY_STATE,
        FALSE,
        if bMachine != 0 {
            &raw const szMachineGPRefreshEvent as LPCWSTR
        } else {
            &raw const szLocalGPRefreshEvent as LPCWSTR
        },
    );
    if !hEvent.is_null() {
        Ret = SetEvent(hEvent) as BOOL;
        CloseHandle(hEvent);
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn RefreshPolicyEx(bMachine: BOOL, dwOptions: u32) -> BOOL {
    const RP_FORCE: u32 = 1;

    if dwOptions & !RP_FORCE != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if dwOptions & RP_FORCE != 0 {
        let mut hEvent: HANDLE;
        let mut Ret: BOOL = 1;

        hEvent = OpenEventW(
            EVENT_MODIFY_STATE,
            FALSE,
            if bMachine != 0 {
                core::ptr::addr_of!(szMachineGPForceRefreshEvent) as LPCWSTR
            } else {
                core::ptr::addr_of!(szLocalGPForceRefreshEvent) as LPCWSTR
            },
        );
        if !hEvent.is_null() {
            Ret = SetEvent(hEvent) as BOOL;
            CloseHandle(hEvent);
        }

        return Ret;
    } else {
        return RefreshPolicy(bMachine);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EnterCriticalPolicySection(bMachine: u8) -> HANDLE {
    let mut SecurityAttributes: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
        nLength: 0,
        lpSecurityDescriptor: core::ptr::null_mut(),
        bInheritHandle: 0,
    };
    let mut lpSecurityDescriptor: PSECURITY_DESCRIPTOR = CreateDefaultSecurityDescriptor();
    
    if !lpSecurityDescriptor.is_null() {
        SecurityAttributes.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
        SecurityAttributes.lpSecurityDescriptor = lpSecurityDescriptor;
        SecurityAttributes.bInheritHandle = 0;
        
        let hSection: HANDLE = CreateMutexW(
            &mut SecurityAttributes,
            FALSE,
            if bMachine != 0 {
                szMachineGPMutex.as_ptr()
            } else {
                szLocalGPMutex.as_ptr()
            },
        );
        
        LocalFree(lpSecurityDescriptor as HLOCAL);
        
        if !hSection.is_null() {
            if WaitForSingleObject(hSection, 600000u32) != 0xffffffffu32 {
                return hSection;
            }
            CloseHandle(hSection);
        }
    }
    
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn LeaveCriticalPolicySection(hSection: HANDLE) -> BOOL {
    if hSection.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    let Ret = ReleaseMutex(hSection);
    CloseHandle(hSection);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn WaitForUserPolicyForegroundProcessing() -> BOOL {
    let mut hEvent: HANDLE = core::ptr::null_mut();
    let mut Ret: BOOL = 0;

    hEvent = OpenEventW(SYNCHRONIZE, FALSE, szLocalGPDoneEvent.as_ptr());
    if !hEvent.is_null() {
        Ret = (WaitForSingleObject(hEvent, INFINITE) != 0xFFFFFFFF) as BOOL;
        CloseHandle(hEvent);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn WaitForMachinePolicyForegroundProcessing() -> BOOL {
    let mut hEvent: HANDLE = core::ptr::null_mut();
    let mut Ret: BOOL = 0;
    
    hEvent = OpenEventW(
        SYNCHRONIZE,
        FALSE,
        szMachineGPDoneEvent.as_ptr()
    );
    if !hEvent.is_null() {
        Ret = (WaitForSingleObject(hEvent, INFINITE) != 0xffffffff) as BOOL;
        CloseHandle(hEvent);
    }
    
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetAppliedGPOListA(
    dwFlags: u32,
    pMachineName: LPCSTR,
    pSidUser: PSID,
    pGuidExtension: *mut GUID,
    ppGPOList: *mut *mut _GROUP_POLICY_OBJECTA,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn GetAppliedGPOListW(
    dwFlags: u32,
    pMachineName: LPCWSTR,
    pSidUser: PSID,
    pGuidExtension: *mut GUID,
    ppGPOList: *mut *mut _GROUP_POLICY_OBJECTW,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2006 ReactOS Team
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
//  * FILE:            dll/win32/userenv/gpolicy.c
//  * PURPOSE:         Group policy functions
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _GP_NOTIFY
// {
//     struct _GP_NOTIFY *Next;
//     HANDLE hEvent;
//     BOOL bMachine;
// } GP_NOTIFY, *PGP_NOTIFY;
//
// typedef enum
// {
//     gpaUpdate = 0,
//     gpaTerminate
// } GP_ACTION;
//
// static const WCHAR szLocalGPApplied[] = L"userenv: User Group Policy has been applied";
// static const WCHAR szLocalGPMutex[] = L"userenv: user policy mutex";
// static const WCHAR szLocalGPRefreshEvent[] = L"userenv: user policy refresh event";
// static const WCHAR szLocalGPForceRefreshEvent[] = L"userenv: user policy force refresh event";
// static const WCHAR szLocalGPDoneEvent[] = L"userenv: User Policy Foreground Done Event";
// static const WCHAR szMachineGPApplied[] = L"Global\\userenv: Machine Group Policy has been applied";
// static const WCHAR szMachineGPMutex[] = L"Global\\userenv: machine policy mutex";
// static const WCHAR szMachineGPRefreshEvent[] = L"Global\\userenv: machine policy refresh event";
// static const WCHAR szMachineGPForceRefreshEvent[] = L"Global\\userenv: machine policy force refresh event";
// static const WCHAR szMachineGPDoneEvent[] = L"Global\\userenv: Machine Policy Foreground Done Event";
//
// static CRITICAL_SECTION GPNotifyLock;
// static PGP_NOTIFY NotificationList = NULL;
// static GP_ACTION GPNotificationAction = gpaUpdate;
// static HANDLE hNotificationThread = NULL;
// static HANDLE hNotificationThreadEvent = NULL;
// static HANDLE hLocalGPAppliedEvent = NULL;
// static HANDLE hMachineGPAppliedEvent = NULL;
//
// VOID
// InitializeGPNotifications(VOID)
// {
//     InitializeCriticalSection(&GPNotifyLock);
// }
//
// VOID
// UninitializeGPNotifications(VOID)
// {
//     EnterCriticalSection(&GPNotifyLock);
//
//     /* rundown the notification thread */
//     if (hNotificationThread != NULL)
//     {
//         ASSERT(hNotificationThreadEvent != NULL);
//
//         /* notify the thread */
//         GPNotificationAction = gpaTerminate;
//         SetEvent(hNotificationThreadEvent);
//
//         LeaveCriticalSection(&GPNotifyLock);
//
//         /* wait for the thread to terminate itself */
//         WaitForSingleObject(hNotificationThread,
//                             INFINITE);
//
//         EnterCriticalSection(&GPNotifyLock);
//
//         if (hNotificationThread != NULL)
//         {
//             /* the handle should be closed by the thread,
//                just in case that didn't happen for an unknown reason */
//             CloseHandle(hNotificationThread);
//             hNotificationThread = NULL;
//         }
//     }
//
//     if (hNotificationThreadEvent != NULL)
//     {
//         CloseHandle(hNotificationThreadEvent);
//         hNotificationThreadEvent = NULL;
//     }
//
//     LeaveCriticalSection(&GPNotifyLock);
//
//     DeleteCriticalSection(&GPNotifyLock);
// }
//
// static
// VOID
// NotifyGPEvents(IN BOOL bMachine)
// {
//     PGP_NOTIFY Notify = NotificationList;
//
//     while (Notify != NULL)
//     {
//         if (Notify->bMachine == bMachine)
//         {
//             SetEvent(Notify->hEvent);
//         }
//
//         Notify = Notify->Next;
//     }
// }
//
// static
// DWORD
// WINAPI
// GPNotificationThreadProc(IN LPVOID lpParameter)
// {
//     HMODULE hModule;
//     DWORD WaitResult, WaitCount;
//     HANDLE WaitHandles[3];
//
//     /* reference the library so we don't screw up if the application
//        causes the DLL to unload while this thread is still running */
//     if (GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
//                            (LPCWSTR)hInstance,
//                            &hModule))
//     {
//         ASSERT(hModule == hInstance);
//
//         EnterCriticalSection(&GPNotifyLock);
//
//         ASSERT(hNotificationThreadEvent != NULL);
//         WaitHandles[0] = hNotificationThreadEvent;
//         for (;;)
//         {
//             ASSERT(hMachineGPAppliedEvent != NULL);
//
//             if (NotificationList == NULL)
//                 break;
//
//             WaitCount = 2;
//             WaitHandles[1] = hMachineGPAppliedEvent;
//
//             if (hLocalGPAppliedEvent != NULL)
//             {
//                 WaitHandles[2] = hLocalGPAppliedEvent;
//                 WaitCount++;
//             }
//
//             LeaveCriticalSection(&GPNotifyLock);
//
//             WaitResult = WaitForMultipleObjects(WaitCount,
//                                                 WaitHandles,
//                                                 FALSE,
//                                                 INFINITE);
//
//             EnterCriticalSection(&GPNotifyLock);
//
//             if (WaitResult != WAIT_FAILED)
//             {
//                 if (WaitResult == WAIT_OBJECT_0)
//                 {
//                     ResetEvent(hNotificationThreadEvent);
//
//                     if (GPNotificationAction == gpaTerminate)
//                     {
//                         /* terminate the thread */
//                         break;
//                     }
//                 }
//                 else if (WaitResult == WAIT_OBJECT_0 + 1 || WaitResult == WAIT_OBJECT_0 + 2)
//                 {
//                     /* group policies have been applied */
//                     if (NotificationList != NULL)
//                     {
//                         NotifyGPEvents((WaitResult == WAIT_OBJECT_0 + 1));
//                     }
//                 }
//                 else if (WaitResult == WAIT_ABANDONED_0 + 2)
//                 {
//                     /* In case the local group policies event was abandoned, keep watching!
//                        But close the handle as it's no longer of any use. */
//                     if (hLocalGPAppliedEvent != NULL)
//                     {
//                         CloseHandle(hLocalGPAppliedEvent);
//                         hLocalGPAppliedEvent = NULL;
//                     }
//                 }
//                 else if (WaitResult == WAIT_ABANDONED_0 || WaitResult == WAIT_ABANDONED_0 + 1)
//                 {
//                     /* terminate the thread if the machine group policies event was abandoned
//                        or for some reason the rundown event got abandoned. */
//                     break;
//                 }
//                 else
//                 {
//                     DPRINT("Unexpected wait result watching the group policy events: 0x%x\n", WaitResult);
//                     ASSERT(FALSE);
//                     break;
//                 }
//
//                 if (NotificationList == NULL)
//                     break;
//             }
//             else
//                 break;
//
//         }
//
//         /* cleanup handles no longer used */
//         ASSERT(hNotificationThread != NULL);
//         ASSERT(hNotificationThreadEvent != NULL);
//
//         CloseHandle(hNotificationThread);
//         CloseHandle(hNotificationThreadEvent);
//         hNotificationThread = NULL;
//         hNotificationThreadEvent = NULL;
//
//         if (hLocalGPAppliedEvent != NULL)
//         {
//             CloseHandle(hLocalGPAppliedEvent);
//             hLocalGPAppliedEvent = NULL;
//         }
//         if (hMachineGPAppliedEvent != NULL)
//         {
//             CloseHandle(hMachineGPAppliedEvent);
//             hMachineGPAppliedEvent = NULL;
//         }
//
//         LeaveCriticalSection(&GPNotifyLock);
//
//         /* dereference the library and exit */
//         FreeLibraryAndExitThread(hModule,
//                                  0);
//     }
//     else
//     {
//         DPRINT1("Referencing the library failed!\n");
//     }
//
//     return 1;
// }
//
// static
// HANDLE
// CreateGPEvent(IN BOOL bMachine,
//               IN PSECURITY_DESCRIPTOR lpSecurityDescriptor)
// {
//     HANDLE hEvent;
//     SECURITY_ATTRIBUTES SecurityAttributes;
//
//     SecurityAttributes.nLength = sizeof(SECURITY_ATTRIBUTES);
//     SecurityAttributes.lpSecurityDescriptor = lpSecurityDescriptor;
//     SecurityAttributes.bInheritHandle = FALSE;
//
//     hEvent = CreateEventW(&SecurityAttributes,
//                           TRUE,
//                           FALSE,
//                           (bMachine ? szMachineGPApplied : szLocalGPApplied));
//
//     return hEvent;
// }
//
// BOOL
// WINAPI
// RegisterGPNotification(IN HANDLE hEvent,
//                        IN BOOL bMachine)
// {
//     PGP_NOTIFY Notify;
//     PSECURITY_DESCRIPTOR lpSecurityDescriptor = NULL;
//     BOOL Ret = FALSE;
//
//     EnterCriticalSection(&GPNotifyLock);
//
//     /* create the thread notification event */
//     if (hNotificationThreadEvent == NULL)
//     {
//         hNotificationThreadEvent = CreateEvent(NULL,
//                                                TRUE,
//                                                FALSE,
//                                                NULL);
//         if (hNotificationThreadEvent == NULL)
//         {
//             goto Cleanup;
//         }
//     }
//
//     /* create or open the machine group policy event */
//     if (hMachineGPAppliedEvent == NULL)
//     {
//         lpSecurityDescriptor = CreateDefaultSecurityDescriptor();
//         if (lpSecurityDescriptor == NULL)
//         {
//             goto Cleanup;
//         }
//
//         hMachineGPAppliedEvent = CreateGPEvent(TRUE,
//                                                lpSecurityDescriptor);
//         if (hMachineGPAppliedEvent == NULL)
//         {
//             goto Cleanup;
//         }
//     }
//
//     /* create or open the local group policy event only if necessary */
//     if (!bMachine && hLocalGPAppliedEvent == NULL)
//     {
//         if (lpSecurityDescriptor == NULL)
//         {
//             lpSecurityDescriptor = CreateDefaultSecurityDescriptor();
//             if (lpSecurityDescriptor == NULL)
//             {
//                 goto Cleanup;
//             }
//         }
//
//         hLocalGPAppliedEvent = CreateGPEvent(FALSE,
//                                              lpSecurityDescriptor);
//         if (hLocalGPAppliedEvent == NULL)
//         {
//             goto Cleanup;
//         }
//     }
//
//     if (hNotificationThread == NULL)
//     {
//         hNotificationThread = CreateThread(NULL,
//                                            0,
//                                            GPNotificationThreadProc,
//                                            NULL,
//                                            0,
//                                            NULL);
//     }
//
//     if (hNotificationThread != NULL)
//     {
//         Notify = (PGP_NOTIFY)LocalAlloc(LMEM_FIXED,
//                                         sizeof(GP_NOTIFY));
//         if (Notify != NULL)
//         {
//             /* add the item to the beginning of the list */
//             Notify->Next = NotificationList;
//             Notify->hEvent = hEvent;
//             Notify->bMachine = bMachine;
//
//             NotificationList = Notify;
//
//             /* notify the thread */
//             GPNotificationAction = gpaUpdate;
//             SetEvent(hNotificationThreadEvent);
//
//             Ret = TRUE;
//         }
//     }
//
// Cleanup:
//     LeaveCriticalSection(&GPNotifyLock);
//
//     if (lpSecurityDescriptor != NULL)
//     {
//         LocalFree((HLOCAL)lpSecurityDescriptor);
//     }
//
//     /* NOTE: don't delete the events or close the handles created */
//
//     return Ret;
// }
//
// BOOL
// WINAPI
// UnregisterGPNotification(IN HANDLE hEvent)
// {
//     PGP_NOTIFY Notify = NULL, *NotifyLink;
//     BOOL Ret = FALSE;
//
//     EnterCriticalSection(&GPNotifyLock);
//
//     Notify = NotificationList;
//     NotifyLink = &NotificationList;
//
//     while (Notify != NULL)
//     {
//         if (Notify->hEvent == hEvent)
//         {
//             /* remove and free the item */
//             *NotifyLink = Notify->Next;
//             LocalFree((HLOCAL)Notify);
//
//             /* notify the thread */
//             if (hNotificationThread != NULL &&
//                 hNotificationThreadEvent != NULL)
//             {
//                 GPNotificationAction = gpaUpdate;
//                 SetEvent(hNotificationThreadEvent);
//             }
//
//             Ret = TRUE;
//             break;
//         }
//
//         NotifyLink = &Notify->Next;
//         Notify = Notify->Next;
//     }
//
//     LeaveCriticalSection(&GPNotifyLock);
//
//     return Ret;
// }
//
// BOOL
// WINAPI
// RefreshPolicy(IN BOOL bMachine)
// {
//     HANDLE hEvent;
//     BOOL Ret = TRUE;
//
//     hEvent = OpenEventW(EVENT_MODIFY_STATE,
//                         FALSE,
//                         (bMachine ? szMachineGPRefreshEvent : szLocalGPRefreshEvent));
//     if (hEvent != NULL)
//     {
//         Ret = SetEvent(hEvent);
//         CloseHandle(hEvent);
//     }
//
//     /* return TRUE even if the mutex doesn't exist! */
//     return Ret;
// }
//
// BOOL
// WINAPI
// RefreshPolicyEx(IN BOOL bMachine,
//                 IN DWORD dwOptions)
// {
//     if (dwOptions & ~RP_FORCE)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (dwOptions & RP_FORCE)
//     {
//         HANDLE hEvent;
//         BOOL Ret = TRUE;
//
//         hEvent = OpenEventW(EVENT_MODIFY_STATE,
//                             FALSE,
//                             (bMachine ? szMachineGPForceRefreshEvent : szLocalGPForceRefreshEvent));
//         if (hEvent != NULL)
//         {
//             Ret = SetEvent(hEvent);
//             CloseHandle(hEvent);
//         }
//
//         /* return TRUE even if the mutex doesn't exist! */
//         return Ret;
//     }
//     else
//     {
//         return RefreshPolicy(bMachine);
//     }
// }
//
// HANDLE
// WINAPI
// EnterCriticalPolicySection(IN BOOL bMachine)
// {
//     SECURITY_ATTRIBUTES SecurityAttributes;
//     PSECURITY_DESCRIPTOR lpSecurityDescriptor;
//     HANDLE hSection;
//
//     /* create or open the mutex */
//     lpSecurityDescriptor = CreateDefaultSecurityDescriptor();
//     if (lpSecurityDescriptor != NULL)
//     {
//         SecurityAttributes.nLength = sizeof(SECURITY_ATTRIBUTES);
//         SecurityAttributes.lpSecurityDescriptor = lpSecurityDescriptor;
//         SecurityAttributes.bInheritHandle = FALSE;
//
//         hSection = CreateMutexW(&SecurityAttributes,
//                                 FALSE,
//                                 (bMachine ? szMachineGPMutex : szLocalGPMutex));
//
//         LocalFree((HLOCAL)lpSecurityDescriptor);
//
//         if (hSection != NULL)
//         {
//             /* wait up to 10 minutes */
//             if (WaitForSingleObject(hSection,
//                                     600000) != WAIT_FAILED)
//             {
//                 return hSection;
//             }
//
//             CloseHandle(hSection);
//         }
//     }
//
//     return NULL;
// }
//
// BOOL
// WINAPI
// LeaveCriticalPolicySection(IN HANDLE hSection)
// {
//     BOOL Ret;
//
//     if (hSection == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Ret = ReleaseMutex(hSection);
//     CloseHandle(hSection);
//
//     return Ret;
// }
//
// BOOL
// WINAPI
// WaitForUserPolicyForegroundProcessing(VOID)
// {
//     HANDLE hEvent;
//     BOOL Ret = FALSE;
//
//     hEvent = OpenEventW(SYNCHRONIZE,
//                         FALSE,
//                         szLocalGPDoneEvent);
//     if (hEvent != NULL)
//     {
//         Ret = WaitForSingleObject(hEvent,
//                                   INFINITE) != WAIT_FAILED;
//         CloseHandle(hEvent);
//     }
//
//     return Ret;
// }
//
// BOOL
// WINAPI
// WaitForMachinePolicyForegroundProcessing(VOID)
// {
//     HANDLE hEvent;
//     BOOL Ret = FALSE;
//
//     hEvent = OpenEventW(SYNCHRONIZE,
//                         FALSE,
//                         szMachineGPDoneEvent);
//     if (hEvent != NULL)
//     {
//         Ret = WaitForSingleObject(hEvent,
//                                   INFINITE) != WAIT_FAILED;
//         CloseHandle(hEvent);
//     }
//
//     return Ret;
// }
//
// DWORD
// WINAPI
// GetAppliedGPOListA(
//        DWORD dwFlags,
//        LPCSTR pMachineName,
//        PSID pSidUser,
//        GUID *pGuidExtension,
//       PGROUP_POLICY_OBJECTA *ppGPOList
// )
// {
//     DPRINT1("GetAppliedGPOListA is UNIMPLEMENTED!\n");
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD
// WINAPI
// GetAppliedGPOListW(
//        DWORD dwFlags,
//        LPCWSTR pMachineName,
//        PSID pSidUser,
//        GUID *pGuidExtension,
//       PGROUP_POLICY_OBJECTW *ppGPOList
// )
// {
//     DPRINT1("GetAppliedGPOListW is UNIMPLEMENTED!\n");
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
