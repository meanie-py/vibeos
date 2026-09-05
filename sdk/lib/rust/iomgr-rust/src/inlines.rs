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
use core::sync::atomic::{AtomicI64, AtomicU32, Ordering};

use ntkrnl_sys::base::{InitializeListHead, InsertHeadList, IsListEmpty, RemoveEntryList};
use ntkrnl_sys::ddk::{
    BOOLEAN, HAL_DISPATCH, KPROCESSOR_MODE, NTSTATUS, PCM_FULL_RESOURCE_DESCRIPTOR,
    PDEVICE_OBJECT, PEPROCESS, PETHREAD, PEXTENDED_DEVOBJ_EXTENSION, PFILE_OBJECT, PIRP,
    PIO_STACK_LOCATION, POPEN_PACKET, PRKEVENT, _CM_FULL_RESOURCE_DESCRIPTOR,
    _IO_STACK_LOCATION, _OPEN_PACKET,
};

/// `IO_TYPE_OPEN_PACKET` from `sdk/include/xdk/iotypes.h`.
const IO_TYPE_OPEN_PACKET: i16 = 8;

// NOTE (2026-07-09): the aarch64 stride/offset shims that used to live here
// are gone — the iotypes.h pack(4) guard now covers _ARM64_, so C and bindgen
// agree on IO_STACK_LOCATION/IRP layout on every arch (see irp.rs note).

#[inline(always)]
unsafe fn irp_current_stack_location(irp: PIRP) -> PIO_STACK_LOCATION {
    (*irp)
        .Tail
        .Overlay
        .__bindgen_anon_2
        .__bindgen_anon_1
        .CurrentStackLocation
}

#[inline(always)]
unsafe fn irp_stack_location_add(
    stack: PIO_STACK_LOCATION,
    count: isize,
) -> PIO_STACK_LOCATION {
    stack.offset(count)
}

