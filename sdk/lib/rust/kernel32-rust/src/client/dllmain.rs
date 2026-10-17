use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
use crate::client::thread::ExitThread;
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
use crate::ntstrsafe::rtl_strcbprintf_w_status;

extern "system" {
    fn CreateRemoteThread( hProcess: HANDLE, lpThreadAttributes: *mut _SECURITY_ATTRIBUTES, dwStackSize: u32, lpStartAddress: PTHREAD_START_ROUTINE, lpParameter: PVOID, dwCreationFlags: u32, lpThreadId: *mut u32, ) -> HANDLE;
}

extern "C" {
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> core::ffi::c_int;
    // ntstrsafe RtlStringCbPrintfW is FORCEINLINE (no exported symbol). Call the
    // CRT _snwprintf directly (same idiom as client/except.rs) and map the result
    // to NTSTATUS via rtl_strcbprintf_w_status. Both share ReactOS's %wZ-aware
    // printf engine, so the UNICODE_STRING (%wZ) conversions format correctly.
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32;
}

extern "system" {
    fn ConDllInitialize(dwReason: DWORD, lpReserved: PVOID) -> u8;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// NOTE: globals imported by other TUs via `extern "C"` blocks (k32_externs,
// utils, path, nls, nls_subst) MUST be #[no_mangle] so the cohab flip links —
// once dllmain.c is gone, these Rust defs are the sole providers of the C symbols.
#[no_mangle]
pub static mut BaseStaticServerData: PBASE_STATIC_SERVER_DATA =
    ::core::ptr::null::<_BASE_STATIC_SERVER_DATA>() as *mut _BASE_STATIC_SERVER_DATA;

#[no_mangle]
pub static mut BaseRunningInServerProcess: BOOLEAN = FALSE as BOOLEAN;

// dllmain-internal (no external importer) — no #[no_mangle] needed.
pub static mut BaseDefaultPathBuffer: [WCHAR; 6140] = [0; 6140];

#[no_mangle]
pub static mut BaseNamedObjectDirectory: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hCurrentModule: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut kernel32_handle: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub static mut Peb: PPEB = ::core::ptr::null::<_PEB>() as *mut _PEB;

pub static mut SessionId: ULONG = 0;

static mut DllInitialized: BOOL = FALSE;

// ARM64-only: CSRSS may be unavailable during early native boot. Matches the
// C `#ifdef _M_ARM64` globals in dllmain.c (BaseCsrConnectionEstablished is
// imported by proc_create.rs:2215 under the same cfg).
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut BaseCsrConnectionEstablished: BOOLEAN = FALSE as BOOLEAN;

#[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
static mut BaseStaticServerDataFallback: ::core::mem::MaybeUninit<_BASE_STATIC_SERVER_DATA> =
    ::core::mem::MaybeUninit::uninit();

/// x86/WoW64 only: set when DllMain deliberately continued after
/// `CsrClientConnectToServer` failed (the leniency below). Anything that would
/// otherwise talk to CSR must consult this first — the port was never
/// connected, so `NtRequestWaitReplyPort` on it is rejected by the kernel with
/// STATUS_INVALID_PARAMETER rather than merely returning an error from csrss.
#[cfg(target_arch = "x86")]
pub(crate) static mut BaseWow64NoCsrConnection: bool = false;

/// x86 only: true when this 32-bit kernel32 runs in a WOW64 process (guest
/// under FEX/wow64). CSR-LPC message thunking isn't built yet, so the guest
/// gets the same no-CSRSS leniency the arm64 native bring-up uses; native
/// i386 boots keep the strict terminate-on-failure behavior.
#[cfg(target_arch = "x86")]
unsafe fn BaseRunningUnderWow64() -> bool {
    const PROCESS_WOW64_INFORMATION: u32 = 26;
    let mut Peb32Ptr: ULONG_PTR = 0;
    let Status = NtQueryInformationProcess(
        NtCurrentProcess(),
        PROCESS_WOW64_INFORMATION,
        core::ptr::addr_of_mut!(Peb32Ptr) as PVOID,
        core::mem::size_of::<ULONG_PTR>() as ULONG,
        core::ptr::null_mut(),
    );
    NT_SUCCESS(Status) && Peb32Ptr != 0
}

#[no_mangle]
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
    // ExitThread does not return (`-> !`); the C's trailing `return 0` is unreachable.
    ExitThread(ExitStatus as u32);
}

