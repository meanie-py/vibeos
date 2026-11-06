use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::{InterlockedCompareExchangePointer, InterlockedExchangePointer};
use ntkrnl_sys::rtl::RtlRaiseStatus;
use ntkrnl_sys::ddk::PRTL_SRWLOCK;
// YieldProcessor + the pointer-width Interlocked* / bit-test intrinsics live in
// the cluster's atomics substrate (ntkrnl-sys only surfaces the base CAS).
use crate::atomics::*;

/// One step of an SRW wait. ReactOS's srw.c only ever spins (`YieldProcessor`, a `pause`), so a
/// waiter never gives up the CPU: on one processor, a waiter that outranks the lock's owner
/// spins forever, because the owner never runs to release it. Chrome hung exactly so, its
/// browser and one more thread spinning in RtlAcquireSRWLockExclusive for twelve minutes on
/// a lock a lower-priority thread held. Windows spins briefly and then blocks, so this spins
/// briefly, then yields to equal priority, then sleeps a tick, which lets ANY ready thread run.
/// Only the waiting changes, not the lock word, so every SRW implementation in the process
/// (ntdll's and ntdll_vista's) still agrees on it. The kernel copy keeps the plain spin: it
/// may run at raised IRQL, where sleeping is not allowed.
#[inline]
unsafe fn SrwBackoff(spins: &mut u32) {
    *spins = spins.saturating_add(1);
    #[cfg(not(feature = "kernel"))]
    {
        if *spins > 128 {
            let mut interval = ntkrnl_sys::ddk::LARGE_INTEGER { QuadPart: -10_000 }; // 1 ms
            ntkrnl_sys::ddk::NtDelayExecution(0, &mut interval);
            return;
        }
        if *spins > 64 {
            ntkrnl_sys::ddk::NtYieldExecution();
            return;
        }
    }
    YieldProcessor();
}

/// A queued waiter's `Wake` word: the releaser hands the lock over and then sets it, so a waiter
/// owns the lock exactly when it reads `WAKE_SET`. A waiter that is going to block first moves
/// it to `WAKE_SLEEPING`, and a releaser that finds that value wakes it with the keyed event
/// (process-wide, key = the word's address, which keyed events require to be even).
///
/// srw.c only ever polls this word, and the backoff above made a long poll sleep 1 ms at a
/// time, which in practice is a timer tick. The lock is already handed to the waiter by then,
/// so every handoff to a waiter that happened to be asleep held the lock for a tick with nobody
/// running it: fault64's VirtualAlloc thread against its AV storm waited ~69 ms an acquire on
/// one vCPU (appstorm64 20261105-204820-babcf_n4). Blocking makes the handoff wake the waiter
/// at once, as Windows does.
const WAKE_SET: i32 = 1;
#[cfg(not(feature = "kernel"))]
const WAKE_SLEEPING: i32 = 2;

#[inline]
unsafe fn WakeWord<'a>(Wake: *mut LONG) -> &'a core::sync::atomic::AtomicI32 {
    core::sync::atomic::AtomicI32::from_ptr(Wake as *mut i32)
}

/// Wait until `Wake` is set: spin and yield as `SrwBackoff` does, then block. The kernel copy
/// keeps spinning (raised IRQL may not sleep), so it never publishes `WAKE_SLEEPING`.
// the kernel copy only spins, so its count is never read
#[cfg_attr(feature = "kernel", allow(unused_variables, unused_assignments))]
unsafe fn SrwWaitForWake(Wake: *mut LONG) {
    use core::sync::atomic::Ordering;
    let word = WakeWord(Wake);
    let mut spins: u32 = 0;
    loop {
        if word.load(Ordering::Acquire) == WAKE_SET {
            return;
        }
        #[cfg(not(feature = "kernel"))]
        if spins >= 128 {
            if word
                .compare_exchange(0, WAKE_SLEEPING, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // The releaser sets the word before it releases us, so it reads WAKE_SET
                // once this returns; loop to re-check rather than trust the status.
                ntkrnl_sys::ddk::NtWaitForKeyedEvent(
                    core::ptr::null_mut(),
                    Wake as PVOID,
                    0,
                    core::ptr::null_mut(),
                );
            }
            continue;
        }
        spins += 1;
        #[cfg(not(feature = "kernel"))]
        if spins > 64 {
            ntkrnl_sys::ddk::NtYieldExecution();
            continue;
        }
        YieldProcessor();
    }
}