/// IopValidateOpenPacket — `static __inline` from `io_x.h`.
///
/// Validates an OPEN_PACKET pointer + Type + Size. The C body uses an explicit
/// NULL check before dereferencing; preserved here as a pointer-null guard.
#[no_mangle]
pub unsafe extern "system" fn IopValidateOpenPacket_shim(open_packet: POPEN_PACKET) -> u8 {
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
pub unsafe extern "system" fn IoGetDevObjExtension_shim(
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
pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext_shim(irp: PIRP) {
    let cur = irp_current_stack_location(irp);
    let next = irp_stack_location_add(cur, -1);
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

// All four are NTAPI (stdcall on i386 → `_Name@N` decoration; cdecl on amd64/arm64).
// Must be `extern "system"` so the i386 link picks up the `_Name@N` symbol.
extern "system" {
    fn KiCheckForKernelApcDelivery();
    fn KeAreAllApcsDisabled() -> u8;
    fn KeSetEvent(event: PRKEVENT, increment: i32, wait: u8) -> i32;
    fn IopAcquireFileObjectLock(
        file_object: PFILE_OBJECT,
        access_mode: KPROCESSOR_MODE,
        alertable: u8,
        lock_failed: *mut u8,
    ) -> NTSTATUS;
}

// `Obf{Reference,Dereference}Object` is `__fastcall` on i386, default elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ObfReferenceObject(object: *mut core::ffi::c_void) -> isize;
    fn ObfDereferenceObject(object: *mut core::ffi::c_void) -> isize;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ObfReferenceObject(object: *mut core::ffi::c_void) -> isize;
    fn ObfDereferenceObject(object: *mut core::ffi::c_void) -> isize;
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
pub unsafe extern "system" fn IopQueueIrpToThread_shim(irp: PIRP) {
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
pub unsafe extern "system" fn IopUnQueueIrpFromThread_shim(irp: PIRP) {
    ntkrnl_sys::ASSERT!(KeAreAllApcsDisabled() != 0);
    let entry = core::ptr::addr_of_mut!((*irp).ThreadListEntry);
    if IsListEmpty(entry) {
        return;
    }
    RemoveEntryList(entry);
    InitializeListHead(entry);
}

// ============================================================================
// Batch 2 — file-object lock pair
//
// IopLockFileObject / IopUnlockFileObject (`static __inline` in io_x.h). Used
// for synchronous I/O: the FILE_OBJECT exposes a `Busy` byte that
// InterlockedExchange flips between 0 and 1; on contention we fall through to
// `IopAcquireFileObjectLock` which blocks on the FO's KEVENT. Unlock is
// symmetric: flip Busy back to 0 and signal any waiters.
// ============================================================================

/// `FO_ALERTABLE_IO` from `iotypes.h` — 0x4 in `FILE_OBJECT::Flags`.
const FO_ALERTABLE_IO: u32 = 0x00000004;

/// IopLockFileObject — `static __inline` in `io_x.h`. Fast path: claim
/// `FO->Busy` via `InterlockedExchange(.., TRUE)`; if the previous value was
/// FALSE we won unconditionally and just take a reference. Slow path: call
/// `IopAcquireFileObjectLock` which blocks on the FO's KEVENT.
#[no_mangle]
pub unsafe extern "system" fn IopLockFileObject_shim(
    file_object: PFILE_OBJECT,
    wait_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let busy = AtomicU32::from_ptr(core::ptr::addr_of_mut!((*file_object).Busy));
    if busy.swap(1, Ordering::SeqCst) == 0 {
        ObfReferenceObject(file_object as *mut core::ffi::c_void);
        return 0; // STATUS_SUCCESS
    }
    let mut lock_failed: u8 = 0;
    let alertable: u8 = if ((*file_object).Flags & FO_ALERTABLE_IO) != 0 {
        1
    } else {
        0
    };
    IopAcquireFileObjectLock(file_object, wait_mode, alertable, &mut lock_failed)
}

/// IopUnlockFileObject — `static __inline` in `io_x.h`. Releases the FO lock
/// taken by `IopLockFileObject`: flip `Busy` back to FALSE (verifying the
/// caller really held it), wake any waiters via `KeSetEvent`, and drop the
/// reference taken on lock acquire.
#[no_mangle]
pub unsafe extern "system" fn IopUnlockFileObject_shim(file_object: PFILE_OBJECT) {
    let busy = AtomicU32::from_ptr(core::ptr::addr_of_mut!((*file_object).Busy));
    let prev = busy.swap(0, Ordering::SeqCst);
    ntkrnl_sys::ASSERT!(prev == 1);
    if (*file_object).Waiters != 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*file_object).Lock), 0, 0);
    }
    ObfDereferenceObject(file_object as *mut core::ffi::c_void);
}

// ============================================================================
// Batch 3 — operation/transfer counters
//
// IopUpdateOperationCount / IopUpdateTransferCount (`static __inline` in
// `io_x.h`). When the global `IoCountOperations` is non-zero, increment the
// per-CPU/global counter and the per-process counter (Read/Write/Other slot
// keyed off `IOP_TRANSFER_TYPE`). Per-process counters are bumped via the
// `ExInterlockedAddLargeStatistic` macro: on amd64/arm64 it's
// `InterlockedAdd64(&Addend->QuadPart, Increment)`; on i386 it's a custom
// `lock addl` + `lock adcl` pair that we render as `AtomicI64::fetch_add`
// which lowers to `lock cmpxchg8b` on i386 (semantically equivalent).
// ============================================================================

/// `IOP_TRANSFER_TYPE` from bindgen — IopReadTransfer = 0, IopWriteTransfer = 1,
/// IopOtherTransfer = 2.
const IOP_READ_TRANSFER: u32 = 0;
const IOP_WRITE_TRANSFER: u32 = 1;

extern "C" {
    static mut IoCountOperations: u8; // BOOLEAN
    static mut IoReadOperationCount: u32;
    static mut IoWriteOperationCount: u32;
    static mut IoOtherOperationCount: u32;
    static mut IoReadTransferCount: i64; // LARGE_INTEGER (QuadPart-equivalent)
    static mut IoWriteTransferCount: i64;
    static mut IoOtherTransferCount: i64;
}

