use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::HANDLE;
use ntkrnl_sys::ddk::{LARGE_INTEGER, NtClose, NtCreateKeyedEvent, NtReleaseKeyedEvent, NtWaitForKeyedEvent, PLARGE_INTEGER, PRTL_CONDITION_VARIABLE, PRTL_CRITICAL_SECTION, PRTL_SRWLOCK};
// Atomics substrate (Acquire/Release CAS + YieldProcessor) and the keyed-event
// constant; the SRW / critical-section primitives are sibling modules so we
// call them directly (identical extern "system" ABI, no ddk decl mismatch).
use crate::atomics::*;
use crate::shared::EVENT_ALL_ACCESS;
// critical-section primitives: sibling module under `cabi` (kernel / ntdll-main),
// but resolved as ntdll.dll IMPORTS under `vista` (critical.c is not in the
// ntdll_vista.dll link — same extern behaviour the old condvar.c relied on).
#[cfg(feature = "cabi")]
use crate::critical::{RtlEnterCriticalSection, RtlLeaveCriticalSection};
#[cfg(all(feature = "vista", not(feature = "cabi")))]
extern "system" {
    fn RtlEnterCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlLeaveCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
}
use crate::srw::{RtlAcquireSRWLockExclusive, RtlAcquireSRWLockShared, RtlReleaseSRWLockExclusive, RtlReleaseSRWLockShared};

// winbase.h
const RTL_CONDITION_VARIABLE_LOCKMODE_SHARED: ULONG = 1;

// C: #define CONTAINING_COND_VAR_WAIT_ENTRY(addr, field)
//        CONTAINING_RECORD(addr, COND_VAR_WAIT_ENTRY, field)
macro_rules! CONTAINING_COND_VAR_WAIT_ENTRY {
    ($address:expr, $field:ident) => {
        (($address as usize) - core::mem::offset_of!(COND_VAR_WAIT_ENTRY, $field))
            as PCOND_VAR_WAIT_ENTRY
    };
}

// ---- file-scope preamble (translated) ----
pub const COND_VAR_UNUSED_FLAG: ULONG_PTR = 1;

pub const COND_VAR_LOCKED_FLAG: ULONG_PTR = 2;

pub const COND_VAR_FLAGS_MASK: ULONG_PTR = 3;

pub const COND_VAR_ADDRESS_MASK: ULONG_PTR = !COND_VAR_FLAGS_MASK;

#[repr(C)]
pub struct _COND_VAR_WAIT_ENTRY {
    pub ListEntry: LIST_ENTRY,
    pub WaitKey: PVOID,
    pub ListRemovalHandled: BOOLEAN,
}
pub type COND_VAR_WAIT_ENTRY = _COND_VAR_WAIT_ENTRY;
pub type PCOND_VAR_WAIT_ENTRY = *mut _COND_VAR_WAIT_ENTRY;

static mut CondVarKeyedEventHandle: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "C" fn InternalCmpXChgCondVarAcq(ConditionVariable: PRTL_CONDITION_VARIABLE,
                                    Exchange: usize,
                                    Comperand: usize) -> usize {
    InterlockedCompareExchangePointerAcquire(core::ptr::addr_of_mut!((*ConditionVariable).Ptr),
                                             Exchange as *mut core::ffi::c_void,
                                             Comperand as *mut core::ffi::c_void) as usize
}
#[no_mangle]
pub unsafe extern "C" fn InternalCmpXChgCondVarRel(ConditionVariable: PRTL_CONDITION_VARIABLE,
                                  Exchange: ULONG_PTR,
                                  Comperand: ULONG_PTR) -> ULONG_PTR {
    InterlockedCompareExchangePointerRelease(core::ptr::addr_of_mut!((*ConditionVariable).Ptr),
                                             Exchange as PVOID,
                                             Comperand as PVOID) as ULONG_PTR
}
#[no_mangle]
pub unsafe extern "C" fn InternalGetListRemovalHandledFlag(Entry: PCOND_VAR_WAIT_ENTRY) -> *mut BOOLEAN {
    core::ptr::addr_of_mut!((*Entry).ListRemovalHandled) as *mut BOOLEAN
}

