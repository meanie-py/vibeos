//! Rust translation of `dll/ntdll/nt_0600/ldr/ldrnotify.c`.
//!
//! DLL load/unload notification registry — the Vista+ public API
//! `LdrRegisterDllNotification` / `LdrUnregisterDllNotification` plus the
//! loader-internal `LdrpSendDllNotifications`. A process-global linked list of
//! registered callbacks, guarded by a critical section.
//!
//! UNGATED (compiled into BOTH the default `ntdll.dll` and the `vista`
//! `ntdll_vista.dll` builds), mirroring `nt_0600::ldr::ldrinit` (`LdrpHeap`).
//! The C lived in `ntdll_vista_static`, which is statically linked into BOTH
//! DLLs, so each kept its own registry copy. Both audiences need symbols here:
//!   * `LdrpSendDllNotifications` is called by the MAIN ntdll.dll loader
//!     (ldrutils.rs on load, ldrapi.rs on unload) — needed in the default build.
//!   * `LdrRegisterDllNotification` / `LdrUnregisterDllNotification` are
//!     exported by `ntdll_vista.spec` (and by ntdll.spec only at 0x600+, above
//!     our 0x502 matrix) — needed in the vista build.
//!
//! Translation notes (vs. the C):
//!   * The C static-initializes the critical section (`LockCount = -1`) and the
//!     list head (`RTL_STATIC_LIST_HEAD`). Rust statics can't express the
//!     mutually self-referential CS/CS_DEBUG pointers nor the self-referential
//!     list head as const initializers, so both are zero-init and lazily
//!     `RtlInitializeCriticalSection` + `InitializeListHead` on first use,
//!     guarded by a once-flag. Mirrors the verifier.rs idiom (zeroed static +
//!     runtime init); see memory feedback_rustconv_selfref_static.
//!   * `LdrpSendDllNotifications`'s `_SEH2_TRY/_SEH2_FINALLY` (which only
//!     ensures `RtlLeaveCriticalSection` runs) is dead under DUMMY_PSEH and
//!     panic=abort never unwinds, so the body runs then unconditionally Leaves.
//!   * `LDR_DLL_NOTIFICATION_ENTRY` is file-local in the C — declared here too.

// This file mirrors ldrnotify.c's file-scope mutable globals (the registry
// LIST_ENTRY + lock), so reads/writes through `static mut` are intentional.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use core::mem::{offset_of, size_of, zeroed};
use core::ptr::{addr_of, addr_of_mut};
use core::sync::atomic::{AtomicU32, Ordering};
use ntdll_sys::nt::*;

use super::ldrinit::LdrpHeap;

// ntstatus.h cast-macro absent from the bindgen surface (verified vs
// ntstatus.h). STATUS_SUCCESS / STATUS_INVALID_PARAMETER / STATUS_NO_MEMORY
// come from ntdll_sys::nt (constants.rs).
const STATUS_DLL_NOT_FOUND: NTSTATUS = 0xC000_0135u32 as NTSTATUS;

#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---------------------------------------------------------------------------
// GLOBALS (C: file-scope statics)
// ---------------------------------------------------------------------------

/// `LDR_DLL_NOTIFICATION_ENTRY` — file-local in ldrnotify.c. `List` MUST be the
/// first field: the registration cookie is the entry address, which the C also
/// treats as the embedded `List` pointer (offset 0) during unregister compares.
#[repr(C)]
struct LDR_DLL_NOTIFICATION_ENTRY {
    List: LIST_ENTRY,
    Callback: PLDR_DLL_NOTIFICATION_FUNCTION,
    Context: *mut c_void,
}

// C: `static RTL_STATIC_LIST_HEAD(LdrpDllNotificationList);` — zero-init here,
// lazily InitializeListHead'd (the self-referential {&head,&head} can't be a
// Rust const initializer).
static mut LdrpDllNotificationList: LIST_ENTRY = unsafe { zeroed() };

// C: statically-initialized critical section (LockCount = -1, self-ref
// DebugInfo). Zero-init here, lazily RtlInitializeCriticalSection'd.
static mut LdrpDllNotificationLock: RTL_CRITICAL_SECTION = unsafe { zeroed() };

// Once-guard for the lazy init: 0 = uninit, 1 = initializing, 2 = ready.
static NOTIF_INIT: AtomicU32 = AtomicU32::new(0);

/// Lazily initialize the notification lock + list head exactly once, replacing
/// the C file's static initializers. Safe under concurrent first calls.
#[inline]
unsafe fn LdrpEnsureNotificationInit() {
    loop {
        match NOTIF_INIT.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {
                RtlInitializeCriticalSection(addr_of_mut!(LdrpDllNotificationLock));
                InitializeListHead(addr_of_mut!(LdrpDllNotificationList));
                NOTIF_INIT.store(2, Ordering::Release);
                return;
            }
            Err(2) => return,
            // Another thread is mid-init (1): spin until it publishes 2.
            Err(_) => core::hint::spin_loop(),
        }
    }
}

