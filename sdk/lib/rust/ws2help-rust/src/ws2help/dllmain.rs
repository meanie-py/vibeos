use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
    fn RegOpenKeyExA(hKey: HKEY, lpSubKey: *const u8, ulOptions: u32, samDesired: u32, phkResult: *mut HKEY) -> i32;
    fn RegQueryValueExA(hKey: HKEY, lpValueName: *const u8, lpReserved: *mut u32, lpType: *mut u32, lpData: LPBYTE, lpcbData: *mut u32) -> i32;
    fn RegCloseKey(hKey: HKEY) -> i32;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
}

// Globals owned by sibling modules (precomp.h `extern`s). Each is defined once as a
// `pub static mut` in its owning module; bring the ones this module touches into scope.
use crate::ws2help::context::{gHandleToIndexMask, gdwSpinCount, ghWriterEvent};
use crate::ws2help::notify::pSDPipe;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut GlobalHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut Ws2helpInitialized: BOOL = FALSE as BOOL;

pub static mut StartupSynchronization: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

pub static mut LibraryHdl: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;
#[no_mangle]
pub unsafe extern "system" fn NewCtxInit() {
    let mut ProductType: NT_PRODUCT_TYPE = NtProductWinNt;
    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
    let mut NumHandleBuckets: u32;
    let mut KeyHandle: HKEY = core::ptr::null_mut();
    let mut RegSize: u32 = core::mem::size_of::<u32>() as u32;
    let mut RegType: u32 = 0;
    let mut Mask: u32;

    RtlGetNtProductType(core::ptr::addr_of_mut!(ProductType));

    GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));

    if SystemInfo.dwNumberOfProcessors > 1 {
        gdwSpinCount = 2000;
    }

    NumHandleBuckets = if ProductType == NtProductWinNt { 8 } else { 32 };

    if RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        b"System\\CurrentControlSet\\Services\\Winsock2\\Parameters\0".as_ptr(),
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(KeyHandle),
    ) == ERROR_SUCCESS as i32
    {
        RegQueryValueExA(
            KeyHandle,
            b"Ws2_32NumHandleBuckets\0".as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::addr_of_mut!(NumHandleBuckets) as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        );

        if SystemInfo.dwNumberOfProcessors > 1 {
            RegQueryValueExA(
                KeyHandle,
                b"Ws2_32SpinCount\0".as_ptr(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(RegType),
                core::ptr::addr_of_mut!(gdwSpinCount) as LPBYTE,
                core::ptr::addr_of_mut!(RegSize),
            );
        }

        RegCloseKey(KeyHandle);
    }

    Mask = 256;
    while (Mask & NumHandleBuckets) == 0 {
        Mask >>= 1;
    }
    NumHandleBuckets = Mask;

    if ProductType == NtProductWinNt {
        if NumHandleBuckets > 32 {
            NumHandleBuckets = 32;
        } else if NumHandleBuckets < 8 {
            NumHandleBuckets = 8;
        }
    } else {
        if NumHandleBuckets > 256 {
            NumHandleBuckets = 256;
        } else if NumHandleBuckets < 32 {
            NumHandleBuckets = 32;
        }
    }

    if gdwSpinCount > 8000 {
        gdwSpinCount = 8000;
    }

    gHandleToIndexMask = NumHandleBuckets.wrapping_sub(1);
}

#[no_mangle]
pub unsafe extern "system" fn Ws2helpInitialize() -> DWORD {

    EnterCriticalSection(core::ptr::addr_of_mut!(StartupSynchronization));
    if Ws2helpInitialized == 0 {
        NewCtxInit();
        Ws2helpInitialized = 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(StartupSynchronization));
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn Ws2HelpDllMain(
    hModule: HANDLE,
    dwReason: DWORD,
    lpReserved: LPVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            LibraryHdl = hModule as HINSTANCE;
            DisableThreadLibraryCalls(hModule as HMODULE);
            InitializeCriticalSection(core::ptr::addr_of_mut!(StartupSynchronization));
            GlobalHeap = GetProcessHeap();
        }
        DLL_THREAD_ATTACH | DLL_THREAD_DETACH => {}
        DLL_PROCESS_DETACH => {
            if !LibraryHdl.is_null() {
                if !lpReserved.is_null() {
                    if !pSDPipe.is_null() {
                        HeapFree(GlobalHeap, 0, pSDPipe as LPVOID);
                    }
                    if !ghWriterEvent.is_null() {
                        CloseHandle(ghWriterEvent);
                    }
                    DeleteCriticalSection(core::ptr::addr_of_mut!(StartupSynchronization));
                    Ws2helpInitialized = 0;
                }
            }
        }
        _ => {}
    }
    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 DLL