unsafe fn InternalLockCondVar(ConditionVariable: PRTL_CONDITION_VARIABLE,
                               InsertEntry: *mut COND_VAR_WAIT_ENTRY,
                               AbortIfLocked: *mut u8) -> *mut COND_VAR_WAIT_ENTRY {
    let mut OldVal: usize = (*ConditionVariable).Ptr as usize;

    loop {
        let NewVal: usize;
        let LockRes: usize;
        let OldListHead: PLIST_ENTRY;

        if OldVal & COND_VAR_LOCKED_FLAG != 0 {
            YieldProcessor();

            if !AbortIfLocked.is_null() && *AbortIfLocked != 0 {
                return core::ptr::null_mut();
            }

            OldVal = *(core::ptr::addr_of_mut!((*ConditionVariable).Ptr) as *mut usize);
            continue;
        }

        OldListHead = (OldVal & COND_VAR_ADDRESS_MASK) as PLIST_ENTRY;
        if InsertEntry.is_null() {
            if OldListHead.is_null() {
                return core::ptr::null_mut();
            }

            NewVal = OldVal;
        } else {
            NewVal = (OldVal & COND_VAR_FLAGS_MASK) |
                     (core::ptr::addr_of_mut!((*InsertEntry).ListEntry) as usize);
        }

        let NewVal = NewVal | COND_VAR_LOCKED_FLAG;
        LockRes = InternalCmpXChgCondVarAcq(ConditionVariable, NewVal, OldVal);
        if LockRes == OldVal {
            if InsertEntry.is_null() {
                return CONTAINING_COND_VAR_WAIT_ENTRY!(OldListHead, ListEntry);
            }

            if OldListHead.is_null() {
                InitializeListHead(core::ptr::addr_of_mut!((*InsertEntry).ListEntry));
            } else {
                InsertTailList(OldListHead, core::ptr::addr_of_mut!((*InsertEntry).ListEntry));
            }

            return InsertEntry;
        }

        OldVal = LockRes;
    }
}
unsafe fn InternalUnlockCondVar(ConditionVariable: PRTL_CONDITION_VARIABLE,
                                RemoveEntry: PCOND_VAR_WAIT_ENTRY) {
    let mut OldVal = (*ConditionVariable).Ptr as ULONG_PTR;
    let mut NewHeadEntry: PLIST_ENTRY;

    NewHeadEntry = (OldVal & COND_VAR_ADDRESS_MASK) as PLIST_ENTRY;
    if RemoveEntry != core::ptr::null_mut() {
        if core::ptr::addr_of_mut!((*RemoveEntry).ListEntry) == NewHeadEntry {
            if !IsListEmpty(NewHeadEntry) {
                NewHeadEntry = (*NewHeadEntry).Flink;
                RemoveEntryList(core::ptr::addr_of_mut!((*RemoveEntry).ListEntry));
            } else {
                NewHeadEntry = core::ptr::null_mut();
            }
        } else {
            RemoveEntryList(core::ptr::addr_of_mut!((*RemoveEntry).ListEntry));
        }

        (*RemoveEntry).ListRemovalHandled = 1u8;
    }

    loop {
        let NewVal = (OldVal & (COND_VAR_FLAGS_MASK ^ COND_VAR_LOCKED_FLAG)) |
                     (NewHeadEntry as ULONG_PTR);
        let LockRes = InternalCmpXChgCondVarRel(ConditionVariable, NewVal, OldVal);
        if LockRes == OldVal {
            break;
        }

        OldVal = LockRes;
    }
}

