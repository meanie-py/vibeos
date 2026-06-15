use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

use crate::substrate::{TRUE, FALSE};
use crate::client::utils::Basep8BitStringToUnicodeString;
use crate::client::except::GlobalTopLevelExceptionFilter;
use crate::client::actctx::BasepProbeForDllManifest;
use crate::client::path::{
    BaseWindowsDirectory, BaseWindowsSystemDirectory, BaseDefaultPath, BaseDefaultPathAppend,
};
use crate::client::heapmem::BaseDllInitializeMemoryManager;
use crate::proc_helpers::InitCommandLines;
use crate::string::nls::{NlsInit, NlsUninit};
use crate::k32_globals::{BasepAppCertDllsList, gcsAppCert};

extern "system" {
    fn CreateRemoteThread( hProcess: HANDLE, lpThreadAttributes: *mut _SECURITY_ATTRIBUTES, dwStackSize: u32, lpStartAddress: PTHREAD_START_ROUTINE, lpParameter: PVOID, dwCreationFlags: u32, lpThreadId: *mut u32, ) -> HANDLE;
    fn ExitThread(dwExitCode: u32);
}

extern "C" {
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> core::ffi::c_int;
    fn RtlStringCbPrintfW(pszDest: *mut u16, cbDest: usize, pszFormat: *const u16, ...) -> NTSTATUS;
}

extern "system" {
    fn ConDllInitialize(dwReason: DWORD, lpReserved: PVOID) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut BaseStaticServerData: PBASE_STATIC_SERVER_DATA =
    ::core::ptr::null::<_BASE_STATIC_SERVER_DATA>() as *mut _BASE_STATIC_SERVER_DATA;

pub static mut BaseRunningInServerProcess: BOOLEAN = FALSE as BOOLEAN;

pub static mut BaseDefaultPathBuffer: [WCHAR; 6140] = [0; 6140];

pub static mut BaseNamedObjectDirectory: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut hCurrentModule: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub static mut kernel32_handle: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub static mut Peb: PPEB = ::core::ptr::null::<_PEB>() as *mut _PEB;

pub static mut SessionId: ULONG = 0;

static mut DllInitialized: BOOL = FALSE;

pub static mut BaseDllDirectoryLock: RTL_CRITICAL_SECTION = RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

pub const WIN_OBJ_DIR: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(
        *b"\\\0W\0i\0n\0d\0o\0w\0s\0\0\0",
    )
};

