use ntkrnl_sys::base::*;
// Cross-crate ke/mm/ARM3 globals+recorders as link symbols (all #[no_mangle]
// in ntoskrnl-rust). crate-split Phase 1b.
extern "C" {
    static mut KeNumberProcessors: core::ffi::c_char;
}
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::macros::*;

use ntkrnl_sys::ddk::{
    EX_RUNDOWN_REF, EX_RUNDOWN_REF_CACHE_AWARE, EX_RUNDOWN_WAIT_BLOCK,
    PEX_RUNDOWN_REF, PEX_RUNDOWN_REF_CACHE_AWARE, PEX_RUNDOWN_WAIT_BLOCK,
    PKEVENT,
    ExGetRunRefForGivenProcessor,
    KeGetCurrentProcessorNumber, KeGetRecommendedSharedDataAlignment,
    KeInitializeEvent, KeWaitForSingleObject,
};

const IO_NO_INCREMENT: LONG = 0;

const EX_RUNDOWN_ACTIVE: usize = 1;
const EX_RUNDOWN_COUNT_INC: usize = 2;
const EX_RUNDOWN_COUNT_SHIFT: usize = 1;

#[inline(always)]
unsafe fn ExpChangeRundown(
    RunRef: PEX_RUNDOWN_REF,
    NewValue: usize,
    OldValue: usize,
) -> usize {
    core::sync::atomic::AtomicUsize::from_ptr(
        core::ptr::addr_of_mut!((*RunRef).__bindgen_anon_1.Count) as *mut usize
    ).compare_exchange(
        OldValue,
        NewValue,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ).unwrap_or_else(|x| x)
}

#[inline(always)]
unsafe fn ExpSetRundown(RunRef: PEX_RUNDOWN_REF, Value: usize) {
    core::sync::atomic::AtomicUsize::from_ptr(
        core::ptr::addr_of_mut!((*RunRef).__bindgen_anon_1.Count) as *mut usize
    ).store(Value, core::sync::atomic::Ordering::SeqCst);
}

#[inline(always)]
unsafe fn _ExAcquireRundownProtection(RunRef: PEX_RUNDOWN_REF) -> BOOLEAN {
    ExfAcquireRundownProtection_impl(RunRef)
}

#[inline(always)]
unsafe fn _ExReleaseRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfReleaseRundownProtection_impl(RunRef);
}

#[inline(always)]
unsafe fn _ExInitializeRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfInitializeRundownProtection_impl(RunRef);
}

// =============================================================================
// Implementation functions (inline, no ABI annotation)
// =============================================================================

#[inline(always)]
unsafe fn ExfAcquireRundownProtection_impl(RunRef: PEX_RUNDOWN_REF) -> BOOLEAN {
    let mut Value = (*RunRef).__bindgen_anon_1.Count as usize;

    loop {
        if (Value & EX_RUNDOWN_ACTIVE) != 0 {
            return 0u8;
        }

        let NewValue = Value + EX_RUNDOWN_COUNT_INC;
        let Result = ExpChangeRundown(RunRef, NewValue, Value);
        if Result == Value {
            return 1u8;
        }

        Value = Result;
    }
}

#[inline(always)]
unsafe fn ExfAcquireRundownProtectionEx_impl(
    RunRef: PEX_RUNDOWN_REF,
    Count: ULONG,
) -> BOOLEAN {
    let mut Value = (*RunRef).__bindgen_anon_1.Count as usize;

    loop {
        if (Value & EX_RUNDOWN_ACTIVE) != 0 {
            return 0u8;
        }

        let NewValue = Value + EX_RUNDOWN_COUNT_INC * (Count as usize);
        let Result = ExpChangeRundown(RunRef, NewValue, Value);
        if Result == Value {
            return 1u8;
        }

        Value = Result;
    }
}

#[inline(always)]
unsafe fn ExfInitializeRundownProtection_impl(RunRef: PEX_RUNDOWN_REF) {
    (*RunRef).__bindgen_anon_1.Count = 0;
}

#[inline(always)]
unsafe fn ExfReInitializeRundownProtection_impl(RunRef: PEX_RUNDOWN_REF) {
    ExpSetRundown(RunRef, 0);
}

#[inline(always)]
unsafe fn ExfRundownCompleted_impl(RunRef: PEX_RUNDOWN_REF) {
    ExpSetRundown(RunRef, EX_RUNDOWN_ACTIVE);
}

#[inline(always)]
unsafe fn ExfReleaseRundownProtection_impl(RunRef: PEX_RUNDOWN_REF) {
    let mut Value = (*RunRef).__bindgen_anon_1.Count as usize;

    loop {
        if (Value & EX_RUNDOWN_ACTIVE) == 0 {
            let NewValue = Value - EX_RUNDOWN_COUNT_INC;
            let Result = ExpChangeRundown(RunRef, NewValue, Value);
            if Result == Value {
                break;
            }

            Value = Result;
        } else {
            let WaitBlock = (Value & !EX_RUNDOWN_ACTIVE) as PEX_RUNDOWN_WAIT_BLOCK;

            if InterlockedDecrementSizeT(
                core::ptr::addr_of_mut!((*WaitBlock).Count) as *mut isize
            ) == 0 {
                KeSetEvent(
                    core::ptr::addr_of_mut!((*WaitBlock).WakeEvent) as PVOID,
                    IO_NO_INCREMENT,
                    0u8,
                );
            }

            break;
        }
    }
}