//  * FILE:        dll/win32/ws2help/dllmain.c
//  * PURPOSE:     WinSock 2 DLL header
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "precomp.h"
//
// /* DATA **********************************************************************/
//
// HANDLE GlobalHeap;
// BOOL Ws2helpInitialized = FALSE;
// CRITICAL_SECTION StartupSynchronization;
// HINSTANCE LibraryHdl;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// WINAPI
// NewCtxInit(VOID)
// {
//     NT_PRODUCT_TYPE ProductType = NtProductWinNt;
//     SYSTEM_INFO SystemInfo;
//     DWORD NumHandleBuckets;
//     HKEY KeyHandle;
//     DWORD RegSize = sizeof(DWORD);
//     DWORD RegType;
//     DWORD Mask;
//
//     /* Try to figure out if this is a workstation or server install */
//     RtlGetNtProductType(&ProductType);
//
//     /* Get the system info */
//     GetSystemInfo(&SystemInfo);
//
//     /* If this is an MP machine, set the default spinlock */
//     if (SystemInfo.dwNumberOfProcessors > 1) gdwSpinCount = 2000;
//
//     /* Figure how many "Handle Buckets" we'll use. Start with the default */
//     NumHandleBuckets = ProductType == NtProductWinNt ? 8 : 32;
//
//     /* Open the registry settings */
//     if (RegOpenKeyEx(HKEY_LOCAL_MACHINE,
//                      "System\\CurrentControlSet\\Services\\Winsock2\\Parameters",
//                      0,
//                      KEY_QUERY_VALUE,
//                      &KeyHandle) == ERROR_SUCCESS)
//     {
//         /* Query the key */
//         RegQueryValueEx(KeyHandle,
//                         "Ws2_32NumHandleBuckets",
//                         0,
//                         &RegType,
//                         (LPBYTE)&NumHandleBuckets,
//                         &RegSize);
//
//         /* Are we on MP? */
//         if (SystemInfo.dwNumberOfProcessors > 1)
//         {
//             /* Also check for a custom spinlock setting */
//             RegQueryValueEx(KeyHandle,
//                             "Ws2_32SpinCount",
//                             0,
//                             &RegType,
//                             (LPBYTE)&gdwSpinCount,
//                             &RegSize);
//         }
//
//         /* Close the key, we're done */
//         RegCloseKey(KeyHandle);
//     }
//
//     /* Now get the bucket count and normalize it to be log2 and within 256 */
//     for (Mask = 256; !(Mask & NumHandleBuckets); Mask >>= 1);
//     NumHandleBuckets = Mask;
//
//     /* Normalize it again, to be within OS parameters */
//     if (ProductType == NtProductWinNt)
//     {
//         /* Is it within norms for non-server editions? */
//         if (NumHandleBuckets > 32) NumHandleBuckets = 32;
//         else if (NumHandleBuckets < 8) NumHandleBuckets = 8;
//     }
//     else
//     {
//         /* Is it within norms for server editions? */
//         if (NumHandleBuckets > 256) NumHandleBuckets = 256;
//         else if (NumHandleBuckets < 32) NumHandleBuckets = 32;
//     }
//
//     /* Normalize the spincount */
//     if (gdwSpinCount > 8000) gdwSpinCount = 8000;
//
//     /* Set the final mask */
//     gHandleToIndexMask = NumHandleBuckets -1;
// }
//
// DWORD
// WINAPI
// Ws2helpInitialize(VOID)
// {
//     /* Enter the startup CS */
//     EnterCriticalSection(&StartupSynchronization);
//
//     /* Check again for init */
//     if (!Ws2helpInitialized)
//     {
//         /* Initialize us */
//         NewCtxInit();
//         Ws2helpInitialized = TRUE;
//     }
//
//     /* Leave the CS and return */
//     LeaveCriticalSection(&StartupSynchronization);
//     return ERROR_SUCCESS;
// }
//
// BOOL
// __stdcall
// Ws2HelpDllMain(
//     HANDLE hModule,
//     DWORD  dwReason,
//     LPVOID lpReserved)
// {
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//
//             /* Save our handle */
//             LibraryHdl = hModule;
//
//             /* Improve Performance */
//             DisableThreadLibraryCalls(hModule);
//
//             /* Initialize startup CS */
//             InitializeCriticalSection(&StartupSynchronization);
//
//             /* Get Global Heap */
//             GlobalHeap = GetProcessHeap();
//         	break;
//
//         case DLL_THREAD_ATTACH:
// 	    case DLL_THREAD_DETACH:
//             break;
//
// 	    case DLL_PROCESS_DETACH:
//
//             /* Make sure we loaded */
//             if (!LibraryHdl) break;
//
//             /* Check if we are cleaning up */
//             if (lpReserved)
//             {
//                 /* Free the security descriptor */
//                 if (pSDPipe) HeapFree(GlobalHeap, 0, pSDPipe);
//
//                 /* Close the event */
//                 if (ghWriterEvent) CloseHandle(ghWriterEvent);
//
//                 /* Delete the startup CS */
//                 DeleteCriticalSection(&StartupSynchronization);
//                 Ws2helpInitialized = FALSE;
//             }
// 			break;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
