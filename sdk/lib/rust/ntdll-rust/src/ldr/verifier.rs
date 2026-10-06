//! Rust translation of `dll/ntdll/ldr/verifier.c`.
//!
//! Application Verifier support — provider DLL loading, IAT thunk snapping,
//! and the Ldr/heap notification callbacks. Almost all of it is gated behind
//! `FLG_APPLICATION_VERIFIER` / page-heap flags, so on a normal boot every
//! entry point early-returns (audit Tier 1, "optional path").
//!
//! Translation notes (vs. the C / the raw rustconv output):
//!   * The `RTL_VERIFIER_*` descriptor types (reactos/verifier.h) are NOT in
//!     the ntdll-sys bindgen surface — declared locally here with the exact
//!     header layout. The descriptor's `Rtlp*` slots are `PVOID` (the C stores
//!     function addresses into them), not typed fn pointers.
//!   * Verifier flag/debug constants are the real evntrace/verifier.h values —
//!     the raw output invented sequential 0x1/0x2/0x4… values.
//!   * `LdrpImageEntry` is an exported *data* symbol (PLDR_DATA_TABLE_ENTRY),
//!     not a function; `SharedUserData` is the `USER_SHARED_DATA` macro.
//!     `NtCurrentTeb`/`NtCurrentPeb` are the ntdll-sys inlines.
//!   * `IMAGE_THUNK_DATA`'s union field is `u1`; TEB's `ClientId` is a direct
//!     field (not an anon union).
//!   * The two `_SEH2_TRY/_SEH2_EXCEPT` blocks run unprotected under DUMMY_PSEH
//!     (the handler is dead code) — rendered as plain blocks.

// This file mirrors verifier.c's manipulation of file-scope mutable globals
// (the provider LIST_ENTRY + lock), so reads/writes through `static mut` are
// expected and intentional.
#![allow(static_mut_refs)]

use core::mem::{offset_of, size_of, size_of_val, zeroed};
use core::ptr::{addr_of, addr_of_mut, null_mut};
use ntdll_sys::nt::*;

extern "system" {
    fn RtlpPageHeapCreate(
        Flags: ULONG,
        Addr: PVOID,
        TotalSize: SIZE_T,
        CommitSize: SIZE_T,
        Lock: PVOID,
        Parameters: PRTL_HEAP_PARAMETERS,
    ) -> HANDLE;
    fn RtlpPageHeapDestroy(HeapPtr: HANDLE) -> PVOID;
    fn LdrpGetProcedureAddress(
        BaseAddress: PVOID,
        Name: *const ANSI_STRING,
        Ordinal: ULONG,
        ProcedureAddress: *mut PVOID,
        ExecuteFlag: BOOLEAN,
    ) -> NTSTATUS;
    fn LdrpFetchAddressOfEntryPoint(BaseAddress: PVOID) -> PVOID;
    fn LdrpCallInitRoutine(
        EntryPoint: PVOID,
        BaseAddress: PVOID,
        Reason: ULONG,
        Context: PVOID,
    ) -> BOOLEAN;
    // Exported *data* symbols (defined in ldrinit.c / the rtl heap), not fns.
    static mut LdrpImageEntry: PLDR_DATA_TABLE_ENTRY;
    static mut RtlpPageHeapEnabled: BOOLEAN;
    static mut RtlpDphGlobalFlags: ULONG;
}

// CRT compares (cdecl) — exported by ntdll, not in the nt FFI surface.
extern "C" {
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn _stricmp(s1: *const i8, s2: *const i8) -> i32;
}

// ntstatus.h — not in the ntdll-sys constants table.
const STATUS_DLL_INIT_FAILED: NTSTATUS = 0xC000_0142u32 as i32;

/// `rtl_get_process_heap()` (FORCEINLINE macro): `NtCurrentPeb()->ProcessHeap`.
#[inline(always)]
unsafe fn rtl_get_process_heap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

/// `SharedUserData` macro: `(KUSER_SHARED_DATA*)USER_SHARED_DATA`.
#[inline(always)]
unsafe fn shared_user_data() -> *const _KUSER_SHARED_DATA {
    USER_SHARED_DATA as usize as *const _KUSER_SHARED_DATA
}

// --- ntdll-private globals (verifier.c file-scope) -------------------
// AVrfpVerifier{Flags,DllsString,Debug} have external linkage in C; keep
// #[no_mangle] so any other ntdll TU could still resolve them.
#[no_mangle]
pub static mut AVrfpVerifierFlags: ULONG = 0;
#[no_mangle]
pub static mut AVrfpVerifierDllsString: [u16; 256] = [0; 256];
#[no_mangle]
pub static mut AVrfpDebug: ULONG = 0;
static mut AVrfpInitialized: BOOLEAN = 0;
// Zero-initialized like the C BSS; AVrfInitializeVerifier sets them up at
// runtime (RtlInitializeCriticalSection / InitializeListHead) before any
// notification can walk the list (all entry points gate on the verifier flag).
static mut AVrfpVerifierLock: RTL_CRITICAL_SECTION = unsafe { zeroed() };
static mut AVrfpVerifierProvidersList: LIST_ENTRY = unsafe { zeroed() };