#[inline(always)]
unsafe fn ExfReleaseRundownProtectionEx_impl(
    RunRef: PEX_RUNDOWN_REF,
    Count: ULONG,
) {
    let mut Value = (*RunRef).__bindgen_anon_1.Count as usize;

    loop {
        if (Value & EX_RUNDOWN_ACTIVE) == 0 {
            let NewValue = Value - EX_RUNDOWN_COUNT_INC * (Count as usize);
            let Result = ExpChangeRundown(RunRef, NewValue, Value);
            if Result == Value {
                break;
            }

            Value = Result;
        } else {
            let WaitBlock = (Value & !EX_RUNDOWN_ACTIVE) as PEX_RUNDOWN_WAIT_BLOCK;

            if InterlockedExchangeAddSizeT(
                core::ptr::addr_of_mut!((*WaitBlock).Count) as *mut isize,
                -(Count as isize),
            ) == (Count as isize) {
                KeSetEvent(
                    core::ptr::addr_of_mut!((*WaitBlock).WakeEvent) as PVOID,
                    IO_NO_INCREMENT,
                    0u8,
                );
            }

            break;
        }
    }
}

#[inline(always)]
unsafe fn ExfWaitForRundownProtectionRelease_impl(RunRef: PEX_RUNDOWN_REF) {
    let mut Value = ExpChangeRundown(RunRef, EX_RUNDOWN_ACTIVE, 0);
    if Value == 0 || Value == EX_RUNDOWN_ACTIVE {
        return;
    }

    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut WaitBlock: EX_RUNDOWN_WAIT_BLOCK = core::mem::zeroed();
    let WaitBlockPointer = (core::ptr::addr_of_mut!(WaitBlock) as usize) | EX_RUNDOWN_ACTIVE;

    loop {
        let CountVal = Value >> EX_RUNDOWN_COUNT_SHIFT;

        if CountVal != 0 && Event.is_null() {
            KeInitializeEvent(
                core::ptr::addr_of_mut!(WaitBlock.WakeEvent) as PKEVENT,
                ntkrnl_sys::ddk::_EVENT_TYPE_SynchronizationEvent as _,
                0u8,
            );
            Event = core::ptr::addr_of_mut!(WaitBlock.WakeEvent) as PKEVENT;
        }

        WaitBlock.Count = CountVal as _;

        let NewValue = ExpChangeRundown(RunRef, WaitBlockPointer, Value);
        if NewValue == Value {
            break;
        }

        Value = NewValue;
    }

    if WaitBlock.Count == 0 {
        return;
    }

    // RunToken pilot (execution-context-ownership-model.md Layer 1): the CPS
    // wait consumes the region's token — a guard borrowing it would not compile.
    ntkrnl_sys::token::with_token::<ntkrnl_sys::token::Passive, _>(|tok| {
        ntkrnl_sys::token::wait_for(
            tok,
            Event as PVOID,
            0,
            0i8,
            0u8,
            core::ptr::null_mut(),
            |_tok, _status| (),
        )
    });
}

#[inline(always)]
unsafe fn ExfAcquireRundownProtectionCacheAware_impl(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) -> BOOLEAN {
    let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
    _ExAcquireRundownProtection(RunRef)
}

#[inline(always)]
unsafe fn ExfAcquireRundownProtectionCacheAwareEx_impl(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
    Count: ULONG,
) -> BOOLEAN {
    let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
    ExfAcquireRundownProtectionEx_impl(RunRef, Count)
}

#[inline(always)]
unsafe fn ExfReleaseRundownProtectionCacheAware_impl(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
    _ExReleaseRundownProtection(RunRef);
}

#[inline(always)]
unsafe fn ExfReleaseRundownProtectionCacheAwareEx_impl(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
    Count: ULONG,
) {
    let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
    ExfReleaseRundownProtectionEx_impl(RunRef, Count);
}

#[inline(always)]
unsafe fn ExfWaitForRundownProtectionReleaseCacheAware_impl(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    let ProcCount = (*RunRefCacheAware).Number;
    if ProcCount == 0 {
        return;
    }

    let mut TotalCount: usize = 0;
    let mut WaitBlock: EX_RUNDOWN_WAIT_BLOCK = core::mem::zeroed();
    WaitBlock.Count = 0;
    let WaitBlockPointer = (core::ptr::addr_of_mut!(WaitBlock) as usize) | EX_RUNDOWN_ACTIVE;

    for Current in 0..ProcCount {
        let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Current);
        let mut Value = (*RunRef).__bindgen_anon_1.Count as usize;

        loop {
            let OldValue = ExpChangeRundown(RunRef, WaitBlockPointer, Value);
            if OldValue == Value {
                break;
            }
            Value = OldValue;
        }

        TotalCount += Value;
    }

    if TotalCount != 0 {
        KeInitializeEvent(
            core::ptr::addr_of_mut!(WaitBlock.WakeEvent) as PKEVENT,
            ntkrnl_sys::ddk::_EVENT_TYPE_SynchronizationEvent as _,
            0u8,
        );

        if InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!(WaitBlock.Count) as *mut isize,
            (TotalCount >> EX_RUNDOWN_COUNT_SHIFT) as isize,
        ) == -((TotalCount >> EX_RUNDOWN_COUNT_SHIFT) as isize) {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(WaitBlock.WakeEvent) as PVOID,
                0,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
        }
    }
}

#[inline(always)]
unsafe fn ExfRundownCompletedCacheAware_impl(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    let ProcCount = (*RunRefCacheAware).Number;
    if ProcCount == 0 {
        return;
    }

    for Current in 0..ProcCount {
        let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Current);
        ExpSetRundown(RunRef, EX_RUNDOWN_ACTIVE);
    }
}