unsafe fn InternalWake(ConditionVariable: PRTL_CONDITION_VARIABLE,
                        ReleaseAll: BOOLEAN) {
    let HeadEntry: PCOND_VAR_WAIT_ENTRY = InternalLockCondVar(ConditionVariable,
                                                              core::ptr::null_mut(),
                                                              core::ptr::null_mut());
    let mut Entry: PCOND_VAR_WAIT_ENTRY;
    let mut NextEntry: PCOND_VAR_WAIT_ENTRY;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut RemoveOnUnlockEntry: PCOND_VAR_WAIT_ENTRY;

    if HeadEntry.is_null() {
        return;
    }

    Timeout.QuadPart = 0;
    RemoveOnUnlockEntry = core::ptr::null_mut();

    Entry = CONTAINING_COND_VAR_WAIT_ENTRY!((*HeadEntry).ListEntry.Blink, ListEntry);
    while !Entry.is_null() {
        if HeadEntry == Entry {
            NextEntry = core::ptr::null_mut();
        } else {
            NextEntry = CONTAINING_COND_VAR_WAIT_ENTRY!((*Entry).ListEntry.Blink, ListEntry);
        }

        let Status = NtReleaseKeyedEvent(CondVarKeyedEventHandle,
                                         core::ptr::addr_of_mut!((*Entry).WaitKey) as *mut _,
                                         0u8,
                                         core::ptr::addr_of_mut!(Timeout));

        if !NT_SUCCESS(Status) {
            Entry = NextEntry;
            continue;
        }

        if HeadEntry == Entry {
            RemoveOnUnlockEntry = HeadEntry;
        } else {
            RemoveEntryList(core::ptr::addr_of_mut!((*Entry).ListEntry));
            *InternalGetListRemovalHandledFlag(Entry) = 1u8;
        }

        if ReleaseAll == 0 {
            break;
        }

        Entry = NextEntry;
    }

    InternalUnlockCondVar(ConditionVariable, RemoveOnUnlockEntry);
}
// Hand-translated from condvar.c:365-456 (rustconv's scanner dropped this static).
// Either CriticalSection or SRWLock must be NULL (not both); the lock is held on
// entry and re-held on return.
unsafe fn InternalSleep(
    ConditionVariable: PRTL_CONDITION_VARIABLE,
    CriticalSection: PRTL_CRITICAL_SECTION,
    SRWLock: PRTL_SRWLOCK,
    SRWFlags: ULONG,
    TimeOut: PLARGE_INTEGER,
) -> NTSTATUS {
    let mut OwnEntry: COND_VAR_WAIT_ENTRY = core::mem::zeroed();

    // Put OwnEntry on the list.
    InternalLockCondVar(
        ConditionVariable,
        core::ptr::addr_of_mut!(OwnEntry),
        core::ptr::null_mut(),
    );
    InternalUnlockCondVar(ConditionVariable, core::ptr::null_mut());

    // Drop the caller-provided lock as preparation for going to sleep.
    if CriticalSection.is_null() {
        if (RTL_CONDITION_VARIABLE_LOCKMODE_SHARED & SRWFlags) == 0 {
            RtlReleaseSRWLockExclusive(SRWLock);
        } else {
            RtlReleaseSRWLockShared(SRWLock);
        }
    } else {
        RtlLeaveCriticalSection(CriticalSection);
    }

    // Sleep using the caller-provided timeout.
    let Status: NTSTATUS = NtWaitForKeyedEvent(
        CondVarKeyedEventHandle,
        core::ptr::addr_of_mut!(OwnEntry.WaitKey) as PVOID,
        0u8,
        TimeOut,
    );

    if *InternalGetListRemovalHandledFlag(core::ptr::addr_of_mut!(OwnEntry)) == 0 {
        // OwnEntry still seems to be on the list; remove it once we hold the lock.
        if !InternalLockCondVar(
            ConditionVariable,
            core::ptr::null_mut(),
            InternalGetListRemovalHandledFlag(core::ptr::addr_of_mut!(OwnEntry)),
        )
        .is_null()
        {
            let RemoveEntry = if OwnEntry.ListRemovalHandled == 0 {
                core::ptr::addr_of_mut!(OwnEntry)
            } else {
                core::ptr::null_mut()
            };
            InternalUnlockCondVar(ConditionVariable, RemoveEntry);
        }
    }

    // Reacquire the caller-provided lock, as we are about to return.
    if CriticalSection.is_null() {
        if (RTL_CONDITION_VARIABLE_LOCKMODE_SHARED & SRWFlags) == 0 {
            RtlAcquireSRWLockExclusive(SRWLock);
        } else {
            RtlAcquireSRWLockShared(SRWLock);
        }
    } else {
        RtlEnterCriticalSection(CriticalSection);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeKeyedEvent() {
    NtCreateKeyedEvent(
        core::ptr::addr_of_mut!(CondVarKeyedEventHandle),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCloseKeyedEvent() {
    NtClose(CondVarKeyedEventHandle);
    CondVarKeyedEventHandle = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE) {
    (*ConditionVariable).Ptr = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn RtlWakeConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE) {
    InternalWake(ConditionVariable, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn RtlWakeAllConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE) {
    InternalWake(ConditionVariable, 1u8);
}

#[no_mangle]
pub unsafe extern "system" fn RtlSleepConditionVariableCS(
    ConditionVariable: PRTL_CONDITION_VARIABLE,
    CriticalSection: PRTL_CRITICAL_SECTION,
    TimeOut: PLARGE_INTEGER,
) -> NTSTATUS {
    InternalSleep(
        ConditionVariable,
        CriticalSection,
        core::ptr::null_mut(),
        0,
        TimeOut,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlSleepConditionVariableSRW(
    ConditionVariable: PRTL_CONDITION_VARIABLE,
    SRWLock: PRTL_SRWLOCK,
    TimeOut: PLARGE_INTEGER,
    Flags: ULONG,
) -> NTSTATUS {
    InternalSleep(
        ConditionVariable,
        core::ptr::null_mut(),
        SRWLock,
        Flags,
        TimeOut,
    )
}