/// Set a waiter's `Wake` after the lock word already names it the owner. Nothing in its wait
/// block is read afterwards: the waiter may return, and its stack block die, the moment the
/// word reads `WAKE_SET`. The keyed-event key is only an address.
unsafe fn SrwSignalWake(Wake: *mut LONG) {
    use core::sync::atomic::Ordering;
    let _old = WakeWord(Wake).swap(WAKE_SET, Ordering::AcqRel);
    #[cfg(not(feature = "kernel"))]
    if _old == WAKE_SLEEPING {
        ntkrnl_sys::ddk::NtReleaseKeyedEvent(
            core::ptr::null_mut(),
            Wake as PVOID,
            0,
            core::ptr::null_mut(),
        );
    }
}

// ntdll-internal NTSTATUS (not surfaced by ntkrnl-sys::constants)
const STATUS_RESOURCE_NOT_OWNED: NTSTATUS = 0xC0000264u32 as NTSTATUS;

// ---- file-scope preamble (translated) ----
#[cfg(target_pointer_width = "64")]
pub const _ONE: i64 = 1;
#[cfg(not(target_pointer_width = "64"))]
pub const _ONE: i32 = 1;

pub const RTL_SRWLOCK_OWNED_BIT: u32 = 0;
pub const RTL_SRWLOCK_CONTENDED_BIT: u32 = 1;
pub const RTL_SRWLOCK_SHARED_BIT: u32 = 2;
pub const RTL_SRWLOCK_CONTENTION_LOCK_BIT: u32 = 3;

#[cfg(target_pointer_width = "64")]
pub const RTL_SRWLOCK_OWNED: i64 = _ONE << RTL_SRWLOCK_OWNED_BIT;
#[cfg(not(target_pointer_width = "64"))]
pub const RTL_SRWLOCK_OWNED: i32 = _ONE << RTL_SRWLOCK_OWNED_BIT;

#[cfg(target_pointer_width = "64")]
pub const RTL_SRWLOCK_CONTENDED: i64 = _ONE << RTL_SRWLOCK_CONTENDED_BIT;
#[cfg(not(target_pointer_width = "64"))]
pub const RTL_SRWLOCK_CONTENDED: i32 = _ONE << RTL_SRWLOCK_CONTENDED_BIT;

#[cfg(target_pointer_width = "64")]
pub const RTL_SRWLOCK_SHARED: i64 = _ONE << RTL_SRWLOCK_SHARED_BIT;
#[cfg(not(target_pointer_width = "64"))]
pub const RTL_SRWLOCK_SHARED: i32 = _ONE << RTL_SRWLOCK_SHARED_BIT;

#[cfg(target_pointer_width = "64")]
pub const RTL_SRWLOCK_CONTENTION_LOCK: i64 = _ONE << RTL_SRWLOCK_CONTENTION_LOCK_BIT;
#[cfg(not(target_pointer_width = "64"))]
pub const RTL_SRWLOCK_CONTENTION_LOCK: i32 = _ONE << RTL_SRWLOCK_CONTENTION_LOCK_BIT;

#[cfg(target_pointer_width = "64")]
pub const RTL_SRWLOCK_MASK: i64 =
    RTL_SRWLOCK_OWNED | RTL_SRWLOCK_CONTENDED | RTL_SRWLOCK_SHARED | RTL_SRWLOCK_CONTENTION_LOCK;
#[cfg(not(target_pointer_width = "64"))]
pub const RTL_SRWLOCK_MASK: i32 =
    RTL_SRWLOCK_OWNED | RTL_SRWLOCK_CONTENDED | RTL_SRWLOCK_SHARED | RTL_SRWLOCK_CONTENTION_LOCK;

pub const RTL_SRWLOCK_BITS: u32 = 4;

#[repr(C)]
pub struct _RTLP_SRWLOCK_SHARED_WAKE {
    pub Wake: LONG,
    pub Next: *mut _RTLP_SRWLOCK_SHARED_WAKE,
}
pub type RTLP_SRWLOCK_SHARED_WAKE = _RTLP_SRWLOCK_SHARED_WAKE;
pub type PRTLP_SRWLOCK_SHARED_WAKE = *mut _RTLP_SRWLOCK_SHARED_WAKE;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _RTLP_SRWLOCK_WAITBLOCK__bindgen_ty_1__bindgen_ty_1 {
    pub SharedWakeChain: PRTLP_SRWLOCK_SHARED_WAKE,
    pub LastSharedWake: PRTLP_SRWLOCK_SHARED_WAKE,
}