const VERIFIER_DLL_FLAGS_RESOLVED: ULONG = 1;
const DLL_PROCESS_VERIFIER: ULONG = 4;

// reactos/verifier.h — real values (NOT the sequential ones rustconv emits).
const RTL_VRF_FLG_HANDLE_CHECKS: ULONG = 0x0000_0004;
const RTL_VRF_FLG_FAST_FILL_HEAP: ULONG = 0x0000_8000;
const RTL_VRF_FLG_LOCK_CHECKS: ULONG = 0x0004_0000;
const RTL_VRF_DBG_SHOWSNAPS: ULONG = 0x0000_0001;
const RTL_VRF_DBG_SHOWFOUNDEXPORTS: ULONG = 0x0000_0002;
const RTL_VRF_DBG_SHOWVERIFIEDEXPORTS: ULONG = 0x0000_0004;
const RTL_VRF_DBG_LISTPROVIDERS: ULONG = 0x0000_0008;
const RTL_VRF_DBG_SHOWCHAINING: ULONG = 0x0000_0010;
const RTL_VRF_DBG_SHOWCHAINING_DEBUG: ULONG = 0x0000_0020;
// ntdll/include/ntdllp.h
const DPH_FLAG_DLL_NOTIFY: ULONG = 0x40;

// --- reactos/verifier.h types (absent from ntdll-sys bindgen) --------
type RTL_VERIFIER_DLL_LOAD_CALLBACK =
    Option<unsafe extern "system" fn(*mut u16, PVOID, SIZE_T, PVOID)>;
type RTL_VERIFIER_DLL_UNLOAD_CALLBACK =
    Option<unsafe extern "system" fn(*mut u16, PVOID, SIZE_T, PVOID)>;
type RTL_VERIFIER_NTDLLHEAPFREE_CALLBACK =
    Option<unsafe extern "system" fn(PVOID, SIZE_T)>;

#[repr(C)]
struct RTL_VERIFIER_THUNK_DESCRIPTOR {
    ThunkName: *mut i8,   // PCHAR
    ThunkOldAddress: PVOID,
    ThunkNewAddress: PVOID,
}

#[repr(C)]
struct RTL_VERIFIER_DLL_DESCRIPTOR {
    DllName: *mut u16, // PWCHAR
    DllFlags: ULONG,   // DWORD
    DllAddress: PVOID,
    DllThunks: *mut RTL_VERIFIER_THUNK_DESCRIPTOR,
}

#[repr(C)]
struct RTL_VERIFIER_PROVIDER_DESCRIPTOR {
    Length: ULONG, // DWORD
    ProviderDlls: *mut RTL_VERIFIER_DLL_DESCRIPTOR,
    ProviderDllLoadCallback: RTL_VERIFIER_DLL_LOAD_CALLBACK,
    ProviderDllUnloadCallback: RTL_VERIFIER_DLL_UNLOAD_CALLBACK,
    VerifierImage: *mut u16, // PWSTR
    VerifierFlags: ULONG,
    VerifierDebug: ULONG,
    RtlpGetStackTraceAddress: PVOID,
    RtlpDebugPageHeapCreate: PVOID,
    RtlpDebugPageHeapDestroy: PVOID,
    ProviderNtdllHeapFreeCallback: RTL_VERIFIER_NTDLLHEAPFREE_CALLBACK,
}

// verifier.c's file-local VERIFIER_PROVIDER.
#[repr(C)]
struct VERIFIER_PROVIDER {
    ListEntry: LIST_ENTRY,
    DllName: _UNICODE_STRING,
    BaseAddress: PVOID,
    EntryPoint: PVOID,
    ProviderDlls: *mut RTL_VERIFIER_DLL_DESCRIPTOR,
    ProviderDllLoadCallback: RTL_VERIFIER_DLL_LOAD_CALLBACK,
    ProviderDllUnloadCallback: RTL_VERIFIER_DLL_UNLOAD_CALLBACK,
    ProviderNtdllHeapFreeCallback: RTL_VERIFIER_NTDLLHEAPFREE_CALLBACK,
}

