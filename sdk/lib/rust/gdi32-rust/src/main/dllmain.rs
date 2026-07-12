//! win32ss/gdi/gdi32/main/dllmain.c → Rust. gdi32.dll entry point + per-process GDI setup.
//!
//! GDI32.DLL has an entry point for DisableThreadLibraryCalls(). The initialization is done
//! by GdiDllInitialize(), which is called from USER32.DLL's entry point. DllMain here only
//! sets up the critical sections early (before user32's Init() drives NtUserProcessConnect,
//! which can trigger GDI operations); GdiProcessSetup runs later, guarded by gbInitialized.
//
// NEEDS-TYPE: CRITICAL_SECTION — the win32k-sys bindgen surface has no `CRITICAL_SECTION`
//   alias; it is identical to RTL_CRITICAL_SECTION (winbase typedef). gcsClientObjLinks (a
//   sibling-owned global) and the InitializeCriticalSection/DeleteCriticalSection params are
//   typed RTL_CRITICAL_SECTION / PRTL_CRITICAL_SECTION here. If central adds a
//   crate::winbase::{CRITICAL_SECTION,LPCRITICAL_SECTION} (like user32-rust), retype to match.
// NEEDS-IMPORT: NtCurrentTeb — reproduced locally as a FORCEINLINE (TEB self-pointer read),
//   mirroring user32-rust/src/internal.rs. If gdi32-rust gains a crate::internal (or gdihelp
//   exposes it), drop the local copy and import it (it is an #[inline] non-exported fn, so the
//   duplicate produces no link symbol and is harmless meanwhile).

use crate::types::*;
// GDI struct/syscall surface + the scalar aliases not present in crate::types. Imported
// selectively (not a glob) to avoid E0659 ambiguity with the crate::types glob above — the
// overlapping aliases are transparent typedefs of the same underlying types regardless.
use win32k_sys::win32k::{
    NtGdiInit, PDEVCAPS, PGDIHANDLECACHE, PGDI_SHARED_HANDLE_TABLE, PGDI_TABLE_ENTRY, PPEB,
    PRTL_CRITICAL_SECTION, RTL_CRITICAL_SECTION, NB_STOCK_OBJECTS, NTSTATUS, PVOID, SIZE_T, ULONG,
    _TEB,
};

// winnt.h DLL_* reason codes (winternl) — not in the bindgen surface.
const DLL_PROCESS_DETACH: ULONG = 0;
const DLL_PROCESS_ATTACH: ULONG = 1;
const DLL_THREAD_ATTACH: ULONG = 2;
const DLL_THREAD_DETACH: ULONG = 3;

// ── NtCurrentTeb() — per-thread TEB self pointer (NtTib.Self). FORCEINLINE, not an export. ──
#[inline]
unsafe fn NtCurrentTeb() -> *mut _TEB {
    let teb: *mut _TEB;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags));
    #[cfg(target_arch = "x86")]
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("mov {}, x18", out(reg) teb,
        options(nostack, pure, nomem, preserves_flags));
    teb
}

// ── cross-file functions (resolve via the kernel32 / ntdll / gdi32 importlibs at link) ──
extern "system" {
    // kernel32 (WINAPI)
    fn DisableThreadLibraryCalls(hLibModule: HANDLE) -> BOOL;
    fn InitializeCriticalSection(lpCriticalSection: PRTL_CRITICAL_SECTION);
    fn DeleteCriticalSection(lpCriticalSection: PRTL_CRITICAL_SECTION);
    fn GetProcessHeap() -> HANDLE;
    // ntdll (NTAPI)
    fn RtlInitializeCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlDeleteCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlZeroMemory(Destination: PVOID, Length: SIZE_T);
    // gdi32 internal (WINAPI) — objects/utils.c
    fn GdiInitializeLanguagePack(InitParam: DWORD) -> BOOL;
}

// ── globals owned by another gdi32-rust file (misc.rs) — referenced by definition there ──
extern "C" {
    static mut GdiHandleTable: PGDI_TABLE_ENTRY;
    static mut GdiSharedHandleTable: PGDI_SHARED_HANDLE_TABLE;
    static mut CurrentProcessId: HANDLE;
    static mut GDI_BatchLimit: DWORD;
    static mut hProcessHeap: HANDLE;
    // objects/stockobj.c: `extern HGDIOBJ stock_objects[];` — only its address is taken.
    static mut stock_objects: [HGDIOBJ; NB_STOCK_OBJECTS as usize];
    // gdiobj.c: `extern CRITICAL_SECTION gcsClientObjLinks;` (see NEEDS-TYPE above).
    static mut gcsClientObjLinks: RTL_CRITICAL_SECTION;
}

// ── globals DEFINED in this file (dllmain.c) ──
// `static BOOL gbInitialized = FALSE;` — file-local (internal linkage), no exported symbol.
static mut gbInitialized: BOOL = FALSE;
#[no_mangle]
pub static mut SetStockObjects: BOOL = FALSE;
#[no_mangle]
pub static mut GdiDevCaps: PDEVCAPS = core::ptr::null_mut();
#[no_mangle]
pub static mut GdiHandleCache: PGDIHANDLECACHE = core::ptr::null_mut();
#[no_mangle]
pub static mut gbLpk: BOOL = FALSE;
#[no_mangle]
pub static mut semLocal: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };

/*
 * GDI32.DLL does have an entry point for DisableThreadLibraryCalls().
 * The initialization is done by a call to GdiDllInitialize(). This
 * call is done from the entry point of USER32.DLL.
 */
#[no_mangle]
pub unsafe extern "system" fn DllMain(hDll: HANDLE, dwReason: ULONG, pReserved: PVOID) -> BOOL {
    let _ = pReserved; // UNREFERENCED_PARAMETER

    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hDll);
            /*
             * Initialize critical sections early so they are ready before
             * user32's Init() calls NtUserProcessConnect which can trigger
             * GDI operations. GdiDllInitialize/GdiProcessSetup runs later
             * and will skip re-initialization via the gbInitialized guard.
             */
            RtlInitializeCriticalSection(core::ptr::addr_of_mut!(semLocal));
            InitializeCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));
        }
        _ => {}
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GdiProcessSetup() {
    if gbInitialized == FALSE {
        gbInitialized = TRUE;
        hProcessHeap = GetProcessHeap();

        let peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;

        /* map the gdi handle table to user space */
        GdiHandleTable = (*peb).GdiSharedHandleTable as PGDI_TABLE_ENTRY;
        GdiSharedHandleTable = (*peb).GdiSharedHandleTable as PGDI_SHARED_HANDLE_TABLE;
        GdiDevCaps = core::ptr::addr_of_mut!((*GdiSharedHandleTable).DevCaps) as PDEVCAPS;

        /* WoW64 (i386 under a 64-bit kernel): the kernel writes 24-byte wide
         * entries — flip the shared-table readers (GdiReadSharedEntry) to the
         * wide layout and find DevCaps after 0x10000 wide entries. */
        #[cfg(target_arch = "x86")]
        {
            extern "system" {
                fn NtQueryInformationProcess(
                    ProcessHandle: HANDLE,
                    ProcessInformationClass: ULONG,
                    ProcessInformation: PVOID,
                    ProcessInformationLength: ULONG,
                    ReturnLength: *mut ULONG,
                ) -> NTSTATUS;
            }
            extern "C" {
                fn DbgPrint(Format: *const u8, ...) -> ULONG;
            }
            const PROCESS_WOW64_INFORMATION: ULONG = 26;
            let mut Wow64Info: usize = 0;
            let WowSt = NtQueryInformationProcess(
                -1isize as HANDLE,
                PROCESS_WOW64_INFORMATION,
                core::ptr::addr_of_mut!(Wow64Info) as PVOID,
                core::mem::size_of::<usize>() as ULONG,
                core::ptr::null_mut(),
            );
            if WowSt >= 0 && Wow64Info != 0 {
                crate::misc::misc::g_GdiWow64 = 1;
                /* 24 = sizeof the 64-bit ENTRY; 0x10000 = GDI_HANDLE_COUNT. */
                GdiDevCaps = ((GdiHandleTable as usize) + 0x10000 * 24) as PDEVCAPS;
            }
            DbgPrint(
                b"[gdi32] GdiProcessSetup: wow64 st=%lx info=%p -> wide-entries=%d\n\0".as_ptr(),
                WowSt,
                Wow64Info as PVOID,
                crate::misc::misc::g_GdiWow64,
            );
        }

        CurrentProcessId = (*NtCurrentTeb()).ClientId.UniqueProcess;
        GDI_BatchLimit = (*peb).GdiDCAttributeList as DWORD;
        GdiHandleCache = core::ptr::addr_of_mut!((*peb).GdiHandleBuffer) as PGDIHANDLECACHE;
        RtlInitializeCriticalSection(core::ptr::addr_of_mut!(semLocal));
        InitializeCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));
        GdiInitializeLanguagePack(0);
    }
}

#[no_mangle]
pub unsafe extern "system" fn GdiProcessShutdown() {
    DeleteCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));
    RtlDeleteCriticalSection(core::ptr::addr_of_mut!(semLocal));
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiDllInitialize(
    hDll: HANDLE,
    dwReason: ULONG,
    pReserved: PVOID,
) -> BOOL {
    let _ = pReserved; // UNREFERENCED_PARAMETER
    let _ = hDll;

    match dwReason {
        DLL_PROCESS_ATTACH => {
            /* Don't bother us for each thread */
            // DisableThreadLibraryCalls(hDll);

            /* Initialize the kernel part of GDI first */
            if NtGdiInit() == FALSE {
                return FALSE;
            }

            /* Now initialize ourselves */
            GdiProcessSetup();
        }

        DLL_THREAD_ATTACH => {
            (*NtCurrentTeb()).GdiTebBatch.Offset = 0;
            (*NtCurrentTeb()).GdiBatchCount = 0;
        }

        DLL_PROCESS_DETACH => {
            /* Cleanup */
            GdiProcessShutdown();
            return TRUE;
        }

        _ => {
            return FALSE;
        }
    }

    /* Very simple, the list will fill itself as it is needed */
    if SetStockObjects == FALSE {
        // Assume ROS is dirty
        RtlZeroMemory(
            core::ptr::addr_of_mut!(stock_objects) as PVOID,
            NB_STOCK_OBJECTS as SIZE_T,
        );
        SetStockObjects = TRUE;
    }

    TRUE
}

/* EOF */