#[repr(C)]
pub union _RTLP_SRWLOCK_WAITBLOCK__bindgen_ty_1 {
    pub Wake: LONG,
    pub __bindgen_anon_1: _RTLP_SRWLOCK_WAITBLOCK__bindgen_ty_1__bindgen_ty_1,
}

#[repr(C)]
pub struct _RTLP_SRWLOCK_WAITBLOCK {
    pub SharedCount: LONG,
    pub Last: *mut _RTLP_SRWLOCK_WAITBLOCK,
    pub Next: *mut _RTLP_SRWLOCK_WAITBLOCK,
    pub __bindgen_anon_1: _RTLP_SRWLOCK_WAITBLOCK__bindgen_ty_1,
    pub Exclusive: BOOLEAN,
}
pub type RTLP_SRWLOCK_WAITBLOCK = _RTLP_SRWLOCK_WAITBLOCK;
pub type PRTLP_SRWLOCK_WAITBLOCK = *mut _RTLP_SRWLOCK_WAITBLOCK;
unsafe fn RtlpReleaseWaitBlockLockExclusive(
    SRWLock: PRTL_SRWLOCK,
    FirstWaitBlock: PRTLP_SRWLOCK_WAITBLOCK,
) {
    let Next: PRTLP_SRWLOCK_WAITBLOCK;
    let mut NewValue: LONG_PTR;

    Next = (*FirstWaitBlock).Next as PRTLP_SRWLOCK_WAITBLOCK;
    if !Next.is_null() {
        NewValue = (Next as LONG_PTR) | RTL_SRWLOCK_OWNED as LONG_PTR | RTL_SRWLOCK_CONTENDED as LONG_PTR;
        if (*FirstWaitBlock).Exclusive == 0 {
            (*Next).SharedCount = (*FirstWaitBlock).SharedCount;

            NewValue |= RTL_SRWLOCK_SHARED as LONG_PTR;
        }

        (*Next).Last = (*FirstWaitBlock).Last;
    } else {
        if (*FirstWaitBlock).Exclusive != 0 {
            NewValue = RTL_SRWLOCK_OWNED as LONG_PTR;
        } else {
            NewValue = (((*FirstWaitBlock).SharedCount as LONG_PTR) << RTL_SRWLOCK_BITS)
                | RTL_SRWLOCK_SHARED as LONG_PTR
                | RTL_SRWLOCK_OWNED as LONG_PTR;
        }
    }

    // Read the waiter's block before the lock word names a new owner (see SrwSignalWake).
    let Exclusive = (*FirstWaitBlock).Exclusive != 0;
    let mut WakeChain: PRTLP_SRWLOCK_SHARED_WAKE =
        (*FirstWaitBlock).__bindgen_anon_1.__bindgen_anon_1.SharedWakeChain;

    let _ = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*SRWLock).Ptr),
        NewValue as PVOID,
    );

    if Exclusive {
        SrwSignalWake(core::ptr::addr_of_mut!((*FirstWaitBlock).__bindgen_anon_1.Wake));
    } else {
        let mut NextWake: PRTLP_SRWLOCK_SHARED_WAKE;
        loop {
            NextWake = (*WakeChain).Next as PRTLP_SRWLOCK_SHARED_WAKE;

            SrwSignalWake(core::ptr::addr_of_mut!((*WakeChain).Wake) as *mut LONG);

            WakeChain = NextWake;
            if WakeChain.is_null() {
                break;
            }
        }
    }
}
unsafe fn RtlpReleaseWaitBlockLockLastShared(
    SRWLock: PRTL_SRWLOCK,
    FirstWaitBlock: PRTLP_SRWLOCK_WAITBLOCK,
) {
    let Next: PRTLP_SRWLOCK_WAITBLOCK;
    let NewValue: LONG_PTR;

    Next = (*FirstWaitBlock).Next;
    if !Next.is_null() {
        NewValue = (Next as LONG_PTR) | RTL_SRWLOCK_OWNED as LONG_PTR | RTL_SRWLOCK_CONTENDED as LONG_PTR;

        (*Next).Last = (*FirstWaitBlock).Last;
    } else {
        NewValue = RTL_SRWLOCK_OWNED as LONG_PTR;
    }

    InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*SRWLock).Ptr),
        NewValue as PVOID,
    );

    SrwSignalWake(core::ptr::addr_of_mut!((*FirstWaitBlock).__bindgen_anon_1.Wake));
}
unsafe fn RtlpReleaseWaitBlockLock(SRWLock: PRTL_SRWLOCK) {
    InterlockedAndPointer(
        core::ptr::addr_of_mut!((*SRWLock).Ptr),
        !RTL_SRWLOCK_CONTENTION_LOCK as LONG_PTR,
    );
}
/// Take the contention bit (the right to edit the wait-block chain) of a contended lock, and
/// return its first wait block; NULL if the lock is no longer contended, for the caller to retry.
///
/// DIVERGES from srw.c, which ORs the bit in first and looks second: on a word that turned out
/// uncontended it then ANDs the bit back out. If another thread had meanwhile CASed in a word
/// without the bit (it compares the value WITH the bit, and builds its new value without it),
/// and a third took the bit legitimately, that AND clears the third thread's bit and lets a
/// fourth in beside it: two threads editing the chain, waiters dropped with no wake.
/// rtl-sync-difftest's spin-only (kernel) build hung that way, six waiters parked on a lock word
/// of 0. Here the bit is only ever set by a CAS on a contended word, so whoever clears it owns it.
unsafe fn RtlpAcquireWaitBlockLock(SRWLock: PRTL_SRWLOCK) -> PRTLP_SRWLOCK_WAITBLOCK {
    let mut spins: u32 = 0;
    loop {
        let CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const LONG_PTR);
        if (CurrentValue & RTL_SRWLOCK_CONTENDED as LONG_PTR) == 0
            || (CurrentValue & !(RTL_SRWLOCK_MASK as LONG_PTR)) == 0
        {
            return core::ptr::null_mut();
        }
        if (CurrentValue & RTL_SRWLOCK_CONTENTION_LOCK as LONG_PTR) == 0
            && InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*SRWLock).Ptr),
                (CurrentValue | RTL_SRWLOCK_CONTENTION_LOCK as LONG_PTR) as PVOID,
                CurrentValue as PVOID,
            ) as LONG_PTR
                == CurrentValue
        {
            return (CurrentValue & !(RTL_SRWLOCK_MASK as LONG_PTR)) as PRTLP_SRWLOCK_WAITBLOCK;
        }
        SrwBackoff(&mut spins);
    }
}
/// Wait until the lock is handed to `WaitBlock`. srw.c also stopped early when the lock word
/// read "owned, not contended", which is what the handoff writes just before it sets `Wake`;
/// waiting for `Wake` alone gets the same answer, and the releaser's last touch of the block
/// (that `Wake` store) can no longer land on a stack frame the waiter already left.
unsafe fn RtlpAcquireSRWLockExclusiveWait(
    _SRWLock: PRTL_SRWLOCK,
    WaitBlock: PRTLP_SRWLOCK_WAITBLOCK,
) {
    SrwWaitForWake(core::ptr::addr_of_mut!((*WaitBlock).__bindgen_anon_1.Wake));
}