/// PsGetCurrentProcess replicated locally — same cross-crate inline-asm dodge
/// as `ke-macros::ke_get_current_thread` (see
/// `feedback_cross_crate_inline_asm.md`). Walks PCR → PRCB → KTHREAD →
/// ApcState.Process to land on the current EPROCESS.
#[inline(always)]
unsafe fn ps_get_current_process() -> PEPROCESS {
    let pcr: *mut ntkrnl_sys::ddk::_KPCR;
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "mov {}, gs:[0x18]",
            out(reg) pcr,
            options(nostack, nomem, preserves_flags),
        );
    }
    #[cfg(target_arch = "x86")]
    {
        let raw: u32;
        core::arch::asm!(
            "mov {:e}, dword ptr fs:[0x1C]",
            out(reg) raw,
            options(nostack, nomem, preserves_flags),
        );
        pcr = raw as *mut ntkrnl_sys::ddk::_KPCR;
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!(
            "mrs {}, tpidr_el1",
            out(reg) pcr,
            options(nostack, nomem, preserves_flags),
        );
    }
    let prcb = {
        #[cfg(target_arch = "x86")]
        {
            (*pcr).Prcb
        }
        #[cfg(not(target_arch = "x86"))]
        {
            (*pcr).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb
        }
    };
    let thread = (*prcb).CurrentThread as *mut ntkrnl_sys::ddk::_KTHREAD;
    (*thread).__bindgen_anon_1.ApcState.Process as PEPROCESS
}

/// Atomic add 1/N to a `LARGE_INTEGER::QuadPart`. Equivalent to
/// `ExInterlockedAddLargeStatistic` on all 3 arches (cmpxchg8b on i386,
/// lock xaddq on amd64, ldxr/stxr on arm64).
#[inline(always)]
unsafe fn add_large_statistic(addend: *mut i64, increment: i32) {
    AtomicI64::from_ptr(addend).fetch_add(increment as i64, Ordering::SeqCst);
}

/// IopUpdateOperationCount — `static __inline` in `io_x.h`. When operation
/// counting is enabled, bump the global Read/Write/Other counter (32-bit,
/// non-atomic — matches the C body) and the per-process LARGE_INTEGER
/// counter atomically.
#[no_mangle]
pub unsafe extern "system" fn IopUpdateOperationCount_shim(transfer_type: u32) {
    if IoCountOperations == 0 {
        return;
    }
    let process = ps_get_current_process();
    let count_to_change: *mut i64 = if transfer_type == IOP_READ_TRANSFER {
        IoReadOperationCount = IoReadOperationCount.wrapping_add(1);
        core::ptr::addr_of_mut!((*process).ReadOperationCount.QuadPart)
    } else if transfer_type == IOP_WRITE_TRANSFER {
        IoWriteOperationCount = IoWriteOperationCount.wrapping_add(1);
        core::ptr::addr_of_mut!((*process).WriteOperationCount.QuadPart)
    } else {
        IoOtherOperationCount = IoOtherOperationCount.wrapping_add(1);
        core::ptr::addr_of_mut!((*process).OtherOperationCount.QuadPart)
    };
    add_large_statistic(count_to_change, 1);
}

/// IopUpdateTransferCount — `static __inline` in `io_x.h`. Bumps both the
/// per-process and global LARGE_INTEGER counters by `transfer_count` bytes.
#[no_mangle]
pub unsafe extern "system" fn IopUpdateTransferCount_shim(transfer_type: u32, transfer_count: u32) {
    if IoCountOperations == 0 {
        return;
    }
    let process = ps_get_current_process();
    let (count_to_change, transfer_to_change): (*mut i64, *mut i64) =
        if transfer_type == IOP_READ_TRANSFER {
            (
                core::ptr::addr_of_mut!((*process).ReadTransferCount.QuadPart),
                core::ptr::addr_of_mut!(IoReadTransferCount),
            )
        } else if transfer_type == IOP_WRITE_TRANSFER {
            (
                core::ptr::addr_of_mut!((*process).WriteTransferCount.QuadPart),
                core::ptr::addr_of_mut!(IoWriteTransferCount),
            )
        } else {
            (
                core::ptr::addr_of_mut!((*process).OtherTransferCount.QuadPart),
                core::ptr::addr_of_mut!(IoOtherTransferCount),
            )
        };
    add_large_statistic(count_to_change, transfer_count as i32);
    add_large_statistic(transfer_to_change, transfer_count as i32);
}