#[inline(always)]
unsafe fn ExfReInitializeRundownProtectionCacheAware_impl(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    let ProcCount = (*RunRefCacheAware).Number;
    if ProcCount == 0 {
        return;
    }

    for Current in 0..ProcCount {
        let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Current);
        ExpSetRundown(RunRef, 0);
    }
}

// =============================================================================
// Exported functions — cfg-gated for calling convention
// On i386, FASTCALL produces @Name@N decoration.
// On amd64/arm64, fastcall is the default C calling convention.
// =============================================================================

// --- ExfAcquireRundownProtection ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfAcquireRundownProtection(RunRef: PEX_RUNDOWN_REF) -> BOOLEAN {
    ExfAcquireRundownProtection_impl(RunRef)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfAcquireRundownProtection(RunRef: PEX_RUNDOWN_REF) -> BOOLEAN {
    ExfAcquireRundownProtection_impl(RunRef)
}

// --- ExfAcquireRundownProtectionEx ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfAcquireRundownProtectionEx(
    RunRef: PEX_RUNDOWN_REF,
    Count: ULONG,
) -> BOOLEAN {
    ExfAcquireRundownProtectionEx_impl(RunRef, Count)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfAcquireRundownProtectionEx(
    RunRef: PEX_RUNDOWN_REF,
    Count: ULONG,
) -> BOOLEAN {
    ExfAcquireRundownProtectionEx_impl(RunRef, Count)
}

// --- ExfInitializeRundownProtection ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfInitializeRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfInitializeRundownProtection_impl(RunRef)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInitializeRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfInitializeRundownProtection_impl(RunRef)
}

// --- ExfReInitializeRundownProtection ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfReInitializeRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfReInitializeRundownProtection_impl(RunRef)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfReInitializeRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfReInitializeRundownProtection_impl(RunRef)
}

// --- ExfRundownCompleted ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfRundownCompleted(RunRef: PEX_RUNDOWN_REF) {
    ExfRundownCompleted_impl(RunRef)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfRundownCompleted(RunRef: PEX_RUNDOWN_REF) {
    ExfRundownCompleted_impl(RunRef)
}

// --- ExfReleaseRundownProtection ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfReleaseRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfReleaseRundownProtection_impl(RunRef)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfReleaseRundownProtection(RunRef: PEX_RUNDOWN_REF) {
    ExfReleaseRundownProtection_impl(RunRef)
}

// --- ExfReleaseRundownProtectionEx ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfReleaseRundownProtectionEx(
    RunRef: PEX_RUNDOWN_REF,
    Count: ULONG,
) {
    ExfReleaseRundownProtectionEx_impl(RunRef, Count)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfReleaseRundownProtectionEx(
    RunRef: PEX_RUNDOWN_REF,
    Count: ULONG,
) {
    ExfReleaseRundownProtectionEx_impl(RunRef, Count)
}

// --- ExfWaitForRundownProtectionRelease ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfWaitForRundownProtectionRelease(RunRef: PEX_RUNDOWN_REF) {
    ExfWaitForRundownProtectionRelease_impl(RunRef)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfWaitForRundownProtectionRelease(RunRef: PEX_RUNDOWN_REF) {
    ExfWaitForRundownProtectionRelease_impl(RunRef)
}

// --- ExfAcquireRundownProtectionCacheAware ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfAcquireRundownProtectionCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) -> BOOLEAN {
    ExfAcquireRundownProtectionCacheAware_impl(RunRefCacheAware)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfAcquireRundownProtectionCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) -> BOOLEAN {
    ExfAcquireRundownProtectionCacheAware_impl(RunRefCacheAware)
}

// --- ExfAcquireRundownProtectionCacheAwareEx ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfAcquireRundownProtectionCacheAwareEx(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
    Count: ULONG,
) -> BOOLEAN {
    ExfAcquireRundownProtectionCacheAwareEx_impl(RunRefCacheAware, Count)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfAcquireRundownProtectionCacheAwareEx(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
    Count: ULONG,
) -> BOOLEAN {
    ExfAcquireRundownProtectionCacheAwareEx_impl(RunRefCacheAware, Count)
}

// --- ExfReleaseRundownProtectionCacheAware ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfReleaseRundownProtectionCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfReleaseRundownProtectionCacheAware_impl(RunRefCacheAware)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfReleaseRundownProtectionCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfReleaseRundownProtectionCacheAware_impl(RunRefCacheAware)
}

// --- ExfReleaseRundownProtectionCacheAwareEx ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfReleaseRundownProtectionCacheAwareEx(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
    Count: ULONG,
) {
    ExfReleaseRundownProtectionCacheAwareEx_impl(RunRefCacheAware, Count)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfReleaseRundownProtectionCacheAwareEx(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
    Count: ULONG,
) {
    ExfReleaseRundownProtectionCacheAwareEx_impl(RunRefCacheAware, Count)
}

// --- ExfWaitForRundownProtectionReleaseCacheAware ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfWaitForRundownProtectionReleaseCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfWaitForRundownProtectionReleaseCacheAware_impl(RunRefCacheAware)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfWaitForRundownProtectionReleaseCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfWaitForRundownProtectionReleaseCacheAware_impl(RunRefCacheAware)
}

// --- ExfRundownCompletedCacheAware ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfRundownCompletedCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfRundownCompletedCacheAware_impl(RunRefCacheAware)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfRundownCompletedCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfRundownCompletedCacheAware_impl(RunRefCacheAware)
}

