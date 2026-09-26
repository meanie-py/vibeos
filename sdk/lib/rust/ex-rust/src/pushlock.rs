use ntkrnl_sys::globals::*;
// Cross-crate ke/mm/ARM3 globals+recorders as link symbols (all #[no_mangle]
// in ntoskrnl-rust). crate-split Phase 1b.
extern "C" {
    fn lockdep_check_park(thread: usize, lock: usize, wanted_mode: u32);
    fn synctrace_pl_park(object: *const core::ffi::c_void, thread: *const core::ffi::c_void);
    fn synctrace_pl_satisfied(object: *const core::ffi::c_void);
    fn synctrace_pl_wake(object: *const core::ffi::c_void, thread: *const core::ffi::c_void);
}
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::{
    EX_PUSH_LOCK, PEX_PUSH_LOCK, EX_PUSH_LOCK_WAIT_BLOCK, PEX_PUSH_LOCK_WAIT_BLOCK,
    PLARGE_INTEGER,
    KeInitializeEvent, KeWaitForSingleObject, KeSignalGateBoostPriority,
    KeInitializeGate, KeWaitForGate, KeSetEventBoostPriority,
    PKGATE,
    _KWAIT_REASON_WrPushLock,
};

/// Emit a `#[no_mangle]` function twice: once with `extern "fastcall"` for i386,
/// once with `extern "C"` for amd64/arm64. All pushlock Exf*/ExBlock*/ExWait*
/// functions are FASTCALL in the C source.
macro_rules! fastcall_export {
    (
        pub unsafe fn $name:ident ( $($param:ident : $pty:ty),* $(,)? ) $(-> $ret:ty)? $body:block
    ) => {
        #[cfg(target_arch = "x86")]
        #[no_mangle]
        pub unsafe extern "fastcall" fn $name ( $($param : $pty),* ) $(-> $ret)? $body

        #[cfg(not(target_arch = "x86"))]
        #[no_mangle]
        pub unsafe extern "C" fn $name ( $($param : $pty),* ) $(-> $ret)? $body
    };
}

// Push lock constants (from extypes.h).
// Type must match bindgen's ULONG_PTR: c_ulonglong (u64) on amd64/arm64, c_ulong (u32) on i386.
#[cfg(target_pointer_width = "64")]
type PushLockVal = u64;
#[cfg(target_pointer_width = "32")]
type PushLockVal = u32;

const EX_PUSH_LOCK_LOCK: PushLockVal = 0x1;
const EX_PUSH_LOCK_WAITING: PushLockVal = 0x2;
const EX_PUSH_LOCK_WAKING: PushLockVal = 0x4;
const EX_PUSH_LOCK_MULTIPLE_SHARED: PushLockVal = 0x8;
const EX_PUSH_LOCK_PTR_BITS: PushLockVal = 0xF;
const EX_PUSH_LOCK_FLAGS_EXCLUSIVE: i32 = 1;
const EX_PUSH_LOCK_FLAGS_WAIT_V: u32 = 1;
const EX_PUSH_LOCK_FLAGS_WAIT: i32 = 2;

extern "C" {
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

static mut ExPushLockSpinCount: ULONG = 0;

#[cfg(target_pointer_width = "64")]
unsafe fn InterlockedAndPointer(ptr: *mut PVOID, val: usize) -> usize {
    core::sync::atomic::AtomicUsize::from_ptr(ptr as *mut usize)
        .fetch_and(val, core::sync::atomic::Ordering::SeqCst)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe fn InterlockedAndPointer(ptr: *mut PVOID, val: usize) -> usize {
    core::sync::atomic::AtomicUsize::from_ptr(ptr as *mut usize)
        .fetch_and(val, core::sync::atomic::Ordering::SeqCst)
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitializePushLocks() {
    #[cfg(feature = "smp")]
    {
        if KeNumberProcessors > 1 {
            ExPushLockSpinCount = 1024;
        }
    }
}

/// EX_PUSH_LOCK_WAIT_BLOCK with the C `__ALIGNED(16)` restored. The bindgen
/// type LOST the alignment attribute, so stack-allocated wait blocks landed on
/// 4-byte boundaries — and the pushlock word ORs 4 FLAG BITS into the block
/// pointer. A block at addr%16==4 makes bit2 of the pointer read as the WAKING
/// flag: every releaser then skips ExfWakePushLock ("already waking") and the
/// waiter parks forever (the i386 "BAD gate"/lost-wakeup session bug). Root
/// cause of the 2026-07-06 winlogon/LSASS boot wedge; see field manual.
#[repr(C, align(16))]
pub(crate) struct AlignedPlWaitBlock(pub(crate) EX_PUSH_LOCK_WAIT_BLOCK);

unsafe fn ExfWakePushLock(PushLock: PEX_PUSH_LOCK, mut OldValue: EX_PUSH_LOCK) {
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut PreviousWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut FirstWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut LastWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut WaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut OldIrql: KIRQL;

    loop {
        // C (ntoskrnl/ex/pushlock.c:87): `ASSERT(!OldValue.MultipleShared)`. The getter
        // returns ULONG_PTR, so Rust's `!` is bitwise and the test could never be zero.
        ntkrnl_sys::ASSERT!(OldValue.__bindgen_anon_1.__bindgen_anon_1.MultipleShared() == 0);

        while OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0 {
            NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_WAKING;

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() == 0);
            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0);
            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0);

            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                return;
            }

            OldValue = NewValue;
        }

        FirstWaitBlock = (OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_PTR_BITS) as PEX_PUSH_LOCK_WAIT_BLOCK;
        WaitBlock = FirstWaitBlock;

        loop {
            LastWaitBlock = (*WaitBlock).Last;

            if !LastWaitBlock.is_null() {
                WaitBlock = LastWaitBlock;
                break;
            }

            PreviousWaitBlock = WaitBlock;
            WaitBlock = (*WaitBlock).Next;
            (*WaitBlock).Previous = PreviousWaitBlock;
        }

        PreviousWaitBlock = (*WaitBlock).Previous;
        if ((*WaitBlock).Flags & EX_PUSH_LOCK_FLAGS_EXCLUSIVE == 0) || PreviousWaitBlock.is_null() {
            NewValue.__bindgen_anon_1.Value = 0;
            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() == 0);

            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                break;
            }

            OldValue = NewValue;
        } else {
            (*FirstWaitBlock).Last = PreviousWaitBlock;
            (*WaitBlock).Previous = core::ptr::null_mut();

            ntkrnl_sys::ASSERT!(FirstWaitBlock != WaitBlock);
            ntkrnl_sys::ASSERT!((*PushLock).__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0);

            InterlockedAndPointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Value) as *mut PVOID,
                (!EX_PUSH_LOCK_WAKING) as usize,
            );

            break;
        }
    }

    OldIrql = DISPATCH_LEVEL as KIRQL;
    if !(*WaitBlock).Previous.is_null() {
        KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    }

    loop {
        PreviousWaitBlock = (*WaitBlock).Previous;

        ntkrnl_sys::ASSERT!((*WaitBlock).Signaled == 0);

        // The C's `#if DBG` -- and this tree builds the C with DBG=1. Gated on
        // cfg(debug_assertions) (off in release) while the waiters' ASSERT!s on
        // Signaled are live, every contended pushlock wait reported
        // "ASSERT FAILED: (*WaitBlock).Signaled != 0" although the wake was correct.
        (*WaitBlock).Signaled = 1u8;

        if core::sync::atomic::AtomicI32::from_ptr(core::ptr::addr_of_mut!((*WaitBlock).Flags))
            .fetch_and(!((1i32) << 1), core::sync::atomic::Ordering::SeqCst)
            & ((1i32) << 1)
            == 0
        {
            // synctrace §3: one waiter woken on this pushlock.
            synctrace_pl_wake(
                PushLock as _, ntkrnl_sys::mm::KeGetCurrentThread() as _);
            KeSignalGateBoostPriority(core::ptr::addr_of_mut!((*WaitBlock).__bindgen_anon_1.WakeGate) as PKGATE);
        }

        WaitBlock = PreviousWaitBlock;
        if WaitBlock.is_null() {
            break;
        }
    }

    if OldIrql != DISPATCH_LEVEL as KIRQL {
        KeLowerIrql(OldIrql);
    }
}

unsafe fn ExpOptimizePushLockList(PushLock: PEX_PUSH_LOCK, mut OldValue: EX_PUSH_LOCK) {
    let mut WaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut LastWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut PreviousWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut FirstWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();

    loop {
        if OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0 {
            ExfWakePushLock(PushLock, OldValue);
            break;
        }

        WaitBlock = (OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_PTR_BITS) as PEX_PUSH_LOCK_WAIT_BLOCK;

        FirstWaitBlock = WaitBlock;
        loop {
            LastWaitBlock = (*WaitBlock).Last;
            if !LastWaitBlock.is_null() {
                (*FirstWaitBlock).Last = LastWaitBlock;
                break;
            }

            PreviousWaitBlock = WaitBlock;
            WaitBlock = (*WaitBlock).Next;
            (*WaitBlock).Previous = PreviousWaitBlock;
        }

        NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_WAKING;

        ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0);
        ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() == 0);

        NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
            NewValue.__bindgen_anon_1.Ptr,
            OldValue.__bindgen_anon_1.Ptr,
        );

        if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
            break;
        }

        OldValue = NewValue;
    }
}