#[no_mangle]
pub unsafe extern "system" fn AVrfReadIFEO(KeyHandle: HANDLE) {
    static VERIFIER_DLLS: [u16; 13] = [
        b'V' as u16, b'e' as u16, b'r' as u16, b'i' as u16, b'f' as u16, b'i' as u16, b'e' as u16,
        b'r' as u16, b'D' as u16, b'l' as u16, b'l' as u16, b's' as u16, 0,
    ];
    static VERIFIER_FLAGS: [u16; 14] = [
        b'V' as u16, b'e' as u16, b'r' as u16, b'i' as u16, b'f' as u16, b'i' as u16, b'e' as u16,
        b'r' as u16, b'F' as u16, b'l' as u16, b'a' as u16, b'g' as u16, b's' as u16, 0,
    ];
    static VERIFIER_DEBUG: [u16; 14] = [
        b'V' as u16, b'e' as u16, b'r' as u16, b'i' as u16, b'f' as u16, b'i' as u16, b'e' as u16,
        b'r' as u16, b'D' as u16, b'e' as u16, b'b' as u16, b'u' as u16, b'g' as u16, 0,
    ];

    let mut status = LdrQueryImageFileKeyOption(
        KeyHandle,
        VERIFIER_DLLS.as_ptr(),
        REG_SZ,
        addr_of_mut!(AVrfpVerifierDllsString) as PVOID,
        (size_of_val(&AVrfpVerifierDllsString) - size_of::<u16>()) as ULONG,
        null_mut(),
    );
    if status < 0 {
        AVrfpVerifierDllsString[0] = 0;
    }

    status = LdrQueryImageFileKeyOption(
        KeyHandle,
        VERIFIER_FLAGS.as_ptr(),
        REG_DWORD,
        addr_of_mut!(AVrfpVerifierFlags) as PVOID,
        size_of::<ULONG>() as ULONG,
        null_mut(),
    );
    if status < 0 {
        AVrfpVerifierFlags =
            RTL_VRF_FLG_HANDLE_CHECKS | RTL_VRF_FLG_FAST_FILL_HEAP | RTL_VRF_FLG_LOCK_CHECKS;
    }

    status = LdrQueryImageFileKeyOption(
        KeyHandle,
        VERIFIER_DEBUG.as_ptr(),
        REG_DWORD,
        addr_of_mut!(AVrfpDebug) as PVOID,
        size_of::<ULONG>() as ULONG,
        null_mut(),
    );
    if status < 0 {
        AVrfpDebug = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn LdrpInitializeApplicationVerifierPackage(
    KeyHandle: HANDLE,
    Peb: PPEB,
    _SystemWide: BOOLEAN,
    ReadAdvancedOptions: BOOLEAN,
) -> NTSTATUS {
    if (*Peb).NtGlobalFlag & FLG_HEAP_PAGE_ALLOCS != 0 {
        if ReadAdvancedOptions != 0 {
            // TODO: Read advanced DPH flags from the registry (UNIMPLEMENTED).
        }
        RtlpPageHeapEnabled = 1;
    }

    AVrfReadIFEO(KeyHandle);

    STATUS_SUCCESS
}

unsafe fn AVrfpIsVerifierProviderDll(BaseAddress: PVOID) -> BOOLEAN {
    let mut entry = AVrfpVerifierProvidersList.Flink;
    while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
        let provider =
            ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry)) as *mut VERIFIER_PROVIDER;
        if BaseAddress == (*provider).BaseAddress {
            return 1;
        }
        entry = (*entry).Flink;
    }
    0
}

unsafe fn AVrfpCountThunks(thunk: PIMAGE_THUNK_DATA) -> SIZE_T {
    let mut count: usize = 0;
    while (*thunk.add(count)).u1.Function != 0 {
        count += 1;
    }
    count as SIZE_T
}

unsafe fn AVrfpSnapDllImports(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    let mut size: ULONG = 0;
    let dll_base = (*LdrEntry).DllBase as *mut u8;

    let mut import_descriptor = RtlImageDirectoryEntryToData(
        dll_base as PVOID,
        1,
        IMAGE_DIRECTORY_ENTRY_IMPORT as u16,
        addr_of_mut!(size),
    ) as *mut _IMAGE_IMPORT_DESCRIPTOR;

    if import_descriptor.is_null() {
        return;
    }

    while (*import_descriptor).Name != 0
        && (*import_descriptor).__bindgen_anon_1.OriginalFirstThunk != 0
    {
        let mut first_thunk =
            dll_base.add((*import_descriptor).FirstThunk as usize) as PIMAGE_THUNK_DATA;
        let mut unprotected_ptr: PVOID = null_mut();
        let mut unprotected_size: SIZE_T = 0;
        let mut old_protection: ULONG = 0;

        while (*first_thunk).u1.Function != 0 {
            let mut entry = AVrfpVerifierProvidersList.Flink;
            while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
                let provider = ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry))
                    as *mut VERIFIER_PROVIDER;
                let mut dll_descriptor = (*provider).ProviderDlls;

                while !dll_descriptor.is_null() && !(*dll_descriptor).DllName.is_null() {
                    let mut thunk_descriptor = (*dll_descriptor).DllThunks;
                    while !thunk_descriptor.is_null() && !(*thunk_descriptor).ThunkName.is_null() {
                        if (*first_thunk).u1.Function as PVOID != (*thunk_descriptor).ThunkOldAddress
                        {
                            thunk_descriptor = thunk_descriptor.add(1);
                            continue;
                        }

                        if unprotected_ptr.is_null() {
                            let mut ptr = addr_of_mut!((*first_thunk).u1.Function) as PVOID;
                            let mut sz: SIZE_T =
                                size_of::<SIZE_T>() as SIZE_T * AVrfpCountThunks(first_thunk);
                            unprotected_ptr = ptr;
                            unprotected_size = sz;

                            let status = NtProtectVirtualMemory(
                                NtCurrentProcess(),
                                addr_of_mut!(ptr),
                                addr_of_mut!(sz),
                                PAGE_EXECUTE_READWRITE,
                                addr_of_mut!(old_protection),
                            );
                            if status < 0 {
                                DbgPrint(
                                    b"AVRF: Unable to unprotect IAT to modify thunks (status %08X).\n\0".as_ptr() as *const i8,
                                    status,
                                );
                                unprotected_ptr = null_mut();
                                thunk_descriptor = thunk_descriptor.add(1);
                                continue;
                            }
                        }

                        if (*thunk_descriptor).ThunkNewAddress.is_null() {
                            DbgPrint(
                                b"AVRF: internal error: New thunk for %s is null.\n\0".as_ptr()
                                    as *const i8,
                                (*thunk_descriptor).ThunkName,
                            );
                            thunk_descriptor = thunk_descriptor.add(1);
                            continue;
                        }

                        (*first_thunk).u1.Function = (*thunk_descriptor).ThunkNewAddress as SIZE_T;
                        if AVrfpDebug & RTL_VRF_DBG_SHOWSNAPS != 0 {
                            DbgPrint(
                                b"AVRF: Snapped (%wZ: %s) with (%wZ: %p).\n\0".as_ptr() as *const i8,
                                addr_of!((*LdrEntry).BaseDllName),
                                (*thunk_descriptor).ThunkName,
                                addr_of!((*provider).DllName),
                                (*thunk_descriptor).ThunkNewAddress,
                            );
                        }
                        thunk_descriptor = thunk_descriptor.add(1);
                    }
                    dll_descriptor = dll_descriptor.add(1);
                }
                entry = (*entry).Flink;
            }
            first_thunk = first_thunk.add(1);
        }

        if !unprotected_ptr.is_null() {
            let mut ptr = unprotected_ptr;
            let mut sz = unprotected_size;
            let status = NtProtectVirtualMemory(
                NtCurrentProcess(),
                addr_of_mut!(ptr),
                addr_of_mut!(sz),
                old_protection,
                addr_of_mut!(old_protection),
            );
            if status < 0 {
                DbgPrint(
                    b"AVRF: Unable to reprotect IAT to modify thunks (status %08X).\n\0".as_ptr()
                        as *const i8,
                    status,
                );
            }
        }

        import_descriptor = import_descriptor.add(1);
    }
}

