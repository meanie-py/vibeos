//! services.c — the SCM main controller (wWinMain entry + bootstrap sequence).
//! The 8th and final services.exe file flipped to Rust (after lock/controlset/
//! security/groupdb/driver/config/database/rpcserver). Faithful translation.
//!
//! Owns the 4 cross-module SCM state flags (ScmInitialize/ScmShutdown/
//! ScmLiveSetup/ScmSetupInProgress) that database.rs + rpcserver.rs read via
//! `extern static mut` — defined here as `#[no_mangle] pub static mut`. Every
//! bootstrap step (ScmInitializeSecurity / ScmCreateServiceDatabase /
//! ScmAutoStartServices / …) is a cdecl `#[no_mangle]` fn in a sibling module,
//! reached here via a local `extern "C"` block. The `goto done` cleanup cascade
//! becomes a labeled block (`'done`). The [SERVICES] DbgPrint trace lines are
//! kept verbatim (load-bearing boot instrumentation); the plain DPRINT/DPRINT1
//! diagnostics are dropped (as in the other flipped files).

use crate::types::{LPSC_RPC_LOCK, SC_RPC_LOCK};
use crate::wstr;
use core::mem::size_of;
use core::ptr::{null, null_mut};
use winsrv_sys::winsrv::*;

use crate::types::HKEY_LOCAL_MACHINE;

const SCM_AUTOSTARTCOMPLETE_EVENT: &[u16] = wstr!("SC_AutoStartComplete");

// services.c globals. The 4 BOOL flags are read cross-module (database.rs /
// rpcserver.rs declare them `extern static mut`) → exported #[no_mangle].
// BOOL is i32; FALSE is u32 in the bindgen glob, so use the typed `0` literal.
#[no_mangle]
pub static mut ScmInitialize: BOOL = 0;
#[no_mangle]
pub static mut ScmShutdown: BOOL = 0;
#[no_mangle]
pub static mut ScmLiveSetup: BOOL = 0;
#[no_mangle]
pub static mut ScmSetupInProgress: BOOL = 0;

// services.c file-static handles (not exported).
static mut hScmShutdownEvent: HANDLE = null_mut();
static mut hScmSecurityServicesEvent: HANDLE = null_mut();

// --- externs: SCM bootstrap helpers (sibling modules, all cdecl) ------------
extern "C" {
    fn ScmInitializeSecurity() -> DWORD; // security.rs
    fn ScmShutdownSecurity(); // security.rs
    fn ScmCreateLastKnownGoodControlSet() -> DWORD; // controlset.rs
    fn ScmCreateServiceDatabase() -> DWORD; // database.rs
    fn ScmShutdownServiceDatabase(); // database.rs
    fn ScmGetBootAndSystemDriverState(); // database.rs
    fn ScmAutoStartServices(); // database.rs
    fn ScmAutoShutdownServices(); // database.rs
    fn ScmInitNamedPipeCriticalSection(); // database.rs
    fn ScmDeleteNamedPipeCriticalSection(); // database.rs
    fn ScmStartRpcServer(); // rpcserver.rs
    fn ScmAcquireServiceStartLock(IsServiceController: BOOL, lpLock: LPSC_RPC_LOCK) -> DWORD; // lock.rs
    fn ScmReleaseServiceStartLock(lpLock: LPSC_RPC_LOCK) -> DWORD;
    // CRT cdecl, not in the winsrv glob.
    fn wcsstr(haystack: *const u16, needle: *const u16) -> *mut u16;
}

// RegisterServicesProcess is user32.dll `@ stdcall` (WINAPI) → extern "system".
// ReportEventW / DeregisterEventSource are advapi32 stdcall, absent from the glob.
extern "system" {
    fn RegisterServicesProcess(ServicesProcessId: DWORD) -> i32;
    fn ReportEventW(
        hEventLog: HANDLE,
        wType: WORD,
        wCategory: WORD,
        dwEventID: DWORD,
        lpUserSid: PVOID,
        wNumStrings: WORD,
        dwDataSize: DWORD,
        lpStrings: *const LPCWSTR,
        lpRawData: PVOID,
    ) -> BOOL;
    fn DeregisterEventSource(hEventLog: HANDLE) -> BOOL;
}

// --- FUNCTIONS --------------------------------------------------------------

