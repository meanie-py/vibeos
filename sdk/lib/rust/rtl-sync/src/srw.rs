use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::{InterlockedCompareExchangePointer, InterlockedExchangePointer};
use ntkrnl_sys::rtl::RtlRaiseStatus;
use ntkrnl_sys::ddk::PRTL_SRWLOCK;
// YieldProcessor + the pointer-width Interlocked* / bit-test intrinsics live in
// the cluster's atomics substrate (ntkrnl-sys only surfaces the base CAS).
use crate::atomics::*;

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

    let _ = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*SRWLock).Ptr),
        NewValue as PVOID,
    );

    if (*FirstWaitBlock).Exclusive != 0 {
        let _ = InterlockedOr(
            core::ptr::addr_of_mut!((*FirstWaitBlock).__bindgen_anon_1.Wake),
            1,
        );
    } else {
        let mut WakeChain: PRTLP_SRWLOCK_SHARED_WAKE;
        let mut NextWake: PRTLP_SRWLOCK_SHARED_WAKE;

        WakeChain = (*FirstWaitBlock).__bindgen_anon_1.__bindgen_anon_1.SharedWakeChain;
        loop {
            NextWake = (*WakeChain).Next as PRTLP_SRWLOCK_SHARED_WAKE;

            let _ = InterlockedOr(
                core::ptr::addr_of_mut!((*WakeChain).Wake) as *mut LONG,
                1,
            );

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

    InterlockedOr(
        core::ptr::addr_of_mut!((*FirstWaitBlock).__bindgen_anon_1.Wake),
        1 as LONG,
    );
}
unsafe fn RtlpReleaseWaitBlockLock(SRWLock: PRTL_SRWLOCK) {
    InterlockedAndPointer(
        core::ptr::addr_of_mut!((*SRWLock).Ptr),
        !RTL_SRWLOCK_CONTENTION_LOCK as LONG_PTR,
    );
}
unsafe fn RtlpAcquireWaitBlockLock(SRWLock: PRTL_SRWLOCK) -> PRTLP_SRWLOCK_WAITBLOCK {
    let mut PrevValue: LONG_PTR;

    loop {
        PrevValue = InterlockedOrPointer(
            core::ptr::addr_of_mut!((*SRWLock).Ptr),
            RTL_SRWLOCK_CONTENTION_LOCK as LONG_PTR,
        ) as LONG_PTR;

        if (PrevValue & RTL_SRWLOCK_CONTENTION_LOCK as LONG_PTR) == 0 {
            break;
        }

        YieldProcessor();
    }

    if (PrevValue & RTL_SRWLOCK_CONTENDED as LONG_PTR) == 0
        || (PrevValue & !(RTL_SRWLOCK_MASK as LONG_PTR)) == 0
    {
        RtlpReleaseWaitBlockLock(SRWLock);
        return core::ptr::null_mut();
    }

    let WaitBlock = (PrevValue & !(RTL_SRWLOCK_MASK as LONG_PTR)) as PRTLP_SRWLOCK_WAITBLOCK;

    WaitBlock
}
unsafe fn RtlpAcquireSRWLockExclusiveWait(
    SRWLock: PRTL_SRWLOCK,
    WaitBlock: PRTLP_SRWLOCK_WAITBLOCK,
) {
    let mut CurrentValue: isize;

    loop {
        CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const isize);
        if (CurrentValue & RTL_SRWLOCK_SHARED as isize) == 0 {
            if (CurrentValue & RTL_SRWLOCK_CONTENDED as isize) != 0 {
                if (*WaitBlock).__bindgen_anon_1.Wake != 0 {
                    break;
                }
            } else {
                break;
            }
        }

        YieldProcessor();
    }
}