unsafe fn AvrfpResolveThunks(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    if AVrfpInitialized == 0 {
        return;
    }

    let mut entry = AVrfpVerifierProvidersList.Flink;
    while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
        let provider =
            ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry)) as *mut VERIFIER_PROVIDER;
        let mut dll_descriptor = (*provider).ProviderDlls;

        while !dll_descriptor.is_null() && !(*dll_descriptor).DllName.is_null() {
            if ((*dll_descriptor).DllFlags & VERIFIER_DLL_FLAGS_RESOLVED) != 0
                || _wcsicmp((*dll_descriptor).DllName, (*LdrEntry).BaseDllName.Buffer) != 0
            {
                dll_descriptor = dll_descriptor.add(1);
                continue;
            }

            if AVrfpDebug & RTL_VRF_DBG_SHOWVERIFIEDEXPORTS != 0 {
                DbgPrint(
                    b"AVRF: pid 0x%X: found dll descriptor for `%wZ' with verified exports\n\0"
                        .as_ptr() as *const i8,
                    (*NtCurrentTeb()).ClientId.UniqueProcess,
                    addr_of!((*LdrEntry).BaseDllName),
                );
            }

            let mut thunk_descriptor = (*dll_descriptor).DllThunks;
            while !thunk_descriptor.is_null() && !(*thunk_descriptor).ThunkName.is_null() {
                if (*thunk_descriptor).ThunkOldAddress.is_null() {
                    let mut thunk_name: ANSI_STRING = zeroed();
                    RtlInitAnsiString(addr_of_mut!(thunk_name), (*thunk_descriptor).ThunkName);
                    // Cannot call the public api — that would run init routines!
                    if LdrpGetProcedureAddress(
                        (*LdrEntry).DllBase,
                        addr_of!(thunk_name),
                        0,
                        addr_of_mut!((*thunk_descriptor).ThunkOldAddress),
                        0,
                    ) >= 0
                    {
                        if AVrfpDebug & RTL_VRF_DBG_SHOWFOUNDEXPORTS != 0 {
                            DbgPrint(
                                b"AVRF: (%wZ) %Z export found.\n\0".as_ptr() as *const i8,
                                addr_of!((*LdrEntry).BaseDllName),
                                addr_of!(thunk_name),
                            );
                        }
                    } else if AVrfpDebug & RTL_VRF_DBG_SHOWFOUNDEXPORTS != 0 {
                        DbgPrint(
                            b"AVRF: warning: did not find `%Z' export in %wZ.\n\0".as_ptr()
                                as *const i8,
                            addr_of!(thunk_name),
                            addr_of!((*LdrEntry).BaseDllName),
                        );
                    }
                }
                thunk_descriptor = thunk_descriptor.add(1);
            }

            (*dll_descriptor).DllFlags |= VERIFIER_DLL_FLAGS_RESOLVED;
            dll_descriptor = dll_descriptor.add(1);
        }

        entry = (*entry).Flink;
    }

    AVrfpSnapDllImports(LdrEntry);
}