/// `VOID PrintString(LPCSTR fmt, ...)` — DBG-only OutputDebugString helper with
/// no callers anywhere in the tree. Kept as a no-op stub to preserve the
/// exported symbol (cdecl; a varargs caller, of which there are none, cleans its
/// own stack). The variadic body is elided rather than pulling in vsprintf.
#[no_mangle]
pub unsafe extern "C" fn PrintString(_fmt: *const i8) {}

unsafe fn CheckForLiveCD() -> DWORD {
    let mut CommandLine: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hSetupKey: HKEY = null_mut();
    let mut dwSetupType: DWORD = 0;
    let mut dwSetupInProgress: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwError: DWORD;

    /* Open the Setup key */
    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("SYSTEM\\Setup").as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hSetupKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    'done: {
        /* Read the SetupType value */
        dwSize = size_of::<DWORD>() as DWORD;
        dwError = RegQueryValueExW(
            hSetupKey,
            wstr!("SetupType").as_ptr(),
            null_mut(),
            &mut dwType,
            &mut dwSetupType as *mut DWORD as *mut u8,
            &mut dwSize,
        ) as DWORD;

        if dwError != ERROR_SUCCESS
            || dwType != REG_DWORD
            || dwSize != size_of::<DWORD>() as DWORD
            || dwSetupType == 0
        {
            break 'done;
        }

        /* Read the CmdLine value */
        dwSize = size_of::<[u16; MAX_PATH as usize]>() as DWORD;
        dwError = RegQueryValueExW(
            hSetupKey,
            wstr!("CmdLine").as_ptr(),
            null_mut(),
            &mut dwType,
            CommandLine.as_mut_ptr() as *mut u8,
            &mut dwSize,
        ) as DWORD;

        if dwError != ERROR_SUCCESS
            || (dwType != REG_SZ && dwType != REG_EXPAND_SZ && dwType != REG_MULTI_SZ)
        {
            break 'done;
        }

        /* Check for the '-mini' option */
        if !wcsstr(CommandLine.as_ptr(), wstr!(" -mini").as_ptr()).is_null() {
            ScmLiveSetup = 1; // TRUE
        }

        /* Read the SystemSetupInProgress value */
        dwSize = size_of::<DWORD>() as DWORD;
        dwError = RegQueryValueExW(
            hSetupKey,
            wstr!("SystemSetupInProgress").as_ptr(),
            null_mut(),
            &mut dwType,
            &mut dwSetupInProgress as *mut DWORD as *mut u8,
            &mut dwSize,
        ) as DWORD;
        // NOTE: the `dwSetupType == 0` here mirrors the C verbatim (it reads
        // dwSetupType, not dwSetupInProgress — preserved faithfully).
        if dwError != ERROR_SUCCESS
            || dwType != REG_DWORD
            || dwSize != size_of::<DWORD>() as DWORD
            || dwSetupType == 0
        {
            break 'done;
        }

        if dwSetupInProgress == 1 {
            ScmSetupInProgress = 1; // TRUE
        }
    }

    // done:
    RegCloseKey(hSetupKey);

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn SetSecurityServicesEvent() -> DWORD {
    let dwError: DWORD;

    if !hScmSecurityServicesEvent.is_null() {
        return ERROR_SUCCESS;
    }

    /* Create or open the SECURITY_SERVICES_STARTED event */
    hScmSecurityServicesEvent = CreateEventW(
        null_mut(),
        1, // TRUE (manual reset)
        0, // FALSE (initial state)
        wstr!("SECURITY_SERVICES_STARTED").as_ptr(),
    );
    if hScmSecurityServicesEvent.is_null() {
        dwError = GetLastError();
        if dwError != ERROR_ALREADY_EXISTS {
            return dwError;
        }

        hScmSecurityServicesEvent = OpenEventW(
            EVENT_MODIFY_STATE,
            0, // FALSE
            wstr!("SECURITY_SERVICES_STARTED").as_ptr(),
        );
        if hScmSecurityServicesEvent.is_null() {
            return GetLastError();
        }
    }

    SetEvent(hScmSecurityServicesEvent);

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ScmLogEvent(
    dwEventId: DWORD,
    wType: WORD,
    wStrings: WORD,
    lpStrings: *mut LPCWSTR,
) {
    /* Skip event logging during initialization — the event log service isn't
     * running yet, and RegisterEventSourceW uses RPC which throws an exception
     * on failure. With DUMMY_PSEH the RPC SEH handler is dead code, so the
     * exception would crash the process. */
    if ScmInitialize != 0 {
        return;
    }

    let hLog: HANDLE = RegisterEventSourceW(null(), wstr!("Service Control Manager").as_ptr());
    if hLog.is_null() {
        return;
    }

    ReportEventW(
        hLog,
        wType,
        0,
        dwEventId,
        null_mut(),
        wStrings,
        0,
        lpStrings as *const LPCWSTR,
        null_mut(),
    );

    DeregisterEventSource(hLog);
}

#[no_mangle]
pub unsafe extern "C" fn ScmWaitForLsa() {
    let hEvent: HANDLE = CreateEventW(null_mut(), 1, 0, wstr!("LSA_RPC_SERVER_ACTIVE").as_ptr());
    if !hEvent.is_null() {
        WaitForSingleObject(hEvent, INFINITE);
        CloseHandle(hEvent);
    }
}

// `BOOL WINAPI ShutdownHandlerRoutine(DWORD)` is a PHANDLER_ROUTINE passed by
// value to SetConsoleCtrlHandler. That fn-ptr type is `extern "stdcall"` on
// i386 but `extern "C"` on amd64/arm64 — Rust treats them as distinct fn-item
// types, so split into a shared impl + two cfg-gated trampolines (same shape as
// rpcserver.rs's ScmStopThread).
unsafe fn ShutdownHandlerRoutine_impl(dwCtrlType: DWORD) -> BOOL {
    if dwCtrlType & (CTRL_SHUTDOWN_EVENT | CTRL_LOGOFF_EVENT) != 0 {
        ScmShutdown = 1; // TRUE

        ScmAutoShutdownServices();
        ScmShutdownServiceDatabase();

        /* Set the shutdown event */
        SetEvent(hScmShutdownEvent);
    }

    1 // TRUE
}

unsafe extern "system" fn ShutdownHandlerRoutine(dwCtrlType: DWORD) -> BOOL {
    ShutdownHandlerRoutine_impl(dwCtrlType)
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    _hInstance: HINSTANCE,
    _hPrevInstance: HINSTANCE,
    _lpCmdLine: LPWSTR,
    _nShowCmd: i32,
) -> i32 {
    let mut hScmStartEvent: HANDLE = null_mut();
    let mut hScmAutoStartCompleteEvent: HANDLE = null_mut();
    let mut Lock: SC_RPC_LOCK = null_mut();
    let mut bCanDeleteNamedPipeCriticalSection: BOOL = 0; // FALSE
    let mut dwError: DWORD;

    DbgPrint(
        b"[SERVICES] wWinMain entered PID=%lu\n\0".as_ptr() as *const i8,
        GetCurrentProcessId(),
    );

    'done: {
        dwError = CheckForLiveCD();
        if dwError != ERROR_SUCCESS {
            DbgPrint(
                b"[SERVICES] CheckForLiveCD failed: %lu\n\0".as_ptr() as *const i8,
                dwError,
            );
            break 'done;
        }
        DbgPrint(b"[SERVICES] CheckForLiveCD OK\n\0".as_ptr() as *const i8);

        /* Make us critical */
        RtlSetProcessIsCritical(1, null_mut(), 1);

        /* We are initializing ourselves */
        ScmInitialize = 1; // TRUE

        /* Create the start event */
        hScmStartEvent = CreateEventW(null_mut(), 1, 0, wstr!("SvcctrlStartEvent_A3752DX").as_ptr());
        if hScmStartEvent.is_null() {
            DbgPrint(
                b"[SERVICES] CreateEvent(SCM_START) failed: %lu\n\0".as_ptr() as *const i8,
                GetLastError(),
            );
            break 'done;
        }

        /* Create the auto-start complete event */
        hScmAutoStartCompleteEvent =
            CreateEventW(null_mut(), 1, 0, SCM_AUTOSTARTCOMPLETE_EVENT.as_ptr());
        if hScmAutoStartCompleteEvent.is_null() {
            DbgPrint(
                b"[SERVICES] CreateEvent(AUTO_START) failed: %lu\n\0".as_ptr() as *const i8,
                GetLastError(),
            );
            break 'done;
        }

        /* Create the shutdown event */
        hScmShutdownEvent = CreateEventW(null_mut(), 1, 0, null());
        if hScmShutdownEvent.is_null() {
            DbgPrint(
                b"[SERVICES] CreateEvent(shutdown) failed: %lu\n\0".as_ptr() as *const i8,
                GetLastError(),
            );
            break 'done;
        }
        DbgPrint(b"[SERVICES] Events created OK\n\0".as_ptr() as *const i8);

        /* Initialize our communication named pipe's critical section */
        ScmInitNamedPipeCriticalSection();
        bCanDeleteNamedPipeCriticalSection = 1; // TRUE

        //    ScmInitThreadManager();

        ScmInitializeSecurity();
        DbgPrint(b"[SERVICES] Security initialized\n\0".as_ptr() as *const i8);

        /* FIXME: more initialization */

        /* Create the 'Last Known Good' control set */
        dwError = ScmCreateLastKnownGoodControlSet();
        if dwError != ERROR_SUCCESS {
            DbgPrint(
                b"[SERVICES] CreateLastKnownGood failed: %lu\n\0".as_ptr() as *const i8,
                dwError,
            );
            break 'done;
        }
        DbgPrint(b"[SERVICES] LastKnownGood OK\n\0".as_ptr() as *const i8);

        /* Create the services database */
        dwError = ScmCreateServiceDatabase();
        if dwError != ERROR_SUCCESS {
            DbgPrint(
                b"[SERVICES] CreateServiceDatabase failed: %lu\n\0".as_ptr() as *const i8,
                dwError,
            );
            break 'done;
        }
        DbgPrint(b"[SERVICES] ServiceDatabase OK\n\0".as_ptr() as *const i8);

        /* Update the services database */
        ScmGetBootAndSystemDriverState();

        /* Register the Service Control Manager process with the ReactOS Subsystem */
        if RegisterServicesProcess(GetCurrentProcessId()) == 0 {
            DbgPrint(b"[SERVICES] RegisterServicesProcess failed\n\0".as_ptr() as *const i8);
            break 'done;
        }
        DbgPrint(b"[SERVICES] RegisterServicesProcess OK\n\0".as_ptr() as *const i8);

        /*
         * Acquire the user service start lock until
         * auto-start services have been started.
         */
        dwError = ScmAcquireServiceStartLock(1, &mut Lock);
        if dwError != ERROR_SUCCESS {
            DbgPrint(
                b"[SERVICES] AcquireServiceStartLock failed: %lu\n\0".as_ptr() as *const i8,
                dwError,
            );
            break 'done;
        }

        /* Start the RPC server */
        ScmStartRpcServer();

        /* Signal start event */
        SetEvent(hScmStartEvent);

        DbgPrint(b"[SERVICES] Initialized, starting auto-start services\n\0".as_ptr() as *const i8);

        /* Register event handler (used for system shutdown) */
        SetConsoleCtrlHandler(Some(ShutdownHandlerRoutine), 1);

        /*
         * Set our shutdown parameters: we want to shutdown after the maintained
         * services (that inherit the default shutdown level of 640).
         */
        SetProcessShutdownParameters(480, SHUTDOWN_NORETRY);

        /* Start auto-start services */
        ScmAutoStartServices();

        /* Signal auto-start complete event */
        SetEvent(hScmAutoStartCompleteEvent);

        /* FIXME: more to do ? */

        /* Release the service start lock */
        ScmReleaseServiceStartLock(&mut Lock);

        /* Initialization finished */
        ScmInitialize = 0; // FALSE

        DbgPrint(b"[SERVICES] Running, waiting for shutdown\n\0".as_ptr() as *const i8);

        /* Wait until the shutdown event gets signaled */
        WaitForSingleObject(hScmShutdownEvent, INFINITE);
    }

    // done:
    DbgPrint(b"[SERVICES] Exiting (reached done label)\n\0".as_ptr() as *const i8);
    ScmShutdownSecurity();

    /* Delete our communication named pipe's critical section */
    if bCanDeleteNamedPipeCriticalSection != 0 {
        ScmDeleteNamedPipeCriticalSection();
    }

    if !hScmSecurityServicesEvent.is_null() {
        CloseHandle(hScmSecurityServicesEvent);
    }

    /* Close the shutdown event */
    if !hScmShutdownEvent.is_null() {
        CloseHandle(hScmShutdownEvent);
    }

    /* Close the auto-start complete event */
    if !hScmAutoStartCompleteEvent.is_null() {
        CloseHandle(hScmAutoStartCompleteEvent);
    }

    /* Close the start event */
    if !hScmStartEvent.is_null() {
        CloseHandle(hScmStartEvent);
    }

    DbgPrint(b"[SERVICES] Finished, calling ExitThread(0)\n\0".as_ptr() as *const i8);

    ExitThread(0);
    0
}