pub const SESSION_DIR: [::core::ffi::c_ushort; 10] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_ushort; 10]>(
        *b"\\\0S\0e\0s\0s\0i\0o\0n\0s\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "system" fn BaseCreateThreadPoolThread(
    Function: PTHREAD_START_ROUTINE,
    Parameter: PVOID,
    ThreadHandle: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    *ThreadHandle = CreateRemoteThread(
        NtCurrentProcess(),
        core::ptr::null_mut(),
        0,
        Function,
        Parameter,
        CREATE_SUSPENDED,
        core::ptr::null_mut(),
    );
    if (*ThreadHandle).is_null() {
        Status = (*NtCurrentTeb()).LastStatusValue as NTSTATUS;
        if NT_SUCCESS(Status) {
            Status = STATUS_UNSUCCESSFUL;
        }
    } else {
        Status = STATUS_SUCCESS;
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn BaseExitThreadPoolThread(ExitStatus: NTSTATUS) -> NTSTATUS {
    ExitThread(ExitStatus as u32);
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hDll: HANDLE,
    dwReason: DWORD,
    lpReserved: LPVOID,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut ConnectInfo: BASESRV_API_CONNECTINFO = core::mem::zeroed();
    let mut ConnectInfoSize: ULONG = core::mem::size_of::<BASESRV_API_CONNECTINFO>() as ULONG;
    let mut SessionDir: [WCHAR; 256] = [0; 256];

    Basep8BitStringToUnicodeString = Some(RtlAnsiStringToUnicodeString);

    Peb = NtCurrentPeb();
    SessionId = (*Peb).SessionId;

    match dwReason {
        DLL_PROCESS_ATTACH => {
            DbgPrint(
                b"[kernel32] DllMain PROCESS_ATTACH PID=%lu\n\0".as_ptr() as *const i8,
                (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG,
            );

            GlobalTopLevelExceptionFilter = core::mem::transmute::<PVOID, crate::client::except::LPTOP_LEVEL_EXCEPTION_FILTER>(
                RtlEncodePointer(core::ptr::null_mut()),
            );

            RtlSetThreadPoolStartFunc(
                core::mem::transmute::<_, PRTL_START_POOL_THREAD>(Some(BaseCreateThreadPoolThread as unsafe extern "system" fn(_, _, _) -> _)),
                core::mem::transmute::<_, PRTL_EXIT_POOL_THREAD>(Some(BaseExitThreadPoolThread as unsafe extern "system" fn(_) -> _)),
            );

            LdrSetDllManifestProber(core::mem::transmute::<_, PLDR_MANIFEST_PROBER_ROUTINE>(Some(BasepProbeForDllManifest as unsafe extern "system" fn(_, _, _) -> _)));

            LdrDisableThreadCalloutsForDll(hDll as PVOID);

            RtlInitUnicodeString(&mut BaseDefaultPath, core::ptr::null());

            if SessionId == 0 {
                wcscpy(
                    SessionDir.as_mut_ptr(),
                    WIN_OBJ_DIR.as_ptr(),
                );
            } else {
                swprintf(
                    SessionDir.as_mut_ptr(),
                    b"%ws\\%ld%ws\0".as_ptr() as *const u16,
                    SESSION_DIR.as_ptr(),
                    SessionId,
                    WIN_OBJ_DIR.as_ptr(),
                );
            }

            Status = CsrClientConnectToServer(
                SessionDir.as_mut_ptr(),
                BASESRV_SERVERDLL_INDEX as ULONG,
                &mut ConnectInfo as *mut BASESRV_API_CONNECTINFO as PVOID,
                &mut ConnectInfoSize,
                &mut BaseRunningInServerProcess,
            );

            if !NT_SUCCESS(Status) {
                DbgPrint(
                    b"[kernel32] CsrClientConnectToServer FAILED Status=0x%lx PID=%lu\n\0".as_ptr() as *const i8,
                    Status,
                    (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG,
                );
                NtTerminateProcess(NtCurrentProcess(), Status);
                return FALSE;
            }

            if (*Peb).ReadOnlyStaticServerData.is_null() {
                RtlAssert(
                    b"Peb->ReadOnlyStaticServerData\0".as_ptr() as *const i8 as PVOID,
                    b"dllmain.c\0".as_ptr() as *const i8 as PVOID,
                    186u32,
                    core::ptr::null_mut(),
                );
            }

            BaseStaticServerData = *(*Peb)
                .ReadOnlyStaticServerData
                .offset(BASESRV_SERVERDLL_INDEX as isize)
                as PBASE_STATIC_SERVER_DATA;

            if BaseStaticServerData.is_null() {
                RtlAssert(
                    b"BaseStaticServerData\0".as_ptr() as *const i8 as PVOID,
                    b"dllmain.c\0".as_ptr() as *const i8 as PVOID,
                    188u32,
                    core::ptr::null_mut(),
                );
            }

            if BaseRunningInServerProcess == 0 {
                CsrNewThread();
            }

            BaseDllInitializeMemoryManager();

            hCurrentModule = hDll as HMODULE;
            kernel32_handle = hCurrentModule;

            BaseWindowsDirectory = (*BaseStaticServerData).WindowsDirectory;
            BaseWindowsSystemDirectory = (*BaseStaticServerData).WindowsSystemDirectory;

            Status = RtlStringCbPrintfW(
                BaseDefaultPathBuffer.as_mut_ptr(),
                core::mem::size_of::<[WCHAR; 6140]>(),
                b"%wZ;%wZ\\system;%wZ;\0".as_ptr() as *const u16,
                &mut BaseWindowsSystemDirectory,
                &mut BaseWindowsDirectory,
                &mut BaseWindowsDirectory,
            );

            if !NT_SUCCESS(Status) {
                DbgPrint(
                    b"(%s:%d) NLS Init failed\n\0".as_ptr() as *const i8,
                    b"dllmain.c\0".as_ptr() as *const i8,
                    218i32,
                );
                return FALSE;
            }

            BaseDefaultPath.Buffer = BaseDefaultPathBuffer.as_mut_ptr();
            BaseDefaultPath.Length = (wcslen(BaseDefaultPathBuffer.as_mut_ptr()) as USHORT as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())
                as USHORT;
            BaseDefaultPath.MaximumLength = core::mem::size_of::<[WCHAR; 6140]>() as USHORT;

            BaseDefaultPathAppend.Buffer = (BaseDefaultPathBuffer.as_mut_ptr() as usize)
                .wrapping_add(BaseDefaultPath.Length as usize)
                as PWSTR;
            BaseDefaultPathAppend.Length = 0u16;
            BaseDefaultPathAppend.MaximumLength = (BaseDefaultPath.MaximumLength as i32
                - BaseDefaultPath.Length as i32)
                as USHORT;

            InitCommandLines();

            RtlInitializeCriticalSection(&mut BaseDllDirectoryLock);

            if NlsInit() == 0 {
                DbgPrint(
                    b"(%s:%d) NLS Init failed\n\0".as_ptr() as *const i8,
                    b"dllmain.c\0".as_ptr() as *const i8,
                    240i32,
                );
                return FALSE;
            }

            if ConDllInitialize(dwReason, SessionDir.as_mut_ptr() as PVOID) == 0 {
                DbgPrint(
                    b"(%s:%d) Failed to set up console\n\0".as_ptr() as *const i8,
                    b"dllmain.c\0".as_ptr() as *const i8,
                    247i32,
                );
                return FALSE;
            }

            InitializeListHead(&mut BasepAppCertDllsList);
            RtlInitializeCriticalSection(&mut gcsAppCert);

            DllInitialized = TRUE as BOOL;

            DbgPrint(
                b"[kernel32] DllMain PROCESS_ATTACH done OK PID=%lu\n\0".as_ptr() as *const i8,
                (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG,
            );
        }

        DLL_PROCESS_DETACH => {
            if DllInitialized != FALSE {
                ConDllInitialize(dwReason, core::ptr::null_mut());
                NlsUninit();
                RtlDeleteCriticalSection(&mut BaseDllDirectoryLock);
            }
        }

        DLL_THREAD_ATTACH => {
            return ConDllInitialize(dwReason, core::ptr::null_mut()) as BOOL;
        }

        _ => {}
    }

    return TRUE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/dllmain.c
//  * PURPOSE:         Initialization
//  * PROGRAMMERS:     Ariadne (ariadne@xs4all.nl)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// PBASE_STATIC_SERVER_DATA BaseStaticServerData;
// #ifdef _M_ARM64
// static BASE_STATIC_SERVER_DATA BaseStaticServerDataFallback;
// BOOLEAN BaseCsrConnectionEstablished = FALSE;
// #endif
// BOOLEAN BaseRunningInServerProcess = FALSE;
//
// WCHAR BaseDefaultPathBuffer[6140];
//
// HANDLE BaseNamedObjectDirectory;
// HMODULE hCurrentModule = NULL;
// HMODULE kernel32_handle = NULL;
// PPEB Peb;
// ULONG SessionId;
// static BOOL DllInitialized = FALSE;
//
// /* Critical section for various kernel32 data structures */
// RTL_CRITICAL_SECTION BaseDllDirectoryLock;
//
// extern BOOL FASTCALL NlsInit(VOID);
// extern VOID FASTCALL NlsUninit(VOID);
//
// #define WIN_OBJ_DIR L"\\Windows"
// #define SESSION_DIR L"\\Sessions"
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// BaseCreateThreadPoolThread(IN PTHREAD_START_ROUTINE Function,
//                            IN PVOID Parameter,
//                            OUT PHANDLE ThreadHandle)
// {
//     NTSTATUS Status;
//
//     /* Create a Win32 thread */
//     *ThreadHandle = CreateRemoteThread(NtCurrentProcess(),
//                                        NULL,
//                                        0,
//                                        Function,
//                                        Parameter,
//                                        CREATE_SUSPENDED,
//                                        NULL);
//     if (!(*ThreadHandle))
//     {
//         /* Get the status value if we couldn't get a handle */
//         Status = NtCurrentTeb()->LastStatusValue;
//         if (NT_SUCCESS(Status)) Status = STATUS_UNSUCCESSFUL;
//     }
//     else
//     {
//         /* Set success code */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* All done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// BaseExitThreadPoolThread(IN NTSTATUS ExitStatus)
// {
//     /* Exit the thread */
//     ExitThread(ExitStatus);
//     return STATUS_SUCCESS;
// }
//
// BOOL
// WINAPI
// DllMain(HANDLE hDll,
//         DWORD dwReason,
//         LPVOID lpReserved)
// {
//     NTSTATUS Status;
//     BASESRV_API_CONNECTINFO ConnectInfo;
//     ULONG ConnectInfoSize = sizeof(ConnectInfo);
//     WCHAR SessionDir[256];
//
//     DPRINT("DllMain(hInst %p, dwReason %lu)\n",
//            hDll, dwReason);
//
//     Basep8BitStringToUnicodeString = RtlAnsiStringToUnicodeString;
//
//     /* Cache the PEB and Session ID */
//     Peb = NtCurrentPeb();
//     SessionId = Peb->SessionId;
//
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//         {
//             DbgPrint("[kernel32] DllMain PROCESS_ATTACH PID=%lu\n", (ULONG)(ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess);
//             /* Set no filter initially */
//             GlobalTopLevelExceptionFilter = RtlEncodePointer(NULL);
//
//             /* Enable the Rtl thread pool and timer queue to use proper Win32 thread */
//             RtlSetThreadPoolStartFunc(BaseCreateThreadPoolThread, BaseExitThreadPoolThread);
//
//             /* Register the manifest prober routine */
//             LdrSetDllManifestProber(BasepProbeForDllManifest);
//
//             /* Don't bother us for each thread */
//             LdrDisableThreadCalloutsForDll((PVOID)hDll);
//
//             /* Initialize default path to NULL */
//             RtlInitUnicodeString(&BaseDefaultPath, NULL);
//
//             /* Setup the Object Directory path */
//             if (!SessionId)
//             {
//                 /* Use the raw path */
//                 wcscpy(SessionDir, WIN_OBJ_DIR);
//             }
//             else
//             {
//                 /* Use the session path */
//                 swprintf(SessionDir,
//                          L"%ws\\%ld%ws",
//                          SESSION_DIR,
//                          SessionId,
//                          WIN_OBJ_DIR);
//             }
//
//             /* Connect to the Base Server */
//             Status = CsrClientConnectToServer(SessionDir,
//                                               BASESRV_SERVERDLL_INDEX,
//                                               &ConnectInfo,
//                                               &ConnectInfoSize,
//                                               &BaseRunningInServerProcess);
//             if (!NT_SUCCESS(Status))
//             {
//                 DbgPrint("[kernel32] CsrClientConnectToServer FAILED Status=0x%lx PID=%lu\n", Status, (ULONG)(ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess);
// #ifdef _M_ARM64
//                 DPRINT1("ARM64: Continuing without CSRSS connection\n");
// #else
//                 NtTerminateProcess(NtCurrentProcess(), Status);
//                 return FALSE;
// #endif
//             }
// #ifdef _M_ARM64
//             else
//             {
//                 BaseCsrConnectionEstablished = TRUE;
//             }
// #endif
//
//             /* Get the server data */
// #ifdef _M_ARM64
//             if (Peb->ReadOnlyStaticServerData)
//             {
//                 BaseStaticServerData = Peb->ReadOnlyStaticServerData[BASESRV_SERVERDLL_INDEX];
//             }
//             if (!BaseStaticServerData)
//             {
//                 DPRINT1("ARM64: ReadOnlyStaticServerData not available, using fallback\n");
//                 RtlZeroMemory(&BaseStaticServerDataFallback, sizeof(BaseStaticServerDataFallback));
//                 NtQuerySystemInformation(SystemBasicInformation,
//                                          &BaseStaticServerDataFallback.SysInfo,
//                                          sizeof(SYSTEM_BASIC_INFORMATION),
//                                          NULL);
//                 RtlInitUnicodeString(&BaseStaticServerDataFallback.WindowsDirectory, L"\\reactos");
//                 RtlInitUnicodeString(&BaseStaticServerDataFallback.WindowsSystemDirectory, L"\\reactos\\system32");
//                 BaseStaticServerDataFallback.TermsrvClientTimeZoneId = TIME_ZONE_ID_INVALID;
//                 BaseStaticServerData = &BaseStaticServerDataFallback;
//             }
// #else
//             ASSERT(Peb->ReadOnlyStaticServerData);
//             BaseStaticServerData = Peb->ReadOnlyStaticServerData[BASESRV_SERVERDLL_INDEX];
//             ASSERT(BaseStaticServerData);
// #endif
//
//             /* Check if we are running a CSR Server */
//             if (!BaseRunningInServerProcess)
//             {
//                 /* Set the termination port for the thread */
//                 DPRINT("Creating new thread for CSR\n");
//                 CsrNewThread();
//             }
//
//             /* Initialize heap handle table */
//             BaseDllInitializeMemoryManager();
//
//             /* Set HMODULE for our DLL */
//             kernel32_handle = hCurrentModule = hDll;
//
//             /* Set the directories */
//             BaseWindowsDirectory = BaseStaticServerData->WindowsDirectory;
//             BaseWindowsSystemDirectory = BaseStaticServerData->WindowsSystemDirectory;
//
//             /* Construct the default path (using the static buffer) */
//             Status = RtlStringCbPrintfW(BaseDefaultPathBuffer,
//                                      sizeof(BaseDefaultPathBuffer),
//                                      L"%wZ;%wZ\\system;%wZ;",
//                                      &BaseWindowsSystemDirectory,
//                                      &BaseWindowsDirectory,
//                                      &BaseWindowsDirectory);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("NLS Init failed\n");
//                 return FALSE;
//             }
//
//             BaseDefaultPath.Buffer = BaseDefaultPathBuffer;
//             BaseDefaultPath.Length = (USHORT)wcslen(BaseDefaultPathBuffer) * sizeof(WCHAR);
//             BaseDefaultPath.MaximumLength = sizeof(BaseDefaultPathBuffer);
//
//             /* Use remaining part of the default path buffer for the append path */
//             BaseDefaultPathAppend.Buffer = (PWSTR)((ULONG_PTR)BaseDefaultPathBuffer + BaseDefaultPath.Length);
//             BaseDefaultPathAppend.Length = 0;
//             BaseDefaultPathAppend.MaximumLength = BaseDefaultPath.MaximumLength - BaseDefaultPath.Length;
//
//             /* Initialize command line */
//             InitCommandLines();
//
//             /* Initialize the DLL critical section */
//             RtlInitializeCriticalSection(&BaseDllDirectoryLock);
//
//             /* Initialize the National Language Support routines */
//             if (!NlsInit())
//             {
//                 DPRINT1("NLS Init failed\n");
//                 return FALSE;
//             }
//
//             /* Initialize Console Support */
//             if (!ConDllInitialize(dwReason, SessionDir))
//             {
//                 DPRINT1("Failed to set up console\n");
//                 return FALSE;
//             }
//
//             /* Initialize application certification globals */
//             InitializeListHead(&BasepAppCertDllsList);
//             RtlInitializeCriticalSection(&gcsAppCert);
//
//             /* Insert more dll attach stuff here! */
//             DllInitialized = TRUE;
//             DbgPrint("[kernel32] DllMain PROCESS_ATTACH done OK PID=%lu\n", (ULONG)(ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess);
//             break;
//         }
//
//         case DLL_PROCESS_DETACH:
//         {
//             if (DllInitialized != FALSE)
//             {
//                 /* Uninitialize console support */
//                 ConDllInitialize(dwReason, NULL);
//
//                 /* Insert more dll detach stuff here! */
//                 NlsUninit();
//
//                 /* Delete DLL critical section */
//                 RtlDeleteCriticalSection(&BaseDllDirectoryLock);
//             }
//             break;
//         }
//
//         case DLL_THREAD_ATTACH:
//         {
//             /* ConDllInitialize sets the current console locale for the new thread */
//             return ConDllInitialize(dwReason, NULL);
//         }
//
//         default:
//             break;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