// Hand-translated from srw.c:265-312 (rustconv's scanner dropped this static).
// Twin of RtlpAcquireSRWLockExclusiveWait above, for the shared-acquire path. srw.c's
// FirstWait == NULL arm also polled the lock word ("collapsed to a simple shared lock"); every
// handoff that collapses it then wakes the whole shared chain, ours included, so `Wake` alone
// answers both arms, for the reason given above.
unsafe fn RtlpAcquireSRWLockSharedWait(
    _SRWLock: PRTL_SRWLOCK,
    _FirstWait: PRTLP_SRWLOCK_WAITBLOCK,
    WakeChain: PRTLP_SRWLOCK_SHARED_WAKE,
) {
    SrwWaitForWake(core::ptr::addr_of_mut!((*WakeChain).Wake) as *mut LONG);
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeSRWLock(SRWLock: PRTL_SRWLOCK) {
    (*SRWLock).Ptr = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn RtlAcquireSRWLockShared(SRWLock: PRTL_SRWLOCK) {
    let mut StackWaitBlock: RTLP_SRWLOCK_WAITBLOCK = core::mem::zeroed();
    let mut SharedWake: RTLP_SRWLOCK_SHARED_WAKE = core::mem::zeroed();
    let mut CurrentValue: isize;
    let mut NewValue: isize;
    let mut First: PRTLP_SRWLOCK_WAITBLOCK;
    let mut Shared: PRTLP_SRWLOCK_WAITBLOCK;
    let mut FirstWait: PRTLP_SRWLOCK_WAITBLOCK;

    let mut spins: u32 = 0;
    loop {
        CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const isize);

        if (CurrentValue & RTL_SRWLOCK_SHARED as isize) != 0 {
            if (CurrentValue & RTL_SRWLOCK_CONTENDED as isize) != 0 {
                First = RtlpAcquireWaitBlockLock(SRWLock);
                if !First.is_null() {
                    FirstWait = core::ptr::null_mut();

                    if (*First).Exclusive != 0 {
                        Shared = (*First).Last;

                        if (*Shared).Exclusive != 0 {
                            StackWaitBlock.Exclusive = 0;
                            StackWaitBlock.SharedCount = 1;
                            StackWaitBlock.Next = core::ptr::null_mut();
                            StackWaitBlock.Last = core::ptr::addr_of_mut!(StackWaitBlock);
                            StackWaitBlock.__bindgen_anon_1.__bindgen_anon_1.SharedWakeChain =
                                core::ptr::addr_of_mut!(SharedWake);

                            (*Shared).Next = core::ptr::addr_of_mut!(StackWaitBlock);
                            (*First).Last = core::ptr::addr_of_mut!(StackWaitBlock);

                            Shared = core::ptr::addr_of_mut!(StackWaitBlock);
                            FirstWait = core::ptr::addr_of_mut!(StackWaitBlock);
                        } else {
                            (*(*Shared).__bindgen_anon_1.__bindgen_anon_1.LastSharedWake).Next =
                                core::ptr::addr_of_mut!(SharedWake);
                            (*Shared).SharedCount += 1;
                        }
                    } else {
                        Shared = First;
                        (*(*Shared).__bindgen_anon_1.__bindgen_anon_1.LastSharedWake).Next =
                            core::ptr::addr_of_mut!(SharedWake);
                        (*Shared).SharedCount += 1;
                    }

                    SharedWake.Next = core::ptr::null_mut();
                    SharedWake.Wake = 0;

                    (*Shared).__bindgen_anon_1.__bindgen_anon_1.LastSharedWake =
                        core::ptr::addr_of_mut!(SharedWake);

                    RtlpReleaseWaitBlockLock(SRWLock);

                    RtlpAcquireSRWLockSharedWait(
                        SRWLock,
                        FirstWait,
                        core::ptr::addr_of_mut!(SharedWake),
                    );

                    break;
                }
            } else {
                NewValue = (CurrentValue >> RTL_SRWLOCK_BITS) + 1;
                NewValue = (NewValue << RTL_SRWLOCK_BITS) | (CurrentValue & RTL_SRWLOCK_MASK as isize);

                if InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!((*SRWLock).Ptr),
                    NewValue as *mut core::ffi::c_void,
                    CurrentValue as *mut core::ffi::c_void,
                ) as isize == CurrentValue
                {
                    break;
                }
            }
        } else {
            if (CurrentValue & RTL_SRWLOCK_OWNED as isize) != 0 {
                if (CurrentValue & RTL_SRWLOCK_CONTENDED as isize) != 0 {
                    SharedWake.Next = core::ptr::null_mut();
                    SharedWake.Wake = 0;

                    StackWaitBlock.Exclusive = 0;
                    StackWaitBlock.SharedCount = 0;
                    StackWaitBlock.Next = core::ptr::null_mut();
                    StackWaitBlock.Last = core::ptr::addr_of_mut!(StackWaitBlock);
                    StackWaitBlock.__bindgen_anon_1.__bindgen_anon_1.SharedWakeChain =
                        core::ptr::addr_of_mut!(SharedWake);

                    First = RtlpAcquireWaitBlockLock(SRWLock);
                    if !First.is_null() {
                        Shared = (*First).Last;
                        if (*Shared).Exclusive != 0 {
                            (*Shared).Next = core::ptr::addr_of_mut!(StackWaitBlock);
                            (*First).Last = core::ptr::addr_of_mut!(StackWaitBlock);

                            Shared = core::ptr::addr_of_mut!(StackWaitBlock);
                            FirstWait = core::ptr::addr_of_mut!(StackWaitBlock);
                        } else {
                            FirstWait = core::ptr::null_mut();
                            (*(*Shared).__bindgen_anon_1.__bindgen_anon_1.LastSharedWake).Next =
                                core::ptr::addr_of_mut!(SharedWake);
                        }

                        (*Shared).SharedCount += 1;
                        (*Shared).__bindgen_anon_1.__bindgen_anon_1.LastSharedWake =
                            core::ptr::addr_of_mut!(SharedWake);

                        RtlpReleaseWaitBlockLock(SRWLock);

                        RtlpAcquireSRWLockSharedWait(
                            SRWLock,
                            FirstWait,
                            core::ptr::addr_of_mut!(SharedWake),
                        );

                        break;
                    }
                } else {
                    SharedWake.Next = core::ptr::null_mut();
                    SharedWake.Wake = 0;

                    StackWaitBlock.Exclusive = 0;
                    StackWaitBlock.SharedCount = 1;
                    StackWaitBlock.Next = core::ptr::null_mut();
                    StackWaitBlock.Last = core::ptr::addr_of_mut!(StackWaitBlock);
                    StackWaitBlock.__bindgen_anon_1.__bindgen_anon_1.SharedWakeChain =
                        core::ptr::addr_of_mut!(SharedWake);
                    StackWaitBlock.__bindgen_anon_1.__bindgen_anon_1.LastSharedWake =
                        core::ptr::addr_of_mut!(SharedWake);

                    NewValue = (core::ptr::addr_of_mut!(StackWaitBlock) as usize) as isize
                        | RTL_SRWLOCK_OWNED as isize
                        | RTL_SRWLOCK_CONTENDED as isize;
                    if InterlockedCompareExchangePointer(
                        core::ptr::addr_of_mut!((*SRWLock).Ptr),
                        NewValue as *mut core::ffi::c_void,
                        CurrentValue as *mut core::ffi::c_void,
                    ) as isize == CurrentValue
                    {
                        RtlpAcquireSRWLockSharedWait(
                            SRWLock,
                            core::ptr::addr_of_mut!(StackWaitBlock),
                            core::ptr::addr_of_mut!(SharedWake),
                        );

                        break;
                    }
                }
            } else {
                NewValue = (1 << RTL_SRWLOCK_BITS)
                    | RTL_SRWLOCK_SHARED as isize
                    | RTL_SRWLOCK_OWNED as isize;

                if InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!((*SRWLock).Ptr),
                    NewValue as *mut core::ffi::c_void,
                    CurrentValue as *mut core::ffi::c_void,
                ) as isize == CurrentValue
                {
                    break;
                }
            }
        }

        SrwBackoff(&mut spins);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleaseSRWLockShared(SRWLock: PRTL_SRWLOCK) {
    let mut CurrentValue: isize;
    let mut NewValue: isize;
    let mut WaitBlock: PRTLP_SRWLOCK_WAITBLOCK;
    let LastShared: bool;

    let mut spins: u32 = 0;
    loop {
        CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const isize);

        if (CurrentValue & RTL_SRWLOCK_SHARED as isize) != 0 {
            if (CurrentValue & RTL_SRWLOCK_CONTENDED as isize) != 0 {
                WaitBlock = RtlpAcquireWaitBlockLock(SRWLock);
                if !WaitBlock.is_null() {
                    (*WaitBlock).SharedCount -= 1;
                    LastShared = (*WaitBlock).SharedCount == 0;

                    if LastShared {
                        RtlpReleaseWaitBlockLockLastShared(SRWLock, WaitBlock);
                    } else {
                        RtlpReleaseWaitBlockLock(SRWLock);
                    }

                    break;
                }
            } else {
                NewValue = CurrentValue >> RTL_SRWLOCK_BITS;

                NewValue -= 1;
                if NewValue != 0 {
                    NewValue = (NewValue << RTL_SRWLOCK_BITS)
                        | RTL_SRWLOCK_SHARED as isize
                        | RTL_SRWLOCK_OWNED as isize;
                }

                if InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!((*SRWLock).Ptr),
                    NewValue as *mut core::ffi::c_void,
                    CurrentValue as *mut core::ffi::c_void,
                ) as isize
                    == CurrentValue
                {
                    break;
                }
            }
        } else {
            RtlRaiseStatus(STATUS_RESOURCE_NOT_OWNED);
        }

        SrwBackoff(&mut spins);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlAcquireSRWLockExclusive(SRWLock: PRTL_SRWLOCK) {
    let mut StackWaitBlock: RTLP_SRWLOCK_WAITBLOCK = core::mem::zeroed();
    let mut First: PRTLP_SRWLOCK_WAITBLOCK;
    let Last: PRTLP_SRWLOCK_WAITBLOCK;

    if InterlockedBitTestAndSetPointer(
        core::ptr::addr_of_mut!((*SRWLock).Ptr) as *mut core::ffi::c_long,
        RTL_SRWLOCK_OWNED_BIT as core::ffi::c_long,
    ) != 0
    {
        let mut CurrentValue: isize;
        let mut NewValue: isize;

        let mut spins: u32 = 0;
        loop {
            CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const isize);

            if (CurrentValue & RTL_SRWLOCK_SHARED as isize) != 0 {
                let mut do_add_wait_block = false;

                if (CurrentValue & RTL_SRWLOCK_CONTENDED as isize) != 0 {
                    do_add_wait_block = true;
                } else {
                    StackWaitBlock.Exclusive = 1u8;
                    StackWaitBlock.SharedCount = (CurrentValue >> RTL_SRWLOCK_BITS) as LONG;
                    StackWaitBlock.Next = core::ptr::null_mut();
                    StackWaitBlock.Last = core::ptr::addr_of_mut!(StackWaitBlock);
                    StackWaitBlock.__bindgen_anon_1.Wake = 0;

                    NewValue = (core::ptr::addr_of_mut!(StackWaitBlock) as usize as isize)
                        | RTL_SRWLOCK_SHARED as isize
                        | RTL_SRWLOCK_CONTENDED as isize
                        | RTL_SRWLOCK_OWNED as isize;

                    if (InterlockedCompareExchangePointer(
                        core::ptr::addr_of_mut!((*SRWLock).Ptr),
                        NewValue as *mut core::ffi::c_void,
                        CurrentValue as *mut core::ffi::c_void,
                    ) as isize)
                        == CurrentValue
                    {
                        RtlpAcquireSRWLockExclusiveWait(SRWLock, core::ptr::addr_of_mut!(StackWaitBlock));
                        break;
                    }
                }

                if do_add_wait_block {
                    StackWaitBlock.Exclusive = 1u8;
                    StackWaitBlock.SharedCount = 0;
                    StackWaitBlock.Next = core::ptr::null_mut();
                    StackWaitBlock.Last = core::ptr::addr_of_mut!(StackWaitBlock);
                    StackWaitBlock.__bindgen_anon_1.Wake = 0;

                    First = RtlpAcquireWaitBlockLock(SRWLock);
                    if !First.is_null() {
                        Last = (*First).Last;
                        (*Last).Next = core::ptr::addr_of_mut!(StackWaitBlock);
                        (*First).Last = core::ptr::addr_of_mut!(StackWaitBlock);

                        RtlpReleaseWaitBlockLock(SRWLock);

                        RtlpAcquireSRWLockExclusiveWait(SRWLock, core::ptr::addr_of_mut!(StackWaitBlock));
                        break;
                    }
                }
            } else {
                if (CurrentValue & RTL_SRWLOCK_OWNED as isize) != 0 {
                    if (CurrentValue & RTL_SRWLOCK_CONTENDED as isize) != 0 {
                        StackWaitBlock.Exclusive = 1u8;
                        StackWaitBlock.SharedCount = 0;
                        StackWaitBlock.Next = core::ptr::null_mut();
                        StackWaitBlock.Last = core::ptr::addr_of_mut!(StackWaitBlock);
                        StackWaitBlock.__bindgen_anon_1.Wake = 0;

                        First = RtlpAcquireWaitBlockLock(SRWLock);
                        if !First.is_null() {
                            Last = (*First).Last;
                            (*Last).Next = core::ptr::addr_of_mut!(StackWaitBlock);
                            (*First).Last = core::ptr::addr_of_mut!(StackWaitBlock);

                            RtlpReleaseWaitBlockLock(SRWLock);

                            RtlpAcquireSRWLockExclusiveWait(SRWLock, core::ptr::addr_of_mut!(StackWaitBlock));
                            break;
                        }
                    } else {
                        StackWaitBlock.Exclusive = 1u8;
                        StackWaitBlock.SharedCount = 0;
                        StackWaitBlock.Next = core::ptr::null_mut();
                        StackWaitBlock.Last = core::ptr::addr_of_mut!(StackWaitBlock);
                        StackWaitBlock.__bindgen_anon_1.Wake = 0;

                        NewValue = (core::ptr::addr_of_mut!(StackWaitBlock) as usize as isize)
                            | RTL_SRWLOCK_OWNED as isize
                            | RTL_SRWLOCK_CONTENDED as isize;
                        if (InterlockedCompareExchangePointer(
                            core::ptr::addr_of_mut!((*SRWLock).Ptr),
                            NewValue as *mut core::ffi::c_void,
                            CurrentValue as *mut core::ffi::c_void,
                        ) as isize)
                            == CurrentValue
                        {
                            RtlpAcquireSRWLockExclusiveWait(SRWLock, core::ptr::addr_of_mut!(StackWaitBlock));
                            break;
                        }
                    }
                } else {
                    if InterlockedBitTestAndSetPointer(
                        core::ptr::addr_of_mut!((*SRWLock).Ptr) as *mut core::ffi::c_long,
                        RTL_SRWLOCK_OWNED_BIT as core::ffi::c_long,
                    ) == 0
                    {
                        break;
                    }
                }
            }

            SrwBackoff(&mut spins);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleaseSRWLockExclusive(SRWLock: PRTL_SRWLOCK) {
    let mut CurrentValue: LONG_PTR;
    let mut NewValue: LONG_PTR;
    let mut WaitBlock: PRTLP_SRWLOCK_WAITBLOCK;

    let mut spins: u32 = 0;
    loop {
        CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const LONG_PTR);

        if (CurrentValue & RTL_SRWLOCK_OWNED as LONG_PTR) == 0 {
            RtlRaiseStatus(STATUS_RESOURCE_NOT_OWNED);
        }

        if (CurrentValue & RTL_SRWLOCK_SHARED as LONG_PTR) == 0 {
            if (CurrentValue & RTL_SRWLOCK_CONTENDED as LONG_PTR) != 0 {
                WaitBlock = RtlpAcquireWaitBlockLock(SRWLock);
                if !WaitBlock.is_null() {
                    RtlpReleaseWaitBlockLockExclusive(SRWLock, WaitBlock);

                    break;
                }
            } else {
                NewValue = 0;
                if InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!((*SRWLock).Ptr),
                    NewValue as PVOID,
                    CurrentValue as PVOID,
                ) as LONG_PTR
                    == CurrentValue
                {
                    break;
                }
            }
        } else {
            RtlRaiseStatus(STATUS_RESOURCE_NOT_OWNED);
        }

        SrwBackoff(&mut spins);
    }
}

/// DIVERGES from srw.c, whose body is wrong two ways. It builds the new word as
/// `((old >> BITS) + 1) | SHARED | OWNED` and never shifts the count back, so a try on a FREE
/// lock stores 7: CONTENDED with no wait block, after which every acquire loops forever looking
/// for one (RtlpAcquireWaitBlockLock finds a null block and backs off). And its compare value
/// keeps OWNED, so on a lock held EXCLUSIVE it compares 1, stores 7 and returns FALSE, having
/// corrupted the word anyway. Both found by rtl-sync-difftest, which hung within a dozen
/// sections once try-acquires joined the mix. What it means, as Windows does: take a share of a
/// lock that is free or held shared with no waiters, else fail without writing.
#[no_mangle]
pub unsafe extern "system" fn RtlTryAcquireSRWLockShared(SRWLock: PRTL_SRWLOCK) -> BOOLEAN {
    loop {
        let CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const LONG_PTR);
        let NewValue = if CurrentValue == 0 {
            (1 << RTL_SRWLOCK_BITS) | RTL_SRWLOCK_SHARED as LONG_PTR | RTL_SRWLOCK_OWNED as LONG_PTR
        } else if (CurrentValue & RTL_SRWLOCK_MASK as LONG_PTR)
            == (RTL_SRWLOCK_SHARED as LONG_PTR | RTL_SRWLOCK_OWNED as LONG_PTR)
        {
            CurrentValue + (1 << RTL_SRWLOCK_BITS)
        } else {
            return 0u8;
        };

        if InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*SRWLock).Ptr),
            NewValue as *mut _,
            CurrentValue as *mut _,
        ) as LONG_PTR
            == CurrentValue
        {
            return 1u8;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlTryAcquireSRWLockExclusive(SRWLock: PRTL_SRWLOCK) -> BOOLEAN {
    if InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!((*SRWLock).Ptr),
        RTL_SRWLOCK_OWNED as *mut core::ffi::c_void,
        core::ptr::null_mut(),
    ) == core::ptr::null_mut()
    {
        1u8
    } else {
        0u8
    }
}
