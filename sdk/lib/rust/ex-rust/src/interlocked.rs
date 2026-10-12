use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};

/// Debug poison value for freed list entries — size-appropriate per architecture.
#[cfg(target_pointer_width = "64")]
const DEAD_ENTRY: usize = 0xBADDD0FFBADDD0FF;
#[cfg(target_pointer_width = "32")]
const DEAD_ENTRY: usize = 0xBADDD0FF;

use ntkrnl_sys::ddk::{
    KeDisableInterrupts,
    KeRestoreInterrupts,
    LARGE_INTEGER,
    PLARGE_INTEGER,
    PLONGLONG,
    PopEntryList,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
type INTERLOCKED_RESULT = LONG;

// x86's INTERLOCKED_RESULT is NOT a sign in {-1, 0, 1}. xdk/x86/ke.h keeps the
// encoding the assembly original returned verbatim — the EFLAGS SIGN/ZERO pair:
// RESULT_NEGATIVE = EFLAG_SIGN = 0x8000, RESULT_ZERO = EFLAG_ZERO = 0x4000,
// RESULT_POSITIVE = 0. The tidy 0/1/2 in exfuncs.h sits under `#ifndef _M_IX86`
// and does not apply here. Nothing in the pipeline catches a wrong value: the
// compiler sees an opaque LONG, and in-tree callers only ever test it against
// ResultZero, so the first witness was ExInterlocked's 26 assertions.
#[cfg(target_arch = "x86")]
const ResultNegative: INTERLOCKED_RESULT = 0x8000;
#[cfg(target_arch = "x86")]
const ResultZero: INTERLOCKED_RESULT = 0x4000;
#[cfg(target_arch = "x86")]
const ResultPositive: INTERLOCKED_RESULT = 0;

// Pin them to the header the bindings were generated from, so a future header
// change is a build error rather than another silent wrong answer.
#[cfg(target_arch = "x86")]
const _: () = {
    assert!(ResultNegative as u32 == ntkrnl_sys::ddk::_INTERLOCKED_RESULT_ResultNegative);
    assert!(ResultZero as u32 == ntkrnl_sys::ddk::_INTERLOCKED_RESULT_ResultZero);
    assert!(ResultPositive as u32 == ntkrnl_sys::ddk::_INTERLOCKED_RESULT_ResultPositive);
};

/// Inline replacement for the C macro PushEntryList.
#[inline(always)]
unsafe fn PushEntryList(ListHead: PSINGLE_LIST_ENTRY, Entry: PSINGLE_LIST_ENTRY) {
    (*Entry).Next = (*ListHead).Next;
    (*ListHead).Next = Entry;
}

/// Inline replacement for the C intrinsic _InterlockedExchange on LONG.
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn _InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Target as *const AtomicI32);
    atomic.swap(Value, Ordering::SeqCst)
}

