//! Rust impls of `static __inline` / FORCEINLINE bodies from
//! `ntoskrnl/include/internal/io_x.h` and `sdk/include/xdk/iofuncs.h` whose
//! C trampolines live in `ntoskrnl/io/iomgr/io_rust_shims.c`. Phase 2 sibling
//! rlib (architecturally identical to `ke-inlines` / `ke-macros` / etc. —
//! the rlib boundary cuts the cross-CGU captures(none) attribute deduction
//! wire under workspace `lto = false`).
//!
//! Pilot batch (3 fns):
//!  - `IopValidateOpenPacket` — type/size sanity of an `OPEN_PACKET`.
//!  - `IoGetDevObjExtension` — DEVICE_OBJECT → EXTENDED_DEVOBJ_EXTENSION cast
//!    (C macro in `ntoskrnl/include/internal/io.h`).
//!  - `IoCopyCurrentIrpStackLocationToNext` — IRP stack memcpy + Control reset
//!    (FORCEINLINE in `iofuncs.h`).
//!
//! C trampolines collapse from the inline body to a one-line `_rs_*`
//! delegation; the `_shim`-suffixed exported symbol that Rust callers link
//! against (via `link_name = "X_shim"`) stays put as the ABI anchor.

#![no_std]

use core::ptr;

use ntkrnl_sys::base::{InitializeListHead, InsertHeadList, IsListEmpty, RemoveEntryList};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PETHREAD, PEXTENDED_DEVOBJ_EXTENSION, PIRP, POPEN_PACKET, _IO_STACK_LOCATION,
    _OPEN_PACKET,
};

/// `IO_TYPE_OPEN_PACKET` from `sdk/include/xdk/iotypes.h`.
const IO_TYPE_OPEN_PACKET: i16 = 8;

/// IopValidateOpenPacket — `static __inline` from `io_x.h`.
///
/// Validates an OPEN_PACKET pointer + Type + Size. The C body uses an explicit
/// NULL check before dereferencing; preserved here as a pointer-null guard.
#[no_mangle]
pub unsafe extern "C" fn _rs_IopValidateOpenPacket(open_packet: POPEN_PACKET) -> u8 {
    if open_packet.is_null() {
        return 0;
    }
    if (*open_packet).Type != IO_TYPE_OPEN_PACKET
        || (*open_packet).Size as usize != core::mem::size_of::<_OPEN_PACKET>()
    {
        return 0;
    }
    1
}

/// IoGetDevObjExtension — `#define` macro from `ntoskrnl/include/internal/io.h`:
/// `((PEXTENDED_DEVOBJ_EXTENSION)(DeviceObject->DeviceObjectExtension))`.
#[no_mangle]
pub unsafe extern "C" fn _rs_IoGetDevObjExtension(
    device_object: PDEVICE_OBJECT,
) -> PEXTENDED_DEVOBJ_EXTENSION {
    (*device_object).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION
}

/// IoCopyCurrentIrpStackLocationToNext — `FORCEINLINE` from `iofuncs.h`.
///
/// Copies the current IRP stack location to the next slot, stopping at the
/// CompletionRoutine field (`FIELD_OFFSET(IO_STACK_LOCATION, CompletionRoutine)`),
/// then zeroes the next slot's Control byte.
#[no_mangle]
pub unsafe extern "C" fn _rs_IoCopyCurrentIrpStackLocationToNext(irp: PIRP) {
    // IoGetCurrentIrpStackLocation: Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation
    let cur = (*irp)
        .Tail
        .Overlay
        .__bindgen_anon_2
        .__bindgen_anon_1
        .CurrentStackLocation;
    // IoGetNextIrpStackLocation: cur - 1
    let next = cur.offset(-1);
    let copy_len = core::mem::offset_of!(_IO_STACK_LOCATION, CompletionRoutine);
    ptr::copy_nonoverlapping(cur as *const u8, next as *mut u8, copy_len);
    (*next).Control = 0;
}