#[no_mangle]
pub unsafe extern "system" fn AVrfDllLoadNotification(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    if ((*NtCurrentPeb()).NtGlobalFlag & FLG_APPLICATION_VERIFIER) == 0 {
        return;
    }

    RtlEnterCriticalSection(addr_of_mut!(AVrfpVerifierLock));
    if AVrfpIsVerifierProviderDll((*LdrEntry).DllBase) == 0 {
        AvrfpResolveThunks(LdrEntry);

        let mut entry = AVrfpVerifierProvidersList.Flink;
        while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
            let provider = ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry))
                as *mut VERIFIER_PROVIDER;
            if let Some(callback) = (*provider).ProviderDllLoadCallback {
                callback(
                    (*LdrEntry).BaseDllName.Buffer,
                    (*LdrEntry).DllBase,
                    (*LdrEntry).SizeOfImage as SIZE_T,
                    LdrEntry as PVOID,
                );
            }
            entry = (*entry).Flink;
        }
    }
    RtlLeaveCriticalSection(addr_of_mut!(AVrfpVerifierLock));
}

#[no_mangle]
pub unsafe extern "system" fn AVrfDllUnloadNotification(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    if ((*NtCurrentPeb()).NtGlobalFlag & FLG_APPLICATION_VERIFIER) == 0 || AVrfpInitialized == 0 {
        return;
    }

    RtlEnterCriticalSection(addr_of_mut!(AVrfpVerifierLock));
    if AVrfpIsVerifierProviderDll((*LdrEntry).DllBase) == 0 {
        let mut entry = AVrfpVerifierProvidersList.Flink;
        while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
            let provider = ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry))
                as *mut VERIFIER_PROVIDER;
            if let Some(callback) = (*provider).ProviderDllUnloadCallback {
                callback(
                    (*LdrEntry).BaseDllName.Buffer,
                    (*LdrEntry).DllBase,
                    (*LdrEntry).SizeOfImage as SIZE_T,
                    LdrEntry as PVOID,
                );
            }
            entry = (*entry).Flink;
        }
    }
    RtlLeaveCriticalSection(addr_of_mut!(AVrfpVerifierLock));
}

#[no_mangle]
pub unsafe extern "system" fn AVrfInternalHeapFreeNotification(
    AllocationBase: PVOID,
    AllocationSize: SIZE_T,
) {
    if ((*NtCurrentPeb()).NtGlobalFlag & FLG_APPLICATION_VERIFIER) == 0 || AVrfpInitialized == 0 {
        return;
    }

    RtlEnterCriticalSection(addr_of_mut!(AVrfpVerifierLock));
    let mut entry = AVrfpVerifierProvidersList.Flink;
    while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
        let provider =
            ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry)) as *mut VERIFIER_PROVIDER;
        if let Some(callback) = (*provider).ProviderNtdllHeapFreeCallback {
            callback(AllocationBase, AllocationSize);
        }
        entry = (*entry).Flink;
    }
    RtlLeaveCriticalSection(addr_of_mut!(AVrfpVerifierLock));
}

#[no_mangle]
pub unsafe extern "system" fn AVrfPageHeapDllNotification(_LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    /* Check if page heap dll notification is turned on */
    if (RtlpDphGlobalFlags & DPH_FLAG_DLL_NOTIFY) == 0 {
        return;
    }
    // We don't support this flag currently (UNIMPLEMENTED).
}

unsafe fn AVrfpResnapInitialModules() {
    let list_head = addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InLoadOrderModuleList);
    let mut list_entry = (*list_head).Flink;

    while list_head != list_entry {
        let ldr_entry = ((list_entry as usize)
            - offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks))
            as PLDR_DATA_TABLE_ENTRY;

        if AVrfpIsVerifierProviderDll((*ldr_entry).DllBase) != 0 {
            if AVrfpDebug & RTL_VRF_DBG_SHOWSNAPS != 0 {
                DbgPrint(
                    b"AVRF: skipped resnapping provider %wZ ...\n\0".as_ptr() as *const i8,
                    addr_of!((*ldr_entry).BaseDllName),
                );
            }
        } else {
            if AVrfpDebug & RTL_VRF_DBG_SHOWSNAPS != 0 {
                DbgPrint(
                    b"AVRF: resnapping %wZ ...\n\0".as_ptr() as *const i8,
                    addr_of!((*ldr_entry).BaseDllName),
                );
            }
            AvrfpResolveThunks(ldr_entry);
        }

        list_entry = (*list_entry).Flink;
    }
}