/// Toggle for the [K32DBG] PROCESS_ATTACH step markers below. Off by default;
/// flip to `true` to bisect where a specific process wedges inside kernel32
/// DllMain (the tool that localized the FEX fexprobe ConDllInitialize hang — a
/// console-subsystem app deadlocking in console attach; see
/// docs/engineering/fex-bringup-plan.md rung 2). Kept in-tree so the diagnostic
/// is one edit away, not a re-derivation.
const K32_DLLMAIN_TRACE: bool = false;

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

            RtlInitUnicodeString(&raw mut BaseDefaultPath, core::ptr::null());

            if SessionId == 0 {
                wcscpy(
                    SessionDir.as_mut_ptr(),
                    WIN_OBJ_DIR.as_ptr(),
                );
            } else {
                swprintf(
                    SessionDir.as_mut_ptr(),
                    w!("%ws\\%ld%ws\0").as_ptr() as *const u16,
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
                &raw mut BaseRunningInServerProcess,
            );

            if !NT_SUCCESS(Status) {
                DbgPrint(
                    b"[kernel32] CsrClientConnectToServer FAILED Status=0x%lx PID=%lu\n\0".as_ptr() as *const i8,
                    Status,
                    (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG,
                );
                // ARM64: continue without a CSRSS connection (native process);
                // every other arch terminates. (dllmain.c #ifdef _M_ARM64)
                #[cfg(target_arch = "aarch64")]
                {
                    DbgPrint(
                        b"ARM64: Continuing without CSRSS connection\n\0".as_ptr() as *const i8,
                    );
                }
                // x86 under WOW64: same leniency — 64-bit csrss refuses the
                // 32-bit CSR_API_CONNECTINFO until the CSR-LPC-32 thunk layer
                // exists. Native i386 keeps the strict terminate.
                #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (CSRSS-less WOW64/arm64 bring-up allowance, no C counterpart)
                {
                    if BaseRunningUnderWow64() {
                        DbgPrint(
                            b"WOW64: Continuing without CSRSS connection\n\0".as_ptr() as *const i8,
                        );
                        BaseWow64NoCsrConnection = true;
                    } else {
                        NtTerminateProcess(NtCurrentProcess(), Status);
                        return FALSE;
                    }
                }
                #[cfg(not(any(target_arch = "aarch64", target_arch = "x86")))] // nolint: cfg-vs-c (CSRSS-less WOW64/arm64 bring-up allowance, no C counterpart)
                {
                    NtTerminateProcess(NtCurrentProcess(), Status);
                    return FALSE;
                }
            } else {
                #[cfg(target_arch = "aarch64")]
                {
                    BaseCsrConnectionEstablished = TRUE as BOOLEAN;
                }
            }

            // Get the server data. On ARM64 ReadOnlyStaticServerData may be
            // absent during native boot → synthesize a minimal fallback rather
            // than asserting. (dllmain.c #ifdef _M_ARM64)
            #[cfg(not(target_arch = "aarch64"))]
            {
                // x86 under WOW64 (no CSRSS connection yet): synthesize the
                // same minimal static-server-data fallback the arm64 native
                // bring-up uses instead of asserting on the NULL PEB field.
                #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (CSRSS-less WOW64/arm64 bring-up allowance, no C counterpart)
                let Wow64NoCsr = (*Peb).ReadOnlyStaticServerData.is_null()
                    && BaseRunningUnderWow64();
                #[cfg(not(target_arch = "x86"))] // nolint: cfg-vs-c (CSRSS-less WOW64/arm64 bring-up allowance, no C counterpart)
                let Wow64NoCsr = false;

                if Wow64NoCsr {
                    #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (CSRSS-less WOW64/arm64 bring-up allowance, no C counterpart)
                    {
                        DbgPrint(
                            b"WOW64: ReadOnlyStaticServerData not available, using fallback\n\0"
                                .as_ptr() as *const i8,
                        );
                        BaseStaticServerDataFallback = ::core::mem::MaybeUninit::zeroed();
                        let fallback = (*(&raw mut BaseStaticServerDataFallback)).as_mut_ptr();
                        NtQuerySystemInformation(
                            SystemBasicInformation,
                            core::ptr::addr_of_mut!((*fallback).SysInfo) as *mut core::ffi::c_void,
                            core::mem::size_of_val(&(*fallback).SysInfo) as u32,
                            core::ptr::null_mut(),
                        );
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!((*fallback).WindowsDirectory),
                            w!("\\reactos").as_ptr(),
                        );
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!((*fallback).WindowsSystemDirectory),
                            w!("\\reactos\\syswow64").as_ptr(),
                        );
                        (*fallback).TermsrvClientTimeZoneId = TIME_ZONE_ID_INVALID as ULONG;
                        BaseStaticServerData = fallback;
                    }
                } else {
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
                }
            }
            #[cfg(target_arch = "aarch64")]
            {
                if !(*Peb).ReadOnlyStaticServerData.is_null() {
                    BaseStaticServerData = *(*Peb)
                        .ReadOnlyStaticServerData
                        .offset(BASESRV_SERVERDLL_INDEX as isize)
                        as PBASE_STATIC_SERVER_DATA;
                }
                // Corruption tripwire (arm64 flake, ~1/10 boots): the CSR shared
                // section's WindowsDirectory.Buffer reads as a KERNEL VA
                // (observed 0xFFFF8000050D4CB0, identical across hits) at
                // winlogon-init time; the %wZ in the BaseDefaultPath _snwprintf
                // below then AVs in ntdll -> winlogon dies -> BUGCHECK 0x4C.
                // kernel32 is user-mode-only, so ANY high-half Buffer here is
                // corruption, never legit. Report LOUDLY with the raw fields
                // (this print is the capture — basesrv wrote shared-heap copies,
                // so a kernel VA means the section content was overwritten), then
                // reroute to the fallback so the boot survives diagnosed instead
                // of dying undiagnosed. Root-cause hunt: task/field-manual.
                if !BaseStaticServerData.is_null() {
                    let wd = core::ptr::addr_of!((*BaseStaticServerData).WindowsDirectory);
                    let wsd = core::ptr::addr_of!((*BaseStaticServerData).WindowsSystemDirectory);
                    let bad = |u: *const _UNICODE_STRING| {
                        let b = (*u).Buffer as usize;
                        ((*u).Length != 0 && b == 0) || (b >> 63) != 0
                    };
                    if bad(wd) || bad(wsd) {
                        DbgPrint(
                            b"[kernel32] BASE_STATIC_SERVER_DATA CORRUPT: WindowsDirectory.Buffer=%p Len=%u WindowsSystemDirectory.Buffer=%p Len=%u data=%p - using fallback (arm64 flake tripwire)\n\0"
                                .as_ptr() as *const i8,
                            (*wd).Buffer,
                            (*wd).Length as u32,
                            (*wsd).Buffer,
                            (*wsd).Length as u32,
                            BaseStaticServerData,
                        );
                        BaseStaticServerData = core::ptr::null_mut();
                    }
                }
                if BaseStaticServerData.is_null() {
                    DbgPrint(
                        b"ARM64: ReadOnlyStaticServerData not available, using fallback\n\0".as_ptr()
                            as *const i8,
                    );
                    BaseStaticServerDataFallback = ::core::mem::MaybeUninit::zeroed();
                    let fallback = (*(&raw mut BaseStaticServerDataFallback)).as_mut_ptr();
                    NtQuerySystemInformation(
                        SystemBasicInformation,
                        core::ptr::addr_of_mut!((*fallback).SysInfo) as *mut core::ffi::c_void,
                        core::mem::size_of_val(&(*fallback).SysInfo) as u32,
                        core::ptr::null_mut(),
                    );
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!((*fallback).WindowsDirectory),
                        w!("\\reactos").as_ptr(),
                    );
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!((*fallback).WindowsSystemDirectory),
                        w!("\\reactos\\system32").as_ptr(),
                    );
                    (*fallback).TermsrvClientTimeZoneId = TIME_ZONE_ID_INVALID as ULONG;
                    BaseStaticServerData = fallback;
                }
            }

            if K32_DLLMAIN_TRACE { DbgPrint(b"[K32DBG] post-CSR-connect (pre-CsrNewThread)\n\0".as_ptr() as *const i8); }
            if BaseRunningInServerProcess == 0 {
                CsrNewThread();
            }
            if K32_DLLMAIN_TRACE { DbgPrint(b"[K32DBG] post-CsrNewThread\n\0".as_ptr() as *const i8); }

            BaseDllInitializeMemoryManager();
            if K32_DLLMAIN_TRACE { DbgPrint(b"[K32DBG] post-BaseDllInitializeMemoryManager\n\0".as_ptr() as *const i8); }

            hCurrentModule = hDll as HMODULE;
            kernel32_handle = hCurrentModule;

            BaseWindowsDirectory = (*BaseStaticServerData).WindowsDirectory;
            BaseWindowsSystemDirectory = (*BaseStaticServerData).WindowsSystemDirectory;

            // arm64 flake tripwire, take 2 (TOCTOU-free): the earlier check at
            // BaseStaticServerData-fetch time kept passing while the %wZ below
            // still faulted — the section content MUTATES between there and
            // here (winlogon races basesrv's population; observed transient
            // Buffers: a kernel-stack VA, and 0x73 = string DATA read as a
            // pointer half). So validate the SNAPSHOT the print actually uses:
            // a bad snapshot is reported loudly and retargeted to the fallback
            // literals; a good snapshot can no longer change under us.
            #[cfg(target_arch = "aarch64")]
            {
                let bad = |u: &_UNICODE_STRING| {
                    let b = u.Buffer as usize;
                    (u.Length != 0 && b == 0)
                        || (b >> 63) != 0
                        || (b != 0 && b < 0x10000)
                        || (b & 1) != 0
                };
                if bad(&*(&raw const BaseWindowsDirectory)) || bad(&*(&raw const BaseWindowsSystemDirectory)) {
                    DbgPrint(
                        b"[kernel32] BASE_STATIC_SERVER_DATA snapshot CORRUPT (raced basesrv?): WinDir.Buffer=%p Len=%u SysDir.Buffer=%p Len=%u data=%p - using fallback strings\n\0"
                            .as_ptr() as *const i8,
                        BaseWindowsDirectory.Buffer,
                        BaseWindowsDirectory.Length as u32,
                        BaseWindowsSystemDirectory.Buffer,
                        BaseWindowsSystemDirectory.Length as u32,
                        BaseStaticServerData,
                    );
                    RtlInitUnicodeString(&raw mut BaseWindowsDirectory, w!("\\reactos").as_ptr());
                    RtlInitUnicodeString(
                        &raw mut BaseWindowsSystemDirectory,
                        w!("\\reactos\\system32").as_ptr(),
                    );
                }
            }

            let cb_dest = core::mem::size_of::<[WCHAR; 6140]>();
            let written = _snwprintf(
                (*(&raw mut BaseDefaultPathBuffer)).as_mut_ptr(),
                (cb_dest / core::mem::size_of::<WCHAR>()).saturating_sub(1),
                w!("%wZ;%wZ\\system;%wZ;\0").as_ptr() as *const u16,
                core::ptr::addr_of!(BaseWindowsSystemDirectory),
                core::ptr::addr_of!(BaseWindowsDirectory),
                core::ptr::addr_of!(BaseWindowsDirectory),
            );
            Status = rtl_strcbprintf_w_status(
                written,
                cb_dest,
                (*(&raw mut BaseDefaultPathBuffer)).as_mut_ptr(),
            );

            if !NT_SUCCESS(Status) {
                DbgPrint(
                    b"(%s:%d) NLS Init failed\n\0".as_ptr() as *const i8,
                    b"dllmain.c\0".as_ptr() as *const i8,
                    218i32,
                );
                return FALSE;
            }

            BaseDefaultPath.Buffer = (*(&raw mut BaseDefaultPathBuffer)).as_mut_ptr();
            BaseDefaultPath.Length = (wcslen((*(&raw mut BaseDefaultPathBuffer)).as_mut_ptr()) as USHORT as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())
                as USHORT;
            BaseDefaultPath.MaximumLength = core::mem::size_of::<[WCHAR; 6140]>() as USHORT;

            BaseDefaultPathAppend.Buffer = ((*(&raw mut BaseDefaultPathBuffer)).as_mut_ptr() as usize)
                .wrapping_add(BaseDefaultPath.Length as usize)
                as PWSTR;
            BaseDefaultPathAppend.Length = 0u16;
            BaseDefaultPathAppend.MaximumLength = (BaseDefaultPath.MaximumLength as i32
                - BaseDefaultPath.Length as i32)
                as USHORT;

            InitCommandLines();
            if K32_DLLMAIN_TRACE { DbgPrint(b"[K32DBG] post-InitCommandLines\n\0".as_ptr() as *const i8); }

            RtlInitializeCriticalSection(&raw mut BaseDllDirectoryLock);

            if K32_DLLMAIN_TRACE { DbgPrint(b"[K32DBG] pre-NlsInit\n\0".as_ptr() as *const i8); }
            if NlsInit() == 0 {
                DbgPrint(
                    b"(%s:%d) NLS Init failed\n\0".as_ptr() as *const i8,
                    b"dllmain.c\0".as_ptr() as *const i8,
                    240i32,
                );
                return FALSE;
            }

            if K32_DLLMAIN_TRACE { DbgPrint(b"[K32DBG] post-NlsInit (pre-ConDllInitialize)\n\0".as_ptr() as *const i8); }
            if ConDllInitialize(dwReason, SessionDir.as_mut_ptr() as PVOID) == 0 {
                DbgPrint(
                    b"(%s:%d) Failed to set up console\n\0".as_ptr() as *const i8,
                    b"dllmain.c\0".as_ptr() as *const i8,
                    247i32,
                );
                return FALSE;
            }
            if K32_DLLMAIN_TRACE { DbgPrint(b"[K32DBG] post-ConDllInitialize\n\0".as_ptr() as *const i8); }

            InitializeListHead(&raw mut BasepAppCertDllsList);
            RtlInitializeCriticalSection(&raw mut gcsAppCert);

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
                RtlDeleteCriticalSection(&raw mut BaseDllDirectoryLock);
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