unsafe fn ExTimedWaitForUnblockPushLock(
    PushLock: PEX_PUSH_LOCK,
    WaitBlock: PVOID,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*(WaitBlock as PEX_PUSH_LOCK_WAIT_BLOCK)).__bindgen_anon_1.WakeEvent),
        0u32,
        0u8,
    );

    #[cfg(feature = "smp")]
    {
        if ExPushLockSpinCount != 0 {
            let mut i = ExPushLockSpinCount;

            loop {
                if (*(core::ptr::addr_of!((*(WaitBlock as PEX_PUSH_LOCK_WAIT_BLOCK)).Flags)
                    as *const LONG as *const core::sync::atomic::AtomicI32))
                    .load(core::sync::atomic::Ordering::SeqCst)
                    & EX_PUSH_LOCK_WAITING as LONG
                    == 0
                {
                    return STATUS_SUCCESS;
                }

                core::hint::spin_loop();
                i -= 1;
                if i == 0 {
                    break;
                }
            }
        }
    }

    if core::sync::atomic::AtomicI32::from_ptr(core::ptr::addr_of_mut!(
        (*(WaitBlock as PEX_PUSH_LOCK_WAIT_BLOCK)).Flags
    ))
    .fetch_and(!((1i32) << EX_PUSH_LOCK_FLAGS_WAIT_V), core::sync::atomic::Ordering::SeqCst)
        & ((1i32) << EX_PUSH_LOCK_FLAGS_WAIT_V)
        != 0
    {
        // synctrace §3: event-path block/wake, keyed on the PUSHLOCK address.
        synctrace_pl_park(
            PushLock as _, ntkrnl_sys::mm::KeGetCurrentThread() as _);
        Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*(WaitBlock as PEX_PUSH_LOCK_WAIT_BLOCK)).__bindgen_anon_1.WakeEvent)
                as PVOID,
            19u32,
            0i8,
            0u8,
            Timeout,
        );
        synctrace_pl_satisfied(PushLock as _);
        if Status != STATUS_SUCCESS {
            ExfUnblockPushLock(PushLock, WaitBlock);
        }
    } else {
        Status = STATUS_SUCCESS;
    }

    Status
}

fastcall_export! {
    pub unsafe fn ExWaitForUnblockPushLock(PushLock: PEX_PUSH_LOCK, WaitBlock: PVOID) {
        ExTimedWaitForUnblockPushLock(PushLock, WaitBlock, core::ptr::null_mut());
    }
}

fastcall_export! {
    pub unsafe fn ExBlockPushLock(PushLock: PEX_PUSH_LOCK, pWaitBlock: PVOID) {
    let WaitBlock = pWaitBlock as PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut OldValue: EX_PUSH_LOCK;

    ntkrnl_sys::ASSERT!((pWaitBlock as ULONG_PTR & 0xF) == 0);

    (*WaitBlock).Flags = EX_PUSH_LOCK_FLAGS_WAIT;

    OldValue = *PushLock;

    loop {
        (*WaitBlock).Next = OldValue.__bindgen_anon_1.Ptr as PEX_PUSH_LOCK_WAIT_BLOCK;

        NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
            WaitBlock as PVOID,
            OldValue.__bindgen_anon_1.Ptr,
        );
        if OldValue.__bindgen_anon_1.Ptr == NewValue.__bindgen_anon_1.Ptr {
            break;
        }

        OldValue = NewValue;
    }
    }
}

fastcall_export! {
    pub unsafe fn ExfAcquirePushLockExclusive(PushLock: PEX_PUSH_LOCK) {
    let mut OldValue = *PushLock;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut TempValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut NeedWake: BOOLEAN;
    let mut Block: AlignedPlWaitBlock = AlignedPlWaitBlock(core::mem::zeroed());
    let WaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK = core::ptr::addr_of_mut!(Block.0);

    loop {
        if OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0 {
            NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value | EX_PUSH_LOCK_LOCK;
            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0);

            if InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            ) != OldValue.__bindgen_anon_1.Ptr
            {
                OldValue = *PushLock;
                continue;
            }

            break;
        } else {
            (*WaitBlock).Flags = EX_PUSH_LOCK_FLAGS_EXCLUSIVE | EX_PUSH_LOCK_FLAGS_WAIT;
            (*WaitBlock).Previous = core::ptr::null_mut();
            NeedWake = 0u8;

            if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0 {
                (*WaitBlock).Last = core::ptr::null_mut();
                (*WaitBlock).ShareCount = 0;
                (*WaitBlock).Next =
                    (OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_PTR_BITS) as PEX_PUSH_LOCK_WAIT_BLOCK;

                NewValue.__bindgen_anon_1.Value = (OldValue.__bindgen_anon_1.Value & EX_PUSH_LOCK_MULTIPLE_SHARED)
                    | EX_PUSH_LOCK_LOCK
                    | EX_PUSH_LOCK_WAKING
                    | EX_PUSH_LOCK_WAITING
                    | (WaitBlock as PushLockVal);

                if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waking() == 0 {
                    NeedWake = 1u8;
                }
            } else {
                (*WaitBlock).Last = WaitBlock;
                (*WaitBlock).ShareCount = OldValue.__bindgen_anon_1.__bindgen_anon_1.Shared() as LONG;

                if OldValue.__bindgen_anon_1.__bindgen_anon_1.Shared() > 1 {
                    NewValue.__bindgen_anon_1.Value = EX_PUSH_LOCK_MULTIPLE_SHARED
                        | EX_PUSH_LOCK_LOCK
                        | EX_PUSH_LOCK_WAITING
                        | (WaitBlock as PushLockVal);
                } else {
                    (*WaitBlock).ShareCount = 0;
                    NewValue.__bindgen_anon_1.Value =
                        EX_PUSH_LOCK_LOCK | EX_PUSH_LOCK_WAITING | (WaitBlock as PushLockVal);
                }
            }

            // DBG bookkeeping, unconditional for the same reason as the waker's
            // `Signaled = 1`: the ASSERT!s that read it are live.
            {
                (*WaitBlock).Signaled = 0;
                (*WaitBlock).OldValue = OldValue;
                (*WaitBlock).NewValue = NewValue;
                (*WaitBlock).PushLock = PushLock;
            }

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0);
            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0);

            TempValue = NewValue;
            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Value != OldValue.__bindgen_anon_1.Value {
                OldValue = *PushLock;
                continue;
            }

            if NeedWake != 0 {
                ExpOptimizePushLockList(PushLock, TempValue);
            }

            KeInitializeGate(core::ptr::addr_of_mut!((*WaitBlock).__bindgen_anon_1.WakeGate));

            #[cfg(feature = "smp")]
            {
                if ExPushLockSpinCount != 0 {
                    let mut i = ExPushLockSpinCount;

                    loop {
                        if (*(core::ptr::addr_of!((*WaitBlock).Flags) as *const LONG
                            as *const core::sync::atomic::AtomicI32))
                            .load(core::sync::atomic::Ordering::SeqCst)
                            & EX_PUSH_LOCK_WAITING as LONG
                            == 0
                        {
                            break;
                        }

                        core::hint::spin_loop();
                        i -= 1;
                        if i == 0 {
                            break;
                        }
                    }
                }
            }

            if core::sync::atomic::AtomicI32::from_ptr(core::ptr::addr_of_mut!((*WaitBlock).Flags))
            .fetch_and(!((1i32) << 1), core::sync::atomic::Ordering::SeqCst)
                & ((1i32) << 1)
                != 0
            {
                // lockdep §4: about to block EXCLUSIVE — if we already hold
                // this pushlock, that's a self-deadlock (records, print-free).
                #[cfg(feature = "lockdep")]
                lockdep_check_park(
                    ntkrnl_sys::mm::KeGetCurrentThread() as usize, PushLock as usize, 2);
                // synctrace §3: account the block/wake on the PUSHLOCK address
                // (the wake gate is a per-wait stack object — useless as a key).
                synctrace_pl_park(
                    PushLock as _, ntkrnl_sys::mm::KeGetCurrentThread() as _);
                KeWaitForGate(
                    core::ptr::addr_of_mut!((*WaitBlock).__bindgen_anon_1.WakeGate),
                    _KWAIT_REASON_WrPushLock,
                    0i8,
                );
                synctrace_pl_satisfied(PushLock as _);
                ntkrnl_sys::ASSERT!((*WaitBlock).Signaled != 0);
            }

            ntkrnl_sys::ASSERT!((*WaitBlock).ShareCount == 0);

            OldValue = NewValue;
        }
    }
    }
}