unsafe fn AvrfpFindDuplicateThunk(
    EndEntry: PLIST_ENTRY,
    DllName: *mut u16,
    ThunkName: *const i8,
) -> PVOID {
    let mut entry = AVrfpVerifierProvidersList.Flink;
    while entry != EndEntry {
        let provider =
            ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry)) as *mut VERIFIER_PROVIDER;

        if AVrfpDebug & RTL_VRF_DBG_SHOWCHAINING_DEBUG != 0 {
            DbgPrint(
                b"AVRF: chain: searching in %wZ\n\0".as_ptr() as *const i8,
                addr_of!((*provider).DllName),
            );
        }

        let mut dll_descriptor = (*provider).ProviderDlls;
        while !dll_descriptor.is_null() && !(*dll_descriptor).DllName.is_null() {
            if AVrfpDebug & RTL_VRF_DBG_SHOWCHAINING_DEBUG != 0 {
                DbgPrint(
                    b"AVRF: chain: dll: %ws\n\0".as_ptr() as *const i8,
                    (*dll_descriptor).DllName,
                );
            }

            if _wcsicmp((*dll_descriptor).DllName, DllName) != 0 {
                dll_descriptor = dll_descriptor.add(1);
                continue;
            }

            let mut thunk_descriptor = (*dll_descriptor).DllThunks;
            while !thunk_descriptor.is_null() && !(*thunk_descriptor).ThunkName.is_null() {
                if AVrfpDebug & RTL_VRF_DBG_SHOWCHAINING_DEBUG != 0 {
                    DbgPrint(
                        b"AVRF: chain: thunk: %s == %s ?\n\0".as_ptr() as *const i8,
                        (*thunk_descriptor).ThunkName,
                        ThunkName,
                    );
                }

                if _stricmp((*thunk_descriptor).ThunkName, ThunkName) == 0 {
                    if AVrfpDebug & RTL_VRF_DBG_SHOWCHAINING_DEBUG != 0 {
                        DbgPrint(
                            b"AVRF: Found duplicate for (%ws: %s) in %wZ\n\0".as_ptr() as *const i8,
                            (*dll_descriptor).DllName,
                            (*thunk_descriptor).ThunkName,
                            addr_of!((*provider).DllName),
                        );
                    }
                    return (*thunk_descriptor).ThunkNewAddress;
                }
                thunk_descriptor = thunk_descriptor.add(1);
            }
            dll_descriptor = dll_descriptor.add(1);
        }
        entry = (*entry).Flink;
    }
    null_mut()
}

unsafe fn AVrfpChainDuplicateThunks() {
    let mut entry = AVrfpVerifierProvidersList.Flink;
    while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
        let provider =
            ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry)) as *mut VERIFIER_PROVIDER;
        let mut dll_descriptor = (*provider).ProviderDlls;

        while !dll_descriptor.is_null() && !(*dll_descriptor).DllName.is_null() {
            let mut thunk_descriptor = (*dll_descriptor).DllThunks;
            while !thunk_descriptor.is_null() && !(*thunk_descriptor).ThunkName.is_null() {
                if AVrfpDebug & RTL_VRF_DBG_SHOWCHAINING_DEBUG != 0 {
                    DbgPrint(
                        b"AVRF: Checking %wZ for duplicate (%ws: %s)\n\0".as_ptr() as *const i8,
                        addr_of!((*provider).DllName),
                        (*dll_descriptor).DllName,
                        (*thunk_descriptor).ThunkName,
                    );
                }

                let ptr = AvrfpFindDuplicateThunk(
                    entry,
                    (*dll_descriptor).DllName,
                    (*thunk_descriptor).ThunkName,
                );
                if !ptr.is_null() {
                    if AVrfpDebug & RTL_VRF_DBG_SHOWCHAINING != 0 {
                        DbgPrint(
                            b"AVRF: Chaining (%ws: %s) to %wZ\n\0".as_ptr() as *const i8,
                            (*dll_descriptor).DllName,
                            (*thunk_descriptor).ThunkName,
                            addr_of!((*provider).DllName),
                        );
                    }
                    (*thunk_descriptor).ThunkOldAddress = ptr;
                }
                thunk_descriptor = thunk_descriptor.add(1);
            }
            dll_descriptor = dll_descriptor.add(1);
        }
        entry = (*entry).Flink;
    }
}

unsafe extern "system" fn AVrfpGetStackTraceAddress(_Arg0: ULONG) -> PVOID {
    // UNIMPLEMENTED
    DbgBreakPoint();
    null_mut()
}

unsafe extern "system" fn AVrfpDebugPageHeapCreate(
    Flags: ULONG,
    Addr: PVOID,
    TotalSize: SIZE_T,
    CommitSize: SIZE_T,
    Lock: PVOID,
    Parameters: PRTL_HEAP_PARAMETERS,
) -> HANDLE {
    let h_heap = RtlpPageHeapCreate(Flags, Addr, TotalSize, CommitSize, Lock, Parameters);
    DbgPrint(
        b"AVRF: DebugPageHeapCreate(Flags=%x, Addr=%p, TotalSize=%u, CommitSize=%u, Lock=%p, Parameters=%p) = %p\n\0".as_ptr() as *const i8,
        Flags, Addr, TotalSize, CommitSize, Lock, Parameters, h_heap,
    );
    h_heap
}

// C: `static PVOID AVrfpDebugPageHeapDestroy(HANDLE)` — no NTAPI ⇒ cdecl.
unsafe extern "C" fn AVrfpDebugPageHeapDestroy(HeapPtr: HANDLE) -> PVOID {
    DbgPrint(
        b"AVRF: DebugPageHeapDestroy(HeapPtr=%p)\n\0".as_ptr() as *const i8,
        HeapPtr,
    );
    RtlpPageHeapDestroy(HeapPtr)
}