// Hand-translated from srw.c:265-312 (rustconv's scanner dropped this static).
// Twin of RtlpAcquireSRWLockExclusiveWait above, for the shared-acquire path.
unsafe fn RtlpAcquireSRWLockSharedWait(
    SRWLock: PRTL_SRWLOCK,
    FirstWait: PRTLP_SRWLOCK_WAITBLOCK,
    WakeChain: PRTLP_SRWLOCK_SHARED_WAKE,
) {
    if !FirstWait.is_null() {
        while (*WakeChain).Wake == 0 {
            YieldProcessor();
        }
    } else {
        let mut CurrentValue: LONG_PTR;

        loop {
            CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const LONG_PTR);
            if (CurrentValue & RTL_SRWLOCK_SHARED as LONG_PTR) != 0 {
                // RTL_SRWLOCK_OWNED is always set when RTL_SRWLOCK_SHARED is.
                if (CurrentValue & RTL_SRWLOCK_CONTENDED as LONG_PTR) != 0 {
                    if (*WakeChain).Wake != 0 {
                        // Our wait block became first in the chain — we own it.
                        break;
                    }
                } else {
                    // Collapsed to a simple shared lock — acquired.
                    break;
                }
            }

            YieldProcessor();
        }
    }
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

        YieldProcessor();
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleaseSRWLockShared(SRWLock: PRTL_SRWLOCK) {
    let mut CurrentValue: isize;
    let mut NewValue: isize;
    let mut WaitBlock: PRTLP_SRWLOCK_WAITBLOCK;
    let LastShared: bool;

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

        YieldProcessor();
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

        loop {
            CurrentValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const isize);

            if (CurrentValue & RTL_SRWLOCK_SHARED as isize) != 0 {
                let mut do_add_wait_block = false;

                if (CurrentValue & RTL_SRWLOCK_CONTENDED as isize) != 0 {
                    do_add_wait_block = true;
                } else {
                    StackWaitBlock.Exclusive = 1u8;
                    StackWaitBlock.SharedCount = (CurrentValue >> RTL_SRWLOCK_BITS) as core::ffi::c_long;
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

            YieldProcessor();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleaseSRWLockExclusive(SRWLock: PRTL_SRWLOCK) {
    let mut CurrentValue: LONG_PTR;
    let mut NewValue: LONG_PTR;
    let mut WaitBlock: PRTLP_SRWLOCK_WAITBLOCK;

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

        YieldProcessor();
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlTryAcquireSRWLockShared(SRWLock: PRTL_SRWLOCK) -> BOOLEAN {
    let mut CompareValue: LONG_PTR;
    let mut NewValue: LONG_PTR;
    let mut GotValue: LONG_PTR;

    loop {
        CompareValue = *(core::ptr::addr_of!((*SRWLock).Ptr) as *const LONG_PTR);
        NewValue = ((CompareValue >> RTL_SRWLOCK_BITS) + 1)
            | RTL_SRWLOCK_SHARED as LONG_PTR
            | RTL_SRWLOCK_OWNED as LONG_PTR;

        CompareValue &= !(RTL_SRWLOCK_MASK as LONG_PTR)
            | RTL_SRWLOCK_SHARED as LONG_PTR
            | RTL_SRWLOCK_OWNED as LONG_PTR;

        GotValue = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*SRWLock).Ptr),
            NewValue as *mut _,
            CompareValue as *mut _,
        ) as LONG_PTR;

        if !((GotValue != CompareValue)
            && (((GotValue & RTL_SRWLOCK_MASK as LONG_PTR)
                == (RTL_SRWLOCK_SHARED as LONG_PTR | RTL_SRWLOCK_OWNED as LONG_PTR))
                || (GotValue == 0)))
        {
            break;
        }
    }

    if ((GotValue & RTL_SRWLOCK_MASK as LONG_PTR)
        == (RTL_SRWLOCK_SHARED as LONG_PTR | RTL_SRWLOCK_OWNED as LONG_PTR))
        || (GotValue == 0)
    {
        1u8
    } else {
        0u8
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