fastcall_export! {
    pub unsafe fn ExfAcquirePushLockShared(PushLock: PEX_PUSH_LOCK) {
    let mut OldValue = *PushLock;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut NeedWake: BOOLEAN;
    let mut Block: AlignedPlWaitBlock = AlignedPlWaitBlock(core::mem::zeroed());
    let WaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK = core::ptr::addr_of_mut!(Block.0);

    loop {
        if (OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0)
            || ((OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() == 0)
                && (OldValue.__bindgen_anon_1.__bindgen_anon_1.Shared() > 0))
        {
            if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() == 0 {
                NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value | EX_PUSH_LOCK_LOCK;
                NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Shared(NewValue.__bindgen_anon_1.__bindgen_anon_1.Shared() + 1);
            } else {
                NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value | EX_PUSH_LOCK_LOCK;
            }

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0);

            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Value != OldValue.__bindgen_anon_1.Value {
                OldValue = *PushLock;
                continue;
            }

            break;
        } else {
            (*WaitBlock).Flags = EX_PUSH_LOCK_FLAGS_WAIT;
            (*WaitBlock).ShareCount = 0;
            NeedWake = 0u8;
            (*WaitBlock).Previous = core::ptr::null_mut();

            if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0 {
                (*WaitBlock).Next =
                    (OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_PTR_BITS) as PEX_PUSH_LOCK_WAIT_BLOCK;
                (*WaitBlock).Last = core::ptr::null_mut();

                NewValue.__bindgen_anon_1.Value = (OldValue.__bindgen_anon_1.Value
                    & (EX_PUSH_LOCK_MULTIPLE_SHARED | EX_PUSH_LOCK_LOCK))
                    | EX_PUSH_LOCK_WAKING
                    | EX_PUSH_LOCK_WAITING
                    | (WaitBlock as PushLockVal);

                if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waking() == 0 {
                    NeedWake = 1u8;
                }
            } else {
                (*WaitBlock).Last = WaitBlock;

                NewValue.__bindgen_anon_1.Value = (OldValue.__bindgen_anon_1.Value & EX_PUSH_LOCK_PTR_BITS)
                    | EX_PUSH_LOCK_WAITING
                    | (WaitBlock as PushLockVal);
            }

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0);

            // DBG bookkeeping, unconditional for the same reason as the waker's
            // `Signaled = 1`: the ASSERT!s that read it are live.
            {
                (*WaitBlock).Signaled = 0;
                (*WaitBlock).OldValue = OldValue;
                (*WaitBlock).NewValue = NewValue;
                (*WaitBlock).PushLock = PushLock;
            }

            // The CAS OVERWRITES NewValue with the value it found, so after it runs
            // NewValue no longer holds the list head we just published -- it holds the
            // previous one. ExpOptimizePushLockList was therefore walking the wrong
            // chain. Keep the written value aside first. The exclusive twin above
            // already does this. Upstream-Commit: 4acacdb7e90
            let TempValue = NewValue;
            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Ptr != OldValue.__bindgen_anon_1.Ptr {
                OldValue = *PushLock;
                continue;
            }

            if NeedWake != 0 {
                ExpOptimizePushLockList(PushLock, TempValue);
            }

            KeInitializeGate(core::ptr::addr_of_mut!((*WaitBlock).__bindgen_anon_1.WakeGate));

            #[cfg(feature = "smp")]
            {
                if ExPushLockSpinCount != 0 {
                    let mut i = ExPushLockSpinCount;

                    loop {
                        if (*(core::ptr::addr_of!((*WaitBlock).Flags) as *const LONG
                            as *const core::sync::atomic::AtomicI32))
                            .load(core::sync::atomic::Ordering::SeqCst)
                            & EX_PUSH_LOCK_WAITING as LONG
                            == 0
                        {
                            break;
                        }

                        core::hint::spin_loop();
                        i -= 1;
                        if i == 0 {
                            break;
                        }
                    }
                }
            }

            if core::sync::atomic::AtomicI32::from_ptr(core::ptr::addr_of_mut!((*WaitBlock).Flags))
            .fetch_and(!((1i32) << 1), core::sync::atomic::Ordering::SeqCst)
                & ((1i32) << 1)
                != 0
            {
                // lockdep §4: about to block SHARED — if we already hold this
                // pushlock, that's a self-deadlock (records, print-free). This
                // is the Mm WS-pushlock wedge: excl teardown -> fault -> shared.
                #[cfg(feature = "lockdep")]
                lockdep_check_park(
                    ntkrnl_sys::mm::KeGetCurrentThread() as usize, PushLock as usize, 1);
                // synctrace §3: account the block/wake on the PUSHLOCK address.
                synctrace_pl_park(
                    PushLock as _, ntkrnl_sys::mm::KeGetCurrentThread() as _);
                KeWaitForGate(
                    core::ptr::addr_of_mut!((*WaitBlock).__bindgen_anon_1.WakeGate),
                    _KWAIT_REASON_WrPushLock,
                    0i8,
                );
                synctrace_pl_satisfied(PushLock as _);
                ntkrnl_sys::ASSERT!((*WaitBlock).Signaled != 0);
            }

            ntkrnl_sys::ASSERT!((*WaitBlock).ShareCount == 0);
        }
    }
    }
}

fastcall_export! {
    pub unsafe fn ExfReleasePushLock(PushLock: PEX_PUSH_LOCK) {
    let mut OldValue = *PushLock;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut WakeValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut WaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut LastWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;

    ntkrnl_sys::ASSERT!(OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0);

    loop {
        if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() == 0 {
            if OldValue.__bindgen_anon_1.__bindgen_anon_1.Shared() > 1 {
                NewValue = OldValue;
                NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Shared(NewValue.__bindgen_anon_1.__bindgen_anon_1.Shared() - 1);
            } else {
                NewValue.__bindgen_anon_1.Value = 0;
            }

            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                return;
            }

            OldValue = NewValue;
        } else {
            if OldValue.__bindgen_anon_1.__bindgen_anon_1.MultipleShared() != 0 {
                WaitBlock =
                    (OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_PTR_BITS) as PEX_PUSH_LOCK_WAIT_BLOCK;

                loop {
                    LastWaitBlock = (*WaitBlock).Last;

                    if !LastWaitBlock.is_null() {
                        WaitBlock = LastWaitBlock;
                        break;
                    }

                    WaitBlock = (*WaitBlock).Next;
                }

                if (*WaitBlock).ShareCount > 0 {
                    ntkrnl_sys::ASSERT!((*WaitBlock).Flags & EX_PUSH_LOCK_FLAGS_EXCLUSIVE != 0);

                    if InterlockedDecrement(core::ptr::addr_of_mut!((*WaitBlock).ShareCount))
                        > 0
                    {
                        return;
                    }
                }
            }

            loop {
                if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0 {
                    NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value;
                    NewValue.__bindgen_anon_1.__bindgen_anon_1.set_MultipleShared(0);
                    NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Locked(0);

                    ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0
                            && NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0
                            && NewValue.__bindgen_anon_1.__bindgen_anon_1.MultipleShared() == 0);

                    NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                        core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                        NewValue.__bindgen_anon_1.Ptr,
                        OldValue.__bindgen_anon_1.Ptr,
                    );
                    if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                        return;
                    }
                } else {
                    NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value;
                    NewValue.__bindgen_anon_1.__bindgen_anon_1.set_MultipleShared(0);
                    NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Locked(0);
                    NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Waking(1);

                    ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0
                            && NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0
                            && NewValue.__bindgen_anon_1.__bindgen_anon_1.MultipleShared() == 0);

                    WakeValue = NewValue;
                    NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                        core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                        NewValue.__bindgen_anon_1.Ptr,
                        OldValue.__bindgen_anon_1.Ptr,
                    );
                    if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                        /* The write was successful. The pushlock is Unlocked and Waking */
                        ExfWakePushLock(PushLock, WakeValue);
                        return;
                    }

                    // The CMPXCHG returned the CURRENT lock value. Retrying without adopting it
                    // re-compares against a stale OldValue every time, so a contended release can
                    // spin forever. Upstream-Commit: f42ee0e23bb
                    OldValue = NewValue;
                    continue;
                }
            }
        }
    }
    }
}