// File-local in practice (only AVrfInitializeVerifier calls it); kept private
// so the private VERIFIER_PROVIDER type doesn't leak through a pub signature.
unsafe fn AVrfpLoadAndInitializeProvider(Provider: *mut VERIFIER_PROVIDER) -> NTSTATUS {
    static SYSTEM32: [u16; 11] = [
        b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        b'3' as u16, b'2' as u16, b'\\' as u16, 0,
    ];

    let mut string_buffer: [u16; MAX_PATH as usize + 11] = [0; MAX_PATH as usize + 11];
    let mut dll_path: _UNICODE_STRING = zeroed();
    let mut descriptor: *mut RTL_VERIFIER_PROVIDER_DESCRIPTOR = null_mut();

    // RtlInitEmptyUnicodeString(&dll_path, StringBuffer, sizeof(StringBuffer))
    // — a FORCEINLINE macro (no exported symbol); inlined.
    dll_path.Buffer = string_buffer.as_mut_ptr();
    dll_path.Length = 0;
    dll_path.MaximumLength = size_of_val(&string_buffer) as u16;
    RtlAppendUnicodeToString(addr_of_mut!(dll_path), (*shared_user_data()).NtSystemRoot.as_ptr());
    RtlAppendUnicodeToString(addr_of_mut!(dll_path), SYSTEM32.as_ptr());

    if AVrfpDebug & RTL_VRF_DBG_SHOWSNAPS != 0 {
        DbgPrint(
            b"AVRF: verifier dll `%wZ'\n\0".as_ptr() as *const i8,
            addr_of!((*Provider).DllName),
        );
    }

    let mut status = LdrLoadDll(
        dll_path.Buffer,
        null_mut(),
        addr_of_mut!((*Provider).DllName),
        addr_of_mut!((*Provider).BaseAddress),
    );
    if status < 0 {
        DbgPrint(
            b"AVRF: %wZ: failed to load provider `%wZ' (status %08X) from %wZ\n\0".as_ptr()
                as *const i8,
            addr_of!((*LdrpImageEntry).BaseDllName),
            addr_of!((*Provider).DllName),
            status,
            addr_of!(dll_path),
        );
        return status;
    }

    /* Prevent someone funny from specifying his own application as provider */
    let image_nt_header = RtlImageNtHeader((*Provider).BaseAddress);
    if image_nt_header.is_null()
        || ((*image_nt_header).FileHeader.Characteristics & IMAGE_FILE_DLL as u16) == 0
    {
        DbgPrint(
            b"AVRF: provider %wZ is not a DLL image\n\0".as_ptr() as *const i8,
            addr_of!((*Provider).DllName),
        );
        return STATUS_DLL_INIT_FAILED;
    }

    (*Provider).EntryPoint = LdrpFetchAddressOfEntryPoint((*Provider).BaseAddress);
    if (*Provider).EntryPoint.is_null() {
        DbgPrint(
            b"AVRF: cannot find an entry point for provider %wZ\n\0".as_ptr() as *const i8,
            addr_of!((*Provider).DllName),
        );
        return STATUS_PROCEDURE_NOT_FOUND;
    }

    // _SEH2_TRY — DUMMY_PSEH: handler is dead code, body runs unprotected.
    status = {
        if LdrpCallInitRoutine(
            (*Provider).EntryPoint,
            (*Provider).BaseAddress,
            DLL_PROCESS_VERIFIER,
            addr_of_mut!(descriptor) as PVOID,
        ) != 0
        {
            if !descriptor.is_null()
                && (*descriptor).Length == size_of::<RTL_VERIFIER_PROVIDER_DESCRIPTOR>() as ULONG
            {
                /* Copy the data */
                (*Provider).ProviderDlls = (*descriptor).ProviderDlls;
                (*Provider).ProviderDllLoadCallback = (*descriptor).ProviderDllLoadCallback;
                (*Provider).ProviderDllUnloadCallback = (*descriptor).ProviderDllUnloadCallback;
                (*Provider).ProviderNtdllHeapFreeCallback =
                    (*descriptor).ProviderNtdllHeapFreeCallback;

                /* Update some info for the provider */
                (*descriptor).VerifierImage = (*LdrpImageEntry).BaseDllName.Buffer;
                (*descriptor).VerifierFlags = AVrfpVerifierFlags;
                (*descriptor).VerifierDebug = AVrfpDebug;
                (*descriptor).RtlpGetStackTraceAddress = AVrfpGetStackTraceAddress as usize as PVOID;
                (*descriptor).RtlpDebugPageHeapCreate = AVrfpDebugPageHeapCreate as usize as PVOID;
                (*descriptor).RtlpDebugPageHeapDestroy = AVrfpDebugPageHeapDestroy as usize as PVOID;
                STATUS_SUCCESS
            } else {
                DbgPrint(
                    b"AVRF: provider %wZ passed an invalid descriptor @ %p\n\0".as_ptr()
                        as *const i8,
                    addr_of!((*Provider).DllName),
                    descriptor,
                );
                STATUS_INVALID_PARAMETER_4
            }
        } else {
            DbgPrint(
                b"AVRF: provider %wZ did not initialize correctly\n\0".as_ptr() as *const i8,
                addr_of!((*Provider).DllName),
            );
            STATUS_DLL_INIT_FAILED
        }
    };

    if status < 0 {
        return status;
    }

    if AVrfpDebug & RTL_VRF_DBG_LISTPROVIDERS != 0 {
        DbgPrint(
            b"AVRF: initialized provider %wZ (descriptor @ %p)\n\0".as_ptr() as *const i8,
            addr_of!((*Provider).DllName),
            descriptor,
        );
    }

    /* Done loading providers, allow dll notifications */
    AVrfpInitialized = 1;

    AVrfpChainDuplicateThunks();
    AVrfpResnapInitialModules();

    /* Manually call with DLL_PROCESS_ATTACH (process not done initializing) */
    status = {
        if LdrpCallInitRoutine(
            (*Provider).EntryPoint,
            (*Provider).BaseAddress,
            DLL_PROCESS_ATTACH,
            null_mut(),
        ) == 0
        {
            DbgPrint(
                b"AVRF: provider %wZ did not initialize correctly\n\0".as_ptr() as *const i8,
                addr_of!((*Provider).DllName),
            );
            STATUS_DLL_INIT_FAILED
        } else {
            STATUS_SUCCESS
        }
    };

    status
}

