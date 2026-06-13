//! Rust translation of the classpnp-referenced subset of
//! `sdk/lib/drivers/ntoskrnl_vista` (the ReactOS "Vista+ kernel APIs" static
//! support lib). Replaces the cmake-built `libntoskrnl_vista.a` *for
//! classpnp.sys only* (the kernels keep the C lib). Tier-B step 5.
//!
//! Decoration matches the C `.a` exactly (verified via `nm`):
//!   i386: `_IoQueueWorkItemEx@16`, `_IoSetMasterIrpStatus` (cdecl — its C decl
//!   omits NTAPI), `_KeQueryHighestNodeNumber@0`, `_KeGetCurrentNodeNumber@0`,
//!   `_KeSetCoalescableTimer@24`, `_PoQueryWatchdogTime@8`.
//!   amd64/arm64: all undecorated.
//! → `IoSetMasterIrpStatus` uses `extern "C"` (cdecl on all arches); the other
//!   five use `extern "system"` (stdcall@N on i386, cdecl elsewhere). lld's
//!   undecorated→decorated fuzzy match reconciles classpnp's mixed-ABI imports,
//!   exactly as it did against the C lib.

#![no_std]
#![allow(non_snake_case, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

use ntkrnl_sys::ddk::{LARGE_INTEGER, PDEVICE_OBJECT, PIO_WORKITEM, PIRP, PKDPC, PKTIMER};

type NTSTATUS = i32;
type BOOLEAN = u8;
type ULONG = u32;
type USHORT = u16;
type PVOID = *mut core::ffi::c_void;

// NTSTATUS constants (faithful to the C source).
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_FT_READ_FROM_COPY: NTSTATUS = 0x4002_0006u32 as NTSTATUS;
const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x8000_0010u32 as NTSTATUS;

// io.c: TAG_IOWI 'IWOI'  (clang multichar → 0x49574F49, displays "IOWI").
const TAG_IOWI: ULONG = 0x4957_4F49;
// mm: NonPagedPoolMustSucceed
const NonPagedPoolMustSucceed: ULONG = 2;

#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// Kernel imports (NTAPI → extern "system"; ntoskrnl resolves them via the
// classpnp import lib). Declared locally so the @N decoration is unambiguous.
extern "system" {
    fn KeSetTimerEx(Timer: PKTIMER, DueTime: LARGE_INTEGER, Period: i32, Dpc: PKDPC) -> BOOLEAN;
    // WorkerRoutine passed as a raw fn-address (PVOID) — the IoQueueWorkItem ABI
    // is a 4/8-byte pointer; this sidesteps Option<fn-ptr> ABI friction (the
    // same shape classpnp-rust itself uses for IoQueueWorkItemEx).
    fn IoQueueWorkItem(IoWorkItem: PIO_WORKITEM, WorkerRoutine: PVOID, QueueType: ULONG, Context: PVOID);
    fn ExAllocatePoolWithTag(PoolType: ULONG, NumberOfBytes: usize, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

// ---- io.c -----------------------------------------------------------------

#[repr(C)]
struct EX_WORKITEM_CONTEXT {
    WorkItem: PIO_WORKITEM,
    WorkItemRoutineEx: PVOID, // PIO_WORKITEM_ROUTINE_EX (NTAPI fn-ptr)
    Context: PVOID,
}

/// IO_WORKITEM_ROUTINE trampoline (NTAPI). Static in C; invokes the stored
/// IO_WORKITEM_ROUTINE_EX `(IoObject, Context, IoWorkItem)` then frees the ctx.
unsafe extern "system" fn IopWorkItemExCallback(DeviceObject: PDEVICE_OBJECT, Ctx: PVOID) {
    let context = Ctx as *mut EX_WORKITEM_CONTEXT;
    let routine: unsafe extern "system" fn(PVOID, PVOID, PIO_WORKITEM) =
        core::mem::transmute((*context).WorkItemRoutineEx);
    routine(DeviceObject as PVOID, (*context).Context, (*context).WorkItem);
    ExFreePoolWithTag(Ctx, TAG_IOWI);
}

#[no_mangle]
pub unsafe extern "system" fn IoQueueWorkItemEx(
    IoWorkItem: PIO_WORKITEM,
    WorkerRoutine: PVOID, // PIO_WORKITEM_ROUTINE_EX
    QueueType: ULONG,     // WORK_QUEUE_TYPE
    Context: PVOID,
) {
    let newContext = ExAllocatePoolWithTag(
        NonPagedPoolMustSucceed,
        core::mem::size_of::<EX_WORKITEM_CONTEXT>(),
        TAG_IOWI,
    ) as *mut EX_WORKITEM_CONTEXT;
    (*newContext).WorkItem = IoWorkItem;
    (*newContext).WorkItemRoutineEx = WorkerRoutine;
    (*newContext).Context = Context;

    IoQueueWorkItem(
        IoWorkItem,
        IopWorkItemExCallback as PVOID,
        QueueType,
        newContext as PVOID,
    );
}

/// cdecl in the C source (no NTAPI) → `extern "C"`.
#[no_mangle]
pub unsafe extern "C" fn IoSetMasterIrpStatus(MasterIrp: PIRP, Status: NTSTATUS) {
    let MasterStatus: NTSTATUS = (*MasterIrp).IoStatus.__bindgen_anon_1.Status;

    if Status == STATUS_FT_READ_FROM_COPY {
        return;
    }

    if (Status == STATUS_VERIFY_REQUIRED)
        || (MasterStatus == STATUS_SUCCESS && !NT_SUCCESS(Status))
        || (!NT_SUCCESS(MasterStatus) && !NT_SUCCESS(Status) && Status > MasterStatus)
    {
        (*MasterIrp).IoStatus.__bindgen_anon_1.Status = Status;
    }
}

// ---- ke.c -----------------------------------------------------------------

#[no_mangle]
pub extern "system" fn KeQueryHighestNodeNumber() -> USHORT {
    0
}

#[no_mangle]
pub extern "system" fn KeGetCurrentNodeNumber() -> USHORT {
    0
}

#[no_mangle]
pub unsafe extern "system" fn KeSetCoalescableTimer(
    Timer: PKTIMER,
    DueTime: LARGE_INTEGER,
    Period: ULONG,
    _TolerableDelay: ULONG,
    Dpc: PKDPC,
) -> BOOLEAN {
    KeSetTimerEx(Timer, DueTime, Period as i32, Dpc)
}

// ---- po.c -----------------------------------------------------------------

#[no_mangle]
pub extern "system" fn PoQueryWatchdogTime(
    _Pdo: PDEVICE_OBJECT,
    _SecondsRemaining: *mut ULONG,
) -> BOOLEAN {
    0 // FALSE
}

// ---- runtime (staticlib) --------------------------------------------------
// Mirrors the hidparser-rust / cnt-rust-lib sibling pattern. Coexists with
// classpnp-rust's panic_handler via --allow-multiple-definition (panic=abort
// never unwinds, so neither body is reached).

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