fastcall_export! {
    pub unsafe fn ExfReleasePushLockShared(PushLock: PEX_PUSH_LOCK) {
    let mut OldValue = *PushLock;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut WakeValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut WaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut LastWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;

    while OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() == 0 {
        if OldValue.__bindgen_anon_1.__bindgen_anon_1.Shared() > 1 {
            NewValue = OldValue;
            NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Shared(NewValue.__bindgen_anon_1.__bindgen_anon_1.Shared() - 1);
        } else {
            NewValue.__bindgen_anon_1.Value = 0;
        }

        NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
            NewValue.__bindgen_anon_1.Ptr,
            OldValue.__bindgen_anon_1.Ptr,
        );
        if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
            return;
        }

        OldValue = NewValue;
    }

    if OldValue.__bindgen_anon_1.__bindgen_anon_1.MultipleShared() != 0 {
        WaitBlock = (OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_PTR_BITS) as PEX_PUSH_LOCK_WAIT_BLOCK;

        loop {
            LastWaitBlock = (*WaitBlock).Last;

            if !LastWaitBlock.is_null() {
                WaitBlock = LastWaitBlock;
                break;
            }

            WaitBlock = (*WaitBlock).Next;
        }

        ntkrnl_sys::ASSERT!((*WaitBlock).ShareCount > 0);
        ntkrnl_sys::ASSERT!((*WaitBlock).Flags & EX_PUSH_LOCK_FLAGS_EXCLUSIVE != 0);

        if InterlockedDecrement(core::ptr::addr_of_mut!((*WaitBlock).ShareCount)) > 0 {
            return;
        }
    }

    loop {
        if OldValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0 {
            NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value;
            NewValue.__bindgen_anon_1.__bindgen_anon_1.set_MultipleShared(0);
            NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Locked(0);

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0
                    && NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0
                    && NewValue.__bindgen_anon_1.__bindgen_anon_1.MultipleShared() == 0);

            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                return;
            }
        } else {
            NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value;
            NewValue.__bindgen_anon_1.__bindgen_anon_1.set_MultipleShared(0);
            NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Locked(0);
            NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Waking(1);

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0
                    && NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0
                    && NewValue.__bindgen_anon_1.__bindgen_anon_1.MultipleShared() == 0);

            WakeValue = NewValue;
            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );
            if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                /* The write was successful. The pushlock is Unlocked and Waking */
                ExfWakePushLock(PushLock, WakeValue);
                return;
            }

            // The CMPXCHG returned the CURRENT lock value. Retrying without adopting it
            // re-compares against a stale OldValue every time, so a contended release can
            // spin forever. Upstream-Commit: f42ee0e23bb
            OldValue = NewValue;
            continue;
        }
    }
    }
}

fastcall_export! {
    pub unsafe fn ExfReleasePushLockExclusive(PushLock: PEX_PUSH_LOCK) {
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut WakeValue: EX_PUSH_LOCK = core::mem::zeroed();
    let mut OldValue = *PushLock;

    loop {
        ntkrnl_sys::ASSERT!(OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0);
        ntkrnl_sys::ASSERT!(OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0 || OldValue.__bindgen_anon_1.__bindgen_anon_1.Shared() == 0);

        if (OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0) && (OldValue.__bindgen_anon_1.__bindgen_anon_1.Waking() == 0)
        {
            NewValue.__bindgen_anon_1.Value = (OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_LOCK) | EX_PUSH_LOCK_WAKING;

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0 && NewValue.__bindgen_anon_1.__bindgen_anon_1.Locked() == 0);

            WakeValue = NewValue;
            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );

            if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                ExfWakePushLock(PushLock, WakeValue);
                break;
            }
        } else {
            NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value & !EX_PUSH_LOCK_LOCK;

            ntkrnl_sys::ASSERT!(NewValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0
                    || NewValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() == 0);

            NewValue.__bindgen_anon_1.Ptr = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
                NewValue.__bindgen_anon_1.Ptr,
                OldValue.__bindgen_anon_1.Ptr,
            );

            if NewValue.__bindgen_anon_1.Value == OldValue.__bindgen_anon_1.Value {
                break;
            }
        }

        OldValue = NewValue;
    }
    }
}

fastcall_export! {
    pub unsafe fn ExfTryToWakePushLock(PushLock: PEX_PUSH_LOCK) {
    let OldValue = *PushLock;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();

    if (OldValue.__bindgen_anon_1.__bindgen_anon_1.Waking() != 0)
        || (OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() != 0)
        || (OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() == 0)
    {
        return;
    }

    NewValue = OldValue;
    NewValue.__bindgen_anon_1.__bindgen_anon_1.set_Waking(1);

    if InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
        NewValue.__bindgen_anon_1.Ptr,
        OldValue.__bindgen_anon_1.Ptr,
    ) == OldValue.__bindgen_anon_1.Ptr
    {
        ExfWakePushLock(PushLock, NewValue);
    }
    }
}