#[no_mangle]
pub unsafe extern "system" fn AVrfInitializeVerifier() -> NTSTATUS {
    static VERIFIER_DLL: [u16; 13] = [
        b'v' as u16, b'e' as u16, b'r' as u16, b'i' as u16, b'f' as u16, b'i' as u16, b'e' as u16,
        b'r' as u16, b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16, 0,
    ];

    let mut status = RtlInitializeCriticalSection(addr_of_mut!(AVrfpVerifierLock));
    InitializeListHead(addr_of_mut!(AVrfpVerifierProvidersList));

    if status < 0 {
        return status;
    }

    DbgPrint(
        b"AVRF: %wZ: pid 0x%X: flags 0x%X: application verifier enabled\n\0".as_ptr() as *const i8,
        addr_of!((*LdrpImageEntry).BaseDllName),
        (*NtCurrentTeb()).ClientId.UniqueProcess,
        AVrfpVerifierFlags,
    );

    let mut provider = RtlAllocateHeap(
        rtl_get_process_heap(),
        HEAP_ZERO_MEMORY,
        size_of::<VERIFIER_PROVIDER>() as SIZE_T,
    ) as *mut VERIFIER_PROVIDER;
    if provider.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlInitUnicodeString(addr_of_mut!((*provider).DllName), VERIFIER_DLL.as_ptr());
    InsertTailList(
        addr_of_mut!(AVrfpVerifierProvidersList),
        addr_of_mut!((*provider).ListEntry),
    );

    let mut next = AVrfpVerifierDllsString.as_mut_ptr();
    loop {
        while *next == b' ' as u16 || *next == b'\t' as u16 {
            next = next.add(1);
        }
        let ptr = next;
        while *next != b' ' as u16 && *next != b'\t' as u16 && *next != 0 {
            next = next.add(1);
        }
        if *next != 0 {
            *next = 0;
            next = next.add(1);
        } else {
            next = null_mut();
        }

        if *ptr != 0 {
            provider = RtlAllocateHeap(
                rtl_get_process_heap(),
                HEAP_ZERO_MEMORY,
                size_of::<VERIFIER_PROVIDER>() as SIZE_T,
            ) as *mut VERIFIER_PROVIDER;
            if provider.is_null() {
                return STATUS_NO_MEMORY;
            }
            RtlInitUnicodeString(addr_of_mut!((*provider).DllName), ptr);
            InsertTailList(
                addr_of_mut!(AVrfpVerifierProvidersList),
                addr_of_mut!((*provider).ListEntry),
            );
        }

        if next.is_null() {
            break;
        }
    }

    let mut entry = AVrfpVerifierProvidersList.Flink;
    while entry != addr_of_mut!(AVrfpVerifierProvidersList) {
        provider =
            ((entry as usize) - offset_of!(VERIFIER_PROVIDER, ListEntry)) as *mut VERIFIER_PROVIDER;
        entry = (*entry).Flink;

        status = AVrfpLoadAndInitializeProvider(provider);
        if status < 0 {
            RemoveEntryList(addr_of_mut!((*provider).ListEntry));
            RtlFreeHeap(rtl_get_process_heap(), 0, provider as PVOID);
        }
    }

    if status < 0 {
        DbgPrint(
            b"AVRF: %wZ: pid 0x%X: application verifier will be disabled due to an initialization error.\n\0".as_ptr() as *const i8,
            addr_of!((*LdrpImageEntry).BaseDllName),
            (*NtCurrentTeb()).ClientId.UniqueProcess,
        );
        (*NtCurrentPeb()).NtGlobalFlag &= !FLG_APPLICATION_VERIFIER;
    }

    status
}
