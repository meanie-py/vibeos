//! ddk_shims.rs — cohab substrate: DDK FORCEINLINEs / macros the mountmgr translation
//! calls as extern functions but which have no exported symbol. Populated during link.
#![allow(non_snake_case, dead_code, unused_imports)]
use ntkrnl_sys::ddk::*;

// IsEqualGUID — DDK inline (guiddef.h); 16-byte compare. No exported symbol.
#[no_mangle]
pub unsafe extern "system" fn IsEqualGUID(a: *const GUID, b: *const GUID) -> u8 {
    (core::slice::from_raw_parts(a as *const u8, 16) == core::slice::from_raw_parts(b as *const u8, 16)) as u8
}

// ============================================================================
// DDK FORCEINLINEs / mntmgr.h macros the translation calls as extern functions
// (no exported symbol). Mirrors disk-rust/classpnp-rust ddk_shims.
// ============================================================================
use core::ffi::c_void;

// ---- mntmgr.h pool macros: AllocatePool(Size)=ExAllocatePoolWithTag(PagedPool,Size,'AtnM') ----
const MOUNTMGR_TAG: u32 = 0x41746E4D; // 'AtnM'
#[no_mangle]
pub unsafe extern "system" fn AllocatePool(Size: u32) -> *mut c_void {
    ExAllocatePoolWithTag(1 /* PagedPool */, Size as _, MOUNTMGR_TAG)
}
#[no_mangle]
pub unsafe extern "system" fn FreePool(P: *mut c_void) {
    ExFreePoolWithTag(P, MOUNTMGR_TAG)
}

// ---- IRP stack inlines (FORCEINLINE, no symbol) ----
#[no_mangle]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation.offset(-1)
}
#[no_mangle]
pub unsafe extern "C" fn IoMarkIrpPending(Irp: PIRP) {
    let stk = (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    (*stk).Control |= 0x01; // SL_PENDING_RETURNED
}

// ---- LIST_ENTRY inline ----
#[no_mangle]
pub unsafe extern "C" fn InsertTailList(ListHead: *mut LIST_ENTRY, Entry: *mut LIST_ENTRY) {
    let blink = (*ListHead).Blink;
    (*Entry).Flink = ListHead;
    (*Entry).Blink = blink;
    (*blink).Flink = Entry;
    (*ListHead).Blink = Entry;
}

// ---- Ke inline ----
// CC = ntkrnl-sys bindgen_i386 decl class (callers import THAT decl; a mismatched
// shim def in the same crate folds every such call to ud2 — the 2026-07-18 i386
// driver-fleet bomb class).
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) { *SpinLock = 0; }

// KeGetCurrentIrql: real export on i386 (KIRQL stdcall); inline on amd64 (cr8), arm64 (prcb).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

// ---- compiler intrinsics (no kernel export on i386) ----
use core::sync::atomic::{AtomicI32, Ordering};
#[no_mangle]
pub unsafe extern "system" fn _InterlockedExchange(Target: *mut i32, Value: i32) -> i32 {
    (&*(Target as *const AtomicI32)).swap(Value, Ordering::SeqCst)
}
// i386 does NOT get these defs: the kernel exports them fastcall
// (@InterlockedExchange@8 etc., ntoskrnl.spec:323-326 — nm-proven in the import
// lib) and mountmgr.rs declares them fastcall behind #[cfg(x86)]. A cdecl
// no_mangle def here collides with that decl in the same LLVM module and folds
// every i386 call site to ud2 (this bugchecked the boot at mountmgr's
// DriverEntry, 0x1E/0xc000001d). 64-bit has no kernel export, so the local
// atomic shims remain.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedExchange(Target: *mut i32, Value: i32) -> i32 {
    (&*(Target as *const AtomicI32)).swap(Value, Ordering::SeqCst)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedIncrement(Addend: *mut i32) -> i32 {
    (&*(Addend as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedDecrement(Addend: *mut i32) -> i32 {
    (&*(Addend as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}

// ---- mountmgr.h MOUNTMGR_IS_* macros (string-pattern tests on UNICODE_STRING) ----
#[no_mangle]
pub unsafe extern "system" fn MOUNTMGR_IS_DRIVE_LETTER(s: *const ntkrnl_sys::ddk::UNICODE_STRING) -> u8 {
    let b = (*s).Buffer;
    ((*s).Length == 28
        && *b.offset(0) == b'\\' as u16 && *b.offset(1) == b'D' as u16
        && *b.offset(2) == b'o' as u16 && *b.offset(3) == b's' as u16
        && *b.offset(4) == b'D' as u16 && *b.offset(5) == b'e' as u16
        && *b.offset(6) == b'v' as u16 && *b.offset(7) == b'i' as u16
        && *b.offset(8) == b'c' as u16 && *b.offset(9) == b'e' as u16
        && *b.offset(10) == b's' as u16 && *b.offset(11) == b'\\' as u16
        && *b.offset(12) >= b'A' as u16 && *b.offset(12) <= b'Z' as u16
        && *b.offset(13) == b':' as u16) as u8
}
#[no_mangle]
pub unsafe extern "system" fn MOUNTMGR_IS_VOLUME_NAME(s: *const ntkrnl_sys::ddk::UNICODE_STRING) -> u8 {
    let b = (*s).Buffer;
    (((*s).Length == 96 || ((*s).Length == 98 && *b.offset(48) == b'\\' as u16))
        && *b.offset(0) == b'\\' as u16
        && (*b.offset(1) == b'?' as u16 || *b.offset(1) == b'\\' as u16)
        && *b.offset(2) == b'?' as u16 && *b.offset(3) == b'\\' as u16
        && *b.offset(4) == b'V' as u16 && *b.offset(5) == b'o' as u16
        && *b.offset(6) == b'l' as u16 && *b.offset(7) == b'u' as u16
        && *b.offset(8) == b'm' as u16 && *b.offset(9) == b'e' as u16
        && *b.offset(10) == b'{' as u16
        && *b.offset(19) == b'-' as u16 && *b.offset(24) == b'-' as u16
        && *b.offset(29) == b'-' as u16 && *b.offset(34) == b'-' as u16
        && *b.offset(47) == b'}' as u16) as u8
}

// ---- IoSetCancelRoutine (FORCEINLINE): atomically set Irp->CancelRoutine, return old ----
#[no_mangle]
pub unsafe extern "C" fn IoSetCancelRoutine(
    Irp: PIRP, CancelRoutine: PDRIVER_CANCEL,
) -> PDRIVER_CANCEL {
    let slot = core::ptr::addr_of_mut!((*Irp).CancelRoutine) as *mut usize;
    let old = core::sync::atomic::AtomicUsize::from_ptr(slot)
        .swap(core::mem::transmute_copy(&CancelRoutine), core::sync::atomic::Ordering::SeqCst);
    core::mem::transmute_copy(&old)
}

// KeAcquireSpinLockRaiseToDpc: exported on amd64/arm64; on i386 it's a macro
// (KfAcquireSpinLock). Provide it on i386: raise IRQL to DISPATCH, acquire at DPC level,
// return old IRQL.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KefAcquireSpinLockAtDpcLevel(SpinLock: *mut usize);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KeAcquireSpinLockRaiseToDpc(SpinLock: *mut usize) -> u8 {
    let old = KeRaiseIrqlToDpcLevel();
    KefAcquireSpinLockAtDpcLevel(SpinLock);
    old
}