/// Inline replacement for the C intrinsic _InterlockedCompareExchange64.
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn _InterlockedCompareExchange64(
    Destination: *mut LONGLONG,
    Exchange: LONGLONG,
    Comparand: LONGLONG,
) -> LONGLONG {
    use core::sync::atomic::{AtomicI64, Ordering};
    let atomic = &*(Destination as *const AtomicI64);
    match atomic.compare_exchange(Comparand, Exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

extern "C" {
    fn KxAcquireSpinLock(Lock: PKSPIN_LOCK);
    fn KxReleaseSpinLock(Lock: PKSPIN_LOCK);
}

#[inline(always)]
unsafe fn _ExiDisableInterruptsAndAcquireSpinlock(Lock: PKSPIN_LOCK) -> BOOLEAN {
    let Enabled = KeDisableInterrupts();
    KxAcquireSpinLock(Lock);
    Enabled
}

#[inline(always)]
unsafe fn _ExiReleaseSpinLockAndRestoreInterrupts(Lock: PKSPIN_LOCK, Enable: BOOLEAN) {
    KxReleaseSpinLock(Lock);
    KeRestoreInterrupts(Enable);
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedAddLargeInteger(
    Addend: PLARGE_INTEGER,
    Increment: LARGE_INTEGER,
    Lock: PKSPIN_LOCK,
) -> LARGE_INTEGER {
    let Addend = opaque_ptr(Addend);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let OldValue = LARGE_INTEGER {
        QuadPart: (*Addend).QuadPart,
    };

    (*Addend).QuadPart += Increment.QuadPart;

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    OldValue
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedAddUlong(
    Addend: *mut ULONG,
    Increment: ULONG,
    Lock: PKSPIN_LOCK,
) -> ULONG {
    let Addend = opaque_ptr(Addend);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let OldValue = *Addend;

    *Addend += Increment;

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    OldValue
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedInsertHeadList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PLIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let ListEntry = opaque_ptr(ListEntry);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let FirstEntry = (*ListHead).Flink;

    InsertHeadList(ListHead, ListEntry);

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    if FirstEntry == ListHead {
        core::ptr::null_mut()
    } else {
        FirstEntry
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedInsertTailList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PLIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let ListEntry = opaque_ptr(ListEntry);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let LastEntry = (*ListHead).Blink;

    InsertTailList(ListHead, ListEntry);

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    if LastEntry == ListHead {
        core::ptr::null_mut()
    } else {
        LastEntry
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedRemoveHeadList(
    ListHead: PLIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PLIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let ListEntry = if IsListEmpty(ListHead) {
        core::ptr::null_mut()
    } else {
        let entry = RemoveHeadList(ListHead);
        // The C's `#if DBG`, and this tree builds with DBG=1 (KmtIsCheckedBuild is TRUE):
        // gated on cfg(debug_assertions), which the release-built kernel crates never set,
        // the poison was compiled out and ExDoubleList/ExSingleList saw live links.
        {
            (*entry).Flink = DEAD_ENTRY as PLIST_ENTRY;
            (*entry).Blink = DEAD_ENTRY as PLIST_ENTRY;
        }
        entry
    };

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    ListEntry
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedPopEntryList(
    ListHead: PSINGLE_LIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PSINGLE_LIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let ListEntry = PopEntryList(ListHead as *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY) as PSINGLE_LIST_ENTRY;
    // The C's `#if DBG`, and this tree builds with DBG=1 (KmtIsCheckedBuild is TRUE):
    // gated on cfg(debug_assertions), which the release-built kernel crates never set,
    // the poison was compiled out and ExDoubleList/ExSingleList saw live links.
    {
        if !ListEntry.is_null() {
            (*ListEntry).Next = DEAD_ENTRY as PSINGLE_LIST_ENTRY;
        }
    }

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    ListEntry
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedPushEntryList(
    ListHead: PSINGLE_LIST_ENTRY,
    ListEntry: PSINGLE_LIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PSINGLE_LIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let ListEntry = opaque_ptr(ListEntry);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let OldListEntry = (*ListHead).Next;

    PushEntryList(ListHead, ListEntry);

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    OldListEntry
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn ExInterlockedIncrementLong(
    Addend: *mut LONG,
    Lock: PKSPIN_LOCK,
) -> INTERLOCKED_RESULT {
    let Addend = opaque_ptr(Addend);
    let Result = InterlockedIncrement(Addend);
    if Result < 0 {
        ResultNegative
    } else if Result > 0 {
        ResultPositive
    } else {
        ResultZero
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn ExInterlockedDecrementLong(
    Addend: *mut LONG,
    Lock: PKSPIN_LOCK,
) -> INTERLOCKED_RESULT {
    let Addend = opaque_ptr(Addend);
    let Result = InterlockedDecrement(Addend);
    if Result < 0 {
        ResultNegative
    } else if Result > 0 {
        ResultPositive
    } else {
        ResultZero
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn ExInterlockedExchangeUlong(
    Target: *mut ULONG,
    Value: ULONG,
    Lock: PKSPIN_LOCK,
) -> ULONG {
    let Target = opaque_ptr(Target);
    _InterlockedExchange(Target as *mut LONG, Value as LONG) as ULONG
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInterlockedAddUlong(
    Addend: *mut ULONG,
    Increment: ULONG,
    Lock: PKSPIN_LOCK,
) -> ULONG {
    let Addend = opaque_ptr(Addend);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let OldValue = *Addend;

    *Addend += Increment;

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    OldValue
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInterlockedInsertHeadList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PLIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let ListEntry = opaque_ptr(ListEntry);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let FirstEntry = (*ListHead).Flink;

    InsertHeadList(ListHead, ListEntry);

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    if FirstEntry == ListHead {
        core::ptr::null_mut()
    } else {
        FirstEntry
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInterlockedInsertTailList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PLIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let ListEntry = opaque_ptr(ListEntry);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let LastEntry = (*ListHead).Blink;

    InsertTailList(ListHead, ListEntry);

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    if LastEntry == ListHead {
        core::ptr::null_mut()
    } else {
        LastEntry
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInterlockedRemoveHeadList(
    ListHead: PLIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PLIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let ListEntry = if IsListEmpty(ListHead) {
        core::ptr::null_mut()
    } else {
        let entry = RemoveHeadList(ListHead);
        // The C's `#if DBG`, and this tree builds with DBG=1 (KmtIsCheckedBuild is TRUE):
        // gated on cfg(debug_assertions), which the release-built kernel crates never set,
        // the poison was compiled out and ExDoubleList/ExSingleList saw live links.
        {
            (*entry).Flink = 0x0BADD0FF as PLIST_ENTRY;
            (*entry).Blink = 0x0BADD0FF as PLIST_ENTRY;
        }
        entry
    };

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    ListEntry
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInterlockedPopEntryList(
    ListHead: PSINGLE_LIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PSINGLE_LIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let ListEntry = PopEntryList(ListHead as *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY) as PSINGLE_LIST_ENTRY;
    // The C's `#if DBG`, and this tree builds with DBG=1 (KmtIsCheckedBuild is TRUE):
    // gated on cfg(debug_assertions), which the release-built kernel crates never set,
    // the poison was compiled out and ExDoubleList/ExSingleList saw live links.
    {
        if !ListEntry.is_null() {
            (*ListEntry).Next = DEAD_ENTRY as PSINGLE_LIST_ENTRY;
        }
    }

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    ListEntry
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInterlockedPushEntryList(
    ListHead: PSINGLE_LIST_ENTRY,
    ListEntry: PSINGLE_LIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PSINGLE_LIST_ENTRY {
    let ListHead = opaque_ptr(ListHead);
    let ListEntry = opaque_ptr(ListEntry);
    let Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);

    let OldListEntry = (*ListHead).Next;

    PushEntryList(ListHead, ListEntry);

    _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);

    OldListEntry
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn Exi386InterlockedIncrementLong(
    Addend: *mut LONG,
) -> INTERLOCKED_RESULT {
    let Addend = opaque_ptr(Addend);
    let Result = InterlockedIncrement(Addend);
    if Result < 0 {
        ResultNegative
    } else if Result > 0 {
        ResultPositive
    } else {
        ResultZero
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn Exfi386InterlockedIncrementLong(
    Addend: *mut LONG,
) -> INTERLOCKED_RESULT {
    let Addend = opaque_ptr(Addend);
    let Result = InterlockedIncrement(Addend);
    if Result < 0 {
        ResultNegative
    } else if Result > 0 {
        ResultPositive
    } else {
        ResultZero
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn Exi386InterlockedDecrementLong(
    Addend: *mut LONG,
) -> INTERLOCKED_RESULT {
    let Addend = opaque_ptr(Addend);
    let Result = InterlockedDecrement(Addend);
    if Result < 0 {
        ResultNegative
    } else if Result > 0 {
        ResultPositive
    } else {
        ResultZero
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn Exfi386InterlockedDecrementLong(
    Addend: *mut LONG,
) -> INTERLOCKED_RESULT {
    let Addend = opaque_ptr(Addend);
    let Result = InterlockedDecrement(Addend);
    if Result < 0 {
        ResultNegative
    } else if Result > 0 {
        ResultPositive
    } else {
        ResultZero
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn Exi386InterlockedExchangeUlong(
    Target: *mut LONG,
    Exchange: LONG,
) -> LONG {
    let Target = opaque_ptr(Target);
    _InterlockedExchange(Target, Exchange)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn Exfi386InterlockedExchangeUlong(
    Target: *mut ULONG,
    Exchange: ULONG,
) -> ULONG {
    let Target = opaque_ptr(Target);
    _InterlockedExchange(Target as *mut LONG, Exchange as LONG) as ULONG
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExInterlockedCompareExchange64(
    Destination: *mut LONGLONG,
    Exchange: PLONGLONG,
    Comparand: PLONGLONG,
    Lock: PKSPIN_LOCK,
) -> LONGLONG {
    let Destination = opaque_ptr(Destination);
    _InterlockedCompareExchange64(Destination, *Exchange, *Comparand)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExfInterlockedCompareExchange64(
    Destination: *mut LONGLONG,
    Exchange: PLONGLONG,
    Comparand: PLONGLONG,
) -> LONGLONG {
    let Destination = opaque_ptr(Destination);
    _InterlockedCompareExchange64(Destination, *Exchange, *Comparand)
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/ex/interlocked.c
// * PURPOSE:         Interlocked functions
// * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
// */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #undef ExInterlockedAddUlong
// #undef ExInterlockedInsertHeadList
// #undef ExInterlockedInsertTailList
// #undef ExInterlockedRemoveHeadList
// #undef ExInterlockedPopEntryList
// #undef ExInterlockedPushEntryList
// #undef ExInterlockedIncrementLong
// #undef ExInterlockedDecrementLong
// #undef ExInterlockedExchangeUlong
// #undef ExInterlockedCompareExchange64
//
//
// /* FUNCTIONS ****************************************************************/
//
// FORCEINLINE
// BOOLEAN
// _ExiDisableInterruptsAndAcquireSpinlock(
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enabled;
//
//     /* Disable interrupts */
//     Enabled = KeDisableInterrupts();
//
//     /* Acquire the spinlock (inline) */
//     KxAcquireSpinLock(Lock);
//
//     return Enabled;
// }
//
// FORCEINLINE
// VOID
// _ExiReleaseSpinLockAndRestoreInterrupts(
//     IN OUT PKSPIN_LOCK Lock,
//     BOOLEAN Enable)
// {
//     /* Release the spinlock */
//     KxReleaseSpinLock(Lock);
//
//     /* Restore interrupts */
//     KeRestoreInterrupts(Enable);
// }
//
//
// LARGE_INTEGER
// NTAPI
// ExInterlockedAddLargeInteger(
//     IN OUT PLARGE_INTEGER Addend,
//     IN LARGE_INTEGER Increment,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     LARGE_INTEGER OldValue;
//     BOOLEAN Enable;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the old value */
//     OldValue.QuadPart = Addend->QuadPart;
//
//     /* Do the operation */
//     Addend->QuadPart += Increment.QuadPart;
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the old value */
//     return OldValue;
// }
//
// ULONG
// NTAPI
// ExInterlockedAddUlong(
//     IN OUT PULONG Addend,
//     IN ULONG Increment,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     ULONG OldValue;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the old value */
//     OldValue = *Addend;
//
//     /* Do the operation */
//     *Addend += Increment;
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the old value */
//     return OldValue;
// }
//
// PLIST_ENTRY
// NTAPI
// ExInterlockedInsertHeadList(
//     IN OUT PLIST_ENTRY ListHead,
//     IN OUT PLIST_ENTRY ListEntry,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PLIST_ENTRY FirstEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the first entry */
//     FirstEntry = ListHead->Flink;
//
//     /* Insert the new entry */
//     InsertHeadList(ListHead, ListEntry);
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the old first entry or NULL for empty list */
//     return (FirstEntry == ListHead) ? NULL : FirstEntry;
// }
//
// PLIST_ENTRY
// NTAPI
// ExInterlockedInsertTailList(
//     IN OUT PLIST_ENTRY ListHead,
//     IN OUT PLIST_ENTRY ListEntry,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PLIST_ENTRY LastEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the last entry */
//     LastEntry = ListHead->Blink;
//
//     /* Insert the new entry */
//     InsertTailList(ListHead, ListEntry);
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the old last entry or NULL for empty list */
//     return (LastEntry == ListHead) ? NULL : LastEntry;
// }
//
// PLIST_ENTRY
// NTAPI
// ExInterlockedRemoveHeadList(
//     IN OUT PLIST_ENTRY ListHead,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PLIST_ENTRY ListEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Check if the list is empty */
//     if (IsListEmpty(ListHead))
//     {
//         /* Return NULL */
//         ListEntry = NULL;
//     }
//     else
//     {
//         /* Remove the first entry from the list head */
//         ListEntry = RemoveHeadList(ListHead);
// #if DBG
//         ListEntry->Flink = (PLIST_ENTRY)(ULONG_PTR)0xBADDD0FFBADDD0FFULL;
//         ListEntry->Blink = (PLIST_ENTRY)(ULONG_PTR)0xBADDD0FFBADDD0FFULL;
// #endif
//     }
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the entry */
//     return ListEntry;
// }
//
// PSINGLE_LIST_ENTRY
// NTAPI
// ExInterlockedPopEntryList(
//     IN OUT PSINGLE_LIST_ENTRY ListHead,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PSINGLE_LIST_ENTRY ListEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Pop the first entry from the list */
//     ListEntry = PopEntryList(ListHead);
// #if DBG
//     if (ListEntry)
//         ListEntry->Next = (PSINGLE_LIST_ENTRY)(ULONG_PTR)0xBADDD0FFBADDD0FFULL;
// #endif
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the entry */
//     return ListEntry;
// }
//
// PSINGLE_LIST_ENTRY
// NTAPI
// ExInterlockedPushEntryList(
//     IN OUT PSINGLE_LIST_ENTRY ListHead,
//     IN OUT PSINGLE_LIST_ENTRY ListEntry,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PSINGLE_LIST_ENTRY OldListEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the old top entry */
//     OldListEntry = ListHead->Next;
//
//     /* Push a new entry on the list */
//     PushEntryList(ListHead, ListEntry);
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the entry */
//     return OldListEntry;
// }
//
// INTERLOCKED_RESULT
// NTAPI
// ExInterlockedIncrementLong(
//   IN PLONG Addend,
//   IN PKSPIN_LOCK Lock)
// {
//     LONG Result;
//
//     Result = InterlockedIncrement(Addend);
//     return (Result < 0) ? ResultNegative :
//            (Result > 0) ? ResultPositive :
//            ResultZero;
// }
//
// INTERLOCKED_RESULT
// NTAPI
// ExInterlockedDecrementLong(
//   IN PLONG Addend,
//   IN PKSPIN_LOCK Lock)
// {
//     LONG Result;
//
//     Result = InterlockedDecrement(Addend);
//     return (Result < 0) ? ResultNegative :
//            (Result > 0) ? ResultPositive :
//            ResultZero;
// }
//
// ULONG
// NTAPI
// ExInterlockedExchangeUlong(
//   IN PULONG Target,
//   IN ULONG Value,
//   IN PKSPIN_LOCK Lock)
// {
//     return (ULONG)_InterlockedExchange((PLONG)Target, (LONG)Value);
// }
//
// #ifdef _M_IX86
//
// ULONG
// FASTCALL
// ExfInterlockedAddUlong(
//     IN OUT PULONG Addend,
//     IN ULONG Increment,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     ULONG OldValue;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the old value */
//     OldValue = *Addend;
//
//     /* Do the operation */
//     *Addend += Increment;
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the old value */
//     return OldValue;
// }
//
// PLIST_ENTRY
// FASTCALL
// ExfInterlockedInsertHeadList(
//     IN OUT PLIST_ENTRY ListHead,
//     IN PLIST_ENTRY ListEntry,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PLIST_ENTRY FirstEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the first entry */
//     FirstEntry = ListHead->Flink;
//
//     /* Insert the new entry */
//     InsertHeadList(ListHead, ListEntry);
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the old first entry or NULL for empty list */
//     return (FirstEntry == ListHead) ? NULL : FirstEntry;
// }
//
// PLIST_ENTRY
// FASTCALL
// ExfInterlockedInsertTailList(
//     IN OUT PLIST_ENTRY ListHead,
//     IN PLIST_ENTRY ListEntry,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PLIST_ENTRY LastEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the last entry */
//     LastEntry = ListHead->Blink;
//
//     /* Insert the new entry */
//     InsertTailList(ListHead, ListEntry);
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* Return the old last entry or NULL for empty list */
//     return (LastEntry == ListHead) ? NULL : LastEntry;
// }
//
//
// PLIST_ENTRY
// FASTCALL
// ExfInterlockedRemoveHeadList(
//     IN OUT PLIST_ENTRY ListHead,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PLIST_ENTRY ListEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Check if the list is empty */
//     if (IsListEmpty(ListHead))
//     {
//         /* Return NULL */
//         ListEntry = NULL;
//     }
//     else
//     {
//         /* Remove the first entry from the list head */
//         ListEntry = RemoveHeadList(ListHead);
// #if DBG
//         ListEntry->Flink = (PLIST_ENTRY)0x0BADD0FF;
//         ListEntry->Blink = (PLIST_ENTRY)0x0BADD0FF;
// #endif
//     }
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* return the entry */
//     return ListEntry;
// }
//
// PSINGLE_LIST_ENTRY
// FASTCALL
// ExfInterlockedPopEntryList(
//     IN OUT PSINGLE_LIST_ENTRY ListHead,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PSINGLE_LIST_ENTRY ListEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Pop the first entry from the list */
//     ListEntry = PopEntryList(ListHead);
// #if DBG
//     if (ListEntry)
//         ListEntry->Next = (PSINGLE_LIST_ENTRY)(ULONG_PTR)0xBADDD0FFBADDD0FFULL;
// #endif
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* return the entry */
//     return ListEntry;
// }
//
// PSINGLE_LIST_ENTRY
// FASTCALL
// ExfInterlockedPushEntryList(
//     IN OUT PSINGLE_LIST_ENTRY ListHead,
//     IN PSINGLE_LIST_ENTRY ListEntry,
//     IN OUT PKSPIN_LOCK Lock)
// {
//     BOOLEAN Enable;
//     PSINGLE_LIST_ENTRY OldListEntry;
//
//     /* Disable interrupts and acquire the spinlock */
//     Enable = _ExiDisableInterruptsAndAcquireSpinlock(Lock);
//
//     /* Save the old top entry */
//     OldListEntry = ListHead->Next;
//
//     /* Push a new entry on the list */
//     PushEntryList(ListHead, ListEntry);
//
//     /* Release the spinlock and restore interrupts */
//     _ExiReleaseSpinLockAndRestoreInterrupts(Lock, Enable);
//
//     /* return the entry */
//     return OldListEntry;
// }
//
// INTERLOCKED_RESULT
// NTAPI
// Exi386InterlockedIncrementLong(
//     IN PLONG Addend)
// {
//     LONG Result;
//
//     Result = InterlockedIncrement(Addend);
//     return (Result < 0) ? ResultNegative :
//            (Result > 0) ? ResultPositive :
//            ResultZero;
// }
//
// INTERLOCKED_RESULT
// FASTCALL
// Exfi386InterlockedIncrementLong(
//     IN OUT LONG volatile *Addend)
// {
//     LONG Result;
//
//     Result = InterlockedIncrement(Addend);
//     return (Result < 0) ? ResultNegative :
//            (Result > 0) ? ResultPositive :
//            ResultZero;
// }
//
// INTERLOCKED_RESULT
// NTAPI
// Exi386InterlockedDecrementLong(
//     IN PLONG Addend)
// {
//     LONG Result;
//
//     Result = InterlockedDecrement(Addend);
//     return (Result < 0) ? ResultNegative :
//            (Result > 0) ? ResultPositive :
//            ResultZero;
// }
//
// INTERLOCKED_RESULT
// FASTCALL
// Exfi386InterlockedDecrementLong(
//     IN OUT PLONG Addend)
// {
//     LONG Result;
//
//     Result = InterlockedDecrement(Addend);
//     return (Result < 0) ? ResultNegative :
//            (Result > 0) ? ResultPositive :
//            ResultZero;
// }
//
// LONG
// NTAPI
// Exi386InterlockedExchangeUlong(
//     PLONG Target,
//     LONG Exchange)
// {
//     return _InterlockedExchange(Target, Exchange);
// }
//
// ULONG
// FASTCALL
// Exfi386InterlockedExchangeUlong(
//     IN OUT PULONG Target,
//     IN ULONG Exchange)
// {
//     return _InterlockedExchange((PLONG)Target, Exchange);
// }
//
// LONGLONG
// FASTCALL
// ExInterlockedCompareExchange64(
//     IN OUT LONGLONG volatile *Destination,
//     IN PLONGLONG Exchange,
//     IN PLONGLONG Comparand,
//     IN PKSPIN_LOCK Lock)
// {
//     return _InterlockedCompareExchange64(Destination, *Exchange, *Comparand);
// }
//
// LONGLONG
// FASTCALL
// ExfInterlockedCompareExchange64(
//     IN OUT LONGLONG volatile *Destination,
//     IN PLONGLONG Exchange,
//     IN PLONGLONG Comparand)
// {
//     return _InterlockedCompareExchange64(Destination, *Exchange, *Comparand);
// }
// #endif
//
// #if 0
//
// VOID
// FASTCALL
// ExInterlockedAddLargeStatistic(
//     IN OUT PLARGE_INTEGER Addend,
//     IN ULONG Increment)
// {
// }
//
//
// #endif
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