// --- ExfReInitializeRundownProtectionCacheAware ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ExfReInitializeRundownProtectionCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfReInitializeRundownProtectionCacheAware_impl(RunRefCacheAware)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfReInitializeRundownProtectionCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExfReInitializeRundownProtectionCacheAware_impl(RunRefCacheAware)
}

// =============================================================================
// NTAPI (stdcall) functions — not FASTCALL, no cfg needed
// =============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExAllocateCacheAwareRundownProtection(
    PoolType: POOL_TYPE,
    Tag: ULONG,
) -> PEX_RUNDOWN_REF_CACHE_AWARE {
    let RunRefCacheAware = ExAllocatePoolWithTag(
        PoolType,
        core::mem::size_of::<EX_RUNDOWN_REF_CACHE_AWARE>(),
        Tag,
    ) as PEX_RUNDOWN_REF_CACHE_AWARE;
    if RunRefCacheAware.is_null() {
        return core::ptr::null_mut();
    }

    (*RunRefCacheAware).Number = KeNumberProcessors as ULONG;
    let RunRefSize = if KeNumberProcessors <= 1 {
        core::mem::size_of::<EX_RUNDOWN_REF>()
    } else {
        let Align = KeGetRecommendedSharedDataAlignment() as usize;
        Align
    };

    (*RunRefCacheAware).RunRefSize = RunRefSize as ULONG;

    let mut PoolToFree = ExAllocatePoolWithTag(
        PoolType,
        RunRefSize * ((*RunRefCacheAware).Number as usize),
        Tag,
    );
    if PoolToFree.is_null() {
        ExFreePoolWithTag(RunRefCacheAware as PVOID, Tag);
        return core::ptr::null_mut();
    }

    let RunRefs = if (*RunRefCacheAware).Number > 1 {
        let Align = KeGetRecommendedSharedDataAlignment() as usize;
        if (PoolToFree as usize) & (Align - 1) != 0 {
            ExFreePoolWithTag(PoolToFree, Tag);

            PoolToFree = ExAllocatePoolWithTag(
                PoolType,
                RunRefSize * (((*RunRefCacheAware).Number + 1) as usize),
                Tag,
            );
            if PoolToFree.is_null() {
                ExFreePoolWithTag(RunRefCacheAware as PVOID, Tag);
                return core::ptr::null_mut();
            }

            ALIGN_UP_BY(PoolToFree as usize, Align) as PVOID
        } else {
            PoolToFree
        }
    } else {
        PoolToFree
    };

    (*RunRefCacheAware).RunRefs = RunRefs as PEX_RUNDOWN_REF;
    (*RunRefCacheAware).PoolToFree = PoolToFree;

    if (*RunRefCacheAware).Number != 0 {
        for Count in 0..(*RunRefCacheAware).Number {
            let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Count);
            _ExInitializeRundownProtection(RunRef);
        }
    }

    RunRefCacheAware
}

#[no_mangle]
pub unsafe extern "system" fn ExFreeCacheAwareRundownProtection(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
) {
    ExFreePoolWithTag((*RunRefCacheAware).PoolToFree, 0);
    ExFreePoolWithTag(RunRefCacheAware as PVOID, 0);
}