fastcall_export! {
    pub unsafe fn ExfUnblockPushLock(PushLock: PEX_PUSH_LOCK, CurrentWaitBlock: PVOID) {
    let mut WaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut NextWaitBlock: PEX_PUSH_LOCK_WAIT_BLOCK;
    let mut OldIrql: KIRQL = DISPATCH_LEVEL as KIRQL;

    WaitBlock = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*PushLock).__bindgen_anon_1.Ptr),
        core::ptr::null_mut(),
    ) as PEX_PUSH_LOCK_WAIT_BLOCK;
    if !WaitBlock.is_null() {
        if !(*WaitBlock).Next.is_null() {
            KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));
        }

        while !WaitBlock.is_null() {
            NextWaitBlock = (*WaitBlock).Next;

            if core::sync::atomic::AtomicI32::from_ptr(core::ptr::addr_of_mut!((*WaitBlock).Flags))
            .fetch_and(
                !((1i32) << EX_PUSH_LOCK_FLAGS_WAIT_V),
                core::sync::atomic::Ordering::SeqCst,
            ) & ((1i32) << EX_PUSH_LOCK_FLAGS_WAIT_V)
                == 0
            {
                // synctrace §3: one event-path waiter woken on this pushlock.
                synctrace_pl_wake(
                    PushLock as _, ntkrnl_sys::mm::KeGetCurrentThread() as _);
                KeSetEventBoostPriority(
                    core::ptr::addr_of_mut!((*WaitBlock).__bindgen_anon_1.WakeEvent),
                    core::ptr::null_mut(),
                );
            }

            WaitBlock = NextWaitBlock;
        }

        if OldIrql != DISPATCH_LEVEL as KIRQL {
            KeLowerIrql(OldIrql);
        }
    }

    if !CurrentWaitBlock.is_null()
        && ((*(CurrentWaitBlock as PEX_PUSH_LOCK_WAIT_BLOCK)).Flags & EX_PUSH_LOCK_FLAGS_WAIT != 0)
    {
        ExWaitForUnblockPushLock(PushLock, CurrentWaitBlock);
    }
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ex/pushlock.c
//  * PURPOSE:         Pushlock and Cache-Aware Pushlock Implementation
//  * PROGRAMMER:      Alex Ionescu (alex.ionescu@reactos.com)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* DATA **********************************************************************/
//
// ULONG ExPushLockSpinCount = 0;
//
// #undef EX_PUSH_LOCK
// #undef PEX_PUSH_LOCK
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// #ifdef _WIN64
// #define InterlockedAndPointer(ptr,val) InterlockedAnd64((PLONGLONG)ptr,(LONGLONG)val)
// #else
// #define InterlockedAndPointer(ptr,val) InterlockedAnd((PLONG)ptr,(LONG)val)
// #endif
//
// /*++
//  * @name ExpInitializePushLocks
//  *
//  *     The ExpInitializePushLocks routine initialized Pushlock support.
//  *
//  * @param None.
//  *
//  * @return None.
//  *
//  * @remarks The ExpInitializePushLocks routine sets up the spin on SMP machines.
//  *
//  *--*/
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpInitializePushLocks(VOID)
// {
// #ifdef CONFIG_SMP
//     /* Initialize an internal 1024-iteration spin for MP CPUs */
//     if (KeNumberProcessors > 1)
//         ExPushLockSpinCount = 1024;
// #endif
// }
//
// /*++
//  * @name ExfWakePushLock
//  *
//  *     The ExfWakePushLock routine wakes a Pushlock that is in the waiting
//  *     state.
//  *
//  * @param PushLock
//  *        Pointer to a pushlock that is waiting.
//  *
//  * @param OldValue
//  *        Last known value of the pushlock before this routine was called.
//  *
//  * @return None.
//  *
//  * @remarks This is an internal routine; do not call it manually. Only the system
//  *          can properly know if the pushlock is ready to be awakened or not.
//  *          External callers should use ExfTrytoWakePushLock.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfWakePushLock(PEX_PUSH_LOCK PushLock,
//                 EX_PUSH_LOCK OldValue)
// {
//     EX_PUSH_LOCK NewValue;
//     PEX_PUSH_LOCK_WAIT_BLOCK PreviousWaitBlock, FirstWaitBlock, LastWaitBlock;
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock;
//     KIRQL OldIrql;
//
//     /* Start main wake loop */
//     for (;;)
//     {
//         /* Sanity checks */
//         ASSERT(!OldValue.MultipleShared);
//
//         /* Check if it's locked */
//         while (OldValue.Locked)
//         {
//             /* It's not waking anymore */
//             NewValue.Value = OldValue.Value &~ EX_PUSH_LOCK_WAKING;
//
//             /* Sanity checks */
//             ASSERT(!NewValue.Waking);
//             ASSERT(NewValue.Locked);
//             ASSERT(NewValue.Waiting);
//
//             /* Write the New Value */
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Value == OldValue.Value) return;
//
//             /* Someone changed the value behind our back, update it*/
//             OldValue = NewValue;
//         }
//
//         /* Save the First Block */
//         FirstWaitBlock = (PEX_PUSH_LOCK_WAIT_BLOCK)(OldValue.Value &
//                           ~EX_PUSH_LOCK_PTR_BITS);
//         WaitBlock = FirstWaitBlock;
//
//         /* Try to find the last block */
//         while (TRUE)
//         {
//             /* Get the last wait block */
//             LastWaitBlock = WaitBlock->Last;
//
//             /* Check if we found it */
//             if (LastWaitBlock)
//             {
//                 /* Use it */
//                 WaitBlock = LastWaitBlock;
//                 break;
//             }
//
//             /* Save the previous block */
//             PreviousWaitBlock = WaitBlock;
//
//             /* Move to next block */
//             WaitBlock = WaitBlock->Next;
//
//             /* Save the previous block */
//             WaitBlock->Previous = PreviousWaitBlock;
//         }
//
//         /* Check if the last Wait Block is not Exclusive or if it's the only one */
//         PreviousWaitBlock = WaitBlock->Previous;
//         if (!(WaitBlock->Flags & EX_PUSH_LOCK_FLAGS_EXCLUSIVE) ||
//             !(PreviousWaitBlock))
//         {
//             /* Destroy the pushlock */
//             NewValue.Value = 0;
//             ASSERT(!NewValue.Waking);
//
//             /* Write the New Value */
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Value == OldValue.Value) break;
//
//             /* Someone changed the value behind our back, update it*/
//             OldValue = NewValue;
//         }
//         else
//         {
//             /* Link the wait blocks */
//             FirstWaitBlock->Last = PreviousWaitBlock;
//             WaitBlock->Previous = NULL;
//
//             /* Sanity checks */
//             ASSERT(FirstWaitBlock != WaitBlock);
//             ASSERT(PushLock->Waiting);
//
//             /* Remove waking bit from pushlock */
//             InterlockedAndPointer(&PushLock->Value, ~EX_PUSH_LOCK_WAKING);
//
//             /* Leave the loop */
//             break;
//         }
//     }
//
//     /* Check if there's a previous block */
//     OldIrql = DISPATCH_LEVEL;
//     if (WaitBlock->Previous)
//     {
//         /* Raise to Dispatch */
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//     }
//
//     /* Signaling loop */
//     for (;;)
//     {
//         /* Get the previous Wait block */
//         PreviousWaitBlock = WaitBlock->Previous;
//
//         /* Sanity check */
//         ASSERT(!WaitBlock->Signaled);
//
// #if DBG
//         /* We are about to get signaled */
//         WaitBlock->Signaled = TRUE;
// #endif
//
//         /* Set the Wait Bit in the Wait Block */
//         if (!InterlockedBitTestAndReset(&WaitBlock->Flags, 1))
//         {
//             /* Nobody signaled us, so do it */
//             KeSignalGateBoostPriority(&WaitBlock->WakeGate);
//         }
//
//         /* Set the wait block and check if there still is one to loop*/
//         WaitBlock = PreviousWaitBlock;
//         if (!WaitBlock) break;
//     }
//
//     /* Check if we have to lower back the IRQL */
//     if (OldIrql != DISPATCH_LEVEL) KeLowerIrql(OldIrql);
// }
//
// /*++
//  * @name ExpOptimizePushLockList
//  *
//  *     The ExpOptimizePushLockList routine optimizes the list of waiters
//  *     associated to a pushlock's wait block.
//  *
//  * @param PushLock
//  *        Pointer to a pushlock whose waiter list needs to be optimized.
//  *
//  * @param OldValue
//  *        Last known value of the pushlock before this routine was called.
//  *
//  * @return None.
//  *
//  * @remarks At the end of the optimization, the pushlock will also be wakened.
//  *
//  *--*/
// VOID
// FASTCALL
// ExpOptimizePushLockList(PEX_PUSH_LOCK PushLock,
//                         EX_PUSH_LOCK OldValue)
// {
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock, LastWaitBlock, PreviousWaitBlock, FirstWaitBlock;
//     EX_PUSH_LOCK NewValue;
//
//     /* Start main loop */
//     for (;;)
//     {
//         /* Check if we've been unlocked */
//         if (!OldValue.Locked)
//         {
//             /* Wake us up and leave */
//             ExfWakePushLock(PushLock, OldValue);
//             break;
//         }
//
//         /* Get the wait block */
//         WaitBlock = (PEX_PUSH_LOCK_WAIT_BLOCK)(OldValue.Value &
//                                                ~EX_PUSH_LOCK_PTR_BITS);
//
//         /* Loop the blocks */
//         FirstWaitBlock = WaitBlock;
//         while (TRUE)
//         {
//             /* Get the last wait block */
//             LastWaitBlock = WaitBlock->Last;
//             if (LastWaitBlock)
//             {
//                 /* Set this as the new last block, we're done */
//                 FirstWaitBlock->Last = LastWaitBlock;
//                 break;
//             }
//
//             /* Save the block */
//             PreviousWaitBlock = WaitBlock;
//
//             /* Get the next block */
//             WaitBlock = WaitBlock->Next;
//
//             /* Save the previous */
//             WaitBlock->Previous = PreviousWaitBlock;
//         }
//
//         /* Remove the wake bit */
//         NewValue.Value = OldValue.Value &~ EX_PUSH_LOCK_WAKING;
//
//         /* Sanity checks */
//         ASSERT(NewValue.Locked);
//         ASSERT(!NewValue.Waking);
//
//         /* Update the value */
//         NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                          NewValue.Ptr,
//                                                          OldValue.Ptr);
//
//         /* If we updated correctly, leave */
//         if (NewValue.Value == OldValue.Value) break;
//
//         /* Update value */
//         OldValue = NewValue;
//     }
// }
//
// /*++
//  * @name ExTimedWaitForUnblockPushLock
//  *
//  *     The ExTimedWaitForUnblockPushLock routine waits for a pushlock
//  *     to be unblocked, for a specified internal.
//  *
//  * @param PushLock
//  *        Pointer to a pushlock whose waiter list needs to be optimized.
//  *
//  * @param WaitBlock
//  *        Pointer to the pushlock's wait block.
//  *
//  * @param Timeout
//  *        Amount of time to wait for this pushlock to be unblocked.
//  *
//  * @return STATUS_SUCCESS is the pushlock is now unblocked, otherwise the error
//  *         code returned by KeWaitForSingleObject.
//  *
//  * @remarks If the wait fails, then a manual unblock is attempted.
//  *
//  *--*/
// NTSTATUS
// FASTCALL
// ExTimedWaitForUnblockPushLock(IN PEX_PUSH_LOCK PushLock,
//                               IN PVOID WaitBlock,
//                               IN PLARGE_INTEGER Timeout)
// {
//     NTSTATUS Status;
//
//     /* Initialize the wait event */
//     KeInitializeEvent(&((PEX_PUSH_LOCK_WAIT_BLOCK)WaitBlock)->WakeEvent,
//                       SynchronizationEvent,
//                       FALSE);
//
// #ifdef CONFIG_SMP
//     /* Spin on the push lock if necessary */
//     if (ExPushLockSpinCount)
//     {
//         ULONG i = ExPushLockSpinCount;
//
//         do
//         {
//             /* Check if we got lucky and can leave early */
//             if (!(*(volatile LONG *)&((PEX_PUSH_LOCK_WAIT_BLOCK)WaitBlock)->Flags & EX_PUSH_LOCK_WAITING))
//                 return STATUS_SUCCESS;
//
//             YieldProcessor();
//         } while (--i);
//     }
// #endif
//
//     /* Now try to remove the wait bit */
//     if (InterlockedBitTestAndReset(&((PEX_PUSH_LOCK_WAIT_BLOCK)WaitBlock)->Flags,
//                                    EX_PUSH_LOCK_FLAGS_WAIT_V))
//     {
//         /* Nobody removed it already, let's do a full wait */
//         Status = KeWaitForSingleObject(&((PEX_PUSH_LOCK_WAIT_BLOCK)WaitBlock)->
//                                        WakeEvent,
//                                        WrPushLock,
//                                        KernelMode,
//                                        FALSE,
//                                        Timeout);
//         /* Check if the wait was satisfied */
//         if (Status != STATUS_SUCCESS)
//         {
//             /* Try unblocking the pushlock if it was not */
//             ExfUnblockPushLock(PushLock, WaitBlock);
//         }
//     }
//     else
//     {
//         /* Someone beat us to it, no need to wait */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
//  * @name ExWaitForUnblockPushLock
//  *
//  *     The ExWaitForUnblockPushLock routine waits for a pushlock
//  *     to be unblocked, for a specified internal.
//  *
//  * @param PushLock
//  *        Pointer to a pushlock whose waiter list needs to be optimized.
//  *
//  * @param WaitBlock
//  *        Pointer to the pushlock's wait block.
//  *
//  * @return STATUS_SUCCESS is the pushlock is now unblocked, otherwise the error
//  *         code returned by KeWaitForSingleObject.
//  *
//  * @remarks If the wait fails, then a manual unblock is attempted.
//  *
//  *--*/
// VOID
// FASTCALL
// ExWaitForUnblockPushLock(IN PEX_PUSH_LOCK PushLock,
//                          IN PVOID WaitBlock)
// {
//     /* Call the timed function with no timeout */
//     ExTimedWaitForUnblockPushLock(PushLock, WaitBlock, NULL);
// }
//
// /*++
//  * @name ExBlockPushLock
//  *
//  *     The ExBlockPushLock routine blocks a pushlock.
//  *
//  * @param PushLock
//  *        Pointer to a pushlock whose waiter list needs to be optimized.
//  *
//  * @param WaitBlock
//  *        Pointer to the pushlock's wait block.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// FASTCALL
// ExBlockPushLock(PEX_PUSH_LOCK PushLock,
//                 PVOID pWaitBlock)
// {
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock = pWaitBlock;
//     EX_PUSH_LOCK NewValue, OldValue;
//
//     /* Detect invalid wait block alignment */
//     ASSERT(((ULONG_PTR)pWaitBlock & 0xF) == 0);
//
//     /* Set the waiting bit */
//     WaitBlock->Flags = EX_PUSH_LOCK_FLAGS_WAIT;
//
//     /* Get the old value */
//     OldValue = *PushLock;
//
//     /* Start block loop */
//     for (;;)
//     {
//         /* Link the wait blocks */
//         WaitBlock->Next = OldValue.Ptr;
//
//         /* Set the new wait block value */
//         NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                          WaitBlock,
//                                                          OldValue.Ptr);
//         if (OldValue.Ptr == NewValue.Ptr) break;
//
//         /* Try again with the new value */
//         OldValue = NewValue;
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name ExAcquirePushLockExclusive
//  * @implemented NT5.1
//  *
//  *     The ExAcquirePushLockExclusive macro exclusively acquires a PushLock.
//  *
//  * @params PushLock
//  *         Pointer to the pushlock which is to be acquired.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExAcquirePushLockShared must be running at IRQL <= APC_LEVEL.
//  *          This macro should usually be paired up with KeAcquireCriticalRegion.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfAcquirePushLockExclusive(PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK OldValue = *PushLock, NewValue, TempValue;
//     BOOLEAN NeedWake;
//     EX_PUSH_LOCK_WAIT_BLOCK Block;
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock = &Block;
//
//     /* Start main loop */
//     for (;;)
//     {
//         /* Check if it's unlocked */
//         if (!OldValue.Locked)
//         {
//             /* Lock it */
//             NewValue.Value = OldValue.Value | EX_PUSH_LOCK_LOCK;
//             ASSERT(NewValue.Locked);
//
//             /* Set the new value */
//             if (InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                   NewValue.Ptr,
//                                                   OldValue.Ptr) != OldValue.Ptr)
//             {
//                 /* Retry */
//                 OldValue = *PushLock;
//                 continue;
//             }
//
//             /* Break out of the loop */
//             break;
//         }
//         else
//         {
//             /* We'll have to create a Waitblock */
//             WaitBlock->Flags = EX_PUSH_LOCK_FLAGS_EXCLUSIVE |
//                                EX_PUSH_LOCK_FLAGS_WAIT;
//             WaitBlock->Previous = NULL;
//             NeedWake = FALSE;
//
//             /* Check if there is already a waiter */
//             if (OldValue.Waiting)
//             {
//                 /* Nobody is the last waiter yet */
//                 WaitBlock->Last = NULL;
//
//                 /* We are an exclusive waiter */
//                 WaitBlock->ShareCount = 0;
//
//                 /* Set the current Wait Block pointer */
//                 WaitBlock->Next = (PEX_PUSH_LOCK_WAIT_BLOCK)(
//                                    OldValue.Value &~ EX_PUSH_LOCK_PTR_BITS);
//
//                 /* Point to ours */
//                 NewValue.Value = (OldValue.Value & EX_PUSH_LOCK_MULTIPLE_SHARED) |
//                                   EX_PUSH_LOCK_LOCK |
//                                   EX_PUSH_LOCK_WAKING |
//                                   EX_PUSH_LOCK_WAITING |
//                                   (ULONG_PTR)WaitBlock;
//
//                 /* Check if the pushlock was already waking */
//                 if (!OldValue.Waking) NeedWake = TRUE;
//             }
//             else
//             {
//                 /* We are the first waiter, so loop the wait block */
//                 WaitBlock->Last = WaitBlock;
//
//                 /* Set the share count */
//                 WaitBlock->ShareCount = (LONG)OldValue.Shared;
//
//                 /* Check if someone is sharing this pushlock */
//                 if (OldValue.Shared > 1)
//                 {
//                     /* Point to our wait block */
//                     NewValue.Value = EX_PUSH_LOCK_MULTIPLE_SHARED |
//                                      EX_PUSH_LOCK_LOCK |
//                                      EX_PUSH_LOCK_WAITING |
//                                      (ULONG_PTR)WaitBlock;
//                 }
//                 else
//                 {
//                     /* No shared count */
//                     WaitBlock->ShareCount = 0;
//
//                     /* Point to our wait block */
//                     NewValue.Value = EX_PUSH_LOCK_LOCK |
//                                      EX_PUSH_LOCK_WAITING |
//                                      (ULONG_PTR)WaitBlock;
//                 }
//             }
//
// #if DBG
//             /* Setup the Debug Wait Block */
//             WaitBlock->Signaled = 0;
//             WaitBlock->OldValue = OldValue;
//             WaitBlock->NewValue = NewValue;
//             WaitBlock->PushLock = PushLock;
// #endif
//
//             /* Sanity check */
//             ASSERT(NewValue.Waiting);
//             ASSERT(NewValue.Locked);
//
//             /* Write the new value */
//             TempValue = NewValue;
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Value != OldValue.Value)
//             {
//                 /* Retry */
//                 OldValue = *PushLock;
//                 continue;
//             }
//
//             /* Check if the pushlock needed waking */
//             if (NeedWake)
//             {
//                 /* Scan the Waiters and Wake PushLocks */
//                 ExpOptimizePushLockList(PushLock, TempValue);
//             }
//
//             /* Set up the Wait Gate */
//             KeInitializeGate(&WaitBlock->WakeGate);
//
// #ifdef CONFIG_SMP
//             /* Now spin on the push lock if necessary */
//             if (ExPushLockSpinCount)
//             {
//                 ULONG i = ExPushLockSpinCount;
//
//                 do
//                 {
//                     if (!(*(volatile LONG *)&WaitBlock->Flags & EX_PUSH_LOCK_WAITING))
//                         break;
//
//                     YieldProcessor();
//                 } while (--i);
//             }
// #endif
//
//             /* Now try to remove the wait bit */
//             if (InterlockedBitTestAndReset(&WaitBlock->Flags, 1))
//             {
//                 /* Nobody removed it already, let's do a full wait */
//                 KeWaitForGate(&WaitBlock->WakeGate, WrPushLock, KernelMode);
//                 ASSERT(WaitBlock->Signaled);
//             }
//
//             /* We shouldn't be shared anymore */
//             ASSERT((WaitBlock->ShareCount == 0));
//
//             /* Loop again */
//             OldValue = NewValue;
//         }
//     }
// }
//
// /*++
//  * @name ExAcquirePushLockShared
//  * @implemented NT5.1
//  *
//  *     The ExAcquirePushLockShared routine acquires a shared PushLock.
//  *
//  * @params PushLock
//  *         Pointer to the pushlock which is to be acquired.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExAcquirePushLockShared must be running at IRQL <= APC_LEVEL.
//  *          This macro should usually be paired up with KeAcquireCriticalRegion.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfAcquirePushLockShared(PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK OldValue = *PushLock, NewValue;
//     BOOLEAN NeedWake;
//     EX_PUSH_LOCK_WAIT_BLOCK Block;
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock = &Block;
//
//     /* Start main loop */
//     for (;;)
//     {
//         /* Check if it's unlocked or if it's waiting without any sharers */
//         if (!(OldValue.Locked) || (!(OldValue.Waiting) && (OldValue.Shared > 0)))
//         {
//             /* Check if anyone is waiting on it */
//             if (!OldValue.Waiting)
//             {
//                 /* Increase the share count and lock it */
//                 NewValue.Value = OldValue.Value | EX_PUSH_LOCK_LOCK;
//                 NewValue.Shared++;
//             }
//             else
//             {
//                 /* Simply set the lock bit */
//                 NewValue.Value = OldValue.Value | EX_PUSH_LOCK_LOCK;
//             }
//
//             /* Sanity check */
//             ASSERT(NewValue.Locked);
//
//             /* Set the new value */
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Value != OldValue.Value)
//             {
//                 /* Retry */
//                 OldValue = *PushLock;
//                 continue;
//             }
//
//             /* Break out of the loop */
//             break;
//         }
//         else
//         {
//             /* We'll have to create a Waitblock */
//             WaitBlock->Flags = EX_PUSH_LOCK_FLAGS_WAIT;
//             WaitBlock->ShareCount = 0;
//             NeedWake = FALSE;
//             WaitBlock->Previous = NULL;
//
//             /* Check if there is already a waiter */
//             if (OldValue.Waiting)
//             {
//                 /* Set the current Wait Block pointer */
//                 WaitBlock->Next = (PEX_PUSH_LOCK_WAIT_BLOCK)(
//                                    OldValue.Value &~ EX_PUSH_LOCK_PTR_BITS);
//
//                 /* Nobody is the last waiter yet */
//                 WaitBlock->Last = NULL;
//
//                 /* Point to ours */
//                 NewValue.Value = (OldValue.Value & (EX_PUSH_LOCK_MULTIPLE_SHARED |
//                                                     EX_PUSH_LOCK_LOCK)) |
//                                   EX_PUSH_LOCK_WAKING |
//                                   EX_PUSH_LOCK_WAITING |
//                                   (ULONG_PTR)WaitBlock;
//
//                 /* Check if the pushlock was already waking */
//                 if (!OldValue.Waking) NeedWake = TRUE;
//             }
//             else
//             {
//                 /* We are the first waiter, so loop the wait block */
//                 WaitBlock->Last = WaitBlock;
//
//                 /* Point to our wait block */
//                 NewValue.Value = (OldValue.Value & EX_PUSH_LOCK_PTR_BITS) |
//                                   EX_PUSH_LOCK_WAITING |
//                                   (ULONG_PTR)WaitBlock;
//             }
//
//             /* Sanity check */
//             ASSERT(NewValue.Waiting);
//
// #if DBG
//             /* Setup the Debug Wait Block */
//             WaitBlock->Signaled = 0;
//             WaitBlock->OldValue = OldValue;
//             WaitBlock->NewValue = NewValue;
//             WaitBlock->PushLock = PushLock;
// #endif
//
//             /* Write the new value */
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Ptr != OldValue.Ptr)
//             {
//                 /* Retry */
//                 OldValue = *PushLock;
//                 continue;
//             }
//
//             /* Update the value now */
//             OldValue = NewValue;
//
//             /* Check if the pushlock needed waking */
//             if (NeedWake)
//             {
//                 /* Scan the Waiters and Wake PushLocks */
//                 ExpOptimizePushLockList(PushLock, OldValue);
//             }
//
//             /* Set up the Wait Gate */
//             KeInitializeGate(&WaitBlock->WakeGate);
//
// #ifdef CONFIG_SMP
//             /* Now spin on the push lock if necessary */
//             if (ExPushLockSpinCount)
//             {
//                 ULONG i = ExPushLockSpinCount;
//
//                 do
//                 {
//                     if (!(*(volatile LONG *)&WaitBlock->Flags & EX_PUSH_LOCK_WAITING))
//                         break;
//
//                     YieldProcessor();
//                 } while (--i);
//             }
// #endif
//
//             /* Now try to remove the wait bit */
//             if (InterlockedBitTestAndReset(&WaitBlock->Flags, 1))
//             {
//                 /* Fast-path did not work, we need to do a full wait */
//                 KeWaitForGate(&WaitBlock->WakeGate, WrPushLock, KernelMode);
//                 ASSERT(WaitBlock->Signaled);
//             }
//
//             /* We shouldn't be shared anymore */
//             ASSERT((WaitBlock->ShareCount == 0));
//         }
//     }
// }
//
// /*++
//  * @name ExfReleasePushLock
//  * @implemented NT5.1
//  *
//  *     The ExReleasePushLock routine releases a previously acquired PushLock.
//  *
//  *
//  * @params PushLock
//  *         Pointer to a previously acquired pushlock.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfReleasePushLock must be running at IRQL <= APC_LEVEL.
//  *          This macro should usually be paired up with KeLeaveCriticalRegion.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfReleasePushLock(PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK OldValue = *PushLock, NewValue, WakeValue;
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock, LastWaitBlock;
//
//     /* Sanity check */
//     ASSERT(OldValue.Locked);
//
//     /* Start main loop */
//     while (TRUE)
//     {
//         /* Check if someone is waiting on the lock */
//         if (!OldValue.Waiting)
//         {
//             /* Check if it's shared */
//             if (OldValue.Shared > 1)
//             {
//                 /* Write the Old Value but decrease share count */
//                 NewValue = OldValue;
//                 NewValue.Shared--;
//             }
//             else
//             {
//                 /* Simply clear the lock */
//                 NewValue.Value = 0;
//             }
//
//             /* Write the New Value */
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Value == OldValue.Value) return;
//
//             /* Did it enter a wait state? */
//             OldValue = NewValue;
//         }
//         else
//         {
//             /* Ok, we do know someone is waiting on it. Are there more then one? */
//             if (OldValue.MultipleShared)
//             {
//                 /* Get the wait block */
//                 WaitBlock = (PEX_PUSH_LOCK_WAIT_BLOCK)(OldValue.Value &
//                                                        ~EX_PUSH_LOCK_PTR_BITS);
//
//                 /* Loop until we find the last wait block */
//                 while (TRUE)
//                 {
//                     /* Get the last wait block */
//                     LastWaitBlock = WaitBlock->Last;
//
//                     /* Did it exist? */
//                     if (LastWaitBlock)
//                     {
//                         /* Choose it */
//                         WaitBlock = LastWaitBlock;
//                         break;
//                     }
//
//                     /* Keep searching */
//                     WaitBlock = WaitBlock->Next;
//                 }
//
//                 /* Make sure the Share Count is above 0 */
//                 if (WaitBlock->ShareCount > 0)
//                 {
//                     /* This shouldn't be an exclusive wait block */
//                     ASSERT(WaitBlock->Flags & EX_PUSH_LOCK_FLAGS_EXCLUSIVE);
//
//                     /* Do the decrease and check if the lock isn't shared anymore */
//                     if (InterlockedDecrement(&WaitBlock->ShareCount) > 0) return;
//                 }
//             }
//
//             /*
//              * If nobody was waiting on the block, then we possibly reduced the number
//              * of times the pushlock was shared, and we unlocked it.
//              * If someone was waiting, and more then one person is waiting, then we
//              * reduced the number of times the pushlock is shared in the wait block.
//              * Therefore, at this point, we can now 'satisfy' the wait.
//              */
//             for (;;)
//             {
//                 /* Now we need to see if it's waking */
//                 if (OldValue.Waking)
//                 {
//                     /* Remove the lock and multiple shared bits */
//                     NewValue.Value = OldValue.Value;
//                     NewValue.MultipleShared = FALSE;
//                     NewValue.Locked = FALSE;
//
//                     /* Sanity check */
//                     ASSERT(NewValue.Waking && !NewValue.Locked && !NewValue.MultipleShared);
//
//                     /* Write the new value */
//                     NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                                      NewValue.Ptr,
//                                                                      OldValue.Ptr);
//                     if (NewValue.Value == OldValue.Value) return;
//                 }
//                 else
//                 {
//                     /* Remove the lock and multiple shared bits */
//                     NewValue.Value = OldValue.Value;
//                     NewValue.MultipleShared = FALSE;
//                     NewValue.Locked = FALSE;
//
//                     /* It's not already waking, so add the wake bit */
//                     NewValue.Waking = TRUE;
//
//                     /* Sanity check */
//                     ASSERT(NewValue.Waking && !NewValue.Locked && !NewValue.MultipleShared);
//
//                     /* Write the new value */
//                     WakeValue = NewValue;
//                     NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                                      NewValue.Ptr,
//                                                                      OldValue.Ptr);
//                     if (NewValue.Value != OldValue.Value) continue;
//
//                     /* The write was successful. The pushlock is Unlocked and Waking */
//                     ExfWakePushLock(PushLock, WakeValue);
//                     return;
//                 }
//             }
//         }
//     }
// }
//
// /*++
//  * @name ExfReleasePushLockShared
//  * @implemented NT5.2
//  *
//  *     The ExfReleasePushLockShared macro releases a previously acquired PushLock.
//  *
//  * @params PushLock
//  *         Pointer to a previously acquired pushlock.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExReleasePushLockShared must be running at IRQL <= APC_LEVEL.
//  *          This macro should usually be paired up with KeLeaveCriticalRegion.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfReleasePushLockShared(PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK OldValue = *PushLock, NewValue, WakeValue;
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock, LastWaitBlock;
//
//     /* Check if someone is waiting on the lock */
//     while (!OldValue.Waiting)
//     {
//         /* Check if it's shared */
//         if (OldValue.Shared > 1)
//         {
//             /* Write the Old Value but decrease share count */
//             NewValue = OldValue;
//             NewValue.Shared--;
//         }
//         else
//         {
//             /* Simply clear the lock */
//             NewValue.Value = 0;
//         }
//
//         /* Write the New Value */
//         NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                          NewValue.Ptr,
//                                                          OldValue.Ptr);
//         if (NewValue.Value == OldValue.Value) return;
//
//         /* Did it enter a wait state? */
//         OldValue = NewValue;
//     }
//
//     /* Ok, we do know someone is waiting on it. Are there more then one? */
//     if (OldValue.MultipleShared)
//     {
//         /* Get the wait block */
//         WaitBlock = (PEX_PUSH_LOCK_WAIT_BLOCK)(OldValue.Value &
//                                                ~EX_PUSH_LOCK_PTR_BITS);
//
//         /* Loop until we find the last wait block */
//         while (TRUE)
//         {
//             /* Get the last wait block */
//             LastWaitBlock = WaitBlock->Last;
//
//             /* Did it exist? */
//             if (LastWaitBlock)
//             {
//                 /* Choose it */
//                 WaitBlock = LastWaitBlock;
//                 break;
//             }
//
//             /* Keep searching */
//             WaitBlock = WaitBlock->Next;
//         }
//
//         /* Sanity checks */
//         ASSERT(WaitBlock->ShareCount > 0);
//         ASSERT(WaitBlock->Flags & EX_PUSH_LOCK_FLAGS_EXCLUSIVE);
//
//         /* Do the decrease and check if the lock isn't shared anymore */
//         if (InterlockedDecrement(&WaitBlock->ShareCount) > 0) return;
//     }
//
//     /*
//      * If nobody was waiting on the block, then we possibly reduced the number
//      * of times the pushlock was shared, and we unlocked it.
//      * If someone was waiting, and more then one person is waiting, then we
//      * reduced the number of times the pushlock is shared in the wait block.
//      * Therefore, at this point, we can now 'satisfy' the wait.
//      */
//     for (;;)
//     {
//         /* Now we need to see if it's waking */
//         if (OldValue.Waking)
//         {
//             /* Remove the lock and multiple shared bits */
//             NewValue.Value = OldValue.Value;
//             NewValue.MultipleShared = FALSE;
//             NewValue.Locked = FALSE;
//
//             /* Sanity check */
//             ASSERT(NewValue.Waking && !NewValue.Locked && !NewValue.MultipleShared);
//
//             /* Write the new value */
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Value == OldValue.Value) return;
//         }
//         else
//         {
//             /* Remove the lock and multiple shared bits */
//             NewValue.Value = OldValue.Value;
//             NewValue.MultipleShared = FALSE;
//             NewValue.Locked = FALSE;
//
//             /* It's not already waking, so add the wake bit */
//             NewValue.Waking = TRUE;
//
//             /* Sanity check */
//             ASSERT(NewValue.Waking && !NewValue.Locked && !NewValue.MultipleShared);
//
//             /* Write the new value */
//             WakeValue = NewValue;
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//             if (NewValue.Value != OldValue.Value) continue;
//
//             /* The write was successful. The pushlock is Unlocked and Waking */
//             ExfWakePushLock(PushLock, WakeValue);
//             return;
//         }
//     }
// }
//
// /*++
//  * ExfReleasePushLockExclusive
//  * @implemented NT5.2
//  *
//  *     The ExfReleasePushLockExclusive routine releases a previously
//  *     exclusively acquired PushLock.
//  *
//  * @params PushLock
//  *         Pointer to a previously acquired pushlock.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExReleasePushLockExclusive must be running at IRQL <= APC_LEVEL.
//  *          This macro should usually be paired up with KeLeaveCriticalRegion.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfReleasePushLockExclusive(PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK NewValue, WakeValue;
//     EX_PUSH_LOCK OldValue = *PushLock;
//
//     /* Loop until we can change */
//     for (;;)
//     {
//         /* Sanity checks */
//         ASSERT(OldValue.Locked);
//         ASSERT(OldValue.Waiting || OldValue.Shared == 0);
//
//         /* Check if it's waiting and not yet waking */
//         if ((OldValue.Waiting) && !(OldValue.Waking))
//         {
//             /* Remove the lock bit, and add the wake bit */
//             NewValue.Value = (OldValue.Value &~ EX_PUSH_LOCK_LOCK) |
//                               EX_PUSH_LOCK_WAKING;
//
//             /* Sanity check */
//             ASSERT(NewValue.Waking && !NewValue.Locked);
//
//             /* Write the New Value. Save our original value for waking */
//             WakeValue = NewValue;
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//
//             /* Check if the value changed behind our back */
//             if (NewValue.Value == OldValue.Value)
//             {
//                 /* Wake the Pushlock */
//                 ExfWakePushLock(PushLock, WakeValue);
//                 break;
//             }
//         }
//         else
//         {
//             /* A simple unlock */
//             NewValue.Value = OldValue.Value &~ EX_PUSH_LOCK_LOCK;
//
//             /* Sanity check */
//             ASSERT(NewValue.Waking || !NewValue.Waiting);
//
//             /* Write the New Value */
//             NewValue.Ptr = InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                                              NewValue.Ptr,
//                                                              OldValue.Ptr);
//
//             /* Check if the value changed behind our back */
//             if (NewValue.Value == OldValue.Value) break;
//         }
//
//         /* Loop again */
//         OldValue = NewValue;
//     }
// }
//
// /*++
//  * @name ExfTryToWakePushLock
//  * @implemented NT5.2
//  *
//  *     The ExfTryToWakePushLock attempts to wake a waiting pushlock.
//  *
//  * @param PushLock
//  *        Pointer to a PushLock which is in the wait state.
//  *
//  * @return None.
//  *
//  * @remarks The pushlock must be in a wait state and must not be already waking.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfTryToWakePushLock(PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK OldValue = *PushLock, NewValue;
//
//     /*
//      * If the Pushlock is not waiting on anything, or if it's already waking up
//      * and locked, don't do anything
//      */
//     if ((OldValue.Waking) || (OldValue.Locked) || !(OldValue.Waiting)) return;
//
//     /* Make it Waking */
//     NewValue = OldValue;
//     NewValue.Waking = TRUE;
//
//     /* Write the New Value */
//     if (InterlockedCompareExchangePointer(&PushLock->Ptr,
//                                           NewValue.Ptr,
//                                           OldValue.Ptr) == OldValue.Ptr)
//     {
//         /* Wake the Pushlock */
//         ExfWakePushLock(PushLock, NewValue);
//     }
// }
//
// /*++
//  * @name ExfUnblockPushLock
//  * @implemented NT5.1
//  *
//  *     The ExfUnblockPushLock routine unblocks a previously blocked PushLock.
//  *
//  * @param PushLock
//  *        Pointer to a previously blocked PushLock.
//  *
//  * @return None.
//  *
//  * @remarks Callers of ExfUnblockPushLock can be running at any IRQL.
//  *
//  *--*/
// VOID
// FASTCALL
// ExfUnblockPushLock(PEX_PUSH_LOCK PushLock,
//                    PVOID CurrentWaitBlock)
// {
//     PEX_PUSH_LOCK_WAIT_BLOCK WaitBlock, NextWaitBlock;
//     KIRQL OldIrql = DISPATCH_LEVEL;
//
//     /* Get the wait block and erase the previous one */
//     WaitBlock = InterlockedExchangePointer(&PushLock->Ptr, NULL);
//     if (WaitBlock)
//     {
//         /* Check if there is a linked pushlock and raise IRQL appropriately */
//         if (WaitBlock->Next) KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
//         /* Start block loop */
//         while (WaitBlock)
//         {
//             /* Get the next block */
//             NextWaitBlock = WaitBlock->Next;
//
//             /* Remove the wait flag from the Wait block */
//             if (!InterlockedBitTestAndReset(&WaitBlock->Flags, EX_PUSH_LOCK_FLAGS_WAIT_V))
//             {
//                 /* Nobody removed the flag before us, so signal the event */
//                 KeSetEventBoostPriority(&WaitBlock->WakeEvent, NULL);
//             }
//
//             /* Try the next one */
//             WaitBlock = NextWaitBlock;
//         }
//
//         /* Lower IRQL if needed */
//         if (OldIrql != DISPATCH_LEVEL) KeLowerIrql(OldIrql);
//     }
//
//     /* Check if we got a wait block that's pending */
//     if ((CurrentWaitBlock) &&
//         (((PEX_PUSH_LOCK_WAIT_BLOCK)CurrentWaitBlock)->Flags &
//            EX_PUSH_LOCK_FLAGS_WAIT))
//     {
//         /* Wait for the pushlock to be unblocked */
//         ExWaitForUnblockPushLock(PushLock, CurrentWaitBlock);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