// ============================================================================
// Batch 4 — surviving `io_rust_shims.c` non-variadic helpers
//
// HAL dispatch trampolines, the `IopIsValidPhysicalDeviceObject` macro
// (`io.h`), and the `CmiGetNextResourceDescriptor` FORCEINLINE
// (`cmreslist.h`). Replaces hand-written C trampolines.
// ============================================================================

extern "C" {
    static mut HalDispatchTable: HAL_DISPATCH;
}

/// `HalInitPnpDriver_shim` — `HALDISPATCH->HalInitPnpDriver()`. The macro
/// `HALDISPATCH` (in `haltypes.h`) expands to a dispatch-table pointer
/// indirection; iomgr.rs links to this via `#[link_name]`.
#[no_mangle]
pub unsafe extern "C" fn HalInitPnpDriver_shim() -> NTSTATUS {
    (HalDispatchTable.HalInitPnpDriver.unwrap())()
}

/// `DNF_ENUMERATED` from `sdk/include/ndk/iotypes.h`.
const DNF_ENUMERATED: u32 = 0x00000010;

/// `IopIsValidPhysicalDeviceObject` — macro in `internal/io.h`. Walks
/// PDO → EXTENDED_DEVOBJ_EXTENSION → DEVICE_NODE and tests the
/// DNF_ENUMERATED flag. NTAPI on i386 — callers in `pnpmgr.rs` and
/// `pnpreport.rs` declare `extern "system"`.
#[no_mangle]
pub unsafe extern "system" fn IopIsValidPhysicalDeviceObject(
    physical_device_object: PDEVICE_OBJECT,
) -> BOOLEAN {
    if physical_device_object.is_null() {
        return 0;
    }
    let ext: PEXTENDED_DEVOBJ_EXTENSION =
        (*physical_device_object).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION;
    if ext.is_null() {
        return 0;
    }
    let device_node = (*ext).DeviceNode;
    if device_node.is_null() {
        return 0;
    }
    if ((*device_node).Flags & DNF_ENUMERATED) != 0 { 1 } else { 0 }
}

/// `CmResourceTypeDeviceSpecific` from `sdk/include/xdk/cmtypes.h`.
const CM_RESOURCE_TYPE_DEVICE_SPECIFIC: u8 = 5;

/// `CmiGetNextResourceDescriptor` — `FORCEINLINE` in `cmreslist.h`. Bindgen
/// emits an `extern "C"` declaration but no symbol exists since it's inline;
/// `pnpres.rs` resolves the call against this implementation.
#[no_mangle]
pub unsafe extern "C" fn CmiGetNextResourceDescriptor(
    resource_descriptor: *const _CM_FULL_RESOURCE_DESCRIPTOR,
) -> PCM_FULL_RESOURCE_DESCRIPTOR {
    let count = (*resource_descriptor).PartialResourceList.Count as usize;
    // Last partial descriptor — note Count >= 1 by construction (the
    // FORCEINLINE in `cmreslist.h` indexes [Count - 1] without a bound check).
    let last_partial = (*resource_descriptor)
        .PartialResourceList
        .PartialDescriptors
        .as_ptr()
        .add(count - 1);
    let extra = if (*last_partial).Type == CM_RESOURCE_TYPE_DEVICE_SPECIFIC {
        (*last_partial).u.DeviceSpecificData.DataSize as usize
    } else {
        0
    };
    let next = (last_partial as *const u8)
        .add(core::mem::size_of::<ntkrnl_sys::ddk::CM_PARTIAL_RESOURCE_DESCRIPTOR>())
        .add(extra);
    next as PCM_FULL_RESOURCE_DESCRIPTOR
}