#[no_mangle]
pub unsafe extern "system" fn ExInitializeRundownProtectionCacheAware(
    RunRefCacheAware: PEX_RUNDOWN_REF_CACHE_AWARE,
    Size: SIZE_T,
) {
    let mut Pool = ((RunRefCacheAware as usize) + core::mem::size_of::<EX_RUNDOWN_REF_CACHE_AWARE>()) as PVOID;

    let mut RunRefSize = core::mem::size_of::<EX_RUNDOWN_REF>();

    let Count = if Size == core::mem::size_of::<EX_RUNDOWN_REF_CACHE_AWARE>() + core::mem::size_of::<EX_RUNDOWN_REF>() {
        1
    } else {
        let Align = KeGetRecommendedSharedDataAlignment() as usize;
        RunRefSize = Align;
        ((Size - core::mem::size_of::<EX_RUNDOWN_REF_CACHE_AWARE>()) / Align) - 1
    };

    if Count != 1 {
        let Align = KeGetRecommendedSharedDataAlignment() as usize;
        Pool = ALIGN_UP_BY(Pool as usize, Align) as PVOID;
    }

    (*RunRefCacheAware).RunRefs = Pool as PEX_RUNDOWN_REF;
    (*RunRefCacheAware).RunRefSize = RunRefSize as ULONG;
    (*RunRefCacheAware).Number = Count as ULONG;
    (*RunRefCacheAware).PoolToFree = 0xBADCA11usize as PVOID;

    if (*RunRefCacheAware).Number != 0 {
        for Count in 0..(*RunRefCacheAware).Number {
            let RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Count);
            _ExInitializeRundownProtection(RunRef);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExSizeOfRundownProtectionCacheAware() -> SIZE_T {
    let Size = if KeNumberProcessors <= 1 {
        core::mem::size_of::<EX_RUNDOWN_REF>()
    } else {
        ((KeNumberProcessors + 1) as usize) * (KeGetRecommendedSharedDataAlignment() as usize)
    };

    Size + core::mem::size_of::<EX_RUNDOWN_REF_CACHE_AWARE>()
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ex/rundown.c
//  * PURPOSE:         Rundown and Cache-Aware Rundown Protection
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Thomas Weidenmueller
//  *                  Pierre Schweitzer
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*++
//  * @name ExfAcquireRundownProtection
//  * @implemented NT5.1
//  *
//  *     The ExfAcquireRundownProtection routine acquires rundown protection for
//  *     the specified descriptor.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @return TRUE if access to the protected structure was granted, FALSE otherwise.
//  *
//  * @remarks Callers of ExfAcquireRundownProtection can be running at any IRQL.
//  *
//  *--*/
// BOOLEAN
// FASTCALL
// ExfAcquireRundownProtection(IN PEX_RUNDOWN_REF RunRef)
// {
//     ULONG_PTR Value = RunRef->Count, NewValue;
//
//     /* Loop until successfully incremented the counter */
//     for (;;)
//     {
//         /* Make sure a rundown is not active */
//         if (Value & EX_RUNDOWN_ACTIVE) return FALSE;
//
//         /* Add a reference */
//         NewValue = Value + EX_RUNDOWN_COUNT_INC;
//
//         /* Change the value */
//         NewValue = ExpChangeRundown(RunRef, NewValue, Value);
//         if (NewValue == Value) return TRUE;
//
//         /* Update it */
//         Value = NewValue;
//     }
// }
//
// /*++
//  * @name ExfAcquireRundownProtectionEx
//  * @implemented NT5.2
//  *
//  *     The ExfAcquireRundownProtectionEx routine acquires multiple rundown
//  *     protection references for the specified descriptor.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @param Count
//  *         Number of times to reference the descriptor.
//  *
//  * @return TRUE if access to the protected structure was granted, FALSE otherwise.
//  *
//  * @remarks Callers of ExfAcquireRundownProtectionEx can be running at any IRQL.
//  *
//  *--*/
// BOOLEAN
// FASTCALL
// ExfAcquireRundownProtectionEx(IN PEX_RUNDOWN_REF RunRef,
//                               IN ULONG Count)
// {
//     ULONG_PTR Value = RunRef->Count, NewValue;
//
//     /* Loop until successfully incremented the counter */
//     for (;;)
//     {
//         /* Make sure a rundown is not active */
//         if (Value & EX_RUNDOWN_ACTIVE) return FALSE;
//
//         /* Add references */
//         NewValue = Value + EX_RUNDOWN_COUNT_INC * Count;
//
//         /* Change the value */
//         NewValue = ExpChangeRundown(RunRef, NewValue, Value);
//         if (NewValue == Value) return TRUE;
//
//         /* Update the value */
//         Value = NewValue;
//     }
// }
//
// /*++
//  * @name ExfInitializeRundownProtection
//  * @implemented NT5.1
//  *
//  *     The ExfInitializeRundownProtection routine initializes a rundown
//  *     protection descriptor.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfInitializeRundownProtection can be running at any IRQL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfInitializeRundownProtection(IN PEX_RUNDOWN_REF RunRef)
// {
//     /* Set the count to zero */
//     RunRef->Count = 0;
// }
//
// /*++
//  * @name ExfReInitializeRundownProtection
//  * @implemented NT5.1
//  *
//  *     The ExfReInitializeRundownProtection routine re-initializes a rundown
//  *     protection descriptor.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfReInitializeRundownProtection can be running at any IRQL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfReInitializeRundownProtection(IN PEX_RUNDOWN_REF RunRef)
// {
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT((RunRef->Count & EX_RUNDOWN_ACTIVE) != 0);
//
//     /* Reset the count */
//     ExpSetRundown(RunRef, 0);
// }
//
// /*++
//  * @name ExfRundownCompleted
//  * @implemented NT5.1
//  *
//  *     The ExfRundownCompleted routine completes the rundown of the specified
//  *     descriptor by setting the active bit.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfRundownCompleted must be running at IRQL <= APC_LEVEL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfRundownCompleted(IN PEX_RUNDOWN_REF RunRef)
// {
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT((RunRef->Count & EX_RUNDOWN_ACTIVE) != 0);
//
//     /* Mark the counter as active */
//     ExpSetRundown(RunRef, EX_RUNDOWN_ACTIVE);
// }
//
// /*++
//  * @name ExfReleaseRundownProtection
//  * @implemented NT5.1
//  *
//  *     The ExfReleaseRundownProtection routine releases the rundown protection
//  *     reference for the specified descriptor.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfReleaseRundownProtection can be running at any IRQL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfReleaseRundownProtection(IN PEX_RUNDOWN_REF RunRef)
// {
//     ULONG_PTR Value = RunRef->Count, NewValue;
//     PEX_RUNDOWN_WAIT_BLOCK WaitBlock;
//
//     /* Loop until successfully incremented the counter */
//     for (;;)
//     {
//         /* Check if rundown is not active */
//         if (!(Value & EX_RUNDOWN_ACTIVE))
//         {
//             /* Sanity check */
//             ASSERT((Value >= EX_RUNDOWN_COUNT_INC) || (KeNumberProcessors > 1));
//
//             /* Get the new value */
//             NewValue = Value - EX_RUNDOWN_COUNT_INC;
//
//             /* Change the value */
//             NewValue = ExpChangeRundown(RunRef, NewValue, Value);
//             if (NewValue == Value) break;
//
//             /* Update value */
//             Value = NewValue;
//         }
//         else
//         {
//             /* Get the wait block */
//             WaitBlock = (PEX_RUNDOWN_WAIT_BLOCK)(Value & ~EX_RUNDOWN_ACTIVE);
//             ASSERT((WaitBlock->Count > 0) || (KeNumberProcessors > 1));
//
//             /* Remove the one count */
//             if (!InterlockedDecrementSizeT(&WaitBlock->Count))
//             {
//                 /* We're down to 0 now, so signal the event */
//                 KeSetEvent(&WaitBlock->WakeEvent, IO_NO_INCREMENT, FALSE);
//             }
//
//             /* We're all done */
//             break;
//         }
//     }
// }
//
// /*++
//  * @name ExfReleaseRundownProtectionEx
//  * @implemented NT5.2
//  *
//  *     The ExfReleaseRundownProtectionEx routine releases multiple rundown
//  *     protection references for the specified descriptor.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @param Count
//  *         Number of times to dereference the descriptor.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfAcquireRundownProtectionEx can be running at any IRQL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfReleaseRundownProtectionEx(IN PEX_RUNDOWN_REF RunRef,
//                               IN ULONG Count)
// {
//     ULONG_PTR Value = RunRef->Count, NewValue;
//     PEX_RUNDOWN_WAIT_BLOCK WaitBlock;
//
//     /* Loop until successfully incremented the counter */
//     for (;;)
//     {
//         /* Check if rundown is not active */
//         if (!(Value & EX_RUNDOWN_ACTIVE))
//         {
//             /* Sanity check */
//             ASSERT((Value >= EX_RUNDOWN_COUNT_INC * Count) ||
//                    (KeNumberProcessors > 1));
//
//             /* Get the new value */
//             NewValue = Value - EX_RUNDOWN_COUNT_INC * Count;
//
//             /* Change the value */
//             NewValue = ExpChangeRundown(RunRef, NewValue, Value);
//             if (NewValue == Value) break;
//
//             /* Update value */
//             Value = NewValue;
//         }
//         else
//         {
//             /* Get the wait block */
//             WaitBlock = (PEX_RUNDOWN_WAIT_BLOCK)(Value & ~EX_RUNDOWN_ACTIVE);
//             ASSERT((WaitBlock->Count >= Count) || (KeNumberProcessors > 1));
//
//             /* Remove the counts */
//             if (InterlockedExchangeAddSizeT(&WaitBlock->Count,
//                                             -(LONG)Count) == (LONG)Count)
//             {
//                 /* We're down to 0 now, so signal the event */
//                 KeSetEvent(&WaitBlock->WakeEvent, IO_NO_INCREMENT, FALSE);
//             }
//
//             /* We're all done */
//             break;
//         }
//     }
// }
//
// /*++
//  * @name ExfWaitForRundownProtectionRelease
//  * @implemented NT5.1
//  *
//  *     The ExfWaitForRundownProtectionRelease routine waits until the specified
//  *     rundown descriptor has been released.
//  *
//  * @param RunRef
//  *        Pointer to a rundown reference descriptor.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfWaitForRundownProtectionRelease must be running
//  *          at IRQL <= APC_LEVEL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfWaitForRundownProtectionRelease(IN PEX_RUNDOWN_REF RunRef)
// {
//     ULONG_PTR Value, Count, NewValue;
//     EX_RUNDOWN_WAIT_BLOCK WaitBlock;
//     PEX_RUNDOWN_WAIT_BLOCK WaitBlockPointer;
//     PKEVENT Event;
//     PAGED_CODE();
//
//     /* Set the active bit */
//     Value = ExpChangeRundown(RunRef, EX_RUNDOWN_ACTIVE, 0);
//     if ((Value == 0) || (Value == EX_RUNDOWN_ACTIVE)) return;
//
//     /* No event for now */
//     Event = NULL;
//     WaitBlockPointer = (PEX_RUNDOWN_WAIT_BLOCK)((ULONG_PTR)&WaitBlock |
//                                                 EX_RUNDOWN_ACTIVE);
//
//     /* Start waitblock set loop */
//     for (;;)
//     {
//         /* Save the count */
//         Count = Value >> EX_RUNDOWN_COUNT_SHIFT;
//
//         /* If the count is over one and we don't have en event yet, create it */
//         if ((Count) && !(Event))
//         {
//             /* Initialize the event */
//             KeInitializeEvent(&WaitBlock.WakeEvent,
//                               SynchronizationEvent,
//                               FALSE);
//
//             /* Set the pointer */
//             Event = &WaitBlock.WakeEvent;
//         }
//
//         /* Set the count */
//         WaitBlock.Count = Count;
//
//         /* Now set the pointer */
//         NewValue = ExpChangeRundown(RunRef, (ULONG_PTR)WaitBlockPointer, Value);
//         if (NewValue == Value) break;
//
//         /* Loop again */
//         Value = NewValue;
//         ASSERT((Value & EX_RUNDOWN_ACTIVE) == 0);
//     }
//
//     /* If the count was 0, we're done */
//     if (!Count) return;
//
//     /* Wait for whoever needs to release to notify us */
//     KeWaitForSingleObject(Event, Executive, KernelMode, FALSE, NULL);
//     ASSERT(WaitBlock.Count == 0);
// }
//
// /*
//  * @implemented NT5.2
//  */
// BOOLEAN
// FASTCALL
// ExfAcquireRundownProtectionCacheAware(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware)
// {
//     PEX_RUNDOWN_REF RunRef;
//
//     RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
//     return _ExAcquireRundownProtection(RunRef);
// }
//
// /*
//  * @implemented NT5.2
//  */
// BOOLEAN
// FASTCALL
// ExfAcquireRundownProtectionCacheAwareEx(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware,
//                                         IN ULONG Count)
// {
//     PEX_RUNDOWN_REF RunRef;
//
//     RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
//     return ExfAcquireRundownProtectionEx(RunRef, Count);
// }
//
// /*
//  * @implemented NT5.2
//  */
// VOID
// FASTCALL
// ExfReleaseRundownProtectionCacheAware(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware)
// {
//     PEX_RUNDOWN_REF RunRef;
//
//     RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
//     _ExReleaseRundownProtection(RunRef);
// }
//
// /*
//  * @implemented NT5.2
//  */
// VOID
// FASTCALL
// ExfReleaseRundownProtectionCacheAwareEx(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware,
//                                         IN ULONG Count)
// {
//     PEX_RUNDOWN_REF RunRef;
//
//     RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, KeGetCurrentProcessorNumber());
//     ExfReleaseRundownProtectionEx(RunRef, Count);
// }
//
// /*
//  * @implemented NT5.2
//  */
// VOID
// FASTCALL
// ExfWaitForRundownProtectionReleaseCacheAware(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware)
// {
//     PEX_RUNDOWN_REF RunRef;
//     EX_RUNDOWN_WAIT_BLOCK WaitBlock;
//     PEX_RUNDOWN_WAIT_BLOCK WaitBlockPointer;
//     ULONG_PTR ProcCount, Current, Value, OldValue, TotalCount;
//
//     ProcCount = RunRefCacheAware->Number;
//     /* No proc, nothing to do */
//     if (ProcCount == 0)
//     {
//         return;
//     }
//
//     TotalCount = 0;
//     WaitBlock.Count = 0;
//     WaitBlockPointer = (PEX_RUNDOWN_WAIT_BLOCK)((ULONG_PTR)&WaitBlock |
//                                                 EX_RUNDOWN_ACTIVE);
//     /* We will check all our runrefs */
//     for (Current = 0; Current < ProcCount; ++Current)
//     {
//         /* Get the runref for the proc */
//         RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Current);
//         /* Loop for setting the wait block */
//         do
//         {
//             Value = RunRef->Count;
//             ASSERT((Value & EX_RUNDOWN_ACTIVE) == 0);
//
//             /* Remove old value and set our waitblock instead */
//             OldValue = ExpChangeRundown(RunRef, WaitBlockPointer, Value);
//             if (OldValue == Value)
//             {
//                 break;
//             }
//
//             Value = OldValue;
//         }
//         while (TRUE);
//
//         /* Count the deleted values */
//         TotalCount += Value;
//     }
//
//     /* Sanity check: we didn't overflow */
//     ASSERT((LONG_PTR)TotalCount >= 0);
//     if (TotalCount != 0)
//     {
//         /* Init the waitblock event */
//         KeInitializeEvent(&WaitBlock.WakeEvent,
//                           SynchronizationEvent,
//                           FALSE);
//
//         /* Do we have to wait? If so, go ahead! */
//         if (InterlockedExchangeAddSizeT(&WaitBlock.Count,
//                                         (LONG_PTR)TotalCount >> EX_RUNDOWN_COUNT_SHIFT) ==
//                                        -(LONG_PTR)(TotalCount >> EX_RUNDOWN_COUNT_SHIFT))
//         {
//             KeWaitForSingleObject(&WaitBlock.WakeEvent, Executive, KernelMode, FALSE, NULL);
//         }
//     }
// }
//
// /*
//  * @implemented NT5.2
//  */
// VOID
// FASTCALL
// ExfRundownCompletedCacheAware(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware)
// {
//     PEX_RUNDOWN_REF RunRef;
//     ULONG ProcCount, Current;
//
//     ProcCount = RunRefCacheAware->Number;
//     /* No proc, nothing to do */
//     if (ProcCount == 0)
//     {
//         return;
//     }
//
//     /* We will mark all our runrefs active */
//     for (Current = 0; Current < ProcCount; ++Current)
//     {
//         /* Get the runref for the proc */
//         RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Current);
//         ASSERT((RunRef->Count & EX_RUNDOWN_ACTIVE) != 0);
//
//         ExpSetRundown(RunRef, EX_RUNDOWN_ACTIVE);
//     }
// }
//
// /*
//  * @implemented NT5.2
//  */
// VOID
// FASTCALL
// ExfReInitializeRundownProtectionCacheAware(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware)
// {
//     PEX_RUNDOWN_REF RunRef;
//     ULONG ProcCount, Current;
//
//     ProcCount = RunRefCacheAware->Number;
//     /* No proc, nothing to do */
//     if (ProcCount == 0)
//     {
//         return;
//     }
//
//     /* We will mark all our runrefs inactive */
//     for (Current = 0; Current < ProcCount; ++Current)
//     {
//         /* Get the runref for the proc */
//         RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Current);
//         ASSERT((RunRef->Count & EX_RUNDOWN_ACTIVE) != 0);
//
//         ExpSetRundown(RunRef, 0);
//     }
// }
//
// /*
//  * @implemented NT5.2
//  */
// PEX_RUNDOWN_REF_CACHE_AWARE
// NTAPI
// ExAllocateCacheAwareRundownProtection(IN POOL_TYPE PoolType,
//                                       IN ULONG Tag)
// {
//     PEX_RUNDOWN_REF RunRef;
//     PVOID PoolToFree, RunRefs;
//     ULONG RunRefSize, Count, Align;
//     PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware;
//
//     PAGED_CODE();
//
//     /* Allocate the master structure */
//     RunRefCacheAware = ExAllocatePoolWithTag(PoolType, sizeof(EX_RUNDOWN_REF_CACHE_AWARE), Tag);
//     if (RunRefCacheAware == NULL)
//     {
//         return NULL;
//     }
//
//     /* Compute the size of each runref */
//     RunRefCacheAware->Number = KeNumberProcessors;
//     if (KeNumberProcessors <= 1)
//     {
//         RunRefSize = sizeof(EX_RUNDOWN_REF);
//     }
//     else
//     {
//         Align = KeGetRecommendedSharedDataAlignment();
//         RunRefSize = Align;
//         ASSERT((RunRefSize & (RunRefSize - 1)) == 0);
//     }
//
//     /* It must at least hold a EX_RUNDOWN_REF structure */
//     ASSERT(sizeof(EX_RUNDOWN_REF) <= RunRefSize);
//     RunRefCacheAware->RunRefSize = RunRefSize;
//
//     /* Allocate our runref pool */
//     PoolToFree = ExAllocatePoolWithTag(PoolType, RunRefSize * RunRefCacheAware->Number, Tag);
//     if (PoolToFree == NULL)
//     {
//         ExFreePoolWithTag(RunRefCacheAware, Tag);
//         return NULL;
//     }
//
//     /* On SMP, check for alignment */
//     if (RunRefCacheAware->Number > 1 && (ULONG_PTR)PoolToFree & (Align - 1))
//     {
//         /* Not properly aligned, do it again! */
//         ExFreePoolWithTag(PoolToFree, Tag);
//
//         /* Allocate a bigger buffer to be able to align properly */
//         PoolToFree = ExAllocatePoolWithTag(PoolType, RunRefSize * (RunRefCacheAware->Number + 1), Tag);
//         if (PoolToFree == NULL)
//         {
//             ExFreePoolWithTag(RunRefCacheAware, Tag);
//             return NULL;
//         }
//
//         RunRefs = (PVOID)ALIGN_UP_BY(PoolToFree, Align);
//     }
//     else
//     {
//         RunRefs = PoolToFree;
//     }
//
//     RunRefCacheAware->RunRefs = RunRefs;
//     RunRefCacheAware->PoolToFree = PoolToFree;
//
//     /* And initialize runref */
//     if (RunRefCacheAware->Number != 0)
//     {
//         for (Count = 0; Count < RunRefCacheAware->Number; ++Count)
//         {
//             RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Count);
//             _ExInitializeRundownProtection(RunRef);
//         }
//     }
//
//     return RunRefCacheAware;
// }
//
// /*
//  * @implemented NT5.2
//  */
// VOID
// NTAPI
// ExFreeCacheAwareRundownProtection(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware)
// {
//     PAGED_CODE();
//
//     /*
//      * This is to be called for RunRefCacheAware that were allocated with
//      * ExAllocateCacheAwareRundownProtection and not for user-allocated
//      * ones
//      */
//     ASSERT(RunRefCacheAware->PoolToFree != (PVOID)0xBADCA11);
//
//     /* We don't know the tag that as used for allocation */
//     ExFreePoolWithTag(RunRefCacheAware->PoolToFree, 0);
//     ExFreePoolWithTag(RunRefCacheAware, 0);
// }
//
// /*
//  * @implemented NT5.2
//  */
// VOID
// NTAPI
// ExInitializeRundownProtectionCacheAware(IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware,
//                                         IN SIZE_T Size)
// {
//     PVOID Pool;
//     PEX_RUNDOWN_REF RunRef;
//     ULONG Count, RunRefSize, Align;
//
//     PAGED_CODE();
//
//     /* Get the user allocate pool for runrefs */
//     Pool = (PVOID)((ULONG_PTR)RunRefCacheAware + sizeof(EX_RUNDOWN_REF_CACHE_AWARE));
//
//     /* By default a runref is structure-sized */
//     RunRefSize = sizeof(EX_RUNDOWN_REF);
//
//     /*
//      * If we just have enough room for a single runref, deduce were on a single
//      * processor machine
//      */
//     if (Size == sizeof(EX_RUNDOWN_REF_CACHE_AWARE) + sizeof(EX_RUNDOWN_REF))
//     {
//         Count = 1;
//     }
//     else
//     {
//         /* Get alignment constraint */
//         Align = KeGetRecommendedSharedDataAlignment();
//
//         /* How many runrefs given the alignment? */
//         RunRefSize = Align;
//         Count = ((Size - sizeof(EX_RUNDOWN_REF_CACHE_AWARE)) / Align) - 1;
//         Pool = (PVOID)ALIGN_UP_BY(Pool, Align);
//     }
//
//     /* Initialize the structure */
//     RunRefCacheAware->RunRefs = Pool;
//     RunRefCacheAware->RunRefSize = RunRefSize;
//     RunRefCacheAware->Number = Count;
//
//     /* There is no allocated pool! */
//     RunRefCacheAware->PoolToFree = (PVOID)0xBADCA11u;
//
//     /* Initialize runref */
//     if (RunRefCacheAware->Number != 0)
//     {
//         for (Count = 0; Count < RunRefCacheAware->Number; ++Count)
//         {
//             RunRef = ExGetRunRefForGivenProcessor(RunRefCacheAware, Count);
//             _ExInitializeRundownProtection(RunRef);
//         }
//     }
// }
//
// /*
//  * @implemented NT5.2
//  */
// SIZE_T
// NTAPI
// ExSizeOfRundownProtectionCacheAware(VOID)
// {
//     SIZE_T Size;
//
//     PAGED_CODE();
//
//     /* Compute the needed size for runrefs */
//     if (KeNumberProcessors <= 1)
//     {
//         Size = sizeof(EX_RUNDOWN_REF);
//     }
//     else
//     {
//         /* We +1, to have enough room for alignment */
//         Size = (KeNumberProcessors + 1) * KeGetRecommendedSharedDataAlignment();
//     }
//
//     /* Return total size (master structure and runrefs) */
//     return Size + sizeof(EX_RUNDOWN_REF_CACHE_AWARE);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