// ============================================================================
// Batch 1 — IRP / thread queueing
//
// IopQueueIrpToThread / IopUnQueueIrpFromThread (FORCEINLINE in `io_x.h`).
// IopQueueIrpToThread enters a guarded region on the IRP's owning thread,
// inserts the IRP at the head of its IrpList, then leaves the guarded region.
// IopUnQueueIrpFromThread is called from `IopCompleteRequest` (and similar)
// where APCs are already disabled; it removes the IRP from its list and
// re-initializes the entry to a self-link.
// ============================================================================

extern "C" {
    fn KiCheckForKernelApcDelivery();
    fn KeAreAllApcsDisabled() -> u8;
}

/// `KeEnterGuardedRegionThread(_Thread)` — disables special APCs by
/// decrementing `KTHREAD::SpecialApcDisable`.
///
/// Path differs per arch:
/// - i386: `__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable`
/// - amd64/arm64: `__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable`
#[inline(always)]
unsafe fn special_apc_disable_ptr(
    tcb: *mut ntkrnl_sys::ddk::_KTHREAD,
) -> *mut i16 {
    #[cfg(target_arch = "x86")]
    {
        core::ptr::addr_of_mut!((*tcb).__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable)
    }
    #[cfg(not(target_arch = "x86"))]
    {
        core::ptr::addr_of_mut!(
            (*tcb)
                .__bindgen_anon_5
                .__bindgen_anon_8
                .__bindgen_anon_1
                .__bindgen_anon_1
                .SpecialApcDisable
        )
    }
}

#[inline(always)]
unsafe fn enter_guarded_region_thread(tcb: *mut ntkrnl_sys::ddk::_KTHREAD) {
    let p = special_apc_disable_ptr(tcb);
    *p = (*p).wrapping_sub(1);
}

#[inline(always)]
unsafe fn leave_guarded_region_thread(tcb: *mut ntkrnl_sys::ddk::_KTHREAD) {
    let p = special_apc_disable_ptr(tcb);
    *p = (*p).wrapping_add(1);
    if *p == 0 {
        // KernelMode = 0 — check kernel APC queue
        // KTHREAD::ApcState lives inside `__bindgen_anon_1` (an anonymous
        // union with `ApcStateFill`) on all 3 arches; KernelMode = 0.
        let kernel_apc_head =
            core::ptr::addr_of_mut!((*tcb).__bindgen_anon_1.ApcState.ApcListHead[0]);
        if !IsListEmpty(kernel_apc_head) {
            KiCheckForKernelApcDelivery();
        }
    }
}

/// IopQueueIrpToThread — FORCEINLINE in `io_x.h`. The IRP's `Tail.Overlay.Thread`
/// must be the current thread; we disable special APCs against that thread,
/// link the IRP onto its `IrpList`, and re-enable.
#[no_mangle]
pub unsafe extern "C" fn _rs_IopQueueIrpToThread(irp: PIRP) {
    let thread: PETHREAD = (*irp).Tail.Overlay.Thread;
    let tcb = core::ptr::addr_of_mut!((*thread).Tcb);
    enter_guarded_region_thread(tcb);
    InsertHeadList(
        core::ptr::addr_of_mut!((*thread).IrpList),
        core::ptr::addr_of_mut!((*irp).ThreadListEntry),
    );
    leave_guarded_region_thread(tcb);
}

/// IopUnQueueIrpFromThread — FORCEINLINE in `io_x.h`. Caller has APCs disabled
/// (called from IopCompleteRequest or with KeEnterCriticalRegion already
/// active). Removes the IRP's `ThreadListEntry` from whatever list it's on
/// (skipping the case where it's already been re-initialized to a self-link),
/// then re-initializes it.
#[no_mangle]
pub unsafe extern "C" fn _rs_IopUnQueueIrpFromThread(irp: PIRP) {
    debug_assert!(KeAreAllApcsDisabled() != 0);
    let entry = core::ptr::addr_of_mut!((*irp).ThreadListEntry);
    if IsListEmpty(entry) {
        return;
    }
    RemoveEntryList(entry);
    InitializeListHead(entry);
}
