//! inlines.rs — partmgr.h FORCEINLINE helpers + header macros the LLM rendered as
//! function calls. They are header inlines / macros, NOT link exports, so the kept-C
//! cohab build never provided them either (all .c are translated) — we define them
//! here. Bodies are faithful to partmgr.h / ntdddisk.h.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]
use crate::types::*;
use ntkrnl_sys::ddk::{
    BOOLEAN, KEVENT, KPRIORITY, KPROCESSOR_MODE, KWAIT_REASON, PIRP, PIO_STACK_LOCATION,
    POOL_TYPE, PRKEVENT, PVOID, SIZE_T, ULONG, ExAllocatePoolWithTag, KeSetEvent,
    KeWaitForSingleObject,
};
// FORCEINLINE IRP-stack walkers (Irp->Tail.Overlay) — NOT ntoskrnl exports; the bindgen
// `ddk` versions are phantom externs that fail at link. Re-export the real inline impls.
pub use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};

// exfuncs.h FORCEINLINE pool allocators (NTAPI, wrap the exported ExAllocatePoolWithTag;
// NOT exports themselves → must be provided inline).
#[inline(always)]
pub unsafe fn ExAllocatePoolZero(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID {
    let p = ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag);
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, NumberOfBytes as usize);
    }
    p
}
#[inline(always)]
pub unsafe fn ExAllocatePoolUninitialized(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID {
    ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag)
}

// DbgPrintEx — variadic kernel debug print. Not in the ntkrnl-sys bindgen surface
// (bindgen drops variadic protos), so declare it here.
extern "C" {
    pub fn DbgPrintEx(ComponentId: u32, Level: u32, Format: *const i8, ...) -> u32;
}

// ntdddisk.h: IsContainerPartition(PartitionType)
pub const PARTITION_EXTENDED: u8 = 0x05;
pub const PARTITION_XINT13_EXTENDED: u8 = 0x0F;
#[inline(always)]
pub unsafe fn IsContainerPartition(PartitionType: u8) -> bool {
    PartitionType == PARTITION_EXTENDED || PartitionType == PARTITION_XINT13_EXTENDED
}

// reactos/debug driverdbg.h: GetIRPMinorFunctionString — diagnostic-only, the value
// only ever feeds a DbgPrintEx %s. The real header impl is a big switch returning a
// string literal; a constant string is behaviorally inert here.
#[inline(always)]
pub unsafe fn GetIRPMinorFunctionString(_MinorFunction: u8) -> *mut i8 {
    b"IRP_MN_?\0".as_ptr() as *mut i8
}

// partmgr.h FORCEINLINE buffer-size verifiers (return BOOLEAN; callers compare `== 0`).
// Size is `usize` so the `(X as SIZE_T)` call args (which resolve to usize via the base
// glob) match on both arches; Information is assigned via `as _` to infer ULONG_PTR.
#[inline(always)]
pub unsafe fn VerifyIrpOutBufferSize(Irp: PIRP, Size: usize) -> BOOLEAN {
    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    if ((*ioStack).Parameters.DeviceIoControl.OutputBufferLength as usize) < Size {
        (*Irp).IoStatus.Information = Size as _;
        return 0 as BOOLEAN; // FALSE
    }
    1 as BOOLEAN // TRUE
}
#[inline(always)]
pub unsafe fn VerifyIrpInBufferSize(Irp: PIRP, Size: usize) -> BOOLEAN {
    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    if ((*ioStack).Parameters.DeviceIoControl.InputBufferLength as usize) < Size {
        (*Irp).IoStatus.Information = Size as _;
        return 0 as BOOLEAN;
    }
    1 as BOOLEAN
}

// partmgr.h FORCEINLINE layout lock (KEVENT SyncEvent, Executive/KernelMode/IO_NO_INCREMENT = 0).
#[inline(always)]
pub unsafe fn PartMgrAcquireLayoutLock(FDOExtension: PFDO_EXTENSION) {
    KeWaitForSingleObject(
        &mut (*FDOExtension).SyncEvent as *mut KEVENT as PVOID,
        0 as KWAIT_REASON,    // Executive
        0 as KPROCESSOR_MODE, // KernelMode
        0 as BOOLEAN,         // Alertable = FALSE
        core::ptr::null_mut(),
    );
}
#[inline(always)]
pub unsafe fn PartMgrReleaseLayoutLock(FDOExtension: PFDO_EXTENSION) {
    KeSetEvent(
        &mut (*FDOExtension).SyncEvent as *mut KEVENT as PRKEVENT,
        0 as KPRIORITY, // IO_NO_INCREMENT
        0 as BOOLEAN,   // Wait = FALSE
    );
}