// ---------------------------------------------------------------------------
// FUNCTIONS
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn LdrRegisterDllNotification(
    Flags: ULONG,
    NotificationFunction: PLDR_DLL_NOTIFICATION_FUNCTION,
    Context: *mut c_void,
    Cookie: *mut *mut c_void,
) -> NTSTATUS {
    // Check input parameters.
    if Flags != 0 || NotificationFunction.is_none() || Cookie.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    LdrpEnsureNotificationInit();

    // Allocate new entry and assign input values.
    let NewEntry = RtlAllocateHeap(
        LdrpHeap,
        0,
        size_of::<LDR_DLL_NOTIFICATION_ENTRY>() as SIZE_T,
    ) as *mut LDR_DLL_NOTIFICATION_ENTRY;
    if NewEntry.is_null() {
        return STATUS_NO_MEMORY;
    }
    (*NewEntry).Callback = NotificationFunction;
    (*NewEntry).Context = Context;

    // Add node to the end of the global list.
    RtlEnterCriticalSection(addr_of_mut!(LdrpDllNotificationLock));
    InsertTailList(
        addr_of_mut!(LdrpDllNotificationList),
        addr_of_mut!((*NewEntry).List),
    );
    RtlLeaveCriticalSection(addr_of_mut!(LdrpDllNotificationLock));

    // Cookie is the address of the new entry. write_volatile on the output
    // pointer (captures(none) guard — see lint_kernel.py / write_volatile rule).
    core::ptr::write_volatile(Cookie, NewEntry as *mut c_void);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LdrUnregisterDllNotification(Cookie: *mut c_void) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_DLL_NOT_FOUND;

    LdrpEnsureNotificationInit();

    // Find entry to remove.
    let head = addr_of_mut!(LdrpDllNotificationList);
    RtlEnterCriticalSection(addr_of_mut!(LdrpDllNotificationLock));
    let mut Entry = (*head).Flink;
    while Entry != head {
        if Entry as *mut c_void == Cookie {
            RemoveEntryList(Entry);
            Status = STATUS_SUCCESS;
            break;
        }
        Entry = (*Entry).Flink;
    }
    RtlLeaveCriticalSection(addr_of_mut!(LdrpDllNotificationLock));

    if NT_SUCCESS(Status) {
        // `Entry` holds the matched node (== Cookie, since `List` is at offset 0).
        RtlFreeHeap(LdrpHeap, 0, Entry as *mut c_void);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrpSendDllNotifications(
    DllEntry: PLDR_DATA_TABLE_ENTRY,
    NotificationReason: ULONG,
) {
    LdrpEnsureNotificationInit();

    // LDR_DLL_LOADED_NOTIFICATION_DATA and ..._UNLOADED_NOTIFICATION_DATA are
    // layout-identical (the C C_ASSERTs this); fill either arm of the union.
    let mut NotificationData: LDR_DLL_NOTIFICATION_DATA = zeroed();
    NotificationData.Loaded.Flags = 0; // Reserved, always 0 (not DllEntry->Flags).
    NotificationData.Loaded.FullDllName = addr_of!((*DllEntry).FullDllName) as PCUNICODE_STRING;
    NotificationData.Loaded.BaseDllName = addr_of!((*DllEntry).BaseDllName) as PCUNICODE_STRING;
    NotificationData.Loaded.DllBase = (*DllEntry).DllBase;
    NotificationData.Loaded.SizeOfImage = (*DllEntry).SizeOfImage;

    // Send notification to all registered callbacks. The C wraps the loop in
    // _SEH2_TRY/_SEH2_FINALLY purely to guarantee the Leave; under DUMMY_PSEH
    // (panic=abort, no unwind) the body runs then we Leave unconditionally.
    let head = addr_of_mut!(LdrpDllNotificationList);
    RtlEnterCriticalSection(addr_of_mut!(LdrpDllNotificationLock));
    let mut Entry = (*head).Flink;
    while Entry != head {
        let NotificationEntry = (Entry as *mut u8)
            .sub(offset_of!(LDR_DLL_NOTIFICATION_ENTRY, List))
            as *mut LDR_DLL_NOTIFICATION_ENTRY;
        if let Some(callback) = (*NotificationEntry).Callback {
            callback(
                NotificationReason,
                addr_of!(NotificationData),
                (*NotificationEntry).Context,
            );
        }
        Entry = (*Entry).Flink;
    }
    RtlLeaveCriticalSection(addr_of_mut!(LdrpDllNotificationLock));
}
