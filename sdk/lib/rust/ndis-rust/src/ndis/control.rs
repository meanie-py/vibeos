use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, PsGetCurrentThread};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #undef NdisAcquireSpinLock, NdisAllocateSpinLock, NdisDprAcquireSpinLock, NdisDprReleaseSpinLock, NdisFreeSpinLock, NdisReleaseSpinLock: preprocessor directives with no Rust declaration to emit.
#[no_mangle]
pub unsafe extern "system" fn NdisInitializeReadWriteLock(Lock: PNDIS_RW_LOCK) {
    RtlZeroMemory!(Lock as *mut core::ffi::c_void, core::mem::size_of::<NDIS_RW_LOCK>());

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.__bindgen_anon_1.SpinLock));
}
#[no_mangle]
pub unsafe extern "system" fn NdisAcquireReadWriteLock(
    Lock: PNDIS_RW_LOCK,
    fWrite: BOOLEAN,
    LockState: PLOCK_STATE,
) {
    let mut RefCount: ULONG;
    let mut ProcessorNumber: UCHAR;
    let mut BusyLoop: UCHAR = core::mem::zeroed();

    ASSERT_IRQL!(DISPATCH_LEVEL);

    if fWrite != 0 {
        if (*Lock).__bindgen_anon_1.__bindgen_anon_1.Context == (PsGetCurrentThread()).cast() {
            (*LockState).LockState = 2;
        } else {
            KeAcquireSpinLock(
                core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.__bindgen_anon_1.SpinLock),
                core::ptr::addr_of_mut!((*LockState).OldIrql),
            );
            /* Check if any other processor helds a shared lock. */
            ProcessorNumber = KeNumberProcessorsValue() as UCHAR;
            loop {
                let fresh = ProcessorNumber;
                ProcessorNumber = ProcessorNumber.wrapping_sub(1);
                if !(fresh != 0) {
                    break;
                }
                if ProcessorNumber as ULONG != KeGetCurrentProcessorNumber() {
                    /* Wait till the shared lock is released. */
                    while (*Lock).__bindgen_anon_2.RefCount[ProcessorNumber as usize].RefCount
                        != 0
                    {
                        core::ptr::write_volatile(
                            core::ptr::addr_of_mut!(BusyLoop),
                            32 as UCHAR,
                        );
                        loop {
                            let fresh2 = core::ptr::read_volatile(
                                core::ptr::addr_of!(BusyLoop),
                            );
                            core::ptr::write_volatile(
                                core::ptr::addr_of_mut!(BusyLoop),
                                core::ptr::read_volatile(core::ptr::addr_of!(BusyLoop))
                                    .wrapping_sub(1),
                            );
                            if !(fresh2 != 0) {
                                break;
                            }
                        }
                    }
                }
            }
            (*Lock).__bindgen_anon_1.__bindgen_anon_1.Context = (PsGetCurrentThread()).cast();
            (*LockState).LockState = 4;
        }
    } else {
        KeRaiseIrql(
            (DISPATCH_LEVEL) as KIRQL,
            core::ptr::addr_of_mut!((*LockState).OldIrql),
        );
        let idx = KeGetCurrentProcessorNumber() as usize;
        RefCount = InterlockedIncrement(
            core::ptr::addr_of_mut!((*Lock).__bindgen_anon_2.RefCount[idx].RefCount) as PLONG,
        ) as ULONG;
        /* Racing with a exclusive write lock case. */
        if (*Lock).__bindgen_anon_1.__bindgen_anon_1.SpinLock != 0 {
            if RefCount == 1 {
                if (*Lock).__bindgen_anon_1.__bindgen_anon_1.Context != (PsGetCurrentThread()).cast() {
                    /* Wait for the exclusive lock to be released. */
                    let p = KeGetCurrentProcessorNumber() as usize;
                    (*Lock).__bindgen_anon_2.RefCount[p].RefCount = (*Lock).__bindgen_anon_2.RefCount[p].RefCount.wrapping_sub(1);
                    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.__bindgen_anon_1.SpinLock));
                    let q = KeGetCurrentProcessorNumber() as usize;
                    (*Lock).__bindgen_anon_2.RefCount[q].RefCount = (*Lock).__bindgen_anon_2.RefCount[q].RefCount.wrapping_add(1);
                    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.__bindgen_anon_1.SpinLock));
                }
            }
        }
        (*Lock).__bindgen_anon_1.__bindgen_anon_1.Context = (PsGetCurrentThread()).cast();
        (*LockState).LockState = 3;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisReleaseReadWriteLock(
    Lock: PNDIS_RW_LOCK,
    LockState: PLOCK_STATE,
) {
    match (*LockState).LockState as i32 {
        2 => {
            return;
        }
        3 => {
            let Idx = KeGetCurrentProcessorNumber() as usize;
            (*Lock).__bindgen_anon_2.RefCount[Idx].RefCount = (*Lock).__bindgen_anon_2.RefCount[Idx].RefCount.wrapping_sub(1);
            (*LockState).LockState = (-(1 as i32)) as USHORT;
            if ((*LockState).OldIrql as i32) < (DISPATCH_LEVEL as i32) {
                KeLowerIrql((*LockState).OldIrql);
            }
            return;
        }
        4 => {
            (*Lock).__bindgen_anon_1.__bindgen_anon_1.Context = core::ptr::null_mut();
            (*LockState).LockState = (-(1 as i32)) as USHORT;
            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.__bindgen_anon_1.SpinLock),
                (*LockState).OldIrql,
            );
            return;
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisAcquireSpinLock(SpinLock: PNDIS_SPIN_LOCK) {
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*SpinLock).SpinLock),
        core::ptr::addr_of_mut!((*SpinLock).OldIrql),
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocateSpinLock(SpinLock: PNDIS_SPIN_LOCK) {
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*SpinLock).SpinLock));
}
#[no_mangle]
pub unsafe extern "system" fn NdisDprAcquireSpinLock(SpinLock: PNDIS_SPIN_LOCK) {
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*SpinLock).SpinLock));
    (*SpinLock).OldIrql = DISPATCH_LEVEL as KIRQL;
}
#[no_mangle]
pub unsafe extern "system" fn NdisDprReleaseSpinLock(SpinLock: PNDIS_SPIN_LOCK) {
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SpinLock).SpinLock));
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeSpinLock(SpinLock: PNDIS_SPIN_LOCK) {}
#[no_mangle]
pub unsafe extern "system" fn NdisInitializeEvent(Event: PNDIS_EVENT) {
    KeInitializeEvent(core::ptr::addr_of_mut!((*Event).Event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
}
#[no_mangle]
pub unsafe extern "system" fn NdisReleaseSpinLock(SpinLock: PNDIS_SPIN_LOCK) {
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*SpinLock).SpinLock), (*SpinLock).OldIrql);
}
#[no_mangle]
pub unsafe extern "system" fn NdisResetEvent(Event: PNDIS_EVENT) {
    KeClearEvent(core::ptr::addr_of_mut!((*Event).Event));
}
#[no_mangle]
pub unsafe extern "system" fn NdisSetEvent(Event: PNDIS_EVENT) {
    KeSetEvent(core::ptr::addr_of_mut!((*Event).Event), IO_NO_INCREMENT as KPRIORITY, FALSE as BOOLEAN);
}
#[no_mangle]
pub unsafe extern "system" fn NdisWaitEvent(
    mut Event: PNDIS_EVENT,
    mut MsToWait: UINT,
) -> BOOLEAN {
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    Timeout.QuadPart = Int32x32To64!(MsToWait as i32, -10000);

    // MsToWait 0 means wait until the event is set (the NDIS contract). The C turned it into
    // a zero timeout, a poll that fails at once. Realtek's WDI driver (rtwlanu) waits with
    // MsToWait 0 at four places.
    Status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*Event).Event) as PVOID,
        _KWAIT_REASON_Executive,
        (0i32) as KPROCESSOR_MODE,
        TRUE as BOOLEAN,
        if MsToWait == 0 { core::ptr::null_mut() } else { core::ptr::addr_of_mut!(Timeout) },
    );

    return (Status == STATUS_SUCCESS) as BOOLEAN;
}
